#!/usr/bin/env bash
set -euo pipefail

# bardo-ctl.sh — Launch the Mori TUI (Ratatui-based pipeline monitor)
#
# Usage: ./mori.sh [--gateway|--no-gateway] <plan-spec> [extra-flags...]
#
#   Plan specs:  "02"  "01-09"  "08a-08d"  (required)
#
# Gateway flag:
#   --gateway      Route agent inference through bardo-gateway (default: auto-detect)
#   --no-gateway   Bypass gateway, agents talk directly to Anthropic/OpenAI
#
#   Auto-detect: if bardo-gateway is listening on port 4000, routes through it.
#   Set BARDO_GATEWAY_API_KEY in .env or environment (default: mori-local-gateway).
#
# Default flags applied automatically:
#   --parallel     Run plans in parallel within each wave
#   --pre-plan     Speculatively prepare briefs for upcoming waves
#
# Override examples:
#   ./mori.sh 02-05 --no-review        skip architect/auditor/scribe/critic
#   ./mori.sh 02-05 --skip-tests       skip cargo test gate
#   ./mori.sh 02-05 --max-agents 4     cap concurrency at 4 agents
#   ./mori.sh 02-05 --refactor         enable post-plan refactoring passes
#
# Context files written/read by agents (all relative to repo root):
#   plans/context/briefs/          — strategist briefs
#   plans/context/reviews/         — review verdicts
#   plans/context/tasks/           — task checklists (TOML)
#   plans/context/docs/            — scribe documentation
#   plans/context/workspace-map.md — crate file tree
#   tmp/plan-runs/bardo-ctl.log    — runtime log
#
# Key bindings:
#   s — start pipeline    q — quit
#   i — inject message    r — resume
#   ↑↓ — select plan       enter — view full log

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTL_DIR="${SCRIPT_DIR}/apps/mori"

# ── Source .env if present ──────────────────────────────────────────
if [[ -f "${SCRIPT_DIR}/.env" ]]; then
  set -a
  source "${SCRIPT_DIR}/.env"
  set +a
fi

# ── Parse gateway flag (strip before passing to mori) ──────────────
GATEWAY_MODE="auto"
PASSTHROUGH_ARGS=()
for arg in "$@"; do
  case "$arg" in
    --gateway)    GATEWAY_MODE="on" ;;
    --no-gateway) GATEWAY_MODE="off" ;;
    *)            PASSTHROUGH_ARGS+=("$arg") ;;
  esac
done
set -- "${PASSTHROUGH_ARGS[@]+"${PASSTHROUGH_ARGS[@]}"}"

# ── Gateway setup ──────────────────────────────────────────────────
GATEWAY_PORT="${BARDO_GATEWAY_PORT:-4000}"
GATEWAY_URL="http://127.0.0.1:${GATEWAY_PORT}"
GATEWAY_KEY="${BARDO_GATEWAY_API_KEY:-mori-local-gateway}"

setup_gateway() {
  export ANTHROPIC_BASE_URL="$GATEWAY_URL"
  export BARDO_GATEWAY_API_KEY="$GATEWAY_KEY"
  echo "  gateway: ${GATEWAY_URL} (key=${GATEWAY_KEY:0:8}...)"
}

if [[ "$GATEWAY_MODE" == "on" ]]; then
  setup_gateway
elif [[ "$GATEWAY_MODE" == "auto" ]]; then
  # Auto-detect: check if gateway is listening
  if curl -s --connect-timeout 1 "${GATEWAY_URL}/v1/health" >/dev/null 2>&1; then
    setup_gateway
  fi
  # If not running, agents go direct — no error
elif [[ "$GATEWAY_MODE" == "off" ]]; then
  unset ANTHROPIC_BASE_URL
fi

# ── Sanity checks ──────────────────────────────────────────────────
if [[ ! -f "${SCRIPT_DIR}/Cargo.toml" ]]; then
  echo "error: expected Cargo.toml at ${SCRIPT_DIR}"
  echo "  This script must live in the repository root."
  exit 1
fi

if [[ ! -d "$CTL_DIR" ]]; then
  echo "error: mori not found at ${CTL_DIR}"
  echo "  Build it first: cargo build -p mori --release"
  exit 1
fi

# Ensure the context directory tree exists before agents try to write into it.
mkdir -p \
  "${SCRIPT_DIR}/plans/context/briefs" \
  "${SCRIPT_DIR}/plans/context/reviews" \
  "${SCRIPT_DIR}/plans/context/tasks" \
  "${SCRIPT_DIR}/plans/context/docs" \
  "${SCRIPT_DIR}/plans/context/summaries" \
  "${SCRIPT_DIR}/plans/context/archive" \
  "${SCRIPT_DIR}/tmp/plan-runs"

# Refresh repo-wide context files for agents (workspace map + preflight snapshot).
if [[ -x "${SCRIPT_DIR}/scripts/bardo-sync-context.sh" ]]; then
  SKIP_CARGO_CHECK="${BARDO_SYNC_CONTEXT_SKIP_CARGO:-}" \
    bash "${SCRIPT_DIR}/scripts/bardo-sync-context.sh" "${SCRIPT_DIR}" || true
fi

# ── Launch ─────────────────────────────────────────────────────────
DEFAULT_FLAGS=(--parallel --pre-plan)

RELEASE_BIN="${SCRIPT_DIR}/target/release/mori"
if [[ -x "$RELEASE_BIN" ]]; then
  exec "$RELEASE_BIN" --repo-root "$SCRIPT_DIR" "${DEFAULT_FLAGS[@]}" "$@"
fi

exec cargo run -p mori --release -- --repo-root "$SCRIPT_DIR" "${DEFAULT_FLAGS[@]}" "$@"

#!/usr/bin/env bash
set -euo pipefail

# bardo-ctl.sh — Launch the Mori TUI (Ratatui-based pipeline monitor)
#
# Usage: ./bardo-ctl.sh <plan-spec> [extra-flags...]
#        ./mori.sh <plan-spec> [extra-flags...]        (symlink)
#
#   Plan specs:  "02"  "01-09"  "08a-08d"  (required)
#
# Default flags applied automatically:
#   --parallel     Run plans in parallel within each wave (same-wave plans run concurrently)
#   --pre-plan     Speculatively prepare briefs for upcoming waves while the current runs
#
# Override examples:
#   ./mori.sh 02-05 --no-review        skip architect/auditor/scribe/critic
#   ./mori.sh 02-05 --skip-tests       skip cargo test gate
#   ./mori.sh 02-05 --max-agents 4     cap concurrency at 4 agents
#   ./mori.sh 02-05 --refactor         enable post-plan refactoring passes
#
# Must be run from the monorepo root or any subdirectory — SCRIPT_DIR is
# always resolved to the directory containing this script (= repo root).
# The --repo-root flag is passed explicitly so mori never has to guess.
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

# Sanity-check: ensure we resolved the right directory.
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

# Default flags — enable parallel wave execution and speculative pre-planning.
DEFAULT_FLAGS=(--parallel --pre-plan)

# Use the workspace release binary (built via `cargo build -p mori --release`).
RELEASE_BIN="${SCRIPT_DIR}/target/release/mori"
if [[ -x "$RELEASE_BIN" ]]; then
  exec "$RELEASE_BIN" --repo-root "$SCRIPT_DIR" "${DEFAULT_FLAGS[@]}" "$@"
fi

# Fallback: build and run via cargo from workspace root.
exec cargo run -p mori --release -- --repo-root "$SCRIPT_DIR" "${DEFAULT_FLAGS[@]}" "$@"

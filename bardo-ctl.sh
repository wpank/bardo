#!/usr/bin/env bash
set -euo pipefail

# bardo-ctl.sh — Launch the Bardo TUI (Ratatui-based pipeline monitor)
#
# Usage: ./bardo-ctl.sh [args...]
#
# The TUI lives at tmp/bardo-ctl/ (gitignored). It reads:
#   tmp/plan-runs/status.json     — current pipeline state
#   tmp/plan-runs/events.jsonl    — event stream
#   tmp/plan-runs/active-run.log  — log tail
#   tmp/plan-runs/current-plan.txt — active plan name
#   tmp/anvil.pid                 — anvil status
#
# Key bindings:
#   s — start pipeline    q — quit
#   i — inject message    r — resume
#   ↑↓ — select plan       enter — view full log

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TUI_DIR="${SCRIPT_DIR}/tmp/bardo-ctl"

if [[ ! -d "$TUI_DIR" ]]; then
  echo "TUI not found at $TUI_DIR"
  echo "The TUI source is at tmp/bardo-ctl/ — build it with: cd tmp/bardo-ctl && cargo build --release"
  exit 1
fi

cd "$TUI_DIR"
cargo run --release -- --repo-root "$SCRIPT_DIR" "$@"

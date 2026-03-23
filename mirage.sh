#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"

if [[ -f "$ENV_FILE" ]]; then
  # shellcheck source=/dev/null
  source "$ENV_FILE"
fi

PORT="${MIRAGE_PORT:-18545}"
EXTRA_ARGS=()

if [[ -n "${MIRAGE_RPC_URL:-}" ]]; then
  EXTRA_ARGS+=(--rpc-url "$MIRAGE_RPC_URL")
fi
if [[ -n "${MIRAGE_WS_URL:-}" ]]; then
  EXTRA_ARGS+=(--ws-url "$MIRAGE_WS_URL")
fi
if [[ -n "${MIRAGE_UPSTREAM_RPS:-}" ]]; then
  EXTRA_ARGS+=(--upstream-rps "$MIRAGE_UPSTREAM_RPS")
fi
if [[ -n "${MIRAGE_UPSTREAM_BURST:-}" ]]; then
  EXTRA_ARGS+=(--upstream-burst "$MIRAGE_UPSTREAM_BURST")
fi

cd "$SCRIPT_DIR"
exec cargo run -p mirage-rs -- --port "$PORT" "${EXTRA_ARGS[@]}" "$@"

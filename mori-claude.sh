#!/usr/bin/env bash
# Launch Claude Code routed through the mori gateway.
# Usage: ./mori-claude.sh
# All args are passed through to claude.
set -e
cd "$(dirname "$0")"
source .env 2>/dev/null || true

export ANTHROPIC_BASE_URL="${BARDO_GATEWAY_URL:-http://localhost:4000}"
export ANTHROPIC_API_KEY="${BARDO_GATEWAY_API_KEY:-mori-local-gateway}"

exec claude "$@"

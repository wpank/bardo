#!/usr/bin/env bash
# Start the mori inference gateway.
# Usage: ./mori-gateway.sh [--port 4000]
set -e
cd "$(dirname "$0")"
source .env 2>/dev/null || true

PORT="${1:-4000}"
if [ "$1" = "--port" ]; then PORT="$2"; fi

echo "Starting mori gateway on port $PORT..."
exec cargo run -p bardo-gateway --release -- \
  --port "$PORT" \
  --api-key "${BARDO_GATEWAY_API_KEY:-mori-local-gateway}"

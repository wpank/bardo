#!/usr/bin/env bash
# Query Uniswap V3 ETH/USDC 0.05% pool slot 0 at t=0, 12, 24, 36s.
# Slot 0 packs sqrtPriceX96 + tick + observationIndex + etc.
# If the fork is following the chain, the value should change over time.
set -euo pipefail

PORT="${MIRAGE_PORT:-18545}"
RPC="http://127.0.0.1:${PORT}"
POOL="0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"

rpc() {
  curl -s -X POST "$RPC" \
    -H "Content-Type: application/json" \
    -d "$1"
}

block_number() {
  rpc '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | python3 -c "import sys,json; d=json.load(sys.stdin); print(int(d['result'],16))"
}

slot0() {
  rpc "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getStorageAt\",\"params\":[\"${POOL}\",\"0x0\",\"latest\"],\"id\":1}" \
    | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['result'])"
}

echo "Checking mirage is up at port ${PORT}..."
if ! curl -sf "$RPC" -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":0}' > /dev/null; then
  echo "ERROR: mirage not responding at $RPC"
  exit 1
fi

echo "Pool: $POOL (Uniswap V3 ETH/USDC 0.05%)"
echo "---"

for t in 0 12 24 36; do
  if [ "$t" -gt 0 ]; then
    echo "Sleeping ${t}s from start..."
    sleep "$t"
  fi
  BLOCK=$(block_number 2>/dev/null || echo "error")
  SLOT=$(slot0 2>/dev/null || echo "error")
  echo "t=+${t}s  block=${BLOCK}  slot0=${SLOT}"
done

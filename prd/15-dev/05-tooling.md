# Developer Tooling

**Version:** 2.0.0
**Last Updated:** 2026-03-16
**Crate:** `mirage-rs/`

---

> **Reader orientation:** This document covers the developer CLI and Rust build tooling for Bardo's local development environment (section 15). The primary interface is the `mirage-rs` CLI binary, which forks any EVM chain and exposes a JSON-RPC endpoint that existing tools (Foundry cast, viem, wagmi) connect to without changes. The document also covers the justfile for common workflows, Rust toolchain setup (cargo, clippy, rustfmt, cargo-nextest), and environment requirements. See `prd2/shared/glossary.md` for full term definitions.

## Overview

Developer tooling is Rust-native. The primary interface is the `mirage-rs` CLI binary, supplemented by a `justfile` for common workflows and standard Rust tooling (cargo, clippy, rustfmt, cargo-nextest).

---

## mirage-rs CLI

```bash
# Live fork -- the common case
mirage-rs --rpc-url wss://eth-mainnet.g.alchemy.com/v2/KEY --follow

# Fresh testnet
mirage-rs --fresh --chain-id 31337

# With deployment and seeding (target, not yet built)
mirage-rs --fresh --deploy --seed

# Full configuration
mirage-rs \
  --rpc-url $RPC_URL \
  --fork-block 21000000 \
  --follow \
  --port 8546 \
  --filter-address 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640 \
  --filter-selector 0x128acb08 \
  --sim-gas \
  --divergence-check \
  --trace-prefetch \
  --log-format json
```

### Flags

| Flag | Default | Description |
| ---- | ------- | ----------- |
| `--rpc-url` | (required for fork) | Upstream RPC endpoint (HTTP or WSS) |
| `--fork-block` | latest | Block number to fork at |
| `--follow` | false | Enable live block following |
| `--fresh` | false | Fresh testnet, no upstream fork |
| `--chain-id` | from upstream | Override chain ID |
| `--port` | 8546 | JSON-RPC server port |
| `--host` | 127.0.0.1 | Bind address |
| `--filter-address` | (none) | Only replay txs touching this address (repeatable) |
| `--filter-selector` | (none) | Only replay txs with this 4-byte selector (repeatable) |
| `--sim-gas` | false | Deterministic gas price simulation |
| `--divergence-check` | false | Compare local vs mainnet tx outcomes |
| `--trace-prefetch` | false | Use `debug_traceBlockByNumber` for state prefetch |
| `--max-lag` | 50 | Block count before skipping ahead |
| `--block-budget` | 30 | Seconds per block before timeout |
| `--log-format` | text | Log output: `text` or `json` |

---

## Status Reporting

mirage-rs writes `.mirage/status.json` every 2 seconds:

```json
{
  "chain_id": 1,
  "fork_block": 21000000,
  "current_block": 21000142,
  "blocks_processed": 142,
  "txs_replayed": 3847,
  "txs_succeeded": 3801,
  "txs_failed": 46,
  "divergence_count": 12,
  "total_gas_used": 1284739200,
  "following": true,
  "uptime_secs": 1723
}
```

The debug UI reads this for its header display. External tooling can poll it for monitoring.

---

## Interacting with mirage-rs

### cast (Foundry)

```bash
# Check block number
cast block-number --rpc-url http://localhost:8546

# Send a swap
cast send $ROUTER "execute(bytes,bytes[],uint256)" \
  $commands $inputs $(date -v+30M +%s) \
  --rpc-url http://localhost:8546

# Set balance
cast rpc mirage_setBalance "0xYourAddress" "0x56BC75E2D63100000"

# Mint USDC
cast rpc mirage_mintERC20 \
  "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" \
  "0xYourAddress" \
  "0x5F5E100" \
  null

# Take snapshot
cast rpc evm_snapshot

# Mine blocks
cast rpc evm_mine
```

### viem (TypeScript)

```typescript
import { createPublicClient, http } from "viem";

const client = createPublicClient({
  transport: http("http://localhost:8546"),
});

const blockNumber = await client.getBlockNumber();
```

Any viem/wagmi/ethers client works -- mirage-rs speaks standard JSON-RPC.

---

## Rust Build Tooling

### justfile

```bash
just build          # cargo build --workspace
just test           # cargo nextest run --workspace
just lint           # cargo clippy --workspace --all-features -- -D warnings
just fmt            # cargo fmt --all
just fmt-check      # cargo fmt --all -- --check
just coverage       # cargo llvm-cov nextest --workspace
just doc            # cargo doc --workspace --no-deps --open
```

### cargo

```bash
cargo build                                   # Debug build
cargo build --release                         # Optimized build
cargo run -- --rpc-url $RPC_URL --follow      # Run directly
cargo test                                    # Run tests
```

### clippy

```bash
cargo clippy --all-features -- -D warnings
```

Pedantic lint group for library code. `clippy.toml` at workspace root.

### rustfmt

```bash
cargo fmt --all              # Format in-place
cargo fmt --all -- --check   # CI check
```

`rustfmt.toml`: edition 2024, 100-char line width, crate-level import granularity.

### cargo-audit

```bash
cargo audit        # Check for known vulnerabilities
```

### cargo-deny

```bash
cargo deny check   # License + advisory + bans + sources
```

Allowed licenses: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC. GPL/AGPL banned.

---

## Environment

### Required

- Rust stable (via `rust-toolchain.toml`)
- An RPC endpoint for fork mode (Alchemy, Infura, or any EVM provider)
- WSS endpoint preferred for live following (HTTP polling works but is slower)

### Optional

- Node.js + pnpm for the debug UI (`packages/dev/ui/`)
- `debug_*` namespace support on the RPC provider for trace-based prefetching
- Foundry (cast) for convenient RPC interaction from the command line

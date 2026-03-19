# mirage-rs

mirage-rs is Bardo's local EVM fork sidecar. It forks Ethereum state from an upstream RPC at a pinned block number, then serves a JSON-RPC interface that golems and tests use for simulation, speculative execution, and scenario backtesting — without touching mainnet.

## Features

- Fork Ethereum state from any RPC-compatible node at a pinned block
- Serve standard `eth_*` JSON-RPC methods locally
- Copy-on-write state branches for speculative execution without committing to the fork
- Scenario runner for simulation and backtesting with TOML fixture assertions
- Memory pressure management with automatic cache eviction, slot-only demotion, and tiered throttling
- Fast-fail startup gating that exits with code `2` when available system memory is below the selected profile plus 128 MB of headroom
- Hardhat/Anvil-compatible helpers for balance manipulation, code injection, impersonation, and time control
- Live event subscriptions over WebSocket-backed `/events/:stream_id` streams
- `mirage_*` custom methods for watchlists, minting, position queries, events, scenarios, and shutdown

## Getting Started

mirage-rs runs as a standalone binary or as an embedded library inside a golem process.

### As a standalone sidecar

Set the required environment variables and start the server:

```bash
export MIRAGE_RPC_URL=https://eth-mainnet.example.com
export MIRAGE_FORK_BLOCK=20000000
cargo run -p mirage-rs
```

The server listens on `http://127.0.0.1:8545` by default.
If the host machine does not have enough free memory for the selected profile, mirage-rs exits before binding the port and returns exit code `2`.

### As a library

```rust
use mirage_rs::{MirageConfig, MirageFork};

let config = MirageConfig::from_env()?;
let fork = MirageFork::new(config).await?;
let client = fork.client();
```

## Configuration

| Variable | Description | Default |
|---|---|---|
| `MIRAGE_RPC_URL` | Upstream HTTP JSON-RPC endpoint | required |
| `MIRAGE_WS_URL` | Upstream WebSocket endpoint (optional) | — |
| `MIRAGE_FORK_BLOCK` | Block number to pin the fork at | latest |
| `MIRAGE_UPSTREAM_RPS` | Max upstream requests per second | 100 |
| `MIRAGE_UPSTREAM_BURST` | Max upstream burst allowance | 200 |

## RPC Surface

### Standard Ethereum methods

- `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getBlockByHash`
- `eth_getBalance`, `eth_getCode`, `eth_getStorageAt`, `eth_getTransactionCount`
- `eth_call`, `eth_estimateGas`
- `eth_sendTransaction`, `eth_sendRawTransaction` (legacy, EIP-2930, EIP-1559, EIP-4844)
- `eth_getTransactionReceipt`, `eth_getLogs`

### EVM control helpers (Hardhat/Anvil compatible)

- `evm_snapshot` / `evm_revert` — save and restore fork state
- `evm_mine` — produce a synthetic block
- `evm_increaseTime` / `evm_setNextBlockTimestamp` — manipulate block timestamps
- `hardhat_setBalance`, `hardhat_setCode`, `hardhat_setStorageAt`, `hardhat_setNonce`
- `hardhat_impersonateAccount` / `hardhat_stopImpersonatingAccount`

### mirage_* methods

- `mirage_status` — current fork status and upstream connection health
- `mirage_resources` — memory, cache size, and upstream usage snapshot
- `mirage_watchlist` — manage the address watchlist
- `mirage_mint` — mint ERC-20 tokens to an address in the local fork
- `mirage_getPosition` — query token balances and protocol positions
- `mirage_subscribeEvents` — register a live log stream and receive events over WebSocket
- `mirage_compareScenarios` — rank completed scenarios by profit, gas, and state-diff footprint
- `mirage_scenarios` — list and run scenario sets
- `mirage_shutdown` — graceful shutdown

## Key Types

- `MirageClient` — JSON-RPC client for use by golems and tests
- `MirageConfig` — connection settings, derived from `GolemConfig` or environment variables
- `MirageFork` — in-process handle around shared fork state
- `ForkState` — lazy three-tier state model (dirty store → local cache → upstream)
- `CowState` — copy-on-write branch for speculative and scenario execution
- `ScenarioRunner` / `ScenarioResult` / `ScenarioAssertions` — scenario execution and TOML fixture assertions
- `ResourceUsage` — snapshot of memory, cache, and upstream usage

## Architecture

mirage-rs has three layers:

**State layer.** `ForkState` implements a lazy three-tier hierarchy. Dirty (locally-modified) state takes precedence, then a populated local cache, then upstream RPC. Writes go to the dirty store only. Copy-on-write branches (`CowState`) let speculative execution proceed without mutating the main fork.

**Execution layer.** Incoming `eth_call` and `eth_sendTransaction` requests run through a local execution engine that handles ETH transfers, ERC-20 operations, and protocol-shaped calldata. Transactions produce synthetic blocks and receipts. The execution engine recognizes common Uniswap and Aave selectors for scenario compatibility.

**Scenario layer.** `ScenarioRunner` loads scenario sets from TOML files. Each scenario gets a fresh `CowState` branch so scenarios run without polluting each other or the base fork. Scenario assertions are evaluated against declared TOML fixtures after execution.

**Events layer.** `mirage-rs` exposes live event streams over WebSocket. Clients register a filtered stream ID over JSON-RPC, then connect to `/events/:stream_id` to receive matching log events as they are produced.

Memory pressure is monitored continuously. At warning tier the cache is trimmed. At throttle tier the cache is trimmed and newly classified contracts are demoted to slot-only reads while new scenario forks remain available. Under emergency pressure the fork drops to `Proxy` mode, the targeted follower stops replaying, and reads continue through upstream.

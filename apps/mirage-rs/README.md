# mirage-rs

**Standalone** | MIT/Apache-2.0 | zero required internal deps

## Use this standalone

```toml
[dependencies]
# As a library (no binary entrypoint, no golem-core dependency)
mirage-rs = { git = "https://github.com/uniswap/bardo", path = "apps/mirage-rs", default-features = false, features = ["library"] }
```

With `default-features = false`, mirage-rs has zero internal workspace dependencies. The optional `golem` feature adds a `GolemConfig` integration convenience, but it's off by default for standalone use.

---

A local Ethereum node for development and testing, like [Anvil](https://getfoundry.sh/reference/anvil/) — but connected to live chains. mirage-rs forks mainnet state lazily over RPC, keeps watched contracts in sync block-by-block, and gives you the full `eth_*` / `evm_*` / `anvil_*` manipulation API you already know. No full node sync. Instant startup.

Where Anvil forks at a pinned block and stays there, mirage-rs optionally follows the chain forward, selectively replaying transactions that touch your contracts so the local view stays current as the market moves.

```bash
# Drop-in replacement for Anvil — fork mainnet on port 8545
mirage-rs --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# With live following over WebSocket
mirage-rs \
  --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  --ws-url wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Isolated mode (no upstream, all accounts start with 1 ETH)
mirage-rs
```

Point any Ethers/Viem/Alloy client at `http://127.0.0.1:8545` and it works the same as Anvil. Your Hardhat tests, Foundry scripts, and custom tooling need zero changes.

## Why not just use Anvil?

Anvil forks at a block and freezes. That's fine for unit tests, but it can't answer questions like: *what happens to my Uniswap position after my transaction, as the next 10 blocks of real market activity play out?*

mirage-rs can, because it:

- **Follows the chain.** A targeted follower subscribes to `newHeads` via WebSocket, filters each block for transactions that touch watched contracts, and replays only those locally. For a typical portfolio of 3-10 DeFi positions, that's ~5-15 transactions per block instead of ~150.
- **Classifies contracts automatically.** When a transaction writes 3+ storage slots on a new address, the diff classifier promotes it to the watch list. Simple token transfers (1-2 slots) get slot-level overrides without full tracking. This propagation is recursive — composability chains across protocols are captured automatically.
- **Branches with copy-on-write.** Scenarios fork from a shared baseline using CoW overlays (~12.8 KB per branch vs ~3.2 MB for a full clone), so you can run parallel what-if simulations cheaply.

For pure unit testing against static state, Anvil is great. For anything that touches live DeFi positions, mirage-rs fills the gap.

## How it works

mirage-rs sits between your application and a real Ethereum RPC endpoint. It maintains a three-layer state model:

```
 Reads flow top-down; first hit wins.

 ┌─────────────────────────────────┐
 │  1. DirtyStore (local writes)   │  ← eth_sendTransaction, setBalance, scenarios
 ├─────────────────────────────────┤
 │  2. ReadCache (LRU + TTL)       │  ← <1µs hot reads, 12s default TTL
 ├─────────────────────────────────┤
 │  3. UpstreamRpc (lazy fetch)    │  ← token-bucket rate limiter, retries w/ backoff
 └────────────────┬────────────────┘
                  │
          Live Ethereum node
```

On first access, account balances, nonces, storage slots, and bytecode are fetched from upstream and cached. Writes go into the dirty overlay and never touch upstream. You get a mutable view of mainnet state without syncing anything.

When a WebSocket URL is configured, the **targeted follower** subscribes to new blocks and replays only the transactions that matter to your watched contracts. Everything else is ignored.

## Installation

From source (requires Rust 1.80+):

```bash
cargo install --path apps/mirage-rs
```

Or run directly from the workspace:

```bash
cargo run -p mirage-rs -- --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
```

## Usage

### Forking mainnet

```bash
# Fork at latest block
mirage-rs --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Specify chain ID and port
mirage-rs --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY --chain-id 1 --port 8545

# With live block following
mirage-rs \
  --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  --ws-url wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
```

### Forking other chains

```bash
# Arbitrum
mirage-rs --rpc-url https://arb-mainnet.g.alchemy.com/v2/YOUR_KEY --chain-id 42161

# Base
mirage-rs --rpc-url https://base-mainnet.g.alchemy.com/v2/YOUR_KEY --chain-id 8453

# Polygon
mirage-rs --rpc-url https://polygon-mainnet.g.alchemy.com/v2/YOUR_KEY --chain-id 137
```

### Using with Foundry

mirage-rs is a drop-in backend for `forge script` and `forge test`:

```bash
# Run a Forge script against the live fork
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast

# Run tests
forge test --fork-url http://127.0.0.1:8545
```

### Using with Hardhat

```js
// hardhat.config.js
module.exports = {
  networks: {
    mirage: {
      url: "http://127.0.0.1:8545",
    },
  },
};
```

```bash
npx hardhat test --network mirage
```

### Using with Viem

```ts
import { createPublicClient, createWalletClient, http } from "viem";
import { mainnet } from "viem/chains";

const transport = http("http://127.0.0.1:8545");

const publicClient = createPublicClient({ chain: mainnet, transport });
const walletClient = createWalletClient({ chain: mainnet, transport });

// Works exactly like Anvil
const blockNumber = await publicClient.getBlockNumber();
```

### Using with Ethers.js

```js
const { ethers } = require("ethers");
const provider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");
const blockNumber = await provider.getBlockNumber();
```

## CLI Reference

```
mirage-rs [OPTIONS]
```

### Server options

| Flag | Default | Description |
|------|---------|-------------|
| `--host` | `127.0.0.1` | Bind address |
| `--port` | `8545` | Bind port |
| `--chain-id` | `1` | Effective chain ID |
| `--watchdog-timeout` | (none) | Shut down after N seconds of inactivity |

### Fork options

| Flag | Default | Description |
|------|---------|-------------|
| `--rpc-url` | (none) | Upstream HTTP JSON-RPC URL. Omit for isolated mode |
| `--ws-url` | (none) | Upstream WebSocket URL. Enables targeted block following |
| `--upstream-rps` | `100` | Upstream requests per second budget |
| `--upstream-burst` | `200` | Upstream burst capacity |
| `--cache-size` | `10000` | Read cache entry capacity |
| `--cache-ttl-secs` | `12` | Read cache TTL in seconds (~1 block) |

### Validation options

| Flag | Default | Description |
|------|---------|-------------|
| `--strict-nonce` | `false` | Reject transactions with incorrect nonces |
| `--strict-balance` | `false` | Reject transactions that overdraw sender balance |
| `--verify-signatures` | `false` | Require valid ECDSA signatures on raw transactions |

By default, validation is relaxed (same as Anvil's default behavior) so you can send transactions from any address without signing.

### Resource profiles

| Flag | Default | Description |
|------|---------|-------------|
| `--profile` | `standard` | Resource profile: `micro`, `standard`, `power` |

Profiles control memory ceilings and capacity limits:

| Profile | Memory ceiling | Watched contracts | Cache entries | Bytecode cache |
|---------|---------------|-------------------|---------------|----------------|
| `micro` | 256 MB | 32 | 5,000 | 1,000 |
| `standard` | 512 MB | 64 | 10,000 | 2,000 |
| `power` | 2 GB | 256 | 50,000 | 10,000 |

The process checks available system memory at startup and exits with code 2 if the selected profile doesn't fit (128 MB headroom margin required). At runtime, memory pressure is monitored and the fork responds in tiers:

| Pressure level | Threshold | Response |
|----------------|-----------|----------|
| Warning | 50% of ceiling | Evict LRU cache entries |
| Throttle | 70% | Demote auto-classified contracts to slot-only reads |
| Emergency | 90% | Demote to proxy mode (disable replay) |

## Supported RPC Methods

### Standard Ethereum methods

The same `eth_*` namespace you use with Anvil and any other node:

| Method | Description |
|--------|-------------|
| `eth_chainId` | Returns the chain ID |
| `eth_blockNumber` | Returns the current block number |
| `eth_gasPrice` | Returns the current gas price |
| `eth_maxPriorityFeePerGas` | Returns the current priority fee |
| `eth_feeHistory` | Returns fee history for a range of blocks |
| `eth_getBalance` | Returns the balance of an address |
| `eth_getTransactionCount` | Returns the nonce of an address |
| `eth_getStorageAt` | Returns the value of a storage slot |
| `eth_getCode` | Returns the bytecode at an address |
| `eth_call` | Executes a call without creating a transaction |
| `eth_estimateGas` | Estimates gas for a transaction |
| `eth_sendTransaction` | Sends a transaction (auto-signed, like Anvil) |
| `eth_sendRawTransaction` | Sends a signed raw transaction |
| `eth_getTransactionReceipt` | Returns the receipt of a transaction |
| `eth_getTransactionByHash` | Returns transaction details by hash |
| `eth_getBlockByNumber` | Returns a block by number |
| `eth_getBlockByHash` | Returns a block by hash |
| `eth_getLogs` | Returns logs matching a filter |
| `web3_clientVersion` | Returns the client version string |
| `net_version` | Returns the network ID |

### EVM manipulation methods

Anvil/Hardhat-compatible state manipulation. If your test suite uses these with Anvil, it works the same here:

| Method | Description |
|--------|-------------|
| `evm_snapshot` | Capture current state, returns a snapshot ID |
| `evm_revert` | Roll back to a snapshot |
| `evm_increaseTime` | Advance the block timestamp by N seconds |
| `evm_setNextBlockTimestamp` | Set a specific next-block timestamp |

### State override methods

Available under the `anvil_*`, `hardhat_*`, and `mirage_*` namespaces (all three work):

| Method | Description |
|--------|-------------|
| `setBalance(address, value)` | Override an account's ETH balance |
| `setStorageAt(address, slot, value)` | Write a single storage slot |
| `setCode(address, bytecode)` | Deploy bytecode at an address |
| `setNonce(address, nonce)` | Override an account's nonce |

```bash
# Set balance using cast (works with any namespace prefix)
cast rpc anvil_setBalance 0xf39F...2266 0xDE0B6B3A7640000 --rpc-url http://127.0.0.1:8545
```

### Mirage-specific methods

These extend the Anvil API with live-chain capabilities:

| Method | Description |
|--------|-------------|
| `mirage_mintERC20(token, to, amount)` | Mint ERC-20 tokens by detecting and writing the balance storage slot |
| `mirage_watchContract(address)` | Add a contract to the targeted follower's watch list |
| `mirage_unwatchContract(address)` | Remove a contract from the watch list |
| `mirage_getWatchList()` | Return all watched contracts with metadata |
| `mirage_prefetchAccount(address)` | Warm the cache for an account |
| `mirage_prefetchSlots(address, slots[])` | Warm specific storage slots |
| `mirage_getDirtySlots(address)` | Return locally modified storage slots for an address |
| `mirage_status()` | Readiness status, chain ID, block number, watch list size |
| `mirage_getResourceUsage()` | Memory, cache stats, pressure score, upstream counters |
| `mirage_setResourceLimits(...)` | Dynamically adjust resource caps at runtime |
| `mirage_getPosition(request)` | Read a DeFi position snapshot |
| `mirage_subscribeEvents(filter)` | Open a WebSocket event stream with address/topic filters |
| `mirage_shutdown()` | Graceful process shutdown |

### Scenario methods

Run branching what-if simulations against live state:

| Method | Description |
|--------|-------------|
| `mirage_beginScenarioSet(baseline)` | Create a scenario set from a baseline state |
| `mirage_defineScenario(setId, scenario)` | Add a scenario with transactions and assertions |
| `mirage_runScenarioSet(setId, mode)` | Execute in `sequential` or `parallel` mode |
| `mirage_getScenarioResults(jobId)` | Poll for results |
| `mirage_compareScenarios(setId)` | Diff outcomes across scenarios in a set |

## Targeted Following

This is the core feature that separates mirage-rs from static forks.

When you provide a `--ws-url`, mirage-rs subscribes to `newHeads` and for each new block:

1. Fetches the full block from upstream
2. Filters for transactions touching any watched address
3. Replays only those transactions through the local fork's EVM
4. Runs the diff classifier on the resulting state changes
5. Auto-promotes new contracts that cross the slot threshold (3+ storage writes)

For a typical DeFi portfolio (3-10 positions), this means replaying ~5-15 transactions per block instead of the full ~150. Blocks process in <100ms at steady state.

### Watch list management

Contracts enter the watch list three ways:

1. **Manual** — call `mirage_watchContract(address)` or define `track.addresses` in a scenario fixture
2. **Auto-classification** — the diff classifier sees 3+ storage slots written on a new address and promotes it
3. **Contagion** — a replayed transaction writes to a new contract that exceeds the slot threshold, recursively extending the watch list

```bash
# Manually watch the Uniswap V3 Router
cast rpc mirage_watchContract 0xE592427A0AEce92De3Edee1F18E0157C05861564 \
  --rpc-url http://127.0.0.1:8545

# Check the watch list
cast rpc mirage_getWatchList --rpc-url http://127.0.0.1:8545
```

## Scenarios

Scenarios let you define branching what-if simulations. Each scenario is a named sequence of transactions that execute against a shared baseline snapshot. In parallel mode, each branch gets an isolated copy-on-write overlay, so branches can't observe each other's mutations.

### TOML fixtures

```toml
# tests/scenarios/eth_crash.toml
[scenario]
name = "eth_crash"
description = "Directional WETH->USDC selloff with repeated router pressure"

[[transactions]]
from = "0x1000000000000000000000000000000000000001"
to = "0x10000000000000000000000000000000000000a0"
value = "0x0"
gas = 320000
data = "0x414bf389..."

[assertions]
watch_list_contains = ["0x10000000000000000000000000000000000000a0"]

[assertions.token_balance_gte]
token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
address = "0x1000000000000000000000000000000000000001"
amount = "0x1"

[track]
addresses = ["0xE592427A0AEce92De3Edee1F18E0157C05861564"]
```

### Included scenarios

| File | Description |
|------|-------------|
| `uniswap_v3_entry.toml` | Position-manager mint + liquidity increase on a watched pool |
| `eth_crash.toml` | Directional WETH→USDC selloff with 20+ router transactions |
| `aave_liquidation.toml` | Oracle shock, account deterioration, and liquidation flow |
| `new_pool.toml` | Deploy token, initialize pool, seed liquidity, route first swap |
| `volume_spike.toml` | High-frequency volume burst across multiple pairs |

### Programmatic usage

```rust
use mirage_rs::{MirageClient, Scenario, RunMode, ScenarioAssertions};

let set_id = client.mirage_begin_scenario_set("latest").await?;

client.mirage_define_scenario(&set_id, &Scenario {
    id: "bull-case".into(),
    name: "large buy".into(),
    transactions: vec![buy_tx],
    track_addresses: vec![pool, router],
    max_gas: Some(500_000),
    timeout: Duration::from_secs(5),
    assertions: ScenarioAssertions::default(),
}).await?;

client.mirage_define_scenario(&set_id, &Scenario {
    id: "bear-case".into(),
    name: "large sell".into(),
    transactions: vec![sell_tx],
    track_addresses: vec![pool, router],
    max_gas: Some(500_000),
    timeout: Duration::from_secs(5),
    assertions: ScenarioAssertions::default(),
}).await?;

// Run both branches in parallel with isolated CoW overlays
let job_id = client.mirage_run_scenario_set(&set_id, RunMode::Parallel).await?;
let results = client.mirage_get_scenario_results(&job_id).await?;
```

## Library Usage

mirage-rs ships as both a binary and a library crate.

```toml
[dependencies]
mirage-rs = { path = "apps/mirage-rs", default-features = false, features = ["library"] }
```

### Spawning a test instance

```rust
use mirage_rs::{MirageClient, spawn_mirage_test_instance, TransactionRequest};
use std::time::Duration;
use alloy_primitives::U256;

let mut instance = spawn_mirage_test_instance(None, Some(18_545)).await?;
let client = MirageClient::new(instance.config()).await?;
client.wait_ready(Duration::from_secs(10)).await?;

// Use it like Anvil
let tx_hash = client.eth_send_transaction(TransactionRequest {
    from: Some(sender),
    to: Some(receiver),
    gas: Some(21_000),
    value: Some(U256::from(1_000_000)),
    ..Default::default()
}).await?;

let snap = client.evm_snapshot().await?;
// ... speculative work ...
client.evm_revert(snap).await?;

instance.shutdown().await?;
```

### GolemConfig integration

Inside the bardo ecosystem, derive the client config from `GolemConfig`:

```rust
let config = MirageConfig::from_golem_config(&golem_config);
let client = MirageClient::new(config).await?;
```

## Architecture

```
                        +-----------------+
                        |  Your app /     |
                        |  Foundry / test |
                        +--------+--------+
                                 |
                            JSON-RPC
                                 |
                        +--------v--------+
                        |   RPC Server    |
                        |  (axum + json-  |
                        |   rpsee)        |
                        +--------+--------+
                                 |
                        +--------v--------+
                        |   MirageFork    |
                        |  (Arc<RwLock>)  |
                        +--------+--------+
                                 |
              +------------------+------------------+
              |                  |                  |
     +--------v------+  +-------v-------+  +-------v--------+
     |  DirtyStore   |  |  ReadCache    |  |  UpstreamRpc   |
     | (local writes)|  | (LRU + TTL)   |  | (rate-limited) |
     +---------------+  +---------------+  +-------+--------+
                                                   |
                                           +-------v--------+
                                           | Ethereum node  |
                                           | (HTTP + WS)    |
                                           +-------+--------+
                                                   |
                                          +--------v--------+
                                          | Targeted        |
                                          | Follower        |
                                          | (newHeads sub,  |
                                          |  selective      |
                                          |  replay)        |
                                          +-----------------+
```

### Core types

**State layer:**

- `MirageFork` — thread-safe handle (`Arc<RwLock<MirageState>>`) shared across the RPC server, follower, and scenario runner.
- `ForkState` — mutable fork state: `HybridDB`, block number, chain ID, timestamp, watch list, snapshot stack, dirty tracking.
- `HybridDB` — three-layer database: `DirtyStore` (local writes) → `ReadCache` (LRU with TTL) → `UpstreamRpc` (lazy fetches).
- `DirtyAccount` / `DirtyStore` — tracks accounts and storage slots modified since the last snapshot or baseline.

**Copy-on-write:**

- `CowState` — shared baseline + per-branch overlay. Branches read from the overlay first, then fall through to the shared baseline. Writes go only to the overlay.
- `MultiVersionStore` — per-slot multi-version storage for the Block-STM test harness.
- `BytecodeCache` — LRU keyed by code hash. Bytecode is immutable, so no CoW needed.

**Upstream:**

- `UpstreamRpc` — wraps `reqwest::blocking::Client` with a token-bucket rate limiter, retries with exponential backoff, and a mock mode for offline testing.
- `ReadCache` — LRU cache with per-entry TTL. Tracks hit/miss counts and supports targeted eviction under memory pressure.

**Replay and speculative execution:**

- `TargetedFollower` — subscribes to `newHeads` via WebSocket, replays only transactions touching watched contracts.
- `SpeculativeExecutor` — runs transactions against a CoW branch without mutating base state. Returns execution result + full `StateDiff` + read set for invalidation tracking.
- `TxReplay` — fetches a historical transaction by hash from upstream and re-executes it locally.
- `DiffClassifier` — inspects state diffs and classifies contracts as `Protocol` (complex, should be watched), `SlotOnly` (simple override), or `ReadOnly`.

**Scenarios:**

- `ScenarioRunner` — orchestrates scenario sets with `Sequential` (revert between runs) or `Parallel` (independent CoW branches) execution modes.
- `Scenario` — named transaction sequence with tracked addresses, gas budget, timeout, and assertions.

**Integration:**

- `MirageClient` / `MirageConfig` — async HTTP client wrapping all RPC methods with retry and timeout.
- `MirageTestInstance` — spawned child process with config access and clean shutdown.
- `EventFilter` / `MirageEvent` — WebSocket event subscription with address/topic filters, carrying provenance (`LocalTx` or `FollowerReplay`).

## Error Handling

All library errors are `MirageError`. Each variant maps to a JSON-RPC error code:

| Variant | Code | When |
|---------|------|------|
| `InvalidParams` | -32602 | Malformed RPC parameters |
| `Unsupported` | -32603 | Operation not supported in current mode |
| `InvalidFrom` | -32010 | Invalid sender address |
| `SnapshotNotFound` | -32001 | Snapshot ID doesn't exist or was consumed |
| `SlotDetectionFailed` | -32020 | ERC-20 balance slot detection failed |
| `WatchListFull` | -32030 | Watch list at capacity for the current profile |
| `UnknownProtocolType` | -32040 | Position helper doesn't recognize the protocol |
| `SetNotFound` | -32050 | Scenario set doesn't exist |
| `JobNotFound` | -32054 | Scenario job doesn't exist |
| `JobNotComplete` | -32055 | Scenario job still running |
| `Upstream` | -32099 | Upstream RPC failure |
| `Timeout` | -32603 | Operation exceeded its time budget |
| `BindFailed` | -32603 | Could not bind the server port |

## Testing

```bash
# Unit tests
cargo test -p mirage-rs

# Integration tests (spawns real mirage processes)
cargo test -p mirage-rs --test integration
```

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `binary` | yes | Includes the CLI entrypoint |
| `library` | no | Library-only builds (no binary dependencies) |
| `sim-gas` | no | Gas simulation instrumentation |

## Startup artifacts

On startup, mirage writes two files:

- `/tmp/mirage-{port}.pid` — process ID
- `/tmp/mirage-{port}-status.json` — `{"status":"ready","port":N}`

Both are cleaned up on shutdown. Use the status file for CI health checks or orchestrator readiness probes.

## Anvil compatibility at a glance

| Capability | Anvil | mirage-rs |
|------------|-------|-----------|
| Fork from RPC | Yes (pinned block) | Yes (latest, follows forward) |
| `eth_*` methods | Full | Common DeFi subset |
| `evm_snapshot` / `evm_revert` | Yes | Yes |
| `anvil_setBalance` / `setStorageAt` / etc. | Yes | Yes (also `hardhat_*` and `mirage_*` prefixes) |
| `evm_increaseTime` / `evm_setNextBlockTimestamp` | Yes | Yes |
| Auto-mine | Yes | Yes |
| Impersonate accounts | Yes | Yes (relaxed signing by default) |
| Live block following | No | Yes (targeted follower via WebSocket) |
| Contract auto-classification | No | Yes (diff classifier + contagion) |
| Copy-on-write scenario branching | No | Yes |
| ERC-20 balance slot detection + mint | No | Yes (`mirage_mintERC20`) |
| Memory pressure management | No | Yes (tiered eviction/demotion) |
| Resource profiles | No | Yes (`micro` / `standard` / `power`) |

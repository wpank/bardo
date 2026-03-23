# mirage-rs

In-process Ethereum fork with lazy upstream reads, copy-on-write scenario branching, and a targeted block follower. mirage-rs runs a JSON-RPC server that speaks the standard Ethereum API backed by a speculative dirty state layer over a live RPC node. No full node sync required.

Built for DeFi simulation: run what-if scenarios against real on-chain state, replay upstream transactions through a local fork, and track protocol contracts as their storage evolves block-by-block.

## How it works

mirage-rs sits between your application and a real Ethereum RPC endpoint. On first access, account balances, nonces, storage slots, and bytecode are fetched from the upstream node and cached locally. Writes go into a dirty overlay that never touches the upstream. This gives you a mutable view of mainnet state without syncing anything.

The fork maintains a three-layer read path:

1. **Dirty store** -- local writes from `eth_sendTransaction`, `mirage_setBalance`, scenario execution, etc.
2. **Read cache** -- LRU cache with configurable TTL (default 12s, roughly one block) sitting in front of upstream calls.
3. **Upstream RPC** -- lazy fetches with token-bucket rate limiting and automatic retries with exponential backoff.

When an upstream WebSocket URL is configured, a **targeted follower** subscribes to `newHeads` and selectively replays transactions that touch watched contracts. This keeps the local fork's view of tracked protocols in sync with the chain without replaying every transaction in every block.

## Running

```bash
# Fork mainnet, bind on default port 8545
cargo run -p mirage-rs -- --rpc-url https://eth-mainnet.example.com

# With WebSocket for targeted block following
cargo run -p mirage-rs -- \
  --rpc-url https://eth-mainnet.example.com \
  --ws-url wss://eth-mainnet.example.com \
  --port 8545 \
  --chain-id 1

# Isolated mode (no upstream -- pure in-memory state, all accounts start with 1 ETH)
cargo run -p mirage-rs

# Power profile with tight upstream rate limits
cargo run -p mirage-rs -- \
  --rpc-url https://eth-mainnet.example.com \
  --profile power \
  --upstream-rps 50 \
  --upstream-burst 100
```

On startup, mirage writes `/tmp/mirage-{port}.pid` and `/tmp/mirage-{port}-status.json` (`{"status":"ready","port":N}`). Both are cleaned up on shutdown.

## CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--host` | `127.0.0.1` | Bind address |
| `--port` | `8545` | Bind port |
| `--rpc-url` | none | Upstream HTTP JSON-RPC URL |
| `--ws-url` | none | Upstream WebSocket URL (enables targeted following) |
| `--upstream-rps` | `100` | Upstream request budget per second |
| `--upstream-burst` | `200` | Upstream burst capacity |
| `--chain-id` | `1` | Effective chain ID |
| `--cache-size` | `10000` | Read cache entry capacity |
| `--cache-ttl-secs` | `12` | Read cache TTL in seconds |
| `--profile` | `standard` | Resource profile: `micro`, `standard`, `power` |
| `--watchdog-timeout` | none | Inactivity shutdown timeout in seconds |
| `--strict-nonce` | `false` | Reject transactions with wrong nonces |
| `--strict-balance` | `false` | Reject transactions that overdraw balance |
| `--verify-signatures` | `false` | Enable ECDSA signature verification on raw transactions |

If `--rpc-url` is provided, the binary probes the upstream before binding. A failed probe exits with code 1. An insufficient-memory error from the resource model exits with code 2.

## Resource profiles

| Profile | Memory ceiling | Watched contracts | Cache entries | Bytecode cache |
|---------|---------------|-------------------|---------------|----------------|
| Micro | 256 MB | 32 | 5,000 | 1,000 |
| Standard | 512 MB | 64 | 10,000 | 2,000 |
| Power | 2 GB | 256 | 50,000 | 10,000 |

The process checks available system memory at startup and exits with code 2 if the selected profile can't fit (with a 128 MB headroom margin).

At runtime, memory pressure is monitored continuously. The fork responds to pressure in tiers:

| Pressure | Threshold | Action |
|----------|-----------|--------|
| Warning | 50% of ceiling | Evict LRU cache entries |
| Throttle | 70% | Demote auto-classified contracts to slot-only reads |
| Emergency | 90% | Demote runtime mode to proxy (disable replay) |

## JSON-RPC API

### Standard Ethereum methods

mirage-rs implements the common subset needed by DeFi tooling:

- `web3_clientVersion`, `net_version`
- `eth_chainId`, `eth_blockNumber`, `eth_gasPrice`, `eth_maxPriorityFeePerGas`, `eth_feeHistory`
- `eth_getBalance`, `eth_getTransactionCount`, `eth_getStorageAt`, `eth_getCode`
- `eth_call`, `eth_estimateGas`
- `eth_sendTransaction`, `eth_sendRawTransaction`
- `eth_getTransactionReceipt`, `eth_getTransactionByHash`
- `eth_getBlockByNumber`, `eth_getBlockByHash`
- `eth_getLogs`

### EVM control methods

Compatible with Hardhat and Anvil:

- `evm_snapshot` -- capture current state, returns a snapshot ID
- `evm_revert` -- roll back to a snapshot
- `evm_increaseTime` -- advance the block timestamp by N seconds
- `evm_setNextBlockTimestamp` -- set a specific next-block timestamp

### State override methods

Available under `mirage_*`, `hardhat_*`, and `anvil_*` prefixes:

- `setBalance(address, value)` -- override an account's ETH balance
- `setStorageAt(address, slot, value)` -- write a single storage slot
- `setCode(address, bytecode)` -- deploy bytecode at an address
- `setNonce(address, nonce)` -- override an account's nonce

### Mirage-specific methods

- `mirage_mintERC20(token, to, amount)` -- mint ERC-20 tokens by detecting and writing the balance storage slot
- `mirage_prefetchAccount(address)` -- warm the cache for an account
- `mirage_prefetchSlots(address, slots[])` -- warm specific storage slots
- `mirage_watchContract(address)` -- add a contract to the targeted follower's watch list
- `mirage_unwatchContract(address)` -- remove a contract from the watch list
- `mirage_getWatchList()` -- return all watched contracts with metadata (source, block added, slot count, replay count)
- `mirage_getDirtySlots(address)` -- return locally modified storage slots for an address
- `mirage_status()` -- readiness status, chain ID, block number, watch list size
- `mirage_getResourceUsage()` -- memory, cache stats, pressure score, upstream call/error counters, mode
- `mirage_setResourceLimits(...)` -- dynamically adjust resource caps at runtime
- `mirage_getPosition(request)` -- read a DeFi position snapshot (raw balances, protocol-specific readers)
- `mirage_subscribeEvents(filter)` -- open a WebSocket event stream with address/topic filters
- `mirage_shutdown()` -- graceful process shutdown
- `mirage_cleanup()` -- clean up transient resources

### Scenario methods

- `mirage_beginScenarioSet(baseline)` -- create a scenario set from a baseline state
- `mirage_defineScenario(setId, scenario)` -- add a scenario with transactions and assertions
- `mirage_runScenarioSet(setId, mode)` -- execute in `sequential` or `parallel` mode
- `mirage_getScenarioResults(jobId)` -- poll for results
- `mirage_compareScenarios(setId)` -- diff outcomes across scenarios in a set

## Scenario system

Scenarios let you define branching what-if simulations. Each scenario is a named sequence of transactions that execute against a shared baseline snapshot. In parallel mode, each branch gets an isolated copy-on-write overlay so execution is non-destructive and branches can't observe each other's mutations.

### TOML fixtures

Scenario fixtures live in `tests/scenarios/` and can be loaded directly by the scenario runner:

```toml
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
token = "0x..."
address = "0x..."
amount = "0x1"

[track]
addresses = ["0x...", "0x..."]
```

### Included scenarios

| File | Description |
|------|-------------|
| `uniswap_v3_entry.toml` | Position-manager mint + liquidity increase on a watched pool |
| `eth_crash.toml` | Directional WETH->USDC selloff with 20+ router transactions |
| `aave_liquidation.toml` | Oracle shock, account deterioration, and liquidation flow |
| `new_pool.toml` | Deploy token, initialize pool, seed liquidity, route first swap |
| `volume_spike.toml` | High-frequency volume burst across multiple pairs |

### Programmatic scenario usage

```rust
use mirage_rs::{MirageClient, Scenario, RunMode, ScenarioAssertions, TransactionRequest};

let set_id = client.mirage_begin_scenario_set("latest").await?;

client.mirage_define_scenario(&set_id, &Scenario {
    id: "left-branch".into(),
    name: "left transfer".into(),
    transactions: vec![tx_a],
    track_addresses: vec![sender, receiver_a],
    max_gas: Some(30_000),
    timeout: Duration::from_secs(1),
    assertions: ScenarioAssertions::default(),
}).await?;

client.mirage_define_scenario(&set_id, &Scenario {
    id: "right-branch".into(),
    name: "right transfer".into(),
    transactions: vec![tx_b],
    track_addresses: vec![sender, receiver_b],
    max_gas: Some(30_000),
    timeout: Duration::from_secs(1),
    assertions: ScenarioAssertions::default(),
}).await?;

let job_id = client.mirage_run_scenario_set(&set_id, RunMode::Parallel).await?;
let job = client.mirage_get_scenario_results(&job_id).await?;
```

## Library usage

mirage-rs ships as both a binary and a library. For library-only builds:

```toml
[dependencies]
mirage-rs = { path = "apps/mirage-rs", default-features = false, features = ["library"] }
```

### Integration test harness

Spawn an isolated mirage instance for integration tests:

```rust
use mirage_rs::{MirageClient, MirageConfig, spawn_mirage_test_instance};

let mut instance = spawn_mirage_test_instance(None, Some(18_545)).await?;
let client = MirageClient::new(instance.config()).await?;
client.wait_ready(Duration::from_secs(10)).await?;

// Send a transaction
let tx_hash = client.eth_send_transaction(TransactionRequest {
    from: Some(sender),
    to: Some(receiver),
    gas: Some(21_000),
    value: Some(U256::from(1_000_000)),
    ..Default::default()
}).await?;

// Snapshot and revert
let snap = client.evm_snapshot().await?;
// ... do speculative work ...
client.evm_revert(snap).await?;

instance.shutdown().await?;
```

### Client from GolemConfig

If you're running inside the bardo/golem ecosystem, derive the client config from `GolemConfig`:

```rust
let config = MirageConfig::from_golem_config(&golem_config);
let client = MirageClient::new(config).await?;
```

## Architecture

```
                        +-----------------+
                        |  Your app /     |
                        |  agent / tests  |
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
                                           +----------------+
```

### Core types

**State layer:**

- `MirageFork` -- thread-safe handle (`Arc<RwLock<MirageState>>`) shared across the RPC server, follower, and scenario runner. Entry point for all fork operations.
- `ForkState` -- mutable fork state: `HybridDB`, block number, chain ID, timestamp, watch list, snapshot stack, dirty tracking. Configurable strict nonce/balance checks and signature verification.
- `HybridDB` -- three-layer database: `DirtyStore` (local writes), `ReadCache` (LRU with TTL), `UpstreamRpc` (lazy fetches). Optionally pins reads to a fixed block for historical mode.
- `DirtyAccount` / `DirtyStore` -- track accounts and storage slots modified since the last snapshot or baseline.

**Copy-on-write:**

- `CowState` -- shared baseline + per-branch overlay. Branches read from the overlay first, then fall through to the shared baseline `Arc<HashMap>`. Writes go only to the overlay.
- `MultiVersionStore` -- per-slot multi-version storage for the simplified Block-STM test harness. Records `VersionEntry` (tx_index, incarnation, value) and materializes the latest values.
- `BytecodeCache` -- separate LRU keyed by code hash. Bytecode is immutable so it doesn't need copy-on-write.

**Upstream:**

- `UpstreamRpc` -- wraps `reqwest::blocking::Client` (built before the Tokio runtime starts to avoid the nested-runtime panic). Token-bucket rate limiter, automatic retries with exponential backoff, mock mode for offline/test use.
- `ReadCache` -- LRU cache with per-entry TTL for accounts, storage slots, and block hashes. Tracks hit/miss counts. Supports targeted eviction under memory pressure.
- `BlockTag` -- `Latest` or `Number(u64)`.

**Replay and speculative execution:**

- `TargetedFollower` -- subscribes to `newHeads` via WebSocket and replays confirmed transactions that touch watched contracts. Configurable block-budget timeout and address/selector filters.
- `SpeculativeExecutor` -- runs transactions against a CoW branch without mutating base state. Returns `SpeculativeResult` with execution result, full `StateDiff`, read set (for invalidation tracking), and timestamp.
- `TxReplay` -- fetches a historical transaction by hash from upstream and re-executes it against the local fork.
- `StateDiff` / `AccountDiff` / `LogEntry` -- structured diff with per-account balance/nonce/code/storage changes and emitted logs.

**Classification:**

- `DiffClassifier` -- inspects state diffs and classifies contracts as `Protocol` (complex storage, should be watched and replayed), `SlotOnly` (simple override), or `ReadOnly` (no writes). Configurable slot threshold, token-interface heuristics, contagion propagation, and watch list cap.
- `WatchEntry` / `WatchSource` -- metadata for watched contracts: how they were added, at which block, initial slot count, replay passes applied.

**Scenarios:**

- `ScenarioRunner` -- orchestrates scenario sets. Supports `Sequential` (revert between runs) and `Parallel` (independent CoW branches) execution modes.
- `Scenario` -- named transaction sequence with `track_addresses`, optional `max_gas`, `timeout`, and `ScenarioAssertions` (watch list membership, token balance lower bounds).
- `ScenarioSet` / `ScenarioJob` / `ScenarioResult` / `ScenarioStatus` / `ScenarioSetStatus` -- lifecycle and result types.

**Integration:**

- `MirageClient` / `MirageConfig` -- async HTTP client wrapping all RPC methods with retry and timeout. Can be derived from `GolemConfig` for ecosystem integration.
- `MirageTestInstance` -- holds a spawned mirage child process, provides its config, and handles clean shutdown.
- `EventFilter` / `EventSource` / `MirageEvent` -- WebSocket event subscription with address/topic filters. Events carry provenance (`LocalTx` or `FollowerReplay`).
- `PositionRequest` / `PositionSnapshot` -- DeFi position query with protocol-type routing and raw balance snapshots.

## Error handling

All library errors are `MirageError`. Each variant maps to a JSON-RPC error code:

| Variant | Code | Description |
|---------|------|-------------|
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

## Cargo features

| Feature | Default | Description |
|---------|---------|-------------|
| `binary` | yes | Include the CLI entrypoint |
| `library` | no | Library-only builds (no binary dependencies) |
| `sim-gas` | no | Gas simulation instrumentation |

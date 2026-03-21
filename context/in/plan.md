# Plan 03: mirage-rs — Full Implementation

## Context

mirage-rs is the in-process EVM fork that replaces Anvil entirely as the testing backbone for the Bardo project. It is intentionally early in the plan sequence: every subsequent plan that touches on-chain simulation verifies against mirage-rs rather than Anvil or a live node.

The core insight of v2 is **inversion**: instead of replaying every block locally, lazy-latest reads pull live state from mainnet on demand. Only locally-modified slots are tracked. Only mainnet transactions touching watched contracts get replayed. The result is a local EVM that stays synchronized with mainnet at minimal compute cost.

mirage-rs ships as both a standalone binary (golem sidecar) and an optional library crate for embedding directly into test processes. The JSON-RPC interface is Anvil/Hardhat-compatible, so all existing tooling (`cast`, `viem`, `wagmi`) works without changes.

Port: **8545** by default (inheriting the Anvil slot from `prd2/shared/port-allocation.md`). Override with `--port`. In test harnesses that run alongside a live Anvil, use `--port 18545`.

## Previous Plan

Plan 02 created `golem-core` with:
- `GolemConfig` — full TOML runtime config schema at `golem_core::config`
- `EventFabric` — tokio broadcast bus + 10K ring buffer at `golem_core::event`
- `CorticalState` — 32-signal lock-free perception surface at `golem_core::cortical`
- `Extension` trait — 20-hook async trait at `golem_core::extension`

mirage-rs imports `GolemConfig` for integration and emits `resource_pressure` values that feed into `CorticalState`.

## Prerequisites

- **Plan 01** — workspace scaffold; `apps/mirage-rs/` directory must exist as a workspace member
- **Plan 02** — `golem-core` crate must be compiled; `GolemConfig` import used in `integration.rs`

## Imports

```rust
// In integration.rs
use golem_core::config::GolemConfig;
```

## Exports

All public types live in `mirage-rs` as a library crate (feature-gated) and are also accessible via the JSON-RPC server. The primary exports for downstream plans:

| Type | Module | Purpose |
|------|--------|---------|
| `MirageClient` | `integration` | Async client for golem components to connect to mirage-rs |
| `MirageConfig` | `integration` | Connection config: url, timeout, retry policy |
| `MirageFork` | `fork` | The main in-process revm fork handle (library mode) |
| `ForkState` | `fork` | Current EVM state: HybridDB + block context |
| `HybridDB` | `fork` | Three-tier database: DirtyStore → ReadCache → upstream RPC |
| `DirtyStore` | `fork` | Write layer for local mutations and watch list |
| `CowState` | `cow` | Copy-on-write state overlay for speculative/scenario execution |
| `SpeculativeExecutor` | `replay` | Execute a tx without broadcasting, return state diff |
| `StateDiff` | `replay` | What changed: AccountDiff, storage writes, logs |
| `ScenarioRunner` | `scenario` | Load → fork → replay → verify |
| `ScenarioResult` | `scenario` | Pass/fail with balances, gas, logs |
| `ResourceUsage` | `resources` | Memory/CPU metrics snapshot |

## Cargo Dependencies

```toml
[dependencies]
# EVM execution
revm = { version = "36", features = ["std", "serde"] }
revm-database = "12"

# Ethereum types and RPC
alloy = { version = "1.7", features = [
    "full",
    "rpc-types",
    "provider-http",
    "provider-ws",
    "transport-http",
    "transport-ws",
    "signer-local",
] }

# JSON-RPC server (Anvil-compatible)
jsonrpsee = { version = "0.26", features = ["server", "http-client"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP server for SSE + REST endpoints
axum = { version = "0.8", features = ["ws"] }
tower-http = { version = "0.6", features = ["cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Caching
lru = "0.12"
dashmap = "6"

# Concurrency
parking_lot = "0.12"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Futures / streams
futures = "0.3"
futures-util = "0.3"

# HTTP client (for upstream RPC proxy)
reqwest = { version = "0.12", features = ["json"] }

# Shared workspace types
golem-core = { path = "../../crates/golem-core" }

[features]
default = ["binary"]
binary = ["tokio/full"]   # JSON-RPC server + binary entrypoint
library = []              # API-only, no server; for embedding in tests
sim-gas = []              # Optional sine-wave gas price simulator
```

## Source Files

```
apps/mirage-rs/
├── Cargo.toml
└── src/
    ├── main.rs          — HTTP/WS server entry point; CLI parsing; startup sequence
    ├── fork.rs          — HybridDB, DirtyStore, ForkState, DiffClassifier, EvmExecutor
    ├── provider.rs      — UpstreamRpc (alloy HTTP+WS provider with retry/rate-limit)
    ├── cow.rs           — CowState, BytecodeCache, MultiVersionStore (Block-STM)
    ├── rpc.rs           — JSON-RPC handler: eth_*, evm_*, hardhat_*, anvil_*, mirage_*
    ├── replay.rs        — TargetedFollower, SpeculativeExecutor, TxReplay, StateDiff
    ├── scenario.rs      — ScenarioSet, ScenarioRunner, ScenarioResult, ScenarioJob
    ├── integration.rs   — MirageClient, MirageConfig (golem sidecar client library)
    └── resources.rs     — ResourceModel, ResourceUsage, pressure tiers, eviction
```

---

## Implementation Details

### Unit 1: Fork Engine & Lazy Provider

**Files:** `fork.rs`, `provider.rs`
**~350 lines**

#### Quick Reference

**HybridDB** — three-tier database implementing revm's `Database` trait. Read priority: DirtyStore → ReadCache → upstream RPC at `latest`.

```rust
use alloy_primitives::{Address, B256, U256};
use revm::primitives::{AccountInfo, Bytecode};
use revm::Database;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct HybridDB {
    pub dirty: DirtyStore,
    pub read_cache: ReadCache,
    pub bytecode_cache: Arc<parking_lot::Mutex<BytecodeCache>>,
    pub upstream: Arc<UpstreamRpc>,
    pub pinned_block: Option<u64>,
    pub cache_ttl: Duration,         // default: 12s (one block)
    pub chain_id: u64,
}

impl Database for HybridDB {
    type Error = MirageError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, MirageError> {
        // 1. DirtyStore: partial override — merge dirty fields with upstream
        if let Some(dirty) = self.dirty.accounts.get(&address) {
            let needs_upstream = dirty.balance.is_none()
                || dirty.nonce.is_none()
                || dirty.code.is_none();
            let base = if needs_upstream {
                self.fetch_account_info(address)?.unwrap_or_default()
            } else {
                AccountInfo::default()
            };
            return Ok(Some(AccountInfo {
                balance: dirty.balance.unwrap_or(base.balance),
                nonce: dirty.nonce.unwrap_or(base.nonce),
                code_hash: dirty.code_hash.unwrap_or(base.code_hash),
                code: dirty.code.clone().or(base.code),
            }));
        }
        // 2. ReadCache
        if let Some(info) = self.read_cache.get_account(&address) {
            return Ok(Some(info));
        }
        // 3. Upstream fetch + cache
        let info = self.fetch_account_info(address)?;
        if let Some(ref i) = info {
            self.read_cache.insert_account(address, i.clone(), self.resolve_block());
        }
        Ok(info)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, MirageError> {
        // 1. DirtyStore slot
        if let Some(dirty) = self.dirty.accounts.get(&address) {
            if let Some(&val) = dirty.storage.get(&index) {
                return Ok(val);
            }
        }
        // 2. ReadCache
        if let Some(val) = self.read_cache.get_storage(&address, &index) {
            return Ok(val);
        }
        // 3. Upstream
        let block = self.resolve_block();
        let val = self.upstream.get_storage_at(address, index, block)?;
        self.read_cache.insert_storage(address, index, val, block);
        Ok(val)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, MirageError> {
        if let Some(bc) = self.bytecode_cache.lock().get(&code_hash) {
            return Ok(bc);
        }
        for dirty in self.dirty.accounts.values() {
            if dirty.code_hash == Some(code_hash) {
                if let Some(ref code) = dirty.code {
                    self.bytecode_cache.lock().insert(code_hash, code.clone());
                    return Ok(code.clone());
                }
            }
        }
        let bc = self.upstream.get_code_by_hash(code_hash, self.resolve_block())?;
        self.bytecode_cache.lock().insert(code_hash, bc.clone());
        Ok(bc)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, MirageError> {
        if let Some(h) = self.read_cache.block_hashes.get(&number) {
            return Ok(*h);
        }
        let h = self.upstream.get_block_hash(number)?;
        self.read_cache.block_hashes.put(number, h);
        Ok(h)
    }
}

/// resolve_block() returns pinned_block if set (historical mode), else "latest"
impl HybridDB {
    fn resolve_block(&self) -> BlockTag { ... }
}
```

**DirtyStore** — the write layer. Holds local mutations and the watch list.

```rust
pub struct DirtyStore {
    pub accounts: HashMap<Address, DirtyAccount>,
    pub watch_list: HashMap<Address, WatchEntry>,
    pub unwatch_list: HashSet<Address>,
    pub total_dirty_slots: u64,
    snapshots: HashMap<u64, Box<DirtyStoreSnapshot>>,
    next_snapshot_id: u64,
}

pub struct DirtyAccount {
    pub balance: Option<U256>,
    pub nonce: Option<u64>,
    pub code: Option<Bytecode>,
    pub code_hash: Option<B256>,
    pub storage: HashMap<U256, U256>,
}

pub struct WatchEntry {
    pub source: WatchSource,
    pub added_at_block: u64,
    pub initial_slot_count: usize,
    pub replay_count: u64,
}

pub enum WatchSource {
    AutoClassified,
    Contagion { parent: Address },
    Manual,
}

// Snapshot/revert: snapshot consumed on revert (single-use).
// Later snapshots are invalidated when reverting to an earlier one.
impl DirtyStore {
    pub fn snapshot(&mut self, block_number: u64, tx_index: u64) -> u64 { ... }
    pub fn revert(&mut self, id: u64) -> Result<(u64, u64), MirageError> { ... }
}
```

**DiffClassifier** — classifies state diffs into Protocol / SlotOnly / ReadOnly.

```rust
pub struct ClassificationConfig {
    pub protocol_slot_threshold: usize,  // default: 3
    pub check_token_interface: bool,
    pub max_watched_contracts: usize,    // default: 64
    pub enable_contagion: bool,
    pub max_contagion_depth: usize,      // default: 2
}

pub enum Classification { Protocol, SlotOnly, ReadOnly }

// Token heuristic: all written slots are high-entropy keccak mapping entries
// AND no low-numbered state slots (< 20) exist → classify as SlotOnly.
// Catches rebasing tokens (stETH, aTokens) that write 3+ mapping slots.

pub struct StateDiff {
    pub accounts: HashMap<Address, AccountDiff>,
}

pub struct AccountDiff {
    pub info_changed: bool,
    pub new_balance: Option<U256>,
    pub new_nonce: Option<u64>,
    pub new_code: Option<Bytecode>,
    pub storage_written: HashMap<U256, U256>,
    pub storage_read: HashSet<U256>,
}
```

**EvmExecutor** — creates and runs the revm EVM instance.

```rust
pub struct EvmExecutor;

impl EvmExecutor {
    // Read-only: never commits to DirtyStore
    pub fn call(state: &ForkState, from: Address, to: Address,
                data: Bytes, value: U256, gas_limit: u64)
        -> Result<ExecutionResult, MirageError>;

    // Writes to DirtyStore, triggers DiffClassifier
    pub fn transact(state: &mut ForkState, from: Address, to: Option<Address>,
                    data: Bytes, value: U256, gas_limit: u64)
        -> Result<(ExecutionResult, StateDiff), MirageError>;
}

// revm 36.x construction pattern:
// let mut evm = Context::mainnet()
//     .with_db(db)
//     .with_block(block_env)
//     .with_cfg(cfg_env)
//     .build_mainnet();
// let result = evm.transact_commit(tx_env)?;
// cfg: Cancun spec, nonce/balance checks disabled by default

// std::mem::replace dance: Context takes ownership of DB.
// After execution, extract modified DB from evm.ctx.journaled_state.database.
```

**UpstreamRpc** (in `provider.rs`) — alloy HTTP+WS provider with rate limiting.

```rust
pub struct UpstreamRpc {
    http: Arc<dyn alloy::providers::Provider>,
    ws:   Option<Arc<dyn alloy::providers::Provider>>,
    rps_limiter: tokio::sync::Semaphore,  // --upstream-rps, default 100
    burst: u32,                            // --upstream-burst, default 200
}

// Construction:
// let http = ProviderBuilder::new().on_http(rpc_url.parse()?);
// let ws   = ProviderBuilder::new().on_ws(connect_pubsub(ws_url).await?).await?;
//
// All fetch methods retry with exponential backoff (max 3 attempts).
// On all retries exhausted: MirageError::UpstreamError(-32099).
```

**Local transaction pipeline** (in `fork.rs`):
1. Execute via `EvmExecutor::transact`
2. Run `DiffClassifier::classify` on the state diff
3. Apply account changes + storage overrides to `DirtyStore`
4. Update watch list with newly classified protocol contracts
5. Generate tx hash, store receipt, advance local block counter

---

### Unit 2: JSON-RPC Compatibility Layer

**Files:** `rpc.rs`, `main.rs`
**~400 lines**

#### Quick Reference

**Supported eth_* methods:**

| Method | Behavior |
|--------|----------|
| `eth_blockNumber` | Local block counter (starts at mainnet head, increments per local tx) |
| `eth_chainId` | Configured chain ID |
| `eth_getBalance` | DirtyStore → upstream |
| `eth_getStorageAt` | DirtyStore → upstream |
| `eth_getCode` | DirtyStore → BytecodeCache → upstream |
| `eth_getTransactionCount` | DirtyStore nonce → upstream |
| `eth_call` | EvmExecutor::call, no state mutation |
| `eth_sendTransaction` | EvmExecutor::transact, triggers DiffClassifier |
| `eth_sendRawTransaction` | Decode EIP-2718 (types 0-3), then same path |
| `eth_getTransactionReceipt` | Local receipt store |
| `eth_getTransactionByHash` | Local tx store |
| `eth_getLogs` | Local logs only (not proxied to mainnet) |
| `eth_getBlockByNumber` | Synthetic local block |
| `eth_getBlockByHash` | Synthetic local block by hash |
| `eth_estimateGas` | Execute against temp snapshot, return gas + 20% buffer |
| `eth_gasPrice` | `0x1` (or sine-wave with `--sim-gas`) |
| `eth_feeHistory` | Constant synthetic values (or sine-wave) |
| `eth_maxPriorityFeePerGas` | `0x0` (or simulated) |
| `net_version` | Chain ID as decimal string |
| `web3_clientVersion` | `"mirage-rs/2.0.0"` |

**Hardhat/Anvil compatibility methods** (both prefixes work identically):

| Method | Effect |
|--------|--------|
| `hardhat_impersonateAccount` / `anvil_impersonateAccount` | Allow `from` without signature |
| `hardhat_stopImpersonatingAccount` / `anvil_stopImpersonatingAccount` | Remove from impersonation list |
| `hardhat_setBalance` / `anvil_setBalance` | Write balance to DirtyStore |
| `hardhat_setCode` / `anvil_setCode` | Write bytecode to DirtyStore |
| `hardhat_setStorageAt` / `anvil_setStorageAt` | Write slot to DirtyStore |
| `hardhat_mine` / `anvil_mine` | Advance local block counter (N blocks, configurable interval) |
| `hardhat_reset` / `anvil_reset` | Destructive reset: clear DirtyStore, ReadCache, watch list |
| `hardhat_setNextBlockBaseFeePerGas` / `anvil_setNextBlockBaseFeePerGas` | Set next block base fee |
| `anvil_setNonce` | Write nonce to DirtyStore |
| `hardhat_setCoinbase` / `anvil_setCoinbase` | Set coinbase for subsequent blocks |
| `anvil_setPrevRandao` | Set PREVRANDAO for next block |

**evm_* test utilities:**

| Method | Effect |
|--------|--------|
| `evm_snapshot` | Capture DirtyStore → returns snapshot ID |
| `evm_revert` | Restore to snapshot (single-use; `-32001` on second call) |
| `evm_mine` | Mine N empty blocks |
| `evm_increaseTime` | Add seconds to current timestamp |
| `evm_setNextBlockTimestamp` | Set exact timestamp for next block |

**mirage_* extensions** (state manipulation):

| Method | Effect |
|--------|--------|
| `mirage_setBalance` | Write balance to DirtyStore |
| `mirage_setCode` | Override bytecode at address |
| `mirage_setStorageAt` | Write slot to DirtyStore |
| `mirage_mintERC20` | Auto-detect balance slot, mint tokens atomically |
| `mirage_prefetchSlots` | Pre-warm ReadCache for specific slots |
| `mirage_prefetchAccount` | Pre-warm account info |
| `mirage_computeDomainSeparator` | Call contract, return EIP-712 domain separator |
| `mirage_cleanup` | Prune stale PID files and artifacts |

**mirage_* watch list:**

| Method | Effect |
|--------|--------|
| `mirage_watchContract` | Add to watch list (manual) |
| `mirage_unwatchContract` | Remove + add to unwatch list |
| `mirage_getWatchList` | Return watch list with metadata |
| `mirage_getDirtySlots` | Return dirty storage for an address |
| `mirage_status` | Full instance status snapshot |

**mirage_* resource management:**

| Method | Effect |
|--------|--------|
| `mirage_getResourceUsage` | Memory, cache hit rate, pressure, upstream call counts |
| `mirage_setResourceLimits` | Adjust limits at runtime |

**mirage_* position helpers:**

| Method | Effect |
|--------|--------|
| `mirage_getPosition` | Token balances + protocol-specific position state (uniswap-v3-position, aave-v3-account, raw-balances) |
| `mirage_subscribeEvents` | SSE or WebSocket event stream filtered by address/topics |

**mirage_* scenario runner:**

| Method | Effect |
|--------|--------|
| `mirage_beginScenarioSet` | Create scenario set from baseline state |
| `mirage_defineScenario` | Add scenario to set (tx list + tracked addresses) |
| `mirage_runScenarioSet` | Execute set (sequential or parallel) → job ID |
| `mirage_getScenarioResults` | Poll job status and results |
| `mirage_compareScenarios` | Ranked comparison by pnl / gas / state_diff |

**Error codes:**

| Code | Name | Trigger |
|------|------|---------|
| -32700 | PARSE_ERROR | Invalid JSON |
| -32600 | INVALID_REQUEST | Not valid JSON-RPC 2.0 |
| -32601 | METHOD_NOT_FOUND | Unknown method |
| -32602 | INVALID_PARAMS | Bad param types or values |
| -32603 | INTERNAL_ERROR | Internal panic/bug |
| -32001 | SNAPSHOT_NOT_FOUND | `evm_revert` with unknown ID |
| -32003 | NONCE_TOO_LOW | With `--strict-nonce` |
| -32004 | NONCE_TOO_HIGH | With `--strict-nonce` |
| -32010 | INVALID_FROM | Not impersonated, no private key |
| -32015 | EXECUTION_REVERTED | EVM reverted (includes revert reason) |
| -32020 | SLOT_DETECTION_FAILED | `mirage_mintERC20` can't find slot |
| -32030 | WATCH_LIST_FULL | Watch list at `max_watched_contracts` |
| -32040 | UNKNOWN_PROTOCOL_TYPE | `mirage_getPosition` unknown type string |
| -32050 | SET_NOT_FOUND | Scenario set ID missing |
| -32051 | SET_ALREADY_RUNNING | Scenario already executing |
| -32052 | SET_HAS_NO_SCENARIOS | Empty scenario set |
| -32053 | PARALLEL_UNAVAILABLE | Parallel mode not available |
| -32054 | JOB_NOT_FOUND | Scenario job ID missing |
| -32055 | JOB_NOT_COMPLETE | Results requested before done |
| -32099 | UPSTREAM_ERROR | Upstream RPC failure after retries |

**Server setup** (axum + jsonrpsee in `main.rs`):

```rust
// JSON-RPC via jsonrpsee
let server = ServerBuilder::default()
    .build(format!("{}:{}", config.host, config.port))
    .await?;

// SSE/REST endpoints via axum (events, health, cleanup)
let app = Router::new()
    .route("/events/:stream_id", get(sse_handler))
    .route("/events/:stream_id", delete(unsubscribe_handler))
    .route("/health", get(health_handler))
    .layer(CorsLayer::permissive());

// Startup sequence:
// 1. Bind port, write /tmp/mirage-${port}.pid
// 2. Verify upstream connectivity via eth_blockNumber
// 3. Write /tmp/mirage-${port}-status.json
// 4. Emit: tracing::info!("mirage ready port={} chain={}", port, chain_id)

// Watchdog: if --watchdog-timeout elapses with no inbound request, exit(0)
```

**Transaction format handling** (EIP-2718 types 0-3):
- Type 0: legacy RLP, `v`/`r`/`s` with optional EIP-155 replay protection
- Type 1: EIP-2930 access list (`0x01 || RLP(...)`)
- Type 2: EIP-1559 fee market (`0x02 || RLP(...)`)
- Type 3: EIP-4844 blob transactions (`0x03 || RLP(...)`) — blob data ignored unless `--validate-blobs`
- Signature verification skipped by default; enabled with `--verify-signatures`
- Impersonation bypasses signature check regardless of `--verify-signatures`

---

### Unit 3: Transaction Replay & Speculative Execution

**Files:** `replay.rs`
**~300 lines**

#### Quick Reference

**TargetedFollower** — subscribes to mainnet blocks via WebSocket, replays only transactions touching watched contracts.

```rust
pub struct TargetedFollower {
    upstream: Arc<UpstreamRpc>,
    state: Arc<parking_lot::RwLock<MirageState>>,
    classifier: DiffClassifier,
    config: FollowerConfig,
}

pub struct FollowerConfig {
    pub ws_url: String,
    pub http_url: String,
    pub block_budget: Duration,   // max replay time per block, default: 10s
    pub filter_addresses: Option<Vec<Address>>,   // manual address filter
    pub filter_selectors: Option<Vec<[u8; 4]>>,   // manual selector filter
}

// Main loop:
// 1. Subscribe to newHeads via alloy WS provider
// 2. For each block: fetch tx list
// 3. Filter: tx.to ∈ watch_list (HashSet lookup, ~1µs per tx)
// 4. Replay matched txs in block order via EvmExecutor::transact_detached
// 5. Apply resulting StateDiff to DirtyStore
// 6. Run DiffClassifier for contagion (depth-capped at max_contagion_depth)
// 7. Log reverted replays; do not corrupt state
```

**TxReplay** — fetch and replay a specific tx by hash.

```rust
pub struct TxReplay {
    pub tx_hash: B256,
}

impl TxReplay {
    pub async fn execute(
        &self,
        upstream: &UpstreamRpc,
        state: &mut ForkState,
    ) -> Result<(ExecutionResult, StateDiff), MirageError>;
    // Fetches tx from upstream, reconstructs TxEnv,
    // runs EvmExecutor::transact, returns result + diff
}
```

**SpeculativeExecutor** — execute a tx against a CoW fork of current state. Does NOT broadcast or commit to DirtyStore.

```rust
pub struct SpeculativeExecutor {
    /// (pending_tx_hash, base_state_block) → result
    cache: HashMap<(B256, u64), SpeculativeResult>,
}

pub struct SpeculativeResult {
    pub result: ExecutionResult,
    pub state_diff: StateDiff,
    pub read_set: HashSet<(Address, U256)>,  // for cache invalidation
    pub computed_at: Instant,
}

// Invalidation triggers:
// 1. tx included on-chain (speculation resolved)
// 2. new block writes to any slot in read_set
// 3. block number or timestamp deviated from assumed values

// Memory cost per pending tx: ~12 KB (CoW overlay, 200 slots)
// Compute cost: single revm transact() call, typically <1ms for cached state
```

**StateDiff** — what changed after executing a transaction.

```rust
pub struct StateDiff {
    pub accounts: HashMap<Address, AccountDiff>,
    pub logs: Vec<LogEntry>,
    pub gas_used: u64,
    pub success: bool,
    pub output: Bytes,
}

pub struct AccountDiff {
    pub info_changed: bool,
    pub new_balance: Option<U256>,
    pub new_nonce: Option<u64>,
    pub new_code: Option<Bytecode>,
    pub storage_written: HashMap<U256, U256>,   // slot → new value
    pub storage_read: HashSet<U256>,             // slots read (for speculation)
}

pub struct LogEntry {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    pub log_index: u32,
}
```

**Gas profiling** — captured from revm's execution result.

```rust
// revm returns ExecutionResult with:
//   gas_used: u64
//   gas_refunded: u64
//   output: Output (bytes or deployment address)
//
// For per-opcode profiling, use revm's Inspector trait:
//   impl Inspector<DB> for GasProfiler {
//       fn step(&mut self, interp: &mut Interpreter, ctx: &mut EvmContext<DB>) {
//           self.record(interp.current_opcode(), interp.gas.remaining());
//       }
//   }
// GasProfiler is opt-in (perf overhead); not enabled by default.
```

**Block-STM parallel execution** (for historical replay mode only):

```rust
pub struct MultiVersionStore {
    // DashMap for concurrent access during parallel block execution
    versions: dashmap::DashMap<(Address, U256), Vec<VersionEntry>>,
}

pub struct VersionEntry {
    pub tx_index: usize,
    pub value: U256,
    pub incarnation: u32,  // incremented on re-execution after conflict
}

// Algorithm:
// 1. Execute all txs in block optimistically in parallel (rayon threadpool)
// 2. Track read/write sets in MultiVersionStore
// 3. Detect conflicts: tx_i reads slot that tx_j < tx_i already wrote
// 4. Re-execute conflicting txs sequentially
// 5. Typical DeFi blocks: <5% conflict rate (Saraph & Herlihy, 2019)
// Block-STM only used in --mode historical; live mode replays 5-15 txs/block
```

---

### Unit 4: Scenario System

**Files:** `scenario.rs`
**~350 lines**

#### Quick Reference

**Scenario definition** (JSON via RPC; TOML file format for built-ins):

```toml
# tests/scenarios/uniswap_v3_entry.toml
[scenario]
name = "uniswap_v3_lp_entry"
description = "Add concentrated liquidity to WETH/USDC 0.05% pool"

[[transactions]]
from = "0x1234...golem"
to   = "0x88e6...pool_manager"
data = "0x..."    # NonfungiblePositionManager.mint() calldata
value = "0x0"
gas   = "0x7a120"

[assertions]
# Verify pool entered watch list after tx
watch_list_contains = ["0x88e6...pool"]
# Verify position NFT minted
token_balance_gte = { token = "0x...nft", address = "0x...golem", amount = "1" }

[track]
addresses = ["0x88e6...pool", "0x...weth", "0x...usdc"]
```

**Built-in scenarios:**

| Scenario | File | Description |
|----------|------|-------------|
| ETH price crash | `tests/scenarios/eth_crash.toml` | 20-40 directional WETH→USDC swaps, 10-90% drop |
| Volume spike | `tests/scenarios/volume_spike.toml` | 100 two-directional swaps, 2/3 buy 1/3 sell |
| Uniswap V3 LP entry | `tests/scenarios/uniswap_v3_entry.toml` | Mint concentrated liquidity position |
| Aave liquidation | `tests/scenarios/aave_liquidation.toml` | Drop ETH oracle 20%, liquidate a position |
| New pool bootstrap | `tests/scenarios/new_pool.toml` | Deploy token, create V3 pool, add liquidity |

**ScenarioSet and ScenarioRunner:**

```rust
pub struct ScenarioSet {
    pub id: String,
    pub baseline_snapshot_id: u64,  // evm_snapshot taken at set creation
    pub scenarios: Vec<Scenario>,
    pub status: ScenarioSetStatus,
}

pub struct Scenario {
    pub id: String,
    pub name: String,
    pub transactions: Vec<TransactionRequest>,
    pub track_addresses: Vec<Address>,
    pub max_gas: Option<u64>,
    pub timeout: Duration,
}

pub struct ScenarioRunner {
    state: Arc<parking_lot::RwLock<MirageState>>,
}

impl ScenarioRunner {
    // Sequential: run each scenario branch from baseline, revert after each
    pub async fn run_sequential(&self, set: &ScenarioSet) -> Vec<ScenarioResult>;

    // Parallel: each scenario gets a CowState branch from baseline
    // Branches are independent; no locking needed between them
    pub async fn run_parallel(&self, set: &ScenarioSet) -> Vec<ScenarioResult>;
}

pub struct ScenarioResult {
    pub scenario_id: String,
    pub name: String,
    pub status: ScenarioStatus,      // Success | Reverted | Timeout | GasExceeded | Error
    pub gas_used: u64,
    pub wall_time_ms: u64,
    pub peak_memory_bytes: u64,
    pub final_balances: HashMap<Address, U256>,
    pub position_state: serde_json::Value,
    pub logs: Vec<LogEntry>,
    pub revert_reason: Option<String>,
}

pub enum ScenarioStatus {
    Success,
    Reverted,
    Timeout,
    GasExceeded,
    Error(String),
}
```

**Scenario composition** — snapshot branching:

```rust
// Sequential pattern:
// 1. baseline_snapshot_id = evm_snapshot()
// 2. For each scenario:
//    a. Run transactions
//    b. Collect results
//    c. evm_revert(baseline_snapshot_id)  -- restores state
//    d. Take new snapshot for next branch (snapshot is single-use)

// CoW parallel pattern:
// 1. Freeze baseline as Arc<HashMap> from current DirtyStore
// 2. Each scenario gets CowState::branch(&baseline)
// 3. Run in parallel (rayon or tokio tasks)
// 4. No revert needed; branches are discarded after result collection
```

**ScenarioJob** (async execution tracking):

```rust
pub struct ScenarioJob {
    pub job_id: String,
    pub set_id: String,
    pub status: JobStatus,   // Running | Complete | Failed
    pub results: Option<Vec<ScenarioResult>>,
    pub total_wall_time_ms: Option<u64>,
}

// Jobs run in background tokio tasks
// Poll via mirage_getScenarioResults until status != Running
// Results include comparison data for mirage_compareScenarios (pnl/gas/state_diff metrics)
```

**Scenario file paths:**

```
tests/
└── scenarios/
    ├── eth_crash.toml
    ├── volume_spike.toml
    ├── uniswap_v3_entry.toml
    ├── aave_liquidation.toml
    └── new_pool.toml
```

---

### Unit 5: Bardo Integration

**Files:** `integration.rs`, `resources.rs`
**~250 lines**

#### Quick Reference

**MirageClient** — async client for golem components connecting to mirage-rs over local JSON-RPC.

```rust
use golem_core::config::GolemConfig;

pub struct MirageConfig {
    pub url: String,          // e.g., "http://127.0.0.1:8545"
    pub timeout: Duration,    // per-request timeout, default: 30s
    pub retry_attempts: u32,  // on connection error, default: 3
    pub retry_backoff: Duration,  // initial backoff, default: 500ms
}

impl MirageConfig {
    /// Derive from GolemConfig (reads mirage.url, mirage.port from TOML)
    pub fn from_golem_config(config: &GolemConfig) -> Self { ... }

    pub fn default_local() -> Self {
        Self {
            url: "http://127.0.0.1:8545".to_string(),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_backoff: Duration::from_millis(500),
        }
    }
}

pub struct MirageClient {
    config: MirageConfig,
    inner: reqwest::Client,
}

impl MirageClient {
    pub async fn new(config: MirageConfig) -> Result<Self, MirageError>;

    // Core operations used by golem components
    pub async fn eth_call(&self, req: TransactionRequest) -> Result<Bytes, MirageError>;
    pub async fn eth_send_transaction(&self, req: TransactionRequest) -> Result<B256, MirageError>;
    pub async fn evm_snapshot(&self) -> Result<u64, MirageError>;
    pub async fn evm_revert(&self, id: u64) -> Result<bool, MirageError>;
    pub async fn mirage_watch_contract(&self, addr: Address) -> Result<(), MirageError>;
    pub async fn mirage_get_position(&self, req: PositionRequest) -> Result<PositionSnapshot, MirageError>;
    pub async fn mirage_status(&self) -> Result<MirageStatus, MirageError>;
    pub async fn mirage_get_resource_usage(&self) -> Result<ResourceUsage, MirageError>;
    pub async fn mirage_begin_scenario_set(&self, baseline: &str) -> Result<String, MirageError>;
    pub async fn mirage_define_scenario(&self, set_id: &str, scenario: &Scenario) -> Result<String, MirageError>;
    pub async fn mirage_run_scenario_set(&self, set_id: &str, mode: RunMode) -> Result<String, MirageError>;
    pub async fn mirage_get_scenario_results(&self, job_id: &str) -> Result<ScenarioJob, MirageError>;

    // Start/stop for test harnesses
    pub async fn wait_ready(&self, timeout: Duration) -> Result<(), MirageError>;
    // Polls mirage_status every 500ms until status == "ready" or timeout
}

pub enum RunMode { Sequential, Parallel }
```

**Golem-chain integration** (Plan 09 will use MirageClient as its test backend):

```rust
// In golem-chain tests: instead of connecting to live RPC,
// tests spawn mirage-rs and pass MirageClient as the provider.
//
// Pattern:
// let mirage = spawn_mirage_test_instance().await?;
// let client = MirageClient::new(mirage.config()).await?;
// // Run test txs against mirage, assert state, tear down
// mirage.shutdown().await;

pub struct MirageTestInstance {
    process: tokio::process::Child,
    port: u16,
    pid_file: PathBuf,
}

impl MirageTestInstance {
    pub fn config(&self) -> MirageConfig { ... }
    pub async fn shutdown(&mut self) -> Result<(), MirageError>;
    // Sends mirage_shutdown, waits 5s, then SIGTERM
}

pub async fn spawn_mirage_test_instance(
    rpc_url: Option<&str>,
    port: Option<u16>,
) -> Result<MirageTestInstance, MirageError>;
// Spawns mirage-rs binary, polls mirage_status until ready
// port defaults to 18545 to avoid conflicting with dev instance on 8545
```

**Heartbeat integration** (Plan 15 uses MirageClient for tick execution):

```rust
// Heartbeat polls mirage_get_position on each gamma tick (~250ms)
// Events from mirage_subscribe_events trigger immediate range checks
// resource_pressure from mirage_get_resource_usage feeds into CorticalState

// MirageClient::subscribe_events returns a tokio Stream:
pub async fn subscribe_events(
    &self,
    filter: EventFilter,
) -> Result<impl futures::Stream<Item = MirageEvent>, MirageError>;
// Uses SSE transport internally

pub struct EventFilter {
    pub addresses: Option<Vec<Address>>,
    pub topics: Option<Vec<B256>>,
}

pub struct MirageEvent {
    pub block_number: u64,
    pub tx_hash: B256,
    pub log_index: u32,
    pub contract: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    pub source: EventSource,  // LocalTx | FollowerReplay
    pub decoded: Option<serde_json::Value>,
}
```

**ResourceModel** (in `resources.rs`):

```rust
pub struct ResourceModel {
    pub profile: Profile,
    pub max_memory_bytes: u64,
    pub max_watched_contracts: usize,
    pub cache_capacity: usize,
    pub cache_ttl: Duration,
}

pub enum Profile {
    Micro,     // 256 MB, 32 contracts, 5_000 cache entries
    Standard,  // 512 MB, 64 contracts, 10_000 cache entries (default)
    Power,     // 2 GB,   256 contracts, 50_000 cache entries
}

pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub resource_pressure: f64,    // 0.0 = idle, 1.0 = at limit
    pub cache_hit_rate: f64,
    pub cache_entries: usize,
    pub cache_capacity: usize,
    pub watch_list_size: usize,
    pub dirty_slot_count: u64,
    pub upstream_rpc_calls: u64,
    pub upstream_rpc_errors: u64,
    pub mode: MirageMode,
    pub disk_usage_bytes: u64,
}

pub enum MirageMode { Live, Historical, Proxy }

// Pressure tiers (hard-coded):
// < 0.5  → green: new forks allowed
// ≥ 0.5  → warning: log, allow forks
// ≥ 0.7  → throttle: evict ReadCache LRU, demote new contracts to SlotOnly, refuse forks
// ≥ 0.9  → emergency: stop TargetedFollower replay entirely, mode = Proxy, emit CorticalState signal

// Eviction: LRU on ReadCache when throttle tier hit.
// DirtyStore is never evicted (preserves local state correctness).
// BytecodeCache: LRU by code_hash, capacity = min(10_000, profile.cache_capacity / 5).

// OS-level spawn gate (checked at startup):
// required = profile.max_memory + 128 MB
// available = sysinfo::System::available_memory()
// if available < required: exit(2) with structured error message
```

---

## Failure Recovery

**revm 36.x API changes.** The prd2 specs use `Context::mainnet().with_db(...).build_mainnet()` and `evm.transact_commit(tx)`. If `revm::primitives` module paths have shifted, check `revm::context` and `revm::handler`. The `Database` trait's four methods (`basic`, `code_by_hash`, `storage`, `block_hash`) are stable. If the `build_mainnet()` builder pattern changed, fall back to `Evm::builder().with_db(db).build()`.

**Alloy provider compilation fails.** Use `ProviderBuilder::new().on_http(url)` for HTTP and `ProviderBuilder::new().on_ws(WsConnect::new(url)).await` for WebSocket. Feature flags required: `provider-http`, `provider-ws`, `transport-http`, `transport-ws`. If `connect_pubsub` is renamed, check `alloy::providers::ws`.

**Port conflict on 8545.** Default port is 8545 (matching Anvil). Test harnesses should use `--port 18545`. If startup binding fails, the error is `MirageError::BindFailed(port)` and the process exits with code 1.

**Memory pressure from cached slots.** LRU eviction fires at the 0.7 throttle tier. `max_slots` per `ReadCache` controlled by `--cache-size` (default 10_000). If memory still climbs, reduce with `mirage_setResourceLimits` at runtime.

**Scenario replay diverges from recorded.** State may have drifted from the recorded baseline. Use `evm_snapshot` before recording the baseline, store the snapshot ID, and replay from that exact ID. For scenarios recorded against a specific block, use `--mode historical --from-block N` to pin the baseline.

**Mainnet tx reverts during TargetedFollower replay.** Expected behavior when the golem's local state diverges from mainnet (e.g., liquidity changed past a swap's slippage tolerance). The follower logs the revert and continues. Check `mirage_status.divergence_detected` to confirm revert was incidental rather than systematic.

**Upstream RPC rate limit.** Reduce `--upstream-rps` (default 100). Use a private archive node for heavy fork workloads. `prefetchSlots` before complex transactions to avoid mid-execution fetch failures.

**WebSocket subscription drops.** TargetedFollower reconnects with exponential backoff. No action needed in golem clients.

---

## Testing Checkpoint

```bash
cargo check -p mirage-rs
cargo test -p mirage-rs -- --nocapture
```

Expected test output:
```
test fork::tests::hybrid_db_dirty_store_wins ... ok
test fork::tests::hybrid_db_read_cache_prevents_rpc ... ok
test fork::tests::hybrid_db_partial_dirty_merge ... ok
test fork::tests::dirty_store_snapshot_revert_roundtrip ... ok
test fork::tests::dirty_store_snapshot_single_use ... ok
test fork::tests::diff_classifier_protocol_at_threshold ... ok
test fork::tests::diff_classifier_token_heuristic ... ok
test fork::tests::diff_classifier_unwatch_prevents_readd ... ok
test cow::tests::cow_branches_share_baseline ... ok
test cow::tests::cow_branches_are_isolated ... ok
test cow::tests::cow_memory_scales_with_overlays ... ok
test cow::tests::bytecode_cache_no_ttl ... ok
test replay::tests::speculative_exec_no_state_commit ... ok
test replay::tests::state_diff_account_and_storage ... ok
test replay::tests::block_stm_matches_sequential ... ok
test scenario::tests::scenario_runner_basic ... ok
test scenario::tests::scenario_runner_revert_restores_baseline ... ok
test integration::tests::mirage_client_wait_ready ... ok
test resources::tests::pressure_tiers ... ok
test resources::tests::lru_eviction_at_capacity ... ok
```

Integration test:
```bash
# Start mirage-rs with a mock upstream (or real RPC via env)
MIRAGE_RPC_URL="${RPC_URL:-http://localhost:8545}" \
    cargo test -p mirage-rs --test integration -- --nocapture
```

Expected integration test steps:
1. Spawn mirage-rs on port 18545 (avoid conflicting with dev instance)
2. Connect via `MirageClient::new(MirageConfig { url: "http://127.0.0.1:18545", .. })`
3. Call `mirage_client.wait_ready(Duration::from_secs(10)).await`
4. Execute a simulated ETH transfer via `eth_send_transaction`
5. Assert sender balance decreased in `eth_get_balance`
6. Assert receiver balance increased
7. Assert `StateDiff` contains the two balance changes
8. Call `evm_snapshot`, modify state, call `evm_revert`, assert state restored
9. Shutdown via `mirage_test_instance.shutdown().await`

```bash
# Expected integration output:
# test integration_eth_transfer_state_diff ... ok
# test integration_snapshot_revert ... ok
# test integration_scenario_runner_cow_isolation ... ok
```

---

## Completion Report

*(Codex fills this after implementation)*

## Verification

### Invariants

<!-- INV-001: HybridDB read priority order -->
- **type**: cross_crate
- **module**: `mirage_rs::fork::HybridDB`
- **property**: Three-tier database read must check DirtyStore first, then ReadCache, then upstream RPC
- **formula**: `read(addr) = DirtyStore[addr] || ReadCache[addr] || upstream_fetch(addr)`
- **constraint**: Priority order is strict; no skipping tiers
- **test_fn**: `test_hybrid_db_tier_priority`
- **strategy**: unit
- **inputs**: `{"address": "Address", "in_dirty": "bool", "in_cache": "bool"}`
- **oracle**: Return DirtyStore value if present; else ReadCache; else upstream
- **severity**: spec
- **source**: plan Quick Reference Unit 1, HybridDB

<!-- INV-002: Cache TTL expiration -->
- **type**: numeric_range
- **module**: `mirage_rs::fork::HybridDB`
- **property**: Cache entries must expire after TTL and re-fetch from upstream
- **formula**: `cache_valid(entry) = (now - entry.cached_at) < cache_ttl`
- **constraint**: `cache_ttl ∈ [1ms, 60s]`, default 12s
- **test_fn**: `test_cache_ttl_expiration`
- **strategy**: proptest
- **inputs**: `{"cache_ttl": "[1, 60000]ms", "elapsed_time": "[0, 70000]ms"}`
- **oracle**: Entry valid iff `elapsed_time < cache_ttl`
- **severity**: spec
- **source**: plan Unit 1, HybridDB field `cache_ttl`

<!-- INV-003: DirtyStore partial merge -->
- **type**: sum_constraint
- **module**: `mirage_rs::fork::HybridDB`
- **property**: DirtyStore partial override merges dirty fields with upstream base when needs_upstream=true
- **formula**: `merged_account = DirtyAccount { balance: dirty.balance || upstream.balance, nonce: dirty.nonce || upstream.nonce, code: dirty.code || upstream.code }`
- **constraint**: All three fields (balance, nonce, code) must be either from DirtyStore OR upstream; no field is left undefined
- **test_fn**: `test_hybrid_db_partial_dirty_merge`
- **strategy**: unit
- **inputs**: `{"dirty_balance": "Option<U256>", "dirty_nonce": "Option<u64>", "dirty_code": "Option<Bytecode>"}`
- **oracle**: Result account has: (dirty.X.is_some()) ? dirty.X : upstream.X for each field
- **severity**: spec
- **source**: plan Unit 1, HybridDB::basic()

<!-- INV-004: DiffClassifier protocol_slot_threshold -->
- **type**: numeric_range
- **module**: `mirage_rs::fork::DiffClassifier`
- **property**: Account classified as Protocol if written slots ≥ protocol_slot_threshold
- **formula**: `classification = Protocol iff storage_written.len() >= protocol_slot_threshold`
- **constraint**: `protocol_slot_threshold > 0`, default 3, `storage_written.len() ∈ [0, ∞)`
- **test_fn**: `test_diff_classifier_protocol_threshold`
- **strategy**: proptest
- **inputs**: `{"slot_count": "[0, 100]", "threshold": "[1, 10]"}`
- **oracle**: If `slot_count >= threshold`, classify as Protocol; else check other heuristics
- **severity**: spec
- **source**: plan Unit 1, ClassificationConfig `protocol_slot_threshold`

<!-- INV-005: Watch list capacity bounds -->
- **type**: numeric_range
- **module**: `mirage_rs::fork::DiffClassifier`
- **property**: Watch list size must not exceed max_watched_contracts
- **formula**: `watch_list.len() <= max_watched_contracts`
- **constraint**: `max_watched_contracts > 0`, default 64, Micro profile 32, Standard 64, Power 256
- **test_fn**: `test_watch_list_capacity_enforced`
- **strategy**: unit
- **inputs**: `{"watch_list_size": "[0, 256]", "max_contracts": "[32, 256]"}`
- **oracle**: `watch_list.len() <= max_contracts` always true; returns error on overflow
- **severity**: spec
- **source**: plan Unit 1, ClassificationConfig `max_watched_contracts`; Unit 5 ResourceModel profiles

<!-- INV-006: Contagion depth cap -->
- **type**: numeric_range
- **module**: `mirage_rs::fork::DiffClassifier`
- **property**: Contagion propagation depth must not exceed max_contagion_depth
- **formula**: `contagion_depth(address) <= max_contagion_depth`
- **constraint**: `max_contagion_depth ≥ 1`, default 2
- **test_fn**: `test_contagion_depth_capped`
- **strategy**: unit
- **inputs**: `{"depth": "[1, 5]", "max_depth": "[2, 5]"}`
- **oracle**: Traversal stops at max_depth; no cycles allowed
- **severity**: spec
- **source**: plan Unit 1, ClassificationConfig `max_contagion_depth`

<!-- INV-007: Snapshot ID single-use guarantee -->
- **type**: state_machine
- **module**: `mirage_rs::fork::DirtyStore`
- **property**: Each snapshot ID can only be reverted to once; second revert call fails
- **formula**: `revert(id) succeeds iff (id exists in snapshots AND id not in reverted_ids)`
- **constraint**: Snapshot consumed on successful revert; invalid revert → error -32001
- **test_fn**: `test_dirty_store_snapshot_single_use`
- **strategy**: unit
- **inputs**: `{"snapshot_id": "u64"}`
- **oracle**: First `revert(id)` succeeds and removes from snapshot map; second call returns `-32001 SNAPSHOT_NOT_FOUND`
- **severity**: spec
- **source**: plan Unit 1, DirtyStore snapshot/revert semantics

<!-- INV-008: Block budget timeout per block -->
- **type**: numeric_range
- **module**: `mirage_rs::replay::TargetedFollower`
- **property**: Each block replay must complete within block_budget duration
- **formula**: `replay_time(block) <= block_budget`
- **constraint**: `block_budget > 0`, default 10s, type `Duration`
- **test_fn**: `test_block_budget_timeout_enforced`
- **strategy**: integration
- **inputs**: `{"block_budget": "[1, 30]s", "replay_duration": "[0, 60]s"}`
- **oracle**: Exceeding budget triggers skip to latest or error
- **severity**: code
- **source**: plan Unit 3, FollowerConfig `block_budget`

<!-- INV-009: TargetedFollower tx filter HashSet lookup -->
- **type**: numeric_range
- **module**: `mirage_rs::replay::TargetedFollower`
- **property**: Transaction filter lookup via address HashSet is O(1) per tx, ~1µs typical latency
- **formula**: `filter_lookup_time ≈ 1µs per tx`
- **constraint**: Lookup must complete before next block arrives (~12-13s Ethereum); batching allowed
- **test_fn**: `test_targeted_follower_filter_throughput`
- **strategy**: integration
- **inputs**: `{"watch_list_size": "[1, 256]", "tx_count": "[10, 10000]"}`
- **oracle**: Throughput acceptable if all txs processed before block timeout
- **severity**: code
- **source**: plan Unit 3, TargetedFollower main loop comment

<!-- INV-010: SpeculativeExecutor cache memory cost -->
- **type**: capacity
- **module**: `mirage_rs::replay::SpeculativeExecutor`
- **property**: Each pending tx cache entry consumes ~12 KB of memory
- **formula**: `memory_per_tx ≈ 12 KB`
- **constraint**: Total speculative cache memory = `pending_tx_count * 12 KB`
- **test_fn**: `test_speculative_executor_memory_per_tx`
- **strategy**: unit
- **inputs**: `{"pending_tx_count": "[1, 1000]"}`
- **oracle**: Measure heap size before/after adding speculative results; ≈12 KB per entry
- **severity**: code
- **source**: plan Unit 3, SpeculativeResult comment

<!-- INV-011: StateDiff invalidation on block write -->
- **type**: event_sequence
- **module**: `mirage_rs::replay::SpeculativeExecutor`
- **property**: Speculative result must be invalidated if new block writes to any slot in read_set
- **formula**: `invalidate(speculation) iff (block.storage_written ∩ speculation.read_set ≠ ∅)`
- **constraint**: No stale speculative results used after invalidating write
- **test_fn**: `test_speculative_invalidation_on_block_write`
- **strategy**: integration
- **inputs**: `{"read_set": "[Address, U256]", "block_writes": "[Address, U256]"}`
- **oracle**: Overlap detected; speculation removed from cache
- **severity**: spec
- **source**: plan Unit 3, SpeculativeExecutor invalidation triggers

<!-- INV-012: Block-STM conflict rate -->
- **type**: monotonic
- **module**: `mirage_rs::replay::MultiVersionStore`
- **property**: DeFi block conflict rate must be <5% in typical workloads (Saraph & Herlihy 2019)
- **formula**: `conflict_rate = (re_executions / total_txs) < 0.05`
- **constraint**: `conflict_rate ∈ [0, 1]`
- **test_fn**: `test_block_stm_conflict_rate`
- **strategy**: integration
- **inputs**: `{"block_txs": "Vec<Tx>"}`
- **oracle**: Execute block via Block-STM; measure re-executions / total_txs
- **severity**: spec
- **source**: plan Unit 3, MultiVersionStore algorithm comment (Saraph & Herlihy 2019)

<!-- INV-013: Scenario status enum valid transitions -->
- **type**: state_machine
- **module**: `mirage_rs::scenario::Scenario`
- **property**: Scenario status transitions follow strict ordering: pending → (Success | Reverted | Timeout | GasExceeded | Error)
- **formula**: `status_transition: Pending → (Success | Reverted | Timeout | GasExceeded | Error)`
- **constraint**: No cycles allowed; no transitions between Success/Reverted/Timeout/GasExceeded
- **test_fn**: `test_scenario_status_valid_transitions`
- **strategy**: unit
- **inputs**: `{"current_status": "ScenarioStatus", "next_status": "ScenarioStatus"}`
- **oracle**: Transition valid iff current is Pending and next is terminal, or pre-condition violated → Error
- **severity**: spec
- **source**: plan Unit 4, ScenarioStatus enum

<!-- INV-014: Scenario revert to baseline -->
- **type**: roundtrip
- **module**: `mirage_rs::scenario::ScenarioRunner`
- **property**: Sequential scenario execution must revert to baseline_snapshot_id after each scenario
- **formula**: `state_after_revert(baseline_id) = state_before_scenario`
- **constraint**: Each scenario sees identical starting state via evm_revert(baseline_snapshot_id)
- **test_fn**: `test_scenario_runner_revert_restores_baseline`
- **strategy**: integration
- **inputs**: `{"baseline_snapshot_id": "u64", "scenario_count": "[1, 10]"}`
- **oracle**: After revert, state matches pre-scenario snapshot; subsequent scenarios see same state
- **severity**: spec
- **source**: plan Unit 4, scenario composition sequential pattern

<!-- INV-015: Scenario timeout constraint -->
- **type**: numeric_range
- **module**: `mirage_rs::scenario::Scenario`
- **property**: Scenario execution must respect timeout duration
- **formula**: `execution_time <= timeout`
- **constraint**: `timeout > 0`, type `Duration`, typical values 5-60s
- **test_fn**: `test_scenario_timeout_enforced`
- **strategy**: integration
- **inputs**: `{"timeout": "[1, 60]s", "actual_time": "[0, 120]s"}`
- **oracle**: If `actual_time > timeout`, status = Timeout
- **severity**: code
- **source**: plan Unit 4, Scenario `timeout` field

<!-- INV-016: Scenario gas limit enforcement -->
- **type**: numeric_range
- **module**: `mirage_rs::scenario::Scenario`
- **property**: Scenario gas usage must not exceed max_gas if set
- **formula**: `gas_used <= max_gas or max_gas = None`
- **constraint**: `max_gas ∈ Option<u64>`, if Some(n) then n > 0
- **test_fn**: `test_scenario_gas_exceeded`
- **strategy**: unit
- **inputs**: `{"max_gas": "Option<u64>", "actual_gas": "u64"}`
- **oracle**: If Some(limit) and `actual_gas > limit`, status = GasExceeded
- **severity**: code
- **source**: plan Unit 4, Scenario `max_gas` field

<!-- INV-017: eth_estimateGas buffer -->
- **type**: numeric_range
- **module**: `mirage_rs::rpc`
- **property**: eth_estimateGas must add 20% buffer to measured gas
- **formula**: `estimated_gas = measured_gas + (measured_gas * 0.20)`
- **constraint**: Buffer percentage ≥ 20%, buffer ≥ 0
- **test_fn**: `test_eth_estimate_gas_buffer`
- **strategy**: unit
- **inputs**: `{"measured_gas": "[1, 30000000]", "buffer_pct": "0.20"}`
- **oracle**: `result == measured * 1.20`
- **severity**: spec
- **source**: plan Unit 2, eth_estimateGas behavior

<!-- INV-018: RPC timeout default -->
- **type**: numeric_range
- **module**: `mirage_rs::integration::MirageConfig`
- **property**: Default RPC timeout must be 30 seconds
- **formula**: `MirageConfig::default_local().timeout == 30s`
- **constraint**: `timeout > 0`, type `Duration`
- **test_fn**: `test_mirage_config_default_timeout`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: `Duration::from_secs(30)`
- **severity**: spec
- **source**: plan Unit 5, MirageConfig default_local()

<!-- INV-019: RPC retry attempts default -->
- **type**: numeric_range
- **module**: `mirage_rs::integration::MirageConfig`
- **property**: Default retry attempts must be 3 with 500ms backoff
- **formula**: `retry_attempts = 3, retry_backoff = 500ms`
- **constraint**: `retry_attempts ≥ 1`, `retry_backoff > 0`
- **test_fn**: `test_mirage_config_default_retries`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: `attempts = 3, backoff = Duration::from_millis(500)`
- **severity**: spec
- **source**: plan Unit 5, MirageConfig defaults

<!-- INV-020: Resource pressure numeric range -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceModel`
- **property**: Resource pressure must always be in [0.0, 1.0]
- **formula**: `resource_pressure ∈ [0.0, 1.0]`
- **constraint**: 0.0 = idle, 1.0 = at limit, intermediate values linear
- **test_fn**: `test_resource_pressure_bounds`
- **strategy**: proptest
- **inputs**: `{"memory_used": "[0, max_memory]", "max_memory": "[256MB, 2GB]"}`
- **oracle**: `pressure = memory_used / max_memory`; must be clamped to [0, 1]
- **severity**: spec
- **source**: plan Unit 5, ResourceUsage `resource_pressure`

<!-- INV-021: Resource pressure tier transitions -->
- **type**: state_machine
- **module**: `mirage_rs::resources::ResourceModel`
- **property**: Resource pressure tier transitions must follow specification
- **formula**: `tier = Green (< 0.5) | Warning (≥ 0.5) | Throttle (≥ 0.7) | Emergency (≥ 0.9)`
- **constraint**: Transitions are monotonic: Green → Warning → Throttle → Emergency
- **test_fn**: `test_resource_pressure_tier_transitions`
- **strategy**: unit
- **inputs**: `{"pressure": "[0.0, 1.0]"}`
- **oracle**: See hardcoded tier thresholds in resources.rs
- **severity**: spec
- **source**: plan Unit 5, ResourceModel pressure tiers comment

<!-- INV-022: Cache hit rate bounds -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceUsage`
- **property**: Cache hit rate must be in [0.0, 1.0]
- **formula**: `cache_hit_rate ∈ [0.0, 1.0]`
- **constraint**: 0.0 = all misses, 1.0 = all hits
- **test_fn**: `test_cache_hit_rate_bounds`
- **strategy**: proptest
- **inputs**: `{"hits": "[0, total]", "total": "[1, 100000]"}`
- **oracle**: `cache_hit_rate = hits / total`; clamped to [0, 1]
- **severity**: code
- **source**: plan Unit 5, ResourceUsage `cache_hit_rate`

<!-- INV-023: Profile memory allocation -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceModel::Profile`
- **property**: Each profile has fixed maximum memory allocation
- **formula**: `Micro: 256MB, Standard: 512MB, Power: 2GB`
- **constraint**: Must be powers of 2 in bytes, Micro < Standard < Power
- **test_fn**: `test_profile_memory_allocations`
- **strategy**: unit
- **inputs**: `{"profile": "Profile"}`
- **oracle**: See Profile enum variants
- **severity**: spec
- **source**: plan Unit 5, ResourceModel Profile enum

<!-- INV-024: Watch list per profile -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceModel::Profile`
- **property**: Each profile specifies max watched contracts
- **formula**: `Micro: 32, Standard: 64, Power: 256`
- **constraint**: Monotonic increase: Micro < Standard < Power
- **test_fn**: `test_profile_watch_limits`
- **strategy**: unit
- **inputs**: `{"profile": "Profile"}`
- **oracle**: See Profile-associated constants
- **severity**: spec
- **source**: plan Unit 5, ResourceModel profiles

<!-- INV-025: Cache capacity per profile -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceModel::Profile`
- **property**: Each profile specifies cache entry capacity
- **formula**: `Micro: 5_000, Standard: 10_000, Power: 50_000`
- **constraint**: Monotonic increase: Micro < Standard < Power
- **test_fn**: `test_profile_cache_capacities`
- **strategy**: unit
- **inputs**: `{"profile": "Profile"}`
- **oracle**: Profile capacity limits
- **severity**: spec
- **source**: plan Unit 5, ResourceModel profiles

<!-- INV-026: Bytecode cache capacity formula -->
- **type**: numeric_range
- **module**: `mirage_rs::resources::ResourceModel`
- **property**: Bytecode cache capacity = min(10_000, profile.cache_capacity / 5)
- **formula**: `bytecode_cache_capacity = min(10_000, profile_capacity / 5)`
- **constraint**: Always ≤ 10_000, always ≤ profile_capacity / 5
- **test_fn**: `test_bytecode_cache_capacity`
- **strategy**: unit
- **inputs**: `{"profile": "Profile"}`
- **oracle**: Compute for each profile: min(10_000, capacity/5)
- **severity**: spec
- **source**: plan Unit 5, ResourceModel bytecode cache comment

<!-- INV-027: Memory pressure startup check -->
- **type**: numeric_range
- **module**: `mirage_rs::resources`
- **property**: At startup, available OS memory must exceed profile.max_memory + 128 MB
- **formula**: `available_memory >= profile.max_memory_bytes + 128 * 1024 * 1024`
- **constraint**: Startup fails (exit 2) if insufficient memory
- **test_fn**: `test_memory_startup_gate`
- **strategy**: integration
- **inputs**: `{"profile": "Profile", "available_mb": "[256, 8192]"}`
- **oracle**: Check sysinfo::System::available_memory(); must exceed requirement
- **severity**: spec
- **source**: plan Unit 5, OS-level spawn gate comment

<!-- INV-028: LRU eviction at throttle tier -->
- **type**: state_machine
- **module**: `mirage_rs::resources`
- **property**: When resource pressure ≥ 0.7, ReadCache LRU eviction is triggered
- **formula**: `if pressure >= 0.7 then evict_lru(ReadCache)`
- **constraint**: Only ReadCache evicted, never DirtyStore
- **test_fn**: `test_lru_eviction_at_throttle_tier`
- **strategy**: integration
- **inputs**: `{"pressure": "[0.6, 0.8]"}`
- **oracle**: At pressure ≥ 0.7, oldest cache entry is removed
- **severity**: spec
- **source**: plan Unit 5, eviction policy comment

<!-- INV-029: Upstream RPC rate limiter -->
- **type**: numeric_range
- **module**: `mirage_rs::provider::UpstreamRpc`
- **property**: Upstream RPC requests must respect rate limit (default 100 RPS)
- **formula**: `rps_limiter: Semaphore with capacity 100`
- **constraint**: `rps ∈ (0, ∞)`, burst ∈ (0, ∞), default rps=100, burst=200
- **test_fn**: `test_upstream_rps_limit`
- **strategy**: integration
- **inputs**: `{"rps": "[10, 500]", "request_rate": "[10, 1000]"}`
- **oracle**: Requests blocked if rate exceeds limit
- **severity**: code
- **source**: plan Unit 1, UpstreamRpc rps_limiter

<!-- INV-030: Upstream RPC exponential backoff retry -->
- **type**: numeric_range
- **module**: `mirage_rs::provider::UpstreamRpc`
- **property**: Upstream RPC retries use exponential backoff, max 3 attempts
- **formula**: `backoff_delay = base_delay * (2 ^ attempt_num), max_attempts = 3`
- **constraint**: `base_delay > 0`, `attempt_num ∈ [0, 2]`
- **test_fn**: `test_upstream_exponential_backoff`
- **strategy**: unit
- **inputs**: `{"attempt": "[0, 2]", "base_delay": "[100, 1000]ms"}`
- **oracle**: Delay = base * 2^attempt; after 3 failures, return UpstreamError(-32099)
- **severity**: code
- **source**: plan Unit 1, UpstreamRpc retry comment

<!-- INV-031: EIP-2718 transaction type handling -->
- **type**: state_machine
- **module**: `mirage_rs::rpc`
- **property**: All EIP-2718 transaction types (0-3) must be correctly parsed and executed
- **formula**: `Type 0: legacy RLP | Type 1: EIP-2930 (0x01 || RLP) | Type 2: EIP-1559 (0x02 || RLP) | Type 3: EIP-4844 (0x03 || RLP)`
- **constraint**: Signature verification skipped by default; impersonation bypasses checks
- **test_fn**: `test_eip2718_type_parsing`
- **strategy**: unit
- **inputs**: `{"tx_type": "[0, 3]", "tx_data": "Bytes"}`
- **oracle**: Correctly decode and execute based on type prefix
- **severity**: spec
- **source**: plan Unit 2, Transaction format handling

<!-- INV-032: Account impersonation validity -->
- **type**: state_machine
- **module**: `mirage_rs::rpc`
- **property**: Impersonated accounts can send transactions without signatures; non-impersonated accounts require valid signature OR private key
- **formula**: `tx_valid = (sender in impersonated_set) OR (signature valid OR has_private_key)`
- **constraint**: No invalid states
- **test_fn**: `test_account_impersonation_validity`
- **strategy**: unit
- **inputs**: `{"sender": "Address", "impersonated": "HashSet<Address>"}`
- **oracle**: Transaction allowed iff sender impersonated or credentials valid
- **severity**: spec
- **source**: plan Unit 2, hardhat_impersonateAccount behavior

<!-- INV-033: Watch list contagion source tracking -->
- **type**: state_machine
- **module**: `mirage_rs::fork::WatchEntry`
- **property**: Each watched contract has source: AutoClassified | Contagion { parent } | Manual
- **formula**: `watch_entry.source ∈ { AutoClassified, Contagion(parent), Manual }`
- **constraint**: Contagion parent must be valid address
- **test_fn**: `test_watch_entry_source_tracking`
- **strategy**: unit
- **inputs**: `{"source": "WatchSource"}`
- **oracle**: All three variants valid; contagion chain traceable
- **severity**: spec
- **source**: plan Unit 1, WatchSource enum

<!-- INV-034: Scenario wallet tracking -->
- **type**: numeric_range
- **module**: `mirage_rs::scenario::ScenarioResult`
- **property**: Final balances map must include all tracked addresses
- **formula**: `final_balances.keys() ⊇ track_addresses`
- **constraint**: No missing tracked addresses
- **test_fn**: `test_scenario_tracks_all_addresses`
- **strategy**: unit
- **inputs**: `{"track_addresses": "Vec<Address>", "final_balances": "HashMap<Address, U256>"}`
- **oracle**: Every tracked address has an entry; may have zero balance
- **severity**: code
- **source**: plan Unit 4, Scenario track_addresses and ScenarioResult

<!-- INV-035: Event sequence on local transaction -->
- **type**: event_sequence
- **module**: `mirage_rs::fork`
- **property**: Local transaction execution must follow: Execute → DiffClassify → DirtyStoreApply → WatchListUpdate → TxReceipt
- **formula**: `Sequence: TxExec → DiffClassify → DirtyApply → WatchUpdate → Receipt`
- **constraint**: No skipped steps; order fixed
- **test_fn**: `test_local_tx_event_sequence`
- **strategy**: integration
- **inputs**: `{"tx": "TransactionRequest"}`
- **oracle**: Observe event stream; must see all steps in order
- **severity**: spec
- **source**: plan Unit 1, local transaction pipeline

<!-- INV-036: Speculative invalidation conditions -->
- **type**: event_sequence
- **module**: `mirage_rs::replay::SpeculativeExecutor`
- **property**: Speculative result must invalidate on: (1) tx included on-chain, (2) new block writes to read_set, (3) block number/timestamp deviation
- **formula**: `invalidate(spec) iff (tx_included) OR (block_writes ∩ read_set ≠ ∅) OR (block_num_changed OR timestamp_changed)`
- **constraint**: Any condition triggers invalidation
- **test_fn**: `test_speculative_invalidation_conditions`
- **strategy**: integration
- **inputs**: `{"condition": "InvalidationTrigger"}`
- **oracle**: Speculation removed from cache on any trigger
- **severity**: spec
- **source**: plan Unit 3, SpeculativeExecutor invalidation triggers

### Regression Anchors

`test_hybrid_db_tier_priority`
`test_cache_ttl_expiration`
`test_hybrid_db_partial_dirty_merge`
`test_diff_classifier_protocol_threshold`
`test_watch_list_capacity_enforced`
`test_contagion_depth_capped`
`test_dirty_store_snapshot_single_use`
`test_block_budget_timeout_enforced`
`test_targeted_follower_filter_throughput`
`test_speculative_executor_memory_per_tx`
`test_speculative_invalidation_on_block_write`
`test_block_stm_conflict_rate`
`test_scenario_status_valid_transitions`
`test_scenario_runner_revert_restores_baseline`
`test_scenario_timeout_enforced`
`test_scenario_gas_exceeded`
`test_eth_estimate_gas_buffer`
`test_mirage_config_default_timeout`
`test_mirage_config_default_retries`
`test_resource_pressure_bounds`
`test_resource_pressure_tier_transitions`
`test_cache_hit_rate_bounds`
`test_profile_memory_allocations`
`test_profile_watch_limits`
`test_profile_cache_capacities`
`test_bytecode_cache_capacity`
`test_memory_startup_gate`
`test_lru_eviction_at_throttle_tier`
`test_upstream_rps_limit`
`test_upstream_exponential_backoff`
`test_eip2718_type_parsing`
`test_account_impersonation_validity`
`test_watch_entry_source_tracking`
`test_scenario_tracks_all_addresses`
`test_local_tx_event_sequence`
`test_speculative_invalidation_conditions`

### Cross-Crate Contracts

| Upstream | Input Condition | Expected Behavior |
|----------|----------------|-------------------|
| `golem-core::GolemConfig` | Valid TOML config with `[mirage]` section | `MirageConfig::from_golem_config()` parses url, port, profile |
| `golem-core::EventFabric` | `resource_pressure` value computed | mirage-rs emits to broadcast bus without blocking |
| `revm::Database` | Implements 4 trait methods (basic, storage, code_by_hash, block_hash) | HybridDB passes all requests through 3-tier lookup |
| `alloy::providers::Provider` | HTTP/WS connection established | UpstreamRpc respects rate limits and retries |
| `jsonrpsee::RpcServer` | JSON-RPC 2.0 request parsed | Server routes to handler; unknown methods return -32601 |

### Event Sequence Assertions

1. **Local Transaction Path:**
   - `eth_sendTransaction` called
   - `EvmExecutor::transact()` executes
   - `DiffClassifier::classify()` runs on StateDiff
   - `DirtyStore::apply_diff()` commits changes
   - Watch list updated with new Protocol contracts
   - `eth_getTransactionReceipt` returns result

2. **Scenario Sequential Execution:**
   - `mirage_beginScenarioSet()` captures baseline snapshot
   - For each scenario: `mirage_defineScenario()` adds to set
   - `mirage_runScenarioSet(Sequential)` begins
   - Each scenario: execute txs → collect results → `evm_revert(baseline_id)` → restore state → next scenario
   - Results returned with final_balances, gas_used, wall_time_ms, status

3. **Resource Pressure Escalation:**
   - Memory usage increases → `resource_pressure` increases
   - At pressure >= 0.5: warning signal emitted, logging enabled
   - At pressure >= 0.7: ReadCache LRU eviction starts, new contracts demoted to SlotOnly
   - At pressure >= 0.9: TargetedFollower stops, mode switches to Proxy, CorticalState signal emitted

4. **TargetedFollower Block Processing:**
   - WebSocket newHeads received
   - Block header enqueued to bounded channel (capacity 32)
   - Processing loop: fetch tx list → filter by watch_list → replay → DiffClassify → apply → log
   - If lagging >50 blocks: skip to latest without intermediate replay
   - If block replay exceeds budget: skip to next and log warning

### Academic References Verified

No academic references (Gompertz, Ebbinghaus, PAD, Plutchik, Kelly criterion, etc.) apply to mirage-rs. This is an engineering/systems specification for EVM transaction replay and state management, not a behavioral/mortality model. All numeric constants and formulas are implementation parameters, not published research constants.


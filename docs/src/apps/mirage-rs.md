# mirage-rs

## What It Is

mirage-rs is Bardo's local Ethereum fork sidecar and embeddable Rust library. It keeps local mutations in-process, lazily reads untouched state from an upstream RPC, exposes an Anvil- and Hardhat-compatible JSON-RPC surface, and adds Bardo-specific helpers for watch lists, scenarios, position inspection, and resource management.

## Features

- Three-tier fork state: dirty overrides, TTL read cache, and upstream lazy reads
- Synthetic local chain state with receipts, blocks, timestamps, snapshots, and revert support
- Standard `eth_*` JSON-RPC methods for balances, storage, code, calls, transactions, receipts, and synthetic blocks
- Hardhat, Anvil, and `evm_*` compatibility helpers for impersonation, direct state writes, mining, and time control
- `mirage_*` extensions for watch lists, token minting, position views, resource inspection, event subscriptions, and scenario jobs
- Copy-on-write helpers and speculative execution for isolated planning paths
- Targeted follower support that replays upstream transactions touching watched contracts
- Resource-pressure handling with cache eviction, slot-only demotion, and proxy-mode fallback
- Async client and spawned-process helpers for tests and other Bardo components

## Getting Started

Start `mirage-rs` as a standalone sidecar:

```bash
cargo run -p mirage-rs -- \
  --port 8545 \
  --rpc-url https://your-mainnet-rpc.example \
  --ws-url wss://your-mainnet-ws.example
```

Port `8545` matches Bardo's canonical local EVM slot from the shared port allocation reference. For test harnesses that should not collide with a development instance, use `18545`:

```bash
cargo run -p mirage-rs -- --port 18545
```

Connect from Rust code through `MirageClient`:

```rust
use std::time::Duration;

use mirage_rs::{MirageClient, MirageConfig};

let client = MirageClient::new(MirageConfig::default_local()).await?;
client.wait_ready(Duration::from_secs(10)).await?;
```

The binary writes `/tmp/mirage-<port>.pid` and `/tmp/mirage-<port>-status.json` at startup, exposes `/health`, and shuts down on `mirage_shutdown`, watchdog timeout, or process signal.

## Configuration

### CLI Flags

| Flag | Description | Default |
|---|---|---|
| `--host` | Bind host | `127.0.0.1` |
| `--port` | JSON-RPC and health/event port | `8545` |
| `--rpc-url` | Upstream HTTP RPC endpoint | unset |
| `--ws-url` | Upstream WebSocket endpoint for follower mode | unset |
| `--upstream-rps` | Upstream requests per second budget | `100` |
| `--upstream-burst` | Upstream burst allowance | `200` |
| `--chain-id` | Chain ID reported by the fork | `1` |
| `--cache-size` | Read-cache capacity | `10000` |
| `--cache-ttl-secs` | Read-cache TTL in seconds | `12` |
| `--profile` | Resource profile (`micro`, `standard`, `power`) | `standard` |
| `--watchdog-timeout` | Exit after this many idle seconds | unset |
| `--strict-nonce` | Reject low or high nonces | `false` |
| `--strict-balance` | Reject transactions that exceed balance | `false` |
| `--verify-signatures` | Validate raw transaction signatures | `false` |

### `golem.toml`

`MirageConfig::from_golem_config` reads the `mirage` section:

```toml
[mirage]
host = "127.0.0.1"
port = 8545
timeout_ms = 30000
retry_attempts = 3
retry_backoff_ms = 500
```

You can also provide a fully qualified URL:

```toml
[mirage]
url = "http://127.0.0.1:18545"
```

Environment overrides are available through the `BARDO_MIRAGE_*` prefix:

- `BARDO_MIRAGE_URL`
- `BARDO_MIRAGE_HOST`
- `BARDO_MIRAGE_PORT`
- `BARDO_MIRAGE_TIMEOUT_MS`
- `BARDO_MIRAGE_RETRY_ATTEMPTS`
- `BARDO_MIRAGE_RETRY_BACKOFF_MS`

## Module Overview

- `fork`: `ReadCache`, `DirtyStore`, `HybridDB`, `ForkState`, `MirageFork`, `MirageStatus`, and the local execution entrypoints
- `provider`: `UpstreamRpc` and `BlockTag` for upstream reads, block fetches, transaction fetches, mock mode, retries, and rate limiting
- `cow`: `CowState`, `BytecodeCache`, `MultiVersionStore`, and `VersionEntry`
- `replay`: `StateDiff`, `AccountDiff`, `LogEntry`, `TargetedFollower`, `TxReplay`, and `SpeculativeExecutor`
- `scenario`: `Scenario`, `ScenarioAssertions`, `ScenarioSet`, `ScenarioRunner`, `ScenarioResult`, `ScenarioJob`, and ranking helpers
- `resources`: `ResourceModel`, `ResourceUsage`, `Profile`, `MirageMode`, and pressure-tier evaluation
- `integration`: `MirageConfig`, `MirageClient`, `MirageTestInstance`, position helpers, and event-subscription helpers
- `rpc`: JSON-RPC registration, health handler, and `/events/{stream_id}` WebSocket delivery

## API

### Core Library Types

```rust
pub type Result<T> = std::result::Result<T, MirageError>;

pub struct TransactionRequest {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas: Option<u64>,
    pub value: Option<U256>,
    pub data: Option<Bytes>,
    pub gas_price: Option<u128>,
    pub nonce: Option<u64>,
    pub chain_id: Option<u64>,
}

pub struct Bytecode(Bytes);
impl Bytecode {
    pub fn new_raw(bytes: Bytes) -> Self;
    pub fn hash_slow(&self) -> B256;
    pub fn bytecode(&self) -> &Bytes;
}

pub struct AccountInfo {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: B256,
    pub code: Option<Bytecode>,
}

pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub output: Bytes,
}
```

### Fork and Replay Surface

```rust
pub struct HybridDB;
impl HybridDB {
    pub fn new(
        upstream: Arc<UpstreamRpc>,
        cache_capacity: usize,
        cache_ttl: Duration,
        bytecode_capacity: NonZeroUsize,
        chain_id: u64,
    ) -> Self;
    pub fn resolve_block(&self) -> BlockTag;
    pub fn set_balance(&mut self, address: Address, balance: U256);
    pub fn set_nonce(&mut self, address: Address, nonce: u64);
    pub fn set_code(&mut self, address: Address, code: Bytecode);
    pub fn set_storage(&mut self, address: Address, slot: U256, value: U256);
    pub fn reset(&mut self);
    pub fn erc20_balance_of(&mut self, token: Address, owner: Address) -> Result<U256>;
    pub fn set_erc20_balance(
        &mut self,
        token: Address,
        owner: Address,
        balance: U256,
    ) -> Result<U256>;
}

pub struct ForkState;
impl ForkState {
    pub fn new(db: HybridDB, local_block_number: u64, chain_id: u64) -> Self;
    pub fn snapshot(&mut self) -> u64;
    pub fn revert(&mut self, id: u64) -> Result<bool>;
    pub fn status(&self, mode: MirageMode) -> MirageStatus;
    pub fn resource_usage(&self, model: &ResourceModel, mode: MirageMode) -> ResourceUsage;
}

pub struct MirageFork;
impl MirageFork {
    pub fn new(fork: ForkState, resource_model: ResourceModel, mode: MirageMode) -> Self;
    pub fn idle_for(&self) -> Duration;
}

pub struct DirtyAccount {
    pub balance: Option<U256>,
    pub nonce: Option<u64>,
    pub code: Option<Bytecode>,
    pub code_hash: Option<B256>,
    pub storage: HashMap<U256, U256>,
}

pub enum WatchSource {
    AutoClassified,
    Contagion { parent: Address },
    Manual,
}

pub struct WatchEntry {
    pub source: WatchSource,
    pub added_at_block: u64,
    pub initial_slot_count: usize,
    pub replay_count: u64,
}

pub struct DirtyStore {
    pub accounts: HashMap<Address, DirtyAccount>,
    pub watch_list: HashMap<Address, WatchEntry>,
    pub unwatch_list: HashSet<Address>,
    pub total_dirty_slots: u64,
}
impl DirtyStore {
    pub fn snapshot(&mut self, block_number: u64, tx_index: u64) -> u64;
    pub fn revert(&mut self, id: u64) -> Result<(u64, u64)>;
    pub fn clear(&mut self);
    pub fn apply_state_diff(&mut self, diff: &StateDiff);
}

pub struct TxReplay {
    pub tx_hash: B256,
}
impl TxReplay {
    pub fn execute(
        &self,
        upstream: &UpstreamRpc,
        state: &mut ForkState,
    ) -> Result<(ExecutionResult, StateDiff)>;
}

pub struct SpeculativeExecutor;
impl SpeculativeExecutor {
    pub fn execute(
        &mut self,
        state: &ForkState,
        request: &TransactionRequest,
    ) -> Result<SpeculativeResult>;
    pub fn invalidate_for_writes(&mut self, writes: &HashSet<(Address, U256)>);
    pub fn invalidate_for_block(&mut self, block_number: u64);
}

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
    pub storage_written: HashMap<U256, U256>,
    pub storage_read: HashSet<U256>,
}

pub struct CowState;
impl CowState {
    pub fn branch(baseline: Arc<HashMap<(Address, U256), U256>>) -> Self;
    pub fn read(&self, address: Address, slot: U256) -> Option<U256>;
    pub fn write(&mut self, address: Address, slot: U256, value: U256);
    pub fn overlay_size(&self) -> usize;
}
```

### Scenarios, Resources, and Integration

```rust
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub transactions: Vec<TransactionRequest>,
    pub track_addresses: Vec<Address>,
    pub max_gas: Option<u64>,
    pub timeout: Duration,
    pub assertions: ScenarioAssertions,
}
impl Scenario {
    pub fn from_toml(id: impl Into<String>, input: &str) -> Result<Self>;
    pub fn evaluate_assertions(&self, state: &mut ForkState) -> Result<()>;
}

pub struct ScenarioRunner;
impl ScenarioRunner {
    pub async fn run_sequential(&self, set: &ScenarioSet) -> Vec<ScenarioResult>;
    pub async fn run_parallel(&self, set: &ScenarioSet) -> Vec<ScenarioResult>;
}

pub struct ResourceModel {
    pub profile: Profile,
    pub max_memory_bytes: u64,
    pub max_watched_contracts: usize,
    pub cache_capacity: usize,
    pub cache_ttl: Duration,
}
impl ResourceModel {
    pub fn for_profile(profile: Profile, cache_ttl: Duration) -> Self;
    pub fn ensure_spawn_budget(&self) -> Result<()>;
    pub fn current_process_memory_bytes() -> u64;
}

pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub resource_pressure: f64,
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
impl ResourceUsage {
    pub fn is_warning(&self) -> bool;
    pub fn is_throttled(&self) -> bool;
    pub fn is_emergency(&self) -> bool;
}

pub struct MirageConfig {
    pub url: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub retry_backoff: Duration,
}
impl MirageConfig {
    pub fn from_golem_config(config: &GolemConfig) -> Self;
    pub fn default_local() -> Self;
}

pub struct PositionRequest {
    pub owner: Address,
    pub protocol_type: String,
    pub contract: Option<Address>,
    pub token_addresses: Vec<Address>,
}

pub struct PositionSnapshot {
    pub owner: Address,
    pub protocol_type: String,
    pub data: serde_json::Value,
}

pub struct EventFilter {
    pub addresses: Option<Vec<Address>>,
    pub topics: Option<Vec<B256>>,
}

pub enum EventSource {
    LocalTx,
    FollowerReplay,
}

pub struct MirageEvent {
    pub block_number: u64,
    pub tx_hash: B256,
    pub log_index: u32,
    pub contract: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    pub source: EventSource,
    pub decoded: Option<serde_json::Value>,
}

pub struct MirageClient;
impl MirageClient {
    pub async fn new(config: MirageConfig) -> Result<Self>;
    pub async fn eth_call(&self, req: TransactionRequest) -> Result<Bytes>;
    pub async fn eth_send_transaction(&self, req: TransactionRequest) -> Result<B256>;
    pub async fn evm_snapshot(&self) -> Result<u64>;
    pub async fn evm_revert(&self, id: u64) -> Result<bool>;
    pub async fn mirage_watch_contract(&self, addr: Address) -> Result<()>;
    pub async fn mirage_get_position(&self, req: PositionRequest) -> Result<PositionSnapshot>;
    pub async fn mirage_status(&self) -> Result<MirageStatus>;
    pub async fn mirage_get_resource_usage(&self) -> Result<ResourceUsage>;
    pub async fn mirage_begin_scenario_set(&self, baseline: &str) -> Result<String>;
    pub async fn mirage_define_scenario(&self, set_id: &str, scenario: &Scenario) -> Result<String>;
    pub async fn mirage_run_scenario_set(&self, set_id: &str, mode: RunMode) -> Result<String>;
    pub async fn mirage_get_scenario_results(&self, job_id: &str) -> Result<ScenarioJob>;
    pub async fn wait_ready(&self, timeout: Duration) -> Result<()>;
    pub async fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> Result<BoxStream<'static, MirageEvent>>;
    pub async fn shutdown(&self) -> Result<bool>;
}

pub struct ScenarioResult {
    pub scenario_id: String,
    pub name: String,
    pub status: ScenarioStatus,
    pub gas_used: u64,
    pub wall_time_ms: u64,
    pub peak_memory_bytes: u64,
    pub pnl_wei: i128,
    pub state_diff_accounts: usize,
    pub state_diff_storage_slots: usize,
    pub final_balances: HashMap<Address, U256>,
    pub position_state: serde_json::Value,
    pub logs: Vec<LogEntry>,
    pub revert_reason: Option<String>,
}

pub struct ScenarioJob {
    pub job_id: String,
    pub set_id: String,
    pub status: JobStatus,
    pub results: Option<Vec<ScenarioResult>>,
    pub total_wall_time_ms: Option<u64>,
}

pub struct MirageTestInstance;
impl MirageTestInstance {
    pub fn config(&self) -> MirageConfig;
    pub async fn shutdown(&mut self) -> Result<()>;
}

pub async fn spawn_mirage_test_instance(
    rpc_url: Option<&str>,
    port: Option<u16>,
) -> Result<MirageTestInstance>;
```

### JSON-RPC Surface

Standard Ethereum methods:

- `web3_clientVersion`
- `net_version`
- `eth_chainId`
- `eth_blockNumber`
- `eth_gasPrice`
- `eth_maxPriorityFeePerGas`
- `eth_feeHistory`
- `eth_getBalance`
- `eth_getTransactionCount`
- `eth_getStorageAt`
- `eth_getCode`
- `eth_call`
- `eth_estimateGas`
- `eth_sendTransaction`
- `eth_sendRawTransaction`
- `eth_getTransactionReceipt`
- `eth_getTransactionByHash`
- `eth_getLogs`
- `eth_getBlockByNumber`
- `eth_getBlockByHash`

Compatibility and test helpers:

- `hardhat_impersonateAccount` and `anvil_impersonateAccount`
- `hardhat_stopImpersonatingAccount` and `anvil_stopImpersonatingAccount`
- `hardhat_setBalance`, `anvil_setBalance`, `mirage_setBalance`
- `hardhat_setStorageAt`, `anvil_setStorageAt`, `mirage_setStorageAt`
- `hardhat_setCode`, `anvil_setCode`, `mirage_setCode`
- `anvil_setNonce`
- `hardhat_mine`, `anvil_mine`, `evm_mine`
- `hardhat_reset`, `anvil_reset`
- `hardhat_setNextBlockBaseFeePerGas`, `anvil_setNextBlockBaseFeePerGas`
- `hardhat_setCoinbase`, `anvil_setCoinbase`
- `anvil_setPrevRandao`
- `evm_snapshot`
- `evm_revert`
- `evm_increaseTime`
- `evm_setNextBlockTimestamp`

mirage extensions:

- `mirage_mintERC20`
- `mirage_prefetchAccount`
- `mirage_prefetchSlots`
- `mirage_watchContract`
- `mirage_unwatchContract`
- `mirage_getWatchList`
- `mirage_getDirtySlots`
- `mirage_status`
- `mirage_getResourceUsage`
- `mirage_setResourceLimits`
- `mirage_getPosition`
- `mirage_subscribeEvents`
- `mirage_beginScenarioSet`
- `mirage_defineScenario`
- `mirage_runScenarioSet`
- `mirage_getScenarioResults`
- `mirage_compareScenarios`
- `mirage_computeDomainSeparator`
- `mirage_cleanup`
- `mirage_shutdown`

Current compatibility notes:

- `eth_getLogs` currently returns an empty array.
- `mirage_setResourceLimits` switches runtime profile presets instead of accepting arbitrary numeric caps.
- `mirage_subscribeEvents` registers a stream over JSON-RPC and delivers events over WebSocket at `/events/{stream_id}`.
- Event publication currently comes from locally committed transaction receipts.
- `mirage_cleanup` is presently a compatibility stub that returns `true`.

## Usage Examples

### Local Transfer With Snapshot and Revert

```rust
use std::time::Duration;

use alloy_primitives::U256;
use mirage_rs::{MirageClient, MirageConfig, TransactionRequest};

let client = MirageClient::new(MirageConfig::default_local()).await?;
client.wait_ready(Duration::from_secs(10)).await?;

let snapshot = client.evm_snapshot().await?;
let tx_hash = client
    .eth_send_transaction(TransactionRequest {
        from: Some("0x1000000000000000000000000000000000000001".parse()?),
        to: Some("0x1000000000000000000000000000000000000002".parse()?),
        gas: Some(21_000),
        value: Some(U256::from(25_u64)),
        data: Some(Default::default()),
        gas_price: None,
        nonce: None,
        chain_id: Some(1),
    })
    .await?;

let _ = tx_hash;
client.evm_revert(snapshot).await?;
```

### Scenario Set Execution

```rust
use std::time::Duration;

use alloy_primitives::U256;
use mirage_rs::{
    JobStatus, MirageClient, MirageConfig, RunMode, Scenario, ScenarioAssertions,
    TransactionRequest,
};

let client = MirageClient::new(MirageConfig::default_local()).await?;
let set_id = client.mirage_begin_scenario_set("latest").await?;

let scenario = Scenario {
    id: "transfer-branch".to_owned(),
    name: "transfer branch".to_owned(),
    transactions: vec![TransactionRequest {
        from: Some("0x3000000000000000000000000000000000000001".parse()?),
        to: Some("0x3000000000000000000000000000000000000002".parse()?),
        gas: Some(21_000),
        value: Some(U256::from(4_u64)),
        data: Some(Default::default()),
        gas_price: None,
        nonce: None,
        chain_id: Some(1),
    }],
    track_addresses: vec![
        "0x3000000000000000000000000000000000000001".parse()?,
        "0x3000000000000000000000000000000000000002".parse()?,
    ],
    max_gas: Some(30_000),
    timeout: Duration::from_secs(1),
    assertions: ScenarioAssertions::default(),
};

client.mirage_define_scenario(&set_id, &scenario).await?;
let job_id = client.mirage_run_scenario_set(&set_id, RunMode::Parallel).await?;

loop {
    let job = client.mirage_get_scenario_results(&job_id).await?;
    if matches!(job.status, JobStatus::Complete | JobStatus::Failed) {
        break;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### Spawned Test Sidecar

```rust
use std::time::Duration;

use mirage_rs::{MirageClient, spawn_mirage_test_instance};

let mut instance = spawn_mirage_test_instance(None, Some(18_545)).await?;
let client = MirageClient::new(instance.config()).await?;

client.wait_ready(Duration::from_secs(10)).await?;

// Run JSON-RPC calls through `client`.

instance.shutdown().await?;
```

## Architecture

mirage-rs is organized around four runtime layers.

The state layer is centered on `HybridDB`. Reads prefer `DirtyStore`, then `ReadCache`, then the upstream RPC. Dirty state is never evicted, while the read cache is trimmed under pressure and bytecode is cached separately by code hash.

The execution layer runs through `ForkState` and `EvmExecutor`. Local transactions update dirty balances, nonces, storage, receipts, synthetic blocks, and watch-list classification. The current executor implements simplified transfer, ERC-20, contract-touch, and contract-creation paths while keeping the public fork surface stable.

The replay and scenario layer builds on the same fork core. `TargetedFollower` replays only upstream transactions that match the watch list or configured filters. `SpeculativeExecutor` clones state for non-committing execution, and `ScenarioRunner` executes sequential or parallel branches from a baseline fork snapshot.

The server layer exposes JSON-RPC through the root endpoint, `/health` for readiness checks, and `/events/{stream_id}` for WebSocket event delivery. Resource sampling can evict cache entries, demote newly classified protocols to slot-only handling, or push the instance into proxy mode when pressure reaches the emergency tier.

## Specification References

- `prd2/15-dev/01-mirage-rs.md`: `Concept`, `Architecture Diagram`, `HybridDB`, `DirtyStore`
- `prd2/15-dev/01b-mirage-rpc.md`: `Standard eth_* Methods`, `Hardhat/Anvil Compatibility Methods`, `evm_* Methods`, `mirage_* Resource Management`, `mirage_* Position Helpers`, `mirage_* Scenario Runner`, `Error Code Reference`
- `prd2/15-dev/01c-mirage-scenarios.md`: `Targeted Follower`, classification rules, and scenario execution model
- `prd2/15-dev/01d-mirage-integration.md`: `Golem Sidecar Lifecycle`, `Resource Model`, `Core Golem Workflows`
- `prd2/shared/port-allocation.md`: `Port Map (Normative)`

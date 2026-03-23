# mirage-rs -- In-Process EVM Fork with Live State Tracking

**Version:** 2.0.0
**Last Updated:** 2026-03-18
**Crate:** `mirage-rs/`

---

> **Reader orientation:** This document specifies the core architecture of mirage-rs, the in-process EVM fork that replaces Anvil for DeFi development (section 15). The central concept is HybridDB, a three-tier database (DirtyStore for local mutations, ReadCache for hot reads, upstream RPC at `latest`) that lets a Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) local transactions coexist with live mainnet state. The document also covers automatic dirty tracking, Copy-on-Write state layers for cheap branching, and Block-STM parallel execution for historical replay. See `prd2/shared/glossary.md` for full term definitions.

## Concept

The standard approach to local EVM development is Anvil: fork mainnet at block N, get a frozen snapshot, run your transactions against it. This works for unit tests. It fails for anything that cares about what happens _after_ your transaction.

If you add liquidity to a Uniswap V3 pool and want to know whether you're in range an hour later, Anvil can't tell you. The pool's price doesn't move because no one else is trading. If you deploy a vault and want to see how its NAV tracks as the underlying assets fluctuate, Anvil gives you silence.

mirage-rs solves this by running a local revm instance that tracks live mainnet state and replays only the transactions that matter. Your local state diverges from mainnet at the point of your first local transaction, but mainnet data keeps flowing in. The result is a parallel universe: your positions exist in a market that keeps moving.

### v1 to v2: The Inversion

v1 replayed every mainnet block through a local revm instance. That meant ~150 transactions per block, every 12 seconds, racing against block production. The cost was enormous: block skipping, stale state for contracts that fell behind, oracle rot, and a permanent race against the chain head.

v2 inverts the model around three ideas:

**Lazy-latest reads.** Every storage slot the golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) hasn't modified reads directly from mainnet at `latest`. Oracle prices, lending rates, token supplies -- all live, zero local compute.

**Automatic dirty tracking.** When a golem transacts locally, revm's state diff classifies each touched contract as either a "protocol" (complex state, the golem's position lives here) or a "token" (balance transfer, slot-level override only). Protocols enter a watch list. Tokens get their specific slots overridden and nothing more.

**Targeted replay.** Only mainnet transactions touching watched protocol contracts get replayed locally. For a typical portfolio of 3-10 DeFi positions, this means ~5-15 transactions per block instead of ~150. Keeping up with the chain head becomes trivial.

The golem transacts normally; mirage figures out what to watch. LP fees accrue from real swaps, Aave positions respond to real liquidations, and Chainlink oracles stay current without any simulation.

---

## Architecture Diagram

```
+------------------------------------------------------------------+
|                          mirage-rs v2                             |
|                                                                  |
|  +------------------------------------------------------------+ |
|  |                     JSON-RPC Server                         | |
|  |  eth_*, mirage_*, evm_*, hardhat_*, anvil_* handlers        | |
|  +---------------+--------------------------+-----------------+ |
|                  |                          |                    |
|           write path                  read path                  |
|                  |                          |                    |
|  +---------------v--------------------------v-----------------+ |
|  |                     EvmExecutor                             | |
|  |   call() / transact() against HybridDB                     | |
|  +---------------+--------------------------+-----------------+ |
|                  |                          |                    |
|  +---------------v--------------+ +---------v-----------------+ |
|  |       DirtyStore             | |      ReadThrough          | |
|  |  +------------------------+  | |  +---------------------+  | |
|  |  | watched_contracts      |  | |  | BytecodeCache (LRU) |  | |
|  |  | (protocol state)       |  | |  | keyed by code_hash  |  | |
|  |  +------------------------+  | |  +---------------------+  | |
|  |  | dirty_slots            |  | |  | ReadCache (LRU+TTL) |  | |
|  |  | (token balances)       |  | |  | (hot read path)     |  | |
|  |  +------------------------+  | |  +---------------------+  | |
|  |  | CoW state layers       |  | |  | Upstream RPC        |  | |
|  |  | (scenario branching)   |  | |  | (fetch at "latest") |  | |
|  |  +------------------------+  | |  +---------------------+  | |
|  +------------------------------+ +---------------------------+ |
|                                                                  |
|  +------------------------------------------------------------+ |
|  |                  TargetedFollower                            | |
|  |  Subscribe to new blocks via WebSocket                      | |
|  |  Scan each block for txs touching watched contracts         | |
|  |  Replay matched txs through EvmExecutor                     | |
|  |  Update DirtyStore + extend watch list via contagion        | |
|  +------------------------------------------------------------+ |
|                                                                  |
|  +------------------------------------------------------------+ |
|  |              SpeculativeExecutor (enhancement)              | |
|  |  Subscribe to pending txs from mempool                      | |
|  |  Pre-execute against CoW fork of current state              | |
|  |  Cache results keyed by (tx_hash, base_state_block)         | |
|  |  Invalidate on state overlap with new blocks                | |
|  +------------------------------------------------------------+ |
+------------------------------------------------------------------+
```

---

## HybridDB

The central database replaces v1's `CacheDB<RemoteDB>` with a three-tier read priority: DirtyStore, then ReadCache, then upstream RPC at `latest`. All local mutations write to DirtyStore. All clean reads flow through the cache with TTL-based expiry.

```rust
use alloy_primitives::{Address, B256, U256};
use revm::primitives::{AccountInfo, Bytecode};
use revm::database::Database;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The main database backing all EVM execution in mirage-rs v2.
///
/// Read priority: DirtyStore -> ReadCache -> Upstream RPC (at "latest")
/// Write destination: DirtyStore (all local mutations land here)
pub struct HybridDB {
    /// All locally-modified state. Never evicted except by evm_revert.
    dirty: DirtyStore,

    /// LRU cache for upstream reads. Entries expire after cache_ttl.
    read_cache: ReadCache,

    /// LRU cache for compiled bytecode, keyed by code hash.
    /// Bytecode is immutable post-deployment -- this cache never invalidates.
    bytecode_cache: BytecodeCache,

    /// Upstream RPC client for fetching live mainnet state.
    upstream: UpstreamRpc,

    /// When set, all upstream reads within a single EVM execution
    /// use this block number instead of "latest". Ensures consistency
    /// within a transaction/call. Set before execution, cleared after.
    pinned_block: Option<u64>,

    /// Cache TTL. Default: 12 seconds (one Ethereum L1 block).
    cache_ttl: Duration,

    /// Chain ID for EVM configuration.
    chain_id: u64,
}
```

### Database Trait Implementation

HybridDB implements revm's `Database` trait, the four-method interface revm calls for all state access during EVM execution: `basic()`, `code_by_hash()`, `storage()`, and `block_hash()`.

```rust
impl Database for HybridDB {
    type Error = MirageError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // 1. Check dirty store for overridden fields
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

        // 2. Check read cache
        if let Some(info) = self.read_cache.get_account(&address) {
            return Ok(Some(info.clone()));
        }

        // 3. Fetch from upstream at pinned block or latest
        let info = self.fetch_account_info(address)?;
        if let Some(ref info) = info {
            let block = self.resolve_block();
            self.read_cache.insert_account(address, info.clone(), block);
        }
        Ok(info)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        // Check bytecode cache first (immutable, never invalidates)
        if let Some(bytecode) = self.bytecode_cache.get(&code_hash) {
            return Ok(bytecode.clone());
        }

        // Check dirty store (locally injected code)
        for dirty in self.dirty.accounts.values() {
            if dirty.code_hash == Some(code_hash) {
                if let Some(ref code) = dirty.code {
                    self.bytecode_cache.insert(code_hash, code.clone());
                    return Ok(code.clone());
                }
            }
        }

        // Fetch from upstream and cache permanently
        let bytecode = self.upstream.get_code_by_hash(
            code_hash, self.resolve_block()
        )?;
        self.bytecode_cache.insert(code_hash, bytecode.clone());
        Ok(bytecode)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // 1. Check dirty store for this slot
        if let Some(dirty) = self.dirty.accounts.get(&address) {
            if let Some(value) = dirty.storage.get(&index) {
                return Ok(*value);
            }
        }

        // 2. Check read cache
        if let Some(value) = self.read_cache.get_storage(&address, &index) {
            return Ok(value);
        }

        // 3. Fetch from upstream
        let block = self.resolve_block();
        let value = self.upstream.get_storage_at(address, index, block)?;
        self.read_cache.insert_storage(address, index, value, block);
        Ok(value)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        if let Some(hash) = self.read_cache.block_hashes.get(&number) {
            return Ok(*hash);
        }
        let hash = self.upstream.get_block_hash(number)?;
        self.read_cache.block_hashes.put(number, hash);
        Ok(hash)
    }
}
```

The `resolve_block()` method returns `BlockTag::Number(from_block)` in historical mode (where all reads pin to the fork block) and `BlockTag::Latest` in live mode. This single branch is the mechanism that makes historical replay work correctly -- no upstream read ever sees state newer than the fork point.

---

## DirtyStore

The DirtyStore holds all locally-modified state and the watch list. It is the successor to v1's `UserOverlay`, promoted from an optional add-on to the primary write layer.

```rust
pub struct DirtyStore {
    /// Per-account dirty state. Keyed by contract/EOA address.
    accounts: HashMap<Address, DirtyAccount>,

    /// Contracts whose mainnet activity should be replayed locally.
    /// Built automatically from transaction state diffs.
    watch_list: HashMap<Address, WatchEntry>,

    /// Contracts explicitly excluded from the watch list.
    /// Prevents auto-classification from re-adding them.
    unwatch_list: HashSet<Address>,

    /// Running counter of total dirty slots, for status reporting
    /// and memory pressure monitoring.
    total_dirty_slots: u64,

    /// Snapshot storage for evm_snapshot / evm_revert.
    snapshots: HashMap<u64, Box<DirtyStoreSnapshot>>,
    next_snapshot_id: u64,
}

pub struct DirtyAccount {
    /// Overridden account fields. Some = locally modified, None = read-through.
    balance: Option<U256>,
    nonce: Option<u64>,
    code: Option<Bytecode>,
    code_hash: Option<B256>,

    /// Per-slot storage overrides. Only these slots are detached from mainnet.
    /// All other slots read through to upstream at latest.
    storage: HashMap<U256, U256>,
}

pub struct WatchEntry {
    source: WatchSource,
    added_at_block: u64,
    initial_slot_count: usize,
    replay_count: u64,
}

pub enum WatchSource {
    /// Local transaction wrote 3+ slots to this contract.
    AutoClassified,
    /// A replayed transaction for a watched contract triggered writes
    /// to this contract (recursive contagion).
    Contagion { parent: Address },
    /// Added explicitly via mirage_watchContract.
    Manual,
}
```

---

## State Diff Classification

When a local transaction executes, revm produces a state diff: every account and storage slot that was read or written. The `DiffClassifier` examines this diff to decide what enters the watch list.

The classification rule is simple. A contract with 3 or more storage slots written by a single transaction is a protocol (complex state, the golem's position lives here). A contract with 1-2 slots written is a token (balance transfer, slot-level override only). The threshold is configurable but the default of 3 works because ERC-20 transfers write at most 2 slots (sender balance + recipient balance), while protocol interactions (LP deposit, borrow, stake) write 3 or more (position state, global accumulators, tick bitmaps, etc.).

A secondary heuristic handles edge cases: if all written slots are high-entropy keccak outputs (typical of mapping entries) with no low-numbered slots (0-20, typical of contract-level state variables), the contract is likely a token even at 3+ writes. Rebasing tokens like stETH that update totalSupply, rebase index, and user balance in one transfer trigger this path.

```rust
pub struct ClassificationConfig {
    /// Minimum slots written to classify as protocol. Default: 3.
    pub protocol_slot_threshold: usize,
    /// Check for ERC-20 storage patterns as a secondary signal.
    pub check_token_interface: bool,
    /// Maximum watched contracts. Safety valve.
    pub max_watched_contracts: usize,
    /// Enable recursive contagion (watched contracts that interact
    /// with new contracts add those to the watch list).
    pub enable_contagion: bool,
    /// Maximum contagion depth. Default: 2.
    pub max_contagion_depth: usize,
}

pub enum Classification {
    Protocol,  // Add to watch list, replay mainnet txs
    SlotOnly,  // Override specific slots, no replay
    ReadOnly,  // No storage written, no action
}
```

### Recursive Contagion

When TargetedFollower replays a mainnet transaction that touches a watched contract, the replay itself may write to new contracts. If those new contracts exceed the slot threshold, they are added to the watch list with `WatchSource::Contagion { parent }`. This captures the composability chain: a golem deposits into a Yearn vault, which calls Aave, which reads from Chainlink. The vault enters the watch list from the golem's transaction; Aave enters via contagion from the vault's replayed interactions.

Contagion depth is capped at `max_contagion_depth` (default: 2) to prevent the watch list from spiraling through the entire DeFi dependency graph.

---

## Bytecode Cache

Contract bytecode is immutable after deployment. Every `eth_getCode` call downloads bytes that never change, yet v1 re-fetched them on every fork. The bytecode cache eliminates this overhead entirely.

```rust
use lru::LruCache;

/// LRU cache for compiled revm bytecode, keyed by code hash.
/// Entries never expire -- bytecode is immutable post-deployment.
/// Shared across all forks via Arc.
pub struct BytecodeCache {
    cache: LruCache<B256, Bytecode>,
}

impl BytecodeCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ),
        }
    }

    pub fn get(&mut self, code_hash: &B256) -> Option<&Bytecode> {
        self.cache.get(code_hash)
    }

    pub fn insert(&mut self, code_hash: B256, bytecode: Bytecode) {
        self.cache.put(code_hash, bytecode);
    }
}
```

For a golem watching ~100 protocols with ~500 unique contracts, the bytecode cache eliminates ~500 RPC calls on the first fork and all calls on subsequent forks. The cache is shared across all mirage instances via `Arc`, so scenario runner child processes and Sanctum F6 forks benefit from the same warmed bytecode. The Foundry team's `foundry-fork-db` uses the same pattern (Urbanek, 2024).

---

## Copy-on-Write State Layers

v1's snapshot/revert mechanism clones the entire DirtyStore on each snapshot. For a baseline of ~50,000 dirty slots, that is ~3.2 MB per snapshot. With 8 parallel scenarios, baseline memory consumption reaches ~25 MB just for state copies.

CoW state layers eliminate this cost. A CoW overlay shares its parent state via `Arc` and stores only the slots that the current branch has modified. Reading checks the overlay first, then falls through to the shared parent. Writing goes to the overlay only.

```rust
use std::sync::Arc;

/// Copy-on-Write state overlay.
/// The baseline is shared (Arc, zero-copy for all scenarios).
/// Each branch stores only its own mutations.
pub struct CowState {
    /// Shared immutable baseline.
    baseline: Arc<HashMap<(Address, U256), U256>>,
    /// Per-branch dirty overlay (only modified slots).
    overlay: HashMap<(Address, U256), U256>,
}

impl CowState {
    /// Create a new branch from a shared baseline.
    pub fn branch(baseline: &Arc<HashMap<(Address, U256), U256>>) -> Self {
        Self {
            baseline: Arc::clone(baseline),
            overlay: HashMap::new(),
        }
    }

    pub fn read(&self, addr: Address, slot: U256) -> Option<U256> {
        self.overlay.get(&(addr, slot))
            .or_else(|| self.baseline.get(&(addr, slot)))
            .copied()
    }

    pub fn write(&mut self, addr: Address, slot: U256, value: U256) {
        self.overlay.insert((addr, slot), value);
    }

    /// Number of slots modified in this branch.
    pub fn overlay_size(&self) -> usize {
        self.overlay.len()
    }
}
```

For a scenario that modifies ~200 storage slots against a 50,000-slot baseline, memory drops from 3.2 MB per branch to ~12.8 KB. At 8 parallel scenarios: 25 MB becomes 100 KB. This matters for the Sanctum F6 fork workflow, where branching should be instant -- create an empty overlay, not clone the entire state.

---

## EVM Executor

```rust
pub struct EvmExecutor;

impl EvmExecutor {
    // Read-only: pins block, does not mutate state
    pub fn call(state: &ForkState, from, to, data, value, gas_limit)
        -> Result<ExecutionResult>;

    // Write: mutates state, triggers DiffClassifier
    pub fn transact(state: &mut ForkState, from, to, data, value, gas_limit, gas_price)
        -> Result<ExecutionResult>;

    // Detached: execute against a borrowed DB, return the modified DB
    pub fn transact_detached(db: ForkDB, tx: TxEnv, block_env, chain_id)
        -> (Result<ExecutionResult>, ForkDB);
}
```

Configuration: Cancun spec (EIP-1153 transient storage, required for V4 PoolManager), nonce/balance checks disabled by default (allows unsigned local transactions), no base fee enforcement by default.

### Transaction Execution (State Commit)

```rust
pub fn transact_env(state: &mut ForkState, tx: TxEnv, block_env: Option<BlockEnv>)
    -> Result<ExecutionResult>
{
    let block = block_env.unwrap_or_else(|| state.current_block_env());
    let cfg = Self::cfg_env(state.chain_id());

    // Swap DB out so Context can take ownership
    let placeholder = ForkDB::new(state.db().db.clone());
    let db = std::mem::replace(state.db_mut(), placeholder);

    let mut evm = Context::mainnet()
        .with_db(db)
        .with_block(block)
        .with_cfg(cfg)
        .build_mainnet();

    let result = evm.transact_commit(tx)?;

    // Put the DB back
    *state.db_mut() = evm.ctx.journaled_state.database;
    Ok(ExecutionResult::from_revm(result))
}
```

The `std::mem::replace` dance is necessary because revm's `Context` takes ownership of the database. After execution, the modified database is extracted from the context and placed back into ForkState.

### Local Transaction Pipeline

When a golem sends a transaction via `eth_sendTransaction`, the full flow is:

1. **Execute** the transaction through revm against HybridDB.
2. **Classify** the resulting state diff (protocol vs. token vs. read-only).
3. **Apply** account info changes and storage overrides to DirtyStore.
4. **Update** the watch list with newly classified protocol contracts.
5. **Generate** a transaction hash, store the receipt, and advance the local block counter.

```rust
pub async fn handle_local_transaction(
    state: &Arc<RwLock<MirageState>>,
    tx: TransactionRequest,
) -> Result<B256> {
    let mut state = state.write();

    let (result, diff) = EvmExecutor::transact(
        &mut state.db,
        tx.from,
        tx.to,
        tx.data.unwrap_or_default(),
        tx.value.unwrap_or_default(),
        tx.gas_limit.unwrap_or(30_000_000),
    )?;

    let classification = state.classifier.classify(
        &diff, &state.db.dirty, state.block_number,
    );

    for (address, account_diff) in &diff.accounts {
        state.db.dirty.apply_account_changes(*address, account_diff);
    }
    state.db.dirty.apply_classification(classification);

    let tx_hash = generate_tx_hash(&tx, state.block_number, state.tx_index);
    state.store_receipt(tx_hash, &result);
    state.advance_block();

    Ok(tx_hash)
}
```

---

## Speculative Pre-Execution

The golem can subscribe to pending mempool transactions and pre-execute them against a CoW fork of the current state before the next block confirms. This turns mirage from a reactive tool (simulate after the fact) into a predictive one (simulate before the block lands).

The design follows the pattern used in MEV extraction (Flashbots' `eth_callBundle`) but applied to the golem's own position awareness:

```rust
pub struct SpeculativeExecutor {
    /// Cache of speculative results.
    /// Key: (pending_tx_hash, base_state_block_number)
    /// Value: execution result + state diff
    cache: HashMap<(B256, u64), SpeculativeResult>,
}

pub struct SpeculativeResult {
    result: ExecutionResult,
    state_diff: StateDiff,
    /// Slots that were read during execution. Used for invalidation:
    /// if any new block transaction writes to an overlapping slot,
    /// this cached result is stale.
    read_set: HashSet<(Address, U256)>,
    computed_at: Instant,
}
```

Cache invalidation fires when:

1. The transaction is included on-chain (the speculation is resolved).
2. Any transaction in a new block modifies state that the cached simulation read (SLOAD/SSTORE overlap via state diffs).
3. The block number or timestamp deviate from the values assumed during speculation.

Pre-execution uses a CoW overlay against the current state, so the memory cost is minimal -- ~12 KB per pending transaction. The compute cost is a single revm `transact()` call, typically under 1ms for cached state.

---

## Block-STM Parallel Execution

For historical replay, mirage must re-execute entire blocks when tracked addresses have active transactions. Block-STM (Gelashvili et al., 2023) enables parallel transaction execution within a single block:

1. Execute all transactions in the block optimistically in parallel.
2. Track per-slot version numbers to detect read-write conflicts.
3. Re-execute conflicting transactions sequentially.
4. On typical DeFi blocks, the conflict rate is <5% (Saraph & Herlihy, 2019), so effective parallelism is high.

```rust
/// Per-slot version tracking for Block-STM conflict detection.
pub struct MultiVersionStore {
    /// For each (address, slot), track the latest version written
    /// and by which transaction index.
    versions: DashMap<(Address, U256), Vec<VersionEntry>>,
}

pub struct VersionEntry {
    tx_index: usize,
    value: U256,
    /// Incarnation number -- incremented on re-execution after conflict.
    incarnation: u32,
}

impl MultiVersionStore {
    /// Read a slot value as seen by transaction at tx_index.
    /// Returns the latest version written by a transaction with index < tx_index.
    pub fn read(&self, addr: Address, slot: U256, tx_index: usize) -> ReadResult {
        match self.versions.get(&(addr, slot)) {
            Some(entries) => {
                let prior = entries.iter()
                    .filter(|e| e.tx_index < tx_index)
                    .max_by_key(|e| e.tx_index);
                match prior {
                    Some(entry) => ReadResult::Found(entry.value, entry.tx_index),
                    None => ReadResult::NotFound,
                }
            }
            None => ReadResult::NotFound,
        }
    }

    /// Record a write from a specific transaction.
    pub fn write(
        &self,
        addr: Address,
        slot: U256,
        tx_index: usize,
        incarnation: u32,
        value: U256,
    ) {
        self.versions.entry((addr, slot))
            .or_default()
            .push(VersionEntry { tx_index, value, incarnation });
    }
}
```

Block-STM is most valuable for historical replay in `--replay-mode replay`, where full blocks of 150+ transactions must be re-executed. In live mode, TargetedFollower only replays 5-15 transactions per block -- too few to benefit from parallelism.

---

## Snapshot and Revert

Snapshots capture the DirtyStore. The ReadCache is not snapshotted -- it is a performance cache, and staleness is handled by TTL expiry. With CoW state layers, snapshots become cheap: the snapshot records the current overlay as a frozen baseline, and new mutations write to a fresh overlay.

```rust
pub struct DirtyStoreSnapshot {
    accounts: HashMap<Address, DirtyAccount>,
    watch_list: HashMap<Address, WatchEntry>,
    unwatch_list: HashSet<Address>,
    total_dirty_slots: u64,
    block_number: u64,
    tx_index: u64,
}

impl DirtyStore {
    pub fn snapshot(&mut self, block_number: u64, tx_index: u64) -> u64 {
        let id = self.next_snapshot_id;
        self.next_snapshot_id += 1;
        self.snapshots.insert(id, Box::new(DirtyStoreSnapshot {
            accounts: self.accounts.clone(),
            watch_list: self.watch_list.clone(),
            unwatch_list: self.unwatch_list.clone(),
            total_dirty_slots: self.total_dirty_slots,
            block_number,
            tx_index,
        }));
        id
    }

    pub fn revert(&mut self, id: u64) -> Result<(u64, u64)> {
        let snapshot = self.snapshots.remove(&id)
            .ok_or_else(|| MirageError::SnapshotNotFound(id))?;

        self.accounts = snapshot.accounts;
        self.watch_list = snapshot.watch_list;
        self.unwatch_list = snapshot.unwatch_list;
        self.total_dirty_slots = snapshot.total_dirty_slots;

        // Remove snapshots taken after this one
        let later: Vec<u64> = self.snapshots.keys()
            .filter(|&&k| k > id).copied().collect();
        for k in later { self.snapshots.remove(&k); }

        Ok((snapshot.block_number, snapshot.tx_index))
    }
}
```

Snapshots are consumed on revert -- calling `evm_revert` with the same ID twice fails. To use a snapshot multiple times (branching), take a new snapshot after each revert.

---

## Library Mode

mirage ships as both a standalone binary (the primary deployment for golem sidecars) and an optional Rust crate for embedding directly into a host process.

```rust
pub struct Mirage {
    state: Arc<RwLock<MirageState>>,
    follower: Option<TargetedFollower>,
}

impl Mirage {
    pub async fn new(config: MirageConfig) -> Result<Self>;
    pub async fn call(&self, req: CallRequest) -> Result<ExecutionResult>;
    pub async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxReceipt>;
    pub fn snapshot(&self) -> u64;
    pub fn revert(&self, snapshot_id: u64) -> Result<()>;
    pub fn watch_contract(&self, address: Address) -> Result<()>;
    pub fn resource_usage(&self) -> ResourceUsage;
    pub async fn get_position(&self, addrs: &[Address]) -> Result<PositionSnapshot>;
    pub fn event_stream(&self) -> impl Stream<Item = LocalEvent>;
}
```

The embedded API calls directly into the same state machine that backs the JSON-RPC server. No serialization overhead, no process spawn latency. Feature flags control what ships:

```toml
[features]
default = ["binary"]
binary  = ["tokio/full", "dep:axum", "dep:tower"]  # JSON-RPC server
library = []                                         # API-only, no server
sim-gas = []                                         # Optional gas price simulator
```

---

## Configuration

```rust
pub struct MirageConfig {
    pub chain_id: u64,
    pub fork_block: Option<u64>,         // None = latest (live mode)
    pub rpc_url: String,                 // Upstream provider
    pub port: u16,                       // Local JSON-RPC port (default: 8545)
    pub host: String,                    // Bind address (default: 127.0.0.1)
    pub follow: bool,                    // Enable TargetedFollower
    pub max_watched_contracts: usize,    // Watch list cap (default: 64)
    pub cache_ttl_secs: u64,             // ReadCache TTL (default: 12)
    pub cache_capacity: usize,           // ReadCache entries (default: 10_000)
    pub max_memory_bytes: u64,           // Memory cap (default: 512 MB)
    pub sim_gas: bool,                   // Enable gas price simulator
    pub profile: Profile,               // micro, standard, or power
    pub watchdog_timeout_secs: u64,      // Watchdog timer (default: 30)
    pub strict_nonce: bool,              // Enforce sequential nonces
    pub strict_balance: bool,            // Enforce sufficient balance
    pub verify_signatures: bool,         // Verify ECDSA signatures
    pub hardfork: Hardfork,              // EVM hardfork (default: Cancun)
    // Historical mode
    pub mode: Mode,                      // Live or Historical
    pub from_block: Option<u64>,         // Historical: pin reads here
    pub to_block: Option<u64>,           // Historical: replay up to here
    pub replay_mode: ReplayMode,         // replay, stateDiff, or hybrid
    pub checkpoint_every: Option<u64>,   // Checkpoint interval
    pub output_dir: Option<PathBuf>,     // PnL + events output
}

pub enum Mode { Live, Historical }
pub enum ReplayMode { Replay, StateDiff, Hybrid }
pub enum Profile { Micro, Standard, Power }
```

---

## Performance Expectations

**Startup.** Instant. Connect to upstream, start JSON-RPC server. No state to load, no blocks to catch up on.

**Steady state (no positions).** Zero overhead. TargetedFollower subscribes to blocks but finds no matches.

**Steady state (3-5 DeFi positions).** Watch list: 5-15 contracts. Per block: scan ~150 tx targets against the watch list (HashSet lookup, ~1us each), replay 0-5 matched txs. Block processing time: <100ms typical.

**Read path.** Hot reads (cached): <1us. Warm reads (cache expired): 20-100ms. Cold reads (first access): 50-200ms.

---

## Dependencies

| Crate | Version | Purpose |
| ----- | ------- | ------- |
| revm | v36 | EVM execution engine |
| revm-database | v12 | CacheDB, DatabaseRef trait |
| alloy | v1.7 | Ethereum types, RPC providers (HTTP + WS) |
| jsonrpsee | v0.26 | JSON-RPC server framework |
| tokio | latest | Async runtime (full feature set) |
| clap | v4.5 | CLI parsing with derive macros |
| tracing | v0.1 | Structured logging |
| tracing-subscriber | v0.3 | Log formatting (plain + JSON) |
| parking_lot | v0.12 | Fast RwLock for shared ForkState |
| reqwest | v0.12 | HTTP client for upstream RPC proxy |
| tower-http | v0.6 | CORS middleware |
| serde / serde_json | latest | Serialization |
| futures | v0.3 | Async streams for WS subscription |
| lru | v0.12 | LRU cache for ReadCache and BytecodeCache |
| dashmap | v6 | Concurrent map for Block-STM MultiVersionStore |

---

## Edge Cases

**Composability depth.** A Yearn vault calls Aave which calls Chainlink which calls... Contagion caps at `max_contagion_depth` (default: 2), preventing the watch list from spiraling through every contract on Ethereum.

**Watch list saturation.** When the watch list hits `max_watched_contracts` (default: 64), new contracts fall back to slot-only classification. The lazy-latest base layer still provides correct reads -- the golem just doesn't get causal replay for those contracts.

**Mainnet transaction reverts.** When a mainnet swap replays against modified pool state, it may revert because the golem's liquidity changed the price past the swap's slippage tolerance. This is correct behavior. The follower logs it and moves on.

**Proxy contracts.** Storage writes from delegatecall land at the proxy's address, not the implementation. The slot count heuristic works correctly because it counts slots at the address where storage was actually modified.

**Reorg handling.** If the upstream chain reorgs, TargetedFollower may have replayed transactions from the old block. The impact is usually negligible for 1-block reorgs. The read cache TTL ensures stale upstream data expires within one block interval.

---

## v1 Compatibility

The v1 API surface (`CacheDB<RemoteDB>`, `ForkState`, full block replay) is replaced by v2. The key structural differences:

| Dimension | v1 | v2 |
| --------- | -- | -- |
| State model | `CacheDB<RemoteDB>` two-layer | HybridDB three-tier (DirtyStore + ReadCache + upstream) |
| Read source | Fork block (pinned at fork time) | `latest` (live reads, lazy) |
| Block replay | Every transaction in every block | Only txs touching watched contracts |
| Watch list | Manual filter by address/selector | Automatic via DiffClassifier |
| Branching | Full clone on snapshot | CoW overlays (~12.8 KB per branch) |
| Prefetching | Trace-based bulk insert | Not needed (lazy-latest eliminates cold reads during sync) |
| Divergence detection | Compare local vs mainnet receipts | Narrowed to watched contracts only |
| Memory overhead | Unbounded (all state cached) | Capped by profile (256 MB to 2 GB) |

v1's full block replay is preserved in v2 as the `--replay-mode replay` option for historical mode. Live mode uses targeted replay exclusively.

---

## Cross-References

- RPC method reference: [01b-mirage-rpc.md](./01b-mirage-rpc.md) -- Full JSON-RPC method catalog: eth_*, mirage_*, evm_*, hardhat/anvil compatibility methods, scenario runner API, error codes
- Scenario runner, historical mode, targeted follower: [01c-mirage-scenarios.md](./01c-mirage-scenarios.md) -- Classification rules, targeted follower pipeline, historical replay modes (replay/stateDiff/hybrid), scenario runner with CoW branching, Latin Hypercube parameter sweeps
- Bardo TUI integration, golem workflows: [01d-mirage-integration.md](./01d-mirage-integration.md) -- F6 fork workflow, Fork Inspector overlay, golem sidecar lifecycle (spawn/warm/simulate/teardown), CorticalState pressure gating, resource profiles
- Transaction compatibility: [01e-mirage-tx-compatibility.md](./01e-mirage-tx-compatibility.md) -- Transaction formats (EIP-2718 types 0-3), signature verification (ECDSA/EIP-1271/EIP-712), gas edge cases, nonce semantics, state injection pitfalls, DeFi-specific concerns (Chainlink staleness, TWAP drift, Permit2)

## References

- Gelashvili, R. et al. (2023). "Block-STM: Scaling Blockchain Execution by Turning Ordering Curse to a Performance Blessing." arXiv (Aptos Labs). -- *Introduces optimistic parallel transaction execution with per-slot version tracking and conflict-driven re-execution; the algorithm behind mirage-rs's parallel historical replay.*
- Saraph, V. & Herlihy, M. (2019). "An Empirical Study of Speculative Concurrency in Ethereum Smart Contracts." arXiv:1901.01376. -- *Measures read-write conflict rates across real Ethereum blocks, finding <5% conflict on typical DeFi blocks; justifies the high effective parallelism of Block-STM in mirage-rs.*
- Urbanek, P. (2024). "How to Simulate MEV Arbitrage with REVM, Anvil and Alloy." Blog post. -- *Describes the bytecode cache pattern used by Foundry's fork-db, which mirage-rs adopts for its immutable bytecode cache keyed by code hash.*
- Yang, T. et al. (2021). "Forerunner: Constraint-based Speculative Transaction Execution for Ethereum." SOSP 2021. -- *Proposes constraint-based speculation for transaction pre-execution; informs mirage-rs's SpeculativeExecutor design for mempool pre-execution against CoW state forks.*
- Reddio team (2025). "Boosting Blockchain Throughput: Parallel EVM Execution with Asynchronous Storage." arXiv:2503.04595. -- *Demonstrates async storage I/O during parallel EVM execution; relevant to mirage-rs's async upstream RPC fetches during Block-STM replay.*

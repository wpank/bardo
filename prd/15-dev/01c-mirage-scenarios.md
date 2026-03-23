# mirage-rs Scenarios, Historical Mode, and Targeted Follower

**Version:** 2.0.0
**Last Updated:** 2026-03-18
**Crate:** `mirage-rs/`

---

> **Reader orientation:** This document specifies the scenario runner, historical replay mode, and targeted follower for mirage-rs, Bardo's in-process EVM fork (section 15). The targeted follower replays only mainnet transactions touching watched contracts, reducing per-block compute by two orders of magnitude versus full block replay. Historical mode pins all reads to a fork block and replays a block range for retrospective analysis. The scenario runner uses Copy-on-Write state layers to branch from a shared baseline and compare strategies in parallel. See `prd2/shared/glossary.md` for full term definitions.

## Targeted Follower

The TargetedFollower is the component that keeps watched protocol state current. It subscribes to new mainnet blocks, scans each block for transactions touching contracts in the watch list, and replays only those transactions through the local EVM. Everything else is ignored -- the lazy-latest read path handles it.

For a typical portfolio of 3-10 DeFi positions, a block with 150 transactions produces 0-5 matches. The computational cost of targeted replay is two orders of magnitude lower than v1's full block replay.

### Classification Rules

```rust
pub struct ClassificationConfig {
    /// Minimum slots written to classify as protocol. Default: 3.
    /// Token transfers write 1-2 slots. Protocol interactions write 3+.
    pub protocol_slot_threshold: usize,

    /// Check for ERC-20/721/1155 interfaces as a secondary signal.
    pub check_token_interface: bool,

    /// Safety valve for watch list size.
    pub max_watched_contracts: usize,

    /// Recursive contagion: watched contracts that interact with
    /// new contracts can add those to the watch list.
    pub enable_contagion: bool,

    /// Maximum contagion depth. Default: 2.
    /// Depth 0 = your direct transaction.
    /// Depth 1 = contracts called by your protocol.
    /// Depth 2 = one level deeper (vault -> lending pool -> oracle).
    pub max_contagion_depth: usize,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            protocol_slot_threshold: 3,
            check_token_interface: true,
            max_watched_contracts: 64,
            enable_contagion: true,
            max_contagion_depth: 2,
        }
    }
}
```

### The Classifier

```rust
pub struct DiffClassifier {
    config: ClassificationConfig,
}

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

impl DiffClassifier {
    pub fn classify(
        &self,
        diff: &StateDiff,
        dirty: &DirtyStore,
        current_block: u64,
    ) -> ClassificationResult {
        let mut result = ClassificationResult::default();

        for (address, account_diff) in &diff.accounts {
            // Already watched -- just update dirty slots.
            if dirty.watch_list.contains_key(address) {
                if !account_diff.storage_written.is_empty() {
                    result.slot_overrides.push((
                        *address,
                        account_diff.storage_written.clone(),
                    ));
                }
                continue;
            }

            // Explicitly unwatched -- slot-only regardless.
            if dirty.unwatch_list.contains(address) {
                if !account_diff.storage_written.is_empty() {
                    result.slot_overrides.push((
                        *address,
                        account_diff.storage_written.clone(),
                    ));
                }
                continue;
            }

            match self.classify_single(*address, account_diff) {
                Classification::Protocol => {
                    let current_count =
                        dirty.watch_list.len() + result.new_watched.len();
                    if current_count >= self.config.max_watched_contracts {
                        // At capacity -- demote to slot-only.
                        result.slot_overrides.push((
                            *address,
                            account_diff.storage_written.clone(),
                        ));
                        continue;
                    }
                    result.new_watched.push((*address, WatchEntry {
                        source: WatchSource::AutoClassified,
                        added_at_block: current_block,
                        initial_slot_count: account_diff.storage_written.len(),
                        replay_count: 0,
                    }));
                    result.slot_overrides.push((
                        *address,
                        account_diff.storage_written.clone(),
                    ));
                }
                Classification::SlotOnly => {
                    if !account_diff.storage_written.is_empty() {
                        result.slot_overrides.push((
                            *address,
                            account_diff.storage_written.clone(),
                        ));
                    }
                }
                Classification::ReadOnly => {
                    result.read_only.push(*address);
                }
            }
        }

        result
    }

    fn classify_single(
        &self,
        address: Address,
        diff: &AccountDiff,
    ) -> Classification {
        if diff.storage_written.is_empty() {
            return Classification::ReadOnly;
        }

        let slot_count = diff.storage_written.len();

        if slot_count >= self.config.protocol_slot_threshold {
            if self.config.check_token_interface
                && self.looks_like_token(address, diff)
            {
                return Classification::SlotOnly;
            }
            return Classification::Protocol;
        }

        Classification::SlotOnly
    }

    /// Heuristic: does this contract look like a token?
    /// If ALL written slots are high-entropy keccak outputs
    /// (mapping entries) and none are low-numbered state slots,
    /// it is likely a token.
    fn looks_like_token(&self, _address: Address, diff: &AccountDiff) -> bool {
        let has_low_slot = diff.storage_written.keys().any(|slot| {
            *slot < U256::from(20)
        });
        !has_low_slot
    }
}
```

The 3-slot threshold is the key heuristic. An ERC-20 transfer writes 1-2 slots (sender balance, receiver balance). A protocol interaction -- LP deposit, borrow, stake -- writes 3+ slots (position state, total liquidity, tick bitmap, etc.). Rebasing tokens that write more slots are caught by the secondary `looks_like_token` check.

### Main Loop

```rust
pub struct TargetedFollower {
    upstream: UpstreamRpc,
    state: Arc<RwLock<MirageState>>,
    classifier: DiffClassifier,
    config: FollowerConfig,
}

pub struct FollowerConfig {
    pub ws_url: String,
    pub http_url: String,
    /// Maximum time per block for replay. Default: 10s.
    pub block_budget: Duration,
    /// Check receipts for aggregator-routed transactions.
    pub use_access_lists: bool,
    pub enable_contagion: bool,
    pub max_contagion_depth: usize,
}

impl TargetedFollower {
    pub async fn run(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let (block_tx, mut block_rx) = tokio::sync::mpsc::channel::<u64>(32);

        // Ingestion task: subscribe to new block headers.
        let upstream = self.upstream.clone();
        let ws_url = self.config.ws_url.clone();
        tokio::spawn(async move {
            Self::block_ingestion_loop(upstream, ws_url, block_tx).await;
        });

        loop {
            tokio::select! {
                Some(block_num) = block_rx.recv() => {
                    // Drain to latest (skip intermediate blocks).
                    let mut latest = block_num;
                    while let Ok(newer) = block_rx.try_recv() {
                        latest = newer;
                    }

                    if let Err(e) = self.process_block(latest).await {
                        tracing::error!(
                            block = latest, error = %e,
                            "block processing failed"
                        );
                    }
                }
                _ = shutdown.changed() => {
                    tracing::info!("targeted follower shutting down");
                    break;
                }
            }
        }
    }
}
```

Skipping intermediate blocks is intentional. If processing is slow, we drain to the newest header rather than queuing a backlog. The lazy-latest read path ensures state remains correct even if the follower falls behind.

### Block Processing

```rust
async fn process_block(&self, block_number: u64) -> Result<()> {
    let start = Instant::now();
    let block = self.upstream.get_block_with_txs(block_number).await?;

    let watch_list: HashSet<Address> = {
        let state = self.state.read();
        state.db.dirty.watch_list.keys().copied().collect()
    };

    if watch_list.is_empty() {
        return Ok(());
    }

    let matched_txs = self.scan_block(&block, &watch_list).await?;
    if matched_txs.is_empty() {
        return Ok(());
    }

    let mut state = self.state.write();
    for tx in &matched_txs {
        if start.elapsed() > self.config.block_budget {
            tracing::warn!(
                block = block_number,
                "block budget exceeded, skipping remaining txs"
            );
            break;
        }
        self.replay_mainnet_tx(&mut state, tx, block_number).await?;
    }

    state.last_mainnet_block = block_number;
    Ok(())
}
```

### Transaction Matching

The matching logic checks in order of cost:

1. `tx.to` (direct call) -- free, covers ~70% of matches.
2. `tx.from` (sender) -- free, covers impersonated accounts.
3. EIP-2930 access list -- free if present in the transaction.
4. Receipt logs (for aggregator-routed transactions) -- one RPC call.

```rust
async fn tx_matches(
    &self,
    tx: &Transaction,
    watch_list: &HashSet<Address>,
) -> bool {
    if let Some(to) = tx.to {
        if watch_list.contains(&to) { return true; }
    }
    if watch_list.contains(&tx.from) { return true; }

    if let Some(ref access_list) = tx.access_list {
        for item in access_list.iter() {
            if watch_list.contains(&item.address) { return true; }
        }
    }

    // For aggregator-routed transactions (1inch, Universal Router,
    // Cowswap), tx.to is the router, not the pool. Catch via receipt logs.
    if self.config.use_access_lists && Self::is_likely_aggregator(tx) {
        if let Ok(receipt) = self.upstream.get_receipt(tx.hash).await {
            for log in &receipt.logs {
                if watch_list.contains(&log.address) { return true; }
            }
        }
    }

    false
}
```

The aggregator detection heuristic checks for known router addresses (Uniswap Universal Router, 1inch, Paraswap, Cowswap) and common function selectors (`execute()`, `swap()`, `multicall()`). This limits receipt fetches to ~5-10 per block instead of ~150.

If the matcher misses a transaction, the consequence is benign: one replay is skipped, and the lazy-latest read path still provides live state for that contract.

### Replay and Contagion

```rust
async fn replay_mainnet_tx(
    &self,
    state: &mut MirageState,
    tx: &Transaction,
    block_number: u64,
) -> Result<()> {
    state.db.pinned_block = Some(block_number - 1);

    let tx_env = TxEnv {
        caller: tx.from,
        transact_to: match tx.to {
            Some(to) => to.into(),
            None => revm::primitives::TxKind::Create,
        },
        data: tx.input.clone(),
        value: tx.value,
        gas_limit: tx.gas_limit,
        ..Default::default()
    };

    let (result, diff) = execute_and_extract(state, tx_env)?;
    state.db.unpin_block();

    match result {
        Ok(_) => {
            for (address, account_diff) in &diff.accounts {
                state.db.dirty.apply_account_changes(
                    *address, account_diff,
                );
            }

            // Contagion: classify newly touched contracts.
            if self.config.enable_contagion {
                let classification = self.classifier.classify(
                    &diff, &state.db.dirty, block_number,
                );
                for (addr, mut entry) in classification.new_watched {
                    entry.source = WatchSource::Contagion {
                        parent: tx.to.unwrap_or_default(),
                    };
                    state.db.dirty.watch_list.insert(addr, entry);
                }
                state.db.dirty.apply_classification(
                    ClassificationResult {
                        new_watched: Vec::new(),
                        slot_overrides: classification.slot_overrides,
                        read_only: classification.read_only,
                    },
                );
            }

            // Invalidate read cache for all touched addresses.
            for address in diff.accounts.keys() {
                state.db.read_cache.invalidate_address(address);
            }

            // Update replay count.
            if let Some(to) = tx.to {
                if let Some(entry) = state.db.dirty.watch_list.get_mut(&to) {
                    entry.replay_count += 1;
                }
            }
        }
        Err(_) => {
            // Mainnet tx reverted locally. Expected when local state
            // has diverged (e.g., the golem's liquidity changed the price
            // enough that a swap's slippage tolerance was exceeded).
        }
    }

    Ok(())
}
```

### Edge Cases

**Same-block ordering.** Multiple transactions touching a watched contract in the same block are replayed in transaction order. This preserves correct sequencing (a large swap followed by a liquidation).

**Composability depth.** A Yearn vault calls Aave which calls Chainlink which calls... Contagion depth 2 catches the vault -> lending pool -> oracle chain without spiraling. Contracts beyond depth 2 read through to mainnet at `latest`.

**Watch list at capacity.** When `max_watched_contracts` (default 64) is reached, new contracts fall back to slot-only classification. The lazy-latest base layer still provides correct reads -- the golem just does not get causal replay for overflow contracts. Graceful degradation, not failure.

**Mainnet tx reverts locally.** When a mainnet swap replays against modified pool state, it may revert because the golem's liquidity changed the price enough that slippage tolerance was exceeded. This is correct: in a real parallel universe, that swap would also fail. The follower logs it and moves on.

**Proxy contracts.** When a transaction calls a proxy (USDC, for example), storage writes happen at the proxy's address. The slot count heuristic counts slots at the address where storage was actually modified. Delegatecall does not change the classification logic.

**Reorg handling.** If the upstream chain reorgs, the follower may have already replayed transactions from the old block. The impact is usually negligible -- most reorgs are 1 block deep. The read cache TTL ensures stale upstream data expires within one block interval.

### Divergence Detection

v2 narrows divergence detection to watched contracts only, since those are the only contracts where local state intentionally detaches from mainnet. After replaying a mainnet transaction, the follower compares the resulting state diff against what mainnet produced. A divergence fires when the local replay produces a different storage value for a watched slot. This is surfaced via `mirage_status` (`divergence_detected: true`) and via `mirage_subscribeEvents`.

---

## Historical Mode

Historical mode forks mirage at a past block and replays forward to a later block. This is how golems perform retrospective analysis: understanding why a position behaved as it did, reconstructing liquidation events, backtesting strategies over historical market conditions.

Historical mode is the controlled return of v1's full-replay behavior. In live mode, v2 reads state lazily at "latest" and only replays transactions touching the watch list. That model breaks for historical work: "latest" during execution is not the same as "latest" at block N. Historical mode solves this by pinning all upstream reads to `--from-block`, disabling TargetedFollower, and using replay sub-modes to walk forward through the block range.

Two research-driven extensions make historical mode more than a replay engine. Reth Execution Extensions provide zero-copy access to historical execution data when running alongside a Reth node, eliminating RPC overhead. Shadow execution replays historical blocks with modified contract bytecode, enabling counterfactual analysis of protocol changes and strategy variations against real market data.

### Invocation

```
mirage-rs \
  --mode historical \
  --from-block 19499990 \
  --to-block 19500010 \
  --replay-mode hybrid \
  --track-addresses 0x...golem,0x...pool \
  --checkpoint-every 100 \
  --output-dir ./replay-output \
  --rpc-url https://mainnet.infura.io/v3/${API_KEY}
```

### CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--mode historical` | required | Activates historical mode |
| `--from-block N` | required | Block to fork at (all upstream reads pin here) |
| `--to-block M` | `latest` | Block to replay up to (inclusive) |
| `--replay-mode` | `hybrid` | `replay`, `stateDiff`, or `hybrid` |
| `--track-addresses` | none | Comma-separated addresses for PnL attribution |
| `--checkpoint-every K` | `0` (disabled) | Save replay state to disk every K blocks |
| `--output-dir` | none | Write PnL CSV and event log here |
| `--max-parallel-fetch` | `4` | Blocks to prefetch in parallel |
| `--shadow-bytecode` | none | Path to modified bytecode for shadow execution |
| `--exex-source` | none | Reth ExEx socket path for direct memory access |
| `--resume-from` | none | Resume replay from a checkpoint file |
| `--quiet` | `false` | Suppress progress output |
| `--json-progress` | `false` | JSON lines on stderr instead of progress bar |

The JSON-RPC server remains available during historical replay. Golems can submit `eth_call` and `eth_sendTransaction` against the replayed state at any point.

### The "latest" Problem

In live mode, HybridDB resolves `pinned_block = None` as `BlockTag::Latest`. In historical mode, this is wrong -- fetching state at "latest" from mainnet gets the current value, not its value at block 19499990.

Historical mode resolves this by setting `pinned_block = from_block` for all upstream reads from startup. The `resolve_block()` method returns `BlockTag::Number(from_block)` instead of `BlockTag::Latest`. This is never relaxed during a historical session.

Key invariant: no upstream read ever fetches state newer than `from_block`. If the upstream node does not have archive data for that block, mirage fails at startup with a clear error.

### Replay Sub-Modes

**`replay` (accurate):** Re-executes every transaction in every block through the local EVM. Accurate gas accounting, full event emission, correct revert reasons. Slow: ~1 block/second on a typical server. When Block-STM is available, parallel execution within each block yields ~3-4x speedup on a 4-core machine. Use when: per-transaction event fidelity matters, or the position depends on event ordering within a block.

**`stateDiff` (fast-forward):** Uses `debug_traceBlockByNumber` with `prestateTracer` to fetch state changes without re-executing transactions locally. ~10x faster than replay. No per-transaction events. Gas accounting unavailable (transactions were not executed locally). Requires an archive node with `debug_traceBlockByNumber` support.

How it works:
1. Fetch `debug_traceBlockByNumber(N, {tracer: "prestateTracer"})` from upstream.
2. Extract the state diff: every account and storage slot that changed.
3. Apply those diffs directly to HybridDB's DirtyStore.
4. Advance the local block counter.

Use when: fast-forwarding through many blocks and only caring about final state at specific checkpoints.

**`hybrid` (default, recommended):** Fast-forward most blocks via StateDiff. Switch to full Replay for blocks where `--track-addresses` have active transactions.

Detection logic:
1. Fetch block header. Scan transaction `to` and `accessList` fields.
2. If any tracked address appears in the block's transaction targets or access lists, mark the block for full Replay.
3. All other blocks: StateDiff fast-forward.

Accurate events and gas accounting for blocks that matter (where the position changed) without paying full replay cost for the rest. On a typical historical range, blocks needing full replay are rare (< 5%), so the pipeline is almost always bound by StateDiff fetch latency rather than EVM execution.

### Reth Execution Extensions

For golems running alongside a Reth full node, Execution Extensions (ExEx) provide direct in-memory access to historical execution data. ExExes are post-execution hooks that run inside the Reth binary, sharing its memory and receiving a stream of `ExExNotification` variants on each block.

```rust
use tokio::sync::mpsc;

pub enum ExExNotification {
    ChainCommitted {
        blocks: Vec<BlockWithSenders>,
        state_diffs: Vec<StateDiff>,
        receipts: Vec<Vec<Receipt>>,
    },
    ChainReverted {
        blocks: Vec<BlockWithSenders>,
    },
    Reorged {
        reverted: Vec<BlockWithSenders>,
        committed: Vec<BlockWithSenders>,
        state_diffs: Vec<StateDiff>,
        receipts: Vec<Vec<Receipt>>,
    },
}

pub struct ExExHistoricalSource {
    rx: mpsc::Receiver<ExExNotification>,
    cached_diffs: HashMap<u64, StateDiff>,
    cached_receipts: HashMap<u64, Vec<Receipt>>,
    range: (u64, u64),
}

impl ExExHistoricalSource {
    /// Request state diffs for a historical range.
    ///
    /// If the Reth node has already executed these blocks (it has,
    /// because ExEx runs post-execution), the diffs are available
    /// in memory without any disk or network access.
    pub async fn replay_range(
        &mut self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<(u64, StateDiff)>> {
        let mut results = Vec::new();

        for block_num in from_block..=to_block {
            if let Some(diff) = self.cached_diffs.get(&block_num) {
                results.push((block_num, diff.clone()));
                continue;
            }

            // Wait for the notification if the block hasn't
            // been processed yet (live replay catching up).
            while let Some(notification) = self.rx.recv().await {
                match notification {
                    ExExNotification::ChainCommitted {
                        blocks,
                        state_diffs,
                        receipts,
                    } => {
                        for (i, block) in blocks.iter().enumerate() {
                            let num = block.number;
                            self.cached_diffs.insert(
                                num,
                                state_diffs[i].clone(),
                            );
                            self.cached_receipts.insert(
                                num,
                                receipts[i].clone(),
                            );

                            if num == block_num {
                                results.push((
                                    num,
                                    state_diffs[i].clone(),
                                ));
                            }
                        }
                        if results.last().map(|(n, _)| *n) == Some(block_num) {
                            break;
                        }
                    }
                    ExExNotification::ChainReverted { blocks } => {
                        // Remove reverted blocks from cache.
                        for block in &blocks {
                            self.cached_diffs.remove(&block.number);
                            self.cached_receipts.remove(&block.number);
                        }
                    }
                    ExExNotification::Reorged {
                        reverted,
                        committed,
                        state_diffs,
                        receipts,
                    } => {
                        for block in &reverted {
                            self.cached_diffs.remove(&block.number);
                        }
                        for (i, block) in committed.iter().enumerate() {
                            let num = block.number;
                            self.cached_diffs.insert(
                                num,
                                state_diffs[i].clone(),
                            );
                            self.cached_receipts.insert(
                                num,
                                receipts[i].clone(),
                            );
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
```

When `--exex-source` is specified, mirage connects to the Reth ExEx socket and uses `ExExHistoricalSource` instead of RPC-based state fetching. Benefits:

- Zero RPC overhead for state diffs (shared memory access).
- Receipts and logs available without separate `eth_getBlockReceipts` calls.
- Reorg awareness built in -- if a reorg happens during replay, the ExEx notification includes both the reverted and new chain segments.
- State diffs are already computed by Reth's execution engine, so there is no redundant work.

### Shadow Execution

Shadow execution replays historical blocks with modified contract bytecode. This answers "what would have happened if the pool used a different fee tier?" or "how would my custom router have performed against last week's swap volume?"

The concept comes from shadow-reth (Shadow.xyz), which implements shadow execution as a Reth ExEx. mirage-rs brings the same capability to the scenario runner without requiring a full Reth node.

```rust
pub struct ShadowConfig {
    /// Map from contract address to replacement bytecode.
    pub bytecode_overrides: HashMap<Address, Bytes>,
    /// Whether to emit "shadow events" from modified bytecode.
    pub emit_shadow_events: bool,
    /// Output path for the shadow diff report.
    pub diff_output: Option<PathBuf>,
}

pub struct ShadowBlockDiff {
    pub block_number: u64,
    pub original_diff: StateDiff,
    pub shadow_diff: StateDiff,
    pub divergent_slots: Vec<(Address, U256, U256, U256)>,
    pub shadow_events: Vec<Log>,
    pub pnl_delta: HashMap<Address, i128>,
}

pub struct ShadowResult {
    pub from_block: u64,
    pub to_block: u64,
    pub block_diffs: Vec<ShadowBlockDiff>,
    pub total_pnl_delta: HashMap<Address, i128>,
    pub first_divergence_block: Option<u64>,
    pub identical_blocks: usize,
}
```

Shadow execution workflow:

1. The golem provides modified bytecode (a new router, a pool with different fee logic, a lending pool with an adjusted liquidation threshold).
2. Historical mode starts normally, pinning reads to `--from-block`.
3. For each block, mirage runs the block twice: once with original bytecode and once with the shadow bytecode applied via `mirage_setCode`.
4. State diffs from both runs are compared. Divergent slots, shadow-only events, and PnL deltas are recorded.
5. The golem receives a structured report: where the modified contract behaved differently, how much PnL it would have gained or lost, and at which block the divergence first appeared.

This closes the sim-to-real gap for protocol development. A golem testing a new LP strategy can shadow-execute it against a week of real mainnet activity and see exactly how it would have performed, including interactions with MEV bots, oracle updates, and other protocol participants that a synthetic simulation cannot reproduce.

### PnL Attribution

When `--track-addresses` and `--output-dir` are specified, mirage writes a PnL report as it replays. At each block boundary, mirage snapshots token balances and position state of tracked addresses. The balance delta from the previous snapshot is the PnL for that block, attributed to one of three sources:

1. **Position changes (own transactions).** Transactions submitted by tracked addresses.
2. **Market movement (replayed mainnet transactions).** Transactions replayed by the follower that modified watched contract state.
3. **Accrued fees (passive).** Balance changes not explained by explicit transactions.

Attribution is approximate. It uses a rule-based classifier, not causal inference: own-address txs map to position changes; watched-contract txs involving tracked addresses map to market movement; the residual maps to accrued fees.

**Output files:**

`pnl.csv`:
```csv
block,timestamp,pnl_usd,pnl_weth,pnl_usdc,attribution
19499990,1700000000,0.00,0,0,baseline
19499991,1700000012,+0.04,0,+40000,accrued_fees
19499995,1700000060,-2.14,-0.001,0,market_movement
19500000,1700000120,+12.45,0,+12450000,own_tx
```

`events.jsonl`:
```json
{"block": 19499991, "tx": "0x...", "contract": "0x...pool", "event": "Swap", "decoded": {...}}
{"block": 19499995, "tx": "0x...", "contract": "0x...oracle", "event": "AnswerUpdated", "decoded": {...}}
```

`shadow_diff.jsonl` (when `--shadow-bytecode` is set):
```json
{"block": 19499991, "address": "0x...pool", "slot": "0x...fee_slot", "original": "0x...500", "shadow": "0x...3000", "pnl_delta_usd": 0.12}
{"block": 19499995, "address": "0x...pool", "slot": "0x...liquidity", "original": "0x...abc", "shadow": "0x...def", "pnl_delta_usd": -0.03}
```

### Progress Reporting

Status file at `--output-dir/status.json`, updated every 10 blocks or 5 seconds:

```json
{
  "mode": "historical",
  "current_block": 19499995,
  "from_block": 19499990,
  "to_block": 19500010,
  "blocks_completed": 5,
  "blocks_total": 20,
  "pct_complete": 25.0,
  "blocks_per_second": 3.2,
  "estimated_remaining_seconds": 4.7,
  "replay_blocks": 2,
  "stateDiff_blocks": 3,
  "last_checkpoint": null
}
```

Progress bar on stderr:
```
[========--------] 52% block 19499995/19500010  3.2 blk/s  ETA 4.7s
```

`--quiet` suppresses progress. `--json-progress` replaces the bar with JSON lines on stderr. They are mutually exclusive.

Ctrl+C triggers a clean interrupt: mirage writes a final checkpoint (if `--checkpoint-every` is set), flushes output files, and exits with code 130.

### Retrospective Use Cases

**"Why was I liquidated?"**

```
mirage-rs --mode historical \
  --from-block 19499990 \
  --to-block 19500010 \
  --replay-mode hybrid \
  --track-addresses 0x...golem,0x...aave_pool,0x...price_oracle \
  --output-dir ./liquidation-analysis
```

Read `events.jsonl` to find: which block had the oracle update that triggered health factor degradation, which block crossed 1.0, which block had the liquidation call and who sent it.

**"How did this LP perform over the last week?"**

```
mirage-rs --mode historical \
  --from-block 19450000 \
  --to-block 19500000 \
  --replay-mode hybrid \
  --track-addresses 0x...golem,0x...pool \
  --checkpoint-every 1000 \
  --output-dir ./weekly-pnl
```

StateDiff fast-forwards through most of the ~50,000 blocks. Full replay triggers only where the golem or pool had active transactions. Result: PnL curve over the week, fee accrual timeline, impermanent loss calculation.

**"Would my custom router have beaten the Universal Router?"**

```
mirage-rs --mode historical \
  --from-block 19400000 \
  --to-block 19500000 \
  --replay-mode replay \
  --shadow-bytecode ./custom_router.bin:0x...universal_router \
  --track-addresses 0x...golem \
  --output-dir ./router-comparison
```

Shadow execution replays every swap that went through the Universal Router, but with the golem's custom bytecode in place. The `shadow_diff.jsonl` shows per-swap PnL differences. If the custom router consistently produces better fills, the golem has evidence to deploy it.

### Range Replay Efficiency

**Parallel block fetching:** While replaying block N, mirage prefetches block N+1's data in a background task. On high-latency RPC connections, this roughly doubles throughput. The prefetch queue holds at most `--max-parallel-fetch` blocks (default 4).

**StateDiff bulk application:** When consecutive blocks are all StateDiff (no tracked addresses active), mirage batches them: fetch diffs for blocks N through N+K in parallel, apply all in sequence. Batch size is `--max-parallel-fetch`.

**Block-STM within replay blocks:** For blocks that require full replay and contain many transactions, Block-STM (Gelashvili et al., 2023) executes transactions in parallel with optimistic concurrency. The multi-version store tracks per-slot versions; conflicting transactions re-execute sequentially. On a 4-core machine, this reduces per-block replay time by roughly 3x for blocks with low conflict rates.

**Checkpoints:** When `--checkpoint-every K` is set, mirage serializes the full DirtyStore to disk every K blocks. Written to `--output-dir/checkpoints/block-{N}.bin`.

On crash or interruption, resume from the most recent checkpoint:

```
mirage-rs --mode historical \
  --from-block 19499990 \
  --to-block 19500010 \
  --resume-from ./replay-output/checkpoints/block-19500000.bin
```

---

## Scenario Runner

The scenario runner is a higher-level abstraction over mirage's snapshot/revert primitives. A golem defines N parameter variations, runs each against a common baseline state, and receives a structured comparison table. The runner handles snapshot management, isolation, timeout enforcement, and result aggregation.

Beyond fixed scenario sets, the runner integrates three research-driven parameter exploration strategies: Latin Hypercube Sampling for space-filling parameter sweeps, Bayesian optimization for iterative refinement toward optima, and Counterfactual Regret Minimization for game-theoretic exit timing analysis. These transform the scenario runner from a brute-force comparator into an intelligent search engine over the simulation space.

### Execution Models

**Sequential (low-spec VMs):** All scenarios run in the same mirage process, one at a time:

```
baseline snapshot -> run scenario 1 -> collect -> revert
                  -> run scenario 2 -> collect -> revert
                  -> run scenario 3 -> collect -> revert
                  -> aggregate -> return results
```

Memory cost: approximately one mirage instance. The snapshot captures baseline state; each scenario modifies it, then reverts. Peak memory is not additive across scenarios. Recommended for 512MB-1GB VMs, scenarios under 30s wall time each, up to ~8 scenarios.

**Parallel (higher-spec VMs):** A coordinator spawns N mirage child processes, each pre-forked at baseline using CoW state layers. Scenarios run concurrently, results collected over JSON-RPC:

```
baseline state (Arc-shared)
    +-- CoW overlay 1 -> scenario 1
    +-- CoW overlay 2 -> scenario 2
    +-- CoW overlay 3 -> scenario 3
    +-- coordinator -> collect all -> aggregate -> return
```

With CoW overlays, per-scenario memory drops from a full state copy (~3.2 MB for 50,000 slots) to the dirty overlay only (~12.8 KB for a typical 200-slot modification). At 8 parallel scenarios: 25 MB becomes 100 KB. The coordinator is a thin wrapper.

| VM RAM | Max parallel scenarios |
|--------|------------------------|
| 1 GB | 2 |
| 2 GB | 4 |
| 4 GB | 8 |
| 8 GB | 16 |

### RPC Protocol

#### `mirage_beginScenarioSet`

Create a new scenario set. Establishes the baseline state that all scenarios branch from.

```json
{ "baseline_state": "latest" }
```

`baseline_state` can be `"latest"` (current mirage state, most common) or a snapshot ID from a previous `evm_snapshot` call. No state is copied yet -- the actual baseline snapshot is taken when `mirage_runScenarioSet` is called, so all scenarios start from the same state even if the caller issues transactions between begin and run.

Returns: `{ "set_id": "abc123" }`

#### `mirage_defineScenario`

Add a scenario to an existing set.

```json
{
  "set_id": "abc123",
  "name": "entry_5eth",
  "transactions": [
    {
      "from": "0x...",
      "to": "0x...",
      "data": "0x...",
      "value": "0x...",
      "gas": "0x7a120"
    }
  ],
  "track_addresses": ["0x...weth", "0x...usdc", "0x...pool"],
  "max_gas": 3000000,
  "timeout_seconds": 30
}
```

`transactions` is an ordered sequence. They run in order, each against the state produced by the previous.

`track_addresses` are addresses whose balances and storage appear in results. Defaults to all addresses touched by the scenario's transactions.

`max_gas` is a per-scenario cumulative gas budget. Default: 10,000,000. The scenario aborts if exceeded.

`timeout_seconds` is wall-clock timeout. Default: 30s. Enforced by OS-level signals in parallel mode, deadline checks between transactions in sequential mode.

Returns: `{ "scenario_id": "abc123-0" }`

#### `mirage_runScenarioSet`

Execute all scenarios in a set.

```json
{ "set_id": "abc123", "mode": "sequential" }
```

`mode` is `"sequential"` or `"parallel"`. Execution is async -- poll `mirage_getScenarioResults` for completion.

Returns: `{ "job_id": "def456" }`

#### `mirage_getScenarioResults`

Poll for results.

```json
{ "job_id": "def456" }
```

Returns:

```json
{
  "job_id": "def456",
  "status": "complete",
  "set_id": "abc123",
  "scenarios": [
    {
      "scenario_id": "abc123-0",
      "name": "entry_1eth",
      "status": "success",
      "gas_used": 312450,
      "wall_time_ms": 145,
      "peak_memory_bytes": 52428800,
      "final_balances": {
        "0x...weth": "0",
        "0x...usdc": "2450000000"
      },
      "position_state": {
        "0x...pool": {
          "tick": -74321,
          "in_range": true,
          "liquidity": "..."
        }
      },
      "logs": [
        {
          "address": "0x...pool",
          "topics": ["0x...Swap"],
          "data": "0x..."
        }
      ],
      "revert_reason": null
    }
  ],
  "total_wall_time_ms": 412,
  "peak_memory_bytes": 52428800
}
```

Job status: `"running"` | `"complete"` | `"failed"`. When running, `scenarios` contains partial results for completed scenarios.

Scenario status: `"success"` | `"reverted"` | `"timeout"` | `"gas_exceeded"` | `"error"`. Reverted scenarios are not failures -- the EVM reverted the last transaction. The runner records the revert reason and continues.

#### `mirage_compareScenarios`

Produce a ranked comparison table from completed results.

```json
{ "job_id": "def456", "metric": "pnl" }
```

**Metric: pnl** -- rank by net balance change in tracked tokens.

```json
{
  "metric": "pnl",
  "baseline_balances": {
    "0x...weth": "1000000000000000000",
    "0x...usdc": "0"
  },
  "ranking": [
    {
      "rank": 1,
      "scenario": "entry_5eth",
      "pnl_usd": 12.45,
      "pnl_token_deltas": {
        "0x...weth": "-5000000000000000000",
        "0x...usdc": "12450000000"
      }
    }
  ]
}
```

**Metric: gas** -- rank by total gas used (lower is better).

**Metric: state_diff** -- per-slot storage changes across scenarios (no ranking, diff view).

```json
{
  "metric": "state_diff",
  "diffs": [
    {
      "address": "0x...pool",
      "slot": "0x...sqrtPriceX96_slot",
      "values": {
        "baseline": "0x...",
        "entry_1eth": "0x...",
        "entry_5eth": "0x...",
        "entry_10eth": "0x..."
      }
    }
  ]
}
```

### Result Structure

```
ScenarioSetResult
+-- job_id: string
+-- status: "running" | "complete" | "failed"
+-- set_id: string
+-- scenarios: ScenarioResult[]
|   +-- scenario_id: string
|   +-- name: string
|   +-- status: "success" | "reverted" | "timeout" | "gas_exceeded" | "error"
|   +-- gas_used: u64
|   +-- wall_time_ms: u64
|   +-- peak_memory_bytes: u64
|   +-- final_balances: { address -> amount }
|   +-- position_state: { address -> protocol-specific state }
|   +-- logs: Log[]
|   +-- revert_reason: string | null
+-- total_wall_time_ms: u64
+-- peak_memory_bytes: u64 (max across all scenarios)
```

### Per-Scenario Limits

Each scenario has:

- `max_gas`: cumulative gas budget across all transactions. Default 10,000,000. Prevents runaway loops.
- `timeout_seconds`: wall-clock timeout. Default 30s. Enforced by SIGTERM in parallel mode, deadline check between transactions in sequential mode.

A scenario that hits either limit gets status `"gas_exceeded"` or `"timeout"`. Other scenarios continue unaffected.

For LHS and Bayesian optimization modes, the per-scenario limits apply to each generated scenario individually. The total evaluation budget is capped by `max_evaluations` in the optimizer config.

### Latin Hypercube Sampling for Parameter Exploration

When a golem needs to explore a continuous parameter space (entry sizes, slippage tolerances, gas prices), fixed grids waste samples by clustering. Latin Hypercube Sampling (McKay, Beckman & Conover, 1979) divides each input dimension into N equal-probability intervals and draws exactly one sample per interval per dimension, producing a space-filling design that converges roughly 50% faster than pure Monte Carlo.

```rust
use rand::Rng;

pub enum ParameterDimension {
    /// Log-normal distribution for trade sizes, token amounts.
    LogNormal { mu: f64, sigma: f64, min: f64, max: f64 },
    /// Uniform distribution for slippage tolerances, fee tiers.
    Uniform { min: f64, max: f64 },
    /// Beta distribution for probability parameters.
    Beta { alpha: f64, beta: f64, scale: f64 },
    /// Discrete set of values (pool fee tiers, protocol addresses).
    Discrete(Vec<f64>),
}

pub struct ParameterSpace {
    pub dimensions: Vec<(String, ParameterDimension)>,
}

/// Generate an N x M Latin Hypercube sample matrix.
/// Each column corresponds to a dimension. Each row is a scenario's
/// parameter vector. Within each column, the N samples fall in
/// distinct strata.
pub fn latin_hypercube(
    space: &ParameterSpace,
    n_samples: usize,
    rng: &mut impl Rng,
) -> Vec<Vec<f64>> {
    let m = space.dimensions.len();
    let mut matrix = vec![vec![0.0f64; m]; n_samples];

    for (col, (_name, dim)) in space.dimensions.iter().enumerate() {
        let mut strata: Vec<f64> = (0..n_samples)
            .map(|i| {
                let low = i as f64 / n_samples as f64;
                let high = (i + 1) as f64 / n_samples as f64;
                rng.gen_range(low..high)
            })
            .collect();

        // Shuffle to break correlation between dimensions.
        // The Iman-Conover method (1982) can preserve desired
        // inter-variable correlations here if needed.
        shuffle(&mut strata, rng);

        // Map each uniform [0,1] sample through the dimension's
        // inverse CDF (quantile function).
        for (row, &u) in strata.iter().enumerate() {
            matrix[row][col] = dim.quantile(u);
        }
    }

    matrix
}

fn shuffle(v: &mut [f64], rng: &mut impl Rng) {
    for i in (1..v.len()).rev() {
        let j = rng.gen_range(0..=i);
        v.swap(i, j);
    }
}

impl ParameterDimension {
    /// Inverse CDF: map a uniform [0,1] sample to the dimension's
    /// distribution. This is the standard quantile transform.
    fn quantile(&self, u: f64) -> f64 {
        match self {
            Self::LogNormal { mu, sigma, min, max } => {
                // Approximate inverse normal CDF (Abramowitz & Stegun).
                let z = inv_normal_cdf(u);
                let raw = (mu + sigma * z).exp();
                raw.clamp(*min, *max)
            }
            Self::Uniform { min, max } => min + u * (max - min),
            Self::Beta { alpha, beta, scale } => {
                // Use regularized incomplete beta function inverse.
                inv_beta_cdf(u, *alpha, *beta) * scale
            }
            Self::Discrete(values) => {
                let idx = (u * values.len() as f64).floor() as usize;
                values[idx.min(values.len() - 1)]
            }
        }
    }
}
```

The golem workflow:

1. Define parameter distributions (log-normal for trade size, beta for slippage fraction, uniform for gas price).
2. Call `mirage_beginScenarioSet` with `"mode": "lhs"` and `"n_samples": 32`.
3. The runner generates an LHS sample matrix, creates 32 scenarios with the sampled parameters, and executes them.
4. Results include the parameter vector for each scenario alongside PnL, gas, and state diffs.
5. Post-processing: empirical CDF of PnL outcomes, Value-at-Risk (VaR) at the 5th percentile, Conditional VaR (expected shortfall), and sensitivity analysis (partial rank correlation between each parameter and PnL).

For 32 LHS samples across 4 dimensions, the space-filling property means the golem has at least one sample in every region of the parameter space. A naive grid with the same coverage (3 levels per dimension) would need 81 scenarios and still miss the gaps between grid points.

### Bayesian Optimization for Iterative Refinement

LHS explores broadly. Bayesian optimization (Snoek, Larochelle & Adams, 2012) narrows in on optima by building a Gaussian Process surrogate model and selecting the next evaluation point where improvement is most likely.

```rust
use std::collections::VecDeque;

pub struct GaussianProcess {
    x_observed: Vec<Vec<f64>>,
    y_observed: Vec<f64>,
    length_scales: Vec<f64>,
    noise_var: f64,
    cholesky_cache: Option<Vec<Vec<f64>>>,
}

pub enum AcquisitionFunction {
    /// Expected Improvement: exploit near the current best.
    ExpectedImprovement,
    /// Upper Confidence Bound: balance explore vs exploit.
    UpperConfidenceBound { kappa: f64 },
    /// Thompson Sampling: natural explore/exploit balance.
    ThompsonSampling,
}

pub struct BayesianOptimizer {
    gp: GaussianProcess,
    acquisition: AcquisitionFunction,
    space: ParameterSpace,
    max_evaluations: usize,
    /// Initial exploration via LHS before switching to GP-guided search.
    initial_samples: usize,
    history: VecDeque<(Vec<f64>, f64)>,
}

impl BayesianOptimizer {
    /// Phase 1 (first initial_samples): LHS for broad prior.
    /// Phase 2 (remaining): GP-guided search via acquisition function.
    pub fn suggest_next(&mut self) -> Vec<f64> {
        if self.gp.x_observed.len() < self.initial_samples {
            return self.next_lhs_point();
        }
        self.gp.fit();
        match &self.acquisition {
            AcquisitionFunction::ExpectedImprovement => self.maximize_ei(),
            AcquisitionFunction::UpperConfidenceBound { kappa } => {
                self.maximize_ucb(*kappa)
            }
            AcquisitionFunction::ThompsonSampling => self.thompson_sample(),
        }
    }

    pub fn record_observation(&mut self, params: Vec<f64>, objective: f64) {
        self.gp.x_observed.push(params.clone());
        self.gp.y_observed.push(objective);
        self.gp.cholesky_cache = None;
        self.history.push_back((params, objective));
    }
}

/// Expected Improvement acquisition function.
///
/// EI(x) = (mu(x) - f_best) * Phi(z) + sigma(x) * phi(z)
/// where z = (mu(x) - f_best) / sigma(x)
///
/// High EI at points where either the predicted mean exceeds
/// the current best (exploitation) or uncertainty is large
/// (exploration). The balance is automatic.
fn expected_improvement(
    mu: f64,
    sigma: f64,
    f_best: f64,
) -> f64 {
    if sigma < 1e-12 {
        return (mu - f_best).max(0.0);
    }
    let z = (mu - f_best) / sigma;
    let phi = (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let big_phi = 0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2));
    (mu - f_best) * big_phi + sigma * phi
}
```

The optimization loop runs as a sequence of scenario sets:

1. **Initial exploration** (8-16 LHS samples): build a broad prior over the parameter space.
2. **GP-guided iteration** (remaining budget): each round, the optimizer suggests 1-4 parameter vectors, the runner executes them as a scenario set, results feed back into the GP.
3. **Convergence check**: stop early if the best observed objective hasn't improved in 3 consecutive rounds, or if the GP's predicted improvement everywhere falls below a threshold.
4. **Return**: the full history of (parameters, objective) pairs, plus the GP's predicted optimum.

For a golem optimizing LP entry size, this typically converges in 20-30 total evaluations where a grid search would require 100+.

### Counterfactual Regret Minimization for Exit Timing

Position exit timing is a sequential decision problem: at each block, the golem can exit or hold. CFR (Zinkevich et al., 2007) decomposes overall regret into per-decision-point counterfactual regret and converges toward optimal timing policies.

The scenario runner's historical mode integration makes this practical: fork at the entry block, fast-forward via StateDiff to each candidate exit block, execute the counterfactual exit, record the PnL.

```rust
pub struct CfrAnalyzer {
    exit_blocks: Vec<u64>,
    counterfactual_pnl: Vec<f64>,
    actual_pnl: f64,
    /// Exp3 weights for exit timing policies.
    policy_weights: Vec<f64>,
    /// Exp3 learning rate. Bounded by O(sqrt(K * T * ln(K))).
    eta: f64,
}

impl CfrAnalyzer {
    pub fn new(exit_blocks: Vec<u64>, actual_pnl: f64) -> Self {
        let k = exit_blocks.len();
        let eta = ((k as f64).ln() / k as f64).sqrt();
        Self {
            policy_weights: vec![1.0 / k as f64; k],
            exit_blocks,
            counterfactual_pnl: Vec::new(),
            actual_pnl,
            eta,
        }
    }

    pub fn record_exit_pnl(&mut self, block: u64, pnl: f64) {
        if let Some(idx) = self.exit_blocks.iter().position(|&b| b == block) {
            if self.counterfactual_pnl.len() <= idx {
                self.counterfactual_pnl.resize(idx + 1, 0.0);
            }
            self.counterfactual_pnl[idx] = pnl;
        }
    }

    pub fn compute_regret(&self) -> CfrResult {
        let best_pnl = self.counterfactual_pnl
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let best_idx = self.counterfactual_pnl
            .iter()
            .position(|&p| p == best_pnl)
            .unwrap_or(0);

        let per_exit_regret: Vec<f64> = self.counterfactual_pnl
            .iter()
            .map(|&p| best_pnl - p)
            .collect();

        CfrResult {
            best_exit_block: self.exit_blocks[best_idx],
            best_pnl,
            actual_pnl: self.actual_pnl,
            total_regret: best_pnl - self.actual_pnl,
            per_exit_regret,
            policy_weights: self.policy_weights.clone(),
        }
    }

    /// Update Exp3 policy weights based on observed counterfactual PnL.
    ///
    /// Exp3 (Auer et al., 2002) guarantees O(sqrt(KT ln K)) regret
    /// over T rounds with K arms. The Discounted CFR variant
    /// (Brown & Sandholm, 2019) discounts early iterations for
    /// faster convergence in non-stationary environments.
    pub fn update_policy(&mut self) {
        let k = self.exit_blocks.len();
        if self.counterfactual_pnl.len() != k { return; }

        let min_pnl = self.counterfactual_pnl.iter()
            .cloned().fold(f64::INFINITY, f64::min);
        let max_pnl = self.counterfactual_pnl.iter()
            .cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_pnl - min_pnl).max(1e-12);

        for i in 0..k {
            let reward = (self.counterfactual_pnl[i] - min_pnl) / range;
            let estimated_reward = reward / self.policy_weights[i];
            self.policy_weights[i] *= (self.eta * estimated_reward).exp();
        }

        let sum: f64 = self.policy_weights.iter().sum();
        for w in &mut self.policy_weights {
            *w /= sum;
        }
    }
}

pub struct CfrResult {
    pub best_exit_block: u64,
    pub best_pnl: f64,
    pub actual_pnl: f64,
    pub total_regret: f64,
    pub per_exit_regret: Vec<f64>,
    pub policy_weights: Vec<f64>,
}
```

The workflow ties together the scenario runner and historical mode:

1. The golem specifies an entry block, a range of candidate exit blocks (e.g., every 100 blocks over a week), and the position parameters.
2. The runner forks at the entry block using historical mode.
3. For each candidate exit block, it fast-forwards via StateDiff, executes the counterfactual exit transaction, and records the PnL.
4. CFR analysis produces the regret profile: which exit point was optimal, how much the golem lost by exiting when it did (or by holding), and updated policy weights that the golem can use for future exit timing.
5. Over multiple retrospective analyses, the Exp3 weights converge toward the timing regions that historically produced the best exits, giving the golem a learned prior for future decisions.

---

## Golem Use Cases

### Entry Size Optimization

Scenarios: 0.1, 0.5, 1, 2, 5, 10 ETH deposits. Metric: `pnl`. Larger entries face more slippage; the golem finds the size where marginal slippage exceeds marginal capital efficiency gain. For continuous optimization: Bayesian optimization mode with a log-normal parameter distribution over [0.01, 50] ETH converges in 15-20 evaluations.

### Timing Sensitivity

Same swap at 5, 10, 20, 50, 100 gwei base fee (using `evm_setNextBlockBaseFeePerGas` before each scenario's transaction). Metric: `pnl`. If outcomes diverge at high gas prices, there is a threshold above which MEV degrades fill quality.

### Protocol Comparison

10 ETH into Uniswap v3, Uniswap v4, and Curve, followed by simulated swap volume. Metric: `pnl` -- total fees accrued minus impermanent loss.

### Stress Testing

32 LHS samples across 3 dimensions: ETH price change (uniform -30% to +30%), BTC price change (correlated with ETH via Iman-Conover), gas spike multiplier (log-normal). Metric: `state_diff` -- which parameter combinations produce health factor below 1.0 (liquidation risk), with sensitivity analysis showing which parameter contributes most to liquidation risk.

### Exit Timing Analysis

CFR mode: candidate exits every 500 blocks (200 exit points). The runner fast-forwards via StateDiff to each candidate, simulates the exit, computes the regret profile. Over repeated analyses, Exp3 weights converge toward optimal timing heuristics.

---

## Interaction Between Live and Historical Modes

| Component | Live mode | Historical mode |
|-----------|-----------|-----------------|
| HybridDB resolve_block() | `latest` | `from_block` (pinned) |
| TargetedFollower | Active | Disabled |
| DiffClassifier | Auto-classifies from local txs | Used during full Replay |
| StateDiff | Not used | Core fast-forward primitive |
| ReadCache TTL | 12s default | Longer TTL viable (state is static) |
| Snapshot/revert | Available for user use | Also used by checkpoints |
| Block-STM | Not needed (few txs per block) | Applied to full-block replay |
| BytecodeCache | Shared across live sessions | Shared; warm from prior replays |
| ExEx source | Receives live block notifications | Receives historical block data |

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32050 | SET_NOT_FOUND | Scenario set ID not found |
| -32051 | SET_ALREADY_RUNNING | Cannot define scenarios after the set started |
| -32052 | SET_HAS_NO_SCENARIOS | Run called on empty set |
| -32053 | PARALLEL_UNAVAILABLE | Parallel mode not supported on this instance |
| -32054 | JOB_NOT_FOUND | Scenario job ID not found |
| -32055 | JOB_NOT_COMPLETE | Results requested before job finished |

---

## Cross-References

- Architecture overview: [01-mirage-rs.md](./01-mirage-rs.md) -- Core architecture: HybridDB three-tier database, DirtyStore, targeted follower, CoW state layers, Block-STM parallel execution
- RPC method reference: [01b-mirage-rpc.md](./01b-mirage-rpc.md) -- Full JSON-RPC method catalog: eth_*, mirage_*, evm_*, hardhat/anvil compatibility, error codes
- Bardo TUI integration, golem workflows: [01d-mirage-integration.md](./01d-mirage-integration.md) -- F6 fork workflow, Fork Inspector overlay, golem sidecar lifecycle, CorticalState pressure gating
- HDC fingerprints for scenario result similarity analysis: [04-hyperdimensional-computing/](../04-hyperdimensional-computing/) -- Hyperdimensional computing module used to fingerprint scenario results for similarity clustering
- DDSketch for PnL distribution tracking across scenario results: [05-data-structures/](../05-data-structures/) -- Probabilistic data structures (DDSketch, t-digest) used to track PnL distributions across scenario runs

## References

- Auer, P. et al. (2002). The nonstochastic multiarmed bandit problem. *SIAM Journal on Computing*, 32(1). -- *Introduces the Exp3 algorithm for adversarial bandit settings; informs the scenario runner's adaptive parameter selection when exploring strategy spaces.*
- Brown, N. & Sandholm, T. (2019). Solving imperfect-information games via discounted regret minimization. *AAAI*. -- *Proposes discounted regret minimization for faster convergence in game-theoretic settings; relevant to multi-agent scenario evaluation.*
- Gelashvili, R. et al. (2023). Block-STM: Scaling blockchain execution by turning ordering curse to a performance blessing. *PPoPP*. -- *Introduces optimistic parallel transaction execution with per-slot version tracking; the algorithm behind mirage-rs's parallel historical replay.*
- Kim, Y. et al. (2021). An off-the-chain execution environment for scalable testing and profiling of smart contracts. *USENIX ATC*. -- *Describes an off-chain execution environment for smart contract profiling; informs the isolation model for scenario execution.*
- McKay, M.D., Beckman, R.J. & Conover, W.J. (1979). A comparison of three methods for selecting values of input variables. *Technometrics*, 21(2). -- *Introduces Latin Hypercube Sampling, the space-filling method used by the scenario runner's parameter sweep to efficiently cover high-dimensional strategy spaces.*
- Saraph, V. & Herlihy, M. (2019). An empirical study of speculative concurrency in Ethereum smart contracts. *arXiv:1901.01376*. -- *Measures read-write conflict rates across real Ethereum blocks (<5% on typical DeFi blocks); justifies Block-STM's high effective parallelism.*
- Shadow.xyz (2024). shadow-reth: Shadow execution via Reth Execution Extensions. *GitHub*. -- *Implements shadow execution alongside a production node via Reth extensions; informs the targeted follower's approach to selective mainnet transaction replay.*
- Snoek, J., Larochelle, H. & Adams, R.P. (2012). Practical Bayesian optimization of machine learning algorithms. *NeurIPS*. -- *Demonstrates Gaussian-process-based Bayesian optimization for hyperparameter tuning; informs the scenario runner's adaptive strategy search.*
- Zinkevich, M. et al. (2007). Regret minimization in games with incomplete information. *NeurIPS*. -- *Introduces counterfactual regret minimization (CFR) for solving extensive-form games; relevant to multi-agent DeFi strategy evaluation.*

# Mirage-RS v2 Engine Testing [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Companion to:** `16-testing/04-mirage.md` (v1 mirage testing)

> Tests for the mirage-rs v2 simulation engine: HybridDB three-tier reads, DirtyStore correctness, Copy-on-Write branching isolation, state diff classification, TargetedFollower replay, and Block-STM parallel execution determinism.

> **Reader orientation:** This document specifies tests for the mirage-rs v2 simulation engine, covering HybridDB three-tier reads, DirtyStore correctness, Copy-on-Write branching isolation, state diff classification, TargetedFollower replay, and Block-STM parallel execution determinism. It belongs to Section 16 (Testing) and verifies the structural correctness of the simulation layer that Golems (mortal autonomous agents) use during development and backtesting. Simulation accuracy against on-chain results is covered separately in `./12-simulation-validation.md` (sim-to-real accuracy tests). See `prd2/shared/glossary.md` for full term definitions.

---

## Document Map

| Section | Topic |
|---------|-------|
| Overview | What v2 testing covers and why it differs from v1 |
| HybridDB Read Priority | Three-tier read ordering correctness |
| DirtyStore Correctness | Write isolation, snapshot/revert, watch list integrity |
| CoW Branching Isolation | Branch independence, overlay-only mutation, baseline immutability |
| State Diff Classification | Protocol vs token vs read-only classification accuracy |
| TargetedFollower Replay | Transaction matching, replay correctness, contagion bounds |
| Block-STM Determinism | Parallel execution produces same results as sequential |
| Snapshot/Revert | Snapshot consumption, revert correctness, nested snapshots |
| Property Test Sketches | Rust `proptest` harnesses for key invariants |

---

## Overview

mirage-rs v1 replayed every mainnet block through a local revm instance. v2 inverts that model: lazy-latest reads from upstream, automatic dirty tracking of local mutations, and targeted replay of only the mainnet transactions touching watched contracts. This inversion changes what needs testing.

v1 testing focused on "does the full replay produce the same state as mainnet?" v2 testing focuses on "does the three-tier read priority produce correct results when local mutations overlap with upstream state?" and "do parallel scenarios running on CoW branches contaminate each other?"

The test properties here are structural. They verify the database layer, the classification heuristics, and the branching model. Simulation accuracy against actual on-chain results is covered separately in `16-testing/12-simulation-validation.md`.

### Dependencies

- `16-testing/04-mirage.md` -- v1 mirage testing (predecessor)
- `16-testing/12-simulation-validation.md` -- accuracy validation against on-chain data
- Research: `02-mirage-rs/00-architecture.md` -- HybridDB, DirtyStore, CoW, Block-STM

---

## 1. HybridDB Read Priority [SPEC]

The HybridDB implements revm's `Database` trait with a three-tier read priority: DirtyStore first, then ReadCache, then upstream RPC at `latest` (or at `pinned_block` in historical mode). Every read must respect this ordering. A violation means the golem sees stale or incorrect state during simulation.

### 1.1 Property: DirtyStore always overrides ReadCache and upstream [SPEC]

**Statement:** For any address A and storage slot S, if `DirtyStore` contains a value V for (A, S), then `HybridDB::storage(A, S)` returns V regardless of what the ReadCache or upstream would return.

**Test approach:** Property test with random address/slot/value triples.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn dirty_store_always_wins(
        addr in any::<[u8; 20]>().prop_map(Address::from),
        slot in any::<U256>(),
        dirty_value in any::<U256>(),
        cache_value in any::<U256>(),
    ) {
        let mut db = HybridDB::test_instance();

        // Plant a value in the read cache.
        db.read_cache.insert_storage(addr, slot, cache_value, 100);

        // Plant a different value in the dirty store.
        db.dirty.accounts
            .entry(addr)
            .or_default()
            .storage
            .insert(slot, dirty_value);

        // Read must return the dirty value.
        let result = db.storage(addr, slot).unwrap();
        prop_assert_eq!(result, dirty_value);
    }
}
```

**Failure mode:** If the DirtyStore lookup is skipped or short-circuited, the golem would read upstream state for slots it has already modified locally. This silently corrupts simulation results.

### 1.2 Property: ReadCache serves reads when DirtyStore has no entry [SPEC]

**Statement:** For any (A, S) not in DirtyStore, if ReadCache holds value V, `HybridDB::storage(A, S)` returns V without making an upstream RPC call.

**Test approach:** Instrument the upstream client with a call counter. Verify zero RPC calls when the cache is warm.

```rust
#[test]
fn read_cache_prevents_rpc_calls() {
    let mut db = HybridDB::test_instance();
    let counter = db.upstream.call_counter();

    let addr = address!("0xdead...");
    let slot = U256::from(42);
    let cached = U256::from(999);

    db.read_cache.insert_storage(addr, slot, cached, 100);

    let result = db.storage(addr, slot).unwrap();
    assert_eq!(result, cached);
    assert_eq!(counter.get(), 0, "should not call upstream when cache is warm");
}
```

### 1.3 Property: Cache TTL expiry forces upstream re-fetch [SPEC]

**Statement:** After `cache_ttl` elapses, a ReadCache entry is stale. The next read for that slot must fetch from upstream and update the cache.

**Test approach:** Set a short TTL (10ms), insert a value, sleep past TTL, read again, verify an upstream call was made.

### 1.4 Property: `pinned_block` consistency within a single EVM execution [SPEC]

**Statement:** When `pinned_block` is set (historical mode), all upstream reads within a single `transact()` call use the same block number. No read sees state from a different block.

**Test approach:** Mock the upstream to log requested block numbers. Execute a transaction that reads multiple slots. Assert all upstream requests specify the same block.

### 1.5 Property: `basic()` merges dirty fields with upstream [SPEC]

**Statement:** When DirtyStore contains partial account info (e.g., only a balance override), the remaining fields (nonce, code) are fetched from upstream and merged correctly.

```rust
proptest! {
    #[test]
    fn basic_merges_partial_dirty(
        addr in any::<[u8; 20]>().prop_map(Address::from),
        dirty_balance in any::<U256>(),
        upstream_nonce in any::<u64>(),
    ) {
        let mut db = HybridDB::test_instance();

        // Dirty store: only balance overridden.
        db.dirty.accounts.entry(addr).or_default().balance = Some(dirty_balance);

        // Upstream returns a different nonce.
        db.upstream.mock_account(addr, AccountInfo {
            nonce: upstream_nonce,
            balance: U256::from(0),
            ..Default::default()
        });

        let info = db.basic(addr).unwrap().unwrap();
        prop_assert_eq!(info.balance, dirty_balance, "balance from dirty store");
        prop_assert_eq!(info.nonce, upstream_nonce, "nonce from upstream");
    }
}
```

---

## 2. DirtyStore Correctness [SPEC]

### 2.1 Property: Local transactions write only to DirtyStore [SPEC]

**Statement:** After `handle_local_transaction()`, all state changes land in `DirtyStore.accounts`. The ReadCache and upstream are not modified.

**Test approach:** Execute a local transaction. Check that ReadCache entry count did not increase. Check that the upstream mock received no write calls.

### 2.2 Property: Watch list grows only from classified transactions [SPEC]

**Statement:** The watch list gains entries only through `DiffClassifier` during local transaction processing or through manual `mirage_watchContract` calls. No other code path adds to the watch list.

**Test approach:** Run a sequence of read-only `eth_call` operations. Assert watch list size remains zero.

### 2.3 Property: Unwatch list prevents re-addition [SPEC]

**Statement:** An address in `DirtyStore.unwatch_list` is never added back to `watch_list` by auto-classification, even if a subsequent transaction writes 10+ slots to it.

```rust
#[test]
fn unwatch_prevents_readd() {
    let mut store = DirtyStore::new();
    let addr = address!("0xbeef...");

    store.unwatch_list.insert(addr);

    // Simulate a state diff with 10 written slots.
    let diff = make_diff(addr, 10);
    let classification = DiffClassifier::default().classify(&diff, &store, 100);

    store.apply_classification(classification);
    assert!(!store.watch_list.contains_key(&addr));
}
```

### 2.4 Property: `total_dirty_slots` tracks accurately [SPEC]

**Statement:** `DirtyStore.total_dirty_slots` equals the sum of `storage.len()` across all `DirtyAccount` entries. Verified after every mutation.

---

## 3. Copy-on-Write Branching Isolation [SPEC]

CoW state layers are the foundation for parallel scenario execution and Sanctum F6 forks. Isolation failures here mean one scenario's state leaks into another, producing incorrect comparisons.

### 3.1 Property: Branches share the same baseline [SPEC]

**Statement:** Two `CowState::branch()` calls from the same `Arc<baseline>` start with identical state. Any slot readable in branch A is readable in branch B with the same value, before either branch writes anything.

```rust
proptest! {
    #[test]
    fn cow_branches_share_baseline(
        slots in prop::collection::hash_map(
            (any::<[u8; 20]>().prop_map(Address::from), any::<U256>()),
            any::<U256>(),
            1..100
        ),
    ) {
        let baseline = Arc::new(slots.clone());
        let branch_a = CowState::branch(&baseline);
        let branch_b = CowState::branch(&baseline);

        for ((addr, slot), expected) in &slots {
            let a_val = branch_a.read(*addr, *slot);
            let b_val = branch_b.read(*addr, *slot);
            prop_assert_eq!(a_val, Some(*expected));
            prop_assert_eq!(b_val, Some(*expected));
        }
    }
}
```

### 3.2 Property: Writes to branch A do not appear in branch B [SPEC]

**Statement:** After `branch_a.write(addr, slot, new_value)`, `branch_b.read(addr, slot)` still returns the baseline value (or `None` if the slot was not in the baseline). This is the core isolation guarantee.

```rust
proptest! {
    #[test]
    fn cow_branches_are_isolated(
        addr in any::<[u8; 20]>().prop_map(Address::from),
        slot in any::<U256>(),
        baseline_val in any::<U256>(),
        branch_a_val in any::<U256>(),
    ) {
        let baseline = Arc::new(
            [((addr, slot), baseline_val)].into_iter().collect()
        );
        let mut branch_a = CowState::branch(&baseline);
        let branch_b = CowState::branch(&baseline);

        branch_a.write(addr, slot, branch_a_val);

        // Branch B must still see baseline.
        prop_assert_eq!(branch_b.read(addr, slot), Some(baseline_val));
        // Branch A sees its own write.
        prop_assert_eq!(branch_a.read(addr, slot), Some(branch_a_val));
    }
}
```

### 3.3 Property: Baseline is never mutated [SPEC]

**Statement:** The `Arc<HashMap>` baseline is immutable after creation. No write to any branch modifies the baseline's reference count contents. Verified by `Arc::strong_count()` remaining stable and by re-reading baseline values after branch mutations.

### 3.4 Property: Overlay size reflects only branch-local writes [SPEC]

**Statement:** `CowState::overlay_size()` equals the number of unique (address, slot) pairs written to that specific branch. Writing the same slot twice in one branch counts as 1. Writing the same slot in two different branches counts as 1 in each.

### 3.5 Memory scaling [SPEC]

**Test:** Create a baseline with 50,000 slots. Branch 8 times. Each branch writes 200 unique slots. Verify total memory is approximately `50,000 * sizeof(entry) + 8 * 200 * sizeof(entry)`, not `9 * 50,000 * sizeof(entry)`.

```rust
#[test]
fn cow_memory_scales_with_overlays_not_baseline() {
    let baseline: HashMap<(Address, U256), U256> =
        (0..50_000u64).map(|i| {
            let addr = Address::from_word(B256::from(U256::from(i)));
            ((addr, U256::from(i)), U256::from(i * 100))
        }).collect();
    let baseline = Arc::new(baseline);

    let mut branches: Vec<CowState> = (0..8)
        .map(|_| CowState::branch(&baseline))
        .collect();

    for (b, branch) in branches.iter_mut().enumerate() {
        for s in 0..200u64 {
            let addr = Address::from_word(B256::from(U256::from(b as u64 * 1000 + s)));
            branch.write(addr, U256::from(s), U256::from(999));
        }
        assert_eq!(branch.overlay_size(), 200);
    }

    // Baseline Arc has 1 (owner) + 8 (branches) strong references.
    assert_eq!(Arc::strong_count(&baseline), 9);
}
```

---

## 4. State Diff Classification [SPEC]

The `DiffClassifier` decides whether a contract enters the watch list (protocol), gets slot-level overrides only (token), or is ignored (read-only). Misclassification means either unnecessary replay overhead (token classified as protocol) or missing state updates (protocol classified as token).

### 4.1 Property: Contracts with >= N written slots classify as Protocol [SPEC]

**Statement:** When a single transaction writes N or more storage slots to a contract (where N = `protocol_slot_threshold`, default 3), the contract is classified as `Classification::Protocol`.

```rust
proptest! {
    #[test]
    fn protocol_classification_at_threshold(
        slot_count in 3..100usize,
    ) {
        let config = ClassificationConfig::default();
        let diff = make_diff_with_slots(test_address(), slot_count);
        let result = DiffClassifier::new(config).classify_contract(&diff);
        prop_assert_eq!(result, Classification::Protocol);
    }
}
```

### 4.2 Property: Contracts with < N written slots classify as SlotOnly [SPEC]

**Statement:** A contract with 1-2 written slots (below default threshold) classifies as `Classification::SlotOnly`, not Protocol.

### 4.3 Property: ERC-20 heuristic overrides slot count for token transfers [SPEC]

**Statement:** When `check_token_interface` is enabled, a contract with 3+ written slots that match the token transfer pattern (high-entropy keccak slots, no low-numbered state variables) classifies as `Classification::SlotOnly` instead of `Protocol`.

**Test case:** Simulate a rebasing token (stETH) that writes to `totalSupply`, `rebaseIndex`, and `balanceOf[user]` in a single transfer -- 3 slots, all mapping-derived keccak outputs.

### 4.4 Property: Read-only transactions produce no classification [SPEC]

**Statement:** A transaction that reads but writes zero storage slots produces `Classification::ReadOnly`. No watch list or slot override changes.

### 4.5 Contagion depth bound [SPEC]

**Statement:** Recursive contagion from `TargetedFollower` replay never exceeds `max_contagion_depth`. After depth D, new contracts encountered during replay fall back to `Classification::SlotOnly`.

**Test approach:** Create a chain of contracts: A calls B calls C calls D. Set `max_contagion_depth = 2`. Replay from A. Verify A and B enter the watch list, but C does not (depth exceeded).

### 4.6 Watch list saturation [SPEC]

**Statement:** When the watch list reaches `max_watched_contracts`, new contracts always classify as `SlotOnly` regardless of slot count.

---

## 5. TargetedFollower Replay [SPEC]

### 5.1 Property: Matching precision for direct calls [SPEC]

**Statement:** A transaction where `tx.to` matches a watched contract address is always identified as a match. False negative rate on direct calls = 0%.

### 5.2 Property: Empty watch list means zero replays [SPEC]

**Statement:** When `watch_list.is_empty()`, `process_block()` returns immediately without fetching block data. Verified by zero upstream calls.

### 5.3 Property: Replay respects transaction ordering within a block [SPEC]

**Statement:** When multiple matched transactions appear in the same block, they are replayed in the same order they appeared on-chain (by transaction index).

### 5.4 Property: Reverted mainnet transactions are handled gracefully [SPEC]

**Statement:** When a replayed mainnet transaction reverts locally (because the golem's state has diverged), the follower logs the revert and continues processing subsequent transactions. No panic, no state corruption.

### 5.5 Contagion from replayed transactions [SPEC]

**Statement:** When a replayed mainnet transaction touches a new contract that exceeds the slot threshold, that contract enters the watch list with `WatchSource::Contagion { parent }`.

---

## 6. Block-STM Parallel Execution Determinism [SPEC]

Block-STM enables parallel transaction execution within a single block for historical replay. The core property: the final state after parallel execution must be identical to sequential execution.

### 6.1 Property: Parallel execution matches sequential execution [SPEC]

**Statement:** For any block of N transactions, executing them through Block-STM with K worker threads produces the same final state as executing them sequentially (one at a time, in order).

```rust
proptest! {
    #[test]
    fn block_stm_deterministic(
        // Generate random blocks with known state interactions.
        txs in prop::collection::vec(arb_transaction(), 10..200),
        num_threads in 1..8usize,
    ) {
        let base_state = default_test_state();

        // Sequential execution.
        let sequential_result = execute_sequential(&base_state, &txs);

        // Parallel execution via Block-STM.
        let parallel_result = execute_block_stm(
            &base_state, &txs, num_threads,
        );

        // Final state must match exactly.
        prop_assert_eq!(
            sequential_result.final_state,
            parallel_result.final_state,
        );
        // Receipt ordering must match.
        prop_assert_eq!(
            sequential_result.receipts,
            parallel_result.receipts,
        );
    }
}
```

### 6.2 Property: Conflict detection catches read-write overlaps [SPEC]

**Statement:** If transaction T_i reads slot (A, S) and transaction T_j (where j > i) writes to (A, S), the MultiVersionStore detects this conflict. T_i is re-executed with T_j's written value.

### 6.3 Property: Version ordering respects transaction index [SPEC]

**Statement:** `MultiVersionStore::read(addr, slot, tx_index)` returns the value written by the highest-indexed transaction below `tx_index`. It never returns a value written by a later transaction.

```rust
proptest! {
    #[test]
    fn mvs_read_respects_ordering(
        values in prop::collection::vec(any::<U256>(), 5..20),
        reader_index in 0..20usize,
    ) {
        let store = MultiVersionStore::new();
        let addr = test_address();
        let slot = U256::from(0);

        // Write values at sequential tx indices.
        for (i, val) in values.iter().enumerate() {
            store.write(addr, slot, i, 0, *val);
        }

        let result = store.read(addr, slot, reader_index);
        if reader_index == 0 {
            // No prior writer.
            assert!(matches!(result, ReadResult::NotFound));
        } else {
            let expected_idx = reader_index.min(values.len()) - 1;
            match result {
                ReadResult::Found(val, writer_idx) => {
                    assert!(writer_idx < reader_index);
                    assert_eq!(val, values[writer_idx]);
                }
                ReadResult::NotFound if reader_index > values.len() => {
                    // Reader beyond last writer: should find the last write.
                    panic!("should have found the last writer");
                }
                _ => {}
            }
        }
    }
}
```

### 6.4 Property: Incarnation numbers increment on re-execution [SPEC]

**Statement:** When a transaction is re-executed after conflict detection, its incarnation number increments. The MultiVersionStore stores the latest incarnation only for each (tx_index, slot) pair.

### 6.5 Performance: Conflict rate on typical DeFi blocks [SPEC]

**Benchmark, not property test.** Replay 100 historical mainnet blocks through Block-STM. Measure the conflict rate (transactions requiring re-execution / total transactions). Expected: < 5% for typical blocks (Saraph & Herlihy, 2019). Flag if any block exceeds 20%.

---

## 7. Snapshot and Revert [SPEC]

### 7.1 Property: Revert restores exact prior state [SPEC]

**Statement:** After `snapshot() -> id`, followed by arbitrary writes, `revert(id)` restores the DirtyStore to the exact state at snapshot time. Every account, every slot, every watch list entry matches.

```rust
proptest! {
    #[test]
    fn snapshot_revert_roundtrip(
        initial_slots in prop::collection::hash_map(
            (any::<[u8; 20]>().prop_map(Address::from), any::<U256>()),
            any::<U256>(),
            1..50
        ),
        mutations in prop::collection::vec(
            (any::<[u8; 20]>().prop_map(Address::from), any::<U256>(), any::<U256>()),
            1..20
        ),
    ) {
        let mut store = DirtyStore::from_slots(initial_slots.clone());

        let snap_id = store.snapshot(100, 0);

        // Apply random mutations.
        for (addr, slot, val) in &mutations {
            store.accounts.entry(*addr).or_default()
                .storage.insert(*slot, *val);
        }

        // Revert.
        store.revert(snap_id).unwrap();

        // Verify state matches initial.
        for ((addr, slot), expected) in &initial_slots {
            let actual = store.accounts.get(addr)
                .and_then(|a| a.storage.get(slot).copied());
            prop_assert_eq!(actual, Some(*expected));
        }
    }
}
```

### 7.2 Property: Snapshot IDs are consumed on revert [SPEC]

**Statement:** Calling `revert(id)` a second time with the same ID returns `Err(SnapshotNotFound)`. Snapshots are single-use.

### 7.3 Property: Later snapshots are invalidated on revert [SPEC]

**Statement:** If snapshot A is taken before snapshot B, reverting to A removes snapshot B. Attempting to revert to B after reverting to A fails.

### 7.4 Property: Watch list is included in snapshots [SPEC]

**Statement:** Reverting restores the watch list and unwatch list to their snapshot-time state. Contracts added to the watch list after the snapshot are removed on revert.

---

## 8. BytecodeCache [SPEC]

### 8.1 Property: Bytecode cache entries never expire [SPEC]

**Statement:** Once bytecode is cached for a given code hash, it remains available indefinitely (LRU eviction aside). No TTL, no invalidation.

### 8.2 Property: Cache is shared across forks via Arc [SPEC]

**Statement:** Two HybridDB instances created with the same `Arc<BytecodeCache>` share cache entries. A bytecode fetched by one instance is immediately available to the other without an upstream call.

### 8.3 Property: LRU eviction respects capacity [SPEC]

**Statement:** The cache never holds more than `capacity` entries. When full, the least recently accessed entry is evicted.

---

## 9. Integration Test Scenarios [SPEC]

### 9.1 Full pipeline: local transaction -> classify -> watch -> replay [SPEC]

**Scenario:**
1. Start mirage-rs v2 connected to a mainnet archive node.
2. Execute a Uniswap v3 LP deposit via `eth_sendTransaction`.
3. Verify the pool contract enters the watch list (>= 3 slots written).
4. Wait for a new mainnet block containing a swap on the same pool.
5. Verify the TargetedFollower replays the swap.
6. Verify the golem's position reflects accrued fees from the replayed swap.

### 9.2 Scenario runner: parallel scenarios with CoW [SPEC]

**Scenario:**
1. Create a baseline state with an active LP position.
2. Define 4 scenarios: exit at 100%, 75%, 50%, 25% of position.
3. Run in parallel mode with CoW branches.
4. Verify each scenario produces different final balances.
5. Verify reverting to baseline after all scenarios restores original state.

### 9.3 Historical mode: pin block + replay range [SPEC]

**Scenario:**
1. Start mirage-rs in `--mode historical --from-block N --to-block N+100`.
2. Verify all upstream reads use block N (never `latest`).
3. Replay 100 blocks in `hybrid` mode.
4. Verify PnL output matches expected attribution.

---

## Test Infrastructure Requirements

| Component | Purpose |
|-----------|---------|
| `HybridDB::test_instance()` | Pre-configured HybridDB with mock upstream |
| `MockUpstreamRpc` | Returns configurable state, tracks call counts |
| `arb_transaction()` | proptest `Arbitrary` impl for test transactions |
| `DirtyStore::from_slots()` | Convenience constructor from a slot map |
| `default_test_state()` | Baseline state for Block-STM tests |

---

## Cross-references

- `16-testing/04-mirage.md` -- v1 mirage testing (Anvil-based fork infrastructure and regime scenario tests)
- `16-testing/12-simulation-validation.md` -- sim-to-real accuracy tests comparing mirage outputs against actual on-chain results
- Research: `02-mirage-rs/00-architecture.md` -- HybridDB three-tier read model, DirtyStore write isolation, CoW branching, and Block-STM parallel execution
- Research: `02-mirage-rs/02-scenario-runner.md` -- LHS parameter exploration, Bayesian optimization, and parallel scenario execution
- Research: `02-mirage-rs/03-historical-mode.md` -- historical block replay, shadow execution with modified bytecode, and PnL attribution

## References

- Gelashvili, R. et al. (2023). Block-STM: Scaling blockchain execution by turning ordering curse to a performance blessing. *PPoPP*. — *The parallel execution algorithm tested in Section 6; proves that optimistic concurrency with multi-version data structures achieves deterministic results matching sequential execution.*
- Saraph, V. & Herlihy, M. (2019). An empirical study of speculative concurrency in Ethereum smart contracts. *arXiv:1901.01376*. — *Measures real-world read/write conflict rates in Ethereum blocks; establishes that most transactions are parallelizable, motivating Block-STM adoption for historical replay.*

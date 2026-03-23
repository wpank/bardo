# ChainScope Attention Model Testing [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Companion to:** `14-chain/04-chain-scope.md`

> Tests for ChainScope's attention model: interest list convergence, position tracking latency, Binary Fuse filter false positive rates, Hebbian reinforcement dynamics, arousal modulation, and the cybernetic feedback loop between CorticalState and watched address scope.

> **Reader orientation:** This document specifies tests for ChainScope, the attention model that determines what on-chain activity a Golem (mortal autonomous agent) watches. It belongs to Section 16 (Testing) and covers interest list convergence, Binary Fuse filter false positive rates, Hebbian reinforcement dynamics, arousal modulation from the Daimon (the affect engine implementing PAD emotional state), and the cybernetic feedback loop between CorticalState (32-signal atomic shared perception surface) and watched address scope. See `prd2/shared/glossary.md` for full term definitions.

---

## Document Map

| Section | Topic |
|---------|-------|
| Overview | What ChainScope does and what testing covers |
| Interest List Convergence | Does the list stabilize under steady conditions? |
| Position Tracking Latency | Time from position change to scope update |
| Binary Fuse Filter | False positive rate, rebuild correctness |
| Hebbian Reinforcement | Address score strengthening dynamics |
| Arousal Modulation | Affect-to-attention link correctness |
| Decay Mechanics | Score decay, pruning, half-life modulation |
| HyperLogLog Cardinality | Estimation accuracy for universe_size |
| Gamma-Tick Rebuild | Full rebuild correctness under concurrent access |
| Property Test Sketches | Rust property tests for key invariants |

---

## Overview

ChainScope answers "what should the golem watch right now?" It maintains three interrelated data structures:

1. **Interest entries** -- scored address list, the source of truth. Rebuilt at every Gamma tick from CorticalState, strategy, positions, and Grimoire.
2. **BinaryFuse8 filter** -- probabilistic membership test for block-level pre-screening in `bardo-witness`. Atomically swapped at each Gamma tick.
3. **DashSet<Address>** -- precise lookup set for triage Stage 1. Rebuilt alongside the filter.

The cybernetic loop: CorticalState (arousal, active positions) drives interest entries, which drive the filter and set, which determine what transactions reach triage, which update CorticalState. Testing must verify that this loop converges to stable behavior under steady conditions and adapts appropriately when conditions change.

### Dependencies

- `14-chain/04-chain-scope.md` -- ChainScope specification
- `14-chain/02-triage.md` -- triage pipeline that consumes ChainScope output
- `16-testing/13-triage-evaluation.md` -- triage evaluation (companion)
- Research: `01-chain-intelligence/04-chain-scope.md` -- attention model design

---

## 1. Interest List Convergence [SPEC]

### 1.1 Property: List stabilizes under steady conditions [SPEC]

**Statement:** When the golem's positions, strategy, and CorticalState remain constant (no new positions, no arousal changes, no new triage events), the interest list converges to a fixed set of addresses within 10 Gamma ticks. Convergence means: the set of addresses with score > 0.05 does not change between consecutive rebuilds.

**Test approach:** Set up a golem with 3 active LP positions. Run 50 Gamma tick rebuilds with no external events. After tick 10, the address set should be identical across consecutive ticks.

```rust
#[test]
fn interest_list_converges_under_steady_state() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let cortical = mock_cortical_state(3 /* positions */, 0.5 /* arousal */);
    let grimoire = mock_grimoire(&[]);

    let mut prev_addrs: Option<Vec<Address>> = None;
    let mut stable_since: Option<usize> = None;

    for tick in 0..50 {
        scope.rebuild(&cortical, &grimoire).await;

        let current: Vec<Address> = scope.interest_entries.read().unwrap()
            .iter()
            .filter(|e| e.score > 0.05)
            .map(|e| e.address)
            .collect();

        if let Some(ref prev) = prev_addrs {
            if *prev == current {
                if stable_since.is_none() {
                    stable_since = Some(tick);
                }
            } else {
                stable_since = None;
            }
        }
        prev_addrs = Some(current);
    }

    let converged_at = stable_since.expect("list never converged");
    assert!(
        converged_at <= 10,
        "converged at tick {} (expected <= 10)",
        converged_at,
    );
}
```

### 1.2 Property: New position appears within 1 Gamma tick [SPEC]

**Statement:** When a new LP position is added to CorticalState, the corresponding contract address and pool address appear in `watched_addresses` after the next Gamma tick rebuild. Latency: exactly 1 Gamma tick (not 0, because the rebuild happens at tick boundaries).

```rust
#[test]
fn new_position_appears_within_one_tick() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let mut cortical = mock_cortical_state(0, 0.5);
    let grimoire = mock_grimoire(&[]);

    // Initial rebuild: no positions.
    scope.rebuild(&cortical, &grimoire).await;
    assert!(scope.watched_addresses.is_empty());

    // Add a position.
    let pool_addr = address!("0xpool...");
    let position_addr = address!("0xposition...");
    cortical.add_position(position_addr, Some(pool_addr));

    // One rebuild.
    scope.rebuild(&cortical, &grimoire).await;

    assert!(scope.watched_addresses.contains(&position_addr));
    assert!(scope.watched_addresses.contains(&pool_addr));
}
```

### 1.3 Property: Position tracking within 2 Gamma ticks [SPEC]

**Statement:** When a position changes (e.g., liquidity range adjusted, partial exit), the interest list reflects the updated position state within 2 Gamma ticks. This is the spec requirement from the task definition.

**Test approach:** Modify a position's `relevant_topics()` between two rebuilds. Verify the filter includes the new topics after at most 2 rebuilds.

```rust
#[test]
fn position_change_reflected_within_2_ticks() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let mut cortical = mock_cortical_state(1, 0.5);
    let grimoire = mock_grimoire(&[]);

    // Initial rebuild with original position.
    scope.rebuild(&cortical, &grimoire).await;

    // Change the position's relevant topics (simulating a range adjustment).
    let new_topic = B256::from([0xAB; 32]);
    cortical.positions[0].add_topic(new_topic);

    // First rebuild after change.
    scope.rebuild(&cortical, &grimoire).await;

    // Check if the new topic is in the filter.
    let topic_hash = xxh3_64(new_topic.as_slice());
    let in_filter = scope.watch_filter.load().contains(&topic_hash);

    if !in_filter {
        // Second rebuild (allowed: spec says within 2 ticks).
        scope.rebuild(&cortical, &grimoire).await;
        let in_filter_2 = scope.watch_filter.load().contains(&topic_hash);
        assert!(in_filter_2, "new topic not in filter after 2 Gamma ticks");
    }
}
```

### 1.4 Property: Removed position decays out [SPEC]

**Statement:** When a position is closed, its contract address is no longer seeded with score 1.0. The address decays according to its InterestReason and eventually falls below the 0.05 prune threshold.

**Test:** Remove a position from CorticalState. Run rebuilds. The address should drop below 0.05 within a predictable number of ticks based on its decay half-life.

---

## 2. Binary Fuse Filter Testing [SPEC]

### 2.1 Property: False positive rate below configured threshold [SPEC]

**Statement:** The BinaryFuse8 filter's false positive rate on random addresses not in the interest list is below 1/256 (approximately 0.39%), the theoretical bound for BinaryFuse8 (Lemire et al., 2022).

```rust
proptest! {
    #[test]
    fn binary_fuse_false_positive_rate(
        // Generate a set of "in" addresses and a set of "out" addresses.
        in_addrs in prop::collection::hash_set(
            any::<[u8; 20]>().prop_map(Address::from),
            100..1000
        ),
        out_addrs in prop::collection::hash_set(
            any::<[u8; 20]>().prop_map(Address::from),
            10000..20000
        ),
    ) {
        // Build filter from "in" addresses.
        let keys: Vec<u64> = in_addrs.iter()
            .map(|a| xxh3_64(a.as_slice()))
            .collect();
        let filter = BinaryFuse8::try_from(&keys).unwrap();

        // All "in" addresses must be present (zero false negatives).
        for addr in &in_addrs {
            let hash = xxh3_64(addr.as_slice());
            prop_assert!(filter.contains(&hash), "false negative");
        }

        // Measure false positive rate on "out" addresses.
        let false_positives = out_addrs.iter()
            .filter(|a| !in_addrs.contains(a))
            .filter(|a| filter.contains(&xxh3_64(a.as_slice())))
            .count();
        let total_out = out_addrs.len() - out_addrs.intersection(&in_addrs).count();
        let fp_rate = false_positives as f64 / total_out as f64;

        // BinaryFuse8 theoretical FP rate: ~1/256 = 0.0039
        // Allow 2x margin for statistical variation.
        prop_assert!(
            fp_rate < 0.008,
            "false positive rate {:.4} exceeds 2x theoretical bound",
            fp_rate,
        );
    }
}
```

### 2.2 Property: Zero false negatives [SPEC]

**Statement:** Every address in the interest list must be present in the filter. False negative rate = 0%. This is a structural property of Binary Fuse filters (they are exact for members of the construction set).

### 2.3 Property: Filter rebuild is atomic [SPEC]

**Statement:** The `arc_swap::ArcSwap` ensures that in-flight reads on the old filter complete without interruption while the new filter is being constructed. No thread ever sees a partially constructed filter.

**Test approach:** Spawn 10 reader threads that continuously query the filter. Spawn 1 writer thread that rebuilds every 10ms. Run for 1 second. Readers should never panic or see inconsistent results.

```rust
#[test]
fn filter_atomic_swap_under_contention() {
    let scope = Arc::new(ChainScope::new(ChainScopeConfig::default()));
    let known_addr = address!("0xknown...");

    // Initial build with known_addr.
    let keys = vec![xxh3_64(known_addr.as_slice())];
    let filter = BinaryFuse8::try_from(&keys).unwrap();
    scope.watch_filter.store(Arc::new(filter));

    let barrier = Arc::new(std::sync::Barrier::new(11));
    let stop = Arc::new(AtomicBool::new(false));

    // 10 reader threads.
    let readers: Vec<_> = (0..10).map(|_| {
        let scope = scope.clone();
        let barrier = barrier.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let mut checks = 0u64;
            while !stop.load(Ordering::Relaxed) {
                let filter = scope.watch_filter.load();
                // known_addr should always be present
                // (rebuilds always include it).
                let hash = xxh3_64(known_addr.as_slice());
                assert!(filter.contains(&hash));
                checks += 1;
            }
            checks
        })
    }).collect();

    // 1 writer thread.
    let writer = {
        let scope = scope.clone();
        let barrier = barrier.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let mut rebuilds = 0u64;
            while !stop.load(Ordering::Relaxed) {
                let keys = vec![xxh3_64(known_addr.as_slice())];
                let filter = BinaryFuse8::try_from(&keys).unwrap();
                scope.watch_filter.store(Arc::new(filter));
                rebuilds += 1;
                std::thread::sleep(Duration::from_millis(10));
            }
            rebuilds
        })
    };

    barrier.wait();
    std::thread::sleep(Duration::from_secs(1));
    stop.store(true, Ordering::Relaxed);

    let total_checks: u64 = readers.into_iter()
        .map(|h| h.join().unwrap())
        .sum();
    let total_rebuilds = writer.join().unwrap();

    // If we got here without panicking, atomicity held.
    println!("{} checks across {} rebuilds", total_checks, total_rebuilds);
    assert!(total_checks > 0);
    assert!(total_rebuilds > 0);
}
```

### 2.4 Filter includes event topics [SPEC]

**Statement:** For interest entries with non-empty `event_topics`, the topics are hashed and included in the filter alongside the address hash. `bardo-witness` uses this to pre-screen entire blocks by both address and topic.

**Test:** Build a filter from entries with known topics. Verify topic hashes are contained.

---

## 3. Hebbian Reinforcement [SPEC]

### 3.1 Property: Reinforcement increases score [SPEC]

**Statement:** Calling `ChainScope::reinforce(addr, event_score)` where `event_score > 0` increases the address's score (capped at 1.0) and updates `last_active`.

```rust
proptest! {
    #[test]
    fn reinforce_increases_score(
        initial_score in 0.1f32..0.9,
        event_score in 0.1f32..1.0,
    ) {
        let scope = ChainScope::new(ChainScopeConfig::default());

        // Seed an entry with known score.
        let addr = address!("0xtest...");
        {
            let mut entries = scope.interest_entries.write().unwrap();
            entries.push(InterestEntry {
                address: addr,
                score: initial_score,
                reason: InterestReason::HighCuriosity,
                decay_half_life: Duration::from_secs(3 * 3600),
                last_active: 0,
                hit_count: 0,
                ..Default::default()
            });
        }

        scope.reinforce(addr, event_score);

        let entries = scope.interest_entries.read().unwrap();
        let entry = entries.iter().find(|e| e.address == addr).unwrap();

        prop_assert!(entry.score >= initial_score);
        prop_assert!(entry.score <= 1.0);
        prop_assert!(entry.hit_count == 1);
        prop_assert!(entry.last_active > 0);
    }
}
```

### 3.2 Property: Frequently co-occurring addresses strengthen as expected [SPEC]

**Statement:** An address reinforced K times has a longer effective decay half-life than an address reinforced once. Specifically, `decay_half_life = base_hl * (1.0 + ln(hit_count))`.

```rust
#[test]
fn frequent_reinforcement_extends_half_life() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let addr = address!("0xfrequent...");

    // Seed entry.
    {
        let mut entries = scope.interest_entries.write().unwrap();
        entries.push(InterestEntry {
            address: addr,
            score: 0.5,
            reason: InterestReason::HighCuriosity,
            decay_half_life: Duration::from_secs(3 * 3600),
            hit_count: 0,
            ..Default::default()
        });
    }

    let base_hl = Duration::from_secs(3 * 3600);

    // Reinforce 20 times.
    for _ in 0..20 {
        scope.reinforce(addr, 0.3);
    }

    let entries = scope.interest_entries.read().unwrap();
    let entry = entries.iter().find(|e| e.address == addr).unwrap();

    assert_eq!(entry.hit_count, 20);

    let expected_hl = base_hl.mul_f32(1.0 + (20.0f32).ln());
    let actual_hl = entry.decay_half_life;

    // Allow 1% tolerance for floating point.
    let ratio = actual_hl.as_secs_f64() / expected_hl.as_secs_f64();
    assert!(
        (0.99..=1.01).contains(&ratio),
        "half-life ratio {:.4} not within 1%",
        ratio,
    );
}
```

### 3.3 Property: Reinforcement of unknown address creates a Discovered entry [SPEC]

**Statement:** Calling `reinforce()` on an address not in the interest list creates a new entry with `InterestReason::HighCuriosity`, score capped at 0.5, and `hit_count = 1`.

```rust
#[test]
fn reinforce_unknown_creates_entry() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let addr = address!("0xnew...");

    // No entry exists.
    assert!(scope.interest_entries.read().unwrap().is_empty());

    scope.reinforce(addr, 0.8);

    let entries = scope.interest_entries.read().unwrap();
    let entry = entries.iter().find(|e| e.address == addr).unwrap();

    assert!(matches!(entry.reason, InterestReason::HighCuriosity));
    assert_eq!(entry.score, 0.5); // capped
    assert_eq!(entry.hit_count, 1);
}
```

---

## 4. Arousal Modulation [SPEC]

### 4.1 Property: High arousal slows decay [SPEC]

**Statement:** `modulated_half_life(base, arousal=1.0) = base * 4.0`. At maximum arousal, addresses decay 4x slower, keeping more addresses in scope.

```rust
proptest! {
    #[test]
    fn arousal_modulates_half_life(
        base_secs in 60u64..86400,
        arousal in 0.0f32..1.0,
    ) {
        let base = Duration::from_secs(base_secs);
        let modulated = modulated_half_life(base, arousal);

        let expected_multiplier = 1.0 + arousal * 3.0;
        let expected = base.mul_f32(expected_multiplier);

        let ratio = modulated.as_secs_f64() / expected.as_secs_f64();
        prop_assert!(
            (0.99..=1.01).contains(&ratio),
            "modulated/expected ratio {:.4} not within 1%",
            ratio,
        );
    }
}
```

### 4.2 Property: Zero arousal uses base half-life [SPEC]

**Statement:** `modulated_half_life(base, 0.0) = base * 1.0`. No arousal amplification at resting state.

### 4.3 Behavioral test: Crisis broadens scope [SPEC]

**Test:** Start with arousal = 0.2, 100 interest entries. Set arousal to 0.9 (crisis). Run 20 Gamma ticks. Count surviving entries at each tick.

**Expected:** The address count at arousal 0.9 is higher than at arousal 0.2 after 20 ticks, because slower decay retains more addresses.

```rust
#[test]
fn high_arousal_retains_more_addresses() {
    let scope = ChainScope::new(ChainScopeConfig::default());

    // Seed 100 HighCuriosity entries with short half-lives.
    seed_curiosity_entries(&scope, 100, Duration::from_secs(600));

    // Low arousal run.
    let cortical_low = mock_cortical_state(0, 0.2);
    let grimoire = mock_grimoire(&[]);
    for _ in 0..20 {
        scope.rebuild(&cortical_low, &grimoire).await;
    }
    let low_count = scope.interest_entries.read().unwrap()
        .iter()
        .filter(|e| e.score > 0.05)
        .count();

    // Reset.
    seed_curiosity_entries(&scope, 100, Duration::from_secs(600));

    // High arousal run.
    let cortical_high = mock_cortical_state(0, 0.9);
    for _ in 0..20 {
        scope.rebuild(&cortical_high, &grimoire).await;
    }
    let high_count = scope.interest_entries.read().unwrap()
        .iter()
        .filter(|e| e.score > 0.05)
        .count();

    assert!(
        high_count > low_count,
        "high arousal should retain more: high={} low={}",
        high_count, low_count,
    );
}
```

---

## 5. Decay and Pruning [SPEC]

### 5.1 Property: Scores below 0.05 are pruned [SPEC]

**Statement:** After a Gamma tick rebuild, no entry in the interest list has `score <= 0.05`. All such entries are removed.

```rust
#[test]
fn low_scores_pruned() {
    let scope = ChainScope::new(ChainScopeConfig::default());

    // Seed entries with very low scores.
    {
        let mut entries = scope.interest_entries.write().unwrap();
        for i in 0..10 {
            entries.push(InterestEntry {
                address: Address::from_word(B256::from(U256::from(i))),
                score: 0.01 * (i as f32 + 1.0), // 0.01 to 0.10
                reason: InterestReason::Discovered,
                last_active: unix_ms() - 100_000, // stale
                decay_half_life: Duration::from_secs(1), // fast decay
                ..Default::default()
            });
        }
    }

    let cortical = mock_cortical_state(0, 0.5);
    let grimoire = mock_grimoire(&[]);
    scope.rebuild(&cortical, &grimoire).await;

    let entries = scope.interest_entries.read().unwrap();
    for entry in entries.iter() {
        assert!(entry.score > 0.05, "entry with score {:.3} not pruned", entry.score);
    }
}
```

### 5.2 Property: ActivePosition entries never decay [SPEC]

**Statement:** Entries with `InterestReason::ActivePosition` have `decay_half_life = Duration::MAX` and score = 1.0 at every rebuild, as long as the position remains in CorticalState.

### 5.3 Property: Exponential decay formula is correct [SPEC]

**Statement:** `decayed_score = score * 0.5^(elapsed / half_life)`. After exactly one half-life, the score is halved.

```rust
proptest! {
    #[test]
    fn exponential_decay_correct(
        initial_score in 0.1f32..1.0,
        half_life_secs in 60u64..86400,
    ) {
        let half_life = Duration::from_secs(half_life_secs);
        let elapsed = half_life; // exactly one half-life

        let decayed = initial_score
            * 0.5_f32.powf(elapsed.as_secs_f32() / half_life.as_secs_f32());

        let expected = initial_score * 0.5;
        let diff = (decayed - expected).abs();
        prop_assert!(diff < 1e-6, "decay error: {}", diff);
    }
}
```

---

## 6. HyperLogLog Cardinality Estimation [SPEC]

### 6.1 Property: Estimate within 0.8% error [SPEC]

**Statement:** HyperLogLog with precision b=14 provides estimates with standard error ~0.8%. For N unique addresses, the estimated count satisfies `|estimate - N| / N < 0.016` (2 standard errors) with probability > 0.95.

```rust
proptest! {
    #[test]
    fn hyperloglog_accuracy(
        n_unique in 1000usize..100_000,
    ) {
        let mut hll: HyperLogLog<14> = HyperLogLog::new();

        // Insert n_unique distinct addresses.
        for i in 0..n_unique {
            let addr = Address::from_word(B256::from(U256::from(i)));
            hll.insert(addr.as_slice());
        }

        let estimate = hll.count() as f64;
        let error = (estimate - n_unique as f64).abs() / n_unique as f64;

        // 2 standard errors at b=14: ~1.6%
        prop_assert!(
            error < 0.016,
            "HLL error {:.4} exceeds 1.6% for N={}",
            error, n_unique,
        );
    }
}
```

### 6.2 Property: Duplicate insertions don't inflate count [SPEC]

**Statement:** Inserting the same address K times produces the same estimate as inserting it once. HyperLogLog counts unique elements, not total insertions.

### 6.3 Property: Reset clears all state [SPEC]

**Statement:** After reset, `count()` returns 0. The sketch is equivalent to a newly constructed one.

---

## 7. Max Watch Size Cap [SPEC]

### 7.1 Property: Interest list never exceeds max_watch_size [SPEC]

**Statement:** After any rebuild, `interest_entries.len() <= max_watch_size`. Entries are sorted by score descending and truncated.

```rust
proptest! {
    #[test]
    fn interest_list_respects_cap(
        n_positions in 0usize..5,
        n_curiosity in 0usize..20000,
        max_watch in 100usize..10000,
    ) {
        let mut config = ChainScopeConfig::default();
        config.max_watch_size = max_watch;
        let scope = ChainScope::new(config);

        let cortical = mock_cortical_state(n_positions, 0.5);
        seed_curiosity_entries(&scope, n_curiosity, Duration::from_secs(3600));
        let grimoire = mock_grimoire(&[]);

        scope.rebuild(&cortical, &grimoire).await;

        let entries = scope.interest_entries.read().unwrap();
        prop_assert!(
            entries.len() <= max_watch,
            "entries {} > cap {}",
            entries.len(), max_watch,
        );
    }
}
```

### 7.2 Property: Truncation preserves highest-scoring entries [SPEC]

**Statement:** When truncation occurs, the retained entries are those with the highest scores. No entry with score S1 is retained while an entry with S2 > S1 is discarded.

### 7.3 Property: Active positions are never truncated [SPEC]

**Statement:** Even when the interest list exceeds `max_watch_size`, entries with `InterestReason::ActivePosition` are always retained (they have score 1.0, the maximum).

---

## 8. CorticalState Integration [SPEC]

### 8.1 Property: active_count reflects high-scoring entries [SPEC]

**Statement:** After rebuild, `CorticalState.active_count` equals the number of interest entries with `score > 0.5`.

```rust
#[test]
fn active_count_matches_high_score_entries() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let cortical = mock_cortical_state(3, 0.5);
    let grimoire = mock_grimoire(&[]);

    seed_curiosity_entries(&scope, 50, Duration::from_secs(3600));
    scope.rebuild(&cortical, &grimoire).await;

    let expected = scope.interest_entries.read().unwrap()
        .iter()
        .filter(|e| e.score > 0.5)
        .count() as u16;

    let actual = cortical.active_count.load(Ordering::Relaxed);
    assert_eq!(actual, expected);
}
```

### 8.2 Property: GolemEvent::ChainScopeAdjusted emitted on rebuild [SPEC]

**Statement:** Each rebuild emits a `ChainScopeAdjusted` event with the correct `interest_count` and `bloom_capacity`.

---

## 9. Cybernetic Loop Integration Test [SPEC]

### 9.1 Full loop: position -> scope -> triage -> reinforce -> scope [SPEC]

End-to-end test of the feedback loop:

1. **Setup:** Golem has 1 LP position in a Uniswap v3 ETH/USDC pool.
2. **Gamma tick 1:** ChainScope seeds the pool and position addresses. Filter includes both.
3. **Block arrives:** Contains a swap on the ETH/USDC pool. `bardo-witness` passes it through the filter. Triage processes it.
4. **Triage scores > 0.5:** The swap is relevant to the golem's position. Triage calls `scope.reinforce()` on the counterparty address.
5. **Gamma tick 2:** Rebuild includes the reinforced counterparty address. Filter now includes it.
6. **Next block:** A transaction from the counterparty (different pool) passes the filter. Triage evaluates it.

**Verification:** The counterparty address entered scope via Hebbian reinforcement, not via position seeding. The feedback loop expanded the golem's attention based on observed relevance.

```rust
#[tokio::test]
async fn cybernetic_loop_integration() {
    let scope = ChainScope::new(ChainScopeConfig::default());
    let mut cortical = mock_cortical_state(0, 0.5);
    let grimoire = mock_grimoire(&[]);

    let pool = address!("0xpool...");
    let position = address!("0xposition...");
    let counterparty = address!("0xcounterparty...");

    cortical.add_position(position, Some(pool));

    // Tick 1: pool and position in scope.
    scope.rebuild(&cortical, &grimoire).await;
    assert!(scope.watched_addresses.contains(&pool));
    assert!(!scope.watched_addresses.contains(&counterparty));

    // Triage finds a relevant swap, reinforces counterparty.
    scope.reinforce(counterparty, 0.6);

    // Tick 2: counterparty now in scope.
    scope.rebuild(&cortical, &grimoire).await;
    assert!(
        scope.watched_addresses.contains(&counterparty),
        "counterparty should be in scope after reinforcement"
    );
}
```

---

## Test Infrastructure Requirements

| Component | Purpose |
|-----------|---------|
| `mock_cortical_state(n_positions, arousal)` | CorticalState with configurable positions and arousal |
| `mock_grimoire(watched_addrs)` | GrimoireHandle that returns given watched addresses |
| `seed_curiosity_entries(scope, n, half_life)` | Populate interest entries with HighCuriosity entries |
| `ChainScopeConfig::default()` | Standard config with `max_watch_size = 10_000` |

---

## Cross-references

- `14-chain/04-chain-scope.md` -- ChainScope specification (interest entries, filter construction, arousal modulation)
- `14-chain/02-triage.md` -- four-stage triage pipeline that consumes ChainScope's watched address set
- `16-testing/13-triage-evaluation.md` -- companion triage testing doc (precision/recall on labeled datasets, Bayesian surprise calibration)
- `01-golem/18-cortical-state.md` -- CorticalState specification (32-signal atomic shared perception surface that drives ChainScope rebuilds)
- Research: `01-chain-intelligence/04-chain-scope.md` -- attention model design (Hebbian reinforcement, decay mechanics, cybernetic feedback loop)

## References

- Feldman, H. & Friston, K. (2010). Attention, Uncertainty, and Free-Energy. *Frontiers in Human Neuroscience*. — *Free-energy framework where attention minimizes surprise; the theoretical basis for ChainScope's arousal-modulated scope widening under high uncertainty.*
- Flajolet, P. et al. (2007). HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm. *Discrete Mathematics and Theoretical Computer Science*. — *The probabilistic cardinality estimator used to track `universe_size` (total unique addresses seen) without storing them all.*
- Lemire, D. et al. (2022). Binary Fuse Filters: Fast and Smaller Than Xor Filters. *Journal of Experimental Algorithmics*. — *The probabilistic membership filter used for block-level pre-screening in bardo-witness; tested for false positive rate bounds in Section 2.*

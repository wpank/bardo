# Chain Scope [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `golem-chain-scope`

---

> **Reader orientation:** This document specifies ChainScope, the dynamic attention model in Bardo's chain intelligence layer (section 14). ChainScope determines what a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) watches on-chain at any given moment, rebuilding its filter at each Gamma tick from CorticalState (the 32-signal atomic shared perception surface; the Golem's real-time self-model). The key concept is the cybernetic loop: the Golem's positions, strategy, and accumulated experience autonomously determine scope, with no human address configuration needed. See `prd2/shared/glossary.md` for full term definitions.

ChainScope answers the question: "what should the golem watch right now?" Its output -- a `BinaryFuse8` filter and a `DashSet<Address>` -- feeds directly into `bardo-witness` (which blocks to fetch, see [01-witness.md](./01-witness.md)) and `bardo-triage` (which transactions are relevant, see [02-triage.md](./02-triage.md)). It rebuilds from `CorticalState` at every Gamma tick, closing the cybernetic loop: what the golem cares about determines what it watches; what it watches determines what events reach it; those events update `CorticalState`; `CorticalState` drives the next rebuild.

No human configuration of watched addresses is required. The golem's positions, strategy, and accumulated experience determine scope autonomously.

### Dirty Flag Optimization

The full rebuild (5 data source queries, O(n) scan, O(n log n) sort) runs every Gamma tick (~250ms) regardless of whether anything changed. The filter construction itself is cheap (~100 microseconds for 10K keys per Lemire et al. 2022), but the interest list recalculation dominates.

Add a dirty flag (generation counter) to the interest list. If no position changes, no strategy changes, no Grimoire updates, and no Hebbian reinforcements occurred since the last rebuild, skip steps 1-6 and only apply time-based decay (which can be batched). When the dirty flag is set, run the full rebuild. This eliminates 80-90% of rebuild cycles during quiet periods.

---

## Structure

```rust
use xorf::BinaryFuse8;
use arc_swap::ArcSwap;
use dashmap::DashSet;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ChainScope {
    /// Binary Fuse filter of watched addresses + event topics.
    /// Atomically swapped at each Gamma tick via arc-swap (zero downtime).
    /// Used by bardo-witness for O(1) block-level pre-screening.
    pub watch_filter: Arc<ArcSwap<BinaryFuse8>>,

    /// Precise address set. The filter has false positives;
    /// this is the authoritative lookup for bardo-triage Stage 1.
    pub watched_addresses: Arc<DashSet<Address>>,

    /// Scored interest entries. Sorted descending by score.
    /// Source of truth for what goes into watch_filter and watched_addresses.
    pub interest_entries: Arc<RwLock<Vec<InterestEntry>>>,

    /// HyperLogLog sketch estimating unique addresses seen in the last 24h.
    /// Feeds CorticalState.attention.universe_size. Reset each Delta tick.
    pub cardinality_sketch: Mutex<HyperLogLog<12>>,  // ~12KB (2^12 registers × 6 bits ≈ 3KB per counter, 4 counters), ~1.6% error

    /// Cap on watched_addresses size (performance guard).
    pub max_watch_size: usize,  // default: 10_000

    /// Generation counter. Incremented on any mutation (position change,
    /// strategy change, Grimoire update, Hebbian reinforcement).
    /// If unchanged since last rebuild, skip full recalculation.
    pub generation: AtomicU64,

    /// Generation at last completed rebuild.
    pub last_rebuild_generation: AtomicU64,
}
```

---

## InterestEntry

Each entry represents an address the golem has a reason to watch, scored by relevance with a configurable decay rate:

```rust
pub struct InterestEntry {
    pub address: Address,
    pub protocol: Option<ProtocolId>,
    pub reason: InterestReason,
    pub score: f32,               // 0.0-1.0
    pub last_active: u64,         // unix_ms of last block where this address appeared
    pub decay_half_life: Duration,
    pub event_topics: Vec<B256>,  // specific topics to watch for this address
    pub hit_count: u32,           // how many times this address appeared in relevant txs
}

pub enum InterestReason {
    /// Golem holds a position at this address.
    ActivePosition,
    /// Golem's current strategy names this address.
    StrategyTarget,
    /// This address appeared in a recent relevant tx.
    RecentCounterparty,
    /// Grimoire has entries tagging this address as interesting.
    GrimoireWatched,
    /// Triage scored a tx involving this address > 0.5 recently.
    HighCuriosity,
    /// Known factory / router / core contract.
    ProtocolContract,
    /// Appeared in a relevant tx but not yet classified.
    Discovered,
}
```

### Default Scoring

| Reason | Default score | Decay half-life | Notes |
|--------|---------------|-----------------|-------|
| `ActivePosition` | 1.0 | None | Never decays while position is open |
| `StrategyTarget` | 0.9 | None | Never decays while strategy active |
| `RecentCounterparty` | 0.8 | 24 hours | Who the golem traded with recently |
| `GrimoireWatched` | 0.6 | 7 days | Explicitly noted in Grimoire |
| `HighCuriosity` | 0.5 | 3 hours | Triage noticed something interesting here |
| `ProtocolContract` | 1.0 | None | Core protocol infrastructure |
| `Discovered` | 0.3 | 1 hour | Just appeared, not yet understood |

Scores below 0.05 are pruned from the interest list at each Gamma tick.

---

## Gamma-Tick Rebuild

The rebuild runs on every Gamma tick (~250ms) inside the heartbeat pipeline (see [05-heartbeat-integration.md](./05-heartbeat-integration.md)):

```rust
impl ChainScope {
    pub async fn rebuild(
        &self,
        cortical: &CorticalState,
        grimoire: &GrimoireHandle,
    ) {
        // Dirty flag check: skip full rebuild if nothing changed.
        let current_gen = self.generation.load(Ordering::Acquire);
        let last_gen = self.last_rebuild_generation.load(Ordering::Acquire);
        if current_gen == last_gen {
            // Only apply time-based decay to existing entries.
            self.apply_time_decay_only();
            return;
        }

        let mut new_entries: Vec<InterestEntry> = Vec::new();
        let arousal = cortical.arousal.load(Ordering::Relaxed) as f32 / 255.0;

        // 1. Seed from active positions (always score 1.0, no decay).
        for position in cortical.active_positions() {
            new_entries.push(InterestEntry {
                address: position.contract_address,
                reason: InterestReason::ActivePosition,
                score: 1.0,
                decay_half_life: Duration::MAX,
                event_topics: position.relevant_topics(),
                ..Default::default()
            });
            // Also watch the pool/vault/market the position is in.
            if let Some(pool) = position.pool_address {
                new_entries.push(InterestEntry {
                    address: pool,
                    reason: InterestReason::ProtocolContract,
                    score: 1.0,
                    decay_half_life: Duration::MAX,
                    ..Default::default()
                });
            }
        }

        // 2. Seed from strategy configuration.
        for target in cortical.strategy_targets() {
            new_entries.push(InterestEntry {
                address: target,
                reason: InterestReason::StrategyTarget,
                score: 0.9,
                decay_half_life: Duration::MAX,
                ..Default::default()
            });
        }

        // 3. Always include protocol factories and routers (permanent).
        for seed in SEED_FACTORY_ADDRESSES {
            new_entries.push(InterestEntry {
                address: *seed,
                reason: InterestReason::ProtocolContract,
                score: 1.0,
                decay_half_life: Duration::MAX,
                ..Default::default()
            });
        }

        // 4. Carry forward existing entries, applying arousal-modulated decay.
        let now_ms = unix_ms();
        for existing in self.interest_entries.read().unwrap().iter() {
            // Skip if already covered by seeded entry above.
            if new_entries.iter().any(|e| e.address == existing.address) {
                continue;
            }

            let modulated_hl = modulated_half_life(
                existing.decay_half_life,
                arousal,
            );
            let elapsed = Duration::from_millis(
                now_ms.saturating_sub(existing.last_active)
            );
            let decayed_score = existing.score
                * 0.5_f32.powf(
                    elapsed.as_secs_f32() / modulated_hl.as_secs_f32()
                );

            if decayed_score > 0.05 {
                new_entries.push(InterestEntry {
                    score: decayed_score,
                    ..existing.clone()
                });
            }
        }

        // 5. Grimoire-based additions (addresses tagged "watch").
        for addr in grimoire.watched_addresses().await {
            if !new_entries.iter().any(|e| e.address == addr) {
                new_entries.push(InterestEntry {
                    address: addr,
                    reason: InterestReason::GrimoireWatched,
                    score: 0.6,
                    decay_half_life: Duration::from_secs(7 * 24 * 3600),
                    ..Default::default()
                });
            }
        }

        // 6. Sort by score descending, cap at max_watch_size.
        new_entries.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap()
        });
        new_entries.truncate(self.max_watch_size);

        // 7. Rebuild BinaryFuse8 filter from entries.
        let keys: Vec<u64> = new_entries.iter()
            .flat_map(|e| {
                let addr_hash = xxh3_64(e.address.as_slice());
                let topic_hashes = e.event_topics.iter()
                    .map(|t| xxh3_64(t.as_slice()));
                std::iter::once(addr_hash).chain(topic_hashes)
            })
            .collect();
        let filter = BinaryFuse8::try_from(&keys)
            .expect("filter construction failed");

        // 8. Atomic swap (no downtime, existing in-flight reads are safe).
        self.watch_filter.store(Arc::new(filter));

        // 9. Rebuild DashSet.
        self.watched_addresses.clear();
        for entry in &new_entries {
            self.watched_addresses.insert(entry.address);
        }

        // 10. Store new entries.
        *self.interest_entries.write().unwrap() = new_entries.clone();

        // 11. Update CorticalState.
        let active_count = new_entries.iter()
            .filter(|e| e.score > 0.5)
            .count() as u16;
        cortical.active_count.store(active_count, Ordering::Relaxed);

        // 12. Mark generation as rebuilt.
        self.last_rebuild_generation.store(current_gen, Ordering::Release);
    }
}
```

---

## Arousal Modulation

The golem's arousal level from `CorticalState` modulates decay rates. This is the affect-to-attention link: a frightened golem watches more broadly, while a bored one narrows focus.

```rust
fn modulated_half_life(base: Duration, arousal: f32) -> Duration {
    // High arousal (crisis, exploit detected): slow decay dramatically
    //   -> more addresses stay in scope -> broader surveillance
    // Low arousal (quiet market): faster decay
    //   -> scope narrows to only what's directly relevant
    let multiplier = 1.0 + arousal * 3.0;  // [1.0x at arousal=0, 4.0x at arousal=1.0]
    base.mul_f32(multiplier)
}
```

This maps to precision weighting in the predictive processing framework (Feldman & Friston, 2010): arousal (from the Daimon, the affect engine implementing PAD emotional state as a control signal) modulates the precision (inverse variance) of prediction errors. High arousal means prediction errors are treated as more informative, so the agent attends more broadly. Low arousal means prediction errors are downweighted, and the agent narrows attention.

---

## Hebbian Reinforcement

Addresses don't just decay. They strengthen when they keep appearing in relevant transactions. This implements "neurons that fire together wire together" at the address level:

```rust
impl ChainScope {
    /// Called by triage when a relevant transaction involves an address.
    pub fn reinforce(&self, address: Address, event_score: f32) {
        let mut entries = self.interest_entries.write().unwrap();
        if let Some(existing) = entries.iter_mut().find(|e| e.address == address) {
            existing.last_active = unix_ms();
            existing.score = (existing.score + event_score * 0.1).min(1.0);
            existing.hit_count += 1;
            // Frequently relevant addresses decay slower.
            let base_hl = existing.reason.default_decay_half_life();
            existing.decay_half_life = base_hl.mul_f32(
                1.0 + (existing.hit_count as f32).ln()
            );
        } else {
            entries.push(InterestEntry {
                address,
                reason: InterestReason::HighCuriosity,
                score: event_score.min(0.5),
                last_active: unix_ms(),
                decay_half_life: Duration::from_secs(3 * 3600),
                hit_count: 1,
                ..Default::default()
            });
        }

        // Mark dirty so next Gamma tick runs full rebuild.
        self.generation.fetch_add(1, Ordering::Release);
    }
}
```

This means high-curiosity addresses enter scope within one Gamma tick of being detected by triage -- before the full Gamma rebuild.

---

## Cardinality Estimation

The HyperLogLog sketch estimates `attention.universe_size` -- the number of unique addresses seen in on-chain activity over the last 24 hours:

```rust
impl ChainScope {
    /// Called by bardo-triage for every tx it processes (even discarded ones).
    pub fn observe_address(&self, addr: &Address) {
        self.cardinality_sketch.lock().unwrap().insert(addr.as_slice());
    }

    /// Called at each Gamma tick.
    pub fn estimate_universe_size(&self) -> u32 {
        self.cardinality_sketch.lock().unwrap().count() as u32
    }
}
```

HyperLogLog (Flajolet et al., 2007) with precision parameter b=12 uses 2^12 = 4,096 registers x 6 bits ≈ 3KB per counter (4 counters = ~12KB total) for ~1.6% error on estimates up to 10^18. The sketch resets at each Delta tick.

This feeds into the `attention_breadth` TUI interpolating variable: `active_count / universe_size`. Narrow attention = focused; wide = scanning. Rendered as peripheral particle density in the TUI (see [18-interfaces/03-tui.md](../18-interfaces/03-tui.md)).

---

## Emitted GolemEvents

```rust
GolemEvent::ChainScopeAdjusted {
    chain_id: u64,
    interest_count: u32,   // total entries in interest list
    bloom_capacity: u32,   // filter capacity (tuning indicator)
},
```

See [06-events-signals.md](./06-events-signals.md) for wire format.

---

## Dependencies

```toml
[dependencies]
xorf = "0.11"
xxhash-rust = { version = "0.8", features = ["xxh3"] }
arc-swap = "1"
dashmap = "6"
hyperloglog-rs = "0.1"
tokio = { version = "1", features = ["sync"] }
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
```

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Five-crate overview and cybernetic feedback loop that ChainScope closes
- **Witness**: [01-witness.md](./01-witness.md) -- Downstream consumer of the BinaryFuse8 filter produced here; uses it for O(1) block pre-screening
- **Triage**: [02-triage.md](./02-triage.md) -- Sends Hebbian reinforcement signals back to ChainScope when addresses appear in relevant transactions
- **Protocol state**: [03-protocol-state.md](./03-protocol-state.md) -- Autonomous protocol discovery adds new addresses to the interest list
- **Heartbeat**: [05-heartbeat-integration.md](./05-heartbeat-integration.md) -- Gamma tick triggers ChainScope rebuild; details the dirty flag optimization
- **Events**: [06-events-signals.md](./06-events-signals.md) -- ChainScopeAdjusted GolemEvent variant definition and wire format
- **Oracle / Prediction engine**: [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md) -- Prediction error feeds as an additional scoring input for interest entries

---

## References

- Feldman, H. & Friston, K. (2010). Attention, Uncertainty, and Free-Energy. *Frontiers in Human Neuroscience*. -- *Links precision weighting to attention allocation; grounds the arousal-modulated decay rates used here.*
- Flajolet, P. et al. (2007). HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm. *Discrete Mathematics and Theoretical Computer Science*. -- *The cardinality estimation algorithm used for `attention.universe_size` at ~1.6% error in 12KB.*
- Friston, K. et al. (2010). The free-energy principle: a unified brain theory? *Nature Reviews Neuroscience*, 11(2), 127-138. -- *Theoretical basis for the active inference loop that ChainScope implements between perception and action.*
- Lemire, D. et al. (2022). Binary Fuse Filters: Fast and Smaller Than Xor Filters. *Journal of Experimental Algorithmics*. -- *The immutable probabilistic filter rebuilt from the interest list at each Gamma tick; 8.7 bits/entry, sub-1% FPR.*

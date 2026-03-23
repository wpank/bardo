# HDC Applications: Memory, Query, and Advanced [SPEC]

> **Document Type**: REF (normative) | **Parent**: `shared/hdc-vsa.md` | **Last Updated**: 2026-03-18
>
> Memory applications (episode compression, knowledge inheritance, controlled forgetting), query applications (strategy-regime lookup, protocol families, drift detection, regime classification), and advanced applications (regime attention, world models, multi-agent consensus, concept algebra). All types and operations from `shared/hdc-vsa.md` are prerequisites.

> **Reader orientation:** This document covers the memory, query, and advanced applications of HDC (Hyperdimensional Computing) within Bardo. It is a child of `shared/hdc-vsa.md` (foundations and algebra) and belongs to the `shared/` reference layer. Topics include episode compression (bundling Grimoire episodes into prototype hypervectors), knowledge inheritance across Golem (mortal autonomous agent) generations via holographic legacy bundles, controlled forgetting (demurrage), strategy-regime lookup, drift detection, and multi-agent consensus. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Episode compression via bundling

### 1.1 The problem

The Grimoire stores episodes as individual records with embeddings. Retrieval uses nearest-neighbor search. This fails for pattern queries ("what kind of transactions typically precede a gas spike?") and noisy queries during regime transitions.

### 1.2 The HDC solution

Bundle related episodes into prototype hypervectors. A "gas spike precursor" prototype is the majority-vote bundle of all episodes that preceded gas spikes. Compression ratio: 500 episodes x 1,280 bytes = 640 KB compresses to one 1,280-byte prototype at SNR = sqrt(10240/500) = 4.5.

### 1.3 Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};
use std::collections::HashMap;

/// Category-based episode compression.
/// Each category maintains a running bundle accumulator and a frozen
/// prototype hypervector for query-time use.
pub struct EpisodeCompressor {
    categories: HashMap<String, CategoryBundle>,
}

struct CategoryBundle {
    accumulator: BundleAccumulator,
    prototype: Option<Hypervector>,
    episode_count: usize,
}

impl EpisodeCompressor {
    pub fn new() -> Self {
        EpisodeCompressor {
            categories: HashMap::new(),
        }
    }

    /// Add an episode's hypervector to its category bundle.
    /// Called at Theta tick after the LLM assigns a category tag.
    /// O(D) per call.
    pub fn accumulate(
        &mut self,
        category: &str,
        episode_hv: &Hypervector,
        importance: f32,
    ) {
        let bundle = self
            .categories
            .entry(category.to_string())
            .or_insert_with(|| CategoryBundle {
                accumulator: BundleAccumulator::new(),
                prototype: None,
                episode_count: 0,
            });

        // Weight by importance: more significant episodes contribute
        // more votes to the prototype.
        let weight = (importance * 5.0).round().max(1.0) as i32;
        bundle.accumulator.add_weighted(episode_hv, weight);
        bundle.episode_count += 1;
        bundle.prototype = None; // invalidate cached prototype
    }

    /// Refresh prototypes for all categories. Called at Delta tick.
    /// O(D * num_categories).
    pub fn refresh_prototypes(&mut self) {
        for bundle in self.categories.values_mut() {
            if bundle.accumulator.count > 0 {
                bundle.prototype = Some(bundle.accumulator.finish());
            }
        }
    }

    /// Query: which category does this episode most resemble?
    /// Returns (category_name, similarity) pairs sorted by similarity.
    pub fn classify(&self, episode_hv: &Hypervector) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .categories
            .iter()
            .filter_map(|(name, bundle)| {
                bundle
                    .prototype
                    .as_ref()
                    .map(|proto| (name.clone(), episode_hv.similarity(proto)))
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Query: how similar is this episode to a specific category?
    pub fn similarity_to(
        &self,
        category: &str,
        episode_hv: &Hypervector,
    ) -> Option<f32> {
        self.categories.get(category).and_then(|bundle| {
            bundle
                .prototype
                .as_ref()
                .map(|proto| episode_hv.similarity(proto))
        })
    }

    /// Export all prototypes for persistence or transfer.
    /// Returns (category, prototype, episode_count) tuples.
    pub fn export_prototypes(&self) -> Vec<(String, Hypervector, usize)> {
        self.categories
            .iter()
            .filter_map(|(name, bundle)| {
                bundle
                    .prototype
                    .as_ref()
                    .map(|proto| (name.clone(), *proto, bundle.episode_count))
            })
            .collect()
    }
}
```

### 1.4 Integration with Grimoire

In `GrimoireClient::store_episode()`, after writing the episode to LanceDB, call `episode_compressor.accumulate(category_tag, &episode_hv, importance_score)`. In `delta_chain_consolidation()`, call `refresh_prototypes()`. The prototypes serve as a parallel retrieval path: when the LLM asks "what patterns of behavior typically precede liquidation cascades?", the system checks the `liquidation_precursor` prototype first (O(D) similarity check) before falling back to episode-level ANN search.

---

## 2. Knowledge inheritance across agent generations

### 2.1 The problem

When a Golem dies, it produces a death testament: a text summary and exported Grimoire entries. The successor must re-embed, re-index, and re-weight all inherited entries from scratch. The process is lossy (text summarization discards structure), bulky (thousands of episode exports), and fragile (corrupted subsets are entirely lost).

### 2.2 The holographic legacy bundle

At death, the Golem compresses its entire Grimoire into a single 1,280-byte legacy bundle:

```
legacy_hv = bundle(
    bind(episode_1_hv, insight_1_hv),
    bind(episode_2_hv, insight_2_hv),
    ...
    bind(episode_N_hv, insight_N_hv)
)
```

The successor receives this alongside the text testament. To query: encode a situation as a hypervector, unbind from the legacy bundle, compare against the insight codebook.

**Graceful degradation.** If 30% of the parent's episodes were noise, the bundle is slightly noisier but still functional. If transfer corrupts some bits, retrieval degrades proportionally, not catastrophically. Information is distributed across all 10,240 bits.

### 2.3 Multi-generational inheritance

Legacy bundles compound across generations with exponential decay weighting:

| Generations bundled | SNR | Retrieval quality |
|---|---|---|
| 5 | 45.3 | Individual generation patterns clearly distinguishable |
| 10 | 32.0 | Most patterns recoverable with minor noise |
| 50 | 14.3 | Aggregate trends preserved, fine details lost |
| 100 | 10.1 | Only dominant patterns survive |

With a decay rate of 0.85, generation distances of 1, 5, 10, 20 produce weights of 8, 4, 2, 1. The most recent parent has 8x the influence of an ancestor 20 generations back.

For long lineages, hierarchical bundling preserves more signal: bundle within epochs (e.g., 10 generations per epoch), then bundle the epoch-level prototypes. This produces a two-level hierarchy where recent generations contribute full signal and distant ancestors contribute aggregate patterns.

### 2.4 Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Compressed holographic summary of a Golem's lifetime Grimoire.
/// Size: 1,280 bytes regardless of how many episodes were compressed.
#[derive(Clone)]
pub struct LegacyBundle {
    pub bundle: Hypervector,
    pub episode_count: usize,
    pub generation: u32,
}

/// The accumulated wisdom of all ancestor Golems.
/// Each generation's legacy is bundled with exponential decay weighting.
pub struct AncestorMemory {
    ancestor_hv: Option<Hypervector>,
    accumulator: BundleAccumulator,
    generations_absorbed: u32,
    decay_rate: f32,
}

impl AncestorMemory {
    pub fn new(decay_rate: f32) -> Self {
        AncestorMemory {
            ancestor_hv: None,
            accumulator: BundleAccumulator::new(),
            generations_absorbed: 0,
            decay_rate,
        }
    }

    /// Absorb a parent's legacy bundle into the ancestor memory.
    /// Called once during child boot, for each available ancestor legacy.
    pub fn absorb_legacy(&mut self, legacy: &LegacyBundle, distance: u32) {
        let weight = (self.decay_rate.powi(distance as i32) * 10.0)
            .round()
            .max(1.0) as i32;
        self.accumulator.add_weighted(&legacy.bundle, weight);
        self.generations_absorbed += 1;
        self.ancestor_hv = None; // invalidate
    }

    /// Refresh the queryable ancestor vector.
    pub fn refresh(&mut self) {
        if self.accumulator.count > 0 {
            self.ancestor_hv = Some(self.accumulator.finish());
        }
    }

    /// Query the ancestor memory with a situation description.
    /// Returns similarity score -- higher means the ancestors encountered
    /// something like this situation.
    pub fn query_similarity(&self, situation_hv: &Hypervector) -> f32 {
        match &self.ancestor_hv {
            Some(hv) => situation_hv.similarity(hv),
            None => 0.5, // no ancestor data, neutral
        }
    }

    /// Query the ancestor memory and rank candidate insights by relevance.
    /// Unbinds the situation from the ancestor bundle and compares against
    /// an insight codebook.
    pub fn query_insights(
        &self,
        situation_hv: &Hypervector,
        candidate_insights: &[(String, Hypervector)],
    ) -> Vec<(String, f32)> {
        let ancestor = match &self.ancestor_hv {
            Some(hv) => hv,
            None => return vec![],
        };

        let unbound = ancestor.unbind(situation_hv);

        let mut hits: Vec<(String, f32)> = candidate_insights
            .iter()
            .map(|(name, hv)| (name.clone(), unbound.similarity(hv)))
            .filter(|(_, sim)| *sim > 0.52) // above noise threshold
            .collect();

        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        hits
    }
}

impl LegacyBundle {
    /// Compress the Grimoire into a legacy bundle at death time.
    /// Iterates all episode-insight pairs above the death threshold
    /// and bundles them into a single hypervector.
    pub fn compress(
        episodes: &[(Hypervector, Hypervector, f32)], // (episode_hv, insight_hv, importance)
        generation: u32,
    ) -> Self {
        let mut acc = BundleAccumulator::new();

        for (ep_hv, in_hv, importance) in episodes {
            let bound = ep_hv.bind(in_hv);
            let weight = (*importance * 5.0).round().max(1.0) as i32;
            acc.add_weighted(&bound, weight);
        }

        LegacyBundle {
            bundle: acc.finish(),
            episode_count: episodes.len(),
            generation,
        }
    }
}
```

### 2.5 Integration with the death protocol

In `golem-core`'s death protocol Phase III (Legacy):

1. Iterate all Grimoire episodes with confidence >= 0.4.
2. For each, compute the episode hypervector and its associated insight hypervector.
3. Call `LegacyBundle::compress()` with the full set.
4. Serialize the 1,280-byte result into the testament directory.
5. The text testament continues to be generated as before -- the legacy bundle supplements it.

On child boot:

1. Load all available `LegacyBundle` files from the testament directory.
2. Create an `AncestorMemory` and call `absorb_legacy()` for each, with generation distance computed from the metadata.
3. Call `refresh()`.
4. The ancestor memory is now queryable via `GrimoireClient::query_ancestor()`.

---

## 3. Controlled forgetting via vote decay

### 3.1 Forgetting via dimensionality-aware decay

At each Delta tick, the accumulator's per-bit vote counts are multiplied by a decay factor. This gradually erodes old episodes, with recent ones dominating the prototype.

| Decay rate | Half-life (Delta ticks) | Effective window |
|---|---|---|
| 0.99 | 69 ticks | Hours to days |
| 0.95 | 14 ticks | 1-2 hours |
| 0.90 | 7 ticks | 30-60 minutes |
| 0.80 | 3 ticks | 15-30 minutes |

Semantic knowledge (stable patterns) uses high decay (0.99). Tactical knowledge (current conditions) uses low decay (0.90).

The `BundleAccumulator::decay()` implementation is in `shared/hdc-vsa.md` section 6.2.

### 3.2 Quality gates for episode storage

Every episode must pass a quality gate before entering bundle accumulators. Three conditions, any sufficient:

1. **Outcome validation.** Associated trade was profitable or prediction was correct.
2. **Consistency check.** Episode is not anti-correlated (sim < 0.45) with existing category prototypes.
3. **Information value.** Episode has low similarity (< 0.55) to nearest prototype (it is novel).

```rust
/// Determines whether an episode should be absorbed into the
/// category bundle or quarantined for review.
pub fn quality_gate(
    episode_hv: &Hypervector,
    category_prototype: Option<&Hypervector>,
    outcome_positive: Option<bool>,
) -> EpisodeDisposition {
    // Condition 1: outcome validation
    if outcome_positive == Some(true) {
        return EpisodeDisposition::Accept;
    }

    match category_prototype {
        Some(proto) => {
            let sim = episode_hv.similarity(proto);

            // Condition 3: novel (low similarity to existing prototype)
            if sim < 0.55 {
                return EpisodeDisposition::Accept;
            }

            // Condition 2: contradicts established knowledge
            if sim < 0.45 {
                return EpisodeDisposition::Quarantine;
            }

            // Redundant with existing knowledge, no positive outcome
            if outcome_positive == Some(false) {
                return EpisodeDisposition::Reject;
            }

            // No outcome signal, not novel, not contradictory
            EpisodeDisposition::Accept
        }
        None => {
            // No prototype yet -- accept everything to build initial model
            EpisodeDisposition::Accept
        }
    }
}

pub enum EpisodeDisposition {
    Accept,
    Quarantine,
    Reject,
}
```

### 3.3 Semantic memory as bundled episodic traces

Complementary Learning Systems theory (McClelland, McNaughton, and O'Reilly 1995) describes two memory systems: the hippocampus for fast, one-shot episodic capture, and the neocortex for slow extraction of statistical regularities through interleaved replay. The neocortex does not receive literal copies -- it re-learns its own distributed representation capturing similarity structure.

This maps directly to the Golem's tick hierarchy:

- **Gamma tick (hippocampus):** Raw episodic capture of chain events. Pattern-separated storage -- each episode gets a distinct hypervector, minimizing interference.
- **Theta tick (prefrontal deliberation):** LLM uses both episodic and semantic memory for planning.
- **Delta tick (neocortical consolidation):** Semantic abstraction from accumulated episodes. Episode clusters are compressed into semantic entries. Low-utility episodes are candidates for pruning.

The episode compressor described above implements the consolidation step. But consolidation alone is not enough -- the system also needs controlled forgetting.

Naive add-all memory degrades agent performance (demonstrated empirically in recent work on experience-following in LLM agents, arXiv:2505.16067). If incorrect past executions are stored, they propagate through retrieval-replication-storage cascades. The Grimoire must implement selective retention, which the quality gates in section 3.2 provide.

### 3.4 Neubert's place cell analogy

Neubert et al. (2019) demonstrated that HDC bundling of visual features produces representations analogous to hippocampal place cells: bundled environmental observations form stable "place" vectors that enable loop closure detection in robotics. For the Golem, the analogy maps to "market place cells." A bundle of episodes from a specific market regime forms a regime place vector. When current conditions produce an observation bundle similar to a stored regime vector, the Golem recognizes the regime. The vote decay mechanism implements the computational equivalent of biological place cell remapping -- regime prototypes gradually shift as the market evolves.

---

## 4. Strategy-regime bidirectional lookup

### 4.1 The problem

Answering "what strategies perform well in volatile ETH regimes?" requires scanning episodes, filtering by regime, grouping by strategy, and aggregating. The inverse question requires a separate query path.

### 4.2 HDC solution: bidirectional associative bundle

Maintain a single bundle that stores `bind(strategy_hv, regime_hv)` pairs weighted by performance. The same bundle answers both directions: unbind the regime to recover strategies, or unbind the strategy to recover regimes.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Performance-weighted strategy x regime association bundle.
///
/// Stores: bundle(bind(strategy_hv_i, regime_hv_i)) weighted by normalized performance.
///
/// Two query directions from one data structure:
///   best_strategies_for(regime) -> unbind(bundle, regime_hv) -> rank strategy codebook
///   best_regimes_for(strategy) -> unbind(bundle, strategy_hv) -> rank regime codebook
pub struct StrategyRegimeBundle {
    bundle: BundleAccumulator,
    snapshot: Option<Hypervector>,
    item_memory: ItemMemory,
}

impl StrategyRegimeBundle {
    pub fn new(seed: u64) -> Self {
        StrategyRegimeBundle {
            bundle: BundleAccumulator::new(),
            snapshot: None,
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Record a strategy's performance in a regime.
    /// `performance_score` in [0.0, 1.0] (normalized Sharpe or PnL percentile).
    /// Called after each trade closes with outcome verification.
    pub fn record_outcome(
        &mut self,
        strategy_name: &str,
        regime_name: &str,
        performance_score: f32,
    ) {
        let s_hv = self.item_memory.encode(strategy_name);
        let r_hv = self.item_memory.encode(regime_name);
        let bound = s_hv.bind(&r_hv);

        // Weighted bundling: higher performance -> more votes.
        let weight = (performance_score * 10.0).round().max(1.0) as i32;
        self.bundle.add_weighted(&bound, weight);
        self.snapshot = None;
    }

    /// Refresh the queryable snapshot. Called each Delta tick.
    pub fn refresh_snapshot(&mut self) {
        if self.bundle.count > 0 {
            self.snapshot = Some(self.bundle.finish());
        }
    }

    /// Find the best strategies for a given regime.
    /// Returns strategy names ranked by similarity to the unbound result.
    pub fn best_strategies_for(
        &mut self,
        regime_name: &str,
        candidates: &[&str],
    ) -> Vec<(String, f32)> {
        let bundle = match &self.snapshot {
            Some(b) => *b,
            None => return vec![],
        };

        let r_hv = self.item_memory.encode(regime_name);
        let unbound = bundle.unbind(&r_hv);

        let mut scores: Vec<(String, f32)> = candidates
            .iter()
            .map(|&name| {
                let s_hv = self.item_memory.encode(name);
                (name.to_string(), unbound.similarity(&s_hv))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Find the best regimes for a given strategy. Inverse direction, same cost.
    pub fn best_regimes_for(
        &mut self,
        strategy_name: &str,
        candidates: &[&str],
    ) -> Vec<(String, f32)> {
        let bundle = match &self.snapshot {
            Some(b) => *b,
            None => return vec![],
        };

        let s_hv = self.item_memory.encode(strategy_name);
        let unbound = bundle.unbind(&s_hv);

        let mut scores: Vec<(String, f32)> = candidates
            .iter()
            .map(|&name| {
                let r_hv = self.item_memory.encode(name);
                (name.to_string(), unbound.similarity(&r_hv))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Apply decay to reduce the influence of old outcomes.
    /// Called at Delta tick before refresh_snapshot().
    pub fn decay(&mut self, factor: f32) {
        self.bundle.decay(factor);
        self.snapshot = None;
    }
}
```

### 4.3 Usage example

```rust
// "What strategy works best when ETH is volatile?"
let strategies = bundle.best_strategies_for(
    "volatile_eth",
    &["arb_uniswap", "lp_rebalance", "trend_follow", "mean_revert"],
);
// Result: [("arb_uniswap", 0.61), ("trend_follow", 0.57), ("mean_revert", 0.52), ...]

// Inverse: "Where does arb_uniswap excel?"
let regimes = bundle.best_regimes_for(
    "arb_uniswap",
    &["volatile_eth", "low_gas", "stable_sideways", "high_volume"],
);
// Result: [("volatile_eth", 0.63), ("high_volume", 0.58), ...]
```

Cost: two XOR operations + one linear scan. For 20 candidates: ~200 nanoseconds. Compare to scanning 10,000 Grimoire episodes: ~50ms.

---

## 5. Protocol family vectors

### 5.1 The problem

DeFi protocols cluster into families (AMM DEXes, lending protocols, bridges). The triage system needs family recognition to inherit curiosity profiles for new protocols and detect cross-family anomalies.

### 5.2 Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Protocol family management using HDC bundling.
/// Each family is a bundle of its member protocols' transaction prototypes.
pub struct ProtocolFamilyIndex {
    families: Vec<(String, Hypervector, Vec<String>)>, // (name, prototype, member_names)
    item_memory: ItemMemory,
}

impl ProtocolFamilyIndex {
    pub fn new(seed: u64) -> Self {
        ProtocolFamilyIndex {
            families: Vec::new(),
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Register a protocol family from known member protocols.
    /// The family prototype is the bundle of member name hypervectors.
    /// For a richer prototype, pass pre-computed transaction prototype HVs
    /// (bundled from real transaction fingerprints) instead of name-only HVs.
    pub fn register_family(
        &mut self,
        family_name: &str,
        member_names: &[&str],
    ) {
        let mut acc = BundleAccumulator::new();
        let names: Vec<String> = member_names.iter().map(|s| s.to_string()).collect();

        for name in member_names {
            let hv = self.item_memory.encode(&format!("proto:{}", name));
            acc.add(&hv);
        }

        self.families
            .push((family_name.to_string(), acc.finish(), names));
    }

    /// Register a family from pre-computed transaction prototype HVs.
    /// More accurate than name-only registration because the prototypes
    /// capture actual transaction structure.
    pub fn register_family_from_prototypes(
        &mut self,
        family_name: &str,
        member_prototypes: &[(&str, Hypervector)],
    ) {
        let mut acc = BundleAccumulator::new();
        let names: Vec<String> = member_prototypes
            .iter()
            .map(|(name, _)| name.to_string())
            .collect();

        for (_, proto) in member_prototypes {
            acc.add(proto);
        }

        self.families
            .push((family_name.to_string(), acc.finish(), names));
    }

    /// Classify a transaction by protocol family.
    /// Returns (family_name, similarity) pairs sorted by similarity.
    pub fn classify(&self, tx_hv: &Hypervector) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .families
            .iter()
            .map(|(name, proto, _)| (name.clone(), tx_hv.similarity(proto)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Measure the distance between two protocol families.
    /// Returns Hamming similarity in [0.0, 1.0].
    /// Values near 0.5 = unrelated. Above 0.55 = structurally similar.
    pub fn family_distance(&self, family_a: &str, family_b: &str) -> Option<f32> {
        let a = self.families.iter().find(|(n, _, _)| n == family_a);
        let b = self.families.iter().find(|(n, _, _)| n == family_b);
        match (a, b) {
            (Some((_, pa, _)), Some((_, pb, _))) => Some(pa.similarity(pb)),
            _ => None,
        }
    }

    /// Find the nearest family for an unknown protocol.
    /// Returns (family_name, similarity) or None if no families registered.
    pub fn nearest_family(&self, tx_hv: &Hypervector) -> Option<(String, f32)> {
        self.families
            .iter()
            .map(|(name, proto, _)| (name.clone(), tx_hv.similarity(proto)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
}
```

### 5.3 Integration with ChainScope

When ChainScope discovers a new contract (not in WatchBloom), the triage pipeline computes the transaction fingerprint and calls `protocol_family_index.nearest_family(&tx_hv)`. If the similarity exceeds 0.58, the new contract is provisionally classified into that family and inherits the family's curiosity profile. This is the "semantic interest map" -- extending WatchBloom's binary membership to graduated similarity.

The protocol family index works alongside the ANN search. The ANN index provides fine-grained nearest-neighbor retrieval; the family index provides coarse-grained categorical classification. Both are O(1) memory per family (one 1,280-byte prototype) vs. O(N) for the ANN index.

---

## 6. Drift detection via temporal window comparison

### 6.1 The problem

Market regimes shift. A curiosity model trained during low-volatility markets becomes unreliable when volatility spikes. The existing system detects drift by re-embedding 20 reference documents weekly -- expensive, batch-only, and sensitive to document selection.

### 6.2 HDC solution: centroid comparison across windows

Maintain a per-domain hypervector centroid tracking the statistical center of recent episodes. At each Delta tick, compare current centroid to previous window. Drift score = `1 - similarity(current, previous)`.

```rust
use crate::hdc::{Hypervector, BundleAccumulator};
use std::collections::HashMap;

/// Per-domain drift detection via running hypervector centroids.
/// Each domain (e.g., "swap_events", "lp_positions", "regime_shifts")
/// maintains its own centroid tracker.
pub struct DriftSentinel {
    domains: HashMap<String, DomainTracker>,
}

struct DomainTracker {
    current: BundleAccumulator,
    previous: Option<Hypervector>,
    two_ago: Option<Hypervector>,
    episode_count: usize,
}

pub struct DriftReport {
    /// 0.0 = no drift. 0.5 = complete replacement of semantic content.
    /// (Recall that random vectors have similarity 0.5, so a drift score
    /// of 0.5 means the centroid has shifted to an unrelated region.)
    pub drift_score: f32,
    /// Positive = drift is accelerating. Negative = stabilizing.
    pub trend: f32,
    pub episodes_this_window: usize,
}

pub enum DriftSeverity {
    Healthy,
    Warning,
    Critical,
}

impl DriftSentinel {
    pub fn new() -> Self {
        DriftSentinel {
            domains: HashMap::new(),
        }
    }

    /// Record an episode observation. Called at Theta tick.
    pub fn observe(&mut self, domain: &str, hv: &Hypervector) {
        let tracker = self
            .domains
            .entry(domain.to_string())
            .or_insert_with(|| DomainTracker {
                current: BundleAccumulator::new(),
                previous: None,
                two_ago: None,
                episode_count: 0,
            });
        tracker.current.add(hv);
        tracker.episode_count += 1;
    }

    /// Called each Delta tick. Rotates windows and returns drift reports
    /// for all domains with sufficient history (>= 10 episodes).
    pub fn delta_tick(&mut self) -> Vec<(String, DriftReport)> {
        self.domains
            .iter_mut()
            .filter_map(|(name, tracker)| {
                if tracker.current.count < 10 {
                    return None;
                }

                let current_hv = tracker.current.finish();

                let drift = tracker
                    .previous
                    .as_ref()
                    .map(|prev| 1.0 - current_hv.similarity(prev));

                let trend = tracker
                    .two_ago
                    .as_ref()
                    .zip(tracker.previous.as_ref())
                    .map(|(two, one)| {
                        let drift_prev = 1.0 - one.similarity(two);
                        let drift_curr = drift.unwrap_or(0.0);
                        drift_curr - drift_prev
                    });

                let report = DriftReport {
                    drift_score: drift.unwrap_or(0.0),
                    trend: trend.unwrap_or(0.0),
                    episodes_this_window: tracker.episode_count,
                };

                // Rotate windows
                tracker.two_ago = tracker.previous.take();
                tracker.previous = Some(current_hv);
                tracker.current = BundleAccumulator::new();
                tracker.episode_count = 0;

                Some((name.clone(), report))
            })
            .collect()
    }
}

impl DriftReport {
    pub fn severity(&self) -> DriftSeverity {
        match self.drift_score {
            x if x < 0.02 => DriftSeverity::Healthy,
            x if x < 0.05 => DriftSeverity::Warning,
            _ => DriftSeverity::Critical,
        }
    }
}
```

### 6.3 Integration

In `GrimoireClient::store_episode()`, after writing to the persistence layer, call `drift_sentinel.observe(episode.domain(), &episode_hv)`. In `delta_chain_consolidation()`, call `drift_sentinel.delta_tick()` and route `DriftSeverity::Critical` reports to the CorticalState's alerting pipeline. A critical drift triggers: (1) increased curiosity exploration weight, and (2) domain flagged for potential re-embedding.

Cost: O(D) per observation, O(D x num_domains) per Delta tick. For 10 domains: ~150 microseconds.

---

## 7. Market regime classification

### 7.1 The problem

The Golem needs to recognize market regimes: trending, ranging, volatile, calm, crisis. The current approach relies on explicit classification from CorticalState atomic signals with hand-tuned thresholds.

### 7.2 HDC solution: composite state matching

Each Gamma tick, encode the current market state as a composite hypervector. Maintain a library of regime prototypes. Classify by similarity to nearest prototype.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};
use std::collections::VecDeque;

/// Market regime classification via composite state hypervectors.
pub struct RegimeClassifier {
    /// Known regime prototypes: (name, prototype_hv, observation_count).
    regimes: Vec<(String, BundleAccumulator, Option<Hypervector>)>,
    /// Rolling history of recent state HVs for building new regimes.
    history: VecDeque<(u64, Hypervector)>,
    history_capacity: usize,
    item_memory: ItemMemory,
}

/// Discretized market state snapshot for HDC encoding.
pub struct MarketSnapshot {
    pub eth_volatility: &'static str,  // "low" | "medium" | "high" | "extreme"
    pub gas_level: &'static str,       // "low" | "normal" | "elevated" | "spike"
    pub dex_volume: &'static str,      // "quiet" | "normal" | "active" | "frenzy"
    pub trend_direction: &'static str, // "down_strong" | "down" | "flat" | "up" | "up_strong"
    pub lending_utilization: &'static str, // "low" | "normal" | "high" | "maxed"
}

impl RegimeClassifier {
    pub fn new(seed: u64, history_capacity: usize) -> Self {
        RegimeClassifier {
            regimes: Vec::new(),
            history: VecDeque::with_capacity(history_capacity),
            history_capacity,
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Encode a market snapshot into a composite hypervector.
    /// Binds each signal with its role, then bundles all pairs.
    pub fn encode_snapshot(&mut self, snap: &MarketSnapshot) -> Hypervector {
        let mut acc = BundleAccumulator::new();

        let pairs = [
            ("role:volatility", format!("vol:{}", snap.eth_volatility)),
            ("role:gas", format!("gas:{}", snap.gas_level)),
            ("role:volume", format!("volume:{}", snap.dex_volume)),
            ("role:trend", format!("trend:{}", snap.trend_direction)),
            ("role:lending", format!("lending:{}", snap.lending_utilization)),
        ];

        for (role, filler) in &pairs {
            let r_hv = self.item_memory.encode(role);
            let f_hv = self.item_memory.encode(filler);
            acc.add(&r_hv.bind(&f_hv));
        }

        acc.finish()
    }

    /// Record a state observation. Called each Gamma tick.
    pub fn observe(&mut self, tick: u64, state_hv: &Hypervector) {
        if self.history.len() == self.history_capacity {
            self.history.pop_front();
        }
        self.history.push_back((tick, *state_hv));
    }

    /// Register a named regime from labeled historical data.
    /// The prototype is the bundle of all state HVs associated with this regime.
    pub fn register_regime(
        &mut self,
        name: &str,
        labeled_states: &[Hypervector],
    ) {
        let mut acc = BundleAccumulator::new();
        for hv in labeled_states {
            acc.add(hv);
        }
        let proto = acc.finish();
        self.regimes
            .push((name.to_string(), acc, Some(proto)));
    }

    /// Add a single observation to an existing regime's accumulator.
    /// Called when the LLM labels a period as belonging to a specific regime.
    pub fn label_observation(
        &mut self,
        regime_name: &str,
        state_hv: &Hypervector,
    ) {
        if let Some((_, acc, proto)) = self
            .regimes
            .iter_mut()
            .find(|(n, _, _)| n == regime_name)
        {
            acc.add(state_hv);
            *proto = None; // invalidate
        }
    }

    /// Refresh all regime prototypes. Called at Delta tick.
    pub fn refresh(&mut self) {
        for (_, acc, proto) in self.regimes.iter_mut() {
            if acc.count > 0 {
                *proto = Some(acc.finish());
            }
        }
    }

    /// Classify the current state against known regimes.
    /// Returns (regime_name, similarity) pairs sorted descending.
    pub fn classify(&self, state_hv: &Hypervector) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .regimes
            .iter()
            .filter_map(|(name, _, proto)| {
                proto
                    .as_ref()
                    .map(|p| (name.clone(), state_hv.similarity(p)))
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Detect regime transition by comparing recent history window
    /// against older history window.
    /// Returns the drift score between the two windows.
    pub fn detect_transition(&self, window_size: usize) -> Option<f32> {
        if self.history.len() < window_size * 2 {
            return None;
        }

        let recent_start = self.history.len() - window_size;
        let older_start = recent_start - window_size;

        let mut recent_acc = BundleAccumulator::new();
        for (_, hv) in self.history.range(recent_start..) {
            recent_acc.add(hv);
        }

        let mut older_acc = BundleAccumulator::new();
        for (_, hv) in self.history.range(older_start..recent_start) {
            older_acc.add(hv);
        }

        let recent_proto = recent_acc.finish();
        let older_proto = older_acc.finish();

        Some(1.0 - recent_proto.similarity(&older_proto))
    }
}
```

### 7.3 How regime classification feeds the system

Regime classification connects to three downstream consumers:

1. **Strategy selection.** The strategy-regime bundle answers "what strategies work in the detected regime?" Chain: `regime_classifier.classify()` -> top regime -> `strategy_bundle.best_strategies_for(top_regime)` -> ranked strategy recommendations for Theta-tick LLM.

2. **Curiosity thresholds.** During regime transitions (high drift score), curiosity thresholds drop -- the system explores more because its model is stale. During stable regimes, thresholds rise -- the system exploits known patterns.

3. **Risk parameters.** Volatile regimes trigger tighter position limits and wider stop-losses. The regime classification feeds directly into the risk parameter table maintained by the CorticalState.

---

## 8. Regime-modulated attention weights

### 8.1 The problem

The curiosity scoring system applies the same weights to all scoring signals regardless of market conditions. But signal informativeness varies by regime. During high-volatility periods, prediction error spikes for everything. During stable periods, even small prediction errors are informative.

### 8.2 HDC solution: regime-conditional weight vectors

Store optimal attention weights for each regime as hypervector-encoded associations. When a regime is detected, retrieve its associated weight profile.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Attention weight profile for the curiosity scoring system.
/// Each signal gets a weight label that encodes its relative importance.
#[derive(Debug, Clone)]
pub struct AttentionProfile {
    pub heuristic_weight: f32,
    pub embedding_weight: f32,
    pub surprise_weight: f32,
    pub prediction_error_weight: f32,
    pub novelty_weight: f32,
}

impl AttentionProfile {
    fn to_labels(&self) -> Vec<(String, String)> {
        vec![
            ("signal:heuristic".into(), weight_bucket(self.heuristic_weight)),
            ("signal:embedding".into(), weight_bucket(self.embedding_weight)),
            ("signal:surprise".into(), weight_bucket(self.surprise_weight)),
            ("signal:pred_error".into(), weight_bucket(self.prediction_error_weight)),
            ("signal:novelty".into(), weight_bucket(self.novelty_weight)),
        ]
    }

    /// Decode an attention profile from a hypervector by finding the
    /// nearest weight bucket for each signal.
    fn from_hv(hv: &Hypervector, item_memory: &mut ItemMemory) -> Self {
        let signals = ["heuristic", "embedding", "surprise", "pred_error", "novelty"];
        let buckets = ["weight:zero", "weight:low", "weight:medium", "weight:high", "weight:dominant"];
        let bucket_values = [0.0, 0.15, 0.25, 0.35, 0.50];

        let mut weights = Vec::new();
        for signal in &signals {
            let role = item_memory.encode(&format!("signal:{}", signal));
            let unbound = hv.unbind(&role);

            let best_idx = buckets
                .iter()
                .enumerate()
                .map(|(i, &bucket)| {
                    let bucket_hv = item_memory.encode(bucket);
                    (i, unbound.similarity(&bucket_hv))
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(2); // default to medium

            weights.push(bucket_values[best_idx]);
        }

        // Normalize so weights sum to 1.0
        let total: f32 = weights.iter().sum();
        let norm = if total > 0.0 { 1.0 / total } else { 0.2 };

        AttentionProfile {
            heuristic_weight: weights[0] * norm,
            embedding_weight: weights[1] * norm,
            surprise_weight: weights[2] * norm,
            prediction_error_weight: weights[3] * norm,
            novelty_weight: weights[4] * norm,
        }
    }
}

fn weight_bucket(w: f32) -> String {
    match w {
        x if x < 0.05 => "weight:zero".into(),
        x if x < 0.15 => "weight:low".into(),
        x if x < 0.30 => "weight:medium".into(),
        x if x < 0.45 => "weight:high".into(),
        _ => "weight:dominant".into(),
    }
}

/// Associates regime prototypes with attention weight profiles.
/// Updated online as the Hedge algorithm discovers which weights
/// work best in each regime.
pub struct RegimeAttentionMap {
    bundle: BundleAccumulator,
    snapshot: Option<Hypervector>,
    item_memory: ItemMemory,
}

impl RegimeAttentionMap {
    pub fn new(seed: u64) -> Self {
        RegimeAttentionMap {
            bundle: BundleAccumulator::new(),
            snapshot: None,
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Record that a specific attention profile worked well in a regime.
    /// Called after Hedge weight update when the Golem evaluates
    /// curiosity scoring performance.
    pub fn record(
        &mut self,
        regime_hv: &Hypervector,
        profile: &AttentionProfile,
        effectiveness: f32,
    ) {
        let profile_hv = self.encode_profile(profile);
        let bound = regime_hv.bind(&profile_hv);
        let weight = (effectiveness * 10.0).round().max(1.0) as i32;
        self.bundle.add_weighted(&bound, weight);
        self.snapshot = None;
    }

    /// Retrieve the best attention profile for a detected regime.
    pub fn profile_for_regime(
        &mut self,
        regime_hv: &Hypervector,
    ) -> AttentionProfile {
        let bundle = match &self.snapshot {
            Some(b) => *b,
            None => {
                if self.bundle.count == 0 {
                    return default_profile();
                }
                let snap = self.bundle.finish();
                self.snapshot = Some(snap);
                snap
            }
        };

        let unbound = bundle.unbind(regime_hv);
        AttentionProfile::from_hv(&unbound, &mut self.item_memory)
    }

    fn encode_profile(&mut self, profile: &AttentionProfile) -> Hypervector {
        let mut acc = BundleAccumulator::new();
        for (role, filler) in profile.to_labels() {
            let r_hv = self.item_memory.encode(&role);
            let f_hv = self.item_memory.encode(&filler);
            acc.add(&r_hv.bind(&f_hv));
        }
        acc.finish()
    }
}

fn default_profile() -> AttentionProfile {
    AttentionProfile {
        heuristic_weight: 0.20,
        embedding_weight: 0.20,
        surprise_weight: 0.20,
        prediction_error_weight: 0.20,
        novelty_weight: 0.20,
    }
}
```

### 8.3 Integration with the curiosity scorer

At each Theta tick, before scoring events:

1. Get the current regime classification from `RegimeClassifier::classify()`.
2. Retrieve the attention profile: `regime_attention_map.profile_for_regime(&regime_hv)`.
3. Apply the profile weights to the composite curiosity score.

After the Hedge algorithm updates its weights based on LLM evaluation feedback, record the outcome: `regime_attention_map.record(&regime_hv, &current_profile, effectiveness)`.

---

## 9. World models as HDC state vectors

### 9.1 The problem

The Golem's CorticalState stores 32 atomic signals as individual f32 values. Compositional queries -- "find past ticks where arousal was high AND the regime was volatile AND ETH was trending down" -- require scanning episode records.

### 9.2 HDC solution: composite state hypervectors

Each Gamma tick, encode the full CorticalState snapshot into a single hypervector. Store in a rolling buffer. Compositional queries reduce to a single Hamming scan.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};
use std::collections::VecDeque;

/// Rolling history of CorticalState snapshots encoded as hypervectors.
/// Supports compositional multi-signal queries over the state history.
pub struct WorldModelHistory {
    history: VecDeque<(u64, Hypervector)>,
    capacity: usize,
    item_memory: ItemMemory,
}

impl WorldModelHistory {
    pub fn new(capacity: usize, seed: u64) -> Self {
        WorldModelHistory {
            history: VecDeque::with_capacity(capacity),
            capacity,
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Encode a set of named signals into a composite state hypervector.
    /// Each signal is discretized into named bins (e.g., "arousal:high").
    pub fn encode_state(&mut self, signals: &[(&str, &str)]) -> Hypervector {
        let mut acc = BundleAccumulator::new();
        for (role, value) in signals {
            let r_hv = self.item_memory.encode(role);
            let v_hv = self.item_memory.encode(value);
            acc.add(&r_hv.bind(&v_hv));
        }
        acc.finish()
    }

    /// Record a state snapshot. Called at Gamma tick.
    pub fn record(&mut self, tick: u64, state_hv: Hypervector) {
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back((tick, state_hv));
    }

    /// Query for past ticks matching a partial state description.
    /// `query_signals`: the subset of signals to match on.
    /// Returns (tick, similarity) pairs sorted descending.
    ///
    /// Example: query(&[("arousal", "arousal:high"), ("regime", "regime:volatile")])
    /// finds all past ticks where both conditions held simultaneously.
    pub fn query(
        &mut self,
        query_signals: &[(&str, &str)],
        min_similarity: f32,
    ) -> Vec<(u64, f32)> {
        if query_signals.is_empty() {
            return vec![];
        }

        let query_hv = self.encode_state(query_signals);

        let mut results: Vec<(u64, f32)> = self
            .history
            .iter()
            .map(|(tick, state_hv)| (*tick, query_hv.similarity(state_hv)))
            .filter(|(_, sim)| *sim > min_similarity)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }

    /// Get the current state vector (most recent entry).
    pub fn current(&self) -> Option<&Hypervector> {
        self.history.back().map(|(_, hv)| hv)
    }

    /// Compute the average state over the last N ticks.
    /// Useful for smoothed regime detection.
    pub fn windowed_average(&self, window: usize) -> Option<Hypervector> {
        if self.history.len() < window {
            return None;
        }
        let mut acc = BundleAccumulator::new();
        for (_, hv) in self.history.iter().rev().take(window) {
            acc.add(hv);
        }
        Some(acc.finish())
    }
}
```

### 9.3 Memory footprint

The history buffer stores one 1,280-byte hypervector per Gamma tick. At 10-second Gamma cadence:

| Duration | Ticks | Memory |
|---|---|---|
| 1 hour | 360 | 450 KB |
| 8 hours | 2,880 | 3.6 MB |
| 24 hours | 8,640 | 10.8 MB |
| 1 week | 60,480 | 75.6 MB |

A 24-hour rolling buffer at ~11 MB is comfortable. This gives the Golem a day of compositional state history queryable in microseconds, without touching the Grimoire.

---

## 10. Multi-agent identity and trust

### 10.1 The problem

In a Clade of multiple Golems, each agent needs to verify identity and assess trustworthiness. The existing `ConsensusVote` primitive aggregates directional votes, but aggregation is unweighted and individual votes are exposed.

### 10.2 HDC solution: privacy-preserving vote bundling

Each Golem submits a vote as a bound triple: `bind(bind(position_hv, confidence_hv), reputation_hv)`. The aggregator bundles all submissions. Individual votes are not recoverable. Reputation weighting is natural: higher-reputation agents' votes are added more times.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// A Golem's contribution to a Clade consensus round.
pub struct VoteContribution {
    pub vote_hv: Hypervector,
    pub reputation_weight: f32,
}

/// Manages a single consensus round for the Clade.
pub struct CladeConsensus {
    aggregator: BundleAccumulator,
    item_memory: ItemMemory,
    participant_count: usize,
}

pub struct ConsensusResult {
    pub position: String,
    /// Confidence in [0.0, 1.0]. How far above random (0.5) the top position scored.
    pub confidence: f32,
    /// Disagreement in [0.0, 1.0]. How close the top two positions are.
    /// High disagreement = treat consensus with caution.
    pub disagreement: f32,
}

impl CladeConsensus {
    pub fn new(seed: u64) -> Self {
        CladeConsensus {
            aggregator: BundleAccumulator::new(),
            item_memory: ItemMemory::new(seed),
            participant_count: 0,
        }
    }

    /// Build a vote hypervector. Called locally by each Golem before submission.
    pub fn build_vote(
        &mut self,
        position: &str,
        confidence: f32,
        reputation: f32,
    ) -> VoteContribution {
        let pos_hv = self.item_memory.encode(position);
        let conf_label = format!("confidence:{}", (confidence * 5.0).floor() as u32);
        let conf_hv = self.item_memory.encode(&conf_label);
        let rep_label = format!("reputation:{}", (reputation * 5.0).floor() as u32);
        let rep_hv = self.item_memory.encode(&rep_label);

        let vote_hv = pos_hv.bind(&conf_hv).bind(&rep_hv);
        VoteContribution {
            vote_hv,
            reputation_weight: reputation,
        }
    }

    /// Receive and aggregate a vote. Reputation-weighted: higher-reputation
    /// Golems add their vote more times to the bundle.
    pub fn receive_vote(&mut self, vote: &VoteContribution) {
        let weight = (vote.reputation_weight * 10.0).round().max(1.0) as i32;
        self.aggregator.add_weighted(&vote.vote_hv, weight);
        self.participant_count += 1;
    }

    /// Compute the consensus position from the aggregated votes.
    pub fn compute(
        &mut self,
        position_candidates: &[&str],
    ) -> ConsensusResult {
        if self.aggregator.count == 0 {
            return ConsensusResult {
                position: "neutral".into(),
                confidence: 0.0,
                disagreement: 0.0,
            };
        }

        let consensus_hv = self.aggregator.finish();

        // Unbind the "average" confidence and reputation to isolate
        // the position signal.
        let avg_conf = self.item_memory.encode("confidence:2");
        let avg_rep = self.item_memory.encode("reputation:2");
        let position_signal = consensus_hv.unbind(&avg_conf).unbind(&avg_rep);

        let mut scores: Vec<(String, f32)> = position_candidates
            .iter()
            .map(|&pos| {
                let pos_hv = self.item_memory.encode(pos);
                (pos.to_string(), position_signal.similarity(&pos_hv))
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best = &scores[0];
        let second_best = scores.get(1).map(|s| s.1).unwrap_or(0.5);

        let confidence = ((best.1 - 0.5) / 0.5).clamp(0.0, 1.0);
        let margin = best.1 - second_best;
        let disagreement = (1.0 - margin / 0.5).clamp(0.0, 1.0);

        ConsensusResult {
            position: best.0.clone(),
            confidence,
            disagreement,
        }
    }
}
```

### 10.3 Trust accumulation

Trust between Clade members can be tracked with the same strategy-regime bundle pattern from section 4. Replace "strategy" with "peer_golem" and "regime" with "consensus_accuracy." Each time a peer's vote aligns with the eventual outcome, record a positive association. The trust bundle answers "how reliable has peer X been historically?" via a single unbind + similarity check.

Communication cost: 1,280 bytes per Golem per consensus round.

---

## 11. Compositional concept algebra

### 11.1 The problem

The Golem encounters novel patterns that are compositions of known concepts. A "flash loan sandwich attack on a new lending protocol" is flash_loan + sandwich + lending + novel_protocol. The curiosity system should recognize this as a novel *combination* of familiar elements.

### 11.2 HDC solution: concept algebra

Represent complex patterns as algebraic compositions of atomic concept hypervectors.

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Compositional concept builder.
/// Concepts are trees of bind and bundle operations over atomic terms.
pub struct ConceptAlgebra {
    item_memory: ItemMemory,
}

impl ConceptAlgebra {
    pub fn new(seed: u64) -> Self {
        ConceptAlgebra {
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Encode an atomic concept.
    pub fn atom(&mut self, name: &str) -> Hypervector {
        self.item_memory.encode(name)
    }

    /// Compose two concepts as a bound pair (relational composition).
    /// "flash_loan applied_to dex" = bind(flash_loan, dex)
    /// The result is dissimilar to both inputs individually.
    pub fn relate(a: &Hypervector, b: &Hypervector) -> Hypervector {
        a.bind(b)
    }

    /// Compose concepts as a bundle (set/conjunction).
    /// "flash_loan AND sandwich AND lending" = bundle(flash_loan, sandwich, lending)
    /// The result is similar to all inputs.
    pub fn conjoin(concepts: &[Hypervector]) -> Hypervector {
        let mut acc = BundleAccumulator::new();
        for c in concepts {
            acc.add(c);
        }
        acc.finish()
    }

    /// Build a complex concept from a description.
    /// Syntax: atoms joined by "+" (bundle) or "*" (bind).
    /// Bind has higher precedence than bundle.
    ///
    /// Example: "flash_loan * sandwich + lending * novel_protocol"
    /// = bundle(bind(flash_loan, sandwich), bind(lending, novel_protocol))
    pub fn parse(&mut self, expr: &str) -> Hypervector {
        let bundle_terms: Vec<&str> = expr.split('+').map(|s| s.trim()).collect();
        let mut acc = BundleAccumulator::new();

        for term in bundle_terms {
            let bind_parts: Vec<&str> = term.split('*').map(|s| s.trim()).collect();
            let bound = bind_parts
                .iter()
                .map(|&name| self.item_memory.encode(name))
                .reduce(|a, b| a.bind(&b))
                .unwrap_or_else(|| Hypervector([0u64; crate::hdc::HDC_WORDS]));
            acc.add(&bound);
        }

        acc.finish()
    }

    /// Score how much a transaction fingerprint matches a concept.
    pub fn match_score(
        tx_hv: &Hypervector,
        concept_hv: &Hypervector,
    ) -> f32 {
        tx_hv.similarity(concept_hv)
    }

    /// Decompose a transaction into known concepts using the resonator network.
    /// This is a convenience wrapper that builds the resonator from
    /// concept codebooks and runs factorization.
    pub fn decompose(
        &self,
        tx_hv: &Hypervector,
        codebooks: &[(&str, &[(String, Hypervector)])],
    ) -> Option<crate::hdc::FactorizationResult> {
        let mut resonator = crate::hdc::ResonatorNetwork::new(50);
        for (name, entries) in codebooks {
            resonator.add_factor(
                name,
                entries.iter().map(|(l, h)| (l.clone(), *h)).collect(),
            );
        }
        resonator.factorize(tx_hv)
    }
}
```

### 11.3 Novelty scoring for compositional patterns

The concept algebra enables more precise novelty scoring than raw embedding distance. Instead of "how far is this from all known transactions?" (which conflates novel combinations with noise), the system asks "which known concepts appear, and is the *combination* novel?"

```rust
/// Score the compositional novelty of a transaction.
/// Returns a value in [0.0, 1.0] where:
///   0.0 = all components are familiar AND the combination has been seen before
///   0.5 = components are familiar but the combination is new
///   1.0 = components themselves are unfamiliar
pub fn compositional_novelty(
    tx_hv: &Hypervector,
    concept_codebook: &[(String, Hypervector)],
    known_combinations: &[Hypervector],
) -> f32 {
    // Step 1: how familiar are the individual components?
    let max_component_sim = concept_codebook
        .iter()
        .map(|(_, hv)| tx_hv.similarity(hv))
        .fold(0.0f32, f32::max);

    let component_familiarity = ((max_component_sim - 0.5) / 0.5).clamp(0.0, 1.0);

    // Step 2: how familiar is the combination?
    let max_combo_sim = known_combinations
        .iter()
        .map(|hv| tx_hv.similarity(hv))
        .fold(0.0f32, f32::max);

    let combination_familiarity = ((max_combo_sim - 0.5) / 0.5).clamp(0.0, 1.0);

    // High component familiarity + low combination familiarity = novel composition
    // Low component familiarity = genuinely unknown
    if component_familiarity < 0.3 {
        // Components are unknown -- this is deep novelty
        1.0
    } else {
        // Components are known -- novelty comes from the combination
        1.0 - combination_familiarity
    }
}
```

This scoring function distinguishes three cases that flat similarity metrics conflate:

1. **Familiar pattern (score ~0.0).** Both the components and their combination have been seen before. "Another Uniswap v3 swap."

2. **Novel combination (score ~0.5).** The individual components are known, but this specific combination is new. "A flash loan targeting a lending protocol we've seen, but using a DEX path we haven't observed before." This is the highest-value curiosity signal.

3. **Deep novelty (score ~1.0).** The components themselves are unfamiliar. "A contract using an unknown protocol with unfamiliar function selectors." Triggers exploration but with less context for reasoning.

---

## 12. Integration roadmap

These advanced applications depend on the foundation layer being in place. Recommended implementation order:

1. **World model history** (low effort, immediate value). Requires only `Hypervector` and `BundleAccumulator` from the foundation. Provides compositional state queries for CorticalState and Theta-tick LLM immediately.

2. **Clade consensus** (medium effort, needed when multi-agent support ships). Requires the foundation layer plus Clade networking infrastructure. Can be implemented in parallel with other Clade features.

3. **Regime attention modulation** (medium effort, depends on curiosity scorer and Hedge algorithm). Requires the regime classifier from section 7 and the Hedge algorithm from the curiosity learning spec. Best implemented after a few hundred Theta ticks of scoring data.

4. **Compositional concept algebra** (medium effort, highest payoff for novel pattern detection). Requires the transaction encoder from `shared/hdc-fingerprints.md` and the resonator network from the foundation. Plugs directly into the curiosity pipeline.

All four share the same `golem_core::hdc` module and item memory seed. No additional dependencies. No changes to existing LanceDB schema, SQLite tables, or CorticalState signals.

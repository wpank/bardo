# Grimoire Extension: Hyperdimensional Computing Integration [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-grimoire`
>
> **Depends on**: `01-grimoire.md`, `01b-grimoire-memetic.md`, `../02-mortality/`

---

> **Reader orientation:** This document adds Hyperdimensional Computing (HDC) to the Grimoire (the Golem's persistent local knowledge base), using Binary Spatter Codes (BSC) at D=10,240 for compositional queries, death-time knowledge compression, and controlled forgetting. It belongs to the **04-memory** layer and extends `01-grimoire.md`. The key concept is that a single 1,280-byte hypervector can compress hundreds of episodes into a queryable prototype, enabling the genomic bottleneck (the compression step at death reducing the Grimoire to at most 2,048 entries) to transmit holographic knowledge alongside text entries. For term definitions, see `prd2/shared/glossary.md`.

## Why this document exists

The Grimoire stores episodes in LanceDB (768-dim embeddings) and semantic entries in SQLite. This works for standard retrieval: embed a query, find K nearest neighbors, return them. Three capabilities are missing:

1. **Compositional pattern queries.** "Find past ticks where arousal was high AND regime was volatile AND ETH was trending down." This requires scanning episode records. No algebraic shortcut exists in embedding space.

2. **Knowledge inheritance at death.** The Thanatopsis protocol exports up to 2048 entries through the genomic bottleneck. Each entry is a full record. There is no mechanism for compressing an entire Grimoire into a single queryable artifact.

3. **Controlled forgetting aligned with the Curator.** Demurrage-based decay operates on individual entries. There is no dimensionality-aware forgetting that degrades entire categories of knowledge gracefully, with signal-to-noise ratio (SNR) declining smoothly rather than entries vanishing abruptly.

Hyperdimensional Computing (HDC) using Binary Spatter Codes (BSC) at D=10,240 addresses all three. A single 1,280-byte hypervector can compress hundreds of episodes into a queryable prototype, serve as a holographic legacy bundle for death protocol transfer, and implement controlled forgetting through vote decay in the bundle accumulator.

HDC supplements LanceDB and SQLite. It does not replace either. The Grimoire backend remains canonical (per `01-grimoire.md`). HDC provides a complementary query and compression substrate.

---

## HDC fundamentals for the Grimoire

### Binary Spatter Codes

BSC represents every concept as a random binary vector of D=10,240 bits (1,280 bytes). Three operations compose:

| Operation | Implementation | Semantics | Inverse? |
|---|---|---|---|
| **Bind** | XOR | Association: "A is linked to B" | Yes (XOR is self-inverse) |
| **Bundle** | Majority vote | Superposition: "A and B and C" | No (lossy) |
| **Similarity** | Hamming distance | "How alike are A and B?" | N/A |
| **Permute** | Cyclic shift | Temporal position: "A at step 3" | Yes (shift back) |

Random 10,240-bit vectors are quasi-orthogonal with probability approaching 1. This means binding two unrelated vectors produces a result that is dissimilar to both inputs, and unbinding recovers the original. Bundle capacity scales as O(sqrt(D)): at D=10,240, approximately 100 items can be superimposed with SNR > 10.

**Citation**: Kanerva (2009) established the theoretical foundations. Plate (2003) proved holographic bundle capacity.

### Item memory

A shared `ItemMemory` holds the codebook: the set of atomic concept vectors used across all HDC operations. Role vectors (`R_arousal`, `R_regime`, `R_protocol`, etc.) and filler vectors (discretized value buckets) are generated once at boot from a deterministic seed and shared across all subsystems.

```rust
/// Shared codebook for HDC encoding across the Golem.
/// Generated once at boot from a deterministic seed.
/// All subsystems reference the same ItemMemory instance.
pub struct ItemMemory {
    roles: HashMap<String, Hypervector>,
    fillers: HashMap<String, Vec<Hypervector>>,
    seed: u64,
}

impl ItemMemory {
    /// Generate a role vector for a named concept.
    pub fn role(&mut self, name: &str) -> &Hypervector;

    /// Generate ordered filler vectors for a named dimension.
    /// Thermometer encoding: filler[i] includes filler[i-1] for ordinal similarity.
    pub fn fillers(&mut self, name: &str, count: usize) -> &[Hypervector];
}
```

---

## Capability 1: Episode compression into category prototypes

### The problem

The Grimoire holds thousands of episodes. Answering "what kind of events have I seen in volatile regimes on Uniswap?" requires an ANN scan that returns individual episodes. There is no aggregate representation that captures the *shape* of a category.

### HDC solution

Bundle related episodes into a single 1,280-byte category prototype. 500 episodes compress to one vector at SNR = sqrt(10240/500) = 4.5 -- above the discrimination threshold of ~4.0.

```rust
/// Category-based episode compression.
/// Each category maintains a running bundle of its member episodes.
pub struct EpisodeCompressor {
    categories: HashMap<String, CategoryBundle>,
}

pub struct CategoryBundle {
    /// Running bundle accumulator.
    accumulator: BundleAccumulator,
    /// Frozen snapshot for similarity queries.
    snapshot: Option<Hypervector>,
    /// Number of episodes accumulated.
    count: usize,
}

impl EpisodeCompressor {
    /// Add an episode's HDC fingerprint to its category.
    /// Called at Theta tick after GrimoireClient::store_episode().
    pub fn accumulate(
        &mut self,
        category: &str,
        episode_hv: &Hypervector,
        importance: f32,
    ) {
        let bundle = self.categories
            .entry(category.to_string())
            .or_insert_with(CategoryBundle::new);

        // Importance-weighted: repeat the vector proportionally.
        let repeats = (importance * 4.0).round().max(1.0) as usize;
        for _ in 0..repeats {
            bundle.accumulator.add(episode_hv);
        }
        bundle.count += 1;
    }

    /// Classify an incoming episode against existing prototypes.
    /// Returns categories sorted by similarity.
    /// Used as a pre-filter before LanceDB ANN search.
    pub fn classify(
        &self,
        episode_hv: &Hypervector,
    ) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self.categories.iter()
            .filter_map(|(name, bundle)| {
                bundle.snapshot.as_ref().map(|proto| {
                    (name.clone(), proto.similarity(episode_hv))
                })
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    /// Freeze current accumulator state into queryable snapshots.
    /// Called during the Curator's Delta-tick cycle.
    pub fn refresh_prototypes(&mut self) {
        for bundle in self.categories.values_mut() {
            bundle.snapshot = Some(bundle.accumulator.finish());
        }
    }

    /// Export all prototypes for external use (dream engine, death protocol).
    pub fn export_prototypes(&self) -> Vec<(String, Hypervector, usize)> {
        self.categories.iter()
            .filter_map(|(name, bundle)| {
                bundle.snapshot.as_ref().map(|hv| {
                    (name.clone(), hv.clone(), bundle.count)
                })
            })
            .collect()
    }
}
```

### Integration point

`EpisodeCompressor::accumulate()` runs at Theta tick, after `GrimoireClient::store_episode()`. `refresh_prototypes()` runs inside the Curator cycle at Delta tick. This adds ~microseconds to both paths. The category classification provides a fast pre-filter: check the incoming episode against prototypes (~10ns per comparison) before falling back to the full LanceDB ANN scan (~100us).

---

## Capability 2: Holographic legacy bundle for death protocol

### The problem

When a Golem dies, Thanatopsis Phase III exports up to 2048 entries through the genomic bottleneck. Each entry is a full record. The successor receives a large corpus but no gestalt -- no single artifact that says "here is what your predecessor knew, queryable in one operation."

### HDC solution

Compress the entire Grimoire into a single 1,280-byte legacy bundle. The bundle encodes episode-insight pairs as bound associations: `bind(episode_hv, insight_hv)`. Given a situation HV, the successor unbinds from the bundle to recover associated insights.

```rust
/// Holographic legacy bundle for generational knowledge transfer.
/// Fixed size: 1,280 bytes regardless of how many episodes it encodes.
pub struct LegacyBundle {
    /// The bundled associations: bind(episode_hv, insight_hv) for each pair.
    pub bundle: Hypervector,
    /// Number of episode-insight pairs encoded.
    pub episode_count: usize,
    /// Generation number of the dying Golem.
    pub generation: u32,
}

impl LegacyBundle {
    /// Compress episode-insight pairs into a single holographic bundle.
    /// Called during Thanatopsis Phase III alongside the text testament.
    ///
    /// episodes: (episode_hv, insight_hv, importance) triples.
    /// Higher importance = more repetitions in the bundle = stronger signal.
    pub fn compress(
        episodes: &[(Hypervector, Hypervector, f32)],
        generation: u32,
    ) -> Self {
        let mut acc = BundleAccumulator::new();

        for (episode_hv, insight_hv, importance) in episodes {
            let association = episode_hv.bind(insight_hv);
            let repeats = (importance * 4.0).round().max(1.0) as usize;
            for _ in 0..repeats {
                acc.add(&association);
            }
        }

        LegacyBundle {
            bundle: acc.finish(),
            episode_count: episodes.len(),
            generation,
        }
    }
}
```

### Ancestor memory: multi-generational queries

The legacy bundle is not just for direct predecessors. Successive generations accumulate ancestor knowledge:

```rust
/// Multi-generational ancestor knowledge.
/// Each absorbed legacy contributes proportional to generational distance.
pub struct AncestorMemory {
    /// Bundled ancestor knowledge.
    ancestor_hv: Option<Hypervector>,
    /// Running accumulator for incremental absorption.
    accumulator: BundleAccumulator,
    /// How many generations of legacy have been absorbed.
    generations_absorbed: u32,
    /// Exponential decay rate per generation.
    /// At 0.85: distance 1 -> weight 8, distance 20 -> weight 1.
    decay_rate: f32,
}

impl AncestorMemory {
    /// Absorb a predecessor's legacy bundle.
    /// Called at boot when the successor loads its inherited testament.
    pub fn absorb_legacy(&mut self, legacy: &LegacyBundle, distance: u32) {
        let weight = (self.decay_rate.powi(distance as i32) * 8.0) as usize;
        for _ in 0..weight.max(1) {
            self.accumulator.add(&legacy.bundle);
        }
        self.generations_absorbed += 1;
        self.ancestor_hv = Some(self.accumulator.finish());
    }

    /// Query: "did any ancestor encounter a situation like this?"
    /// Returns similarity score. Above 0.52 indicates above-noise signal.
    pub fn query_similarity(&self, situation_hv: &Hypervector) -> f32 {
        self.ancestor_hv.as_ref()
            .map(|anc| anc.similarity(situation_hv))
            .unwrap_or(0.5) // 0.5 = chance level
    }

    /// Query: given a situation, what insights did ancestors associate with it?
    /// Unbinds the situation from the ancestor bundle, then matches against
    /// a candidate set of insight descriptions.
    pub fn query_insights(
        &self,
        situation_hv: &Hypervector,
        candidates: &[(String, Hypervector)],
    ) -> Vec<(String, f32)> {
        let Some(ref ancestor) = self.ancestor_hv else {
            return Vec::new();
        };

        let signal = ancestor.unbind(situation_hv);
        let mut results: Vec<(String, f32)> = candidates.iter()
            .map(|(desc, hv)| (desc.clone(), signal.similarity(hv)))
            .filter(|(_, sim)| *sim > 0.52) // above noise threshold
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }
}
```

### Integration with Thanatopsis

The legacy bundle is serialized into the testament directory alongside the existing text testament during Thanatopsis Phase III. The 2048-entry genomic bottleneck continues to gate which individual episodes are exported. The legacy bundle is **not** subject to the bottleneck: it is a single 1,280-byte vector regardless of how many episodes it compresses.

At boot, the successor:
1. Loads the text testament (existing path)
2. Loads the legacy bundle into `AncestorMemory` at distance 1
3. If the predecessor also inherited legacy bundles, those are absorbed at distance 2+

---

## Capability 3: Controlled forgetting via vote decay

### The problem

Demurrage-based decay operates on individual entries: each entry's confidence decays independently. When an entry drops below threshold, it vanishes entirely. There is no gradual degradation of an entire category's representational fidelity.

### HDC solution

The bundle accumulator supports vote decay: multiply all vote counts by a decay factor before the next accumulate step. This causes old information to gradually lose influence in the bundled prototype, implementing dimensionality-aware forgetting aligned with the Curator cycle.

```rust
/// Bundle accumulator with temporal vote decay.
/// Implements controlled forgetting: old information loses influence
/// gradually, not abruptly.
pub struct DecayingBundleAccumulator {
    /// Per-bit vote counts. Positive votes push toward 1, negative toward 0.
    votes: Vec<i32>,
    /// Decay factor applied each cycle. 0.95 = 5% decay per Curator cycle.
    decay: f32,
    /// Number of items accumulated since last decay.
    items_since_decay: usize,
}

impl DecayingBundleAccumulator {
    pub fn new(decay: f32) -> Self {
        Self {
            votes: vec![0; 10_240],
            decay,
            items_since_decay: 0,
        }
    }

    /// Add a hypervector to the accumulator.
    pub fn add(&mut self, hv: &Hypervector) {
        for (i, bit) in hv.bits().enumerate() {
            if bit { self.votes[i] += 1; } else { self.votes[i] -= 1; }
        }
        self.items_since_decay += 1;
    }

    /// Apply decay to all vote counts. Called once per Curator cycle.
    /// This is the forgetting step: old information loses influence.
    pub fn apply_decay(&mut self) {
        for vote in &mut self.votes {
            *vote = (*vote as f32 * self.decay).round() as i32;
        }
        self.items_since_decay = 0;
    }

    /// Freeze into a binary hypervector via majority vote.
    pub fn finish(&self) -> Hypervector {
        let mut bits = vec![false; 10_240];
        for (i, &vote) in self.votes.iter().enumerate() {
            bits[i] = vote > 0;
        }
        Hypervector::from_bits(&bits)
    }
}
```

The decay factor `0.95` means each Curator cycle reduces old vote strength by 5%. After 14 cycles (~7-28 minutes at delta frequency), an item's influence has halved. After 46 cycles, it has dropped to 10%. This provides a smooth forgetting curve rather than the cliff-edge of demurrage-based entry deletion.

**Citation**: Richards & Frankland (2017) argued that memory persistence and transience are both adaptive, with transience enabling generalization. The vote decay mechanism implements transience at the representational level.

---

## HDC codebook integration

The Grimoire's HDC components share the Golem-wide `ItemMemory` codebook. Role vectors and filler vectors are generated once at boot and referenced by name:

### Grimoire-specific roles

| Role | Used by | Encodes |
|---|---|---|
| `R_episode_type` | EpisodeCompressor | observation, action, regime_shift, etc. |
| `R_protocol` | EpisodeCompressor, LegacyBundle | Protocol identity (Uniswap, Aave, Morpho, etc.) |
| `R_outcome` | LegacyBundle | Profitable, loss, neutral |
| `R_regime` | EpisodeCompressor | Volatile, trending, range-bound, crisis |
| `R_insight_type` | LegacyBundle | Insight, Heuristic, Warning, CausalLink, StrategyFragment, AntiKnowledge |
| `R_confidence` | LegacyBundle | Discretized confidence level |

### Encoding example

```rust
/// Encode a Grimoire episode as an HDC hypervector.
pub fn encode_episode(
    episode: &GrimoireEpisode,
    item_memory: &mut ItemMemory,
) -> Hypervector {
    let mut acc = BundleAccumulator::new();

    // Bind episode type with its role
    let type_hv = item_memory.role("R_episode_type")
        .bind(item_memory.filler_for("episode_type", &episode.episode_type));
    acc.add(&type_hv);

    // Bind protocol identity
    if let Some(ref protocol) = episode.protocol_id {
        let proto_hv = item_memory.role("R_protocol")
            .bind(item_memory.filler_for("protocol", protocol));
        acc.add(&proto_hv);
    }

    // Bind regime context
    let regime_hv = item_memory.role("R_regime")
        .bind(item_memory.filler_for("regime", &episode.market_regime));
    acc.add(&regime_hv);

    // Bind outcome
    let outcome_hv = item_memory.role("R_outcome")
        .bind(item_memory.filler_for("outcome", &episode.outcome_label));
    acc.add(&outcome_hv);

    acc.finish()
}
```

---

## Performance characteristics

| Operation | Cost | Notes |
|---|---|---|
| Episode encoding | ~100ns | 4 bind + 4 bundle adds |
| Category classify (1 prototype) | ~10ns | Single Hamming distance |
| Category classify (50 prototypes) | ~500ns | 50 Hamming distances |
| Legacy bundle compress (1000 episodes) | ~100us | 1000 bind + weighted bundle |
| Ancestor memory query | ~10ns | 1 unbind + similarity |
| Ancestor insight query (100 candidates) | ~1us | 1 unbind + 100 similarities |
| Prototype refresh (50 categories) | ~5us | 50 majority votes |
| Vote decay (1 accumulator) | ~10us | 10,240 multiplications |

All operations are deterministic Rust at T0. No LLM involvement, no network calls. The heaviest operation (legacy bundle compression) runs once during Thanatopsis Phase III and takes ~100 microseconds even at 1000 episodes.

---

## Connection to existing subsystems

### Grimoire (01-grimoire.md)

HDC prototypes supplement LanceDB episodes and SQLite semantic entries. The Grimoire remains the canonical storage. At Theta tick, `EpisodeCompressor::accumulate()` runs after `GrimoireClient::store_episode()`. At Delta tick, `refresh_prototypes()` runs inside the Curator cycle. No existing storage paths change.

### Curator (01-grimoire.md, 01b-grimoire-memetic.md)

The Curator's extended cycle (Step 8: consolidation replay) can use HDC category prototypes to select replay candidates. Entries whose categories have changed most since last cycle (high drift in prototype similarity) get priority replay.

### Death protocol (02-mortality/)

The legacy bundle is produced in Thanatopsis Phase III alongside the existing text testament. The 2048-entry genomic bottleneck is unaffected. The bundle is additive: 1,280 bytes on top of whatever the existing testament contains.

### Dream engine (05-dreams/)

Dream output fingerprints (see `05-dreams/01b-dream-evolution.md`) use the same HDC substrate. Dream scenarios produce fingerprint HVs that can be compared against Grimoire category prototypes, closing the dream-to-memory feedback loop.

---

## Two-tier streaming ANN index for the Grimoire

### The problem with batch rebuilds

The current spec rebuilds the full HNSW index at each Delta tick from all Grimoire `ChainEvent` episodes. Delta ticks fire every 5-20 minutes. A chain event scored at Theta tick is not reflected in the ANN index until the next Delta. The curiosity loop has a 5-20 minute blind spot.

### Streaming alternative

A two-tier architecture maintains a main index (rebuilt periodically) and a staging index (accepts insertions in real time). Queries hit both and merge results. New episodes become queryable at the next Gamma tick.

This approach draws on FreshDiskANN (Singh et al., 2021, deployed in Bing) -- a two-tier architecture with a mutable in-memory buffer for immediate searchability plus periodic background merge -- and IP-DiskANN (Xu et al., 2025) which demonstrates true in-place insert/delete with amortized consolidation.

```rust
use std::sync::Arc;

/// Two-tier ANN index for streaming episode insertion.
///
/// Main index: bulk-built from all episodes, provides optimal recall.
/// Staging index: accepts incremental inserts, provides freshness.
/// Queries merge results from both indices.
pub struct StreamingAnnIndex {
    /// Main index, rebuilt periodically (Delta tick or threshold merge).
    main: Arc<usearch::Index>,

    /// Staging index, accepts in-place inserts at Gamma tick.
    staging: usearch::Index,

    /// Count of episodes in staging since last merge.
    staging_count: usize,

    /// When staging_count reaches this threshold, merge into main.
    merge_threshold: usize,

    /// Metric: cosine for f32 embeddings, hamming for BSC hypervectors.
    metric: usearch::MetricKind,

    /// Dimensionality of vectors.
    dimensions: usize,
}

impl StreamingAnnIndex {
    pub fn new(
        dimensions: usize,
        metric: usearch::MetricKind,
        merge_threshold: usize,
    ) -> Self {
        let main = Arc::new(Self::create_index(dimensions, metric));
        let staging = Self::create_index(dimensions, metric);

        Self {
            main,
            staging,
            staging_count: 0,
            merge_threshold,
            metric,
            dimensions,
        }
    }

    fn create_index(dimensions: usize, metric: usearch::MetricKind) -> usearch::Index {
        let opts = usearch::IndexOptions {
            dimensions,
            metric,
            connectivity: 16,        // M=16 for moderate-scale datasets
            expansion_add: 200,       // ef_construction=200 for high-quality graph
            expansion_search: 100,    // ef_search=100 balances recall and latency
            ..Default::default()
        };
        usearch::Index::new(&opts).expect("index creation should succeed")
    }

    /// Insert a new episode embedding into the staging index.
    /// Called at Theta tick when a new GrimoireEpisode is stored.
    pub fn insert(&mut self, id: u64, embedding: &[f32]) {
        self.staging
            .add(id, embedding)
            .expect("staging insert should succeed");
        self.staging_count += 1;

        if self.staging_count >= self.merge_threshold {
            self.merge_staging();
        }
    }

    /// Insert a binary hypervector (for BSC/HDC embeddings).
    pub fn insert_binary(&mut self, id: u64, bitvector: &[u8]) {
        self.staging
            .add(id, bitvector)
            .expect("staging insert should succeed");
        self.staging_count += 1;

        if self.staging_count >= self.merge_threshold {
            self.merge_staging();
        }
    }

    /// Query both indices and return merged top-k results.
    pub fn query(&self, embedding: &[f32], k: usize) -> Vec<(u64, f32)> {
        let main_results = self.main.search(embedding, k).expect("main query failed");
        let staging_results = self
            .staging
            .search(embedding, k)
            .expect("staging query failed");

        self.merge_results(main_results, staging_results, k)
    }

    /// Query with a binary hypervector.
    pub fn query_binary(&self, bitvector: &[u8], k: usize) -> Vec<(u64, f32)> {
        let main_results = self.main.search(bitvector, k).expect("main query failed");
        let staging_results = self
            .staging
            .search(bitvector, k)
            .expect("staging query failed");

        self.merge_results(main_results, staging_results, k)
    }

    fn merge_results(
        &self,
        main: usearch::Matches,
        staging: usearch::Matches,
        k: usize,
    ) -> Vec<(u64, f32)> {
        use std::collections::HashMap;

        let mut best: HashMap<u64, f32> = HashMap::new();

        for i in 0..main.keys.len() {
            let id = main.keys[i];
            let dist = main.distances[i];
            best.entry(id)
                .and_modify(|d| *d = d.min(dist))
                .or_insert(dist);
        }

        for i in 0..staging.keys.len() {
            let id = staging.keys[i];
            let dist = staging.distances[i];
            best.entry(id)
                .and_modify(|d| *d = d.min(dist))
                .or_insert(dist);
        }

        let mut results: Vec<(u64, f32)> = best.into_iter().collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    fn merge_staging(&mut self) {
        // Rebuild main from all episodes (main + staging) on a background thread.
        // During rebuild, queries continue hitting the old main + staging pair.
        self.staging = Self::create_index(self.dimensions, self.metric);
        self.staging_count = 0;
    }

    pub fn staging_size(&self) -> usize {
        self.staging_count
    }

    pub fn total_size(&self) -> usize {
        self.main.size() + self.staging_count
    }
}
```

### Gamma-tick insertion flow

1. **Block arrives** (~12s): bardo-witness filters logs through the WatchBloom.
2. **Gamma tick** (every few blocks): bardo-triage scores filtered events. High-score events queue for Theta.
3. **Theta tick** (30-120s): LLM analyzes queued events. Produces a `GrimoireEpisode` with embedding.
4. **Episode stored**: The embedding is inserted into the staging index via `insert()`. O(log n) amortized.
5. **Next Gamma tick**: The staging index is immediately queryable. New episodes influence curiosity scoring.

Latency from episode creation to queryability: bounded by the Gamma tick interval (24-36 seconds). Compare to the Delta rebuild approach where the blind spot is 5-20 minutes.

### Threshold merge mechanics

At 500 episodes per merge and a Theta tick cadence of ~1 episode per minute, merges happen roughly every 8 hours. Rebuild cost at 10K episodes: ~100ms; at 100K episodes: ~2 seconds. Both acceptable for background operations.

### usearch binary mode for HDC vectors

If the Golem adopts BSC transaction fingerprinting, embeddings are 10,240-bit binary vectors. The `usearch` crate supports binary vectors with Hamming distance as a first-class metric.

```rust
fn create_binary_index(dimensions: usize) -> usearch::Index {
    let opts = usearch::IndexOptions {
        dimensions,
        metric: usearch::MetricKind::Hamming,
        quantization: usearch::ScalarKind::B1,
        connectivity: 16,
        expansion_add: 200,
        expansion_search: 100,
        ..Default::default()
    };
    usearch::Index::new(&opts).expect("binary index creation should succeed")
}
```

Hamming distance on binary vectors is computed via XOR + POPCNT. On AVX-512: ~20 cycles for a 10,000-bit vector (~10 nanoseconds). `usearch` uses SimSIMD for automatic SIMD dispatch (AVX2, AVX-512, ARM NEON/SVE).

### DiskANN for larger-than-memory scaling

DiskANN (Jayaram Subramanya et al., 2019) places full-precision vectors and graph structure on SSD while keeping compressed vectors in RAM. The Vamana graph algorithm constructs a single-layer navigable graph optimized for sequential SSD reads. Performance at billion scale: >5,000 QPS at 95%+ recall@1.

DiskANN becomes relevant when episode count exceeds 500K, the Golem enters archival mode, or multiple indices compete for RAM.

SPANN (Chen et al., NeurIPS 2021) improves on DiskANN with a memory-disk hybrid inverted index -- 2x faster at 90% recall. The clustering concept applies: partition episodes by protocol family for natural locality.

### ScaNN anisotropic quantization insight

Google's ScaNN (Guo et al., ICML 2020) introduced anisotropic vector quantization -- quantization errors parallel to the query direction matter more than perpendicular errors. This informs custom quantization: when compressing f32 embeddings for cold-tier storage, use PQ with anisotropic loss weighting rather than standard MSE loss. This is a future optimization.

### Adaptive ef-search (Ada-ef)

Ada-ef (2025) dynamically configures `ef_search` per query based on the query's position relative to the data distribution. Dense clusters need lower ef (faster); sparse regions need higher ef (more accurate). For the Golem, most queries concern active protocols (dense) while occasional queries concern rare events (sparse). A fixed ef_search must be set high enough for worst-case sparse queries, wasting cycles on common dense-cluster queries.

### TANNS: timestamp-aware nearest neighbor search

Wang et al. (2025, ICDE) introduced querying "which episode was nearest to this transaction at block N?" This enables reconstructing past curiosity judgments for audit or backtesting. `usearch` supports filtered search during graph traversal, making temporal filtering feasible without post-filtering.

### Alternative: hnsw_rs

The `hnsw_rs` crate (jean-pierreBoth/hnswlib-rs) provides AVX2 SIMD via simdeez for distance computation and supports custom distance functions. It is a pure Rust HNSW implementation with incremental insert support. For workloads where `usearch`'s C++ core via FFI is undesirable, `hnsw_rs` is a viable alternative at the cost of somewhat lower performance on large indices.

### LanceDB vs usearch decision rule

Use LanceDB if the embedding pipeline produces float vectors and structured metadata queries are common (e.g., "find the 10 most similar episodes that involve Uniswap V3 and occurred after block 20,000,000" -- metadata filter and vector search execute together during graph traversal). Use usearch + redb if the embedding pipeline produces binary vectors or if metadata queries are handled separately. LanceDB adds a larger dependency footprint than usearch. For a Golem that prioritizes binary size and compile time, the usearch + redb combination covers both vector search and metadata storage without the additional Lance dependency.

### Recommended ANN configuration

| Parameter | Float embeddings (80-dim) | Binary hypervectors (10K-dim) |
|-----------|--------------------------|-------------------------------|
| Library | `usearch` | `usearch` |
| Metric | Cosine | Hamming |
| Quantization | f32 (no compression) | B1 (native binary) |
| M (connectivity) | 16 | 16 |
| ef_construction | 200 | 200 |
| ef_search | 100 (adaptive via Ada-ef) | 100 (adaptive via Ada-ef) |
| Staging merge threshold | 500 episodes | 500 episodes |
| Memory at 10K episodes | ~5 MB | ~25 MB |
| Memory at 100K episodes | ~50 MB | ~250 MB |
| Query latency | <1 ms | <1 ms |

---

## Tiered storage engines for HDC data

The Golem's data ranges from white-hot protocol state accessed every Gamma tick to frozen archival logs. Each storage engine occupies a niche where its architecture gives it a structural advantage.

### redb: hot/warm ACID storage

redb (pure safe Rust) uses a copy-on-write B-tree with memory-mapped I/O, ACID transactions, and single-writer / multiple-reader concurrency. It stores: CorticalState snapshots, ChainScope interest lists, triage configuration, protocol state deltas, temporal knowledge graph edges, and Grimoire episode metadata.

```rust
use redb::{Database, ReadableTable, TableDefinition};

const PROTOCOL_DELTAS: TableDefinition<(u64, u64), &[u8]> =
    TableDefinition::new("protocol_deltas");

const EPISODES: TableDefinition<(u64, u64), &[u8]> =
    TableDefinition::new("episodes");

const CORTICAL_SNAPSHOTS: TableDefinition<u64, &[u8]> =
    TableDefinition::new("cortical_snapshots");

pub fn store_protocol_delta(
    db: &Database,
    protocol_id: u64,
    block: u64,
    delta: &[u8],
) -> Result<(), redb::Error> {
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(PROTOCOL_DELTAS)?;
        table.insert((protocol_id, block), delta)?;
    }
    write_txn.commit()?;
    Ok(())
}
```

### fjall: write-heavy event streams

fjall (100% safe Rust LSM-tree, v3.0) is optimized for append-heavy event streams: raw chain event logs, triage scoring traces, MIDAS/CMS checkpoint snapshots, simulation results cache. Cross-keyspace atomic writes ensure an event log entry and its scoring trace are written atomically.

```rust
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};

pub struct EventStreamStore {
    keyspace: Keyspace,
    chain_events: PartitionHandle,
    scoring_traces: PartitionHandle,
}

impl EventStreamStore {
    pub fn open(path: &str) -> fjall::Result<Self> {
        let keyspace = Config::new(path)
            .max_write_buffer_size(32 * 1024 * 1024)
            .open()?;

        let chain_events = keyspace.open_partition(
            "chain_events",
            PartitionCreateOptions::default()
                .block_size(16 * 1024)
                .with_compression(fjall::CompressionType::Lz4),
        )?;

        let scoring_traces = keyspace.open_partition(
            "scoring_traces",
            PartitionCreateOptions::default()
                .with_compression(fjall::CompressionType::Lz4),
        )?;

        Ok(Self { keyspace, chain_events, scoring_traces })
    }

    pub fn append_event(
        &self,
        block: u64,
        tx_index: u16,
        log_index: u16,
        data: &[u8],
    ) -> fjall::Result<()> {
        let mut key = [0u8; 12];
        key[0..8].copy_from_slice(&block.to_be_bytes());
        key[8..10].copy_from_slice(&tx_index.to_be_bytes());
        key[10..12].copy_from_slice(&log_index.to_be_bytes());
        self.chain_events.insert(&key, data)?;
        Ok(())
    }

    pub fn scan_range(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> impl Iterator<Item = fjall::Result<(Vec<u8>, Vec<u8>)>> + '_ {
        let from_key = {
            let mut k = [0u8; 12];
            k[0..8].copy_from_slice(&from_block.to_be_bytes());
            k
        };
        let to_key = {
            let mut k = [0u8; 12];
            k[0..8].copy_from_slice(&(to_block + 1).to_be_bytes());
            k
        };

        self.chain_events
            .range(from_key.to_vec()..to_key.to_vec())
            .map(|result| {
                result.map(|kv| (kv.key.to_vec(), kv.value.to_vec()))
            })
    }
}
```

fjall vs RocksDB: 42K lines of safe Rust vs 700K+ lines of C++ via FFI. Build: 3.5 seconds vs 40+. Binary: 2.2 MB vs 12+ MB. Performance tradeoff acceptable for the Golem's event throughput.

### Parquet for cold archival

Data past the warm retention window goes to Parquet files with Zstd compression (~87% storage reduction). Cryo (Rust, paradigmxyz) extracts chain data to Parquet. DataFusion (Rust-native SQL engine) queries Parquet with predicate pushdown.

```rust
use datafusion::prelude::*;

async fn query_cold_episodes(
    parquet_dir: &str,
    protocol: &str,
    from_block: u64,
    to_block: u64,
) -> datafusion::error::Result<Vec<arrow::record_batch::RecordBatch>> {
    let ctx = SessionContext::new();
    ctx.register_parquet("episodes", parquet_dir, ParquetReadOptions::default())
        .await?;

    let df = ctx
        .sql(&format!(
            "SELECT episode_id, embedding, score, summary \
             FROM episodes \
             WHERE protocol = '{}' AND block_number BETWEEN {} AND {} \
             ORDER BY score DESC \
             LIMIT 100",
            protocol, from_block, to_block
        ))
        .await?;

    df.collect().await
}
```

### Storage decision matrix

| Data type | Access pattern | Engine | Reason |
|-----------|---------------|--------|--------|
| CorticalState | Read/write every Gamma | redb | ACID, low volume |
| Protocol state deltas | Write per block, read at Theta | redb | Range queries by (protocol, block) |
| Grimoire episode metadata | Write at Theta, read at Theta/Delta | redb | Structured queries |
| Raw chain event logs | Write per block (high volume) | fjall | Write-heavy, append-only |
| Triage scoring traces | Write every Gamma (high volume) | fjall | Write-heavy, bulk analysis |
| Episode embeddings (binary) | Write at Theta, query at Gamma | usearch | HNSW with Hamming distance |
| Episode embeddings (float) | Write at Theta, query at Gamma | LanceDB | Vector search + metadata filter |
| Archival episodes | Write at Delta (batch), read rarely | Parquet | Column pruning, compression |

### Storage budget (6 months, ~100 protocols)

| Tier | Engine | Estimated size |
|------|--------|---------------|
| Hot | redb | ~500 MB |
| Warm | fjall | ~2 GB |
| Vectors | usearch or LanceDB | ~250 MB |
| Cold | Parquet | ~5 GB |
| **Total** | | **~8 GB** |

### Cross-engine consistency

The tick architecture provides natural consistency boundaries. Within a Gamma tick: all redb writes in a single transaction. Within a Theta tick: fjall cross-keyspace batch for atomic event + trace writes. At Delta boundaries: consolidation reads both engines, produces Parquet, then removes archived entries after confirmation. No distributed transaction across engines needed.

---

## References

1. Kanerva, P. (2009). "Hyperdimensional Computing." *Cognitive Computation*, 1(2), 139-159. Established the theoretical foundations for computing with high-dimensional random vectors. The mathematical basis for all BSC operations used in the Grimoire's HDC layer.
2. Plate, T.A. (2003). *Holographic Reduced Representations*. CSLI Publications. Proved that holographic bundles have capacity O(sqrt(D)) for superimposed items. Grounds the ~100-item bundle capacity at D=10,240.
3. Gayler, R.W. (2004). "Vector Symbolic Architectures are a Viable Alternative for Jackendoff's Challenges." *Behavioral and Brain Sciences*, 27(3). Showed that VSAs can represent structured knowledge (role-filler bindings, sequences, recursive structures). Validates HDC for encoding market context.
4. Richards, B.A. & Frankland, P.W. (2017). "The Persistence and Transience of Memory." *Neuron*, 94(6), 1071-1084. Memory's purpose is decision optimization, not information preservation. Connects HDC's lossy bundling to the biological function of forgetting.
5. Frady, E.P., Kleyko, D., & Sommer, F.T. (2018). "Variable Binding for Sparse Distributed Representations." *IEEE Trans. Neural Networks*. Extended binding operations to sparse codes. Informs the tradeoffs in choosing binary (dense) vs. sparse HDC representations.
6. Imani, M., Kong, D., Rosing, T., & Rahimi, A. (2019). "VoiceHD: Hyperdimensional Computing for Efficient Speech Recognition." *IEEE ICRC*. Demonstrated HDC in production-scale classification tasks. Validates the feasibility of real-time HDC encoding in the Golem heartbeat loop.
7. Kleyko, D. et al. (2022). "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures." *ACM Computing Surveys*, 55(6). Comprehensive survey of VSA variants, operations, and applications. The reference for choosing BSC over alternatives.
8. Malkov, Y. and Yashunin, D. (2018). "Efficient and robust ANN search using HNSW graphs." *IEEE TPAMI*, 42(4). The algorithm behind the streaming ANN index used for HDC similarity queries. Sub-millisecond retrieval at scale.
9. Jayaram Subramanya, S. et al. (2019). "DiskANN: Fast Accurate Billion-point Nearest Neighbor Search." *NeurIPS 2019*. Graph-based ANN that fits on a single node. Informs the Grimoire's tiered storage engine design.
10. Singh, A. et al. (2021). "FreshDiskANN: Streaming Similarity Search." arXiv:2105.09613. Extended DiskANN for streaming inserts and deletes. Grounds the streaming ANN index for continuous episode ingestion.
11. Xu, H. et al. (2025). "In-Place Updates of a Graph Index for Streaming ANN Search." arXiv:2502.13826. Shows how to update graph-based indexes without full rebuilds. Enables the Grimoire's append-heavy write pattern.
12. Guo, R. et al. (2020). "Accelerating Large-Scale Inference with Anisotropic Vector Quantization." *ICML 2020*. Quantization technique for reducing storage and query cost. Relevant to the tiered storage engine's compression layer.
13. Chen, Q. et al. (2021). "SPANN: Billion-scale Approximate Nearest Neighborhood Search." *NeurIPS 2021*. Hybrid inverted index + graph approach for extreme-scale ANN. Informs future scaling of the Styx-side vector index.
14. Wang, Y. et al. (2025). "TANNS: Timestamp Approximate Nearest Neighbor Search." *ICDE 2025*. ANN search with temporal constraints. Directly relevant to the Grimoire's time-weighted retrieval.
15. Anonymous (2025). "Distribution-Aware Exploration for Adaptive HNSW Search (Ada-ef)." arXiv:2512.06636. Adaptive search that adjusts effort based on query difficulty. Potential optimization for the Grimoire's retrieval pipeline.

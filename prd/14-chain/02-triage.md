# Triage Pipeline [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `bardo-triage`

---

> **Reader orientation:** This document specifies the triage pipeline, the most architecturally dense component of Bardo's chain intelligence layer (section 14). It describes a 4-stage classification pipeline that scores every transaction a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) sees, using rule-based filters, streaming anomaly detection (MIDAS-R, DDSketch), hyperdimensional computing fingerprints, and Bayesian surprise. The key concept is the curiosity score: a Hedge-weighted composite that determines whether a transaction is noise, worth silent tracking, or worth escalating to the Golem's LLM-powered cognitive loop. See `prd2/shared/glossary.md` for full term definitions.

The triage pipeline is the golem's judgment layer for on-chain activity. For every block that passes the Binary Fuse pre-screen (see [01-witness.md](./01-witness.md)), the pipeline classifies each transaction: what is it, how interesting is it, and what should happen next. No LLM calls touch this path -- that's what makes it fast. LLM analysis happens asynchronously at Theta tick on high-scoring events only.

This is the most architecturally dense component of the chain intelligence system. It synthesizes four competing research directions -- rule-based heuristics, streaming anomaly detection, hyperdimensional computing, and Bayesian surprise -- into a unified 4-stage pipeline that runs at block rate.

Each chain gets its own `TriageEngine` instance with its own statistical structures (MIDAS-R, DDSketch, Count-Min Sketch). Cross-chain signals flow through the event fabric, not shared mutable state. This eliminates Mutex contention at aggregate throughput across multiple chains.

---

## Pipeline Overview

```
Block (full, with receipts)
    |
    v
Stage 1: Rule-based fast filters
    -- Known MEV patterns, known contracts, value thresholds
    -- Rejects >90% of transactions. Cost: O(1) per tx.
    |
    v
Stage 2: Statistical anomaly detection
    -- MIDAS-R for streaming graph anomalies
    -- DDSketch for distribution monitoring
    -- Count-Min Sketch for frequency spikes
    |
    v
Stage 3: Contextual enrichment
    -- Protocol state lookup
    -- ABI resolution (async, non-blocking)
    -- Historical comparison via ANN
    |
    v
Stage 4: Upgraded scoring
    -- HDC/BSC fingerprint generation (10K-dim binary vector)
    -- Bayesian surprise (KL divergence) -> feeds Oracle as PredictionDomain
    -- Two-tier ANN lookup (main + staging HNSW)
    -- Thompson sampling for exploration-exploitation
    -- Final curiosity score = Hedge-weighted combination
```

---

## The TriageEngine

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct TriageEngine {
    /// Stage implementations, executed in sequence.
    stages: Vec<Box<dyn TriageStage + Send + Sync>>,

    /// Curiosity scorer combines all Stage 4 signals.
    curiosity: Arc<CuriosityScorer>,

    /// Channel to protocol state engine for state updates.
    protocol_tx: mpsc::Sender<TriageEvent>,

    /// Channel to Theta tick for high-score LLM analysis.
    llm_queue: mpsc::Sender<TriageEvent>,

    /// redb handle for event persistence.
    store: Arc<TriageStore>,
}

/// Every stage implements this trait. Stages can filter (return None
/// to drop a transaction) or enrich (add metadata and pass through).
#[async_trait]
pub trait TriageStage: Send + Sync {
    /// Process a single transaction. Return None to drop it.
    async fn process(&self, ctx: &mut TriageContext) -> Option<()>;

    /// Human-readable stage name for metrics and tracing.
    fn name(&self) -> &'static str;
}

/// Accumulates metadata as a transaction passes through stages.
pub struct TriageContext {
    pub tx: NormalizedTx,
    pub receipt: TransactionReceipt,
    pub block: BlockMetadata,
    pub decoded_logs: Vec<DecodedLog>,
    pub category: TxCategory,
    pub enrichments: Vec<Enrichment>,
    pub scores: StageScores,
    pub fingerprint: Option<Hypervector>,
}
```

The engine feeds each transaction through stages sequentially. Any stage can drop a transaction by returning `None`. Survivors reach Stage 4 for final scoring and routing.

---

## Stage 1: Rule-Based Fast Filters

The first pass is pure determinism. O(1) per transaction, no allocations, no network calls.

```rust
pub struct RuleFilterStage {
    /// Known ABI event selectors: topic0 -> EventMetadata.
    /// Populated from ProtocolRegistry, refreshed at Gamma tick via RwLock swap.
    event_selector_map: Arc<RwLock<HashMap<B256, EventMetadata>>>,

    /// Watched addresses from ChainScope.
    watched_addresses: Arc<DashSet<Address>>,

    /// Known MEV bot bytecode hashes.
    mev_bytecode_hashes: HashSet<B256>,

    /// Large value threshold (modulated by behavioral phase).
    large_value_threshold_wei: AtomicU64,
}

#[async_trait]
impl TriageStage for RuleFilterStage {
    async fn process(&self, ctx: &mut TriageContext) -> Option<()> {
        // 1. Decode logs against known ABI selectors.
        for log in &ctx.receipt.logs {
            let topic0 = log.topics().first()?;
            if let Some(meta) = self.event_selector_map.read().get(topic0) {
                ctx.decoded_logs.push(DecodedLog {
                    log: log.clone(),
                    event_name: meta.event_name.clone(),
                    protocol_id: meta.protocol_id.clone(),
                });
            }
        }

        // 2. Address triage: does any involved address match scope?
        let from_watched = self.watched_addresses.contains(&ctx.tx.from);
        let to_watched = ctx.tx.to
            .map(|a| self.watched_addresses.contains(&a))
            .unwrap_or(false);
        let log_watched = ctx.decoded_logs.iter()
            .any(|l| self.watched_addresses.contains(&l.log.address));
        let has_decoded = !ctx.decoded_logs.is_empty();

        if !from_watched && !to_watched && !log_watched && !has_decoded {
            // No connection to anything the golem cares about.
            return None;
        }

        // 3. Fast category assignment for known patterns.
        if ctx.tx.creates.is_some() {
            ctx.category = TxCategory::ContractDeploy {
                deployer: ctx.tx.from,
                bytecode_hash: keccak256(&ctx.tx.input),
            };
        } else if self.mev_bytecode_hashes.contains(
            &ctx.tx.to.map(|a| keccak256_of_code(a)).unwrap_or_default()
        ) {
            ctx.category = TxCategory::MevActivity {
                pattern: detect_mev_pattern(&ctx.tx, &ctx.decoded_logs),
            };
        }

        Some(())
    }

    fn name(&self) -> &'static str { "rule_filter" }
}
```

The selector map is pre-populated at startup with selectors for the ~20-30 seed protocol families (Uniswap v2/v3/v4 Swap/Mint/Burn, Aave events, etc.). When new protocols are discovered, their selectors are added via `RwLock<HashMap>` refresh at the next Gamma tick.

---

## Stage 2: Statistical Anomaly Detection

This stage adds streaming statistical signals that detect patterns rule-based filters miss. All structures operate in constant memory with constant-time updates.

```rust
pub struct AnomalyDetectionStage {
    /// MIDAS-R: streaming graph anomaly detection.
    /// Models txs as (from, to, block) edge stream.
    /// Detects flash-loan cascades, wash trading, MEV coordination.
    midas: Mutex<MidasR>,

    /// DDSketch: gas price distribution tracker.
    /// Relative error guarantee: |x_hat - x_q| <= alpha * x_q.
    gas_sketch: Mutex<DDSketch>,

    /// DDSketch per watched protocol: swap value distributions.
    protocol_sketches: DashMap<ProtocolId, DDSketch>,

    /// Count-Min Sketch: address frequency estimation.
    /// Detects first-seen addresses and activity spikes.
    address_frequency: Mutex<AddressFrequencySketch>,
}

#[async_trait]
impl TriageStage for AnomalyDetectionStage {
    async fn process(&self, ctx: &mut TriageContext) -> Option<()> {
        // MIDAS-R anomaly score: model tx as directed edge.
        let midas_score = {
            let mut m = self.midas.lock();
            m.insert_edge(
                hash_addr(ctx.tx.from),
                hash_addr(ctx.tx.to.unwrap_or_default()),
                ctx.block.number,
            )
        };
        ctx.scores.midas_anomaly = midas_score;

        // Address frequency: detect spikes and first-seen addresses.
        let freq = {
            let mut sketch = self.address_frequency.lock();
            let to_addr = ctx.tx.to.unwrap_or_default();
            sketch.observe(&to_addr, ctx.block.number);
            sketch.estimate_frequency(&to_addr)
        };
        ctx.scores.address_novelty = if freq < 1.0 { 0.4 } else { 0.0 };
        ctx.scores.frequency_spike = if freq > sketch_baseline(&ctx.tx) * 3.0 {
            0.3
        } else {
            0.0
        };

        // Gas distribution: flag outliers.
        {
            let mut gs = self.gas_sketch.lock();
            gs.add(ctx.tx.gas_used as f64);
        }

        // Protocol-specific value distribution tracking.
        if let Some(protocol_id) = ctx.decoded_logs.first()
            .and_then(|l| l.protocol_id.as_ref())
        {
            let mut sketch = self.protocol_sketches
                .entry(protocol_id.clone())
                .or_insert_with(DDSketch::default);
            if let Some(value_usd) = extract_usd_value(&ctx.decoded_logs) {
                sketch.add(value_usd);
            }
        }

        // Stage 2 never drops -- it only enriches.
        Some(())
    }

    fn name(&self) -> &'static str { "anomaly_detection" }
}
```

### MIDAS-R

MIDAS (Bhatia et al., 2020) maintains two Count-Min Sketch structures -- total edge count and current-tick edge count -- and computes a chi-squared anomaly score comparing observed to expected frequency. It processes each edge in O(1) time and O(1) memory while achieving 42-52% higher AUC than prior streaming anomaly detectors. MIDAS-R extends this to node-level frequencies, catching both edge-specific and node-level bursts.

For the triage pipeline, model Ethereum transactions as `(from_address, to_address, block_number)` edge streams. MIDAS detects flash-loan cascades, wash trading microbursts, and MEV bot coordination as anomalous edge clusters.

**Wall-clock MIDAS-R ticks**: On fast chains (Base at 2s blocks), per-block edge counts drop below the chi-squared power threshold (Cochran, 1952, requires expected cell counts >= 5). Switch from `block_number` to wall-clock time windows: `tick = floor(timestamp / tick_duration)` where `tick_duration = max(block_time, 12s)`. This normalizes statistical density across chains. On Base, this aggregates ~6 blocks per MIDAS tick, restoring cell counts to the same range as Ethereum mainnet. On Ethereum, behavior is unchanged.

### DDSketch

DDSketch (Masson et al., 2019) uses logarithmic bucketing with a formal relative error guarantee: for any quantile q, the returned value x_hat satisfies `|x_hat - x_q| <= alpha * x_q`. This matters for heavy-tailed distributions like gas prices and swap values, where absolute-error guarantees are meaningless. Configured with alpha = 0.01 (1% relative error).

### Count-Min Sketch

The Count-Min Sketch (Cormode & Muthukrishnan, 2005) provides frequency estimation in O(1/epsilon * log(1/delta)) counters. A sliding-window variant with two sketches (current + previous, exponential decay) enables spike detection and first-seen address identification without maintaining full address sets.

```rust
pub struct AddressFrequencySketch {
    current: CountMinSketch<Address>,
    previous: CountMinSketch<Address>,
    window_blocks: u64,
    last_rotation_block: u64,
}

impl AddressFrequencySketch {
    pub fn observe(&mut self, addr: &Address, block: u64) {
        if block - self.last_rotation_block >= self.window_blocks {
            self.previous = self.current.clone();
            self.current.clear();
            self.last_rotation_block = block;
        }
        self.current.add(addr);
    }

    pub fn estimate_frequency(&self, addr: &Address) -> f32 {
        self.current.estimate(addr) as f32
            + self.previous.estimate(addr) as f32 * 0.5
    }
}
```

---

## Stage 3: Contextual Enrichment

For transactions that survived Stages 1-2, add context from the protocol state engine and ABI resolution chain. This stage involves async operations (eth_calls, HTTP lookups) but they never block the triage pipeline -- they run in background queues.

```rust
pub struct EnrichmentStage {
    protocol_state: Arc<ProtocolStateEngine>,
    abi_resolver: Arc<AbiResolver>,
    protocol_registry: Arc<ProtocolRegistry>,
}

#[async_trait]
impl TriageStage for EnrichmentStage {
    async fn process(&self, ctx: &mut TriageContext) -> Option<()> {
        let to_addr = ctx.tx.to?;

        // Protocol state lookup (zero-latency DashMap read).
        if let Some(state) = self.protocol_state.get_hot(ctx.block.chain_id, to_addr) {
            ctx.enrichments.push(Enrichment::ProtocolState(state));
        }

        // If the contract is unknown, queue for async fingerprinting.
        if !self.protocol_registry.contains(&to_addr) {
            self.fingerprint_unknown(to_addr, ctx).await;
        }

        // Assign protocol-aware category if possible.
        if let Some(decoded) = ctx.decoded_logs.first() {
            if let Some(protocol_id) = &decoded.protocol_id {
                ctx.category = TxCategory::ProtocolInteraction {
                    protocol_id: protocol_id.clone(),
                    action: decoded.event_name.clone(),
                    protocol_family: self.protocol_registry
                        .family_of(protocol_id)
                        .unwrap_or(ProtocolFamily::Unknown { bytecode_hash: B256::ZERO }),
                };
            }
        }

        Some(())
    }

    fn name(&self) -> &'static str { "enrichment" }
}

impl EnrichmentStage {
    /// Fingerprint an unknown contract. Non-blocking -- queues work
    /// for background resolution.
    async fn fingerprint_unknown(&self, address: Address, ctx: &mut TriageContext) {
        // 1. Bytecode hash lookup (fast, local).
        if let Some(known) = self.protocol_registry.lookup_bytecode_hash(address) {
            ctx.enrichments.push(Enrichment::KnownFamily(known));
            return;
        }

        // 2. Queue for async ABI resolution (non-blocking).
        // Resolution priority:
        //   Sourcify (~60% coverage, no key)
        //   -> Etherscan (~80% verified, optional key)
        //   -> 4byte.directory (selector fragments, no key)
        //   -> Heimdall-rs / WhatsABI (bytecode decompilation)
        self.abi_resolver.enqueue(AbiRequest {
            address,
            chain_id: ctx.block.chain_id,
        });
    }
}
```

ABI resolution is async and non-blocking. When an ABI resolves, the contract is added to `ProtocolRegistry` and its selectors enter the log decoder map at the next Gamma tick. Partial ABIs are useful -- even knowing just the function selector resolves `Unknown` triage entries to named interactions.

---

## Stage 4: Upgraded Scoring

This is where everything converges. Five scoring signals combine through a Hedge-weighted compositor with Thompson sampling for exploration-exploitation.

### Signal 1: HDC/BSC Fingerprint

Transaction fingerprints are 10,000-dimensional Binary Spatter Code vectors (Kanerva, 2009). See [shared/hdc-vsa.md](../shared/hdc-vsa.md) for the BSC algebra. Each transaction is encoded compositionally using role-filler binding:

```rust
use golem_core::hdc::{Hypervector, BundleAccumulator, ItemMemory};

pub struct HdcTxEncoder {
    /// Codebook: maps named concepts to fixed random hypervectors.
    codebook: Mutex<ItemMemory>,

    /// Role vectors for structural positions.
    role_from: Hypervector,
    role_to: Hypervector,
    role_selector: Hypervector,
    role_value_bucket: Hypervector,
    role_log_topic: Hypervector,
    role_gas_bucket: Hypervector,
}

impl HdcTxEncoder {
    /// Encode a transaction as a 10K-dim binary hypervector.
    /// Deterministic: the same transaction always produces the same vector.
    /// Cost: O(D * num_fields) where D = 10,240 bits. ~1us total.
    pub fn encode(&self, tx: &NormalizedTx, logs: &[DecodedLog]) -> Hypervector {
        let mut codebook = self.codebook.lock();

        let from_hv = codebook.encode(&format!("addr:{}", tx.from));
        let to_hv = codebook.encode(&format!(
            "addr:{}", tx.to.unwrap_or_default()
        ));
        let sel_hv = codebook.encode(&format!(
            "sel:{}", hex::encode(&tx.input[..4.min(tx.input.len())])
        ));
        let value_hv = codebook.encode(&format!(
            "val_bucket:{}", value_bucket(tx.value)
        ));
        let gas_hv = codebook.encode(&format!(
            "gas_bucket:{}", gas_bucket(tx.gas_used)
        ));

        // Bind each filler to its role, then bundle (superpose).
        let mut acc = BundleAccumulator::new();
        acc.add(&from_hv.bind(&self.role_from));
        acc.add(&to_hv.bind(&self.role_to));
        acc.add(&sel_hv.bind(&self.role_selector));
        acc.add(&value_hv.bind(&self.role_value_bucket));
        acc.add(&gas_hv.bind(&self.role_gas_bucket));

        // Encode log structure: each topic bound to its position.
        for (i, log) in logs.iter().enumerate() {
            if let Some(topic0) = log.log.topics().first() {
                let topic_hv = codebook.encode(&format!("topic:{}", topic0));
                let pos_hv = topic_hv.permute(i); // positional encoding
                acc.add(&pos_hv.bind(&self.role_log_topic));
            }
        }

        acc.finish()
    }
}
```

The HDC encoding is algebraically composable and invertible. Two transactions involving the same addresses in different roles produce different but related fingerprints -- the Hamming similarity reflects structural kinship. New feature types (block position, MEV pattern, address cluster) are added without changing the vector format -- just add more binding terms.

At D = 10,240, each fingerprint is 1,280 bytes. Comparison (Hamming distance via XOR + POPCNT) takes ~10 nanoseconds with SIMD. The ANN index for 10,000 Grimoire episodes at this dimensionality is ~12.5 MB.

Use logarithmic quantization (8-12 buckets per log decade) for gas and value fields. A transaction worth 0.001 ETH and one worth 1000 ETH should not occupy adjacent buckets. Log-scale bucketing produces more informative fingerprints because it matches the scale-free distribution of DeFi transaction values.

### Signal 2: Bayesian Surprise

Bayesian surprise (Itti & Baldi, 2009) measures how much an observation shifts the golem's beliefs. Surprise = KL divergence from prior to posterior. Unlike Shannon information, surprise filters out irreducible randomness -- only data that genuinely updates the model scores high.

Bayesian surprise feeds INTO the Oracle (see [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md)) as a `PredictionDomain`. The Oracle is canonical for prediction error and action gating. The KL divergence output is one of the Oracle's prediction domain inputs, not a parallel prediction system.

```rust
pub struct BayesianSurprise {
    /// Per-protocol conjugate models.
    /// Gamma-Poisson for event rates.
    /// Normal-Inverse-Gamma for value distributions.
    /// Beta-Bernoulli for binary features.
    protocol_models: DashMap<ProtocolId, ProtocolBelief>,
}

pub struct ProtocolBelief {
    /// Event rate: Gamma(alpha, beta) prior.
    /// Observe k events -> posterior Gamma(alpha+k, beta+1).
    event_rate: GammaPoisson,

    /// Value distribution: Normal-Inverse-Gamma prior.
    value_dist: NormalInverseGamma,

    /// Binary features: Beta(a, b) priors.
    liquidation_occurred: BetaBernoulli,
    large_swap: BetaBernoulli,
}

impl BayesianSurprise {
    /// Compute surprise for a triage event.
    /// Returns KL divergence from prior to posterior.
    /// Cost: O(1) -- pure parameter arithmetic with conjugate priors.
    pub fn compute(&self, event: &TriageContext) -> f32 {
        let protocol_id = match &event.category {
            TxCategory::ProtocolInteraction { protocol_id, .. } => protocol_id,
            _ => return 0.0,  // no model for non-protocol txs
        };

        let mut belief = self.protocol_models
            .entry(protocol_id.clone())
            .or_insert_with(ProtocolBelief::default);

        // Compute KL divergence before updating.
        let rate_surprise = belief.event_rate.kl_divergence_on_observe(1);
        let value_surprise = extract_usd_value(&event.decoded_logs)
            .map(|v| belief.value_dist.kl_divergence_on_observe(v))
            .unwrap_or(0.0);

        // Update posterior (the observation becomes the new prior).
        belief.event_rate.observe(1);
        if let Some(v) = extract_usd_value(&event.decoded_logs) {
            belief.value_dist.observe(v);
        }

        rate_surprise + value_surprise
    }
}

/// Gamma-Poisson conjugate model for event rate estimation.
struct GammaPoisson {
    alpha: f64,  // shape
    beta: f64,   // rate
}

impl GammaPoisson {
    fn kl_divergence_on_observe(&self, k: u64) -> f32 {
        // KL[Gamma(alpha+k, beta+1) || Gamma(alpha, beta)]
        // Analytic formula via digamma functions.
        let a_post = self.alpha + k as f64;
        let b_post = self.beta + 1.0;
        let kl = (a_post - self.alpha) * digamma(a_post)
            - ln_gamma(a_post) + ln_gamma(self.alpha)
            + self.alpha * (b_post / self.beta).ln()
            + a_post * (self.beta - b_post) / b_post;
        kl.abs() as f32
    }

    fn observe(&mut self, k: u64) {
        self.alpha += k as f64;
        self.beta += 1.0;
    }
}
```

### Signal 3: Two-Tier ANN Lookup

The HNSW index has two tiers: a main index (rebuilt at Delta-level consolidation) and a staging index that accepts inserts at Theta tick when new Grimoire episodes are stored. New episodes become queryable at the next Gamma tick, not at the next Delta.

```rust
pub struct CuriosityAnnIndex {
    /// Main index: rebuilt from scratch periodically.
    main: usearch::Index,

    /// Staging: accepts in-place inserts since last rebuild.
    staging: usearch::Index,

    /// Number of episodes in staging. Merge at threshold.
    staging_count: AtomicUsize,

    /// Merge staging into main when this threshold is reached.
    staging_threshold: usize,  // default: 500
}

impl CuriosityAnnIndex {
    /// Called at Theta tick when a new GrimoireEpisode is stored.
    pub fn insert_episode(&self, id: u64, embedding: &[f32]) {
        self.staging.add(id, embedding).ok();
        self.staging_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Query both tiers, merge, return top-k.
    /// Cost: ~50us for k=10 at 10,000 episodes.
    pub fn query(&self, embedding: &[f32], k: usize) -> Vec<(u64, f32)> {
        let main_results = self.main.search(embedding, k).unwrap_or_default();
        let staging_results = self.staging.search(embedding, k).unwrap_or_default();

        let mut combined: Vec<(u64, f32)> = main_results.keys.iter()
            .zip(main_results.distances.iter())
            .chain(staging_results.keys.iter().zip(staging_results.distances.iter()))
            .map(|(&id, &dist)| (id, dist))
            .collect();

        combined.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        combined.dedup_by_key(|x| x.0);
        combined.truncate(k);
        combined
    }

    /// Merge staging into main. Called when staging_count >= threshold.
    /// Runs at Delta tick cadence. ~100ms at 10K episodes.
    pub fn merge_staging(&mut self) {
        // Rebuild main from all episodes (main + staging).
        self.staging_count.store(0, Ordering::Relaxed);
    }
}
```

This tightens the curiosity loop from `chain event -> Theta LLM -> Grimoire episode -> (Delta wait) -> ANN query` to `chain event -> Theta LLM -> Grimoire episode -> (next Gamma) -> ANN query`.

### Signal 4: Thompson Sampling for Exploration-Exploitation

Fixed score thresholds waste the golem's attention budget. Thompson sampling (Thompson, 1933; Russo et al., 2017) maintains Beta posteriors over each routing bucket's "is this event actually relevant?" probability and samples from them to decide routing.

```rust
pub struct ThompsonRouter {
    /// Beta(alpha, beta) posteriors per routing action.
    /// Updated by LLM feedback at Theta tick.
    actions: [BetaPosterior; 4],  // escalate, emit, silent, discard
}

struct BetaPosterior {
    alpha: f64,  // successes + 1
    beta: f64,   // failures + 1
}

impl ThompsonRouter {
    /// Sample from posteriors and route based on drawn probabilities.
    /// This naturally explores uncertain regions of the score space.
    pub fn route(&self, score: f32, rng: &mut impl Rng) -> RoutingDecision {
        // Sample expected relevance for each action.
        let samples: Vec<f64> = self.actions.iter()
            .map(|a| Beta::new(a.alpha, a.beta).unwrap().sample(rng))
            .collect();

        // Select action with highest sampled value, weighted by score.
        if score > 0.7 && samples[0] > samples[3] {
            RoutingDecision::Escalate  // -> TriageAlert + LLM queue
        } else if score > 0.4 && samples[1] > samples[3] {
            RoutingDecision::Emit      // -> ChainEvent + protocol state
        } else if score > 0.15 {
            RoutingDecision::Silent    // -> protocol state only
        } else {
            RoutingDecision::Discard   // -> redb audit only
        }
    }

    /// Update posteriors based on LLM feedback.
    /// Called at Theta tick after analyze_chain_event().
    pub fn update(&mut self, action: usize, was_relevant: bool) {
        if was_relevant {
            self.actions[action].alpha += 1.0;
        } else {
            self.actions[action].beta += 1.0;
        }
    }
}
```

### Signal 5: Contrastive Embedding Similarity (80-dim Fallback)

The handcrafted 80-dimensional fingerprint serves as a cold-start fallback before the HDC codebook and ANN index have accumulated enough data:

```rust
pub fn compute_fallback_fingerprint(tx: &NormalizedTx, logs: &[DecodedLog]) -> [f32; 80] {
    let mut fp = [0.0f32; 80];

    // one_hot(to_type, 16): known protocol family
    if let Some(family_idx) = protocol_family_index(tx.to) {
        fp[family_idx] = 1.0;
    }

    // hash_truncate(func_selector, 8): function selector
    let sel = &tx.input[..4.min(tx.input.len())];
    for (i, byte) in sel.iter().enumerate().take(8) {
        fp[16 + i] = *byte as f32 / 255.0;
    }

    // log_buckets(value_usd, 8): value magnitude
    let value_log = (tx.value.to::<f64>() + 1.0).log10();
    let bucket = (value_log * 8.0 / 20.0).min(7.0) as usize;
    fp[24 + bucket] = 1.0;

    // topic_hashes(log_topics, 32): event topic fingerprints
    for (i, log) in logs.iter().enumerate().take(4) {
        if let Some(topic) = log.log.topics().first() {
            for j in 0..8 {
                fp[32 + i * 8 + j] = topic.0[j] as f32 / 255.0;
            }
        }
    }

    // gas_bucket(gas_used, 8): computational complexity
    let gas_log = (tx.gas_used as f64 + 1.0).log10();
    let gas_bucket = (gas_log * 8.0 / 8.0).min(7.0) as usize;
    fp[64 + gas_bucket] = 1.0;

    // from_cluster(from_addr, 8): address cluster
    for (i, byte) in tx.from.0[..8].iter().enumerate() {
        fp[72 + i] = *byte as f32 / 255.0;
    }

    fp
}
```

### Combining Signals: The Hedge Compositor

The Hedge/Exponential Weights algorithm (Shalev-Shwartz, 2011) combines all five signals with learned weights that converge to the best fixed weighting in hindsight, guaranteeing O(sqrt(T)) regret.

Use Discounted Hedge (Herbster & Warmuth, 1998, also called Fixed-Share): before the multiplicative weight update, apply `weights *= discount` where discount = 0.99. This gives weights a half-life of ~69 rounds, ensuring full regime adaptation within ~200 triage events. The standard Hedge's adaptive eta (`eta = sqrt(ln(N) / t)`) decreases over time, making it *less* responsive to distribution shifts -- the opposite of what a non-stationary DeFi market needs.

```rust
pub struct CuriosityScorer {
    hdc_encoder: HdcTxEncoder,
    bayesian: BayesianSurprise,
    ann_index: CuriosityAnnIndex,
    thompson: ThompsonRouter,
    midas: Arc<MidasR>,

    /// Hedge weights over scoring signals.
    /// Updated at Theta tick based on LLM evaluation feedback.
    weights: [f64; 5],
    learning_rate: f64,  // eta = 0.1

    /// Discount factor for non-stationary Hedge (Herbster & Warmuth 1998).
    /// Applied before each weight update. Default: 0.99.
    discount: f64,

    /// Grimoire episode count, for cold-start heuristic weight.
    episode_count: AtomicUsize,
}

impl CuriosityScorer {
    pub fn score(&self, ctx: &mut TriageContext) -> f32 {
        let episode_count = self.episode_count.load(Ordering::Relaxed);

        // Signal 1: HDC fingerprint similarity to interesting/boring bundles.
        let hv = self.hdc_encoder.encode(&ctx.tx, &ctx.decoded_logs);
        ctx.fingerprint = Some(hv);
        let hdc_score = self.hdc_codebook_score(&hv);

        // Signal 2: Bayesian surprise (KL divergence).
        let surprise = self.bayesian.compute(ctx);

        // Signal 3: ANN similarity to past chain_event episodes.
        let ann_score = if episode_count > 10 {
            let fallback_fp = compute_fallback_fingerprint(&ctx.tx, &ctx.decoded_logs);
            let results = self.ann_index.query(&fallback_fp, 5);
            results.first().map(|(_, dist)| 1.0 - dist).unwrap_or(0.0)
        } else {
            0.0
        };

        // Signal 4: MIDAS anomaly score (already computed in Stage 2).
        let midas_score = ctx.scores.midas_anomaly;

        // Signal 5: Heuristic score (rule-based, cold-start dominant).
        let heuristic = self.score_heuristic(ctx);

        // Cold-start weighting: heuristics dominate early.
        let cold_start_factor = 1.0 - (episode_count as f32 / 1000.0).min(0.6);

        // Hedge-weighted combination.
        let signals = [hdc_score, surprise, ann_score, midas_score, heuristic];
        let weighted: f32 = signals.iter()
            .zip(self.weights.iter())
            .map(|(s, w)| s * *w as f32)
            .sum();

        // Blend with cold-start heuristic.
        let final_score = heuristic * cold_start_factor
            + weighted * (1.0 - cold_start_factor);

        final_score.clamp(0.0, 1.0)
    }

    /// Update Hedge weights from LLM feedback.
    /// Called at Theta tick after analyze_chain_event().
    pub fn update_weights(&mut self, signals: &[f32; 5], actual_importance: f32) {
        // Discounted Hedge: decay weights before update for non-stationarity.
        for w in &mut self.weights {
            *w *= self.discount;
        }

        for (i, &signal) in signals.iter().enumerate() {
            let loss = (signal - actual_importance).abs() as f64;
            self.weights[i] *= (-self.learning_rate * loss).exp();
        }
        // Normalize.
        let sum: f64 = self.weights.iter().sum();
        for w in &mut self.weights {
            *w /= sum;
        }
    }
}
```

---

## TxCategory

Categories are dynamically derived, not hardcoded:

```rust
pub enum TxCategory {
    ProtocolInteraction {
        protocol_id: ProtocolId,
        action: String,         // "Swap", "AddLiquidity", "Borrow", etc.
        protocol_family: ProtocolFamily,
    },
    TokenTransfer {
        token: Address,
        direction: Direction,   // In | Out | Internal
        value_usd: Option<f64>,
    },
    LargeValueMove {
        usd_estimate: f64,
        from_cluster: AddressCluster,
        to_cluster: AddressCluster,
    },
    NewProtocolDeployed {
        factory: Address,
        child: Address,
        family: ProtocolFamily,
    },
    ContractDeploy {
        deployer: Address,
        bytecode_hash: B256,
    },
    MevActivity {
        pattern: MevPattern,    // Sandwich | Backrun | Frontrun | Arbitrage
    },
    Unknown,
}
```

For unknown contracts, category defaults to `Unknown` until the ABI resolver populates the registry. At that point, historical events from that contract in redb get retroactively re-categorized at the next Delta tick.

---

## Self-Learning Curiosity Loop

The curiosity model improves through the golem's own behavior, without external labels:

**Positive reinforcement** -- When the golem takes an action (trade, rebalance, position alert) triggered by a chain event, triage retrieves that event's fingerprint from redb and submits it for upweighting.

**Negative reinforcement** -- When the golem explicitly dismisses a queued high-score event at Theta cognition, the fingerprint is tagged negative and downweights similar future events.

**Epsilon-greedy exploration** -- With probability epsilon (decaying but never zero), a random low-score event is escalated to LLM analysis. This provides counterfactual labels for events the golem would otherwise never evaluate, preventing the partial monitoring bias where the golem can't learn that it's discarding interesting events.

```
[TriageAlert emitted]
    |
    v
[Theta: LLM analyzes with positions + strategy + recent Grimoire]
    |
    v
[Grimoire episode stored: ChainEvent, with embedding]
    |
    v
[Staging ANN index: insert immediately]
    |
    v
[Next Gamma: scoring uses updated index]
    |
    v
[Better-scored events -> better routing -> more relevant alerts]
    ^ (loop)
```

---

## Routing

| Score range | Action |
|-------------|--------|
| > 0.8 (or Thompson samples escalation) | `GolemEvent::TriageAlert` + add to Theta LLM queue |
| 0.5-0.8 | `GolemEvent::ChainEvent` + update protocol state |
| 0.2-0.5 | Update protocol state silently (no event emitted) |
| < 0.2 | Discard (written to redb for gap audit only) |

---

## Storage

```
Table "triage_events" (redb):
  key:   (chain_id: u32, block_number: u64, tx_index: u32)
  value: TxRecord { tx_hash, category, score, fingerprint_hash, timestamp_ms }

Table "triage_index_by_score" (redb):
  key:   (score_bucket: u8, timestamp_ms: u64)
  value: (chain_id: u32, block_number: u64, tx_index: u32)

Table "negative_fingerprints" (redb):
  key:   fingerprint_hash: B256
  value: downweight_factor: f32
```

Retention: 7 days by default. Compacted at each Delta tick. In `declining` behavioral phase, retention halves automatically. Low-score events (< 0.2) carry a `discarded: true` flag for gap detection only -- excluded from retroactive re-scoring.

---

## Emitted GolemEvents

```rust
GolemEvent::ChainEvent {
    chain_id: u64,
    tx_hash: String,
    block_number: u64,
    category: String,
    protocol_id: Option<String>,
    curiosity_score: f32,
    summary: Option<String>,  // populated by LLM at Theta tick
},

GolemEvent::TriageAlert {
    chain_id: u64,
    tx_hash: String,
    block_number: u64,
    category: String,
    score: f32,
    reason: String,  // human-readable: "active position counterparty"
},
```

See [06-events-signals.md](./06-events-signals.md) for wire format and subscription categories.

---

## Dependencies

```toml
[dependencies]
alloy = { version = "1" }
alloy-primitives = "1"
redb = "2"
usearch = "2"
dashmap = "6"
xorf = "0.11"
xxhash-rust = { version = "0.8", features = ["xxh3"] }
midas_rs = "0.1"               # MIDAS-R streaming anomaly detection
sketches-ddsketch = "0.2"      # DDSketch distribution tracking
streaming_algorithms = "0.3"   # Count-Min Sketch + Top-K
reqwest = { version = "0.12", features = ["json"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
rand = "0.8"
rand_distr = "0.4"             # Beta distribution for Thompson sampling
```

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Five-crate overview of chain intelligence, data flow from WebSocket to storage, cybernetic feedback loop
- **Witness**: [01-witness.md](./01-witness.md) -- Upstream block ingestion via WebSocket, Binary Fuse pre-screening that feeds blocks into this pipeline
- **Protocol state**: [03-protocol-state.md](./03-protocol-state.md) -- Downstream consumer: maintains live protocol state from triage events, autonomous discovery
- **Chain scope**: [04-chain-scope.md](./04-chain-scope.md) -- Source of watched addresses for Stage 1 filtering; rebuilt each Gamma tick from CorticalState
- **Heartbeat integration**: [05-heartbeat-integration.md](./05-heartbeat-integration.md) -- Theta tick LLM analysis of high-score events, Delta tick compaction and ANN merge
- **Events**: [06-events-signals.md](./06-events-signals.md) -- ChainEvent and TriageAlert GolemEvent variants emitted by this pipeline
- **Anomaly detection**: [09-anomaly-detection.md](./09-anomaly-detection.md) -- Full MIDAS-R, BOCPD, PELT, and ADWIN algorithm specifications
- **Oracle / Prediction engine**: [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md) -- Bayesian surprise KL divergence feeds in as a PredictionDomain; Oracle handles non-stationarity
- **HDC/VSA**: [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- BSC algebra, codebook management, ANN integration for hyperdimensional fingerprints

---

## References

- Bhatia, S. et al. (2020). MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams. arXiv:1911.04464. *AAAI 2020*. -- *Streaming graph anomaly detection via Count-Min Sketch; used in Stage 2 to detect transaction bursts between address pairs.*
- Cochran, W.G. (1952). The chi-squared test of goodness of fit. *Annals of Mathematical Statistics*, 23(3), 315-345. -- *Statistical test underlying the MIDAS-R anomaly score computation.*
- Cormode, G. & Muthukrishnan, S. (2005). An improved data stream summary: the Count-Min Sketch. *Journal of Algorithms*. -- *The probabilistic frequency estimator used for selector frequency tracking and MIDAS internals.*
- Graf, T.M. & Lemire, D. (2020). Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters. arXiv:1912.08258. -- *Predecessor to Binary Fuse filters; establishes the immutable-rebuild pattern.*
- Herbster, M. & Warmuth, M.K. (1998). Tracking the Best Expert. *Machine Learning*, 32(2), 151-178. -- *Fixed-share variant of the Hedge algorithm used to weight scoring signals adaptively.*
- Itti, L. & Baldi, P. (2009). Bayesian surprise attracts human attention. *Vision Research*, 49(10), 1295-1306. -- *Formalizes surprise as KL divergence between prior and posterior; the theoretical basis for curiosity scoring.*
- Kanerva, P. (2009). Hyperdimensional Computing: An Introduction. *Cognitive Computation*, 1(2). -- *Foundational work on high-dimensional distributed representations used in HDC fingerprinting.*
- Kleyko, D. et al. (2023). A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures. *ACM Computing Surveys*, 55(9). arXiv:2111.06077. -- *Comprehensive survey of VSA/HDC techniques; covers the BSC algebra and binding operations used here.*
- Masson, C. et al. (2019). DDSketch: A Fast and Fully-Mergeable Quantile Sketch with Relative-Error Guarantees. *PVLDB*, 12(12). -- *The quantile sketch used for distribution monitoring in Stage 2; mergeable across chains at Delta tick.*
- Russo, D. et al. (2017). A Tutorial on Thompson Sampling. arXiv:1707.02038. -- *Tutorial on the Bayesian exploration-exploitation strategy used for triage escalation decisions.*
- Shalev-Shwartz, S. (2011). Online Learning and Online Convex Optimization. *Foundations and Trends in Machine Learning*, 4(2), 107-194. -- *Theoretical framework for the Hedge algorithm that weights scoring signals.*
- Thompson, W.R. (1933). On the likelihood that one unknown probability exceeds another. *Biometrika*. -- *Original Thompson sampling paper; the bandit algorithm used to balance exploration vs. exploitation in triage.*

---

## Appendix A: Full Streaming Sketch Implementations

The triage pipeline uses four sketch families for bounded-memory analytics. All operate in O(1) time per update and O(1) memory. Total memory budget across all sketches: under 1 MB. This appendix provides the complete Rust implementations referenced by Stage 2.

### A.1 Count-Min Sketch with Conservative Update

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A Count-Min Sketch with Conservative Update.
/// Dimensions: depth x width, each cell a u64 counter.
pub struct CountMinSketch {
    depth: usize,
    width: usize,
    table: Vec<Vec<u64>>,
    seeds: Vec<u64>,
    total: u64,
}

impl CountMinSketch {
    pub fn new(depth: usize, width: usize) -> Self {
        let seeds: Vec<u64> = (0..depth).map(|i| 0xBEEF_CAFE_u64.wrapping_add(i as u64)).collect();
        Self {
            depth,
            width,
            table: vec![vec![0u64; width]; depth],
            seeds,
            total: 0,
        }
    }

    fn hash_at(&self, row: usize, key: &[u8]) -> usize {
        let mut hasher = DefaultHasher::new();
        self.seeds[row].hash(&mut hasher);
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.width
    }

    /// Conservative Update: only increment to min + 1, avoiding overcount.
    pub fn add(&mut self, key: &[u8]) {
        let min_val = self.estimate(key);
        for row in 0..self.depth {
            let col = self.hash_at(row, key);
            if self.table[row][col] <= min_val {
                self.table[row][col] = min_val + 1;
            }
        }
        self.total += 1;
    }

    pub fn estimate(&self, key: &[u8]) -> u64 {
        (0..self.depth)
            .map(|row| {
                let col = self.hash_at(row, key);
                self.table[row][col]
            })
            .min()
            .unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn clear(&mut self) {
        for row in &mut self.table {
            row.fill(0);
        }
        self.total = 0;
    }
}
```

At epsilon = 0.001 and delta = 0.01, the structure needs roughly 2,718 columns and 5 rows -- about 106 KB with 64-bit counters. The Conservative Update variant improves accuracy on Zipfian distributions (the distribution that address activity follows on Ethereum) by only incrementing each counter to max(current, min_across_rows + 1).

### A.2 Sliding-Window Frequency Tracker

A single CMS gives cumulative counts. For relative frequency detection, the sliding-window approach uses two sketches:

```rust
/// Sliding-window frequency tracker using two CMS instances.
/// Current window accumulates counts; previous window provides baseline.
/// At each rotation, current becomes previous and a fresh sketch starts.
pub struct SlidingFrequencySketch {
    current: CountMinSketch,
    previous: CountMinSketch,
    window_blocks: u64,
    last_rotation_block: u64,
}

impl SlidingFrequencySketch {
    pub fn new(depth: usize, width: usize, window_blocks: u64) -> Self {
        Self {
            current: CountMinSketch::new(depth, width),
            previous: CountMinSketch::new(depth, width),
            window_blocks,
            last_rotation_block: 0,
        }
    }

    /// Record an observation. Rotates windows when enough blocks have passed.
    pub fn observe(&mut self, key: &[u8], block_number: u64) {
        if block_number.saturating_sub(self.last_rotation_block) >= self.window_blocks {
            std::mem::swap(&mut self.previous, &mut self.current);
            self.current.clear();
            self.last_rotation_block = block_number;
        }
        self.current.add(key);
    }

    /// Weighted frequency: current window at full weight, previous at 50%.
    pub fn frequency(&self, key: &[u8]) -> f64 {
        self.current.estimate(key) as f64
            + self.previous.estimate(key) as f64 * 0.5
    }

    /// Spike ratio: current frequency vs previous window baseline.
    /// Returns None if previous window has no data for this key.
    pub fn spike_ratio(&self, key: &[u8]) -> Option<f64> {
        let prev = self.previous.estimate(key);
        if prev == 0 {
            return None;
        }
        Some(self.current.estimate(key) as f64 / prev as f64)
    }
}
```

The frequency sketch feeds two signals into the curiosity scorer:

1. **Novelty signal** -- an address with estimated frequency 0 is first-seen. Assign a flat curiosity bonus (e.g., 0.3).
2. **Spike signal** -- an address appearing at 3x its rolling baseline gets a curiosity boost (e.g., 0.2). Spikes often indicate arbitrage bursts, liquidation cascades, or exploit probing.

The Learned Space Saving (LSS) variant from Shahout and Mitzenmacher (2024) improves heavy-hitter identification by giving ML-predicted heavy hitters dedicated exact counters while the long tail stays in the sketch. This is a future optimization path.

### A.3 Full MIDAS-R Implementation

```rust
/// MIDAS-R anomaly detector for the transaction graph stream.
/// Uses three CMS instances: edge-level total, edge-level current tick,
/// and node-level current tick.
pub struct MidasR {
    /// Total edge counts across all ticks
    edge_total: CountMinSketch,
    /// Edge counts in current tick only
    edge_current: CountMinSketch,
    /// Node (source) counts in current tick
    node_current: CountMinSketch,
    /// Current tick identifier (block number or Gamma tick)
    current_tick: u64,
    /// Total ticks observed
    tick_count: u64,
    /// Temporal decay factor applied at each tick boundary
    decay: f64,
}

impl MidasR {
    pub fn new(depth: usize, width: usize, decay: f64) -> Self {
        Self {
            edge_total: CountMinSketch::new(depth, width),
            edge_current: CountMinSketch::new(depth, width),
            node_current: CountMinSketch::new(depth, width),
            current_tick: 0,
            tick_count: 0,
            decay,
        }
    }

    /// Process a new edge and return its anomaly score.
    /// Higher scores indicate more anomalous edges.
    pub fn score_edge(&mut self, source: &[u8], dest: &[u8], tick: u64) -> f64 {
        // Advance tick if needed
        if tick > self.current_tick {
            self.advance_tick(tick);
        }

        // Build the edge key by concatenating source and dest
        let mut edge_key = Vec::with_capacity(source.len() + dest.len());
        edge_key.extend_from_slice(source);
        edge_key.extend_from_slice(dest);

        // Update counts
        self.edge_total.add(&edge_key);
        self.edge_current.add(&edge_key);
        self.node_current.add(source);

        // Chi-squared score for edge
        let a = self.edge_current.estimate(&edge_key) as f64;
        let s = self.edge_total.estimate(&edge_key) as f64;
        let expected = s / (self.tick_count.max(1) as f64);

        let edge_score = if expected > 0.0 {
            (a - expected).powi(2) / expected
        } else {
            0.0
        };

        // Node-level burst score
        let node_count = self.node_current.estimate(source) as f64;
        let node_total = self.edge_total.estimate(source) as f64;
        let node_expected = node_total / (self.tick_count.max(1) as f64);

        let node_score = if node_expected > 0.0 {
            (node_count - node_expected).powi(2) / node_expected
        } else {
            0.0
        };

        // Combined score: max of edge and node anomaly
        edge_score.max(node_score)
    }

    fn advance_tick(&mut self, new_tick: u64) {
        self.edge_current.clear();
        self.node_current.clear();
        self.current_tick = new_tick;
        self.tick_count += 1;
    }
}
```

The scoring formula for an edge (u, v) at tick t: `score(u, v, t) = (a_uv(t) - s_uv * t_current / T)^2 / (s_uv * t_current / T)` where `a_uv(t)` is the current-tick count, `s_uv` is the total count, `t_current` is the current tick duration, and `T` is total time.

MIDAS detects: flash-loan cascades (burst of edges from single source to many targets within one block), wash trading (repeated edges between the same pair at abnormal rates), and MEV bot coordination (multiple bots sending edges to the same target simultaneously).

The Adaptive-GraphSketch (Ekle and Eberle, 2025) advances the state of the art with a 3D tensor sketch incorporating temporal decay, Bayesian posterior scoring, and EWMA-based adaptive thresholding -- achieving up to 6.5% AUC gain over MIDAS-R. Its temporal binning maps naturally to Gamma ticks, but no Rust implementation exists yet.

### A.4 DDSketch Gas Distribution Tracker

```rust
// Cargo.toml dependency:
// sketches-ddsketch = "0.3"

use sketches_ddsketch::DDSketch;

/// Gas price distribution tracker using DDSketch.
/// Provides relative-error quantile estimates with mergeability.
pub struct GasDistributionTracker {
    /// Current Gamma window sketch
    current: DDSketch,
    /// Accumulated Theta window sketch (merged from Gamma windows)
    theta_accumulated: DDSketch,
    /// Configuration: relative accuracy
    alpha: f64,
}

impl GasDistributionTracker {
    pub fn new(alpha: f64) -> Self {
        let config = sketches_ddsketch::Config::new(alpha, 2048, 1.0e-9);
        Self {
            current: DDSketch::new(config),
            theta_accumulated: DDSketch::new(config),
            alpha,
        }
    }

    /// Record a gas price observation.
    pub fn observe(&mut self, gas_price_gwei: f64) {
        self.current.add(gas_price_gwei);
    }

    /// Query a quantile from the current Gamma window.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        self.current.quantile(q).ok()
    }

    /// Merge current window into the Theta accumulator and reset.
    /// Called at each Gamma tick boundary.
    pub fn rotate_gamma(&mut self) {
        self.theta_accumulated.merge(&self.current).ok();
        let config = sketches_ddsketch::Config::new(self.alpha, 2048, 1.0e-9);
        self.current = DDSketch::new(config);
    }

    /// Get Theta-level quantile from accumulated Gamma windows.
    pub fn theta_quantile(&self, q: f64) -> Option<f64> {
        self.theta_accumulated.quantile(q).ok()
    }
}
```

DDSketch bucket boundaries follow a geometric progression: bucket i covers `[gamma^i, gamma^(i+1))` where `gamma = (1 + alpha) / (1 - alpha)`. This guarantees relative error on heavy-tailed distributions. DDSketch is fully mergeable: merge per-Gamma sketches at Theta boundaries, and per-Theta sketches at Delta boundaries.

### A.5 t-digest Swap Distribution Tracker

The t-digest (Dunning and Ertl, 2019) maintains centroids with a scale function that allocates more resolution to distribution tails. It lacks DDSketch's formal worst-case guarantee but achieves better empirical accuracy at extreme quantiles (less than 0.1% error at p50 in practice).

```rust
// Cargo.toml dependency:
// tdigest = "0.5"

use tdigest::TDigest;

/// Swap value distribution tracker using t-digest.
/// Tracks per-pool value distributions with mergeable windows.
pub struct SwapDistributionTracker {
    /// Per-pool t-digest instances keyed by pool address
    pools: std::collections::HashMap<[u8; 20], TDigest>,
    /// Maximum number of centroids per digest
    max_centroids: usize,
}

impl SwapDistributionTracker {
    pub fn new(max_centroids: usize) -> Self {
        Self {
            pools: std::collections::HashMap::new(),
            max_centroids,
        }
    }

    /// Record a swap value for a specific pool.
    pub fn observe(&mut self, pool: &[u8; 20], value: f64) {
        let digest = self
            .pools
            .entry(*pool)
            .or_insert_with(|| TDigest::new_with_size(self.max_centroids));
        let new_digest = digest.merge_unsorted(vec![value]);
        self.pools.insert(*pool, new_digest);
    }

    /// Estimate quantile for a pool.
    pub fn quantile(&self, pool: &[u8; 20], q: f64) -> Option<f64> {
        self.pools.get(pool).map(|d| d.estimate_quantile(q))
    }

    /// Merge two trackers (e.g., combining Theta windows at Delta boundary).
    pub fn merge(&mut self, other: &SwapDistributionTracker) {
        for (pool, other_digest) in &other.pools {
            let entry = self
                .pools
                .entry(*pool)
                .or_insert_with(|| TDigest::new_with_size(self.max_centroids));
            let merged = TDigest::merge_digests(vec![entry.clone(), other_digest.clone()]);
            self.pools.insert(*pool, merged);
        }
    }
}
```

**Recommendation:** DDSketch for gas prices (heavy tail, formal bounds). t-digest for per-pool swap value distributions (empirically better at extremes, mergeable across time windows).

| Property | DDSketch | t-digest |
|----------|----------|----------|
| Error guarantee | Formal relative error | Empirical only |
| Extreme quantile accuracy | Good | Better |
| Mergeability | Yes | Yes |
| Memory footprint | ~2KB at alpha=0.01 | ~5KB at 200 centroids |
| Best use case | Gas prices (heavy tail, needs formal bounds) | Swap values per pool (empirical precision, many instances) |

### A.6 Pipeline Integration Map

| Structure | Rust crate | Pipeline point | Update frequency | Snapshot frequency |
|-----------|------------|---------------|-----------------|-------------------|
| CMS-CU + sliding window | `streaming_algorithms` or inline (~80 lines) | bardo-triage address frequency | Every transaction at Gamma tick | Theta tick snapshot for scoring |
| MIDAS-R | `midas_rs` or custom (~200 lines) | bardo-triage Stage 4 anomaly | Every transaction (block-level ticks) | Gamma composite score |
| DDSketch | `sketches-ddsketch` | bardo-witness gas tracking | Every transaction at Gamma tick | Merge at Theta, report at Delta |
| t-digest | `tdigest` | bardo-triage swap distributions | Per-pool at Gamma tick | Merge at Delta for profiling |
| Space Saving (Top-K) | `streaming_algorithms::Top` | bardo-triage heavy hitters | Theta-window accumulation | Top-100 per Theta cycle |

For time-awareness, apply exponential decay (alpha = 0.95) to CMS counters at each Gamma tick -- an O(width * depth) operation. Maintain separate DDSketch instances per Theta window; merge at Delta boundaries for long-horizon distribution profiles.

---

## Appendix B: Probabilistic Filters for Set Membership

The chain intelligence pipeline performs millions of set membership queries per block. This appendix covers the Binary Fuse filter replacement for WatchBloom and Roaring Bitmaps for block gap tracking.

### B.1 Binary Fuse Filters

The key decision: **replace `fastbloom` with `xorf::BinaryFuse8` throughout**. Binary Fuse filters achieve 8.7 bits per entry -- near the theoretical minimum of log2(1/epsilon) = 6.64 bits at 1% FPR. Standard Bloom filters sit at roughly 44% overhead above the theoretical minimum.

The WatchBloom is rebuilt from scratch at every Gamma tick via atomic arc-swap. Immutability is not a constraint, making Binary Fuse filters the ideal choice.

| Metric | fastbloom (Bloom) | xorf::BinaryFuse8 |
|--------|-------------------|-------------------|
| Bits per entry (1% FPR) | ~9.6 | ~8.7 |
| FPR at 8 bits/entry | ~2.1% | ~0.39% |
| Query: memory accesses | k (typically 7) | 3 |
| Query: branching | k conditional checks | 0 |
| Construction speed | O(n * k) | O(n), ~2x faster than xor |
| Mutability | Insert-only | Immutable (rebuild) |

The `logsBloom` field in Ethereum block headers is a fixed 2048-bit Bloom filter defined by the Yellow Paper (section 4.3). That structure cannot be changed. The WatchBloom is the Golem's internal structure and can use any filter type.

```rust
use arc_swap::ArcSwap;
use std::sync::Arc;
use xorf::{BinaryFuse8, Filter};

/// An interest entry from ChainScope representing an address or
/// event topic the Golem is watching.
pub struct InterestEntry {
    pub address: [u8; 20],
    pub event_topics: Vec<[u8; 32]>,
}

/// Build a Binary Fuse filter from the current interest set.
pub fn build_watch_filter(entries: &[InterestEntry]) -> BinaryFuse8 {
    let keys: Vec<u64> = entries
        .iter()
        .flat_map(|entry| {
            let addr_hash = xxhash_rust::xxh3::xxh3_64(&entry.address);
            let topic_hashes = entry
                .event_topics
                .iter()
                .map(|t| xxhash_rust::xxh3::xxh3_64(t));
            std::iter::once(addr_hash).chain(topic_hashes)
        })
        .collect();

    BinaryFuse8::try_from(&keys).expect("filter construction should not fail for valid input")
}

/// Check whether a log entry might match the watch set.
/// False positives possible (0.39% rate); false negatives are not.
pub fn check_log(filter: &BinaryFuse8, address: &[u8; 20], topic: &[u8; 32]) -> bool {
    let addr_key = xxhash_rust::xxh3::xxh3_64(address);
    let topic_key = xxhash_rust::xxh3::xxh3_64(topic);
    filter.contains(&addr_key) || filter.contains(&topic_key)
}
```

The `ArcSwap<BinaryFuse8>` pattern: readers call `load()` for a zero-allocation non-blocking read. The writer (Gamma tick rebuild) calls `store(new_arc)` for atomic pointer replacement. Memory reclamation is automatic through Arc reference counting.

**When to use Cuckoo filters instead:** Fan et al. (2014) introduced Cuckoo filters for when deletions are needed without full rebuild. At 1% FPR: ~12.6 bits per entry -- roughly 40% more space than Binary Fuse. Reserve for a future "live-update-without-full-rebuild" path.

### B.2 Roaring Bitmaps for Block Gap Tracking

Roaring Bitmaps (Chambi, Lemire, Kaser, and Godin, 2016) partition the integer space into chunks of 2^16 values, automatically selecting the most efficient representation per chunk: array (sparse), bitmap (dense), or run-length (sequential).

For Ethereum's monotonically increasing block numbers, run-length containers dominate. A contiguous range from block 18,000,000 to 22,000,000 compresses to ~8 bytes. Compare to `HashSet<u64>` at ~64 bytes per entry: 256 MB vs. 8 bytes.

```rust
use roaring::RoaringTreemap;

/// Block gap tracker using Roaring Bitmaps.
/// Tracks which blocks have been processed and identifies gaps.
pub struct BlockTracker {
    processed: RoaringTreemap,
    head: u64,
}

impl BlockTracker {
    pub fn new() -> Self {
        Self {
            processed: RoaringTreemap::new(),
            head: 0,
        }
    }

    pub fn mark_processed(&mut self, block: u64) {
        self.processed.insert(block);
        if block > self.head {
            self.head = block;
        }
    }

    pub fn mark_range(&mut self, start: u64, end_inclusive: u64) {
        self.processed.insert_range(start..=end_inclusive);
        if end_inclusive > self.head {
            self.head = end_inclusive;
        }
    }

    pub fn is_processed(&self, block: u64) -> bool {
        self.processed.contains(block)
    }

    pub fn gaps(&self, start: u64) -> Vec<u64> {
        let mut gaps = Vec::new();
        for block in start..=self.head {
            if !self.processed.contains(block) {
                gaps.push(block);
            }
        }
        gaps
    }

    pub fn count(&self) -> u64 {
        self.processed.len()
    }
}
```

**Cargo dependencies:**

```toml
roaring = "0.10"
```

Set operations (union, intersection, difference) are O(n) in containers, not elements. SIMD-accelerated via portable_simd in `roaring-rs`.

### B.3 Complete Filter Comparison

| Filter type | Bits/entry (1% FPR) | Construction | Query cost | Mutability | Deletion | Best use in bardo |
|-------------|---------------------|-------------|------------|------------|----------|-------------------|
| Binary Fuse 8 | ~8.7 | O(n), fast | 3 accesses, 0 branches | Immutable | Rebuild | WatchBloom (primary) |
| Xor filter | ~9.8 | O(n), slower | 3 accesses, 0 branches | Immutable | Rebuild | Legacy alternative |
| Standard Bloom | ~9.6 | O(n*k) | k accesses, k branches | Insert-only | No | Ethereum logsBloom check |
| Cuckoo filter | ~12.6 | O(n) | 2 accesses | Insert + delete | Yes | Dynamic watchlists between rebuilds |
| Counting Bloom | ~38.4 (4x Bloom) | O(n*k) | k accesses | Insert + delete | Yes | Frequency counting (rare) |
| Roaring Bitmap | Variable (run-length) | Incremental | O(1) contains | Full CRUD | Yes | Block gap tracking, set operations |

### B.4 Complete Filter Stack

The chain intelligence pipeline uses a layered filter strategy:

1. **Ethereum logsBloom** (protocol-level Bloom filter): First-pass filter on each block header. Reject blocks with no matching logs. Zero cost -- the data is already in the block header.

2. **WatchBloom** (BinaryFuse8 via arc-swap): Second-pass filter on log entries within matching blocks. 0.39% FPR, rebuilt at each Gamma tick. Eliminates >99% of irrelevant logs before any decoding.

3. **Block tracker** (RoaringTreemap): Tracks processed blocks, identifies gaps for backfill. Sub-MB memory for millions of contiguous blocks.

4. **Address sets** (RoaringBitmap): Fast set operations for interest overlap, coverage analysis, and deduplication across pipeline stages.

Total memory footprint for all probabilistic filters: under 500 KB at typical Golem operating scale (watching ~100 protocols with ~500 unique contracts).

---

## Appendix B References

- Bloom, B.H. (1970). "Space/time trade-offs in hash coding with allowable errors." *Communications of the ACM*, 13(7), 422-426.
- Chambi, S., Lemire, D., Kaser, O., and Godin, R. (2016). "Better bitmap performance with Roaring bitmaps." *Software: Practice and Experience*, 46(5), 709-719.
- Dunning, T. and Ertl, O. (2019). "Computing Extremely Accurate Quantiles Using t-Digests." arXiv:1902.04023.
- Ekle, P. and Eberle, W. (2025). "Adaptive-GraphSketch: Streaming Anomaly Detection in Dynamic Graphs." arXiv:2509.11633.
- Fan, B., Andersen, D.G., Kaminsky, M., and Mitzenmacher, M. (2014). "Cuckoo Filter: Practically Better Than Bloom." *CoNEXT 2014*.
- Lemire, D., Kaser, O., and Kurz, N. (2022). "Binary Fuse Filters: Fast and Smaller Than Xor Filters." *Journal of Experimental Algorithmics*. arXiv:2201.01174.
- Lemire, D., Ssi-Yan-Kai, G., and Kaser, O. (2018). "Consistently faster and smaller compressed bitmaps with Roaring." *Software: Practice and Experience*, 46(11).
- Metwally, A., Agrawal, D., and El Abbadi, A. (2005). "Efficient Computation of Frequent and Top-k Elements in Data Streams." *ICDT 2005*.
- Shahout, R. and Mitzenmacher, M. (2024). "Learning-Based Heavy Hitters and Flow Frequency Estimation in Streams." arXiv:2406.16270.

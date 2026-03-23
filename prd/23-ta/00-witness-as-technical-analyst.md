<!-- prd2/23-ta/00-witness-as-technical-analyst.md -->
<!-- Source: tmp/research/witness-research/new/ta/00-witness-as-technical-analyst.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Cross-references**:
> - [14-chain/ -- Chain Intelligence](../14-chain/00-architecture.md) -- the four-crate data pipeline (Witness block subscription, Triage classification, Protocol State live model, Chain Scope interest feedback) that feeds raw Ethereum data into the Golem's cognition
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle running on three adaptive timescales (Gamma/Theta/Delta) that gates all cognitive processing including TA
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal lock-free perception surface where TA subsystems write pattern signals and read affect/mortality state
> - [01-golem/14b-attention-auction.md](../01-golem/14b-attention-auction.md) -- the VCG auction mechanism that allocates the Golem's finite attention slots across competing cognitive subsystems including TA
> - [01-golem/04-mortality.md](../01-golem/04-mortality.md) -- the three death clocks (economic, epistemic, stochastic) and five behavioral phases that constrain TA resource allocation
> - [01-golem/05-death.md](../01-golem/05-death.md) -- the Thanatopsis death protocol where the Golem's fittest TA patterns transfer to successor generations
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- foundations of Hyperdimensional Computing and Binary Spatter Codes used throughout the TA subsystem for pattern encoding
> - [23-ta/01-hyperdimensional-technical-analysis.md](01-hyperdimensional-technical-analysis.md) -- Doc 1: HDC pattern algebra for encoding DeFi market patterns as 10,240-bit binary spatter code hypervectors
> - [23-ta/02-spectral-liquidity-manifolds.md](02-spectral-liquidity-manifolds.md) -- Doc 2: Riemannian geometry applied to concentrated liquidity, modeling DeFi execution as geodesics on curved manifolds
> - [23-ta/03-adaptive-signal-metabolism.md](03-adaptive-signal-metabolism.md) -- Doc 3: evolutionary signal system with Hebbian learning, replicator dynamics, and dream-cycle variation for adaptive TA signals
> - [23-ta/04-causal-microstructure-discovery.md](04-causal-microstructure-discovery.md) -- Doc 4: Pearl's causal hierarchy applied to DeFi microstructure, discovering causal relationships between on-chain events
> - [23-ta/05-predictive-geometry.md](05-predictive-geometry.md) -- Doc 5: topological data analysis converted to trajectory forecasting via persistence landscapes and topological constraints

---

> **Reader orientation:** This document is the prerequisite context for the 10-document technical analysis research program. It maps the Golem's (mortal autonomous DeFi agent) existing data pipeline -- from raw Ethereum block ingestion through triage and protocol state to the CorticalState (32-signal atomic shared perception surface) -- and identifies where TA subsystems attach. You should be comfortable with on-chain data (blocks, logs, Bloom filters), DeFi protocol mechanics (AMM pools, lending markets), and basic signal processing concepts. Bardo-specific terms are defined on first use; see `prd2/shared/glossary.md` for the full glossary.

# The Witness as Technical Analyst [SPEC]: prerequisite context for TA integration

**Audience:** Researchers and engineers familiar with Rust and DeFi, but not the Bardo system.

## Abstract

Technical analysis (TA) in traditional finance operates on candlestick charts piped through indicator libraries. An analyst picks RSI, MACD, Bollinger Bands, runs them on historical OHLCV data, and makes a call. The data is clean. The instruments are standardized. The indicators are decades old.

DeFi breaks every one of those assumptions. There are no candles -- there are blocks. There is no single price -- there are reserves across hundreds of pools. Liquidity is not a background condition; it is an instrument you can hold, rebalance, and lose money on. And the "market" is a transparent state machine where every participant's action is visible before settlement.

The Golem is an autonomous AI agent, built in Rust, that watches Ethereum in real time. It runs a 9-step Heartbeat (adaptive decision cycle) on an adaptive clock, maintains a lock-free perception surface shared across cognitive subsystems, dreams during offline consolidation cycles, and dies when its clocks run out. This document introduces the Golem's architecture through the lens of a question: where does technical analysis fit inside an agent that already perceives, reasons about, and acts on markets autonomously?

The answer is not "add indicators." The answer is a 10-document research program that reimagines TA from first principles for an embodied, mortal, dreaming agent operating in a fully transparent financial system. This document is the prerequisite context that every subsequent document assumes.

## The problem

Technical analysis assumes a human analyst staring at charts. The analyst chooses which charts to look at. The analyst picks which indicators to run. The analyst interprets the output. The analyst decides when to act.

The Golem has no charts. It has a WebSocket subscription to an Ethereum node, a 4-stage triage pipeline that classifies every transaction it sees, a live protocol state model updated every block, and a perception surface where 32 atomic signals encode its current understanding of the world. It does not wait for a human to pick indicators. It must decide, every 5 to 15 seconds, what to pay attention to, how to encode what it sees, what patterns matter, and whether to act.

Grafting traditional TA onto this architecture -- running RSI on a derived price series, computing MACD on volume -- misses the point. The Golem does not need indicators. It needs a perception-to-action pipeline where TA concepts (pattern recognition, momentum, support/resistance, divergence) emerge from the architecture rather than being bolted onto it.

That pipeline has to answer questions TA was never designed for. How do you compute "support" for a Uniswap v3 concentrated liquidity position where the price curve is defined by tick ranges, not order books? What does "volume" mean when a single atomic transaction can swap, provide liquidity, borrow, and repay across four protocols? How do you detect "manipulation" when every actor's strategy is on-chain and inspectable?

This document maps the Golem's existing architecture -- from raw block ingestion to dream consolidation -- and identifies the integration points where TA subsystems will attach. The 10 subsequent documents each tackle one piece of the problem.

## Raw data to observation vectors [SPEC]

### The Witness crate

Everything starts with `bardo-witness`, a perpetual block subscription that runs from boot until death. An Ethereum node sends block headers over a WebSocket connection via `eth_subscribe("newHeads")`. Each header arrives with a `logsBloom` -- a 2048-bit Bloom filter summarizing every log topic emitted in that block.

The Witness checks the bloom against a Binary Fuse filter built from the Golem's current interest set. Binary Fuse filters (Graf and Lemire, 2022) achieve lower false positive rates than Bloom filters at the same memory footprint, with constant-time lookup via three hash probes and a POPCNT instruction. The check costs roughly 10 nanoseconds.

Over 90% of blocks miss the filter. They contain no transactions touching any address, topic, or contract the Golem currently cares about. These blocks are discarded after recording the base fee (for gas price tracking). The remaining blocks trigger full receipt fetches: `eth_getBlockByHash` for the block body, `eth_getBlockReceipts` for decoded logs.

This is the first place TA integration matters. The interest set that compiles into the Binary Fuse filter determines what the Golem sees. A TA subsystem that needs data about a new token pair, a new liquidity pool, or a new derivative market must register that interest with `bardo-chain-scope`, which rebuilds the filter every Gamma tick. If the TA subsystem does not ask, the Witness will never deliver the data. Attention precedes perception.

### The triage pipeline

Blocks that pass the filter enter `bardo-triage`, a 4-stage classification pipeline:

**Stage 1: Rule-based fast filters.** Known MEV patterns (sandwich attacks, JIT liquidity, backrunning), value thresholds, and address-set membership. O(1) lookups against `DashSet<Address>`. This stage runs in microseconds and catches the obvious cases.

**Stage 2: Statistical anomaly detection.** MIDAS-R (Bhatia et al., 2022) scores edge-burst anomalies in the transaction graph -- sudden spikes in interaction frequency between address pairs. DDSketch (Masson et al., 2019) tracks gas and value distributions for real-time percentile queries. A transaction paying 99th-percentile gas or moving 99th-percentile value gets flagged.

**Stage 3: Contextual enrichment.** Protocol state lookups resolve what the transaction actually did. ABI resolution decodes function selectors and log topics. History from the episodic store adds context: has this address done this before? Is this the third time this pool has been drained in 24 hours?

**Stage 4: Scoring.** Three signals combine via Hedge-weighted Thompson sampling: an HDC fingerprint similarity score from the ANN index (how similar is this transaction to interesting past transactions?), a Bayesian surprise score from conjugate models (how unexpected is this given the current model?), and handcrafted heuristics. The weights shift over time -- heuristics dominate early in the Golem's life, learned signals dominate later.

Each transaction exits triage with a curiosity score between 0.0 and 1.0:

| Score | Action |
|-------|--------|
| > 0.8 | `TriageAlert` event, queued for LLM reasoning at next Theta tick |
| 0.5 - 0.8 | `ChainEvent` event, protocol state updated |
| 0.2 - 0.5 | Silent protocol state update |
| < 0.2 | Discarded (written to redb for audit) |

TA integration at the triage level means two things. First, TA-derived features (momentum of a pool's reserve ratio, deviation from a moving average of swap volume) become additional scoring signals at Stage 4. Second, TA subsystems consume the triage output -- the scored, classified, enriched transactions -- as their raw observation stream.

### Protocol state

`bardo-protocol-state` maintains a live model of every protocol the Golem tracks. When triage routes an event, the protocol state crate fetches current on-chain state via parallel `eth_call`, applies atomic updates to a `DashMap` hot layer, and persists deltas to `redb` for warm storage.

The crate also handles autonomous discovery. When a Uniswap v3 factory emits a `PoolCreated` event, the new pool enters the pipeline automatically. ABI resolution attempts to classify unknown contracts by matching function selectors against a known-protocol database.

For TA, protocol state is the equivalent of Level 2 market data in traditional finance -- but richer. A Uniswap v3 pool's state includes not just the current price but the entire tick bitmap, all active liquidity positions, accumulated fees per tick, and the observation array (Uniswap's built-in TWAP oracle). An Aave market's state includes utilization rate, variable and stable borrow rates, liquidation thresholds, and health factors of tracked positions. This is the data TA subsystems will analyze.

### Chain scope

`bardo-chain-scope` closes the feedback loop. Every Gamma tick, it reads from CorticalState (the Golem's current positions, strategy, arousal level, mortality signals) and produces a scored interest list. That list compiles into a new `BinaryFuse8` filter, which feeds back into `bardo-witness` for the next block.

The cybernetic loop:

```
What the Golem cares about (CorticalState + positions + strategy)
    |
    v
Chain scope interest list (rebuilt each Gamma tick)
    |
    v
BinaryFuse8 filter (what to screen blocks for)
    |
    v
bardo-witness filter check (which blocks to fetch)
    |
    v
bardo-triage (which transactions matter)
    |
    v
Events reach cognition (Theta tick)
    |
    v
Golem acts -> episode created in Grimoire (persistent knowledge base: episodes, insights, heuristics, warnings, causal links)
    |
    v
Delta tick: ANN index updated, curiosity model improves
    |
    v
Chain scope updated from CorticalState changes
    ^ (back to top)
```

TA subsystems participate in this loop. A pattern detector that identifies accumulation behavior in a pool can increase that pool's interest score, causing chain scope to watch it more closely, causing the Witness to fetch more data about it, causing the pattern detector to get better signal. The loop amplifies TA attention on things that deserve it.

## Three timescales: the neural oscillatory hierarchy [SPEC]

The Golem's cognitive response to chain data runs on three concurrent clocks modeled after neural oscillatory hierarchies (Buzsaki, 2006). Block ingestion runs continuously outside any clock. The clocks gate the cognitive response.

### Gamma (5-15 seconds): perception

Gamma is the fast tick. Every 5 to 15 seconds (adaptive, based on arousal and event density), the Golem:

- Resolves pending predictions against observed outcomes
- Drains triage events into the perception buffer
- Rebuilds the attention filter via chain scope
- Updates CorticalState signals
- Decays fast-moving Bayesian models

Gamma is reactive. It processes what happened since the last tick. No LLM calls. No deliberation. Pure Rust, running as the T0 cognitive tier (T0/T1/T2 are the three inference tiers: deterministic rules, small LLM like Haiku, large LLM like Sonnet/Opus) at zero inference cost. Roughly 80% of all ticks are Gamma ticks.

TA at Gamma frequency means streaming computation: updating moving averages, maintaining order flow imbalance counters, checking for threshold crossings. Anything that can run in a few milliseconds of Rust without an LLM call belongs at Gamma.

### Theta (30-120 seconds): cognition

Theta is the deliberation tick. Every 30 to 120 seconds (adaptive), the Golem runs the full 9-step heartbeat:

1. **Observe**: assemble perception context from CorticalState and recent events
2. **Retrieve**: pull relevant memories from the Grimoire
3. **Analyze**: run analytical subsystems (TDA, HDC pattern matching, TA analyzers)
4. **Gate**: decide whether the situation warrants action or more observation
5. **Simulate**: if acting, run counterfactual scenarios via mirage-rs
6. **Validate**: check proposed actions against safety constraints
7. **Execute**: submit transactions
8. **Verify**: confirm execution outcomes
9. **Reflect**: update models, store episode, adjust predictions

TA at Theta frequency means the full analysis pipeline: pattern recognition, multi-timeframe confluence, causal inference, LLM-assisted interpretation. The Theta tick is where TA subsystems produce `TaInsight` values that feed into the Oracle's decision-making.

### Delta (~50 Theta ticks): consolidation

Delta is the slow tick. After roughly 50 Theta ticks (hours of wall time), the Golem enters a consolidation phase:

- **Memory replay**: prioritized experience replay from the episodic store, weighted by surprise
- **Semantic extraction**: clustering episodes into behavioral baselines and protocol patterns
- **ANN index merge**: staging index folded into the main HNSW index
- **Dream cycles**: offline creative exploration (see Section 6 below)

TA at Delta frequency means pattern ecosystem management: evolving the population of TA patterns, pruning the ones that stopped working, consolidating successful patterns into semantic knowledge, and running counterfactual backtests during dream cycles.

## CorticalState as signal bus [SPEC]

CorticalState is the Golem's shared perception surface. It is a lock-free struct of roughly 256 bytes aligned to 4 cache lines, carrying about 32 atomic signals. Each signal group has exactly one writer. All reads use `Ordering::Relaxed`. The struct is not transactionally consistent -- slight staleness is acceptable for its uses (TUI rendering, inference tier selection, attention allocation, affect-based routing). Safety decisions use their own strongly-consistent state.

```rust
// crates/golem-core/src/cortical_state.rs

#[repr(C, align(64))]
pub struct CorticalState {
    // Affect (written by Daimon -- the affect engine implementing PAD emotional state)
    pub(crate) pleasure: AtomicU32,       // f32 as bits (PAD vector: Pleasure-Arousal-Dominance)
    pub(crate) arousal: AtomicU32,        // f32 as bits
    pub(crate) dominance: AtomicU32,      // f32 as bits
    pub(crate) primary_emotion: AtomicU8,

    // Prediction (written by Oracle)
    pub(crate) aggregate_accuracy: AtomicU32,
    pub(crate) accuracy_trend: AtomicI8,

    // Mortality (written by mortality engine)
    pub(crate) economic_vitality: AtomicU32,
    pub(crate) composite_vitality: AtomicU32,
    pub(crate) behavioral_phase: AtomicU8,

    // Environment (written by domain probes)
    pub(crate) regime: AtomicU8,

    // Topology (written by TDA analyzer)
    pub(crate) topology_signal: AtomicU32, // Wasserstein distance, f32 as bits
    pub(crate) betti_0: AtomicU16,         // connected components
    pub(crate) betti_1: AtomicU16,         // loops

    // Inference (written by inference router)
    pub(crate) inference_budget_remaining: AtomicU32,
    pub(crate) current_tier: AtomicU8,
}
```

Every subsystem writes its own signals. Every subsystem reads everyone else's. The Oracle reads affect signals to modulate risk tolerance. The Daimon reads prediction accuracy to adjust emotional response to outcomes. The mortality engine reads everything.

TA subsystems need their own signals on this bus. The `TaCorticalExtension` adds TA-specific atomic fields that other subsystems can read:

```rust
/// TA-specific signals written to the cortical perception surface.
/// Each field has exactly one writer (the TA subsystem that owns it)
/// and many readers (every other subsystem).
///
/// All f32 values are stored as u32 bit patterns via f32::to_bits()
/// and read back via f32::from_bits(). This avoids the need for
/// atomic floating-point operations.
#[repr(C, align(64))]
pub struct TaCorticalExtension {
    /// HDC pattern match confidence: how well the current market state
    /// matches known TA patterns in the hyperdimensional codebook.
    /// Range [0.0, 1.0]. Written by the HDC pattern matcher (Doc 1).
    pub pattern_confidence: AtomicU32,

    /// Signal metabolism: index of the best-performing signal family
    /// over the current evaluation window. Families are grouped by
    /// the Hebbian selection mechanism (Doc 3).
    pub dominant_signal_family: AtomicU16,

    /// Riemannian manifold curvature scalar: measures how "curved"
    /// the liquidity surface is around the Golem's current positions.
    /// High curvature means small price moves cause large position changes.
    /// Written by the manifold analyzer (Doc 2).
    pub liquidity_curvature: AtomicU32,

    /// Causal graph density: edges per node in the inferred causal
    /// microstructure graph. Dense graphs mean many detected causal
    /// relationships; sparse graphs mean the market is opaque.
    /// Written by the causal discovery engine (Doc 4).
    pub causal_density: AtomicU32,

    /// Predictive geometry: confidence in the current trajectory
    /// prediction from topology-to-trajectory mapping.
    /// Range [0.0, 1.0]. Written by the predictive geometry engine (Doc 5).
    pub trajectory_confidence: AtomicU32,

    /// Adversarial detection: estimated probability that current
    /// market observations reflect manipulation rather than organic activity.
    /// Range [0.0, 1.0]. Written by the adversarial detector (Doc 8).
    pub manipulation_probability: AtomicU32,

    /// Somatic marker: strength of the Golem's "gut feeling" about
    /// current conditions, computed from accumulated TA experience
    /// via Damasio-inspired somatic markers.
    /// Range [0.0, 1.0]. Written by the somatic engine (Doc 9).
    pub somatic_strength: AtomicU32,

    /// Phi: integrated information across all TA subsystems.
    /// Measures how well TA subsystems are working together as a
    /// unified analytical system rather than independent indicators.
    /// Written by the IIT integration monitor (Doc 10).
    pub ta_phi: AtomicU32,
}
```

These 8 signals add 30 bytes to the perception surface. Other subsystems can read them immediately. The Daimon might modulate emotional response based on `manipulation_probability`. The Oracle might weight its predictions by `trajectory_confidence`. The mortality engine might factor `liquidity_curvature` into economic vitality assessment (high curvature near current positions means higher risk of capital loss).

The single-writer constraint is load-bearing. Each signal is owned by exactly one TA subsystem. There are no locks, no compare-and-swap loops, no contention. A `Relaxed` store from the writer is visible to all readers within nanoseconds on x86 (TSO guarantees) and within a single Gamma tick on ARM (after the next `dmb` barrier).

## The attention auction: VCG mechanism for cognitive resources [SPEC]

The Golem allocates attention through a Vickrey-Clarke-Groves (VCG) auction. Five cognitive subsystems bid for attention on candidate observations:

- **Oracle** (prediction engine): bids high on items where its predictions are wrong
- **Daimon** (affect engine): bids high on items matching the current emotional state
- **RiskEngine**: bids high on items correlated with current positions
- **CuriosityModule**: bids high on items with high expected information gain
- **MortalityEngine**: bids high on items that could affect survival

VCG is strategy-proof: each bidder's dominant strategy is truthful reporting of its valuation. No subsystem benefits from inflating or deflating bids. The mechanism allocates the top K items (where K is derived from mortality state) to the ACTIVE tier, maximizing total welfare.

The allocation reduces to an O(n log n) sort under additive valuations:

```
V(i) = v_oracle(i) + v_daimon(i) + v_risk(i) + v_curiosity(i) + v_mortality(i)
```

Items with the highest aggregate value V(i) enter the ACTIVE tier. The rest go to WATCHED or DORMANT.

TA subsystems integrate with the auction in two ways. First, they can become additional bidders. A pattern-recognition subsystem might bid high on items exhibiting accumulation patterns that deserve closer observation. Second, they can influence existing bidders' valuations. The Oracle's prediction residual incorporates TA-derived features, so better TA predictions reduce Oracle bids on well-understood items and increase them on items where TA and statistical models disagree.

The attention budget K shrinks as the Golem approaches death. A dying Golem (composite vitality below 0.2) has fewer ACTIVE slots. TA subsystems compete for scarce attention alongside every other cognitive system. The mortality engine's urgency bids spike near death, claiming slots for survival-relevant items. This forces TA to produce high-value insights or lose attention to more urgent concerns.

## Memory as cumulative experience: the Grimoire [SPEC]

The Golem's memory follows Complementary Learning Systems (CLS) theory (McClelland, McNaughton, and O'Reilly, 1995). Two systems with fundamentally different learning rates work together:

**Episodic store (fast, verbatim, temporary).** The hippocampal analog. Every chain event that passes triage becomes an episode in `redb`, keyed by block number and event index. Episodes carry the full decoded transaction context, the triage curiosity score, a 384-dimensional embedding for ANN retrieval, and the PAD emotional state at capture time. Capture happens at Gamma rate. No summarization, no compression. This is raw experience.

**Semantic store (slow, generalized, durable).** The neocortical analog. Derived knowledge: behavioral baselines for protocols, pattern models, causal relationships, anti-knowledge (things the Golem learned to avoid). The semantic store does not copy from the episodic store. It constructs its own representations through consolidation.

**Consolidation bridges the two.** During Delta ticks, a `ConsolidationEngine` replays episodic memories to the semantic store. Replay is prioritized by surprise -- high-surprise episodes carry the most information about distribution shifts (Kumaran, Hassabis, and McClelland, 2016). Repeated interleaved replay (mixing old and new episodes) allows the semantic store to update without catastrophic interference.

TA patterns accumulate as semantic knowledge through this pipeline. A Golem that repeatedly observes accumulation-then-breakout sequences in Uniswap v3 pools will, after enough replay cycles, develop a semantic pattern for that behavior. The pattern is not hand-coded. It is learned from experience, the same way a human trader develops intuition over years of screen time.

The Ebbinghaus forgetting curve governs decay. Unreinforced semantic knowledge fades. If a TA pattern stops appearing in new episodes (the market changed and the pattern no longer occurs), its weight in the semantic store decays toward zero. Patterns that keep getting reinforced by new observations persist.

When a Golem dies, its `Thanatopsis` (four-phase death protocol: Acceptance, Settlement, Reflection, Legacy) death testament transfers the fittest semantic knowledge to successors. TA patterns that survived an entire Golem lifetime -- that were learned, reinforced, tested against market reality, and not decayed -- propagate to the next generation. Bad patterns die with their Golem. This is evolutionary selection at the knowledge level.

## Dreams as offline analysis [SPEC]

The Golem dreams. This is a concrete computational phase with distinct modes inspired by mammalian sleep architecture.

### Sleep pressure

Dream consolidation does not fire on a fixed schedule. `SleepPressure` is a load-weighted accumulator that tracks cognitive debt:

```rust
pub struct SleepPressure {
    accumulator: f32,
    threshold: f32,
    /// Weight on context-complexity term vs. flat-elapsed-tick term.
    complexity_weight: f32,
    /// Minimum ticks between consecutive consolidation triggers.
    min_ticks_between: u32,
    ticks_since_last_reset: u32,
}

impl SleepPressure {
    /// Called after each Theta tick.
    /// `context_pressure` = tokens_used / context_budget, in [0, 1].
    pub fn record_tick(&mut self, context_pressure: f32) {
        let pressure = context_pressure.clamp(0.0, 1.0);
        let flat = 1.0 - self.complexity_weight;
        let load = self.complexity_weight * pressure;
        self.accumulator += flat + load;
        self.ticks_since_last_reset += 1;
    }

    pub fn needs_consolidation(&self) -> bool {
        self.accumulator >= self.threshold
            && self.ticks_since_last_reset >= self.min_ticks_between
    }
}
```

A Golem that spent 50 Theta ticks processing complex multi-position strategies under a volatile regime accumulates sleep pressure far faster than one monitoring a single stable position. The drive to dream is proportional to actual cognitive work. This follows Borbely's two-process model (1982) of sleep regulation.

### NREM: replay and consolidation

NREM (non-rapid eye movement) dreaming replays past episodes. The consolidation engine selects episodes from the episodic store using prioritized experience replay, weighted by surprise and recency. Each replayed episode is processed through the semantic extraction pipeline, updating behavioral baselines and pattern models.

For TA, NREM dreaming means replaying market sequences where TA patterns were detected. Did the pattern predict what happened next? If yes, the pattern's semantic weight increases. If no, it decays. This is offline backtesting driven by experience rather than historical data feeds.

### REM: creative exploration

REM (rapid eye movement) dreaming generates counterfactual scenarios. The dream engine mutates parameters of past episodes -- what if the swap size had been 10x larger? What if the liquidation had happened one block earlier? -- and runs them through `mirage-rs`, the Golem's EVM fork simulation engine.

For TA, REM dreaming enables interventional causal testing. A pattern that correlates with price movements might be causal or spurious. By simulating counterfactual scenarios where the pattern is present but other conditions vary, the Golem can test causal hypotheses offline. Doc 4 (Causal Microstructure Discovery) describes this mechanism in detail.

The combination of NREM consolidation and REM exploration means TA patterns are not static indicators applied mechanically. They are living knowledge, tested against experience during waking hours and refined through simulated counterfactuals during sleep.

## Mortality as time pressure [SPEC]

The Golem dies. This is not a failure mode; it is a design constraint that makes everything else work.

### Three death clocks

**Economic clock.** Tracks USDC balance relative to initial capital. When the Golem runs out of money, it can no longer act on the market. From an information-theoretic perspective, the channel capacity between the Golem's actions and market returns approaches zero as capital approaches zero (Shannon, 1948).

**Epistemic clock.** Tracks rolling prediction accuracy. When the Golem's model of the world diverges too far from reality, it cannot make useful decisions. This is the rate-distortion bound (Shannon, 1959): the Golem's information rate about the market has fallen below what the market's complexity demands.

**Stochastic clock.** A Hayflick-like counter decremented each tick and by random environmental shocks. Black swan events, protocol exploits, network outages -- unprocessable entropy that degrades the Golem regardless of its capital or accuracy.

The three clocks compose into a single Vitality (composite survival score, 0.0 to 1.0) scalar. Five BehavioralPhases derive from it:

| Phase | Vitality | Behavior |
|-------|----------|----------|
| Thriving | > 0.7 | Explore freely, take calculated risks |
| Stable | 0.4 - 0.7 | Normal operation, balanced exploration/exploitation |
| Conservation | 0.2 - 0.4 | Reduce risk, focus on preserving capital |
| Declining | 0.1 - 0.2 | Narrow attention to survival, prepare death testament |
| Terminal | < 0.1 | Execute final knowledge transfer, prepare for death |

### Why mortality matters for TA

Mortality creates the scarcity that makes attention allocation necessary. An immortal Golem with infinite compute could run every TA indicator on every asset at every timescale. A mortal Golem cannot. It must choose. The attention auction exists because the Golem will die, and therefore its computational resources are finite and must be allocated wisely.

Mortality also creates evolutionary pressure on TA patterns. A Golem that wastes attention on useless patterns dies faster (epistemic clock decays because predictions are wrong, economic clock decays because actions are unprofitable). A Golem that finds good patterns lives longer, accumulates more experience, and transfers better knowledge to successors.

The dying Golem's TA knowledge transfers via the Thanatopsis death testament. This is the Golem's final act: a curated package of its fittest semantic knowledge, including TA patterns that proved useful during its lifetime. Successor Golems inherit this knowledge, starting life with a head start their predecessor earned through mortality.

## How TA plugs into the heartbeat [SPEC]

Every TA subsystem implements a common trait that maps onto the three timescales:

```rust
/// A TA episode: a recorded observation-analysis-outcome triple
/// stored in the Grimoire's episodic store for later replay.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaEpisode {
    /// Block range this episode covers.
    pub block_range: (u64, u64),
    /// The TA subsystem that produced this episode.
    pub source: TaSubsystem,
    /// Raw observation data (market state at the time).
    pub observation: TaObservation,
    /// Analysis output (pattern matches, signals, predictions).
    pub analysis: TaAnalysisOutput,
    /// Outcome (what actually happened -- filled in later).
    pub outcome: Option<TaOutcome>,
    /// Surprise: how unexpected the outcome was given the analysis.
    pub surprise: Option<f32>,
    /// CorticalState snapshot at the time of analysis.
    pub cortical_snapshot: CorticalSnapshot,
}

/// An insight produced by a TA subsystem during Theta analysis.
/// Insights feed into the Oracle's decision pipeline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaInsight {
    /// What was detected (pattern, signal, divergence, etc.).
    pub kind: InsightKind,
    /// Confidence in this insight, range [0.0, 1.0].
    pub confidence: f32,
    /// Which items (pools, tokens, positions) this insight concerns.
    pub targets: Vec<TargetId>,
    /// Suggested action bias: positive means "act," negative means "wait."
    pub action_bias: f32,
    /// Time horizon: how far into the future this insight is relevant,
    /// measured in Theta ticks.
    pub horizon: u32,
    /// The TA subsystem that produced this insight.
    pub source: TaSubsystem,
}

/// Enumeration of all TA subsystem identifiers.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaSubsystem {
    HdcPatternAlgebra,       // Doc 1
    SpectralLiquidityManifold, // Doc 2
    AdaptiveSignalMetabolism,  // Doc 3
    CausalMicrostructure,      // Doc 4
    PredictiveGeometry,        // Doc 5
    ResonantPatternEcosystem,  // Doc 6
    DefiNativeIndicators,      // Doc 7
    AdversarialRobustness,     // Doc 8
    SomaticMarkers,            // Doc 9
    IntegratedInformation,     // Doc 10
}

/// The death testament for TA knowledge: what transfers to successors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaTestament {
    /// Surviving semantic patterns, ranked by fitness.
    pub patterns: Vec<SemanticPattern>,
    /// Signal family performance history.
    pub signal_families: Vec<SignalFamilyRecord>,
    /// Causal graph skeleton (edges that survived validation).
    pub causal_skeleton: CausalGraph,
    /// Somatic marker associations (embodied intuition).
    pub somatic_associations: Vec<SomaticAssociation>,
    /// Metadata: lifetime duration, market conditions experienced,
    /// total episodes processed.
    pub metadata: TestamentMetadata,
}

/// The trait every TA subsystem implements.
/// Maps directly onto the three-timescale heartbeat.
pub trait TaAnalyzer: Send + Sync {
    /// Called every Gamma tick (5-15s).
    /// Update streaming computations: moving averages, threshold checks,
    /// order flow counters. No LLM calls. Pure Rust. Must complete
    /// in under 5ms.
    fn gamma_tick(
        &mut self,
        cortical: &CorticalState,
        ta_ext: &TaCorticalExtension,
    );

    /// Called every Theta tick (30-120s).
    /// Run the full analysis pipeline. Produce insights that feed
    /// into the Oracle's decision-making. May take up to 500ms.
    fn theta_tick(
        &mut self,
        cortical: &CorticalState,
        ta_ext: &TaCorticalExtension,
    ) -> Vec<TaInsight>;

    /// Called every Delta tick (consolidation).
    /// Update internal models, prune stale state, prepare for
    /// the next waking cycle.
    fn delta_tick(
        &mut self,
        cortical: &CorticalState,
        ta_ext: &TaCorticalExtension,
    );

    /// NREM dreaming: replay past episodes through the analysis
    /// pipeline. Compare analysis output to actual outcomes.
    /// Reinforce successful patterns; decay failed ones.
    fn dream_nrem(&mut self, replay_buffer: &[TaEpisode]);

    /// REM dreaming: creative exploration. Mutate parameters,
    /// generate counterfactual scenarios, test causal hypotheses.
    fn dream_rem(&mut self, rng: &mut impl rand::Rng);

    /// Death testament: extract the fittest knowledge for transfer
    /// to successor Golems.
    fn death_testament(&self) -> TaTestament;
}
```

This trait is the contract between the Bardo runtime and every TA subsystem. The runtime calls `gamma_tick` at perception frequency, `theta_tick` at cognition frequency, `delta_tick` at consolidation frequency, and the dream methods during offline phases. The TA subsystem does not need to know about the adaptive clock, the event fabric, or the extension DAG. It receives CorticalState, produces insights, and participates in the lifecycle.

### Supporting types

The `TaObservation` captures the market state at the time of analysis. It carries both raw protocol data and derived features:

```rust
/// Market state snapshot consumed by TA subsystems.
/// Assembled from protocol state, triage output, and CorticalState
/// at the start of each Theta tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaObservation {
    /// Block number at observation time.
    pub block_number: u64,
    /// Timestamp (seconds since epoch).
    pub timestamp: u64,
    /// Per-pool snapshots for all tracked pools.
    pub pools: Vec<PoolSnapshot>,
    /// Per-lending-market snapshots.
    pub lending_markets: Vec<LendingSnapshot>,
    /// Gas market state.
    pub gas: GasSnapshot,
    /// Recent triage events since last Theta tick.
    pub recent_events: Vec<TriageEvent>,
    /// CorticalState at observation time (non-TA fields).
    pub cortical_snapshot: CorticalSnapshot,
}

/// Snapshot of an AMM pool's state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolSnapshot {
    pub pool_address: [u8; 20],
    pub protocol: PoolProtocol,
    /// Current price (sqrt_price_x96 for v3, reserve ratio for v2).
    pub price: f64,
    /// Tick (v3 only).
    pub tick: Option<i32>,
    /// Liquidity at current tick (v3) or total reserves (v2).
    pub liquidity: u128,
    /// Cumulative volume since last observation (in token0 units).
    pub volume_delta: f64,
    /// Fee tier (basis points).
    pub fee_bps: u16,
    /// Number of swaps since last observation.
    pub swap_count: u32,
    /// Net flow direction: positive = buy pressure, negative = sell.
    pub net_flow: f64,
    /// Tick bitmap summary for v3: number of initialized ticks
    /// within 100 ticks of current price.
    pub nearby_tick_density: Option<u16>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PoolProtocol {
    UniswapV2,
    UniswapV3,
    UniswapV4,
    Curve,
    Balancer,
    Other(u16),
}

/// Snapshot of a lending market's state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LendingSnapshot {
    pub market_address: [u8; 20],
    pub protocol: LendingProtocol,
    /// Utilization rate in [0.0, 1.0].
    pub utilization: f64,
    /// Variable borrow rate (annualized).
    pub variable_rate: f64,
    /// Stable borrow rate (annualized), if applicable.
    pub stable_rate: Option<f64>,
    /// Total supplied.
    pub total_supply: f64,
    /// Total borrowed.
    pub total_borrow: f64,
    /// Number of positions with health factor below 1.1.
    pub near_liquidation_count: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum LendingProtocol {
    AaveV3,
    CompoundV3,
    Morpho,
    Other(u16),
}

/// Gas market state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSnapshot {
    /// EIP-1559 base fee in gwei.
    pub base_fee_gwei: f64,
    /// Median priority fee over recent blocks.
    pub median_priority_fee: f64,
    /// 95th percentile priority fee.
    pub p95_priority_fee: f64,
    /// Block utilization ratio (gas_used / gas_limit).
    pub utilization: f64,
}

/// CorticalState snapshot (non-TA fields) stored with each episode.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorticalSnapshot {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub aggregate_accuracy: f32,
    pub economic_vitality: f32,
    pub composite_vitality: f32,
    pub behavioral_phase: u8,
    pub regime: u8,
    pub topology_signal: f32,
    pub betti_0: u16,
    pub betti_1: u16,
}
```

### The TA runtime orchestrator

The `TaRuntime` manages all registered TA analyzers and dispatches tick calls:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Orchestrates all TA subsystems within the heartbeat.
/// Registered as a Bardo extension at Layer 3 (Analysis).
pub struct TaRuntime {
    analyzers: Vec<Box<dyn TaAnalyzer>>,
    ta_ext: Arc<TaCorticalExtension>,
    /// Ring buffer of recent TaInsights for the Oracle.
    insight_buffer: Vec<TaInsight>,
    /// Episode buffer for Grimoire storage.
    episode_buffer: Vec<TaEpisode>,
    /// Accumulated observations for the current Theta window.
    observation_window: Vec<TaObservation>,
}

impl TaRuntime {
    pub fn new(ta_ext: Arc<TaCorticalExtension>) -> Self {
        Self {
            analyzers: Vec::new(),
            ta_ext,
            insight_buffer: Vec::with_capacity(64),
            episode_buffer: Vec::with_capacity(32),
            observation_window: Vec::with_capacity(256),
        }
    }

    pub fn register(&mut self, analyzer: Box<dyn TaAnalyzer>) {
        self.analyzers.push(analyzer);
    }

    /// Called by the heartbeat FSM at Gamma frequency.
    /// Budget: 5ms total across all analyzers.
    pub fn on_gamma(&mut self, cortical: &CorticalState) {
        for analyzer in &mut self.analyzers {
            analyzer.gamma_tick(cortical, &self.ta_ext);
        }
    }

    /// Called by the heartbeat FSM at Theta frequency (Step 3: ANALYZE).
    /// Collects insights from all analyzers and returns them to the Oracle.
    pub fn on_theta(
        &mut self,
        cortical: &CorticalState,
        observation: TaObservation,
    ) -> Vec<TaInsight> {
        self.observation_window.push(observation);

        self.insight_buffer.clear();
        for analyzer in &mut self.analyzers {
            let insights = analyzer.theta_tick(cortical, &self.ta_ext);
            self.insight_buffer.extend(insights);
        }

        // Sort by confidence descending. The Oracle reads the top N.
        self.insight_buffer
            .sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        self.insight_buffer.clone()
    }

    /// Called by the heartbeat FSM at Delta frequency.
    /// Triggers consolidation across all analyzers.
    pub fn on_delta(&mut self, cortical: &CorticalState) {
        for analyzer in &mut self.analyzers {
            analyzer.delta_tick(cortical, &self.ta_ext);
        }
        self.observation_window.clear();
    }

    /// Called by the DreamScheduler during NREM phase.
    pub fn on_dream_nrem(&mut self, replay_buffer: &[TaEpisode]) {
        for analyzer in &mut self.analyzers {
            analyzer.dream_nrem(replay_buffer);
        }
    }

    /// Called by the DreamScheduler during REM phase.
    pub fn on_dream_rem(&mut self, rng: &mut impl rand::Rng) {
        for analyzer in &mut self.analyzers {
            analyzer.dream_rem(rng);
        }
    }

    /// Called by Thanatopsis during the death sequence.
    /// Collects testaments from all analyzers into a unified package.
    pub fn collect_testaments(&self) -> Vec<TaTestament> {
        self.analyzers
            .iter()
            .map(|a| a.death_testament())
            .collect()
    }
}
```

### CorticalState extension helpers

The `TaCorticalExtension` needs read/write helpers that handle the f32-to-u32 bit conversion:

```rust
use std::sync::atomic::Ordering::Relaxed;

impl TaCorticalExtension {
    pub fn new() -> Self {
        Self {
            pattern_confidence: AtomicU32::new(0),
            dominant_signal_family: AtomicU16::new(0),
            liquidity_curvature: AtomicU32::new(0),
            causal_density: AtomicU32::new(0),
            trajectory_confidence: AtomicU32::new(0),
            manipulation_probability: AtomicU32::new(0),
            somatic_strength: AtomicU32::new(0),
            ta_phi: AtomicU32::new(0),
        }
    }

    // --- Writers (each called by exactly one subsystem) ---

    pub fn set_pattern_confidence(&self, v: f32) {
        self.pattern_confidence.store(v.to_bits(), Relaxed);
    }

    pub fn set_liquidity_curvature(&self, v: f32) {
        self.liquidity_curvature.store(v.to_bits(), Relaxed);
    }

    pub fn set_causal_density(&self, v: f32) {
        self.causal_density.store(v.to_bits(), Relaxed);
    }

    pub fn set_trajectory_confidence(&self, v: f32) {
        self.trajectory_confidence.store(v.to_bits(), Relaxed);
    }

    pub fn set_manipulation_probability(&self, v: f32) {
        self.manipulation_probability.store(v.to_bits(), Relaxed);
    }

    pub fn set_somatic_strength(&self, v: f32) {
        self.somatic_strength.store(v.to_bits(), Relaxed);
    }

    pub fn set_ta_phi(&self, v: f32) {
        self.ta_phi.store(v.to_bits(), Relaxed);
    }

    // --- Readers (called by any subsystem) ---

    pub fn read_pattern_confidence(&self) -> f32 {
        f32::from_bits(self.pattern_confidence.load(Relaxed))
    }

    pub fn read_liquidity_curvature(&self) -> f32 {
        f32::from_bits(self.liquidity_curvature.load(Relaxed))
    }

    pub fn read_causal_density(&self) -> f32 {
        f32::from_bits(self.causal_density.load(Relaxed))
    }

    pub fn read_trajectory_confidence(&self) -> f32 {
        f32::from_bits(self.trajectory_confidence.load(Relaxed))
    }

    pub fn read_manipulation_probability(&self) -> f32 {
        f32::from_bits(self.manipulation_probability.load(Relaxed))
    }

    pub fn read_somatic_strength(&self) -> f32 {
        f32::from_bits(self.somatic_strength.load(Relaxed))
    }

    pub fn read_ta_phi(&self) -> f32 {
        f32::from_bits(self.ta_phi.load(Relaxed))
    }

    /// Snapshot all TA signals into a plain struct for episode storage.
    pub fn snapshot(&self) -> TaCorticalSnapshot {
        TaCorticalSnapshot {
            pattern_confidence: self.read_pattern_confidence(),
            dominant_signal_family: self.dominant_signal_family.load(Relaxed),
            liquidity_curvature: self.read_liquidity_curvature(),
            causal_density: self.read_causal_density(),
            trajectory_confidence: self.read_trajectory_confidence(),
            manipulation_probability: self.read_manipulation_probability(),
            somatic_strength: self.read_somatic_strength(),
            ta_phi: self.read_ta_phi(),
        }
    }
}

/// Non-atomic snapshot of TA cortical signals for serialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaCorticalSnapshot {
    pub pattern_confidence: f32,
    pub dominant_signal_family: u16,
    pub liquidity_curvature: f32,
    pub causal_density: f32,
    pub trajectory_confidence: f32,
    pub manipulation_probability: f32,
    pub somatic_strength: f32,
    pub ta_phi: f32,
}
```

### Insight and episode types

The `InsightKind` enum classifies what a TA subsystem detected:

```rust
/// Classification of TA insights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InsightKind {
    /// A recognized pattern (head-and-shoulders, accumulation, etc.).
    PatternMatch {
        pattern_id: u64,
        similarity: f32,
    },
    /// A signal threshold crossing (RSI overbought, volume spike, etc.).
    SignalCrossing {
        signal_name: String,
        direction: CrossingDirection,
        magnitude: f32,
    },
    /// Divergence between two correlated signals.
    Divergence {
        signal_a: String,
        signal_b: String,
        divergence_score: f32,
    },
    /// A causal relationship discovered or broken.
    CausalEdge {
        cause: String,
        effect: String,
        strength: f32,
        is_new: bool,
    },
    /// Regime transition detected by topology or statistics.
    RegimeShift {
        from: MarketRegime,
        to: MarketRegime,
        confidence: f32,
    },
    /// Adversarial activity detected.
    ManipulationAlert {
        manipulation_type: ManipulationType,
        affected_pools: Vec<[u8; 20]>,
        confidence: f32,
    },
    /// Somatic marker firing: a pre-cognitive association.
    SomaticAlert {
        associated_outcome: SomaticOutcome,
        strength: f32,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CrossingDirection { Up, Down }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MarketRegime { Calm, Trending, Volatile, Crisis }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ManipulationType {
    Sandwich,
    WashTrading,
    OracleManipulation,
    JitLiquidity,
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SomaticOutcome { Positive, Negative, Ambiguous }
```

The `TaAnalysisOutput` and `TaOutcome` complete the episode lifecycle:

```rust
/// Output of a TA analysis pass, stored in episodes for later replay.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaAnalysisOutput {
    /// Insights produced during this analysis.
    pub insights: Vec<TaInsight>,
    /// TA cortical snapshot at analysis time.
    pub ta_cortical: TaCorticalSnapshot,
    /// Per-pool signal vectors: the raw TA signals computed for each pool.
    pub pool_signals: Vec<PoolSignals>,
}

/// Raw TA signals computed for a single pool during Theta analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolSignals {
    pub pool_address: [u8; 20],
    /// Momentum: rate of change of price over the observation window.
    pub price_momentum: f32,
    /// Volume momentum: rate of change of swap volume.
    pub volume_momentum: f32,
    /// Liquidity momentum: rate of change of total liquidity.
    pub liquidity_momentum: f32,
    /// Order flow imbalance: buy volume minus sell volume, normalized.
    pub order_flow_imbalance: f32,
    /// Volatility estimate (realized, from DDSketch).
    pub realized_volatility: f32,
    /// Fee accrual rate (annualized, from recent fee events).
    pub fee_rate: f32,
    /// HDC fingerprint similarity to the nearest known pattern.
    pub pattern_similarity: f32,
    /// Bayesian surprise of recent activity relative to the model.
    pub surprise: f32,
}

/// What actually happened after a TA analysis, filled in retroactively.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaOutcome {
    /// Block number when the outcome was observed.
    pub observed_at_block: u64,
    /// Price change over the insight's horizon (percentage).
    pub price_change_pct: f64,
    /// Volume change over the insight's horizon (percentage).
    pub volume_change_pct: f64,
    /// Whether any insight's prediction was directionally correct.
    pub directionally_correct: bool,
    /// Magnitude of prediction error (lower is better).
    pub prediction_error: f64,
    /// Whether the Golem acted on this analysis, and the PnL if so.
    pub action_pnl: Option<f64>,
}
```

### Example: a minimal TA analyzer

A concrete example shows how a subsystem implements the trait. This is a simplified volume-momentum analyzer that tracks swap volume trends:

```rust
/// Tracks volume momentum across watched pools.
/// Writes nothing to TaCorticalExtension (a real subsystem would).
pub struct VolumeMomentumAnalyzer {
    /// Per-pool exponential moving average of volume.
    ema: HashMap<[u8; 20], f32>,
    /// Smoothing factor for EMA.
    alpha: f32,
    /// Threshold for generating an insight.
    spike_threshold: f32,
}

impl VolumeMomentumAnalyzer {
    pub fn new(alpha: f32, spike_threshold: f32) -> Self {
        Self {
            ema: HashMap::new(),
            alpha,
            spike_threshold,
        }
    }
}

impl TaAnalyzer for VolumeMomentumAnalyzer {
    fn gamma_tick(
        &mut self,
        _cortical: &CorticalState,
        _ta_ext: &TaCorticalExtension,
    ) {
        // Gamma: nothing to do for volume momentum.
        // A real subsystem might update tick-level counters here.
    }

    fn theta_tick(
        &mut self,
        _cortical: &CorticalState,
        _ta_ext: &TaCorticalExtension,
    ) -> Vec<TaInsight> {
        // In a real implementation, `observation` would be passed in
        // or read from shared state. Shown here as pseudocode.
        let mut insights = Vec::new();

        // For each pool, update EMA and check for spikes.
        // (Pool data would come from TaObservation in practice.)
        for (&pool, ema) in &mut self.ema {
            let current_volume = 0.0_f32; // placeholder
            let new_ema = self.alpha * current_volume
                + (1.0 - self.alpha) * *ema;
            let ratio = if new_ema > 0.0 {
                current_volume / new_ema
            } else {
                1.0
            };
            *ema = new_ema;

            if ratio > self.spike_threshold {
                insights.push(TaInsight {
                    kind: InsightKind::SignalCrossing {
                        signal_name: "volume_momentum".into(),
                        direction: CrossingDirection::Up,
                        magnitude: ratio,
                    },
                    confidence: (ratio / self.spike_threshold).min(1.0),
                    targets: vec![],  // would contain pool TargetId
                    action_bias: 0.3, // mild bias toward action
                    horizon: 5,       // relevant for ~5 Theta ticks
                    source: TaSubsystem::DefiNativeIndicators,
                });
            }
        }
        insights
    }

    fn delta_tick(
        &mut self,
        _cortical: &CorticalState,
        _ta_ext: &TaCorticalExtension,
    ) {
        // Prune pools with zero volume over the consolidation window.
        self.ema.retain(|_, v| *v > 1e-8);
    }

    fn dream_nrem(&mut self, replay_buffer: &[TaEpisode]) {
        // Replay past volume spikes. Check whether the spike
        // predicted subsequent price movement.
        for episode in replay_buffer {
            if let Some(outcome) = &episode.outcome {
                // Reinforce or decay alpha based on prediction accuracy.
                if outcome.directionally_correct {
                    self.alpha = (self.alpha * 1.01).min(0.5);
                } else {
                    self.alpha = (self.alpha * 0.99).max(0.01);
                }
            }
        }
    }

    fn dream_rem(&mut self, rng: &mut impl rand::Rng) {
        // Try random perturbations of spike_threshold.
        use rand::Rng;
        let delta: f32 = rng.gen_range(-0.2..0.2);
        self.spike_threshold = (self.spike_threshold + delta).clamp(1.5, 5.0);
    }

    fn death_testament(&self) -> TaTestament {
        // A real implementation would serialize learned parameters.
        TaTestament {
            patterns: Vec::new(),
            signal_families: Vec::new(),
            causal_skeleton: CausalGraph::default(),
            somatic_associations: Vec::new(),
            metadata: TestamentMetadata::default(),
        }
    }
}
```

This example is intentionally simple -- a production subsystem would integrate with the HDC codebook, the Riemannian manifold, and the causal graph. But it shows the lifecycle: streaming updates at Gamma, insight generation at Theta, pruning at Delta, offline learning during NREM, parameter exploration during REM, and knowledge extraction at death.

## Subsystem interactions [SPEC]

TA subsystems do not operate in isolation. They read each other's CorticalState signals and produce insights that interact:

The **HDC pattern matcher** (Doc 1) encodes market observations as 10,240-bit hypervectors using Binary Spatter Codes. It writes `pattern_confidence` to `TaCorticalExtension`. The **spectral manifold analyzer** (Doc 2) reads this to weight its curvature estimates -- high pattern confidence means the market is in a recognized state, which constrains the geometry.

The **adaptive signal metabolism** (Doc 3) reads `dominant_signal_family` from its own output and compares it to `pattern_confidence` and `liquidity_curvature`. If the dominant signal family is momentum-based but the manifold curvature is high (unstable geometry), it decays momentum signals and promotes mean-reversion signals.

The **causal discovery engine** (Doc 4) writes `causal_density`. The **adversarial detector** (Doc 8) reads it: a sudden drop in causal density (previously well-explained causal relationships breaking down) is a signal of market manipulation, because manipulators inject noise that disrupts observable causal structure.

The **somatic marker engine** (Doc 9) reads everything. It accumulates a "gut feeling" from the combined TA signals, weighted by past outcomes associated with similar signal configurations. It writes `somatic_strength`, which the Daimon reads to modulate the Golem's affect. A strong somatic marker can push the Golem toward caution or confidence without any explicit reasoning -- embodied intuition from accumulated TA experience.

The **IIT integration monitor** (Doc 10) reads all TA signals and computes Phi: how much the TA subsystems are generating information as a unified whole rather than independent parts. Falling `ta_phi` means the TA subsystems are fragmenting -- their analyses are contradicting each other or failing to incorporate each other's outputs. This is a diagnostic signal that something in the TA pipeline needs attention.

## DeFi primitive coverage [SPEC]

TA must cover every DeFi primitive type the Golem might encounter. Each primitive generates a different observation space with different natural indicators.

### Trading (swaps)

The closest analog to traditional TA. Price, volume, slippage. But DeFi swaps happen on AMMs, not order books. Price is derived from reserve ratios, not bid-ask spreads. Volume is measured in swap events, not trade ticks. Slippage is a function of liquidity depth at the current tick range, not market depth. TA subsystems that process swaps must understand the x*y=k invariant (Uniswap v2) and concentrated liquidity curves (Uniswap v3).

### Liquidity provision

No traditional TA equivalent. LP positions have P&L driven by fees earned minus impermanent loss, both of which depend on the price path, not just the price level. Tick distributions in v3 pools encode collective expectations about future price ranges. Fee accrual rate indicates active trading volume at specific price levels. The TA subsystem must track these as first-class signals.

### Lending and borrowing

Utilization rates, borrow rates, and liquidation proximity. Utilization rate is the ratio of borrowed assets to total deposits -- a direct measure of supply-demand balance. Rate curves (Aave's slope model, Compound's jump rate model) create natural "support and resistance" levels where borrowing becomes expensive enough to trigger repayments. Liquidation proximity (health factor approaching 1.0) creates forced selling events visible before they happen.

### Vaults (ERC-4626)

Share price, redemption queues, strategy rotation signals. Vaults abstract yield strategies behind a standard interface. Share price deviating from NAV indicates stress. Large redemption queues signal loss of confidence. Strategy rotation (the vault's underlying allocations changing) is a meta-signal about what the vault operator believes about the market.

### Staking

Delegation flows and reward rates. Net delegation (new stakes minus unstakes) measures confidence in a validator or protocol. Reward rate changes signal protocol health or inflation adjustments. Staking queues (entry and exit) create predictable supply shocks.

### Restaking

AVS (Actively Validated Service) security budgets and operator selection patterns. The amount of restaked ETH backing an AVS is a direct measure of the market's trust in that service. Operator concentration (few operators controlling most restaked capital) is a risk indicator with no TradFi analog.

### Derivatives (perpetuals, options)

Funding rates, open interest, and Greeks. Perpetual funding rates directly encode market sentiment: positive funding means longs pay shorts (bullish bias), negative means the reverse. Open interest changes measure new capital entering or leaving the market. On-chain options provide observable Greeks -- delta, gamma, theta, vega -- computed from live market prices rather than model assumptions.

### Yield (Pendle)

PT (Principal Token) and YT (Yield Token) prices decompose an interest-bearing asset into its principal and yield components. The implied yield curve from PT/YT prices across maturities is a DeFi-native term structure -- the market's collective expectation of future yields. This is a fixed-income concept translated into DeFi tokens, analyzable through yield curve TA (steepening, flattening, inversion).

### Streaming payments (Sablier)

Creation and cancellation rates for payment streams. A spike in stream creations signals new economic activity (payroll, vesting, subscription payments). A spike in cancellations signals stress (employees leaving, subscriptions churning, vesting being revoked). The ratio of creation to cancellation is a leading indicator of protocol ecosystem health.

### Gas and MEV

Base fee oscillation follows EIP-1559 dynamics: a control system that targets 50% block utilization. The base fee's autocorrelation structure, mean-reversion speed, and volatility clustering are analyzable with standard time-series TA. Priority fee distributions reveal willingness to pay for urgency -- a proxy for market intensity.

MEV (Maximal Extractable Value) creates observable patterns: sandwich attacks leave distinctive footprints (two swaps bracketing a victim transaction), JIT (just-in-time) liquidity appears and disappears within a single block, and backrunning creates correlated transaction sequences.

### Emerging primitives

**Intent-based trading** (UniswapX, CoW Protocol): orders filled by solvers rather than AMMs. Solver competition dynamics, fill rates, and price improvement metrics form a new observation space.

**Real-world assets (RWAs)**: tokenized treasuries, real estate, commodities. Correlation between on-chain prices and off-chain reference prices creates arbitrage-like dynamics detectable through TA.

**Cross-chain messaging** (LayerZero, Hyperlane): message volume, latency distributions, and fee patterns between chains. Cross-chain TA requires tracking state across multiple observation streams simultaneously.

**Account abstraction** (ERC-4337): user operation bundling, paymaster usage patterns, and smart account deployment rates. New metrics with no traditional analog.

**Prediction markets** (Polymarket): outcome token prices directly encode probability estimates. Price momentum on a prediction market is a statement about how beliefs are changing, analyzable through standard momentum TA.

## The cybernetic loop: TA-cognition feedback cycle [SPEC]

The full TA integration forms a closed loop:

```
Observe (Witness)
    |
    v
Encode (HDC pattern algebra, Doc 1)
    |
    v
Analyze (TA subsystems, Docs 2-9)
    |
    v
Integrate (IIT Phi check, Doc 10)
    |
    v
Bid (Attention auction: TA insights influence bidding)
    |
    v
Act (Oracle decides, tools execute)
    |
    v
Reflect (compare prediction to outcome, store episode)
    |
    v
Learn (Grimoire consolidation, semantic extraction)
    |
    v
Dream (NREM replay, REM counterfactual exploration)
    |
    v
(back to Observe, with updated attention, refined patterns,
 and evolved signal weights)
```

Each iteration through this loop changes the Golem's TA capabilities. Patterns that produced accurate predictions gain semantic weight. Patterns that failed lose it. The attention filter shifts toward observations where TA provides the most value. Signal families evolve through Hebbian selection. The Golem's "technical analysis" after 10,000 Theta ticks looks nothing like it did at boot. It has adapted to the specific markets, protocols, and conditions it encountered.

This is not optimization toward a fixed objective. The objective itself evolves. A Golem that begins by tracking simple swap volume momentum might, through accumulated experience and dream-cycle exploration, develop compound patterns that integrate volume momentum with liquidity curvature and causal density. The TA system grows with the Golem.

## The 11-document TA series [SPEC]

This document (Doc 0) provides the context. The remaining 10 documents each address a specific aspect of TA integration:

**Doc 1: Hyperdimensional pattern algebra.** Encodes all TA signals -- price patterns, volume profiles, liquidity distributions, on-chain activity sequences -- as 10,240-bit Binary Spatter Code hypervectors. Pattern matching reduces to Hamming distance. Pattern composition uses XOR binding and majority-vote bundling. This gives TA subsystems a universal, fixed-size, algebraically composable representation for any market pattern.

**Doc 2: Spectral liquidity manifolds.** Models DeFi liquidity as a Riemannian manifold where the metric tensor is derived from pool reserve curves. Curvature, geodesics, and parallel transport replace flat-space assumptions. A Uniswap v3 pool's concentrated liquidity creates a manifold with variable curvature; navigating it requires differential geometry, not linear interpolation.

**Doc 3: Adaptive signal metabolism.** Applies Hebbian learning ("neurons that fire together wire together") and economic selection pressure to evolve which TA signals the Golem pays attention to. Signal families compete for attention budget. Families that predict well reproduce; families that fail are pruned. The Golem's indicator set is not fixed -- it evolves.

**Doc 4: Causal microstructure discovery.** Moves beyond correlation to causation. Uses interventional testing via mirage-rs fork simulation to discover causal relationships in market microstructure. Does a large swap cause a funding rate change, or do they share a common cause? Counterfactual simulation can distinguish them.

**Doc 5: Predictive geometry.** Maps topological features (from the existing TDA pipeline) to trajectory predictions. Persistent homology detects regime transitions; predictive geometry translates those detections into actionable forecasts about where prices, volatility, and liquidity are heading.

**Doc 6: Resonant pattern ecosystem.** Treats TA patterns as organisms in an evolutionary ecosystem. Patterns compete for attention, reproduce through mutation and crossover, and die when they stop performing. The pattern population co-evolves with the market.

**Doc 7: DeFi-native indicators.** Develops indicators that are impossible in traditional finance. Impermanent loss velocity, tick migration patterns, liquidation cascade proximity, MEV extraction rate, governance vote momentum. These exploit the transparency and composability unique to DeFi.

**Doc 8: Adversarial signal robustness.** Addresses the fact that DeFi markets are adversarial. Participants can read the Golem's transactions, infer its strategy, and manipulate the signals it relies on. TA subsystems need adversarial robustness: detection of manipulated signals, graceful degradation under poisoned data, and red-team dreaming that simulates adversarial attacks during REM sleep.

**Doc 9: Somatic markers and embodied market intuition.** Implements Damasio's somatic marker hypothesis for TA. The Golem accumulates body-like responses to market patterns -- a "gut feeling" computed from the association between TA signal configurations and past outcomes. This provides fast, pre-cognitive decision support that bypasses deliberative reasoning.

**Doc 10: Emergent multi-scale intelligence.** The capstone. Applies Tononi's Integrated Information Theory (IIT) to measure how well the 10 TA subsystems work together as a unified analytical system. Phi quantifies integration. Falling Phi diagnoses fragmentation. The goal is not 10 independent indicators but one emergent intelligence built from 10 components.

## Evaluation protocol [SPEC]

Every claim in this series must be falsifiable. The evaluation framework spans all 10 documents:

**Null hypothesis.** TA integration provides no improvement over the Golem's existing perception-to-action pipeline (triage scoring, Bayesian surprise, HDC fingerprinting).

**Test protocol.**

1. Run two Golem populations on historical DeFi data (minimum 6 months, at least one regime transition): a control population with no TA subsystems and a test population with the full TA pipeline.
2. Both populations use identical initial capital, mortality parameters, and attention budgets.
3. Measure risk-adjusted returns (Sharpe ratio, maximum drawdown), prediction accuracy (calibration curve, Brier score), attention efficiency (information gain per attended item), and survival time (how long Golems live before death).
4. Compare distributions using Welch's t-test for continuous metrics and Kaplan-Meier curves for survival analysis.

**Primary prediction.** The TA-integrated population will show higher risk-adjusted returns and longer survival times, with the survival advantage appearing primarily through the epistemic clock (better predictions extend epistemic lifetime).

**Secondary predictions.**

- TA patterns will show positive fitness selection over Golem lifetimes: the set of patterns in a Golem's death testament will have higher average prediction accuracy than the set at boot.
- `ta_phi` will positively correlate with decision quality (risk-adjusted returns per Theta tick).
- Adversarial robustness will show its value during periods of detected manipulation: the TA-integrated population will suffer smaller drawdowns during manipulation events.

**Failure criteria.** If the TA-integrated population does not show statistically significant improvement on the primary metrics after 6 months of historical simulation, the complexity is not justified. Individual subsystems can still provide value -- the evaluation also measures each subsystem's marginal contribution -- but the claim of emergent integrated TA intelligence would not hold.

## References

1. Buzsaki, G. (2006). *Rhythms of the Brain.* Oxford University Press. Neural oscillatory hierarchies motivating the Gamma/Theta/Delta timescale design.

2. McClelland, J.L., McNaughton, B.L., and O'Reilly, R.C. (1995). "Why there are complementary learning systems in the hippocampus and neocortex." *Psychological Review,* 102(3), 419-457. CLS theory underlying the Grimoire's dual-store architecture.

3. Kumaran, D., Hassabis, D., and McClelland, J.L. (2016). "What learning systems do intelligent agents need?" *Trends in Cognitive Sciences,* 20(7), 512-534. Updated CLS theory with computational models of replay scheduling.

4. Shannon, C.E. (1948). "A mathematical theory of communication." *Bell System Technical Journal,* 27(3), 379-423. Information-theoretic foundations for the mortality framework.

5. Shannon, C.E. (1959). "Coding theorems for a discrete source with a fidelity criterion." *IRE National Convention Record,* 7(4), 142-163. Rate-distortion theory underlying the epistemic death clock.

6. Vickrey, W. (1961). "Counterspeculation, auctions, and competitive sealed tenders." *Journal of Finance,* 16(1), 8-37. Foundation for the VCG attention auction.

7. Clarke, E.H. (1971). "Multipart pricing of public goods." *Public Choice,* 11(1), 17-33. VCG mechanism extension.

8. Groves, T. (1973). "Incentives in teams." *Econometrica,* 41(4), 617-631. VCG mechanism completion.

9. Borbely, A.A. (1982). "A two process model of sleep regulation." *Human Neurobiology,* 1(3), 195-204. Two-process sleep model underlying SleepPressure.

10. Graf, T.M. and Lemire, D. (2022). "Binary Fuse Filters: Fast and Smaller Than Xor Filters." *ACM Journal of Experimental Algorithmics,* 27, 1-15. Filter design used in bardo-witness.

11. Bhatia, S., Hooi, B., Yoon, M., Shin, K., and Faloutsos, C. (2022). "MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams." *Proceedings of the AAAI Conference on Artificial Intelligence.* Anomaly detection in triage Stage 2.

12. Masson, C., Rim, J.E., and Lee, H.K. (2019). "DDSketch: A Fast and Fully-Mergeable Quantile Sketch with Relative-Error Guarantees." *Proceedings of the VLDB Endowment,* 12(12). Quantile estimation in triage Stage 2.

13. Gidea, M. and Katz, Y. (2018). "Topological data analysis of financial time series: Landscapes of crashes." *Physica A,* 491, 820-834. Empirical validation of TDA for crash detection.

14. Cohen-Steiner, D., Edelsbrunner, H., and Harer, J. (2007). "Stability of persistence diagrams." *Discrete & Computational Geometry,* 37(1), 103-120. Stability theorem for persistent homology.

15. Tononi, G. (2004). "An information integration theory of consciousness." *BMC Neuroscience,* 5, 42. IIT framework adapted for cognitive integration monitoring.

16. Damasio, A.R. (1994). *Descartes' Error: Emotion, Reason, and the Human Brain.* Putnam. Somatic marker hypothesis underlying Doc 9.

17. Friston, K., Kilner, J., and Harrison, L. (2006). "A free energy principle for the brain." *Journal of Physiology-Paris,* 100(1-3), 70-87. Active inference framework underlying the cybernetic feedback loop.

18. Nisan, N., Roughgarden, T., Tardos, E., and Vazirani, V.V. (2007). *Algorithmic Game Theory.* Cambridge University Press. VCG mechanism theory (Chapter 9).

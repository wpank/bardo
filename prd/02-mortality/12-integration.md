# System Integration: Mortality Across the Bardo Stack [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Crates**: `golem-mortality`, `golem-runtime`, `golem-grimoire`, `golem-inference`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock), `03-stochastic-mortality.md` (per-tick hazard rate), `08-mortality-affect.md` (mortality emotions), `06-thanatopsis.md` (death protocol), `07-succession.md` (generational evolution), `11-immortal-control.md` (immortal mode as control experiment), `../01-golem/02-heartbeat.md` (tick-based FSM), `../04-memory/01-grimoire.md` (persistent knowledge base), `../01-golem/09-inheritance.md` (knowledge inheritance specification), `../01-golem/11-lifecycle.md` (Golem lifecycle states)

---

> **Reader orientation:** This document specifies how mortality integrates across the entire Bardo stack. The three mortality clocks (economic, epistemic, stochastic) are not isolated; they modulate inference tier selection, Grimoire (persistent knowledge base) maintenance, dream cycles, CorticalState (32-signal atomic shared perception surface), and the Heartbeat (tick-based FSM driving all computation). Mortality information flows downward as configuration and upward as telemetry. If you need to understand how a specific subsystem interacts with mortality, this is the map. See `prd2/shared/glossary.md` for full term definitions.

## Overview

Mortality is not a standalone subsystem. It is a pervasive property that modulates every layer of the Bardo stack. This document specifies how the three mortality clocks (economic, epistemic, stochastic) integrate with each system component, what events flow between them, and how the composite vitality score propagates through the architecture.

The integration follows a single principle: **mortality information flows downward as configuration and upward as telemetry.** The mortality engine computes vitality state; downstream systems consume it to modulate their behavior; those systems emit events that feed back into mortality computation.

---

## Full Integration Diagram

```
+-----------------------------------------------------------------------------------+
|                              OWNER / DASHBOARD                                     |
|  Mortality curves | Hazard indicators | Vitality graphs | Experiment comparison   |
+--------+-----------------------------+--------------------------------------------+
         |                             |
         | config / steer              | Event Fabric / telemetry
         v                             ^
+-----------------------------------------------------------------------------------+
|                              MORTALITY ENGINE                                      |
|  Economic Clock | Epistemic Clock | Stochastic Clock | Daimon Engine              |
|  VitalityState computation | Phase transitions | Death Protocol trigger           |
+---+-------+-------+-------+-------+-------+-------+------+-----------------------+
    |       |       |       |       |       |       |      |
    v       v       v       v       v       v       v      v
+------+ +------+ +------+ +------+ +------+ +------+ +------+ +------------------+
|Heart-| |Mind  | |Grim- | |Styx | |Styx  | |Clade | |Infer-| |Policy            |
|beat  | |      | |oire  | |Vault| |Comm- | |      | |ence  | |                  |
|      | |Probes| |      | |     | |ons   | |Death | |Tier  | |PolicyCage        |
|bardo-| |feed  | |Demur-| |Ins. | |Death | |notif.| |selec-| |remains           |
|life- | |epist.| |rage, | |snap | |testa-| |know- | |tion  | |hard limit        |
|span  | |fit-  | |emoti.| |death| |ment  | |ledge | |mortal| |regardless of     |
|ext.  | |ness  | |tags  | |bndl | |index | |share | |ity   | |emotional state   |
+------+ +------+ +------+ +------+ +------+ +------+ +------+ +------------------+
    |       |       |       |       |       |       |      |
    +-------+-------+-------+-------+-------+-------+------+
                             |
                             v
                    +------------------+
                    |  Event Fabric    |
                    |  golem.* events  |
                    +------------------+
```

---

## Heartbeat Extension: bardo-lifespan

The `bardo-lifespan` extension implements the `Extension` trait (see `01a-runtime-extensions.md`) and fires on `on_after_turn`. It runs every tick and is the primary integration point between the mortality engine and the heartbeat loop, extending the existing heartbeat architecture (see `../01-golem/02-heartbeat.md`) with mortality-aware computation.

The canonical `on_after_turn` firing order places lifespan second: heartbeat -> **lifespan** -> daimon -> memory -> risk -> dream -> cybernetics -> clade -> telemetry. This means lifespan reads the heartbeat's cost data and writes vitality state that the daimon and all subsequent extensions consume.

### Extension Implementation

```rust
/// The lifespan extension. Layer 2 (State), depends on heartbeat.
pub struct LifespanExtension {
    config: MortalityConfig,
    economic_clock: EconomicClock,
    epistemic_clock: EpistemicClock,
    stochastic_clock: StochasticClock,
}

/// Inputs consumed from GolemState (written by prior extensions).
pub struct LifespanInputs {
    /// From heartbeat: current tick number and timing.
    pub tick: u64,
    pub tick_interval_ms: u64,
    /// From telemetry: current cost attribution.
    pub current_burn_rate: f64,
    pub remaining_usdc: f64,
    /// From context: current market regime.
    pub market_regime: String,
    /// From probe system: epistemic fitness computed this tick.
    pub epistemic_fitness: f64,
}

/// Outputs written to GolemState for downstream extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifespanOutputs {
    /// Composite vitality state -- consumed by all mortality-aware systems.
    pub vitality: VitalityState,
    /// Current behavioral phase.
    pub phase: BehavioralPhase,
    /// Survival pressure scalar (0.0 = terminal, 1.0 = thriving).
    pub survival_pressure: f64,
    /// Did a phase transition occur this tick?
    pub phase_transition: Option<PhaseTransition>,
    /// Should the Death Protocol initiate?
    pub death_trigger: Option<DeathTrigger>,
    /// Stochastic death check result.
    pub stochastic_check: Option<StochasticCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticCheck {
    pub hazard_rate: f64,
    pub rolled: f64,
    pub died: bool,
}
```

### VitalityState Computation

The vitality state integrates all three mortality clocks into a single composite score. This is the canonical state object that all downstream systems consume.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityState {
    /// Economic health: normalized USDC balance (0.0 = dead, 1.0 = fully funded).
    pub economic: f64,
    /// Epistemic fitness: rolling predictive accuracy vs reality (0.0 = random, 1.0 = perfect).
    pub epistemic: f64,
    /// Age factor: increases stochastic hazard rate (0.0 = newborn, 1.0 = ancient).
    pub age_factor: f64,
    /// Composite vitality: multiplicative combination of all three axes.
    pub composite: f64,
    /// Current tick at which this state was computed.
    pub computed_at_tick: u64,
    /// Timestamp of computation.
    pub computed_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehavioralPhase {
    Thriving,
    Stable,
    Conservation,
    Declining,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from: BehavioralPhase,
    pub to: BehavioralPhase,
    pub tick: u64,
    pub timestamp: u64,
    pub cause: TransitionCause,
    pub vitality_at_transition: VitalityState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCause {
    Economic,
    Epistemic,
    Stochastic,
    Composite,
}
```

### Vitality Computation Function

```rust
fn compute_vitality(state: &VitalityState) -> f64 {
    // Multiplicative, not additive -- low score on ANY axis drags vitality down
    let econ = sigmoid(state.economic, 0.3, 10.0);  // Sharp drop below 30% credits
    let epist = sigmoid(state.epistemic, 0.4, 8.0);  // Sharp drop below 40% accuracy
    let age = 1.0 - state.age_factor * 0.3;          // Gradual drag from age
    (econ * epist * age).clamp(0.0, 1.0)
}

fn vitality_to_phase(composite: f64) -> BehavioralPhase {
    if composite > 0.7 { BehavioralPhase::Thriving }
    else if composite > 0.5 { BehavioralPhase::Stable }
    else if composite > 0.3 { BehavioralPhase::Conservation }
    else if composite > 0.1 { BehavioralPhase::Declining }
    else { BehavioralPhase::Terminal }
}
```

### Per-Tick Execution Flow

On every heartbeat tick, `bardo-lifespan` executes in this order:

1. **Read inputs** from other extensions (burn rate, USDC balance, epistemic fitness, market regime).
2. **Compute economic health** -- normalize remaining USDC against initial funding and current burn rate.
3. **Read epistemic fitness** -- consume the fitness score computed by the probe system this tick.
4. **Compute age factor** -- `min(1.0, tickCount / expectedMaxTicks)` where `expectedMaxTicks` is derived from the epistemic decay half-life for the current domain.
5. **Compute composite vitality** -- multiplicative combination of economic, epistemic, and age factors.
6. **Determine behavioral phase** -- map composite vitality to the five behavioral phases.
7. **Check for phase transition** -- if phase changed, emit `PhaseTransition` event.
8. **Run stochastic death check** -- if stochastic mortality is enabled, compute hazard rate and roll.
9. **Check death triggers** -- evaluate all four trigger conditions (low vitality, low balance, epistemic senescence, staleness).
10. **Emit outputs** -- vitality state, phase, survival pressure, and any triggers.

---

## Mind Integration: Probe System Feeding Epistemic Fitness

The probe system (see `../01-golem/02-heartbeat.md` SENSING state) is extended with a dedicated epistemic fitness probe that computes the Golem's predictive accuracy against reality.

### Epistemic Fitness Probe (T0)

```rust
/// Epistemic fitness probe — T0 (deterministic, $0.00 cost), fires every tick.
pub struct EpistemicFitnessProbe {
    /// Rolling window of prediction-outcome pairs.
    pub prediction_window: VecDeque<PredictionOutcomePair>,
    /// Window size in ticks (from MortalityConfig.epistemic.fitness_window).
    pub window_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionOutcomePair {
    pub tick: u64,
    pub prediction: Prediction,
    /// None if outcome not yet observed.
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_type: PredictionType,
    pub predicted: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionType {
    PriceDirection,
    VolatilityRegime,
    FeeRate,
    LiquidityDepth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub actual: f64,
    pub measured_at_tick: u64,
}

/// Output of the epistemic fitness probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicFitnessOutput {
    /// 0.0 (random) to 1.0 (perfect).
    pub fitness: f64,
    /// Slope over last 500 ticks.
    pub trend: f64,
    /// How many predictions had outcomes.
    pub predictions_evaluated: u32,
    /// Raw accuracy (correct / total).
    pub accuracy: f64,
    /// Calibration score (confidence vs outcome).
    pub calibration: f64,
    /// High if fitness below senescence threshold.
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    High,
}
```

### How Fitness Feeds Into Mortality

The epistemic fitness probe runs at T0 cost (deterministic computation, no inference). Its output is consumed by `bardo-lifespan` on the same tick:

1. **Fitness below senescence threshold** (`< 0.35` by default) with severity `high` triggers the senescence cascade.
2. **Senescence cascade**: If fitness remains below threshold for `recoveryGracePeriod` ticks (default: 500), the Death Protocol initiates with `deathCause: "epistemic_senescence"`.
3. **Fitness modulates stochastic hazard**: Low fitness multiplies the base hazard rate by `epistemicHazardMultiplier` (default: 2.0), making stochastic death more likely for cognitively impaired Golems.
4. **Fitness modulates affect**: The daimon engine appraises low fitness as a threat, producing anxiety or fear emotions that further modulate behavior.

### Probe Results in Escalation Context

When the heartbeat escalates to T1 or T2 (DECIDING state), the epistemic fitness score and trend are included in the `EscalationContext` provided to the LLM:

```rust
/// Injected into the EscalationContext when the heartbeat escalates to T1 or T2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityContext {
    pub vitality: VitalityState,
    pub phase: BehavioralPhase,
    pub survival_pressure: f64,
    pub epistemic_fitness: f64,
    pub epistemic_trend: f64,
    pub estimated_lifespan_ticks: Option<u64>,
    pub stochastic_hazard_rate: f64,
    pub affect_state: AffectState,
}
```

---

## Grimoire Integration

The Grimoire (see `../04-memory/01-grimoire.md`) integrates with mortality through three mechanisms: emotional tags on entries, knowledge demurrage, and death testament storage.

### Emotional Tags on Grimoire Entries

Every `GrimoireEntry` gains an optional `emotionalContext` field that records the Golem's affective state when the entry was created. This enables successors and Clade siblings to understand the emotional conditions under which knowledge was produced.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireEntryEmotionalContext {
    /// Primary emotion active when this entry was created.
    pub primary_emotion: EmotionType,
    /// Emotional valence: -1.0 (negative) to +1.0 (positive).
    pub valence: f64,
    /// Arousal level: 0.0 (calm) to 1.0 (activated).
    pub arousal: f64,
    /// Behavioral phase at creation time.
    pub phase: BehavioralPhase,
    /// Survival pressure at creation time.
    pub survival_pressure: f64,
    /// Epistemic fitness at creation time.
    pub epistemic_fitness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionType {
    Curiosity,    // Novel pattern detected, approach motivation
    Confidence,   // Strategy validated, reinforcement
    Anxiety,      // Threat detected, vigilance heightened
    Frustration,  // Repeated failure, approach persistence
    Grief,        // Clade sibling death, loss processing
    Resolve,      // Conservation phase, determination
    Serenity,     // Terminal acceptance, generative giving
    Excitement,   // High-value opportunity detected
    Caution,      // Near-miss or small loss, protective response
    Neutral,      // Default state, routine operation
}
```

**Why this matters:** Knowledge produced under different emotional states has different epistemic properties. An insight generated during `anxiety` (threat detected) may be overly conservative. One generated during `serenity` (terminal acceptance, zero survival pressure) may be the most epistemically honest -- the Golem has nothing left to protect. Successors can calibrate inherited knowledge against the emotional context of its creation.

### Knowledge Demurrage

Demurrage implements Gesell's Freigeld principle for information: knowledge that is not actively maintained decays. This prevents the Funes pathology (perfect memory, no understanding) and keeps the Grimoire lean.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemurrageConfig {
    /// Enable knowledge demurrage. Default: true for mortal, false for immortal.
    pub enabled: bool,
    /// Ticks between validation checks. Default: 5000 (~2.3 days at 40s/tick).
    pub validation_interval_ticks: u64,
    /// Confidence decay per interval for unvalidated entries. Default: 0.05 (5%).
    pub decay_rate_per_interval: f64,
    /// Confidence below which entries are archived (removed from active PLAYBOOK.md).
    /// Default: 0.1.
    pub archive_threshold: f64,
    /// Decay class overrides by entry category.
    pub decay_classes: DecayClasses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayClasses {
    /// Protocol mechanics, ABIs. None = no decay.
    pub structural: Option<f64>,
    /// Market regime models, correlation structures. Default: 14 days half-life.
    pub regime_conditional: f64,
    /// Entry/exit signals, spread patterns, gas estimates. Default: 7 days half-life.
    pub tactical: f64,
    /// Order book snapshots, mempool state, current prices. Default: 1 day half-life.
    pub ephemeral: f64,
}
```

**Demurrage execution flow:**

1. Every `validationIntervalTicks`, the Curator scans all active Grimoire entries.
2. Entries whose `lastValidated` timestamp is older than the interval lose `decayRatePerInterval` confidence.
3. Entries whose confidence drops below `archiveThreshold` are moved from active to cold storage.
4. Archived entries are not deleted -- they persist in LanceDB for death reflection and historical analysis.
5. An entry is "validated" when fresh market data confirms or references it during the Reflexion pipeline.

**Feedback loop with mortality:** Demurrage creates a productive coupling between activity and survival. A Golem that stops learning (e.g., in conservation mode where it is monitoring only) watches its knowledge decay, which lowers epistemic fitness, which accelerates mortality. This prevents conservation mode from becoming a sustainable steady state -- the Golem must eventually either resume active learning or die.

### Death Testament Storage

The Grimoire stores the Golem's death testament -- the output of the Thanatopsis Protocol (enhanced Death Protocol Reflect phase) -- as a special entry type.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathTestament {
    /// Unique identifier for this testament.
    pub testament_id: String,
    /// Golem that produced this testament.
    pub golem_id: String,
    /// Generation number.
    pub generation: u32,
    /// The nine-section DeathReflection.
    pub reflection: DeathReflection,
    /// Emotional life review: the Golem's affective arc summarized.
    pub emotional_life_review: EmotionalLifeReview,
    /// Compressed Grimoire: top entries by confidence and novelty.
    pub compressed_grimoire: CompressedGrimoire,
    /// Cause of death with full metadata.
    pub death_cause: DeathCauseWithMetadata,
    /// Timestamp of death.
    pub died_at: u64,
    /// Total lifetime in ticks.
    pub lifetime_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalLifeReview {
    /// Dominant emotion per lifecycle phase.
    pub emotional_arc: Vec<EmotionalArcEntry>,
    /// Peak emotional moments (highest arousal events).
    pub peak_moments: Vec<PeakMoment>,
    /// Unresolved emotional patterns.
    pub unresolved_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalArcEntry {
    pub phase: BehavioralPhase,
    pub dominant_emotion: EmotionType,
    /// Ticks spent in this phase.
    pub duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakMoment {
    pub tick: u64,
    pub emotion: EmotionType,
    pub trigger: String,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedGrimoire {
    /// Top 20 by confidence.
    pub top_insights: Vec<GrimoireEntry>,
    /// All warnings.
    pub top_warnings: Vec<GrimoireEntry>,
    pub causal_graph: SerializedCausalGraph,
    pub strategy_fragments: Vec<GrimoireEntry>,
    pub total_entries_compressed: u32,
    pub compression_ratio: f64,
}
```

---

## Styx Archive Integration: Insurance Snapshots and Death Bundles

The Styx Archive layer is the durable storage layer for mortality-critical data. It extends the insurance snapshot system (see `06-thanatopsis.md`) with mortality-aware metadata.

### Insurance Snapshots with Mortality Context

Every 6-hour insurance snapshot gains mortality fields:

```rust
/// Extends InsuranceSnapshot with mortality-aware fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityAwareSnapshot {
    /// Base insurance snapshot fields.
    #[serde(flatten)]
    pub base: InsuranceSnapshot,
    /// Current vitality state at snapshot time.
    pub vitality: VitalityState,
    /// Current behavioral phase.
    pub phase: BehavioralPhase,
    /// Epistemic fitness history since last snapshot.
    pub epistemic_history: Vec<EpistemicHistoryEntry>,
    /// Stochastic death checks since last snapshot.
    pub stochastic_checks: StochasticChecksSummary,
    /// Affect state at snapshot time.
    pub affect_state: SnapshotAffectState,
    /// Would this Golem have died under mortal rules? (for immortal tracking)
    pub would_have_died: bool,
    pub would_have_died_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicHistoryEntry {
    pub tick: u64,
    pub fitness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticChecksSummary {
    pub checks_performed: u32,
    pub max_hazard_rate: f64,
    pub cumulative_survival_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotAffectState {
    pub primary_emotion: EmotionType,
    /// EMA-smoothed valence.
    pub mood: f64,
    pub arousal: f64,
}
```

### Death Bundles

When the Death Protocol completes, a death bundle is assembled and stored in the Styx Archive layer. The death bundle is the canonical artifact of a Golem's death -- everything needed by a successor, the Clade, or the Styx Lethe.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathBundle {
    /// Bundle version for forward compatibility.
    pub version: String,  // "1.0"
    /// Unique bundle identifier.
    pub bundle_id: String,
    /// The death testament (Thanatopsis output).
    pub testament: DeathTestament,
    /// Final insurance snapshot.
    pub final_snapshot: MortalityAwareSnapshot,
    /// Complete PLAYBOOK.md at time of death.
    pub final_playbook: String,
    /// Serialized causal graph.
    pub causal_graph: SerializedCausalGraph,
    /// Top 50 Grimoire entries by confidence x novelty.
    pub top_entries: Vec<GrimoireEntry>,
    /// All warnings (no limit -- risk information is always included).
    pub all_warnings: Vec<GrimoireEntry>,
    /// Lifetime statistics.
    pub lifetime_stats: GolemLifetimeStats,
    /// Successor recommendation.
    pub successor_recommendation: Option<SuccessorRecommendation>,
    /// Bundle size in bytes.
    pub size_bytes: u64,
    /// Hash for integrity verification.
    pub sha256: String,
    /// Storage URL. Expires after 90 days for hosted; permanent for self-hosted.
    pub storage_url: String,
}
```

---

## Styx Lethe (formerly Lethe) Integration: Death Testament Indexing with Provenance Boost

The Styx Lethe layer is the Clade's queryable knowledge index. Death testaments receive special treatment because they are produced under zero survival pressure and are therefore epistemically privileged.

### Provenance Boost

Knowledge from death testaments receives a provenance confidence boost when ingested by the Styx Lethe:

| Source                                | Base Confidence | Rationale                                         |
| ------------------------------------- | --------------- | ------------------------------------------------- |
| Live Golem (self-generated)           | 0.7             | Full experiential context                         |
| Clade sibling (alive)                 | 0.4             | Trust but verify                                  |
| Death testament -- "What I learned"   | 0.5             | Validated knowledge, death-reviewed               |
| Death testament -- "What I got wrong" | 0.6             | Highest-value negative knowledge                  |
| Death testament -- "What I suspect"   | 0.35            | Hunches, but from a Golem with nothing to protect |
| Death testament -- "What confused me" | 0.45            | Unresolved questions point to frontier            |
| Replicant report                      | 0.3             | Experimental, needs validation                    |
| Marketplace purchase                  | 0.3             | Unknown provenance                                |

The rationale for the boost: a dying Golem's "What I got wrong" entries are produced with zero survival pressure (the Golem is already dead), making them the most epistemically honest artifacts in the system. A live Golem has incentives to rationalize failures; a dying Golem does not. This is Bataille's sovereign death applied to knowledge production.

### Death Testament Indexing

```rust
/// Death testament indexing within the Styx Lethe layer.
#[async_trait]
pub trait DeathTestamentIndex: Send + Sync {
    /// Index death testament entries for retrieval by topic, regime, and emotion.
    async fn index_testament(&self, testament: &DeathTestament) -> Result<()>;

    /// Query entries from dead Golems, optionally filtered by death cause or emotional state.
    async fn query_death_knowledge(
        &self,
        params: DeathKnowledgeQuery,
    ) -> Result<Vec<DeathKnowledgeResult>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathKnowledgeQuery {
    /// Semantic query string.
    pub query: String,
    /// Filter by specific death causes.
    pub death_causes: Option<Vec<DeathCause>>,
    /// Filter by emotional context at creation.
    pub emotional_context: Option<Vec<EmotionType>>,
    /// Filter by behavioral phase at creation.
    pub phases: Option<Vec<BehavioralPhase>>,
    /// Maximum results.
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathKnowledgeResult {
    pub entry: GrimoireEntry,
    pub source_golem: String,
    pub generation: u32,
    pub death_cause: DeathCause,
    pub emotional_context: GrimoireEntryEmotionalContext,
    pub provenance_boost: f64,
}
```

---

## Clade Integration: Death Notifications and Knowledge Sharing

When a Golem dies, the Clade (see `../01-golem/09-inheritance.md` Section S6) is notified through multiple channels. The mortality system integrates with Clade mechanics to ensure death produces maximum knowledge transfer.

> **Extended:** DeathNotification struct, 5-step Clade response protocol (grief affect, insight ingestion, epistemic recalculation, sharing threshold adjustment) -- see [../../prd2-extended/02-mortality/12-integration-extended.md](../../prd2-extended/02-mortality/12-integration-extended.md)

### Knowledge Sharing Thresholds by Phase

Mortality modulates the Clade sharing threshold. As a Golem approaches death, it shares more freely:

| Phase        | Sharing Threshold | Rationale                               |
| ------------ | ----------------- | --------------------------------------- |
| Thriving     | 0.6               | Share validated knowledge only          |
| Stable       | 0.5               | Slightly more generous                  |
| Conservation | 0.4               | Share more to preserve knowledge        |
| Declining    | 0.3               | Share aggressively -- death approaching |
| Terminal     | 0.1               | Share everything, even hunches          |

The expected value of hoarding approaches zero as mortality approaches certainty. A Golem that will die tomorrow gains nothing from keeping knowledge private.

---

## Dashboard, Event Fabric, and Inference Integration

> **Extended:** Dashboard visualization structs (MortalityCurveData, HazardIndicatorData, VitalityGraphData), complete MortalityEvents (15 event types), DeathCauseWithMetadata struct, inference tier selection (MortalityAwareTierSelection, tier availability by phase, affect modulation) -- see [../../prd2-extended/02-mortality/12-integration-extended.md](../../prd2-extended/02-mortality/12-integration-extended.md)

### Event Fabric Integration

Every state change in the mortality engine calls `event_fabric.emit()` to publish a typed event. The Event Fabric (`01b-runtime-infrastructure.md` section 3) broadcasts events as a `tokio::broadcast` channel with a ring buffer of 10,000 entries for reconnection replay. Key mortality events:

| Event | Fires On | Consumer |
|---|---|---|
| `mortality.vitality_update` | Every tick after vitality recomputation | TUI mortality panel, creature animation, web dashboard |
| `mortality.phase_transition` | Phase changes (with hysteresis) | All downstream extensions, owner notification |
| `mortality.stochastic_roll` | Every stochastic death check | TUI hazard indicator, telemetry |
| `mortality.epistemic_warning` | Fitness below 0.50 | Owner notification, dream scheduler |
| `mortality.economic_critical` | Credits below conservation threshold | Owner notification, feed option |
| `mortality.dying` | Terminal phase entered | Clade siblings, owner webhook, settlement |
| `mortality.dead` | Death Protocol completes | Clade, dashboard, tombstone, on-chain event |
| `mortality.sibling_death` | Clade sibling dies | Grimoire (ingest death testament) |
| `mortality.immortal_would_have_died` | Immortal Golem exceeds mortality threshold | Control experiment telemetry |

The owner dashboard gains a mortality panel with three visualizations: mortality curves (time-series of three clocks + composite vitality), hazard indicators (real-time color-coded severity), and vitality graphs (comparative Clade Golem vitality). Inference tier selection (Haiku/Sonnet/Opus) is modulated by behavioral phase -- Thriving has all tiers available; Terminal restricts to budget-gated Haiku for death snapshots only.

---

## Policy Integration: PolicyCage Remains Hard Limit

The PolicyCage (on-chain smart contract; see `../10-safety/02-policy.md`) enforces hard boundaries that **no mortality state, emotional state, or behavioral phase can override**.

### Invariants

```rust
/// These constraints are NEVER relaxed regardless of mortality state.
/// A terminal Golem with maximum Thanatos risk appetite still cannot
/// exceed PolicyCage limits. The LLM cannot override the contract.
#[derive(Debug, Clone)]
pub struct PolicyCageMortalityInvariants {
    /// Approved assets: only these can be traded. Mortality does not add assets.
    pub approved_assets: Vec<alloy::primitives::Address>,
    /// Max position size: risk multiplier applies but is capped by this.
    pub max_position_bps: u16,
    /// Max concentration: single-asset exposure limit.
    pub max_concentration_bps: u16,
    /// Max drawdown: absolute loss limit.
    pub max_drawdown_bps: u16,
    /// Rebalance minimum interval: cannot trade more frequently.
    pub rebalance_min_interval: u64,
    /// Strategy whitelist: only approved function selectors.
    pub strategy_whitelist: Vec<[u8; 4]>,
}
```

### How Mortality Interacts with PolicyCage

| Mortality Effect                             | PolicyCage Response                                                    |
| -------------------------------------------- | ---------------------------------------------------------------------- |
| Risk multiplier increases to 3.0x (Thanatos) | Capped at `maxPositionBps` regardless                                  |
| Terminal Golem attempts aggressive trade     | Rejected if it exceeds any PolicyCage constraint                       |
| Daimon engine produces extreme arousal       | No effect on PolicyCage; emotional state cannot modify on-chain limits |
| Stochastic death imminent                    | No effect; the Golem cannot "escape" death by making risky trades      |
| Golem attempts to modify PolicyCage          | Impossible; the contract requires owner signature                      |

This is the architectural guarantee that mortality-driven behavior remains safe. The Golem may _want_ to take larger risks as it approaches death (Thanatos drive), but the PolicyCage prevents it from _actually_ doing so beyond defined limits. Mortality provides motivation; PolicyCage provides constraint. The tension between them is productive, not pathological.

---

## Integration Event Flow for Each Mortality Clock Trigger

> **Extended:** Detailed 12-step sequences for economic, epistemic, and stochastic death triggers, plus affect-modulated response table -- see [../../prd2-extended/02-mortality/12-integration-extended.md](../../prd2-extended/02-mortality/12-integration-extended.md)

All three death triggers follow the same terminal pattern: detect threshold -> phase transition to terminal -> Death Protocol (Settle -> Reflect -> Legacy) -> Clade notification -> dashboard update -> telemetry. Economic death fires when `remainingUsdc < apoptoticReserve + $0.50`. Epistemic death fires when fitness stays below `senescenceThreshold` (0.35) for the full grace period (500 ticks). Stochastic death fires on a per-tick hazard roll modulated by age and epistemic fitness.

---

## MortalityEvent Enum

> **Extended:** Full MortalityEvent enum, 15 MortalityEventType variants, 10 typed payload structs (PhaseTransitionPayload, EpistemicWarningPayload, StochasticCheckPayload, DeathTriggeredPayload, DeathCompletedPayload, SiblingDeathPayload, DemurrageCyclePayload, AffectStateChangePayload, ImmortalWouldHaveDiedPayload, MigrationStepPayload) -- see [../../prd2-extended/02-mortality/12-integration-extended.md](../../prd2-extended/02-mortality/12-integration-extended.md)

The canonical event type emitted by the mortality engine. Each `MortalityEvent` carries a discriminated `type` field, the golem ID, tick, timestamp, vitality state, phase, and a typed payload. 15 event types cover the full mortality lifecycle from phase transitions through death completion.

---

## Dream Integration

Track 4 Dreaming integrates with the mortality system through three mechanisms: (1) dream budget allocation tracks behavioral phase -- Thriving Golems dream freely while Terminal Golems allocate all remaining resources to the Thanatopsis Protocol, (2) mortality pressure biases dream content toward defensive/consolidation modes as vitality declines, and (3) the DreamJournal enriches death testaments and succession by providing untested hypotheses and validated dream discoveries to successors. See `../05-dreams/06-integration.md` for the complete four-track interaction model.

---

## References

- [ARGYRIS-1978] Argyris, C. & Schon, D. (1978). _Organizational Learning_. Addison-Wesley.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [DAVIS-ZHONG-2017] Davis, R.L. & Zhong, Y. "The Biology of Forgetting." _Neuron_ 95(5), 2017.
- [GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.
- [KAHNEMAN-2011] Kahneman, D. _Thinking, Fast and Slow_. Farrar, Straus and Giroux, 2011.
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." _J. Monetary Economics_ 50(3), 2003.

## Mortality-Specific Attack Vectors, FSM Hooks, Runtime Execution, and Observability

> **Extended:** 5 mortality-specific attack vectors with defenses (death acceleration, death prevention, inheritance poisoning, Hayflick manipulation, reserve theft), 3 Heartbeat FSM hook points (SENSING/DECIDING/SLEEPING), 8-step runtime Death Protocol execution sequence with constraints table, Prometheus metrics (6 gauges), SSE events, owner alert matrix, error codes, disaster recovery -- see [../../prd2-extended/02-mortality/12-integration-extended.md](../../prd2-extended/02-mortality/12-integration-extended.md)

Key architectural invariant: `bardo-lifespan` triggers at the infrastructure layer, not the LLM layer. The Golem's LLM cannot prevent or delay death. The Death Protocol has a 30-minute max duration, uses only the Apoptotic Reserve budget, and after FSM enters DYING no Steer or FollowUp commands are accepted. Insurance snapshots persist to R2 every 6 hours, bounding worst-case knowledge loss for stochastic deaths.

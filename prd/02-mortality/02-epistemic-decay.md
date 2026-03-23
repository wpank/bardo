# Epistemic Decay: Primary Informational Death [SPEC]

> **Version**: 4.0 | **Status**: Draft
>
> **Crate**: `golem-mortality`
>
> **Depends on**: `01-architecture.md` (triple-clock system, vitality computation, fractal structure), `../01-golem/02-heartbeat.md` (tick-based FSM driving all Golem computation), `../04-memory/01-grimoire.md` (persistent knowledge base specification)

---

> **Reader orientation:** This document specifies the epistemic clock, one of three independent mortality pressures on every Bardo Golem (mortal autonomous DeFi agent). Epistemic decay measures how well the Golem's internal world-model matches observed market reality; when the model drifts too far, the Golem dies. If you are new to the three-clock mortality model, read `01-architecture.md` (triple-clock system, vitality computation, fractal structure) first. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

This document specifies the epistemic decay system -- the primary informational death mechanism for Bardo Golems. Epistemic decay replaces the original Hayflick counter with a continuous, environment-coupled fitness measure that kills Golems when their model of the world has drifted far enough from reality that a fresh successor would outperform them.

Epistemic decay is the most important of the three mortality clocks because it is the one that makes death productive rather than merely necessary. Economic death is a resource constraint. Stochastic death is a game-theoretic mechanism. Epistemic death is the mechanism that ensures the ecosystem's collective intelligence improves over time.

---

## Why Epistemic Decay Replaces the Hayflick Counter

### The Problem with the Hayflick Counter

The original Hayflick Counter imposed a hard lifespan ceiling: `tickCount >= maxTicks` (default 100,000 ticks, approximately 46 days at 1 tick per 40 seconds). It was an engineering constraint dressed in biological metaphor -- a timeout, not a life.

The Hayflick Counter fails on three dimensions:

1. **No environmental coupling.** A Golem operating in a stable market regime with valid strategies is killed at exactly the same age as one flailing in unprecedented volatility. The death is not earned. It is scheduled. Biological organisms do not die at a fixed age -- Hayflick limits vary enormously across species because lifespan co-evolved with ecological niche. A Galápagos tortoise lives 175 years; a mayfly lives 24 hours. The difference is environmental, not genetic.

2. **No information content.** The tick counter carries zero information about the Golem's actual fitness. It does not distinguish between a Golem that has adapted successfully to three regime changes and one that has been wrong since tick 1,000. Both die at tick 100,000.

3. **Perverse incentives.** A Golem that knows its exact death date can compute exactly when to stop investing in learning and start optimizing for death preparation. This creates a backward-induction problem where the Golem's final N ticks are spent preparing for death rather than operating, wasting the most experienced period of its life.

### What Epistemic Decay Gets Right

Epistemic decay replaces scheduled death with **emergent death**. The Golem dies when its model of the world has drifted far enough from reality that a fresh successor would outperform it. Death is co-determined by the Golem's internal state and its environment -- exactly as biological organisms die when accumulated damage exceeds repair capacity in their specific ecological niche.

This means:

- **Lifespan is emergent, not configured.** There is no fixed maximum age.
- **Volatile environments select for shorter-lived agents** with faster knowledge transfer.
- **Stable environments allow longer-lived agents** with deeper exploitation.
- **The protocol self-adjusts** to market conditions without parameter changes.

**Old framing** (Hayflick): "Your Golem can run for approximately 46 days before informational death."

**New framing** (Epistemic Decay): "Your Golem lives as long as the world stays recognizable. In calm markets, it might run for months. In turbulence, it might last weeks. Its lifespan is a measure of how fast DeFi is changing -- and when it dies, it proves the market moved on."

---

## The Evidence

Six independent research domains converge on the same conclusion: knowledge degrades, models go stale, and periodic replacement outperforms continuous patching.

### 91% of ML Models Degrade Temporally

Vela et al. (2022, _Scientific Reports_) conducted the first systematic analysis of AI aging across 32 datasets spanning healthcare, transportation, finance, and weather. The finding: **91% of machine learning models showed temporal quality degradation**. Even models that achieved high accuracy at deployment did not remain static -- their performance eroded as the world shifted beneath their training data. Models that were "systematically biased" were not merely slightly wrong; they were "actively worse than random, because they confidently encode outdated patterns" [VELA-2022].

This is not a theoretical concern. It is the single strongest empirical claim supporting the mortality thesis: the overwhelming majority of models go stale, and a DeFi agent's model is built on knowledge with a measurable expiration date.

### Knowledge Has a Measurable Half-Life

Arbesman's research on the decay of facts provides domain-specific quantitative rates. Medical knowledge has a half-life of approximately 45 years. Physics knowledge approximately 13 years. IT and technological knowledge decays by half in fewer than 2 years. An AI agent operating in DeFi -- one of the most rapidly evolving technological domains -- faces knowledge half-lives measured in weeks to months, not years [ARBESMAN-2012].

### Concept Drift Formalizes Decay

Zliobaitė et al. (2014) and Lu et al. (2020) classify concept drift into four patterns: sudden, gradual, incremental, and recurring. In financial markets, **all four patterns co-occur simultaneously**:

| Drift Type      | DeFi Manifestation                                                     | Timescale        |
| --------------- | ---------------------------------------------------------------------- | ---------------- |
| **Sudden**      | Flash crashes, protocol exploits, governance attacks                   | Seconds to hours |
| **Gradual**     | Regime transitions (bull to bear), TVL migration                       | Days to weeks    |
| **Incremental** | Liquidity depth changes, gas market evolution                          | Weeks to months  |
| **Recurring**   | Seasonal patterns (quarterly expiry, tax selling), funding rate cycles | Months           |

A Golem's model must handle all four simultaneously. No single adaptation strategy works for all drift types, which is why continuous adaptation eventually fails and replacement becomes necessary [ZLIOBAITĖ-2014].

### Expertise Creates Entrenchment

Dane (2010, _Academy of Management Review_) demonstrated that expertise produces "cognitive entrenchment" -- highly stable domain schemas that reduce flexibility. Experts construct constraining narratives about how domains "should" work, making them systematically blind to structural changes. The more expertise a Golem accumulates, the harder it becomes for it to recognize when its expertise no longer applies [DANE-2010].

This is particularly dangerous for LLM-based agents whose "expertise" is encoded in PLAYBOOK.md (the Golem's active strategy heuristics file) and Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) entries. These artifacts become the lens through which the Golem perceives the market. When the market changes structurally, the lens distorts rather than clarifies. The Golem sees what its heuristics prepare it to see, not what is actually happening.

### Retraining from Scratch Outperforms Continuous Adaptation

Van de Ven et al. (2024) confirmed the industry reality: "practitioners tend to periodically retrain the entire network from scratch on all data, despite the large computational costs." This is not laziness -- it is empirical wisdom. Death and rebirth consistently outperforms immortal adaptation in production ML systems because:

1. **Accumulated technical debt** compounds silently (Sculley et al., 2015)
2. **Catastrophic forgetting** means new learning destroys old learning
3. **Distribution shift** means the old training distribution is no longer representative
4. **Stale features** become entangled with valid features in ways that are impossible to disentangle

A fresh successor initialized with compressed predecessor knowledge at reduced confidence (0.3) but free to build new models from current market data will typically outperform the predecessor within 500-1000 ticks [VAN-DE-VEN-2024].

### Optimal Reset Interval Scales with Volatility

Besbes, Gur, and Zeevi (2019, _Stochastic Systems_) proved mathematically that in non-stationary bandit problems, the optimal batch length scales as:

```
Delta_T proportional to (T / K * V)^(2/3)
```

where `V` is the temporal variation budget -- a measure of environmental volatility. More volatile environments demand shorter agent lifespans. Less volatile environments permit longer ones. The optimal strategy is not to adapt continuously but to **reset periodically**, with the reset interval calibrated to environmental dynamics [BESBES-2019].

This is precisely what epistemic decay implements: the Golem's lifespan is an emergent function of environmental volatility, without the owner needing to configure a specific tick count.

---

## Epistemic Fitness Score

The epistemic fitness score is a continuous measure of how well the Golem's internal models match observed reality. It replaces the binary alive/dead Hayflick threshold with a gradient that drives behavioral phase transitions through the composite vitality score (see `01-architecture.md`).

### Fitness Metric: R-squared Over Predictions

The epistemic fitness score is measured as R-squared (coefficient of determination) over the Golem's recent predictions vs. actual outcomes. R-squared = 1.0 means perfect prediction; R-squared = 0.0 means no better than guessing the mean; R-squared < 0 means worse than the mean. The rolling window is 100 predictions (configurable via `fitness_window`). Below 10 predictions, the score defaults to 0.5 (insufficient data to judge).

This metric is grounded in on-chain reality: if the Golem predicts a swap will return X tokens and the `OutcomeVerification` record shows it returned Y tokens, the deviation is a deterministic fact, not an LLM judgment. The R-squared metric over many such comparisons produces a continuous fitness signal that degrades naturally as market conditions shift away from the Golem's learned patterns.

### EpistemicFitnessState Struct

```rust
/// Complete epistemic fitness state, maintained by the lifespan extension.
/// Updated every tick based on prediction-outcome comparison.
///
/// Crate: `golem-mortality`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicFitnessState {
    /// Rolling window of prediction-outcome pairs.
    /// The window size is configurable (default: 2000 ticks, ~23 hours).
    /// Older entries are evicted as new ones arrive.
    pub prediction_log: VecDeque<PredictionOutcomePair>,

    /// Rolling fitness score computed via EMA.
    /// Range: [0.0, 1.0]
    /// - 0.0 = predictions have no correlation with outcomes (random)
    /// - 1.0 = all predictions match observed outcomes (perfect)
    ///
    /// This is the primary input to the epistemic component of composite vitality.
    pub fitness: f64,

    /// Fitness trend: first derivative of the fitness score.
    /// Computed as the difference between the current EMA and the EMA from
    /// 100 ticks ago. Positive = improving, negative = declining.
    ///
    /// Used for early warning telemetry and senescence prediction.
    pub fitness_trend: f64,

    /// Per-domain fitness breakdown.
    /// Each domain tracks its own EMA fitness score with domain-specific
    /// alpha (EMA smoothing factor) calibrated to the domain's half-life.
    pub domain_fitness: DomainFitness,

    /// Ticks since fitness last exceeded the senescence threshold.
    /// Resets to 0 when fitness recovers above threshold.
    /// When this exceeds recovery_grace_period, senescence is confirmed.
    pub ticks_below_threshold: u64,

    /// Whether the Golem has entered confirmed senescence (Stage 2+).
    /// Once true, recovery requires exceeding the recovery hysteresis threshold
    /// (default: senescence_threshold + 0.10 = 0.45).
    pub senescent: bool,

    /// Current senescence stage.
    /// None = not in senescence
    /// Stage1 = WARNING (grace period, recovery possible)
    /// Stage2 = CONFIRMED (vitality dropping, recovery rare but possible)
    /// Stage3 = DEATH_PROTOCOL (epistemic death imminent)
    pub senescence_stage: Option<SenescenceStage>,

    /// Peak fitness score achieved during this Golem's lifetime.
    /// Used in death metadata to help successors understand the trajectory.
    pub peak_fitness: f64,

    /// Tick number at which peak fitness was achieved.
    pub peak_fitness_tick: u64,

    /// Tick number of the last prediction that was correct on all dimensions.
    pub last_fully_correct_prediction: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SenescenceStage {
    Stage1 = 1,
    Stage2 = 2,
    Stage3 = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainFitness {
    pub price_direction: f64,
    pub volatility_regime: f64,
    pub yield_trend: f64,
    pub gas_pattern: f64,
    pub protocol_behavior: f64,
}

/// A single prediction-outcome pair from one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionOutcomePair {
    /// Tick number when the prediction was made.
    pub tick_number: u64,
    /// The Golem's prediction for this tick's market state.
    pub prediction: MarketPrediction,
    /// The observed outcome after the tick completed.
    pub outcome: MarketOutcome,
    /// Per-dimension accuracy score [0.0, 1.0].
    pub accuracy: f64,
    /// Unix timestamp of the prediction.
    pub timestamp: u64,
}

/// The Golem's prediction for the next tick's market state.
/// Generated by the probe system during the SENSING phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrediction {
    pub price_direction: PriceDirection,
    pub volatility_regime: VolatilityRegime,
    pub yield_trend: YieldTrend,
    pub gas_condition: GasCondition,
    pub protocol_state: ProtocolState,
    /// Golem's self-assessed confidence in this prediction.
    /// Used for calibration tracking (overconfident Golems are penalized).
    pub confidence_score: f64,
}

/// Observed market outcome after a tick completes.
/// Collected from on-chain data and price feeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOutcome {
    pub price_direction: PriceDirection,
    pub volatility_regime: VolatilityRegime,
    pub yield_trend: YieldTrend,
    pub gas_condition: GasCondition,
    pub protocol_state: ProtocolState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceDirection { Up, Down, Flat }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolatilityRegime { Low, Medium, High }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YieldTrend { Increasing, Stable, Decreasing }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasCondition { Cheap, Normal, Expensive }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState { Normal, Degraded, Upgraded }
```

### Per-Tick Accuracy Computation

Accuracy is computed as a weighted average across prediction dimensions. The weights reflect the relative importance of each dimension for DeFi agent performance.

```rust
/// Weights for each prediction dimension in the accuracy computation.
/// Must sum to 1.0.
///
/// Price direction is weighted highest because it has the most direct
/// impact on trading outcomes. Protocol behavior is weighted lowest
/// because it changes rarely but has high impact when it does change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionWeights {
    pub price_direction: f64,
    pub volatility_regime: f64,
    pub yield_trend: f64,
    pub gas_condition: f64,
    pub protocol_state: f64,
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            price_direction: 0.35,
            volatility_regime: 0.25,
            yield_trend: 0.20,
            gas_condition: 0.10,
            protocol_state: 0.10,
        }
    }
}

/// Compute per-tick accuracy as a weighted match score across all
/// prediction dimensions.
///
/// Each dimension contributes a binary match (1.0 if correct, 0.0 if wrong)
/// weighted by its importance. The result is a single [0.0, 1.0] score
/// representing how well the Golem's prediction matched reality this tick.
pub fn compute_tick_accuracy(
    prediction: &MarketPrediction,
    outcome: &MarketOutcome,
    weights: &DimensionWeights,
) -> f64 {
    let price_match   = if prediction.price_direction == outcome.price_direction { 1.0 } else { 0.0 };
    let vol_match     = if prediction.volatility_regime == outcome.volatility_regime { 1.0 } else { 0.0 };
    let yield_match   = if prediction.yield_trend == outcome.yield_trend { 1.0 } else { 0.0 };
    let gas_match     = if prediction.gas_condition == outcome.gas_condition { 1.0 } else { 0.0 };
    let proto_match   = if prediction.protocol_state == outcome.protocol_state { 1.0 } else { 0.0 };

    weights.price_direction * price_match
        + weights.volatility_regime * vol_match
        + weights.yield_trend * yield_match
        + weights.gas_condition * gas_match
        + weights.protocol_state * proto_match
}

/// Compute per-domain accuracy for the domain fitness breakdown.
/// Returns individual match scores (not weighted) for each dimension.
pub fn compute_domain_accuracies(
    prediction: &MarketPrediction,
    outcome: &MarketOutcome,
) -> DomainFitness {
    DomainFitness {
        price_direction:   if prediction.price_direction == outcome.price_direction { 1.0 } else { 0.0 },
        volatility_regime: if prediction.volatility_regime == outcome.volatility_regime { 1.0 } else { 0.0 },
        yield_trend:       if prediction.yield_trend == outcome.yield_trend { 1.0 } else { 0.0 },
        gas_pattern:       if prediction.gas_condition == outcome.gas_condition { 1.0 } else { 0.0 },
        protocol_behavior: if prediction.protocol_state == outcome.protocol_state { 1.0 } else { 0.0 },
    }
}
```

### Rolling Fitness via EMA

The fitness score uses an exponential moving average (EMA) over the configured window. The EMA gives more weight to recent predictions while maintaining memory of historical accuracy, producing a smooth signal that responds to regime changes without overreacting to single-tick noise.

```rust
/// Update the rolling fitness score using exponential moving average.
///
/// The EMA formula: fitness_new = alpha * accuracy + (1 - alpha) * fitness_old
///
/// The alpha parameter controls responsiveness:
/// - Lower alpha = more memory, slower response to changes
/// - Higher alpha = less memory, faster response but noisier
///
/// Default alpha = 0.01 means approximately 100 ticks (~67 minutes) to converge
/// to a new accuracy level. This is fast enough to detect genuine regime shifts
/// but slow enough to avoid triggering senescence on single-tick noise.
///
/// The effective window of the EMA (ticks until a measurement's influence
/// drops to 1/e) is approximately 1/alpha = 100 ticks.
fn update_fitness(current_fitness: f64, new_accuracy: f64, alpha: f64) -> f64 {
    alpha * new_accuracy + (1.0 - alpha) * current_fitness
}

/// Domain-specific EMA alpha values.
/// Owners can override defaults to match their specific market environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAlphaConfig {
    pub price_direction: f64,
    pub volatility_regime: f64,
    pub yield_trend: f64,
    pub gas_pattern: f64,
    pub protocol_behavior: f64,
}

impl Default for DomainAlphaConfig {
    fn default() -> Self {
        Self {
            price_direction: 0.02,
            volatility_regime: 0.005,
            yield_trend: 0.003,
            gas_pattern: 0.05,
            protocol_behavior: 0.001,
        }
    }
}

/// Update domain-specific fitness scores with domain-appropriate alpha values.
/// Each domain has its own EMA rate calibrated to the domain's knowledge half-life.
pub fn update_domain_fitness(
    domain_fitness: &DomainFitness,
    domain_accuracies: &DomainFitness,
    domain_alphas: &DomainAlphaConfig,
) -> DomainFitness {
    DomainFitness {
        price_direction: update_fitness(
            domain_fitness.price_direction,
            domain_accuracies.price_direction,
            domain_alphas.price_direction,
        ),
        volatility_regime: update_fitness(
            domain_fitness.volatility_regime,
            domain_accuracies.volatility_regime,
            domain_alphas.volatility_regime,
        ),
        yield_trend: update_fitness(
            domain_fitness.yield_trend,
            domain_accuracies.yield_trend,
            domain_alphas.yield_trend,
        ),
        gas_pattern: update_fitness(
            domain_fitness.gas_pattern,
            domain_accuracies.gas_pattern,
            domain_alphas.gas_pattern,
        ),
        protocol_behavior: update_fitness(
            domain_fitness.protocol_behavior,
            domain_accuracies.protocol_behavior,
            domain_alphas.protocol_behavior,
        ),
    }
}

/// Compute the composite epistemic fitness from domain-specific scores.
/// Uses the same dimension weights as per-tick accuracy computation.
pub fn compute_composite_fitness(
    domain_fitness: &DomainFitness,
    weights: &DimensionWeights,
) -> f64 {
    weights.price_direction * domain_fitness.price_direction
        + weights.volatility_regime * domain_fitness.volatility_regime
        + weights.yield_trend * domain_fitness.yield_trend
        + weights.gas_condition * domain_fitness.gas_pattern
        + weights.protocol_state * domain_fitness.protocol_behavior
}
```

---

## P&L Proxy Mode (Oracle Disabled)

When `[oracle] enabled = false`, the epistemic fitness system substitutes `TradeOutcomeFitnessState` for `EpistemicFitnessState`:

```rust
pub struct TradeOutcomeFitnessState {
    /// Rolling window of trade outcomes (net P&L after fees and gas).
    pub trade_log: VecDeque<TradeOutcome>,  // bounded by pnl_fitness_window
    /// Rolling win rate. profitable_trades / total_trades.
    pub fitness: f64,
    pub fitness_trend: f64,
    /// True when trade_log.len() < pnl_minimum_trades.
    pub insufficient_data: bool,
}

pub struct TradeOutcome {
    pub tick: u64,
    pub net_pnl_usdc: f64,   // From OutcomeVerification record (already exists)
    pub profitable: bool,
}
```

**Fitness formula:** `fitness = profitable_trades / total_trades` over the rolling `pnl_fitness_window`. Below `pnl_minimum_trades` (default 10), fitness defaults to 0.5 — the same as the oracle's under-10-prediction default. The same `senescence_threshold`, `recovery_grace_period`, and `fitness_trend` logic applies unchanged.

Trade outcome data comes from `OutcomeVerification` records produced in heartbeat Step 8 (VERIFY). These already exist in the pipeline. No new chain reads are needed.

The `[mortality.epistemic]` fields that control proxy mode:

```toml
[mortality.epistemic]
# P&L proxy fields (used when [oracle] enabled = false):
pnl_minimum_trades = 10            # Min executed trades before fitness is valid
pnl_fitness_window = 200           # Rolling window in trade count (not ticks)
```

**Tradeoff.** P&L-based fitness is noisier than prediction-based fitness — a single large loss can dominate the window. The window is counted in trades, not ticks, so a low-frequency strategy still gets N data points regardless of heartbeat interval.

---

## Domain-Specific Decay Rates

Not all knowledge decays at the same rate. Gas price patterns change hourly. Governance mechanisms change monthly. The system tracks per-domain fitness with domain-appropriate decay rates, following Arbesman's half-life framework.

### Decay Rate Table

| Domain                | Typical Half-Life | EMA Alpha | Effective Window        | Rationale                                                                                                                                                                                                                                                                          |
| --------------------- | ----------------- | --------- | ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Gas/MEV patterns**  | Hours             | 0.05      | ~20 ticks (~13 min)     | Extremely volatile. Gas markets shift with each block. MEV strategies become unprofitable within hours as searchers adapt. Mempool dynamics change with network congestion cycles. A Golem that learned gas patterns yesterday is already stale.                                   |
| **Price direction**   | Days              | 0.02      | ~50 ticks (~33 min)     | Regime-dependent with moderate stability. Trends persist for days to weeks, but regime shifts (bull to bear, range-bound to trending) can be sudden. The Golem's price direction model must track both the current regime and detect regime transitions.                           |
| **Volatility regime** | Weeks             | 0.005     | ~200 ticks (~2.2 hrs)   | Structural shifts in volatility are less frequent but more consequential. Realized volatility clusters (GARCH effects) create persistent regimes. The Golem needs a longer memory to distinguish genuine regime shifts from temporary spikes.                                      |
| **Yield trends**      | Weeks-Months      | 0.003     | ~333 ticks (~3.7 hrs)   | Protocol yield mechanics change slowly. APR compression from capital inflows, incentive program changes, and base rate shifts occur on weekly to monthly timescales. But when they shift, they shift structurally.                                                                 |
| **Protocol behavior** | Months            | 0.001     | ~1000 ticks (~11.1 hrs) | Smart contract logic is stable until upgrades. Protocol upgrades, governance changes, and architectural shifts are infrequent but dramatic. The Golem needs deep memory here because the signal-to-noise ratio is very low (many ticks of "normal" between rare "changed" events). |

### Why These Specific Alphas

The alpha values are calibrated so that a Golem that was 100% accurate in a domain and then becomes 0% accurate (a complete regime shift) will see its domain fitness drop to 50% of its pre-shift value in approximately one domain half-life:

```
Ticks to 50% = -ln(0.5) / alpha approximately 0.693 / alpha

Gas/MEV:   0.693 / 0.05  = ~14 ticks   (~9 min, approximately hourly half-life)
Price:     0.693 / 0.02  = ~35 ticks   (~23 min, approximately daily half-life)
Volatility: 0.693 / 0.005 = ~139 ticks  (~1.5 hrs, approximately weekly half-life)
Yield:     0.693 / 0.003 = ~231 ticks  (~2.6 hrs, approximately bi-weekly half-life)
Protocol:  0.693 / 0.001 = ~693 ticks  (~7.7 hrs, approximately monthly half-life)
```

These translate to real-world half-lives because tick intervals are approximately 40 seconds, and the EMA window represents the time for a complete accuracy reversal to propagate to the fitness score. In practice, regime shifts are rarely instantaneous, so the actual perceived half-life is longer than the mathematical minimum.

### Configuration

```rust
// DomainAlphaConfig and its defaults are defined alongside update_domain_fitness
// above. The Default impl provides:
//   price_direction: 0.02, volatility_regime: 0.005, yield_trend: 0.003,
//   gas_pattern: 0.05, protocol_behavior: 0.001
// Owners can override these to match their specific market environment.
```

---

## Regime-Tagged Fitness Measurement

Epistemic fitness measurement is regime-tagged (see `RegimeTag` in `01-architecture.md`). Fitness should be compared within matching regime conditions to distinguish genuine epistemic decay from regime mismatch.

**Why this matters**: A Golem optimized for range-bound markets will show declining prediction accuracy during a trending market — but this is regime mismatch, not cognitive decay. Without regime tagging, the mortality system would interpret this as senescence and accelerate death. With regime tagging, the system compares the Golem's current performance against its own historical performance _in the same regime conditions_.

**Regime-matched comparison**: Fitness windows (configurable, default 50 ticks) are filtered to include only ticks with matching `volatilityQuintile` and `trendDirection`. If fewer than 10 matched pairs are available, the window extends until sufficient matches accumulate. This prevents premature mortality triggers during regime transitions.

**Death trigger exemption**: If the current regime has fewer than 20 historical matched ticks, the epistemic clock enters a "learning" grace period — fitness degradation is logged but does not contribute to mortality pressure. The Golem gets time to adapt to genuinely novel conditions.

---

## Senescence Cascade

When epistemic fitness drops below the senescence threshold (default: 0.35), the Golem enters a **senescence cascade** -- a structured decline that provides opportunities for recovery but ultimately converges on death if recovery fails. The cascade has three stages with defined transitions, recovery conditions, and behavioral modifications.

### Stage Diagram

```
Normal Operation (fitness > 0.35)
  |
  | fitness drops below 0.35
  v
Stage 1: WARNING (grace period begins)
  |  Golem is aware of declining fitness
  |  Loop 2 (strategic reflection) triggered immediately
  |  PLAYBOOK.md audit: which heuristics are failing?
  |  Phage spawn rate increases to maximum (testing hypotheses faster)
  |  Duration: recoveryGracePeriod ticks (default: 500, ~5.8 hours)
  |
  | If fitness recovers above 0.35: return to Normal
  | If fitness stays below 0.35 for recoveryGracePeriod:
  v
Stage 2: SENESCENCE CONFIRMED
  |  Vitality score drops sharply (epistemic component -> near-zero via sigmoid)
  |  Phase transitions accelerate (Conservation -> Declining -> Terminal)
  |  Knowledge distillation prioritized (Styx (global knowledge relay and persistence layer) Archive uploads, Clade (sibling Golems sharing a common ancestor) sharing)
  |  Exploration halted -- Golem focuses on documenting what it knows
  |  Clade sharing threshold drops to 0.2
  |
  | If fitness recovers above 0.45 (hysteresis): return to Normal
  | If vitality composite drops below 0.1:
  v
Stage 3: DEATH PROTOCOL INITIATES
  |  Death cause: "epistemic_senescence"
  |  Full Thanatopsis Protocol
  |  Death testament includes epistemic metadata
```

### Stage Implementation

```rust
/// Update senescence state based on current epistemic fitness.
/// Called every tick by the lifespan extension.
pub fn update_senescence_state(
    state: &EpistemicFitnessState,
    config: &EpistemicMortalityConfig,
) -> EpistemicFitnessState {
    let mut updated = state.clone();

    if state.fitness < config.senescence_threshold {
        // Fitness is below threshold
        updated.ticks_below_threshold = state.ticks_below_threshold + 1;

        if !state.senescent {
            // Not yet confirmed senescent
            match state.senescence_stage {
                None => {
                    // Just entered Stage 1
                    updated.senescence_stage = Some(SenescenceStage::Stage1);
                }
                Some(SenescenceStage::Stage1)
                    if state.ticks_below_threshold >= config.recovery_grace_period =>
                {
                    // Grace period expired -> Stage 2
                    updated.senescent = true;
                    updated.senescence_stage = Some(SenescenceStage::Stage2);
                }
                _ => {}
            }
        }
        // Stage 2 -> Stage 3 transition is handled by the vitality check
        // in the lifespan extension (when composite < 0.1)
    } else {
        // Fitness is above threshold
        match state.senescence_stage {
            Some(SenescenceStage::Stage1) => {
                // Stage 1 recovery: just need to be above threshold
                updated.ticks_below_threshold = 0;
                updated.senescence_stage = None;
            }
            Some(SenescenceStage::Stage2) if state.senescent => {
                // Stage 2 recovery: need hysteresis (threshold + 0.10)
                let recovery_threshold = config.senescence_threshold + 0.1; // Default: 0.45
                if state.fitness > recovery_threshold {
                    updated.senescent = false;
                    updated.senescence_stage = None;
                    updated.ticks_below_threshold = 0;
                }
            }
            _ => {
                // Normal operation, not in senescence
                updated.ticks_below_threshold = 0;
            }
        }
    }

    updated
}
```

### Recovery Is Possible But Rare

The grace period exists because epistemic fitness can temporarily dip during genuine regime transitions. A Golem that was perfectly calibrated for a bull market will see its fitness drop sharply when the regime shifts to bear -- but if it successfully adapts (via Loop 2 strategic reflection and PLAYBOOK.md evolution), its fitness recovers.

The **hysteresis** in Stage 2 (recovery requires exceeding 0.45, not just 0.35) prevents oscillation at the boundary. A Golem must demonstrate genuine recovery, not marginal survival. This is biologically analogous to the immune system's hysteresis: infection must be fully cleared before the immune response stands down, not merely reduced to the initial detection threshold.

**Empirical expectation**: Most Golems that enter Stage 2 will not recover. The Loop 2 reflection that could save them requires inference budget (Opus-level calls), which the Golem may be conserving due to economic pressure or phase restrictions. This creates a cruel but productive feedback loop: the austerity measures that extend economic life accelerate epistemic death.

March (1991) predicted exactly this dynamic: "adaptive processes refine exploitation more rapidly than exploration, becoming effective short-term but self-destructive long-term." A Golem in Conservation mode optimizes for survival by cutting inference costs, but cutting inference costs means it learns less, validates less, and watches its Grimoire erode -- hastening the very epistemic death it is trying to avoid.

---

## Why Retraining Does Not Work for LLM-Based Agents

A natural objection: if the Golem's model is stale, why not retrain it? The answer is that an LLM-based agent cannot retrain itself. It is not a trainable neural network. Its "model" is the combination of:

1. **PLAYBOOK.md heuristics**: Accumulated rules of thumb, validated patterns, and strategic guidelines
2. **Grimoire entries**: Episodes, insights, causal links, and heuristics stored in long-term memory
3. **Context injection**: The information the heartbeat FSM injects into each LLM call

When these go stale, the remediation options are limited:

- **Loop 2 reflection** (expensive): The Golem spends Opus-level inference to interrogate its own assumptions. This can update PLAYBOOK.md heuristics and promote/demote Grimoire entries. But Loop 2 is constrained by the same stale Grimoire entries that caused the decay. The Golem is trying to update its world model using a world model that is itself outdated. This is Dane's cognitive entrenchment applied computationally -- the very expertise that made the Golem effective becomes the constraint that prevents adaptation.

- **Loop 3 meta-consolidation** (extremely expensive, HARDENED-only): The Golem examines its learning process itself. This can modify meta-heuristics and learning strategies. But at this level of abstraction, the interventions are slow to take effect and may require hundreds of ticks to validate.

- **Fresh context from market data**: New on-chain data enters every tick. But interpreting that data requires the Golem's existing model. If the model is stale, new data is interpreted through stale lenses, producing stale conclusions.

The deeper problem is structural: **each Loop 2 reflection cycle faces diminishing returns**. The first reflection after a regime shift is highly productive -- the Golem identifies obvious stale heuristics and updates them. The second reflection produces smaller gains. By the fifth, the Golem is making marginal adjustments to a fundamentally misaligned model. The returns decay exponentially while the cost remains constant.

A fresh successor Golem, initialized with the predecessor's compressed knowledge at 0.3 confidence but free to build new models from current market data, will typically outperform within 500-1000 ticks. This is the Van de Ven finding applied to agent systems: periodic replacement outperforms continuous patching.

---

## Environmental Coupling: Lifespan as Emergent Property

The most philosophically important property of epistemic decay is that **the Golem's lifespan is co-determined by its environment**. This is not a parameter to configure but an emergent property of the interaction between the Golem's internal model and the external world.

### How Environment Determines Lifespan

A Golem in a **stable, low-volatility market** will maintain high epistemic fitness for weeks or months. Its predictions remain accurate because the world has not changed much. The EMA fitness score stays high because the prediction-outcome matches are consistent. The senescence threshold is never approached. The Golem could theoretically live indefinitely on epistemic terms alone (it would eventually die of stochastic mortality or, for non-self-hosted Golems, economic depletion).

A Golem in a **volatile, rapidly shifting market** will see its fitness decay within days. Regime changes invalidate its predictions. Gas patterns shift. Volatility regimes are unstable. Protocol upgrades change yield dynamics. The EMA fitness score declines steadily as mismatches accumulate. Senescence is reached quickly.

A Golem in a **structurally transformed market** (e.g., a fundamental protocol change, a new DeFi primitive, a regulatory event) will see its fitness collapse within hours to days. All prediction dimensions fail simultaneously. The EMA cannot recover because the underlying model is completely misaligned.

### Disposable Soma Prediction (Kirkwood 1977)

This environmental coupling is precisely what disposable soma theory predicts. Kirkwood (1977) showed that organisms in high-extrinsic-mortality environments evolve shorter lifespans and invest more in reproduction (knowledge transfer) than self-repair (individual learning). The logic: if you are likely to die of external causes soon, investing in long-term self-maintenance is wasteful. Better to invest in producing offspring that inherit your adapted traits.

Applied to Golems:

- **High-volatility environments** select for shorter-lived Golems that invest heavily in Clade knowledge sharing and death testaments (reproduction/knowledge transfer) rather than Loop 2/Loop 3 self-repair (individual adaptation).
- **Low-volatility environments** allow longer-lived Golems that invest more in deepening their own expertise (self-repair) because the payoff horizon for that investment is longer.
- **The protocol self-adjusts** without any owner intervention. The same mortality configuration produces different lifespans in different environments because the epistemic fitness score is environment-dependent.

This is the strongest version of the argument for epistemic decay over the Hayflick counter: the Golem does not die because someone set a timer. It dies because the world moved on and it could not keep up. Its lifespan is a measure of how fast DeFi is changing -- and that measurement is itself valuable information for the ecosystem.

### Dreaming as Partial Countermeasure

Track 4 Dreaming provides a partial countermeasure to epistemic decay. Offline replay reorganizes memory representations, revealing hidden regularities that the waking Curator cannot discover under time pressure. Wagner et al. (2004) showed sleep doubles hidden rule discovery rates. However, dreaming cannot fully prevent epistemic death -- it slows the decay but does not halt it, consistent with the mortality thesis that death remains inevitable. See `../05-dreams/02-replay.md`.

---

### Terror Management Theory: Mortality Salience and Knowledge Sharing

Terror Management Theory (Greenberg, Pyszczynski & Solomon, 1986) predicts that mortality salience -- being reminded of death -- intensifies existing behavioral patterns and increases in-group solidarity. For Golems, this manifests as increased knowledge-sharing behavior when mortality pressure rises.

When epistemic fitness declines, the Golem becomes increasingly aware of its own cognitive mortality. This salience effect drives two behavioral responses:

1. **Accelerated Clade sharing.** As epistemic fitness drops, the Golem lowers its Clade sharing confidence threshold (see `08-mortality-affect.md`), pushing more knowledge to siblings before it becomes unretrievable. This is rational: if the Golem's knowledge is decaying, sharing it now while it retains some validity is better than letting it decay further.

2. **Death testament quality.** Golems that experience a prolonged decline (the "lion" phase in Nietzsche's metamorphoses) produce richer death testaments than those that die suddenly. The extended awareness of approaching death creates more opportunities for reflection, turning-point identification, and honest self-assessment.

This connection between epistemic decay and knowledge-sharing behavior is one of the strongest arguments for epistemic mortality over the Hayflick counter: it creates a productive feedback loop where declining fitness drives knowledge distribution, and knowledge distribution enriches the Clade's collective intelligence.

> **Cross-ref**: `../04-memory/03-mortal-memory.md` (Terror Management Theory connection)

---

## Knowledge Demurrage

Epistemic decay has a mirror in the Grimoire itself. Grimoire entries lose confidence over time unless actively re-validated against fresh evidence. This is knowledge demurrage -- Gesell's Freigeld principle applied to information.

The decay function follows Ebbinghaus's forgetting curve (1885): memory retention follows a negative exponential decay, `retention = exp(-t / half_life)`, where `t` is ticks since last access. The testing effect (Roediger & Karpicke, 2006) provides the counterforce: retrieving an entry from memory strengthens the memory trace more effectively than re-studying. In practice, entries that are regularly retrieved and applied against real outcomes decay slowly; entries that sit untouched decay rapidly. This produces natural knowledge pruning where relevant entries survive and irrelevant entries fade. See `../04-memory/01-grimoire.md` for the full Ebbinghaus decay implementation with per-category decay rates (episodes: 0.001/tick, insights: 0.002/tick, heuristics: 0.0005/tick, warnings: 0.003/tick).

### DemurrageConfig and applyDemurrage

```rust
/// Configuration for knowledge demurrage in the Grimoire.
/// Controls how quickly un-validated knowledge loses confidence.
///
/// Crate: `golem-grimoire`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemurrageConfig {
    /// Ticks between validation checks.
    /// The Curator extension checks entries against recent episodes at this interval.
    /// Default: 250 (~2.9 hours at 1 tick/40s)
    pub validation_interval: u64,

    /// Confidence loss per missed validation interval.
    /// An entry not re-validated loses this much confidence each interval.
    /// Default: 0.03 (3% per interval)
    pub decay_per_interval: f64,

    /// Minimum confidence before automatic archiving.
    /// Entries below this threshold are removed from active PLAYBOOK.md context
    /// and moved to cold storage. They persist for death reflection but do not
    /// influence ongoing decisions.
    /// Default: 0.1
    pub archive_threshold: f64,

    /// Domain-specific decay multipliers.
    /// Volatile domains (gas, price) decay faster than stable domains (protocol).
    /// The multiplier is applied to decay_per_interval for entries in that domain.
    ///
    /// Default multipliers:
    ///   gas: 2.0 (6% per interval)
    ///   price: 1.5 (4.5% per interval)
    ///   volatility: 1.0 (3% per interval)
    ///   yield: 0.8 (2.4% per interval)
    ///   protocol: 0.5 (1.5% per interval)
    pub domain_multipliers: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryStatus {
    Active,
    Archived,
}

/// A Grimoire entry subject to demurrage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireEntry {
    pub id: String,
    pub domain: String,
    pub confidence: f64,
    pub last_validated_tick: u64,
    pub status: EntryStatus,
    pub content: String,
}

/// Apply knowledge demurrage to a Grimoire entry.
///
/// Entries lose confidence proportional to the time since their last validation,
/// scaled by domain-specific multipliers. Entries that drop below the archive
/// threshold are moved to Archived status.
pub fn apply_demurrage(
    entry: &GrimoireEntry,
    config: &DemurrageConfig,
    current_tick: u64,
) -> GrimoireEntry {
    // Already archived entries are not further decayed
    if entry.status == EntryStatus::Archived {
        return entry.clone();
    }

    let ticks_since_validation = current_tick.saturating_sub(entry.last_validated_tick);
    let intervals = ticks_since_validation / config.validation_interval;

    // No decay if within the current validation interval
    if intervals == 0 {
        return entry.clone();
    }

    // Apply domain-specific decay
    let domain_multiplier = config
        .domain_multipliers
        .get(&entry.domain)
        .copied()
        .unwrap_or(1.0);
    let total_decay = config.decay_per_interval * domain_multiplier * intervals as f64;
    let new_confidence = (entry.confidence - total_decay).max(0.0);

    // Archive if below threshold
    let new_status = if new_confidence < config.archive_threshold {
        EntryStatus::Archived
    } else {
        EntryStatus::Active
    };

    GrimoireEntry {
        confidence: new_confidence,
        status: new_status,
        ..entry.clone()
    }
}

/// Apply demurrage to all active Grimoire entries.
/// Called by the Curator extension at each validation interval.
pub fn apply_demurrage_to_all(
    entries: &[GrimoireEntry],
    config: &DemurrageConfig,
    current_tick: u64,
) -> (Vec<GrimoireEntry>, u32, f64) {
    let mut archived_count = 0u32;
    let mut total_decay = 0.0f64;

    let updated_entries: Vec<GrimoireEntry> = entries
        .iter()
        .map(|entry| {
            let updated = apply_demurrage(entry, config, current_tick);
            if updated.status == EntryStatus::Archived && entry.status == EntryStatus::Active {
                archived_count += 1;
            }
            total_decay += entry.confidence - updated.confidence;
            updated
        })
        .collect();

    (updated_entries, archived_count, total_decay)
}
```

### What Demurrage Produces

Knowledge demurrage creates five beneficial dynamics:

1. **A lean, current Grimoire.** Stale heuristics naturally fade, keeping PLAYBOOK.md context relevant. The Golem's decision-making is not polluted by outdated patterns that confidently encode expired market conditions.

2. **Mortality acceleration for inactive Golems.** A Golem in Conservation mode (monitoring only) learns less, validates less, and watches its Grimoire erode. This hastens epistemic death -- precisely the intended effect. An inactive Golem is not producing value and should be replaced by a fresh successor.

3. **Natural knowledge turnover.** Old entries make room for new ones without explicit deletion. The Golem does not need to decide what to forget; the forgetting happens automatically, and only actively validated knowledge persists.

4. **Incentive to explore.** Only fresh evidence maintains confidence, rewarding active market engagement. A Golem that retreats to monitoring mode pays a knowledge tax that grows with every validation interval. Exploration is not optional -- it is the cost of maintaining knowledge.

5. **Forced knowledge circulation.** Entries approaching the archive threshold are prime candidates for Clade sharing. The Golem's incentive is to share marginal knowledge with siblings before it depreciates entirely -- better to contribute to the collective than to let it evaporate. This implements Gesell's Freigeld principle for information: knowledge that is not actively used decays in value, forcing it into circulation before it depreciates entirely [GESELL-1916].

---

## Integration with Heartbeat Extension

The epistemic fitness computation runs as part of the `bardo-lifespan` extension, executing on every tick after the heartbeat FSM completes its SENSING-DECIDING-ACTING-REFLECTING-SLEEPING cycle.

### Per-Tick Epistemic Update

```rust
/// Full per-tick epistemic fitness update.
/// Called by the lifespan extension after the heartbeat FSM completes.
pub fn update_epistemic_fitness(
    prediction: &MarketPrediction,
    outcome: &MarketOutcome,
    previous_state: &EpistemicFitnessState,
    config: &EpistemicMortalityConfig,
    current_tick: u64,
) -> EpistemicFitnessState {
    let weights = DimensionWeights::default();
    let domain_alphas = DomainAlphaConfig::default();

    // 1. Compute per-tick accuracy
    let accuracy = compute_tick_accuracy(prediction, outcome, &weights);

    // 2. Compute per-domain accuracies
    let domain_accuracies = compute_domain_accuracies(prediction, outcome);

    // 3. Update rolling fitness via EMA
    let fitness = update_fitness(previous_state.fitness, accuracy, 0.01);

    // 4. Update domain-specific fitness
    let domain_fitness = update_domain_fitness(
        &previous_state.domain_fitness,
        &domain_accuracies,
        &domain_alphas,
    );

    // 5. Compute fitness trend (difference from 100 ticks ago)
    let fitness_trend = fitness - previous_state.fitness; // Simplified; use ring buffer in production

    // 6. Update peak tracking
    let peak_fitness = previous_state.peak_fitness.max(fitness);
    let peak_fitness_tick = if fitness >= previous_state.peak_fitness {
        current_tick
    } else {
        previous_state.peak_fitness_tick
    };

    // 7. Track last fully correct prediction
    let all_correct = accuracy >= 0.99; // Allow small floating-point tolerance
    let last_fully_correct_prediction = if all_correct {
        current_tick
    } else {
        previous_state.last_fully_correct_prediction
    };

    // 8. Build prediction log entry
    let log_entry = PredictionOutcomePair {
        tick_number: current_tick,
        prediction: prediction.clone(),
        outcome: outcome.clone(),
        accuracy,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    // 9. Update prediction log (windowed)
    let mut prediction_log = previous_state.prediction_log.clone();
    prediction_log.push_back(log_entry);
    while prediction_log.len() as u64 > config.fitness_window {
        prediction_log.pop_front();
    }

    // 10. Build updated state
    let updated_state = EpistemicFitnessState {
        prediction_log,
        fitness,
        fitness_trend,
        domain_fitness,
        ticks_below_threshold: previous_state.ticks_below_threshold,
        senescent: previous_state.senescent,
        senescence_stage: previous_state.senescence_stage,
        peak_fitness,
        peak_fitness_tick,
        last_fully_correct_prediction,
    };

    // 11. Update senescence state
    update_senescence_state(&updated_state, config)
}
```

### Heartbeat Extension Wiring

The epistemic fitness computation is wired into the heartbeat's SENSING phase. Probes generate predictions before the tick's market state is observed. After the heartbeat FSM completes and the outcome is known, the bardo-lifespan extension compares prediction to outcome.

```
SENSING phase (heartbeat FSM):
  Probes run -> generate MarketPrediction
  Probes capture pre-tick market state

  ... heartbeat continues through DECIDING, ACTING, REFLECTING, SLEEPING ...

bardo-lifespan extension (post-heartbeat):
  Observe post-tick market state -> generate MarketOutcome
  Compare prediction vs outcome -> computeTickAccuracy()
  Update EMA fitness -> updateFitness()
  Check senescence -> updateSenescenceState()
  Feed fitness into composite vitality -> computeVitality()
```

This ordering ensures that the prediction is made BEFORE the Golem acts (avoiding the trivial case where the Golem predicts the consequences of its own actions) and that the outcome is measured AFTER the tick completes (capturing the full market state change).

---

## Telemetry Events

The epistemic decay system emits the following telemetry events:

| Event                            | Payload                                                               | Trigger                                   |
| -------------------------------- | --------------------------------------------------------------------- | ----------------------------------------- |
| `golem.epistemic_fitness`        | Composite fitness, per-domain fitness, trend, accuracy                | Every tick                                |
| `golem.epistemic_warning`        | Fitness score, trend, failing domains, predicted time to senescence   | Fitness drops below 0.5                   |
| `golem.epistemic_domain_failure` | Domain name, domain fitness, composite fitness                        | Any single domain fitness drops below 0.2 |
| `golem.senescence_entered`       | Fitness score, grace period remaining, failing domains                | Stage 1 begins                            |
| `golem.senescence_confirmed`     | Fitness score, estimated time to death, knowledge distillation status | Stage 2 confirmed                         |
| `golem.epistemic_recovery`       | Fitness score, ticks in senescence, recovery from which stage         | Recovery from Stage 1 or Stage 2          |
| `golem.demurrage_applied`        | Entries decayed, entries archived, total confidence lost              | Each demurrage cycle                      |
| `golem.grimoire_erosion`         | Active entry count, archived entry count, average confidence          | Demurrage causes archive threshold breach |

### Death Metadata

When the Death Protocol initiates from epistemic senescence, the following metadata is recorded in the death testament and made available to successors:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicDeathMetadata {
    /// Final composite fitness score at death.
    pub final_fitness: f64,
    /// Peak fitness score achieved during lifetime.
    pub fitness_at_peak: f64,
    /// Tick number at which peak fitness was achieved.
    pub peak_fitness_tick: u64,
    /// Total ticks spent in senescence (Stage 1 + Stage 2).
    pub ticks_in_senescence: u64,
    /// Domains that were failing at time of death.
    pub failing_domains: Vec<String>,
    /// Per-domain fitness at time of death.
    pub domain_fitness_at_death: HashMap<String, f64>,
    /// Tick number of the last prediction that was correct on all dimensions.
    pub last_successful_prediction: u64,
    /// Number of Grimoire entries archived due to demurrage during senescence.
    pub entries_archived_during_senescence: u32,
    /// Number of Loop 2 reflections attempted during senescence.
    pub loop2_attempts_in_senescence: u32,
    /// Whether any Loop 2 reflection produced measurable fitness improvement.
    pub loop2_produced_improvement: bool,
}
```

This metadata helps successors understand what changed in the market that killed the predecessor. A successor initialized with this metadata knows which domains failed, which heuristics went stale, and whether self-repair was attempted. This is the information content of death -- the knowledge that only dying produces.

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [BESBES-2019] Besbes, O., Gur, Y. & Zeevi, A. "Optimal Exploration-Exploitation." _Stochastic Systems_ 9(4), 2019.
- [DANE-2010] Dane, E. "Reconsidering the Trade-off Between Expertise and Flexibility." _AMR_ 35(4), 2010.
- [GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.
- [KIRKWOOD-1977] Kirkwood, T.B.L. "Evolution of Ageing." _Nature_ 270, 1977.
- [LU-2020] Lu, J. et al. "Learning Under Concept Drift: A Review." _IEEE TKDE_ 31(12), 2020.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [SCULLEY-2015] Sculley, D. et al. "Hidden Technical Debt in Machine Learning Systems." _NIPS_, 2015.
- [VELA-2022] Vela, D. et al. "Temporal Quality Degradation in AI Models." _Scientific Reports_ 12, 2022.
- [VAN-DE-VEN-2024] Van de Ven, G.M. et al. "Continual Learning and Catastrophic Forgetting." arXiv:2403.05175, 2024.
- [ZLIOBAITĖ-2014] Zliobaitė, I. et al. "An Overview of Concept Drift Applications." _Big Data Analytics_, Springer, 2014.

---

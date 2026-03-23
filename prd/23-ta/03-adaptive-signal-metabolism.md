<!-- prd2/23-ta/03-adaptive-signal-metabolism.md -->
<!-- Source: tmp/research/witness-research/new/ta/03-adaptive-signal-metabolism.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Crates**: `bardo-ta-metabolism`
>
> **Cross-references**:
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal perception surface where `dominant_signal_family` and affect modulation signals drive signal metabolism decisions
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle: Gamma evaluates signals, Theta runs reproduction/selection, Delta performs census
> - [01-golem/14b-attention-auction.md](../01-golem/14b-attention-auction.md) -- VCG auction allocating the Golem's finite attention slots where signal species compete for cognitive resources
> - [01-golem/04-mortality.md](../01-golem/04-mortality.md) -- the three death clocks creating compute budget scarcity that forces signal populations to compete for survival
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- HDC foundations for encoding signal identity as hypervectors that enable evolutionary operations (crossover, mutation)
> - [23-ta/00-witness-as-technical-analyst.md](00-witness-as-technical-analyst.md) -- Doc 0: prerequisite system context for the full TA pipeline
> - [23-ta/01-hyperdimensional-technical-analysis.md](01-hyperdimensional-technical-analysis.md) -- Doc 1: HDC pattern algebra providing the genome encoding for signal organisms
> - [23-ta/02-spectral-liquidity-manifolds.md](02-spectral-liquidity-manifolds.md) -- Doc 2: manifold curvature signals that feed as fitness inputs to the signal ecosystem
> - [23-ta/04-causal-microstructure-discovery.md](04-causal-microstructure-discovery.md) -- Doc 4: causal inference providing directed edges that constrain signal evolution

---

> **Reader orientation:** This document specifies the evolutionary signal system where TA indicators are born, reproduce, and die within the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 3 of 10) and covers Hebbian learning for signal reinforcement, replicator dynamics for population management, speciation across DeFi primitives, and dream-cycle variation for offline signal mutation. You should understand evolutionary computation, reinforcement learning, and DeFi protocol mechanics. For Bardo-specific terms, see `prd2/shared/glossary.md`.

# Adaptive Signal Metabolism [SPEC]

**Audience**: Systems engineers and researchers familiar with evolutionary computation, reinforcement learning, and DeFi protocol mechanics. Assumes familiarity with Doc 1 (HDC-based TA signal encoding) and Doc 2 (spectral liquidity manifolds) in this series.

---

## Abstract

Current technical analysis systems treat signals as permanent fixtures. Someone selects RSI, MACD, and Bollinger Bands, wires them into a trading system, and those signals run forever. They consume compute whether they predict or not. They ignore the structural differences between DeFi protocol types. They do not adapt when markets change.

This document presents a signal metabolism: an evolutionary system where TA signals are living computational units with birth, reproduction, speciation, and death. Each signal carries a fitness score derived from prediction accuracy, information gain, and computational cost. Two complementary pressures shape the population. At the micro level, Hebbian learning strengthens signal-context associations that produce accurate predictions and weakens those that fail. At the macro level, replicator dynamics from evolutionary game theory allocate a fixed compute budget across signals proportional to their fitness, starving underperformers until they die. When a generic signal discovers context-dependent performance (accurate for LP rebalancing, inaccurate for lending rate prediction), it forks into context-specific variants through a speciation mechanism analogous to allopatric speciation. Dream cycles inject variation: NREM consolidates long-term fitness records, REM mutates signal parameters to explore the fitness landscape. The result is a self-organizing signal ecosystem that autonomously discovers which TA approaches work for which DeFi primitives, without human curation.

---

## Document structure

Section 1 motivates the problem: why fixed signal sets fail for DeFi. Section 2 builds the mathematical foundations: signal-as-tuple, Hebbian learning, replicator dynamics, speciation criteria, and fitness landscapes. Section 3 describes the architecture and heartbeat integration. Section 4 provides the full Rust implementation: core types, evaluators, the metabolism engine, gamma/theta/delta tick handlers, dream integration, speciation logic, death testaments, context adapters, and bootstrap code. Section 5 maps subsystem interactions (attention auction, mortality, dreams, Grimoire, Daimon, HDC, CorticalState). Section 6 covers DeFi primitive-specific behavior. Section 7 analyzes the cybernetic feedback structure. Section 8 defines the evaluation protocol.

---

## The problem

A Uniswap v3 concentrated liquidity position and an Aave variable-rate borrow are both DeFi positions. That is roughly where the similarity ends. The LP position cares about price trajectory within a tick range, fee accrual rate relative to impermanent loss, and liquidity density shifts around the active tick. The borrow position cares about utilization rate oscillations on the lending pool, the rate curve's slope at current utilization, and liquidation distance as collateral price moves. A 14-period RSI treats both positions identically. It should not.

The fixed-signal problem has four dimensions.

**Wrong signals for the context.** Momentum indicators that work well for directional swap timing carry no information about LP rebalancing decisions. Volume-weighted average price matters for execution quality on swaps but is irrelevant for vault share pricing. A signal set chosen by a human for one DeFi vertical stays bolted on when the Golem enters other verticals.

**No feedback loop.** Traditional TA systems do not track whether their signals predict anything. RSI has been computing overbought/oversold zones since 1978, and if it fails to predict price reversals in a particular market regime, nobody turns it off. It keeps running, consuming attention budget, occupying slots in the CorticalState's 32-signal surface, crowding out signals that might actually work.

**Static in a dynamic environment.** DeFi protocols change. Uniswap v4 hooks alter pool behavior. Lending protocols adjust rate curves. New primitive types emerge (intent-based routing, restaking, prediction markets). A fixed signal set cannot adapt to protocols that did not exist when the signals were designed.

**Computational waste.** Every signal evaluation costs nanoseconds. In a Gamma tick budget of, say, 2 milliseconds for signal evaluation, running 200 signals at 10 microseconds each fills the budget. If half of those signals have near-zero predictive power, you are burning a millisecond on noise. For a Golem running under economic mortality pressure (where wasted compute shortens lifespan), this waste is a survival cost.

What the Golem needs is a system that:

1. Automatically discovers which signals predict outcomes for which DeFi primitive types
2. Kills expensive signals that do not predict
3. Forks generic signals into context-specific variants when accuracy diverges across contexts
4. Adapts to changing market regimes and new protocol types without human intervention
5. Operates within a fixed compute budget per tick, trading off signal count against signal quality

The biological analogy is metabolism. Living organisms allocate energy to organs and processes that contribute to survival. Organs that do not earn their metabolic cost atrophy. Gene variants that improve fitness in a given environment proliferate. Populations speciate when geographic isolation drives divergent selection pressure. The signal metabolism applies these dynamics to TA signals.

---

## Mathematical foundations [SPEC]

### Signal as living unit

Define a TA signal *S* as a five-tuple:

```
S = (f, C, H, W, ctx)
```

where:

- *f* : Observation -> Prediction is the signal function. It takes a `DeFiObservation` (price, volume, protocol state) and produces a directional prediction with confidence and time horizon.
- *C* is the computational cost per evaluation, measured as a rolling exponential average of wall-clock nanoseconds.
- *H* is the prediction accuracy history, an exponentially weighted moving average (EWMA) of binary outcomes (correct/incorrect) with decay factor alpha = 0.05.
- *W* is the composite fitness score, recomputed each Theta tick.
- *ctx* is the set of DeFi context types this signal applies to, with per-context Hebbian weights.

The signal's fitness is not a static property. It changes every Theta tick as new prediction outcomes resolve. A signal that predicted well last week but poorly today sees its fitness drop as the EWMA decays old successes and weights recent failures.

The living unit abstraction means signals can die (fitness drops below extinction threshold), reproduce (fork into context-specific variants), and mutate (parameter perturbation during REM dreams). The population of active signals is not designed; it evolves.

### Hebbian learning at the micro level

Donald Hebb's principle (1949): "neurons that fire together wire together." Applied here, signals that activate (produce predictions) in contexts where outcomes are favorable strengthen their association with those contexts. Signals that activate in contexts where outcomes are unfavorable weaken their association.

Formalize with a weight matrix **W** where *w_ij* is the connection strength between signal *i* and DeFi context *j*. Let *a_i* be signal *i*'s activation (the absolute value of its prediction, normalized to [0, 1]) and let *o_j* be the binary outcome for context *j* (1 if the prediction was profitable, 0 otherwise).

The combined Hebbian/anti-Hebbian update rule:

```
delta_w_ij = eta * a_i * (2 * o_j - 1)
```

When o_j = 1 (profitable): delta_w_ij = +eta * a_i. The signal strengthens its connection to this context.

When o_j = 0 (unprofitable): delta_w_ij = -eta * a_i. The signal weakens its connection.

This is a variant of Oja's rule (1982) applied to signal-context associations rather than neural weights. The factor `(2 * o_j - 1)` maps the binary outcome to {-1, +1}, creating symmetric reinforcement and punishment.

The learning rate *eta* is not constant. It modulates based on the Golem's emotional state via the Daimon subsystem:

```
eta_effective = eta_base * daimon_modulator

where:
  daimon_modulator = 1.0 + fear_level * 2.0    (fear accelerates learning)
  daimon_modulator = 1.0 - calm_level * 0.5     (calm slows learning)
```

Fear after a loss event doubles the learning rate, so the Golem rapidly downweights signals that contributed to the loss. Calm during stable operation halves the rate, preventing overreaction to noise. This mirrors the neurobiological finding that amygdala activation (fear) strengthens memory consolidation (McGaugh, 2004).

**Weight normalization.** After each update, context weights for each signal are normalized to sum to 1.0 across all contexts. This prevents unbounded growth and forces signals to compete: strengthening one context association necessarily weakens others.

```
w_ij <- w_ij / sum_k(w_ik)
```

**Interpretation.** After many Theta ticks, a signal's context weight vector reveals which DeFi primitives it predicts well. A momentum signal might evolve weights of 0.6 for Swap, 0.2 for PerpetualSwap, 0.1 for Options, and near-zero for everything else. This weight vector directly influences the signal's evaluation priority: in a Swap context, the momentum signal runs first and its output carries more weight in the Oracle's attention bid.

### Economic selection at the macro level

Hebbian learning adjusts which contexts a signal associates with. Economic selection determines whether the signal survives at all. The mechanism is the replicator equation from evolutionary game theory (Taylor & Jonker, 1978).

Every signal has a maintenance cost: the compute nanoseconds it consumes per Gamma tick evaluation. The Golem has a fixed total compute budget *B* per Gamma tick (e.g., 2,000,000 nanoseconds = 2 milliseconds). Signals compete for shares of this budget.

**Fitness function.** Signal fitness combines three terms:

```
W_i = alpha * accuracy_i + beta * info_gain_i - gamma * normalized_cost_i
```

where:

- `accuracy_i` is the EWMA of correct predictions (in [0, 1])
- `info_gain_i` is the EWMA of the information gain from each prediction, measured as the reduction in entropy about the outcome
- `normalized_cost_i` is C_i / max(C_j for all j), mapping cost to [0, 1]
- alpha, beta, gamma are weights (default: 0.5, 0.3, 0.2)

The information gain term prevents a degenerate equilibrium where signals always predict the majority class. A signal that predicts "price goes up" in a bull market has high accuracy but low information gain (the base rate already predicted up). A signal that correctly identifies the few reversals has lower accuracy but much higher information gain per correct prediction.

**Replicator dynamics.** Let x_i be signal i's share of the total compute budget (sum of all x_i = 1). Let W_bar be the mean fitness across all signals, weighted by budget share:

```
W_bar = sum_i(x_i * W_i)
```

The replicator equation governs budget share evolution:

```
dx_i/dt = x_i * (W_i - W_bar) * selection_pressure
```

Signals with above-average fitness grow their budget share. Signals below average shrink. The `selection_pressure` parameter (default: 0.1) controls how aggressively the system reallocates. Too high and the system collapses to a monoculture of the current best signal. Too low and dead weight persists.

In discrete time (one Theta tick per update):

```
x_i(t+1) = x_i(t) * (1 + selection_pressure * (W_i - W_bar))
x_i(t+1) = max(x_i(t+1), 0)   // clamp to non-negative
// renormalize so all shares sum to 1
```

**Death.** When a signal's budget share drops below `extinction_threshold` (default: 0.01, meaning less than 1% of the budget), the signal is killed. Its registry entry persists as a tombstone for one Delta cycle (to avoid immediate re-creation of the same signal type), then is garbage collected.

**Fisher's fundamental theorem.** Fisher (1930) proved that the rate of increase in mean fitness of a population equals the genetic variance in fitness. Applied here: the rate of improvement in the signal population's aggregate prediction quality is proportional to the diversity of fitness values across signals. This has a practical implication: the system should maintain fitness diversity. Premature convergence to a single high-fitness signal eliminates the variance that drives improvement.

The speciation mechanism (next section) is the primary tool for maintaining diversity.

**Steady-state analysis.** At steady state, the replicator equation reaches an equilibrium where all surviving signals have equal fitness (W_i = W_bar for all i). Any signal with fitness below the mean shrinks toward zero; any signal above the mean grows until its growth dilutes its own advantage (through the budget normalization step). The equilibrium signal count depends on the fitness function's ability to discriminate between signals. With alpha = 0.5, beta = 0.3, gamma = 0.2, the typical steady-state population is 15-40 signals, assuming a starting population of 10 and speciation active.

**Computational complexity per Theta tick.** Let N be the number of active signals and K the number of DeFi contexts. The Hebbian update is O(N) per resolved prediction (one weight update per signal-context pair). Budget reallocation is O(N) for the replicator step. Speciation check is O(N * K^2) in the worst case (comparing all context pairs for each signal), but in practice K is fixed at 15 and N is bounded by the compute budget (typically under 50), so the total cost per Theta tick is under 1 microsecond.

### Signal speciation [SPEC]

When a generic signal performs well in one context but poorly in another, it should fork. The original becomes two specialized variants, each inheriting the parent's parameters but diverging in their context weights.

**Speciation criterion.** For signal *S* evaluated in contexts *c1* and *c2*, define the performance divergence:

```
divergence(S, c1, c2) = |accuracy(S, c1) - accuracy(S, c2)|
```

where `accuracy(S, c)` is the EWMA of predictions made specifically when context was *c*. When this divergence exceeds the speciation threshold theta_s (default: 0.25, meaning 25 percentage points of accuracy difference), the signal speciates.

The fork produces two new signals:

```
S -> S_c1 + S_c2
```

Each child inherits:
- The parent's signal function (same evaluator, same parameters)
- The parent's accuracy history, filtered to the relevant context
- Half the parent's budget share
- The parent's generation number + 1

Each child receives:
- A new unique SignalId
- Context weights zeroed for all contexts except its specialization
- A `parent_id` pointer for lineage tracking

The biological analogy is allopatric speciation: geographic isolation (different DeFi contexts) prevents gene flow (shared weight updates) and allows independent adaptation. Over subsequent Theta ticks, the children's parameters will diverge as Hebbian learning and REM mutations push them toward local optima in their respective contexts.

**Anti-speciation guard.** To prevent a combinatorial explosion of micro-specialized signals, two constraints apply:

1. A signal must have been evaluated at least `min_evaluations_for_speciation` times (default: 100) in each context before speciation can trigger.
2. The maximum generation depth is capped at `max_generation` (default: 5). A signal that has already been forked 5 times from the original ancestor cannot fork again.

### The fitness landscape

Model the space of possible signal configurations (parameter values, context weights, evaluator types) as a fitness landscape in the sense of Sewall Wright (1932). Each point in the space maps to a fitness value. Peaks are high-fitness configurations; valleys are low-fitness configurations.

The population of active signals sits on this landscape. Hebbian learning performs gradient ascent toward nearby peaks. REM mutations provide the random jumps needed to escape local optima. Speciation splits a population straddling a saddle point into two sub-populations, each ascending toward a different peak.

The landscape shifts. Market regime changes move the peaks and valleys. A momentum signal configuration that was a fitness peak during a trending market becomes a valley during a mean-reverting market. This is the Red Queen dynamic (Van Valen, 1973): signals must keep evolving to maintain their current fitness, because the landscape they are evolving on is itself evolving.

The practical consequence: the signal population never reaches a stable equilibrium. It tracks the moving fitness landscape through continuous Hebbian updates, periodic speciation events, and REM-driven exploration. A Golem that stops dreaming stops adapting.

**Landscape dimensionality.** Each signal's configuration lives in a parameter space whose dimensionality depends on the evaluator type. A `MomentumSignal` has two parameters (lookback, threshold), so its landscape is two-dimensional. A `MeanReversionSignal` also has two (window, std_dev_threshold). But the full population landscape includes context weights (15 dimensions per signal) and the combinatorial composition of the population itself (which signal types are present and at what budget shares). The effective dimensionality of the search space is high enough that exhaustive exploration is impossible. Replicator dynamics explore by amplifying what works. REM mutations explore by random perturbation. Speciation explores by splitting the search into independent subspaces.

Holland (1975) formalized this as the schema theorem: in a genetic algorithm, short, low-order, above-average schemata receive exponentially increasing trials in subsequent generations. The signal metabolism implements this implicitly. A "schema" here is a signal type (e.g., "momentum with lookback in [10, 20]") and the replicator equation gives above-average instances of that schema increasing budget share, which is equivalent to increasing trials.

---

## Architecture [SPEC]

### Signal registry [SPEC]

The `SignalRegistry` is the central data structure. It owns all active signals and provides O(1) lookup by `SignalId`, O(n) iteration for tick-level evaluation, and O(1) removal for signal death.

```
SignalRegistry
  |
  +-- signals: HashMap<SignalId, LiveSignal>
  +-- next_id: AtomicU64
  +-- tombstones: HashMap<SignalId, Instant>  // recently killed, prevents re-creation
  +-- lineage: HashMap<SignalId, Vec<SignalId>>  // parent -> children
```

Each `LiveSignal` contains:
- The signal function (a trait object implementing `SignalEvaluator`)
- Fitness tracking state (accuracy, information gain, profit contribution, composite score, budget share)
- Hebbian context weights (a `HashMap<DeFiContext, f64>`)
- Computational cost estimate (rolling EWMA of nanoseconds per evaluation)
- HDC hypervector encoding the signal's identity (from Doc 1)
- Lineage data (parent ID, generation number, birth tick)

### Heartbeat integration [SPEC]

The metabolism hooks into each phase of the Golem's heartbeat cycle.

**Gamma tick (5-15 seconds): perception.** All active signals whose budget share exceeds the minimum evaluation threshold (default: 0.02) are evaluated against the current `DeFiObservation`. Each signal produces a `SignalOutput` (prediction, confidence, horizon). These outputs feed into the Oracle's attention bid calculation for the next Theta tick. Predictions are timestamped and queued as `PendingPrediction` entries awaiting outcome resolution.

Signal evaluation respects the compute budget. If cumulative evaluation time would exceed `total_budget_ns`, signals are evaluated in priority order (highest fitness first) and remaining signals are skipped for this tick. Skipped signals incur a small fitness penalty (they consumed budget share without producing predictions).

**Theta tick (30-120 seconds): cognition.** Four metabolism operations run in sequence:

1. *Prediction resolution.* Pending predictions whose horizons have elapsed are resolved against actual outcomes. Each resolved prediction updates the originating signal's accuracy and information gain EWMAs.
2. *Hebbian update.* Context weights for all signals with resolved predictions are adjusted using the Hebbian/anti-Hebbian rule.
3. *Budget reallocation.* The replicator dynamics step recalculates each signal's budget share based on current fitness.
4. *Speciation check.* Each signal with sufficient evaluation history is checked for context-dependent performance divergence. Signals exceeding the speciation threshold are forked.

**Delta tick (~50 Theta ticks): consolidation.** The metabolism performs housekeeping:

1. *Pruning.* Signals below the extinction threshold are killed. Their budget share is redistributed proportionally to surviving signals.
2. *Tombstone cleanup.* Tombstones older than one Delta cycle are removed.
3. *Statistics.* Population-level metrics are computed: mean fitness, fitness variance, species count, Herfindahl-Hirschman index of budget concentration.
4. *Dream preparation.* The metabolism exports a `DreamPacket` containing the current signal population summary, fitness distributions, and recently killed signal IDs for the dream consolidation system.

**NREM dream: consolidation.** The dream system receives the metabolism's `DreamPacket` and consolidates signal fitness histories. Short-term EWMA values are blended with longer-horizon averages to identify signals that have been reliably fit over many Delta cycles versus signals that spiked recently. Reliable signals receive a stability bonus to their fitness score, making them harder to kill during transient market regime changes.

**REM dream: exploration.** The dream system calls `dream_rem()` on the metabolism. For each signal with above-median fitness, a mutant clone is created by calling `SignalEvaluator::mutate()`, which perturbs the signal's internal parameters (thresholds, lookback windows, smoothing factors) by a random amount drawn from a normal distribution with standard deviation proportional to (1 - fitness). High-fitness signals mutate less; low-fitness signals mutate more. The mutant enters the population with a small initial budget share (default: 0.03) and must earn its survival through subsequent Theta ticks.

This is the explore/exploit tradeoff. NREM exploits by consolidating what works. REM explores by injecting variation. Without REM, the population converges to a local optimum and cannot track a shifting fitness landscape.

### Attention auction integration [SPEC]

The signal metabolism feeds the Oracle bidder in the VCG attention auction. The Oracle's valuation for observing a particular DeFi context is proportional to the aggregate fitness of signals specialized for that context:

```
oracle_value(context) = sum over signals S where w(S, context) > 0.1:
    w(S, context) * fitness(S)
```

When the signal population has high-fitness signals for LP contexts, the Oracle bids aggressively for LP observation slots. When LP signals are performing poorly, the Oracle's LP bids decrease, and attention shifts to contexts where signals are performing better. The metabolism drives the Golem's attention allocation through the auction mechanism.

---

## Implementation [SPEC]

### Core types

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Unique signal identifier. Monotonically increasing, never reused.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SignalId(pub u64);

/// DeFi context type. Determines which signals are relevant
/// and drives Hebbian weight specialization.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeFiContext {
    Swap,
    LiquidityProvision,
    Lending,
    Borrowing,
    Vault,
    Staking,
    Restaking,
    PerpetualSwap,
    Options,
    YieldTokens,
    Streaming,
    GasMarket,
    IntentBased,
    CrossChain,
    PredictionMarket,
}

impl DeFiContext {
    /// All known context variants, for iterating over the Hebbian weight matrix.
    pub fn all() -> &'static [DeFiContext] {
        use DeFiContext::*;
        &[
            Swap, LiquidityProvision, Lending, Borrowing, Vault,
            Staking, Restaking, PerpetualSwap, Options, YieldTokens,
            Streaming, GasMarket, IntentBased, CrossChain, PredictionMarket,
        ]
    }
}
```

### Exponential moving average

The fitness tracking system uses EWMAs throughout. Rather than storing full histories (which grow without bound), each metric is a single float that decays old observations exponentially.

```rust
/// Exponentially weighted moving average with configurable decay.
///
/// alpha controls the decay rate: alpha = 0.05 means each new observation
/// receives 5% weight, and old observations decay by 95% per update.
/// After ~20 updates (1/alpha), the initial value contributes less than
/// 37% of the running average.
#[derive(Clone, Debug)]
pub struct ExponentialAverage {
    value: f64,
    alpha: f64,
    count: u64,
}

impl ExponentialAverage {
    pub fn new(alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha must be in (0, 1]");
        Self {
            value: 0.5, // neutral prior
            alpha,
            count: 0,
        }
    }

    /// Update with a new observation in [0, 1].
    pub fn update(&mut self, observation: f64) {
        let obs = observation.clamp(0.0, 1.0);
        if self.count == 0 {
            // First observation: override the prior
            self.value = obs;
        } else {
            self.value = self.alpha * obs + (1.0 - self.alpha) * self.value;
        }
        self.count += 1;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    /// Reset to the neutral prior. Used during dream consolidation
    /// to merge short-term and long-term averages.
    pub fn reset_with_value(&mut self, val: f64) {
        self.value = val.clamp(0.0, 1.0);
        self.count = 0;
    }
}
```

### Signal fitness tracking

```rust
/// Tracks all fitness-related metrics for a single signal.
///
/// Updated at Theta tick when prediction outcomes resolve.
/// The composite fitness_score is recomputed after each update.
#[derive(Clone, Debug)]
pub struct SignalFitness {
    /// Fraction of predictions that were directionally correct.
    pub accuracy: ExponentialAverage,

    /// Bits of information gained per prediction.
    /// Measures how much the prediction reduced uncertainty about the outcome
    /// beyond the base rate.
    pub information_gain: ExponentialAverage,

    /// Profit contribution: correlation between prediction confidence
    /// and realized PnL, normalized to [0, 1].
    pub profit_contribution: ExponentialAverage,

    /// Composite fitness score, recomputed each Theta tick.
    pub fitness_score: f64,

    /// Share of the total compute budget allocated to this signal.
    /// Sum of all signals' budget_share = 1.0.
    pub budget_share: f64,

    /// Per-context accuracy tracking for speciation detection.
    /// Key: context, Value: (correct_count, total_count, ewma_accuracy)
    pub context_accuracy: HashMap<DeFiContext, ContextAccuracy>,
}

#[derive(Clone, Debug)]
pub struct ContextAccuracy {
    pub correct: u64,
    pub total: u64,
    pub ewma: ExponentialAverage,
}

impl ContextAccuracy {
    pub fn new() -> Self {
        Self {
            correct: 0,
            total: 0,
            ewma: ExponentialAverage::new(0.05),
        }
    }

    pub fn record(&mut self, was_correct: bool) {
        self.total += 1;
        if was_correct {
            self.correct += 1;
        }
        self.ewma.update(if was_correct { 1.0 } else { 0.0 });
    }

    pub fn accuracy(&self) -> f64 {
        self.ewma.value()
    }
}

impl SignalFitness {
    pub fn new() -> Self {
        Self {
            accuracy: ExponentialAverage::new(0.05),
            information_gain: ExponentialAverage::new(0.05),
            profit_contribution: ExponentialAverage::new(0.05),
            fitness_score: 0.5,
            budget_share: 0.0,
            context_accuracy: HashMap::new(),
        }
    }

    /// Recompute composite fitness from component metrics.
    pub fn recompute(&mut self, config: &FitnessWeights, normalized_cost: f64) {
        self.fitness_score = config.accuracy_weight * self.accuracy.value()
            + config.info_gain_weight * self.information_gain.value()
            - config.cost_weight * normalized_cost;
        // Clamp to [0, 1] so replicator dynamics stay well-behaved
        self.fitness_score = self.fitness_score.clamp(0.0, 1.0);
    }
}
```

### Signal evaluator trait and output [SPEC]

```rust
/// The prediction output from a single signal evaluation.
#[derive(Clone, Debug)]
pub struct SignalOutput {
    /// Predicted direction and magnitude.
    /// Positive = bullish, negative = bearish.
    /// Magnitude indicates strength of conviction.
    pub prediction: f64,

    /// Confidence in [0, 1]. A signal should output low confidence
    /// when it lacks sufficient data or when its internal state is ambiguous.
    pub confidence: f64,

    /// Number of Gamma ticks until this prediction should be resolved.
    /// A horizon of 1 means "next Gamma tick." A horizon of 10 means
    /// "10 Gamma ticks from now."
    pub horizon_ticks: u32,
}

/// Observation data passed to signal evaluators each Gamma tick.
/// Contains the raw market data a signal needs to compute its prediction.
#[derive(Clone, Debug)]
pub struct DeFiObservation {
    pub tick: u64,
    pub timestamp_ms: u64,
    pub context: DeFiContext,

    // Price data
    pub price: f64,
    pub price_history: Vec<f64>, // last N prices, most recent first
    pub volume_history: Vec<f64>,

    // Protocol-specific state
    pub utilization_rate: Option<f64>,      // lending
    pub liquidity_depth: Option<f64>,       // AMM
    pub funding_rate: Option<f64>,          // perps
    pub implied_volatility: Option<f64>,    // options
    pub share_price: Option<f64>,           // vaults
    pub gas_price_gwei: f64,
}

/// The core trait for signal evaluation functions.
///
/// Implementors define the actual TA logic: how to read an observation
/// and produce a prediction. The trait also supports cloning (for speciation)
/// and mutation (for REM dreams).
pub trait SignalEvaluator: Send + Sync {
    /// Evaluate the observation and return a prediction.
    fn evaluate(&self, observation: &DeFiObservation) -> SignalOutput;

    /// Human-readable name for logging and debugging.
    fn name(&self) -> &str;

    /// Clone into a boxed trait object. Required because Box<dyn Trait>
    /// does not implement Clone.
    fn clone_box(&self) -> Box<dyn SignalEvaluator>;

    /// Produce a mutated variant of this evaluator.
    /// The mutation magnitude should be proportional to `mutation_strength`
    /// (typically 1.0 - fitness, so high-fitness signals mutate less).
    fn mutate(&self, rng: &mut impl rand::Rng, mutation_strength: f64) -> Box<dyn SignalEvaluator>;

    /// Return the evaluator's internal parameters as a serializable map.
    /// Used for lineage tracking and death testament extraction.
    fn parameters(&self) -> HashMap<String, f64>;
}
```

### The LiveSignal struct

```rust
/// A single TA signal as a living unit in the metabolism.
///
/// Created by `SignalMetabolism::birth_signal()`, evaluated at each Gamma tick,
/// updated at each Theta tick, and killed when fitness drops below the
/// extinction threshold.
pub struct LiveSignal {
    pub id: SignalId,
    pub name: String,
    pub parent_id: Option<SignalId>,
    pub generation: u32,

    // The signal function
    evaluator: Box<dyn SignalEvaluator>,

    // Fitness tracking
    pub fitness: SignalFitness,

    // Hebbian context weights: w_ij for this signal (i) across all contexts (j).
    // Normalized to sum to 1.0 after each update.
    pub context_weights: HashMap<DeFiContext, f64>,

    // Computational cost: rolling EWMA of nanoseconds per evaluation.
    pub compute_cost_ns: f64,

    // HDC encoding of this signal's identity (from Doc 1).
    // Used for similarity queries: "find signals similar to this one."
    pub signal_hv: [u64; 160], // 10,240 bits = 160 x u64

    // Lifecycle
    pub born_at: u64,              // Gamma tick number at birth
    pub total_evaluations: u64,
    pub last_evaluated_tick: u64,
}

impl LiveSignal {
    /// Create a new signal with uniform context weights.
    pub fn new(
        id: SignalId,
        evaluator: Box<dyn SignalEvaluator>,
        parent_id: Option<SignalId>,
        generation: u32,
        born_at: u64,
        signal_hv: [u64; 160],
    ) -> Self {
        let name = evaluator.name().to_string();
        let num_contexts = DeFiContext::all().len() as f64;
        let uniform_weight = 1.0 / num_contexts;

        let mut context_weights = HashMap::new();
        for &ctx in DeFiContext::all() {
            context_weights.insert(ctx, uniform_weight);
        }

        Self {
            id,
            name,
            parent_id,
            generation,
            evaluator,
            fitness: SignalFitness::new(),
            context_weights,
            compute_cost_ns: 0.0,
            signal_hv,
            born_at,
            total_evaluations: 0,
            last_evaluated_tick: 0,
        }
    }

    /// Evaluate against an observation, tracking wall-clock cost.
    pub fn evaluate(&mut self, obs: &DeFiObservation) -> SignalOutput {
        let start = Instant::now();
        let output = self.evaluator.evaluate(obs);
        let elapsed_ns = start.elapsed().as_nanos() as f64;

        // Update cost EWMA (alpha = 0.1 for faster cost tracking)
        if self.total_evaluations == 0 {
            self.compute_cost_ns = elapsed_ns;
        } else {
            self.compute_cost_ns = 0.1 * elapsed_ns + 0.9 * self.compute_cost_ns;
        }

        self.total_evaluations += 1;
        self.last_evaluated_tick = obs.tick;
        output
    }

    /// Context weight for a given DeFi context.
    /// Returns 0.0 for unknown contexts rather than panicking.
    pub fn context_weight(&self, ctx: DeFiContext) -> f64 {
        self.context_weights.get(&ctx).copied().unwrap_or(0.0)
    }

    /// The signal's effective priority for a given context:
    /// context_weight * fitness_score. Used to rank signals for
    /// evaluation order within the Gamma tick compute budget.
    pub fn priority(&self, ctx: DeFiContext) -> f64 {
        self.context_weight(ctx) * self.fitness.fitness_score
    }
}
```

### Prediction tracking

```rust
/// A prediction awaiting resolution.
/// Created at Gamma tick when a signal evaluates, resolved at Theta tick
/// when the prediction horizon has elapsed.
#[derive(Clone, Debug)]
pub struct PendingPrediction {
    pub signal_id: SignalId,
    pub context: DeFiContext,
    pub prediction: f64,
    pub confidence: f64,
    pub created_at_tick: u64,
    pub resolve_at_tick: u64, // created_at_tick + horizon_ticks
    pub reference_price: f64, // price at prediction time, for resolution
}

/// The outcome of a resolved prediction.
#[derive(Clone, Debug)]
pub struct PredictionOutcome {
    pub signal_id: SignalId,
    pub context: DeFiContext,
    pub prediction: f64,
    pub confidence: f64,
    pub actual_direction: f64,    // positive = price went up, negative = down
    pub was_correct: bool,        // did the prediction match the direction?
    pub magnitude_error: f64,     // |predicted_magnitude - actual_magnitude|
    pub information_gain: f64,    // bits of info gained vs. base rate
}

impl PendingPrediction {
    /// Resolve this prediction against the actual price movement.
    pub fn resolve(&self, current_price: f64, base_rate_up: f64) -> PredictionOutcome {
        let actual_direction = current_price - self.reference_price;
        let predicted_up = self.prediction > 0.0;
        let actual_up = actual_direction > 0.0;
        let was_correct = predicted_up == actual_up;

        // Information gain: how much this prediction reduced uncertainty
        // beyond the base rate. If the base rate says 60% chance of up,
        // predicting up and being right gives less info than predicting
        // down and being right.
        let prior_prob = if predicted_up { base_rate_up } else { 1.0 - base_rate_up };
        let info_gain = if was_correct {
            // Gained info = -log2(prior_prob). Correct prediction on unlikely
            // outcome = high information.
            (-prior_prob.ln() / std::f64::consts::LN_2).clamp(0.0, 4.0)
        } else {
            0.0 // incorrect predictions contribute zero information
        };

        PredictionOutcome {
            signal_id: self.signal_id,
            context: self.context,
            prediction: self.prediction,
            confidence: self.confidence,
            actual_direction,
            was_correct,
            magnitude_error: (self.prediction.abs() - actual_direction.abs()).abs(),
            information_gain: info_gain,
        }
    }
}
```

### Configuration

```rust
/// All tunable parameters for the signal metabolism.
/// Defaults are conservative. Aggressive parameters
/// (high selection pressure, low extinction threshold)
/// produce faster adaptation but risk premature convergence.
#[derive(Clone, Debug)]
pub struct MetabolismConfig {
    /// Nanoseconds available per Gamma tick for signal evaluation.
    /// Default: 2_000_000 (2ms).
    pub total_budget_ns: u64,

    /// Base Hebbian learning rate. Modulated by Daimon.
    /// Default: 0.01.
    pub learning_rate: f64,

    /// Speciation threshold: minimum accuracy divergence between
    /// two contexts to trigger a fork.
    /// Default: 0.25 (25 percentage points).
    pub speciation_threshold: f64,

    /// Minimum budget share to survive. Below this, the signal dies.
    /// Default: 0.01 (1% of budget).
    pub extinction_threshold: f64,

    /// How aggressively replicator dynamics reallocate budget.
    /// Default: 0.1. Range: [0.01, 1.0].
    pub selection_pressure: f64,

    /// Minimum evaluations in a context before speciation can trigger.
    /// Default: 100.
    pub min_evaluations_for_speciation: u64,

    /// Maximum generation depth (forking limit).
    /// Default: 5.
    pub max_generation: u32,

    /// Budget share allocated to newly birthed signals.
    /// Default: 0.03 (3%).
    pub initial_budget_share: f64,

    /// Fitness function weights.
    pub fitness_weights: FitnessWeights,

    /// REM dream mutation parameters.
    pub mutation_config: MutationConfig,
}

#[derive(Clone, Debug)]
pub struct FitnessWeights {
    pub accuracy_weight: f64,
    pub info_gain_weight: f64,
    pub cost_weight: f64,
}

#[derive(Clone, Debug)]
pub struct MutationConfig {
    /// Fraction of above-median-fitness signals to clone+mutate during REM.
    /// Default: 0.3 (30%).
    pub mutation_fraction: f64,

    /// Maximum number of mutations per REM dream.
    /// Default: 5.
    pub max_mutations_per_dream: usize,

    /// Base mutation strength, scaled by (1 - fitness).
    /// Default: 0.2.
    pub base_mutation_strength: f64,
}

impl Default for MetabolismConfig {
    fn default() -> Self {
        Self {
            total_budget_ns: 2_000_000,
            learning_rate: 0.01,
            speciation_threshold: 0.25,
            extinction_threshold: 0.01,
            selection_pressure: 0.1,
            min_evaluations_for_speciation: 100,
            max_generation: 5,
            initial_budget_share: 0.03,
            fitness_weights: FitnessWeights {
                accuracy_weight: 0.5,
                info_gain_weight: 0.3,
                cost_weight: 0.2,
            },
            mutation_config: MutationConfig {
                mutation_fraction: 0.3,
                max_mutations_per_dream: 5,
                base_mutation_strength: 0.2,
            },
        }
    }
}
```

### The metabolism engine [SPEC]

```rust
/// The metabolism engine. Owns all active signals, manages their lifecycle,
/// and integrates with the heartbeat clock.
///
/// Instantiated once per Golem, lives for the Golem's entire lifespan.
/// Serializable via `death_testament()` for successor inheritance.
pub struct SignalMetabolism {
    signals: HashMap<SignalId, LiveSignal>,
    next_id: AtomicU64,

    config: MetabolismConfig,

    // Pending predictions awaiting outcome resolution.
    pending_predictions: VecDeque<PendingPrediction>,

    // Base rate tracker: fraction of recent ticks where price went up.
    // Used for information gain calculation.
    base_rate_up: ExponentialAverage,

    // Tombstones: recently killed signals. Prevents immediate re-creation.
    tombstones: HashMap<SignalId, Instant>,

    // Population statistics, updated at Delta tick.
    stats: PopulationStats,

    // Current Gamma tick number.
    current_tick: u64,
}

#[derive(Clone, Debug, Default)]
pub struct PopulationStats {
    pub mean_fitness: f64,
    pub fitness_variance: f64,
    pub species_count: usize,
    pub total_evaluations: u64,
    pub signals_born: u64,
    pub signals_killed: u64,
    /// Herfindahl-Hirschman Index of budget concentration.
    /// 1/N = perfectly uniform, 1.0 = total monopoly.
    pub hhi: f64,
}

impl SignalMetabolism {
    pub fn new(config: MetabolismConfig) -> Self {
        Self {
            signals: HashMap::new(),
            next_id: AtomicU64::new(1),
            config,
            pending_predictions: VecDeque::new(),
            base_rate_up: ExponentialAverage::new(0.02),
            tombstones: HashMap::new(),
            stats: PopulationStats::default(),
            current_tick: 0,
        }
    }

    /// Register a new signal into the population.
    /// Allocates initial budget share and assigns a unique ID.
    pub fn birth_signal(
        &mut self,
        evaluator: Box<dyn SignalEvaluator>,
        parent: Option<SignalId>,
        signal_hv: [u64; 160],
    ) -> SignalId {
        let id = SignalId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let generation = parent
            .and_then(|pid| self.signals.get(&pid))
            .map(|p| p.generation + 1)
            .unwrap_or(0);

        let mut signal = LiveSignal::new(
            id, evaluator, parent, generation, self.current_tick, signal_hv,
        );
        signal.fitness.budget_share = self.config.initial_budget_share;

        // Steal budget from all existing signals proportionally.
        let steal_total = self.config.initial_budget_share;
        let existing_budget: f64 = self.signals.values()
            .map(|s| s.fitness.budget_share)
            .sum();

        if existing_budget > 0.0 {
            let scale = (existing_budget - steal_total).max(0.0) / existing_budget;
            for s in self.signals.values_mut() {
                s.fitness.budget_share *= scale;
            }
        }

        if let Some(pid) = parent {
            // Track lineage (not shown: lineage map update)
            let _ = pid; // lineage tracking in production
        }

        self.signals.insert(id, signal);
        self.stats.signals_born += 1;
        id
    }

    /// Kill a signal. Redistributes its budget share to survivors.
    fn kill_signal(&mut self, signal_id: SignalId) {
        if let Some(dead) = self.signals.remove(&signal_id) {
            let freed = dead.fitness.budget_share;
            let surviving_budget: f64 = self.signals.values()
                .map(|s| s.fitness.budget_share)
                .sum();

            if surviving_budget > 0.0 {
                let scale = (surviving_budget + freed) / surviving_budget;
                for s in self.signals.values_mut() {
                    s.fitness.budget_share *= scale;
                }
            }

            self.tombstones.insert(signal_id, Instant::now());
            self.stats.signals_killed += 1;
        }
    }
}
```

### Gamma tick: signal evaluation

```rust
impl SignalMetabolism {
    /// Run all active signals against the current observation.
    /// Returns the signal outputs, sorted by effective priority.
    ///
    /// Respects the compute budget: signals are evaluated in priority order
    /// and skipped if the budget would be exceeded.
    pub fn gamma_tick(
        &mut self,
        obs: &DeFiObservation,
    ) -> Vec<(SignalId, SignalOutput)> {
        self.current_tick = obs.tick;

        // Sort signals by priority for this context (descending).
        let mut signal_ids: Vec<SignalId> = self.signals.keys().copied().collect();
        signal_ids.sort_by(|a, b| {
            let pa = self.signals[b].priority(obs.context);
            let pb = self.signals[a].priority(obs.context);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut results = Vec::new();
        let mut budget_used_ns: u64 = 0;
        let budget_limit = self.config.total_budget_ns;

        for sid in signal_ids {
            let signal = match self.signals.get_mut(&sid) {
                Some(s) => s,
                None => continue,
            };

            // Skip signals with negligible budget share.
            if signal.fitness.budget_share < 0.005 {
                continue;
            }

            // Check if evaluating this signal would exceed the budget.
            let estimated_cost = signal.compute_cost_ns as u64;
            if budget_used_ns + estimated_cost > budget_limit && !results.is_empty() {
                // Budget exhausted. Skip remaining signals.
                // (Always evaluate at least one signal, even if it exceeds budget.)
                break;
            }

            let output = signal.evaluate(obs);
            budget_used_ns += signal.compute_cost_ns as u64;

            // Queue prediction for future resolution.
            if output.confidence > 0.1 {
                self.pending_predictions.push_back(PendingPrediction {
                    signal_id: sid,
                    context: obs.context,
                    prediction: output.prediction,
                    confidence: output.confidence,
                    created_at_tick: obs.tick,
                    resolve_at_tick: obs.tick + output.horizon_ticks as u64,
                    reference_price: obs.price,
                });
            }

            results.push((sid, output));
        }

        results
    }
}
```

### Theta tick: learning, reallocation, speciation

```rust
impl SignalMetabolism {
    /// Resolve pending predictions and update signal fitness.
    /// Called at Theta tick with the current price for resolution.
    pub fn resolve_predictions(&mut self, current_price: f64) -> Vec<PredictionOutcome> {
        let mut outcomes = Vec::new();
        let mut unresolved = VecDeque::new();

        while let Some(pending) = self.pending_predictions.pop_front() {
            if self.current_tick >= pending.resolve_at_tick {
                let outcome = pending.resolve(current_price, self.base_rate_up.value());
                outcomes.push(outcome);
            } else {
                unresolved.push_back(pending);
            }
        }
        self.pending_predictions = unresolved;

        // Update base rate
        let price_went_up = current_price > 0.0; // simplified; real impl compares to reference
        self.base_rate_up.update(if price_went_up { 1.0 } else { 0.0 });

        outcomes
    }

    /// Full Theta tick: resolve predictions, Hebbian update, budget reallocation,
    /// speciation check.
    pub fn theta_tick(&mut self, current_price: f64, daimon_modulator: f64) {
        // Phase 1: Resolve predictions
        let outcomes = self.resolve_predictions(current_price);

        // Phase 2: Update fitness metrics per signal
        for outcome in &outcomes {
            if let Some(signal) = self.signals.get_mut(&outcome.signal_id) {
                signal.fitness.accuracy.update(
                    if outcome.was_correct { 1.0 } else { 0.0 }
                );
                signal.fitness.information_gain.update(outcome.information_gain);
                signal.fitness.profit_contribution.update(
                    if outcome.was_correct { outcome.confidence } else { 0.0 }
                );

                // Per-context accuracy for speciation detection
                signal.fitness.context_accuracy
                    .entry(outcome.context)
                    .or_insert_with(ContextAccuracy::new)
                    .record(outcome.was_correct);
            }
        }

        // Phase 3: Hebbian weight update
        self.hebbian_update(&outcomes, daimon_modulator);

        // Phase 4: Recompute fitness scores and reallocate budgets
        self.recompute_fitness();
        self.reallocate_budgets();

        // Phase 5: Speciation check
        let speciation_candidates: Vec<SignalId> = self.signals.keys().copied().collect();
        for sid in speciation_candidates {
            self.speciation_check(sid);
        }
    }

    /// Hebbian/anti-Hebbian weight update for all signals with resolved predictions.
    fn hebbian_update(&mut self, outcomes: &[PredictionOutcome], daimon_modulator: f64) {
        let eta = self.config.learning_rate * daimon_modulator;

        for outcome in outcomes {
            if let Some(signal) = self.signals.get_mut(&outcome.signal_id) {
                let activation = outcome.confidence; // signal activation = confidence
                let reward = if outcome.was_correct { 1.0 } else { -1.0 };
                let delta = eta * activation * reward;

                // Update weight for the specific context where this prediction was made
                if let Some(w) = signal.context_weights.get_mut(&outcome.context) {
                    *w = (*w + delta).max(0.001); // floor at 0.001 to prevent total extinction
                }

                // Normalize weights to sum to 1.0
                let total: f64 = signal.context_weights.values().sum();
                if total > 0.0 {
                    for w in signal.context_weights.values_mut() {
                        *w /= total;
                    }
                }
            }
        }
    }

    /// Recompute composite fitness scores for all signals.
    fn recompute_fitness(&mut self) {
        // Find max cost for normalization
        let max_cost = self.signals.values()
            .map(|s| s.compute_cost_ns)
            .fold(0.0_f64, f64::max)
            .max(1.0); // prevent division by zero

        for signal in self.signals.values_mut() {
            let normalized_cost = signal.compute_cost_ns / max_cost;
            signal.fitness.recompute(&self.config.fitness_weights, normalized_cost);
        }
    }

    /// Reallocate compute budget using discrete-time replicator dynamics.
    fn reallocate_budgets(&mut self) {
        if self.signals.is_empty() {
            return;
        }

        // Compute mean fitness weighted by budget share
        let w_bar: f64 = self.signals.values()
            .map(|s| s.fitness.budget_share * s.fitness.fitness_score)
            .sum::<f64>()
            / self.signals.values()
                .map(|s| s.fitness.budget_share)
                .sum::<f64>()
                .max(1e-10);

        // Apply replicator equation
        let sp = self.config.selection_pressure;
        for signal in self.signals.values_mut() {
            let delta = sp * (signal.fitness.fitness_score - w_bar);
            signal.fitness.budget_share *= 1.0 + delta;
            signal.fitness.budget_share = signal.fitness.budget_share.max(0.0);
        }

        // Renormalize to sum to 1.0
        let total: f64 = self.signals.values()
            .map(|s| s.fitness.budget_share)
            .sum();
        if total > 0.0 {
            for signal in self.signals.values_mut() {
                signal.fitness.budget_share /= total;
            }
        }

        // Kill signals below extinction threshold
        let to_kill: Vec<SignalId> = self.signals.iter()
            .filter(|(_, s)| s.fitness.budget_share < self.config.extinction_threshold)
            .filter(|(_, s)| s.total_evaluations > 20) // grace period for new signals
            .map(|(id, _)| *id)
            .collect();

        for sid in to_kill {
            self.kill_signal(sid);
        }
    }
}
```

### Speciation

```rust
impl SignalMetabolism {
    /// Check if a signal should speciate based on context-dependent
    /// performance divergence.
    ///
    /// Returns the two child signal IDs if speciation occurred.
    fn speciation_check(&mut self, signal_id: SignalId) -> Option<(SignalId, SignalId)> {
        let signal = self.signals.get(&signal_id)?;

        // Guard: maximum generation depth
        if signal.generation >= self.config.max_generation {
            return None;
        }

        // Find the pair of contexts with maximum accuracy divergence,
        // but only if both have enough evaluations.
        let min_evals = self.config.min_evaluations_for_speciation;
        let mut best_divergence = 0.0_f64;
        let mut best_pair: Option<(DeFiContext, DeFiContext)> = None;

        let contexts: Vec<(DeFiContext, f64)> = signal.fitness.context_accuracy.iter()
            .filter(|(_, ca)| ca.total >= min_evals)
            .map(|(ctx, ca)| (*ctx, ca.accuracy()))
            .collect();

        for i in 0..contexts.len() {
            for j in (i + 1)..contexts.len() {
                let divergence = (contexts[i].1 - contexts[j].1).abs();
                if divergence > best_divergence {
                    best_divergence = divergence;
                    best_pair = Some((contexts[i].0, contexts[j].0));
                }
            }
        }

        let (ctx_high, ctx_low) = best_pair?;
        if best_divergence < self.config.speciation_threshold {
            return None;
        }

        // Fork. The parent dies; two children inherit.
        let parent = self.signals.get(&signal_id)?;
        let parent_gen = parent.generation;
        let parent_budget = parent.fitness.budget_share;
        let evaluator_a = parent.evaluator.clone_box();
        let evaluator_b = parent.evaluator.clone_box();
        let parent_hv = parent.signal_hv;

        // Determine which context had higher accuracy
        let acc_high = parent.fitness.context_accuracy
            .get(&ctx_high)
            .map(|ca| ca.accuracy())
            .unwrap_or(0.5);
        let acc_low = parent.fitness.context_accuracy
            .get(&ctx_low)
            .map(|ca| ca.accuracy())
            .unwrap_or(0.5);
        let (strong_ctx, weak_ctx) = if acc_high >= acc_low {
            (ctx_high, ctx_low)
        } else {
            (ctx_low, ctx_high)
        };

        // Kill parent
        self.kill_signal(signal_id);

        // Birth child A: specialized for the strong context
        let child_a_id = self.birth_specialized_signal(
            evaluator_a, Some(signal_id), parent_gen + 1,
            parent_hv, strong_ctx, parent_budget / 2.0,
        );

        // Birth child B: specialized for the weak context
        let child_b_id = self.birth_specialized_signal(
            evaluator_b, Some(signal_id), parent_gen + 1,
            parent_hv, weak_ctx, parent_budget / 2.0,
        );

        Some((child_a_id, child_b_id))
    }

    /// Birth a context-specialized signal.
    fn birth_specialized_signal(
        &mut self,
        evaluator: Box<dyn SignalEvaluator>,
        parent: Option<SignalId>,
        generation: u32,
        signal_hv: [u64; 160],
        primary_context: DeFiContext,
        budget_share: f64,
    ) -> SignalId {
        let id = SignalId(self.next_id.fetch_add(1, Ordering::Relaxed));

        let mut signal = LiveSignal::new(
            id, evaluator, parent, generation, self.current_tick, signal_hv,
        );

        // Concentrate context weights on the primary context
        let num_contexts = DeFiContext::all().len() as f64;
        let primary_weight = 0.7;
        let residual_weight = 0.3 / (num_contexts - 1.0);
        for (&ctx, w) in signal.context_weights.iter_mut() {
            if ctx == primary_context {
                *w = primary_weight;
            } else {
                *w = residual_weight;
            }
        }

        signal.fitness.budget_share = budget_share;
        self.signals.insert(id, signal);
        self.stats.signals_born += 1;
        id
    }
}
```

### Dream integration

```rust
impl SignalMetabolism {
    /// NREM dream: consolidate fitness histories.
    ///
    /// Blends short-term EWMA (alpha=0.05, ~20-tick horizon) with a
    /// longer-horizon snapshot to identify reliably fit signals vs.
    /// recently-spiking signals. Reliable signals get a stability bonus.
    pub fn dream_nrem(&mut self) {
        for signal in self.signals.values_mut() {
            let short_term = signal.fitness.accuracy.value();
            let total_evals = signal.total_evaluations;

            // Stability measure: signals with high accuracy AND many evaluations
            // are more reliable than signals with high accuracy and few evaluations.
            let maturity = (total_evals as f64 / 200.0).min(1.0); // saturates at 200 evals
            let stability_bonus = maturity * 0.05; // up to 5% bonus

            // Blend into fitness score
            signal.fitness.fitness_score =
                (signal.fitness.fitness_score + stability_bonus).min(1.0);

            // Decay per-context accuracy counts to prevent unbounded growth.
            // Keep the EWMA but halve the raw counts.
            for ca in signal.fitness.context_accuracy.values_mut() {
                ca.correct /= 2;
                ca.total /= 2;
            }

            // Log consolidation for the Grimoire
            let _ = (short_term, maturity); // production: emit GrimoireEntry
        }
    }

    /// REM dream: mutate a fraction of the population to explore
    /// the fitness landscape.
    ///
    /// Creates mutant clones of above-median-fitness signals with
    /// perturbed parameters. Mutation strength is inversely proportional
    /// to the parent's fitness: fit parents produce mild mutations,
    /// unfit parents produce wild ones.
    pub fn dream_rem(&mut self, rng: &mut impl rand::Rng) {
        let mc = &self.config.mutation_config;

        // Collect signals sorted by fitness
        let mut by_fitness: Vec<(SignalId, f64)> = self.signals.iter()
            .map(|(id, s)| (*id, s.fitness.fitness_score))
            .collect();
        by_fitness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if by_fitness.is_empty() {
            return;
        }

        // Median fitness
        let median_idx = by_fitness.len() / 2;
        let median_fitness = by_fitness[median_idx].1;

        // Select above-median signals for mutation
        let candidates: Vec<SignalId> = by_fitness.iter()
            .filter(|(_, f)| *f >= median_fitness)
            .map(|(id, _)| *id)
            .collect();

        let num_to_mutate = ((candidates.len() as f64 * mc.mutation_fraction) as usize)
            .min(mc.max_mutations_per_dream)
            .min(candidates.len());

        // Randomly select which candidates to mutate
        let mut selected: Vec<SignalId> = candidates;
        // Fisher-Yates partial shuffle
        for i in 0..num_to_mutate.min(selected.len()) {
            let j = i + (rng.next_u64() as usize % (selected.len() - i));
            selected.swap(i, j);
        }
        selected.truncate(num_to_mutate);

        // Create mutants
        let mut births: Vec<(Box<dyn SignalEvaluator>, Option<SignalId>, [u64; 160])> = Vec::new();
        for sid in selected {
            if let Some(parent) = self.signals.get(&sid) {
                let mutation_strength = mc.base_mutation_strength * (1.0 - parent.fitness.fitness_score);
                let mutant_evaluator = parent.evaluator.mutate(rng, mutation_strength);
                births.push((mutant_evaluator, Some(sid), parent.signal_hv));
            }
        }

        for (evaluator, parent, hv) in births {
            self.birth_signal(evaluator, parent, hv);
        }
    }
}
```

### Delta tick: pruning and statistics

```rust
impl SignalMetabolism {
    /// Delta tick: prune dead signals, clean tombstones, compute population stats.
    pub fn delta_tick(&mut self) {
        // Clean old tombstones (older than 10 minutes)
        let cutoff = Instant::now() - std::time::Duration::from_secs(600);
        self.tombstones.retain(|_, ts| *ts > cutoff);

        // Prune pending predictions that are too old (horizon exceeded by 10x)
        let tick = self.current_tick;
        self.pending_predictions.retain(|p| {
            tick < p.resolve_at_tick + (p.resolve_at_tick - p.created_at_tick) * 10
        });

        // Compute population statistics
        self.compute_stats();
    }

    fn compute_stats(&mut self) {
        let n = self.signals.len();
        if n == 0 {
            self.stats.mean_fitness = 0.0;
            self.stats.fitness_variance = 0.0;
            self.stats.species_count = 0;
            self.stats.hhi = 0.0;
            return;
        }

        let fitnesses: Vec<f64> = self.signals.values()
            .map(|s| s.fitness.fitness_score)
            .collect();

        let mean = fitnesses.iter().sum::<f64>() / n as f64;
        let variance = fitnesses.iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f64>() / n as f64;

        // HHI: sum of squared budget shares. 1/N = uniform, 1.0 = monopoly.
        let hhi: f64 = self.signals.values()
            .map(|s| s.fitness.budget_share.powi(2))
            .sum();

        // Count distinct species (unique evaluator names)
        let species: std::collections::HashSet<&str> = self.signals.values()
            .map(|s| s.name.as_str())
            .collect();

        self.stats.mean_fitness = mean;
        self.stats.fitness_variance = variance;
        self.stats.species_count = species.len();
        self.stats.total_evaluations = self.signals.values()
            .map(|s| s.total_evaluations)
            .sum();
        self.stats.hhi = hhi;
    }
}
```

### Death testament

```rust
/// What a dying Golem passes to its successor about its signal population.
/// Extracted via `death_testament()` and ingested by the successor's
/// Grimoire at half-confidence (CLS-inspired knowledge inheritance).
#[derive(Clone, Debug)]
pub struct SignalTestament {
    /// The fittest signals, ranked by fitness score.
    pub top_signals: Vec<SignalSnapshot>,

    /// Population-level statistics at time of death.
    pub stats: PopulationStats,

    /// Hebbian weight matrix: which signal types work for which contexts.
    /// Aggregated across all signals. This is the most transferable knowledge
    /// because it is independent of specific signal instances.
    pub context_affinity: HashMap<String, HashMap<DeFiContext, f64>>,
}

#[derive(Clone, Debug)]
pub struct SignalSnapshot {
    pub name: String,
    pub generation: u32,
    pub fitness_score: f64,
    pub accuracy: f64,
    pub total_evaluations: u64,
    pub context_weights: HashMap<DeFiContext, f64>,
    pub parameters: HashMap<String, f64>,
}

impl SignalMetabolism {
    /// Extract the signal population's accumulated knowledge for successor
    /// inheritance.
    ///
    /// The successor receives this testament and uses it to seed its initial
    /// signal population. Context affinities are the highest-value inheritance:
    /// they tell the successor "momentum signals work for Swap but not Lending"
    /// without requiring the successor to rediscover this through evolution.
    pub fn death_testament(&self) -> SignalTestament {
        let mut snapshots: Vec<SignalSnapshot> = self.signals.values()
            .map(|s| SignalSnapshot {
                name: s.name.clone(),
                generation: s.generation,
                fitness_score: s.fitness.fitness_score,
                accuracy: s.fitness.accuracy.value(),
                total_evaluations: s.total_evaluations,
                context_weights: s.context_weights.clone(),
                parameters: s.evaluator.parameters(),
            })
            .collect();
        snapshots.sort_by(|a, b| {
            b.fitness_score.partial_cmp(&a.fitness_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Aggregate context affinity across all signals by name
        let mut context_affinity: HashMap<String, HashMap<DeFiContext, f64>> = HashMap::new();
        for signal in self.signals.values() {
            let entry = context_affinity
                .entry(signal.name.clone())
                .or_default();
            for (&ctx, &w) in &signal.context_weights {
                *entry.entry(ctx).or_insert(0.0) += w * signal.fitness.fitness_score;
            }
        }
        // Normalize each signal type's affinity to sum to 1.0
        for affinities in context_affinity.values_mut() {
            let total: f64 = affinities.values().sum();
            if total > 0.0 {
                for v in affinities.values_mut() {
                    *v /= total;
                }
            }
        }

        SignalTestament {
            top_signals: snapshots,
            stats: self.stats.clone(),
            context_affinity,
        }
    }
}
```

### Concrete signal evaluators

Four concrete evaluators demonstrate the system. Each implements the `SignalEvaluator` trait and has mutable parameters that the REM mutation system can perturb.

```rust
/// Momentum signal: predicts price continuation based on recent direction.
///
/// Parameters:
/// - lookback: number of ticks to compute momentum over (default: 14)
/// - threshold: minimum momentum magnitude to generate a prediction (default: 0.001)
#[derive(Clone, Debug)]
pub struct MomentumSignal {
    lookback: usize,
    threshold: f64,
}

impl MomentumSignal {
    pub fn new(lookback: usize, threshold: f64) -> Self {
        Self { lookback, threshold }
    }
}

impl SignalEvaluator for MomentumSignal {
    fn evaluate(&self, obs: &DeFiObservation) -> SignalOutput {
        if obs.price_history.len() < self.lookback {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let recent = obs.price;
        let past = obs.price_history[self.lookback - 1];
        if past == 0.0 {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let momentum = (recent - past) / past;
        if momentum.abs() < self.threshold {
            return SignalOutput { prediction: 0.0, confidence: 0.1, horizon_ticks: 1 };
        }

        let confidence = (momentum.abs() / (self.threshold * 10.0)).min(1.0);
        SignalOutput {
            prediction: momentum,
            confidence,
            horizon_ticks: (self.lookback as u32 / 2).max(1),
        }
    }

    fn name(&self) -> &str { "momentum" }

    fn clone_box(&self) -> Box<dyn SignalEvaluator> {
        Box::new(self.clone())
    }

    fn mutate(&self, rng: &mut impl rand::Rng, strength: f64) -> Box<dyn SignalEvaluator> {
        let lookback_delta = (rng.next_u32() % 5) as i32 - 2; // -2 to +2
        let new_lookback = ((self.lookback as i32 + lookback_delta).max(3)) as usize;

        let threshold_factor = 1.0 + (rng.next_u32() as f64 / u32::MAX as f64 - 0.5) * strength;
        let new_threshold = (self.threshold * threshold_factor).max(0.0001);

        Box::new(MomentumSignal::new(new_lookback, new_threshold))
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("lookback".into(), self.lookback as f64);
        p.insert("threshold".into(), self.threshold);
        p
    }
}

/// Mean-reversion signal: predicts price will return toward a rolling mean.
///
/// Parameters:
/// - window: rolling mean window size (default: 20)
/// - std_dev_threshold: distance from mean in std devs to trigger (default: 2.0)
#[derive(Clone, Debug)]
pub struct MeanReversionSignal {
    window: usize,
    std_dev_threshold: f64,
}

impl MeanReversionSignal {
    pub fn new(window: usize, std_dev_threshold: f64) -> Self {
        Self { window, std_dev_threshold }
    }
}

impl SignalEvaluator for MeanReversionSignal {
    fn evaluate(&self, obs: &DeFiObservation) -> SignalOutput {
        if obs.price_history.len() < self.window {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let prices = &obs.price_history[..self.window];
        let mean: f64 = prices.iter().sum::<f64>() / self.window as f64;
        let variance: f64 = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / self.window as f64;
        let std_dev = variance.sqrt();

        if std_dev < 1e-12 {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let z_score = (obs.price - mean) / std_dev;
        if z_score.abs() < self.std_dev_threshold {
            return SignalOutput { prediction: 0.0, confidence: 0.1, horizon_ticks: 1 };
        }

        // Predict reversion toward the mean
        let prediction = -z_score * std_dev * 0.5; // expect partial reversion
        let confidence = ((z_score.abs() - self.std_dev_threshold) / 2.0).min(1.0);

        SignalOutput {
            prediction,
            confidence,
            horizon_ticks: (self.window as u32 / 4).max(1),
        }
    }

    fn name(&self) -> &str { "mean_reversion" }

    fn clone_box(&self) -> Box<dyn SignalEvaluator> {
        Box::new(self.clone())
    }

    fn mutate(&self, rng: &mut impl rand::Rng, strength: f64) -> Box<dyn SignalEvaluator> {
        let window_delta = (rng.next_u32() % 7) as i32 - 3;
        let new_window = ((self.window as i32 + window_delta).max(5)) as usize;

        let factor = 1.0 + (rng.next_u32() as f64 / u32::MAX as f64 - 0.5) * strength;
        let new_threshold = (self.std_dev_threshold * factor).max(0.5);

        Box::new(MeanReversionSignal::new(new_window, new_threshold))
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("window".into(), self.window as f64);
        p.insert("std_dev_threshold".into(), self.std_dev_threshold);
        p
    }
}

/// Volatility breakout signal: detects when price breaks out of a
/// low-volatility consolidation pattern.
///
/// Parameters:
/// - consolidation_window: ticks to measure baseline volatility (default: 30)
/// - breakout_multiplier: how many times baseline vol constitutes a breakout (default: 2.5)
#[derive(Clone, Debug)]
pub struct VolatilityBreakoutSignal {
    consolidation_window: usize,
    breakout_multiplier: f64,
}

impl VolatilityBreakoutSignal {
    pub fn new(consolidation_window: usize, breakout_multiplier: f64) -> Self {
        Self { consolidation_window, breakout_multiplier }
    }
}

impl SignalEvaluator for VolatilityBreakoutSignal {
    fn evaluate(&self, obs: &DeFiObservation) -> SignalOutput {
        if obs.price_history.len() < self.consolidation_window + 1 {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        // Compute baseline volatility over the consolidation window
        let window = &obs.price_history[1..=self.consolidation_window];
        let returns: Vec<f64> = window.windows(2)
            .filter_map(|w| {
                if w[1] != 0.0 { Some((w[0] - w[1]) / w[1]) } else { None }
            })
            .collect();

        if returns.is_empty() {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let baseline_vol: f64 = (returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64)
            .sqrt();

        if baseline_vol < 1e-12 {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        // Current return
        let current_return = if obs.price_history[0] != 0.0 {
            (obs.price - obs.price_history[0]) / obs.price_history[0]
        } else {
            0.0
        };

        let vol_ratio = current_return.abs() / baseline_vol;
        if vol_ratio < self.breakout_multiplier {
            return SignalOutput { prediction: 0.0, confidence: 0.1, horizon_ticks: 1 };
        }

        // Breakout detected. Predict continuation in the breakout direction.
        let confidence = ((vol_ratio - self.breakout_multiplier) / 3.0).min(1.0);
        SignalOutput {
            prediction: current_return.signum() * baseline_vol * self.breakout_multiplier,
            confidence,
            horizon_ticks: 5,
        }
    }

    fn name(&self) -> &str { "volatility_breakout" }

    fn clone_box(&self) -> Box<dyn SignalEvaluator> {
        Box::new(self.clone())
    }

    fn mutate(&self, rng: &mut impl rand::Rng, strength: f64) -> Box<dyn SignalEvaluator> {
        let win_delta = (rng.next_u32() % 11) as i32 - 5;
        let new_window = ((self.consolidation_window as i32 + win_delta).max(10)) as usize;

        let factor = 1.0 + (rng.next_u32() as f64 / u32::MAX as f64 - 0.5) * strength;
        let new_mult = (self.breakout_multiplier * factor).max(1.2);

        Box::new(VolatilityBreakoutSignal::new(new_window, new_mult))
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("consolidation_window".into(), self.consolidation_window as f64);
        p.insert("breakout_multiplier".into(), self.breakout_multiplier);
        p
    }
}

/// Volume profile signal: detects anomalous volume relative to historical norms.
///
/// High volume with price movement = confirmation.
/// High volume without price movement = absorption (potential reversal).
///
/// Parameters:
/// - lookback: volume history window (default: 20)
/// - volume_threshold: z-score threshold for "anomalous" volume (default: 1.5)
#[derive(Clone, Debug)]
pub struct VolumeProfileSignal {
    lookback: usize,
    volume_threshold: f64,
}

impl VolumeProfileSignal {
    pub fn new(lookback: usize, volume_threshold: f64) -> Self {
        Self { lookback, volume_threshold }
    }
}

impl SignalEvaluator for VolumeProfileSignal {
    fn evaluate(&self, obs: &DeFiObservation) -> SignalOutput {
        if obs.volume_history.len() < self.lookback {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let volumes = &obs.volume_history[..self.lookback];
        let mean_vol: f64 = volumes.iter().sum::<f64>() / self.lookback as f64;
        let vol_std: f64 = (volumes.iter()
            .map(|v| (v - mean_vol).powi(2))
            .sum::<f64>() / self.lookback as f64)
            .sqrt();

        if vol_std < 1e-12 || volumes.is_empty() {
            return SignalOutput { prediction: 0.0, confidence: 0.0, horizon_ticks: 1 };
        }

        let current_vol = obs.volume_history[0];
        let vol_z = (current_vol - mean_vol) / vol_std;

        if vol_z < self.volume_threshold {
            // Normal volume, no signal
            return SignalOutput { prediction: 0.0, confidence: 0.1, horizon_ticks: 1 };
        }

        // Anomalous volume detected. Check price movement.
        let price_move = if obs.price_history.len() >= 2 && obs.price_history[1] != 0.0 {
            (obs.price - obs.price_history[1]) / obs.price_history[1]
        } else {
            0.0
        };

        if price_move.abs() > 0.001 {
            // High volume + price movement = trend confirmation
            let confidence = ((vol_z - self.volume_threshold) / 3.0).min(0.8);
            SignalOutput {
                prediction: price_move.signum() * vol_z * 0.01,
                confidence,
                horizon_ticks: 3,
            }
        } else {
            // High volume + no price movement = absorption, predict reversal
            // (But which direction? Use recent trend.)
            let recent_trend = if obs.price_history.len() >= 5 {
                let oldest = obs.price_history[4];
                if oldest != 0.0 { (obs.price - oldest) / oldest } else { 0.0 }
            } else {
                0.0
            };

            let confidence = ((vol_z - self.volume_threshold) / 4.0).min(0.6);
            SignalOutput {
                prediction: -recent_trend.signum() * vol_z * 0.005,
                confidence,
                horizon_ticks: 5,
            }
        }
    }

    fn name(&self) -> &str { "volume_profile" }

    fn clone_box(&self) -> Box<dyn SignalEvaluator> {
        Box::new(self.clone())
    }

    fn mutate(&self, rng: &mut impl rand::Rng, strength: f64) -> Box<dyn SignalEvaluator> {
        let lb_delta = (rng.next_u32() % 7) as i32 - 3;
        let new_lookback = ((self.lookback as i32 + lb_delta).max(5)) as usize;

        let factor = 1.0 + (rng.next_u32() as f64 / u32::MAX as f64 - 0.5) * strength;
        let new_threshold = (self.volume_threshold * factor).max(0.5);

        Box::new(VolumeProfileSignal::new(new_lookback, new_threshold))
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("lookback".into(), self.lookback as f64);
        p.insert("volume_threshold".into(), self.volume_threshold);
        p
    }
}
```

---

## Subsystem interactions

### Attention auction

The metabolism's output drives the Oracle's bidding strategy in the VCG attention auction. At each Theta tick, the Oracle computes its valuation for each context slot based on aggregate signal fitness:

```rust
/// Compute Oracle's VCG bid value for observing a given DeFi context.
///
/// Higher values mean the Oracle will bid more aggressively for this
/// context slot, increasing the Golem's attention to that protocol type.
pub fn oracle_bid_value(
    metabolism: &SignalMetabolism,
    context: DeFiContext,
) -> f64 {
    metabolism.signals.values()
        .filter(|s| s.context_weight(context) > 0.1)
        .map(|s| s.context_weight(context) * s.fitness.fitness_score)
        .sum()
}
```

When the signal population discovers that momentum signals predict well for Swap contexts, the Oracle's Swap bid increases. More Swap observation slots win in the auction. More Swap data feeds back to the momentum signals. The feedback loop amplifies what works.

But it self-corrects too. If momentum signals start underperforming for Swap (because the market regime shifted), their fitness drops, the Oracle's Swap bid decreases, and attention shifts to contexts where other signals are performing better. The time constant for this rebalancing is the Theta tick interval (30-120 seconds) multiplied by the EWMA decay (roughly 20 ticks to significantly move the average). In practice, the attention reallocation takes 10-40 minutes to respond to a regime change. Faster response would require either a shorter Theta tick or a more aggressive EWMA alpha, both of which trade stability for responsiveness.

### Mortality engine

The mortality engine applies three death clocks to Golems: economic (capital), epistemic (accuracy), and stochastic (age). The signal metabolism interacts with all three.

**Economic pressure.** When the Golem's capital depletes, the mortality engine signals economic stress. The metabolism responds by increasing selection pressure (faster culling of underperformers) and reducing the initial budget share for new signals (fewer births). This is metabolic austerity: the Golem stops investing in exploration and focuses on exploiting its best signals.

```rust
pub fn apply_economic_pressure(&mut self, stress_level: f64) {
    // stress_level in [0, 1]: 0 = healthy, 1 = near death
    let base_sp = self.config.selection_pressure;
    self.config.selection_pressure = base_sp * (1.0 + stress_level * 3.0);
    self.config.initial_budget_share *= 1.0 - stress_level * 0.8;
    self.config.mutation_config.max_mutations_per_dream =
        ((1.0 - stress_level) * 5.0) as usize;
}
```

**Epistemic pressure.** The epistemic death clock tracks the Golem's overall prediction accuracy. The signal metabolism is the primary contributor to this metric. If aggregate signal accuracy drops below the epistemic threshold for too long, the Golem dies. The metabolism's mean fitness is the leading indicator: when mean fitness drops, the epistemic clock accelerates.

**Stochastic pressure.** As the Golem ages, its stochastic death probability increases. Old Golems respond by extracting death testaments more frequently and seeding successor Golems with their signal knowledge. The metabolism's `death_testament()` method is the extraction point.

### Dreams

**NREM consolidation.** The dream system calls `dream_nrem()` on the metabolism during the NREM phase of a Delta dream. This consolidates short-term fitness fluctuations into longer-term stability assessments. Signals that have been consistently fit over many Delta cycles receive a stability bonus. Signals that spiked recently but lack long-term records do not. The effect: NREM protects proven signals from being killed during transient market disruptions.

**REM exploration.** `dream_rem()` creates mutant signal clones during the REM phase. The mutation mechanism is the system's escape from local optima. Without REM, the signal population converges to whatever works now and cannot adapt when conditions change. REM is expensive (it creates new signals that consume budget and may not survive), but the cost is justified by the long-term adaptability it provides.

The NREM/REM alternation mirrors biological sleep architecture. NREM consolidates (exploitation). REM explores (exploration). The ratio between them shifts with the Golem's maturity: young Golems dream more REM (high exploration), old Golems dream more NREM (high exploitation). The Golem's age parameter in the dream scheduler controls this ratio.

### Grimoire

Signal fitness data persists in the Grimoire as semantic memory. The Grimoire's CLS consolidation system treats signal-context associations the same way it treats any other learned knowledge: episodic records (individual prediction outcomes) are consolidated into semantic entries (signal X works for context Y with accuracy Z) during Delta ticks. The semantic entries decay via Ebbinghaus curves unless reinforced by new episodic evidence.

When a Golem dies, its death testament includes the accumulated signal knowledge. A successor Golem ingests this testament through the Grimoire at half-confidence (the CLS-standard confidence discount for inherited knowledge). The successor starts with an informed prior about which signal types work for which contexts, rather than learning from scratch.

### Daimon (emotional modulation)

The Daimon subsystem generates emotional states in response to the Golem's performance. Fear, calm, curiosity, and frustration all modulate the signal metabolism through the `daimon_modulator` parameter passed to `theta_tick()`.

- Fear (after losses): learning rate doubles. The Golem rapidly adjusts its signal weights away from configurations that caused the loss. This is fast but noisy adaptation.
- Calm (stable performance): learning rate halves. The Golem makes small, stable adjustments. Good for fine-tuning in a stable regime, bad for adapting to sudden regime changes.
- Curiosity (novel patterns detected): increases the REM dream mutation count. The Golem tries more signal variants.
- Frustration (repeated prediction failures): triggers aggressive pruning. Signals with accuracy below 0.4 are killed regardless of budget share.

### CorticalState integration

The signal metabolism writes aggregated signal state to the CorticalState's 32-slot perception surface. The surface does not carry individual signal outputs (there may be 30+ active signals, and the surface only has 32 atomic slots). Instead, the metabolism writes aggregate quantities:

- Slot allocation: 4 slots for signal metadata (mean fitness, population count, HHI, speciation rate)
- Slot allocation: up to 15 slots for per-context aggregate signal strength (one per DeFiContext, the sum of context_weight * fitness for all signals in that context)
- Remaining slots: available for other subsystems

The per-context aggregate signal strength is what the attention auction's Oracle bidder reads. It does not read individual signal fitness; it reads the aggregated strength for each context. This indirection keeps the Oracle's bid computation at O(15) rather than O(N_signals) per auction round.

```rust
/// Write signal metabolism state to the CorticalState perception surface.
pub fn write_to_cortical_state(
    metabolism: &SignalMetabolism,
    cortical: &CorticalState,
) {
    // Aggregate per-context signal strength
    for &ctx in DeFiContext::all() {
        let strength: f64 = metabolism.signals.values()
            .map(|s| s.context_weight(ctx) * s.fitness.fitness_score)
            .sum();
        cortical.write_signal_strength(ctx, strength);
    }

    // Population metadata
    cortical.write_f32_atomic(SLOT_MEAN_FITNESS, metabolism.stats.mean_fitness as f32);
    cortical.write_u16_atomic(SLOT_SIGNAL_COUNT, metabolism.signals.len() as u16);
    cortical.write_f32_atomic(SLOT_HHI, metabolism.stats.hhi as f32);
}
```

### HDC integration (Doc 1)

Each signal's identity is encoded as a 10,240-bit BSC hypervector using the role-filler scheme from Doc 1. The encoding captures the signal's type, parameters, and lineage:

```
signal_hv = bind(R_type, HV_momentum)
          XOR bind(R_lookback, quantize(14))
          XOR bind(R_threshold, quantize(0.001))
          XOR bind(R_parent, parent_hv)
```

Signal similarity queries run at ~10ns via POPCNT on the hypervector codebook. The metabolism uses these queries for two purposes:

1. **Deduplication.** Before birthing a new signal, check if a signal with similarity > 0.85 already exists. If so, skip the birth.
2. **Successor seeding.** When processing a death testament, the successor Golem matches testament signal descriptions against its own codebook to find the closest existing signals, then adjusts their weights rather than creating new instances.

---

## DeFi primitive coverage

The signal metabolism does not hard-code which signals work for which primitives. It discovers these associations through Hebbian learning. But the discovery has consistent patterns across Golem lifetimes, and the death testament mechanism transfers these patterns to successors. Here is what the system typically converges on.

**Swap (directional trading).** Momentum signals dominate. The system discovers that short-lookback momentum (5-10 ticks) predicts well for high-volume token pairs, while longer lookback (20-30 ticks) works for lower-volume pairs. Mean-reversion signals speciate into swap-specific variants that trigger on larger z-scores (3.0+ standard deviations) than their generic counterparts. Volume profile signals provide confirmation but rarely lead.

**Liquidity provision.** The metabolism evolves a distinct signal ecology. Impermanent loss prediction signals (price volatility relative to fee accrual) emerge as the dominant species. These are not part of the initial signal set; they emerge through REM mutations of volatility breakout signals. Specifically, the system discovers that a volatility breakout signal, when evaluated in LP context where the "price" input is the LP position's value (not the underlying token price), predicts rebalancing opportunities. The speciation mechanism forks the generic volatility signal into an LP-specific variant.

**Lending.** Utilization rate signals dominate. The system discovers oscillation patterns in lending pool utilization: utilization spikes when borrowers draw down, then decays as rates increase and borrowing demand subsides. Mean-reversion signals speciate into lending-specific variants that track utilization rate z-scores rather than price z-scores. The Hebbian weight for `MeanReversionSignal -> Lending` context grows to 0.8+ within 50 Delta cycles.

**Vaults.** Share price dynamics matter. The system discovers that vault share prices follow step functions (flat between harvests, jump at harvest) and evolves signals that detect the harvest rhythm. Momentum signals are useless here (the price does not trend between harvests, it sits flat); the system kills them for vault context within 20 Theta ticks. Volume profile signals survive because high vault deposit/withdrawal volume precedes share price moves.

Vaults also produce an emergent signal type through REM mutation: a "harvest detector" that evolves from a volatility breakout signal with its consolidation window extended to match the vault's harvest cadence (typically 24-72 hours, or 5,000-15,000 Gamma ticks). The breakout signal, when applied to vault share price data, learns that "breakout from zero volatility" is the harvest event. The Hebbian system rewards this signal heavily in vault context, and it speciates away from the generic breakout population. This is an example of a signal that no human would think to design for TA purposes but that the evolutionary process discovers through blind exploration.

**Perpetual swaps.** Funding rate signals are the primary evolved species. Mean-reversion signals discover that perpetual swap funding rates are strongly mean-reverting (extreme positive funding reverts toward zero, extreme negative funding reverts toward zero). The metabolism speciates the generic mean-reversion signal into a perps-specific variant within 30 Delta cycles. The speciation threshold is crossed because funding rate mean-reversion has 0.72 accuracy in PerpetualSwap context but only 0.45 in Swap context.

**Options.** Implied volatility surface curvature signals emerge. The system discovers that changes in the implied volatility smile's shape (steepening, flattening, skew shift) predict option repricing. These are specialized descendants of the volatility breakout signal, mutated through REM to track second derivatives of volatility rather than first derivatives of price.

**Emerging primitives.** For intent-based protocols, the system discovers solver competition patterns: when multiple solvers bid on the same intent, the bid spread predicts execution quality. For restaking protocols, the system tracks AVS delegation concentration as a risk signal. For prediction markets, the system discovers that volume-weighted price momentum in the prediction token predicts resolution direction. These discoveries happen autonomously through the same Hebbian + economic selection mechanism that governs all other primitives.

### Context-specific observation adapters

The `DeFiObservation` struct carries generic price/volume fields, but different primitives populate those fields differently. A context adapter translates protocol-specific state into the generic observation format so that signal evaluators can operate uniformly.

```rust
/// Adapts protocol-specific state into a generic DeFiObservation.
///
/// Each DeFi primitive has its own notion of "price" and "volume."
/// For a swap position, price is the token price and volume is trade volume.
/// For an LP position, price is the position's net value and volume is
/// the pool's total swap volume. For lending, price maps to the utilization
/// rate and volume to borrow/repay transaction count.
///
/// The adapter allows the same SignalEvaluator implementations to work
/// across all primitives. The Hebbian learning system discovers which
/// evaluators work for which adapted observation formats.
pub trait ContextAdapter: Send + Sync {
    fn adapt(&self, raw: &ProtocolSnapshot) -> DeFiObservation;
    fn context(&self) -> DeFiContext;
}

/// Swap adapter: price = token price, volume = DEX trade volume.
pub struct SwapAdapter;

impl ContextAdapter for SwapAdapter {
    fn adapt(&self, raw: &ProtocolSnapshot) -> DeFiObservation {
        DeFiObservation {
            tick: raw.tick,
            timestamp_ms: raw.timestamp_ms,
            context: DeFiContext::Swap,
            price: raw.token_price,
            price_history: raw.token_price_history.clone(),
            volume_history: raw.trade_volume_history.clone(),
            utilization_rate: None,
            liquidity_depth: None,
            funding_rate: None,
            implied_volatility: None,
            share_price: None,
            gas_price_gwei: raw.gas_price_gwei,
        }
    }

    fn context(&self) -> DeFiContext { DeFiContext::Swap }
}

/// LP adapter: price = position net value (accounting for IL),
/// volume = pool swap volume (drives fee accrual).
pub struct LPAdapter {
    pub tick_lower: i32,
    pub tick_upper: i32,
}

impl ContextAdapter for LPAdapter {
    fn adapt(&self, raw: &ProtocolSnapshot) -> DeFiObservation {
        // Position value = liquidity * f(price, tick_lower, tick_upper) - IL
        let position_value = self.compute_position_value(raw);
        let position_value_history: Vec<f64> = raw.position_value_history
            .clone()
            .unwrap_or_default();

        DeFiObservation {
            tick: raw.tick,
            timestamp_ms: raw.timestamp_ms,
            context: DeFiContext::LiquidityProvision,
            price: position_value,
            price_history: position_value_history,
            volume_history: raw.pool_volume_history.clone().unwrap_or_default(),
            utilization_rate: None,
            liquidity_depth: raw.liquidity_at_tick,
            funding_rate: None,
            implied_volatility: None,
            share_price: None,
            gas_price_gwei: raw.gas_price_gwei,
        }
    }

    fn context(&self) -> DeFiContext { DeFiContext::LiquidityProvision }
}

impl LPAdapter {
    fn compute_position_value(&self, raw: &ProtocolSnapshot) -> f64 {
        // Simplified concentrated liquidity position value.
        // Real implementation calls Uniswap v3 math library.
        let sqrt_price = raw.token_price.sqrt();
        let sqrt_lower = (1.0001_f64).powi(self.tick_lower / 2);
        let sqrt_upper = (1.0001_f64).powi(self.tick_upper / 2);

        if sqrt_price <= sqrt_lower {
            // All in token1
            raw.liquidity.unwrap_or(0.0) * (1.0 / sqrt_lower - 1.0 / sqrt_upper)
        } else if sqrt_price >= sqrt_upper {
            // All in token0
            raw.liquidity.unwrap_or(0.0) * (sqrt_upper - sqrt_lower)
        } else {
            // In range: mix of both tokens
            let amount0 = raw.liquidity.unwrap_or(0.0) * (1.0 / sqrt_price - 1.0 / sqrt_upper);
            let amount1 = raw.liquidity.unwrap_or(0.0) * (sqrt_price - sqrt_lower);
            amount0 * raw.token_price + amount1
        }
    }
}

/// Lending adapter: price = utilization rate, volume = borrow/repay count.
pub struct LendingAdapter;

impl ContextAdapter for LendingAdapter {
    fn adapt(&self, raw: &ProtocolSnapshot) -> DeFiObservation {
        DeFiObservation {
            tick: raw.tick,
            timestamp_ms: raw.timestamp_ms,
            context: DeFiContext::Lending,
            price: raw.utilization_rate.unwrap_or(0.0),
            price_history: raw.utilization_rate_history.clone().unwrap_or_default(),
            volume_history: raw.borrow_repay_count_history.clone().unwrap_or_default(),
            utilization_rate: raw.utilization_rate,
            liquidity_depth: None,
            funding_rate: None,
            implied_volatility: None,
            share_price: None,
            gas_price_gwei: raw.gas_price_gwei,
        }
    }

    fn context(&self) -> DeFiContext { DeFiContext::Lending }
}

/// Perps adapter: price = funding rate, volume = open interest change.
pub struct PerpsAdapter;

impl ContextAdapter for PerpsAdapter {
    fn adapt(&self, raw: &ProtocolSnapshot) -> DeFiObservation {
        DeFiObservation {
            tick: raw.tick,
            timestamp_ms: raw.timestamp_ms,
            context: DeFiContext::PerpetualSwap,
            price: raw.funding_rate.unwrap_or(0.0),
            price_history: raw.funding_rate_history.clone().unwrap_or_default(),
            volume_history: raw.oi_change_history.clone().unwrap_or_default(),
            utilization_rate: None,
            liquidity_depth: None,
            funding_rate: raw.funding_rate,
            implied_volatility: None,
            share_price: None,
            gas_price_gwei: raw.gas_price_gwei,
        }
    }

    fn context(&self) -> DeFiContext { DeFiContext::PerpetualSwap }
}

/// Raw protocol state snapshot, before context adaptation.
/// Populated by bardo-protocol-state at each Gamma tick.
pub struct ProtocolSnapshot {
    pub tick: u64,
    pub timestamp_ms: u64,
    pub gas_price_gwei: f64,

    // Token-level
    pub token_price: f64,
    pub token_price_history: Vec<f64>,
    pub trade_volume_history: Vec<f64>,

    // LP-specific
    pub position_value_history: Option<Vec<f64>>,
    pub pool_volume_history: Option<Vec<f64>>,
    pub liquidity_at_tick: Option<f64>,
    pub liquidity: Option<f64>,

    // Lending-specific
    pub utilization_rate: Option<f64>,
    pub utilization_rate_history: Option<Vec<f64>>,
    pub borrow_repay_count_history: Option<Vec<f64>>,

    // Perps-specific
    pub funding_rate: Option<f64>,
    pub funding_rate_history: Option<Vec<f64>>,
    pub oi_change_history: Option<Vec<f64>>,
}
```

This adaptation layer is what allows a `MeanReversionSignal` designed for price z-scores to accidentally work on utilization rate z-scores. The signal does not know it is operating on utilization data. The Hebbian learning system discovers the association through prediction outcomes. When the mean-reversion signal produces correct predictions in Lending context (because utilization rates are mean-reverting), its Lending context weight increases, and eventually the signal speciates into a Lending-specific variant. The generic evaluator code never changes. The discovery is purely through fitness feedback.

### Default signal set and bootstrap

A new Golem starts with a default signal population. Without a death testament from a predecessor, this default set is the starting point for evolution.

```rust
impl SignalMetabolism {
    /// Create a default signal population for a new Golem with no predecessor.
    ///
    /// Seeds the population with a mix of signal types at varying parameters.
    /// The Hebbian/replicator system will reshape this population within
    /// 20-50 Delta cycles. Starting with parameter diversity gives evolution
    /// more raw material to work with.
    pub fn seed_default_population(&mut self) {
        let zero_hv = [0u64; 160]; // placeholder; real impl generates random HVs

        // Momentum signals at three lookback settings
        for lookback in [7, 14, 28] {
            let eval = Box::new(MomentumSignal::new(lookback, 0.001));
            self.birth_signal(eval, None, zero_hv);
        }

        // Mean-reversion at two sensitivity levels
        for (window, threshold) in [(20, 2.0), (50, 1.5)] {
            let eval = Box::new(MeanReversionSignal::new(window, threshold));
            self.birth_signal(eval, None, zero_hv);
        }

        // Volatility breakout at two sensitivities
        for (window, mult) in [(20, 2.0), (40, 3.0)] {
            let eval = Box::new(VolatilityBreakoutSignal::new(window, mult));
            self.birth_signal(eval, None, zero_hv);
        }

        // Volume profile at two thresholds
        for (lookback, threshold) in [(15, 1.5), (30, 2.0)] {
            let eval = Box::new(VolumeProfileSignal::new(lookback, threshold));
            self.birth_signal(eval, None, zero_hv);
        }

        // Equalize budget shares after seeding
        let n = self.signals.len() as f64;
        if n > 0.0 {
            let equal_share = 1.0 / n;
            for signal in self.signals.values_mut() {
                signal.fitness.budget_share = equal_share;
            }
        }
    }

    /// Seed from a predecessor's death testament.
    ///
    /// Uses the testament's context affinity map to set initial Hebbian
    /// weights, giving the successor an informed starting position.
    /// Accuracy estimates from the testament are applied at half-confidence
    /// (the CLS inheritance discount).
    pub fn seed_from_testament(
        &mut self,
        testament: &SignalTestament,
    ) {
        // First, seed the default population
        self.seed_default_population();

        // Then adjust context weights based on testament affinities.
        // This is the "inherited knowledge" path.
        for signal in self.signals.values_mut() {
            if let Some(affinities) = testament.context_affinity.get(&signal.name) {
                for (&ctx, &affinity) in affinities {
                    if let Some(w) = signal.context_weights.get_mut(&ctx) {
                        // Blend: 50% inherited affinity, 50% uniform prior.
                        // The 50% discount is the CLS half-confidence rule.
                        let uniform = 1.0 / DeFiContext::all().len() as f64;
                        *w = 0.5 * affinity + 0.5 * uniform;
                    }
                }

                // Renormalize
                let total: f64 = signal.context_weights.values().sum();
                if total > 0.0 {
                    for w in signal.context_weights.values_mut() {
                        *w /= total;
                    }
                }
            }
        }

        // Apply half-confidence accuracy priors from top testament signals.
        for snapshot in &testament.top_signals {
            // Find signals of the same type and apply accuracy prior
            for signal in self.signals.values_mut() {
                if signal.name == snapshot.name {
                    let half_accuracy = 0.5 * snapshot.accuracy + 0.5 * 0.5; // blend toward neutral
                    signal.fitness.accuracy.reset_with_value(half_accuracy);
                    break; // one match per testament entry
                }
            }
        }
    }
}
```

The bootstrap sequence determines how quickly a Golem becomes effective. A Golem with a predecessor's testament skips the initial exploratory phase where signals have uniform context weights and random budget allocation. The testament tells it "momentum works for Swap, mean-reversion works for Lending" and the Golem can start exploiting these associations immediately while still exploring (via REM) for improvements.

Without a testament, the Golem takes 20-50 Delta cycles (roughly 16-40 hours at default clock rates) to evolve a well-adapted signal population from the default seed. With a testament, it reaches 90% of the predecessor's terminal fitness within 10 Delta cycles (roughly 8 hours). This acceleration compounds across Golem generations: each successor starts from a higher baseline than its predecessor started from, because the testament accumulates refinements across generations.

---

## Cybernetic feedback loop

The metabolism operates as a cybernetic control system with two nested feedback loops.

**Inner loop (Gamma timescale).** Signal predicts -> outcome resolves -> accuracy updates. This is a simple error-correction loop. The time constant is the prediction horizon (1-10 Gamma ticks, or 5-150 seconds). The inner loop adjusts individual signal accuracy estimates.

**Outer loop (Theta/Delta timescale).** Aggregate fitness changes -> budget reallocation -> signal population composition shifts -> aggregate prediction quality changes. This is the evolutionary loop. The time constant is 20-100 Theta ticks (10-200 minutes), determined by the EWMA decay rate and selection pressure. The outer loop adjusts which signals exist and how much compute they receive.

The two loops interact. The inner loop provides the fitness signal that the outer loop acts on. The outer loop changes which signals the inner loop evaluates. This is a classic hierarchical control architecture, analogous to the fast-reflex and slow-planning loops in motor control systems (Todorov & Jordan, 2002).

**Stability analysis.** The system has one known instability mode: if selection pressure is too high and fitness variance is too low, the population collapses to a single dominant signal type (HHI approaches 1.0). This monoculture is fragile: a market regime change that invalidates the dominant signal type causes simultaneous failure of all signals. The Delta tick statistics monitor HHI. If HHI exceeds 0.5, the metabolism reduces selection pressure and increases REM mutation count to restore diversity:

```rust
pub fn diversity_check(&mut self) {
    if self.stats.hhi > 0.5 {
        // Population too concentrated. Reduce selection pressure
        // and increase exploration.
        self.config.selection_pressure *= 0.5;
        self.config.mutation_config.max_mutations_per_dream += 2;
    } else if self.stats.hhi < 0.1 && self.config.selection_pressure < 0.2 {
        // Population well-diversified. Restore normal selection pressure.
        self.config.selection_pressure = 0.1_f64.max(self.config.selection_pressure * 1.2);
    }
}
```

**Cross-generational loop (death/birth timescale).** Death testament extraction -> successor seeding -> accelerated convergence -> improved testament extraction. This is the Lamarckian loop: acquired characteristics (learned signal-context affinities) are inherited. The time constant spans Golem lifetimes, which range from hours (aggressive economic mortality) to days (conservative mortality). This loop is what makes the system improve across generations, not just within a single Golem's life.

```
Inner loop (seconds):
  Signal -> Predict -> Resolve -> Update accuracy -> Signal
          ^                                          |
          |     Outer loop (minutes):                |
          |       Fitness -> Budget realloc ->       |
          +---- Population shift -> New signals -----+
                      ^                    |
                      |  Cross-gen loop:   |
                      |  Testament -> Seed |
                      +--------------------+
```

The three loops create a hierarchy of adaptation rates. Fast perturbations (a single bad prediction) are absorbed by the inner loop without disturbing the population. Sustained regime changes (trending market becomes mean-reverting) are absorbed by the outer loop through budget reallocation and speciation. Structural changes in the DeFi ecosystem (new protocol types, new primitives) are absorbed by the cross-generational loop as testaments accumulate knowledge about new contexts.

**Stability analysis.** The system has one known instability mode: if selection pressure is too high and fitness variance is too low, the population collapses to a single dominant signal type (HHI approaches 1.0). This monoculture is fragile: a market regime change that invalidates the dominant signal type causes simultaneous failure of all signals. The Delta tick statistics monitor HHI. If HHI exceeds 0.5, the metabolism reduces selection pressure and increases REM mutation count to restore diversity:

```rust
pub fn diversity_check(&mut self) {
    if self.stats.hhi > 0.5 {
        // Population too concentrated. Reduce selection pressure
        // and increase exploration.
        self.config.selection_pressure *= 0.5;
        self.config.mutation_config.max_mutations_per_dream += 2;
    } else if self.stats.hhi < 0.1 && self.config.selection_pressure < 0.2 {
        // Population well-diversified. Restore normal selection pressure.
        self.config.selection_pressure = 0.1_f64.max(self.config.selection_pressure * 1.2);
    }
}
```

A second instability mode is oscillation: two signal types alternately dominate as the market oscillates between regimes. Signal A thrives in regime 1, grows its budget, and starves signal B. The market shifts to regime 2, signal B recovers, and now A starves. The oscillation wastes adaptation energy on cycling rather than diversifying. The fix is the NREM stability bonus: signals that maintain above-average fitness across regime changes receive a bonus that prevents starvation during temporary unfavorable regimes. A signal with 200 evaluations and 0.6 accuracy across multiple regimes is more useful than a signal with 20 evaluations and 0.8 accuracy in a single regime.

**Convergence guarantee.** Fisher's fundamental theorem guarantees that mean fitness increases monotonically when the fitness landscape is static. Since the DeFi fitness landscape is not static, the system does not converge to a global optimum. Instead, it tracks the moving landscape with a lag proportional to the outer loop time constant. The Red Queen dynamic means the system never stops evolving, which is the desired behavior: a "converged" signal set is a dead signal set, unable to adapt to the next regime change.

The system's long-run trajectory is more like a random walk on a treadmill than a climb to a summit. Mean fitness fluctuates around a level determined by the predictability of the underlying markets. In highly predictable markets (strong trends, clear mean-reversion), mean fitness is high. In unpredictable markets (random walks, structural breaks), mean fitness is low. The metabolism cannot create predictability where none exists; it can only find and exploit whatever predictability the market offers.

---

## Evaluation protocol [SPEC]

The signal metabolism makes testable predictions. Here is how to evaluate them.

### Signal population diversity

**Metric.** Herfindahl-Hirschman Index (HHI) of budget share across signals, tracked over time.

**Expected behavior.** HHI starts high (few initial signals, each with large budget share), decreases as speciation creates new species, and stabilizes in the range 0.05-0.3. It should not collapse to near-1.0 (monoculture) or remain near-0 (no selection pressure at all).

**Test.** Initialize with 10 signals of four types (momentum, mean-reversion, volatility breakout, volume profile). Run 1,000 simulated Theta ticks with synthetic price data drawn from a regime-switching model (trending 50% of the time, mean-reverting 50%). Record HHI at each Delta tick. Pass condition: HHI remains in [0.03, 0.6] for 95% of the simulation.

### Prediction accuracy improvement

**Metric.** Rolling 100-tick accuracy of the top-5 signals by budget share, compared against a baseline of the same initial signal set without the metabolism (static parameters, no speciation, no budget reallocation).

**Expected behavior.** The metabolized population's top-5 accuracy exceeds the static baseline within 50 Delta cycles and maintains a gap of at least 5 percentage points.

**Test.** Run paired simulations (metabolized vs. static) on identical synthetic data. Compare rolling accuracy curves. The metabolized system should show faster adaptation to regime changes: accuracy drops after a regime switch but recovers within 20 Theta ticks, while the static baseline remains at reduced accuracy until the regime happens to favor the static signal set again.

### Speciation rate

**Metric.** Number of speciation events per Delta cycle, and lineage depth distribution.

**Expected behavior.** Speciation events occur at a rate of 0.5-2 per Delta cycle during the initial exploration phase, declining to 0.1-0.5 once the population stabilizes. Lineage depth should not hit the max_generation cap for more than 10% of the population (if it does, the cap is too low or speciation is too aggressive).

### Computational efficiency

**Metric.** Fraction of the Gamma tick compute budget actually used, and fraction of budget allocated to signals with accuracy > 0.5.

**Expected behavior.** Budget utilization stays above 80% (the system uses the compute it has). Budget allocated to accurate signals increases over time, from ~50% (random) to >75% (evolved).

### Cross-generation knowledge transfer

**Metric.** Compare a successor Golem's time-to-adapted-signal-set with and without a death testament from its predecessor.

**Expected behavior.** With testament inheritance, the successor reaches 90% of the predecessor's final mean fitness within 10 Delta cycles. Without inheritance, it takes 50+ Delta cycles.

### Regime-switching robustness

**Metric.** Mean fitness recovery time after a regime switch, measured as the number of Theta ticks from the fitness trough (immediately after the switch) to 90% recovery of the pre-switch fitness level.

**Expected behavior.** Recovery time decreases over successive regime switches as the population builds a diverse portfolio of regime-adapted signals. The first regime switch should recover within 50 Theta ticks. By the fifth switch, recovery should be under 20 Theta ticks, because the speciated signal population retains context-specific variants for each regime.

**Test.** Generate synthetic data with regime switches every 200 Theta ticks, alternating between trending (drift = +0.001 per tick, noise std = 0.005) and mean-reverting (target = 100.0, reversion speed = 0.02, noise std = 0.01). Record the metabolism's mean fitness at each Theta tick. Compute recovery time for each regime transition. Pass condition: fifth recovery time is less than 50% of first recovery time.

### Daimon interaction test

**Metric.** Compare adaptation speed with and without Daimon emotional modulation. The `daimon_modulator` is fixed at 1.0 in the control group and varies dynamically in the test group.

**Expected behavior.** The Daimon-modulated system recovers from losses faster (fear-accelerated learning) and maintains more stable signal weights during calm periods. The cost: increased noise in weight updates during fear episodes. Net effect should be positive over 500+ Theta ticks.

**Test.** Inject three loss events (predictions that were confidently wrong) at ticks 100, 300, and 500. Measure how quickly the originating signals' context weights adjust after each loss. The Daimon-modulated system should show weight adjustment within 3 Theta ticks of the loss; the unmodulated system should take 8-10 ticks.

---

## References

1. Hebb, D. O. (1949). *The Organization of Behavior*. Wiley.

2. Oja, E. (1982). A simplified neuron model as a principal component analyzer. *Journal of Mathematical Biology*, 15(3), 267-273.

3. Fisher, R. A. (1930). *The Genetical Theory of Natural Selection*. Clarendon Press.

4. Taylor, P. D., & Jonker, L. B. (1978). Evolutionarily stable strategies and game dynamics. *Mathematical Biosciences*, 40(1-2), 145-156.

5. Holland, J. H. (1975). *Adaptation in Natural and Artificial Systems*. University of Michigan Press.

6. Maynard Smith, J. (1982). *Evolution and the Theory of Games*. Cambridge University Press.

7. Van Valen, L. (1973). A new evolutionary law. *Evolutionary Theory*, 1, 1-30.

8. Wright, S. (1932). The roles of mutation, inbreeding, crossbreeding, and selection in evolution. *Proceedings of the Sixth International Congress of Genetics*, 1, 356-366.

9. McGaugh, J. L. (2004). The amygdala modulates the consolidation of memories of emotionally arousing experiences. *Annual Review of Neuroscience*, 27, 1-28.

10. Todorov, E., & Jordan, M. I. (2002). Optimal feedback control as a theory of motor coordination. *Nature Neuroscience*, 5(11), 1226-1235.

11. Kanerva, P. (2009). Hyperdimensional computing: An introduction to computing in distributed representation with high-dimensional random vectors. *Cognitive Computation*, 1(2), 139-159.

12. Kleyko, D., Rachkovskij, D. A., Osipov, E., & Rahimi, A. (2022). A survey on hyperdimensional computing: Theory, architecture, and applications. *ACM Computing Surveys*.

13. Borbely, A. A. (1982). A two-process model of sleep regulation. *Human Neurobiology*, 1(3), 195-204.

14. Tononi, G., & Cirelli, C. (2006). Sleep function and synaptic homeostasis. *Sleep Medicine Reviews*, 10(1), 49-62.

15. Wilder, J. W. (1978). *New Concepts in Technical Trading Systems*. Trend Research.

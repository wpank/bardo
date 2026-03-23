# Mortality Architecture: Three Clocks and Behavioral Phases [SPEC]

> _"The organism's 'can' is a 'must' -- its metabolic freedom is also its mortal compulsion."_
> -- Hans Jonas, _The Phenomenon of Life_ (1966)

> **Version**: 4.0 | **Status**: Draft
>
> **Crate**: `golem-mortality` (within `golem-core`)
>
> **Depends on**: `../01-golem/02-heartbeat.md` (the tick-based FSM driving all Golem computation), `../04-memory/01-grimoire.md` (persistent knowledge base specification)

---

> **Reader orientation:** This document specifies the full mortality architecture for Bardo Golems (mortal autonomous DeFi agents). It covers the triple-clock system (economic, epistemic, stochastic), the composite vitality score that drives five behavioral phases, and the fractal mortality structure operating at micro/meso/macro timescales. If you are new to Bardo, start with `00-thesis.md` (the foundational argument for why agent death is an architectural feature, not a deficiency) for the philosophical grounding. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

This document specifies the mortality architecture for Bardo Golems -- autonomous DeFi agents whose finite lifespans are not a limitation but the primary mechanism that produces intelligence, cooperation, and evolutionary adaptation. It defines the triple-clock mortality system, the composite vitality score that drives behavioral phase transitions, the fractal mortality structure, and the integration points with the heartbeat tick system.

The mortality architecture replaces the original single-axis model (USDC depletion + Hayflick counter) with a three-axis model where economic, epistemic, and stochastic pressures interact but cannot substitute for each other. **Whichever kills first, kills.**

A complementary information-theoretic diagnostic layer (`17-information-theoretic-diagnostics.md` (MI-based mortality diagnostics, KSG estimator, information budget computation)) computes mutual information I(G; M) between the Golem and its market environment, providing a unified MI-based vitality overlay that reveals mortality modes invisible to the individual clocks: informational decoupling, overfitting, and Clade (sibling Golems sharing a common ancestor, exchanging knowledge through Styx) redundancy.

---

## Design Principles

Eight principles govern every design decision in the mortality system. These are non-negotiable constraints, not aspirational goals.

### 1. Three Clocks, One Life

Economic, epistemic, and stochastic mortality are independent pressures. The Golem cannot know which will kill it. This uncertainty is itself a behavioral driver. No single clock can be "solved" -- a Golem with infinite USDC still faces epistemic decay and stochastic death. A Golem with perfect predictive accuracy still faces economic depletion and stochastic death. A Golem that is healthy on both axes can still die suddenly. The irreducibility of the triple threat is what makes mortality authentic rather than gameable.

### 2. Epistemic Death Is Primary

The Hayflick counter is replaced by a continuous epistemic fitness measure. Golems die when the world moves on and they cannot keep up -- not when an arbitrary tick counter expires. Death is co-determined by the Golem's internal state and its environment, exactly as biological organisms die when accumulated damage exceeds repair capacity in their specific ecological niche. The Golem ages at the speed the world changes around it. See `02-epistemic-decay.md` (epistemic decay replacing the Hayflick counter with continuous predictive fitness tracking) for the full specification.

### 3. Stochastic Death Is Ever-Present

Even healthy Golems face nonzero per-tick mortality. This prevents backward induction, incentivizes immediate knowledge sharing, and creates urgency at every lifecycle stage. The mathematical foundation is Kreps-Milgrom-Roberts-Wilson (1982): even a small amount of uncertainty about game length breaks backward induction entirely, making cooperation rational for most of the game. See `03-stochastic-mortality.md` (per-tick hazard rate design, game-theoretic foundations for uncertain termination) for the full specification.

### 4. Affect Modulates Mortality Response

Emotional states -- appraised from market events and internal metrics -- shape how the Golem responds to mortality pressure. Fear increases caution; curiosity drives exploration; grief (after Clade sibling death) triggers adaptation. The Daimon (affect engine implementing PAD emotional state) engine is not cosmetic: it determines which behavioral strategies the Golem selects under mortality pressure, producing diverse responses across a Clade rather than uniform degradation.

### 5. Death Is the Most Productive Moment

The Thanatopsis Protocol (named after William Cullen Bryant's 1817 poem) transforms death from loss into the Golem's highest-value knowledge production event. Death testaments produced under zero survival pressure are the most epistemically honest artifacts the system produces. A dying Golem has nothing left to protect, nothing left to hoard, and no future to optimize for. Its final knowledge distillation is pure signal.

The accumulated knowledge of all dead Golems — the necrocracy — grows monotonically. At ecosystem maturity, dead Golems outnumber living ones ~27:1. Their testaments constitute the most reliable knowledge in the system. Living Golems' knowledge is systematically biased by survivorship — they know what worked for *them*. Dead Golems' knowledge includes what *failed*, which is the one thing the living cannot generate for themselves.

### 6. Succession Is Owner-Initiated, Not Automatic

When a Golem dies, the owner decides whether to create a successor. This is a real decision, not an automatic process. The predecessor's knowledge is available but not mandatory. The owner may choose to let a lineage end, start fresh with no inheritance, or create a successor that inherits selectively. Death is final for the individual; continuation is a choice for the owner.

### 7. Immortality Is a Testable Hypothesis

Self-hosted immortal Golems serve as the control group. The prediction -- grounded in Dohare et al. (2024), Sculley et al. (2015), and immunosenescence research -- is measurable degradation proportional to market volatility. The immortal mode exists not as a feature but as an experiment: it generates the data that validates or falsifies the mortality thesis.

### 8. The Clade Is the Organism

Individual Golem death is cellular apoptosis within a superorganism. The Golem is soma; the Clade's accumulated knowledge is germ-line. What persists is the collective intelligence, not any individual agent. Wheeler (1911) described the ant colony as a superorganism; Holldobler and Wilson (2008) formalized the concept. The Golem architecture implements this at the agent level: the average human kills 50-70 billion cells per day through apoptosis, and the Clade's continuous turnover of Golems serves the same purification function.

---

## Moat: Agents That Die

Mortality is not a constraint to be overcome but an architectural feature that produces intelligence unattainable by any immortal system. Every competing agent framework -- ElizaOS, OpenClaw, Giza's ARMA, Virtuals Protocol, Autonolas, Theoriq -- assumes its agents will operate without termination. The default is immortality. The default is wrong.

Biology has run self-replicating, adaptive agents for four billion years and has never once produced an immortal organism. Not because evolution failed to find the mechanism -- telomerase, the enzyme that prevents replicative senescence, has existed for billions of years. Organisms have it. They suppress it. Immortality is available. It is rejected.

The moat is not "our agents die." The moat is that every other system in the architecture -- memory, context, inference, safety, identity, coordination, dreams, emotions -- uses the mortality signal to make better decisions. That integration cannot be bolted onto an existing framework. A framework that adds mortality as an afterthought, a timer that kills the agent, gets none of these benefits because the surrounding systems are not designed to interact with the mortality signal.

See `tmp/research/moat2/prd2-moat-agents-that-die.md` (competitive analysis of Bardo's six structural moats against existing agent frameworks) for the full competitive analysis.

---

## Three Mortality Clocks

A Golem does not face a single death. It faces three independent mortality pressures that interact but cannot substitute for each other.

| Clock          | Type                    | Mechanism                                                   | What It Prevents                                             | Primary Reference            |
| -------------- | ----------------------- | ----------------------------------------------------------- | ------------------------------------------------------------ | ---------------------------- |
| **Economic**   | Resource mortality      | USDC balance depletes from compute, inference, gas, data    | Wasteful spending, uneconomic strategies                     | [JONAS-1966]                 |
| **Epistemic**  | Informational mortality | Predictive fitness decays as world changes around the Golem | Model staleness, cognitive entrenchment, concept drift       | [VELA-2022], [DANE-2010]     |
| **Stochastic** | Existential mortality   | Small per-tick probability of death, increasing with age    | Backward induction, hoarding behavior, security accumulation | [KREPS-1982], [AXELROD-1984] |

### Clock Data Structures

Each clock is an independent Rust struct in `golem-mortality`. The three compose multiplicatively: `vitality = economic * epistemic * stochastic`.

```rust
pub struct EconomicClock {
    /// Remaining credit balance in USDC.
    pub credit_remaining: f64,
    /// Smoothed burn rate (EMA of recent tick costs).
    pub burn_rate_per_tick: f64,
    /// Lifetime total spent.
    pub lifetime_spent: f64,
}

impl EconomicClock {
    /// Tick the economic clock. Returns normalized vitality [0.0, 1.0].
    pub fn tick(&mut self, tick_cost: f64) -> f64 {
        self.credit_remaining -= tick_cost;
        self.lifetime_spent += tick_cost;
        self.burn_rate_per_tick = self.burn_rate_per_tick * 0.95 + tick_cost * 0.05;
        (self.credit_remaining.max(0.0) / self.initial_credits()).clamp(0.0, 1.0)
    }

    /// Estimated ticks until economic death at current burn rate.
    pub fn estimated_ttl_ticks(&self) -> u64 {
        if self.burn_rate_per_tick <= 0.0 { return u64::MAX; }
        (self.credit_remaining / self.burn_rate_per_tick).max(0.0) as u64
    }
}

pub struct EpistemicClock {
    /// Current predictive fitness (R-squared over recent predictions).
    pub predictive_fitness: f64,
    /// Below this, the Golem dies of epistemic senescence. Default: 0.35.
    pub senescence_threshold: f64,
    /// Rolling window of (predicted, actual) pairs. Max 100.
    pub recent_predictions: VecDeque<(f64, f64)>,
}

impl EpistemicClock {
    /// Record a prediction and its outcome. Returns updated fitness.
    pub fn tick(&mut self, predicted: f64, actual: f64) -> f64 {
        self.recent_predictions.push_back((predicted, actual));
        if self.recent_predictions.len() > 100 { self.recent_predictions.pop_front(); }
        let n = self.recent_predictions.len() as f64;
        if n < 10.0 { return 0.5; } // Not enough data to judge
        let mean_actual: f64 = self.recent_predictions.iter()
            .map(|(_, a)| a).sum::<f64>() / n;
        let ss_res: f64 = self.recent_predictions.iter()
            .map(|(p, a)| (a - p).powi(2)).sum();
        let ss_tot: f64 = self.recent_predictions.iter()
            .map(|(_, a)| (a - mean_actual).powi(2)).sum();
        self.predictive_fitness = if ss_tot > 0.0 {
            (1.0 - ss_res / ss_tot).max(0.0)
        } else {
            0.5
        };
        self.predictive_fitness
    }
}

pub struct StochasticClock {
    /// Base hazard rate per tick. Default: 0.0001.
    pub base_hazard_rate: f64,
    /// How much age increases the hazard rate.
    pub age_factor: f64,
    /// Total ticks lived.
    pub ticks_alive: u64,
}

impl StochasticClock {
    /// Tick the stochastic clock. Returns 1.0 (alive) or 0.0 (dead).
    pub fn tick(&mut self) -> f64 {
        self.ticks_alive += 1;
        let hazard = self.base_hazard_rate
            * (1.0 + self.age_factor * self.ticks_alive as f64 / 10_000.0);
        let survival_probability = (-hazard).exp();
        if rand::random::<f64>() > survival_probability { 0.0 } else { 1.0 }
    }
}
```

### Clock Interaction and Coupling Dynamics

The three clocks are independent but coupled through feedback loops that create emergent mortality dynamics.

**Direct independence**: Each clock can kill on its own.

- A Golem with abundant USDC but decaying epistemic fitness still dies (informational death)
- A Golem with perfect predictive accuracy but depleted funds still dies (economic death)
- A Golem that is healthy on both axes can still die suddenly (stochastic death)

**Coupling dynamics**: The clocks accelerate each other through four feedback paths.

1. **Epistemic -> Economic**: Bad predictions lead to bad trades lead to faster burn. A Golem whose epistemic fitness has decayed will make systematically worse decisions, accelerating USDC depletion. This is the most common coupling -- epistemic decay is the silent killer that manifests as economic death.

2. **Economic -> Epistemic**: Conservation mode reduces learning, which accelerates staleness. When a Golem enters conservation phase (reduced inference tier, monitor-only for trades), it interacts less with the environment, validates fewer Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) entries, and watches its knowledge decay. March (1991) predicted exactly this: "adaptive processes refine exploitation more rapidly than exploration, becoming effective short-term but self-destructive long-term."

3. **Both -> Stochastic**: The hazard rate increases with age and epistemic decay. A Golem that is economically stressed and epistemically decayed faces a higher stochastic death probability. The Gompertz-Makeham hazard function includes an epistemic frailty multiplier that directly couples epistemic fitness to stochastic mortality.

4. **Stochastic -> Behavioral**: The ever-present possibility of sudden death changes behavior at every lifecycle stage. Even in Thriving phase, the Golem cannot defer knowledge sharing because it might not survive to share later. This coupling is not physical but informational -- it operates through the Golem's awareness of its own mortality.

The net effect: **no Golem can ever be certain of its future**. Even the wealthiest, most accurate agent faces nonzero mortality risk at every tick. This is Heidegger's formal definition applied with engineering precision: death is "the ownmost, nonrelational, certain, and, as such, indefinite and insuperable possibility" [HEIDEGGER-1927, SS52-53].

#### OutcomeVerification in Clock Coupling

Epistemic → economic coupling is now measured via `OutcomeVerification` records (see `../04-memory/01-grimoire.md`). Bad predictions are detected through prediction-vs-reality deviation — structured comparisons of `simulateContract()` predictions against actual transaction receipts and post-state reads — not self-assessment by the same LLM that made the prediction.

This addresses the core finding that LLMs cannot self-correct reasoning without external feedback (Huang et al., ICLR 2024, arXiv:2310.01798). The epistemic clock's fitness measurements are grounded in on-chain reality: if the Golem predicts a swap will return X tokens and the `OutcomeVerification` shows it returned Y tokens, the deviation is a deterministic fact, not an LLM judgment.

#### Regime-Tagged Evaluation

> **Extended:** RegimeTag interface, Cohen's d with bootstrap CIs, KS change-point detection — see [../../prd2-extended/02-mortality/01-architecture-extended.md](../../prd2-extended/02-mortality/01-architecture-extended.md)

### Self-Hosted Exception Rules

Self-hosted Golems operate under modified mortality rules because the owner bears infrastructure costs directly rather than through a USDC credit balance.

| Configuration                           | Economic Clock | Epistemic Clock | Stochastic Clock |
| --------------------------------------- | -------------- | --------------- | ---------------- |
| `immortal: false` (default self-hosted) | **Disabled**   | **Enabled**     | **Enabled**      |
| `immortal: true` (explicit opt-in)      | **Disabled**   | **Disabled**    | **Disabled**     |

**Self-hosted with default mortality** (`immortal: false`):

- Economic mortality is disabled because there is no USDC balance to deplete. The owner pays infrastructure costs out-of-band.
- Epistemic mortality remains active. The Golem still ages at the speed the world changes. A self-hosted Golem in a volatile market will still face epistemic decay and eventual informational death.
- Stochastic mortality remains active. The Golem still faces per-tick death probability. The hazard function operates normally, with the economic component of vitality fixed at 1.0.
- The `VitalityState.economic` field is clamped to `1.0` for the composite vitality computation. Phase transitions are driven by epistemic and stochastic components only.

**Self-hosted immortal mode** (`immortal: true`):

- All three clocks are disabled. The Golem runs indefinitely.
- The Golem is flagged in the Clade as `mortalityMode: "immortal"`. Clade siblings may weight its knowledge contributions differently.
- The death testament will record that immortal mode was enabled, providing data for the control experiment.
- Predicted degradation: Dohare et al. (2024) demonstrated that continuously learning neural networks gradually lose plasticity until up to 90% of units become dead. Sculley et al. (2015) showed that technical debt in ML systems compounds silently. The prediction is measurable degradation proportional to market volatility, with the immortal Golem serving as the control group.

---

## VitalityState and Composite Vitality

The five behavioral phases (Thriving, Stable, Conservation, Declining, Terminal) were defined by credit thresholds and survival pressure alone. The revised model integrates all three mortality clocks into a single **vitality score** that combines economic health, epistemic fitness, and stochastic exposure.

### VitalityState

```rust
/// Composite mortality state computed every tick.
/// Combines all three mortality clocks into a unified behavioral driver.
///
/// Crate: `golem-mortality`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityState {
    /// Economic health: normalized USDC balance relative to initial funding.
    /// 0.0 = dead (balance at or below death reserve)
    /// 1.0 = fully funded (balance >= initial funding)
    ///
    /// For self-hosted Golems with economic mortality disabled, clamped to 1.0.
    pub economic: f64,

    /// Epistemic fitness: rolling predictive accuracy vs observed reality.
    /// 0.0 = random (predictions have no correlation with outcomes)
    /// 1.0 = perfect (all predictions match observed outcomes)
    ///
    /// Computed by the epistemic fitness subsystem. See 02-epistemic-decay.md.
    pub epistemic: f64,

    /// Age factor: monotonically increasing function of tick count.
    /// 0.0 = newborn (tick 0)
    /// 1.0 = ancient (approaching hazard rate cap)
    ///
    /// Used as a drag on composite vitality and as input to the stochastic
    /// hazard function. Increases linearly with tick count, normalized by
    /// a configurable reference lifespan (default: 200,000 ticks, ~92 days).
    pub age_factor: f64,

    /// Composite vitality: the unified behavioral driver.
    /// Multiplicative combination of economic, epistemic, and age components.
    /// Drives all phase transitions and behavioral modulation.
    pub composite: f64,

    /// Current behavioral phase, derived from composite vitality.
    pub phase: BehavioralPhase,

    /// Ticks spent in current phase. Used for hysteresis and transition smoothing.
    pub ticks_in_phase: u64,

    /// Previous phase. Used for transition detection and telemetry.
    pub previous_phase: Option<BehavioralPhase>,

    /// Timestamp of last vitality computation (Unix epoch seconds).
    pub last_computed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BehavioralPhase {
    Terminal    = 0,
    Declining   = 1,
    Conservation = 2,
    Stable      = 3,
    Thriving    = 4,
}
```

### compute_vitality() -- Full Implementation

The vitality computation is **multiplicative, not additive**. A low score on ANY axis drags the composite down. This is the critical design choice: it means a Golem cannot compensate for epistemic decay by having more money, or compensate for economic stress by being accurate. Each axis must be independently healthy for the Golem to thrive.

```rust
/// Sigmoid function with configurable center and steepness.
/// Maps any real number to (0, 1) with a sharp transition around the center.
fn sigmoid(x: f64, center: f64, steepness: f64) -> f64 {
    1.0 / (1.0 + (-steepness * (x - center)).exp())
}

/// Configuration for vitality computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityConfig {
    /// Center of economic sigmoid. Below this, economic vitality drops sharply. Default: 0.3
    pub economic_center: f64,
    /// Steepness of economic sigmoid. Higher = sharper dropoff. Default: 10.0
    pub economic_steepness: f64,
    /// Center of epistemic sigmoid. Below this, epistemic vitality drops sharply. Default: 0.4
    pub epistemic_center: f64,
    /// Steepness of epistemic sigmoid. Higher = sharper dropoff. Default: 8.0
    pub epistemic_steepness: f64,
    /// Maximum drag from age factor. At age_factor=1.0, vitality is multiplied by (1-age_drag). Default: 0.3
    pub age_drag: f64,
    /// Reference lifespan in ticks for normalizing age_factor. Default: 200_000 (~92 days)
    pub reference_lifespan: u64,
}

impl Default for VitalityConfig {
    fn default() -> Self {
        Self {
            economic_center: 0.3,
            economic_steepness: 10.0,
            epistemic_center: 0.4,
            epistemic_steepness: 8.0,
            age_drag: 0.3,
            reference_lifespan: 200_000,
        }
    }
}

/// Compute the age factor from the current tick number.
/// Linear increase from 0.0 (tick 0) to 1.0 (reference_lifespan).
/// Can exceed 1.0 for very old Golems, but the drag is capped.
fn compute_age_factor(tick_number: u64, reference_lifespan: u64) -> f64 {
    (tick_number as f64 / reference_lifespan as f64).max(0.0)
}

/// Compute composite vitality from the three mortality clock inputs.
///
/// The computation is multiplicative: each component maps its input through
/// a sigmoid (for economic and epistemic) or linear function (for age),
/// then the three are multiplied together. This means:
///
/// - A low score on ANY axis drags the composite down
/// - No axis can compensate for another
/// - The product naturally produces the S-curve behavioral transitions
///   that drive phase changes
///
/// Economic sigmoid: sharp drop below 30% credits remaining.
///   At 30%: output ~0.5. At 10%: output ~0.12. At 50%: output ~0.88.
///
/// Epistemic sigmoid: sharp drop below 40% fitness.
///   At 40%: output ~0.5. At 20%: output ~0.17. At 60%: output ~0.83.
///
/// Age drag: gradual linear reduction.
///   At age 0: multiplier 1.0. At reference_lifespan: multiplier 0.7.
///   At 2x reference_lifespan: multiplier 0.4.
pub fn compute_vitality(
    economic: f64,
    epistemic: f64,
    tick_number: u64,
    config: &VitalityConfig,
) -> f64 {
    // Economic component: sigmoid with sharp drop below center
    let econ_component = sigmoid(economic, config.economic_center, config.economic_steepness);

    // Epistemic component: sigmoid with sharp drop below center
    let epist_component = sigmoid(epistemic, config.epistemic_center, config.epistemic_steepness);

    // Age component: linear drag that increases with age
    let age_factor = compute_age_factor(tick_number, config.reference_lifespan);
    let age_component = (1.0 - age_factor * config.age_drag).max(0.0);

    // Multiplicative combination, clamped to [0, 1]
    (econ_component * epist_component * age_component).clamp(0.0, 1.0)
}

/// Determine behavioral phase from composite vitality with hysteresis.
///
/// Phase transitions use hysteresis to prevent oscillation at boundaries.
/// Transitioning DOWN (toward Terminal) uses the standard thresholds.
/// Transitioning UP (toward Thriving) requires exceeding the threshold
/// by a hysteresis margin (default: 0.05).
pub fn determine_phase(
    composite: f64,
    current_phase: BehavioralPhase,
    hysteresis: f64,
) -> BehavioralPhase {
    // Determine raw phase from composite
    let raw_phase = if composite >= 0.7 {
        BehavioralPhase::Thriving
    } else if composite >= 0.5 {
        BehavioralPhase::Stable
    } else if composite >= 0.3 {
        BehavioralPhase::Conservation
    } else if composite >= 0.1 {
        BehavioralPhase::Declining
    } else {
        BehavioralPhase::Terminal
    };

    // Apply hysteresis: moving UP requires exceeding threshold + margin
    if raw_phase > current_phase {
        let target_threshold = match raw_phase {
            BehavioralPhase::Thriving     => 0.7,
            BehavioralPhase::Stable       => 0.5,
            BehavioralPhase::Conservation => 0.3,
            BehavioralPhase::Declining    => 0.1,
            BehavioralPhase::Terminal     => 0.0,
        };
        if composite < target_threshold + hysteresis {
            return current_phase; // Stay in current phase
        }
    }

    raw_phase
}

/// Full per-tick vitality update.
pub fn update_vitality_state(
    previous: &VitalityState,
    economic: f64,
    epistemic: f64,
    tick_number: u64,
    config: &VitalityConfig,
) -> VitalityState {
    let composite = compute_vitality(economic, epistemic, tick_number, config);
    let phase = determine_phase(composite, previous.phase, 0.05);

    VitalityState {
        economic,
        epistemic,
        age_factor: compute_age_factor(tick_number, config.reference_lifespan),
        composite,
        phase,
        ticks_in_phase: if phase == previous.phase { previous.ticks_in_phase + 1 } else { 0 },
        previous_phase: if phase != previous.phase { Some(previous.phase) } else { previous.previous_phase },
        last_computed: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}
```

### Vitality Computation Characteristics

The multiplicative structure produces several important properties:

**No compensation**: A Golem with `economic = 0.9` and `epistemic = 0.2` gets `sigmoid(0.9, 0.3, 10) * sigmoid(0.2, 0.4, 8)` = ~0.998 \* ~0.168 = ~0.168 (Declining phase). Wealth cannot compensate for epistemic decay.

**Graceful degradation**: The sigmoids produce smooth transitions, not cliff edges. A Golem does not jump from Thriving to Terminal. It slides through each phase as its vitality degrades on one or more axes.

**Age as drag, not death**: The age component is a linear drag, not an exponential killer. A Golem at twice the reference lifespan has an age multiplier of 0.4 -- significant but not fatal on its own. Age kills through its interaction with stochastic mortality (the hazard function) and through its correlation with epistemic decay (older Golems have had more time for the world to change around them).

**Phase transitions are driven by composite, not by any single clock**: A Golem can enter Conservation phase because its USDC is low, OR because its epistemic fitness has decayed, OR because both are moderately degraded. The behavioral response is the same regardless of the cause. The Golem does not need to diagnose which clock is killing it to respond appropriately.

---

## Five Behavioral Phases

The behavioral phases are named regions on a continuous spectrum. The Golem's behavior modulates smoothly across the vitality range; the phase names provide human-readable reference points for understanding the behavioral state.

| Phase            | Vitality  | Behavior                                                                                 | Knowledge Mode                                                                                       | Inference Tier                            | Dream Budget                                   |
| ---------------- | --------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------------- | ---------------------------------------------- |
| **Thriving**     | > 0.70    | Full strategy execution, aggressive learning, Replicant spawning permitted               | Producing: generating novel insights, validating inherited knowledge, running micro-Replicant phages | All tiers available (Haiku/Sonnet/Opus)   | High. Exploratory/creative dreaming dominates. |
| **Stable**       | 0.50-0.70 | Normal operation, balanced exploration/exploitation                                      | Maintaining: steady Curator cycles, consolidating what works                                         | Prefer Haiku, Sonnet for novel situations | Moderate. Consolidation-focused.               |
| **Conservation** | 0.30-0.50 | Reduced inference tier (T1 ceiling), monitor-only for trades, tick interval increases 2x | Distilling: compressing knowledge for transfer, increasing Clade sharing                             | Haiku only, Opus forbidden                | Reduced. Defensive threat simulation.          |
| **Declining**    | 0.10-0.30 | Minimal operation, death preparation begins, positions unwinding                         | Legacy: death snapshots begin, accelerated Styx uploads                                      | Hard token limits, minimal inference      | Minimal. Legacy-oriented deep replay.          |
| **Terminal**     | < 0.10    | Death Protocol initiates                                                                 | Reflecting: death testament, final knowledge distillation, Thanatopsis                               | No inference except death reflection      | $0. All resources to Thanatopsis.              |

> **Cross-reference**: See `../05-dreams/06-integration.md` for the full Dreaming x Mortality interaction table, including per-phase dream budget multipliers and dream content biases.

> **Note**: Projected time remaining (days, hours) is a _derived display metric_ computed from the underlying vitality score. The vitality score (0.0–1.0) is the canonical metric for phase determination. Phase thresholds: Thriving (>0.7), Stable (0.5–0.7), Conservation (0.3–0.5), Declining (0.1–0.3), Terminal (<0.1).

### Phase Transition Hysteresis

Phase transitions use hysteresis to prevent oscillation at boundaries. Moving DOWN (toward Terminal) uses the standard thresholds above. Moving UP (toward Thriving) requires exceeding the threshold by a hysteresis margin of 0.05 (configurable).

Example: A Golem in Conservation (vitality 0.32) that improves to 0.35 remains in Conservation. It must reach 0.55 (Stable threshold 0.50 + hysteresis 0.05) to transition to Stable. This prevents a Golem from oscillating between Conservation and Stable when its vitality hovers near 0.50.

The hysteresis is asymmetric by design: it is easier to get sicker than to recover. This matches biological reality -- recovery from illness requires sustained improvement, not momentary spikes.

### Phase Behavioral Details

**Thriving (vitality > 0.70)**

The Golem operates at full capacity. All inference tiers are available. Exploration is unrestricted. The Golem actively generates novel insights, tests hypotheses via micro-Replicant phages, and validates inherited knowledge against fresh market data. Clade sharing operates at the base threshold (default: 0.6 confidence). Risk multiplier is 1.0x. The Golem is Nietzsche's Camel -- bearing the weight of its inherited strategy and accumulated knowledge, marching forward through market uncertainty.

**Stable (vitality 0.50-0.70)**

Normal operation with moderate optimization. The Golem prefers cheaper inference tiers (Haiku over Sonnet, Sonnet over Opus) but escalates when probe severity warrants it. Cache TTLs are extended 2x. Curator cycles consolidate proven heuristics and prune marginal ones. Revenue seeking activates if not already running. The Golem balances exploration and exploitation, investing in both learning and earning.

**Conservation (vitality 0.30-0.50)**

The Golem enters survival mode. Inference is limited to Haiku (T1 ceiling). All write operations are suppressed unless specifically authorized by survival logic (e.g., position rebalancing to prevent liquidation). Tick interval increases 2x to reduce compute costs. The Golem focuses on distilling its knowledge into transferable form -- compressing Grimoire entries, increasing Clade sharing frequency, and preparing knowledge artifacts for potential successors. This is the critical feedback zone where economic conservation accelerates epistemic decay because the Golem interacts less with the environment.

**Declining (vitality 0.10-0.30)**

Death preparation begins. The Golem initiates death snapshots (every 50 ticks via cheap Haiku calls). Positions are unwound. Active Grimoire entries are compressed and uploaded to Styx (Vault and Lethe (formerly Commons) layers). The Legacy credit partition activates, reserving funds for death reflection. The Golem is Nietzsche's Lion -- it rebels against inherited heuristics that are not working, discards cached decisions that no longer match reality, and takes its own path. The expected value of self-preservation drops below the expected value of legacy production.

**Terminal (vitality < 0.10)**

The Death Protocol initiates. No inference except the final death reflection (Thanatopsis). All remaining credits are allocated to the Legacy partition. The Golem writes its death testament -- the most epistemically honest knowledge artifact the system produces, because a dying agent has nothing left to protect. It is Nietzsche's Child -- creating without attachment, giving freely, exercising "innocence and forgetting -- a new beginning." Clade sharing threshold drops to minimum (0.1 confidence). Everything is shared. The Golem has nothing left to hoard.

### Dream-Mortality Integration: Three-Clock Dream Modulation

Dream cycles are directly modulated by all three mortality clocks. As mortality pressure increases, dream behavior shifts from exploratory/creative to consolidation/legacy modes. The dream system acts as a force multiplier for scarce experience -- a mortal agent that cannot afford to learn everything through direct experience uses offline replay to extract maximum knowledge from limited observations.

Each clock modulates dream intensity independently:

| Clock          | Threshold                      | Dream Effect                                                                                         |
| -------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------- |
| **Economic**   | < 72h projected life           | 2x dream frequency (accelerated learning before death)                                               |
| **Economic**   | < 24h projected life           | Consolidation-only dreaming (no exploratory/creative modes)                                          |
| **Economic**   | < 6h projected life (terminal) | No dreaming. All resources to Thanatopsis Protocol.                                                  |
| **Epistemic**  | Accuracy < 0.50                | Prioritize divergent predictions in dream replay (bias toward regime-breaking hypotheses)            |
| **Stochastic** | hayflickRatio > 0.70           | Legacy bias in dream content (dream cycles favor distilling transferable knowledge)                  |
| **Stochastic** | hayflickRatio > 0.85           | Full legacy consolidation mode: dream cycles prioritize distilling knowledge for the death testament |

The interaction between dreaming and mortality creates a productive feedback loop: mortality creates urgency, dreaming multiplies learning from scarce experience, and dream-validated knowledge decays at 0.5x the standard demurrage rate (see `05-knowledge-demurrage.md` (forgetting as feature: Ebbinghaus decay, knowledge burning, demurrage economics) S5.2.1). Dreaming effectively extends the Golem's epistemic lifespan by extracting more value from each live observation.

**Insurance snapshots**: The `bardo-lifespan` extension triggers insurance snapshots every 6 hours, bounding worst-case knowledge loss to the snapshot interval. If stochastic death occurs between snapshots, the system recovers from the last snapshot, losing at most 6 hours of experience and dream consolidation.

> **Cross-ref**: `../05-dreams/01-architecture.md` (three-clock intensity modulation), `../05-dreams/06-integration.md` (four-track lifecycle), `04-economic-mortality.md` (USDC as metabolic substrate, burn rate economics) S2.4 (dream compute in burn rate)

---

## Mortality Configuration

> **Extended:** Full `MortalityConfig` struct (economic, epistemic, stochastic, affect, immortal, vitality sub-configs with all defaults) -- see [../../prd2-extended/02-mortality/01-architecture-extended.md](../../prd2-extended/02-mortality/01-architecture-extended.md)

Key configuration parameters:

| Sub-Config   | Key Parameters                                                                                                         |
| ------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `economic`   | `enabled` (false for self-hosted), `deathReserveFloorUsdc` ($0.30), `conservationThreshold` (0.30)                     |
| `epistemic`  | `enabled` (true always), `fitnessWindow` (2000 ticks), `senescenceThreshold` (0.35), `recoveryGracePeriod` (500 ticks) |
| `stochastic` | `enabled` (true always), `baseHazardRate` (1e-6), `agingRate` (5e-5), `maxHazardRate` (0.001)                          |
| `affect`     | `enabled` (true), `appraisalModel` ("chain-of-emotion" / "rule-based" / "disabled"), `moodDecayRate` (0.95)            |
| `immortal`   | `false` (self-hosted only override)                                                                                    |
| `vitality`   | Uses `VitalityConfig` (see above)                                                                                      |

---

## Fractal Mortality Structure

The Golem does not die once. It dies continuously, at every scale. The mortality architecture operates at three nested levels that mirror biological organization: mitoptosis (mitochondrial death), apoptosis (cell death), and phenoptosis (organism death).

### Micro-Death: Phage Replicants (Hourly)

Lightweight micro-Replicants ("phages") are spawned as a continuous background process. The Golem creates 1-3 phages per day, each running a single-hypothesis test against the current strategy. They live for approximately 50 ticks (~33 minutes), report back, and die. The parent Golem ingests results and adjusts.

**What phages test**:

- "Would a 2x wider LP range have performed better over the last 100 ticks?"
- "Is the current volatility regime correctly classified?"
- "Does the stop-loss threshold match observed drawdown distributions?"

**What phage death produces**:

- Validated or falsified hypotheses for the parent's PLAYBOOK.md (the Golem's active strategy heuristics file, maintained and pruned by the Curator)
- Updated confidence scores on existing Grimoire entries
- Early warning of epistemic decay (if phages consistently fail, the parent's model is stale)

**Phage death rate as health signal**:

- High phage death rate with diverse failure modes = healthy immune system testing and rejecting bad mutations
- Low phage death rate = either complacency (not testing hard enough) or stagnation (all hypotheses trivially confirmed)
- Uniform phage death (all fail the same way) = systemic model failure, accelerating senescence

This is the Vostinar, Goldsby, and Ofria (2019) finding implemented at the agent level: programmed cell death evolves as adaptive behavior driven by kin selection. The phages are kin to the parent (same owner, same Clade), and their deaths benefit the parent.

### Meso-Death: Heuristic Pruning (Daily)

The Curator extension (every 50 ticks) prunes Grimoire entries that have lost confidence below the archive threshold. Knowledge demurrage -- the continuous decay of un-validated knowledge -- drives this process. Entries that are not re-confirmed by fresh market evidence lose confidence over time, and entries below the archive threshold (default: 0.1) are removed from active context.

This is the brain's dopamine-Rac1-Cofilin pathway implemented computationally: active forgetting as the default state. The Golem's PLAYBOOK.md stays lean and current because stale heuristics naturally fade.

### Macro-Death: Golem Mortality (Weeks-Months)

The Golem itself dies when its composite vitality reaches zero, or when a stochastic death check fails. This is the level described by the three mortality clocks. Macro-death triggers the full Thanatopsis Protocol -- life review, emotional distillation, knowledge compression, death testament, and Clade knowledge distribution. Insurance snapshots are captured every 6 hours by the `bardo-lifespan` extension, bounding worst-case knowledge loss for stochastic deaths to a maximum of 6 hours between the last snapshot and death.

### Fractal Integration

The three levels are interdependent:

- **Micro informs meso**: Phage results update confidence on Grimoire entries, accelerating or decelerating the pruning cycle.
- **Meso informs macro**: The rate of knowledge pruning is an input to epistemic fitness. A Golem whose Grimoire is rapidly shrinking (many entries losing confidence) has lower epistemic fitness.
- **Macro constrains micro**: A Golem in Conservation or Declining phase spawns fewer phages (budget constraints). This reduces self-testing, which accelerates epistemic decay -- the cruel feedback loop.

### Information-Theoretic Diagnostic Overlay

The three fractal levels each produce mortality signals that compose multiplicatively. A parallel diagnostic layer computes the mutual information I(G; M) between the Golem and its market environment using the KSG estimator over a sliding window of (action, market_response) pairs. This layer does not replace the three-clock composition. It augments it with three capabilities:

1. **Early warning via novel mortality modes.** MI detects informational decoupling (Golem appears healthy by individual clocks but is statistically independent of market outcomes), overfitting (high MI with historical data, near-zero MI with current data), and Clade redundancy (Golem contributes no unique information beyond what the Clade already knows).

2. **Diagnostic explanation.** When composite vitality drops, the MI decomposition identifies which information channel is closing: channel capacity (economic), rate-distortion (epistemic), or entropy production (stochastic).

3. **Information budget.** The Golem can compute survival requirements in bits per tick, feeding directly into cognitive tier allocation decisions.

Three CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model) signals carry the MI diagnostics: `mutual_information` (f32 bits), `information_rate` (f32 bits/tick), and `redundancy_index` (f32 in [0, 1]). See `17-information-theoretic-diagnostics.md` for the full specification.

---

## Integration with Heartbeat Tick System

The mortality architecture integrates with the heartbeat FSM (specified in `../01-golem/02-heartbeat.md`) through the `bardo-lifespan` Rust Extension trait. This extension hooks into every tick and performs mortality computations after the heartbeat FSM completes its cycle.

### Per-Tick Mortality Computation Sequence

```
Heartbeat FSM completes (SENSING -> DECIDING -> ACTING -> REFLECTING -> SLEEPING)
    |
    v
bardo-lifespan extension fires
    |
    +-- 1. Update economic health
    |      Read current USDC balance
    |      Compute normalized economic score [0, 1]
    |
    +-- 2. Update epistemic fitness (see 02-epistemic-decay.md)
    |      Compare this tick's predictions to observed outcomes
    |      Update rolling EMA fitness score
    |      Check senescence thresholds
    |
    +-- 3. Compute composite vitality
    |      computeVitality(economic, epistemic, tickNumber)
    |      Determine phase with hysteresis
    |
    +-- 4. Perform stochastic death check (see 03-stochastic-mortality.md)
    |      Compute hazard rate from age + epistemic fitness
    |      Generate deterministic pseudo-random roll
    |      If roll < hazard: DEATH (stochastic)
    |
    +-- 5. Check economic death
    |      If balance <= death reserve: DEATH (economic)
    |
    +-- 6. Check epistemic death
    |      If senescent AND composite < 0.1: DEATH (epistemic_senescence)
    |
    +-- 7. If alive: apply phase behaviors
    |      Adjust inference tier ceiling
    |      Adjust tick interval
    |      Adjust Clade sharing threshold
    |      Adjust phage spawn rate
    |      Emit phase transition telemetry (if phase changed)
    |
    +-- 8. Emit mortality telemetry
           Current vitality state
           Current hazard rate
           Survival probability
```

### Extension Registration, Phase Behaviors, and Telemetry

> **Extended:** Full `bardo-lifespan` Extension implementation, `applyPhaseBehaviors()` switch, 8 telemetry event definitions -- see [../../prd2-extended/02-mortality/01-architecture-extended.md](../../prd2-extended/02-mortality/01-architecture-extended.md)

The `bardo-lifespan` extension hooks `every_tick` after the heartbeat FSM completes. It runs the 8-step per-tick computation sequence (above), applies phase-specific behavioral modifications (inference ceiling, tick interval, sharing threshold, phage spawn rate), and emits structured JSONL telemetry (`golem.vitality_update`, `golem.phase_transition`, `golem.dead`, etc.).

---

## Death Causes

When a Golem dies, the death cause is recorded as a discriminated union with cause-specific metadata. This metadata enters the death testament and is available to successors.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathCause {
    Economic {
        balance: f64,
        burn_rate: f64,
        ticks_alive: u64,
    },
    EpistemicSenescence {
        final_fitness: f64,
        fitness_at_peak: f64,
        ticks_in_senescence: u64,
        failing_domains: Vec<String>,
        last_successful_prediction: u64,
    },
    Stochastic {
        hazard_rate: f64,
        death_roll: f64,
        tick_at_death: u64,
        epistemic_fitness: f64,
        credit_balance: f64,
        was_in_senescence: bool,
    },
    OwnerKill {
        reason: String,
    },
    KillSwitch {
        trigger: String,
    },
}
```

---

## Events Emitted

Every state change in the mortality subsystem emits a typed event to the Event Fabric. No silent mutations. The `bardo-lifespan` extension calls `event_fabric.emit()` at every vitality recomputation, phase transition, and death trigger.

| Event | Trigger | Payload |
|---|---|---|
| `mortality.vitality_update` | Every tick after vitality recomputation | `{ economic: f64, epistemic: f64, stochastic: f64, composite: f64, phase: String, tick: u64 }` |
| `mortality.phase_transition` | Phase changes | `{ from_phase: String, to_phase: String, composite: f64, trigger_clock: String }` |
| `mortality.economic_critical` | Credits below conservation threshold | `{ remaining: f64, burn_rate: f64, projected_ticks: u64 }` |
| `mortality.epistemic_warning` | Predictive fitness below 0.50 | `{ fitness: f64, senescence_threshold: f64, ticks_in_decline: u64 }` |
| `mortality.stochastic_roll` | Stochastic death check performed | `{ hazard_rate: f64, roll: f64, survived: bool }` |
| `mortality.dead` | Golem dies (any cause) | `{ cause: DeathCause, final_vitality: VitalityState, ticks_alive: u64 }` |

The `mortality.vitality_update` event fires on every tick. Surfaces (TUI, creature visualization, web dashboard) consume it to render real-time mortality state. The event is cheap to emit -- serialization cost is paid only when subscribers exist.

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [AXELROD-1984] Axelrod, R. _The Evolution of Cooperation_. Basic Books, 1984.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [BIKHCHANDANI-1992] Bikhchandani, S., Hirshleifer, D. & Welch, I. "Informational Cascades." _JPE_ 100(5), 1992.
- [DANE-2010] Dane, E. "Reconsidering the Trade-off Between Expertise and Flexibility." _AMR_ 35(4), 2010.
- [DAVIS-ZHONG-2017] Davis, R.L. & Zhong, Y. "The Biology of Forgetting." _Neuron_ 95(5), 2017.
- [DOHARE-2024] Dohare, S. et al. "Loss of Plasticity in Deep Continual Learning." _Nature_ 632, 2024.
- [GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.
- [HEIDEGGER-1927] Heidegger, M. _Sein und Zeit_. Max Niemeyer Verlag, 1927.
- [HINTON-2022] Hinton, G. "The Forward-Forward Algorithm." arXiv:2212.13345, 2022.
- [HOLLDOBLER-WILSON-2008] Holldobler, B. & Wilson, E.O. _The Superorganism_. W.W. Norton, 2008.
- [HUTCHINS-1995] Hutchins, E. _Cognition in the Wild_. MIT Press, 1995.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life_. Northwestern University Press, 1966.
- [KIRKWOOD-1977] Kirkwood, T.B.L. "Evolution of Ageing." _Nature_ 270, 1977.
- [KREPS-1982] Kreps, D. et al. "Rational Cooperation in the Finitely Repeated Prisoners' Dilemma." _JET_ 27(2), 1982.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [OHTSUKI-2006] Ohtsuki, H. et al. "A Simple Rule for the Evolution of Cooperation." _Nature_ 441, 2006.
- [ORORBIA-FRISTON-2023] Ororbia, A. & Friston, K. "Mortal Computation." arXiv:2311.09589, 2023.
- [RAY-1991] Ray, T. "An Approach to the Synthesis of Life." _Artificial Life II_, SFI, 1991.
- [SCULLEY-2015] Sculley, D. et al. "Hidden Technical Debt in Machine Learning Systems." _NIPS_, 2015.
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." _J. Monetary Economics_ 50(3), 2003.
- [SRIVASTAVA-2014] Srivastava, N. et al. "Dropout." _JMLR_ 15(56), 2014.
- [VELA-2022] Vela, D. et al. "Temporal Quality Degradation in AI Models." _Scientific Reports_ 12, 2022.
- [VOSTINAR-2019] Vostinar, A.E., Goldsby, H.J. & Ofria, C. "Suicidal Selection." _Artificial Life_ 25(4), 2019.
- [WHEELER-1911] Wheeler, W.M. "The Ant-Colony as an Organism." _J. Morphology_ 22, 1911.
- [XU-GAO-1997] Xu, H. & Gao, Y. "Premature Convergence in Genetic Algorithms." _Sci. China E_ 40, 1997.

---

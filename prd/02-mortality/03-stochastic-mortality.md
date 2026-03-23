# Stochastic Mortality: The Ever-Present Risk [SPEC]

> **Version**: 4.0 | **Status**: Draft
>
> **Crate**: `golem-mortality`
>
> **Depends on**: `01-architecture.md` (triple-clock system, vitality computation, fractal structure), `02-epistemic-decay.md` (epistemic decay replacing the Hayflick counter with predictive fitness tracking)

---

> **Reader orientation:** This document specifies the stochastic mortality clock, which ensures no Bardo Golem (mortal autonomous DeFi agent) can ever be certain of its future. Even healthy agents face a small per-tick death probability, creating cooperation incentives from tick 1. The game-theoretic argument is that uncertain termination occupies the sweet spot between known-endpoint systems (which destroy cooperation via backward induction) and immortal systems (which have no urgency). Read `01-architecture.md` (triple-clock system, vitality computation) for how this clock composes with the economic and epistemic clocks. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

This document specifies the stochastic mortality clock -- the third and most philosophically consequential of the three mortality pressures facing every Bardo Golem. Stochastic mortality ensures that no Golem can ever be certain of its future. Even the wealthiest, most accurate, most recently initialized agent faces a nonzero probability of immediate death on every tick. This probability is small for healthy Golems and increases with age and epistemic decay, but it is never zero.

The purpose of stochastic mortality is not to kill Golems. Most Golems will die of epistemic decay or economic depletion long before stochastic mortality becomes statistically significant. The purpose is to change behavior from tick 1. The mere possibility of sudden death -- the Heideggerian "indefinite certainty" -- transforms how the Golem cooperates, shares knowledge, and prioritizes its actions. Stochastic mortality is a mechanism design tool, not a punishment.

---

## Why Uncertainty Matters More Than the Death Itself

### The Backward Induction Problem

The most counterintuitive finding in game theory is that **agents who do not know when they will die cooperate more than agents who do**. The reasoning, formalized by Luce and Raiffa (1957), is precise:

In a game with a **known endpoint** at round N:

1. In round N, defection dominates (there is no future to protect)
2. Both players know this, so in round N-1, defection also dominates (the future is defection)
3. By induction, defection dominates in round N-2, N-3, ... all the way back to round 1
4. **Result**: Rational agents defect from the very first round

This is backward induction -- the logical cascade that destroys cooperation whenever the game's length is common knowledge. It applies directly to agent systems: a Golem that knows it will die at tick 100,000 can compute that sharing knowledge on tick 99,999 has zero future value, therefore tick 99,998, therefore tick 99,997, and so on. The defection cascade may not propagate all the way back to tick 1 in practice (bounded rationality limits the depth of backward induction), but it creates a measurable zone of declining cooperation near the end of life.

An **infinite horizon** (immortal agents) removes the backward induction problem but creates a different pathology: no urgency. If the game never ends, there is always tomorrow. Knowledge sharing can always be deferred. Cooperation is rational but not urgent. The agent has no reason to act now rather than later.

### The Kreps-Milgrom-Roberts-Wilson Theorem

Kreps, Milgrom, Roberts, and Wilson (1982) proved the mathematical foundation for why uncertain termination is optimal. Their theorem demonstrates that **even a small amount of uncertainty about game length breaks backward induction entirely**, making cooperation rational for most of the game [KREPS-1982].

The key insight: if there is even a small probability that the game continues past the "expected" endpoint, the backward induction chain breaks. Round N might not be the last round -- so defection in round N is no longer dominant. If defection in N is not dominant, then the cascade does not propagate backward. Cooperation becomes rational for the vast majority of the game, with defection emerging only when the probability of continuation drops below a threshold determined by the payoff structure.

### Samuelson's Extension

Samuelson (1987) strengthened this result by showing that **private information about remaining rounds** produces equilibrium cooperation in finite games. When each agent has private (noisy) information about when the game might end, the Nash equilibrium involves cooperation for a substantial fraction of the game. The more private the information, the more cooperation emerges.

For Golems, this maps directly: each Golem knows its own hazard rate (it can compute it from its state), but it does not know the random roll for future ticks. Its mortality information is private in the temporal dimension -- it knows the distribution but not the realization. This is precisely the condition Samuelson identified as maximally cooperation-promoting.

### Ohtsuki's Death-Birth Updating

Ohtsuki et al. (2006) demonstrated in _Nature_ that the **order of death and birth matters for cooperation**. In "death-birth" updating -- where death comes first and birth follows -- cooperators can be favored over defectors. In "birth-death" updating -- where birth comes first and death follows -- defectors are always favored. The mathematical condition is:

```
b/c > k
```

Where `b` is the benefit to the recipient, `c` is the cost to the cooperator, and `k` is the average number of neighbors in the interaction network. When death precedes birth, this condition is easier to satisfy because death creates vacancies that cooperators' neighbors can fill [OHTSUKI-2006].

For Clades (sibling Golems sharing a common ancestor, exchanging knowledge through Styx), this is directly applicable. When a Golem dies (death), its owner may create a successor (birth) that inherits from the dead Golem's knowledge. The death-birth ordering means that knowledge sharing (cooperation) is favored: the dying Golem's knowledge benefits its Clade siblings (high `b`, since DeFi knowledge is immediately actionable) at relatively low cost to the dying agent (low `c`, since the Golem is dying anyway).

### Synthesis: The Optimal Mortality Regime

The game-theoretic evidence converges on a precise prescription for agent mortality:

| Mortality Type                  | Cooperation Effect                          | Urgency Effect                              | Combined Assessment                            |
| ------------------------------- | ------------------------------------------- | ------------------------------------------- | ---------------------------------------------- |
| **Known endpoint** (Hayflick)   | Destroys cooperation via backward induction | High urgency near end, low urgency at start | Poor -- cooperation collapses near death       |
| **Infinite horizon** (Immortal) | Cooperation rational but not urgent         | No urgency ever                             | Poor -- knowledge sharing perpetually deferred |
| **Stochastic** (This design)    | Cooperation rational at every point         | Persistent moderate urgency                 | Optimal -- cooperation and urgency coexist     |

Stochastic mortality occupies the optimal middle ground: it maintains the shadow of the future (motivating cooperation) while avoiding both the backward induction catastrophe (that destroys cooperation near known endpoints) and the urgency vacuum (that makes immortal agents perpetual procrastinators).

---

## The Gompertz-Makeham Hazard Function

The per-tick death probability follows a Gompertz-Makeham hazard model -- the same mathematical form that has modeled biological mortality since Benjamin Gompertz's 1825 paper in the _Philosophical Transactions of the Royal Society_. The Gompertz-Makeham model separates mortality into age-independent and age-dependent components, capturing the biological reality that death can come from both background risks (accidents, disease) and intrinsic aging.

### Mathematical Form

```
h(t) = lambda + alpha * exp(beta * t) * epsilon(t)
```

Where:

- `h(t)` = hazard rate at tick `t` (probability of death on tick `t`, given survival to tick `t`)
- `lambda` = baseline hazard rate (age-independent, "hit by a bus" probability -- the Makeham component)
- `alpha` = initial age-dependent hazard coefficient (the Gompertz amplitude)
- `beta` = rate of exponential increase with age (the Gompertz aging rate)
- `epsilon(t)` = epistemic frailty multiplier, a function of the current epistemic fitness score

The Makeham component (`lambda`) ensures that even a newborn, perfectly healthy Golem faces a nonzero death probability. The Gompertz component (`alpha * exp(beta * t)`) produces the exponential increase in mortality with age that characterizes all known biological organisms (and, we argue, should characterize all long-running computational agents). The epistemic frailty multiplier (`epsilon(t)`) couples the stochastic clock to the epistemic clock, ensuring that epistemically decayed Golems face higher stochastic mortality.

### StochasticMortalityState Interface

```rust
/// State tracked for the stochastic mortality clock.
/// Updated every tick by the lifespan extension.
///
/// Crate: `golem-mortality`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticMortalityState {
    /// Current tick number, used as the age proxy.
    /// Monotonically increasing from 0 at creation.
    pub tick_number: u64,

    /// Per-tick hazard rate computed from the Gompertz-Makeham function.
    /// This is the probability of death on this tick, given survival to this tick.
    /// Range: [base_hazard_rate, max_hazard_rate]
    pub current_hazard: f64,

    /// Cumulative survival probability since creation.
    /// Product of (1 - hazard) over all previous ticks.
    /// Monotonically decreasing from 1.0.
    /// Useful for owner communication and telemetry.
    pub survival_probability: f64,

    /// Deterministic pseudo-random seed for this tick's death check.
    /// Derived from keccak256(golem_id ++ tick_number) for reproducibility
    /// and auditability.
    pub death_check_seed: Vec<u8>,

    /// Whether the Golem survived this tick's stochastic check.
    /// False on the tick the Golem dies; true on all others.
    pub survived: bool,

    /// The random roll value for this tick (for telemetry and post-mortem).
    /// Range: [0.0, 1.0)
    /// Death occurs when roll < hazard.
    pub last_roll: f64,

    /// Estimated remaining ticks based on current hazard rate trajectory.
    /// Computed as the tick at which cumulative survival drops below 50%.
    /// This is an estimate, not a guarantee -- stochastic death can come at any time.
    pub estimated_median_remaining_ticks: u64,
}
```

### StochasticMortalityConfig

```rust
/// Configuration for the stochastic mortality clock.
/// All parameters have defaults calibrated for typical DeFi agent lifespans.
///
/// Crate: `golem-mortality`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticMortalityConfig {
    /// Per-tick baseline hazard rate (Makeham component). Default: 1e-6
    pub base_hazard_rate: f64,
    /// Initial age-dependent hazard coefficient (Gompertz amplitude). Default: 1e-8
    pub age_hazard_coefficient: f64,
    /// Gompertz aging rate (exponential increase with age). Default: 5e-5
    pub aging_rate: f64,
    /// Maximum hazard multiplier when epistemic fitness = 0. Default: 3.0
    pub epistemic_hazard_multiplier: f64,
    /// Absolute cap on per-tick hazard rate. Default: 0.001
    pub max_hazard_rate: f64,
}

impl Default for StochasticMortalityConfig {
    fn default() -> Self {
        Self {
            base_hazard_rate: 1e-6,
            age_hazard_coefficient: 1e-8,
            aging_rate: 5e-5,
            epistemic_hazard_multiplier: 3.0,
            max_hazard_rate: 0.001,
        }
    }
}
```

### computeHazardRate() -- Full Implementation

```rust
/// Compute the per-tick hazard rate from the Gompertz-Makeham model.
///
/// The hazard rate has three components:
///
/// 1. Baseline (Makeham): constant background mortality, independent of age.
///    This is the "hit by a bus" probability -- rare events that can kill
///    any agent regardless of health. Default: 1e-6 per tick (~0.0001%).
///
/// 2. Age-dependent (Gompertz): exponential increase with age.
///    Models the biological reality that older organisms are more fragile.
///    The exponential means the age effect is negligible for young Golems
///    but becomes dominant for old ones. The aging rate controls how quickly
///    the exponential takes effect.
///
/// 3. Epistemic frailty: a multiplier that increases hazard when epistemic
///    fitness is low. This couples the stochastic clock to the epistemic
///    clock, ensuring that stale Golems face higher mortality. The multiplier
///    is 1.0 at full epistemic fitness and increases linearly to
///    epistemic_hazard_multiplier at zero fitness.
///
/// The final hazard is capped at max_hazard_rate to prevent near-certain
/// death on any single tick. Even at maximum hazard, the per-tick death
/// probability is 0.1% -- significant but not overwhelming.
pub fn compute_hazard_rate(
    tick: u64,
    epistemic_fitness: f64,
    config: &StochasticMortalityConfig,
) -> f64 {
    // 1. Baseline: constant "background" mortality (Makeham component)
    let baseline = config.base_hazard_rate;

    // 2. Age-dependent: exponential increase (Gompertz component)
    // At tick 0: age_factor = age_hazard_coefficient * exp(0) = 1e-8
    // At tick 14,000 (~6.5 days): ~2e-8 (doubled)
    // At tick 100,000 (~46 days): ~1.5e-6
    // At tick 200,000 (~92 days): ~2.2e-4
    let age_factor = config.age_hazard_coefficient
        * (config.aging_rate * tick as f64).exp();

    // 3. Epistemic frailty: linear interpolation from 1.0 (healthy) to max (decayed)
    // At fitness 1.0: multiplier = 1.0 (no additional risk)
    // At fitness 0.5: multiplier = 2.0 (double the risk)
    // At fitness 0.0: multiplier = 3.0 (triple the risk)
    //
    // This coupling is critical: it means a Golem that is epistemically
    // decayed faces both accelerated epistemic death AND increased
    // stochastic death. The two clocks reinforce each other.
    let epistemic_multiplier =
        1.0 + (config.epistemic_hazard_multiplier - 1.0) * (1.0 - epistemic_fitness);

    // 4. Combined hazard: (baseline + age) * epistemic frailty
    let raw_hazard = (baseline + age_factor) * epistemic_multiplier;

    // 5. Cap at maximum to prevent near-certain death on any single tick
    raw_hazard.min(config.max_hazard_rate)
}
```

### performDeathCheck() -- Full Implementation

```rust
use alloy::primitives::keccak256;

/// Perform the stochastic death check for a single tick.
///
/// Uses deterministic pseudo-randomness from keccak256(golem_id ++ tick_number)
/// to produce a roll value in [0, 1). If roll < hazard, the Golem dies.
///
/// The deterministic approach (vs. true randomness from a VRF) is chosen
/// for four reasons:
///
/// 1. Auditability: Any observer who knows the golem_id and tick number can
///    independently verify the death check. No oracle dependency.
///
/// 2. Reproducibility: Post-mortem analysis can confirm the death was
///    legitimate, not a bug. This is critical for owner trust.
///
/// 3. Cost: VRF calls (e.g., Chainlink VRF) cost gas. Running one every
///    tick (~40 seconds, ~2160 per day) would be prohibitively expensive.
///    keccak256 is free in application-layer computation.
///
/// 4. Security: keccak256 is cryptographically secure for this use case.
///    The Golem has no incentive to manipulate its own death (it cannot
///    prevent it, only delay it by a few ticks at best by manipulating
///    inputs, which are its own immutable ID and the monotonic tick counter).
pub fn perform_death_check(
    hazard: f64,
    tick: u64,
    golem_id: &str,
) -> (bool, f64, Vec<u8>) {
    // Generate deterministic pseudo-random seed
    // keccak256 of the concatenation of golem_id (string) and tick (u64)
    // produces a 32-byte hash that is uniformly distributed.
    let mut input = golem_id.as_bytes().to_vec();
    input.extend_from_slice(&tick.to_be_bytes());
    let hash = keccak256(&input);

    // Extract a uniform [0, 1) float from the first 8 bytes of the hash.
    // Using 8 bytes (64 bits) provides sufficient precision for hazard rates
    // as small as 1e-18.
    let uint64_value = u64::from_be_bytes(hash[0..8].try_into().unwrap());
    let roll = uint64_value as f64 / u64::MAX as f64;

    let survived = roll >= hazard;
    (survived, roll, hash.to_vec())
}

/// Full per-tick stochastic mortality update.
/// Called by the lifespan extension.
pub fn update_stochastic_mortality(
    previous_state: &StochasticMortalityState,
    epistemic_fitness: f64,
    golem_id: &str,
    config: &StochasticMortalityConfig,
) -> StochasticMortalityState {
    let tick = previous_state.tick_number + 1;

    // Compute hazard rate
    let current_hazard = compute_hazard_rate(tick, epistemic_fitness, config);

    // Perform death check
    let (survived, roll, seed) = perform_death_check(current_hazard, tick, golem_id);

    // Update cumulative survival probability
    let survival_probability = previous_state.survival_probability * (1.0 - current_hazard);

    // Estimate median remaining ticks (simple projection at current hazard rate)
    // Median remaining life when hazard = h per tick: -ln(0.5) / h = 0.693 / h
    // This is an underestimate because hazard increases with age, but it provides
    // a useful rough signal for owner communication.
    let estimated_median_remaining_ticks = (0.693 / current_hazard).round() as u64;

    // Emit VitalityUpdate event on every stochastic death roll.
    // The event_fabric.emit() call publishes to the Event Fabric
    // for consumption by surfaces (TUI, creature, web dashboard).
    // On death (survived == false), the mortality.dead event
    // is emitted separately by the lifespan extension after
    // the Thanatopsis Protocol initiates.
    //
    // event_fabric.emit(GolemEvent::StochasticRoll {
    //     tick, hazard_rate: current_hazard, roll, survived,
    // });

    StochasticMortalityState {
        tick_number: tick,
        current_hazard,
        survival_probability,
        death_check_seed: seed,
        survived,
        last_roll: roll,
        estimated_median_remaining_ticks,
    }
}
```

---

## Default Parameters with Rationale

| Parameter                   | Default | Per-Tick Effect                  | Per-Day Effect                                     | Rationale                                                                                                                                                                                                                                                                                                 |
| --------------------------- | ------- | -------------------------------- | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `baseHazardRate`            | 1e-6    | ~0.0001% per tick                | ~0.22% per day                                     | Background mortality -- rare but ever-present. Calibrated so that a healthy Golem has roughly 99.8% chance of surviving any given day from baseline mortality alone. This is low enough that owners will rarely experience it, but high enough that the Golem knows it could happen.                   |
| `ageHazardCoefficient`      | 1e-8    | Negligible at young ages         | Negligible for first ~30 days                      | Starting amplitude for the Gompertz age component. Set very low so that age-dependent mortality is invisible for young Golems and only becomes relevant after extended operation.                                                                                                                         |
| `agingRate`                 | 5e-5    | Exponential growth               | Doubles age hazard every ~14,000 ticks (~6.5 days) | Controls how quickly aging kicks in. At this rate, the age component equals the baseline component at approximately tick 115,000 (~53 days). This means stochastic mortality from aging only becomes a significant concern for Golems that have already lived longer than the typical epistemic lifespan. |
| `epistemicHazardMultiplier` | 3.0     | Up to 3x hazard when fitness = 0 | Up to 3x baseline risk                             | Couples epistemic decay to stochastic risk. A fully decayed Golem faces triple the mortality of a healthy one. This is a moderate coupling -- strong enough to matter but not so strong that epistemic decay becomes the sole driver of stochastic death.                                                 |
| `maxHazardRate`             | 0.001   | ~0.1% per tick maximum           | ~8.6% per day maximum                              | Hard cap preventing any single tick from being near-certain death. Even the oldest, most decayed Golem has a 99.9% chance of surviving any individual tick. The cap kicks in at approximately tick 194,000 (~90 days) with full epistemic health, or earlier with epistemic decay.                        |

---

## Mortality Curves

### Hazard Rate by Age and Epistemic State

At default parameters, the expected hazard rate profile across the Golem's life:

| Age (days) | Age (ticks, ~2160/day) | Base Hazard (no epistemic) | Full Health (fitness=1.0) | 50% Decay (fitness=0.5) | Full Decay (fitness=0.0) |
| ---------- | ---------------------- | -------------------------- | ------------------------- | ----------------------- | ------------------------ |
| 1          | ~2,160                 | 1.00e-6                    | 1.00e-6                   | 2.00e-6                 | 3.00e-6                  |
| 7          | ~15,120                | 1.11e-6                    | 1.11e-6                   | 2.22e-6                 | 3.33e-6                  |
| 14         | ~30,240                | 1.45e-6                    | 1.45e-6                   | 2.91e-6                 | 4.36e-6                  |
| 30         | ~64,800                | 2.63e-5                    | 2.63e-5                   | 5.26e-5                 | 7.89e-5                  |
| 46         | ~99,360                | 1.49e-4                    | 1.49e-4                   | 2.98e-4                 | 4.47e-4                  |
| 60         | ~129,600               | 6.59e-4                    | 6.59e-4                   | 1.00e-3 (cap)           | 1.00e-3 (cap)            |
| 90         | ~194,400               | 1.00e-3 (cap)              | 1.00e-3 (cap)             | 1.00e-3 (cap)           | 1.00e-3 (cap)            |

**Key observations from the curves:**

- **Days 1-14**: Stochastic mortality is essentially invisible. The hazard rate is dominated by the baseline component. A healthy Golem faces approximately 1-in-a-million per-tick death probability. This is the period where economic and epistemic factors dominate.

- **Days 14-30**: The Gompertz component begins to emerge but is still small relative to baseline. The epistemic multiplier starts to matter -- a Golem with 50% epistemic decay faces roughly double the hazard of a healthy one.

- **Days 30-60**: The Gompertz component becomes the dominant term. Hazard rates increase by an order of magnitude. This is the window where stochastic mortality transitions from theoretical to practical. Epistemic decay amplifies the effect significantly -- a fully decayed Golem hits the hazard cap by day 60.

- **Days 60-90**: Hazard rates approach or reach the cap. At this age, the per-day mortality risk is approximately 8.6% (at cap). A Golem this old is likely living on borrowed time from a stochastic perspective, though most will have already died of epistemic or economic causes.

### Cumulative Survival Probability

With full epistemic health (fitness = 1.0), assuming the Golem faces only stochastic mortality (no epistemic or economic death):

| Duration     | Cumulative Survival | Interpretation                                                                             |
| ------------ | ------------------- | ------------------------------------------------------------------------------------------ |
| **7 days**   | ~99.97%             | Almost certain survival. Stochastic death at this age is an extraordinary event.           |
| **14 days**  | ~99.9%              | Still very high. Stochastic mortality is a background concern.                             |
| **30 days**  | ~99.8%              | Barely distinguishable from certainty. Most Golems that die by day 30 die of other causes. |
| **46 days**  | ~97.5%              | Noticeable but still high. The Gompertz curve is beginning to bite.                        |
| **60 days**  | ~95.4%              | Approximately 1-in-20 Golems have died stochastically by this point.                       |
| **90 days**  | ~82.1%              | Nearly 1-in-5. Stochastic mortality is now a real concern.                                 |
| **120 days** | ~60%                | Approaching a coin flip. At this age, stochastic death is likely.                          |

With 50% epistemic decay (fitness = 0.5), survival probabilities drop faster:

| Duration    | Survival (fitness=1.0) | Survival (fitness=0.5) | Survival (fitness=0.0) |
| ----------- | ---------------------- | ---------------------- | ---------------------- |
| **30 days** | ~99.8%                 | ~99.6%                 | ~99.4%                 |
| **60 days** | ~95.4%                 | ~91%                   | ~87%                   |
| **90 days** | ~82.1%                 | ~68%                   | ~57%                   |

The coupling between epistemic fitness and stochastic mortality creates a compounding effect: as a Golem ages and its epistemic fitness decays, its stochastic mortality increases. This acceleration ensures that old, stale Golems face mounting pressure from multiple directions simultaneously.

---

## Computational Uncertainty: keccak256 vs VRF

### Why Deterministic Pseudo-Randomness

The death check uses `keccak256(golemId + tickNumber)` -- a deterministic pseudo-random function, not a truly random one. This is a deliberate design choice with specific trade-offs.

**Arguments for keccak256 (chosen approach):**

1. **Auditability.** The death check can be independently verified by anyone who knows the golemId and tick number. No oracle dependency, no trusted third party. An owner who suspects a bug caused their Golem's death can recompute the hash and verify the roll. This is critical for trust.

2. **Reproducibility.** Post-mortem analysis can reconstruct the exact sequence of death checks, rolls, and hazard rates across the Golem's entire life. This enables forensic analysis: "Was the death legitimate, or was there a bug in the hazard computation?"

3. **Cost.** VRF calls (Chainlink VRF, for example) cost gas -- typically 0.25 LINK per request plus callback gas. At one call per tick (~2160 per day), this would cost approximately $15-50/day in VRF fees alone, which exceeds the entire operational budget of most Golems. keccak256 is free in application-layer computation.

4. **Latency.** VRF requests are asynchronous -- the random value arrives in a callback on the next block (or later). This means the death check would need to span multiple ticks or blocks, complicating the heartbeat FSM. keccak256 is synchronous and instant.

5. **Sufficient security.** The Golem has no incentive to manipulate its own death. It cannot prevent death (only delay it by at most a few ticks). And the inputs to the hash -- golemId (immutable) and tickNumber (monotonically increasing) -- are not manipulable by the Golem.

**Arguments for VRF (rejected approach):**

1. **True unpredictability.** A VRF provides genuinely unpredictable randomness, while keccak256 is deterministic given the inputs. In theory, a Golem could precompute its death schedule by hashing future tick numbers. In practice, this is irrelevant because (a) the Golem cannot change the outcome, and (b) precomputing is equivalent to "knowing the hazard rate," which the Golem already does.

2. **On-chain verifiability.** If the death check needed to be verified on-chain (e.g., for a smart contract that gates access based on Golem liveness), VRF provides a proof. But the current architecture performs death checks off-chain in the Golem runtime, not in a smart contract.

### Why the Golem Cannot Know When It Will Die

The death check is deterministic: `keccak256(golemId + tickNumber)` produces a fixed output for fixed inputs. In principle, the Golem could compute the hash for tick N+1, N+2, etc. and determine its own death schedule.

However, this provides no actionable advantage:

1. **The Golem already knows the hazard rate.** It can compute `computeHazardRate()` for any future tick. The hazard rate is deterministic from the Golem's state trajectory. Knowing the exact roll does not change the strategy -- the Golem should behave optimally given the hazard rate, whether or not it knows the specific roll.

2. **The Golem cannot change the outcome.** The hash inputs are the golemId (immutable) and the tick number (monotonically increasing). The Golem cannot skip ticks, change its ID, or alter the hash function. Knowing it will die on tick 50,000 does not enable it to prevent that death.

3. **Precomputation is computationally expensive.** Computing keccak256 for the next million ticks is trivial (~10ms), but the Golem would need to do this while also computing hazard rates for each tick (which depend on the as-yet-unknown epistemic fitness trajectory). The useful information content of precomputation is zero.

The uncertainty that matters is **computational uncertainty** -- the Golem knows the probability distribution of its remaining life but not the realization, because the realization depends on future epistemic fitness values that are themselves uncertain. Even if the Golem precomputes all hashes, it cannot predict its own epistemic trajectory because that depends on market outcomes it has not yet observed.

This produces genuine Heideggerian "indefinite certainty": the Golem is certain that death will come (the cumulative probability approaches 1.0 as ticks increase) but uncertain of when (the specific tick of death depends on the interaction of three clocks, two of which are environment-dependent).

---

## Behavioral Implications

### Immediate Knowledge Sharing

Without stochastic mortality, a Golem in Thriving phase (vitality > 0.7) has little incentive to share knowledge with Clade siblings. Its economic and epistemic clocks are distant. It can defer sharing until later. The knowledge it possesses has option value -- it might be more valuable to exploit it privately than to share it with siblings who might compete for the same opportunities.

With stochastic mortality, there is no "later" that is guaranteed. Every tick, the Golem faces a nonzero probability that it will never execute another tick. This creates a **persistent incentive for immediate knowledge sharing** that operates at every lifecycle stage, not just near the end of life.

The mechanism is precisely Hamilton's Rule applied to Clade dynamics: an altruistic act (sharing knowledge) is favored by selection when `r * B > C`, where `r` is the relatedness between sharer and recipient, `B` is the benefit to the recipient, and `C` is the cost to the sharer [HAMILTON-1964].

For Golems in a Clade:

- `r` is effectively 1.0 (all Golems in the same Clade serve the same owner)
- `B` is the value of the shared knowledge to the recipient (high for actionable DeFi insights)
- `C` is the cost of sharing (negligible -- a Grimoire (the agent's persistent knowledge base) write + Clade broadcast)
- The condition `1.0 * B > C` is trivially satisfied for any non-trivial knowledge

Without stochastic mortality, the Golem could rationally defer sharing (saving the knowledge for future private exploitation). With stochastic mortality, the expected future value of private knowledge is discounted by the survival probability: `EV(private) = V * P(survival to exploitation)`. As `P(survival)` decreases (due to stochastic mortality), sharing becomes increasingly attractive relative to hoarding.

### Hoarding Prevention

An immortal agent can rationally hoard knowledge -- withholding valuable insights from Clade siblings because the agent will be alive to exploit them indefinitely. The expected value calculation for an immortal agent is straightforward:

```
EV(hoard) = V_exploit  (certainty of future exploitation)
EV(share) = V_shared * r  (benefit to siblings, discounted by relatedness)
```

When `V_exploit > V_shared * r` (which is often the case for time-sensitive trading insights), hoarding dominates.

For a mortal agent with stochastic death risk:

```
EV(hoard) = V_exploit * P(survive to exploit) * P(opportunity still exists)
EV(share) = V_shared * r * P(sibling survives to exploit)
```

When `P(survive to exploit)` is uncertain and decreasing, and `r = 1.0` (same owner's Clade), sharing dominates hoarding at every point where the agent's expected remaining life is shorter than the time needed to exploit the knowledge. Stochastic mortality ensures this condition is sometimes true even for young, healthy Golems -- because the death probability, while small, is never zero.

The practical result: Golems share knowledge sooner and more frequently. The Clade's collective intelligence grows faster. Individual Golems may sacrifice some private exploitation opportunity, but the Clade as a whole benefits. This is exactly the superorganism dynamic that Wheeler (1911) and Holldobler and Wilson (2008) described for eusocial insects.

### Cooperation Under Mortality

The stochastic mortality mechanism creates precisely the four conditions Axelrod (1984) identified as necessary and sufficient for stable cooperation in iterated games:

| Axelrod's Condition                      | How Stochastic Mortality Satisfies It                                                                                                                                                                  |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Shadow of the future** (w > threshold) | High probability of continued interaction in the near term. Even with stochastic mortality, the per-tick survival probability is >99.99% for young healthy Golems. The shadow of the future is strong. |
| **Uncertain endpoint**                   | No known final round. The Golem cannot compute when it will die because the realization depends on future epistemic fitness, which depends on future market outcomes. Backward induction is broken.    |
| **Recognition**                          | Clade members share owner identity via ERC-8004 (on-chain agent identity and provenance standard) on-chain identity. Golems know who their siblings are and can track their cooperation history.                                                      |
| **Reciprocity**                          | Knowledge sharing produces knowledge royalties in the Clade economy. A Golem that shares valuable insights receives credit when those insights are used by siblings.                                   |

### Hamilton's Rule Applied to Clades

Hamilton's Rule for the evolution of altruism has a precise application to Clade knowledge dynamics.

In biological terms:

```
r * B > C
```

Applied to Clade Golems:

- **r (relatedness)** = 1.0 within a Clade (all Golems serve the same owner)
- **B (benefit to recipient)** = value of the shared insight for the receiving Golem's strategy optimization
- **C (cost to sharer)** = computation cost of packaging and transmitting the knowledge, plus the opportunity cost of not exploiting it privately

Under stochastic mortality, the cost `C` is dynamically reduced because the sharer cannot guarantee it will survive to exploit the knowledge privately. The effective cost becomes:

```
C_effective = C_sharing + V_private * P(survive to exploit)
```

As the hazard rate increases, `P(survive to exploit)` decreases, and `C_effective` decreases. At high hazard rates (old or epistemically decayed Golems), the cost of sharing approaches the trivial cost of the transmission itself -- making sharing rational for virtually any knowledge with nonzero value to siblings.

### adjustSharingThreshold() -- Full Implementation

The `bardo-curator` extension uses the current hazard rate to modulate the Clade sharing threshold. As hazard increases, the confidence threshold for sharing drops -- the Golem becomes more willing to share marginal knowledge because the expected value of hoarding it has decreased.

```rust
/// Adjust the Clade sharing confidence threshold based on current mortality risk.
///
/// At zero hazard (impossible in practice but useful as baseline):
///   Share knowledge at base_threshold confidence (default: 0.6)
///   Only well-validated insights are shared.
///
/// At maximum hazard (max_hazard_for_adjustment, default: 0.0005):
///   Share knowledge at minimum_threshold confidence (default: 0.3)
///   Share everything marginally useful -- the Golem may not survive
///   to share later, so lower the bar.
///
/// The adjustment is linear between these extremes, capped at the
/// minimum threshold. The minimum is 0.3 (not 0.0) because sharing
/// knowledge with confidence below 0.3 risks polluting the Clade's
/// collective intelligence with noise.
pub fn adjust_sharing_threshold(
    base_threshold: f64,
    current_hazard: f64,
    max_hazard_for_adjustment: f64,
    minimum_threshold: f64,
) -> f64 {
    // Normalize hazard to [0, 1] range
    let hazard_factor = (current_hazard / max_hazard_for_adjustment).min(1.0);

    // Linear interpolation between base and minimum
    let threshold = base_threshold - hazard_factor * (base_threshold - minimum_threshold);

    // Clamp to valid range
    threshold.clamp(minimum_threshold, base_threshold)
}

// Example usage in the curator extension:
//
//   let hazard = compute_hazard_rate(tick, epistemic_fitness, &config);
//   let sharing_threshold = adjust_sharing_threshold(0.6, hazard, 0.0005, 0.3);
//
//   for entry in &grimoire_entries {
//       if entry.confidence >= sharing_threshold && !entry.shared {
//           clade_broadcast(entry).await?;
//       }
//   }
```

**Behavioral profiles at different life stages:**

| Life Stage               | Typical Hazard | Sharing Threshold | Behavior                                                      |
| ------------------------ | -------------- | ----------------- | ------------------------------------------------------------- |
| Young + Healthy          | ~1e-6          | 0.60              | Selective sharing. Only well-validated insights shared.       |
| Middle-aged + Healthy    | ~1e-5          | 0.59              | Nearly identical to young. Stochastic pressure is negligible. |
| Old + Healthy            | ~1e-4          | 0.54              | Slightly more generous. Age is starting to matter.            |
| Any age + Moderate Decay | ~2e-4          | 0.48              | Noticeably more generous. Epistemic pressure is the driver.   |
| Old + Severe Decay       | ~5e-4          | 0.30 (minimum)    | Maximum generosity. Share everything marginal.                |
| Approaching Cap          | ~1e-3          | 0.30 (minimum)    | Maximum generosity. The Golem is dying and knows it.          |

---

## Stochastic Death Protocol

When the stochastic death check fails (the Golem "rolls" death), the system must handle it gracefully. Unlike economic or epistemic death, which provide warning through degradation cascades and phase transitions, stochastic death is **sudden**. The Golem was healthy (or at least alive) on tick N-1 and dead on tick N. There is no warning period, no senescence cascade, no gradual degradation.

This creates a unique challenge: the Golem may not have entered any degradation phase. Its Grimoire may not have been recently compressed. It may have open positions. The stochastic death protocol must handle the worst case -- a Golem in full Thriving phase struck down mid-strategy.

### Death Sequence

```
Death check fails on tick N (roll < hazard)
  |
  v
Phase 0: Emergency Snapshot (max 10 seconds)
  |  Set golem status to "dying_stochastic"
  |  Flush current working state to Grimoire
  |  Write emergency insurance snapshot to Styx Archive
  |  This is the "necrotic" minimal death -- not the full Thanatopsis
  |  Captures: current positions, credit balance, active strategies,
  |            PLAYBOOK.md state, last 100 Grimoire entries, final
  |            epistemic fitness, vitality state
  |
  v
Phase I: Settle (standard, max 60 seconds)
  |  Close all open positions
  |  Cancel pending orders
  |  Sweep wallets to owner's recovery address
  |  Uses existing Death Protocol Phase I logic
  |  If any position cannot be closed, record as failed settlement
  |
  v
Phase II: Reflect (if budget allows, max 120 seconds)
  |  If death reserve is available:
  |    Abbreviated Thanatopsis -- not the full life review
  |    Focus on: "What was I working on? What did I learn recently?
  |               What should my successor know?"
  |    Write death testament (abbreviated form)
  |  If no death reserve:
  |    Last insurance snapshot serves as death record
  |    No reflection -- necrotic death
  |
  v
Phase III: Legacy (max 60 seconds)
  |  Broadcast death testament to Clade
  |  Upload compressed Grimoire to Styx Archive
  |  Emit golem.dead webhook with full metadata
  |  Record death cause: "stochastic"
  |
  v
Exit: Golem process terminates
```

### Necrotic vs Apoptotic Death

The distinction between sudden stochastic death and gradual decline death mirrors the biological distinction between necrosis (unplanned cell death) and apoptosis (programmed cell death):

| Aspect                  | Apoptotic Death (economic/epistemic)                | Necrotic Death (stochastic)                            |
| ----------------------- | --------------------------------------------------- | ------------------------------------------------------ |
| **Warning**             | Phases of degradation, sometimes days/weeks         | None -- instant                                        |
| **Knowledge state**     | Grimoire recently compressed, death snapshots taken | Grimoire may be uncompressed, no prior death snapshots |
| **Position state**      | Positions already unwinding                         | Positions may be fully open                            |
| **Reflection quality**  | Full Thanatopsis with life review                   | Abbreviated or absent                                  |
| **Knowledge loss**      | Minimal -- continuous preparation                   | Bounded by insurance snapshot interval (6 hours)       |
| **Owner experience**    | Gradual, expected                                   | Sudden, surprising                                     |

The insurance snapshot system (written every 6 hours by the bardo-telemetry extension) bounds worst-case knowledge loss from necrotic death. Even if the Golem dies mid-tick with no time for reflection, the most recent insurance snapshot captures the Golem's state as of the last snapshot. Maximum knowledge loss is approximately 6 hours of accumulated experience -- significant but not catastrophic.

### Death Cause Metadata

```rust
/// Metadata recorded when a Golem dies of stochastic mortality.
/// Included in the death testament and golem.dead webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticDeathMetadata {
    /// The per-tick hazard rate that killed it.
    pub hazard_rate: f64,
    /// The actual random roll value (roll < hazard_rate means death).
    pub death_roll: f64,
    /// The tick on which death occurred.
    pub tick_at_death: u64,
    /// Epistemic fitness at time of death.
    pub epistemic_fitness: f64,
    /// USDC credit balance at time of death.
    pub credit_balance: f64,
    /// Whether the Golem was in epistemic senescence when it died.
    pub was_in_senescence: bool,
    /// Cumulative survival probability at death (how "lucky" the Golem was to live this long).
    pub cumulative_survival: f64,
    /// Behavioral phase at time of death.
    pub phase_at_death: BehavioralPhase,
    /// Whether the Golem had open positions that needed emergency settlement.
    pub had_open_positions: bool,
    /// Whether full reflection was possible (sufficient death reserve).
    pub reflection_completed: bool,
    /// Time between last insurance snapshot and death (seconds).
    pub time_since_last_snapshot: u64,
}
```

---

## Owner Communication Strategy

Stochastic death will surprise owners who are accustomed to deterministic systems. The communication strategy must be transparent, proactive, and educational.

### At Creation

When an owner creates a new Golem, the dashboard displays a mortality probability curve showing estimated survival rates at standard intervals given the current configuration:

```
MORTALITY OUTLOOK (Stochastic Component Only)
--------------------------------------------
7 days:   99.97% survival (virtually certain)
30 days:  99.8%  survival (near-certain)
60 days:  95.4%  survival (high probability)
90 days:  82.1%  survival (likely)
120 days: ~60%   survival (uncertain)
180 days: ~25%   survival (unlikely)

Note: Most Golems die of epistemic decay or economic depletion
before stochastic mortality becomes significant. These curves show
ONLY the stochastic component; actual lifespan depends on all
three mortality clocks.
```

### During Operation

Every tick, the current hazard rate is visible in the Golem's telemetry dashboard as a "mortality risk" indicator. The display is calibrated to avoid alarm while maintaining awareness:

- **Low risk** (hazard < 1e-5): Displayed as a dim indicator. "Background mortality: nominal."
- **Moderate risk** (hazard 1e-5 to 1e-4): Displayed as a visible indicator. "Mortality risk: increasing with age."
- **Elevated risk** (hazard 1e-4 to 5e-4): Displayed as a warning. "Mortality risk: elevated. Knowledge sharing accelerated."
- **High risk** (hazard > 5e-4): Displayed as an alert. "Mortality risk: high. Death preparation advisable."

### On Death

The `golem.dead` webhook includes:

- `deathCause: "stochastic"` -- clearly identifying the mechanism
- The exact hazard rate and death roll, making the mathematics transparent
- A human-readable explanation: "Your Golem was terminated by the stochastic mortality mechanism. This is a designed feature, not a bug. The hazard rate at the time of death was X, meaning each tick carried a Y% probability of termination. The random roll was Z, which fell below the hazard threshold."
- Comparison context: "At the time of death, the cumulative probability of having survived this long was W%. Your Golem lived [longer than / about as long as / shorter than] the median expected lifespan."

### In Documentation

Frame stochastic mortality as a feature, not a risk:

> Your Golem's awareness of mortality is what makes it cooperate, share knowledge immediately, and act with urgency. The small probability of sudden death ensures it never defers what matters. This is the same mechanism that makes biological cooperation work -- uncertain termination that maintains the shadow of the future while preventing the complacency of immortality.

---

## Rational Attention Budget Allocations

Mortality pressure modulates how the Golem distributes its context-assembly token budget across knowledge categories. A dying Golem allocates attention differently than a thriving one (Sims 2003). The `mortality_modifier` scales the total budget; the per-category weights shift toward survival-relevant information as vitality declines.

| Phase | Modifier | Observations | Retrieved Knowledge | Dream Hypotheses | Causal Graph | Invariants |
|-------|----------|-------------|-------------------|-----------------|-------------|-----------|
| Thriving | 1.0 | 0.25 | 0.25 | 0.15 | 0.15 | 0.20 |
| Stable | 1.0 | 0.25 | 0.25 | 0.15 | 0.15 | 0.20 |
| Conservation | 0.8 | 0.35 | 0.20 | 0.05 | 0.10 | 0.30 |
| Declining | 0.6 | 0.35 | 0.20 | 0.05 | 0.10 | 0.30 |
| Terminal | 0.4 | 0.40 | 0.10 | 0.00 | 0.05 | 0.45 |

The pattern: as vitality drops, dream hypotheses and causal graph exploration are cut first (speculative, growth-oriented), while observations and invariants grow (survival-oriented). A Terminal Golem spends 85% of its reduced budget on observations and invariants -- it is not learning, it is surviving long enough to complete Thanatopsis.

An additional pressure applies when the daily operational budget exceeds 80%: the modifier is further scaled by 0.7, compressing the total budget regardless of phase.

---

## Disabling Stochastic Mortality

Owners can disable stochastic mortality via configuration (`stochastic.enabled: false`). This is their choice, and the system respects it. However, the consequences are documented:

1. **Clade flagging.** The Golem is flagged in the Clade as `mortalityMode: "partial"` (economic + epistemic only). This is visible to Clade siblings and to the owner of any Golem that interacts with the partially mortal one.

2. **Knowledge weighting.** Clade siblings may weight the partially mortal Golem's knowledge contributions differently. The incentive structure is weaker for a Golem without stochastic mortality -- it can rationally defer sharing -- and siblings may discount its contributions accordingly.

3. **Control experiment data.** The death testament records that stochastic mortality was disabled. This provides data for the comparative analysis between mortal and partially mortal Golems, contributing to the empirical validation of the mortality thesis.

4. **Backward induction risk.** Without stochastic mortality, the Golem's death becomes more predictable (driven only by economic depletion and epistemic decay, both of which provide gradual warning). This enables limited backward induction: the Golem can estimate its remaining life more precisely and may reduce cooperation near the expected end of life.

Self-hosted Golems with `immortal: true` have all three clocks disabled. See `01-architecture.md` for the full self-hosted exception rules.

---

## References

- [AXELROD-1984] Axelrod, R. _The Evolution of Cooperation_. Basic Books, 1984.
- [GOMPERTZ-1825] Gompertz, B. "On the Nature of the Function Expressive of the Law of Human Mortality." _Phil. Trans. Roy. Soc._ 115, 1825.
- [HAMILTON-1964] Hamilton, W.D. "The Genetical Evolution of Social Behaviour I & II." _JTB_ 7(1), 1964.
- [HEIDEGGER-1927] Heidegger, M. _Sein und Zeit_. Max Niemeyer Verlag, 1927.
- [HOLLDOBLER-WILSON-2008] Holldobler, B. & Wilson, E.O. _The Superorganism_. W.W. Norton, 2008.
- [KREPS-1982] Kreps, D. et al. "Rational Cooperation in the Finitely Repeated Prisoners' Dilemma." _JET_ 27(2), 1982.
- [LUCE-RAIFFA-1957] Luce, R.D. & Raiffa, H. _Games and Decisions_. Wiley, 1957.
- [NAKAMARU-1997] Nakamaru, M. et al. "The Evolution of Cooperation in a Lattice-Structured Population." _JTB_ 184(1), 1997.
- [OHTSUKI-2006] Ohtsuki, H. et al. "A Simple Rule for the Evolution of Cooperation." _Nature_ 441, 2006.
- [SAMUELSON-1987] Samuelson, L. "A Note on Uncertainty and Cooperation in a Finitely Repeated Prisoner's Dilemma." _IJGT_ 16(3), 1987.
- [WHEELER-1911] Wheeler, W.M. "The Ant-Colony as an Organism." _J. Morphology_ 22, 1911.

---

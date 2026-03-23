# golem-mortality

Three independent death clocks that drive Golem lifespan, behavioral phase transitions, and epistemic decay. Any clock reaching zero triggers succession — but they're designed to fire at different rates. Epistemic death is the primary mechanism by design: an agent whose world model has drifted enough that a fresh successor would outperform it _should_ die.

## Clocks

### `EconomicClock`

Counts USDC burned as metabolic substrate. Constructed with an initial credit balance and a `death_reserve_proportion` (default 5%), which locks a portion of credits into an `ApoptoticReserve` for graceful shutdown. The spendable balance is `initial_credits * (1 - reserve_proportion)`.

Call `tick_cost(cost: f64)` after each tick to deduct the actual spend. The clock maintains an EMA burn rate (α=0.05) and a `lifetime_mean` across all ticks. `estimated_ttl_ticks()` returns `credit_remaining / burn_rate_per_tick`.

Fires `ClockEvent::Dead { cause: DeathCause::Economic }` when `credit_remaining <= 0`.

### `EpistemicClock`

Tracks prediction accuracy across five market dimensions using a rolling R² computed over a 100-observation window. Accuracy is measured as a weighted binary match between `MarketPrediction` and `MarketOutcome`:

| Dimension | Weight |
|---|---|
| `price_direction` | 0.35 |
| `volatility_regime` | 0.25 |
| `yield_trend` | 0.20 |
| `gas_condition` | 0.10 |
| `protocol_state` | 0.10 |

R² requires at least 10 observations; returns 0.5 before that. The clock enters senescence when R² falls below 0.35, tracked through three stages:

- `Stage1` — warning, within the 50-tick grace period
- `Stage2` — confirmed, begins 100-tick escalation window
- `Stage3` — fires `ClockEvent::Dead { cause: DeathCause::Epistemic }`

Recovery requires R² > 0.45 (the 0.10 hysteresis gap prevents chatter). Recovery resets the senescence state machine but does not reset fitness to a fixed value — the EMA continues from where it is.

Call `record(PredictionOutcomePair)` after each tick with the predicted and actual scores.

### `StochasticClock`

Gompertz-Makeham hazard model with a deterministic death check using `keccak256(golem_id || tick.to_be_bytes())`. The hazard at tick `t` is:

```
h(t) = (λ + α·exp(β·t)) · ε(epistemic_fitness)
```

where `ε(f) = 1.0 + (multiplier - 1.0) * (1.0 - f)`. With `multiplier=3.0`, an agent at zero epistemic fitness faces 3× the hazard of a perfectly accurate one.

Default parameters: λ=1e-6, α=1e-8, β=5e-5, cap=0.001. At default settings the baseline hazard at tick 0 is ~1.01e-6 per tick. The cumulative `survival_probability` is the running product of `(1 - h)` across all ticks.

The death check is deterministic: same `(golem_id, tick)` pair always produces the same roll. This makes mortality reproducible across simulations.

## `VitalityState` and `BehavioralPhase`

`VitalityState` holds the composite view across all three clocks. The composite vitality is:

```
sigmoid(economic, center=0.3, steepness=10.0)
  × sigmoid(epistemic, center=0.4, steepness=8.0)
  × (1.0 - age_factor × age_drag).max(0.0)
```

where `age_factor = tick / reference_lifespan` (default 200,000 ticks). The composite is multiplicative — high economic health does not compensate for low epistemic fitness.

`BehavioralPhase` (from `golem_core::cortical`) maps composite vitality to phases:

| Phase | Composite range |
|---|---|
| `Thriving` | ≥ 0.70 |
| `Stable` | ≥ 0.50 |
| `Conservation` | ≥ 0.30 |
| `Declining` | ≥ 0.10 |
| `Terminal` | < 0.10 |

Downward transitions use raw thresholds. Upward transitions require `composite ≥ threshold + 0.05` to prevent oscillation. `ticks_in_phase` resets to 0 on any transition; `previous_phase` records where you came from.

`is_dead(state)` returns `true` if `composite < 0.01` or either `economic == 0.0` or `epistemic == 0.0`.

## Other modules

**`demurrage`** — Ebbinghaus decay on Grimoire entries. Entries transition `Active → Archived → Burned` as confidence drops. Domain multipliers and type weights modulate per-entry decay rates.

**`fractal`** — Three-level structure mirroring biological organization (mitoptosis, apoptosis, phenoptosis). Micro-death: Phage sub-agents test single hypotheses, live ~50 ticks (~33 minutes), report back, and die. Meso-death: Curator prunes stale Grimoire entries daily via knowledge demurrage. Macro-death: the Golem itself dies over weeks to months via the three mortality clocks. Feedback loop: micro informs meso (Phage results update Grimoire confidence), meso informs macro (pruning rate feeds epistemic fitness), macro constrains micro (a Golem in Conservation spawns fewer Phages, reducing self-testing and accelerating its own epistemic decay).

**`mortal_memory`** — Grimoire integration under mortality pressure. Epistemic age discounting and autopsy reporting for entries affected by mortality-driven decay.

## Information-Theoretic Diagnostics

A diagnostic layer that runs alongside the three clocks. It computes the mutual information I(G; M) between a Golem's state G and the market environment M using the KSG (Kraskov-Stogbauer-Grassberger) estimator, operating on sliding windows of (action, market_response) pairs.

```
I(G; M) = H(G) + H(M) - H(G, M)
```

A Golem with I(G; M) = 0 knows nothing — its state is statistically independent of the market's.

Each clock decomposes naturally into an MI component. Economic mortality maps to channel capacity: capital constrains the action space, and as capital shrinks, the channel capacity between Golem and market approaches zero. Epistemic mortality maps to rate-distortion: the Golem's actual information rate must exceed the minimum required to maintain acceptable prediction accuracy. Stochastic mortality maps to entropy production: the cumulative randomness injected into the Golem's trajectory.

This framing exposes three mortality modes invisible to the individual clocks:

- **Informational decoupling**: the Golem appears healthy by all three clocks but is statistically independent of market outcomes — it acts but nothing it does matters.
- **Overfitting**: high historical MI, near-zero current MI. The Golem has memorized past regimes that no longer apply.
- **Clade redundancy**: the Golem contributes no unique information that its siblings don't already provide. From the Clade's perspective, it is dead even if its individual clocks are healthy.

Bits become the common currency the multiplicative vitality system otherwise lacks.

## Usage

```rust
use golem_mortality::{
    EconomicClock, EpistemicClock, StochasticClock,
    MortalityClock, ClockContext, ClockEvent, DeathCause,
    VitalityState, VitalityConfig,
};
use golem_mortality::epistemic::PredictionOutcomePair;
use golem_mortality::stochastic::StochasticMortalityConfig;
use golem_mortality::vitality::update_vitality_state;

let mut econ = EconomicClock::new(10_000.0, 0.05);
let mut epist = EpistemicClock::new();
let mut stoch = StochasticClock::new(StochasticMortalityConfig::default());
let config = VitalityConfig::default();
let mut state = VitalityState::default();

// Per tick:
econ.tick_cost(0.50);  // deduct actual spend
epist.record(PredictionOutcomePair {
    predicted_score: 0.8,
    actual_score: 0.75,
    tick: 42,
});

let ctx = ClockContext {
    tick: 42,
    epistemic_fitness: epist.vitality(),
    golem_id: "golem-abc".into(),
};

let stoch_event = stoch.tick(&ctx);
let econ_event = econ.tick(&ctx);
let epist_event = epist.tick(&ctx);

update_vitality_state(
    &mut state,
    econ.vitality(),
    epist.vitality(),
    42,
    &config,
    now_epoch_secs,
);

if let ClockEvent::Dead { cause } = stoch_event {
    // handle DeathCause::Stochastic
}
```

## Dependencies

```toml
[dependencies]
golem-mortality = { path = "../../crates/golem-mortality" }
```

Depends on `golem-core` (for `BehavioralPhase`, `CorticalState`) and `golem-grimoire` (for mortal memory integration). Uses `alloy` for `keccak256` in the stochastic death check.

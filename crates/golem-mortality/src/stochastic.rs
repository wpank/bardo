//! Stochastic death clock — Gompertz-Makeham hazard with deterministic keccak256 death check.

use crate::clock::{ClockContext, ClockEvent, DeathCause, MortalityClock};
use alloy::primitives::keccak256;
use serde::{Deserialize, Serialize};

/// Configuration for the Gompertz-Makeham hazard model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StochasticMortalityConfig {
    /// Makeham component: age-independent background hazard rate (lambda).
    pub base_hazard_rate: f64,
    /// Gompertz amplitude (alpha).
    pub age_hazard_coefficient: f64,
    /// Gompertz rate (beta).
    pub aging_rate: f64,
    /// Multiplier on hazard due to epistemic frailty.
    pub epistemic_hazard_multiplier: f64,
    /// Maximum per-tick hazard rate cap.
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

/// Current state of the stochastic mortality clock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StochasticMortalityState {
    /// Current tick number.
    pub tick_number: u64,
    /// Current per-tick hazard rate.
    pub current_hazard: f64,
    /// Cumulative survival probability (product of (1 - h) over all ticks).
    pub survival_probability: f64,
    /// Last roll value from the death check.
    pub last_roll: f64,
    /// Whether the entity survived the last check.
    pub survived: bool,
    /// Configuration.
    pub config: StochasticMortalityConfig,
}

impl StochasticMortalityState {
    /// Create initial state with default config.
    pub fn new(config: StochasticMortalityConfig) -> Self {
        Self {
            tick_number: 0,
            current_hazard: config.base_hazard_rate,
            survival_probability: 1.0,
            last_roll: 1.0,
            survived: true,
            config,
        }
    }
}

/// Compute the Gompertz-Makeham hazard rate.
///
/// h(t) = (lambda + alpha * exp(beta * t)) * epsilon(t)
/// where epsilon(t) = 1.0 + (multiplier - 1.0) * (1.0 - epistemic_fitness)
pub fn compute_hazard_rate(
    tick: u64,
    epistemic_fitness: f64,
    config: &StochasticMortalityConfig,
) -> f64 {
    let baseline = config.base_hazard_rate;
    let age_factor = config.age_hazard_coefficient * (config.aging_rate * tick as f64).exp();
    let epist_mult = 1.0 + (config.epistemic_hazard_multiplier - 1.0) * (1.0 - epistemic_fitness);
    ((baseline + age_factor) * epist_mult).min(config.max_hazard_rate)
}

/// Deterministic death check using keccak256(golem_id ++ tick).
///
/// Returns (survived, roll, hash_bytes).
pub fn perform_death_check(hazard: f64, tick: u64, golem_id: &str) -> (bool, f64, Vec<u8>) {
    let mut input = golem_id.as_bytes().to_vec();
    input.extend_from_slice(&tick.to_be_bytes());
    let hash = keccak256(&input);
    let roll_bytes: [u8; 8] = hash[0..8].try_into().expect("hash is 32 bytes");
    let roll = u64::from_be_bytes(roll_bytes) as f64 / u64::MAX as f64;
    let survived = roll >= hazard;
    (survived, roll, hash.to_vec())
}

/// Update the stochastic mortality state for a new tick.
pub fn update_stochastic_mortality(
    state: &mut StochasticMortalityState,
    tick: u64,
    epistemic_fitness: f64,
    golem_id: &str,
) {
    state.tick_number = tick;
    state.current_hazard = compute_hazard_rate(tick, epistemic_fitness, &state.config);
    let (survived, roll, _) = perform_death_check(state.current_hazard, tick, golem_id);
    state.last_roll = roll;
    state.survived = survived;
    state.survival_probability *= 1.0 - state.current_hazard;
}

/// The stochastic death clock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StochasticClock {
    /// Inner state.
    pub state: StochasticMortalityState,
}

impl StochasticClock {
    /// Create with default config.
    pub fn new(config: StochasticMortalityConfig) -> Self {
        Self {
            state: StochasticMortalityState::new(config),
        }
    }
}

impl MortalityClock for StochasticClock {
    fn vitality(&self) -> f64 {
        if self.state.survived {
            // Vitality is the survival probability.
            self.state.survival_probability.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn tick(&mut self, ctx: &ClockContext) -> ClockEvent {
        update_stochastic_mortality(
            &mut self.state,
            ctx.tick,
            ctx.epistemic_fitness,
            &ctx.golem_id,
        );
        if !self.state.survived {
            ClockEvent::Dead {
                cause: DeathCause::Stochastic,
            }
        } else {
            ClockEvent::Alive {
                vitality: self.vitality(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stochastic_hazard_draws() {
        let config = StochasticMortalityConfig::default();
        let hazard = compute_hazard_rate(0, 0.8, &config);
        assert!(hazard > 0.0);
        assert!(hazard <= config.max_hazard_rate);
    }

    #[test]
    fn hazard_rate_increases_with_age() {
        let config = StochasticMortalityConfig::default();
        let h0 = compute_hazard_rate(0, 0.8, &config);
        let h1 = compute_hazard_rate(100_000, 0.8, &config);
        let h2 = compute_hazard_rate(200_000, 0.8, &config);
        assert!(h1 > h0, "h(100k)={h1} should be > h(0)={h0}");
        assert!(h2 > h1, "h(200k)={h2} should be > h(100k)={h1}");
    }

    #[test]
    fn epistemic_frailty_multiplier_scales_hazard() {
        let config = StochasticMortalityConfig::default();
        // Perfect fitness → multiplier 1.0.
        let h_perfect = compute_hazard_rate(1000, 1.0, &config);
        // Zero fitness → multiplier 3.0.
        let h_zero = compute_hazard_rate(1000, 0.0, &config);
        assert!(
            (h_zero / h_perfect - 3.0).abs() < 0.01,
            "ratio={}, expected 3.0",
            h_zero / h_perfect
        );
    }

    #[test]
    fn deterministic_death_check_reproducible() {
        let (s1, r1, h1) = perform_death_check(0.0001, 42, "golem-abc");
        let (s2, r2, h2) = perform_death_check(0.0001, 42, "golem-abc");
        assert_eq!(s1, s2);
        assert!((r1 - r2).abs() < f64::EPSILON);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hazard_baseline_at_zero() {
        let config = StochasticMortalityConfig::default();
        // At tick 0, fitness 1.0: h = (1e-6 + 1e-8 * exp(0)) * 1.0 = 1.01e-6.
        let h = compute_hazard_rate(0, 1.0, &config);
        assert!((h - 1.01e-6).abs() < 1e-8, "h(0)={h}, expected ~1.01e-6");
    }

    #[test]
    fn test_hazard_rate_parameters_default() {
        let config = StochasticMortalityConfig::default();
        assert!((config.base_hazard_rate - 1e-6).abs() < f64::EPSILON);
        assert!((config.age_hazard_coefficient - 1e-8).abs() < f64::EPSILON);
        assert!((config.aging_rate - 5e-5).abs() < f64::EPSILON);
        assert!((config.epistemic_hazard_multiplier - 3.0).abs() < f64::EPSILON);
        assert!((config.max_hazard_rate - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_epistemic_frailty_linear_interpolation() {
        let config = StochasticMortalityConfig::default();
        // epsilon(1.0) = 1.0, epsilon(0.5) = 2.0, epsilon(0.0) = 3.0.
        let h_1 = compute_hazard_rate(0, 1.0, &config);
        let h_05 = compute_hazard_rate(0, 0.5, &config);
        let h_0 = compute_hazard_rate(0, 0.0, &config);
        // Ratios should be 1:2:3.
        assert!((h_05 / h_1 - 2.0).abs() < 0.01);
        assert!((h_0 / h_1 - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_hazard_cap_enforced() {
        let config = StochasticMortalityConfig::default();
        // Very old tick with zero fitness → should be capped.
        let h = compute_hazard_rate(1_000_000, 0.0, &config);
        assert!(h <= config.max_hazard_rate + f64::EPSILON);
    }

    #[test]
    fn test_death_check_roll_range() {
        for tick in [0u64, 1, 100, 1000, 999_999] {
            let (_, roll, _) = perform_death_check(0.5, tick, "test-golem");
            assert!(
                roll >= 0.0 && roll <= 1.0,
                "roll={roll} out of range at tick={tick}"
            );
        }
    }

    #[test]
    fn test_survival_probability_cumulative_product() {
        let config = StochasticMortalityConfig::default();
        let mut state = StochasticMortalityState::new(config);
        let initial = state.survival_probability;
        assert!((initial - 1.0).abs() < f64::EPSILON);

        for tick in 0..10 {
            update_stochastic_mortality(&mut state, tick, 0.8, "test-golem");
        }
        // Survival probability should have decreased.
        assert!(state.survival_probability < initial);
        assert!(state.survival_probability > 0.0);
    }

    #[test]
    fn test_survival_probability_decreasing() {
        let config = StochasticMortalityConfig::default();
        let mut state = StochasticMortalityState::new(config);
        let mut prev = state.survival_probability;

        for tick in 0..100 {
            update_stochastic_mortality(&mut state, tick, 0.8, "monotone-test");
            assert!(
                state.survival_probability <= prev + f64::EPSILON,
                "survival increased at tick {tick}: {} > {prev}",
                state.survival_probability
            );
            prev = state.survival_probability;
        }
    }

    #[test]
    fn test_hazard_gompertz_monotonic_age() {
        let config = StochasticMortalityConfig::default();
        let ticks = [0u64, 50_000, 100_000, 150_000, 200_000];
        for pair in ticks.windows(2) {
            let h1 = compute_hazard_rate(pair[0], 0.8, &config);
            let h2 = compute_hazard_rate(pair[1], 0.8, &config);
            assert!(
                h2 >= h1,
                "hazard not monotonic: h({})={h1} > h({})={h2}",
                pair[0],
                pair[1]
            );
        }
    }
}

//! Composite vitality state and behavioral phase system.

use golem_core::cortical::BehavioralPhase;
use serde::{Deserialize, Serialize};

/// Configuration for vitality computation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VitalityConfig {
    /// Sigmoid center for economic component.
    pub economic_center: f64,
    /// Sigmoid steepness for economic component.
    pub economic_steepness: f64,
    /// Sigmoid center for epistemic component.
    pub epistemic_center: f64,
    /// Sigmoid steepness for epistemic component.
    pub epistemic_steepness: f64,
    /// Age drag coefficient.
    pub age_drag: f64,
    /// Reference lifespan in ticks.
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

/// Composite mortality state across all three clocks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VitalityState {
    /// Normalized economic balance in [0.0, 1.0].
    pub economic: f64,
    /// Rolling R-squared epistemic fitness in [0.0, 1.0].
    pub epistemic: f64,
    /// Age factor = tick / reference_lifespan (unbounded upward).
    pub age_factor: f64,
    /// Multiplicative composite in [0.0, 1.0].
    pub composite: f64,
    /// Current behavioral phase.
    pub phase: BehavioralPhase,
    /// Ticks spent in the current phase.
    pub ticks_in_phase: u64,
    /// Phase before the most recent transition (if any).
    pub previous_phase: Option<BehavioralPhase>,
    /// Unix epoch seconds when last computed.
    pub last_computed: u64,
}

impl Default for VitalityState {
    fn default() -> Self {
        Self {
            economic: 1.0,
            epistemic: 0.5,
            age_factor: 0.0,
            composite: 1.0,
            phase: BehavioralPhase::Thriving,
            ticks_in_phase: 0,
            previous_phase: None,
            last_computed: 0,
        }
    }
}

/// Standard sigmoid function: 1 / (1 + exp(-steepness * (x - center))).
#[inline]
pub fn sigmoid(x: f64, center: f64, steepness: f64) -> f64 {
    1.0 / (1.0 + (-steepness * (x - center)).exp())
}

/// Compute the age factor (linear, unbounded).
#[inline]
pub fn compute_age_factor(tick: u64, reference_lifespan: u64) -> f64 {
    tick as f64 / reference_lifespan as f64
}

/// Compute composite vitality from the three components.
pub fn compute_vitality(economic: f64, epistemic: f64, tick: u64, config: &VitalityConfig) -> f64 {
    let econ_component = sigmoid(economic, config.economic_center, config.economic_steepness);
    let epist_component = sigmoid(
        epistemic,
        config.epistemic_center,
        config.epistemic_steepness,
    );
    let age_factor = compute_age_factor(tick, config.reference_lifespan);
    let age_component = (1.0 - age_factor * config.age_drag).max(0.0);
    (econ_component * epist_component * age_component).clamp(0.0, 1.0)
}

/// Determine behavioral phase with hysteresis.
///
/// Downward transitions (toward Terminal) use raw thresholds.
/// Upward transitions (toward Thriving) require exceeding the threshold by `hysteresis`.
pub fn determine_phase(
    composite: f64,
    current_phase: BehavioralPhase,
    hysteresis: f64,
) -> BehavioralPhase {
    // Raw phase from thresholds (no hysteresis).
    let raw = BehavioralPhase::from_vitality(composite);
    let raw_ord = raw as u8;
    let cur_ord = current_phase as u8;

    // BehavioralPhase repr: Thriving=0, Stable=1, Conservation=2, Declining=3, Terminal=4.
    // "Downward" = toward Terminal = increasing ord.
    // "Upward" = toward Thriving = decreasing ord.

    if raw_ord >= cur_ord {
        // Same phase or downward: use raw thresholds directly.
        raw
    } else {
        // Upward transition: require composite >= threshold + hysteresis.
        // Thresholds for each phase (the floor composite to enter that phase going up).
        let up_threshold = match raw {
            BehavioralPhase::Thriving => 0.70 + hysteresis,
            BehavioralPhase::Stable => 0.50 + hysteresis,
            BehavioralPhase::Conservation => 0.30 + hysteresis,
            BehavioralPhase::Declining => 0.10 + hysteresis,
            BehavioralPhase::Terminal => return BehavioralPhase::Terminal,
        };
        if composite >= up_threshold {
            raw
        } else {
            current_phase
        }
    }
}

/// Update the vitality state for a new tick.
pub fn update_vitality_state(
    state: &mut VitalityState,
    economic: f64,
    epistemic: f64,
    tick: u64,
    config: &VitalityConfig,
    now_epoch_secs: u64,
) {
    state.economic = economic;
    state.epistemic = epistemic;
    state.age_factor = compute_age_factor(tick, config.reference_lifespan);
    state.composite = compute_vitality(economic, epistemic, tick, config);

    let new_phase = determine_phase(state.composite, state.phase, 0.05);
    if new_phase != state.phase {
        state.previous_phase = Some(state.phase);
        state.ticks_in_phase = 0;
        state.phase = new_phase;
    } else {
        state.ticks_in_phase += 1;
    }
    state.last_computed = now_epoch_secs;
}

/// Check the composite death condition.
pub fn is_dead(state: &VitalityState) -> bool {
    state.composite < 0.01 || state.economic == 0.0 || state.epistemic == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    #[test]
    fn vitality_state_composite_computation() {
        let config = VitalityConfig::default();
        let composite = compute_vitality(0.5, 0.5, 0, &config);
        // At tick 0, age_component = 1.0.
        // econ = sigmoid(0.5, 0.3, 10.0), epist = sigmoid(0.5, 0.4, 8.0)
        let expected_econ = sigmoid(0.5, 0.3, 10.0);
        let expected_epist = sigmoid(0.5, 0.4, 8.0);
        let expected = expected_econ * expected_epist * 1.0;
        assert!((composite - expected).abs() < EPS);
    }

    #[test]
    fn phase_transitions_follow_order() {
        // High composite → Thriving.
        assert_eq!(
            determine_phase(0.80, BehavioralPhase::Thriving, 0.05),
            BehavioralPhase::Thriving
        );
        // Low composite → Terminal.
        assert_eq!(
            determine_phase(0.05, BehavioralPhase::Thriving, 0.05),
            BehavioralPhase::Terminal
        );
    }

    #[test]
    fn phase_hysteresis_prevents_oscillation() {
        // From Conservation, composite 0.52 is above Stable raw threshold (0.50),
        // but below Stable + hysteresis (0.55) for upward transition.
        let phase = determine_phase(0.52, BehavioralPhase::Conservation, 0.05);
        assert_eq!(phase, BehavioralPhase::Conservation);

        // At 0.56 it should transition up.
        let phase = determine_phase(0.56, BehavioralPhase::Conservation, 0.05);
        assert_eq!(phase, BehavioralPhase::Stable);
    }

    #[test]
    fn test_sigmoid_center_returns_half() {
        let val = sigmoid(0.3, 0.3, 10.0);
        assert!((val - 0.5).abs() < EPS);
    }

    #[test]
    fn test_composite_multiplicative_no_compensation() {
        let config = VitalityConfig::default();
        // High economic, low epistemic — should be in Declining.
        let composite = compute_vitality(0.9, 0.2, 0, &config);
        assert!(composite < 0.3, "composite={composite}, expected < 0.3");
    }

    #[test]
    fn test_age_factor_linear_monotonic() {
        let ticks = [0, 50_000, 100_000, 150_000, 200_000, 300_000];
        for pair in ticks.windows(2) {
            let a = compute_age_factor(pair[0], 200_000);
            let b = compute_age_factor(pair[1], 200_000);
            assert!(
                b > a,
                "age_factor not monotonic at ticks {},{}",
                pair[0],
                pair[1]
            );
        }
        assert!((compute_age_factor(0, 200_000)).abs() < EPS);
        assert!((compute_age_factor(200_000, 200_000) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_phase_boundaries_exact() {
        assert_eq!(
            determine_phase(0.08, BehavioralPhase::Terminal, 0.05),
            BehavioralPhase::Terminal
        );
        assert_eq!(
            determine_phase(0.70, BehavioralPhase::Thriving, 0.05),
            BehavioralPhase::Thriving
        );
    }

    #[test]
    fn test_phase_hysteresis_upward_transition() {
        // From Stable, need >= 0.75 to go up to Thriving.
        assert_eq!(
            determine_phase(0.72, BehavioralPhase::Stable, 0.05),
            BehavioralPhase::Stable
        );
        assert_eq!(
            determine_phase(0.76, BehavioralPhase::Stable, 0.05),
            BehavioralPhase::Thriving
        );
    }

    #[test]
    fn test_economic_sigmoid_calibration() {
        let val_03 = sigmoid(0.3, 0.3, 10.0);
        assert!((val_03 - 0.5).abs() < 0.01);
        let val_01 = sigmoid(0.1, 0.3, 10.0);
        assert!((val_01 - 0.12).abs() < 0.02);
        let val_05 = sigmoid(0.5, 0.3, 10.0);
        assert!((val_05 - 0.88).abs() < 0.02);
    }

    #[test]
    fn test_epistemic_sigmoid_calibration() {
        let val_04 = sigmoid(0.4, 0.4, 8.0);
        assert!((val_04 - 0.5).abs() < 0.01);
        let val_02 = sigmoid(0.2, 0.4, 8.0);
        assert!((val_02 - 0.17).abs() < 0.03);
        let val_06 = sigmoid(0.6, 0.4, 8.0);
        assert!((val_06 - 0.83).abs() < 0.03);
    }

    #[test]
    fn test_age_drag_linear() {
        let config = VitalityConfig::default();
        let cases = [(0, 1.0), (100_000, 0.85), (200_000, 0.7)];
        for (tick, expected) in cases {
            let af = compute_age_factor(tick, config.reference_lifespan);
            let component = (1.0 - af * config.age_drag).max(0.0);
            assert!(
                (component - expected).abs() < EPS,
                "tick={tick}: got {component}, expected {expected}"
            );
        }
    }

    #[test]
    fn test_death_condition_composite_below_threshold() {
        let state = VitalityState {
            composite: 0.009,
            economic: 0.1,
            epistemic: 0.1,
            ..Default::default()
        };
        assert!(is_dead(&state));

        let state2 = VitalityState {
            composite: 0.01,
            economic: 0.1,
            epistemic: 0.1,
            ..Default::default()
        };
        assert!(!is_dead(&state2));
    }

    #[test]
    fn test_death_condition_any_clock_zero() {
        let state = VitalityState {
            composite: 0.5,
            economic: 0.0,
            epistemic: 0.5,
            ..Default::default()
        };
        assert!(is_dead(&state));

        let state2 = VitalityState {
            composite: 0.5,
            economic: 0.5,
            epistemic: 0.0,
            ..Default::default()
        };
        assert!(is_dead(&state2));
    }

    #[test]
    fn test_ticks_in_phase_reset_on_transition() {
        let config = VitalityConfig::default();
        let mut state = VitalityState::default();
        state.phase = BehavioralPhase::Stable;
        state.ticks_in_phase = 10;

        // Force a downward transition.
        update_vitality_state(&mut state, 0.05, 0.05, 100, &config, 1000);
        assert_eq!(state.ticks_in_phase, 0);
        assert_eq!(state.previous_phase, Some(BehavioralPhase::Stable));
    }

    #[test]
    fn test_ticks_in_phase_increment_stable() {
        let config = VitalityConfig::default();
        let mut state = VitalityState::default();
        state.phase = BehavioralPhase::Thriving;
        state.ticks_in_phase = 0;

        // Keep in Thriving.
        update_vitality_state(&mut state, 0.9, 0.9, 0, &config, 1000);
        assert_eq!(state.phase, BehavioralPhase::Thriving);
        assert_eq!(state.ticks_in_phase, 1);

        update_vitality_state(&mut state, 0.9, 0.9, 1, &config, 1001);
        assert_eq!(state.ticks_in_phase, 2);
    }
}

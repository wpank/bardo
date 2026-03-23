//! Maps `CorticalSnapshot` signals to `AtomicParameterBridge` CV outputs.

use golem_core::cortical::CorticalSnapshot;

use crate::params::{AtomicParameterBridge, cv_index};

/// Maps cortical signals to sonification CV parameters.
///
/// Reads a `CorticalSnapshot` (point-in-time copy of `CorticalState` atomics)
/// and writes derived values into the `AtomicParameterBridge` for the rack
/// processor thread to consume.
pub struct CvMapper;

impl CvMapper {
    /// Update the parameter bridge from a cortical snapshot.
    ///
    /// Current mappings:
    /// - `composite_vitality → master_level` (clamped 0.0-1.0)
    /// - `arousal → event_density` (clamped 0.0-1.0)
    pub fn update_from_snapshot(snapshot: &CorticalSnapshot, bridge: &AtomicParameterBridge) {
        let composite_vitality = compute_composite_vitality(snapshot);
        bridge.write(cv_index::MASTER_LEVEL, composite_vitality.clamp(0.0, 1.0));
        bridge.write(cv_index::EVENT_DENSITY, snapshot.arousal.clamp(0.0, 1.0));
    }
}

/// Derives composite vitality from the three vitality signals.
/// Uses the same weighted average as the mortality subsystem.
fn compute_composite_vitality(snapshot: &CorticalSnapshot) -> f32 {
    // Weighted average: economic 40%, epistemic 30%, stochastic 30%
    snapshot.economic_vitality * 0.4
        + snapshot.epistemic_vitality * 0.3
        + snapshot.stochastic_vitality * 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_snapshot() -> CorticalSnapshot {
        CorticalSnapshot {
            pleasure: 0.0,
            arousal: 0.0,
            dominance: 0.0,
            primary_emotion: 0,
            aggregate_accuracy: 0.0,
            accuracy_trend: 0,
            surprise_rate: 0.0,
            pending_predictions: 0,
            universe_size: 0,
            active_count: 0,
            watched_count: 0,
            regime: 0,
            regime_confidence: 0.0,
            gas_gwei: 0.0,
            economic_vitality: 0.0,
            epistemic_vitality: 0.0,
            stochastic_vitality: 0.0,
            behavioral_phase: 0,
            inference_budget_remaining: 0.0,
            current_tier: 0,
            creative_mode: 0,
            fragments_captured: 0,
            last_novel_prediction_tick: 0,
            compounding_momentum: 0.0,
        }
    }

    #[test]
    fn zero_vitality_maps_to_zero_master() {
        let bridge = AtomicParameterBridge::new();
        let snap = default_snapshot();
        CvMapper::update_from_snapshot(&snap, &bridge);
        assert!((bridge.read(cv_index::MASTER_LEVEL)).abs() < f32::EPSILON);
    }

    #[test]
    fn nonzero_vitality_maps_to_positive_master() {
        let bridge = AtomicParameterBridge::new();
        let mut snap = default_snapshot();
        snap.economic_vitality = 0.8;
        snap.epistemic_vitality = 0.6;
        snap.stochastic_vitality = 0.4;
        CvMapper::update_from_snapshot(&snap, &bridge);
        let master = bridge.read(cv_index::MASTER_LEVEL);
        assert!(master > 0.0);
        // 0.8*0.4 + 0.6*0.3 + 0.4*0.3 = 0.32 + 0.18 + 0.12 = 0.62
        assert!((master - 0.62).abs() < 1e-5);
    }

    #[test]
    fn arousal_maps_to_event_density() {
        let bridge = AtomicParameterBridge::new();
        let mut snap = default_snapshot();
        snap.arousal = 0.75;
        CvMapper::update_from_snapshot(&snap, &bridge);
        assert!((bridge.read(cv_index::EVENT_DENSITY) - 0.75).abs() < f32::EPSILON);
    }
}

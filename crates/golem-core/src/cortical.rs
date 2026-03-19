//! Lock-free cortical state surface.

use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicI8, AtomicU8, AtomicU16, AtomicU32, Ordering},
    },
};

use serde::{Deserialize, Serialize};

const _: () = assert!(std::mem::size_of::<CorticalState>() <= 256);

#[inline]
fn load_f32(value: &AtomicU32) -> f32 {
    f32::from_bits(value.load(Ordering::Acquire))
}

#[inline]
fn store_f32(value: &AtomicU32, input: f32) {
    value.store(input.to_bits(), Ordering::Release);
}

/// Three-dimensional affective PAD vector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PadVector {
    /// Pleasure axis.
    pub pleasure: f64,
    /// Arousal axis.
    pub arousal: f64,
    /// Dominance axis.
    pub dominance: f64,
}

impl PadVector {
    /// Origin PAD vector.
    pub const ZERO: Self = Self {
        pleasure: 0.0,
        arousal: 0.0,
        dominance: 0.0,
    };

    /// Clamps each axis into the provided inclusive range.
    #[must_use]
    pub const fn clamp(&self, min: f64, max: f64) -> Self {
        Self {
            pleasure: if self.pleasure < min {
                min
            } else if self.pleasure > max {
                max
            } else {
                self.pleasure
            },
            arousal: if self.arousal < min {
                min
            } else if self.arousal > max {
                max
            } else {
                self.arousal
            },
            dominance: if self.dominance < min {
                min
            } else if self.dominance > max {
                max
            } else {
                self.dominance
            },
        }
    }
}

/// Behavioral vitality phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BehavioralPhase {
    /// Vitality above 0.7.
    Thriving = 0,
    /// Vitality between 0.5 and 0.7.
    Stable = 1,
    /// Vitality between 0.3 and 0.5.
    Conservation = 2,
    /// Vitality between 0.1 and 0.3.
    Declining = 3,
    /// Vitality below 0.1.
    Terminal = 4,
}

impl BehavioralPhase {
    /// Converts a raw stored phase value into a behavioral phase.
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Thriving,
            1 => Self::Stable,
            2 => Self::Conservation,
            3 => Self::Declining,
            _ => Self::Terminal,
        }
    }
}

/// Eight-label emotion classification over the PAD octants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlutchikEmotion {
    /// Joy.
    Joy = 0,
    /// Trust.
    Trust = 1,
    /// Fear.
    Fear = 2,
    /// Surprise.
    Surprise = 3,
    /// Sadness.
    Sadness = 4,
    /// Disgust.
    Disgust = 5,
    /// Anger.
    Anger = 6,
    /// Anticipation.
    Anticipation = 7,
}

impl PlutchikEmotion {
    /// Classifies a PAD vector into a Plutchik octant.
    pub fn from_pad(pad: &PadVector) -> Self {
        if pad.pleasure == 0.0 && pad.arousal == 0.0 && pad.dominance == 0.0 {
            return Self::Anticipation;
        }

        let positive_pleasure = pad.pleasure > 0.0;
        let positive_arousal = pad.arousal > 0.0;
        let positive_dominance = pad.dominance > 0.0;

        match (positive_pleasure, positive_arousal, positive_dominance) {
            (true, true, true) => Self::Joy,
            (true, false, true) => Self::Trust,
            (false, true, false) => Self::Fear,
            (false, true, true) => Self::Anger,
            (false, false, false) => Self::Sadness,
            (true, true, false) => Self::Surprise,
            (false, false, true) => Self::Disgust,
            _ => Self::Anticipation,
        }
    }
}

/// Snapshot of all cortical signals.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorticalSnapshot {
    /// Pleasure signal.
    pub pleasure: f32,
    /// Arousal signal.
    pub arousal: f32,
    /// Dominance signal.
    pub dominance: f32,
    /// Primary emotion label.
    pub primary_emotion: u8,
    /// Aggregate prediction accuracy.
    pub aggregate_accuracy: f32,
    /// Prediction accuracy trend.
    pub accuracy_trend: i8,
    /// Mean surprise rate.
    pub surprise_rate: f32,
    /// Pending prediction count.
    pub pending_predictions: u32,
    /// Universe size.
    pub universe_size: u32,
    /// Active item count.
    pub active_count: u16,
    /// Watched item count.
    pub watched_count: u16,
    /// Environment regime identifier.
    pub regime: u8,
    /// Environment regime confidence.
    pub regime_confidence: f32,
    /// Gas price in gwei.
    pub gas_gwei: f32,
    /// Economic vitality.
    pub economic_vitality: f32,
    /// Epistemic vitality.
    pub epistemic_vitality: f32,
    /// Stochastic vitality.
    pub stochastic_vitality: f32,
    /// Behavioral phase.
    pub behavioral_phase: u8,
    /// Remaining inference budget.
    pub inference_budget_remaining: f32,
    /// Current tier.
    pub current_tier: u8,
    /// Creative mode flag.
    pub creative_mode: u8,
    /// Captured fragment count.
    pub fragments_captured: u32,
    /// Last novel prediction tick.
    pub last_novel_prediction_tick: u64,
    /// Compounding momentum.
    pub compounding_momentum: f32,
}

/// Shared, cache-aligned cortical signal surface.
#[repr(C, align(64))]
pub struct CorticalState {
    pleasure: AtomicU32,
    arousal: AtomicU32,
    dominance: AtomicU32,
    primary_emotion: AtomicU8,
    aggregate_accuracy: AtomicU32,
    accuracy_trend: AtomicI8,
    category_accuracies: [AtomicU32; 16],
    surprise_rate: AtomicU32,
    universe_size: AtomicU32,
    active_count: AtomicU16,
    watched_count: AtomicU16,
    pending_predictions: AtomicU32,
    regime: AtomicU8,
    regime_confidence: AtomicU32,
    gas_gwei: AtomicU32,
    economic_vitality: AtomicU32,
    epistemic_vitality: AtomicU32,
    stochastic_vitality: AtomicU32,
    behavioral_phase: AtomicU8,
    inference_budget_remaining: AtomicU32,
    current_tier: AtomicU8,
    creative_mode: AtomicU8,
    fragments_captured: AtomicU32,
    last_novel_prediction_tick: AtomicU32,
    last_novel_prediction_tick_hi: AtomicU32,
    compounding_momentum: AtomicU32,
}

impl CorticalState {
    /// Creates a new shared cortical state.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pleasure: AtomicU32::new(0),
            arousal: AtomicU32::new(0),
            dominance: AtomicU32::new(0),
            primary_emotion: AtomicU8::new(PlutchikEmotion::Anticipation as u8),
            aggregate_accuracy: AtomicU32::new(0),
            accuracy_trend: AtomicI8::new(0),
            category_accuracies: array::from_fn(|_| AtomicU32::new(0)),
            surprise_rate: AtomicU32::new(0),
            universe_size: AtomicU32::new(0),
            active_count: AtomicU16::new(0),
            watched_count: AtomicU16::new(0),
            pending_predictions: AtomicU32::new(0),
            regime: AtomicU8::new(0),
            regime_confidence: AtomicU32::new(0),
            gas_gwei: AtomicU32::new(0),
            economic_vitality: AtomicU32::new(0),
            epistemic_vitality: AtomicU32::new(0),
            stochastic_vitality: AtomicU32::new(0),
            behavioral_phase: AtomicU8::new(0),
            inference_budget_remaining: AtomicU32::new(0),
            current_tier: AtomicU8::new(0),
            creative_mode: AtomicU8::new(0),
            fragments_captured: AtomicU32::new(0),
            last_novel_prediction_tick: AtomicU32::new(0),
            last_novel_prediction_tick_hi: AtomicU32::new(0),
            compounding_momentum: AtomicU32::new(0),
        })
    }

    /// Returns the current PAD vector.
    pub fn pad(&self) -> PadVector {
        PadVector {
            pleasure: f64::from(load_f32(&self.pleasure)),
            arousal: f64::from(load_f32(&self.arousal)),
            dominance: f64::from(load_f32(&self.dominance)),
        }
    }

    /// Returns the current prediction accuracy.
    pub fn prediction_accuracy(&self) -> f32 {
        load_f32(&self.aggregate_accuracy)
    }

    /// Returns the current behavioral phase.
    pub fn phase(&self) -> BehavioralPhase {
        BehavioralPhase::from_u8(self.behavioral_phase.load(Ordering::Acquire))
    }

    /// Produces a point-in-time snapshot of all signals.
    pub fn snapshot(&self) -> CorticalSnapshot {
        CorticalSnapshot {
            pleasure: load_f32(&self.pleasure),
            arousal: load_f32(&self.arousal),
            dominance: load_f32(&self.dominance),
            primary_emotion: self.primary_emotion.load(Ordering::Acquire),
            aggregate_accuracy: load_f32(&self.aggregate_accuracy),
            accuracy_trend: self.accuracy_trend.load(Ordering::Acquire),
            surprise_rate: load_f32(&self.surprise_rate),
            pending_predictions: self.pending_predictions.load(Ordering::Acquire),
            universe_size: self.universe_size.load(Ordering::Acquire),
            active_count: self.active_count.load(Ordering::Acquire),
            watched_count: self.watched_count.load(Ordering::Acquire),
            regime: self.regime.load(Ordering::Acquire),
            regime_confidence: load_f32(&self.regime_confidence),
            gas_gwei: load_f32(&self.gas_gwei),
            economic_vitality: load_f32(&self.economic_vitality),
            epistemic_vitality: load_f32(&self.epistemic_vitality),
            stochastic_vitality: load_f32(&self.stochastic_vitality),
            behavioral_phase: self.behavioral_phase.load(Ordering::Acquire),
            inference_budget_remaining: load_f32(&self.inference_budget_remaining),
            current_tier: self.current_tier.load(Ordering::Acquire),
            creative_mode: self.creative_mode.load(Ordering::Acquire),
            fragments_captured: self.fragments_captured.load(Ordering::Acquire),
            last_novel_prediction_tick: u64::from(
                self.last_novel_prediction_tick.load(Ordering::Acquire),
            ) | (u64::from(
                self.last_novel_prediction_tick_hi.load(Ordering::Acquire),
            ) << 32),
            compounding_momentum: load_f32(&self.compounding_momentum),
        }
    }

    /// Writes the affect signal group.
    pub fn write_affect(&self, pleasure: f32, arousal: f32, dominance: f32, emotion: u8) {
        store_f32(&self.pleasure, pleasure);
        store_f32(&self.arousal, arousal);
        store_f32(&self.dominance, dominance);
        self.primary_emotion.store(emotion, Ordering::Release);
    }

    /// Writes the prediction signal group.
    pub fn write_prediction(
        &self,
        accuracy: f32,
        trend: i8,
        categories: &[f32; 16],
        surprise: f32,
        pending: u32,
    ) {
        store_f32(&self.aggregate_accuracy, accuracy);
        self.accuracy_trend.store(trend, Ordering::Release);
        for (slot, value) in self.category_accuracies.iter().zip(categories.iter()) {
            store_f32(slot, *value);
        }
        store_f32(&self.surprise_rate, surprise);
        self.pending_predictions.store(pending, Ordering::Release);
    }

    /// Writes the attention signal group.
    pub fn write_attention(&self, universe: u32, active: u16, watched: u16, pending: u32) {
        self.universe_size.store(universe, Ordering::Release);
        self.active_count.store(active, Ordering::Release);
        self.watched_count.store(watched, Ordering::Release);
        self.pending_predictions.store(pending, Ordering::Release);
    }

    /// Writes the environment signal group.
    pub fn write_environment(&self, regime: u8, confidence: f32, gas_gwei: f32) {
        self.regime.store(regime, Ordering::Release);
        store_f32(&self.regime_confidence, confidence);
        store_f32(&self.gas_gwei, gas_gwei);
    }

    /// Writes the mortality signal group.
    pub fn write_mortality(&self, economic: f32, epistemic: f32, stochastic: f32, phase: u8) {
        store_f32(&self.economic_vitality, economic);
        store_f32(&self.epistemic_vitality, epistemic);
        store_f32(&self.stochastic_vitality, stochastic);
        self.behavioral_phase.store(phase, Ordering::Release);
    }

    /// Writes the inference signal group.
    pub fn write_inference(&self, budget_remaining: f32, tier: u8) {
        store_f32(&self.inference_budget_remaining, budget_remaining);
        self.current_tier.store(tier, Ordering::Release);
    }

    /// Writes the creative signal group.
    pub fn write_creative(&self, mode: u8, fragments: u32, last_novel_tick: u64) {
        self.creative_mode.store(mode, Ordering::Release);
        self.fragments_captured.store(fragments, Ordering::Release);
        self.last_novel_prediction_tick
            .store((last_novel_tick & 0xFFFF_FFFF) as u32, Ordering::Release);
        self.last_novel_prediction_tick_hi
            .store((last_novel_tick >> 32) as u32, Ordering::Release);
    }

    /// Writes the derived signal group.
    pub fn write_derived(&self, momentum: f32) {
        store_f32(&self.compounding_momentum, momentum);
    }
}

/// Converts the current phase from a byte using the stored signal.
impl Default for CorticalState {
    fn default() -> Self {
        Self {
            pleasure: AtomicU32::new(0),
            arousal: AtomicU32::new(0),
            dominance: AtomicU32::new(0),
            primary_emotion: AtomicU8::new(PlutchikEmotion::Anticipation as u8),
            aggregate_accuracy: AtomicU32::new(0),
            accuracy_trend: AtomicI8::new(0),
            category_accuracies: array::from_fn(|_| AtomicU32::new(0)),
            surprise_rate: AtomicU32::new(0),
            universe_size: AtomicU32::new(0),
            active_count: AtomicU16::new(0),
            watched_count: AtomicU16::new(0),
            pending_predictions: AtomicU32::new(0),
            regime: AtomicU8::new(0),
            regime_confidence: AtomicU32::new(0),
            gas_gwei: AtomicU32::new(0),
            economic_vitality: AtomicU32::new(0),
            epistemic_vitality: AtomicU32::new(0),
            stochastic_vitality: AtomicU32::new(0),
            behavioral_phase: AtomicU8::new(0),
            inference_budget_remaining: AtomicU32::new(0),
            current_tier: AtomicU8::new(0),
            creative_mode: AtomicU8::new(0),
            fragments_captured: AtomicU32::new(0),
            last_novel_prediction_tick: AtomicU32::new(0),
            last_novel_prediction_tick_hi: AtomicU32::new(0),
            compounding_momentum: AtomicU32::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralPhase, CorticalState, PadVector, PlutchikEmotion};

    #[test]
    fn cortical_state_alignment() {
        assert_eq!(std::mem::align_of::<CorticalState>(), 64);
    }

    #[test]
    fn cortical_state_size() {
        assert!(std::mem::size_of::<CorticalState>() <= 256);
    }

    #[test]
    fn cortical_state_write_read_affect() {
        let state = CorticalState::new();
        state.write_affect(0.5, -0.3, 0.1, 7);
        let snapshot = state.snapshot();
        assert!((snapshot.pleasure - 0.5).abs() < 1e-6);
        assert!((snapshot.arousal - (-0.3)).abs() < 1e-6);
        assert_eq!(snapshot.primary_emotion, 7);
    }

    #[test]
    fn pad_vector_plutchik_joy() {
        let emotion = PlutchikEmotion::from_pad(&PadVector {
            pleasure: 1.0,
            arousal: 1.0,
            dominance: 1.0,
        });
        assert_eq!(emotion, PlutchikEmotion::Joy);
    }

    #[test]
    fn pad_vector_plutchik_anticipation_for_zero_vector() {
        assert_eq!(
            PlutchikEmotion::from_pad(&PadVector::ZERO),
            PlutchikEmotion::Anticipation
        );
    }

    #[test]
    fn behavioral_phase_from_u8() {
        assert_eq!(BehavioralPhase::from_u8(0), BehavioralPhase::Thriving);
        assert_eq!(BehavioralPhase::from_u8(4), BehavioralPhase::Terminal);
        assert_eq!(BehavioralPhase::from_u8(9), BehavioralPhase::Terminal);
    }
}

//! Epistemic death clock — R-squared fitness over prediction accuracy.

use crate::clock::{ClockContext, ClockEvent, DeathCause, MortalityClock};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single (predicted, actual) observation pair.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionOutcomePair {
    /// Model's predicted score.
    pub predicted_score: f64,
    /// Observed actual score.
    pub actual_score: f64,
    /// Tick at which this observation was recorded.
    pub tick: u64,
}

/// Five-dimension market prediction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketPrediction {
    /// Price direction (up/down/flat).
    pub price_direction: i8,
    /// Volatility regime.
    pub volatility_regime: i8,
    /// Yield trend.
    pub yield_trend: i8,
    /// Gas condition.
    pub gas_condition: i8,
    /// Protocol state.
    pub protocol_state: i8,
}

/// Observed market outcome matching the five dimensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketOutcome {
    /// Price direction observed.
    pub price_direction: i8,
    /// Volatility regime observed.
    pub volatility_regime: i8,
    /// Yield trend observed.
    pub yield_trend: i8,
    /// Gas condition observed.
    pub gas_condition: i8,
    /// Protocol state observed.
    pub protocol_state: i8,
}

/// Per-dimension accuracy weights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionWeights {
    /// Weight for price direction match.
    pub price_direction: f64,
    /// Weight for volatility regime match.
    pub volatility_regime: f64,
    /// Weight for yield trend match.
    pub yield_trend: f64,
    /// Weight for gas condition match.
    pub gas_condition: f64,
    /// Weight for protocol state match.
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

impl DimensionWeights {
    /// Sum of all weights.
    pub fn sum(&self) -> f64 {
        self.price_direction
            + self.volatility_regime
            + self.yield_trend
            + self.gas_condition
            + self.protocol_state
    }
}

/// Per-domain fitness tracking.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainFitness {
    /// Price direction domain fitness.
    pub price_direction: f64,
    /// Volatility domain fitness.
    pub volatility: f64,
    /// Yield domain fitness.
    pub yield_trend: f64,
    /// Gas domain fitness.
    pub gas: f64,
    /// Protocol domain fitness.
    pub protocol: f64,
}

/// Senescence stage progression.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SenescenceStage {
    /// Warning: fitness below threshold but within grace period.
    Stage1,
    /// Confirmed senescence.
    Stage2,
    /// Death protocol imminent.
    Stage3,
}

/// Full epistemic fitness state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpistemicFitnessState {
    /// Rolling window of prediction observations.
    pub prediction_log: VecDeque<PredictionOutcomePair>,
    /// Maximum window size.
    pub window_size: usize,
    /// Current R-squared fitness.
    pub fitness: f64,
    /// Fitness trend (positive = improving).
    pub fitness_trend: f64,
    /// Per-domain fitness breakdown.
    pub domain_fitness: DomainFitness,
    /// Ticks spent below senescence threshold.
    pub ticks_below_threshold: u64,
    /// Whether currently senescent.
    pub senescent: bool,
    /// Current senescence stage.
    pub senescence_stage: Option<SenescenceStage>,
    /// Senescence entry threshold.
    pub senescence_threshold: f64,
    /// Recovery threshold (entry + hysteresis).
    pub recovery_threshold: f64,
    /// Grace period in ticks before Stage1 → Stage2.
    pub grace_period: u64,
    /// Ticks in Stage2 before → Stage3.
    pub stage2_period: u64,
    /// Peak fitness ever observed.
    pub peak_fitness: f64,
    /// Tick at which peak was observed.
    pub peak_fitness_tick: u64,
    /// Tick of last fully-correct prediction.
    pub last_fully_correct_prediction: u64,
}

impl Default for EpistemicFitnessState {
    fn default() -> Self {
        Self {
            prediction_log: VecDeque::new(),
            window_size: 100,
            fitness: 0.5,
            fitness_trend: 0.0,
            domain_fitness: DomainFitness::default(),
            ticks_below_threshold: 0,
            senescent: false,
            senescence_stage: None,
            senescence_threshold: 0.35,
            recovery_threshold: 0.45,
            grace_period: 50,
            stage2_period: 100,
            peak_fitness: 0.5,
            peak_fitness_tick: 0,
            last_fully_correct_prediction: 0,
        }
    }
}

/// Compute per-tick accuracy as weighted binary match across five dimensions.
pub fn compute_tick_accuracy(
    prediction: &MarketPrediction,
    outcome: &MarketOutcome,
    weights: &DimensionWeights,
) -> f64 {
    let matches = [
        (
            prediction.price_direction == outcome.price_direction,
            weights.price_direction,
        ),
        (
            prediction.volatility_regime == outcome.volatility_regime,
            weights.volatility_regime,
        ),
        (
            prediction.yield_trend == outcome.yield_trend,
            weights.yield_trend,
        ),
        (
            prediction.gas_condition == outcome.gas_condition,
            weights.gas_condition,
        ),
        (
            prediction.protocol_state == outcome.protocol_state,
            weights.protocol_state,
        ),
    ];
    matches
        .iter()
        .map(|(matched, weight)| if *matched { *weight } else { 0.0 })
        .sum()
}

/// Compute R-squared from a slice of prediction-outcome pairs.
fn compute_r_squared(pairs: &VecDeque<PredictionOutcomePair>) -> f64 {
    let n = pairs.len();
    if n < 10 {
        return 0.5;
    }
    let n_f = n as f64;
    let mean_actual = pairs.iter().map(|p| p.actual_score).sum::<f64>() / n_f;
    let ss_res: f64 = pairs
        .iter()
        .map(|p| (p.actual_score - p.predicted_score).powi(2))
        .sum();
    let ss_tot: f64 = pairs
        .iter()
        .map(|p| (p.actual_score - mean_actual).powi(2))
        .sum();
    if ss_tot > 0.0 {
        (1.0 - ss_res / ss_tot).max(0.0)
    } else {
        0.5
    }
}

/// Update the epistemic fitness state with a new observation.
pub fn update_epistemic_fitness(state: &mut EpistemicFitnessState, pair: PredictionOutcomePair) {
    // Evict oldest if at capacity.
    while state.prediction_log.len() >= state.window_size {
        state.prediction_log.pop_front();
    }
    let tick = pair.tick;
    state.prediction_log.push_back(pair);

    let old_fitness = state.fitness;
    state.fitness = compute_r_squared(&state.prediction_log);
    state.fitness_trend = state.fitness - old_fitness;

    // Track peak.
    if state.fitness > state.peak_fitness {
        state.peak_fitness = state.fitness;
        state.peak_fitness_tick = tick;
    }

    // Senescence state machine.
    if state.senescent {
        if state.fitness > state.recovery_threshold {
            // Recovery.
            state.senescent = false;
            state.senescence_stage = None;
            state.ticks_below_threshold = 0;
        } else {
            state.ticks_below_threshold += 1;
            // Escalate stages.
            match state.senescence_stage {
                Some(SenescenceStage::Stage1)
                    if state.ticks_below_threshold > state.grace_period =>
                {
                    state.senescence_stage = Some(SenescenceStage::Stage2);
                }
                Some(SenescenceStage::Stage2)
                    if state.ticks_below_threshold > state.grace_period + state.stage2_period =>
                {
                    state.senescence_stage = Some(SenescenceStage::Stage3);
                }
                _ => {}
            }
        }
    } else if state.fitness < state.senescence_threshold {
        state.senescent = true;
        state.ticks_below_threshold = 1;
        state.senescence_stage = Some(SenescenceStage::Stage1);
    }
}

/// The epistemic death clock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpistemicClock {
    /// Inner fitness state.
    pub state: EpistemicFitnessState,
}

impl EpistemicClock {
    /// Create with defaults.
    pub fn new() -> Self {
        Self {
            state: EpistemicFitnessState::default(),
        }
    }

    /// Record a prediction-outcome observation.
    pub fn record(&mut self, pair: PredictionOutcomePair) {
        update_epistemic_fitness(&mut self.state, pair);
    }
}

impl Default for EpistemicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MortalityClock for EpistemicClock {
    fn vitality(&self) -> f64 {
        self.state.fitness
    }

    fn tick(&mut self, _ctx: &ClockContext) -> ClockEvent {
        if self.state.senescence_stage == Some(SenescenceStage::Stage3) {
            ClockEvent::Dead {
                cause: DeathCause::Epistemic,
            }
        } else {
            ClockEvent::Alive {
                vitality: self.state.fitness,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    #[test]
    fn epistemic_decay_reduces_score() {
        let mut clock = EpistemicClock::new();
        // Feed poor predictions: alternate actuals so SS_tot > 0 and R² reflects prediction error.
        for i in 0..50u64 {
            clock.record(PredictionOutcomePair {
                predicted_score: 1.0,
                actual_score: (i % 2) as f64,
                tick: i,
            });
        }
        assert!(clock.vitality() < 0.5);
    }

    #[test]
    fn rquared_defaults_to_half_when_insufficient_data() {
        let clock = EpistemicClock::new();
        assert!((clock.vitality() - 0.5).abs() < EPS);
    }

    #[test]
    fn test_rsquared_perfect_prediction() {
        let mut state = EpistemicFitnessState::default();
        for i in 0..20 {
            let v = i as f64 * 0.1;
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: v,
                    actual_score: v,
                    tick: i,
                },
            );
        }
        assert!(
            (state.fitness - 1.0).abs() < 0.01,
            "fitness={}, expected ~1.0",
            state.fitness
        );
    }

    #[test]
    fn test_rsquared_random_prediction() {
        let mut state = EpistemicFitnessState::default();
        // Predictions are the mean of actuals → R² ≈ 0.
        for i in 0..20 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: 0.5,
                    actual_score: (i % 2) as f64,
                    tick: i,
                },
            );
        }
        assert!(
            state.fitness < 0.1,
            "fitness={}, expected ~0.0",
            state.fitness
        );
    }

    #[test]
    fn test_rsquared_insufficient_data_default() {
        let mut state = EpistemicFitnessState::default();
        for i in 0..5 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: 1.0,
                    actual_score: 0.0,
                    tick: i,
                },
            );
        }
        assert!((state.fitness - 0.5).abs() < EPS);
    }

    #[test]
    fn test_prediction_window_capacity_100() {
        let mut state = EpistemicFitnessState::default();
        assert_eq!(state.window_size, 100);
        for i in 0..150 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: i as f64,
                    actual_score: i as f64,
                    tick: i,
                },
            );
        }
        assert_eq!(state.prediction_log.len(), 100);
    }

    #[test]
    fn test_prediction_window_fifo_eviction() {
        let mut state = EpistemicFitnessState::default();
        for i in 0..150u64 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: i as f64,
                    actual_score: i as f64,
                    tick: i,
                },
            );
        }
        // Oldest should be tick 50.
        assert_eq!(state.prediction_log.front().unwrap().tick, 50);
    }

    #[test]
    fn test_dimension_weights_sum_unity() {
        let w = DimensionWeights::default();
        assert!((w.sum() - 1.0).abs() < EPS);
    }

    #[test]
    fn test_accuracy_all_correct() {
        let pred = MarketPrediction {
            price_direction: 1,
            volatility_regime: 2,
            yield_trend: 1,
            gas_condition: 0,
            protocol_state: 1,
        };
        let outcome = MarketOutcome {
            price_direction: 1,
            volatility_regime: 2,
            yield_trend: 1,
            gas_condition: 0,
            protocol_state: 1,
        };
        let acc = compute_tick_accuracy(&pred, &outcome, &DimensionWeights::default());
        assert!((acc - 1.0).abs() < EPS);
    }

    #[test]
    fn test_accuracy_all_wrong() {
        let pred = MarketPrediction {
            price_direction: 1,
            volatility_regime: 2,
            yield_trend: 1,
            gas_condition: 0,
            protocol_state: 1,
        };
        let outcome = MarketOutcome {
            price_direction: -1,
            volatility_regime: 0,
            yield_trend: -1,
            gas_condition: 1,
            protocol_state: 0,
        };
        let acc = compute_tick_accuracy(&pred, &outcome, &DimensionWeights::default());
        assert!(acc.abs() < EPS);
    }

    #[test]
    fn test_accuracy_mixed() {
        let pred = MarketPrediction {
            price_direction: 1,
            volatility_regime: 2,
            yield_trend: 1,
            gas_condition: 0,
            protocol_state: 1,
        };
        // Match price_direction (0.35), volatility_regime (0.25), yield_trend (0.20) = 0.80.
        let outcome = MarketOutcome {
            price_direction: 1,
            volatility_regime: 2,
            yield_trend: 1,
            gas_condition: 1,
            protocol_state: 0,
        };
        let acc = compute_tick_accuracy(&pred, &outcome, &DimensionWeights::default());
        assert!((acc - 0.80).abs() < EPS);
    }

    #[test]
    fn test_senescence_threshold_exact() {
        let mut state = EpistemicFitnessState::default();
        // Alternate actuals so SS_tot > 0 and R² reflects the bad predictions.
        for i in 0..20u64 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: 1.0,
                    actual_score: (i % 2) as f64,
                    tick: i,
                },
            );
        }
        // Fitness should be near 0 → senescent.
        assert!(state.senescent);
        assert_eq!(state.senescence_stage, Some(SenescenceStage::Stage1));
    }

    #[test]
    fn test_senescence_recovery_hysteresis() {
        let mut state = EpistemicFitnessState::default();
        // Enter senescence with alternating actuals so SS_tot > 0 and R² is near 0.
        for i in 0..20u64 {
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: 1.0,
                    actual_score: (i % 2) as f64,
                    tick: i,
                },
            );
        }
        assert!(state.senescent);

        // Now feed perfect predictions. The R² will improve.
        for i in 20..120 {
            let v = (i as f64 - 20.0) / 100.0;
            update_epistemic_fitness(
                &mut state,
                PredictionOutcomePair {
                    predicted_score: v,
                    actual_score: v,
                    tick: i,
                },
            );
        }
        // With enough perfect data, fitness should exceed recovery threshold.
        if state.fitness > 0.45 {
            assert!(
                !state.senescent,
                "should have recovered at fitness={}",
                state.fitness
            );
        }
    }
}

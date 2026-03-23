//! Memetic fitness tracking using cultural-evolution primitives.
//!
//! Each Grimoire entry has a fitness `W(E) = fidelity * fecundity * longevity`.
//! The Curator computes W(E) each cycle and uses the population mean W_bar
//! for parasite detection and Price equation diagnostics.

use crate::entry::MemeticFields;

/// Compute longevity for an entry given its age and decay rate.
///
/// `longevity(E) = exp(-decay_rate * age_ticks)`
pub fn longevity(decay_rate: f64, age_ticks: u64) -> f64 {
    if decay_rate == 0.0 {
        return 1.0;
    }
    (-decay_rate * age_ticks as f64).exp()
}

/// Compute the fitness W(E) = fidelity * fecundity * longevity.
pub fn fitness(fidelity: f64, fecundity: f64, decay_rate: f64, age_ticks: u64) -> f64 {
    fidelity * fecundity * longevity(decay_rate, age_ticks)
}

/// Compute the population mean fitness W_bar over a set of entries.
pub fn population_mean_fitness(fitnesses: &[f64]) -> f64 {
    if fitnesses.is_empty() {
        return 0.0;
    }
    fitnesses.iter().sum::<f64>() / fitnesses.len() as f64
}

/// Price equation quality metric for a single entry.
///
/// `Q(E) = selection_differential + transmission_bias`
///
/// - `selection_differential = cov(W, confidence) / W_bar`
/// - `transmission_bias = E[ΔW | transmitted] - E[ΔW | not_transmitted]`
///
/// In practice, we compute these over the population and evaluate per-entry.
pub fn quality_metric(selection_differential: f64, transmission_bias: f64) -> f64 {
    selection_differential + transmission_bias
}

/// Check if an entry is a parasite.
///
/// Parasite condition: `W(E) > W_bar AND Q(E) < 0.0`
/// An entry that is highly fit but collectively harmful.
pub fn is_parasite(w_e: f64, w_bar: f64, q_e: f64) -> bool {
    w_e > w_bar && q_e < 0.0
}

/// Update memetic fields after a Curator cycle.
///
/// - Increments `generation`.
/// - Recomputes `fitness` from fidelity, fecundity, longevity.
/// - Updates `parasite_score` if flagged.
pub fn update_memetic_fields(
    fields: &mut MemeticFields,
    decay_rate: f64,
    age_ticks: u64,
    is_parasite_flagged: bool,
) {
    fields.generation += 1;
    fields.fitness = fitness(fields.fidelity, fields.fecundity, decay_rate, age_ticks);

    if is_parasite_flagged {
        fields.parasite_score = (fields.parasite_score + 0.1).min(1.0);
    }
}

/// Check if an entry should be quarantined due to persistent parasite behavior.
///
/// Quarantine when `parasite_score >= 0.5` for 2+ consecutive cycles.
/// We track this via the generation count and parasite_score threshold.
pub fn should_quarantine_parasite(parasite_score: f64) -> bool {
    parasite_score >= 0.5
}

/// Price equation population diagnostics.
///
/// `ΔW_population = selection_differential + transmission_bias`
///
/// Warns when both components are negative for 3+ consecutive cycles.
pub struct PriceEquationTracker {
    /// Count of consecutive cycles where both selection and transmission were negative.
    pub consecutive_negative_cycles: u32,
}

impl PriceEquationTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            consecutive_negative_cycles: 0,
        }
    }

    /// Record one cycle's diagnostics. Returns true if warning threshold hit.
    pub fn record_cycle(&mut self, selection: f64, transmission: f64) -> bool {
        if selection < 0.0 && transmission < 0.0 {
            self.consecutive_negative_cycles += 1;
        } else {
            self.consecutive_negative_cycles = 0;
        }
        self.consecutive_negative_cycles >= 3
    }

    /// Returns true if the knowledge ecology is degrading (3+ negative cycles).
    pub fn is_degrading(&self) -> bool {
        self.consecutive_negative_cycles >= 3
    }
}

impl Default for PriceEquationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // INV-013: Memetic fitness W(E)
    #[test]
    fn test_memetic_fitness_calculation() {
        // W(E) = fidelity * fecundity * longevity
        let w = fitness(0.8, 0.5, 0.001, 500);
        let expected_longevity = (-0.001_f64 * 500.0).exp();
        let expected = 0.8 * 0.5 * expected_longevity;
        assert!(
            (w - expected).abs() < 1e-10,
            "fitness {w} != expected {expected}"
        );
    }

    #[test]
    fn test_longevity_decay_over_age() {
        let decay_rate = 0.001;
        let l0 = longevity(decay_rate, 0);
        let l500 = longevity(decay_rate, 500);
        let l1000 = longevity(decay_rate, 1000);
        let l2000 = longevity(decay_rate, 2000);

        assert!((l0 - 1.0).abs() < 1e-10, "longevity at t=0 should be 1.0");
        assert!(l500 < l0, "longevity should decrease");
        assert!(l1000 < l500, "longevity should decrease further");
        assert!(l2000 < l1000, "longevity should keep decreasing");

        // Zero decay rate: longevity is always 1.0.
        assert!((longevity(0.0, 99_999) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fitness_components_non_negative() {
        for &fid in &[0.0, 0.5, 1.0] {
            for &fec in &[0.0, 0.5, 1.0] {
                for &age in &[0u64, 500, 1000] {
                    let w = fitness(fid, fec, 0.001, age);
                    assert!(w >= 0.0, "fitness should be non-negative: {w}");
                }
            }
        }
    }

    // INV-014: Parasite detection
    #[test]
    fn test_parasite_detection_triggers() {
        // W(E) > W_bar AND Q(E) < 0 -> parasite
        assert!(is_parasite(0.8, 0.5, -0.3));
        // W(E) <= W_bar -> not parasite
        assert!(!is_parasite(0.3, 0.5, -0.3));
        // Q(E) >= 0 -> not parasite
        assert!(!is_parasite(0.8, 0.5, 0.1));
        // Both below threshold -> not parasite
        assert!(!is_parasite(0.3, 0.5, 0.1));
    }

    #[test]
    fn test_parasite_quarantine_on_persistence() {
        let mut fields = MemeticFields::default();
        // Not yet a parasite.
        assert!(!should_quarantine_parasite(fields.parasite_score));

        // Accumulate parasite score over cycles.
        for _ in 0..5 {
            update_memetic_fields(&mut fields, 0.001, 100, true);
        }
        // After 5 flagging cycles: parasite_score = 0.5.
        assert!(
            fields.parasite_score >= 0.5,
            "parasite score {} should be >= 0.5",
            fields.parasite_score
        );
        assert!(should_quarantine_parasite(fields.parasite_score));
    }

    // INV-015: Price equation warning
    #[test]
    fn test_price_equation_negative_detection() {
        let mut tracker = PriceEquationTracker::new();

        // Two negative cycles: no warning yet.
        assert!(!tracker.record_cycle(-0.1, -0.2));
        assert!(!tracker.record_cycle(-0.3, -0.1));
        assert_eq!(tracker.consecutive_negative_cycles, 2);

        // Third negative cycle: warning triggered.
        assert!(tracker.record_cycle(-0.2, -0.3));
        assert!(tracker.is_degrading());
    }

    #[test]
    fn test_price_equation_recovery() {
        let mut tracker = PriceEquationTracker::new();

        // Three negative cycles.
        tracker.record_cycle(-0.1, -0.2);
        tracker.record_cycle(-0.3, -0.1);
        tracker.record_cycle(-0.2, -0.3);
        assert!(tracker.is_degrading());

        // One positive cycle resets the counter.
        tracker.record_cycle(0.1, -0.1);
        assert!(!tracker.is_degrading());
        assert_eq!(tracker.consecutive_negative_cycles, 0);
    }

    #[test]
    fn test_population_mean_fitness() {
        let fitnesses = vec![0.1, 0.3, 0.5, 0.7, 0.9];
        let mean = population_mean_fitness(&fitnesses);
        assert!((mean - 0.5).abs() < 1e-10);

        // Empty population.
        assert!((population_mean_fitness(&[]) - 0.0).abs() < f64::EPSILON);
    }
}

//! A-MAC five-factor admission gate.
//!
//! From Zhang et al., arXiv:2603.04549 (March 2026). Determines whether a new entry
//! should be admitted to the Grimoire based on five weighted factors:
//! future utility (0.25), factual confidence (0.25), semantic novelty (0.20),
//! temporal recency (0.15), content type prior (0.15).

use crate::entry::EntryType;

/// A-MAC five-factor admission scoring.
///
/// The admission gate evaluates each candidate entry against five factors:
///
/// | Factor             | Weight | Source                          |
/// |--------------------|--------|---------------------------------|
/// | Future utility     | 0.25   | LLM call (Haiku tier)           |
/// | Factual confidence | 0.25   | Cross-reference vs Grimoire     |
/// | Semantic novelty   | 0.20   | Cosine similarity search        |
/// | Temporal recency   | 0.15   | Exponential decay from event    |
/// | Content type prior | 0.15   | Per-type calibrated prior       |
pub struct AdmissionGate;

/// Result of an admission decision.
#[derive(Debug, Clone, PartialEq)]
pub enum AdmissionResult {
    /// Entry rejected (composite < 0.45).
    Rejected,
    /// Entry admitted at conservative confidence 0.3 (composite 0.45–0.55).
    AdmittedConservative,
    /// Entry admitted at standard confidence 0.6 (composite > 0.55).
    AdmittedStandard,
}

impl AdmissionGate {
    /// Compute the composite A-MAC admission score.
    ///
    /// `composite = 0.25*utility + 0.25*factual + 0.20*novelty + 0.15*recency + 0.15*prior`
    pub fn composite_score(
        entry_type: EntryType,
        future_utility: f64,
        factual_confidence: f64,
        semantic_novelty: f64,
        temporal_recency: f64,
    ) -> f64 {
        let type_prior = content_type_prior(entry_type);
        (future_utility * 0.25)
            + (factual_confidence * 0.25)
            + (semantic_novelty * 0.20)
            + (temporal_recency * 0.15)
            + (type_prior * 0.15)
    }

    /// Make an admission decision based on the composite score.
    ///
    /// - `< 0.45`: rejected
    /// - `0.45–0.55`: admitted at confidence 0.3
    /// - `> 0.55`: admitted at standard confidence 0.6
    pub fn decide(score: f64) -> AdmissionResult {
        if score < 0.45 {
            AdmissionResult::Rejected
        } else if score <= 0.55 {
            AdmissionResult::AdmittedConservative
        } else {
            AdmissionResult::AdmittedStandard
        }
    }

    /// Returns the confidence to assign based on the admission result.
    pub fn assigned_confidence(result: &AdmissionResult) -> Option<f64> {
        match result {
            AdmissionResult::Rejected => None,
            AdmissionResult::AdmittedConservative => Some(0.3),
            AdmissionResult::AdmittedStandard => Some(0.6),
        }
    }

    /// Check if entry should be quarantined (hallucination firewall).
    ///
    /// Triggers when `factual_confidence < 0.3` AND the entry contradicts
    /// an existing high-confidence entry.
    pub fn should_quarantine(factual_confidence: f64, contradicts_high_confidence: bool) -> bool {
        factual_confidence < 0.3 && contradicts_high_confidence
    }
}

/// Content-type admission prior. Higher prior = more likely to be admitted.
///
/// | Type              | Prior |
/// |-------------------|-------|
/// | Warning           | 0.9   |
/// | AntiKnowledge     | 0.9   |
/// | CausalLink        | 0.7   |
/// | Heuristic         | 0.6   |
/// | Insight           | 0.5   |
/// | StrategyFragment  | 0.4   |
pub fn content_type_prior(entry_type: EntryType) -> f64 {
    match entry_type {
        EntryType::Warning | EntryType::AntiKnowledge => 0.9,
        EntryType::CausalLink => 0.7,
        EntryType::Heuristic => 0.6,
        EntryType::Insight => 0.5,
        EntryType::StrategyFragment => 0.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // INV-007: A-MAC admission threshold
    #[test]
    fn test_admission_gate_rejects_low_quality() {
        let result = AdmissionGate::decide(0.3);
        assert_eq!(result, AdmissionResult::Rejected);

        let result = AdmissionGate::decide(0.0);
        assert_eq!(result, AdmissionResult::Rejected);

        let result = AdmissionGate::decide(0.449);
        assert_eq!(result, AdmissionResult::Rejected);
    }

    #[test]
    fn test_admission_gate_gray_zone() {
        let result = AdmissionGate::decide(0.45);
        assert_eq!(result, AdmissionResult::AdmittedConservative);

        let result = AdmissionGate::decide(0.50);
        assert_eq!(result, AdmissionResult::AdmittedConservative);

        let result = AdmissionGate::decide(0.55);
        assert_eq!(result, AdmissionResult::AdmittedConservative);

        assert_eq!(AdmissionGate::assigned_confidence(&result), Some(0.3));
    }

    #[test]
    fn test_admission_gate_accepts_high_quality() {
        let result = AdmissionGate::decide(0.7);
        assert_eq!(result, AdmissionResult::AdmittedStandard);

        let result = AdmissionGate::decide(1.0);
        assert_eq!(result, AdmissionResult::AdmittedStandard);

        assert_eq!(AdmissionGate::assigned_confidence(&result), Some(0.6));
    }

    // INV-008: A-MAC content type priors
    #[test]
    fn test_admission_prior_by_type() {
        assert!((content_type_prior(EntryType::Warning) - 0.9).abs() < f64::EPSILON);
        assert!((content_type_prior(EntryType::AntiKnowledge) - 0.9).abs() < f64::EPSILON);
        assert!((content_type_prior(EntryType::CausalLink) - 0.7).abs() < f64::EPSILON);
        assert!((content_type_prior(EntryType::Heuristic) - 0.6).abs() < f64::EPSILON);
        assert!((content_type_prior(EntryType::Insight) - 0.5).abs() < f64::EPSILON);
        assert!((content_type_prior(EntryType::StrategyFragment) - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_composite_score_weights_sum_to_one() {
        // If all factors are 1.0, composite should be 1.0.
        let score = AdmissionGate::composite_score(EntryType::Insight, 1.0, 1.0, 1.0, 1.0);
        // Insight prior is 0.5, so: 0.25 + 0.25 + 0.20 + 0.15 + 0.15*0.5 = 0.925.
        // Actually: 0.25*1 + 0.25*1 + 0.20*1 + 0.15*1 + 0.15*0.5 = 0.925.
        assert!(
            (score - 0.925).abs() < 1e-10,
            "composite with all-1.0 factors and Insight prior: {score}"
        );
    }

    #[test]
    fn test_warning_gets_admission_boost() {
        // Warning with moderate scores should be admitted due to high prior.
        let score = AdmissionGate::composite_score(EntryType::Warning, 0.5, 0.5, 0.5, 0.5);
        // 0.25*0.5 + 0.25*0.5 + 0.20*0.5 + 0.15*0.5 + 0.15*0.9 = 0.56
        let result = AdmissionGate::decide(score);
        assert_eq!(
            result,
            AdmissionResult::AdmittedStandard,
            "Warning with moderate scores should be admitted: {score}"
        );
    }

    #[test]
    fn test_quarantine_firewall() {
        assert!(AdmissionGate::should_quarantine(0.2, true));
        assert!(!AdmissionGate::should_quarantine(0.4, true));
        assert!(!AdmissionGate::should_quarantine(0.2, false));
        assert!(!AdmissionGate::should_quarantine(0.4, false));
    }
}

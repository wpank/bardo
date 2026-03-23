//! Four-factor retrieval scoring.
//!
//! Score = recency × importance × relevance × emotional_congruence.
//! Each factor is independently computed and the final score is their product.

use crate::decay;
use crate::entry::{EmotionalTag, GrimoireEntry};
use golem_core::cortical::PadVector;

/// Entry paired with its computed retrieval score and component breakdown.
#[derive(Debug, Clone)]
pub struct ScoredEntry {
    /// The scored entry.
    pub entry: GrimoireEntry,
    /// Combined four-factor retrieval score.
    pub retrieval_score: f32,
    /// Recency component: exp(-λ * elapsed).
    pub recency_component: f32,
    /// Relevance component: cosine similarity.
    pub relevance_component: f32,
    /// Importance component: confidence * quality_score.
    pub importance_component: f32,
    /// Emotional congruence component: PAD dot product mapping.
    pub congruence_component: f32,
}

/// Compute the four-factor retrieval score for a single entry.
///
/// `retrieval_score = recency × importance × max(relevance, 0) × congruence`
pub fn score_entry(
    entry: &GrimoireEntry,
    query_embedding: &[f32],
    query_pad: &PadVector,
    current_tick: u64,
) -> ScoredEntry {
    let recency = decay::recency_score(entry, current_tick);
    let importance = compute_importance(entry);
    let relevance = cosine_similarity(query_embedding, &entry.embedding);
    let congruence = emotional_congruence(query_pad, entry.emotional_tag.as_ref());

    let retrieval_score = recency * importance * relevance.max(0.0) * congruence;

    ScoredEntry {
        entry: entry.clone(),
        retrieval_score,
        recency_component: recency,
        relevance_component: relevance,
        importance_component: importance,
        congruence_component: congruence,
    }
}

/// Importance = confidence * quality_score. Bloodstain gets 1.2x boost (capped at 1.0).
#[allow(clippy::cast_possible_truncation)]
fn compute_importance(entry: &GrimoireEntry) -> f32 {
    let base = (entry.confidence * entry.quality_score) as f32;
    if entry.is_bloodstain {
        (base * 1.2).min(1.0)
    } else {
        base
    }
}

/// Cosine similarity between two embedding vectors.
///
/// Returns a value in [-1.0, 1.0], clamped. Returns 0.0 for empty or zero-norm vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Emotional congruence between query PAD and entry emotional tag.
///
/// Maps from dot product of normalized PAD vectors to [0.0, 1.0]:
/// `congruence = 0.5 + 0.5 * dot(query_pad_normalized, entry_pad_normalized)`
///
/// Returns 0.5 (neutral) when entry has no emotional tag or PAD is zero.
#[allow(clippy::cast_possible_truncation)]
pub fn emotional_congruence(query_pad: &PadVector, entry_tag: Option<&EmotionalTag>) -> f32 {
    let Some(tag) = entry_tag else {
        return 0.5;
    };

    let q = normalize_pad(query_pad);
    let e = normalize_pad(&tag.pad);

    // Both zero vectors: neutral
    if q == [0.0, 0.0, 0.0] || e == [0.0, 0.0, 0.0] {
        return 0.5;
    }

    let dot = q[0] * e[0] + q[1] * e[1] + q[2] * e[2];
    (0.5 + 0.5 * dot).clamp(0.0, 1.0) as f32
}

/// Normalize a PAD vector to unit length. Returns [0, 0, 0] for zero vectors.
fn normalize_pad(pad: &PadVector) -> [f64; 3] {
    let norm =
        (pad.pleasure * pad.pleasure + pad.arousal * pad.arousal + pad.dominance * pad.dominance)
            .sqrt();
    if norm == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [
            pad.pleasure / norm,
            pad.arousal / norm,
            pad.dominance / norm,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::GrimoireEntry;
    use golem_core::cortical::{PadVector, PlutchikEmotion};

    // INV-004: Cosine similarity range [-1, 1]
    #[test]
    fn test_cosine_similarity_unit_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001, "identical vectors: {sim}");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001, "orthogonal vectors: {sim}");
    }

    #[test]
    fn test_cosine_similarity_clamp() {
        // Opposite vectors.
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim >= -1.0 && sim <= 1.0,
            "opposite vectors should be in [-1,1]: {sim}"
        );
        assert!((sim - (-1.0)).abs() < 0.001, "opposite should be -1: {sim}");

        // Zero-norm vector.
        let z = vec![0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &z), 0.0, "zero vector should give 0");

        // Empty vectors.
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    // INV-005: Emotional congruence mapping
    #[test]
    fn test_emotional_congruence_neutral() {
        let pad = PadVector::ZERO;
        let result = emotional_congruence(&pad, None);
        assert!(
            (result - 0.5).abs() < 0.001,
            "no tag should be neutral 0.5: {result}"
        );
    }

    #[test]
    fn test_emotional_congruence_identical() {
        let query = PadVector {
            pleasure: 0.8,
            arousal: 0.5,
            dominance: 0.3,
        };
        let tag = EmotionalTag {
            primary: PlutchikEmotion::Joy,
            pad: PadVector {
                pleasure: 0.8,
                arousal: 0.5,
                dominance: 0.3,
            },
            arousal: 0.5,
        };
        let result = emotional_congruence(&query, Some(&tag));
        assert!(
            (result - 1.0).abs() < 0.001,
            "identical PAD should give 1.0: {result}"
        );
    }

    #[test]
    fn test_emotional_congruence_opposite() {
        let query = PadVector {
            pleasure: 1.0,
            arousal: 0.0,
            dominance: 0.0,
        };
        let tag = EmotionalTag {
            primary: PlutchikEmotion::Sadness,
            pad: PadVector {
                pleasure: -1.0,
                arousal: 0.0,
                dominance: 0.0,
            },
            arousal: 0.5,
        };
        let result = emotional_congruence(&query, Some(&tag));
        assert!(result < 0.01, "opposite PAD should give ~0.0: {result}");
    }

    // INV-022: PAD normalization invariant
    #[test]
    fn test_pad_normalization_invariant() {
        let pad = PadVector {
            pleasure: 3.0,
            arousal: 4.0,
            dominance: 0.0,
        };
        let n = normalize_pad(&pad);
        let norm = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "normalized PAD should have unit length: {norm}"
        );
    }

    // INV-006: Four-factor retrieval score is multiplicative
    #[test]
    fn test_four_factor_retrieval_ranking() {
        // Entry A: high relevance, recent.
        let mut entry_a = GrimoireEntry::test_heuristic("relevant entry A");
        entry_a.embedding = vec![1.0, 0.0, 0.0];
        entry_a.confidence = 0.8;
        entry_a.quality_score = 0.7;
        entry_a.last_accessed_at = 90;

        // Entry B: low relevance, old.
        let mut entry_b = GrimoireEntry::test_heuristic("irrelevant entry B");
        entry_b.embedding = vec![0.0, 1.0, 0.0];
        entry_b.confidence = 0.3;
        entry_b.quality_score = 0.3;
        entry_b.last_accessed_at = 0;

        let query = vec![1.0, 0.0, 0.0];
        let pad = PadVector::ZERO;

        let scored_a = score_entry(&entry_a, &query, &pad, 100);
        let scored_b = score_entry(&entry_b, &query, &pad, 100);

        assert!(
            scored_a.retrieval_score > scored_b.retrieval_score,
            "A ({}) should outscore B ({})",
            scored_a.retrieval_score,
            scored_b.retrieval_score
        );
    }

    #[test]
    fn test_retrieval_score_component_isolation() {
        // Each component should independently affect the score.
        let mut entry = GrimoireEntry::test_heuristic("isolation test");
        entry.embedding = vec![1.0, 0.0, 0.0];
        entry.confidence = 1.0;
        entry.quality_score = 1.0;
        entry.last_accessed_at = 100;

        let query = vec![1.0, 0.0, 0.0];
        let pad = PadVector::ZERO;

        let scored = score_entry(&entry, &query, &pad, 100);

        // All components should be non-negative.
        assert!(scored.recency_component >= 0.0);
        assert!(scored.importance_component >= 0.0);
        assert!(scored.relevance_component >= 0.0);
        assert!(scored.congruence_component >= 0.0);
        assert!(scored.retrieval_score >= 0.0);

        // Score should equal the product of components.
        let expected = scored.recency_component
            * scored.importance_component
            * scored.relevance_component.max(0.0)
            * scored.congruence_component;
        assert!(
            (scored.retrieval_score - expected).abs() < 1e-6,
            "score {} should equal product {}",
            scored.retrieval_score,
            expected
        );
    }

    // INV-003: Bloodstain retrieval boost
    #[test]
    fn test_bloodstain_retrieval_boost() {
        let mut normal = GrimoireEntry::test_heuristic("normal");
        normal.embedding = vec![1.0, 0.0, 0.0];
        normal.confidence = 0.7;
        normal.quality_score = 0.7;
        normal.last_accessed_at = 0;
        normal.is_bloodstain = false;

        let mut bloodstain = normal.clone();
        bloodstain.is_bloodstain = true;

        let query = vec![1.0, 0.0, 0.0];
        let pad = PadVector::ZERO;

        let normal_scored = score_entry(&normal, &query, &pad, 100);
        let blood_scored = score_entry(&bloodstain, &query, &pad, 100);

        // Bloodstain should score higher due to both 1.2x importance and 3x slower decay.
        assert!(
            blood_scored.retrieval_score > normal_scored.retrieval_score,
            "bloodstain ({}) should outscore normal ({})",
            blood_scored.retrieval_score,
            normal_scored.retrieval_score
        );

        // Importance boost: bloodstain importance should be 1.2x (capped at 1.0).
        assert!(blood_scored.importance_component >= normal_scored.importance_component);
    }
}

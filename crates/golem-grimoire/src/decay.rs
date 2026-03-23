//! Ebbinghaus decay formulas with per-type half-lives.
//!
//! Implements the forgetting curve: `retention(t) = exp(-(t - last_accessed) / (half_life * strength))`.
//! Bloodstain entries decay 3x slower. Low-quality entries (quality < 0.3) decay 2x faster.

use crate::entry::{DecayClass, EntryType, GrimoireEntry};

/// Decay constant (lambda) per tick for each `DecayClass`.
///
/// Half-life in ticks = ln(2) / lambda.
///
/// | Class              | λ/tick  | Half-life (ticks) | ~Real time at 40s/tick |
/// |--------------------|---------|-------------------|------------------------|
/// | Ephemeral          | 0.01    | ~69               | ~48 hours              |
/// | Tactical           | 0.001   | ~693              | ~7 days                |
/// | RegimeConditional  | 0.0005  | ~1386             | ~14 days               |
/// | Structural         | 0.0     | ∞                 | never                  |
/// | Procedural         | 0.0001  | ~6931             | very slow              |
pub fn lambda_for_class(class: DecayClass) -> f64 {
    match class {
        DecayClass::Ephemeral => 0.01,
        DecayClass::Tactical => 0.001,
        DecayClass::RegimeConditional => 0.0005,
        DecayClass::Structural => 0.0,
        DecayClass::Procedural => 0.0001,
    }
}

/// Computes the effective lambda for an entry, accounting for bloodstain modifier
/// and quality-based acceleration.
///
/// - Bloodstain entries: effective λ = λ × (1/3), i.e. 3x slower decay.
/// - Quality score < 0.3: effective λ = λ × 2.0, i.e. 2x faster decay.
pub fn effective_lambda(entry: &GrimoireEntry) -> f64 {
    let base = lambda_for_class(entry.decay_class);
    let mut lambda = base;

    // Bloodstain modifier: 3x slower decay
    if entry.is_bloodstain {
        lambda /= 3.0;
    }

    // Low quality accelerates decay
    if entry.quality_score < 0.3 {
        lambda *= 2.0;
    }

    lambda
}

/// Ebbinghaus retention at time `t` ticks since last access.
///
/// `retention(t) = exp(-(t - last_accessed) / (half_life_ticks * strength))`
///
/// Using lambda directly: `retention(t) = exp(-lambda * elapsed / max(strength, 1))`
pub fn retention(lambda: f64, elapsed_ticks: u64, strength: u32) -> f64 {
    if lambda == 0.0 {
        return 1.0; // Structural: no decay
    }
    let s = f64::from(strength.max(1));
    (-lambda * (elapsed_ticks as f64) / s).exp()
}

/// Computes the effective confidence of an entry at a given tick.
///
/// `effective_confidence = confidence * retention(t)`
/// Floor: 0.05 for standard entries, 0.30 for `AntiKnowledge`.
pub fn effective_confidence(entry: &GrimoireEntry, current_tick: u64) -> f64 {
    let elapsed = current_tick.saturating_sub(entry.last_accessed_at.max(0) as u64);
    let lambda = effective_lambda(entry);
    let r = retention(lambda, elapsed, entry.strength);
    let raw = entry.confidence * r;
    let floor = entry.category.confidence_floor();
    raw.max(floor)
}

/// Computes the recency score for retrieval ranking.
///
/// Uses the same lambda-based exponential decay but without the strength divisor,
/// since recency should reflect raw temporal distance.
pub fn recency_score(entry: &GrimoireEntry, current_tick: u64) -> f32 {
    let elapsed = current_tick.saturating_sub(entry.last_accessed_at.max(0) as u64);
    let mut lambda = lambda_for_class(entry.decay_class);

    // Bloodstain: 3x slower
    if entry.is_bloodstain {
        lambda /= 3.0;
    }

    #[allow(clippy::cast_possible_truncation)]
    let score = (-lambda * elapsed as f64).exp() as f32;
    score
}

/// Returns the half-life in ticks for a decay class (ln(2) / lambda).
/// Returns `f64::INFINITY` for Structural (no decay).
pub fn half_life_ticks(class: DecayClass) -> f64 {
    let lambda = lambda_for_class(class);
    if lambda == 0.0 {
        f64::INFINITY
    } else {
        f64::ln(2.0) / lambda
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{DecayClass, EntryType, GrimoireEntry};

    // INV-001: Ebbinghaus decay formula
    #[test]
    fn test_ebbinghaus_decay_at_half_life() {
        // At t = half_life, retention should be ~0.5 (for strength=1).
        // retention(t) = exp(-lambda * t / strength)
        // At half_life = ln(2)/lambda, retention = exp(-ln(2)) = 0.5.

        for &(class, expected_lambda) in &[
            (DecayClass::Ephemeral, 0.01),
            (DecayClass::Tactical, 0.001),
            (DecayClass::RegimeConditional, 0.0005),
            (DecayClass::Procedural, 0.0001),
        ] {
            let lambda = lambda_for_class(class);
            assert!(
                (lambda - expected_lambda).abs() < 1e-10,
                "lambda mismatch for {class:?}"
            );

            let hl = half_life_ticks(class);
            let r = retention(lambda, hl as u64, 1);
            // At half-life, retention should be ~exp(-ln(2)) ≈ 0.5.
            // But since we use integer ticks, there's rounding.
            assert!(
                (r - 0.5).abs() < 0.05,
                "retention at half-life should be ~0.5, got {r} for {class:?}"
            );
        }

        // Structural: no decay at any time.
        let r = retention(lambda_for_class(DecayClass::Structural), 999_999, 1);
        assert!(
            (r - 1.0).abs() < f64::EPSILON,
            "Structural should not decay"
        );
    }

    // INV-002: Decay class lambda values produce expected half-lives
    #[test]
    fn test_decay_class_half_lives() {
        let hl_eph = half_life_ticks(DecayClass::Ephemeral);
        let hl_tac = half_life_ticks(DecayClass::Tactical);
        let hl_reg = half_life_ticks(DecayClass::RegimeConditional);
        let hl_str = half_life_ticks(DecayClass::Structural);
        let hl_pro = half_life_ticks(DecayClass::Procedural);

        // Ephemeral: ~69 ticks
        assert!((hl_eph - 69.3).abs() < 1.0, "Ephemeral half-life: {hl_eph}");
        // Tactical: ~693 ticks
        assert!((hl_tac - 693.1).abs() < 1.0, "Tactical half-life: {hl_tac}");
        // RegimeConditional: ~1386 ticks
        assert!(
            (hl_reg - 1386.3).abs() < 1.0,
            "RegimeConditional half-life: {hl_reg}"
        );
        // Structural: infinity
        assert!(hl_str.is_infinite(), "Structural should be infinite");
        // Procedural: ~6931 ticks
        assert!(
            (hl_pro - 6931.5).abs() < 1.0,
            "Procedural half-life: {hl_pro}"
        );

        // Ordering: Ephemeral < Tactical < RegimeConditional < Procedural < Structural
        assert!(hl_eph < hl_tac);
        assert!(hl_tac < hl_reg);
        assert!(hl_reg < hl_pro);
        assert!(hl_pro < hl_str);
    }

    // INV-003: Bloodstain decay 3x slower
    #[test]
    fn test_bloodstain_decay_3x_slower() {
        let mut normal = GrimoireEntry::test_heuristic("normal");
        normal.last_accessed_at = 0;
        normal.is_bloodstain = false;

        let mut bloodstain = GrimoireEntry::test_heuristic("bloodstain");
        bloodstain.last_accessed_at = 0;
        bloodstain.is_bloodstain = true;

        let tick = 500;
        let normal_lambda = effective_lambda(&normal);
        let blood_lambda = effective_lambda(&bloodstain);

        // Bloodstain lambda should be 1/3 of normal.
        assert!(
            (blood_lambda - normal_lambda / 3.0).abs() < 1e-10,
            "bloodstain lambda {blood_lambda} should be normal/3 = {}",
            normal_lambda / 3.0
        );

        let normal_recency = recency_score(&normal, tick);
        let blood_recency = recency_score(&bloodstain, tick);

        // Bloodstain should have higher recency (slower decay).
        assert!(
            blood_recency > normal_recency,
            "bloodstain recency {blood_recency} should exceed normal {normal_recency}"
        );
    }

    // INV-019: Quality score < 0.3 triggers 2x decay acceleration
    #[test]
    fn test_low_quality_decay_acceleration() {
        let mut high_quality = GrimoireEntry::test_heuristic("high q");
        high_quality.quality_score = 0.5;
        high_quality.is_bloodstain = false;

        let mut low_quality = GrimoireEntry::test_heuristic("low q");
        low_quality.quality_score = 0.2;
        low_quality.is_bloodstain = false;

        let lambda_high = effective_lambda(&high_quality);
        let lambda_low = effective_lambda(&low_quality);

        assert!(
            (lambda_low - lambda_high * 2.0).abs() < 1e-10,
            "low quality lambda {lambda_low} should be 2x high quality {lambda_high}"
        );
    }

    // INV-009 (entry-level): AntiKnowledge never archived (confidence floor)
    #[test]
    fn test_anti_knowledge_never_archived() {
        let mut entry = GrimoireEntry::test_with_category(EntryType::AntiKnowledge, 0.35);
        entry.last_accessed_at = 0;
        entry.strength = 1;

        // Even after many ticks, effective confidence should not drop below 0.30.
        let eff = effective_confidence(&entry, 100_000);
        assert!(
            eff >= 0.30,
            "AntiKnowledge effective confidence {eff} should be >= 0.30"
        );
    }

    #[test]
    fn test_decay_reduces_score_over_time() {
        let mut entry = GrimoireEntry::test_heuristic("decay test");
        entry.confidence = 0.8;
        entry.last_accessed_at = 0;

        let c0 = effective_confidence(&entry, 0);
        let c100 = effective_confidence(&entry, 100);
        let c500 = effective_confidence(&entry, 500);
        let c1000 = effective_confidence(&entry, 1000);

        assert!(c0 >= c100, "c0={c0} should >= c100={c100}");
        assert!(c100 >= c500, "c100={c100} should >= c500={c500}");
        assert!(c500 >= c1000, "c500={c500} should >= c1000={c1000}");
    }

    #[test]
    fn test_strength_slows_decay() {
        let lambda = 0.001;
        let elapsed = 500;

        let r1 = retention(lambda, elapsed, 1);
        let r3 = retention(lambda, elapsed, 3);

        // Higher strength should yield higher retention.
        assert!(
            r3 > r1,
            "strength=3 retention {r3} should exceed strength=1 retention {r1}"
        );
    }
}

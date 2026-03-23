//! Cross-model verification for high-stakes actions.
//!
//! When a vault action exceeds $500 USD, two models are queried and their
//! outputs compared. If agreement falls below 0.7, the action is blocked.

use serde::Serialize;

/// USD threshold above which cross-model verification is required.
pub const VERIFICATION_STAKE_THRESHOLD_USD: f64 = 500.0;
/// Minimum agreement score for verification to pass.
pub const VERIFICATION_AGREEMENT_MIN: f64 = 0.7;

/// A single model's verification response.
#[derive(Debug, Clone, Serialize)]
pub struct VerificationResponse {
    pub model: String,
    pub parsed_action: ParsedAction,
    pub raw_response: String,
}

/// Parsed action components extracted from a model response.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedAction {
    /// buy / sell / hold / other.
    pub direction: String,
    /// Nominal USD value of the action.
    pub amount_usd: f64,
    /// immediate / delayed.
    pub urgency: String,
    /// 0.0-1.0 risk score from the model.
    pub risk_score: f64,
}

/// Result of cross-model verification.
#[derive(Debug, Clone, Serialize)]
pub struct CrossModelVerification {
    pub primary: VerificationResponse,
    pub secondary: VerificationResponse,
    /// Computed agreement score (0.0-1.0).
    pub agreement: f64,
    /// True when agreement >= VERIFICATION_AGREEMENT_MIN.
    pub passed: bool,
}

/// Compute agreement between two parsed actions.
///
/// Scoring weights:
/// - Direction agreement (buy/sell/hold): 40%
/// - Amount within 20% of each other: 30%
/// - Timing agreement: 15%
/// - Risk assessment alignment: 15%
pub fn compute_action_agreement(a: &ParsedAction, b: &ParsedAction) -> f64 {
    let mut score = 0.0_f64;

    // Direction agreement (40%).
    if a.direction == b.direction {
        score += 0.4;
    }

    // Amount within range (30%).
    let max_amount = a.amount_usd.max(b.amount_usd);
    let min_amount = a.amount_usd.min(b.amount_usd);
    if max_amount > 0.0 {
        let amount_ratio = min_amount / max_amount;
        score += 0.3 * amount_ratio;
    } else {
        // Both zero -- perfect agreement.
        score += 0.3;
    }

    // Timing agreement (15%).
    if a.urgency == b.urgency {
        score += 0.15;
    }

    // Risk assessment alignment (15%).
    let risk_diff = (a.risk_score - b.risk_score).abs();
    score += 0.15 * (1.0 - risk_diff);

    score
}

/// Whether an action's USD value exceeds the verification threshold.
pub fn requires_verification(amount_usd: f64) -> bool {
    amount_usd >= VERIFICATION_STAKE_THRESHOLD_USD
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(direction: &str, amount: f64, urgency: &str, risk: f64) -> ParsedAction {
        ParsedAction {
            direction: direction.to_string(),
            amount_usd: amount,
            urgency: urgency.to_string(),
            risk_score: risk,
        }
    }

    #[test]
    fn test_agreement_perfect() {
        let a = make_action("buy", 1000.0, "immediate", 0.3);
        let b = make_action("buy", 1000.0, "immediate", 0.3);
        let agreement = compute_action_agreement(&a, &b);
        assert!((agreement - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_agreement_direction_mismatch() {
        let a = make_action("buy", 1000.0, "immediate", 0.3);
        let b = make_action("sell", 1000.0, "immediate", 0.3);
        let agreement = compute_action_agreement(&a, &b);
        // 0 + 0.3 + 0.15 + 0.15 = 0.6 (below 0.7 threshold).
        assert!((agreement - 0.6).abs() < 1e-10);
        assert!(agreement < VERIFICATION_AGREEMENT_MIN);
    }

    #[test]
    fn test_agreement_amount_divergence() {
        let a = make_action("buy", 1000.0, "immediate", 0.3);
        let b = make_action("buy", 500.0, "immediate", 0.3);
        let agreement = compute_action_agreement(&a, &b);
        // 0.4 + 0.3*(500/1000) + 0.15 + 0.15 = 0.4 + 0.15 + 0.15 + 0.15 = 0.85
        assert!((agreement - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_agreement_timing_mismatch() {
        let a = make_action("buy", 1000.0, "immediate", 0.3);
        let b = make_action("buy", 1000.0, "delayed", 0.3);
        let agreement = compute_action_agreement(&a, &b);
        // 0.4 + 0.3 + 0.0 + 0.15 = 0.85
        assert!((agreement - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_agreement_risk_divergence() {
        let a = make_action("buy", 1000.0, "immediate", 0.1);
        let b = make_action("buy", 1000.0, "immediate", 0.9);
        let agreement = compute_action_agreement(&a, &b);
        // 0.4 + 0.3 + 0.15 + 0.15*(1-0.8) = 0.85 + 0.03 = 0.88
        assert!((agreement - 0.88).abs() < 1e-10);
    }

    #[test]
    fn test_requires_verification() {
        assert!(!requires_verification(499.99));
        assert!(requires_verification(500.0));
        assert!(requires_verification(1000.0));
    }

    #[test]
    fn test_agreement_both_zero_amounts() {
        let a = make_action("hold", 0.0, "delayed", 0.5);
        let b = make_action("hold", 0.0, "delayed", 0.5);
        let agreement = compute_action_agreement(&a, &b);
        assert!((agreement - 1.0).abs() < 1e-10);
    }
}

//! Model selection for the Bankr provider based on vitality (sustainability ratio).

use serde::{Deserialize, Serialize};

/// Vitality thresholds for model selection.
pub const VITALITY_HEALTHY: f64 = 0.6;
pub const VITALITY_CAUTIOUS: f64 = 0.3;
pub const VITALITY_TERMINAL: f64 = 0.1;

/// Subsystems exempt from terminal conservation.
const EXEMPT_SUBSYSTEMS: &[&str] = &["risk", "death", "operator"];

/// Which model tier is being used (for logging/metrics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BankrModelTier {
    T1,
    T2Balanced,
    T2Deep,
    Suppressed,
}

/// Maps Bankr tiers to concrete model names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankrModelMapping {
    /// T1: cheap, fast.
    pub t1: String,
    /// T2 balanced: mid-range reasoning.
    pub t2_balanced: String,
    /// T2 deep: maximum reasoning.
    pub t2_deep: String,
}

impl Default for BankrModelMapping {
    fn default() -> Self {
        Self {
            t1: "gemini-2.5-flash".to_string(),
            t2_balanced: "claude-sonnet-4".to_string(),
            t2_deep: "claude-opus-4-6".to_string(),
        }
    }
}

/// Select a Bankr model based on vitality and subsystem.
///
/// Returns None when vitality is terminal and subsystem is not exempt
/// (model suppressed to conserve resources).
pub fn select_bankr_model<'a>(
    mapping: &'a BankrModelMapping,
    vitality: f64,
    subsystem: &str,
) -> Option<(&'a str, BankrModelTier)> {
    let is_exempt = EXEMPT_SUBSYSTEMS.contains(&subsystem);

    if vitality >= VITALITY_HEALTHY || is_exempt {
        // Full access or exempt: use deep model.
        Some((&mapping.t2_deep, BankrModelTier::T2Deep))
    } else if vitality >= VITALITY_CAUTIOUS {
        // Cautious: downgrade to balanced.
        Some((&mapping.t2_balanced, BankrModelTier::T2Balanced))
    } else if vitality >= VITALITY_TERMINAL {
        // Low vitality: use T1.
        Some((&mapping.t1, BankrModelTier::T1))
    } else {
        // Terminal: suppress unless exempt.
        if is_exempt {
            Some((&mapping.t1, BankrModelTier::T1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_mapping() -> BankrModelMapping {
        BankrModelMapping::default()
    }

    #[test]
    fn test_select_healthy() {
        let m = default_mapping();
        let (model, tier) = select_bankr_model(&m, 0.8, "trade").unwrap();
        assert_eq!(model, "claude-opus-4-6");
        assert_eq!(tier, BankrModelTier::T2Deep);
    }

    #[test]
    fn test_select_cautious() {
        let m = default_mapping();
        let (model, tier) = select_bankr_model(&m, 0.45, "trade").unwrap();
        assert_eq!(model, "claude-sonnet-4");
        assert_eq!(tier, BankrModelTier::T2Balanced);
    }

    #[test]
    fn test_select_low_vitality() {
        let m = default_mapping();
        let (model, tier) = select_bankr_model(&m, 0.2, "trade").unwrap();
        assert_eq!(model, "gemini-2.5-flash");
        assert_eq!(tier, BankrModelTier::T1);
    }

    #[test]
    fn test_select_terminal_suppressed() {
        let m = default_mapping();
        let result = select_bankr_model(&m, 0.05, "trade");
        assert!(result.is_none());
    }

    #[test]
    fn test_select_terminal_exempt_death() {
        let m = default_mapping();
        // Exempt subsystems bypass all conservation -- get T2Deep.
        let (model, tier) = select_bankr_model(&m, 0.05, "death").unwrap();
        assert_eq!(model, "claude-opus-4-6");
        assert_eq!(tier, BankrModelTier::T2Deep);
    }

    #[test]
    fn test_select_terminal_exempt_risk() {
        let m = default_mapping();
        let (model, tier) = select_bankr_model(&m, 0.05, "risk").unwrap();
        assert_eq!(model, "claude-opus-4-6");
        assert_eq!(tier, BankrModelTier::T2Deep);
    }

    #[test]
    fn test_select_terminal_exempt_operator() {
        let m = default_mapping();
        let (model, tier) = select_bankr_model(&m, 0.05, "operator").unwrap();
        assert_eq!(model, "claude-opus-4-6");
        assert_eq!(tier, BankrModelTier::T2Deep);
    }

    #[test]
    fn test_select_exempt_always_gets_deep() {
        let m = default_mapping();
        // Even at low vitality, exempt subsystems get T2Deep.
        let (model, tier) = select_bankr_model(&m, 0.2, "death").unwrap();
        assert_eq!(model, "claude-opus-4-6");
        assert_eq!(tier, BankrModelTier::T2Deep);
    }

    #[test]
    fn test_vitality_boundary_healthy() {
        let m = default_mapping();
        // At exactly VITALITY_HEALTHY.
        let (_, tier) = select_bankr_model(&m, VITALITY_HEALTHY, "trade").unwrap();
        assert_eq!(tier, BankrModelTier::T2Deep);
        // Just below.
        let (_, tier) = select_bankr_model(&m, VITALITY_HEALTHY - 0.01, "trade").unwrap();
        assert_eq!(tier, BankrModelTier::T2Balanced);
    }

    #[test]
    fn test_vitality_boundary_cautious() {
        let m = default_mapping();
        let (_, tier) = select_bankr_model(&m, VITALITY_CAUTIOUS, "trade").unwrap();
        assert_eq!(tier, BankrModelTier::T2Balanced);
        let (_, tier) = select_bankr_model(&m, VITALITY_CAUTIOUS - 0.01, "trade").unwrap();
        assert_eq!(tier, BankrModelTier::T1);
    }
}

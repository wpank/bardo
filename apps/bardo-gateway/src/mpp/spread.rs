//! Spread calculation and ERC-8004 reputation-based discounts.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

/// ERC-8004 reputation tiers with corresponding spread percentages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReputationTier {
    None,
    Basic,
    Verified,
    Trusted,
    Sovereign,
}

impl ReputationTier {
    /// Spread multiplier for this tier (e.g. 0.20 = 20%).
    pub fn spread(&self) -> f64 {
        match self {
            Self::None => 0.20,
            Self::Basic => 0.18,
            Self::Verified => 0.15,
            Self::Trusted => 0.12,
            Self::Sovereign => 0.08,
        }
    }

    /// Look up reputation tier for an address.
    ///
    /// Phase 1 stub: always returns `None` (20% spread).
    /// Phase 2 will query ERC-8004 registry via golem-chain.
    pub async fn for_address(_addr: Address) -> Self {
        Self::None
    }
}

impl Default for ReputationTier {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_tiers() {
        assert!((ReputationTier::None.spread() - 0.20).abs() < f64::EPSILON);
        assert!((ReputationTier::Basic.spread() - 0.18).abs() < f64::EPSILON);
        assert!((ReputationTier::Verified.spread() - 0.15).abs() < f64::EPSILON);
        assert!((ReputationTier::Trusted.spread() - 0.12).abs() < f64::EPSILON);
        assert!((ReputationTier::Sovereign.spread() - 0.08).abs() < f64::EPSILON);
    }
}

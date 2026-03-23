//! Spread calculation and reputation-based discounts.
//!
//! Spread is the markup applied on top of provider cost. Higher reputation
//! tiers get lower spreads, incentivizing on-chain identity and history.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

/// Reputation tiers with corresponding spread percentages.
///
/// Tiers map to on-chain attestations (e.g. ERC-8004). Higher tiers
/// get lower spreads as a reward for established identity.
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
    /// Stub: always returns `None` (20% spread).
    /// Future: query an on-chain registry.
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
    fn spread_decreases_with_tier() {
        let tiers = [
            ReputationTier::None,
            ReputationTier::Basic,
            ReputationTier::Verified,
            ReputationTier::Trusted,
            ReputationTier::Sovereign,
        ];
        for pair in tiers.windows(2) {
            assert!(pair[0].spread() > pair[1].spread());
        }
    }

    #[test]
    fn spread_values() {
        assert!((ReputationTier::None.spread() - 0.20).abs() < f64::EPSILON);
        assert!((ReputationTier::Sovereign.spread() - 0.08).abs() < f64::EPSILON);
    }
}

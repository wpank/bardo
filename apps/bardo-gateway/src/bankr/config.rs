//! Bankr provider configuration types.

use serde::{Deserialize, Serialize};

use super::metabolic::MetabolicLoopConfig;
use super::routing::BankrModelMapping;

/// Minimum credits before auto-replenish triggers.
pub const DEFAULT_REPLENISH_THRESHOLD_CREDITS: u64 = 500;
/// USDC amount to top up on each auto-replenish (micro-USDC: 10_000_000 = $10).
pub const DEFAULT_REPLENISH_AMOUNT_USDC: u64 = 10_000_000;
/// Maximum number of auto-replenish operations per 24h window.
pub const DEFAULT_MAX_REPLENISH_PER_DAY: u32 = 5;

/// Sustainability ratio at which throttling begins.
pub const THROTTLE_RATIO_THRESHOLD: f64 = 1.0;
/// Sustainability ratio at which emergency-only mode activates.
pub const EMERGENCY_RATIO_THRESHOLD: f64 = 0.5;

/// Auto-replenish policy for credit management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplenishPolicy {
    /// Trigger replenish when credits drop below this value.
    pub threshold_credits: u64,
    /// How much USDC to top up (micro-USDC units, 6 decimals).
    pub replenish_amount_usdc: u64,
    /// Hard cap on replenish operations per day.
    pub max_replenish_per_day: u32,
}

impl Default for AutoReplenishPolicy {
    fn default() -> Self {
        Self {
            threshold_credits: DEFAULT_REPLENISH_THRESHOLD_CREDITS,
            replenish_amount_usdc: DEFAULT_REPLENISH_AMOUNT_USDC,
            max_replenish_per_day: DEFAULT_MAX_REPLENISH_PER_DAY,
        }
    }
}

/// Self-funding configuration for the metabolic loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfFundingConfig {
    /// Enable the metabolic loop.
    pub enabled: bool,
    /// Fraction of vault revenue allocated to compute (0.0-1.0).
    pub revenue_allocation_fraction: f64,
    /// Sustainability ratio below which to throttle requests.
    pub throttle_threshold: f64,
    /// Credit count below which to warn the operator.
    pub credit_low_water_mark: u64,
    /// Auto-replenish policy.
    pub auto_replenish: AutoReplenishPolicy,
}

impl Default for SelfFundingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            revenue_allocation_fraction: 0.6,
            throttle_threshold: THROTTLE_RATIO_THRESHOLD,
            credit_low_water_mark: DEFAULT_REPLENISH_THRESHOLD_CREDITS,
            auto_replenish: AutoReplenishPolicy::default(),
        }
    }
}

/// Controls what inference tier is accessible based on sustainability ratio.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThrottlePolicy {
    /// ratio >= throttle_threshold: unrestricted access.
    FullAccess,
    /// EMERGENCY_RATIO_THRESHOLD <= ratio < throttle_threshold.
    ReducedThroughput { factor: f64 },
    /// ratio < EMERGENCY_RATIO_THRESHOLD: only exempt subsystems.
    EmergencyOnly,
}

impl ThrottlePolicy {
    /// Derive policy from the sustainability ratio and config.
    pub fn from_ratio(ratio: f64, config: &SelfFundingConfig) -> Self {
        if ratio >= config.throttle_threshold {
            ThrottlePolicy::FullAccess
        } else if ratio >= EMERGENCY_RATIO_THRESHOLD {
            ThrottlePolicy::ReducedThroughput {
                factor: ratio / config.throttle_threshold,
            }
        } else {
            ThrottlePolicy::EmergencyOnly
        }
    }

    pub fn allows_t2_deep(&self) -> bool {
        matches!(self, ThrottlePolicy::FullAccess)
    }

    pub fn allows_t2_balanced(&self) -> bool {
        matches!(
            self,
            ThrottlePolicy::FullAccess | ThrottlePolicy::ReducedThroughput { .. }
        )
    }

    pub fn allows_t1(&self) -> bool {
        matches!(
            self,
            ThrottlePolicy::FullAccess | ThrottlePolicy::ReducedThroughput { .. }
        )
    }
}

/// Top-level Bankr provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankrProviderConfig {
    /// Bankr API base URL.
    pub base_url: String,
    /// Bankr API key.
    pub api_key: String,
    /// Wallet identifier for credit management.
    pub wallet_id: String,
    /// Model mapping configuration.
    pub model_mapping: BankrModelMapping,
    /// Self-funding configuration.
    pub self_funding: SelfFundingConfig,
    /// Metabolic loop monitor config.
    pub metabolic: MetabolicLoopConfig,
}

impl Default for BankrProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.bankr.ai/v1".to_string(),
            api_key: String::new(),
            wallet_id: String::new(),
            model_mapping: BankrModelMapping::default(),
            self_funding: SelfFundingConfig::default(),
            metabolic: MetabolicLoopConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_policy_full_access() {
        let config = SelfFundingConfig::default();
        let policy = ThrottlePolicy::from_ratio(1.5, &config);
        assert_eq!(policy, ThrottlePolicy::FullAccess);
        assert!(policy.allows_t2_deep());
        assert!(policy.allows_t2_balanced());
        assert!(policy.allows_t1());
    }

    #[test]
    fn test_throttle_policy_reduced() {
        let config = SelfFundingConfig::default();
        let policy = ThrottlePolicy::from_ratio(0.7, &config);
        assert!(matches!(policy, ThrottlePolicy::ReducedThroughput { .. }));
        assert!(!policy.allows_t2_deep());
        assert!(policy.allows_t2_balanced());
        assert!(policy.allows_t1());
    }

    #[test]
    fn test_throttle_policy_emergency() {
        let config = SelfFundingConfig::default();
        let policy = ThrottlePolicy::from_ratio(0.3, &config);
        assert_eq!(policy, ThrottlePolicy::EmergencyOnly);
        assert!(!policy.allows_t2_deep());
        assert!(!policy.allows_t2_balanced());
        assert!(!policy.allows_t1());
    }

    #[test]
    fn test_throttle_reduced_factor() {
        let config = SelfFundingConfig::default();
        let policy = ThrottlePolicy::from_ratio(0.7, &config);
        if let ThrottlePolicy::ReducedThroughput { factor } = policy {
            assert!((factor - 0.7).abs() < 1e-10);
        } else {
            panic!("expected ReducedThroughput");
        }
    }

    #[test]
    fn test_auto_replenish_defaults() {
        let policy = AutoReplenishPolicy::default();
        assert_eq!(
            policy.threshold_credits,
            DEFAULT_REPLENISH_THRESHOLD_CREDITS
        );
        assert_eq!(policy.replenish_amount_usdc, DEFAULT_REPLENISH_AMOUNT_USDC);
        assert_eq!(policy.max_replenish_per_day, DEFAULT_MAX_REPLENISH_PER_DAY);
    }
}

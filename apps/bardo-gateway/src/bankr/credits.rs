//! Credit conversion and balance tracking for the Bankr provider.
//!
//! Converts vault fee USDC payments into Bankr compute credits.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Exchange rate between USDC and Bankr compute credits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConversionRate {
    /// Credits issued per 1 USDC.
    pub credits_per_usdc: f64,
    /// Unix timestamp when this rate was fetched.
    pub fetched_at: u64,
}

impl Default for ConversionRate {
    fn default() -> Self {
        Self {
            credits_per_usdc: 1000.0,
            fetched_at: 0,
        }
    }
}

impl ConversionRate {
    pub fn usdc_to_credits(&self, usdc: f64) -> u64 {
        (usdc * self.credits_per_usdc) as u64
    }

    pub fn credits_to_usdc(&self, credits: u64) -> f64 {
        credits as f64 / self.credits_per_usdc
    }
}

/// Snapshot of the agent's Bankr credit ledger.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CreditBalance {
    /// Credits available for immediate inference use.
    pub current_credits: u64,
    /// Credits from a pending top-up transaction (not yet confirmed).
    pub pending_credits: u64,
    /// Credits consumed since last reset.
    pub spent_credits: u64,
}

impl CreditBalance {
    pub fn available(&self) -> u64 {
        self.current_credits
    }

    pub fn total_ever_held(&self) -> u64 {
        self.current_credits + self.pending_credits + self.spent_credits
    }

    /// Deduct credits for a request. Returns false if insufficient.
    pub fn deduct(&mut self, amount: u64) -> bool {
        if self.current_credits < amount {
            return false;
        }
        self.current_credits -= amount;
        self.spent_credits += amount;
        true
    }

    /// Add credits (from top-up or conversion).
    pub fn deposit(&mut self, amount: u64) {
        self.current_credits += amount;
    }
}

/// Converts vault fee USDC into Bankr compute credits.
pub struct VaultFeeConverter {
    rate: Arc<RwLock<ConversionRate>>,
    balance: Arc<RwLock<CreditBalance>>,
}

impl VaultFeeConverter {
    pub fn new(initial_rate: ConversionRate, initial_balance: CreditBalance) -> Self {
        Self {
            rate: Arc::new(RwLock::new(initial_rate)),
            balance: Arc::new(RwLock::new(initial_balance)),
        }
    }

    /// Convert a USDC amount to credits at current rate.
    pub async fn usdc_to_credits(&self, usdc: f64) -> u64 {
        let rate = *self.rate.read().await;
        rate.usdc_to_credits(usdc)
    }

    /// Read current balance.
    pub async fn current_balance(&self) -> CreditBalance {
        *self.balance.read().await
    }

    /// Update the conversion rate.
    pub async fn update_rate(&self, new_rate: ConversionRate) {
        *self.rate.write().await = new_rate;
    }

    /// Deposit credits into the balance.
    pub async fn deposit_credits(&self, amount: u64) {
        self.balance.write().await.deposit(amount);
    }

    /// Get the shared balance handle.
    pub fn balance_handle(&self) -> Arc<RwLock<CreditBalance>> {
        self.balance.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_rate_usdc_to_credits() {
        let rate = ConversionRate {
            credits_per_usdc: 1000.0,
            fetched_at: 0,
        };
        assert_eq!(rate.usdc_to_credits(5.0), 5000);
    }

    #[test]
    fn test_conversion_rate_credits_to_usdc() {
        let rate = ConversionRate {
            credits_per_usdc: 1000.0,
            fetched_at: 0,
        };
        assert!((rate.credits_to_usdc(2500) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_credit_balance_available() {
        let balance = CreditBalance {
            current_credits: 800,
            pending_credits: 200,
            spent_credits: 500,
        };
        assert_eq!(balance.available(), 800);
        assert_eq!(balance.total_ever_held(), 1500);
    }

    #[test]
    fn test_credit_balance_deduct() {
        let mut balance = CreditBalance {
            current_credits: 100,
            pending_credits: 0,
            spent_credits: 0,
        };
        assert!(balance.deduct(50));
        assert_eq!(balance.current_credits, 50);
        assert_eq!(balance.spent_credits, 50);
        assert!(!balance.deduct(60));
        assert_eq!(balance.current_credits, 50);
    }

    #[test]
    fn test_credit_balance_deposit() {
        let mut balance = CreditBalance::default();
        balance.deposit(1000);
        assert_eq!(balance.current_credits, 1000);
    }
}

//! DIEM budget tracking for VVV staking.
//!
//! Tracks per-category consumption within a daily DIEM allocation.
//! Four categories: Waking (60%), Dream (15%), Sleepwalker (15%), Reserve (10%).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// DIEM budget allocation categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiemCategory {
    /// Primary inference workload (60%).
    Waking,
    /// Background/dream inference (15%).
    Dream,
    /// Observatory research, sleepwalker phenotype only (15%).
    Sleepwalker,
    /// Emergency reserve, rolls over to next day (10%).
    Reserve,
}

/// Budget allocation fractions per category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiemAllocations {
    pub waking_fraction: f64,
    pub dream_fraction: f64,
    pub sleepwalker_fraction: f64,
    pub rollover_fraction: f64,
}

impl Default for DiemAllocations {
    fn default() -> Self {
        Self {
            waking_fraction: 0.60,
            dream_fraction: 0.15,
            sleepwalker_fraction: 0.15,
            rollover_fraction: 0.10,
        }
    }
}

impl DiemAllocations {
    pub fn fraction_for(&self, category: DiemCategory) -> f64 {
        match category {
            DiemCategory::Waking => self.waking_fraction,
            DiemCategory::Dream => self.dream_fraction,
            DiemCategory::Sleepwalker => self.sleepwalker_fraction,
            DiemCategory::Reserve => self.rollover_fraction,
        }
    }

    pub fn sum(&self) -> f64 {
        self.waking_fraction
            + self.dream_fraction
            + self.sleepwalker_fraction
            + self.rollover_fraction
    }
}

/// Configuration for the DIEM tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiemConfig {
    /// Daily DIEM allocation in USD (computed from pro-rata VVV stake share).
    pub daily_diem_allocation: f64,
    /// Budget split across categories.
    pub allocations: DiemAllocations,
}

impl Default for DiemConfig {
    fn default() -> Self {
        Self {
            daily_diem_allocation: 10.0,
            allocations: DiemAllocations::default(),
        }
    }
}

/// Budget error when a category is exhausted.
#[derive(Debug, thiserror::Error)]
pub enum DiemBudgetError {
    #[error("diem category {category:?} exhausted; {remaining:.4} usd remaining")]
    CategoryExhausted {
        category: DiemCategory,
        remaining: f64,
    },
}

/// Tracks DIEM consumption per category within a daily budget.
#[derive(Debug)]
pub struct DiemTracker {
    config: DiemConfig,
    /// Per-category consumption (USD used today).
    consumed_by_category: HashMap<DiemCategory, f64>,
    /// Unused reserve DIEM carried from previous days.
    rollover_balance: f64,
}

impl DiemTracker {
    pub fn new(config: DiemConfig) -> Self {
        Self {
            config,
            consumed_by_category: HashMap::new(),
            rollover_balance: 0.0,
        }
    }

    pub fn has_balance(&self) -> bool {
        self.remaining_today() > 0.0
    }

    pub fn remaining_today(&self) -> f64 {
        let total = self.config.daily_diem_allocation + self.rollover_balance;
        let consumed: f64 = self.consumed_by_category.values().sum();
        (total - consumed).max(0.0)
    }

    pub fn remaining_for_category(&self, category: DiemCategory) -> f64 {
        let budget =
            self.config.daily_diem_allocation * self.config.allocations.fraction_for(category);
        let consumed = self
            .consumed_by_category
            .get(&category)
            .copied()
            .unwrap_or(0.0);
        (budget - consumed).max(0.0)
    }

    /// Records inference consumption. Returns Err if over budget for category.
    pub fn consume(
        &mut self,
        cost_usd: f64,
        category: DiemCategory,
    ) -> Result<(), DiemBudgetError> {
        if cost_usd > self.remaining_for_category(category) {
            return Err(DiemBudgetError::CategoryExhausted {
                category,
                remaining: self.remaining_for_category(category),
            });
        }
        *self.consumed_by_category.entry(category).or_insert(0.0) += cost_usd;
        Ok(())
    }

    /// End-of-day rollover. Reserve fraction carries forward; other categories reset.
    pub fn end_of_day(&mut self) {
        let reserve_budget =
            self.config.daily_diem_allocation * self.config.allocations.rollover_fraction;
        let reserve_consumed = self
            .consumed_by_category
            .get(&DiemCategory::Reserve)
            .copied()
            .unwrap_or(0.0);
        self.rollover_balance = (reserve_budget - reserve_consumed).max(0.0);
        self.consumed_by_category.clear();
    }

    pub fn rollover_balance(&self) -> f64 {
        self.rollover_balance
    }

    pub fn daily_allocation(&self) -> f64 {
        self.config.daily_diem_allocation
    }

    pub fn consumed_total(&self) -> f64 {
        self.consumed_by_category.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_tracker() -> DiemTracker {
        DiemTracker::new(DiemConfig {
            daily_diem_allocation: 10.0,
            allocations: DiemAllocations::default(),
        })
    }

    #[test]
    fn test_diem_allocation_sum() {
        let alloc = DiemAllocations::default();
        let sum = alloc.sum();
        assert!((sum - 1.0).abs() < f64::EPSILON * 4.0);
    }

    #[test]
    fn test_diem_category_remaining_initial() {
        let tracker = default_tracker();
        // Waking: 10 * 0.60 = 6.0
        assert!((tracker.remaining_for_category(DiemCategory::Waking) - 6.0).abs() < 1e-10);
        // Dream: 10 * 0.15 = 1.5
        assert!((tracker.remaining_for_category(DiemCategory::Dream) - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_diem_category_nonnegative() {
        let mut tracker = default_tracker();
        // Exhaust waking budget.
        let _ = tracker.consume(6.0, DiemCategory::Waking);
        assert!(tracker.remaining_for_category(DiemCategory::Waking) >= 0.0);
        // Attempt to over-consume.
        let result = tracker.consume(0.01, DiemCategory::Waking);
        assert!(result.is_err());
    }

    #[test]
    fn test_diem_consumption_monotonic() {
        let mut tracker = default_tracker();
        let before = tracker.remaining_for_category(DiemCategory::Waking);
        tracker.consume(1.0, DiemCategory::Waking).unwrap();
        let after = tracker.remaining_for_category(DiemCategory::Waking);
        assert!(after < before);
    }

    #[test]
    fn test_diem_end_of_day() {
        let mut tracker = default_tracker();
        // Consume some waking and leave reserve untouched.
        tracker.consume(3.0, DiemCategory::Waking).unwrap();
        tracker.end_of_day();

        // Rollover should be the full reserve budget (10 * 0.10 = 1.0).
        assert!((tracker.rollover_balance() - 1.0).abs() < 1e-10);
        // Consumed should be cleared.
        assert_eq!(tracker.consumed_total(), 0.0);
    }

    #[test]
    fn test_diem_end_of_day_partial_reserve() {
        let mut tracker = default_tracker();
        // Consume half the reserve budget.
        tracker.consume(0.5, DiemCategory::Reserve).unwrap();
        tracker.end_of_day();

        // Rollover = max(0, 1.0 - 0.5) = 0.5
        assert!((tracker.rollover_balance() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_diem_conservation() {
        let mut tracker = default_tracker();
        tracker.consume(2.0, DiemCategory::Waking).unwrap();
        tracker.consume(0.5, DiemCategory::Dream).unwrap();

        let consumed = tracker.consumed_total();
        let remaining = tracker.remaining_today();
        let total = tracker.daily_allocation() + tracker.rollover_balance();
        assert!((consumed + remaining - total).abs() < 1e-10);
    }

    #[test]
    fn test_dream_budget_independent() {
        let mut tracker = default_tracker();
        // Exhaust waking.
        tracker.consume(6.0, DiemCategory::Waking).unwrap();
        // Dream should still have its full budget.
        assert!((tracker.remaining_for_category(DiemCategory::Dream) - 1.5).abs() < 1e-10);
        tracker.consume(1.0, DiemCategory::Dream).unwrap();
        assert!((tracker.remaining_for_category(DiemCategory::Dream) - 0.5).abs() < 1e-10);
    }
}

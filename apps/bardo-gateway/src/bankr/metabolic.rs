//! Metabolic loop monitor -- tracks the sustainability ratio that determines
//! whether a Golem is self-funding its inference costs from vault revenue.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// The single number that determines metabolic health.
///
/// `ratio = total_daily_revenue / total_daily_inference_cost`
/// >= 1.0 means self-sustaining; the economic death clock stops.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SustainabilityRatio {
    /// Total daily revenue (all sources, USD).
    pub total_daily_revenue: f64,
    /// Total daily inference cost (USD).
    pub total_daily_inference_cost: f64,
    /// Computed ratio. f64::INFINITY when inference_cost == 0.
    pub ratio: f64,
    /// True when ratio >= 1.0.
    pub is_self_sustaining: bool,
}

impl SustainabilityRatio {
    pub fn compute(total_daily_revenue: f64, total_daily_inference_cost: f64) -> Self {
        let ratio = if total_daily_inference_cost > 0.0 {
            total_daily_revenue / total_daily_inference_cost
        } else {
            f64::INFINITY
        };
        Self {
            total_daily_revenue,
            total_daily_inference_cost,
            ratio,
            is_self_sustaining: ratio >= 1.0,
        }
    }

    pub fn zero() -> Self {
        Self::compute(0.0, 0.0)
    }
}

impl Default for SustainabilityRatio {
    fn default() -> Self {
        Self::zero()
    }
}

/// Configuration for the metabolic loop monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicLoopConfig {
    /// How often the monitor recomputes the ratio (seconds).
    pub update_interval_secs: u64,
    /// Rolling window for revenue/cost averaging (days).
    pub rolling_window_days: u32,
}

impl Default for MetabolicLoopConfig {
    fn default() -> Self {
        Self {
            update_interval_secs: 60,
            rolling_window_days: 7,
        }
    }
}

/// Tracks metabolic loop health via a shared sustainability ratio.
pub struct MetabolicLoopMonitor {
    current: Arc<RwLock<SustainabilityRatio>>,
    _config: MetabolicLoopConfig,
}

impl MetabolicLoopMonitor {
    pub fn new(config: MetabolicLoopConfig) -> Self {
        Self {
            current: Arc::new(RwLock::new(SustainabilityRatio::zero())),
            _config: config,
        }
    }

    /// Update the ratio from fresh revenue/cost data.
    pub async fn update(
        &self,
        total_daily_revenue: f64,
        total_daily_inference_cost: f64,
    ) -> SustainabilityRatio {
        let ratio = SustainabilityRatio::compute(total_daily_revenue, total_daily_inference_cost);
        *self.current.write().await = ratio;
        ratio
    }

    /// Read the most recent ratio without triggering a recompute.
    pub async fn current_ratio(&self) -> SustainabilityRatio {
        *self.current.read().await
    }

    /// Get the shared ratio handle (for BankrProvider to hold a clone).
    pub fn ratio_handle(&self) -> Arc<RwLock<SustainabilityRatio>> {
        self.current.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_self_sustaining() {
        let r = SustainabilityRatio::compute(100.0, 80.0);
        assert_eq!(r.ratio, 1.25);
        assert!(r.is_self_sustaining);
    }

    #[test]
    fn test_ratio_dying() {
        let r = SustainabilityRatio::compute(10.0, 80.0);
        assert_eq!(r.ratio, 0.125);
        assert!(!r.is_self_sustaining);
    }

    #[test]
    fn test_ratio_zero_cost() {
        let r = SustainabilityRatio::compute(100.0, 0.0);
        assert!(r.ratio.is_infinite());
        assert!(r.is_self_sustaining);
    }

    #[test]
    fn test_ratio_both_zero() {
        let r = SustainabilityRatio::compute(0.0, 0.0);
        assert!(r.ratio.is_infinite());
        assert!(r.is_self_sustaining);
    }

    #[test]
    fn test_ratio_zero_revenue() {
        let r = SustainabilityRatio::compute(0.0, 50.0);
        assert_eq!(r.ratio, 0.0);
        assert!(!r.is_self_sustaining);
    }

    #[tokio::test]
    async fn test_monitor_update() {
        let monitor = MetabolicLoopMonitor::new(MetabolicLoopConfig::default());
        let r = monitor.update(100.0, 80.0).await;
        assert!(r.is_self_sustaining);

        let current = monitor.current_ratio().await;
        assert_eq!(current.ratio, r.ratio);
    }
}

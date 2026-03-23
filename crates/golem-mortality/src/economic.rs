//! Economic death clock — USDC balance depletion.

use crate::clock::{ClockContext, ClockEvent, DeathCause, MortalityClock};
use serde::{Deserialize, Serialize};

/// Full economic vitality state including burn rate and partitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomicVitalityState {
    /// Spendable USDC balance (death reserve excluded).
    pub credit_remaining: f64,
    /// Initial USDC credit allocation.
    pub initial_credits: f64,
    /// EMA-smoothed burn rate per tick (alpha = 0.05).
    pub burn_rate_per_tick: f64,
    /// Total USDC spent over lifetime.
    pub lifetime_spent: f64,
    /// Partitioned cost breakdown.
    pub burn_rate_state: BurnRateState,
    /// Locked death fund.
    pub apoptotic_reserve: ApoptoticReserve,
}

/// Component breakdown of burn rate.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BurnRateComponents {
    /// LLM inference cost per tick.
    pub llm: f64,
    /// Gas cost per tick.
    pub gas: f64,
    /// Data feed cost per tick.
    pub data: f64,
}

/// Burn rate tracking state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BurnRateState {
    /// Current EMA burn rate.
    pub current: f64,
    /// Per-component breakdown.
    pub components: BurnRateComponents,
    /// Mean burn rate over lifetime.
    pub lifetime_mean: f64,
    /// Number of ticks observed.
    pub ticks_observed: u64,
}

/// Locked death fund for graceful shutdown.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApoptoticReserve {
    /// Amount locked for death protocol.
    pub locked_amount: f64,
    /// Proportion of initial credits locked.
    pub proportion: f64,
}

impl Default for ApoptoticReserve {
    fn default() -> Self {
        Self {
            locked_amount: 0.0,
            proportion: 0.05,
        }
    }
}

/// Credit partition categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CreditPartition {
    /// LLM inference.
    Llm,
    /// On-chain gas.
    Gas,
    /// Data feeds.
    Data,
    /// General/unpartitioned.
    General,
}

/// Cost profile per regime.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegimeCostProfile {
    /// Expected cost per tick in this regime.
    pub expected_cost_per_tick: f64,
    /// Regime identifier.
    pub regime: String,
}

/// The economic death clock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomicClock {
    /// Current economic state.
    pub state: EconomicVitalityState,
}

impl EconomicClock {
    /// Create a new economic clock with the given initial credits.
    pub fn new(initial_credits: f64, death_reserve_proportion: f64) -> Self {
        let reserve = initial_credits * death_reserve_proportion;
        Self {
            state: EconomicVitalityState {
                credit_remaining: initial_credits - reserve,
                initial_credits,
                burn_rate_per_tick: 0.0,
                lifetime_spent: 0.0,
                burn_rate_state: BurnRateState::default(),
                apoptotic_reserve: ApoptoticReserve {
                    locked_amount: reserve,
                    proportion: death_reserve_proportion,
                },
            },
        }
    }

    /// Estimated ticks until economic death.
    pub fn estimated_ttl_ticks(&self) -> u64 {
        if self.state.burn_rate_per_tick > 0.0 {
            (self.state.credit_remaining / self.state.burn_rate_per_tick) as u64
        } else {
            u64::MAX
        }
    }

    /// Process a tick cost.
    pub fn tick_cost(&mut self, cost: f64) -> f64 {
        self.state.credit_remaining -= cost;
        self.state.lifetime_spent += cost;
        // EMA smoothing: alpha = 0.05.
        self.state.burn_rate_per_tick = self.state.burn_rate_per_tick * 0.95 + cost * 0.05;

        // Update burn rate state.
        self.state.burn_rate_state.current = self.state.burn_rate_per_tick;
        self.state.burn_rate_state.ticks_observed += 1;
        let n = self.state.burn_rate_state.ticks_observed as f64;
        self.state.burn_rate_state.lifetime_mean =
            self.state.burn_rate_state.lifetime_mean * ((n - 1.0) / n) + cost / n;

        // Return current vitality.
        (self.state.credit_remaining.max(0.0) / self.state.initial_credits).clamp(0.0, 1.0)
    }
}

/// Compute burn rate with EMA smoothing.
pub fn compute_burn_rate(current_rate: f64, tick_cost: f64, alpha: f64) -> f64 {
    current_rate * (1.0 - alpha) + tick_cost * alpha
}

/// Compute apoptotic reserve from initial credits.
pub fn compute_apoptotic_reserve(initial_credits: f64, proportion: f64) -> ApoptoticReserve {
    ApoptoticReserve {
        locked_amount: initial_credits * proportion,
        proportion,
    }
}

/// Compute dynamic death reserve based on current burn rate.
pub fn compute_dynamic_death_reserve(burn_rate: f64, min_ticks: u64, floor: f64) -> f64 {
    (burn_rate * min_ticks as f64).max(floor)
}

/// Compute survival pressure using sigmoid centered at 48 projected life hours.
pub fn compute_survival_pressure(
    credit_remaining: f64,
    burn_rate_per_tick: f64,
    ticks_per_hour: f64,
) -> f64 {
    if burn_rate_per_tick <= 0.0 {
        return 0.0;
    }
    let projected_hours = credit_remaining / (burn_rate_per_tick * ticks_per_hour);
    crate::vitality::sigmoid(projected_hours, 48.0, 0.1)
}

/// Rebalance credit partitions based on current regime.
pub fn predictive_rebalance(
    _credit_remaining: f64,
    _regime: &RegimeCostProfile,
) -> BurnRateComponents {
    // Stub: future plans implement regime-aware rebalancing.
    BurnRateComponents::default()
}

impl MortalityClock for EconomicClock {
    fn vitality(&self) -> f64 {
        (self.state.credit_remaining.max(0.0) / self.state.initial_credits).clamp(0.0, 1.0)
    }

    fn tick(&mut self, _ctx: &ClockContext) -> ClockEvent {
        // The actual cost is passed via tick_cost(); this advances with zero cost.
        let v = self.vitality();
        if v <= 0.0 {
            ClockEvent::Dead {
                cause: DeathCause::Economic,
            }
        } else {
            ClockEvent::Alive { vitality: v }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    #[test]
    fn economic_clock_depletes_to_zero() {
        let mut clock = EconomicClock::new(100.0, 0.0);
        for _ in 0..100 {
            clock.tick_cost(1.0);
        }
        assert!(clock.vitality() < EPS);
    }

    #[test]
    fn burn_rate_ema_smoothing() {
        let mut clock = EconomicClock::new(10000.0, 0.0);
        // Feed constant cost of 1.0 for many ticks.
        for _ in 0..200 {
            clock.tick_cost(1.0);
        }
        // EMA should converge near 1.0.
        assert!(
            (clock.state.burn_rate_per_tick - 1.0).abs() < 0.01,
            "burn_rate={}, expected ~1.0",
            clock.state.burn_rate_per_tick
        );
    }

    #[test]
    fn test_burn_rate_convergence_60_ticks() {
        let mut rate = 0.0;
        for _ in 0..60 {
            rate = compute_burn_rate(rate, 2.0, 0.05);
        }
        // After 60 ticks at constant cost=2.0, should be close to 2.0.
        assert!((rate - 2.0).abs() < 0.15, "rate={rate}, expected near 2.0");
    }

    #[test]
    fn test_economic_vitality_clamp_bounds() {
        let mut clock = EconomicClock::new(1000.0, 0.0);
        // Overspend.
        clock.tick_cost(2000.0);
        assert!(clock.vitality() >= 0.0);
        assert!(clock.vitality() <= 1.0);
    }

    #[test]
    fn test_economic_vitality_normalization() {
        let cases: [(f64, f64); 4] = [(0.0, 0.0), (250.0, 0.25), (500.0, 0.5), (1000.0, 1.0)];
        for (credit, expected) in cases {
            let mut clock = EconomicClock::new(1000.0, 0.0);
            clock.state.credit_remaining = credit;
            assert!(
                (clock.vitality() - expected).abs() < 1e-10,
                "credit={credit}: got {}, expected {expected}",
                clock.vitality()
            );
        }
    }

    #[test]
    fn test_estimated_ttl_ticks_calculation() {
        let mut clock = EconomicClock::new(1000.0, 0.0);
        clock.state.credit_remaining = 100.0;
        clock.state.burn_rate_per_tick = 1.0;
        assert_eq!(clock.estimated_ttl_ticks(), 100);
    }

    #[test]
    fn test_estimated_ttl_ticks_zero_burn() {
        let clock = EconomicClock::new(1000.0, 0.0);
        assert_eq!(clock.estimated_ttl_ticks(), u64::MAX);
    }
}

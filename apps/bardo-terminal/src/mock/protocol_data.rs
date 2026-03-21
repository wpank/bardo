//! Placeholder protocol shapes for the Protocol Views screen.
//!
//! Each type exposes `mock_default()` with plausible sample values for UI work.

// Constructors are used by upcoming protocol widgets; unit tests exercise them today.
#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

/// Current tick relative to a liquidity range on a Uniswap-style pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockTickRange {
    /// Lower tick bound of the active range.
    // TODO(plan-70a): replace with live pool range from chain intelligence
    pub(crate) lower_tick: i32,
    /// Upper tick bound of the active range.
    pub(crate) upper_tick: i32,
    /// Last observed current tick.
    pub(crate) current_tick: i32,
}

impl MockTickRange {
    /// Sample range with the current tick centered in-band.
    pub(crate) fn mock_default() -> Self {
        Self {
            lower_tick: -887_220,
            upper_tick: -887_200,
            current_tick: -887_210,
        }
    }

    /// Fraction in `[0.0, 1.0]` where the current tick sits between bounds, after clamping.
    ///
    /// Returns [`None`] when `lower_tick >= upper_tick`.
    pub(crate) fn position_fraction(&self) -> Option<f64> {
        if self.lower_tick >= self.upper_tick {
            return None;
        }
        let span = f64::from(self.upper_tick - self.lower_tick);
        let clamped = self.current_tick.clamp(self.lower_tick, self.upper_tick);
        let offset = f64::from(clamped - self.lower_tick);
        Some(offset / span)
    }

    /// Whether the current tick lies inside `[lower_tick, upper_tick]`.
    pub(crate) fn is_in_range(&self) -> bool {
        self.current_tick >= self.lower_tick && self.current_tick <= self.upper_tick
    }
}

/// Uniswap-style pool snapshot for the pool widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockPoolState {
    // TODO(plan-70a): wire Uniswap pool subscriber / quote API
    pub(crate) base_symbol: String,
    pub(crate) quote_symbol: String,
    pub(crate) fee_bps: u16,
    pub(crate) chain: String,
    /// Price of one unit of base in quote terms (display units).
    pub(crate) price_quote: f64,
    pub(crate) tick_range: MockTickRange,
    /// Normalized depth samples in `[0.0, 1.0]` for braille sparklines.
    pub(crate) depth_samples: Vec<f64>,
    pub(crate) tvl_usd: f64,
    pub(crate) volume_24h_usd: f64,
}

impl MockPoolState {
    pub(crate) fn mock_default() -> Self {
        Self {
            base_symbol: "ETH".to_string(),
            quote_symbol: "USDC".to_string(),
            fee_bps: 5,
            chain: "Base".to_string(),
            price_quote: 3_421.50,
            tick_range: MockTickRange::mock_default(),
            depth_samples: vec![0.12, 0.35, 0.58, 0.72, 0.91, 0.65, 0.40, 0.22],
            tvl_usd: 128_400_000.0,
            volume_24h_usd: 42_100_000.0,
        }
    }
}

/// Lending market snapshot (Aave / Morpho / Compound-style).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockLendingMarket {
    // TODO(plan-70b): wire lending protocol market data
    pub(crate) protocol: String,
    pub(crate) asset_symbol: String,
    pub(crate) chain: String,
    /// Utilization ratio in `[0.0, 1.0]`.
    pub(crate) utilization: f64,
    pub(crate) supply_apy: f64,
    pub(crate) borrow_apy: f64,
    pub(crate) total_supplied_usd: f64,
    pub(crate) total_borrowed_usd: f64,
}

impl MockLendingMarket {
    pub(crate) fn mock_default() -> Self {
        Self {
            protocol: "Aave V3".to_string(),
            asset_symbol: "USDC".to_string(),
            chain: "Base".to_string(),
            utilization: 0.72,
            supply_apy: 0.042,
            borrow_apy: 0.061,
            total_supplied_usd: 240_000_000.0,
            total_borrowed_usd: 172_800_000.0,
        }
    }
}

/// ERC-4626 vault snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockVaultState {
    // TODO(plan-70b): wire vault share price / TVL feeds
    pub(crate) vault_name: String,
    /// Protocol brand, e.g. "Beefy", "Yearn" (used when the panel title is abbreviated).
    pub(crate) protocol_name: String,
    pub(crate) chain: String,
    pub(crate) asset_symbol: String,
    pub(crate) nav_per_share: f64,
    pub(crate) tvl_usd: f64,
    pub(crate) apy: f64,
    /// Relative change in share price over 24h (e.g. `0.0031` = +0.31%).
    pub(crate) share_price_24h_change: f64,
}

impl MockVaultState {
    pub(crate) fn mock_default() -> Self {
        Self {
            vault_name: "Beefy USDC/ETH".to_string(),
            protocol_name: "Beefy".to_string(),
            chain: "Base".to_string(),
            asset_symbol: "USDC".to_string(),
            nav_per_share: 1.0842,
            tvl_usd: 56_200_000.0,
            apy: 0.148,
            share_price_24h_change: 0.0031,
        }
    }
}

/// High-level bridge transfer status for the bridge widget badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum MockBridgeStatus {
    Quoted,
    Pending,
    InFlight,
    Complete,
    Failed,
}

/// Single bridge route / transfer row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockBridgeRoute {
    // TODO(plan-70c): wire bridge relayer / status API
    pub(crate) bridge_name: String,
    pub(crate) source_chain: String,
    pub(crate) dest_chain: String,
    pub(crate) token_symbol: String,
    pub(crate) amount: f64,
    pub(crate) fee_usd: f64,
    pub(crate) eta_seconds: u64,
    pub(crate) status: MockBridgeStatus,
    /// Elapsed seconds while in flight (for progress).
    pub(crate) elapsed_seconds: Option<u64>,
    /// Estimated total seconds for the transfer (for progress).
    pub(crate) estimated_seconds: Option<u64>,
}

impl MockBridgeRoute {
    pub(crate) fn mock_default() -> Self {
        Self {
            bridge_name: "Across".to_string(),
            source_chain: "Ethereum".to_string(),
            dest_chain: "Base".to_string(),
            token_symbol: "ETH".to_string(),
            amount: 2.5,
            fee_usd: 4.85,
            eta_seconds: 180,
            status: MockBridgeStatus::InFlight,
            elapsed_seconds: Some(90),
            estimated_seconds: Some(180),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_pool_state_tick_range_fraction() {
        let pool = MockPoolState::mock_default();
        let frac = pool
            .tick_range
            .position_fraction()
            .expect("valid default tick range");
        assert!(frac > 0.0 && frac < 1.0, "default mock should be mid-range");
        assert!(pool.tick_range.is_in_range());
    }

    #[test]
    fn test_mock_tick_range_position_fraction_none_when_invalid() {
        let range = MockTickRange {
            lower_tick: 100,
            upper_tick: 100,
            current_tick: 100,
        };
        assert!(range.position_fraction().is_none());
    }

    #[test]
    fn test_mock_pool_depth_samples_normalized() {
        let pool = MockPoolState::mock_default();
        for sample in &pool.depth_samples {
            assert!((0.0..=1.0).contains(sample));
        }
    }

    #[test]
    fn test_mock_lending_utilization_in_unit_interval() {
        let m = MockLendingMarket::mock_default();
        assert!((0.0..=1.0).contains(&m.utilization));
    }

    #[test]
    fn test_mock_bridge_route_in_flight_has_timing() {
        let route = MockBridgeRoute::mock_default();
        assert_eq!(route.status, MockBridgeStatus::InFlight);
        assert!(route.elapsed_seconds.is_some());
        assert!(route.estimated_seconds.is_some());
    }

    #[test]
    fn test_mock_vault_state_defaults() {
        let vault = MockVaultState::mock_default();
        assert!(vault.nav_per_share > 0.0);
        assert!(vault.tvl_usd > 0.0);
    }
}

//! Placeholder protocol shapes for the Protocol Views screen.
//!
//! Each type exposes `mock_default()` with plausible sample values for UI work.
//! Plans 70a-70c will replace each field's source with a real subsystem connection.

// Constructors are used by upcoming protocol widgets; unit tests exercise them today.
#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tick range
// ---------------------------------------------------------------------------

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

    /// Fraction in `[0.0, 1.0]` where the current tick sits between bounds,
    /// clamped to the range edges.
    ///
    /// Returns [`None`] when `lower_tick >= upper_tick` (degenerate range).
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

// ---------------------------------------------------------------------------
// Pool state
// ---------------------------------------------------------------------------

/// Placeholder Uniswap V3/V4 pool state.
/// TODO(plan-70a): replace with golem_chain_intelligence::protocol_state::UniswapPoolState
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockPoolState {
    /// Token symbols, e.g. ("ETH", "USDC").
    pub(crate) token0_symbol: String,
    pub(crate) token1_symbol: String,
    /// Current price of token0 in terms of token1.
    /// TODO(plan-70a): from sqrtPriceX96 via golem-chain-intelligence
    pub(crate) current_price: f64,
    /// Current tick (i24 on-chain, i32 here for convenience).
    /// TODO(plan-70a): from slot0.tick
    pub(crate) current_tick: i32,
    /// Tick range where most liquidity is concentrated.
    pub(crate) tick_range: MockTickRange,
    /// Active in-range liquidity (u128 on-chain, u128 here).
    /// TODO(plan-70a): from pool.liquidity()
    pub(crate) active_liquidity: u128,
    /// Pool fee tier in basis points (e.g. 30 = 0.30%, 5 = 0.05%, 100 = 1.00%).
    pub(crate) fee_tier_bps: u16,
    /// Total value locked in USD.
    /// TODO(plan-70a): from Uniswap subgraph or golem-chain-intelligence
    pub(crate) tvl_usd: f64,
    /// 24-hour trading volume in USD.
    /// TODO(plan-70a): from Uniswap subgraph
    pub(crate) volume_24h_usd: f64,
    /// Liquidity depth samples for sparkline. Each value is a relative liquidity
    /// density at evenly-spaced tick positions across the display range.
    /// Normalized to `[0.0, 1.0]` relative to the maximum.
    /// TODO(plan-70a): from get_tick_data tool
    pub(crate) depth_samples: Vec<f64>,
    /// Chain name, e.g. "Base", "Ethereum".
    pub(crate) chain: String,
}

impl MockPoolState {
    pub(crate) fn mock_default() -> Self {
        Self {
            token0_symbol: "ETH".into(),
            token1_symbol: "USDC".into(),
            current_price: 3_421.50,
            current_tick: -196_680,
            tick_range: MockTickRange {
                lower_tick: -197_500,
                upper_tick: -195_800,
                current_tick: -196_680,
            },
            active_liquidity: 12_500_000_000_000_000_000,
            fee_tier_bps: 5, // 0.05%
            tvl_usd: 48_200_000.0,
            volume_24h_usd: 3_100_000.0,
            depth_samples: vec![
                0.2, 0.35, 0.55, 0.72, 0.88, 1.0, 0.95, 0.80, 0.60, 0.40, 0.25, 0.15, 0.10, 0.08,
                0.05, 0.04, 0.03, 0.02, 0.01, 0.01,
            ],
            chain: "Base".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Lending market
// ---------------------------------------------------------------------------

/// Placeholder lending market state (Aave V3 / Morpho / Compound).
/// TODO(plan-70a): replace with golem_chain_intelligence::protocol_state::LendingMarketState
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockLendingMarket {
    /// Protocol name, e.g. "Aave V3", "Morpho", "Compound".
    pub(crate) protocol_name: String,
    /// Underlying asset symbol, e.g. "USDC", "WETH".
    pub(crate) asset_symbol: String,
    /// Utilization rate as a fraction (0.0 = 0%, 1.0 = 100%).
    /// TODO(plan-70a): totalDebt / totalLiquidity from market state
    pub(crate) utilization: f64,
    /// Variable supply APY as a fraction (0.0 = 0%, 0.05 = 5%).
    /// TODO(plan-70a): from get_aave_rates / get_morpho_market_info
    pub(crate) supply_apy: f64,
    /// Variable borrow APY as a fraction.
    /// TODO(plan-70a): from get_aave_rates / get_morpho_market_info
    pub(crate) borrow_apy: f64,
    /// Total supplied liquidity in USD.
    /// TODO(plan-70a): from market totalSupply * oracle price
    pub(crate) total_liquidity_usd: f64,
    /// Total outstanding borrows in USD.
    /// TODO(plan-70a): from market totalDebt * oracle price
    pub(crate) total_borrow_usd: f64,
    /// Chain name.
    pub(crate) chain: String,
}

impl MockLendingMarket {
    pub(crate) fn mock_default() -> Self {
        Self {
            protocol_name: "Aave V3".into(),
            asset_symbol: "USDC".into(),
            utilization: 0.78,
            supply_apy: 0.042,
            borrow_apy: 0.061,
            total_liquidity_usd: 125_000_000.0,
            total_borrow_usd: 97_500_000.0,
            chain: "Base".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Vault state
// ---------------------------------------------------------------------------

/// Placeholder ERC-4626 vault state (Beefy, Yearn, or any ERC-4626 vault).
/// TODO(plan-70a): replace with golem_chain_intelligence::protocol_state::VaultState
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockVaultState {
    /// Vault name, e.g. "Beefy USDC/WETH", "Yearn USDC".
    pub(crate) vault_name: String,
    /// Underlying asset symbol.
    pub(crate) asset_symbol: String,
    /// Net asset value per share in asset units (convertToAssets(1e18) / 1e18).
    /// TODO(plan-70a): from ERC-4626 convertToAssets()
    pub(crate) nav_per_share: f64,
    /// Total value locked in USD.
    /// TODO(plan-70a): totalAssets() * oracle price
    pub(crate) tvl_usd: f64,
    /// 24-hour change in share price as a fraction (positive = gain).
    /// TODO(plan-70a): compare nav_per_share to 24h-ago snapshot
    pub(crate) share_price_24h_change: f64,
    /// Annual percentage yield as a fraction (0.08 = 8%).
    /// TODO(plan-70a): from get_vault_apy tool
    pub(crate) apy: f64,
    /// Protocol name, e.g. "Beefy", "Yearn".
    pub(crate) protocol_name: String,
    /// Chain name.
    pub(crate) chain: String,
}

impl MockVaultState {
    pub(crate) fn mock_default() -> Self {
        Self {
            vault_name: "Beefy USDC/ETH".into(),
            asset_symbol: "USDC".into(),
            nav_per_share: 1.0842,
            tvl_usd: 8_400_000.0,
            share_price_24h_change: 0.0031,
            apy: 0.148,
            protocol_name: "Beefy".into(),
            chain: "Base".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Bridge route
// ---------------------------------------------------------------------------

/// Transfer lifecycle state for a mock bridge route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MockBridgeStatus {
    /// No transfer in progress — showing a quoted route.
    Quoted,
    /// Transfer submitted, waiting for source-chain confirmation.
    Pending,
    /// Source confirmed, waiting for destination fill.
    InFlight,
    /// Transfer complete.
    Complete,
    /// Transfer failed or timed out.
    Failed,
}

/// Placeholder bridge route state (Across, Stargate, Hop).
/// TODO(plan-70a): replace with golem_chain_intelligence::bridge::BridgeRouteState
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MockBridgeRoute {
    /// Source chain name, e.g. "Ethereum".
    pub(crate) from_chain: String,
    /// Destination chain name, e.g. "Base".
    pub(crate) to_chain: String,
    /// Token being bridged, e.g. "USDC".
    pub(crate) token_symbol: String,
    /// Amount being bridged in token units (human-readable, not wei).
    pub(crate) amount: f64,
    /// Bridge fee in USD.
    /// TODO(plan-70a): from get_bridge_quote
    pub(crate) fee_usd: f64,
    /// Estimated completion time in seconds.
    /// TODO(plan-70a): from get_bridge_quote estimated fill time
    pub(crate) estimated_time_secs: u64,
    /// Bridge protocol name, e.g. "Across", "Stargate".
    pub(crate) bridge_name: String,
    /// Current transfer status.
    pub(crate) status: MockBridgeStatus,
}

impl MockBridgeRoute {
    pub(crate) fn mock_default() -> Self {
        Self {
            from_chain: "Ethereum".into(),
            to_chain: "Base".into(),
            token_symbol: "USDC".into(),
            amount: 5_000.0,
            fee_usd: 1.85,
            estimated_time_secs: 18,
            bridge_name: "Across".into(),
            status: MockBridgeStatus::Quoted,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_pool_state_tick_range_fraction() {
        let pool = MockPoolState::mock_default();
        let frac = pool.tick_range.position_fraction().unwrap();
        assert!(frac > 0.0 && frac < 1.0, "default mock should be mid-range");
        assert!(
            pool.tick_range.is_in_range(),
            "default mock current_tick should be in range"
        );
    }

    #[test]
    fn tick_range_degenerate_returns_none() {
        let tr = MockTickRange {
            lower_tick: 100,
            upper_tick: 100,
            current_tick: 100,
        };
        assert!(tr.position_fraction().is_none());
    }

    #[test]
    fn tick_range_clamps_out_of_range() {
        let tr = MockTickRange {
            lower_tick: 0,
            upper_tick: 100,
            current_tick: 200,
        };
        assert_eq!(tr.position_fraction(), Some(1.0));
        assert!(!tr.is_in_range());

        let tr_below = MockTickRange {
            lower_tick: 0,
            upper_tick: 100,
            current_tick: -50,
        };
        assert_eq!(tr_below.position_fraction(), Some(0.0));
        assert!(!tr_below.is_in_range());
    }

    #[test]
    fn mock_lending_market_utilization_valid() {
        let market = MockLendingMarket::mock_default();
        assert!((0.0..=1.0).contains(&market.utilization));
        assert!(
            market.borrow_apy > market.supply_apy,
            "borrow APY must exceed supply APY"
        );
    }

    #[test]
    fn mock_vault_24h_change_signed() {
        let vault = MockVaultState::mock_default();
        assert!(vault.share_price_24h_change.abs() < 1.0);
    }

    #[test]
    fn mock_bridge_status_all_variants() {
        let variants = [
            MockBridgeStatus::Quoted,
            MockBridgeStatus::Pending,
            MockBridgeStatus::InFlight,
            MockBridgeStatus::Complete,
            MockBridgeStatus::Failed,
        ];
        assert_eq!(variants.len(), 5);

        let route = MockBridgeRoute::mock_default();
        assert_eq!(route.status, MockBridgeStatus::Quoted);
    }
}

//! Protocol-specific visualization widgets.

#![allow(dead_code)]
#![allow(unused_imports)]

pub(crate) mod uniswap_pool;
pub(crate) mod vault;

pub(crate) use uniswap_pool::{
    UniswapPoolWidget, UniswapPoolWidgetConfig, format_fee_tier, format_usd_compact,
};
pub(crate) use vault::VaultWidget;

pub(crate) mod lending_market;

pub(crate) use lending_market::LendingMarketWidget;

pub(crate) mod bridge_status;

pub(crate) use bridge_status::{BridgeStatusWidget, BridgeStatusWidgetConfig, flight_progress_pct};

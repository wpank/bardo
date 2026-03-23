//! # MPP — Machine Payment Protocol
//!
//! HTTP 402-based payment protocol for machine-to-machine API billing.
//!
//! MPP defines a three-step flow:
//! 1. **Challenge**: server returns 402 with a [`PaymentRequired`] quote
//! 2. **Credential**: client signs an ERC-3009 authorization and retries with [`PaymentCredential`]
//! 3. **Receipt**: server validates, serves the request, and returns a [`PaymentReceipt`]
//!
//! Two payment intents:
//! - **Charge**: one-shot per-request billing
//! - **Session**: pre-funded balance with per-request draws
//!
//! ## Feature flags
//!
//! - `session` (default): enables [`session::SessionStore`] and session types
//! - `db` (default): enables [`db::MppDb`] for SQLite persistence

pub mod currency;
pub mod error;
pub mod spread;
pub mod types;
pub mod verifier;

#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "db")]
pub mod db;

pub use error::MppError;
pub use types::*;
pub use verifier::{
    BASE_CHAIN_ID, Erc3009Domain, USDC_BASE, verify_authorization, verify_authorization_with_domain,
};

use alloy::primitives::Address;

#[cfg(feature = "session")]
use session::SessionStore;

/// MPP configuration.
#[derive(Debug, Clone)]
pub struct MppConfig {
    /// Whether MPP is enabled.
    pub enabled: bool,
    /// Server's recipient address (receives payments).
    pub recipient_address: Address,
    /// Default spread percentage (0.20 = 20%).
    pub default_spread: f64,
    /// Session TTL in seconds.
    pub session_ttl: u64,
    /// Quote validity in seconds (how long a 402 response is valid).
    pub quote_validity: u64,
}

impl Default for MppConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            recipient_address: Address::ZERO,
            default_spread: 0.20,
            session_ttl: 3600,
            quote_validity: 300,
        }
    }
}

/// Runtime MPP state.
#[cfg(feature = "session")]
pub struct MppState {
    pub config: MppConfig,
    pub sessions: SessionStore,
}

#[cfg(feature = "session")]
impl MppState {
    pub fn new(config: MppConfig) -> Self {
        Self {
            sessions: SessionStore::new(),
            config,
        }
    }
}

//! MPP (Machine Payment Protocol) — server-side payment layer.
//!
//! Implements HTTP 402 challenge/credential/receipt flow for inference billing.
//! Supports two payment intents:
//! - **Charge**: one-shot per-request billing
//! - **Session**: pre-funded streaming pay-as-you-go

pub mod db;
pub mod error;
pub mod estimator;
pub mod middleware;
pub mod session;
pub mod spread;
pub mod types;
pub mod verifier;

pub use error::MppError;
pub use types::*;

use alloy::primitives::Address;
use session::MppSessionStore;

/// MPP configuration.
#[derive(Debug, Clone)]
pub struct MppConfig {
    /// Whether MPP is enabled.
    pub enabled: bool,
    /// Gateway's recipient address (receives payments).
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

/// Runtime MPP state held in AppState.
pub struct MppState {
    pub config: MppConfig,
    pub sessions: MppSessionStore,
}

impl MppState {
    pub fn new(config: MppConfig) -> Self {
        Self {
            sessions: MppSessionStore::new(),
            config,
        }
    }
}

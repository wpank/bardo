//! `golem-chain` — Alloy provider, ERC-8004, Permit2, Warden, revm simulation, block/log types.
//!
//! This crate provides the chain interaction layer for Bardo Golems:
//! - Static chain registry with all 12 supported networks
//! - Cached alloy providers for RPC access
//! - ERC-8004 agent identity registry (read-only, L1)
//! - Warden time-delay safety mechanism
//! - Local EVM simulation via revm

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod identity;
pub mod provider;
pub mod revm_sim;
pub mod warden;

pub use config::{ChainConfig, ChainId, ChainRegistry, ContractAddresses};
pub use error::ChainError;
pub use identity::{AgentIdentity, Capability8004, Erc8004Registry, ServiceEndpoint};
pub use provider::{CacheKey, CachedValue, ChainProvider};
pub use revm_sim::{RevmSimulator, SimRequest, SimResult};
pub use warden::{ActionType, Warden, WardenAction, WardenStatus};

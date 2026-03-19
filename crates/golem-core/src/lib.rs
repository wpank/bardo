//! `golem-core` - shared types, config, `GolemId`, `PADVector`, `MarketRegime`, `CognitiveTier`, `GolemConfig`, `CorticalState`, `EventFabric`, `TaintLabel`, bump allocator.
//!
//! **Implemented by:** Plan 02
//!
//! This crate already contains the configuration scaffold used by later plans.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;

pub use config::GolemConfig;
pub use error::{GolemError, Result};

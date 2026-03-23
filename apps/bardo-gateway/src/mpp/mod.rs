//! MPP (Machine Payment Protocol) — server-side payment layer.
//!
//! Protocol primitives live in the `mpp` crate (aliased as `mpp_core` here
//! to avoid collision with this module name). This module re-exports them
//! and adds gateway-specific middleware and cost estimation.

pub mod estimator;
pub mod middleware;

// Re-export everything from the mpp crate so existing `crate::mpp::*` paths work.
pub use mpp_core::*;

// Re-export sub-modules for paths like `crate::mpp::session::SessionStatus`.
pub use mpp_core::currency;
pub use mpp_core::db;
pub use mpp_core::error;
pub use mpp_core::session;
pub use mpp_core::spread;
pub use mpp_core::types;
pub use mpp_core::verifier;

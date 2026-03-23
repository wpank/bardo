//! Tier-based model routing.
//!
//! Re-exports [`TierRouter`] from `bardo-primitives`, where the implementation
//! and full test suite live. `golem-inference` keeps this module so that
//! `use golem_inference::TierRouter` continues to work unchanged.

pub use bardo_primitives::TierRouter;

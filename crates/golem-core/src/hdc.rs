//! Re-exports `HdcVector` from `bardo-primitives`.
//!
//! All implementation and tests live in `bardo-primitives`. golem-core
//! re-exports the type so that existing `use golem_core::HdcVector` call
//! sites continue to work without change.

pub use bardo_primitives::HdcVector;

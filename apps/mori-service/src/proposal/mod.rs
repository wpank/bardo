//! Proposal generation engine.
//!
//! Takes accumulated draft context and produces a costed, milestoned proposal.

pub mod cost;
pub mod engine;

pub use engine::ProposalEngine;

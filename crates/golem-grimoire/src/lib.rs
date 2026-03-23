//! `golem-grimoire` - `LanceDB` episodic store, `SQLite` semantic store (5 entry types), `PLAYBOOK.md`, four-factor retrieval, curator.
//!
//! **Implemented by:** Plan 01, Plan 12
//!
//! Stub types are provided for downstream crates. Full implementation in a later plan.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod entry;
pub mod writer;

pub use entry::{DecayClass, EntryStatus, GrimoireEntry};
pub use writer::GrimoireWriter;

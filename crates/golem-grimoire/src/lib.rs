//! `golem-grimoire` - `LanceDB` episodic store, `SQLite` semantic store (5 entry types), `PLAYBOOK.md`, four-factor retrieval, curator.
//!
//! **Implemented by:** Plan 01, Plan 12

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod admission;
pub mod decay;
pub mod entry;
pub mod error;
pub mod memetic;
pub mod retrieval;
pub mod substrate;
pub mod writer;

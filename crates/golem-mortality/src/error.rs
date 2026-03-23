//! Crate error type.

use thiserror::Error;

/// Errors from the mortality subsystem.
#[derive(Debug, Error)]
pub enum MortalityError {
    /// Invalid configuration parameter.
    #[error("invalid config: {0}")]
    Config(String),

    /// Clock produced an invalid state.
    #[error("clock error: {0}")]
    Clock(String),

    /// Demurrage computation failed.
    #[error("demurrage error: {0}")]
    Demurrage(String),

    /// Grimoire integration error.
    #[error("grimoire error: {0}")]
    Grimoire(String),
}

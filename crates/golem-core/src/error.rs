//! Error surface for `golem-core`.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for `golem-core`.
pub type Result<T> = std::result::Result<T, GolemError>;

/// Crate-wide error type for configuration and I/O failures.
#[derive(Debug, Error)]
pub enum GolemError {
    /// Configuration parsing or validation failure.
    #[error("configuration error: {0}")]
    Config(String),
    /// Filesystem or other I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing failure.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

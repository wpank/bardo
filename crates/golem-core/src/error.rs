//! Error surface for `golem-core`.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for `golem-core`.
pub type Result<T> = std::result::Result<T, GolemError>;

/// Crate-wide error type for configuration, initialization, and runtime plumbing.
#[derive(Debug, Error)]
pub enum GolemError {
    /// Configuration parsing or validation failure.
    #[error("configuration error: {0}")]
    Config(String),
    /// Initialization failure.
    #[error("initialization error: {0}")]
    Init(String),
    /// Extension hook failure.
    #[error("extension error in '{extension}': {source}")]
    Extension {
        /// Name of the extension that produced the failure.
        extension: String,
        /// Wrapped source error.
        source: anyhow::Error,
    },
    /// Event-fabric failure.
    #[error("event fabric error: {0}")]
    EventFabric(String),
    /// Cortical-state failure.
    #[error("cortical state error: {0}")]
    CorticalState(String),
    /// Filesystem or other I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing failure.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// Serialization failure.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

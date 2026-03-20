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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn error_display_messages() {
        let config_err = GolemError::Config("invalid field".to_string());
        assert!(config_err.to_string().contains("configuration error"));
        assert!(config_err.to_string().contains("invalid field"));

        let init_err = GolemError::Init("failed to start".to_string());
        assert!(init_err.to_string().contains("initialization error"));
        assert!(init_err.to_string().contains("failed to start"));

        let fabric_err = GolemError::EventFabric("buffer full".to_string());
        assert!(fabric_err.to_string().contains("event fabric error"));
        assert!(fabric_err.to_string().contains("buffer full"));

        let cortical_err = GolemError::CorticalState("alignment failed".to_string());
        assert!(cortical_err.to_string().contains("cortical state error"));
        assert!(cortical_err.to_string().contains("alignment failed"));
    }

    #[test]
    fn error_extension_variant() {
        let source = anyhow::anyhow!("inner error message");
        let ext_err = GolemError::Extension {
            extension: "test-extension".to_string(),
            source,
        };
        let msg = ext_err.to_string();
        assert!(msg.contains("extension error"));
        assert!(msg.contains("test-extension"));
        assert!(msg.contains("inner error message"));
    }

    #[test]
    fn error_io_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let golem_err: GolemError = io_err.into();
        let msg = golem_err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn error_toml_conversion() {
        let invalid_toml = "invalid = [unclosed";
        let toml_err = toml::from_str::<toml::Value>(invalid_toml).unwrap_err();
        let golem_err: GolemError = toml_err.into();
        let msg = golem_err.to_string();
        assert!(msg.contains("TOML parse error"));
    }

    #[test]
    fn error_serde_conversion() {
        let invalid_json = "{ invalid json }";
        let serde_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let golem_err: GolemError = serde_err.into();
        let msg = golem_err.to_string();
        assert!(msg.contains("serialization error"));
    }

    #[test]
    fn result_type_alias() {
        fn returns_result() -> Result<u32> {
            Ok(42)
        }
        fn returns_error() -> Result<u32> {
            Err(GolemError::Config("test".to_string()))
        }

        assert_eq!(returns_result().unwrap(), 42);
        assert!(returns_error().is_err());
    }
}

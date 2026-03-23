//! Unified error type for the Grimoire crate.

use thiserror::Error;

/// Unified error type covering all Grimoire failure modes.
#[derive(Debug, Error)]
pub enum GrimoireError {
    /// Storage backend (LanceDB or SQLite) is unavailable.
    #[error("storage unavailable: {0}")]
    StorageUnavailable(String),

    /// Embedding model is not initialized or failed.
    #[error("embedder unavailable: {0}")]
    EmbedderUnavailable(String),

    /// Entry was rejected by the A-MAC admission gate.
    #[error("entry rejected: {reason}")]
    AdmissionRejected {
        /// Why admission failed.
        reason: String,
    },

    /// Curator maintenance cycle failed.
    #[error("curator failed: {0}")]
    CuratorFailed(String),

    /// SQLite operation failed.
    #[error("sqlite: {0}")]
    Sqlite(String),

    /// LanceDB operation failed.
    #[error("lancedb: {0}")]
    Lance(String),

    /// PLAYBOOK.md write failed.
    #[error("playbook: {0}")]
    Playbook(String),

    /// Cycle detected in causal DAG.
    #[error("causal cycle: {0}")]
    CausalCycle(String),

    /// Invalid internal state.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Serialization or deserialization failed.
    #[error("serialization: {0}")]
    Serialization(String),

    /// Requested resource not found.
    #[error("not found: {0}")]
    NotFound(String),
}

impl From<rusqlite::Error> for GrimoireError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sqlite(err.to_string())
    }
}

impl From<serde_json::Error> for GrimoireError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for GrimoireError {
    fn from(err: std::io::Error) -> Self {
        Self::StorageUnavailable(err.to_string())
    }
}

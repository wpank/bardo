//! Error types for the mori-index crate.

/// Errors that can occur during indexing operations.
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    /// Database error from rusqlite.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    /// Failed to parse a source file.
    #[error("parse error in {file}: {message}")]
    Parse {
        /// The file that failed to parse.
        file: String,
        /// Description of the parse failure.
        message: String,
    },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The index has not been initialized (no migration run).
    #[error("index not initialized")]
    NotInitialized,

    /// Embedding model error.
    #[cfg(feature = "embedding")]
    #[error("embedding error: {0}")]
    Embedding(String),

    /// Snapshot read/write error.
    #[cfg(feature = "snapshot")]
    #[error("snapshot error: {0}")]
    Snapshot(String),
}

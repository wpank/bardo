//! Error types for context assembly.

/// Errors from context assembly.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    /// Underlying index error.
    #[error("index error: {0}")]
    Index(#[from] mori_index::error::IndexError),

    /// I/O error reading source files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No results found for the query.
    #[error("no results for query: {0}")]
    NoResults(String),
}

//! MPP-specific error types.

/// Errors from the MPP payment layer.
#[derive(Debug, thiserror::Error)]
pub enum MppError {
    #[error("invalid recipient address")]
    InvalidRecipient,

    #[error("insufficient payment amount: need {needed}, got {got}")]
    InsufficientAmount { needed: String, got: String },

    #[error("authorization expired")]
    ExpiredAuthorization,

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("session not active")]
    SessionNotActive,

    #[error("insufficient session balance")]
    InsufficientBalance,

    #[error("signature recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("invalid credential: {0}")]
    InvalidCredential(String),
}

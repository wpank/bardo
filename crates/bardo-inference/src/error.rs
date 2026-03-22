//! Inference error types.

use serde::{Deserialize, Serialize};

/// Errors from inference operations.
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    /// Request validation failed.
    #[error("validation error: {0}")]
    Validation(String),

    /// Provider returned an error.
    #[error("provider error: {0}")]
    Provider(String),

    /// Request timed out.
    #[error("inference request timed out")]
    Timeout,

    /// Authentication failed.
    #[error("unauthorized")]
    Unauthorized,

    /// T0 tier suppresses inference.
    #[error("T0 tier: inference suppressed")]
    T0Suppressed,

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// JSON error payload matching the Anthropic error shape.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Error details.
    pub error: ErrorDetail,
}

/// Error detail within the payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error type identifier.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-readable message.
    pub message: String,
    /// Machine-readable code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl From<&InferenceError> for ErrorPayload {
    fn from(err: &InferenceError) -> Self {
        let (error_type, message) = match err {
            InferenceError::Validation(msg) => ("validation_error", msg.clone()),
            InferenceError::Provider(msg) => ("provider_error", msg.clone()),
            InferenceError::Timeout => ("timeout", "inference request timed out".into()),
            InferenceError::Unauthorized => ("unauthorized", "invalid or missing API key".into()),
            InferenceError::T0Suppressed => ("t0_suppressed", "T0 tier suppresses inference".into()),
            InferenceError::Internal(msg) => ("internal_error", msg.clone()),
        };
        Self {
            error: ErrorDetail {
                error_type: error_type.into(),
                message,
                code: None,
            },
        }
    }
}

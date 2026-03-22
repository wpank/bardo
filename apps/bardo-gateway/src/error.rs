//! Gateway error types.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bardo_inference::{ErrorPayload, InferenceError};

/// Application-level error that converts to HTTP responses.
#[derive(Debug)]
pub enum AppError {
    /// Request validation failed.
    BadRequest(String),
    /// Authentication failed.
    Unauthorized(String),
    /// Provider returned an error.
    ProviderError(String),
    /// Internal server error.
    Internal(String),
    /// Inference error (wraps the crate error).
    Inference(InferenceError),
}

impl From<InferenceError> for AppError {
    fn from(err: InferenceError) -> Self {
        Self::Inference(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, payload) = match &self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, error_json("bad_request", msg)),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, error_json("unauthorized", msg)),
            Self::ProviderError(msg) => {
                (StatusCode::BAD_GATEWAY, error_json("provider_error", msg))
            }
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_json("internal_error", msg),
            ),
            Self::Inference(err) => {
                let status = match err {
                    InferenceError::Validation(_) => StatusCode::BAD_REQUEST,
                    InferenceError::Unauthorized => StatusCode::UNAUTHORIZED,
                    InferenceError::Timeout => StatusCode::GATEWAY_TIMEOUT,
                    InferenceError::T0Suppressed => StatusCode::OK,
                    _ => StatusCode::BAD_GATEWAY,
                };
                let payload = ErrorPayload::from(err);
                (status, serde_json::to_string(&payload).unwrap_or_default())
            }
        };

        let mut resp = (status, payload).into_response();
        resp.headers_mut()
            .insert("content-type", "application/json".parse().unwrap());
        resp
    }
}

fn error_json(error_type: &str, message: &str) -> String {
    serde_json::json!({
        "error": {
            "type": error_type,
            "message": message
        }
    })
    .to_string()
}

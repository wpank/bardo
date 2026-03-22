//! Authentication middleware.

use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::AppError;
use crate::state::AppState;

/// Middleware that validates the X-Api-Key header.
pub async fn require_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match key {
        Some(k) if k == state.api_key => Ok(next.run(request).await),
        Some(_) => Err(AppError::Unauthorized("invalid API key".into())),
        None => {
            // Also check Authorization: Bearer
            let bearer = request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "));

            match bearer {
                Some(k) if k == state.api_key => Ok(next.run(request).await),
                _ => Err(AppError::Unauthorized("missing API key".into())),
            }
        }
    }
}

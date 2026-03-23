//! Authentication middleware.

use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::AppError;
use crate::mpp::types::AuthMode;
use crate::state::AppState;

/// Middleware that validates the X-Api-Key header.
///
/// When MPP is enabled and no valid API key is found, the request is tagged
/// as `AuthMode::Mpp` and passed through for the MPP middleware to handle.
pub async fn require_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match key {
        Some(k) if k == state.api_key => {
            request.extensions_mut().insert(AuthMode::ApiKey);
            Ok(next.run(request).await)
        }
        Some(_) => Err(AppError::Unauthorized("invalid API key".into())),
        None => {
            // Also check Authorization: Bearer
            let bearer = request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "));

            match bearer {
                Some(k) if k == state.api_key => {
                    request.extensions_mut().insert(AuthMode::ApiKey);
                    Ok(next.run(request).await)
                }
                _ => {
                    // No valid API key. If MPP is enabled, pass through for MPP handling.
                    if state.mpp.is_some() {
                        request.extensions_mut().insert(AuthMode::Mpp);
                        Ok(next.run(request).await)
                    } else {
                        Err(AppError::Unauthorized("missing API key".into()))
                    }
                }
            }
        }
    }
}

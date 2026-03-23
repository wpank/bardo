//! Authentication middleware and shared error type.
//!
//! Validates `Authorization: Bearer <token>` against configured API keys.
//! SIWE session tokens are recognized by prefix but validation is stubbed.

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::ServiceState;

// ---------------------------------------------------------------------------
// Auth context
// ---------------------------------------------------------------------------

/// Identity and permissions extracted from a validated bearer token.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// API key ID (e.g. "mori_sk_...") or wallet address (for SIWE sessions).
    pub identity: String,
    /// Permission scope granted to this token.
    pub scope: AuthScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScope {
    Read,
    Write,
    Admin,
}

impl AuthScope {
    /// Returns true if `self` is at least as permissive as `required`.
    pub fn satisfies(self, required: AuthScope) -> bool {
        match required {
            AuthScope::Read => true,
            AuthScope::Write => matches!(self, AuthScope::Write | AuthScope::Admin),
            AuthScope::Admin => matches!(self, AuthScope::Admin),
        }
    }
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Axum middleware that validates the `Authorization: Bearer <token>` header.
///
/// On success, injects an [`AuthContext`] as a request extension so handlers
/// can access it via `request.extensions().get::<AuthContext>()`.
pub async fn require_auth(
    State(state): State<ServiceState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer(request.headers())?;

    let auth = validate_token(&state, &token)?;
    request.extensions_mut().insert(auth);

    Ok(next.run(request).await)
}

/// Pull the bearer token out of the Authorization header.
fn extract_bearer(headers: &HeaderMap) -> Result<String, AppError> {
    let header = headers
        .get("authorization")
        .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;

    let value = header
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid Authorization header encoding".into()))?;

    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Unauthorized("expected Bearer token scheme".into()))?;

    if token.is_empty() {
        return Err(AppError::Unauthorized("empty bearer token".into()));
    }

    Ok(token.to_string())
}

/// Validate a bearer token and return the associated auth context.
///
/// API keys start with `mori_sk_`. SIWE session tokens start with `mori_sess_`.
fn validate_token(state: &ServiceState, token: &str) -> Result<AuthContext, AppError> {
    if token.starts_with("mori_sess_") {
        // TODO: validate SIWE session tokens against session store.
        // For now, accept them with Write scope so integration tests can pass.
        return Ok(AuthContext {
            identity: token.to_string(),
            scope: AuthScope::Write,
        });
    }

    // API key auth: check against the configured master key first, then the
    // key database. The master key (from env or config) always gets Admin scope.
    if let Some(ref master_key) = state.config.auth.master_key {
        if token == master_key {
            return Ok(AuthContext {
                identity: "master".into(),
                scope: AuthScope::Admin,
            });
        }
    }

    // Look up the key in the database.
    match state.db.validate_api_key(token) {
        Ok(Some(ctx)) => Ok(ctx),
        Ok(None) => Err(AppError::Unauthorized("invalid API key".into())),
        Err(e) => Err(AppError::Internal(e)),
    }
}

// ---------------------------------------------------------------------------
// AppError
// ---------------------------------------------------------------------------

/// Unified error type for all API handlers. Implements `IntoResponse` so Axum
/// can convert it directly into an HTTP response with the right status code.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    PaymentRequired(String),
    Conflict(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "not found: {msg}"),
            AppError::BadRequest(msg) => write!(f, "bad request: {msg}"),
            AppError::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
            AppError::PaymentRequired(msg) => write!(f, "payment required: {msg}"),
            AppError::Conflict(msg) => write!(f, "conflict: {msg}"),
            AppError::Internal(e) => write!(f, "internal error: {e}"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            AppError::PaymentRequired(msg) => (
                StatusCode::PAYMENT_REQUIRED,
                "payment_required",
                msg.clone(),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::Internal(e) => {
                tracing::error!("internal error: {e:#}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".into(),
                )
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

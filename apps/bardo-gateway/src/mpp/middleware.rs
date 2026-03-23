//! Axum middleware implementing the MPP payment protocol flow.
//!
//! For requests tagged as `AuthMode::Mpp`:
//! 1. No `X-Payment` header → estimate cost, return 402
//! 2. `X-Payment` header present → validate credential, attach `PaymentContext`
//! 3. After handler → settle, inject `Payment-Receipt` header

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::state::AppState;

use super::estimator::{
    estimate_input_tokens, estimate_provider_cost_usd, estimate_request_cost, usd_from_usdc,
    usdc_from_usd,
};
use super::types::*;
use super::verifier::verify_authorization_offchain;

/// Generate a random 32-byte nonce.
fn generate_nonce() -> alloy::primitives::FixedBytes<32> {
    use alloy::primitives::FixedBytes;
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    FixedBytes::from(bytes)
}

/// Current time as Unix seconds.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// MPP middleware: intercepts requests, handles 402 flow, validates credentials.
pub async fn mpp_payment(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Skip if not in MPP auth mode.
    let auth_mode = request
        .extensions()
        .get::<AuthMode>()
        .copied()
        .unwrap_or(AuthMode::ApiKey);

    if auth_mode != AuthMode::Mpp {
        return Ok(next.run(request).await);
    }

    let mpp_arc = state
        .mpp
        .clone()
        .ok_or_else(|| AppError::Internal("MPP not configured".into()))?;

    let spread_pct = mpp_arc.config.default_spread;
    let recipient = mpp_arc.config.recipient_address;
    let quote_validity = mpp_arc.config.quote_validity;

    // Check for X-Payment header.
    let payment_header = request
        .headers()
        .get("x-payment")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match payment_header {
        None => {
            // No credential: estimate cost and return 402.
            let (_parts, body) = request.into_parts();
            let body_bytes = axum::body::to_bytes(body, state.max_body_size)
                .await
                .map_err(|_| AppError::BadRequest("body too large".into()))?;

            let raw: Value = serde_json::from_slice(&body_bytes)
                .map_err(|e| AppError::BadRequest(format!("invalid JSON: {e}")))?;

            let model = raw
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("claude-sonnet-4");

            let input_tokens = estimate_input_tokens(&raw);
            let amount = estimate_request_cost(model, input_tokens, &state.pricing, spread_pct);

            let provider_cost_usd = estimate_provider_cost_usd(model, input_tokens, &state.pricing);
            let provider_cost_usdc = usdc_from_usd(provider_cost_usd);
            let spread_usdc = amount.saturating_sub(provider_cost_usdc);

            let payment_required = PaymentRequired {
                amount,
                asset: super::verifier::USDC_BASE,
                chain_id: super::verifier::BASE_CHAIN_ID,
                recipient,
                expiry: now_secs() + quote_validity,
                intent: PaymentIntent::Charge,
                nonce: generate_nonce(),
                session_id: None,
                breakdown: Some(CostBreakdown {
                    provider_cost: provider_cost_usdc,
                    spread: spread_usdc,
                    spread_pct,
                }),
            };

            let body_json = serde_json::to_vec(&payment_required)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let header_value = serde_json::to_string(&payment_required)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            Ok(Response::builder()
                .status(StatusCode::PAYMENT_REQUIRED)
                .header("x-payment-required", header_value)
                .header("content-type", "application/json")
                .body(Body::from(body_json))
                .unwrap())
        }
        Some(credential_str) => {
            // Parse and validate credential.
            let credential: PaymentCredential = serde_json::from_str(&credential_str)
                .map_err(|e| AppError::BadRequest(format!("invalid X-Payment: {e}")))?;

            match credential.intent {
                PaymentIntent::Charge => handle_charge(&mpp_arc, request, credential, next).await,
                PaymentIntent::Session => {
                    handle_session_draw(&mpp_arc, request, credential, next).await
                }
            }
        }
    }
}

/// Handle a Charge intent: verify authorization, run handler, settle, emit receipt.
async fn handle_charge(
    mpp_state: &super::MppState,
    mut request: Request<Body>,
    credential: PaymentCredential,
    next: Next,
) -> Result<Response, AppError> {
    let recipient = mpp_state.config.recipient_address;
    let spread_pct = mpp_state.config.default_spread;

    // Verify ERC-3009 signature off-chain.
    verify_authorization_offchain(
        &credential.authorization,
        recipient,
        alloy::primitives::U256::ZERO,
    )
    .map_err(|e| AppError::Unauthorized(format!("payment verification failed: {e}")))?;

    // Attach payment context to the request.
    let ctx = PaymentContext {
        intent: PaymentIntent::Charge,
        payer: credential.authorization.from,
        authorized_amount: credential.authorization.value,
        session_id: None,
        credential,
    };
    request.extensions_mut().insert(ctx);

    // Run the actual handler.
    let mut response = next.run(request).await;

    // Post-handler: compute actual charge and inject receipt.
    inject_charge_receipt(&mut response, spread_pct);

    Ok(response)
}

/// Handle a Session draw: check balance, run handler, draw actual cost, emit receipt.
async fn handle_session_draw(
    mpp_state: &super::MppState,
    mut request: Request<Body>,
    credential: PaymentCredential,
    next: Next,
) -> Result<Response, AppError> {
    let session_id = credential
        .session_id
        .clone()
        .ok_or_else(|| AppError::BadRequest("session_id required for Session intent".into()))?;

    // Check session exists and has balance.
    let session = mpp_state
        .sessions
        .get(&session_id)
        .ok_or_else(|| AppError::BadRequest(format!("session not found: {session_id}")))?;

    if session.remaining().is_zero() {
        return Err(AppError::PaymentRequired(
            "session balance exhausted".into(),
        ));
    }

    let ctx = PaymentContext {
        intent: PaymentIntent::Session,
        payer: session.payer,
        authorized_amount: session.remaining(),
        session_id: Some(session_id.clone()),
        credential,
    };
    request.extensions_mut().insert(ctx);

    // Run the handler.
    let mut response = next.run(request).await;

    // Post-handler: draw actual cost from session.
    let spread_pct = mpp_state.config.default_spread;
    if let Some(cost_usd) = extract_cost_usd(&response) {
        let charge_usd = cost_usd * (1.0 + spread_pct);
        let charge_usdc = usdc_from_usd(charge_usd);

        let remaining = mpp_state
            .sessions
            .draw(
                &session_id,
                charge_usdc,
                &uuid::Uuid::new_v4().to_string(),
                "",
            )
            .unwrap_or(alloy::primitives::U256::ZERO);

        let receipt = PaymentReceipt {
            receipt_id: uuid::Uuid::new_v4().to_string(),
            amount_charged: charge_usdc,
            amount_remaining: Some(remaining),
            tx_hash: None,
            session_id: Some(session_id),
            timestamp: now_secs(),
        };

        inject_receipt_headers(&mut response, &receipt, cost_usd, spread_pct);
    }

    Ok(response)
}

/// Extract the `x-mori-cost-usd` header value from a response.
fn extract_cost_usd(response: &Response) -> Option<f64> {
    response
        .headers()
        .get("x-mori-cost-usd")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
}

/// Inject Payment-Receipt and X-Bardo-* headers for a Charge intent.
fn inject_charge_receipt(response: &mut Response, spread_pct: f64) {
    if let Some(cost_usd) = extract_cost_usd(response) {
        let charge_usd = cost_usd * (1.0 + spread_pct);
        let charge_usdc = usdc_from_usd(charge_usd);

        let receipt = PaymentReceipt {
            receipt_id: uuid::Uuid::new_v4().to_string(),
            amount_charged: charge_usdc,
            amount_remaining: None,
            tx_hash: None,
            session_id: None,
            timestamp: now_secs(),
        };

        inject_receipt_headers(response, &receipt, cost_usd, spread_pct);
    }
}

/// Inject `Payment-Receipt` and `X-Bardo-*` headers into a response.
fn inject_receipt_headers(
    response: &mut Response,
    receipt: &PaymentReceipt,
    provider_cost_usd: f64,
    spread_pct: f64,
) {
    let headers = response.headers_mut();

    if let Ok(receipt_str) = serde_json::to_string(receipt) {
        if let Ok(hv) = receipt_str.parse() {
            headers.insert("payment-receipt", hv);
        }
    }

    let charge_usd = provider_cost_usd * (1.0 + spread_pct);
    let spread_usd = charge_usd - provider_cost_usd;

    // Get naive cost from existing header if available.
    let naive_usd = headers
        .get("x-mori-naive-cost-usd")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(provider_cost_usd);

    let savings_usd = naive_usd - charge_usd;

    if let Ok(v) = format!("{charge_usd:.6}").parse() {
        headers.insert("x-bardo-cost", v);
    }
    if let Ok(v) = format!("{provider_cost_usd:.6}").parse() {
        headers.insert("x-bardo-provider-cost", v);
    }
    if let Ok(v) = format!("{spread_usd:.6}").parse() {
        headers.insert("x-bardo-spread", v);
    }
    if let Ok(v) = format!("{spread_pct:.2}").parse() {
        headers.insert("x-bardo-spread-pct", v);
    }
    if let Ok(v) = format!("{naive_usd:.6}").parse() {
        headers.insert("x-bardo-naive-cost", v);
    }
    if savings_usd > 0.0 {
        if let Ok(v) = format!("{savings_usd:.6}").parse() {
            headers.insert("x-bardo-savings", v);
        }
    }

    if let Some(remaining) = receipt.amount_remaining {
        let balance_usd = usd_from_usdc(remaining);
        if let Ok(v) = format!("{balance_usd:.6}").parse() {
            headers.insert("x-bardo-balance", v);
        }
    }
}

/// Session open handler (called from route, not middleware).
pub async fn session_open(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<SessionOpenRequest>,
) -> Result<axum::Json<SessionOpenResponse>, AppError> {
    let mpp_state = state
        .mpp
        .as_ref()
        .ok_or_else(|| AppError::Internal("MPP not configured".into()))?;

    let recipient = mpp_state.config.recipient_address;

    // Verify the funding authorization.
    verify_authorization_offchain(
        &payload.authorization,
        recipient,
        alloy::primitives::U256::ZERO,
    )
    .map_err(|e| AppError::Unauthorized(format!("session open failed: {e}")))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let session = mpp_state.sessions.open(
        session_id.clone(),
        payload.authorization.from,
        payload.authorization.value,
        mpp_state.config.session_ttl,
    );

    Ok(axum::Json(SessionOpenResponse {
        session_id,
        funded_amount: session.funded_amount,
        expires_at: session.expires_at.timestamp() as u64,
    }))
}

/// Session status handler.
pub async fn session_status(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<axum::Json<SessionStatusResponse>, AppError> {
    let mpp_state = state
        .mpp
        .as_ref()
        .ok_or_else(|| AppError::Internal("MPP not configured".into()))?;

    let session = mpp_state
        .sessions
        .get(&session_id)
        .ok_or_else(|| AppError::BadRequest(format!("session not found: {session_id}")))?;

    let remaining = session.remaining();
    let draw_count = session.draws.len() as u64;
    let created_at = session.created_at.timestamp() as u64;
    let expires_at = session.expires_at.timestamp() as u64;

    Ok(axum::Json(SessionStatusResponse {
        session_id: session.session_id,
        payer: session.payer,
        funded_amount: session.funded_amount,
        drawn_amount: session.drawn_amount,
        remaining,
        status: session.status,
        created_at,
        expires_at,
        draw_count,
    }))
}

/// Session close handler.
pub async fn session_close(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<axum::Json<super::session::SessionSettlement>, AppError> {
    let mpp_state = state
        .mpp
        .as_ref()
        .ok_or_else(|| AppError::Internal("MPP not configured".into()))?;

    let settlement = mpp_state
        .sessions
        .close(&session_id)
        .ok_or_else(|| AppError::BadRequest(format!("session not found: {session_id}")))?;

    Ok(axum::Json(settlement))
}

// ── Request/Response types for session routes ──

#[derive(Debug, Deserialize)]
pub struct SessionOpenRequest {
    pub authorization: Erc3009Authorization,
}

#[derive(Debug, Serialize)]
pub struct SessionOpenResponse {
    pub session_id: String,
    pub funded_amount: alloy::primitives::U256,
    pub expires_at: u64,
}

#[derive(Debug, Serialize)]
pub struct SessionStatusResponse {
    pub session_id: String,
    pub payer: alloy::primitives::Address,
    pub funded_amount: alloy::primitives::U256,
    pub drawn_amount: alloy::primitives::U256,
    pub remaining: alloy::primitives::U256,
    pub status: super::session::MppSessionStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub draw_count: u64,
}

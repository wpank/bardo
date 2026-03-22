//! Route handlers for the gateway.

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;
use std::time::Instant;

use crate::cache::ResponseCache;
use crate::error::AppError;
use crate::format;
use crate::prefix;
use crate::provider;
use crate::session::SessionCost;
use crate::sse;
use crate::state::{AppState, CachedResponse};
use crate::tier::TierInfo;
use crate::tools::ToolTracker;

/// Minimum requests before tool pruning kicks in.
const TOOL_PRUNE_MIN_REQUESTS: usize = 5;

/// Health check endpoint (no auth required).
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "bardo-gateway",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Stats endpoint: token usage, costs, cache hits, savings.
pub async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    let mut resp = state.stats.to_json();
    resp.cache_entries = state.cache.len() as usize;
    Json(resp)
}

/// Per-session cost breakdown endpoint.
pub async fn costs(State(state): State<AppState>) -> impl IntoResponse {
    let sessions: Vec<SessionCost> = state
        .sessions
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    Json(serde_json::json!({
        "sessions": sessions,
        "total_sessions": sessions.len(),
    }))
}

/// Extract session ID from headers (optional).
fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-mori-session-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

/// Record session cost data after a request completes.
fn record_session_cost(
    state: &AppState,
    session_id: &str,
    model: &str,
    provider_name: &str,
    input_tokens: u64,
    output_tokens: u64,
    cost_usd: f64,
    savings_usd: f64,
) {
    state
        .sessions
        .entry(session_id.to_string())
        .or_insert_with(|| SessionCost::new(session_id.to_string()))
        .record(
            model,
            provider_name,
            input_tokens,
            output_tokens,
            cost_usd,
            savings_usd,
        );
}

/// Record tool usage from a response into the tracker.
fn record_tool_usage(state: &AppState, session_id: &str, response: &Value) {
    let tool_names = ToolTracker::extract_used_tools(response);
    for name in &tool_names {
        state.tool_tracker.record_usage(session_id, name);
    }
}

/// Apply cost-saving preprocessing to a raw JSON request.
///
/// Returns the (possibly modified) raw JSON and the session ID if present.
fn preprocess_request(state: &AppState, raw: &mut Value, headers: &HeaderMap) -> Option<String> {
    let session_id = session_id_from_headers(headers);

    // Tier-based model override
    if let Some(tier_info) = TierInfo::from_headers(headers) {
        if let Some(ref model) = tier_info.routed_model {
            raw["model"] = Value::String(model.clone());
            tracing::info!(
                tier = ?tier_info.tier,
                vitality = tier_info.vitality,
                model = %model,
                "tier routing overrode model"
            );
        }
    }

    // Tool pruning (only if session is known)
    if let Some(ref sid) = session_id {
        state.tool_tracker.record_request(sid);

        if let Some(tools) = raw.get("tools").and_then(|t| t.as_array()) {
            if let Some(pruned) =
                state
                    .tool_tracker
                    .prune_tools(sid, tools, TOOL_PRUNE_MIN_REQUESTS)
            {
                let original_count = tools.len();
                let pruned_count = pruned.len();
                raw["tools"] = Value::Array(pruned);
                tracing::info!(
                    session = %sid,
                    original = original_count,
                    pruned = pruned_count,
                    "pruned unused tool definitions"
                );
            }
        }
    }

    session_id
}

/// Unified messages endpoint: auto-detects Anthropic vs OpenAI format,
/// forwards to the appropriate provider, returns in the caller's format.
pub async fn messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let start = Instant::now();

    // Parse raw JSON
    let mut raw: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid JSON: {e}")))?;

    // Check for T0 suppression before any other work
    if let Some(tier_info) = TierInfo::from_headers(&headers) {
        if tier_info.routed_model.is_none() {
            // T0: return empty success immediately
            tracing::info!("T0 tier suppressed inference");
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .header("x-mori-tier", "T0")
                .header("x-mori-suppressed", "true")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "id": "t0-suppressed",
                        "type": "message",
                        "role": "assistant",
                        "content": [],
                        "model": "none",
                        "stop_reason": "end_turn",
                        "usage": {"input_tokens": 0, "output_tokens": 0}
                    }))
                    .unwrap_or_default(),
                ))
                .unwrap_or_default());
        }
    }

    // Apply preprocessing (tier routing model override, tool pruning)
    let session_id = preprocess_request(&state, &mut raw, &headers);

    // Normalize JSON key ordering for better cache hits
    let normalized = prefix::normalize_json_ordering(&raw);
    let normalized_bytes = serde_json::to_vec(&normalized).unwrap_or_default();

    let is_streaming = normalized
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Check hash cache (only for non-streaming requests)
    if !is_streaming {
        let hash = ResponseCache::request_hash(&normalized_bytes);
        if let Some(cached) = state.cache.get(&hash).await {
            let elapsed = start.elapsed();
            state.stats.record_request(&cached.model, 0, 0, 0.0, true);
            state.stats.record_cache_hit_savings(cached.cost_usd);

            if let Some(ref sid) = session_id {
                record_session_cost(
                    &state,
                    sid,
                    &cached.model,
                    "cache",
                    0,
                    0,
                    0.0,
                    cached.cost_usd,
                );
            }

            tracing::info!(
                cache = "hash-hit",
                saved_usd = cached.cost_usd,
                elapsed_ms = elapsed.as_millis(),
                "returning cached response"
            );
            return Ok(build_response_with_cost_headers(
                cached.body,
                &cached.content_type,
                cached.cost_usd,
                0.0, // cached = no actual cost
                true,
                &cached.model,
                elapsed,
            ));
        }
    }

    // /v1/messages is always Anthropic format — Claude Code and the Anthropic SDK
    // both hit this endpoint. Format detection only matters for /v1/chat/completions.
    //
    // Exception: if the model is a non-claude model and OpenAI is configured,
    // translate the request to OpenAI format and translate the response back.
    let model = normalized
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    if should_route_to_openai(model, &state) {
        handle_openai_for_anthropic_client(
            &state,
            &normalized,
            &normalized_bytes,
            is_streaming,
            start,
            session_id.as_deref(),
        )
        .await
    } else {
        handle_anthropic(
            &state,
            &normalized,
            &normalized_bytes,
            is_streaming,
            start,
            session_id.as_deref(),
        )
        .await
    }
}

/// Returns true when a non-claude model should be routed to OpenAI.
fn should_route_to_openai(model: &str, state: &AppState) -> bool {
    state.openai_api_key.is_some() && !model.contains("claude")
}

/// Handle a request from an Anthropic-format client (e.g. Claude Code) that
/// names a non-claude model. Translates the request to OpenAI format, forwards
/// it, then translates the response back to Anthropic format so the client
/// sees what it expects.
async fn handle_openai_for_anthropic_client(
    state: &AppState,
    raw: &Value,
    body: &[u8],
    is_streaming: bool,
    start: Instant,
    session_id: Option<&str>,
) -> Result<Response, AppError> {
    let openai_key = state
        .openai_api_key
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("OpenAI API key not configured".into()))?;

    let model = raw
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    let openai_req = format::anthropic_body_to_openai_request(raw);

    if is_streaming {
        let byte_stream = provider::openai_stream(&state.http, openai_key, &openai_req).await?;
        let translated = sse::openai_stream_to_anthropic(byte_stream, model);
        let response_body = Body::from_stream(translated);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("x-mori-format", "anthropic-via-openai")
            .header("x-mori-stream", "true")
            .body(response_body)
            .unwrap_or_default());
    }

    let (parsed, _raw_bytes) =
        provider::openai_complete(&state.http, openai_key, &openai_req).await?;

    let resp_model = parsed
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    let usage = parsed.get("usage");
    let input_tokens = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    let output_tokens = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    let (ip, op) = state.price_for_model(resp_model);
    let cost = (input_tokens as f64 * ip / 1_000_000.0) + (output_tokens as f64 * op / 1_000_000.0);

    // Translate back to Anthropic format for the client
    let anthropic_json = format::openai_body_to_anthropic_response(&parsed);
    let anthropic_bytes = Bytes::from(serde_json::to_vec(&anthropic_json).unwrap_or_default());

    let elapsed = start.elapsed();

    // Cache using the original Anthropic-format request hash
    let hash = ResponseCache::request_hash(body);
    state
        .cache
        .put(
            hash,
            CachedResponse {
                body: anthropic_bytes.clone(),
                content_type: "application/json".into(),
                cost_usd: cost,
                model: resp_model.to_string(),
                cached_at: chrono::Utc::now(),
            },
        )
        .await;

    state
        .stats
        .record_request(resp_model, input_tokens, output_tokens, cost, false);

    if let Some(sid) = session_id {
        record_session_cost(
            state,
            sid,
            resp_model,
            "openai",
            input_tokens,
            output_tokens,
            cost,
            0.0,
        );
    }

    tracing::info!(
        model = %resp_model,
        input_tokens,
        output_tokens,
        cost_usd = cost,
        elapsed_ms = elapsed.as_millis(),
        "openai request for anthropic client complete"
    );

    Ok(build_response_with_cost_headers(
        anthropic_bytes,
        "application/json",
        cost,
        cost,
        false,
        resp_model,
        elapsed,
    ))
}

/// Handle Anthropic-format requests.
async fn handle_anthropic(
    state: &AppState,
    raw: &Value,
    body: &[u8],
    is_streaming: bool,
    start: Instant,
    session_id: Option<&str>,
) -> Result<Response, AppError> {
    if is_streaming {
        let byte_stream =
            provider::anthropic_stream_raw(&state.http, &state.anthropic_api_key, raw).await?;

        let stats = state.stats.clone();
        let pricing = state.pricing.clone();
        let sessions = state.sessions.clone();
        let tool_tracker = state.tool_tracker.clone();
        let sid = session_id.map(String::from);
        let price_fn = {
            let pricing = pricing.clone();
            move |model: &str| -> (f64, f64) {
                pricing
                    .iter()
                    .find(|p| model.contains(&p.model) || p.model.contains(model))
                    .map(|p| (p.input_per_m, p.output_per_m))
                    .unwrap_or((3.0, 15.0))
            }
        };
        let model_name = raw
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        let tapped_stream = byte_stream.map(move |result| {
            if let Ok(ref bytes) = result {
                let text = String::from_utf8_lossy(bytes);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<Value>(data) {
                            if let Some(usage) = event.get("usage") {
                                let input = usage
                                    .get("input_tokens")
                                    .and_then(|t| t.as_u64())
                                    .unwrap_or(0);
                                let output = usage
                                    .get("output_tokens")
                                    .and_then(|t| t.as_u64())
                                    .unwrap_or(0);
                                if input > 0 || output > 0 {
                                    let (ip, op) = price_fn(&model_name);
                                    let cost = (input as f64 * ip / 1_000_000.0)
                                        + (output as f64 * op / 1_000_000.0);
                                    stats.record_request(&model_name, input, output, cost, false);

                                    if let Some(ref sid) = sid {
                                        sessions
                                            .entry(sid.clone())
                                            .or_insert_with(|| SessionCost::new(sid.clone()))
                                            .record(
                                                &model_name,
                                                "anthropic",
                                                input,
                                                output,
                                                cost,
                                                0.0,
                                            );
                                    }

                                    tracing::info!(
                                        model = %model_name,
                                        input_tokens = input,
                                        output_tokens = output,
                                        cost_usd = cost,
                                        stream = true,
                                        "anthropic stream complete"
                                    );
                                }
                            }

                            // Track tool usage from streaming events
                            if let Some(ref sid) = sid {
                                let tools = ToolTracker::extract_used_tools(&event);
                                for name in &tools {
                                    tool_tracker.record_usage(sid, name);
                                }
                            }
                        }
                    }
                }
            }
            result
        });

        let body = Body::from_stream(tapped_stream);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("x-mori-format", "anthropic")
            .header("x-mori-stream", "true")
            .body(body)
            .unwrap_or_default())
    } else {
        let (parsed_value, response_bytes) =
            provider::anthropic_complete_raw(&state.http, &state.anthropic_api_key, raw).await?;

        let model = parsed_value
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        let usage = parsed_value.get("usage");
        let input_tokens = usage
            .and_then(|u| u.get("input_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .and_then(|u| u.get("output_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        let cache_read = usage
            .and_then(|u| u.get("cache_read_input_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        let (input_price, output_price) = state.price_for_model(model);
        let regular_input = input_tokens.saturating_sub(cache_read);
        let cost = (regular_input as f64 * input_price / 1_000_000.0)
            + (cache_read as f64 * input_price * 0.1 / 1_000_000.0)
            + (output_tokens as f64 * output_price / 1_000_000.0);

        let elapsed = start.elapsed();

        // Cache the response
        let hash = ResponseCache::request_hash(body);
        state
            .cache
            .put(
                hash,
                CachedResponse {
                    body: response_bytes.clone(),
                    content_type: "application/json".into(),
                    cost_usd: cost,
                    model: model.to_string(),
                    cached_at: chrono::Utc::now(),
                },
            )
            .await;

        state
            .stats
            .record_request(model, input_tokens, output_tokens, cost, false);

        // Session tracking and tool usage recording
        if let Some(sid) = session_id {
            record_session_cost(
                state,
                sid,
                model,
                "anthropic",
                input_tokens,
                output_tokens,
                cost,
                0.0,
            );
            record_tool_usage(state, sid, &parsed_value);
        }

        tracing::info!(
            model = %model,
            input_tokens,
            output_tokens,
            cost_usd = cost,
            elapsed_ms = elapsed.as_millis(),
            "anthropic request complete"
        );

        Ok(build_response_with_cost_headers(
            response_bytes,
            "application/json",
            cost,
            cost,
            false,
            model,
            elapsed,
        ))
    }
}

/// Handle OpenAI-format requests.
async fn handle_openai(
    state: &AppState,
    raw: &Value,
    body: &[u8],
    is_streaming: bool,
    start: Instant,
    session_id: Option<&str>,
) -> Result<Response, AppError> {
    let openai_key = state
        .openai_api_key
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("OpenAI API key not configured".into()))?;

    if is_streaming {
        let byte_stream = provider::openai_stream(&state.http, openai_key, raw).await?;

        let body = Body::from_stream(byte_stream);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("x-mori-format", "openai")
            .header("x-mori-stream", "true")
            .body(body)
            .unwrap_or_default())
    } else {
        let (parsed, response_bytes) =
            provider::openai_complete(&state.http, openai_key, raw).await?;

        let model = parsed
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        let usage = parsed.get("usage");
        let input_tokens = usage
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        let (input_price, output_price) = state.price_for_model(model);
        let cost = (input_tokens as f64 * input_price / 1_000_000.0)
            + (output_tokens as f64 * output_price / 1_000_000.0);

        let elapsed = start.elapsed();

        // Cache
        let hash = ResponseCache::request_hash(body);
        state
            .cache
            .put(
                hash,
                CachedResponse {
                    body: response_bytes.clone(),
                    content_type: "application/json".into(),
                    cost_usd: cost,
                    model: model.to_string(),
                    cached_at: chrono::Utc::now(),
                },
            )
            .await;

        state
            .stats
            .record_request(model, input_tokens, output_tokens, cost, false);

        if let Some(sid) = session_id {
            record_session_cost(
                state,
                sid,
                model,
                "openai",
                input_tokens,
                output_tokens,
                cost,
                0.0,
            );
        }

        tracing::info!(
            model = %model,
            input_tokens,
            output_tokens,
            cost_usd = cost,
            elapsed_ms = elapsed.as_millis(),
            "openai request complete"
        );

        Ok(build_response_with_cost_headers(
            response_bytes,
            "application/json",
            cost,
            cost,
            false,
            model,
            elapsed,
        ))
    }
}

/// Build a response with cost tracking headers.
fn build_response_with_cost_headers(
    body: Bytes,
    content_type: &str,
    naive_cost: f64,
    actual_cost: f64,
    is_cache_hit: bool,
    model: &str,
    elapsed: std::time::Duration,
) -> Response {
    let savings = naive_cost - actual_cost;
    let cache_status = if is_cache_hit { "hash-hit" } else { "miss" };

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .header("x-mori-cost-usd", format!("{actual_cost:.6}"))
        .header("x-mori-naive-cost-usd", format!("{naive_cost:.6}"))
        .header("x-mori-savings-usd", format!("{savings:.6}"))
        .header("x-mori-cache-status", cache_status)
        .header("x-mori-model", model)
        .header("x-mori-elapsed-ms", elapsed.as_millis().to_string())
        .body(Body::from(body))
        .unwrap_or_default()
}

/// OpenAI-compatible chat completions endpoint.
/// Translates between OpenAI format and Anthropic, or forwards to OpenAI directly.
pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let start = Instant::now();

    let mut raw: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid JSON: {e}")))?;

    // Check for T0 suppression
    if let Some(tier_info) = TierInfo::from_headers(&headers) {
        if tier_info.routed_model.is_none() {
            tracing::info!("T0 tier suppressed inference");
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .header("x-mori-tier", "T0")
                .header("x-mori-suppressed", "true")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "id": "t0-suppressed",
                        "type": "message",
                        "role": "assistant",
                        "content": [],
                        "model": "none",
                        "stop_reason": "end_turn",
                        "usage": {"input_tokens": 0, "output_tokens": 0}
                    }))
                    .unwrap_or_default(),
                ))
                .unwrap_or_default());
        }
    }

    // Apply preprocessing
    let session_id = preprocess_request(&state, &mut raw, &headers);

    // Normalize
    let normalized = prefix::normalize_json_ordering(&raw);

    let model = normalized
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let is_streaming = normalized
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // If the model is a Claude model, translate to Anthropic format and forward
    if model.contains("claude") {
        let openai_req: format::OpenAiRequest = serde_json::from_value(normalized.clone())
            .map_err(|e| AppError::BadRequest(format!("invalid OpenAI request: {e}")))?;

        let anthropic_req = format::openai_to_anthropic(&openai_req);
        let anthropic_body =
            serde_json::to_vec(&anthropic_req).map_err(|e| AppError::Internal(e.to_string()))?;

        if !is_streaming {
            let anthropic_value: Value = serde_json::from_slice(&anthropic_body)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            // Check cache
            let hash = ResponseCache::request_hash(&anthropic_body);
            if let Some(cached) = state.cache.get(&hash).await {
                let elapsed = start.elapsed();
                state.stats.record_request(&cached.model, 0, 0, 0.0, true);
                state.stats.record_cache_hit_savings(cached.cost_usd);

                if let Some(ref sid) = session_id {
                    record_session_cost(
                        &state,
                        sid,
                        &cached.model,
                        "cache",
                        0,
                        0,
                        0.0,
                        cached.cost_usd,
                    );
                }

                return Ok(build_response_with_cost_headers(
                    cached.body,
                    "application/json",
                    cached.cost_usd,
                    0.0,
                    true,
                    &cached.model,
                    elapsed,
                ));
            }

            let (parsed_value, _response_bytes) = provider::anthropic_complete_raw(
                &state.http,
                &state.anthropic_api_key,
                &anthropic_value,
            )
            .await?;

            let resp_model = parsed_value
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown");
            let resp_usage = parsed_value.get("usage");
            let inp = resp_usage
                .and_then(|u| u.get("input_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            let outp = resp_usage
                .and_then(|u| u.get("output_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            let (ip, op) = state.price_for_model(resp_model);
            let cost = (inp as f64 * ip / 1_000_000.0) + (outp as f64 * op / 1_000_000.0);

            state
                .stats
                .record_request(resp_model, inp, outp, cost, false);

            if let Some(ref sid) = session_id {
                record_session_cost(&state, sid, resp_model, "anthropic", inp, outp, cost, 0.0);
                record_tool_usage(&state, sid, &parsed_value);
            }

            // Translate Anthropic response → OpenAI format for this client
            let openai_json = format::anthropic_body_to_openai_response(&parsed_value);
            let openai_bytes = Bytes::from(serde_json::to_vec(&openai_json).unwrap_or_default());

            state
                .cache
                .put(
                    hash,
                    CachedResponse {
                        body: openai_bytes.clone(),
                        content_type: "application/json".into(),
                        cost_usd: cost,
                        model: resp_model.to_string(),
                        cached_at: chrono::Utc::now(),
                    },
                )
                .await;

            let elapsed = start.elapsed();
            Ok(build_response_with_cost_headers(
                openai_bytes,
                "application/json",
                cost,
                cost,
                false,
                resp_model,
                elapsed,
            ))
        } else {
            // Streaming: translate Anthropic SSE → OpenAI SSE for the client
            let anthropic_value: Value =
                serde_json::from_slice(&anthropic_body).unwrap_or_default();
            let byte_stream = provider::anthropic_stream_raw(
                &state.http,
                &state.anthropic_api_key,
                &anthropic_value,
            )
            .await?;
            let translated = sse::anthropic_stream_to_openai(byte_stream, model.to_string());
            let body = Body::from_stream(translated);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("x-mori-format", "openai")
                .header("x-mori-stream", "true")
                .body(body)
                .unwrap_or_default())
        }
    } else {
        // Non-Claude model: forward to OpenAI directly
        let normalized_bytes = serde_json::to_vec(&normalized).unwrap_or_default();
        handle_openai(
            &state,
            &normalized,
            &normalized_bytes,
            is_streaming,
            start,
            session_id.as_deref(),
        )
        .await
    }
}

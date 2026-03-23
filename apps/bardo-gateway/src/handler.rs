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
use std::sync::Arc;
use std::time::Instant;

use crate::cache::ResponseCache;
use crate::error::AppError;
use crate::format;
use crate::prefix;
use crate::providers::Provider;
use crate::session::SessionCost;
use crate::sse;
use crate::state::{AppState, CachedResponse};
use crate::tier::TierInfo;
use crate::tools::ToolTracker;

use crate::pricing::ModelPricing;
use crate::providers::UsageDetails;
use crate::state::StatsEvent;

/// Minimum requests before tool pruning kicks in.
const TOOL_PRUNE_MIN_REQUESTS: usize = 5;

/// Compute actual and naive cost from usage details and pricing.
///
/// Returns `(actual_cost, naive_cost)` in USD.
/// - `actual_cost`: what is actually charged (with cache discounts, reasoning rates, batch discount)
/// - `naive_cost`: what it would cost at full price (no caching, no batch)
fn compute_cost(usage: &UsageDetails, pricing: &ModelPricing, is_batch: bool) -> (f64, f64) {
    let ip = pricing.input_per_m;
    let op = pricing.output_per_m;
    let cached_ip = pricing.cached_input_per_m.unwrap_or(ip * 0.1);
    let reasoning_p = pricing.reasoning_per_m.unwrap_or(op);

    // Actual cost with all discounts applied.
    let mut cost = 0.0;
    // Anthropic: input_tokens is non-cached, cache fields are separate.
    // OpenAI: cached_tokens is a subset of input_tokens (use saturating_sub).
    let fresh_input = usage.input_tokens.saturating_sub(usage.cached_tokens);
    cost += fresh_input as f64 * ip / 1e6;
    cost += usage.cached_tokens as f64 * cached_ip / 1e6; // OpenAI cached (50% off)
    cost += usage.cache_creation_input_tokens as f64 * ip * 1.25 / 1e6; // Anthropic write (25% surcharge)
    cost += usage.cache_read_input_tokens as f64 * cached_ip / 1e6; // Anthropic read (90% off)
    let regular_output = usage.output_tokens.saturating_sub(usage.reasoning_tokens);
    cost += regular_output as f64 * op / 1e6;
    cost += usage.reasoning_tokens as f64 * reasoning_p / 1e6; // o-series reasoning
    cost += usage.thinking_tokens as f64 * op / 1e6; // Anthropic thinking

    if is_batch {
        cost *= 0.5;
    }

    // Naive cost: what Anthropic would charge WITHOUT the gateway's cache_control
    // injection. No cache reads, no cache writes — just regular input pricing.
    // Cache creation and cache read tokens would have been regular input tokens.
    let total_input =
        usage.input_tokens + usage.cache_read_input_tokens + usage.cache_creation_input_tokens;
    let total_output = usage.output_tokens + usage.thinking_tokens;
    let naive = (total_input as f64 * ip / 1e6) + (total_output as f64 * op / 1e6);

    // Naive is always >= actual. On a cache write request, the 25% surcharge
    // makes actual > naive — but that's an investment, not a loss. Clamp naive
    // to at least actual so savings is never negative per-request.
    let naive = naive.max(cost);

    (cost, naive)
}

/// Fire-and-forget broadcast of a per-request stats event to dashboard clients.
fn emit_stats_event(
    stats_tx: &tokio::sync::broadcast::Sender<StatsEvent>,
    stats: &crate::state::GatewayStats,
    model: &str,
    provider: &str,
    usage: &UsageDetails,
    cost_usd: f64,
    naive_cost_usd: f64,
    savings_usd: f64,
    cache_hit: bool,
    is_batch: bool,
    elapsed_ms: u64,
    streaming: bool,
    session_id: Option<&str>,
) {
    let seq = stats
        .event_seq
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let _ = stats_tx.send(StatsEvent {
        seq,
        timestamp_ms,
        model: model.to_string(),
        provider: provider.to_string(),
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_read_tokens: usage.cache_read_input_tokens,
        cache_create_tokens: usage.cache_creation_input_tokens,
        openai_cached_tokens: usage.cached_tokens,
        reasoning_tokens: usage.reasoning_tokens,
        thinking_tokens: usage.thinking_tokens,
        cost_usd,
        naive_cost_usd,
        savings_usd,
        cache_hit,
        prefix_cache_warm: usage.cache_read_input_tokens > 0 || usage.cached_tokens > 0,
        is_batch,
        elapsed_ms,
        streaming,
        session_id: session_id.map(String::from),
    });
}

// ── Stream accumulation and SSE synthesis for cache support ──────────────

/// Accumulates SSE events during streaming to reconstruct the full response body.
struct StreamAccumulator {
    id: String,
    model: String,
    /// Content blocks accumulated from content_block_start + content_block_delta.
    content_blocks: Vec<Value>,
    /// Index of the current block being accumulated.
    current_block_idx: Option<usize>,
    /// Stop reason from message_delta.
    stop_reason: Option<String>,
    /// Usage counters.
    input_tokens: u64,
    output_tokens: u64,
    cache_read_input_tokens: u64,
    cache_creation_input_tokens: u64,
    thinking_tokens: u64,
    /// Whether message_stop has been received.
    complete: bool,
    /// Cost computed from the stream stats.
    cost: f64,
    naive_cost: f64,
}

impl StreamAccumulator {
    fn new() -> Self {
        Self {
            id: String::new(),
            model: String::new(),
            content_blocks: Vec::new(),
            current_block_idx: None,
            stop_reason: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
            thinking_tokens: 0,
            complete: false,
            cost: 0.0,
            naive_cost: 0.0,
        }
    }

    /// Process an SSE event and accumulate data for response reconstruction.
    fn process_event(&mut self, event: &Value) {
        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match event_type {
            "message_start" => {
                if let Some(msg) = event.get("message") {
                    self.id = msg
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("msg_cached")
                        .to_string();
                    self.model = msg
                        .get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if let Some(u) = msg.get("usage") {
                        self.input_tokens =
                            u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                        self.cache_read_input_tokens = u
                            .get("cache_read_input_tokens")
                            .and_then(|t| t.as_u64())
                            .unwrap_or(0);
                        self.cache_creation_input_tokens = u
                            .get("cache_creation_input_tokens")
                            .and_then(|t| t.as_u64())
                            .unwrap_or(0);
                        self.thinking_tokens = u
                            .get("thinking_tokens")
                            .and_then(|t| t.as_u64())
                            .unwrap_or(0);
                    }
                }
            }
            "content_block_start" => {
                let idx = event.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                if let Some(block) = event.get("content_block") {
                    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("text");
                    let skeleton = match block_type {
                        "text" => serde_json::json!({"type": "text", "text": ""}),
                        "thinking" => {
                            serde_json::json!({"type": "thinking", "thinking": ""})
                        }
                        "tool_use" => {
                            serde_json::json!({
                                "type": "tool_use",
                                "id": block.get("id").cloned().unwrap_or(Value::String("".into())),
                                "name": block.get("name").cloned().unwrap_or(Value::String("".into())),
                                "input": {}
                            })
                        }
                        _ => serde_json::json!({"type": block_type}),
                    };
                    // Ensure vec is large enough
                    while self.content_blocks.len() <= idx {
                        self.content_blocks.push(Value::Null);
                    }
                    self.content_blocks[idx] = skeleton;
                    self.current_block_idx = Some(idx);
                }
            }
            "content_block_delta" => {
                let idx = event.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                if let Some(delta) = event.get("delta") {
                    let delta_type = delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if idx < self.content_blocks.len() {
                        match delta_type {
                            "text_delta" => {
                                if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                    if let Some(existing) = self.content_blocks[idx]
                                        .get("text")
                                        .and_then(|t| t.as_str())
                                    {
                                        let mut combined = existing.to_string();
                                        combined.push_str(text);
                                        self.content_blocks[idx]["text"] = Value::String(combined);
                                    }
                                }
                            }
                            "thinking_delta" => {
                                if let Some(text) = delta.get("thinking").and_then(|t| t.as_str()) {
                                    if let Some(existing) = self.content_blocks[idx]
                                        .get("thinking")
                                        .and_then(|t| t.as_str())
                                    {
                                        let mut combined = existing.to_string();
                                        combined.push_str(text);
                                        self.content_blocks[idx]["thinking"] =
                                            Value::String(combined);
                                    }
                                }
                            }
                            "input_json_delta" => {
                                if let Some(json_str) =
                                    delta.get("partial_json").and_then(|t| t.as_str())
                                {
                                    // Accumulate the JSON string; parse at block_stop.
                                    let key = "__partial_json";
                                    let existing = self.content_blocks[idx]
                                        .get(key)
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("");
                                    let mut combined = existing.to_string();
                                    combined.push_str(json_str);
                                    self.content_blocks[idx][key] = Value::String(combined);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            "content_block_stop" => {
                let idx = event.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                // Finalize tool_use input from accumulated partial JSON.
                if idx < self.content_blocks.len() {
                    if let Some(partial) = self.content_blocks[idx]
                        .get("__partial_json")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                    {
                        if let Ok(parsed) = serde_json::from_str::<Value>(&partial) {
                            self.content_blocks[idx]["input"] = parsed;
                        }
                        // Remove temp key
                        if let Some(obj) = self.content_blocks[idx].as_object_mut() {
                            obj.remove("__partial_json");
                        }
                    }
                }
                self.current_block_idx = None;
            }
            "message_delta" => {
                if let Some(delta) = event.get("delta") {
                    self.stop_reason = delta
                        .get("stop_reason")
                        .and_then(|s| s.as_str())
                        .map(String::from);
                }
                if let Some(usage) = event.get("usage") {
                    self.output_tokens = usage
                        .get("output_tokens")
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0);
                }
            }
            "message_stop" => {
                self.complete = true;
            }
            _ => {}
        }
    }

    /// Reconstruct the full Anthropic Messages API response JSON.
    fn to_response_bytes(&self) -> Bytes {
        let body = serde_json::json!({
            "id": self.id,
            "type": "message",
            "role": "assistant",
            "model": self.model,
            "content": self.content_blocks,
            "stop_reason": self.stop_reason.as_deref().unwrap_or("end_turn"),
            "stop_sequence": null,
            "usage": {
                "input_tokens": self.input_tokens,
                "output_tokens": self.output_tokens,
                "cache_read_input_tokens": self.cache_read_input_tokens,
                "cache_creation_input_tokens": self.cache_creation_input_tokens,
            }
        });
        Bytes::from(serde_json::to_vec(&body).unwrap_or_default())
    }
}

/// Synthesize Anthropic SSE events from a cached JSON response body.
fn synthesize_sse_from_cached(cached_body: &Bytes) -> Bytes {
    let body: Value = match serde_json::from_slice(cached_body) {
        Ok(v) => v,
        Err(_) => return cached_body.clone(),
    };

    let mut sse = String::new();

    // Build message shell (empty content, usage without output_tokens)
    let mut msg_shell = body.clone();
    msg_shell["content"] = Value::Array(vec![]);
    msg_shell["stop_reason"] = Value::Null;
    msg_shell["stop_sequence"] = Value::Null;
    // Input-only usage for message_start
    let input_usage = serde_json::json!({
        "input_tokens": body.get("usage").and_then(|u| u.get("input_tokens")).cloned().unwrap_or(Value::Number(0.into())),
        "output_tokens": 0,
        "cache_read_input_tokens": body.get("usage").and_then(|u| u.get("cache_read_input_tokens")).cloned().unwrap_or(Value::Number(0.into())),
        "cache_creation_input_tokens": body.get("usage").and_then(|u| u.get("cache_creation_input_tokens")).cloned().unwrap_or(Value::Number(0.into())),
    });
    msg_shell["usage"] = input_usage;

    let start_event = serde_json::json!({"type": "message_start", "message": msg_shell});
    sse.push_str(&format!("event: message_start\ndata: {start_event}\n\n"));

    // Emit content blocks
    if let Some(content) = body.get("content").and_then(|c| c.as_array()) {
        for (i, block) in content.iter().enumerate() {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("text");

            // content_block_start (skeleton)
            let skeleton = match block_type {
                "text" => serde_json::json!({"type": "text", "text": ""}),
                "thinking" => serde_json::json!({"type": "thinking", "thinking": ""}),
                "tool_use" => serde_json::json!({
                    "type": "tool_use",
                    "id": block.get("id").cloned().unwrap_or(Value::String("".into())),
                    "name": block.get("name").cloned().unwrap_or(Value::String("".into())),
                    "input": {}
                }),
                _ => block.clone(),
            };
            let cbs = serde_json::json!({"type": "content_block_start", "index": i, "content_block": skeleton});
            sse.push_str(&format!("event: content_block_start\ndata: {cbs}\n\n"));

            // content_block_delta (full content as single delta)
            match block_type {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        if !text.is_empty() {
                            let cbd = serde_json::json!({
                                "type": "content_block_delta",
                                "index": i,
                                "delta": {"type": "text_delta", "text": text}
                            });
                            sse.push_str(&format!("event: content_block_delta\ndata: {cbd}\n\n"));
                        }
                    }
                }
                "thinking" => {
                    if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                        if !text.is_empty() {
                            let cbd = serde_json::json!({
                                "type": "content_block_delta",
                                "index": i,
                                "delta": {"type": "thinking_delta", "thinking": text}
                            });
                            sse.push_str(&format!("event: content_block_delta\ndata: {cbd}\n\n"));
                        }
                    }
                }
                "tool_use" => {
                    if let Some(input) = block.get("input") {
                        let json_str = serde_json::to_string(input).unwrap_or_default();
                        if json_str != "{}" {
                            let cbd = serde_json::json!({
                                "type": "content_block_delta",
                                "index": i,
                                "delta": {"type": "input_json_delta", "partial_json": json_str}
                            });
                            sse.push_str(&format!("event: content_block_delta\ndata: {cbd}\n\n"));
                        }
                    }
                }
                _ => {}
            }

            // content_block_stop
            let stop = serde_json::json!({"type": "content_block_stop", "index": i});
            sse.push_str(&format!("event: content_block_stop\ndata: {stop}\n\n"));
        }
    }

    // message_delta
    let output_tokens = body
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .cloned()
        .unwrap_or(Value::Number(0.into()));
    let stop_reason = body
        .get("stop_reason")
        .cloned()
        .unwrap_or(Value::String("end_turn".into()));
    let md = serde_json::json!({
        "type": "message_delta",
        "delta": {"stop_reason": stop_reason, "stop_sequence": null},
        "usage": {"output_tokens": output_tokens}
    });
    sse.push_str(&format!("event: message_delta\ndata: {md}\n\n"));

    // message_stop
    sse.push_str("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n");

    Bytes::from(sse)
}

/// Synthesize OpenAI SSE events from a cached OpenAI JSON response body.
fn synthesize_openai_sse_from_cached(cached_body: &Bytes, model: &str) -> Bytes {
    let body: Value = match serde_json::from_slice(cached_body) {
        Ok(v) => v,
        Err(_) => return cached_body.clone(),
    };

    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("chatcmpl-cached");

    let mut sse = String::new();

    // Extract text content from the first choice
    let content = body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    // Role chunk
    let role_chunk = serde_json::json!({
        "id": id,
        "object": "chat.completion.chunk",
        "model": model,
        "choices": [{"index": 0, "delta": {"role": "assistant", "content": ""}, "finish_reason": null}]
    });
    sse.push_str(&format!("data: {role_chunk}\n\n"));

    // Content chunk (full text as single delta)
    if !content.is_empty() {
        let content_chunk = serde_json::json!({
            "id": id,
            "object": "chat.completion.chunk",
            "model": model,
            "choices": [{"index": 0, "delta": {"content": content}, "finish_reason": null}]
        });
        sse.push_str(&format!("data: {content_chunk}\n\n"));
    }

    // Finish chunk with usage
    let usage = body.get("usage").cloned().unwrap_or(serde_json::json!({}));
    let finish_chunk = serde_json::json!({
        "id": id,
        "object": "chat.completion.chunk",
        "model": model,
        "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
        "usage": usage
    });
    sse.push_str(&format!("data: {finish_chunk}\n\n"));
    sse.push_str("data: [DONE]\n\n");

    Bytes::from(sse)
}
/// Returns a guard that releases the permit on drop.
async fn acquire_concurrency(
    state: &AppState,
) -> Result<Option<tokio::sync::OwnedSemaphorePermit>, AppError> {
    match &state.concurrency {
        Some(sem) => match sem.clone().try_acquire_owned() {
            Ok(permit) => Ok(Some(permit)),
            Err(_) => Err(AppError::BadRequest(
                "server overloaded, too many concurrent requests".into(),
            )),
        },
        None => Ok(None),
    }
}

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

/// Model catalog endpoint (no auth required).
///
/// Returns all models from all configured providers in OpenAI catalog format.
pub async fn models(State(state): State<AppState>) -> impl IntoResponse {
    let data: Vec<Value> = state.providers.iter().flat_map(|p| p.catalog()).collect();
    Json(serde_json::json!({"object": "list", "data": data}))
}

/// Extract session ID from headers (optional).
fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-mori-session-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

/// Derive a session ID from the request body when no explicit header is set.
///
/// Hashes the system prompt to produce a stable identifier — requests with the
/// same system prompt likely come from the same agent/session. This enables
/// tool pruning and per-session cost tracking for clients (like Claude CLI)
/// that don't send the `x-mori-session-id` header.
fn derive_session_id(body: &Value) -> Option<String> {
    let system = body.get("system")?;
    let text = match system {
        Value::String(s) => s.as_str().to_string(),
        Value::Array(arr) => arr
            .iter()
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<&str>>()
            .join(""),
        _ => return None,
    };
    if text.is_empty() {
        return None;
    }
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    Some(format!("auto-{:016x}", h.finish()))
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
    let session_id = session_id_from_headers(headers).or_else(|| derive_session_id(raw));

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
                let tokens_saved = (original_count - pruned_count) * 150; // ~150 tokens per tool def
                let model_name = raw.get("model").and_then(|m| m.as_str()).unwrap_or("");
                let input_price = state.pricing.lookup(model_name).input_per_m;
                let savings_usd = tokens_saved as f64 * input_price / 1e6;
                raw["tools"] = Value::Array(pruned);
                state
                    .stats
                    .tools_pruned_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                state
                    .stats
                    .tool_tokens_saved
                    .fetch_add(tokens_saved as u64, std::sync::atomic::Ordering::Relaxed);
                state.stats.tool_savings_micro_usd.fetch_add(
                    (savings_usd * 1_000_000.0) as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                tracing::info!(
                    session = %sid,
                    original = original_count,
                    pruned = pruned_count,
                    tokens_saved,
                    "pruned unused tool definitions"
                );
            }
        }
    }

    session_id
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

/// Handle a request using the given provider. Response is always in Anthropic format.
///
/// Injects Anthropic cache_control after the hash is computed, calls the provider,
/// taps the stream for stats (streaming path), caches and records stats (non-streaming path).
async fn handle_with_provider(
    state: &AppState,
    provider: &Arc<dyn Provider>,
    mut raw: Value,
    body: &[u8],
    is_streaming: bool,
    start: Instant,
    session_id: Option<&str>,
) -> Result<Response, AppError> {
    // Compress long conversation histories before sending to the provider.
    // This is cheap (~$0.005 via haiku) and saves 40-70% on input tokens.
    if let Ok(saved) = crate::compress::compress_history_if_needed(
        &mut raw,
        &state.http,
        // Use Anthropic keys for the compression call (always haiku).
        &[state.anthropic_api_key.clone()],
    )
    .await
    {
        if saved > 0 {
            state
                .stats
                .history_compressed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            state
                .stats
                .compression_tokens_saved
                .fetch_add(saved, std::sync::atomic::Ordering::Relaxed);
            tracing::info!(saved_tokens = saved, "conversation history compressed");
        }
    }

    // Inject cache_control AFTER hash was computed so the local cache key is unaffected.
    prefix::inject_anthropic_cache_control(&mut raw);

    let provider_name = provider.name().to_string();

    // Semantic cache check (streaming and non-streaming).
    // Skip for tool_result follow-ups — these are context-dependent and the
    // cache text can't distinguish different tool outputs (they all hash the same).
    if !crate::semantic_cache::SemanticCache::last_message_has_tool_results(&raw) {
        let cache_text = crate::semantic_cache::SemanticCache::extract_cache_text(&raw);
        if let Some(hit) = state.semantic_cache.lookup(&cache_text) {
            let elapsed = start.elapsed();
            state
                .stats
                .record_request(&hit.model, 0, 0, 0.0, hit.cost_usd, true);
            state
                .stats
                .semantic_cache_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let empty_usage = UsageDetails::default();
            emit_stats_event(
                &state.stats_tx,
                &state.stats,
                &hit.model,
                "semantic-cache",
                &empty_usage,
                0.0,
                hit.cost_usd,
                hit.cost_usd,
                true,
                false,
                elapsed.as_millis() as u64,
                is_streaming,
                session_id,
            );

            tracing::info!(
                cache = "semantic-hit",
                saved_usd = hit.cost_usd,
                elapsed_ms = elapsed.as_millis(),
                streaming = is_streaming,
                "returning semantically cached response"
            );

            if is_streaming {
                let sse_body = synthesize_sse_from_cached(&hit.response);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .header("x-mori-format", "anthropic")
                    .header("x-mori-stream", "true")
                    .header("x-mori-cache-status", "semantic-hit")
                    .body(Body::from(sse_body))
                    .unwrap_or_default());
            } else {
                return Ok(build_response_with_cost_headers(
                    hit.response,
                    "application/json",
                    hit.cost_usd,
                    0.0,
                    true,
                    &hit.model,
                    elapsed,
                ));
            }
        }
    }

    if is_streaming {
        let byte_stream = provider.stream(&raw).await.map_err(|e| {
            tracing::warn!(provider = %provider_name, error = ?e, "upstream stream error");
            e
        })?;

        let stats = state.stats.clone();
        let stats_tx = state.stats_tx.clone();
        let pricing = state.pricing.clone();
        let sessions = state.sessions.clone();
        let tool_tracker = state.tool_tracker.clone();
        let sid = session_id.map(String::from);
        let stream_start = start;
        let model_name = raw
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();
        let pname = provider_name.clone();

        // Shared accumulator for reconstructing the full response for caching.
        let accumulator = Arc::new(std::sync::Mutex::new(StreamAccumulator::new()));
        let accum_clone = accumulator.clone();
        let cache_for_write = state.cache.clone();
        let sem_cache_for_write = state.semantic_cache.clone();
        let raw_for_cache = raw.clone();
        let skip_sem_cache =
            crate::semantic_cache::SemanticCache::last_message_has_tool_results(&raw);
        let normalized_for_cache = body.to_vec();

        // Mutable state buffered across SSE events within the stream.
        // Anthropic sends input/cache tokens in message_start, output in message_delta.
        let mut input_buf: u64 = 0;
        let mut cache_read_buf: u64 = 0;
        let mut cache_create_buf: u64 = 0;
        let mut thinking_buf: u64 = 0;

        let tapped_stream = byte_stream.map(move |result| {
            if let Ok(ref bytes) = result {
                let text = String::from_utf8_lossy(bytes);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<Value>(data) {
                            // Feed event to accumulator for response reconstruction.
                            if let Ok(mut acc) = accum_clone.lock() {
                                acc.process_event(&event);

                                // On stream completion with end_turn, write to caches.
                                if acc.complete
                                    && acc.stop_reason.as_deref() == Some("end_turn")
                                    && acc.cost > 0.0
                                {
                                    let response_bytes = acc.to_response_bytes();
                                    let model = acc.model.clone();
                                    let cost = acc.cost;
                                    let cache = cache_for_write.clone();
                                    let sem_cache = sem_cache_for_write.clone();
                                    let raw_c = raw_for_cache.clone();
                                    let norm = normalized_for_cache.clone();
                                    // Reset complete to avoid double-writing.
                                    acc.complete = false;

                                    tokio::spawn(async move {
                                        let hash = ResponseCache::request_hash(&norm);
                                        cache
                                            .put(
                                                hash,
                                                CachedResponse {
                                                    body: response_bytes.clone(),
                                                    content_type: "application/json".into(),
                                                    cost_usd: cost,
                                                    model: model.clone(),
                                                    cached_at: chrono::Utc::now(),
                                                },
                                            )
                                            .await;
                                        if !skip_sem_cache {
                                            let cache_text =
                                                crate::semantic_cache::SemanticCache::extract_cache_text(
                                                    &raw_c,
                                                );
                                            sem_cache.store(
                                                &cache_text,
                                                response_bytes,
                                                cost,
                                                model,
                                            );
                                        }
                                    });
                                }
                            }

                            // message_start: buffer input/cache/thinking tokens.
                            if let Some(msg_usage) =
                                event.get("message").and_then(|m| m.get("usage"))
                            {
                                input_buf = msg_usage.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                cache_read_buf = msg_usage.get("cache_read_input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                cache_create_buf = msg_usage.get("cache_creation_input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                thinking_buf = msg_usage.get("thinking_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                            }

                            // message_delta: final output count — record stats once.
                            // For translated OpenAI streams, this also carries input_tokens,
                            // cached_tokens, and reasoning_tokens (set by sse.rs translator).
                            if let Some(usage) = event.get("usage") {
                                let output = usage.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                // OpenAI-translated streams include input tokens in message_delta
                                // (not message_start) because OpenAI sends usage on the last chunk.
                                if let Some(inp) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                                    if inp > 0 { input_buf = inp; }
                                }
                                let oai_cached = usage.get("cached_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                let oai_reasoning = usage.get("reasoning_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                if output > 0 {
                                    let u = UsageDetails {
                                        input_tokens: input_buf,
                                        output_tokens: output,
                                        cache_read_input_tokens: cache_read_buf,
                                        cache_creation_input_tokens: cache_create_buf,
                                        thinking_tokens: thinking_buf,
                                        cached_tokens: oai_cached,
                                        reasoning_tokens: oai_reasoning,
                                    };
                                    let p = pricing.lookup(&model_name);
                                    let (cost, naive) = compute_cost(&u, p, false);
                                    let savings = naive - cost;

                                    // Store cost in accumulator for cache write.
                                    if let Ok(mut acc) = accum_clone.lock() {
                                        acc.cost = cost;
                                        acc.naive_cost = naive;
                                    }

                                    stats.record_request(&model_name, u.input_tokens, output, cost, naive, false);

                                    if let Some(ref sid) = sid {
                                        sessions.entry(sid.clone())
                                            .or_insert_with(|| SessionCost::new(sid.clone()))
                                            .record(&model_name, &pname, u.input_tokens, output, cost, 0.0);
                                    }

                                    tracing::info!(
                                        model = %model_name,
                                        provider = %pname,
                                        input_tokens = u.input_tokens,
                                        output_tokens = output,
                                        cache_read = u.cache_read_input_tokens,
                                        thinking = u.thinking_tokens,
                                        cost_usd = cost,
                                        savings_usd = savings,
                                        stream = true,
                                        "stream complete"
                                    );

                                    emit_stats_event(
                                        &stats_tx, &stats, &model_name, &pname,
                                        &u, cost, naive, savings,
                                        false, false,
                                        stream_start.elapsed().as_millis() as u64,
                                        true, sid.as_deref(),
                                    );
                                }
                            }

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

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("x-mori-format", "anthropic")
            .header("x-mori-stream", "true")
            .header("x-mori-provider", provider_name)
            .body(Body::from_stream(tapped_stream))
            .unwrap_or_default())
    } else {
        // In-flight coalescing: if another request with the same body hash is
        // already in-flight, wait for its result instead of sending a duplicate
        // upstream request. This is common when multiple mori agents share the
        // same system prompt + tools and send near-identical requests.
        let coalesce_hash = ResponseCache::request_hash(body);

        // Check if there's already an in-flight request for this hash.
        if let Some(sender) = state.inflight.get(&coalesce_hash) {
            let mut rx = sender.subscribe();
            drop(sender); // Release DashMap lock before awaiting.
            tracing::info!("coalescing with in-flight request");
            state
                .stats
                .coalesced_requests
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            match rx.recv().await {
                Ok(Ok(response_bytes)) => {
                    let elapsed = start.elapsed();
                    // Parse the coalesced response for cost tracking.
                    if let Ok(parsed) = serde_json::from_slice::<Value>(&response_bytes) {
                        let model = parsed
                            .get("model")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown");
                        let pricing = state.pricing.lookup(model);
                        let u = UsageDetails::default(); // Coalesced — didn't pay for this one.
                        let naive_cost = pricing.input_per_m * 0.001; // rough estimate for coalesced
                        state
                            .stats
                            .record_request(model, 0, 0, 0.0, naive_cost, true);

                        emit_stats_event(
                            &state.stats_tx,
                            &state.stats,
                            model,
                            "coalesced",
                            &u,
                            0.0,
                            naive_cost,
                            naive_cost,
                            true,
                            false,
                            elapsed.as_millis() as u64,
                            false,
                            session_id,
                        );
                    }
                    return Ok(build_response_with_cost_headers(
                        response_bytes,
                        "application/json",
                        0.0,
                        0.0,
                        true,
                        "coalesced",
                        elapsed,
                    ));
                }
                _ => {
                    // Original request failed or channel closed — fall through to make our own request.
                    tracing::debug!("coalesced request failed, making own request");
                }
            }
        }

        // Register ourselves as the in-flight request for this hash.
        let (inflight_tx, _) = tokio::sync::broadcast::channel::<Result<Bytes, String>>(4);
        state.inflight.insert(coalesce_hash, inflight_tx.clone());

        let provider_resp = provider.complete(&raw).await.map_err(|e| {
            // Remove ourselves from inflight map on error, notify waiters.
            state.inflight.remove(&coalesce_hash);
            let _ = inflight_tx.send(Err(format!("{e:?}")));
            tracing::warn!(provider = %provider_name, error = ?e, "upstream complete error");
            e
        })?;

        // Notify any coalesced waiters with the response.
        let _ = inflight_tx.send(Ok(provider_resp.raw.clone()));
        state.inflight.remove(&coalesce_hash);

        let model = provider_resp
            .body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        let u = &provider_resp.usage;
        let pricing = state.pricing.lookup(model);
        let (cost, naive) = compute_cost(u, pricing, false);
        let savings = naive - cost;

        let elapsed = start.elapsed();

        // Only cache end_turn responses (tool_use responses have unique IDs that can't be replayed).
        let stop_reason = provider_resp
            .body
            .get("stop_reason")
            .and_then(|s| s.as_str());
        if stop_reason == Some("end_turn") {
            let hash = ResponseCache::request_hash(body);
            state
                .cache
                .put(
                    hash,
                    CachedResponse {
                        body: provider_resp.raw.clone(),
                        content_type: "application/json".into(),
                        cost_usd: cost,
                        model: model.to_string(),
                        cached_at: chrono::Utc::now(),
                    },
                )
                .await;

            // Also store in the semantic cache for fuzzy matching.
            // Skip if the request contained tool_result — those produce
            // degenerate cache keys (empty user text) and cause false matches.
            if !crate::semantic_cache::SemanticCache::last_message_has_tool_results(&raw) {
                let cache_text = crate::semantic_cache::SemanticCache::extract_cache_text(&raw);
                state.semantic_cache.store(
                    &cache_text,
                    provider_resp.raw.clone(),
                    cost,
                    model.to_string(),
                );
            }
        }

        state
            .stats
            .record_request(model, u.input_tokens, u.output_tokens, cost, naive, false);

        emit_stats_event(
            &state.stats_tx,
            &state.stats,
            model,
            &provider_resp.provider,
            u,
            cost,
            naive,
            savings,
            false,
            false,
            elapsed.as_millis() as u64,
            false,
            session_id,
        );

        if let Some(sid) = session_id {
            record_session_cost(
                state,
                sid,
                model,
                &provider_resp.provider,
                u.input_tokens,
                u.output_tokens,
                cost,
                0.0,
            );
            record_tool_usage(state, sid, &provider_resp.body);
        }

        tracing::info!(
            model = %model,
            provider = %provider_resp.provider,
            input_tokens = u.input_tokens,
            output_tokens = u.output_tokens,
            cache_read_tokens = u.cache_read_input_tokens,
            cost_usd = cost,
            elapsed_ms = elapsed.as_millis(),
            "request complete"
        );

        let prefix_cache_status = if u.cache_read_input_tokens > 0 || u.cached_tokens > 0 {
            "warm"
        } else {
            "cold"
        };
        let mut resp = build_response_with_cost_headers(
            provider_resp.raw,
            "application/json",
            cost,
            cost,
            false,
            model,
            elapsed,
        );
        resp.headers_mut().insert(
            "x-mori-prefix-cache-status",
            prefix_cache_status.parse().unwrap(),
        );
        resp.headers_mut()
            .insert("x-mori-provider", provider_resp.provider.parse().unwrap());
        Ok(resp)
    }
}

/// Unified messages endpoint: auto-detects Anthropic vs OpenAI format,
/// forwards to the appropriate provider, returns in the caller's format.
pub async fn messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let _permit = acquire_concurrency(&state).await?;
    let start = Instant::now();

    // Parse raw JSON
    let mut raw: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid JSON: {e}")))?;

    // Check for T0 suppression before any other work
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

    // Apply preprocessing (tier routing model override, tool pruning)
    let session_id = preprocess_request(&state, &mut raw, &headers);

    // Strip per-request variable content (UUIDs, timestamps) from system prompt
    prefix::strip_variable_content(&mut raw);
    prefix::sort_tools_by_name(&mut raw);
    state
        .stats
        .variables_stripped
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state
        .stats
        .requests_normalized
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Normalize JSON key ordering in place for better cache hits (avoids full clone)
    prefix::normalize_json_ordering_in_place(&mut raw);
    let normalized_bytes = serde_json::to_vec(&raw).unwrap_or_default();

    let is_streaming = raw.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    // Check hash cache (streaming and non-streaming)
    let hash = ResponseCache::request_hash(&normalized_bytes);
    if let Some(cached) = state.cache.get(&hash).await {
        let elapsed = start.elapsed();
        state
            .stats
            .record_request(&cached.model, 0, 0, 0.0, cached.cost_usd, true);
        state
            .stats
            .hash_cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        emit_stats_event(
            &state.stats_tx,
            &state.stats,
            &cached.model,
            "cache",
            &UsageDetails::default(),
            0.0,
            cached.cost_usd,
            cached.cost_usd,
            true,
            false,
            elapsed.as_millis() as u64,
            is_streaming,
            session_id.as_deref(),
        );

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
            streaming = is_streaming,
            "returning cached response"
        );

        if is_streaming {
            let sse_body = synthesize_sse_from_cached(&cached.body);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("x-mori-format", "anthropic")
                .header("x-mori-stream", "true")
                .header("x-mori-cache-status", "hash-hit")
                .body(Body::from(sse_body))
                .unwrap_or_default());
        } else {
            return Ok(build_response_with_cost_headers(
                cached.body,
                &cached.content_type,
                cached.cost_usd,
                0.0,
                true,
                &cached.model,
                elapsed,
            ));
        }
    }

    let model = raw.get("model").and_then(|m| m.as_str()).unwrap_or("");

    let provider = state.resolve_provider(model).ok_or_else(|| {
        AppError::BadRequest(format!("no provider configured for model '{model}'"))
    })?;

    handle_with_provider(
        &state,
        provider,
        raw,
        &normalized_bytes,
        is_streaming,
        start,
        session_id.as_deref(),
    )
    .await
}

/// OpenAI-compatible chat completions endpoint.
///
/// Translates between OpenAI format and the provider system. Any configured provider
/// can be used — claude models, gpt models, or anything on OpenRouter.
pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let _permit = acquire_concurrency(&state).await?;
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

    let session_id = preprocess_request(&state, &mut raw, &headers);
    prefix::normalize_json_ordering_in_place(&mut raw);

    let model = raw.get("model").and_then(|m| m.as_str()).unwrap_or("");
    let is_streaming = raw.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    let provider = state.resolve_provider(model).ok_or_else(|| {
        AppError::BadRequest(format!("no provider configured for model '{model}'"))
    })?;

    // Translate OpenAI format → Anthropic format (internal canonical format).
    let openai_req: format::OpenAiRequest = serde_json::from_value(raw.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid OpenAI request: {e}")))?;
    let anthropic_req = format::openai_to_anthropic(&openai_req);
    let anthropic_body =
        serde_json::to_value(&anthropic_req).map_err(|e| AppError::Internal(e.to_string()))?;
    let anthropic_bytes =
        serde_json::to_vec(&anthropic_body).map_err(|e| AppError::Internal(e.to_string()))?;

    // Check hash cache (streaming and non-streaming)
    let hash = ResponseCache::request_hash(&anthropic_bytes);
    if let Some(cached) = state.cache.get(&hash).await {
        let elapsed = start.elapsed();
        state
            .stats
            .record_request(&cached.model, 0, 0, 0.0, cached.cost_usd, true);
        state
            .stats
            .hash_cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        emit_stats_event(
            &state.stats_tx,
            &state.stats,
            &cached.model,
            "cache",
            &UsageDetails::default(),
            0.0,
            cached.cost_usd,
            cached.cost_usd,
            true,
            false,
            elapsed.as_millis() as u64,
            is_streaming,
            session_id.as_deref(),
        );

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
            streaming = is_streaming,
            endpoint = "chat_completions",
            "returning cached response"
        );

        if is_streaming {
            let sse_body = synthesize_openai_sse_from_cached(&cached.body, model);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("x-mori-format", "openai")
                .header("x-mori-stream", "true")
                .header("x-mori-cache-status", "hash-hit")
                .body(Body::from(sse_body))
                .unwrap_or_default());
        } else {
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
    }

    if is_streaming {
        let mut provider_raw = anthropic_body.clone();
        prefix::inject_anthropic_cache_control(&mut provider_raw);

        let byte_stream = provider.stream(&provider_raw).await.map_err(|e| {
            tracing::warn!(provider = %provider.name(), model = %model, error = ?e, "upstream stream error (chat_completions)");
            e
        })?;

        // Tap the raw Anthropic SSE stream for stats before translating to OpenAI format.
        let stats = state.stats.clone();
        let stats_tx = state.stats_tx.clone();
        let pricing = state.pricing.clone();
        let sessions = state.sessions.clone();
        let sid = session_id.clone();
        let stream_start = start;
        let model_str = model.to_string();
        let pname = provider.name().to_string();

        // Shared accumulator for cache writes from streaming.
        let accumulator = Arc::new(std::sync::Mutex::new(StreamAccumulator::new()));
        let accum_clone = accumulator.clone();
        let cache_for_write = state.cache.clone();
        let sem_cache_for_write = state.semantic_cache.clone();
        let anthropic_body_for_cache = anthropic_body.clone();
        let anthropic_bytes_for_cache = anthropic_bytes.clone();
        let model_for_cache = model.to_string();
        let skip_sem_cache_oai =
            crate::semantic_cache::SemanticCache::last_message_has_tool_results(&anthropic_body);

        let mut input_buf: u64 = 0;
        let mut cache_read_buf: u64 = 0;
        let mut cache_create_buf: u64 = 0;
        let mut thinking_buf: u64 = 0;

        let tapped = byte_stream.map(move |result| {
            if let Ok(ref bytes) = result {
                let text = String::from_utf8_lossy(bytes);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<Value>(data) {
                            // Feed event to accumulator for response reconstruction.
                            if let Ok(mut acc) = accum_clone.lock() {
                                acc.process_event(&event);

                                if acc.complete
                                    && acc.stop_reason.as_deref() == Some("end_turn")
                                    && acc.cost > 0.0
                                {
                                    let anth_response = acc.to_response_bytes();
                                    let model = acc.model.clone();
                                    let cost = acc.cost;
                                    let cache = cache_for_write.clone();
                                    let sem_cache = sem_cache_for_write.clone();
                                    let anth_body = anthropic_body_for_cache.clone();
                                    let anth_bytes = anthropic_bytes_for_cache.clone();
                                    let m_str = model_for_cache.clone();
                                    acc.complete = false;

                                    tokio::spawn(async move {
                                        // Translate to OpenAI format for cache storage
                                        // (matches what the non-streaming path stores).
                                        let anth_val: Value =
                                            serde_json::from_slice(&anth_response)
                                                .unwrap_or_default();
                                        let openai_json =
                                            format::anthropic_body_to_openai_response(&anth_val);
                                        let openai_bytes = Bytes::from(
                                            serde_json::to_vec(&openai_json).unwrap_or_default(),
                                        );

                                        let hash = ResponseCache::request_hash(&anth_bytes);
                                        cache
                                            .put(
                                                hash,
                                                CachedResponse {
                                                    body: openai_bytes,
                                                    content_type: "application/json".into(),
                                                    cost_usd: cost,
                                                    model: m_str,
                                                    cached_at: chrono::Utc::now(),
                                                },
                                            )
                                            .await;

                                        if !skip_sem_cache_oai {
                                            let cache_text =
                                                crate::semantic_cache::SemanticCache::extract_cache_text(
                                                    &anth_body,
                                                );
                                            sem_cache.store(
                                                &cache_text,
                                                anth_response,
                                                cost,
                                                model,
                                            );
                                        }
                                    });
                                }
                            }

                            if let Some(msg_usage) = event.get("message").and_then(|m| m.get("usage")) {
                                input_buf = msg_usage.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                cache_read_buf = msg_usage.get("cache_read_input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                cache_create_buf = msg_usage.get("cache_creation_input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                thinking_buf = msg_usage.get("thinking_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                            }
                            if let Some(usage) = event.get("usage") {
                                let output = usage.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                if let Some(inp) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                                    if inp > 0 { input_buf = inp; }
                                }
                                let oai_cached = usage.get("cached_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                let oai_reasoning = usage.get("reasoning_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                                if output > 0 {
                                    let u = UsageDetails {
                                        input_tokens: input_buf, output_tokens: output,
                                        cache_read_input_tokens: cache_read_buf,
                                        cache_creation_input_tokens: cache_create_buf,
                                        thinking_tokens: thinking_buf,
                                        cached_tokens: oai_cached,
                                        reasoning_tokens: oai_reasoning,
                                    };
                                    let p = pricing.lookup(&model_str);
                                    let (cost, naive) = compute_cost(&u, p, false);
                                    let savings = naive - cost;

                                    // Store cost in accumulator for cache write.
                                    if let Ok(mut acc) = accum_clone.lock() {
                                        acc.cost = cost;
                                        acc.naive_cost = naive;
                                    }

                                    stats.record_request(&model_str, u.input_tokens, output, cost, naive, false);

                                    if let Some(ref sid) = sid {
                                        sessions.entry(sid.clone())
                                            .or_insert_with(|| SessionCost::new(sid.clone()))
                                            .record(&model_str, &pname, u.input_tokens, output, cost, 0.0);
                                    }

                                    emit_stats_event(
                                        &stats_tx, &stats, &model_str, &pname,
                                        &u, cost, naive, savings,
                                        false, false,
                                        stream_start.elapsed().as_millis() as u64,
                                        true, sid.as_deref(),
                                    );

                                    tracing::info!(
                                        model = %model_str, provider = %pname,
                                        input_tokens = u.input_tokens, output_tokens = output,
                                        cost_usd = cost, savings_usd = savings,
                                        stream = true,
                                        "chat_completions stream complete"
                                    );
                                }
                            }
                        }
                    }
                }
            }
            result
        });

        // Translate the tapped Anthropic SSE → OpenAI SSE for the client.
        let translated = sse::anthropic_stream_to_openai(Box::pin(tapped), model.to_string());
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("x-mori-format", "openai")
            .header("x-mori-stream", "true")
            .header("x-mori-provider", provider.name())
            .body(Body::from_stream(translated))
            .unwrap_or_default())
    } else {
        let mut provider_raw = anthropic_body.clone();
        prefix::inject_anthropic_cache_control(&mut provider_raw);

        let provider_resp = provider.complete(&provider_raw).await.map_err(|e| {
            tracing::warn!(provider = %provider.name(), model = %model, error = ?e, "upstream complete error (chat_completions)");
            e
        })?;

        let resp_model = provider_resp
            .body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        let u = &provider_resp.usage;
        let p = state.pricing.lookup(resp_model);
        let (cost, naive) = compute_cost(u, p, false);
        let savings = naive - cost;

        state.stats.record_request(
            resp_model,
            u.input_tokens,
            u.output_tokens,
            cost,
            naive,
            false,
        );

        emit_stats_event(
            &state.stats_tx,
            &state.stats,
            resp_model,
            &provider_resp.provider,
            u,
            cost,
            naive,
            savings,
            false,
            false,
            start.elapsed().as_millis() as u64,
            false,
            session_id.as_deref(),
        );

        if let Some(ref sid) = session_id {
            record_session_cost(
                &state,
                sid,
                resp_model,
                &provider_resp.provider,
                u.input_tokens,
                u.output_tokens,
                cost,
                0.0,
            );
            record_tool_usage(&state, sid, &provider_resp.body);
        }

        // Translate Anthropic response → OpenAI format for this client.
        let openai_json = format::anthropic_body_to_openai_response(&provider_resp.body);
        let openai_bytes = Bytes::from(serde_json::to_vec(&openai_json).unwrap_or_default());

        // Only cache end_turn responses.
        let stop_reason = provider_resp
            .body
            .get("stop_reason")
            .and_then(|s| s.as_str());
        if stop_reason == Some("end_turn") {
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
        }

        let elapsed = start.elapsed();
        tracing::info!(
            model = %resp_model,
            provider = %provider_resp.provider,
            input_tokens = u.input_tokens,
            output_tokens = u.output_tokens,
            cache_read_tokens = u.cache_read_input_tokens,
            cost_usd = cost,
            elapsed_ms = elapsed.as_millis(),
            "chat_completions request complete"
        );

        Ok(build_response_with_cost_headers(
            openai_bytes,
            "application/json",
            cost,
            cost,
            false,
            resp_model,
            elapsed,
        ))
    }
}

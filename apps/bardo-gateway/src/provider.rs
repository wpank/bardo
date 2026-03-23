//! Provider backends that forward requests to LLM APIs.

use bytes::Bytes;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;

use crate::error::AppError;

/// Maximum number of retries for transient upstream errors (429, 529, 5xx).
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (doubled each retry).
const BASE_RETRY_DELAY_MS: u64 = 500;

/// Whether an HTTP status code is retryable (429 rate-limit or 5xx server error).
fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

/// Anthropic API fields that are valid in the Messages API.
/// Anything not in this list gets stripped before forwarding.
const ANTHROPIC_ALLOWED_FIELDS: &[&str] = &[
    "model",
    "messages",
    "max_tokens",
    "system",
    "temperature",
    "top_p",
    "top_k",
    "stream",
    "stop_sequences",
    "tools",
    "tool_choice",
    "metadata",
    "thinking",
];

/// Strip fields that the Anthropic API doesn't accept and fix message
/// structure issues (orphaned tool_use/tool_result blocks, consecutive
/// text blocks, content after tool_use).
///
/// This is the last function before the HTTP call — nothing can modify
/// messages after this runs.
fn sanitize_anthropic_request(body: &Value) -> Value {
    let mut clean = if let Value::Object(map) = body {
        let mut c = serde_json::Map::new();
        for (key, value) in map {
            if ANTHROPIC_ALLOWED_FIELDS.contains(&key.as_str()) {
                c.insert(key.clone(), value.clone());
            } else {
                tracing::debug!(field = %key, "stripping non-API field from request");
            }
        }
        Value::Object(c)
    } else {
        body.clone()
    };

    // Merge consecutive text blocks in message content arrays. Cached
    // responses reconstructed from SSE deltas can have hundreds of tiny
    // text blocks, which hits Anthropic's content block limit.
    merge_text_blocks(&mut clean);

    // Truncate assistant messages after the last tool_use block.
    // The API requires the turn to end at tool_use so the next message
    // can provide tool_result.
    truncate_after_tool_use(&mut clean);

    // Fix orphaned tool_use/tool_result blocks in the messages array.
    sanitize_tool_blocks(&mut clean);

    // If the request has no `tools` field (or it's empty), strip all
    // tool_use/tool_result blocks from history. The API rejects tool_use
    // references when no tools are defined.
    let has_tools = clean
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    if !has_tools {
        strip_all_tool_blocks(&mut clean);
    }

    clean
}

/// Merge consecutive text content blocks within each message.
///
/// Cached responses reconstructed from SSE stream deltas can produce
/// hundreds of tiny `{type: "text", text: "..."}` blocks.
fn merge_text_blocks(body: &mut Value) {
    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    for msg in messages.iter_mut() {
        let content = match msg.get_mut("content").and_then(|c| c.as_array_mut()) {
            Some(c) => c,
            None => continue,
        };

        if content.len() <= 1 {
            continue;
        }

        let has_consecutive_text = content.windows(2).any(|w| {
            w[0].get("type").and_then(|t| t.as_str()) == Some("text")
                && w[1].get("type").and_then(|t| t.as_str()) == Some("text")
        });

        if !has_consecutive_text {
            continue;
        }

        let mut merged: Vec<Value> = Vec::with_capacity(content.len());
        let mut text_buf = String::new();

        for block in content.iter() {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    text_buf.push_str(t);
                }
            } else {
                if !text_buf.is_empty() {
                    merged.push(serde_json::json!({"type": "text", "text": text_buf}));
                    text_buf.clear();
                }
                merged.push(block.clone());
            }
        }
        if !text_buf.is_empty() {
            merged.push(serde_json::json!({"type": "text", "text": text_buf}));
        }

        let old_len = content.len();
        *content = merged;

        if content.len() < old_len {
            tracing::info!(
                old_blocks = old_len,
                new_blocks = content.len(),
                "merged consecutive text blocks in message"
            );
        }
    }
}

/// Truncate assistant message content after the last tool_use block.
///
/// The API requires that when an assistant message contains tool_use blocks,
/// no content follows after them — the next message must be a user message
/// with tool_result.
fn truncate_after_tool_use(body: &mut Value) {
    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    for msg in messages.iter_mut() {
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            continue;
        }
        let content = match msg.get_mut("content").and_then(|c| c.as_array_mut()) {
            Some(c) => c,
            None => continue,
        };

        let last_tool_use_idx = content
            .iter()
            .rposition(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"));

        if let Some(idx) = last_tool_use_idx {
            if idx + 1 < content.len() {
                let removed = content.len() - idx - 1;
                content.truncate(idx + 1);
                tracing::warn!(
                    removed_blocks = removed,
                    "truncated content after tool_use in assistant message"
                );
            }
        }
    }
}

/// Remove ALL tool_use and tool_result content blocks from messages.
///
/// Used when the request has no `tools` field — the API rejects tool_use
/// references in conversation history when no tools are defined.
fn strip_all_tool_blocks(body: &mut Value) {
    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    let mut stripped = 0usize;
    for msg in messages.iter_mut() {
        if let Some(content) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
            let before = content.len();
            content.retain(|b| {
                let btype = b.get("type").and_then(|t| t.as_str()).unwrap_or("");
                btype != "tool_use" && btype != "tool_result"
            });
            stripped += before - content.len();

            if content.is_empty() {
                *content =
                    vec![serde_json::json!({"type": "text", "text": "[tool interaction removed]"})];
            }
        }
    }

    if stripped > 0 {
        tracing::warn!(stripped, "stripped all tool blocks (no tools field)");
    }
}

/// Fix orphaned tool_use/tool_result blocks in the messages array.
///
/// 1. Strip tool_use blocks without an `id` field (malformed)
/// 2. Strip tool_use blocks with no matching tool_result in the next message
/// 3. Re-collect IDs, then strip orphaned tool_result blocks
/// 4. Replace any now-empty content arrays with placeholder text
/// 5. Final validation pass to catch anything the above missed
fn sanitize_tool_blocks(body: &mut Value) {
    use std::collections::HashSet;

    let messages = match body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        Some(m) => m,
        None => return,
    };

    let len = messages.len();
    if len == 0 {
        return;
    }

    let mut modified = false;

    // Forward pass: collect tool_result IDs, then strip invalid tool_use blocks.
    {
        let result_ids_per_msg: Vec<HashSet<String>> = messages
            .iter()
            .map(|msg| {
                let mut ids = HashSet::new();
                if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                        for block in content {
                            if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                                if let Some(id) = block.get("tool_use_id").and_then(|i| i.as_str())
                                {
                                    ids.insert(id.to_string());
                                }
                            }
                        }
                    }
                }
                ids
            })
            .collect();

        let empty: HashSet<String> = HashSet::new();

        for i in 0..len {
            if messages[i].get("role").and_then(|r| r.as_str()) != Some("assistant") {
                continue;
            }

            let next_results = if i + 1 < len {
                &result_ids_per_msg[i + 1]
            } else {
                &empty
            };

            let content = match messages[i]
                .get_mut("content")
                .and_then(|c| c.as_array_mut())
            {
                Some(c) => c,
                None => continue,
            };

            let before = content.len();
            content.retain(|b| {
                if b.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                    return true;
                }
                match b.get("id").and_then(|id| id.as_str()) {
                    Some(id) if next_results.contains(id) => true,
                    Some(id) => {
                        tracing::warn!(msg_idx = i, id, "stripping orphaned tool_use");
                        false
                    }
                    None => {
                        tracing::warn!(msg_idx = i, "stripping tool_use without id field");
                        false
                    }
                }
            });
            if content.len() < before {
                modified = true;
            }
        }
    }

    // Reverse pass: re-collect tool_use IDs (reflecting forward-pass changes),
    // then strip orphaned tool_result blocks.
    {
        let use_ids_per_msg: Vec<HashSet<String>> = messages
            .iter()
            .map(|msg| {
                let mut ids = HashSet::new();
                if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                        for block in content {
                            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                if let Some(id) = block.get("id").and_then(|i| i.as_str()) {
                                    ids.insert(id.to_string());
                                }
                            }
                        }
                    }
                }
                ids
            })
            .collect();

        let empty: HashSet<String> = HashSet::new();

        for i in 0..len {
            if messages[i].get("role").and_then(|r| r.as_str()) != Some("user") {
                continue;
            }

            let prev_uses = if i > 0 {
                &use_ids_per_msg[i - 1]
            } else {
                &empty
            };

            let content = match messages[i]
                .get_mut("content")
                .and_then(|c| c.as_array_mut())
            {
                Some(c) => c,
                None => continue,
            };

            let before = content.len();
            content.retain(|b| {
                if b.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
                    return true;
                }
                match b.get("tool_use_id").and_then(|id| id.as_str()) {
                    Some(id) if prev_uses.contains(id) => true,
                    Some(id) => {
                        tracing::warn!(msg_idx = i, id, "stripping orphaned tool_result");
                        false
                    }
                    None => {
                        tracing::warn!(msg_idx = i, "stripping tool_result without tool_use_id");
                        false
                    }
                }
            });
            if content.len() < before {
                modified = true;
            }
        }
    }

    // Fix empty content arrays left by stripping.
    for i in 0..len {
        if let Some(content) = messages[i]
            .get_mut("content")
            .and_then(|c| c.as_array_mut())
        {
            if content.is_empty() {
                *content =
                    vec![serde_json::json!({"type": "text", "text": "[tool interaction removed]"})];
                modified = true;
            }
        }
    }

    // Final safety net: if any tool_use still lacks a matching tool_result,
    // force-strip it. This catches edge cases the above passes might miss.
    {
        let mut remaining = false;
        for i in 0..len {
            if messages[i].get("role").and_then(|r| r.as_str()) != Some("assistant") {
                continue;
            }
            let has_tool_use = messages[i]
                .get("content")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                })
                .unwrap_or(false);
            if !has_tool_use {
                continue;
            }

            let next_result_ids: std::collections::HashSet<String> = if i + 1 < len {
                messages[i + 1]
                    .get("content")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|b| {
                                if b.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                                    b.get("tool_use_id")
                                        .and_then(|i| i.as_str())
                                        .map(String::from)
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                std::collections::HashSet::new()
            };

            if let Some(content) = messages[i]
                .get_mut("content")
                .and_then(|c| c.as_array_mut())
            {
                let before = content.len();
                content.retain(|b| {
                    if b.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                        return true;
                    }
                    let id = b.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    if next_result_ids.contains(id) {
                        true
                    } else {
                        tracing::error!(
                            msg_idx = i,
                            tool_use_id = id,
                            "safety-net: force-stripping orphaned tool_use"
                        );
                        false
                    }
                });
                if content.len() < before {
                    remaining = true;
                    if content.is_empty() {
                        *content = vec![
                            serde_json::json!({"type": "text", "text": "[tool interaction removed]"}),
                        ];
                    }
                }
            }
        }
        if remaining {
            modified = true;
        }
    }

    if modified {
        tracing::warn!("sanitized tool blocks in API request");
    }
}

/// Send an HTTP request to Anthropic with automatic retry on transient errors.
///
/// Accepts multiple API keys and rotates through them on 429 (rate limit) errors.
/// On 429: immediately try the next key (no delay if a fresh key is available).
/// On 5xx: retry with exponential backoff on the same key.
/// Respects the `retry-after` header when present.
async fn anthropic_request_with_retry(
    http: &reqwest::Client,
    api_keys: &[String],
    body: &Value,
) -> Result<reqwest::Response, AppError> {
    let num_keys = api_keys.len();

    for attempt in 0..=MAX_RETRIES {
        let key_idx = attempt as usize % num_keys;
        let api_key = &api_keys[key_idx];

        let resp = http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "prompt-caching-2024-07-31")
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                tracing::warn!(attempt = attempt + 1, key_index = key_idx, error = %e, "anthropic network error");
                AppError::ProviderError(e.to_string())
            })?;

        let status = resp.status();
        if status.is_success() {
            if attempt > 0 {
                tracing::info!(
                    attempt = attempt + 1,
                    key_index = key_idx,
                    "anthropic request succeeded on retry"
                );
            }
            return Ok(resp);
        }

        // Non-retryable error or final attempt — return the error immediately.
        if !is_retryable_status(status.as_u16()) || attempt == MAX_RETRIES {
            let status_code = status.as_u16();
            let err_body = resp.text().await.unwrap_or_default();
            tracing::warn!(
                status = %status,
                attempt = attempt + 1,
                key_index = key_idx,
                body = %&err_body[..err_body.len().min(300)],
                "anthropic request failed (not retrying)"
            );
            // Forward upstream 4xx as-is so callers know the request was
            // rejected, not that the gateway is broken (which 502 implies).
            if (400..500).contains(&status_code) {
                return Err(AppError::UpstreamClientError {
                    status: status_code,
                    body: err_body,
                });
            }
            return Err(AppError::ProviderError(format!("{status}: {err_body}")));
        }

        // Retryable error — extract delay and retry.
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());

        // On 429 with another key available: skip the delay (the next key
        // has its own rate limit budget). Otherwise: exponential backoff.
        let is_rate_limit = status.as_u16() == 429;
        let next_key_idx = (attempt as usize + 1) % num_keys;
        let skip_delay = is_rate_limit && next_key_idx != key_idx;

        let delay = if skip_delay {
            std::time::Duration::ZERO
        } else {
            retry_after
                .map(|s| std::time::Duration::from_secs_f64(s.min(30.0)))
                .unwrap_or_else(|| {
                    std::time::Duration::from_millis(BASE_RETRY_DELAY_MS * 2u64.pow(attempt))
                })
        };

        let err_body = resp.text().await.unwrap_or_default();
        tracing::warn!(
            status = %status,
            attempt = attempt + 1,
            max_retries = MAX_RETRIES,
            key_index = key_idx,
            next_key_index = next_key_idx,
            delay_ms = delay.as_millis(),
            body = %&err_body[..err_body.len().min(200)],
            "retrying anthropic request after transient error"
        );

        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }

    unreachable!()
}

/// Forward a non-streaming request to the Anthropic API using raw JSON (no type deserialization).
/// This handles any message format Claude Code might send.
/// Strips non-API fields (context_management, etc.) before forwarding.
/// Automatically retries on 429/5xx with key rotation and exponential backoff.
pub async fn anthropic_complete_raw(
    http: &reqwest::Client,
    api_keys: &[String],
    body: &Value,
) -> Result<(Value, Bytes), AppError> {
    let clean_body = sanitize_anthropic_request(body);

    let resp = anthropic_request_with_retry(http, api_keys, &clean_body).await?;

    let body_bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let parsed: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| AppError::Internal(format!("parse error: {e}")))?;

    Ok((parsed, body_bytes))
}

/// Forward a streaming request to the Anthropic API using raw JSON.
/// Strips non-API fields before forwarding.
/// Automatically retries on 429/5xx with key rotation and exponential backoff.
pub async fn anthropic_stream_raw(
    http: &reqwest::Client,
    api_keys: &[String],
    body: &Value,
) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, AppError> {
    let mut stream_body = sanitize_anthropic_request(body);
    stream_body["stream"] = Value::Bool(true);

    let resp = anthropic_request_with_retry(http, api_keys, &stream_body).await?;

    Ok(Box::pin(resp.bytes_stream()))
}

/// Forward a non-streaming request to the OpenAI API.
pub async fn openai_complete(
    http: &reqwest::Client,
    api_key: &str,
    body: &Value,
) -> Result<(Value, Bytes), AppError> {
    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .header("authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| AppError::ProviderError(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        if status.as_u16() < 500 {
            return Err(AppError::UpstreamClientError {
                status: status.as_u16(),
                body: err_body,
            });
        }
        return Err(AppError::ProviderError(format!("{status}: {err_body}")));
    }

    let body_bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let parsed: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| AppError::Internal(format!("parse error: {e}")))?;

    Ok((parsed, body_bytes))
}

/// Forward a streaming request to the OpenAI API, returning the raw byte stream.
pub async fn openai_stream(
    http: &reqwest::Client,
    api_key: &str,
    body: &Value,
) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, AppError> {
    let mut stream_body = body.clone();
    stream_body["stream"] = Value::Bool(true);

    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .header("authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .json(&stream_body)
        .send()
        .await
        .map_err(|e| AppError::ProviderError(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        if status.as_u16() < 500 {
            return Err(AppError::UpstreamClientError {
                status: status.as_u16(),
                body: err_body,
            });
        }
        return Err(AppError::ProviderError(format!("{status}: {err_body}")));
    }

    Ok(Box::pin(resp.bytes_stream()))
}

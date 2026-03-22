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

/// Strip fields that the Anthropic API doesn't accept.
/// Claude Code adds internal fields (context_management, etc.) that cause 400 errors.
fn sanitize_anthropic_request(body: &Value) -> Value {
    if let Value::Object(map) = body {
        let mut clean = serde_json::Map::new();
        for (key, value) in map {
            if ANTHROPIC_ALLOWED_FIELDS.contains(&key.as_str()) {
                clean.insert(key.clone(), value.clone());
            } else {
                tracing::debug!(field = %key, "stripping non-API field from request");
            }
        }
        Value::Object(clean)
    } else {
        body.clone()
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
                tracing::info!(attempt = attempt + 1, key_index = key_idx, "anthropic request succeeded on retry");
            }
            return Ok(resp);
        }

        // Non-retryable error or final attempt — return the error immediately.
        if !is_retryable_status(status.as_u16()) || attempt == MAX_RETRIES {
            let err_body = resp.text().await.unwrap_or_default();
            tracing::warn!(
                status = %status,
                attempt = attempt + 1,
                key_index = key_idx,
                body = %&err_body[..err_body.len().min(300)],
                "anthropic request failed (not retrying)"
            );
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
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::ProviderError(format!("{status}: {body}")));
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
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::ProviderError(format!("{status}: {body}")));
    }

    Ok(Box::pin(resp.bytes_stream()))
}

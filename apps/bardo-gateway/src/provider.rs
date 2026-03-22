//! Provider backends that forward requests to LLM APIs.

use bytes::Bytes;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;

use crate::error::AppError;

/// Configuration for a single provider backend.
#[derive(Clone, Debug)]
pub struct ProviderConfig {
    /// Provider name (e.g. "anthropic", "openai").
    pub name: String,
    /// Lower number = tried first.
    pub priority: u8,
    /// Whether this provider is currently enabled.
    pub enabled: bool,
}

/// Whether an HTTP status code is retryable (429 rate-limit or 5xx server error).
fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

/// Try providers in priority order for a non-streaming request.
///
/// Iterates enabled providers sorted by priority. On retryable errors (429, 5xx,
/// network failures), moves to the next provider. On non-retryable client errors
/// (4xx other than 429), returns the error immediately.
pub async fn route_with_fallback(
    http: &reqwest::Client,
    anthropic_key: &str,
    openai_key: Option<&str>,
    raw: &Value,
    is_streaming: bool,
    providers: &[ProviderConfig],
) -> Result<(Value, Bytes), AppError> {
    let _ = is_streaming; // reserved for future streaming fallback

    let mut sorted: Vec<&ProviderConfig> = providers.iter().filter(|p| p.enabled).collect();
    sorted.sort_by_key(|p| p.priority);

    let mut last_err = AppError::ProviderError("no providers configured".into());

    for provider in &sorted {
        let result = match provider.name.as_str() {
            "anthropic" => anthropic_complete_raw(http, anthropic_key, raw).await,
            "openai" => {
                if let Some(key) = openai_key {
                    openai_complete(http, key, raw).await
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        match result {
            Ok(resp) => return Ok(resp),
            Err(AppError::ProviderError(ref msg)) => {
                // Parse status code from error message (format: "STATUS: body")
                let retryable = msg
                    .split(':')
                    .next()
                    .and_then(|s| s.trim().parse::<u16>().ok())
                    .is_some_and(is_retryable_status);

                if retryable {
                    tracing::warn!(
                        provider = %provider.name,
                        error = %msg,
                        "retryable error, trying next provider"
                    );
                    last_err = result.unwrap_err();
                    continue;
                }
                return result;
            }
            Err(e) => {
                // Network errors are retryable
                tracing::warn!(
                    provider = %provider.name,
                    error = ?e,
                    "network error, trying next provider"
                );
                last_err = e;
                continue;
            }
        }
    }

    Err(last_err)
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
    "temperature",
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

/// Forward a non-streaming request to the Anthropic API using raw JSON (no type deserialization).
/// This handles any message format Claude Code might send.
/// Strips non-API fields (context_management, etc.) before forwarding.
pub async fn anthropic_complete_raw(
    http: &reqwest::Client,
    api_key: &str,
    body: &Value,
) -> Result<(Value, Bytes), AppError> {
    let clean_body = sanitize_anthropic_request(body);

    let resp = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&clean_body)
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

/// Forward a streaming request to the Anthropic API using raw JSON.
/// Strips non-API fields before forwarding.
pub async fn anthropic_stream_raw(
    http: &reqwest::Client,
    api_key: &str,
    body: &Value,
) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, AppError> {
    let mut stream_body = sanitize_anthropic_request(body);
    stream_body["stream"] = Value::Bool(true);

    let resp = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
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

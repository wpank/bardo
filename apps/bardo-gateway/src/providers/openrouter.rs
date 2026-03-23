//! OpenRouter backend provider.
//!
//! OpenRouter exposes an OpenAI-compatible API at `openrouter.ai/api/v1` giving
//! access to 400+ models from many providers. Acts as a catch-all: any model not
//! accepted by Anthropic or OpenAI backends will land here.
//!
//! Sends optimization headers: `X-Custom-Provider-Order` to prefer native APIs,
//! `X-Fallback-Models` for automatic failover on overloaded models.

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;

use crate::error::AppError;
use crate::format;
use crate::sse;

use super::{ByteStream, Provider, ProviderResponse, UsageDetails};

const OPENROUTER_BASE: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Routes any model to OpenRouter's 400+ model catalog.
///
/// Placed last in the provider priority list — accepts everything not claimed
/// by Anthropic or OpenAI backends.
pub struct OpenRouterProvider {
    pub http: reqwest::Client,
    pub api_key: String,
}

impl OpenRouterProvider {
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self { http, api_key }
    }

    /// Common headers for all OpenRouter requests.
    fn apply_headers(&self, req: reqwest::RequestBuilder, model: &str) -> reqwest::RequestBuilder {
        let mut req = req
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .header("http-referer", "https://bardo.run")
            .header("x-title", "Bardo Gateway")
            // Prefer the model's native provider API for lower latency and full feature support.
            .header("x-custom-provider-order", "Anthropic,OpenAI,Google");

        // Set fallback models for popular choices so OpenRouter auto-routes on overload.
        if let Some(fallback) = fallback_for(model) {
            req = req.header("x-fallback-models", fallback);
        }

        req
    }

    async fn post_complete(&self, body: &Value, model: &str) -> Result<(Value, Bytes), AppError> {
        let resp = self
            .apply_headers(self.http.post(OPENROUTER_BASE), model)
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("openrouter request failed: {e}")))?;

        let status = resp.status();
        let raw_bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::ProviderError(format!("openrouter read failed: {e}")))?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&raw_bytes).into_owned();
            if status.as_u16() < 500 {
                return Err(AppError::UpstreamClientError {
                    status: status.as_u16(),
                    body: text,
                });
            }
            return Err(AppError::ProviderError(format!(
                "openrouter returned {status}: {text}"
            )));
        }

        let parsed: Value = serde_json::from_slice(&raw_bytes)
            .map_err(|e| AppError::ProviderError(format!("openrouter parse failed: {e}")))?;

        Ok((parsed, Bytes::copy_from_slice(&raw_bytes)))
    }

    async fn post_stream(&self, mut body: Value, model: &str) -> Result<ByteStream, AppError> {
        body["stream"] = Value::Bool(true);

        let resp = self
            .apply_headers(self.http.post(OPENROUTER_BASE), model)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::ProviderError(format!("openrouter stream request failed: {e}"))
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            if status.as_u16() < 500 {
                return Err(AppError::UpstreamClientError {
                    status: status.as_u16(),
                    body: err_body,
                });
            }
            return Err(AppError::ProviderError(format!(
                "openrouter stream returned {status}: {err_body}"
            )));
        }

        Ok(Box::pin(resp.bytes_stream()))
    }
}

/// Known fallback models for popular choices.
fn fallback_for(model: &str) -> Option<&'static str> {
    if model.contains("claude-sonnet") {
        Some("anthropic/claude-3.5-sonnet")
    } else if model.contains("gpt-4o") && !model.contains("mini") {
        Some("openai/gpt-4o-mini")
    } else if model.contains("gemini-2.0-flash") {
        Some("google/gemini-1.5-flash")
    } else {
        None
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn accepts(&self, _model: &str) -> bool {
        true
    }

    fn catalog(&self) -> Vec<Value> {
        vec![
            or_entry("openai/gpt-4o"),
            or_entry("openai/gpt-4o-mini"),
            or_entry("openai/o3-mini"),
            or_entry("anthropic/claude-opus-4-5"),
            or_entry("anthropic/claude-sonnet-4-5"),
            or_entry("meta-llama/llama-3.3-70b-instruct"),
            or_entry("meta-llama/llama-3.1-8b-instruct"),
            or_entry("google/gemini-2.0-flash-001"),
            or_entry("google/gemini-2.5-pro-preview-03-25"),
            or_entry("deepseek/deepseek-r1"),
            or_entry("deepseek/deepseek-chat"),
            or_entry("qwen/qwen-2.5-72b-instruct"),
            or_entry("mistralai/mistral-large"),
            or_entry("x-ai/grok-3-mini-beta"),
        ]
    }

    async fn complete(&self, body: &Value) -> Result<ProviderResponse, AppError> {
        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        let openai_req = format::anthropic_body_to_openai_request(body);
        let (parsed, _raw) = self.post_complete(&openai_req, &model).await?;

        // Extract usage before format translation discards detailed fields.
        let u = parsed.get("usage");
        let usage = UsageDetails {
            input_tokens: u
                .and_then(|v| v.get("prompt_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            output_tokens: u
                .and_then(|v| v.get("completion_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            cached_tokens: u
                .and_then(|v| v.get("prompt_tokens_details"))
                .and_then(|d| d.get("cached_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            reasoning_tokens: u
                .and_then(|v| v.get("completion_tokens_details"))
                .and_then(|d| d.get("reasoning_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            ..Default::default()
        };

        let mut anthropic_body = format::openai_body_to_anthropic_response(&parsed);
        anthropic_body["model"] = Value::String(model);

        let raw_bytes = Bytes::from(serde_json::to_vec(&anthropic_body).unwrap_or_default());
        Ok(ProviderResponse {
            body: anthropic_body,
            raw: raw_bytes,
            provider: "openrouter".into(),
            usage,
        })
    }

    async fn stream(&self, body: &Value) -> Result<ByteStream, AppError> {
        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        let openai_req = format::anthropic_body_to_openai_request(body);
        let byte_stream = self.post_stream(openai_req, &model).await?;
        Ok(sse::openai_stream_to_anthropic(byte_stream, model))
    }
}

fn or_entry(id: &str) -> Value {
    serde_json::json!({
        "id": id,
        "object": "model",
        "provider": "openrouter",
    })
}

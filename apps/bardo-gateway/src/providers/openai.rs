//! OpenAI backend provider.
//!
//! Translates Anthropic-format requests to OpenAI Chat Completions format,
//! forwards to `api.openai.com`, and translates responses back to Anthropic format.
//! Extracts `cached_tokens` and `reasoning_tokens` from OpenAI's detailed usage.

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;

use crate::error::AppError;
use crate::format;
use crate::provider as raw;
use crate::sse;

use super::{ByteStream, Provider, ProviderResponse, UsageDetails};

/// Routes `gpt-*`, `o1`, `o3`, `o4-*` models to the OpenAI Chat Completions API.
pub struct OpenAiProvider {
    pub http: reqwest::Client,
    pub api_key: String,
}

impl OpenAiProvider {
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self { http, api_key }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn accepts(&self, model: &str) -> bool {
        model.starts_with("gpt-")
            || model.starts_with("o1")
            || model.starts_with("o3")
            || model.starts_with("o4")
    }

    fn catalog(&self) -> Vec<Value> {
        vec![
            model_entry("gpt-4o"),
            model_entry("gpt-4o-mini"),
            model_entry("o1"),
            model_entry("o3-mini"),
            model_entry("o4-mini"),
        ]
    }

    async fn complete(&self, body: &Value) -> Result<ProviderResponse, AppError> {
        let openai_req = format::anthropic_body_to_openai_request(body);
        let (parsed, _raw) = raw::openai_complete(&self.http, &self.api_key, &openai_req).await?;

        // Extract detailed usage BEFORE format translation discards it.
        let u = parsed.get("usage");
        let usage = UsageDetails {
            input_tokens: u.and_then(|v| v.get("prompt_tokens")).and_then(|t| t.as_u64()).unwrap_or(0),
            output_tokens: u.and_then(|v| v.get("completion_tokens")).and_then(|t| t.as_u64()).unwrap_or(0),
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

        let model = parsed
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");

        let mut anthropic_body = format::openai_body_to_anthropic_response(&parsed);
        if let Some(req_model) = body.get("model").and_then(|m| m.as_str()) {
            anthropic_body["model"] = Value::String(req_model.to_string());
        } else {
            anthropic_body["model"] = Value::String(model.to_string());
        }

        let raw_bytes = Bytes::from(serde_json::to_vec(&anthropic_body).unwrap_or_default());
        Ok(ProviderResponse {
            body: anthropic_body,
            raw: raw_bytes,
            provider: "openai".into(),
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
        let byte_stream = raw::openai_stream(&self.http, &self.api_key, &openai_req).await?;
        Ok(sse::openai_stream_to_anthropic(byte_stream, model))
    }
}

fn model_entry(id: &str) -> Value {
    serde_json::json!({
        "id": id,
        "object": "model",
        "provider": "openai",
    })
}

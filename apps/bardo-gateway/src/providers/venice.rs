//! Venice backend provider.
//!
//! Venice exposes an OpenAI-compatible API at `api.venice.ai/api/v1` with
//! zero-retention guarantees. All API requests are private by default --
//! Venice retains nothing.
//!
//! Injects `venice_parameters` into the request body and sends
//! `X-Venice-Version` header for API versioning.

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;

use crate::error::AppError;
use crate::format;
use crate::sse;

use super::{ByteStream, Provider, ProviderResponse, UsageDetails};

const VENICE_BASE: &str = "https://api.venice.ai/api/v1/chat/completions";
const VENICE_VERSION: &str = "2024-12-01";

/// Known Venice model identifiers.
const VENICE_MODELS: &[&str] = &[
    "llama-3.3-70b",
    "deepseek-ai-DeepSeek-R1",
    "zai-org-glm-4.7",
    "qwen-2.5-vl-72b",
];

/// Routes Venice-hosted models to the Venice API (zero-retention inference).
///
/// Placed after Anthropic in the provider priority list. Claims models
/// with a `venice/` prefix or matching known Venice model names.
pub struct VeniceProvider {
    pub http: reqwest::Client,
    pub api_key: String,
}

impl VeniceProvider {
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self { http, api_key }
    }

    fn apply_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .header("x-venice-version", VENICE_VERSION)
    }

    /// Inject Venice-specific parameters into the OpenAI-format request body.
    fn inject_venice_params(body: &mut Value) {
        body["venice_parameters"] = serde_json::json!({
            "strip_thinking_response": false,
            "enable_web_search": "off",
        });
    }

    /// Strip `venice/` prefix from model name if present.
    fn normalize_model(body: &mut Value) {
        if let Some(m) = body.get("model").and_then(|v| v.as_str()) {
            if let Some(stripped) = m.strip_prefix("venice/") {
                body["model"] = Value::String(stripped.to_string());
            }
        }
    }

    async fn post_complete(&self, body: &Value) -> Result<(Value, Bytes), AppError> {
        let resp = self
            .apply_headers(self.http.post(VENICE_BASE))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("venice request failed: {e}")))?;

        let status = resp.status();
        let raw_bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::ProviderError(format!("venice read failed: {e}")))?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&raw_bytes).into_owned();
            if status.as_u16() < 500 {
                return Err(AppError::UpstreamClientError {
                    status: status.as_u16(),
                    body: text,
                });
            }
            return Err(AppError::ProviderError(format!(
                "venice returned {status}: {text}"
            )));
        }

        let parsed: Value = serde_json::from_slice(&raw_bytes)
            .map_err(|e| AppError::ProviderError(format!("venice parse failed: {e}")))?;

        Ok((parsed, Bytes::copy_from_slice(&raw_bytes)))
    }

    async fn post_stream(&self, mut body: Value) -> Result<ByteStream, AppError> {
        body["stream"] = Value::Bool(true);

        let resp = self
            .apply_headers(self.http.post(VENICE_BASE))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("venice stream request failed: {e}")))?;

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
                "venice stream returned {status}: {err_body}"
            )));
        }

        Ok(Box::pin(resp.bytes_stream()))
    }
}

#[async_trait]
impl Provider for VeniceProvider {
    fn name(&self) -> &str {
        "venice"
    }

    fn accepts(&self, model: &str) -> bool {
        model.starts_with("venice/") || VENICE_MODELS.iter().any(|&m| model == m)
    }

    fn catalog(&self) -> Vec<Value> {
        VENICE_MODELS
            .iter()
            .map(|&id| {
                serde_json::json!({
                    "id": id,
                    "object": "model",
                    "provider": "venice",
                })
            })
            .collect()
    }

    async fn complete(&self, body: &Value) -> Result<ProviderResponse, AppError> {
        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut openai_req = format::anthropic_body_to_openai_request(body);
        Self::normalize_model(&mut openai_req);
        Self::inject_venice_params(&mut openai_req);

        let (parsed, _raw) = self.post_complete(&openai_req).await?;

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
            ..Default::default()
        };

        let mut anthropic_body = format::openai_body_to_anthropic_response(&parsed);
        anthropic_body["model"] = Value::String(model);

        let raw_bytes = Bytes::from(serde_json::to_vec(&anthropic_body).unwrap_or_default());
        Ok(ProviderResponse {
            body: anthropic_body,
            raw: raw_bytes,
            provider: "venice".into(),
            usage,
        })
    }

    async fn stream(&self, body: &Value) -> Result<ByteStream, AppError> {
        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut openai_req = format::anthropic_body_to_openai_request(body);
        Self::normalize_model(&mut openai_req);
        Self::inject_venice_params(&mut openai_req);

        let byte_stream = self.post_stream(openai_req).await?;
        Ok(sse::openai_stream_to_anthropic(byte_stream, model))
    }
}

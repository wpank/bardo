//! Bankr backend provider.
//!
//! Bankr exposes an OpenAI-compatible inference API with integrated credit
//! management and self-funding capabilities. The Bankr provider connects
//! vault management revenue to inference costs through a "metabolic loop."
//!
//! Models are routed via `bankr/` prefix: `bankr/gemini-2.5-flash`,
//! `bankr/claude-sonnet-4`, `bankr/claude-opus-4-6`.

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;

use crate::error::AppError;
use crate::format;
use crate::sse;

use super::{ByteStream, Provider, ProviderResponse, UsageDetails};

const BANKR_DEFAULT_BASE: &str = "https://api.bankr.ai/v1/chat/completions";

/// Known Bankr model aliases (shown with bankr/ prefix in catalog).
const BANKR_CATALOG_MODELS: &[&str] = &[
    "bankr/gemini-2.5-flash",
    "bankr/claude-sonnet-4",
    "bankr/claude-opus-4-6",
];

/// Routes `bankr/*` models to the Bankr inference API.
///
/// Placed after OpenAI in the provider priority list. Only claims models
/// with a `bankr/` prefix to avoid conflicts with other providers.
pub struct BankrProvider {
    pub http: reqwest::Client,
    pub api_key: String,
    pub base_url: String,
}

impl BankrProvider {
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self {
            http,
            api_key,
            base_url: BANKR_DEFAULT_BASE.to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = if base_url.ends_with("/chat/completions") {
            base_url
        } else {
            format!("{}/chat/completions", base_url.trim_end_matches('/'))
        };
        self
    }

    fn apply_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
    }

    /// Strip `bankr/` prefix from model name.
    fn strip_prefix(model: &str) -> &str {
        model.strip_prefix("bankr/").unwrap_or(model)
    }

    async fn post_complete(&self, body: &Value) -> Result<(Value, Bytes), AppError> {
        let resp = self
            .apply_headers(self.http.post(&self.base_url))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("bankr request failed: {e}")))?;

        let status = resp.status();
        let raw_bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::ProviderError(format!("bankr read failed: {e}")))?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&raw_bytes).into_owned();
            if status.as_u16() < 500 {
                return Err(AppError::UpstreamClientError {
                    status: status.as_u16(),
                    body: text,
                });
            }
            return Err(AppError::ProviderError(format!(
                "bankr returned {status}: {text}"
            )));
        }

        let parsed: Value = serde_json::from_slice(&raw_bytes)
            .map_err(|e| AppError::ProviderError(format!("bankr parse failed: {e}")))?;

        Ok((parsed, Bytes::copy_from_slice(&raw_bytes)))
    }

    async fn post_stream(&self, mut body: Value) -> Result<ByteStream, AppError> {
        body["stream"] = Value::Bool(true);

        let resp = self
            .apply_headers(self.http.post(&self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("bankr stream request failed: {e}")))?;

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
                "bankr stream returned {status}: {err_body}"
            )));
        }

        Ok(Box::pin(resp.bytes_stream()))
    }
}

#[async_trait]
impl Provider for BankrProvider {
    fn name(&self) -> &str {
        "bankr"
    }

    fn accepts(&self, model: &str) -> bool {
        model.starts_with("bankr/")
    }

    fn catalog(&self) -> Vec<Value> {
        BANKR_CATALOG_MODELS
            .iter()
            .map(|&id| {
                serde_json::json!({
                    "id": id,
                    "object": "model",
                    "provider": "bankr",
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
        // Strip bankr/ prefix so Bankr API gets the raw model name.
        if let Some(m) = openai_req.get("model").and_then(|v| v.as_str()) {
            let stripped = Self::strip_prefix(m);
            openai_req["model"] = Value::String(stripped.to_string());
        }

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
            provider: "bankr".into(),
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
        if let Some(m) = openai_req.get("model").and_then(|v| v.as_str()) {
            let stripped = Self::strip_prefix(m);
            openai_req["model"] = Value::String(stripped.to_string());
        }

        let byte_stream = self.post_stream(openai_req).await?;
        Ok(sse::openai_stream_to_anthropic(byte_stream, model))
    }
}

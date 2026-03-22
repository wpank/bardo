//! Anthropic backend provider.

use async_trait::async_trait;
use serde_json::Value;

use crate::error::AppError;
use crate::provider as raw;

use super::{ByteStream, Provider, ProviderResponse, UsageDetails, get_u64};

/// Routes `claude-*` models to the Anthropic Messages API.
///
/// Holds one or more API keys. On 429 (rate limit), the retry logic in
/// `provider::anthropic_request_with_retry` rotates to the next key before
/// backing off, effectively multiplying your rate limit budget.
pub struct AnthropicProvider {
    pub http: reqwest::Client,
    pub api_keys: Vec<String>,
}

impl AnthropicProvider {
    pub fn new(http: reqwest::Client, api_keys: Vec<String>) -> Self {
        assert!(!api_keys.is_empty(), "at least one Anthropic API key required");
        Self { http, api_keys }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn accepts(&self, model: &str) -> bool {
        model.contains("claude")
    }

    fn catalog(&self) -> Vec<Value> {
        vec![
            model_entry("claude-opus-4-6", "anthropic", "T2"),
            model_entry("claude-sonnet-4-6", "anthropic", "T2"),
            model_entry("claude-haiku-4-5-20251001", "anthropic", "T1"),
        ]
    }

    async fn complete(&self, body: &Value) -> Result<ProviderResponse, AppError> {
        let (parsed, raw_bytes) =
            raw::anthropic_complete_raw(&self.http, &self.api_keys, body).await?;

        let u = parsed.get("usage");
        let usage = UsageDetails {
            input_tokens: get_u64(u, "input_tokens"),
            output_tokens: get_u64(u, "output_tokens"),
            cache_read_input_tokens: get_u64(u, "cache_read_input_tokens"),
            cache_creation_input_tokens: get_u64(u, "cache_creation_input_tokens"),
            thinking_tokens: get_u64(u, "thinking_tokens"),
            ..Default::default()
        };

        Ok(ProviderResponse {
            body: parsed,
            raw: raw_bytes,
            provider: "anthropic".into(),
            usage,
        })
    }

    async fn stream(&self, body: &Value) -> Result<ByteStream, AppError> {
        raw::anthropic_stream_raw(&self.http, &self.api_keys, body).await
    }
}

fn model_entry(id: &str, provider: &str, tier: &str) -> Value {
    serde_json::json!({
        "id": id,
        "object": "model",
        "provider": provider,
        "tier": tier,
    })
}

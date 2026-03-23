//! Provider trait and shared types for the gateway's pluggable backend system.
//!
//! Each provider implements [`Provider`], which normalizes responses to **Anthropic format**
//! as the internal canonical representation. The handler is then format-agnostic: it
//! calls `provider.complete()` or `provider.stream()` and always gets back Anthropic-format
//! bytes, regardless of what the underlying API actually speaks.
//!
//! Resolution is first-match by priority:
//! 1. `AnthropicProvider`  — `claude-*` models
//! 2. `VeniceProvider`     — `venice/*`, Venice-hosted models (zero-retention)
//! 3. `OpenAiProvider`     — `gpt-*`, `o1`, `o3`, `o4-*` models
//! 4. `BankrProvider`      — `bankr/*` models (self-funding inference)
//! 5. `OpenRouterProvider` — catch-all for everything else

use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use serde_json::Value;

use serde::Serialize;

use crate::error::AppError;

pub mod anthropic;
pub mod bankr;
pub mod openai;
pub mod openrouter;
pub mod venice;

pub use anthropic::AnthropicProvider;
pub use bankr::BankrProvider;
pub use openai::OpenAiProvider;
pub use openrouter::OpenRouterProvider;
pub use venice::VeniceProvider;

/// Pinned byte stream returned by streaming provider calls.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// Provider-specific token usage details, preserved through format translation.
///
/// Each provider fills the fields relevant to its API. The handler uses this for
/// accurate cost computation instead of re-extracting from the translated body.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UsageDetails {
    /// Non-cached input tokens (full price).
    pub input_tokens: u64,
    /// Output tokens (full price, excluding reasoning).
    pub output_tokens: u64,
    // ── Anthropic-specific ──
    /// Tokens read from Anthropic's server-side KV cache (10% of input price).
    pub cache_read_input_tokens: u64,
    /// Tokens written to Anthropic's cache on this request (125% of input price).
    pub cache_creation_input_tokens: u64,
    /// Extended thinking tokens (charged at output price).
    pub thinking_tokens: u64,
    // ── OpenAI-specific ──
    /// Tokens served from OpenAI's automatic prefix cache (50% of input price).
    /// This is a subset of input_tokens (unlike Anthropic where they're separate).
    pub cached_tokens: u64,
    /// Reasoning tokens from o-series models (charged at reasoning rate).
    /// This is a subset of output_tokens.
    pub reasoning_tokens: u64,
}

/// Helper: extract a u64 from a nested JSON path.
pub fn get_u64(obj: Option<&Value>, key: &str) -> u64 {
    obj.and_then(|o| o.get(key))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

/// Normalized response from a provider — always in Anthropic format.
pub struct ProviderResponse {
    /// Parsed Anthropic-format response body.
    pub body: Value,
    /// Serialized bytes (stored in the blake3 cache).
    pub raw: Bytes,
    /// Provider name, used for logging and `x-mori-provider` header.
    pub provider: String,
    /// Detailed token usage, filled by the provider from the raw response.
    pub usage: UsageDetails,
}

/// A backend inference provider.
///
/// Implementations translate between their native format and Anthropic format
/// so the handler layer is fully format-agnostic.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Short name used in logs and response headers (e.g. `"anthropic"`, `"openrouter"`).
    fn name(&self) -> &str;

    /// Whether this provider handles the given model identifier.
    ///
    /// Resolution is first-match: providers are checked in priority order and
    /// the first returning `true` is selected.
    fn accepts(&self, model: &str) -> bool;

    /// Static model catalog for `GET /v1/models`.
    fn catalog(&self) -> Vec<Value>;

    /// Non-streaming inference. Returns an Anthropic-format response.
    async fn complete(&self, body: &Value) -> Result<ProviderResponse, AppError>;

    /// Streaming inference. Returns an Anthropic-format SSE byte stream.
    async fn stream(&self, body: &Value) -> Result<ByteStream, AppError>;
}

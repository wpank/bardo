//! Core inference types: messages, requests, responses, streaming chunks.

use serde::{Deserialize, Serialize};

use crate::error::InferenceError;

// ── Message types ──────────────────────────────────────────────────

/// Role in a conversation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message.
    User,
    /// Assistant response.
    Assistant,
}

/// A content block within a message.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content.
    Text {
        /// The text.
        text: String,
    },
    /// Tool use request from the model.
    ToolUse {
        /// Tool call ID.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input as JSON value.
        input: serde_json::Value,
    },
    /// Tool result from the caller.
    ToolResult {
        /// Matching tool_use ID.
        tool_use_id: String,
        /// Tool output.
        content: String,
    },
}

/// A conversation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Who sent this message.
    pub role: Role,
    /// Content blocks.
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a simple text message.
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }
}

// ── Request / Response ─────────────────────────────────────────────

/// An inference request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model identifier (e.g., "claude-sonnet-4-20250514").
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Optional system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Maximum output tokens.
    pub max_tokens: u32,
    /// Sampling temperature.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Whether to stream the response.
    #[serde(default)]
    pub stream: bool,
    /// Tool definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    /// Request metadata (not sent to provider, used internally).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

fn default_temperature() -> f32 {
    1.0
}

impl InferenceRequest {
    /// Validate the request fields.
    pub fn validate(&self) -> Result<(), InferenceError> {
        if self.model.is_empty() {
            return Err(InferenceError::Validation("model must be non-empty".into()));
        }
        if self.max_tokens == 0 || self.max_tokens > 128_000 {
            return Err(InferenceError::Validation(format!(
                "max_tokens must be in [1, 128000], got {}",
                self.max_tokens
            )));
        }
        if self.temperature < 0.0 || self.temperature > 2.0 || self.temperature.is_nan() {
            return Err(InferenceError::Validation(format!(
                "temperature must be in [0.0, 2.0], got {}",
                self.temperature
            )));
        }
        Ok(())
    }
}

/// Token usage from a response.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens consumed.
    pub input_tokens: u64,
    /// Output tokens generated.
    pub output_tokens: u64,
    /// Tokens read from cache (Anthropic prompt caching).
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    /// Tokens written to cache.
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

impl TokenUsage {
    /// Compute USD cost based on model pricing.
    pub fn cost_usd(&self, input_price_per_m: f64, output_price_per_m: f64) -> f64 {
        let cached_discount = 0.1; // 90% discount on cached reads
        let regular_input = self.input_tokens.saturating_sub(self.cache_read_input_tokens);
        let cached_input = self.cache_read_input_tokens;

        let input_cost =
            (regular_input as f64 * input_price_per_m / 1_000_000.0) +
            (cached_input as f64 * input_price_per_m * cached_discount / 1_000_000.0);
        let output_cost = self.output_tokens as f64 * output_price_per_m / 1_000_000.0;

        input_cost + output_cost
    }
}

/// Why the model stopped generating.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of response.
    EndTurn,
    /// Hit max_tokens limit.
    MaxTokens,
    /// Model wants to use a tool.
    ToolUse,
    /// Hit a stop sequence.
    StopSequence,
}

/// A complete inference response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Response ID.
    pub id: String,
    /// Content blocks.
    pub content: Vec<ContentBlock>,
    /// Model that generated the response.
    pub model: String,
    /// Why generation stopped.
    pub stop_reason: Option<StopReason>,
    /// Token usage.
    pub usage: TokenUsage,
}

/// A streaming chunk from SSE.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceChunk {
    /// Event type (e.g., "content_block_delta", "message_stop").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Index in the content block array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    /// Delta content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<ChunkDelta>,
    /// Usage (sent on message_stop).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// Delta within a streaming chunk.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkDelta {
    /// Text delta.
    TextDelta {
        /// The text fragment.
        text: String,
    },
    /// Input JSON delta (tool use).
    InputJsonDelta {
        /// Partial JSON.
        partial_json: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_empty_model() {
        let req = InferenceRequest {
            model: String::new(),
            messages: vec![],
            system: None,
            max_tokens: 100,
            temperature: 1.0,
            stream: false,
            tools: None,
            metadata: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn validate_max_tokens_bounds() {
        let mut req = InferenceRequest {
            model: "test".into(),
            messages: vec![],
            system: None,
            max_tokens: 0,
            temperature: 1.0,
            stream: false,
            tools: None,
            metadata: None,
        };
        assert!(req.validate().is_err());
        req.max_tokens = 200_000;
        assert!(req.validate().is_err());
        req.max_tokens = 4096;
        assert!(req.validate().is_ok());
    }

    #[test]
    fn validate_temperature_bounds() {
        let mut req = InferenceRequest {
            model: "test".into(),
            messages: vec![],
            system: None,
            max_tokens: 100,
            temperature: -0.1,
            stream: false,
            tools: None,
            metadata: None,
        };
        assert!(req.validate().is_err());
        req.temperature = 2.1;
        assert!(req.validate().is_err());
        req.temperature = f32::NAN;
        assert!(req.validate().is_err());
        req.temperature = 0.7;
        assert!(req.validate().is_ok());
    }

    #[test]
    fn token_usage_cost() {
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
        };
        // Sonnet: $3/1M in, $15/1M out
        let cost = usage.cost_usd(3.0, 15.0);
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn token_usage_cost_with_cache() {
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_input_tokens: 800,
            cache_creation_input_tokens: 0,
        };
        // 200 regular input + 800 cached at 10% rate
        let cost = usage.cost_usd(3.0, 15.0);
        let expected = (200.0 * 3.0 / 1_000_000.0)
            + (800.0 * 3.0 * 0.1 / 1_000_000.0)
            + (500.0 * 15.0 / 1_000_000.0);
        assert!((cost - expected).abs() < 0.000001);
    }

    #[test]
    fn message_roundtrip() {
        let msg = Message::text(Role::User, "hello");
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, Role::User);
        assert_eq!(parsed.content.len(), 1);
    }

    #[test]
    fn stop_reason_serde() {
        let json = r#""end_turn""#;
        let parsed: StopReason = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, StopReason::EndTurn);
    }
}

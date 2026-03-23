# bardo-inference

Wire types for the Anthropic/OpenAI-compatible inference protocol. No Golem platform dependencies, no HTTP client, no model routing. Just the types that both sides of the inference boundary share.

`bardo-gateway` serializes these to JSON and sends them to Anthropic. `golem-inference` deserializes responses from `bardo-gateway`. Both crates depend on this one for the shared vocabulary.

## Exports

```rust
pub use error::{ErrorDetail, ErrorPayload, InferenceError};
pub use types::{
    ChunkDelta, ContentBlock, InferenceChunk, InferenceRequest, InferenceResponse,
    Message, Role, StopReason, TokenUsage,
};
```

## Request and Response

`InferenceRequest` is what you send:

```rust
pub struct InferenceRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,   // default 1.0
    pub stream: bool,
    pub tools: Option<Vec<serde_json::Value>>,
    pub metadata: Option<serde_json::Value>,
}
```

Validation via `request.validate()` rejects empty model strings, `max_tokens` outside `[1, 128000]`, temperature outside `[0.0, 2.0]`, and NaN temperature.

`InferenceResponse` is what you get back:

```rust
pub struct InferenceResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<StopReason>,
    pub usage: TokenUsage,
}
```

## Messages and Content

`Message` holds a `Role` and a `Vec<ContentBlock>`:

```rust
pub enum Role { User, Assistant }

pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String },
}
```

Convenience constructor for text messages:

```rust
Message::text(Role::User, "what is the current ETH gas price?")
```

## Streaming

`InferenceChunk` carries one SSE event:

```rust
pub struct InferenceChunk {
    pub event_type: String,        // "content_block_delta", "message_stop", etc.
    pub index: Option<u32>,
    pub delta: Option<ChunkDelta>,
    pub usage: Option<TokenUsage>, // only on message_stop
}

pub enum ChunkDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}
```

## Token Usage and Cost

`TokenUsage` carries Anthropic prompt caching fields:

```rust
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
}
```

The `cost_usd` method applies a 90% discount on cached reads:

```rust
let cost = usage.cost_usd(3.0, 15.0); // Sonnet: $3/M in, $15/M out
```

## Stop Reasons

```rust
pub enum StopReason {
    EndTurn,
    MaxTokens,
    ToolUse,
    StopSequence,
}
```

Serializes as `"end_turn"`, `"max_tokens"`, `"tool_use"`, `"stop_sequence"` to match the Anthropic API.

## Errors

`InferenceError` variants: `Validation(String)`, `Provider(String)`, `Timeout`, `Unauthorized`.

`ErrorPayload` and `ErrorDetail` serialize to `{"error": {"type": "...", "message": "...", "code": "..."}}` for API-compatible error responses.

## Usage

```toml
[dependencies]
bardo-inference = { path = "../../crates/bardo-inference" }
```

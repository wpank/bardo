# bardo-inference

Wire types for the Anthropic/OpenAI-compatible inference protocol. No HTTP client, no model routing, no platform dependencies. Just the types that both sides of an inference boundary share.

## Install

```toml
[dependencies]
bardo-inference = { git = "https://github.com/uniswap/bardo", path = "crates/bardo-inference" }
```

External deps: `serde`, `serde_json`, `thiserror`, `uuid`. No workspace dependencies.

If you're building an inference proxy, an LLM client, or anything that speaks the Anthropic/OpenAI wire format, these types save you from redefining them.

## Request and response

### Sending a request

```rust
use bardo_inference::{InferenceRequest, Message, Role};

let request = InferenceRequest {
    model: "claude-sonnet-4-6".into(),
    messages: vec![Message::text(Role::User, "what is the current ETH gas price?")],
    system: Some("You are a blockchain analyst.".into()),
    max_tokens: 1024,
    temperature: 0.7,
    stream: false,
    tools: None,
    metadata: None,
};

// Validate before sending
request.validate()?;

// Serialize to JSON — ready for any HTTP client
let body = serde_json::to_string(&request)?;
```

Validation rejects: empty model strings, `max_tokens` outside `[1, 128_000]`, temperature outside `[0.0, 2.0]`, and NaN temperature.

### Reading a response

```rust
use bardo_inference::{InferenceResponse, ContentBlock, StopReason};

let response: InferenceResponse = serde_json::from_str(&body)?;

for block in &response.content {
    match block {
        ContentBlock::Text { text } => println!("{text}"),
        ContentBlock::ToolUse { id, name, input } => {
            println!("tool call: {name}({input})");
        }
        ContentBlock::ToolResult { tool_use_id, content } => {
            println!("tool result for {tool_use_id}: {content}");
        }
    }
}

// Check why the model stopped
match response.stop_reason {
    Some(StopReason::EndTurn) => { /* normal completion */ }
    Some(StopReason::ToolUse) => { /* needs tool execution */ }
    Some(StopReason::MaxTokens) => { /* hit the limit */ }
    _ => {}
}
```

## Messages and content blocks

```rust
use bardo_inference::{Message, Role, ContentBlock};

// Convenience constructor for text messages
let msg = Message::text(Role::User, "explain the auth middleware");

// Full control with multiple content blocks
let msg = Message {
    role: Role::Assistant,
    content: vec![
        ContentBlock::ToolUse {
            id: "call_01".into(),
            name: "search_code".into(),
            input: serde_json::json!({"query": "auth"}),
        },
    ],
};
```

Content blocks use internally tagged serialization (`"type"` field discriminator):

```json
{"type": "text", "text": "hello"}
{"type": "tool_use", "id": "call_01", "name": "search_code", "input": {...}}
{"type": "tool_result", "tool_use_id": "call_01", "content": "found 3 results"}
```

## Streaming

`InferenceChunk` carries one SSE event from a streaming response:

```rust
use bardo_inference::{InferenceChunk, ChunkDelta};

let chunk: InferenceChunk = serde_json::from_str(&event_data)?;

match chunk.delta {
    Some(ChunkDelta::TextDelta { text }) => print!("{text}"),
    Some(ChunkDelta::InputJsonDelta { partial_json }) => {
        // Accumulate partial JSON for tool_use streaming
        buffer.push_str(&partial_json);
    }
    None => {}
}

// Final chunk carries token usage
if let Some(usage) = chunk.usage {
    println!("tokens: {}in / {}out", usage.input_tokens, usage.output_tokens);
}
```

The `event_type` field maps to Anthropic SSE event names: `"content_block_delta"`, `"message_stop"`, etc.

## Token usage and cost

`TokenUsage` includes Anthropic prompt caching fields:

```rust
use bardo_inference::TokenUsage;

let usage = TokenUsage {
    input_tokens: 45000,
    output_tokens: 1200,
    cache_read_input_tokens: 30000,    // 90% discount
    cache_creation_input_tokens: 5000, // 25% surcharge (investment)
};

// Compute cost with per-million-token pricing
let cost = usage.cost_usd(3.0, 15.0); // Sonnet: $3/M input, $15/M output
```

The `cost_usd` method applies a 90% discount on `cache_read_input_tokens`. Cache creation tokens are charged at full input rate (the 25% surcharge is handled by the gateway layer, not here).

Both cache fields default to 0 during deserialization, so responses from providers that don't support prompt caching work without modification.

## Serde details

The types are designed for wire compatibility with the Anthropic Messages API:

| Type | Serialization | Notes |
|------|--------------|-------|
| `Role` | `"user"`, `"assistant"` | lowercase rename |
| `StopReason` | `"end_turn"`, `"max_tokens"`, `"tool_use"`, `"stop_sequence"` | snake_case rename |
| `ContentBlock` | Tagged by `"type"` field | `"text"`, `"tool_use"`, `"tool_result"` |
| `ChunkDelta` | Tagged by `"type"` field | `"text_delta"`, `"input_json_delta"` |
| `InferenceChunk.event_type` | JSON field `"type"` | Renamed from Rust field name |

Fields that skip serialization when `None`: `system`, `tools`, `metadata` on requests; `index`, `delta`, `usage` on chunks; `code` on error details.

Default values on deserialization: `temperature` defaults to 1.0, `stream` defaults to false, cache token fields default to 0.

## Errors

```rust
use bardo_inference::{InferenceError, ErrorPayload};

// Create an error
let err = InferenceError::Validation("model cannot be empty".into());

// Convert to API-compatible JSON payload
let payload = ErrorPayload::from(&err);
let json = serde_json::to_string(&payload)?;
// -> {"error":{"type":"validation_error","message":"model cannot be empty"}}
```

| Variant | Error type | Use case |
|---------|-----------|----------|
| `Validation(String)` | `"validation_error"` | Bad request parameters |
| `Provider(String)` | `"provider_error"` | Upstream API failure |
| `Timeout` | `"timeout"` | Request timed out |
| `Unauthorized` | `"unauthorized"` | Missing or invalid API key |
| `T0Suppressed` | `"t0_suppressed"` | T0 tier blocks inference (no LLM call needed) |
| `Internal(String)` | `"internal_error"` | System-level failure |

## Use cases

- **Inference proxies** — `bardo-gateway` uses these types as its internal representation, translating between Anthropic and OpenAI wire formats at the edges
- **LLM client libraries** — type-safe request construction with validation, avoiding runtime serialization errors
- **Agent frameworks** — shared types between orchestrator and agent processes, with tool-use content blocks for structured tool calling
- **Cost tracking** — `TokenUsage::cost_usd()` gives you per-request cost computation with cache discount awareness

## Architecture

```
src/
├── lib.rs    # re-exports all public types
├── types.rs  # InferenceRequest, InferenceResponse, Message, Role, ContentBlock,
│             # InferenceChunk, ChunkDelta, TokenUsage, StopReason
└── error.rs  # InferenceError, ErrorPayload, ErrorDetail
```

## License

MIT/Apache-2.0

# golem-inference

Tier routing, inference types, and the HTTP client that talks to `bardo-gateway`. This is the client side of inference — `bardo-gateway` is the server side. No Anthropic API calls happen here directly.

## Exports

```rust
pub use client::{GatewayClient, InferenceClient};
pub use error::{ErrorPayload, InferenceError};
pub use router::TierRouter;
pub use types::{
    ChunkDelta, ContentBlock, InferenceChunk, InferenceMeta, InferenceRequest,
    InferenceResponse, Message, Role, StopReason, TokenUsage,
};
```

The `types` module re-exports from `bardo-inference`, so you get the same wire types either way.

## TierRouter

Zero-state unit struct. All model selection logic is in one place.

```rust
TierRouter::select_model(tier: CognitiveTier, vitality: f32) -> Option<&'static str>
```

| Tier | Vitality | Model                   |
|------|----------|-------------------------|
| T0   | any      | `None` (suppressed)     |
| T1   | any      | `"claude-haiku-4-5"`    |
| T2   | ≥ 0.3    | `"claude-opus-4-6"`     |
| T2   | < 0.3    | `"claude-sonnet-4"`     |

The transition at 0.3 is sharp. Vitality exactly 0.3 routes to Opus. Vitality 0.2999 routes to Sonnet.

`CognitiveTier` is a re-export of `golem_core::CognitiveTier`, which is itself `bardo_primitives::InferenceTier`. The name difference is intentional — tier routing at the gateway boundary uses `CognitiveTier`.

## GatewayClient

Implements `InferenceClient` by posting to a `bardo-gateway` instance over HTTP.

```rust
let client = GatewayClient::new("http://localhost:4000", api_key);

// Non-streaming
let response: InferenceResponse = client.complete(&request, &meta).await?;

// Streaming
let mut stream = client.stream(&request, &meta).await?;
while let Some(chunk) = stream.next().await {
    let chunk: InferenceChunk = chunk?;
    // chunk.delta carries ChunkDelta::TextDelta { text } or ChunkDelta::InputJsonDelta { partial_json }
}
```

From environment variables:

```rust
// BARDO_GATEWAY_URL (default: http://127.0.0.1:4000)
// BARDO_GATEWAY_API_KEY (required)
let client = GatewayClient::from_env()?;
```

Requests hit `POST /v1/messages`. The client attaches four headers per request: `X-Api-Key`, `X-Golem-Id`, `X-Tier`, `X-Vitality`, `X-Request-Id`. These are how the gateway attributes costs and enforces tier policies.

## InferenceClient Trait

```rust
#[async_trait]
pub trait InferenceClient: Send + Sync {
    async fn complete(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<InferenceResponse, InferenceError>;

    async fn stream(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<InferenceChunk, InferenceError>> + Send>>, InferenceError>;
}
```

## InferenceMeta

Every request carries metadata for cost attribution and audit:

```rust
pub struct InferenceMeta {
    pub golem_id: GolemId,
    pub tier: CognitiveTier,
    pub vitality: f32,
    pub request_id: Uuid,  // v4, fresh per request
}
```

Create a fresh `InferenceMeta` for each call. Do not reuse `request_id` across calls.

## SSE Streaming

The `sse` module parses the raw byte stream from the gateway into `InferenceChunk` values. `GatewayClient::stream` sets `request.stream = true` automatically before sending. You do not need to set it yourself.

## Gateway Caching

When routing through `bardo-gateway`, inference benefits from three cache layers. The hash cache matches exact request payloads and returns in sub-millisecond. The semantic cache uses embedding similarity to catch near-duplicate requests — different wording, same intent — and returns in 5-20ms. The prompt prefix cache is provider-native: Anthropic offers a 90% token discount on cached prefix tokens, which matters at high request volume.

These layers combine with context engineering (smaller prompts reduce both latency and cost) and multi-provider routing. The gateway can route across Anthropic, OpenAI, Venice, BlockRun, OpenRouter, and local models depending on tier, vitality, and availability. Combined, the total inference cost reduction across the cache and routing stack runs 40-85% relative to uncached single-provider calls.

## Usage

```toml
[dependencies]
golem-inference = { path = "../../crates/golem-inference" }
```

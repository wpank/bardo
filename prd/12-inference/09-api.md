# 09 -- API reference [SPEC]

**Unified endpoint catalog, request format, response headers, 402 response, provider management**

> Related: [00-overview.md](00-overview.md) (gateway architecture and x402 payment flows), [05-sessions.md](05-sessions.md) (checkpoint/resume and scratchpad working memory), [08-observability.md](08-observability.md) (per-agent cost attribution and OTEL traces), [11-privacy-trust.md](11-privacy-trust.md) (cryptographic audit trail, signing, and provenance endpoints)

---

> **Reader orientation:** This document is the API reference for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and catalogs all 33 HTTP endpoints exposed by the Axum-based gateway, including inference, sessions, memory, analytics, provider management, and identity linking. The key concept is that the gateway accepts both OpenAI and Anthropic API formats natively, with x402 (micropayment protocol for HTTP-native USDC payments on Base) or prepaid API key authentication. For term definitions, see `prd2/shared/glossary.md`.

## Endpoint catalog

### Inference (5)

| Method | Path                   | Purpose                                       |
| ------ | ---------------------- | --------------------------------------------- |
| `POST` | `/v1/chat/completions` | Chat completion (OpenAI format)               |
| `POST` | `/v1/messages`         | Chat completion (Anthropic Messages format)   |
| `POST` | `/v1/completions`      | Chat completion (auto-detect format)          |
| `POST` | `/v1/embeddings`       | Text embedding                                |
| `GET`  | `/v1/models`           | List available models (from BlockRun catalog) |

### Context engineering (4)

| Method | Path                           | Purpose                      |
| ------ | ------------------------------ | ---------------------------- |
| `PUT`  | `/v1/collections/{name}`       | Create/update RAG collection |
| `POST` | `/v1/collections/{name}/query` | Query RAG collection         |
| `POST` | `/v1/memory`                   | Store agent memory           |
| `POST` | `/v1/memory/search`            | Search agent memories        |

### Tool registry (3)

| Method   | Path                 | Purpose                         |
| -------- | -------------------- | ------------------------------- |
| `PUT`    | `/v1/tools/{toolId}` | Register/update tool definition |
| `GET`    | `/v1/tools/search`   | Search tools by capability      |
| `DELETE` | `/v1/tools/{toolId}` | Remove tool registration        |

### Session management (7)

| Method  | Path                            | Purpose                                     |
| ------- | ------------------------------- | ------------------------------------------- |
| `POST`  | `/v1/sessions`                  | Create session                              |
| `POST`  | `/v1/sessions/{id}/compact`     | Trigger compaction                          |
| `POST`  | `/v1/sessions/{id}/handoff`     | Trigger handoff (new session with briefing) |
| `POST`  | `/v1/sessions/{id}/checkpoint`  | Create checkpoint                           |
| `POST`  | `/v1/sessions/{id}/resume`      | Resume from checkpoint                      |
| `POST`  | `/v1/sessions/{parentId}/spawn` | Spawn sub-agent                             |
| `PATCH` | `/v1/sessions/{id}/scratchpad`  | Update working memory                       |

### Templates (2)

| Method | Path                   | Purpose                          |
| ------ | ---------------------- | -------------------------------- |
| `PUT`  | `/v1/templates/{name}` | Register/update prompt template  |
| `GET`  | `/v1/templates/{name}` | Get template by name and version |

### Audit and provenance (3)

| Method | Path                             | Purpose                                                  |
| ------ | -------------------------------- | -------------------------------------------------------- |
| `GET`  | `/v1/audit/events/{eventId}`     | Get signed audit event with Merkle proof                 |
| `GET`  | `/v1/audit/verify`               | Verify agent's hash chain integrity                      |
| `GET`  | `/v1/audit/provenance/{traceId}` | Get full provenance record (intent + policy + inference) |

### Identity (2)

| Method | Path                             | Purpose                                         |
| ------ | -------------------------------- | ----------------------------------------------- |
| `POST` | `/v1/identity/link`              | Link ERC-8004 identity for reputation discounts |
| `GET`  | `/.well-known/bardo-gateway-key` | Gateway Ed25519 public key (JWK format)         |

### Analytics (4)

| Method | Path                   | Purpose                    |
| ------ | ---------------------- | -------------------------- |
| `GET`  | `/v1/analytics/spend`  | Per-agent cost attribution |
| `GET`  | `/v1/analytics/traces` | Query OTEL traces          |
| `GET`  | `/v1/analytics/cache`  | Cache performance metrics  |
| `GET`  | `/v1/health`           | Gateway health check       |

### Provider management (3)

| Method | Path                    | Purpose                                                     |
| ------ | ----------------------- | ----------------------------------------------------------- |
| `GET`  | `/v1/providers`         | List configured providers and their status                  |
| `GET`  | `/v1/providers/health`  | Provider health summary (latency, error rate, availability) |
| `POST` | `/v1/providers/resolve` | Test-resolve an intent against configured providers         |

---

## Axum router

All 33 endpoints map to a single Axum `Router`. Auth and rate-limiting are tower middleware layers applied after route registration.

```rust
// crates/bardo-gateway/src/router.rs

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Inference (5)
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(messages))
        .route("/v1/completions", post(completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/v1/models", get(list_models))
        // Context engineering (4)
        .route("/v1/collections/:name", put(upsert_collection))
        .route("/v1/collections/:name/query", post(query_collection))
        .route("/v1/memory", post(store_memory))
        .route("/v1/memory/search", post(search_memory))
        // Tool registry (3)
        .route("/v1/tools/:tool_id", put(upsert_tool).delete(delete_tool))
        .route("/v1/tools/search", get(search_tools))
        // Sessions (7)
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/:id/compact", post(compact_session))
        .route("/v1/sessions/:id/handoff", post(handoff_session))
        .route("/v1/sessions/:id/checkpoint", post(checkpoint_session))
        .route("/v1/sessions/:id/resume", post(resume_session))
        .route("/v1/sessions/:parent_id/spawn", post(spawn_session))
        .route("/v1/sessions/:id/scratchpad", patch(update_scratchpad))
        // Templates (2)
        .route("/v1/templates/:name", put(upsert_template).get(get_template))
        // Audit (3)
        .route("/v1/audit/events/:event_id", get(get_audit_event))
        .route("/v1/audit/verify", get(verify_audit))
        .route("/v1/audit/provenance/:trace_id", get(get_provenance))
        // Identity (2)
        .route("/v1/identity/link", post(link_identity))
        .route("/.well-known/bardo-gateway-key", get(gateway_key))
        // Analytics (4)
        .route("/v1/analytics/spend", get(analytics_spend))
        .route("/v1/analytics/traces", get(analytics_traces))
        .route("/v1/analytics/cache", get(analytics_cache))
        .route("/v1/health", get(health_check))
        // Provider management (3)
        .route("/v1/providers", get(list_providers))
        .route("/v1/providers/health", get(provider_health))
        .route("/v1/providers/resolve", post(resolve_provider))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .with_state(state)
}
```

---

## Gateway internal types

The gateway normalizes both OpenAI and Anthropic request formats into a single internal `ChatCompletionRequest`. This struct reaches the provider router, the cache layer, and the audit pipeline.

```rust
// crates/bardo-gateway/src/types.rs

/// Normalized chat completion request used by all internal subsystems.
/// Constructed from either OpenAI or Anthropic wire format during deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier, e.g. "claude-sonnet-4" or "gpt-4o"
    pub model: String,

    /// Conversation messages (normalized to role + content blocks)
    pub messages: Vec<Message>,

    /// System prompt, extracted from top-level (Anthropic) or messages[0] (OpenAI)
    pub system: Option<String>,

    /// Tool definitions available to the model
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,

    /// Whether to stream the response via SSE
    #[serde(default)]
    pub stream: bool,

    /// Sampling temperature (0.0 - 2.0)
    pub temperature: Option<f32>,

    /// Max tokens to generate
    pub max_tokens: Option<u32>,

    /// Top-p nucleus sampling
    pub top_p: Option<f32>,

    /// Stop sequences
    #[serde(default)]
    pub stop: Vec<String>,

    // -- Bardo extensions (not present in upstream API formats) --

    /// Session ID for context continuity across requests
    #[serde(rename = "x-bardo-session")]
    pub session_id: Option<SessionId>,

    /// Agent identity (ERC-8004 DID or API key fingerprint)
    #[serde(rename = "x-bardo-agent")]
    pub agent_id: Option<AgentId>,

    /// Prompt template name + variable bindings
    #[serde(rename = "x-bardo-template")]
    pub template: Option<TemplateRef>,

    /// Cache control: "skip", "read-only", "write-only", or "default"
    #[serde(rename = "x-bardo-cache")]
    pub cache_policy: Option<CachePolicy>,

    /// Provider routing hint: "fastest", "cheapest", or a specific provider name
    #[serde(rename = "x-bardo-routing")]
    pub routing_hint: Option<RoutingHint>,

    /// Security class override: "standard", "confidential", "private"
    #[serde(rename = "x-bardo-security-class")]
    pub security_class: Option<SecurityClass>,

    /// Subsystem identifier for pipeline profile selection
    #[serde(rename = "x-bardo-subsystem")]
    pub subsystem: Option<String>,
}
```

---

## Response headers

Every response carries `X-Bardo-*` headers. These are the constant declarations used across all handler functions.

```rust
// crates/bardo-gateway/src/headers.rs

use axum::http::HeaderName;

/// Which provider + model served this request (e.g. "blockrun/claude-sonnet-4")
pub const X_BARDO_BACKEND: HeaderName = HeaderName::from_static("x-bardo-backend");

/// Processing pipeline used: "standard", "cached", "streaming"
pub const X_BARDO_PIPELINE: HeaderName = HeaderName::from_static("x-bardo-pipeline");

/// Number of reasoning/thinking tokens consumed (extended thinking models only)
pub const X_BARDO_REASONING_TOKENS: HeaderName =
    HeaderName::from_static("x-bardo-reasoning-tokens");

/// Unique trace ID for this request (matches OTEL trace)
pub const X_BARDO_TRACE_ID: HeaderName = HeaderName::from_static("x-bardo-trace-id");

/// Session ID, echoed back when x-bardo-session was provided
pub const X_BARDO_SESSION: HeaderName = HeaderName::from_static("x-bardo-session");

/// Cache status: "hit", "miss", or "skip"
pub const X_BARDO_CACHE: HeaderName = HeaderName::from_static("x-bardo-cache");

/// Cost in microdollars charged for this request
pub const X_BARDO_COST_USD: HeaderName = HeaderName::from_static("x-bardo-cost-usd");

/// Remaining prepaid balance in microdollars (prepaid auth only)
pub const X_BARDO_BALANCE: HeaderName = HeaderName::from_static("x-bardo-balance");

/// Time in milliseconds from request receipt to first byte
pub const X_BARDO_TTFB_MS: HeaderName = HeaderName::from_static("x-bardo-ttfb-ms");

/// SHA-256 hash of the signed audit event for this request
pub const X_BARDO_AUDIT_HASH: HeaderName = HeaderName::from_static("x-bardo-audit-hash");

/// Security class used for this request: "standard", "confidential", "private"
pub const X_BARDO_SECURITY_CLASS: HeaderName =
    HeaderName::from_static("x-bardo-security-class");
```

Example response headers:

```
X-Bardo-Backend: blockrun/claude-sonnet-4
X-Bardo-Pipeline: standard
X-Bardo-Reasoning-Tokens: 1247
X-Bardo-Trace-Id: 7a3f9c1e-4b2d-4e8a-9f1c-3d5e7a9b1c3d
X-Bardo-Cache: miss
X-Bardo-Cost-Usd: 4200
X-Bardo-Ttfb-Ms: 312
X-Bardo-Security-Class: standard
```

---

## Authentication

Two auth modes. See [00-overview.md](00-overview.md) for setup.

**Prepaid balance (recommended):**

```
Authorization: Bearer bardo_sk_abc123...
```

**Per-request x402:**

```
X-Payment: <signed-usdc-authorization>
X-Bardo-Wallet: <base-wallet-address>
```

Auth detection: `bardo_sk_*`/`bardo_pk_*` -> prepaid; `X-Payment` -> x402; neither -> rejected with:

```json
{
  "error": {
    "message": "Invalid API key. Deposit USDC at https://bardo.example.com/deposit to get a key.",
    "type": "authentication_error",
    "code": "invalid_api_key"
  }
}
```

---

## Dual API format

Bardo accepts both Anthropic Messages API and OpenAI Chat Completions API. Auto-detection works from request shape: top-level `system` field + `messages[].content` blocks with `type` means Anthropic; simple string content means OpenAI. Responses return in the caller's detected format.

---

> **Extended:** BardoCompletionRequest schema, full response header catalog, 402 payment response format, and SSE streaming specs -- see [../../prd2-extended/12-inference/09-api-extended.md](../../prd2-extended/12-inference/09-api-extended.md)

---

## Cross-references

| Topic                      | Document                                   | What it covers |
| -------------------------- | ------------------------------------------ | -------------- |
| x402 payment flow          | [00-overview.md](00-overview.md)           | Gateway architecture, x402 payment protocol detail, prepaid balance and per-request flows, and auth detection logic |
| Session endpoints detail   | [05-sessions.md](05-sessions.md)           | Checkpoint/resume, scratchpad working memory, sub-agent spawning, and session lifecycle management |
| Memory endpoints detail    | [06-memory.md](06-memory.md)               | Agent memory service: Styx retrieval augmentation, importance scoring, and background consolidation |
| Analytics endpoints detail | [08-observability.md](08-observability.md) | Per-agent cost attribution, OpenTelemetry traces, Event Fabric integration, and cache performance metrics |
| Safety/privacy             | [07-safety.md](07-safety.md)               | PII detection via compiled regex, prompt injection defense via DeBERTa classifier, and audit logging |
| Privacy and trust          | [11-privacy-trust.md](11-privacy-trust.md) | Three security classes, Venice private cognition, cryptographic audit trail, and cache encryption |
| Multi-provider architecture | [12-providers.md](12-providers.md)        | Five provider backends (BlockRun, OpenRouter, Venice, Bankr, Direct Key) with self-describing resolution |
| Rust implementation        | [14-rust-implementation.md](14-rust-implementation.md) | 10-crate Rust workspace, Axum HTTP server, dependency versions, and WASM compilation targets |

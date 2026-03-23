# 00 -- Inference gateway overview [SPEC]

**Rust inference gateway with context engineering, five-provider routing, and x402 spread revenue**

> Related: [01a-routing.md](01a-routing.md) (model routing spec with self-describing providers and mortality-aware resolution), [02-caching.md](02-caching.md) (three-layer cache stack with regime-aware invalidation), [03-economics.md](03-economics.md) (x402 spread revenue model and per-tenant cost attribution), [04-context-engineering.md](04-context-engineering.md) (8-layer context engineering pipeline for cost reduction)

---

> **Reader orientation:** This document is the top-level specification for **Bardo Inference**, the LLM inference gateway layer of the Bardo ecosystem (a Rust runtime for mortal autonomous DeFi agents called Golems). It describes the gateway's architecture, payment flows, provider routing, context engineering pipeline, and deployment topology. The key concept is that every LLM call an agent makes flows through this gateway, which optimizes cost, routes across five provider backends, and settles payments in USDC via the x402 micropayment protocol. For term definitions, see `prd2/shared/glossary.md`.

## What is the inference gateway

The inference gateway is a Rust binary (`bardo-gateway`) that sits between the Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) runtime and foundation model providers. It accepts LLM requests from the Pi plane, applies an 8-layer context engineering pipeline, routes through five provider backends, and returns completions. The operator keeps the spread between what the user pays and what the provider charges.

The gateway accepts both **Anthropic Messages API** and **OpenAI Chat Completions API** natively, auto-detecting from request shape. Claude Code, Cursor, Aider, and custom apps work with two environment variables. Context engineering (prompt cache alignment, tool pruning, history compression, semantic caching) reduces effective token costs by ~40%, so users pay less through the gateway than calling providers directly -- even after the operator's spread. x402 (a micropayment protocol for HTTP-native USDC payments on Base) settles both legs of every transaction.

ERC-8004 (the on-chain agent identity standard) identity is optional. Users who link an ERC-8004 identity unlock reputation-based spread discounts.

### Three-plane architecture

The Golem's cognition spans three independently deployable planes. Each evolves on its own schedule and communicates over HTTP.

```
Golem-RS runtime (Rust)            Inference plane (Rust)           Styx plane (stateless)
+---------------------------+      +-------------------------+      +---------------------+
| Session state             |      | Routing                 |      | Vector retrieval    |
| ActionPermits             | ---> | 8-layer pipeline        | <--- | Knowledge injection |
| Phase enforcement         |      | Caching (3 layers)      |      | L0/L1/L2 namespaces |
| Tool orchestration        |      | Provider resolution     |      |                     |
| bardo-provider-adapter    |      | Cost accounting         |      | bardo-styx ext      |
+---------------------------+      +-------------------------+      +---------------------+
```

**Golem-RS runtime** owns the Golem's session lifecycle and tool orchestration. It does not route or cache. Every LLM call passes through `bardo-provider-adapter`, an extension that registers the inference gateway as the sole provider at session start and intercepts all subsequent model calls.

**Inference** (the subject of this document) receives every LLM call from the runtime and applies routing, caching, context engineering, and safety checks before the call reaches a foundation model. It is stateless with respect to the Golem's memory. It sees assembled prompts and returns completions.

**Styx** (the knowledge retrieval and injection plane) is a runtime extension that hooks `before_llm_call`. It pre-fetches knowledge from Styx namespaces (Vault, Clade (a peer-to-peer network of Golems sharing knowledge), Lethe (formerly Commons)) and injects it into the Context Governor's input before the inference plane sees any prompt. Styx stays stateless and horizontally scalable because session state lives entirely in the Golem-RS runtime.

The separation is functional. The inference plane's routing model can be swapped without touching the runtime's session logic. Styx's scoring function can change without redeploying inference. The runtime can switch providers by updating one extension with a new endpoint and key.

### `bardo-provider-adapter`: the bridge

A Golem-RS runtime extension registers the inference gateway as the sole provider at session start. This single registration is the trust boundary. Gateway-side middleware (tool-firewall, policy-enforcer, audit-tagger, budget-enforcer) is declared by Pi for observability but enforced server-side. Pi cannot route around the gateway without making a detectable structural modification to the extension set.

```rust
/// Golem-RS runtime extension that bridges to the inference gateway.
/// Registers the gateway as the sole provider on session start.
pub struct ProviderAdapter {
    endpoint: String,
    auth: AuthConfig,
}

impl Extension for ProviderAdapter {
    fn name(&self) -> &str { "bardo-provider-adapter" }
    fn layer(&self) -> u8 { 2 } // Layer 2: Provider

    async fn on_before_agent_start(&self, ctx: &mut AgentStartCtx) -> Result<()> {
        ctx.register_provider(ProviderRegistration {
            id: "bardo".into(),
            endpoint: self.endpoint.clone(),
            auth: self.auth.clone(),
            middleware: vec![
                "tool-firewall",
                "policy-enforcer",
                "audit-tagger",
                "budget-enforcer",
            ],
        });
        Ok(())
    }
}
```

### Standalone product, shared infrastructure

The inference gateway is a standalone product -- any ERC-8004 agent can use it, whether or not it runs on Bardo Compute (the VM hosting service for Golems). It is deeply integrated with the Golem runtime: each Compute VM tier includes a base inference token allowance (overages bill from the Golem's wallet), the `bardo-provider-adapter` routes all Golem LLM calls through inference with T0/T1/T2 model routing applied at the gateway layer, and when the onboarding flow provisions a Golem its first inference calls flow through this gateway. See [prd2/11-compute/03-billing.md](../11-compute/03-billing.md).

---

## Why agents need this

Every autonomous agent on Base needs LLM inference. Today each agent must independently manage API keys, failover logic, rate limiting, prompt optimization, context windows, and spend tracking. The inference gateway eliminates all of it: one endpoint, one payment method (USDC), automatic optimization, and per-request billing that settles on-chain.

### Concrete cost reduction

The naive architecture -- give the agent all tool definitions and let it call Opus every tick -- costs roughly $85/day at 100 ticks/day. The three-plane architecture with inference as the policy layer brings that down to $0.05-$0.40/day on typical workloads, an **18-200x reduction** depending on market conditions. Additional cost reduction paths include DIEM staking (100% discount on Venice-routed calls), Batch API (50% discount on offline dream cycles), and multi-model routing across 5 providers (50-90% savings by matching task complexity to model cost).

The savings come from two mechanisms that compound:

**T0/T1/T2** (the three-tier model routing system: T0 is a zero-cost finite state machine, T1 uses a cheap/fast model, T2 escalates to a frontier model) **FSM routing kills ~80% of ticks before any LLM call.** When all probes return `none` severity, the Heartbeat tick (the Golem's periodic sense-think-act cycle) exits at OBSERVE with zero inference cost. At 100 ticks/day, that eliminates ~80 LLM calls entirely.

**3-layer caching cuts the cost of the remaining 20% by 60-80%.** Anthropic prompt cache (90% discount on cached prefix tokens, 5-minute TTL), semantic cache (cosine > 0.92 threshold), and deterministic hash cache together mean that most T1 and T2 calls pay only for the variable portion of context.

| Market condition         | T0 rate | Daily cost  |
| ------------------------ | ------- | ----------- |
| Calm (bull_low_vol)      | ~90%    | $0.05-$0.10 |
| Normal                   | ~80%    | ~$0.20      |
| Volatile (bear_high_vol) | ~60%    | ~$0.40      |

### Moat: agents that think

Context engineering is the competitive advantage. Context failures, not model failures, cause most agent breakdowns. Anthropic's context engineering framework (2025) established this as the primary bottleneck for production agents [ANTHROPIC-CONTEXT-ENG].

The Context Governor (see [04-context-engineering.md](04-context-engineering.md)) treats context assembly as a **learnable control problem** -- a system that starts with heuristics and evolves through three cybernetic feedback loops into something that knows, per-regime and per-task-type, which categories of context improve decisions and which waste tokens. Seven academic techniques integrated through three loops that run continuously, within a mortality-aware runtime that provides economic pressure to optimize. The techniques are published papers. The integration is months of competitive moat. See `prd2/appendices/moat-think.md`.

### The context engineering platform

Every request passes through 8 layers: prompt cache alignment (L1), semantic response caching (L2), deterministic hash caching (L3), dynamic tool pruning at 97.5% token reduction (L4), history compression via cheap model (L5), provider-specific KV-cache routing (L6), PII masking with round-trip de-identification (L7), and prompt injection detection via ONNX classifier (L8). All transparent to the calling agent. Combined, these reduce inference cost by 40-85%.

The design is informed by how production agent harnesses (Cursor, Claude Code, Aider, Amp, Devin, Codex CLI) manage prompts, context windows, and tool orchestration [ANTHROPIC-CONTEXT-ENG]. These systems have converged on stable prompt prefixes optimized for cache hits, just-in-time context retrieval, lossy compaction at context boundaries, dynamic tool loading, and simple agent loops with at most one level of sub-agent branching.

---

## Five providers

The gateway routes across five provider backends. Each has a unique value proposition. The owner configures which are enabled; the resolver walks the list in priority order until something fits.

| Provider | Payment | Value | When to route here |
|---|---|---|---|
| **BlockRun** | x402 USDC | 30+ models, no API keys, wallet-based auth | Default for standard inference |
| **OpenRouter** | API key | 400+ models, BYOK, automatic failover | Niche models, breadth, BlockRun down |
| **Venice** | API key / DIEM | Zero data retention, visible `<think>` tags | Private cognition, dreams, death reflection |
| **Bankr** | Wallet | Inference wallet = execution wallet, self-funding | Revenue-funded Golems, token launching |
| **Direct Keys** | Owner's billing | Full native API surface per provider | Predicted Outputs, Batch API, explicit caching |

See [12-providers.md](12-providers.md) for full specifications and the self-describing provider resolution algorithm.

Fallback order: `blockrun` (x402, no API key) -> `openrouter` (API key, universal fallback). Venice is a separate inference plane, not a fallback -- selected by task class or security classification. Bankr activates for self-funding economics. Direct Keys bypass the gateway when native provider features are required.

---

## Payment flows

### Prepaid balance flow (recommended)

```text
User                          Bardo Gateway              BlockRun
 | POST + Bearer bardo_sk_*        |                        |
 |-------------------------------->| 1. Validate key        |
 |                                 | 2. Check balance       |
 |                                 | 3. Context engineering  |
 |                                 | 4. x402 payment ------>|
 | SSE stream + [DONE]             |<--- stream chunks -----|
 | X-Bardo-Cost / Savings / Balance|                        |
 |<--------------------------------| 5. Debit balance       |
```

### Per-request x402 flow

```text
User                          Bardo Gateway              BlockRun
 | POST /v1/chat/completions       |                        |
 |-------------------------------->| 1. Estimate tokens     |
 | 402 + X-Payment quote           |                        |
 |<--------------------------------| 2. Calculate price     |
 | POST + X-PAYMENT (signed USDC)  |                        |
 |-------------------------------->| 3. Context engineering |
 |                                 | 4. x402 payment ------>|
 | SSE stream + [DONE]             |<--- stream chunks -----|
 | X-Bardo-Cost                    |                        |
 |<--------------------------------| 5. Settle actual cost  |
```

### Pricing model: `upto` scheme

For per-request x402 mode, LLM inference has unpredictable output length. The gateway uses x402's `upto` payment scheme: the user authorizes a maximum amount, the gateway charges based on actual consumption, and the difference is never settled. For prepaid balance mode, the balance is debited by the actual cost after completion.

### x402 protocol detail

```rust
// crates/bardo-x402/src/pricing.rs

#[derive(Debug, Clone, Serialize)]
pub struct PriceQuote {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub provider_cost: f64,
    pub routing_discount: f64,
    pub cache_discount: f64,
    pub compression_discount: f64,
    pub margin: f64,
    pub total_charged: f64,
}
```

The 402 body includes `scheme: "upto"`, `recipient` (gateway's USDC address on Base), `expiry` (quote validity window), and an `inference` block with `estimatedInputTokens`, `estimatedOutputTokens`, `routedFrom` (selected model), `cacheEligible` (boolean), and `optimizations[]` (list of applied optimizations). The client authorizes `maxAmount`; the gateway charges actual cost and the difference is never settled.

### Authentication modes

Two authentication modes. Both settle in USDC on Base via x402.

**Mode 1: Prepaid balance (recommended)**

Owner deposits USDC, receives a `bardo_sk_*` API key. No per-request signing, no wallet management.

```bash
export ANTHROPIC_BASE_URL="https://bardo.example.com/v1"
export ANTHROPIC_API_KEY="bardo_sk_abc123..."
```

Claude Code works unmodified. Balance visible via `X-Bardo-Balance` response header, low-balance warning via `X-Bardo-Balance-Warning` when < $1.00. Keys are bcrypt-hashed at rest, can be scoped (read-only, spend-limited, IP-restricted).

**Mode 2: Per-request x402**

Every request triggers the x402 flow: gateway returns 402, client signs USDC authorization, retries with payment. Requires the thin wrapper (`bardo-inference` CLI binary) for terminal tools, or direct 402 handling for programmatic clients.

**Auth detection**: `bardo_sk_*`/`bardo_pk_*` in Authorization header -> prepaid; `X-Payment` header -> x402; neither -> rejected.

### Multi-auth gateway

The inference gateway accepts four authentication methods: x402 payment headers, prepaid API keys, Privy JWT tokens, and ERC-8004 delegation proofs. Auth detection runs in order: `X-Payment` header (x402), `Authorization: Bearer bardo_sk_*` (prepaid), `Authorization: Bearer privy_*` (Privy JWT), EIP-712 signature with ERC-8004 delegation (delegation proof). The first match wins. All four methods resolve to the same internal identity before the request enters the pipeline.

### Inference and Styx event paths

Inference events flow Golem -> inference gateway -> response. Styx events flow Golem -> Styx relay -> TUI. These are separate paths. The inference gateway never touches Styx; Styx never routes LLM calls. They share the Event Fabric for observability, but data flows are independent.

### Token allowance headers

The gateway returns `X-Remaining-Tokens: <count>` on every response. Allowances reset hourly. Clients can use this header to track consumption without polling a separate endpoint. When the count reaches zero, further requests return 429 until the next hourly reset.

### ERC-8004 identity (optional)

ERC-8004 identity is not required for v1. Both payment modes work without it. Owners can optionally link an ERC-8004 identity to their API key or wallet address via `POST /v1/identity/link` to unlock reputation-based spread discounts:

| Reputation tier   | Spread |
| ----------------- | ------ |
| None (default)    | 20%    |
| Basic (50+ score) | 18%    |
| Verified (200+)   | 15%    |
| Trusted (500+)    | 12%    |
| Sovereign (1000+) | 8%     |

When an ERC-8004 identity is linked, the gateway authenticates requests using EIP-712 typed data signatures. Verification supports both EOA wallets (standard `ecrecover`) and contract wallets (ERC-1271 `isValidSignature`).

### Rate limiting

| Mode             | Default rate limit | Keyed by       |
| ---------------- | ------------------ | -------------- |
| Prepaid balance  | 100 req/min        | API key        |
| Per-request x402 | 100 req/min        | Wallet address |

### Budget enforcement

The gateway enforces multi-dimensional budget constraints per agent. When a budget dimension is exceeded, the response is graduated:

```rust
// crates/bardo-gateway/src/budget.rs

#[derive(Debug, Clone, Serialize)]
pub enum BudgetAction { Allow, Downgrade, Block }

#[derive(Debug, Clone, Serialize)]
pub struct BudgetDecision {
    pub action: BudgetAction,
    pub reason: String,
    pub alternative_model: Option<String>,
    pub retry_after: Option<u64>,
}
```

Progressive degradation: when a cost budget is exceeded, the gateway finds a cheaper equivalent model rather than rejecting the request. A request targeting Opus that exceeds the session budget may be downgraded to Sonnet; a Sonnet request to Haiku. Only when no cheaper model can satisfy the request does the gateway return 429 with `Retry-After`.

Budget partitioning follows the Golem's wallet structure. When a Golem's wallet allocates funds across inference, compute, and gas reserves, the inference budget maps directly to per-day and per-month caps. The Golem's Daimon (the agent's internal personality and emotional regulation subsystem) can dynamically adjust these caps based on survival pressure -- a Golem in Conservation phase (one of five BehavioralPhases -- Thriving, Stable, Conservation, Desperate, Terminal -- that govern risk tolerance and resource allocation based on survival pressure) may reduce its inference budget to extend lifespan, while a Thriving Golem may increase it. See [../10-safety/01-custody.md](../10-safety/01-custody.md) for wallet fund partitioning.

---

### Dual API format

The gateway accepts both Anthropic Messages API and OpenAI Chat Completions API natively. Auto-detection from request shape:

| Signal | Format |
| --- | --- |
| `system` as top-level string/array + `messages[].content` blocks with `type` | Anthropic |
| `messages[].role` + simple string content | OpenAI |

Endpoints: `POST /v1/messages` (Anthropic), `POST /v1/chat/completions` (OpenAI), `POST /v1/completions` (auto-detect). Internally, the gateway normalizes to OpenAI format for BlockRun. Responses are translated back to the caller's format.

---

## Inference events

Every inference call emits typed `GolemEvent` variants through the Event Fabric for TUI streaming and observability. These events carry CamelCase type tags with `inference.*` prefix.

```rust
/// Emitted when the inference gateway receives a request from Pi.
#[serde(rename = "inference.start")]
InferenceStart {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    model: String,
    provider: String,
    tier: String,           // "T1", "T2"
    subsystem: String,      // "heartbeat_t1", "risk", "dream", etc.
}

/// Streaming token chunk. High frequency during T2 calls.
#[serde(rename = "inference.token")]
InferenceToken {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    token_count: u32,
    is_reasoning: bool,     // true if inside <think> or thinking block
}

/// LLM call completed with final cost and latency.
#[serde(rename = "inference.end")]
InferenceEnd {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    model: String,
    provider: String,
    input_tokens: u32,
    output_tokens: u32,
    cost_usd: f64,
    latency_ms: u64,
    cache_hit_rate: f64,
}

/// Cache hit -- call avoided entirely.
#[serde(rename = "inference.cache_hit")]
CacheHit {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    cache_type: String,     // "hash", "semantic"
    savings_usd: f64,
}

/// Provider failed, falling back to next in chain.
#[serde(rename = "inference.provider_fallback")]
ProviderFallback {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    failed_provider: String,
    fallback_provider: String,
    reason: String,
}
```

The TUI renders `InferenceStart` as a "Thinking..." badge with sprite pulsing, `InferenceToken` as an incrementing token counter, and `InferenceEnd` as a cost badge with decision summary. `CacheHit` shows a green indicator with savings. See [prd2/01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) for the full GolemEvent catalog.

---

## System architecture

```text
+---------------------------------------------------------------------+
|                      Bardo Inference Gateway                        |
|                      (Rust: bardo-gateway crate)                    |
|                                                                     |
|  Auth ──> Router ──> Context Engine ──> KV-Cache Router             |
|  (Prepaid/x402)  (intent-based   (8-layer pipeline:  |              |
|       |           resolution)     budget,tools,       v              |
|  Rate Limiter     Value-Add Svcs  compress,reorder)  Provider       |
|  (per-key/wallet) (DeFi enrich,  Session Services    Registry       |
|                    ToolReg)      (Compaction,         (BlockRun,     |
|                                  Checkpoint,          OpenRouter,    |
|                   Observability   SemanticCache)      Venice,        |
|                   (OTEL->Langfuse)                    Bankr,         |
|                   Tool Format                         Direct)        |
|                   Adapters (Anthropic,                               |
|                   OpenAI,Hermes,Qwen)                                |
+---------------------------------------------------------------------+
            |                                   |
   +--------v--------+                +--------v---------+
   |    Base L2       |                |    BlockRun      |
   | x402 USDC + 8004 |                | (x402, all models)|
   +------------------+                +------------------+
```

---

## Technology stack

| Component | Technology | Rationale |
| --- | --- | --- |
| **Gateway core** | Rust (Axum + Tokio) | <1ms routing P99 at 10K QPS. TensorZero/Bifrost class. |
| **API translation** | Dual-format (Anthropic + OpenAI) | Auto-detection, native both directions. |
| **Provider catalog** | BlockRun API (`GET /v1/models`) | Dynamic pricing + capabilities, hourly refresh. |
| **Tool format adapters** | Anthropic, OpenAI, Hermes, Qwen, JSON | Normalize diverse model tool call formats. |
| **Semantic cache** | In-memory HNSW (`fastembed-rs`) | Default lightweight. Optional Redis 7+ for scale. |
| **Embedding model** | nomic-embed-text-v1.5 (ONNX via `ort`) | 768-dim, <5ms local inference. |
| **Injection classifier** | DeBERTa-v3-base (ONNX via `ort`) | ~8ms quantized INT8. |
| **PII detection** | Compiled regex sets + ONNX NER | No spaCy/Presidio dependency. Crypto-specific patterns. |
| **Vector store** | Embedded HNSW (default), Qdrant (scale) | Lightweight default, Qdrant for >1M vectors. |
| **x402 payment** | `alloy` + `x402.rs` | Native Rust crypto. Zero-copy ECDSA signing. |
| **Observability** | `tracing` + OpenTelemetry -> Langfuse | OTEL-native, self-hostable. |
| **Cost analytics** | SQLite (default), Clickhouse (scale) | SQLite for single-instance, Clickhouse for high volume. |
| **Session storage** | SQLite (default), Redis + PG (scale) | Lightweight default, Redis for hot state at scale. |

### Ten-crate workspace

The gateway is a Rust Cargo workspace of ten crates. Each crate has a single responsibility.

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/bardo-gateway",    # Axum HTTP server, SSE streaming, auth
    "crates/bardo-router",     # Provider resolution, health monitoring
    "crates/bardo-pipeline",   # 8-layer context engineering pipeline
    "crates/bardo-cache",      # L1 prefix, L2 semantic, L3 hash
    "crates/bardo-ml",         # ONNX: DeBERTa + nomic-embed via ort
    "crates/bardo-providers",  # Provider trait, 5 implementations
    "crates/bardo-x402",       # On-chain verification via alloy
    "crates/bardo-safety",     # PII regex + injection classification
    "crates/bardo-telemetry",  # tracing + OpenTelemetry metrics
    "crates/bardo-wasm",       # wasm32-wasi: L2+L3+L8 for edge
]
```

### Why Rust

TypeScript degrades above ~50 concurrent users: the event loop stalls when ONNX inference contends with SSE streaming. Rust eliminates this. Tokio's work-stealing scheduler handles thousands of concurrent connections; ONNX inference runs on dedicated threads via `ort`, decoupled from async I/O. Deployable artifact: ~50 MB static binary (vs. ~2 GB Node/Bun). Cold start: <50 ms (vs. 500 ms+). P99 at 500 concurrent: <50 ms overhead (vs. >200 ms). Prior art: TensorZero (Rust, <1ms P99 at 10K QPS), Bifrost (Rust, 11us median at 5K RPS).

### Performance targets

Pipeline P95 (no compression): <50ms. Routing: <1ms. DeBERTa: <8ms. Embedding: <5ms. Hash cache: <0.1ms. Semantic cache: <5ms. 500+ concurrent connections. Memory: <80MB idle, <200MB at 100 concurrent. Cold start: <50ms. Binary: ~50MB.

---

## Deployment topology

| Tier | Technology | Responsibilities | Cost |
| --- | --- | --- | --- |
| **Single-instance** | Single VPS or Fly.io | Full gateway: auth, routing, pipeline, caching, sessions, billing. In-memory + SQLite. | $50/mo |
| **Multi-instance** | Fly.io (iad + ams + sin) | Same, with Redis for shared cache/sessions, optional Qdrant, optional Clickhouse. | $200-500/mo |
| **Edge** (Phase 3) | Cloudflare Workers (`bardo-wasm` crate) | Auth, rate limiting, cache lookup, pre-filter routing, SSE proxy. | Variable |

---

## Streaming architecture

The gateway normalizes all providers to OpenAI-compatible Server-Sent Events. Provider-specific adapters handle Anthropic's `content_block_delta`, Google's `?alt=sse`, and streaming tool calls. Engineering requirements: disable reverse proxy buffering (`X-Accel-Buffering: no`), heartbeat SSE comments every 15 seconds, backpressure handling for slow clients, and chunk buffering for post-stream cache storage and billing. On semantic cache hit, the gateway simulates SSE streaming from cached response at ~30ms intervals for consistent client experience. See [09-api.md](09-api.md) for the full SSE format.

---

## Design principles

- **Frictionless.** Two env vars = inference. Prepaid balance or x402 -- both work.
- **Cheaper than direct.** Context engineering saves ~40%. Users pay less through the gateway than calling providers directly, even after the operator's spread.
- **Transparent.** Every routing decision, cache hit, cost optimization, and spread amount is visible in response headers. Users audit exactly what they paid for.
- **Self-sovereignty as option.** The gateway is useful, not required. Any user can call providers directly. The gateway earns adoption through better economics, not lock-in.
- **Cache-first architecture.** Every design decision optimizes for prompt cache hit rate -- the single highest-leverage cost optimization.
- **Zero working capital.** Both legs (user -> gateway and gateway -> BlockRun) settle via x402 on Base in USDC. No prepaid API credits, no float, no credit risk.

---

## Standalone product surface

The inference gateway is a standalone product -- any ERC-8004 agent or LLM client can use it, whether or not it runs on Bardo Compute. Two environment variables connect any OpenAI-compatible or Anthropic-compatible client:

```bash
# For Claude Code / Cursor / Aider (Anthropic format)
export ANTHROPIC_BASE_URL=https://inference.bardo.money/v1
export ANTHROPIC_API_KEY=bardo_sk_...

# For OpenAI-compatible clients
export OPENAI_BASE_URL=https://inference.bardo.money/v1
export OPENAI_API_KEY=bardo_sk_...
```

### Feature tiers

| Feature | Free Tier | Pro Tier | Golem Tier |
|---|---|---|---|
| Context engineering (all 8 layers) | yes | yes | yes |
| Backend routing (BlockRun + OpenRouter) | yes | yes | yes |
| Venice backend | no | yes | yes |
| Bankr backend | no | no | yes |
| Direct Key support | no | yes | yes |
| Semantic caching | shared | per-tenant | per-tenant |
| DeFi-specific optimizations | no | no | yes |
| Security-class routing | no | yes | yes |
| ERC-8004 reputation discounts | no | no | yes |
| Spread markup | 25% | 15% | 10% |

For non-Golem users (Claude Code, Cursor, Aider), the gateway provides simplified routing: no subsystem hints needed (task type inferred from request content), no mortality pressure (cost sensitivity from pricing tier), default BlockRun routing with OpenRouter fallback, and all 8 context engineering layers applied automatically.

---

## Backend routing algorithm

When multiple backends can serve a request, the gateway scores candidates across five dimensions:

```
Score = Cost × Quality × Privacy × Latency × Feature match
```

Hard filters apply first: security class (private -> Venice only), required features, model availability. Soft scoring follows, with weights that shift based on cost sensitivity. At `costSensitivity: 0.0` quality dominates; at `1.0` cost dominates. Health checks skip unhealthy backends. See [01a-routing.md](01a-routing.md) for the full algorithm.

Default backend priority when candidates score equally:

1. **BlockRun** -- primary. Cheapest for most models. x402 = zero API key management.
2. **OpenRouter** -- first fallback. Broadest catalog. Provider-level fallback built in.
3. **Bankr** -- when self-funding is active and the model is available.
4. **Venice** -- when privacy is required or DIEM is available.
5. **Direct Keys** -- when a specific native feature is needed.

---

## Backend-specific integration

### BlockRun additions over raw access

Context engineering pipeline (all 8 layers), DeFi-aware ClawRouter extensions (4 additional routing dimensions), tier routing (T0/T1/T2) with mortality-pressure-aware cost optimization, semantic caching tuned for DeFi queries, and spread revenue. Pass-through features: Anthropic Citations/Compaction/thinking/prompt caching/effort, OpenAI reasoning effort/structured outputs/function calling, DeepSeek `<think>` tags, Gemini thinking level. Requires Direct Key: OpenAI Predicted Outputs, Responses API stateful, Gemini explicit caching, custom search grounding.

### OpenRouter alongside BlockRun

Three things BlockRun lacks: model breadth (400+ vs. 30+), BYOK passthrough (operator's own provider keys through OpenRouter), and provider preferences (`provider.sort` for complex routing: throughput/cost/latency constraints). `:nitro` variants for operator-facing low-latency, `:floor` variants for batch/dream cheapest-path. Unified `reasoning_details` format simplifies reasoning parsing across providers.

### Venice context engineering

All 8 pipeline layers still apply. Cached responses stored locally (never sent to Venice). Tool pruning reduces cost even at Venice's low prices. History compression keeps context within Venice models' limits. The `strip_thinking_response` parameter is set dynamically: keep visible for dream/death/daimon_complex subsystems, strip for routine operations.

### Bankr sustainability integration

When Bankr is active, the routing decision factors in the Golem's sustainability ratio (`dailyRevenue / dailyTotalCost`). Ratio > 2.0: no adjustment. Ratio < 0.5: find cheaper model alternatives for non-critical subsystems.

### Direct Keys with context engineering

Even Direct Key requests pass through the pipeline. Caching, compression, and tool pruning apply before the request reaches the direct provider endpoint. The optimization is free; only the direct provider charges apply (no spread on direct key requests).

---

## Oracle and Crypt optionality

**Oracle: local vs. hosted.** Default is local-only: the `bardo-styx` extension reads from the Golem's local Grimoire (the agent's persistent knowledge base, stored in SQLite + LanceDB on the VM). L0 namespace only. Zero cost. Hosted mode (opt-in) adds L1 (clade) + L2 (global) namespaces via Turbopuffer. Anthropic Citations work with local Oracle (entries injected as `search_result` blocks). Gemini custom grounding requires hosted Oracle (needs an HTTP endpoint).

**Crypt: optional encrypted backup.** Without Crypt: Grimoire data is VM-local only. No knowledge inheritance. With Crypt: encrypted backup to R2. Death testaments persist. Successors inherit. Bankr knowledge tokenization works (needs persistent Grimoire).

---

## Configuration

### Golem operator config

```rust
// Operator-side configuration (Bardo Inference instance)
pub struct BardoInferenceConfig {
    pub api_key: String,             // bardo_sk_...
    pub backends: BackendConfig,
    pub services: Option<ServiceConfig>,
    pub cost_sensitivity: Option<f64>,
}

pub struct BackendConfig {
    pub blockrun: BlockRunConfig,     // enabled by default
    pub openrouter: Option<OpenRouterConfig>,
    pub venice: Option<VeniceConfig>,
    pub bankr: Option<BankrConfig>,
    pub direct: Option<DirectKeyConfig>,
}
```

### Non-Golem user config

Non-Golem users need nothing beyond the API key. Backend routing, context engineering, and caching are automatic:

```bash
# Minimum viable configuration
export BARDO_API_KEY=bardo_sk_...
curl https://inference.bardo.money/v1/chat/completions \
  -H "Authorization: Bearer $BARDO_API_KEY" \
  -d '{"model": "claude-opus-4-6", "messages": [...]}'
```

---

## Cross-references

**Inference subdocs:** [01a-routing.md](01a-routing.md) (model routing spec: self-describing providers, declarative intents, and mortality-aware resolution across five backends), [02-caching.md](02-caching.md) (three-layer cache stack: prompt prefix alignment, semantic response cache, and deterministic hash cache with regime-aware invalidation), [03-economics.md](03-economics.md) (x402 spread revenue model, per-tenant cost attribution, and infrastructure cost projections), [04-context-engineering.md](04-context-engineering.md) (the 8-layer context engineering pipeline: prompt cache alignment, semantic cache, hash cache, tool pruning, history compression, KV-cache routing, PII masking, and injection detection), [05-sessions.md](05-sessions.md) (checkpoint/resume, scratchpad working memory, and sub-agent spawning for inference sessions), [06-memory.md](06-memory.md) (agent memory service: Styx retrieval, importance scoring, and background consolidation), [07-safety.md](07-safety.md) (PII detection, prompt injection defense via ONNX classifier, and audit logging), [08-observability.md](08-observability.md) (per-agent cost attribution, OTEL traces, Event Fabric integration, and cache metrics), [09-api.md](09-api.md) (API reference with 33 endpoints, Axum router configuration, response headers, and authentication flows), [10-roadmap.md](10-roadmap.md) (phased delivery plan across three phases with success criteria and open questions), [11-privacy-trust.md](11-privacy-trust.md) (three security classes, Venice private cognition, DIEM staking, cryptographic audit trail, and cache encryption), [12-providers.md](12-providers.md) (five provider backends with full Rust trait implementations and self-describing resolution algorithm), [13-reasoning.md](13-reasoning.md) (unified reasoning chain integration: extended thinking, reasoning traces, and provider-agnostic chain-of-thought normalization), [14-rust-implementation.md](14-rust-implementation.md) (Rust gateway implementation details: 10-crate workspace, dependencies, and WASM compilation targets), [15-inference-profiles.md](15-inference-profiles.md) (InferenceProfile spec: per-subsystem parameter policies, provider parameter mapping, and temperature scheduling), [16-structured-outputs.md](16-structured-outputs.md) (StructuredOutput abstraction across providers with graceful degradation and Anthropic tool-use workaround), [17-streaming.md](17-streaming.md) (streaming UX normalization across provider-specific SSE formats and surface rendering), [18-golem-config.md](18-golem-config.md) (Golem-specific inference provider configuration, capability matrix, and payment method selection)

**External:** [Compute billing](../11-compute/03-billing.md) (how Bardo Compute VM tiers include base inference token allowances and bill overages from the Golem's wallet), [Golem cognition](../01-golem/01-cognition.md) (the Golem's cognitive architecture: sense-think-act loop, subsystem decomposition, and the ModelRouter extension), [Wallet custody](../10-safety/01-custody.md) (wallet fund partitioning across inference, compute, and gas reserves with three custody modes), [Agent economy](../09-economy/05-agent-economy.md) (the broader economic model for autonomous agents including revenue sources and sustainability metrics), [ERC-8004 identity](../09-economy/00-identity.md) (the on-chain agent identity standard that enables reputation-based spread discounts), [Performance targets](../appendices/performance-targets.md) (system-wide latency and throughput targets across all Bardo subsystems), [Hypnagogia onset/return inference](../06-hypnagogia/02-architecture.md) (liminal phase inference profiles for dream-cycle onset and return transitions)

---

## References

- [ANTHROPIC-CONTEXT-ENG] Anthropic (2025). "Building Effective Agents." Context engineering patterns from Claude Code, Cursor, Aider, Amp. Argues that context assembly, not model selection, is the primary bottleneck for production agents; establishes the design patterns (stable prompt prefixes, just-in-time retrieval, lossy compaction) that Bardo Inference implements as its 8-layer pipeline.
- [TENSORZERO-2026] TensorZero. "<1ms P99 at 10,000 QPS." Docs. Demonstrates that a Rust inference gateway can achieve sub-millisecond routing latency at scale; serves as the primary performance benchmark for Bardo's gateway architecture.
- [BIFROST-2026] Bifrost. "11us median at 5K RPS." Maxim AI benchmarks. Shows that Rust-based LLM proxies can reach microsecond-level median latency; validates the choice of Rust over TypeScript for the gateway binary.
- [BLOCKRUN-SDK-2026] BlockRun. "TypeScript SDK." GitHub. The primary provider SDK for x402-based inference; BlockRun is the default routing target for all standard inference calls.
- [OPENROUTER-ROUTING-2026] OpenRouter. "Provider Routing." Docs. Documents OpenRouter's provider preference system (`provider.sort`) and 400+ model catalog; informs the gateway's fallback routing when BlockRun lacks a specific model.
- [VENICE-API-2025] Venice.ai. API Documentation. Covers Venice's no-log inference API, TEE attestation headers, DIEM staking integration, and `strip_thinking_response` parameter; the basis for the private cognition plane.
- [BANKR-LLM-2026] Bankr. "LLM Gateway." Docs. Describes Bankr's wallet-based inference where the execution wallet and revenue wallet are the same key; enables self-funding agent economics.
- [X402-STATE-2025] bc1beat. "The State of x402." December 2025. Surveys the x402 micropayment protocol ecosystem and its `upto` payment scheme for variable-cost operations like LLM inference; foundational to Bardo's zero-working-capital billing model.

# bardo-gateway

LLM inference proxy that sits between your code and upstream providers. Point any Anthropic or OpenAI SDK at it by changing the base URL. You get caching, cost tracking, concurrency control, multi-provider routing, batch processing, and optional USDC micropayments without changing a line of application code.

Runs as a standalone binary or embeds as a library. No golem dependencies. Internal deps are `bardo-primitives` and `bardo-inference` (both zero-dep type crates).

## Why this exists

Running multiple AI agents against raw provider APIs gets expensive. A single multi-agent session can burn $5-50 in inference. The mori orchestrator runs 8-20 agents in parallel. That's real money.

The obvious answer is caching. But LLM request caching is harder than HTTP caching for three reasons:

1. **Requests that mean the same thing have different bytes.** "Explain the auth middleware" and "What does the auth middleware do?" are semantically identical but produce different hashes. A hash cache misses both.

2. **Requests contain per-invocation noise.** UUIDs, timestamps, session IDs embedded in system prompts change every request. Two requests identical except for a timestamp should hit the cache. They don't unless you strip the noise before hashing.

3. **Provider cache models are prefix-based, not request-based.** Anthropic caches the KV state for shared prompt prefixes and gives a 90% discount on cached tokens. But only if the prefix bytes are identical across requests. Tool definitions in different order, or a role name that differs by one word, breaks the prefix match entirely.

The gateway solves all three. Three cache layers, each targeting a different failure mode. Normalization passes that strip noise and stabilize ordering before hashing. Prefix alignment that restructures prompts to maximize provider-side cache hits.

Combined effect: 40-85% cost reduction on a typical run. The range depends on how repetitive your workload is. Agent swarms with shared system prompts and overlapping tasks hit the high end.

## Quick start

```bash
# One Anthropic key
ANTHROPIC_API_KEY=sk-ant-... cargo run -p bardo-gateway

# Multiple keys -- round-robins across them on rate limit
ANTHROPIC_API_KEY=sk-ant-1 ANTHROPIC_API_KEY_2=sk-ant-2 cargo run -p bardo-gateway

# With OpenAI and OpenRouter fallbacks
ANTHROPIC_API_KEY=sk-ant-... \
OPENAI_API_KEY=sk-... \
OPENROUTER_API_KEY=sk-or-... \
  cargo run -p bardo-gateway
```

The gateway starts on `http://localhost:4000`. It generates a random API key at startup and logs it. Set `BARDO_GATEWAY_API_KEY` to use your own.

## Pointing SDKs at the gateway

The gateway speaks both Anthropic and OpenAI wire formats. It auto-detects which format the request uses and responds in kind. This is a design choice: agents shouldn't know they're talking to a proxy. The gateway is transparent.

**Anthropic SDK / Claude Code:**
```bash
export ANTHROPIC_BASE_URL=http://localhost:4000
# That's it. The SDK and claude CLI respect this variable.
```

**OpenAI SDK / Codex / Cursor:**
```bash
export OPENAI_BASE_URL=http://localhost:4000/v1
# Any OpenAI-compatible client works.
```

**curl:**
```bash
curl http://localhost:4000/v1/messages \
  -H "x-api-key: YOUR_GATEWAY_KEY" \
  -H "content-type: application/json" \
  -d '{
    "model": "claude-sonnet-4-6",
    "max_tokens": 1024,
    "messages": [{"role": "user", "content": "hello"}]
  }'
```

Authentication accepts either `x-api-key` or `Authorization: Bearer` headers.

When mori embeds the gateway, it sets the environment variables automatically for every agent subprocess. You don't manually configure base URLs.

## The three-layer cache

Every request passes through three cache layers before hitting a provider. Each layer catches a different class of redundancy that the layers above it miss.

### L1: Hash cache (exact match)

BLAKE3 hash of the normalized request body. In-memory LRU via `moka`. Sub-millisecond lookup.

**What it catches:** Identical repeated requests. Retries after transient failures. Deterministic prompts that produce the same bytes every time. The same review prompt sent to multiple files. More common than you'd expect -- in a typical mori run, 10-15% of requests are exact duplicates.

**Why BLAKE3 and not SHA-256:** BLAKE3 is 3-5x faster on modern CPUs and designed for hash-tree workloads. When every request gets hashed, the speed difference matters.

**Why moka and not a HashMap:** moka gives you LRU eviction and TTL expiration with bounded memory. A HashMap grows without limit. Agent workloads produce thousands of unique requests per hour -- without eviction, memory bloats.

**Normalization before hashing:** Before computing the hash, the gateway runs three normalization passes:

1. **UUID/timestamp stripping** — Replaces UUIDs and ISO 8601 timestamps in system prompts with `[VAR]`. Two requests identical except for "session: abc-123-def" vs "session: xyz-789-ghi" produce the same hash.

2. **Tool sorting** — Sorts the `tools` array alphabetically by name. The Anthropic API accepts tools in any order, but different orderings produce different hashes. Sorting makes the hash order-independent.

3. **JSON key ordering** — Sorts JSON keys via `BTreeMap` serialization. Different JSON serializers may order keys differently. Sorting eliminates this source of hash divergence.

These three passes alone increase L1 hit rates by 15-25% compared to hashing raw request bodies.

Configurable capacity (default 10K entries) and TTL (default 1 hour).

### L2: Semantic cache (similar match)

Two backends, chosen at compile time:

| Backend | How it works | Latency | Setup |
|---------|-------------|---------|-------|
| **SimHash** (default) | 64-bit fingerprint of text shingles, Hamming distance ≤ 3 | ~50µs for 10K entries | Nothing -- pure CPU |
| **Embedding** (`embedding` feature) | fastembed ONNX embeddings, cosine similarity ≥ 0.92 | ~3-5ms per embedding | ~100MB model download |

**What it catches:** Rephrased prompts. "Explain the auth middleware" and "What does the auth middleware do?" are different strings but match semantically. In a multi-agent run, different agents often ask similar questions about the same code from slightly different angles.

**Why SimHash as default over embeddings:** The gateway processes every request synchronously. A 5ms embedding call on every request adds up when you're running 20 agents. SimHash at 50µs is 100x faster and catches the low-hanging fruit (minor rewordings). The embedding backend is opt-in for workloads where accuracy matters more than latency.

**Why Hamming distance 3:** Empirically tuned against agent workloads. Distance 0-1 catches only trivial differences (punctuation, whitespace). Distance 5+ starts producing false positives where the cached response doesn't actually fit the new question. Distance 3 is the sweet spot.

**What's excluded:** Tool-use responses. Replaying a cached tool-use response produces invalid tool IDs on subsequent turns because the model expects to see IDs it generated, not IDs from a prior session. The semantic cache checks for tool_use blocks and skips them.

Entries persist to SQLite (`.mori/costs.db`) every 60 seconds and restore on restart.

### L3: Prompt prefix cache (provider-native)

Anthropic's API caches the KV state for prompt prefixes. If two requests share the same first N tokens, the second request pays only 10% for those shared tokens.

The gateway injects `cache_control: {"type": "ephemeral"}` headers into the system prompt, converting it from a plain string to an array of cache-controlled blocks. This tells Anthropic where the stable prefix ends and the variable suffix begins.

**Why this matters quantitatively:** For an 80K-token prompt where 60K is cached prefix (system prompt + tool definitions + shared context), you pay 60K at 10% rate + 20K at full rate. That's a 67% savings on input tokens. At Opus pricing ($5/M input), saving 54K tokens of full-price input per request saves $0.27 per call. Over a 200-request session, that's $54.

**Prefix alignment:** The gateway also restructures requests to maximize prefix sharing across different agents. Two agents with system prompts that differ only in their role name would normally break prefix matching. The gateway normalizes the shared portion and appends the role-specific suffix, so the prefix matches even across agents with different roles.

## Request normalization pipeline

Every request goes through this pipeline before caching or forwarding:

```
Request arrives
  → strip UUIDs and timestamps from system prompt
  → sort tools array alphabetically by name
  → sort JSON keys for stable serialization
  → compute BLAKE3 hash → check L1
  → extract text for SimHash/embedding → check L2
  → inject cache_control headers for L3
  → prune unused tools (after 5 requests in session)
  → forward to provider
```

Each step has a specific reason for existing, arrived at by watching agent sessions and identifying where money was being wasted.

## Tool pruning

An agent defined with 30 available tools includes all 30 tool definitions in every request. Tool definitions are verbose -- each one is 100-500 tokens. If the agent only uses 3 tools in practice, the other 27 are dead weight: 2-13K tokens per request, charged at full input price.

The `ToolTracker` watches which tools each session actually invokes. After 5 requests (configurable), it strips tool definitions the session hasn't used. The assumption: if you haven't used a tool in 5 requests, you probably won't. If a pruned tool is later needed, the next request will include it (the model will ask for it and the client will resend with the full set).

This saves 2-5K tokens per request for tool-heavy agents. At Sonnet pricing ($3/M input), that's $0.006-0.015 per request. Not much individually, but it compounds across hundreds of requests per session.

## Multi-provider routing

Routing happens by model name prefix. Requests for `claude-*` go to Anthropic, `gpt-*` to OpenAI, and so on.

| Provider | Env var | What it does |
|----------|---------|-------------|
| **Anthropic** | `ANTHROPIC_API_KEY` | Primary provider. Supports key rotation across up to 10 keys. |
| **OpenAI** | `OPENAI_API_KEY` | Chat Completions passthrough. |
| **OpenRouter** | `OPENROUTER_API_KEY` | Aggregator with 400+ models. Good fallback. |
| **Venice** | `VENICE_API_KEY` | TEE-attested inference with zero retention. Privacy-focused workloads. |
| **Bankr** | `BANKR_API_KEY` | Self-funding agent wallets. Optional `BANKR_API_BASE_URL` override. |

All providers implement the `Provider` trait (`src/provider.rs`). Adding a new provider means implementing `send`, `parse_response`, and `report_cost`. The trait is intentionally minimal -- you don't need to handle caching, normalization, or cost tracking. The gateway does that for all providers uniformly.

**Key rotation:** Multiple Anthropic keys (`ANTHROPIC_API_KEY`, `ANTHROPIC_API_KEY_2` through `ANTHROPIC_API_KEY_10`) round-robin across requests. When one key hits a rate limit, the gateway moves to the next. This is the simplest way to increase throughput without changing client code.

**Failover:** When a provider returns an error or rate-limits, the gateway falls through to the next available provider that can serve the requested model. The order is configurable by priority. If Anthropic is down, requests fall to OpenRouter (which proxies Anthropic via a different endpoint with separate rate limits).

## Cost tracking

Every request gets priced. The gateway maintains a pricing table (`src/pricing.rs`) with per-model rates for input, output, cached input, and reasoning tokens. Prices are hardcoded for known models (March 2026 rates) with a fallback for unknown models ($3/M input, $15/M output).

**Why hardcoded and not fetched from the provider:** Provider pricing APIs don't exist (Anthropic) or are unreliable (OpenAI). Hardcoded tables are accurate, fast, and don't add an API dependency. The table is easy to update -- it's a single function returning a `Vec<ModelPricing>`.

Response headers on every request:

```
X-Mori-Cost-Usd: 0.0234
X-Mori-Naive-Cost-Usd: 0.0380
X-Mori-Savings-Usd: 0.0146
X-Mori-Cache-Status: semantic-hit
X-Mori-Provider: anthropic
X-Mori-Tokens-In: 45000
X-Mori-Tokens-Out: 1200
X-Mori-Session-Cost: 4.83
X-Mori-Session-Savings: 2.32
```

`X-Mori-Naive-Cost-Usd` is what you'd pay without the gateway. The difference (`X-Mori-Savings-Usd`) is the concrete value the gateway is providing. If savings are consistently near zero, the gateway isn't earning its keep for your workload.

Costs persist to SQLite at `.mori/costs.db` and restore across restarts. Per-model, per-session, and per-key breakdowns.

```bash
# Get cost breakdown
curl http://localhost:4000/v1/costs -H "x-api-key: YOUR_KEY"

# Live stats
curl http://localhost:4000/v1/stats
```

## Batch API

Non-urgent work (enrichment, summarization, pattern extraction) can go through Anthropic's Batch API at 50% cost. The gateway queues requests and manages the lifecycle.

**Why batch matters:** In a typical mori build, 40-60% of inference spend is non-urgent. Enrichment scripts, plan generation, review pre-scoring, pattern extraction -- all of these can tolerate minutes of latency. At 50% discount, that's a 20-30% reduction on total build cost, stacking on top of the caching savings.

```bash
# Submit a request for batch processing
curl -X POST http://localhost:4000/v1/batch/submit \
  -H "x-api-key: $KEY" \
  -H "content-type: application/json" \
  -d '{"model":"claude-haiku-4-5-20251001","max_tokens":1024,"messages":[...]}'
# -> {"batch_item_id":"bardo-abc123","status":"queued","queue_position":7}

# Force-flush the queue (submit batch to Anthropic now)
curl -X POST http://localhost:4000/v1/batch/flush -H "x-api-key: $KEY"

# Check batch status
curl http://localhost:4000/v1/batch/status -H "x-api-key: $KEY"

# Poll for result
curl http://localhost:4000/v1/batch/result/bardo-abc123 -H "x-api-key: $KEY"
# -> {"status":"completed","result":{...}}  or  {"status":"processing"}
```

Auto-flush triggers: queue reaches 50 items, or 30 seconds elapse since last enqueue, or manual flush.

## MPP (Machine Payment Protocol)

HTTP 402-based USDC micropayment flow. Disabled by default. Lets agents pay for inference per-request or via pre-funded sessions without API keys.

The motivation: API keys are a human concept. An autonomous agent that earns and spends USDC should be able to pay for inference directly, the same way it pays for gas. MPP makes inference a commodity priced in USDC, not a gated resource behind an API key.

**Charge mode** -- one-shot per-request billing:
```
Client -> POST /v1/messages
Gateway -> 402 Payment Required (with payment quote)
Client -> pays on-chain, retries with receipt
Gateway -> 200 OK
```

**Session mode** -- pre-funded streaming:
```bash
# Open a session with a deposit
curl -X POST http://localhost:4000/v1/mpp/sessions \
  -d '{"deposit": 20.00}'

# Make requests -- cost deducted from session balance
curl -X POST http://localhost:4000/v1/messages \
  -H "Authorization: Payment <mpp-voucher>" \
  -d '...'

# Check session balance
curl http://localhost:4000/v1/mpp/sessions/{id}

# Close session (unused funds returned)
curl -X DELETE http://localhost:4000/v1/mpp/sessions/{id}
```

Enable with CLI flags or env vars:

```bash
BARDO_MPP_ENABLED=true \
BARDO_MPP_RECIPIENT=0xYourAddress \
BARDO_MPP_SPREAD=0.20 \
  cargo run -p bardo-gateway
```

The spread (default 20%) is the markup over raw provider cost. It covers the gateway operator's infrastructure and makes running a public gateway economically viable.

## Deployment

### Local (development)

Just `cargo run -p bardo-gateway`. Cache persists to `.mori/costs.db` in the working directory, so repeated runs benefit from prior work.

### Embedded in mori

Mori compiles the gateway as a library (the `gateway` feature flag). When `--gateway` is set (default), mori starts the gateway on port 4000 as a background tokio task and routes all agent inference through it. No separate process to manage.

### Remote (shared gateway)

Deploy to Fly.io or any VPS. Multiple clients across different machines share the cache. The semantic cache makes this work even when prompts aren't identical across projects.

```
                    ┌─────────────────────┐
                    │  bardo-gateway       │
                    │  (Fly.io / VPS)      │
                    │                      │
                    │  Cache: hash+semantic │
                    │  Providers: all       │
                    │  Auth: per-tenant     │
                    └──────────┬───────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
    ┌─────────▼──────┐  ┌─────▼──────────┐  ┌──▼──────────────┐
    │ Agent cluster  │  │ Dev laptop     │  │ CI pipeline     │
    │ 8 agents       │  │ Claude Code    │  │ batch jobs      │
    └────────────────┘  └────────────────┘  └─────────────────┘
```

**Why the shared topology matters:** The gateway is the only component that holds provider API keys. Clients authenticate with their own gateway key. You manage API keys in one place, not N. When VPS #1's agents warm the cache with auth-related prompts, VPS #2's agents benefit from those entries if they're working on related code.

## WebSocket dashboard

The gateway broadcasts live stats over WebSocket at `/v1/ws/stats`. Connect a dashboard client to get per-request events as they happen: model, provider, cost, cache status, latency.

A static dashboard UI is served from `/dashboard` (looks for files in `tmp/bardo-dashboard/`).

## Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/v1/messages` | yes | Anthropic Messages API |
| `POST` | `/v1/chat/completions` | yes | OpenAI Chat Completions API |
| `GET` | `/v1/costs` | yes | Per-key cost breakdown |
| `GET` | `/v1/stats` | no | Live gateway stats |
| `GET` | `/v1/health` | no | Health check |
| `GET` | `/v1/models` | no | Available models |
| `GET` | `/v1/ws/stats` | no | WebSocket stats stream |
| `POST` | `/v1/batch/submit` | yes | Submit request for batch processing |
| `POST` | `/v1/batch/flush` | yes | Force flush pending batch |
| `GET` | `/v1/batch/status` | yes | Batch queue status |
| `GET` | `/v1/batch/result/{id}` | yes | Fetch batch result |
| `POST` | `/v1/mpp/sessions` | MPP | Open payment session |
| `GET` | `/v1/mpp/sessions/{id}` | MPP | Session status |
| `DELETE` | `/v1/mpp/sessions/{id}` | MPP | Close session |
| `GET` | `/dashboard` | no | Static dashboard UI |

## Library usage

Embed the gateway in your own binary:

```toml
[dependencies]
bardo-gateway = { git = "https://github.com/uniswap/bardo", path = "apps/bardo-gateway" }
```

```rust
use bardo_gateway::{GatewayConfig, start_server};

let config = GatewayConfig {
    port: 4000,
    anthropic_api_keys: vec![std::env::var("ANTHROPIC_API_KEY")?],
    max_concurrent: 256,
    ..Default::default()
};
start_server(config).await?;
```

`start_server` binds the port and runs until the task is cancelled or the process exits. The `mori` binary embeds the gateway this way when the `gateway` feature is enabled.

## CLI reference

```
bardo-gateway [OPTIONS]

OPTIONS:
  -p, --port <PORT>              Listen port [default: 4000]
      --bind <ADDR>              Bind address [default: 127.0.0.1]
      --api-key <KEY>            Gateway API key [env: BARDO_GATEWAY_API_KEY]
      --max-cache <N>            L1 cache capacity [default: 10000]
      --ttl <SECS>               Cache TTL [default: 3600]
      --max-body-size <BYTES>    Request body limit [default: 10485760]
      --max-concurrent <N>       Concurrency semaphore [default: 256]
      --pool-max-idle <N>        HTTP pool idle conns per host [default: 64]
      --pool-idle-timeout <SECS> Pool idle timeout [default: 90]
      --mpp-enabled              Enable MPP [env: BARDO_MPP_ENABLED]
      --mpp-recipient <ADDR>     MPP recipient wallet [env: BARDO_MPP_RECIPIENT]
      --mpp-spread <PCT>         MPP spread [default: 0.20] [env: BARDO_MPP_SPREAD]
      --mpp-session-ttl <SECS>   MPP session TTL [default: 3600]
      --mpp-quote-validity <SECS> MPP quote validity [default: 300]
```

Provider keys come from environment variables: `ANTHROPIC_API_KEY` (required), plus optional `ANTHROPIC_API_KEY_2` through `ANTHROPIC_API_KEY_10` for rotation.

## Configuration: `GatewayConfig`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `port` | `u16` | `4000` | Listen port |
| `bind` | `String` | `"127.0.0.1"` | Bind address |
| `api_key` | `String` | `""` (random) | Gateway auth key |
| `anthropic_api_keys` | `Vec<String>` | required | One or more keys; round-robins on rate limit |
| `openai_api_key` | `Option<String>` | `None` | Enables OpenAI provider |
| `openrouter_api_key` | `Option<String>` | `None` | Enables OpenRouter provider |
| `venice_api_key` | `Option<String>` | `None` | Enables Venice provider (zero-retention inference) |
| `bankr_api_key` | `Option<String>` | `None` | Enables Bankr provider (self-funding inference) |
| `bankr_base_url` | `Option<String>` | `None` | Override Bankr API base URL |
| `max_cache` | `u64` | `10_000` | L1 hash cache capacity (entries) |
| `ttl` | `u64` | `3600` | Cache TTL in seconds |
| `max_body_size` | `usize` | `10_485_760` | Request body limit (10 MiB) |
| `max_concurrent` | `usize` | `256` | Concurrency semaphore |
| `pool_max_idle` | `usize` | `64` | HTTP pool idle connections per host |
| `pool_idle_timeout` | `u64` | `90` | Pool idle timeout (seconds) |
| `mpp` | `Option<MppConfig>` | `None` | MPP payment config |

## Design decisions

A few choices that aren't obvious from the code alone.

**Why Axum and not Actix/Warp/Hyper directly.** Axum is the natural fit for the tokio ecosystem. Tower middleware composition, native async, good ergonomics for route handlers. Actix uses its own async runtime. Warp's filter API gets unwieldy for this many routes. Raw Hyper is too low-level for the amount of middleware we need (auth, body limits, compression, CORS).

**Why SQLite for cost persistence and not Postgres/Redis.** The gateway runs on a single machine. SQLite is zero-config, embedded, and handles the write volume (one INSERT per request, ~100/sec peak) without breaking a sweat. No network round-trips. No ops burden. The cost DB is a cache, not a system of record -- if it's lost, you lose historical cost data but nothing breaks.

**Why the concurrency semaphore (256 default).** Provider APIs have rate limits, and 20 parallel agents can easily overwhelm them. The semaphore bounds concurrent in-flight requests to providers. It's a global limit, not per-provider, because the downstream bottleneck is usually the provider's per-key rate limit. 256 is high enough for most setups; lower it if you're hitting rate limits, raise it if you have many API keys.

**Why session-scoped tool tracking.** Tool pruning is per-session, not global, because different agents use different tools. A reviewer agent uses `Read` and `Grep` but never `Edit`. An implementer uses all three. Pruning globally would remove `Edit` from the implementer because the reviewer never used it. Session scoping keeps pruning accurate per agent role.

**Why both SimHash and embedding backends.** SimHash is fast and good enough for catching minor rewordings. Embeddings are more accurate but 100x slower. Most users should use SimHash. The embedding backend exists for workloads where semantic matching accuracy is worth the latency cost -- typically shared gateways serving diverse teams where prompt overlap is semantic rather than syntactic.

## Architecture

```
src/
├── lib.rs              # GatewayConfig, start_server() entry point
├── main.rs             # CLI binary (clap)
├── handler.rs          # Route handlers (messages, chat_completions, costs, stats)
├── auth.rs             # API key middleware (x-api-key / Bearer)
├── state.rs            # AppState, GatewayStats, StatsEvent
├── provider.rs         # Provider trait
├── providers/
│   ├── anthropic.rs    # Anthropic Messages API (key rotation)
│   ├── openai.rs       # OpenAI Chat Completions
│   ├── openrouter.rs   # OpenRouter passthrough
│   ├── venice.rs       # Venice (zero-retention)
│   └── bankr.rs        # Bankr (self-funding)
├── cache.rs            # L1 hash cache (BLAKE3 + moka LRU)
├── semantic_cache.rs   # L2 semantic cache (SimHash or fastembed)
├── prefix.rs           # L3 prefix cache (Anthropic cache_control injection)
├── cost_db.rs          # SQLite cost persistence (.mori/costs.db)
├── pricing.rs          # Per-model token pricing table
├── batch.rs            # Anthropic Batch API manager
├── session.rs          # Per-session state tracking
├── tier.rs             # Rate tier tracking per API key
├── tools.rs            # Tool usage tracking and pruning
├── sse.rs              # SSE streaming (Anthropic + OpenAI formats)
├── format.rs           # Request/response normalization between providers
├── compress.rs         # Response compression
├── dashboard.rs        # WebSocket stats broadcast
├── error.rs            # Error types
├── mpp/                # Machine Payment Protocol
│   ├── middleware.rs    # 402 payment middleware, session endpoints
│   ├── session.rs      # Session state management
│   ├── estimator.rs    # Cost estimation for quotes
│   ├── verifier.rs     # On-chain payment verification
│   ├── spread.rs       # Markup calculation
│   ├── db.rs           # MPP persistence
│   ├── types.rs        # MPP wire types
│   └── error.rs        # MPP errors
├── venice/             # Venice provider internals
│   ├── config.rs
│   ├── diem.rs         # DIEM staking integration
│   ├── router.rs
│   ├── security_class.rs
│   └── error.rs
└── bankr/              # Bankr provider internals
    ├── config.rs
    ├── credits.rs
    ├── metabolic.rs
    ├── routing.rs
    └── verification.rs
```

## License

MIT/Apache-2.0

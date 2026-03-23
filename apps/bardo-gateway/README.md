# bardo-gateway

Production inference proxy for the Bardo stack. Sits between clients (agents, tools, shells) and upstream LLM providers, adding caching, cost tracking, concurrency control, and optional USDC micropayments. Any Anthropic or OpenAI SDK can point at it by changing just the base URL.

Runs as a standalone binary or embeds as a library inside `mori` via `use bardo_gateway::{GatewayConfig, start_server}`.

## Running

```bash
# Standalone binary — needs at least one Anthropic key
ANTHROPIC_API_KEY=sk-ant-... cargo run -p bardo-gateway

# Multiple Anthropic keys (round-robin rotation across them)
ANTHROPIC_API_KEYS=sk-ant-1,sk-ant-2 cargo run -p bardo-gateway
```

Point any Anthropic or OpenAI SDK at `http://localhost:4000` and set the `x-api-key` header to your configured `api_key`. If you leave `api_key` empty in config, the gateway generates a random UUID at startup and logs it.

## Routes

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/messages` | Anthropic Messages API |
| `POST` | `/v1/chat/completions` | OpenAI Chat Completions API |
| `GET` | `/v1/costs` | Per-key cost breakdown |
| `GET` | `/v1/stats` | Live gateway stats |
| `GET` | `/v1/health` | Health check (no auth) |
| `GET` | `/v1/models` | Available models (no auth) |
| `GET` | `/v1/ws/stats` | WebSocket stats stream |
| `POST` | `/v1/batch/submit` | Anthropic batch submit |
| `POST` | `/v1/batch/flush` | Force flush pending batch |
| `GET` | `/v1/batch/status` | Batch queue status |
| `GET` | `/v1/batch/result/{id}` | Fetch batch result |
| `POST` | `/v1/mpp/sessions` | Open MPP payment session |
| `GET/DELETE` | `/v1/mpp/sessions/{id}` | Session status / close |
| `GET` | `/dashboard` | Static dashboard UI (served from `tmp/bardo-dashboard/`) |

## Library usage

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

`start_server` runs until the process exits or the task is cancelled. It returns the bound address on success. The `mori` binary embeds the gateway this way when the `gateway` feature is enabled (the default).

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
| `pool_max_idle` | `usize` | `64` | HTTP connection pool idle connections per host |
| `pool_idle_timeout` | `u64` | `90` | Connection pool idle timeout (seconds) |
| `mpp` | `Option<MppConfig>` | `None` | MPP payment protocol config |

## Providers

All providers implement the `Provider` trait (`src/provider.rs`). Routing happens by model name prefix — requests to `claude-*` go to Anthropic, `gpt-*` to OpenAI, etc.

- `AnthropicProvider` — primary provider; supports key rotation across `anthropic_api_keys`
- `OpenAiProvider` — Chat Completions passthrough
- `OpenRouterProvider` — OpenRouter passthrough; useful for model fallbacks
- `VeniceProvider` — zero-retention inference (Venice AI)
- `BankrProvider` — self-funding inference with optional custom `base_url`

## Caching

Three independent caching layers, applied in order:

**L1: Hash cache** (`src/cache.rs`) — exact-match on a BLAKE3 hash of the full request body. Backed by `moka` with LRU eviction. Zero CPU overhead on a hit.

**L2: Semantic cache** (`src/semantic_cache.rs`) — two backends:

- Default (`SimHash`): 64-bit SimHash fingerprint with Hamming distance matching (threshold 3). Pure CPU, ~50µs for 10K entries, no model download.
- Optional (`embedding` feature): fastembed ONNX embeddings with brute-force cosine similarity at threshold 0.92. ~3–5ms per embedding, ~100 MB model download. More accurate for rephrased prompts.

Semantic cache entries persist to SQLite every 60 seconds. Tool-use responses are intentionally excluded from the semantic cache — replaying them produces invalid tool IDs on subsequent turns.

**L3: Prompt prefix cache** (`src/prefix.rs`) — passes Anthropic's native `cache_control` prefix caching headers. Reduces billed tokens on long system prompts.

## Cost tracking

`src/cost_db.rs` — SQLite database at `.mori/costs.db`. Tracks total tokens, USD cost, and naive cost (what you'd pay without caching) per model, per session, per API key. Stats persist across restarts and are restored on startup. The `/v1/costs` endpoint returns a JSON breakdown.

`src/tools.rs` — `ToolTracker` records which tools each session calls and how often, also persisted to SQLite every 5 minutes.

## MPP (Machine Payment Protocol)

`src/mpp/` — HTTP 402-based USDC micropayment flow. Disabled by default; enable by setting `mpp.enabled = true` in `MppConfig`.

Two payment modes:
- **Charge**: one-shot per-request billing. Gateway responds 402 with a payment quote; client pays on-chain and retries with a receipt.
- **Session**: pre-funded streaming pay-as-you-go. Client opens a session via `POST /v1/mpp/sessions`, deposits funds, then makes requests until the balance is exhausted.

`MppConfig` fields: `recipient_address` (Alloy `Address`), `default_spread` (default 0.20 = 20% markup), `session_ttl` (3600s), `quote_validity` (300s).

## Other modules

- `src/auth.rs` — `require_auth` middleware; checks `x-api-key` or `Authorization: Bearer` header
- `src/batch.rs` / `BatchManager` — queues requests and flushes to Anthropic's Batch API; polls for results; has a background flush timer
- `src/compress.rs` — response compression
- `src/session.rs` — tracks per-session state (last active, tool usage); sessions evict after 1 hour of inactivity
- `src/tier.rs` — rate tier tracking per API key
- `src/sse.rs` — SSE streaming support for both Anthropic and OpenAI response formats
- `src/format.rs` — request/response normalization between provider formats
- `src/dashboard.rs` — WebSocket handler that broadcasts `StatsEvent` to connected dashboard clients

## Dependencies

```toml
axum = "0.8"
tokio = "1.50"
reqwest = "0.12"          # provider HTTP client
alloy = "1.7"             # MPP payment verification
blake3                    # L1 cache hashing
moka                      # L1 cache eviction
dashmap                   # concurrent maps (sessions, in-flight requests)
fastembed = "5.13"        # optional: semantic cache embeddings (embedding feature)
rusqlite = "0.32"         # cost_db and semantic cache persistence
uuid                      # session IDs
bardo-primitives          # shared workspace types
bardo-inference           # shared inference protocol types
```

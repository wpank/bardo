# 14 -- Rust gateway implementation [SPEC]

**Single-binary inference gateway with sub-millisecond routing**

> Cross-references: [00-overview.md](../00-overview.md) (gateway architecture and design principles), [12-providers.md](./12-providers.md) (five provider backends with Rust trait implementations), [13-reasoning.md](./13-reasoning.md) (unified reasoning chain integration and streaming parser), [04-context-engineering.md](./04-context-engineering.md) (8-layer pipeline this gateway implements)

---

> **Reader orientation:** This document specifies the Rust implementation of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes the 10-crate Cargo workspace, Axum HTTP server, dependency catalog, WASM compilation targets, and build/deployment process. The key concept is that a single static ~50 MB Rust binary replaces a ~2 GB Node/Bun deployment, with <50ms cold start and sub-millisecond routing at thousands of concurrent connections. For term definitions, see `prd2/shared/glossary.md`.

## 1. Why Rust

TypeScript on Bun or Node degrades above ~50 concurrent users. The event loop stalls when ONNX inference (thread pool) contends with SSE streaming (I/O). At 200 concurrent Claude Code sessions the tail latency doubles; at 500 it becomes unusable.

Rust eliminates this. Tokio's work-stealing scheduler handles thousands of concurrent connections without a shared event loop bottleneck. ONNX inference runs on dedicated threads via `ort`, completely decoupled from async I/O.

| Dimension | TypeScript (Bun/Node) | Rust (Tokio + Axum) |
|---|---|---|
| Deployable artifact | ~2 GB (runtime + node_modules) | ~50 MB static binary |
| Cold start | 500 ms+ | <50 ms |
| P99 at 500 concurrent | >200 ms pipeline overhead | <50 ms pipeline overhead |
| Memory at 100 concurrent | ~800 MB | <200 MB |
| ONNX thread contention | Shares libuv pool | Isolated thread pool |

If Bardo serves hundreds of concurrent Claude Code and Cursor users alongside autonomous Golems, the gateway must be Rust. Pi runtime stays TypeScript. The boundary is HTTP.

### Key Rust ecosystem advantages

**`fastembed-rs`**: Production-grade embedding library (used by Qdrant) wrapping `ort` + HuggingFace `tokenizers`. The entire embed-search-compare path is a single native binary with zero FFI overhead. In TypeScript, this path crosses JS-to-C++ boundaries on every call.

**`alloy`**: Canonical Rust Ethereum library. Zero-copy ECDSA signing for x402 payment authorizations (~0.1ms native vs. ~2ms through ethers.js). Transaction batching via `sol!` macro for contract interaction.

**In-process vector search**: HNSW implementations (`instant-distance`, `qdrant-client` embedded) run similarity search with zero serialization overhead. The embedding-to-search-to-result path stays in the same memory space.

**Zero-copy SSE streaming**: Ownership model enables forwarding SSE bytes from backends to clients without copying into string buffers, parsing JSON, and re-serializing. At high throughput (128K output tokens), TypeScript creates thousands of GC-pressured string allocations. Rust forwards with zero allocations.

### Prior art

- **TensorZero** -- Rust. <1 ms P99 routing at 10K QPS. Similar architecture.
- **Bifrost** -- Rust. 11 us median at 5K RPS. Zero-copy SSE forwarding.
- **Helicone** -- TypeScript, heavily optimized. 8 ms P50. Hits a wall at high concurrency (V8 GC pauses).
- **LiteLLM** -- Python. Adds 15-30 ms per request. Compatibility reference, not a performance target.

---

## 2. Crate workspace layout

```toml
[workspace]
members = [
    "crates/bardo-gateway",
    "crates/bardo-router",
    "crates/bardo-pipeline",
    "crates/bardo-cache",
    "crates/bardo-ml",
    "crates/bardo-providers",
    "crates/bardo-x402",
    "crates/bardo-safety",
    "crates/bardo-telemetry",
    "crates/bardo-wasm",
]
```

**`bardo-gateway`** -- Axum HTTP server. SSE streaming with zero-copy chunk forwarding. Auth middleware (API key + x402 payment verification). Rate limiting via token bucket. Health endpoints. Only crate that binds a port. See: [09-api.md](09-api.md).

**`bardo-router`** -- Provider resolution, model-to-provider mapping, health monitoring with exponential backoff, failover chains. Stateless: routing state lives in `Arc<DashMap>`. See: [01a-routing.md](01a-routing.md).

**`bardo-pipeline`** -- 8-layer context engineering pipeline. Parallel execution of independent layers via `tokio::join!`. Owns the `InferenceRequest` lifecycle from receipt to provider dispatch. See: [04-context-engineering.md](04-context-engineering.md).

**`bardo-cache`** -- Three tiers. L1: prompt alignment (prefix matching for KV cache reuse). L2: semantic similarity via `fastembed-rs` + HNSW index. L3: deterministic hash via `DashMap` (lock-free concurrent hash map). See: [02-caching.md](02-caching.md).

**`bardo-ml`** -- ONNX Runtime via `ort`. Two models: DeBERTa-v3 for injection detection (L8, quantized INT8, <8ms) and nomic-embed-text-v1.5 for semantic embeddings (L2, 768-dim, <5ms via `fastembed-rs`). Both load once at startup, infer on dedicated threads via rayon. See: [07-safety.md](07-safety.md).

**`bardo-providers`** -- `Provider` trait with five implementations: BlockRun, OpenRouter, Venice, Bankr, Direct Key. Each adapter handles auth, request translation, SSE parsing, error mapping. Streaming is the default path. See: [12-providers.md](12-providers.md).

**`bardo-x402`** -- On-chain verification via `alloy`. EIP-3009 payment proof validation. ERC-8004 identity reads for agent authentication. Zero-copy ECDSA signing (~0.1ms in native Rust vs. ~2ms through JS bindings). Stateless: each request carries its own proof. See: [00-overview.md](../00-overview.md).

**`bardo-safety`** -- PII detection via compiled regex sets (SSN, credit card, email, phone). Injection classification delegates to `bardo-ml` for DeBERTa inference. Returns typed verdicts. See: [07-safety.md](07-safety.md).

**`bardo-telemetry`** -- `tracing` subscriber config. OpenTelemetry metrics (request count, latency histograms, cache hit rates, model inference times). Structured JSON for production, pretty output for development. See: [08-observability.md](08-observability.md).

**`bardo-wasm`** -- `wasm32-wasi` compilation target. Exposes pipeline subset (L2, L3, L8) for edge deployment. Uses `ort-tract` (pure-Rust ONNX alternative) since native `ort` cannot compile to WASM. See: section 7 below.

---

## 3. Crate-to-domain file mapping

Each crate's business logic, data types, and protocol details are specified in a domain file. This file covers Rust-specific concerns: workspace layout, dependencies, performance, and compilation targets.

| Crate | Primary domain file | Content |
|---|---|---|
| bardo-gateway | [09-api.md](09-api.md) | Router, handlers, AppState, SSE streaming |
| bardo-router | [01a-routing.md](01a-routing.md) | Classification, tier assignment, provider selection |
| bardo-pipeline | [04-context-engineering.md](04-context-engineering.md) | 8-layer pipeline, prompt assembly, tool pruning |
| bardo-cache | [02-caching.md](02-caching.md) | Hash cache, semantic cache, encryption |
| bardo-ml | [07-safety.md](07-safety.md) | DeBERTa injection detection, embedding |
| bardo-providers | [12-providers.md](12-providers.md) | Provider trait, 5 implementations |
| bardo-x402 | [00-overview.md](../00-overview.md) | Payment verification, pricing |
| bardo-safety | [07-safety.md](07-safety.md) | PII detection, injection classification |
| bardo-telemetry | [08-observability.md](08-observability.md) | OTEL metrics, audit logging, spend analytics |
| bardo-wasm | (this file, section 7) | Edge deployment, WASM compilation |

---

## 4. Dependency manifest

```toml
[workspace.dependencies]
# HTTP & async
axum = "0.8"
tokio = { version = "1.43", features = ["full"] }
hyper = "1.6"
tower = "0.5"
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# ML inference
ort = "2.0"
fastembed = "5.0"
tokenizers = "0.21"

# Blockchain
alloy = { version = "1.0", features = ["full"] }

# Concurrency & data structures
dashmap = "6.1"
bytes = "1.9"
futures = "0.3"

# Observability
tracing = "0.1"
tracing-subscriber = "0.3"
opentelemetry = "0.28"

# Safety & parsing
regex = "1.11"
uuid = { version = "1.0", features = ["v4"] }

# Error handling
anyhow = "1.0"
thiserror = "2.0"
```

Version pins are minimums. `ort = "2.0"` requires ONNX Runtime 1.19+ shared libraries; the build script downloads them on first compile. `alloy = "1.0"` with `full` includes providers, signers, and contract bindings. `fastembed = "5.0"` bundles `ort` internally but we configure it to share the workspace instance to avoid duplicate runtime loads.

---

## 5. Gateway and pipeline

Router setup, `AppState`, and the SSE streaming handler are defined in [09-api.md](09-api.md).

The streaming path is the hot path. `Bytes` buffers from upstream providers forward to the client without copying. The only allocation per chunk is the SSE event wrapper itself.

Pipeline implementation details are in [04-context-engineering.md](04-context-engineering.md). Four layers (L2, L3, L7, L8) are independent and run in parallel via `tokio::join!`, collapsing ~30 ms of sequential work into ~8 ms wall clock (bounded by DeBERTa inference).

### Pipeline timing breakdown

Common case (no compression):

```
L1 align         -- <1ms  --+
                             |
L2 semantic      -- ~5ms  --+
L3 hash          -- <0.1ms --+-- parallel: ~8ms wall clock
L7 PII regex     -- <1ms  --+
L8 DeBERTa       -- ~8ms  --+
                             |
L4 tool prune    -- <1ms  --+
L6 route         -- <1ms  --+

Total (no L5)    -- ~12ms
```

When L5 fires it adds 200-2000 ms. Acceptable because compression only triggers when the context window is nearly full, and the alternative (truncation) loses information.

---

## 6. Performance targets

| Metric | Target | Notes |
|---|---|---|
| Pipeline P95 (no compression) | <50 ms | L5 excluded; gate-to-gate |
| Routing overhead | <1 ms | Provider resolution + health check |
| DeBERTa inference | <8 ms | Quantized INT8, single input |
| Embedding (nomic-embed) | <5 ms | 384-dim via fastembed-rs |
| Hash cache lookup | <0.1 ms | DashMap, lock-free read |
| Semantic cache lookup | <5 ms | HNSW search, top-1 |
| Concurrent connections | 500+ | Tokio work-stealing |
| Memory (idle) | <80 MB | Models loaded, no active requests |
| Memory (100 concurrent) | <200 MB | Per-request allocation minimal |
| Cold start | <50 ms | Binary startup to first request |
| Binary size | ~50 MB | Static link, release build with LTO |

Targets assume x86-64 with 4+ cores. ARM (Graviton, Apple Silicon) is faster on ML inference due to better SIMD throughput on quantized models.

### Concurrency under load

The decisive Rust advantage: latency stays flat under concurrent load. Tokio handles I/O across all cores; rayon handles CPU-bound ML inference with work-stealing that naturally batches concurrent requests.

| Concurrent requests | P95 target |
|---|---|
| 1 | 12ms |
| 10 | 13ms |
| 50 | 15ms |
| 100 | 18ms |
| 500 | 25ms |

TypeScript degrades above ~50 concurrent ONNX calls because all ML inference contends for the same thread pool (default 4 threads). Request 50 waits ~187ms in queue time alone. In Rust, 50 concurrent embeddings become 6-7 batched ONNX calls via rayon work-stealing. Request 50 completes in the same time as request 1.

### Comparison to alternatives

| Gateway | Language | P50 | P99 | Concurrent |
|---|---|---|---|---|
| Bardo (target) | Rust | <10 ms | <50 ms | 500+ |
| TensorZero | Rust | <0.5 ms | <1 ms | 10K+ |
| Bifrost | Rust | 11 us | ~50 us | 5K+ |
| Helicone | TypeScript | 8 ms | ~40 ms | ~200 |
| LiteLLM | Python | 15 ms | ~80 ms | ~100 |

TensorZero and Bifrost are faster because they skip ML inference in the hot path. Bardo's DeBERTa injection scan adds ~8 ms. Deliberate trade: safety costs single-digit milliseconds.

---

## 7. WASM compilation

Target: `wasm32-wasi` for edge deployment on Cloudflare Workers and Fastly Compute.

### What runs at the edge

| Layer | Edge | Origin | Reason |
|---|---|---|---|
| L1 Prompt alignment | Yes | Yes | Pure string manipulation |
| L2 Semantic cache | Yes | Yes | `ort-tract` for embedding |
| L3 Hash cache | Yes | Yes | SHA-256 + in-memory map |
| L4 Tool pruning | Yes | Yes | String matching |
| L5 History compression | No | Yes | Requires LLM call |
| L6 Provider routing | No | Yes | Needs provider health state |
| L7 PII detection | No | Yes | Regex works but model is better |
| L8 Injection detection | Yes* | Yes | Quantized model via `ort-tract` |

*L8 at edge uses a smaller quantized variant. Full DeBERTa runs on origin.

### Build configuration

```toml
# crates/bardo-wasm/Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
bardo-cache = { path = "../bardo-cache", features = ["wasm"] }
bardo-safety = { path = "../bardo-safety", features = ["wasm"] }
bardo-pipeline = { path = "../bardo-pipeline", features = ["wasm"] }
tract-onnx = "0.21"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

### Constraints

- **Module size**: ~120 MB with quantized INT8 models embedded. Cloudflare Workers paid plan allows up to 200 MB.
- **No native `ort`**: ONNX Runtime's C++ core does not compile to WASM. `tract-onnx` is a pure-Rust reimplementation covering the ONNX operators DeBERTa and nomic-embed need.
- **No outbound HTTP during edge execution**: Cache lookups and safety checks run locally. Cache misses proxy to origin.
- **Startup**: WASM instantiation ~200 ms on Cloudflare (model deserialization dominates). Per-request overhead after instantiation matches native targets.

---

## 8. Build instructions

```bash
# Release build (native)
cargo build --release

# Run tests
cargo test

# WASM target (edge deployment)
cargo build --target wasm32-wasi -p bardo-wasm

# Check all targets without building
cargo check --workspace
```

The release build uses LTO and `codegen-units = 1` for size optimization. WASM builds require the `wasm32-wasi` target installed via `rustup target add wasm32-wasi`.

---

## 9. Migration path

### Phase 1: Rust gateway from day one

The Rust binary is the gateway. There is no TypeScript gateway to replace. Pi runtime stays TypeScript; the boundary is HTTP. The gateway calls Pi over localhost or Unix socket for agent state, memory, and tool execution.

```
Client --HTTP--> Rust Gateway --HTTP--> Pi Runtime (TypeScript)
                     |
                     +-- Provider (Anthropic, OpenAI, ...)
                     +-- Cache (L1, L2, L3)
                     +-- Safety (L7, L8)
```

Acceptance criteria:
- All API tests pass against the Rust binary
- P95 latency below 50 ms
- Memory below 200 MB at 100 concurrent
- Zero downtime deployment via blue-green

### Phase 2: Edge deployment

WASM binary on Cloudflare Workers. Edge handles L1-L4 and L8. Cache misses proxy to origin.

```
Client --HTTP--> Edge (WASM)
                   |  cache hit --> respond directly
                   |  cache miss --> proxy to Origin (Rust Gateway)
                   +--------------> Rust Gateway --> Provider
```

Expected edge cache hit rate: 15-25% for repeated queries within a session. Offloads roughly a quarter of origin traffic.

### Phase 3: Client-side WASM

WASM module in the browser or Claude Code's Node process. Offline hash cache and tool pruning before the request leaves the client. No ML models (too large for client distribution).

```
Claude Code --> Client WASM (L1, L3, L4) --HTTP--> Gateway
```

Phase 3 is speculative. Depends on whether ~5-10 ms client-side savings justify integration complexity.

---

## 10. Key Rust ecosystem crates

### `fastembed-rs` (embedding)

Production-grade embedding library (used by Qdrant) wrapping `ort` + HuggingFace `tokenizers`. The entire embed-search-compare path is a single native binary with zero FFI overhead. In TypeScript, this path crosses JS-to-C++ boundaries on every call. Supports MiniLM-L6-v2, BGE, SPLADE sparse embeddings, and reranking.

### `alloy` (x402 / crypto)

Canonical Rust Ethereum library. Zero-copy ECDSA signing for x402 payment authorizations (~0.1ms native vs. ~2ms through ethers.js). Transaction batching via `sol!` macro. The payment layer and inference layer run in the same process, eliminating a service boundary.

### `candle` (optional ML)

HuggingFace's pure-Rust ML framework runs transformer models natively with CUDA, Metal, and CPU backends. No ONNX conversion needed. Enables loading newer embedding models immediately when released without waiting for ONNX export support.

### In-process vector search

HNSW implementations (`instant-distance`, `qdrant-client` embedded) run similarity search with zero serialization overhead. The embedding -> search -> result path stays in the same memory space. No external vector service needed for the default deployment.

### PII masking in Rust

No Presidio equivalent in Rust. Built as compiled regex sets (SSN, credit card, email, phone, wallet addresses) + a small ONNX NER model via `ort` for name/location detection. The `regex` crate is compiled to native code with DFA optimization -- 5-10x faster than JavaScript's RegExp for pattern-heavy PII scanning.

---

## 11. Estimated performance after Rust rewrite

### Pipeline latency comparison

| Profile | TypeScript/Bun p95 | Rust p95 | Improvement |
|---|---|---|---|
| `minimal` (T0 heartbeat) | <1ms | <0.1ms | Both negligible |
| `fast` (Golem internal) | ~15ms | ~5ms | 3x |
| `standard` (user-facing) | ~40ms | ~12ms | 3.3x |
| `full` (high-security) | ~65ms | ~20ms | 3.2x |

### Resource usage

| Metric | TypeScript/Bun | Rust |
|---|---|---|
| Memory (idle) | ~200MB (V8 heap + models) | ~80MB (models only) |
| Memory (100 concurrent) | ~500MB | ~120MB |
| CPU at 50 RPS | ~40% (4 cores) | ~15% (8 cores, work-stealing) |
| Binary/deployment size | ~2GB (runtime + deps + models) | ~50MB (static binary + models) |
| Cold start | ~500ms (Bun) | ~50ms |

### Development effort estimate

| Component | Effort |
|---|---|
| HTTP proxy + SSE streaming + routing | 1 week |
| Backend integrations (5 providers) | 1 week |
| Semantic cache (fastembed-rs + HNSW) | 3 days |
| Hash cache (in-memory DashMap) | 1 day |
| Prompt cache alignment + tool pruning | 4 days |
| PII masking (regex + ONNX NER) | 1 week |
| Injection detection (ort + DeBERTa) | 3 days |
| x402 payment integration (alloy) | 3 days |
| History compression (LLM call) | 1 day |
| Configuration + API surface | 3 days |
| Testing + benchmarking | 1 week |
| **Total** | **~6 weeks** |

---

## 12. Cross-references

| Document | Relationship | What it covers |
|---|---|---|
| [00-overview.md](../00-overview.md) | System architecture; gateway position in the stack | Top-level gateway spec: architecture, payment flows, five-provider routing, and deployment topology |
| [04-context-engineering.md](./04-context-engineering.md) | Defines the 8-layer pipeline this gateway implements | Prompt cache alignment, semantic cache, hash cache, tool pruning, history compression, KV-cache routing, PII masking, and injection detection |
| [12-providers.md](./12-providers.md) | Provider routing logic, failover chains, health monitoring | Five provider backends with full Rust Provider trait implementations and self-describing resolution |
| [13-reasoning.md](./13-reasoning.md) | Reasoning token parsing applied during SSE streaming | Unified reasoning chain integration: extended thinking, visible think tags, and provider-agnostic normalization |
| [03-economics.md](./03-economics.md) | Cost budgets enforced at the routing layer | x402 spread revenue model, per-tenant cost attribution, and budget enforcement logic |
| [07-safety.md](./07-safety.md) | Safety policy that L7 and L8 enforce | PII detection via compiled regex, prompt injection defense via DeBERTa classifier, and audit logging |

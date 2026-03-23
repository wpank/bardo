# [SUPERSEDED by 14-rust-implementation.md]

# Bardo Inference: Performance, Latency & Practicality

> **Document Type**: SPEC (normative) | **Version**: 1.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-14
>
> **Package**: `@bardo/inference`
>
> **Depends on**: `prd2-bardo-inference.md`
>
> **Purpose**: Honest analysis of the performance cost of Bardo Inference's 8-layer context engineering pipeline. Per-layer latency budgets, parallelization strategy, conditional bypass rules, streaming implications, and the architecture decisions that keep total overhead under the threshold where users notice.

---

> **Reader orientation:** This document (superseded by `14-rust-implementation.md`) analyzes the performance cost of Bardo Inference's (the LLM inference gateway for mortal autonomous DeFi agents called Golems) 8-layer context engineering pipeline. It belongs to the inference plane and covers per-layer latency budgets, parallelization strategy, conditional bypass rules, and streaming implications from the original TypeScript/Bun perspective. The key concept is that the full pipeline adds <50ms at P95, dominated by ONNX model inference (DeBERTa at 8ms, nomic-embed at 5ms), keeping overhead well below the threshold where users notice. For term definitions, see `prd2/shared/glossary.md`.

## 1. The Core Tension

Every layer in the context engineering pipeline adds value (cost savings, quality improvement, safety) but also adds **latency**. An 8-layer pipeline that takes 500ms to process before the first token streams from the LLM is a terrible UX — the user stares at a blank screen for half a second before anything happens, on top of the LLM's own time-to-first-token (TTFT).

The target: **total pipeline overhead < 50ms for the p95 request**. This is achievable because most layers are either sub-millisecond (hash lookups, reordering), parallelizable, or conditionally skipped.

For context, production LLM gateway benchmarks show:
- **Bifrost (Go-based)**: 11µs overhead at 5,000 RPS — effectively invisible
- **Cloudflare AI Gateway**: 10–50ms proxy latency
- **TrueFoundry**: ~3–4ms latency at 350+ RPS
- **LiteLLM (Python-based)**: 15–30ms reported by users, with concurrency bottlenecks

Bardo Inference is a TypeScript/Bun service with heavier per-request processing than a pure proxy, but lighter than a Python-based gateway. The goal is to sit in the 20–50ms range for the full pipeline, which is imperceptible when the LLM itself takes 500ms–30s.

---

## 2. Per-Layer Latency Budget

### 2.1 Layer Breakdown

| Layer | Operation | Expected Latency | Parallelizable? | Can Skip? |
|---|---|---|---|---|
| **L1: Prompt cache alignment** | Reorder system prompt segments, place `cache_control` breakpoints | **< 1ms** | N/A (pure computation) | Skip if no Anthropic-backed request |
| **L2: Semantic cache check** | Embed request → cosine similarity search against vector store | **5–25ms** | ✅ Parallel with L3, L7, L8 | Skip for streaming conversations, tool-heavy requests |
| **L3: Hash cache check** | SHA-256 hash → O(1) lookup | **< 0.5ms** | ✅ Parallel with L2 | Never skip — always cheap |
| **L4: Tool pruning** | Analyze tool definitions, replace with meta-tools | **1–3ms** | N/A (must complete before request is sent) | Skip if < 5 tools defined |
| **L5: History compression** | Check token count; if over threshold, summarize older turns | **0ms (check) / 200–2000ms (compress)** | ⚠️ Only when triggered | Skip if under 80% context limit |
| **L6: Lost-in-the-middle** | Reorder context by priority within message array | **< 1ms** | N/A (pure computation) | Skip if < 10 context items |
| **L7: PII masking** | Presidio NER scan + regex patterns + entity replacement | **5–50ms** | ✅ Parallel with L2 | Skip if no PII detected in fast pre-scan |
| **L8: Injection detection** | DeBERTa-v3 classifier on user input | **10–40ms** | ✅ Parallel with L2, L7 | Skip for Golem-internal requests (trusted source) |

### 2.2 The Critical Insight: Most Layers Are Parallel

The pipeline is NOT a sequential 8-step chain. Layers 2, 3, 7, and 8 all operate on the raw input and can run simultaneously. The execution graph looks like this:

```
Request arrives
    │
    ├─ PARALLEL GROUP A (fire all at once):
    │   ├─ L2: Semantic cache check     (~15ms)
    │   ├─ L3: Hash cache check          (~0.5ms)
    │   ├─ L7: PII masking pre-scan      (~10ms)
    │   └─ L8: Injection detection        (~20ms)
    │
    │   ← Wait for all to complete (~20ms wall clock)
    │
    ├─ DECISION POINT:
    │   ├─ L2 or L3 cache hit? → Return cached response. Done. (0ms more)
    │   ├─ L8 injection detected? → Block or warn. Done.
    │   └─ Continue to sequential layers ↓
    │
    ├─ SEQUENTIAL GROUP B (must happen in order):
    │   ├─ L7: Full PII masking (if pre-scan found entities)  (~15ms)
    │   ├─ L1: Prompt cache alignment    (~1ms)
    │   ├─ L4: Tool pruning              (~2ms)
    │   ├─ L6: Position optimization     (~1ms)
    │   └─ L5: History compression check (~0ms, or triggers async)
    │
    └─► Send to backend (~0ms — the request is ready)
         ← LLM TTFT: 200ms–2000ms (this dominates)

Total pipeline overhead: ~20ms (parallel) + ~19ms (sequential) = ~39ms p50
                         ~25ms (parallel) + ~40ms (sequential) = ~65ms p95
```

**The key point**: The LLM's own time-to-first-token is typically 200ms (Groq/Cerebras fast models) to 2000ms+ (Opus 4.6 with deep thinking). Even at the p95 of 65ms, the pipeline overhead is **3–30% of the total TTFT**, which is imperceptible to users.

### 2.3 Per-Layer Deep Dive

#### L2: Semantic Cache — The Biggest Variable

The semantic cache is the most latency-variable layer. It requires:
1. **Embedding the request** — converting the input to a vector
2. **Similarity search** — finding the nearest neighbor in the cache

**Embedding latency** depends heavily on the model:
- **all-MiniLM-L6-v2 (CPU, ONNX optimized)**: ~10–20ms per sentence on modern CPU. Sub-30ms confirmed by benchmark. This is a 33M parameter model that runs entirely on CPU.
- **Model2Vec (static embeddings)**: ~0.04ms per sentence (25,000+ sentences/sec). 500x faster than MiniLM with ~90% of the quality. Suitable if MiniLM is too slow.
- **Ollama embedding endpoint**: ~5–15ms (local, GPU-backed if available)

**Similarity search** latency depends on the vector store:
- **In-memory (HNSW via hnswlib-node)**: < 1ms for 100K vectors
- **SQLite + vector extension**: 1–3ms
- **External (Turbopuffer, Qdrant)**: 5–20ms (network hop)

**Recommendation**: Use ONNX-optimized MiniLM-L6-v2 with in-memory HNSW for the semantic cache. Total: ~15ms. If this is too slow, switch to Model2Vec (~2ms total) with acceptable quality tradeoff.

**Bypass condition**: Skip semantic cache for streaming conversations where the user is mid-flow (the response needs to be fresh, not cached). Also skip for requests with tool calls (tool outputs vary, caching is rarely useful).

#### L7: PII Masking — Presidio Overhead

Presidio uses spaCy NER (en_core_web_lg, ~750MB model) plus regex patterns. The NER step is the bottleneck.

**Latency profile**:
- **spaCy NER on short text (< 500 chars)**: ~5–15ms CPU
- **spaCy NER on medium text (500–2000 chars)**: ~15–30ms CPU
- **spaCy NER on long text (2000+ chars)**: ~30–80ms CPU
- **Regex-only patterns (no NER)**: < 1ms

**Optimization strategy**: Two-phase approach.
1. **Fast pre-scan** (< 2ms): Regex-only check for obvious PII patterns (emails, phone numbers, SSNs, credit cards, wallet addresses). If nothing found, skip NER entirely. This catches ~60% of PII cases.
2. **Full NER scan** (only when pre-scan finds patterns OR for high-security requests): Run spaCy NER. Run in parallel with other layers.

**Alternative**: Use a lighter spaCy model (en_core_web_sm, ~12MB) for the NER step. 3–5x faster with ~10% accuracy loss. Acceptable for most use cases — false negatives go to the LLM, which is less sensitive than sending unmasked PII.

**Bypass condition**: Skip PII masking entirely for Venice-routed requests (Venice has structural zero data retention — PII masking is redundant). Skip for Golem-internal requests (heartbeat, dreams) where no operator PII is present.

#### L8: Injection Detection — DeBERTa Overhead

The DeBERTa-v3-base prompt injection classifier is a ~184M parameter model.

**Latency profile**:
- **DeBERTa-v3-base (CPU, PyTorch)**: ~30–80ms
- **DeBERTa-v3-base (CPU, ONNX Runtime)**: ~15–35ms
- **DeBERTa-v3-small (CPU, ONNX Runtime)**: ~8–20ms (slightly less accurate)
- **DeBERTa-v3-base (GPU)**: ~3–5ms

**Recommendation**: Use ONNX-optimized DeBERTa-v3-small for default, DeBERTa-v3-base for high-security. Run in parallel with semantic cache and PII masking.

**Bypass condition**: Skip for Golem-internal requests (heartbeat, dreams, curator — the prompts are generated by trusted code, not user input). Only run on operator-facing requests and tool results from external sources.

#### L5: History Compression — The Sleeping Giant

History compression is **not** on the critical path for most requests. It only triggers when the conversation approaches the context limit (80% of the model's window). When it does trigger, it's expensive: an LLM call to summarize older turns (200–2000ms depending on the summarization model).

**Mitigation**:
1. **Never block on compression**. Compress asynchronously: send the current request with the full history, and compress in the background for the next request.
2. **Pre-compress proactively**: When history reaches 60% of the limit, start compressing in the background (during idle time between heartbeats).
3. **Use Anthropic Compaction when available**: Server-side compaction happens in-band and is typically faster + higher quality than client-side.
4. **Compression model choice matters**: Use Haiku or Gemini Flash (~200ms TTFT) for compression, not Opus (~1500ms TTFT). The summary doesn't need to be brilliant — it needs to be fast and accurate.

---

## 3. Streaming Considerations

### 3.1 Time-to-First-Token (TTFT) Impact

TTFT is the most perceptible latency metric. Users notice when the screen is blank.

```
Without Bardo Inference:
  User sends message → LLM processes → first token streams
  TTFT = LLM processing time (200ms–2000ms)

With Bardo Inference:
  User sends message → Pipeline (20–65ms) → LLM processes → first token streams
  TTFT = Pipeline (20–65ms) + LLM processing time (200ms–2000ms)

Overhead: 1–30% increase in TTFT
```

For fast models (Groq Llama at ~200ms TTFT), the pipeline adds a noticeable ~30% overhead. For standard models (Claude Opus at ~1500ms TTFT), it adds ~3%. For thinking models (R1 extended thinking at 5000ms+), it's invisible.

### 3.2 Streaming Passthrough

Once the pipeline completes and the request reaches the backend, **streaming works identically to a direct connection**. Bardo Inference forwards SSE events as they arrive — there is no buffering or batching of streaming tokens. The `bardo-ui-bridge` parser operates on individual events as they stream through.

```
Backend SSE event → Bardo Inference (passthrough, ~0ms) → Client
```

The only streaming-specific overhead is the `bardo-ui-bridge` parsing (think tag detection, citation extraction), which is pure string processing at < 0.1ms per event.

### 3.3 Cache Hits: Dramatically Faster

When the semantic or hash cache hits, the response is returned **without any LLM call**:

```
Cache hit path:
  User sends message → Pipeline parallel group (~20ms) → Cache hit → Return cached response
  TTFT: ~20ms (vs. 200–2000ms without cache)

This is 10–100x faster than a fresh LLM call.
```

For Golem heartbeat ticks in calm markets (where ~20% of requests are semantically identical to recent ones), this means 1 in 5 ticks completes in ~20ms instead of ~500ms. This is a massive UX win for the TUI and web dashboard — the Golem appears to "think" instantly for routine checks.

---

## 4. Conditional Bypass Rules

Not every request needs every layer. The pipeline dynamically skips layers based on request characteristics:

### 4.1 Bypass Matrix

| Layer | Skip When | Rationale |
|---|---|---|
| L1: Cache alignment | Request targets non-Anthropic model | Only Anthropic benefits from prefix caching |
| L2: Semantic cache | Streaming multi-turn conversation, tool-heavy request, `cache: false` header | Mid-conversation responses should be fresh |
| L3: Hash cache | Never | Always < 0.5ms, always worth checking |
| L4: Tool pruning | < 5 tools defined, or model has native tool_search (GPT-5.4) | Overhead not worth it for small tool sets |
| L5: History compression | Under 80% context limit | No compression needed |
| L6: Position optimization | < 10 context items | Reordering is negligible for small contexts |
| L7: PII masking | Venice-routed request, Golem-internal request, fast pre-scan negative | Venice = structural privacy. Internal = no user PII |
| L8: Injection detection | Golem-internal request (trusted source), `security: skip` header (non-Golem power users) | Heartbeat/dream prompts are generated code, not user input |

### 4.2 Fast Path Profiles

The pipeline automatically selects a fast path based on request metadata:

```typescript
type PipelineProfile = "full" | "standard" | "fast" | "minimal";

function selectProfile(request: InferenceRequest): PipelineProfile {
  // Golem heartbeat T0 — deterministic, no LLM call
  if (request.bardo?.subsystem === "heartbeat_t0") return "minimal";

  // Golem internal subsystems (dreams, curator, daimon)
  if (request.bardo?.subsystem && !request.bardo.isOperatorFacing) return "fast";

  // Operator-facing or external user request
  if (request.bardo?.isOperatorFacing || !request.bardo) return "standard";

  // High-security request (death reflection, MEV-sensitive)
  if (request.securityClass === "private") return "full";

  return "standard";
}

const PROFILES: Record<PipelineProfile, LayerConfig> = {
  full: {
    // All 8 layers, no skips. Maximum optimization + safety.
    cacheAlignment: true, semanticCache: true, hashCache: true,
    toolPruning: true, historyCompression: true, positionOptimization: true,
    piiMasking: true, injectionDetection: true,
  },
  standard: {
    // Standard path for most requests. Skip injection detection for trusted sources.
    cacheAlignment: true, semanticCache: true, hashCache: true,
    toolPruning: true, historyCompression: true, positionOptimization: true,
    piiMasking: true, injectionDetection: true,
  },
  fast: {
    // Golem internal. Skip PII masking and injection detection.
    cacheAlignment: true, semanticCache: true, hashCache: true,
    toolPruning: true, historyCompression: true, positionOptimization: true,
    piiMasking: false, injectionDetection: false,
  },
  minimal: {
    // T0 heartbeat. Only hash cache (in case of exact repeat).
    cacheAlignment: false, semanticCache: false, hashCache: true,
    toolPruning: false, historyCompression: false, positionOptimization: false,
    piiMasking: false, injectionDetection: false,
  },
};
```

**Latency by profile**:

| Profile | Typical Latency | When Used |
|---|---|---|
| `minimal` | < 1ms | T0 heartbeat (no LLM call anyway) |
| `fast` | ~5–15ms | Golem internal (dreams, curator, daimon) |
| `standard` | ~20–40ms | Operator conversation, external users |
| `full` | ~30–65ms | Private/high-security requests |

---

## 5. Infrastructure Sizing

### 5.1 Compute Requirements for Low Latency

| Component | Minimum for < 50ms | Recommended |
|---|---|---|
| **CPU** | 4 cores (embedding + DeBERTa compete for CPU) | 8 cores (comfortable headroom) |
| **RAM** | 4GB (MiniLM ~100MB + DeBERTa ~370MB + spaCy ~750MB + cache) | 8GB |
| **GPU** | Not required (ONNX on CPU is sufficient) | Optional: small GPU (T4/L4) for DeBERTa (~5ms instead of ~20ms) |
| **Vector store** | In-memory HNSW (part of process RAM) | Same |
| **Disk** | 1GB (models + SQLite cache) | 10GB (larger cache) |

### 5.2 Model Loading Strategy

The biggest single latency penalty is **cold start** — loading models into memory on first request. This can take 5–15 seconds for spaCy + DeBERTa + MiniLM.

**Mitigation**:
- Load all models at server startup, not on first request
- Use Bun's fast startup (vs. Node.js) for ~2x faster cold start
- Keep the server warm with a health check endpoint that exercises the pipeline
- If using serverless (not recommended for Bardo Inference), use provisioned concurrency

### 5.3 Concurrency

Each request's parallel group (L2, L3, L7, L8) uses ~4 concurrent operations. With Bun's event loop + worker threads for CPU-bound ONNX inference:

- **1 concurrent request**: Full CPU available, ~20ms parallel group
- **10 concurrent requests**: CPU contention on ONNX inference, ~40ms parallel group
- **50+ concurrent requests**: Queue depth increases, consider horizontal scaling

**Recommendation for single-instance public Bardo Inference**: 8-core machine handles ~20 concurrent requests with < 50ms pipeline latency. Above that, horizontal scaling (multiple instances behind a load balancer) with shared cache (Redis or Turso for SQLite).

---

## 6. The "Is It Worth It?" Analysis

### 6.1 Latency Cost vs. Dollar Cost Savings

| Scenario | Pipeline Overhead | Dollar Savings | Worth It? |
|---|---|---|---|
| Operator typing a message (Claude Opus) | +40ms on 1500ms TTFT | 60–70% cost reduction | **Yes** — imperceptible latency, massive savings |
| Heartbeat T1 (Gemini Flash) | +15ms on 300ms TTFT | 40–50% cost reduction | **Yes** — 5% latency increase, significant savings |
| Heartbeat T0 (no LLM call) | +1ms on 0ms | $0 (no LLM call to save on) | **Neutral** — minimal profile adds ~nothing |
| Dream cycle (DeepSeek R1, 120s timeout) | +40ms on 5000ms+ | 20–30% cost reduction | **Yes** — 0.8% latency increase, good savings |
| Risk assessment (Claude Opus, tools) | +40ms on 2000ms TTFT | 50% cost reduction (tool pruning alone) | **Yes** — 2% latency increase, critical savings |
| Fast model (Groq Llama, 200ms TTFT) | +40ms on 200ms TTFT | Semantic cache hits = 0ms | **Maybe** — 20% latency increase, but cache hits are 10x faster |

### 6.2 When Bardo Inference Adds Negative Latency

Cache hits don't just save money — they're **faster** than direct provider calls:

```
Direct to provider:     ~800ms average TTFT (Claude Sonnet)
Bardo Inference miss:   ~840ms average TTFT (40ms overhead + 800ms LLM)
Bardo Inference hit:    ~15ms TTFT (cached response, no LLM call)

At 20% hit rate:
  Weighted average TTFT = 0.20 × 15ms + 0.80 × 840ms = 675ms

675ms < 800ms → Bardo Inference is FASTER on average despite per-request overhead
```

This is the key insight: **semantic caching transforms Bardo Inference from a latency cost into a latency benefit** at hit rates above ~5% (which heartbeat ticks easily achieve in calm markets).

---

## 7. What to Monitor

### 7.1 Latency Metrics

```typescript
interface PipelineMetrics {
  // Per-layer timing
  layers: {
    cacheAlignment: { p50Ms: number; p95Ms: number; p99Ms: number; skipRate: number };
    semanticCache: { p50Ms: number; p95Ms: number; hitRate: number; skipRate: number };
    hashCache: { p50Ms: number; p95Ms: number; hitRate: number };
    toolPruning: { p50Ms: number; p95Ms: number; tokensSaved: number; skipRate: number };
    historyCompression: { p50Ms: number; p95Ms: number; triggerRate: number };
    positionOptimization: { p50Ms: number; p95Ms: number; skipRate: number };
    piiMasking: { p50Ms: number; p95Ms: number; detectionRate: number; skipRate: number };
    injectionDetection: { p50Ms: number; p95Ms: number; flagRate: number; skipRate: number };
  };

  // Aggregate
  pipeline: {
    totalP50Ms: number;      // Target: < 30ms
    totalP95Ms: number;      // Target: < 50ms
    totalP99Ms: number;      // Target: < 100ms
    cacheHitRate: number;    // Target: > 15%
    skipRate: number;        // How often layers are bypassed
  };

  // End-to-end (including LLM)
  endToEnd: {
    ttftP50Ms: number;
    ttftP95Ms: number;
    totalP50Ms: number;      // Including full response streaming
  };
}
```

### 7.2 Alert Thresholds

| Metric | Warning | Critical |
|---|---|---|
| Pipeline p95 latency | > 80ms | > 150ms |
| Semantic cache embedding latency | > 30ms | > 50ms |
| DeBERTa inference latency | > 40ms | > 80ms |
| PII masking latency | > 40ms | > 100ms |
| History compression trigger rate | > 30% of requests | > 50% (context window too small or conversations too long) |
| Cache hit rate (heartbeat) | < 10% | < 5% (cache not working) |

---

## 8. Implementation Recommendations

### 8.1 Language & Runtime

**Bun over Node.js**: Bun's faster startup (~2x), native SQLite support, and efficient event loop make it the better choice. The ONNX Runtime for DeBERTa and MiniLM runs via native bindings (C++) regardless of the JS runtime.

**Not Go**: Unlike Bifrost (11µs overhead in Go), Bardo Inference does real computation per request (embedding, NER, classification). The proxy routing overhead is negligible; the ML inference layers dominate. Go's advantage in raw proxy throughput doesn't apply here. TypeScript is the right choice for integration with the Pi ecosystem (all TypeScript).

### 8.2 ONNX for All ML Models

Convert all ML models (MiniLM, DeBERTa, spaCy) to ONNX format for inference:
- **2–4x faster** than PyTorch on CPU
- **Consistent latency** (no Python GIL contention)
- **Smaller memory footprint**
- **Native C++ execution** via onnxruntime-node

### 8.3 Consider Model2Vec for Semantic Cache

If MiniLM embedding latency (~15ms) is too high, switch to Model2Vec static embeddings:
- **500x faster** (~0.04ms per sentence)
- **~90% of MiniLM quality** on retrieval tasks
- **2.5MB model size** (vs. 100MB for MiniLM)
- **No ONNX needed** — pure JavaScript computation

The quality tradeoff is acceptable for semantic caching (threshold-based matching), where near-misses don't matter (they just result in a cache miss, not a wrong answer).

### 8.4 Presidio Alternatives

If Presidio's spaCy dependency is too heavy (~750MB for en_core_web_lg + ~30ms latency):

1. **Regex-only PII masking**: Handles emails, phone numbers, SSNs, credit cards, wallet addresses. Covers ~80% of PII cases at < 1ms. Good enough for most DeFi use cases (wallet addresses are the primary PII concern).

2. **Lighter NER model**: Use spaCy's en_core_web_sm (12MB, ~5ms) instead of en_core_web_lg. Catches names and locations with lower accuracy but much faster.

3. **LLM-based PII detection**: Use the LLM itself to detect and redact PII in its response. This adds zero pre-request latency but means PII reaches the LLM (acceptable for Venice, not for other providers).

### 8.5 Async Everything That Can Be Async

- **History compression**: Never block. Compress in the background after responding.
- **PII masking on responses**: Mask in the streaming response handler, not pre-response.
- **Injection detection**: If running in `fast` profile, can be deferred to post-response logging (detect-and-flag rather than detect-and-block).
- **Usage tracking**: Log asynchronously. Never block a response to write a metric.

---

## 9. Failure Modes & Graceful Degradation

| Failure | Impact | Mitigation |
|---|---|---|
| Embedding model crashes | Semantic cache unavailable | Hash cache still works. Requests pass through uncached. |
| DeBERTa model crashes | Injection detection unavailable | Log the failure, continue without detection. Alert operator. |
| spaCy/Presidio crashes | PII masking unavailable | Regex-only fallback (< 1ms). Log the failure. |
| Vector store corrupted | Semantic cache miss on everything | Rebuild from scratch (cache is ephemeral). No data loss. |
| Cache full (memory pressure) | Eviction rate increases | LRU eviction is automatic. Worst case: lower hit rate. |
| All layers fail | Pipeline degrades to passthrough | Request goes directly to backend with zero optimization. Still works. |

**Core principle**: Every pipeline layer failing degrades to **passthrough**, not to **error**. The worst case is "Bardo Inference acts like a dumb proxy." The user's request always goes through.

---

## 10. Benchmarking Plan

Before shipping, run these benchmarks on the target hardware:

```typescript
// Benchmark suite
const benchmarks = [
  // Individual layer benchmarks
  { name: "hash_cache_lookup", iterations: 10000, target_p95_ms: 1 },
  { name: "semantic_cache_embed", iterations: 1000, target_p95_ms: 25 },
  { name: "semantic_cache_search", iterations: 1000, target_p95_ms: 5 },
  { name: "tool_pruning_171_tools", iterations: 1000, target_p95_ms: 5 },
  { name: "prompt_cache_alignment", iterations: 1000, target_p95_ms: 2 },
  { name: "pii_masking_short_text", iterations: 1000, target_p95_ms: 15 },
  { name: "pii_masking_long_text", iterations: 100, target_p95_ms: 50 },
  { name: "injection_detection", iterations: 1000, target_p95_ms: 30 },

  // Full pipeline benchmarks
  { name: "pipeline_minimal_profile", iterations: 1000, target_p95_ms: 2 },
  { name: "pipeline_fast_profile", iterations: 1000, target_p95_ms: 20 },
  { name: "pipeline_standard_profile", iterations: 1000, target_p95_ms: 50 },
  { name: "pipeline_full_profile", iterations: 100, target_p95_ms: 80 },

  // Concurrent load benchmarks
  { name: "10_concurrent_standard", iterations: 100, target_p95_ms: 60 },
  { name: "50_concurrent_standard", iterations: 50, target_p95_ms: 100 },

  // End-to-end (including backend call)
  { name: "e2e_cache_hit", iterations: 100, target_p95_ms: 25 },
  { name: "e2e_cache_miss_haiku", iterations: 50, target_p95_ms: 600 },
  { name: "e2e_cache_miss_opus", iterations: 20, target_p95_ms: 2500 },
];
```

---

## References

- [BIFROST-BENCHMARKS-2026] Bifrost. "11µs overhead at 5,000 RPS." Maxim AI benchmarks, 2026.
- [CLOUDFLARE-AI-GATEWAY-2025] Cloudflare. "AI Gateway: 10–50ms proxy latency." Docs.
- [TRUEFOUNDRY-BENCHMARK-2025] TrueFoundry. "~3–4ms latency, 350+ RPS on 1 vCPU." Blog.
- [LITELLM-LATENCY-2025] Reddit users. "15–30ms additional latency." r/LLMDevs.
- [MINILM-BENCHMARK-2025] AIMultiple. "Sub-30ms cluster: MiniLM-L6-v2 < 30ms latency." Benchmark, 2025.
- [MODEL2VEC-2025] Hrishikesh. "500x faster than MiniLM." Medium, 2025.
- [PRESIDIO-LATENCY-2024] Microsoft Presidio. "Latency depends heavily on recognizers and NLP models." GitHub Discussion #1097.
- [PROTECTAI-DEBERTA-2024] ProtectAI. "deberta-v3-base-prompt-injection-v2." HuggingFace.
- [PORTKEY-SEMANTIC-CACHE-2025] Portkey. "Semantic Caching for AI Gateway." 2025.
- [ONNX-INFERENCE-2025] ONNX Runtime. "2–4x faster than PyTorch on CPU." Docs.

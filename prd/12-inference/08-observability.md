# 08 -- Observability and analytics [SPEC]

**Per-agent cost attribution, OpenTelemetry traces, Event Fabric integration, cache metrics, performance targets**

> Related: [07-safety.md](07-safety.md) (PII detection, prompt injection defense, and audit logging), [03-economics.md](03-economics.md) (x402 spread revenue model and per-tenant cost attribution), [09-api.md](09-api.md) (API reference with 33 endpoints including analytics), [11-privacy-trust.md](11-privacy-trust.md) (cryptographic audit trail with Merkle anchoring and provenance records)

---

> **Reader orientation:** This document specifies the observability and analytics layer of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and covers per-agent cost attribution, OpenTelemetry trace integration, Event Fabric event streaming, and cache performance metrics. The key concept is that every inference call produces structured telemetry that enables operators to understand cost, latency, and cache efficiency per agent and per subsystem. For term definitions, see `prd2/shared/glossary.md`.

## Per-tenant cost attribution API

Every user (API key or wallet address) can query their inference spend with full breakdowns by model, optimization type, and time period. Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) agents get additional per-strategy attribution.

### GET /v1/analytics/spend

```rust
// crates/bardo-telemetry/src/spend.rs
#[derive(Debug, Clone, Serialize)]
pub struct SpendAnalytics {
    pub tenant_id: String,               // API key hash or wallet address
    pub period_start: u64,
    pub period_end: u64,
    pub total_spend_usdc: f64,           // paid by user (including spread)
    pub total_blockrun_cost_usdc: f64,   // paid to BlockRun
    pub total_spread_usdc: f64,          // earned by operator
    pub estimated_naive_cost_usdc: f64,  // without context engineering
    pub total_requests: u64,
    pub prompt_cache_hit_rate: f64,
    pub semantic_cache_hit_rate: f64,
    pub tokens_saved_by_routing: u64,
    pub tokens_saved_by_compression: u64,
    pub tokens_saved_by_tool_pruning: u64,
    pub cost_saved_total_usdc: f64,
    pub compactions_triggered: u64,
    pub handoffs_triggered: u64,
    pub savings_semantic_cache: f64,     // USDC saved by optimization type
    pub savings_prefix_cache: f64,
    pub savings_tool_pruning: f64,
    pub savings_history_compression: f64,
    pub by_model: Vec<ModelSpend>,       // model_id, provider, requests, spend/cost, purpose
}
```

---

## OTEL metrics and tracing setup

A single `MetricsCollector` registers every instrument from the tables below. Pipeline layers take a shared reference instead of touching global state.

```rust
// crates/bardo-telemetry/src/metrics.rs
use opentelemetry::{global, metrics::{Counter, Histogram, Gauge, Meter}};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct MetricsCollector {
    pub cache_hit_rate: Gauge<f64>,
    pub cache_entries: Gauge<u64>,
    pub cache_savings_usd: Counter<f64>,
    pub cache_stale_served: Counter<u64>,
    pub prefix_cache_hit_rate: Gauge<f64>,
    pub cache_invalidations: Counter<u64>,
    pub spread_earned_total: Counter<f64>,
    pub blockrun_cost_total: Counter<f64>,
    pub user_savings_total: Counter<f64>,
    pub spread_pct_effective: Gauge<f64>,
    pub fallback_requests: Counter<u64>,
    pub pipeline_duration_ms: [Histogram<f64>; 8], // one per layer
    pub pipeline_total_duration_ms: Histogram<f64>,
    pub pipeline_profile: Counter<u64>,
    pub batch_api_savings_usd: Counter<f64>,
    pub diem_credits_consumed: Counter<u64>,
}

impl MetricsCollector {
    pub fn new(m: &Meter) -> Self {
        Self {
            cache_hit_rate:        m.f64_gauge("bardo_cache_hit_rate").build(),
            cache_entries:         m.u64_gauge("bardo_cache_entries").build(),
            cache_savings_usd:     m.f64_counter("bardo_cache_savings_usd").build(),
            cache_stale_served:    m.u64_counter("bardo_cache_stale_served").build(),
            prefix_cache_hit_rate: m.f64_gauge("bardo_prefix_cache_hit_rate").build(),
            cache_invalidations:   m.u64_counter("bardo_cache_invalidations").build(),
            spread_earned_total:   m.f64_counter("bardo_spread_earned_total").build(),
            blockrun_cost_total:   m.f64_counter("bardo_blockrun_cost_total").build(),
            user_savings_total:    m.f64_counter("bardo_user_savings_total").build(),
            spread_pct_effective:  m.f64_gauge("bardo_spread_pct_effective").build(),
            fallback_requests:     m.u64_counter("bardo_fallback_requests").build(),
            pipeline_duration_ms: std::array::from_fn(|i| {
                m.f64_histogram(format!("bardo_pipeline_l{}_duration_ms", i + 1)).build()
            }),
            pipeline_total_duration_ms: m.f64_histogram("bardo_pipeline_total_duration_ms").build(),
            pipeline_profile:      m.u64_counter("bardo_pipeline_profile").build(),
            batch_api_savings_usd: m.f64_counter("bardo_batch_api_savings_usd").build(),
            diem_credits_consumed: m.u64_counter("bardo_diem_credits_consumed").build(),
        }
    }
}

/// Wire tracing_subscriber + OTEL exporters. Call once at startup.
pub fn init_telemetry(endpoint: &str) -> SdkMeterProvider {
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(
            opentelemetry_otlp::MetricExporter::builder()
                .with_tonic().with_endpoint(endpoint).build()
                .expect("OTLP metric exporter init failed"),
        ).build();
    global::set_meter_provider(provider.clone());
    let tracer = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic().with_endpoint(endpoint).build()
        .expect("OTLP span exporter init failed");
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    provider
}
```

> **Extended:** OTEL trace tree, quality evaluation pipeline, arena model evaluation (Phase 2+), cost dashboard visualizations -- see [../../prd2-extended/12-inference/08-observability-extended.md](../../prd2-extended/12-inference/08-observability-extended.md)

---

## Event Fabric integration

The inference gateway emits `GolemEvent` variants to the Event Fabric (see `01b-runtime-infrastructure.md` section 3 for the `tokio::broadcast` ring buffer). Every inference state transition becomes a typed, serializable event consumed by surfaces (TUI, web, Telegram, Discord) without cross-referencing other events.

### Inference events emitted

| Event | Trigger | Key fields |
|---|---|---|
| `InferenceStart` | Request enters the pipeline | `model`, `tier`, `estimated_input_tokens`, `pipeline_profile` |
| `InferenceToken` | Streaming chunk received | `model`, `tier`, `token_count` (cumulative) |
| `InferenceEnd` | LLM call completes | `input_tokens`, `output_tokens`, `cache_read_tokens`, `cost_usd`, `latency_ms` |
| `CacheHit` | Semantic or hash cache hit | `cache_layer` (L2_semantic, L3_hash), `similarity`, `savings_usd` |
| `ProviderFallback` | Primary provider fails | `failed_provider`, `fallback_provider`, `error` |

These events carry `CamelCase` variant names and `snake_case` fields in the wire format, tagged with `#[serde(tag = "type")]`. Each is self-contained: an `InferenceEnd` event includes model, tier, token counts, cost, and latency in a single payload.

### Event emission pattern

```rust
// crates/bardo-telemetry/src/events.rs

/// Emit an inference event to the Event Fabric.
/// Zero cost when no subscriber exists (subscriber count check first).
pub fn emit_inference_event(
    fabric: &EventFabric,
    golem_id: &str,
    tick: u64,
    event: GolemEvent,
) {
    // Only serialize when someone is listening
    if fabric.subscriber_count() > 0 {
        fabric.emit(event);
    }
}
```

Events feed directly into the OTEL metrics collector: every `InferenceEnd` event updates `bardo_pipeline_total_duration_ms`, `bardo_cache_hit_rate`, and cost counters.

---

## Cache metrics

Prometheus-compatible, with alert thresholds:

| Metric | Type | Alert threshold |
|---|---|---|
| `bardo_cache_hit_rate` | Gauge | <30% for 15 min |
| `bardo_cache_entries` | Gauge | >maxEntries \* 0.95 |
| `bardo_cache_savings_usd` | Counter | -- |
| `bardo_cache_stale_served` | Counter | >0 |
| `bardo_prefix_cache_hit_rate` | Gauge | <50% for 15 min |
| `bardo_cache_invalidations` | Counter | Spike detection |

Anthropic's Claude Code team declares internal SEV incidents when prompt cache hit rates drop. The gateway applies the same discipline -- `bardo_prefix_cache_hit_rate` below 50% triggers alerts because prompt assembly has broken cache alignment.

---

## Revenue metrics

> **Moved to [../revenue-model.md](../revenue-model.md) Section 3.2 (Inference Gateway, Revenue Metrics).**

---

## Pipeline profiling metrics

Per-layer latency tracking for the 8-layer pipeline. Targets assume a Rust gateway with 4+ cores:

| Metric | Type | Alert threshold | Performance target |
|---|---|---|---|
| `bardo_pipeline_l1_duration_ms` | Histogram | >5ms | <1ms |
| `bardo_pipeline_l2_duration_ms` | Histogram | >20ms | <5ms (fastembed-rs + HNSW) |
| `bardo_pipeline_l3_duration_ms` | Histogram | >1ms | <0.1ms (DashMap lookup) |
| `bardo_pipeline_l4_duration_ms` | Histogram | >5ms | <1ms |
| `bardo_pipeline_l5_duration_ms` | Histogram | >3000ms | 200-2000ms (conditional) |
| `bardo_pipeline_l6_duration_ms` | Histogram | >5ms | <1ms |
| `bardo_pipeline_l7_duration_ms` | Histogram | >5ms | <1ms (compiled regex) |
| `bardo_pipeline_l8_duration_ms` | Histogram | >15ms | <8ms (DeBERTa INT8) |
| `bardo_pipeline_total_duration_ms` | Histogram | >50ms | <12ms (no L5, parallel) |
| `bardo_pipeline_profile` | Counter | -- | -- |
| `bardo_batch_api_savings_usd` | Counter | -- | -- |
| `bardo_diem_credits_consumed` | Counter | -- | -- |

### Performance targets by profile

| Profile | P95 target | Layers active | When used |
|---|---|---|---|
| Minimal | <1ms | L3 only | T0 heartbeat ticks |
| Fast | <15ms | L1, L3, L6 | Golem internal calls |
| Standard | <40ms | L1-L6 | User-facing requests |
| Full | <65ms | L1-L8 | Private/high-security requests |

### Concurrency targets

| Concurrent requests | P95 target |
|---|---|
| 1 | 12ms |
| 10 | 13ms |
| 50 | 15ms |
| 100 | 18ms |
| 500 | 25ms |

Tokio's work-stealing scheduler + rayon for data parallelism keep latency flat under load. Unlike TypeScript gateways (which degrade above ~50 concurrent ONNX calls), the Rust gateway batches concurrent embedding and DeBERTa inference across all cores.

---

## Cross-references

| Topic | Document | What it covers |
|---|---|---|
| Audit log schema | [07-safety.md](07-safety.md) | PII detection, prompt injection defense, and the audit log entry structure for every inference request |
| Revenue model | [03-economics.md](03-economics.md) | x402 spread revenue model, per-tenant cost attribution formulas, and infrastructure cost projections |
| Analytics endpoints | [09-api.md](09-api.md) | API reference with 33 endpoints including GET /v1/analytics/spend and spend breakdown queries |
| Cache architecture | [02-caching.md](02-caching.md) | Three-layer cache stack (hash, semantic, prefix) with regime-aware invalidation and hit rate metrics |
| Privacy and trust | [11-privacy-trust.md](11-privacy-trust.md) | Cryptographic audit trail with hash chains and Merkle tree anchoring for provenance verification |
| Event Fabric | [../01-golem/01b-runtime-infrastructure.md](../01-golem/01b-runtime-infrastructure.md) | The runtime's event streaming infrastructure that carries GolemEvents from subsystems to TUI and telemetry consumers |
| Rust implementation | [14-rust-implementation.md](14-rust-implementation.md) | 10-crate Rust workspace including bardo-telemetry crate for tracing and OpenTelemetry metrics |

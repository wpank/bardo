//! Shared application state.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::pricing::PricingTable;
use crate::providers::Provider;
use dashmap::DashMap;
use serde::Serialize;

use tokio::sync::{Semaphore, broadcast};

use crate::cache::ResponseCache;
use crate::session::SessionCost;
use crate::tools::ToolTracker;

/// Shared gateway state accessible from all handlers.
#[derive(Clone)]
pub struct AppState {
    /// API key required for authenticated endpoints.
    pub api_key: String,
    /// Anthropic API key (kept for legacy path and `AnthropicProvider`).
    pub anthropic_api_key: String,
    /// OpenAI API key (optional).
    pub openai_api_key: Option<String>,
    /// OpenRouter API key (optional).
    pub openrouter_api_key: Option<String>,
    /// Ordered provider list (first match wins on `resolve_provider`).
    pub providers: Vec<Arc<dyn Provider>>,
    /// HTTP client for provider requests.
    pub http: reqwest::Client,
    /// Async LRU + TTL response cache (moka-backed).
    pub cache: Arc<ResponseCache>,
    /// Model pricing table (exact-match HashMap + substring fallback).
    pub pricing: PricingTable,
    /// Gateway bind address (for logging).
    pub bind_addr: String,
    /// Running statistics.
    pub stats: Arc<GatewayStats>,
    /// Per-session cost tracking.
    pub sessions: Arc<DashMap<String, SessionCost>>,
    /// Per-session tool usage tracker for pruning.
    pub tool_tracker: Arc<ToolTracker>,
    /// Concurrency limiter (None = unlimited).
    pub concurrency: Option<Arc<Semaphore>>,
    /// Broadcast channel for real-time stats events (dashboard WebSocket).
    pub stats_tx: broadcast::Sender<StatsEvent>,
    /// Batch API manager (None if batch not enabled).
    pub batch_manager: Option<Arc<crate::batch::BatchManager>>,
    /// Semantic cache (L2) — catches similar but non-identical requests.
    pub semantic_cache: Arc<crate::semantic_cache::SemanticCache>,
    /// In-flight request coalescing — waiters subscribe to a broadcast for the same hash.
    pub inflight: Arc<DashMap<[u8; 32], tokio::sync::broadcast::Sender<Result<bytes::Bytes, String>>>>,
}

impl AppState {
    /// Return the first provider that accepts `model`, or `None`.
    ///
    /// Providers are checked in priority order (index 0 = highest priority).
    pub fn resolve_provider(&self, model: &str) -> Option<&Arc<dyn Provider>> {
        self.providers.iter().find(|p| p.accepts(model))
    }

    /// Look up pricing for a model. O(1) for exact matches, O(n) substring fallback.
    pub fn price_for_model(&self, model: &str) -> (f64, f64) {
        self.pricing.price_for_model(model)
    }
}

/// Atomic counters for gateway statistics.
pub struct GatewayStats {
    pub total_requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub total_input_tokens: AtomicU64,
    pub total_output_tokens: AtomicU64,
    /// Cost in micro-USD (multiply by 1e-6 to get USD). Avoids floating point atomics.
    pub total_cost_micro_usd: AtomicU64,
    /// Naive cost (what it would have cost without caching) in micro-USD.
    pub total_naive_cost_micro_usd: AtomicU64,
    /// Per-model request counts.
    pub model_counts: DashMap<String, u64>,
    /// Monotonic sequence counter for stats events.
    pub event_seq: AtomicU64,
}

impl GatewayStats {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
            total_cost_micro_usd: AtomicU64::new(0),
            total_naive_cost_micro_usd: AtomicU64::new(0),
            model_counts: DashMap::new(),
            event_seq: AtomicU64::new(0),
        }
    }

    pub fn record_request(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
        naive_cost_usd: f64,
        is_cache_hit: bool,
    ) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
        self.total_input_tokens
            .fetch_add(input_tokens, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(output_tokens, Ordering::Relaxed);

        let cost_micro = (cost_usd * 1_000_000.0) as u64;
        self.total_cost_micro_usd
            .fetch_add(cost_micro, Ordering::Relaxed);

        let naive_micro = (naive_cost_usd * 1_000_000.0) as u64;
        self.total_naive_cost_micro_usd
            .fetch_add(naive_micro, Ordering::Relaxed);

        *self.model_counts.entry(model.to_string()).or_insert(0) += 1;
    }

    pub fn to_json(&self) -> StatsResponse {
        let total_cost = self.total_cost_micro_usd.load(Ordering::Relaxed) as f64 / 1_000_000.0;
        let total_naive =
            self.total_naive_cost_micro_usd.load(Ordering::Relaxed) as f64 / 1_000_000.0;

        let mut models = Vec::new();
        for entry in self.model_counts.iter() {
            models.push(ModelStat {
                model: entry.key().clone(),
                requests: *entry.value(),
            });
        }
        models.sort_by(|a, b| b.requests.cmp(&a.requests));

        StatsResponse {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_hit_rate: {
                let total = self.total_requests.load(Ordering::Relaxed);
                if total == 0 {
                    0.0
                } else {
                    self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
                }
            },
            total_input_tokens: self.total_input_tokens.load(Ordering::Relaxed),
            total_output_tokens: self.total_output_tokens.load(Ordering::Relaxed),
            total_cost_usd: total_cost,
            total_naive_cost_usd: total_naive,
            total_savings_usd: total_naive - total_cost,
            savings_rate: if total_naive > 0.0 {
                (total_naive - total_cost) / total_naive
            } else {
                0.0
            },
            cache_entries: 0, // filled by handler
            models,
        }
    }
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub total_naive_cost_usd: f64,
    pub total_savings_usd: f64,
    pub savings_rate: f64,
    pub cache_entries: usize,
    pub models: Vec<ModelStat>,
}

#[derive(Serialize)]
pub struct ModelStat {
    pub model: String,
    pub requests: u64,
}

/// A single per-request event broadcast to dashboard WebSocket clients.
#[derive(Clone, Debug, Serialize)]
pub struct StatsEvent {
    pub seq: u64,
    pub timestamp_ms: u64,
    pub model: String,
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_create_tokens: u64,
    /// OpenAI cached input tokens (subset of input_tokens, 50% discount).
    pub openai_cached_tokens: u64,
    /// o-series reasoning tokens (subset of output_tokens).
    pub reasoning_tokens: u64,
    /// Anthropic extended thinking tokens (charged at output rate).
    pub thinking_tokens: u64,
    pub cost_usd: f64,
    pub naive_cost_usd: f64,
    pub savings_usd: f64,
    pub cache_hit: bool,
    pub prefix_cache_warm: bool,
    pub is_batch: bool,
    pub elapsed_ms: u64,
    pub streaming: bool,
    pub session_id: Option<String>,
}

/// A cached response with metadata.
#[derive(Clone)]
pub struct CachedResponse {
    /// Response body bytes.
    pub body: bytes::Bytes,
    /// Content-Type header.
    pub content_type: String,
    /// Cost headers from the original response.
    pub cost_usd: f64,
    /// Model that generated the response.
    pub model: String,
    /// When this entry was cached.
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

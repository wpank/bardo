//! Shared application state.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::pricing::ModelPricing;
use dashmap::DashMap;
use serde::Serialize;

use crate::cache::ResponseCache;
use crate::session::SessionCost;
use crate::tools::ToolTracker;

/// Shared gateway state accessible from all handlers.
#[derive(Clone)]
pub struct AppState {
    /// API key required for authenticated endpoints.
    pub api_key: String,
    /// Anthropic API key for forwarding requests.
    pub anthropic_api_key: String,
    /// OpenAI API key for forwarding requests.
    pub openai_api_key: Option<String>,
    /// HTTP client for provider requests.
    pub http: reqwest::Client,
    /// Async LRU + TTL response cache (moka-backed).
    pub cache: Arc<ResponseCache>,
    /// Model pricing table.
    pub pricing: Vec<ModelPricing>,
    /// Gateway bind address (for logging).
    pub bind_addr: String,
    /// Running statistics.
    pub stats: Arc<GatewayStats>,
    /// Per-session cost tracking.
    pub sessions: Arc<DashMap<String, SessionCost>>,
    /// Per-session tool usage tracker for pruning.
    pub tool_tracker: Arc<ToolTracker>,
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
        }
    }

    pub fn record_request(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
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

        let micro = (cost_usd * 1_000_000.0) as u64;
        self.total_cost_micro_usd
            .fetch_add(micro, Ordering::Relaxed);
        // Naive cost is always the non-cached cost (even on cache hits)
        self.total_naive_cost_micro_usd
            .fetch_add(micro, Ordering::Relaxed);

        *self.model_counts.entry(model.to_string()).or_insert(0) += 1;
    }

    pub fn record_cache_hit_savings(&self, saved_usd: f64) {
        // On a cache hit, actual cost is 0 but naive cost was recorded.
        // We need to NOT add to total_cost, so just record the savings in naive.
        let micro = (saved_usd * 1_000_000.0) as u64;
        self.total_naive_cost_micro_usd
            .fetch_add(micro, Ordering::Relaxed);
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

impl AppState {
    /// Look up pricing for a model.
    pub fn price_for_model(&self, model: &str) -> (f64, f64) {
        self.pricing
            .iter()
            .find(|p| model.contains(&p.model) || p.model.contains(model))
            .map(|p| (p.input_per_m, p.output_per_m))
            .unwrap_or((3.0, 15.0)) // default to Sonnet pricing
    }
}

//! Per-session cost tracking.
//!
//! Each session (identified by `X-Mori-Session-Id`) accumulates token usage,
//! cost, and savings across all requests in that session.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Accumulated cost data for a single session.
#[derive(Clone, Debug, Serialize)]
pub struct SessionCost {
    /// Session identifier (from `X-Mori-Session-Id` header).
    pub session_id: String,
    /// When this session was first seen.
    pub created_at: DateTime<Utc>,
    /// Total requests in this session.
    pub total_requests: u64,
    /// Total cost in USD (actual spend).
    pub total_cost_usd: f64,
    /// Total savings from caching and tier routing.
    pub total_savings_usd: f64,
    /// Per-model breakdown.
    pub by_model: HashMap<String, ModelSessionCost>,
    /// Per-provider cost totals.
    pub by_provider: HashMap<String, f64>,
}

/// Per-model cost breakdown within a session.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ModelSessionCost {
    /// Number of requests using this model.
    pub requests: u64,
    /// Total input tokens for this model.
    pub input_tokens: u64,
    /// Total output tokens for this model.
    pub output_tokens: u64,
    /// Total cost in USD for this model.
    pub cost_usd: f64,
}

impl SessionCost {
    /// Create a new zeroed session.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            created_at: Utc::now(),
            total_requests: 0,
            total_cost_usd: 0.0,
            total_savings_usd: 0.0,
            by_model: HashMap::new(),
            by_provider: HashMap::new(),
        }
    }

    /// Record a completed request's cost and token usage.
    pub fn record(
        &mut self,
        model: &str,
        provider: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
        savings_usd: f64,
    ) {
        self.total_requests += 1;
        self.total_cost_usd += cost_usd;
        self.total_savings_usd += savings_usd;

        let model_cost = self.by_model.entry(model.to_string()).or_default();
        model_cost.requests += 1;
        model_cost.input_tokens += input_tokens;
        model_cost.output_tokens += output_tokens;
        model_cost.cost_usd += cost_usd;

        *self.by_provider.entry(provider.to_string()).or_insert(0.0) += cost_usd;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_is_zeroed() {
        let s = SessionCost::new("test-123".into());
        assert_eq!(s.session_id, "test-123");
        assert_eq!(s.total_requests, 0);
        assert!((s.total_cost_usd - 0.0).abs() < f64::EPSILON);
        assert!(s.by_model.is_empty());
        assert!(s.by_provider.is_empty());
    }

    #[test]
    fn record_accumulates() {
        let mut s = SessionCost::new("sess".into());
        s.record("claude-sonnet-4", "anthropic", 1000, 500, 0.01, 0.005);
        s.record("claude-sonnet-4", "anthropic", 2000, 1000, 0.02, 0.0);
        s.record("gpt-4o", "openai", 500, 200, 0.005, 0.0);

        assert_eq!(s.total_requests, 3);
        assert!((s.total_cost_usd - 0.035).abs() < 1e-10);
        assert!((s.total_savings_usd - 0.005).abs() < 1e-10);

        let sonnet = s.by_model.get("claude-sonnet-4");
        assert!(sonnet.is_some());
        let default_cost = ModelSessionCost::default();
        let sonnet = sonnet.unwrap_or(&default_cost);
        assert_eq!(sonnet.requests, 2);
        assert_eq!(sonnet.input_tokens, 3000);

        assert_eq!(s.by_provider.len(), 2);
    }
}

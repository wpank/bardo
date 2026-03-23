//! Cost estimation for Mori-as-a-Service proposals.
//!
//! Given a description of work decomposed into tasks with model tier assignments,
//! estimates the total USD cost including inference, compute overhead, and spread.
//!
//! Pricing pulled from the gateway's March 2026 rate table:
//! - Opus:   $5 / $25 / $0.50  (input / output / cached-input per 1M tokens)
//! - Sonnet: $3 / $15 / $0.30
//! - Haiku:  $1 / $5  / $0.10

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Model tier
// ---------------------------------------------------------------------------

/// Model tier for routing tasks to LLM providers.
///
/// Each tier maps to a specific cost profile. The mapping is intentionally
/// coarse: three tiers cover the realistic range of agentic coding tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// Simple tasks: auto-fix, classification, config edits.
    Haiku,
    /// Moderate tasks: standard implementation, review, test writing.
    Sonnet,
    /// Complex tasks: cross-crate refactors, architecture, security audits.
    Opus,
}

impl ModelTier {
    /// Return the pricing profile for this tier.
    pub fn pricing(&self) -> ModelPricing {
        match self {
            ModelTier::Haiku => ModelPricing {
                input_per_m: 1.0,
                output_per_m: 5.0,
                cached_input_per_m: 0.10,
            },
            ModelTier::Sonnet => ModelPricing {
                input_per_m: 3.0,
                output_per_m: 15.0,
                cached_input_per_m: 0.30,
            },
            ModelTier::Opus => ModelPricing {
                input_per_m: 5.0,
                output_per_m: 25.0,
                cached_input_per_m: 0.50,
            },
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            ModelTier::Haiku => "haiku",
            ModelTier::Sonnet => "sonnet",
            ModelTier::Opus => "opus",
        }
    }
}

// ---------------------------------------------------------------------------
// Pricing
// ---------------------------------------------------------------------------

/// Per-million-token pricing for a model tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelPricing {
    /// USD per 1M input tokens (non-cached).
    pub input_per_m: f64,
    /// USD per 1M output tokens.
    pub output_per_m: f64,
    /// USD per 1M cached input tokens.
    pub cached_input_per_m: f64,
}

// ---------------------------------------------------------------------------
// Task-level estimation
// ---------------------------------------------------------------------------

/// Inputs for estimating a single task's cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEstimate {
    pub tier: ModelTier,
    /// Expected input token consumption for this task.
    pub input_tokens: u64,
    /// Expected output token consumption for this task.
    pub output_tokens: u64,
    /// Fraction of input tokens expected to hit prompt cache (0.0 - 1.0).
    /// Typical values: 0.4 for first plan in a wave, 0.7+ for subsequent
    /// plans that share enrichment context.
    pub cache_hit_rate: f64,
}

/// Estimate the raw inference cost for a single task in USD.
///
/// Splits input tokens into cached and uncached portions using `cache_hit_rate`,
/// then prices each portion at the tier's rates.
///
/// Does NOT include spread or compute overhead; those are applied at the plan level.
pub fn estimate_task_cost(
    tier: ModelTier,
    estimated_input_tokens: u64,
    estimated_output_tokens: u64,
    cache_hit_rate: f64,
) -> f64 {
    let pricing = tier.pricing();
    let rate = cache_hit_rate.clamp(0.0, 1.0);

    let total_input = estimated_input_tokens as f64;
    let cached_input = total_input * rate;
    let uncached_input = total_input - cached_input;

    let input_cost = uncached_input * pricing.input_per_m / 1_000_000.0
        + cached_input * pricing.cached_input_per_m / 1_000_000.0;

    let output_cost = estimated_output_tokens as f64 * pricing.output_per_m / 1_000_000.0;

    input_cost + output_cost
}

// ---------------------------------------------------------------------------
// Plan-level estimation
// ---------------------------------------------------------------------------

/// Aggregated cost estimate for a plan (a collection of tasks).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlanCostEstimate {
    /// Total LLM inference cost (sum of all task costs).
    pub inference: f64,
    /// Estimated non-inference compute cost (CI, compilation, deployment infra).
    /// Heuristic: 15% of inference cost.
    pub compute: f64,
    /// inference + compute, before spread.
    pub subtotal: f64,
    /// Final cost after applying the spread margin.
    pub total: f64,
}

/// Fraction of inference cost attributed to compute overhead.
const COMPUTE_FRACTION: f64 = 0.15;

/// Default spread percentage (20%).
pub const DEFAULT_SPREAD_PCT: f64 = 0.20;

/// Estimate the total cost for a plan given its constituent task estimates.
///
/// `spread_pct` is the gateway spread applied on top of raw costs (e.g. 0.20 = 20%).
pub fn estimate_plan_cost(tasks: &[TaskEstimate], spread_pct: f64) -> PlanCostEstimate {
    let inference: f64 = tasks
        .iter()
        .map(|t| estimate_task_cost(t.tier, t.input_tokens, t.output_tokens, t.cache_hit_rate))
        .sum();

    let compute = inference * COMPUTE_FRACTION;
    let subtotal = inference + compute;
    let total = subtotal * (1.0 + spread_pct);

    PlanCostEstimate {
        inference,
        compute,
        subtotal,
        total,
    }
}

// ---------------------------------------------------------------------------
// Token estimation heuristics
// ---------------------------------------------------------------------------

/// Heuristic estimates for token consumption per task, by model tier.
///
/// These are median values observed across real Mori builds. The range is wide
/// because task scope varies, but these give a reasonable center for proposal math.
pub struct TokenHeuristics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_hit_rate: f64,
}

impl TokenHeuristics {
    /// Return median token estimates for a given tier.
    ///
    /// Haiku tasks are short: classification, auto-fix, config edits.
    /// Sonnet tasks are standard implementations: write a module, add tests.
    /// Opus tasks are heavy: cross-crate refactors, architectural decisions.
    pub fn for_tier(tier: ModelTier) -> Self {
        match tier {
            ModelTier::Haiku => TokenHeuristics {
                input_tokens: 8_000,
                output_tokens: 2_000,
                cache_hit_rate: 0.60,
            },
            ModelTier::Sonnet => TokenHeuristics {
                input_tokens: 40_000,
                output_tokens: 12_000,
                cache_hit_rate: 0.50,
            },
            ModelTier::Opus => TokenHeuristics {
                input_tokens: 60_000,
                output_tokens: 18_000,
                cache_hit_rate: 0.45,
            },
        }
    }

    /// Convert to a `TaskEstimate` for cost calculation.
    pub fn to_estimate(&self, tier: ModelTier) -> TaskEstimate {
        TaskEstimate {
            tier,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cache_hit_rate: self.cache_hit_rate,
        }
    }
}

// ---------------------------------------------------------------------------
// Model tier distribution
// ---------------------------------------------------------------------------

/// Describes how tasks within a milestone distribute across model tiers.
///
/// For a 10-task milestone, a typical distribution might be:
/// - 2 haiku (config, classification)
/// - 6 sonnet (implementation, tests)
/// - 2 opus (architecture, complex integration)
pub struct TierDistribution {
    pub haiku_frac: f64,
    pub sonnet_frac: f64,
    pub opus_frac: f64,
}

impl TierDistribution {
    /// Default distribution for a standard implementation milestone.
    pub fn standard() -> Self {
        TierDistribution {
            haiku_frac: 0.15,
            sonnet_frac: 0.65,
            opus_frac: 0.20,
        }
    }

    /// Distribution skewed toward simpler work (CLI tools, CRUD).
    pub fn simple() -> Self {
        TierDistribution {
            haiku_frac: 0.25,
            sonnet_frac: 0.60,
            opus_frac: 0.15,
        }
    }

    /// Distribution for complex, architecture-heavy work.
    pub fn complex() -> Self {
        TierDistribution {
            haiku_frac: 0.10,
            sonnet_frac: 0.50,
            opus_frac: 0.40,
        }
    }

    /// Generate task estimates for `task_count` tasks using this distribution.
    pub fn to_task_estimates(&self, task_count: u32) -> Vec<TaskEstimate> {
        let n = task_count as f64;
        let haiku_count = (n * self.haiku_frac).round() as u32;
        let opus_count = (n * self.opus_frac).round() as u32;
        // Sonnet gets the remainder to avoid rounding drift.
        let sonnet_count = task_count
            .saturating_sub(haiku_count)
            .saturating_sub(opus_count);

        let mut estimates = Vec::with_capacity(task_count as usize);

        for _ in 0..haiku_count {
            estimates
                .push(TokenHeuristics::for_tier(ModelTier::Haiku).to_estimate(ModelTier::Haiku));
        }
        for _ in 0..sonnet_count {
            estimates
                .push(TokenHeuristics::for_tier(ModelTier::Sonnet).to_estimate(ModelTier::Sonnet));
        }
        for _ in 0..opus_count {
            estimates.push(TokenHeuristics::for_tier(ModelTier::Opus).to_estimate(ModelTier::Opus));
        }

        estimates
    }
}

// ---------------------------------------------------------------------------
// Convenience: estimate a milestone from task count + distribution
// ---------------------------------------------------------------------------

/// Estimate the cost of a milestone given a task count, tier distribution, and spread.
///
/// This is the primary entry point used by `ProposalEngine` when it doesn't
/// have per-task detail and is working from heuristic classification alone.
pub fn estimate_milestone_cost(
    task_count: u32,
    distribution: &TierDistribution,
    spread_pct: f64,
) -> PlanCostEstimate {
    let estimates = distribution.to_task_estimates(task_count);
    estimate_plan_cost(&estimates, spread_pct)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haiku_task_cost_is_cheap() {
        let cost = estimate_task_cost(ModelTier::Haiku, 8_000, 2_000, 0.60);
        // Uncached input: 3200 tokens * $1/1M = $0.0032
        // Cached input:   4800 tokens * $0.10/1M = $0.00048
        // Output:         2000 tokens * $5/1M = $0.01
        // Total: ~$0.01368
        assert!(cost > 0.01, "haiku cost too low: {cost}");
        assert!(cost < 0.05, "haiku cost too high: {cost}");
    }

    #[test]
    fn sonnet_task_cost_is_moderate() {
        let cost = estimate_task_cost(ModelTier::Sonnet, 40_000, 12_000, 0.50);
        // Uncached input: 20000 * $3/1M = $0.06
        // Cached input:   20000 * $0.30/1M = $0.006
        // Output:         12000 * $15/1M = $0.18
        // Total: ~$0.246
        assert!(cost > 0.20, "sonnet cost too low: {cost}");
        assert!(cost < 0.35, "sonnet cost too high: {cost}");
    }

    #[test]
    fn opus_task_cost_is_expensive() {
        let cost = estimate_task_cost(ModelTier::Opus, 60_000, 18_000, 0.45);
        // Uncached input: 33000 * $5/1M = $0.165
        // Cached input:   27000 * $0.50/1M = $0.0135
        // Output:         18000 * $25/1M = $0.45
        // Total: ~$0.6285
        assert!(cost > 0.55, "opus cost too low: {cost}");
        assert!(cost < 0.75, "opus cost too high: {cost}");
    }

    #[test]
    fn plan_cost_includes_compute_and_spread() {
        let tasks = vec![
            TaskEstimate {
                tier: ModelTier::Sonnet,
                input_tokens: 40_000,
                output_tokens: 12_000,
                cache_hit_rate: 0.50,
            },
            TaskEstimate {
                tier: ModelTier::Sonnet,
                input_tokens: 40_000,
                output_tokens: 12_000,
                cache_hit_rate: 0.50,
            },
        ];
        let estimate = estimate_plan_cost(&tasks, 0.20);

        // Two sonnet tasks ~ $0.246 each = ~$0.492 inference
        assert!(estimate.inference > 0.45);
        assert!(estimate.inference < 0.55);

        // Compute = 15% of inference
        let expected_compute = estimate.inference * 0.15;
        assert!((estimate.compute - expected_compute).abs() < 0.001);

        // Total = subtotal * 1.20
        let expected_total = estimate.subtotal * 1.20;
        assert!((estimate.total - expected_total).abs() < 0.001);

        // Total should be meaningfully higher than inference alone
        assert!(estimate.total > estimate.inference * 1.30);
    }

    #[test]
    fn milestone_cost_scales_with_task_count() {
        let dist = TierDistribution::standard();
        let small = estimate_milestone_cost(5, &dist, 0.20);
        let large = estimate_milestone_cost(20, &dist, 0.20);

        // 4x the tasks should produce roughly 4x the cost.
        let ratio = large.total / small.total;
        assert!(ratio > 3.0, "cost ratio too low: {ratio}");
        assert!(ratio < 5.0, "cost ratio too high: {ratio}");
    }

    #[test]
    fn zero_cache_hit_rate_costs_more() {
        let with_cache = estimate_task_cost(ModelTier::Sonnet, 40_000, 12_000, 0.50);
        let no_cache = estimate_task_cost(ModelTier::Sonnet, 40_000, 12_000, 0.0);
        assert!(no_cache > with_cache, "no cache should cost more");
    }

    #[test]
    fn full_cache_hit_rate_costs_less() {
        let partial = estimate_task_cost(ModelTier::Opus, 60_000, 18_000, 0.45);
        let full = estimate_task_cost(ModelTier::Opus, 60_000, 18_000, 1.0);
        assert!(full < partial, "full cache should cost less");
    }

    #[test]
    fn tier_distribution_sums_to_task_count() {
        let dist = TierDistribution::standard();
        let estimates = dist.to_task_estimates(10);
        assert_eq!(estimates.len(), 10);

        let dist = TierDistribution::complex();
        let estimates = dist.to_task_estimates(7);
        assert_eq!(estimates.len(), 7);
    }

    #[test]
    fn complex_distribution_costs_more_than_simple() {
        let simple = estimate_milestone_cost(10, &TierDistribution::simple(), 0.20);
        let complex = estimate_milestone_cost(10, &TierDistribution::complex(), 0.20);
        assert!(
            complex.total > simple.total,
            "complex should cost more: {:.2} vs {:.2}",
            complex.total,
            simple.total
        );
    }
}

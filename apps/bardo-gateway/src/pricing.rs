//! Gateway-local model pricing table.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Pricing per model (USD per 1M tokens).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model identifier (exact or substring match).
    pub model: String,
    /// Input token price per 1M tokens (full price, non-cached).
    pub input_per_m: f64,
    /// Output token price per 1M tokens.
    pub output_per_m: f64,
    /// Cached input token price per 1M tokens.
    /// Anthropic: 10% of input. OpenAI: 50% of input.
    /// None = use provider default (Anthropic 10%, OpenAI 50%).
    #[serde(default)]
    pub cached_input_per_m: Option<f64>,
    /// Reasoning token price per 1M tokens (o-series models).
    /// None = same as output price.
    #[serde(default)]
    pub reasoning_per_m: Option<f64>,
}

/// Fast pricing lookup: exact match HashMap + substring fallback Vec.
#[derive(Clone, Debug)]
pub struct PricingTable {
    exact: HashMap<String, usize>,
    entries: Vec<ModelPricing>,
}

impl PricingTable {
    pub fn new(entries: Vec<ModelPricing>) -> Self {
        let exact = entries
            .iter()
            .enumerate()
            .map(|(i, p)| (p.model.clone(), i))
            .collect();
        Self { exact, entries }
    }

    /// Look up full pricing for a model. O(1) exact, O(n) substring fallback.
    pub fn lookup(&self, model: &str) -> &ModelPricing {
        if let Some(&idx) = self.exact.get(model) {
            return &self.entries[idx];
        }
        self.entries
            .iter()
            .find(|p| model.contains(&p.model) || p.model.contains(model))
            .unwrap_or_else(|| default_model_pricing())
    }

    /// Shorthand for (input_per_m, output_per_m).
    pub fn price_for_model(&self, model: &str) -> (f64, f64) {
        let p = self.lookup(model);
        (p.input_per_m, p.output_per_m)
    }
}

/// Fallback when no model matches.
fn default_model_pricing() -> &'static ModelPricing {
    use std::sync::LazyLock;
    static P: LazyLock<ModelPricing> = LazyLock::new(|| ModelPricing {
        model: String::new(),
        input_per_m: 3.0,
        output_per_m: 15.0,
        cached_input_per_m: None,
        reasoning_per_m: None,
    });
    &P
}

/// Default pricing table (March 2026 rates).
pub fn default_pricing() -> Vec<ModelPricing> {
    vec![
        // ── Anthropic ──
        // Prices from https://platform.claude.com/docs/en/about-claude/pricing
        // Cached input = 10% of input price (0.1x). Cache write = 1.25x input.
        ModelPricing {
            model: "claude-opus-4-6".into(),
            input_per_m: 5.0,
            output_per_m: 25.0,
            cached_input_per_m: Some(0.5), // 10% of 5
            reasoning_per_m: None,
        },
        ModelPricing {
            model: "claude-sonnet-4-20250514".into(),
            input_per_m: 3.0,
            output_per_m: 15.0,
            cached_input_per_m: Some(0.3),
            reasoning_per_m: None,
        },
        ModelPricing {
            model: "claude-sonnet-4".into(),
            input_per_m: 3.0,
            output_per_m: 15.0,
            cached_input_per_m: Some(0.3),
            reasoning_per_m: None,
        },
        ModelPricing {
            model: "claude-haiku-4-5".into(),
            input_per_m: 1.0,
            output_per_m: 5.0,
            cached_input_per_m: Some(0.1), // 10% of 1
            reasoning_per_m: None,
        },
        ModelPricing {
            model: "claude-haiku-4-5-20251001".into(),
            input_per_m: 1.0,
            output_per_m: 5.0,
            cached_input_per_m: Some(0.1),
            reasoning_per_m: None,
        },
        // ── OpenAI ──
        // Cached input = 50% of input price.
        ModelPricing {
            model: "gpt-4o".into(),
            input_per_m: 2.5,
            output_per_m: 10.0,
            cached_input_per_m: Some(1.25), // 50% of 2.5
            reasoning_per_m: None,
        },
        ModelPricing {
            model: "gpt-4o-mini".into(),
            input_per_m: 0.15,
            output_per_m: 0.6,
            cached_input_per_m: Some(0.075), // 50% of 0.15
            reasoning_per_m: None,
        },
        // ── OpenAI o-series ──
        // Reasoning tokens charged at output rate.
        ModelPricing {
            model: "o3".into(),
            input_per_m: 10.0,
            output_per_m: 40.0,
            cached_input_per_m: Some(5.0), // 50% of 10
            reasoning_per_m: Some(40.0),   // same as output
        },
        ModelPricing {
            model: "o4-mini".into(),
            input_per_m: 1.1,
            output_per_m: 4.4,
            cached_input_per_m: Some(0.55), // 50% of 1.1
            reasoning_per_m: Some(4.4),
        },
    ]
}

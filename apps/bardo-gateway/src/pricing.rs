//! Gateway-local model pricing table.

use serde::{Deserialize, Serialize};

/// Pricing per model (USD per 1M tokens).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model identifier.
    pub model: String,
    /// Input token price per 1M tokens.
    pub input_per_m: f64,
    /// Output token price per 1M tokens.
    pub output_per_m: f64,
}

/// Default pricing table (March 2026 rates).
pub fn default_pricing() -> Vec<ModelPricing> {
    vec![
        ModelPricing {
            model: "claude-opus-4-6".into(),
            input_per_m: 15.0,
            output_per_m: 75.0,
        },
        ModelPricing {
            model: "claude-sonnet-4-20250514".into(),
            input_per_m: 3.0,
            output_per_m: 15.0,
        },
        ModelPricing {
            model: "claude-sonnet-4".into(),
            input_per_m: 3.0,
            output_per_m: 15.0,
        },
        ModelPricing {
            model: "claude-haiku-4-5".into(),
            input_per_m: 0.8,
            output_per_m: 4.0,
        },
        ModelPricing {
            model: "claude-haiku-4-5-20251001".into(),
            input_per_m: 0.8,
            output_per_m: 4.0,
        },
        ModelPricing {
            model: "gpt-4o".into(),
            input_per_m: 2.5,
            output_per_m: 10.0,
        },
        ModelPricing {
            model: "gpt-4o-mini".into(),
            input_per_m: 0.15,
            output_per_m: 0.6,
        },
        ModelPricing {
            model: "o3".into(),
            input_per_m: 10.0,
            output_per_m: 40.0,
        },
        ModelPricing {
            model: "o4-mini".into(),
            input_per_m: 1.1,
            output_per_m: 4.4,
        },
    ]
}

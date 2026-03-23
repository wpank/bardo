//! Pre-request cost estimation for 402 amount calculation.
//!
//! Estimates the total request cost BEFORE the upstream call, using model
//! pricing and input token count with a statistical output multiplier.

use alloy::primitives::U256;
use serde_json::Value;

use crate::pricing::PricingTable;
pub use mpp_core::currency::{usd_from_usdc, usdc_from_usd};

/// Estimate the total request cost in USDC base units (6 decimals).
///
/// Applies a conservative output estimate and the spread multiplier.
/// For Charge intent, over-estimating is safe: the receipt shows actual charge.
pub fn estimate_request_cost(
    model: &str,
    input_tokens: u64,
    pricing: &PricingTable,
    spread_pct: f64,
) -> U256 {
    let p = pricing.lookup(model);

    let input_cost = input_tokens as f64 * p.input_per_m / 1e6;

    let estimated_output = estimate_output_tokens(model, input_tokens);
    let output_cost = estimated_output as f64 * p.output_per_m / 1e6;

    let provider_cost = input_cost + output_cost;
    let total = provider_cost * (1.0 + spread_pct);

    usdc_from_usd(total)
}

/// Estimated provider cost in USD (no spread) for cost breakdown.
pub fn estimate_provider_cost_usd(model: &str, input_tokens: u64, pricing: &PricingTable) -> f64 {
    let p = pricing.lookup(model);
    let input_cost = input_tokens as f64 * p.input_per_m / 1e6;
    let estimated_output = estimate_output_tokens(model, input_tokens);
    let output_cost = estimated_output as f64 * p.output_per_m / 1e6;
    input_cost + output_cost
}

/// Model-specific output token estimation heuristics.
fn estimate_output_tokens(model: &str, input_tokens: u64) -> u64 {
    let base: u64 = if model.contains("haiku") {
        1024
    } else if model.contains("opus") {
        4096
    } else {
        2048
    };
    // Scale slightly with input length (longer context tends to produce longer responses).
    // Cap the scaling contribution at 2048 tokens.
    base + (input_tokens / 20).min(2048)
}

/// Estimate input token count from a request body.
///
/// Uses a character-based heuristic: ~4 characters per token on average.
/// Counts characters in the `messages` array and `system` field.
pub fn estimate_input_tokens(body: &Value) -> u64 {
    let mut chars: u64 = 0;

    // System prompt.
    if let Some(system) = body.get("system") {
        chars += json_char_count(system);
    }

    // Messages array.
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            chars += json_char_count(msg);
        }
    }

    // Tools definitions.
    if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
        for tool in tools {
            chars += json_char_count(tool);
        }
    }

    // ~4 chars per token.
    (chars / 4).max(1)
}

/// Count characters in a JSON value (recursive over strings).
fn json_char_count(val: &Value) -> u64 {
    match val {
        Value::String(s) => s.len() as u64,
        Value::Array(arr) => arr.iter().map(json_char_count).sum(),
        Value::Object(map) => map.values().map(json_char_count).sum(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::default_pricing;

    #[test]
    fn estimate_haiku_cost() {
        let table = PricingTable::new(default_pricing());
        // haiku: $1/M input, $5/M output
        // 10K input tokens = $0.01 input
        // estimated output: 1024 + 500 = 1524 tokens -> $0.00762 output
        // total provider: ~$0.01762
        // with 20% spread: ~$0.02114
        let cost = estimate_request_cost("claude-haiku-4-5", 10_000, &table, 0.20);
        let usd = usd_from_usdc(cost);
        assert!(usd > 0.01, "cost should be > $0.01, got {usd}");
        assert!(usd < 0.10, "cost should be < $0.10, got {usd}");
    }

    #[test]
    fn estimate_opus_cost() {
        let table = PricingTable::new(default_pricing());
        // opus: $5/M input, $25/M output
        // 50K input = $0.25
        // estimated output: 4096 + 2048 = 6144 -> $0.1536
        // provider: ~$0.4036
        // with 20% spread: ~$0.4843
        let cost = estimate_request_cost("claude-opus-4-6", 50_000, &table, 0.20);
        let usd = usd_from_usdc(cost);
        assert!(usd > 0.30, "cost should be > $0.30, got {usd}");
        assert!(usd < 1.00, "cost should be < $1.00, got {usd}");
    }

    #[test]
    fn usdc_roundtrip() {
        let usd = 1.234567;
        let usdc = usdc_from_usd(usd);
        let back = usd_from_usdc(usdc);
        assert!((back - usd).abs() < 0.000001);
    }

    #[test]
    fn estimate_tokens_from_body() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4",
            "system": "You are a helpful assistant.",
            "messages": [
                {"role": "user", "content": "Hello, how are you doing today?"}
            ]
        });
        let tokens = estimate_input_tokens(&body);
        assert!(tokens > 5, "should estimate some tokens, got {tokens}");
        assert!(tokens < 100, "should not overcount, got {tokens}");
    }
}

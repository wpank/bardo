//! JSON key normalization and prompt cache optimization for the gateway.
//!
//! This module has two jobs:
//!
//! 1. **Local hash cache** — `normalize_json_ordering` (key sort) + `strip_variable_content`
//!    (UUID/timestamp removal) + `sort_tools_by_name` (stable array order) improve blake3
//!    cache hit rates by ensuring semantically-identical requests produce identical bytes.
//!
//! 2. **Anthropic server-side prompt cache** — `inject_anthropic_cache_control` converts
//!    the `system` field into an array with `cache_control: {"type":"ephemeral"}` so
//!    Anthropic caches the stable prefix server-side (90% token discount on repeated calls).

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

// ── Variable stripping ─────────────────────────────────────────────

/// Matches common per-request variables embedded in system prompts:
/// - UUID v1-v5 (any variant)
/// - ISO 8601 timestamps (with or without fractional seconds and timezone)
static VARIABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?xi)
        # UUID (v1-v5, any variant)
        [0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}
        |
        # ISO 8601 timestamp  (2024-01-15T10:30:00Z / 2024-01-15T10:30:00.123+05:30)
        \d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?
        ",
    )
    .expect("VARIABLE_RE is valid")
});

/// Replace UUIDs and ISO 8601 timestamps in the `system` prompt with `[VAR]`.
///
/// Called **before** hashing so requests that differ only in per-request IDs
/// embedded in the system prompt produce identical blake3 cache keys.
///
/// Only operates on string-form `system`; no-op if absent or already an array.
pub fn strip_variable_content(body: &mut Value) {
    if let Some(Value::String(system)) = body.get("system").cloned() {
        let stripped = VARIABLE_RE.replace_all(&system, "[VAR]").into_owned();
        body["system"] = Value::String(stripped);
    }
}

// ── Tool ordering ──────────────────────────────────────────────────

/// Stable-sort the `tools` array by tool `name` (ascending, case-sensitive).
///
/// The Anthropic API accepts tools in any order; tool pruning already removes
/// unused tools. Sorting ensures two requests with the same tools in different
/// order produce identical cache keys. Tools without a `name` sort to the end.
///
/// Called **before** hashing.
pub fn sort_tools_by_name(body: &mut Value) {
    if let Some(Value::Array(tools)) = body.get_mut("tools") {
        tools.sort_by(|a, b| {
            let na = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            na.cmp(nb)
        });
    }
}

// ── Anthropic cache_control injection ─────────────────────────────

/// Layer boundary marker pattern used by mori's context assembly.
/// System prompts with these markers get multi-point cache_control injection.
const LAYER_MARKER: &str = "<!-- mori:layer:";

/// Inject Anthropic `cache_control: {"type":"ephemeral"}` on the system prompt.
///
/// **Multi-layer mode:** If the system prompt contains `<!-- mori:layer:N -->`
/// markers, split into multiple content blocks with `cache_control` on each
/// boundary. Anthropic allows up to 4 cache breakpoints; the most stable
/// layers (role instructions, workspace map) get cached across agents while
/// task-specific context changes per request.
///
/// **Single-layer mode (fallback):** If no markers are present, applies one
/// `cache_control` to the entire system prompt (original behavior).
///
/// Called **after** hashing (does not affect local cache keys) and **before**
/// the provider call. Only applied on Anthropic-bound requests.
pub fn inject_anthropic_cache_control(body: &mut Value) {
    match body.get("system").cloned() {
        Some(Value::String(s)) => {
            if s.contains(LAYER_MARKER) {
                // Multi-layer: split at markers and cache each layer.
                let blocks = split_into_cached_blocks(&s);
                body["system"] = Value::Array(blocks);
            } else {
                // Single-layer: cache the whole thing.
                body["system"] = serde_json::json!([{
                    "type": "text",
                    "text": s,
                    "cache_control": {"type": "ephemeral"}
                }]);
            }
        }
        Some(Value::Array(_)) => {
            // Already an array: add cache_control to the last element.
            if let Some(Value::Array(blocks)) = body.get_mut("system") {
                if let Some(last) = blocks.last_mut() {
                    last["cache_control"] = serde_json::json!({"type": "ephemeral"});
                }
            }
        }
        _ => {}
    }
}

/// Split a system prompt at `<!-- mori:layer:N -->` markers into
/// separate content blocks, each with `cache_control: ephemeral`.
/// The last block (task-specific context) does NOT get cache_control
/// since it changes every request.
///
/// Anthropic allows up to 4 cache breakpoints. We use at most 3
/// (layers 1-3 cached, layer 4+ uncached).
fn split_into_cached_blocks(text: &str) -> Vec<Value> {
    let mut blocks = Vec::new();
    let mut remaining = text;
    let mut layer_count = 0;

    while let Some(pos) = remaining.find(LAYER_MARKER) {
        // Find the end of the marker: <!-- mori:layer:N -->
        let marker_start = pos;
        let after_prefix = &remaining[pos + LAYER_MARKER.len()..];
        let marker_end = after_prefix
            .find("-->")
            .map(|p| pos + LAYER_MARKER.len() + p + 3)
            .unwrap_or(remaining.len());

        // Text before this marker is a cacheable block.
        let chunk = remaining[..marker_start].trim();
        if !chunk.is_empty() {
            layer_count += 1;
            // Cache the first 3 layers (Anthropic limit: 4 breakpoints).
            if layer_count <= 3 {
                blocks.push(serde_json::json!({
                    "type": "text",
                    "text": chunk,
                    "cache_control": {"type": "ephemeral"}
                }));
            } else {
                blocks.push(serde_json::json!({
                    "type": "text",
                    "text": chunk,
                }));
            }
        }

        remaining = &remaining[marker_end..];
    }

    // Remaining text after the last marker: task-specific, NOT cached.
    let tail = remaining.trim();
    if !tail.is_empty() {
        blocks.push(serde_json::json!({
            "type": "text",
            "text": tail,
        }));
    }

    // Fallback: if splitting produced nothing useful, cache the whole thing.
    if blocks.is_empty() {
        blocks.push(serde_json::json!({
            "type": "text",
            "text": text,
            "cache_control": {"type": "ephemeral"}
        }));
    }

    blocks
}

// ── JSON key normalization ─────────────────────────────────────────

/// Recursively sort JSON object keys alphabetically, in place.
///
/// Arrays preserve element order; only object keys are reordered.
/// This ensures identical content produces identical serialized bytes
/// regardless of the original key insertion order.
pub fn normalize_json_ordering_in_place(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Sort by collecting into BTreeMap and back.
            for (_, v) in map.iter_mut() {
                normalize_json_ordering_in_place(v);
            }
            let sorted: BTreeMap<String, Value> = std::mem::take(map).into_iter().collect();
            *map = sorted.into_iter().collect();
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                normalize_json_ordering_in_place(v);
            }
        }
        _ => {}
    }
}

/// Recursively sort JSON object keys alphabetically (returns new value).
///
/// Kept for backward compatibility with tests.
pub fn normalize_json_ordering(value: &Value) -> Value {
    let mut cloned = value.clone();
    normalize_json_ordering_in_place(&mut cloned);
    cloned
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── normalize_json_ordering ────────────────────────────────────

    #[test]
    fn different_key_orders_produce_same_output() {
        let a: Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap_or_default();
        let b: Value = serde_json::from_str(r#"{"a": 2, "m": 3, "z": 1}"#).unwrap_or_default();

        let norm_a = serde_json::to_string(&normalize_json_ordering(&a)).unwrap_or_default();
        let norm_b = serde_json::to_string(&normalize_json_ordering(&b)).unwrap_or_default();

        assert_eq!(norm_a, norm_b);
        assert_eq!(norm_a, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn nested_objects_are_normalized() {
        let input: Value =
            serde_json::from_str(r#"{"b": {"y": 1, "x": 2}, "a": [{"q": 1, "p": 2}]}"#)
                .unwrap_or_default();

        let normalized =
            serde_json::to_string(&normalize_json_ordering(&input)).unwrap_or_default();
        assert_eq!(normalized, r#"{"a":[{"p":2,"q":1}],"b":{"x":2,"y":1}}"#);
    }

    #[test]
    fn array_order_is_preserved() {
        let input: Value = serde_json::from_str(r#"[3, 1, 2]"#).unwrap_or_default();
        let normalized =
            serde_json::to_string(&normalize_json_ordering(&input)).unwrap_or_default();
        assert_eq!(normalized, "[3,1,2]");
    }

    #[test]
    fn scalars_pass_through() {
        assert_eq!(normalize_json_ordering(&Value::Null), Value::Null);
        assert_eq!(
            normalize_json_ordering(&Value::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(
            normalize_json_ordering(&serde_json::json!(42)),
            serde_json::json!(42)
        );
    }

    // ── strip_variable_content ─────────────────────────────────────

    #[test]
    fn strip_variable_content_removes_uuid() {
        let mut body = serde_json::json!({
            "system": "Session: 550e8400-e29b-41d4-a716-446655440000. Be helpful."
        });
        strip_variable_content(&mut body);
        assert_eq!(body["system"], "Session: [VAR]. Be helpful.");
    }

    #[test]
    fn strip_variable_content_removes_iso_timestamp() {
        let mut body = serde_json::json!({
            "system": "As of 2024-01-15T10:30:00Z, you are an assistant."
        });
        strip_variable_content(&mut body);
        assert_eq!(body["system"], "As of [VAR], you are an assistant.");
    }

    #[test]
    fn strip_variable_content_removes_timestamp_with_offset() {
        let mut body = serde_json::json!({
            "system": "Time: 2024-03-22T14:55:00.123+05:30"
        });
        strip_variable_content(&mut body);
        assert_eq!(body["system"], "Time: [VAR]");
    }

    #[test]
    fn strip_variable_content_noop_for_clean_system() {
        let mut body = serde_json::json!({
            "system": "You are a helpful assistant."
        });
        strip_variable_content(&mut body);
        assert_eq!(body["system"], "You are a helpful assistant.");
    }

    #[test]
    fn strip_variable_content_noop_when_no_system_field() {
        let mut body = serde_json::json!({"messages": []});
        strip_variable_content(&mut body);
        assert!(body.get("system").is_none());
    }

    #[test]
    fn strip_variable_content_noop_for_array_system() {
        // Array-form system (already injected cache_control) should not be touched.
        let original = serde_json::json!([{"type": "text", "text": "hello", "cache_control": {"type": "ephemeral"}}]);
        let mut body = serde_json::json!({"system": original.clone()});
        strip_variable_content(&mut body);
        assert_eq!(body["system"], original);
    }

    // ── sort_tools_by_name ─────────────────────────────────────────

    #[test]
    fn sort_tools_by_name_sorts_alphabetically() {
        let mut body = serde_json::json!({
            "tools": [
                {"name": "zebra", "description": "z tool"},
                {"name": "alpha", "description": "a tool"},
                {"name": "mango", "description": "m tool"},
            ]
        });
        sort_tools_by_name(&mut body);
        let names: Vec<&str> = body["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["alpha", "mango", "zebra"]);
    }

    #[test]
    fn sort_tools_by_name_noop_without_tools_field() {
        let mut body = serde_json::json!({"messages": []});
        sort_tools_by_name(&mut body); // must not panic
        assert!(body.get("tools").is_none());
    }

    // ── inject_anthropic_cache_control ─────────────────────────────

    #[test]
    fn inject_cache_control_converts_string_to_array() {
        let mut body = serde_json::json!({
            "system": "You are a helpful assistant.",
            "model": "claude-opus-4-6",
        });
        inject_anthropic_cache_control(&mut body);

        let blocks = body["system"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[0]["text"], "You are a helpful assistant.");
        assert_eq!(blocks[0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn inject_cache_control_adds_to_last_array_block() {
        let mut body = serde_json::json!({
            "system": [
                {"type": "text", "text": "Global prefix."},
                {"type": "text", "text": "Agent-specific rules."},
            ]
        });
        inject_anthropic_cache_control(&mut body);

        let blocks = body["system"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        // First block untouched
        assert!(blocks[0].get("cache_control").is_none());
        // Last block gets the marker
        assert_eq!(blocks[1]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn inject_cache_control_noop_when_no_system_field() {
        let mut body = serde_json::json!({"messages": [], "model": "claude-haiku-4-5"});
        inject_anthropic_cache_control(&mut body);
        assert!(body.get("system").is_none());
    }
}

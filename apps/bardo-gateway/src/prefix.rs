//! JSON key normalization for prefix cache optimization.
//!
//! When two requests contain the same data but with different key ordering,
//! they produce different bytes and miss the blake3 hash cache. Normalizing
//! key order (alphabetical via `BTreeMap`) before hashing ensures identical
//! content always produces the same hash.

use std::collections::BTreeMap;

use serde_json::Value;

/// Recursively sort JSON object keys alphabetically.
///
/// Arrays preserve element order; only object keys are reordered.
/// This ensures identical content produces identical serialized bytes
/// regardless of the original key insertion order.
pub fn normalize_json_ordering(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let ordered: BTreeMap<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_json_ordering(v)))
                .collect();
            Value::Object(ordered.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_json_ordering).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

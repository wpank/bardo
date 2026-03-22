//! HDC fingerprinting for symbols.
//!
//! Each symbol is encoded into a 10,240-bit hyperdimensional vector that captures
//! its kind, name, and parameter types. Similar symbols produce similar fingerprints.

use crate::symbol::{Symbol, SymbolKind};
use bardo_primitives::HdcVector;

/// Get the deterministic role vector for a `SymbolKind`.
fn role_vector(kind: SymbolKind) -> HdcVector {
    let seed = match kind {
        SymbolKind::Function => b"mori:role:function" as &[u8],
        SymbolKind::Struct => b"mori:role:struct",
        SymbolKind::Enum => b"mori:role:enum",
        SymbolKind::Trait => b"mori:role:trait",
        SymbolKind::TypeAlias => b"mori:role:type_alias",
        SymbolKind::Const => b"mori:role:const",
        SymbolKind::Module => b"mori:role:module",
        SymbolKind::Impl => b"mori:role:impl",
        SymbolKind::Use => b"mori:role:use",
        SymbolKind::Macro => b"mori:role:macro",
    };
    HdcVector::from_seed(seed)
}

/// Encode a name into an HDC vector via character trigrams.
///
/// The name is split into overlapping 3-character windows. Each trigram
/// is used as a seed for a deterministic vector, and all trigram vectors
/// are bundled together.
fn encode_name(name: &str) -> HdcVector {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < 3 {
        // For very short names, just seed from the whole name
        return HdcVector::from_seed(name.as_bytes());
    }

    let trigram_vecs: Vec<HdcVector> = chars
        .windows(3)
        .map(|w| {
            let trigram: String = w.iter().collect();
            HdcVector::from_seed(trigram.as_bytes())
        })
        .collect();

    let refs: Vec<&HdcVector> = trigram_vecs.iter().collect();
    HdcVector::bundle(&refs)
}

/// Known Rust primitive and common types for param extraction.
const KNOWN_TYPES: &[&str] = &[
    "String", "str", "Vec", "Option", "Result", "Box", "Rc", "Arc", "HashMap", "HashSet",
    "BTreeMap", "BTreeSet", "Path", "PathBuf", "bool", "char", "usize", "isize", "u8", "u16",
    "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
];

/// Extract type-like words from a signature.
///
/// Splits on common delimiters and keeps words that start with an uppercase
/// letter or are known Rust types.
fn extract_param_types(signature: &str) -> Vec<String> {
    let mut types = Vec::new();

    for word in signature.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            continue;
        }

        let first_char = trimmed.chars().next().unwrap_or('a');
        let is_uppercase_start = first_char.is_uppercase();
        let is_known = KNOWN_TYPES.contains(&trimmed);

        if is_uppercase_start || is_known {
            types.push(trimmed.to_string());
        }
    }

    types
}

/// Encode parameter types into an HDC vector.
fn encode_params(signature: &str) -> HdcVector {
    let types = extract_param_types(signature);
    if types.is_empty() {
        return HdcVector::from_seed(b"mori:no_params");
    }

    let type_vecs: Vec<HdcVector> = types
        .iter()
        .map(|t| HdcVector::from_seed(t.as_bytes()))
        .collect();

    let refs: Vec<&HdcVector> = type_vecs.iter().collect();
    HdcVector::bundle(&refs)
}

/// Compute the HDC fingerprint for a symbol.
///
/// The fingerprint encodes three aspects:
/// 1. The symbol's kind (function, struct, etc.)
/// 2. The symbol's name (via character trigrams)
/// 3. The parameter/field types in the signature
///
/// These are combined: `bind(role_vec, bundle(name_vec, param_vec))`
pub fn fingerprint(symbol: &Symbol) -> HdcVector {
    let role_vec = role_vector(symbol.kind);
    let name_vec = encode_name(&symbol.name);
    let param_vec = encode_params(&symbol.signature);

    let combined = HdcVector::bundle(&[&name_vec, &param_vec]);
    role_vec.bind(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{Symbol, SymbolKind, Visibility};

    fn make_symbol(name: &str, kind: SymbolKind, signature: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            file: "test.rs".to_string(),
            line: 1,
            signature: signature.to_string(),
            visibility: Visibility::Public,
            doc: None,
            content_hash: blake3::hash(signature.as_bytes()).into(),
        }
    }

    #[test]
    fn same_symbol_same_fingerprint() {
        let sym = make_symbol("foo", SymbolKind::Function, "fn foo(x: u32) -> String");
        let fp1 = fingerprint(&sym);
        let fp2 = fingerprint(&sym);
        let sim = fp1.similarity(&fp2);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "Same symbol should produce identical fingerprints, got similarity={sim}"
        );
    }

    #[test]
    fn similar_functions_high_similarity() {
        let sym1 = make_symbol(
            "process_input",
            SymbolKind::Function,
            "fn process_input(data: &str) -> Result<String>",
        );
        let sym2 = make_symbol(
            "process_output",
            SymbolKind::Function,
            "fn process_output(data: &str) -> Result<String>",
        );
        let fp1 = fingerprint(&sym1);
        let fp2 = fingerprint(&sym2);
        let sim = fp1.similarity(&fp2);
        assert!(
            sim > 0.55,
            "Similar functions should have high similarity, got {sim}"
        );
    }

    #[test]
    fn different_kinds_lower_similarity() {
        let func = make_symbol(
            "Config",
            SymbolKind::Function,
            "fn Config(name: String) -> Config",
        );
        let strct = make_symbol(
            "Config",
            SymbolKind::Struct,
            "struct Config { name: String }",
        );
        let fp_func = fingerprint(&func);
        let fp_struct = fingerprint(&strct);
        let sim = fp_func.similarity(&fp_struct);
        // Different kinds should reduce similarity (though not to zero since names match)
        assert!(
            sim < 0.85,
            "Different kinds should have lower similarity, got {sim}"
        );
    }

    #[test]
    fn completely_different_symbols() {
        let sym1 = make_symbol(
            "parse_config",
            SymbolKind::Function,
            "fn parse_config(path: &Path) -> Config",
        );
        let sym2 = make_symbol("Color", SymbolKind::Enum, "enum Color { Red, Green, Blue }");
        let fp1 = fingerprint(&sym1);
        let fp2 = fingerprint(&sym2);
        let sim = fp1.similarity(&fp2);
        assert!(
            sim < 0.65,
            "Completely different symbols should have low similarity, got {sim}"
        );
    }

    #[test]
    fn extract_param_types_works() {
        let types = extract_param_types("fn foo(name: &str, count: u32) -> Option<String>");
        assert!(types.contains(&"String".to_string()), "types: {types:?}");
        assert!(types.contains(&"Option".to_string()), "types: {types:?}");
        assert!(types.contains(&"u32".to_string()), "types: {types:?}");
        assert!(types.contains(&"str".to_string()), "types: {types:?}");
    }
}

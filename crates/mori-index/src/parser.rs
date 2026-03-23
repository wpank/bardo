//! Tree-sitter based Rust parser for symbol and reference extraction.

use crate::error::IndexError;
use crate::symbol::{RefKind, Symbol, SymbolKind, SymbolRef, Visibility};

/// Result of parsing a single Rust source file.
#[derive(Debug, Default)]
pub struct ParseResult {
    /// Symbols defined in the file.
    pub symbols: Vec<Symbol>,
    /// References from this file to other symbols.
    pub refs: Vec<SymbolRef>,
}

/// A tree-sitter parser configured for Rust source code.
pub struct RustParser {
    parser: tree_sitter::Parser,
}

impl RustParser {
    /// Create a new Rust parser.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Parse` if tree-sitter cannot be configured for Rust.
    pub fn new() -> Result<Self, IndexError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| IndexError::Parse {
                file: String::new(),
                message: format!("failed to set Rust language: {e}"),
            })?;
        Ok(Self { parser })
    }

    /// Parse a Rust source file and extract symbols and references.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Parse` if tree-sitter fails to produce a tree.
    pub fn parse_file(&mut self, path: &str, source: &[u8]) -> Result<ParseResult, IndexError> {
        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| IndexError::Parse {
                file: path.to_string(),
                message: "tree-sitter returned no tree".to_string(),
            })?;

        let root = tree.root_node();
        let mut result = ParseResult::default();

        let mut cursor = root.walk();
        walk_tree(
            &mut cursor,
            source,
            path,
            &mut result.symbols,
            &mut result.refs,
        );

        Ok(result)
    }
}

/// Recursively walk the tree and extract symbols and references.
fn walk_tree(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    file: &str,
    symbols: &mut Vec<Symbol>,
    refs: &mut Vec<SymbolRef>,
) {
    loop {
        let node = cursor.node();
        let kind_str = node.kind();

        match kind_str {
            "function_item" | "function_signature_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Function) {
                    symbols.push(sym);
                }
                // Extract refs from function body
                extract_refs_from_body(&node, source, file, refs);
            }
            "struct_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Struct) {
                    symbols.push(sym);
                }
            }
            "enum_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Enum) {
                    symbols.push(sym);
                }
            }
            "trait_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Trait) {
                    symbols.push(sym);
                }
            }
            "impl_item" => {
                if let Some(sym) = extract_impl_symbol(&node, source, file) {
                    symbols.push(sym);
                }
                // Extract refs from impl body
                extract_refs_from_body(&node, source, file, refs);
            }
            "type_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::TypeAlias) {
                    symbols.push(sym);
                }
            }
            "const_item" | "static_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Const) {
                    symbols.push(sym);
                }
            }
            "mod_item" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Module) {
                    symbols.push(sym);
                }
            }
            "use_declaration" => {
                if let Some(sym) = extract_use_symbol(&node, source, file) {
                    // Also create a SymbolRef for the import
                    refs.push(SymbolRef {
                        from_file: file.to_string(),
                        from_line: node.start_position().row as u32 + 1,
                        target: sym.name.clone(),
                        ref_kind: RefKind::Import,
                    });
                    symbols.push(sym);
                }
            }
            "macro_definition" => {
                if let Some(sym) = extract_symbol(&node, source, file, SymbolKind::Macro) {
                    symbols.push(sym);
                }
            }
            _ => {
                // Recurse into children for top-level traversal
                if cursor.goto_first_child() {
                    walk_tree(cursor, source, file, symbols, refs);
                    cursor.goto_parent();
                }
            }
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

/// Extract a symbol from a named node.
fn extract_symbol(
    node: &tree_sitter::Node,
    source: &[u8],
    file: &str,
    kind: SymbolKind,
) -> Option<Symbol> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .map(String::from)?;

    let visibility = extract_visibility(node, source);
    let signature = extract_signature(node, source);
    let doc = extract_doc_comment(node, source);
    let line = node.start_position().row as u32 + 1;
    let content_hash = blake3::hash(signature.as_bytes()).into();

    Some(Symbol {
        name,
        kind,
        file: file.to_string(),
        line,
        signature,
        visibility,
        doc,
        content_hash,
    })
}

/// Extract an impl block symbol. Impl blocks don't have a simple "name" field,
/// so we build one from the type being implemented.
fn extract_impl_symbol(node: &tree_sitter::Node, source: &[u8], file: &str) -> Option<Symbol> {
    let name = node
        .child_by_field_name("type")
        .and_then(|n| n.utf8_text(source).ok())
        .map(String::from)
        .unwrap_or_else(|| "impl".to_string());

    let visibility = Visibility::Private; // impl blocks don't have visibility
    let signature = extract_signature(node, source);
    let doc = extract_doc_comment(node, source);
    let line = node.start_position().row as u32 + 1;
    let content_hash = blake3::hash(signature.as_bytes()).into();

    Some(Symbol {
        name,
        kind: SymbolKind::Impl,
        file: file.to_string(),
        line,
        signature,
        visibility,
        doc,
        content_hash,
    })
}

/// Extract a use declaration symbol.
fn extract_use_symbol(node: &tree_sitter::Node, source: &[u8], file: &str) -> Option<Symbol> {
    let text = node.utf8_text(source).ok()?;
    let visibility = extract_visibility(node, source);
    let line = node.start_position().row as u32 + 1;
    let content_hash = blake3::hash(text.as_bytes()).into();

    // The "name" for a use is the full path text (minus the `use` keyword and `;`)
    let name = text
        .strip_prefix("pub ")
        .or_else(|| text.strip_prefix("pub(crate) "))
        .or_else(|| text.strip_prefix("pub(super) "))
        .unwrap_or(text)
        .strip_prefix("use ")
        .unwrap_or(text)
        .trim_end_matches(';')
        .trim()
        .to_string();

    Some(Symbol {
        name,
        kind: SymbolKind::Use,
        file: file.to_string(),
        line,
        signature: text.to_string(),
        visibility,
        doc: None,
        content_hash,
    })
}

/// Extract visibility from a node by looking for a visibility_modifier child.
fn extract_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                let text = child.utf8_text(source).unwrap_or_default();
                return if text.contains("crate") {
                    Visibility::Crate
                } else if text.contains("super") || text.contains("in ") {
                    Visibility::Restricted
                } else {
                    Visibility::Public
                };
            }
        }
    }
    Visibility::Private
}

/// Extract signature text (up to the opening brace, or full text for items without blocks).
fn extract_signature(node: &tree_sitter::Node, source: &[u8]) -> String {
    let full_text = node.utf8_text(source).unwrap_or_default();

    // Look for the opening brace and truncate there
    if let Some(brace_pos) = full_text.find('{') {
        let sig = full_text[..brace_pos].trim();
        return sig.to_string();
    }

    // No brace found: return the full text (e.g., function signatures in traits, type aliases)
    full_text.trim().to_string()
}

/// Extract doc comments from preceding line_comment siblings starting with `///`.
fn extract_doc_comment(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut sibling = node.prev_sibling();

    while let Some(sib) = sibling {
        if sib.kind() == "line_comment" {
            let text = sib.utf8_text(source).unwrap_or_default();
            if let Some(stripped) = text.strip_prefix("///") {
                doc_lines.push(stripped.trim().to_string());
            } else {
                break;
            }
        } else {
            break;
        }
        sibling = sib.prev_sibling();
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join("\n"))
    }
}

/// Extract references from within a node's body (function bodies, impl blocks).
fn extract_refs_from_body(
    node: &tree_sitter::Node,
    source: &[u8],
    file: &str,
    refs: &mut Vec<SymbolRef>,
) {
    let mut body_cursor = node.walk();
    extract_refs_recursive(&mut body_cursor, source, file, refs);
}

/// Recursively find references within a subtree.
fn extract_refs_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    file: &str,
    refs: &mut Vec<SymbolRef>,
) {
    let node = cursor.node();
    match node.kind() {
        "use_declaration" => {
            if let Some(text) = node.utf8_text(source).ok() {
                let target = text
                    .strip_prefix("use ")
                    .unwrap_or(text)
                    .trim_end_matches(';')
                    .trim()
                    .to_string();
                refs.push(SymbolRef {
                    from_file: file.to_string(),
                    from_line: node.start_position().row as u32 + 1,
                    target,
                    ref_kind: RefKind::Import,
                });
            }
        }
        "type_identifier" => {
            if let Some(text) = node.utf8_text(source).ok() {
                refs.push(SymbolRef {
                    from_file: file.to_string(),
                    from_line: node.start_position().row as u32 + 1,
                    target: text.to_string(),
                    ref_kind: RefKind::TypeRef,
                });
            }
        }
        "call_expression" => {
            // The function name is typically the first child
            if let Some(func_node) = node.child(0) {
                if let Some(text) = func_node.utf8_text(source).ok() {
                    refs.push(SymbolRef {
                        from_file: file.to_string(),
                        from_line: node.start_position().row as u32 + 1,
                        target: text.to_string(),
                        ref_kind: RefKind::Call,
                    });
                }
            }
        }
        _ => {}
    }

    if cursor.goto_first_child() {
        loop {
            extract_refs_recursive(cursor, source, file, refs);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RUST: &[u8] = br#"
use std::collections::HashMap;

/// A greeting function.
/// Returns a formatted string.
pub fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}

struct Config {
    name: String,
    value: u32,
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub trait Drawable {
    fn draw(&self);
}

impl Drawable for Config {
    fn draw(&self) {
        let _map: HashMap<String, u32> = HashMap::new();
    }
}

type Alias = Vec<String>;

pub const MAX_SIZE: usize = 100;

mod inner {
    pub fn helper() {}
}

macro_rules! my_macro {
    () => {};
}
"#;

    #[test]
    fn parse_extracts_symbols() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        let names: Vec<&str> = result.symbols.iter().map(|s| s.name.as_str()).collect();

        // Check key symbols are found
        assert!(names.contains(&"greet"), "missing greet, found: {names:?}");
        assert!(
            names.contains(&"Config"),
            "missing Config, found: {names:?}"
        );
        assert!(names.contains(&"Color"), "missing Color, found: {names:?}");
        assert!(
            names.contains(&"Drawable"),
            "missing Drawable, found: {names:?}"
        );
        assert!(
            names.contains(&"MAX_SIZE"),
            "missing MAX_SIZE, found: {names:?}"
        );
        assert!(names.contains(&"inner"), "missing inner, found: {names:?}");
    }

    #[test]
    fn parse_extracts_function_kind() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        let greet = result
            .symbols
            .iter()
            .find(|s| s.name == "greet")
            .unwrap_or_else(|| {
                panic!("greet not found");
            });
        assert_eq!(greet.kind, SymbolKind::Function);
        assert_eq!(greet.visibility, Visibility::Public);
    }

    #[test]
    fn parse_extracts_doc_comments() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        let greet = result
            .symbols
            .iter()
            .find(|s| s.name == "greet")
            .unwrap_or_else(|| {
                panic!("greet not found");
            });
        assert!(greet.doc.is_some(), "greet should have doc comments");
        let doc = greet.doc.as_ref().unwrap_or(&String::new()).clone();
        assert!(
            doc.contains("greeting"),
            "doc should contain 'greeting', got: {doc}"
        );
    }

    #[test]
    fn parse_extracts_refs() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        // Should have at least one import ref from the `use` statement
        let import_refs: Vec<_> = result
            .refs
            .iter()
            .filter(|r| r.ref_kind == RefKind::Import)
            .collect();
        assert!(
            !import_refs.is_empty(),
            "should have import refs, found: {:?}",
            result.refs
        );

        // Should have type refs from the impl body (HashMap)
        let type_refs: Vec<_> = result
            .refs
            .iter()
            .filter(|r| r.ref_kind == RefKind::TypeRef)
            .collect();
        assert!(
            !type_refs.is_empty(),
            "should have type refs, found: {:?}",
            result.refs
        );
    }

    #[test]
    fn parse_extracts_use_symbol() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        let use_syms: Vec<_> = result
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Use)
            .collect();
        assert!(
            !use_syms.is_empty(),
            "should have use symbols, found kinds: {:?}",
            result.symbols.iter().map(|s| s.kind).collect::<Vec<_>>()
        );
    }

    #[test]
    fn parse_extracts_impl_symbol() {
        let mut parser = RustParser::new().ok().unwrap_or_else(|| {
            panic!("Failed to create parser");
        });
        let result = parser
            .parse_file("test.rs", SAMPLE_RUST)
            .ok()
            .unwrap_or_else(|| {
                panic!("Failed to parse file");
            });

        let impl_syms: Vec<_> = result
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Impl)
            .collect();
        assert!(
            !impl_syms.is_empty(),
            "should have impl symbols, found kinds: {:?}",
            result.symbols.iter().map(|s| s.kind).collect::<Vec<_>>()
        );
    }
}

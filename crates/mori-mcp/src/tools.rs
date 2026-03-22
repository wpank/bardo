//! Tool definitions and dispatch for the MCP server.
//!
//! Wraps `mori-index` operations as five MCP tools: `search_code`,
//! `get_symbol_context`, `get_file_ast`, `find_similar_patterns`, and
//! `get_index_stats`.

use mori_index::Index;
use serde_json::{Value, json};

use crate::protocol::{ToolCallResult, ToolContent, ToolDefinition};

/// Return all tool definitions for `tools/list`.
pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "search_code".into(),
            description: "Search the code index by symbol name, optionally filtered by kind and visibility.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query (symbol name substring)" },
                    "kind": { "type": "string", "description": "Optional symbol kind filter: function, struct, enum, trait, type_alias, const, module, impl, use, macro" },
                    "visibility": { "type": "string", "description": "Optional visibility filter: public, crate, restricted, private" },
                    "limit": { "type": "number", "description": "Max results to return (default 20)" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "get_symbol_context".into(),
            description: "Get full details for a symbol and related symbols found in its signature.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Name of the symbol to look up" },
                    "limit": { "type": "number", "description": "Max related symbols to return (default 10)" }
                },
                "required": ["symbol_name"]
            }),
        },
        ToolDefinition {
            name: "get_file_ast".into(),
            description: "List all indexed symbols from a given file path.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "File path (relative to project root) to inspect" }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "find_similar_patterns".into(),
            description: "Find symbols structurally similar to a named symbol using HDC fingerprints.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Name of the symbol to find similarities for" },
                    "threshold": { "type": "number", "description": "Minimum similarity threshold 0.0-1.0 (default 0.6)" },
                    "limit": { "type": "number", "description": "Max results to return (default 10)" }
                },
                "required": ["symbol_name"]
            }),
        },
        ToolDefinition {
            name: "get_index_stats".into(),
            description: "Return statistics about the current index: file count, symbol count, reference count.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Dispatch a tool call by name.
pub fn call_tool(index: &mut Index, name: &str, arguments: &Value) -> ToolCallResult {
    match name {
        "search_code" => tool_search_code(index, arguments),
        "get_symbol_context" => tool_get_symbol_context(index, arguments),
        "get_file_ast" => tool_get_file_ast(index, arguments),
        "find_similar_patterns" => tool_find_similar_patterns(index, arguments),
        "get_index_stats" => tool_get_index_stats(index),
        _ => tool_error(format!("Unknown tool: {name}")),
    }
}

// ---------------------------------------------------------------------------
// Individual tools
// ---------------------------------------------------------------------------

fn tool_search_code(index: &Index, args: &Value) -> ToolCallResult {
    let query = match args.get("query").and_then(Value::as_str) {
        Some(q) => q,
        None => return tool_error("Missing required parameter: query".into()),
    };

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;

    let kind_str = args.get("kind").and_then(Value::as_str);
    let visibility_str = args.get("visibility").and_then(Value::as_str);

    let results = if let Some(kind_s) = kind_str {
        let kind = match parse_symbol_kind(kind_s) {
            Some(k) => k,
            None => return tool_error(format!("Unknown symbol kind: {kind_s}")),
        };
        let vis = visibility_str.and_then(parse_visibility);
        match index.search_kind(kind, vis, limit) {
            Ok(r) => r,
            Err(e) => return tool_error(format!("search_kind failed: {e}")),
        }
    } else {
        match index.search(query, limit) {
            Ok(r) => r,
            Err(e) => return tool_error(format!("search failed: {e}")),
        }
    };

    let items: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "name": r.symbol.name,
                "file": r.symbol.file,
                "line": r.symbol.line,
                "kind": format!("{:?}", r.symbol.kind),
                "signature": r.symbol.signature,
                "score": r.score,
            })
        })
        .collect();

    tool_text(serde_json::to_string_pretty(&items).unwrap_or_default())
}

fn tool_get_symbol_context(index: &mut Index, args: &Value) -> ToolCallResult {
    let symbol_name = match args.get("symbol_name").and_then(Value::as_str) {
        Some(n) => n,
        None => return tool_error("Missing required parameter: symbol_name".into()),
    };

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;

    // Find the symbol itself.
    let primary = match index.search(symbol_name, 1) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    let Some(sym) = primary.first() else {
        return tool_error(format!("Symbol not found: {symbol_name}"));
    };

    // Extract type-like words from the signature to search for related symbols.
    let related_names: Vec<String> = sym
        .symbol
        .signature
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| {
            let first = w.chars().next().unwrap_or('a');
            first.is_uppercase() && *w != sym.symbol.name
        })
        .map(String::from)
        .collect();

    let mut related = Vec::new();
    for rname in &related_names {
        if related.len() >= limit {
            break;
        }
        if let Ok(results) = index.search(rname, 1) {
            for r in results {
                if r.symbol.name != sym.symbol.name {
                    related.push(json!({
                        "name": r.symbol.name,
                        "file": r.symbol.file,
                        "line": r.symbol.line,
                        "kind": format!("{:?}", r.symbol.kind),
                        "signature": r.symbol.signature,
                    }));
                }
            }
        }
    }

    let output = json!({
        "symbol": {
            "name": sym.symbol.name,
            "file": sym.symbol.file,
            "line": sym.symbol.line,
            "kind": format!("{:?}", sym.symbol.kind),
            "signature": sym.symbol.signature,
            "visibility": format!("{:?}", sym.symbol.visibility),
            "doc": sym.symbol.doc,
        },
        "related": related,
    });

    tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
}

fn tool_get_file_ast(index: &Index, args: &Value) -> ToolCallResult {
    let file_path = match args.get("file_path").and_then(Value::as_str) {
        Some(f) => f,
        None => return tool_error("Missing required parameter: file_path".into()),
    };

    // Use the filename as a keyword query. The DB search_by_name looks at
    // symbol names, not file paths, so instead we search with a generous
    // limit and filter client-side. This is an approximation because Index
    // doesn't expose a `symbols_in_file()` method directly.
    //
    // We search for the last path component to get a broader match set,
    // then filter by file.
    let query = file_path
        .rsplit('/')
        .next()
        .unwrap_or(file_path)
        .trim_end_matches(".rs");

    let results = match index.search(query, 500) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    let file_symbols: Vec<Value> = results
        .iter()
        .filter(|r| r.symbol.file.contains(file_path) || file_path.contains(&r.symbol.file))
        .map(|r| {
            json!({
                "name": r.symbol.name,
                "line": r.symbol.line,
                "kind": format!("{:?}", r.symbol.kind),
                "signature": r.symbol.signature,
                "visibility": format!("{:?}", r.symbol.visibility),
            })
        })
        .collect();

    if file_symbols.is_empty() {
        return tool_text(format!("No indexed symbols found for file: {file_path}"));
    }

    tool_text(serde_json::to_string_pretty(&file_symbols).unwrap_or_default())
}

fn tool_find_similar_patterns(index: &mut Index, args: &Value) -> ToolCallResult {
    let symbol_name = match args.get("symbol_name").and_then(Value::as_str) {
        Some(n) => n,
        None => return tool_error("Missing required parameter: symbol_name".into()),
    };

    let threshold = args.get("threshold").and_then(Value::as_f64).unwrap_or(0.6) as f32;

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;

    // Find the symbol first to get its fingerprint.
    let primary = match index.search(symbol_name, 1) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    let Some(sym) = primary.first() else {
        return tool_error(format!("Symbol not found: {symbol_name}"));
    };

    // Compute the fingerprint from the symbol data.
    let fp = mori_index::fingerprint::fingerprint(&sym.symbol);

    let similar = match index.search_similar(&fp, threshold, limit) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search_similar failed: {e}")),
    };

    let items: Vec<Value> = similar
        .iter()
        .map(|r| {
            json!({
                "name": r.symbol.name,
                "file": r.symbol.file,
                "line": r.symbol.line,
                "kind": format!("{:?}", r.symbol.kind),
                "signature": r.symbol.signature,
                "similarity": r.score,
            })
        })
        .collect();

    tool_text(serde_json::to_string_pretty(&items).unwrap_or_default())
}

fn tool_get_index_stats(index: &Index) -> ToolCallResult {
    match index.stats() {
        Ok(stats) => {
            let output = json!({
                "files": stats.files,
                "symbols": stats.symbols,
                "refs": stats.refs,
                "resolved_refs": stats.resolved_refs,
            });
            tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
        }
        Err(e) => tool_error(format!("stats failed: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool_text(text: String) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent {
            type_: "text".into(),
            text,
        }],
        is_error: None,
    }
}

fn tool_error(message: String) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent {
            type_: "text".into(),
            text: message,
        }],
        is_error: Some(true),
    }
}

fn parse_symbol_kind(s: &str) -> Option<mori_index::symbol::SymbolKind> {
    use mori_index::symbol::SymbolKind;
    match s {
        "function" => Some(SymbolKind::Function),
        "struct" => Some(SymbolKind::Struct),
        "enum" => Some(SymbolKind::Enum),
        "trait" => Some(SymbolKind::Trait),
        "type_alias" => Some(SymbolKind::TypeAlias),
        "const" => Some(SymbolKind::Const),
        "module" => Some(SymbolKind::Module),
        "impl" => Some(SymbolKind::Impl),
        "use" => Some(SymbolKind::Use),
        "macro" => Some(SymbolKind::Macro),
        _ => None,
    }
}

fn parse_visibility(s: &str) -> Option<mori_index::symbol::Visibility> {
    use mori_index::symbol::Visibility;
    match s {
        "public" => Some(Visibility::Public),
        "crate" => Some(Visibility::Crate),
        "restricted" => Some(Visibility::Restricted),
        "private" => Some(Visibility::Private),
        _ => None,
    }
}

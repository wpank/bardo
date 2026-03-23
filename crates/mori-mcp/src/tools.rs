//! Tool definitions and dispatch for the MCP server.
//!
//! Wraps `mori-index` and `mori-context` operations as MCP tools: `search_code`,
//! `get_symbol_context`, `get_file_ast`, `find_similar_patterns`,
//! `get_index_stats`, `get_context`, `find_references`, `find_implementations`,
//! `get_callers`, and `workspace_map`.

use mori_index::Index;
use mori_index::graph::Direction;
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
        ToolDefinition {
            name: "get_context".into(),
            description: "Search the index and return rich context blocks with source snippets, related symbols, and documentation. Supports keyword, similarity, and hybrid search strategies.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query (symbol name or natural language)" },
                    "strategy": { "type": "string", "description": "Search strategy: keyword (default), similar, or hybrid" },
                    "limit": { "type": "number", "description": "Max context blocks to return (default 10)" },
                    "context_lines": { "type": "number", "description": "Lines of source context around each symbol (default 10)" },
                    "threshold": { "type": "number", "description": "Similarity threshold for similar/hybrid strategies (default 0.5)" },
                    "format": { "type": "string", "description": "Output format: json (default) or markdown" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "find_references".into(),
            description: "Find all references to a symbol: callers, importers, type users, and implementors.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Name of the symbol to find references for" },
                    "limit": { "type": "number", "description": "Max results (default 50)" }
                },
                "required": ["symbol_name"]
            }),
        },
        ToolDefinition {
            name: "find_implementations".into(),
            description: "Find all types that implement a given trait.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trait_name": { "type": "string", "description": "Name of the trait to find implementations for" }
                },
                "required": ["trait_name"]
            }),
        },
        ToolDefinition {
            name: "get_callers".into(),
            description: "Trace the transitive call/dependency graph: who calls this symbol (callers) or what it calls (callees), up to N hops deep.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Name of the symbol to trace" },
                    "depth": { "type": "number", "description": "Max hops (default 3, max 10)" },
                    "direction": { "type": "string", "description": "\"callers\" (default) traces what uses this symbol, \"callees\" traces what this symbol uses" }
                },
                "required": ["symbol_name"]
            }),
        },
        ToolDefinition {
            name: "workspace_map".into(),
            description: "Return the workspace crate graph: members, inter-crate dependencies, and indexed symbol counts per crate.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "detail_level": { "type": "string", "description": "\"summary\" (default) returns crate graph, \"full\" adds public symbol lists" }
                },
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
        "get_context" => tool_get_context(index, arguments),
        "find_references" => tool_find_references(index, arguments),
        "find_implementations" => tool_find_implementations(index, arguments),
        "get_callers" => tool_get_callers(index, arguments),
        "workspace_map" => tool_workspace_map(index, arguments),
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

    // Direct lookup by file path in the index database.
    let file_id = match index.db().file_id_by_path(file_path) {
        Ok(Some(id)) => id,
        Ok(None) => return tool_text(format!("No indexed symbols found for file: {file_path}")),
        Err(e) => return tool_error(format!("file lookup failed: {e}")),
    };

    let symbols = match index.db().symbols_in_file(file_id) {
        Ok(s) => s,
        Err(e) => return tool_error(format!("symbols_in_file failed: {e}")),
    };

    if symbols.is_empty() {
        return tool_text(format!("No indexed symbols found for file: {file_path}"));
    }

    let file_symbols: Vec<Value> = symbols
        .iter()
        .map(|s| {
            json!({
                "name": s.symbol.name,
                "line": s.symbol.line,
                "kind": format!("{:?}", s.symbol.kind),
                "signature": s.symbol.signature,
                "visibility": format!("{:?}", s.symbol.visibility),
            })
        })
        .collect();

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

fn tool_get_context(index: &mut Index, args: &Value) -> ToolCallResult {
    let query_str = match args.get("query").and_then(Value::as_str) {
        Some(q) => q,
        None => return tool_error("Missing required parameter: query".into()),
    };

    let strategy_str = args
        .get("strategy")
        .and_then(Value::as_str)
        .unwrap_or("keyword");
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
    let context_lines = args
        .get("context_lines")
        .and_then(Value::as_u64)
        .unwrap_or(10) as usize;
    let threshold = args.get("threshold").and_then(Value::as_f64).unwrap_or(0.5) as f32;
    let format = args.get("format").and_then(Value::as_str).unwrap_or("json");

    let strategy = match strategy_str {
        "keyword" => mori_context::SearchStrategy::Keyword,
        "similar" => mori_context::SearchStrategy::Similar { threshold },
        "hybrid" => mori_context::SearchStrategy::Hybrid { threshold },
        other => return tool_error(format!("Unknown strategy: {other}")),
    };

    let ctx_query = mori_context::ContextQuery {
        query: query_str.to_string(),
        strategy,
        limit,
        context_lines,
        include_related: true,
    };

    match mori_context::assemble(index, &ctx_query) {
        Ok(response) => {
            let output = match format {
                "markdown" => response.to_markdown(),
                _ => response.to_json(),
            };
            tool_text(output)
        }
        Err(e) => tool_error(format!("context assembly failed: {e}")),
    }
}

fn tool_find_references(index: &Index, args: &Value) -> ToolCallResult {
    let symbol_name = match args.get("symbol_name").and_then(Value::as_str) {
        Some(n) => n,
        None => return tool_error("Missing required parameter: symbol_name".into()),
    };

    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;

    // Find the symbol by name.
    let results = match index.db().search_by_name(symbol_name, 5) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    // Prefer exact name match, fall back to first result.
    let sym = results
        .iter()
        .find(|s| s.symbol.name == symbol_name)
        .or_else(|| results.first());

    let Some(sym) = sym else {
        return tool_error(format!("Symbol not found: {symbol_name}"));
    };

    let referrers = match index.db().symbol_referrers_detailed(sym.id) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("symbol_referrers_detailed failed: {e}")),
    };

    let items: Vec<Value> = referrers
        .iter()
        .take(limit)
        .map(|(s, ref_kind, from_line)| {
            json!({
                "name": s.symbol.name,
                "file": s.symbol.file,
                "line": s.symbol.line,
                "kind": format!("{:?}", s.symbol.kind),
                "signature": s.symbol.signature,
                "ref_kind": ref_kind,
                "from_line": from_line,
            })
        })
        .collect();

    let output = json!({
        "symbol": symbol_name,
        "reference_count": referrers.len(),
        "references": items,
    });

    tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
}

fn tool_find_implementations(index: &Index, args: &Value) -> ToolCallResult {
    let trait_name = match args.get("trait_name").and_then(Value::as_str) {
        Some(n) => n,
        None => return tool_error("Missing required parameter: trait_name".into()),
    };

    // Search for the trait.
    let results = match index.db().search_by_name(trait_name, 10) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    // Find a result that is actually a Trait.
    let trait_sym = results
        .iter()
        .find(|s| {
            s.symbol.kind == mori_index::symbol::SymbolKind::Trait && s.symbol.name == trait_name
        })
        .or_else(|| {
            results
                .iter()
                .find(|s| s.symbol.kind == mori_index::symbol::SymbolKind::Trait)
        });

    let Some(trait_sym) = trait_sym else {
        return tool_error(format!("Trait not found: {trait_name}"));
    };

    let impls = match index.db().impl_trait_referrers(trait_sym.id) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("impl_trait_referrers failed: {e}")),
    };

    let items: Vec<Value> = impls
        .iter()
        .map(|s| {
            json!({
                "name": s.symbol.name,
                "file": s.symbol.file,
                "line": s.symbol.line,
                "signature": s.symbol.signature,
            })
        })
        .collect();

    let output = json!({
        "trait": trait_name,
        "implementation_count": items.len(),
        "implementations": items,
    });

    tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
}

fn tool_get_callers(index: &mut Index, args: &Value) -> ToolCallResult {
    let symbol_name = match args.get("symbol_name").and_then(Value::as_str) {
        Some(n) => n,
        None => return tool_error("Missing required parameter: symbol_name".into()),
    };

    let depth = args
        .get("depth")
        .and_then(Value::as_u64)
        .unwrap_or(3)
        .min(10) as usize;

    let direction_str = args
        .get("direction")
        .and_then(Value::as_str)
        .unwrap_or("callers");

    let direction = match direction_str {
        "callers" => Direction::Reverse,
        "callees" => Direction::Forward,
        other => {
            return tool_error(format!(
                "Unknown direction: {other}. Use \"callers\" or \"callees\"."
            ));
        }
    };

    // Find the symbol.
    let results = match index.db().search_by_name(symbol_name, 5) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("search failed: {e}")),
    };

    let sym = results
        .iter()
        .find(|s| s.symbol.name == symbol_name)
        .or_else(|| results.first());

    let Some(sym) = sym else {
        return tool_error(format!("Symbol not found: {symbol_name}"));
    };

    // Ensure graph is built.
    if let Err(e) = index.rebuild_graph() {
        return tool_error(format!("graph rebuild failed: {e}"));
    }

    let graph = match index.graph() {
        Some(g) => g,
        None => return tool_error("Graph not available".into()),
    };

    let traversal = graph.transitive(sym.id, depth, direction);

    // Load symbol details for each reachable node.
    let mut items = Vec::with_capacity(traversal.len());
    for (node_id, node_depth) in &traversal {
        if let Ok(Some(node_sym)) = index.db().symbol_by_id(*node_id) {
            items.push(json!({
                "name": node_sym.symbol.name,
                "file": node_sym.symbol.file,
                "line": node_sym.symbol.line,
                "kind": format!("{:?}", node_sym.symbol.kind),
                "depth": node_depth,
            }));
        }
    }

    let output = json!({
        "symbol": symbol_name,
        "direction": direction_str,
        "max_depth": depth,
        "node_count": items.len(),
        "nodes": items,
    });

    tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
}

fn tool_workspace_map(index: &Index, args: &Value) -> ToolCallResult {
    let detail_level = args
        .get("detail_level")
        .and_then(Value::as_str)
        .unwrap_or("summary");

    let root = index.root();
    let root_cargo = root.join("Cargo.toml");

    let cargo_str = match std::fs::read_to_string(&root_cargo) {
        Ok(s) => s,
        Err(e) => return tool_error(format!("Failed to read {}: {e}", root_cargo.display())),
    };

    let cargo_val: toml::Value = match cargo_str.parse() {
        Ok(v) => v,
        Err(e) => return tool_error(format!("Failed to parse Cargo.toml: {e}")),
    };

    // Extract workspace members.
    let members = cargo_val
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // For each member, read its Cargo.toml and extract name + path deps.
    let mut crates = Vec::new();
    for member in &members {
        let member_cargo = root.join(member).join("Cargo.toml");
        let member_str = match std::fs::read_to_string(&member_cargo) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let member_val: toml::Value = match member_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let name = member_val
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or(member.as_str());

        // Collect path dependencies (internal workspace deps).
        let mut deps = Vec::new();
        if let Some(dep_table) = member_val.get("dependencies").and_then(|d| d.as_table()) {
            for (dep_name, dep_val) in dep_table {
                let has_path = dep_val.get("path").and_then(|p| p.as_str()).is_some();
                if has_path {
                    deps.push(dep_name.clone());
                }
            }
        }

        // Count indexed symbols for this crate's files.
        let prefix = format!("{member}/");
        let symbol_count = match index.db().all_file_paths() {
            Ok(paths) => {
                let file_count: usize = paths.iter().filter(|p| p.starts_with(&prefix)).count();
                // If we have files, count their symbols.
                if file_count > 0 && detail_level == "full" {
                    paths
                        .iter()
                        .filter(|p| p.starts_with(&prefix))
                        .map(|p| {
                            index
                                .db()
                                .file_id_by_path(p)
                                .ok()
                                .flatten()
                                .and_then(|fid| index.db().symbols_in_file(fid).ok())
                                .map_or(0, |s| s.len())
                        })
                        .sum::<usize>()
                } else {
                    0
                }
            }
            Err(_) => 0,
        };

        let mut crate_info = json!({
            "name": name,
            "path": member,
            "deps": deps,
        });

        if detail_level == "full" {
            crate_info["symbol_count"] = json!(symbol_count);
        }

        crates.push(crate_info);
    }

    let output = json!({
        "workspace_members": members.len(),
        "crates": crates,
    });

    tool_text(serde_json::to_string_pretty(&output).unwrap_or_default())
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

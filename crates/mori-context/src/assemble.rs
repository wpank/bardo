//! Context assembly: search → snippets → structured output.

use std::collections::{HashMap, HashSet};

use mori_index::Index;
use mori_index::search::SearchResult;
use serde::Serialize;

use crate::error::ContextError;
use crate::query::{ContextQuery, SearchStrategy};
use crate::snippet::{self, Snippet, SnippetConfig};

/// A single context block for one symbol.
#[derive(Debug, Clone, Serialize)]
pub struct ContextBlock {
    /// Symbol name.
    pub name: String,
    /// Symbol kind (e.g. "Function", "Struct").
    pub kind: String,
    /// File path relative to project root.
    pub file: String,
    /// Line number in the file.
    pub line: u32,
    /// Full signature.
    pub signature: String,
    /// Doc comment, if any.
    pub doc: Option<String>,
    /// Source code snippet around the symbol.
    pub snippet: Option<Snippet>,
    /// Related symbols (types referenced in signature).
    pub related: Vec<RelatedSymbol>,
    /// Search relevance score.
    pub score: f32,
}

/// A related symbol referenced from a primary result's signature.
#[derive(Debug, Clone, Serialize)]
pub struct RelatedSymbol {
    /// Symbol name.
    pub name: String,
    /// Symbol kind.
    pub kind: String,
    /// File path.
    pub file: String,
    /// Line number.
    pub line: u32,
    /// Signature.
    pub signature: String,
}

/// The full context response.
#[derive(Debug, Clone, Serialize)]
pub struct ContextResponse {
    /// The original query.
    pub query: String,
    /// Context blocks, ordered by relevance.
    pub blocks: Vec<ContextBlock>,
    /// Total symbols considered.
    pub total_candidates: usize,
}

impl ContextResponse {
    /// Render the context as a markdown string for LLM injection.
    pub fn to_markdown(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str(&format!("# Code Context: {}\n\n", self.query));

        for block in &self.blocks {
            out.push_str(&format!(
                "## {} `{}` ({}:{})\n",
                block.kind, block.name, block.file, block.line
            ));

            out.push_str(&format!("```rust\n{}\n```\n", block.signature));

            if let Some(ref doc) = block.doc {
                out.push_str(&format!("\n{doc}\n"));
            }

            if let Some(ref snip) = block.snippet {
                out.push_str(&format!(
                    "\n### Source (lines {}–{})\n```rust\n{}\n```\n",
                    snip.start_line, snip.end_line, snip.code
                ));
            }

            if !block.related.is_empty() {
                out.push_str("\n### Related\n");
                for rel in &block.related {
                    out.push_str(&format!(
                        "- {} `{}` ({}:{}): `{}`\n",
                        rel.kind, rel.name, rel.file, rel.line, rel.signature
                    ));
                }
            }

            out.push('\n');
        }

        out
    }

    /// Render as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Assemble context from the index for a given query.
pub fn assemble(index: &mut Index, query: &ContextQuery) -> Result<ContextResponse, ContextError> {
    let results = run_search(index, query)?;
    let total_candidates = results.len();

    let snip_config = SnippetConfig::from_context_lines(query.context_lines);
    let root = index.root().to_path_buf();

    let mut blocks = Vec::with_capacity(results.len().min(query.limit));
    let mut seen: HashSet<(String, u32)> = HashSet::new();

    for result in results.into_iter().take(query.limit) {
        let key = (result.symbol.file.clone(), result.symbol.line);
        if !seen.insert(key) {
            continue;
        }

        let snippet =
            snippet::extract_snippet(&root, &result.symbol.file, result.symbol.line, &snip_config)
                .ok();

        let related = if query.include_related {
            find_related(index, &result)?
        } else {
            Vec::new()
        };

        blocks.push(ContextBlock {
            name: result.symbol.name,
            kind: format!("{:?}", result.symbol.kind),
            file: result.symbol.file,
            line: result.symbol.line,
            signature: result.symbol.signature,
            doc: result.symbol.doc,
            snippet,
            related,
            score: result.score,
        });
    }

    Ok(ContextResponse {
        query: query.query.clone(),
        blocks,
        total_candidates,
    })
}

/// Run the appropriate search strategy.
fn run_search(index: &mut Index, query: &ContextQuery) -> Result<Vec<SearchResult>, ContextError> {
    let per_list = query.limit * 3;

    match &query.strategy {
        SearchStrategy::Keyword => Ok(index.search(&query.query, per_list)?),
        SearchStrategy::Similar { threshold } => {
            // Need a symbol to fingerprint from — search by name first.
            let primary = index.search(&query.query, 1)?;
            let Some(sym) = primary.first() else {
                return Err(ContextError::NoResults(query.query.clone()));
            };
            let fp = mori_index::fingerprint::fingerprint(&sym.symbol);
            Ok(index.search_similar(&fp, *threshold, per_list)?)
        }
        SearchStrategy::Hybrid { threshold } => {
            // Run keyword + HDC, merge via RRF.
            let keyword_results = index.search(&query.query, per_list)?;

            // For HDC, we need a fingerprint. Use the top keyword result if available.
            let hdc_results = if let Some(sym) = keyword_results.first() {
                let fp = mori_index::fingerprint::fingerprint(&sym.symbol);
                index.search_similar(&fp, *threshold, per_list)?
            } else {
                Vec::new()
            };

            Ok(rrf_merge(keyword_results, hdc_results, query.limit * 2))
        }
    }
}

/// Reciprocal Rank Fusion of two result lists.
fn rrf_merge(
    list_a: Vec<SearchResult>,
    list_b: Vec<SearchResult>,
    limit: usize,
) -> Vec<SearchResult> {
    const K: f32 = 60.0;

    // key: (file, name, line)
    let mut merged: HashMap<(String, String, u32), (f32, SearchResult)> = HashMap::new();

    for (rank, result) in list_a.into_iter().enumerate() {
        let key = (
            result.symbol.file.clone(),
            result.symbol.name.clone(),
            result.symbol.line,
        );
        let rrf = 1.0 / (K + rank as f32 + 1.0);
        let entry = merged.entry(key).or_insert((0.0, result));
        entry.0 += rrf;
    }

    for (rank, result) in list_b.into_iter().enumerate() {
        let key = (
            result.symbol.file.clone(),
            result.symbol.name.clone(),
            result.symbol.line,
        );
        let rrf = 1.0 / (K + rank as f32 + 1.0);
        merged
            .entry(key)
            .and_modify(|(score, _)| *score += rrf)
            .or_insert((rrf, result));
    }

    let mut results: Vec<SearchResult> = merged
        .into_values()
        .map(|(score, mut r)| {
            r.score = score;
            r
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

/// Find related symbols by extracting type-like names from a result's signature.
fn find_related(index: &Index, result: &SearchResult) -> Result<Vec<RelatedSymbol>, ContextError> {
    let sig = &result.symbol.signature;
    let primary_name = &result.symbol.name;

    let type_names: Vec<String> = sig
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| {
            let first = w.chars().next().unwrap_or('a');
            first.is_uppercase() && *w != primary_name
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .map(String::from)
        .collect();

    let mut related = Vec::new();
    for name in &type_names {
        if related.len() >= 5 {
            break;
        }
        if let Ok(results) = index.search(name, 1) {
            for r in results {
                if r.symbol.name != *primary_name {
                    related.push(RelatedSymbol {
                        name: r.symbol.name,
                        kind: format!("{:?}", r.symbol.kind),
                        file: r.symbol.file,
                        line: r.symbol.line,
                        signature: r.symbol.signature,
                    });
                }
            }
        }
    }

    Ok(related)
}

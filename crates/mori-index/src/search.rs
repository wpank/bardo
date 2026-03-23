//! Search operations over the symbol index.

use std::collections::HashMap;

use crate::db::Db;
use crate::error::IndexError;
use crate::symbol::{Symbol, SymbolKind, Visibility};
use bardo_primitives::HdcVector;

/// A search result with scoring and match metadata.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched symbol.
    pub symbol: Symbol,
    /// Relevance score (interpretation depends on `match_kind`).
    pub score: f32,
    /// How this result was matched.
    pub match_kind: MatchKind,
}

/// How a search result was found.
#[derive(Debug, Clone)]
pub enum MatchKind {
    /// Matched by keyword/name search.
    Keyword,
    /// Matched by structural query (kind, visibility).
    Structural,
    /// Matched by HDC vector similarity.
    Similarity {
        /// The cosine-like similarity score (0.0 to 1.0).
        similarity: f32,
    },
    /// Matched by dense embedding similarity.
    #[cfg(feature = "embedding")]
    Embedding {
        /// Cosine similarity score (0.0 to 1.0).
        similarity: f32,
    },
    /// Matched by hybrid search (multiple strategies fused via RRF).
    Hybrid {
        /// Which strategies contributed to this result.
        sources: Vec<String>,
    },
}

/// Search symbols by name (keyword LIKE match).
///
/// # Errors
///
/// Returns `IndexError::Db` on database failure.
pub fn search_keyword(db: &Db, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
    let db_results = db.search_by_name(query, limit)?;
    Ok(db_results
        .into_iter()
        .enumerate()
        .map(|(i, db_sym)| SearchResult {
            symbol: db_sym.symbol,
            score: 1.0 - (i as f32 * 0.01), // simple decreasing score
            match_kind: MatchKind::Keyword,
        })
        .collect())
}

/// Search symbols by kind and optional visibility.
///
/// # Errors
///
/// Returns `IndexError::Db` on database failure.
pub fn search_structural(
    db: &Db,
    kind: SymbolKind,
    visibility: Option<Visibility>,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexError> {
    let db_results = db.search_by_kind(kind, visibility, limit)?;
    Ok(db_results
        .into_iter()
        .enumerate()
        .map(|(i, db_sym)| SearchResult {
            symbol: db_sym.symbol,
            score: 1.0 - (i as f32 * 0.01),
            match_kind: MatchKind::Structural,
        })
        .collect())
}

/// Search symbols by HDC fingerprint similarity.
///
/// Loads all HDC vectors from the database, computes similarity against the
/// query fingerprint, filters by threshold, and returns the top results.
///
/// # Errors
///
/// Returns `IndexError::Db` on database failure.
pub fn search_similar(
    db: &Db,
    query_fingerprint: &HdcVector,
    threshold: f32,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexError> {
    let all = db.all_hdc_symbols()?;
    let mut scored: Vec<SearchResult> = Vec::new();

    for (_id, symbol, blob) in all {
        if blob.len() != 1280 {
            continue;
        }
        let mut arr = [0u8; 1280];
        arr.copy_from_slice(&blob);
        let vec = HdcVector::from_bytes(&arr);
        let sim = query_fingerprint.similarity(&vec);

        if sim >= threshold {
            scored.push(SearchResult {
                symbol,
                score: sim,
                match_kind: MatchKind::Similarity { similarity: sim },
            });
        }
    }

    // Sort by similarity descending
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(limit);
    Ok(scored)
}

/// RRF constant. Standard value from the original Cormack et al. paper.
const RRF_K: f32 = 60.0;

/// Identity key for deduplicating symbols across result lists.
fn symbol_key(s: &Symbol) -> (String, String, u32) {
    (s.file.clone(), s.name.clone(), s.line)
}

/// Reciprocal Rank Fusion: merge multiple ranked lists into one.
///
/// Each input is a `(label, results)` pair. The label identifies the strategy
/// (e.g. "keyword", "embedding") and ends up in `MatchKind::Hybrid::sources`.
///
/// For each result at rank `r` (0-indexed) in a list, its RRF contribution is
/// `1 / (k + r + 1)`. Scores are summed across all lists where a symbol appears.
fn rrf_merge(lists: Vec<(&str, Vec<SearchResult>)>, limit: usize) -> Vec<SearchResult> {
    // Map from symbol key -> (accumulated RRF score, best Symbol, source labels)
    let mut merged: HashMap<(String, String, u32), (f32, Symbol, Vec<String>)> = HashMap::new();

    for (label, results) in lists {
        for (rank, result) in results.into_iter().enumerate() {
            let key = symbol_key(&result.symbol);
            let rrf_score = 1.0 / (RRF_K + rank as f32 + 1.0);

            let entry = merged
                .entry(key)
                .or_insert_with(|| (0.0, result.symbol.clone(), Vec::new()));
            entry.0 += rrf_score;
            entry.2.push(label.to_string());
        }
    }

    let mut results: Vec<SearchResult> = merged
        .into_values()
        .map(|(score, symbol, sources)| SearchResult {
            symbol,
            score,
            match_kind: MatchKind::Hybrid { sources },
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

/// Hybrid search: runs keyword + HDC similarity and merges results with RRF.
///
/// When the `embedding` feature is enabled and `embedding_store` is provided,
/// embedding-based semantic search is included in the fusion.
///
/// # Errors
///
/// Returns `IndexError` on database or embedding failure.
pub fn search_hybrid(
    db: &Db,
    query: &str,
    query_fingerprint: Option<&HdcVector>,
    #[cfg(feature = "embedding")] embedding_store: Option<&mut crate::embedding::EmbeddingStore>,
    similarity_threshold: f32,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexError> {
    // Per-strategy limit is higher to give RRF enough candidates.
    let per_list = limit * 3;

    let mut lists: Vec<(&str, Vec<SearchResult>)> = Vec::new();

    // 1. Keyword search
    let keyword_results = search_keyword(db, query, per_list)?;
    if !keyword_results.is_empty() {
        lists.push(("keyword", keyword_results));
    }

    // 2. HDC similarity search (if fingerprint provided)
    if let Some(fp) = query_fingerprint {
        let sim_results = search_similar(db, fp, similarity_threshold, per_list)?;
        if !sim_results.is_empty() {
            lists.push(("hdc", sim_results));
        }
    }

    // 3. Embedding search (if feature enabled and store provided)
    #[cfg(feature = "embedding")]
    if let Some(store) = embedding_store {
        let emb_results =
            crate::embedding::search_embedding(db, store, query, similarity_threshold, per_list)?;
        if !emb_results.is_empty() {
            lists.push(("embedding", emb_results));
        }
    }

    if lists.is_empty() {
        return Ok(vec![]);
    }

    Ok(rrf_merge(lists, limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{Symbol, SymbolKind, Visibility};

    fn make_symbol(name: &str, line: u32) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            file: "test.rs".to_string(),
            line,
            signature: format!("fn {name}()"),
            visibility: Visibility::Public,
            doc: None,
            content_hash: [0u8; 32],
        }
    }

    fn make_result(name: &str, line: u32, score: f32, kind: MatchKind) -> SearchResult {
        SearchResult {
            symbol: make_symbol(name, line),
            score,
            match_kind: kind,
        }
    }

    #[test]
    fn rrf_merge_single_list() {
        let lists = vec![(
            "keyword",
            vec![
                make_result("foo", 1, 1.0, MatchKind::Keyword),
                make_result("bar", 2, 0.9, MatchKind::Keyword),
            ],
        )];

        let merged = rrf_merge(lists, 10);
        assert_eq!(merged.len(), 2);
        // First result should be "foo" (rank 0 -> higher RRF score)
        assert_eq!(merged[0].symbol.name, "foo");
        assert_eq!(merged[1].symbol.name, "bar");
    }

    #[test]
    fn rrf_merge_shared_result_ranks_higher() {
        // "shared" appears in both lists, "only_kw" and "only_sim" appear in one each.
        let lists = vec![
            (
                "keyword",
                vec![
                    make_result("only_kw", 1, 1.0, MatchKind::Keyword),
                    make_result("shared", 10, 0.9, MatchKind::Keyword),
                ],
            ),
            (
                "hdc",
                vec![
                    make_result(
                        "only_sim",
                        2,
                        0.8,
                        MatchKind::Similarity { similarity: 0.8 },
                    ),
                    make_result("shared", 10, 0.7, MatchKind::Similarity { similarity: 0.7 }),
                ],
            ),
        ];

        let merged = rrf_merge(lists, 10);
        assert_eq!(merged.len(), 3);
        // "shared" should be first because it gets RRF contributions from both lists
        assert_eq!(merged[0].symbol.name, "shared");
        if let MatchKind::Hybrid { ref sources } = merged[0].match_kind {
            assert_eq!(sources.len(), 2);
        } else {
            panic!("expected Hybrid match kind");
        }
    }

    #[test]
    fn rrf_merge_respects_limit() {
        let results: Vec<_> = (0..20)
            .map(|i| {
                make_result(
                    &format!("sym_{i}"),
                    i,
                    1.0 - i as f32 * 0.01,
                    MatchKind::Keyword,
                )
            })
            .collect();
        let lists = vec![("keyword", results)];
        let merged = rrf_merge(lists, 5);
        assert_eq!(merged.len(), 5);
    }
}

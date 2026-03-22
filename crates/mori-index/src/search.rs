//! Search operations over the symbol index.

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

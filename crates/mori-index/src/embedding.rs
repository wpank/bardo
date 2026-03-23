//! Embedding-based semantic search for code symbols.
//!
//! Wraps `fastembed` to provide dense vector representations of code symbols,
//! enabling natural-language search over the index. Vectors are stored in SQLite
//! alongside the main index for simplicity.

use crate::db::Db;
use crate::error::IndexError;
use crate::search::{MatchKind, SearchResult};
use crate::symbol::Symbol;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

/// Embedding vector dimensionality for the default model (BGE-small-en-v1.5).
const EMBEDDING_DIM: usize = 384;

/// Manages embedding generation and storage.
pub struct EmbeddingStore {
    model: TextEmbedding,
}

impl EmbeddingStore {
    /// Create a new embedding store with the default model.
    ///
    /// Uses `BGE-small-en-v1.5` (33M params) for fast local inference.
    /// The model is downloaded on first use (~50MB).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Embedding` if the model fails to load.
    pub fn new() -> Result<Self, IndexError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true),
        )
        .map_err(|e| IndexError::Embedding(e.to_string()))?;

        Ok(Self { model })
    }

    /// Embed a single text string into a dense vector.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Embedding` on model failure.
    pub fn embed_text(&mut self, text: &str) -> Result<Vec<f32>, IndexError> {
        let results = self
            .model
            .embed(vec![text], None)
            .map_err(|e| IndexError::Embedding(e.to_string()))?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| IndexError::Embedding("empty embedding result".into()))
    }

    /// Embed multiple texts in a batch (more efficient than one-by-one).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Embedding` on model failure.
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, IndexError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        self.model
            .embed(texts, None)
            .map_err(|e| IndexError::Embedding(e.to_string()))
    }

    /// Build the text representation of a symbol for embedding.
    ///
    /// Produces a natural language string that captures the symbol's meaning.
    pub fn symbol_text(symbol: &Symbol) -> String {
        let mut text = format!("{} {}: {}", symbol.kind, symbol.name, symbol.signature);
        if let Some(ref doc) = symbol.doc {
            text.push('\n');
            text.push_str(doc);
        }
        text
    }
}

/// Run the embeddings migration on the database. Idempotent.
///
/// # Errors
///
/// Returns `IndexError::Db` on SQL failure.
pub fn migrate_embeddings(db: &Db) -> Result<(), IndexError> {
    db.conn().execute_batch(
        "CREATE TABLE IF NOT EXISTS embeddings (
            symbol_id INTEGER PRIMARY KEY REFERENCES symbols(id) ON DELETE CASCADE,
            vector BLOB NOT NULL
        );",
    )?;
    Ok(())
}

/// Store an embedding vector for a symbol.
///
/// # Errors
///
/// Returns `IndexError::Db` on SQL failure.
pub fn store_embedding(db: &Db, symbol_id: i64, vector: &[f32]) -> Result<(), IndexError> {
    let blob = vector_to_bytes(vector);
    db.conn().execute(
        "INSERT OR REPLACE INTO embeddings (symbol_id, vector) VALUES (?1, ?2)",
        rusqlite::params![symbol_id, blob],
    )?;
    Ok(())
}

/// Get count of symbols that have embeddings.
///
/// # Errors
///
/// Returns `IndexError::Db` on SQL failure.
pub fn embedding_count(db: &Db) -> Result<usize, IndexError> {
    let count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))?;
    Ok(count as usize)
}

/// Get symbol IDs that don't have embeddings yet.
///
/// # Errors
///
/// Returns `IndexError::Db` on SQL failure.
pub fn unembedded_symbol_ids(
    db: &Db,
) -> Result<Vec<(i64, String, String, Option<String>, String)>, IndexError> {
    let mut stmt = db.conn().prepare(
        "SELECT s.id, s.name, s.signature, s.doc, s.kind
         FROM symbols s
         WHERE s.id NOT IN (SELECT symbol_id FROM embeddings)",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Search symbols by embedding similarity.
///
/// Embeds the query, loads all symbol embeddings, computes cosine similarity,
/// and returns the top results above the threshold.
///
/// # Errors
///
/// Returns `IndexError` on database or embedding failure.
pub fn search_embedding(
    db: &Db,
    store: &mut EmbeddingStore,
    query: &str,
    threshold: f32,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexError> {
    let query_vec = store.embed_text(query)?;

    let mut stmt = db.conn().prepare(
        "SELECT e.symbol_id, e.vector,
                s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, f.path
         FROM embeddings e
         JOIN symbols s ON e.symbol_id = s.id
         LEFT JOIN files f ON s.file_id = f.id",
    )?;

    let mut scored: Vec<SearchResult> = Vec::new();

    let rows = stmt.query_map([], |row| {
        let _symbol_id: i64 = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let name: String = row.get(2)?;
        let kind_str: String = row.get(3)?;
        let line: u32 = row.get(4)?;
        let signature: String = row.get(5)?;
        let visibility_str: String = row.get(6)?;
        let doc: Option<String> = row.get(7)?;
        let content_hash_blob: Option<Vec<u8>> = row.get(8)?;
        let file_path: Option<String> = row.get(9)?;

        Ok((
            blob,
            name,
            kind_str,
            line,
            signature,
            visibility_str,
            doc,
            content_hash_blob,
            file_path,
        ))
    })?;

    for row in rows {
        let (
            blob,
            name,
            kind_str,
            line,
            signature,
            visibility_str,
            doc,
            content_hash_blob,
            file_path,
        ) = row?;

        let vec = bytes_to_vector(&blob);
        if vec.len() != query_vec.len() {
            continue;
        }

        let sim = cosine_similarity(&query_vec, &vec);
        if sim < threshold {
            continue;
        }

        let mut content_hash = [0u8; 32];
        if let Some(blob) = content_hash_blob {
            let len = blob.len().min(32);
            content_hash[..len].copy_from_slice(&blob[..len]);
        }

        scored.push(SearchResult {
            symbol: Symbol {
                name,
                kind: crate::db::str_to_kind_pub(&kind_str),
                file: file_path.unwrap_or_default(),
                line,
                signature,
                visibility: crate::db::str_to_visibility_pub(&visibility_str),
                doc,
                content_hash,
            },
            score: sim,
            match_kind: MatchKind::Embedding { similarity: sim },
        });
    }

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(limit);
    Ok(scored)
}

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 { 0.0 } else { dot / denom }
}

/// Serialize a float vector to bytes (little-endian f32).
fn vector_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Deserialize bytes back to a float vector.
fn bytes_to_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            f32::from_le_bytes(arr)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn vector_roundtrip() {
        let original = vec![1.5, -2.3, 0.0, 42.0];
        let bytes = vector_to_bytes(&original);
        let recovered = bytes_to_vector(&bytes);
        assert_eq!(original, recovered);
    }
}

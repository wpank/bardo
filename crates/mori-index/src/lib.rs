//! `mori-index` -- AST-based code index with HDC fingerprinting.
//!
//! Provides incremental Rust source indexing backed by SQLite, with hyperdimensional
//! computing fingerprints for structural similarity search and PageRank-based
//! symbol ranking.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod db;
#[cfg(feature = "embedding")]
pub mod embedding;
pub mod error;
pub mod fingerprint;
pub mod graph;
#[cfg(feature = "salsa-memo")]
#[allow(missing_docs)]
pub mod memo;
pub mod parser;
pub mod search;
#[cfg(feature = "snapshot")]
#[allow(unsafe_code)]
pub mod snapshot;
pub mod symbol;
pub mod update;

use std::path::{Path, PathBuf};

use crate::db::{Db, IndexStats};
use crate::error::IndexError;
use crate::graph::SymbolGraph;
use crate::parser::RustParser;
use crate::search::SearchResult;
use crate::symbol::{SymbolKind, Visibility};
use crate::update::UpdateStats;
use bardo_primitives::HdcVector;

/// The main index handle. Wraps the database, parser, and optional graph.
pub struct Index {
    db: Db,
    root: PathBuf,
    graph: Option<SymbolGraph>,
    /// Whether the graph needs rebuilding (set after any update with changes).
    graph_dirty: bool,
    #[cfg(feature = "snapshot")]
    snap: Option<snapshot::MmapSnapshot>,
}

impl Index {
    /// Open (or create) an index at the given project root.
    ///
    /// Creates a `.mori/` directory if it doesn't exist, opens the SQLite database
    /// at `{root}/.mori/index.db`, and runs migrations.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` if the directory can't be created or the DB fails to open.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, IndexError> {
        let root = root.as_ref().to_path_buf();
        let mori_dir = root.join(".mori");
        std::fs::create_dir_all(&mori_dir)?;

        let db_path = mori_dir.join("index.db");
        let db_path_str = db_path.to_string_lossy().to_string();
        let db = Db::open(&db_path_str)?;
        db.migrate()?;
        #[cfg(feature = "embedding")]
        embedding::migrate_embeddings(&db)?;

        #[cfg(feature = "snapshot")]
        let snap = {
            let snap_path = mori_dir.join("index.snapshot.rkyv");
            snapshot::MmapSnapshot::open(&snap_path).ok()
        };

        Ok(Self {
            db,
            root,
            graph: None,
            graph_dirty: true,
            #[cfg(feature = "snapshot")]
            snap,
        })
    }

    /// Open an index with a pre-configured database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns `IndexError` if migration fails.
    pub fn open_with_db(db: Db, root: impl AsRef<Path>) -> Result<Self, IndexError> {
        db.migrate()?;
        #[cfg(feature = "embedding")]
        embedding::migrate_embeddings(&db)?;
        Ok(Self {
            db,
            root: root.as_ref().to_path_buf(),
            graph: None,
            graph_dirty: true,
            #[cfg(feature = "snapshot")]
            snap: None,
        })
    }

    /// Run an incremental update: scan the project root for changed `.rs` files,
    /// re-parse them, and update the index.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on I/O, parse, or database failures.
    pub fn update(&mut self) -> Result<UpdateStats, IndexError> {
        let mut parser = RustParser::new()?;
        let root = self.root.clone();
        let stats = update::incremental_update(&mut self.db, &mut parser, &root)?;

        let had_changes =
            stats.files_changed > 0 || stats.files_added > 0 || stats.files_removed > 0;

        if had_changes {
            self.graph_dirty = true;
        }

        #[cfg(feature = "snapshot")]
        if had_changes {
            let snap_path = self.root.join(".mori/index.snapshot.rkyv");
            if let Err(e) = snapshot::write_snapshot(&self.db, &snap_path) {
                tracing::warn!("snapshot write failed (non-fatal): {e}");
            } else {
                self.snap = snapshot::MmapSnapshot::open(&snap_path).ok();
            }
        }

        Ok(stats)
    }

    /// Rebuild the in-memory symbol graph from the database.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn rebuild_graph(&mut self) -> Result<(), IndexError> {
        if !self.graph_dirty && self.graph.is_some() {
            return Ok(());
        }
        let edges = self.db.load_graph_edges()?;
        let node_count = self.db.symbol_count()?;
        self.graph = Some(SymbolGraph::from_edges(&edges, node_count));
        self.graph_dirty = false;
        Ok(())
    }

    /// Search symbols by name.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        #[cfg(feature = "snapshot")]
        if let Some(ref snap) = self.snap {
            return Ok(snap.search_by_name(query, limit));
        }
        search::search_keyword(&self.db, query, limit)
    }

    /// Search symbols by kind and optional visibility.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn search_kind(
        &self,
        kind: SymbolKind,
        visibility: Option<Visibility>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, IndexError> {
        search::search_structural(&self.db, kind, visibility, limit)
    }

    /// Search symbols by HDC fingerprint similarity.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn search_similar(
        &self,
        fp: &HdcVector,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<SearchResult>, IndexError> {
        #[cfg(feature = "snapshot")]
        if let Some(ref snap) = self.snap {
            return Ok(snap.search_similar(fp, threshold, limit));
        }
        search::search_similar(&self.db, fp, threshold, limit)
    }

    /// Hybrid search: fuses keyword, HDC, and (optionally) embedding results via RRF.
    ///
    /// Pass `query_fingerprint` if you have an HDC vector for the query.
    /// When the `embedding` feature is enabled, pass an `EmbeddingStore` for
    /// semantic search inclusion.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database or embedding failure.
    pub fn search_hybrid(
        &self,
        query: &str,
        query_fingerprint: Option<&HdcVector>,
        #[cfg(feature = "embedding")] embedding_store: Option<&mut embedding::EmbeddingStore>,
        similarity_threshold: f32,
        limit: usize,
    ) -> Result<Vec<SearchResult>, IndexError> {
        search::search_hybrid(
            &self.db,
            query,
            query_fingerprint,
            #[cfg(feature = "embedding")]
            embedding_store,
            similarity_threshold,
            limit,
        )
    }

    /// Get PageRank-scored symbols, optionally biased toward specific files.
    ///
    /// Requires `rebuild_graph()` to have been called first.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::NotInitialized` if the graph hasn't been built.
    pub fn ranked_symbols(
        &self,
        bias_file_ids: &[i64],
        iterations: usize,
        damping: f32,
    ) -> Result<Vec<(i64, f32)>, IndexError> {
        let graph = self.graph.as_ref().ok_or(IndexError::NotInitialized)?;
        let file_map = self.db.symbol_file_map()?;
        Ok(graph.pagerank(bias_file_ids, &file_map, iterations, damping))
    }

    /// Get index statistics.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn stats(&self) -> Result<IndexStats, IndexError> {
        self.db.stats()
    }

    /// Get the project root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read source code from a file relative to the project root.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Io` if the file can't be read.
    pub fn read_source(&self, rel_path: &str) -> Result<String, IndexError> {
        let full = self.root.join(rel_path);
        Ok(std::fs::read_to_string(full)?)
    }

    /// Access the database directly (for advanced queries).
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Access the in-memory symbol graph, if built.
    ///
    /// Call `rebuild_graph()` first to ensure the graph is available.
    pub fn graph(&self) -> Option<&SymbolGraph> {
        self.graph.as_ref()
    }

    /// Generate embeddings for symbols that don't have them yet.
    ///
    /// Requires the `embedding` feature. Lazily initializes the embedding model
    /// on first call (~50MB download).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Embedding` on model or generation failures.
    #[cfg(feature = "embedding")]
    pub fn embed_symbols(&self) -> Result<usize, IndexError> {
        let unembedded = embedding::unembedded_symbol_ids(&self.db)?;
        if unembedded.is_empty() {
            return Ok(0);
        }

        tracing::info!(
            count = unembedded.len(),
            "generating embeddings for new symbols"
        );
        let mut store = embedding::EmbeddingStore::new()?;

        // Build text representations and batch embed.
        let texts: Vec<String> = unembedded
            .iter()
            .map(|(_, name, sig, doc, kind)| {
                let mut text = format!("{kind} {name}: {sig}");
                if let Some(d) = doc {
                    text.push('\n');
                    text.push_str(d);
                }
                text
            })
            .collect();

        let ids: Vec<i64> = unembedded.iter().map(|(id, ..)| *id).collect();

        // Batch in chunks of 64 to avoid OOM on large indexes.
        let mut total = 0;
        for chunk_start in (0..texts.len()).step_by(64) {
            let chunk_end = (chunk_start + 64).min(texts.len());
            let chunk_texts = texts[chunk_start..chunk_end].to_vec();
            let chunk_ids = &ids[chunk_start..chunk_end];

            let vectors = store.embed_batch(chunk_texts)?;
            for (id, vec) in chunk_ids.iter().zip(vectors.iter()) {
                embedding::store_embedding(&self.db, *id, vec)?;
            }
            total += chunk_ids.len();
        }

        tracing::info!(embedded = total, "embedding generation complete");
        Ok(total)
    }

    /// Returns `true` if a snapshot is loaded and available for fast reads.
    #[cfg(feature = "snapshot")]
    pub fn has_snapshot(&self) -> bool {
        self.snap.is_some()
    }
}

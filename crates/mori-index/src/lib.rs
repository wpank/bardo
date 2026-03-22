//! `mori-index` -- AST-based code index with HDC fingerprinting.
//!
//! Provides incremental Rust source indexing backed by SQLite, with hyperdimensional
//! computing fingerprints for structural similarity search and PageRank-based
//! symbol ranking.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod db;
pub mod error;
pub mod fingerprint;
pub mod graph;
pub mod parser;
pub mod search;
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

        Ok(Self {
            db,
            root,
            graph: None,
        })
    }

    /// Open an index with a pre-configured database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns `IndexError` if migration fails.
    pub fn open_with_db(db: Db, root: impl AsRef<Path>) -> Result<Self, IndexError> {
        db.migrate()?;
        Ok(Self {
            db,
            root: root.as_ref().to_path_buf(),
            graph: None,
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
        update::incremental_update(&mut self.db, &mut parser, &root)
    }

    /// Rebuild the in-memory symbol graph from the database.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn rebuild_graph(&mut self) -> Result<(), IndexError> {
        let edges = self.db.load_graph_edges()?;
        let node_count = self.db.symbol_count()?;
        self.graph = Some(SymbolGraph::from_edges(&edges, node_count));
        Ok(())
    }

    /// Search symbols by name.
    ///
    /// # Errors
    ///
    /// Returns `IndexError` on database failure.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
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
        search::search_similar(&self.db, fp, threshold, limit)
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
}

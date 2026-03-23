//! Salsa-based memoization layer for incremental computation.
//!
//! Sits on top of the SQLite persistence layer. Memoizes:
//! - File content → parsed symbols (skip parse if file bytes unchanged)
//! - Symbol → fingerprint (skip if symbol signature unchanged)
//!
//! The salsa database lives on the `Index` struct and persists across
//! calls to `update()`. When a file's content changes, dependent tracked
//! functions are automatically re-executed; unchanged files get cache hits.

use bardo_primitives::HdcVector;

use crate::fingerprint;
use crate::parser::RustParser;
use crate::symbol::{Symbol, SymbolRef};

/// The salsa database trait.
#[salsa::db]
pub trait MemoDb: salsa::Database {}

/// Concrete salsa database.
#[salsa::db]
#[derive(Default)]
pub struct MemoDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for MemoDatabase {}

#[salsa::db]
impl MemoDb for MemoDatabase {}

/// A source file known to the index. Salsa input — the source of truth.
///
/// When `content_hash` or `content` are updated via their setters,
/// all tracked functions that depend on this file are invalidated.
#[salsa::input]
pub struct SourceFile {
    /// Relative path from project root.
    pub path: String,

    /// Blake3 hash of the file content (32 bytes).
    pub content_hash: Vec<u8>,

    /// Raw file bytes.
    pub content: Vec<u8>,
}

/// Tracked function: parse a file into symbols.
///
/// Re-executes only when the `SourceFile`'s content changes.
/// On cache hit, returns the previously computed result instantly.
#[salsa::tracked]
pub fn parsed_symbols(db: &dyn MemoDb, file: SourceFile) -> Vec<Symbol> {
    let path = file.path(db);
    let content = file.content(db);

    let mut parser = RustParser::new().expect("parser init");
    let result = parser.parse_file(&path, &content).expect("parse failed");

    result.symbols
}

/// Tracked function: extract refs from a file.
#[salsa::tracked]
pub fn parsed_refs(db: &dyn MemoDb, file: SourceFile) -> Vec<SymbolRef> {
    let path = file.path(db);
    let content = file.content(db);

    let mut parser = RustParser::new().expect("parser init");
    let result = parser.parse_file(&path, &content).expect("parse failed");

    result.refs
}

/// Tracked function: compute HDC fingerprints for all symbols in a file.
///
/// Depends on `parsed_symbols`, so it's automatically invalidated when
/// the file's parse result changes.
#[salsa::tracked]
pub fn file_fingerprints(db: &dyn MemoDb, file: SourceFile) -> Vec<HdcVector> {
    let symbols = parsed_symbols(db, file);
    symbols
        .iter()
        .map(|sym| fingerprint::fingerprint(sym))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_parse_caches() {
        let db = MemoDatabase::default();

        let source = b"pub fn hello() {}\npub struct World;";
        let file = SourceFile::new(
            &db,
            "test.rs".into(),
            blake3::hash(source).as_bytes().to_vec(),
            source.to_vec(),
        );

        // First call: actually parses
        let symbols = parsed_symbols(&db, file);
        assert!(
            symbols.len() >= 2,
            "should parse at least 2 symbols, got {}",
            symbols.len()
        );

        // Second call: cache hit (same result)
        let symbols2 = parsed_symbols(&db, file);
        assert_eq!(symbols2.len(), symbols.len());
    }

    #[test]
    fn memo_fingerprints_deterministic() {
        let db = MemoDatabase::default();

        let source = b"pub fn foo(x: u32) -> String { todo!() }";
        let file = SourceFile::new(
            &db,
            "test.rs".into(),
            blake3::hash(source).as_bytes().to_vec(),
            source.to_vec(),
        );

        let fps1 = file_fingerprints(&db, file);
        let fps2 = file_fingerprints(&db, file);

        assert_eq!(fps1.len(), fps2.len());
        for (a, b) in fps1.iter().zip(fps2.iter()) {
            assert!(
                (a.similarity(b) - 1.0).abs() < 1e-6,
                "fingerprints should be identical"
            );
        }
    }

    #[test]
    fn memo_invalidates_on_change() {
        use salsa::Setter;

        let mut db = MemoDatabase::default();

        let source1 = b"pub fn original() {}";
        let file = SourceFile::new(
            &db,
            "test.rs".into(),
            blake3::hash(source1).as_bytes().to_vec(),
            source1.to_vec(),
        );

        let symbols1 = parsed_symbols(&db, file);
        let name1 = &symbols1[0].name;
        assert!(name1.contains("original"), "first parse: {name1}");

        // Update the file content
        let source2 = b"pub fn changed() {}";
        file.set_content_hash(&mut db)
            .to(blake3::hash(source2).as_bytes().to_vec());
        file.set_content(&mut db).to(source2.to_vec());

        let symbols2 = parsed_symbols(&db, file);
        let name2 = &symbols2[0].name;
        assert!(
            name2.contains("changed"),
            "second parse after update: {name2}"
        );
    }
}

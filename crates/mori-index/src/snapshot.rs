//! Memory-mapped index snapshot for zero-copy reads.
//!
//! After each incremental update, the full symbol index is serialized to an rkyv
//! archive file. On subsequent opens, the snapshot is mmap'd for zero-copy access
//! to symbol data, HDC fingerprints, and refs — bypassing SQLite row-by-row
//! deserialization entirely.

use std::path::Path;

use bardo_primitives::HdcVector;
use rkyv::rancor::Error as RkyvError;

use crate::db::Db;
use crate::error::IndexError;
use crate::search::{MatchKind, SearchResult};
use crate::symbol::{Symbol, SymbolKind, Visibility};

// ---------------------------------------------------------------------------
// Snapshot data types
// ---------------------------------------------------------------------------

/// Top-level archived index. Contains all symbols, file paths, and refs.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct IndexSnapshot {
    /// All symbols, sorted by name for binary-search keyword lookup.
    pub symbols: Vec<SnapshotSymbol>,
    /// Deduplicated file paths. Symbols reference these by index.
    pub file_paths: Vec<String>,
    /// All references between symbols.
    pub refs: Vec<SnapshotRef>,
    /// Summary stats at snapshot time.
    pub stats: SnapshotStats,
}

/// A symbol in the snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct SnapshotSymbol {
    /// SQLite row id.
    pub id: i64,
    /// Symbol name.
    pub name: String,
    /// `SymbolKind` as u8 discriminant.
    pub kind: u8,
    /// Index into `IndexSnapshot::file_paths`.
    pub file_idx: u32,
    /// Line number (1-indexed).
    pub line: u32,
    /// Signature text.
    pub signature: String,
    /// `Visibility` as u8 discriminant.
    pub visibility: u8,
    /// Doc comment.
    pub doc: Option<String>,
    /// Blake3 content hash.
    pub content_hash: [u8; 32],
    /// HDC fingerprint (1280 bytes) if computed.
    pub hdc_bytes: Option<Vec<u8>>,
}

/// A reference in the snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct SnapshotRef {
    /// Source symbol id.
    pub from_id: i64,
    /// Target symbol name.
    pub to_name: String,
    /// Resolved target symbol id.
    pub to_id: Option<i64>,
    /// `RefKind` as u8 discriminant.
    pub ref_kind: u8,
}

/// Summary counts.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct SnapshotStats {
    /// Number of indexed files.
    pub file_count: u32,
    /// Number of indexed symbols.
    pub symbol_count: u32,
    /// Number of references.
    pub ref_count: u32,
}

// ---------------------------------------------------------------------------
// SymbolKind / Visibility ↔ u8
// ---------------------------------------------------------------------------

fn kind_to_u8(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Function => 0,
        SymbolKind::Struct => 1,
        SymbolKind::Enum => 2,
        SymbolKind::Trait => 3,
        SymbolKind::TypeAlias => 4,
        SymbolKind::Const => 5,
        SymbolKind::Module => 6,
        SymbolKind::Impl => 7,
        SymbolKind::Use => 8,
        SymbolKind::Macro => 9,
    }
}

fn u8_to_kind(v: u8) -> SymbolKind {
    match v {
        0 => SymbolKind::Function,
        1 => SymbolKind::Struct,
        2 => SymbolKind::Enum,
        3 => SymbolKind::Trait,
        4 => SymbolKind::TypeAlias,
        5 => SymbolKind::Const,
        6 => SymbolKind::Module,
        7 => SymbolKind::Impl,
        8 => SymbolKind::Use,
        9 => SymbolKind::Macro,
        _ => SymbolKind::Function,
    }
}

fn vis_to_u8(vis: Visibility) -> u8 {
    match vis {
        Visibility::Public => 0,
        Visibility::Crate => 1,
        Visibility::Restricted => 2,
        Visibility::Private => 3,
    }
}

fn u8_to_vis(v: u8) -> Visibility {
    match v {
        0 => Visibility::Public,
        1 => Visibility::Crate,
        2 => Visibility::Restricted,
        _ => Visibility::Private,
    }
}

fn ref_kind_to_u8(kind: crate::symbol::RefKind) -> u8 {
    use crate::symbol::RefKind;
    match kind {
        RefKind::Import => 0,
        RefKind::TypeRef => 1,
        RefKind::Call => 2,
        RefKind::ImplTrait => 3,
    }
}

// ---------------------------------------------------------------------------
// Write snapshot
// ---------------------------------------------------------------------------

/// Serialize the current index state to a snapshot file.
///
/// Writes atomically: serializes to a temp file, then renames over the target.
///
/// # Errors
///
/// Returns `IndexError::Snapshot` on serialization or I/O failure.
pub fn write_snapshot(db: &Db, snapshot_path: &Path) -> Result<(), IndexError> {
    // Build deduplicated file path list
    let all_paths = db.all_file_paths()?;
    let mut path_to_idx: std::collections::HashMap<String, u32> =
        std::collections::HashMap::with_capacity(all_paths.len());
    let mut file_paths: Vec<String> = Vec::with_capacity(all_paths.len());
    for path in &all_paths {
        let idx = file_paths.len() as u32;
        path_to_idx.insert(path.clone(), idx);
        file_paths.push(path.clone());
    }

    // Load all symbols
    let db_symbols = db.all_hdc_symbols_full()?;
    let mut symbols: Vec<SnapshotSymbol> = db_symbols
        .into_iter()
        .map(|(id, sym, hdc_blob)| {
            let file_idx = path_to_idx.get(&sym.file).copied().unwrap_or(u32::MAX);
            SnapshotSymbol {
                id,
                name: sym.name,
                kind: kind_to_u8(sym.kind),
                file_idx,
                line: sym.line,
                signature: sym.signature,
                visibility: vis_to_u8(sym.visibility),
                doc: sym.doc,
                content_hash: sym.content_hash,
                hdc_bytes: if hdc_blob.is_empty() {
                    None
                } else {
                    Some(hdc_blob)
                },
            }
        })
        .collect();

    // Sort by name for binary-search keyword lookup
    symbols.sort_by(|a, b| a.name.cmp(&b.name));

    // Load all refs
    let db_refs = db.all_refs()?;
    let refs: Vec<SnapshotRef> = db_refs
        .into_iter()
        .map(|(from_id, to_name, to_id, ref_kind)| SnapshotRef {
            from_id,
            to_name,
            to_id,
            ref_kind: ref_kind_to_u8(ref_kind),
        })
        .collect();

    let stats = db.stats()?;
    let snapshot = IndexSnapshot {
        symbols,
        file_paths,
        refs,
        stats: SnapshotStats {
            file_count: stats.files as u32,
            symbol_count: stats.symbols as u32,
            ref_count: stats.refs as u32,
        },
    };

    // Serialize
    let bytes = rkyv::to_bytes::<RkyvError>(&snapshot)
        .map_err(|e| IndexError::Snapshot(format!("serialize: {e}")))?;

    // Write atomically
    let tmp_path = snapshot_path.with_extension("rkyv.tmp");
    std::fs::write(&tmp_path, &bytes)?;
    std::fs::rename(&tmp_path, snapshot_path)?;

    tracing::info!(
        size_bytes = bytes.len(),
        symbols = snapshot.stats.symbol_count,
        "snapshot written"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Memory-mapped snapshot reader
// ---------------------------------------------------------------------------

/// A memory-mapped snapshot for zero-copy reads.
pub struct MmapSnapshot {
    // The mmap must live as long as we access archived data.
    // The archived reference is derived from the mmap bytes on each access.
    mmap: memmap2::Mmap,
}

impl MmapSnapshot {
    /// Open and validate a snapshot file.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Snapshot` if the file can't be opened or is invalid.
    ///
    /// # Safety note
    ///
    /// The mmap call is unsafe because the OS could theoretically modify the
    /// file while it's mapped. We mitigate this by opening read-only and
    /// writing snapshots atomically (write tmp + rename).
    pub fn open(path: &Path) -> Result<Self, IndexError> {
        if !path.exists() {
            return Err(IndexError::Snapshot("snapshot file not found".into()));
        }

        let file = std::fs::File::open(path)?;

        // SAFETY: File is opened read-only. Snapshots are written atomically
        // (write to .tmp, then rename), so partial reads cannot occur. The mmap
        // remains valid for the lifetime of MmapSnapshot which owns the Mmap.
        #[allow(unsafe_code)]
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file) }
            .map_err(|e| IndexError::Snapshot(format!("mmap failed: {e}")))?;

        // Validate the archived data with bytecheck
        let _archived = rkyv::access::<ArchivedIndexSnapshot, RkyvError>(&mmap)
            .map_err(|e| IndexError::Snapshot(format!("invalid snapshot: {e}")))?;

        Ok(Self { mmap })
    }

    /// Get the archived root. The returned reference borrows from the mmap.
    fn archived(&self) -> &ArchivedIndexSnapshot {
        // Already validated in open(). This will not fail.
        rkyv::access::<ArchivedIndexSnapshot, RkyvError>(&self.mmap)
            .expect("snapshot was validated on open")
    }

    /// Number of symbols in the snapshot.
    pub fn symbol_count(&self) -> usize {
        self.archived().symbols.len()
    }

    /// Number of files in the snapshot.
    pub fn file_count(&self) -> usize {
        self.archived().file_paths.len()
    }

    /// Search symbols by HDC fingerprint similarity (zero-copy).
    ///
    /// Iterates the archived symbol data without heap allocation per symbol.
    /// Only allocates for matching results above threshold.
    pub fn search_similar(
        &self,
        query: &HdcVector,
        threshold: f32,
        limit: usize,
    ) -> Vec<SearchResult> {
        let archived = self.archived();
        let mut scored: Vec<SearchResult> = Vec::new();

        for sym in archived.symbols.iter() {
            let hdc_bytes = match &sym.hdc_bytes {
                rkyv::option::ArchivedOption::Some(bytes) => bytes,
                rkyv::option::ArchivedOption::None => continue,
            };
            if hdc_bytes.len() != 1280 {
                continue;
            }

            let mut arr = [0u8; 1280];
            arr.copy_from_slice(hdc_bytes.as_slice());
            let vec = HdcVector::from_bytes(&arr);
            let sim = query.similarity(&vec);

            if sim >= threshold {
                let file = archived_sym_file(&archived.file_paths, &sym.file_idx);
                scored.push(SearchResult {
                    symbol: archived_sym_to_symbol(sym, &file),
                    score: sim,
                    match_kind: MatchKind::Similarity { similarity: sim },
                });
            }
        }

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);
        scored
    }

    /// Search symbols by name substring (case-insensitive).
    ///
    /// Since symbols are sorted by name, we can do a linear scan with early
    /// termination for prefix matches. For general substring matches, we scan
    /// all symbols (same as the SQLite LIKE approach).
    pub fn search_by_name(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let archived = self.archived();
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        for (i, sym) in archived.symbols.iter().enumerate() {
            let name: &str = &sym.name;
            if name.to_lowercase().contains(&query_lower) {
                let file = archived_sym_file(&archived.file_paths, &sym.file_idx);
                results.push(SearchResult {
                    symbol: archived_sym_to_symbol(sym, &file),
                    score: 1.0 - (i as f32 * 0.0001),
                    match_kind: MatchKind::Keyword,
                });
                if results.len() >= limit {
                    break;
                }
            }
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a file_idx to a file path string from the archived file_paths vec.
fn archived_sym_file(
    file_paths: &rkyv::vec::ArchivedVec<rkyv::string::ArchivedString>,
    file_idx: &rkyv::Archived<u32>,
) -> String {
    let idx: u32 = (*file_idx).into();
    let idx = idx as usize;
    if idx < file_paths.len() {
        file_paths[idx].as_str().to_owned()
    } else {
        String::new()
    }
}

/// Convert an archived symbol to a regular `Symbol`.
fn archived_sym_to_symbol(sym: &ArchivedSnapshotSymbol, file: &str) -> Symbol {
    let kind_u8: u8 = sym.kind.into();
    let vis_u8: u8 = sym.visibility.into();
    let mut content_hash = [0u8; 32];
    content_hash.copy_from_slice(&sym.content_hash);

    Symbol {
        name: sym.name.as_str().to_owned(),
        kind: u8_to_kind(kind_u8),
        file: file.to_owned(),
        line: sym.line.into(),
        signature: sym.signature.as_str().to_owned(),
        visibility: u8_to_vis(vis_u8),
        doc: match &sym.doc {
            rkyv::option::ArchivedOption::Some(d) => Some(d.as_str().to_owned()),
            rkyv::option::ArchivedOption::None => None,
        },
        content_hash,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::symbol::{RefKind, Symbol, SymbolKind, Visibility};

    fn test_db() -> Db {
        let db = Db::open_in_memory().expect("open in-memory db");
        db.migrate().expect("migrate");
        db
    }

    fn sample_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            file: "test.rs".to_string(),
            line: 1,
            signature: format!("fn {name}()"),
            visibility: Visibility::Public,
            doc: None,
            content_hash: blake3::hash(format!("fn {name}()").as_bytes()).into(),
        }
    }

    #[test]
    fn snapshot_roundtrip() {
        let db = test_db();
        let hash = blake3::hash(b"content").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).expect("upsert");

        let syms = vec![
            sample_symbol("alpha", SymbolKind::Function),
            sample_symbol("beta", SymbolKind::Struct),
            sample_symbol("gamma", SymbolKind::Function),
        ];
        let ids = db.insert_symbols(file_id, &syms).expect("insert");

        // Set HDC fingerprints
        for &id in &ids {
            let fp = HdcVector::from_seed(format!("fp-{id}").as_bytes());
            db.set_hdc(id, &fp).expect("set_hdc");
        }

        // Insert a ref
        let refs = vec![crate::symbol::SymbolRef {
            from_file: "test.rs".into(),
            from_line: 5,
            target: "beta".into(),
            ref_kind: RefKind::Call,
        }];
        db.insert_refs(&refs, &[ids[0]]).expect("insert_refs");

        // Write snapshot to a temp file
        let tmp_dir = std::env::temp_dir().join("mori-snapshot-test");
        std::fs::create_dir_all(&tmp_dir).expect("mkdir");
        let snap_path = tmp_dir.join("test.snapshot.rkyv");
        write_snapshot(&db, &snap_path).expect("write_snapshot");

        // Open and validate
        let snap = MmapSnapshot::open(&snap_path).expect("open snapshot");
        assert_eq!(snap.symbol_count(), 3);
        assert_eq!(snap.file_count(), 1);

        // Keyword search
        let results = snap.search_by_name("alpha", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.name, "alpha");

        // HDC similarity search — querying with the same fingerprint should match
        let query_fp = HdcVector::from_seed(format!("fp-{}", ids[0]).as_bytes());
        let results = snap.search_similar(&query_fp, 0.9, 10);
        assert!(!results.is_empty(), "should find at least 1 similar symbol");

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn snapshot_sorted_by_name() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).expect("upsert");

        // Insert in reverse alphabetical order
        let syms = vec![
            sample_symbol("zebra", SymbolKind::Function),
            sample_symbol("apple", SymbolKind::Struct),
            sample_symbol("mango", SymbolKind::Function),
        ];
        db.insert_symbols(file_id, &syms).expect("insert");

        let tmp_dir = std::env::temp_dir().join("mori-snapshot-sort-test");
        std::fs::create_dir_all(&tmp_dir).expect("mkdir");
        let snap_path = tmp_dir.join("test.snapshot.rkyv");
        write_snapshot(&db, &snap_path).expect("write_snapshot");

        let snap = MmapSnapshot::open(&snap_path).expect("open");
        let archived = snap.archived();

        // Verify sorted order
        let names: Vec<&str> = archived.symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["apple", "mango", "zebra"]);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}

//! Incremental index update logic.
//!
//! Walks a directory tree for `.rs` files, compares file hashes against the database,
//! and re-parses only changed or new files.

use std::path::Path;
use std::time::Instant;

use crate::db::Db;
use crate::error::IndexError;
use crate::fingerprint;
use crate::parser::RustParser;

/// Statistics from an incremental update.
#[derive(Debug, Default, Clone)]
pub struct UpdateStats {
    /// Total .rs files found on disk.
    pub files_scanned: usize,
    /// Files whose content changed since last index.
    pub files_changed: usize,
    /// Files not previously in the index.
    pub files_added: usize,
    /// Files in the index that no longer exist on disk.
    pub files_removed: usize,
    /// Total symbols inserted in this update.
    pub symbols_added: usize,
    /// Time spent parsing, in milliseconds.
    pub parse_time_ms: u64,
    /// Time spent on database operations, in milliseconds.
    pub db_time_ms: u64,
}

/// Perform an incremental update of the index.
///
/// Walks `root` for `*.rs` files (skipping hidden dirs and `target/`), hashes each file,
/// compares with the database, and re-parses changed or new files. Deleted files are
/// removed from the index. All DB writes happen in a single transaction.
///
/// # Errors
///
/// Returns `IndexError` on I/O, parse, or database failures.
pub fn incremental_update(
    db: &mut Db,
    parser: &mut RustParser,
    root: &Path,
) -> Result<UpdateStats, IndexError> {
    let mut stats = UpdateStats::default();

    // Collect all .rs files on disk
    let mut rs_files: Vec<String> = Vec::new();
    collect_rs_files(root, root, &mut rs_files)?;
    stats.files_scanned = rs_files.len();

    // Get currently indexed files for deletion detection
    let indexed_paths = db.all_file_paths()?;
    let indexed_set: std::collections::HashSet<String> = indexed_paths.into_iter().collect();
    let disk_set: std::collections::HashSet<String> = rs_files.iter().cloned().collect();

    // Start transaction
    let conn = db.conn_mut();
    let tx = conn.transaction()?;

    let mut total_parse_ms = 0u64;
    let mut total_db_ms = 0u64;

    // Process each file
    for rel_path in &rs_files {
        let abs_path = root.join(rel_path);
        let source = std::fs::read(&abs_path)?;
        let hash = blake3::hash(&source);
        let hash_bytes = hash.as_bytes().to_vec();

        // Check if file is unchanged
        let stored_hash: Option<Vec<u8>> = tx
            .query_row(
                "SELECT hash FROM files WHERE path = ?1",
                rusqlite::params![rel_path],
                |row| row.get(0),
            )
            .ok();

        let is_new = stored_hash.is_none();
        let is_changed = stored_hash
            .as_ref()
            .map_or(true, |h| h.as_slice() != hash_bytes.as_slice());

        if !is_changed {
            continue;
        }

        if is_new {
            stats.files_added += 1;
        } else {
            stats.files_changed += 1;
        }

        // Parse
        let t0 = Instant::now();
        let parse_result = parser.parse_file(rel_path, &source)?;
        total_parse_ms += t0.elapsed().as_millis() as u64;

        // DB writes
        let t1 = Instant::now();

        // Upsert file
        tx.execute(
            "INSERT INTO files (path, hash, mtime_ns) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET hash = excluded.hash, mtime_ns = excluded.mtime_ns",
            rusqlite::params![rel_path, &hash_bytes, 0i64],
        )?;
        let file_id: i64 = tx.query_row(
            "SELECT id FROM files WHERE path = ?1",
            rusqlite::params![rel_path],
            |row| row.get(0),
        )?;

        // Clear old symbols (cascades to refs)
        tx.execute(
            "DELETE FROM symbols WHERE file_id = ?1",
            rusqlite::params![file_id],
        )?;

        // Insert symbols
        let mut symbol_ids = Vec::with_capacity(parse_result.symbols.len());
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (file_id, name, kind, line, signature, visibility, doc, content_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;

            for sym in &parse_result.symbols {
                stmt.execute(rusqlite::params![
                    file_id,
                    sym.name,
                    kind_to_str(sym.kind),
                    sym.line,
                    sym.signature,
                    vis_to_str(sym.visibility),
                    sym.doc,
                    &sym.content_hash[..],
                ])?;
                symbol_ids.push(tx.last_insert_rowid());
            }
        }

        stats.symbols_added += symbol_ids.len();

        // Fingerprint and store HDC vectors
        for (sym, &sym_id) in parse_result.symbols.iter().zip(symbol_ids.iter()) {
            let fp = fingerprint::fingerprint(sym);
            let bytes = fp.to_bytes();
            tx.execute(
                "UPDATE symbols SET hdc_blob = ?1 WHERE id = ?2",
                rusqlite::params![&bytes[..], sym_id],
            )?;
        }

        // Insert refs (map each ref to its closest symbol by line)
        if !parse_result.refs.is_empty() && !symbol_ids.is_empty() {
            let mut ref_stmt = tx.prepare(
                "INSERT INTO refs (from_symbol, to_name, ref_kind, from_line)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for r in &parse_result.refs {
                // Find the closest preceding symbol for this ref
                let from_id =
                    find_enclosing_symbol(r.from_line, &parse_result.symbols, &symbol_ids);

                ref_stmt.execute(rusqlite::params![
                    from_id,
                    r.target,
                    ref_kind_to_str(r.ref_kind),
                    r.from_line,
                ])?;
            }
        }

        total_db_ms += t1.elapsed().as_millis() as u64;
    }

    // Remove deleted files
    for path in &indexed_set {
        if !disk_set.contains(path) {
            tx.execute("DELETE FROM files WHERE path = ?1", rusqlite::params![path])?;
            stats.files_removed += 1;
        }
    }

    // Resolve refs
    tx.execute(
        "UPDATE refs SET to_symbol = (
            SELECT s.id FROM symbols s WHERE s.name = refs.to_name LIMIT 1
        ) WHERE to_symbol IS NULL",
        [],
    )?;

    tx.commit()?;

    stats.parse_time_ms = total_parse_ms;
    stats.db_time_ms = total_db_ms;

    Ok(stats)
}

/// Find the symbol id that encloses a given line (closest preceding symbol).
fn find_enclosing_symbol(line: u32, symbols: &[crate::symbol::Symbol], ids: &[i64]) -> i64 {
    let mut best_id = ids.first().copied().unwrap_or(0);
    let mut best_line = 0u32;

    for (sym, &id) in symbols.iter().zip(ids.iter()) {
        if sym.line <= line && sym.line >= best_line {
            best_id = id;
            best_line = sym.line;
        }
    }

    best_id
}

/// Recursively collect .rs files under root, storing paths relative to root.
fn collect_rs_files(dir: &Path, root: &Path, results: &mut Vec<String>) -> Result<(), IndexError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(IndexError::Io(e)),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dirs and target/
        if file_name.starts_with('.') || file_name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_rs_files(&path, root, results)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(rel) = path.strip_prefix(root) {
                results.push(rel.to_string_lossy().to_string());
            }
        }
    }

    Ok(())
}

/// Convert `SymbolKind` to string for DB storage.
fn kind_to_str(kind: crate::symbol::SymbolKind) -> &'static str {
    use crate::symbol::SymbolKind;
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Trait => "trait",
        SymbolKind::TypeAlias => "type_alias",
        SymbolKind::Const => "const",
        SymbolKind::Module => "module",
        SymbolKind::Impl => "impl",
        SymbolKind::Use => "use",
        SymbolKind::Macro => "macro",
    }
}

/// Convert `Visibility` to string for DB storage.
fn vis_to_str(vis: crate::symbol::Visibility) -> &'static str {
    use crate::symbol::Visibility;
    match vis {
        Visibility::Public => "public",
        Visibility::Crate => "crate",
        Visibility::Restricted => "restricted",
        Visibility::Private => "private",
    }
}

/// Convert `RefKind` to string for DB storage.
fn ref_kind_to_str(kind: crate::symbol::RefKind) -> &'static str {
    use crate::symbol::RefKind;
    match kind {
        RefKind::Import => "import",
        RefKind::TypeRef => "type_ref",
        RefKind::Call => "call",
        RefKind::ImplTrait => "impl_trait",
    }
}

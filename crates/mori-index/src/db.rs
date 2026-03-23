//! SQLite storage layer for the symbol index.

use crate::error::IndexError;
use crate::symbol::{Symbol, SymbolKind, SymbolRef, Visibility};
use bardo_primitives::HdcVector;
use rusqlite::{Connection, OptionalExtension, params};

/// Database handle for the symbol index.
pub struct Db {
    /// The underlying SQLite connection.
    conn: Connection,
}

/// Statistics about the current index.
#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    /// Number of indexed files.
    pub files: usize,
    /// Number of indexed symbols.
    pub symbols: usize,
    /// Number of references.
    pub refs: usize,
    /// Number of resolved references (where to_symbol is not NULL).
    pub resolved_refs: usize,
}

/// A symbol loaded from the database with its row id.
#[derive(Debug, Clone)]
pub struct DbSymbol {
    /// Row id in the symbols table.
    pub id: i64,
    /// The symbol data.
    pub symbol: Symbol,
    /// HDC fingerprint blob, if computed.
    pub hdc_blob: Option<Vec<u8>>,
}

impl Db {
    /// Open (or create) a database at the given path with WAL mode.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on connection failure.
    pub fn open(path: &str) -> Result<Self, IndexError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on connection failure.
    pub fn open_in_memory() -> Result<Self, IndexError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// Access the underlying connection (for transactions).
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Access the underlying connection mutably (for transactions).
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Run schema migrations. Idempotent.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn migrate(&self) -> Result<(), IndexError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                hash BLOB,
                mtime_ns INTEGER
            );

            CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                line INTEGER NOT NULL,
                signature TEXT NOT NULL,
                visibility TEXT NOT NULL,
                doc TEXT,
                content_hash BLOB,
                hdc_blob BLOB
            );

            CREATE TABLE IF NOT EXISTS refs (
                id INTEGER PRIMARY KEY,
                from_symbol INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
                to_name TEXT NOT NULL,
                to_symbol INTEGER REFERENCES symbols(id) ON DELETE SET NULL,
                ref_kind TEXT NOT NULL,
                from_line INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
            CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);
            CREATE INDEX IF NOT EXISTS idx_refs_from ON refs(from_symbol);
            CREATE INDEX IF NOT EXISTS idx_refs_to ON refs(to_symbol);
            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
            ",
        )?;
        Ok(())
    }

    /// Get the stored hash for a file, or None if not indexed.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn file_hash(&self, path: &str) -> Result<Option<Vec<u8>>, IndexError> {
        let result = self
            .conn
            .query_row(
                "SELECT hash FROM files WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Get the file id for a path, or None if not indexed.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn file_id(&self, path: &str) -> Result<Option<i64>, IndexError> {
        let result = self
            .conn
            .query_row(
                "SELECT id FROM files WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Upsert a file record. Returns the file id.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn upsert_file(&self, path: &str, hash: &[u8], mtime_ns: i64) -> Result<i64, IndexError> {
        self.conn.execute(
            "INSERT INTO files (path, hash, mtime_ns)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET hash = excluded.hash, mtime_ns = excluded.mtime_ns",
            params![path, hash, mtime_ns],
        )?;

        let id = self.conn.query_row(
            "SELECT id FROM files WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    /// Delete all symbols (and cascading refs) for a file.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn clear_file_symbols(&self, file_id: i64) -> Result<(), IndexError> {
        self.conn
            .execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])?;
        Ok(())
    }

    /// Insert a batch of symbols for a file. Returns the inserted row ids.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn insert_symbols(&self, file_id: i64, symbols: &[Symbol]) -> Result<Vec<i64>, IndexError> {
        let mut ids = Vec::with_capacity(symbols.len());
        let mut stmt = self.conn.prepare(
            "INSERT INTO symbols (file_id, name, kind, line, signature, visibility, doc, content_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for sym in symbols {
            stmt.execute(params![
                file_id,
                sym.name,
                kind_to_str(sym.kind),
                sym.line,
                sym.signature,
                visibility_to_str(sym.visibility),
                sym.doc,
                &sym.content_hash[..],
            ])?;
            ids.push(self.conn.last_insert_rowid());
        }
        Ok(ids)
    }

    /// Insert a batch of references. `from_symbol_ids` maps each ref to its source symbol row id.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn insert_refs(
        &self,
        refs: &[SymbolRef],
        from_symbol_ids: &[i64],
    ) -> Result<(), IndexError> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO refs (from_symbol, to_name, ref_kind, from_line)
             VALUES (?1, ?2, ?3, ?4)",
        )?;

        for (r, &from_id) in refs.iter().zip(from_symbol_ids.iter()) {
            stmt.execute(params![
                from_id,
                r.target,
                ref_kind_to_str(r.ref_kind),
                r.from_line,
            ])?;
        }
        Ok(())
    }

    /// Set the HDC fingerprint for a symbol.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn set_hdc(&self, symbol_id: i64, hdc: &HdcVector) -> Result<(), IndexError> {
        let bytes = hdc.to_bytes();
        self.conn.execute(
            "UPDATE symbols SET hdc_blob = ?1 WHERE id = ?2",
            params![&bytes[..], symbol_id],
        )?;
        Ok(())
    }

    /// Attempt to resolve unresolved refs by matching to_name against symbol names.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn resolve_refs(&self) -> Result<usize, IndexError> {
        let count = self.conn.execute(
            "UPDATE refs SET to_symbol = (
                SELECT s.id FROM symbols s WHERE s.name = refs.to_name LIMIT 1
            ) WHERE to_symbol IS NULL",
            [],
        )?;
        Ok(count)
    }

    /// Search symbols by name using LIKE.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn search_by_name(&self, query: &str, limit: usize) -> Result<Vec<DbSymbol>, IndexError> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
             FROM symbols s LEFT JOIN files f ON s.file_id = f.id
             WHERE s.name LIKE ?1 LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(row_to_db_symbol(row)?)
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Search symbols by kind, optionally filtered by visibility.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn search_by_kind(
        &self,
        kind: SymbolKind,
        visibility: Option<Visibility>,
        limit: usize,
    ) -> Result<Vec<DbSymbol>, IndexError> {
        let kind_str = kind_to_str(kind);

        if let Some(vis) = visibility {
            let vis_str = visibility_to_str(vis);
            let mut stmt = self.conn.prepare(
                "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
                 FROM symbols s LEFT JOIN files f ON s.file_id = f.id
                 WHERE s.kind = ?1 AND s.visibility = ?2 LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![kind_str, vis_str, limit as i64], |row| {
                Ok(row_to_db_symbol(row)?)
            })?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
                 FROM symbols s LEFT JOIN files f ON s.file_id = f.id
                 WHERE s.kind = ?1 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![kind_str, limit as i64], |row| {
                Ok(row_to_db_symbol(row)?)
            })?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        }
    }

    /// Load all symbols that have an HDC fingerprint (id + blob).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn all_hdc_symbols(&self) -> Result<Vec<(i64, Symbol, Vec<u8>)>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
             FROM symbols s LEFT JOIN files f ON s.file_id = f.id
             WHERE s.hdc_blob IS NOT NULL",
        )?;

        let rows = stmt.query_map([], |row| {
            let db_sym = row_to_db_symbol(row)?;
            let blob = db_sym.hdc_blob.unwrap_or_default();
            Ok((db_sym.id, db_sym.symbol, blob))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Load all symbols with file paths resolved (for snapshot serialization).
    ///
    /// Returns `(id, Symbol, hdc_blob)` for ALL symbols (including those without fingerprints).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn all_hdc_symbols_full(&self) -> Result<Vec<(i64, Symbol, Vec<u8>)>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
             FROM symbols s LEFT JOIN files f ON s.file_id = f.id",
        )?;

        let rows = stmt.query_map([], |row| {
            let db_sym = row_to_db_symbol(row)?;
            let blob = db_sym.hdc_blob.unwrap_or_default();
            Ok((db_sym.id, db_sym.symbol, blob))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Load all references (for snapshot serialization).
    ///
    /// Returns `(from_symbol_id, to_name, to_symbol_id, ref_kind)`.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn all_refs(
        &self,
    ) -> Result<Vec<(i64, String, Option<i64>, crate::symbol::RefKind)>, IndexError> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_symbol, to_name, to_symbol, ref_kind FROM refs")?;

        let rows = stmt.query_map([], |row| {
            let from_id: i64 = row.get(0)?;
            let to_name: String = row.get(1)?;
            let to_id: Option<i64> = row.get(2)?;
            let ref_kind_str: String = row.get(3)?;
            Ok((from_id, to_name, to_id, str_to_ref_kind(&ref_kind_str)))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Load graph edges: (from_symbol, to_symbol) for all resolved refs.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn load_graph_edges(&self) -> Result<Vec<(i64, i64)>, IndexError> {
        let mut stmt = self
            .conn
            .prepare("SELECT from_symbol, to_symbol FROM refs WHERE to_symbol IS NOT NULL")?;

        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get index statistics.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn stats(&self) -> Result<IndexStats, IndexError> {
        let files: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        let symbols: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;
        let refs: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM refs", [], |row| row.get(0))?;
        let resolved_refs: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM refs WHERE to_symbol IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        Ok(IndexStats {
            files: files as usize,
            symbols: symbols as usize,
            refs: refs as usize,
            resolved_refs: resolved_refs as usize,
        })
    }

    /// Get all indexed file paths.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn all_file_paths(&self) -> Result<Vec<String>, IndexError> {
        let mut stmt = self.conn.prepare("SELECT path FROM files")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    /// Remove a file and all its symbols/refs (via cascade).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn remove_file(&self, path: &str) -> Result<(), IndexError> {
        self.conn
            .execute("DELETE FROM files WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// Get a mapping from symbol id to file id for all symbols.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_file_map(&self) -> Result<std::collections::HashMap<i64, i64>, IndexError> {
        let mut stmt = self.conn.prepare("SELECT id, file_id FROM symbols")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (id, file_id) = row?;
            map.insert(id, file_id);
        }
        Ok(map)
    }

    /// Get the total number of symbols (used for graph construction).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_count(&self) -> Result<usize, IndexError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get all symbols belonging to a file.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbols_in_file(&self, file_id: i64) -> Result<Vec<DbSymbol>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
             FROM symbols s LEFT JOIN files f ON s.file_id = f.id
             WHERE s.file_id = ?1 ORDER BY s.line",
        )?;

        let rows = stmt.query_map(params![file_id], |row| Ok(row_to_db_symbol(row)?))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Resolve a file path to its database id.
    ///
    /// This is an alias for `file_id` with a clearer name.
    pub fn file_id_by_path(&self, path: &str) -> Result<Option<i64>, IndexError> {
        self.file_id(path)
    }

    /// Get a symbol by its row id.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_by_id(&self, id: i64) -> Result<Option<DbSymbol>, IndexError> {
        let result = self
            .conn
            .query_row(
                "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
                 FROM symbols s LEFT JOIN files f ON s.file_id = f.id
                 WHERE s.id = ?1",
                params![id],
                |row| row_to_db_symbol(row),
            )
            .optional()?;
        Ok(result)
    }

    /// Get all symbols that reference a given symbol with full details.
    ///
    /// Returns `(referring_symbol, ref_kind, from_line)` for each reference.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_referrers_detailed(
        &self,
        symbol_id: i64,
    ) -> Result<Vec<(DbSymbol, String, u32)>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path, r.ref_kind, r.from_line
             FROM refs r
             JOIN symbols s ON r.from_symbol = s.id
             LEFT JOIN files f ON s.file_id = f.id
             WHERE r.to_symbol = ?1",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| {
            let sym = row_to_db_symbol(row)?;
            let ref_kind: String = row.get(11)?;
            let from_line: u32 = row.get(12)?;
            Ok((sym, ref_kind, from_line))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Find impl blocks that implement a given trait.
    ///
    /// Filters references by `ref_kind = 'impl_trait'`.
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn impl_trait_referrers(&self, symbol_id: i64) -> Result<Vec<DbSymbol>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.file_id, s.name, s.kind, s.line, s.signature, s.visibility, s.doc, s.content_hash, s.hdc_blob, f.path
             FROM refs r
             JOIN symbols s ON r.from_symbol = s.id
             LEFT JOIN files f ON s.file_id = f.id
             WHERE r.to_symbol = ?1 AND r.ref_kind = 'impl_trait'",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| row_to_db_symbol(row))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get all symbols that reference a given symbol (incoming edges).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_referrers(&self, symbol_id: i64) -> Result<Vec<(i64, String)>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT r.from_symbol, s.name FROM refs r
             JOIN symbols s ON r.from_symbol = s.id
             WHERE r.to_symbol = ?1",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get all symbols referenced by a given symbol (outgoing edges).
    ///
    /// # Errors
    ///
    /// Returns `IndexError::Db` on SQL failure.
    pub fn symbol_references(&self, symbol_id: i64) -> Result<Vec<(i64, String)>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT r.to_symbol, r.to_name FROM refs r
             WHERE r.from_symbol = ?1 AND r.to_symbol IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}

/// Convert a rusqlite row to a `DbSymbol`.
///
/// Expects columns: id, file_id, name, kind, line, signature, visibility, doc,
/// content_hash, hdc_blob, file_path (from LEFT JOIN files).
fn row_to_db_symbol(row: &rusqlite::Row) -> rusqlite::Result<DbSymbol> {
    let id: i64 = row.get(0)?;
    let _file_id: i64 = row.get(1)?;
    let name: String = row.get(2)?;
    let kind_str: String = row.get(3)?;
    let line: u32 = row.get(4)?;
    let signature: String = row.get(5)?;
    let visibility_str: String = row.get(6)?;
    let doc: Option<String> = row.get(7)?;
    let content_hash_blob: Option<Vec<u8>> = row.get(8)?;
    let hdc_blob: Option<Vec<u8>> = row.get(9)?;
    let file_path: Option<String> = row.get(10)?;

    let mut content_hash = [0u8; 32];
    if let Some(blob) = content_hash_blob {
        let len = blob.len().min(32);
        content_hash[..len].copy_from_slice(&blob[..len]);
    }

    Ok(DbSymbol {
        id,
        symbol: Symbol {
            name,
            kind: str_to_kind(&kind_str),
            file: file_path.unwrap_or_default(),
            line,
            signature,
            visibility: str_to_visibility(&visibility_str),
            doc,
            content_hash,
        },
        hdc_blob,
    })
}

/// Convert `SymbolKind` to its database string representation.
fn kind_to_str(kind: SymbolKind) -> &'static str {
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

/// Convert a database string back to `SymbolKind`.
fn str_to_kind(s: &str) -> SymbolKind {
    str_to_kind_pub(s)
}

/// Convert a database string back to `SymbolKind` (public for cross-module use).
pub fn str_to_kind_pub(s: &str) -> SymbolKind {
    match s {
        "function" => SymbolKind::Function,
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "type_alias" => SymbolKind::TypeAlias,
        "const" => SymbolKind::Const,
        "module" => SymbolKind::Module,
        "impl" => SymbolKind::Impl,
        "use" => SymbolKind::Use,
        "macro" => SymbolKind::Macro,
        _ => SymbolKind::Function, // fallback
    }
}

/// Convert `Visibility` to its database string representation.
fn visibility_to_str(vis: Visibility) -> &'static str {
    match vis {
        Visibility::Public => "public",
        Visibility::Crate => "crate",
        Visibility::Restricted => "restricted",
        Visibility::Private => "private",
    }
}

/// Convert `RefKind` to its database string representation.
fn ref_kind_to_str(kind: crate::symbol::RefKind) -> &'static str {
    use crate::symbol::RefKind;
    match kind {
        RefKind::Import => "import",
        RefKind::TypeRef => "type_ref",
        RefKind::Call => "call",
        RefKind::ImplTrait => "impl_trait",
    }
}

/// Convert a database string back to `Visibility`.
fn str_to_visibility(s: &str) -> Visibility {
    str_to_visibility_pub(s)
}

/// Convert a database string back to `Visibility` (public for cross-module use).
pub fn str_to_visibility_pub(s: &str) -> Visibility {
    match s {
        "public" => Visibility::Public,
        "crate" => Visibility::Crate,
        "restricted" => Visibility::Restricted,
        _ => Visibility::Private,
    }
}

/// Convert a database string back to `RefKind`.
fn str_to_ref_kind(s: &str) -> crate::symbol::RefKind {
    use crate::symbol::RefKind;
    match s {
        "import" => RefKind::Import,
        "type_ref" => RefKind::TypeRef,
        "call" => RefKind::Call,
        "impl_trait" => RefKind::ImplTrait,
        _ => RefKind::Call,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{RefKind, SymbolKind};

    fn test_db() -> Db {
        let db = Db::open_in_memory().unwrap_or_else(|e| {
            panic!("Failed to open in-memory db: {e}");
        });
        db.migrate().unwrap_or_else(|e| {
            panic!("Failed to migrate: {e}");
        });
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
    fn migrate_idempotent() {
        let db = test_db();
        // Running migrate again should not fail
        db.migrate().unwrap_or_else(|e| {
            panic!("Second migrate failed: {e}");
        });
        db.migrate().unwrap_or_else(|e| {
            panic!("Third migrate failed: {e}");
        });
    }

    #[test]
    fn upsert_file_and_lookup() {
        let db = test_db();
        let hash = blake3::hash(b"content").as_bytes().to_vec();
        let file_id = db
            .upsert_file("src/main.rs", &hash, 12345)
            .unwrap_or_else(|e| {
                panic!("upsert_file failed: {e}");
            });
        assert!(file_id > 0);

        let stored = db.file_hash("src/main.rs").unwrap_or_else(|e| {
            panic!("file_hash failed: {e}");
        });
        assert!(stored.is_some());
        assert_eq!(stored.unwrap_or_default(), hash);

        // Upsert again with new hash
        let new_hash = blake3::hash(b"new content").as_bytes().to_vec();
        let file_id2 = db
            .upsert_file("src/main.rs", &new_hash, 99999)
            .unwrap_or_else(|e| {
                panic!("upsert_file (2) failed: {e}");
            });
        assert_eq!(file_id, file_id2);

        let stored2 = db.file_hash("src/main.rs").unwrap_or_else(|e| {
            panic!("file_hash (2) failed: {e}");
        });
        assert_eq!(stored2.unwrap_or_default(), new_hash);
    }

    #[test]
    fn insert_and_search() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap_or_else(|e| {
            panic!("upsert failed: {e}");
        });

        let syms = vec![
            sample_symbol("my_function", SymbolKind::Function),
            sample_symbol("MyStruct", SymbolKind::Struct),
            sample_symbol("another_fn", SymbolKind::Function),
        ];

        let ids = db.insert_symbols(file_id, &syms).unwrap_or_else(|e| {
            panic!("insert_symbols failed: {e}");
        });
        assert_eq!(ids.len(), 3);

        // Search by name
        let results = db.search_by_name("my_func", 10).unwrap_or_else(|e| {
            panic!("search_by_name failed: {e}");
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.name, "my_function");

        // Search by kind
        let results = db
            .search_by_kind(SymbolKind::Function, None, 10)
            .unwrap_or_else(|e| {
                panic!("search_by_kind failed: {e}");
            });
        assert_eq!(results.len(), 2);

        // Search by kind + visibility
        let results = db
            .search_by_kind(SymbolKind::Function, Some(Visibility::Public), 10)
            .unwrap_or_else(|e| {
                panic!("search_by_kind with vis failed: {e}");
            });
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn cascade_delete() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap_or_else(|e| {
            panic!("upsert failed: {e}");
        });

        let syms = vec![sample_symbol("foo", SymbolKind::Function)];
        let ids = db.insert_symbols(file_id, &syms).unwrap_or_else(|e| {
            panic!("insert_symbols failed: {e}");
        });

        // Insert a ref
        let refs = vec![SymbolRef {
            from_file: "test.rs".to_string(),
            from_line: 5,
            target: "bar".to_string(),
            ref_kind: RefKind::Call,
        }];
        db.insert_refs(&refs, &ids).unwrap_or_else(|e| {
            panic!("insert_refs failed: {e}");
        });

        let stats_before = db.stats().unwrap_or_else(|e| {
            panic!("stats failed: {e}");
        });
        assert_eq!(stats_before.symbols, 1);
        assert_eq!(stats_before.refs, 1);

        // Remove the file (should cascade)
        db.remove_file("test.rs").unwrap_or_else(|e| {
            panic!("remove_file failed: {e}");
        });

        let stats_after = db.stats().unwrap_or_else(|e| {
            panic!("stats failed: {e}");
        });
        assert_eq!(stats_after.symbols, 0);
        assert_eq!(stats_after.refs, 0);
    }

    #[test]
    fn symbol_by_id_lookup() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap();

        let syms = vec![sample_symbol("target_fn", SymbolKind::Function)];
        let ids = db.insert_symbols(file_id, &syms).unwrap();

        let found = db.symbol_by_id(ids[0]).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.symbol.name, "target_fn");
        assert_eq!(found.id, ids[0]);

        // Miss case
        let missing = db.symbol_by_id(99999).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn symbol_referrers_detailed_test() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap();

        let syms = vec![
            sample_symbol("callee", SymbolKind::Function),
            sample_symbol("caller_a", SymbolKind::Function),
            sample_symbol("caller_b", SymbolKind::Function),
        ];
        let ids = db.insert_symbols(file_id, &syms).unwrap();

        // caller_a calls callee at line 10, caller_b imports callee at line 20
        let refs = vec![
            SymbolRef {
                from_file: "test.rs".to_string(),
                from_line: 10,
                target: "callee".to_string(),
                ref_kind: RefKind::Call,
            },
            SymbolRef {
                from_file: "test.rs".to_string(),
                from_line: 20,
                target: "callee".to_string(),
                ref_kind: RefKind::Import,
            },
        ];
        db.insert_refs(&refs, &[ids[1], ids[2]]).unwrap();
        db.resolve_refs().unwrap();

        let detailed = db.symbol_referrers_detailed(ids[0]).unwrap();
        assert_eq!(detailed.len(), 2);

        let names: Vec<&str> = detailed
            .iter()
            .map(|(s, _, _)| s.symbol.name.as_str())
            .collect();
        assert!(names.contains(&"caller_a"));
        assert!(names.contains(&"caller_b"));

        // Check ref_kind and from_line
        let call_ref = detailed
            .iter()
            .find(|(s, _, _)| s.symbol.name == "caller_a")
            .unwrap();
        assert_eq!(call_ref.1, "call");
        assert_eq!(call_ref.2, 10);

        let import_ref = detailed
            .iter()
            .find(|(s, _, _)| s.symbol.name == "caller_b")
            .unwrap();
        assert_eq!(import_ref.1, "import");
        assert_eq!(import_ref.2, 20);
    }

    #[test]
    fn impl_trait_referrers_test() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap();

        let syms = vec![
            Symbol {
                name: "MyTrait".to_string(),
                kind: SymbolKind::Trait,
                file: "test.rs".to_string(),
                line: 1,
                signature: "pub trait MyTrait".to_string(),
                visibility: Visibility::Public,
                doc: None,
                content_hash: blake3::hash(b"trait").into(),
            },
            Symbol {
                name: "MyStruct".to_string(),
                kind: SymbolKind::Impl,
                file: "test.rs".to_string(),
                line: 10,
                signature: "impl MyTrait for MyStruct".to_string(),
                visibility: Visibility::Public,
                doc: None,
                content_hash: blake3::hash(b"impl").into(),
            },
            sample_symbol("some_caller", SymbolKind::Function),
        ];
        let ids = db.insert_symbols(file_id, &syms).unwrap();

        // impl_trait ref from MyStruct impl to MyTrait
        let refs = vec![
            SymbolRef {
                from_file: "test.rs".to_string(),
                from_line: 10,
                target: "MyTrait".to_string(),
                ref_kind: RefKind::ImplTrait,
            },
            // A regular call ref that should NOT be returned
            SymbolRef {
                from_file: "test.rs".to_string(),
                from_line: 15,
                target: "MyTrait".to_string(),
                ref_kind: RefKind::Call,
            },
        ];
        db.insert_refs(&refs, &[ids[1], ids[2]]).unwrap();
        db.resolve_refs().unwrap();

        let impls = db.impl_trait_referrers(ids[0]).unwrap();
        assert_eq!(impls.len(), 1);
        assert_eq!(impls[0].symbol.name, "MyStruct");
    }

    #[test]
    fn hdc_roundtrip() {
        let db = test_db();
        let hash = blake3::hash(b"x").as_bytes().to_vec();
        let file_id = db.upsert_file("test.rs", &hash, 0).unwrap_or_else(|e| {
            panic!("upsert failed: {e}");
        });

        let syms = vec![sample_symbol("my_fn", SymbolKind::Function)];
        let ids = db.insert_symbols(file_id, &syms).unwrap_or_else(|e| {
            panic!("insert_symbols failed: {e}");
        });

        let hdc = HdcVector::from_seed(b"test fingerprint");
        db.set_hdc(ids[0], &hdc).unwrap_or_else(|e| {
            panic!("set_hdc failed: {e}");
        });

        let all = db.all_hdc_symbols().unwrap_or_else(|e| {
            panic!("all_hdc_symbols failed: {e}");
        });
        assert_eq!(all.len(), 1);

        let blob = &all[0].2;
        assert_eq!(blob.len(), 1280);

        let mut arr = [0u8; 1280];
        arr.copy_from_slice(blob);
        let recovered = HdcVector::from_bytes(&arr);
        let sim = hdc.similarity(&recovered);
        assert!((sim - 1.0).abs() < 1e-6, "HDC roundtrip failed: sim={sim}");
    }
}

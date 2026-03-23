//! SQLite semantic store for Grimoire entries and causal DAG.
//!
//! Stores the six entry types (Insight, Heuristic, Warning, `CausalLink`,
//! `StrategyFragment`, `AntiKnowledge`) with full metadata, memetic fields,
//! and a causal edge table for the knowledge DAG.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::entry::{
    DecayClass, EmotionalTag, EntrySource, EntryType, GrimoireEntry, KnowledgePolarity,
    MemeticFields, Provenance,
};
use crate::error::GrimoireError;
use golem_core::cortical::{PadVector, PlutchikEmotion};
use golem_core::id::GolemId;

/// SQLite-backed semantic store for Grimoire entries.
pub struct SemanticStore {
    conn: Connection,
}

impl SemanticStore {
    /// Open or create the SQLite database at `data_dir/grimoire.db`.
    /// Creates all tables and indexes idempotently.
    pub fn open(data_dir: &Path) -> Result<Self, GrimoireError> {
        let db_path = data_dir.join("grimoire.db");
        let conn = Connection::open(&db_path)?;
        let store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    /// Open an in-memory SQLite database (for testing).
    pub fn open_memory() -> Result<Self, GrimoireError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    fn create_schema(&self) -> Result<(), GrimoireError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS grimoire_entries (
                id              TEXT PRIMARY KEY,
                golem_id        TEXT NOT NULL,
                category        TEXT NOT NULL,
                content         TEXT NOT NULL,
                confidence      REAL NOT NULL DEFAULT 0.6,
                quality_score   REAL NOT NULL DEFAULT 0.5,
                decay_class     TEXT NOT NULL,
                valid_from      INTEGER NOT NULL,
                valid_until     INTEGER NOT NULL DEFAULT 0,
                last_accessed_at INTEGER NOT NULL,
                strength        INTEGER NOT NULL DEFAULT 1,
                validated_count INTEGER NOT NULL DEFAULT 0,
                contradicted_count INTEGER NOT NULL DEFAULT 0,
                provenance      TEXT NOT NULL,
                source_golem_id TEXT,
                tags            TEXT NOT NULL DEFAULT '[]',
                emotional_primary TEXT,
                emotional_arousal REAL,
                pad_pleasure    REAL,
                pad_arousal     REAL,
                pad_dominance   REAL,
                is_bloodstain   INTEGER NOT NULL DEFAULT 0,
                polarity        TEXT NOT NULL DEFAULT 'positive',
                meme_fidelity   REAL NOT NULL DEFAULT 1.0,
                meme_fecundity  REAL NOT NULL DEFAULT 0.0,
                meme_fitness    REAL NOT NULL DEFAULT 0.0,
                meme_parasite_score REAL NOT NULL DEFAULT 0.0,
                meme_generation INTEGER NOT NULL DEFAULT 0,
                consecutive_low_confidence INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_category ON grimoire_entries(category);
            CREATE INDEX IF NOT EXISTS idx_confidence ON grimoire_entries(confidence DESC);
            CREATE INDEX IF NOT EXISTS idx_last_accessed ON grimoire_entries(last_accessed_at DESC);
            CREATE INDEX IF NOT EXISTS idx_decay_class ON grimoire_entries(decay_class);

            CREATE TABLE IF NOT EXISTS causal_edges (
                source_id   TEXT NOT NULL REFERENCES grimoire_entries(id),
                target_id   TEXT NOT NULL REFERENCES grimoire_entries(id),
                weight      REAL NOT NULL DEFAULT 1.0,
                evidence    INTEGER NOT NULL DEFAULT 1,
                created_at  INTEGER NOT NULL,
                PRIMARY KEY (source_id, target_id)
            );

            CREATE TABLE IF NOT EXISTS quarantined_entries (
                id           TEXT PRIMARY KEY,
                entry_json   TEXT NOT NULL,
                reason       TEXT NOT NULL,
                quarantined_at INTEGER NOT NULL,
                reviewed     INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS archived_entries (
                id           TEXT PRIMARY KEY,
                entry_json   TEXT NOT NULL,
                archived_at  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS lethe_vote_queue (
                id           TEXT PRIMARY KEY,
                vote_json    TEXT NOT NULL,
                created_at   INTEGER NOT NULL,
                published    INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(())
    }

    /// Insert a new entry into the semantic store.
    pub fn insert(&self, entry: &GrimoireEntry) -> Result<(), GrimoireError> {
        let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
        let emotional_primary = entry
            .emotional_tag
            .as_ref()
            .map(|t| format!("{:?}", t.primary));

        self.conn.execute(
            "INSERT INTO grimoire_entries (
                id, golem_id, category, content, confidence, quality_score,
                decay_class, valid_from, valid_until, last_accessed_at, strength,
                validated_count, contradicted_count, provenance, source_golem_id,
                tags, emotional_primary, emotional_arousal, pad_pleasure, pad_arousal,
                pad_dominance, is_bloodstain, polarity,
                meme_fidelity, meme_fecundity, meme_fitness,
                meme_parasite_score, meme_generation, consecutive_low_confidence
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29
            )",
            params![
                entry.id.to_string(),
                entry.golem_id.as_uuid().to_string(),
                entry.category.as_str(),
                entry.content,
                entry.confidence,
                entry.quality_score,
                entry.decay_class.as_str(),
                entry.valid_from,
                entry.valid_until,
                entry.last_accessed_at,
                entry.strength,
                entry.validated_count,
                entry.contradicted_count,
                entry.provenance.as_str(),
                entry.source.golem_id,
                tags_json,
                emotional_primary,
                entry.emotional_tag.as_ref().map(|t| t.arousal),
                entry.emotional_tag.as_ref().map(|t| t.pad.pleasure),
                entry.emotional_tag.as_ref().map(|t| t.pad.arousal),
                entry.emotional_tag.as_ref().map(|t| t.pad.dominance),
                i32::from(entry.is_bloodstain),
                entry.polarity.as_str(),
                entry.memetic.fidelity,
                entry.memetic.fecundity,
                entry.memetic.fitness,
                entry.memetic.parasite_score,
                entry.memetic.generation,
                entry.memetic.consecutive_low_confidence,
            ],
        )?;
        Ok(())
    }

    /// Get a single entry by ID.
    pub fn get_by_id(&self, id: &Uuid) -> Result<Option<GrimoireEntry>, GrimoireError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, golem_id, category, content, confidence, quality_score,
                    decay_class, valid_from, valid_until, last_accessed_at, strength,
                    validated_count, contradicted_count, provenance, source_golem_id,
                    tags, emotional_primary, emotional_arousal,
                    pad_pleasure, pad_arousal, pad_dominance,
                    is_bloodstain, polarity,
                    meme_fidelity, meme_fecundity, meme_fitness,
                    meme_parasite_score, meme_generation, consecutive_low_confidence
             FROM grimoire_entries WHERE id = ?1",
        )?;

        let result = stmt
            .query_row(params![id.to_string()], |row| Ok(row_to_entry(row)))
            .optional()?;

        match result {
            Some(Ok(entry)) => Ok(Some(entry)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Query all active entries (for Curator operations).
    pub fn query_all_active(&self) -> Result<Vec<GrimoireEntry>, GrimoireError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, golem_id, category, content, confidence, quality_score,
                    decay_class, valid_from, valid_until, last_accessed_at, strength,
                    validated_count, contradicted_count, provenance, source_golem_id,
                    tags, emotional_primary, emotional_arousal,
                    pad_pleasure, pad_arousal, pad_dominance,
                    is_bloodstain, polarity,
                    meme_fidelity, meme_fecundity, meme_fitness,
                    meme_parasite_score, meme_generation, consecutive_low_confidence
             FROM grimoire_entries",
        )?;

        let rows = stmt.query_map([], |row| Ok(row_to_entry(row)))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row??);
        }
        Ok(entries)
    }

    /// Update confidence for an entry.
    pub fn update_confidence(&self, id: &Uuid, confidence: f64) -> Result<(), GrimoireError> {
        self.conn.execute(
            "UPDATE grimoire_entries SET confidence = ?1 WHERE id = ?2",
            params![confidence, id.to_string()],
        )?;
        Ok(())
    }

    /// Update `last_accessed_at` and increment strength.
    pub fn mark_accessed(&self, id: &Uuid, timestamp: i64) -> Result<(), GrimoireError> {
        self.conn.execute(
            "UPDATE grimoire_entries SET last_accessed_at = ?1, strength = strength + 1 WHERE id = ?2",
            params![timestamp, id.to_string()],
        )?;
        Ok(())
    }

    /// Update memetic fields for an entry.
    pub fn update_memetic(&self, id: &Uuid, fields: &MemeticFields) -> Result<(), GrimoireError> {
        self.conn.execute(
            "UPDATE grimoire_entries SET
                meme_fidelity = ?1, meme_fecundity = ?2, meme_fitness = ?3,
                meme_parasite_score = ?4, meme_generation = ?5,
                consecutive_low_confidence = ?6
             WHERE id = ?7",
            params![
                fields.fidelity,
                fields.fecundity,
                fields.fitness,
                fields.parasite_score,
                fields.generation,
                fields.consecutive_low_confidence,
                id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Archive an entry: move to `archived_entries` and delete from active.
    pub fn archive_entry(&self, id: &Uuid, tick: i64) -> Result<(), GrimoireError> {
        // Read the entry first.
        let entry = self.get_by_id(id)?;
        let Some(entry) = entry else {
            return Err(GrimoireError::NotFound(format!("entry {id}")));
        };
        let json = serde_json::to_string(&entry)
            .map_err(|e| GrimoireError::Serialization(e.to_string()))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO archived_entries (id, entry_json, archived_at) VALUES (?1, ?2, ?3)",
            params![id.to_string(), json, tick],
        )?;
        self.conn.execute(
            "DELETE FROM grimoire_entries WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Quarantine an entry.
    pub fn quarantine_entry(
        &self,
        id: &Uuid,
        reason: &str,
        tick: i64,
    ) -> Result<(), GrimoireError> {
        let entry = self.get_by_id(id)?;
        let Some(entry) = entry else {
            return Err(GrimoireError::NotFound(format!("entry {id}")));
        };
        let json = serde_json::to_string(&entry)
            .map_err(|e| GrimoireError::Serialization(e.to_string()))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO quarantined_entries (id, entry_json, reason, quarantined_at) VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), json, reason, tick],
        )?;
        self.conn.execute(
            "DELETE FROM grimoire_entries WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Delete an entry (used for merging duplicates).
    pub fn delete_entry(&self, id: &Uuid) -> Result<(), GrimoireError> {
        self.conn.execute(
            "DELETE FROM grimoire_entries WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Insert a causal edge.
    pub fn insert_causal_edge(
        &self,
        source_id: &Uuid,
        target_id: &Uuid,
        weight: f64,
        tick: i64,
    ) -> Result<(), GrimoireError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO causal_edges (source_id, target_id, weight, evidence, created_at)
             VALUES (?1, ?2, ?3, 1, ?4)",
            params![
                source_id.to_string(),
                target_id.to_string(),
                weight,
                tick,
            ],
        )?;
        Ok(())
    }

    /// Get all causal edges.
    pub fn all_causal_edges(&self) -> Result<Vec<(Uuid, Uuid, f64)>, GrimoireError> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, target_id, weight FROM causal_edges")?;
        let rows = stmt.query_map([], |row| {
            let src: String = row.get(0)?;
            let tgt: String = row.get(1)?;
            let w: f64 = row.get(2)?;
            Ok((src, tgt, w))
        })?;

        let mut edges = Vec::new();
        for row in rows {
            let (src, tgt, w) = row?;
            let src_uuid =
                Uuid::parse_str(&src).map_err(|e| GrimoireError::InvalidState(e.to_string()))?;
            let tgt_uuid =
                Uuid::parse_str(&tgt).map_err(|e| GrimoireError::InvalidState(e.to_string()))?;
            edges.push((src_uuid, tgt_uuid, w));
        }
        Ok(edges)
    }

    /// Count active entries.
    pub fn count_active(&self) -> Result<usize, GrimoireError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM grimoire_entries", [], |row| {
                    row.get(0)
                })?;
        Ok(count as usize)
    }

    /// Check if an entry exists in `archived_entries`.
    pub fn is_archived(&self, id: &Uuid) -> Result<bool, GrimoireError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM archived_entries WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

/// Parse a row from `grimoire_entries` into a `GrimoireEntry`.
fn row_to_entry(row: &rusqlite::Row<'_>) -> Result<GrimoireEntry, GrimoireError> {
    let id_str: String = row
        .get(0)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let golem_id_str: String = row
        .get(1)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let category_str: String = row
        .get(2)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let content: String = row
        .get(3)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let confidence: f64 = row
        .get(4)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let quality_score: f64 = row
        .get(5)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let decay_class_str: String = row
        .get(6)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let valid_from: i64 = row
        .get(7)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let valid_until: i64 = row
        .get(8)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let last_accessed_at: i64 = row
        .get(9)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let strength: u32 = row
        .get(10)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let validated_count: u32 = row
        .get(11)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let contradicted_count: u32 = row
        .get(12)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let provenance_str: String = row
        .get(13)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let source_golem_id: Option<String> = row
        .get(14)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let tags_json: String = row
        .get(15)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let emotional_primary: Option<String> = row
        .get(16)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let emotional_arousal: Option<f64> = row
        .get(17)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let pad_pleasure: Option<f64> = row
        .get(18)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let pad_arousal: Option<f64> = row
        .get(19)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let pad_dominance: Option<f64> = row
        .get(20)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let is_bloodstain: i32 = row
        .get(21)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let polarity_str: String = row
        .get(22)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let meme_fidelity: f64 = row
        .get(23)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let meme_fecundity: f64 = row
        .get(24)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let meme_fitness: f64 = row
        .get(25)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let meme_parasite_score: f64 = row
        .get(26)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let meme_generation: u32 = row
        .get(27)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;
    let consecutive_low: u32 = row
        .get(28)
        .map_err(|e| GrimoireError::Sqlite(e.to_string()))?;

    let id = Uuid::parse_str(&id_str).map_err(|e| GrimoireError::InvalidState(e.to_string()))?;
    let golem_uuid =
        Uuid::parse_str(&golem_id_str).map_err(|e| GrimoireError::InvalidState(e.to_string()))?;
    let category = EntryType::from_str_repr(&category_str)
        .ok_or_else(|| GrimoireError::InvalidState(format!("unknown category: {category_str}")))?;
    let decay_class = DecayClass::from_str_repr(&decay_class_str).ok_or_else(|| {
        GrimoireError::InvalidState(format!("unknown decay class: {decay_class_str}"))
    })?;
    let provenance = Provenance::from_str_repr(&provenance_str).ok_or_else(|| {
        GrimoireError::InvalidState(format!("unknown provenance: {provenance_str}"))
    })?;
    let polarity = KnowledgePolarity::from_str_repr(&polarity_str)
        .ok_or_else(|| GrimoireError::InvalidState(format!("unknown polarity: {polarity_str}")))?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

    let emotional_tag = if let (Some(_primary_str), Some(arousal), Some(p), Some(a), Some(d)) = (
        emotional_primary,
        emotional_arousal,
        pad_pleasure,
        pad_arousal,
        pad_dominance,
    ) {
        Some(EmotionalTag {
            primary: PlutchikEmotion::from_pad(&PadVector {
                pleasure: p,
                arousal: a,
                dominance: d,
            }),
            pad: PadVector {
                pleasure: p,
                arousal: a,
                dominance: d,
            },
            arousal,
        })
    } else {
        None
    };

    Ok(GrimoireEntry {
        id,
        golem_id: GolemId::from_uuid(golem_uuid),
        category,
        content,
        embedding: Vec::new(), // Embeddings are in the episodic store, not SQLite.
        confidence,
        quality_score,
        decay_class,
        valid_from,
        valid_until,
        parent_episode_ids: Vec::new(),
        causal_parents: Vec::new(),
        tags,
        provenance,
        source: EntrySource {
            golem_id: source_golem_id.unwrap_or_default(),
            generation_number: None,
            owner_address: None,
        },
        emotional_tag,
        last_accessed_at,
        strength,
        validated_count,
        contradicted_count,
        memetic: MemeticFields {
            fidelity: meme_fidelity,
            fecundity: meme_fecundity,
            fitness: meme_fitness,
            parasite_score: meme_parasite_score,
            generation: meme_generation,
            consecutive_low_confidence: consecutive_low,
        },
        is_bloodstain: is_bloodstain != 0,
        polarity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::GrimoireEntry;

    #[test]
    fn test_sqlite_schema_creates_idempotent() {
        let store = SemanticStore::open_memory().expect("open");
        // Creating schema again should not fail.
        store.create_schema().expect("idempotent schema");
    }

    #[test]
    fn test_insert_and_get_by_id() {
        let store = SemanticStore::open_memory().expect("open");
        let entry = GrimoireEntry::test_heuristic("gas below 10 gwei");
        store.insert(&entry).expect("insert");

        let fetched = store.get_by_id(&entry.id).expect("get").expect("found");
        assert_eq!(fetched.id, entry.id);
        assert_eq!(fetched.content, entry.content);
        assert_eq!(fetched.category, EntryType::Heuristic);
    }

    #[test]
    fn test_archive_entry() {
        let store = SemanticStore::open_memory().expect("open");
        let entry = GrimoireEntry::test_heuristic("to archive");
        store.insert(&entry).expect("insert");

        store.archive_entry(&entry.id, 100).expect("archive");

        // Should no longer be in active entries.
        assert!(store.get_by_id(&entry.id).expect("get").is_none());
        // Should be in archived_entries.
        assert!(store.is_archived(&entry.id).expect("check"));
    }

    #[test]
    fn test_mark_accessed_increments_strength() {
        let store = SemanticStore::open_memory().expect("open");
        let entry = GrimoireEntry::test_heuristic("strength test");
        store.insert(&entry).expect("insert");

        store.mark_accessed(&entry.id, 1000).expect("mark");
        let fetched = store.get_by_id(&entry.id).expect("get").expect("found");
        assert_eq!(fetched.strength, entry.strength + 1);
        assert_eq!(fetched.last_accessed_at, 1000);
    }
}

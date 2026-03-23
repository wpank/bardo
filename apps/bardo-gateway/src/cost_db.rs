//! Persistent cost tracking, semantic cache, and tool usage via SQLite.
//!
//! Subscribes to the stats broadcast channel and writes every request's
//! cost data to `.mori/costs.db`. On startup, restores aggregate stats
//! so counters survive gateway restarts. Also persists the semantic cache
//! (SimHash fingerprints) and per-session tool usage patterns.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use rusqlite::Connection;
use tokio::sync::{Mutex, broadcast};

use crate::state::{GatewayStats, StatsEvent};

/// Batched SQLite cost writer with restore-on-startup.
pub struct CostDb {
    conn: Arc<Mutex<Connection>>,
}

/// Aggregated stats restored from the database on startup.
pub struct RestoredStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_micro_usd: u64,
    pub total_naive_cost_micro_usd: u64,
    pub model_counts: Vec<(String, u64)>,
}

/// A persisted semantic cache entry.
pub struct PersistedCacheEntry {
    pub fingerprint: u64,
    pub response: Bytes,
    pub cost_usd: f64,
    pub model: String,
}

/// A persisted tool usage record.
pub struct PersistedToolUsage {
    pub session_id: String,
    pub tools_used: HashSet<String>,
    pub request_count: usize,
}

impl CostDb {
    /// Open (or create) the cost database at the given path.
    pub fn open(path: &PathBuf) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;

             CREATE TABLE IF NOT EXISTS requests (
                 id          INTEGER PRIMARY KEY AUTOINCREMENT,
                 timestamp   TEXT    NOT NULL,
                 model       TEXT    NOT NULL,
                 provider    TEXT    NOT NULL,
                 session_id  TEXT,
                 input_tokens      INTEGER NOT NULL DEFAULT 0,
                 output_tokens     INTEGER NOT NULL DEFAULT 0,
                 cache_read_tokens INTEGER NOT NULL DEFAULT 0,
                 cache_create_tokens INTEGER NOT NULL DEFAULT 0,
                 openai_cached_tokens INTEGER NOT NULL DEFAULT 0,
                 reasoning_tokens  INTEGER NOT NULL DEFAULT 0,
                 thinking_tokens   INTEGER NOT NULL DEFAULT 0,
                 cost_usd          REAL NOT NULL DEFAULT 0,
                 naive_cost_usd    REAL NOT NULL DEFAULT 0,
                 savings_usd       REAL NOT NULL DEFAULT 0,
                 is_batch          INTEGER NOT NULL DEFAULT 0,
                 is_cache_hit      INTEGER NOT NULL DEFAULT 0,
                 elapsed_ms        INTEGER NOT NULL DEFAULT 0,
                 streaming         INTEGER NOT NULL DEFAULT 0
             );

             CREATE INDEX IF NOT EXISTS idx_requests_timestamp ON requests(timestamp);
             CREATE INDEX IF NOT EXISTS idx_requests_model ON requests(model);
             CREATE INDEX IF NOT EXISTS idx_requests_session ON requests(session_id);

             CREATE TABLE IF NOT EXISTS semantic_cache (
                 fingerprint INTEGER PRIMARY KEY,
                 response    BLOB NOT NULL,
                 cost_usd    REAL NOT NULL,
                 model       TEXT NOT NULL,
                 created_at  TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS tool_usage (
                 session_id    TEXT NOT NULL,
                 tool_name     TEXT NOT NULL,
                 request_count INTEGER NOT NULL DEFAULT 0,
                 PRIMARY KEY (session_id, tool_name)
             );
             CREATE TABLE IF NOT EXISTS tool_session_counts (
                 session_id    TEXT PRIMARY KEY,
                 request_count INTEGER NOT NULL DEFAULT 0
             );

             CREATE TABLE IF NOT EXISTS schema_version (
                 version INTEGER NOT NULL
             );",
        )?;

        // Run migrations.
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Run schema migrations. Each migration runs once, tracked by version number.
    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let conn = tokio::task::block_in_place(|| self.conn.blocking_lock());

        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if current_version < 2 {
            // Migration 2: Fix pricing — opus 4.6 was $15/$75 (legacy 4.1 rate),
            // should be $5/$25. Haiku 4.5 was $0.80/$4, should be $1/$5.
            // Recalculate cost/naive/savings for all affected rows using token counts.
            tracing::info!("running migration v2: fix opus/haiku pricing");

            conn.execute_batch("BEGIN TRANSACTION")?;

            // Opus 4.6: old rates $15/$75/$1.50 cache, new rates $5/$25/$0.50 cache
            // Scale factor: new/old = 5/15 = 1/3 for input-related, 25/75 = 1/3 for output
            // Since ALL token types were priced at 3x, we can just divide by 3.
            let opus_updated = conn.execute(
                "UPDATE requests SET
                    cost_usd = cost_usd / 3.0,
                    naive_cost_usd = naive_cost_usd / 3.0,
                    savings_usd = savings_usd / 3.0
                 WHERE model LIKE '%opus-4-6%' OR model LIKE '%opus-4.6%'",
                [],
            )?;

            // Haiku 4.5: old rates $0.80/$4, new rates $1/$5
            // Scale factor: new/old = 1/0.8 = 1.25 for input, 5/4 = 1.25 for output
            let haiku_updated = conn.execute(
                "UPDATE requests SET
                    cost_usd = cost_usd * 1.25,
                    naive_cost_usd = naive_cost_usd * 1.25,
                    savings_usd = savings_usd * 1.25
                 WHERE model LIKE '%haiku-4-5%' OR model LIKE '%haiku-4.5%'",
                [],
            )?;

            conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])?;

            conn.execute_batch("COMMIT")?;

            tracing::info!(
                opus_rows = opus_updated,
                haiku_rows = haiku_updated,
                "migration v2 complete: pricing corrected"
            );
        }

        Ok(())
    }

    // ── Stats restore ────────────────────────────────────────────────

    /// Restore aggregate stats from the database. Called once at startup.
    pub fn restore_stats(&self) -> Result<RestoredStats, rusqlite::Error> {
        let conn = tokio::task::block_in_place(|| self.conn.blocking_lock());

        let mut stmt = conn.prepare(
            "SELECT
                COUNT(*) as total_requests,
                SUM(CASE WHEN is_cache_hit = 1 THEN 1 ELSE 0 END) as cache_hits,
                SUM(CASE WHEN is_cache_hit = 0 THEN 1 ELSE 0 END) as cache_misses,
                COALESCE(SUM(input_tokens), 0) as total_input,
                COALESCE(SUM(output_tokens), 0) as total_output,
                COALESCE(SUM(cost_usd), 0.0) as total_cost,
                COALESCE(SUM(naive_cost_usd), 0.0) as total_naive
             FROM requests",
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(RestoredStats {
                total_requests: row.get::<_, i64>(0)? as u64,
                cache_hits: row.get::<_, i64>(1)? as u64,
                cache_misses: row.get::<_, i64>(2)? as u64,
                total_input_tokens: row.get::<_, i64>(3)? as u64,
                total_output_tokens: row.get::<_, i64>(4)? as u64,
                total_cost_micro_usd: (row.get::<_, f64>(5)? * 1_000_000.0) as u64,
                total_naive_cost_micro_usd: (row.get::<_, f64>(6)? * 1_000_000.0) as u64,
                model_counts: Vec::new(), // filled below
            })
        })?;

        // Per-model counts.
        let mut model_stmt = conn.prepare("SELECT model, COUNT(*) FROM requests GROUP BY model")?;
        let model_counts: Vec<(String, u64)> = model_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(RestoredStats {
            model_counts,
            ..stats
        })
    }

    /// Apply restored stats to a GatewayStats instance.
    pub fn apply_restored(stats: &GatewayStats, restored: &RestoredStats) {
        use std::sync::atomic::Ordering::Relaxed;
        stats.total_requests.store(restored.total_requests, Relaxed);
        stats.cache_hits.store(restored.cache_hits, Relaxed);
        stats.cache_misses.store(restored.cache_misses, Relaxed);
        stats
            .total_input_tokens
            .store(restored.total_input_tokens, Relaxed);
        stats
            .total_output_tokens
            .store(restored.total_output_tokens, Relaxed);
        stats
            .total_cost_micro_usd
            .store(restored.total_cost_micro_usd, Relaxed);
        stats
            .total_naive_cost_micro_usd
            .store(restored.total_naive_cost_micro_usd, Relaxed);
        for (model, count) in &restored.model_counts {
            stats.model_counts.insert(model.clone(), *count);
        }
    }

    // ── Semantic cache persistence ───────────────────────────────────

    /// Load all semantic cache entries from the database.
    pub fn load_semantic_cache(&self) -> Result<Vec<PersistedCacheEntry>, rusqlite::Error> {
        let conn = tokio::task::block_in_place(|| self.conn.blocking_lock());
        let mut stmt =
            conn.prepare("SELECT fingerprint, response, cost_usd, model FROM semantic_cache")?;
        let entries = stmt
            .query_map([], |row| {
                Ok(PersistedCacheEntry {
                    fingerprint: row.get::<_, i64>(0)? as u64,
                    response: Bytes::from(row.get::<_, Vec<u8>>(1)?),
                    cost_usd: row.get(2)?,
                    model: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }

    /// Persist a semantic cache entry.
    pub async fn save_semantic_entry(
        &self,
        fingerprint: u64,
        response: &[u8],
        cost_usd: f64,
        model: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO semantic_cache (fingerprint, response, cost_usd, model, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            rusqlite::params![fingerprint as i64, response, cost_usd, model],
        )?;
        Ok(())
    }

    // ── Tool usage persistence ───────────────────────────────────────

    /// Load tool usage for all sessions.
    pub fn load_tool_usage(&self) -> Result<Vec<PersistedToolUsage>, rusqlite::Error> {
        let conn = tokio::task::block_in_place(|| self.conn.blocking_lock());

        let mut count_stmt =
            conn.prepare("SELECT session_id, request_count FROM tool_session_counts")?;
        let counts: std::collections::HashMap<String, usize> = count_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut usage_stmt = conn.prepare("SELECT session_id, tool_name FROM tool_usage")?;
        let mut sessions: std::collections::HashMap<String, HashSet<String>> =
            std::collections::HashMap::new();
        usage_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .for_each(|(sid, tool)| {
                sessions.entry(sid).or_default().insert(tool);
            });

        let result = sessions
            .into_iter()
            .map(|(sid, tools)| PersistedToolUsage {
                request_count: counts.get(&sid).copied().unwrap_or(0),
                session_id: sid,
                tools_used: tools,
            })
            .collect();

        Ok(result)
    }

    /// Persist tool usage for a session (called periodically or on shutdown).
    pub async fn save_tool_usage(
        &self,
        session_id: &str,
        tools: &HashSet<String>,
        request_count: usize,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO tool_session_counts (session_id, request_count) VALUES (?1, ?2)",
            rusqlite::params![session_id, request_count as i64],
        )?;
        for tool in tools {
            conn.execute(
                "INSERT OR IGNORE INTO tool_usage (session_id, tool_name, request_count) VALUES (?1, ?2, 0)",
                rusqlite::params![session_id, tool],
            )?;
        }
        Ok(())
    }

    // ── Stats writer ─────────────────────────────────────────────────

    /// Start a background task that subscribes to the broadcast channel
    /// and batches inserts every 100ms.
    pub fn start_writer(self: &Arc<Self>, mut rx: broadcast::Receiver<StatsEvent>) {
        let db = Arc::clone(self);
        tokio::spawn(async move {
            let mut batch: Vec<StatsEvent> = Vec::with_capacity(64);
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            let events: Vec<StatsEvent> = batch.drain(..).collect();
                            if let Err(e) = db.insert_batch(&events).await {
                                tracing::warn!(error = %e, count = events.len(), "cost_db write failed");
                            }
                        }
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(event) => batch.push(event),
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                tracing::debug!(skipped = n, "cost_db writer lagged");
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }

            if !batch.is_empty() {
                let _ = db.insert_batch(&batch).await;
            }
        });
    }

    async fn insert_batch(&self, events: &[StatsEvent]) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO requests (
                timestamp, model, provider, session_id,
                input_tokens, output_tokens, cache_read_tokens, cache_create_tokens,
                openai_cached_tokens, reasoning_tokens, thinking_tokens,
                cost_usd, naive_cost_usd, savings_usd,
                is_batch, is_cache_hit, elapsed_ms, streaming
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        )?;

        for ev in events {
            let ts = chrono::DateTime::from_timestamp_millis(ev.timestamp_ms as i64)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            stmt.execute(rusqlite::params![
                ts,
                ev.model,
                ev.provider,
                ev.session_id,
                ev.input_tokens,
                ev.output_tokens,
                ev.cache_read_tokens,
                ev.cache_create_tokens,
                ev.openai_cached_tokens,
                ev.reasoning_tokens,
                ev.thinking_tokens,
                ev.cost_usd,
                ev.naive_cost_usd,
                ev.savings_usd,
                ev.is_batch as i32,
                ev.cache_hit as i32,
                ev.elapsed_ms,
                ev.streaming as i32,
            ])?;
        }

        Ok(())
    }
}

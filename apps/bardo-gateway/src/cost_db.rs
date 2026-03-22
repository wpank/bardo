//! Persistent cost tracking via SQLite.
//!
//! Subscribes to the stats broadcast channel and writes every request's
//! cost data to `.mori/costs.db`. Writes are batched (100ms) to avoid
//! write amplification. Enables historical cost queries via `mori stats`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rusqlite::Connection;
use tokio::sync::{broadcast, Mutex};

use crate::state::StatsEvent;

/// Batched SQLite cost writer.
pub struct CostDb {
    conn: Arc<Mutex<Connection>>,
}

impl CostDb {
    /// Open (or create) the cost database at the given path.
    pub fn open(path: &PathBuf) -> Result<Self, rusqlite::Error> {
        // Ensure parent directory exists.
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
             CREATE INDEX IF NOT EXISTS idx_requests_session ON requests(session_id);",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

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

            // Flush remaining events on shutdown.
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

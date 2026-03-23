//! SQLite persistence for MPP payment records and session state.

use std::path::PathBuf;
use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

/// MPP database for payment and session persistence.
pub struct MppDb {
    conn: Arc<Mutex<Connection>>,
}

impl MppDb {
    /// Open (or create) the MPP tables in the given database.
    pub fn open(path: &PathBuf) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;

             CREATE TABLE IF NOT EXISTS mpp_payments (
                 id                  TEXT PRIMARY KEY,
                 timestamp           TEXT NOT NULL,
                 payer               TEXT NOT NULL,
                 intent              TEXT NOT NULL,
                 session_id          TEXT,
                 amount_quoted       INTEGER NOT NULL,
                 amount_charged      INTEGER NOT NULL,
                 provider_cost_micro INTEGER NOT NULL,
                 spread_micro        INTEGER NOT NULL,
                 spread_pct          REAL NOT NULL,
                 model               TEXT NOT NULL,
                 tx_hash             TEXT,
                 settled             INTEGER NOT NULL DEFAULT 0
             );

             CREATE TABLE IF NOT EXISTS mpp_sessions (
                 session_id   TEXT PRIMARY KEY,
                 payer        TEXT NOT NULL,
                 funded_amount INTEGER NOT NULL,
                 drawn_amount  INTEGER NOT NULL,
                 status       TEXT NOT NULL,
                 created_at   TEXT NOT NULL,
                 expires_at   TEXT NOT NULL,
                 settled_at   TEXT
             );

             CREATE INDEX IF NOT EXISTS idx_mpp_payments_payer
                 ON mpp_payments(payer);
             CREATE INDEX IF NOT EXISTS idx_mpp_payments_session
                 ON mpp_payments(session_id);
             CREATE INDEX IF NOT EXISTS idx_mpp_sessions_payer
                 ON mpp_sessions(payer);",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Record a payment.
    pub async fn record_payment(
        &self,
        receipt_id: &str,
        payer: &str,
        intent: &str,
        session_id: Option<&str>,
        amount_quoted: u64,
        amount_charged: u64,
        provider_cost_micro: u64,
        spread_micro: u64,
        spread_pct: f64,
        model: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO mpp_payments (id, timestamp, payer, intent, session_id, amount_quoted, amount_charged, provider_cost_micro, spread_micro, spread_pct, model)
             VALUES (?1, datetime('now'), ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                receipt_id,
                payer,
                intent,
                session_id,
                amount_quoted,
                amount_charged,
                provider_cost_micro,
                spread_micro,
                spread_pct,
                model,
            ],
        )?;
        Ok(())
    }

    /// Record a session open.
    pub async fn record_session_open(
        &self,
        session_id: &str,
        payer: &str,
        funded_amount: u64,
        created_at: &str,
        expires_at: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO mpp_sessions (session_id, payer, funded_amount, drawn_amount, status, created_at, expires_at)
             VALUES (?1, ?2, ?3, 0, 'active', ?4, ?5)",
            rusqlite::params![session_id, payer, funded_amount, created_at, expires_at],
        )?;
        Ok(())
    }

    /// Update session drawn amount.
    pub async fn update_session_draw(
        &self,
        session_id: &str,
        drawn_amount: u64,
        status: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE mpp_sessions SET drawn_amount = ?1, status = ?2 WHERE session_id = ?3",
            rusqlite::params![drawn_amount, status, session_id],
        )?;
        Ok(())
    }

    /// Close a session.
    pub async fn close_session(&self, session_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE mpp_sessions SET status = 'settled', settled_at = datetime('now') WHERE session_id = ?1",
            rusqlite::params![session_id],
        )?;
        Ok(())
    }
}

//! MPP session management for the Session payment intent.
//!
//! Sessions let a client pre-fund a balance and draw against it per-request,
//! avoiding the overhead of a new ERC-3009 signature on every call.

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::error::MppError;

/// State of an MPP payment session.
#[derive(Debug, Clone)]
pub struct MppSession {
    pub session_id: String,
    pub payer: Address,
    /// Total funded (initial + top-ups) in token base units.
    pub funded_amount: U256,
    /// Total drawn so far in token base units.
    pub drawn_amount: U256,
    pub created_at: DateTime<Utc>,
    pub last_draw_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: SessionStatus,
    /// Per-request draw records for audit.
    pub draws: Vec<DrawRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Exhausted,
    Expired,
    Settled,
}

#[derive(Debug, Clone)]
pub struct DrawRecord {
    pub request_id: String,
    pub amount: U256,
    pub model: String,
    pub timestamp: DateTime<Utc>,
}

/// Final settlement summary when a session closes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettlement {
    pub session_id: String,
    pub total_funded: U256,
    pub total_drawn: U256,
    pub refund_amount: U256,
    pub draw_count: u64,
}

impl MppSession {
    /// Remaining balance.
    pub fn remaining(&self) -> U256 {
        self.funded_amount.saturating_sub(self.drawn_amount)
    }

    /// Draw an amount from the session balance.
    pub fn draw(&mut self, amount: U256, request_id: &str, model: &str) -> Result<U256, MppError> {
        if self.status != SessionStatus::Active {
            return Err(MppError::SessionNotActive);
        }
        if amount > self.remaining() {
            return Err(MppError::InsufficientBalance);
        }
        self.drawn_amount += amount;
        self.last_draw_at = Utc::now();
        self.draws.push(DrawRecord {
            request_id: request_id.to_string(),
            amount,
            model: model.to_string(),
            timestamp: Utc::now(),
        });
        if self.remaining().is_zero() {
            self.status = SessionStatus::Exhausted;
        }
        Ok(self.remaining())
    }

    /// Add funds to an active session.
    pub fn top_up(&mut self, amount: U256) -> Result<U256, MppError> {
        if self.status != SessionStatus::Active {
            return Err(MppError::SessionNotActive);
        }
        self.funded_amount += amount;
        Ok(self.remaining())
    }
}

/// Concurrent in-memory session store.
///
/// Thread-safe via `DashMap`. For production persistence, pair with [`crate::db::MppDb`].
pub struct SessionStore {
    sessions: DashMap<String, MppSession>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Create a new session.
    pub fn open(
        &self,
        session_id: String,
        payer: Address,
        funded: U256,
        ttl_secs: u64,
    ) -> MppSession {
        let now = Utc::now();
        let session = MppSession {
            session_id: session_id.clone(),
            payer,
            funded_amount: funded,
            drawn_amount: U256::ZERO,
            created_at: now,
            last_draw_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            status: SessionStatus::Active,
            draws: Vec::new(),
        };
        self.sessions.insert(session_id, session.clone());
        session
    }

    /// Get a session by ID.
    pub fn get(&self, session_id: &str) -> Option<MppSession> {
        self.sessions.get(session_id).map(|r| r.clone())
    }

    /// Draw from a session.
    pub fn draw(
        &self,
        session_id: &str,
        amount: U256,
        request_id: &str,
        model: &str,
    ) -> Result<U256, MppError> {
        let mut entry = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| MppError::SessionNotFound(session_id.to_string()))?;
        entry.draw(amount, request_id, model)
    }

    /// Top up a session.
    pub fn top_up(&self, session_id: &str, amount: U256) -> Result<U256, MppError> {
        let mut entry = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| MppError::SessionNotFound(session_id.to_string()))?;
        entry.top_up(amount)
    }

    /// Close a session and return settlement summary.
    pub fn close(&self, session_id: &str) -> Option<SessionSettlement> {
        let (_, mut session) = self.sessions.remove(session_id)?;
        session.status = SessionStatus::Settled;
        let refund = session.remaining();
        let draw_count = session.draws.len() as u64;
        Some(SessionSettlement {
            session_id: session.session_id,
            total_funded: session.funded_amount,
            total_drawn: session.drawn_amount,
            refund_amount: refund,
            draw_count,
        })
    }

    /// Expire stale sessions. Returns IDs of expired sessions.
    pub fn expire_stale(&self) -> Vec<String> {
        let now = Utc::now();
        let mut expired = Vec::new();
        for mut entry in self.sessions.iter_mut() {
            if entry.expires_at < now && entry.status == SessionStatus::Active {
                entry.status = SessionStatus::Expired;
                expired.push(entry.session_id.clone());
            }
        }
        expired
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_lifecycle() {
        let store = SessionStore::new();
        let payer = Address::ZERO;
        let funded = U256::from(1_000_000u64); // $1.00

        let session = store.open("s1".into(), payer, funded, 3600);
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.remaining(), funded);

        let remaining = store
            .draw("s1", U256::from(500_000u64), "req1", "claude-sonnet-4")
            .unwrap();
        assert_eq!(remaining, U256::from(500_000u64));

        // Draw remaining -> exhausted.
        let remaining = store
            .draw("s1", U256::from(500_000u64), "req2", "claude-sonnet-4")
            .unwrap();
        assert_eq!(remaining, U256::ZERO);

        // Next draw fails.
        let err = store.draw("s1", U256::from(1u64), "req3", "claude-sonnet-4");
        assert!(err.is_err());
    }

    #[test]
    fn session_topup() {
        let store = SessionStore::new();
        let payer = Address::ZERO;
        let funded = U256::from(100_000u64); // $0.10

        store.open("s2".into(), payer, funded, 3600);
        store
            .draw("s2", U256::from(80_000u64), "req1", "claude-haiku-4-5")
            .unwrap();

        let remaining = store.top_up("s2", U256::from(200_000u64)).unwrap();
        assert_eq!(remaining, U256::from(220_000u64));

        let settlement = store.close("s2").unwrap();
        assert_eq!(settlement.total_funded, U256::from(300_000u64));
        assert_eq!(settlement.total_drawn, U256::from(80_000u64));
        assert_eq!(settlement.refund_amount, U256::from(220_000u64));
        assert_eq!(settlement.draw_count, 1);
    }
}

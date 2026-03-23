//! SQLite persistence layer for Mori-as-a-Service.
//!
//! Stores the full lifecycle: drafts, proposals, runs, adjustments, deliverables,
//! receipts, and account links. All methods take `&self` with interior mutability
//! via `parking_lot::Mutex`.

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::types::{
    Adjustment, AdjustmentType, CostEstimate, Deliverable, DeliveryType, Draft, DraftMessage,
    FundingMode, MessageRole, Milestone, MilestoneStatus, Proposal, ProposalStatus, Receipt, Run,
    RunStatus,
};

/// Wraps a SQLite connection behind a parking_lot mutex for thread-safe access.
pub struct ServiceDb {
    conn: Mutex<Connection>,
}

// ---------------------------------------------------------------------------
// ID generation helpers
// ---------------------------------------------------------------------------

/// Generate a draft ID like "d-1a2b3c4d".
pub fn generate_draft_id() -> String {
    let u = Uuid::new_v4();
    format!("d-{}", &u.simple().to_string()[..8])
}

/// Generate a proposal ID like "p-1a2b3c4d".
pub fn generate_proposal_id() -> String {
    let u = Uuid::new_v4();
    format!("p-{}", &u.simple().to_string()[..8])
}

/// Generate a run ID like "r-1a2b3c4d".
pub fn generate_run_id() -> String {
    let u = Uuid::new_v4();
    format!("r-{}", &u.simple().to_string()[..8])
}

/// Generate a receipt ID like "rc-1a2b3c4d".
pub fn generate_receipt_id() -> String {
    let u = Uuid::new_v4();
    format!("rc-{}", &u.simple().to_string()[..8])
}

// ---------------------------------------------------------------------------
// Serde helpers for enums <-> TEXT
// ---------------------------------------------------------------------------

fn role_to_str(r: &MessageRole) -> &'static str {
    match r {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
    }
}

fn str_to_role(s: &str) -> MessageRole {
    match s {
        "assistant" => MessageRole::Assistant,
        _ => MessageRole::User,
    }
}

fn proposal_status_to_str(s: &ProposalStatus) -> &'static str {
    match s {
        ProposalStatus::Draft => "draft",
        ProposalStatus::Proposed => "proposed",
        ProposalStatus::Accepted => "accepted",
        ProposalStatus::Expired => "expired",
        ProposalStatus::Cancelled => "cancelled",
    }
}

fn str_to_proposal_status(s: &str) -> ProposalStatus {
    match s {
        "proposed" => ProposalStatus::Proposed,
        "accepted" => ProposalStatus::Accepted,
        "expired" => ProposalStatus::Expired,
        "cancelled" => ProposalStatus::Cancelled,
        _ => ProposalStatus::Draft,
    }
}

fn milestone_status_to_str(s: &MilestoneStatus) -> &'static str {
    match s {
        MilestoneStatus::Pending => "pending",
        MilestoneStatus::InProgress => "in_progress",
        MilestoneStatus::Complete => "complete",
        MilestoneStatus::Failed => "failed",
        MilestoneStatus::Skipped => "skipped",
    }
}

fn str_to_milestone_status(s: &str) -> MilestoneStatus {
    match s {
        "in_progress" => MilestoneStatus::InProgress,
        "complete" => MilestoneStatus::Complete,
        "failed" => MilestoneStatus::Failed,
        "skipped" => MilestoneStatus::Skipped,
        _ => MilestoneStatus::Pending,
    }
}

fn run_status_to_str(s: &RunStatus) -> &'static str {
    match s {
        RunStatus::Building => "building",
        RunStatus::Paused => "paused",
        RunStatus::Delivering => "delivering",
        RunStatus::Complete => "complete",
        RunStatus::Failed => "failed",
        RunStatus::Cancelled => "cancelled",
        RunStatus::Disputed => "disputed",
        RunStatus::Settled => "settled",
    }
}

fn str_to_run_status(s: &str) -> RunStatus {
    match s {
        "paused" => RunStatus::Paused,
        "delivering" => RunStatus::Delivering,
        "complete" => RunStatus::Complete,
        "failed" => RunStatus::Failed,
        "cancelled" => RunStatus::Cancelled,
        "disputed" => RunStatus::Disputed,
        "settled" => RunStatus::Settled,
        _ => RunStatus::Building,
    }
}

fn funding_mode_to_str(m: &FundingMode) -> &'static str {
    match m {
        FundingMode::Escrow => "escrow",
        FundingMode::Session => "session",
    }
}

fn str_to_funding_mode(s: &str) -> FundingMode {
    match s {
        "session" => FundingMode::Session,
        _ => FundingMode::Escrow,
    }
}

fn adjustment_type_to_str(t: &AdjustmentType) -> &'static str {
    match t {
        AdjustmentType::TopUp => "top_up",
        AdjustmentType::ScopeReduction => "scope_reduction",
        AdjustmentType::FeatureAddition => "feature_addition",
    }
}

fn str_to_adjustment_type(s: &str) -> AdjustmentType {
    match s {
        "scope_reduction" => AdjustmentType::ScopeReduction,
        "feature_addition" => AdjustmentType::FeatureAddition,
        _ => AdjustmentType::TopUp,
    }
}

fn delivery_type_to_str(t: &DeliveryType) -> &'static str {
    match t {
        DeliveryType::Repo => "repo",
        DeliveryType::Pr => "pr",
        DeliveryType::Deployment => "deployment",
    }
}

fn str_to_delivery_type(s: &str) -> DeliveryType {
    match s {
        "pr" => DeliveryType::Pr,
        "deployment" => DeliveryType::Deployment,
        _ => DeliveryType::Repo,
    }
}

/// Parse an ISO 8601 string into `DateTime<Utc>`, falling back to `Utc::now()` on failure.
fn parse_dt(s: &str) -> DateTime<Utc> {
    s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now())
}

/// Format a `DateTime<Utc>` as an RFC 3339 string (ISO 8601 compatible).
fn fmt_dt(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

// ---------------------------------------------------------------------------
// ServiceDb implementation
// ---------------------------------------------------------------------------

impl ServiceDb {
    /// Open the database at `path`, creating the file and all tables if they don't exist.
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let p = std::path::Path::new(path);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;

             CREATE TABLE IF NOT EXISTS drafts (
                 draft_id             TEXT PRIMARY KEY,
                 repo_url             TEXT,
                 accumulated_context  TEXT NOT NULL DEFAULT '',
                 session_cost_so_far  REAL NOT NULL DEFAULT 0.0,
                 created_at           TEXT NOT NULL,
                 updated_at           TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS draft_messages (
                 id        INTEGER PRIMARY KEY AUTOINCREMENT,
                 draft_id  TEXT NOT NULL REFERENCES drafts(draft_id),
                 role      TEXT NOT NULL,
                 content   TEXT NOT NULL,
                 cost      REAL NOT NULL DEFAULT 0.0,
                 timestamp TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS proposals (
                 proposal_id     TEXT PRIMARY KEY,
                 draft_id        TEXT NOT NULL REFERENCES drafts(draft_id),
                 version         INTEGER NOT NULL DEFAULT 1,
                 total_cost      REAL NOT NULL DEFAULT 0.0,
                 draft_cost_spent REAL NOT NULL DEFAULT 0.0,
                 net_cost        REAL NOT NULL DEFAULT 0.0,
                 valid_for       TEXT NOT NULL DEFAULT '24h',
                 status          TEXT NOT NULL DEFAULT 'draft',
                 created_at      TEXT NOT NULL,
                 expires_at      TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS milestones (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 proposal_id     TEXT NOT NULL REFERENCES proposals(proposal_id),
                 run_id          TEXT,
                 name            TEXT NOT NULL,
                 plans           TEXT NOT NULL DEFAULT '[]',
                 task_count      INTEGER NOT NULL DEFAULT 0,
                 inference_cost  REAL NOT NULL DEFAULT 0.0,
                 compute_cost    REAL NOT NULL DEFAULT 0.0,
                 total_cost      REAL NOT NULL DEFAULT 0.0,
                 estimated_time  TEXT NOT NULL DEFAULT '',
                 actual_cost     REAL,
                 status          TEXT NOT NULL DEFAULT 'pending'
             );

             CREATE TABLE IF NOT EXISTS runs (
                 run_id           TEXT PRIMARY KEY,
                 proposal_id      TEXT NOT NULL REFERENCES proposals(proposal_id),
                 funding_mode     TEXT NOT NULL DEFAULT 'escrow',
                 session_id       TEXT,
                 escrow_address   TEXT,
                 budget_total     REAL NOT NULL DEFAULT 0.0,
                 budget_spent     REAL NOT NULL DEFAULT 0.0,
                 retry_buffer_pct REAL NOT NULL DEFAULT 0.15,
                 status           TEXT NOT NULL DEFAULT 'building',
                 current_plan     TEXT,
                 started_at       TEXT NOT NULL,
                 finished_at      TEXT
             );

             CREATE TABLE IF NOT EXISTS adjustments (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 run_id          TEXT NOT NULL REFERENCES runs(run_id),
                 adjustment_type TEXT NOT NULL,
                 amount          REAL NOT NULL DEFAULT 0.0,
                 description     TEXT NOT NULL DEFAULT '',
                 plans_affected  TEXT NOT NULL DEFAULT '[]',
                 timestamp       TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS deliverables (
                 id            INTEGER PRIMARY KEY AUTOINCREMENT,
                 run_id        TEXT NOT NULL REFERENCES runs(run_id),
                 delivery_type TEXT NOT NULL,
                 url           TEXT NOT NULL,
                 artifact_hash TEXT NOT NULL DEFAULT ''
             );

             CREATE TABLE IF NOT EXISTS receipts (
                 receipt_id           TEXT PRIMARY KEY,
                 proposal_id          TEXT NOT NULL,
                 run_id               TEXT NOT NULL,
                 milestones_completed INTEGER NOT NULL DEFAULT 0,
                 milestones_total     INTEGER NOT NULL DEFAULT 0,
                 total_cost           REAL NOT NULL DEFAULT 0.0,
                 inference_cost       REAL NOT NULL DEFAULT 0.0,
                 compute_cost         REAL NOT NULL DEFAULT 0.0,
                 draft_cost           REAL NOT NULL DEFAULT 0.0,
                 refund               REAL NOT NULL DEFAULT 0.0,
                 receipt_hash         TEXT NOT NULL DEFAULT '',
                 settled_at           TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS account_links (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 wallet_address  TEXT UNIQUE NOT NULL,
                 twitter_handle  TEXT,
                 github_username TEXT,
                 created_at      TEXT NOT NULL
             );

             CREATE INDEX IF NOT EXISTS idx_draft_messages_draft
                 ON draft_messages(draft_id);
             CREATE INDEX IF NOT EXISTS idx_milestones_proposal
                 ON milestones(proposal_id);
             CREATE INDEX IF NOT EXISTS idx_milestones_run
                 ON milestones(run_id);
             CREATE INDEX IF NOT EXISTS idx_adjustments_run
                 ON adjustments(run_id);
             CREATE INDEX IF NOT EXISTS idx_deliverables_run
                 ON deliverables(run_id);
             CREATE INDEX IF NOT EXISTS idx_receipts_run
                 ON receipts(run_id);
             CREATE INDEX IF NOT EXISTS idx_account_links_twitter
                 ON account_links(twitter_handle);
             CREATE INDEX IF NOT EXISTS idx_account_links_github
                 ON account_links(github_username);",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // -----------------------------------------------------------------------
    // Drafts
    // -----------------------------------------------------------------------

    /// Insert a new draft.
    pub fn create_draft(&self, draft: &Draft) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO drafts (draft_id, repo_url, accumulated_context, session_cost_so_far, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                draft.draft_id,
                draft.repo_url,
                draft.accumulated_context,
                draft.session_cost_so_far,
                fmt_dt(&draft.created_at),
                fmt_dt(&draft.updated_at),
            ],
        )?;

        // Persist any messages that came with the draft.
        for msg in &draft.messages {
            conn.execute(
                "INSERT INTO draft_messages (draft_id, role, content, cost, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    draft.draft_id,
                    role_to_str(&msg.role),
                    msg.content,
                    msg.cost,
                    fmt_dt(&msg.timestamp),
                ],
            )?;
        }

        Ok(())
    }

    /// Fetch a draft by ID, including its messages.
    pub fn get_draft(&self, draft_id: &str) -> anyhow::Result<Option<Draft>> {
        let conn = self.conn.lock();

        let maybe = conn
            .query_row(
                "SELECT draft_id, repo_url, accumulated_context, session_cost_so_far, created_at, updated_at
                 FROM drafts WHERE draft_id = ?1",
                params![draft_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?;

        let (id, repo_url, ctx, cost, created, updated) = match maybe {
            Some(t) => t,
            None => return Ok(None),
        };

        let messages = Self::query_messages(&conn, &id)?;

        Ok(Some(Draft {
            draft_id: id,
            repo_url,
            accumulated_context: ctx,
            session_cost_so_far: cost,
            messages,
            created_at: parse_dt(&created),
            updated_at: parse_dt(&updated),
        }))
    }

    /// Update an existing draft's mutable fields.
    pub fn update_draft(&self, draft: &Draft) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE drafts SET repo_url = ?1, accumulated_context = ?2, session_cost_so_far = ?3, updated_at = ?4
             WHERE draft_id = ?5",
            params![
                draft.repo_url,
                draft.accumulated_context,
                draft.session_cost_so_far,
                fmt_dt(&draft.updated_at),
                draft.draft_id,
            ],
        )?;
        Ok(())
    }

    /// Append a single message to a draft.
    pub fn add_draft_message(&self, draft_id: &str, msg: &DraftMessage) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO draft_messages (draft_id, role, content, cost, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                draft_id,
                role_to_str(&msg.role),
                msg.content,
                msg.cost,
                fmt_dt(&msg.timestamp),
            ],
        )?;
        Ok(())
    }

    /// Return all messages for a draft, ordered by insertion.
    pub fn get_draft_messages(&self, draft_id: &str) -> anyhow::Result<Vec<DraftMessage>> {
        let conn = self.conn.lock();
        Self::query_messages(&conn, draft_id)
    }

    /// Internal helper: query draft_messages for a given draft_id. Caller holds the lock.
    fn query_messages(conn: &Connection, draft_id: &str) -> anyhow::Result<Vec<DraftMessage>> {
        let mut stmt = conn.prepare(
            "SELECT role, content, cost, timestamp FROM draft_messages
             WHERE draft_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![draft_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut msgs = Vec::new();
        for r in rows {
            let (role_s, content, cost, ts) = r?;
            msgs.push(DraftMessage {
                role: str_to_role(&role_s),
                content,
                cost,
                timestamp: parse_dt(&ts),
            });
        }
        Ok(msgs)
    }

    // -----------------------------------------------------------------------
    // Proposals
    // -----------------------------------------------------------------------

    /// Insert a new proposal (without milestones -- use `save_milestones` for those).
    pub fn create_proposal(&self, proposal: &Proposal) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO proposals (proposal_id, draft_id, version, total_cost, draft_cost_spent, net_cost, valid_for, status, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                proposal.proposal_id,
                proposal.draft_id,
                proposal.version,
                proposal.total_cost,
                proposal.draft_cost_spent,
                proposal.net_cost,
                proposal.valid_for,
                proposal_status_to_str(&proposal.status),
                fmt_dt(&proposal.created_at),
                fmt_dt(&proposal.expires_at),
            ],
        )?;
        Ok(())
    }

    /// Fetch a proposal by ID, including its milestones.
    pub fn get_proposal(&self, proposal_id: &str) -> anyhow::Result<Option<Proposal>> {
        let conn = self.conn.lock();

        let maybe = conn
            .query_row(
                "SELECT proposal_id, draft_id, version, total_cost, draft_cost_spent, net_cost, valid_for, status, created_at, expires_at
                 FROM proposals WHERE proposal_id = ?1",
                params![proposal_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                },
            )
            .optional()?;

        let (pid, did, ver, tc, dcs, nc, vf, st, ca, ea) = match maybe {
            Some(t) => t,
            None => return Ok(None),
        };

        let milestones = Self::query_milestones_inner(&conn, &pid)?;

        Ok(Some(Proposal {
            proposal_id: pid,
            draft_id: did,
            version: ver,
            total_cost: tc,
            draft_cost_spent: dcs,
            net_cost: nc,
            valid_for: vf,
            status: str_to_proposal_status(&st),
            milestones,
            created_at: parse_dt(&ca),
            expires_at: parse_dt(&ea),
        }))
    }

    /// Update a proposal's scalar fields (not milestones).
    pub fn update_proposal(&self, proposal: &Proposal) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE proposals SET version = ?1, total_cost = ?2, draft_cost_spent = ?3, net_cost = ?4,
             valid_for = ?5, status = ?6, expires_at = ?7
             WHERE proposal_id = ?8",
            params![
                proposal.version,
                proposal.total_cost,
                proposal.draft_cost_spent,
                proposal.net_cost,
                proposal.valid_for,
                proposal_status_to_str(&proposal.status),
                fmt_dt(&proposal.expires_at),
                proposal.proposal_id,
            ],
        )?;
        Ok(())
    }

    /// Replace all milestones for a proposal (delete + re-insert).
    pub fn save_milestones(
        &self,
        proposal_id: &str,
        milestones: &[Milestone],
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM milestones WHERE proposal_id = ?1",
            params![proposal_id],
        )?;
        for m in milestones {
            let plans_json = serde_json::to_string(&m.plans)?;
            conn.execute(
                "INSERT INTO milestones (proposal_id, run_id, name, plans, task_count, inference_cost, compute_cost, total_cost, estimated_time, actual_cost, status)
                 VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    proposal_id,
                    m.name,
                    plans_json,
                    m.task_count,
                    m.estimated_cost.inference,
                    m.estimated_cost.compute,
                    m.estimated_cost.total,
                    m.estimated_time,
                    m.actual_cost,
                    milestone_status_to_str(&m.status),
                ],
            )?;
        }
        Ok(())
    }

    /// Fetch all milestones for a proposal, ordered by insertion.
    pub fn get_milestones(&self, proposal_id: &str) -> anyhow::Result<Vec<Milestone>> {
        let conn = self.conn.lock();
        Self::query_milestones_inner(&conn, proposal_id)
    }

    /// Internal helper: query milestones while the caller already holds the connection lock.
    fn query_milestones_inner(
        conn: &Connection,
        proposal_id: &str,
    ) -> anyhow::Result<Vec<Milestone>> {
        let mut stmt = conn.prepare(
            "SELECT name, plans, task_count, inference_cost, compute_cost, total_cost, estimated_time, actual_cost, status
             FROM milestones WHERE proposal_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![proposal_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;

        let mut milestones = Vec::new();
        for r in rows {
            let (name, plans_json, task_count, inf, comp, total, time, actual, status_s) = r?;
            let plans: Vec<String> = serde_json::from_str(&plans_json).unwrap_or_default();
            milestones.push(Milestone {
                name,
                plans,
                task_count,
                estimated_cost: CostEstimate {
                    inference: inf,
                    compute: comp,
                    total,
                },
                estimated_time: time,
                actual_cost: actual,
                status: str_to_milestone_status(&status_s),
            });
        }
        Ok(milestones)
    }

    // -----------------------------------------------------------------------
    // Runs
    // -----------------------------------------------------------------------

    /// Insert a new run.
    pub fn create_run(&self, run: &Run) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO runs (run_id, proposal_id, funding_mode, session_id, escrow_address, budget_total, budget_spent, retry_buffer_pct, status, current_plan, started_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                run.run_id,
                run.proposal_id,
                funding_mode_to_str(&run.funding_mode),
                run.session_id,
                run.escrow_address,
                run.budget_total,
                run.budget_spent,
                run.retry_buffer_pct,
                run_status_to_str(&run.status),
                run.current_plan,
                fmt_dt(&run.started_at),
                run.finished_at.as_ref().map(fmt_dt),
            ],
        )?;

        // Persist run-level milestones (link to run_id).
        for m in &run.milestones {
            let plans_json = serde_json::to_string(&m.plans)?;
            conn.execute(
                "INSERT INTO milestones (proposal_id, run_id, name, plans, task_count, inference_cost, compute_cost, total_cost, estimated_time, actual_cost, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    run.proposal_id,
                    run.run_id,
                    m.name,
                    plans_json,
                    m.task_count,
                    m.estimated_cost.inference,
                    m.estimated_cost.compute,
                    m.estimated_cost.total,
                    m.estimated_time,
                    m.actual_cost,
                    milestone_status_to_str(&m.status),
                ],
            )?;
        }

        // Persist any initial adjustments.
        for adj in &run.adjustments {
            let plans_json = serde_json::to_string(&adj.plans_affected)?;
            conn.execute(
                "INSERT INTO adjustments (run_id, adjustment_type, amount, description, plans_affected, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    run.run_id,
                    adjustment_type_to_str(&adj.adjustment_type),
                    adj.amount,
                    adj.description,
                    plans_json,
                    fmt_dt(&adj.timestamp),
                ],
            )?;
        }

        Ok(())
    }

    /// Fetch a run by ID, including its milestones and adjustments.
    pub fn get_run(&self, run_id: &str) -> anyhow::Result<Option<Run>> {
        let conn = self.conn.lock();

        let maybe = conn
            .query_row(
                "SELECT run_id, proposal_id, funding_mode, session_id, escrow_address,
                        budget_total, budget_spent, retry_buffer_pct, status, current_plan,
                        started_at, finished_at
                 FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, f64>(6)?,
                        row.get::<_, f64>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            )
            .optional()?;

        let (rid, pid, fm, sid, ea, bt, bs, rbp, st, cp, sa, fa) = match maybe {
            Some(t) => t,
            None => return Ok(None),
        };

        let milestones = Self::query_run_milestones(&conn, &rid)?;
        let adjustments = Self::query_adjustments_inner(&conn, &rid)?;

        Ok(Some(Run {
            run_id: rid,
            proposal_id: pid,
            funding_mode: str_to_funding_mode(&fm),
            session_id: sid,
            escrow_address: ea,
            budget_total: bt,
            budget_spent: bs,
            retry_buffer_pct: rbp,
            status: str_to_run_status(&st),
            current_plan: cp,
            milestones,
            adjustments,
            started_at: parse_dt(&sa),
            finished_at: fa.as_deref().map(parse_dt),
        }))
    }

    /// Update all mutable fields on a run.
    pub fn update_run(&self, run: &Run) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE runs SET funding_mode = ?1, session_id = ?2, escrow_address = ?3,
             budget_total = ?4, budget_spent = ?5, retry_buffer_pct = ?6,
             status = ?7, current_plan = ?8, finished_at = ?9
             WHERE run_id = ?10",
            params![
                funding_mode_to_str(&run.funding_mode),
                run.session_id,
                run.escrow_address,
                run.budget_total,
                run.budget_spent,
                run.retry_buffer_pct,
                run_status_to_str(&run.status),
                run.current_plan,
                run.finished_at.as_ref().map(fmt_dt),
                run.run_id,
            ],
        )?;
        Ok(())
    }

    /// Update only a run's status.
    pub fn update_run_status(&self, run_id: &str, status: RunStatus) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        let finished = match status {
            RunStatus::Complete | RunStatus::Failed | RunStatus::Cancelled | RunStatus::Settled => {
                Some(fmt_dt(&Utc::now()))
            }
            _ => None,
        };
        conn.execute(
            "UPDATE runs SET status = ?1, finished_at = COALESCE(?2, finished_at) WHERE run_id = ?3",
            params![run_status_to_str(&status), finished, run_id],
        )?;
        Ok(())
    }

    /// Update only a run's budget_spent.
    pub fn update_run_cost(&self, run_id: &str, budget_spent: f64) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE runs SET budget_spent = ?1 WHERE run_id = ?2",
            params![budget_spent, run_id],
        )?;
        Ok(())
    }

    /// Internal helper: query milestones linked to a specific run.
    fn query_run_milestones(conn: &Connection, run_id: &str) -> anyhow::Result<Vec<Milestone>> {
        let mut stmt = conn.prepare(
            "SELECT name, plans, task_count, inference_cost, compute_cost, total_cost, estimated_time, actual_cost, status
             FROM milestones WHERE run_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;

        let mut milestones = Vec::new();
        for r in rows {
            let (name, plans_json, task_count, inf, comp, total, time, actual, status_s) = r?;
            let plans: Vec<String> = serde_json::from_str(&plans_json).unwrap_or_default();
            milestones.push(Milestone {
                name,
                plans,
                task_count,
                estimated_cost: CostEstimate {
                    inference: inf,
                    compute: comp,
                    total,
                },
                estimated_time: time,
                actual_cost: actual,
                status: str_to_milestone_status(&status_s),
            });
        }
        Ok(milestones)
    }

    // -----------------------------------------------------------------------
    // Adjustments
    // -----------------------------------------------------------------------

    /// Record a cost adjustment against a run.
    pub fn add_adjustment(&self, run_id: &str, adj: &Adjustment) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        let plans_json = serde_json::to_string(&adj.plans_affected)?;
        conn.execute(
            "INSERT INTO adjustments (run_id, adjustment_type, amount, description, plans_affected, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                run_id,
                adjustment_type_to_str(&adj.adjustment_type),
                adj.amount,
                adj.description,
                plans_json,
                fmt_dt(&adj.timestamp),
            ],
        )?;
        Ok(())
    }

    /// Fetch all adjustments for a run, ordered by insertion.
    pub fn get_adjustments(&self, run_id: &str) -> anyhow::Result<Vec<Adjustment>> {
        let conn = self.conn.lock();
        Self::query_adjustments_inner(&conn, run_id)
    }

    /// Internal helper: query adjustments while the caller already holds the lock.
    fn query_adjustments_inner(conn: &Connection, run_id: &str) -> anyhow::Result<Vec<Adjustment>> {
        let mut stmt = conn.prepare(
            "SELECT adjustment_type, amount, description, plans_affected, timestamp
             FROM adjustments WHERE run_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut adjs = Vec::new();
        for r in rows {
            let (type_s, amount, desc, plans_json, ts) = r?;
            let plans: Vec<String> = serde_json::from_str(&plans_json).unwrap_or_default();
            adjs.push(Adjustment {
                adjustment_type: str_to_adjustment_type(&type_s),
                amount,
                description: desc,
                plans_affected: plans,
                timestamp: parse_dt(&ts),
            });
        }
        Ok(adjs)
    }

    // -----------------------------------------------------------------------
    // Deliverables
    // -----------------------------------------------------------------------

    /// Attach a deliverable to a run.
    pub fn add_deliverable(&self, run_id: &str, d: &Deliverable) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO deliverables (run_id, delivery_type, url, artifact_hash)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                run_id,
                delivery_type_to_str(&d.delivery_type),
                d.url,
                d.artifact_hash,
            ],
        )?;
        Ok(())
    }

    /// Fetch all deliverables for a run.
    pub fn get_deliverables(&self, run_id: &str) -> anyhow::Result<Vec<Deliverable>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT delivery_type, url, artifact_hash FROM deliverables
             WHERE run_id = ?1 ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut deliverables = Vec::new();
        for r in rows {
            let (type_s, url, hash) = r?;
            deliverables.push(Deliverable {
                delivery_type: str_to_delivery_type(&type_s),
                url,
                artifact_hash: hash,
            });
        }
        Ok(deliverables)
    }

    // -----------------------------------------------------------------------
    // Receipts
    // -----------------------------------------------------------------------

    /// Store a settlement receipt.
    pub fn create_receipt(&self, receipt: &Receipt) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO receipts (receipt_id, proposal_id, run_id, milestones_completed, milestones_total, total_cost, inference_cost, compute_cost, draft_cost, refund, receipt_hash, settled_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                generate_receipt_id(),
                receipt.proposal_id,
                receipt.run_id,
                receipt.milestones_completed,
                receipt.milestones_total,
                receipt.total_cost,
                receipt.breakdown.inference,
                receipt.breakdown.compute,
                receipt.draft_cost,
                receipt.refund,
                receipt.receipt_hash,
                fmt_dt(&receipt.settled_at),
            ],
        )?;
        Ok(())
    }

    /// Fetch the receipt for a given run.
    pub fn get_receipt(&self, run_id: &str) -> anyhow::Result<Option<Receipt>> {
        let conn = self.conn.lock();
        let maybe = conn
            .query_row(
                "SELECT proposal_id, run_id, milestones_completed, milestones_total,
                        total_cost, inference_cost, compute_cost, draft_cost, refund,
                        receipt_hash, settled_at
                 FROM receipts WHERE run_id = ?1",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, u32>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, f64>(5)?,
                        row.get::<_, f64>(6)?,
                        row.get::<_, f64>(7)?,
                        row.get::<_, f64>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                },
            )
            .optional()?;

        let (pid, rid, mc, mt, tc, inf, comp, dc, refund, hash, settled) = match maybe {
            Some(t) => t,
            None => return Ok(None),
        };

        // Fetch adjustments for this run to populate the receipt's adjustments vec.
        let adjustments = Self::query_adjustments_inner(&conn, &rid)?;

        Ok(Some(Receipt {
            proposal_id: pid,
            run_id: rid,
            milestones_completed: mc,
            milestones_total: mt,
            total_cost: tc,
            breakdown: CostEstimate {
                inference: inf,
                compute: comp,
                total: tc,
            },
            draft_cost: dc,
            adjustments,
            refund,
            receipt_hash: hash,
            settled_at: parse_dt(&settled),
        }))
    }

    // -----------------------------------------------------------------------
    // Account linking
    // -----------------------------------------------------------------------

    /// Link a Twitter handle to a wallet address (upsert).
    pub fn link_twitter(&self, wallet: &str, handle: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO account_links (wallet_address, twitter_handle, created_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(wallet_address) DO UPDATE SET twitter_handle = excluded.twitter_handle",
            params![wallet, handle, fmt_dt(&Utc::now())],
        )?;
        Ok(())
    }

    /// Link a GitHub username to a wallet address (upsert).
    pub fn link_github(&self, wallet: &str, username: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO account_links (wallet_address, github_username, created_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(wallet_address) DO UPDATE SET github_username = excluded.github_username",
            params![wallet, username, fmt_dt(&Utc::now())],
        )?;
        Ok(())
    }

    /// Validate an API key against the database.
    ///
    /// Returns `Ok(None)` when the key is not found. Database-backed API keys
    /// are not implemented yet; the master key path in `auth.rs` handles the
    /// configured key.
    pub fn validate_api_key(
        &self,
        _token: &str,
    ) -> Result<Option<crate::api::auth::AuthContext>, anyhow::Error> {
        Ok(None)
    }

    /// Look up the wallet address linked to a Twitter handle.
    pub fn get_wallet_for_twitter(&self, handle: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock();
        let result = conn
            .query_row(
                "SELECT wallet_address FROM account_links WHERE twitter_handle = ?1",
                params![handle],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Look up the wallet address linked to a GitHub username.
    pub fn get_wallet_for_github(&self, username: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock();
        let result = conn
            .query_row(
                "SELECT wallet_address FROM account_links WHERE github_username = ?1",
                params![username],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_db() -> ServiceDb {
        ServiceDb::open(":memory:").expect("open in-memory db")
    }

    #[test]
    fn draft_roundtrip() {
        let db = test_db();
        let now = Utc::now();
        let draft = Draft {
            draft_id: generate_draft_id(),
            repo_url: Some("https://github.com/example/repo".into()),
            accumulated_context: "build me a thing".into(),
            session_cost_so_far: 0.05,
            messages: vec![DraftMessage {
                role: MessageRole::User,
                content: "hello".into(),
                cost: 0.01,
                timestamp: now,
            }],
            created_at: now,
            updated_at: now,
        };

        db.create_draft(&draft).unwrap();
        let loaded = db.get_draft(&draft.draft_id).unwrap().unwrap();
        assert_eq!(loaded.draft_id, draft.draft_id);
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "hello");

        // Add another message.
        db.add_draft_message(
            &draft.draft_id,
            &DraftMessage {
                role: MessageRole::Assistant,
                content: "sure".into(),
                cost: 0.02,
                timestamp: now,
            },
        )
        .unwrap();

        let msgs = db.get_draft_messages(&draft.draft_id).unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn proposal_milestones_roundtrip() {
        let db = test_db();
        let now = Utc::now();

        // Need a draft first (FK).
        let draft = Draft {
            draft_id: generate_draft_id(),
            repo_url: None,
            accumulated_context: String::new(),
            session_cost_so_far: 0.0,
            messages: vec![],
            created_at: now,
            updated_at: now,
        };
        db.create_draft(&draft).unwrap();

        let proposal = Proposal {
            proposal_id: generate_proposal_id(),
            draft_id: draft.draft_id.clone(),
            version: 1,
            total_cost: 10.0,
            draft_cost_spent: 0.5,
            net_cost: 9.5,
            valid_for: "24h".into(),
            status: ProposalStatus::Proposed,
            milestones: vec![],
            created_at: now,
            expires_at: now,
        };
        db.create_proposal(&proposal).unwrap();

        let milestones = vec![Milestone {
            name: "core".into(),
            plans: vec!["01-models".into(), "02-api".into()],
            task_count: 8,
            estimated_cost: CostEstimate {
                inference: 3.0,
                compute: 1.0,
                total: 4.0,
            },
            estimated_time: "~30 min".into(),
            actual_cost: None,
            status: MilestoneStatus::Pending,
        }];
        db.save_milestones(&proposal.proposal_id, &milestones)
            .unwrap();

        let loaded = db.get_proposal(&proposal.proposal_id).unwrap().unwrap();
        assert_eq!(loaded.milestones.len(), 1);
        assert_eq!(loaded.milestones[0].plans.len(), 2);
        assert_eq!(loaded.milestones[0].plans[0], "01-models");
    }

    #[test]
    fn run_lifecycle() {
        let db = test_db();
        let now = Utc::now();

        let draft = Draft {
            draft_id: generate_draft_id(),
            repo_url: None,
            accumulated_context: String::new(),
            session_cost_so_far: 0.0,
            messages: vec![],
            created_at: now,
            updated_at: now,
        };
        db.create_draft(&draft).unwrap();

        let proposal = Proposal {
            proposal_id: generate_proposal_id(),
            draft_id: draft.draft_id.clone(),
            version: 1,
            total_cost: 10.0,
            draft_cost_spent: 0.0,
            net_cost: 10.0,
            valid_for: "24h".into(),
            status: ProposalStatus::Accepted,
            milestones: vec![],
            created_at: now,
            expires_at: now,
        };
        db.create_proposal(&proposal).unwrap();

        let run = Run {
            run_id: generate_run_id(),
            proposal_id: proposal.proposal_id.clone(),
            funding_mode: FundingMode::Escrow,
            session_id: None,
            escrow_address: Some("0xabc".into()),
            budget_total: 10.0,
            budget_spent: 0.0,
            retry_buffer_pct: 0.15,
            status: RunStatus::Building,
            milestones: vec![],
            adjustments: vec![],
            current_plan: None,
            started_at: now,
            finished_at: None,
        };
        db.create_run(&run).unwrap();

        db.update_run_cost(&run.run_id, 3.5).unwrap();
        db.update_run_status(&run.run_id, RunStatus::Complete)
            .unwrap();

        let loaded = db.get_run(&run.run_id).unwrap().unwrap();
        assert_eq!(loaded.budget_spent, 3.5);
        assert_eq!(loaded.status, RunStatus::Complete);
        assert!(loaded.finished_at.is_some());
    }

    #[test]
    fn deliverables_and_receipts() {
        let db = test_db();
        let now = Utc::now();

        // Scaffold draft -> proposal -> run.
        let draft = Draft {
            draft_id: generate_draft_id(),
            repo_url: None,
            accumulated_context: String::new(),
            session_cost_so_far: 0.0,
            messages: vec![],
            created_at: now,
            updated_at: now,
        };
        db.create_draft(&draft).unwrap();
        let proposal = Proposal {
            proposal_id: generate_proposal_id(),
            draft_id: draft.draft_id.clone(),
            version: 1,
            total_cost: 5.0,
            draft_cost_spent: 0.0,
            net_cost: 5.0,
            valid_for: "24h".into(),
            status: ProposalStatus::Accepted,
            milestones: vec![],
            created_at: now,
            expires_at: now,
        };
        db.create_proposal(&proposal).unwrap();
        let run = Run {
            run_id: generate_run_id(),
            proposal_id: proposal.proposal_id.clone(),
            funding_mode: FundingMode::Session,
            session_id: Some("sess-1".into()),
            escrow_address: None,
            budget_total: 5.0,
            budget_spent: 4.2,
            retry_buffer_pct: 0.15,
            status: RunStatus::Complete,
            milestones: vec![],
            adjustments: vec![],
            current_plan: None,
            started_at: now,
            finished_at: Some(now),
        };
        db.create_run(&run).unwrap();

        db.add_deliverable(
            &run.run_id,
            &Deliverable {
                delivery_type: DeliveryType::Pr,
                url: "https://github.com/example/repo/pull/42".into(),
                artifact_hash: "abc123".into(),
            },
        )
        .unwrap();

        let ds = db.get_deliverables(&run.run_id).unwrap();
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].url, "https://github.com/example/repo/pull/42");

        let receipt = Receipt {
            proposal_id: proposal.proposal_id.clone(),
            run_id: run.run_id.clone(),
            milestones_completed: 1,
            milestones_total: 1,
            total_cost: 4.2,
            breakdown: CostEstimate {
                inference: 3.0,
                compute: 1.2,
                total: 4.2,
            },
            draft_cost: 0.5,
            adjustments: vec![],
            refund: 0.3,
            receipt_hash: "deadbeef".into(),
            settled_at: now,
        };
        db.create_receipt(&receipt).unwrap();

        let loaded = db.get_receipt(&run.run_id).unwrap().unwrap();
        assert_eq!(loaded.total_cost, 4.2);
        assert_eq!(loaded.refund, 0.3);
    }

    #[test]
    fn account_linking() {
        let db = test_db();

        db.link_twitter("0xwallet1", "@alice").unwrap();
        db.link_github("0xwallet1", "alice-gh").unwrap();

        assert_eq!(
            db.get_wallet_for_twitter("@alice").unwrap(),
            Some("0xwallet1".into())
        );
        assert_eq!(
            db.get_wallet_for_github("alice-gh").unwrap(),
            Some("0xwallet1".into())
        );
        assert_eq!(db.get_wallet_for_twitter("@bob").unwrap(), None);

        // Upsert: same wallet, new twitter handle.
        db.link_twitter("0xwallet1", "@alice_new").unwrap();
        assert_eq!(
            db.get_wallet_for_twitter("@alice_new").unwrap(),
            Some("0xwallet1".into())
        );
    }
}

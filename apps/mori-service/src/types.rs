//! Core domain types for Mori-as-a-Service.
//!
//! These types model the full lifecycle: draft -> proposal -> run -> delivery -> settlement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// IDs
// ---------------------------------------------------------------------------

/// Draft identifier (e.g. "d-1a2b").
pub type DraftId = String;
/// Proposal identifier (e.g. "p-3c4d").
pub type ProposalId = String;
/// Run identifier (e.g. "r-5e6f").
pub type RunId = String;
/// Receipt identifier.
pub type ReceiptId = String;

// ---------------------------------------------------------------------------
// Draft
// ---------------------------------------------------------------------------

/// A conversational workspace where a PRD takes shape via iterative x402 interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub draft_id: DraftId,
    pub messages: Vec<DraftMessage>,
    /// Accumulated PRD fragments, decisions, clarifications.
    pub accumulated_context: String,
    /// Total cost of drafting interactions so far (USD).
    pub session_cost_so_far: f64,
    /// Optional repo URL for codebase context.
    pub repo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftMessage {
    pub role: MessageRole,
    pub content: String,
    pub cost: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
}

// ---------------------------------------------------------------------------
// Proposal
// ---------------------------------------------------------------------------

/// A costed estimate generated from a draft, broken into milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: ProposalId,
    pub draft_id: DraftId,
    pub version: u32,
    pub milestones: Vec<Milestone>,
    pub total_cost: f64,
    pub draft_cost_spent: f64,
    pub net_cost: f64,
    /// Duration string like "24h".
    pub valid_for: String,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    /// Plan IDs (e.g. ["01-user-model", "02-session-store"]).
    pub plans: Vec<String>,
    pub task_count: u32,
    pub estimated_cost: CostEstimate,
    /// Human-readable time estimate (e.g. "~45 min").
    pub estimated_time: String,
    pub actual_cost: Option<f64>,
    pub status: MilestoneStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CostEstimate {
    /// LLM token costs.
    pub inference: f64,
    /// Non-inference: CI, compilation, deployment infra.
    pub compute: f64,
    /// inference + compute.
    pub total: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Proposed,
    Accepted,
    Expired,
    Cancelled,
}

/// Interaction mode for Twitter and other conversational surfaces.
///
/// Simple mode produces a single compact quote. Conversational mode breaks the
/// project into milestones and invites multi-turn refinement before commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionMode {
    /// Under cost threshold -- quick quote, single approval.
    Simple,
    /// Over threshold or ambiguous -- multi-turn refinement.
    Conversational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    Skipped,
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

/// An active build execution triggered by accepting a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub run_id: RunId,
    pub proposal_id: ProposalId,
    pub funding_mode: FundingMode,
    pub session_id: Option<String>,
    pub escrow_address: Option<String>,
    pub budget_total: f64,
    pub budget_spent: f64,
    pub retry_buffer_pct: f64,
    pub status: RunStatus,
    pub milestones: Vec<Milestone>,
    pub adjustments: Vec<Adjustment>,
    pub current_plan: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundingMode {
    Escrow,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Building,
    Paused,
    Delivering,
    Complete,
    Failed,
    Cancelled,
    Disputed,
    Settled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    pub adjustment_type: AdjustmentType,
    pub amount: f64,
    pub description: String,
    pub plans_affected: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    TopUp,
    ScopeReduction,
    FeatureAddition,
}

// ---------------------------------------------------------------------------
// Delivery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    #[serde(rename = "type")]
    pub delivery_type: DeliveryType,
    pub url: String,
    pub artifact_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    Repo,
    Pr,
    Deployment,
}

// ---------------------------------------------------------------------------
// Receipt
// ---------------------------------------------------------------------------

/// Settlement receipt produced after a run completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub proposal_id: ProposalId,
    pub run_id: RunId,
    pub milestones_completed: u32,
    pub milestones_total: u32,
    pub total_cost: f64,
    pub breakdown: CostEstimate,
    pub draft_cost: f64,
    pub adjustments: Vec<Adjustment>,
    pub refund: f64,
    pub receipt_hash: String,
    pub settled_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// SSE Events
// ---------------------------------------------------------------------------

/// Events streamed to clients during a build via SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BuildEvent {
    PlanStarted {
        plan: String,
        milestone: String,
        timestamp: i64,
    },
    PlanCompleted {
        plan: String,
        cost: f64,
        duration: String,
        timestamp: i64,
    },
    GatePassed {
        plan: String,
        gate: String,
        timestamp: i64,
    },
    GateFailed {
        plan: String,
        gate: String,
        reason: String,
        retry: u32,
        max_retries: u32,
        timestamp: i64,
    },
    ReviewSubmitted {
        plan: String,
        verdict: String,
        issues: u32,
        timestamp: i64,
    },
    BudgetAlert {
        threshold: u32,
        run_cost: f64,
        budget_total: f64,
        reason: String,
        description: String,
        additional_cost_estimate: Option<f64>,
        action_required: String,
    },
    CostUpdate {
        run_cost: f64,
        budget_remaining: f64,
        pct_used: u32,
    },
    BuildComplete {
        run_id: String,
        milestones_completed: u32,
        total_cost: f64,
        deliverables: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// API request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateDraftRequest {
    pub content: String,
    pub repo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IterateDraftRequest {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateProposalRequest {
    pub draft_id: DraftId,
}

#[derive(Debug, Deserialize)]
pub struct ModifyProposalRequest {
    pub add_milestones: Option<Vec<MilestoneDescription>>,
    pub remove_milestones: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MilestoneDescription {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptProposalRequest {
    pub funding: FundingMode,
    pub deposit: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ReduceScopeRequest {
    pub skip_plans: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddFeatureRequest {
    pub description: String,
    pub max_additional_cost: f64,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Service-level configuration loaded from .mori/config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub billing: BillingConfig,
    pub auth: AuthConfig,
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    pub accept_proposals: bool,
    pub min_proposal_value: f64,
    pub max_proposal_value: f64,
    /// Fraction reserved for retries (e.g. 0.15 = 15%).
    pub retry_buffer_pct: f64,
    #[serde(default)]
    pub escrow: EscrowConfig,
    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscrowConfig {
    pub evaluator: String,
    pub release_per_milestone: bool,
    pub dispute_window_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    pub min_deposit: f64,
    pub max_duration_hours: u32,
    pub auto_close_idle_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// "api_key", "siwe", or "both".
    pub mode: String,
    /// Optional master API key (from env or config). Always gets Admin scope.
    #[serde(default)]
    pub master_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub spread_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub github: Option<GitHubConfig>,
    #[serde(default)]
    pub twitter: Option<TwitterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub app_id: u64,
    pub private_key_path: String,
    pub webhook_secret: String,
    pub bot_username: String,
    #[serde(default)]
    pub defaults: GitHubDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubDefaults {
    pub auto_propose_on_issue: bool,
    pub auto_build_on_approve: bool,
    pub max_cost_per_build: f64,
    pub approver_teams: Vec<String>,
    pub review_on_pr_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub bearer_token: String,
    pub bot_username: String,
    pub webhook_url: Option<String>,
    pub simple_mode_threshold: f64,
    pub max_requests_per_user_hour: u32,
    pub min_account_age_days: u32,
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub blocklist: Vec<String>,
}

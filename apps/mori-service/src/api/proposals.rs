//! Proposal endpoints.
//!
//! Proposals are costed estimates generated from drafts, broken into milestones
//! with time and cost estimates. Clients can modify, accept, or let them expire.

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use serde_json::json;

use crate::state::ServiceState;
use crate::types::{
    AcceptProposalRequest, FundingMode, GenerateProposalRequest, ModifyProposalRequest,
    ProposalStatus, Run, RunStatus,
};

use super::auth::AppError;

/// Mount proposal routes.
pub fn router() -> Router<ServiceState> {
    Router::new()
        .route("/proposals", post(generate_proposal))
        .route("/proposals/{id}", get(get_proposal))
        .route("/proposals/{id}/modify", post(modify_proposal))
        .route("/proposals/{id}/accept", post(accept_proposal))
}

// ---------------------------------------------------------------------------
// POST /proposals
// ---------------------------------------------------------------------------

/// Generate a proposal from a draft. Calls the ProposalEngine to produce
/// milestones, cost estimates, and time estimates.
async fn generate_proposal(
    State(state): State<ServiceState>,
    Json(payload): Json<GenerateProposalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let draft = state
        .db
        .get_draft(&payload.draft_id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("draft {} not found", payload.draft_id)))?;

    let proposal = state
        .engine
        .generate_proposal(&draft, state.config.gateway.spread_pct)
        .map_err(|e| AppError::Internal(e.into()))?;

    state
        .db
        .create_proposal(&proposal)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "proposal_id": proposal.proposal_id,
        "milestones": proposal.milestones.iter().map(|m| json!({
            "name": m.name,
            "plans": m.plans,
            "tasks": m.task_count,
            "estimated_cost": {
                "inference": m.estimated_cost.inference,
                "compute": m.estimated_cost.compute,
                "total": m.estimated_cost.total,
            },
            "estimated_time": m.estimated_time,
        })).collect::<Vec<_>>(),
        "total_cost": proposal.total_cost,
        "draft_cost_spent": proposal.draft_cost_spent,
        "net_cost": proposal.net_cost,
        "valid_for": proposal.valid_for,
    })))
}

// ---------------------------------------------------------------------------
// GET /proposals/:id
// ---------------------------------------------------------------------------

/// Retrieve a proposal by ID.
async fn get_proposal(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let proposal = state
        .db
        .get_proposal(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("proposal {id} not found")))?;

    Ok(Json(json!({
        "proposal_id": proposal.proposal_id,
        "version": proposal.version,
        "milestones": proposal.milestones.iter().map(|m| json!({
            "name": m.name,
            "plans": m.plans,
            "tasks": m.task_count,
            "estimated_cost": {
                "inference": m.estimated_cost.inference,
                "compute": m.estimated_cost.compute,
                "total": m.estimated_cost.total,
            },
            "estimated_time": m.estimated_time,
            "status": m.status,
        })).collect::<Vec<_>>(),
        "total_cost": proposal.total_cost,
        "draft_cost_spent": proposal.draft_cost_spent,
        "net_cost": proposal.net_cost,
        "valid_for": proposal.valid_for,
        "status": proposal.status,
        "created_at": proposal.created_at,
        "expires_at": proposal.expires_at,
    })))
}

// ---------------------------------------------------------------------------
// POST /proposals/:id/modify
// ---------------------------------------------------------------------------

/// Adjust milestones on an existing proposal: add new ones, remove others,
/// and re-estimate costs. The proposal_id stays the same; version increments.
async fn modify_proposal(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
    Json(payload): Json<ModifyProposalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let proposal = state
        .db
        .get_proposal(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("proposal {id} not found")))?;

    if proposal.status != ProposalStatus::Proposed && proposal.status != ProposalStatus::Draft {
        return Err(AppError::Conflict(format!(
            "proposal {id} has status {:?} and cannot be modified",
            proposal.status
        )));
    }

    let updated = state
        .engine
        .modify_proposal(proposal, &payload, state.config.gateway.spread_pct)
        .map_err(|e| AppError::Internal(e.into()))?;

    state
        .db
        .update_proposal(&updated)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "proposal_id": updated.proposal_id,
        "version": updated.version,
        "milestones": updated.milestones.iter().map(|m| json!({
            "name": m.name,
            "plans": m.plans,
            "tasks": m.task_count,
            "estimated_cost": {
                "inference": m.estimated_cost.inference,
                "compute": m.estimated_cost.compute,
                "total": m.estimated_cost.total,
            },
            "estimated_time": m.estimated_time,
        })).collect::<Vec<_>>(),
        "total_cost": updated.total_cost,
        "draft_cost_spent": updated.draft_cost_spent,
        "net_cost": updated.net_cost,
        "valid_for": updated.valid_for,
    })))
}

// ---------------------------------------------------------------------------
// POST /proposals/:id/accept
// ---------------------------------------------------------------------------

/// Accept a proposal, fund it, and create a run.
///
/// The `funding` field selects escrow (on-chain ERC-8183) or session (MPP).
/// In production the `X-Payment` header carries the signed USDC authorization;
/// for now we just create the run and return funding info.
async fn accept_proposal(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
    Json(payload): Json<AcceptProposalRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut proposal = state
        .db
        .get_proposal(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("proposal {id} not found")))?;

    if proposal.status != ProposalStatus::Proposed {
        return Err(AppError::Conflict(format!(
            "proposal {id} has status {:?} and cannot be accepted",
            proposal.status
        )));
    }

    // Check expiry.
    if Utc::now() > proposal.expires_at {
        proposal.status = ProposalStatus::Expired;
        let _ = state.db.update_proposal(&proposal);
        return Err(AppError::Conflict(format!("proposal {id} has expired")));
    }

    // Validate deposit if provided.
    if let Some(deposit) = payload.deposit {
        if deposit < proposal.net_cost {
            return Err(AppError::PaymentRequired(format!(
                "deposit ${:.2} is less than net cost ${:.2}",
                deposit, proposal.net_cost
            )));
        }
    }

    let run_id = generate_run_id();
    let now = Utc::now();

    // Determine funding-specific fields.
    let (session_id, escrow_address) = match payload.funding {
        FundingMode::Session => (Some(generate_session_id()), None),
        FundingMode::Escrow => (None, Some(generate_escrow_address())),
    };

    let run = Run {
        run_id: run_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        funding_mode: payload.funding,
        session_id: session_id.clone(),
        escrow_address: escrow_address.clone(),
        budget_total: payload.deposit.unwrap_or(proposal.net_cost),
        budget_spent: 0.0,
        retry_buffer_pct: state.config.billing.retry_buffer_pct,
        status: RunStatus::Building,
        milestones: proposal.milestones.clone(),
        adjustments: vec![],
        current_plan: proposal
            .milestones
            .first()
            .and_then(|m| m.plans.first().cloned()),
        started_at: now,
        finished_at: None,
    };

    // Mark proposal as accepted.
    proposal.status = ProposalStatus::Accepted;
    state
        .db
        .update_proposal(&proposal)
        .map_err(|e| AppError::Internal(e.into()))?;

    state
        .db
        .create_run(&run)
        .map_err(|e| AppError::Internal(e.into()))?;

    // Initialize the SSE broadcast channel for this run.
    state.event_sender(&run_id);

    let mut resp = json!({
        "run_id": run_id,
        "status": "building",
    });

    if let Some(addr) = escrow_address {
        resp["escrow_address"] = json!(addr);
    }
    if let Some(sid) = session_id {
        resp["session_id"] = json!(sid);
    }

    Ok(Json(resp))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_run_id() -> String {
    let id = uuid::Uuid::new_v4();
    format!("r-{}", &id.to_string()[..8])
}

fn generate_session_id() -> String {
    let id = uuid::Uuid::new_v4();
    format!("s-{}", &id.to_string()[..8])
}

fn generate_escrow_address() -> String {
    // Placeholder: in production this deploys or references an on-chain escrow.
    let id = uuid::Uuid::new_v4();
    format!("0x{}", &id.to_string().replace('-', "")[..40])
}

//! Delivery endpoints.
//!
//! Once a run completes, clients retrieve build artifacts and the
//! settlement receipt showing final cost breakdown and any refund.

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde_json::json;

use crate::state::ServiceState;
use crate::types::RunStatus;

use super::auth::AppError;

/// Mount delivery routes.
pub fn router() -> Router<ServiceState> {
    Router::new()
        .route("/runs/{id}/deliverables", get(list_deliverables))
        .route("/runs/{id}/receipt", get(get_receipt))
}

// ---------------------------------------------------------------------------
// GET /runs/:id/deliverables
// ---------------------------------------------------------------------------

/// List deliverables produced by a completed (or partially completed) run.
///
/// Deliverables include repo URLs, pull request links, and deployment URLs,
/// each with an artifact hash for verification.
async fn list_deliverables(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Verify the run exists.
    let run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    // Only runs that have reached a terminal or delivering state have deliverables.
    if matches!(run.status, RunStatus::Building | RunStatus::Paused) {
        return Ok(Json(json!({
            "deliverables": [],
            "note": "run is still in progress",
        })));
    }

    let deliverables = state
        .db
        .get_deliverables(&id)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "deliverables": deliverables.iter().map(|d| json!({
            "type": d.delivery_type,
            "url": d.url,
            "artifact_hash": d.artifact_hash,
        })).collect::<Vec<_>>(),
    })))
}

// ---------------------------------------------------------------------------
// GET /runs/:id/receipt
// ---------------------------------------------------------------------------

/// Get the settlement receipt for a completed run.
///
/// The receipt contains the final cost breakdown, milestone completion counts,
/// any adjustments (top-ups, scope changes), refund amount, and a receipt hash
/// for on-chain verification.
async fn get_receipt(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Verify the run exists.
    let run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    // Receipt is only available for terminal states.
    if matches!(
        run.status,
        RunStatus::Building | RunStatus::Paused | RunStatus::Delivering
    ) {
        return Err(AppError::Conflict(format!(
            "run {id} has status {:?}; receipt not yet available",
            run.status
        )));
    }

    let receipt = state
        .db
        .get_receipt(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("receipt for run {id} not found")))?;

    Ok(Json(json!({
        "proposal_id": receipt.proposal_id,
        "run_id": receipt.run_id,
        "milestones_completed": receipt.milestones_completed,
        "milestones_total": receipt.milestones_total,
        "total_cost": receipt.total_cost,
        "breakdown": {
            "inference": receipt.breakdown.inference,
            "compute": receipt.breakdown.compute,
            "total": receipt.breakdown.total,
        },
        "draft_cost": receipt.draft_cost,
        "adjustments": receipt.adjustments.iter().map(|a| json!({
            "type": a.adjustment_type,
            "amount": a.amount,
            "description": a.description,
        })).collect::<Vec<_>>(),
        "refund": receipt.refund,
        "receipt_hash": receipt.receipt_hash,
        "settled_at": receipt.settled_at,
    })))
}

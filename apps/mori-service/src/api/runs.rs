//! Run endpoints.
//!
//! A run is an active build execution triggered by accepting a proposal.
//! Endpoints here cover status, live cost, SSE streaming, budget adjustments,
//! scope changes, and cancellation.

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{delete, get, post},
};
use chrono::Utc;
use futures_util::stream::Stream;
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::state::ServiceState;
use crate::types::{
    AddFeatureRequest, Adjustment, AdjustmentType, MilestoneStatus, ReduceScopeRequest, RunStatus,
};

use super::auth::AppError;

/// Mount run routes.
pub fn router() -> Router<ServiceState> {
    Router::new()
        .route("/runs/{id}", get(get_run))
        .route("/runs/{id}", delete(cancel_run))
        .route("/runs/{id}/cost", get(get_run_cost))
        .route("/runs/{id}/events", get(run_events))
        .route("/runs/{id}/top-up", post(top_up))
        .route("/runs/{id}/reduce-scope", post(reduce_scope))
        .route("/runs/{id}/add-feature", post(add_feature))
}

// ---------------------------------------------------------------------------
// GET /runs/:id
// ---------------------------------------------------------------------------

/// Get the current status of a run.
async fn get_run(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    let elapsed = {
        let dur = Utc::now() - run.started_at;
        let secs = dur.num_seconds();
        format!("{}m{}s", secs / 60, secs % 60)
    };

    Ok(Json(json!({
        "run_id": run.run_id,
        "status": run.status,
        "milestones": run.milestones.iter().map(|m| {
            let completed = m.plans.iter().filter(|_| {
                // In a real implementation, plan statuses would be tracked individually.
                // For now, approximate: if milestone is complete, all plans are done.
                m.status == MilestoneStatus::Complete
            }).count();
            json!({
                "name": m.name,
                "status": m.status,
                "plans_completed": completed,
                "plans_total": m.plans.len(),
            })
        }).collect::<Vec<_>>(),
        "current_plan": run.current_plan,
        "elapsed_time": elapsed,
    })))
}

// ---------------------------------------------------------------------------
// GET /runs/:id/cost
// ---------------------------------------------------------------------------

/// Live cost breakdown per milestone, including estimated vs. actual.
async fn get_run_cost(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    let milestones: Vec<serde_json::Value> = run
        .milestones
        .iter()
        .map(|m| {
            let estimated = m.estimated_cost.total;
            let actual = m.actual_cost.unwrap_or(0.0);
            let mut obj = json!({
                "name": m.name,
                "status": m.status,
                "estimated_cost": estimated,
            });
            match m.status {
                MilestoneStatus::Complete => {
                    obj["actual_cost"] = json!(actual);
                    obj["delta"] = json!(actual - estimated);
                }
                MilestoneStatus::InProgress => {
                    obj["actual_cost_so_far"] = json!(actual);
                }
                _ => {}
            }
            obj
        })
        .collect();

    let total_estimated: f64 = run.milestones.iter().map(|m| m.estimated_cost.total).sum();

    Ok(Json(json!({
        "milestones": milestones,
        "total_estimated": total_estimated,
        "total_actual": run.budget_spent,
        "remaining_deposit": run.budget_total - run.budget_spent,
    })))
}

// ---------------------------------------------------------------------------
// GET /runs/:id/events  (SSE)
// ---------------------------------------------------------------------------

/// Server-sent event stream for a running build.
///
/// Subscribes to the run's broadcast channel and forwards each `BuildEvent`
/// as a JSON SSE event. Every event includes cost headers as comment lines.
async fn run_events(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // Verify the run exists before opening the stream.
    let run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    let rx = state.event_receiver(&id);
    let db = state.db.clone();
    let run_id = id.clone();

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        match result {
            Ok(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();

                // Build the SSE event with cost metadata as a comment.
                // Fetch current cost from DB for accurate headers.
                let (run_cost, budget_remaining, pct_used) = db
                    .get_run(&run_id)
                    .ok()
                    .flatten()
                    .map(|r| {
                        let remaining = r.budget_total - r.budget_spent;
                        let pct = if r.budget_total > 0.0 {
                            ((r.budget_spent / r.budget_total) * 100.0) as u32
                        } else {
                            0
                        };
                        (r.budget_spent, remaining, pct)
                    })
                    .unwrap_or((0.0, 0.0, 0));

                let comment = format!(
                    "X-Mori-Run-Cost: {run_cost:.2}, \
                     X-Mori-Budget-Remaining: {budget_remaining:.2}, \
                     X-Mori-Budget-Pct-Used: {pct_used}"
                );

                let sse_event = Event::default().data(data).comment(comment);
                Some(Ok(sse_event))
            }
            Err(_) => {
                // Lagged behind or channel closed -- skip.
                None
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    ))
}

// ---------------------------------------------------------------------------
// POST /runs/:id/top-up
// ---------------------------------------------------------------------------

/// Add budget to a running build.
///
/// In production, the `X-Payment` header carries the signed USDC authorization.
/// For now, we accept a JSON body with the amount.
async fn top_up(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    if !matches!(run.status, RunStatus::Building | RunStatus::Paused) {
        return Err(AppError::Conflict(format!(
            "run {id} has status {:?} and cannot be topped up",
            run.status
        )));
    }

    // TODO: extract amount from X-Payment header. Placeholder: $5.00.
    let top_up_amount = 5.0;

    run.budget_total += top_up_amount;
    run.adjustments.push(Adjustment {
        adjustment_type: AdjustmentType::TopUp,
        amount: top_up_amount,
        description: "budget top-up".into(),
        plans_affected: vec![],
        timestamp: Utc::now(),
    });

    // If paused due to budget, resume.
    if run.status == RunStatus::Paused {
        run.status = RunStatus::Building;
    }

    state
        .db
        .update_run(&run)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "new_budget": run.budget_total,
        "status": if run.status == RunStatus::Building { "resuming" } else { "building" },
    })))
}

// ---------------------------------------------------------------------------
// POST /runs/:id/reduce-scope
// ---------------------------------------------------------------------------

/// Skip specified plans from the build to reduce cost.
/// Plans already completed cannot be skipped.
async fn reduce_scope(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
    Json(payload): Json<ReduceScopeRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.skip_plans.is_empty() {
        return Err(AppError::BadRequest("skip_plans must not be empty".into()));
    }

    let mut run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    if !matches!(run.status, RunStatus::Building | RunStatus::Paused) {
        return Err(AppError::Conflict(format!(
            "run {id} has status {:?} and cannot have scope reduced",
            run.status
        )));
    }

    // Mark plans as skipped and track the cost reduction.
    let mut cost_saved = 0.0;
    for milestone in &mut run.milestones {
        if milestone.status == MilestoneStatus::Complete {
            continue;
        }
        for plan_name in &payload.skip_plans {
            if milestone.plans.contains(plan_name) {
                // Approximate: divide milestone cost evenly across plans.
                let plan_count = milestone.plans.len() as f64;
                let per_plan_cost = if plan_count > 0.0 {
                    milestone.estimated_cost.total / plan_count
                } else {
                    0.0
                };
                cost_saved += per_plan_cost;
            }
        }
        // If all plans in this milestone are being skipped, mark it skipped.
        if milestone
            .plans
            .iter()
            .all(|p| payload.skip_plans.contains(p))
        {
            milestone.status = MilestoneStatus::Skipped;
        }
    }

    run.adjustments.push(Adjustment {
        adjustment_type: AdjustmentType::ScopeReduction,
        amount: -cost_saved,
        description: format!("skipped plans: {}", payload.skip_plans.join(", ")),
        plans_affected: payload.skip_plans,
        timestamp: Utc::now(),
    });

    state
        .db
        .update_run(&run)
        .map_err(|e| AppError::Internal(e.into()))?;

    let revised_cost: f64 = run
        .milestones
        .iter()
        .filter(|m| m.status != MilestoneStatus::Skipped)
        .map(|m| m.estimated_cost.total)
        .sum();

    Ok(Json(json!({
        "revised_cost": revised_cost,
        "budget_remaining": run.budget_total - run.budget_spent,
    })))
}

// ---------------------------------------------------------------------------
// POST /runs/:id/add-feature
// ---------------------------------------------------------------------------

/// Extend a running build with a new feature. Mori generates new plans,
/// inserts them into the build DAG, and continues.
async fn add_feature(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
    Json(payload): Json<AddFeatureRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.description.trim().is_empty() {
        return Err(AppError::BadRequest(
            "feature description must not be empty".into(),
        ));
    }

    let mut run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    if !matches!(run.status, RunStatus::Building | RunStatus::Paused) {
        return Err(AppError::Conflict(format!(
            "run {id} has status {:?} and cannot have features added",
            run.status
        )));
    }

    // Generate new plans from the feature description.
    let new_milestone = state
        .engine
        .plan_feature(&payload.description, payload.max_additional_cost)
        .map_err(|e| AppError::Internal(e.into()))?;

    let new_plan_names: Vec<String> = new_milestone.plans.clone();
    let additional_cost = new_milestone.estimated_cost.total;

    if additional_cost > payload.max_additional_cost {
        return Err(AppError::PaymentRequired(format!(
            "feature costs ${additional_cost:.2} but max_additional_cost is ${:.2}",
            payload.max_additional_cost
        )));
    }

    run.milestones.push(new_milestone);
    run.budget_total += additional_cost;

    run.adjustments.push(Adjustment {
        adjustment_type: AdjustmentType::FeatureAddition,
        amount: additional_cost,
        description: payload.description,
        plans_affected: new_plan_names.clone(),
        timestamp: Utc::now(),
    });

    state
        .db
        .update_run(&run)
        .map_err(|e| AppError::Internal(e.into()))?;

    let revised_total: f64 = run.milestones.iter().map(|m| m.estimated_cost.total).sum();

    Ok(Json(json!({
        "new_plans": new_plan_names,
        "revised_total": revised_total,
        "status": "replanning",
    })))
}

// ---------------------------------------------------------------------------
// DELETE /runs/:id
// ---------------------------------------------------------------------------

/// Cancel a running build. Completed milestones are settled; remaining
/// deposit is refunded.
async fn cancel_run(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mut run = state
        .db
        .get_run(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("run {id} not found")))?;

    if matches!(
        run.status,
        RunStatus::Complete | RunStatus::Cancelled | RunStatus::Settled
    ) {
        return Err(AppError::Conflict(format!(
            "run {id} has status {:?} and cannot be cancelled",
            run.status
        )));
    }

    let milestones_completed = run
        .milestones
        .iter()
        .filter(|m| m.status == MilestoneStatus::Complete)
        .count() as u32;

    let refund_amount = run.budget_total - run.budget_spent;

    run.status = RunStatus::Cancelled;
    run.finished_at = Some(Utc::now());

    // Mark in-progress milestones as skipped.
    for m in &mut run.milestones {
        if matches!(
            m.status,
            MilestoneStatus::Pending | MilestoneStatus::InProgress
        ) {
            m.status = MilestoneStatus::Skipped;
        }
    }

    state
        .db
        .update_run(&run)
        .map_err(|e| AppError::Internal(e.into()))?;

    // Clean up the event channel.
    state.event_channels.remove(&id);

    Ok(Json(json!({
        "refund_amount": refund_amount,
        "milestones_completed": milestones_completed,
    })))
}

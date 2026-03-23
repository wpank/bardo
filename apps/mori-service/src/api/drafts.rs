//! Draft endpoints.
//!
//! Drafts are conversational workspaces where PRDs take shape through
//! iterative refinement. Each interaction accumulates context and cost.

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use serde_json::json;

use crate::state::ServiceState;
use crate::types::{CreateDraftRequest, Draft, DraftMessage, IterateDraftRequest, MessageRole};

use super::auth::AppError;

/// Mount draft routes.
pub fn router() -> Router<ServiceState> {
    Router::new()
        .route("/drafts", post(create_draft))
        .route("/drafts/{id}", get(get_draft))
        .route("/drafts/{id}/iterate", post(iterate_draft))
}

// ---------------------------------------------------------------------------
// POST /drafts
// ---------------------------------------------------------------------------

/// Create a new draft from an initial user message.
///
/// In production this is x402-billed; for now we skip payment verification
/// and just create the draft.
async fn create_draft(
    State(state): State<ServiceState>,
    Json(payload): Json<CreateDraftRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("content must not be empty".into()));
    }

    let draft_id = generate_draft_id();
    let now = Utc::now();

    // Produce an initial assistant response from the proposal engine.
    let response_text = state
        .engine
        .draft_respond(&payload.content, payload.repo_url.as_deref())
        .map_err(|e| AppError::Internal(e.into()))?;

    // Estimated cost of this single interaction.
    let interaction_cost = 0.03;

    let user_msg = DraftMessage {
        role: MessageRole::User,
        content: payload.content,
        cost: 0.0,
        timestamp: now,
    };

    let assistant_msg = DraftMessage {
        role: MessageRole::Assistant,
        content: response_text.clone(),
        cost: interaction_cost,
        timestamp: now,
    };

    let draft = Draft {
        draft_id: draft_id.clone(),
        messages: vec![user_msg, assistant_msg],
        accumulated_context: response_text.clone(),
        session_cost_so_far: interaction_cost,
        repo_url: payload.repo_url,
        created_at: now,
        updated_at: now,
    };

    state
        .db
        .create_draft(&draft)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "draft_id": draft_id,
        "response": response_text,
        "session_cost_so_far": draft.session_cost_so_far,
    })))
}

// ---------------------------------------------------------------------------
// POST /drafts/:id/iterate
// ---------------------------------------------------------------------------

/// Add a user message to an existing draft and get an assistant response.
async fn iterate_draft(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
    Json(payload): Json<IterateDraftRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.message.trim().is_empty() {
        return Err(AppError::BadRequest("message must not be empty".into()));
    }

    let mut draft = state
        .db
        .get_draft(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("draft {id} not found")))?;

    let now = Utc::now();

    // Generate assistant response using accumulated context.
    let response_text = state
        .engine
        .draft_iterate(&draft.accumulated_context, &payload.message)
        .map_err(|e| AppError::Internal(e.into()))?;

    let interaction_cost = 0.03;

    draft.messages.push(DraftMessage {
        role: MessageRole::User,
        content: payload.message,
        cost: 0.0,
        timestamp: now,
    });

    draft.messages.push(DraftMessage {
        role: MessageRole::Assistant,
        content: response_text.clone(),
        cost: interaction_cost,
        timestamp: now,
    });

    draft.accumulated_context.push('\n');
    draft.accumulated_context.push_str(&response_text);
    draft.session_cost_so_far += interaction_cost;
    draft.updated_at = now;

    state
        .db
        .update_draft(&draft)
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "response": response_text,
        "session_cost_so_far": draft.session_cost_so_far,
    })))
}

// ---------------------------------------------------------------------------
// GET /drafts/:id
// ---------------------------------------------------------------------------

/// Retrieve a draft with its full message history.
async fn get_draft(
    State(state): State<ServiceState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let draft = state
        .db
        .get_draft(&id)
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound(format!("draft {id} not found")))?;

    Ok(Json(json!({
        "draft_id": draft.draft_id,
        "messages": draft.messages,
        "cost_so_far": draft.session_cost_so_far,
        "created_at": draft.created_at,
    })))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_draft_id() -> String {
    let id = uuid::Uuid::new_v4();
    format!("d-{}", &id.to_string()[..8])
}

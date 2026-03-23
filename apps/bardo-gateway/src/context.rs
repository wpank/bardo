//! Code context endpoint — serves LLM-ready context from the mori index.
//!
//! Gated behind the `context` feature. Exposes `GET /v1/context` which searches
//! the index and returns structured context blocks.

use std::path::PathBuf;
use std::sync::Mutex;

use axum::extract::{Query, State};
use axum::response::Json;
use mori_context::{ContextQuery, ContextResponse, SearchStrategy};
use mori_index::Index;
use serde::{Deserialize, Serialize};

/// Shared context state holding the mori index.
pub struct ContextState {
    index: Mutex<Index>,
}

impl ContextState {
    /// Open the index at `root`. Returns `None` if the index can't be opened.
    pub fn try_new(root: &PathBuf) -> Option<Self> {
        match Index::open(root) {
            Ok(index) => Some(Self {
                index: Mutex::new(index),
            }),
            Err(e) => {
                tracing::warn!(error = %e, "code context disabled (index open failed)");
                None
            }
        }
    }
}

/// Query parameters for `GET /v1/context`.
#[derive(Debug, Deserialize)]
pub struct ContextParams {
    /// Search query (symbol name or natural-language description).
    pub q: String,
    /// Search strategy: "keyword", "similar", or "hybrid" (default: "keyword").
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Max results (default: 10).
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Lines of source context per symbol (default: 10).
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
    /// Similarity threshold for HDC/hybrid (default: 0.5).
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    /// Output format: "json" or "markdown" (default: "json").
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_strategy() -> String { "keyword".into() }
fn default_limit() -> usize { 10 }
fn default_context_lines() -> usize { 10 }
fn default_threshold() -> f32 { 0.5 }
fn default_format() -> String { "json".into() }

/// Response wrapper.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ContextOutput {
    /// JSON response.
    Json(ContextResponse),
    /// Markdown response.
    Markdown { markdown: String },
}

/// `GET /v1/context?q=...`
pub async fn context_query(
    State(ctx): State<std::sync::Arc<ContextState>>,
    Query(params): Query<ContextParams>,
) -> Result<Json<ContextOutput>, String> {
    let strategy = match params.strategy.as_str() {
        "keyword" => SearchStrategy::Keyword,
        "similar" => SearchStrategy::Similar {
            threshold: params.threshold,
        },
        "hybrid" => SearchStrategy::Hybrid {
            threshold: params.threshold,
        },
        other => return Err(format!("unknown strategy: {other}")),
    };

    let query = ContextQuery {
        query: params.q,
        strategy,
        limit: params.limit,
        context_lines: params.context_lines,
        include_related: true,
    };

    let response = {
        let mut index = ctx.index.lock().map_err(|e| format!("lock poisoned: {e}"))?;
        mori_context::assemble(&mut index, &query).map_err(|e| format!("context error: {e}"))?
    };

    match params.format.as_str() {
        "markdown" => Ok(Json(ContextOutput::Markdown {
            markdown: response.to_markdown(),
        })),
        _ => Ok(Json(ContextOutput::Json(response))),
    }
}

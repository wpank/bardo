//! Anthropic Batch API support.
//!
//! Non-time-sensitive requests (enrichment, background analysis) can be submitted
//! via the Batch API for a 50% cost reduction. Requests are queued locally, then
//! submitted as a batch to Anthropic's `/v1/messages/batches` endpoint.
//!
//! Flow:
//! 1. Client sends `POST /v1/batch/submit` with a Messages API body
//! 2. Gateway queues the request, returns `202 Accepted` with an item ID
//! 3. Queue is flushed when it reaches a threshold, a timer fires, or manually
//! 4. Batch is submitted to Anthropic, gateway polls for completion
//! 5. Results are stored and available via `GET /v1/batch/result/{id}`
//! 6. WebSocket clients are notified when a batch completes

use std::sync::Arc;
use std::time::Instant;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::AppError;
use crate::prefix;
use crate::state::AppState;

/// How many requests to accumulate before auto-flushing.
const DEFAULT_FLUSH_THRESHOLD: usize = 50;
/// How often to auto-flush (seconds), even if threshold not reached.
const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 30;
/// How often to poll Anthropic for batch completion (seconds).
const POLL_INTERVAL_SECS: u64 = 60;

/// Manages the batch request queue, submission, and result retrieval.
pub struct BatchManager {
    http: reqwest::Client,
    api_keys: Vec<String>,
    queue: Mutex<Vec<BatchItem>>,
    /// Active batches submitted to Anthropic, keyed by batch ID.
    pub active: Arc<DashMap<String, BatchInfo>>,
    /// Completed results, keyed by custom_id (the item-level ID returned to clients).
    pub results: Arc<DashMap<String, BatchResult>>,
    flush_threshold: usize,
}

struct BatchItem {
    custom_id: String,
    body: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchInfo {
    pub batch_id: String,
    pub status: String,
    pub submitted_at_ms: u64,
    pub request_count: usize,
    pub item_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchResult {
    pub custom_id: String,
    pub batch_id: String,
    pub result: Value,
    pub completed_at_ms: u64,
}

#[derive(Deserialize)]
pub struct BatchSubmitRequest {
    #[serde(flatten)]
    pub body: Value,
}

impl BatchManager {
    pub fn new(http: reqwest::Client, api_keys: Vec<String>) -> Self {
        Self {
            http,
            api_keys,
            queue: Mutex::new(Vec::new()),
            active: Arc::new(DashMap::new()),
            results: Arc::new(DashMap::new()),
            flush_threshold: DEFAULT_FLUSH_THRESHOLD,
        }
    }

    /// Enqueue a request for batch processing. Returns the custom_id.
    pub async fn enqueue(&self, mut body: Value) -> String {
        let custom_id = format!("bardo-{}", uuid::Uuid::new_v4());

        // Apply the same preprocessing as real-time requests.
        prefix::inject_anthropic_cache_control(&mut body);

        let mut queue = self.queue.lock().await;
        queue.push(BatchItem {
            custom_id: custom_id.clone(),
            body,
        });

        // Auto-flush if threshold reached.
        if queue.len() >= self.flush_threshold {
            let items: Vec<BatchItem> = queue.drain(..).collect();
            drop(queue);
            tokio::spawn({
                let mgr = self.http.clone();
                let keys = self.api_keys.clone();
                let active = self.active.clone();
                let results = self.results.clone();
                async move {
                    if let Err(e) = submit_batch(&mgr, &keys, items, &active, &results).await {
                        tracing::error!(error = ?e, "auto-flush batch submission failed");
                    }
                }
            });
        }

        custom_id
    }

    /// Force-flush the current queue as a batch.
    pub async fn flush(&self) -> Result<Option<String>, AppError> {
        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return Ok(None);
        }
        let items: Vec<BatchItem> = queue.drain(..).collect();
        drop(queue);

        let batch_id = submit_batch(
            &self.http,
            &self.api_keys,
            items,
            &self.active,
            &self.results,
        )
        .await?;

        Ok(Some(batch_id))
    }

    /// Number of items currently queued (not yet submitted).
    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Start the background poll loop that checks active batches for completion.
    pub fn start_poll_loop(self: &Arc<Self>) {
        let mgr = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(POLL_INTERVAL_SECS));
            loop {
                interval.tick().await;
                poll_active_batches(&mgr.http, &mgr.api_keys, &mgr.active, &mgr.results).await;
            }
        });
    }

    /// Start the auto-flush timer.
    pub fn start_flush_timer(self: &Arc<Self>) {
        let mgr = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                DEFAULT_FLUSH_INTERVAL_SECS,
            ));
            loop {
                interval.tick().await;
                let queue_len = mgr.queue.lock().await.len();
                if queue_len > 0 {
                    tracing::info!(queue_len, "auto-flushing batch queue");
                    if let Err(e) = mgr.flush().await {
                        tracing::error!(error = ?e, "auto-flush failed");
                    }
                }
            }
        });
    }
}

/// Submit a batch of items to Anthropic's Batch API.
async fn submit_batch(
    http: &reqwest::Client,
    api_keys: &[String],
    items: Vec<BatchItem>,
    active: &DashMap<String, BatchInfo>,
    _results: &DashMap<String, BatchResult>,
) -> Result<String, AppError> {
    let api_key = &api_keys[0]; // Use primary key for batch submissions.

    // Build JSONL body for the batch.
    let requests: Vec<Value> = items
        .iter()
        .map(|item| {
            serde_json::json!({
                "custom_id": item.custom_id,
                "params": item.body,
            })
        })
        .collect();

    let batch_body = serde_json::json!({
        "requests": requests,
    });

    let resp = http
        .post("https://api.anthropic.com/v1/messages/batches")
        .header("x-api-key", api_key.as_str())
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "message-batches-2024-09-24")
        .header("content-type", "application/json")
        .json(&batch_body)
        .send()
        .await
        .map_err(|e| AppError::ProviderError(format!("batch submit failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::ProviderError(format!(
            "batch submit returned {status}: {body}"
        )));
    }

    let parsed: Value = resp
        .json()
        .await
        .map_err(|e| AppError::ProviderError(format!("batch submit parse failed: {e}")))?;

    let batch_id = parsed
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let item_ids: Vec<String> = items.iter().map(|i| i.custom_id.clone()).collect();
    let count = item_ids.len();

    active.insert(
        batch_id.clone(),
        BatchInfo {
            batch_id: batch_id.clone(),
            status: "in_progress".into(),
            submitted_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            request_count: count,
            item_ids,
        },
    );

    tracing::info!(
        batch_id = %batch_id,
        request_count = count,
        "batch submitted to Anthropic"
    );

    Ok(batch_id)
}

/// Poll active batches for completion and download results.
async fn poll_active_batches(
    http: &reqwest::Client,
    api_keys: &[String],
    active: &DashMap<String, BatchInfo>,
    results: &DashMap<String, BatchResult>,
) {
    let api_key = &api_keys[0];
    let batch_ids: Vec<String> = active.iter().map(|e| e.key().clone()).collect();

    for batch_id in batch_ids {
        let resp = http
            .get(format!(
                "https://api.anthropic.com/v1/messages/batches/{batch_id}"
            ))
            .header("x-api-key", api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "message-batches-2024-09-24")
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(batch_id = %batch_id, error = %e, "batch poll failed");
                continue;
            }
        };

        if !resp.status().is_success() {
            continue;
        }

        let parsed: Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };

        let status = parsed
            .get("processing_status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        if status == "ended" {
            tracing::info!(batch_id = %batch_id, "batch completed, downloading results");

            // Download results.
            let results_resp = http
                .get(format!(
                    "https://api.anthropic.com/v1/messages/batches/{batch_id}/results"
                ))
                .header("x-api-key", api_key.as_str())
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "message-batches-2024-09-24")
                .send()
                .await;

            if let Ok(rr) = results_resp {
                if let Ok(body) = rr.text().await {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    // Results come as JSONL.
                    for line in body.lines() {
                        if let Ok(entry) = serde_json::from_str::<Value>(line) {
                            let custom_id = entry
                                .get("custom_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let result = entry
                                .get("result")
                                .cloned()
                                .unwrap_or(Value::Null);

                            if !custom_id.is_empty() {
                                results.insert(
                                    custom_id.clone(),
                                    BatchResult {
                                        custom_id,
                                        batch_id: batch_id.clone(),
                                        result,
                                        completed_at_ms: now_ms,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            // Update batch status.
            if let Some(mut info) = active.get_mut(&batch_id) {
                info.status = "ended".into();
            }
            // Remove from active after a delay (keep for status queries).
            let active_clone = active.clone();
            let bid = batch_id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                active_clone.remove(&bid);
            });
        } else {
            // Update status.
            if let Some(mut info) = active.get_mut(&batch_id) {
                info.status = status.to_string();
            }
        }
    }
}

// ── HTTP Handlers ────────────────────────────────────────────────

/// `POST /v1/batch/submit` — Queue a request for batch processing.
pub async fn batch_submit(
    State(state): State<AppState>,
    Json(req): Json<BatchSubmitRequest>,
) -> Result<Response, AppError> {
    let mgr = state
        .batch_manager
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("batch API not enabled".into()))?;

    let custom_id = mgr.enqueue(req.body).await;
    let queue_len = mgr.queue_len().await;

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "batch_item_id": custom_id,
            "status": "queued",
            "queue_position": queue_len,
        })),
    )
        .into_response())
}

/// `POST /v1/batch/flush` — Force-submit the current queue.
pub async fn batch_flush(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mgr = state
        .batch_manager
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("batch API not enabled".into()))?;

    let batch_id = mgr.flush().await?;

    Ok(Json(serde_json::json!({
        "flushed": batch_id.is_some(),
        "batch_id": batch_id,
    })))
}

/// `GET /v1/batch/status` — List active batches.
pub async fn batch_status(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mgr = state
        .batch_manager
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("batch API not enabled".into()))?;

    let batches: Vec<BatchInfo> = mgr.active.iter().map(|e| e.value().clone()).collect();
    let queue_len = mgr.queue_len().await;

    Ok(Json(serde_json::json!({
        "queue_length": queue_len,
        "active_batches": batches,
        "completed_results": mgr.results.len(),
    })))
}

/// `GET /v1/batch/result/:id` — Retrieve a completed batch item result.
pub async fn batch_result(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let mgr = state
        .batch_manager
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("batch API not enabled".into()))?;

    match mgr.results.get(&id) {
        Some(entry) => Ok(Json(serde_json::json!({
            "status": "completed",
            "result": entry.value().clone(),
        }))),
        None => {
            // Check if it's still queued or in a batch.
            let in_queue = mgr
                .queue
                .lock()
                .await
                .iter()
                .any(|item| item.custom_id == id);
            let in_batch = mgr.active.iter().any(|e| e.value().item_ids.contains(&id));

            if in_queue {
                Ok(Json(serde_json::json!({"status": "queued"})))
            } else if in_batch {
                Ok(Json(serde_json::json!({"status": "processing"})))
            } else {
                Ok(Json(serde_json::json!({"status": "not_found"})))
            }
        }
    }
}

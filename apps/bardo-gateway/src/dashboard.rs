//! Dashboard WebSocket and static file serving.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::state::AppState;

/// WebSocket upgrade handler: sends initial stats snapshot, then streams events.
pub async fn ws_stats(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_stats_socket(socket, state))
}

async fn handle_stats_socket(mut socket: WebSocket, state: AppState) {
    // 1. Send initial snapshot so client gets current totals immediately.
    let mut snapshot = state.stats.to_json();
    snapshot.cache_entries = state.cache.len() as usize;
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "snapshot",
        "data": snapshot,
    })) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    // 2. Subscribe to broadcast channel and forward per-request events.
    let mut rx = state.stats_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(event) => {
                let msg = serde_json::json!({
                    "type": "event",
                    "data": event,
                });
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::debug!(skipped = n, "dashboard ws client lagged");
                let msg = serde_json::json!({ "type": "lagged", "skipped": n });
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

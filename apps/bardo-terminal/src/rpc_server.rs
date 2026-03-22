//! Minimal JSON-RPC over HTTP server for headless mode.

use std::{sync::Arc, time::Instant};

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Value,
}

impl JsonRpcResponse {
    fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result,
        }
    }
}

struct ServerState {
    start: Instant,
    action_tx: mpsc::Sender<String>,
    shutdown_tx: mpsc::Sender<()>,
}

/// JSON-RPC server that exposes terminal control methods over HTTP.
pub struct RpcServer {
    port: u16,
    action_tx: mpsc::Sender<String>,
    shutdown_tx: mpsc::Sender<()>,
}

impl RpcServer {
    /// Creates a new RPC server bound to `port`, forwarding actions and shutdown signals.
    pub fn new(port: u16, action_tx: mpsc::Sender<String>, shutdown_tx: mpsc::Sender<()>) -> Self {
        Self {
            port,
            action_tx,
            shutdown_tx,
        }
    }

    /// Starts the HTTP server, blocking until shutdown.
    pub async fn run(self) -> anyhow::Result<()> {
        let state = Arc::new(ServerState {
            start: Instant::now(),
            action_tx: self.action_tx,
            shutdown_tx: self.shutdown_tx,
        });

        let app = Router::new().route(
            "/",
            post({
                let state = Arc::clone(&state);
                move |body| handle_rpc(body, state)
            }),
        );

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        tracing::info!(port = self.port, "RPC server listening");
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_rpc(
    Json(req): Json<JsonRpcRequest>,
    state: Arc<ServerState>,
) -> Json<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "terminal.health" => {
            let uptime_ms = state.start.elapsed().as_millis() as u64;
            serde_json::json!({ "status": "ok", "uptime_ms": uptime_ms })
        }
        "terminal.snapshot" => {
            serde_json::json!({ "screen": "home", "size": [120, 40] })
        }
        "terminal.action" => {
            let action = req
                .params
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let _ = state.action_tx.send(action).await;
            serde_json::json!({ "ok": true })
        }
        "terminal.shutdown" => {
            let _ = state.shutdown_tx.send(()).await;
            serde_json::json!({ "ok": true })
        }
        other => {
            serde_json::json!({ "error": format!("unknown method: {other}") })
        }
    };

    Json(JsonRpcResponse::ok(req.id, result))
}

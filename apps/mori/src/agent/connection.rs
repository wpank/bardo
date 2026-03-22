use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

use super::events::AgentEvent;
use super::protocol::*;
use super::roles::AgentRole;

pub struct AppServerConnection {
    role: AgentRole,
    child: Child,
    stdin: tokio::io::BufWriter<tokio::process::ChildStdin>,
    next_id: AtomicU64,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    /// Pending response channels keyed by request id
    pending: std::collections::HashMap<u64, oneshot::Sender<Value>>,
    /// Receiver for responses routed from the reader task
    response_rx: mpsc::UnboundedReceiver<(u64, Value)>,
    /// Current thread ID for this agent
    current_thread_id: Option<String>,
    /// Instance ID injected into all emitted AgentEvents.
    instance_id: Arc<Mutex<Option<String>>>,
    /// Handles for reader tasks so we can clean up on kill
    reader_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl AppServerConnection {
    /// Spawn a new `codex app-server` process and start reading events.
    #[instrument(skip_all, fields(role = ?role))]
    pub async fn spawn(
        role: AgentRole,
        working_dir: &PathBuf,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        reasoning_effort: &str,
        instance_id: Option<String>,
        fast_mode: bool,
    ) -> Result<Self> {
        let effort_arg = format!("model_reasoning_effort=\"{reasoning_effort}\"");
        let mut cmd = Command::new("codex");
        cmd.args(["app-server", "-c", &effort_arg]);
        if fast_mode {
            cmd.args(["-c", "service_tier=\"fast\""]);
        }
        cmd.current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Ensure agents use sccache and disable incremental for cross-branch cache hits.
        // Incremental compilation marks outputs non-cacheable by sccache; disabling it
        // lets sccache cache Rust compilations across branch switches (which agents do constantly).
        cmd.env("CARGO_INCREMENTAL", "0");
        // Propagate sccache wrapper if configured in parent env
        if let Ok(wrapper) = std::env::var("RUSTC_WRAPPER") {
            cmd.env("RUSTC_WRAPPER", &wrapper);
        }
        // Help sccache share cache across worktrees by normalizing base paths
        cmd.env("SCCACHE_BASEDIRS", working_dir);
        // Gateway: route through mori inference gateway if configured
        if let Ok(url) = std::env::var("OPENAI_BASE_URL") {
            cmd.env("OPENAI_BASE_URL", &url);
        }

        tracing::info!(
            "Spawning codex app-server [{role}] effort={reasoning_effort} fast={fast_mode} cwd={}",
            working_dir.display()
        );

        cmd.kill_on_drop(true);
        let mut child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn codex app-server for {role} (cwd={})",
                working_dir.display()
            )
        })?;

        let stdin = child.stdin.take().context("No stdin on child")?;
        let stdout = child.stdout.take().context("No stdout on child")?;

        // Shared instance_id: wrapped in Arc<Mutex> so the reader task sees
        // updates when this connection is recycled from the warm pool.
        let shared_iid = Arc::new(Mutex::new(instance_id.clone()));

        // Spawn stderr reader: log at error level AND emit as AgentEvent so the TUI shows it.
        let mut reader_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
        if let Some(stderr) = child.stderr.take() {
            let stderr_tx = event_tx.clone();
            let stderr_role = role;
            let stderr_iid = Arc::clone(&shared_iid);
            reader_tasks.push(tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        tracing::error!("[codex/{stderr_role}] {line}");
                        let _ = stderr_tx.send(AgentEvent::Error {
                            role: stderr_role,
                            instance: stderr_iid.lock().unwrap().clone(),
                            error: format!("[codex stderr] {line}"),
                        });
                    }
                }
            }));
        }

        let stdin = tokio::io::BufWriter::new(stdin);

        // Channel for routing responses back to the main connection
        let (resp_tx, resp_rx) = mpsc::unbounded_channel::<(u64, Value)>();

        // Spawn stdout reader task
        let tx = event_tx.clone();
        let reader_role = role;
        let reader_iid = Arc::clone(&shared_iid);
        reader_tasks.push(tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<RawServerMessage>(&line) {
                    Ok(msg) => {
                        if msg.is_server_request() {
                            let method = msg.method.as_deref().unwrap_or("");
                            let params = msg.params.as_ref();
                            let msg_id = msg.id.as_ref();
                            if let Some(event) =
                                parse_notification(reader_role, method, params, msg_id)
                            {
                                let _ = tx
                                    .send(event.with_instance(reader_iid.lock().unwrap().clone()));
                            }
                        } else if msg.is_response() {
                            let id = msg.numeric_id().unwrap_or(0);
                            let val = if let Some(err) = msg.error {
                                // Include data.message if present for better error diagnostics
                                let error_msg = if let Some(data) = &err.data {
                                    if let Some(msg_val) =
                                        data.get("message").and_then(|v| v.as_str())
                                    {
                                        format!("{} ({})", err.message, msg_val)
                                    } else {
                                        err.message.clone()
                                    }
                                } else {
                                    err.message.clone()
                                };
                                serde_json::json!({"error": error_msg})
                            } else {
                                msg.result.unwrap_or(Value::Null)
                            };
                            let _ = resp_tx.send((id, val));
                        } else if msg.is_notification() {
                            let method = msg.method.as_deref().unwrap_or("");
                            let params = msg.params.as_ref();
                            if let Some(event) =
                                parse_notification(reader_role, method, params, None)
                            {
                                let _ = tx
                                    .send(event.with_instance(reader_iid.lock().unwrap().clone()));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AgentEvent::Error {
                            role: reader_role,
                            instance: reader_iid.lock().unwrap().clone(),
                            error: format!("Parse error: {e}: {}", {
                                let n = line.len().min(200);
                                let n = (0..=n)
                                    .rev()
                                    .find(|&i| line.is_char_boundary(i))
                                    .unwrap_or(0);
                                &line[..n]
                            }),
                        });
                    }
                }
            }
            let _ = tx.send(AgentEvent::Exited {
                role: reader_role,
                instance: reader_iid.lock().unwrap().clone(),
                exit_code: None,
            });
        }));

        Ok(Self {
            role,
            child,
            stdin,
            next_id: AtomicU64::new(1),
            event_tx,
            pending: std::collections::HashMap::new(),
            response_rx: resp_rx,
            current_thread_id: None,
            instance_id: shared_iid,
            reader_tasks,
        })
    }

    /// Send the initialize handshake.
    pub async fn initialize(&mut self, _working_dir: &str) -> Result<()> {
        let params = InitializeParams {
            client_info: ClientInfo {
                name: "bardo-ctl".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        let id = self
            .send_request("initialize", Some(serde_json::to_value(params)?))
            .await?;
        // Drain the response
        self.recv_response(id).await?;
        Ok(())
    }

    /// Create a new thread and store its ID. Retries up to 3 times on failure.
    pub async fn create_thread(&mut self) -> Result<String> {
        let mut last_err = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                // Brief delay before retry to let app-server stabilize
                tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64)).await;
            }
            let id = self
                .send_request("thread/start", Some(serde_json::json!({})))
                .await?;
            let resp = self.recv_response(id).await?;

            // Extract thread ID from response: {"thread":{"id":"...",...},...}
            if let Some(thread_id) = resp
                .get("thread")
                .and_then(|t| t.get("id"))
                .and_then(|id| id.as_str())
                .map(String::from)
            {
                self.current_thread_id = Some(thread_id.clone());
                return Ok(thread_id);
            }

            // Check for error in response
            last_err = if let Some(err) = resp.get("error") {
                format!("thread/start error: {err}")
            } else {
                format!(
                    "No thread ID in response: {}",
                    resp.to_string().chars().take(200).collect::<String>()
                )
            };
            let _ = self.event_tx.send(AgentEvent::Error {
                role: self.role,
                instance: self.instance_id.lock().unwrap().clone(),
                error: format!("thread/start attempt {}: {last_err}", attempt + 1),
            });
        }
        anyhow::bail!("Failed to create thread after 3 attempts: {last_err}")
    }

    /// Start a new turn with the given message.
    /// Waits for the turn/start response with a 30s timeout to detect failures early.
    pub async fn turn_start(&mut self, message: &str, model: Option<&str>) -> Result<()> {
        // B5: Validate process liveness before sending
        if let Some(status) = self.child.try_wait()? {
            bail!(
                "codex app-server for {} already exited with {status}",
                self.role
            );
        }

        let thread_id = match &self.current_thread_id {
            Some(id) => id.clone(),
            None => self.create_thread().await?,
        };

        let params = TurnStartParams {
            input: vec![InputItem::Text {
                text: message.to_string(),
            }],
            thread_id,
            model: model.map(String::from),
        };
        let id = self
            .send_request("turn/start", Some(serde_json::to_value(params)?))
            .await?;
        // Wait for acknowledgement with 30s timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), self.recv_response(id)).await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => bail!("turn/start timed out after 30s for {}", self.role),
        }
    }

    /// Interrupt the current turn.
    pub async fn turn_interrupt(&mut self) -> Result<()> {
        // `params` must be present (even if empty) — codex app-server rejects
        // requests where the field is absent entirely.
        self.send_request("turn/interrupt", Some(serde_json::json!({})))
            .await?;
        Ok(())
    }

    /// Respond to an approval request using JSON-RPC response format.
    ///
    /// `approval_id` must be the raw JSON-RPC `id` value from the server-initiated
    /// request, echoed back verbatim (type preserved: string stays string, number
    /// stays number). Using the wrong id causes "Invalid request" errors on the
    /// codex side.
    pub async fn respond_approval(
        &mut self,
        approval_id: &serde_json::Value,
        approved: bool,
    ) -> Result<()> {
        let decision = if approved { "accept" } else { "decline" };
        // Echo the original JSON-RPC request id exactly — codex matches by id.
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": approval_id,
            "result": { "decision": decision }
        });
        let mut json = serde_json::to_string(&resp)?;
        json.push('\n');
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Set the thread ID (for resuming across iterations).
    pub fn set_thread_id(&mut self, thread_id: Option<String>) {
        self.current_thread_id = thread_id;
    }

    /// Get the current thread ID.
    pub fn thread_id(&self) -> Option<&str> {
        self.current_thread_id.as_deref()
    }

    /// Update the instance ID on a recycled warm-pool connection.
    /// The reader task holds an Arc clone and will see this new value immediately.
    pub fn update_instance_id(&self, new_id: Option<String>) {
        *self.instance_id.lock().unwrap() = new_id;
    }

    /// Kill the child process with graceful shutdown + SIGKILL fallback.
    /// Also awaits reader tasks to prevent leaked background tokio tasks.
    pub async fn kill(&mut self) -> Result<()> {
        tracing::debug!(role = ?self.role, "[codex] kill() called");
        // Graceful: close stdin to signal EOF
        drop(self.stdin.get_mut());

        // Wait up to 3s for graceful exit
        let exited =
            tokio::time::timeout(std::time::Duration::from_secs(3), self.child.wait()).await;

        if exited.is_err() || exited.as_ref().is_ok_and(|r| r.is_err()) {
            // SIGKILL fallback
            let _ = self.child.kill().await;
            let _ =
                tokio::time::timeout(std::time::Duration::from_secs(2), self.child.wait()).await;
        }

        // Clean up reader tasks with 5s timeout
        for handle in self.reader_tasks.drain(..) {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        }

        Ok(())
    }

    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<u64> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest::new(id, method, params);
        let mut json = serde_json::to_string(&req)?;
        json.push('\n');
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(id)
    }

    /// Wait for the response matching `expected_id`, discarding any stale
    /// responses for other IDs (e.g. error replies to fire-and-forget calls).
    async fn recv_response(&mut self, expected_id: u64) -> Result<Value> {
        loop {
            match self.response_rx.recv().await {
                Some((id, val)) => {
                    if id != expected_id {
                        // Stale response for a different request — discard and keep waiting.
                        tracing::debug!(
                            "discarding stale response id={id} (waiting for {expected_id})"
                        );
                        continue;
                    }
                    if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
                        tracing::error!(
                            "[codex/{}] server error on request {expected_id}: {err}",
                            self.role
                        );
                        bail!("codex app-server error for {}: {err}", self.role);
                    }
                    return Ok(val);
                }
                None => {
                    tracing::error!(
                        "[codex/{}] response channel closed (codex process likely exited) — \
                        check that the configured model is supported by `codex app-server`",
                        self.role
                    );
                    bail!(
                        "codex app-server for {} exited without responding — \
                        verify the model is supported by `codex app-server` (Cursor models like \
                        composer-2 are not valid here)",
                        self.role
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CursorAcpConnection
// ---------------------------------------------------------------------------

pub struct CursorAcpConnection {
    role: AgentRole,
    child: Child,
    stdin: tokio::io::BufWriter<tokio::process::ChildStdin>,
    next_id: AtomicU64,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    response_rx: mpsc::UnboundedReceiver<(u64, Value)>,
    current_session_id: Option<String>,
    instance_id: Arc<Mutex<Option<String>>>,
    working_dir: PathBuf,
    reader_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl CursorAcpConnection {
    /// Spawn a new `agent --force [--model <slug>] acp` process and start reading events.
    ///
    /// `--force` = yolo mode (no per-tool approval gates).
    /// `acp`     = start the Agent Communication Protocol JSON-RPC server over stdio.
    /// `--model` = optional model override (defaults to `auto` in cursor config).
    /// Note: `--mode` is NOT passed here — mode is set per-session via `session/new`.
    #[instrument(skip_all, fields(role = ?role))]
    pub async fn spawn(
        role: AgentRole,
        working_dir: &PathBuf,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        model: Option<&str>,
        instance_id: Option<String>,
    ) -> Result<Self> {
        let model_str = model.unwrap_or("auto");
        tracing::info!(
            "Spawning cursor agent [{role}] model={model_str} cwd={}",
            working_dir.display()
        );
        let mut cmd = Command::new("agent");
        cmd.arg("--force");
        // Disable character-by-character streaming to reduce CPU load (100+ msg/sec -> 1-2 msg/turn)
        cmd.args(["--output-format", "json"]);
        if let Some(slug) = model {
            cmd.args(["--model", slug]);
        }
        cmd.arg("acp");
        cmd.current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd.env("CARGO_INCREMENTAL", "0");
        if let Ok(wrapper) = std::env::var("RUSTC_WRAPPER") {
            cmd.env("RUSTC_WRAPPER", &wrapper);
        }
        cmd.env("SCCACHE_BASEDIRS", working_dir);
        // Gateway: route through mori inference gateway if configured
        if let Ok(url) = std::env::var("OPENAI_BASE_URL") {
            cmd.env("OPENAI_BASE_URL", &url);
        }

        cmd.kill_on_drop(true);
        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn cursor agent for {role}"))?;

        let stdin = child.stdin.take().context("No stdin on child")?;
        let stdout = child.stdout.take().context("No stdout on child")?;

        let shared_iid = Arc::new(Mutex::new(instance_id.clone()));

        let mut reader_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
        let had_stderr = Arc::new(std::sync::atomic::AtomicBool::new(false));
        if let Some(stderr) = child.stderr.take() {
            let stderr_tx = event_tx.clone();
            let stderr_role = role;
            let stderr_iid = Arc::clone(&shared_iid);
            let had_stderr_clone = Arc::clone(&had_stderr);
            reader_tasks.push(tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        had_stderr_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        tracing::error!("[cursor/{stderr_role}] stderr: {line}");
                        let _ = stderr_tx.send(AgentEvent::Error {
                            role: stderr_role,
                            instance: stderr_iid.lock().unwrap().clone(),
                            error: format!("[cursor stderr] {line}"),
                        });
                    }
                }
            }));
        }

        let stdin = tokio::io::BufWriter::new(stdin);
        let (resp_tx, resp_rx) = mpsc::unbounded_channel::<(u64, Value)>();

        let tx = event_tx.clone();
        let reader_role = role;
        let reader_iid = Arc::clone(&shared_iid);
        let had_stderr_for_reader = Arc::clone(&had_stderr);
        reader_tasks.push(tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut had_stdout = false;
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                had_stdout = true;
                if line.contains("\"agent_thought_chunk\"") {
                    tracing::trace!("[cursor/{reader_role}] ← {line}");
                } else {
                    tracing::info!("[cursor/{reader_role}] ← {line}");
                }
                match serde_json::from_str::<RawServerMessage>(&line) {
                    Ok(msg) => {
                        if msg.is_server_request() {
                            let method = msg.method.as_deref().unwrap_or("");
                            let params = msg.params.as_ref();
                            let msg_id = msg.id.as_ref();
                            if let Some(event) = parse_cursor_notification(reader_role, method, params, msg_id) {
                                let _ = tx.send(event.with_instance(reader_iid.lock().unwrap().clone()));
                            }
                        } else if msg.is_response() {
                            let id = msg.numeric_id().unwrap_or(0);
                            // Turn completion: session/prompt response contains {"stopReason":"end_turn"}.
                            // This is the only completion signal in ACP — there is no done notification.
                            if let Some(stop) = msg.result.as_ref()
                                .and_then(|r| r.get("stopReason"))
                                .and_then(|s| s.as_str())
                            {
                                tracing::info!("[cursor/{reader_role}] turn completed (stopReason={stop}) id={id}");
                                let iid = reader_iid.lock().unwrap().clone();
                                let _ = tx.send(AgentEvent::TurnCompleted {
                                    role: reader_role,
                                    instance: iid,
                                    thread_id: None,
                                });
                            }
                            let val = if let Some(err) = msg.error {
                                // Include data.message if present for better error diagnostics
                                let error_msg = if let Some(data) = &err.data {
                                    if let Some(msg_val) = data.get("message").and_then(|v| v.as_str()) {
                                        format!("{} ({})", err.message, msg_val)
                                    } else {
                                        err.message.clone()
                                    }
                                } else {
                                    err.message.clone()
                                };
                                serde_json::json!({"error": error_msg})
                            } else {
                                msg.result.unwrap_or(Value::Null)
                            };
                            let _ = resp_tx.send((id, val));
                        } else if msg.is_notification() {
                            let method = msg.method.as_deref().unwrap_or("");
                            let params = msg.params.as_ref();
                            if let Some(event) = parse_cursor_notification(reader_role, method, params, None) {
                                let _ = tx.send(event.with_instance(reader_iid.lock().unwrap().clone()));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AgentEvent::Error {
                            role: reader_role,
                            instance: reader_iid.lock().unwrap().clone(),
                            error: format!("Cursor parse error: {e}: {}", {
                                let n = line.len().min(200);
                                let n = (0..=n).rev().find(|&i| line.is_char_boundary(i)).unwrap_or(0);
                                &line[..n]
                            }),
                        });
                    }
                }
            }
            // If the process exited with no output at all, emit a diagnostic hint.
            if !had_stdout && !had_stderr_for_reader.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = tx.send(AgentEvent::Error {
                    role: reader_role,
                    instance: reader_iid.lock().unwrap().clone(),
                    error: format!(
                        "[cursor] agent process exited with no output — verify `agent` binary is in PATH and supports `--force acp` mode"
                    ),
                });
            }
            let _ = tx.send(AgentEvent::Exited {
                role: reader_role,
                instance: reader_iid.lock().unwrap().clone(),
                exit_code: None,
            });
        }));

        Ok(Self {
            role,
            child,
            stdin,
            next_id: AtomicU64::new(1),
            event_tx,
            response_rx: resp_rx,
            current_session_id: None,
            instance_id: shared_iid,
            working_dir: working_dir.clone(),
            reader_tasks,
        })
    }

    /// Send the ACP initialize handshake and create an initial session.
    pub async fn initialize(&mut self, working_dir: &str) -> Result<()> {
        self.working_dir = PathBuf::from(working_dir);
        let params = super::protocol::CursorInitializeParams {
            // ACP requires integer 1, not a date string — sends {"protocolVersion":1}
            protocol_version: 1,
            client_info: ClientInfo {
                name: "bardo-ctl".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            client_capabilities: serde_json::json!({}),
        };
        let id = self
            .send_request("initialize", Some(serde_json::to_value(params)?))
            .await?;
        // Cursor agent cold start (auth checks, model load) can take 60-90s on first run.
        // Without a timeout bardo-ctl hangs indefinitely and the TUI appears frozen.
        let resp = tokio::time::timeout(
            std::time::Duration::from_secs(90),
            self.recv_response(id),
        )
        .await
        .map_err(|_| anyhow::anyhow!(
            "cursor `initialize` timed out after 90s — is `agent --force acp` in PATH and responding?"
        ))??;
        tracing::info!("[cursor/{}] initialize response: {}", self.role, resp);
        tokio::time::timeout(std::time::Duration::from_secs(60), self.create_session())
            .await
            .map_err(|_| anyhow::anyhow!("cursor `session/new` timed out after 60s"))??;
        Ok(())
    }

    async fn create_session(&mut self) -> Result<String> {
        let cwd = self.working_dir.to_string_lossy().to_string();
        // Use "." when cwd is empty (warm-pool reuse before a real working dir is set).
        // The process's actual cwd is correct; this is just the protocol field.
        let cwd = if cwd.is_empty() { ".".to_string() } else { cwd };
        let params = super::protocol::CursorSessionNewParams {
            cwd,
            mode: "agent", // Composer 2 — full tool access
            // Required by ACP; omitting causes {"error":{"code":-32603}}
            mcp_servers: vec![],
        };
        let id = self
            .send_request("session/new", Some(serde_json::to_value(params)?))
            .await?;
        let resp = self.recv_response(id).await?;
        tracing::info!("[cursor/{}] session/new response: {}", self.role, resp);
        let session_id = resp
            .get("sessionId")
            .or_else(|| resp.get("session_id"))
            .or_else(|| resp.get("id"))
            .and_then(|s| s.as_str())
            .map(String::from)
            .ok_or_else(|| {
                anyhow::anyhow!("No sessionId in Cursor session/new response: {}", resp)
            })?;
        self.current_session_id = Some(session_id.clone());
        Ok(session_id)
    }

    /// Start a new turn — reuses existing session or creates a fresh one.
    /// Validates process is alive first.
    pub async fn turn_start(&mut self, message: &str, _model: Option<&str>) -> Result<()> {
        // B5: Validate process liveness before sending
        if let Some(status) = self.child.try_wait()? {
            bail!(
                "cursor agent for {} already exited with {status}",
                self.role
            );
        }

        let session_id = match &self.current_session_id {
            Some(id) => id.clone(),
            None => self.create_session().await?,
        };
        tracing::info!(
            "[cursor/{}] starting turn for session {}",
            self.role,
            session_id
        );
        let params = super::protocol::CursorPromptParams {
            session_id,
            prompt: vec![super::protocol::CursorPromptItem {
                kind: "text",
                text: message.to_string(),
            }],
        };
        // Fire-and-forget: the session/prompt response comes after the agent finishes,
        // with {"stopReason":"end_turn"}. The reader task detects that and emits TurnCompleted.
        // Content streams in via session/update notifications (agent_message_chunk).
        self.send_request("session/prompt", Some(serde_json::to_value(params)?))
            .await?;
        Ok(())
    }

    /// No-op — Cursor ACP with --force has no approval gates.
    pub async fn turn_interrupt(&mut self) -> Result<()> {
        tracing::debug!("turn_interrupt is a no-op for Cursor ACP");
        Ok(())
    }

    /// No-op — --force disables approval gates entirely.
    pub async fn respond_approval(
        &mut self,
        _approval_id: &serde_json::Value,
        _approved: bool,
    ) -> Result<()> {
        Ok(())
    }

    /// Map thread_id to session_id for parity with Codex.
    pub fn set_thread_id(&mut self, thread_id: Option<String>) {
        self.current_session_id = thread_id;
    }

    pub fn thread_id(&self) -> Option<&str> {
        self.current_session_id.as_deref()
    }

    pub fn update_instance_id(&self, new_id: Option<String>) {
        *self.instance_id.lock().unwrap() = new_id;
    }

    pub async fn kill(&mut self) -> Result<()> {
        tracing::debug!(role = ?self.role, "[cursor] kill() called");
        // Graceful: close stdin
        drop(self.stdin.get_mut());
        let exited =
            tokio::time::timeout(std::time::Duration::from_secs(3), self.child.wait()).await;
        if exited.is_err() || exited.as_ref().is_ok_and(|r| r.is_err()) {
            let _ = self.child.kill().await;
            let _ =
                tokio::time::timeout(std::time::Duration::from_secs(2), self.child.wait()).await;
        }
        for handle in self.reader_tasks.drain(..) {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        }
        Ok(())
    }

    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<u64> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest::new(id, method, params);
        let mut json = serde_json::to_string(&req)?;
        // Log full JSON for protocol messages, truncate for large prompts
        let log_json = if json.len() > 500 {
            &json[..500]
        } else {
            &json
        };
        tracing::info!("[cursor/{}] → {}", self.role, log_json);
        json.push('\n');
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(id)
    }

    async fn recv_response(&mut self, expected_id: u64) -> Result<Value> {
        loop {
            match self.response_rx.recv().await {
                Some((id, val)) => {
                    if id != expected_id {
                        tracing::debug!(
                            "cursor: discarding stale response id={id} (waiting for {expected_id})"
                        );
                        continue;
                    }
                    tracing::info!("[cursor/{}] ← response id={id}: {}", self.role, val);
                    if val.get("error").is_some() {
                        tracing::error!(
                            "[cursor/{}] server error on request {expected_id}: {val}",
                            self.role
                        );
                        bail!("cursor agent error for {}: {val}", self.role);
                    }
                    return Ok(val);
                }
                None => {
                    tracing::error!(
                        "[cursor/{}] response channel closed (cursor agent process likely exited) — \
                        check `agent --force acp` is available and `.cursor/cli.json` is valid",
                        self.role
                    );
                    bail!(
                        "cursor agent for {} exited without responding — \
                        verify `agent` binary supports `--force acp` mode",
                        self.role
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ClaudeConnection — per-turn claude CLI subprocess
// ---------------------------------------------------------------------------

pub struct ClaudeConnection {
    role: AgentRole,
    working_dir: PathBuf,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    /// Session ID from the last `result` event — used for `--resume` on next turn.
    current_session_id: Option<String>,
    /// The child process for the currently running turn (None between turns).
    active_child: Option<tokio::process::Child>,
    instance_id: Arc<Mutex<Option<String>>>,
    /// Reasoning effort level for --effort flag (e.g. "high", "medium", "low")
    effort: String,
}

impl ClaudeConnection {
    /// No persistent server — just initialise state.
    #[instrument(skip_all, fields(role = ?role))]
    pub async fn spawn(
        role: AgentRole,
        working_dir: &PathBuf,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        _model: Option<&str>,
        instance_id: Option<String>,
        effort: Option<&str>,
    ) -> Result<Self> {
        Ok(Self {
            role,
            working_dir: working_dir.clone(),
            event_tx,
            current_session_id: None,
            active_child: None,
            instance_id: Arc::new(Mutex::new(instance_id)),
            effort: effort.unwrap_or("max").to_string(),
        })
    }

    /// No-op — no persistent server to handshake.
    pub async fn initialize(&mut self, working_dir: &str) -> Result<()> {
        self.working_dir = PathBuf::from(working_dir);
        Ok(())
    }

    /// Spawn a fresh `claude` subprocess for this turn.
    pub async fn turn_start(&mut self, message: &str, model: Option<&str>) -> Result<()> {
        // Kill any still-running child from a previous turn.
        if self.active_child.is_some() {
            self.turn_interrupt().await?;
        }

        let model_slug = model.unwrap_or("claude-opus-4-6");
        let effort_label = &self.effort;
        let mut cmd = Command::new("claude");
        cmd.arg("--print")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--model")
            .arg(model_slug)
            .arg("--effort")
            .arg(effort_label)
            .arg("--dangerously-skip-permissions");

        // MCP context server: inject --mcp-config if the config file exists.
        // The config points at the mori binary which serves search_code,
        // get_symbol_context, find_similar_patterns, etc. over stdio.
        if let Ok(mori_config) = std::env::var("MORI_MCP_CONFIG") {
            if std::path::Path::new(&mori_config).exists() {
                cmd.arg("--mcp-config").arg(&mori_config);
            }
        } else {
            // Auto-detect: walk up from working_dir to find .mori/mcp-config.json
            // (worktrees are subdirs of the main repo, so we may need to go up)
            let mut search = Some(self.working_dir.as_path());
            while let Some(dir) = search {
                let candidate = dir.join(".mori/mcp-config.json");
                if candidate.exists() {
                    cmd.arg("--mcp-config").arg(&candidate);
                    break;
                }
                search = dir.parent();
            }
        }

        if let Some(ref sid) = self.current_session_id {
            cmd.arg("--resume").arg(sid);
        }

        cmd.current_dir(&self.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd.env("CARGO_INCREMENTAL", "0");
        if let Ok(wrapper) = std::env::var("RUSTC_WRAPPER") {
            cmd.env("RUSTC_WRAPPER", &wrapper);
        }
        cmd.env("SCCACHE_BASEDIRS", &self.working_dir);
        // Gateway: when ANTHROPIC_BASE_URL is set, route through the gateway.
        // In that case, use BARDO_GATEWAY_API_KEY as the agent's API key
        // (the gateway authenticates with Anthropic using the real key internally).
        if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
            cmd.env("ANTHROPIC_BASE_URL", &url);
            if let Ok(gw_key) = std::env::var("BARDO_GATEWAY_API_KEY") {
                cmd.env("ANTHROPIC_API_KEY", gw_key);
            }
        } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            // No gateway: pass the real API key directly
            cmd.env("ANTHROPIC_API_KEY", key);
        }

        tracing::info!(
            role = %self.role,
            model = model_slug,
            effort = %effort_label,
            session = ?self.current_session_id,
            cwd = %self.working_dir.display(),
            "Spawning claude CLI turn"
        );

        cmd.kill_on_drop(true);
        let mut child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn claude for {} (is `claude` in PATH?)",
                self.role
            )
        })?;

        // Write prompt to stdin then close it (EOF signals end of input to the CLI).
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(message.as_bytes()).await?;
            // stdin dropped here → EOF
        }

        let stdout = child.stdout.take().context("No stdout on claude child")?;

        // Stderr passthrough → AgentEvent::Error
        if let Some(stderr) = child.stderr.take() {
            let stderr_tx = self.event_tx.clone();
            let stderr_role = self.role;
            let stderr_iid = Arc::clone(&self.instance_id);
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        tracing::error!("[claude/{stderr_role}] stderr: {line}");
                        let _ = stderr_tx.send(AgentEvent::Error {
                            role: stderr_role,
                            instance: stderr_iid.lock().unwrap().clone(),
                            error: format!("[claude stderr] {line}"),
                        });
                    }
                }
            });
        }

        // Stdout reader — parses stream-json lines.
        let tx = self.event_tx.clone();
        let reader_role = self.role;
        let reader_iid = Arc::clone(&self.instance_id);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let instance = reader_iid.lock().unwrap().clone();
                match serde_json::from_str::<super::protocol::ClaudeStreamEvent>(&line) {
                    Ok(event) => {
                        parse_claude_event(reader_role, instance, event, &tx);
                    }
                    Err(e) => {
                        let n = line.len().min(200);
                        let n = (0..=n)
                            .rev()
                            .find(|&i| line.is_char_boundary(i))
                            .unwrap_or(0);
                        let _ = tx.send(AgentEvent::Error {
                            role: reader_role,
                            instance,
                            error: format!("claude parse error: {e} — raw: {}", &line[..n]),
                        });
                    }
                }
            }
            // stdout closed → turn is done (or process exited early)
            let instance = reader_iid.lock().unwrap().clone();
            let _ = tx.send(AgentEvent::Exited {
                role: reader_role,
                instance,
                exit_code: None,
            });
        });

        self.active_child = Some(child);
        Ok(())
    }

    /// Kill the currently running turn's subprocess.
    pub async fn turn_interrupt(&mut self) -> Result<()> {
        if let Some(mut child) = self.active_child.take() {
            child.kill().await.ok();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(3), child.wait()).await;
        }
        Ok(())
    }

    /// No-op — `--dangerously-skip-permissions` means no approval gates.
    pub async fn respond_approval(
        &mut self,
        _id: &serde_json::Value,
        _approved: bool,
    ) -> Result<()> {
        Ok(())
    }

    /// `thread_id` maps to Claude session_id (used with `--resume`).
    pub fn set_thread_id(&mut self, id: Option<String>) {
        self.current_session_id = id;
    }

    pub fn thread_id(&self) -> Option<&str> {
        self.current_session_id.as_deref()
    }

    pub fn update_instance_id(&self, new_id: Option<String>) {
        *self.instance_id.lock().unwrap() = new_id;
    }

    pub async fn kill(&mut self) -> Result<()> {
        tracing::debug!(role = ?self.role, "[claude] kill() called");
        self.turn_interrupt().await
    }
}

// ---------------------------------------------------------------------------
// AgentConnection — unified enum over all backends
// ---------------------------------------------------------------------------

pub enum AgentConnection {
    Codex(AppServerConnection),
    Cursor(CursorAcpConnection),
    Claude(ClaudeConnection),
}

impl AgentConnection {
    pub async fn turn_start(&mut self, message: &str, model: Option<&str>) -> Result<()> {
        match self {
            Self::Codex(c) => c.turn_start(message, model).await,
            Self::Cursor(c) => c.turn_start(message, model).await,
            Self::Claude(c) => c.turn_start(message, model).await,
        }
    }

    pub async fn turn_interrupt(&mut self) -> Result<()> {
        match self {
            Self::Codex(c) => c.turn_interrupt().await,
            Self::Cursor(c) => c.turn_interrupt().await,
            Self::Claude(c) => c.turn_interrupt().await,
        }
    }

    pub async fn respond_approval(
        &mut self,
        approval_id: &serde_json::Value,
        approved: bool,
    ) -> Result<()> {
        match self {
            Self::Codex(c) => c.respond_approval(approval_id, approved).await,
            Self::Cursor(c) => c.respond_approval(approval_id, approved).await,
            Self::Claude(c) => c.respond_approval(approval_id, approved).await,
        }
    }

    pub fn set_thread_id(&mut self, thread_id: Option<String>) {
        match self {
            Self::Codex(c) => c.set_thread_id(thread_id),
            Self::Cursor(c) => c.set_thread_id(thread_id),
            Self::Claude(c) => c.set_thread_id(thread_id),
        }
    }

    pub fn thread_id(&self) -> Option<&str> {
        match self {
            Self::Codex(c) => c.thread_id(),
            Self::Cursor(c) => c.thread_id(),
            Self::Claude(c) => c.thread_id(),
        }
    }

    pub fn update_instance_id(&self, new_id: Option<String>) {
        match self {
            Self::Codex(c) => c.update_instance_id(new_id),
            Self::Cursor(c) => c.update_instance_id(new_id),
            Self::Claude(c) => c.update_instance_id(new_id),
        }
    }

    pub async fn kill(&mut self) -> Result<()> {
        match self {
            Self::Codex(c) => c.kill().await,
            Self::Cursor(c) => c.kill().await,
            Self::Claude(c) => c.kill().await,
        }
    }
}

// ---------------------------------------------------------------------------
// Codex parse_notification (existing)
// ---------------------------------------------------------------------------

/// Parse a notification (or server-initiated request) from the app-server into an AgentEvent.
///
/// `msg_id` is the top-level JSON-RPC `id` from the message, if present.
/// For server-initiated requests (approval requests), this is what must be
/// echoed back in the response — NOT anything inside `params`.
fn parse_notification(
    role: AgentRole,
    method: &str,
    params: Option<&Value>,
    msg_id: Option<&Value>,
) -> Option<AgentEvent> {
    match method {
        // Text output from the agent
        "item/agentMessage/delta" => {
            let content = params
                .and_then(|p| p.get("delta"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            if content.is_empty() {
                return None;
            }
            Some(AgentEvent::MessageDelta {
                role,
                instance: None,
                content,
            })
        }
        // Turn completed
        "turn/completed" | "thread/updated" => {
            // thread/updated with status idle means turn is done
            if method == "thread/updated" {
                let status = params
                    .and_then(|p| p.get("thread"))
                    .and_then(|t| t.get("status"))
                    .and_then(|s| s.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                if status != "idle" {
                    return None;
                }
            }
            let thread_id = params
                .and_then(|p| {
                    p.get("threadId")
                        .or_else(|| p.get("thread").and_then(|t| t.get("id")))
                })
                .and_then(|t| t.as_str())
                .map(String::from);
            Some(AgentEvent::TurnCompleted {
                role,
                instance: None,
                thread_id,
            })
        }
        // Diff update
        "turn/diff/updated" => {
            let diff = params
                .and_then(|p| p.get("diff"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            Some(AgentEvent::DiffUpdated {
                role,
                instance: None,
                diff,
            })
        }
        // Approval request
        "item/commandExecution/requestApproval" => {
            let command = params
                .and_then(|p| p.get("command"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            // Use the top-level JSON-RPC message id (not params.id) — that's
            // the id codex expects echoed back in the approval response.
            let approval_id = msg_id.cloned().unwrap_or(serde_json::Value::Null);
            Some(AgentEvent::ApprovalRequested {
                role,
                instance: None,
                command,
                approval_id,
            })
        }
        // Token usage (v2 protocol: thread/tokenUsage/updated)
        "thread/tokenUsage/updated" => {
            let usage = params.and_then(|p| p.get("tokenUsage"));
            let last = usage.and_then(|u| u.get("last"));
            let total = usage.and_then(|u| u.get("total"));
            // input_tokens = last turn's context fill (actual window usage)
            // Falls back to total if "last" isn't available
            let input_tokens = last
                .and_then(|t| t.get("inputTokens"))
                .and_then(|t| t.as_u64())
                .or_else(|| {
                    total
                        .and_then(|t| t.get("inputTokens"))
                        .and_then(|t| t.as_u64())
                })
                .unwrap_or(0);
            // output_tokens = cumulative generated tokens (cost tracking)
            let output_tokens = total
                .and_then(|t| t.get("outputTokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            let context_window = usage
                .and_then(|u| u.get("modelContextWindow"))
                .and_then(|w| w.as_u64());
            Some(AgentEvent::TokenUsage {
                role,
                instance: None,
                input_tokens,
                output_tokens,
                context_window,
                cost_usd: None,
            })
        }
        // Command execution output (stream to command_output panel)
        "item/commandExecution/outputDelta" => {
            let delta = params
                .and_then(|p| p.get("delta"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            if delta.is_empty() {
                return None;
            }
            Some(AgentEvent::CommandOutput {
                role,
                instance: None,
                content: delta,
            })
        }
        _ => {
            tracing::debug!("Unhandled notification: {method}");
            None
        }
    }
}

/// Parse a Cursor ACP notification into an AgentEvent.
///
/// Real wire format (confirmed from log):
/// ```json
/// {"method":"session/update","params":{
///   "sessionId":"...",
///   "update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"..."}}
/// }}
/// ```
/// NOTE: turn completion does NOT come via a notification — it comes as a response to the
/// `session/prompt` request with `{"result":{"stopReason":"end_turn"}}`. That is handled
/// in the reader task's response handler, not here.
fn parse_cursor_notification(
    role: AgentRole,
    method: &str,
    params: Option<&Value>,
    _msg_id: Option<&Value>,
) -> Option<AgentEvent> {
    match method {
        "session/update" => {
            let Some(params_val) = params else {
                return None;
            };
            // Payload lives under params.update, not params directly
            let update = params_val.get("update")?;
            let kind = update
                .get("sessionUpdate")
                .and_then(|k| k.as_str())
                .unwrap_or("");

            match kind {
                "agent_message_chunk" => {
                    // Content at update.content.text
                    let content = update
                        .get("content")
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();
                    if content.is_empty() {
                        return None;
                    }
                    Some(AgentEvent::MessageDelta {
                        role,
                        instance: None,
                        content,
                    })
                }
                // Internal reasoning — don't surface in the output panel
                "agent_thought_chunk" => None,
                // tool_call: agent is about to run a tool — log it
                "tool_call" => {
                    let title = update.get("title").and_then(|t| t.as_str()).unwrap_or("?");
                    let kind = update.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                    tracing::info!("[cursor/{role}] tool '{title}' kind={kind}");
                    None
                }
                // tool_call_update: tool finished — emit rawOutput content for execute tools
                "tool_call_update" => {
                    let status = update.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    if status == "completed" {
                        let content = update
                            .get("rawOutput")
                            .and_then(|o| o.get("content"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("");
                        if !content.is_empty() {
                            return Some(AgentEvent::CommandOutput {
                                role,
                                instance: None,
                                content: content.to_string(),
                            });
                        }
                    }
                    None
                }
                // Available commands info — purely informational
                "available_commands_update" => None,
                other => {
                    // Try to extract text from unknown types; log for debugging
                    let content = update
                        .get("content")
                        .and_then(|c| c.get("text").or(Some(c)))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    if !content.is_empty() {
                        tracing::info!(
                            "[cursor/{role}] session/update kind='{other}' content_len={}",
                            content.len()
                        );
                        Some(AgentEvent::MessageDelta {
                            role,
                            instance: None,
                            content: content.to_string(),
                        })
                    } else {
                        tracing::info!(
                            "[cursor/{role}] session/update kind='{other}' (no text content)"
                        );
                        None
                    }
                }
            }
        }
        // Approval gate — should never fire with --force, but handle defensively
        "session/request_permission" => {
            tracing::warn!(
                "cursor: received session/request_permission despite --force flag; dropping"
            );
            None
        }
        "cursor/update_todos" | "cursor/task" | "cursor/create_plan" => None,
        _ => {
            tracing::warn!("cursor: unhandled notification '{method}'");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Claude stream-json event parser
// ---------------------------------------------------------------------------

fn parse_claude_event(
    role: AgentRole,
    instance: Option<String>,
    event: super::protocol::ClaudeStreamEvent,
    tx: &mpsc::UnboundedSender<AgentEvent>,
) {
    use super::protocol::{ClaudeContentBlock, ClaudeStreamEvent};

    match event {
        ClaudeStreamEvent::Assistant(a) => {
            for block in &a.message.content {
                if let ClaudeContentBlock::Text { text } = block {
                    if !text.is_empty() {
                        let _ = tx.send(AgentEvent::MessageDelta {
                            role,
                            instance: instance.clone(),
                            content: text.clone(),
                        });
                    }
                }
            }
            if let Some(usage) = &a.message.usage {
                let _ = tx.send(AgentEvent::TokenUsage {
                    role,
                    instance: instance.clone(),
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    context_window: None,
                    cost_usd: None,
                });
            }
        }

        ClaudeStreamEvent::Result(r) => {
            if r.is_error {
                let _ = tx.send(AgentEvent::Error {
                    role,
                    instance: instance.clone(),
                    error: format!("claude turn error: subtype={}", r.subtype),
                });
            }
            if let Some(usage) = &r.usage {
                let _ = tx.send(AgentEvent::TokenUsage {
                    role,
                    instance: instance.clone(),
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    context_window: None,
                    cost_usd: r.total_cost_usd,
                });
            }
            // TurnCompleted carries session_id as thread_id so the orchestrator can
            // call set_thread_id → next turn_start will use --resume <session_id>.
            let _ = tx.send(AgentEvent::TurnCompleted {
                role,
                instance,
                thread_id: Some(r.session_id),
            });
        }

        ClaudeStreamEvent::System(_) | ClaudeStreamEvent::Tool(_) | ClaudeStreamEvent::Unknown => {
            // Informational — no AgentEvent needed.
        }
    }
}

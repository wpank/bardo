pub mod connection;
pub mod events;
pub mod protocol;
pub mod roles;

// Re-exports
pub use connection::{AgentConnection, AppServerConnection, ClaudeConnection, CursorAcpConnection};
pub use events::AgentEvent;
pub use roles::{AgentBackend, AgentRole, ModelSpec};

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::sync::mpsc;

pub struct AgentPool {
    connections: HashMap<AgentRole, AgentConnection>,
    working_dir: PathBuf,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    fast_mode: bool,
    fallback_model: Option<String>,
}

impl AgentPool {
    pub fn new(working_dir: PathBuf, event_tx: mpsc::UnboundedSender<AgentEvent>) -> Self {
        Self {
            connections: HashMap::new(),
            working_dir,
            event_tx,
            fast_mode: false,
            fallback_model: None,
        }
    }

    pub fn set_fast_mode(&mut self, val: bool) {
        self.fast_mode = val;
    }

    pub fn set_fallback_model(&mut self, model: Option<String>) {
        self.fallback_model = model;
    }

    // NOTE: Sequential mode pre-warming (future enhancement):
    // AgentPool (single-agent per role) could benefit from warm spawning similarly to MultiAgentPool.
    // To implement:
    // 1. Add an `Option<(AgentConnection, PathBuf)>` field for warm agent storage
    // 2. Add pre_spawn_warm/promote_warm/evict_warm methods (similar to MultiAgentPool)
    // 3. In app/sequential.rs, after spawning implementer, call pool.pre_spawn_warm(QuickReviewer)
    // 4. When gates complete, call pool.promote_warm(QuickReviewer) instead of cold spawn
    // 5. Handle eviction on gate failures (AutoFix path)
    // This would save ~5-15s per phase transition in sequential mode.

    /// Spawn a connection for a role (no fallback, no dedup check).
    async fn try_spawn(
        &mut self,
        role: AgentRole,
        effort: &str,
        model: Option<&str>,
    ) -> Result<()> {
        let effective_backend = model
            .map(AgentBackend::from_model)
            .unwrap_or_else(|| role.backend());
        tracing::info!(
            role = %role,
            backend = ?effective_backend,
            model = model.unwrap_or("(default)"),
            effort = effort,
            cwd = %self.working_dir.display(),
            "Spawning agent"
        );
        let wd_str = self.working_dir.to_string_lossy().to_string();
        let conn = match effective_backend {
            AgentBackend::Codex => {
                let mut c = AppServerConnection::spawn(
                    role,
                    &self.working_dir,
                    self.event_tx.clone(),
                    effort,
                    None,
                    self.fast_mode,
                )
                .await?;
                c.initialize(&wd_str).await?;
                AgentConnection::Codex(c)
            }
            AgentBackend::Cursor => {
                let mut c = CursorAcpConnection::spawn(
                    role,
                    &self.working_dir,
                    self.event_tx.clone(),
                    model,
                    None,
                )
                .await?;
                c.initialize(&wd_str).await?;
                AgentConnection::Cursor(c)
            }
            AgentBackend::Claude => {
                let mut c = ClaudeConnection::spawn(
                    role,
                    &self.working_dir,
                    self.event_tx.clone(),
                    model,
                    None,
                    Some(effort),
                )
                .await?;
                c.initialize(&wd_str).await?;
                AgentConnection::Claude(c)
            }
        };
        self.connections.insert(role, conn);
        Ok(())
    }

    /// Spawn an agent for the given role if not already running.
    /// On failure, retries once with the fallback model if configured.
    pub async fn spawn(
        &mut self,
        role: AgentRole,
        effort: &str,
        model: Option<&str>,
    ) -> Result<()> {
        if self.connections.contains_key(&role) {
            return Ok(());
        }
        match self.try_spawn(role, effort, model).await {
            Ok(()) => Ok(()),
            Err(primary_err) => {
                if let Some(fb) = self.fallback_model.clone() {
                    if Some(fb.as_str()) != model {
                        tracing::warn!(
                            role = %role,
                            primary_model = model.unwrap_or("(default)"),
                            fallback_model = %fb,
                            "Primary spawn failed, trying fallback: {primary_err}"
                        );
                        return self.try_spawn(role, effort, Some(&fb)).await;
                    }
                }
                Err(primary_err)
            }
        }
    }

    /// Start a turn for the given role.
    pub async fn turn_start(
        &mut self,
        role: AgentRole,
        message: &str,
        model: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .connections
            .get_mut(&role)
            .with_context(|| format!("Agent {role} not spawned"))?;
        conn.turn_start(message, model).await
    }

    /// Interrupt a turn for the given role.
    pub async fn turn_interrupt(&mut self, role: AgentRole) -> Result<()> {
        if let Some(conn) = self.connections.get_mut(&role) {
            conn.turn_interrupt().await?;
        }
        Ok(())
    }

    /// Respond to an approval request.
    pub async fn respond_approval(
        &mut self,
        role: AgentRole,
        approval_id: &serde_json::Value,
        approved: bool,
    ) -> Result<()> {
        let conn = self
            .connections
            .get_mut(&role)
            .with_context(|| format!("Agent {role} not spawned"))?;
        conn.respond_approval(approval_id, approved).await
    }

    /// Set thread ID for a role (for cross-iteration persistence).
    pub fn set_thread_id(&mut self, role: AgentRole, thread_id: Option<String>) {
        if let Some(conn) = self.connections.get_mut(&role) {
            conn.set_thread_id(thread_id);
        }
    }

    /// Get thread ID for a role.
    pub fn thread_id(&self, role: AgentRole) -> Option<&str> {
        self.connections.get(&role).and_then(|c| c.thread_id())
    }

    /// Kill all agent processes.
    pub async fn kill_all(&mut self) {
        for conn in self.connections.values_mut() {
            let _ = conn.kill().await;
        }
        self.connections.clear();
    }

    /// Kill a specific agent.
    pub async fn kill(&mut self, role: AgentRole) {
        tracing::info!(role = %role, "Killing agent");
        if let Some(mut conn) = self.connections.remove(&role) {
            let _ = conn.kill().await;
        }
    }

    /// Reset all thread IDs so agents create fresh threads on next turn.
    /// For Cursor agents this resets the session ID; the next turn_start creates a new session.
    pub fn reset_all_threads(&mut self) {
        for conn in self.connections.values_mut() {
            conn.set_thread_id(None);
        }
    }

    pub fn is_spawned(&self, role: AgentRole) -> bool {
        self.connections.contains_key(&role)
    }
}

// ---------------------------------------------------------------------------
// MultiAgentPool — supports multiple instances per role
// ---------------------------------------------------------------------------

/// Unique identifier for an agent instance. Supports multiple agents per role.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentInstanceId {
    pub role: AgentRole,
    pub instance: String,
}

impl AgentInstanceId {
    pub fn new(role: AgentRole, instance: impl Into<String>) -> Self {
        Self {
            role,
            instance: instance.into(),
        }
    }

    pub fn default_for(role: AgentRole) -> Self {
        Self {
            role,
            instance: "default".to_string(),
        }
    }
}

impl fmt::Display for AgentInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.instance == "default" {
            write!(f, "{}", self.role)
        } else {
            write!(f, "{}:{}", self.role, self.instance)
        }
    }
}

/// Agent pool supporting multiple instances per role.
/// Used for parallel task execution where multiple implementers
/// run in separate worktrees simultaneously.
pub struct MultiAgentPool {
    connections: HashMap<AgentInstanceId, AgentConnection>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    default_working_dir: PathBuf,
    fast_mode: bool,
    fallback_model: Option<String>,
    /// Warm pool: pre-spawned idle agents waiting for turn_start.
    /// Maps (role, instance) to a ready connection (no turn_start called yet).
    warm_pool: HashMap<(AgentRole, String), AgentConnection>,
}

impl MultiAgentPool {
    pub fn new(working_dir: PathBuf, event_tx: mpsc::UnboundedSender<AgentEvent>) -> Self {
        Self {
            connections: HashMap::new(),
            event_tx,
            default_working_dir: working_dir,
            fast_mode: false,
            fallback_model: None,
            warm_pool: HashMap::new(),
        }
    }

    pub fn set_fast_mode(&mut self, val: bool) {
        self.fast_mode = val;
    }

    pub fn set_fallback_model(&mut self, model: Option<String>) {
        self.fallback_model = model;
    }

    /// Build a connection for an agent instance (no dedup check, no fallback).
    async fn try_spawn_connection(
        &self,
        id: &AgentInstanceId,
        wd: &PathBuf,
        effort: &str,
        model: Option<&str>,
    ) -> Result<AgentConnection> {
        let effective_backend = model
            .map(AgentBackend::from_model)
            .unwrap_or_else(|| id.role.backend());

        tracing::info!(
            "Spawning {} backend={:?} model={} cwd={}",
            id,
            effective_backend,
            model.unwrap_or("(none)"),
            wd.display()
        );

        let wd_str = wd.to_string_lossy().to_string();
        match effective_backend {
            AgentBackend::Codex => {
                let mut c = AppServerConnection::spawn(
                    id.role,
                    wd,
                    self.event_tx.clone(),
                    effort,
                    Some(id.instance.clone()),
                    self.fast_mode,
                )
                .await?;
                c.initialize(&wd_str).await?;
                Ok(AgentConnection::Codex(c))
            }
            AgentBackend::Cursor => {
                let mut c = CursorAcpConnection::spawn(
                    id.role,
                    wd,
                    self.event_tx.clone(),
                    model,
                    Some(id.instance.clone()),
                )
                .await?;
                c.initialize(&wd_str).await?;
                Ok(AgentConnection::Cursor(c))
            }
            AgentBackend::Claude => {
                let mut c = ClaudeConnection::spawn(
                    id.role,
                    wd,
                    self.event_tx.clone(),
                    model,
                    Some(id.instance.clone()),
                    Some(effort),
                )
                .await?;
                c.initialize(&wd_str).await?;
                Ok(AgentConnection::Claude(c))
            }
        }
    }

    /// Spawn an agent with a specific instance ID and optional custom working directory.
    /// On failure, retries once with the fallback model if configured.
    pub async fn spawn_instance(
        &mut self,
        id: AgentInstanceId,
        working_dir: Option<PathBuf>,
        effort: &str,
        model: Option<&str>,
    ) -> Result<()> {
        if self.connections.contains_key(&id) {
            return Ok(());
        }
        let wd = working_dir.unwrap_or_else(|| self.default_working_dir.clone());

        match self.try_spawn_connection(&id, &wd, effort, model).await {
            Ok(conn) => {
                self.connections.insert(id, conn);
                Ok(())
            }
            Err(primary_err) => {
                if let Some(fb) = self.fallback_model.clone() {
                    if Some(fb.as_str()) != model {
                        tracing::warn!(
                            id = %id,
                            primary_model = model.unwrap_or("(default)"),
                            fallback_model = %fb,
                            "Primary spawn failed, trying fallback: {primary_err}"
                        );
                        let conn = self
                            .try_spawn_connection(&id, &wd, effort, Some(&fb))
                            .await?;
                        self.connections.insert(id, conn);
                        return Ok(());
                    }
                }
                Err(primary_err)
            }
        }
    }

    /// Spawn a default (single) agent for a role.
    pub async fn spawn(
        &mut self,
        role: AgentRole,
        effort: &str,
        model: Option<&str>,
    ) -> Result<()> {
        self.spawn_instance(AgentInstanceId::default_for(role), None, effort, model)
            .await
    }

    /// Start a turn for a specific instance.
    pub async fn turn_start(
        &mut self,
        id: &AgentInstanceId,
        message: &str,
        model: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .connections
            .get_mut(id)
            .with_context(|| format!("Agent {} not spawned", id))?;
        conn.turn_start(message, model).await
    }

    /// Start turn for default instance of a role.
    pub async fn turn_start_default(
        &mut self,
        role: AgentRole,
        message: &str,
        model: Option<&str>,
    ) -> Result<()> {
        self.turn_start(&AgentInstanceId::default_for(role), message, model)
            .await
    }

    /// Interrupt a specific instance.
    pub async fn turn_interrupt(&mut self, id: &AgentInstanceId) -> Result<()> {
        if let Some(conn) = self.connections.get_mut(id) {
            conn.turn_interrupt().await?;
        }
        Ok(())
    }

    /// Respond to an approval request.
    pub async fn respond_approval(
        &mut self,
        id: &AgentInstanceId,
        approval_id: &serde_json::Value,
        approved: bool,
    ) -> Result<()> {
        let conn = self
            .connections
            .get_mut(id)
            .with_context(|| format!("Agent {} not spawned", id))?;
        conn.respond_approval(approval_id, approved).await
    }

    /// Kill a specific instance.
    pub async fn kill_instance(&mut self, id: &AgentInstanceId) {
        if let Some(mut conn) = self.connections.remove(id) {
            let _ = conn.kill().await;
        }
    }

    /// Kill all agents associated with a plan (matched by instance ID containing the plan name).
    pub async fn kill_plan_agents(&mut self, plan: &str) {
        let ids: Vec<AgentInstanceId> = self
            .connections
            .keys()
            .filter(|id| id.instance.contains(plan))
            .cloned()
            .collect();
        for id in ids {
            self.kill_instance(&id).await;
        }
    }

    /// Kill all instances of a role.
    pub async fn kill_role(&mut self, role: AgentRole) {
        let ids: Vec<AgentInstanceId> = self
            .connections
            .keys()
            .filter(|id| id.role == role)
            .cloned()
            .collect();
        for id in ids {
            self.kill_instance(&id).await;
        }
    }

    /// Kill all agents.
    pub async fn kill_all(&mut self) {
        let ids: Vec<AgentInstanceId> = self.connections.keys().cloned().collect();
        for id in ids {
            if let Some(mut conn) = self.connections.remove(&id) {
                let _ = conn.kill().await;
            }
        }
    }

    /// Get all active instances for a role.
    pub fn instances_for_role(&self, role: AgentRole) -> Vec<&AgentInstanceId> {
        self.connections
            .keys()
            .filter(|id| id.role == role)
            .collect()
    }

    /// Total active agent count.
    pub fn active_count(&self) -> usize {
        self.connections.len()
    }

    /// Active agent count for a specific role.
    pub fn role_count(&self, role: AgentRole) -> usize {
        self.connections.keys().filter(|id| id.role == role).count()
    }

    pub fn is_spawned(&self, id: &AgentInstanceId) -> bool {
        self.connections.contains_key(id)
    }

    /// Insert a pre-built connection (used for parallel spawn then sequential insert).
    pub fn insert_connection(&mut self, id: AgentInstanceId, conn: AgentConnection) {
        self.connections.insert(id, conn);
    }

    /// Clone the event sender (for out-of-pool spawning).
    pub fn event_tx(&self) -> mpsc::UnboundedSender<AgentEvent> {
        self.event_tx.clone()
    }

    pub fn set_thread_id(&mut self, id: &AgentInstanceId, thread_id: Option<String>) {
        if let Some(conn) = self.connections.get_mut(id) {
            conn.set_thread_id(thread_id);
        }
    }

    pub fn thread_id(&self, id: &AgentInstanceId) -> Option<&str> {
        self.connections.get(id).and_then(|c| c.thread_id())
    }

    pub fn reset_all_threads(&mut self) {
        for conn in self.connections.values_mut() {
            conn.set_thread_id(None);
        }
    }

    /// Pre-spawn an agent into the warm pool. Agent is initialized but turn_start is NOT called.
    /// If a warm agent already exists for this role/instance, returns early (idempotent).
    /// On failure, retries once with the fallback model if configured.
    pub async fn pre_spawn_warm(
        &mut self,
        id: AgentInstanceId,
        working_dir: Option<PathBuf>,
        effort: &str,
        model: Option<&str>,
    ) -> Result<()> {
        let pool_key = (id.role, id.instance.clone());

        // Already in warm pool or active connections
        if self.warm_pool.contains_key(&pool_key) || self.connections.contains_key(&id) {
            return Ok(());
        }

        let wd = working_dir.unwrap_or_else(|| self.default_working_dir.clone());

        let conn = match self.try_spawn_connection(&id, &wd, effort, model).await {
            Ok(conn) => conn,
            Err(primary_err) => {
                if let Some(fb) = self.fallback_model.clone() {
                    if Some(fb.as_str()) != model {
                        tracing::warn!(
                            id = %id,
                            fallback_model = %fb,
                            "Warm spawn failed, trying fallback: {primary_err}"
                        );
                        self.try_spawn_connection(&id, &wd, effort, Some(&fb))
                            .await?
                    } else {
                        return Err(primary_err);
                    }
                } else {
                    return Err(primary_err);
                }
            }
        };

        self.warm_pool.insert(pool_key, conn);
        Ok(())
    }

    /// Promote a warm agent to active (if ready). Returns the AgentInstanceId if successful.
    /// Waits up to 100ms for the agent to be ready. If not ready by then, returns None and the caller
    /// should spawn a cold agent instead.
    pub async fn promote_warm(
        &mut self,
        role: AgentRole,
        instance: &str,
    ) -> Option<AgentInstanceId> {
        let pool_key = (role, instance.to_string());

        // Check if warm agent exists and is ready. Since pre_spawn_warm calls initialize(),
        // the agent is ready to receive turn_start immediately upon insertion into warm_pool.
        if let Some(conn) = self.warm_pool.remove(&pool_key) {
            let id = AgentInstanceId::new(role, instance);
            // Note: working_dir tracking would need to be added separately if needed for the warm pool.
            // For now, we store it on promotion.
            self.connections.insert(id.clone(), conn);
            tracing::info!("Promoted warm agent {}", id);
            return Some(id);
        }

        None
    }

    /// Evict (kill) a warm agent that won't be needed. No-op if agent doesn't exist in warm pool.
    pub async fn evict_warm(&mut self, role: AgentRole, instance: &str) -> Result<()> {
        let pool_key = (role, instance.to_string());

        if let Some(mut conn) = self.warm_pool.remove(&pool_key) {
            tracing::info!("Evicting warm agent {}:{}", role, instance);
            conn.kill().await?;
        }

        Ok(())
    }
}

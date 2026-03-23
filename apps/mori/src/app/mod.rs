mod events;
mod gates;
mod parallel;
mod sequential;
mod tui_actions;
mod util;

#[allow(unused_imports)]
use self::events::*;
#[allow(unused_imports)]
use self::gates::*;
#[allow(unused_imports)]
use self::parallel::*;
#[allow(unused_imports)]
use self::sequential::*;
#[allow(unused_imports)]
use self::tui_actions::*;
#[allow(unused_imports)]
use self::util::*;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{info, warn};

use crate::agent::{
    AgentBackend, AgentEvent, AgentInstanceId, AgentPool, AgentRole, MultiAgentPool,
};
use crate::conductor::{Conductor, ConductorAction, ConductorConfig, ConductorContext};
use crate::git::worktree::WorktreeManager;
use crate::git::{AutoStashSession, GitEvent, GitManager};
use crate::orchestrator::phase::Verdict;
use crate::orchestrator::tasks;
use crate::orchestrator::{
    ExecutorAction, GlobalTaskId, Orchestrator, OrchestratorConfig, OrchestratorEvent,
    OrchestratorState, ParallelExecutor, PlanPhase, UnifiedTaskDag,
};
use crate::state::persistence::PersistenceManager;
use crate::state::{
    AgentPaneGroup, ConductorHistoryEntry, ConfirmAction, DetailSubTab, FocusZone, InputMode,
    LogLevel, Notification, PendingApproval, PipelineRunState, PlanDetailTab, RunPlanEntry,
    RunPlanStatus, RunState, VerifyEntry, VerifyStatus,
};
use crate::tui::{self, atmosphere::Atmosphere, input, TuiAction};

/// Gate completion messages for non-blocking gate execution
enum GateCompletion {
    Compile {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    Clippy {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    Test {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    PostMerge {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    TerminalRender {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    GolemLifecycle {
        plan: String,
        result: anyhow::Result<crate::orchestrator::gates::GateResult>,
    },
    MergeComplete {
        plan: String,
        success: bool,
        error: Option<String>,
        commit_count: String,
    },
    ReconcileComplete {
        messages: Vec<String>,
        merged_plans: Vec<String>,
        already_reconciled: Vec<String>,
    },
}

/// Verification task completion
enum VerifyCompletion {
    Done {
        plan_base: String,
        plan_num: String,
        passed: bool,
        output: String,
        summary: Option<String>,
    },
}

/// CLI configuration passed from main
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub repo_root: PathBuf,
    pub plan_specs: Vec<String>,
    pub no_review: bool,
    pub skip_tests: bool,
    pub max_iterations: u32,
    pub batch_size: Option<usize>,
    pub batch_id: String,
    pub model: Option<String>,
    pub headless: bool,
    pub no_docs: bool,
    pub max_agents: usize,
    pub max_parallel_plans: usize,
    pub parallel: bool,
    pub pre_plan: bool,
    pub refactor: bool,
    pub fast: bool,
    pub express: bool,
    pub fallback_model: Option<String>,
    /// Execution preset (e.g. "quality", "balanced", "cost", "speed").
    pub preset: Option<String>,
    /// Whether the embedded gateway is enabled.
    pub gateway_enabled: bool,
    /// Port for the embedded gateway.
    pub gateway_port: u16,
    /// API key for the embedded gateway (auto-generated if empty).
    pub gateway_api_key: String,
}

/// Run the application
pub async fn run(mut config: AppConfig) -> Result<()> {
    // Start the embedded inference gateway if enabled.
    #[cfg(feature = "gateway")]
    if config.gateway_enabled {
        start_embedded_gateway(&mut config).await;
    }

    // When gateway is disabled (--no-gateway), clear any gateway-related env vars
    // so agents don't accidentally route through an external or stale gateway.
    if !config.gateway_enabled {
        std::env::remove_var("ANTHROPIC_BASE_URL");
        std::env::remove_var("BARDO_GATEWAY_API_KEY");
    }

    if config.parallel {
        return run_parallel(config).await;
    }
    run_sequential(config).await
}

/// Start the embedded bardo-gateway as a background tokio task.
///
/// Sets `ANTHROPIC_BASE_URL` and `BARDO_GATEWAY_API_KEY` in the process
/// environment so all spawned agent subprocesses route through the gateway.
#[cfg(feature = "gateway")]
async fn start_embedded_gateway(config: &mut AppConfig) {
    use bardo_gateway::GatewayConfig;

    // Auto-generate API key if not set.
    if config.gateway_api_key.is_empty() {
        config.gateway_api_key = format!("mori-{}", fastrand::u64(..));
    }

    // Collect Anthropic API keys from environment.
    let primary = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            warn!("ANTHROPIC_API_KEY not set, skipping embedded gateway");
            config.gateway_enabled = false;
            return;
        }
    };
    let mut api_keys = vec![primary];
    for i in 2..=10 {
        if let Ok(key) = std::env::var(format!("ANTHROPIC_API_KEY_{i}")) {
            if !key.is_empty() {
                api_keys.push(key);
            }
        }
    }

    let gw_config = GatewayConfig {
        port: config.gateway_port,
        bind: "127.0.0.1".into(),
        api_key: config.gateway_api_key.clone(),
        anthropic_api_keys: api_keys,
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty()),
        openrouter_api_key: std::env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|k| !k.is_empty()),
        ..GatewayConfig::default()
    };

    let port = config.gateway_port;
    let api_key = config.gateway_api_key.clone();

    // Spawn gateway as a background task.
    tokio::spawn(async move {
        if let Err(e) = bardo_gateway::start_server(gw_config).await {
            tracing::error!(error = %e, "embedded gateway exited with error");
        }
    });

    // Give the gateway a moment to bind, then verify it's actually listening.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let base_url = format!("http://127.0.0.1:{port}");
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await {
        Ok(_) => {
            // Gateway is listening — set env vars so agents route through it.
            std::env::set_var("ANTHROPIC_BASE_URL", &base_url);
            std::env::set_var("BARDO_GATEWAY_API_KEY", &api_key);
        }
        Err(e) => {
            warn!(port, error = %e, "embedded gateway failed to bind — agents will use direct API");
            config.gateway_enabled = false;
            return;
        }
    }

    info!(
        port,
        url = %base_url,
        "embedded gateway started — all agents will route through it"
    );
}

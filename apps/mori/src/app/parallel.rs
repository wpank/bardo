use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::{mpsc, Semaphore};
use tracing::{info, warn};

use crate::agent::{AgentBackend, AgentEvent, AgentInstanceId, AgentRole, MultiAgentPool};
use crate::conductor::{Conductor, ConductorAction, ConductorConfig, ConductorContext};
use crate::git::worktree::WorktreeManager;
use crate::git::{AutoStashSession, GitEvent, GitManager};
use crate::orchestrator::phase::Verdict;
use crate::orchestrator::{
    ExecutorAction, GlobalTaskId, Orchestrator, OrchestratorConfig, OrchestratorEvent,
    ParallelExecutor, PlanPhase, UnifiedTaskDag,
};
use crate::state::persistence::PersistenceManager;
use crate::state::{
    AgentPaneGroup, ConductorHistoryEntry, ConfirmAction, DetailSubTab, FocusZone, InputMode,
    LogLevel, Notification, PipelineRunState, PlanDetailTab, RunPlanEntry, RunPlanStatus, RunState,
};
use crate::tui::{self, atmosphere::Atmosphere, input, TuiAction};

use super::*;

// ---------------------------------------------------------------------------
// Parallel execution mode
// ---------------------------------------------------------------------------

/// Count active agents across all roles (implementers, reviewers, scribes, auto-fixers, etc.).
/// Used to enforce a global agent budget so non-implementer agents don't pile up unbounded.
fn active_agent_count(state: &RunState) -> usize {
    state.parallel_agents.iter().filter(|p| p.active).count()
}

/// Update the executor's view of total active agents before calling schedule_next().
fn sync_agent_budget(executor: &mut ParallelExecutor, state: &RunState) {
    executor.set_total_active_agents(active_agent_count(state));
}

/// Result of a background agent cold-start, sent to the select loop for processing
/// so the TUI tick arm is never blocked during agent initialization.
pub(crate) enum AgentSpawnReady {
    Single {
        task_id: GlobalTaskId,
        instance_id: String,
        prompt: String,
        result: anyhow::Result<(AgentInstanceId, crate::agent::AgentConnection, PathBuf)>,
    },
    Batch {
        task_ids: Vec<GlobalTaskId>,
        plan_base: String,
        instance_id: String,
        prompt: String,
        result: anyhow::Result<(AgentInstanceId, crate::agent::AgentConnection, PathBuf)>,
    },
}

/// Parse the summary line of `git diff --stat HEAD` output into (added, removed) line counts.
/// The summary looks like: "3 files changed, 84 insertions(+), 12 deletions(-)"
fn parse_diff_stat_summary(diff_stat: &str) -> (u32, u32) {
    let mut added = 0u32;
    let mut removed = 0u32;
    if let Some(summary) = diff_stat.lines().last() {
        for part in summary.split(',') {
            let p = part.trim();
            if p.contains("insertion") {
                added = p
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            } else if p.contains("deletion") {
                removed = p
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            }
        }
    }
    (added, removed)
}

/// Populate `context/in/` under a **plan worktree** so agents can read mirrored briefs, extracts, etc.
/// Skips when the agent runs in the main repo root (no worktree).
fn inject_implementer_context_in_worktree(
    repo_root: &std::path::Path,
    worktree_path: &std::path::Path,
    working_dir: &std::path::Path,
    plan_info: &crate::orchestrator::PlanInfo,
    iter: u32,
) {
    if working_dir == repo_root || !worktree_path.exists() {
        return;
    }
    let artifact_root = repo_root.join("tmp/bardo-artifacts");
    let _ = std::fs::create_dir_all(&artifact_root);
    let registry_root = repo_root.join("plans/context/registry");
    let _ = std::fs::create_dir_all(&registry_root);
    let artifacts = crate::orchestrator::ArtifactStore::new(artifact_root);
    let registry = crate::orchestrator::Registry::new(registry_root);
    if let Err(e) = registry.init() {
        tracing::warn!("registry init for context/in inject: {e}");
    }
    let injector = crate::orchestrator::inject::ContextInjector {
        artifact_store: &artifacts,
        registry: &registry,
        repo_root,
    };
    let plan_deps: Vec<String> = plan_info
        .frontmatter
        .as_ref()
        .map(|f| f.depends_on.clone())
        .unwrap_or_default();
    if let Err(e) = injector.inject_for_implementer(working_dir, &plan_info.num, iter, &plan_deps) {
        tracing::warn!(
            "context/in inject failed for plan {}: {e}",
            plan_info.display_name()
        );
    }
}

/// Attempt to spawn a raw agent connection. Used by background spawn closures.
async fn try_spawn_raw(
    backend: AgentBackend,
    role: AgentRole,
    wd: &std::path::Path,
    tx: mpsc::UnboundedSender<AgentEvent>,
    effort: &str,
    iid: String,
    fast_mode: bool,
    model: Option<&str>,
) -> anyhow::Result<(
    AgentInstanceId,
    crate::agent::AgentConnection,
    std::path::PathBuf,
)> {
    let aid = AgentInstanceId::new(role, iid.clone());
    let effective_backend = model.map(AgentBackend::from_model).unwrap_or(backend);
    let conn = match effective_backend {
        AgentBackend::Codex => {
            let mut c = crate::agent::AppServerConnection::spawn(
                role,
                &wd.to_path_buf(),
                tx,
                effort,
                Some(iid),
                fast_mode,
            )
            .await?;
            c.initialize(&wd.to_string_lossy()).await?;
            crate::agent::AgentConnection::Codex(c)
        }
        AgentBackend::Cursor => {
            let mut c = crate::agent::CursorAcpConnection::spawn(
                role,
                &wd.to_path_buf(),
                tx,
                model,
                Some(iid),
            )
            .await?;
            c.initialize(&wd.to_string_lossy()).await?;
            crate::agent::AgentConnection::Cursor(c)
        }
        AgentBackend::Claude => {
            let mut c = crate::agent::ClaudeConnection::spawn(
                role,
                &wd.to_path_buf(),
                tx,
                model,
                Some(iid),
                Some(effort),
            )
            .await?;
            c.initialize(&wd.to_string_lossy()).await?;
            crate::agent::AgentConnection::Claude(c)
        }
    };
    Ok((aid, conn, wd.to_path_buf()))
}

/// Execute actions returned by the parallel executor.
pub(crate) async fn execute_actions(
    actions: Vec<ExecutorAction>,
    executor: &mut ParallelExecutor,
    pool: &mut MultiAgentPool,
    worktree_mgr: &WorktreeManager,
    state: &mut RunState,
    config: &AppConfig,
    persistence: &crate::state::persistence::PersistenceManager,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
    batch_branch: &str,
    git_manager: &GitManager,
    spawn_ready_tx: &mpsc::UnboundedSender<AgentSpawnReady>,
) -> Result<()> {
    // When paused, don't execute any new actions
    if state.pipeline_run_state == PipelineRunState::Paused {
        return Ok(());
    }

    // Separate SpawnTaskAgent / SpawnTaskAgentBatch actions so they can be spawned concurrently.
    let (spawn_actions, other_actions): (Vec<_>, Vec<_>) = actions.into_iter().partition(|a| {
        matches!(
            a,
            ExecutorAction::SpawnTaskAgent { .. } | ExecutorAction::SpawnTaskAgentBatch { .. }
        )
    });

    for action in other_actions {
        match action {
            ExecutorAction::CreatePipeline { ref plan } => {
                info!("Creating pipeline for plan {plan}");
                // Extract prd2 context for the plan being started
                let plan_num = plan.split('-').next().unwrap_or(plan);
                let _ =
                    crate::orchestrator::context::extract_prd2_context(&config.repo_root, plan_num);
                state.add_log("executor", &format!("Starting plan {plan}"), LogLevel::Info);
                state
                    .plan_start_times
                    .insert(plan.clone(), std::time::Instant::now());
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "implementer".to_string();
                }
                state
                    .plan_phase_started
                    .insert(plan.clone(), std::time::Instant::now());
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.status = RunPlanStatus::Active;
                    if entry.started_at.is_none() {
                        entry.started_at = Some(std::time::Instant::now());
                    }
                }
            }
            ExecutorAction::EnsureWorktree { ref plan } => {
                let wt_mgr = worktree_mgr.clone();
                let plan_clone = plan.clone();
                let batch_clone = batch_branch.to_string();
                match tokio::task::spawn_blocking(move || {
                    wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                })
                .await
                {
                    Ok(Err(e)) => {
                        warn!("Worktree creation failed for {plan}: {e}");
                        state.add_log(
                            "executor",
                            &format!("Worktree failed for {plan}: {e}"),
                            LogLevel::Warn,
                        );
                    }
                    Err(e) => {
                        warn!("Worktree spawn_blocking panicked for {plan}: {e}");
                        state.add_log(
                            "executor",
                            &format!("Worktree failed for {plan}: {e}"),
                            LogLevel::Warn,
                        );
                    }
                    Ok(Ok(_)) => {}
                }
            }
            ExecutorAction::SpawnTaskAgent { .. } | ExecutorAction::SpawnTaskAgentBatch { .. } => { /* handled in parallel batch below */
            }
            ExecutorAction::RunPlanGates { ref plan } => {
                // Gates run in plan worktrees with separate target dirs — no serialization needed.
                // Multiple plans can gate in parallel.

                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    if entry.status == RunPlanStatus::Pending {
                        entry.status = RunPlanStatus::Active;
                        if entry.started_at.is_none() {
                            entry.started_at = Some(std::time::Instant::now());
                        }
                    }
                    entry.phase = "compile-gate".to_string();
                }
                state
                    .plan_phase_started
                    .insert(plan.clone(), std::time::Instant::now());
                // Checkpoint on gate transition so restarts resume from here, not scratch.
                let (inp_tok, out_tok) = (
                    state
                        .parallel_agents
                        .iter()
                        .map(|p| p.input_tokens)
                        .sum::<u64>(),
                    state
                        .parallel_agents
                        .iter()
                        .map(|p| p.output_tokens)
                        .sum::<u64>(),
                );
                write_checkpoint(
                    &executor,
                    &persistence,
                    &worktree_mgr,
                    batch_branch,
                    inp_tok,
                    out_tok,
                );
                let worktree_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                let gate_dir = if worktree_path.exists() {
                    worktree_path
                } else {
                    config.repo_root.clone()
                };
                info!(
                    "gate[{plan}] dir={} is_worktree={}",
                    gate_dir.display(),
                    gate_dir != config.repo_root
                );

                // Worktree health check: ensure Cargo.toml exists and copy tests/harness if missing
                if !gate_dir.join("Cargo.toml").exists() {
                    warn!("gate[{plan}] Cargo.toml missing in {}", gate_dir.display());
                    state.add_log(
                        "gate",
                        &format!("Cargo.toml missing in worktree for {plan}"),
                        LogLevel::Warn,
                    );
                }
                let harness_dir = gate_dir.join("tests/harness");
                if !harness_dir.exists() {
                    let src_harness = config.repo_root.join("tests/harness");
                    if src_harness.exists() {
                        info!("gate[{plan}] copying tests/harness from repo root to worktree");
                        let _ = std::fs::create_dir_all(gate_dir.join("tests"));
                        let dest = gate_dir.join("tests/harness");
                        // cp -r via a helper; fs_extra or manual walk
                        let src = src_harness.clone();
                        let dst = dest.clone();
                        let _ = std::process::Command::new("cp")
                            .args(["-r", &src.to_string_lossy(), &dst.to_string_lossy()])
                            .output();
                    }
                }

                state.add_log(
                    "executor",
                    &format!("Running gates for {plan} in {}", gate_dir.display()),
                    LogLevel::Info,
                );
                let gate_dir_main = gate_dir.clone();
                let tx = gate_tx.clone();
                let plan_name = plan.clone();
                let clippy_enabled = state.config.clippy_enabled;
                tokio::spawn(async move {
                    let _ = crate::orchestrator::gates::format_gate(&gate_dir_main).await;
                    // Use combined clippy+compile gate to avoid cache invalidation
                    let result = if clippy_enabled {
                        crate::orchestrator::gates::clippy_compile_gate(&gate_dir_main, &plan_name)
                            .await
                    } else {
                        crate::orchestrator::gates::compile_gate(&gate_dir_main, &plan_name).await
                    };
                    let _ = tx.send(GateCompletion::Compile {
                        plan: plan_name,
                        result,
                    });
                });
                state
                    .gate_running
                    .insert(format!("cargo clippy ({})", plan));

                // Conditionally run terminal render and golem lifecycle gates
                // based on which crates the plan touches.
                let plan_num = plan.split('-').next().unwrap_or(plan);
                let plan_path = config.repo_root.join("plans").join(format!("{}.md", plan));
                let plan_content = std::fs::read_to_string(&plan_path).unwrap_or_default();

                if crate::orchestrator::gates::plan_touches_crate(&plan_content, "bardo-terminal") {
                    let gate_dir2 = gate_dir.clone();
                    let tx = gate_tx.clone();
                    let pn = plan.clone();
                    tokio::spawn(async move {
                        let result =
                            crate::orchestrator::gates::terminal_render_gate(&gate_dir2).await;
                        let _ = tx.send(GateCompletion::TerminalRender { plan: pn, result });
                    });
                    state
                        .gate_running
                        .insert(format!("terminal render ({})", plan));
                }

                if crate::orchestrator::gates::plan_touches_crate(&plan_content, "golem-") {
                    let gate_dir2 = gate_dir.clone();
                    let tx = gate_tx.clone();
                    let pn = plan.clone();
                    tokio::spawn(async move {
                        let result =
                            crate::orchestrator::gates::golem_lifecycle_gate(&gate_dir2).await;
                        let _ = tx.send(GateCompletion::GolemLifecycle { plan: pn, result });
                    });
                    state
                        .gate_running
                        .insert(format!("golem lifecycle ({})", plan));
                }
            }
            ExecutorAction::PreSpawnWarmReviewer { ref plan } => {
                // Gate-review overlap: pre-spawn a warm reviewer agent while gates run.
                // Determine reviewer role based on plan complexity.
                let reviewer_role = if let Some(ps) = executor.plan_states.get(plan) {
                    if ps.iteration > 1 {
                        AgentRole::Architect
                    } else {
                        AgentRole::QuickReviewer
                    }
                } else {
                    AgentRole::QuickReviewer
                };

                let reviewer_instance_id = match reviewer_role {
                    AgentRole::QuickReviewer => format!("quick:{}", plan),
                    AgentRole::Architect => format!("arch:{}", plan),
                    _ => format!("quick:{}", plan),
                };
                let aid = AgentInstanceId::new(reviewer_role, reviewer_instance_id.clone());
                let effort = state.config.effort_for(reviewer_role).label().to_string();
                let model = state.config.model_for(reviewer_role).map(|s| s.to_string());

                let worktree_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                let working_dir = if worktree_path.exists() {
                    Some(worktree_path.clone())
                } else {
                    None
                };

                state.add_log(
                    "executor",
                    &format!(
                        "Pre-spawning warm {} for {} (overlap with gates)",
                        reviewer_role, plan
                    ),
                    LogLevel::Info,
                );

                // Pre-spawn reviewer in background. We don't await this — it's spawned async.
                // The executor will track active_reviewer_instance for later use.
                let plan_clone = plan.clone();
                let instance_id_clone = reviewer_instance_id.clone();
                pool.pre_spawn_warm(aid, working_dir, &effort, model.as_deref())
                    .await
                    .ok();
                executor.set_active_reviewer(&plan_clone, instance_id_clone.clone());

                state.add_log(
                    "executor",
                    &format!(
                        "Warm {} spawning for {} (no turn_start yet)",
                        reviewer_role, plan
                    ),
                    LogLevel::Debug,
                );
            }
            ExecutorAction::RunPlanReviews { ref plan } => {
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    if entry.status == RunPlanStatus::Pending {
                        entry.status = RunPlanStatus::Active;
                    }
                    entry.phase = "reviewing".to_string();
                }
                state
                    .plan_phase_started
                    .insert(plan.clone(), std::time::Instant::now());
                // Checkpoint on review transition so restarts resume from reviewing, not gating.
                let (inp_tok, out_tok) = (
                    state
                        .parallel_agents
                        .iter()
                        .map(|p| p.input_tokens)
                        .sum::<u64>(),
                    state
                        .parallel_agents
                        .iter()
                        .map(|p| p.output_tokens)
                        .sum::<u64>(),
                );
                write_checkpoint(
                    &executor,
                    &persistence,
                    &worktree_mgr,
                    batch_branch,
                    inp_tok,
                    out_tok,
                );
                if config.no_review || config.express {
                    if !config.no_docs && state.config.scribe_enabled && executor.can_spawn_more() {
                        // Fast path with docs: skip code reviewers but run scribe
                        state.add_log(
                            "executor",
                            &format!("No-review fast path for {plan} → spawning scribe"),
                            LogLevel::Info,
                        );
                        state
                            .plan_review_stage
                            .insert(plan.clone(), crate::state::ReviewStage::ScribePending);
                        let mut pending = std::collections::HashSet::new();

                        let plan_num = plan.split('-').next().unwrap_or(plan).to_string();
                        let pi = crate::orchestrator::plan::discover_plans(
                            &config.repo_root.join("plans"),
                            &[plan_num],
                        )
                        .ok()
                        .and_then(|ps| ps.into_iter().find(|p| p.base == *plan));

                        if let Some(ref plan_info) = pi {
                            if let Ok(prompt) = crate::orchestrator::prompts::scribe_prompt(
                                &config.repo_root,
                                plan_info,
                                None,
                            ) {
                                let pfx = "scribe";
                                let review_iid = format!("{pfx}:{plan}");
                                let aid =
                                    AgentInstanceId::new(AgentRole::Scribe, review_iid.clone());
                                let effort = state.config.effort_for(AgentRole::Scribe).label();
                                if pool
                                    .spawn_instance(
                                        aid.clone(),
                                        None,
                                        effort,
                                        state.config.model_for(AgentRole::Scribe),
                                    )
                                    .await
                                    .is_ok()
                                {
                                    pool.set_thread_id(&aid, None);
                                    if let Err(e) = pool
                                        .turn_start(
                                            &aid,
                                            &prompt,
                                            state.config.model_for(AgentRole::Scribe),
                                        )
                                        .await
                                    {
                                        tracing::error!(
                                            "Failed to start scribe turn for {plan}: {e}"
                                        );
                                        state.add_log(
                                            "executor",
                                            &format!("scribe turn_start failed for {plan}: {e}"),
                                            LogLevel::Error,
                                        );
                                    }
                                    state
                                        .parallel_agents
                                        .push(crate::state::ParallelAgentState {
                                            instance_id: review_iid,
                                            role: AgentRole::Scribe,
                                            plan: plan.clone(),
                                            task: pfx.to_string(),
                                            output: String::new(),
                                            input_tokens: 0,
                                            output_tokens: 0,
                                            cost_usd: 0.0,
                                            active: true,
                                            finished_at: None,
                                            model: String::new(),
                                            turn_started: false,
                                            render_cache: Default::default(),
                                        });
                                    pending.insert(AgentRole::Scribe);
                                }
                            }
                        }

                        if pending.is_empty() {
                            // Scribe spawn failed — skip to merge
                            state.plan_review_stage.remove(plan);
                            let merge_actions = executor.handle_plan_reviews_passed(plan);
                            Box::pin(execute_actions(
                                merge_actions,
                                executor,
                                pool,
                                worktree_mgr,
                                state,
                                config,
                                persistence,
                                gate_tx,
                                batch_branch,
                                git_manager,
                                spawn_ready_tx,
                            ))
                            .await?;
                        } else {
                            state.plan_pending_reviews.insert(plan.clone(), pending);
                        }
                    } else {
                        // Fast path, no docs: merge immediately
                        let merge_actions = executor.handle_plan_reviews_passed(plan);
                        Box::pin(execute_actions(
                            merge_actions,
                            executor,
                            pool,
                            worktree_mgr,
                            state,
                            config,
                            persistence,
                            gate_tx,
                            batch_branch,
                            git_manager,
                            spawn_ready_tx,
                        ))
                        .await?;
                    }
                } else {
                    let plan_num = plan.split('-').next().unwrap_or(plan);
                    let plan_info = crate::orchestrator::plan::discover_plans(
                        &config.repo_root.join("plans"),
                        &[plan_num.to_string()],
                    )
                    .ok()
                    .and_then(|ps| ps.into_iter().find(|p| p.base == *plan));

                    if let Some(ref pi) = plan_info {
                        // Classify plan using task file when frontmatter is absent (most plans)
                        let plan_task_file =
                            crate::orchestrator::tasks::load_checklist(&config.repo_root, plan_num)
                                .ok()
                                .flatten();
                        let complexity = crate::orchestrator::complexity::classify_plan_with_tasks(
                            pi,
                            plan_task_file.as_ref(),
                        );
                        let pipeline =
                            crate::orchestrator::complexity::PipelineConfig::for_complexity(
                                complexity,
                            );

                        if !pipeline.run_reviews {
                            // Trivial/Simple: skip reviews entirely.
                            state.add_log(
                                "executor",
                                &format!("Skipping review for {plan} (complexity={complexity:?})"),
                                LogLevel::Info,
                            );
                            let merge_actions = executor.handle_plan_reviews_passed(plan);
                            Box::pin(execute_actions(
                                merge_actions,
                                executor,
                                pool,
                                worktree_mgr,
                                state,
                                config,
                                persistence,
                                gate_tx,
                                batch_branch,
                                git_manager,
                                spawn_ready_tx,
                            ))
                            .await?;
                        } else {
                            let iteration = executor.plan_iteration(plan);

                            // Create worktree (shared between QuickReviewer and Architect paths)
                            let wt_mgr = worktree_mgr.clone();
                            let plan_clone = plan.clone();
                            let batch_clone = batch_branch.to_string();
                            let wd = match tokio::task::spawn_blocking(move || {
                                wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                            })
                            .await
                            {
                                Ok(Ok(pw)) => {
                                    info!("review[{plan}] worktree ready at {}", pw.path.display());
                                    Some(pw.path)
                                }
                                Ok(Err(e)) => {
                                    state.add_log(
                                        "executor",
                                        &format!("Worktree failed for {plan}: {e}"),
                                        LogLevel::Warn,
                                    );
                                    None
                                }
                                Err(e) => {
                                    warn!("Worktree spawn_blocking panicked for {plan}: {e}");
                                    None
                                }
                            };

                            if let Some(ref wd_path) = wd {
                                let crates_count = std::fs::read_dir(wd_path.join("crates"))
                                    .map(|d| d.count())
                                    .unwrap_or(0);
                                let apps_count = std::fs::read_dir(wd_path.join("apps"))
                                    .map(|d| d.count())
                                    .unwrap_or(0);
                                info!("review[{plan}] worktree contents: crates={crates_count} apps={apps_count}");
                                state.add_log(
                                    "executor",
                                    &format!(
                                        "Worktree for {plan}: {} crates, {} apps",
                                        crates_count, apps_count,
                                    ),
                                    LogLevel::Info,
                                );
                                match crate::orchestrator::context::regenerate_workspace_map(
                                    wd_path,
                                ) {
                                    Ok(ref map) => {
                                        let line_count = map.lines().count();
                                        info!("review[{plan}] workspace map regenerated: {line_count} lines");
                                        state.add_log("executor", &format!(
                                            "Refreshed workspace map for {plan} from worktree ({line_count} lines)"
                                        ), LogLevel::Info);
                                    }
                                    Err(e) => state.add_log(
                                        "executor",
                                        &format!("Failed to refresh workspace map for {plan}: {e}"),
                                        LogLevel::Warn,
                                    ),
                                }
                            }

                            // Sync the executor's agent budget before spawning review agents.
                            sync_agent_budget(executor, &state);

                            // Guard: no other review agent for this plan still active.
                            // First, clear stale entries — marked active but process already gone from pool.
                            let review_roles = |r: &AgentRole| {
                                matches!(
                                    r,
                                    AgentRole::Architect
                                        | AgentRole::Auditor
                                        | AgentRole::Scribe
                                        | AgentRole::Critic
                                        | AgentRole::QuickReviewer
                                )
                            };
                            for pa in state.parallel_agents.iter_mut().filter(|pa| {
                                pa.plan == *plan && pa.active && review_roles(&pa.role)
                            }) {
                                let aid = AgentInstanceId::new(pa.role, pa.instance_id.clone());
                                if !pool.is_spawned(&aid) {
                                    warn!(
                                        "review[{plan}] clearing stale agent entry: {} not in pool",
                                        pa.instance_id
                                    );
                                    pa.active = false;
                                }
                            }
                            let active_review = state
                                .parallel_agents
                                .iter()
                                .any(|pa| pa.plan == *plan && pa.active && review_roles(&pa.role));
                            if active_review {
                                warn!("review[{plan}] overlap detected — another review agent is still active, skipping spawn");
                                state.add_log("executor", &format!("Review overlap for {plan} — waiting for active review to finish"), LogLevel::Warn);
                            } else if !executor.can_spawn_more() {
                                warn!(
                                    "review[{plan}] deferred — agent budget full ({} active)",
                                    active_agent_count(&state)
                                );
                                state.add_log(
                                    "executor",
                                    &format!("Review deferred for {plan} — agent budget full"),
                                    LogLevel::Warn,
                                );
                            } else if pipeline.use_quick_review {
                                // Standard plan: single QuickReviewer pass (combines arch+audit concerns)
                                match crate::orchestrator::prompts::quick_reviewer_prompt(
                                    &config.repo_root,
                                    pi,
                                    iteration,
                                    wd.as_deref(),
                                ) {
                                    Ok(prompt) => {
                                        info!("review[{plan}] quick-reviewer prompt_len={} iteration={}", prompt.len(), iteration);
                                        state.add_log("executor", &format!(
                                            "QuickReviewer for {plan}: iter={iteration} prompt={}chars", prompt.len(),
                                        ), LogLevel::Info);
                                        let effort = state
                                            .config
                                            .effort_for(AgentRole::QuickReviewer)
                                            .label();
                                        let warm_iid_opt =
                                            executor.get_active_reviewer(plan).map(|s| s.clone());
                                        let (iid, used_warm) = if let Some(ref warm_iid) =
                                            warm_iid_opt
                                        {
                                            if pool
                                                .promote_warm(AgentRole::QuickReviewer, warm_iid)
                                                .await
                                                .is_some()
                                            {
                                                executor.clear_active_reviewer(plan);
                                                (warm_iid.clone(), true)
                                            } else {
                                                executor.clear_active_reviewer(plan); // evicted; don't retry
                                                (format!("quick:{plan}"), false)
                                            }
                                        } else {
                                            (format!("quick:{plan}"), false)
                                        };
                                        let aid = AgentInstanceId::new(
                                            AgentRole::QuickReviewer,
                                            iid.clone(),
                                        );
                                        if used_warm
                                            || pool
                                                .spawn_instance(
                                                    aid.clone(),
                                                    wd,
                                                    effort,
                                                    state
                                                        .config
                                                        .model_for(AgentRole::QuickReviewer),
                                                )
                                                .await
                                                .is_ok()
                                        {
                                            if !used_warm {
                                                pool.set_thread_id(&aid, None);
                                            }
                                            if let Err(e) = pool
                                                .turn_start(
                                                    &aid,
                                                    &prompt,
                                                    state
                                                        .config
                                                        .model_for(AgentRole::QuickReviewer),
                                                )
                                                .await
                                            {
                                                tracing::error!("Failed to start quick-reviewer turn for {plan}: {e}");
                                                state.add_log("executor", &format!("QuickReviewer turn_start failed for {plan}: {e}"), LogLevel::Error);
                                            }
                                            state.parallel_agents.push(
                                                crate::state::ParallelAgentState {
                                                    instance_id: iid,
                                                    role: AgentRole::QuickReviewer,
                                                    plan: plan.clone(),
                                                    task: "quick".to_string(),
                                                    output: String::new(),
                                                    input_tokens: 0,
                                                    output_tokens: 0,
                                                    cost_usd: 0.0,
                                                    active: true,
                                                    finished_at: None,
                                                    model: String::new(),
                                                    turn_started: false,
                                                    render_cache: Default::default(),
                                                },
                                            );
                                            let mut pending = std::collections::HashSet::new();
                                            pending.insert(AgentRole::QuickReviewer);
                                            state
                                                .plan_pending_reviews
                                                .insert(plan.clone(), pending);
                                            state.plan_review_stage.insert(
                                                plan.clone(),
                                                crate::state::ReviewStage::ReviewerPending,
                                            );
                                            state.add_log(
                                                "executor",
                                                &format!(
                                                    "QuickReviewer started for {plan} [model={}]",
                                                    state
                                                        .config
                                                        .model_for(AgentRole::QuickReviewer)
                                                        .unwrap_or("default")
                                                ),
                                                LogLevel::Info,
                                            );
                                        } else {
                                            let merge_actions =
                                                executor.handle_plan_reviews_passed(plan);
                                            Box::pin(execute_actions(
                                                merge_actions,
                                                executor,
                                                pool,
                                                worktree_mgr,
                                                state,
                                                config,
                                                persistence,
                                                gate_tx,
                                                batch_branch,
                                                git_manager,
                                                spawn_ready_tx,
                                            ))
                                            .await?;
                                        }
                                    }
                                    Err(_) => {
                                        let merge_actions =
                                            executor.handle_plan_reviews_passed(plan);
                                        Box::pin(execute_actions(
                                            merge_actions,
                                            executor,
                                            pool,
                                            worktree_mgr,
                                            state,
                                            config,
                                            persistence,
                                            gate_tx,
                                            batch_branch,
                                            git_manager,
                                            spawn_ready_tx,
                                        ))
                                        .await?;
                                    }
                                }
                            } else if let Ok(prompt) =
                                crate::orchestrator::prompts::combined_reviewer_prompt(
                                    &config.repo_root,
                                    pi,
                                    iteration,
                                    wd.as_deref(),
                                )
                            {
                                // Complex plan: combined Reviewer → Scribe pipeline
                                info!(
                                    "review[{plan}] reviewer prompt_len={} iteration={} wd={:?}",
                                    prompt.len(),
                                    iteration,
                                    wd.as_ref().map(|p| p.display().to_string())
                                );
                                state.add_log(
                                    "executor",
                                    &format!(
                                        "Reviewer for {plan}: iter={iteration} prompt={}chars",
                                        prompt.len(),
                                    ),
                                    LogLevel::Info,
                                );
                                let effort = state.config.effort_for(AgentRole::Architect).label();
                                let warm_iid_opt =
                                    executor.get_active_reviewer(plan).map(|s| s.clone());
                                let (iid, used_warm) = if let Some(ref warm_iid) = warm_iid_opt {
                                    if pool
                                        .promote_warm(AgentRole::Architect, warm_iid)
                                        .await
                                        .is_some()
                                    {
                                        executor.clear_active_reviewer(plan);
                                        (warm_iid.clone(), true)
                                    } else {
                                        executor.clear_active_reviewer(plan);
                                        (format!("arch:{plan}"), false)
                                    }
                                } else {
                                    (format!("arch:{plan}"), false)
                                };
                                let aid = AgentInstanceId::new(AgentRole::Architect, iid.clone());
                                if used_warm
                                    || pool
                                        .spawn_instance(
                                            aid.clone(),
                                            wd,
                                            effort,
                                            state.config.model_for(AgentRole::Architect),
                                        )
                                        .await
                                        .is_ok()
                                {
                                    if !used_warm {
                                        pool.set_thread_id(&aid, None);
                                    }
                                    if let Err(e) = pool
                                        .turn_start(
                                            &aid,
                                            &prompt,
                                            state.config.model_for(AgentRole::Architect),
                                        )
                                        .await
                                    {
                                        tracing::error!(
                                            "Failed to start architect turn for {plan}: {e}"
                                        );
                                        state.add_log(
                                            "executor",
                                            &format!("Architect turn_start failed for {plan}: {e}"),
                                            LogLevel::Error,
                                        );
                                    }
                                    state
                                        .parallel_agents
                                        .push(crate::state::ParallelAgentState {
                                            instance_id: iid,
                                            role: AgentRole::Architect,
                                            plan: plan.clone(),
                                            task: "arch".to_string(),
                                            output: String::new(),
                                            input_tokens: 0,
                                            output_tokens: 0,
                                            cost_usd: 0.0,
                                            active: true,
                                            finished_at: None,
                                            model: String::new(),
                                            turn_started: false,
                                            render_cache: Default::default(),
                                        });
                                    let mut pending = std::collections::HashSet::new();
                                    pending.insert(AgentRole::Architect);
                                    state.plan_pending_reviews.insert(plan.clone(), pending);
                                    state.plan_review_stage.insert(
                                        plan.clone(),
                                        crate::state::ReviewStage::ReviewerPending,
                                    );
                                    state.add_log(
                                        "executor",
                                        &format!(
                                            "Reviewer started for {plan} [model={}]",
                                            state
                                                .config
                                                .model_for(AgentRole::Architect)
                                                .unwrap_or("default")
                                        ),
                                        LogLevel::Info,
                                    );
                                } else {
                                    // Spawn failed — skip to merge.
                                    let merge_actions = executor.handle_plan_reviews_passed(plan);
                                    Box::pin(execute_actions(
                                        merge_actions,
                                        executor,
                                        pool,
                                        worktree_mgr,
                                        state,
                                        config,
                                        persistence,
                                        gate_tx,
                                        batch_branch,
                                        git_manager,
                                        spawn_ready_tx,
                                    ))
                                    .await?;
                                }
                            } else {
                                // Prompt build failed — skip to merge.
                                let merge_actions = executor.handle_plan_reviews_passed(plan);
                                Box::pin(execute_actions(
                                    merge_actions,
                                    executor,
                                    pool,
                                    worktree_mgr,
                                    state,
                                    config,
                                    persistence,
                                    gate_tx,
                                    batch_branch,
                                    git_manager,
                                    spawn_ready_tx,
                                ))
                                .await?;
                            }
                        }
                    } else {
                        // Plan not found — skip to merge.
                        let merge_actions = executor.handle_plan_reviews_passed(plan);
                        Box::pin(execute_actions(
                            merge_actions,
                            executor,
                            pool,
                            worktree_mgr,
                            state,
                            config,
                            persistence,
                            gate_tx,
                            batch_branch,
                            git_manager,
                            spawn_ready_tx,
                        ))
                        .await?;
                    }
                }
            }
            ExecutorAction::MergePlanToBatch { ref plan } => {
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "committing".to_string();
                }
                state
                    .plan_phase_started
                    .insert(plan.clone(), std::time::Instant::now());
                state.add_log(
                    "executor",
                    &format!("Merging {plan} to batch"),
                    LogLevel::Info,
                );

                // Phase validation: conductor checks review quality before merge proceeds
                {
                    let iteration = executor.plan_iteration(plan);
                    state.pending_phase_validation =
                        Some((plan.clone(), "reviews_passed".to_string()));
                    parallel_consult_conductor(
                        state, pool,
                        &format!("Phase validation: reviews passed for {plan} (iter {iteration}). Merge proceeding — any concerns?"),
                        &format!("Plan {plan} passed reviews and is about to merge to batch branch."),
                    ).await;
                }

                // Write pre-merge checkpoint so crash recovery can detect stale merges
                {
                    let (inp_tok, out_tok) = (
                        state
                            .parallel_agents
                            .iter()
                            .map(|p| p.input_tokens)
                            .sum::<u64>(),
                        state
                            .parallel_agents
                            .iter()
                            .map(|p| p.output_tokens)
                            .sum::<u64>(),
                    );
                    write_checkpoint(
                        &executor,
                        &persistence,
                        &worktree_mgr,
                        &batch_branch,
                        inp_tok,
                        out_tok,
                    );
                }

                // Move all blocking git operations to a background thread so the
                // TUI event loop stays responsive during merges.
                let plan_clone = plan.clone();
                let plan_fallback = plan.clone();
                let repo_root = config.repo_root.clone();
                let batch = batch_branch.to_string();
                let tx = gate_tx.clone();

                tokio::task::spawn_blocking(move || {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let merge_start = std::time::Instant::now();
                        let wt_mgr = crate::git::worktree::WorktreeManager::new(repo_root.clone());

                        // Commit agent work in the plan worktree
                        let commit_msg = format!(
                            "plan({}): {}",
                            plan_clone.split('-').next().unwrap_or(&plan_clone),
                            &plan_clone
                        );
                        let worktree_path =
                            wt_mgr.worktree_base().join(format!("plan-{}", &plan_clone));
                        if worktree_path.exists() {
                            let _ = crate::git::ops::run_git(&worktree_path, &["add", "-A"]);
                            let status = crate::git::ops::run_git(
                                &worktree_path,
                                &["status", "--porcelain"],
                            )
                            .unwrap_or_default();
                            if !status.trim().is_empty() {
                                let file_count =
                                    status.lines().filter(|l| !l.trim().is_empty()).count();
                                tracing::info!(
                                    "merge[{}]: committing {} files in worktree",
                                    plan_clone,
                                    file_count
                                );
                                if let Err(e) = crate::git::ops::run_git_with_plumbing_author(
                                    &worktree_path,
                                    &["commit", "-m", &commit_msg],
                                ) {
                                    tracing::warn!(
                                        "merge[{}]: worktree commit failed: {e}",
                                        plan_clone
                                    );
                                }
                            }
                        } else {
                            tracing::info!(
                                "merge[{}]: no worktree, committing in repo root",
                                plan_clone
                            );
                            let _ = crate::git::ops::run_git(&repo_root, &["add", "-A"]);
                            let status =
                                crate::git::ops::run_git(&repo_root, &["status", "--porcelain"])
                                    .unwrap_or_default();
                            if !status.trim().is_empty() {
                                let _ = crate::git::ops::run_git_with_plumbing_author(
                                    &repo_root,
                                    &["commit", "-m", &commit_msg],
                                );
                            }
                        }

                        // Count commits BEFORE cleanup (while worktree still exists)
                        let commit_count = if worktree_path.exists() {
                            crate::git::ops::run_git(
                                &worktree_path,
                                &["rev-list", "--count", "HEAD"],
                            )
                            .unwrap_or_default()
                            .trim()
                            .to_string()
                        } else {
                            "?".to_string()
                        };

                        // Merge plan worktree into batch branch
                        let wt_merge_path =
                            wt_mgr.worktree_base().join(format!("plan-{}", &plan_clone));
                        let mut merge_error: Option<String> = None;
                        if wt_merge_path.exists() {
                            tracing::info!("merge[{}]: merging via worktree", plan_clone);
                            let pw = crate::git::worktree::PlanWorktree {
                                path: wt_merge_path,
                                branch: format!("codex/plan/{}", &plan_clone),
                                plan_base: plan_clone.clone(),
                            };
                            if let Err(e) = wt_mgr.merge_plan_worktree(&pw, &batch) {
                                tracing::warn!("merge[{}]: merge failed: {e}", plan_clone);
                                merge_error = Some(format!("{e}"));
                            } else {
                                // Update main repo working tree if batch branch is checked out
                                let current = crate::git::ops::run_git(
                                    &repo_root,
                                    &["rev-parse", "--abbrev-ref", "HEAD"],
                                )
                                .unwrap_or_default();
                                if current.trim() == batch {
                                    let _ = crate::git::ops::run_git(
                                        &repo_root,
                                        &["reset", "--hard", "HEAD"],
                                    );
                                }
                                // Clean up worktree after successful merge
                                if let Err(e) = wt_mgr.cleanup_plan_worktree(&pw) {
                                    tracing::warn!(
                                        "merge[{}]: cleanup failed (non-fatal): {e}",
                                        plan_clone
                                    );
                                }
                            }
                        } else {
                            tracing::info!(
                                "merge[{}]: merging via branch (no worktree)",
                                plan_clone
                            );
                            let plan_branch = format!("codex/plan/{}", &plan_clone);
                            let message = format!("merge(batch): codex/plan/{}", &plan_clone);
                            if let Err(e) = crate::git::ops::run_git(
                                &repo_root,
                                &["merge", "--no-ff", "-m", &message, &plan_branch],
                            ) {
                                tracing::warn!("merge[{}]: branch merge failed: {e}", plan_clone);
                                merge_error = Some(format!("{e}"));
                            } else {
                                let _ = crate::git::ops::run_git(
                                    &repo_root,
                                    &["branch", "-d", &plan_branch],
                                );
                            }
                        }

                        let elapsed = merge_start.elapsed();
                        tracing::info!(
                            "merge[{}]: {} in {:.1}s",
                            plan_clone,
                            if merge_error.is_some() {
                                "FAILED"
                            } else {
                                "SUCCESS"
                            },
                            elapsed.as_secs_f64()
                        );

                        (plan_clone, merge_error, commit_count)
                    }));

                    match result {
                        Ok((plan, merge_error, commit_count)) => {
                            let _ = tx.send(GateCompletion::MergeComplete {
                                plan,
                                success: merge_error.is_none(),
                                error: merge_error,
                                commit_count,
                            });
                        }
                        Err(_) => {
                            tracing::error!("merge[{}]: blocking task panicked", plan_fallback);
                            let _ = tx.send(GateCompletion::MergeComplete {
                                plan: plan_fallback,
                                success: false,
                                error: Some("merge task panicked".to_string()),
                                commit_count: "0".to_string(),
                            });
                        }
                    }
                });
            }
            ExecutorAction::SpawnPrePlanner { ref plan } => {
                state.add_log("executor", &format!("Pre-planning {plan}"), LogLevel::Info);
                // NOTE: Don't mark_pre_planned until spawn succeeds

                let plan_num = plan.split('-').next().unwrap_or(plan);
                let pi = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan));

                if let Some(plan_info) = pi {
                    let instance_id = format!("pre-planner:{plan}");
                    let aid = AgentInstanceId::new(AgentRole::PrePlanner, instance_id.clone());
                    let effort = state.config.effort_for(AgentRole::PrePlanner).label();
                    match pool
                        .spawn_instance(
                            aid.clone(),
                            None,
                            effort,
                            state.config.model_for(AgentRole::PrePlanner),
                        )
                        .await
                    {
                        Ok(()) => {
                            let prompt = crate::orchestrator::prompts::pre_planner_prompt(
                                &config.repo_root,
                                &plan_info,
                            )
                            .unwrap_or_default();
                            pool.set_thread_id(&aid, None);
                            if let Err(e) = pool
                                .turn_start(
                                    &aid,
                                    &prompt,
                                    state.config.model_for(AgentRole::PrePlanner),
                                )
                                .await
                            {
                                tracing::error!("Failed to start pre-planner turn for {plan}: {e}");
                                state.add_log(
                                    "executor",
                                    &format!("PrePlanner turn_start failed for {plan}: {e}"),
                                    LogLevel::Error,
                                );
                            }
                            executor.mark_pre_planned(plan); // Only after success
                        }
                        Err(e) => {
                            tracing::error!("Failed to spawn pre-planner for {plan}: {e}");
                            state.add_log(
                                "executor",
                                &format!("Pre-planner spawn failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                            // Don't mark as pre-planned so it can be retried
                        }
                    }
                }
            }
            ExecutorAction::SpawnRefactorer { ref batch_branch } => {
                state.add_log(
                    "executor",
                    &format!("Refactoring on {batch_branch}"),
                    LogLevel::Info,
                );

                let completed: Vec<String> = executor.completed_plan_names();
                let instance_id = "refactorer:batch".to_string();
                let aid = AgentInstanceId::new(AgentRole::Refactorer, instance_id.clone());
                let effort = state.config.effort_for(AgentRole::Refactorer).label();
                if let Ok(()) = pool
                    .spawn_instance(
                        aid.clone(),
                        None,
                        effort,
                        state.config.model_for(AgentRole::Refactorer),
                    )
                    .await
                {
                    let prompt = crate::orchestrator::prompts::batch_refactorer_prompt(
                        batch_branch,
                        &completed,
                    );
                    pool.set_thread_id(&aid, None);
                    if let Err(e) = pool
                        .turn_start(&aid, &prompt, state.config.model_for(AgentRole::Refactorer))
                        .await
                    {
                        tracing::error!("Failed to start refactorer turn: {e}");
                        state.add_log(
                            "executor",
                            &format!("Refactorer turn_start failed: {e}"),
                            LogLevel::Error,
                        );
                    }
                } else {
                    executor.handle_refactoring_complete();
                }
            }
            ExecutorAction::SpawnDocVerifier { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Doc verification for {plan}"),
                    LogLevel::Info,
                );

                let plan_num = plan.split('-').next().unwrap_or(plan);
                let pi = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan));

                if let Some(plan_info) = pi {
                    let instance_id = format!("doc-verifier:{plan}");
                    let aid = AgentInstanceId::new(AgentRole::DocVerifier, instance_id.clone());
                    let effort = state.config.effort_for(AgentRole::DocVerifier).label();
                    if let Ok(()) = pool
                        .spawn_instance(
                            aid.clone(),
                            None,
                            effort,
                            state.config.model_for(AgentRole::DocVerifier),
                        )
                        .await
                    {
                        // Get recent diff for the doc verifier
                        let diff = git_manager.diff_branch(batch_branch).unwrap_or_default();
                        let prompt = crate::orchestrator::prompts::doc_verifier_prompt(
                            &config.repo_root,
                            &plan_info,
                            &diff,
                        )
                        .unwrap_or_default();
                        pool.set_thread_id(&aid, None);
                        if let Err(e) = pool
                            .turn_start(
                                &aid,
                                &prompt,
                                state.config.model_for(AgentRole::DocVerifier),
                            )
                            .await
                        {
                            tracing::error!("Failed to start doc-verifier turn for {plan}: {e}");
                            state.add_log(
                                "executor",
                                &format!("DocVerifier turn_start failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                        }
                    }
                }
            }
            ExecutorAction::RunIntegrationTests { ref batch_branch } => {
                state.add_log(
                    "executor",
                    &format!("Running integration tests on {batch_branch}"),
                    LogLevel::Info,
                );

                let instance_id = "integration-tester:batch".to_string();
                let aid = AgentInstanceId::new(AgentRole::IntegrationTester, instance_id.clone());
                let effort = state
                    .config
                    .effort_for(AgentRole::IntegrationTester)
                    .label();
                if let Ok(()) = pool
                    .spawn_instance(
                        aid.clone(),
                        None,
                        effort,
                        state.config.model_for(AgentRole::IntegrationTester),
                    )
                    .await
                {
                    let completed: Vec<String> = executor
                        .active_plans()
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    let prompt = crate::orchestrator::prompts::integration_tester_prompt(
                        batch_branch,
                        &completed,
                    );
                    pool.set_thread_id(&aid, None);
                    if let Err(e) = pool
                        .turn_start(
                            &aid,
                            &prompt,
                            state.config.model_for(AgentRole::IntegrationTester),
                        )
                        .await
                    {
                        tracing::error!("Failed to start integration-tester turn: {e}");
                        state.add_log(
                            "executor",
                            &format!("IntegrationTester turn_start failed: {e}"),
                            LogLevel::Error,
                        );
                    }
                } else {
                    executor.handle_integration_tests_complete();
                }
            }
            ExecutorAction::ResolveMergeConflict { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Resolving merge conflict for {plan}"),
                    LogLevel::Warn,
                );

                let instance_id = format!("merge-resolver:{plan}");
                let aid = AgentInstanceId::new(AgentRole::MergeResolver, instance_id.clone());
                let effort = state.config.effort_for(AgentRole::MergeResolver).label();
                if let Ok(()) = pool
                    .spawn_instance(
                        aid.clone(),
                        None,
                        effort,
                        state.config.model_for(AgentRole::MergeResolver),
                    )
                    .await
                {
                    // Get conflicting files
                    let conflicts = crate::git::ops::run_git(
                        &config.repo_root,
                        &["diff", "--name-only", "--diff-filter=U"],
                    )
                    .unwrap_or_default();
                    let conflicting_files: Vec<String> =
                        conflicts.lines().map(|s| s.to_string()).collect();
                    let prompt = crate::orchestrator::prompts::merge_resolver_prompt(
                        plan,
                        &conflicting_files,
                        batch_branch,
                    );
                    pool.set_thread_id(&aid, None);
                    if let Err(e) = pool
                        .turn_start(
                            &aid,
                            &prompt,
                            state.config.model_for(AgentRole::MergeResolver),
                        )
                        .await
                    {
                        tracing::error!("Failed to start merge-resolver turn for {plan}: {e}");
                        state.add_log(
                            "executor",
                            &format!("MergeResolver turn_start failed for {plan}: {e}"),
                            LogLevel::Error,
                        );
                    }
                }
            }
            ExecutorAction::ReGatePlan { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Re-gating plan {plan} (invariant cascade)"),
                    LogLevel::Info,
                );
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "gating".to_string();
                }
                let actions = vec![ExecutorAction::RunPlanGates { plan: plan.clone() }];
                Box::pin(execute_actions(
                    actions,
                    executor,
                    pool,
                    worktree_mgr,
                    state,
                    config,
                    persistence,
                    gate_tx,
                    batch_branch,
                    git_manager,
                    spawn_ready_tx,
                ))
                .await?;
            }
            ExecutorAction::SpawnImplementer { ref plan } => {
                // Express mode: spawn a single implementer agent (no strategist).
                state.add_log(
                    "executor",
                    &format!("Express: spawning implementer for {plan}"),
                    LogLevel::Info,
                );
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "implementer".to_string();
                }
                state
                    .plan_phase_started
                    .insert(plan.clone(), std::time::Instant::now());

                let plan_num = plan.split('-').next().unwrap_or(plan);
                let pi = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan));

                if let Some(ref plan_info) = pi {
                    let iteration = executor.plan_iteration(plan);
                    // Generate static brief (replaces strategist)
                    let static_brief = crate::orchestrator::prompts::generate_static_brief(
                        &config.repo_root,
                        plan_info,
                    )
                    .unwrap_or_default();

                    match crate::orchestrator::prompts::express_implementer_prompt(
                        &config.repo_root,
                        plan_info,
                        &static_brief,
                        iteration,
                        None,
                    ) {
                        Ok(prompt) => {
                            let iid = format!("express-impl:{plan}");
                            let aid = AgentInstanceId::new(AgentRole::Implementer, iid.clone());
                            let effort = state.config.effort_for(AgentRole::Implementer).label();
                            let worktree_fallback =
                                worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                            let wt_mgr = worktree_mgr.clone();
                            let plan_clone = plan.clone();
                            let batch_clone = batch_branch.to_string();
                            let wd = match tokio::task::spawn_blocking(move || {
                                wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                            })
                            .await
                            {
                                Ok(Ok(pw)) => Some(pw.path),
                                Ok(Err(_)) | Err(_) => {
                                    if worktree_fallback.exists() {
                                        Some(worktree_fallback)
                                    } else {
                                        None
                                    }
                                }
                            };
                            if pool
                                .spawn_instance(
                                    aid.clone(),
                                    wd,
                                    effort,
                                    state.config.model_for(AgentRole::Implementer),
                                )
                                .await
                                .is_ok()
                            {
                                pool.set_thread_id(&aid, None);
                                if let Err(e) = pool
                                    .turn_start(
                                        &aid,
                                        &prompt,
                                        state.config.model_for(AgentRole::Implementer),
                                    )
                                    .await
                                {
                                    tracing::error!(
                                        "Express implementer turn_start failed for {plan}: {e}"
                                    );
                                    state.add_log(
                                        "executor",
                                        &format!("Express implementer failed for {plan}: {e}"),
                                        LogLevel::Error,
                                    );
                                }
                                state
                                    .parallel_agents
                                    .push(crate::state::ParallelAgentState {
                                        instance_id: iid,
                                        role: AgentRole::Implementer,
                                        plan: plan.clone(),
                                        task: "express-impl".to_string(),
                                        output: String::new(),
                                        input_tokens: 0,
                                        output_tokens: 0,
                                        cost_usd: 0.0,
                                        active: true,
                                        finished_at: None,
                                        model: String::new(),
                                        turn_started: false,
                                        render_cache: Default::default(),
                                    });
                                // Mark the __whole__ task as in-flight so schedule_next() tracks budget
                                let whole_id = crate::orchestrator::GlobalTaskId {
                                    plan: plan.clone(),
                                    task: "__whole__".to_string(),
                                };
                                executor
                                    .record_task_started(whole_id, format!("express-impl:{plan}"));
                            } else {
                                // Spawn failed — release the _pending_: budget placeholder and fail the plan
                                executor.release_pending_sentinel(plan);
                                state.add_log(
                                    "executor",
                                    &format!("Express implementer spawn failed for {plan}"),
                                    LogLevel::Error,
                                );
                                state.plan_doc_revisions.remove(plan);
                                let actions = executor.handle_plan_gates_failed(plan);
                                Box::pin(execute_actions(
                                    actions,
                                    executor,
                                    pool,
                                    worktree_mgr,
                                    state,
                                    config,
                                    persistence,
                                    gate_tx,
                                    batch_branch,
                                    git_manager,
                                    spawn_ready_tx,
                                ))
                                .await?;
                            }
                        }
                        Err(e) => {
                            state.add_log(
                                "executor",
                                &format!("Express prompt failed for {plan}: {e}"),
                                LogLevel::Warn,
                            );
                        }
                    }
                } else {
                    state.add_log(
                        "executor",
                        &format!("Plan {plan} not found — skipping express implementer"),
                        LogLevel::Warn,
                    );
                }
            }
            ExecutorAction::AutoFixErrors {
                ref plan,
                ref errors,
            } => {
                // Express mode: lightweight targeted fix agent after gate failure.
                sync_agent_budget(executor, &state);
                state.add_log(
                    "executor",
                    &format!("Express: auto-fixing errors for {plan}"),
                    LogLevel::Info,
                );
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "auto-fix".to_string();
                }
                let worktree_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                let working_dir = if worktree_path.exists() {
                    worktree_path
                } else {
                    config.repo_root.clone()
                };

                // Collect affected files from error output
                let affected_files: Vec<String> = errors
                    .lines()
                    .filter(|l| l.contains("  --> "))
                    .filter_map(|l| l.trim().strip_prefix("--> "))
                    .filter_map(|l| l.split(':').next())
                    .map(|s| s.to_string())
                    .collect();

                let plan_num = plan.split('-').next().unwrap_or(plan);
                let pi = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan));

                if let Some(ref plan_info) = pi {
                    if !executor.can_spawn_more() {
                        state.add_log(
                            "executor",
                            &format!(
                                "AutoFixer deferred for {plan} — agent budget full ({} active)",
                                active_agent_count(&state)
                            ),
                            LogLevel::Warn,
                        );
                    } else {
                        let prompt = crate::orchestrator::prompts::auto_fix_prompt(
                            plan_info,
                            errors,
                            &affected_files,
                        );
                        let auto_fix_model = state.config.auto_fix_model.clone();
                        let auto_fix_model_ref = Some(auto_fix_model.as_str());
                        let aid =
                            AgentInstanceId::new(AgentRole::AutoFixer, format!("auto-fix:{plan}"));
                        let effort = state
                            .config
                            .effort_for(AgentRole::AutoFixer)
                            .label()
                            .to_string();
                        if pool
                            .spawn_instance(
                                aid.clone(),
                                Some(working_dir),
                                &effort,
                                auto_fix_model_ref,
                            )
                            .await
                            .is_ok()
                        {
                            pool.set_thread_id(&aid, None);
                            if let Err(e) = pool.turn_start(&aid, &prompt, auto_fix_model_ref).await
                            {
                                tracing::error!("Auto-fixer turn_start failed for {plan}: {e}");
                                state.add_log(
                                    "executor",
                                    &format!("AutoFixer turn_start failed for {plan}: {e}"),
                                    LogLevel::Error,
                                );
                            }
                            state
                                .parallel_agents
                                .push(crate::state::ParallelAgentState {
                                    instance_id: format!("auto-fix:{plan}"),
                                    role: AgentRole::AutoFixer,
                                    plan: plan.clone(),
                                    task: "auto-fix".to_string(),
                                    output: String::new(),
                                    input_tokens: 0,
                                    output_tokens: 0,
                                    cost_usd: 0.0,
                                    active: true,
                                    finished_at: None,
                                    model: String::new(),
                                    turn_started: false,
                                    render_cache: Default::default(),
                                });
                        } else {
                            // Spawn failed — fail the plan
                            state.add_log(
                                "executor",
                                &format!("AutoFixer spawn failed for {plan} — failing plan"),
                                LogLevel::Error,
                            );
                            if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                                ps.phase = crate::orchestrator::PlanPhase::Failed(
                                    "auto-fixer spawn failed".to_string(),
                                );
                            }
                        }
                    } // can_spawn_more else
                } else {
                    state.add_log(
                        "executor",
                        &format!("Plan {plan} not found for auto-fix"),
                        LogLevel::Warn,
                    );
                }
            }
            ExecutorAction::DiagnoseError {
                ref plan,
                ref gate_output,
            } => {
                state.add_log(
                    "executor",
                    &format!("Diagnosing errors for {plan}"),
                    LogLevel::Info,
                );
                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == *plan) {
                    entry.phase = "error-diagnosis".to_string();
                }
                let worktree_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                let working_dir = if worktree_path.exists() {
                    worktree_path
                } else {
                    config.repo_root.clone()
                };
                // Collect affected files from the gate output (files mentioned in error lines)
                let affected_files: Vec<String> = gate_output
                    .lines()
                    .filter(|l| l.contains("  --> "))
                    .filter_map(|l| l.trim().strip_prefix("--> "))
                    .filter_map(|l| l.split(':').next())
                    .map(|s| s.to_string())
                    .collect();
                let prompt = crate::orchestrator::prompts::error_diagnoser_prompt(
                    plan,
                    gate_output,
                    &affected_files,
                );
                let aid = AgentInstanceId::new(AgentRole::ErrorDiagnoser, format!("{plan}-diag"));
                let effort = state
                    .config
                    .effort_for(AgentRole::ErrorDiagnoser)
                    .label()
                    .to_string();
                if let Err(e) = pool
                    .spawn_instance(
                        aid.clone(),
                        Some(working_dir),
                        &effort,
                        state.config.model_for(AgentRole::ErrorDiagnoser),
                    )
                    .await
                {
                    tracing::error!("Failed to spawn error-diagnoser for {plan}: {e}");
                    state.add_log(
                        "executor",
                        &format!("ErrorDiagnoser spawn failed for {plan}: {e}"),
                        LogLevel::Error,
                    );
                } else if let Err(e) = pool
                    .turn_start(
                        &aid,
                        &prompt,
                        state.config.model_for(AgentRole::ErrorDiagnoser),
                    )
                    .await
                {
                    tracing::error!("Failed to start error-diagnoser turn for {plan}: {e}");
                    state.add_log(
                        "executor",
                        &format!("ErrorDiagnoser turn_start failed for {plan}: {e}"),
                        LogLevel::Error,
                    );
                    pool.kill_instance(&aid).await;
                }
            }
            ExecutorAction::ValidateDependencies { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Validating dependencies for {plan}"),
                    LogLevel::Info,
                );
                let plan_num = plan.split('-').next().unwrap_or(plan);
                if let Some(pi) = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan))
                {
                    if let Ok(prompt) = crate::orchestrator::prompts::dependency_validator_prompt(
                        &config.repo_root,
                        &pi,
                    ) {
                        let aid = AgentInstanceId::new(
                            AgentRole::DependencyValidator,
                            format!("{plan}-depv"),
                        );
                        let effort = state
                            .config
                            .effort_for(AgentRole::DependencyValidator)
                            .label()
                            .to_string();
                        if let Err(e) = pool
                            .spawn_instance(
                                aid.clone(),
                                None,
                                &effort,
                                state.config.model_for(AgentRole::DependencyValidator),
                            )
                            .await
                        {
                            tracing::error!("Failed to spawn dependency-validator for {plan}: {e}");
                            state.add_log(
                                "executor",
                                &format!("DependencyValidator spawn failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                        } else if let Err(e) = pool
                            .turn_start(
                                &aid,
                                &prompt,
                                state.config.model_for(AgentRole::DependencyValidator),
                            )
                            .await
                        {
                            tracing::error!(
                                "Failed to start dependency-validator turn for {plan}: {e}"
                            );
                            state.add_log(
                                "executor",
                                &format!("DependencyValidator turn_start failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                            pool.kill_instance(&aid).await;
                        }
                    }
                }
            }
            ExecutorAction::ExtractPatterns { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Extracting patterns for {plan}"),
                    LogLevel::Info,
                );
                let plan_num = plan.split('-').next().unwrap_or(plan);
                if let Some(pi) = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[plan_num.to_string()],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan))
                {
                    if let Ok(prompt) = crate::orchestrator::prompts::pattern_extractor_prompt(
                        &config.repo_root,
                        &pi,
                    ) {
                        let aid = AgentInstanceId::new(
                            AgentRole::PatternExtractor,
                            format!("{plan}-patn"),
                        );
                        let effort = state
                            .config
                            .effort_for(AgentRole::PatternExtractor)
                            .label()
                            .to_string();
                        if let Err(e) = pool
                            .spawn_instance(
                                aid.clone(),
                                None,
                                &effort,
                                state.config.model_for(AgentRole::PatternExtractor),
                            )
                            .await
                        {
                            tracing::error!("Failed to spawn pattern-extractor for {plan}: {e}");
                            state.add_log(
                                "executor",
                                &format!("PatternExtractor spawn failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                        } else if let Err(e) = pool
                            .turn_start(
                                &aid,
                                &prompt,
                                state.config.model_for(AgentRole::PatternExtractor),
                            )
                            .await
                        {
                            tracing::error!(
                                "Failed to start pattern-extractor turn for {plan}: {e}"
                            );
                            state.add_log(
                                "executor",
                                &format!("PatternExtractor turn_start failed for {plan}: {e}"),
                                LogLevel::Error,
                            );
                            pool.kill_instance(&aid).await;
                        }
                    }
                }
            }
            ExecutorAction::RunPostMergeRegression { ref batch_branch } => {
                state.add_log(
                    "executor",
                    "Running post-merge regression tests",
                    LogLevel::Info,
                );
                let repo = config.repo_root.clone();
                let tx = gate_tx.clone();
                tokio::spawn(async move {
                    let result =
                        crate::orchestrator::gates::post_merge_regression_gate(&repo, 900).await;
                    let _ = tx.send(GateCompletion::PostMerge {
                        plan: "__regression__".to_string(),
                        result,
                    });
                });
            }
            ExecutorAction::PlanTimeout { ref plan } => {
                state.add_log(
                    "executor",
                    &format!(
                        "Plan {} exceeded wall-clock timeout — consulting conductor",
                        plan
                    ),
                    LogLevel::Warn,
                );
                parallel_consult_conductor(
                    state,
                    pool,
                    &format!("Plan {plan} has exceeded its 45-minute wall-clock timeout."),
                    "Options: FORCE_ADVANCE, RESTART, or extend timeout.",
                )
                .await;
                // Force-advance after timeout if conductor doesn't respond
                if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                    ps.phase = PlanPhase::Failed("wall-clock timeout".to_string());
                }
                // Clear merge_in_progress if this plan was the currently-merging one
                if executor.currently_merging.as_deref() == Some(plan.as_str()) {
                    executor.merge_in_progress = false;
                    executor.currently_merging = None;
                    state.add_log(
                        "executor",
                        &format!("Cleared merge_in_progress for timed-out plan {plan}"),
                        LogLevel::Warn,
                    );
                }
            }
            ExecutorAction::ForceAdvancePlan {
                ref plan,
                ref reason,
            } => {
                state.add_log(
                    "executor",
                    &format!("Force-advancing plan {plan}: {reason}"),
                    LogLevel::Warn,
                );
                if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                    ps.phase = PlanPhase::Failed(reason.clone());
                }
                // Clear merge_in_progress if this plan was the currently-merging one
                if executor.currently_merging.as_deref() == Some(plan.as_str()) {
                    executor.merge_in_progress = false;
                    executor.currently_merging = None;
                    state.add_log(
                        "executor",
                        &format!("Cleared merge_in_progress for force-advanced plan {plan}"),
                        LogLevel::Warn,
                    );
                }
            }
            ExecutorAction::CleanRetryPlan { ref plan } => {
                state.add_log(
                    "executor",
                    &format!("Clean retry: removing worktree + branch for {plan}"),
                    LogLevel::Warn,
                );

                // Kill any active agents for this plan
                let plan_agents: Vec<(AgentRole, String)> = state
                    .parallel_agents
                    .iter()
                    .filter(|p| p.plan == *plan && p.active)
                    .map(|p| (p.role, p.instance_id.clone()))
                    .collect();
                for (role, iid) in &plan_agents {
                    let aid = AgentInstanceId::new(*role, iid.clone());
                    pool.kill_instance(&aid).await;
                }

                // Remove parallel_agents entries for this plan
                state.parallel_agents.retain(|p| p.plan != *plan);

                // Clean up the worktree and branch via git
                let branch = format!("codex/plan/{plan}");
                let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                let pw = crate::git::worktree::PlanWorktree {
                    path: wt_path,
                    branch,
                    plan_base: plan.clone(),
                };
                if let Err(e) = worktree_mgr.cleanup_plan_worktree(&pw) {
                    tracing::warn!(
                        "Failed to cleanup worktree for {plan}: {e} — continuing anyway"
                    );
                    // Force remove if cleanup failed
                    let _ = std::fs::remove_dir_all(&pw.path);
                    let _ = crate::git::ops::run_git(&config.repo_root, &["worktree", "prune"]);
                    let _ =
                        crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &pw.branch]);
                }

                state.add_log(
                    "executor",
                    &format!("Clean retry complete for {plan} — will reschedule"),
                    LogLevel::Info,
                );
            }
            ExecutorAction::PrepareContext {
                ref worktree,
                ref plan,
                iter,
                for_role,
            } => {
                state.add_log(
                    "executor",
                    &format!(
                        "Preparing context/in for plan {plan} iter={iter} role={}",
                        for_role.label()
                    ),
                    LogLevel::Info,
                );
                let prefix = plan.split('-').next().unwrap_or(plan).to_string();
                if let Some(pi) = crate::orchestrator::plan::discover_plans(
                    &config.repo_root.join("plans"),
                    &[prefix],
                )
                .ok()
                .and_then(|plans| plans.into_iter().find(|p| p.base == *plan))
                {
                    let artifact_root = config.repo_root.join("tmp/bardo-artifacts");
                    let _ = std::fs::create_dir_all(&artifact_root);
                    let registry_root = config.repo_root.join("plans/context/registry");
                    let _ = std::fs::create_dir_all(&registry_root);
                    let artifacts = crate::orchestrator::ArtifactStore::new(artifact_root);
                    let registry = crate::orchestrator::Registry::new(registry_root);
                    if let Err(e) = registry.init() {
                        state.add_log("executor", &format!("registry init: {e}"), LogLevel::Warn);
                    }
                    let injector = crate::orchestrator::inject::ContextInjector {
                        artifact_store: &artifacts,
                        registry: &registry,
                        repo_root: &config.repo_root,
                    };
                    let plan_deps: Vec<String> = pi
                        .frontmatter
                        .as_ref()
                        .map(|f| f.depends_on.clone())
                        .unwrap_or_default();
                    let inject_res = match for_role {
                        AgentRole::Architect
                        | AgentRole::Auditor
                        | AgentRole::Scribe
                        | AgentRole::Critic
                        | AgentRole::QuickReviewer => {
                            injector.inject_for_reviewer(worktree, &pi.num, iter, &plan_deps)
                        }
                        _ => injector.inject_for_implementer(worktree, &pi.num, iter, &plan_deps),
                    };
                    if let Err(e) = inject_res {
                        state.add_log(
                            "executor",
                            &format!("context/in injection failed for {plan}: {e}"),
                            LogLevel::Warn,
                        );
                    }
                } else {
                    state.add_log(
                        "executor",
                        &format!("PrepareContext: no PlanInfo for {plan}"),
                        LogLevel::Warn,
                    );
                }
            }
            ExecutorAction::CollectOutput { ref plan, .. } => {
                state.add_log(
                    "executor",
                    &format!("Collecting output for plan {plan}"),
                    LogLevel::Debug,
                );
            }
            ExecutorAction::StartReviewerInParallel {
                ref plan,
                ref instance_id,
            } => {
                // This action directs the event loop to start a reviewer turn immediately,
                // running in parallel with gates. The executor has already set active_reviewer_instance.
                // The event loop should spawn the reviewer agent and call turn_start.
                // This is a directive; the actual reviewer spawn happens in the agent handler.
                state.add_log(
                    "executor",
                    &format!("Starting reviewer for {plan} in parallel with gates"),
                    LogLevel::Info,
                );
                executor.set_active_reviewer(plan, instance_id.clone());
            }
            ExecutorAction::CancelActiveReviewer {
                ref plan,
                ref instance_id,
            } => {
                // Gates failed while reviewer was running in overlap. Interrupt and kill the reviewer.
                state.add_log(
                    "executor",
                    &format!("Cancelling reviewer for {plan} (gates failed)"),
                    LogLevel::Warn,
                );
                // Look up the actual role from parallel_agents by instance_id
                let role = state
                    .parallel_agents
                    .iter()
                    .find(|pa| pa.instance_id == *instance_id)
                    .map(|pa| pa.role)
                    .unwrap_or(AgentRole::QuickReviewer); // fallback to QuickReviewer if not found
                let aid = AgentInstanceId::new(role, instance_id.clone());
                pool.turn_interrupt(&aid).await.ok();
                pool.kill_instance(&aid).await;
                executor.clear_active_reviewer(plan);
            }
        }
    }

    // Split spawn actions into single-task and batch.
    let (single_spawn_actions, batch_spawn_actions): (Vec<_>, Vec<_>) = spawn_actions
        .into_iter()
        .partition(|a| matches!(a, ExecutorAction::SpawnTaskAgent { .. }));

    // Process SpawnTaskAgent actions concurrently (parallel cold-start spawning).
    if !single_spawn_actions.is_empty() {
        let spawn_actions = single_spawn_actions;
        // Phase 1: prepare data for every spawn (disk reads, prompt building) — sequential.
        struct SpawnReady {
            aid: AgentInstanceId,
            working_dir: PathBuf,
            prompt: String,
            effort: String,
            task_id: crate::orchestrator::GlobalTaskId,
            instance_id: String,
        }

        let mut ready: Vec<SpawnReady> = Vec::new();
        for action in spawn_actions {
            let ExecutorAction::SpawnTaskAgent {
                task_id,
                instance_id,
            } = action
            else {
                continue;
            };
            let plan_base = &task_id.plan;
            let task_str = &task_id.task;

            if !state.plans.iter().any(|p| p.base == *plan_base) {
                warn!("Plan {plan_base} not found in state");
                continue;
            }

            let checklist = crate::orchestrator::tasks::load_checklist(
                &config.repo_root,
                &plan_base.split('-').next().unwrap_or(plan_base),
            )
            .ok()
            .flatten();
            let task_opt = checklist
                .as_ref()
                .and_then(|cl| cl.tasks.iter().find(|t| t.id == *task_str).cloned());

            let pi = crate::orchestrator::plan::discover_plans(
                &config.repo_root.join("plans"),
                &[plan_base.split('-').next().unwrap_or(plan_base).to_string()],
            )
            .ok()
            .and_then(|plans| plans.into_iter().find(|p| p.base == *plan_base));

            let worktree_path = worktree_mgr
                .worktree_base()
                .join(format!("plan-{plan_base}"));
            let working_dir = if worktree_path.exists() {
                worktree_path.clone()
            } else {
                config.repo_root.clone()
            };

            if let Some(ref plan) = pi {
                let iter = executor.plan_iteration(plan_base);
                inject_implementer_context_in_worktree(
                    &config.repo_root,
                    &worktree_path,
                    &working_dir,
                    plan,
                    iter,
                );
            }

            info!("spawn[{plan_base}:{task_str}] working_dir={} worktree_exists={} task_found={} plan_found={}",
                working_dir.display(),
                worktree_path.exists(),
                task_opt.is_some(),
                pi.is_some(),
            );

            let prompt = if let (Some(ref task), Some(plan)) = (&task_opt, pi.as_ref()) {
                let brief_path = config
                    .repo_root
                    .join(format!("plans/context/briefs/{}-brief.md", plan.num));
                let brief = std::fs::read_to_string(&brief_path).unwrap_or_default();
                let all_tasks = checklist.as_ref().map(|cl| cl.tasks.as_slice());
                let review_feedback: Vec<String> = executor.get_review_feedback(plan_base).to_vec();
                info!("spawn[{plan_base}:{task_str}] files={:?} brief_len={} review_feedback_count={} all_tasks_count={}",
                    task.files, brief.len(), review_feedback.len(),
                    all_tasks.map(|t| t.len()).unwrap_or(0),
                );
                crate::orchestrator::prompts::task_implementer_prompt(
                    &config.repo_root,
                    plan,
                    &task,
                    &brief,
                    &review_feedback,
                    all_tasks,
                )
                .unwrap_or_else(|_| format!("Implement task {task_str} for plan {plan_base}."))
            } else {
                warn!("spawn[{plan_base}:{task_str}] FALLBACK PROMPT — task or plan not found");
                format!("Implement task {task_str} for plan {plan_base}.")
            };

            let effort = state
                .config
                .effort_for(AgentRole::Implementer)
                .label()
                .to_string();
            let model = state
                .config
                .model_for(AgentRole::Implementer)
                .unwrap_or("default");
            info!(
                "spawn[{plan_base}:{task_str}] model={model} effort={effort} prompt_len={}",
                prompt.len()
            );
            state.add_log(
                "executor",
                &format!(
                    "Spawning {task_str} in {} [model={model}, prompt={}chars]",
                    working_dir.display(),
                    prompt.len(),
                ),
                LogLevel::Info,
            );

            let aid = AgentInstanceId::new(AgentRole::Implementer, instance_id.clone());
            ready.push(SpawnReady {
                aid,
                working_dir,
                prompt,
                effort,
                task_id,
                instance_id,
            });
        }

        // Determine effective backend from the implementer model — same logic as spawn_instance.
        let impl_model = state
            .config
            .model_for(AgentRole::Implementer)
            .map(String::from);
        let impl_backend = impl_model
            .as_deref()
            .map(AgentBackend::from_model)
            .unwrap_or_else(|| AgentRole::Implementer.backend());

        // Phase 2: all spawns are cold starts (no warm pool).
        let cold_starts: Vec<SpawnReady> = ready;

        // Phase 3: spawn cold starts concurrently via JoinSet.
        // The closure always returns the task_id so failures can be attributed to the plan.
        if !cold_starts.is_empty() {
            let event_tx = pool.event_tx();
            type SpawnOutcome = (
                crate::orchestrator::GlobalTaskId,
                String,
                String,
                anyhow::Result<(AgentInstanceId, crate::agent::AgentConnection, PathBuf)>,
            );
            let mut join_set: tokio::task::JoinSet<SpawnOutcome> = tokio::task::JoinSet::new();

            // Cursor agents are resource-intensive; stagger spawns to avoid overload
            let cursor_semaphore = Arc::new(Semaphore::new(1));
            let mut spawn_delay_millis: u64 = 0;

            for sr in cold_starts {
                let tx = event_tx.clone();
                let role = sr.aid.role;
                let iid = sr.aid.instance.clone();
                let wd = sr.working_dir.clone();
                let effort = sr.effort.clone();
                let task_id = sr.task_id.clone();
                let instance_id = sr.instance_id.clone();
                let prompt = sr.prompt.clone();
                let fast_mode = state.config.fast_mode;
                let backend = impl_backend;
                let model = impl_model.clone();
                let fallback = state.config.fallback_model.clone();
                let cursor_sem = Arc::clone(&cursor_semaphore);

                // For cursor agents, add increasing delays to stagger initialization
                if matches!(backend, AgentBackend::Cursor) {
                    spawn_delay_millis += 1500; // 1.5s between each cursor spawn
                }
                let init_delay = spawn_delay_millis;

                join_set.spawn(async move {
                    // For Cursor backend: acquire semaphore + add init delay
                    let _permit = if matches!(backend, AgentBackend::Cursor) {
                        if init_delay > 0 {
                            tokio::time::sleep(Duration::from_millis(init_delay)).await;
                        }
                        Some(cursor_sem.acquire().await.ok())
                    } else {
                        None
                    };

                    tracing::info!(
                        "Cold-start spawn backend={:?} model={}",
                        backend,
                        model.as_deref().unwrap_or("(none)")
                    );
                    let result = tokio::time::timeout(
                        std::time::Duration::from_secs(90),
                        try_spawn_raw(
                            backend,
                            role,
                            &wd,
                            tx.clone(),
                            &effort,
                            iid.clone(),
                            fast_mode,
                            model.as_deref(),
                        ),
                    )
                    .await
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("agent spawn timed out after 90s")));

                    // Fallback: retry once with fallback model if configured and different
                    let result = match result {
                        Ok(v) => Ok(v),
                        Err(primary_err) => {
                            if let Some(ref fb) = fallback {
                                if fallback.as_deref() != model.as_deref() {
                                    tracing::warn!(
                                        role = %role,
                                        fallback_model = %fb,
                                        "Cold-start spawn failed, trying fallback: {primary_err}"
                                    );
                                    tokio::time::timeout(
                                        std::time::Duration::from_secs(90),
                                        try_spawn_raw(
                                            backend,
                                            role,
                                            &wd,
                                            tx,
                                            &effort,
                                            iid,
                                            fast_mode,
                                            Some(fb),
                                        ),
                                    )
                                    .await
                                    .unwrap_or_else(|_| {
                                        Err(anyhow::anyhow!(
                                            "fallback agent spawn timed out after 90s"
                                        ))
                                    })
                                } else {
                                    Err(primary_err)
                                }
                            } else {
                                Err(primary_err)
                            }
                        }
                    };
                    (task_id, instance_id, prompt, result)
                });
            }

            // Fire-and-forget: spawn results are sent to the select loop via spawn_ready_tx
            // so the TUI tick arm can fire while agents are initializing.
            let tx = spawn_ready_tx.clone();
            tokio::spawn(async move {
                while let Some(join_result) = join_set.join_next().await {
                    match join_result {
                        Ok((task_id, instance_id, prompt, result)) => {
                            let _ = tx.send(AgentSpawnReady::Single {
                                task_id,
                                instance_id,
                                prompt,
                                result,
                            });
                        }
                        Err(e) => {
                            warn!("Spawn task panicked: {e}");
                        }
                    }
                }
            });
        }
    }

    // Process SpawnTaskAgentBatch actions — one agent per plan-batch.
    if !batch_spawn_actions.is_empty() {
        // Phase 1: prepare data for each batch spawn.
        struct SpawnReadyBatch {
            aid: AgentInstanceId,
            working_dir: PathBuf,
            prompt: String,
            effort: String,
            task_ids: Vec<crate::orchestrator::GlobalTaskId>,
            plan_base: String,
            instance_id: String,
        }

        let mut batch_ready: Vec<SpawnReadyBatch> = Vec::new();
        for action in batch_spawn_actions {
            let ExecutorAction::SpawnTaskAgentBatch {
                plan: plan_base,
                tasks,
                instance_id,
            } = action
            else {
                continue;
            };

            if !state.plans.iter().any(|p| p.base == *plan_base) {
                warn!("Batch spawn: plan {plan_base} not found in state");
                continue;
            }

            let checklist = crate::orchestrator::tasks::load_checklist(
                &config.repo_root,
                &plan_base.split('-').next().unwrap_or(&plan_base),
            )
            .ok()
            .flatten();

            // Collect Task metadata for each task_id in the batch.
            let batch_tasks: Vec<crate::orchestrator::tasks::Task> = tasks
                .iter()
                .filter_map(|gid| {
                    checklist
                        .as_ref()
                        .and_then(|cl| cl.tasks.iter().find(|t| t.id == gid.task))
                        .cloned()
                })
                .collect();

            let pi = crate::orchestrator::plan::discover_plans(
                &config.repo_root.join("plans"),
                &[plan_base
                    .split('-')
                    .next()
                    .unwrap_or(&plan_base)
                    .to_string()],
            )
            .ok()
            .and_then(|plans| plans.into_iter().find(|p| p.base == *plan_base));

            let worktree_path = worktree_mgr
                .worktree_base()
                .join(format!("plan-{plan_base}"));
            let working_dir = if worktree_path.exists() {
                worktree_path.clone()
            } else {
                config.repo_root.clone()
            };

            if let Some(ref plan) = pi {
                let iter = executor.plan_iteration(&plan_base);
                inject_implementer_context_in_worktree(
                    &config.repo_root,
                    &worktree_path,
                    &working_dir,
                    plan,
                    iter,
                );
            }

            info!(
                "batch-spawn[{plan_base}] tasks={} working_dir={} plan_found={}",
                tasks.len(),
                working_dir.display(),
                pi.is_some()
            );

            let prompt = if let Some(plan) = pi.as_ref() {
                let brief_path = config
                    .repo_root
                    .join(format!("plans/context/briefs/{}-brief.md", plan.num));
                let brief = std::fs::read_to_string(&brief_path).unwrap_or_default();
                let all_tasks = checklist.as_ref().map(|cl| cl.tasks.as_slice());
                let review_feedback = executor.get_review_feedback(&plan_base).to_vec();
                crate::orchestrator::prompts::task_implementer_batch_prompt(
                    &config.repo_root,
                    plan,
                    &batch_tasks,
                    &brief,
                    &review_feedback,
                    all_tasks,
                )
                .unwrap_or_else(|_| format!("Implement all tasks for plan {plan_base}."))
            } else {
                warn!("batch-spawn[{plan_base}] plan not found — using fallback prompt");
                format!("Implement all tasks for plan {plan_base}.")
            };

            let effort = state
                .config
                .effort_for(AgentRole::Implementer)
                .label()
                .to_string();
            let model = state
                .config
                .model_for(AgentRole::Implementer)
                .unwrap_or("default");
            info!(
                "batch-spawn[{plan_base}] model={model} effort={effort} prompt_len={}",
                prompt.len()
            );
            state.add_log(
                "executor",
                &format!(
                    "Spawning batch ({} tasks) for {plan_base} [model={model}, prompt={}chars]",
                    tasks.len(),
                    prompt.len(),
                ),
                LogLevel::Info,
            );

            let aid = AgentInstanceId::new(AgentRole::Implementer, instance_id.clone());
            batch_ready.push(SpawnReadyBatch {
                aid,
                working_dir,
                prompt,
                effort,
                task_ids: tasks,
                plan_base,
                instance_id,
            });
        }

        // Determine backend for implementer model.
        let impl_model = state
            .config
            .model_for(AgentRole::Implementer)
            .map(String::from);
        let impl_backend = impl_model
            .as_deref()
            .map(AgentBackend::from_model)
            .unwrap_or_else(|| AgentRole::Implementer.backend());

        // Phase 2: cold-start each batch agent concurrently via JoinSet.
        type BatchSpawnOutcome = (
            Vec<crate::orchestrator::GlobalTaskId>,
            String, // plan_base
            String, // instance_id
            String, // prompt
            anyhow::Result<(AgentInstanceId, crate::agent::AgentConnection, PathBuf)>,
        );
        if !batch_ready.is_empty() {
            let event_tx = pool.event_tx();
            let mut join_set: tokio::task::JoinSet<BatchSpawnOutcome> = tokio::task::JoinSet::new();

            // Cursor agents are resource-intensive and fail when spawning concurrently.
            // Spawn them sequentially with delays to avoid overwhelming cursor's initialization.
            let cursor_semaphore = Arc::new(Semaphore::new(1));
            let mut spawn_delay_millis: u64 = 0;

            for sr in batch_ready {
                let tx = event_tx.clone();
                let role = sr.aid.role;
                let iid = sr.aid.instance.clone();
                let wd = sr.working_dir.clone();
                let effort = sr.effort.clone();
                let task_ids = sr.task_ids.clone();
                let plan_base = sr.plan_base.clone();
                let instance_id = sr.instance_id.clone();
                let prompt = sr.prompt.clone();
                let fast_mode = state.config.fast_mode;
                let backend = impl_backend;
                let model = impl_model.clone();
                let fallback = state.config.fallback_model.clone();
                let cursor_sem = Arc::clone(&cursor_semaphore);

                // For cursor agents, add increasing delays to stagger initialization
                if matches!(backend, AgentBackend::Cursor) {
                    spawn_delay_millis += 1500; // 1.5s between each cursor spawn
                }
                let init_delay = spawn_delay_millis;

                join_set.spawn(async move {
                    // For Cursor backend: acquire semaphore + add init delay
                    // (cursor app-server fails when multiple sessions initialize concurrently)
                    let _permit = if matches!(backend, AgentBackend::Cursor) {
                        if init_delay > 0 {
                            tokio::time::sleep(Duration::from_millis(init_delay)).await;
                        }
                        Some(cursor_sem.acquire().await.ok())
                    } else {
                        None
                    };

                    tracing::info!(
                        "Batch cold-start spawn backend={:?} model={}",
                        backend,
                        model.as_deref().unwrap_or("(none)")
                    );
                    let result = tokio::time::timeout(
                        std::time::Duration::from_secs(90),
                        try_spawn_raw(
                            backend,
                            role,
                            &wd,
                            tx.clone(),
                            &effort,
                            iid.clone(),
                            fast_mode,
                            model.as_deref(),
                        ),
                    )
                    .await
                    .unwrap_or_else(|_| {
                        Err(anyhow::anyhow!("batch agent spawn timed out after 90s"))
                    });

                    // Fallback: retry once with fallback model if configured and different
                    let result = match result {
                        Ok(v) => Ok(v),
                        Err(primary_err) => {
                            if let Some(ref fb) = fallback {
                                if fallback.as_deref() != model.as_deref() {
                                    tracing::warn!(
                                        role = %role,
                                        fallback_model = %fb,
                                        "Batch spawn failed, trying fallback: {primary_err}"
                                    );
                                    tokio::time::timeout(
                                        std::time::Duration::from_secs(90),
                                        try_spawn_raw(
                                            backend,
                                            role,
                                            &wd,
                                            tx,
                                            &effort,
                                            iid,
                                            fast_mode,
                                            Some(fb),
                                        ),
                                    )
                                    .await
                                    .unwrap_or_else(|_| {
                                        Err(anyhow::anyhow!(
                                            "fallback batch spawn timed out after 90s"
                                        ))
                                    })
                                } else {
                                    Err(primary_err)
                                }
                            } else {
                                Err(primary_err)
                            }
                        }
                    };
                    (task_ids, plan_base, instance_id, prompt, result)
                });
            }

            // Fire-and-forget: batch spawn results go to the select loop.
            let tx = spawn_ready_tx.clone();
            tokio::spawn(async move {
                while let Some(join_result) = join_set.join_next().await {
                    match join_result {
                        Ok((task_ids, plan_base, instance_id, prompt, result)) => {
                            let _ = tx.send(AgentSpawnReady::Batch {
                                task_ids,
                                plan_base,
                                instance_id,
                                prompt,
                                result,
                            });
                        }
                        Err(e) => {
                            warn!("Batch spawn task panicked: {e}");
                        }
                    }
                }
            });
        }
    }

    Ok(())
}

/// Consult the conductor LLM agent in parallel mode.
/// Sends a state snapshot and event description. Non-blocking — the response
/// is handled when the conductor's TurnCompleted fires.
pub(crate) async fn parallel_consult_conductor(
    state: &mut RunState,
    pool: &mut MultiAgentPool,
    event: &str,
    plan_summary: &str,
) {
    let conductor_iid = "conductor:llm".to_string();
    let aid = AgentInstanceId::new(AgentRole::Conductor, conductor_iid.clone());
    // Re-spawn the conductor if it exited (--print mode exits after each turn)
    if !pool.is_spawned(&aid) {
        let effort = state.config.effort_for(AgentRole::Conductor).label();
        if let Err(e) = pool
            .spawn_instance(
                aid.clone(),
                None,
                effort,
                state.config.model_for(AgentRole::Conductor),
            )
            .await
        {
            state.add_log(
                "conductor",
                &format!("Failed to re-spawn conductor: {e}"),
                LogLevel::Warn,
            );
            return;
        }
        pool.set_thread_id(&aid, None);
    }
    // Don't consult if still processing a previous message
    let is_active = state
        .parallel_agents
        .iter()
        .find(|p| p.instance_id == conductor_iid)
        .map(|p| p.active)
        .unwrap_or(false);
    if is_active {
        return;
    }

    // Build a snapshot of active plans and their states
    let active_plans: Vec<String> = state
        .plans
        .iter()
        .filter(|p| matches!(p.status, RunPlanStatus::Active))
        .map(|p| {
            let review_stage = state
                .plan_review_stage
                .get(&p.base)
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| p.phase.clone());
            let doc_revs = state.plan_doc_revisions.get(&p.base).copied().unwrap_or(0);
            format!(
                "  - {} [iter={} phase={} doc_revisions={}]",
                p.base, p.iteration, review_stage, doc_revs
            )
        })
        .collect();

    let msg = format!(
        "## Event\n{event}\n\n## Plan Details\n{plan_summary}\n\n## Active Plans\n{}\n\n## Agent Activity\n{} active agents\n\nWhat's your assessment?",
        active_plans.join("\n"),
        state.parallel_agents.iter().filter(|p| p.active).count(),
    );

    match pool
        .turn_start(&aid, &msg, state.config.model_for(AgentRole::Conductor))
        .await
    {
        Ok(()) => {
            if let Some(pa) = state
                .parallel_agents
                .iter_mut()
                .find(|p| p.instance_id == conductor_iid)
            {
                pa.active = true;
                pa.finished_at = None;
            }
        }
        Err(e) => {
            state.add_log(
                "conductor",
                &format!("Failed to consult conductor LLM: {e}"),
                LogLevel::Warn,
            );
        }
    }
}

/// Parallel execution entry point — uses ParallelExecutor + MultiAgentPool.
pub(crate) async fn run_parallel(config: AppConfig) -> Result<()> {
    // Channels
    let (agent_tx, mut agent_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let (orch_tx, _orch_rx) = mpsc::unbounded_channel::<OrchestratorEvent>();
    let (git_tx, mut git_rx) = mpsc::unbounded_channel::<crate::git::GitEvent>();

    // Initialize subsystems
    let orch_config = OrchestratorConfig {
        repo_root: config.repo_root.clone(),
        plans_dir: config.repo_root.join("plans"),
        no_review: config.no_review,
        skip_tests: config.skip_tests,
        max_iterations: config.max_iterations,
        batch_size: config.batch_size,
        model: config.model.clone(),
        no_docs: config.no_docs,
        max_parallel_plans: config.max_parallel_plans,
        ..OrchestratorConfig::new(config.repo_root.clone())
    };
    let mut orchestrator = Orchestrator::new(orch_config, orch_tx);
    orchestrator.discover_plans(&config.plan_specs)?;

    let mut pool = MultiAgentPool::new(config.repo_root.clone(), agent_tx);
    let (spawn_ready_tx, mut spawn_ready_rx) = mpsc::unbounded_channel::<AgentSpawnReady>();
    let git_manager = GitManager::new(config.repo_root.clone(), git_tx);
    let _auto_stash_session = AutoStashSession::new(&git_manager);
    let persistence = crate::state::persistence::PersistenceManager::new(&config.repo_root);
    persistence.ensure_dirs()?;
    persistence.write_pid()?;

    let batch_branch = format!("codex/batch/{}", config.batch_id);
    // In parallel mode all commits go through plan worktrees; the main working
    // tree must NOT switch branches, otherwise the user's files change under them.
    git_manager.ensure_batch_branch(&batch_branch)?;

    // Worktree manager — validate and repair any stale state from prior crashes
    let worktree_mgr = WorktreeManager::new(config.repo_root.clone());
    worktree_mgr.init()?;
    let valid_worktrees = worktree_mgr.validate_and_repair().unwrap_or_default();
    if !valid_worktrees.is_empty() {
        info!(
            "Found {} existing plan worktrees from prior run",
            valid_worktrees.len()
        );
    }

    // Load task files
    let mut task_files: HashMap<String, crate::orchestrator::tasks::TaskFile> = HashMap::new();
    for plan in &orchestrator.plans {
        if let Ok(Some(cl)) =
            crate::orchestrator::tasks::load_checklist(&config.repo_root, &plan.num)
        {
            task_files.insert(
                plan.base.clone(),
                crate::orchestrator::tasks::TaskFile {
                    meta: crate::orchestrator::tasks::TaskMeta {
                        plan: plan.base.clone(),
                        iteration: cl.iteration,
                        total: cl.tasks.len(),
                        done: cl.done_count(),
                        max_parallel: None,
                        estimated_total_minutes: None,
                    },
                    tasks: cl.tasks,
                },
            );
        }
    }

    // Build unified task DAG
    let dag = UnifiedTaskDag::from_plans(&orchestrator.plans, &task_files)?;
    info!(
        "Unified DAG: {} nodes, max width {}, critical path {}m",
        dag.node_count(),
        dag.max_width(),
        dag.critical_path_minutes(),
    );

    let refactor_interval = if config.refactor { 5 } else { 0 };
    let mut executor = ParallelExecutor::new(
        dag,
        config.max_agents,
        refactor_interval,
        batch_branch.clone(),
    );
    // Crash recovery: try task-state.json first, fall back to events.jsonl
    // recovered_state_msg is applied to `state` after it is initialized below.
    let mut recovered_state_msg: Option<String> = None;
    let mut recovered_correction_factor: Option<f64> = None;
    info!("=== Crash recovery: loading persisted state ===");
    if let Ok(Some(saved)) = persistence.load_task_state() {
        info!("  task-state.json: {} completed tasks, {} in-flight, {} completed plans, {} plan phases",
            saved.completed_tasks.len(), saved.in_flight.len(),
            saved.completed_plans.len(), saved.plan_phases.len(),
        );
        for (plan, phase) in &saved.plan_phases {
            info!("    plan {plan}: phase={phase}");
        }
        for task in &saved.completed_tasks {
            info!("    completed: {task}");
        }
        // Parse saved plan phase strings back to PlanPhase enum
        let plan_phases: HashMap<String, crate::orchestrator::executor::PlanPhase> = saved
            .plan_phases
            .iter()
            .filter_map(|(k, v)| {
                serde_json::from_str::<crate::orchestrator::executor::PlanPhase>(v)
                    .ok()
                    .map(|phase| (k.clone(), phase))
            })
            .collect();
        let plan_phase_keys: Vec<String> = plan_phases.keys().cloned().collect();
        let plan_phases_len = plan_phases.len();
        let snapshot = crate::orchestrator::ExecutorSnapshot {
            completed_tasks: saved.completed_tasks,
            in_flight_tasks: saved.in_flight,
            completed_plans: saved.completed_plans,
            plan_phases,
            plan_iterations: saved.plan_iterations,
            merge_queue: saved.merge_queue,
            plans_since_refactor: saved.plans_since_refactor,
            plans_since_integration_test: saved.plans_since_integration_test,
            review_feedback: saved.review_feedback,
        };
        // Extract merge checkpoint and correction factor before `saved` goes out of scope
        let stale_merge_checkpoint = saved.merge_in_progress.clone();
        let saved_correction_factor = saved.correction_factor;
        executor.restore(snapshot)?;
        recovered_correction_factor = saved_correction_factor;
        info!("Restored executor state from task-state.json");

        // Load output logs for in-flight plans
        for plan_base in &plan_phase_keys {
            let output_lines = persistence.load_output_tail(plan_base, 200);
            if !output_lines.is_empty() {
                info!(
                    "  loaded {} lines of output for plan {}",
                    output_lines.len(),
                    plan_base
                );
            }
        }

        // Set resume notice in orchestrator state
        if plan_phases_len > 0 {
            let msg = format!(
                "Resumed: {} plans in flight, checkpoint loaded",
                plan_phases_len
            );
            info!("Resume notice: {}", msg);
            recovered_state_msg = Some(msg);
        }

        // Recover from in-progress merge
        if let Some(ref merge_cp) = stale_merge_checkpoint {
            warn!(
                "Found in-progress merge for plan {} — aborting stale merge",
                merge_cp.plan
            );
            let wt_path = worktree_mgr
                .worktree_base()
                .join(format!("plan-{}", merge_cp.plan));
            if wt_path.exists() {
                let _ = crate::git::ops::run_git(&wt_path, &["merge", "--abort"]);
            }
            // The plan will be re-queued for merge by schedule_next
        }
    } else {
        // Fallback: reconstruct from events.jsonl
        let completed_plans = persistence.completed_plans().unwrap_or_default();
        let completed_tasks = persistence
            .completed_tasks_from_events()
            .unwrap_or_default();
        if !completed_plans.is_empty() || !completed_tasks.is_empty() {
            let snapshot = crate::orchestrator::ExecutorSnapshot {
                completed_tasks,
                in_flight_tasks: HashMap::new(),
                completed_plans: completed_plans.iter().cloned().collect(),
                plan_phases: HashMap::new(),
                plan_iterations: HashMap::new(),
                merge_queue: Vec::new(),
                plans_since_refactor: 0,
                plans_since_integration_test: 0,
                review_feedback: HashMap::new(),
            };
            executor.restore(snapshot)?;
            info!("Restored executor state from events.jsonl (fallback)");

            // In fallback mode, load output for any completed plans
            for plan in &completed_plans {
                let output_lines = persistence.load_output_tail(plan, 200);
                if !output_lines.is_empty() {
                    info!(
                        "  loaded {} lines of output for plan {}",
                        output_lines.len(),
                        plan
                    );
                }
            }

            if !completed_plans.is_empty() {
                let msg = format!("Recovered: {} plans from events log", completed_plans.len());
                info!("Resume notice: {}", msg);
                recovered_state_msg = Some(msg);
            }
        } else {
            // Fresh run: no prior state found, clean up stale output logs
            info!("Fresh run detected — cleaning up stale output logs");
            let _ = persistence.cleanup_output_logs();
        }
    }

    // Validate git state: verify batch branch exists
    if !git_manager.branch_exists(&batch_branch) {
        info!("Batch branch {} missing, creating from main", batch_branch);
        git_manager.setup_batch_branch(&batch_branch)?;
    }

    // ── Validate persisted state against actual worktree/branch/file state ──
    // This catches the common scenario where the process is killed and restarted:
    // task-state.json says tasks are "done" but the worktree was recreated fresh
    // (or files were lost), so the executor would skip them.
    {
        let completed_plans_set: HashSet<String> =
            executor.completed_plan_names().into_iter().collect();
        let all_plans: Vec<String> = orchestrator.plans.iter().map(|p| p.base.clone()).collect();
        let mut invalidated: Vec<String> = Vec::new();

        info!("=== State/worktree reconciliation ===");
        info!("  Completed plans (merged): {:?}", completed_plans_set);
        info!("  All plans: {:?}", all_plans);

        for plan in &all_plans {
            if completed_plans_set.contains(plan) {
                info!("  [{plan}] SKIP — fully merged, no validation needed");
                continue;
            }

            let plan_branch = format!("codex/plan/{plan}");
            let branch_exists = git_manager.branch_exists(&plan_branch);
            let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
            let wt_exists = wt_path.exists();
            let completed_task_ids: Vec<String> = executor
                .completed_tasks_iter()
                .filter(|gid| gid.plan == *plan)
                .map(|gid| gid.task.clone())
                .collect();
            let in_flight_ids: Vec<String> = executor
                .in_flight_task_ids()
                .filter(|gid| gid.plan == *plan)
                .map(|gid| gid.task.clone())
                .collect();

            info!(
                "  [{plan}] branch={} worktree={} completed_tasks={} in_flight={}",
                if branch_exists { "exists" } else { "MISSING" },
                if wt_exists { "exists" } else { "MISSING" },
                completed_task_ids.len(),
                in_flight_ids.len(),
            );

            // Case 1: Both branch and worktree are gone — reset everything
            if !branch_exists && !wt_exists && !completed_task_ids.is_empty() {
                info!("  [{plan}] RESET — branch+worktree both missing, {} completed tasks invalidated", completed_task_ids.len());
                invalidated.push(plan.clone());
                continue;
            }

            // Case 1b: Branch exists but has no commits beyond batch base — all work was lost.
            // Exception: skip this check for plans already past implementation (Gating, Reviewing,
            // Merging) because the commit only happens at merge time. Resetting a plan in Gating
            // would force a full re-implementation unnecessarily.
            let plan_phase = executor.plan_phase(plan);
            let past_implementation = matches!(
                plan_phase,
                Some(crate::orchestrator::PlanPhase::Gating)
                    | Some(crate::orchestrator::PlanPhase::Reviewing)
                    | Some(crate::orchestrator::PlanPhase::Merging)
                    | Some(crate::orchestrator::PlanPhase::Complete)
            );
            if branch_exists && !completed_task_ids.is_empty() && !past_implementation {
                let plan_branch_ref = format!("codex/plan/{plan}");
                let count_str = crate::git::ops::run_git(
                    &config.repo_root,
                    &[
                        "rev-list",
                        "--count",
                        &format!("{batch_branch}..{plan_branch_ref}"),
                    ],
                )
                .unwrap_or_default();
                let commit_count: u32 = count_str.trim().parse().unwrap_or(0);
                if commit_count == 0 {
                    info!("  [{plan}] RESET — branch has 0 commits beyond {batch_branch}, {} tasks stale", completed_task_ids.len());
                    invalidated.push(plan.clone());
                    continue;
                }
                info!("  [{plan}] branch has {commit_count} commits beyond {batch_branch}");
            } else if past_implementation {
                info!("  [{plan}] SKIP commit check — plan is in {:?} phase (commits happen at merge time)", plan_phase);
            }

            // Case 2: Worktree exists — verify completed tasks' files actually exist there
            if wt_exists && !completed_task_ids.is_empty() {
                let plan_num = plan.split('-').next().unwrap_or(plan);
                if let Ok(Some(cl)) =
                    crate::orchestrator::tasks::load_checklist(&config.repo_root, plan_num)
                {
                    let mut missing_files = 0u32;
                    let mut checked_files = 0u32;
                    for task in &cl.tasks {
                        if !completed_task_ids.contains(&task.id) {
                            continue;
                        }
                        for file in &task.files {
                            checked_files += 1;
                            let file_path = wt_path.join(file);
                            if !file_path.exists() {
                                missing_files += 1;
                                info!(
                                    "  [{plan}] Task {} file MISSING in worktree: {}",
                                    task.id, file
                                );
                            }
                        }
                    }
                    info!(
                        "  [{plan}] File check: {checked_files} checked, {missing_files} missing"
                    );

                    // If more than half of completed tasks' files are missing, the worktree
                    // was likely recreated fresh — reset the whole plan
                    if missing_files > 0 && checked_files > 0 && missing_files * 2 >= checked_files
                    {
                        info!("  [{plan}] RESET — {missing_files}/{checked_files} files missing in worktree, state is stale");
                        invalidated.push(plan.clone());
                    } else if missing_files > 0 {
                        info!("  [{plan}] WARN — {missing_files} files missing but majority present, keeping state");
                    }
                } else {
                    info!("  [{plan}] No task checklist found — skipping file validation");
                }
            }
        }

        for plan in &invalidated {
            executor.reset_plan(plan);
            info!("  Reset plan {plan}: cleared all completed/in-flight tasks");
        }
        if invalidated.is_empty() {
            info!("=== Reconciliation complete: all plans valid ===");
        } else {
            info!(
                "=== Reconciliation complete: {} plans invalidated: {:?} ===",
                invalidated.len(),
                invalidated
            );
        }
    }

    // In-flight tasks from a previous run are always stale — the agent processes
    // died with the process. Clear them so the budget starts at max_agents.
    executor.clear_all_in_flight();

    // Build initial state
    let mut state = RunState::default();
    state.repo_root = Some(config.repo_root.clone());
    if let Some(msg) = recovered_state_msg {
        state.orchestrator_state = msg;
    }
    if let Some(cf) = recovered_correction_factor {
        state.time_estimator.correction_factor = cf;
        info!("Restored time estimator correction_factor={:.3}", cf);
    }
    state.plans = orchestrator
        .plans
        .iter()
        .map(|p| {
            let est = p.frontmatter.as_ref().and_then(|f| f.estimated_minutes);
            RunPlanEntry {
                base: p.base.clone(),
                num: p.num.clone(),
                status: RunPlanStatus::Pending,
                iteration: 0,
                phase: String::new(),
                estimated_minutes: est,
                actual_minutes: None,
                started_at: None,
                git_branch_short: Some(format!("codex/plan/{}", p.base)),
                git_last_commit_secs: None,
                git_dirty: None,
                merged_to_main_at: None,
                merge_commit: None,
            }
        })
        .collect();
    // Reflect restored state in TUI plan entries
    let restored_completed = executor.completed_plan_names();
    for entry in &mut state.plans {
        if restored_completed.contains(&entry.base) {
            entry.status = RunPlanStatus::CompletedPrior;
            entry.phase = "complete".to_string();
        } else if executor.plan_phase(&entry.base).is_some() {
            entry.status = RunPlanStatus::Active;
            entry.iteration = executor.plan_iteration(&entry.base);
        }
    }
    if !restored_completed.is_empty() {
        state.add_log(
            "executor",
            &format!(
                "Resumed: {} plans already complete",
                restored_completed.len(),
            ),
            LogLevel::Info,
        );
    }

    // Seed task completion overlay from both executor state AND raw task-state.json
    // (executor may lose __whole__ entries during DAG expansion/scheduling)
    for gid in executor.completed_tasks_iter() {
        state.executor_completed_tasks.insert(gid.to_string());
    }
    // Also seed directly from persisted completed_tasks to catch any entries
    // the executor dropped during scheduling/expansion
    if let Ok(Some(saved)) = persistence.load_task_state() {
        for task_str in &saved.completed_tasks {
            state.executor_completed_tasks.insert(task_str.clone());
        }
    }
    // Mark all tasks of completed plans as done
    for plan in &state.plans {
        if matches!(
            plan.status,
            RunPlanStatus::CompletedPrior | RunPlanStatus::Completed | RunPlanStatus::MergedToMain
        ) {
            if let Some(cl) = state.plan_task_cache.get(&plan.base) {
                for t in &cl.tasks {
                    state
                        .executor_completed_tasks
                        .insert(format!("{}:{}", plan.base, t.id));
                }
            }
            state
                .executor_completed_tasks
                .insert(format!("{}:__whole__", plan.base));
        }
    }
    let whole_count = state
        .executor_completed_tasks
        .iter()
        .filter(|s| s.ends_with(":__whole__"))
        .count();
    info!(
        "TUI progress seed: {} executor_completed_tasks ({} __whole__), {} plans in state",
        state.executor_completed_tasks.len(),
        whole_count,
        state.plans.len(),
    );

    // Build wave structure for TUI display (mirrors the sequential run() path)
    if let Ok(dag) =
        crate::orchestrator::PlanDag::from_plans_and_tasks(&orchestrator.plans, &task_files)
    {
        let waves = dag.compute_waves();
        state.execution_waves = waves.iter().map(|w| (w.index, w.plans.clone())).collect();
        for idx in 0..waves.len() {
            state.wave_expanded.insert(idx);
        }
    }

    state.orchestrator_state = "parallel-execute".to_string();
    state.config = crate::state::config::ConfigState::from_app_config(
        config.model.as_deref(),
        config.skip_tests,
        config.max_iterations,
        config.no_docs,
        config.no_review,
        config.fast,
    );
    if let Some(loaded) = crate::state::config::ConfigState::load(&config.repo_root) {
        state.config = loaded;
    }
    // Re-apply CLI flags that should always take precedence over persisted config.
    if config.fast {
        state.config.fast_mode = true;
    }
    if config.express {
        state.config.express_mode = true;
    }
    pool.set_fast_mode(state.config.fast_mode);
    pool.set_fallback_model(state.config.fallback_model.clone());
    if config.express {
        executor.set_express_mode(true, state.config.max_auto_fix_attempts);
        state.add_log(
            "executor",
            "Express mode enabled: no strategist, no reviews, auto-fix on gate failure",
            LogLevel::Info,
        );
    }
    state.git_branch = git_manager.current_branch().unwrap_or_default();
    state.git_branch_tree = crate::git::graph::build_branch_tree(&config.repo_root, &state.plans);

    // Preflight
    crate::orchestrator::context::write_preflight_files(&config.repo_root)?;

    // Extract prd2 context for all plans upfront
    for plan_entry in &state.plans {
        let plan_num = plan_entry
            .base
            .split('-')
            .next()
            .unwrap_or(&plan_entry.base);
        let _ = crate::orchestrator::context::extract_prd2_context(&config.repo_root, plan_num);
    }

    // Gate channel
    let (gate_tx, mut gate_rx) = mpsc::unbounded_channel::<GateCompletion>();

    // Terminal setup
    let mut terminal = tui::init().context("Failed to initialize terminal")?;
    crossterm::event::poll(std::time::Duration::ZERO)
        .context("Terminal event source failed to initialize — is stdin a TTY?")?;
    let mut term_events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(33));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut sys_collector = crate::sys_metrics::SysCollector::new();
    let mut atmosphere = crate::tui::atmosphere::Atmosphere::new();
    let mut frame_skip_counter: u32 = 0;
    let mut last_user_input = Instant::now();

    // Conductor for failure detection in parallel mode
    let mut conductor =
        crate::conductor::Conductor::new(crate::conductor::ConductorConfig::default());
    let mut last_message_at: Option<std::time::Instant> = None;
    let mut last_turn_had_output = true;
    let mut last_turn_duration_secs: u64 = 0;

    // Spawn Conductor LLM agent (meta-orchestrator, lives for the whole run)
    {
        let conductor_iid = "conductor:llm".to_string();
        let aid = AgentInstanceId::new(AgentRole::Conductor, conductor_iid.clone());
        match pool
            .spawn_instance(
                aid.clone(),
                None,
                "max",
                state.config.model_for(AgentRole::Conductor),
            )
            .await
        {
            Ok(()) => {
                pool.set_thread_id(&aid, None);
                let system_prompt = crate::conductor::llm::conductor_system_prompt();
                let plan_summary: String = orchestrator
                    .plans
                    .iter()
                    .map(|p| format!("  - {}", p.base))
                    .collect::<Vec<_>>()
                    .join("\n");
                let init_msg = format!(
                    "{system_prompt}\n\n## Current Batch (parallel mode)\n\nPlans to execute:\n{plan_summary}\n\nThe pipeline is starting in parallel mode. Respond with [OK] to acknowledge.",
                );
                if let Err(e) = pool
                    .turn_start(
                        &aid,
                        &init_msg,
                        state.config.model_for(AgentRole::Conductor),
                    )
                    .await
                {
                    tracing::error!("Failed to start conductor turn: {e}");
                    state.add_log(
                        "executor",
                        &format!("Conductor turn_start failed: {e}"),
                        LogLevel::Error,
                    );
                }
                state
                    .parallel_agents
                    .push(crate::state::ParallelAgentState {
                        instance_id: conductor_iid,
                        role: AgentRole::Conductor,
                        plan: "global".to_string(),
                        task: "conductor".to_string(),
                        output: String::new(),
                        input_tokens: 0,
                        output_tokens: 0,
                        cost_usd: 0.0,
                        active: true,
                        finished_at: None,
                        model: state
                            .config
                            .model_for(AgentRole::Conductor)
                            .unwrap_or("default")
                            .to_string(),
                        turn_started: false,
                        render_cache: Default::default(),
                    });
                let cond_model = state
                    .config
                    .model_for(AgentRole::Conductor)
                    .unwrap_or("default");
                state.add_log(
                    "conductor",
                    &format!("Conductor LLM agent spawned [model={cond_model}]"),
                    LogLevel::Info,
                );
            }
            Err(e) => {
                state.add_log(
                    "conductor",
                    &format!("Failed to spawn conductor LLM: {e}"),
                    LogLevel::Warn,
                );
            }
        }
    }

    // Kick off initial scheduling
    let initial_actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
    state.add_log(
        "executor",
        &format!(
            "Parallel mode: {} plans, {} max agents, {} initial actions",
            orchestrator.plans.len(),
            config.max_agents,
            initial_actions.len(),
        ),
        LogLevel::Info,
    );

    execute_actions(
        initial_actions,
        &mut executor,
        &mut pool,
        &worktree_mgr,
        &mut state,
        &config,
        &persistence,
        &gate_tx,
        &batch_branch,
        &git_manager,
        &spawn_ready_tx,
    )
    .await?;

    // Seed task checklist for the default-selected plan
    parallel_refresh_tasks(&mut state, &config);

    // Now that checklists are loaded, expand completed plans' tasks into the overlay
    for plan in &state.plans {
        if matches!(
            plan.status,
            RunPlanStatus::CompletedPrior | RunPlanStatus::Completed | RunPlanStatus::MergedToMain
        ) {
            if let Some(cl) = state.plan_task_cache.get(&plan.base) {
                for t in &cl.tasks {
                    state
                        .executor_completed_tasks
                        .insert(format!("{}:{}", plan.base, t.id));
                }
            }
            state
                .executor_completed_tasks
                .insert(format!("{}:__whole__", plan.base));
        }
    }

    {
        let (done, total) = state.task_weighted_progress();
        let whole_in_overlay = state
            .executor_completed_tasks
            .iter()
            .filter(|s| s.ends_with(":__whole__"))
            .count();
        let completed_prior = state
            .plans
            .iter()
            .filter(|p| matches!(p.status, RunPlanStatus::CompletedPrior))
            .count();
        info!("TUI progress: {done}/{total} (overlay={}, __whole__={}, completed_prior={}, cache_keys={})",
            state.executor_completed_tasks.len(), whole_in_overlay, completed_prior,
            state.plan_task_cache.len());
        // Check cache directly for a few plans
        for key in &[
            "01-workspace-scaffold",
            "02-core-types",
            "08c-perspective-modes",
        ] {
            let in_cache = state.plan_task_cache.get(*key).map(|cl| cl.total());
            let via_method = state.checklist_for_plan(key).map(|cl| cl.total());
            info!("  cache[{key}]={in_cache:?}, method={via_method:?}");
        }
    }

    // Persistence checkpoint timer
    let mut checkpoint = tokio::time::interval(Duration::from_secs(10));
    checkpoint.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Task checklist refresh timer (5s intervals for all cached plans)
    let mut task_refresh = tokio::time::interval(Duration::from_secs(5));
    task_refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Crash state snapshot throttle (~2s)
    let mut last_crash_snapshot = Instant::now();

    // Worktree reconciliation counter (runs every 12th task_refresh tick = ~60s)
    let mut reconciliation_counter: u32 = 0;

    // Persistence throttle for agent output (avoid 100+ disk writes/sec)
    // Batch writes to every 100ms instead of every message
    let mut persist_flush = tokio::time::interval(Duration::from_millis(100));
    persist_flush.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Message processing throttle — process max 20 messages per tick to avoid CPU spike
    // from agent text streaming (100+ msg/s). Unprocessed messages queue up in agent_rx.
    let mut messages_this_tick: u32 = 0;
    const MAX_MESSAGES_PER_TICK: u32 = 20;

    // SIGTERM handler for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to register SIGTERM handler");

    // Main event loop — render is gated to the 30fps tick arm to avoid jitter
    // from high-frequency agent message streams (100+ msg/s with 9 agents).
    // No `biased;` — fair scheduling prevents agent_rx from starving tick/key arms.
    loop {
        messages_this_tick = 0; // Reset message counter at start of loop
        tokio::select! {
            _ = sigterm.recv() => {
                info!("SIGTERM received — writing checkpoint and exiting");
                let (inp_tok, out_tok) = (
                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                );
                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                state.add_log("system", "SIGTERM received, checkpoint saved", LogLevel::Warn);
                break;
            }
            Some(event) = agent_rx.recv(), if messages_this_tick < MAX_MESSAGES_PER_TICK => {
                messages_this_tick += 1;
                match event {
                    AgentEvent::TurnCompleted { role, instance: Some(ref iid), .. } => {
                        info!("Turn completed for instance {iid}");
                        // Track turn duration for conductor
                        if let Some(started) = state.turn_started_at.remove(iid) {
                            last_turn_duration_secs = started.elapsed().as_secs();
                            last_turn_had_output = state.parallel_agents.iter()
                                .find(|p| p.instance_id == *iid)
                                .map(|p| p.output.len() > 200)
                                .unwrap_or(true);
                        }
                        // Mark parallel agent as inactive with finish timestamp
                        if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                            pa.active = false;
                            pa.finished_at = Some(std::time::Instant::now());
                        }
                        // Find which task(s) this instance was running (batch-aware).
                        let instance_tasks = executor.tasks_for_instance(iid);
                        if !instance_tasks.is_empty() {
                            // Validate the agent actually produced meaningful output
                            let agent_output = get_parallel_agent_output(&state, iid);
                            let output_len = agent_output.len();
                            let input_tokens = state.parallel_agents.iter()
                                .find(|p| p.instance_id == *iid)
                                .map(|p| p.input_tokens)
                                .unwrap_or(0);
                            info!("complete[{}] tasks={} output_len={} input_tokens={} role={:?}",
                                iid, instance_tasks.len(), output_len, input_tokens, role);
                            let actions = if output_len < 50 {
                                // No meaningful output — treat as failed, retry
                                state.add_log("executor", &format!(
                                    "Instance {} turn completed with no output ({} chars, {}tok) — retrying",
                                    iid, output_len, input_tokens,
                                ), LogLevel::Warn);
                                info!("complete[{}] RETRY — output too short ({})", iid, output_len);
                                executor.handle_instance_failed(iid)
                            } else {
                                for task_id in &instance_tasks {
                                    state.executor_completed_tasks.insert(task_id.to_string());
                                    let event = crate::state::persistence::PersistenceManager::make_task_event(
                                        "task_done", &task_id.plan, Some(&task_id.task),
                                        Some(iid), None,
                                    );
                                    let _ = persistence.append_task_event(&event);
                                }
                                state.add_log("executor", &format!(
                                    "Instance {} done ({} tasks, {}chars, {}tok)",
                                    iid, instance_tasks.len(), output_len, input_tokens,
                                ), LogLevel::Info);
                                info!("complete[{}] DONE — tasks={} output={} tokens={}", iid, instance_tasks.len(), output_len, input_tokens);
                                executor.handle_instance_complete(iid)
                            };
                            parallel_refresh_tasks(&mut state, &config);

                            // Kill agent (no warm pool reuse)
                            let aid = AgentInstanceId::new(role, iid.clone());
                            pool.kill_instance(&aid).await;

                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;

                            // Checkpoint immediately after task completion
                            let (inp_tok, out_tok) = (
                                state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                            );
                            write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                        } else if iid.starts_with("pre-planner:")
                               || iid.starts_with("doc-verifier:")
                               || iid.starts_with("merge-resolver:") {
                            // Meta agents not in the task DAG or review list.
                            // Their completion may unblock plan scheduling — fire schedule_next.
                            let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if iid.starts_with("refactorer:") {
                            executor.handle_refactoring_complete();
                            let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if iid.starts_with("integration-tester:") {
                            executor.handle_integration_tests_complete();
                            let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if iid.starts_with("conductor:") {
                            // Conductor LLM turn completed — parse directive
                            let output = get_parallel_agent_output(&state, iid);
                            if let Some(directive) = crate::conductor::llm::parse_directive(&output) {
                                use crate::conductor::llm::ConductorDirective;
                                let is_ok = matches!(&directive, ConductorDirective::Ok);
                                match directive {
                                    ConductorDirective::Ok => {
                                        state.add_log("conductor", "LLM assessment: OK", LogLevel::Info);
                                    }
                                    ConductorDirective::Nudge { role, message } => {
                                        state.add_log("conductor", &format!(
                                            "LLM nudge → {role}: {}", message.chars().take(80).collect::<String>()
                                        ), LogLevel::Warn);
                                        // Find an active agent with this role and inject
                                        let target_iid = state.parallel_agents.iter()
                                            .find(|p| p.role == role && p.active)
                                            .map(|p| p.instance_id.clone());
                                        if let Some(ref tid) = target_iid {
                                            let aid = AgentInstanceId::new(role, tid.clone());
                                            if pool.is_spawned(&aid) {
                                                let _ = pool.turn_interrupt(&aid).await;
                                                if let Err(e) = pool.turn_start(&aid, &message, None).await {
                                                    tracing::error!("Failed to start nudge turn for {role}:{tid}: {e}");
                                                    state.add_log("executor", &format!("Nudge turn_start failed for {role}:{tid}: {e}"), LogLevel::Error);
                                                }
                                            }
                                        }
                                    }
                                    ConductorDirective::SkipReviews => {
                                        state.add_log("conductor", "LLM directive: skip reviews", LogLevel::Warn);
                                        // Find plans in review (both from plan_review_stage and stuck in executor)
                                        let from_stage: Vec<String> = state.plan_review_stage.keys().cloned().collect();
                                        let from_executor: Vec<String> = executor.plan_states.iter()
                                            .filter(|(p, s)| matches!(s.phase, crate::orchestrator::executor::PlanPhase::Reviewing) && !state.plan_review_stage.contains_key(p.as_str()))
                                            .map(|(p, _)| p.clone())
                                            .collect();
                                        let reviewing_plans: Vec<String> = from_stage.into_iter().chain(from_executor).collect();
                                        for plan_base in reviewing_plans {
                                            state.plan_review_stage.remove(&plan_base);
                                            state.plan_pending_reviews.remove(&plan_base);
                                            let actions = executor.handle_plan_reviews_passed(&plan_base);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                    ConductorDirective::ForceAdvance => {
                                        state.add_log("conductor", "LLM directive: force advance", LogLevel::Warn);
                                        // Find plans in review (both from plan_review_stage and stuck in executor)
                                        let from_stage: Vec<String> = state.plan_review_stage.keys().cloned().collect();
                                        let from_executor: Vec<String> = executor.plan_states.iter()
                                            .filter(|(p, s)| matches!(s.phase, crate::orchestrator::executor::PlanPhase::Reviewing) && !state.plan_review_stage.contains_key(p.as_str()))
                                            .map(|(p, _)| p.clone())
                                            .collect();
                                        let reviewing_plans: Vec<String> = from_stage.into_iter().chain(from_executor).collect();
                                        for plan_base in reviewing_plans {
                                            state.plan_review_stage.remove(&plan_base);
                                            state.plan_pending_reviews.remove(&plan_base);
                                            let actions = executor.handle_plan_reviews_passed(&plan_base);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                    ConductorDirective::Throttle { limit } => {
                                        state.add_log("conductor", &format!("LLM throttle → {limit}"), LogLevel::Warn);
                                        conductor.rate_limiter.soft_limit = limit;
                                    }
                                    ConductorDirective::Restart { role } => {
                                        state.add_log("conductor", &format!("LLM restart → {role}"), LogLevel::Warn);
                                        // Kill active agents with this role
                                        let target_iids: Vec<String> = state.parallel_agents.iter()
                                            .filter(|p| p.role == role && p.active)
                                            .map(|p| p.instance_id.clone())
                                            .collect();
                                        for tid in &target_iids {
                                            let aid = AgentInstanceId::new(role, tid.clone());
                                            pool.kill_instance(&aid).await;
                                        }
                                    }
                                    ConductorDirective::PrePlan { .. } => {
                                        // Ignored in parallel mode — pre-planning is handled by the executor
                                    }
                                    ConductorDirective::Enrich { plan_num } => {
                                        state.add_log(
                                            "conductor",
                                            &format!("LLM directive: ENRICH {plan_num} — running bardo-enrich.sh"),
                                            LogLevel::Warn,
                                        );
                                        let root = config.repo_root.clone();
                                        let pn = plan_num.clone();
                                        let enrich_result = tokio::task::spawn_blocking(move || {
                                            let script = root.join("bardo-enrich.sh");
                                            if !script.is_file() {
                                                return Err(format!(
                                                    "bardo-enrich.sh not found at {}",
                                                    script.display()
                                                ));
                                            }
                                            std::process::Command::new("bash")
                                                .arg(&script)
                                                .arg(&pn)
                                                .current_dir(&root)
                                                .status()
                                                .map_err(|e| e.to_string())
                                        })
                                        .await;
                                        match enrich_result {
                                            Ok(Ok(status)) if status.success() => {
                                                state.add_log(
                                                    "conductor",
                                                    &format!("ENRICH {plan_num}: bardo-enrich.sh exited OK"),
                                                    LogLevel::Info,
                                                );
                                            }
                                            Ok(Ok(status)) => {
                                                state.add_log(
                                                    "conductor",
                                                    &format!(
                                                        "ENRICH {plan_num}: bardo-enrich.sh failed ({status})"
                                                    ),
                                                    LogLevel::Error,
                                                );
                                            }
                                            Ok(Err(e)) => {
                                                state.add_log(
                                                    "conductor",
                                                    &format!("ENRICH {plan_num}: {e}"),
                                                    LogLevel::Error,
                                                );
                                            }
                                            Err(e) => {
                                                state.add_log(
                                                    "conductor",
                                                    &format!("ENRICH {plan_num}: join error {e}"),
                                                    LogLevel::Error,
                                                );
                                            }
                                        }
                                    }
                                    ConductorDirective::ResetReview(reason) => {
                                        state.add_log("conductor", &format!("LLM RESET_REVIEW → {}", reason.chars().take(80).collect::<String>()), LogLevel::Warn);
                                        // Kill all review-phase agents across all plans
                                        for review_role in &[AgentRole::DocVerifier, AgentRole::Scribe, AgentRole::Critic, AgentRole::Auditor] {
                                            let target_iids: Vec<String> = state.parallel_agents.iter()
                                                .filter(|p| p.role == *review_role && p.active)
                                                .map(|p| p.instance_id.clone())
                                                .collect();
                                            for tid in &target_iids {
                                                let aid = AgentInstanceId::new(*review_role, tid.clone());
                                                pool.kill_instance(&aid).await;
                                            }
                                        }
                                        state.conductor_reset_brief = Some(format!("[CONDUCTOR RESET] {reason}"));
                                    }
                                    ConductorDirective::SpawnReview { plan_base } => {
                                        state.add_log("conductor", &format!("LLM directive: SPAWN-REVIEW {plan_base}"), LogLevel::Warn);
                                        pool.kill_plan_agents(&plan_base).await;
                                        state.plan_review_stage.remove(&plan_base);
                                        state.plan_pending_reviews.remove(&plan_base);
                                        state.parallel_agents.retain(|p| p.plan != plan_base);
                                        let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                    ConductorDirective::Validate { .. }
                                    | ConductorDirective::FixPlan { .. }
                                    | ConductorDirective::SkipValidation { .. } => {
                                        // New validation directives — not yet wired in parallel mode
                                    }
                                    ConductorDirective::PhaseReject { plan, reason } => {
                                        state.add_log("conductor", &format!(
                                            "PHASE-REJECT {plan}: {}", reason.chars().take(120).collect::<String>()
                                        ), LogLevel::Warn);

                                        let phase_validated = state.pending_phase_validation.take()
                                            .filter(|(p, _)| *p == plan)
                                            .map(|(_, phase)| phase);

                                        let actions = match phase_validated.as_deref() {
                                            Some("gates_passed") => {
                                                // Kill any review agents for this plan
                                                let review_iids: Vec<(AgentRole, String)> = state.parallel_agents.iter()
                                                    .filter(|p| p.plan == plan && matches!(p.role,
                                                        AgentRole::Architect | AgentRole::Auditor |
                                                        AgentRole::Scribe | AgentRole::Critic |
                                                        AgentRole::QuickReviewer))
                                                    .map(|p| (p.role, p.instance_id.clone()))
                                                    .collect();
                                                for (role, iid) in &review_iids {
                                                    let aid = AgentInstanceId::new(*role, iid.clone());
                                                    pool.kill_instance(&aid).await;
                                                }
                                                state.plan_review_stage.remove(&plan);
                                                state.plan_pending_reviews.remove(&plan);
                                                executor.store_review_feedback(&plan, format!("[CONDUCTOR PHASE-REJECT] {reason}"));
                                                executor.handle_plan_revise(&plan)
                                            }
                                            Some("reviews_passed") => {
                                                // Cancel merge if queued/in progress
                                                if executor.currently_merging.as_deref() == Some(plan.as_str()) {
                                                    executor.merge_in_progress = false;
                                                    executor.currently_merging = None;
                                                }
                                                executor.merge_queue.retain(|p| p != &plan);
                                                executor.reverify_plan(&plan)
                                            }
                                            Some("merged") => {
                                                state.add_log("conductor", &format!(
                                                    "PHASE-REJECT after merge for {plan} — cannot undo. Reason: {reason}"
                                                ), LogLevel::Error);
                                                vec![]
                                            }
                                            _ => {
                                                // No phase context — re-verify
                                                executor.reverify_plan(&plan)
                                            }
                                        };
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                    ConductorDirective::RetryPlan { plan } => {
                                        state.add_log("conductor", &format!(
                                            "RETRY-PLAN {plan} — resetting failed plan for retry"
                                        ), LogLevel::Warn);
                                        let actions = executor.retry_failed_plan(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                }
                                // Clear pending phase validation on any directive
                                state.pending_phase_validation = None;
                                // If conductor said OK and there's a pending inject, forward it
                                if is_ok {
                                    if let Some(pending) = state.pending_inject.take() {
                                        if let (Some(role), Some(ref target_iid)) = (pending.target_role, &pending.target_instance_id) {
                                            let aid = AgentInstanceId::new(role, target_iid.clone());
                                            if pool.is_spawned(&aid) {
                                                let _ = pool.turn_interrupt(&aid).await;
                                                let inject_msg = format!("Supervisor message: {}\n\nContinue from where you left off.", pending.message);
                                                if let Err(e) = pool.turn_start(&aid, &inject_msg, None).await {
                                                    tracing::error!("Failed to start inject turn for {role}:{target_iid}: {e}");
                                                    state.add_log("executor", &format!("Inject turn_start failed for {role}:{target_iid}: {e}"), LogLevel::Error);
                                                }
                                                if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *target_iid) {
                                                    pa_mut.active = true;
                                                }
                                                state.add_log("inject", &format!("[conductor→{role}:{target_iid}] {}", pending.message), LogLevel::Info);
                                                let echo = format!("\n--- Conductor-routed inject ---\n{}\n-------------------------------\n", pending.message);
                                                if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *target_iid) {
                                                    pa_mut.output.push_str(&echo);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Conductor issued a directive instead of OK — clear pending inject
                                    // (the directive itself handles routing)
                                    state.pending_inject = None;
                                }
                            } else {
                                // No directive parsed — if there's a pending inject, forward it as fallback
                                if let Some(pending) = state.pending_inject.take() {
                                    if let (Some(role), Some(ref target_iid)) = (pending.target_role, &pending.target_instance_id) {
                                        let aid = AgentInstanceId::new(role, target_iid.clone());
                                        if pool.is_spawned(&aid) {
                                            let _ = pool.turn_interrupt(&aid).await;
                                            let inject_msg = format!("Supervisor message: {}\n\nContinue from where you left off.", pending.message);
                                            if let Err(e) = pool.turn_start(&aid, &inject_msg, None).await {
                                                tracing::error!("Failed to start fallback inject turn for {role}:{target_iid}: {e}");
                                                state.add_log("executor", &format!("Fallback inject turn_start failed for {role}:{target_iid}: {e}"), LogLevel::Error);
                                            }
                                            if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *target_iid) {
                                                pa_mut.active = true;
                                            }
                                            state.add_log("inject", &format!("[fallback:{role}:{target_iid}] {}", pending.message), LogLevel::Info);
                                        }
                                    }
                                }
                            }
                            // Mark conductor as idle, ready for next consultation
                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                pa.active = false;
                            }
                            // Always-on: ensure conductor stays spawned after each turn
                            let conductor_aid_recheck = AgentInstanceId::new(AgentRole::Conductor, iid.clone());
                            if !pool.is_spawned(&conductor_aid_recheck) {
                                let effort = state.config.effort_for(AgentRole::Conductor).label();
                                match pool.spawn_instance(
                                    conductor_aid_recheck.clone(), None, effort,
                                    state.config.model_for(AgentRole::Conductor),
                                ).await {
                                    Ok(()) => {
                                        pool.set_thread_id(&conductor_aid_recheck, None);
                                        state.add_log("conductor", "Conductor re-spawned (always-on)", LogLevel::Info);
                                    }
                                    Err(e) => {
                                        state.add_log("conductor", &format!("Conductor re-spawn failed: {e}"), LogLevel::Warn);
                                    }
                                }
                            }
                        } else if iid.starts_with("express-impl:") {
                            // Express implementer completed — run gates.
                            let plan = iid.strip_prefix("express-impl:").unwrap_or(iid).to_string();
                            state.add_log("executor", &format!("Express implementer done for {plan} → running gates"), LogLevel::Info);
                            let aid = AgentInstanceId::new(role, iid.clone());
                            pool.kill_instance(&aid).await;
                            // Mark __whole__ task complete so the executor knows implementation is done
                            let whole_id = crate::orchestrator::GlobalTaskId {
                                plan: plan.clone(),
                                task: "__whole__".to_string(),
                            };
                            let actions = executor.handle_task_complete(whole_id);
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if iid.starts_with("auto-fix:") {
                            // Auto-fixer completed — re-run gates.
                            let plan = iid.strip_prefix("auto-fix:").unwrap_or(iid).to_string();
                            state.add_log("executor", &format!("Auto-fixer done for {plan} → re-running gates"), LogLevel::Info);
                            let aid = AgentInstanceId::new(role, iid.clone());
                            pool.kill_instance(&aid).await;
                            let actions = executor.handle_auto_fix_complete(&plan);
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if let Some((prefix, plan)) = review_plan_from_iid(iid) {
                            // Review agent completed — staged flow.
                            let review_output = get_parallel_agent_output(&state, iid);
                            let output_len = review_output.len();
                            let stage = state.plan_review_stage.get(&plan).cloned();

                            // Read the review file from disk (the correct source for verdicts)
                            let wd_for_review = {
                                let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                if wt_path.exists() { Some(wt_path) } else { None }
                            };
                            let file_content = read_review_file(
                                &config.repo_root,
                                wd_for_review.as_deref(),
                                prefix,
                                &plan,
                            );

                            // Review context now flows through context/out/ (see inject.rs::collect_review).
                            // The legacy file copy is removed; ArtifactStore handles persistence.

                            let file_verdict = crate::orchestrator::phase::extract_verdict(&file_content);
                            // Use file verdict if found, fall back to chat output
                            let (verdict, verdict_source) = if !matches!(file_verdict, crate::orchestrator::phase::Verdict::Revise { .. }) {
                                (file_verdict, "file")
                            } else {
                                (crate::orchestrator::phase::extract_verdict(&review_output), "chat")
                            };

                            // Diagnostic logging for verdict
                            state.add_log("executor", &format!(
                                "{prefix} completed for {plan}: verdict={} (from {verdict_source}), output_len={output_len}",
                                match &verdict {
                                    crate::orchestrator::phase::Verdict::Approve => "APPROVE".to_string(),
                                    crate::orchestrator::phase::Verdict::Revise { issues } => format!("REVISE ({} issues)", issues.len()),
                                }
                            ), if matches!(verdict, crate::orchestrator::phase::Verdict::Revise { .. }) && output_len > 100 {
                                LogLevel::Warn
                            } else {
                                LogLevel::Info
                            });
                            if output_len < 50 {
                                state.add_log("executor", &format!(
                                    "{prefix} for {plan} produced very little output ({output_len} chars) — may indicate agent failure"
                                ), LogLevel::Warn);
                            }

                            let worktree_fallback = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                            let wt_mgr = worktree_mgr.clone();
                            let plan_clone = plan.clone();
                            let batch_clone = batch_branch.to_string();
                            let wd = match tokio::task::spawn_blocking(move || {
                                wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                            }).await {
                                Ok(Ok(pw)) => Some(pw.path),
                                Ok(Err(_)) | Err(_) => {
                                    if worktree_fallback.exists() { Some(worktree_fallback) } else { None }
                                }
                            };

                            match (prefix, &stage) {
                                // ── QuickReviewer completed (Standard plans) ─────
                                ("quick", Some(crate::state::ReviewStage::ReviewerPending)) => {
                                    use crate::orchestrator::phase::Verdict;
                                    state.plan_review_stage.remove(&plan);
                                    state.plan_pending_reviews.remove(&plan);
                                    match verdict {
                                        Verdict::Revise { .. } if output_len < 50 => {
                                            state.add_log("executor", &format!(
                                                "QuickReviewer empty output for {plan} ({output_len} chars) — re-implementing"
                                            ), LogLevel::Warn);
                                            state.plan_doc_revisions.remove(&plan);
                                            let actions = executor.handle_plan_gates_failed(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                        Verdict::Revise { .. } => {
                                            state.add_log("executor", &format!(
                                                "QuickReviewer REVISE for {plan} → proceeding (non-blocking, archiving nits)"
                                            ), LogLevel::Warn);
                                            let iteration = executor.plan_iteration(&plan);
                                            let plan_num = plan.split('-').next().unwrap_or(&plan);
                                            let archive_dir = config.repo_root.join(format!(
                                                "plans/context/archive/{}/iter-{}", plan_num, iteration
                                            ));
                                            std::fs::create_dir_all(&archive_dir).ok();
                                            std::fs::write(archive_dir.join(format!("{plan_num}-quick.md")), &review_output).ok();
                                            if let Some(PlanPhase::Reviewing) = executor.plan_phase(&plan) {
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        }
                                        Verdict::Approve => {
                                            state.add_log("executor", &format!(
                                                "QuickReviewer APPROVE for {plan} → merge"
                                            ), LogLevel::Info);
                                            // Guard: only merge if plan is still in Reviewing phase
                                            // (prevents acting on stale TurnCompleted events)
                                            if let Some(PlanPhase::Reviewing) = executor.plan_phase(&plan) {
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            } else {
                                                state.add_log("executor", &format!(
                                                    "QuickReviewer APPROVE for {plan} ignored — plan not in Reviewing phase"
                                                ), LogLevel::Warn);
                                            }
                                        }
                                    }
                                }

                                // ── Architect completed ──────────────────────────
                                ("arch", Some(crate::state::ReviewStage::ReviewerPending)) => {
                                    use crate::orchestrator::phase::Verdict;
                                    match verdict {
                                        Verdict::Revise { .. } if output_len < 50 => {
                                            // Agent produced no meaningful output — re-implement
                                            state.add_log("executor", &format!(
                                                "Architect empty output for {plan} ({output_len} chars) — agent failure, re-implementing"
                                            ), LogLevel::Warn);
                                            state.plan_review_stage.remove(&plan);
                                            state.plan_pending_reviews.remove(&plan);
                                            state.plan_doc_revisions.remove(&plan);
                                            let actions = executor.handle_plan_gates_failed(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                        Verdict::Revise { .. } => {
                                            state.add_log("executor", &format!("Architect REVISE for {plan} → re-implement"), LogLevel::Warn);
                                            state.plan_review_stage.remove(&plan);
                                            state.plan_pending_reviews.remove(&plan);

                                            // Archive review output
                                            let iteration = executor.plan_iteration(&plan);
                                            let plan_num = plan.split('-').next().unwrap_or(&plan);
                                            let archive_dir = config.repo_root.join(format!(
                                                "plans/context/archive/{}/iter-{}", plan_num, iteration
                                            ));
                                            std::fs::create_dir_all(&archive_dir).ok();
                                            let arch_path = archive_dir.join(format!("{}-arch.md", plan_num));
                                            std::fs::write(&arch_path, &review_output).ok();
                                            info!(
                                                "REVISE[{plan}] iter={iteration} feedback_len={} archive={}",
                                                review_output.len(), arch_path.display()
                                            );

                                            executor.store_review_feedback(&plan, review_output.clone());

                                            // Consult conductor about the REVISE
                                            let revise_summary = review_output.chars().take(500).collect::<String>();
                                            parallel_consult_conductor(
                                                &mut state, &mut pool,
                                                &format!("Architect REVISE for {plan} (iter {}). Re-running strategist.", iteration),
                                                &revise_summary,
                                            ).await;

                                            let actions = executor.handle_plan_revise(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                        Verdict::Approve => {
                                            let verdict_label = "APPROVE";
                                            state.add_log("executor", &format!("Reviewer {verdict_label} for {plan} → scribe"), LogLevel::Info);
                                            state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::ScribePending);
                                            let mut pending = std::collections::HashSet::new();

                                            let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                            let pi = crate::orchestrator::plan::discover_plans(
                                                &config.repo_root.join("plans"), &[plan_num],
                                            ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));

                                            if let Some(ref plan_info) = pi {
                                                if let Ok(prompt) = crate::orchestrator::prompts::scribe_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                    let pfx = "scribe";
                                                    let review_iid = format!("{pfx}:{plan}");
                                                    let aid = AgentInstanceId::new(AgentRole::Scribe, review_iid.clone());
                                                    let effort = state.config.effort_for(AgentRole::Scribe).label();
                                                    if pool.spawn_instance(aid.clone(), wd.clone(), effort, state.config.model_for(AgentRole::Scribe)).await.is_ok() {
                                                        pool.set_thread_id(&aid, None);
                                                        if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Scribe)).await {
                                                            tracing::error!("Failed to start scribe turn for {plan}: {e}");
                                                            state.add_log("executor", &format!("scribe turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                        }
                                                        state.parallel_agents.push(crate::state::ParallelAgentState {
                                                            instance_id: review_iid,
                                                            role: AgentRole::Scribe,
                                                            plan: plan.clone(),
                                                            task: pfx.to_string(),
                                                            output: String::new(),
                                                            input_tokens: 0,
                                                            output_tokens: 0,
                                                            cost_usd: 0.0,
                                                            active: true,
                                                            finished_at: None,
                                                            model: String::new(),
                                                            turn_started: false,
                                                                            render_cache: Default::default(),
                                                        });
                                                        pending.insert(AgentRole::Scribe);
                                                    }
                                                }
                                            }

                                            if pending.is_empty() {
                                                // Scribe spawn failed — skip to critic
                                                state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::CriticPending);
                                            } else {
                                                state.plan_pending_reviews.insert(plan.clone(), pending);
                                            }
                                        }
                                    }
                                }

                                // ── Scribe completed ─────────────────────────────
                                ("scribe", Some(crate::state::ReviewStage::ScribePending)) => {
                                    let all_done = if let Some(pending) = state.plan_pending_reviews.get_mut(&plan) {
                                        pending.remove(&AgentRole::Scribe);
                                        pending.is_empty()
                                    } else {
                                        false // missing entry means pending was already cleared — do not advance
                                    };

                                    if all_done {
                                        // Guard: only spawn critic if plan is still in Reviewing phase
                                        // (prevents acting on stale TurnCompleted events)
                                        if let Some(crate::orchestrator::executor::PlanPhase::Reviewing) = executor.plan_phase(&plan) {
                                            state.plan_pending_reviews.remove(&plan);
                                            state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::CriticPending);
                                            state.add_log("executor", &format!("Scribe done for {plan} → critic"), LogLevel::Info);

                                            // Spawn critic
                                            let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                            let pi = crate::orchestrator::plan::discover_plans(
                                                &config.repo_root.join("plans"), &[plan_num],
                                            ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));

                                            if let Some(ref plan_info) = pi {
                                            let iid_critic = format!("critic:{plan}");
                                            let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                            let effort = state.config.effort_for(AgentRole::Critic).label();
                                            match crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                Ok(prompt) => {
                                                    if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                        pool.set_thread_id(&aid, None);
                                                        if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                            tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                            state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                        }
                                                        state.parallel_agents.push(crate::state::ParallelAgentState {
                                                            instance_id: iid_critic,
                                                            role: AgentRole::Critic,
                                                            plan: plan.clone(),
                                                            task: "critic".to_string(),
                                                            output: String::new(),
                                                            input_tokens: 0,
                                                            output_tokens: 0,
                                                            cost_usd: 0.0,
                                                            active: true,
                                                            finished_at: None,
                                                            model: String::new(),
                                                            turn_started: false,
                                                                            render_cache: Default::default(),
                                                        });
                                                    } else {
                                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                                        execute_actions(
                                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                            &spawn_ready_tx,
                                                        ).await?;
                                                    }
                                                }
                                                Err(_) => {
                                                    let actions = executor.handle_plan_reviews_passed(&plan);
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                }
                                            }
                                            } else {
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        } else {
                                            state.add_log("executor", &format!(
                                                "Audit+scribe completion for {plan} ignored — plan not in Reviewing phase"
                                            ), LogLevel::Warn);
                                        }
                                    }
                                }

                                // ── Critic completed ─────────────────────────────
                                ("critic", Some(crate::state::ReviewStage::CriticPending)) => {
                                    use crate::orchestrator::phase::Verdict;
                                    match verdict {
                                        Verdict::Revise { .. } if output_len < 50 => {
                                            // Agent produced no meaningful output -- retry or fail
                                            let retry_key = format!("critic-missing:{plan}");
                                            let retry_count = {
                                                let retries = state.plan_agent_retries.entry(retry_key.clone()).or_insert(0);
                                                *retries += 1;
                                                *retries
                                            };
                                            if retry_count <= 2 {
                                                state.add_log("executor", &format!(
                                                    "Critic MISSING for {plan} ({output_len} chars) — retrying ({retry_count}/2)"
                                                ), LogLevel::Warn);
                                                // Re-spawn critic
                                                let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                                let pi = crate::orchestrator::plan::discover_plans(
                                                    &config.repo_root.join("plans"), &[plan_num],
                                                ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));
                                                if let Some(ref plan_info) = pi {
                                                    let iid_critic = format!("critic:{plan}");
                                                    let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                                    let effort = state.config.effort_for(AgentRole::Critic).label();
                                                    if let Ok(prompt) = crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                        if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                            pool.set_thread_id(&aid, None);
                                                            if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                                tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                                state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                            }
                                                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid_critic) {
                                                                pa.active = true;
                                                                pa.finished_at = None;
                                                                pa.output.clear();
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                state.add_log("executor", &format!(
                                                    "Critic MISSING for {plan} after {retry_count} retries — marking Failed"
                                                ), LogLevel::Error);
                                                state.plan_review_stage.remove(&plan);
                                                state.plan_pending_reviews.remove(&plan);
                                                if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                                                    ps.phase = PlanPhase::Failed("critic agent failed repeatedly".to_string());
                                                }
                                                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                                    entry.status = RunPlanStatus::Failed;
                                                }
                                                let drain_actions = executor.drain_merge_queue();
                                                execute_actions(
                                                    drain_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        }
                                        Verdict::Approve => {
                                            state.add_log("executor", &format!("Critic APPROVE for {plan} → merge"), LogLevel::Info);
                                            state.plan_review_stage.remove(&plan);
                                            state.plan_pending_reviews.remove(&plan);
                                            let actions = executor.handle_plan_reviews_passed(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                        Verdict::Revise { .. } => {
                                            // Circuit breaker: cap critic/scribe cycles at 2
                                            let doc_rev_count = {
                                                let entry = state.plan_doc_revisions.entry(plan.clone()).or_insert(0);
                                                *entry += 1;
                                                *entry
                                            };
                                            if doc_rev_count > 2 {
                                                state.add_log("executor", &format!(
                                                    "Critic/scribe loop capped at 2 for {plan} — committing with current docs"
                                                ), LogLevel::Warn);
                                                parallel_consult_conductor(
                                                    &mut state, &mut pool,
                                                    &format!("Circuit breaker: critic/scribe loop for {plan} hit cap ({}). Auto-merging.", doc_rev_count),
                                                    "",
                                                ).await;
                                                state.plan_review_stage.remove(&plan);
                                                state.plan_pending_reviews.remove(&plan);
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            } else {
                                            // Re-run scribe only, then critic again
                                            state.add_log("executor", &format!(
                                                "Critic REVISE for {plan} (doc rev {}/{}) → re-run scribe",
                                                doc_rev_count, 2
                                            ), LogLevel::Warn);

                                            let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                            let pi = crate::orchestrator::plan::discover_plans(
                                                &config.repo_root.join("plans"), &[plan_num],
                                            ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));

                                            if let Some(ref plan_info) = pi {
                                                // Unique iid with timestamp suffix to avoid collisions
                                                let ts = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .map(|d| d.as_secs())
                                                    .unwrap_or(0);
                                                let scribe_iid = format!("scribe:{plan}:{ts}");
                                                let aid = AgentInstanceId::new(AgentRole::Scribe, scribe_iid.clone());
                                                let effort = state.config.effort_for(AgentRole::Scribe).label();
                                                if let Ok(prompt) = crate::orchestrator::prompts::doc_revision_prompt(&config.repo_root, plan_info, &review_output, wd.as_deref()) {
                                                    if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Scribe)).await.is_ok() {
                                                        pool.set_thread_id(&aid, None);
                                                        if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Scribe)).await {
                                                            tracing::error!("Failed to start scribe turn for {plan}: {e}");
                                                            state.add_log("executor", &format!("Scribe turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                        }
                                                        state.parallel_agents.push(crate::state::ParallelAgentState {
                                                            instance_id: scribe_iid,
                                                            role: AgentRole::Scribe,
                                                            plan: plan.clone(),
                                                            task: "scribe".to_string(),
                                                            output: String::new(),
                                                            input_tokens: 0,
                                                            output_tokens: 0,
                                                            cost_usd: 0.0,
                                                            active: true,
                                                            finished_at: None,
                                                            model: String::new(),
                                                            turn_started: false,
                                                                            render_cache: Default::default(),
                                                        });
                                                        let mut pending = std::collections::HashSet::new();
                                                        pending.insert(AgentRole::Scribe);
                                                        state.plan_pending_reviews.insert(plan.clone(), pending);
                                                        // Set stage to DocRevisionScribePending to distinguish from initial audit+scribe
                                                        // (prevents stale original scribe event from triggering critic)
                                                        state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::DocRevisionScribePending);
                                                    } else {
                                                        // Scribe spawn failed — merge anyway
                                                        state.plan_review_stage.remove(&plan);
                                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                                        execute_actions(
                                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                            &spawn_ready_tx,
                                                        ).await?;
                                                    }
                                                } else {
                                                    state.plan_review_stage.remove(&plan);
                                                    let actions = executor.handle_plan_reviews_passed(&plan);
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                }
                                            } else {
                                                state.plan_review_stage.remove(&plan);
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                            } // end else (under cap)
                                        }
                                        Verdict::Revise { .. } => {
                                            // Missing with substantial output — retry critic once, then merge with warning
                                            let retry_key = format!("critic-missing-output:{plan}");
                                            let retry_count = {
                                                let retries = state.plan_agent_retries.entry(retry_key.clone()).or_insert(0);
                                                *retries += 1;
                                                *retries
                                            };
                                            if retry_count <= 1 {
                                                state.add_log("executor", &format!(
                                                    "Critic MISSING for {plan} (has output, {output_len} chars) — retrying once"
                                                ), LogLevel::Warn);
                                                let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                                let pi = crate::orchestrator::plan::discover_plans(
                                                    &config.repo_root.join("plans"), &[plan_num],
                                                ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));
                                                if let Some(ref plan_info) = pi {
                                                    let iid_critic = format!("critic:{plan}");
                                                    let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                                    let effort = state.config.effort_for(AgentRole::Critic).label();
                                                    if let Ok(prompt) = crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                        if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                            pool.set_thread_id(&aid, None);
                                                            if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                                tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                                state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                            }
                                                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid_critic) {
                                                                pa.active = true;
                                                                pa.finished_at = None;
                                                                pa.output.clear();
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                state.add_log("executor", &format!(
                                                    "Critic MISSING for {plan} after retry (output present) — merging with warning"
                                                ), LogLevel::Warn);
                                                state.plan_review_stage.remove(&plan);
                                                state.plan_pending_reviews.remove(&plan);
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        }
                                    }
                                }

                                // ── Scribe doc-revision completed ─
                                ("scribe", Some(crate::state::ReviewStage::DocRevisionScribePending)) => {
                                    state.plan_pending_reviews.remove(&plan);
                                    state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::CriticPending);
                                    state.add_log("executor", &format!("Scribe doc-revision done for {plan} → critic"), LogLevel::Info);

                                    let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                    let pi = crate::orchestrator::plan::discover_plans(
                                        &config.repo_root.join("plans"), &[plan_num],
                                    ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));

                                    if let Some(ref plan_info) = pi {
                                        let iid_critic = format!("critic:{plan}");
                                        let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                        let effort = state.config.effort_for(AgentRole::Critic).label();
                                        match crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                            Ok(prompt) => {
                                                if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                    pool.set_thread_id(&aid, None);
                                                    if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                        tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                        state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                    }
                                                    state.parallel_agents.push(crate::state::ParallelAgentState {
                                                        instance_id: iid_critic,
                                                        role: AgentRole::Critic,
                                                        plan: plan.clone(),
                                                        task: "critic".to_string(),
                                                        output: String::new(),
                                                        input_tokens: 0,
                                                        output_tokens: 0,
                                                        cost_usd: 0.0,
                                                        active: true,
                                                        finished_at: None,
                                                        model: String::new(),
                                                        turn_started: false,
                                                                        render_cache: Default::default(),
                                                    });
                                                } else {
                                                    state.plan_review_stage.remove(&plan);
                                                    let actions = executor.handle_plan_reviews_passed(&plan);
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                }
                                            }
                                            Err(_) => {
                                                state.plan_review_stage.remove(&plan);
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        }
                                    } else {
                                        state.plan_review_stage.remove(&plan);
                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                }

                                // ── Scribe retry completed (critic REVISE cycle) ─
                                ("scribe", Some(crate::state::ReviewStage::CriticPending)) => {
                                    // This shouldn't normally happen — stage should be DocRevisionScribePending.
                                    // But if it does (stale scribe event), check if critic is already running before spawning another.
                                    let already_running = state.parallel_agents.iter()
                                        .any(|a| a.instance_id == format!("critic:{plan}") && a.active);
                                    if already_running {
                                        state.add_log("executor", &format!(
                                            "Scribe retry event for {plan} ignored — critic already running"
                                        ), LogLevel::Warn);
                                    } else {
                                        state.plan_pending_reviews.remove(&plan);
                                        state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::CriticPending);
                                        state.add_log("executor", &format!("Scribe retry done for {plan} → critic"), LogLevel::Info);

                                        let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                        let pi = crate::orchestrator::plan::discover_plans(
                                            &config.repo_root.join("plans"), &[plan_num],
                                        ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));

                                        if let Some(ref plan_info) = pi {
                                            let iid_critic = format!("critic:{plan}");
                                            let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                            let effort = state.config.effort_for(AgentRole::Critic).label();
                                            match crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                Ok(prompt) => {
                                                    if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                    pool.set_thread_id(&aid, None);
                                                    if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                        tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                        state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                    }
                                                    state.parallel_agents.push(crate::state::ParallelAgentState {
                                                        instance_id: iid_critic,
                                                        role: AgentRole::Critic,
                                                        plan: plan.clone(),
                                                        task: "critic".to_string(),
                                                        output: String::new(),
                                                        input_tokens: 0,
                                                        output_tokens: 0,
                                                        cost_usd: 0.0,
                                                        active: true,
                                                        finished_at: None,
                                                        model: String::new(),
                                                        turn_started: false,
                                                                        render_cache: Default::default(),
                                                    });
                                                } else {
                                                    state.plan_review_stage.remove(&plan);
                                                    let actions = executor.handle_plan_reviews_passed(&plan);
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                }
                                            }
                                            Err(_) => {
                                                state.plan_review_stage.remove(&plan);
                                                let actions = executor.handle_plan_reviews_passed(&plan);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        }
                                    } else {
                                        state.plan_review_stage.remove(&plan);
                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                    }
                                }

                                // ── Fallback for unrecognized stage/prefix combos ─
                                _ => {
                                    state.add_log("executor", &format!(
                                        "Unexpected review event ({prefix}, {stage:?}) for {plan} — ignoring"
                                    ), LogLevel::Warn);
                                    // Remove from pending tracking only — do not advance pipeline.
                                    // This prevents stale events from triggering premature merges.
                                    let role_key = match prefix {
                                        "arch"  => AgentRole::Architect,
                                        "audit" => AgentRole::Auditor,
                                        "scribe" => AgentRole::Scribe,
                                        _       => AgentRole::Critic,
                                    };
                                    if let Some(pending) = state.plan_pending_reviews.get_mut(&plan) {
                                        pending.remove(&role_key);
                                        // intentionally do NOT call handle_plan_reviews_passed here
                                    }
                                }
                            }
                        }
                    }
                    AgentEvent::TurnCompleted { role, .. } => {
                        // Non-instance agent (legacy path)
                        info!("Non-instance turn completed for {role}");
                    }
                    AgentEvent::MessageDelta { role, content, instance, .. } => {
                        last_message_at = Some(std::time::Instant::now());
                        // NOTE: Minimal output buffering to reduce CPU cost.
                        // Only keep last 512 bytes per agent for UI display (enough to show current work).
                        // Full output is already persisted at task completion and checkpoints.
                        if let Some(ref iid) = instance {
                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                // Keep only last 512 chars to avoid expensive string operations every message
                                const MAX_OUTPUT: usize = 512;
                                pa.output.push_str(&content);
                                if pa.output.len() > MAX_OUTPUT {
                                    let mut truncate_at = pa.output.len().saturating_sub(MAX_OUTPUT);
                                    // Find a valid UTF-8 boundary (not a continuation byte: 0b10xxxxxx)
                                    while truncate_at > 0 && (pa.output.as_bytes()[truncate_at] & 0xC0) == 0x80 {
                                        truncate_at -= 1;
                                    }
                                    pa.output.drain(..truncate_at);
                                }
                            }
                        }
                        // Don't buffer full output to main agent state (causes rendering overhead).
                        // Role-level output is rarely displayed; per-instance output is what matters.
                    }
                    AgentEvent::CommandOutput { content, .. } => {
                        state.command_output.push_str(&content);
                    }
                    AgentEvent::ApprovalRequested { role, approval_id, instance, .. } => {
                        // Auto-approve in parallel mode
                        let aid = if let Some(ref iid) = instance {
                            AgentInstanceId::new(role, iid.clone())
                        } else {
                            AgentInstanceId::default_for(role)
                        };
                        let _ = pool.respond_approval(&aid, &approval_id, true).await;
                    }
                    AgentEvent::TokenUsage { role, input_tokens, output_tokens, context_window, instance, cost_usd, .. } => {
                        if let Some(ref iid) = instance {
                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                pa.input_tokens = input_tokens;
                                pa.output_tokens = output_tokens;
                                // Accumulate cost to per-instance and global totals
                                if let Some(cost) = cost_usd {
                                    pa.cost_usd += cost;
                                    state.cumulative_cost_usd += cost;
                                    *state.cost_per_plan.entry(pa.plan.clone()).or_insert(0.0) += cost;
                                }
                            }
                        }
                        let agent = state.agent_state_mut(role);
                        agent.input_tokens = input_tokens;
                        agent.output_tokens = output_tokens;
                        if let Some(window) = context_window {
                            state.context_limit = window;
                        }
                    }
                    AgentEvent::Error { role, error, .. } => {
                        state.add_log(&role.to_string(), &error, LogLevel::Error);
                        warn!("Agent error ({}): {}", role, error);
                    }
                    AgentEvent::Exited { role, exit_code, ref instance } => {
                        let iid = instance.clone().unwrap_or_default();
                        state.add_log(&role.to_string(), &format!("Exited {:?} ({})", exit_code, iid), LogLevel::Warn);

                        // Mark the parallel agent as inactive
                        if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                            pa.active = false;
                            pa.finished_at = Some(std::time::Instant::now());
                        }

                        // If this was a review agent, unblock the review pipeline
                        if let Some((prefix, plan)) = review_plan_from_iid(&iid) {
                            // Retry once before advancing
                            let retry_key = format!("{prefix}:{plan}");
                            let retry_count = {
                                let retries = state.plan_agent_retries.entry(retry_key.clone()).or_insert(0);
                                *retries += 1;
                                *retries
                            };
                            if retry_count <= 1 {
                                state.add_log("executor", &format!(
                                    "{prefix} exited for {plan} — retrying (attempt {retry_count})"
                                ), LogLevel::Warn);
                                // Re-spawn the same agent
                                let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                let pi = crate::orchestrator::plan::discover_plans(
                                    &config.repo_root.join("plans"), &[plan_num],
                                ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));
                                if let Some(ref plan_info) = pi {
                                    let iteration = executor.plan_iteration(&plan);
                                    let worktree_fallback = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                    let wt_mgr = worktree_mgr.clone();
                                    let plan_clone = plan.clone();
                                    let batch_clone = batch_branch.to_string();
                                    let wd = match tokio::task::spawn_blocking(move || {
                                        wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                                    }).await {
                                        Ok(Ok(pw)) => Some(pw.path),
                                        Ok(Err(_)) | Err(_) => {
                                            if worktree_fallback.exists() { Some(worktree_fallback) } else { None }
                                        }
                                    };
                                    let prompt_result = match prefix {
                                        "arch" => crate::orchestrator::prompts::combined_reviewer_prompt(&config.repo_root, plan_info, iteration, wd.as_deref()),
                                        "scribe" => crate::orchestrator::prompts::scribe_prompt(&config.repo_root, plan_info, wd.as_deref()),
                                        "critic" => crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()),
                                        "quick" => crate::orchestrator::prompts::quick_reviewer_prompt(&config.repo_root, plan_info, iteration, wd.as_deref()),
                                        _ => Err(anyhow::anyhow!("unknown prefix")),
                                    };
                                    let role_for_prefix = match prefix {
                                        "arch" => AgentRole::Architect,
                                        "audit" => AgentRole::Auditor,
                                        "scribe" => AgentRole::Scribe,
                                        "critic" => AgentRole::Critic,
                                        "quick" => AgentRole::QuickReviewer,
                                        _ => AgentRole::Architect,
                                    };
                                    if let Ok(prompt) = prompt_result {
                                        let retry_iid = iid.clone();
                                        let aid = AgentInstanceId::new(role_for_prefix, retry_iid.clone());
                                        let effort = state.config.effort_for(role_for_prefix).label();
                                        if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(role_for_prefix)).await.is_ok() {
                                            pool.set_thread_id(&aid, None);
                                            if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(role_for_prefix)).await {
                                                tracing::error!("Failed to start {prefix} retry turn for {plan}: {e}");
                                                state.add_log("executor", &format!("{prefix} retry turn_start failed for {plan}: {e}"), LogLevel::Error);
                                            }
                                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == retry_iid) {
                                                pa.active = true;
                                                pa.finished_at = None;
                                            }
                                        }
                                        // Skip the fallthrough — retry in progress
                                        continue;
                                    }
                                }
                                // Retry failed to build prompt — fall through to normal handling
                            }

                            let stage = state.plan_review_stage.get(&plan).cloned();
                            match (prefix, &stage) {
                                // ── QuickReviewer died ───────────────────────────
                                ("quick", Some(crate::state::ReviewStage::ReviewerPending)) => {
                                    state.plan_review_stage.remove(&plan);
                                    state.plan_pending_reviews.remove(&plan);
                                    state.add_log("executor", &format!(
                                        "QuickReviewer exited for {plan} — skipping review, merging"
                                    ), LogLevel::Warn);
                                    // Guard: only merge if plan is still in Reviewing phase
                                    // (prevents acting on stale AgentExited events)
                                    if let Some(PlanPhase::Reviewing) = executor.plan_phase(&plan) {
                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    } else {
                                        state.add_log("executor", &format!(
                                            "QuickReviewer exit for {plan} ignored — plan not in Reviewing phase"
                                        ), LogLevel::Warn);
                                    }
                                }
                                ("arch", Some(crate::state::ReviewStage::ReviewerPending)) => {
                                    let arch_output = get_parallel_agent_output(&state, &iid);
                                    if arch_output.len() < 50 {
                                        // Reviewer crashed with no meaningful output — re-implement
                                        state.add_log("executor", &format!(
                                            "Reviewer exited for {plan} with no output — re-implementing"
                                        ), LogLevel::Warn);
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        state.plan_doc_revisions.remove(&plan);
                                        let actions = executor.handle_plan_gates_failed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                        continue;
                                    }
                                    // Reviewer died but produced output — treat as non-blocking
                                    state.add_log("executor", &format!(
                                        "Reviewer exited for {plan} — treating as non-blocking, spawning scribe"
                                    ), LogLevel::Warn);
                                    state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::ScribePending);
                                    state.plan_pending_reviews.remove(&plan);
                                    // Re-discover plan info and spawn scribe
                                    let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                    let pi = crate::orchestrator::plan::discover_plans(
                                        &config.repo_root.join("plans"), &[plan_num],
                                    ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));
                                    if let Some(ref plan_info) = pi {
                                        let worktree_fallback = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                        let wt_mgr = worktree_mgr.clone();
                                        let plan_clone = plan.clone();
                                        let batch_clone = batch_branch.to_string();
                                        let wd = match tokio::task::spawn_blocking(move || {
                                            wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                                        }).await {
                                            Ok(Ok(pw)) => Some(pw.path),
                                            Ok(Err(_)) | Err(_) => {
                                                if worktree_fallback.exists() { Some(worktree_fallback) } else { None }
                                            }
                                        };
                                        let mut pending = std::collections::HashSet::new();
                                        if let Ok(prompt) = crate::orchestrator::prompts::scribe_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                            let pfx = "scribe";
                                            let review_iid = format!("{pfx}:{plan}");
                                            let aid = AgentInstanceId::new(AgentRole::Scribe, review_iid.clone());
                                            let effort = state.config.effort_for(AgentRole::Scribe).label();
                                            if pool.spawn_instance(aid.clone(), wd.clone(), effort, state.config.model_for(AgentRole::Scribe)).await.is_ok() {
                                                pool.set_thread_id(&aid, None);
                                                if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Scribe)).await {
                                                    tracing::error!("Failed to start scribe turn for {plan}: {e}");
                                                    state.add_log("executor", &format!("scribe turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                }
                                                state.parallel_agents.push(crate::state::ParallelAgentState {
                                                    instance_id: review_iid,
                                                    role: AgentRole::Scribe,
                                                    plan: plan.clone(),
                                                    task: pfx.to_string(),
                                                    output: String::new(),
                                                    input_tokens: 0,
                                                    output_tokens: 0,
                                                    cost_usd: 0.0,
                                                    active: true,
                                                    finished_at: None,
                                                    model: String::new(),
                                                    turn_started: false,
                                                                    render_cache: Default::default(),
                                                });
                                                pending.insert(AgentRole::Scribe);
                                            }
                                        }
                                        if pending.is_empty() {
                                            // Scribe spawn failed — skip to merge
                                            state.plan_review_stage.remove(&plan);
                                            let actions = executor.handle_plan_reviews_passed(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        } else {
                                            state.plan_pending_reviews.insert(plan, pending);
                                        }
                                    } else {
                                        state.plan_review_stage.remove(&plan);
                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                }
                                ("scribe", Some(crate::state::ReviewStage::ScribePending)) => {
                                    // Scribe died — remove from pending, advance if all done
                                    let all_done = if let Some(pending) = state.plan_pending_reviews.get_mut(&plan) {
                                        pending.remove(&AgentRole::Scribe);
                                        pending.is_empty()
                                    } else {
                                        false // missing entry means pending was already cleared — do not advance
                                    };
                                    if all_done {
                                        state.add_log("executor", &format!(
                                            "Scribe exited for {plan} — advancing to critic"
                                        ), LogLevel::Warn);
                                        state.plan_pending_reviews.remove(&plan);
                                        state.plan_review_stage.insert(plan.clone(), crate::state::ReviewStage::CriticPending);
                                        let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();
                                        let pi = crate::orchestrator::plan::discover_plans(
                                            &config.repo_root.join("plans"), &[plan_num],
                                        ).ok().and_then(|ps| ps.into_iter().find(|p| p.base == plan));
                                        if let Some(ref plan_info) = pi {
                                            let worktree_fallback = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                            let wt_mgr = worktree_mgr.clone();
                                            let plan_clone = plan.clone();
                                            let batch_clone = batch_branch.to_string();
                                            let wd = match tokio::task::spawn_blocking(move || {
                                                wt_mgr.create_plan_worktree(&plan_clone, &batch_clone)
                                            }).await {
                                                Ok(Ok(pw)) => Some(pw.path),
                                                Ok(Err(_)) | Err(_) => {
                                                    if worktree_fallback.exists() { Some(worktree_fallback) } else { None }
                                                }
                                            };
                                            let iid_critic = format!("critic:{plan}");
                                            let aid = AgentInstanceId::new(AgentRole::Critic, iid_critic.clone());
                                            let effort = state.config.effort_for(AgentRole::Critic).label();
                                            match crate::orchestrator::prompts::critic_prompt(&config.repo_root, plan_info, wd.as_deref()) {
                                                Ok(prompt) => {
                                                    if pool.spawn_instance(aid.clone(), wd, effort, state.config.model_for(AgentRole::Critic)).await.is_ok() {
                                                        pool.set_thread_id(&aid, None);
                                                        if let Err(e) = pool.turn_start(&aid, &prompt, state.config.model_for(AgentRole::Critic)).await {
                                                            tracing::error!("Failed to start critic turn for {plan}: {e}");
                                                            state.add_log("executor", &format!("Critic turn_start failed for {plan}: {e}"), LogLevel::Error);
                                                        }
                                                        state.parallel_agents.push(crate::state::ParallelAgentState {
                                                            instance_id: iid_critic,
                                                            role: AgentRole::Critic,
                                                            plan: plan.clone(),
                                                            task: "critic".to_string(),
                                                            output: String::new(),
                                                            input_tokens: 0,
                                                            output_tokens: 0,
                                                            cost_usd: 0.0,
                                                            active: true,
                                                            finished_at: None,
                                                            model: String::new(),
                                                            turn_started: false,
                                                                            render_cache: Default::default(),
                                                        });
                                                    } else {
                                                        state.plan_review_stage.remove(&plan);
                                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                                        execute_actions(
                                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                            &spawn_ready_tx,
                                                        ).await?;
                                                    }
                                                }
                                                Err(_) => {
                                                    state.plan_review_stage.remove(&plan);
                                                    let actions = executor.handle_plan_reviews_passed(&plan);
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                }
                                            }
                                        } else {
                                            state.plan_review_stage.remove(&plan);
                                            let actions = executor.handle_plan_reviews_passed(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                }
                                ("critic", _) => {
                                    let critic_output = get_parallel_agent_output(&state, &iid);
                                    if critic_output.len() < 50 {
                                        // Critic crashed with no meaningful output — mark Failed
                                        state.add_log("executor", &format!(
                                            "Critic exited for {plan} with no output — marking Failed"
                                        ), LogLevel::Error);
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                                            ps.phase = PlanPhase::Failed("critic agent crashed".to_string());
                                        }
                                        if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                            entry.status = RunPlanStatus::Failed;
                                        }
                                        let drain_actions = executor.drain_merge_queue();
                                        execute_actions(
                                            drain_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    } else {
                                        // Critic died but produced output — treat as non-blocking, merge
                                        state.add_log("executor", &format!(
                                            "Critic exited for {plan} — treating as non-blocking"
                                        ), LogLevel::Warn);
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        let actions = executor.handle_plan_reviews_passed(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                }
                                _ => {}
                            }
                        } else if iid.starts_with("express-impl:") {
                            // Express implementer exited — treat as complete and run gates
                            let plan = iid.strip_prefix("express-impl:").unwrap_or(&iid).to_string();
                            state.add_log("executor", &format!(
                                "Express implementer exited for {plan} — advancing to gates"
                            ), LogLevel::Warn);
                            let whole_id = crate::orchestrator::GlobalTaskId {
                                plan: plan.clone(),
                                task: "__whole__".to_string(),
                            };
                            let actions = executor.handle_task_complete(whole_id);
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if iid.starts_with("auto-fix:") {
                            // Auto-fixer exited — re-run gates regardless
                            let plan = iid.strip_prefix("auto-fix:").unwrap_or(&iid).to_string();
                            state.add_log("executor", &format!(
                                "Auto-fixer exited for {plan} — re-running gates"
                            ), LogLevel::Warn);
                            let actions = executor.handle_auto_fix_complete(&plan);
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        } else if !iid.is_empty() {
                            // Implementer task agent died — check output to decide retry vs complete (batch-aware).
                            let instance_tasks = executor.tasks_for_instance(&iid);
                            if !instance_tasks.is_empty() {
                                let agent_output = get_parallel_agent_output(&state, &iid);
                                let output_len = agent_output.len();
                                let input_tokens = state.parallel_agents.iter()
                                    .find(|p| p.instance_id == iid)
                                    .map(|p| p.input_tokens)
                                    .unwrap_or(0);
                                info!("exited[{}] tasks={} output_len={} input_tokens={} role={:?}",
                                    iid, instance_tasks.len(), output_len, input_tokens, role);
                                let actions = if output_len < 50 {
                                    // Apply spawn backoff so retries don't race.
                                    // Extract plan name from instance tasks.
                                    if let Some(first_task) = instance_tasks.first() {
                                        executor.record_spawn_failure(&first_task.plan);
                                    }
                                    state.add_log("executor", &format!(
                                        "Implementer exited for {} with no output ({}chars, {}tok) — will retry with backoff",
                                        iid, output_len, input_tokens
                                    ), LogLevel::Warn);
                                    executor.handle_instance_failed(&iid)
                                } else {
                                    for task_id in &instance_tasks {
                                        state.executor_completed_tasks.insert(task_id.to_string());
                                    }
                                    state.add_log("executor", &format!(
                                        "Implementer exited for {} — marking complete ({} tasks, {}chars, {}tok)",
                                        iid, instance_tasks.len(), output_len, input_tokens
                                    ), LogLevel::Warn);
                                    executor.handle_instance_complete(&iid)
                                };
                                parallel_refresh_tasks(&mut state, &config);
                                // Kill agent connection
                                let aid = AgentInstanceId::new(role, iid.clone());
                                pool.kill_instance(&aid).await;
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;

                                // Checkpoint immediately after task completion
                                let (inp_tok, out_tok) = (
                                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                                );
                                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some(completion) = gate_rx.recv() => {
                match completion {
                    GateCompletion::Compile { plan, result } => {
                        state.gate_running.remove(&format!("cargo clippy ({})", plan));
                        let gate = match result {
                            Ok(g) => g,
                            Err(e) => {
                                state.add_log("gate", &format!("Compile gate error ({plan}): {e}"), LogLevel::Error);
                                // Cancel active reviewer if overlapping
                                let mut actions = vec![];
                                if let Some(reviewer_id) = executor.get_active_reviewer(&plan) {
                                    actions.push(ExecutorAction::CancelActiveReviewer {
                                        plan: plan.clone(),
                                        instance_id: reviewer_id,
                                    });
                                }
                                state.plan_doc_revisions.remove(&plan);
                                actions.extend(executor.handle_plan_gates_failed(&plan));
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                                continue;
                            }
                        };
                        state.command_output = gate.output.clone();
                        state.plan_gate_outputs.insert(plan.clone(), gate.output.clone());
                        if gate.passed {
                            state.add_log("gate", &format!("Compile PASS ({plan})"), LogLevel::Info);
                            // Update ETA correction factor based on actual elapsed time
                            if let Some(started) = state.plan_start_times.get(&plan) {
                                let elapsed_mins = (started.elapsed().as_secs() / 60) as u32;
                                state.time_estimator.record_plan_complete(&plan, elapsed_mins);
                            }
                            let actions = executor.handle_plan_gates_passed(&plan);
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;

                            // Phase validation: ask conductor if implementation looks OK
                            let iteration = executor.plan_iteration(&plan);
                            state.pending_phase_validation = Some((plan.clone(), "gates_passed".to_string()));
                            parallel_consult_conductor(
                                &mut state, &mut pool,
                                &format!("Phase validation: gates PASSED for {plan} (iter {iteration}). Does the implementation look correct?"),
                                &format!("Plan {plan} passed compile+test gates. Verify implementation quality before reviews proceed."),
                            ).await;
                        } else {
                            state.add_log("gate", &format!("Compile FAIL ({plan})"), LogLevel::Error);
                            // Cancel active reviewer if overlapping
                            let mut actions = vec![];
                            if let Some(reviewer_id) = executor.get_active_reviewer(&plan) {
                                actions.push(ExecutorAction::CancelActiveReviewer {
                                    plan: plan.clone(),
                                    instance_id: reviewer_id,
                                });
                            }
                            state.plan_doc_revisions.remove(&plan);
                            actions.extend(executor.handle_plan_gates_failed_with_errors(&plan, gate.output.clone()));
                            execute_actions(
                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        }
                    }
                    GateCompletion::Clippy { .. } | GateCompletion::Test { .. } => {
                        // In parallel mode, just log and move on
                        state.add_log("gate", "Gate completed", LogLevel::Info);
                    }
                    GateCompletion::TerminalRender { plan, result } => {
                        match result {
                            Ok(gate) => {
                                if gate.passed {
                                    state.add_log("gate", &format!("Terminal render gate PASS ({plan})"), LogLevel::Info);
                                } else {
                                    state.add_log("gate", &format!("Terminal render gate WARN ({plan}): {}", gate.output.lines().last().unwrap_or("")), LogLevel::Warn);
                                }
                            }
                            Err(e) => {
                                state.add_log("gate", &format!("Terminal render gate ERROR ({plan}): {e}"), LogLevel::Warn);
                            }
                        }
                    }
                    GateCompletion::GolemLifecycle { plan, result } => {
                        match result {
                            Ok(gate) => {
                                if gate.passed {
                                    let count = gate.test_count.as_ref().map_or(0, |c| c.passed);
                                    state.add_log("gate", &format!("Golem lifecycle gate PASS ({plan}, {count} tests)"), LogLevel::Info);
                                } else {
                                    state.add_log("gate", &format!("Golem lifecycle gate FAIL ({plan}): {}", gate.output.lines().last().unwrap_or("")), LogLevel::Error);
                                }
                            }
                            Err(e) => {
                                state.add_log("gate", &format!("Golem lifecycle gate ERROR ({plan}): {e}"), LogLevel::Warn);
                            }
                        }
                    }
                    GateCompletion::PostMerge { plan, result } => {
                        match result {
                            Ok(gate) if gate.passed => {
                                state.add_log("gate", &format!("Post-merge check PASS ({plan})"), LogLevel::Info);
                            }
                            Ok(gate) => {
                                state.add_log("gate", &format!("Post-merge check FAIL ({plan}): {}", gate.output.lines().last().unwrap_or("")), LogLevel::Error);
                            }
                            Err(e) => {
                                state.add_log("gate", &format!("Post-merge check ERROR ({plan}): {e}"), LogLevel::Error);
                            }
                        }
                    }
                    GateCompletion::MergeComplete { plan, success, error, commit_count } => {
                        if success {
                            // Spawn post-merge integration check
                            {
                                let repo = config.repo_root.clone();
                                let tx = gate_tx.clone();
                                let pm_plan = plan.clone();
                                tokio::spawn(async move {
                                    let result = crate::orchestrator::gates::post_merge_compile_check(&repo).await;
                                    let _ = tx.send(GateCompletion::PostMerge { plan: pm_plan, result });
                                });
                            }

                            info!("merge[{plan}] success, commits={commit_count}");

                            // Plan completion summary
                            {
                                let elapsed = state.plan_start_times.get(&plan)
                                    .map(|t| crate::state::format_duration(t.elapsed().as_secs()))
                                    .unwrap_or_else(|| "unknown".to_string());
                                let files_changed = commit_count.parse::<u32>().unwrap_or(0);
                                info!(
                                    "plan_complete[{plan}] time={elapsed} commits={commit_count} files_changed={files_changed}"
                                );
                            }

                            let merged_actions = executor.handle_plan_merged(&plan);
                            // Write post-merge checkpoint (merge_in_progress now cleared, correction_factor persisted)
                            {
                                let (inp_tok, out_tok) = (
                                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                                );
                                write_checkpoint_with_state(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok, &state);
                            }
                            if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                entry.status = RunPlanStatus::Completed;
                                entry.phase = "complete".to_string();
                            }
                            // Clean up parallel agents for this completed plan
                            state.parallel_agents.retain(|a| {
                                !(a.plan == plan || a.instance_id.contains(&plan))
                            });
                            state.add_log("executor", &format!("Plan {plan} merged (commits={commit_count})"), LogLevel::Info);

                            let event = crate::state::persistence::PersistenceManager::make_task_event(
                                "plan_merged", &plan, None, None, None,
                            );
                            let _ = persistence.append_task_event(&event);

                            execute_actions(
                                merged_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;

                            // Phase validation: conductor checks merge quality
                            {
                                let prog = executor.progress();
                                state.pending_phase_validation = Some((plan.clone(), "merged".to_string()));
                                parallel_consult_conductor(
                                    &mut state, &mut pool,
                                    &format!("Phase validation: {plan} merged (commits={commit_count}). {}/{} plans complete.",
                                        prog.completed_plans, prog.total_plans),
                                    &format!("Plan {plan} merged to batch branch. Verify merge looks clean."),
                                ).await;
                            }

                            // Clean up the completed plan's worktree to free disk space
                            {
                                let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                if wt_path.exists() {
                                    let pw = crate::git::worktree::PlanWorktree {
                                        path: wt_path,
                                        branch: format!("codex/plan/{plan}"),
                                        plan_base: plan.clone(),
                                    };
                                    if let Err(e) = worktree_mgr.cleanup_plan_worktree(&pw) {
                                        warn!("Failed to clean up worktree for {plan}: {e}");
                                    } else {
                                        info!("Cleaned up worktree for completed plan {plan}");
                                    }
                                }
                            }

                            // Prune stale worktrees in background after successful merge
                            let repo = config.repo_root.clone();
                            tokio::task::spawn_blocking(move || {
                                let _ = crate::git::ops::run_git(&repo, &["worktree", "prune"]);
                            });
                        } else {
                            let err_msg = error.as_deref().unwrap_or("unknown");
                            state.add_log("git", &format!("Merge failed for {plan}: {err_msg}"), LogLevel::Error);

                            // Clean up dirty state in background to avoid blocking the event loop
                            let repo = config.repo_root.clone();
                            let batch = batch_branch.to_string();
                            tokio::task::spawn_blocking(move || {
                                let _ = crate::git::ops::run_git(&repo, &["merge", "--abort"]);
                                let _ = crate::git::ops::run_git(&repo, &["reset", "--hard"]);
                                let _ = crate::git::ops::run_git(&repo, &["checkout", &batch]);
                            });

                            executor.merge_in_progress = false;
                            executor.currently_merging = None;
                            // Write post-merge-failure checkpoint (merge_in_progress now cleared)
                            {
                                let (inp_tok, out_tok) = (
                                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                                );
                                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                            }
                            let iteration = executor.plan_states.get(plan.as_str())
                                .map(|s| s.iteration)
                                .unwrap_or(1);
                            if iteration < 3 {
                                state.add_log("executor", &format!(
                                    "Plan {plan} merge failed (attempt {iteration}) — auto-recovering: re-implementing"
                                ), LogLevel::Warn);
                                // Clean worktree
                                let wt_path2 = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                if wt_path2.exists() {
                                    let pw2 = crate::git::worktree::PlanWorktree {
                                        path: wt_path2,
                                        branch: format!("codex/plan/{plan}"),
                                        plan_base: plan.clone(),
                                    };
                                    let _ = worktree_mgr.cleanup_plan_worktree(&pw2);
                                }
                                let plan_branch = format!("codex/plan/{plan}");
                                let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                state.plan_doc_revisions.remove(&plan);
                                let fail_actions = executor.handle_plan_gates_failed(&plan);
                                execute_actions(
                                    fail_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            } else {
                                if let Some(ps) = executor.plan_states.get_mut(plan.as_str()) {
                                    ps.phase = PlanPhase::Failed(format!("merge failed after {iteration} attempts"));
                                }
                                if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                    entry.status = RunPlanStatus::Failed;
                                }
                                state.add_log("executor", &format!(
                                    "Plan {plan} merge failed {iteration} times — giving up (ctrl+d to reset)"
                                ), LogLevel::Error);
                            }
                            // Drain merge queue for other plans
                            let drain_actions = executor.drain_merge_queue();
                            execute_actions(
                                drain_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                &spawn_ready_tx,
                            ).await?;
                        }
                    }
                    GateCompletion::ReconcileComplete { messages, merged_plans, already_reconciled } => {
                        state.git_reconcile_in_progress = false;
                        for msg in &messages {
                            let level = if msg.contains("ERROR") { LogLevel::Error } else { LogLevel::Info };
                            state.add_log("reconcile", msg, level);
                        }
                        // Sync newly merged plans with executor
                        for plan in &merged_plans {
                            if let Some(entry) = state.plans.iter_mut().find(|p| &p.base == plan) {
                                entry.status = RunPlanStatus::Completed;
                                entry.phase = "complete".to_string();
                            }
                            let _ = executor.handle_plan_merged(plan);
                            // Clean up parallel agents for this completed plan
                            state.parallel_agents.retain(|a| {
                                !(a.plan == *plan || a.instance_id.contains(plan.as_str()))
                            });
                            let event = crate::state::persistence::PersistenceManager::make_task_event(
                                "plan_merged", plan, None, None, None,
                            );
                            let _ = persistence.append_task_event(&event);
                        }
                        // Sync already-reconciled plans that the executor didn't know about
                        let already_complete: std::collections::HashSet<String> = executor.completed_plan_names().into_iter().collect();
                        for plan in &already_reconciled {
                            if !already_complete.contains(plan) {
                                if let Some(entry) = state.plans.iter_mut().find(|p| &p.base == plan) {
                                    entry.status = RunPlanStatus::Completed;
                                    entry.phase = "complete".to_string();
                                }
                                let _ = executor.handle_plan_merged(plan);
                                // Clean up parallel agents for this reconciled plan
                                state.parallel_agents.retain(|a| {
                                    !(a.plan == *plan || a.instance_id.contains(plan.as_str()))
                                });
                                state.add_log("reconcile", &format!("{plan}: synced to executor as complete"), LogLevel::Info);
                            }
                        }
                        let total = merged_plans.len();
                        let synced = already_reconciled.iter().filter(|p| !merged_plans.contains(p)).count();
                        let summary = match (total, synced) {
                            (0, 0) => "Reconcile done: everything was already up-to-date".to_string(),
                            (n, 0) => format!("Reconcile done: {n} plan(s) newly merged"),
                            (0, s) => format!("Reconcile done: {s} plan(s) synced to executor"),
                            (n, s) => format!("Reconcile done: {n} newly merged, {s} synced"),
                        };
                        state.add_log("reconcile", &summary, LogLevel::Info);
                        state.notifications.push(crate::state::Notification {
                            message: summary,
                            created: std::time::Instant::now(),
                            ttl_secs: 8,
                            level: LogLevel::Info,
                        });
                    }
                }
            }
            Some(result) = term_events.next() => {
                match result {
                    Ok(Event::Key(key)) => {
                    last_user_input = Instant::now();
                    info!(
                        "KEY: code={:?} kind={:?} modifiers={:?} tab={} mode={:?}",
                        key.code, key.kind, key.modifiers,
                        state.active_tab, state.input_mode
                    );
                    if key.kind != KeyEventKind::Press { continue; }
                    let sel_plan = state.plans.get(state.selected_plan_idx).map(|p| p.base.as_str()).unwrap_or("");
                    let action = crate::tui::input::handle_key(
                        key, &state.input_mode, &state.message_input,
                        &state.focus, state.show_plan_detail, state.active_tab,
                        &state.agent_pane_group, state.show_task_detail, sel_plan,
                        state.show_task_picker,
                    );
                    match action {
                        TuiAction::Quit => break,
                        TuiAction::SwitchTab(idx) => {
                            if idx < 6 {
                                let prev = state.active_tab;
                                state.active_tab = idx;
                                // Clear pipeline header when leaving Plans tab
                                if prev == 1 && idx != 1 {
                                    state.pipeline_header_selected = false;
                                }
                                if prev == 0 && idx == 1 && !state.execution_waves.is_empty() {
                                    if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                        for (wi, (_, bases)) in state.execution_waves.iter().enumerate() {
                                            if let Some(pi) = bases.iter().position(|b| *b == plan.base) {
                                                state.selected_wave_idx = wi;
                                                state.selected_plan_idx = pi;
                                                break;
                                            }
                                        }
                                    }
                                }
                                if prev == 1 && idx == 0 && !state.execution_waves.is_empty() {
                                    if let Some((_, bases)) = state.execution_waves.get(state.selected_wave_idx) {
                                        if let Some(base) = bases.get(state.selected_plan_idx) {
                                            if let Some(gi) = state.plans.iter().position(|p| &p.base == base) {
                                                state.selected_plan_idx = gi;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        TuiAction::SelectPlanUp => {
                            state.selected_plan_idx = state.selected_plan_idx.saturating_sub(1);
                            parallel_refresh_tasks(&mut state, &config);
                        }
                        TuiAction::SelectPlanDown => {
                            if state.selected_plan_idx + 1 < state.plans.len() {
                                state.selected_plan_idx += 1;
                            }
                            parallel_refresh_tasks(&mut state, &config);
                        }
                        TuiAction::NavigateUp => {
                            match state.active_tab {
                                1 => {
                                    if state.pipeline_header_selected {
                                        // already at top, nowhere to go
                                    } else if state.selected_plan_idx > 0 {
                                        state.selected_plan_idx -= 1;
                                    } else if state.selected_wave_idx > 0 {
                                        state.selected_wave_idx -= 1;
                                        let count = state.execution_waves
                                            .get(state.selected_wave_idx)
                                            .map(|(_, p)| p.len())
                                            .unwrap_or(1);
                                        state.selected_plan_idx = count.saturating_sub(1);
                                    } else {
                                        state.pipeline_header_selected = true;
                                    }
                                    // E1: Scroll-to-reveal
                                    ensure_selection_visible(&mut state);
                                    parallel_refresh_tasks(&mut state, &config);
                                }
                                2 => {
                                    // Agents view: navigate agent list
                                    state.agent_list_cursor = state.agent_list_cursor.saturating_sub(1);
                                }
                                3 => {
                                    // Git view: navigate branch tree
                                    state.git_branch_cursor = state.git_branch_cursor.saturating_sub(1);
                                }
                                _ => {}
                            }
                        }
                        TuiAction::NavigateDown => {
                            match state.active_tab {
                                1 => {
                                    if state.pipeline_header_selected {
                                        state.pipeline_header_selected = false;
                                    } else {
                                    let wave_plan_count = state.execution_waves
                                        .get(state.selected_wave_idx)
                                        .map(|(_, p)| p.len())
                                        .unwrap_or(state.plans.len());
                                    if state.selected_plan_idx + 1 < wave_plan_count {
                                        state.selected_plan_idx += 1;
                                    } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                        state.selected_wave_idx += 1;
                                        state.selected_plan_idx = 0;
                                    }
                                    }
                                    // E1: Scroll-to-reveal
                                    ensure_selection_visible(&mut state);
                                    parallel_refresh_tasks(&mut state, &config);
                                }
                                2 => {
                                    // Agents view: navigate agent list
                                    let max = if !state.parallel_agents.is_empty() {
                                        state.parallel_agents.len().saturating_sub(1)
                                    } else {
                                        state.agents.len().saturating_sub(1)
                                    };
                                    if state.agent_list_cursor < max {
                                        state.agent_list_cursor += 1;
                                    }
                                }
                                3 => {
                                    // Git view: navigate branch tree
                                    let max = state.git_branch_tree.len().saturating_sub(1);
                                    if state.git_branch_cursor < max {
                                        state.git_branch_cursor += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        TuiAction::NavigatePageUp => {
                            for _ in 0..10 {
                                if state.selected_plan_idx > 0 {
                                    state.selected_plan_idx -= 1;
                                } else if state.selected_wave_idx > 0 {
                                    state.selected_wave_idx -= 1;
                                    let count = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(1);
                                    state.selected_plan_idx = count.saturating_sub(1);
                                } else { break; }
                            }
                        }
                        TuiAction::NavigatePageDown => {
                            for _ in 0..10 {
                                let wpc = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(state.plans.len());
                                if state.selected_plan_idx + 1 < wpc {
                                    state.selected_plan_idx += 1;
                                } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                    state.selected_wave_idx += 1;
                                    state.selected_plan_idx = 0;
                                } else { break; }
                            }
                        }
                        TuiAction::WaveNext => {
                            state.pipeline_header_selected = false;
                            if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                state.selected_wave_idx += 1;
                                state.selected_plan_idx = 0;
                            }
                            parallel_refresh_tasks(&mut state, &config);
                        }
                        TuiAction::WavePrev => {
                            state.pipeline_header_selected = false;
                            if state.selected_wave_idx > 0 {
                                state.selected_wave_idx -= 1;
                                state.selected_plan_idx = 0;
                            }
                            parallel_refresh_tasks(&mut state, &config);
                        }
                        TuiAction::ScrollLogUp => { state.log_scroll = state.log_scroll.saturating_add(10); }
                        TuiAction::ScrollLogDown => { state.log_scroll = state.log_scroll.saturating_sub(10); }

                        // --- Agent / sub-tab navigation ---
                        TuiAction::SwitchAgentTab(idx) => {
                            state.manual_agent_tab = true;
                            if idx == usize::MAX {
                                // Cycle based on actual agent count for the selected plan
                                let agent_count = if !state.parallel_agents.is_empty() {
                                    let selected_base = state.plans.get(state.selected_plan_idx)
                                        .map(|p| p.base.as_str())
                                        .unwrap_or("");
                                    state.parallel_agents.iter()
                                        .filter(|p| p.plan.contains(selected_base) || selected_base.contains(p.plan.as_str()))
                                        .count()
                                } else {
                                    7
                                };
                                let wrap = agent_count.max(1);
                                state.selected_agent_tab = (state.selected_agent_tab + 1) % wrap;
                            } else if idx < 7 {
                                state.selected_agent_tab = idx;
                            }
                        }
                        TuiAction::SwitchDetailSubTab(idx) => {
                            state.detail_sub_tab = match idx {
                                0 => DetailSubTab::Agents,
                                1 => DetailSubTab::Output,
                                2 => DetailSubTab::Diff,
                                3 => DetailSubTab::Errors,
                                4 => DetailSubTab::Git,
                                _ => DetailSubTab::Agents,
                            };
                            if state.active_tab != 0 { state.active_tab = 0; }
                        }

                        // --- Focus cycling (Tab / Shift-Tab) ---
                        TuiAction::FocusNext => {
                            let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
                            state.focus = match state.focus {
                                FocusZone::Plans => FocusZone::Tasks,
                                FocusZone::Tasks => FocusZone::AgentOutput,
                                FocusZone::AgentOutput => {
                                    if has_cmd { FocusZone::CommandOutput } else { FocusZone::Plans }
                                }
                                FocusZone::CommandOutput => FocusZone::Plans,
                            };
                        }
                        TuiAction::FocusPrev => {
                            let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
                            state.focus = match state.focus {
                                FocusZone::Plans => {
                                    if has_cmd { FocusZone::CommandOutput } else { FocusZone::AgentOutput }
                                }
                                FocusZone::Tasks => FocusZone::Plans,
                                FocusZone::AgentOutput => FocusZone::Tasks,
                                FocusZone::CommandOutput => FocusZone::AgentOutput,
                            };
                        }
                        TuiAction::ScrollFocusedUp => {
                            state.task_scroll = state.task_scroll.saturating_sub(1);
                        }
                        TuiAction::ScrollFocusedDown => {
                            let max = state.task_checklist.as_ref().map(|c| c.tasks.len()).unwrap_or(0);
                            if state.task_scroll + 1 < max {
                                state.task_scroll += 1;
                            }
                        }

                        // --- Plan detail ---
                        TuiAction::ShowPlanDetail => {
                            if let Some(repo_root) = &state.repo_root {
                                if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                    let path = repo_root.join(format!("plans/{}.md", plan.base));
                                    state.plan_detail_content = std::fs::read_to_string(&path)
                                        .unwrap_or_else(|_| "Plan file not found.".to_string());
                                    state.plan_detail_scroll = 0;
                                    state.show_plan_detail = true;
                                    let is_completed = matches!(plan.status,
                                        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior);
                                    if is_completed {
                                        state.plan_summary_content = crate::orchestrator::context::read_summary(
                                            repo_root, &plan.num
                                        ).unwrap_or(None).unwrap_or_default();
                                        state.plan_summary_scroll = 0;
                                        state.plan_detail_tab = if state.plan_summary_content.is_empty() {
                                            PlanDetailTab::PlanDetails
                                        } else {
                                            PlanDetailTab::Summary
                                        };
                                    } else {
                                        state.plan_summary_content.clear();
                                        state.plan_detail_tab = PlanDetailTab::PlanDetails;
                                    }
                                }
                            }
                        }
                        TuiAction::ClosePlanDetail => {
                            state.show_plan_detail = false;
                            state.show_wave_overview = false;
                            state.show_agent_pool_modal = false;
                        }
                        TuiAction::ScrollDetailUp => {
                            let accel = state.scroll_accel.tick();
                            match state.plan_detail_tab {
                                PlanDetailTab::Summary => {
                                    state.plan_summary_scroll = state.plan_summary_scroll.saturating_sub(accel);
                                }
                                PlanDetailTab::PlanDetails => {
                                    state.plan_detail_scroll = state.plan_detail_scroll.saturating_sub(accel);
                                }
                            }
                        }
                        TuiAction::ScrollDetailDown => {
                            let accel = state.scroll_accel.tick();
                            match state.plan_detail_tab {
                                PlanDetailTab::Summary => { state.plan_summary_scroll += accel; }
                                PlanDetailTab::PlanDetails => { state.plan_detail_scroll += accel; }
                            }
                        }
                        TuiAction::ScrollDetailPageUp => {
                            let page = state.terminal_height.saturating_sub(6) as usize;
                            match state.plan_detail_tab {
                                PlanDetailTab::Summary => {
                                    state.plan_summary_scroll = state.plan_summary_scroll.saturating_sub(page);
                                }
                                PlanDetailTab::PlanDetails => {
                                    state.plan_detail_scroll = state.plan_detail_scroll.saturating_sub(page);
                                }
                            }
                        }
                        TuiAction::ScrollDetailPageDown => {
                            let page = state.terminal_height.saturating_sub(6) as usize;
                            match state.plan_detail_tab {
                                PlanDetailTab::Summary => { state.plan_summary_scroll += page; }
                                PlanDetailTab::PlanDetails => { state.plan_detail_scroll += page; }
                            }
                        }
                        TuiAction::SwitchDetailTab => {
                            state.plan_detail_tab = match state.plan_detail_tab {
                                PlanDetailTab::Summary => PlanDetailTab::PlanDetails,
                                PlanDetailTab::PlanDetails => {
                                    if state.plan_summary_content.is_empty() {
                                        PlanDetailTab::PlanDetails
                                    } else {
                                        PlanDetailTab::Summary
                                    }
                                }
                            };
                        }
                        TuiAction::DrillIn => {
                            if state.active_tab == 1 && !state.execution_waves.is_empty() {
                                if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                    let plan_base = plan.base.clone();
                                    for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                        if wave_plans.contains(&plan_base) {
                                            state.wave_expanded.insert(idx);
                                            break;
                                        }
                                    }
                                }
                                parallel_refresh_tasks(&mut state, &config);
                            }
                        }
                        TuiAction::DrillOut => {
                            if state.active_tab == 1 && !state.execution_waves.is_empty() {
                                if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                    let plan_base = plan.base.clone();
                                    for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                        if wave_plans.contains(&plan_base) {
                                            state.wave_expanded.remove(&idx);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // --- Agent output scroll ---
                        TuiAction::ScrollAgentUp => {
                            let total = current_agent_line_count(&state);
                            let page = 10;
                            state.agent_scroll = Some(match state.agent_scroll {
                                None => total.saturating_sub(page),
                                Some(n) => n.saturating_sub(page),
                            });
                        }
                        TuiAction::ScrollAgentDown => {
                            if let Some(n) = state.agent_scroll {
                                let total = current_agent_line_count(&state);
                                let new = n + 10;
                                if new >= total.saturating_sub(20) {
                                    state.agent_scroll = None;
                                } else {
                                    state.agent_scroll = Some(new);
                                }
                            }
                        }
                        TuiAction::ScrollAgentEnd => { state.agent_scroll = None; }

                        // --- Diff / command output scroll ---
                        TuiAction::ScrollDiffUp => {
                            if state.focus == FocusZone::CommandOutput {
                                let total = state.command_output.lines().count();
                                let page = 10;
                                state.command_output_scroll = Some(match state.command_output_scroll {
                                    None => total.saturating_sub(page),
                                    Some(n) => n.saturating_sub(page),
                                });
                            } else {
                                let total = state.branch_diff.lines().count();
                                let page = 10;
                                state.diff_scroll = Some(match state.diff_scroll {
                                    None => total.saturating_sub(page),
                                    Some(n) => n.saturating_sub(page),
                                });
                            }
                        }
                        TuiAction::ScrollDiffDown => {
                            if state.focus == FocusZone::CommandOutput {
                                if let Some(n) = state.command_output_scroll {
                                    let total = state.command_output.lines().count();
                                    let new = n + 10;
                                    if new >= total.saturating_sub(20) {
                                        state.command_output_scroll = None;
                                    } else {
                                        state.command_output_scroll = Some(new);
                                    }
                                }
                            } else if let Some(n) = state.diff_scroll {
                                let total = state.branch_diff.lines().count();
                                let new = n + 10;
                                if new >= total.saturating_sub(20) {
                                    state.diff_scroll = None;
                                } else {
                                    state.diff_scroll = Some(new);
                                }
                            }
                        }

                        // --- Modals ---
                        TuiAction::ShowHelp => { state.show_help = !state.show_help; }
                        TuiAction::ShowWaveOverview => {
                            state.show_wave_overview = !state.show_wave_overview;
                            state.show_agent_pool_modal = false;
                        }
                        TuiAction::ShowAgentPoolModal => {
                            state.show_agent_pool_modal = !state.show_agent_pool_modal;
                            state.show_wave_overview = false;
                        }
                        TuiAction::DismissNotification => { state.notifications.pop(); }
                        TuiAction::ShowTaskDetail => {
                            state.show_task_detail = true;
                            state.task_detail_scroll = 0;
                        }
                        TuiAction::CloseTaskDetail => { state.show_task_detail = false; }
                        TuiAction::ScrollTaskDetailUp => {
                            state.task_detail_scroll = state.task_detail_scroll.saturating_sub(1);
                        }
                        TuiAction::ScrollTaskDetailDown => { state.task_detail_scroll += 1; }

                        // --- Wave collapse/expand ---
                        TuiAction::CollapseExpand => {
                            if !state.execution_waves.is_empty() {
                                if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                    let plan_base = plan.base.clone();
                                    for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                        if wave_plans.contains(&plan_base) {
                                            if state.wave_expanded.contains(&idx) {
                                                state.wave_expanded.remove(&idx);
                                            } else {
                                                state.wave_expanded.insert(idx);
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // --- Filter ---
                        TuiAction::StartFilter => {
                            state.input_mode = InputMode::Filter;
                            state.filter_text.clear();
                            state.filter_active = true;
                        }
                        TuiAction::AcceptFilter => {
                            state.input_mode = InputMode::Normal;
                            state.filter_active = false;
                        }
                        TuiAction::CancelFilter => {
                            state.input_mode = InputMode::Normal;
                            state.filter_text.clear();
                            state.filter_active = false;
                        }
                        TuiAction::InputChar(c) => {
                            match state.input_mode {
                                InputMode::Inject => { state.message_input.push(c); }
                                InputMode::Filter => { state.filter_text.push(c); }
                                _ => {}
                            }
                        }
                        TuiAction::InputBackspace => {
                            match state.input_mode {
                                InputMode::Inject => { state.message_input.pop(); }
                                InputMode::Filter => { state.filter_text.pop(); }
                                _ => {}
                            }
                        }

                        // --- Config panel ---
                        TuiAction::ConfigUp => {
                            state.config.selected_row = state.config.selected_row.saturating_sub(1);
                        }
                        TuiAction::ConfigDown => {
                            let max = state.config.row_count().saturating_sub(1);
                            if state.config.selected_row < max {
                                state.config.selected_row += 1;
                            }
                        }
                        TuiAction::ConfigLeft => { handle_config_cycle(&mut state, false); }
                        TuiAction::ConfigRight => { handle_config_cycle(&mut state, true); }
                        TuiAction::ConfigSelect => {
                            handle_config_select(&mut state, &config);
                            // Hot reload: kill agents whose model changed since last Apply
                            for role in state.pending_agent_kills.drain(..).collect::<Vec<_>>() {
                                tracing::info!("Hot reload: killing {} (model changed to {})",
                                    role, state.config.model_for(role).unwrap_or("?"));
                                pool.kill_role(role).await;
                                state.add_log("config",
                                    &format!("Reloaded {}: model={}", role, state.config.model_for(role).unwrap_or("?")),
                                    LogLevel::Info);
                            }
                            // Sync fallback model to pool
                            pool.set_fallback_model(state.config.fallback_model.clone());
                        }

                        // --- Agent pane group / verify tabs ---
                        TuiAction::ToggleAgentPaneGroup => {
                            state.agent_pane_group = match state.agent_pane_group {
                                AgentPaneGroup::Implementation => AgentPaneGroup::Verification,
                                AgentPaneGroup::Verification => AgentPaneGroup::Implementation,
                            };
                        }
                        TuiAction::VerifyTabNext => {
                            if !state.verify_entries.is_empty() {
                                state.selected_verify_idx =
                                    (state.selected_verify_idx + 1) % state.verify_entries.len();
                            }
                        }
                        TuiAction::VerifyTabPrev => {
                            if !state.verify_entries.is_empty() {
                                state.selected_verify_idx = if state.selected_verify_idx == 0 {
                                    state.verify_entries.len() - 1
                                } else {
                                    state.selected_verify_idx - 1
                                };
                            }
                        }

                        // ExpandCollapse toggles task detail in the task pane
                        TuiAction::ExpandCollapse => {
                            if let Some(ref cl) = state.task_checklist {
                                if let Some(task) = cl.tasks.get(state.task_scroll) {
                                    let tid = task.id.clone();
                                    if state.task_expanded.as_deref() == Some(&tid) {
                                        state.task_expanded = None;
                                    } else {
                                        state.task_expanded = Some(tid);
                                    }
                                }
                            }
                        }

                        // --- Inject support in parallel mode ---
                        TuiAction::StartInject => {
                            state.input_mode = InputMode::Inject;
                            state.message_input.clear();
                            // Resolve selected_agent_tab agent for modal title
                            let selected_base = state.plans.get(state.selected_plan_idx)
                                .map(|p| p.base.clone()).unwrap_or_default();
                            let mut tab_agents: Vec<&crate::state::ParallelAgentState> = state.parallel_agents.iter()
                                .filter(|p| p.plan.contains(&selected_base) || selected_base.contains(p.plan.as_str()))
                                .collect();
                            tab_agents.sort_by_key(|p| (p.role != AgentRole::Implementer, p.task.clone()));
                            if let Some(pa) = tab_agents.get(state.selected_agent_tab) {
                                state.steer_target = Some(format!("{}:{}", pa.role, pa.instance_id));
                            } else {
                                state.steer_target = None;
                            }
                        }
                        TuiAction::SubmitInject(msg) => {
                            state.input_mode = InputMode::Normal;
                            state.message_input.clear();
                            if !msg.is_empty() {
                                let lower = msg.trim().to_lowercase();
                                if lower == "/reset" || lower == "/restart" {
                                    // Reset the selected plan (reuse ResetPlanState logic)
                                    if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                        let plan = entry.base.clone();
                                        state.add_log("inject", &format!("/reset → resetting plan {plan}"), LogLevel::Warn);
                                        pool.kill_plan_agents(&plan).await;
                                        executor.reset_plan(&plan);
                                        let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                        if wt_path.exists() {
                                            let pw = crate::git::worktree::PlanWorktree {
                                                path: wt_path,
                                                branch: format!("codex/plan/{plan}"),
                                                plan_base: plan.clone(),
                                            };
                                            let _ = worktree_mgr.cleanup_plan_worktree(&pw);
                                        }
                                        let _ = git_manager.delete_tag(&format!("plan/{plan}"));
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        state.plan_doc_revisions.remove(&plan);
                                        state.plan_agent_retries.retain(|k, _| !k.contains(&plan));
                                        state.parallel_agents.retain(|p| p.plan != plan);
                                        state.executor_completed_tasks.retain(|k| !k.starts_with(&format!("{plan}:")));
                                        state.plan_gate_outputs.remove(&plan);
                                        state.plan_phase_started.remove(&plan);
                                        state.plan_start_times.remove(&plan);
                                        state.task_started_at.retain(|k, _| !k.starts_with(&format!("{plan}:")));
                                        let plan_branch = format!("codex/plan/{plan}");
                                        let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                        if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                            entry.status = RunPlanStatus::Pending;
                                            entry.phase = String::new();
                                            entry.iteration = 0;
                                            entry.started_at = None;
                                        }
                                        let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                } else if lower == "/strategy" || lower == "/replan" {
                                    // Kill current agents for this plan, route to strategist
                                    if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                        let plan = entry.base.clone();
                                        state.add_log("inject", &format!("/strategy → re-running strategist for {plan}"), LogLevel::Warn);
                                        pool.kill_plan_agents(&plan).await;
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        state.parallel_agents.retain(|p| p.plan != plan);
                                        let actions = executor.handle_plan_revise(&plan);
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                } else if lower == "/review" {
                                    // Force review spawn for stuck plans (simpler & safer version)
                                    if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                        let plan = entry.base.clone();
                                        state.add_log("inject", &format!("/review → forcing review spawn for {plan}"), LogLevel::Warn);
                                        pool.kill_plan_agents(&plan).await;
                                        state.plan_review_stage.remove(&plan);
                                        state.plan_pending_reviews.remove(&plan);
                                        state.parallel_agents.retain(|p| p.plan != plan);
                                        let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                } else if msg.starts_with("/direct ") {
                                    // /direct bypass: send directly to selected agent
                                    let direct_msg = msg.strip_prefix("/direct ").unwrap_or(&msg);
                                    let selected_base = state.plans.get(state.selected_plan_idx)
                                        .map(|p| p.base.clone())
                                        .unwrap_or_default();
                                    let mut plan_agents: Vec<&crate::state::ParallelAgentState> = state.parallel_agents.iter()
                                        .filter(|p| p.plan.contains(&selected_base) || selected_base.contains(p.plan.as_str()))
                                        .collect();
                                    plan_agents.sort_by_key(|p| (
                                        p.role != AgentRole::Implementer,
                                        p.task.clone(),
                                    ));
                                    if let Some(pa) = plan_agents.get(state.selected_agent_tab) {
                                        let iid = pa.instance_id.clone();
                                        let role = pa.role;
                                        let aid = AgentInstanceId::new(role, iid.clone());
                                        if pool.is_spawned(&aid) {
                                            let _ = pool.turn_interrupt(&aid).await;
                                            let inject_msg = format!("Supervisor message: {direct_msg}\n\nContinue from where you left off.");
                                            if let Err(e) = pool.turn_start(&aid, &inject_msg, None).await {
                                                tracing::error!("Failed to start direct inject turn for {role}:{iid}: {e}");
                                                state.add_log("executor", &format!("Direct inject turn_start failed for {role}:{iid}: {e}"), LogLevel::Error);
                                            }
                                            if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                                                pa_mut.active = true;
                                            }
                                            state.add_log("inject", &format!("[direct:{role}:{iid}] {direct_msg}"), LogLevel::Info);
                                            let echo = format!("\n--- Direct inject ---\n{direct_msg}\n---------------------\n");
                                            if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                                                pa_mut.output.push_str(&echo);
                                            }
                                        } else {
                                            state.add_log("inject", &format!("[{role}:{iid}] Agent not spawned"), LogLevel::Warn);
                                        }
                                    } else {
                                        state.add_log("inject", "No agent selected", LogLevel::Warn);
                                    }
                                } else {
                                // Route through conductor — target is the agent at selected_agent_tab
                                let selected_base = state.plans.get(state.selected_plan_idx)
                                    .map(|p| p.base.clone()).unwrap_or_default();
                                let mut tab_agents: Vec<&crate::state::ParallelAgentState> = state.parallel_agents.iter()
                                    .filter(|p| p.plan.contains(&selected_base) || selected_base.contains(p.plan.as_str()))
                                    .collect();
                                tab_agents.sort_by_key(|p| (p.role != AgentRole::Implementer, p.task.clone()));
                                let target_info: Option<(AgentRole, String, String)> = tab_agents
                                    .get(state.selected_agent_tab)
                                    .map(|pa| (pa.role, pa.instance_id.clone(), pa.task.clone()));

                                // Send to conductor first (even if no agents spawned yet)
                                let conductor_iid = "conductor:llm".to_string();
                                let conductor_aid = AgentInstanceId::new(AgentRole::Conductor, conductor_iid.clone());
                                let conductor_active = state.parallel_agents.iter()
                                    .find(|p| p.instance_id == conductor_iid)
                                    .map(|p| p.active)
                                    .unwrap_or(false);

                                let target_desc = target_info.as_ref()
                                    .map(|(role, _iid, task)| format!("{role}:{task}"))
                                    .unwrap_or_else(|| "no agents for this plan".to_string());

                                // Re-spawn conductor if needed
                                if !pool.is_spawned(&conductor_aid) {
                                    let effort = state.config.effort_for(AgentRole::Conductor).label();
                                    if let Err(e) = pool.spawn_instance(conductor_aid.clone(), None, effort, state.config.model_for(AgentRole::Conductor)).await {
                                        tracing::error!("Failed to re-spawn conductor: {e}");
                                        state.add_log("executor", &format!("Conductor spawn_instance failed: {e}"), LogLevel::Error);
                                    }
                                    pool.set_thread_id(&conductor_aid, None);
                                }

                                if !conductor_active && pool.is_spawned(&conductor_aid) {
                                    // Route through conductor
                                    let conductor_msg = format!(
                                        "## User Inject\n\nUser message: {msg}\n\nCurrently selected agent: {target_desc}\n\nShould this message go to the selected agent, or should you issue a directive?\n\nRespond with:\n- [OK] to forward to {target_desc}\n- [NUDGE role:instance_id] message — to send to a specific agent\n- [RESTART role:instance_id] — to restart an agent\n- [RESET_REVIEW] — to reset the review cycle\n- Or any other directive from your protocol.",
                                    );
                                    match pool.turn_start(&conductor_aid, &conductor_msg, state.config.model_for(AgentRole::Conductor)).await {
                                        Ok(()) => {
                                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == conductor_iid) {
                                                pa.active = true;
                                                pa.finished_at = None;
                                            }
                                            // Store the pending inject so we can forward after conductor responds
                                            state.pending_inject = Some(crate::state::PendingInject {
                                                message: msg.clone(),
                                                target_role: target_info.as_ref().map(|(r, _, _)| *r),
                                                target_instance_id: target_info.as_ref().map(|(_, iid, _)| iid.clone()),
                                            });
                                            state.add_log("inject", &format!("[conductor] Supervisor message: {msg}"), LogLevel::Info);
                                            let echo = format!("\n--- Inject routed to conductor ---\n{msg}\n----------------------------------\n");
                                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == conductor_iid) {
                                                pa.output.push_str(&echo);
                                            }
                                        }
                                        Err(e) => {
                                            // Conductor failed — fall through to direct inject
                                            state.add_log("inject", &format!("Conductor routing failed ({e}), sending directly"), LogLevel::Warn);
                                            if let Some((role, iid, _task)) = target_info {
                                                let aid = AgentInstanceId::new(role, iid.clone());
                                                if pool.is_spawned(&aid) {
                                                    let _ = pool.turn_interrupt(&aid).await;
                                                    let inject_msg = format!("Supervisor message: {msg}\n\nContinue from where you left off.");
                                                    if let Err(e) = pool.turn_start(&aid, &inject_msg, None).await {
                                                        tracing::error!("Failed to start fallback inject turn for {role}:{iid}: {e}");
                                                        state.add_log("executor", &format!("Fallback inject turn_start failed for {role}:{iid}: {e}"), LogLevel::Error);
                                                    }
                                                    if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                                                        pa_mut.active = true;
                                                    }
                                                    state.add_log("inject", &format!("[{role}:{iid}] {msg}"), LogLevel::Info);
                                                }
                                            } else {
                                                state.add_log("inject", "No agents spawned for this plan yet", LogLevel::Warn);
                                                state.notifications.push(Notification {
                                                    message: "No agents to steer. Start the plan first.".to_string(),
                                                    created: std::time::Instant::now(),
                                                    ttl_secs: 5,
                                                    level: LogLevel::Warn,
                                                });
                                            }
                                        }
                                    }
                                } else {
                                    // Conductor busy or unavailable — send directly
                                    if let Some((role, iid, _task)) = target_info {
                                        let aid = AgentInstanceId::new(role, iid.clone());
                                        if pool.is_spawned(&aid) {
                                            let _ = pool.turn_interrupt(&aid).await;
                                            let inject_msg = format!("Supervisor message: {msg}\n\nContinue from where you left off.");
                                            if let Err(e) = pool.turn_start(&aid, &inject_msg, None).await {
                                                tracing::error!("Failed to start direct inject turn for {role}:{iid}: {e}");
                                                state.add_log("executor", &format!("Direct inject turn_start failed for {role}:{iid}: {e}"), LogLevel::Error);
                                            }
                                            if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                                                pa_mut.active = true;
                                            }
                                            state.add_log("inject", &format!("[{role}:{iid}] {msg} (conductor busy)"), LogLevel::Info);
                                            let echo = format!("\n--- Supervisor inject ---\n{msg}\n-------------------------\n");
                                            if let Some(pa_mut) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                                                pa_mut.output.push_str(&echo);
                                            }
                                        } else {
                                            state.add_log("inject", &format!("[{role}:{iid}] Agent not spawned — select an active agent"), LogLevel::Warn);
                                            state.notifications.push(Notification {
                                                message: format!("{} not active — select an active agent (Tab/`)", role.label()),
                                                created: std::time::Instant::now(),
                                                ttl_secs: 5,
                                                level: LogLevel::Warn,
                                            });
                                        }
                                    } else {
                                        state.add_log("inject", "No agent selected", LogLevel::Warn);
                                    }
                                }
                                }
                            }
                        }
                        TuiAction::CancelInject => {
                            state.input_mode = InputMode::Normal;
                            state.message_input.clear();
                        }

                        // --- Confirmation modal flow ---
                        TuiAction::RequestConfirm(confirm_action) => {
                            state.pending_confirm = Some(confirm_action);
                            state.input_mode = InputMode::Confirm;
                        }
                        TuiAction::ConfirmYes => {
                            if let Some(confirmed) = state.pending_confirm.take() {
                                state.input_mode = InputMode::Normal;
                                match confirmed {
                                    ConfirmAction::ResetSelectedPlan(_) => {
                                        // Reset selected plan and re-queue from scratch
                                        if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                            let plan = entry.base.clone();
                                            state.add_log("executor", &format!("Resetting plan {plan}"), LogLevel::Warn);
                                            pool.kill_plan_agents(&plan).await;
                                            executor.reset_plan(&plan);
                                            let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                            if wt_path.exists() {
                                                let pw = crate::git::worktree::PlanWorktree {
                                                    path: wt_path,
                                                    branch: format!("codex/plan/{plan}"),
                                                    plan_base: plan.clone(),
                                                };
                                                let _ = worktree_mgr.cleanup_plan_worktree(&pw);
                                            }
                                            let _ = git_manager.delete_tag(&format!("plan/{plan}"));
                                            state.plan_review_stage.remove(&plan);
                                            state.plan_pending_reviews.remove(&plan);
                                            state.plan_doc_revisions.remove(&plan);
                                            state.plan_agent_retries.retain(|k, _| !k.contains(&plan));
                                            state.parallel_agents.retain(|p| p.plan != plan);
                                            state.executor_completed_tasks.retain(|k| !k.starts_with(&format!("{plan}:")));
                                            state.plan_gate_outputs.remove(&plan);
                                            state.plan_phase_started.remove(&plan);
                                            state.plan_start_times.remove(&plan);
                                            state.task_started_at.retain(|k, _| !k.starts_with(&format!("{plan}:")));
                                            let plan_branch = format!("codex/plan/{plan}");
                                            let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                            if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                                entry.status = RunPlanStatus::Pending;
                                                entry.phase = String::new();
                                                entry.iteration = 0;
                                                entry.started_at = None;
                                            }
                                            state.add_log("executor", &format!("Plan {plan} fully reset — re-queuing"), LogLevel::Info);
                                            let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                    ConfirmAction::ReverifyPlan(_) => {
                                        if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                            let plan = entry.base.clone();
                                            state.add_log("executor", &format!("Re-verifying plan {plan} (gates + reviews only)"), LogLevel::Info);
                                            pool.kill_plan_agents(&plan).await;
                                            state.plan_review_stage.remove(&plan);
                                            state.plan_pending_reviews.remove(&plan);
                                            state.plan_gate_outputs.remove(&plan);
                                            state.plan_phase_started.remove(&plan);
                                            let actions = executor.reverify_plan(&plan);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                    ConfirmAction::RestartAllPlans => {
                                        state.add_log("executor", "Restarting ALL plans", LogLevel::Warn);
                                        let plan_bases: Vec<String> = state.plans.iter().map(|p| p.base.clone()).collect();
                                        for plan in &plan_bases {
                                            pool.kill_plan_agents(plan).await;
                                            executor.reset_plan(plan);
                                            let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                            if wt_path.exists() {
                                                let pw = crate::git::worktree::PlanWorktree {
                                                    path: wt_path,
                                                    branch: format!("codex/plan/{plan}"),
                                                    plan_base: plan.clone(),
                                                };
                                                let _ = worktree_mgr.cleanup_plan_worktree(&pw);
                                            }
                                            let _ = git_manager.delete_tag(&format!("plan/{plan}"));
                                            let plan_branch = format!("codex/plan/{plan}");
                                            let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                        }
                                        state.plan_review_stage.clear();
                                        state.plan_pending_reviews.clear();
                                        state.plan_doc_revisions.clear();
                                        state.plan_agent_retries.clear();
                                        state.parallel_agents.clear();
                                        state.executor_completed_tasks.clear();
                                        state.plan_gate_outputs.clear();
                                        state.plan_phase_started.clear();
                                        state.plan_start_times.clear();
                                        state.task_started_at.clear();
                                        for entry in &mut state.plans {
                                            entry.status = RunPlanStatus::Pending;
                                            entry.phase = String::new();
                                            entry.iteration = 0;
                                            entry.started_at = None;
                                        }
                                        let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                        execute_actions(
                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                            &spawn_ready_tx,
                                        ).await?;
                                    }
                                    ConfirmAction::RestartPhase => {
                                        state.add_log("executor", "Restart phase not available in parallel mode", LogLevel::Warn);
                                    }
                                    ConfirmAction::ForceAdvance(_) => {
                                        // Force advance not supported in parallel mode
                                        state.add_log("executor", "Force advance not available in parallel mode", LogLevel::Warn);
                                    }
                                    ConfirmAction::GitReconcile => {
                                        if state.git_reconcile_in_progress {
                                            state.add_log("git", "Reconcile already in progress", LogLevel::Warn);
                                        } else {
                                            state.git_reconcile_in_progress = true;
                                            state.add_log("git", "Starting git reconcile...", LogLevel::Info);
                                            let repo = config.repo_root.clone();
                                            let batch = batch_branch.to_string();
                                            let bid = config.batch_id.clone();
                                            let plan_info: Vec<(String, String)> = state.plans.iter()
                                                .map(|p| (p.base.clone(), p.num.clone()))
                                                .collect();
                                            let tx = gate_tx.clone();
                                            tokio::task::spawn_blocking(move || {
                                                let (messages, merged_plans, already_reconciled) = git_reconcile(&repo, &batch, &bid, &plan_info);
                                                let _ = tx.send(GateCompletion::ReconcileComplete { messages, merged_plans, already_reconciled });
                                            });
                                        }
                                    }
                                    ConfirmAction::IngestTask { plan_num, task_id } => {
                                        if let Some(repo_root) = &state.repo_root.clone() {
                                            let path = repo_root.join(format!("plans/context/tasks/{plan_num}-tasks.toml"));
                                            if let Ok(content) = std::fs::read_to_string(&path) {
                                                if let Ok(mut val) = toml::from_str::<toml::Value>(&content) {
                                                    if let Some(tasks) = val.get_mut("tasks").and_then(|v| v.as_array_mut()) {
                                                        for task in tasks.iter_mut() {
                                                            if task.get("id").and_then(|v| v.as_str()) == Some(task_id.as_str()) {
                                                                if let Some(tbl) = task.as_table_mut() {
                                                                    tbl.insert("status".to_string(),
                                                                        toml::Value::String("pending".to_string()));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    if let Ok(out) = toml::to_string(&val) {
                                                        let _ = std::fs::write(&path, out);
                                                    }
                                                }
                                            }
                                            state.add_log("orch",
                                                &format!("Ingested task {task_id} (plan {plan_num}) — reset to pending"),
                                                LogLevel::Info);
                                        }
                                    }
                                    ConfirmAction::MergeBatchToMain { batch_branch: ref bb, .. } => {
                                        let bb = bb.clone();
                                        state.add_log("git", &format!("Merging {bb} → main…"), LogLevel::Info);
                                        let gm_repo = config.repo_root.clone();
                                        let bb2 = bb.clone();
                                        let result = tokio::task::spawn_blocking(move || {
                                            let event_tx = tokio::sync::mpsc::unbounded_channel::<crate::git::GitEvent>().0;
                                            let gm = crate::git::GitManager::new(gm_repo, event_tx);
                                            gm.merge_batch_to_main(&bb2)
                                        }).await?;
                                        match result {
                                            Ok(hash) => {
                                                let now = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();
                                                let short_hash = hash[..hash.len().min(7)].to_string();
                                                let merged_bases: Vec<String> = state.plans.iter()
                                                    .filter(|p| matches!(p.status, RunPlanStatus::Completed | RunPlanStatus::CompletedPrior))
                                                    .map(|p| p.base.clone())
                                                    .collect();
                                                for plan in state.plans.iter_mut() {
                                                    if matches!(plan.status, RunPlanStatus::Completed | RunPlanStatus::CompletedPrior) {
                                                        plan.status = RunPlanStatus::MergedToMain;
                                                        plan.merged_to_main_at = Some(now);
                                                        plan.merge_commit = Some(short_hash.clone());
                                                    }
                                                }
                                                state.main_merges.push(crate::state::MainMergeRecord {
                                                    batch_branch: bb.clone(),
                                                    merge_commit: short_hash.clone(),
                                                    merged_at: now,
                                                    plan_bases: merged_bases,
                                                });
                                                state.add_log("git",
                                                    &format!("✓ Merged {bb} → main @ {short_hash}"),
                                                    LogLevel::Info);
                                                state.notifications.push(crate::state::Notification {
                                                    message: format!("⬆ main ← {bb} @ {short_hash}"),
                                                    created: std::time::Instant::now(),
                                                    ttl_secs: 10,
                                                    level: LogLevel::Info,
                                                });
                                            }
                                            Err(e) => {
                                                state.add_log("git", &format!("Merge failed: {e}"), LogLevel::Error);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        TuiAction::ConfirmNo => {
                            state.pending_confirm = None;
                            state.input_mode = InputMode::Normal;
                        }

                        // --- Pause/resume ---
                        TuiAction::TogglePause => {
                            state.pipeline_run_state = match state.pipeline_run_state {
                                PipelineRunState::Running => {
                                    state.add_log("executor", "Pipeline paused — no new work will be scheduled", LogLevel::Warn);
                                    PipelineRunState::Paused
                                }
                                PipelineRunState::Paused => {
                                    state.add_log("executor", "Pipeline resumed", LogLevel::Info);
                                    PipelineRunState::Running
                                }
                            };
                            // If resuming, schedule any pending work
                            if state.pipeline_run_state == PipelineRunState::Running {
                                let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            }
                        }

                        TuiAction::RestartPlan => {
                            let selected = state.plans.get(state.selected_plan_idx)
                                .map(|p| p.base.clone());
                            if let Some(plan) = selected {
                                state.add_log("executor", &format!("Restarting plan {plan}"), LogLevel::Warn);
                                executor.reset_plan(&plan);
                                // Kill all agents (implementers and reviewers) for this plan
                                pool.kill_plan_agents(&plan).await;
                                // Cleanup worktree
                                let wt = crate::git::worktree::PlanWorktree {
                                    path: worktree_mgr.worktree_base().join(format!("plan-{plan}")),
                                    branch: format!("codex/plan/{plan}"),
                                    plan_base: plan.clone(),
                                };
                                let _ = worktree_mgr.cleanup_plan_worktree(&wt);
                                // Reschedule
                                let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            }
                        }
                        TuiAction::ForceAdvance => {
                            let selected = state.plans.get(state.selected_plan_idx)
                                .map(|p| p.base.clone());
                            if let Some(plan) = selected {
                                state.add_log("executor", &format!("Force-advancing plan {plan}"), LogLevel::Warn);
                                if let Some(phase) = executor.plan_phase(&plan) {
                                    let actions = match phase {
                                        PlanPhase::Gating => {
                                            executor.handle_plan_gates_passed(&plan)
                                        }
                                        PlanPhase::Reviewing => {
                                            executor.handle_plan_reviews_passed(&plan)
                                        }
                                        _ => {
                                            state.add_log("executor", &format!("Cannot force-advance {plan} in phase {phase:?}"), LogLevel::Warn);
                                            vec![]
                                        }
                                    };
                                    execute_actions(
                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                        &spawn_ready_tx,
                                    ).await?;
                                }
                            }
                        }
                        TuiAction::ReverifyPlan => {
                            let selected = state.plans.get(state.selected_plan_idx)
                                .map(|p| p.base.clone());
                            if let Some(plan) = selected {
                                state.add_log("executor", &format!("Re-verifying plan {plan}"), LogLevel::Warn);
                                let actions = executor.reverify_plan(&plan);
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            }
                        }
                        TuiAction::ApproveCommand | TuiAction::ApproveAll | TuiAction::RejectCommand
                        | TuiAction::RestartPhase | TuiAction::ResetPlanState => {
                            state.add_log("tui", "Not available in parallel mode", LogLevel::Warn);
                        }
                        TuiAction::OpenTaskPicker => {
                            state.show_task_picker = true;
                            state.task_picker_cursor = 0;
                        }
                        TuiAction::CloseTaskPicker => {
                            state.show_task_picker = false;
                        }
                        TuiAction::TaskPickerUp => {
                            state.task_picker_cursor = state.task_picker_cursor.saturating_sub(1);
                        }
                        TuiAction::TaskPickerDown => {
                            state.task_picker_cursor = state.task_picker_cursor.saturating_add(1);
                        }
                        TuiAction::TaskPickerConfirm => {
                            state.show_task_picker = false;
                        }
                        TuiAction::PrepareMergeBatchToMain => {
                            let plan_count = state.plans.iter()
                                .filter(|p| matches!(p.status, RunPlanStatus::Completed | RunPlanStatus::CompletedPrior))
                                .count();
                            let failed_count = state.plans.iter()
                                .filter(|p| matches!(p.status, RunPlanStatus::Failed))
                                .count();
                            let last_commit = git_manager
                                .log_oneline(1)
                                .ok()
                                .and_then(|s| s.split_whitespace().next().map(|h| h[..h.len().min(7)].to_string()))
                                .unwrap_or_else(|| "unknown".to_string());
                            state.pending_confirm = Some(ConfirmAction::MergeBatchToMain {
                                batch_branch: batch_branch.to_string(),
                                plan_count,
                                failed_count,
                                last_commit,
                            });
                            state.input_mode = InputMode::Confirm;
                        }
                        TuiAction::None => {}
                    }
                    // Draw immediately after a keypress so state changes are visible
                    // without waiting for the next tick.
                    terminal.draw(|f| { tui::layout::render(f, &state, &atmosphere); })?;
                    }
                    Ok(_) => {} // Mouse, Resize, etc
                    Err(e) => {
                        warn!("Terminal event error: {e}");
                    }
                }
            }
            Some(spawn_ready) = spawn_ready_rx.recv() => {
                // Background agent spawn completed — insert connection and start the turn.
                // This arm fires instead of blocking execute_actions, so the tick arm
                // continues to render frames while agents cold-start.
                match spawn_ready {
                    AgentSpawnReady::Single { task_id, instance_id, prompt, result } => {
                        match result {
                            Ok((aid, conn, _wd)) => {
                                pool.insert_connection(aid.clone(), conn);
                                executor.record_task_started(task_id.clone(), instance_id.clone());
                                executor.record_spawn_success(&task_id.plan);
                                let model = state.config.model_for(AgentRole::Implementer).map(|s| s.to_string());
                                let model_label = model.as_deref().unwrap_or("default");
                                info!("Spawning agent {instance_id} for {task_id}");
                                state.add_log("executor", &format!("Task {task_id} started ({instance_id}) [model={model_label}]"), LogLevel::Info);
                                let task_key = format!("{}:{}", task_id.plan, task_id.task);
                                if !state.parallel_agents.iter().any(|p| p.instance_id == instance_id) {
                                    state.parallel_agents.push(crate::state::ParallelAgentState {
                                        instance_id: instance_id.clone(),
                                        role: aid.role,
                                        plan: task_id.plan.clone(),
                                        task: task_id.task.clone(),
                                        output: String::new(),
                                        input_tokens: 0,
                                        output_tokens: 0,
                                        cost_usd: 0.0,
                                        active: true,
                                        finished_at: None,
                                        model: model_label.to_string(),
                                        turn_started: false,
                                                        render_cache: Default::default(),
                                    });
                                } else if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == instance_id) {
                                    pa.active = true;
                                    pa.plan = task_id.plan.clone();
                                    pa.task = task_id.task.clone();
                                    pa.output.clear();
                                    pa.finished_at = None;
                                }
                                state.task_started_at.insert(task_key, std::time::Instant::now());
                                let (inp_tok, out_tok) = (
                                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                                );
                                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                                let event = crate::state::persistence::PersistenceManager::make_task_event(
                                    "task_start", &task_id.plan, Some(&task_id.task), Some(&instance_id), None,
                                );
                                let _ = persistence.append_task_event(&event);
                                pool.set_thread_id(&aid, None);
                                state.turn_started_at.insert(instance_id.clone(), std::time::Instant::now());
                                if let Err(e) = pool.turn_start(&aid, &prompt, model.as_deref()).await {
                                    warn!("Failed to start turn for {}: {e}", aid);
                                    state.add_log("executor", &format!(
                                        "Turn start failed for {} — will retry: {e}", task_id
                                    ), LogLevel::Warn);
                                    let actions = executor.handle_task_failed(task_id.clone());
                                    execute_actions(
                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                        &spawn_ready_tx,
                                    ).await?;
                                } else {
                                    // Mark turn_started so TUI shows "processing..." instead of "waiting..."
                                    if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == instance_id) {
                                        pa.turn_started = true;
                                    }
                                }
                            }
                            Err(e) => {
                                executor.record_spawn_failure(&task_id.plan);
                                warn!("Failed to spawn agent for plan {}: {e}", task_id.plan);
                                state.add_log("executor", &format!("Spawn failed: {e}"), LogLevel::Error);
                                let actions = executor.handle_task_failed(task_id.clone());
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            }
                        }
                    }
                    AgentSpawnReady::Batch { task_ids, plan_base, instance_id, prompt, result } => {
                        match result {
                            Ok((aid, conn, _wd)) => {
                                pool.insert_connection(aid.clone(), conn);
                                for task_id in &task_ids {
                                    executor.record_task_started(task_id.clone(), instance_id.clone());
                                }
                                executor.record_spawn_success(&plan_base);
                                let model = state.config.model_for(AgentRole::Implementer).map(|s| s.to_string());
                                let model_label = model.as_deref().unwrap_or("default");
                                let task_label = if task_ids.len() == 1 {
                                    task_ids[0].task.clone()
                                } else {
                                    format!("[batch:{}]", task_ids.len())
                                };
                                info!("Starting batch turn for {instance_id} ({} tasks)", task_ids.len());
                                state.add_log("executor", &format!(
                                    "Batch {instance_id} started ({} tasks) [model={model_label}]", task_ids.len()
                                ), LogLevel::Info);
                                if !state.parallel_agents.iter().any(|p| p.instance_id == instance_id) {
                                    state.parallel_agents.push(crate::state::ParallelAgentState {
                                        instance_id: instance_id.clone(),
                                        role: aid.role,
                                        plan: plan_base.clone(),
                                        task: task_label,
                                        output: String::new(),
                                        input_tokens: 0,
                                        output_tokens: 0,
                                        cost_usd: 0.0,
                                        active: true,
                                        finished_at: None,
                                        model: model_label.to_string(),
                                        turn_started: false,
                                                        render_cache: Default::default(),
                                    });
                                } else if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == instance_id) {
                                    pa.active = true;
                                    pa.plan = plan_base.clone();
                                    pa.output.clear();
                                    pa.finished_at = None;
                                }
                                for task_id in &task_ids {
                                    let task_key = format!("{}:{}", task_id.plan, task_id.task);
                                    state.task_started_at.insert(task_key, std::time::Instant::now());
                                    let event = crate::state::persistence::PersistenceManager::make_task_event(
                                        "task_start", &task_id.plan, Some(&task_id.task), Some(&instance_id), None,
                                    );
                                    let _ = persistence.append_task_event(&event);
                                }
                                let (inp_tok, out_tok) = (
                                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                                );
                                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);
                                pool.set_thread_id(&aid, None);
                                state.turn_started_at.insert(instance_id.clone(), std::time::Instant::now());
                                if let Err(e) = pool.turn_start(&aid, &prompt, model.as_deref()).await {
                                    warn!("Failed to start batch turn for {instance_id}: {e}");
                                    state.add_log("executor", &format!(
                                        "Batch turn start failed for {plan_base} — will retry: {e}"
                                    ), LogLevel::Warn);
                                    let actions = executor.handle_instance_failed(&instance_id);
                                    execute_actions(
                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                        &spawn_ready_tx,
                                    ).await?;
                                }
                            }
                            Err(e) => {
                                executor.record_spawn_failure(&plan_base);
                                warn!("Failed to spawn batch agent for plan {}: {e}", plan_base);
                                state.add_log("executor", &format!("Batch spawn failed for {plan_base}: {e}"), LogLevel::Error);
                                let actions = executor.handle_instance_failed(&instance_id);
                                execute_actions(
                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                    &spawn_ready_tx,
                                ).await?;
                            }
                        }
                    }
                }
            }
            _ = checkpoint.tick() => {
                // Periodic persistence (safety net — primary writes happen on task completion)
                let (inp_tok, out_tok) = (
                    state.parallel_agents.iter().map(|p| p.input_tokens).sum::<u64>(),
                    state.parallel_agents.iter().map(|p| p.output_tokens).sum::<u64>(),
                );

                // Run periodic integrity check and update diagnostics for conductor
                let issues = executor.integrity_check();
                state.executor_state_summary = executor.state_summary();
                if !issues.is_empty() {
                    state.add_log("executor", &format!("Integrity check: {} auto-fixes applied", issues.len()), LogLevel::Warn);
                }

                write_checkpoint(&executor, &persistence, &worktree_mgr, &batch_branch, inp_tok, out_tok);

                // Hard timeout: kill agents stuck longer than 30 minutes
                let timeout_limit = std::time::Duration::from_secs(30 * 60);
                let timed_out: Vec<(String, std::time::Duration)> = state.turn_started_at.iter()
                    .filter(|(_, started)| started.elapsed() > timeout_limit)
                    .map(|(iid, started)| (iid.clone(), started.elapsed()))
                    .collect();
                for (iid, duration) in timed_out {
                    let mins = duration.as_secs() / 60;
                    state.add_log("executor", &format!(
                        "Agent {} timed out after {}min — killing", iid, mins
                    ), LogLevel::Error);
                    warn!("Hard timeout: agent {} after {}min", iid, mins);
                    state.turn_started_at.remove(&iid);
                    // Find and kill the agent
                    if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == iid) {
                        pa.active = false;
                        pa.finished_at = Some(std::time::Instant::now());
                        let role = pa.role;
                        let aid = AgentInstanceId::new(role, iid.clone());
                        pool.kill_instance(&aid).await;
                    }
                    // Treat as task failure (batch-aware).
                    if !executor.tasks_for_instance(&iid).is_empty() {
                        let actions = executor.handle_instance_failed(&iid);
                        execute_actions(
                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                            &spawn_ready_tx,
                        ).await?;
                    }
                }

                // Update progress in state
                let progress = executor.progress();
                state.orchestrator_state = format!(
                    "parallel {}/{} plans, {}/{} tasks, {} in-flight",
                    progress.completed_plans, progress.total_plans,
                    progress.completed_tasks, progress.total_tasks,
                    progress.in_flight_tasks,
                );
            }
            _ = task_refresh.tick() => {
                parallel_refresh_tasks(&mut state, &config);
                // Re-schedule runnable tasks (recovers from spawn failures without recursion).
                let sched_actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                if !sched_actions.is_empty() {
                    execute_actions(
                        sched_actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                        &spawn_ready_tx,
                    ).await?;
                }
                // Remove finished agents older than 60s
                let cutoff = std::time::Duration::from_secs(60);
                state.parallel_agents.retain(|pa| {
                    pa.active || pa.finished_at.map(|t| t.elapsed() < cutoff).unwrap_or(true)
                });
                // Lightweight worktree reconciliation every ~60s (12 ticks * 5s)
                reconciliation_counter += 1;
                if reconciliation_counter >= 12 {
                    reconciliation_counter = 0;
                    let missing_wt_plans: Vec<String> = executor.active_plans().iter()
                        .filter(|plan| {
                            let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                            !wt_path.exists()
                        })
                        .map(|p| p.to_string())
                        .collect();
                    for plan in &missing_wt_plans {
                        warn!("Worktree disappeared for active plan {plan} — resetting");
                        state.add_log("executor", &format!(
                            "Worktree missing for {plan} — resetting plan"
                        ), LogLevel::Error);
                        let killed = executor.reset_plan(plan);
                        for gid in &killed {
                            let iid = format!("implementer:{}:{}", gid.plan, gid.task);
                            let aid = AgentInstanceId::new(AgentRole::Implementer, iid);
                            pool.kill_instance(&aid).await;
                        }
                    }
                    // Reschedule after any resets
                    let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                    if !actions.is_empty() {
                        execute_actions(
                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                            &spawn_ready_tx,
                        ).await?;
                    }
                }
                // Refresh git data in background (skip during merge to avoid lock contention)
                if !executor.merge_in_progress {
                    let repo = config.repo_root.clone();
                    let plans_snapshot = state.plans.clone();
                    if let Ok(result) = tokio::task::spawn_blocking(move || {
                        let tree = crate::git::graph::build_branch_tree(&repo, &plans_snapshot);
                        let graph = crate::git::graph::log_graph(&repo, 40).unwrap_or_default();
                        let worktrees = crate::git::worktree::list_worktrees(&repo).unwrap_or_default();

                        // Collect per-worktree git stats: (plan_base -> (branch_short, last_commit_secs, added, removed))
                        let worktree_base = repo.join(".worktrees");
                        let mut plan_git_stats: std::collections::HashMap<String, (String, u64, u32, u32)> =
                            std::collections::HashMap::new();
                        for wt in &worktrees {
                            let path = std::path::Path::new(&wt.path);
                            if !path.starts_with(&worktree_base) {
                                continue;
                            }
                            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            let plan_base = match dir_name.strip_prefix("plan-") {
                                Some(b) if !b.contains("-T") => b,
                                _ => continue,
                            };
                            let branch_short = wt.branch.clone();
                            let last_commit_secs = crate::git::ops::run_git(path, &["log", "-1", "--format=%ct"])
                                .ok()
                                .and_then(|s| s.trim().parse::<u64>().ok())
                                .unwrap_or(0);
                            let diff_stat = crate::git::ops::run_git(path, &["diff", "--stat", "HEAD"])
                                .unwrap_or_default();
                            let (added, removed) = parse_diff_stat_summary(&diff_stat);
                            plan_git_stats.insert(plan_base.to_string(), (branch_short, last_commit_secs, added, removed));
                        }

                        // Main repo last commit time
                        let main_commit_secs = crate::git::ops::run_git(&repo, &["log", "-1", "--format=%ct"])
                            .ok()
                            .and_then(|s| s.trim().parse::<u64>().ok());

                        (tree, graph, worktrees, plan_git_stats, main_commit_secs)
                    }).await {
                        state.git_branch_tree = result.0;
                        state.git_commit_graph = result.1;
                        state.git_worktree_list = result.2;
                        // Update per-plan git info
                        for plan in &mut state.plans {
                            if let Some((branch_short, last_commit_secs, added, removed)) =
                                result.3.get(&plan.base)
                            {
                                plan.git_branch_short = Some(branch_short.clone());
                                plan.git_last_commit_secs = Some(*last_commit_secs);
                                plan.git_dirty = if *added == 0 && *removed == 0 {
                                    None
                                } else {
                                    Some((*added, *removed))
                                };
                            }
                        }
                        state.git_last_commit_secs = result.4;
                    }
                }
            }
            _ = tick.tick() => {
                atmosphere.tick_with_degraded(state.any_agent_active());
                sys_collector.poll(&mut state.sys);
                if let Some((x, y)) = state.particle_burst_pending.take() {
                    atmosphere.spawn_burst(x, y, 8);
                }
                // Update global crash state every ~2s
                if last_crash_snapshot.elapsed() >= Duration::from_secs(2) {
                    crate::update_crash_state(state.snapshot_for_crash());
                    last_crash_snapshot = Instant::now();
                }
                // Clamp agent tab to actual plan agent count
                if !state.parallel_agents.is_empty() {
                    let selected_base = state.plans.get(state.selected_plan_idx)
                        .map(|p| p.base.clone())
                        .unwrap_or_default();
                    let plan_agent_count = state.parallel_agents.iter()
                        .filter(|p| p.plan.contains(&selected_base) || selected_base.contains(p.plan.as_str()))
                        .count();
                    if plan_agent_count > 0 && state.selected_agent_tab >= plan_agent_count {
                        state.selected_agent_tab = plan_agent_count.saturating_sub(1);
                    }
                }
                // Sample token burn history for sparklines
                {
                    let samples: Vec<_> = state.agents.iter().map(|(&r, a)| (r, a.input_tokens)).collect();
                    for (role, tokens) in samples {
                        let history = state.token_burn_history.entry(role).or_default();
                        history.push_back(tokens);
                        if history.len() > 120 {
                            history.pop_front();
                        }
                    }
                }
                // Adaptive frame rate: 30 FPS during user interaction, ~10 FPS when agents are working.
                // Skip 2 of every 3 frames when agents are active and no recent user input.
                frame_skip_counter = frame_skip_counter.wrapping_add(1);
                let user_idle = last_user_input.elapsed() > Duration::from_secs(2);
                let should_draw = if state.any_agent_active() && user_idle {
                    frame_skip_counter % 3 == 0
                } else {
                    true
                };
                if should_draw {
                    state.terminal_height = terminal.size()?.height;
                    terminal.draw(|f| {
                        tui::layout::render(f, &state, &atmosphere);
                    })?;
                }
                state.notifications.retain(|n| n.created.elapsed().as_secs() < n.ttl_secs);

                // Conductor tick — detect stalls, ghost turns, context pressure
                // Run per-plan so each plan gets its own phase timeout / review loop detection.
                {
                    let plan_bases: Vec<String> = state.plans.iter()
                        .filter(|p| matches!(p.status, RunPlanStatus::Active))
                        .map(|p| p.base.clone())
                        .collect();

                    let mut all_interventions = Vec::new();
                    let mut should_consult_llm = false;

                    for plan_base in &plan_bases {
                        let plan_agents: Vec<&crate::state::ParallelAgentState> = state.parallel_agents.iter()
                            .filter(|p| p.plan == *plan_base && p.active)
                            .collect();
                        let active_role = plan_agents.first().map(|p| p.role);
                        let max_tokens = plan_agents.iter().map(|p| p.input_tokens).max().unwrap_or(0);
                        // Per-plan doc revision count as proxy for consecutive revise
                        let doc_revs = state.plan_doc_revisions.get(plan_base).copied().unwrap_or(0);
                        let plan_phase = state.plan_review_stage.get(plan_base);
                        let is_reviewing = plan_phase.is_some();

                        // Build task summary for the task-continuation watcher.
                        let task_summary = {
                            let progress = executor.task_progress_for_plan(plan_base);
                            progress.map(|(completed, in_flight, total)| crate::conductor::TaskSummary {
                                plan: plan_base.clone(),
                                queued: total.saturating_sub(completed + in_flight) as u32,
                                in_flight: in_flight as u32,
                                completed: completed as u32,
                            })
                        };
                        let active_instance_id = state.parallel_agents.iter()
                            .find(|p| p.plan == *plan_base && p.role == AgentRole::Implementer && p.active)
                            .map(|p| p.instance_id.clone());

                        let ctx = crate::conductor::ConductorContext {
                            active_role,
                            last_message_at,
                            phase_started: state.plan_phase_started.get(plan_base).copied(),
                            compile_fail_count: 0,
                            last_compile_error: String::new(),
                            task_last_change: last_message_at,
                            input_tokens: max_tokens,
                            context_limit: state.context_limit,
                            iteration: executor.plan_iteration(plan_base),
                            consecutive_revise_count: doc_revs,
                            orchestrator_state: if is_reviewing { "reviewing".to_string() } else { "implementing".to_string() },
                            last_turn_had_output,
                            last_turn_duration_secs,
                            last_turn_has_git_changes: false,
                            plan: plan_base.clone(),
                            agent_backend: None,
                            task_summary,
                            active_instance_id,
                            test_pass_count: 0,
                            test_fail_count: 0,
                            compile_gates_passed: false,
                        };

                        // Check if any watcher fires — triggers LLM consultation
                        let watcher_interventions = conductor.tick(&ctx);
                        if !watcher_interventions.is_empty() {
                            should_consult_llm = true;
                            for i in watcher_interventions {
                                all_interventions.push((plan_base.clone(), i));
                            }
                        }
                    }

                    // Periodically consult the conductor LLM at phase transitions and stalls
                    // If watchers fired OR if it's been 60+ seconds since last consultation
                    let conductor_iid = "conductor:llm".to_string();
                    let conductor_idle = state.parallel_agents.iter()
                        .find(|p| p.instance_id == conductor_iid)
                        .map(|p| !p.active)
                        .unwrap_or(true);

                    let last_conductor_consult = state.last_periodic_conductor_consult.unwrap_or_else(|| Instant::now());
                    let time_since_consult = last_conductor_consult.elapsed();

                    if conductor_idle && (should_consult_llm || time_since_consult >= Duration::from_secs(60)) {
                        // Conductor is idle and either watchers fired or it's been 60s — send state snapshot
                        let conductor_aid = AgentInstanceId::new(AgentRole::Conductor, conductor_iid.clone());
                        if pool.is_spawned(&conductor_aid) {
                            let snapshot = crate::conductor::llm::state_snapshot(&state, "Periodic checkpoint");
                            if let Err(e) = pool.turn_start(&conductor_aid, &snapshot, state.config.model_for(AgentRole::Conductor)).await {
                                state.add_log("conductor", &format!("Failed to start periodic consultation: {e}"), LogLevel::Warn);
                            } else {
                                state.last_periodic_conductor_consult = Some(Instant::now());
                                if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == conductor_iid) {
                                    pa.active = true;
                                    pa.finished_at = None;
                                }
                                state.add_log("conductor", "Consulting LLM for guidance", LogLevel::Info);
                            }
                        }
                    }

                    for (plan_base, intervention) in all_interventions {
                        state.conductor_history.push(crate::state::ConductorHistoryEntry {
                            timestamp: intervention.timestamp.format("%H:%M:%S").to_string(),
                            watcher: intervention.watcher.clone(),
                            target: format!("{}:{}", plan_base, intervention.target_role),
                            message: intervention.message.clone(),
                        });
                        state.add_log("conductor", &format!(
                            "[{}] {} → {}:{}: {}",
                            intervention.tier_label(),
                            intervention.watcher,
                            plan_base,
                            intervention.target_role,
                            intervention.message,
                        ), LogLevel::Warn);

                        // Record conductor action in the agent list so it's visible
                        if intervention.action.is_some() {
                            let conductor_iid = format!("conductor:{plan_base}");
                            let action_desc = format!(
                                "[{}] {} → {}: {}\n",
                                intervention.tier_label(),
                                intervention.watcher,
                                intervention.target_role,
                                intervention.message,
                            );
                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == conductor_iid) {
                                pa.output.push_str(&action_desc);
                                pa.active = true;
                                pa.finished_at = None;
                            } else {
                                state.parallel_agents.push(crate::state::ParallelAgentState {
                                    instance_id: conductor_iid,
                                    role: AgentRole::Conductor,
                                    plan: plan_base.clone(),
                                    task: "conductor".to_string(),
                                    output: action_desc,
                                    input_tokens: 0,
                                    output_tokens: 0,
                                    cost_usd: 0.0,
                                    active: true,
                                    finished_at: None,
                                    model: String::new(),
                                    turn_started: false,
                                                    render_cache: Default::default(),
                                });
                            }
                        }

                        // Execute conductor actions
                        let intervention_watcher = intervention.watcher.clone();
                        if let Some(action) = intervention.action {
                            match action {
                                ConductorAction::SkipReviews => {
                                    // Kill active review agents for this plan and force merge
                                    state.add_log("conductor", &format!(
                                        "Skipping reviews for {plan_base} — loop detected"
                                    ), LogLevel::Warn);
                                    // Kill active review agents for this plan
                                    let review_iids: Vec<(AgentRole, String)> = state.parallel_agents.iter()
                                        .filter(|p| p.plan == plan_base && p.active && matches!(
                                            p.role,
                                            AgentRole::Architect | AgentRole::Auditor | AgentRole::Scribe | AgentRole::Critic
                                        ))
                                        .map(|p| (p.role, p.instance_id.clone()))
                                        .collect();
                                    for (role, iid) in &review_iids {
                                        let aid = AgentInstanceId::new(*role, iid.clone());
                                        pool.kill_instance(&aid).await;
                                        if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                            pa.active = false;
                                            pa.finished_at = Some(std::time::Instant::now());
                                        }
                                    }
                                    state.plan_review_stage.remove(&plan_base);
                                    state.plan_pending_reviews.remove(&plan_base);
                                    let actions = executor.handle_plan_reviews_passed(&plan_base);
                                    execute_actions(
                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                        &spawn_ready_tx,
                                    ).await?;
                                }
                                ConductorAction::RestartAgent { role } => {
                                    // Kill and respawn the target agent for this plan
                                    let target_iids: Vec<String> = state.parallel_agents.iter()
                                        .filter(|p| p.plan == plan_base && p.role == role && p.active)
                                        .map(|p| p.instance_id.clone())
                                        .collect();
                                    for iid in &target_iids {
                                        let aid = AgentInstanceId::new(role, iid.clone());
                                        pool.kill_instance(&aid).await;
                                        if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                            pa.active = false;
                                            pa.finished_at = Some(std::time::Instant::now());
                                        }
                                    }
                                    state.add_log("conductor", &format!(
                                        "Restarted {role} for {plan_base}"
                                    ), LogLevel::Warn);
                                    // Killing a review agent in-flight: treat as if it finished
                                    // with APPROVE so the pipeline can advance
                                    if matches!(role, AgentRole::Critic | AgentRole::Scribe | AgentRole::Architect | AgentRole::Auditor) {
                                        // Check if this unblocks review pipeline
                                        if let Some(pending) = state.plan_pending_reviews.get_mut(&plan_base) {
                                            pending.remove(&role);
                                            if pending.is_empty() {
                                                state.plan_pending_reviews.remove(&plan_base);
                                                // Advance: if we were waiting on this role, skip to merge
                                                state.plan_review_stage.remove(&plan_base);
                                                let actions = executor.handle_plan_reviews_passed(&plan_base);
                                                execute_actions(
                                                    actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                    &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                    &spawn_ready_tx,
                                                ).await?;
                                            }
                                        } else {
                                            // No pending set means this was a solo agent (critic).
                                            // Skip to merge.
                                            state.plan_review_stage.remove(&plan_base);
                                            let actions = executor.handle_plan_reviews_passed(&plan_base);
                                            execute_actions(
                                                actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                &spawn_ready_tx,
                                            ).await?;
                                        }
                                    }
                                }
                                ConductorAction::SendMessage { role, message } => {
                                    // Inject a message into the target agent for this plan
                                    let target_pa = state.parallel_agents.iter()
                                        .find(|p| p.plan == plan_base && p.role == role && p.active)
                                        .map(|p| p.instance_id.clone());
                                    if let Some(iid) = target_pa {
                                        let aid = AgentInstanceId::new(role, iid.clone());
                                        if pool.is_spawned(&aid) {
                                            let _ = pool.turn_interrupt(&aid).await;
                                            if let Err(e) = pool.turn_start(&aid, &message, None).await {
                                                tracing::error!("Failed to start conductor SendMessage turn for {role}:{iid}: {e}");
                                                state.add_log("executor", &format!("Conductor SendMessage turn_start failed for {role}:{iid}: {e}"), LogLevel::Error);
                                            }
                                        }
                                    }
                                }
                                ConductorAction::AssignAdditionalTasks { instance_id, task_descriptions } => {
                                    // Inject additional tasks into a warm implementer rather than cold-starting.
                                    let message = format!(
                                        "Your current tasks are done. Here are additional tasks to implement now:\n\n{}",
                                        task_descriptions.join("\n\n")
                                    );
                                    let aid = AgentInstanceId::new(AgentRole::Implementer, instance_id.clone());
                                    if pool.is_spawned(&aid) {
                                        let _ = pool.turn_interrupt(&aid).await;
                                        if let Err(e) = pool.turn_start(&aid, &message, None).await {
                                            tracing::error!("Failed to assign tasks to {instance_id}: {e}");
                                            state.add_log("executor", &format!(
                                                "AssignAdditionalTasks failed for {instance_id}: {e}"
                                            ), LogLevel::Error);
                                        } else {
                                            state.add_log("conductor", &format!(
                                                "Assigned {} additional task(s) to warm agent {instance_id}",
                                                task_descriptions.len()
                                            ), LogLevel::Info);
                                        }
                                    }
                                }
                                ConductorAction::ForceAdvance => {
                                    // Log any outstanding test failures as deferred
                                    if let Some(gate_output) = state.plan_gate_outputs.get(&plan_base) {
                                        let failing = crate::orchestrator::gates::extract_failing_test_names(gate_output);
                                        let snippet = crate::orchestrator::gates::extract_test_failure_snippet(gate_output, 50);
                                        if !failing.is_empty() {
                                            let watcher_name = intervention_watcher.as_str();
                                            let reason = if watcher_name == "TestFailureBudget" {
                                                let counts = crate::orchestrator::gates::parse_test_counts_pub(gate_output);
                                                let total = counts.0 + counts.1;
                                                crate::state::persistence::DeferredReason::BudgetAllowed {
                                                    pass_rate: if total > 0 { counts.0 as f64 / total as f64 } else { 0.0 },
                                                    threshold: 0.7,
                                                }
                                            } else {
                                                crate::state::persistence::DeferredReason::ForceAdvanced
                                            };
                                            let failures: Vec<_> = failing.iter().map(|name| {
                                                crate::state::persistence::DeferredFailure {
                                                    plan: plan_base.clone(),
                                                    task_id: String::new(),
                                                    title: format!("Test failure: {}", name),
                                                    task_type: "test_gate".to_string(),
                                                    command: "cargo test".to_string(),
                                                    test_fns: vec![name.clone()],
                                                    reason: reason.clone(),
                                                    error_snippet: snippet.clone(),
                                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                                    iteration: executor.plan_iteration(&plan_base),
                                                }
                                            }).collect();
                                            let batch_id = batch_branch.strip_prefix("codex/batch/").unwrap_or(&batch_branch);
                                            let _ = persistence.append_deferred_failures(batch_id, failures);
                                            state.add_log("conductor", &format!(
                                                "Logged {} deferred test failure(s) for {plan_base}",
                                                failing.len()
                                            ), LogLevel::Info);
                                        }
                                    }

                                    // Force the plan past reviews and merge
                                    state.add_log("conductor", &format!(
                                        "Force advancing {plan_base}"
                                    ), LogLevel::Warn);
                                    // Kill all active agents for this plan
                                    let plan_iids: Vec<(AgentRole, String)> = state.parallel_agents.iter()
                                        .filter(|p| p.plan == plan_base && p.active)
                                        .map(|p| (p.role, p.instance_id.clone()))
                                        .collect();
                                    for (role, iid) in &plan_iids {
                                        let aid = AgentInstanceId::new(*role, iid.clone());
                                        pool.kill_instance(&aid).await;
                                        if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == *iid) {
                                            pa.active = false;
                                            pa.finished_at = Some(std::time::Instant::now());
                                        }
                                    }
                                    state.plan_review_stage.remove(&plan_base);
                                    state.plan_pending_reviews.remove(&plan_base);
                                    let actions = executor.handle_plan_reviews_passed(&plan_base);
                                    execute_actions(
                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                        &spawn_ready_tx,
                                    ).await?;
                                }
                                ConductorAction::PingWarmAgent { instance_id } => {
                                    // Send a lightweight keepalive signal to a warm agent (no prompt, just ACK).
                                    // This is used to prevent idle-timeout on warm agents waiting for turn_start.
                                    // For now, log it. In the future, could emit a special ACK message to the agent.
                                    state.add_log("conductor", &format!(
                                        "Pinging warm agent {instance_id}"
                                    ), LogLevel::Debug);
                                    // TODO: implement lightweight agent keepalive if needed
                                }
                                ConductorAction::SpawnValidation { .. } => {
                                    state.add_log("conductor", "Action SpawnValidation not yet implemented in parallel mode", LogLevel::Warn);
                                }
                                ConductorAction::GenerateFixPlan { .. } => {
                                    state.add_log("conductor", "Action GenerateFixPlan not yet implemented in parallel mode", LogLevel::Warn);
                                }
                                ConductorAction::InsertGate { .. } => {
                                    state.add_log("conductor", "Action InsertGate not yet implemented in parallel mode", LogLevel::Warn);
                                }
                                ConductorAction::SkipValidation { .. } => {
                                    state.add_log("conductor", "Action SkipValidation not yet implemented in parallel mode", LogLevel::Warn);
                                }
                            }
                            // Mark conductor agent as done after action
                            let conductor_iid = format!("conductor:{plan_base}");
                            if let Some(pa) = state.parallel_agents.iter_mut().find(|p| p.instance_id == conductor_iid) {
                                pa.active = false;
                                pa.finished_at = Some(std::time::Instant::now());
                            }
                        }
                    }
                }

                // Periodic conductor consultation during implementation (every 120s)
                {
                    let should_consult = state.last_periodic_conductor_consult
                        .map(|t| t.elapsed() >= Duration::from_secs(120))
                        .unwrap_or(true);
                    let has_active_impl = state.parallel_agents.iter()
                        .any(|p| p.role == AgentRole::Implementer && p.active);
                    if should_consult && has_active_impl {
                        let active_tasks: Vec<String> = state.parallel_agents.iter()
                            .filter(|p| p.active)
                            .map(|p| format!("  - {}:{} [{}] tokens={}", p.role, p.task, if p.active { "active" } else { "idle" }, p.input_tokens))
                            .collect();
                        let plan_summary = active_tasks.join("\n");
                        parallel_consult_conductor(
                            &mut state, &mut pool,
                            "Periodic implementation check-in (120s interval)",
                            &plan_summary,
                        ).await;
                        state.last_periodic_conductor_consult = Some(Instant::now());
                    }
                }

            }
        }

        if executor.is_complete() {
            state.complete = true;
            state.orchestrator_state = "complete".to_string();
            state.add_log("executor", "All plans complete", LogLevel::Info);

            // Generate batch summary
            let batch_summary = crate::orchestrator::context::generate_batch_summary(
                &config.batch_id,
                &[],
                executor.progress().total_plans,
                executor.progress().total_tasks,
                Duration::from_secs(0), // TODO: track wall clock
                0,
                (0, 0),
            );
            let _ = crate::orchestrator::context::write_batch_summary(
                &config.repo_root,
                &config.batch_id,
                &batch_summary,
            );

            // Show completion screen — keep animating until the user quits
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        atmosphere.tick_with_degraded(state.any_agent_active());
                        terminal.draw(|f| { tui::layout::render(f, &state, &atmosphere); })?;
                    }
                    Some(result) = term_events.next() => {
                        match result {
                        Ok(Event::Key(key)) => {
                            info!(
                                "KEY(completion): code={:?} kind={:?} modifiers={:?} tab={} mode={:?}",
                                key.code, key.kind, key.modifiers,
                                state.active_tab, state.input_mode
                            );
                            if key.kind == KeyEventKind::Press {
                                let sel_plan = state.plans.get(state.selected_plan_idx).map(|p| p.base.as_str()).unwrap_or("");
                                let action = crate::tui::input::handle_key(
                                    key, &state.input_mode, &state.message_input,
                                    &state.focus, state.show_plan_detail, state.active_tab,
                                    &state.agent_pane_group, state.show_task_detail, sel_plan,
                                    state.show_task_picker,
                                );
                                match action {
                                    TuiAction::Quit => break,
                                    TuiAction::SwitchTab(idx) if idx < 6 => { state.active_tab = idx; }
                                    TuiAction::NavigateUp => {
                                        match state.active_tab {
                                            1 => {
                                                if !state.pipeline_header_selected {
                                                    if state.selected_plan_idx > 0 {
                                                        state.selected_plan_idx -= 1;
                                                    } else if state.selected_wave_idx > 0 {
                                                        state.selected_wave_idx -= 1;
                                                        let count = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(1);
                                                        state.selected_plan_idx = count.saturating_sub(1);
                                                    } else {
                                                        state.pipeline_header_selected = true;
                                                    }
                                                }
                                                parallel_refresh_tasks(&mut state, &config);
                                            }
                                            2 => { state.agent_list_cursor = state.agent_list_cursor.saturating_sub(1); }
                                            3 => { state.git_branch_cursor = state.git_branch_cursor.saturating_sub(1); }
                                            _ => {}
                                        }
                                    }
                                    TuiAction::NavigateDown => {
                                        match state.active_tab {
                                            1 => {
                                                if state.pipeline_header_selected {
                                                    state.pipeline_header_selected = false;
                                                } else {
                                                    let wave_plan_count = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(state.plans.len());
                                                    if state.selected_plan_idx + 1 < wave_plan_count {
                                                        state.selected_plan_idx += 1;
                                                    } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                                        state.selected_wave_idx += 1;
                                                        state.selected_plan_idx = 0;
                                                    }
                                                }
                                                parallel_refresh_tasks(&mut state, &config);
                                            }
                                            2 => {
                                                let max = if !state.parallel_agents.is_empty() { state.parallel_agents.len().saturating_sub(1) } else { state.agents.len().saturating_sub(1) };
                                                if state.agent_list_cursor < max { state.agent_list_cursor += 1; }
                                            }
                                            3 => {
                                                let max = state.git_branch_tree.len().saturating_sub(1);
                                                if state.git_branch_cursor < max { state.git_branch_cursor += 1; }
                                            }
                                            _ => {}
                                        }
                                    }
                                    TuiAction::NavigatePageUp => {
                                        for _ in 0..10 {
                                            if state.selected_plan_idx > 0 {
                                                state.selected_plan_idx -= 1;
                                            } else if state.selected_wave_idx > 0 {
                                                state.selected_wave_idx -= 1;
                                                let count = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(1);
                                                state.selected_plan_idx = count.saturating_sub(1);
                                            } else { break; }
                                        }
                                        parallel_refresh_tasks(&mut state, &config);
                                    }
                                    TuiAction::NavigatePageDown => {
                                        for _ in 0..10 {
                                            let wpc = state.execution_waves.get(state.selected_wave_idx).map(|(_, p)| p.len()).unwrap_or(state.plans.len());
                                            if state.selected_plan_idx + 1 < wpc {
                                                state.selected_plan_idx += 1;
                                            } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                                state.selected_wave_idx += 1;
                                                state.selected_plan_idx = 0;
                                            } else { break; }
                                        }
                                        parallel_refresh_tasks(&mut state, &config);
                                    }
                                    TuiAction::FocusNext => {
                                        let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
                                        state.focus = match state.focus {
                                            FocusZone::Plans => FocusZone::Tasks,
                                            FocusZone::Tasks => FocusZone::AgentOutput,
                                            FocusZone::AgentOutput => if has_cmd { FocusZone::CommandOutput } else { FocusZone::Plans },
                                            FocusZone::CommandOutput => FocusZone::Plans,
                                        };
                                    }
                                    TuiAction::FocusPrev => {
                                        let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
                                        state.focus = match state.focus {
                                            FocusZone::Plans => if has_cmd { FocusZone::CommandOutput } else { FocusZone::AgentOutput },
                                            FocusZone::Tasks => FocusZone::Plans,
                                            FocusZone::AgentOutput => FocusZone::Tasks,
                                            FocusZone::CommandOutput => FocusZone::AgentOutput,
                                        };
                                    }
                                    TuiAction::WaveNext => {
                                        state.pipeline_header_selected = false;
                                        if state.selected_wave_idx + 1 < state.execution_waves.len() {
                                            state.selected_wave_idx += 1;
                                            state.selected_plan_idx = 0;
                                        }
                                        parallel_refresh_tasks(&mut state, &config);
                                    }
                                    TuiAction::WavePrev => {
                                        state.pipeline_header_selected = false;
                                        if state.selected_wave_idx > 0 {
                                            state.selected_wave_idx -= 1;
                                            state.selected_plan_idx = 0;
                                        }
                                        parallel_refresh_tasks(&mut state, &config);
                                    }
                                    TuiAction::DrillIn => {
                                        if state.active_tab == 1 && !state.execution_waves.is_empty() {
                                            if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                                let plan_base = plan.base.clone();
                                                for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                                    if wave_plans.contains(&plan_base) { state.wave_expanded.insert(idx); break; }
                                                }
                                            }
                                            parallel_refresh_tasks(&mut state, &config);
                                        }
                                    }
                                    TuiAction::DrillOut => {
                                        if state.active_tab == 1 && !state.execution_waves.is_empty() {
                                            if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                                let plan_base = plan.base.clone();
                                                for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                                    if wave_plans.contains(&plan_base) { state.wave_expanded.remove(&idx); break; }
                                                }
                                            }
                                        }
                                    }
                                    TuiAction::ScrollLogUp => { state.log_scroll = state.log_scroll.saturating_add(10); }
                                    TuiAction::ScrollLogDown => { state.log_scroll = state.log_scroll.saturating_sub(10); }
                                    TuiAction::ScrollAgentUp => {
                                        let total = current_agent_line_count(&state);
                                        state.agent_scroll = Some(match state.agent_scroll { None => total.saturating_sub(10), Some(n) => n.saturating_sub(10) });
                                    }
                                    TuiAction::ScrollAgentDown => {
                                        if let Some(n) = state.agent_scroll {
                                            let total = current_agent_line_count(&state);
                                            let new = n + 10;
                                            if new >= total.saturating_sub(20) { state.agent_scroll = None; } else { state.agent_scroll = Some(new); }
                                        }
                                    }
                                    TuiAction::ScrollAgentEnd => { state.agent_scroll = None; }
                                    TuiAction::SwitchAgentTab(idx) => {
                                        state.manual_agent_tab = true;
                                        if idx == usize::MAX {
                                            let agent_count = if !state.parallel_agents.is_empty() {
                                                let selected_base = state.plans.get(state.selected_plan_idx).map(|p| p.base.as_str()).unwrap_or("");
                                                state.parallel_agents.iter().filter(|p| p.plan.contains(selected_base) || selected_base.contains(p.plan.as_str())).count()
                                            } else { 7 };
                                            state.selected_agent_tab = (state.selected_agent_tab + 1) % agent_count.max(1);
                                        } else if idx < 7 { state.selected_agent_tab = idx; }
                                    }
                                    TuiAction::ShowPlanDetail => {
                                        if let Some(repo_root) = &state.repo_root {
                                            if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                                let path = repo_root.join(format!("plans/{}.md", plan.base));
                                                state.plan_detail_content = std::fs::read_to_string(&path).unwrap_or_else(|_| "Plan file not found.".to_string());
                                                state.plan_detail_scroll = 0;
                                                state.show_plan_detail = true;
                                            }
                                        }
                                    }
                                    TuiAction::ClosePlanDetail => {
                                        state.show_plan_detail = false;
                                        state.show_wave_overview = false;
                                        state.show_agent_pool_modal = false;
                                    }
                                    TuiAction::ScrollDetailUp => {
                                        let accel = state.scroll_accel.tick();
                                        match state.plan_detail_tab {
                                            PlanDetailTab::Summary => { state.plan_summary_scroll = state.plan_summary_scroll.saturating_sub(accel); }
                                            PlanDetailTab::PlanDetails => { state.plan_detail_scroll = state.plan_detail_scroll.saturating_sub(accel); }
                                        }
                                    }
                                    TuiAction::ScrollDetailDown => {
                                        let accel = state.scroll_accel.tick();
                                        match state.plan_detail_tab {
                                            PlanDetailTab::Summary => { state.plan_summary_scroll += accel; }
                                            PlanDetailTab::PlanDetails => { state.plan_detail_scroll += accel; }
                                        }
                                    }
                                    TuiAction::ScrollDetailPageUp => {
                                        let page = state.terminal_height.saturating_sub(6) as usize;
                                        match state.plan_detail_tab {
                                            PlanDetailTab::Summary => { state.plan_summary_scroll = state.plan_summary_scroll.saturating_sub(page); }
                                            PlanDetailTab::PlanDetails => { state.plan_detail_scroll = state.plan_detail_scroll.saturating_sub(page); }
                                        }
                                    }
                                    TuiAction::ScrollDetailPageDown => {
                                        let page = state.terminal_height.saturating_sub(6) as usize;
                                        match state.plan_detail_tab {
                                            PlanDetailTab::Summary => { state.plan_summary_scroll += page; }
                                            PlanDetailTab::PlanDetails => { state.plan_detail_scroll += page; }
                                        }
                                    }
                                    TuiAction::SwitchDetailTab => {
                                        state.plan_detail_tab = match state.plan_detail_tab {
                                            PlanDetailTab::Summary => PlanDetailTab::PlanDetails,
                                            PlanDetailTab::PlanDetails => if state.plan_summary_content.is_empty() { PlanDetailTab::PlanDetails } else { PlanDetailTab::Summary },
                                        };
                                    }
                                    TuiAction::ShowHelp => { state.show_help = !state.show_help; }
                                    TuiAction::ScrollFocusedUp => { state.task_scroll = state.task_scroll.saturating_sub(1); }
                                    TuiAction::ScrollFocusedDown => {
                                        let max = state.task_checklist.as_ref().map(|c| c.tasks.len()).unwrap_or(0);
                                        if state.task_scroll + 1 < max { state.task_scroll += 1; }
                                    }

                                    // --- Tab 0 plan selection ---
                                    TuiAction::SelectPlanUp => {
                                        state.selected_plan_idx = state.selected_plan_idx.saturating_sub(1);
                                        parallel_refresh_tasks(&mut state, &config);
                                    }
                                    TuiAction::SelectPlanDown => {
                                        if state.selected_plan_idx + 1 < state.plans.len() {
                                            state.selected_plan_idx += 1;
                                        }
                                        parallel_refresh_tasks(&mut state, &config);
                                    }

                                    // --- Detail sub-tabs ---
                                    TuiAction::SwitchDetailSubTab(idx) => {
                                        state.detail_sub_tab = match idx {
                                            0 => DetailSubTab::Agents,
                                            1 => DetailSubTab::Output,
                                            2 => DetailSubTab::Diff,
                                            3 => DetailSubTab::Errors,
                                            4 => DetailSubTab::Git,
                                            _ => DetailSubTab::Agents,
                                        };
                                        if state.active_tab != 0 { state.active_tab = 0; }
                                    }

                                    // --- Task detail ---
                                    TuiAction::ShowTaskDetail => {
                                        state.show_task_detail = true;
                                        state.task_detail_scroll = 0;
                                    }
                                    TuiAction::CloseTaskDetail => { state.show_task_detail = false; }
                                    TuiAction::ScrollTaskDetailUp => {
                                        state.task_detail_scroll = state.task_detail_scroll.saturating_sub(1);
                                    }
                                    TuiAction::ScrollTaskDetailDown => { state.task_detail_scroll += 1; }

                                    // --- Modals ---
                                    TuiAction::ShowWaveOverview => {
                                        state.show_wave_overview = !state.show_wave_overview;
                                        state.show_agent_pool_modal = false;
                                    }
                                    TuiAction::ShowAgentPoolModal => {
                                        state.show_agent_pool_modal = !state.show_agent_pool_modal;
                                        state.show_wave_overview = false;
                                    }
                                    TuiAction::DismissNotification => { state.notifications.pop(); }

                                    // --- Wave collapse/expand ---
                                    TuiAction::CollapseExpand => {
                                        if !state.execution_waves.is_empty() {
                                            if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                                                let plan_base = plan.base.clone();
                                                for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                                                    if wave_plans.contains(&plan_base) {
                                                        if state.wave_expanded.contains(&idx) {
                                                            state.wave_expanded.remove(&idx);
                                                        } else {
                                                            state.wave_expanded.insert(idx);
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // --- Task expand/collapse ---
                                    TuiAction::ExpandCollapse => {
                                        if let Some(ref cl) = state.task_checklist {
                                            if let Some(task) = cl.tasks.get(state.task_scroll) {
                                                let tid = task.id.clone();
                                                if state.task_expanded.as_deref() == Some(&tid) {
                                                    state.task_expanded = None;
                                                } else {
                                                    state.task_expanded = Some(tid);
                                                }
                                            }
                                        }
                                    }

                                    // --- Diff / command output scroll ---
                                    TuiAction::ScrollDiffUp => {
                                        if state.focus == FocusZone::CommandOutput {
                                            let total = state.command_output.lines().count();
                                            let page = 10;
                                            state.command_output_scroll = Some(match state.command_output_scroll {
                                                None => total.saturating_sub(page),
                                                Some(n) => n.saturating_sub(page),
                                            });
                                        } else {
                                            let total = state.branch_diff.lines().count();
                                            let page = 10;
                                            state.diff_scroll = Some(match state.diff_scroll {
                                                None => total.saturating_sub(page),
                                                Some(n) => n.saturating_sub(page),
                                            });
                                        }
                                    }
                                    TuiAction::ScrollDiffDown => {
                                        if state.focus == FocusZone::CommandOutput {
                                            if let Some(n) = state.command_output_scroll {
                                                let total = state.command_output.lines().count();
                                                let new = n + 10;
                                                if new >= total.saturating_sub(20) {
                                                    state.command_output_scroll = None;
                                                } else {
                                                    state.command_output_scroll = Some(new);
                                                }
                                            }
                                        } else if let Some(n) = state.diff_scroll {
                                            let total = state.branch_diff.lines().count();
                                            let new = n + 10;
                                            if new >= total.saturating_sub(20) {
                                                state.diff_scroll = None;
                                            } else {
                                                state.diff_scroll = Some(new);
                                            }
                                        }
                                    }

                                    // --- Config panel ---
                                    TuiAction::ConfigUp => {
                                        state.config.selected_row = state.config.selected_row.saturating_sub(1);
                                    }
                                    TuiAction::ConfigDown => {
                                        let max = state.config.row_count().saturating_sub(1);
                                        if state.config.selected_row < max {
                                            state.config.selected_row += 1;
                                        }
                                    }
                                    TuiAction::ConfigLeft => { handle_config_cycle(&mut state, false); }
                                    TuiAction::ConfigRight => { handle_config_cycle(&mut state, true); }
                                    TuiAction::ConfigSelect => {
                                        handle_config_select(&mut state, &config);
                                        // Hot reload: kill agents whose model changed since last Apply
                                        for role in state.pending_agent_kills.drain(..).collect::<Vec<_>>() {
                                            tracing::info!("Hot reload: killing {} (model changed to {})",
                                                role, state.config.model_for(role).unwrap_or("?"));
                                            pool.kill_role(role).await;
                                            state.add_log("config",
                                                &format!("Reloaded {}: model={}", role, state.config.model_for(role).unwrap_or("?")),
                                                LogLevel::Info);
                                        }
                                        // Sync fallback model to pool
                                        pool.set_fallback_model(state.config.fallback_model.clone());
                                    }

                                    // --- Agent pane group / verify tabs ---
                                    TuiAction::ToggleAgentPaneGroup => {
                                        state.agent_pane_group = match state.agent_pane_group {
                                            AgentPaneGroup::Implementation => AgentPaneGroup::Verification,
                                            AgentPaneGroup::Verification => AgentPaneGroup::Implementation,
                                        };
                                    }
                                    TuiAction::VerifyTabNext => {
                                        if !state.verify_entries.is_empty() {
                                            state.selected_verify_idx =
                                                (state.selected_verify_idx + 1) % state.verify_entries.len();
                                        }
                                    }
                                    TuiAction::VerifyTabPrev => {
                                        if !state.verify_entries.is_empty() {
                                            state.selected_verify_idx = if state.selected_verify_idx == 0 {
                                                state.verify_entries.len() - 1
                                            } else {
                                                state.selected_verify_idx - 1
                                            };
                                        }
                                    }

                                    // --- Confirm flow (ctrl+r, ctrl+d, etc.) ---
                                    TuiAction::RequestConfirm(confirm_action) => {
                                        state.pending_confirm = Some(confirm_action);
                                        state.input_mode = InputMode::Confirm;
                                    }
                                    TuiAction::ConfirmYes => {
                                        if let Some(confirmed) = state.pending_confirm.take() {
                                            state.input_mode = InputMode::Normal;
                                            match confirmed {
                                                ConfirmAction::RestartAllPlans => {
                                                    state.add_log("executor", "Restarting ALL plans", LogLevel::Warn);
                                                    let plan_bases: Vec<String> = state.plans.iter().map(|p| p.base.clone()).collect();
                                                    for plan in &plan_bases {
                                                        pool.kill_plan_agents(plan).await;
                                                        executor.reset_plan(plan);
                                                        let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                                        if wt_path.exists() {
                                                            let pw = crate::git::worktree::PlanWorktree {
                                                                path: wt_path,
                                                                branch: format!("codex/plan/{plan}"),
                                                                plan_base: plan.clone(),
                                                            };
                                                            let _ = worktree_mgr.cleanup_plan_worktree(&pw);
                                                        }
                                                        let _ = git_manager.delete_tag(&format!("plan/{plan}"));
                                                        let plan_branch = format!("codex/plan/{plan}");
                                                        let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                                    }
                                                    state.plan_review_stage.clear();
                                                    state.plan_pending_reviews.clear();
                                                    state.plan_doc_revisions.clear();
                                                    state.plan_agent_retries.clear();
                                                    state.parallel_agents.clear();
                                                    state.executor_completed_tasks.clear();
                                                    state.plan_gate_outputs.clear();
                                                    state.plan_phase_started.clear();
                                                    state.plan_start_times.clear();
                                                    state.task_started_at.clear();
                                                    for entry in &mut state.plans {
                                                        entry.status = RunPlanStatus::Pending;
                                                        entry.phase = String::new();
                                                        entry.iteration = 0;
                                                        entry.started_at = None;
                                                    }
                                                    state.complete = false;
                                                    // Clear persisted state so executor doesn't restore old completions
                                                    let _ = std::fs::remove_file(config.repo_root.join("tmp/plan-runs/task-state.json"));
                                                    let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                                    execute_actions(
                                                        actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                        &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                        &spawn_ready_tx,
                                                    ).await?;
                                                    // Break out of completion loop so the executor loop takes over
                                                    break;
                                                }
                                                ConfirmAction::RestartPhase => {
                                                    state.add_log("executor", "Restart phase not available in parallel mode", LogLevel::Warn);
                                                }
                                                ConfirmAction::ResetSelectedPlan(_) => {
                                                    if let Some(entry) = state.plans.get(state.selected_plan_idx) {
                                                        let plan = entry.base.clone();
                                                        state.add_log("executor", &format!("Resetting plan {plan}"), LogLevel::Warn);
                                                        pool.kill_plan_agents(&plan).await;
                                                        executor.reset_plan(&plan);
                                                        let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                                                        if wt_path.exists() {
                                                            let pw = crate::git::worktree::PlanWorktree {
                                                                path: wt_path,
                                                                branch: format!("codex/plan/{plan}"),
                                                                plan_base: plan.clone(),
                                                            };
                                                            let _ = worktree_mgr.cleanup_plan_worktree(&pw);
                                                        }
                                                        let _ = git_manager.delete_tag(&format!("plan/{plan}"));
                                                        state.plan_review_stage.remove(&plan);
                                                        state.plan_pending_reviews.remove(&plan);
                                                        state.plan_doc_revisions.remove(&plan);
                                                        state.plan_agent_retries.retain(|k, _| !k.contains(&plan));
                                                        state.parallel_agents.retain(|p| p.plan != plan);
                                                        state.executor_completed_tasks.retain(|k| !k.starts_with(&format!("{plan}:")));
                                                        state.plan_gate_outputs.remove(&plan);
                                                        state.plan_phase_started.remove(&plan);
                                                        state.plan_start_times.remove(&plan);
                                                        state.task_started_at.retain(|k, _| !k.starts_with(&format!("{plan}:")));
                                                        let plan_branch = format!("codex/plan/{plan}");
                                                        let _ = crate::git::ops::run_git(&config.repo_root, &["branch", "-D", &plan_branch]);
                                                        if let Some(entry) = state.plans.iter_mut().find(|p| p.base == plan) {
                                                            entry.status = RunPlanStatus::Pending;
                                                            entry.phase = String::new();
                                                            entry.iteration = 0;
                                                            entry.started_at = None;
                                                        }
                                                        state.complete = false;
                                                        // Clear persisted state so executor doesn't restore old completions
                                                        let _ = std::fs::remove_file(config.repo_root.join("tmp/plan-runs/task-state.json"));
                                                        let actions = executor.schedule_next_with_budget(Some(active_agent_count(&state)));
                                                        execute_actions(
                                                            actions, &mut executor, &mut pool, &worktree_mgr, &mut state,
                                                            &config, &persistence, &gate_tx, &batch_branch, &git_manager,
                                                            &spawn_ready_tx,
                                                        ).await?;
                                                        break;
                                                    }
                                                }
                                                ConfirmAction::GitReconcile => {
                                                    if state.git_reconcile_in_progress {
                                                        state.add_log("git", "Reconcile already in progress", LogLevel::Warn);
                                                    } else {
                                                        state.git_reconcile_in_progress = true;
                                                        state.add_log("git", "Starting git reconcile...", LogLevel::Info);
                                                        let repo = config.repo_root.clone();
                                                        let batch = batch_branch.to_string();
                                                        let bid = config.batch_id.clone();
                                                        let plan_info: Vec<(String, String)> = state.plans.iter()
                                                            .map(|p| (p.base.clone(), p.num.clone()))
                                                            .collect();
                                                        let tx = gate_tx.clone();
                                                        tokio::task::spawn_blocking(move || {
                                                            let (messages, merged_plans, already_reconciled) = git_reconcile(&repo, &batch, &bid, &plan_info);
                                                            let _ = tx.send(GateCompletion::ReconcileComplete { messages, merged_plans, already_reconciled });
                                                        });
                                                    }
                                                }
                                                _ => {
                                                    state.add_log("executor", "Action not available on completion screen", LogLevel::Warn);
                                                }
                                            }
                                        }
                                    }
                                    TuiAction::ConfirmNo => {
                                        state.pending_confirm = None;
                                        state.input_mode = InputMode::Normal;
                                    }

                                    _ => {}
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Terminal event error (completion): {e}");
                        }
                        }
                    }
                }
            }
            // If plans were restarted, loop back to the executor;
            // otherwise (Quit), break out to cleanup.
            if state.complete {
                break;
            } else {
                continue;
            }
        }
    }

    // Cleanup
    pool.kill_all().await;
    worktree_mgr.cleanup_all()?;
    persistence.cleanup_pid();
    let _ = tui::restore();

    Ok(())
}

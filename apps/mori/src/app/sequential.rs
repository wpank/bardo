use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{info, warn};

use crate::agent::{AgentEvent, AgentPool, AgentRole};
use crate::conductor::{Conductor, ConductorAction, ConductorConfig, ConductorContext};
use crate::git::{AutoStashSession, GitEvent, GitManager};
use crate::orchestrator::tasks;
use crate::orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorEvent, OrchestratorState};
use crate::state::persistence::PersistenceManager;
use crate::state::{
    ConductorHistoryEntry, LogLevel, Notification, RunPlanEntry, RunPlanStatus, RunState,
};
use crate::tui::{self, atmosphere::Atmosphere, input, TuiAction};

use super::*;

/// Run the application in sequential mode.
pub(crate) async fn run_sequential(config: AppConfig) -> Result<()> {
    // Channels
    let (agent_tx, mut agent_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let (orch_tx, mut orch_rx) = mpsc::unbounded_channel::<OrchestratorEvent>();
    let (git_tx, mut git_rx) = mpsc::unbounded_channel::<GitEvent>();

    // Initialize subsystems
    let orch_config = OrchestratorConfig {
        repo_root: config.repo_root.clone(),
        plans_dir: crate::orchestrator::paths::plans_root(&config.repo_root),
        no_review: config.no_review,
        skip_tests: config.skip_tests,
        max_iterations: config.max_iterations,
        batch_size: config.batch_size,
        model: config.model.clone(),
        no_docs: config.no_docs,
        ..OrchestratorConfig::new(config.repo_root.clone())
    };
    let mut orchestrator = Orchestrator::new(orch_config, orch_tx.clone());
    orchestrator.discover_plans(&config.plan_specs)?;

    let mut agent_pool = AgentPool::new(config.repo_root.clone(), agent_tx);
    let git_manager = GitManager::new(config.repo_root.clone(), git_tx);
    let _auto_stash_session = AutoStashSession::new(&git_manager);
    let persistence = PersistenceManager::new(&config.repo_root);
    persistence.ensure_dirs()?;
    persistence.write_pid()?;

    let started_at = chrono::Utc::now().to_rfc3339();
    let run_id = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();

    // Gate completion channel (non-blocking gates)
    let (gate_tx, mut gate_rx) = mpsc::unbounded_channel::<GateCompletion>();

    // Verification completion channel
    let (verify_tx, mut verify_rx) = mpsc::unbounded_channel::<VerifyCompletion>();

    // Build initial state
    let mut state = RunState::default();
    state.repo_root = Some(config.repo_root.clone());
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

    // Load plan estimates into the time estimator
    for plan in &orchestrator.plans {
        if let Some(ref fm) = plan.frontmatter {
            if let Some(est) = fm.estimated_minutes {
                state.time_estimator.load_plan_estimate(&plan.base, est);
            }
        }
    }

    // Load task files so the plan DAG can derive deps from cross-plan task refs.
    let mut seq_task_files: std::collections::HashMap<
        String,
        crate::orchestrator::tasks::TaskFile,
    > = std::collections::HashMap::new();
    for plan in &orchestrator.plans {
        if let Ok(Some(cl)) =
            crate::orchestrator::tasks::load_checklist(&config.repo_root, &plan.num)
        {
            seq_task_files.insert(
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

    // Build plan DAG and compute execution waves
    if let Ok(dag) =
        crate::orchestrator::PlanDag::from_plans_and_tasks(&orchestrator.plans, &seq_task_files)
    {
        let waves = dag.compute_waves();
        state.execution_waves = waves.iter().map(|w| (w.index, w.plans.clone())).collect();
        // Expand all waves by default in the plan tree
        for idx in 0..waves.len() {
            state.wave_expanded.insert(idx);
        }
        let total_eta = dag.estimated_total_minutes();
        if total_eta > 0 {
            state.add_log(
                "dag",
                &format!(
                    "Plan DAG: {} waves, ETA ~{}",
                    waves.len(),
                    crate::state::format_duration_minutes(total_eta),
                ),
                LogLevel::Info,
            );
        }
    }

    // Kill stale processes from prior run
    if let Some(stale_pid) = persistence.check_stale_pid() {
        state.add_log(
            "orch",
            &format!("Killed stale prior run (PID {})", stale_pid),
            LogLevel::Warn,
        );
    }

    // Terminal setup
    let mut terminal = tui::init().context("Failed to initialize terminal")?;
    let cleanup = || {
        let _ = tui::restore();
    };

    // Pre-initialize crossterm's event reader so we get a clear error instead of
    // a panic if the terminal event source cannot be created (e.g. when the process
    // is backgrounded and mio's kqueue setup fails).
    crossterm::event::poll(std::time::Duration::ZERO)
        .context("Terminal event source failed to initialize — is stdin a TTY?")?;
    let mut term_events = EventStream::new();

    // Tick interval (30fps)
    let mut tick = tokio::time::interval(Duration::from_millis(33));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut sys_collector = crate::sys_metrics::SysCollector::new();

    // Atmosphere
    let mut atmosphere = Atmosphere::new();

    // First paint before git / preflight / agent setup. `tui::init` already switched to the
    // alternate screen; without an immediate draw the user sees a blank view while `start_plan`
    // runs (including prd2 extraction and strategist spawn).
    state.terminal_height = terminal.size()?.height;
    terminal.draw(|f| {
        tui::layout::render(f, &state, &atmosphere);
    })?;

    // Conductor
    let conductor_config = ConductorConfig {
        llm_enabled: true,
        context_pressure_ratio: state.config.context_pressure_pct as f64 / 100.0,
        agent_soft_limit: config.max_agents,
        ..ConductorConfig::default()
    };
    let mut conductor = Conductor::new(conductor_config);

    // Conductor context tracking
    let mut last_message_at: Option<Instant> = None;
    let mut task_last_change: Option<Instant> = None;
    let mut compile_fail_count: u32 = 0;
    let mut last_compile_error = String::new();
    let mut last_turn_had_output = true;
    let mut last_turn_duration_secs: u64 = 0;
    let mut turn_started_at: Option<Instant> = None;

    // Set up batch branch
    let batch_branch = format!("codex/batch/{}", config.batch_id);
    git_manager.setup_batch_branch(&batch_branch)?;

    // Initialize config from CLI flags, then try to load persisted config.
    // CLI flags that were explicitly passed always win over the persisted value.
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
    // Apply execution preset (after load so preset overrides persisted values)
    if let Some(ref preset) = config.preset {
        state.config.apply_preset(preset);
    }
    agent_pool.set_fast_mode(state.config.fast_mode);
    agent_pool.set_fallback_model(state.config.fallback_model.clone());

    // Set git branch and build branch tree
    state.git_branch = git_manager.current_branch().unwrap_or_default();
    state.git_branch_tree = crate::git::graph::build_branch_tree(&config.repo_root, &state.plans);

    orchestrator.set_state(OrchestratorState::PlanReady);
    state.orchestrator_state = "plan-ready".to_string();

    // Spawn the Conductor meta-agent (lives for the whole run).
    // Paint an "initializing" frame first — cursor agent cold start can take 60-90s
    // and without this the TUI looks frozen before the event loop starts.
    state.add_log(
        "conductor",
        "Spawning conductor agent (cursor cold start may take up to 90s)…",
        LogLevel::Info,
    );
    terminal.draw(|f| {
        tui::layout::render(f, &state, &atmosphere);
    })?;
    {
        if let Err(e) = agent_pool
            .spawn(
                AgentRole::Conductor,
                "max",
                state.config.model_for(AgentRole::Conductor),
            )
            .await
        {
            state.add_log(
                "conductor",
                &format!("Failed to spawn conductor agent: {e}"),
                LogLevel::Warn,
            );
        } else {
            state.add_log("conductor", "Conductor agent spawned", LogLevel::Info);
            // Send initial system prompt
            let system_prompt = crate::conductor::llm::conductor_system_prompt();
            let plan_summary: String = orchestrator
                .plans
                .iter()
                .map(|p| format!("  - {}", p.base))
                .collect::<Vec<_>>()
                .join("\n");
            let init_msg = format!(
                "{system_prompt}\n\n## Current Batch\n\nPlans to execute:\n{plan_summary}\n\nThe pipeline is starting. Respond with [OK] to acknowledge.",
            );
            let _ = agent_pool
                .turn_start(
                    AgentRole::Conductor,
                    &init_msg,
                    state.config.model_for(AgentRole::Conductor),
                )
                .await;
            state.agent_state_mut(AgentRole::Conductor).active = true;
        }
    }

    // Start first plan. Paint before spawning plan agents (same cold-start lag applies).
    // Errors here used to crash bardo-ctl; now they surface in the TUI instead so the
    // user can see what went wrong without losing the terminal session.
    if !orchestrator.plans.is_empty() {
        terminal.draw(|f| {
            tui::layout::render(f, &state, &atmosphere);
        })?;
        if let Err(e) = start_plan(
            &mut state,
            &mut orchestrator,
            &mut agent_pool,
            &git_manager,
            &persistence,
            &config,
            &batch_branch,
            &run_id,
            &started_at,
            &mut compile_fail_count,
        )
        .await
        {
            state.add_log(
                "orch",
                &format!("Failed to start plan: {e}"),
                LogLevel::Error,
            );
        }
    }

    // Spawn retroactive verifiers for completed-prior plans
    spawn_retroactive_verifiers(&mut state, &config, verify_tx.clone());

    // Task checklist poll timer
    let mut task_poll = tokio::time::interval(Duration::from_secs(5));
    task_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Git diff check throttle (runs every ~5s instead of every 33ms tick)
    let mut git_check_counter: u32 = 0;
    let mut has_git_changes = false;

    // Crash state snapshot throttle (~2s)
    let mut last_crash_snapshot = Instant::now();

    // Main event loop
    loop {
        atmosphere.tick_with_degraded(state.any_agent_active());
        sys_collector.poll(&mut state.sys);

        // Update global crash state every ~2s
        if last_crash_snapshot.elapsed() >= Duration::from_secs(2) {
            crate::update_crash_state(state.snapshot_for_crash());
            last_crash_snapshot = Instant::now();
        }
        if let Some((x, y)) = state.particle_burst_pending.take() {
            atmosphere.spawn_burst(x, y, 8);
        }

        // Sample token burn history for sparklines (~1 sample per tick, ring buffer of 120)
        {
            let samples: Vec<_> = state
                .agents
                .iter()
                .map(|(&r, a)| (r, a.input_tokens))
                .collect();
            for (role, tokens) in samples {
                let history = state.token_burn_history.entry(role).or_default();
                history.push_back(tokens);
                if history.len() > 120 {
                    history.pop_front();
                }
            }
        }

        let term_h = terminal.size()?.height;
        state.terminal_height = term_h;
        terminal.draw(|f| {
            tui::layout::render(f, &state, &atmosphere);
        })?;

        tokio::select! {
            Some(event) = agent_rx.recv() => {
                if matches!(event, AgentEvent::MessageDelta { .. }) {
                    last_message_at = Some(Instant::now());
                }
                handle_agent_event(
                    &mut state, &mut orchestrator, &mut agent_pool, &git_manager, &persistence,
                    &config, &mut conductor, &batch_branch, &run_id, &started_at, event,
                    &mut task_last_change,
                    &gate_tx,
                    &mut turn_started_at,
                    &mut last_turn_had_output,
                    &mut last_turn_duration_secs,
                ).await?;
            }
            Some(event) = orch_rx.recv() => {
                handle_orchestrator_event(&mut state, &event);
            }
            Some(event) = git_rx.recv() => {
                handle_git_event(&mut state, &event);
            }
            Some(result) = term_events.next() => {
                match result {
                    Ok(Event::Key(key)) => {
                        info!(
                            "KEY(seq): code={:?} kind={:?} modifiers={:?} tab={} mode={:?}",
                            key.code, key.kind, key.modifiers,
                            state.active_tab, state.input_mode
                        );
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        let sel_plan = state.plans.get(state.selected_plan_idx).map(|p| p.base.as_str()).unwrap_or("");
                        let action = input::handle_key(key, &state.input_mode, &state.message_input, &state.focus, state.show_plan_detail, state.active_tab, &state.agent_pane_group, state.show_task_detail, sel_plan, state.show_task_picker);
                        let should_quit = handle_tui_action(
                            &mut state, &mut agent_pool, &mut orchestrator,
                            &git_manager, &persistence, &config, &batch_branch, &run_id, &started_at,
                            action,
                        ).await?;
                        if should_quit {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Terminal event error (seq): {e}");
                    }
                }
            }
            _ = task_poll.tick() => {
                if let Some(plan) = orchestrator.current_plan() {
                    let num = plan.num.clone();
                    let plan_base = plan.base.clone();
                    if let Ok(Some(cl)) = tasks::load_checklist(&config.repo_root, &num) {
                        let new_done = cl.done_count();
                        let old_done = state.task_checklist.as_ref().map(|c| c.done_count()).unwrap_or(0);
                        if new_done != old_done {
                            task_last_change = Some(Instant::now());
                        }
                        // Load task time estimates into the estimator
                        for task in &cl.tasks {
                            if let Some(est) = task.estimated_minutes {
                                state.time_estimator.load_task_estimate(&plan_base, &task.id, est);
                            }
                        }
                        state.task_checklist = Some(cl);
                        state.active_task_display = state.task_checklist.as_ref()
                            .and_then(|c| c.active_task())
                            .map(|t| format!("{} {}", t.id, t.title));
                    }
                }
            }
            Some(completion) = gate_rx.recv() => {
                match completion {
                    GateCompletion::Compile { result, .. } => {
                        state.gate_running.remove("cargo check --workspace");
                        let gate = result?;
                        state.command_output = gate.output.clone();
                        handle_compile_result(
                            &mut state, &mut orchestrator, &mut agent_pool,
                            &git_manager, &persistence, &config, &batch_branch,
                            &run_id, &started_at, &gate_tx,
                            &mut compile_fail_count, &mut last_compile_error,
                            gate,
                        ).await?;
                    }
                    GateCompletion::Clippy { result, .. } => {
                        state.gate_running.remove("cargo clippy --workspace");
                        let gate = result?;
                        state.command_output = gate.output.clone();
                        state.add_log("gate", "Clippy gate complete", LogLevel::Info);

                        // Query sccache stats after clippy
                        if let Some(stats) = crate::orchestrator::gates::sccache_stats().await {
                            state.sccache_stats = Some(stats);
                        }

                        // Proceed to test gate or reviews
                        let plan = orchestrator.current_plan().cloned();
                        if !config.skip_tests {
                            start_test_gate(&mut state, &mut orchestrator, &config, &gate_tx);
                        } else if !config.no_review {
                            start_parallel_reviews(&mut state, &mut orchestrator, &mut agent_pool, &persistence, &config).await?;
                        } else if let Some(plan) = plan {
                            commit_and_advance(
                                &mut state, &mut orchestrator, &mut agent_pool, &git_manager, &persistence,
                                &config, &batch_branch, &run_id, &started_at, &plan,
                            ).await?;
                        }
                    }
                    GateCompletion::Test { result, .. } => {
                        state.gate_running.remove("cargo test --workspace");
                        let gate = result?;
                        state.command_output = gate.output.clone();
                        handle_test_result(
                            &mut state, &mut orchestrator, &mut agent_pool,
                            &git_manager, &persistence, &config, &batch_branch,
                            &run_id, &started_at,
                            gate,
                        ).await?;
                    }
                    GateCompletion::TerminalRender { plan, result } => {
                        state.gate_running.remove("terminal render test");
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
                        state.gate_running.remove("golem lifecycle test");
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
                        state.gate_running.remove("post-merge regression");
                        match result {
                            Ok(gate) if gate.passed => {
                                state.add_log("gate", &format!("Post-merge check PASS ({plan})"), LogLevel::Info);
                            }
                            Ok(gate) => {
                                state.add_log("gate", &format!("Post-merge check WARN ({plan}): {}", gate.output.lines().last().unwrap_or("")), LogLevel::Warn);
                            }
                            Err(e) => {
                                state.add_log("gate", &format!("Post-merge check ERROR ({plan}): {e}"), LogLevel::Warn);
                            }
                        }
                    }
                    GateCompletion::MergeComplete { .. } => {
                        state.gate_running.remove("merge");
                        // Sequential mode does not use async merges
                    }
                    GateCompletion::ReconcileComplete { messages, merged_plans, .. } => {
                        state.gate_running.remove("git reconcile");
                        state.git_reconcile_in_progress = false;
                        for msg in &messages {
                            let level = if msg.contains("ERROR") { LogLevel::Error } else { LogLevel::Info };
                            state.add_log("reconcile", msg, level);
                        }
                        for plan in &merged_plans {
                            if let Some(entry) = state.plans.iter_mut().find(|p| &p.base == plan) {
                                entry.status = RunPlanStatus::Completed;
                                entry.phase = "complete".to_string();
                            }
                        }
                    }
                }
            }
            Some(completion) = verify_rx.recv() => {
                match completion {
                    VerifyCompletion::Done { plan_base, plan_num, passed, output, summary } => {
                        // Update verify entry status
                        if let Some(entry) = state.verify_entries.iter_mut().find(|e| e.plan_base == plan_base && e.plan_num == plan_num) {
                            entry.status = if passed {
                                VerifyStatus::Passed
                            } else {
                                VerifyStatus::Failed("verification failed".to_string())
                            };
                            entry.output = output;
                        }
                        // Write summary if generated
                        if let Some(ref summary_content) = summary {
                            let _ = crate::orchestrator::context::write_summary(&config.repo_root, &plan_num, summary_content);
                        }
                        // Toast
                        let icon = if passed { "✓" } else { "✗" };
                        state.notifications.push(Notification {
                            message: format!("{icon} verify {plan_base}: {}", if passed { "pass" } else { "fail" }),
                            created: Instant::now(),
                            ttl_secs: 8,
                            level: if passed { LogLevel::Info } else { LogLevel::Warn },
                        });
                    }
                }
            }
            _ = tick.tick() => {
                // Expire notifications
                state.notifications.retain(|n| n.created.elapsed().as_secs() < n.ttl_secs);

                // Update token flash counters
                {
                    let roles: Vec<AgentRole> = state.agents.keys().copied().collect();
                    for role in roles {
                        let cur = state.agents.get(&role)
                            .map(|a| (a.input_tokens, a.output_tokens))
                            .unwrap_or((0, 0));
                        let prev = state.token_prev.get(&role).copied().unwrap_or((0, 0));
                        if cur != prev {
                            state.token_flash.insert(role, 10);
                            state.token_prev.insert(role, cur);
                        }
                        if let Some(f) = state.token_flash.get_mut(&role) {
                            *f = f.saturating_sub(1);
                        }
                    }
                }

                // Ghost turn git diff check (throttled to ~5s to avoid blocking the runtime)
                git_check_counter += 1;
                if git_check_counter >= 150 {
                    git_check_counter = 0;
                    has_git_changes = tokio::process::Command::new("git")
                        .args(["diff", "--stat"])
                        .current_dir(&config.repo_root)
                        .output().await
                        .map(|o| !o.stdout.is_empty())
                        .unwrap_or(false);
                    // Refresh branch tree for git view
                    state.git_branch_tree = crate::git::graph::build_branch_tree(&config.repo_root, &state.plans);
                }

                // Run conductor watchers
                let active_role = state.agents.iter()
                    .find(|(_, s)| s.active)
                    .map(|(r, _)| *r);
                let input_tokens = active_role
                    .and_then(|r| state.agent_state(r))
                    .map(|a| a.input_tokens)
                    .unwrap_or(0);

                let ctx = ConductorContext {
                    active_role,
                    last_message_at,
                    phase_started: state.phase_started,
                    compile_fail_count,
                    last_compile_error: last_compile_error.clone(),
                    task_last_change,
                    input_tokens,
                    context_limit: state.context_limit,
                    iteration: state.current_iteration,
                    consecutive_revise_count: state.consecutive_revise_count,
                    orchestrator_state: state.orchestrator_state.clone(),
                    last_turn_had_output,
                    last_turn_duration_secs,
                    last_turn_has_git_changes: has_git_changes,
                    plan: String::new(),
                    agent_backend: None,
                    task_summary: None,
                    active_instance_id: None,
                    test_pass_count: 0,
                    test_fail_count: 0,
                    compile_gates_passed: compile_fail_count == 0,
                };

                let interventions = conductor.tick(&ctx);
                for intervention in interventions {
                    state.conductor_history.push(ConductorHistoryEntry {
                        timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                        watcher: intervention.watcher.clone(),
                        target: intervention.target_role.to_string(),
                        message: intervention.message.clone(),
                    });
                    state.add_log(
                        "conductor",
                        &format!("[{}] {}", intervention.watcher, intervention.message),
                        LogLevel::Warn,
                    );

                    // Execute the conductor's action
                    use crate::conductor::ConductorAction;
                    match intervention.action {
                        Some(ConductorAction::SendMessage { role, ref message }) => {
                            if agent_pool.is_spawned(role) {
                                let echo = format!("\n--- Conductor ---\n{message}\n-----------------\n");
                                state.agent_state_mut(role).output.push_str(&echo);
                                let _ = agent_pool.turn_interrupt(role).await;
                                let _ = agent_pool.turn_start(role, message, None).await;
                                state.agent_state_mut(role).active = true;
                            }
                        }
                        Some(ConductorAction::RestartAgent { role }) => {
                            agent_pool.kill(role).await;
                            state.agent_state_mut(role).active = false;
                            state.add_log("conductor", &format!("Killed and will restart {role}"), LogLevel::Warn);
                            // Re-dispatch the current phase
                            let _ = restart_current_phase(
                                &mut state, &mut orchestrator, &mut agent_pool, &persistence, &config,
                            ).await;
                        }
                        Some(ConductorAction::ForceAdvance) => {
                            let watcher_name = intervention.watcher.as_str();
                            state.add_log("conductor", &format!("Force advancing — {watcher_name}"), LogLevel::Warn);

                            // Log any outstanding test failures as deferred
                            if !state.last_gate_passed && !state.last_gate_output.is_empty() {
                                let plan_base = orchestrator.current_plan()
                                    .map(|p| p.base.clone())
                                    .unwrap_or_default();
                                let failing = crate::orchestrator::gates::extract_failing_test_names(&state.last_gate_output);
                                let snippet = crate::orchestrator::gates::extract_test_failure_snippet(&state.last_gate_output, 50);
                                let reason = if watcher_name == "TestFailureBudget" {
                                    crate::state::persistence::DeferredReason::BudgetAllowed {
                                        pass_rate: if state.last_gate_output.is_empty() { 0.0 } else {
                                            let counts = crate::orchestrator::gates::parse_test_counts_pub(&state.last_gate_output);
                                            let total = counts.0 + counts.1;
                                            if total > 0 { counts.0 as f64 / total as f64 } else { 0.0 }
                                        },
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
                                        iteration: state.current_iteration,
                                    }
                                }).collect();
                                if !failures.is_empty() {
                                    let batch_id = batch_branch.strip_prefix("codex/batch/").unwrap_or(&batch_branch);
                                    let _ = persistence.append_deferred_failures(batch_id, failures);
                                }
                            }

                            let _ = force_advance(
                                &mut state, &mut orchestrator, &mut agent_pool, &git_manager,
                                &persistence, &config, &batch_branch, &run_id, &started_at,
                            ).await;
                        }
                        Some(ConductorAction::SkipReviews) => {
                            state.add_log("conductor", "Skipping reviews — review loop detected", LogLevel::Warn);
                            // Kill active reviewers
                            for &r in &[AgentRole::Architect, AgentRole::Auditor, AgentRole::Scribe, AgentRole::Critic] {
                                if agent_pool.is_spawned(r) {
                                    agent_pool.kill(r).await;
                                    state.agent_state_mut(r).active = false;
                                }
                            }
                            state.pending_reviews.clear();
                            // Go straight to commit
                            if let Some(plan) = orchestrator.current_plan().cloned() {
                                let _ = commit_and_advance(
                                    &mut state, &mut orchestrator, &mut agent_pool, &git_manager,
                                    &persistence, &config, &batch_branch, &run_id, &started_at, &plan,
                                ).await;
                            }
                        }
                        Some(ConductorAction::PingWarmAgent { instance_id }) => {
                            // Lightweight keepalive for a warm agent (no-op in sequential mode for now).
                            state.add_log("conductor", &format!("Pinging warm agent {instance_id}"), LogLevel::Debug);
                        }
                        Some(ConductorAction::SpawnValidation { .. })
                        | Some(ConductorAction::GenerateFixPlan { .. })
                        | Some(ConductorAction::InsertGate { .. })
                        | Some(ConductorAction::SkipValidation { .. })
                        | Some(ConductorAction::AssignAdditionalTasks { .. }) => {
                            // Not applicable in sequential mode
                        }
                        None => {
                            // Legacy: just send a steering message
                            let role = intervention.target_role;
                            if agent_pool.is_spawned(role) {
                                let steering = crate::conductor::actions::steering_message(&intervention);
                                let _ = agent_pool.turn_interrupt(role).await;
                                let _ = agent_pool.turn_start(role, &steering, None).await;
                                state.agent_state_mut(role).active = true;
                            }
                        }
                    }
                }
            }
        }

        if state.complete {
            loop {
                atmosphere.tick_with_degraded(state.any_agent_active());
                let term_h = terminal.size()?.height;
                state.terminal_height = term_h;
                terminal.draw(|f| {
                    tui::layout::render(f, &state, &atmosphere);
                })?;
                if let Some(Ok(Event::Key(key))) = term_events.next().await {
                    if key.kind == KeyEventKind::Press {
                        let sel_plan = state
                            .plans
                            .get(state.selected_plan_idx)
                            .map(|p| p.base.as_str())
                            .unwrap_or("");
                        let action = input::handle_key(
                            key,
                            &state.input_mode,
                            &state.message_input,
                            &state.focus,
                            state.show_plan_detail,
                            state.active_tab,
                            &state.agent_pane_group,
                            state.show_task_detail,
                            sel_plan,
                            state.show_task_picker,
                        );
                        let should_quit = handle_tui_action(
                            &mut state,
                            &mut agent_pool,
                            &mut orchestrator,
                            &git_manager,
                            &persistence,
                            &config,
                            &batch_branch,
                            &run_id,
                            &started_at,
                            action,
                        )
                        .await?;
                        if should_quit {
                            break;
                        }
                    }
                }
            }
            break;
        }
    }

    // Cleanup
    agent_pool.kill_all().await;
    persistence.cleanup_pid();
    cleanup();

    Ok(())
}

/// Record elapsed time for the outgoing phase and set the new phase.
/// Also auto-switches the agent tab unless the user manually picked one.
pub(crate) fn transition_phase(state: &mut RunState, new_phase: &str) {
    if let Some(started) = state.phase_started {
        let elapsed = started.elapsed();
        state
            .phase_elapsed
            .push((state.current_phase.clone(), elapsed));
        // Feed actual phase duration into the time estimator
        state
            .time_estimator
            .record_phase_complete(&state.current_phase, elapsed.as_secs_f64() / 60.0);
    }
    state.current_phase = new_phase.to_string();
    state.phase_started = Some(Instant::now());
    state.auto_respond_count.clear();
    if !state.manual_agent_tab {
        state.selected_agent_tab = match new_phase {
            "strategist" => 0,
            "implementer" => 1,
            "reviewing" => 2,
            "critic-review" => 5,
            "doc-revision" => 4, // scribe tab
            _ => state.selected_agent_tab,
        };
    }
    state.manual_agent_tab = false;
}

/// Start executing the current plan
pub(crate) async fn start_plan(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    compile_fail_count: &mut u32,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    if orchestrator.should_skip_plan(&plan) {
        info!("Skipping plan {} (tag exists, completed prior)", plan.base);
        state.add_log(
            "orch",
            &format!("{} completed in prior run", plan.base),
            LogLevel::Info,
        );
        if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
            entry.status = RunPlanStatus::CompletedPrior;
            entry.phase = "complete".to_string();
        }
        orchestrator.plans_completed += 1;
        persistence.append_event(&PersistenceManager::make_event(
            "plan_skip",
            &plan.base,
            "",
            0,
        ))?;
        orchestrator.advance_to_next_plan();
        return Box::pin(start_plan(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
            compile_fail_count,
        ))
        .await;
    }

    // Check prior completions from events.jsonl (state resumption)
    if let Ok(completed) = persistence.completed_plans() {
        if completed.contains(&plan.base) {
            info!("Skipping plan {} (completed in prior run)", plan.base);
            state.add_log(
                "orch",
                &format!("Resuming: {} already completed", plan.base),
                LogLevel::Info,
            );
            if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
                entry.status = RunPlanStatus::CompletedPrior;
                entry.phase = "complete".to_string();
            }
            orchestrator.plans_completed += 1;
            orchestrator.advance_to_next_plan();
            return Box::pin(start_plan(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
                compile_fail_count,
            ))
            .await;
        }
    }

    // Reset compile fail count for the new plan
    *compile_fail_count = 0;

    // Fresh context for each plan: reset threads so agents get clean context windows
    agent_pool.reset_all_threads();
    state.reset_for_new_plan();

    state.current_plan_idx = orchestrator.current_plan_idx;
    state.current_iteration = 1;
    state.selected_plan_idx = orchestrator.current_plan_idx;
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.status = RunPlanStatus::Active;
        entry.phase = "preflight".to_string();
        entry.started_at = Some(Instant::now());
    }

    state.add_log(
        "orch",
        &format!("Starting plan: {}", plan.base),
        LogLevel::Info,
    );

    let status = PersistenceManager::make_status(
        run_id,
        &config.batch_id,
        orchestrator.total_plans() as u32,
        orchestrator.plans_completed as u32,
        &plan.base,
        "preflight",
        1,
        started_at,
    );
    persistence.write_status(&status)?;
    persistence.write_current_plan(&plan.base)?;
    persistence.append_event(&PersistenceManager::make_event(
        "plan_start",
        &plan.base,
        "preflight",
        1,
    ))?;

    git_manager.setup_plan_branch(&plan.base, batch_branch)?;

    orchestrator.set_state(OrchestratorState::Preflight);
    state.orchestrator_state = "preflight".to_string();
    state.phase_elapsed.clear();
    transition_phase(state, "preflight");

    crate::orchestrator::context::write_preflight_files(&config.repo_root)?;

    // Extract prd2 context for the current plan
    let _ = crate::orchestrator::context::extract_prd2_context(&config.repo_root, &plan.num);

    // DX preflight checks
    let warnings = crate::orchestrator::preflight::preflight_dx_checks(&config.repo_root);
    for w in &warnings {
        state.add_log("dx", w, LogLevel::Info);
    }
    state.preflight_warnings = warnings;

    state.add_log("orch", "Preflight complete", LogLevel::Info);

    // Offline enrichment replaced strategist — go straight to implementer
    orchestrator.set_state(OrchestratorState::Implementer);
    state.orchestrator_state = "implementer".to_string();
    transition_phase(state, "implementer");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "implementer".to_string();
    }
    agent_pool
        .spawn(
            AgentRole::Implementer,
            state.config.effort_for(AgentRole::Implementer).label(),
            state.config.model_for(AgentRole::Implementer),
        )
        .await?;
    let prompt = crate::orchestrator::prompts::implementer_prompt(&config.repo_root, &plan)?;
    agent_pool
        .turn_start(
            AgentRole::Implementer,
            &prompt,
            state.config.model_for(AgentRole::Implementer),
        )
        .await?;
    state.agent_state_mut(AgentRole::Implementer).active = true;
    persistence.append_event(&PersistenceManager::make_event(
        "phase_start",
        &plan.base,
        "implementer",
        1,
    ))?;

    Ok(())
}

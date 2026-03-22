use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use crate::agent::{AgentPool, AgentRole};
use crate::git::{GitEvent, GitManager};
use crate::orchestrator::{Orchestrator, OrchestratorEvent, OrchestratorState};
use crate::state::persistence::PersistenceManager;
use crate::state::{
    AgentPaneGroup, ConfirmAction, DetailSubTab, FocusZone, InputMode, LogLevel, Notification,
    PipelineRunState, PlanDetailTab, RunPlanStatus, RunState, VerifyEntry, VerifyStatus,
};
use crate::tui::TuiAction;

use super::*;

pub(crate) fn handle_orchestrator_event(state: &mut RunState, event: &OrchestratorEvent) {
    match event {
        OrchestratorEvent::StateChanged { to, .. } => {
            state.orchestrator_state = to.label().to_string();
        }
        OrchestratorEvent::PlanStarted { plan, index, total } => {
            state.add_log(
                "orch",
                &format!("Plan {}/{}: {}", index + 1, total, plan.base),
                LogLevel::Info,
            );
        }
        OrchestratorEvent::PlanCompleted { plan } => {
            state.add_log("orch", &format!("Completed: {}", plan.base), LogLevel::Info);
        }
        OrchestratorEvent::PlanSkipped { plan, reason } => {
            state.add_log(
                "orch",
                &format!("Skipped {}: {}", plan.base, reason),
                LogLevel::Info,
            );
        }
        OrchestratorEvent::PhaseStarted { phase, iteration } => {
            state.current_phase = phase.label().to_string();
            state.current_iteration = *iteration;
            state.phase_started = Some(Instant::now());
        }
        OrchestratorEvent::GateResult {
            gate,
            passed,
            output,
        } => {
            state.last_gate_output = output.clone();
            state.add_log(
                "gate",
                &format!("{}: {}", gate, if *passed { "PASS" } else { "FAIL" }),
                if *passed {
                    LogLevel::Info
                } else {
                    LogLevel::Error
                },
            );
        }
        OrchestratorEvent::ReviewCapHit {
            ref plan,
            iterations,
        } => {
            state.add_log(
                "orch",
                &format!(
                    "Review cap hit for {plan} after {iterations} revisions, force-committing"
                ),
                LogLevel::Warn,
            );
        }
        OrchestratorEvent::RunComplete => {
            state.complete = true;
        }
        OrchestratorEvent::Error { message } => {
            state.error = Some(message.clone());
            state.add_log("orch", message, LogLevel::Error);
        }
    }
}

/// Reconcile all plan branches: commit outstanding work, merge unmerged plans
/// to the batch branch, prune worktrees, and merge batch to staging.
/// Designed to be idempotent — safe to run multiple times.
pub(crate) fn git_reconcile(
    repo: &std::path::Path,
    batch_branch: &str,
    batch_id: &str,
    plans: &[(String, String)], // (base, num)
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut messages: Vec<String> = Vec::new();
    let mut merged_plans: Vec<String> = Vec::new();
    let mut already_reconciled: Vec<String> = Vec::new();
    let wt_mgr = crate::git::worktree::WorktreeManager::new(repo.to_path_buf());

    // Abort any in-progress merge and get onto the batch branch
    let _ = crate::git::ops::run_git(repo, &["merge", "--abort"]);
    let _ = crate::git::ops::run_git(repo, &["reset", "--hard"]);
    if let Err(e) = crate::git::ops::run_git(repo, &["checkout", batch_branch]) {
        messages.push(format!("ERROR: could not checkout {batch_branch}: {e}"));
        return (messages, merged_plans, already_reconciled);
    }

    for (base, num) in plans {
        let tag = format!("plan/{base}");
        let plan_branch = format!("codex/plan/{base}");
        let wt_path = wt_mgr.worktree_base().join(format!("plan-{base}"));

        // Already merged? (tag exists)
        let tag_exists = crate::git::ops::run_git(repo, &["tag", "-l", &tag])
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if tag_exists {
            messages.push(format!("{base}: already reconciled (tag exists)"));
            already_reconciled.push(base.clone());
            // Clean up any leftover worktree
            if wt_path.exists() {
                let pw = crate::git::worktree::PlanWorktree {
                    path: wt_path.clone(),
                    branch: plan_branch.clone(),
                    plan_base: base.clone(),
                };
                let _ = wt_mgr.cleanup_plan_worktree(&pw);
                messages.push(format!("{base}: cleaned up leftover worktree"));
            }
            // Clean up any leftover branch that's fully merged
            if crate::git::ops::run_git(repo, &["rev-parse", "--verify", &plan_branch]).is_ok() {
                let _ = crate::git::ops::run_git(repo, &["branch", "-d", &plan_branch]);
                messages.push(format!("{base}: cleaned up leftover branch"));
            }
            continue;
        }

        let branch_exists =
            crate::git::ops::run_git(repo, &["rev-parse", "--verify", &plan_branch]).is_ok();

        if !branch_exists && !wt_path.exists() {
            messages.push(format!("{base}: no branch or worktree found, skipping"));
            continue;
        }

        // Commit any uncommitted work in the worktree
        if wt_path.exists() {
            let _ = crate::git::ops::run_git(&wt_path, &["add", "-A"]);
            let status =
                crate::git::ops::run_git(&wt_path, &["status", "--porcelain"]).unwrap_or_default();
            if !status.trim().is_empty() {
                let msg = format!("plan({num}): {base} [reconciled]");
                let _ = crate::git::ops::run_git(&wt_path, &["commit", "-m", &msg]);
                messages.push(format!("{base}: committed outstanding work in worktree"));
            }
        }

        // Ensure we're on batch branch before merging
        let _ = crate::git::ops::run_git(repo, &["checkout", batch_branch]);

        // Merge
        if wt_path.exists() {
            let pw = crate::git::worktree::PlanWorktree {
                path: wt_path,
                branch: plan_branch.clone(),
                plan_base: base.clone(),
            };
            match wt_mgr.merge_plan_worktree(&pw, batch_branch) {
                Ok(()) => {
                    let _ = wt_mgr.cleanup_plan_worktree(&pw);
                    let _ = crate::git::ops::run_git(repo, &["tag", &tag]);
                    messages.push(format!("{base}: merged via worktree and tagged"));
                    merged_plans.push(base.clone());
                }
                Err(e) => {
                    let _ = crate::git::ops::run_git(repo, &["merge", "--abort"]);
                    messages.push(format!("{base}: ERROR worktree merge failed: {e}"));
                }
            }
        } else if branch_exists {
            let merge_msg = format!("Merge branch '{plan_branch}' into {batch_branch}");
            match crate::git::ops::run_git(
                repo,
                &["merge", "--no-ff", "-m", &merge_msg, &plan_branch],
            ) {
                Ok(_) => {
                    let _ = crate::git::ops::run_git(repo, &["branch", "-d", &plan_branch]);
                    let _ = crate::git::ops::run_git(repo, &["tag", &tag]);
                    messages.push(format!("{base}: merged branch and tagged"));
                    merged_plans.push(base.clone());
                }
                Err(e) => {
                    let _ = crate::git::ops::run_git(repo, &["merge", "--abort"]);
                    messages.push(format!("{base}: ERROR branch merge failed: {e}"));
                }
            }
        }
    }

    // Prune stale worktrees
    let _ = crate::git::ops::run_git(repo, &["worktree", "prune"]);
    messages.push("Pruned stale worktrees".to_string());

    // Ensure we're on batch branch
    let _ = crate::git::ops::run_git(repo, &["checkout", batch_branch]);

    // Merge batch to staging (idempotent: check if already up-to-date)
    let staging = format!("staging/{batch_id}");
    let staging_exists =
        crate::git::ops::run_git(repo, &["rev-parse", "--verify", &staging]).is_ok();
    if staging_exists {
        // Check if batch is already ancestor of staging
        let already_merged = crate::git::ops::run_git(
            repo,
            &["merge-base", "--is-ancestor", batch_branch, &staging],
        )
        .is_ok();
        if already_merged {
            messages.push(format!("Staging {staging} already up-to-date"));
        } else {
            let _ = crate::git::ops::run_git(repo, &["checkout", &staging]);
            let merge_msg = format!("merge(batch): {batch_branch}");
            match crate::git::ops::run_git(
                repo,
                &["merge", "--no-ff", "-m", &merge_msg, batch_branch],
            ) {
                Ok(_) => messages.push(format!("Merged batch to staging: {staging}")),
                Err(e) => {
                    let _ = crate::git::ops::run_git(repo, &["merge", "--abort"]);
                    messages.push(format!("ERROR merging to staging: {e}"));
                }
            }
            let _ = crate::git::ops::run_git(repo, &["checkout", batch_branch]);
        }
    } else {
        // Create staging from main, then merge batch
        let _ = crate::git::ops::run_git(repo, &["checkout", "-b", &staging, "main"]);
        let merge_msg = format!("merge(batch): {batch_branch}");
        match crate::git::ops::run_git(repo, &["merge", "--no-ff", "-m", &merge_msg, batch_branch])
        {
            Ok(_) => messages.push(format!("Created staging and merged batch: {staging}")),
            Err(e) => {
                let _ = crate::git::ops::run_git(repo, &["merge", "--abort"]);
                messages.push(format!("ERROR creating staging: {e}"));
            }
        }
        let _ = crate::git::ops::run_git(repo, &["checkout", batch_branch]);
    }

    (messages, merged_plans, already_reconciled)
}

pub(crate) fn handle_git_event(state: &mut RunState, event: &GitEvent) {
    match event {
        GitEvent::BranchCreated { name } => {
            state.add_log("git", &format!("Branch: {name}"), LogLevel::Info);
            state.git_branch = name.clone();
        }
        GitEvent::Committed { hash, message } => {
            state.add_log("git", &format!("{hash} {message}"), LogLevel::Info);
        }
        GitEvent::Merged { source, target } => {
            state.add_log(
                "git",
                &format!("Merged {source} → {target}"),
                LogLevel::Info,
            );
        }
        GitEvent::Tagged { name } => {
            state.add_log("git", &format!("Tag: {name}"), LogLevel::Info);
        }
        GitEvent::Error { message } => {
            state.add_log("git", message, LogLevel::Error);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_tui_action(
    state: &mut RunState,
    agent_pool: &mut AgentPool,
    orchestrator: &mut Orchestrator,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    action: TuiAction,
) -> Result<bool> {
    match action {
        TuiAction::Quit => return Ok(true),
        TuiAction::SwitchTab(idx) => {
            if idx < 6 {
                let prev = state.active_tab;
                state.active_tab = idx;
                // Clear pipeline header when leaving Plans tab
                if prev == 1 && idx != 1 {
                    state.pipeline_header_selected = false;
                }
                // Dashboard→Plans: convert global selected_plan_idx to within-wave
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
                // Plans→Dashboard: convert within-wave back to global index
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
            // Keep scroll in range
            if state.selected_plan_idx < state.plan_scroll_offset {
                state.plan_scroll_offset = state.selected_plan_idx;
            }
            // Load selected plan's task checklist if not cached
            if let Some(base) = state
                .plans
                .get(state.selected_plan_idx)
                .map(|p| p.base.clone())
            {
                state.ensure_checklist_cached(&base);
            }
        }
        TuiAction::SelectPlanDown => {
            if state.selected_plan_idx + 1 < state.plans.len() {
                state.selected_plan_idx += 1;
            }
            let visible = (state.terminal_height.min(12).saturating_sub(4)) as usize;
            if visible > 0 && state.selected_plan_idx >= state.plan_scroll_offset + visible {
                state.plan_scroll_offset = state.selected_plan_idx.saturating_sub(visible) + 1;
            }
            // Load selected plan's task checklist if not cached
            if let Some(base) = state
                .plans
                .get(state.selected_plan_idx)
                .map(|p| p.base.clone())
            {
                state.ensure_checklist_cached(&base);
            }
        }
        TuiAction::ScrollLogUp => {
            state.log_scroll = state.log_scroll.saturating_add(10);
        }
        TuiAction::ScrollLogDown => {
            state.log_scroll = state.log_scroll.saturating_sub(10);
        }
        TuiAction::SwitchAgentTab(idx) => {
            state.manual_agent_tab = true;
            if idx == usize::MAX {
                state.selected_agent_tab = (state.selected_agent_tab + 1) % 7;
            } else if idx < 7 {
                state.selected_agent_tab = idx;
            }
        }
        TuiAction::ApproveCommand => {
            if let Some(approval) = state.pending_approval.take() {
                agent_pool
                    .respond_approval(approval.role, &approval.approval_id, true)
                    .await?;
            }
        }
        TuiAction::ApproveAll => {
            // E4: Approve all pending approvals (sequential mode has only one)
            if let Some(approval) = state.pending_approval.take() {
                agent_pool
                    .respond_approval(approval.role, &approval.approval_id, true)
                    .await?;
            }
        }
        TuiAction::RejectCommand => {
            if let Some(approval) = state.pending_approval.take() {
                agent_pool
                    .respond_approval(approval.role, &approval.approval_id, false)
                    .await?;
            }
        }
        TuiAction::StartInject => {
            state.input_mode = InputMode::Inject;
            state.message_input.clear();
            // Resolve the visually-selected agent and store its label for modal display
            if let Some((role, instance_id, _task)) = state.resolve_agent_list_cursor() {
                state.steer_target = Some(format!("{role}:{instance_id}"));
            } else {
                state.steer_target = None;
            }
        }
        TuiAction::SubmitInject(msg) => {
            state.input_mode = InputMode::Normal;
            state.message_input.clear();
            if !msg.is_empty() {
                // In parallel mode, injections are routed through the conductor
                // Signal that an injection is pending (parallel.rs will handle it in next turn)
                if !state.parallel_agents.is_empty() {
                    state.pending_inject = Some(crate::state::PendingInject {
                        message: msg.clone(),
                        target_role: None, // Will be determined by parallel.rs logic
                        target_instance_id: None,
                    });
                    state.add_log("inject", &format!("Supervisor: {msg}"), LogLevel::Info);
                } else {
                    // Sequential mode: inject directly
                    let roles = [
                        AgentRole::Strategist,
                        AgentRole::Implementer,
                        AgentRole::Architect,
                        AgentRole::Auditor,
                        AgentRole::Scribe,
                        AgentRole::Critic,
                        AgentRole::Conductor,
                    ];
                    let role = roles
                        .get(state.selected_agent_tab)
                        .copied()
                        .unwrap_or(AgentRole::Implementer);
                    if agent_pool.is_spawned(role) {
                        let _ = agent_pool.turn_interrupt(role).await;
                        let inject_msg = format!(
                            "Supervisor message: {msg}\n\nContinue from where you left off."
                        );
                        let _ = agent_pool.turn_start(role, &inject_msg, None).await;
                        state.agent_state_mut(role).active = true;
                        state.add_log("inject", &format!("[{role}] {msg}"), LogLevel::Info);
                        // Echo inject into agent output panel
                        let echo = format!(
                            "\n--- Supervisor inject ---\n{msg}\n-------------------------\n"
                        );
                        state.agent_state_mut(role).output.push_str(&echo);
                    } else {
                        state.add_log(
                            "inject",
                            &format!("[{role}] Agent not active"),
                            LogLevel::Warn,
                        );
                    }
                }
            }
        }
        TuiAction::CancelInject => {
            state.input_mode = InputMode::Normal;
            state.message_input.clear();
        }
        TuiAction::InputChar(c) => match state.input_mode {
            InputMode::Filter => {
                state.filter_text.push(c);
            }
            _ => {
                state.message_input.push(c);
            }
        },
        TuiAction::InputBackspace => match state.input_mode {
            InputMode::Filter => {
                state.filter_text.pop();
            }
            _ => {
                state.message_input.pop();
            }
        },
        TuiAction::FocusNext => {
            let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
            state.focus = match state.focus {
                FocusZone::Plans => FocusZone::Tasks,
                FocusZone::Tasks => FocusZone::AgentOutput,
                FocusZone::AgentOutput => {
                    if has_cmd {
                        FocusZone::CommandOutput
                    } else {
                        FocusZone::Plans
                    }
                }
                FocusZone::CommandOutput => FocusZone::Plans,
            };
        }
        TuiAction::FocusPrev => {
            let has_cmd = !state.gate_running.is_empty() || !state.command_output.is_empty();
            state.focus = match state.focus {
                FocusZone::Plans => {
                    if has_cmd {
                        FocusZone::CommandOutput
                    } else {
                        FocusZone::AgentOutput
                    }
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
            let max = state
                .task_checklist
                .as_ref()
                .map(|c| c.tasks.len())
                .unwrap_or(0);
            if state.task_scroll + 1 < max {
                state.task_scroll += 1;
            }
        }
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
        TuiAction::ScrollAgentUp => {
            let total = current_agent_line_count(state);
            let page = 10;
            state.agent_scroll = Some(match state.agent_scroll {
                None => total.saturating_sub(page),
                Some(n) => n.saturating_sub(page),
            });
        }
        TuiAction::ScrollAgentDown => {
            if let Some(n) = state.agent_scroll {
                let total = current_agent_line_count(state);
                let new = n + 10;
                if new >= total.saturating_sub(20) {
                    state.agent_scroll = None; // back to auto-scroll
                } else {
                    state.agent_scroll = Some(new);
                }
            }
        }
        TuiAction::ScrollAgentEnd => {
            state.agent_scroll = None;
        }
        TuiAction::ShowPlanDetail => {
            if let Some(repo_root) = &state.repo_root {
                if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                    let path = repo_root.join(format!("plans/{}.md", plan.base));
                    state.plan_detail_content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|_| "Plan file not found.".to_string());
                    state.plan_detail_scroll = 0;
                    state.show_plan_detail = true;

                    // Load summary for completed plans
                    let is_completed = matches!(
                        plan.status,
                        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                    );
                    if is_completed {
                        state.plan_summary_content =
                            crate::orchestrator::context::read_summary(repo_root, &plan.num)
                                .unwrap_or(None)
                                .unwrap_or_default();
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
                PlanDetailTab::Summary => {
                    state.plan_summary_scroll += accel;
                }
                PlanDetailTab::PlanDetails => {
                    state.plan_detail_scroll += accel;
                }
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
                PlanDetailTab::Summary => {
                    state.plan_summary_scroll += page;
                }
                PlanDetailTab::PlanDetails => {
                    state.plan_detail_scroll += page;
                }
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
        TuiAction::DismissNotification => {
            state.notifications.pop();
        }
        TuiAction::ShowHelp => {
            state.show_help = !state.show_help;
        }
        TuiAction::ShowWaveOverview => {
            state.show_wave_overview = !state.show_wave_overview;
            state.show_agent_pool_modal = false;
        }
        TuiAction::ShowAgentPoolModal => {
            state.show_agent_pool_modal = !state.show_agent_pool_modal;
            state.show_wave_overview = false;
        }
        TuiAction::RestartPhase => {
            restart_current_phase(state, orchestrator, agent_pool, persistence, config).await?;
        }
        TuiAction::RestartPlan => {
            restart_plan(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
            )
            .await?;
        }
        TuiAction::ScrollDiffUp => {
            if state.focus == FocusZone::CommandOutput {
                // Scroll command output panel
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
            } else {
                if let Some(n) = state.diff_scroll {
                    let total = state.branch_diff.lines().count();
                    let new = n + 10;
                    if new >= total.saturating_sub(20) {
                        state.diff_scroll = None;
                    } else {
                        state.diff_scroll = Some(new);
                    }
                }
            }
        }
        TuiAction::ConfigUp => {
            state.config.selected_row = state.config.selected_row.saturating_sub(1);
        }
        TuiAction::ConfigDown => {
            let max = state.config.row_count().saturating_sub(1);
            if state.config.selected_row < max {
                state.config.selected_row += 1;
            }
        }
        TuiAction::ConfigLeft => {
            handle_config_cycle(state, false);
        }
        TuiAction::ConfigRight => {
            handle_config_cycle(state, true);
        }
        TuiAction::ConfigSelect => {
            handle_config_select(state, config);
            // Hot reload: kill agents whose model changed since last Apply
            for role in state.pending_agent_kills.drain(..).collect::<Vec<_>>() {
                tracing::info!(
                    "Hot reload: killing {} (model changed to {})",
                    role,
                    state.config.model_for(role).unwrap_or("?")
                );
                agent_pool.kill(role).await;
                state.add_log(
                    "config",
                    &format!(
                        "Reloaded {}: model={}",
                        role,
                        state.config.model_for(role).unwrap_or("?")
                    ),
                    LogLevel::Info,
                );
            }
        }
        TuiAction::ForceAdvance => {
            force_advance(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
            )
            .await?;
        }
        TuiAction::ResetPlanState => {
            reset_plan_state(state, orchestrator, persistence, config);
        }
        TuiAction::ReverifyPlan => {
            reverify_plan(state, orchestrator, persistence, config).await?;
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
            // Switch to dashboard tab since sub-tabs only exist there
            if state.active_tab != 0 {
                state.active_tab = 0;
            }
        }
        TuiAction::StartFilter => {
            state.input_mode = InputMode::Filter;
            state.filter_text.clear();
            state.filter_active = true;
        }
        TuiAction::AcceptFilter => {
            state.input_mode = InputMode::Normal;
            state.filter_active = false;
            // filter_text stays — keeps the filter applied
        }
        TuiAction::CancelFilter => {
            state.input_mode = InputMode::Normal;
            state.filter_text.clear();
            state.filter_active = false;
        }
        TuiAction::ShowTaskDetail => {
            state.show_task_detail = true;
            state.task_detail_scroll = 0;
        }
        TuiAction::CloseTaskDetail => {
            state.show_task_detail = false;
        }
        TuiAction::ScrollTaskDetailUp => {
            state.task_detail_scroll = state.task_detail_scroll.saturating_sub(1);
        }
        TuiAction::ScrollTaskDetailDown => {
            state.task_detail_scroll += 1;
        }
        TuiAction::CollapseExpand => {
            // Toggle wave expansion for the wave containing the selected plan
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
        TuiAction::NavigateUp => {
            // Context-dependent on active tab
            match state.active_tab {
                1 => {
                    // Plans browser: navigate within-wave plan list
                    if state.pipeline_header_selected {
                        // already at top, nowhere to go
                    } else if state.selected_plan_idx > 0 {
                        state.selected_plan_idx -= 1;
                    } else if state.selected_wave_idx > 0 {
                        // Wrap to previous wave's last plan
                        state.selected_wave_idx -= 1;
                        let count = state
                            .execution_waves
                            .get(state.selected_wave_idx)
                            .map(|(_, p)| p.len())
                            .unwrap_or(1);
                        state.selected_plan_idx = count.saturating_sub(1);
                    } else {
                        // Already at wave 0, plan 0 — enter pipeline header
                        state.pipeline_header_selected = true;
                    }
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
                    // Plans browser: navigate within-wave plan list
                    if state.pipeline_header_selected {
                        state.pipeline_header_selected = false;
                        // selection stays at wave 0, plan 0
                    } else {
                        let wave_plan_count = state
                            .execution_waves
                            .get(state.selected_wave_idx)
                            .map(|(_, p)| p.len())
                            .unwrap_or(state.plans.len());
                        if state.selected_plan_idx + 1 < wave_plan_count {
                            state.selected_plan_idx += 1;
                        } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                            // Wrap to next wave
                            state.selected_wave_idx += 1;
                            state.selected_plan_idx = 0;
                        }
                    }
                }
                2 => {
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
                    let max = state.git_branch_tree.len().saturating_sub(1);
                    if state.git_branch_cursor < max {
                        state.git_branch_cursor += 1;
                    }
                }
                _ => {}
            }
        }
        TuiAction::NavigatePageUp => {
            if state.active_tab == 1 {
                for _ in 0..10 {
                    if state.selected_plan_idx > 0 {
                        state.selected_plan_idx -= 1;
                    } else if state.selected_wave_idx > 0 {
                        state.selected_wave_idx -= 1;
                        let count = state
                            .execution_waves
                            .get(state.selected_wave_idx)
                            .map(|(_, p)| p.len())
                            .unwrap_or(1);
                        state.selected_plan_idx = count.saturating_sub(1);
                    } else {
                        break;
                    }
                }
            }
        }
        TuiAction::NavigatePageDown => {
            if state.active_tab == 1 {
                for _ in 0..10 {
                    let wave_plan_count = state
                        .execution_waves
                        .get(state.selected_wave_idx)
                        .map(|(_, p)| p.len())
                        .unwrap_or(state.plans.len());
                    if state.selected_plan_idx + 1 < wave_plan_count {
                        state.selected_plan_idx += 1;
                    } else if state.selected_wave_idx + 1 < state.execution_waves.len() {
                        state.selected_wave_idx += 1;
                        state.selected_plan_idx = 0;
                    } else {
                        break;
                    }
                }
            }
        }
        TuiAction::WaveNext => {
            state.pipeline_header_selected = false;
            if state.selected_wave_idx + 1 < state.execution_waves.len() {
                state.selected_wave_idx += 1;
                state.selected_plan_idx = 0;
            }
        }
        TuiAction::WavePrev => {
            state.pipeline_header_selected = false;
            if state.selected_wave_idx > 0 {
                state.selected_wave_idx -= 1;
                state.selected_plan_idx = 0;
            }
        }
        TuiAction::DrillIn => {
            // Plans browser: expand wave or show plan detail
            if state.active_tab == 1 {
                // Toggle wave expansion, or show plan detail
                if !state.execution_waves.is_empty() {
                    if let Some(plan) = state.plans.get(state.selected_plan_idx) {
                        let plan_base = plan.base.clone();
                        for (idx, (_, wave_plans)) in state.execution_waves.iter().enumerate() {
                            if wave_plans.contains(&plan_base) {
                                state.wave_expanded.insert(idx);
                                break;
                            }
                        }
                    }
                }
            }
        }
        TuiAction::DrillOut => {
            // Plans browser: collapse wave or go back
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
        TuiAction::RequestConfirm(confirm_action) => {
            state.pending_confirm = Some(confirm_action);
            state.input_mode = InputMode::Confirm;
        }
        TuiAction::ConfirmYes => {
            if let Some(confirmed) = state.pending_confirm.take() {
                state.input_mode = InputMode::Normal;
                match confirmed {
                    ConfirmAction::RestartAllPlans => {
                        restart_plan(
                            state,
                            orchestrator,
                            agent_pool,
                            git_manager,
                            persistence,
                            config,
                            batch_branch,
                            run_id,
                            started_at,
                        )
                        .await?;
                    }
                    ConfirmAction::RestartPhase => {
                        restart_current_phase(state, orchestrator, agent_pool, persistence, config)
                            .await?;
                    }
                    ConfirmAction::ResetSelectedPlan(_) => {
                        reset_plan_state(state, orchestrator, persistence, config);
                    }
                    ConfirmAction::ForceAdvance(_) => {
                        force_advance(
                            state,
                            orchestrator,
                            agent_pool,
                            git_manager,
                            persistence,
                            config,
                            batch_branch,
                            run_id,
                            started_at,
                        )
                        .await?;
                    }
                    ConfirmAction::ReverifyPlan(_) => {
                        reverify_plan(state, orchestrator, persistence, config).await?;
                    }
                    ConfirmAction::GitReconcile => {
                        if !state.git_reconcile_in_progress {
                            state.git_reconcile_in_progress = true;
                            state.add_log("git", "Starting git reconcile...", LogLevel::Info);
                            let repo = config.repo_root.clone();
                            let batch = batch_branch.to_string();
                            let bid = config.batch_id.clone();
                            let plan_info: Vec<(String, String)> = state
                                .plans
                                .iter()
                                .map(|p| (p.base.clone(), p.num.clone()))
                                .collect();
                            let (messages, merged_plans, _already_reconciled) =
                                tokio::task::spawn_blocking(move || {
                                    git_reconcile(&repo, &batch, &bid, &plan_info)
                                })
                                .await?;
                            state.git_reconcile_in_progress = false;
                            for msg in &messages {
                                let level = if msg.contains("ERROR") {
                                    LogLevel::Error
                                } else {
                                    LogLevel::Info
                                };
                                state.add_log("reconcile", msg, level);
                            }
                            for plan in &merged_plans {
                                if let Some(entry) =
                                    state.plans.iter_mut().find(|p| &p.base == plan)
                                {
                                    entry.status = RunPlanStatus::Completed;
                                    entry.phase = "complete".to_string();
                                }
                            }
                        }
                    }
                    ConfirmAction::MergeBatchToMain {
                        batch_branch: ref bb,
                        ..
                    } => {
                        let bb = bb.clone();
                        state.add_log("git", &format!("Merging {bb} → main…"), LogLevel::Info);
                        let repo = config.repo_root.clone();
                        let gm_repo = repo.clone();
                        let bb2 = bb.clone();
                        let result = tokio::task::spawn_blocking(move || {
                            let event_tx = tokio::sync::mpsc::unbounded_channel::<GitEvent>().0;
                            let gm = crate::git::GitManager::new(gm_repo, event_tx);
                            gm.merge_batch_to_main(&bb2)
                        })
                        .await?;
                        match result {
                            Ok(hash) => {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let short_hash = hash[..hash.len().min(7)].to_string();
                                // Mark all completed plans as MergedToMain
                                let merged_bases: Vec<String> = state
                                    .plans
                                    .iter()
                                    .filter(|p| {
                                        matches!(
                                            p.status,
                                            RunPlanStatus::Completed
                                                | RunPlanStatus::CompletedPrior
                                        )
                                    })
                                    .map(|p| p.base.clone())
                                    .collect();
                                for plan in state.plans.iter_mut() {
                                    if matches!(
                                        plan.status,
                                        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                                    ) {
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
                                state.add_log(
                                    "git",
                                    &format!("✓ Merged {bb} → main @ {short_hash}"),
                                    LogLevel::Info,
                                );
                                state.notifications.push(crate::state::Notification {
                                    message: format!("⬆ main ← {bb} @ {short_hash}"),
                                    created: std::time::Instant::now(),
                                    ttl_secs: 10,
                                    level: LogLevel::Info,
                                });
                            }
                            Err(e) => {
                                state.add_log(
                                    "git",
                                    &format!("Merge failed: {e}"),
                                    LogLevel::Error,
                                );
                            }
                        }
                    }
                    ConfirmAction::IngestTask { plan_num, task_id } => {
                        if let Some(repo_root) = &state.repo_root.clone() {
                            let path = repo_root
                                .join(format!("plans/context/tasks/{plan_num}-tasks.toml"));
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(mut val) = toml::from_str::<toml::Value>(&content) {
                                    if let Some(tasks) =
                                        val.get_mut("tasks").and_then(|v| v.as_array_mut())
                                    {
                                        for task in tasks.iter_mut() {
                                            if task.get("id").and_then(|v| v.as_str())
                                                == Some(task_id.as_str())
                                            {
                                                if let Some(tbl) = task.as_table_mut() {
                                                    tbl.insert(
                                                        "status".to_string(),
                                                        toml::Value::String("pending".to_string()),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    if let Ok(out) = toml::to_string(&val) {
                                        let _ = std::fs::write(&path, out);
                                    }
                                }
                            }
                            state.add_log(
                                "orch",
                                &format!(
                                    "Ingested task {task_id} (plan {plan_num}) — reset to pending"
                                ),
                                LogLevel::Info,
                            );
                        }
                    }
                }
            }
        }
        TuiAction::ConfirmNo => {
            state.pending_confirm = None;
            state.input_mode = InputMode::Normal;
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
            let total = picker_task_count(state);
            if state.task_picker_cursor + 1 < total {
                state.task_picker_cursor += 1;
            }
        }
        TuiAction::TaskPickerConfirm => {
            if let Some((plan_num, task_id)) = picker_task_pick(state) {
                state.show_task_picker = false;
                return Box::pin(handle_tui_action(
                    state,
                    agent_pool,
                    orchestrator,
                    git_manager,
                    persistence,
                    config,
                    batch_branch,
                    run_id,
                    started_at,
                    TuiAction::RequestConfirm(ConfirmAction::IngestTask { plan_num, task_id }),
                ))
                .await;
            }
        }
        TuiAction::TogglePause => {
            state.pipeline_run_state = match state.pipeline_run_state {
                PipelineRunState::Running => {
                    state.add_log("orch", "Pipeline paused", LogLevel::Warn);
                    PipelineRunState::Paused
                }
                PipelineRunState::Paused => {
                    state.add_log("orch", "Pipeline resumed", LogLevel::Info);
                    PipelineRunState::Running
                }
            };
        }
        TuiAction::PrepareMergeBatchToMain => {
            // Gather info from current state, then raise the confirm dialog.
            let plan_count = state
                .plans
                .iter()
                .filter(|p| {
                    matches!(
                        p.status,
                        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                    )
                })
                .count();
            let failed_count = state
                .plans
                .iter()
                .filter(|p| matches!(p.status, RunPlanStatus::Failed))
                .count();
            let last_commit = git_manager
                .log_oneline(1)
                .ok()
                .and_then(|s| {
                    s.split_whitespace()
                        .next()
                        .map(|h| h[..h.len().min(7)].to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());
            let action = ConfirmAction::MergeBatchToMain {
                batch_branch: batch_branch.to_string(),
                plan_count,
                failed_count,
                last_commit,
            };
            state.pending_confirm = Some(action);
            state.input_mode = InputMode::Confirm;
        }
        TuiAction::None => {}
    }
    Ok(false)
}

/// Total number of tasks shown in the task picker modal.
fn picker_task_count(state: &RunState) -> usize {
    let mut total = 0;
    if let Some(ref cl) = state.task_checklist {
        total += cl.tasks.len();
    }
    for cl in state.plan_task_cache.values() {
        total += cl.tasks.len();
    }
    total
}

/// Return the `(plan_num, task_id)` at the current picker cursor, if any.
fn picker_task_pick(state: &RunState) -> Option<(String, String)> {
    let mut items: Vec<(String, String)> = Vec::new();
    if let Some(ref cl) = state.task_checklist {
        for t in &cl.tasks {
            items.push((cl.plan_num.clone(), t.id.clone()));
        }
    }
    for (pnum, cl) in &state.plan_task_cache {
        for t in &cl.tasks {
            items.push((pnum.clone(), t.id.clone()));
        }
    }
    let cursor = state.task_picker_cursor.min(items.len().saturating_sub(1));
    items.into_iter().nth(cursor)
}

/// Cycle a config value left (prev) or right (next) for the currently selected row.
pub(crate) fn handle_config_cycle(state: &mut RunState, forward: bool) {
    let row = state.config.selected_row;
    let models = state.config.available_models.clone();
    let ctx_options = [100u32, 150, 200, 300, 500, 1000];
    let (s1, s2, s3, _s4, ..) = crate::state::config::ConfigState::layout();
    let n = crate::agent::AgentRole::ALL_AGENTS.len();

    // Update active_section from row position
    let (section, _) = state.config.section_of_row(row);
    state.config.active_section = section;

    // Section 0: Backend Defaults (rows 0..s1)
    if row < s1 {
        match row {
            // Row 0: codex default — only cycle through Codex models
            0 => {
                let codex: Vec<_> = models
                    .iter()
                    .filter(|m| {
                        crate::agent::AgentBackend::from_model(&m.slug)
                            == crate::agent::AgentBackend::Codex
                    })
                    .collect();
                if codex.is_empty() {
                    return;
                }
                let current = crate::state::config::normalize_model_slug(
                    &state.config.codex_default_model.clone(),
                    &models,
                );
                let idx = codex.iter().position(|m| m.slug == current).unwrap_or(0);
                let new_idx = cycle_idx(idx, codex.len(), forward);
                if let Some(model) = codex.get(new_idx) {
                    state.config.codex_default_model = model.slug.clone();
                }
            }
            // Row 1: cursor default — only cycle through Cursor models
            1 => {
                let cursor: Vec<_> = models
                    .iter()
                    .filter(|m| {
                        crate::agent::AgentBackend::from_model(&m.slug)
                            == crate::agent::AgentBackend::Cursor
                    })
                    .collect();
                if cursor.is_empty() {
                    return;
                }
                let current = crate::state::config::normalize_model_slug(
                    &state.config.cursor_default_model.clone(),
                    &models,
                );
                let idx = cursor.iter().position(|m| m.slug == current).unwrap_or(0);
                let new_idx = cycle_idx(idx, cursor.len(), forward);
                if let Some(model) = cursor.get(new_idx) {
                    state.config.cursor_default_model = model.slug.clone();
                }
            }
            // Row 2: claude default — only cycle through Claude models
            2 => {
                let claude: Vec<_> = models
                    .iter()
                    .filter(|m| {
                        crate::agent::AgentBackend::from_model(&m.slug)
                            == crate::agent::AgentBackend::Claude
                    })
                    .collect();
                if claude.is_empty() {
                    return;
                }
                let current = crate::state::config::normalize_model_slug(
                    &state.config.claude_default_model.clone(),
                    &models,
                );
                let idx = claude.iter().position(|m| m.slug == current).unwrap_or(0);
                let new_idx = cycle_idx(idx, claude.len(), forward);
                if let Some(model) = claude.get(new_idx) {
                    state.config.claude_default_model = model.slug.clone();
                }
            }
            // Row 3: conductor model
            3 => {
                let current = crate::state::config::normalize_model_slug(
                    &state.config.conductor_model.clone(),
                    &models,
                );
                let idx = models.iter().position(|m| m.slug == current).unwrap_or(0);
                let new_idx = cycle_idx(idx, models.len(), forward);
                if let Some(model) = models.get(new_idx) {
                    state.config.conductor_model = model.slug.clone();
                }
            }
            // Row 4: fallback model — cycle through all models + None
            4 => {
                let mut slugs: Vec<Option<String>> = vec![None];
                slugs.extend(models.iter().map(|m| Some(m.slug.clone())));
                let current_idx = match &state.config.fallback_model {
                    None => 0,
                    Some(current) => slugs
                        .iter()
                        .position(|o| o.as_deref() == Some(current))
                        .unwrap_or(0),
                };
                let new_idx = cycle_idx(current_idx, slugs.len(), forward);
                state.config.fallback_model = slugs[new_idx].clone();
            }
            _ => {}
        }
    }
    // Section 1: Per-role model overrides (rows s1..s2)
    else if row < s2 {
        let mut sorted_models = models.clone();
        sorted_models.sort_by_key(|m| match crate::agent::AgentBackend::from_model(&m.slug) {
            crate::agent::AgentBackend::Claude => 0u8,
            crate::agent::AgentBackend::Cursor => 1,
            crate::agent::AgentBackend::Codex => 2,
        });
        let role = if row == s1 {
            crate::agent::AgentRole::Conductor
        } else {
            crate::agent::AgentRole::ALL_AGENTS[row - s1 - 1]
        };
        let key = role.label().to_string();
        let default_model = state.config.model_for(role).unwrap_or("").to_string();
        let current = state
            .config
            .role_models
            .get(&key)
            .cloned()
            .unwrap_or(default_model);
        let idx = sorted_models
            .iter()
            .position(|m| m.slug == current)
            .unwrap_or(0);
        let new_idx = cycle_idx(idx, sorted_models.len(), forward);
        if let Some(model) = sorted_models.get(new_idx) {
            state.config.role_models.insert(key, model.slug.clone());
        }
    }
    // Section 2: Context & Effort (rows s2..s3)
    else if row < s3 {
        let offset = row - s2;
        if offset == 0 {
            // Global context limit
            let idx = ctx_options
                .iter()
                .position(|&v| v == state.config.context_limit_k)
                .unwrap_or(2);
            let new_idx = cycle_idx(idx, ctx_options.len(), forward);
            state.config.context_limit_k = ctx_options[new_idx];
            state.context_limit = (state.config.context_limit_k as u64) * 1000;
        } else if offset <= n {
            // Per-role context
            let role = crate::agent::AgentRole::ALL_AGENTS[offset - 1];
            let key = role.label().to_string();
            let current = state
                .config
                .role_context_k
                .get(&key)
                .copied()
                .unwrap_or(state.config.context_limit_k);
            let idx = ctx_options.iter().position(|&v| v == current).unwrap_or(2);
            let new_idx = cycle_idx(idx, ctx_options.len(), forward);
            state
                .config
                .role_context_k
                .insert(key, ctx_options[new_idx]);
        } else {
            // Reasoning effort
            state.config.default_effort = if forward {
                state.config.default_effort.cycle_next()
            } else {
                state.config.default_effort.cycle_prev()
            };
        }
    }
    // Section 4: Execution cycleable rows
    else {
        let (_s1, _s2, _s3, s4, ..) = crate::state::config::ConfigState::layout();
        let offset = row.saturating_sub(s4);
        match offset {
            1 => {
                // max agents
                if forward {
                    state.config.max_agents = (state.config.max_agents + 1).min(32);
                } else {
                    state.config.max_agents = state.config.max_agents.saturating_sub(1).max(1);
                }
            }
            5 => {
                // max iterations
                if forward {
                    state.config.max_iterations = (state.config.max_iterations + 1).min(20);
                } else {
                    state.config.max_iterations =
                        state.config.max_iterations.saturating_sub(1).max(1);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn cycle_idx(current: usize, len: usize, forward: bool) -> usize {
    util::cycle_idx(current, len, forward)
}

/// Handle Enter/Space on a config row (toggles and apply).
pub(crate) fn handle_config_select(state: &mut RunState, config: &AppConfig) {
    let row = state.config.selected_row;
    let (s1, s2, s3, s4, apply, _total) = crate::state::config::ConfigState::layout();

    // Update active_section for all row types
    state.config.active_section = state.config.section_of_row(row).0;

    if row == apply {
        // Apply button
        let prev_models = crate::state::config::ConfigState::load(&config.repo_root)
            .map(|c| c.snapshot_models())
            .unwrap_or_default();
        state.context_limit = (state.config.context_limit_k as u64) * 1000;
        match state.config.save(&config.repo_root) {
            Ok(()) => {
                let changed = state.config.changed_roles(&prev_models);
                if !changed.is_empty() {
                    tracing::info!(
                        "Config apply: {} role(s) have new models — queuing kills: {}",
                        changed.len(),
                        changed
                            .iter()
                            .map(|r| r.label())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    state.pending_agent_kills.extend(changed.iter().copied());
                }
                state.add_log(
                    "config",
                    &format!(
                        "Config saved: codex={} cursor={} claude={} conductor={}",
                        state.config.codex_default_model,
                        state.config.cursor_default_model,
                        state.config.claude_default_model,
                        state.config.conductor_model,
                    ),
                    LogLevel::Info,
                );
            }
            Err(e) => {
                state.add_log("config", &format!("Save failed: {e}"), LogLevel::Error);
            }
        }
    } else if row >= s3 && row < s4 {
        // Section 3: Agent Toggles
        match row - s3 {
            0 => state.config.architect_enabled = !state.config.architect_enabled,
            1 => state.config.auditor_enabled = !state.config.auditor_enabled,
            2 => state.config.scribe_enabled = !state.config.scribe_enabled,
            3 => state.config.critic_enabled = !state.config.critic_enabled,
            _ => {}
        }
    } else if row >= s4 && row < apply {
        // Section 4: Execution
        match row - s4 {
            0 => state.config.parallel_enabled = !state.config.parallel_enabled,
            // 1 = max_agents (cycleable, fall through)
            2 => state.config.auto_advance_batch = !state.config.auto_advance_batch,
            3 => state.config.auto_advance_plan = !state.config.auto_advance_plan,
            4 => state.config.pre_plan = !state.config.pre_plan,
            // 5 = max_iterations (cycleable, fall through)
            6 => state.config.skip_tests = !state.config.skip_tests,
            7 => state.config.clippy_enabled = !state.config.clippy_enabled,
            _ => handle_config_cycle(state, true),
        }
    } else {
        // Sections 0-2: all cycleable
        handle_config_cycle(state, true);
    }
}

/// Force commit current state and advance to the next plan.
/// Useful when stuck in iteration loops — skips remaining reviews.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn force_advance(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    // Kill active agents
    for (role, agent) in state.agents.iter_mut() {
        if agent.active {
            agent_pool.kill(*role).await;
            agent.active = false;
        }
    }

    state.add_log(
        "orch",
        &format!("Force advancing past plan {}", plan.base),
        LogLevel::Warn,
    );

    commit_and_advance(
        state,
        orchestrator,
        agent_pool,
        git_manager,
        persistence,
        config,
        batch_branch,
        run_id,
        started_at,
        &plan,
    )
    .await
}

/// Reset plan state files so plans can be re-run.
/// Deletes git tags, events, and context files for the selected plan.
pub(crate) fn reset_plan_state(
    state: &mut RunState,
    _orchestrator: &mut Orchestrator,
    _persistence: &PersistenceManager,
    config: &AppConfig,
) {
    if let Some(plan) = state.plans.get(state.selected_plan_idx) {
        let base = plan.base.clone();
        let tag = format!("plan/{base}");

        // Delete the git tag
        let _ = std::process::Command::new("git")
            .args(["tag", "-d", &tag])
            .current_dir(&config.repo_root)
            .output();

        // Remove this plan's events from events.jsonl
        let events_path = crate::orchestrator::paths::runs_dir(&config.repo_root).join("events.jsonl");
        if let Ok(content) = std::fs::read_to_string(&events_path) {
            let filtered: Vec<&str> = content
                .lines()
                .filter(|line| {
                    // Keep lines that don't reference this plan
                    !line.contains(&format!("\"plan\":\"{base}\""))
                        && !line.contains(&format!("\"plan\": \"{base}\""))
                })
                .collect();
            let _ = std::fs::write(&events_path, filtered.join("\n") + "\n");
        }

        // Reset plan status back to Pending
        if let Some(entry) = state.plans.get_mut(state.selected_plan_idx) {
            entry.status = RunPlanStatus::Pending;
            entry.iteration = 0;
            entry.phase.clear();
        }

        // Clean up context files for this plan
        let num = state
            .plans
            .get(state.selected_plan_idx)
            .map(|p| p.num.clone())
            .unwrap_or_default();
        let context_dir = config.repo_root.join("plans/context");
        let _ = std::fs::remove_file(context_dir.join(format!("briefs/{num}-brief.md")));
        // Reset task statuses to pending instead of deleting the file
        // (task TOMLs are pre-generated with context and structure)
        let tasks_path = context_dir.join(format!("tasks/{num}-tasks.toml"));
        if tasks_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&tasks_path) {
                let reset = content
                    .lines()
                    .map(|line| {
                        if line.starts_with("status") {
                            r#"status = "pending""#
                        } else {
                            line
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = std::fs::write(&tasks_path, reset);
            }
        }
        let _ = std::fs::remove_file(context_dir.join(format!("reviews/{num}-arch.md")));
        let _ = std::fs::remove_file(context_dir.join(format!("reviews/{num}-audit.md")));
        let _ = std::fs::remove_file(context_dir.join(format!("reviews/{num}-critic.md")));
        let _ = std::fs::remove_file(context_dir.join(format!("docs/{num}-docs.md")));
        let _ = std::fs::remove_dir_all(context_dir.join(format!("archive/{num}")));

        // Also clear the complete flag so the pipeline can continue
        state.complete = false;
        state.error = None;

        state.add_log(
            "orch",
            &format!("Reset plan {base} — use ctrl+r to restart pipeline"),
            LogLevel::Warn,
        );
    }
}

/// Re-verify: re-run gates + reviews for the selected plan without re-implementing.
pub(crate) async fn reverify_plan(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    _persistence: &PersistenceManager,
    _config: &AppConfig,
) -> anyhow::Result<()> {
    if let Some(plan) = state.plans.get(state.selected_plan_idx) {
        let base = plan.base.clone();
        state.add_log(
            "orch",
            &format!("Re-verifying plan {base} (gates + reviews only)"),
            LogLevel::Info,
        );
        // Move to Gating if orchestrator supports it
        let _ = orchestrator;
        if let Some(entry) = state.plans.get_mut(state.selected_plan_idx) {
            entry.phase = "gating".to_string();
        }
    }
    Ok(())
}

/// Restart the current phase: kill active agent, re-dispatch.
pub(crate) async fn restart_current_phase(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    persistence: &PersistenceManager,
    config: &AppConfig,
) -> Result<()> {
    let active_role = state.agents.iter().find(|(_, s)| s.active).map(|(r, _)| *r);
    if let Some(role) = active_role {
        agent_pool.kill(role).await;
        state.agent_state_mut(role).active = false;
    }
    state.add_log(
        "orch",
        &format!("Restarting phase: {}", state.current_phase),
        LogLevel::Warn,
    );

    match orchestrator.state {
        OrchestratorState::Implementer => {
            let plan = match orchestrator.current_plan() {
                Some(p) => p.clone(),
                None => return Ok(()),
            };
            agent_pool
                .spawn(
                    AgentRole::Implementer,
                    state.config.effort_for(AgentRole::Implementer).label(),
                    state.config.model_for(AgentRole::Implementer),
                )
                .await?;
            let prompt = if config.no_review {
                crate::orchestrator::prompts::implementer_prompt(&config.repo_root, &plan)?
            } else if state.current_iteration > 1 {
                // Iteration 2+: use surgical fix prompt (stripped context, focused on errors)
                crate::orchestrator::prompts::implementer_fix_prompt(
                    &config.repo_root,
                    &plan,
                    state.current_iteration,
                )?
            } else {
                // Iteration 1: full context prompt
                crate::orchestrator::prompts::implementer_prompt_with_brief(
                    &config.repo_root,
                    &plan,
                    state.current_iteration,
                )?
            };
            agent_pool
                .turn_start(
                    AgentRole::Implementer,
                    &prompt,
                    state.config.model_for(AgentRole::Implementer),
                )
                .await?;
            state.agent_state_mut(AgentRole::Implementer).active = true;
        }
        OrchestratorState::Reviewing => {
            start_parallel_reviews(state, orchestrator, agent_pool, persistence, config).await?;
        }
        OrchestratorState::DocRevision | OrchestratorState::CriticReview => {
            // Restart the doc revision or critic phase
            let plan = match orchestrator.current_plan() {
                Some(p) => p.clone(),
                None => return Ok(()),
            };
            let critic_path = config
                .repo_root
                .join(format!("plans/context/reviews/{}-critic.md", plan.num));
            let critic_content = std::fs::read_to_string(&critic_path).unwrap_or_default();
            start_doc_revision(
                state,
                orchestrator,
                agent_pool,
                persistence,
                config,
                &critic_content,
            )
            .await?;
        }
        _ => {
            state.add_log("orch", "Cannot restart this phase", LogLevel::Warn);
        }
    }
    Ok(())
}

/// Restart the entire plan: kill all agents, reset state, clear context files, re-start.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn restart_plan(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
) -> Result<()> {
    agent_pool.kill_all().await;
    state.clear_agent_outputs();
    state.current_iteration = 1;
    orchestrator.current_iteration = 1;
    state.complete = false;
    state.error = None;
    state.iteration_reason.clear();
    state.task_checklist = None;
    state.command_output.clear();

    // Clear ALL state for a full fresh start
    // 1. Clear events.jsonl so completed_plans() doesn't skip things
    let events_path = crate::orchestrator::paths::runs_dir(&config.repo_root).join("events.jsonl");
    let _ = std::fs::write(&events_path, "");

    // 2. Clear context files for ALL plans
    let context_dir = config.repo_root.join("plans/context");
    let _ = std::fs::remove_dir_all(context_dir.join("briefs"));
    let _ = std::fs::remove_dir_all(context_dir.join("tasks"));
    let _ = std::fs::remove_dir_all(context_dir.join("reviews"));
    let _ = std::fs::remove_dir_all(context_dir.join("docs"));
    let _ = std::fs::remove_dir_all(context_dir.join("archive"));
    let _ = std::fs::create_dir_all(context_dir.join("briefs"));
    let _ = std::fs::create_dir_all(context_dir.join("tasks"));
    let _ = std::fs::create_dir_all(context_dir.join("reviews"));
    let _ = std::fs::create_dir_all(context_dir.join("docs"));
    let _ = std::fs::create_dir_all(context_dir.join("archive"));
    let _ = std::fs::remove_file(context_dir.join("last-completed.md"));
    let _ = std::fs::remove_file(context_dir.join("last-gate-output.txt"));

    // 3. Delete ALL plan tags
    let _ = std::process::Command::new("bash")
        .args(["-c", "git tag -l 'plan/*' | xargs -I{} git tag -d {}"])
        .current_dir(&config.repo_root)
        .output();

    // 4. Reset ALL plan statuses to Pending
    for entry in state.plans.iter_mut() {
        entry.status = RunPlanStatus::Pending;
        entry.iteration = 0;
        entry.phase.clear();
    }

    // 5. Reset orchestrator to beginning
    orchestrator.current_plan_idx = 0;
    orchestrator.plans_completed = 0;
    orchestrator.set_state(OrchestratorState::PlanReady);
    state.orchestrator_state = "plan-ready".to_string();
    state.current_plan_idx = 0;

    state.add_log(
        "orch",
        "Full reset — restarting all plans from scratch",
        LogLevel::Warn,
    );

    let mut compile_fail_count = 0u32;
    start_plan(
        state,
        orchestrator,
        agent_pool,
        git_manager,
        persistence,
        config,
        batch_branch,
        run_id,
        started_at,
        &mut compile_fail_count,
    )
    .await
}

/// Spawn retroactive verification tasks for completed-prior plans.
/// Each task checks workspace compilation and generates a summary if missing.
pub(crate) fn spawn_retroactive_verifiers(
    state: &mut RunState,
    config: &AppConfig,
    verify_tx: mpsc::UnboundedSender<VerifyCompletion>,
) {
    let completed_prior: Vec<_> = state
        .plans
        .iter()
        .filter(|p| p.status == RunPlanStatus::CompletedPrior)
        .map(|p| (p.base.clone(), p.num.clone()))
        .collect();

    if completed_prior.is_empty() {
        return;
    }

    state.add_log(
        "verify",
        &format!("Spawning {} retroactive verifiers", completed_prior.len()),
        LogLevel::Info,
    );

    for (base, num) in completed_prior {
        // Add verify entry to state
        state.verify_entries.push(VerifyEntry {
            plan_base: base.clone(),
            plan_num: num.clone(),
            status: VerifyStatus::Running,
            output: String::new(),
            started: Some(Instant::now()),
        });

        let tx = verify_tx.clone();
        let repo_root = config.repo_root.clone();
        let plan_base = base.clone();
        let plan_num = num.clone();

        tokio::spawn(async move {
            // Check if summary already exists
            let has_summary = crate::orchestrator::context::read_summary(&repo_root, &plan_num)
                .ok()
                .flatten()
                .is_some();

            // Parse plan frontmatter to get crates_touched
            let plan_path = repo_root.join(format!("plans/{plan_base}.md"));
            let crates_touched = tokio::fs::read_to_string(&plan_path)
                .await
                .ok()
                .and_then(|content| crate::orchestrator::plan::parse_frontmatter(&content))
                .map(|fm| fm.crates_touched)
                .unwrap_or_default();

            // Run cargo check: per-crate if crates_touched is available, otherwise workspace
            let check_output = if crates_touched.is_empty() {
                tokio::process::Command::new("cargo")
                    .args(["check", "--workspace"])
                    .current_dir(&repo_root)
                    .output()
                    .await
            } else {
                let mut args = vec!["check".to_string()];
                for c in &crates_touched {
                    args.push("-p".to_string());
                    args.push(c.clone());
                }
                tokio::process::Command::new("cargo")
                    .args(&args)
                    .current_dir(&repo_root)
                    .output()
                    .await
            };

            let (passed, output) = match check_output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    let combined = format!("{stdout}\n{stderr}");
                    (o.status.success(), combined)
                }
                Err(e) => (false, format!("Failed to run cargo check: {e}")),
            };

            // Generate summary if missing
            let summary = if !has_summary {
                let plan_info = crate::orchestrator::plan::PlanInfo {
                    base: plan_base.clone(),
                    num: plan_num.clone(),
                    path: plan_path,
                    frontmatter: None,
                };
                crate::orchestrator::context::generate_summary(
                    &repo_root,
                    &plan_info,
                    1,
                    Duration::from_secs(0),
                )
                .ok()
            } else {
                None
            };

            let _ = tx.send(VerifyCompletion::Done {
                plan_base,
                plan_num,
                passed,
                output,
                summary,
            });
        });
    }
}

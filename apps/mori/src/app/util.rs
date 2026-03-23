use std::collections::HashMap;

use tracing::info;

use crate::agent::AgentRole;
use crate::git::worktree::WorktreeManager;
use crate::orchestrator::ParallelExecutor;
use crate::state::RunState;

use super::AppConfig;

pub(crate) fn ensure_selection_visible(state: &mut RunState) {
    // Compute the "global" line index of the selection across all waves.
    let mut global_idx = 0usize;
    for (wi, (_, plans)) in state.execution_waves.iter().enumerate() {
        if wi == state.selected_wave_idx {
            global_idx += state.selected_plan_idx;
            break;
        }
        global_idx += plans.len() + 1; // +1 for wave header line
    }
    // Approximate visible height for the plan list (half terminal, minimum 5)
    let visible = (state.terminal_height as usize / 2).max(5);
    if global_idx < state.plan_scroll_offset {
        state.plan_scroll_offset = global_idx;
    } else if global_idx >= state.plan_scroll_offset + visible {
        state.plan_scroll_offset = global_idx.saturating_sub(visible - 1);
    }
}

pub(crate) fn parallel_refresh_tasks(state: &mut RunState, config: &AppConfig) {
    // Reload checklist for every plan, not just the selected one
    let plans: Vec<(String, String)> = state
        .plans
        .iter()
        .map(|p| (p.base.clone(), p.num.clone()))
        .collect();
    for (plan_base, plan_num) in plans {
        if let Ok(Some(cl)) =
            crate::orchestrator::tasks::load_checklist(&config.repo_root, &plan_num)
        {
            state.plan_task_cache.insert(plan_base.clone(), cl.clone());
            if state.selected_plan_idx < state.plans.len()
                && state.plans[state.selected_plan_idx].base == plan_base
            {
                state.task_checklist = Some(cl);
            }
        }
    }
}

/// For review agents spawned during the parallel review phase, extract the prefix and plan base.
/// Instance IDs are formatted as `"{prefix}:{plan_base}"` (e.g. `"arch:01-foo"`).
pub(crate) fn review_plan_from_iid(iid: &str) -> Option<(&'static str, String)> {
    for prefix in &["arch", "audit", "scribe", "critic", "quick"] {
        if let Some(rest) = iid.strip_prefix(&format!("{prefix}:")) {
            // Handle timestamped IDs like "scribe:01-foo:1234567890"
            // Plan base is the part before any second colon
            let plan = rest.split(':').next().unwrap_or(rest).to_string();
            return Some((prefix, plan));
        }
    }
    None
}

/// Extract the plan base from any instance ID.
/// Instance IDs are formatted as `"{role}:{plan_base}:{timestamp?}"` (e.g. `"impl:01-foo:1234567890"`).
/// Returns the plan base portion (e.g. `"01-foo"`).
pub(crate) fn plan_base_from_iid(iid: &str) -> Option<String> {
    // Split by ':' and take the second part (first part is the role, second is the plan)
    iid.split(':').nth(1).map(|s| s.to_string())
}

/// Extract the last ~4KB output from a parallel agent by instance ID.
pub(crate) fn get_parallel_agent_output(state: &RunState, instance_id: &str) -> String {
    state
        .parallel_agents
        .iter()
        .find(|p| p.instance_id == instance_id)
        .map(|p| p.output.clone())
        .unwrap_or_default()
}

/// Read the review file an agent wrote to disk, checking worktree first then repo root.
/// Returns empty string if no file found.
pub(crate) fn read_review_file(
    repo_root: &std::path::Path,
    worktree: Option<&std::path::Path>,
    prefix: &str,
    plan: &str,
) -> String {
    let plan_num = plan.split('-').next().unwrap_or(plan);
    let suffix = match prefix {
        "arch" => "arch",
        "audit" => "audit",
        "critic" => "critic",
        "quick" => "quick",
        _ => return String::new(),
    };
    let filename = format!("plans/context/reviews/{plan_num}-{suffix}.md");
    // Try worktree first
    if let Some(wt) = worktree {
        if let Ok(c) = std::fs::read_to_string(wt.join(&filename)) {
            if !c.trim().is_empty() {
                return c;
            }
        }
    }
    std::fs::read_to_string(repo_root.join(&filename)).unwrap_or_default()
}

/// Write a checkpoint (task-state.json) immediately from current executor state.
/// Called after every task completion to minimize data loss on crash.
pub(crate) fn write_checkpoint(
    executor: &ParallelExecutor,
    persistence: &crate::state::persistence::PersistenceManager,
    worktree_mgr: &WorktreeManager,
    batch_branch: &str,
    total_input_tokens: u64,
    total_output_tokens: u64,
) {
    write_checkpoint_inner(
        executor,
        persistence,
        worktree_mgr,
        batch_branch,
        total_input_tokens,
        total_output_tokens,
        None,
    )
}

/// Write a checkpoint with an optional correction factor from the TimeEstimator.
pub(crate) fn write_checkpoint_with_state(
    executor: &ParallelExecutor,
    persistence: &crate::state::persistence::PersistenceManager,
    worktree_mgr: &WorktreeManager,
    batch_branch: &str,
    total_input_tokens: u64,
    total_output_tokens: u64,
    state: &RunState,
) {
    let cf = if (state.time_estimator.correction_factor - 1.0).abs() > f64::EPSILON {
        Some(state.time_estimator.correction_factor)
    } else {
        None
    };
    write_checkpoint_inner(
        executor,
        persistence,
        worktree_mgr,
        batch_branch,
        total_input_tokens,
        total_output_tokens,
        cf,
    )
}

fn write_checkpoint_inner(
    executor: &ParallelExecutor,
    persistence: &crate::state::persistence::PersistenceManager,
    worktree_mgr: &WorktreeManager,
    batch_branch: &str,
    total_input_tokens: u64,
    total_output_tokens: u64,
    correction_factor: Option<f64>,
) {
    let snapshot = executor.snapshot();
    let task_state = crate::state::persistence::TaskStateFile {
        version: 2,
        run_id: String::new(),
        batch_branch: batch_branch.to_string(),
        completed_tasks: snapshot.completed_tasks,
        in_flight: snapshot.in_flight_tasks,
        completed_plans: snapshot.completed_plans,
        total_tokens: crate::state::persistence::TokenCount {
            input: total_input_tokens,
            output: total_output_tokens,
        },
        plan_iterations: snapshot.plan_iterations,
        merge_queue: snapshot.merge_queue,
        plans_since_refactor: snapshot.plans_since_refactor,
        plans_since_integration_test: snapshot.plans_since_integration_test,
        active_worktrees: {
            let mut wts = HashMap::new();
            for plan in executor.active_plans() {
                let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                if wt_path.exists() {
                    wts.insert(plan.to_string(), wt_path.to_string_lossy().to_string());
                }
            }
            wts
        },
        plan_phases: snapshot
            .plan_phases
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::to_string(v).unwrap_or_else(|_| format!("{v:?}")),
                )
            })
            .collect(),
        merge_in_progress: if executor.merge_in_progress {
            executor
                .plan_states
                .iter()
                .find(|(_, s)| matches!(s.phase, crate::orchestrator::executor::PlanPhase::Merging))
                .map(|(plan, _)| {
                    let wt_path = worktree_mgr.worktree_base().join(format!("plan-{plan}"));
                    let worktree_head = crate::git::ops::run_git(&wt_path, &["rev-parse", "HEAD"])
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    crate::state::persistence::MergeCheckpoint {
                        plan: plan.clone(),
                        worktree_head,
                        batch_ref: batch_branch.to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }
                })
        } else {
            None
        },
        review_feedback: snapshot.review_feedback,
        correction_factor,
    };
    let _ = persistence.write_task_state(&task_state);
}

pub(crate) fn current_agent_line_count(state: &RunState) -> usize {
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
    state
        .agent_state(role)
        .map(|a| a.output.lines().count())
        .unwrap_or(0)
}

pub(crate) fn cycle_idx(current: usize, len: usize, forward: bool) -> usize {
    if forward {
        (current + 1) % len
    } else {
        if current == 0 {
            len - 1
        } else {
            current - 1
        }
    }
}

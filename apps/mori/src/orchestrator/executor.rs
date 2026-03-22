use std::collections::{HashMap, HashSet};
use std::time::Instant;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::unified_dag::{GlobalTaskId, UnifiedTaskDag};

/// Actions the executor wants the outer event loop to perform.
#[derive(Debug, Clone)]
pub enum ExecutorAction {
    /// Ensure a plan pipeline exists in the active set.
    CreatePipeline { plan: String },
    /// Ensure a worktree exists for this plan.
    EnsureWorktree { plan: String },
    /// Spawn an implementer agent for one task.
    SpawnTaskAgent {
        task_id: GlobalTaskId,
        instance_id: String,
    },
    /// Spawn one implementer agent for a batch of tasks (same plan).
    SpawnTaskAgentBatch {
        plan: String,
        tasks: Vec<GlobalTaskId>,
        instance_id: String,
    },
    /// All tasks for this plan are done — run compile + test gates.
    RunPlanGates { plan: String },
    /// Pre-spawn a warm reviewer for gate-review overlap. Called alongside RunPlanGates.
    /// The event loop should spawn the reviewer and store its instance_id in PlanState.active_reviewer_instance.
    PreSpawnWarmReviewer { plan: String },
    /// Gates passed — run parallel reviews (Architect + Auditor + Scribe).
    RunPlanReviews { plan: String },
    /// Reviews passed — merge plan branch into batch.
    MergePlanToBatch { plan: String },
    /// Speculatively pre-plan an upcoming plan.
    SpawnPrePlanner { plan: String },
    /// Run a batch-level refactoring pass.
    SpawnRefactorer { batch_branch: String },
    /// Verify docs after a plan merges.
    SpawnDocVerifier { plan: String },
    /// Run cross-crate integration tests on the batch branch.
    RunIntegrationTests { batch_branch: String },
    /// Start reviewer in parallel with gates (gate-review overlap).
    /// Emitted simultaneously with RunPlanGates when implementer completes.
    /// If gates fail, the reviewer instance should be interrupted and killed.
    StartReviewerInParallel { plan: String, instance_id: String },
    /// Interrupt and kill an active reviewer (called when gates fail during overlap).
    CancelActiveReviewer { plan: String, instance_id: String },
    /// Attempt to resolve merge conflicts for a plan.
    ResolveMergeConflict { plan: String },
    /// Re-run gates (not full re-implementation) for a dependent plan
    /// after an upstream invariant fix changed its API.
    ReGatePlan { plan: String },
    /// Spawn an error diagnoser to produce targeted fixes instead of full re-implementation.
    DiagnoseError { plan: String, gate_output: String },
    /// Spawn a dependency validator before implementation starts.
    ValidateDependencies { plan: String },
    /// Spawn a pattern extractor to analyze existing code patterns.
    ExtractPatterns { plan: String },
    /// Run workspace-wide regression tests after a plan merges.
    RunPostMergeRegression { batch_branch: String },
    /// A plan exceeded its wall-clock timeout.
    PlanTimeout { plan: String },
    /// Force-advance a plan blocking the merge queue.
    ForceAdvancePlan { plan: String, reason: String },
    /// Express mode: spawn a single implementer (no strategist) for a plan.
    SpawnImplementer { plan: String },
    /// Express mode: spawn a lightweight auto-fixer after gate failure.
    AutoFixErrors { plan: String, errors: String },
    /// Full clean retry: remove worktree + branch, clear all state, reschedule from scratch.
    CleanRetryPlan { plan: String },
    /// Written to context/in/ before SpawnTaskAgent or review agent.
    PrepareContext {
        worktree: std::path::PathBuf,
        plan: String,
        iter: u32,
        for_role: crate::agent::AgentRole,
    },
    /// Read from context/out/ after agent completes; archive to ArtifactStore.
    CollectOutput {
        worktree: std::path::PathBuf,
        plan: String,
        iter: u32,
        from_role: crate::agent::AgentRole,
    },
}

/// Tracks the lifecycle phase of a plan within the executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanPhase {
    /// Tasks are being executed.
    Implementing,
    /// All tasks done, gates running.
    Gating,
    /// Gates passed, reviews running.
    Reviewing,
    /// Reviews passed, merging to batch.
    Merging,
    /// Fully merged into batch.
    Complete,
    /// Something failed.
    Failed(String),
    /// Express mode: auto-fixing compile/test errors after a gate failure.
    AutoFixing,
}

/// Per-plan tracking within the executor.
#[derive(Debug, Clone)]
pub struct PlanState {
    pub plan_base: String,
    pub phase: PlanPhase,
    pub started_at: Instant,
    pub iteration: u32,
    /// If set, a reviewer agent is running in parallel with gates (gate-review overlap).
    /// When gates complete, check this to decide if reviewer should continue or be cancelled.
    pub active_reviewer_instance: Option<String>,
    /// The primary implementer agent for this plan — stays warm across phases
    /// so gate-failure fix turns reuse context instead of cold-starting.
    pub primary_agent_instance: Option<String>,
}

/// Snapshot of executor state for persistence / crash recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorSnapshot {
    pub completed_tasks: Vec<String>,
    pub in_flight_tasks: HashMap<String, String>,
    pub completed_plans: Vec<String>,
    pub plan_phases: HashMap<String, PlanPhase>,
    /// Per-plan iteration counts (plan_base -> iteration)
    #[serde(default)]
    pub plan_iterations: HashMap<String, u32>,
    /// Plans waiting to merge (dependency-ordered)
    #[serde(default)]
    pub merge_queue: Vec<String>,
    #[serde(default)]
    pub plans_since_refactor: usize,
    #[serde(default)]
    pub plans_since_integration_test: usize,
    /// Archived review feedback per plan (persisted for crash recovery)
    #[serde(default)]
    pub review_feedback: HashMap<String, Vec<String>>,
}

/// The parallel scheduling engine.
///
/// Replaces the sequential plan loop in `app.rs`. The executor owns the unified
/// task DAG and makes all scheduling decisions. The event loop just dispatches
/// the resulting `ExecutorAction`s and routes events back.
pub struct ParallelExecutor {
    dag: UnifiedTaskDag,
    completed_tasks: HashSet<GlobalTaskId>,
    in_flight_tasks: HashMap<GlobalTaskId, String>,
    max_concurrent_agents: usize,
    pub plan_states: HashMap<String, PlanState>,
    completed_plans: HashSet<String>,
    /// Plans that finished gates/reviews and are ready to merge, but waiting
    /// for dependency-order merge slot. Merges are serialized: only one plan
    /// merges to batch at a time, and plans merge in dependency order.
    pub merge_queue: Vec<String>,
    /// True while a merge is in progress (prevents concurrent merges).
    pub merge_in_progress: bool,
    /// The plan currently executing a merge (set when merge_in_progress = true).
    pub currently_merging: Option<String>,
    pre_planned: HashSet<String>,
    /// Archived review feedback per plan (for re-implementation prompts).
    pub review_feedback: HashMap<String, Vec<String>>,
    plans_since_refactor: usize,
    plans_since_integration_test: usize,
    refactor_interval: usize,
    /// Run integration tests every N plans (default 3). 0 = disabled.
    integration_test_interval: usize,
    refactoring_active: bool,
    integration_testing_active: bool,
    batch_branch: String,
    /// Tracks active utility agent instance IDs (pre-planners, refactorers, etc.)
    /// so they count against the agent budget.
    utility_agents: HashSet<String>,
    /// Per-task failure count — after 3 failures, permanently fail the plan (A2).
    task_failure_count: HashMap<GlobalTaskId, u32>,
    /// Wall-clock limit per plan (A1). Default 45 minutes.
    wall_clock_limit: std::time::Duration,
    /// When each merge queue entry was added (A6).
    merge_queue_entered_at: HashMap<String, Instant>,
    /// Express mode: skip strategist and reviews, use auto-fix on gate failure.
    pub express_mode: bool,
    /// Max auto-fix attempts per plan before failing (express mode only).
    max_auto_fix_attempts: u32,
    /// Consecutive spawn failure count per plan. Reset on any successful spawn.
    spawn_failure_counts: HashMap<String, u32>,
    /// Plans blocked from scheduling until this instant (after 3 consecutive failures).
    spawn_blocked_until: HashMap<String, Instant>,
    /// Total active agents across ALL roles (set by the event loop before schedule_next).
    /// Used for budget calculation so reviewers/auto-fixers/scribes count against the limit.
    total_active_agents: usize,
}

impl ParallelExecutor {
    pub fn new(
        dag: UnifiedTaskDag,
        max_concurrent_agents: usize,
        refactor_interval: usize,
        batch_branch: String,
    ) -> Self {
        Self {
            dag,
            completed_tasks: HashSet::new(),
            in_flight_tasks: HashMap::new(),
            max_concurrent_agents,
            plan_states: HashMap::new(),
            completed_plans: HashSet::new(),
            merge_queue: Vec::new(),
            merge_in_progress: false,
            currently_merging: None,
            pre_planned: HashSet::new(),
            review_feedback: HashMap::new(),
            plans_since_refactor: 0,
            plans_since_integration_test: 0,
            refactor_interval,
            integration_test_interval: 3,
            refactoring_active: false,
            integration_testing_active: false,
            batch_branch,
            utility_agents: HashSet::new(),
            task_failure_count: HashMap::new(),
            wall_clock_limit: std::time::Duration::from_secs(45 * 60),
            merge_queue_entered_at: HashMap::new(),
            express_mode: false,
            max_auto_fix_attempts: 3,
            spawn_failure_counts: HashMap::new(),
            spawn_blocked_until: HashMap::new(),
            total_active_agents: 0,
            // Note: active_reviewer_instance is initialized per-plan in create_plan_state()
        }
    }

    /// Configure express mode after construction.
    pub fn set_express_mode(&mut self, enabled: bool, max_auto_fix_attempts: u32) {
        self.express_mode = enabled;
        self.max_auto_fix_attempts = max_auto_fix_attempts;
    }

    /// Record a spawn failure for a plan. Apply exponential backoff starting from first failure.
    /// Backoff: 1st failure = 2s, 2nd = 4s, 3rd+ = 30s (full block).
    pub fn record_spawn_failure(&mut self, plan: &str) {
        let count = self
            .spawn_failure_counts
            .entry(plan.to_string())
            .or_insert(0);
        *count += 1;
        let n = *count;
        let backoff_secs = match n {
            1 => 2,  // 1st failure: 2 second backoff
            2 => 4,  // 2nd failure: 4 second backoff
            _ => 30, // 3rd+ failures: 30 second backoff
        };
        let blocked_until = Instant::now() + std::time::Duration::from_secs(backoff_secs);
        self.spawn_blocked_until
            .insert(plan.to_string(), blocked_until);
        tracing::warn!(
            "Plan {} spawn failed ({}/3) — backoff {}s",
            plan,
            n,
            backoff_secs
        );
    }

    /// Record a successful spawn for a plan. Clears any backoff state.
    pub fn record_spawn_success(&mut self, plan: &str) {
        self.spawn_failure_counts.remove(plan);
        self.spawn_blocked_until.remove(plan);
    }

    /// Core scheduling loop. Called after every state change (task complete,
    /// gate result, etc.) to determine what to do next.
    pub fn schedule_next(&mut self) -> Vec<ExecutorAction> {
        self.schedule_next_with_budget(None)
    }

    /// Schedule next actions with an explicit total active agent count.
    /// When `active_agents` is provided, it overrides the internal counter
    /// so that reviewers, auto-fixers, scribes, and other non-implementer
    /// agents count against the concurrency budget.
    pub fn schedule_next_with_budget(
        &mut self,
        active_agents: Option<usize>,
    ) -> Vec<ExecutorAction> {
        if let Some(count) = active_agents {
            self.total_active_agents = count;
        }
        let mut actions = vec![];

        // A1: Check wall-clock timeout for each active plan
        let timed_out: Vec<String> = self
            .plan_states
            .iter()
            .filter(|(_, s)| !matches!(s.phase, PlanPhase::Complete | PlanPhase::Failed(_)))
            .filter(|(_, s)| s.started_at.elapsed() > self.wall_clock_limit)
            .map(|(name, _)| name.clone())
            .collect();
        for plan in timed_out {
            actions.push(ExecutorAction::PlanTimeout { plan });
        }

        // Auto-complete synthetic __whole__ plan nodes whose plan-level deps
        // are all satisfied AND whose pipeline has already run (strategist +
        // implementer). Without the pipeline check, whole-nodes get marked done
        // instantly, skipping strategist/implementer/gates entirely.
        let whole_nodes: Vec<GlobalTaskId> = self
            .dag
            .all_plans()
            .iter()
            .map(|p| GlobalTaskId {
                plan: p.to_string(),
                task: "__whole__".to_string(),
            })
            .filter(|gid| {
                !self.completed_tasks.contains(gid)
                    && self.dag.tasks_for_plan(&gid.plan).len() == 1
                    && self
                        .dag
                        .tasks_for_plan(&gid.plan)
                        .iter()
                        .any(|t| t.task == "__whole__")
            })
            .collect();
        for gid in whole_nodes {
            // Only auto-complete if this plan has already been through the
            // implementation pipeline (has a plan_state). Otherwise the plan
            // needs to go through strategist → implementer → gates first.
            if !self.plan_states.contains_key(&gid.plan) {
                continue;
            }
            let deps_met = self.dag.task_deps(&gid).map_or(true, |deps| {
                deps.iter().all(|d| self.completed_tasks.contains(d))
            });
            if deps_met {
                info!(
                    "Plan {} whole-node auto-completed (deps satisfied)",
                    gid.plan
                );
                self.completed_tasks.insert(gid);
            }
        }

        // Recover stalled plans: all tasks completed but no plan pipeline state yet
        // (happens when a prior run completed all tasks then crashed before gating).
        // We detect these here so schedule_next() emits the right follow-on action.
        let all_plans: Vec<String> = self.dag.all_plans().iter().map(|s| s.to_string()).collect();
        for plan in &all_plans {
            if self.completed_plans.contains(plan.as_str())
                || self.plan_states.contains_key(plan.as_str())
            {
                continue;
            }
            let plan_tasks = self.dag.tasks_for_plan(plan);
            if !plan_tasks.is_empty()
                && self
                    .dag
                    .all_plan_tasks_complete(plan, &self.completed_tasks)
            {
                self.plan_states.insert(
                    plan.clone(),
                    PlanState {
                        plan_base: plan.clone(),
                        phase: PlanPhase::Gating,
                        started_at: Instant::now(),
                        iteration: 1,
                        active_reviewer_instance: None,
                        primary_agent_instance: None,
                    },
                );
                info!("Recovered stalled plan {plan} — all tasks done, advancing to gating");
                actions.extend(self.emit_gates_with_warm_reviewer(&plan));
            }
        }

        // Recover plans stuck in Implementing or Failed with all tasks done. This covers:
        //   - Restart after crash mid-run: plan_state exists (Implementing) + all tasks in
        //     completed_tasks, but handle_task_complete is never re-called after restore.
        //   - Premature failure: plan marked Failed while agents still in-flight; tasks
        //     eventually all completed but the Failed phase blocked gate dispatch.
        for plan in &all_plans {
            if self.completed_plans.contains(plan.as_str()) {
                continue;
            }
            let needs_recovery = self
                .plan_states
                .get(plan.as_str())
                .map(|s| matches!(s.phase, PlanPhase::Failed(_) | PlanPhase::Implementing))
                .unwrap_or(false);
            if needs_recovery
                && self
                    .dag
                    .all_plan_tasks_complete(plan, &self.completed_tasks)
            {
                // Make sure no tasks are still actually in-flight before declaring done
                let any_in_flight = self.in_flight_tasks.keys().any(|gid| &gid.plan == plan);
                if any_in_flight {
                    continue;
                }
                if let Some(state) = self.plan_states.get_mut(plan.as_str()) {
                    info!(plan = %plan, prior_phase = ?state.phase, "Recovered plan — all tasks done, advancing to gating");
                    state.phase = PlanPhase::Gating;
                    state.started_at = Instant::now();
                }
                actions.extend(self.emit_gates_with_warm_reviewer(&plan));
            }
        }

        let budget = self.max_concurrent_agents.saturating_sub(
            self.total_active_agents
                .max(self.in_flight_tasks.len() + self.utility_agents.len()),
        );
        if budget == 0 {
            debug!(
                max = self.max_concurrent_agents,
                total = self.total_active_agents,
                in_flight = self.in_flight_tasks.len(),
                utility = self.utility_agents.len(),
                "schedule_next: budget=0"
            );
            return actions;
        }

        let in_flight_ids: HashSet<GlobalTaskId> = self.in_flight_tasks.keys().cloned().collect();

        let runnable: Vec<GlobalTaskId> = self
            .dag
            .next_runnable(&self.completed_tasks, &in_flight_ids)
            .into_iter()
            .cloned()
            .collect();

        // Collect tasks to batch-spawn, keeping __whole__ nodes on the fast path.
        let mut tasks_to_batch: Vec<GlobalTaskId> = Vec::new();

        for task_id in runnable.into_iter().take(budget) {
            // For __whole__ nodes (plans without task breakdowns):
            // create the pipeline and spawn a strategist so the full
            // strategist → implementer → gates flow runs.
            if task_id.task == "__whole__" {
                if !self.plan_states.contains_key(&task_id.plan) {
                    self.plan_states.insert(
                        task_id.plan.clone(),
                        PlanState {
                            plan_base: task_id.plan.clone(),
                            phase: PlanPhase::Implementing,
                            started_at: Instant::now(),
                            iteration: 1,
                            active_reviewer_instance: None,
                            primary_agent_instance: None,
                        },
                    );
                    // Reserve the budget slot so concurrent strategist/implementer agents
                    // count against max_concurrent_agents. Uses a sentinel value that
                    // task_for_instance() skips, so TurnCompleted routes to the
                    // strategist/express-impl handler — not the task handler.
                    self.in_flight_tasks
                        .insert(task_id.clone(), format!("_pending_:{}", task_id.plan));
                    actions.push(ExecutorAction::CreatePipeline {
                        plan: task_id.plan.clone(),
                    });
                    actions.push(ExecutorAction::EnsureWorktree {
                        plan: task_id.plan.clone(),
                    });
                    // Offline enrichment replaced strategist — go straight to schedule_next()
                    if self.express_mode {
                        actions.push(ExecutorAction::SpawnImplementer {
                            plan: task_id.plan.clone(),
                        });
                    } else {
                        // Mark plan as Implementing and schedule tasks directly
                        if let Some(state) = self.plan_states.get_mut(&task_id.plan) {
                            state.phase = PlanPhase::Implementing;
                        }
                        actions.extend(self.schedule_next());
                    }
                }
                continue;
            }

            // Skip tasks for plans that have permanently failed
            if let Some(state) = self.plan_states.get(&task_id.plan) {
                if matches!(state.phase, PlanPhase::Failed(_)) {
                    continue;
                }
            }

            // Skip plans that are in spawn backoff
            if let Some(&blocked_until) = self.spawn_blocked_until.get(&task_id.plan) {
                if Instant::now() < blocked_until {
                    continue;
                } else {
                    self.spawn_blocked_until.remove(&task_id.plan);
                }
            }

            tasks_to_batch.push(task_id);
        }

        // Group tasks by plan — emit one SpawnTaskAgentBatch per plan.
        // Pipeline creation actions must come before batch-spawn actions so that
        // execute_actions (which processes non-spawn actions first) has the
        // pipeline ready before the batch spawn fires.
        let mut seen_plans: HashSet<String> = HashSet::new();
        for task_id in &tasks_to_batch {
            if seen_plans.insert(task_id.plan.clone())
                && !self.plan_states.contains_key(&task_id.plan)
            {
                actions.push(ExecutorAction::CreatePipeline {
                    plan: task_id.plan.clone(),
                });
                actions.push(ExecutorAction::EnsureWorktree {
                    plan: task_id.plan.clone(),
                });
            }
        }

        let mut plan_groups: HashMap<String, Vec<GlobalTaskId>> = HashMap::new();
        for task_id in tasks_to_batch {
            plan_groups
                .entry(task_id.plan.clone())
                .or_default()
                .push(task_id);
        }

        for (plan_id, tasks) in plan_groups {
            let groups = self.dag.independent_groups(&tasks);
            if groups.len() <= 1 {
                let instance_id = format!("implementer:{}", plan_id);
                // Eagerly mark tasks as in-flight so concurrent schedule_next() calls
                // don't see them as runnable and create duplicate spawns.
                for task_id in &tasks {
                    self.in_flight_tasks
                        .insert(task_id.clone(), instance_id.clone());
                }
                actions.push(ExecutorAction::SpawnTaskAgentBatch {
                    plan: plan_id,
                    tasks,
                    instance_id,
                });
            } else {
                info!(plan = %plan_id, groups = groups.len(), "Splitting into independent task groups");
                for (idx, group) in groups.into_iter().enumerate() {
                    let instance_id = format!("implementer:{}:g{}", plan_id, idx);
                    for task_id in &group {
                        self.in_flight_tasks
                            .insert(task_id.clone(), instance_id.clone());
                    }
                    actions.push(ExecutorAction::SpawnTaskAgentBatch {
                        plan: plan_id.clone(),
                        tasks: group,
                        instance_id,
                    });
                }
            }
        }

        // Schedule pre-planning for upcoming plans (if budget allows)
        actions.extend(self.schedule_pre_planning());

        actions
    }

    /// Mark a task as completed and return follow-up actions.
    pub fn handle_task_complete(&mut self, task_id: GlobalTaskId) -> Vec<ExecutorAction> {
        self.in_flight_tasks.remove(&task_id);
        self.completed_tasks.insert(task_id.clone());

        info!("Task completed: {}", task_id);

        let mut actions = vec![];

        // Check if all tasks for this plan are done
        if self
            .dag
            .all_plan_tasks_complete(&task_id.plan, &self.completed_tasks)
        {
            let plan = task_id.plan.clone();
            if let Some(state) = self.plan_states.get_mut(&plan) {
                // Also recover plans that were marked Failed due to retry exhaustion but
                // whose tasks eventually all completed — the failure was premature.
                if matches!(state.phase, PlanPhase::Implementing | PlanPhase::Failed(_)) {
                    state.phase = PlanPhase::Gating;
                    info!("All tasks complete for plan {}, running gates", plan);
                    actions.extend(self.emit_gates_with_warm_reviewer(&plan));
                }
            }
        }

        // Schedule more work
        actions.extend(self.schedule_next());
        actions
    }

    /// A2: Handle task failure with retry limit. After 3 failures, permanently fail the plan.
    pub fn handle_task_failed(&mut self, task_id: GlobalTaskId) -> Vec<ExecutorAction> {
        self.in_flight_tasks.remove(&task_id);

        // Don't retry tasks for plans already permanently failed — they would just increment
        // the failure counter indefinitely and keep spawning agents for a dead plan.
        if let Some(state) = self.plan_states.get(&task_id.plan) {
            if matches!(state.phase, PlanPhase::Failed(_)) {
                info!(task = %task_id, "task exited but plan already failed — ignoring");
                return self.schedule_next();
            }
        }

        let count = self.task_failure_count.entry(task_id.clone()).or_insert(0);
        *count += 1;
        let failures = *count;

        if failures >= 5 {
            let reason = format!("task {} failed {} times", task_id, failures);
            info!(plan = %task_id.plan, reason = %reason, "Permanently failing plan");
            if let Some(state) = self.plan_states.get_mut(&task_id.plan) {
                state.phase = PlanPhase::Failed(reason);
            }
            // Cancel remaining in-flight tasks for this plan
            let plan_in_flight: Vec<GlobalTaskId> = self
                .in_flight_tasks
                .keys()
                .filter(|gid| gid.plan == task_id.plan)
                .cloned()
                .collect();
            for gid in plan_in_flight {
                self.in_flight_tasks.remove(&gid);
            }
            // Remove from merge queue if queued
            self.merge_queue.retain(|p| p != &task_id.plan);
            self.merge_queue_entered_at.remove(&task_id.plan);
            return self.schedule_next();
        }

        info!(task = %task_id, attempt = failures, "Task failed — will retry");
        // Mark task as not completed so it becomes runnable again
        self.completed_tasks.remove(&task_id);
        self.schedule_next()
    }

    /// Record that an agent was spawned for a task.
    pub fn record_task_started(&mut self, task_id: GlobalTaskId, instance_id: String) {
        self.in_flight_tasks.insert(task_id.clone(), instance_id);

        // Ensure plan state exists
        if !self.plan_states.contains_key(&task_id.plan) {
            self.plan_states.insert(
                task_id.plan.clone(),
                PlanState {
                    plan_base: task_id.plan.clone(),
                    phase: PlanPhase::Implementing,
                    started_at: Instant::now(),
                    iteration: 1,
                    active_reviewer_instance: None,
                    primary_agent_instance: None,
                },
            );
        }
    }

    /// Release the _pending_: budget placeholder for a plan.
    /// Called when the initial agent spawn for a plan fails, to free the reserved budget slot.
    pub fn release_pending_sentinel(&mut self, plan: &str) {
        let whole_id = GlobalTaskId {
            plan: plan.to_string(),
            task: "__whole__".to_string(),
        };
        self.in_flight_tasks.remove(&whole_id);
    }

    /// Gates passed for a plan — move to reviews (or skip directly to merge in express mode).
    pub fn handle_plan_gates_passed(&mut self, plan: &str) -> Vec<ExecutorAction> {
        // Guard: if already in a review/merge/complete/failed phase, don't re-trigger reviews
        // This prevents double-gate events from spawning multiple review cycles
        if let Some(state) = self.plan_states.get(plan) {
            if matches!(
                state.phase,
                PlanPhase::Reviewing
                    | PlanPhase::Merging
                    | PlanPhase::Complete
                    | PlanPhase::Failed(_)
            ) {
                return vec![];
            }
        }
        if self.express_mode {
            // Express: skip reviews entirely, go straight to merge.
            return self.handle_plan_reviews_passed(plan);
        }
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.phase = PlanPhase::Reviewing;
        }
        vec![ExecutorAction::RunPlanReviews {
            plan: plan.to_string(),
        }]
    }

    /// Gates failed — auto-fix in express mode, re-implement in standard mode.
    pub fn handle_plan_gates_failed(&mut self, plan: &str) -> Vec<ExecutorAction> {
        self.handle_plan_gates_failed_with_errors(plan, String::new())
    }

    /// Gates failed with captured error output — auto-fix in express mode.
    pub fn handle_plan_gates_failed_with_errors(
        &mut self,
        plan: &str,
        errors: String,
    ) -> Vec<ExecutorAction> {
        if self.express_mode {
            let attempts = self.plan_states.get(plan).map(|s| s.iteration).unwrap_or(1);
            if attempts > self.max_auto_fix_attempts {
                let reason = format!("{} auto-fix attempts exhausted", self.max_auto_fix_attempts);
                info!("Failing plan {plan} — {reason}");
                if let Some(state) = self.plan_states.get_mut(plan) {
                    state.phase = PlanPhase::Failed(reason);
                }
                return self.schedule_next();
            }
            if let Some(state) = self.plan_states.get_mut(plan) {
                state.iteration += 1;
                state.phase = PlanPhase::AutoFixing;
            }
            return vec![ExecutorAction::AutoFixErrors {
                plan: plan.to_string(),
                errors,
            }];
        }

        if let Some(state) = self.plan_states.get_mut(plan) {
            state.iteration += 1;
            state.phase = PlanPhase::Implementing;
        }
        // The tasks need to be re-run. Mark all plan tasks as not-completed
        // so they become runnable again.
        let plan_tasks: Vec<GlobalTaskId> =
            self.dag.tasks_for_plan(plan).into_iter().cloned().collect();
        for tid in &plan_tasks {
            self.completed_tasks.remove(tid);
        }
        self.schedule_next()
    }

    /// Auto-fix agent completed — re-run gates.
    pub fn handle_auto_fix_complete(&mut self, plan: &str) -> Vec<ExecutorAction> {
        // Reset failure count for all tasks in this plan so they get fresh retries
        let plan_tasks: Vec<GlobalTaskId> =
            self.dag.tasks_for_plan(plan).into_iter().cloned().collect();
        for task_id in plan_tasks {
            self.task_failure_count.remove(&task_id);
        }

        if let Some(state) = self.plan_states.get_mut(plan) {
            state.phase = PlanPhase::Gating;
        }
        vec![ExecutorAction::RunPlanGates {
            plan: plan.to_string(),
        }]
    }

    /// Reviews passed — enqueue for merge. Merges are serialized and
    /// happen in dependency order to prevent conflicts.
    pub fn handle_plan_reviews_passed(&mut self, plan: &str) -> Vec<ExecutorAction> {
        // Don't merge if plan is not ready (still implementing/gating) or already in a later phase
        if let Some(state) = self.plan_states.get(plan) {
            if matches!(
                state.phase,
                PlanPhase::Implementing
                    | PlanPhase::Gating
                    | PlanPhase::Failed(_)
                    | PlanPhase::Complete
                    | PlanPhase::Merging
            ) {
                return vec![];
            }
        }
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.phase = PlanPhase::Merging;
        }
        if !self.merge_queue.contains(&plan.to_string()) {
            self.merge_queue.push(plan.to_string());
            self.merge_queue_entered_at
                .entry(plan.to_string())
                .or_insert_with(Instant::now);
        }
        self.drain_merge_queue()
    }

    /// Try to dequeue and merge the next plan whose plan-level deps are all
    /// merged. Returns at most one MergePlanToBatch action (serialized).
    /// A6: Detects merge queue deadlocks (stale entries > 30min).
    pub fn drain_merge_queue(&mut self) -> Vec<ExecutorAction> {
        if self.merge_in_progress || self.merge_queue.is_empty() {
            // A6: Check for stale entries even when merge is in progress
            if !self.merge_queue.is_empty() {
                let mut deadlock_actions = vec![];
                let stale: Vec<String> = self
                    .merge_queue_entered_at
                    .iter()
                    .filter(|(plan, entered)| {
                        self.merge_queue.contains(plan)
                            && entered.elapsed() > std::time::Duration::from_secs(30 * 60)
                    })
                    .map(|(plan, _)| plan.clone())
                    .collect();
                for plan in stale {
                    self.merge_queue_entered_at.remove(&plan);
                    deadlock_actions.push(ExecutorAction::ForceAdvancePlan {
                        plan,
                        reason: "merge queue deadlock (30min stale)".to_string(),
                    });
                }
                if !deadlock_actions.is_empty() {
                    return deadlock_actions;
                }
            }
            return vec![];
        }
        // Find the first queued plan whose plan-level dependencies are all in
        // completed_plans (already merged to batch).
        let ready_idx = self.merge_queue.iter().position(|plan| {
            let deps = self.dag.plan_dependencies(plan);
            deps.iter().all(|dep| self.completed_plans.contains(dep))
        });
        if let Some(idx) = ready_idx {
            let plan = self.merge_queue.remove(idx);
            self.merge_queue_entered_at.remove(&plan);
            self.merge_in_progress = true;
            self.currently_merging = Some(plan.clone());
            vec![ExecutorAction::MergePlanToBatch { plan }]
        } else {
            // A6: Log warning for entries stale > 15min
            for (plan, entered) in &self.merge_queue_entered_at {
                if self.merge_queue.contains(plan)
                    && entered.elapsed() > std::time::Duration::from_secs(15 * 60)
                {
                    info!(
                        "Merge queue warning: {} waiting for {}min",
                        plan,
                        entered.elapsed().as_secs() / 60
                    );
                }
            }
            vec![]
        }
    }

    /// Plan merged to batch — mark complete, drain merge queue, schedule more.
    pub fn handle_plan_merged(&mut self, plan: &str) -> Vec<ExecutorAction> {
        self.merge_in_progress = false;
        self.currently_merging = None;
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.phase = PlanPhase::Complete;
        }
        self.completed_plans.insert(plan.to_string());
        self.plans_since_refactor += 1;
        self.plans_since_integration_test += 1;

        info!(
            "Plan {} merged. {}/{} complete, {} since refactor, {} since itest",
            plan,
            self.completed_plans.len(),
            self.dag.plan_count(),
            self.plans_since_refactor,
            self.plans_since_integration_test,
        );

        let mut actions = vec![];

        if !self.express_mode {
            // Doc verification for this plan
            actions.push(ExecutorAction::SpawnDocVerifier {
                plan: plan.to_string(),
            });

            // Post-merge regression test (workspace-wide, non-blocking diagnostic)
            actions.push(ExecutorAction::RunPostMergeRegression {
                batch_branch: self.batch_branch.clone(),
            });
        }

        // Trigger refactoring if interval reached
        if self.refactor_interval > 0
            && self.plans_since_refactor >= self.refactor_interval
            && !self.refactoring_active
        {
            self.plans_since_refactor = 0;
            self.refactoring_active = true;
            actions.push(ExecutorAction::SpawnRefactorer {
                batch_branch: self.batch_branch.clone(),
            });
        }

        // Trigger integration tests if interval reached
        if self.integration_test_interval > 0
            && self.plans_since_integration_test >= self.integration_test_interval
            && !self.integration_testing_active
        {
            self.plans_since_integration_test = 0;
            self.integration_testing_active = true;
            actions.push(ExecutorAction::RunIntegrationTests {
                batch_branch: self.batch_branch.clone(),
            });
        }

        // Drain merge queue — a just-completed plan may unblock queued merges
        actions.extend(self.drain_merge_queue());

        // Schedule more work
        actions.extend(self.schedule_next());
        actions
    }

    /// Refactoring pass completed.
    pub fn handle_refactoring_complete(&mut self) {
        self.refactoring_active = false;
    }

    /// Integration test pass completed.
    pub fn handle_integration_tests_complete(&mut self) {
        self.integration_testing_active = false;
    }

    /// When an invariant fix changes an upstream crate's API, downstream
    /// completed plans need re-gating. This finds completed plans that depend
    /// on the failed plan and re-runs gates (not full re-implementation).
    pub fn handle_invariant_cascade(&mut self, failed_plan: &str) -> Vec<ExecutorAction> {
        let dependents: Vec<String> = self
            .completed_plans
            .iter()
            .filter(|plan| {
                let deps = self.dag.plan_dependencies(plan);
                deps.contains(&failed_plan.to_string())
            })
            .cloned()
            .collect();

        if dependents.is_empty() {
            return vec![];
        }

        info!(
            "Invariant cascade from {}: re-gating {} dependent plans",
            failed_plan,
            dependents.len()
        );

        let mut actions = Vec::new();
        for plan in dependents {
            // Move dependent back to Gating phase (not full re-implementation)
            if let Some(state) = self.plan_states.get_mut(&plan) {
                state.phase = PlanPhase::Gating;
            }
            // Remove from completed so it gets re-verified
            self.completed_plans.remove(&plan);
            actions.push(ExecutorAction::ReGatePlan { plan });
        }

        actions
    }

    /// Is the entire run finished?
    pub fn is_complete(&self) -> bool {
        let total = self.dag.plan_count();
        let completed = self.completed_plans.len();

        if completed >= total {
            // Sanity check: don't declare complete if there are plans still in-flight
            // or in the merge queue. This guards against DAG plan_count undercount.
            let in_flight = self.in_flight_tasks.len();
            let merging = self.merge_queue.len() + if self.merge_in_progress { 1 } else { 0 };
            let has_active_plans = self
                .plan_states
                .values()
                .any(|ps| !matches!(ps.phase, PlanPhase::Complete | PlanPhase::Failed(_)));

            if in_flight > 0 || merging > 0 || has_active_plans {
                tracing::warn!(
                    "is_complete: completed_plans({}) >= dag.plan_count({}) but {} tasks in-flight, \
                     {} in merge queue, active_plans={}. NOT declaring complete.",
                    completed, total, in_flight, merging, has_active_plans,
                );
                return false;
            }

            tracing::info!(
                "is_complete: {}/{} plans done, no in-flight work. Run complete.",
                completed,
                total,
            );
            true
        } else {
            false
        }
    }

    /// Progress stats for the TUI.
    /// Run integrity checks on executor state. Returns a list of issues found
    /// and auto-fixes what it can. Called periodically and on restore.
    pub fn integrity_check(&mut self) -> Vec<String> {
        let mut issues = Vec::new();

        // 1. in_flight tasks that are also completed
        let stale_in_flight: Vec<GlobalTaskId> = self
            .in_flight_tasks
            .keys()
            .filter(|gid| self.completed_tasks.contains(gid))
            .cloned()
            .collect();
        for gid in &stale_in_flight {
            self.in_flight_tasks.remove(gid);
        }
        if !stale_in_flight.is_empty() {
            let msg = format!(
                "Cleared {} stale in_flight entries (already completed)",
                stale_in_flight.len()
            );
            warn!("{}", msg);
            issues.push(msg);
        }

        // 2. Plans in completed_plans that don't have Complete phase
        for plan in &self.completed_plans {
            if let Some(state) = self.plan_states.get(plan) {
                if !matches!(state.phase, PlanPhase::Complete) {
                    let msg = format!(
                        "Plan {} in completed_plans but phase={:?} — fixing to Complete",
                        plan, state.phase
                    );
                    warn!("{}", msg);
                    issues.push(msg);
                }
            }
            // Set phase to Complete
            self.plan_states.insert(
                plan.clone(),
                PlanState {
                    plan_base: plan.clone(),
                    phase: PlanPhase::Complete,
                    started_at: Instant::now(),
                    iteration: self.plan_states.get(plan).map(|s| s.iteration).unwrap_or(1),
                    active_reviewer_instance: None,
                    primary_agent_instance: None,
                },
            );
        }

        // 3. Plans with Complete phase not in completed_plans
        let phase_complete: Vec<String> = self
            .plan_states
            .iter()
            .filter(|(_, s)| matches!(s.phase, PlanPhase::Complete))
            .map(|(k, _)| k.clone())
            .collect();
        for plan in phase_complete {
            if !self.completed_plans.contains(&plan) {
                let msg = format!(
                    "Plan {} has Complete phase but not in completed_plans — adding",
                    plan
                );
                warn!("{}", msg);
                issues.push(msg);
                self.completed_plans.insert(plan);
            }
        }

        // 4. Plans with in-flight tasks but no plan_state
        let inflight_plans: HashSet<String> = self
            .in_flight_tasks
            .keys()
            .map(|gid| gid.plan.clone())
            .collect();
        for plan in &inflight_plans {
            if !self.plan_states.contains_key(plan) && !self.completed_plans.contains(plan) {
                let msg = format!(
                    "Plan {} has in-flight tasks but no plan_state — setting to Implementing",
                    plan
                );
                warn!("{}", msg);
                issues.push(msg);
                self.plan_states.insert(
                    plan.clone(),
                    PlanState {
                        plan_base: plan.clone(),
                        phase: PlanPhase::Implementing,
                        started_at: Instant::now(),
                        iteration: 1,
                        active_reviewer_instance: None,
                        primary_agent_instance: None,
                    },
                );
            }
        }

        if issues.is_empty() {
            debug!("Integrity check passed — no issues found");
        } else {
            info!(
                "Integrity check: {} issues found and auto-fixed",
                issues.len()
            );
        }
        issues
    }

    /// Generate a summary of executor state for conductor visibility.
    pub fn state_summary(&self) -> String {
        let mut lines = Vec::new();

        // Failed plans
        let failed: Vec<(&str, &str)> = self
            .plan_states
            .iter()
            .filter_map(|(k, v)| match &v.phase {
                PlanPhase::Failed(reason) => Some((k.as_str(), reason.as_str())),
                _ => None,
            })
            .collect();
        if !failed.is_empty() {
            lines.push(format!(
                "FAILED PLANS ({}): {}",
                failed.len(),
                failed
                    .iter()
                    .map(|(p, r)| format!("{p}: {r}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }

        // Plans stuck in Implementing with all tasks done
        let all_plans = self.dag.all_plans();
        for plan in &all_plans {
            if self.completed_plans.contains(*plan) {
                continue;
            }
            if let Some(state) = self.plan_states.get(*plan) {
                if matches!(state.phase, PlanPhase::Implementing) {
                    let all_done = self
                        .dag
                        .all_plan_tasks_complete(plan, &self.completed_tasks);
                    let any_inflight = self.in_flight_tasks.keys().any(|gid| gid.plan == *plan);
                    if all_done && !any_inflight {
                        lines.push(format!("STUCK: {plan} all tasks done but still Implementing (should advance to Gating)"));
                    }
                }
            }
        }

        // Plans with high failure counts
        let mut plan_failures: HashMap<&str, u32> = HashMap::new();
        for (gid, count) in &self.task_failure_count {
            let entry = plan_failures.entry(gid.plan.as_str()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        for (plan, count) in &plan_failures {
            if *count >= 2 {
                lines.push(format!(
                    "HIGH_FAILURES: {plan} max task failure count = {count}/5"
                ));
            }
        }

        // Progress summary
        let progress = self.progress();
        lines.push(format!(
            "Progress: {}/{} tasks, {}/{} plans",
            progress.completed_tasks,
            progress.total_tasks,
            progress.completed_plans,
            progress.total_plans
        ));

        lines.join("\n")
    }

    pub fn progress(&self) -> ExecutorProgress {
        ExecutorProgress {
            total_plans: self.dag.plan_count(),
            completed_plans: self.completed_plans.len(),
            total_tasks: self.dag.node_count(),
            completed_tasks: self.completed_tasks.len(),
            in_flight_tasks: self.in_flight_tasks.len(),
            plans_since_refactor: self.plans_since_refactor,
        }
    }

    /// Get the current phase of a plan.
    pub fn plan_phase(&self, plan: &str) -> Option<&PlanPhase> {
        self.plan_states.get(plan).map(|s| &s.phase)
    }

    /// Get the iteration count for a plan.
    pub fn plan_iteration(&self, plan: &str) -> u32 {
        self.plan_states.get(plan).map(|s| s.iteration).unwrap_or(1)
    }

    /// Store review feedback for a plan (used in re-implementation prompts).
    pub fn store_review_feedback(&mut self, plan: &str, feedback: String) {
        let entries = self.review_feedback.entry(plan.to_string()).or_default();
        entries.push(feedback);
        // Cap at 3 entries per plan (matches max iterations)
        while entries.len() > 3 {
            entries.remove(0);
        }
    }

    /// Get stored review feedback for a plan.
    pub fn get_review_feedback(&self, plan: &str) -> &[String] {
        self.review_feedback
            .get(plan)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Which plans are currently active (have in-flight tasks or are gating/reviewing)?
    pub fn active_plans(&self) -> Vec<&str> {
        self.plan_states
            .iter()
            .filter(|(_, s)| !matches!(s.phase, PlanPhase::Complete | PlanPhase::Failed(_)))
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get all in-flight task instance IDs.
    pub fn in_flight_instances(&self) -> &HashMap<GlobalTaskId, String> {
        &self.in_flight_tasks
    }

    /// Get names of all completed plans.
    pub fn completed_plan_names(&self) -> Vec<String> {
        self.completed_plans.iter().cloned().collect()
    }

    /// Snapshot for persistence.
    pub fn snapshot(&self) -> ExecutorSnapshot {
        ExecutorSnapshot {
            completed_tasks: self.completed_tasks.iter().map(|t| t.to_string()).collect(),
            // Only persist in_flight tasks that aren't also completed
            // (prevents stale overlap after crash recovery)
            in_flight_tasks: self
                .in_flight_tasks
                .iter()
                .filter(|(k, _)| !self.completed_tasks.contains(k))
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            completed_plans: self.completed_plans.iter().cloned().collect(),
            plan_phases: self
                .plan_states
                .iter()
                .map(|(k, v)| (k.clone(), v.phase.clone()))
                .collect(),
            plan_iterations: self
                .plan_states
                .iter()
                .map(|(k, v)| (k.clone(), v.iteration))
                .collect(),
            merge_queue: self.merge_queue.clone(),
            plans_since_refactor: self.plans_since_refactor,
            plans_since_integration_test: self.plans_since_integration_test,
            review_feedback: self.review_feedback.clone(),
        }
    }

    /// Restore from a persisted snapshot.
    pub fn restore(&mut self, snapshot: ExecutorSnapshot) -> Result<()> {
        self.completed_tasks = snapshot
            .completed_tasks
            .iter()
            .filter_map(|s| GlobalTaskId::parse(s))
            .collect();

        self.in_flight_tasks.clear();
        // Don't restore in_flight — those agents are dead after a crash.
        // They'll be re-scheduled by schedule_next().

        self.completed_plans = snapshot.completed_plans.into_iter().collect();
        self.merge_queue = snapshot.merge_queue;
        self.plans_since_refactor = snapshot.plans_since_refactor;
        self.plans_since_integration_test = snapshot.plans_since_integration_test;
        self.review_feedback = snapshot.review_feedback;

        // Ensure all tasks of completed plans are in completed_tasks so that
        // cross-plan task-level dependencies resolve correctly when scheduling
        // the next plan. This covers the case where completed_tasks in the
        // snapshot is incomplete (e.g. from a prior sequential run).
        for plan in &self.completed_plans {
            for task_gid in self.dag.tasks_for_plan(plan) {
                self.completed_tasks.insert(task_gid.clone());
            }
        }

        // Rebuild plan states from snapshot phases + iterations
        for (plan, phase) in &snapshot.plan_phases {
            let iteration = snapshot.plan_iterations.get(plan).copied().unwrap_or(1);
            self.plan_states.insert(
                plan.clone(),
                PlanState {
                    plan_base: plan.clone(),
                    phase: phase.clone(),
                    started_at: Instant::now(), // approximate
                    iteration,
                    active_reviewer_instance: None,
                    primary_agent_instance: None,
                },
            );
        }

        // Recover plans that have completed tasks but are missing from plan_phases.
        // This happens when a plan was reset (RETRY-PLAN removed it from plan_states)
        // but still has work tracked in the DAG. Set them to Implementing so they
        // get rescheduled.
        let tracked_plans: HashSet<String> = snapshot
            .plan_phases
            .keys()
            .cloned()
            .chain(self.completed_plans.iter().cloned())
            .collect();
        let all_dag_plans = self.dag.all_plans();
        let mut recovered = 0;
        for plan in &all_dag_plans {
            let plan_s = plan.to_string();
            if !tracked_plans.contains(&plan_s) {
                // Check if this plan has any completed tasks — if so, it was in progress
                let has_tasks = self.completed_tasks.iter().any(|gid| gid.plan == plan_s);
                if has_tasks {
                    info!(
                        plan = plan,
                        "Recovering orphaned plan — has completed tasks but no phase entry"
                    );
                    self.plan_states.insert(
                        plan_s.clone(),
                        PlanState {
                            plan_base: plan_s,
                            phase: PlanPhase::Implementing,
                            started_at: Instant::now(),
                            iteration: 1,
                            active_reviewer_instance: None,
                            primary_agent_instance: None,
                        },
                    );
                    recovered += 1;
                }
            }
        }

        info!(
            "Executor restored: {} completed tasks, {} completed plans, {} in merge queue, {} orphaned plans recovered",
            self.completed_tasks.len(),
            self.completed_plans.len(),
            self.merge_queue.len(),
            recovered,
        );

        // Run integrity checks and auto-fix any issues
        let issues = self.integrity_check();
        if !issues.is_empty() {
            for issue in &issues {
                warn!("Restore integrity: {}", issue);
            }
        }

        Ok(())
    }

    /// Pre-planning is now handled offline by bardo-enrich.sh before any agent runs.
    /// This function is retained for API compatibility but always returns empty.
    fn schedule_pre_planning(&self) -> Vec<ExecutorAction> {
        vec![]
    }

    /// Record that pre-planning is done for a plan.
    pub fn mark_pre_planned(&mut self, plan: &str) {
        self.pre_planned.insert(plan.to_string());
    }

    /// Iterate over all completed task IDs.
    pub fn completed_tasks_iter(&self) -> impl Iterator<Item = &GlobalTaskId> {
        self.completed_tasks.iter()
    }

    pub fn in_flight_task_ids(&self) -> impl Iterator<Item = &GlobalTaskId> {
        self.in_flight_tasks.keys()
    }

    /// Fully reset a plan so it can be re-queued from scratch.
    /// Returns the list of in-flight task IDs that were cleared (caller should kill agents).
    pub fn reset_plan(&mut self, plan: &str) -> Vec<GlobalTaskId> {
        self.completed_plans.remove(plan);
        self.merge_queue.retain(|p| p != plan);
        self.plan_states.remove(plan);
        let in_flight: Vec<GlobalTaskId> = self
            .in_flight_tasks
            .keys()
            .filter(|gid| gid.plan == plan)
            .cloned()
            .collect();
        for gid in &in_flight {
            self.in_flight_tasks.remove(gid);
        }
        let plan_tasks: Vec<GlobalTaskId> =
            self.dag.tasks_for_plan(plan).into_iter().cloned().collect();
        for tid in &plan_tasks {
            self.completed_tasks.remove(tid);
        }
        self.review_feedback.remove(plan);
        in_flight
    }

    /// Clear all in-flight task records (called on startup — all agent
    /// processes from a previous run are dead after restart).
    pub fn clear_all_in_flight(&mut self) {
        let count = self.in_flight_tasks.len();
        self.in_flight_tasks.clear();
        if count > 0 {
            info!("Cleared {} stale in-flight tasks from previous run", count);
        }
    }

    /// Re-verify a plan: move it back to the Gating phase without re-running
    /// implementation tasks. This re-runs gates and reviews only.
    pub fn reverify_plan(&mut self, plan: &str) -> Vec<ExecutorAction> {
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.phase = PlanPhase::Gating;
        }
        // Remove from merge queue if it was waiting to merge
        self.merge_queue.retain(|p| p != plan);
        vec![ExecutorAction::RunPlanGates {
            plan: plan.to_string(),
        }]
    }

    /// Replace the DAG (after strategist rewrites task TOMLs).
    /// Preserves all other executor state; clears completed/in-flight tasks
    /// for the affected plan so the new task IDs can be scheduled.
    pub fn replace_dag(&mut self, dag: UnifiedTaskDag, affected_plan: &str) {
        let old_count = self.dag.node_count();
        let new_count = dag.node_count();
        self.dag = dag;

        // Clear tasks for the affected plan so new IDs are schedulable
        let stale: Vec<GlobalTaskId> = self
            .completed_tasks
            .iter()
            .filter(|gid| gid.plan == affected_plan)
            .cloned()
            .collect();
        for gid in &stale {
            self.completed_tasks.remove(gid);
        }
        let stale_inflight: Vec<GlobalTaskId> = self
            .in_flight_tasks
            .keys()
            .filter(|gid| gid.plan == affected_plan)
            .cloned()
            .collect();
        for gid in &stale_inflight {
            self.in_flight_tasks.remove(gid);
        }

        info!(
            "DAG rebuild: old={} new={} tasks for {affected_plan} (cleared {} completed, {} in-flight)",
            old_count, new_count, stale.len(), stale_inflight.len()
        );
    }

    /// Architect said REVISE — route through strategist before re-implementing.
    pub fn handle_plan_revise(&mut self, plan: &str) -> Vec<ExecutorAction> {
        let iteration = if let Some(state) = self.plan_states.get_mut(plan) {
            state.iteration += 1;
            state.phase = PlanPhase::Implementing;
            state.iteration
        } else {
            1
        };
        let plan_tasks: Vec<GlobalTaskId> =
            self.dag.tasks_for_plan(plan).into_iter().cloned().collect();
        let cleared = plan_tasks.len();
        for tid in &plan_tasks {
            self.completed_tasks.remove(tid);
        }
        let feedback_len = self
            .review_feedback
            .get(plan)
            .map(|v| v.iter().map(|s| s.len()).sum::<usize>())
            .unwrap_or(0);
        info!(
            "handle_plan_revise: {plan} iter={iteration} cleared={cleared} tasks, feedback={feedback_len} chars"
        );
        // Offline enrichment replaced strategist — go straight to schedule_next()
        self.schedule_next()
    }

    /// Update the total active agent count (called from event loop before schedule_next).
    pub fn set_total_active_agents(&mut self, count: usize) {
        self.total_active_agents = count;
    }

    /// Check if the agent budget allows spawning another agent.
    pub fn can_spawn_more(&self) -> bool {
        self.total_active_agents < self.max_concurrent_agents
    }

    /// Record that a utility agent was spawned (pre-planner, refactorer, etc.)
    pub fn track_utility_agent(&mut self, instance_id: String) {
        self.utility_agents.insert(instance_id);
    }

    /// Remove a utility agent from tracking (completed or failed).
    pub fn untrack_utility_agent(&mut self, instance_id: &str) {
        self.utility_agents.remove(instance_id);
    }

    /// Remove a task from in-flight WITHOUT marking it completed, so it can be retried
    /// on the next `schedule_next()` call. Use for spawn/turn failures where no work was done.
    /// Resolve an instance_id back to a GlobalTaskId.
    pub fn task_for_instance(&self, instance_id: &str) -> Option<GlobalTaskId> {
        // Skip _pending_ sentinels — those are budget placeholders, not real agents.
        // Real agent iids (strategist:, express-impl:, auto-fix:, …) must not match them.
        self.in_flight_tasks
            .iter()
            .find(|(_, v)| v.as_str() == instance_id && !v.starts_with("_pending_:"))
            .map(|(k, _)| k.clone())
    }

    /// Return (completed, in_flight, total) task counts for a plan.
    /// Returns None if the plan has no tasks in the DAG.
    pub fn task_progress_for_plan(&self, plan: &str) -> Option<(usize, usize, usize)> {
        let plan_tasks = self.dag.tasks_for_plan(plan);
        if plan_tasks.is_empty() {
            return None;
        }
        let total = plan_tasks.len();
        let completed = plan_tasks
            .iter()
            .filter(|t| self.completed_tasks.contains(*t))
            .count();
        let in_flight = plan_tasks
            .iter()
            .filter(|t| self.in_flight_tasks.contains_key(*t))
            .count();
        Some((completed, in_flight, total))
    }

    /// Resolve all GlobalTaskIds for a given instance_id.
    /// Returns multiple tasks when the instance was spawned via SpawnTaskAgentBatch.
    pub fn tasks_for_instance(&self, instance_id: &str) -> Vec<GlobalTaskId> {
        self.in_flight_tasks
            .iter()
            .filter(|(_, v)| v.as_str() == instance_id && !v.starts_with("_pending_:"))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Mark all tasks for an instance as complete and return follow-up actions.
    /// Works for both single-task and batch instances.
    pub fn handle_instance_complete(&mut self, instance_id: &str) -> Vec<ExecutorAction> {
        let tasks = self.tasks_for_instance(instance_id);
        if tasks.is_empty() {
            return self.schedule_next();
        }

        let mut actions = Vec::new();
        let mut gated_plans: HashSet<String> = HashSet::new();

        for task_id in &tasks {
            self.in_flight_tasks.remove(task_id);
            self.completed_tasks.insert(task_id.clone());
            info!(task = %task_id, "Task completed");
        }

        // Emit RunPlanGates once per plan whose tasks are now all done.
        for task_id in &tasks {
            if gated_plans.contains(&task_id.plan) {
                continue;
            }
            if self
                .dag
                .all_plan_tasks_complete(&task_id.plan, &self.completed_tasks)
            {
                if let Some(state) = self.plan_states.get_mut(&task_id.plan) {
                    if matches!(state.phase, PlanPhase::Implementing | PlanPhase::Failed(_)) {
                        state.phase = PlanPhase::Gating;
                        gated_plans.insert(task_id.plan.clone());
                        info!(
                            "All tasks complete for plan {}, running gates",
                            task_id.plan
                        );
                        actions.extend(self.emit_gates_with_warm_reviewer(&task_id.plan));
                    }
                }
            }
        }

        actions.extend(self.schedule_next());
        actions
    }

    /// Mark all tasks for an instance as failed and return follow-up actions.
    /// Works for both single-task and batch instances.
    pub fn handle_instance_failed(&mut self, instance_id: &str) -> Vec<ExecutorAction> {
        let tasks = self.tasks_for_instance(instance_id);
        if tasks.is_empty() {
            return self.schedule_next();
        }

        // Remove all tasks from in_flight and increment failure count for each.
        // Delegate plan-failure logic to handle_task_failed for the lead task.
        let lead = tasks[0].clone();
        for task_id in &tasks[1..] {
            self.in_flight_tasks.remove(task_id);
            // Also remove from completed_tasks so it can be retried
            self.completed_tasks.remove(task_id);
            let count = self.task_failure_count.entry(task_id.clone()).or_insert(0);
            *count += 1;
        }
        self.handle_task_failed(lead)
    }

    /// Reset a plan from Failed state so it can be retried from a completely clean state.
    /// Removes worktree + branch, clears all task state, and reschedules from scratch.
    pub fn retry_failed_plan(&mut self, plan: &str) -> Vec<ExecutorAction> {
        match self.plan_states.get(plan) {
            Some(state) if matches!(state.phase, PlanPhase::Failed(_)) => {}
            _ => {
                info!(
                    plan = plan,
                    "RETRY-PLAN ignored — plan is not in Failed state"
                );
                return vec![];
            }
        }

        // Remove the plan state entirely so it starts fresh
        self.plan_states.remove(plan);

        // Clear ALL completed tasks for this plan
        let plan_tasks: Vec<GlobalTaskId> = self
            .completed_tasks
            .iter()
            .filter(|gid| gid.plan == plan)
            .cloned()
            .collect();
        for gid in plan_tasks {
            self.completed_tasks.remove(&gid);
        }

        // Clear in-flight tasks
        let in_flight: Vec<GlobalTaskId> = self
            .in_flight_tasks
            .keys()
            .filter(|gid| gid.plan == plan)
            .cloned()
            .collect();
        for gid in in_flight {
            self.in_flight_tasks.remove(&gid);
        }

        // Clear task failure counts for this plan
        let task_keys: Vec<GlobalTaskId> = self
            .task_failure_count
            .keys()
            .filter(|gid| gid.plan == plan)
            .cloned()
            .collect();
        for key in task_keys {
            self.task_failure_count.remove(&key);
        }

        // Clear spawn backoff
        self.spawn_failure_counts.remove(plan);
        self.spawn_blocked_until.remove(plan);

        // Remove from completed_plans
        self.completed_plans.remove(plan);

        // Remove from merge queue
        self.merge_queue.retain(|p| p != plan);
        self.merge_queue_entered_at.remove(plan);

        info!(
            plan = plan,
            "RETRY-PLAN — full clean reset, removing worktree + branch"
        );

        // Emit CleanRetryPlan action (handled in parallel loop to clean worktree/branch)
        // followed by schedule_next to pick up the now-unblocked plan
        let mut actions = vec![ExecutorAction::CleanRetryPlan {
            plan: plan.to_string(),
        }];
        actions.extend(self.schedule_next());
        actions
    }
}

/// Progress stats for the TUI.
#[derive(Debug, Clone)]
pub struct ExecutorProgress {
    pub total_plans: usize,
    pub completed_plans: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_flight_tasks: usize,
    pub plans_since_refactor: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan::{PlanFrontmatter, PlanInfo};
    use crate::orchestrator::tasks::{Task, TaskFile, TaskMeta, TaskStatus};
    use std::path::PathBuf;

    fn make_plan(base: &str, depends_on: Vec<&str>) -> PlanInfo {
        PlanInfo {
            base: base.to_string(),
            num: base.split('-').next().unwrap_or(base).to_string(),
            path: PathBuf::from(format!("plans/{base}.md")),
            frontmatter: Some(PlanFrontmatter {
                plan: Some(base.to_string()),
                depends_on: depends_on.into_iter().map(String::from).collect(),
                parallel_with: Vec::new(),
                crates_touched: Vec::new(),
                estimated_tasks: None,
                estimated_parallel_width: None,
                estimated_minutes: Some(30),
                refactor_after: false,
                parallel_safe: true,
                tasks: vec![],
            }),
        }
    }

    fn make_task(id: &str, deps: &[&str], files: &[&str]) -> Task {
        Task {
            id: id.to_string(),
            title: format!("Task {id}"),
            status: TaskStatus::Pending,
            files: files.iter().map(|s| s.to_string()).collect(),
            acceptance: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            parallel_group: None,
            exclusive_files: true,
            estimated_minutes: Some(10),
            types_to_define: None,
            formulas: None,
            test_invariants: None,
            imports: None,
            example_pattern: None,
            context_files: None,
            plan_section: None,
            skills: None,
        }
    }

    fn make_task_file(plan: &str, tasks: Vec<Task>) -> TaskFile {
        TaskFile {
            meta: TaskMeta {
                plan: plan.to_string(),
                iteration: 1,
                total: tasks.len(),
                done: 0,
                max_parallel: Some(3),
                estimated_total_minutes: None,
            },
            tasks,
        }
    }

    #[test]
    fn executor_schedules_independent_tasks() {
        let plans = vec![make_plan("01-alpha", vec![]), make_plan("02-beta", vec![])];
        let mut task_files = HashMap::new();
        task_files.insert(
            "01-alpha".to_string(),
            make_task_file(
                "01-alpha",
                vec![
                    make_task("T1", &[], &["crates/alpha/src/a.rs"]),
                    make_task("T2", &[], &["crates/alpha/src/b.rs"]),
                ],
            ),
        );
        task_files.insert(
            "02-beta".to_string(),
            make_task_file(
                "02-beta",
                vec![make_task("T1", &[], &["crates/beta/src/a.rs"])],
            ),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 6, 5, "codex/batch/test".to_string());

        let actions = executor.schedule_next();
        // Should schedule 2 batches (one per plan, 2 tasks in 01-alpha + 1 in 02-beta)
        let batch_count = actions
            .iter()
            .filter(|a| matches!(a, ExecutorAction::SpawnTaskAgentBatch { .. }))
            .count();
        // Also accept the old single-task variant for backward compat in tests
        let single_count = actions
            .iter()
            .filter(|a| matches!(a, ExecutorAction::SpawnTaskAgent { .. }))
            .count();
        let total_spawns = batch_count + single_count;
        assert!(
            total_spawns > 0,
            "expected at least one spawn action, got none"
        );
        // Verify the total task count across all batches is 3
        let total_tasks: usize = actions
            .iter()
            .map(|a| match a {
                ExecutorAction::SpawnTaskAgentBatch { tasks, .. } => tasks.len(),
                ExecutorAction::SpawnTaskAgent { .. } => 1,
                _ => 0,
            })
            .sum();
        assert_eq!(total_tasks, 3);
    }

    #[test]
    fn executor_respects_budget() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let mut task_files = HashMap::new();
        task_files.insert(
            "01-alpha".to_string(),
            make_task_file(
                "01-alpha",
                vec![
                    make_task("T1", &[], &["a.rs"]),
                    make_task("T2", &[], &["b.rs"]),
                    make_task("T3", &[], &["c.rs"]),
                ],
            ),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 2, 0, "codex/batch/test".to_string());

        let actions = executor.schedule_next();
        // With budget=2, only 2 tasks should be scheduled (as one batch with 2 tasks)
        let total_tasks: usize = actions
            .iter()
            .map(|a| match a {
                ExecutorAction::SpawnTaskAgentBatch { tasks, .. } => tasks.len(),
                ExecutorAction::SpawnTaskAgent { .. } => 1,
                _ => 0,
            })
            .sum();
        assert_eq!(total_tasks, 2); // budget is 2
    }

    #[test]
    fn executor_triggers_gates_when_plan_done() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let mut task_files = HashMap::new();
        task_files.insert(
            "01-alpha".to_string(),
            make_task_file("01-alpha", vec![make_task("T1", &[], &["a.rs"])]),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 4, 0, "codex/batch/test".to_string());

        let tid = GlobalTaskId {
            plan: "01-alpha".to_string(),
            task: "T1".to_string(),
        };
        executor.record_task_started(tid.clone(), "impl:01:T1".to_string());
        let actions = executor.handle_task_complete(tid);

        let has_gates = actions
            .iter()
            .any(|a| matches!(a, ExecutorAction::RunPlanGates { plan } if plan == "01-alpha"));
        assert!(has_gates);
    }

    #[test]
    fn executor_tracks_completion() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let mut task_files = HashMap::new();
        task_files.insert(
            "01-alpha".to_string(),
            make_task_file("01-alpha", vec![make_task("T1", &[], &["a.rs"])]),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 4, 0, "codex/batch/test".to_string());

        assert!(!executor.is_complete());

        let tid = GlobalTaskId {
            plan: "01-alpha".to_string(),
            task: "T1".to_string(),
        };
        executor.record_task_started(tid.clone(), "impl:01:T1".to_string());
        executor.handle_task_complete(tid);
        executor.handle_plan_gates_passed("01-alpha");
        executor.handle_plan_reviews_passed("01-alpha");
        executor.handle_plan_merged("01-alpha");

        assert!(executor.is_complete());
    }

    #[test]
    fn executor_refactoring_trigger() {
        let plans = vec![
            make_plan("01-a", vec![]),
            make_plan("02-b", vec![]),
            make_plan("03-c", vec![]),
        ];
        let mut task_files = HashMap::new();
        for plan in &plans {
            task_files.insert(
                plan.base.clone(),
                make_task_file(
                    &plan.base,
                    vec![make_task("T1", &[], &[&format!("{}.rs", plan.base)])],
                ),
            );
        }

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 4, 2, "codex/batch/test".to_string());

        // Complete 2 plans — should trigger refactoring
        for (i, plan) in ["01-a", "02-b"].iter().enumerate() {
            let tid = GlobalTaskId {
                plan: plan.to_string(),
                task: "T1".to_string(),
            };
            executor.record_task_started(tid.clone(), format!("impl:{i}:T1"));
            executor.handle_task_complete(tid);
            executor.handle_plan_gates_passed(plan);
            executor.handle_plan_reviews_passed(plan);
            let actions = executor.handle_plan_merged(plan);

            if i == 1 {
                let has_refactor = actions
                    .iter()
                    .any(|a| matches!(a, ExecutorAction::SpawnRefactorer { .. }));
                assert!(has_refactor, "Should trigger refactoring after 2 plans");
            }
        }
    }

    #[test]
    fn executor_snapshot_restore() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let mut task_files = HashMap::new();
        task_files.insert(
            "01-alpha".to_string(),
            make_task_file(
                "01-alpha",
                vec![
                    make_task("T1", &[], &["a.rs"]),
                    make_task("T2", &["T1"], &["b.rs"]),
                ],
            ),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor = ParallelExecutor::new(dag, 4, 0, "codex/batch/test".to_string());

        let tid = GlobalTaskId {
            plan: "01-alpha".to_string(),
            task: "T1".to_string(),
        };
        executor.record_task_started(tid.clone(), "impl:01:T1".to_string());
        executor.handle_task_complete(tid);

        let snap = executor.snapshot();
        assert_eq!(snap.completed_tasks.len(), 1);
        assert!(snap.completed_tasks.contains(&"01-alpha:T1".to_string()));

        // Restore into a fresh executor
        let dag2 = UnifiedTaskDag::from_plans(&plans, &task_files).unwrap();
        let mut executor2 = ParallelExecutor::new(dag2, 4, 0, "codex/batch/test".to_string());
        executor2.restore(snap).unwrap();
        assert_eq!(executor2.completed_tasks.len(), 1);

        // T2 should now be runnable (either as single or batch)
        let actions = executor2.schedule_next();
        let has_t2 = actions.iter().any(|a| match a {
            ExecutorAction::SpawnTaskAgent { task_id, .. } => task_id.task == "T2",
            ExecutorAction::SpawnTaskAgentBatch { tasks, .. } => {
                tasks.iter().any(|t| t.task == "T2")
            }
            _ => false,
        });
        assert!(has_t2);
    }
}

// ============================================================================
// Gate-review overlap and warm pool helpers
// ============================================================================
//
// GATE-REVIEW OVERLAP PATTERN
// ===========================
//
// When all plan tasks are complete, schedule_next() emits BOTH:
// 1. RunPlanGates { plan } — run cargo check/clippy/tests asynchronously on gate_rx
// 2. PreSpawnWarmReviewer { plan } — pre-spawn warm reviewer (no turn_start yet)
//
// Event loop flow:
// 1. execute_actions() receives both actions
// 2. RunPlanGates spawns async gate tasks (as before)
// 3. PreSpawnWarmReviewer handler:
//    - Calls pool.pre_spawn_warm(QuickReviewer or Architect, ...)
//    - Records instance_id in executor.set_active_reviewer(plan, instance_id)
// 4. When gates complete on gate_rx:
//    - If PASSED: emit StartReviewerInParallel, which calls pool.turn_start()
//      on the warm reviewer to begin its turn
//    - If FAILED: emit CancelActiveReviewer action, which turn_interrupt + kill_instance,
//      then proceed to re-impl or AutoFix
//
// Use executor.get_active_reviewer(plan) to retrieve the instance_id when
// gates complete and need to route to turn_start or cancellation.

impl ParallelExecutor {
    /// Record that a reviewer is now running in overlap with gates for this plan.
    /// Called when gates are dispatched AND the reviewer is immediately started.
    pub fn set_active_reviewer(&mut self, plan: &str, instance_id: String) {
        if let Some(state) = self.plan_states.get_mut(plan) {
            // Guard: do not overwrite active reviewer without cleanup
            if state.active_reviewer_instance.is_some() {
                warn!("Plan {plan} — active reviewer already set; overwriting without cleanup (race condition)");
            }
            state.active_reviewer_instance = Some(instance_id);
            info!("Plan {plan} — reviewer starting in overlap with gates");
        }
    }

    /// Clear the active reviewer for a plan (called when reviewer completes or is cancelled).
    pub fn clear_active_reviewer(&mut self, plan: &str) {
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.active_reviewer_instance = None;
        }
    }

    /// Set the primary implementer agent for a plan (stays warm across phases).
    pub fn set_primary_agent(&mut self, plan: &str, instance_id: String) {
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.primary_agent_instance = Some(instance_id);
        }
    }

    /// Get the primary implementer agent instance ID if one is alive for this plan.
    pub fn get_primary_agent(&self, plan: &str) -> Option<String> {
        self.plan_states
            .get(plan)
            .and_then(|s| s.primary_agent_instance.clone())
    }

    /// Clear the primary agent for a plan (called on Complete or Failed).
    pub fn clear_primary_agent(&mut self, plan: &str) {
        if let Some(state) = self.plan_states.get_mut(plan) {
            state.primary_agent_instance = None;
        }
    }

    /// Get the active reviewer instance ID if one is running for this plan.
    pub fn get_active_reviewer(&self, plan: &str) -> Option<String> {
        self.plan_states
            .get(plan)
            .and_then(|s| s.active_reviewer_instance.clone())
    }

    /// Emit both RunPlanGates and PreSpawnWarmReviewer for gate-review overlap.
    /// Used when all plan tasks are complete and gates should start.
    pub fn emit_gates_with_warm_reviewer(&self, plan: &str) -> Vec<ExecutorAction> {
        vec![
            ExecutorAction::RunPlanGates {
                plan: plan.to_string(),
            },
            ExecutorAction::PreSpawnWarmReviewer {
                plan: plan.to_string(),
            },
        ]
    }
}

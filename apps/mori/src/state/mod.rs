pub mod config;
pub mod history;
pub mod persistence;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use ratatui::text::Line;

use crate::agent::AgentRole;

/// Cached parsed output lines to avoid re-parsing every frame.
/// The render function checks if `last_len` matches `output.len()`;
/// if so, it reuses `lines` instead of re-parsing.
#[derive(Debug, Clone, Default)]
pub struct CachedRender {
    pub last_len: usize,
    pub lines: Vec<Line<'static>>,
}

/// Tracks consecutive scroll events for acceleration.
/// The longer you hold a scroll key, the faster it scrolls.
#[derive(Debug, Clone)]
pub struct ScrollAccel {
    pub last_scroll: Instant,
    pub consecutive: u32,
}

impl Default for ScrollAccel {
    fn default() -> Self {
        Self {
            last_scroll: Instant::now(),
            consecutive: 0,
        }
    }
}

impl ScrollAccel {
    /// Call on each scroll event. Returns a multiplier (1..=8).
    /// Resets if more than 300ms since the last scroll.
    pub fn tick(&mut self) -> usize {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_scroll);
        if elapsed.as_millis() > 300 {
            self.consecutive = 0;
        }
        self.consecutive += 1;
        self.last_scroll = now;
        match self.consecutive {
            1..=5 => 1,
            6..=15 => 2,
            16..=30 => 4,
            _ => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlanDetailTab {
    #[default]
    Summary,
    PlanDetails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyStatus {
    Pending,
    Running,
    Passed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct VerifyEntry {
    pub plan_base: String,
    pub plan_num: String,
    pub status: VerifyStatus,
    pub output: String,
    pub started: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub created: Instant,
    pub ttl_secs: u64,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentPaneGroup {
    #[default]
    Implementation,
    Verification,
}

/// Which sub-tab is active in the dashboard detail panel
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DetailSubTab {
    #[default]
    Agents,
    Output,
    Diff,
    Errors,
    Git,
}

/// Input mode for the TUI
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Inject,
    Filter,
    Confirm,
}

/// A destructive action waiting for user confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    RestartAllPlans,
    RestartPhase,
    ResetSelectedPlan(String),
    ForceAdvance(String),
    ReverifyPlan(String),
    GitReconcile,
    IngestTask {
        plan_num: String,
        task_id: String,
    },
    /// Merge the batch branch into main
    MergeBatchToMain {
        batch_branch: String,
        /// Plans completed in this batch
        plan_count: usize,
        /// Plans that failed (informational)
        failed_count: usize,
        /// Short hash of batch branch HEAD
        last_commit: String,
    },
}

impl ConfirmAction {
    pub fn title(&self) -> &str {
        match self {
            Self::RestartAllPlans => "Restart ALL plans",
            Self::RestartPhase => "Restart current phase",
            Self::ResetSelectedPlan(_) => "Reset plan",
            Self::ForceAdvance(_) => "Force advance",
            Self::ReverifyPlan(_) => "Re-verify plan",
            Self::GitReconcile => "Git Reconcile",
            Self::IngestTask { .. } => "Ingest task",
            Self::MergeBatchToMain { .. } => "Merge batch → main",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::RestartAllPlans => "Kill all agents and restart every plan from scratch. All progress will be lost.".to_string(),
            Self::RestartPhase => "Restart the current phase for the selected plan. Implementation will be reset to re-run this phase.".to_string(),
            Self::ResetSelectedPlan(p) => format!("Reset plan {p} from scratch. Deletes worktree, branch, and all progress for this plan."),
            Self::ForceAdvance(p) => format!("Force commit and advance past plan {p}. Skips remaining gates and reviews."),
            Self::ReverifyPlan(p) => format!("Re-run gates and reviews for plan {p}. Implementation is kept, only verification restarts."),
            Self::GitReconcile => "Reconcile all plan branches: commit outstanding work, merge unmerged plans to batch, prune stale worktrees, and merge batch to staging. Safe to run multiple times.".to_string(),
            Self::IngestTask { plan_num, task_id } =>
                format!("Reset task {task_id} (plan {plan_num}) to pending so the pipeline re-processes it."),
            Self::MergeBatchToMain { batch_branch, plan_count, failed_count, last_commit } => {
                let failed_note = if *failed_count > 0 {
                    format!("  ⚠ {failed_count} plan(s) failed — they will NOT be included.\n")
                } else {
                    String::new()
                };
                format!(
                    "{failed_note}  Branch:  {batch_branch}\n  Plans:   {plan_count} completed\n  HEAD:    {last_commit}\n\n  This merges {batch_branch} into main with --no-ff.\n  Cannot be undone without a force-push."
                )
            }
        }
    }
}

/// Whether the pipeline is running or paused
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PipelineRunState {
    #[default]
    Running,
    Paused,
}

/// Which panel/zone has focus in the pipeline view
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusZone {
    #[default]
    Plans,
    Tasks,
    AgentOutput,
    CommandOutput,
}

/// Which stage of the review pipeline a plan is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewStage {
    ReviewerPending,
    ScribePending,
    DocRevisionScribePending,
    CriticPending,
}

/// Status of an individual plan within the orchestrator runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPlanStatus {
    Pending,
    Active,
    Completed,
    CompletedPrior,
    Skipped,
    Failed,
    /// Plan's batch branch has been merged into main
    MergedToMain,
}

/// Record of a batch-to-main merge event
#[derive(Debug, Clone)]
pub struct MainMergeRecord {
    pub batch_branch: String,
    pub merge_commit: String,
    /// Unix timestamp of the merge
    pub merged_at: u64,
    /// Plan bases included in this batch
    pub plan_bases: Vec<String>,
}

/// Orchestrator-side plan entry
#[derive(Debug, Clone)]
pub struct RunPlanEntry {
    pub base: String,
    pub num: String,
    pub status: RunPlanStatus,
    pub iteration: u32,
    pub phase: String,
    pub estimated_minutes: Option<u32>,
    pub actual_minutes: Option<u32>,
    pub started_at: Option<Instant>,
    /// Short branch name: strips "codex/plan/" prefix (e.g. "04-terminal-scaffold")
    pub git_branch_short: Option<String>,
    /// Unix timestamp of last commit in the plan's worktree
    pub git_last_commit_secs: Option<u64>,
    /// Uncommitted lines: (added, removed) from `git diff --stat HEAD`
    pub git_dirty: Option<(u32, u32)>,
    /// Unix timestamp when this plan's batch was merged to main
    pub merged_to_main_at: Option<u64>,
    /// Short commit hash of the merge-to-main commit
    pub merge_commit: Option<String>,
}

/// Per-instance state for parallel execution display
#[derive(Debug, Clone)]
pub struct ParallelAgentState {
    pub instance_id: String,
    pub role: AgentRole,
    pub plan: String,
    pub task: String,
    /// Last ~4KB of streamed output for the live tail
    pub output: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Cumulative cost for this agent instance in USD.
    pub cost_usd: f64,
    pub active: bool,
    pub finished_at: Option<std::time::Instant>,
    /// Model slug used for this agent (e.g. "gpt-5.4", "cursor-fast")
    pub model: String,
    /// Whether turn_start has been called (agent is processing but no output yet).
    pub turn_started: bool,
    /// Cached parsed render lines — avoids re-parsing output every frame.
    pub render_cache: RefCell<CachedRender>,
}

/// Per-agent state for display
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    pub output: String,
    pub diff: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub active: bool,
    pub thread_id: Option<String>,
    pub turn_count: u32,
    pub current_plan: Option<String>,
    pub current_task: Option<String>,
    /// Cached parsed render lines — avoids re-parsing output every frame.
    pub render_cache: RefCell<CachedRender>,
}

/// Pending approval request
#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub role: AgentRole,
    pub command: String,
    /// Raw JSON-RPC id from the server-initiated request (string or number).
    pub approval_id: serde_json::Value,
}

/// Tracks estimated and actual durations to provide a continuously-updating ETA.
///
/// The estimator maintains:
/// - Static estimates from plan frontmatter / task metadata
/// - Actual observed durations from completed plans/tasks/phases
/// - A correction factor (exponential moving average of actual/estimated ratios)
///   that makes future estimates more accurate as the run progresses
#[derive(Debug, Clone)]
pub struct TimeEstimator {
    /// Correction factor: actual_time / estimated_time averaged across completed work.
    /// Starts at 1.0 (trust the estimates) and adjusts as plans complete.
    pub correction_factor: f64,

    /// Plan estimates from frontmatter (plan_base -> estimated_minutes)
    pub plan_estimates: HashMap<String, u32>,

    /// Actual durations for completed plans (plan_base -> actual_minutes)
    pub plan_actuals: HashMap<String, u32>,

    /// Per-task estimates (plan_base::task_id -> estimated_minutes)
    pub task_estimates: HashMap<String, u32>,

    /// Per-task actuals (plan_base::task_id -> actual_minutes)
    pub task_actuals: HashMap<String, u32>,

    /// Phase-type averages: how long each phase typically takes
    /// (e.g. "strategist" -> average minutes across all completed strategist phases)
    pub phase_averages: HashMap<String, f64>,

    /// Number of samples per phase type (for running average)
    pub phase_counts: HashMap<String, u32>,
}

impl Default for TimeEstimator {
    fn default() -> Self {
        Self {
            correction_factor: 1.0,
            plan_estimates: HashMap::new(),
            plan_actuals: HashMap::new(),
            task_estimates: HashMap::new(),
            task_actuals: HashMap::new(),
            phase_averages: HashMap::new(),
            phase_counts: HashMap::new(),
        }
    }
}

impl TimeEstimator {
    /// Record a completed plan and update the correction factor.
    pub fn record_plan_complete(&mut self, plan_base: &str, actual_minutes: u32) {
        self.plan_actuals
            .insert(plan_base.to_string(), actual_minutes);
        self.update_correction_factor();
    }

    /// Record a completed task.
    pub fn record_task_complete(&mut self, plan_base: &str, task_id: &str, actual_minutes: u32) {
        let key = format!("{plan_base}::{task_id}");
        self.task_actuals.insert(key, actual_minutes);
    }

    /// Record a completed phase duration and update the phase average.
    pub fn record_phase_complete(&mut self, phase: &str, actual_minutes: f64) {
        let count = self.phase_counts.entry(phase.to_string()).or_default();
        let avg = self.phase_averages.entry(phase.to_string()).or_default();
        *count += 1;
        // Running average
        *avg = *avg + (actual_minutes - *avg) / *count as f64;
    }

    /// Load static estimates from plan frontmatter.
    pub fn load_plan_estimate(&mut self, plan_base: &str, estimated_minutes: u32) {
        self.plan_estimates
            .insert(plan_base.to_string(), estimated_minutes);
    }

    /// Load task estimates.
    pub fn load_task_estimate(&mut self, plan_base: &str, task_id: &str, estimated_minutes: u32) {
        let key = format!("{plan_base}::{task_id}");
        self.task_estimates.insert(key, estimated_minutes);
    }

    /// Get corrected estimate for a plan (applies correction factor).
    pub fn corrected_plan_estimate(&self, plan_base: &str) -> u32 {
        let raw = self.plan_estimates.get(plan_base).copied().unwrap_or(15);
        (raw as f64 * self.correction_factor).ceil() as u32
    }

    /// Get estimated minutes remaining for all incomplete plans.
    /// Takes into account parallel execution waves if wave estimates are provided.
    pub fn estimated_remaining_minutes(&self, remaining_plans: &[String]) -> u32 {
        remaining_plans
            .iter()
            .map(|p| self.corrected_plan_estimate(p))
            .sum()
    }

    /// Get estimated remaining for a wave (max of plan estimates in that wave).
    pub fn estimated_wave_minutes(&self, wave_plans: &[String]) -> u32 {
        wave_plans
            .iter()
            .map(|p| self.corrected_plan_estimate(p))
            .max()
            .unwrap_or(15)
    }

    /// Get estimated remaining time for current plan based on phase averages
    /// and what phases are left.
    pub fn estimated_plan_remaining(&self, current_phase: &str, phases: &[&str]) -> f64 {
        let current_idx = phases.iter().position(|&p| p == current_phase).unwrap_or(0);
        phases[current_idx..]
            .iter()
            .map(|&p| self.phase_averages.get(p).copied().unwrap_or(3.0))
            .sum()
    }

    /// Get estimated remaining seconds with sub-second precision.
    /// Uses phase averages with interpolation from current phase progress.
    /// Wave-aware: parallel waves complete in max(plan_durations) not sum().
    pub fn estimated_remaining_seconds_precise(
        &self,
        remaining_plans: &[String],
        waves: Option<&[Vec<String>]>,
        current_phase: Option<&str>,
        phase_elapsed_secs: f64,
    ) -> f64 {
        // If wave information is available, use wave-aware estimation
        if let Some(waves) = waves {
            let mut total = 0.0;
            for wave in waves {
                // Parallel wave: time = max of plan estimates
                let wave_secs = wave
                    .iter()
                    .map(|p| self.corrected_plan_estimate(p) as f64 * 60.0)
                    .fold(0.0f64, f64::max);
                total += wave_secs;
                // Add ~30s merge time per plan in wave
                total += wave.len() as f64 * 30.0;
            }
            return total;
        }

        // Sequential: sum of remaining plan estimates
        let mut total_secs: f64 = remaining_plans
            .iter()
            .map(|p| self.corrected_plan_estimate(p) as f64 * 60.0)
            .sum();

        // Subtract progress in current phase
        if let Some(phase) = current_phase {
            let phase_avg_secs = self.phase_averages.get(phase).copied().unwrap_or(3.0) * 60.0;
            let progress = (phase_elapsed_secs / phase_avg_secs).min(1.0);
            total_secs -= phase_avg_secs * progress;
        }

        total_secs.max(0.0)
    }

    /// Update correction factor from completed plan data.
    /// Uses exponential moving average with alpha=0.3 (recent data weighted more).
    fn update_correction_factor(&mut self) {
        let ratios: Vec<f64> = self
            .plan_actuals
            .iter()
            .filter_map(|(plan, &actual)| {
                let estimated = self.plan_estimates.get(plan).copied()?;
                if estimated == 0 {
                    return None;
                }
                Some(actual as f64 / estimated as f64)
            })
            .collect();

        if ratios.is_empty() {
            return;
        }

        // Exponential moving average of ratios
        let alpha = 0.3;
        let mut ema = ratios[0];
        for &r in &ratios[1..] {
            ema = alpha * r + (1.0 - alpha) * ema;
        }
        self.correction_factor = ema.clamp(0.5, 3.0); // Don't let it drift too far
    }
}

/// Format a duration in seconds as a human-readable string (e.g. "3m42s", "42s", "1h05m12s").
pub fn format_duration(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}h{minutes:02}m{seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m{seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

/// Format a duration in minutes as a human-readable string. Wrapper around format_duration.
pub fn format_duration_minutes(minutes: u32) -> String {
    format_duration(minutes as u64 * 60)
}

/// Format fractional seconds into human-readable duration.
pub fn format_duration_f64(total_secs: f64) -> String {
    format_duration(total_secs.ceil() as u64)
}

/// Format a unix timestamp as "X ago" relative to now.
pub fn format_ago(commit_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let elapsed = now.saturating_sub(commit_secs);
    match elapsed {
        0..=59 => format!("{elapsed}s ago"),
        60..=3599 => format!("{}m ago", elapsed / 60),
        3600..=86399 => format!("{}h ago", elapsed / 3600),
        _ => format!("{}d ago", elapsed / 86400),
    }
}

/// Estimate cost from token counts when the gateway isn't providing cost headers.
pub fn estimate_cost_from_tokens(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
    let (input_per_m, output_per_m) = match model {
        m if m.contains("opus") => (15.0, 75.0),
        m if m.contains("sonnet") => (3.0, 15.0),
        m if m.contains("haiku") => (0.8, 4.0),
        _ => (3.0, 15.0), // default to sonnet pricing
    };
    (input_tokens as f64 * input_per_m / 1_000_000.0)
        + (output_tokens as f64 * output_per_m / 1_000_000.0)
}

/// Format a USD cost compactly: $0.00, $1.23, $12.40
pub fn fmt_cost(usd: f64) -> String {
    if usd < 0.01 {
        "$0.00".to_string()
    } else if usd < 100.0 {
        format!("${:.2}", usd)
    } else {
        format!("${:.0}", usd)
    }
}

/// Runtime state shared between orchestrator, TUI, and event loop
#[derive(Debug)]
pub struct RunState {
    pub orchestrator_state: String,
    pub plans: Vec<RunPlanEntry>,
    pub current_plan_idx: usize,
    pub current_iteration: u32,
    pub current_phase: String,
    pub agents: HashMap<AgentRole, AgentState>,
    pub active_tab: usize,
    pub selected_plan_idx: usize,
    pub selected_agent_tab: usize,
    pub phase_started: Option<Instant>,
    pub run_started: Instant,
    pub pending_approval: Option<PendingApproval>,
    pub log_messages: Vec<LogEntry>,
    pub log_scroll: usize,
    pub last_gate_output: String,
    pub input_mode: InputMode,
    pub message_input: String,
    pub complete: bool,
    pub error: Option<String>,
    /// Track which reviewers are still running in the current review phase
    pub pending_reviews: HashSet<AgentRole>,
    /// Conductor intervention history for TUI display
    pub conductor_history: Vec<ConductorHistoryEntry>,
    /// Which zone has focus
    pub focus: FocusZone,
    /// Agent output scroll position: None = auto-scroll, Some(offset) = pinned
    pub agent_scroll: Option<usize>,
    /// Gates currently running (e.g. "cargo clippy (01-foo)"). Empty when idle.
    pub gate_running: HashSet<String>,
    /// Context token limit for agents
    pub context_limit: u64,
    /// Diff panel scroll position: None = auto-scroll, Some(offset) = pinned
    pub diff_scroll: Option<usize>,
    /// Branch-level diff (updated after commits)
    pub branch_diff: String,
    /// Scroll position in the task list
    pub task_scroll: usize,
    /// Which task is expanded to show detail
    pub task_expanded: Option<String>,
    /// Whether plan detail modal is visible
    pub show_plan_detail: bool,
    /// Content of the plan detail modal
    pub plan_detail_content: String,
    /// Scroll position in the plan detail modal
    pub plan_detail_scroll: usize,
    /// Whether user manually switched agent tab (suppresses auto-tab)
    pub manual_agent_tab: bool,
    /// Elapsed times for completed phases
    pub phase_elapsed: Vec<(String, std::time::Duration)>,
    /// Display string for the currently active task
    pub active_task_display: Option<String>,
    /// Task checklist (moved from app.rs local)
    pub task_checklist: Option<crate::orchestrator::tasks::TaskChecklist>,
    /// Whether help modal is visible
    pub show_help: bool,
    /// Whether wave overview modal is visible
    pub show_wave_overview: bool,
    /// Whether agent pool modal is visible
    pub show_agent_pool_modal: bool,
    /// Repo root path for reading plan files
    pub repo_root: Option<std::path::PathBuf>,
    /// Interactive config state
    pub config: config::ConfigState,
    /// Current git branch name
    pub git_branch: String,
    /// Unix timestamp of last commit in the main repo worktree (for status bar)
    pub git_last_commit_secs: Option<u64>,
    /// Gate/terminal command output (streamed or full)
    pub command_output: String,
    /// Scroll position in command output panel: None = auto-scroll
    pub command_output_scroll: Option<usize>,
    /// sccache stats string (e.g. "sccache: 87% (423/487)")
    pub sccache_stats: Option<String>,
    /// DX preflight warnings
    pub preflight_warnings: Vec<String>,
    /// Why the current iteration is happening (filled on iter > 1)
    pub iteration_reason: String,
    /// How many consecutive REVISE verdicts (resets on APPROVE or commit)
    pub consecutive_revise_count: u32,
    /// Which tab is active in the plan detail modal
    pub plan_detail_tab: PlanDetailTab,
    /// Executive summary content for the selected plan
    pub plan_summary_content: String,
    /// Scroll position in the summary tab
    pub plan_summary_scroll: usize,
    /// Retroactive verification entries
    pub verify_entries: Vec<VerifyEntry>,
    /// Selected verification entry index
    pub selected_verify_idx: usize,
    /// Scroll offset for the plan list (first visible plan index)
    pub plan_scroll_offset: usize,
    /// Toast notifications
    pub notifications: Vec<Notification>,
    /// Which group of agents the pane shows
    pub agent_pane_group: AgentPaneGroup,
    /// Code verdict from architect+auditor ("APPROVE" or "REVISE")
    pub code_verdict: String,
    /// Doc verdict from critic ("APPROVE", "REVISE", or "")
    pub doc_verdict: String,
    /// Per-role count of auto-responses sent in the current phase (resets on phase transition)
    pub auto_respond_count: HashMap<AgentRole, u32>,
    /// Whether the last gate (compile/test) passed
    pub last_gate_passed: bool,
    /// Previous token counts per agent (for flash detection)
    pub token_prev: HashMap<AgentRole, (u64, u64)>,
    /// Remaining flash frames per agent (counts down each tick)
    pub token_flash: HashMap<AgentRole, u32>,
    /// How many doc revision iterations have occurred this plan
    pub doc_revision_count: u32,
    /// Time estimation tracker
    pub time_estimator: TimeEstimator,
    /// Wave structure for parallel plans (filled from DAG)
    pub execution_waves: Vec<(usize, Vec<String>)>,
    /// Current wave index
    pub current_wave: usize,
    /// Terminal height (captured each frame for scroll calculations)
    pub terminal_height: u16,
    /// Pending particle burst coordinates (x, y) — consumed by the draw loop
    pub particle_burst_pending: Option<(f32, f32)>,
    /// Scroll acceleration: timestamp of last scroll key, consecutive count
    pub scroll_accel: ScrollAccel,
    /// Active detail panel sub-tab (dashboard)
    pub detail_sub_tab: DetailSubTab,
    /// Which waves are expanded in the plan tree (by wave index)
    pub wave_expanded: HashSet<usize>,
    /// Fuzzy filter text for plan tree
    pub filter_text: String,
    /// Whether the filter input is active
    pub filter_active: bool,
    /// Token burn history per agent (ring buffer of input_tokens snapshots, sampled each tick)
    pub token_burn_history: HashMap<AgentRole, VecDeque<u64>>,
    /// Whether the task detail modal is visible
    pub show_task_detail: bool,
    /// Scroll position in the task detail modal
    pub task_detail_scroll: usize,
    /// Whether the task picker modal is visible
    pub show_task_picker: bool,
    /// Cursor position in the task picker modal
    pub task_picker_cursor: usize,
    /// Selected wave index for Plans browser view
    pub selected_wave_idx: usize,
    /// Cursor in the agent list (Agents view)
    pub agent_list_cursor: usize,
    /// Cursor in the git branch tree
    pub git_branch_cursor: usize,
    /// Git view mode: Log or Diff
    pub git_view_mode: GitViewMode,
    /// Computed branch hierarchy for git view
    pub git_branch_tree: Vec<GitTreeNode>,
    /// Cached commit graph lines (refreshed in background)
    pub git_commit_graph: Vec<String>,
    /// Cached worktree list (refreshed in background)
    pub git_worktree_list: Vec<crate::git::worktree::WorktreeEntry>,
    /// Whether a git reconcile operation is currently running
    pub git_reconcile_in_progress: bool,
    /// On-demand loaded task checklists for non-active plans
    pub plan_task_cache: HashMap<String, crate::orchestrator::tasks::TaskChecklist>,
    /// Per-instance state for all active parallel agents
    pub parallel_agents: Vec<ParallelAgentState>,
    /// When each plan pipeline was created (for ETA correction)
    pub plan_start_times: HashMap<String, std::time::Instant>,
    /// Per-plan gate output (compile/clippy output per plan in parallel mode)
    pub plan_gate_outputs: HashMap<String, String>,
    /// When each plan's current phase started (for per-plan timeline in parallel mode)
    pub plan_phase_started: HashMap<String, std::time::Instant>,
    /// Reviewers still pending per plan in parallel review phase (arch/audit/scribe → then critic)
    pub plan_pending_reviews: HashMap<String, HashSet<AgentRole>>,
    /// Which stage of the review pipeline each plan is in
    pub plan_review_stage: HashMap<String, ReviewStage>,
    /// Per-plan doc revision count (critic/scribe cycles) — caps at 2 then auto-approves
    pub plan_doc_revisions: HashMap<String, u32>,
    /// Retry count for review agents that exited unexpectedly ("prefix:plan" → count)
    pub plan_agent_retries: HashMap<String, u32>,
    /// When true, the Plans view shows the pipeline-level meta-agent overview
    /// instead of a specific plan's detail.
    pub pipeline_header_selected: bool,
    /// Task IDs (as "plan:task" strings) completed by the executor.
    /// Overlays Done on TOML Pending when they differ (e.g. resumed from prior run).
    pub executor_completed_tasks: HashSet<String>,
    /// Executor state summary for conductor visibility — updated on each checkpoint.
    pub executor_state_summary: String,
    /// When each task was started (key: "plan:task"), for countdown display.
    pub task_started_at: HashMap<String, std::time::Instant>,
    /// System resource snapshot (CPU / mem / disk), updated ~1/s
    pub sys: crate::sys_metrics::SysSnapshot,
    /// When each parallel agent instance was spawned (for conductor turn duration tracking)
    pub turn_started_at: HashMap<String, Instant>,
    /// Pending confirmation for a destructive action
    pub pending_confirm: Option<ConfirmAction>,
    /// Whether the pipeline is paused or running
    pub pipeline_run_state: PipelineRunState,
    /// Corrective brief from conductor RESET_REVIEW — injected into restarted reviewers
    pub conductor_reset_brief: Option<String>,
    /// Tracks which plan/phase the conductor is currently validating (plan_base, phase_name)
    pub pending_phase_validation: Option<(String, String)>,
    /// C1: Per-plan code revision count (code REVISE verdicts)
    pub plan_code_revisions: HashMap<String, u32>,
    /// A4: Phase restart count (resets on phase transitions)
    pub phase_restart_count: u32,
    /// Pending inject message waiting for conductor routing decision
    pub pending_inject: Option<PendingInject>,
    /// Last time the conductor was consulted periodically during implementation
    pub last_periodic_conductor_consult: Option<Instant>,
    /// Target agent label for the currently open inject modal (e.g., "impl[s46]:01-ws:T1")
    pub steer_target: Option<String>,
    /// Roles queued for kill after a config model change (hot reload).
    /// Drained in the event loop after ConfigSelect.
    pub pending_agent_kills: Vec<AgentRole>,
    /// Whether commit_and_advance is currently in progress (guards against double-dispatch)
    pub committing_in_progress: bool,
    /// History of batch→main merge events
    pub main_merges: Vec<MainMergeRecord>,
    /// Cumulative cost across all agents in USD.
    pub cumulative_cost_usd: f64,
    /// Cost per plan in USD.
    pub cost_per_plan: HashMap<String, f64>,
}

/// A user inject message pending conductor routing decision
#[derive(Debug, Clone)]
pub struct PendingInject {
    pub message: String,
    pub target_role: Option<AgentRole>,
    pub target_instance_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GitViewMode {
    #[default]
    Log,
    Diff,
}

#[derive(Debug, Clone)]
pub struct GitTreeNode {
    pub name: String,
    pub short_name: String,
    pub depth: usize,
    pub plan_base: Option<String>,
    pub task_id: Option<String>,
    pub status: Option<RunPlanStatus>,
    pub is_current: bool,
    pub commit: String,
    pub worktree: Option<String>,
    pub is_last_sibling: bool,
}

#[derive(Debug, Clone)]
pub struct ConductorHistoryEntry {
    pub timestamp: String,
    pub watcher: String,
    pub target: String,
    pub message: String,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            orchestrator_state: "initializing".to_string(),
            plans: Vec::new(),
            current_plan_idx: 0,
            current_iteration: 1,
            current_phase: String::new(),
            agents: HashMap::new(),
            active_tab: 0,
            selected_plan_idx: 0,
            selected_agent_tab: 0,
            phase_started: None,
            run_started: Instant::now(),
            pending_approval: None,
            log_messages: Vec::new(),
            log_scroll: 0,
            last_gate_output: String::new(),
            input_mode: InputMode::Normal,
            message_input: String::new(),
            complete: false,
            error: None,
            pending_reviews: HashSet::new(),
            conductor_history: Vec::new(),
            focus: FocusZone::default(),
            agent_scroll: None,
            gate_running: HashSet::new(),
            context_limit: 200_000,
            diff_scroll: None,
            branch_diff: String::new(),
            task_scroll: 0,
            task_expanded: None,
            show_plan_detail: false,
            plan_detail_content: String::new(),
            plan_detail_scroll: 0,
            manual_agent_tab: false,
            phase_elapsed: Vec::new(),
            active_task_display: None,
            task_checklist: None,
            show_help: false,
            show_wave_overview: false,
            show_agent_pool_modal: false,
            repo_root: None,
            config: config::ConfigState::default(),
            git_branch: String::new(),
            git_last_commit_secs: None,
            command_output: String::new(),
            command_output_scroll: None,
            sccache_stats: None,
            preflight_warnings: Vec::new(),
            iteration_reason: String::new(),
            consecutive_revise_count: 0,
            plan_detail_tab: PlanDetailTab::default(),
            plan_summary_content: String::new(),
            plan_summary_scroll: 0,
            verify_entries: Vec::new(),
            selected_verify_idx: 0,
            plan_scroll_offset: 0,
            notifications: Vec::new(),
            agent_pane_group: AgentPaneGroup::default(),
            code_verdict: String::new(),
            doc_verdict: String::new(),
            auto_respond_count: HashMap::new(),
            last_gate_passed: false,
            token_prev: HashMap::new(),
            token_flash: HashMap::new(),
            doc_revision_count: 0,
            time_estimator: TimeEstimator::default(),
            execution_waves: Vec::new(),
            current_wave: 0,
            terminal_height: 0,
            particle_burst_pending: None,
            scroll_accel: ScrollAccel::default(),
            detail_sub_tab: DetailSubTab::default(),
            wave_expanded: HashSet::new(),
            filter_text: String::new(),
            filter_active: false,
            token_burn_history: HashMap::new(),
            show_task_detail: false,
            task_detail_scroll: 0,
            show_task_picker: false,
            task_picker_cursor: 0,
            selected_wave_idx: 0,
            agent_list_cursor: 0,
            git_branch_cursor: 0,
            git_view_mode: GitViewMode::default(),
            git_branch_tree: Vec::new(),
            git_commit_graph: Vec::new(),
            git_worktree_list: Vec::new(),
            git_reconcile_in_progress: false,
            plan_task_cache: HashMap::new(),
            parallel_agents: Vec::new(),
            plan_start_times: HashMap::new(),
            plan_gate_outputs: HashMap::new(),
            plan_phase_started: HashMap::new(),
            plan_pending_reviews: HashMap::new(),
            plan_review_stage: HashMap::new(),
            plan_doc_revisions: HashMap::new(),
            plan_agent_retries: HashMap::new(),
            pipeline_header_selected: false,
            executor_completed_tasks: HashSet::new(),
            executor_state_summary: String::new(),
            task_started_at: HashMap::new(),
            sys: crate::sys_metrics::SysSnapshot::default(),
            turn_started_at: HashMap::new(),
            pending_confirm: None,
            pipeline_run_state: PipelineRunState::default(),
            conductor_reset_brief: None,
            pending_phase_validation: None,
            plan_code_revisions: HashMap::new(),
            phase_restart_count: 0,
            pending_inject: None,
            last_periodic_conductor_consult: None,
            steer_target: None,
            pending_agent_kills: Vec::new(),
            committing_in_progress: false,
            main_merges: Vec::new(),
            cumulative_cost_usd: 0.0,
            cost_per_plan: HashMap::new(),
        }
    }
}

impl RunState {
    pub fn add_log(&mut self, source: &str, message: &str, level: LogLevel) {
        self.log_messages.push(LogEntry {
            timestamp: chrono::Utc::now(),
            source: source.to_string(),
            message: message.to_string(),
            level,
        });
    }

    pub fn current_plan(&self) -> Option<&RunPlanEntry> {
        self.plans.get(self.current_plan_idx)
    }

    pub fn agent_state(&self, role: AgentRole) -> Option<&AgentState> {
        self.agents.get(&role)
    }

    pub fn agent_state_mut(&mut self, role: AgentRole) -> &mut AgentState {
        self.agents.entry(role).or_default()
    }

    /// Returns true if any agent (sequential or parallel) is currently active.
    pub fn any_agent_active(&self) -> bool {
        self.agents.values().any(|a| a.active) || self.parallel_agents.iter().any(|p| p.active)
    }

    pub fn clear_agent_outputs(&mut self) {
        for state in self.agents.values_mut() {
            state.output.clear();
            state.diff.clear();
            state.active = false;
        }
    }

    /// Full reset for a new plan: clear outputs, tokens, command panel, phase timings.
    /// Called between plans so each plan starts with a clean slate.
    pub fn reset_for_new_plan(&mut self) {
        for state in self.agents.values_mut() {
            state.output.clear();
            state.diff.clear();
            state.active = false;
            state.input_tokens = 0;
            state.output_tokens = 0;
            state.thread_id = None;
            state.turn_count = 0;
            state.current_plan = None;
            state.current_task = None;
        }
        self.command_output.clear();
        self.gate_running.clear();
        self.last_gate_output.clear();
        self.phase_elapsed.clear();
        self.task_checklist = None;
        self.active_task_display = None;
        self.iteration_reason.clear();
        self.consecutive_revise_count = 0;
        self.agent_scroll = None;
        self.manual_agent_tab = false;
        self.code_verdict.clear();
        self.doc_verdict.clear();
        self.auto_respond_count.clear();
        self.last_gate_passed = false;
        self.token_prev.clear();
        self.token_flash.clear();
        self.doc_revision_count = 0;
    }

    /// Format remaining time as human-readable string (e.g. "2h14m", "45m", "3m").
    pub fn format_eta(&self) -> String {
        let remaining = self.estimated_remaining_minutes();
        format_duration_minutes(remaining)
    }

    /// Total estimated remaining minutes across all incomplete plans.
    pub fn estimated_remaining_minutes(&self) -> u32 {
        let remaining_plans: Vec<String> = self
            .plans
            .iter()
            .filter(|p| matches!(p.status, RunPlanStatus::Pending | RunPlanStatus::Active))
            .map(|p| p.base.clone())
            .collect();

        if self.execution_waves.is_empty() {
            // Sequential fallback
            self.time_estimator
                .estimated_remaining_minutes(&remaining_plans)
        } else {
            // Wave-aware: sum of remaining wave maxes
            let completed: HashSet<String> = self
                .plans
                .iter()
                .filter(|p| {
                    matches!(
                        p.status,
                        RunPlanStatus::Completed
                            | RunPlanStatus::CompletedPrior
                            | RunPlanStatus::MergedToMain
                    )
                })
                .map(|p| p.base.clone())
                .collect();

            self.execution_waves
                .iter()
                .filter(|(_, plans)| plans.iter().any(|p| !completed.contains(p)))
                .map(|(_, plans)| self.time_estimator.estimated_wave_minutes(plans))
                .sum()
        }
    }

    /// Total estimated remaining seconds across all incomplete plans.
    /// Subtracts elapsed time on active plans so the ETA actually counts down.
    /// In parallel mode, accounts for all in-flight plans (not just one).
    pub fn estimated_remaining_seconds(&self) -> u64 {
        let base_secs = self.estimated_remaining_minutes() as f64 * 60.0;
        let elapsed = self.run_started.elapsed().as_secs_f64();

        // Parallel mode: use wall-clock throughput for more accurate ETA
        if !self.parallel_agents.is_empty() || !self.plan_start_times.is_empty() {
            let (task_done, task_total) = self.task_weighted_progress();
            if task_total > 0 && task_done > 0 {
                let done_frac = task_done as f64 / task_total as f64;

                // Method 1: throughput-based (wall clock / done% = total time)
                // Only use if we've been running > 60s to avoid early noise
                let throughput_eta = if elapsed > 60.0 && done_frac > 0.01 {
                    let total_projected = elapsed / done_frac;
                    (total_projected - elapsed).max(0.0)
                } else {
                    f64::MAX
                };

                // Method 2: estimate-based (static estimates minus done fraction)
                let estimate_eta = base_secs * (1.0 - done_frac);

                // Blend: prefer throughput after we have enough data (>5% done),
                // otherwise lean on static estimates
                let eta = if done_frac > 0.05 && throughput_eta < f64::MAX {
                    // Weighted blend: 70% throughput, 30% estimate
                    throughput_eta * 0.7 + estimate_eta * 0.3
                } else if throughput_eta < f64::MAX {
                    // Early: 50/50
                    throughput_eta * 0.5 + estimate_eta * 0.5
                } else {
                    estimate_eta
                };

                return eta.max(0.0) as u64;
            }
            // No tasks done yet — use static estimate minus elapsed
            return (base_secs - elapsed).max(0.0) as u64;
        }

        // Sequential mode: deduct from the single current plan
        if let Some(plan) = self.current_plan() {
            let plan_elapsed = plan
                .started_at
                .map(|s| s.elapsed().as_secs_f64())
                .unwrap_or(0.0);

            let task_deduction = self
                .checklist_for_plan(&plan.base)
                .filter(|cl| cl.total() > 0)
                .map(|cl| {
                    let done_frac = cl.done_count() as f64 / cl.total() as f64;
                    let plan_est =
                        self.time_estimator.corrected_plan_estimate(&plan.base) as f64 * 60.0;
                    plan_est * done_frac
                })
                .unwrap_or(0.0);

            let deduction = plan_elapsed.max(task_deduction);
            return (base_secs - deduction).max(0.0) as u64;
        }
        (base_secs - elapsed).max(0.0) as u64
    }

    /// Look up checklist for any plan (live if matching, else cached).
    pub fn checklist_for_plan(
        &self,
        plan_base: &str,
    ) -> Option<&crate::orchestrator::tasks::TaskChecklist> {
        if let Some(ref cl) = self.task_checklist {
            // Match plan_num strictly against the numeric prefix in plan_base.
            // Formats: "b04-terminal-scaffold" or "04-terminal-scaffold" should match plan_num "04"
            let patterns = [
                format!("b{}-", cl.plan_num), // "b04-"
                format!("{}-", cl.plan_num),  // "04-"
                format!("b{}", cl.plan_num),  // "b04" (exact match for short base)
                cl.plan_num.clone(),          // "04" (exact match)
            ];

            for pattern in patterns {
                if plan_base.starts_with(&pattern) || plan_base == &pattern {
                    return Some(cl);
                }
            }
        }
        self.plan_task_cache.get(plan_base)
    }

    /// Checklist for the currently selected plan in the dashboard.
    pub fn selected_checklist(&self) -> Option<&crate::orchestrator::tasks::TaskChecklist> {
        let base = &self.plans.get(self.selected_plan_idx)?.base;
        self.checklist_for_plan(base)
    }

    /// (done_tasks, total_tasks) across all plans — task-weighted progress.
    pub fn task_weighted_progress(&self) -> (usize, usize) {
        let mut done = 0usize;
        let mut total = 0usize;
        let mut whole_hit = 0usize;
        let mut individual_hit = 0usize;
        let mut no_checklist = 0usize;
        for plan in &self.plans {
            // Check if the entire plan was completed via express mode (__whole__)
            let whole_key = format!("{}:__whole__", plan.base);
            let in_overlay = self.executor_completed_tasks.contains(&whole_key);
            let status_done = matches!(
                plan.status,
                RunPlanStatus::Completed
                    | RunPlanStatus::CompletedPrior
                    | RunPlanStatus::MergedToMain
            );
            let whole_done = in_overlay || status_done;

            if let Some(cl) = self.checklist_for_plan(&plan.base) {
                if whole_done {
                    // Express mode or completed plan — count all tasks as done
                    done += cl.total();
                    total += cl.total();
                    whole_hit += 1;
                } else {
                    // Count individual task completion
                    let d = cl
                        .tasks
                        .iter()
                        .filter(|t| {
                            matches!(t.status, crate::orchestrator::tasks::TaskStatus::Done)
                                || self
                                    .executor_completed_tasks
                                    .contains(&format!("{}:{}", plan.base, t.id))
                                || self
                                    .executor_completed_tasks
                                    .contains(&format!("{}:{}", cl.plan_num, t.id))
                        })
                        .count();
                    done += d;
                    if d > 0 {
                        individual_hit += 1;
                    }
                    total += cl.total();
                }
            } else {
                // No checklist — count as 1 task
                no_checklist += 1;
                if whole_done {
                    done += 1;
                }
                total += 1;
            }
        }
        // Only log at debug level since this is called on every frame render
        if whole_hit > 0 || individual_hit > 0 {
            tracing::debug!(
                "task_weighted_progress: {done}/{total} (whole_hit={whole_hit}, individual={individual_hit}, no_checklist={no_checklist})"
            );
        }
        (done, total)
    }

    /// (done_tasks, total_tasks) within a specific wave.
    pub fn wave_task_progress(&self, wave_idx: usize) -> (usize, usize) {
        let mut done = 0usize;
        let mut total = 0usize;
        if let Some((_, bases)) = self.execution_waves.get(wave_idx) {
            for base in bases {
                let whole_done = self
                    .executor_completed_tasks
                    .contains(&format!("{base}:__whole__"))
                    || self.plans.iter().any(|p| {
                        &p.base == base
                            && matches!(
                                p.status,
                                RunPlanStatus::Completed
                                    | RunPlanStatus::CompletedPrior
                                    | RunPlanStatus::MergedToMain
                            )
                    });

                if let Some(cl) = self.checklist_for_plan(base) {
                    if whole_done {
                        done += cl.total();
                    } else {
                        let d = cl
                            .tasks
                            .iter()
                            .filter(|t| {
                                matches!(t.status, crate::orchestrator::tasks::TaskStatus::Done)
                                    || self
                                        .executor_completed_tasks
                                        .contains(&format!("{}:{}", base, t.id))
                                    || self
                                        .executor_completed_tasks
                                        .contains(&format!("{}:{}", cl.plan_num, t.id))
                            })
                            .count();
                        done += d;
                    }
                    total += cl.total();
                } else {
                    if whole_done {
                        done += 1;
                    }
                    total += 1;
                }
            }
        }
        (done, total)
    }

    /// Load and cache the checklist for a plan if not already present.
    /// Call this when the user selects a different plan so task_progress has data.
    pub fn ensure_checklist_cached(&mut self, plan_base: &str) {
        // Already have it live or cached?
        if self.checklist_for_plan(plan_base).is_some() {
            return;
        }
        // Try to find the plan_num from the plans list
        let plan_num = self
            .plans
            .iter()
            .find(|p| p.base == plan_base)
            .map(|p| p.num.clone());
        let plan_num = match plan_num {
            Some(n) => n,
            None => return,
        };
        // Try loading from disk
        if let Some(ref root) = self.repo_root.clone() {
            if let Ok(Some(cl)) = crate::orchestrator::tasks::load_checklist(root, &plan_num) {
                self.plan_task_cache.insert(plan_base.to_string(), cl);
            }
        }
    }

    /// Estimated remaining for just the current plan.
    pub fn estimated_current_plan_remaining(&self) -> Option<u32> {
        let plan = self.current_plan()?;
        if let Some(started) = plan.started_at {
            let elapsed = started.elapsed().as_secs() / 60;
            let total_est = self.time_estimator.corrected_plan_estimate(&plan.base);
            Some(total_est.saturating_sub(elapsed as u32))
        } else {
            Some(self.time_estimator.corrected_plan_estimate(&plan.base))
        }
    }

    /// Snapshot the current state for crash reporting.
    /// Extracts only serializable fields, converting Instants to elapsed durations.
    pub fn snapshot_for_crash(&self) -> persistence::CrashAppState {
        let plan_statuses: Vec<(String, String)> = self
            .plans
            .iter()
            .map(|p| {
                let label = match p.status {
                    RunPlanStatus::Pending => "pending",
                    RunPlanStatus::Active => "active",
                    RunPlanStatus::Completed => "completed",
                    RunPlanStatus::CompletedPrior => "completed-prior",
                    RunPlanStatus::Skipped => "skipped",
                    RunPlanStatus::Failed => "failed",
                    RunPlanStatus::MergedToMain => "merged-to-main",
                };
                (p.base.clone(), label.to_string())
            })
            .collect();

        let active_agents: Vec<String> = self
            .agents
            .iter()
            .filter(|(_, s)| s.active)
            .map(|(role, _)| format!("{role:?}"))
            .collect();

        let n = self.log_messages.len();
        let start = n.saturating_sub(50);
        let recent_in_memory_logs: Vec<String> = self.log_messages[start..]
            .iter()
            .map(|e| format!("[{:?}] {}: {}", e.level, e.source, e.message))
            .collect();

        let gate_running: Vec<String> = self.gate_running.iter().cloned().collect();

        persistence::CrashAppState {
            orchestrator_state: self.orchestrator_state.clone(),
            current_plan: self.current_plan().map(|p| p.base.clone()),
            current_phase: self.current_phase.clone(),
            current_iteration: self.current_iteration,
            plan_statuses,
            active_agents,
            recent_in_memory_logs,
            gate_running,
            error: self.error.clone(),
        }
    }

    /// Resolve the highlighted agent list cursor to (role, instance_id, task).
    /// In parallel mode, filters and sorts agents by (active, role, task) and returns the agent at agent_list_cursor.
    /// Returns None if no agents are available or cursor is out of bounds.
    pub fn resolve_agent_list_cursor(&self) -> Option<(AgentRole, String, String)> {
        if !self.parallel_agents.is_empty() {
            // Parallel mode: sort by (active, role, task) like parallel_pool does
            let mut filtered: Vec<(usize, &ParallelAgentState)> =
                self.parallel_agents.iter().enumerate().collect();
            filtered.sort_by_key(|(_, p)| (!p.active, p.role.short().to_string(), p.task.clone()));

            // Get the agent at cursor position
            if let Some((_, agent)) = filtered.get(self.agent_list_cursor) {
                return Some((agent.role, agent.instance_id.clone(), agent.task.clone()));
            }
        }
        None
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CURRENT_STATE_VERSION: u32 = 2;

fn default_version() -> u32 {
    1
}

/// Status file format (backward compatible with run-plans.sh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusFile {
    #[serde(default = "default_version")]
    pub version: u32,
    pub run_id: String,
    pub batch_id: String,
    pub plans_total: u32,
    pub plans_completed: u32,
    pub plans_remaining: u32,
    pub current_plan: String,
    pub current_phase: String,
    pub current_iteration: u32,
    pub started_at: String,
    pub last_activity: String,
    pub pid: u32,
    pub anvil_pid: u32,
    pub hang_threshold_seconds: u32,
}

/// Event file line format (backward compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLine {
    pub ts: String,
    pub event: String,
    pub plan: String,
    pub phase: String,
    #[serde(default)]
    pub iter: u32,
}

/// Task-level state for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateFile {
    #[serde(default = "default_version")]
    pub version: u32,
    pub run_id: String,
    pub batch_branch: String,
    pub completed_tasks: Vec<String>, // "plan:task" format e.g. "09-chain-layer:T1"
    pub in_flight: HashMap<String, String>, // "plan:task" -> instance_id
    pub completed_plans: Vec<String>,
    pub total_tokens: TokenCount,
    /// Per-plan iteration counts for crash recovery (plan_base -> iteration)
    #[serde(default)]
    pub plan_iterations: HashMap<String, u32>,
    /// Plans waiting in the merge queue (dependency-ordered)
    #[serde(default)]
    pub merge_queue: Vec<String>,
    /// Plans since last refactoring pass
    #[serde(default)]
    pub plans_since_refactor: usize,
    /// Plans since last integration test
    #[serde(default)]
    pub plans_since_integration_test: usize,
    /// Active worktree paths (plan_base -> worktree path) for crash recovery
    #[serde(default)]
    pub active_worktrees: HashMap<String, String>,
    /// Per-plan phase strings (plan_base -> phase label) for crash recovery
    #[serde(default)]
    pub plan_phases: HashMap<String, String>,
    /// G2: In-progress merge checkpoint (plan name, worktree HEAD, batch ref, timestamp)
    #[serde(default)]
    pub merge_in_progress: Option<MergeCheckpoint>,
    /// Archived review feedback per plan (persisted for crash recovery)
    #[serde(default)]
    pub review_feedback: HashMap<String, Vec<String>>,
    /// TimeEstimator correction factor (persisted across restarts)
    #[serde(default)]
    pub correction_factor: Option<f64>,
}

/// G2: Checkpoint written before a merge, cleared after.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCheckpoint {
    pub plan: String,
    pub worktree_head: String,
    pub batch_ref: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
}

/// Task-level event for events.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub ts: String,
    pub event: String, // "task_start", "task_done", "plan_gates_passed", "plan_merged", etc
    pub plan: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<u64>,
}

pub struct PersistenceManager {
    log_dir: PathBuf,
    status_file: PathBuf,
    events_file: PathBuf,
}

impl PersistenceManager {
    pub fn new(repo_root: &Path) -> Self {
        let log_dir = crate::orchestrator::paths::runs_dir(repo_root);
        let status_file = log_dir.join("status.json");
        let events_file = log_dir.join("events.jsonl");
        Self {
            log_dir,
            status_file,
            events_file,
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.log_dir)?;
        std::fs::create_dir_all(self.log_dir.join("output"))?;
        Ok(())
    }

    /// Write the current status (atomic: write to .tmp then rename).
    /// G4: Also archives to status-archive/ keeping latest 10.
    pub fn write_status(&self, status: &StatusFile) -> Result<()> {
        let json = serde_json::to_string_pretty(status)?;
        atomic_write(&self.status_file, &json)?;

        // G4: Archive rotation
        let archive_dir = self.log_dir.join("status-archive");
        std::fs::create_dir_all(&archive_dir).ok();
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let archive_path = archive_dir.join(format!("status-{ts}.json"));
        std::fs::write(&archive_path, &json).ok();

        // Prune: keep only latest 10
        if let Ok(entries) = std::fs::read_dir(&archive_dir) {
            let mut files: Vec<std::path::PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .collect();
            files.sort();
            while files.len() > 10 {
                if let Some(old) = files.first() {
                    std::fs::remove_file(old).ok();
                }
                files.remove(0);
            }
        }

        Ok(())
    }

    /// Append an event line
    pub fn append_event(&self, event: &EventLine) -> Result<()> {
        use std::io::Write;
        let json = serde_json::to_string(event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_file)?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    /// Write the current plan name
    pub fn write_current_plan(&self, plan: &str) -> Result<()> {
        std::fs::write(self.log_dir.join("current-plan.txt"), plan)?;
        Ok(())
    }

    /// Write the run PID
    pub fn write_pid(&self) -> Result<()> {
        let pid = std::process::id();
        std::fs::write(self.log_dir.join("run.pid"), pid.to_string())?;
        Ok(())
    }

    /// Clean up PID file on exit
    pub fn cleanup_pid(&self) {
        let _ = std::fs::remove_file(self.log_dir.join("run.pid"));
    }

    /// Helper: create a status snapshot
    pub fn make_status(
        run_id: &str,
        batch_id: &str,
        plans_total: u32,
        plans_completed: u32,
        current_plan: &str,
        current_phase: &str,
        current_iteration: u32,
        started_at: &str,
    ) -> StatusFile {
        StatusFile {
            version: CURRENT_STATE_VERSION,
            run_id: run_id.to_string(),
            batch_id: batch_id.to_string(),
            plans_total,
            plans_completed,
            plans_remaining: plans_total.saturating_sub(plans_completed),
            current_plan: current_plan.to_string(),
            current_phase: current_phase.to_string(),
            current_iteration,
            started_at: started_at.to_string(),
            last_activity: chrono::Utc::now().to_rfc3339(),
            pid: std::process::id(),
            anvil_pid: 0,
            hang_threshold_seconds: 600,
        }
    }

    /// Helper: create an event line
    pub fn make_event(event: &str, plan: &str, phase: &str, iter: u32) -> EventLine {
        EventLine {
            ts: chrono::Utc::now().to_rfc3339(),
            event: event.to_string(),
            plan: plan.to_string(),
            phase: phase.to_string(),
            iter,
        }
    }

    /// Read all events from the events.jsonl file.
    pub fn read_events(&self) -> Result<Vec<EventLine>> {
        let content = match std::fs::read_to_string(&self.events_file) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        Ok(content
            .lines()
            .filter_map(|line| serde_json::from_str::<EventLine>(line).ok())
            .collect())
    }

    /// Get the set of plan bases that have a "plan_done" event.
    pub fn completed_plans(&self) -> Result<std::collections::HashSet<String>> {
        Ok(self
            .read_events()?
            .iter()
            .filter(|e| e.event == "plan_done")
            .map(|e| e.plan.clone())
            .collect())
    }

    /// Check if a stale PID file exists from a prior run.
    /// If the process is still running, attempt to kill it.
    pub fn check_stale_pid(&self) -> Option<u32> {
        let path = self.log_dir.join("run.pid");
        let pid_str = std::fs::read_to_string(&path).ok()?;
        let pid: u32 = pid_str.trim().parse().ok()?;

        let current_pid = std::process::id();
        if pid == current_pid {
            return None;
        }

        // Check if process is alive using kill -0
        let alive = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if alive {
            // Verify it's actually bardo-ctl before killing (PIDs get recycled)
            let is_bardo = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "comm="])
                .output()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .to_lowercase()
                        .contains("bardo")
                })
                .unwrap_or(false);

            if !is_bardo {
                // PID was recycled to a different process, don't kill it
                return None;
            }

            // Kill the stale process tree
            let _ = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output();
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
            // Kill leftover codex app-server processes
            let _ = std::process::Command::new("pkill")
                .args(["-f", "codex.*app-server"])
                .output();
            let _ = std::fs::remove_file(&path);
            return Some(pid);
        }

        // Process dead, clean up stale PID file
        let _ = std::fs::remove_file(&path);
        None
    }

    /// Write task-level state for crash recovery (atomic: write to .tmp then rename)
    pub fn write_task_state(&self, state: &TaskStateFile) -> Result<()> {
        let path = self.log_dir.join("task-state.json");
        let json = serde_json::to_string_pretty(state)?;
        atomic_write(&path, &json)
    }

    /// Load task-level state from disk (for crash recovery).
    /// Returns None if the file doesn't exist or is corrupted.
    pub fn load_task_state(&self) -> Result<Option<TaskStateFile>> {
        let path = self.log_dir.join("task-state.json");
        if !path.exists() {
            return Ok(None);
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to read task-state.json: {e}");
                return Ok(None);
            }
        };
        match serde_json::from_str::<TaskStateFile>(&content) {
            Ok(state) => {
                if state.version < CURRENT_STATE_VERSION {
                    tracing::warn!(
                        "task-state.json version {} is older than current {}, using serde defaults for new fields",
                        state.version, CURRENT_STATE_VERSION,
                    );
                }
                Ok(Some(state))
            }
            Err(e) => {
                tracing::warn!("Corrupted task-state.json, ignoring: {e}");
                // Rename the corrupted file so it doesn't block future runs
                let backup = self.log_dir.join("task-state.json.corrupt");
                let _ = std::fs::rename(&path, &backup);
                Ok(None)
            }
        }
    }

    /// Append a task-level event
    pub fn append_task_event(&self, event: &TaskEvent) -> Result<()> {
        use std::io::Write;
        let json = serde_json::to_string(event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_file)?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    /// Create a task event
    pub fn make_task_event(
        event: &str,
        plan: &str,
        task: Option<&str>,
        instance: Option<&str>,
        duration_secs: Option<u64>,
    ) -> TaskEvent {
        TaskEvent {
            ts: chrono::Utc::now().to_rfc3339(),
            event: event.to_string(),
            plan: plan.to_string(),
            task: task.map(|s| s.to_string()),
            instance: instance.map(|s| s.to_string()),
            duration_secs,
        }
    }

    /// Reconstruct completed tasks from events.jsonl (fallback when task-state.json is missing).
    /// Scans for "task_done" events and returns their plan:task identifiers.
    pub fn completed_tasks_from_events(&self) -> Result<Vec<String>> {
        let content = match std::fs::read_to_string(&self.events_file) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let mut tasks = Vec::new();
        for line in content.lines() {
            // Try parsing as TaskEvent first (has optional task field)
            if let Ok(te) = serde_json::from_str::<TaskEvent>(line) {
                if te.event == "task_done" {
                    if let Some(task) = te.task {
                        tasks.push(format!("{}:{}", te.plan, task));
                    }
                }
            }
        }
        Ok(tasks)
    }
}

// --- Crash Report types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub timestamp: String,
    pub crash_type: String,
    pub message: String,
    pub location: Option<String>,
    pub backtrace: String,
    pub error_signature: String,
    pub recent_logs: Vec<String>,
    pub app_state: CrashAppState,
    pub config_summary: String,
    pub environment: CrashEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrashAppState {
    pub orchestrator_state: String,
    pub current_plan: Option<String>,
    pub current_phase: String,
    pub current_iteration: u32,
    pub plan_statuses: Vec<(String, String)>,
    pub active_agents: Vec<String>,
    pub recent_in_memory_logs: Vec<String>,
    pub gate_running: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashEnvironment {
    pub rust_version: String,
    pub os: String,
    pub terminal_size: Option<(u16, u16)>,
    pub pid: u32,
}

/// Compute a 12-char hex error signature from message + location.
pub fn compute_error_signature(message: &str, location: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    hasher.update(location.as_bytes());
    let result = hasher.finalize();
    hex_encode(&result[..6])
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl PersistenceManager {
    /// Write a crash report (atomic).
    pub fn write_crash_report(&self, report: &CrashReport) -> Result<()> {
        let path = self.log_dir.join("crash-report.json");
        let json = serde_json::to_string_pretty(report)?;
        atomic_write(&path, &json)
    }

    /// Get the path to the output log file for a plan.
    pub fn output_log_path(&self, plan_base: &str) -> PathBuf {
        self.log_dir
            .join("output")
            .join(format!("{}.log", plan_base))
    }

    /// Append a line to the output log for a plan.
    /// Rotates the file when it exceeds ~2000 lines (keeps only the latest half).
    pub fn append_output_line(&self, plan_base: &str, line: &str) -> Result<()> {
        use std::io::Write;
        let path = self.output_log_path(plan_base);

        // Ensure output directory exists
        std::fs::create_dir_all(path.parent().unwrap_or(&self.log_dir))?;

        // Append the line
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{line}")?;

        // Check if rotation is needed (~2000 lines)
        if let Ok(content) = std::fs::read_to_string(&path) {
            let line_count = content.lines().count();
            if line_count > 2000 {
                // Keep only the latest half
                let keep_lines: Vec<&str> = content.lines().collect();
                let start_idx = keep_lines.len() / 2;
                let new_content = keep_lines[start_idx..].join("\n") + "\n";
                std::fs::write(&path, new_content)?;
            }
        }

        Ok(())
    }

    /// Load the last N lines from the output log for a plan.
    /// Returns an empty vec if the file doesn't exist.
    pub fn load_output_tail(&self, plan_base: &str, n: usize) -> Vec<String> {
        let path = self.output_log_path(plan_base);
        read_last_n_lines(&path, n)
    }

    /// Clean up all output log files (for fresh runs).
    pub fn cleanup_output_logs(&self) -> Result<()> {
        let output_dir = self.log_dir.join("output");
        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir)?;
            std::fs::create_dir_all(&output_dir)?;
        }
        Ok(())
    }
}

/// Read the last N lines from a file. Returns an empty vec if the file doesn't exist.
pub fn read_last_n_lines(path: &Path, n: usize) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].iter().map(|s| s.to_string()).collect()
}

/// Build a CrashEnvironment snapshot.
pub fn crash_environment() -> CrashEnvironment {
    let terminal_size = crossterm::terminal::size().ok();
    CrashEnvironment {
        rust_version: option_env!("CARGO_PKG_RUST_VERSION")
            .unwrap_or("unknown")
            .to_string(),
        os: std::env::consts::OS.to_string(),
        terminal_size,
        pid: std::process::id(),
    }
}

// ---------------------------------------------------------------------------
// Deferred failures — structured log of test/gate failures that were skipped
// (non-blocking or budget-allowed) so they can be batch-addressed later.
// ---------------------------------------------------------------------------

/// A single test or gate failure that was deferred (not blocking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredFailure {
    /// Plan that produced this failure (e.g. "03-mirage-rs")
    pub plan: String,
    /// Task ID from verify-tasks.toml (e.g. "UT7", "INV10")
    pub task_id: String,
    /// Human-readable title
    pub title: String,
    /// Task type: "unit_test", "invariant", "integration_test"
    pub task_type: String,
    /// The cargo command that was run
    pub command: String,
    /// Individual test function names that were expected
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_fns: Vec<String>,
    /// Why this failure was deferred
    pub reason: DeferredReason,
    /// First ~50 lines of error output (enough to diagnose, not overwhelming)
    pub error_snippet: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Iteration number when the failure occurred
    #[serde(default)]
    pub iteration: u32,
}

/// Why a failure was deferred instead of blocking the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferredReason {
    /// Task was marked blocking = false in verify-tasks.toml
    NonBlocking,
    /// TestFailureBudget watcher decided pass rate was good enough
    BudgetAllowed { pass_rate: f64, threshold: f64 },
    /// Force-advanced by user or iteration limit
    ForceAdvanced,
}

/// The full deferred-failures log written to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredFailureLog {
    /// Run/batch identifier
    pub batch_id: String,
    /// When this log was last updated
    pub updated_at: String,
    /// Total deferred failures across all plans
    pub total: usize,
    /// The individual failures
    pub failures: Vec<DeferredFailure>,
}

impl PersistenceManager {
    /// Path to the deferred failures file.
    pub fn deferred_failures_path(&self) -> PathBuf {
        self.log_dir.join("deferred-failures.toml")
    }

    /// Load existing deferred failures (returns empty log if file doesn't exist).
    pub fn load_deferred_failures(&self) -> DeferredFailureLog {
        let path = self.deferred_failures_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(log) = toml::from_str(&content) {
                return log;
            }
        }
        DeferredFailureLog {
            batch_id: String::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            total: 0,
            failures: Vec::new(),
        }
    }

    /// Append multiple deferred failures and flush to disk.
    pub fn append_deferred_failures(
        &self,
        batch_id: &str,
        failures: Vec<DeferredFailure>,
    ) -> Result<()> {
        if failures.is_empty() {
            return Ok(());
        }
        let mut log = self.load_deferred_failures();
        log.batch_id = batch_id.to_string();
        log.updated_at = chrono::Utc::now().to_rfc3339();
        log.failures.extend(failures);
        log.total = log.failures.len();
        self.write_deferred_failures(&log)
    }

    /// Write the full deferred failures log (atomic).
    fn write_deferred_failures(&self, log: &DeferredFailureLog) -> Result<()> {
        let path = self.deferred_failures_path();
        let content = toml::to_string_pretty(log)?;
        atomic_write_toml(&path, &content)
    }
}

/// Write content to a file atomically: write to a .tmp sibling, then rename.
/// This prevents corruption if the process is killed mid-write.
fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp_path = path.with_extension(format!("json.tmp.{}", std::process::id()));
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

fn atomic_write_toml(path: &Path, content: &str) -> Result<()> {
    let tmp_path = path.with_extension(format!("toml.tmp.{}", std::process::id()));
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

// ── Cost persistence ────────────────────────────────────────────────

/// Per-plan cost record stored as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCostRecord {
    /// Plan base name (e.g., "12-grimoire").
    pub plan: String,
    /// Lifetime cost in USD (accumulates across runs).
    pub cost_usd: f64,
    /// Number of iterations that contributed to this cost.
    pub iterations: u32,
    /// Last updated timestamp (ISO 8601).
    pub last_updated: String,
}

/// Aggregate cost summary across all plans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    /// Total cost across all plans.
    pub total_cost_usd: f64,
    /// Per-plan breakdown.
    pub plans: Vec<PlanCostRecord>,
    /// Last updated timestamp.
    pub last_updated: String,
}

impl PersistenceManager {
    /// Path to the cost summary file.
    fn cost_summary_path(&self) -> PathBuf {
        let costs_dir = self.log_dir.join("costs");
        costs_dir.join("summary.json")
    }

    /// Load the cost summary from disk. Returns empty summary if file doesn't exist.
    pub fn load_cost_summary(&self) -> CostSummary {
        let path = self.cost_summary_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| CostSummary {
                total_cost_usd: 0.0,
                plans: Vec::new(),
                last_updated: String::new(),
            }),
            Err(_) => CostSummary {
                total_cost_usd: 0.0,
                plans: Vec::new(),
                last_updated: String::new(),
            },
        }
    }

    /// Save the current cost_per_plan data, merging with existing lifetime costs.
    ///
    /// This accumulates — if plan 12 cost $5 last run and $3 this run,
    /// the persisted total is $8.
    pub fn save_costs(&self, cost_per_plan: &HashMap<String, f64>) -> Result<()> {
        let costs_dir = self.log_dir.join("costs");
        std::fs::create_dir_all(&costs_dir)?;

        // Load existing summary to merge
        let mut summary = self.load_cost_summary();
        let now = chrono::Utc::now().to_rfc3339();

        for (plan, &session_cost) in cost_per_plan {
            if session_cost <= 0.0 {
                continue;
            }
            if let Some(existing) = summary.plans.iter_mut().find(|p| p.plan == *plan) {
                existing.cost_usd += session_cost;
                existing.iterations += 1;
                existing.last_updated = now.clone();
            } else {
                summary.plans.push(PlanCostRecord {
                    plan: plan.clone(),
                    cost_usd: session_cost,
                    iterations: 1,
                    last_updated: now.clone(),
                });
            }
        }

        summary.total_cost_usd = summary.plans.iter().map(|p| p.cost_usd).sum();
        summary.last_updated = now;
        summary.plans.sort_by(|a, b| {
            b.cost_usd
                .partial_cmp(&a.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let json = serde_json::to_string_pretty(&summary)?;
        atomic_write(&self.cost_summary_path(), &json)?;

        Ok(())
    }

    /// Load persisted per-plan costs into a HashMap for restoring RunState on startup.
    pub fn load_cost_per_plan(&self) -> HashMap<String, f64> {
        let summary = self.load_cost_summary();
        summary
            .plans
            .into_iter()
            .map(|p| (p.plan, p.cost_usd))
            .collect()
    }
}

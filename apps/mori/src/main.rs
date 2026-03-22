mod agent;
mod app;
mod conductor;
mod git;
mod orchestrator;
mod state;
mod sys_metrics;
mod tui;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::state::persistence::{CrashAppState, CrashReport};

/// Global crash state: updated periodically from the event loop so the panic hook can read it.
static CRASH_STATE: OnceLock<Arc<Mutex<Option<CrashAppState>>>> = OnceLock::new();

/// Update the global crash state snapshot (called from app.rs event loops).
pub fn update_crash_state(state: CrashAppState) {
    if let Some(global) = CRASH_STATE.get() {
        if let Ok(mut guard) = global.lock() {
            *guard = Some(state);
        }
    }
}

use clap::{CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(name = "bardo-ctl", version, about = "Bardo plan orchestrator + TUI")]
struct Cli {
    /// Plan specs to run/validate (e.g. "01" "01-09" "08a-08d").
    /// If omitted, reads from .mori/queue.toml.
    plans: Vec<String>,

    /// Validate plan files and show parallelism stats (no execution)
    #[arg(long)]
    validate: bool,

    /// Skip the strategist + review loop (single implementer pass)
    #[arg(long)]
    no_review: bool,

    /// Skip the cargo test gate
    #[arg(long)]
    skip_tests: bool,

    /// Maximum review iterations before halting
    #[arg(long, default_value = "8")]
    max_iterations: u32,

    /// Pause after every N plans for review
    #[arg(long)]
    batch_size: Option<usize>,

    /// Override batch branch suffix
    #[arg(long, default_value_t = default_batch_id())]
    batch_id: String,

    /// Override the model
    #[arg(long)]
    model: Option<String>,

    /// Run without TUI (log to stdout)
    #[arg(long)]
    headless: bool,

    /// Skip Scribe + Critic phases (faster iteration)
    #[arg(long)]
    no_docs: bool,

    /// Repository root (default: current directory)
    #[arg(long)]
    repo_root: Option<PathBuf>,

    /// Maximum parallel agents
    #[arg(long, default_value = "8")]
    max_agents: usize,

    /// Maximum plans executing in parallel (default: 3, or 6 in express mode)
    #[arg(long, default_value = "3")]
    max_parallel_plans: usize,

    /// Enable parallel plan execution (plans in same wave run concurrently)
    #[arg(long)]
    parallel: bool,

    /// Enable speculative pre-planning for upcoming waves
    #[arg(long)]
    pre_plan: bool,

    /// Enable post-plan refactoring passes
    #[arg(long)]
    refactor: bool,

    /// Enable Codex fast mode (1.5× speed, 2× credits; GPT-5.4 only)
    #[arg(long)]
    fast: bool,

    /// Express mode: single-pass implementer, no strategist/reviews, up to 20 concurrent agents
    #[arg(long)]
    express: bool,

    /// Fallback model for spawn failures (retries once with this model before giving up)
    #[arg(long)]
    fallback_model: Option<String>,

    /// F2: Clean up merged plan branches and prune worktrees
    #[arg(long)]
    cleanup: bool,

    /// Build the DAG, print the execution plan, and exit without spawning agents
    #[arg(long)]
    dry_run: bool,

    /// Force reading .mori/queue.toml even when plan specs are given on CLI
    #[arg(long)]
    queue: bool,

    /// Run only plans from a specific milestone (reads .mori/queue.toml)
    #[arg(long)]
    milestone: Option<String>,

    /// Execution preset: "quality", "balanced", "cost", or "speed"
    #[arg(long)]
    preset: Option<String>,

    /// Enable embedded inference gateway (default: enabled if compiled with gateway feature)
    #[arg(long, default_value = "true")]
    gateway: bool,

    /// Disable the embedded inference gateway
    #[arg(long)]
    no_gateway: bool,

    /// Port for the embedded gateway
    #[arg(long, default_value = "4000")]
    gateway_port: u16,
}

fn default_batch_id() -> String {
    chrono::Local::now().format("%Y%m%d").to_string()
}

fn main() -> anyhow::Result<()> {
    let matches = Cli::command().get_matches();
    let mut cli = Cli::from_arg_matches(&matches).unwrap();

    // Express mode: auto-bump parallelism defaults unless explicitly overridden
    if cli.express {
        if matches.value_source("max_agents") != Some(clap::parser::ValueSource::CommandLine) {
            cli.max_agents = 20;
        }
        if matches.value_source("max_parallel_plans")
            != Some(clap::parser::ValueSource::CommandLine)
        {
            cli.max_parallel_plans = 6;
        }
    }

    let repo_root = cli.repo_root.unwrap_or_else(|| {
        std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| PathBuf::from(s.trim()))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    });

    // --- Queue & milestone resolution ---
    // If no plan specs given on CLI (or --queue / --milestone is set), try .mori/queue.toml
    let queue_config = if cli.plans.is_empty() || cli.queue || cli.milestone.is_some() {
        crate::orchestrator::queue::QueueConfig::load(&repo_root)
    } else {
        None
    };

    if let Some(ref qc) = queue_config {
        // Resolve plan specs from queue config
        if cli.plans.is_empty() || cli.queue {
            if let Some(ref ms_name) = cli.milestone {
                // --milestone: load only that milestone's plans
                cli.plans = qc.milestone_plans(ms_name).unwrap_or_default();
                if cli.plans.is_empty() {
                    anyhow::bail!("Milestone {:?} not found or has no plans", ms_name);
                }
            } else {
                cli.plans = qc.all_plan_specs();
            }
        } else if let Some(ref ms_name) = cli.milestone {
            // CLI specs given + --milestone: use milestone plans instead
            cli.plans = qc.milestone_plans(ms_name).unwrap_or_default();
            if cli.plans.is_empty() {
                anyhow::bail!("Milestone {:?} not found or has no plans", ms_name);
            }
        }

        // Apply run settings from queue (CLI flags take precedence)
        if matches.value_source("max_agents") != Some(clap::parser::ValueSource::CommandLine) {
            if let Some(max) = qc.run.max_agents {
                cli.max_agents = max;
            }
        }
        if matches.value_source("max_parallel_plans")
            != Some(clap::parser::ValueSource::CommandLine)
        {
            if let Some(max) = qc.run.max_parallel_plans {
                cli.max_parallel_plans = max;
            }
        }
        if qc.run.mode.as_deref() == Some("express") && !cli.express {
            cli.express = true;
        }
    }

    // Bail if still no plans after queue resolution
    if cli.plans.is_empty() && !cli.cleanup {
        anyhow::bail!(
            "No plan specs provided. Pass plan specs on the CLI or create .mori/queue.toml"
        );
    }

    // Validate mode: print stats and exit
    if cli.validate {
        return run_validate(&repo_root, &cli.plans);
    }

    // F2: Cleanup mode
    if cli.cleanup {
        return run_cleanup(&repo_root, &cli.batch_id);
    }

    // Dry-run mode: build DAG, print execution plan, exit
    if cli.dry_run {
        return run_dry_run(&repo_root, &cli.plans, cli.max_agents);
    }

    // Set up file-based tracing (don't write to terminal — TUI owns it)
    let log_dir = crate::orchestrator::paths::runs_dir(&repo_root);
    std::fs::create_dir_all(&log_dir)?;

    // Truncate log at startup so each run starts clean
    let _ = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(log_dir.join("bardo-ctl.log"));

    let file_appender = tracing_appender::rolling::never(&log_dir, "bardo-ctl.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .json()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_current_span(true)
        .init();

    // Initialize global crash state
    let crash_state = Arc::new(Mutex::new(None::<CrashAppState>));
    CRASH_STATE.get_or_init(|| crash_state.clone());

    // Resolve preset: CLI --preset takes priority, then queue config preset
    let preset = cli
        .preset
        .or_else(|| queue_config.as_ref().and_then(|q| q.run.preset.clone()));

    let config = app::AppConfig {
        repo_root,
        plan_specs: cli.plans,
        no_review: cli.no_review,
        skip_tests: cli.skip_tests,
        max_iterations: cli.max_iterations,
        batch_size: cli.batch_size,
        batch_id: cli.batch_id,
        model: cli.model,
        headless: cli.headless,
        no_docs: cli.no_docs,
        max_agents: cli.max_agents,
        max_parallel_plans: cli.max_parallel_plans,
        parallel: cli.parallel,
        pre_plan: cli.pre_plan,
        refactor: cli.refactor,
        fast: cli.fast,
        express: cli.express,
        fallback_model: cli.fallback_model,
        preset,
        gateway_enabled: cli.gateway && !cli.no_gateway,
        gateway_port: cli.gateway_port,
        gateway_api_key: String::new(), // auto-generated at startup
    };

    // Panic hook: restore terminal, capture crash report, log, then default handler.
    let default_panic = std::panic::take_hook();
    let panic_log_dir = log_dir.clone();
    let panic_repo_root = config.repo_root.clone();
    let panic_config_summary = format!("{:?}", config);
    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();

        // Capture backtrace
        let backtrace = format!("{}", std::backtrace::Backtrace::force_capture());

        // Read global crash state
        let app_state = CRASH_STATE
            .get()
            .and_then(|s| s.lock().ok())
            .and_then(|guard| guard.clone())
            .unwrap_or_default();

        // Read recent log lines from disk
        let log_path = panic_log_dir.join("bardo-ctl.log");
        let recent_logs = state::persistence::read_last_n_lines(&log_path, 100);

        let sig = state::persistence::compute_error_signature(&msg, &location);

        let report = CrashReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            crash_type: "panic".to_string(),
            message: msg.clone(),
            location: Some(location.clone()),
            backtrace,
            error_signature: sig,
            recent_logs,
            app_state,
            config_summary: panic_config_summary.clone(),
            environment: state::persistence::crash_environment(),
        };

        let persistence = state::persistence::PersistenceManager::new(&panic_repo_root);
        if let Err(e) = persistence.write_crash_report(&report) {
            eprintln!("Failed to write crash report: {e}");
        }

        tracing::error!("PANIC at {location}: {msg}");
        default_panic(info);
    }));

    // Build tokio runtime and run
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let result = rt.block_on(app::run(config.clone()));

    // Always restore the terminal — run() may exit early via ? without calling tui::restore()
    let _ = tui::restore();

    if let Err(ref e) = result {
        // Write crash report for non-panic errors too
        let error_msg = format!("{e:#}");
        let sig = state::persistence::compute_error_signature(&error_msg, "main::run");

        let app_state = CRASH_STATE
            .get()
            .and_then(|s| s.lock().ok())
            .and_then(|guard| guard.clone())
            .unwrap_or_default();

        let recent_logs =
            state::persistence::read_last_n_lines(&log_dir.join("bardo-ctl.log"), 100);

        let report = CrashReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            crash_type: "error".to_string(),
            message: error_msg.clone(),
            location: Some("main::run".to_string()),
            backtrace: String::new(),
            error_signature: sig,
            recent_logs,
            app_state,
            config_summary: format!("{:?}", config),
            environment: state::persistence::crash_environment(),
        };

        let persistence = state::persistence::PersistenceManager::new(&config.repo_root);
        let _ = persistence.write_crash_report(&report);

        eprintln!("bardo-ctl error: {error_msg}");
        tracing::error!("Fatal error: {error_msg}");
    }
    result
}

fn run_validate(repo_root: &PathBuf, plan_specs: &[String]) -> anyhow::Result<()> {
    use orchestrator::dag::PlanDag;
    use orchestrator::plan::discover_plans;
    use orchestrator::tasks;
    use orchestrator::unified_dag::UnifiedTaskDag;

    let plans_dir = crate::orchestrator::paths::plans_root(&repo_root);
    let plans = discover_plans(&plans_dir, plan_specs)?;

    println!("Plans discovered: {}", plans.len());
    for p in &plans {
        let deps = p
            .frontmatter
            .as_ref()
            .map(|f| f.depends_on.join(", "))
            .unwrap_or_else(|| "(no frontmatter)".to_string());
        let safe = p
            .frontmatter
            .as_ref()
            .map(|f| f.parallel_safe)
            .unwrap_or(false);
        println!("  {} [deps: {}] [parallel_safe: {}]", p.base, deps, safe);
    }

    // Load task files
    let mut task_files: HashMap<String, orchestrator::tasks::TaskFile> = HashMap::new();
    let mut total_tasks = 0usize;
    for plan in &plans {
        if let Ok(Some(cl)) = tasks::load_checklist(repo_root, &plan.num) {
            total_tasks += cl.tasks.len();
            task_files.insert(
                plan.base.clone(),
                orchestrator::tasks::TaskFile {
                    meta: orchestrator::tasks::TaskMeta {
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
    println!(
        "\nTask files loaded: {} plans, {} total tasks",
        task_files.len(),
        total_tasks
    );
    let plans_without_tasks: Vec<&str> = plans
        .iter()
        .filter(|p| !task_files.contains_key(&p.base))
        .map(|p| p.base.as_str())
        .collect();
    if !plans_without_tasks.is_empty() {
        println!(
            "  Plans without task breakdown: {}",
            plans_without_tasks.join(", ")
        );
    }

    // Build unified task DAG
    println!("\n--- Unified Task DAG ---");
    match UnifiedTaskDag::from_plans(&plans, &task_files) {
        Ok(dag) => {
            println!("  Nodes: {}", dag.node_count());
            println!("  Max parallelism width: {}", dag.max_width());
            println!("  Critical path: {} min", dag.critical_path_minutes());

            // Dangling references
            let dangling = UnifiedTaskDag::find_dangling_references(&plans, &task_files);
            if dangling.is_empty() {
                println!("  Dangling references: none");
            } else {
                println!("  Dangling references:");
                for d in &dangling {
                    println!("    - {d}");
                }
            }
        }
        Err(e) => {
            println!("  ERROR building DAG: {e}");
        }
    }

    // Build plan-level DAG for wave visualization
    println!("\n--- Plan DAG (waves) ---");
    match PlanDag::from_plans_and_tasks(&plans, &task_files) {
        Ok(dag) => {
            let waves = dag.compute_waves();
            println!("  Waves: {}", waves.len());
            println!("  Estimated total: {} min", dag.estimated_total_minutes());
            for wave in &waves {
                println!(
                    "  Wave {}: {} (est {}m)",
                    wave.index,
                    wave.plans.join(", "),
                    wave.estimated_minutes,
                );
            }
            let cp = dag.critical_path();
            if !cp.is_empty() {
                println!("  Critical path: {}", cp.join(" -> "));
            }
        }
        Err(e) => {
            println!("  ERROR: {e}");
        }
    }

    Ok(())
}

/// F2: Clean up merged plan branches and prune worktrees.
fn run_cleanup(repo_root: &PathBuf, batch_id: &str) -> anyhow::Result<()> {
    let batch_branch = format!("codex/batch/{batch_id}");
    println!("Cleaning up merged branches for batch {batch_branch}...");

    // List all codex/plan/* branches
    let output = std::process::Command::new("git")
        .args(["branch", "--list", "codex/plan/*"])
        .current_dir(repo_root)
        .output()?;
    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().trim_start_matches("* ").to_string())
        .filter(|b| !b.is_empty())
        .collect();

    println!("Found {} plan branches", branches.len());

    let mut deleted = 0;
    for branch in &branches {
        // Check if branch is fully merged into batch
        let merged = std::process::Command::new("git")
            .args(["merge-base", "--is-ancestor", branch, &batch_branch])
            .current_dir(repo_root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if merged {
            println!("  Deleting merged branch: {branch}");
            let _ = std::process::Command::new("git")
                .args(["branch", "-D", branch])
                .current_dir(repo_root)
                .output();
            deleted += 1;
        } else {
            println!("  Keeping unmerged branch: {branch}");
        }
    }

    // Prune worktrees
    let _ = std::process::Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_root)
        .output();

    println!(
        "Deleted {deleted}/{} branches, pruned worktrees",
        branches.len()
    );
    Ok(())
}

/// Dry-run mode: build the DAG, print the execution plan, and exit.
fn run_dry_run(
    repo_root: &PathBuf,
    plan_specs: &[String],
    max_agents: usize,
) -> anyhow::Result<()> {
    use orchestrator::dag::PlanDag;
    use orchestrator::plan::discover_plans;
    use orchestrator::tasks;
    use orchestrator::unified_dag::UnifiedTaskDag;

    let plans_dir = crate::orchestrator::paths::plans_root(&repo_root);
    let plans = discover_plans(&plans_dir, plan_specs)?;

    println!("=== Dry Run ===\n");
    println!("Plans: {}", plans.len());
    println!("Max agents: {max_agents}");

    // Load task files
    let mut task_files: HashMap<String, orchestrator::tasks::TaskFile> = HashMap::new();
    let mut total_tasks = 0usize;
    for plan in &plans {
        if let Ok(Some(cl)) = tasks::load_checklist(repo_root, &plan.num) {
            total_tasks += cl.tasks.len();
            task_files.insert(
                plan.base.clone(),
                orchestrator::tasks::TaskFile {
                    meta: orchestrator::tasks::TaskMeta {
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
    println!("Tasks: {total_tasks}");

    // Wave breakdown
    println!("\n--- Execution Plan (waves) ---");
    match PlanDag::from_plans_and_tasks(&plans, &task_files) {
        Ok(dag) => {
            let waves = dag.compute_waves();
            for wave in &waves {
                let task_count: usize = wave
                    .plans
                    .iter()
                    .map(|p| task_files.get(p).map(|tf| tf.tasks.len()).unwrap_or(1))
                    .sum();
                println!(
                    "  Wave {}: {} plans, {} tasks, ~{}min est",
                    wave.index,
                    wave.plans.len(),
                    task_count,
                    wave.estimated_minutes,
                );
                for p in &wave.plans {
                    let tc = task_files.get(p).map(|tf| tf.tasks.len()).unwrap_or(0);
                    let deps = plans
                        .iter()
                        .find(|pi| pi.base == *p)
                        .and_then(|pi| pi.frontmatter.as_ref())
                        .map(|f| f.depends_on.join(", "))
                        .unwrap_or_default();
                    let dep_str = if deps.is_empty() {
                        String::new()
                    } else {
                        format!(" (deps: {deps})")
                    };
                    println!("    - {p}: {tc} tasks{dep_str}");
                }
            }

            let cp = dag.critical_path();
            if !cp.is_empty() {
                println!("\nCritical path: {}", cp.join(" -> "));
            }
            println!("Estimated total: ~{}min", dag.estimated_total_minutes());
            println!("Waves: {}", waves.len());
        }
        Err(e) => println!("  ERROR building DAG: {e}"),
    }

    // Unified DAG stats
    if let Ok(dag) = UnifiedTaskDag::from_plans(&plans, &task_files) {
        println!("\n--- Task DAG ---");
        println!("  Nodes: {}", dag.node_count());
        println!("  Max parallelism: {}", dag.max_width());
        println!(
            "  Effective parallelism: {} (capped by --max-agents)",
            dag.max_width().min(max_agents)
        );
    }

    println!("\nNo agents spawned (dry run).");
    Ok(())
}

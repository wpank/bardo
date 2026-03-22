pub mod artifacts;
pub mod autofix;
pub mod batch;
pub mod complexity;
pub mod context;
pub mod coordination;
pub mod dag;
pub mod event_log;
pub mod executor;
pub mod gates;
pub mod inject;
pub mod iteration_memory;
pub mod memory;
pub mod paths;
pub mod phase;
pub mod pipeline;
pub mod plan;
pub mod preflight;
pub mod queue;
pub mod prompts;
pub mod registry;
pub mod review;
pub mod schema;
pub mod skills;
pub mod tasks;
pub mod unified_dag;

use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

pub use artifacts::ArtifactStore;
pub use dag::{ExecutionWave, PlanDag};
pub use executor::{
    ExecutorAction, ExecutorProgress, ExecutorSnapshot, ParallelExecutor, PlanPhase,
};
pub use phase::Phase;
pub use pipeline::{GateType, PipelineAction, PipelineEvent, PipelinePhase, PlanPipeline};
pub use plan::PlanInfo;
pub use registry::Registry;
pub use schema::{CompletionReport, ReviewIssue, ReviewReport, ReviewVerdict};
pub use unified_dag::{GlobalTaskId, UnifiedTaskDag};

/// Orchestrator state machine states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestratorState {
    Initializing,
    PlanReady,
    Preflight,
    Implementer,
    CompileGate,
    TestGate,
    Reviewing,    // Architect + Auditor + Scribe parallel
    CriticReview, // Critic reviews docs
    Verdict,
    DocRevision,
    Committing,
    BatchCheck,
    BatchPaused,
    Complete,
    Halted {
        reason: String,
    },
    // Parallel execution states
    ParallelWave {
        wave_idx: usize,
        active_plans: Vec<String>,
        completed: HashSet<String>,
        failed: HashSet<String>,
    },
    WaveMerge {
        wave_idx: usize,
        plans_to_merge: Vec<String>,
        merge_idx: usize,
    },
    PrePlanning,
    Refactoring,
}

/// Policy for handling plan failures within a parallel wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveErrorPolicy {
    /// Halt the entire wave when any plan fails.
    HaltWave,
    /// Continue running other plans in the wave; merge successes.
    ContinueWave,
    /// Skip the failed plan and continue.
    SkipAndContinue,
}

impl Default for WaveErrorPolicy {
    fn default() -> Self {
        Self::ContinueWave
    }
}

impl OrchestratorState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Initializing => "initializing",
            Self::PlanReady => "plan-ready",
            Self::Preflight => "preflight",
            Self::Implementer => "implementer",
            Self::CompileGate => "compile-gate",
            Self::TestGate => "test-gate",
            Self::Reviewing => "reviewing",
            Self::CriticReview => "critic-review",
            Self::Verdict => "verdict",
            Self::DocRevision => "doc-revision",
            Self::Committing => "committing",
            Self::BatchCheck => "batch-check",
            Self::BatchPaused => "batch-paused",
            Self::Complete => "complete",
            Self::Halted { .. } => "halted",
            Self::ParallelWave { .. } => "parallel-wave",
            Self::WaveMerge { .. } => "wave-merge",
            Self::PrePlanning => "pre-planning",
            Self::Refactoring => "refactoring",
        }
    }
}

/// Events emitted by the orchestrator for the TUI / event bus
#[derive(Debug, Clone)]
pub enum OrchestratorEvent {
    StateChanged {
        from: OrchestratorState,
        to: OrchestratorState,
    },
    PlanStarted {
        plan: PlanInfo,
        index: usize,
        total: usize,
    },
    PlanCompleted {
        plan: PlanInfo,
    },
    PlanSkipped {
        plan: PlanInfo,
        reason: String,
    },
    PhaseStarted {
        phase: Phase,
        iteration: u32,
    },
    GateResult {
        gate: String,
        passed: bool,
        output: String,
    },
    ReviewCapHit {
        plan: String,
        iterations: u32,
    },
    RunComplete,
    Error {
        message: String,
    },
}

/// Orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub repo_root: PathBuf,
    pub plans_dir: PathBuf,
    pub no_review: bool,
    pub skip_tests: bool,
    pub max_iterations: u32,
    pub batch_size: Option<usize>,
    pub model: Option<String>,
    pub no_docs: bool,
    /// Maximum number of plans executing in parallel (default 3, max 4).
    pub max_parallel_plans: usize,
    /// What to do when a plan fails in a parallel wave.
    pub wave_error_policy: WaveErrorPolicy,
    /// Enable parallel wave execution.
    pub parallel: bool,
    /// Run pre-planning agent ahead of execution.
    pub pre_plan: bool,
    /// Run a refactorer every N completed plans (0 = disabled).
    pub refactor_interval: usize,
    pub artifact_store_root: Option<std::path::PathBuf>,
    pub registry_root: Option<std::path::PathBuf>,
}

impl OrchestratorConfig {
    pub fn new(repo_root: PathBuf) -> Self {
        let plans_dir = paths::plans_root(&repo_root);
        Self {
            repo_root,
            plans_dir,
            no_review: false,
            skip_tests: false,
            max_iterations: 3,
            batch_size: None,
            model: None,
            no_docs: false,
            max_parallel_plans: 3,
            wave_error_policy: WaveErrorPolicy::default(),
            parallel: false,
            pre_plan: false,
            refactor_interval: 5,
            artifact_store_root: None,
            registry_root: None,
        }
    }
}

/// The orchestrator drives plan execution
pub struct Orchestrator {
    pub config: OrchestratorConfig,
    pub state: OrchestratorState,
    pub plans: Vec<PlanInfo>,
    pub current_plan_idx: usize,
    pub current_iteration: u32,
    pub plans_completed: usize,
    event_tx: tokio::sync::mpsc::UnboundedSender<OrchestratorEvent>,
}

impl Orchestrator {
    pub fn new(
        config: OrchestratorConfig,
        event_tx: tokio::sync::mpsc::UnboundedSender<OrchestratorEvent>,
    ) -> Self {
        Self {
            config,
            state: OrchestratorState::Initializing,
            plans: Vec::new(),
            current_plan_idx: 0,
            current_iteration: 1,
            plans_completed: 0,
            event_tx,
        }
    }

    /// Discover plans from the spec (e.g. "01-09", "03", "08a-08d")
    pub fn discover_plans(&mut self, specs: &[String]) -> Result<()> {
        self.plans = plan::discover_plans(&self.config.plans_dir, specs)?;
        Ok(())
    }

    pub fn current_plan(&self) -> Option<&PlanInfo> {
        self.plans.get(self.current_plan_idx)
    }

    pub fn total_plans(&self) -> usize {
        self.plans.len()
    }

    fn transition(&mut self, new_state: OrchestratorState) {
        let old = self.state.clone();
        self.state = new_state.clone();
        let _ = self.event_tx.send(OrchestratorEvent::StateChanged {
            from: old,
            to: new_state,
        });
    }

    /// Check if a plan should be skipped (already has a git tag)
    pub fn should_skip_plan(&self, plan: &PlanInfo) -> bool {
        // Check for git tag plan/{base}
        let tag_name = format!("plan/{}", plan.base);
        std::process::Command::new("git")
            .args(["tag", "-l", &tag_name])
            .current_dir(&self.config.repo_root)
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false)
    }

    /// Advance to next plan or complete
    pub fn advance_to_next_plan(&mut self) -> bool {
        self.current_plan_idx += 1;
        self.current_iteration = 1;
        if self.current_plan_idx >= self.plans.len() {
            self.transition(OrchestratorState::Complete);
            let _ = self.event_tx.send(OrchestratorEvent::RunComplete);
            false
        } else {
            self.transition(OrchestratorState::PlanReady);
            true
        }
    }

    pub fn set_state(&mut self, state: OrchestratorState) {
        self.transition(state);
    }

    pub fn emit(&self, event: OrchestratorEvent) {
        let _ = self.event_tx.send(event);
    }
}

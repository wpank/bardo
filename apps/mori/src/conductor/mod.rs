pub mod actions;
pub mod llm;
pub mod watchers;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::agent::AgentRole;

/// Intervention tiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionTier {
    Nudge,
    Restart,
    Abort,
}

/// What the conductor wants the event loop to do
#[derive(Debug, Clone)]
pub enum ConductorAction {
    /// Send a message to an agent (nudge/steer)
    SendMessage { role: AgentRole, message: String },
    /// Kill and restart an agent from scratch
    RestartAgent { role: AgentRole },
    /// Force advance past the current plan (skip reviews, commit what's there)
    ForceAdvance,
    /// Skip the review phase and commit directly
    SkipReviews,
    /// Spawn a validation pass
    SpawnValidation {
        validation_type: String,
        plan: String,
    },
    /// Generate a fix plan from validation failures
    GenerateFixPlan {
        title: String,
        description: String,
        crates: Vec<String>,
    },
    /// Insert a gate into a plan pipeline
    InsertGate { plan: String, gate_type: String },
    /// Skip a validation for a given plan
    SkipValidation {
        validation_type: String,
        plan: String,
    },
    /// Inject additional task descriptions into a warm implementer agent.
    AssignAdditionalTasks {
        instance_id: String,
        task_descriptions: Vec<String>,
    },
    /// Send a lightweight keepalive ping to a warm agent (no tokens, just "you're next" signal)
    PingWarmAgent { instance_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationType {
    Terminal,
    Golem,
    Spec,
    Integration,
    Coverage,
    Regression,
}

/// A conductor intervention
#[derive(Debug, Clone)]
pub struct Intervention {
    pub tier: InterventionTier,
    pub watcher: String,
    pub target_role: AgentRole,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: Option<ConductorAction>,
}

impl Intervention {
    pub fn tier_label(&self) -> &'static str {
        match self.tier {
            InterventionTier::Nudge => "nudge",
            InterventionTier::Restart => "restart",
            InterventionTier::Abort => "abort",
        }
    }
}

/// Conductor configuration
#[derive(Debug, Clone)]
pub struct ConductorConfig {
    pub silence_timeout: Duration,
    pub compile_fail_threshold: u32,
    pub task_stall_timeout: Duration,
    pub context_pressure_ratio: f64,
    pub phase_timeout: Duration,
    pub llm_enabled: bool,
    /// Soft limit on concurrent agents
    pub agent_soft_limit: usize,
    /// Minimum test pass rate to force-advance (0.0–1.0). If compile gates pass
    /// and at least this fraction of tests pass, the conductor treats remaining
    /// failures as non-actionable and moves on.
    pub test_pass_budget_ratio: f64,
}

impl Default for ConductorConfig {
    fn default() -> Self {
        Self {
            silence_timeout: Duration::from_secs(300),
            compile_fail_threshold: 3,
            task_stall_timeout: Duration::from_secs(600),
            context_pressure_ratio: 0.8,
            phase_timeout: Duration::from_secs(900),
            llm_enabled: true,
            agent_soft_limit: 8,
            test_pass_budget_ratio: 0.7,
        }
    }
}

/// Tracks agent spawning rate to avoid API throttling.
pub struct RateLimiter {
    pub soft_limit: usize,
    pub active_count: usize,
    pub spawn_queue: Vec<PendingSpawn>,
}

#[derive(Debug, Clone)]
pub struct PendingSpawn {
    pub role: AgentRole,
    pub instance: String,
    pub priority: u32, // lower = higher priority
}

impl RateLimiter {
    pub fn new(soft_limit: usize) -> Self {
        Self {
            soft_limit,
            active_count: 0,
            spawn_queue: Vec::new(),
        }
    }

    /// Check if we can spawn another agent.
    pub fn can_spawn(&self) -> bool {
        self.active_count < self.soft_limit
    }

    /// Record an agent spawn.
    pub fn record_spawn(&mut self) {
        self.active_count += 1;
    }

    /// Record an agent exit.
    pub fn record_exit(&mut self) {
        self.active_count = self.active_count.saturating_sub(1);
    }

    /// Queue a spawn request if at limit.
    pub fn enqueue(&mut self, role: AgentRole, instance: String, priority: u32) {
        self.spawn_queue.push(PendingSpawn {
            role,
            instance,
            priority,
        });
        self.spawn_queue.sort_by_key(|s| s.priority);
    }

    /// Get the next spawn from the queue if capacity is available.
    pub fn next_queued(&mut self) -> Option<PendingSpawn> {
        if self.can_spawn() && !self.spawn_queue.is_empty() {
            Some(self.spawn_queue.remove(0))
        } else {
            None
        }
    }

    /// Priority for a role (lower = higher priority = spawned first)
    pub fn role_priority(role: AgentRole) -> u32 {
        match role {
            AgentRole::Implementer => 0,
            AgentRole::Strategist => 1,
            AgentRole::Architect | AgentRole::Auditor => 2,
            AgentRole::Scribe | AgentRole::Critic => 3,
            AgentRole::PrePlanner => 4,
            AgentRole::Refactorer => 5,
            AgentRole::DocVerifier => 6,
            AgentRole::IntegrationTester => 6,
            AgentRole::MergeResolver => 1,
            AgentRole::TerminalValidator
            | AgentRole::GolemLifecycleTester
            | AgentRole::SpecDriftDetector
            | AgentRole::RegressionDetector
            | AgentRole::PerformanceSentinel
            | AgentRole::CoverageTracker
            | AgentRole::PlanLifecycleManager
            | AgentRole::CrossSystemTester
            | AgentRole::PatternExtractor
            | AgentRole::SnapshotComparator => 6,
            AgentRole::ErrorDiagnoser => 1,
            AgentRole::DependencyValidator => 2,
            AgentRole::Researcher => 3,
            AgentRole::AutoFixer => 1,
            AgentRole::QuickReviewer => 2,
            AgentRole::FullLoopValidator => 6,
            AgentRole::Conductor => 7,
        }
    }
}

/// The Conductor meta-orchestrator
pub struct Conductor {
    pub config: ConductorConfig,
    pub rate_limiter: RateLimiter,
    watchers: watchers::WatcherSet,
    history: Vec<Intervention>,
    cooldowns: HashMap<String, Instant>,
    cooldown_duration: Duration,
}

impl Conductor {
    pub fn new(config: ConductorConfig) -> Self {
        let watchers = watchers::WatcherSet::new(&config);
        let rate_limiter = RateLimiter::new(config.agent_soft_limit);
        Self {
            config,
            rate_limiter,
            watchers,
            history: Vec::new(),
            cooldowns: HashMap::new(),
            cooldown_duration: Duration::from_secs(120),
        }
    }

    /// Tick the conductor — called from the event loop on each frame
    pub fn tick(&mut self, ctx: &ConductorContext) -> Vec<Intervention> {
        let mut interventions = Vec::new();

        for intervention in self.watchers.check(ctx) {
            let key = if ctx.plan.is_empty() {
                format!("{}:{}", intervention.watcher, intervention.target_role)
            } else {
                format!(
                    "{}:{}:{}",
                    intervention.watcher, ctx.plan, intervention.target_role
                )
            };

            // Check cooldown
            if let Some(last_fire) = self.cooldowns.get(&key) {
                if last_fire.elapsed() < self.cooldown_duration {
                    continue;
                }
            }

            self.cooldowns.insert(key, Instant::now());
            self.history.push(intervention.clone());
            interventions.push(intervention);
        }

        interventions
    }

    pub fn history(&self) -> &[Intervention] {
        &self.history
    }
}

/// Summary of task progress for a plan — used by the task-continuation watcher.
#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub plan: String,
    pub queued: u32,
    pub in_flight: u32,
    pub completed: u32,
}

/// Context snapshot passed to the conductor on each tick
#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub active_role: Option<AgentRole>,
    pub last_message_at: Option<Instant>,
    pub phase_started: Option<Instant>,
    pub compile_fail_count: u32,
    pub last_compile_error: String,
    pub task_last_change: Option<Instant>,
    pub input_tokens: u64,
    pub context_limit: u64,
    /// Current iteration number
    pub iteration: u32,
    /// How many consecutive REVISE verdicts (resets on APPROVE)
    pub consecutive_revise_count: u32,
    /// Current orchestrator state label
    pub orchestrator_state: String,
    /// Whether the last agent turn produced meaningful output (> 200 chars)
    pub last_turn_had_output: bool,
    /// How many seconds the last agent turn took
    pub last_turn_duration_secs: u64,
    /// Whether the last turn left staged git changes (suppresses ghost turn detection)
    pub last_turn_has_git_changes: bool,
    /// Plan identifier (for per-plan cooldown keys in parallel mode)
    pub plan: String,
    /// D4: Agent backend (if Claude, double silence timeout)
    pub agent_backend: Option<crate::agent::AgentBackend>,
    /// DAG task summary for the current plan (used by task-continuation watcher).
    pub task_summary: Option<TaskSummary>,
    /// Instance ID of the active implementer batch agent for this plan.
    pub active_instance_id: Option<String>,
    /// Number of test tasks that passed in the current plan.
    pub test_pass_count: u32,
    /// Number of test tasks that failed in the current plan.
    pub test_fail_count: u32,
    /// Whether all compile gates passed for the current plan.
    pub compile_gates_passed: bool,
}

impl Default for ConductorContext {
    fn default() -> Self {
        Self {
            active_role: None,
            last_message_at: None,
            phase_started: None,
            compile_fail_count: 0,
            last_compile_error: String::new(),
            task_last_change: None,
            input_tokens: 0,
            context_limit: 200_000,
            iteration: 1,
            consecutive_revise_count: 0,
            orchestrator_state: String::new(),
            last_turn_had_output: true,
            last_turn_duration_secs: 0,
            last_turn_has_git_changes: false,
            plan: String::new(),
            agent_backend: None,
            task_summary: None,
            active_instance_id: None,
            test_pass_count: 0,
            test_fail_count: 0,
            compile_gates_passed: false,
        }
    }
}

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use tracing::warn;

use super::autofix;
use super::event_log;
use super::plan::PlanInfo;
use crate::agent::AgentRole;

/// Execution phase for a single plan pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelinePhase {
    Preflight,
    Strategist,
    Implementer,
    CompileGate,
    TestGate,
    Reviewing,
    CriticReview,
    Verdict,
    DocRevision,
    Committing,
    Complete,
    Failed { reason: String },
    TerminalValidation,
    GolemLifecycleTest,
    DependencyDenyCheck,
    IgnoredTestCheck,
    SpecComplianceCheck,
    RegressionCheck,
    CoverageCheck,
    IntegrationTest,
    AutoFix { fix_plan: String },
    QuickFix,
    ErrorDiagnosis,
    DependencyValidation,
    PatternExtraction,
}

impl PipelinePhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Strategist => "strategist",
            Self::Implementer => "implementer",
            Self::CompileGate => "compile-gate",
            Self::TestGate => "test-gate",
            Self::Reviewing => "reviewing",
            Self::CriticReview => "critic-review",
            Self::Verdict => "verdict",
            Self::DocRevision => "doc-revision",
            Self::Committing => "committing",
            Self::Complete => "complete",
            Self::Failed { .. } => "failed",
            Self::TerminalValidation => "terminal-validation",
            Self::GolemLifecycleTest => "golem-lifecycle-test",
            Self::DependencyDenyCheck => "dep-deny-check",
            Self::IgnoredTestCheck => "ignored-test-check",
            Self::SpecComplianceCheck => "spec-compliance-check",
            Self::RegressionCheck => "regression-check",
            Self::CoverageCheck => "coverage-check",
            Self::IntegrationTest => "integration-test",
            Self::AutoFix { .. } => "auto-fix",
            Self::QuickFix => "quick-fix",
            Self::ErrorDiagnosis => "error-diagnosis",
            Self::DependencyValidation => "dep-validation",
            Self::PatternExtraction => "pattern-extraction",
        }
    }
}

/// A self-contained pipeline for executing a single plan.
/// Pure state machine: handle_event() -> Vec<PipelineAction>.
pub struct PlanPipeline {
    pub plan: PlanInfo,
    pub plan_idx: usize,
    pub phase: PipelinePhase,
    pub iteration: u32,
    pub started_at: Instant,
    pub phase_started: Instant,
    pub worktree_path: Option<PathBuf>,
    pub branch: String,
    pub agent_instances: HashMap<AgentRole, String>,
    pub pending_reviews: HashSet<AgentRole>,
    pub compile_fail_count: u32,
    pub consecutive_revise_count: u32,
    pub doc_revision_count: u32,
    pub code_verdict: String,
    pub doc_verdict: String,
    pub completed: bool,
    pub failed: bool,
    pub config: super::complexity::PipelineConfig,
    instance_prefix: String,
    /// Repo root for event logging. None in tests.
    repo_root: Option<PathBuf>,
}

/// Actions the pipeline wants the executor to perform
#[derive(Debug, Clone)]
pub enum PipelineAction {
    SpawnAgent {
        instance_id: String,
        working_dir: PathBuf,
        role: AgentRole,
        prompt: String,
        effort: String,
        model: Option<String>,
    },
    KillAgent {
        instance_id: String,
    },
    RunGate {
        gate_type: GateType,
        working_dir: PathBuf,
    },
    Commit {
        message: String,
    },
    MergeToBatch,
    Complete,
    Halt {
        reason: String,
    },
    ReviewCapHit {
        plan_base: String,
        iterations: u32,
    },
    RunAutoFix {
        working_dir: PathBuf,
        json_output: String,
    },
}

#[derive(Debug, Clone)]
pub enum GateType {
    Compile,
    Clippy,
    Test,
    InvariantGate,
    TerminalRender {
        screens: Vec<String>,
    },
    SpecCompliance {
        prd2_paths: Vec<String>,
        crate_paths: Vec<String>,
    },
    IntegrationCheck,
    CoverageThreshold {
        min_pct: f64,
    },
    IgnoredTestEnforcement,
    DependencyDeny,
}

/// Events that flow into a pipeline
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    AgentCompleted {
        role: AgentRole,
        instance_id: String,
    },
    AgentError {
        role: AgentRole,
        instance_id: String,
        error: String,
    },
    GatePassed {
        gate: GateType,
    },
    GateFailed {
        gate: GateType,
        output: String,
    },
    VerdictReceived {
        role: AgentRole,
        verdict: String,
    },
    PhaseTimeout,
}

impl PlanPipeline {
    pub fn new(
        plan: PlanInfo,
        plan_idx: usize,
        worktree_path: Option<PathBuf>,
        branch: String,
    ) -> Self {
        let instance_prefix = plan.base.clone();
        let config = super::complexity::PipelineConfig::for_complexity(
            super::complexity::classify_plan(&plan),
        );
        Self {
            plan,
            plan_idx,
            phase: PipelinePhase::Preflight,
            iteration: 0,
            started_at: Instant::now(),
            phase_started: Instant::now(),
            worktree_path,
            branch,
            agent_instances: HashMap::new(),
            pending_reviews: HashSet::new(),
            compile_fail_count: 0,
            consecutive_revise_count: 0,
            doc_revision_count: 0,
            code_verdict: String::new(),
            doc_verdict: String::new(),
            completed: false,
            failed: false,
            config,
            instance_prefix,
            repo_root: None,
        }
    }

    /// Set the repo root for event logging.
    pub fn set_repo_root(&mut self, repo_root: PathBuf) {
        self.repo_root = Some(repo_root);
    }

    /// Get the instance ID for a role in this pipeline
    pub fn instance_id(&self, role: AgentRole) -> String {
        format!("{}:{}", role, self.instance_prefix)
    }

    /// Get the working directory (worktree or repo root)
    pub fn working_dir(&self, repo_root: &PathBuf) -> PathBuf {
        self.worktree_path
            .clone()
            .unwrap_or_else(|| repo_root.clone())
    }

    /// Process an event and return actions for the executor
    pub fn handle_event(&mut self, event: PipelineEvent) -> Vec<PipelineAction> {
        match (&self.phase, event) {
            // Preflight -> Strategist (or skip to Implementer if trivial)
            (PipelinePhase::Preflight, PipelineEvent::AgentCompleted { .. }) => {
                if self.config.run_strategist {
                    self.transition(PipelinePhase::Strategist)
                } else {
                    self.transition(PipelinePhase::Implementer)
                }
            }
            // Strategist -> Implementer
            (PipelinePhase::Strategist, PipelineEvent::AgentCompleted { .. }) => {
                self.transition(PipelinePhase::Implementer)
            }
            // Implementer -> CompileGate
            (PipelinePhase::Implementer, PipelineEvent::AgentCompleted { .. }) => {
                self.transition(PipelinePhase::CompileGate)
            }
            // CompileGate pass -> DependencyDenyCheck
            (PipelinePhase::CompileGate, PipelineEvent::GatePassed { .. }) => {
                self.compile_fail_count = 0;
                self.transition(PipelinePhase::DependencyDenyCheck)
            }
            // DependencyDenyCheck pass -> TestGate
            (PipelinePhase::DependencyDenyCheck, PipelineEvent::GatePassed { .. }) => {
                self.transition(PipelinePhase::TestGate)
            }
            // DependencyDenyCheck fail -> back to Implementer
            (PipelinePhase::DependencyDenyCheck, PipelineEvent::GateFailed { output, .. }) => {
                self.compile_fail_count += 1;
                if self.compile_fail_count >= 3 {
                    self.failed = true;
                    vec![PipelineAction::Halt {
                        reason: format!("Dependency deny failed 3 times: {}", output),
                    }]
                } else {
                    self.transition(PipelinePhase::Implementer)
                }
            }
            // CompileGate fail -> try autofix for simple errors, otherwise Implementer
            (PipelinePhase::CompileGate, PipelineEvent::GateFailed { output, .. }) => {
                self.compile_fail_count += 1;
                if self.compile_fail_count >= 3 {
                    self.failed = true;
                    vec![PipelineAction::Halt {
                        reason: format!("Compile failed 3 times: {}", output),
                    }]
                } else {
                    // Try autofix: if all errors are simple with rustc suggestions, apply them directly
                    let errors = autofix::parse_cargo_json_errors(&output);
                    let all_simple = !errors.is_empty() && errors.iter().all(|e| e.is_simple());
                    if all_simple {
                        self.transition(PipelinePhase::AutoFix {
                            fix_plan: output.clone(),
                        })
                    } else {
                        self.transition(PipelinePhase::Implementer)
                    }
                }
            }
            // TestGate pass -> IgnoredTestCheck
            (PipelinePhase::TestGate, PipelineEvent::GatePassed { .. }) => {
                self.transition(PipelinePhase::IgnoredTestCheck)
            }
            // IgnoredTestCheck pass -> SpecComplianceCheck (invariant gate)
            (PipelinePhase::IgnoredTestCheck, PipelineEvent::GatePassed { .. }) => {
                self.transition(PipelinePhase::SpecComplianceCheck)
            }
            // IgnoredTestCheck fail -> back to Implementer
            (PipelinePhase::IgnoredTestCheck, PipelineEvent::GateFailed { output, .. }) => {
                self.compile_fail_count += 1;
                if self.compile_fail_count >= 3 {
                    self.failed = true;
                    vec![PipelineAction::Halt {
                        reason: format!("Ignored test enforcement failed 3 times: {}", output),
                    }]
                } else {
                    self.transition(PipelinePhase::Implementer)
                }
            }
            // TestGate fail -> try autofix for warning-only issues, otherwise Implementer
            (PipelinePhase::TestGate, PipelineEvent::GateFailed { output, .. }) => {
                self.compile_fail_count += 1;
                if self.compile_fail_count >= 3 {
                    self.failed = true;
                    vec![PipelineAction::Halt {
                        reason: format!("Tests failed 3 times: {}", output),
                    }]
                } else {
                    // Check if failures are just unused imports/variables (fixable by cargo fix)
                    let has_only_warnings =
                        output.contains("warning:") && !output.contains("error[E");
                    if has_only_warnings {
                        self.transition(PipelinePhase::AutoFix {
                            fix_plan: output.clone(),
                        })
                    } else {
                        self.transition(PipelinePhase::Implementer)
                    }
                }
            }
            // SpecComplianceCheck (invariant gate) pass -> Reviewing (or skip to Committing)
            (PipelinePhase::SpecComplianceCheck, PipelineEvent::GatePassed { .. }) => {
                if self.config.run_reviews {
                    self.transition(PipelinePhase::Reviewing)
                } else {
                    self.transition(PipelinePhase::Committing)
                }
            }
            // SpecComplianceCheck fail -> back to Implementer (code bug) or continue (spec issue)
            (PipelinePhase::SpecComplianceCheck, PipelineEvent::GateFailed { output, .. }) => {
                // If the output indicates only spec issues (#[ignore] SPEC_ISSUE),
                // treat as non-blocking and proceed to Reviewing.
                // Otherwise, it's a code bug — re-run implementer.
                if output.contains("SPEC_ISSUE") && !output.contains("FAILED") {
                    self.transition(PipelinePhase::Reviewing)
                } else {
                    self.compile_fail_count += 1;
                    if self.compile_fail_count >= 3 {
                        self.failed = true;
                        vec![PipelineAction::Halt {
                            reason: format!("Invariant gate failed 3 times: {}", output),
                        }]
                    } else {
                        self.transition(PipelinePhase::Implementer)
                    }
                }
            }
            // Reviewing -> when all reviews done -> Verdict
            (PipelinePhase::Reviewing, PipelineEvent::AgentCompleted { role, .. }) => {
                self.pending_reviews.remove(&role);
                if self.pending_reviews.is_empty() {
                    self.transition(PipelinePhase::Verdict)
                } else {
                    vec![]
                }
            }
            // Verdict -> Committing or DocRevision or QuickFix or back to Implementer
            (PipelinePhase::Verdict, PipelineEvent::VerdictReceived { verdict, .. }) => {
                // Try structured review parsing for smarter routing
                if let Some(structured) = super::review::parse_structured_review(&verdict) {
                    use super::review::VerdictStatus;
                    match structured.verdict.overall {
                        VerdictStatus::Approve | VerdictStatus::Skip => {
                            return self.transition(PipelinePhase::Committing);
                        }
                        VerdictStatus::Revise => {
                            // Code approved but docs need revision -> DocRevision only
                            if structured.verdict.code == VerdictStatus::Approve
                                && structured.verdict.docs == VerdictStatus::Revise
                            {
                                self.doc_revision_count += 1;
                                return self.transition(PipelinePhase::DocRevision);
                            }
                            // All blocking issues are quick-fixable -> QuickFix path
                            if structured.all_issues_quick_fixable() {
                                self.consecutive_revise_count += 1;
                                if self.consecutive_revise_count >= self.config.max_iterations {
                                    let mut actions = vec![PipelineAction::ReviewCapHit {
                                        plan_base: self.plan.base.clone(),
                                        iterations: self.consecutive_revise_count,
                                    }];
                                    actions.extend(self.transition(PipelinePhase::Committing));
                                    return actions;
                                }
                                return self.transition(PipelinePhase::QuickFix);
                            }
                            // Complex issues -> full Strategist re-run
                            self.consecutive_revise_count += 1;
                            if self.consecutive_revise_count >= self.config.max_iterations {
                                let mut actions = vec![PipelineAction::ReviewCapHit {
                                    plan_base: self.plan.base.clone(),
                                    iterations: self.consecutive_revise_count,
                                }];
                                actions.extend(self.transition(PipelinePhase::Committing));
                                return actions;
                            }
                            return self.transition(PipelinePhase::Implementer);
                        }
                    }
                }

                // Fallback: string-based verdict parsing (legacy reviews without TOML)
                if verdict.contains("approve") || verdict.contains("pass") {
                    self.transition(PipelinePhase::Committing)
                } else if verdict.contains("doc") {
                    self.doc_revision_count += 1;
                    self.transition(PipelinePhase::DocRevision)
                } else {
                    self.consecutive_revise_count += 1;
                    if self.consecutive_revise_count >= self.config.max_iterations {
                        let mut actions = vec![PipelineAction::ReviewCapHit {
                            plan_base: self.plan.base.clone(),
                            iterations: self.consecutive_revise_count,
                        }];
                        actions.extend(self.transition(PipelinePhase::Committing));
                        return actions;
                    } else {
                        self.transition(PipelinePhase::Implementer)
                    }
                }
            }
            // DocRevision -> Committing
            (PipelinePhase::DocRevision, PipelineEvent::AgentCompleted { .. }) => {
                self.transition(PipelinePhase::Committing)
            }
            // Committing -> Complete
            (PipelinePhase::Committing, PipelineEvent::AgentCompleted { .. }) => {
                self.completed = true;
                self.transition(PipelinePhase::Complete)
            }
            // AutoFix completed -> re-check with CompileGate
            (PipelinePhase::AutoFix { .. }, PipelineEvent::AgentCompleted { .. }) => {
                self.transition(PipelinePhase::CompileGate)
            }
            // QuickFix completed -> re-enter gates (skip full review if issues were mechanical)
            (PipelinePhase::QuickFix, PipelineEvent::AgentCompleted { .. }) => {
                self.transition(PipelinePhase::CompileGate)
            }
            // Error handling
            (_, PipelineEvent::AgentError { error, .. }) => {
                self.failed = true;
                vec![PipelineAction::Halt { reason: error }]
            }
            (_, PipelineEvent::PhaseTimeout) => {
                self.failed = true;
                vec![PipelineAction::Halt {
                    reason: "Phase timeout".to_string(),
                }]
            }
            _ => vec![],
        }
    }

    fn transition(&mut self, new_phase: PipelinePhase) -> Vec<PipelineAction> {
        let old_label = self.phase.label();
        self.phase = new_phase;
        self.phase_started = Instant::now();
        // Log the phase transition if we have a repo root
        if let Some(ref root) = self.repo_root {
            if let Err(e) = event_log::log_event(
                root,
                &self.plan.num,
                "phase_transition",
                &[("from", old_label), ("to", self.phase.label())],
            ) {
                warn!("Failed to log phase transition: {e}");
            }
        }
        // Return empty -- the executor reads the new phase and spawns appropriate agents
        vec![]
    }

    /// Whether this pipeline is still running
    pub fn is_active(&self) -> bool {
        !self.completed && !self.failed
    }

    /// Elapsed seconds since pipeline started
    pub fn elapsed_secs(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64()
    }

    /// Elapsed seconds in current phase
    pub fn phase_elapsed_secs(&self) -> f64 {
        self.phase_started.elapsed().as_secs_f64()
    }
}

/// Find which pipeline owns a given agent instance
pub fn find_pipeline_for_instance(pipelines: &[PlanPipeline], instance_id: &str) -> Option<usize> {
    pipelines
        .iter()
        .position(|p| p.agent_instances.values().any(|id| id == instance_id))
}

// ---------------------------------------------------------------------------
// Pre-planning context
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PrePlanStatus {
    Pending,
    Running,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct PrePlanContext {
    pub plan_base: String,
    pub workspace_map: String,
    pub suggested_tasks: Option<String>,
    pub file_dependencies: Vec<String>,
    pub estimated_minutes: Option<u32>,
    pub status: PrePlanStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan::{PlanFrontmatter, PlanInfo};

    fn test_plan() -> PlanInfo {
        PlanInfo {
            base: "06-terminal-nav".to_string(),
            num: "06".to_string(),
            path: std::path::PathBuf::from("plans/06-terminal-nav.md"),
            frontmatter: Some(PlanFrontmatter {
                plan: Some("06-terminal-nav".to_string()),
                depends_on: vec![],
                parallel_with: vec![],
                crates_touched: vec![],
                estimated_tasks: None,
                estimated_parallel_width: None,
                estimated_minutes: Some(30),
                refactor_after: false,
                parallel_safe: true,
                tasks: vec![],
            }),
        }
    }

    fn make_pipeline() -> PlanPipeline {
        PlanPipeline::new(
            test_plan(),
            0,
            None,
            "codex/plan/06-terminal-nav".to_string(),
        )
    }

    #[test]
    fn full_pipeline_flow_with_new_phases() {
        let mut p = make_pipeline();

        // Preflight -> Implementer (strategist replaced by bardo-enrich.sh offline pipeline)
        p.phase = PipelinePhase::Preflight;
        p.handle_event(PipelineEvent::AgentCompleted {
            role: AgentRole::Conductor,
            instance_id: "pre".into(),
        });
        assert_eq!(p.phase, PipelinePhase::Implementer);

        // Implementer -> CompileGate
        p.handle_event(PipelineEvent::AgentCompleted {
            role: AgentRole::Implementer,
            instance_id: "impl".into(),
        });
        assert_eq!(p.phase, PipelinePhase::CompileGate);

        // CompileGate pass -> DependencyDenyCheck
        p.handle_event(PipelineEvent::GatePassed {
            gate: GateType::Compile,
        });
        assert_eq!(p.phase, PipelinePhase::DependencyDenyCheck);

        // DependencyDenyCheck pass -> TestGate
        p.handle_event(PipelineEvent::GatePassed {
            gate: GateType::DependencyDeny,
        });
        assert_eq!(p.phase, PipelinePhase::TestGate);

        // TestGate pass -> IgnoredTestCheck
        p.handle_event(PipelineEvent::GatePassed {
            gate: GateType::Test,
        });
        assert_eq!(p.phase, PipelinePhase::IgnoredTestCheck);

        // IgnoredTestCheck pass -> SpecComplianceCheck
        p.handle_event(PipelineEvent::GatePassed {
            gate: GateType::IgnoredTestEnforcement,
        });
        assert_eq!(p.phase, PipelinePhase::SpecComplianceCheck);

        // SpecComplianceCheck pass -> Committing (run_reviews=false for default test plan)
        p.handle_event(PipelineEvent::GatePassed {
            gate: GateType::InvariantGate,
        });
        assert_eq!(p.phase, PipelinePhase::Committing);
    }

    #[test]
    fn dependency_deny_fail_routes_to_implementer() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::DependencyDenyCheck;

        p.handle_event(PipelineEvent::GateFailed {
            gate: GateType::DependencyDeny,
            output: "GPL dependency found".into(),
        });
        assert_eq!(p.phase, PipelinePhase::Implementer);
        assert_eq!(p.compile_fail_count, 1);
    }

    #[test]
    fn ignored_test_fail_routes_to_implementer() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::IgnoredTestCheck;

        p.handle_event(PipelineEvent::GateFailed {
            gate: GateType::IgnoredTestEnforcement,
            output: "test still fails".into(),
        });
        assert_eq!(p.phase, PipelinePhase::Implementer);
        assert_eq!(p.compile_fail_count, 1);
    }

    #[test]
    fn review_cap_emits_alert_then_commits() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::Verdict;
        p.consecutive_revise_count = 2; // will become 3

        let actions = p.handle_event(PipelineEvent::VerdictReceived {
            role: AgentRole::Architect,
            verdict: "revise: needs more work".into(),
        });

        assert_eq!(p.phase, PipelinePhase::Committing);
        assert_eq!(p.consecutive_revise_count, 3);

        // Should have ReviewCapHit action
        let has_alert = actions.iter().any(|a| {
            matches!(a, PipelineAction::ReviewCapHit { plan_base, iterations }
                if plan_base == "06-terminal-nav" && *iterations == 3)
        });
        assert!(has_alert, "Expected ReviewCapHit action");
    }

    #[test]
    fn review_under_cap_revises_normally() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::Verdict;
        p.consecutive_revise_count = 0;

        let actions = p.handle_event(PipelineEvent::VerdictReceived {
            role: AgentRole::Architect,
            verdict: "revise: needs more work".into(),
        });

        assert_eq!(p.phase, PipelinePhase::Implementer);
        assert_eq!(p.consecutive_revise_count, 1);
        // No ReviewCapHit
        let has_alert = actions
            .iter()
            .any(|a| matches!(a, PipelineAction::ReviewCapHit { .. }));
        assert!(!has_alert);
    }

    #[test]
    fn compile_fail_simple_routes_to_autofix() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::CompileGate;

        // Simulate a simple import error with JSON output
        let json_output = r#"{"reason":"compiler-message","message":{"code":{"code":"E0432"},"message":"unresolved import `foo::Bar`","level":"error","spans":[{"file_name":"src/lib.rs","line_start":1,"line_end":1,"text":[{"text":"use foo::Bar;"}],"suggested_replacement":null,"is_primary":true}],"children":[]}}"#;

        p.handle_event(PipelineEvent::GateFailed {
            gate: GateType::Compile,
            output: json_output.to_string(),
        });
        assert!(matches!(p.phase, PipelinePhase::AutoFix { .. }));
    }

    #[test]
    fn autofix_completed_returns_to_compile_gate() {
        let mut p = make_pipeline();
        p.phase = PipelinePhase::AutoFix {
            fix_plan: "test".into(),
        };

        p.handle_event(PipelineEvent::AgentCompleted {
            role: AgentRole::Conductor,
            instance_id: "fix".into(),
        });
        assert_eq!(p.phase, PipelinePhase::CompileGate);
    }
}

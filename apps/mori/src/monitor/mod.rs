pub mod config;
pub mod patterns;
pub mod steering;

use std::time::Instant;
use crate::agent::AgentRole;

/// A detected pattern with a suggested intervention
#[derive(Debug, Clone)]
pub struct Intervention {
    pub pattern: String,
    pub target_role: AgentRole,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Monitor pool managing all pattern matchers
pub struct MonitorPool {
    matchers: Vec<Box<dyn patterns::PatternMatcher>>,
    history: Vec<Intervention>,
    cooldowns: std::collections::HashMap<String, Instant>,
    cooldown_duration: std::time::Duration,
}

impl MonitorPool {
    pub fn new(cooldown_secs: u64) -> Self {
        Self {
            matchers: vec![
                Box::new(patterns::CompileFailRepeat::new()),
                Box::new(patterns::ContextWindowPressure::new()),
                Box::new(patterns::StuckPattern::new()),
                Box::new(patterns::TokenBurnRate::new()),
                Box::new(patterns::PhaseTimeout::new()),
                Box::new(patterns::TerminalUnresponsive::new()),
                Box::new(patterns::TerminalRenderRegression::new()),
                Box::new(patterns::GolemLifecycleViolation::new()),
                Box::new(patterns::SpecDriftAccumulation::new()),
                Box::new(patterns::CoverageDrop::new()),
                Box::new(patterns::CrossCrateBreak::new()),
                Box::new(patterns::SpecWeakeningDetector::new()),
            ],
            history: Vec::new(),
            cooldowns: std::collections::HashMap::new(),
            cooldown_duration: std::time::Duration::from_secs(cooldown_secs),
        }
    }

    /// Feed an event to all pattern matchers, return any triggered interventions
    pub fn check(&mut self, event: &MonitorEvent) -> Vec<Intervention> {
        let mut interventions = Vec::new();

        for matcher in &mut self.matchers {
            if let Some(intervention) = matcher.check(event) {
                let pattern_name = intervention.pattern.clone();

                // Check cooldown
                if let Some(last_fire) = self.cooldowns.get(&pattern_name) {
                    if last_fire.elapsed() < self.cooldown_duration {
                        continue;
                    }
                }

                self.cooldowns.insert(pattern_name, Instant::now());
                self.history.push(intervention.clone());
                interventions.push(intervention);
            }
        }

        interventions
    }

    pub fn history(&self) -> &[Intervention] {
        &self.history
    }

    pub fn matchers(&self) -> &[Box<dyn patterns::PatternMatcher>] {
        &self.matchers
    }
}

/// Events fed to the monitor system
#[derive(Debug, Clone)]
pub enum MonitorEvent {
    CompileGateResult { passed: bool, error: String },
    TestGateResult { passed: bool, passed_count: u32, failed_count: u32 },
    TokenUsage { role: AgentRole, input_tokens: u64, output_tokens: u64 },
    AgentOutput { role: AgentRole, content: String },
    PhaseElapsed { phase: String, elapsed_secs: u64 },
    /// Tracks cumulative output tokens over time for burn rate detection
    TokenBurnSnapshot { role: AgentRole, output_tokens: u64, elapsed_secs: u64 },
    TerminalHealthCheck { responsive: bool, active_screen: String },
    TerminalRenderResult { screen: String, assertions_total: u32, assertions_passed: u32, failures: Vec<String> },
    GolemLifecycleEvent { scenario: String, passed: bool, events_emitted: u32, detail: String },
    SpecDriftReport { types_checked: u32, types_drifted: u32, drift_pct: f64 },
    RegressionDetected { test_name: String, crate_name: String, error: String },
    CoverageChange { crate_name: String, before_pct: f64, after_pct: f64, delta: f64 },
    PlanTaskUpdate { plan_num: String, task_id: String, from_status: String, to_status: String },
    CrossCrateConflict { upstream: String, downstream: String, conflict_type: String, detail: String },
    /// Fired when a git diff shows removed assert lines in a spec-severity invariant test.
    InvariantTestWeakened { test_fn: String, invariant_id: String, removed_assertions: u32 },
}

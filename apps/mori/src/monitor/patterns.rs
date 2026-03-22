use std::collections::HashMap;
use crate::agent::AgentRole;
use super::{Intervention, MonitorEvent};

/// Trait for pattern matchers
pub trait PatternMatcher: Send {
    fn name(&self) -> &str;
    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention>;
    fn status(&self) -> PatternStatus;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternStatus {
    Watching,
    Triggered,
    Cooldown,
}

/// Detect same compile error repeated 3+ times
pub struct CompileFailRepeat {
    fail_count: u32,
    last_error: String,
    status: PatternStatus,
}

impl CompileFailRepeat {
    pub fn new() -> Self {
        Self {
            fail_count: 0,
            last_error: String::new(),
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for CompileFailRepeat {
    fn name(&self) -> &str { "CompileFailRepeat" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::CompileGateResult { passed, error } = event {
            if !passed {
                if *error == self.last_error {
                    self.fail_count += 1;
                } else {
                    self.fail_count = 1;
                    self.last_error = error.clone();
                }

                if self.fail_count >= 3 {
                    self.status = PatternStatus::Triggered;
                    let short_error = error.lines().take(5).collect::<Vec<_>>().join("\n");
                    return Some(Intervention {
                        pattern: self.name().to_string(),
                        target_role: AgentRole::Implementer,
                        message: format!(
                            "The same compilation error has occurred {} times. Error:\n{}\n\nTry a different approach to fix it.",
                            self.fail_count, short_error
                        ),
                        timestamp: chrono::Utc::now(),
                    });
                }
            } else {
                self.fail_count = 0;
                self.last_error.clear();
                self.status = PatternStatus::Watching;
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Detect context window pressure (>80% usage)
pub struct ContextWindowPressure {
    threshold: f64,
    context_limit: u64,
    status: PatternStatus,
}

impl ContextWindowPressure {
    pub fn new() -> Self {
        Self {
            threshold: 0.8,
            context_limit: 200_000, // default context window
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for ContextWindowPressure {
    fn name(&self) -> &str { "ContextWindowPressure" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::TokenUsage { role, input_tokens, .. } = event {
            let usage_ratio = *input_tokens as f64 / self.context_limit as f64;
            if usage_ratio > self.threshold {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: *role,
                    message: format!(
                        "Context window at {:.0}%. Checkpoint your work to CONTEXT.md now and wrap up.",
                        usage_ratio * 100.0
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Detect agent repeating the same approach
pub struct StuckPattern {
    recent_chunks: Vec<String>,
    status: PatternStatus,
}

impl StuckPattern {
    pub fn new() -> Self {
        Self {
            recent_chunks: Vec::new(),
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for StuckPattern {
    fn name(&self) -> &str { "StuckPattern" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::AgentOutput { role, content } = event {
            // Track chunks of output, look for repetition
            if content.len() > 50 {
                let chunk = content.chars().take(100).collect::<String>();
                let repeat_count = self.recent_chunks.iter().filter(|c| *c == &chunk).count();
                self.recent_chunks.push(chunk);

                // Keep only last 20 chunks
                if self.recent_chunks.len() > 20 {
                    self.recent_chunks.remove(0);
                }

                if repeat_count >= 3 {
                    self.status = PatternStatus::Triggered;
                    return Some(Intervention {
                        pattern: self.name().to_string(),
                        target_role: *role,
                        message: "You appear to be repeating the same approach. Try a different strategy.".to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Detect high token burn rate with low productivity.
/// If an agent is generating >1000 output tokens/minute but the phase hasn't
/// advanced (no gate passed, no files committed), it's likely spinning.
pub struct TokenBurnRate {
    snapshots: Vec<(u64, u64)>, // (output_tokens, elapsed_secs)
    status: PatternStatus,
}

impl TokenBurnRate {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for TokenBurnRate {
    fn name(&self) -> &str { "TokenBurnRate" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::TokenBurnSnapshot { role, output_tokens, elapsed_secs } = event {
            self.snapshots.push((*output_tokens, *elapsed_secs));

            // Need at least 2 snapshots to compute rate
            if self.snapshots.len() < 2 {
                return None;
            }

            // Keep last 10 snapshots
            if self.snapshots.len() > 10 {
                self.snapshots.remove(0);
            }

            let first = &self.snapshots[0];
            let last = &self.snapshots[self.snapshots.len() - 1];
            let token_delta = last.0.saturating_sub(first.0);
            let time_delta = last.1.saturating_sub(first.1);

            if time_delta == 0 {
                return None;
            }

            // Tokens per minute
            let tpm = (token_delta as f64 / time_delta as f64) * 60.0;

            // High burn: >2000 tokens/min sustained over 3+ minutes
            if tpm > 2000.0 && time_delta > 180 {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: *role,
                    message: format!(
                        "High token burn rate: {:.0} tokens/min over {}s. Stop and evaluate: are you making progress or spinning? If the same error keeps recurring, try a fundamentally different approach.",
                        tpm, time_delta
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Detect implementer phase running too long without completing.
/// After 15 minutes, nudge the agent to wrap up.
pub struct PhaseTimeout {
    threshold_secs: u64,
    status: PatternStatus,
}

impl PhaseTimeout {
    pub fn new() -> Self {
        Self {
            threshold_secs: 900, // 15 minutes
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for PhaseTimeout {
    fn name(&self) -> &str { "PhaseTimeout" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::PhaseElapsed { phase, elapsed_secs } = event {
            if (phase == "implementer" || phase == "strategist") && *elapsed_secs > self.threshold_secs {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: if phase == "strategist" { AgentRole::Strategist } else { AgentRole::Implementer },
                    message: format!(
                        "Phase '{}' has been running for {} minutes. Wrap up your current work now — commit what you have and let the gates verify it. Partial progress that compiles is better than a perfect implementation that times out.",
                        phase, elapsed_secs / 60
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Counts consecutive TerminalHealthCheck { responsive: false }. At 3+, fires intervention.
pub struct TerminalUnresponsive {
    consecutive_failures: u32,
    status: PatternStatus,
}

impl TerminalUnresponsive {
    pub fn new() -> Self {
        Self { consecutive_failures: 0, status: PatternStatus::Watching }
    }
}

impl PatternMatcher for TerminalUnresponsive {
    fn name(&self) -> &str { "TerminalUnresponsive" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::TerminalHealthCheck { responsive, .. } = event {
            if !responsive {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= 3 {
                    self.status = PatternStatus::Triggered;
                    return Some(Intervention {
                        pattern: self.name().to_string(),
                        target_role: AgentRole::TerminalValidator,
                        message: format!(
                            "Terminal unresponsive for {} consecutive health checks.",
                            self.consecutive_failures
                        ),
                        timestamp: chrono::Utc::now(),
                    });
                }
            } else {
                self.consecutive_failures = 0;
                self.status = PatternStatus::Watching;
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Tracks per-screen pass/fail. Fires when a screen that previously passed now fails.
pub struct TerminalRenderRegression {
    screen_passed: HashMap<String, bool>,
    status: PatternStatus,
}

impl TerminalRenderRegression {
    pub fn new() -> Self {
        Self { screen_passed: HashMap::new(), status: PatternStatus::Watching }
    }
}

impl PatternMatcher for TerminalRenderRegression {
    fn name(&self) -> &str { "TerminalRenderRegression" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::TerminalRenderResult { screen, assertions_total, assertions_passed, .. } = event {
            let passed = *assertions_passed == *assertions_total;
            let previously_passed = self.screen_passed.get(screen).copied().unwrap_or(false);
            self.screen_passed.insert(screen.clone(), passed);

            if previously_passed && !passed {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: AgentRole::TerminalValidator,
                    message: format!(
                        "Terminal render regression on screen '{}': was passing, now {}/{} assertions pass.",
                        screen, assertions_passed, assertions_total
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Counts GolemLifecycleEvent { passed: false }. At 3+, fires intervention.
pub struct GolemLifecycleViolation {
    fail_count: u32,
    status: PatternStatus,
}

impl GolemLifecycleViolation {
    pub fn new() -> Self {
        Self { fail_count: 0, status: PatternStatus::Watching }
    }
}

impl PatternMatcher for GolemLifecycleViolation {
    fn name(&self) -> &str { "GolemLifecycleViolation" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::GolemLifecycleEvent { passed, scenario, .. } = event {
            if !passed {
                self.fail_count += 1;
                if self.fail_count >= 3 {
                    self.status = PatternStatus::Triggered;
                    return Some(Intervention {
                        pattern: self.name().to_string(),
                        target_role: AgentRole::GolemLifecycleTester,
                        message: format!(
                            "Golem lifecycle failures reached {} (latest scenario: '{}').",
                            self.fail_count, scenario
                        ),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Tracks cumulative drift_pct. Fires when it exceeds 25.0.
pub struct SpecDriftAccumulation {
    cumulative_drift: f64,
    status: PatternStatus,
}

impl SpecDriftAccumulation {
    pub fn new() -> Self {
        Self { cumulative_drift: 0.0, status: PatternStatus::Watching }
    }
}

impl PatternMatcher for SpecDriftAccumulation {
    fn name(&self) -> &str { "SpecDriftAccumulation" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::SpecDriftReport { drift_pct, .. } = event {
            self.cumulative_drift += drift_pct;
            if self.cumulative_drift > 25.0 {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: AgentRole::SpecDriftDetector,
                    message: format!(
                        "Cumulative spec drift has reached {:.1}%, exceeding the 25% threshold.",
                        self.cumulative_drift
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Fires on CoverageChange where delta < -5.0.
pub struct CoverageDrop {
    status: PatternStatus,
}

impl CoverageDrop {
    pub fn new() -> Self {
        Self { status: PatternStatus::Watching }
    }
}

impl PatternMatcher for CoverageDrop {
    fn name(&self) -> &str { "CoverageDrop" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::CoverageChange { crate_name, delta, before_pct, after_pct, .. } = event {
            if *delta < -5.0 {
                self.status = PatternStatus::Triggered;
                return Some(Intervention {
                    pattern: self.name().to_string(),
                    target_role: AgentRole::CoverageTracker,
                    message: format!(
                        "Coverage dropped by {:.1}% in '{}' ({:.1}% -> {:.1}%).",
                        delta.abs(), crate_name, before_pct, after_pct
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Detects when a spec-severity invariant test has had assertions removed or weakened.
/// Fires on InvariantTestWeakened events (generated when git diff shows removed assert
/// lines in test files corresponding to severity: spec invariants).
pub struct SpecWeakeningDetector {
    weakened_count: u32,
    status: PatternStatus,
}

impl SpecWeakeningDetector {
    pub fn new() -> Self {
        Self {
            weakened_count: 0,
            status: PatternStatus::Watching,
        }
    }
}

impl PatternMatcher for SpecWeakeningDetector {
    fn name(&self) -> &str { "SpecWeakeningDetector" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::InvariantTestWeakened { test_fn, invariant_id, removed_assertions } = event {
            self.weakened_count += 1;
            self.status = PatternStatus::Triggered;
            return Some(Intervention {
                pattern: self.name().to_string(),
                target_role: AgentRole::Auditor,
                message: format!(
                    "SPEC WEAKENING DETECTED: Test '{}' ({}) had {} assertion(s) removed. \
                     Spec-severity invariants must not be weakened. Fix the implementation, not the test.",
                    test_fn, invariant_id, removed_assertions
                ),
                timestamp: chrono::Utc::now(),
            });
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

/// Fires immediately on any CrossCrateConflict.
pub struct CrossCrateBreak {
    status: PatternStatus,
}

impl CrossCrateBreak {
    pub fn new() -> Self {
        Self { status: PatternStatus::Watching }
    }
}

impl PatternMatcher for CrossCrateBreak {
    fn name(&self) -> &str { "CrossCrateBreak" }

    fn check(&mut self, event: &MonitorEvent) -> Option<Intervention> {
        if let MonitorEvent::CrossCrateConflict { upstream, downstream, conflict_type, detail } = event {
            self.status = PatternStatus::Triggered;
            return Some(Intervention {
                pattern: self.name().to_string(),
                target_role: AgentRole::CrossSystemTester,
                message: format!(
                    "Cross-crate conflict: {} -> {} ({}): {}",
                    upstream, downstream, conflict_type, detail
                ),
                timestamp: chrono::Utc::now(),
            });
        }
        None
    }

    fn status(&self) -> PatternStatus { self.status.clone() }
}

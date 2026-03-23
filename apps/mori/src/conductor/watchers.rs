use std::time::Duration;

use super::{ConductorAction, ConductorConfig, ConductorContext, Intervention, InterventionTier};
use crate::agent::AgentRole;

/// Set of all watchers
pub struct WatcherSet {
    silence_timeout: Duration,
    compile_fail_threshold: u32,
    task_stall_timeout: Duration,
    context_pressure_ratio: f64,
    phase_timeout: Duration,
    test_pass_budget_ratio: f64,
}

impl WatcherSet {
    pub fn new(config: &ConductorConfig) -> Self {
        Self {
            silence_timeout: config.silence_timeout,
            compile_fail_threshold: config.compile_fail_threshold,
            task_stall_timeout: config.task_stall_timeout,
            context_pressure_ratio: config.context_pressure_ratio,
            phase_timeout: config.phase_timeout,
            test_pass_budget_ratio: config.test_pass_budget_ratio,
        }
    }

    pub fn check(&self, ctx: &ConductorContext) -> Vec<Intervention> {
        let mut results = Vec::new();

        // === D3: Ghost turn detector (relaxed) ===
        // Fire on no output + fast turn alone (removed git_changes condition).
        if !ctx.last_turn_had_output && ctx.last_turn_duration_secs < 5 {
            if let Some(role) = ctx.active_role {
                results.push(Intervention {
                    tier: InterventionTier::Restart,
                    watcher: "GhostTurn".to_string(),
                    target_role: role,
                    message: "Agent completed turn instantly with no output. Restarting agent."
                        .to_string(),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::RestartAgent { role }),
                });
            }
        }

        // === Review loop detector ===
        // If we've had 3+ consecutive REVISE verdicts but compile/tests pass,
        // the reviewers are stuck on non-blocking issues. Skip them.
        if ctx.consecutive_revise_count >= 3 && ctx.orchestrator_state == "reviewing" {
            results.push(Intervention {
                tier: InterventionTier::Abort,
                watcher: "ReviewLoop".to_string(),
                target_role: AgentRole::Architect,
                message: format!(
                    "Review loop detected: {} consecutive REVISE verdicts with passing gates. Skipping reviews.",
                    ctx.consecutive_revise_count
                ),
                timestamp: chrono::Utc::now(),
                action: Some(ConductorAction::SkipReviews),
            });
        }

        // === Iteration loop detector ===
        // If iteration >= 4 and we're cycling through strategist/implementer again,
        // the cycle is not converging. Force advance.
        if ctx.iteration >= 6
            && (ctx.orchestrator_state == "strategist" || ctx.orchestrator_state == "implementer")
        {
            results.push(Intervention {
                tier: InterventionTier::Abort,
                watcher: "IterationLoop".to_string(),
                target_role: AgentRole::Implementer,
                message: format!(
                    "Iteration {} — pipeline is looping without convergence. Force advancing.",
                    ctx.iteration
                ),
                timestamp: chrono::Utc::now(),
                action: Some(ConductorAction::ForceAdvance),
            });
        }

        // === Test failure budget ===
        // If compile gates passed and we have test results, check the pass rate.
        // When the ratio of passing tests meets the budget threshold, force-advance
        // rather than retrying — remaining failures are likely name mismatches or
        // unrealistic test specs, not actionable bugs.
        let total_tests = ctx.test_pass_count + ctx.test_fail_count;
        if ctx.compile_gates_passed
            && total_tests > 0
            && ctx.test_fail_count > 0
            && (ctx.orchestrator_state == "test_gate" || ctx.orchestrator_state == "reviewing")
        {
            let pass_rate = ctx.test_pass_count as f64 / total_tests as f64;
            if pass_rate >= self.test_pass_budget_ratio {
                results.push(Intervention {
                    tier: InterventionTier::Abort,
                    watcher: "TestFailureBudget".to_string(),
                    target_role: AgentRole::Implementer,
                    message: format!(
                        "Test pass rate {}/{} ({:.0}%) meets budget threshold ({:.0}%). \
                         {} failing test(s) treated as non-actionable. Force advancing.",
                        ctx.test_pass_count,
                        total_tests,
                        pass_rate * 100.0,
                        self.test_pass_budget_ratio * 100.0,
                        ctx.test_fail_count,
                    ),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::ForceAdvance),
                });
            }
        }

        // Need an active role for the remaining watchers
        let role = match ctx.active_role {
            Some(r) => r,
            None => return results,
        };

        // === Agent silence watcher (D4: double timeout for Claude backend) ===
        let effective_silence_timeout =
            if matches!(ctx.agent_backend, Some(crate::agent::AgentBackend::Claude)) {
                self.silence_timeout * 2
            } else {
                self.silence_timeout
            };
        if let Some(last_msg) = ctx.last_message_at {
            if last_msg.elapsed() > effective_silence_timeout {
                results.push(Intervention {
                    tier: InterventionTier::Nudge,
                    watcher: "AgentSilence".to_string(),
                    target_role: role,
                    message: format!(
                        "No output for {} seconds. Are you stuck? Summarize your current state and continue.",
                        last_msg.elapsed().as_secs()
                    ),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::SendMessage {
                        role,
                        message: "You've been silent for too long. Summarize what you've done so far and continue working. If you're stuck, try a different approach.".to_string(),
                    }),
                });
            }
        }

        // === D1: Compile repeated failure — graduated escalation ===
        if ctx.compile_fail_count >= self.compile_fail_threshold {
            let short_error: String = ctx
                .last_compile_error
                .lines()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");

            let (tier, action) = if ctx.compile_fail_count >= 7 {
                // D1: 7+ failures → force advance
                (InterventionTier::Abort, ConductorAction::ForceAdvance)
            } else if ctx.compile_fail_count >= 5 {
                // D1: 5+ failures → restart implementer
                (
                    InterventionTier::Restart,
                    ConductorAction::RestartAgent {
                        role: AgentRole::Implementer,
                    },
                )
            } else {
                // 3+ failures → nudge
                (InterventionTier::Nudge, ConductorAction::SendMessage {
                    role: AgentRole::Implementer,
                    message: format!(
                        "COMPILATION HAS FAILED {} TIMES. The same approach is not working. You must:\n1. Read the exact error message\n2. Identify the root cause (not a symptom)\n3. Try a completely different implementation strategy\n\nError:\n{}",
                        ctx.compile_fail_count, short_error
                    ),
                })
            };

            results.push(Intervention {
                tier,
                watcher: "CompileFailRepeat".to_string(),
                target_role: AgentRole::Implementer,
                message: format!(
                    "Compilation has failed {} times.\nError:\n{}",
                    ctx.compile_fail_count, short_error
                ),
                timestamp: chrono::Utc::now(),
                action: Some(action),
            });
        }

        // === Task progress stall ===
        if let Some(last_change) = ctx.task_last_change {
            if last_change.elapsed() > self.task_stall_timeout {
                results.push(Intervention {
                    tier: InterventionTier::Nudge,
                    watcher: "TaskStall".to_string(),
                    target_role: role,
                    message: "No task progress for 10 minutes.".to_string(),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::SendMessage {
                        role,
                        message: "No task status change for 10 minutes. Update your task progress in the tasks TOML file and continue working.".to_string(),
                    }),
                });
            }
        }

        // === D2: Context window pressure — graduated ===
        if ctx.context_limit > 0 {
            let ratio = ctx.input_tokens as f64 / ctx.context_limit as f64;
            if ratio > 0.95 {
                // D2: 95% — hard stop, restart agent with fresh context
                results.push(Intervention {
                    tier: InterventionTier::Restart,
                    watcher: "ContextPressure".to_string(),
                    target_role: role,
                    message: format!("Context window at {:.0}% — hard reset.", ratio * 100.0),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::RestartAgent { role }),
                });
            } else if ratio > self.context_pressure_ratio {
                results.push(Intervention {
                    tier: InterventionTier::Nudge,
                    watcher: "ContextPressure".to_string(),
                    target_role: role,
                    message: format!("Context window at {:.0}%.", ratio * 100.0),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::SendMessage {
                        role,
                        message: format!(
                            "Your context window is at {:.0}% capacity. Wrap up your current task immediately. Write your progress to CONTEXT.md and finish your turn.",
                            ratio * 100.0
                        ),
                    }),
                });
            }
        }

        // === A4: Phase timeout with escalation ===
        if let Some(started) = ctx.phase_started {
            if started.elapsed() > self.phase_timeout {
                results.push(Intervention {
                    tier: InterventionTier::Restart,
                    watcher: "PhaseTimeout".to_string(),
                    target_role: role,
                    message: format!(
                        "Phase running for {} minutes.",
                        started.elapsed().as_secs() / 60
                    ),
                    timestamp: chrono::Utc::now(),
                    // A4: after 2 restarts on same phase, force advance
                    action: Some(ConductorAction::RestartAgent { role }),
                });
            }
        }

        // === Task-continuation watcher ===
        // Fires when: the batch implementer finished its assigned tasks but more
        // queued tasks are ready in the same plan. Instead of cold-starting a new
        // agent, send a message to the warm implementer so it picks up more work.
        if let (Some(ref summary), Some(ref instance_id)) =
            (&ctx.task_summary, &ctx.active_instance_id)
        {
            if summary.queued > 0 && summary.in_flight == 0 && role == AgentRole::Implementer {
                results.push(Intervention {
                    tier: InterventionTier::Nudge,
                    watcher: "TaskContinuation".to_string(),
                    target_role: AgentRole::Implementer,
                    message: format!(
                        "Plan {} has {} more task(s) queued — assigning to warm agent {}",
                        summary.plan, summary.queued, instance_id,
                    ),
                    timestamp: chrono::Utc::now(),
                    action: Some(ConductorAction::AssignAdditionalTasks {
                        instance_id: instance_id.clone(),
                        task_descriptions: vec![format!(
                            "Plan {} has {} additional task(s) ready. Continue implementing them now.",
                            summary.plan, summary.queued,
                        )],
                    }),
                });
            }
        }

        results
    }
}

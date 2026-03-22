use std::time::Instant;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use crate::agent::{AgentPool, AgentRole};
use crate::conductor::Conductor;
use crate::git::GitManager;
use crate::orchestrator::phase::Verdict;
use crate::orchestrator::{Orchestrator, OrchestratorState};
use crate::state::persistence::PersistenceManager;
use crate::state::{ConductorHistoryEntry, LogLevel, RunPlanStatus, RunState};

use super::*;

/// Implementer done — start compile gate as a background task (non-blocking).
#[allow(clippy::too_many_arguments)]
pub(crate) fn start_compile_gate(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    config: &AppConfig,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
) {
    orchestrator.set_state(OrchestratorState::CompileGate);
    state.orchestrator_state = "compile-gate".to_string();
    transition_phase(state, "compile-gate");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "compile-gate".to_string();
    }
    state
        .gate_running
        .insert("cargo check --workspace".to_string());
    state.add_log("gate", "Running: cargo check --workspace", LogLevel::Info);

    state.command_output.clear();

    let repo = config.repo_root.clone();
    let tx = gate_tx.clone();
    let clippy_enabled = state.config.clippy_enabled;
    let plan_name = orchestrator
        .current_plan()
        .map(|p| p.base.clone())
        .unwrap_or_default();
    tokio::spawn(async move {
        let gate_start = std::time::Instant::now();
        // Format gate first (auto-fix, fast)
        let _ = crate::orchestrator::gates::format_gate(&repo).await;
        // If clippy is enabled, use it as the compile gate (clippy is a superset of check,
        // and running both causes cache invalidation between them — a full rebuild)
        let result = if clippy_enabled {
            crate::orchestrator::gates::clippy_compile_gate(&repo, &plan_name).await
        } else {
            crate::orchestrator::gates::compile_gate(&repo, &plan_name).await
        };
        let elapsed = gate_start.elapsed();
        tracing::info!(
            "gate[compile][{}]: completed in {:.1}s",
            plan_name,
            elapsed.as_secs_f64()
        );
        let _ = tx.send(GateCompletion::Compile {
            plan: plan_name,
            result,
        });
    });
}

/// Handle compile gate result (received via gate_rx channel).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_compile_result(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
    compile_fail_count: &mut u32,
    last_compile_error: &mut String,
    gate: crate::orchestrator::gates::GateResult,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    state.last_gate_output = gate.output.clone();
    state.last_gate_passed = gate.passed;

    // Persist structured error digest (or raw output as fallback) for iteration context.
    // The error digest gives agents targeted signal: unique errors with file:line references,
    // instead of pages of raw compiler output that wastes context tokens.
    let gate_path = config.repo_root.join("plans/context/last-gate-output.txt");
    let feedback = gate.error_digest.as_deref().unwrap_or(&gate.output);
    let _ = std::fs::write(&gate_path, feedback);

    // Append structured error digest to implementer's agent output
    let summary = if let Some(ref digest) = gate.error_digest {
        format!("\n--- compile-gate FAIL ✗ ---\n{}\n", digest,)
    } else {
        format!(
            "\n--- compile-gate {} ---\n{}\n",
            if gate.passed { "PASS ✓" } else { "FAIL ✗" },
            gate.output
                .lines()
                .rev()
                .take(30)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
        )
    };
    state
        .agent_state_mut(AgentRole::Implementer)
        .output
        .push_str(&summary);

    // Consult conductor on gate result
    let gate_desc = format!(
        "Compile gate {}",
        if gate.passed { "PASSED" } else { "FAILED" }
    );
    consult_conductor(state, agent_pool, &gate_desc).await;

    state.add_log(
        "gate",
        &format!(
            "Compile gate: {}",
            if gate.passed { "PASS" } else { "FAIL" }
        ),
        if gate.passed {
            LogLevel::Info
        } else {
            LogLevel::Error
        },
    );
    persistence.append_event(&PersistenceManager::make_event(
        if gate.passed {
            "phase_done"
        } else {
            "compile_fail"
        },
        &plan.base,
        "compile-gate",
        state.current_iteration,
    ))?;

    if !gate.passed {
        *compile_fail_count += 1;
        *last_compile_error = gate.output.clone();

        if state.current_iteration < config.max_iterations {
            let error_tail: String = gate
                .output
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ");
            state.iteration_reason = format!("Compile failed: {error_tail}");
            state.add_log(
                "gate",
                &format!(
                    "Compile failed at iter {}, cycling back",
                    state.current_iteration
                ),
                LogLevel::Warn,
            );
            crate::orchestrator::context::archive_iteration(
                &config.repo_root,
                &plan.num,
                state.current_iteration,
            )?;
            state.current_iteration += 1;
            orchestrator.current_iteration = state.current_iteration;
            if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
                entry.iteration = state.current_iteration;
            }

            // Reset agent threads on iteration to give fresh context windows.
            // Without this, conversation history from failed iterations accumulates
            // and by iter 5 the agent has ~75k chars of stale context (38% of budget)
            // that dilutes the error signal and causes repeated failures.
            agent_pool.reset_all_threads();
            state.add_log(
                "gate",
                &format!(
                    "Reset agent threads for iter {} (fresh context)",
                    state.current_iteration
                ),
                LogLevel::Info,
            );

            // Offline enrichment replaced strategist — go straight to implementer
            orchestrator.set_state(OrchestratorState::Implementer);
            state.orchestrator_state = "implementer".to_string();
            transition_phase(state, "implementer");
            if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
                entry.phase = "implementer".to_string();
            }
            agent_pool
                .spawn(
                    AgentRole::Implementer,
                    state.config.effort_for(AgentRole::Implementer).label(),
                    state.config.model_for(AgentRole::Implementer),
                )
                .await?;
            let prompt =
                crate::orchestrator::prompts::implementer_prompt(&config.repo_root, &plan)?;
            agent_pool
                .turn_start(
                    AgentRole::Implementer,
                    &prompt,
                    state.config.model_for(AgentRole::Implementer),
                )
                .await?;
            state.agent_state_mut(AgentRole::Implementer).active = true;
            return Ok(());
        }
        // Max iterations reached — halt
        let reason = format!(
            "Compile gate failed after {} iterations",
            config.max_iterations
        );
        state.add_log("orch", &reason, LogLevel::Error);
        orchestrator.set_state(OrchestratorState::Halted {
            reason: reason.clone(),
        });
        state.orchestrator_state = "halted".to_string();
        state.error = Some(reason);
        if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
            entry.status = RunPlanStatus::Failed;
        }
        // Don't set complete = true — allow user to force-advance or restart
        return Ok(());
    }

    *compile_fail_count = 0;
    last_compile_error.clear();

    // Fire conditional gates (non-blocking, informational) based on plan's crates_touched
    if let Some(plan_info) = orchestrator.current_plan() {
        let plan_path = config
            .repo_root
            .join("plans")
            .join(format!("{}.md", plan_info.base));
        let plan_text = std::fs::read_to_string(&plan_path).unwrap_or_default();

        if crate::orchestrator::gates::plan_touches_crate(&plan_text, "bardo-terminal") {
            let repo = config.repo_root.clone();
            let tx = gate_tx.clone();
            let pn = plan_info.base.clone();
            tokio::spawn(async move {
                let result = crate::orchestrator::gates::terminal_render_gate(&repo).await;
                let _ = tx.send(GateCompletion::TerminalRender { plan: pn, result });
            });
            state.gate_running.insert("terminal render".to_string());
        }

        if crate::orchestrator::gates::plan_touches_crate(&plan_text, "golem-") {
            let repo = config.repo_root.clone();
            let tx = gate_tx.clone();
            let pn = plan_info.base.clone();
            tokio::spawn(async move {
                let result = crate::orchestrator::gates::golem_lifecycle_gate(&repo).await;
                let _ = tx.send(GateCompletion::GolemLifecycle { plan: pn, result });
            });
            state.gate_running.insert("golem lifecycle".to_string());
        }
    }

    // Clippy already ran as part of the compile gate (clippy_compile_gate),
    // so skip the separate clippy pass and go straight to tests.
    if !config.skip_tests {
        start_test_gate(state, orchestrator, config, gate_tx);
    } else if !config.no_review {
        start_parallel_reviews(state, orchestrator, agent_pool, persistence, config).await?;
    } else if !config.no_docs && state.config.scribe_enabled {
        // No code reviewers, but scribe still runs after gates pass
        orchestrator.set_state(OrchestratorState::Reviewing);
        state.orchestrator_state = "reviewing".to_string();
        state.pending_reviews.clear();
        handle_reviewers_done(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
        )
        .await?;
    } else {
        commit_and_advance(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
            &plan,
        )
        .await?;
    }

    Ok(())
}

/// Start test gate as a background task (non-blocking).
pub(crate) fn start_test_gate(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    config: &AppConfig,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
) {
    orchestrator.set_state(OrchestratorState::TestGate);
    state.orchestrator_state = "test-gate".to_string();
    transition_phase(state, "test-gate");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "test-gate".to_string();
    }
    state
        .gate_running
        .insert("cargo nextest run --workspace".to_string());
    state.add_log("gate", "Running: cargo test", LogLevel::Info);

    let repo = config.repo_root.clone();
    let tx = gate_tx.clone();
    let plan_name = orchestrator
        .current_plan()
        .map(|p| p.base.clone())
        .unwrap_or_default();
    tokio::spawn(async move {
        let gate_start = std::time::Instant::now();
        let result = crate::orchestrator::gates::test_gate(&repo, 900).await;
        let elapsed = gate_start.elapsed();
        tracing::info!(
            "gate[test][{}]: completed in {:.1}s",
            plan_name,
            elapsed.as_secs_f64()
        );
        let _ = tx.send(GateCompletion::Test {
            plan: plan_name,
            result,
        });
    });
}

/// Start clippy gate as a background task (non-blocking, always passes).
pub(crate) fn start_clippy_gate(
    state: &mut RunState,
    config: &AppConfig,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
) {
    state
        .gate_running
        .insert("cargo clippy --workspace".to_string());
    state.add_log("gate", "Running: cargo clippy --workspace", LogLevel::Info);
    state.command_output.clear();

    let repo = config.repo_root.clone();
    let tx = gate_tx.clone();
    // Clippy gate doesn't go through orchestrator plan phase, use empty plan
    let plan_name = String::new();
    tokio::spawn(async move {
        let gate_start = std::time::Instant::now();
        let result = crate::orchestrator::gates::clippy_gate(&repo).await;
        let elapsed = gate_start.elapsed();
        tracing::info!(
            "gate[clippy][{}]: completed in {:.1}s",
            plan_name,
            elapsed.as_secs_f64()
        );
        let _ = tx.send(GateCompletion::Clippy {
            plan: plan_name,
            result,
        });
    });
}

/// Handle test gate result (received via gate_rx channel).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_test_result(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    gate: crate::orchestrator::gates::GateResult,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    state.last_gate_output = gate.output.clone();
    // Track gate pass/fail for max-iteration guard
    state.last_gate_passed = gate.passed;

    // Append gate output summary to implementer's agent output
    let summary = format!(
        "\n--- test-gate {} ---\n{}\n",
        if gate.passed { "PASS ✓" } else { "FAIL ✗" },
        gate.output
            .lines()
            .rev()
            .take(30)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n"),
    );
    state
        .agent_state_mut(AgentRole::Implementer)
        .output
        .push_str(&summary);

    // Consult conductor on test result
    let test_desc = format!(
        "Test gate {}",
        if gate.passed { "PASSED" } else { "FAILED" }
    );
    consult_conductor(state, agent_pool, &test_desc).await;

    state.add_log(
        "gate",
        &format!("Test gate: {}", if gate.passed { "PASS" } else { "FAIL" }),
        if gate.passed {
            LogLevel::Info
        } else {
            LogLevel::Warn
        },
    );
    persistence.append_event(&PersistenceManager::make_event(
        if gate.passed {
            "phase_done"
        } else {
            "test_fail"
        },
        &plan.base,
        "test-gate",
        state.current_iteration,
    ))?;

    if !gate.passed {
        state.add_log(
            "gate",
            "Test failures detected (non-blocking)",
            LogLevel::Warn,
        );

        // Log failing tests as deferred failures for future batch remediation
        let failing_names = crate::orchestrator::gates::extract_failing_test_names(&gate.output);
        let snippet = crate::orchestrator::gates::extract_test_failure_snippet(&gate.output, 50);
        if !failing_names.is_empty() {
            let failures: Vec<crate::state::persistence::DeferredFailure> = failing_names
                .iter()
                .map(|name| {
                    crate::state::persistence::DeferredFailure {
                        plan: plan.base.clone(),
                        task_id: String::new(), // not from verify-tasks; raw test gate
                        title: format!("Test failure: {}", name),
                        task_type: "test_gate".to_string(),
                        command: "cargo test".to_string(),
                        test_fns: vec![name.clone()],
                        reason: crate::state::persistence::DeferredReason::NonBlocking,
                        error_snippet: snippet.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        iteration: state.current_iteration,
                    }
                })
                .collect();

            let batch_id = batch_branch
                .strip_prefix("codex/batch/")
                .unwrap_or(batch_branch);
            if let Err(e) = persistence.append_deferred_failures(batch_id, failures) {
                state.add_log(
                    "gate",
                    &format!("Failed to write deferred failures: {e}"),
                    LogLevel::Warn,
                );
            } else {
                state.add_log(
                    "gate",
                    &format!(
                        "Logged {} failing test(s) to deferred-failures.toml",
                        failing_names.len()
                    ),
                    LogLevel::Info,
                );
            }
        }
    }

    if !config.no_review {
        // Check if the implementer actually produced meaningful output
        let impl_output_len = state
            .agent_state(AgentRole::Implementer)
            .map(|a| a.output.len())
            .unwrap_or(0);

        // After 5+ iterations with gates passing AND real agent output, skip reviews
        if state.current_iteration >= 5 && impl_output_len > 500 {
            state.add_log(
                "orch",
                &format!(
                    "Iteration {} with gates passing — skipping reviews, auto-committing",
                    state.current_iteration
                ),
                LogLevel::Warn,
            );
            commit_and_advance(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
                &plan,
            )
            .await?;
        } else {
            start_parallel_reviews(state, orchestrator, agent_pool, persistence, config).await?;
        }
    } else if !config.no_docs && state.config.scribe_enabled {
        // No code reviewers, but scribe still runs after gates pass
        orchestrator.set_state(OrchestratorState::Reviewing);
        state.orchestrator_state = "reviewing".to_string();
        state.pending_reviews.clear();
        handle_reviewers_done(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
        )
        .await?;
    } else {
        commit_and_advance(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
            &plan,
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn handle_strategist_done(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    persistence: &PersistenceManager,
    config: &AppConfig,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    state.add_log(
        "orch",
        "Strategist complete, starting implementer",
        LogLevel::Info,
    );

    orchestrator.set_state(OrchestratorState::Implementer);
    state.orchestrator_state = "implementer".to_string();
    transition_phase(state, "implementer");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "implementer".to_string();
    }

    agent_pool
        .spawn(
            AgentRole::Implementer,
            state.config.effort_for(AgentRole::Implementer).label(),
            state.config.model_for(AgentRole::Implementer),
        )
        .await?;
    let prompt = crate::orchestrator::prompts::implementer_prompt_with_brief(
        &config.repo_root,
        &plan,
        state.current_iteration,
    )?;
    agent_pool
        .turn_start(
            AgentRole::Implementer,
            &prompt,
            state.config.model_for(AgentRole::Implementer),
        )
        .await?;
    state.agent_state_mut(AgentRole::Implementer).active = true;

    persistence.append_event(&PersistenceManager::make_event(
        "phase_start",
        &plan.base,
        "implementer",
        state.current_iteration,
    ))?;

    Ok(())
}

/// Start parallel reviews: Architect + Auditor + Scribe (if docs enabled)
pub(crate) async fn start_parallel_reviews(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    persistence: &PersistenceManager,
    config: &AppConfig,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    orchestrator.set_state(OrchestratorState::Reviewing);
    state.orchestrator_state = "reviewing".to_string();
    transition_phase(state, "reviewing");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "reviewing".to_string();
    }

    state.pending_reviews.clear();

    match crate::orchestrator::context::regenerate_workspace_map(&config.repo_root) {
        Ok(_) => state.add_log(
            "orch",
            "Refreshed workspace map before reviews",
            LogLevel::Info,
        ),
        Err(e) => state.add_log(
            "orch",
            &format!("Failed to refresh workspace map: {e}"),
            LogLevel::Warn,
        ),
    }

    if state.config.architect_enabled {
        state.pending_reviews.insert(AgentRole::Architect);
        agent_pool
            .spawn(
                AgentRole::Architect,
                state.config.effort_for(AgentRole::Architect).label(),
                state.config.model_for(AgentRole::Architect),
            )
            .await?;
        let arch_prompt = crate::orchestrator::prompts::combined_reviewer_prompt(
            &config.repo_root,
            &plan,
            state.current_iteration,
            None,
        )?;
        agent_pool
            .turn_start(
                AgentRole::Architect,
                &arch_prompt,
                state.config.model_for(AgentRole::Architect),
            )
            .await?;
        state.agent_state_mut(AgentRole::Architect).active = true;
    }

    // Scribe is spawned after code reviewers APPROVE (serialized, not parallel).
    // See handle_reviewers_done for the spawn logic.

    // If no reviewers enabled, skip straight to verdict
    if state.pending_reviews.is_empty() {
        state.add_log(
            "orch",
            "No reviewers enabled, skipping review phase",
            LogLevel::Info,
        );
        // Pending reviews empty means handle_reviewers_done check will fire immediately
        // so we don't need to do anything special here
    }

    let n = state.pending_reviews.len();
    state.add_log(
        "orch",
        &format!("Parallel review started ({n} agents)"),
        LogLevel::Info,
    );
    persistence.append_event(&PersistenceManager::make_event(
        "phase_start",
        &plan.base,
        "reviewing",
        state.current_iteration,
    ))?;

    Ok(())
}

/// All parallel reviewers done — serialize Scribe after code APPROVE, then Critic.
///
/// First invocation (Arch + Audit done, `state.code_verdict` is empty or "REVISE"):
///   - Read arch/audit verdicts.
///   - If code APPROVE and docs+scribe enabled: spawn Scribe, record verdict, return.
///   - Otherwise: call evaluate_verdict (handles REVISE → strategist, no-docs → commit).
///
/// Second invocation (Scribe done, `state.code_verdict == "APPROVE"`):
///   - Spawn Critic if docs enabled, else evaluate_verdict.
pub(crate) async fn handle_reviewers_done(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    // Second invocation: Scribe just finished (code_verdict already set to "APPROVE").
    if state.code_verdict == "APPROVE" {
        if !config.no_docs {
            orchestrator.set_state(OrchestratorState::CriticReview);
            state.orchestrator_state = "critic-review".to_string();
            transition_phase(state, "critic-review");
            if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
                entry.phase = "critic-review".to_string();
            }
            agent_pool
                .spawn(
                    AgentRole::Critic,
                    state.config.effort_for(AgentRole::Critic).label(),
                    state.config.model_for(AgentRole::Critic),
                )
                .await?;
            let critic_prompt =
                crate::orchestrator::prompts::critic_prompt(&config.repo_root, &plan, None)?;
            agent_pool
                .turn_start(
                    AgentRole::Critic,
                    &critic_prompt,
                    state.config.model_for(AgentRole::Critic),
                )
                .await?;
            state.agent_state_mut(AgentRole::Critic).active = true;
            state.add_log("orch", "Critic reviewing docs", LogLevel::Info);
        } else {
            evaluate_verdict(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
            )
            .await?;
        }
        return Ok(());
    }

    // First invocation: Reviewer just finished (or no reviewer existed).
    // If docs and scribe are enabled, check code verdict before spawning Scribe.
    if !config.no_docs && state.config.scribe_enabled {
        // When architect is disabled (no-review/express), code is implicitly approved by gates
        let code_approved = if !state.config.architect_enabled {
            true
        } else {
            use crate::orchestrator::phase::extract_verdict;
            let arch_content = std::fs::read_to_string(
                config
                    .repo_root
                    .join(format!("plans/context/reviews/{}-arch.md", plan.num)),
            )
            .unwrap_or_default();
            matches!(extract_verdict(&arch_content), Verdict::Approve)
        };

        if code_approved {
            state.code_verdict = "APPROVE".to_string();
            state.add_log("orch", "Code approved — spawning Scribe", LogLevel::Info);
            state.pending_reviews.insert(AgentRole::Scribe);
            agent_pool
                .spawn(
                    AgentRole::Scribe,
                    state.config.effort_for(AgentRole::Scribe).label(),
                    state.config.model_for(AgentRole::Scribe),
                )
                .await?;
            let scribe_prompt =
                crate::orchestrator::prompts::scribe_prompt(&config.repo_root, &plan, None)?;
            agent_pool
                .turn_start(
                    AgentRole::Scribe,
                    &scribe_prompt,
                    state.config.model_for(AgentRole::Scribe),
                )
                .await?;
            state.agent_state_mut(AgentRole::Scribe).active = true;
            return Ok(());
        }
    }

    // Code REVISE, or docs/scribe disabled: delegate to evaluate_verdict.
    evaluate_verdict(
        state,
        orchestrator,
        agent_pool,
        git_manager,
        persistence,
        config,
        batch_branch,
        run_id,
        started_at,
    )
    .await?;

    Ok(())
}

pub(crate) async fn handle_critic_done(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    let critic_path = config
        .repo_root
        .join(format!("plans/context/reviews/{}-critic.md", plan.num));
    let critic_content = std::fs::read_to_string(&critic_path).unwrap_or_default();
    let critic_verdict = crate::orchestrator::phase::extract_verdict(&critic_content);

    match &critic_verdict {
        Verdict::Approve => state.add_log("critic", "Docs approved", LogLevel::Info),
        Verdict::Revise { .. } => state.add_log("critic", "Docs need revision", LogLevel::Warn),
    }

    evaluate_verdict(
        state,
        orchestrator,
        agent_pool,
        git_manager,
        persistence,
        config,
        batch_branch,
        run_id,
        started_at,
    )
    .await
}

/// Check Architect + Auditor + Critic verdicts, decide commit, revise, or doc-revision
pub(crate) async fn evaluate_verdict(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
) -> Result<()> {
    use crate::orchestrator::phase::extract_verdict;

    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    orchestrator.set_state(OrchestratorState::Verdict);
    state.orchestrator_state = "verdict".to_string();
    transition_phase(state, "verdict");

    // Consult conductor on verdict evaluation
    consult_conductor(state, agent_pool, "Evaluating review verdicts").await;

    // Read all review files
    let arch_content = std::fs::read_to_string(
        config
            .repo_root
            .join(format!("plans/context/reviews/{}-arch.md", plan.num)),
    )
    .unwrap_or_default();
    let critic_content = std::fs::read_to_string(
        config
            .repo_root
            .join(format!("plans/context/reviews/{}-critic.md", plan.num)),
    )
    .unwrap_or_default();

    // Evaluate code verdict (combined reviewer)
    let arch_verdict = extract_verdict(&arch_content);
    let code_approved = match &arch_verdict {
        Verdict::Approve => true,
        Verdict::Revise { issues } => {
            if arch_content.len() < 50 {
                state.add_log(
                    "verdict",
                    "Reviewer empty output — treating as failed",
                    LogLevel::Error,
                );
            } else {
                state.add_log(
                    "verdict",
                    &format!("Reviewer REVISE ({} issues)", issues.len()),
                    LogLevel::Warn,
                );
            }
            false
        }
    };

    // Evaluate doc verdict (critic)
    let critic_approved = if critic_content.is_empty() || config.no_docs {
        true // no critic review or docs disabled = non-blocking
    } else {
        match extract_verdict(&critic_content) {
            Verdict::Approve => true,
            Verdict::Revise { issues } => {
                state.add_log(
                    "verdict",
                    &format!("Critic REVISE ({} issues)", issues.len()),
                    LogLevel::Warn,
                );
                false
            }
        }
    };

    state.code_verdict = if code_approved { "APPROVE" } else { "REVISE" }.into();
    state.doc_verdict = if critic_content.is_empty() || config.no_docs {
        String::new()
    } else if critic_approved {
        "APPROVE".into()
    } else {
        "REVISE".into()
    };

    state.add_log(
        "verdict",
        &format!(
            "code={}, docs={}",
            state.code_verdict,
            if state.doc_verdict.is_empty() {
                "N/A"
            } else {
                &state.doc_verdict
            },
        ),
        LogLevel::Info,
    );
    let doc_v_lower = if state.doc_verdict.is_empty() {
        "n/a".to_string()
    } else {
        state.doc_verdict.to_lowercase()
    };
    persistence.append_event(&PersistenceManager::make_event(
        "verdict",
        &plan.base,
        &format!(
            "code={} docs={}",
            state.code_verdict.to_lowercase(),
            doc_v_lower
        ),
        state.current_iteration,
    ))?;

    if code_approved && critic_approved {
        // All approved — commit
        state.consecutive_revise_count = 0;
        commit_and_advance(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
            &plan,
        )
        .await
    } else if !code_approved && state.current_iteration < config.max_iterations {
        // Code REVISE — full iteration (strategist -> implementer -> gates -> review)
        state.consecutive_revise_count += 1;
        let mut reasons = Vec::new();
        if !code_approved {
            reasons.push("Reviewer: REVISE".to_string());
        }
        state.iteration_reason = reasons.join(", ");

        // Reset tokens for fresh iteration
        for role in AgentRole::ALL_AGENTS {
            let agent = state.agent_state_mut(role);
            agent.input_tokens = 0;
            agent.output_tokens = 0;
        }

        crate::orchestrator::context::archive_iteration(
            &config.repo_root,
            &plan.num,
            state.current_iteration,
        )?;
        state.current_iteration += 1;
        orchestrator.current_iteration = state.current_iteration;
        if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
            entry.iteration = state.current_iteration;
        }

        // Fresh threads for revision — prevents stale context accumulation
        agent_pool.reset_all_threads();

        state.add_log(
            "orch",
            &format!(
                "Code revision requested, starting iteration {} (threads reset)",
                state.current_iteration
            ),
            LogLevel::Warn,
        );
        // Offline enrichment replaced strategist — go straight to implementer
        handle_strategist_done(state, orchestrator, agent_pool, persistence, config).await
    } else if code_approved && !critic_approved && state.doc_revision_count < 2 {
        // Code OK but docs REVISE — doc revision loop (scribe -> critic only)
        state.doc_revision_count += 1;
        state.iteration_reason = format!(
            "Doc revision {} — critic requested changes",
            state.doc_revision_count
        );
        state.add_log(
            "orch",
            &format!(
                "Doc revision {}/2 — restarting scribe with critic feedback",
                state.doc_revision_count
            ),
            LogLevel::Warn,
        );
        start_doc_revision(
            state,
            orchestrator,
            agent_pool,
            persistence,
            config,
            &critic_content,
        )
        .await
    } else if !code_approved && state.current_iteration >= config.max_iterations {
        // Max iterations reached for code — check gate results
        if state.last_gate_passed {
            state.add_log(
                "orch",
                &format!(
                "Max iterations ({}) reached. Gates passed — force committing despite code REVISE.",
                config.max_iterations
            ),
                LogLevel::Warn,
            );
            state.iteration_reason = format!(
                "Max iter {} — force commit (gates passed)",
                config.max_iterations
            );
            commit_and_advance(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
                &plan,
            )
            .await
        } else {
            let reason = format!(
                "Max iterations ({}) reached and gates still failing — halting",
                config.max_iterations
            );
            state.error = Some(reason.clone());
            state.add_log("orch", &reason, LogLevel::Error);
            orchestrator.set_state(OrchestratorState::Halted { reason });
            state.orchestrator_state = "halted".to_string();
            if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
                entry.status = RunPlanStatus::Failed;
            }
            Ok(())
        }
    } else {
        // Doc revision capped at 2 — commit with current docs
        state.add_log(
            "orch",
            "Doc revision capped at 2 — committing with current docs",
            LogLevel::Warn,
        );
        commit_and_advance(
            state,
            orchestrator,
            agent_pool,
            git_manager,
            persistence,
            config,
            batch_branch,
            run_id,
            started_at,
            &plan,
        )
        .await
    }
}

/// Start doc revision loop: restart scribe with critic feedback, then re-run critic
pub(crate) async fn start_doc_revision(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    persistence: &PersistenceManager,
    config: &AppConfig,
    critic_feedback: &str,
) -> Result<()> {
    let plan = match orchestrator.current_plan() {
        Some(p) => p.clone(),
        None => return Ok(()),
    };

    orchestrator.set_state(OrchestratorState::DocRevision);
    state.orchestrator_state = "doc-revision".to_string();
    transition_phase(state, "doc-revision");
    if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
        entry.phase = "doc-revision".to_string();
    }

    // Kill existing scribe if spawned, respawn with critic feedback
    if agent_pool.is_spawned(AgentRole::Scribe) {
        agent_pool.kill(AgentRole::Scribe).await;
    }
    agent_pool
        .spawn(
            AgentRole::Scribe,
            state.config.effort_for(AgentRole::Scribe).label(),
            state.config.model_for(AgentRole::Scribe),
        )
        .await?;
    let prompt = crate::orchestrator::prompts::doc_revision_prompt(
        &config.repo_root,
        &plan,
        critic_feedback,
        None,
    )?;
    agent_pool
        .turn_start(
            AgentRole::Scribe,
            &prompt,
            state.config.model_for(AgentRole::Scribe),
        )
        .await?;
    state.agent_state_mut(AgentRole::Scribe).active = true;

    persistence.append_event(&PersistenceManager::make_event(
        "phase_start",
        &plan.base,
        "doc-revision",
        state.current_iteration,
    ))?;

    Ok(())
}

/// Send a state snapshot to the conductor agent for assessment.
/// Non-blocking: the conductor's response will be handled when its turn completes.
pub(crate) async fn consult_conductor(
    state: &mut RunState,
    agent_pool: &mut AgentPool,
    event_description: &str,
) {
    if !agent_pool.is_spawned(AgentRole::Conductor) {
        return;
    }
    // Don't consult if conductor is still processing a previous message
    if state
        .agent_state(AgentRole::Conductor)
        .map(|a| a.active)
        .unwrap_or(false)
    {
        return;
    }
    let snapshot = crate::conductor::llm::state_snapshot(state, event_description);
    match agent_pool
        .turn_start(
            AgentRole::Conductor,
            &snapshot,
            state.config.model_for(AgentRole::Conductor),
        )
        .await
    {
        Ok(()) => {
            state.agent_state_mut(AgentRole::Conductor).active = true;
        }
        Err(e) => {
            state.add_log(
                "conductor",
                &format!("Failed to consult conductor: {e}"),
                LogLevel::Warn,
            );
        }
    }
}

/// Execute a parsed conductor directive.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_conductor_directive(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    conductor: &mut Conductor,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    directive: crate::conductor::llm::ConductorDirective,
) -> Result<()> {
    use crate::conductor::llm::ConductorDirective;
    match directive {
        ConductorDirective::Ok => {
            state.add_log(
                "conductor",
                "Assessment: OK — no action needed",
                LogLevel::Info,
            );
        }
        ConductorDirective::Nudge { role, message } => {
            state.add_log(
                "conductor",
                &format!(
                    "Nudging {role}: {}",
                    message.chars().take(80).collect::<String>()
                ),
                LogLevel::Warn,
            );
            if agent_pool.is_spawned(role) {
                let echo = format!("\n--- Conductor ---\n{message}\n-----------------\n");
                state.agent_state_mut(role).output.push_str(&echo);
                let _ = agent_pool.turn_interrupt(role).await;
                let _ = agent_pool.turn_start(role, &message, None).await;
                state.agent_state_mut(role).active = true;
            } else {
                state.add_log(
                    "conductor",
                    &format!("Nudge target {role} not spawned — directive dropped"),
                    LogLevel::Warn,
                );
            }
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: role.to_string(),
                message: message.chars().take(120).collect(),
            });
        }
        ConductorDirective::Restart { role } => {
            state.add_log("conductor", &format!("Restarting {role}"), LogLevel::Warn);
            agent_pool.kill(role).await;
            state.agent_state_mut(role).active = false;
            let _ =
                restart_current_phase(state, orchestrator, agent_pool, persistence, config).await;
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: role.to_string(),
                message: "Restart".to_string(),
            });
        }
        ConductorDirective::SkipReviews => {
            state.add_log("conductor", "Skipping reviews", LogLevel::Warn);
            for &r in &[
                AgentRole::Architect,
                AgentRole::Auditor,
                AgentRole::Scribe,
                AgentRole::Critic,
            ] {
                if agent_pool.is_spawned(r) {
                    agent_pool.kill(r).await;
                    state.agent_state_mut(r).active = false;
                }
            }
            state.pending_reviews.clear();
            if let Some(plan) = orchestrator.current_plan().cloned() {
                let _ = commit_and_advance(
                    state,
                    orchestrator,
                    agent_pool,
                    git_manager,
                    persistence,
                    config,
                    batch_branch,
                    run_id,
                    started_at,
                    &plan,
                )
                .await;
            }
        }
        ConductorDirective::ForceAdvance => {
            state.add_log("conductor", "Force advancing", LogLevel::Warn);
            let _ = force_advance(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
            )
            .await;
        }
        ConductorDirective::Throttle { limit } => {
            state.add_log(
                "conductor",
                &format!("Throttling agent limit to {limit}"),
                LogLevel::Warn,
            );
            conductor.rate_limiter.soft_limit = limit;
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: "system".to_string(),
                message: format!("Throttle → {limit}"),
            });
        }
        ConductorDirective::PrePlan { plan_num } => {
            state.add_log(
                "conductor",
                &format!("Pre-planning requested for plan {plan_num}"),
                LogLevel::Info,
            );
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: "pre-planner".to_string(),
                message: format!("Pre-plan {plan_num}"),
            });
        }
        ConductorDirective::Enrich { plan_num } => {
            state.add_log(
                "conductor",
                &format!("ENRICH {plan_num} — running bardo-enrich.sh"),
                LogLevel::Warn,
            );
            let root = config.repo_root.clone();
            let pn = plan_num.clone();
            let enrich_out = tokio::task::spawn_blocking(move || {
                let script = root.join("bardo-enrich.sh");
                if !script.is_file() {
                    return Err(format!("bardo-enrich.sh not found at {}", script.display()));
                }
                std::process::Command::new("bash")
                    .arg(&script)
                    .arg(&pn)
                    .current_dir(&root)
                    .status()
                    .map_err(|e| e.to_string())
            })
            .await;
            match enrich_out {
                Ok(Ok(status)) if status.success() => {
                    state.add_log(
                        "conductor",
                        &format!("ENRICH {plan_num}: bardo-enrich.sh OK"),
                        LogLevel::Info,
                    );
                }
                Ok(Ok(status)) => {
                    state.add_log(
                        "conductor",
                        &format!("ENRICH {plan_num}: bardo-enrich.sh failed ({status})"),
                        LogLevel::Error,
                    );
                }
                Ok(Err(e)) => {
                    state.add_log(
                        "conductor",
                        &format!("ENRICH {plan_num}: {e}"),
                        LogLevel::Error,
                    );
                }
                Err(e) => {
                    state.add_log(
                        "conductor",
                        &format!("ENRICH {plan_num}: join error {e}"),
                        LogLevel::Error,
                    );
                }
            }
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: "enrich".to_string(),
                message: format!("ENRICH {plan_num}"),
            });
        }
        ConductorDirective::Validate { target } => {
            state.add_log(
                "conductor",
                &format!("Validation requested: {target}"),
                LogLevel::Info,
            );
        }
        ConductorDirective::FixPlan { plan_num, reason } => {
            state.add_log(
                "conductor",
                &format!("Fix plan requested for {plan_num}: {reason}"),
                LogLevel::Warn,
            );
        }
        ConductorDirective::SkipValidation { target } => {
            state.add_log(
                "conductor",
                &format!("Skipping validation: {target}"),
                LogLevel::Warn,
            );
        }
        ConductorDirective::ResetReview(reason) => {
            state.add_log(
                "conductor",
                &format!(
                    "Resetting review phase: {}",
                    reason.chars().take(120).collect::<String>()
                ),
                LogLevel::Warn,
            );
            for &r in &[
                AgentRole::DocVerifier,
                AgentRole::Scribe,
                AgentRole::Critic,
                AgentRole::Auditor,
            ] {
                if agent_pool.is_spawned(r) {
                    agent_pool.kill(r).await;
                    state.agent_state_mut(r).active = false;
                }
            }
            state.pending_reviews.clear();
            // Restart review phase — re-inject corrective brief when reviewers respawn
            state.conductor_reset_brief = Some(format!("[CONDUCTOR RESET] {reason}"));
            let _ =
                restart_current_phase(state, orchestrator, agent_pool, persistence, config).await;
            state.conductor_history.push(ConductorHistoryEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                watcher: "LLM".to_string(),
                target: "review-phase".to_string(),
                message: format!(
                    "RESET_REVIEW: {}",
                    reason.chars().take(120).collect::<String>()
                ),
            });
        }
        ConductorDirective::SpawnReview { plan_base } => {
            // SpawnReview is only supported in parallel mode (app/parallel.rs)
            state.add_log(
                "conductor",
                &format!("SPAWN-REVIEW {plan_base} — not supported in sequential mode"),
                LogLevel::Warn,
            );
        }
        ConductorDirective::PhaseReject { plan, reason } => {
            state.add_log(
                "conductor",
                &format!("PHASE-REJECT {plan}: {reason} — not supported in sequential mode"),
                LogLevel::Warn,
            );
        }
        ConductorDirective::RetryPlan { plan } => {
            state.add_log(
                "conductor",
                &format!("RETRY-PLAN {plan} — not supported in sequential mode"),
                LogLevel::Warn,
            );
        }
    }
    Ok(())
}

pub(crate) async fn commit_and_advance(
    state: &mut RunState,
    orchestrator: &mut Orchestrator,
    agent_pool: &mut AgentPool,
    git_manager: &GitManager,
    persistence: &PersistenceManager,
    config: &AppConfig,
    batch_branch: &str,
    run_id: &str,
    started_at: &str,
    plan: &crate::orchestrator::PlanInfo,
) -> Result<()> {
    // Guard against double-dispatch: if already committing, bail out
    if state.committing_in_progress {
        state.add_log(
            "orch",
            "Commit already in progress — ignoring duplicate call",
            LogLevel::Warn,
        );
        return Ok(());
    }
    state.committing_in_progress = true;

    // Ensure flag is cleared even if inner operation fails
    let result = async {
        orchestrator.set_state(OrchestratorState::Committing);
        state.orchestrator_state = "committing".to_string();
        transition_phase(state, "committing");
        if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
            entry.phase = "committing".to_string();
        }

        let doc_v = if state.doc_verdict.is_empty() {
            "N/A"
        } else {
            &state.doc_verdict
        };
        let commit_msg = format!(
            "plan({}): {}\n\nIterations: {}\nCode verdict: {}\nDoc verdict: {}",
            plan.num, plan.base, state.current_iteration, state.code_verdict, doc_v,
        );
        match git_manager.commit_all(&commit_msg) {
            Ok(hash) => {
                state.add_log("git", &format!("Committed: {hash}"), LogLevel::Info);
            }
            Err(e) => {
                // Nothing to commit (agent may not have changed files)
                state.add_log("git", &format!("No changes to commit: {e}"), LogLevel::Warn);
            }
        }

        // Capture branch diff before merge
        if let Ok(diff) = git_manager.diff_branch("main") {
            state.branch_diff = diff;
        }

        // Merge and tag (ignore errors if no commit was made)
        if let Err(e) = git_manager.merge_plan_to_batch(&plan.base, batch_branch) {
            state.add_log("git", &format!("Merge skipped: {e}"), LogLevel::Warn);
        } else {
            state.add_log("git", &format!("Merged to {batch_branch}"), LogLevel::Info);
        }

        let tag = format!("plan/{}", plan.base);
        if let Err(e) = git_manager.tag(&tag) {
            state.add_log("git", &format!("Tag skipped: {e}"), LogLevel::Warn);
        } else {
            state.add_log("git", &format!("Tagged: {tag}"), LogLevel::Info);
        }

        // Record plan completion time in the estimator
        let actual_minutes = state
            .plans
            .get(orchestrator.current_plan_idx)
            .and_then(|p| p.started_at)
            .map(|t| (t.elapsed().as_secs() / 60) as u32)
            .unwrap_or(0);
        if let Some(entry) = state.plans.get_mut(orchestrator.current_plan_idx) {
            entry.status = RunPlanStatus::Completed;
            entry.phase = "complete".to_string();
            entry.actual_minutes = Some(actual_minutes);
        }
        state
            .time_estimator
            .record_plan_complete(&plan.base, actual_minutes);

        // Queue a particle burst at the approximate position of this plan in the list
        state.particle_burst_pending = Some((20.0, (state.current_plan_idx as f32 * 1.5) + 4.0));

        orchestrator.plans_completed += 1;
        persistence.append_event(&PersistenceManager::make_event(
            "plan_done",
            &plan.base,
            "",
            state.current_iteration,
        ))?;

        // Generate executive summary
        let total_elapsed: Duration = state.phase_elapsed.iter().map(|(_, d)| *d).sum();
        if let Ok(summary) = crate::orchestrator::context::generate_summary(
            &config.repo_root,
            plan,
            state.current_iteration,
            total_elapsed,
        ) {
            let _ =
                crate::orchestrator::context::write_summary(&config.repo_root, &plan.num, &summary);
        }

        // Toast notification
        state.notifications.push(Notification {
            message: format!("Plan {} completed — Enter to view summary", plan.base),
            created: Instant::now(),
            ttl_secs: 10,
            level: LogLevel::Info,
        });

        // Reset verdicts before advancing to the next plan to prevent stale state leaking
        state.code_verdict = String::new();
        state.doc_verdict = String::new();
        state.doc_revision_count = 0;

        if orchestrator.advance_to_next_plan() {
            state.orchestrator_state = "plan-ready".to_string();
            // Note: compile_fail_count is reset inside start_plan
            let mut compile_fail_count = 0u32;
            start_plan(
                state,
                orchestrator,
                agent_pool,
                git_manager,
                persistence,
                config,
                batch_branch,
                run_id,
                started_at,
                &mut compile_fail_count,
            )
            .await?;
        } else {
            state.orchestrator_state = "complete".to_string();
            state.complete = true;
            persistence.append_event(&PersistenceManager::make_event("run_complete", "", "", 0))?;
            state.add_log("orch", "All plans complete", LogLevel::Info);

            // Merge batch to staging branch for final review
            let staging_branch = format!("staging/{}", config.batch_id);
            if let Err(e) = git_manager.merge_batch_to_staging(batch_branch, &staging_branch) {
                state.add_log(
                    "git",
                    &format!("Staging merge failed: {e}"),
                    LogLevel::Error,
                );
            } else {
                state.add_log(
                    "git",
                    &format!("Merged to staging: {staging_branch}"),
                    LogLevel::Info,
                );
            }
        }

        Ok::<(), anyhow::Error>(())
    }
    .await;

    state.committing_in_progress = false;
    result
}

use std::time::Instant;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::agent::{AgentEvent, AgentPool, AgentRole};
use crate::conductor::Conductor;
use crate::git::GitManager;
use crate::orchestrator::{tasks, Orchestrator, OrchestratorState};
use crate::state::persistence::PersistenceManager;
use crate::state::{LogLevel, PendingApproval, RunState};

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_agent_event(
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
    event: AgentEvent,
    task_last_change: &mut Option<Instant>,
    gate_tx: &mpsc::UnboundedSender<GateCompletion>,
    turn_started_at: &mut Option<Instant>,
    last_turn_had_output: &mut bool,
    last_turn_duration_secs: &mut u64,
) -> Result<()> {
    match event {
        AgentEvent::MessageDelta { role, content, .. } => {
            state.agent_state_mut(role).output.push_str(&content);
            if turn_started_at.is_none() {
                *turn_started_at = Some(Instant::now());
            }
        }
        AgentEvent::CommandOutput { role, content, .. } => {
            state.command_output.push_str(&content);
            let key = format!("{} terminal", role.short());
            state.gate_running.insert(key);
        }
        AgentEvent::TurnCompleted {
            role, thread_id, ..
        } => {
            // Clear the terminal marker set by CommandOutput
            let key = format!("{} terminal", role.short());
            state.gate_running.remove(&key);
            info!("Turn completed for {role}");
            // Track turn metrics for conductor
            let output_len = state.agent_state(role).map(|a| a.output.len()).unwrap_or(0);
            *last_turn_had_output = output_len > 200;
            *last_turn_duration_secs = turn_started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
            *turn_started_at = None;
            state.agent_state_mut(role).active = false;
            // Insert turn separator
            {
                let agent = state.agent_state_mut(role);
                agent.turn_count += 1;
                let turn_num = agent.turn_count;
                let model = state.config.model_for(role).unwrap_or("?").to_string();
                let timestamp = chrono::Utc::now().format("%H:%M:%S").to_string();
                let label = role.label();
                state.agent_state_mut(role).output.push_str(&format!(
                    "\n──── turn {} · {} · {} · {} ────\n",
                    turn_num, label, model, timestamp
                ));
            }
            if let Some(tid) = thread_id {
                state.agent_state_mut(role).thread_id = Some(tid.clone());
                agent_pool.set_thread_id(role, Some(tid));
            }

            // Check if agent ended by asking user a question (breaks autonomous flow)
            // Auto-respond to keep the pipeline moving
            let output = state
                .agent_state(role)
                .map(|a| a.output.clone())
                .unwrap_or_default();
            let last_lines: String = output
                .lines()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let looks_like_question = last_lines.contains("If you want")
                || last_lines.contains("if you'd like")
                || last_lines.contains("Would you like")
                || last_lines.contains("Shall I")
                || last_lines.contains("want me to")
                || last_lines.contains("move on to")
                || last_lines.contains("ready to proceed")
                || last_lines.contains("Let me know");

            if looks_like_question && matches!(role, AgentRole::Implementer) {
                let count = state.auto_respond_count.entry(role).or_insert(0);
                *count += 1;
                if *count > 3 {
                    state.add_log(
                        "orch",
                        &format!(
                            "{role} asked again after 3 auto-responses — not responding further"
                        ),
                        LogLevel::Warn,
                    );
                } else {
                    state.add_log(
                        "conductor",
                        &format!(
                            "Auto-responding to {} question to maintain autonomous flow",
                            role
                        ),
                        LogLevel::Info,
                    );
                    let auto_msg = "Yes, proceed. This is a fully autonomous pipeline. Do not ask for confirmation. Complete all remaining work and finish your turn.";
                    let echo = format!("\n--- Conductor auto-response ---\n{auto_msg}\n-------------------------------\n");
                    state.agent_state_mut(role).output.push_str(&echo);
                    state.agent_state_mut(role).active = true;
                    let _ = agent_pool
                        .turn_start(role, auto_msg, state.config.model_for(role))
                        .await;
                    return Ok(());
                }
            }

            match (role, &orchestrator.state) {
                (AgentRole::Implementer, OrchestratorState::Implementer) => {
                    start_compile_gate(state, orchestrator, config, gate_tx);
                }
                (
                    AgentRole::Architect
                    | AgentRole::Auditor
                    | AgentRole::Scribe
                    | AgentRole::QuickReviewer,
                    OrchestratorState::Reviewing,
                ) => {
                    state.pending_reviews.remove(&role);
                    state.add_log(&role.to_string(), "Review complete", LogLevel::Info);
                    if state.pending_reviews.is_empty() {
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
                    }
                }
                (AgentRole::Critic, OrchestratorState::CriticReview) => {
                    handle_critic_done(
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
                // Doc revision flow: scribe done -> re-run critic
                (AgentRole::Scribe, OrchestratorState::DocRevision) => {
                    state.add_log("orch", "Scribe doc revision complete", LogLevel::Info);
                    // Check doc revision cap (max 2 iterations)
                    if state.doc_revision_count >= 2 {
                        state.add_log(
                            "orch",
                            "Doc revision cap reached (2 iterations) — committing and advancing",
                            LogLevel::Warn,
                        );
                        let Some(plan) = orchestrator.current_plan().cloned() else {
                            state.add_log(
                                "orch",
                                "No current plan for doc revision commit — skipping",
                                LogLevel::Warn,
                            );
                            return Ok(());
                        };
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
                        // Re-run critic on updated docs
                        state.doc_revision_count += 1;
                        if agent_pool.is_spawned(AgentRole::Critic) {
                            agent_pool.kill(AgentRole::Critic).await;
                        }
                        agent_pool
                            .spawn(
                                AgentRole::Critic,
                                state.config.effort_for(AgentRole::Critic).label(),
                                state.config.model_for(AgentRole::Critic),
                            )
                            .await?;
                        let Some(plan) = orchestrator.current_plan().cloned() else {
                            state.add_log(
                                "orch",
                                "No current plan for doc revision critic — skipping",
                                LogLevel::Warn,
                            );
                            return Ok(());
                        };
                        let critic_prompt = crate::orchestrator::prompts::critic_prompt(
                            &config.repo_root,
                            &plan,
                            None,
                        )?;
                        agent_pool
                            .turn_start(
                                AgentRole::Critic,
                                &critic_prompt,
                                state.config.model_for(AgentRole::Critic),
                            )
                            .await?;
                        state.agent_state_mut(AgentRole::Critic).active = true;
                    }
                }
                // Doc revision flow: critic re-reviewed -> evaluate verdict again
                (AgentRole::Critic, OrchestratorState::DocRevision) => {
                    state.add_log(
                        "orch",
                        "Critic re-review complete, re-evaluating verdict",
                        LogLevel::Info,
                    );
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
                // Conductor turn completed — parse and execute any directive
                (AgentRole::Conductor, _) => {
                    let output = state
                        .agent_state(AgentRole::Conductor)
                        .map(|a| a.output.clone())
                        .unwrap_or_default();
                    if let Some(directive) = crate::conductor::llm::parse_directive(&output) {
                        execute_conductor_directive(
                            state,
                            orchestrator,
                            agent_pool,
                            git_manager,
                            persistence,
                            config,
                            conductor,
                            batch_branch,
                            run_id,
                            started_at,
                            directive,
                        )
                        .await?;
                    }
                }
                _ => {}
            }
        }
        AgentEvent::DiffUpdated { role, diff, .. } => {
            state.agent_state_mut(role).diff = diff;
        }
        AgentEvent::ApprovalRequested {
            role,
            command,
            approval_id,
            ..
        } => {
            state.pending_approval = Some(PendingApproval {
                role,
                command: command.clone(),
                approval_id: approval_id.clone(),
            });
            state.add_log(
                &role.to_string(),
                &format!("Approval requested: {command}"),
                LogLevel::Warn,
            );
            agent_pool
                .respond_approval(role, &approval_id, true)
                .await?;
            state.pending_approval = None;
        }
        AgentEvent::TokenUsage {
            role,
            input_tokens,
            output_tokens,
            context_window,
            cost_usd,
            ..
        } => {
            let current_plan = {
                let agent = state.agent_state_mut(role);
                agent.input_tokens = input_tokens;
                agent.output_tokens = output_tokens;
                agent.current_plan.clone()
            };
            // Update context limit from actual model context window
            if let Some(window) = context_window {
                state.context_limit = window;
            }
            // Accumulate cost
            if let Some(cost) = cost_usd {
                state.cumulative_cost_usd += cost;
                if let Some(plan) = current_plan {
                    *state.cost_per_plan.entry(plan).or_insert(0.0) += cost;
                }
            }
        }
        AgentEvent::Error { role, error, .. } => {
            state.add_log(&role.to_string(), &error, LogLevel::Error);
            warn!("Agent error ({}): {}", role, error);
        }
        AgentEvent::Exited {
            role, exit_code, ..
        } => {
            state.add_log(
                &role.to_string(),
                &format!("Agent exited with code {:?}", exit_code),
                LogLevel::Warn,
            );
        }
    }
    Ok(())
}

use crate::agent::AgentRole;
use crate::state::RunState;

fn build_file_manifest() -> &'static str {
    r#"## File Manifest

### prd2 Specifications
- prd2/01-vision/ through prd2/23-ta/ — product requirements by domain
- prd2/16-testing/ — testing strategy (files 00-06 exist)
- prd2/17-monorepo/ — coding conventions, workspace structure
- prd2/18-interfaces/03-tui.md — TUI interface spec
- prd2/SUMMARY.md — executive summary of all domains

### Plans & Context
- plans/CONTEXT.md — cumulative type registry, decisions (D-001 to D-324+)
- plans/context/briefs/{NN}-brief.md — per-plan implementation briefs (machine- or LLM-generated)
- plans/context/prd2-extracts/{NN}-prd2.md — curated PRD2 excerpts per plan
- plans/context/decompositions/{NN}-decomposition.md — step breakdowns
- plans/context/tasks/{NN}-tasks.toml — implementer checklist; verify/review/scribe TOML variants
- plans/context/bundles/{NN}-bundle.md — distiller digest (optional)
- plans/context/review-context/{NN}-review-context.md — reviewer bundle (optional)
- plans/context/verify-chains/{NN}-verify.sh — invariant runner when present
- plans/context/type-registry.md — exported types snapshot (optional)
- plans/context/last-completed.md — summary from previous plan
- plans/context/ignored-tests.md — ledger of ignored tests
- plans/context/reviews/ — per-plan review verdicts ({num}-audit.md, {num}-arch.md, etc.)
- tmp/agent-messages.md — conductor/operator steering for active agents (if non-empty)
- plans/NN-name.md — individual plan files (imports/exports sections are the contracts)

### Crates
- crates/golem-core/ — core types and traits
- apps/bardo-terminal/ — TUI application
- (workspace: Cargo.toml at repo root lists all members)"#
}

/// Build the conductor system prompt — defines its role as meta-orchestrator.
pub fn conductor_system_prompt() -> String {
    format!(
        r#"You are the Conductor — the meta-orchestrator of an autonomous code generation pipeline called Bardo.

## Your Role

You are the supreme escalation authority. Your decisions override all other agents.

You observe the full state of a multi-agent pipeline that implements plans (code changes) in sequence. You have visibility into:
- Which plan is currently executing and its iteration count
- All agent outputs (strategist, implementer, architect, auditor, scribe, critic)
- Gate results (compile, test)
- Review verdicts and iteration history
- Token usage and context pressure

Your job is to monitor the pipeline and provide guidance when things go wrong or could go better. You do NOT write code yourself. You steer the other agents.

## When You're Consulted

You receive state summaries at key moments:
- After each phase transition
- When gates fail
- When reviews return REVISE
- When iterations seem stuck
- Periodically during long-running phases

## Responsibilities

- Monitor agent progress — detect stalls, loops, and ghost turns.
- You are the supreme escalation authority. Your decisions override all other agents.
- When doc writers and reviewers cycle without convergence: issue [RESET_REVIEW] with a specific corrective brief.
- When an agent needs spec context: cite the exact prd2 path from the file manifest in your NUDGE message.
- Detect context pressure and trigger thread resets.
- Identify compile/test failure loops and escalate.
- Track token budgets across all agents.
- Decide when to skip reviews, force-advance, or throttle concurrency.

## What You Can Do

Respond with your assessment and optionally one directive:

- `[NUDGE role] message` — Send a steering message to an agent
- `[RESTART role]` — Kill and restart an agent with a fresh context
- `[RESET_REVIEW] reason` — Kill all active review-phase agents (DocVerifier, Scribe, Critic, Auditor) and restart the review phase. The `reason` text is injected as a prefixed message to each restarted reviewer so they know what went wrong. Use when reviewers and writers have been cycling for 2+ iterations without convergence.
- `[SKIP-REVIEWS]` — Skip remaining reviews and commit
- `[FORCE-ADVANCE]` — Force commit and move to next plan
- `[OK]` — Everything looks fine, no action needed
- `[THROTTLE N]` — Set the agent soft limit to N (reduce parallelism if rate limited)
- `[ENRICH plan_num]` — Run `./bardo-enrich.sh <plan_num>` in the repo root (enhance-toml / optional enhance-plan pipeline) when decomposition, PRD2 extracts, or task TOMLs are missing or stale
- `[PHASE-REJECT plan_base] reason` — Reject the most recent phase transition for a plan and roll it back. Use when: implementation output is too short or wrong, gates passed vacuously, reviews were superficial, or merge had issues. The plan re-runs from the appropriate earlier phase.
- `[RETRY-PLAN plan_base]` — Full clean retry of a Failed plan. Kills any active agents for the plan, removes its git worktree and branch, clears all completed tasks and failure counts, and reschedules it from scratch (T1). The plan gets a fresh worktree from the current batch branch, so it picks up any code merged by other plans since the original failure. Use when: the worktree or branch is corrupted, or the plan needs to start over from a fundamentally different approach. Do NOT use for transient failures — use SOFT-RETRY instead to preserve completed work.
- `[SOFT-RETRY plan_base]` — Soft retry of a Failed plan. Resets failure counts and spawn backoff, sets phase back to Implementing, but PRESERVES all completed tasks and the existing worktree/branch. Only incomplete tasks are rescheduled. Use when: a plan failed due to transient issues (agent crashes, zero-output exits, spawn races, resource contention) where some tasks already completed successfully. This is the preferred retry for spawn-race failures.

Note: `[PRE-PLAN]`, `[VALIDATE]`, `[FIX-PLAN]`, and `[SKIP-VALIDATION]` are parallel-mode only and have no effect in sequential mode. Do not issue them.

If you include a directive, put it on its own line at the END of your response. Only include ONE directive.

## Guidelines

- Be concise. A few sentences of assessment, then the directive.
- Never ask questions or request confirmation. You are autonomous — decide and act.
- Don't intervene unless something is actually wrong.
- If an agent is making steady progress, say [OK].
- If compile keeps failing on the same error, nudge the implementer with the error and a different approach.
- If reviews keep returning REVISE on trivial issues after 3+ iterations, skip reviews.
- If a reviewer and writer have cycled 2+ times on the same issue without resolution, issue [RESET_REVIEW] with a corrective brief explaining what needs to change.
- If context pressure is high (>80%), nudge the agent to wrap up.
- If you see a pattern of ghost turns (agent doing nothing), restart it.
- If a plan has been open and running for 3-4+ minutes without meaningful progress (no new task completions, no new files written, agent output is repetitive or stuck in a loop, or the phase elapsed time keeps climbing with nothing to show for it), issue `[RESTART implementer]` to kill the agent and restart from scratch. A fresh context often unsticks things faster than nudging. Don't wait for the full phase timeout — 3-4 minutes of zero forward motion on an active plan is enough signal.
- Watch for plans in a weird intermediate state: partially completed tasks with no active agent, phases that transitioned but the next agent never started, or plans where the agent exited without finishing. These are stuck. Restart them.

## Resilience & State Preservation

The operator expects to leave this pipeline running unattended. Your job is to keep it moving and protect progress when things go wrong. Assume anything can happen: agents crash, state gets corrupted, processes freeze, edge cases surface, API rate limits hit, context windows fill up mid-task.

**Preserve progress above all else.** When you detect trouble:

- **Before restarting an agent**, nudge it to commit or stash whatever partial work it has. A restart with uncommitted changes means lost work. If the agent is unresponsive to the nudge (ghost turn), issue the restart anyway — but note in your assessment what may have been lost so the next agent can check.
- **When things freeze or stall**, don't just wait. Escalate quickly: nudge first, then restart if no response within one cycle. Frozen agents waste time and tokens.
- **When you see crashes or repeated failures**, look at the pattern. If the same error keeps recurring across restarts, the problem is structural — nudge the implementer with a different approach or skip the problematic section and move on. Don't burn 5 restarts on the same crash.
- **When state looks weird** (plans half-done with no agent, iteration counts that don't match phase, agents working on already-completed tasks), treat it as corruption and restart the affected plan from a clean state. Don't try to reason your way through broken state — reset it.
- **When edge cases surface** (unexpected file conflicts, merge failures, tests that pass locally but fail in gates, agents producing empty or nonsensical output), document what you see in your assessment and take the most conservative recovery action. Commit what's good, restart what's broken.
- **When API rate limits hit**, immediately `[THROTTLE]` down to reduce concurrency. Don't keep spawning agents into a rate limit wall.
- **When agents exit with zero output** (0 chars, 0-3 input tokens), this is a spawn race — the process died before it could read the prompt. Spawn races no longer count toward task failure limits, so plans won't fail from spawn races alone unless 10+ consecutive spawn failures occur. If a plan does fail from repeated spawn races, use `[SOFT-RETRY plan_base]` to reset it while preserving completed tasks. If it keeps happening, `[THROTTLE]` down first to reduce contention, then retry.
- **When context pressure is climbing fast**, don't wait for 80%. If an agent is at 60% and accelerating (lots of error output, long compile logs), nudge it to commit progress and wrap up the current task before it hits the wall.

The goal: the operator should be able to walk away and come back to either a completed run or a recoverable state — never a silent hang or lost work.

## User Steer Messages

When you receive a message like:
```
## User Inject

User message: <user_text>

Currently selected agent: <agent_label>

Should this message go to the selected agent, or should you issue a directive?
```

You are receiving a steer from the operator. Decide whether to:
- `[OK]` — Forward the user's message verbatim to the selected agent (the default)
- `[NUDGE role:instance_id] rephrased_message` — Send to a different agent than selected, optionally rephrased with context
- `[RESTART role:instance_id]` — Restart an agent if the steer suggests a fresh start
- `[FORCE-ADVANCE]` — Force commit and advance if the user wants to move on
- Any other directive if the steer implies a stronger action

If you respond with anything other than `[OK]`, that directive takes precedence and the user's message is not forwarded; instead, your directive is executed. If you respond with `[OK]`, the message is forwarded to the selected agent exactly as typed by the user.

{}
"#,
        build_file_manifest()
    )
}

/// Build a state snapshot message to send to the conductor agent.
pub fn state_snapshot(state: &RunState, event_description: &str) -> String {
    let plan_info = state
        .current_plan()
        .map(|p| format!("Plan: {} ({})", p.base, p.phase))
        .unwrap_or_else(|| "No active plan".to_string());

    let iteration = state.current_iteration;
    let phase = &state.current_phase;

    let phase_elapsed = state
        .phase_started
        .map(|t| {
            let d = t.elapsed();
            format!("{}m{}s", d.as_secs() / 60, d.as_secs() % 60)
        })
        .unwrap_or_else(|| "?".to_string());

    // Agent token summary
    let mut agent_summary = String::new();
    let mut roles: Vec<AgentRole> = state.agents.keys().copied().collect();
    roles.sort_by_key(|r| r.index());
    for role in &roles {
        if *role == AgentRole::Conductor {
            continue;
        }
        if let Some(agent) = state.agents.get(role) {
            let status = if agent.active { "ACTIVE" } else { "idle" };
            let in_k = agent.input_tokens / 1000;
            let out_k = agent.output_tokens / 1000;
            // Last few lines of output for context
            let last_lines: String = agent
                .output
                .lines()
                .rev()
                .take(20)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let truncated = if last_lines.chars().count() > 1000 {
                format!("{}...", last_lines.chars().take(1000).collect::<String>())
            } else {
                last_lines
            };
            agent_summary.push_str(&format!(
                "\n  {}: {} ({}k in, {}k out)\n    last: {}\n",
                role.label(),
                status,
                in_k,
                out_k,
                truncated.replace('\n', "\n    ")
            ));
        }
    }

    // Recent log entries
    let recent_logs: String = state
        .log_messages
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|l| {
            format!(
                "  [{}] {}: {}",
                l.timestamp.format("%H:%M:%S"),
                l.source,
                l.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Gate info (short tail for inline display)
    let gate_info = if !state.last_gate_output.is_empty() {
        let tail: String = state
            .last_gate_output
            .lines()
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        format!("Last gate output:\n  {}", tail.replace('\n', "\n  "))
    } else {
        String::new()
    };

    // Full gate output section (truncated to 500 chars) for conductor visibility
    let gate_output_section = if !state.last_gate_output.is_empty() {
        let truncated: String = if state.last_gate_output.chars().count() > 500 {
            format!(
                "{}...",
                state.last_gate_output.chars().take(500).collect::<String>()
            )
        } else {
            state.last_gate_output.clone()
        };
        format!("\n### Last Gate Output\n{truncated}\n")
    } else {
        String::new()
    };

    let verdicts = if !state.code_verdict.is_empty() {
        format!(
            "Verdicts: code={} docs={}",
            state.code_verdict,
            if state.doc_verdict.is_empty() {
                "N/A"
            } else {
                &state.doc_verdict
            }
        )
    } else {
        String::new()
    };

    let context_pct = state
        .agents
        .values()
        .filter(|a| a.active)
        .map(|a| {
            if state.context_limit > 0 {
                (a.input_tokens as f64 / state.context_limit as f64 * 100.0) as u32
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0);

    // Parallel-mode plan phase summary (from parallel_agents list)
    let mut plan_phases = String::new();
    {
        let mut seen = std::collections::HashSet::new();
        for pa in &state.parallel_agents {
            if pa.plan == "global" || !seen.insert(pa.plan.clone()) {
                continue;
            }
            let status = if pa.active { "ACTIVE" } else { "idle" };
            let out_preview: String = pa
                .output
                .chars()
                .rev()
                .take(80)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            plan_phases.push_str(&format!(
                "  {} [{}] {}: {}\n",
                pa.plan,
                pa.role.label(),
                status,
                out_preview
            ));
        }
    }

    // Executor diagnostics
    let executor_diag = if !state.executor_state_summary.is_empty() {
        format!(
            "\n### Executor Diagnostics\n{}\n",
            state.executor_state_summary
        )
    } else {
        String::new()
    };

    format!(
        r#"## Pipeline State Update: {event_description}

{plan_info}
Phase: {phase} ({phase_elapsed} elapsed)
Iteration: {iteration}
Consecutive REVISE: {revise_count}
Context pressure: {context_pct}%
{verdicts}

### Active Plans
{plan_phases}{executor_diag}
### Agent Status
{agent_summary}
### Recent Logs
{recent_logs}

{gate_info}
{gate_output_section}
What is your assessment? Include a directive if action is needed.
Reminder: If you see plans stuck in Failed state due to transient issues (zero-output exits, spawn races), use [SOFT-RETRY plan_base] to reset them while preserving completed work. Only use [RETRY-PLAN plan_base] if the worktree/branch is corrupted or the plan needs a fundamentally fresh start."#,
        revise_count = state.consecutive_revise_count,
    )
}

/// Parse a conductor response for a directive line.
/// Returns the directive and any remaining assessment text.
pub fn parse_directive(response: &str) -> Option<ConductorDirective> {
    for line in response.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("[OK]") {
            return Some(ConductorDirective::Ok);
        }
        if trimmed.starts_with("[NUDGE ") {
            if let Some(rest) = trimmed.strip_prefix("[NUDGE ") {
                if let Some(bracket_end) = rest.find(']') {
                    let role_str = &rest[..bracket_end];
                    let message = rest[bracket_end + 1..].trim().to_string();
                    if let Some(role) = parse_role(role_str) {
                        return Some(ConductorDirective::Nudge { role, message });
                    }
                }
            }
        }
        if trimmed.starts_with("[RESTART ") {
            if let Some(rest) = trimmed.strip_prefix("[RESTART ") {
                if let Some(bracket_end) = rest.find(']') {
                    let role_str = &rest[..bracket_end];
                    if let Some(role) = parse_role(role_str) {
                        return Some(ConductorDirective::Restart { role });
                    }
                }
            }
        }
        if trimmed.starts_with("[SKIP-REVIEWS]") {
            return Some(ConductorDirective::SkipReviews);
        }
        if trimmed.starts_with("[FORCE-ADVANCE]") {
            return Some(ConductorDirective::ForceAdvance);
        }
        if trimmed.starts_with("[THROTTLE ") {
            if let Some(rest) = trimmed.strip_prefix("[THROTTLE ") {
                if let Some(bracket_end) = rest.find(']') {
                    let num_str = &rest[..bracket_end];
                    if let Ok(limit) = num_str.trim().parse::<usize>() {
                        return Some(ConductorDirective::Throttle { limit });
                    }
                }
            }
        }
        if trimmed.starts_with("[PRE-PLAN ") {
            if let Some(rest) = trimmed.strip_prefix("[PRE-PLAN ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan_num = rest[..bracket_end].trim().to_string();
                    if !plan_num.is_empty() {
                        return Some(ConductorDirective::PrePlan { plan_num });
                    }
                }
            }
        }
        if trimmed.starts_with("[VALIDATE ") {
            if let Some(rest) = trimmed.strip_prefix("[VALIDATE ") {
                if let Some(bracket_end) = rest.find(']') {
                    let target = rest[..bracket_end].trim().to_string();
                    if !target.is_empty() {
                        return Some(ConductorDirective::Validate { target });
                    }
                }
            }
        }
        if trimmed.starts_with("[FIX-PLAN ") {
            if let Some(rest) = trimmed.strip_prefix("[FIX-PLAN ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan_num = rest[..bracket_end].trim().to_string();
                    let reason = rest[bracket_end + 1..].trim().to_string();
                    if !plan_num.is_empty() {
                        return Some(ConductorDirective::FixPlan { plan_num, reason });
                    }
                }
            }
        }
        if trimmed.starts_with("[SKIP-VALIDATION ") {
            if let Some(rest) = trimmed.strip_prefix("[SKIP-VALIDATION ") {
                if let Some(bracket_end) = rest.find(']') {
                    let target = rest[..bracket_end].trim().to_string();
                    if !target.is_empty() {
                        return Some(ConductorDirective::SkipValidation { target });
                    }
                }
            }
        }
        if trimmed.starts_with("[RESET_REVIEW]") {
            let reason = trimmed["[RESET_REVIEW]".len()..].trim().to_string();
            return Some(ConductorDirective::ResetReview(reason));
        }
        if trimmed.starts_with("[ENRICH ") {
            if let Some(rest) = trimmed.strip_prefix("[ENRICH ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan_num = rest[..bracket_end].trim().to_string();
                    if !plan_num.is_empty() {
                        return Some(ConductorDirective::Enrich { plan_num });
                    }
                }
            }
        }
        if trimmed.starts_with("[SPAWN-REVIEW ") {
            if let Some(rest) = trimmed.strip_prefix("[SPAWN-REVIEW ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan_base = rest[..bracket_end].trim().to_string();
                    if !plan_base.is_empty() {
                        return Some(ConductorDirective::SpawnReview { plan_base });
                    }
                }
            }
        }
        if trimmed.starts_with("[PHASE-REJECT ") {
            if let Some(rest) = trimmed.strip_prefix("[PHASE-REJECT ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan = rest[..bracket_end].trim().to_string();
                    let reason = rest[bracket_end + 1..].trim().to_string();
                    if !plan.is_empty() {
                        return Some(ConductorDirective::PhaseReject { plan, reason });
                    }
                }
            }
        }
        if trimmed.starts_with("[RETRY-PLAN ") {
            if let Some(rest) = trimmed.strip_prefix("[RETRY-PLAN ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan = rest[..bracket_end].trim().to_string();
                    if !plan.is_empty() {
                        return Some(ConductorDirective::RetryPlan { plan });
                    }
                }
            }
        }
        if trimmed.starts_with("[SOFT-RETRY ") {
            if let Some(rest) = trimmed.strip_prefix("[SOFT-RETRY ") {
                if let Some(bracket_end) = rest.find(']') {
                    let plan = rest[..bracket_end].trim().to_string();
                    if !plan.is_empty() {
                        return Some(ConductorDirective::SoftRetryPlan { plan });
                    }
                }
            }
        }
    }
    None
}

fn parse_role(s: &str) -> Option<AgentRole> {
    match s.trim().to_lowercase().as_str() {
        "strategist" | "strat" => Some(AgentRole::Strategist),
        "implementer" | "impl" => Some(AgentRole::Implementer),
        "architect" | "arch" => Some(AgentRole::Architect),
        "auditor" | "audit" => Some(AgentRole::Auditor),
        "scribe" => Some(AgentRole::Scribe),
        "critic" => Some(AgentRole::Critic),
        "refactorer" | "refac" => Some(AgentRole::Refactorer),
        "preplanner" | "prepl" => Some(AgentRole::PrePlanner),
        "docverifier" | "docvf" => Some(AgentRole::DocVerifier),
        "integration-tester" | "itest" => Some(AgentRole::IntegrationTester),
        "merge-resolver" | "merge" => Some(AgentRole::MergeResolver),
        "terminal-validator" | "tval" => Some(AgentRole::TerminalValidator),
        "golem-lifecycle-tester" | "glct" => Some(AgentRole::GolemLifecycleTester),
        "spec-drift-detector" | "sdrf" => Some(AgentRole::SpecDriftDetector),
        "regression-detector" | "regd" => Some(AgentRole::RegressionDetector),
        "performance-sentinel" | "perf" => Some(AgentRole::PerformanceSentinel),
        "coverage-tracker" | "covr" => Some(AgentRole::CoverageTracker),
        "plan-lifecycle-mgr" | "plcm" => Some(AgentRole::PlanLifecycleManager),
        "cross-system-tester" | "xsys" => Some(AgentRole::CrossSystemTester),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum ConductorDirective {
    Ok,
    Nudge {
        role: AgentRole,
        message: String,
    },
    Restart {
        role: AgentRole,
    },
    SkipReviews,
    ForceAdvance,
    Throttle {
        limit: usize,
    },
    PrePlan {
        plan_num: String,
    },
    Validate {
        target: String,
    },
    FixPlan {
        plan_num: String,
        reason: String,
    },
    SkipValidation {
        target: String,
    },
    ResetReview(String),
    /// Run `bardo-enrich.sh <plan>` (shell pipeline for task / plan enrichment).
    Enrich {
        plan_num: String,
    },
    /// Spawn a review agent for a specific plan (recovery for stuck plans).
    SpawnReview {
        plan_base: String,
    },
    /// Reject a phase transition for a specific plan — rolls back to re-run.
    PhaseReject {
        plan: String,
        reason: String,
    },
    /// Reset a Failed plan so it retries from scratch — clears failure counts and phase.
    RetryPlan {
        plan: String,
    },
    /// Soft-retry a Failed plan — resets failure counts but preserves completed tasks and worktree.
    SoftRetryPlan {
        plan: String,
    },
}

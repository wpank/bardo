# Mori overview

## The problem

AI coding tools give you one agent with one context window. That agent can read some files, write some code, and hope for the best. For small tasks, this works. For anything larger, it breaks down in predictable ways.

**Context limits.** A 200k-token context window sounds generous until you have a 50-crate Rust workspace. The agent can't see the whole codebase. It makes decisions that conflict with code three directories away, because it never read that code.

**No verification.** The agent writes code and hands it back to you. It doesn't compile the result. It doesn't run the tests. You paste the error back, it tries again, you paste again. This manual error loop is the dominant workflow in every AI editor shipping today.

**No coordination.** If you want to parallelize, you open two chat windows. Both agents edit the same file. You get merge conflicts. There's no scheduler that knows which files each agent needs, no way to partition work so parallel agents stay out of each other's way.

**No recovery.** The agent crashes, the browser tab closes, the SSH connection drops. Your progress is gone. The agent was holding everything in conversation memory, which is volatile. You start over.

**No cost awareness.** Every request goes to the most expensive model at full price. A config file change costs the same as implementing a complex algorithm. There's no routing, no caching, no batching.

Building real software requires multiple agents working in parallel with isolation between them, verification of their output, and awareness of what things cost. No existing tool does this.


## What Mori is

Mori is a Rust binary that turns product requirements into deployed software through multi-agent orchestration.

It is not an IDE plugin. Not a chatbot. Not a copilot. It is infrastructure. You give it a PRD and it produces code, tests, documentation, and deployments without human intervention.

The central idea is a document hierarchy that compresses context at each layer:

```
PRD (what you want)
  -> Plans (how to build it, in what order)
    -> Tasks (atomic work units with file assignments and acceptance criteria)
      -> Briefs (pre-assembled context, budget-fitted per role)
        -> Prompts (what the agent actually sees)
```

An implementer agent doesn't read your 15-page PRD. It reads three paragraphs extracted from the PRD that are relevant to its task, the type signatures it needs to import, the specific files it should modify, and the test criteria that prove it worked. The enrichment pipeline extracts exactly this, before the agent ever starts.

This is what makes parallelism possible. When each agent sees only its own slice of the problem, agents don't step on each other. When context is pre-computed rather than searched at runtime, agents don't waste tokens on exploration.


## Five-layer architecture

Mori is organized into five layers. Each handles a different concern.

### Execution

Every agent runs in its own git worktree -- a physical copy of the repo on its own branch. No shared mutable state between agents. When an agent finishes, quality gates run: `cargo check`, `cargo test`, linting. If gates pass, reviewer agents evaluate the design. If gates fail, the agent gets structured error output and tries again. A merge queue serializes completed work back to the batch branch in dependency order.

### Context

A 32-script enrichment pipeline generates all context artifacts before agents start. Most of these scripts are deterministic: tree-sitter AST extraction, dependency graph analysis, workspace mapping, PRD-to-plan section extraction. Not LLM calls. The output is a set of per-plan artifacts (briefs, task definitions, PRD extracts, type registries) and global artifacts (workspace map, preflight snapshot, cross-plan context). At injection time, `ContextInjector` copies exactly the right files into each agent's `context/in/` directory based on its role. Implementers get task lists and briefs. Reviewers get completion summaries and implementation notes. Each agent sees what it needs and nothing else.

### Inference

bardo-gateway is an LLM proxy that sits between agents and model providers. It provides three-layer caching (hash-exact, semantic similarity, prompt prefix), multi-provider routing across Anthropic, OpenAI, and others, and per-request cost tracking. Task routing picks the right model for each job: opus for complex implementation, haiku for config file changes, sonnet for reviews. Four presets (quality, balanced, cost, speed) let you tune the tradeoff. The gateway cuts inference costs 40-85% on typical builds.

### Agent

Mori defines 28 roles in its `AgentRole` enum. The primary pipeline uses a subset for any given build: strategist (decomposes plans into tasks), implementer (writes code), architect (reviews design), auditor (reviews correctness), scribe (updates documentation). Specialized roles activate on demand: error-diagnoser (targeted fixes instead of full re-implementation), pattern-extractor (learns idioms from existing code), merge-resolver (handles conflicts), auto-fixer (lightweight post-gate repairs). Agents are backend-agnostic -- the same role can run on Claude Code, OpenAI Codex, or Cursor, selected per-task based on the model routing configuration.

### Orchestration

The executor builds a unified DAG of tasks across all plans. Tasks declare their file assignments. The DAG detects file conflicts and prevents two agents from editing the same file simultaneously. Within a plan, tasks with no file overlap run in parallel. Across plans, dependency ordering determines which plans can proceed. The conductor watches for failures and intervenes: retrying with targeted fixes, re-gating downstream plans when an API changes, force-advancing stuck plans. All executor state is serialized to an `ExecutorSnapshot` on every transition, so a crash mid-build resumes from exactly where it stopped.


## How a build works end-to-end

You run `mori build` and point it at a PRD.

**Enrichment.** The 32-script pipeline runs first. It parses the codebase with tree-sitter to extract type signatures, function boundaries, and import graphs. It maps the workspace structure. It slices the PRD into per-plan extracts. It generates task TOML files with file assignments, dependency declarations, and acceptance criteria. It produces context briefs sized to each role's token budget. Most of this runs in under a minute with no LLM calls.

**DAG construction.** The executor reads all plan task files and builds a unified cross-plan DAG. Each task node carries a `GlobalTaskId` (plan + task number), a list of files it will touch, and its dependencies on other tasks. The scheduler identifies which tasks can run in the first wave -- those with no unmet dependencies and no file conflicts with other ready tasks.

**Dispatch.** For each ready task, the executor emits a `SpawnTaskAgent` action. The event loop creates a worktree, calls `ContextInjector::inject_for_implementer` to copy the right artifacts into `context/in/`, and launches an agent process. Multiple agents run simultaneously, each in its own worktree, each seeing only its task's context.

**Gates.** When all tasks in a plan complete, the executor emits `RunPlanGates`. The event loop runs `cargo check` and `cargo test` in the plan's worktree. If gates fail, the agent gets the compiler errors and test failures as structured input, a reflection analysis identifies what went wrong, and the plan re-enters the implementation phase. If gates pass, the plan advances to review.

**Review.** The executor emits `RunPlanReviews`, which spawns architect and auditor agents in parallel. Reviewers read the implementation diff, the completion summary (compile status, test counts, implementer notes), and the original plan. They produce structured `ReviewReport` JSON with pass/fail verdicts and specific feedback. If reviews fail, the feedback feeds back into the next implementation iteration.

**Merge.** Once reviews pass, `MergePlanToBatch` fires. The merge queue respects dependency ordering -- if plan 3 depends on plan 2, plan 2 merges first even if plan 3 finished earlier. After merge, post-merge regression tests run on the batch branch to catch integration issues.

**Output.** The batch branch accumulates all merged plans. When the build completes, the result is a git branch with all changes, ready for PR creation or deployment.


## What makes it different

Each of these is covered in depth in its own document. The short version:

**Task-level DAG scheduling.** Most multi-agent tools (the few that exist) schedule at the plan level: run plan 1, then plan 2. Mori schedules at the task level, across plans. If plan 3 task 2 has no file conflicts with plan 5 task 1, they run simultaneously. This collapses wall-clock time without introducing merge conflicts.

**Offline enrichment.** Context is pre-computed by deterministic scripts, not searched by agents at runtime. Agents don't spend tokens grepping the codebase. They open `context/in/brief.md` and start working. This makes context reproducible: same codebase state produces the same artifacts every time.

**Per-role context budgeting.** An implementer gets different context than a reviewer. A reviewer gets the completion summary and diff that an implementer never sees. Each role's context is sized to fit its model's token budget with room for the actual work.

**Automated iteration with learning.** When gates fail, the reflection loop analyzes what went wrong and injects that analysis into the next attempt. The playbook system accumulates lessons across builds -- if a pattern of failure repeats, future agents see warnings about it before they start.

**Crash recovery.** `ExecutorSnapshot` is written to disk on every state transition. It records completed tasks, in-flight tasks, plan phases, iteration counts, and merge queue ordering. A restart reads the snapshot and resumes. No work is lost.

**Cost-aware scheduling.** Model tier routing sends cheap tasks to cheap models. The batch API queues non-urgent work at 50% cost. Three-layer caching avoids redundant inference. Per-agent and per-plan cost tracking is visible in the TUI in real time.


## Reading guide

This document covers what Mori is and how it works at a high level. The remaining documents in this series go deep on each subsystem:

- **mori-context-engineering.md** -- The 32-script enrichment pipeline, token budgeting, and how context artifacts are generated and injected.
- **mori-unified-dag.md** -- Cross-plan task DAG construction, file conflict detection, wave scheduling, and dependency resolution.
- **mori-quality-gates.md** -- Compile and test gates, the reflection loop, structured error feedback, and iteration mechanics.
- **mori-parallel-execution.md** -- Git worktree management, agent isolation, concurrent dispatch, and merge queue ordering.
- **mori-cost-efficiency.md** -- bardo-gateway caching, model tier routing, batch API usage, and per-build cost tracking.
- **mori-resilience.md** -- Crash recovery, `ExecutorSnapshot` persistence, conductor interventions, and failure handling.
- **mori-agent-architecture.md** -- The 28 roles, backend abstraction (Claude/Codex/Cursor), per-task model selection, and skills injection.

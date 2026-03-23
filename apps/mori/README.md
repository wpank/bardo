# Mori

Takes a specification, decomposes it into dependency-ordered plans, engineers targeted context for each one, and dispatches a fleet of AI agents in parallel across isolated git worktrees to implement, test, review, and merge the results.

53,000 lines of Rust. 26 agent roles. DAG-scheduled parallel execution. A Ratatui TUI that shows you everything.

## The problem it solves

The bottleneck in AI-assisted development is not model quality. It is context.

Current tools put you in a chat window and hope the model figures out what to do. The model sees whatever fits in its context window, which is usually not enough. You re-explain the same things, watch the agent make decisions that conflict with work done two conversations ago, and manually stitch together outputs that don't fit.

An agent implementing your authentication layer needs to understand the three OAuth providers you support, their token exchange flows, and the session types from two crates upstream. That context lives across your PRD, six spec documents, and a handful of source files. You can't dump all of it into a prompt. You have to engineer what goes in, when, and how much.

Mori's answer: a document hierarchy where each layer compresses and targets context for the layer below it.

```
PRD (what you want)
  -> Plans (how to build it, in what order)
    -> Tasks (atomic units of work, with files and acceptance criteria)
      -> Briefs (everything an agent needs, budget-fitted to its context window)
        -> Prompt (what the agent actually sees)
```

Each layer reduces the next agent's context burden. An implementer doesn't need your entire PRD. It needs the 3 paragraphs relevant to its task, the types it should import, the files it should touch, and the tests that prove it worked. The enrichment pipeline extracts exactly that.

## Quick start

```bash
# Primary entry point
./mori.sh 01-05

# With flags
./mori.sh 08a-08d --no-review --skip-tests

# Express mode: single-pass, no reviews, up to 20 concurrent agents
./mori.sh 01-09 --express

# Dry run: print execution plan, DAG stats, wave breakdown, no agents spawned
./mori.sh 01-09 --dry-run

# Validate plan files and print parallelism stats
./mori.sh 01-09 --validate
```

`mori.sh` handles `.env` sourcing, gateway auto-detection, and default flag injection (`--parallel --pre-plan`). Use it instead of `cargo run -p mori` directly unless you know what you're doing.

## How a build works

### 1. Write plans

Plans are numbered markdown files in `plans/`. Each one has YAML frontmatter declaring its dependencies and the files it touches:

```yaml
---
plan: "03-oauth-providers"
depends_on: ["01-user-model", "02-session-store"]
files_touched: ["src/auth/", "src/middleware/"]
parallel_safe: true
---
```

The plan body contains what to build: prerequisite tables, import blocks, directory trees, step-by-step implementation instructions, and acceptance criteria. Plan specs on the CLI (`"01"`, `"01-09"`, `"08a-08d"`) select which plans to run. If you omit them, mori reads `.mori/queue.toml`.

### 2. DAG construction and wave scheduling

Plans declare `depends_on` in frontmatter. Mori builds a directed acyclic graph and groups plans into execution waves:

```
Wave 1: [plan-01, plan-02]          # no dependencies, run in parallel
Wave 2: [plan-03]                    # depends on 01 and 02
Wave 3: [plan-04, plan-05, plan-06]  # depend only on 03
```

Within each wave, plans run concurrently. Waves execute sequentially.

But plan-level parallelism leaves performance on the table. Two plans in the same wave might touch completely different files. Mori's unified task DAG goes further: it looks at all tasks across all plans in a wave, builds a file-conflict graph, and partitions them into independent groups using union-find. Tasks in different groups run simultaneously even if they belong to different plans.

```
Wave 2 plans: [plan-03, plan-04]
  plan-03 tasks: T1 (src/auth/), T2 (src/auth/), T3 (src/middleware/)
  plan-04 tasks: T1 (src/api/), T2 (src/api/), T3 (src/models/)

  Group A: plan-03/T1, plan-03/T2    (file conflict on src/auth/)
  Group B: plan-03/T3                (no conflict with anything)
  Group C: plan-04/T1, plan-04/T2    (file conflict on src/api/)
  Group D: plan-04/T3                (no conflict)

  -> Groups A, B, C, D all dispatch concurrently
  -> Within each group, tasks respect depends_on ordering
```

### 3. Context enrichment

Before any agent runs, the enrichment pipeline generates context artifacts for each plan:

**Briefs** -- strategist-authored summaries of what the plan needs, fitted to the agent's token budget. The budget is computed dynamically: 60% reserved for input, 40% for output, then allocated across plan text, workspace map, PRD extracts, file context, and review history.

**Task TOMLs** -- atomic units of work with file assignments, acceptance criteria, and parallel grouping:

```toml
[[task]]
id = "T1"
title = "OAuthProvider trait definition"
status = "pending"
files = ["src/auth/provider.rs", "src/auth/mod.rs"]
acceptance = ["Trait compiles", "Has authorize_url, exchange_code, fetch_profile methods"]
depends_on = ["02:T3"]
parallel_group = "A"
```

Cross-plan dependencies (`"02:T3"`) let the scheduler extract parallelism across plan boundaries rather than treating each plan as an atomic blob. The `exclusive_files` flag (default true) means no other task can write to these files while this task runs.

**Workspace maps** -- auto-generated crate-level file trees so agents know what exists, filtered per-agent to show only the crates they need.

**PRD extracts** -- the specific paragraphs from the PRD relevant to this plan, not the whole document.

All context artifacts live as files on disk under `plans/context/`. They are diffable, editable, and versionable. Nothing lives only in memory.

### 4. Agent dispatch

Each agent gets its own git worktree -- a physical copy of the repo on its own branch. No shared mutable state between agents. All worktrees share a single `sccache` instance with normalized base directories, so the second agent to compile a shared dependency gets a near-instant cache hit instead of recompiling from scratch.

Mori spawns the agent with an assembled prompt containing the plan, brief, task TOML, workspace map, and relevant file context. The agent works through its tasks, writing code and running intermediate compile checks.

### 5. Gate pipeline

After the implementer finishes, the plan passes through gates:

```
Preflight -> Implement -> Compile Gate -> Test Gate -> Review -> Verdict -> Merge
```

**Compile gate.** `cargo check`. Binary pass/fail. On failure, the implementer gets another iteration with a structured error digest (unique errors with file:line references, not pages of raw compiler output).

**Test gate.** `cargo test`. On failure, the implementer sees which tests failed and gets another iteration.

**Review.** Up to three reviewers run in parallel:
- Architect: API design, interface consistency, cross-plan coherence
- Auditor: test coverage, safety, correctness
- Scribe: documentation completeness

Each produces a structured verdict (approve / request-changes / block).

**Verdict.** All approve -> merge. Any request changes -> implementer gets another iteration with review feedback. Any block -> escalate to the conductor.

A plan can iterate up to `max_iterations` times (default 8). Each re-review checks only changed files.

### 6. Iteration memory

When an agent fails, the failure feeds forward. Each iteration builds cumulative DO NOT RETRY lists from gate errors and review blockers. If iteration 1 failed with a type mismatch and iteration 2 failed with a missing trait bound, iteration 3 sees both entries. The agent can't repeat either mistake without addressing the underlying cause.

This came from watching an agent hit the same type mismatch four iterations in a row, each time "fixing" it with a slightly different wrong approach. The DO NOT RETRY list forces the agent to try something genuinely new.

On the success side, plans that pass on the first try get recorded as golden-path examples. Future decompositions pull up to 2 golden-path examples of the same category, so a new data-structural plan gets shown how a previous one succeeded.

### 7. Merge

The branch merges into the batch branch. If there are conflicts, the MergeResolver agent handles them. Post-merge, the conductor checks batch integrity before advancing to the next wave.

## The conductor

The conductor is a monitoring layer that watches running agents and intervenes when things go wrong.

**Watchers:**
- Silence timeout (5 min): agent hasn't produced output. Probably stuck. Conductor sends a nudge or restarts.
- Iteration loop detector: agent is making the same fix repeatedly. Conductor injects diagnostic context or switches strategy.
- Error pattern matcher: recognizes common failure patterns (lifetime issues, import cycles, missing types) and injects targeted hints before the agent wastes tokens rediscovering them.

**Interventions:**
- Inject context: send additional information to a running agent.
- Restart with different model: if the current model is struggling, swap to a more capable one.
- Skip and advance: if a plan is stuck after max iterations, skip it and continue.
- Force merge: override a review block when the conductor determines the block is incorrect.

Implementers get the highest spawn priority. The conductor gives itself the lowest, because it should never starve an implementer of a slot.

## Agent backends

Mori is provider-agnostic. Backend is inferred from the model slug:

| Slug pattern | Backend |
|---|---|
| `claude-*` | Claude Code CLI |
| `composer-*`, `cursor-*`, `auto`, `sonnet-*`, `opus-*`, `haiku-*`, `gemini-*` | Cursor |
| Everything else (`gpt-*`, `o3`, `o4-mini`, etc.) | Codex CLI |

You can mix backends in one run. An opus-class implementer via Claude, a haiku-class scribe via Cursor, a gpt-class config fixer via Codex -- all in the same wave.

## Write for amnesia

Every agent session starts cold. No conversation memory, no shared state, no hidden context. The files on disk are the only truth.

This is a feature:

- **Debuggable.** If an agent produces bad output, read its input artifacts. Everything it saw is in `plans/context/`.
- **Reproducible.** Same inputs -> same artifacts -> same agent behavior.
- **Scalable.** Add more agents without worrying about shared state corruption. Each one reads files and writes files.
- **Resumable.** Crash in the middle of plan 7? Restart from plan 7. All prior work is committed. All context artifacts are on disk.

## CLI reference

```
mori [plans...] [flags]
```

Plan specs: `"01"`, `"01-09"`, `"08a-08d"`. If omitted, reads from `.mori/queue.toml`.

### Execution control

| Flag | Default | Description |
|------|---------|-------------|
| `--validate` | off | Parse plans, build DAG, print stats, exit |
| `--dry-run` | off | Build DAG, print wave plan, exit without spawning agents |
| `--no-review` | off | Skip strategist + review loop (single implementer pass) |
| `--skip-tests` | off | Skip the `cargo test` gate |
| `--no-docs` | off | Skip Scribe and Critic phases |
| `--max-iterations N` | 8 | Max review cycles before halting a plan |
| `--parallel` | off | Run plans within the same wave concurrently |
| `--express` | off | Single-pass, no reviews; defaults `--max-agents 20 --max-parallel-plans 6` |
| `--pre-plan` | off | Speculatively prepare briefs for upcoming waves |
| `--refactor` | off | Enable post-plan refactoring passes |
| `--fast` | off | Codex fast mode (1.5x speed, 2x credits) |

### Agent limits

| Flag | Default | Description |
|------|---------|-------------|
| `--max-agents N` | 8 | Maximum parallel agents |
| `--max-parallel-plans N` | 3 | Maximum plans executing in parallel (6 in express mode) |
| `--model NAME` | from config | Override the LLM model |
| `--fallback-model NAME` | none | Retry once with this model on spawn failure |

### Batch and queue

| Flag | Default | Description |
|------|---------|-------------|
| `--batch-size N` | none | Pause after every N plans for manual review |
| `--batch-id ID` | today's date | Override batch branch suffix |
| `--queue` | off | Force reading `.mori/queue.toml` even when plan specs are on CLI |
| `--milestone NAME` | none | Run only plans from a named milestone in `queue.toml` |
| `--preset NAME` | none | Execution preset: `quality`, `balanced`, `cost`, `speed` |

### Other

| Flag | Default | Description |
|------|---------|-------------|
| `--headless` | off | Disable TUI, log to stdout |
| `--repo-root PATH` | CWD | Override repository root |
| `--cleanup` | off | Delete merged `codex/plan/*` branches, prune worktrees |
| `--gateway` | true | Enable embedded bardo-gateway |
| `--no-gateway` | off | Bypass embedded gateway |
| `--gateway-port N` | 4000 | Port for the embedded gateway |

## Queue file

Instead of long CLI invocations, write `.mori/queue.toml`:

```toml
[run]
mode = "express"
max_agents = 12
max_parallel_plans = 4
preset = "balanced"

[[milestone]]
name = "Core infrastructure"
plans = ["01", "02", "03"]

[[milestone]]
name = "API layer"
plans = ["04", "05a", "05b"]
```

Run with `./mori.sh` (no plan args) and it reads the queue. `--milestone "Core infrastructure"` runs only that group.

## Execution presets

Presets tune the quality/speed/cost tradeoff:

| Preset | Review | Tests | Max agents | Max parallel plans | Use case |
|--------|--------|-------|-----------|-------------------|----------|
| `quality` | Full (arch+audit+scribe+critic) | Always | 8 | 3 | Production code, PRs |
| `balanced` | Architect + Auditor | Always | 12 | 4 | Default development |
| `cost` | QuickReviewer only | Always | 6 | 2 | Budget-conscious |
| `speed` | None | Skip | 20 | 6 | Scaffolding, prototyping |

## The TUI

Mori's TUI is a Ratatui application with 10 views, 26 widgets, and 12 modal dialogs. It uses the ROSEDUST palette (rose on violet-black, CRT scanlines, phosphor effects).

The dashboard shows wave progress, active agents with their current task and token consumption, gate results, review verdicts, budget tracking (per-agent, per-plan, per-milestone), and system metrics. Token sparklines render in braille characters. Phase timelines show how long each stage took. The agent pool displays live output streams from all running agents.

Why a TUI? Because steering an agent swarm is an interactive problem. You need to see what's happening across all plans, spot the stuck ones, and intervene before they burn tokens going in circles. Running headless was tried first -- check back after 20 minutes and discover an agent had been stuck in a compile-fix loop for 15 of them.

| Key | Action |
|-----|--------|
| `s` | Start pipeline |
| `q` | Quit (with confirmation) |
| `i` | Inject message to running agent |
| `r` | Resume paused plan |
| `?` | Help |
| `Tab` | Next view |
| `1-9` | Jump to view |
| `Up/Down` | Select plan/task |
| `Enter` | View detail |

## Embedded gateway

Mori can embed the bardo-gateway inference proxy (enabled by default). The gateway sits between agents and model providers:

- Multi-provider routing (Anthropic, OpenAI, OpenRouter)
- Three-layer caching (hash, semantic, prompt prefix)
- Rate limit management and failover across API keys
- Cost tracking per agent, per plan, per milestone

When running embedded, the gateway starts on `--gateway-port` (default 4000) and agents route through it automatically. For shared setups, run the gateway standalone with `mori-gateway.sh` and point multiple mori instances at it.

## Crash recovery

Mori writes crash reports to `.mori/runs/` on both panics and non-panic errors. Reports include the error with backtrace, app state at time of crash (which plans were running, which agents were active, what phase each was in), and recent log lines.

The supervisor script (`mori-supervisor.sh`) watches the process and restarts on crash. Since all state is on disk (committed code, context artifacts, plan progress), the restart picks up where it left off.

## Shell scripts

| Script | Purpose |
|--------|---------|
| `mori.sh` | Primary entry point; handles `.env`, gateway detection, default flags |
| `mori-supervisor.sh` | Watch supervisor -- restarts mori on crash |
| `bardo-enrich.sh` | Enrich plan artifacts (briefs, task breakdowns) before a run |
| `mori-gateway.sh` | Start bardo-gateway standalone |

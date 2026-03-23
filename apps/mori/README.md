# Mori

A context engineering orchestration service. Takes product requirements, decomposes them into dependency-ordered plans, enriches each plan with targeted context, and dispatches a fleet of specialized AI agents in parallel across isolated git worktrees to implement, test, review, and merge the results.

53,000 lines of Rust. 26 agent roles. DAG-scheduled parallel execution. A Ratatui TUI that shows you everything.

## Why this exists

The bottleneck in AI-assisted development is not model quality. It is context.

Current tools put you in a chat window and hope the model figures out what to do. The model sees whatever fits in its context window, which is usually not enough. You re-explain the same things, watch the agent make decisions that conflict with work done two conversations ago, and manually stitch together outputs that don't fit.

Mori's answer: build a document hierarchy where each layer compresses and targets context for the layer below it.

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
# From repo root -- this is the primary entry point
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

### 1. Plan discovery

Mori reads numbered markdown files from `plans/`. Each plan has YAML frontmatter declaring dependencies and the files it touches:

```yaml
---
plan: "03-oauth-providers"
depends_on: ["01-user-model", "02-session-store"]
files_touched: ["src/auth/", "src/middleware/"]
parallel_safe: true
---
```

Plan specs on the CLI (`"01"`, `"01-09"`, `"08a-08d"`) select which plans to run. If you omit them, mori reads `.mori/queue.toml`.

### 2. DAG construction and wave scheduling

Plans declare `depends_on` in frontmatter. Mori builds a directed acyclic graph and groups plans into execution waves:

```
Wave 1: [plan-01, plan-02]          # no dependencies, run in parallel
Wave 2: [plan-03]                    # depends on 01 and 02
Wave 3: [plan-04, plan-05, plan-06]  # depend only on 03
```

Within each wave, plans run concurrently. Waves execute sequentially.

But plan-level parallelism leaves performance on the table. Two plans in the same wave might touch completely different files. Mori's unified task DAG goes further: it looks at all tasks across all plans in a wave, builds a file-conflict graph (which tasks touch overlapping files), and partitions them into independent groups using union-find. Tasks in different groups run simultaneously even if they belong to different plans.

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

**Briefs** -- strategist-authored summaries of what the plan needs, fitted to the agent's token budget. The budget is computed dynamically based on the model's context window: 60% reserved for input, 40% for output, then allocated across plan text, workspace map, PRD extracts, file context, and review history.

**Task TOMLs** -- atomic units of work with file assignments, acceptance criteria, and parallel grouping:

```toml
[[task]]
id = "T1"
title = "OAuthProvider trait definition"
status = "pending"
files = ["src/auth/provider.rs", "src/auth/mod.rs"]
acceptance = ["Trait compiles", "Has authorize_url, exchange_code, fetch_profile methods"]
```

**Workspace maps** -- auto-generated crate-level file trees so agents know what exists. Filtered per-agent to show only the crates they need.

**PRD extracts** -- the specific paragraphs from the PRD relevant to this plan, not the whole document.

All context artifacts live as files on disk under `plans/context/`. They are diffable, editable, and versionable. Nothing lives only in memory.

### 4. Agent dispatch

Each agent gets its own git worktree -- a physical copy of the repo on its own branch. No shared mutable state between agents.

Mori spawns the agent (Claude Code, Codex, or Cursor -- backend inferred from the model slug) with an assembled prompt containing the plan, brief, task TOML, workspace map, and relevant file context. The agent works through its tasks, writing code and running intermediate compile checks.

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

### 6. Merge

The branch merges into the batch branch. If there are conflicts, the MergeResolver agent handles them. Post-merge, the conductor checks batch integrity before advancing to the next wave.

## Agent roles

Mori defines 26 specialized agent roles. Each carries a default model assignment, but actual model selection happens at the task level based on routing tags (complexity, category, quality, speed).

**Core (always present):**

| Role | What it does |
|------|-------------|
| Implementer | Writes code, runs compile checks, marks tasks complete |
| Conductor | Monitors running agents, intervenes on failures (silence timeouts, iteration loops, error patterns) |

**Review pipeline (configurable, on by default):**

| Role | What it does |
|------|-------------|
| Architect | API design, interface consistency, cross-plan coherence |
| Auditor | Safety, error handling, edge cases, test coverage |
| Scribe | Documentation, README updates, inline comments |
| Critic | Reviews the scribe's output for accuracy |
| QuickReviewer | Single-pass review combining arch+audit concerns (used for simpler plans) |

**Planning and strategy:**

| Role | What it does |
|------|-------------|
| Strategist | Pre-plans upcoming waves, identifies risks, writes briefs |
| PrePlanner | Speculatively prepares context for future waves while current wave executes |
| PlanLifecycleManager | Tracks plan state across iterations |

**Automated responders (triggered by events):**

| Role | What it does |
|------|-------------|
| AutoFixer | Triggered by gate failures, applies targeted fixes |
| MergeResolver | Triggered by merge conflicts |
| ErrorDiagnoser | Triggered by cryptic test failures, produces root-cause analysis |
| Refactorer | Post-merge cleanup passes |

**Validation and testing:**

| Role | What it does |
|------|-------------|
| TerminalValidator | Validates TUI rendering and interaction |
| GolemLifecycleTester | Tests golem lifecycle state transitions |
| IntegrationTester | Cross-crate integration testing |
| CrossSystemTester | End-to-end pipeline validation |
| FullLoopValidator | Validates mirage + terminal + runtime in full-loop test phase |
| DependencyValidator | Checks cross-plan type dependencies |
| RegressionDetector | Detects regressions introduced by new plans |
| PerformanceSentinel | Monitors for performance degradation |
| CoverageTracker | Tracks and enforces test coverage |
| SpecDriftDetector | Detects divergence between PRD and implementation |

**Knowledge and analysis:**

| Role | What it does |
|------|-------------|
| Researcher | Deep research tasks (codebase exploration, API investigation) |
| PatternExtractor | Extracts reusable patterns from successful builds |
| SnapshotComparator | Compares before/after snapshots for regression detection |

## Agent backends

Mori is provider-agnostic. Backend is inferred from the model slug:

| Slug pattern | Backend |
|---|---|
| `claude-*` | Claude Code CLI |
| `composer-*`, `cursor-*`, `auto`, `sonnet-*`, `opus-*`, `haiku-*`, `gemini-*` | Cursor |
| Everything else (`gpt-*`, `o3`, `o4-mini`, etc.) | Codex CLI |

You can mix backends in one run. An opus-class implementer via Claude, a haiku-class scribe via Cursor, a gpt-class config fixer via Codex -- all in the same wave.

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

## Write for amnesia

Every agent session starts cold. No conversation memory, no shared state, no hidden context. The files on disk are the only truth.

This is a feature:

- **Debuggable.** If an agent produces bad output, read its input artifacts. Everything it saw is in `plans/context/`.
- **Reproducible.** Same inputs -> same artifacts -> same agent behavior.
- **Scalable.** Add more agents without worrying about shared state corruption. Each one reads files and writes files.
- **Resumable.** Crash in the middle of plan 7? Restart from plan 7. All prior work is committed. All context artifacts are on disk.

## Directory layout

### On disk

```
plans/                              # Plan markdown files
  01-workspace-scaffold.md
  02-core-types.md
  ...
plans/context/
  briefs/                           # Strategist briefs per plan
  reviews/                          # Review verdicts
  tasks/                            # Task checklists (TOML)
  docs/                             # Scribe documentation
  workspace-map.md                  # Auto-generated crate file tree
.mori/
  config.toml                       # Mori configuration
  queue.toml                        # Plan queue and milestones
  costs.db                          # SQLite cost tracking
  index.db                          # Codebase index
  memory/                           # Episode, pattern, playbook tiers
  plans/                            # Plan execution state
  runs/                             # Per-run logs and crash reports
tmp/plan-runs/
  bardo-ctl.log                     # JSON-structured runtime log
```

### Source modules

```
src/
  main.rs                           # CLI parsing, crash handler, runtime bootstrap
  orchestrator/
    dag.rs                          # PlanDag -- plan-level dependency graph, wave computation
    unified_dag.rs                  # UnifiedTaskDag -- task-level graph across all plans
    executor.rs                     # Plan execution engine
    pipeline.rs                     # Pipeline orchestration state machine
    gates.rs                        # Post-implementation gates (compile, test)
    review.rs                       # Review verdict handling
    plan.rs                         # Plan discovery and frontmatter parsing
    tasks.rs                        # Task checklist loading (TOML)
    queue.rs                        # .mori/queue.toml parsing
    prompts.rs                      # Agent prompt construction with dynamic budgeting
    context.rs                      # Context injection (workspace maps, filtered maps)
    coordination.rs                 # Inter-agent coordination
    preflight.rs                    # Pre-run checks
    phase.rs                        # Execution phase state machine
    paths.rs                        # Canonical paths (.mori/, plans/, tmp/)
    batch.rs                        # Batch execution
    registry.rs                     # Plan/task registry
    iteration_memory.rs             # Per-plan iteration state
    event_log.rs                    # Structured event log
    artifacts.rs                    # Immutable artifact store
    autofix.rs                      # Automated fix dispatch
    complexity.rs                   # Task complexity analysis
    inject.rs                       # Context injection
    memory.rs                       # Episode/pattern/playbook memory
    reflection.rs                   # Post-build reflection
    schema.rs                       # Review and completion report schemas
    skills.rs                       # Skill injection
  agent/
    mod.rs                          # Agent spawning, lifecycle management
    connection.rs                   # Agent communication (stdin/stdout/WebSocket)
    events.rs                       # Agent event stream parsing
    protocol.rs                     # Agent wire protocol
    roles.rs                        # 26 agent roles, model specs, backend inference
  conductor/
    actions.rs                      # LLM-powered conductor interventions
    llm.rs                          # LLM client (routes through bardo-gateway)
    watchers.rs                     # Background task watchers (silence, iteration, error)
  git/
    graph.rs                        # Git graph utilities
    ops.rs                          # Git operations (commit, merge, branch)
    worktree.rs                     # Git worktree allocation and cleanup
  app/
    mod.rs                          # AppConfig, main event loop (app::run)
    gates.rs                        # Gate execution (compile, test, lint)
    events.rs                       # TUI event dispatch
    parallel.rs                     # Parallel execution coordinator
    sequential.rs                   # Sequential execution fallback
    tui_actions.rs                  # TUI action handlers (inject, resume, etc.)
    util.rs                         # Shared utilities
  state/
    persistence.rs                  # Crash reports, state snapshots, recovery
  monitor/
    mod.rs                          # Background monitor framework
    steering.rs                     # Conductor steering logic
    patterns.rs                     # Error pattern detection
    config.rs                       # Monitor configuration
  tui/
    mod.rs                          # Terminal setup/restore, frame rendering
    layout.rs                       # Screen layout computation
    input.rs                        # Key event handling
    color.rs                        # ROSEDUST palette
    theme.rs                        # Theme configuration
    atmosphere.rs                   # Atmospheric effects (noise, scanlines)
    postfx.rs                       # Post-processing effects
    postfx_pipeline.rs              # Effect composition pipeline
    vfx.rs                          # Visual effects (bloom, glow)
    math.rs                         # Animation math (lerp, easing)
    bars.rs                         # Progress bar rendering
    tabs.rs                         # Tab navigation
    effects_config.rs               # Effect configuration
    views/                          # Screen views
      dashboard.rs                  # Main dashboard (wave progress, agent pool, budget)
      pipeline.rs                   # Pipeline phase visualization
      plans.rs                      # Plan list with status badges
      tasks.rs                      # Task checklist view
      agents.rs                     # Active agent pool
      review.rs                     # Review verdict display
      logs.rs                       # Scrollable log viewer
      git_view.rs                   # Git branch tree
      monitors.rs                   # System metrics
      config.rs                     # Runtime configuration
    widgets/                        # 26 reusable TUI widgets
      header_bar.rs                 # Top bar (batch, wave, phase, budget)
      status_bar.rs                 # Bottom status line
      plan_list.rs                  # Plan list with phase indicators
      plan_tree.rs                  # Tree view of plan dependencies
      task_progress.rs              # Task completion progress
      agent_pool.rs                 # Agent pool display
      agent_grid.rs                 # Grid layout for active agents
      agent_output.rs               # Live agent output stream
      parallel_pool.rs              # Parallel execution pool
      wave_bar.rs                   # Wave progress bar
      wave_progress.rs              # Wave completion tracker
      phase_bar.rs                  # Phase pipeline visualization
      phase_timeline.rs             # Phase timing timeline
      token_bar.rs                  # Token usage bar
      token_sparkline.rs            # Token usage sparkline (braille)
      braille.rs                    # Braille character rendering
      branch_tree.rs                # Git branch tree widget
      context_gauge.rs              # Context budget gauge
      diff_panel.rs                 # Diff display
      error_digest.rs               # Structured error display
      command_output.rs             # Command output panel
      status_badge.rs               # Status badge (pass/fail/pending)
      scrollbar.rs                  # Scrollbar
      tab_bar.rs                    # Tab bar
      sys_metrics.rs                # CPU/memory/disk gauges
    modals/                         # Modal dialogs
      help.rs                       # Key binding help
      inject.rs                     # Message injection to running agent
      plan_detail.rs                # Full plan detail view
      task_detail.rs                # Full task detail view
      task_picker.rs                # Task selection
      wave_overview.rs              # Wave detail view
      batch_review.rs               # Batch pause review
      approval.rs                   # Review approval/rejection
      agent_pool_modal.rs           # Agent pool management
      confirm.rs                    # Confirmation dialog
      notification.rs               # Notification toast
      quit.rs                       # Quit confirmation
  sys_metrics.rs                    # System resource monitoring (sysinfo)
```

## TUI

Mori's TUI is a Ratatui application with 10 views, 26 widgets, and 12 modal dialogs. It uses the ROSEDUST palette from Bardo's design system (rose on violet-black, CRT scanlines, phosphor effects).

### Key bindings

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

### What you see

The dashboard shows wave progress, active agents with their current task and token consumption, gate results, review verdicts, budget tracking (per-agent, per-plan, per-milestone), and system metrics. Token sparklines render in braille characters. Phase timelines show how long each stage took. The agent pool displays live output streams from all running agents.

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

## Crash recovery

Mori writes crash reports to `.mori/runs/` on both panics and non-panic errors. Reports include:
- Error message and backtrace
- App state at time of crash (which plans were running, which agents were active, what phase each was in)
- Recent log lines from `bardo-ctl.log`
- Error signature (for deduplication)
- Environment information

The supervisor script (`mori-supervisor.sh`) watches the process and restarts on crash. Since all state is on disk (committed code, context artifacts, plan progress), the restart picks up where it left off.

## Shell scripts

| Script | Purpose |
|--------|---------|
| `mori.sh` | Primary entry point; handles `.env`, gateway detection, default flags |
| `mori-supervisor.sh` | Watch supervisor -- restarts mori on crash |
| `bardo-enrich.sh` | Enrich plan artifacts (briefs, task breakdowns) before a run |
| `mori-gateway.sh` | Start bardo-gateway standalone |

## Embedded gateway

Mori can embed the bardo-gateway inference proxy (enabled by default if compiled with the `gateway` feature). The gateway sits between agents and model providers, providing:

- Multi-provider routing (Anthropic, OpenAI, OpenRouter)
- Three-layer caching (hash, semantic, prompt prefix)
- Prompt prefix alignment for cache hit rates
- Rate limit management and failover across API keys
- Cost tracking per agent, per plan, per milestone

When running embedded, the gateway starts on `--gateway-port` (default 4000) and agents route through it automatically. For shared setups, run the gateway standalone with `mori-gateway.sh` and point multiple mori instances at it.

## Dependencies

```toml
ratatui = "=0.29.0"       # TUI (pinned for rustc 1.85 compat)
crossterm = "0.28"        # Terminal event stream
tokio = "1.50"            # Async runtime
clap                      # CLI parsing
serde / serde_json / serde_yaml / toml  # Config and plan parsing
tracing / tracing-subscriber / tracing-appender  # Structured file logging
reqwest = "0.12"          # HTTP (gateway and LLM calls)
chrono                    # Timestamps
sha2                      # Plan content hashing
sysinfo                   # System metrics
dirs                      # Home directory resolution
fastrand                  # Lightweight random (worktree name generation)
bardo-gateway             # Optional (gateway feature, default enabled)
```

## Vision docs

The `tmp/death/` directory contains 29 vision documents covering the full roadmap:

| Doc | Topic |
|-----|-------|
| 00 | Overview and core thesis |
| 01 | Project structure |
| 02 | Document pipeline |
| 03 | Provider backends |
| 04 | Orchestration and DAG scheduling |
| 05 | Interfaces (TUI, CLI, MCP) |
| 06 | Server mode and remote operation |
| 07 | Deployment (Fly.io) |
| 09 | Inference gateway |
| 10 | Task routing and model selection |
| 11 | Queue management |
| 13 | Agent-native crypto (ERC-8004, x402, SIWE) |
| 15 | Cost tracking |
| 16 | Autonomous verification |
| 17 | Context engine (tree-sitter, PageRank, HDC) |
| 18 | Context as a service (MCP) |
| 19 | Rust performance targets |
| 21 | Agent optimization |
| 22 | Cybernetic learning (episodes, patterns, playbook) |
| 23 | Platform integrations |
| 25 | Dependency architecture |
| 26 | Live ingest (real-time operator directives) |
| 28 | Batch API strategy |
| 29 | Fly.io deployment |

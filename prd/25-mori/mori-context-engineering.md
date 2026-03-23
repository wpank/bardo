# Mori context engineering

**Version:** 1.0.0
**Last Updated:** 2026-03-23

---

> **Reader orientation:** This document explains how Mori assembles context for each agent. It covers the document hierarchy, enrichment pipeline, dynamic prompt budgeting, context injection, and the cost savings that result from doing all of this instead of dumping the whole codebase into a prompt. If you want the code, start with `apps/mori/src/orchestrator/prompts.rs` (budget computation), `inject.rs` (worktree injection), and `crates/mori-mcp/src/enrich.rs` (enrichment pipeline).

---

## The context problem

LLMs have fixed context windows. The best models top out at 200K tokens. A real codebase has millions. The Bardo monorepo alone, counting Rust source, PRD documents, plans, scripts, and configuration, runs well past 2 million tokens.

The naive approach is to dump everything in. Paste the whole repo into the prompt, let the model figure out what matters. Most AI coding tools work this way, maybe with some retrieval on top.

It wastes tokens on irrelevant code. An agent implementing OAuth providers does not need to see the deployment pipeline, the TUI rendering code, or the 47 other plans that have nothing to do with authentication. Every irrelevant token displaces a relevant one. It also degrades model attention -- research on long-context LLMs shows consistent degradation in the middle of large prompts ("lost in the middle"). More noise means the model is more likely to miss the signal.

And it costs a fortune. At $3/M input tokens for Sonnet, sending 150K tokens per agent per task across a 100-task build costs $45 in input alone. Do that a few times during development and you've burned through hundreds of dollars on context the model never needed.

Mori takes the opposite approach. Instead of sending everything and hoping, it pre-computes exactly the right context for each agent and assembles it into the smallest prompt that produces the best output.

## The document hierarchy

Context flows through five levels, each one compressing the previous:

```
PRD (5,000+ words, human-authored)
  -> Plans (10 plans, ~500 words each)
    -> Tasks (3-8 per plan, ~100 words each)
      -> Briefs (one per plan, targeted context only)
        -> Assembled prompt (budget-fitted to the model's window)
```

A PRD describes what you want built. Plans break it into ordered, dependency-aware chunks with explicit imports and exports. Tasks decompose each plan into atomic work units: exact files, acceptance criteria, estimated duration. Briefs pull together the plan, its PRD context, and cross-plan dependencies into one document an agent can read without needing anything else.

The compression is aggressive. A 5,000-word PRD becomes a 2,000-word brief for a single plan, which becomes a 500-word task description plus only the files that task touches. Agents get the right abstraction level with the right amount of detail. Nothing more.

This pyramid exists as files on disk. Every artifact is diffable, editable, and version-controlled. Nothing lives only in memory or conversation history.

## The enrichment pipeline

Before any agent spawns, Mori runs an enrichment pipeline that transforms raw plans into agent-ready artifacts. Nine steps, two phases.

**Phase 1 runs sequentially in real time.** These steps produce artifacts that later steps depend on:

- **PRD extraction** -- Regex-based, no LLM call. Scans the plan for `prd2/` references and extracts the relevant sections. Cost: $0.00. Time: milliseconds.
- **Brief generation** -- Parses plan headings to extract prerequisites, imports, exports, and execution order. No LLM call in extraction mode. Cost: $0.00.
- **Task generation** -- Parses `## Unit` headings and backtick-delimited file paths into structured TOML. No LLM call. Cost: $0.00.

Three of the nine enrichment steps need no LLM at all. They parse markdown structure, extract file paths from backticks, and build TOML from headings. This matters for reproducibility (same input always produces same output) and cost (free).

**Phase 2 is batchable.** These steps have no downstream dependencies within a single enrichment run, so they can be submitted to the Batch API for 50% cost savings:

- **Verification tasks** (Sonnet) -- Generates `verify-tasks.toml` with testable acceptance criteria
- **Review tasks** (Sonnet) -- Generates `review-tasks.toml` with review checklists
- **Step-by-step decomposition** (Sonnet) -- Breaks the plan into atomic steps with checkpoints
- **Testing backlog** (Sonnet) -- Generates test cases covering edge conditions
- **Review rubric and invariants** (Haiku) -- Produces scoring criteria for reviewers
- **Scribe task list** (Sonnet) -- Generates documentation tasks for post-implementation

The model routing is explicit in code. Haiku handles the cheaper steps (PRD extraction, brief generation, task generation, invariants). Sonnet handles the steps requiring deeper reasoning (verification, review, decomposition, testing, scribe). When you run `enrich-all.sh --batch`, Phase 1 executes immediately and Phase 2 submits to the Batch API for background processing.

Output for a single plan looks like this:

```
plans/03-oauth-providers/
  plan.md               # the plan itself
  brief.md              # synthesized execution document
  tasks.toml            # structured task breakdown
  prd-extract.md        # relevant PRD sections
  verify-tasks.toml     # verification acceptance criteria
  review-tasks.toml     # review checklist
  decomposition.md      # step-by-step execution guide
  testing-backlog.md    # test cases
  rubric.md             # review scoring rubric
  scribe-tasks.toml     # documentation tasks
```

Ten artifacts per plan, all generated before the first agent starts working. The agent's context is pre-computed, not discovered at runtime.

## Dynamic prompt budgeting

Different roles need different context. An Implementer needs plan details and code files. A Strategist needs the workspace-level view and room for analysis. A Reviewer needs the plan and the previous implementation's results. One-size-fits-all prompts waste tokens on some roles and starve others.

Mori computes per-role budgets dynamically based on the model's context window:

```
Total context window:
  200K tokens  (Opus, Sonnet)
  128K tokens  (GPT-5, o3, o4)
  100K tokens  (everything else)

Reserve 40% for output, 60% for input.
Convert tokens to characters at ~4 chars/token.
```

The 60% input budget gets divided differently per role:

**Implementer** -- 25% plan, 20% PRD extract, 10% workspace map, 10% brief, 10% reviews, 10% code files, 5% cross-plan context, 5% instructions, 5% skills

**Strategist** -- 30% plan, 20% workspace map, 20% PRD extract, 7% context, 5% brief, 10% reviews, 5% instructions, 5% skills (no code files; strategists don't edit code)

**Architect/Auditor** -- 25% plan, 15% workspace map, 15% PRD extract, 15% reviews, 10% brief, 8% code files, 5% instructions, 2% context

On a 200K-token model, the Implementer gets 120K tokens of input budget. The plan section alone gets 30K tokens (25% of 120K), more than enough for any plan. But code files get only 12K tokens (10%). That forces the system to be selective about which files it includes, which is the point.

When a section exceeds its budget, Mori truncates from the tail and appends a `...(truncated)` marker. Heads are preserved because sequential reasoning depends on the introduction and methodology, not the tail. High-priority sections (priority 5) get truncated rather than dropped entirely. Lower-priority sections get dropped first when the budget is tight.

Sections also carry a `cache_layer` tag. The prompt assembler inserts `<!-- mori:layer:N -->` markers at layer transitions. The gateway reads these markers to place `cache_control` breakpoints, maximizing prefix cache hits across requests.

## Context injection

Before each agent spawns, Mori creates a `context/in/` directory in the agent's git worktree and writes the files that agent needs. `inject.rs` handles this for both implementers and reviewers.

An Implementer worktree receives:

| File | Source | Purpose |
|------|--------|---------|
| `plan.md` | Per-plan directory | The plan spec |
| `brief.md` | Per-plan directory | Synthesized execution document |
| `tasks.toml` | Per-plan directory | Structured task list with files and acceptance criteria |
| `prd2-extract.md` | Per-plan directory | Relevant PRD sections |
| `verify-tasks.toml` | Per-plan directory | Verification acceptance criteria |
| `workspace-map.md` | Global artifact | Full crate/module listing |
| `preflight.md` | Global artifact | Git status, recent commits, cargo check result |
| `ignored-tests.md` | Global artifact | Known-broken tests to skip |
| `cross-plan-context.md` | `plans/CONTEXT.md` | Types defined by other plans, crate boundaries, public traits |
| `agent-messages.md` | `tmp/agent-messages.md` | Operator steering directives |
| `agents.md` | `AGENTS.md` | Agent role definitions and conventions |
| `prev-reviews.md` | Artifact store (iter > 1 only) | Previous iteration's review feedback |
| `playbook.md` | `.mori/memory/playbook.toml` | Pattern-matched advice from prior builds |
| `reflections.md` | Iteration memory | Prior iteration reflections (reflexion loop) |

That is 14 files for an implementer on iteration 2+, fewer on the first iteration.

Reviewers get the same set minus `tasks.toml` (they don't need it) plus `completion-summary.md`, which contains the implementer's compile status, test counts, notes, and deviations. The reviewer sees what the implementer did and what they said about it.

After the agent finishes, Mori reads structured output from `context/out/`. Implementers write `completion.json`. Reviewers write `review.json`. Both are typed schemas (`CompletionReport`, `ReviewReport`) that the orchestrator parses to decide what happens next: pass, retry with feedback, or escalate.

Context flows in one direction. Files go into `context/in/` before the agent starts. Structured results come out of `context/out/` after it finishes. The agent never searches the codebase for context. The context is already there.

## The 83% reduction

Walk through a concrete build.

A naive approach sends the full codebase context to every agent. For Bardo, that is roughly 150K tokens per agent per task. An implementer working on OAuth providers receives the entire workspace: gateway code, TUI rendering, deployment scripts, all 20 plans, the full PRD. Most of it irrelevant.

Mori sends ~25K tokens of targeted context:

| Section | Tokens |
|---------|--------|
| Plan spec | ~3,000 |
| Brief | ~2,000 |
| Relevant code files | ~8,000 |
| Task list (TOML) | ~1,000 |
| Workspace map | ~3,000 |
| PRD extract | ~4,000 |
| Previous reviews | ~2,000 |
| Reflections + playbook | ~2,000 |
| **Total** | **~25,000** |

That is an 83% reduction from 150K to 25K.

The cost difference scales with the build. At $3/M input tokens on Sonnet (and $15/M output tokens), a task that sends 150K input tokens and generates 10K output tokens costs roughly $0.60 in input and $0.15 in output. With Mori's targeted context, the same task sends 25K input tokens: $0.075 in input. Per task, that is $0.75 vs $0.225. Modest.

But a real build has 20 plans averaging 5 tasks each. That is 100 agent invocations, and each plan also gets reviewer passes (Architect, Auditor), adding another 40+ invocations. Across 140 invocations, the difference compounds: $105 naive vs $31.50 targeted. And that is before caching.

The gateway's prefix cache makes it even cheaper. The workspace map, PRD extract, and system prompt are identical across agents working on the same plan. Anthropic caches matching prefixes at a 90% discount. In practice, 40-50% of an agent's input tokens hit the prefix cache, dropping the effective input cost further.

The dollar savings are real. But the quality improvement matters more. An agent reading 25K tokens of targeted context produces better code than one drowning in 150K tokens of noise. Gate pass rates go up. Iteration counts go down. Context engineering pays for itself twice: once in cost, once in output quality.

More context is not better context. The right 25K tokens beat the lazy 150K every time.

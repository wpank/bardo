# Mori agent architecture

## Why specialization matters

Ask a general-purpose AI agent to "implement this feature" and watch what happens. It tries to write the code, write the tests, check for architectural problems, and update the docs, all in a single inference call with a single overloaded prompt. The result: mediocre code, missing edge cases, docs that parrot the implementation, and architectural decisions that nobody reviewed.

The problem is attention. A prompt stuffed with competing instructions ("write clean code AND check for regressions AND follow the existing patterns AND update the README") creates a priority conflict the model can't resolve. It'll spend tokens on whichever instruction is most salient in its context window, which is usually the last thing mentioned. Everything else degrades.

Mori separates concerns. One agent writes code. A different agent reviews the architecture. Another audits for bugs. Another writes documentation. Each agent gets a focused prompt, a context window loaded with only what it needs, and a model selected to match the difficulty of its specific job. A one-line import fix doesn't need Opus. An architecture review of a cross-crate refactor does.

This isn't just about prompt quality. It's about cost and speed. When the Implementer finishes writing code, the Architect and Auditor can review it in parallel. The Scribe can write docs at the same time. You're not waiting for a single monolithic agent to context-switch between tasks it shouldn't be doing simultaneously.

## Core pipeline roles

Every build, regardless of complexity, touches five roles.

**Conductor** is the meta-orchestrator. It doesn't write code or review anything. It watches all other agents, detects stalls, manages the token budget, and issues interventions: abort a stuck implementer, retry a failed gate, escalate a task to a higher-capability model, reallocate work when a parallel agent dies. The Conductor always runs on Opus because its decisions (abort vs. retry, escalate vs. continue) have outsized consequences. A bad call here wastes the entire build's budget.

**Implementer** writes code. Its context window contains the plan, the brief, the task list, and the relevant source files. Nothing else. No review instructions, no documentation templates, no architectural guidelines. It writes code. For standard tasks (single crate, clear requirements), it runs on Sonnet. For complex cross-crate work, it upgrades to Opus. The complexity classifier makes this decision automatically based on crate count, task count, estimated time, and dependency depth.

**Architect** reviews design correctness after the code is written and gates pass. Does the implementation match the PRD intent? Does it follow existing patterns in the codebase? Does it introduce architectural debt that'll compound across future plans? The Architect doesn't care about typos or missing error handling. That's someone else's job.

**Auditor** reviews code quality. Bugs, edge cases, error handling gaps, test coverage. The Auditor is looking at a different layer than the Architect: not "is this the right design?" but "does this code actually work?" Missing nil checks, uncovered error paths, race conditions in concurrent code.

**Scribe** generates documentation. Module-level docs, README updates, inline comments where the logic isn't self-evident. The Scribe gets the humanizer skill injected by default, so its output reads like a person wrote it.

Architect, Auditor, and Scribe run in parallel after gates pass. They don't block each other and they don't share context. Each reads the implementation output independently.

## Express mode roles

The full five-role pipeline is correct but expensive. For Standard-complexity plans (2-3 crates, a handful of tasks, under an hour of estimated work), Mori offers two express roles that compress the pipeline.

**QuickReviewer** combines the Architect and Auditor concerns into a single inference call. One agent checks both design correctness and code quality in one pass. For Standard plans, one review is enough. You don't need two separate agents spending tokens on overlapping context. The QuickReviewer runs on Claude, same as the Auditor, and gets a 5-minute budget.

**AutoFixer** handles post-gate failures that don't warrant re-running the full Implementer. A missing import, a type mismatch, a forgotten `pub` modifier. These are mechanical fixes. AutoFixer runs on Haiku, the cheapest and fastest model available, because the fix is almost always obvious from the compiler error. If the AutoFixer can't resolve it (the error requires real design thinking), the pipeline escalates back to the Implementer.

## Specialized validators

Domain-specific verification agents that run as additional gate phases when configured. Not every project needs all of them. You enable what matters for your codebase.

**TerminalValidator** verifies TUI rendering and interaction patterns. If your project has a terminal interface built on ratatui, this agent checks that widget layouts are correct, key bindings work, and visual rendering matches expectations.

**GolemLifecycleTester** tests the Golem creation, heartbeat, and mortality lifecycle. Specific to bardo's core domain: does the entity get created properly, does it maintain its heartbeat, does it die when it should?

**SpecDriftDetector** compares the implementation against PRD requirements. After several plans of iterative development, implementations drift from the original spec. This agent catches the gap before it compounds.

**RegressionDetector** checks for regressions in existing functionality. New code passes its own tests but breaks something that worked before. This agent runs the broader test suite and diffs behavior.

**PerformanceSentinel** monitors for performance degradation. Did this change add an O(n^2) loop where there was an O(n) one? Did it introduce unnecessary allocations in a hot path?

**CoverageTracker** verifies that test coverage targets are met. Not just "do tests exist" but "do the tests cover the branches that matter."

**CrossSystemTester** validates cross-crate integration. When a change touches the boundary between two crates, this agent verifies that both sides of the interface still agree.

**FullLoopValidator** runs end-to-end pipeline tests across the full system (mirage, terminal, runtime). The most expensive validator. Use it when you need confidence that the entire stack works together, not just individual components.

All of these run on Codex (via the AppServer backend) except FullLoopValidator, which runs on Claude.

## Utility roles

These agents support the pipeline without sitting on the critical path. They run in the background, ahead of the current work, or on-demand when something goes wrong.

**PrePlanner** does speculative task breakdown for upcoming plans. While the current wave of plans is building, PrePlanner looks at the next wave and generates draft task lists. When those plans reach the front of the queue, the breakdown work is already done.

**Refactorer** runs a batch refactoring pass every N plans. As agents implement features, they accumulate small bits of technical debt: duplicated helper functions, inconsistent naming, patterns that diverged across crates. The Refactorer cleans this up periodically rather than burdening every Implementer with refactoring concerns.

**MergeResolver** handles conflict resolution when plan branches can't cleanly merge. In parallel execution mode, multiple Implementers work in separate worktrees simultaneously. Their branches sometimes conflict. MergeResolver understands the intent of both changes and produces a correct merge rather than a mechanical one.

**ErrorDiagnoser** provides targeted analysis of gate failures when the Implementer's self-reflection isn't enough. Some failures are subtle: a test passes locally but fails in CI, a compile error points to the wrong line, a runtime panic has a misleading backtrace. ErrorDiagnoser digs deeper.

**Researcher** investigates a specific problem domain before implementation begins. When a task requires understanding an unfamiliar API, a complex algorithm, or an external system's behavior, the Researcher does that investigation and produces a summary the Implementer can consume.

## Backend mapping

Mori doesn't assume every agent should run on the same AI provider. Three backends are supported:

**Claude** (direct Anthropic API via the Claude Code CLI) is the default for core pipeline roles: Conductor, Implementer, Auditor, Scribe, Critic, Researcher, AutoFixer, QuickReviewer, and FullLoopValidator. These roles benefit from Claude's strong code generation and reasoning.

**Codex** (OpenAI via AppServer) handles the remaining specialized roles: Architect, Refactorer, PrePlanner, all the domain validators, ErrorDiagnoser, and the pattern/snapshot analysis agents. The `codex app-server` process provides a JSON-RPC interface that Mori drives with structured requests.

**Cursor** (via ACP, the Agent Communication Protocol) is available when IDE integration matters. Model slugs starting with `composer-`, `cursor-`, `sonnet-`, `opus-`, `haiku-`, `gemini-`, or `kimi-` route to Cursor automatically.

Backend selection is automatic: `AgentBackend::from_model()` infers the backend from the model slug. But you can override everything. The `--model` flag sets a global override. Per-role overrides live in `.mori/config.toml` under `[role_models]`. You can run Implementers on Claude while running Architects on GPT-4o if that produces better results for your codebase. The backend follows the model.

Fallback behavior is built in. If the primary model fails to spawn (rate limit, API error, service outage), the pool retries once with the configured fallback model. This happens transparently. The warm pool mechanism goes further: `MultiAgentPool` can pre-spawn agents before they're needed, so the next phase transition doesn't pay the cold-start cost.

## Skills injection

Skills are short behavioral instructions injected into agent prompts. They encode project-specific knowledge without rewriting the prompt templates.

The system works at two levels. Role-level defaults are defined in `default_skills_for_role()`: Scribe, Critic, and DocVerifier always get the `humanizer` skill, which teaches the agent to avoid AI-sounding prose. All other roles start with no default skills.

Task-level overrides come from the `skills` field in `tasks.toml`. If a specific task touches the cache layer, you add `skills = ["lru-eviction-policy"]` and that skill file gets loaded and injected into the Implementer's prompt for that task only. Skills are additive: task skills merge with role defaults, deduplicated.

Auto-detection adds a third layer. If the Implementer or TerminalValidator is working on files that contain "bardo-terminal" or "terminal" in their paths, the `ratatui-cinematic` skill is injected automatically. No manual configuration needed.

Skills resolve from two locations: `.claude/skills/{name}/SKILL.md` in the project root first, then `~/.claude/skills/{name}/SKILL.md` at the user level. Project-level skills override user-level ones. The content gets wrapped in `<skill name="...">` XML tags and injected into the prompt, with a character budget (currently 5% of the total input budget per role) split evenly across all loaded skills.

This is how you teach agents about your codebase's conventions without maintaining custom prompt forks. The skill files are version-controlled alongside the code. When conventions change, the skills change with them.

# How mori minimizes inference costs

AI inference is expensive. Claude Opus costs $5 per million input tokens and $25 per million output tokens. A 20-plan build where every task gets 100K tokens of Opus context runs up hundreds of dollars, and most of that spend is waste: context the agent didn't need, prompts that could have been cached, a $25/M-output model fixing a missing import statement. Mori treats inference cost as a first-class engineering constraint. Every layer of the system, from task classification to prompt assembly to payment delegation, is designed to spend less without producing worse code.

## Model tier routing

Not every task needs Opus. A config file update is not an architectural decision. Mori classifies tasks during enrichment and routes each one to the cheapest model that can handle it.

Three tiers:

**Haiku ($1/$5 per million tokens, input/output).** Briefs, task generation, classification, auto-fix attempts, reflection analysis. Anything that doesn't require deep reasoning. Fast, too: responses come back in 1-2 seconds.

**Sonnet ($3/$15 per million tokens).** Standard implementation, verification, review, decomposition. The workhorse model. Handles the bulk of actual code generation.

**Opus ($5/$25 per million tokens).** Complex cross-crate refactors, architectural decisions, conductor reasoning, security-critical code. Reserved for problems where cheaper models produce noticeably worse output.

The routing decision comes from tags assigned during enrichment. Each task gets scored on six dimensions: complexity (trivial through expert), category (implementation, refactor, test, docs, config, review, fix), quality (draft, production, critical), speed (fast, normal, careful), reasoning depth, and context weight. These tags feed a selection matrix:

```
                    trivial    simple     moderate   complex    expert
implementation      haiku      sonnet     sonnet     opus       opus
refactor            sonnet     sonnet     opus       opus       opus
test                haiku      haiku      sonnet     sonnet     opus
config              haiku      haiku      haiku      sonnet     sonnet
fix (fast cycle)    haiku      sonnet     sonnet     opus       opus
```

Quality shifts the tier up or down. A `critical` task bumps one tier (sonnet becomes opus). A `draft` task drops one tier (sonnet becomes haiku). The matrix lives in `.mori/config.toml` and is fully overridable, so you can force opus for everything in `src/crypto/` or haiku for all test tasks.

The gateway's pricing table (in `apps/bardo-gateway/src/pricing.rs`) tracks per-model rates and computes costs from actual token counts. Cached input tokens get a 90% discount on Anthropic models. The default fallback if a model isn't in the table: $3/$15 per million, which is Sonnet pricing. Conservative, not generous.

Result: a 20-task build that would cost $45 with Opus everywhere costs ~$18 with tier routing. Quality stays equivalent because gate checks (compile, test) don't care which model wrote the code.

## Batch API for 50% savings

Six of the nine enrichment pipeline steps don't need real-time responses. Verification, review pre-scoring, decomposition, testing backlog, invariants, scribe tasks: all of these can wait minutes instead of demanding sub-second streaming.

Anthropic's Batch API processes these requests asynchronously at half the per-token price. The gateway's `BatchManager` queues requests, auto-flushes when the queue hits 50 items or 30 seconds have elapsed, and polls for results in the background. Results typically arrive in 10-30 minutes.

Since enrichment runs before agent execution starts, the latency is absorbed into the pipeline. Nothing blocks. The enrichment scripts submit all their prompts as batch items, flush, and poll. A 20-plan build with 32 enrichment scripts per plan generates ~640 batch requests, all processed in a single Anthropic batch at half price.

For a build where enrichment costs $3 at real-time rates, batch processing drops that to $1.50. Across 100 builds a month, that is $150 in savings from this one change alone.

## Gateway caching

The bardo-gateway sits between agents and model providers. Every API call passes through it. Agents don't know it exists. The gateway implements three caching layers, each catching a different class of redundancy.

**Hash cache (L3).** SHA-256 of the full request body. Sub-millisecond lookup in an LRU. Catches identical repeated requests: retries, deterministic enrichment passes, the same review prompt sent to multiple plans. Hit means zero cost.

**Semantic cache (L2).** Embedding-based similarity search. The gateway embeds each prompt with a cheap embedding model and stores it in an HNSW index. When a new request's embedding is above a cosine similarity threshold (default: 0.92), the cached response returns instead. "Explain the auth middleware" and "What does the auth middleware do?" are different strings but the same question. Lookup takes 5-20ms, still far faster than an inference call.

**Prompt prefix cache (L1).** Anthropic's server-side caching gives a 90% discount on cached input tokens. Mori structures every prompt so the system prompt, workspace map, and PRD extract form a stable prefix. These rarely change between requests. Only the task-specific tail varies. For an 80K-token prompt where 60K is cached prefix, you pay for 60K at the 10% rate plus 20K at full rate. That is a 67% savings on input alone.

A warm gateway with a shared cache across runs means repeated patterns (the workspace map, the type registry, the PRD sections that every agent receives) are cached after the first request. If three mori instances share a remote gateway, they share the cache too.

## Reflection dedup

When a gate check fails (compile error, test failure), mori generates a reflection: a Haiku analysis of what went wrong and what to try differently. Each reflection costs ~$0.01.

But the same compile error often recurs across iterations. The error code matches, the file matches, the problem is the same. The reflection system checks iteration memory before generating a new reflection. It extracts the first error line from the gate output and compares it against previously recorded patterns. If it has seen this exact error before, it skips the API call and reuses the prior diagnosis.

The dedup logic lives in `reflection.rs`. The function `spawn_reflection` loads the plan's iteration memory, checks `has_error_pattern` against the current error line, and returns early if there is a match. Over a 20-plan build with 2-3 iterations per plan and ~40 total gate failures, this saves dozens of Haiku calls. Not a massive dollar amount per build, but it adds up at scale, and it is the right engineering instinct: don't pay for answers you already have.

## Budget delegation via SPTs

The orchestrator allocates its total budget across sub-agents using Shared Payment Tokens. Each SPT is a scoped, time-limited authorization to spend money. The breakdown for a typical $15 build:

- Implementer agent: $8.00 max
- Reviewer agent: $3.00 max
- AutoFixer agent: $2.00 max
- Reserve: $2.25 (for retries and model escalation)

Each SPT has an expiry (2-4 hours), a scope (which services the agent can charge, like the inference gateway and MCP tools), and a hard cap. If an agent exhausts its token, it stops and reports to the conductor. The conductor can reallocate from the reserve, downgrade the model tier for remaining tasks, or escalate to the user for a top-up.

No single agent can drain the entire budget. The config enforces a ceiling: by default, no sub-agent gets more than 60% of the total allocation. This is protection against runaway agents, but it is also a cost discipline mechanism. An implementer with an $8 cap makes different decisions than one with unlimited spend. It escalates less aggressively, uses cheaper models when it can, and stops earlier when stuck.

## Concrete cost walkthrough

A 6-plan build, budgeted at $15 total. Here is where the money goes:

| Phase | Detail | Cost |
|---|---|---|
| Enrichment | 9 steps per plan, 6 batched at 50% off | ~$1.50 |
| Implementation | 6 plans, avg 4 tasks each, mostly Sonnet | ~$8.00 |
| Gates | cargo check + cargo test, zero inference | $0.00 |
| Reviews | 3 complex plans get QuickReviewer, 3 skip | ~$2.00 |
| Reflections | 4 gate failures at $0.01 each | ~$0.04 |
| AutoFixer | 2 simple gate failures via Haiku | ~$0.30 |
| Reserve | Unused, returned to client | ~$3.16 |

Total actual spend: $11.84 against the $15 budget. The client gets $3.16 back.

Notice what is not costing money. Gate checks are pure computation: `cargo check` and `cargo test` run locally and cost zero inference tokens. Three of six plans skip review entirely because their tasks are simple enough that passing gates is sufficient verification. Reflections cost four cents total. The AutoFixer handles two trivial compile errors with Haiku at $0.15 each instead of burning Sonnet or Opus tokens on an obvious missing import.

The expensive line is implementation at $8. That is where the actual code gets written, mostly by Sonnet. The three complex plans that warranted review used another $2. Everything else, all the infrastructure around it, costs less than $2 combined.

This is the goal: spend on the work that matters, spend as little as possible on everything else, and give back what you don't use.

# mori-service

Paid build orchestration over HTTP. You describe what you want built, mori prices it, you fund it with USDC, and mori builds it with a fleet of AI agents while you watch costs tick in real time.

This is the service layer that turns mori (the local build orchestrator) into something anyone can call over a network. It adds the parts that make that work: a proposal flow so you know what you're paying before committing, USDC payment via x402/MPP, live cost streaming via SSE, and surfaces for GitHub issues and Twitter DMs.

## The thesis

The bottleneck in AI-assisted software engineering is not model quality. It is context.

Give an LLM the right 3,000 tokens and it writes correct code. Give it 30,000 tokens of noise and it hallucinates. Current tools put you in a chat window and hope the model figures out what to do. The model sees whatever fits in its context window, which is usually not enough. You re-explain the same things, watch the agent make decisions that conflict with work done two conversations ago, and manually stitch together outputs that don't fit.

Mori's answer: don't give the model a chat window. Give it a document hierarchy where each layer compresses and targets context for the layer below it.

```
PRD (what the client wants)
  -> Plans (how to build it, in what order, with dependency DAG)
    -> Tasks (atomic units, with files, acceptance criteria, routing tags)
      -> Briefs (everything an agent needs, budget-fitted to its context window)
        -> Prompt (what the agent actually sees — smallest possible, highest relevance)
```

An implementer doesn't need the entire PRD. It needs the 3 paragraphs relevant to its task, the types it should import, the files it should touch, and the tests that prove it worked. The enrichment pipeline extracts exactly that. The service layer wraps this pipeline in economics: price the work, collect payment, execute, deliver, settle.

## How the engine works

When a client funds a proposal, mori doesn't just spawn an agent and hope. It runs a structured pipeline that turns requirements into deployed code through five orchestration layers.

### Document pipeline

The enrichment pipeline generates all downstream artifacts before any agent runs. For each plan, it produces:

- **PRD extracts** — the specific paragraphs from the client's requirements relevant to this plan
- **Task TOMLs** — atomic units of work with file assignments, acceptance criteria, complexity classification, and parallel grouping
- **Decompositions** — step-by-step implementation instructions with checkpoints
- **Briefs** — synthesized context packages fitted to the agent's token budget
- **Verify chains** — runnable invariant check scripts
- **Review rubrics** — structured evaluation criteria per plan

Each artifact is a file on disk — diffable, editable, versionable. Nothing lives only in memory.

### DAG scheduling

Plans declare dependencies. Mori builds a directed acyclic graph and groups plans into execution waves:

```
Wave 1: [plan-01, plan-02]          # no dependencies, run in parallel
Wave 2: [plan-03]                    # depends on 01 and 02
Wave 3: [plan-04, plan-05, plan-06]  # depend only on 03
```

But plan-level parallelism leaves performance on the table. The unified task DAG goes further: it looks at all tasks across all plans in a wave, builds a file-conflict graph, and partitions them into independent groups using union-find. Tasks in different groups run simultaneously even if they belong to different plans.

For a 20-plan project, this typically extracts 3-4x more parallelism than wave scheduling alone. A 5-hour sequential build drops to 25 minutes with 8 agents.

### Agent dispatch and isolation

Each agent gets its own git worktree — a physical copy of the repo on its own branch. No shared mutable state between agents. All worktrees share a single `sccache` instance, so the second agent to compile a shared dependency gets a near-instant cache hit.

The agent receives an assembled prompt containing the plan, brief, task TOML, workspace map, and relevant file context. Prompt assembly uses priority-based greedy fitting: each context section has a priority, and the assembler packs highest-priority sections first until the model's context window is full.

### Gate pipeline

After the implementer finishes, the plan passes through verification gates:

```
Preflight -> Implement -> Compile Gate -> Test Gate -> Review -> Verdict -> Merge
```

**Compile gate** — `cargo check`. Binary pass/fail. On failure, the implementer gets another iteration with a structured error digest (unique errors with file:line references, not pages of raw compiler output).

**Test gate** — `cargo test`. On failure, the implementer sees which tests failed and gets another iteration.

**Review** — up to three reviewers run in parallel (Architect, Auditor, Scribe), each producing a structured verdict. All approve -> merge. Any request changes -> another iteration with review feedback.

A plan can iterate up to 8 times. Each re-review checks only changed files.

### Task routing

Not every task needs the same model. A trait definition is harder than a config file update. A security audit needs more rigor than a README tweak.

During enrichment, each task gets classified across six dimensions:

| Dimension | Values | What it controls |
|-----------|--------|-----------------|
| Complexity | trivial, simple, moderate, complex, expert | Model tier (haiku/sonnet/opus) |
| Category | implementation, refactor, test, docs, config, review, fix | Base model selection |
| Quality | draft, production, critical | Tier shift up/down |
| Speed | fast, normal, careful | Latency budget |
| Reasoning | none, light, needs_extended | Extended thinking toggle |
| Context weight | light, medium, heavy | Token budget allocation |

A trivial config task routes to Haiku ($0.80/M input). A complex implementation routes to Opus ($15/M input). The classification itself costs fractions of a cent per task — Haiku reads the task description and assigns tags.

Budget-aware degradation: as spend increases through the build, remaining tasks route to cheaper models. Gates are model-agnostic — code compiles regardless of which model wrote it.

## The 9-layer context engine

Nine layers compose to reduce input tokens by 76% and increase gate pass rate from 65% to 92%:

| Layer | What it does | Cost | Impact |
|-------|-------------|------|--------|
| AST extraction | Tree-sitter parse: signatures, types, imports | $0, 6ms/file | 10-50x token reduction vs reading full files |
| Workspace index | Symbol graph + PageRank ranking | $0, 2ms query | Top-50 symbols cover 80% of cross-file references |
| Semantic search | HDC fingerprint + optional embedding hybrid | $0, 15ms | 94% retrieval accuracy vs 62% for grep |
| Change detection | Blake3 content hashing, incremental re-parse | $0, 1ms | Only re-process what changed |
| Prefix alignment | Deterministic JSON serialization for cache hits | $0, 5ms | 91% Anthropic prefix cache hit rate |
| Context compression | Structural (signatures only) + token-level | $0, 50ms | 4.2x compression ratio |
| Research agent | Cheap agent explores codebase before planning | $0.10, 30s | Grounds plans in actual code state |
| Extended thinking | Claude extended thinking at decision points | $0.30 | Reduces plan structure errors |
| Quality gates | Static analysis + LLM-judge rubric | $0.02, 25s | 94% first-pass gate rate |

Each layer targets a different failure mode. AST extraction replaces expensive LLM calls with free deterministic parsing. The workspace index finds relevant cross-file context that grep misses. Prefix alignment makes the inference gateway's cache actually work. They stack multiplicatively — the combined effect is that a task costing $2.50 via Claude Code direct costs $0.42 through mori.

## Inference gateway integration

Every LLM call from every agent passes through the bardo-gateway inference proxy. The gateway adds three cache layers (hash, semantic, prefix), multi-provider routing, cost tracking, and concurrency control. The service doesn't manage any of this directly — it just points agents at the gateway and the gateway handles the rest.

Measured savings from the gateway alone: 85% cache hit rate, 96.6% cost reduction on repeated patterns ($182 actual vs $5,352 naive cost in one production run).

For the service, the gateway is also how MPP payment works. The client's USDC session funds inference calls through the gateway. The gateway tracks per-request cost and deducts from the session balance. The service sees cost headers on every response and streams them to the client via SSE.

## Cost analysis

Concrete comparison (OAuth2 service, 3 plans, 12 tasks):

| Metric | Claude Code direct | Mori |
|--------|-------------------|------|
| Cost | $3.34 | $1.00 |
| Time | 5 hours | 25 minutes |
| Bugs found post-merge | 2 + 1 security issue | 0 |

At scale (projected):

| Complexity | Without mori | With mori | Savings |
|------------|-------------|-----------|---------|
| Trivial (config change) | $0.50, 15min | $0.05, 2min | 10x |
| Simple (single feature) | $2.00, 1.5hr | $0.30, 10min | 6.7x |
| Moderate (multi-file feature) | $8.00, 5hr | $1.00, 25min | 8x |
| Complex (cross-module system) | $25.00, 2 days | $4.00, 2hr | 6.3x |
| Major (full service) | $80.00, 1 week | $12.00, 6hr | 6.7x |

The savings come from every layer: document pipeline (-35%), task routing (-40%), prefix caching (-45%), hash/semantic cache (-15%), context compression (-20%), AST extraction (-8%), HDC pre-filter (-3%). They compound — an 87% cumulative reduction.

## Why this exists

Mori was built as a local tool. You run it in a terminal, it reads plan files from disk, and it dispatches agents. That's fine when you're the one running it. But the interesting question is: what if someone else pays you to run it for them?

That question turns a build tool into a service. And services need things build tools don't: pricing, payments, scope negotiation, budget controls, live status updates, settlement receipts. This crate is all of that.

The payment model uses USDC on Base via the Machine Payment Protocol (the `mpp` crate). No accounts, no invoices, no billing cycles. The client's wallet is their identity. They sign an ERC-3009 authorization, and the service draws from it as it works.

## The lifecycle

Every interaction follows the same path: **draft -> proposal -> run -> delivery -> settlement**.

### 1. Drafting

The client describes what they want. Maybe it's vague ("I need an OAuth service"), maybe it's specific. Either way, they iterate on it through a conversational API. Each interaction costs a few cents in inference tokens, billed per-request via x402.

```
POST /api/v1/drafts
{ "content": "I want an OAuth2 service with Google and GitHub providers" }

-> { "draft_id": "d-1a2b", "response": "Here's a rough structure...", "session_cost_so_far": 0.03 }
```

The draft accumulates context: PRD fragments, architectural decisions, clarifications. The client sees `session_cost_so_far` climbing as they iterate. No commitment, no escrow, just pay-as-you-think.

```
POST /api/v1/drafts/d-1a2b/iterate
{ "message": "Actually, also support SAML for enterprise" }

-> { "response": "SAML adds complexity. Here's what changes...", "session_cost_so_far": 0.06 }
```

### 2. Proposal

When the draft is solid enough, the client asks for a formal proposal. Mori decomposes the work into milestones, estimates task counts and model tiers, and prices it.

```
POST /api/v1/proposals
{ "draft_id": "d-1a2b" }

-> {
     "proposal_id": "p-3c4d",
     "milestones": [
       {
         "name": "Core Auth",
         "plans": ["01-user-model", "02-session-store", "03-oauth-providers"],
         "tasks": 12,
         "estimated_cost": { "inference": 8.50, "compute": 2.00, "total": 10.50 },
         "estimated_time": "~45 min"
       },
       {
         "name": "Rate Limiting & Hardening",
         "plans": ["04-rate-limiter", "05-security-audit"],
         "tasks": 6,
         "estimated_cost": { "inference": 4.20, "compute": 1.00, "total": 5.20 },
         "estimated_time": "~25 min"
       }
     ],
     "total_cost": 15.70,
     "draft_cost_spent": 0.45,
     "net_cost": 15.25,
     "valid_for": "24h"
   }
```

Cost breaks down by milestone, by type (inference vs compute), and by plan. Draft costs already spent are deducted. The client sees exactly what they're paying for.

Proposals can be modified before acceptance -- add milestones, remove them, adjust scope. Each modification re-estimates costs.

### 3. Acceptance and funding

The client accepts and funds the proposal. Two funding options:

**Escrow** -- full amount locked on-chain. Funds release milestone-by-milestone as each passes its gates. If the build fails, remaining funds refund.

**Session** -- MPP session with a deposit. Mori draws per-request as it works. The client can top up mid-build. Unused funds return when the session closes.

```
POST /api/v1/proposals/p-3c4d/accept
{ "funding": "session", "deposit": 20.00 }

-> { "run_id": "r-5e6f", "session_id": "s-7g8h", "status": "building" }
```

### 4. Building (live cost streaming)

The run executes through mori's orchestrator: plans dispatch to agents in parallel waves, agents write code in isolated worktrees, gates verify it, reviewers check it, branches merge. The client watches it happen through SSE.

```
GET /api/v1/runs/r-5e6f/events

data: {"event":"plan_started","plan":"01-user-model","milestone":"Core Auth","timestamp":1711234567}
: X-Mori-Run-Cost: 2.40, X-Mori-Budget-Remaining: 17.60, X-Mori-Budget-Pct-Used: 12

data: {"event":"gate_passed","plan":"01-user-model","gate":"compile","timestamp":1711234600}
: X-Mori-Run-Cost: 3.10, X-Mori-Budget-Remaining: 16.90, X-Mori-Budget-Pct-Used: 15
```

Every SSE event carries cost headers as comments. The client always knows where the budget stands.

A separate cost endpoint gives the full breakdown:

```
GET /api/v1/runs/r-5e6f/cost

-> {
     "milestones": [
       { "name": "Core Auth", "status": "complete", "estimated_cost": 10.50, "actual_cost": 9.80, "delta": -0.70 },
       { "name": "Rate Limiting", "status": "in_progress", "estimated_cost": 5.20, "actual_cost_so_far": 2.10 }
     ],
     "total_estimated": 15.70,
     "total_actual": 11.90,
     "remaining_deposit": 8.10
   }
```

### 5. Mid-build adjustments

Builds don't always go to plan. The service handles three kinds of mid-build adjustments:

**Top-up** -- add more budget. If a build pauses because the budget ran out, topping up resumes it.

```
POST /api/v1/runs/r-5e6f/top-up
-> { "new_budget": 25.00, "status": "resuming" }
```

**Reduce scope** -- skip plans to cut costs. Completed plans can't be skipped.

```
POST /api/v1/runs/r-5e6f/reduce-scope
{ "skip_plans": ["05-security-audit"] }
-> { "revised_cost": 12.20, "budget_remaining": 8.10 }
```

**Add features** -- extend the build with new plans mid-flight.

```
POST /api/v1/runs/r-5e6f/add-feature
{ "description": "Also add GitHub SSO", "max_additional_cost": 8.00 }
-> { "new_plans": ["06-github-sso"], "revised_total": 23.70, "status": "replanning" }
```

Each adjustment is incremental. No renegotiating the whole proposal.

### 6. Delivery and settlement

When the build finishes, the client retrieves deliverables (repo URLs, PR links, deployment URLs, each with an artifact hash) and a settlement receipt:

```
GET /api/v1/runs/r-5e6f/receipt

-> {
     "milestones_completed": 2,
     "total_cost": 14.30,
     "breakdown": { "inference": 10.20, "compute": 3.10, "total": 13.30 },
     "draft_cost": 0.45,
     "adjustments": [{ "type": "top_up", "amount": 5.00 }],
     "refund": 5.70,
     "receipt_hash": "0x..."
   }
```

The receipt hash can be verified on-chain. The refund returns to the client's wallet automatically.

## Surfaces

The REST API is the primary interface, but mori-service also connects to two external platforms that act as natural-language front doors.

### GitHub

Install as a GitHub App. When someone opens an issue with a `mori:build` label (or the bot is @mentioned), mori-service generates a proposal as an issue comment. Team leads approve by applying `mori:build` and commenting approval. The build runs, and mori opens a PR with the results.

Labels control the workflow:
- `mori:build` -- trigger a build from this issue
- `mori:investigate` -- research the issue without building
- `mori:review` -- review an existing PR
- `mori:hold` -- pause an active build
- `mori:planning`, `mori:building`, `mori:ready` -- lifecycle labels managed by mori

The bot posts cost breakdowns, milestone progress, and gate results as issue comments. When the build is done, it opens a PR and comments a receipt.

### Twitter

@mention the bot with a build request. Mori classifies the tweet (build request, approval, clarification, status check) and responds in-thread.

Two interaction modes based on cost:
- **Simple** (under threshold): single quote tweet, reply "BUILD" to approve
- **Conversational** (over threshold): multi-turn refinement thread before commitment

Rate limiting, account age checks, allowlists, and blocklists prevent abuse. The bot formats proposals as compact tweet threads with cost breakdowns.

## Cost estimation

The proposal engine prices work without making LLM calls. It parses the draft, identifies feature boundaries via keyword analysis, classifies complexity (trivial through epic), and runs the numbers.

Each complexity level maps to a task count range and a model tier distribution:

| Complexity | Tasks | Typical tier mix |
|-----------|-------|-----------------|
| Trivial | 2-3 | Mostly haiku |
| Small | 3-5 | Mostly haiku |
| Medium | 5-10 | Mixed sonnet/haiku with some opus |
| Large | 10-18 | Mixed sonnet/opus |
| Epic | 18-30 | Mostly opus |

Each task gets a token budget based on its tier. The cost module prices tokens using the gateway's rate table (opus at $15/$75, sonnet at $3/$15, haiku at $0.80/$4 per million input/output tokens), applies a configurable spread, and adds a retry buffer (default 15%) for gate failures that require re-runs.

## Running

```bash
cargo run -p mori-service

# With explicit config
MORI_SERVICE_PORT=8080 MORI_MASTER_KEY=my-secret cargo run -p mori-service
```

Configuration loads from `.mori/service.toml` or falls back to defaults. Environment variables override for the essentials.

## Configuration

```toml
# .mori/service.toml

[billing]
accept_proposals = true
min_proposal_value = 1.00
max_proposal_value = 500.00
retry_buffer_pct = 0.15

[billing.escrow]
evaluator = "automated"
release_per_milestone = true
dispute_window_hours = 48

[billing.session]
min_deposit = 5.00
max_duration_hours = 24
auto_close_idle_minutes = 30

[auth]
mode = "both"           # "api_key", "siwe", or "both"

[gateway]
spread_pct = 0.20       # 20% markup on inference costs

[integrations.github]
app_id = 123456
private_key_path = "/path/to/key.pem"
webhook_secret = "whsec_..."
bot_username = "mori-bot"

[integrations.github.defaults]
auto_propose_on_issue = true
auto_build_on_approve = true
max_cost_per_build = 100.0

[integrations.twitter]
api_key = "..."
api_secret = "..."
bearer_token = "..."
bot_username = "mori_build"
simple_mode_threshold = 10.0
max_requests_per_user_hour = 5
min_account_age_days = 30
```

## Authentication

Two auth modes, usable independently or together:

**API keys** -- bearer tokens prefixed `mori_sk_`. Three scopes: read, write, admin. The master key (from env var `MORI_MASTER_KEY` or config) always gets admin scope.

**SIWE sessions** -- Sign-In with Ethereum. Tokens prefixed `mori_sess_`. The wallet address becomes the identity. (Validation currently stubbed -- the tokens are accepted with write scope.)

```
Authorization: Bearer mori_sk_a1b2c3d4e5f6...
```

## API reference

| Method | Path | What it does |
|--------|------|-------------|
| `POST` | `/api/v1/drafts` | Create a draft from initial description |
| `GET` | `/api/v1/drafts/{id}` | Get draft with message history |
| `POST` | `/api/v1/drafts/{id}/iterate` | Add a message, get a response |
| `POST` | `/api/v1/proposals` | Generate a costed proposal from a draft |
| `GET` | `/api/v1/proposals/{id}` | Get proposal details |
| `POST` | `/api/v1/proposals/{id}/modify` | Add/remove milestones, re-estimate |
| `POST` | `/api/v1/proposals/{id}/accept` | Accept and fund, creates a run |
| `GET` | `/api/v1/runs/{id}` | Run status and milestone progress |
| `GET` | `/api/v1/runs/{id}/cost` | Live cost breakdown |
| `GET` | `/api/v1/runs/{id}/events` | SSE event stream |
| `POST` | `/api/v1/runs/{id}/top-up` | Add budget |
| `POST` | `/api/v1/runs/{id}/reduce-scope` | Skip plans |
| `POST` | `/api/v1/runs/{id}/add-feature` | Extend the build |
| `DELETE` | `/api/v1/runs/{id}` | Cancel, settle completed work, refund rest |
| `GET` | `/api/v1/runs/{id}/deliverables` | Build artifacts |
| `GET` | `/api/v1/runs/{id}/receipt` | Settlement receipt |
| `GET` | `/health` | Health check |

## Persistence

SQLite via `.mori/service.db`. Seven tables: drafts, proposals, runs, deliverables, receipts, api_keys, and rate_limits. WAL mode. All state survives restarts.

## Architecture

```
src/
├── main.rs                # Server startup, config loading
├── lib.rs                 # Service entry point
├── state.rs               # AppState, shared service state
├── types.rs               # Wire types (drafts, proposals, runs, receipts)
├── db.rs                  # SQLite persistence (7 tables, WAL mode)
├── api/
│   ├── mod.rs             # Route registration
│   ├── auth.rs            # API key + SIWE authentication middleware
│   ├── drafts.rs          # Draft CRUD + iteration
│   ├── proposals.rs       # Proposal generation + modification
│   ├── runs.rs            # Run management, SSE events, cost streaming
│   └── deliverables.rs    # Artifact delivery + settlement receipts
├── proposal/
│   ├── mod.rs             # Proposal engine coordination
│   ├── engine.rs          # Feature detection, complexity classification
│   └── cost.rs            # Token budgeting, tier pricing, spread calculation
└── integrations/
    ├── mod.rs             # Integration registration
    ├── github.rs          # GitHub App webhook handler
    └── twitter.rs         # Twitter bot interface
```

## What you can learn from this

If you're building a service that bills for AI agent work, the patterns here are transferable:

**Proposal-first billing.** Don't charge for work that hasn't been scoped. Let the client iterate cheaply (drafts), see a price (proposals), then commit (acceptance). The draft-to-proposal flow costs pennies; the build costs dollars. This separation of exploration from commitment changes how people interact with the system.

**Live cost transparency.** Every SSE event carries cost headers. The client never has to guess where the budget stands. Budget alerts fire at configurable thresholds. The client can top up, reduce scope, or cancel at any point. This removes the anxiety of open-ended AI billing.

**Incremental funding.** Top-ups, scope reductions, and feature additions are first-class operations, not exceptions. Builds are living things -- scope changes mid-flight. The billing model has to accommodate that without restarting the whole proposal cycle.

**Wallet-as-identity.** No user accounts, no email verification, no password resets. The client's Ethereum wallet is their identity. SIWE for session auth, ERC-3009 for payments. One credential handles both authentication and payment. This collapses an entire user management system into a signature.

**Settlement receipts.** Every completed build produces a receipt with a hash. The receipt is the proof. It records what was proposed, what was built, what was charged, and what was refunded. The hash makes it verifiable. If you're building on-chain settlement, this receipt is what gets submitted.

**Context over compute.** The 6-7x cost reduction doesn't come from cheaper models. It comes from feeding the same models better context. The document hierarchy, task routing, workspace index, and prompt assembly pipeline are where the savings live. The model selection matrix just avoids overpaying for easy tasks.

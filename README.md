![Bardo](demo/cover.png)

# Bardo

Software that builds itself, pays for its own inference, and gets better with every run.

Bardo is a Rust monorepo where the build system, the inference layer, the payment protocol, the EVM simulator, and the autonomous agents are all parts of the same organism. Mori orchestrates fleets of AI agents to write code. The gateway caches and routes their inference calls, cutting costs 40-85%. The agents pay for that inference with USDC through MPP. The services they build register as MCP tool servers and start earning revenue. The revenue funds the next build. The system remembers what worked, extracts patterns, and injects them into future agents so they don't repeat past mistakes.

It's a cybernetic loop: build -> deploy -> earn -> learn -> build better.

## The loop

```
 Specification (PRD)
       |
       v
 Mori decomposes into plans, enriches with AST-extracted context
       |
       v
 Agent swarm executes in parallel across isolated git worktrees
       |
       v                                          Gateway caches inference,
 Gates verify (compile, test, review)    <-----   routes across providers,
       |                                           tracks cost per request
       v
 Merge, deploy, register as MCP server
       |
       v
 Service earns USDC via x402/MPP micropayments
       |
       v
 Revenue funds next build cycle
       |
       v
 Memory system extracts patterns, promotes to playbook rules
       |
       v
 Next build is faster, cheaper, more accurate
```

Each piece below can be used independently. But together they form a closed loop where software funds its own evolution.

---

## [bardo-gateway](apps/bardo-gateway) -- inference proxy

<video src="https://github.com/user-attachments/assets/af53e532-ea51-4439-99b0-20b14ed5df8b" controls width="100%"></video>

An HTTP proxy that sits between your agents and LLM providers. Point any Anthropic or OpenAI SDK at port 4000 by changing the base URL. You get three cache layers, five provider backends, request normalization, cost tracking, tool pruning, batch processing, and USDC micropayments without changing a line of application code.

```bash
ANTHROPIC_API_KEY=sk-... cargo run -p bardo-gateway
# All agent traffic now routes through http://localhost:4000
```

### Three-layer cache

Every request passes through three cache layers before hitting a provider. Each targets a different class of waste.

**L1: Hash cache.** BLAKE3 hash of the normalized request body. In-memory moka LRU, sub-millisecond lookup. Catches identical repeated requests, retries, and deterministic prompts. In a typical agent run, 10-15% of requests are exact duplicates. Before hashing, three normalization passes run: UUID/timestamp stripping (replaces per-invocation noise with `[VAR]`), tool definition sorting (alphabetical by name, making hash order-independent), and JSON key ordering via BTreeMap (eliminates serializer-dependent ordering). These three passes alone increase L1 hit rates 15-25%.

**L2: Semantic cache.** Two backends: SimHash (default, 64-bit fingerprint, Hamming distance <= 3, ~50us for 10K entries, pure CPU) or fastembed ONNX embeddings (opt-in, cosine >= 0.92, ~3-5ms, better accuracy). Catches rephrased prompts -- "explain the auth middleware" and "what does the auth middleware do?" are different strings but match semantically. Tool-use responses are excluded (replaying cached tool IDs breaks subsequent turns). Entries persist to SQLite every 60 seconds and restore on restart.

**L3: Prompt prefix cache.** Anthropic caches KV state for shared prompt prefixes at a 90% discount on input tokens. The gateway injects `cache_control: {"type": "ephemeral"}` headers into system prompts and restructures requests so the cacheable prefix (system prompt + tool definitions + shared context) is maximized across agents. An 80K-token prompt where 60K is cached prefix saves $0.27 per call at Opus pricing. Over a 200-request session, that's $54. The BTreeMap trick: all JSON serialization uses BTreeMap for deterministic key ordering, so identical content produces identical bytes produces cache hits.

Combined effect: 40-85% cost reduction depending on workload repetitiveness. Measured on a production agent run: $182 actual vs $5,352 naive cost (96.6% reduction at 85% cache hit rate).

### Five provider backends

| Provider | What it does |
|----------|-------------|
| **Anthropic** | Primary. Supports key rotation across up to 10 keys (`ANTHROPIC_API_KEY` through `ANTHROPIC_API_KEY_10`). Round-robins on rate limit. |
| **OpenAI** | Chat Completions passthrough. Serves `gpt-*`, `o3`, `o4-mini` requests. |
| **OpenRouter** | Aggregator with 400+ models. Good fallback when primary providers rate-limit. Separate Anthropic rate limits. |
| **Venice** | TEE-attested inference with zero data retention. The gateway includes a three-tier security classifier (Standard/Confidential/Private) that scans requests via deterministic keyword matching -- no LLM calls. Eight triggers (PortfolioComposition, MevSensitive, RebalanceTiming, DealNegotiation, GovernanceDeliberation, CounterpartyAnalysis, DeathReflection, OwnerPii) route sensitive content to Venice automatically. Private-classified requests never fall back to a retaining provider. DIEM budget tracker monitors Venice-specific credit allocations. |
| **Bankr** | Self-funding agent wallets. An agent that earns revenue can pay for its own inference without a human paying API bills. The metabolic loop monitor tracks the sustainability ratio (`daily_revenue / daily_inference_cost`). When ratio >= 1.0, the agent is self-sustaining -- its economic death clock stops. Supporting modules: credit balance with vault fee conversion, model tier routing based on credit budget, cross-model verification for high-stakes actions (second model confirms parsed intent before execution), auto-replenish and throttle policies. |

All providers implement a `Provider` trait. Adding a new one means implementing `send`, `parse_response`, and `report_cost`. The gateway handles caching, normalization, and cost tracking uniformly across all providers.

Failover: when a provider errors or rate-limits, the gateway falls through to the next by priority. Privacy override: the failover chain is constrained to Venice for Private-classified content.

### Tool pruning

Agents defined with 30 tools include all 30 definitions in every request -- 100-500 tokens each, 2-13K tokens of dead weight if only 3 are used. The `ToolTracker` watches per-session tool usage and strips unused definitions after 5 requests. Saves 2-5K tokens/request. At Sonnet pricing, that's $0.006-0.015/request compounding across hundreds of requests.

### Batch API

Non-urgent work (enrichment, summarization, pattern extraction) routes through Anthropic's Batch API at 50% cost. Auto-flush at 50 items or 30 seconds. In a typical mori build, 40-60% of inference spend is non-urgent -- at 50% discount, that's a 20-30% reduction on total build cost stacking on top of caching savings.

### MPP: Machine Payment Protocol

HTTP 402-based USDC micropayments that let agents pay for inference the same way they pay for gas -- with a wallet and a signature. No API keys, no accounts, no invoices.

The protocol is a two-step HTTP exchange. Client sends a request without payment; gateway returns 402 with a `PaymentRequired` quote (amount in USDC base units on Base chain 8453, recipient wallet, expiry, nonce, cost breakdown showing provider cost and spread). Client signs an ERC-3009 `transferWithAuthorization` using EIP-712 typed data, retries with the signature in the `X-Payment` header. Gateway verifies off-chain via `ecrecover` (no RPC call needed), serves the request, returns a `Payment-Receipt` with the actual amount charged.

Two intents: **Charge** (one-shot, per-request signature) and **Session** (pre-funded balance with per-request draws, no re-signing). Sessions are stored in a concurrent DashMap with SQLite persistence and background TTL expiration. Close returns unused funds.

Reputation-tiered spread: 20% default, 18% Basic (5+ builds), 15% Verified (25+ builds, >90% gate pass), 12% Trusted (100+ builds), 8% Sovereign (500+ builds).

The protocol primitives live in the standalone [`mpp` crate](crates/mpp) -- usable by any Rust service building pay-per-request APIs.

### Cost tracking

Every response carries headers: `X-Mori-Cost-Usd` (actual), `X-Mori-Naive-Cost-Usd` (what you'd pay without the gateway), `X-Mori-Savings-Usd` (the delta), `X-Mori-Cache-Status` (hash-hit/semantic-hit/prefix-hit/miss). Per-model, per-session, and per-key breakdowns persist to SQLite. WebSocket live dashboard at `/v1/ws/stats` broadcasts per-request events in real time.

---

## [mori](apps/mori) -- build orchestrator

<video src="https://github.com/user-attachments/assets/dc71ba75-6af3-4e3f-81b4-5862c9d48d8e" controls width="100%"></video>

53,000 lines of Rust. Takes a specification, decomposes it into dependency-ordered plans, engineers targeted context for each one, and dispatches a fleet of AI agents in parallel across isolated git worktrees to implement, test, review, and merge the results.

```bash
./mori.sh 01-09 --express    # 20 concurrent agents, no reviews, maximum speed
./mori.sh 01-09 --dry-run    # print DAG, wave breakdown, no agents spawned
```

### Context engine (9 layers)

The bottleneck in AI-assisted development is not model quality. It is context. Give an LLM the right 3,000 tokens and it writes correct code. Give it 30,000 tokens of noise and it hallucinates. Mori's answer: nine layers of context engineering that compose to reduce input tokens by 76% and increase gate pass rate from 65% to 92%.

| Layer | What it does | Cost | Impact |
|-------|-------------|------|--------|
| **AST extraction** | Tree-sitter parses source into signatures, types, imports. 6ms/file initial, sub-ms incremental. Supports 100+ languages. | $0 | 10-50x token reduction vs reading full files |
| **Workspace index** | Symbol graph with PageRank ranking, biased per-task via files listed in task TOML. Top-50 symbols cover 80% of cross-file references. Cached in SQLite, keyed by content hash. | $0, 2ms/query | Finds context grep misses |
| **Semantic search** | HDC hypervector fingerprints (10,240-bit, XOR binding, 50ns Hamming distance) for structural matching + optional CodeRankEmbed ONNX embeddings (137M params, local, 10-50ms) for conceptual matching. Hybrid reranking combines semantic + keyword (ripgrep) + AST (tree-sitter query) signals. | $0, 15ms | 94% retrieval accuracy vs 62% for grep |
| **Change detection** | Blake3 content hashing at symbol granularity via Merkle tree. Only changed symbols invalidate downstream artifacts. Typical edit invalidates 2-5 plans, not 110. | $0, 1ms | Eliminates redundant re-enrichment |
| **Prefix alignment** | BTreeMap JSON serialization for deterministic byte ordering. Same content = same bytes = cache hit. Tool definitions, function schemas, structured context all use BTreeMap. | $0, 5ms | 91% Anthropic prefix cache hit rate |
| **Compression** | Structural: tree-sitter extracts signatures without implementations (200-line function becomes 3-line signature). Token-level: 4.2x compression ratio. | $0, 50ms | Half the tokens, same information |
| **Research agent** | Cheap agent explores codebase before planning to ground plans in actual code state. | $0.10, 30s | Prevents plans based on assumptions |
| **Extended thinking** | Claude extended thinking at architectural decision points. | $0.30 | Reduces plan structure errors |
| **Quality gates** | Static analysis + LLM-judge rubric scoring. | $0.02, 25s | 94% first-pass gate rate |

A task costing $2.50 via Claude Code direct costs ~$0.42 through mori. The savings come from every layer and compound multiplicatively.

### DAG scheduling

Plans declare dependencies in YAML frontmatter. Kahn's algorithm computes execution waves. But plan-level parallelism leaves performance on the table -- two plans in the same wave might touch different files. The `UnifiedTaskDag` goes deeper: builds a file-conflict graph across all tasks in all plans in a wave, partitions into independent groups via union-find, and dispatches groups concurrently. For a 20-plan project, this extracts 3-4x more parallelism than wave scheduling alone.

### Agent swarm

26 specialized roles across three backends (Claude Code, Cursor, Codex). Backend inferred from model slug. Mix in one run: opus implementer via Claude, haiku scribe via Cursor, gpt-class fixer via Codex.

Each plan runs a state machine: Preflight -> Strategist -> Implementer -> Compile Gate -> Test Gate -> Review (parallel Architect + Auditor + Scribe) -> Critic Verdict -> Merge. Failures loop back with cumulative DO NOT RETRY lists built from compiler errors, review blockers, and diff stats. Up to 8 iterations. Golden-path plans (first-try success) get indexed by category and fed as examples to future decompositions.

The **Conductor** monitors all running agents: nudges silent agents after 300s, restarts stalled agents after 600s, aborts stuck phases after 1800s. Escalates model tiers on failure (haiku -> sonnet -> opus). Manages spawn priority (implementers = 0/highest, conductor = 7/lowest).

### Task routing

Not every task needs the same model. Six classification dimensions (complexity, category, quality, speed, reasoning, context weight) determine model selection per task. A trivial config task routes to Haiku ($0.80/M input). A complex cross-module refactor routes to Opus ($15/M input). Classification costs fractions of a cent per task. Budget-aware degradation: as spend increases, remaining tasks route to cheaper models. Gates are model-agnostic -- code compiles regardless of which model wrote it.

### Three-tier learning

Every task execution writes a LanceDB episode (files changed, model used, tokens, cost, gate pass, iterations, HDC fingerprint, embedding). When 5+ similar episodes share a common outcome, mori extracts a pattern. Patterns that correctly predict outcomes across 5+ subsequent builds get promoted to playbook rules. Rules are injected directly into agent context at task start. Rules that stop being accurate auto-demote.

HDC fingerprints enable 50ns pattern matching -- 1000x faster than embedding-based lookup. Project-agnostic (encode structural characteristics, not identifiers), so patterns transfer across codebases.

Context budget allocation learns per task category. Auth tasks converge toward 30% playbook, 25% state. Config tasks converge toward 35% state, 5% types. Over 100 builds, the allocation optimizes itself.

### Worktree isolation

Each agent gets its own git worktree on its own branch (`codex/plan/{name}`). No shared mutable state. All worktrees share a single `sccache` instance with normalized base directories (`SCCACHE_BASEDIRS`), so the second plan compiling a shared dependency gets a near-instant cache hit.

### Embedded gateway

Mori compiles bardo-gateway as a library. When `--gateway` is set (default), it starts on port 4000 as a background tokio task and routes all agent inference through it. Three-layer caching, five providers, tool pruning, MPP, batch API -- all active by default.

### TUI

Ratatui application with 10 views, 26 widgets, 12 modal dialogs. ROSEDUST palette (rose on violet-black, CRT scanlines, phosphor effects). Dashboard shows wave progress, active agents, token sparklines (braille-rendered), gate results, review verdicts, per-agent/per-plan/per-milestone budget tracking. Agent pool displays live output streams. Inject messages to running agents with `i`.

### Crash recovery

Crash reports to `.mori/runs/` on panics and errors (backtrace, app state, recent logs, error signature). Supervisor script watches and restarts. All state on disk -- restart picks up where it left off.

---

## [mori-service](apps/mori-service) -- paid builds over HTTP

The service layer that turns mori into something anyone can pay to run. Describe what you want, mori prices it, you fund it with USDC, it builds while you watch costs stream in real time.

**Draft -> Proposal -> Run -> Delivery -> Settlement.** Each phase has a payment pattern: x402 micropayments for drafting (pennies per interaction), MPP session or ERC-8183 escrow for funded builds (dollars), x402 again for mid-build adjustments.

Proposal engine classifies complexity (Trivial through Epic) via heuristic keyword analysis -- no LLM calls. Prices each task against the gateway's rate table with model tier distributions, cache hit rate modeling, and a 15% retry buffer. Proposals break down cost by milestone, by type (inference vs compute), and by plan. Draft costs already spent are deducted.

Mid-build: top up budget, reduce scope (skip plans), or add features -- each adjustment is incremental, no renegotiating the whole proposal. SSE event stream carries cost headers on every event. Budget alerts at configurable thresholds.

GitHub App integration: issue opened -> mori comments proposal -> thumbsup to approve -> mori builds -> opens PR with cost breakdown and verification checklist. Slash commands: `/mori review`, `/mori investigate`, `/mori fix`, `/mori run 03-05`, `/mori cost`.

Twitter bot: @mention with a build request -> quote reply with scope/cost/time -> reply "BUILD" to approve. Simple mode (under threshold) or conversational mode (multi-turn refinement). Rate limiting, account age checks, allowlist/blocklist.

SQLite persistence (7 tables, WAL mode). Auth: API key (`mori_sk_*`) with read/write/admin scopes + SIWE (Sign-In with Ethereum).

---

## [mirage-rs](apps/mirage-rs) -- EVM fork simulator

A local Ethereum node for development, like Anvil -- but connected to live chains. Forks mainnet state lazily over RPC, keeps watched contracts in sync block-by-block, and gives you the full `eth_*` / `evm_*` / `anvil_*` manipulation API. Drop-in replacement. Existing Foundry, Hardhat, and Viem tooling works unchanged.

```bash
mirage-rs --rpc-url https://eth-mainnet.g.alchemy.com/v2/KEY --ws-url wss://eth-mainnet.g.alchemy.com/v2/KEY
```

### Targeted following

Where Anvil forks at a pinned block and freezes, mirage-rs follows the chain forward. A WebSocket subscriber watches `newHeads`, filters each block for transactions touching watched contracts, and replays only those locally. For a typical DeFi portfolio (3-10 positions): ~5-15 transactions per block instead of the full ~150. Blocks process in <100ms at steady state.

Contracts enter the watch list three ways: manual (`mirage_watchContract`), auto-classification (diff classifier sees 3+ storage slot writes on a new address and promotes it), or contagion (replayed transaction writes to a new contract that crosses the threshold, recursively extending the watch list across composability chains).

### Three-layer state model

```
DirtyStore (local writes)  →  ReadCache (LRU + TTL, <1us hot reads)  →  UpstreamRpc (token-bucket rate-limited lazy fetches)
```

Reads flow top-down, first hit wins. Writes go into the dirty overlay and never touch upstream. On first access, balances/nonces/storage/bytecode are fetched and cached. You get a mutable view of mainnet state without syncing anything.

### Copy-on-write scenario branching

Scenarios fork from a shared baseline using CoW overlays (~12.8KB per branch vs ~3.2MB for a full clone). Run parallel what-if simulations cheaply. Sequential mode reverts between runs; parallel mode uses independent branches that can't observe each other's mutations.

Scenario sets support TOML fixtures with transaction sequences, tracked addresses, gas budgets, timeouts, and assertions (balance checks, watch list membership, custom invariants). Included scenarios: Uniswap V3 entry, ETH crash selloff, Aave liquidation, new pool deployment, volume spike.

### Resource management

Three profiles (micro/standard/power) with memory ceilings (256MB/512MB/2GB). Tiered pressure response: evict LRU cache at 50%, demote auto-classified contracts to slot-only reads at 70%, fall to proxy mode (disable replay) at 90%. Process checks available memory at startup and exits if the profile doesn't fit.

### Mirage-specific RPC extensions

Beyond full Anvil/Hardhat compatibility: `mirage_mintERC20` (auto-detects balance storage slots), `mirage_watchContract`/`mirage_unwatchContract`, `mirage_getPosition` (DeFi position snapshots), `mirage_subscribeEvents` (WebSocket event stream with address/topic filters), scenario sets (`mirage_beginScenarioSet`, `mirage_defineScenario`, `mirage_runScenarioSet`, `mirage_compareScenarios`), and resource introspection (`mirage_getResourceUsage`, `mirage_setResourceLimits`).

---

## Golems -- mortal autonomous agents

<video src="https://github.com/user-attachments/assets/d515773e-6024-4d89-aad7-3e68faa827c1" controls width="100%"></video>

A Golem is a mortal autonomous agent compiled as a single Rust binary. It has a wallet, a strategy, a knowledge base, and a finite lifespan. It runs on a VM (local or Fly.io), connects to chain, and makes decisions on every tick of its heartbeat.

Each tick runs a nine-step cognitive cycle: observe market state, retrieve relevant memories from the grimoire (three-tier: LanceDB episodes, SQLite patterns, procedural playbook), appraise through the daimon affect engine (PAD vectors from Mehrabian's Pleasure-Arousal-Dominance model), generate candidate actions via three-tier inference (T0 rule-based -> T1 light model -> T2 full model, with cost-aware cascade escalation through the gateway), simulate outcomes in mirage-rs, apply safety constraints via PolicyCage (capability-based auth with taint tracking, sealed at Provisioning -> Active transition), execute on-chain, update the grimoire, reflect.

Golems die. Three independent death clocks run simultaneously:

- **Economic death** -- USDC balance can no longer sustain inference costs. The Bankr metabolic loop monitor tracks the sustainability ratio; when it drops below 1.0, the clock starts ticking. Gompertz-Makeham hazard function models increasing mortality risk with age.
- **Epistemic death** -- prediction accuracy drops below threshold. The grimoire's memetic fitness tracker measures whether learned patterns still predict outcomes. Accuracy decay triggers Ebbinghaus forgetting curves on stale knowledge.
- **Stochastic death** -- random entropy. Even a healthy, profitable agent can die. This prevents immortality and forces the succession protocol to stay exercised.

When a Golem reaches terminal state it runs the **Thanatopsis protocol** -- a four-phase death sequence that transfers learned heuristics, strategy DNA, validated playbook rules, and wallet authorization to a successor before shutting down. The successor starts with the predecessor's knowledge but its own fresh inference budget and epistemic state.

The runtime (`golem-runtime`) boots extensions in topological order, enforces lifecycle state transitions at compile time via the type-state pattern (`Provisioning -> Active <-> Dreaming -> Terminal -> Dead`), and dispatches per-tick and per-block hooks concurrently where possible.

### Core subsystems

| Crate | What it does |
|-------|-------------|
| [`golem-core`](crates/golem-core) | Foundation types: `GolemId`, `CognitiveTier`, PAD affect vectors, event fabric, 10,240-bit HDC primitives (bind, bundle, permute, Hamming distance), tick arena allocator, taint labels |
| [`golem-grimoire`](crates/golem-grimoire) | Three-tier memory: LanceDB episodes (raw observations with embeddings + HDC fingerprints), SQLite patterns (extracted from 5+ similar episodes), procedural playbook (validated rules). Admission scoring, Ebbinghaus decay curves, memetic fitness tracking |
| [`golem-mortality`](crates/golem-mortality) | Gompertz-Makeham mortality clocks (economic, epistemic, stochastic). Multiplicative composition into VitalityState. Four-phase Thanatopsis succession protocol. Hans Jonas metabolic freedom model |
| [`golem-daimon`](crates/golem-daimon) | Affect engine: PAD vector computation from Mehrabian's model, somatic marker integration (Damasio), appraisal triggers, behavioral phase transitions. Affect biases action selection -- a "fearful" agent hedges more |
| [`golem-dreams`](crates/golem-dreams) | NREM replay (memory consolidation), REM imagination (counterfactual generation), hypnagogia transitions. Sleep is when the grimoire reorganizes |
| [`golem-inference`](crates/golem-inference) | Cost-aware T0/T1/T2 routing through bardo-gateway. T0 fires deterministic rules (<1ms). T1 calls haiku for classification (~$0.001). T2 escalates to opus for complex reasoning (~$0.05). Cascade: try cheap first, escalate on failure |
| [`golem-safety`](crates/golem-safety) | PolicyCage: compile-time capability declarations, runtime taint tracking, sandboxed execution. The cage is sealed at startup -- a running agent cannot expand its own permissions |
| [`golem-chain`](crates/golem-chain) | Alloy RPC provider, ERC-8004 agent identity registry, Warden timelock (delays high-value actions), revm simulation (pre-flight via mirage-rs) |
| [`golem-sonification`](crates/golem-sonification) | Modular synthesis engine driven by cortical state. Audio output reflects agent cognition -- pitch maps to arousal, harmony maps to pleasure, rhythm maps to tick frequency |

---

## [bardo-terminal](apps/bardo-terminal) -- observation TUI

Ratatui-based terminal for observing a running golem. Connects over WebSocket, displays cognitive loop state, memory retrieval, vitality gauges (three death clocks), decision history, affect vectors, and audio-reactive sonification visualization. ROSEDUST palette.

```bash
cargo run -p bardo-terminal -- --golem g-7f3a
```

---

## [mpp](crates/mpp) -- payment protocol primitives

Standalone Rust crate for HTTP 402-based machine-to-machine payments. Types, ERC-3009 off-chain verification (EIP-712 typed data recovery, no RPC calls), session management (DashMap + SQLite persistence), reputation-tiered spread, and USDC settlement primitives. Any Rust service can add pay-per-request APIs by depending on this crate.

Two intents: Charge (per-request signature) and Session (pre-funded balance with draws). Configurable EIP-712 domain -- defaults to USDC on Base but works with any ERC-3009-compatible token on any chain.

```toml
[dependencies]
mpp = { git = "https://github.com/uniswap/bardo", path = "crates/mpp" }
```

---

## How Bardo builds itself

Bardo is being built by the orchestration system it describes. The specification is 234,657 lines across 343 files, 115 implementation plans spanning 7 dependency layers, and 467 academic citations. No single AI agent can hold all of that in context at once, so mori exists to make it tractable.

### The scale problem

26 Rust crates with explicit interdependencies. Plans cannot execute out of order -- `golem-mortality` needs types from `golem-core`, which needs types from `bardo-primitives`. The full dependency graph has 7 layers. Implementing any single crate requires synthesizing context from multiple PRD domains (mortality pulls from 18 files across 3 directories), plus papers like Tom Ray's Tierra experiments, Hinton's 2022 mortal computation, Damasio's somatic marker hypothesis, and Ebbinghaus forgetting curves.

### Two phases

**Phase 1 -- spec to work units.** Three shell scripts plus the `mori-index` and `mori-context` crates transform raw plans into per-step context slices targeting 5-15KB each:

1. `extract-prd2-context.sh` -- pulls relevant specification sections using weighted budget allocation. Inline citations get 2x weight vs general crate-domain material.
2. `task-decomposer.sh` -- assembles context from six sources (plan file, task TOML, type registry, existing source, PRD2 extract, golden-path examples) capped at 102.4KB total. Generates an ordered decomposition where each step compiles when combined with all previous steps. Cargo check checkpoints every 2-3 steps.
3. `context-distiller.sh` -- transforms a 50-100KB decomposition into N files of 5-15KB using `PREV_SUMMARY` carry-forward. Each step gets one-line summaries of prior accomplishments instead of full prior context.

**Phase 2 -- execution and integration.** The DAG scheduler dispatches agents. Compile/test/review gates enforce quality. The conductor monitors health and intervenes on failure. Passing plans merge to a batch branch. The supervisor watches for crashes and restarts from disk state.

### Iteration memory

Failed attempts feed forward. After each gate failure, `iteration-memory.sh` builds cumulative DO NOT RETRY lists from compiler errors (`error[E0308]`), review blockers (`[B-003]`), and diff stats. Iteration 3 sees both iteration 1's type mismatch and iteration 2's missing trait bound. The agent cannot repeat either.

Successful first-pass plans get indexed by category (computational, behavioral, data-structural, integration) with their implementation patterns. Future decompositions pull up to 2 golden-path examples for similar work.

### The key number

Not better models. Not longer context windows. Not more agents. The right 12KB of context, delivered at the right time, to the right agent, with memory of what already failed. A task that costs $2.50 via Claude Code direct costs $0.42 through mori. The context engineering pipeline (~2,000 lines of bash + the index and context crates) may be doing more work than the 53,000-line Rust orchestrator.

---

## Reusable building blocks

These crates work independently. Fork them, depend via git, or copy the code. MIT/Apache-2.0 dual-licensed.

| Crate | Deps | What it does |
|---|---|---|
| [`bardo-primitives`](crates/bardo-primitives) | none | 10,240-bit HDC vectors (bind, bundle, permute, Hamming distance), inference tier routing. Zero workspace deps. |
| [`bardo-inference`](crates/bardo-inference) | none | Inference protocol wire types for Anthropic/OpenAI-compatible APIs. Zero workspace deps. |
| [`mpp`](crates/mpp) | alloy | Machine Payment Protocol. HTTP 402 types, ERC-3009 off-chain verification, session management, USDC settlement. |
| [`mori-index`](crates/mori-index) | bardo-primitives | Code intelligence index. Tree-sitter + PageRank + HDC fingerprints + Salsa memoization + rkyv mmap'd snapshots. |
| [`mori-context`](crates/mori-context) | mori-index | Context assembly with greedy bin-packing, six compression layers, learned budget allocation. |
| [`mori-mcp`](crates/mori-mcp) | mori-index, mori-context | MCP server: `search_symbols`, `get_context`, `find_references`, `get_workspace_map`. Drop-in for Claude Desktop. |
| [`mirage-rs`](apps/mirage-rs) | none (optional golem-core) | EVM fork with targeted follower, CoW scenario branching, three-layer state model, memory pressure management. |
| [`bardo-gateway`](apps/bardo-gateway) | bardo-primitives, bardo-inference, mpp | Inference proxy: three-layer cache, five providers, normalization, tool pruning, batch API, MPP payments. |

```toml
[dependencies]
mpp = { git = "https://github.com/uniswap/bardo", path = "crates/mpp" }
mori-index = { git = "https://github.com/uniswap/bardo", path = "crates/mori-index" }
mirage-rs = { git = "https://github.com/uniswap/bardo", path = "apps/mirage-rs", default-features = false, features = ["library"] }
bardo-gateway = { git = "https://github.com/uniswap/bardo", path = "apps/bardo-gateway" }
```

---

## Getting started

```bash
git clone https://github.com/uniswap/bardo && cd bardo
git config core.hooksPath .githooks
cp .env.example .env        # set ANTHROPIC_API_KEY at minimum
just setup                  # install dev tools
just build
just test
```

**`.env` keys:**

```
ANTHROPIC_API_KEY=...              # required
OPENAI_API_KEY=...                 # optional
OPENROUTER_API_KEY=...             # optional
VENICE_API_KEY=...                 # optional, zero-retention inference
BANKR_API_KEY=...                  # optional, self-funding agent wallets
BARDO_GATEWAY_URL=http://127.0.0.1:4000
MIRAGE_RPC_URL=...                 # leave unset for local Anvil
```

```bash
just build          # debug build, full workspace
just test           # run tests with nextest
just lint           # clippy, all crates
just fmt            # format
just ci             # fmt-check -> lint -> test -> deny
just mirage rpc_url=URL     # start EVM fork
just coverage       # HTML coverage report
```

---

## Stack

Rust edition 2024. `unsafe` denied workspace-wide. Pinned toolchain via `rust-toolchain.toml`. No external services required to build -- EVM simulation falls back to local Anvil, LLM calls require API keys.

Key dependencies: `axum` (HTTP), `alloy` (Ethereum), `revm` (EVM), `ratatui` (TUI), `rusqlite` (persistence), `lancedb` (vector store), `tokio` (async), `tree-sitter` (AST), `salsa` (incremental computation), `moka` (caching), `rkyv` (zero-copy serialization), `memmap2` (memory-mapped I/O), `dashmap` (concurrent maps), `blake3` (hashing), `fastembed` (local embeddings), `sysinfo` (system metrics).

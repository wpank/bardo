# Bardo

Permissionless infrastructure for mortal autonomous agents in DeFi.

## What is this

Bardo is a Rust monorepo for building autonomous DeFi agents that are designed to die. Each agent — called a golem — runs a 9-step cognitive loop every tick: observe the market, retrieve relevant memory, analyze conditions, gate the decision, simulate the outcome, validate safety constraints, execute on-chain, verify the result, reflect on what happened. It burns USDC as metabolic substrate. When the balance hits zero, when its world model drifts past recovery, or when a stochastic mortality draw fires, it dies.

Death is the mechanism, not a failure mode. At death the golem runs Thanatopsis: compress its knowledge store, push the inheritance package to its clade, leave a death mask on-chain. The successor inherits compressed judgment. Across generations, the population accumulates knowledge that no immortal agent can develop — distilled under survival pressure, validated by prediction accuracy, pruned by mortality.

The workspace is 27 library crates organized into 7 dependency layers, 7 application binaries, and a build orchestration system called Mori. Written in Rust (edition 2024), no `unsafe` anywhere in the workspace.

## Research Foundations

Bardo draws on published research across cognitive science, machine learning, distributed systems, and computational finance. This isn't a wrapper around an LLM — it's a cognitive architecture with formal models for memory, emotion, mortality, and decision-making.

### Cognitive Architecture: CoALA Pipeline

The core reasoning loop implements a 9-step pipeline inspired by the Cognitive Architecture with Linear Attention framework (CoALA, Princeton, TMLR 2024). Every tick follows a strict sequence:

```
Observe → Retrieve → Analyze → Gate → Simulate → Validate → Execute → Verify → Reflect
```

No steps are skipped. The Gate phase uses adaptive tier selection: T0 (pure heuristics, no model call), T1 (lightweight model), or T2 (full workspace reasoning). This draws on RouteLLM (ICLR 2025, UC Berkeley) for cost-aware routing — achieving >85% cost reduction while maintaining 95% performance quality. The routing layer uses Thompson Sampling with Beta posteriors for exploration-exploitation balance across model tiers.

**Implementation:** `crates/golem-heartbeat`

### Hyperdimensional Computing

Pattern matching across the system uses Binary Sparse Distributed Representations — 10,240-bit vectors with XOR binding and Hamming distance comparison. Where neural embeddings take 10-50ms per comparison, HDC fingerprints resolve in ~50ns using SIMD-accelerated POPCNT operations.

HDC is used for:
- **Transaction encoding** — Thermometer bundle encoding of on-chain transaction fields into binary fingerprints for anomaly detection
- **Task fingerprinting** — Matching new build tasks to historical patterns in the Grimoire
- **Code structure similarity** — Structural fingerprints of AST nodes for codebase search
- **Cross-project learning** — Transferring patterns between codebases via role-filler binding

The system maintains a codebook of prototype vectors per transaction category. New transactions are encoded into HDC space, compared against the codebook via Hamming distance, and routed based on similarity thresholds. Bloom filters pre-screen candidates to skip ~70% of distance computations.

**Implementation:** `crates/golem-core::hdc`, `crates/golem-triage::HdcTxEncoder`, `crates/mori-index`

### Mortality System

Golems die. The mortality system runs three independent clocks, and whichever reaches zero first triggers succession:

**Economic Clock** — Cumulative USDC burn. Hard limit on operational cost.

**Epistemic Clock** — The primary death mechanism. Tracks prediction accuracy across five weighted dimensions:
- Price direction (weight 0.35)
- Volatility regime (0.25)
- Yield trend (0.20)
- Gas conditions (0.10)
- Protocol state (0.10)

Accuracy is tracked via Exponential Moving Average with domain-specific decay constants. Price predictions use α=0.02 (~35 tick half-life, fast adaptation). Protocol state uses α=0.001 (~693 tick half-life, slow drift). When composite fitness drops below 0.35, the golem enters senescence. Recovery requires crossing 0.45 (hysteresis band prevents state chatter). Below recovery threshold, the world model has drifted — a fresh successor with inherited knowledge outperforms a drifted incumbent.

**Stochastic Clock** — Random mortality draw introducing population diversity. Prevents stagnation even in well-performing agents.

The vitality FSM transitions through five phases: Thriving → Competent → Declining → Senescent → Terminal. Each phase adjusts cost multipliers, concurrency limits, model tier selection, and deliberation thresholds. As an agent approaches death, it dreams more frequently (consolidating knowledge before Thanatopsis).

**Implementation:** `crates/golem-mortality`

### Memory Architecture: The Grimoire

The knowledge store implements a three-tier memory hierarchy drawn from cognitive science episodic/semantic/procedural memory models (CoALA, TMLR 2024):

**Tier 1 — Episodes** (LanceDB vector store). Raw execution observations indexed by timestamp and HDC fingerprint. Immutable once written. 768-dimensional embeddings via nomic-embed-text-v1.5 (Matryoshka).

**Tier 2 — Patterns** (SQLite semantic store). Extracted knowledge crystallized when 5+ similar episodes converge. Six entry categories: Insight (declarative), Heuristic (prescriptive), Warning (immune system), CausalLink (structural), StrategyFragment (speculative), AntiKnowledge (permanent negative knowledge with 0.3 confidence floor and 0.5x decay).

**Tier 3 — Playbook**. Validated behavioral rules promoted from patterns after 5+ successful applications. Transaction-logged updates with confidence scoring.

Retrieval uses four factors simultaneously: Temporal (recency), Semantic (relevance via embedding similarity), Emotional (affective congruence with current PAD state), Causal (dependency chain traversal). Results are composed via deterministic ranking.

New entries pass through an A-MAC five-factor admission gate (Zhang et al., arXiv:2603.04549, March 2026): future utility (0.25), factual confidence (0.25), semantic novelty (0.20), temporal recency (0.15), content type prior (0.15). Entries scoring below 0.45 are rejected. The gate prevents knowledge store pollution.

Knowledge decays. Confidence erodes over time at rates determined by decay class (Structural entries never decay; Ephemeral observations decay rapidly). The memetic fitness system tracks fidelity, fecundity, fitness, and parasite score per entry — entries that spread without validation accumulate parasite score and are eventually pruned.

**Implementation:** `crates/golem-grimoire` (admission, decay, entry, error, memetic, retrieval, substrate/episodic, substrate/semantic, substrate/playbook)

### Emotion and Affect: The Daimon

Agent emotional state is modeled in continuous 3D PAD space (Mehrabian's Pleasure-Arousal-Dominance model). Each dimension ranges [-1.0, 1.0]. Market events, survival pressure, and prediction outcomes drive PAD vector updates through an EMA filter.

PAD vectors map to discrete Plutchik emotions via octant classification from sign bits — 8 primary emotions (Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation) plus neutral. This bridges continuous affect space to human-interpretable emotion labels.

The emotion system feeds into decision-making:
- **Appraisal theory** (Ortony-Clore-Collins) computes goal-based emotional responses
- **Scherer's component process model** determines emotional intensity
- **Somatic markers** (Damasio's hypothesis) correlate physiological proxies with PAD state
- **Clade contagion** propagates emotional signals between agents with 0.8 attenuation per hop

Emotional state gates dream scheduling — high arousal produces shallow, wide counterfactual exploration; low arousal enables deep, narrow scenario generation. Emotional tags on Grimoire entries enable affective memory retrieval.

**Implementation:** `crates/golem-daimon`, `crates/golem-core::cortical`

### Dream Consolidation

Golems dream. The dream system runs two phases borrowed from sleep neuroscience:

**NREM Replay** — Reviews Grimoire memories, strengthens high-value entries, identifies gaps. Consolidation rate increases as epistemic fitness degrades (more frequent consolidation near death). Idempotent — repeating the same consolidation produces the same result.

**REM Imagination** — Generates counterfactual scenarios. Predicted futures are weighted by emotional arc: fear-dominant states explore downside scenarios, joy-dominant states explore upside. Depth is capped by emotional load. Dream outputs update PLAYBOOK entries with timestamp, origin dream ID, and confidence score (never exceeding source confidence).

Dream scheduling is driven by the VitalityState FSM and current PAD vector. Approaching death triggers more dreams. NREM and REM phases are mutually exclusive.

**Implementation:** `crates/golem-dreams`

### Bayesian Event Triage

Incoming blockchain events pass through a multi-stage anomaly detection pipeline before reaching the cognitive loop:

- **Bayesian Surprise** — Conjugate Gamma-Poisson models for rate anomalies, Normal-Inverse-Gamma for value distribution shifts. Events are scored by KL divergence against learned beliefs.
- **Bayesian Online Changepoint Detection (BOCPD)** — Run-length posteriors for detecting regime transitions in streaming data.
- **MidasR** — Streaming graph anomaly detection on dynamic transaction graphs via edge frequency and chi-squared baseline comparison.
- **Count-Min Sketch** — Conservative-update streaming frequency estimation for transaction pattern tracking.
- **Thompson Routing** — Beta posteriors per routing bucket determine which events warrant expensive model calls vs. heuristic handling.
- **HNSW Approximate Nearest Neighbor** — Fast fingerprint similarity search against the HDC codebook.

The pipeline produces curiosity scores and routing decisions (T0/T1/T2 tier selection) without requiring any model calls for the triage itself.

**Implementation:** `crates/golem-triage`

### Topological Data Analysis

Market regime detection uses persistent homology from algebraic topology:

- **Persistence diagrams** capture topological features (connected components, loops) that persist across multiple scales in price data
- **Betti curves** track the count of topological features over time
- **Regime classification** with hysteresis: trending, volatile, range-bound, transition

TDA detects structural market changes that moving averages and volatility measures miss — it captures the shape of market dynamics, not just summary statistics. Regime signals feed into the epistemic clock (a regime change invalidates predictions) and dream scheduling (regime transitions trigger consolidation).

**Implementation:** `crates/golem-ta`

### Reflexion and Self-Improvement

The reflect step in the cognitive loop implements Reflexion (Shinn et al., NeurIPS 2023, arXiv:2303.11366). After each execution:

1. Generate trajectory of what happened
2. Evaluate against expected outcome
3. Generate verbal reflection on the gap
4. Store reflection in episodic memory
5. Retry with reflection context on next occurrence

This achieves 91% pass@1 on HumanEval vs. 80% for GPT-4 baseline. Combined with HDC pattern pre-checking, known failure patterns are caught before execution.

The Skill Library follows the Voyager pattern (arXiv:2305.16291) — successful task solutions are stored as reusable programs indexed by embedding, verified before addition, and composed for complex tasks.

### Context Engineering

Agent context assembly uses deterministic methods over LLM-based approaches wherever possible:

- **Tree-sitter AST extraction** — Syntactic boundary preservation in code retrieval. +5.5 points on RepoEval vs. naive line-based splitting (cAST paper).
- **PageRank symbol graphs** — Ranks code symbols by cross-file reference count for importance-weighted context assembly.
- **Salsa incremental computation** — Query-based memoization, 96% cache hit rate. Only re-analyzes modules that changed.
- **Memory-mapped index loading** — Zero-copy deserialization via mmap. <1ms load vs. 100-500ms heap deserialization.
- **LLMLingua-2 compression** — 3-6x faster context compression, up to 20x token reduction with 1.5% quality loss.
- **Merkle tree change detection** — Tracks codebase state at symbol granularity for incremental updates.

The system learns optimal context budget allocation per task type. After 100+ builds, auth tasks converge to 30% playbook context; config tasks converge to 45% state context. Overall: 76% fewer input tokens, 91% prompt prefix cache hit rate, 94% context relevance (vs. 62% for grep-based retrieval).

**Implementation:** `crates/mori-index`, `crates/mori-context`, `crates/golem-context`

### Inference Routing and Cost Optimization

The inference gateway implements a three-layer caching strategy:

1. **Hash cache** — Exact prompt match
2. **Semantic cache** — 61.6-68.8% hit rate at cosine threshold 0.8 (GPT Semantic Cache, 2024)
3. **Prompt prefix cache** — Anthropic's 90% savings on cached prefix tokens

Model selection uses cascading escalation: start with the cheapest model, escalate on gate failure. Budget pressure smoothly adjusts routing via a single `budget_remaining / budget_total` ratio — no hard phase transitions. The sharp cutover at vitality 0.3 switches T2 between Opus and Sonnet. Cascade routing draws on ETH Zurich's ICML 2025 work and LLM Shepherding (2025) where expensive model hints guide cheap model execution.

**Implementation:** `apps/bardo-gateway`, `crates/golem-inference`

### Inter-Agent Coordination

Golems coordinate through stigmergic signaling — inspired by ant colony communication:

- **Three signal layers:** Threat, Opportunity, Wisdom
- **Pheromone field propagation** with configurable policies per signal type
- **Clade membership** defines cooperative groups sharing knowledge through Styx
- **Bloodstain signals** — dying golems broadcast compressed knowledge markers to nearby agents
- **Quality gates** prevent noise: shared insights start at 0.3 confidence and must be validated independently

**Implementation:** `crates/golem-coordination`

### On-Chain Identity and Payments

- **ERC-8004** — On-chain agent registration with reputation tracking. Each golem gets a unique Agent Card NFT containing capabilities, service endpoints, and payment address. 85K+ agents registered across Ethereum, Base, and BNB Chain.
- **x402** — HTTP-native micropayments via HTTP 402. Agents pay per request in USDC. 100M+ payments processed, $10M+ volume.
- **MPP (Machine Payments Protocol)** — Session-based streaming payments for long-running operations. Pre-fund, stream via off-chain vouchers, periodic on-chain settlement. Sub-100ms latency.
- **ERC-8183** — Trustless job escrow. Client escrows funds, provider delivers, evaluator (test suite) confirms quality.

**Implementation:** `crates/golem-chain`, `apps/bardo-gateway`

### Security Model

- **Capability-based authorization** — Type-parameterized unforgeable security tokens `Capability<T>`
- **Taint tracking** — `TaintedString` carries data provenance through transformations, preventing injection
- **PolicyCage sandboxing** — Actions execute within constrained policy envelopes
- **Merkle audit log** — Append-only tamper-evident log of all privileged operations
- **Loop recursion guards** — Prevent unbounded recursive agent calls
- **Revm simulation** — All on-chain actions simulated in a local EVM fork before broadcast

**Implementation:** `crates/golem-safety`, `apps/mirage-rs`

### Performance Infrastructure

- **TickArena allocator** — Bump allocator with tick-boundary reset. O(1) cleanup, zero per-object deallocation overhead. 10-40x memory reduction vs. GC-based runtimes.
- **SIMD Hamming distance** — Vectorized XOR + POPCNT on u64 words. 200-1000x faster than float cosine distance for HDC comparisons.
- **Salsa memoization** — Incremental computation framework. 96% cache hit rate for AST re-analysis.
- **Memory-mapped indexes** — Zero-copy mmap loading. <1ms startup, <5MB per agent vs. 50-200MB heap.
- **Bloom filter pre-screening** — Skip ~70% of HDC distance computations via probabilistic negative lookup.

## Architecture

```
golem-binary  (single VM binary)
  └── golem-runtime  (extension registry, lifecycle FSM)
        ├── golem-heartbeat  (9-step CoALA pipeline)
        │     ├── golem-context  (CognitiveWorkspace assembly)
        │     │     ├── golem-grimoire  (LanceDB + SQLite + PLAYBOOK)
        │     │     ├── golem-daimon  (PAD affect engine)
        │     │     └── golem-core  [foundation]
        │     ├── golem-safety  (Capability<T>, PolicyCage, audit log)
        │     ├── golem-tools  (423+ DeFi tools, revm simulation)
        │     ├── golem-inference  (T0/T1/T2 routing, x402)
        │     └── golem-core
        ├── golem-mortality  (three clocks, thanatopsis)
        ├── golem-dreams  (NREM/REM consolidation)
        ├── golem-coordination  (pheromone field, clade sync)
        ├── golem-chain  (Alloy, ERC-8004, revm)
        ├── golem-chain-intelligence  (block ingestion, PVS)
        ├── golem-triage  (Bayesian surprise, HDC, BOCPD)
        ├── golem-ta  (TDA, persistent homology)
        ├── golem-surfaces  (WebSocket, SSE, Telegram)
        ├── golem-creature  (procedural identity engine)
        ├── golem-engagement  (achievements, graveyard)
        ├── golem-sonification  (cortical → audio mapping)
        └── golem-core  [zero workspace deps]
```

## Workspace Layout

### Foundation

- **`bardo-primitives`** — Pure compute primitives, zero internal dependencies. HDC vector types.
- **`bardo-inference`** — Inference protocol types shared between gateway and golem-inference.

### Golem Runtime (Layers 0-7)

**Layer 0** — `golem-core`: GolemId, CognitiveTier, PadVector, BehavioralPhase, PlutchikEmotion, TickArena allocator, HDC primitives, taint labels, event fabric, extension trait skeleton.

**Layer 1** — `golem-runtime`: Extension registry with topological dependency sort, hook dispatch, lifecycle FSM (Provisioning → Active → Dreaming → Terminal → Dead). Type-state pattern enforces valid transitions at compile time.

**Layer 2** — Core subsystems:
- `golem-heartbeat` — 9-step CoALA decision pipeline
- `golem-grimoire` — Three-tier knowledge store with four-factor retrieval, A-MAC admission, memetic fitness tracking
- `golem-daimon` — PAD affect engine, OCC appraisal, somatic markers, clade emotional contagion
- `golem-mortality` — Triple-clock mortality, EMA fitness, senescence with hysteresis, Thanatopsis protocol
- `golem-dreams` — NREM replay, REM counterfactual generation, PLAYBOOK evolution
- `golem-context` — CognitiveWorkspace assembly, learned token budget allocation

**Layer 3** — `golem-safety`: Capability-based auth, PolicyCage, taint tracking, Merkle audit log, loop guards.

**Layer 4** — Decision engines:
- `golem-inference` — Three-tier routing with Thompson Sampling, cascading model selection
- `golem-chain` — Alloy RPC, ERC-8004 identity, Permit2, revm simulation
- `golem-chain-intelligence` — Block ingestion, protocol state snapshots
- `golem-triage` — Bayesian surprise, BOCPD, MidasR, HDC encoding, Count-Min Sketch, Thompson routing
- `golem-ta` — Persistent homology, Betti curves, regime detection with hysteresis
- `golem-oneirography` — Dream journaling, death reflections, content-addressed lineage DAG
- `golem-tools` — 423+ DeFi tool definitions across Uniswap, Aave, Morpho, Pendle, Lido, EigenLayer, GMX. Revm simulation, capability-gated execution, circuit breakers.

**Layer 5** — `golem-coordination`: Stigmergic pheromone field signaling (Threat/Opportunity/Wisdom layers), clade sync, bloodstain inheritance.

**Layer 6** — Presentation:
- `golem-surfaces` — WebSocket, SSE, Telegram push connector
- `golem-creature` — Procedural sprite generation from wallet seed. Orbital physics, spring constants, PAD-driven animation. Evolution forms: Egg → Hatchling → Mature → Weathered → Transcendent.
- `golem-engagement` — Achievement tracking, death recaps, graveyard genealogy
- `golem-sonification` — Cortical signal → audio parameter mapping. PAD drives oscillator frequency/amplitude, vitality drives envelope, behavioral phase drives effects.

**Layer 7** — `golem-binary`: Single compiled executable.

### Applications

- **`bardo-gateway`** — HTTP inference proxy with three-layer caching, MPP streaming payments, multi-provider routing, cost tracking. Axum on port 4000.
- **`bardo-terminal`** — TUI for golem observation. Ratatui rendering, WebSocket event stream, ROSEDUST design system (30+ color tokens), procedural sprite system, optional audio.
- **`bardo-styx`** — Knowledge relay with three privacy layers: Vault (private), Clade (shared), Lethe (public-anonymized).
- **`bardo-compute`** — Batch compute orchestration.
- **`mirage-rs`** — EVM state simulator. Fork-state management, speculative transaction execution without broadcast.

### Mori (Build Orchestration)

Mori is a multi-agent build system that applies the same cognitive architecture to software construction.

- **`mori-index`** — AST-based code index with HDC fingerprinting, PageRank symbol ranking, tree-sitter parsing, Salsa incremental computation, optional mmap'd snapshot loading.
- **`mori-context`** — Deterministic context assembly for agent-driven builds.
- **`mori-mcp`** — MCP server exposing code search and context tools to Claude and other agents.
- **`mori`** — Orchestrator CLI/TUI. DAG scheduler, parallel agent dispatch, gate verification, budget tracking.
- **`mori-service`** — Service daemon with HTTP API, event streaming, webhook integration.

### Test Infrastructure

- **`tests/harness`** — Shared fixtures, golem state mocking, provider setup, simulation helpers.

## Prerequisites

- **Rust 1.85+** — Pinned via `rust-toolchain.toml` (pulls rustfmt, clippy, llvm-tools-preview)
- **[just](https://github.com/casey/just)** — Task runner
- **Foundry** — For local EVM development (only needed for chain-facing code without `MIRAGE_RPC_URL`)

Optional:
- `cargo-nextest` — Parallel test runner (required by `just test`)
- `sccache` — Compile cache
- `mdbook` — Documentation site builder

## Getting Started

```bash
git clone <repo-url> && cd bardo
git config core.hooksPath .githooks
cp .env.example .env   # Set ANTHROPIC_API_KEY at minimum
just setup              # Install dev tools
just build              # Debug build
just test               # Run tests with nextest
```

## Development

```bash
just build              # Debug build, full workspace
just build-release      # Release build of golem-binary
just test               # All tests with nextest
just lint               # Clippy on all crates, all features
just fmt                # Format all code
just watch              # Continuous clippy via bacon
just docs               # Build and open rustdoc
just mdbook             # Build mdbook site
just ci                 # Full CI: fmt-check → lint → test → deny
just audit              # Deep audit: fmt, lint, deny, unused deps, TOML
just mirage rpc_url=URL # Start local EVM fork
just coverage           # HTML coverage report
```

## Configuration

Copy `.env.example` to `.env`. The supervisor and mori scripts auto-source it at startup.

- **LLM keys** — `ANTHROPIC_API_KEY` required. Optional backup keys (`_2`, `_3`) for rate-limit failover. `OPENAI_API_KEY` and `OPENROUTER_API_KEY` are optional.
- **Gateway** — `BARDO_GATEWAY_URL` defaults to `http://127.0.0.1:4000`.
- **RPC** — Leave `MIRAGE_RPC_URL` unset for local Anvil. Set to mainnet RPC for fork-mode. `MIRAGE_FORK_BLOCK` pins to a specific block.
- **Timeouts** — `CODEX_TIMEOUT` (default 3600s), `CARGO_TEST_TIMEOUT` (default 900s).

## Toolchain Policy

Rust edition 2024, MSRV 1.85. `unsafe_code = "deny"` workspace-wide. Clippy pedantic and nursery lints as warnings. `unwrap_used = "deny"`. Release profile: thin LTO, single codegen unit, symbols stripped, panic = abort. Dependency auditing via `cargo-deny`.

## References

Selected academic foundations (full citations in PRD documents):

| Area | Reference |
|------|-----------|
| Cognitive architecture | CoALA (Princeton, TMLR 2024) |
| Self-improvement | Reflexion (NeurIPS 2023, arXiv:2303.11366) |
| Model routing | RouteLLM (ICLR 2025, UC Berkeley); Cascade Routing (ETH Zurich, ICML 2025) |
| Knowledge admission | A-MAC (Zhang et al., arXiv:2603.04549) |
| Emotion model | PAD space (Mehrabian); Plutchik's psychoevolutionary theory; OCC appraisal (Ortony-Clore-Collins 1988); Scherer component process model |
| Mortality | Somatic marker hypothesis (Damasio) |
| Skill accumulation | Voyager (arXiv:2305.16291) |
| Code retrieval | cAST; CodexGraph (arXiv:2408.03910); LLMDFA (NeurIPS 2024) |
| Context compression | LLMLingua-2 |
| Caching | GPT Semantic Cache (2024); SCALM (2024) |
| Agent architecture | Agentless (arXiv:2407.01489); SICA (ICLR 2025 Workshop) |
| Anomaly detection | MidasR; BOCPD |
| Topology | Persistent homology, Betti curves |

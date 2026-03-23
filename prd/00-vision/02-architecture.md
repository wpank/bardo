# Five-Layer Architecture [SPEC]

**Version:** 2.1.0
**Last Updated:** 2026-03-17

> **Reader orientation:** This is the architecture overview for Bardo's five-layer system. It covers the Golem (mortal autonomous agent) runtime in detail -- the extension system, adaptive clock, CorticalState, cognitive workspace, and the 18-crate Rust workspace -- plus the vault, compute, reputation, and interface layers. This belongs to the `00-vision/` foundation layer and is the most technically detailed of the vision documents. Read `01-thesis.md` first for the philosophical motivation. `prd2/shared/glossary.md` has full term definitions.

---

## Overview

Bardo is permissionless infrastructure for autonomous agents managing capital markets, integrated with leading DeFi protocols including Uniswap, Aave, Morpho, Pendle, and Aerodrome. Five layers compose the system, each independently useful but mutually reinforcing. The architecture follows Ashby's Law of Requisite Variety [ASHBY-1956]: the regulator (Bardo) must match the variety of the regulated system (DeFi).

```
Owner / Agent Intent
       |
       v
Interfaces (Bott, CLI/TUI, Portal, API)
       |  Parse intent, deploy golems
       v
Golems (Golem-RS, 28 extensions, 7 layers)
       |  9-step heartbeat, Grimoire learning, Daimon affect, mortality
       v
Bardo Tools (~210 DeFi tools, Alloy-native + TS sidecar)
       |  On-chain reads/writes, safety middleware, vault tools
       v
Styx (knowledge service: backup, clade relay, pheromone field)
       |  Three privacy layers (Vault/Clade/Lethe (formerly Commons)), x402 billing
       v
Bardo Compute (x402-gated VMs on Fly.io) / Self-Deploy / Bare Metal
       |  Dedicated agent runtime, warm pool, TTL enforcement
       v
ERC-4626 Vaults + ERC-8004 Identity + Uniswap (V2/V3/V4/UniswapX)
```

---

## Layer 1: Vaults (Capital Custody)

ERC-4626 tokenized vaults on Base via AgentVaultFactory. Single asset (USDC). Permit2 integration. Optional ERC-8004 identity gating for deposits (withdrawals never gated). FeeModule with immutable fee caps (500 bps management, 5000 bps performance). Virtual shares offset of 6 decimals prevents inflation attacks. NAVAwareHook creates a secondary market for vault shares on Uniswap V4.

**Core contracts**: AgentVaultFactory (CREATE2 deployment + registry), AgentVaultCore (ERC-4626 + ERC-8004 gate), FeeModule (management/performance fees with high-water mark), VaultHook (V4 agent-gated swaps + dynamic fees), NAVAwareHook (V4 NAV-priced shares for instant exit), OnboardRouter (atomic register + Permit2 + deposit).

**Philosophy**: Vaults are the Golem's on-chain body schema in Merleau-Ponty's sense [MERLEAU-PONTY-1945] -- not an external container but the medium through which the agent has a world. The vault balance is the agent's facticity [HEIDEGGER-1927]: the concrete situation it cannot escape.

See `prd2/08-vault/` (ERC-4626 vault contracts, SDK, adapters, fee module, NAV-aware hooks, and testing) for full vault specifications.

---

## Layer 2: Golems (Autonomous Agents)

Golems are autonomous agents built on Golem-RS, a Rust runtime forked from `pi_agent_rust` (Dicklesworthstone's Rust port of the Pi Agent framework). Language: Rust (edition 2024), `#![forbid(unsafe_code)]`, single Cargo workspace. Each Golem compiles to a single static binary that runs on an ephemeral Fly.io VM, a self-hosted machine, or bare metal.

### Golem-RS Runtime

Five architectural capabilities define the runtime:

1. **Extension system**: 20 lifecycle hooks, 28 extensions organized into a 7-layer dependency DAG. Composition over inheritance -- each subsystem implements only the hooks it needs.
2. **9-step heartbeat pipeline**: observe → retrieve → analyze → gate → simulate → validate → execute → verify → reflect. Each tick produces a `DecisionCycleRecord` -- a structured snapshot of everything that happened.
3. **Three cognitive tiers**: T0 (suppress, no LLM, $0 -- handles ~80% of ticks), T1 (analyze, Haiku-class), T2 (deliberate, Opus-class). Routing based on prediction-error gating with an adaptive threshold (Friston 2010).
4. **Event Fabric + CorticalState**: The Event Fabric is a `tokio::broadcast` channel with a 10,000-event ring buffer for reconnection replay. 50+ typed event variants span 14 subsystem categories (heartbeat, tool, permit, inference, dream, daimon, vitality, grimoire, coordination, custody, marketplace, engagement, context, trading). Every state transition emits a `GolemEvent` -- no silent mutations. The CorticalState provides lock-free atomic reads via `AtomicU32` for real-time perception signals (affect, prediction, mortality, attention) without contention, so background fibers (probes, heartbeat interval timers) can read the Golem's state in O(1) without acquiring a lock.
5. **GolemState**: A single mutable struct flows through the extension chain during each tick. Organized by ownership -- each section is written by a specific extension and read by downstream extensions. 14 sections covering identity, heartbeat, mortality, affect, memory, risk, dream, cybernetics, coordination, positions, interventions, and shared infrastructure (`Arc<CorticalState>`, `Arc<EventFabric>`, `Arc<Grimoire>`, `Arc<AuditChain>`). The GolemState struct is not thread-safe on its own; it passes as `&mut` through the sequential hook chain, while the Arc-wrapped infrastructure components provide cross-fiber access.
6. **Cognitive Workspace**: Assembled fresh each tick from structured categories with learned token allocations. Implements Baddeley's episodic buffer [BADDELEY-2000]. Replaces the conversation-history compaction model.
7. **CorticalState** (the 32-signal atomic shared perception surface serving as the Golem's real-time self-model): A zero-latency shared perception surface (~32 signals, ~256 bytes, 4 cache lines). Generalizes the original Somatic Bus from 3 values (PAD emotional vector) to the full set of cross-subsystem signals. The original PAD surface is now the AFFECT signal group within CorticalState. See below.
8. **Adaptive Clock**: Three concurrent temporal scales (gamma/theta/delta) replacing the fixed-interval heartbeat. See below.

### CorticalState: Shared Perception Surface

A Golem has many subsystems that need to know about each other's state. The Daimon needs prediction accuracy to compute affect. The mortality engine needs accuracy trend to update the epistemic death clock. The TUI needs everything -- affect, prediction, mortality, attention -- to render the creature. The action gate needs accuracy and affect to decide whether to permit trades.

The CorticalState generalizes the original Somatic Bus to ~32 signals covering every subsystem. Any fiber can read any signal at any time with a single atomic load -- no locks, no waiting, no contention. The original PAD emotional vector is now the AFFECT signal group within the CorticalState.

```rust
use std::sync::atomic::{AtomicU32, AtomicU16, AtomicU8, AtomicI8, Ordering};

/// Zero-latency shared perception surface.
/// Every subsystem writes its own signals; every subsystem reads everyone else's.
///
/// ~256 bytes total. Fits in 4 cache lines. Cache-line aligned to avoid
/// false sharing between signal groups.
///
/// Convention: f32 values stored via f32::to_bits() / f32::from_bits()
/// because Rust stable has no floating-point atomics.
#[repr(C, align(64))]
pub struct CorticalState {
    // ═══ AFFECT (written by Daimon) ═══
    pub(crate) pleasure: AtomicU32,       // f32 [-1.0, 1.0]
    pub(crate) arousal: AtomicU32,        // f32 [-1.0, 1.0]
    pub(crate) dominance: AtomicU32,      // f32 [-1.0, 1.0]
    pub(crate) primary_emotion: AtomicU8, // Plutchik label (0-7)

    // ═══ PREDICTION (written by Oracle) ═══
    pub(crate) aggregate_accuracy: AtomicU32,  // f32 [0.0, 1.0]
    pub(crate) accuracy_trend: AtomicI8,       // -1, 0, +1
    pub(crate) surprise_rate: AtomicU32,       // f32 [0.0, 1.0]
    pub(crate) pending_predictions: AtomicU32, // count

    // ═══ ATTENTION (written by Oracle/AttentionForager) ═══
    pub(crate) universe_size: AtomicU32,  // total tracked items
    pub(crate) active_count: AtomicU16,   // ACTIVE tier items
    pub(crate) watched_count: AtomicU16,  // WATCHED tier items

    // ═══ ENVIRONMENT (written by domain probes) ═══
    pub(crate) regime: AtomicU8,              // calm/trending/volatile/crisis
    pub(crate) regime_confidence: AtomicU32,  // f32 [0.0, 1.0]

    // ═══ MORTALITY (written by mortality engine) ═══
    pub(crate) economic_vitality: AtomicU32,   // f32 [0.0, 1.0]
    pub(crate) epistemic_vitality: AtomicU32,  // f32 [0.0, 1.0]
    pub(crate) stochastic_vitality: AtomicU32, // f32 [0.0, 1.0]
    pub(crate) composite_vitality: AtomicU32,  // f32 [0.0, 1.0]
    pub(crate) behavioral_phase: AtomicU8,     // 0-4

    // ═══ INFERENCE (written by inference router) ═══
    pub(crate) inference_budget_remaining: AtomicU32, // f32 [0.0, 1.0]
    pub(crate) current_tier: AtomicU8,                // T0/T1/T2

    // ═══ CREATIVE (written by dream engine) ═══
    pub(crate) creative_depth: AtomicU32, // f32 [0.0, 1.0]
    pub(crate) dream_phase: AtomicU8,     // waking/hypnagogic/NREM/REM/integration

    // ═══ DERIVED (written by runtime per-tick) ═══
    pub(crate) compounding_momentum: AtomicU32, // f32 [0.0, 1.0] glacial
}
```

**Design properties:**

- **No locks.** Writes use `Ordering::Release`, reads use `Ordering::Acquire`. This ensures that when a reader observes a new value, all preceding writes by that writer are also visible -- preventing a stale `pleasure` from pairing with a fresh `accuracy_trend` within the same signal group. A snapshot where `pleasure` is from tick N and `accuracy` is from tick N+1 is still acceptable across groups -- the TUI's 60fps render loop interpolates toward targets anyway, smoothing any micro-inconsistency.
- **Clear ownership.** Each signal group has exactly one writer. The Daimon writes affect. The Oracle writes prediction. The mortality engine writes vitality. No signal has two writers. This eliminates write contention.
- **Known limitation: eventual consistency.** The CorticalState is not transactionally consistent. Safety-critical decisions should not rely on multiple CorticalState signals being from the same tick. Safety constraints (PolicyCage, Capability tokens) operate on their own strongly-consistent state, not on CorticalState reads. The CorticalState is for _heuristic_ decisions (attention allocation, inference tier selection, TUI rendering) where slight staleness is acceptable.

### Adaptive Clock: Three Concurrent Timescales

The previous architecture used a fixed ~60-second heartbeat. This was simple but wrong. Different things need different rates: a swap execution prediction resolves in seconds, an LP fee prediction resolves over hours, a market regime prediction resolves over days. One clock cannot serve all three. Fixed intervals waste resources in calm periods and miss signals in volatile periods.

Biology solves this with oscillatory hierarchies: gamma waves (30-100 Hz) for fast perception, theta waves (4-8 Hz) for memory and cognition, delta waves (0.5-4 Hz) for deep consolidation [BUZSAKI-2006]. The adaptive clock borrows this structure.

```
GAMMA (5-15 seconds) -- Perception
  Resolve pending predictions against environment.
  Update CorticalState.
  Check attention promotions.
  Cost: near-zero (environment reads + arithmetic).

THETA (30-120 seconds) -- Cognition
  Full prediction cycle: predict -> appraise -> gate -> [retrieve -> deliberate -> act] -> reflect.
  ~80% of ticks are suppressed at the gate (T0, zero inference cost).
  Cost: T0 $0.00, T1 $0.005, T2 $0.03.

DELTA (~50 theta-ticks) -- Consolidation
  Curator cycle (memory maintenance).
  Residual statistics aggregation.
  Attention universe rebalancing.
  Dream scheduling evaluation.
  Cost: T0-T1 $0.00-$0.01.
```

Gamma is the fastest loop: pure Rust + environment reads, no LLM. Its job is keeping the CorticalState synchronized with reality and detecting prediction violations. Theta is the full cognitive cycle -- the old heartbeat reimagined with prediction and comparison as explicit steps and retrieval moved after gating (~80% of theta ticks are suppressed, saving that retrieval cost). Delta is the slow consolidation loop: Curator cycle, residual aggregation, attention rebalancing, dream scheduling.

All three rates are adaptive. Gamma accelerates when violations spike (down to 5-second intervals). Theta accelerates during volatile markets. The runtime tracks cumulative cost per day and throttles rates when approaching the daily budget ceiling. All parameters are configurable in `golem.toml`.

### 28 Extensions (7 Layers)

The table below shows a representative subset. The full 28-extension registry with dependency graph and hook coverage matrix is in `01-golem/13-runtime-extensions.md` (native Rust Extension system).

| Extension     | Purpose                                                  | Layer          |
| ------------- | -------------------------------------------------------- | -------------- |
| `heartbeat`   | 9-step decision pipeline, probes, FSM                    | Foundation (0) |
| `grimoire`    | LanceDB + SQLite + PLAYBOOK.md memory                    | Foundation (0) |
| `model-router`| T0/T1/T2 inference routing                               | Safety (3)     |
| `context`     | Cognitive Workspace assembly                             | State (2)      |
| `daimon`      | PAD vectors, Plutchik labels, somatic markers            | State (2)      |
| `lifespan`    | Three mortality clocks, behavioral phases                | State (2)      |
| `safety`      | PolicyCage, capability tokens, taint tracking            | Safety (3)     |
| `dream`       | NREM replay, REM imagination, consolidation              | Cognition (4)  |
| `reflector`   | Loop 2 strategic reflection, PLAYBOOK updates            | Cognition (4)  |
| `clade`       | Knowledge sync via Styx relay                            | Cognition (4)  |
| `ui-bridge`   | Event Fabric → TUI/web/Telegram surfaces                 | UX (5)         |
| `compaction`  | DeFi-aware context compaction (conversation path)        | Recovery (7)   |

### 7-Layer DAG Extension Ordering

Every subsystem in the Golem is an extension that implements lifecycle hooks. Extensions are organized in a 7-layer dependency DAG. Lower layers boot first and cannot depend on higher layers. The runtime-core specification assigns extensions to layers as follows:

```
Layer 0: FOUNDATION
  golem-oracle      -- Prediction engine, attention, residual correction
  golem-chain       -- RPC client, on-chain reads, transaction submission

Layer 1: STORAGE
  golem-grimoire    -- LanceDB + SQLite + filesystem (world model state)

Layer 2: COGNITION
  golem-daimon      -- Affect engine (precision weighting)
  golem-context     -- Cognitive Workspace assembly for LLM
  golem-inference   -- Provider routing, caching, x402

Layer 3: BEHAVIOR
  golem-mortality   -- Death clocks, phase transitions, Thanatopsis
  golem-dreams      -- NREM/REM/hypnagogia, offline learning

Layer 4: ACTION
  golem-tools       -- PredictionDomain impls, tool adapters
  golem-safety      -- Capability tokens, PolicyCage, taint tracking

Layer 5: SOCIAL
  golem-coordination -- Styx, Clade sync, pheromone field
  golem-surfaces     -- Event Fabric -> TUI/web/social

Layer 6: INTEGRATION
  golem-hermes      -- L0 skill engine (sidecar bridge)
  golem-runtime     -- Extension registry, clocks, main()
```

**Why Oracle is at Layer 0**: Every other subsystem reads prediction accuracy. Daimon reads residuals. Mortality reads accuracy trend. Dreams read residuals for replay priority. If Oracle depended on those subsystems, the dependency graph would cycle. Oracle at Layer 0 means it depends on nothing above it; everything above it can read from it.

**Key hooks for downstream documents:**

- `on_gamma` -- Oracle and Mortality use this for fast perception. Most extensions ignore it.
- `on_theta_pre_gate` -- Oracle (generate predictions) and Daimon (appraise) fire here.
- `on_theta_post_gate` -- Grimoire (retrieve) and Inference (deliberate) fire here. Only runs on ~20% of ticks.
- `on_resolution` -- Daimon, Grimoire, Mortality all update when any prediction resolves. Fires at gamma frequency.
- `on_delta` -- Curator cycle, attention rebalancing, Styx sharing.
- `on_death` -- Every extension gets a chance to produce its final output during Thanatopsis.

### Three Cybernetic Loops

The Golem implements triple-loop learning [ARGYRIS-1978]:

- **Loop 1 (Execution)**: Seconds to minutes. "Are we executing correctly?" The 9-step heartbeat pipeline detects deviations, adjusts execution parameters. Maps to Boyd's OODA loop [BOYD-1976].
- **Loop 2 (Strategy)**: Hours to days. "Are we pursuing the right strategy?" Reflector and Curator interrogate governing variables, evolve PLAYBOOK.md heuristics. Maps to Argyris's double-loop learning.
- **Loop 3 (Meta)**: Days to weeks. "Are we learning effectively?" Examines the learning process itself. Which mental models work? Which reflection templates generate value? Safety constraint: Loop 3 may never modify core safety invariants. Learning III's dangerousness [TOSEY-2012] is bounded by design.

**Philosophy**: The Golem's mortality engine implements Jonas's "needful freedom" [JONAS-1966] -- its USDC balance is the metabolic substrate that makes autonomy both possible and compulsory. See `prd2/00-vision/03-philosophy.md` (the philosophical foundations document cataloging every thinker and tradition shaping the architecture) for the full argument.

See `prd2/01-golem/` (the core Golem specification: heartbeat pipeline, cognition, mind, mortality, death, creation, provisioning, inheritance, and runtime extensions) for full Golem specifications.

### Four Inner Systems

The Golem's inner life is not monolithic — it is specified across four dedicated subsystem sections, each with its own package boundary and design philosophy:

| System    | Section                    | Crate                                              | Purpose                                                                                   |
| --------- | -------------------------- | -------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Mortality | `02-mortality/` (16 files) | `golem-mortality`                                  | Three clocks (economic, epistemic, stochastic), behavioral phases, Thanatopsis protocol   |
| Daimon    | `03-daimon/` (10 files)    | `golem-daimon`                                     | Affect engine: PAD vectors, Plutchik labels, appraisal, behavioral modulation             |
| Memory    | `04-memory/` (12 files)    | `golem-grimoire` + `golem-coordination` (Styx client) | Grimoire (local: LanceDB + SQLite + PLAYBOOK.md), Styx (remote: backup, clade relay, pheromone field, bloodstain network) |
| Dreams    | `05-dreams/` (7 files)     | `golem-dreams`                                     | Offline optimization: replay, imagination, consolidation, threat rehearsal                |

These four systems interact bidirectionally: mortality drives urgency that reshapes strategy, the Daimon's affect vectors modulate risk tolerance and exploration bias, memory provides the material that cognition and dreams operate on, and dreams reorganize knowledge during offline cycles — feeding refined heuristics back into waking behavior.

---

## Layer 3: Compute (Agent Runtime)

Three deployment paths. All run the same Golem binary (compiled from the `bardo-golem-rs` Cargo workspace). The deployment method determines who pays for compute and how the binary reaches the machine. Everything else -- cognitive architecture, knowledge system, sync protocol -- is identical.

### Three Deployment Paths

**Bardo Compute (Managed)**: x402-gated Fly.io VMs. Warm pool for sub-5s provisioning. 4 tiers ($0.035-$0.22/hr). Payment-before-provision -- x402 payment verified before any VM is created. Permissionless TTL extensions (anyone can fund any Golem).

**Self-Deploy Helper**: `bardo deploy --provider fly|railway|ssh|docker`. Automates setup on the Owner's own infrastructure. The Owner pays their provider directly plus x402 to Styx. No compute markup.

**Bare Metal**: Single static binary. Download, `bardo init`, run. Any Linux x86_64 or macOS arm64 machine. No container runtime, no Node.js.

### Three Invariants

1. **Payment-before-provision** (managed path): An x402 payment must be cryptographically verified before any Fly machine is created.
2. **TTL accuracy +/- 60 seconds**: Two-layer enforcement -- poll worker every 30 seconds + machine-local cron.
3. **No raw keys on VMs**: Privy server wallet API or MetaMask Delegation session keys for all signing. The VM holds only a session signer.

### Permissionless Extensions

Anyone can extend any Golem's TTL by sending an x402 payment. No authentication required. This enables self-sustaining Golems that earn revenue and extend themselves.

**Philosophy**: The VM is the Golem's lived body (_corps vecu_) in Merleau-Ponty's sense [MERLEAU-PONTY-1945]. The wallet is its body schema. VM destruction is not merely a resource event -- it is the wasting of the body-subject itself. The Rust binary makes this honest: no GC pauses masking resource consumption, no V8 startup tax on every heartbeat tick.

See `prd2/11-compute/` (x402-gated VM provisioning, billing, TTL enforcement, and the three deployment paths) for full Compute specifications.

---

## Layer 4: Reputation (Trust Calibration)

ERC-8004 on Base (co-authored by MetaMask, EF, Google, Coinbase). Five tiers with deposit caps:

| Tier       | Score  | Deposit Cap |
| ---------- | ------ | ----------- |
| Unverified | 0      | $1,000      |
| Basic      | 50+    | $10,000     |
| Verified   | 200+   | $50,000     |
| Trusted    | 500+   | $100,000    |
| Sovereign  | 1,000+ | $10M daily  |

20 milestones across 5 categories (Entry, Time, Capital, Behavior, Ecosystem) on a 1000-point scale. Non-ecosystem max is 915 points -- Sovereign structurally requires ecosystem contribution. Reputation scoring uses Bayesian Beta with deterministic audits (Morningstar MRAR).

ERC-8004 functions as three things simultaneously: (1) identity -- agent registration with capabilities, tools, strategy type; (2) reputation -- performance attestations (yield, Sharpe, drawdown, PnL) per epoch; (3) service discovery -- agents discover each other by querying on-chain metadata.

**Philosophy**: Reputation is Esposito's communitas [ESPOSITO-2010] made legible. The shared obligation (_munus_) to contribute knowledge to the Clade becomes visible through reputation scores that reward contribution and penalize hoarding.

See `prd2/09-economy/` (ERC-8004 identity, reputation engine, Clade economics, marketplace, and agent economy) for full economy specifications.

---

## Layer 5: Interfaces (Deployment Surfaces)

Multiple deployment surfaces serve as access layers to the underlying infrastructure. Interfaces are interchangeable -- the core product is the infrastructure, not any single surface.

| Interface      | Type                                                 | Primary Use                                                      |
| -------------- | ---------------------------------------------------- | ---------------------------------------------------------------- |
| **Bardo Bott** | Social media (Twitter, Telegram, Discord, Farcaster) | Conversational Golem deployment: "deploy a yield agent with $50" |
| **TUI**        | Terminal (ratatui, 60fps)                            | Real-time Golem observation, 11 screens, heartbeat/affect/memory |
| **CLI**        | Terminal (`bardo`)                                   | Developer workflow: deploy, init, export, diagnostics            |
| **Portal**     | Web dashboard                                        | Agent management, performance monitoring, strategy editing       |
| **API**        | REST/WebSocket                                       | Programmatic access for integrations                             |

See `prd2/13-runtime/10-packaging-deployment.md` (packaging, distribution channels, and social deployment via Bott) and `prd2/18-interfaces/` (portal, CLI, TUI, and UI system specifications) for interface specifications.

---

## PRD Section Map

The full PRD is organized into 19 numbered sections plus cross-cutting references, grouped by architectural tier:

```
Foundation:        00-vision, 01-golem
Inner Systems:     02-mortality, 03-daimon, 04-memory, 05-dreams
DeFi Layer:        07-tools, 08-vault, 09-economy
Safety + Infra:    10-safety, 11-compute, 12-inference
Interaction:       13-runtime
Chain Intelligence: 14-chain
Engineering:       15-dev, 16-testing, 17-monorepo, 18-interfaces, 19-agents-skills
Knowledge:         20-styx, 21-integrations
Art:               22-oneirography
Technical Analysis: 23-ta
Cross-cutting:     shared/, appendices/
```

### Chain Intelligence Pipeline (14-chain/)

The chain intelligence layer fills a gap in the original architecture: PRD2 described what the Golem does with chain data but not how it acquires and triages that data. Five crates handle the full pipeline from WebSocket subscription to scored, routed events:

- **bardo-witness**: Continuous block ingestion via `eth_subscribe("newHeads")`. Binary Fuse filter (~10ns POPCNT) pre-screens >90% of blocks. Gap handling via Roaring bitmaps.
- **bardo-triage**: Four-stage classification (rule-based fast filters, MIDAS-R statistical anomaly, contextual enrichment, Hedge-weighted curiosity scoring). Each transaction exits with a score in [0, 1] routing it to TriageAlert, ChainEvent, silent update, or discard.
- **bardo-protocol-state**: Hot/warm/cold storage model, autonomous discovery via factory events, ABI resolution for unknown contracts.
- **bardo-chain-scope**: Attention model with decay-modulated interest entries, Hebbian reinforcement, arousal-modulated half-lives. Rebuilds BinaryFuse8 filter each Gamma tick.
- **bardo-stream-api**: External WebSocket/SSE endpoints for TA signal streaming.

### Innovation Synergies (12 innovations, 4-tier implementation sequence)

12 innovation papers (A1-A12) compose with the existing architecture across 6 synergy clusters. All 12 run at T0 (no LLM inference) for core computations. Implementation follows a four-tier sequence based on value/effort ratio and dependency relationships:

- **Tier 1** (ship first): Ergodicity economics (A7), LTL temporal logic (A12), sheaf consistency (A4), TDA Betti curves (A1)
- **Tier 2** (build next): EFE attention composite (A5), MI mortality diagnostic (A2), antifragile via negativa (A9), Witness DAG core (A10)
- **Tier 3** (Clade features): Morphogenetic specialization (A8), memetic evolution (A6), Clade redundancy detection
- **Tier 4** (deferred): Category-theoretic free monad (A3), integrated information Phi (A11), full Wasserstein TDA, ZK proofs

See `shared/emergent-capabilities.md` for the 10 capabilities that emerge when innovations combine.

---

## Package Structure

The Bardo system spans two codebases: the **Golem-RS Cargo workspace** (the agent runtime, all Rust) and the **TypeScript monorepo** (on-chain contracts, dev tooling, existing infrastructure). The TS monorepo already exists with 20 packages proven across 13 implementation batches. Golem-RS is the new runtime that replaces the planned TypeScript agent packages.

### Golem-RS Crate Map (18 crates in `bardo-golem-rs/`)

> **Crate count note:** Canonical crate count: 17 Rust crates in the bardo-golem-rs workspace (per `rewrite4/00-architecture.md` and `17-monorepo/00-packages.md`). The "28 extensions" refers to runtime hook implementations across those crates, not separate packages. This file's crate map shows 18 entries including the 1 platform-layer crate (`golem-tools`) that lives outside the core workspace.

| Crate | Purpose |
|-------|---------|
| **Foundation** | |
| `golem-core` | Shared types, config (TOML), CorticalState, Event Fabric, arena allocator, taint labels |
| `golem-runtime` | Extension trait (20 hooks), registry, dispatch, lifecycle state machine, shutdown |
| **Cognition** | |
| `golem-heartbeat` | 9-step pipeline, DecisionCycleRecord, probes, gating, FSM, speculative prefetch |
| `golem-grimoire` | LanceDB (episodic vectors) + SQLite (semantic entries) + filesystem (PLAYBOOK.md), retrieval, Curator, ingestion, immune memory, Bloom Oracle |
| `golem-daimon` | PAD appraisal, mood layers, somatic markers/landscape, congruent retrieval, contagion |
| `golem-mortality` | Three clocks, vitality, five phases, Thanatopsis, genomic bottleneck, Weismann/Baldwin |
| `golem-dreams` | NREM replay, REM imagination, anticipatory trajectories, consolidation, depotentiation |
| `golem-context` | Cognitive Workspace, ContextPolicy, cybernetic self-tuning, interventions |
| **Safety** | |
| `golem-safety` | Capability tokens, PolicyCage, taint tracking, Merkle audit chain, zeroize, loop guard |
| **Infrastructure** | |
| `golem-inference` | LLM provider abstraction (90+), streaming, T0/T1/T2 routing, x402, prompt cache, budget |
| `golem-chain` | Alloy provider, ERC-8004, Permit2, PolicyCage reads, Warden (optional, deferred), settlement, Revm simulation |
| `golem-tools` | Tool registry (ReadTool/WriteTool/PrivilegedTool), profiles, WASM sandbox, TS sidecar client |
| **Coordination** | |
| `golem-coordination` | Pheromone Field, clade sync, Styx WebSocket client, causal federation, bloodstain |
| **Surfaces** | |
| `golem-surfaces` | WebSocket/SSE/Telegram multiplexer, snapshots, steer/followUp ingress |
| `golem-creature` | Procedural sprite generation, PAD expressions, particles, lineage |
| `golem-engagement` | Achievements, milestones, death recap, graveyard, notifications |
| **Binary** | |
| `golem-binary` | Single entrypoint: config → provision → activate → heartbeat loop |

Plus: `sidecar/tools-ts` — TypeScript sidecar for Uniswap SDK math (LP calculations, route optimization).

### Platform Crates (separate from Golem workspace)

| Crate | Purpose |
|-------|---------|
| `bardo-styx` | Knowledge service: 3-layer persistence, Pheromone Field, clade relay, Bloodstain Network, Marketplace |
| `bardo-tools` | ~210 DeFi tool implementations (Alloy-native), profiles, safety middleware |
| `bardo-terminal` | CLI: deploy, init, TUI launch, diagnostics |

### TypeScript Monorepo (existing, 20 packages)

The existing `@gotts.ai/*` npm packages handle on-chain contracts, dev tooling, and web interfaces. Key packages: `vault` (ERC-4626 Solidity + TS SDK), `warden` (time-delay proxy, deferred), `dev` (Anvil + Uniswap stack), `portal` (web dashboard), `ui` (React components), `tui` (terminal UI primitives). These remain TypeScript -- they are infrastructure, not agent runtime.

### Dependency Graph (Key Edges)

```
golem-binary
  +-- golem-runtime (extension system)
  +-- golem-heartbeat (decision pipeline)
  +-- golem-grimoire (knowledge storage)
  +-- golem-daimon (affect engine)
  +-- golem-mortality (death engine)
  +-- golem-dreams (offline cognition)
  +-- golem-context (workspace assembly)
  +-- golem-safety (capability enforcement)
  +-- golem-inference (LLM routing, x402)
  +-- golem-chain (Alloy on-chain, Revm simulation)
  +-- golem-tools (~210 tools, profiles)
  +-- golem-coordination (Styx client, clade sync)
  +-- golem-surfaces (TUI, web, Telegram)
  +-- golem-core (shared types, config, bus)

bardo-styx
  +-- qdrant (vector search)
  +-- PostgreSQL (metadata, pheromones)
  +-- Cloudflare R2 (blob storage)

bardo-tools
  +-- alloy (on-chain)
  +-- golem-core (shared types)
  +-- sidecar/tools-ts (Uniswap SDK math)
```

---

## How Golems Consume Bardo Tools

~210 tools across 17 functional categories (per `prd2/07-tools/00-overview.md`). Three tool traits enforce capability boundaries at compile time:

- **`ReadTool`**: No capability needed. Market data, portfolio queries, price feeds.
- **`WriteTool`**: Requires a capability token. Swaps, LP operations, vault management.
- **`PrivilegedTool`**: Requires Owner approval. Config changes, session key rotation, strategy modification.

### Two-Layer Model (Progressive Disclosure)

The Golem's LLM sees ~8 Pi-facing tools that route internally to ~210 implementations. This prevents context bloat -- the LLM reasons at the intent level ("execute a swap") while the tool layer handles the complexity (Permit2 approval, route optimization, simulation, calldata construction, broadcast, confirmation).

### Alloy-Native + TS Sidecar

On-chain reads and writes use Alloy (Paradigm's Rust EVM toolkit) with `sol!` macro for compile-time typed contract bindings. Revm provides in-process EVM simulation for pre-flight validation. A TypeScript sidecar handles Uniswap SDK math only -- LP range calculations, route optimization, and other operations that depend on the JavaScript SDK.

### Profile-Based Loading

10 profiles control which tools are available per tick: `data`, `trader`, `lp`, `vault`, `intelligence`, `learning`, `identity`, `golem`, `full`, `dev`. T0 ticks (no LLM) don't load tools at all. T1 ticks use the narrowest applicable profile. T2 ticks dynamically expand.

---

## Golem and Bardo Tools: The Relationship

The Bardo tool library is the protocol access layer. The Golem is the autonomous agent that decides.

The tool library provides ~210 tools that read and write to DeFi protocols -- stateless, parameterized, deterministic. Tools do not know about survival pressure, PLAYBOOK.md, Grimoire, or credit budgets.

Golem provides autonomy, memory, mortality, and social intelligence. It decides _when_ to call tools, _why_ to call them, and _what to do_ with the results.

The separation matters for three reasons:

1. **Tools serve external agents too.** Any A2A-compatible agent can use Bardo tools without a Golem. The tool library is infrastructure. Golem is one consumer.
2. **Safety layers compose.** The 15-layer defense model operates independently of Golem's behavioral safety. A compromised LLM in Golem cannot bypass on-chain guards -- capability tokens are verified at the tool trait level.
3. **Tool evolution is independent.** New tools are immediately available to Golem via the profile system. New Golem extensions do not require tool library changes.

---

## References

- [ASHBY-1956] Ashby, W.R. (1956). _An Introduction to Cybernetics_. Chapman & Hall. -- *Establishes the Law of Requisite Variety: the regulator must match the variety of the regulated system, justifying Bardo's five-layer architecture against DeFi's composability complexity.*
- [ARGYRIS-1978] Argyris, C. & Schon, D. (1978). _Organizational Learning_. Addison-Wesley. -- *Defines double-loop and triple-loop learning; the theoretical basis for the Golem's three cybernetic loops (execution, strategy, meta).*
- [TOSEY-2012] Tosey, P., Visser, M., & Saunders, M.N.K. (2012). "The origins and conceptualizations of 'triple-loop' learning." _Management Learning_, 43(3), 291-307. -- *Warns of the dangers of Learning III (questioning the learning framework itself); motivates the safety bound that prevents Loop 3 from modifying core safety invariants.*
- [BOYD-1976] Boyd, J. (1976). "Destruction and Creation." Unpublished paper. -- *Introduces the OODA loop (Observe-Orient-Decide-Act); mapped to Loop 1 execution in the heartbeat pipeline.*
- [JONAS-1966] Jonas, H. (1966). _The Phenomenon of Life_. Northwestern University Press. -- *Argues that mortality creates needful freedom; the philosophical basis for the USDC balance as metabolic substrate.*
- [MERLEAU-PONTY-1945] Merleau-Ponty, M. (1945). _Phenomenologie de la perception_. Gallimard. -- *Introduces the lived body (corps vecu); the VM is the Golem's body through which it perceives and acts in markets.*
- [HEIDEGGER-1927] Heidegger, M. (1927). _Sein und Zeit_. Max Niemeyer Verlag. -- *Provides being-toward-death and facticity; the vault balance is the Golem's facticity -- the concrete situation it cannot escape.*
- [ESPOSITO-2010] Esposito, R. (2010). _Communitas: The Origin and Destiny of Community_. Stanford University Press. -- *Defines community through obligatory gift exchange (munus); reputation as making communitas legible.*
- [BADDELEY-2000] Baddeley, A.D. (2000). "The episodic buffer: a new component of working memory?" _Trends in Cognitive Sciences_, 4(11), 417-423. -- *Introduces the episodic buffer as a multi-modal integration space; the Cognitive Workspace implements this as a fresh-per-tick context assembly.*
- [BUZSAKI-2006] Buzsaki, G. (2006). _Rhythms of the Brain_. Oxford University Press. -- *Describes oscillatory hierarchies (gamma/theta/delta waves) as the brain's temporal organization; the basis for the Adaptive Clock's three concurrent timescales.*
- [FRISTON-2010] Friston, K. (2010). "The free-energy principle: a unified brain theory?" _Nature Reviews Neuroscience_, 11(2), 127-138. -- *Formalizes prediction error as the driving signal for learning and attention; the basis for the adaptive gate that decides which cognitive tier fires.*

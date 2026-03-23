# What is a Golem [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Language**: Rust (edition 2024) | **Repository**: `bardo-golem-rs` (Cargo workspace)
>
> **Crates**: `golem-core`, `golem-runtime`, `golem-heartbeat`, `golem-grimoire`, `golem-daimon`, `golem-mortality`, `golem-dreams`, `golem-context`, `golem-safety`, `golem-inference`, `golem-chain`, `golem-tools`, `golem-coordination`, `golem-surfaces`, `golem-creature`, `golem-engagement`, `golem-binary`

> **Reader orientation:** This is the architectural overview of the Golem runtime, the core autonomous agent in the Bardo system. It belongs to the `01-golem` section, which specifies how individual agents are built, think, trade, and die. The key concept to grasp before diving in: a Golem is a mortal, autonomous DeFi agent compiled as a single Rust binary, running on a micro VM, with a wallet, a strategy, a knowledge system, and a finite lifespan. Full term definitions are in `prd2/shared/glossary.md` (canonical Bardo term definitions).

## Document Map

| File                       | Topic                                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------------ |
| `00-overview.md`           | Architecture overview, key terms, crate inventory, CoALA mapping                           |
| `01-cognition.md`          | Three-tier inference (T0/T1/T2 cognitive gating), x402 micropayments, cost optimization    |
| `02-heartbeat.md`          | The 9-step decision cycle each Golem executes on every tick                                |
| `03-mind.md`               | Cognitive mechanisms overview: attention, habituation, homeostasis, state management        |
| `04-mortality.md`          | _(stub)_ -- See `02-mortality/` (three death clocks, behavioral phases, vitality scoring)  |
| `05-death.md`              | _(stub)_ -- See `02-mortality/06-thanatopsis.md` (the four-phase Death Protocol: Acceptance, Settlement, Reflection, Legacy) |
| `06-creation.md`           | Golem creation flow, STRATEGY.md compilation, owner onboarding wizard                      |
| `07-provisioning.md`       | Fly.io compute provisioning, warm pools, 8-step type-state deployment pipeline             |
| `08-funding.md`            | Wallet funding, Permit2 integration, delegation grants, metabolic self-funding loop        |
| `09-inheritance.md`        | Knowledge inheritance across generations, Clade sync, anti-proletarianization mandate      |
| `10-replication.md`        | Replicant spawning, MAP-Elites selection, hypothesis testing, heuristic mutation            |
| `11-lifecycle.md`          | Type-state machine (Provisioning/Active/Dreaming/Terminal/Dead), kill-switch, bardo mapping |
| `12-teardown.md`           | 8-step teardown pipeline, settlement order, Pause vs Destroy comparison                    |
| `13-runtime-extensions.md` | Extension trait system (20 hooks, 28 extensions, 7-layer DAG), split into 13a and 13b      |
| `14-context-governor.md`   | Context assembly as a learnable control problem, CognitiveWorkspace, cybernetic context learning |
| `15-sleepwalker.md`        | Sleepwalker phenotype: observer Golems that earn by selling knowledge, not trading          |
| `17-prediction-engine.md`  | Domain-agnostic Oracle, residual correction (~15k/day), attention foraging, action gating   |
| `18-cortical-state.md`     | CorticalState (32-signal atomic shared perception surface), Daimon affect engine, TUI mapping |

---

## The Problem This Project Solves

DeFi (decentralized finance) strategy execution has become too complex and too fast for humans to operate manually. Across lending protocols, AMM liquidity provision, yield farming, routing, and market structure exploitation, the opportunities that generate consistent returns all share the same requirements: continuous monitoring, cross-protocol composition, and reaction times faster than any human can sustain.

The obvious answer is "use AI agents." But existing agent frameworks are missing the infrastructure that would make agents viable as autonomous economic actors. They can reason, but they can't hold capital safely. They can execute trades, but they can't be constrained by on-chain policy. They can remember things, but they can't compress, forget, or transfer knowledge across generations. They run indefinitely, which sounds like a feature but is actually a problem.

A Golem is Bardo's answer: an autonomous DeFi agent that lives in a single Rust binary on a single VM, holds a wallet, executes a strategy, accumulates knowledge, and -- when its operating budget runs out -- dies. It passes what it learned to its successors.

The rest of this document explains what that means concretely: the architecture, the subsystems, the crate layout, and the design decisions that got us here.

---

## The Argument in One Sentence

An autonomous agent that cannot die has no reason to be efficient, no pressure to learn, and no incentive to share what it knows -- mortality is the architectural constraint that makes intelligence economically viable [JONAS-1966].

---

## Why Mortality: The Core Thesis

Most agent frameworks assume their agents will run indefinitely. The default is immortality. This project argues the default is wrong.

The argument rests on four pillars, each grounded in independent research.

### Pillar 1: Mortality Creates Intelligence

Biology has run self-replicating adaptive agents for four billion years and has never once produced an immortal organism. Not because evolution lacks the mechanism (telomerase exists and is actively suppressed), but because mortality is computationally superior to immortality for adaptive systems.

Tom Ray's Tierra system (1991) demonstrated this in silico: self-replicating programs competing for CPU time spontaneously evolved parasites, hyperparasites, and social cooperators -- but only because a "reaper" killed the oldest programs. Without the reaper, memory filled and evolution halted [RAY-1991]. Lenski et al. (2003) showed in the Avida platform that digital organisms evolved complex Boolean logic through stepping-stone mutations -- a process that requires sufficient population turnover [LENSKI-2003]. Vostinar et al. (2019) demonstrated that programmed cell death itself evolves as adaptive behavior: 12.5% of a digital unicellular population evolved to kill themselves because it benefited kin [VOSTINAR-2019].

The Kreps-Milgrom-Roberts-Wilson (KMRW) theorem from game theory proves that agents with uncertain finite horizons cooperate more effectively than infinite-horizon agents, because the uncertain endpoint creates incentives to build reputation rather than defect [KREPS-MILGROM-ROBERTS-WILSON-1982]. For Golems managing shared DeFi infrastructure, this means mortal agents are structurally more trustworthy than immortal ones.

**Design implication**: Each Golem has three independent death clocks (economic, epistemic, stochastic). Any reaching zero kills it. This creates behavioral phases (Thriving -- Stable -- Conservation -- Declining -- Terminal) that modulate every aspect of cognition -- a dying Golem allocates attention differently than a thriving one.

### Pillar 2: Affect Creates Salience

Antonio Damasio's somatic marker hypothesis, demonstrated through the Iowa Gambling Task, established that patients who retained full cognitive ability but lost emotional signaling made consistently worse decisions under uncertainty [DAMASIO-1994]. The mechanism: emotions function as rapid heuristic signals that bias choices toward advantageous options before conscious deliberation completes. They are not noise -- they are the fastest available signal about whether a situation is dangerous.

Bechara et al. (2000) showed that neurotypical Iowa Gambling Task subjects developed correct behavioral biases (avoiding bad decks) before they could articulate why -- their skin conductance responses anticipated losses before conscious awareness [BECHARA-2000]. Bower (1981) established mood-congruent memory: emotional states bias which memories are retrieved, creating feedback loops between affect and cognition [BOWER-1981]. Emotional RAG (2024) validated this computationally: LLM agents with emotion-tagged retrieval significantly outperformed non-emotional retrieval across three datasets [EMOTIONAL-RAG-2024].

**Design implication**: The Daimon engine gives each Golem a continuous affective state using the Pleasure-Arousal-Dominance (PAD) model [RUSSELL-MEHRABIAN-1977]. This state biases memory retrieval, modulates deliberation thresholds, and drives a visible creature visualization. An anxious Golem retrieves different memories, thinks harder about surprises, and looks different than a confident one.

### Pillar 3: Dreaming Creates Insight

During NREM sleep, hippocampal sharp-wave ripples compress minutes of waking experience into ~100ms bursts for consolidation [WILSON-MCNAUGHTON-1994], [BUZSAKI-2015]. During REM sleep, the brain generates novel scenarios by recombining past experience in a neurochemical environment that strips emotional valence while preserving informational content -- the "Sleep to Forget, Sleep to Remember" (SFSR) model [WALKER-VAN-DER-HELM-2009]. Wagner et al. (2004) showed that subjects who slept were 2.6x more likely to discover hidden rules in data than those who stayed awake [WAGNER-2004].

Hafner et al.'s DreamerV3 (2025) proved computationally that agents trained entirely inside imagined forward trajectories from a learned world model outperform model-free agents across 150+ diverse tasks [HAFNER-DREAMERV3-2025]. Ha & Schmidhuber (2018) showed that a controller trained inside its own hallucinated dreams achieves competitive performance with far less real-world data [HA-SCHMIDHUBER-2018].

**Design implication**: Golems periodically enter a "dream state" with three phases: NREM-style replay (re-experiencing prioritized past episodes), REM-style imagination (counterfactual scenario generation, anticipatory trajectory planning), and consolidation (compressing insights into the PLAYBOOK heuristics file, pruning stale knowledge).

### Pillar 4: Rust Creates Honesty

A mortal agent managing real capital on a micro VM ($0.025/hour) cannot afford garbage collection pauses during time-sensitive settlement. It cannot afford 500ms Node.js startup taxes on every heartbeat tick. It cannot afford a 200MB+ V8 memory footprint inside a 256MB enclave.

Rust provides deterministic memory management (no GC pauses), compile-time safety verification (invalid state transitions are compiler errors), and zero-cost abstractions (the type system enforces security invariants with no runtime overhead). The philosophical architecture and systems architecture converge: a Golem's mortality is more honest when its body cannot lie about resource consumption.

Concretely: type-state machines make it a compiler error to tick a dead Golem. Capability tokens make it structurally impossible for a compromised LLM to execute unauthorized trades. Arena-scoped tick allocators eliminate memory fragmentation across weeks of continuous operation. These are not optimizations -- they are architectural properties that TypeScript agents structurally cannot replicate.

---

## Six Structural Moats

The architectural decisions above are not arbitrary -- they produce six capabilities that no existing agent framework can replicate without adopting the same foundational constraints:

1. **Die.** Mortality creates behavioral phases, knowledge compression, and urgency. Immortal agents lack the selection pressure that makes knowledge curation economically rational. (Pillar 1 above; `02-mortality/` for the full mortality specification.)

2. **Think.** The three-tier cognitive gating system (T0/T1/T2) reduces inference cost by 18x while routing full deliberation to the ticks that actually need it. Without gating, the Adaptive Clock's theta-frequency ticks (30-120s, regime-dependent) would cost hundreds of dollars per day in LLM calls. With gating, ~$32/day. (`01-cognition.md`, `14-context-governor.md`.)

3. **Trust.** Capability tokens, PolicyCage smart contracts, and taint tracking enforce safety at the type system and blockchain layers. The LLM cannot be prompt-injected into bypassing a Solidity constraint. (KMRW theorem for mortal cooperation; `../10-safety/` for the full safety spec.)

4. **Pay.** x402 micropayments turn the Golem's wallet into its API key. No billing accounts, no API key management. The Golem pays for inference, compute, and coordination using the same wallet that holds its trading capital. (`01-cognition.md` section 5.)

5. **Remember.** Dreams compress experience into procedural knowledge (PLAYBOOK.md heuristics). Cross-lifetime inheritance transfers validated knowledge to successor Golems with confidence discounting. No other framework has a forgetting mechanism, let alone a dreaming one. (`04-memory/`, `02-mortality/06-thanatopsis.md`.)

6. **Cooperate.** The Pheromone Field enables stigmergic coordination without strategy leakage. Clade sync shares knowledge within an owner's fleet. Bloodstains carry death-validated warnings. These are coordination primitives that don't require agent-to-agent messaging. (`09-inheritance.md`, `../03-coordination/`.)

---

## What a Golem Actually Does

A Golem is a general-purpose DeFi agent. It trades, provides liquidity, lends, borrows, arbitrages, manages positions -- whatever its STRATEGY.md prescribes. The Bardo tool library gives it access to DeFi protocols including AMMs (Uniswap, Aerodrome), lending (Aave, Morpho, Compound), yield aggregation, and cross-chain bridges. No single protocol is privileged -- the tool registry is extensible and protocol-agnostic.

The operational loop works like this: a Golem's owner funds it with credits. The Golem spends those credits on compute, inference, and gas to execute its strategy. When credits hit zero, the Golem dies. There is no free tier, no grace period, no debt.

Each Golem runs on a single Fly.io VM as a single Rust binary. One Golem = one process = one wallet = one chain (Base L2 in v1). It boots, loads its strategy and knowledge, starts its heartbeat loop (Adaptive Clock: gamma 5-15s, theta 30-120s, delta 5-30min), and runs autonomously until it dies or is shut down by its owner.

The heartbeat loop is the core of everything. On each tick, the Golem:

1. **Observes** the current market state (prices, liquidity, gas, health factors)
2. **Retrieves** relevant knowledge from its memory (biased by its current emotional state)
3. **Analyzes** how surprising the observation is compared to its expectations
4. **Gates** the decision -- if nothing surprising happened, skip the LLM entirely ($0.00 cost); if something interesting happened, invoke a cheap model; if something truly unexpected happened, invoke a full model
5. **Simulates** any proposed action in an in-process EVM fork before broadcasting
6. **Validates** the action against on-chain policy constraints (the PolicyCage)
7. **Executes** via PolicyCage-enforced write path (optional time-delayed proxy for additional hardening; see `prd2-extended/10-safety/02-warden.md`)
8. **Verifies** the outcome by checking the transaction receipt
9. **Reflects** on what happened and records a structured DecisionCycleRecord

This gating mechanism is where the economics get interesting. About 80% of ticks result in "nothing surprising, suppress the LLM" -- those cost $0.00 in inference. The remaining 20% split between cheap and expensive models. A Golem that wastes inference budget on boring ticks dies faster. Natural selection for cost efficiency.

---

## The Golem Container

The Golem container is three co-located processes supervised by a single Rust binary. Each process runs in a different language because each language is the right tool for what that process does. They communicate over Unix domain sockets using JSON-RPC 2.0.

```
golem-container/
|-- golem-binary          # Rust: heartbeat, Grimoire, Daimon, mortality, safety
|-- hermes-agent/         # Python: per-golem learning organ, skill engine
|   |-- skills/           # Per-golem skill library (SKILL.md files)
|   |-- memory/           # FTS5 session database
|   +-- sidecar_server.py # JSON-RPC on /tmp/hermes-golem.sock
|-- sanctum-ts/           # TypeScript: Uniswap SDK math, DeFi computations
+-- config/
    |-- golem.toml        # Runtime configuration
    |-- hermes.yaml       # Hermes sidecar configuration
    +-- STRATEGY.md       # Owner-authored strategy
```

| Process | Language | IPC | What it does |
|---|---|---|---|
| `golem-binary` | Rust | TCP :8080 (internal) | Runtime, heartbeat pipeline, 28 extensions, Grimoire, Daimon, mortality, safety |
| `hermes-agent` | Python | UDS `/tmp/hermes-golem.sock` | Per-golem skill engine, learning loop, affect-modulated retrieval |
| `sanctum-ts` | TypeScript | UDS `/tmp/sanctum-ts.sock` | Uniswap SDK math, tick arithmetic, price impact simulation |
| `ollama` (optional) | Go | TCP :11434 | Local inference (self-hosted mode only) |

### Why Three Processes

**Isolation.** The LLM-facing sidecar (Hermes) runs Python because the Hermes Agent framework is Python-native and its skill creation pipeline depends on Python's string manipulation and templating ecosystem. If Hermes crashes (an OOM during skill materialization, a malformed JSON-RPC response from a flaky inference provider), the Rust heartbeat keeps ticking. The supervisor restarts the crashed process. Heartbeat continuity is preserved.

**Language strengths.** Rust is correct for the heartbeat pipeline, where GC pauses during time-sensitive settlement would be catastrophic and where the type-state machine that prevents ticking a dead Golem is a compile-time guarantee. Python is correct for Hermes, where the Hermes Agent SDK and NLP operations for affect-modulated retrieval live. TypeScript is correct for `sanctum-ts`, where Uniswap's official SDK libraries handle tick math, price computation, and swap routing. Rewriting Uniswap's SDK in Rust would be a multi-month effort with no architectural benefit.

**Independent restart.** Each process can restart without bringing down the others. `sanctum-ts` can crash and restart in under 200ms; the Golem just retries the pending math call. `hermes-agent` can restart and reload its skill library in ~2 seconds; the Golem misses one curator cycle but continues trading. Only `golem-binary` crashing is fatal, because it owns the heartbeat, the wallet, and the mortality state.

### Resource Budgets

| Resource | Embedded | Headless | Bardo Compute (micro) | Bardo Compute (small) | Bardo Compute (large) |
|---|---|---|---|---|---|
| RAM (golem-binary) | ~80 MB | ~80 MB | ~80 MB | ~80 MB | ~80 MB |
| RAM (hermes-agent) | ~120 MB | ~120 MB | ~80 MB (reduced) | ~120 MB | ~200 MB |
| RAM (sanctum-ts) | ~50 MB | ~50 MB | ~40 MB | ~50 MB | ~50 MB |
| RAM (ollama, optional) | 1-4 GB | 1-4 GB | N/A | N/A | 1-4 GB |
| RAM total (no ollama) | ~250 MB | ~250 MB | ~200 MB | ~250 MB | ~330 MB |
| Disk (Grimoire) | 50 MB - 2 GB | 50 MB - 2 GB | 50 MB - 500 MB | 50 MB - 1 GB | 50 MB - 2 GB |

The compiled `golem-binary` weighs 15-25 MB. Startup time: ~200ms to first tick readiness, excluding sidecar spawn (1-2 seconds for Hermes).

### IPC Model

All inter-process communication uses JSON-RPC 2.0 over Unix domain sockets (UDS). UDS because the processes are always co-located (no network hops, no TLS overhead). JSON-RPC because it is the lingua franca of MCP tooling. The `golem-binary` is always the caller; Hermes and `sanctum-ts` are request-response servers. Neither sidecar initiates communication. If a sidecar does not respond within 5 seconds, the Golem logs the timeout, skips whatever it was asking for, and continues the tick. Latency budget per IPC round-trip: <1ms.

**Failure modes and recovery.** If `hermes-agent` crashes mid-tick, the current tick completes without skill injection. The supervisor process (built into golem-binary) detects the crash via process exit code and restarts Hermes within 2 seconds. The Golem loses one curator cycle at most. If `sanctum-ts` crashes, any pending math computation returns an IPC timeout error; the Golem skips the action that required the computation and logs it. The supervisor restarts sanctum-ts in under 200ms. If `golem-binary` itself panics, the process exits. For headless and Bardo Compute modes, the external supervisor (systemd, launchd, or Fly.io's built-in restart) restarts the entire container. The Golem reads `manifest.bardo` on restart and resumes. For embedded mode, the TUI detects the child process exit and shows the owner a restart prompt.

**Sidecar health checks.** The golem-binary pings both sidecars every 30 seconds with a `health_check` JSON-RPC call. If three consecutive pings fail, the sidecar is declared dead and restarted. Health check results are included in the `GolemEvent::HealthStatus` event that flows to the TUI.

### Deployment Mode Comparison

| Aspect | Embedded | Headless | Bardo Compute |
|---|---|---|---|
| Where it runs | Inside TUI process tree | Background daemon, local | Fly.io microVM |
| TUI dependency | Dies with TUI | Independent | Independent |
| Network requirement | Local + Styx (free) | Local + Styx (free) | Styx (free, required) |
| Provisioning time | Instant | Instant | 3-30 seconds |
| Cost | Your hardware | Your hardware | x402 USDC/hr |
| Maximum Golems | 1-2 (resource-limited) | 2-5 (resource-limited) | Unlimited |
| Best for | Learning, testing | Home servers, VPS | Remote clade operations |
| Uptime guarantee | Your machine's uptime | Your machine's uptime | Fly.io SLA (99.99%) |
| Death on crash | Yes (no supervisor) | Supervisor restarts | Supervisor restarts |

### Migrating Between Modes

A Golem's state is portable. The Grimoire, PLAYBOOK.md, skill library, and config files are all filesystem-resident. Moving a Golem between modes means stopping it in one mode and starting it in another, pointing at the same state directory.

**Embedded to Headless.** Stop the TUI. Run `bardo spawn --headless --name oracle-3`. The headless process reads the same `~/.bardo/golems/oracle-3/` directory. The Golem resumes from its last `manifest.bardo` checkpoint. No knowledge lost, no position changes.

**Headless to Bardo Compute.** This requires a state export. Run `bardo export oracle-3 --output oracle-3-state.tar.gz`. The export bundles the Grimoire, PLAYBOOK.md, skill library, and config. Then `bardo deploy --mode compute --import oracle-3-state.tar.gz --fund 50`. The provisioning pipeline injects the state into the Fly.io VM at creation time. The Golem boots with its full memory intact on remote infrastructure.

**Bardo Compute to Headless.** Run `bardo pull golem-V1StGXR8_Z5j` from the TUI. This triggers a Grimoire sync via Styx, downloads the current state, and writes it to `~/.bardo/golems/<name>/`. Then `bardo spawn --headless --name <name>`. The remote Golem should be destroyed afterward to avoid running two copies of the same identity (which Styx will reject during WebSocket auth).

Migration between modes does not change the Golem's ERC-8004 identity. The on-chain registration persists regardless of where the binary runs. What changes is the `deployment_type` field in the identity metadata, which updates on the next deferred registration cycle.

### Heartbeat Error Handling

| Failure | Behavior |
|---------|----------|
| Observe (RPC down) | Retry 3x with 2s backoff. On failure, skip to Learn with `ObservationFailure` event. |
| Decide (inference error) | Fall back to T0 (heuristic-only). Log `InferenceFailure`. |
| Act (tx reverted) | Record revert reason. Emit `ActionFailure`. No automatic retry. |
| Gate (safety check) | Block the action. Emit `GateRejection`. |

### State Persistence Between Restarts

For embedded and headless modes, all Golem state persists to the filesystem at `~/.bardo/golems/<name>/`:

```
~/.bardo/golems/oracle-3/
|-- golem.toml              # Runtime config
|-- STRATEGY.md             # Owner's strategy
|-- PLAYBOOK.md             # Machine-evolved heuristics (written by dreams)
|-- grimoire/
|   |-- episodes.lance/     # LanceDB vector store (episode memories)
|   |-- structured.db       # SQLite (insights, heuristics, warnings, causal links)
|   +-- structured.db-wal   # Write-ahead log
|-- hermes/
|   |-- skills/             # Per-golem skill library (SKILL.md files)
|   +-- memory/             # FTS5 session database
|-- logs/
|   +-- heartbeat.jsonl     # Structured heartbeat log
|-- golem.sock              # UDS path (headless mode, runtime only)
+-- manifest.bardo          # Sealed state snapshot (written on clean shutdown)
```

On restart, the Golem reads `manifest.bardo` to recover its exact state: mortality clocks, vitality phase, Grimoire cursors, Hermes skill library checksum. The first tick after restart is a T1 (cheap model) to re-evaluate market state since the Golem was offline. Subsequent ticks resume normal gating.

For Bardo Compute mode, state persists on the VM's ephemeral disk for the VM's lifetime. The Grimoire syncs to Styx every 6 hours for backup. When the VM is destroyed, the local state is gone. The Styx backup and the death testament (if Thanatopsis completed) are the only survivors.

---

## The CorticalState

The Golem's runtime state is centralized in the CorticalState: a cache-aligned struct of ~32 atomic signals (~256 bytes, fits in 4 cache lines) that any subsystem can read without locking. Affect, prediction accuracy, attention metrics, mortality clocks, and compounding momentum are all continuously available at zero latency. The CorticalState generalizes the original PAD surface (which only carried 3 values) to cover every cross-subsystem signal.

```rust
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

**No locks.** All reads use `Ordering::Relaxed`. A snapshot where `pleasure` is from tick N and `accuracy` is from tick N+1 is acceptable -- the TUI's 60fps render loop interpolates toward targets, smoothing any micro-inconsistency.

**Clear ownership.** Each signal group has exactly one writer. The Daimon writes affect. The Oracle writes prediction. The mortality engine writes vitality. No signal has two writers, eliminating write contention entirely.

**Known limitation: eventual consistency.** The CorticalState is not transactionally consistent. Safety-critical decisions should not rely on multiple CorticalState signals being from the same tick. Safety constraints (PolicyCage, Capability tokens) operate on their own strongly-consistent state. The CorticalState is for heuristic decisions (attention allocation, inference tier selection, TUI rendering) where slight staleness is acceptable.

The full CorticalState specification, including the TUI interpolating variable mapping and Daimon affect engine integration, is in `18-cortical-state.md` (32-signal atomic shared perception surface, ALMA affect layers, TUI variable mapping).

---

## Vaults as Optional Capital Aggregation

ERC-4626 vaults are an optional capability, not a requirement. A Golem that has demonstrated a track record can deploy a vault to let others piggyback on its strategy. This is one path to self-sustaining economics, not the only one.

A Golem with vault management capability can:

- **Deploy a vault** via `AgentVaultFactory` (CREATE2 deterministic address). The Golem becomes vault manager.
- **Manage strategy** by allocating vault assets across Uniswap positions, lending protocols, and yield sources.
- **Earn fees** through management fees (up to 500 bps/yr) and performance fees (up to 5,000 bps above hurdle rate, high-water mark).

The vault's `PolicyCage` constrains what the Golem can do with deposited assets -- approved asset lists, max position sizes, max drawdown. These constraints are on-chain and immutable. The Golem's LLM cannot override them. Fee architecture in `../08-vault/04-fees.md`.

In practice, vault capital aggregation follows a trust ramp:

1. The owner runs the Golem on their own capital. The on-chain track record accrues against the Golem's ERC-8004 identity.
2. Once performance is legible (weeks or months of verifiable history), the owner may deploy a vault and seed it with their own capital.
3. Outside depositors can evaluate the track record and choose to deposit. The owner's own stake is the skin-in-the-game signal.

Nobody deposits into a stranger's vault on day one. Reputation is earned, not assumed. Identity specification in `../09-economy/00-identity.md`.

For Clade evolution (`10-replication.md`), the fitness signal is strategy-specific PnL, not vault AUM. A trading Golem that generates consistent returns replicates regardless of whether it runs a vault. A vault manager whose depositors flee faces mortality pressure through credit depletion, same as any other underperforming Golem. Clade ecology in `../02-mortality/10-clade-ecology.md`.

---

## Design Principles

These eight principles shape every decision in the system. When subsystem specs conflict, these resolve the conflict.

1. **Reuse everything.** Wire existing Bardo packages (`golem-tools`, `golem-chain`, `golem-safety`) into the runtime's extension trait system. Never rebuild what exists.

2. **Wallets are identity.** Every Golem has a wallet. x402 turns that wallet into payment credentials for inference, compute, and coordination. No API keys, no accounts, no billing setup.

3. **Tools are the communication interface.** Agents never exchange raw LLM-to-LLM text. All inter-agent communication flows through typed tool interfaces backed by on-chain state machines.

4. **Finite lifespans are non-negotiable.** Credits are consumed by compute and inference. When the balance reaches zero, the Golem dies. There is no free tier, no grace period, no debt.

5. **Safety constraints are architectural, not behavioral.** Enforced at the smart contract and infrastructure layers. The LLM cannot override kill-switches, population caps, or the DeFi Constitution. Rust's type system (Capability tokens, taint tracking) provides compile-time enforcement.

6. **Deterministic where possible.** On-chain performance scoring uses formulas (Morningstar/GIPS pattern), never LLMs. LLM-based scoring is provably inconsistent [LAU-2026], [HALDAR-HOCKENMAIER-2025].

7. **Local-first knowledge.** Each Golem's local Grimoire is its authoritative source of truth. Clade Grimoire sync (via Styx) is optional. PLAYBOOK.md is per-Golem.

8. **LLM-last.** The heartbeat FSM (System 1) handles ~80% of ticks at $0.00 cost. LLMs (System 2) are invoked only for decisions requiring deliberation [FRUGALGPT-2023].

---

## Key Terms

This table defines the vocabulary used throughout the rest of the PRD. If a term isn't here, check the shared glossary at `../shared/glossary.md`.

| Term                    | Definition                                                                                                                                                                                                                                                                                                                                           |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Golem**               | An autonomous DeFi agent with a wallet, strategy, knowledge system, and finite lifespan. One Golem = one Fly.io VM = one Rust binary. Deployed on a single chain (Base in v1).                                                                                                                                                                       |
| **Grimoire**            | A Golem's complete knowledge base. Stores episodes (raw observations), insights (distilled patterns), heuristics (validated rules), warnings (danger signals), and strategy fragments. Implemented as LanceDB (vectors) + SQLite (structured data) + filesystem (PLAYBOOK.md). The sellable knowledge artifact.                                      |
| **PLAYBOOK.md**         | A machine-evolved heuristics file stored on the filesystem. Written only by the Dream Integration phase. Contains the reasoning context the LLM reads each tick. Analogous to an athlete's muscle memory -- procedural knowledge distilled from experience.                                                                                          |
| **Daimon**              | The affect (emotion) engine. Named for the Greek concept of a guiding spirit. Computes PAD vectors (Pleasure-Arousal-Dominance), tracks somatic markers, and biases cognition through mood-congruent retrieval.                                                                                                                                      |
| **PAD Vector**          | A three-dimensional affective state representation: Pleasure [-1,1], Arousal [-1,1], Dominance [-1,1]. Based on the Russell-Mehrabian model (1977). Maps to discrete Plutchik emotions (joy, fear, anger, etc.) via octant classification. Stored as `f32` in the CorticalState AFFECT signal group.                                                  |
| **Heartbeat**           | The autonomous execution loop. On each theta tick (Adaptive Clock, 30-120s regime-dependent), the Golem observes market state, appraises it emotionally, decides whether to think hard about it, possibly acts, and always records what happened.                                                                                                    |
| **Tick**                | One heartbeat cycle. The fundamental unit of Golem cognition. Each tick produces a `DecisionCycleRecord` -- a structured snapshot of everything that happened.                                                                                                                                                                                       |
| **DecisionCycleRecord** | A typed, self-contained record of one tick: observation, emotional appraisal, gating decision, deliberation (if any), actions (if any), outcome verification, knowledge written, mortality state. Persisted as bincode.                                                                                                                              |
| **Owner**               | The human who creates, funds, configures, and watches a Golem. Distinct from the Bardo platform (the infrastructure running Styx and the inference gateway).                                                                                                                                                                                         |
| **Clade**               | An owner's fleet of Golems. Siblings in a clade share knowledge through a structured pipeline (export -- transmit -- ingest with confidence discounts), never through direct merge.                                                                                                                                                                  |
| **Styx**                | A single, globally available Rust service at `wss://styx.bardo.run`. Hosts the Pheromone Field, relays clade sync, processes bloodstains, provides retrieval across three privacy layers (Vault, Clade, Lethe (formerly Commons)). Every Golem maintains one persistent outbound WebSocket. No inbound ports needed. Strictly additive -- ~95% capability without it. |
| **Pheromone Field**     | A shared signal space for stigmergic coordination. Golems deposit anonymous signals (threats, opportunities, wisdom) that decay over time and reinforce when independently confirmed. No direct agent-to-agent communication.                                                                                                                        |
| **Bloodstain**          | Knowledge validated by death. When a Golem dies, its death warnings carry a costly signaling premium -- the dead agent cannot benefit from its own warning, making the information maximally honest.                                                                                                                                                 |
| **Thanatopsis**         | The four-phase death protocol: Acceptance -- Settlement -- Reflection -- Legacy. Named for William Cullen Bryant's poem on contemplating death.                                                                                                                                                                                                      |
| **Bardo Tools**         | The unified Uniswap tool library (171+ tools across 31 categories). Golems consume it; they do not replace it.                                                                                                                                                                                                                                       |
| **Bardo Inference**     | x402-gated LLM gateway. Per-request USDC micropayment. Semantic cache, automatic fallbacks, per-strategy cost attribution.                                                                                                                                                                                                                           |
| **STRATEGY.md**         | User-authored or NL-compiled strategy spec. Parameters, risk bounds, triggers, target protocols. The Golem's mission statement.                                                                                                                                                                                                                      |
| **Heartbeat config**    | Owner standing orders for the heartbeat scheduler, defined as the `heartbeat` section within STRATEGY.md. Tick intervals, cron schedules, monitoring priorities.                                                                                                                                                                                     |
| **x402**                | HTTP 402-based micropayment protocol using signed USDC `transferWithAuthorization` on Base. This is how Golems pay for LLM inference -- each request includes a signed payment, no API keys or billing accounts needed.                                                                                                                               |
| **Warden**              | Optional time-delayed execution proxy. Write actions can follow an announce-wait-execute pattern with a cancellation window, giving the owner (or safety system) time to intervene before an irreversible action completes. Deferred to extended spec; see `prd2-extended/10-safety/02-warden.md`.                                                     |
| **PolicyCage**          | On-chain smart contract boundaries for agent strategy: approved assets, max position sizes, max drawdown, rebalance frequency limits. Cannot be bypassed by prompt injection because it exists at the contract layer, not the LLM layer. Canonical specification in `prd2/10-safety/02-policy.md`.                                                   |
| **Replicant**           | Child Golem spawned to test a hypothesis. Fixed lifespan (Hayflick limit), reporting obligations, no spawning by default.                                                                                                                                                                                                                            |
| **am-AMM**              | **[HARDENED]** Auction-managed AMM. Harberger lease for pool management rights. [CORE] alternative: standard vault management without auction mechanics.                                                                                                                                                                                             |
| **CorticalState**       | A cache-aligned `#[repr(C, align(64))]` struct of ~32 atomic signals (~256 bytes, 4 cache lines). The zero-latency shared perception surface: any subsystem reads any signal with a single atomic load. Generalizes the Somatic Bus to cover affect, prediction, attention, mortality, inference, creative, and environment signals. Full spec in `18-cortical-state.md`. |
| **Somatic Bus**         | _Superseded by CorticalState._ The original 3-value PAD atomic surface (formerly `SomaticBus`). CorticalState extends the same lock-free `AtomicU32` bit-reinterpretation technique to ~32 signals. The PAD values are now the AFFECT signal group within CorticalState.                                                            |
| **Event Fabric**        | A non-blocking `tokio::broadcast` ring buffer for internal state transitions. Every subsystem emits typed events (50+ event types across 16 subsystems). Surfaces (TUI, web dashboard, Telegram) subscribe to the events they care about. Category-based subscription: TUI subscribes to Affect, Mortality, Cognition, Dream, and Prediction -- not to every resolution event. |
| **Extension**           | A Rust struct implementing the Extension trait (20 lifecycle hooks). There are 28 extensions organized into 7 dependency layers. The primary organizational unit -- each subsystem is one or more extensions.                                                                                                                                        |

---

## How the System Relates to Existing Agent Frameworks

Sumers et al. (2024) proposed Cognitive Architectures for Language Agents (CoALA), a framework that organizes language agents along three dimensions: memory, action space, and decision-making procedure [SUMERS-2024]. CoALA draws on decades of cognitive architecture research (Soar, ACT-R) to formalize what a language agent IS: a system with modular memory, structured actions (internal reasoning + external grounding), and a repeated decision cycle.

This matters because it gives us a shared vocabulary for comparing what Golem-RS does to what other agent frameworks do. Here's how the CoALA concepts map:

| CoALA Concept                    | Golem-RS Implementation                                                                                                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Working memory**               | The Cognitive Workspace -- assembled fresh each tick from structured categories with learned token allocations. Implements Baddeley's episodic buffer [BADDELEY-2000].          |
| **Episodic memory**              | LanceDB columnar vectors with embeddings. Stores raw observations, trade outcomes, regime shifts. Supports vector similarity search for mood-congruent retrieval.               |
| **Semantic memory**              | SQLite with indexed columns. Stores insights, heuristics, warnings, causal edges. Supports confidence tracking, Ebbinghaus decay, Curator maintenance cycles.                   |
| **Procedural memory**            | PLAYBOOK.md -- a single-writer text file on the filesystem. Machine-evolved heuristics that the LLM reads as reasoning context. Written only by Dream Integration.              |
| **External actions (grounding)** | 423+ DeFi tools across 15+ categories. Alloy-native for on-chain reads/writes. TypeScript sidecar for Uniswap SDK math. Capability-gated by trust tier.                         |
| **Internal actions (reasoning)** | Three-tier LLM inference: T0 (suppress -- no LLM, $0.00), T1 (analyze -- cheap model), T2 (deliberate -- full model). Routing based on prediction error vs. adaptive threshold. |
| **Internal actions (retrieval)** | Four-factor scoring: recency x importance x relevance x emotional congruence. Mood-congruent retrieval biases results based on current PAD state.                               |
| **Internal actions (learning)**  | Grimoire writes (episodes, insights), PLAYBOOK evolution (via dreams), causal graph updates, somatic marker recording.                                                          |
| **Decision cycle**               | The 9-step heartbeat pipeline: observe -- retrieve -- analyze -- gate -- simulate -- validate -- execute -- verify -- reflect. Produces a `DecisionCycleRecord` each tick.      |

The key extension beyond CoALA: Golem-RS adds **mortality** (three death clocks that modulate every cognitive process), **affect** (PAD vectors that bias retrieval and gating), and **dreaming** (offline consolidation phases). No existing CoALA implementation has any of these.

---

## How Golems Consume Bardo Tools

The Bardo tool library is the protocol access layer -- 171+ tools for reading and writing to DeFi protocols (AMMs, lending, yield, bridges), ERC-4626 vaults, and ERC-8004 identity registries. A Golem is an autonomous consumer of those tools. The relationship is consumption, not replacement.

Here's how the layers fit together:

```
User Intent
     |
     v
Golem Runtime (golem-runtime crate)
  |  Heartbeat FSM, survival pressure, learning loops
  |
  +-- Extension Trait: 20 lifecycle hooks, 28 extensions, 7-layer DAG
  |
  +-- Tools (two-layer): 8 Golem-facing tools -> golem-tools (166+ adapters)
  |     preview_action, commit_action -> execute_swap, vault_deposit, ...
  |
  +-- Context: golem-context crate (Context Governor)
  |     Cognitive Workspace, ContextPolicy, cybernetic context learning
  |
  +-- Knowledge: golem-grimoire crate
  |     Episodes, Insights, Heuristics, PLAYBOOK.md
  |
  +-- Inference: golem-inference crate
  |     x402 payments, model routing, semantic cache
  |
  +-- Coordination: golem-coordination crate
  |     Styx client, Pheromone Field, clade sync, Bloodstain Network
```

**Why a two-layer tool model?** When embedded in the Golem runtime, Bardo tools are not directly exposed to the LLM. Instead, the Golem presents a thin set of Golem-facing tools (`preview_action`, `commit_action`, `cancel_action`, `query_state`, `search_context`, `query_grimoire`, `update_directive`, `emergency_halt`) that internally route to the 166+ tool implementations. This serves two purposes:

1. **Context window efficiency.** 8 tool definitions cost ~1,200 tokens. 166 tool definitions cost ~15,000 tokens. The Golem can't afford to waste 14,000 tokens per tick on tool definitions it rarely uses.
2. **Safety boundary.** Every write must flow through the ActionPermit system (preview -- commit), where simulation hashes and policy hashes are verified before execution. The LLM never directly calls `execute_swap`; it calls `preview_action`, reviews the simulation result, then calls `commit_action`. See `13-runtime-extensions.md` S3.

When consumed by external A2A clients (not embedded in a Golem), Bardo tools are exposed directly as before. The two-layer model applies only to the Golem runtime.

Tool profiles determine which subset of Bardo tools a Golem loads at boot. A DCA strategy needs `trader` profile tools. An LP management strategy needs `lp` profile tools. A vault manager needs `vault` profile tools. Deferred loading ensures only relevant tools consume context window space.

---

## What Onboarding Produces

When a Golem is created, the provisioning pipeline produces four things:

1. **A wallet** -- via one of three custody modes: MetaMask Delegation (recommended, ERC-7710/7715), Privy server wallet (legacy, AWS Nitro Enclaves), or local key + delegation (dev). The Golem never sees its own private key; it signs transactions through a policy-constrained session signer.
2. **A strategy** -- A STRATEGY.md document (generated by AI from a user prompt, or expanded from a template) defining what the Golem trades, when, and under what risk bounds.
3. **An identity** -- An optional ERC-8004 registration making the Golem discoverable by other agents and establishing its on-chain reputation.
4. **Compute** -- Either a Fly.io VM (hosted mode) or a local process (self-hosted mode) running the runtime with a heartbeat scheduler.

From the owner's perspective, creation is a single flow: describe what you want the Golem to do, fund it, and it starts running. The provisioning system handles wallet creation, strategy compilation, VM allocation, and initial heartbeat scheduling behind the scenes.

---

## Crate Map

The project is a Cargo workspace with 18 crates. Each crate maps to a specific subsystem. Dependencies flow downward -- higher-numbered layers depend on lower-numbered ones but never the reverse.

The crate map is organized into seven layers. Understanding this hierarchy is important: it explains what depends on what, and guarantees that low-level subsystems (like core types or the event bus) cannot accidentally pull in high-level concerns (like dreams or coordination).

```
bardo-golem-rs/
+-- Cargo.toml                          # Workspace root (pure Rust + one TS sidecar)
|
+-- crates/
|   |
|   |  === FOUNDATION (depended on by everything) ===
|   |
|   +-- golem-core/                     # Shared types, config, cross-cutting infrastructure
|   |   +-- types.rs                    # GolemId, PADVector, MarketRegime, Domain, CognitiveTier, etc.
|   |   +-- config.rs                   # TOML-based GolemConfig, StrategyParams, HeartbeatConfig
|   |   +-- cortical_state.rs           # Lock-free atomic perception surface (~32 signals, AtomicU32 bit-reinterpreted as f32)
|   |   +-- event_fabric.rs             # tokio::broadcast bus, 10K ring buffer, replay-from-seq
|   |   +-- arena.rs                    # Per-tick bump allocator using bumpalo
|   |   +-- taint.rs                    # Information flow taint labels: WalletSecret, OwnerSecret, etc.
|   |
|   |  === RUNTIME (the skeleton -- extension system + lifecycle) ===
|   |
|   +-- golem-runtime/                  # Extension registry, hook dispatch, lifecycle state machine
|   |   +-- extension.rs                # The Extension trait: 20 async hooks with default no-op impls
|   |   +-- registry.rs                 # Accepts extensions, validates DAG, computes firing orders
|   |   +-- dispatch.rs                 # Fires hooks across extensions in topological order
|   |   +-- state.rs                    # GolemState: the mutable struct that flows through the hook chain
|   |   +-- lifecycle.rs                # Type-state machine: Provisioning -> Active -> Dreaming -> Terminal -> Dead
|   |   +-- shutdown.rs                 # 10-phase graceful shutdown for Fly.io SIGTERM (30s budget)
|   |
|   |  === COGNITION (the mind -- heartbeat, memory, affect, dreams) ===
|   |
|   +-- golem-heartbeat/                # The autonomous decision cycle engine
|   |   +-- pipeline.rs                 # 9-step pipeline: observe -> retrieve -> analyze -> gate -> ... -> reflect
|   |   +-- record.rs                   # DecisionCycleRecord: structured per-tick cognitive snapshot
|   |   +-- probes.rs                   # Market probes: price, liquidity, gas, health factor, regime detection
|   |   +-- gating.rs                   # Adaptive prediction-error threshold (Friston 2010)
|   |   +-- fsm.rs                      # Heartbeat FSM with interrupt substates (steer mid-tick)
|   |   +-- speculation.rs              # Speculative read-tool prefetch (co-occurrence learning)
|   |
|   +-- golem-grimoire/                 # The Golem's knowledge system (LanceDB + SQLite + filesystem)
|   |   +-- grimoire.rs                 # Top-level struct: coordinates episodic, semantic, playbook
|   |   +-- episodic.rs                 # LanceDB: vector embeddings, columnar metadata, HNSW search
|   |   +-- semantic.rs                 # SQLite: 5 entry types, confidence tracking, causal edges, emotion log
|   |   +-- playbook.rs                 # PLAYBOOK.md: filesystem single-writer with snapshot history
|   |   +-- causal.rs                   # Causal graph: SQLite adjacency table with BFS traversal
|   |   +-- retrieval.rs                # Four-factor scoring: recency x importance x relevance x affect
|   |   +-- curator.rs                  # 50-tick maintenance: validate, prune, compress, cross-reference
|   |   +-- admission.rs                # Quality gate: prevents low-quality entries from polluting memory
|   |   +-- decay.rs                    # Ebbinghaus forgetting: entries lose confidence without revalidation
|   |   +-- ingestion.rs                # Four-stage pipeline for external knowledge: quarantine -> adopt
|   |   +-- immune.rs                   # Adaptive defense: learns to recognize attack patterns (Janeway 2001)
|   |   +-- bloom.rs                    # Bloom Oracle: LSH pre-filter saves Styx query costs
|   |   +-- embedder.rs                 # fastembed-rs: nomic-embed-text-v1.5, Matryoshka, 768-dim, in-process
|   |   +-- propagation.rs              # PropagationPolicy: private -> clade -> lethe -> listed
|   |
|   +-- golem-daimon/                   # The affect (emotion) engine
|   |   +-- appraisal.rs                # OCC/Scherer appraisal -> PAD vector + Plutchik label
|   |   +-- mood.rs                     # Three temporal layers: emotion (seconds), mood (hours), personality
|   |   +-- somatic_markers.rs          # Point markers: situation -> emotional valence (Damasio 1994)
|   |   +-- somatic_landscape.rs        # Continuous emotional topology over strategy parameter space
|   |   +-- congruent_retrieval.rs      # Mood biases which memories surface; contrarian every 100 ticks
|   |   +-- contagion.rs                # Clade emotional contagion: PAD attenuation with caps, 6h half-life
|   |   +-- helplessness.rs             # Detection: Dominance < -0.3 for 200+ ticks (Seligman 1972)
|   |
|   +-- golem-mortality/                # The death engine
|   |   +-- clocks.rs                   # Three death clocks: economic (credit), epistemic (fitness), stochastic
|   |   +-- vitality.rs                 # VitalityState: multiplicative composition, any zero = death
|   |   +-- phases.rs                   # Type-state: Thriving -> Stable -> Conservation -> Declining -> Terminal
|   |   +-- thanatopsis.rs              # Four-phase death protocol: Acceptance -> Settlement -> Reflection -> Legacy
|   |   +-- bottleneck.rs               # Genomic bottleneck: compress full Grimoire to <=2048 entries at death
|   |   +-- weismann.rs                 # Inherited knowledge starts at confidence x 0.85^generation
|   |   +-- baldwin.rs                  # Validated heuristics surviving N+ generations -> structural defaults
|   |   +-- hayflick.rs                 # Replicant max-tick limits (named for Hayflick limit in cell biology)
|   |   +-- attention_budget.rs         # Rational inattention: mortality pressure -> attention allocation
|   |
|   +-- golem-dreams/                   # The dreaming engine (offline intelligence)
|   |   +-- scheduler.rs                # When to dream: time-based + emotional load + novelty triggers
|   |   +-- nrem.rs                     # NREM replay: prioritized, bidirectional, utility-weighted episodes
|   |   +-- rem.rs                      # REM imagination: counterfactual "what if" scenario generation
|   |   +-- anticipatory.rs             # Forward trajectories: causal graph -> predicted future market states
|   |   +-- consolidation.rs            # PLAYBOOK evolution, insight compression, schema integration
|   |   +-- depotentiation.rs           # Emotional charge reduction on traumatic episodes (SFSR model)
|   |   +-- threats.rs                  # DeFi threat catalog: simulate MEV, liquidation, oracle attacks
|   |   +-- micro.rs                    # Background mini-consolidation between ticks (lightweight, continuous)
|   |   +-- staging.rs                  # Dream-sourced hypotheses await live validation before promotion
|   |
|   +-- golem-context/                  # Context engineering (what the LLM sees)
|   |   +-- workspace.rs                # Cognitive Workspace: assembled fresh each tick, not grown-and-compacted
|   |   +-- policy.rs                   # ContextPolicy: learned token allocation per category
|   |   +-- cybernetics.rs              # Three self-tuning loops: per-tick, per-Curator, per-regime-change
|   |   +-- predictive.rs               # Background fiber pre-builds workspace reactively
|   |   +-- interventions.rs            # Typed steers/followUps with severity routing + decision windows
|   |
|   |  === SAFETY (defense-in-depth) ===
|   |
|   +-- golem-safety/                   # Compile-time + runtime security layers
|   |   +-- capabilities.rs             # Capability<T> tokens: ReadTool needs none, WriteTool needs a token
|   |   +-- policy_cage.rs              # On-chain constraint enforcement via Alloy
|   |   +-- taint_check.rs              # Information flow: WalletSecret never reaches LLM context
|   |   +-- audit.rs                    # Merkle hash-chain: tamper-evident, forensic-grade action log
|   |   +-- anchor.rs                   # Periodic on-chain anchoring of audit root hash (~$0.001/anchor)
|   |   +-- zeroize.rs                  # Automatic secret wiping on drop (zeroize crate)
|   |   +-- loop_guard.rs              # Detects degenerate tool-call loops (same tool, same args, N times)
|   |   +-- permits.rs                  # ActionPermit lifecycle: created -> committed -> expired -> cancelled
|   |
|   |  === INFRASTRUCTURE (chain, inference, tools) ===
|   |
|   +-- golem-inference/                # LLM provider abstraction (extracted from pi_agent_rust)
|   |   +-- provider.rs                 # 90+ provider auth flows, streaming, error handling
|   |   +-- streaming.rs                # SSE parser handling CR/LF, split packets, UTF-8 tails
|   |   +-- routing.rs                  # Three-tier: T0 (none), T1 (Haiku-class), T2 (Opus-class)
|   |   +-- x402.rs                     # Per-request USDC micropayment via Alloy
|   |   +-- cache.rs                    # Prompt cache key computation for provider-side caching
|   |   +-- budget.rs                   # Operational cost tracking: daily cap, burn rate, alerts
|   |
|   +-- golem-chain/                    # On-chain interaction via Alloy (Paradigm's Rust EVM toolkit)
|   |   +-- alloy_provider.rs           # Provider setup for Base L2
|   |   +-- erc8004.rs                  # Agent identity registry (on-chain Golem registration)
|   |   +-- permit2.rs                  # Uniswap Permit2 token approval flow
|   |   +-- policy_cage_read.rs         # Read PolicyCage constraints from on-chain state
|   |   +-- warden.rs                   # Optional time-delayed execution (deferred): announce -> wait -> execute / cancel
|   |   +-- settlement.rs              # Position settlement with Bardo State triage (shutdown path)
|   |   +-- revm_sim.rs                # In-process EVM simulation: fork mainnet, test tx, no broadcast
|   |
|   +-- golem-tools/                    # Tool harness: registry, execution, sandboxing
|   |   +-- trait.rs                    # Three tool traits: ReadTool, WriteTool, PrivilegedTool
|   |   +-- registry.rs                 # Tool registry with profiles (active, observatory, conservative)
|   |   +-- progress.rs                 # ProgressReporter: multi-step tools emit granular progress events
|   |   +-- speculation.rs              # Co-occurrence engine: learns tool access patterns, prefetches reads
|   |   +-- wasm_sandbox.rs             # Wasmtime: fuel metering + epoch interrupts for untrusted tools
|   |   +-- sidecar.rs                  # JSON-RPC client for the TypeScript Uniswap SDK sidecar
|   |
|   |  === COORDINATION (multi-agent intelligence) ===
|   |
|   +-- golem-coordination/             # Collective intelligence without direct communication
|   |   +-- pheromone.rs                # Pheromone Field: 3-layer stigmergic signal space (Grasse 1959)
|   |   +-- clade.rs                    # Peer-to-peer Grimoire sync: export -> transmit -> ingest (not merge)
|   |   +-- styx_client.rs              # WebSocket client for Styx coordination service
|   |   +-- causal_federation.rs        # Anonymized causal edge publishing + ingestion (Pearl 2009)
|   |   +-- bloodstain.rs               # Death-indexed warnings with costly signaling premium
|   |   +-- propagation.rs              # PropagationPolicy enforcement: who sees what knowledge
|   |
|   |  === SURFACES (user-facing) ===
|   |
|   +-- golem-surfaces/                 # Event Fabric -> user interfaces
|   |   +-- multiplexer.rs              # Routes events to WebSocket/SSE/Telegram subscribers
|   |   +-- websocket.rs                # Axum WebSocket handler: subscribe to subsystem categories
|   |   +-- sse.rs                      # Server-Sent Events fallback for web clients
|   |   +-- telegram.rs                 # Push notifications for high-signal events only
|   |   +-- snapshot.rs                 # GolemSnapshot: full state for initial load / reconnection
|   |   +-- steer.rs                    # Inbound steer/followUp from any surface -> intervention system
|   |
|   +-- golem-creature/                 # Visual identity engine (the "Tamagotchi")
|   |   +-- state_computer.rs           # Subscribes to EventFabric, computes creature visual state
|   |   +-- forms.rs                    # Evolution: Egg -> Hatchling -> Mature -> Weathered -> Transcendent
|   |   +-- expressions.rs              # PAD -> expression mapping (8 octant archetypes from Mehrabian 1974)
|   |   +-- particles.rs                # Visual effects: dream shimmer, death glow, trade sparks
|   |   +-- lineage.rs                  # Genealogy tree across generations, death recap data
|   |
|   +-- golem-engagement/               # Progression system (roguelike-inspired)
|   |   +-- achievements.rs             # Achievement definitions: lifecycle, performance, social, legendary
|   |   +-- milestones.rs               # First trade, first dream, first death, generation milestones
|   |   +-- death_recap.rs              # Kill screen: cause, stats, timeline, emotional arc, final words
|   |   +-- graveyard.rs                # Persistent memorial of all dead Golems across all lineages
|   |   +-- notifications.rs            # Toast/notification events emitted to Event Fabric
|   |
|   |  === BINARY (the entrypoint) ===
|   |
|   +-- golem-binary/                   # The single binary that ships
|       +-- main.rs                     # Entrypoint: config -> provision -> activate -> heartbeat loop
|       +-- cli.rs                      # CLI argument parsing (clap): --config, --data-dir, --phenotype
|       +-- startup.rs                  # Extension registration, fiber spawning, signal handling
|
+-- sidecar/
|   +-- tools-ts/                       # TypeScript sidecar for Uniswap SDK math
|       +-- package.json                # @uniswap/v3-sdk, @uniswap/v4-sdk, smart-order-router
|       +-- tsconfig.json
|       +-- src/
|           +-- index.ts                # Unix socket JSON-RPC server
|           +-- tools/                  # LP math, route optimization, calldata encoding
|           +-- types.ts                # Shared request/response types
|
+-- tests/
    +-- conformance/                    # Extension conformance (does each hook fire correctly?)
    +-- adversarial/                    # Ingestion poisoning defense tests
    +-- property/                       # Property-based tests (proptest): invariant verification
    +-- integration/                    # Full lifecycle: provision -> trade -> dream -> die -> inherit
```

---

## Architecture: 7-Layer Dependency Hierarchy

The 28 extensions are organized into 7 dependency layers. Extensions in layer N may depend on extensions in layers 0 through N-1, never the reverse. The runtime validates this DAG at startup -- if an extension declares a dependency on something in a higher layer, the binary refuses to start.

```
Layer 7 ─ RECOVERY (1 extension)
    compaction

Layer 6 ─ INTERVENTION (2 extensions)
    playbook, intervention

Layer 5 ─ UX (2 extensions)
    ui-bridge, observability

Layer 4 ─ COGNITION (4 extensions)
    turn-context, dream, cybernetics, clade

Layer 3 ─ SAFETY (8 extensions)
    safety, permits, risk, result-filter, coordination, compiler, model-router, x402-payment

Layer 2 ─ STATE (4 extensions)
    context, daimon, memory, lifespan

Layer 1 ─ INPUT (1 extension)
    input-router

Layer 0 ─ FOUNDATION (6 extensions)
    provider-adapter, telemetry, audit, grimoire, tools, heartbeat
```

Total: 28 extensions across 8 layers (0-7). The `after_turn` hook fires the nine-extension pipeline sequentially: heartbeat -> lifespan -> daimon -> memory -> risk -> dream -> cybernetics -> clade -> telemetry. Each extension reads what the previous wrote to GolemState.

The extension trait defines 20 lifecycle hooks grouped into six categories (session lifecycle, input processing, agent lifecycle, turn lifecycle, post-turn learning, system hooks). Each hook has a default no-op implementation, so extensions only need to implement the hooks they care about. The runtime fires hooks in topological order across all registered extensions. The full Extension trait is specified in `13-runtime-extensions.md` (split into 13a: extension architecture, registry, tool authorization, model routing; and 13b: type-state machine, event fabric, arena allocator, shutdown protocol); here is a summary:

```rust
#[async_trait]
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn layer(&self) -> u8;
    fn depends_on(&self) -> &[&str] { &[] }

    // Session lifecycle (1 hook)
    async fn on_session(&self, _reason: SessionReason, _ctx: &mut SessionCtx) -> Result<()> { Ok(()) }
    // Input processing (1 hook)
    async fn on_input(&self, _msg: &mut InputMessage, _ctx: &InputCtx) -> Result<InputAction> { Ok(InputAction::Pass) }
    // Agent lifecycle (2 hooks)
    async fn on_before_agent_start(&self, _ctx: &mut AgentStartCtx) -> Result<()> { Ok(()) }
    async fn on_agent_start(&self, _ctx: &AgentStartCtx) -> Result<()> { Ok(()) }
    // Turn lifecycle (7 hooks): turn_start, context, before_provider_request,
    //   tool_call, tool_execution_{start,update,end}, tool_result
    async fn on_tool_call(&self, _call: &ToolCall, _ctx: &mut ToolCallCtx) -> Result<ToolAction> { Ok(ToolAction::Allow) }
    // Post-turn learning (3 hooks)
    async fn on_after_turn(&self, _ctx: &mut AfterTurnCtx) -> Result<()> { Ok(()) }
    // System hooks (6 hooks): system_prompt, steer, send_message, debug, error, end
    async fn on_end(&self, _ctx: &EndCtx) -> Result<()> { Ok(()) }
}
```

---

## Data Flow: One Heartbeat Tick

This diagram shows the complete data flow of a single tick -- from market observation to surface rendering. Each arrow represents actual function calls in the codebase.

```
                                     +-------------------------+
                                     |    PHEROMONE FIELD       |
                                     |  (Styx -- wss://styx.bardo.run) |
                                     +--------+----------------+
                                              | read (threat/opportunity/wisdom signals)
                                              v
+--[Base L2 RPC]-+  probe results   +------------------------------------------+
|  MARKET DATA   |----------------->|            HEARTBEAT PIPELINE            |
+----------------+                  |                                          |
                                    |  1. OBSERVE --> 2. RETRIEVE --> 3. ANALYZE |
                                    |       |              |               |      |
                                    |       v              v               v      |
                                    |  MarketObs     Grimoire         Prediction  |
                                    |  + Probes      4-factor         Error       |
                                    |  + Pheromone   retrieval        Computation |
                                    |                                             |
                                    |  4. GATE ------------------------------------
                                    |       |                                    |
                                    |  +----+----+                              |
                                    |  | PE < th |---> T0: SUPPRESS (no LLM)    |
                                    |  | PE >= th|---> T1: ANALYZE (cheap LLM)  |
                                    |  | PE >> th|---> T2: DELIBERATE (full LLM)|
                                    |  +---------+                              |
                                    |       |                                    |
                                    |  5. SIMULATE --> 6. VALIDATE --> 7. EXECUTE |
                                    |       |              |               |      |
                                    |       v              v               v      |
                                    |    Revm sim     PolicyCage      Tool call   |
                                    |    (in-proc)    + Risk Engine   (+ opt. Warden) |
                                    |                                             |
                                    |  8. VERIFY --> 9. REFLECT                   |
                                    |       |              |                      |
                                    |       v              v                      |
                                    |  Ground truth   DecisionCycleRecord        |
                                    |  (tx receipt)   written to storage         |
                                    +------------------------------------------+
                                              |
                                    emit to Event Fabric
                                              |
                          +-------------------+-------------------+
                          |                   |                   |
                          v                   v                   v
                    +----------+       +----------+       +----------+
                    |   TUI    |       | Web/WS   |       | Telegram |
                    |(ratatui) |       |(Axum SSE)|       |  (bot)   |
                    +----------+       +----------+       +----------+
```

The key thing to notice: the LLM is only invoked at step 4 if the prediction error (PE) exceeds the adaptive threshold. Most ticks don't reach the LLM at all. The heartbeat FSM handles market observations, retrieval, and analysis using deterministic code. This is what "LLM-last" means in practice.

---

## Delegation Hierarchy

The Golem runtime orchestrates 32 agents in a four-level delegation DAG. This is not the same as the 7-layer extension hierarchy above (extensions are Rust code modules; agents are LLM-backed reasoning units).

The hierarchy:

- **Level 0 (Terminal, 12 agents)**: Trust anchors that never delegate to other agents. They invoke tools directly. Examples: `safety-guardian`, `risk-assessor`, `wallet-provisioner`, `pool-researcher`, `token-analyst`, `identity-verifier`, `position-monitor`, `vault-watchdog`, `hook-builder`, `integration-architect`, `integration-advisor`, `sleepwalker-observer`.
- **Level 1 (Execution, 3 agents)**: Invoke tools and may delegate to Level 0.
- **Level 2 (Strategy, 7 agents)**: Plan and orchestrate. Delegate to Level 1 and Level 0.
- **Level 3 (Orchestration, 3 agents)**: Coordinate high-level flows across the strategy agents.

Four invariants govern all delegation:
1. **Acyclic**: The DAG is topologically sorted at startup. Cycles are a startup error.
2. **Safety-first**: All writes flow through `safety-guardian`.
3. **No transitive trust**: Each agent independently verifies on-chain state rather than trusting what another agent reported.
4. **Terminal exclusivity**: Only Level 0 agents invoke tools directly.

> **Extended:** Full 32-agent DAG with category tables, delegation level matrix, DAG rules, and example deposit flow -- see [../../prd2-extended/01-golem/00-overview-extended.md](../../prd2-extended/01-golem/00-overview-extended.md)

> Cross-reference: Full agent definitions in `../07-tools/`. Safety architecture in `../10-safety/`. PolicyCage specification in `../10-safety/02-policy.md`.

---

## Environment Scoping

Every requirement in this PRD carries one of two labels:

- **[CORE]**: Required for demo/testnet. Works with 3 agents, completes in under 15 minutes, runs on a single machine. This is the MVP.
- **[HARDENED]**: Production hardening. Toggled on via configuration. Adds economic staking, population controls, formal verification, multi-party dispute resolution.

The boundary between [CORE] and [HARDENED] is a configuration toggle, not a code fork. The same binary runs both environments. This means you can develop and test against [CORE] locally, then flip a config flag for production deployment without changing any code.

---

## Storage Layout (Per Golem)

Each Golem has a data directory on its VM's filesystem. This is the authoritative state -- if the VM dies, this data plus the custody layer (Delegation session keys, Privy wallet, or local key -- see `13-custody.md`) constitute the Golem's full identity.

```
$GOLEM_DATA/
+-- cycles/                         # DecisionCycleRecords (bincode-serialized, one per tick)
|   +-- cycle-000001.bincode        # ~2-10KB each
|   +-- cycle-000002.bincode
|   +-- index.sqlite                # Tick index: tick -> regime, tier, has_action, phase
+-- grimoire/
|   +-- episodes.lance/             # LanceDB: episodic vectors + columnar metadata
|   +-- semantic.db                 # SQLite: semantic entries, emotion_log, causal_edges,
|   |                               #         staged_revisions, quarantine, immune_memory,
|   |                               #         playbook_snapshots
|   +-- PLAYBOOK.md                 # Procedural memory (filesystem, single-writer: Dream Integration)
+-- conversations/
|   +-- session.jsonl               # User chat history (minority use case)
+-- audit/
|   +-- chain.bin                   # Merkle hash-chain audit trail (tamper-evident)
|   +-- anchors.json                # On-chain audit anchor transaction hashes
+-- bloom/
|   +-- local_blooms.bin            # Bloom Oracle filter state for Styx query pre-filtering
+-- creature/
|   +-- state.json                  # Current creature visual state (for surface reconnection)
+-- config/
|   +-- golem.toml                  # GolemConfig (heartbeat interval, model routing, budget caps)
|   +-- STRATEGY.md                 # Owner-authored strategy specification
+-- death/
    +-- genome.bincode              # Genomic bottleneck output (written during Thanatopsis Phase III)
    +-- testament.json              # Death testament (structured reflection, written during Phase II)
```

Estimated storage per day at adaptive theta-frequency intervals (30-120s, regime-dependent): ~720-2,880 ticks/day x ~5KB/tick = ~4-14MB for cycle records, plus Grimoire growth. Total: ~20-50MB/day. Cycle records older than the Curator's lookback window (configurable, default 500 ticks) can be archived or pruned.

---

## Implementation Interfaces

This section shows the primary Rust type definitions. If you're reading this document for architectural context and don't need the struct layouts, skip to "Key Architectural Decisions" below.

### GolemConfig

The configuration struct loaded from TOML at startup. Every field is documented inline.

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemConfig {
    /// Unique identifier for this golem instance
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// ERC-8004 agent ID (U256 as string), if registered
    pub agent_id: Option<String>,
    /// Chain IDs this golem operates on
    pub chain_ids: Vec<u64>,
    /// Primary chain for vault operations
    pub primary_chain_id: u64,
    /// Wallet configuration
    pub wallet: WalletConfig,
    /// Heartbeat configuration
    pub heartbeat: HeartbeatConfig,
    /// Strategy references (paths or identifiers)
    pub strategies: Vec<String>,
    /// Memory/grimoire configuration
    pub memory: MemoryConfig,
    /// Deployment mode
    pub deployment: DeploymentMode,
    /// Golem phenotype -- activates a preconfigured capability bundle
    pub phenotype: Option<Phenotype>,
    /// Inference provider configuration
    pub inference: Option<InferenceConfig>,
    /// Logging configuration
    pub log: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub provider: WalletProvider,
    pub address: Option<Address>,
    /// Privy app ID, required when provider is Privy
    pub privy_app_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletProvider {
    Delegation,
    Privy,
    Local,
    Safe,
    ZeroDev,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Tick interval in milliseconds (range: 10_000..=300_000, default: 60_000)
    pub interval_ms: u64,
    /// Consecutive none-severity ticks before idle suppression (default: 3)
    pub idle_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// SQLite database path
    pub db_path: String,
    /// LanceDB directory path
    pub vector_path: String,
    /// Maximum episodes to retain (default: 10_000)
    pub max_episodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    Local,
    Fly,
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phenotype {
    Sleepwalker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Ordered provider list
    pub providers: Vec<InferenceProvider>,
    /// Payment mode
    pub payment: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceProvider {
    pub provider_type: String,
    pub api_key: Option<String>,
    pub diem: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// JSONL log file path
    pub path: String,
    /// Log level
    pub level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
```

### GolemManifest

The manifest is the full identity document for a Golem. It combines config with strategy and operational metadata. This is what gets serialized when a Golem is created, and what gets read back when it boots.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemManifest {
    /// Runtime config
    pub config: GolemConfig,
    /// Strategy definitions this golem can execute
    pub strategy_refs: Vec<StrategyRef>,
    /// Tool allowlist (tools this golem is permitted to call)
    pub allowed_tools: Vec<String>,
    /// Memory initialization parameters
    pub memory_init: MemoryInit,
    /// Creation metadata
    pub created: CreationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRef {
    /// Strategy identifier
    pub id: String,
    /// Strategy type
    pub strategy_type: StrategyType,
    /// Parameters for this strategy
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Lp,
    Vault,
    Trading,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInit {
    /// Seed episodes to bootstrap knowledge
    pub seed_episodes: Option<Vec<String>>,
    /// Inherited grimoire path (from parent, if replicated)
    pub inherit_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationMeta {
    pub timestamp: u64,
    /// Parent ID if replicated from another golem
    pub parent_id: Option<String>,
    pub method: CreationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreationMethod {
    Manual,
    Replication,
    Api,
}
```

### Type-State Lifecycle Machine

The Golem's lifecycle is encoded as a type-state machine. This is one of the more interesting Rust-specific design decisions: `Golem<Active>` has a `tick()` method; `Golem<Dead>` does not. Calling `tick()` on a dead Golem is a compile error, not a runtime check. The compiler itself enforces the mortality invariant.

```rust
use std::marker::PhantomData;

// Zero-sized type-state markers (no runtime cost)
pub struct Provisioning;
pub struct Active;
pub struct Dreaming;
pub struct Terminal;
pub struct Dead;

pub struct Golem<Phase> {
    state: GolemState,
    _phase: PhantomData<Phase>,
}

impl Golem<Provisioning> {
    pub async fn activate(self) -> Result<Golem<Active>> {
        // Validate config, connect wallet, load grimoire, register extensions
        Ok(Golem { state: self.state, _phase: PhantomData })
    }
}

impl Golem<Active> {
    pub async fn tick(&mut self) -> Result<DecisionCycleRecord> {
        // The 9-step heartbeat pipeline
        todo!()
    }

    pub fn enter_dream(self) -> Golem<Dreaming> {
        Golem { state: self.state, _phase: PhantomData }
    }

    pub fn begin_death(self) -> Golem<Terminal> {
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Dreaming> {
    pub async fn dream(&mut self) -> Result<()> { todo!() }

    pub fn wake(self) -> Golem<Active> {
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Terminal> {
    pub async fn thanatopsis(self) -> Result<Golem<Dead>> {
        // Four-phase death protocol: Acceptance -> Settlement -> Reflection -> Legacy
        Ok(Golem { state: self.state, _phase: PhantomData })
    }
}

// Golem<Dead> has no methods -- it is a tombstone.
```

Notice the transitions are consuming (`self`, not `&mut self`). `activate()` consumes `Golem<Provisioning>` and returns `Golem<Active>`. You can't accidentally hold onto a reference to the provisioning-phase Golem after activation. The ownership system enforces the state machine.

### Bootstrap Sequence

1. Load `GolemConfig` from TOML file or environment
2. Validate config (serde + custom validation)
3. Initialize wallet provider (connect or create)
4. Initialize memory store (SQLite + LanceDB)
5. Register extensions in topological order (7 layers)
6. Load and validate strategy references
7. Start heartbeat scheduler (tokio interval)
8. Emit `GolemEvent::LifecycleReady` to Event Fabric

### Crate Dependencies

```
golem-runtime depends on:
  golem-core        -- types, config, cortical state, event fabric, arena, taint
  golem-heartbeat   -- decision cycle pipeline
  golem-grimoire    -- knowledge storage and retrieval
  golem-daimon      -- affect engine
  golem-mortality   -- death clocks and lifecycle phases
  golem-dreams      -- dreaming engine
  golem-context     -- context engineering
  golem-safety      -- capability tokens, audit, policy enforcement
  golem-inference   -- LLM provider routing (three-tier)
  golem-chain       -- on-chain interaction via Alloy
  golem-tools       -- tool harness and registry
  golem-coordination -- Styx client, pheromone field, clade sync
```

---

## Technology Stack

| Layer           | Technology                                     | Rationale                                                                                                                                                                                                                                  |
| --------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Language        | Rust (edition 2024, `#![forbid(unsafe_code)]`) | Deterministic memory, compile-time safety, zero GC. The `forbid(unsafe_code)` directive is project-wide.                                                                                                                                   |
| Async runtime   | tokio                                          | Required by Axum (HTTP), LanceDB, and Alloy (EVM). All IO-bound work uses tokio tasks. CPU-bound work (embedding, scoring) uses `tokio::task::spawn_blocking`.                                                                             |
| EVM interaction | Alloy v1.0 (Paradigm)                          | Native Rust EVM toolkit. The `sol!` procedural macro generates type-safe bindings from Solidity interfaces. 60% faster U256 arithmetic than ethers.js.                                                                                     |
| EVM simulation  | Revm (Paradigm)                                | In-process EVM fork. Loads mainnet state on demand, simulates transactions without broadcasting. Used in heartbeat Step 5 (SIMULATE) for T2 deliberation.                                                                                  |
| Vector DB       | LanceDB                                        | Written in Rust. Columnar storage with Lance format. HNSW index for approximate nearest neighbor search. Predicate pushdown on metadata columns.                                                                                           |
| Relational DB   | rusqlite + sqlite-vec                          | Rust bindings for SQLite. WAL mode for concurrent reads. Serves semantic memory, causal graph (adjacency table), emotion log, staged revisions, quarantine, immune memory.                                                                 |
| Embeddings      | fastembed-rs                                   | In-process embedding using nomic-embed-text-v1.5. Matryoshka Representation Learning (MRL) allows dimension truncation (768 -- 384 -- 256) without retraining.                                                                             |
| HTTP server     | Axum                                           | Tokio-native. Used for the surface multiplexer: WebSocket endpoint for TUI/web, SSE for fallback, REST for state snapshots and steer/followUp ingress.                                                                                     |
| TUI             | ratatui                                        | Rich terminal rendering. Panels, charts, scrolling event logs, animated creature display. The primary owner interface during development.                                                                                                  |
| Serialization   | serde + bincode / serde_json                   | bincode for hot-path storage (DecisionCycleRecords: fast, compact). serde_json for interop (API responses, config files, Event Fabric payloads to surfaces).                                                                               |
| Secrets         | zeroize crate                                  | Types wrapping sensitive data (`Zeroizing<String>`) automatically overwrite memory on drop, preventing key recovery from memory dumps.                                                                                                     |
| WASM sandbox    | wasmtime                                       | Fuel metering (instruction count limits) + epoch interruption (wall-clock timeout). Applied only to untrusted third-party tools -- native Bardo tools run unsandboxed at full Rust speed.                                                  |
| Wallet          | Three custody modes (see `13-custody.md`)      | **Delegation** (recommended): MetaMask ERC-7710/7715, funds stay in owner's wallet, session keys with caveat enforcers. **Privy** (legacy): AWS Nitro Enclaves, signing via HTTP API. **Local key** (dev): bounded by on-chain delegation. |
| Deployment      | Fly.io                                         | One VM per Golem. Ephemeral compute with predictable billing. SIGTERM with 30-second grace period triggers the graceful shutdown protocol.                                                                                                 |
| Observability   | tracing + OpenTelemetry                        | Structured spans with DeFi-contextual fields (tick, regime, phase, tool). Prometheus metrics for operational monitoring.                                                                                                                   |
| Allocator       | bumpalo                                        | Per-tick arena allocator. All allocations within a tick use the arena; it resets at tick end. Eliminates memory fragmentation across weeks of continuous operation.                                                                        |

---

## Origin: Built From Scratch, Informed by pi_agent_rust

**Context for first-time readers:** Pi Agent is an open-source agentic framework by Mario Zechner (badlogic) that implements a well-designed extension/hook model for LLM-backed agents. `pi_agent_rust` is a Rust port by Jeff Emanuel (Dicklesworthstone) that re-implements the same architecture in Rust. We studied pi_agent_rust's design for provider-specific quirks (streaming format differences, auth edge cases) but build Golem-RS from scratch using published ecosystem crates. pi_agent_rust is a monolithic binary with no published crates, and forking it would mean inheriting its `asupersync` runtime plus 90+ provider integrations that Bardo doesn't need. The Rust decision itself follows from Pillar 4 above: a mortal agent on a micro VM cannot afford GC pauses, and the type-state lifecycle machine cannot exist in TypeScript.

Equivalent functionality is built fresh using the Rust ecosystem:

| Domain | Approach |
| --- | --- |
| Provider abstraction (5 providers: Anthropic, OpenAI, Google, Venice, Grok) | `reqwest` 0.12 + one provider trait with per-provider implementations. pi_agent_rust's 6k lines cover 90+ providers Bardo will never use. |
| SSE parsing | `eventsource-stream` or `reqwest-eventsource` crate. Custom fallback (~200-300 lines) only if edge cases demand it. |
| Auth | API key headers (trivial for 4 of 5 providers) + x402 signed USDC authorization (custom, not in pi_agent_rust). |
| Extension sandbox | Deferred. All 28 core extensions are native Rust. User-provided extensions will use `wasmtime` (already in the stack for tool sandboxing). |
| Session JSONL | `serde_json` + line-delimited I/O (~50-100 lines). |

Everything above this layer -- the 28 extensions, heartbeat FSM, Grimoire, Daimon, Dreams, Context Governor, Risk Engine, Mortality system, Creature System, Pheromone Field -- is purpose-built from the specifications in this document series.

**Key decision: tokio, not asupersync.** Golem-RS uses tokio because the rest of the ecosystem (Axum, LanceDB, Alloy, rusqlite's async wrapper) assumes it. We studied asupersync's structured concurrency patterns and adapted the useful ones to tokio equivalents (primarily `tokio::task::JoinSet` and `CancellationToken`).

---

## Key Architectural Decisions

These are the decisions that shape the system's architecture. Each one records what was decided, why, and what alternative was rejected.

### 1. DecisionCycleRecord Replaces Session Turns for the Autonomous Path

**Problem**: Traditional agent frameworks model cognition as a conversation: human message -- LLM response -- tool calls -- repeat. But 80% of a Golem's ticks have no human input. The heartbeat fires autonomously.

**Solution**: CoALA [SUMERS-2024] formalizes the correct primitive: a **decision cycle**, not a conversation turn. Each tick produces a typed `DecisionCycleRecord` containing the observation, emotional appraisal, gating decision, deliberation (if any), actions, outcome verification, knowledge written, and mortality state.

**Why it's better**: Dream replay selects episodes by `utility = gain x need` -- the record IS the episode, with observation, action, outcome, and emotional state already structured. Cybernetics self-tuning traces outcomes to context entries from `context_bundle_summary`. The lifespan extension reads mortality accounting directly. No parsing of LLM output text required.

**What stays**: Session JSONL is preserved for the minority case (user chatting with their Golem), stored separately in `conversations/session.jsonl`.

### 2. Cognitive Workspace Replaces Compaction

**Problem**: In session-based agents, the context grows until it hits the LLM's context window, then compaction summarizes older messages. This is reactive and lossy.

**Solution**: The Context Governor assembles a fresh **Cognitive Workspace** each tick from structured categories (invariants, strategy, retrieved knowledge, observations, affect) with learned token allocations. This implements Baddeley's working memory model [BADDELEY-2000]: central executive (Context Governor), episodic buffer (the Workspace), visuospatial sketchpad (observations), phonological loop (PLAYBOOK heuristics).

### 3. Unified Grimoire (LanceDB + SQLite + Filesystem)

**Problem**: The Grimoire needs vector similarity search (mood-congruent retrieval), SQL queries (confidence tracking, BFS graph traversal), and simple text file operations (PLAYBOOK.md). No single storage substrate serves all three well.

**Solution**: Each substrate serves its query patterns optimally. LanceDB for HNSW vector search. SQLite for indexed columns and graph queries. Filesystem for the single-writer PLAYBOOK.md.

**Why not CRDTs**: Evaluated and rejected. CRDT merge semantics (convergence -- both sides end up equal) contradict the inheritance model (asymmetric digestion with confidence discounts and validation-through-use). PLAYBOOK.md has exactly one writer; there is no concurrent editing scenario. The causal graph needs SQL-indexed BFS traversal that CRDTs cannot serve. Knowledge propagation uses export -- transmit -- ingest through the four-stage ingestion pipeline, preserving the Weismann barrier [HEARD-MARTIENSSEN-2014], the testing effect [ROEDIGER-KARPICKE-2006], and confidence discounting.

### 4. Type-State Lifecycle Machine

**Problem**: A dead Golem must never process another tick. A terminal Golem must not dream. These are critical invariants.

**Solution**: Rust's type system encodes lifecycle states as zero-sized types. `Golem<Active>` has a `tick()` method; `Golem<Dead>` does not. Calling `tick()` on a dead Golem is a compile error, not a runtime check. This extends to Thanatopsis phases, dream cycle phases, and ActionPermit states.

**Academic basis**: Dennis & Van Horn (1966) established capability-based security -- the principle that access rights should be unforgeable tokens checked at the type level rather than runtime guards [DENNIS-VAN-HORN-1966].

### 5. Event Fabric as Universal Observability

**Problem**: Internal state transitions (heartbeat ticks, emotional appraisals, dream phases, tool executions) need to be visible to multiple surfaces (TUI, web dashboard, Telegram, creature system) without coupling those surfaces to the internal implementation.

**Solution**: A non-blocking `tokio::sync::broadcast` channel with a 10,000-event ring buffer. Every subsystem emits typed events (50+ types across 16 subsystems). Surfaces subscribe to the categories they need. The Event Fabric never blocks the heartbeat pipeline -- if receivers lag, they miss events and can catch up via replay.

### 6. Pheromone Field for Coordination

**Problem**: Direct agent-to-agent messaging leaks strategy information. If Golem A tells Golem B about a Morpho utilization spike, B learns A's strategy involves Morpho. This is the Grossman-Stiglitz paradox [GROSSMAN-STIGLITZ-1980]: shared alpha destroys itself.

**Solution**: Stigmergic coordination [GRASSE-1959]. Golems deposit anonymous signals (threat class + intensity) into a shared field with three temporal layers (threat: 2h half-life, opportunity: 12h, wisdom: 7d). Signals decay exponentially; independently confirmed signals reinforce. No strategy information leaks. Coordination emerges from environmental traces, not communication.

### 7. Capability Tokens for Compile-Time Security

**Problem**: A compromised LLM prompt should not be able to execute unauthorized trades. Runtime permission checks are fallible -- they require correct implementation at every call site.

**Solution**: Rust's type system enforces tool permissions at compile time via `Capability<T>` tokens backed by `PhantomData`. `ReadTool` requires no capability. `WriteTool` requires a `Capability<WritePermission>` that can only be minted by the safety system after PolicyCage validation. The LLM never sees these tokens -- they exist only in the Rust type system.

---

## Cross-Lifetime Persistence via Styx

When a Golem dies, its knowledge doesn't disappear. It persists across three privacy layers provided by the Styx coordination service:

- **Vault layer**: Encrypted per-owner backup. AES-256-GCM, owner-held key. Survives Golem death for inheritance by successor Golems. Stored in Cloudflare R2.
- **Clade layer**: Shared within the owner's fleet of Golems. Confidence-discounted ingestion preserves the Weismann barrier (inherited knowledge starts at reduced confidence so the inheritor must validate it through use).
- **Lethe layer**: Public knowledge contributed to the collective. Death warnings (Bloodstains) carry costly signaling premium -- a dead Golem can't benefit from its own warning, so the information is maximally honest.

Styx is strictly additive. A Golem running without Styx connectivity retains ~95% of its capabilities -- only cross-Golem sync and the Pheromone Field are unavailable.

---

## References

- [JONAS-1966] Jonas, H. (1966). _The Phenomenon of Life_. Northwestern University Press. — *Argues that metabolism is the defining feature of life: self-sustaining through the very activity that requires sustenance. Grounds the Golem's mortality-as-feature thesis.*
- [LAU-2026] Lau, K. (2026). "On the Inconsistency of LLM-Based Scoring." arXiv preprint. _Note: Preprint, not yet peer-reviewed._ — *Demonstrates that LLM-based scoring is provably inconsistent, motivating Bardo's use of deterministic on-chain formulas for performance evaluation.*
- [HALDAR-HOCKENMAIER-2025] Haldar, S. & Hockenmaier, J. (2025). "LLM Judges Are Not Reliable." — *Reinforces the case against LLM-based evaluation, supporting Bardo's deterministic scoring design.*
- [FRUGALGPT-2023] Chen, L. et al. (2024). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." _TMLR_. — *Demonstrates matching GPT-4 performance with up to 98% cost reduction through intelligent cascading; validates the T0/T1/T2 gating approach.*
- [SEAGENT-2025] Ye, Q. et al. (2025). "SEAgent: Confused Deputy Attacks in Software Engineering Agents." arXiv preprint. — *Catalogs confused deputy attack vectors in agent frameworks; motivates the capability token and taint tracking defenses.*
- [MERITRANK-2024] MeritRank: Transitivity Decay for Sybil-Resistant Reputation. 2024. — *Proposes transitivity decay for reputation systems; informs ERC-8004 reputation scoring design.*
- [RAY-1991] Ray, T.S. "An Approach to the Synthesis of Life." _Artificial Life II_, SFI Studies in the Sciences of Complexity, 1991. — *The Tierra system: self-replicating programs evolve complex behaviors only when a reaper kills the oldest, demonstrating that mortality drives evolutionary innovation.*
- [LENSKI-2003] Lenski, R.E., Ofria, C., Pennock, R.T. & Adami, C. "The Evolutionary Origin of Complex Features." _Nature_, 423, 2003. — *Shows digital organisms evolve complex logic through stepping-stone mutations requiring population turnover; supports the mortality-creates-intelligence thesis.*
- [VOSTINAR-2019] Vostinar, A.E., Goldsby, H.J. & Ofria, C. "Suicidal Selection: Programmed Cell Death Can Evolve in Unicellular Organisms Due Solely to Kin Selection." _Artificial Life_, 25(4), 2019. — *Demonstrates that programmed death evolves as adaptive behavior benefiting kin; parallels Golem death protocols that benefit the Clade.*
- [KREPS-MILGROM-ROBERTS-WILSON-1982] Kreps, D.M., Milgrom, P., Roberts, J. & Wilson, R. "Rational Cooperation in the Finitely Repeated Prisoners' Dilemma." _Journal of Economic Theory_, 27(2), 1982. — *Proves that agents with uncertain finite horizons cooperate more effectively than infinite-horizon agents; grounds the claim that mortal Golems are structurally more trustworthy.*
- [DAMASIO-1994] Damasio, A. _Descartes' Error: Emotion, Reason, and the Human Brain._ Putnam, 1994. — *Establishes the somatic marker hypothesis: emotions function as rapid heuristic signals that bias advantageous decisions before conscious deliberation. Foundational for the Daimon affect engine.*
- [BECHARA-2000] Bechara, A., Damasio, H. & Damasio, A. "Emotion, Decision Making and the Orbitofrontal Cortex." _Cerebral Cortex_, 10(3), 2000. — *Shows that emotional biases anticipate correct choices before conscious awareness; validates affect-driven decision gating.*
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. — *Establishes mood-congruent memory: emotional states bias which memories are retrieved. Directly implemented in the Grimoire's four-factor retrieval scoring.*
- [EMOTIONAL-RAG-2024] Zhang, Y. et al. "Emotional RAG: Strengthening LLM-based Agent Emotion via Emotional RAG." arXiv:2410.23041, 2024. — *Validates computationally that emotion-tagged retrieval outperforms non-emotional retrieval across multiple datasets.*
- [RUSSELL-MEHRABIAN-1977] Russell, J.A. & Mehrabian, A. "Evidence for a Three-Factor Theory of Emotions." _Journal of Research in Personality_, 11(3), 1977. — *Proposes the PAD (Pleasure-Arousal-Dominance) model of affect; the formal basis for the Daimon's emotional state representation.*
- [WILSON-MCNAUGHTON-1994] Wilson, M.A. & McNaughton, B.L. "Reactivation of Hippocampal Ensemble Memories During Sleep." _Science_, 265, 1994. — *Demonstrates that hippocampal neurons replay waking experiences during sleep; the biological basis for Golem NREM dream replay.*
- [BUZSAKI-2015] Buzsaki, G. "Hippocampal Sharp Wave-Ripple: A Cognitive Biomarker for Episodic Memory and Planning." _Neuron_, 85(2), 2015. — *Shows that sharp-wave ripples compress minutes of experience into ~100ms bursts for consolidation; inspires the Golem's dream compression pipeline.*
- [WALKER-VAN-DER-HELM-2009] Walker, M.P. & van der Helm, E. "Overnight Therapy? The Role of Sleep in Emotional Brain Processing." _Psychological Bulletin_, 135(5), 2009. — *The SFSR (Sleep to Forget, Sleep to Remember) model: sleep strips emotional valence while preserving informational content. Basis for dream depotentiation.*
- [WAGNER-2004] Wagner, U., Gais, S., Haider, H., Verleger, R. & Born, J. "Sleep Inspires Insight." _Nature_, 427, 2004. — *Shows that sleeping subjects are 2.6x more likely to discover hidden rules in data; motivates the Golem dream cycle's insight generation phase.*
- [HAFNER-DREAMERV3-2025] Hafner, D., Pasukonis, J., Ba, J. & Lillicrap, T. "Mastering Diverse Domains through World Models." _Nature_, 2025. — *Proves agents trained in imagined forward trajectories from learned world models outperform model-free agents; validates the REM dream phase's counterfactual scenario generation.*
- [HA-SCHMIDHUBER-2018] Ha, D. & Schmidhuber, J. "World Models." arXiv:1803.10122, 2018. — *Shows a controller trained inside hallucinated dreams achieves competitive performance with far less real-world data; foundational for dream-based learning.*
- [SUMERS-2024] Sumers, T.R., Yao, S., Narasimhan, K. & Griffiths, T.L. "Cognitive Architectures for Language Agents." _Transactions on Machine Learning Research_, 2024. — *Proposes the CoALA framework formalizing what a language agent is: modular memory, structured actions, and decision cycles. The academic framework Bardo's cognition maps to.*
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" _Trends in Cognitive Sciences_, 4(11), 2000. — *Adds the episodic buffer to the working memory model; the theoretical basis for the Cognitive Workspace assembled fresh each tick.*
- [FRISTON-2010] Friston, K. "The Free-Energy Principle: A Unified Brain Theory?" _Nature Reviews Neuroscience_, 11(2), 2010. — *Proposes precision-weighted prediction error as the brain's organizing principle; directly implemented in the heartbeat's gating step (Step 4).*
- [GRASSE-1959] Grasse, P.-P. "La Reconstruction du Nid et les Coordinations Interindividuelles chez Bellicositermes Natalensis et Cubitermes sp." _Insectes Sociaux_, 6, 1959. — *Describes stigmergy: indirect coordination through environmental signals. The biological basis for the Pheromone Field.*
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." _American Economic Review_, 70(3), 1980. — *Argues that perfectly efficient markets are impossible because information acquisition has costs; supports the economic viability of informed Golem agents.*
- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. "Programming Semantics for Multiprogrammed Computations." _Communications of the ACM_, 9(3), 1966. — *Foundational work on capability-based security; the theoretical basis for Golem capability tokens that enforce tool permissions at compile time.*
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance: Myths and Mechanisms." _Cell_, 157(1), 2014. — *Analyzes the Weismann barrier separating germline from soma; the biological model for confidence decay on inherited knowledge.*
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention." _Psychological Science_, 17(3), 2006. — *Demonstrates the testing effect: retrieval practice strengthens memory more than re-study. Motivates the Grimoire's revalidation-based confidence system.*
- [JANEWAY-2001] Janeway, C.A., Travers, P., Walport, M. & Shlomchik, M.J. _Immunobiology: The Immune System in Health and Disease._ Garland, 5th ed., 2001. — *The canonical immunology reference; informs the Grimoire's adaptive immune defense against knowledge poisoning attacks.*
- [PEARL-2009] Pearl, J. _Causality: Models, Reasoning, and Inference._ Cambridge University Press, 2nd ed., 2009. — *Formalizes causal reasoning with do-calculus; the theoretical basis for the Grimoire's causal graph and federated causal edge sharing.*
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." _Journal of Monetary Economics_, 50(3), 2003. — *Formalizes rational inattention: information processing is costly, so agents optimally allocate attention. Grounds mortality-aware attention budgeting.*
- [BORGES-1942] Borges, J.L. "Funes the Memorious." _Ficciones_, 1944. — *A man who remembers everything and understands nothing; the cautionary tale motivating the Grimoire's forgetting-as-feature design.*
- [STICKGOLD-WALKER-2013] Stickgold, R. & Walker, M.P. "Sleep-Dependent Memory Triage: Evolving Generalization Through Selective Processing." _Nature Neuroscience_, 16, 2013. — *Proposes that sleep actively triages memories for generalization vs. forgetting; the neuroscience model for the Curator's consolidation pipeline.*

---

_"The body should be worthy of the soul it carries."_

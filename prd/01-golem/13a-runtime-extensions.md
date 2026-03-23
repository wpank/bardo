# Runtime Extensions -- Part A [SPEC]

**Extension Architecture, Registry, GolemState, Interventions, Session Tree, Tool Authorization, Model Routing, Adaptive Clock**

> **Version**: 3.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-core` (cortical_state.rs, event_fabric.rs, arena.rs, taint.rs), `golem-runtime` (extension.rs, registry.rs, dispatch.rs, state.rs, lifecycle.rs, shutdown.rs)
>
> **Prerequisites**: Read `00-architecture.md` for the glossary, thesis, and system overview.
>
> Sources: `03-agent-runtime/00-runtime-primitives`, `03-agent-runtime/06-event-wakeup`, `03-agent-runtime/08-metrics-tracing`

> **Reader orientation:** This is Part A of the runtime extensions specification. It covers the native Rust extension architecture that forms the skeleton of the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) runtime: the Extension trait (20 lifecycle hooks), the 28 concrete extensions organized into 7 dependency layers, the ExtensionRegistry, GolemState, Interventions, the Session Tree (conversation as a Merkle hash-chained tree), Tool Authorization (three-tier capability tokens), Model Routing (T0/T1/T2 cognitive tiers), and the Adaptive Clock (gamma/theta/delta oscillators). Part B is in `13b-runtime-extensions.md` (type-state machine, event fabric, CorticalState, arena allocator, shutdown). See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What This Document Covers

Every subsystem in the Golem runtime -- heartbeat, memory, affect, dreams, risk, mortality, coordination, inference -- is implemented as a native Rust **extension**: a struct that hooks into the Golem's lifecycle at defined points. There is no plugin system, no scripting sandbox, no foreign function interface. Extensions are compiled into the binary and dispatched through a trait-object registry.

This document specifies:

1. Why a native Rust extension architecture (not a plugin/Pi runtime)
2. The `Extension` trait and its 20 lifecycle hooks
3. The 28 concrete extensions organized into 7 dependency layers
4. The `ExtensionRegistry` that validates and dispatches them
5. `GolemState` -- the mutable struct flowing through the extension chain
6. Type-state lifecycle enforcement via phantom types
7. The Event Fabric (`tokio::broadcast` ring buffer)
8. The CorticalState (lock-free atomics for real-time perception reads)
9. The arena allocator (`bumpalo` per-tick allocation)
10. The 10-phase graceful shutdown protocol
11. The main binary and startup sequence

---

## 1. Why Native Rust Extensions

The Golem runtime inherits its extension model from [pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust), itself a Rust port of the Pi Agent framework. Pi's core insight was correct: an agent runtime should define WHEN things happen (lifecycle hooks) but not WHAT happens (extension implementations). What changes is the implementation substrate.

Pi ran extensions in a QuickJS sandbox -- JavaScript executing inside a JavaScript runtime, with serialization boundaries at every hook invocation. The Golem runtime runs extensions as native Rust structs implementing a trait. The `Extension` trait replaces the plugin boundary entirely. The compiler enforces what a sandbox can only check at runtime:

- **Composition over inheritance.** Each subsystem implements only the hooks it needs. The heartbeat extension overrides `on_after_turn`; the safety extension overrides `on_tool_call`. Neither knows or cares about the other's implementation. This is Rust's trait system doing what it does best -- defining shared behavior contracts without coupling implementations.

- **Zero-cost dispatch.** When the registry fires `on_after_turn` across 9 extensions in sequence, each call is a vtable lookup on a `dyn Extension` trait object. No serialization, no sandbox boundary crossing, no GC pause between extensions. The data flows through `&mut GolemState` directly.

- **Compile-time safety.** The type-state lifecycle machine (Section 6) makes invalid state transitions impossible to write. A `Golem<Dead>` has no `tick()` method. A `Golem<Provisioning>` has no `steer()` method. These are not runtime checks -- they are compilation failures. Pi enforced these with runtime guards (`if phase === "dead" return`); Rust enforces them with the type system.

- **Ordered data flow.** Extensions fire in a defined sequence within each hook. The Daimon reads the heartbeat's probe results. The memory extension reads the Daimon's appraisal. The dream scheduler reads both. This is a carefully designed data dependency chain, and it works because `&mut GolemState` guarantees single-writer semantics -- no data races, no stale reads, no locks.

- **Testability.** Each extension can be tested in isolation with a mock `GolemState`. The full system can be tested by registering a subset of extensions. No sandbox setup, no IPC mocking.

The model is analogous to middleware in web frameworks (Express, Axum), but richer: middleware chains are linear, while the extension system supports a DAG of dependencies across multiple hook types.

---

## 2. The Extension Trait: 20 Lifecycle Hooks

Every extension implements the `Extension` trait. All 20 hooks have default no-op implementations, so extensions only override the hooks they use. The hooks are grouped into six categories that correspond to the phases of a cognitive cycle.

```rust
use async_trait::async_trait;
use anyhow::Result;

/// The Extension trait defines 20 lifecycle hooks.
///
/// Each hook fires at a specific point in the Golem's cognitive cycle.
/// Extensions implement only the hooks they need -- all others default to no-ops.
///
/// Extensions are registered with:
/// - A unique name (e.g., "heartbeat", "daimon")
/// - A layer number (0-7) that determines dependency ordering
/// - An optional list of dependencies (extensions that must fire before this one)
///
/// The extension system is adapted from Pi Agent's hook architecture for
/// Rust's ownership model. The key difference: Pi extensions ran in a
/// QuickJS sandbox (JavaScript); Golem extensions run as native Rust with
/// zero overhead.
#[async_trait]
pub trait Extension: Send + Sync + 'static {
    /// Unique identifier for this extension (e.g., "heartbeat", "daimon").
    /// Used for dependency resolution and debugging.
    fn name(&self) -> &str;

    /// Layer in the dependency graph (0 = foundation, 7 = recovery).
    /// Extensions in higher layers may depend on extensions in lower layers,
    /// but never the reverse. This ensures the dependency graph is acyclic.
    fn layer(&self) -> u8;

    /// Names of extensions that must fire before this one in shared hooks.
    /// Dependencies must be in the same or lower layer.
    fn depends_on(&self) -> &[&str] { &[] }
```

### Category 1: Session Lifecycle (hook 1)

These hooks fire when the Golem's overall session begins, resumes, or undergoes structural changes.

```rust
    /// SESSION LIFECYCLE
    ///
    /// Fires on session-level events:
    /// - `SessionReason::Start`: Golem just booted. Extensions initialize.
    /// - `SessionReason::Resume`: Golem resuming from a checkpoint.
    /// - `SessionReason::BeforeCompact`: Context compaction about to occur
    ///   (conversation sidecar only).
    /// - `SessionReason::BeforeBranch`: Conversation branch about to be created.
    ///
    /// Used by: grimoire (initialize storage), heartbeat (load FSM state),
    ///          audit (open chain).
    async fn on_session(
        &self,
        _reason: SessionReason,
        _ctx: &mut SessionCtx,
    ) -> Result<()> {
        Ok(())
    }
```

### Category 2: Input Processing (hook 2)

When input arrives from any source -- user message, owner steer, system event -- this hook can transform, route, or suppress it.

```rust
    /// INPUT PROCESSING
    ///
    /// Fires when external input arrives. The extension can:
    /// - `InputAction::Pass`: Let the input through unchanged.
    /// - `InputAction::Transform(new_msg)`: Modify the input.
    /// - `InputAction::Suppress`: Block the input entirely.
    ///
    /// Used by: input-router (classify as steer/followUp/message and route
    ///          accordingly).
    async fn on_input(
        &self,
        _msg: &mut InputMessage,
        _ctx: &InputCtx,
    ) -> Result<InputAction> {
        Ok(InputAction::Pass)
    }
```

### Category 3: Agent Lifecycle (hooks 3-4)

These hooks bracket the LLM invocation. "Agent" here means the LLM reasoning session, not the Golem itself.

```rust
    /// AGENT LIFECYCLE -- BEFORE START
    ///
    /// Fires before the LLM begins processing. This is where context assembly
    /// happens: the context extension builds the Cognitive Workspace, the compiler
    /// extension injects STRATEGY.md, the playbook extension injects PLAYBOOK.md.
    ///
    /// Used by: context (assemble Cognitive Workspace), compiler (parse STRATEGY.md),
    ///          tools (register available tools for this tier), model-router (select
    ///          model), safety (enforce phase restrictions), x402-payment (prepare
    ///          micropayment).
    async fn on_before_agent_start(
        &self,
        _ctx: &mut AgentStartCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// AGENT LIFECYCLE -- AFTER START
    ///
    /// Fires after the agent is initialized but before it generates its first
    /// response. Used for logging and metrics.
    async fn on_agent_start(&self, _ctx: &AgentStartCtx) -> Result<()> {
        Ok(())
    }
```

### Category 4: Turn Lifecycle (hooks 5-11)

A "turn" is one LLM request-response cycle. Within a single tick, there may be zero turns (T0: no LLM call), one turn (T1: analyze), or multiple turns (T2: deliberate, then tool calls, then continuation).

```rust
    /// TURN START -- before the LLM generates a response.
    async fn on_turn_start(&self, _ctx: &TurnStartCtx) -> Result<()> {
        Ok(())
    }

    /// CONTEXT ASSEMBLY -- modify the messages array before sending to the LLM.
    /// This is where the turn-context extension injects phase-appropriate context
    /// subsets (progressive disclosure per the Context Governor's policy).
    async fn on_context(
        &self,
        _messages: &mut Vec<AgentMessage>,
        _ctx: &ContextCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// BEFORE PROVIDER REQUEST -- last chance to modify the LLM request.
    /// The model-router sets the model; x402-payment attaches the micropayment;
    /// the safety extension checks for tainted data entering the context.
    async fn on_before_provider_request(
        &self,
        _ctx: &mut ProviderReqCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// TOOL CALL -- the LLM has proposed a tool call.
    ///
    /// This is the safety checkpoint. Extensions can:
    /// - `ToolAction::Allow`: Let the call proceed.
    /// - `ToolAction::Block(reason)`: Reject the call. The LLM receives the
    ///   rejection.
    /// - `ToolAction::Modify(new_call)`: Alter the call (e.g., reduce amount).
    ///
    /// Firing order: safety -> permits -> risk -> result-filter.
    /// Each layer can block. The most restrictive wins.
    ///
    /// Used by: safety (PolicyCage check), permits (ActionPermit lifecycle),
    ///          risk (five-layer risk assessment), loop-guard (repetition
    ///          detection).
    async fn on_tool_call(
        &self,
        _call: &ToolCall,
        _ctx: &mut ToolCallCtx,
    ) -> Result<ToolAction> {
        Ok(ToolAction::Allow)
    }

    /// TOOL EXECUTION lifecycle -- three hooks for progress tracking.
    /// Multi-step tools (e.g., LP rebalance: simulate -> build calldata ->
    /// broadcast -> confirm) emit progress events at each step.
    async fn on_tool_execution_start(
        &self,
        _ctx: &ToolExecCtx,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_tool_execution_update(
        &self,
        _ctx: &ToolExecCtx,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_tool_execution_end(
        &self,
        _ctx: &ToolExecCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// TOOL RESULT -- the tool has returned a result.
    /// Extensions can transform or redact the result before the LLM sees it.
    ///
    /// Used by: result-filter (redact sensitive data, compress verbose output,
    ///          capture ground truth for OutcomeVerification).
    async fn on_tool_result(
        &self,
        _result: &mut ToolResult,
        _ctx: &ToolResultCtx,
    ) -> Result<()> {
        Ok(())
    }
```

### Category 5: Post-Turn Learning (hooks 12-14)

After the LLM has responded and any tool calls have completed, these hooks handle learning, bookkeeping, and state evolution. The `on_after_turn` hook is the critical path -- it's where the nine subsystems fire in sequence.

```rust
    /// TURN END -- the LLM has produced a response for this turn.
    async fn on_turn_end(&self, _ctx: &TurnEndCtx) -> Result<()> {
        Ok(())
    }

    /// AGENT END -- the LLM session is closing.
    async fn on_agent_end(&self, _ctx: &AgentEndCtx) -> Result<()> {
        Ok(())
    }

    /// AFTER TURN -- the post-processing hook.
    ///
    /// This is where the nine core subsystems fire in a strict sequence,
    /// each reading what the previous wrote to GolemState:
    ///
    ///   heartbeat -> lifespan -> daimon -> memory -> risk ->
    ///   dream -> cybernetics -> clade -> telemetry
    ///
    /// The ordering is intentional and non-negotiable:
    /// 1. Heartbeat evaluates probes and writes observation/regime data.
    /// 2. Lifespan ticks mortality clocks, using heartbeat's cost data.
    /// 3. Daimon runs emotional appraisal, using heartbeat's observation
    ///    and lifespan's mortality state.
    /// 4. Memory writes episodes to Grimoire, using Daimon's emotional tags.
    /// 5. Risk assesses current position danger, using all of the above.
    /// 6. Dream checks if dreaming is warranted, using emotional load
    ///    and novelty metrics from all prior extensions.
    /// 7. Cybernetics self-tunes the Context Governor, correlating outcomes
    ///    with the context entries that produced them.
    /// 8. Clade syncs with siblings and reads the Pheromone Field.
    /// 9. Telemetry emits final metrics and cost tracking.
    ///
    /// Each extension in this chain receives a mutable reference to
    /// GolemState. It reads fields set by earlier extensions and writes
    /// fields that later extensions will read. This is a pipeline,
    /// not parallel execution.
    async fn on_after_turn(
        &self,
        _ctx: &mut AfterTurnCtx,
    ) -> Result<()> {
        Ok(())
    }
```

### Category 6: System Hooks (hooks 15-20)

```rust
    /// SYSTEM PROMPT -- modify the system prompt before the LLM sees it.
    /// The playbook extension injects PLAYBOOK.md + survival state here.
    async fn on_system_prompt(
        &self,
        _prompt: &mut String,
        _ctx: &PromptCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// STEER -- a mid-execution interrupt has arrived.
    /// Steers are high-priority interventions that can preempt the current tick.
    /// The semantics: "change what the Golem is doing RIGHT NOW."
    async fn on_steer(
        &self,
        _msg: &SteerMessage,
        _ctx: &mut SteerCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// SEND MESSAGE -- an outbound message is being sent to a surface.
    /// The intervention extension intercepts this to log owner communications.
    async fn on_send_message(
        &self,
        _msg: &OutboundMessage,
        _ctx: &MsgCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// DEBUG -- debug information requested (e.g., owner asks "what are you
    /// thinking?").
    async fn on_debug(&self, _ctx: &DebugCtx) -> Result<()> {
        Ok(())
    }

    /// ERROR -- an error occurred somewhere in the pipeline.
    /// Extensions can log, alert, or attempt recovery.
    async fn on_error(
        &self,
        _err: &GolemError,
        _ctx: &ErrorCtx,
    ) -> Result<()> {
        Ok(())
    }

    /// END -- the Golem is shutting down (graceful or forced).
    /// Extensions flush state, sync to Styx, and release resources.
    async fn on_end(&self, _ctx: &EndCtx) -> Result<()> {
        Ok(())
    }
}
```

---

## 3. The 28 Extensions

Each extension is a Rust struct implementing the `Extension` trait. They are organized into seven dependency layers, from foundation (layer 0) through recovery (layer 7).

### Layer 0 -- Foundation (6 extensions)

These have no dependencies. They provide the infrastructure that everything else builds on.

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 1 | `provider-adapter` | `before_agent_start` | Routes LLM calls through the inference provider abstraction. Extracted from pi_agent_rust, supports 90+ providers. Handles auth, streaming, error recovery, and failover. |
| 2 | `telemetry` | `after_turn`, `tool_call` | Emits OpenTelemetry spans and Prometheus metrics. Tracks per-tick cost (inference + gas), latency percentiles, tool call success rates, and cognitive tier distribution. |
| 3 | `audit` | `tool_call`, `after_turn`, `session(before_compact)` | Writes every action to the Merkle hash-chain audit trail. Each entry contains a SHA-256 hash of the previous entry, creating a tamper-evident forensic log. Supports periodic on-chain anchoring. |
| 4 | `grimoire` | `session(start)`, `before_agent_start` | Initializes the Grimoire (LanceDB + SQLite + PLAYBOOK.md). On session start, opens or creates the storage files. On before_agent_start, preloads the embedding model and decision cache. |
| 5 | `tools` | `before_agent_start` | Loads DeFi tools from the configured profile (active: ~423 tools, observatory: ~72 read-only tools, conservative: limited writes). Registers tools with the tool registry and sets up the TypeScript sidecar connection if needed. |
| 6 | `heartbeat` | `session(start)`, `after_turn` | The autonomous decision cycle engine. On session start, initializes the heartbeat FSM. On after_turn, runs the 9-step pipeline (observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect). See `02a-heartbeat-pipeline.md`. |

### Layer 1 -- Input (1 extension)

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 7 | `input-router` | `input` | Classifies incoming messages as user messages, owner steers, followUps, or system events. Routes steers to the interrupt handler; queues followUps for the next decision window. Depends on heartbeat for FSM state. |

### Layer 2 -- State (4 extensions)

These manage the Golem's core state: knowledge, affect, mortality.

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 8 | `context` | `before_agent_start` | The Context Governor. Assembles the Cognitive Workspace from structured categories (invariants, strategy, retrieved knowledge, observations, affect) with learned token allocations. Implements Baddeley's working memory model [BADDELEY-2000]. |
| 9 | `daimon` | `after_turn` | The affect engine. Runs OCC/Scherer appraisal on the current observation, computes PAD vector and Plutchik label, checks somatic markers, queries the Somatic Landscape, and writes the result to the CorticalState for zero-latency reads by other fibers. |
| 10 | `memory` | `after_turn` | Episode writer and Curator. Writes the current tick's observation/action/outcome as an episodic entry in LanceDB. Runs the Curator cycle every 50 ticks (validate, prune, compress, cross-reference). Enforces the Admission Gate quality threshold. |
| 11 | `lifespan` | `after_turn`, `on_end` | Ticks the three mortality clocks (economic, epistemic, stochastic). Computes VitalityState and BehavioralPhase. Triggers phase transitions. On death, initiates the Thanatopsis protocol. |

### Layer 3 -- Safety (8 extensions)

The safety layer intercepts tool calls, validates constraints, and manages risk.

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 12 | `safety` | `tool_call`, `before_agent_start` | PolicyCage enforcement. Reads on-chain constraints (approved assets, max positions, max drawdown) via Alloy and blocks any tool call that would violate them. Also enforces behavioral phase restrictions (e.g., Conservation phase blocks new position entry). Performs taint checking on data entering LLM context. |
| 13 | `permits` | `tool_call` | ActionPermit lifecycle. Write tools require a permit (created -> committed -> expired/cancelled/blocked). The permit records the capability token hash, value limit, and PolicyCage constraint snapshot for audit. |
| 14 | `risk` | `tool_call`, `after_turn` | Five-layer risk assessment: Layer 1 (Hard Shields / PolicyCage), Layer 2 (Kelly Sizing -- optimal position sizing under uncertainty), Layer 3 (Adaptive Guardrails -- learned from the Golem's own outcomes), Layer 4 (Observation -- monitor for emerging threats), Layer 5 (DeFi-specific threats -- MEV, oracle manipulation, flash loan attacks). |
| 15 | `result-filter` | `tool_result` | Redacts sensitive data from tool results before the LLM sees them (e.g., wallet addresses, internal metrics). Compresses verbose output to fit context budgets. Captures ground truth (actual transaction outcomes) for the OutcomeVerification system. |
| 16 | `coordination` | `tool_call` | Handles cross-agent tool calls using ERC-8001/8033/8183 typed interfaces. These are on-chain coordination primitives -- Golems don't chat, they interact through typed transactions. |
| 17 | `compiler` | `before_agent_start` | Parses STRATEGY.md into typed `StrategyParams`. The owner writes STRATEGY.md in natural language or structured markdown; the compiler extracts risk bounds, target protocols, and trigger conditions into the format the heartbeat pipeline consumes. |
| 18 | `model-router` | `before_agent_start` | Three-tier model routing. Based on the cognitive tier from the gating step: T0 -> no model (skip), T1 -> Haiku-class (~$0.001/call), T2 -> Opus-class (~$0.10/call). Implements FrugalGPT's adaptive routing principle [CHEN-2023]. |
| 19 | `x402-payment` | `before_provider_request` | Attaches x402 micropayment authorization to each LLM request. Signs a USDC `transferWithAuthorization` via Alloy. The Golem's wallet IS its API key -- no separate billing accounts. |

### Layer 4 -- Cognition (4 extensions)

Higher-order cognitive processes that depend on the state and safety layers.

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 20 | `turn-context` | `context` | Injects turn-phase-aware context subsets. Progressive disclosure: the LLM doesn't see everything at once. During observation, it sees market data. During deliberation, it sees the full Cognitive Workspace. Based on the Context Governor's learned policy [BADDELEY-2000]. |
| 21 | `dream` | `after_turn` | The DreamScheduler decides whether to enter a dream cycle (based on time since last dream, emotional load, and novelty). If dreaming, transitions the Golem to the Dreaming state and runs the three-phase cycle: NREM replay, REM imagination, Consolidation. Also manages background micro-consolidation. |
| 22 | `cybernetics` | `after_turn` | Three self-tuning loops that adapt the Context Governor over time. Loop 1 (per-tick): correlate which context entries appeared in the workspace with outcome quality. Loop 2 (per-Curator cycle): adjust token allocations across categories. Loop 3 (per-regime change): partially reset correlations and re-adapt. |
| 23 | `clade` | `after_turn` | Peer-to-peer Grimoire synchronization within the owner's clade. Exports entries with `propagation >= Clade`, transmits to siblings, receives their exports, and ingests through the four-stage pipeline (quarantine, consensus, sandbox, adopt). Also gossips Bloom Oracle filters and reads the Pheromone Field. |

### Layer 5 -- UX (2 extensions)

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 24 | `ui-bridge` | `agent_start`, `agent_end`, `turn_start`, `turn_end`, `tool_execution_*` | Translates hook firings into Event Fabric events. This is the bridge between the internal pipeline and external surfaces. Every GolemEvent type (50+ across 16 subsystems) originates from this extension mapping an internal state transition to a typed event payload. |
| 25 | `observability` | `debug`, `error` | Structured logging with DeFi-contextual fields (tick, regime, phase, tool, cost). Error capture with context. Prometheus metric updates for operational dashboards. |

### Layer 6 -- Intervention (2 extensions)

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 26 | `playbook` | `system_prompt` | Injects PLAYBOOK.md and survival state (mortality phase, vitality, active warnings) into the LLM's system prompt. The LLM reads PLAYBOOK heuristics as reasoning scaffolding. |
| 27 | `intervention` | `send_message`, `steer` | Manages the owner steer/followUp lifecycle. Steers preempt the current tick. FollowUps are queued for the appropriate decision window (NextDecide, NextReflect, NextCurator, NextDream). Self-steers from the risk daemon enter through this same pathway. |

### Layer 7 -- Recovery (1 extension)

| # | Name | Hooks Used | What It Does |
|---|------|-----------|--------------|
| 28 | `compaction` | `session(before_compact)`, `session(before_branch)` | DeFi-aware compaction for the conversation sidecar. Preserves critical invariants during compaction: PolicyCage constraint hash, current position state, behavioral phase, active warnings, somatic state. Only relevant when the user is chatting with the Golem -- the heartbeat path uses the Cognitive Workspace instead of compaction. |

### Hook Coverage Matrix

| Hook | Extensions (firing order) |
|---|---|
| `session(start)` | `grimoire` -> `heartbeat` |
| `input` | `input-router` |
| `before_agent_start` | `tools` -> `context` -> `compiler` -> `model-router` -> `safety` |
| `agent_start` | `ui-bridge` |
| `turn_start` | `ui-bridge` |
| `context` | `turn-context` |
| `before_provider_request` | `x402-payment` |
| `tool_call` | `telemetry` -> `safety` -> `risk` -> `permits` -> `coordination` |
| `tool_execution_start` | `ui-bridge` |
| `tool_execution_update` | `ui-bridge` |
| `tool_execution_end` | `ui-bridge` |
| `tool_result` | `result-filter` |
| `turn_end` | `ui-bridge` |
| `agent_end` | `ui-bridge` |
| `after_turn` | `heartbeat` -> `lifespan` -> `daimon` -> `memory` -> `risk` -> `dream` -> `cybernetics` -> `clade` -> `telemetry` |
| `session(before_compact)` | `compaction` -> `audit` |
| `session(before_branch)` | `compaction` |
| `system_prompt` | `playbook` |
| `send_message` | `intervention` |
| `steer` | `intervention` |
| `debug` | `observability` |
| `error` | `observability` |
| `on_end` | `lifespan` |

**Coverage: 20/20 hooks occupied with meaningful behavior. All 28 extensions wired.**

---

## 4. The 7-Layer Dependency Graph

The dependency graph ensures extensions fire in a consistent order that respects data dependencies. An extension in layer N may depend on extensions in layers 0 through N, but never on a higher layer. The arrows show "depends on" relationships.

```
Layer 0 -- Foundation (no dependencies):
    provider-adapter, telemetry, audit, grimoire, tools, heartbeat

Layer 1 -- Input:
    input-router         <- heartbeat

Layer 2 -- State:
    context              <- grimoire
    daimon               <- grimoire
    memory               <- grimoire
    lifespan             <- heartbeat

Layer 3 -- Safety:
    safety               <- tools
    permits              <- safety
    risk                 <- safety + grimoire
    result-filter        <- safety
    coordination         <- tools
    compiler             <- tools
    model-router         <- provider-adapter
    x402-payment         <- provider-adapter

Layer 4 -- Cognition:
    turn-context         <- context
    dream                <- grimoire + daimon
    cybernetics          <- context + grimoire
    clade                <- grimoire + coordination

Layer 5 -- UX:
    ui-bridge            <- telemetry
    observability        <- telemetry

Layer 6 -- Intervention:
    playbook             <- grimoire
    intervention         <- heartbeat

Layer 7 -- Recovery:
    compaction           <- context
```

### Why Sequential, Not Parallel

The `after_turn` hook chain fires extensions **sequentially**, not in parallel. This is intentional. Each extension reads what the previous one wrote to GolemState:

1. **Heartbeat** writes `observation`, `regime`, `probe_results`, `prediction_error`, `cognitive_tier`
2. **Lifespan** reads heartbeat's cost data, writes `vitality`, `phase`, `credit_remaining`
3. **Daimon** reads observation + vitality, writes `current_pad`, `primary_emotion`, `somatic_markers_fired`
4. **Memory** reads observation + PAD (for emotional tagging), writes `episodes_written`, `grimoire_mutations`
5. **Risk** reads all of the above, writes `risk_assessment`, `active_warnings`
6. **Dream** reads emotional load + novelty + risk, writes `dream_state`
7. **Cybernetics** reads outcome + context entries, writes `context_policy_revision`
8. **Clade** reads all local state, syncs with peers, writes `clade_sync_status`, `pheromone_readings`
9. **Telemetry** reads everything, emits final metrics

Parallelizing this would require each extension to work with stale data -- the Daimon would appraise an observation without knowing the current mortality phase, and the dream scheduler would not know the latest emotional state. The sequential pipeline ensures every extension has the freshest possible data. Rust's `&mut GolemState` borrow makes this data flow explicit at the type level.

---

## 5. The Extension Registry

The registry accepts extensions, validates that the dependency graph is acyclic, and pre-computes firing orders per hook.

```rust
use std::sync::Arc;
use std::collections::HashMap;

/// The extension registry. Validates dependencies and computes firing orders.
///
/// Usage:
/// 1. Create a new registry: `let mut registry = ExtensionRegistry::new();`
/// 2. Register all extensions: `registry.register(Arc::new(HeartbeatExt::new(...)));`
/// 3. Build (validates + computes orders): `registry.build();`
/// 4. Fire hooks: `registry.fire_after_turn(&mut ctx).await?;`
pub struct ExtensionRegistry {
    /// All registered extensions, sorted by layer after build().
    extensions: Vec<Arc<dyn Extension>>,

    /// Pre-computed firing orders per hook type.
    /// Each entry is a Vec of indices into `extensions`.
    /// Avoids re-computing topological sort on every hook invocation.
    firing_orders: HashMap<HookId, Vec<usize>>,
}

/// Identifies which hook to fire. Used as the key for pre-computed orders.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum HookId {
    Session,
    Input,
    BeforeAgentStart,
    AgentStart,
    TurnStart,
    Context,
    BeforeProviderRequest,
    ToolCall,
    ToolExecutionStart,
    ToolExecutionUpdate,
    ToolExecutionEnd,
    ToolResult,
    TurnEnd,
    AgentEnd,
    AfterTurn,
    SystemPrompt,
    Steer,
    SendMessage,
    Debug,
    Error,
    End,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            firing_orders: HashMap::new(),
        }
    }

    /// Register an extension. Order of registration doesn't matter --
    /// the registry sorts by layer during build().
    pub fn register(&mut self, ext: Arc<dyn Extension>) {
        self.extensions.push(ext);
    }

    /// Validate the dependency graph and compute firing orders.
    ///
    /// This method:
    /// 1. Sorts all extensions by layer (0 first, 7 last).
    /// 2. Validates that every declared dependency exists and is in an
    ///    equal or lower layer.
    /// 3. Pre-computes the topological firing order for each hook type.
    ///
    /// Panics if the dependency graph contains cycles or missing dependencies.
    /// This is a startup-time check -- it fails fast rather than at runtime.
    pub fn build(&mut self) {
        // Sort by layer
        self.extensions.sort_by_key(|e| e.layer());

        // Validate dependencies
        let names: Vec<&str> = self.extensions.iter().map(|e| e.name()).collect();
        for ext in &self.extensions {
            for dep in ext.depends_on() {
                // Dependency must exist
                let dep_idx = names.iter().position(|n| n == dep)
                    .unwrap_or_else(|| panic!(
                        "Extension '{}' depends on '{}' which is not registered. \
                         Registered extensions: {:?}",
                        ext.name(), dep, names
                    ));
                // Dependency must be in same or lower layer
                let dep_layer = self.extensions[dep_idx].layer();
                assert!(
                    dep_layer <= ext.layer(),
                    "Extension '{}' (layer {}) depends on '{}' (layer {}). \
                     Dependencies must be in an equal or lower layer.",
                    ext.name(), ext.layer(), dep, dep_layer
                );
            }
        }

        // Pre-compute after_turn firing order
        // This is the canonical order: heartbeat -> lifespan -> daimon ->
        // memory -> risk -> dream -> cybernetics -> clade -> telemetry
        self.firing_orders.insert(
            HookId::AfterTurn,
            self.compute_topological_order(HookId::AfterTurn),
        );

        // Pre-compute all other hook orders
        for hook_id in [
            HookId::Session, HookId::Input, HookId::BeforeAgentStart,
            HookId::ToolCall, HookId::ToolResult, HookId::SystemPrompt,
            HookId::Steer, HookId::End,
        ] {
            self.firing_orders.insert(
                hook_id.clone(),
                self.compute_topological_order(hook_id),
            );
        }
    }

    /// Fire the after_turn hook across all extensions that implement it.
    /// Extensions fire in the pre-computed topological order.
    pub async fn fire_after_turn(&self, ctx: &mut AfterTurnCtx) -> Result<()> {
        let order = self.firing_orders.get(&HookId::AfterTurn)
            .expect("Registry not built -- call build() before firing hooks");
        for &idx in order {
            self.extensions[idx].on_after_turn(ctx).await?;
        }
        Ok(())
    }

    /// Fire the tool_call hook. Returns the most restrictive action
    /// (any Block beats all Allows).
    pub async fn fire_tool_call(
        &self,
        call: &ToolCall,
        ctx: &mut ToolCallCtx,
    ) -> Result<ToolAction> {
        let order = self.firing_orders.get(&HookId::ToolCall)
            .expect("Registry not built");
        let mut result = ToolAction::Allow;
        for &idx in order {
            let action = self.extensions[idx].on_tool_call(call, ctx).await?;
            match action {
                ToolAction::Block(_) => return Ok(action), // Immediate block
                ToolAction::Modify(_) => result = action,  // Modification overrides allow
                ToolAction::Allow => {}                     // No change
            }
        }
        Ok(result)
    }

    /// Compute topological order for a given hook, respecting layer ordering
    /// and declared dependencies.
    fn compute_topological_order(&self, _hook_id: HookId) -> Vec<usize> {
        // Layer ordering is sufficient because all dependencies flow from
        // higher to lower layers. Within a layer, registration order
        // (which the build() sort makes deterministic) applies.
        //
        // A full Kahn's algorithm implementation would be needed if
        // intra-layer dependencies were complex, but the current
        // 28-extension graph doesn't require it.
        (0..self.extensions.len()).collect()
    }
}
```

---

## 6. GolemState: The Mutable World

GolemState is the single mutable struct that extensions read and write during hook execution. It represents everything the Golem currently "knows" about itself and its environment.

The struct is organized by ownership: each section is written by a specific extension (or set of extensions) and read by downstream extensions. This ownership pattern is what makes the sequential `after_turn` firing order correct -- each section is written once per tick, then read by later extensions.

```rust
/// The mutable state of a living Golem. Passed through the extension chain.
///
/// Organization: each section has a primary writer (the extension that sets it)
/// and multiple readers (downstream extensions that consume it).
///
/// This struct is NOT thread-safe on its own -- it is passed as `&mut` through
/// the sequential hook chain. The CorticalState and EventFabric (wrapped in Arc)
/// provide thread-safe cross-fiber access for the specific data that needs it.
pub struct GolemState {
    // ===================================================================
    // IDENTITY (set once at provisioning, read-only thereafter)
    // ===================================================================
    /// Unique identifier for this Golem (derived from on-chain ERC-8004
    /// registration).
    pub id: GolemId,
    /// Configuration loaded from golem.toml + STRATEGY.md.
    pub config: GolemConfig,
    /// The Golem's generation number (0 = first-gen, N = Nth successor).
    pub generation: u32,

    // ===================================================================
    // HEARTBEAT STATE
    // Writer: heartbeat extension (after_turn)
    // Readers: lifespan, daimon, memory, risk, dream, cybernetics, clade
    // ===================================================================
    /// Current tick number (monotonically increasing, starts at 0).
    pub current_tick: u64,
    /// The latest market observation (prices, positions, gas, anomalies).
    /// None before the first tick completes.
    pub observation: Option<Observation>,
    /// Results from market probes (price deviation, liquidity change, etc.).
    pub probe_results: Vec<ProbeResult>,
    /// Current market regime (trending_up, trending_down, volatile,
    /// range_bound, unknown). Detected from probe results using regime
    /// classification.
    pub regime: MarketRegime,
    /// How surprising was this tick's observation vs. the Golem's
    /// expectations? Range [0.0, 1.0]. Compared against the adaptive
    /// threshold to determine cognitive tier (T0/T1/T2).
    pub prediction_error: f64,
    /// Which cognitive tier was selected for this tick.
    /// T0 = suppress (no LLM, $0.00), T1 = analyze (cheap LLM),
    /// T2 = deliberate (full LLM).
    pub cognitive_tier: CognitiveTier,

    // ===================================================================
    // MORTALITY STATE
    // Writer: lifespan extension (after_turn)
    // Readers: daimon, dreams, context, gating
    // ===================================================================
    /// Three-component mortality state. Any component reaching zero kills
    /// the Golem. Multiplicative composition:
    /// vitality_composite = economic * epistemic * stochastic.
    pub vitality: VitalityState,
    /// Current behavioral phase derived from vitality composite.
    /// Thriving (>0.7) -> Stable (0.5-0.7) -> Conservation (0.3-0.5) ->
    /// Declining (0.1-0.3) -> Terminal (<0.1).
    pub phase: BehavioralPhase,
    /// Remaining credit balance in USDC. When this reaches zero, economic
    /// death.
    pub credit_remaining: f64,
    /// Estimated ticks until death at current burn rate.
    pub estimated_ttl_ticks: u64,

    // ===================================================================
    // AFFECT STATE
    // Writer: daimon extension (after_turn)
    // Readers: memory (emotional tags), dreams (emotional load),
    //          context (mood-congruent retrieval), gating (arousal)
    // ===================================================================
    /// Current Pleasure-Arousal-Dominance vector. Updated by the Daimon
    /// after appraisal. Also written to the CorticalState for zero-latency
    /// reads by background fibers.
    pub current_pad: PADVector,
    /// Discrete emotion label derived from PAD octant.
    pub primary_emotion: PlutchikLabel,
    /// Somatic markers that fired this tick (situation -> emotional residue
    /// of past outcomes).
    pub somatic_markers_fired: Vec<SomaticMarkerSummary>,

    // ===================================================================
    // MEMORY STATE
    // Writer: memory extension (after_turn)
    // Readers: dreams (episodes for replay), cybernetics (learning rate)
    // ===================================================================
    /// Episodes written to the Grimoire during this tick.
    pub episodes_written_this_tick: Vec<EpisodeId>,
    /// All Grimoire mutations this tick (inserts, updates, promotions,
    /// deletions).
    pub grimoire_mutations: Vec<GrimoireMutation>,
    /// Admission Gate statistics (accepted/rejected and why).
    pub admission_gate_stats: AdmissionGateStats,

    // ===================================================================
    // RISK STATE
    // Writer: risk extension (tool_call, after_turn)
    // Readers: dreams (risk signals), clade (warnings shared)
    // ===================================================================
    /// Result of the five-layer risk assessment for this tick.
    pub risk_assessment: Option<RiskAssessment>,
    /// Currently active warnings (risk signals that haven't been resolved).
    pub active_warnings: Vec<Warning>,

    // ===================================================================
    // DREAM STATE
    // Writer: dream extension (after_turn)
    // Readers: cybernetics (dream insights feed self-tuning)
    // ===================================================================
    /// Current dream state (Idle, Scheduled, InProgress, Completed).
    pub dream_state: DreamState,
    /// Result of the most recent dream cycle (if any).
    pub last_dream_result: Option<DreamCycleResult>,

    // ===================================================================
    // CYBERNETICS STATE
    // Writer: cybernetics extension (after_turn)
    // ===================================================================
    /// Current revision of the Context Governor's learned policy.
    pub context_policy_revision: u32,
    /// What the cybernetics extension adjusted this tick.
    pub self_tune_adjustments: Vec<PolicyAdjustment>,

    // ===================================================================
    // COORDINATION STATE
    // Writer: clade extension (after_turn)
    // ===================================================================
    /// Result of the latest clade sync (entries sent/received).
    pub clade_sync_status: CladeSyncStatus,
    /// Current Pheromone Field readings (threats, opportunities, wisdom).
    pub pheromone_readings: PheromoneReadings,

    // ===================================================================
    // POSITION STATE (managed by tool results and chain reads)
    // ===================================================================
    /// Snapshots of all current DeFi positions (LP ranges, lending
    /// positions, vault deposits).
    pub positions: Vec<PositionSnapshot>,
    /// Total portfolio value in USD.
    pub portfolio_value: f64,

    // ===================================================================
    // INTERVENTION STATE (managed by intervention extension)
    // ===================================================================
    /// Active steers -- high-priority interrupts that preempt the current
    /// tick.
    pub active_steers: Vec<Intervention>,
    /// Pending followUps -- queued for the next appropriate decision
    /// window.
    pub pending_followups: Vec<Intervention>,

    // ===================================================================
    // SHARED INFRASTRUCTURE (Arc-wrapped for cross-fiber access)
    // ===================================================================
    /// Lock-free atomic perception surface. See Section 8.
    pub cortical_state: Arc<CorticalState>,
    /// Broadcast event bus. See Section 7.
    pub event_fabric: Arc<EventFabric>,
    /// The Golem's knowledge base (LanceDB + SQLite + filesystem).
    pub grimoire: Arc<Grimoire>,
    /// Merkle hash-chain audit trail.
    pub audit: Arc<AuditChain>,
}
```

### Intervention Primitives (from source 00-runtime-primitives)

The intervention system uses typed `InterventionKind` variants and delivery windows targeting specific cognitive phases. Steers are preemptive (interrupt current execution). FollowUps are non-preemptive (delivered when idle). Internal systems (risk daemon, mortality engine) use the same pathway via `SelfSteer`.

```rust
// crates/golem-surfaces/src/intervention.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionKind {
    /// Preempt current execution.
    Steer { cancel_in_flight: bool },
    /// Queue for a specific decision window.
    FollowUp { window: DeliveryWindow },
    /// System-generated: from internal daemons (risk, mortality).
    SelfSteer { source: SelfSteerSource },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryWindow {
    NextDecide,
    NextReflect,
    NextCurator,
    NextDream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelfSteerSource {
    RiskDaemon,
    MortalityEngine,
    PoliceCage,
    AttentionForager,
}

pub struct Intervention {
    pub id: InterventionId,
    pub kind: InterventionKind,
    pub message: String,
    pub priority: u8,
    pub sender: InterventionSender,
    pub created_at: u64,
    pub deadline: Option<u64>,
}

pub struct InterventionQueue {
    high_priority: VecDeque<Intervention>,
    low_priority: BTreeMap<DeliveryWindow, VecDeque<Intervention>>,
}

impl InterventionQueue {
    pub fn push_steer(
        &mut self, msg: String, cancel_in_flight: bool, priority: u8,
    ) -> InterventionId { /* ... */ }

    pub fn push_follow_up(
        &mut self, msg: String, window: DeliveryWindow,
    ) -> InterventionId { /* ... */ }

    pub fn drain_steers(&mut self) -> Vec<Intervention> { /* ... */ }

    pub fn drain_for_window(
        &mut self, window: DeliveryWindow,
    ) -> Vec<Intervention> { /* ... */ }
}
```

Every intervention becomes a Grimoire episode so the agent learns from its response pattern.

### Session Tree (from source 00-runtime-primitives)

Conversation history is modeled as a tree, not a linear list. The trunk is the main conversation. At any point you can branch, creating an alternative path from any earlier node. Branches are typed with explicit semantics.

```rust
// crates/golem-inference/src/session.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchKind {
    /// What-if analysis from a historical checkpoint.
    Sim { hypothesis: String },
    /// Preview owner intervention before applying it.
    PreSteer { pending_steer: String },
    /// Creative exploration during sleep cycles.
    Dream { seed: DreamSeed },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub kind: NodeKind,
    pub messages: Vec<AgentMessage>,
    /// SHA-256 of parent hash + this node's content.
    pub content_hash: [u8; 32],
    pub created_at: u64,
    pub label: Option<String>,
    pub metadata: ConversationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub model: String,
    pub cost_usd: f64,
    pub token_count: u32,
    pub golem_id: u64,
    pub checkpoint_id: Option<String>,
}

pub struct ConversationTree {
    nodes: HashMap<NodeId, ConversationNode>,
    current_head: NodeId,
    root: NodeId,
}

impl ConversationTree {
    pub fn new(root_messages: Vec<AgentMessage>) -> Self { /* ... */ }
    pub fn navigate(&mut self, target: NodeId) -> Result<Vec<NodeId>> { /* ... */ }
    pub fn branch(
        &self, from: NodeId, kind: BranchKind,
    ) -> ConversationBranch { /* ... */ }
    pub fn current_path(&self) -> Vec<&ConversationNode> { /* ... */ }
    pub fn find_by_hash(
        &self, hash: [u8; 32],
    ) -> Option<&ConversationNode> { /* ... */ }
    pub fn verify_integrity(&self) -> bool { /* ... */ }
}
```

The Merkle hash chain makes post-hoc history editing detectable. In a DeFi context, an agent that made a bad trade has an incentive (if compromised) to edit history. The chain prevents that silently.

Branches are typed, single-level only (no branch-of-branch), and cost-attributed. `BranchSession` wraps a ConversationTree with cost tracking and a commit/abort protocol:

```rust
pub struct BranchSession {
    pub id: SessionId,
    pub parent_id: SessionId,
    pub root_node: NodeId,
    pub kind: BranchKind,
    pub tree: ConversationTree,
    pub cost_usd: f64,
    pub max_cost_usd: f64,
    pub pending_predictions: Vec<SimulationPrediction>,
}
```

**ChronoNav** provides time-travel navigation, explicit about direction (forward vs. backward), with structured summaries and audit-chain recording:

```rust
pub struct ChronoNavRequest {
    pub target: NodeId,
    pub direction: NavDirection,
    pub summary_policy: SummaryPolicy,
    pub instruction_update: Option<InstructionUpdate>,
}

#[derive(Debug, Clone)]
pub enum SummaryPolicy {
    None,
    Structural,
    Narrative { max_tokens: u32 },
    Custom(String),
}
```

### Three-Tier Tool Authorization (from source 00-runtime-primitives)

Tools are organized into three trust tiers enforced by the type system. `Capability<T>` tokens are created only by the safety extension, consumed by move semantics, and cannot be forged or reused. The `PhantomData<fn(T) -> T>` (invariant over T) prevents using a `Capability<SwapTool>` where a `Capability<WithdrawTool>` is expected.

```rust
// crates/golem-tools/src/lib.rs

/// Unforgeable, single-use capability token.
/// No Default, no Clone, no Copy.
pub struct Capability<T> {
    pub value_limit: f64,
    pub expires_at: u64,
    pub policy_hash: [u8; 32],
    pub permit_id: String,
    _marker: PhantomData<fn(T) -> T>,
}

// Tier 1: read tools -- no capability needed
#[async_trait]
pub trait ReadTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_read(
        &self, params: serde_json::Value, ctx: &ToolContext,
    ) -> Result<ToolResult>;
}

// Tier 2: write tools -- capability consumed on use
#[async_trait]
pub trait WriteTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_write(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
        capability: Capability<Self>,
    ) -> Result<ToolResult>
    where Self: Sized;
}

// Tier 3: privileged tools -- capability + owner approval
#[async_trait]
pub trait PrivilegedTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_privileged(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
        capability: Capability<Self>,
        owner_approval: OwnerApproval,
    ) -> Result<ToolResult>
    where Self: Sized;
}
```

Move semantics make "impossible" mean "won't compile." Code that would call a write tool without a capability token does not compile.

> **Note**: Section 11 (above) shows the simplified Capability pattern used in the registry. This three-tier model from the source is the full design with value limits, policy hashes, and tier-specific trait bounds.

### Model Routing (from source 00-runtime-primitives)

The `ProviderRouter` implements a full routing layer with three cognitive tiers, cost tracking, budget enforcement, and fallback chains.

```rust
// crates/golem-inference/src/provider_router.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveTier {
    /// No LLM. Pure Rust. ~80% of ticks. $0.
    T0,
    /// Haiku-class. Analysis. ~18% of ticks. ~$0.001/call.
    T1,
    /// Opus-class. Deliberation. ~2% of ticks. ~$0.03-$0.10/call.
    T2,
}

pub struct ProviderRouter {
    tier_configs: HashMap<CognitiveTier, TierConfig>,
    providers: HashMap<String, Box<dyn LlmProvider>>,
    daily_spend: HashMap<CognitiveTier, AtomicF64>,
    daily_budget: f64,
    auth: Arc<AuthStorage>,
}

impl ProviderRouter {
    pub async fn complete(
        &self,
        tier: CognitiveTier,
        messages: Vec<AgentMessage>,
        tools: &[ToolDef],
    ) -> Result<ProviderResponse> { /* ... */ }

    pub fn budget_ok(&self, tier: CognitiveTier) -> bool { /* ... */ }
}
```

Model selection is not a user-facing API. It is the model-router extension's job, driven by the prediction-error gate result for each tick.

### Adaptive Clock (from source 00-runtime-primitives)

Three concurrent temporal scales, modeled after neural oscillatory hierarchies:

```rust
// crates/golem-runtime/src/adaptive_clock.rs

pub struct AdaptiveClock {
    gamma: ClockOscillator,   // 5-15s: perception
    theta: ClockOscillator,   // 30-120s: cognition
    delta: DeltaCounter,      // ~50 theta-ticks: consolidation
    cost_tracker: DailyCostTracker,
    config: ClockConfig,
}

pub struct ClockOscillator {
    min_interval: Duration,
    max_interval: Duration,
    current_interval: Duration,
}

impl ClockOscillator {
    pub fn accelerate(&mut self) {
        self.current_interval = (self.current_interval / 2).max(self.min_interval);
    }
    pub fn decelerate(&mut self) {
        self.current_interval = (self.current_interval * 2).min(self.max_interval);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickKind { Gamma, Theta, Delta }
```

Gamma handles perception (resolve predictions, update CorticalState). Theta handles cognition (full predict-appraise-gate-retrieve-deliberate-act-reflect pipeline). Delta handles consolidation (Curator cycle, memory maintenance, dream scheduling). The clock throttles all rates when approaching the daily budget ceiling.

### Context Management: Compaction Preservation (from source 00-runtime-primitives)

When context compaction applies (to the conversation sidecar), it preserves structured data:

```rust
// crates/golem-context/src/compaction.rs

pub struct CompactionPreserve {
    pub policy_cage_hash: [u8; 32],
    pub positions: Vec<PositionSummary>,
    pub behavioral_phase: BehavioralPhase,
    pub playbook_heuristics: Vec<PlaybookEntry>,
    pub active_warnings: Vec<RiskWarning>,
}

pub struct CompactionResult {
    pub structured: CompactionPreserve,
    pub narrative: String,
    pub tokens_before: u32,
    pub tokens_after: u32,
    pub compression_ratio: f32,
}
```

---

*Continued in [13b-runtime-extensions.md](./13b-runtime-extensions.md): Type-State Machine, Event Fabric, CorticalState, Arena, Shutdown, Main Binary (Sections 7-15).*

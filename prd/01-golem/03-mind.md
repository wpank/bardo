# 03 -- Mind: Cognitive Mechanisms and State Management [SPEC]

**Runtime Primitives, Attention, Habituation, Homeostasis, and State Forensics**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-runtime` (mind.rs, attention.rs, habituation.rs, homeostasis.rs), `golem-state` (snapshot.rs, delta.rs, replay.rs, metrics.rs)
>
> Cross-references: [01-cognition.md](./01-cognition.md), [02-heartbeat.md](./02-heartbeat.md) (adaptive clock, event fabric), [17-prediction-engine.md](./17-prediction-engine.md) (Oracle), [18-cortical-state.md](./18-cortical-state.md) (CorticalState)
>
> Sources: `03-agent-runtime/00` through `03-agent-runtime/10`

> **Reader orientation:** This document is the overview of the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) "mind" -- the runtime mechanisms between raw perception and deliberate reasoning. It sits in the `01-golem` cognition layer. The key concept: these 10 cognitive mechanisms (attention salience, sleep pressure, habituation, homeostasis, compensation, event wakeup, state snapshots, context deltas, episodic replay, metrics) operate concurrently alongside the Heartbeat (the 9-step decision cycle), modulating how each tick behaves. Detailed specs are split into `03b-cognitive-mechanisms.md` (attention, sleep, habituation, homeostasis, compensation, event wakeup) and `03c-state-management.md` (snapshots, deltas, replay, metrics). See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What This File Covers

The Golem's "mind" is the set of runtime mechanisms between raw perception (CorticalState) and deliberate reasoning (LLM inference). These mechanisms handle attention salience, habituation, homeostasis, compensation/rollback, state snapshots, context compression, episodic replay, and metrics/tracing. They are the cognitive infrastructure that makes the heartbeat pipeline adaptive rather than mechanical.

This is an overview. Detailed specifications are split across:

- **[03b-cognitive-mechanisms.md](./03b-cognitive-mechanisms.md)** -- attention salience, sleep pressure, habituation, homeostasis, compensation/rollback, event-driven wakeup
- **[03c-state-management.md](./03c-state-management.md)** -- state snapshots, context delta compression, episodic replay, metrics/tracing

---

## Architecture: 10 Cognitive Mechanisms

The mind consists of 10 runtime primitives that operate alongside the 9-step heartbeat cycle. They are not heartbeat steps; they are concurrent processes that modulate how the heartbeat operates.

| # | Mechanism | Purpose | Frequency | Crate |
|---|-----------|---------|-----------|-------|
| 1 | AttentionSalience | Priority queue with decay for observation items | Gamma | `golem-runtime` |
| 2 | SleepPressure | Accumulates pressure for dream consolidation | Gamma | `golem-runtime` |
| 3 | HabituationMask | Per-pattern exposure attenuation | Gamma | `golem-runtime` |
| 4 | HomeostasisRegulator | Proportional control for signal stability | Theta | `golem-runtime` |
| 5 | CompensationChain | Saga-pattern rollback for multi-step actions | Per-action | `golem-runtime` |
| 6 | EventDrivenWakeup | Condition-based clock interrupts | Async | `golem-runtime` |
| 7 | StateSnapshot | Content-addressed full agent state | Delta | `golem-state` |
| 8 | ContextDelta | I-frame/P-frame context compression | Theta | `golem-state` |
| 9 | EpisodicReplay | Case-based reasoning for deliberation | Theta (T1+) | `golem-state` |
| 10 | MetricsEmitter | Wide-event telemetry and W3C tracing | Per-tick | `golem-state` |

### Relationship to the heartbeat

```
Heartbeat Step 1 (OBSERVE)
    |
    |-- AttentionSalience: which observations get priority
    |-- HabituationMask: attenuate repeated patterns
    |-- EventDrivenWakeup: interrupt normal cadence for urgent events
    v
Heartbeat Step 3 (ANALYZE)
    |
    |-- SleepPressure: accumulate toward dream threshold
    |-- HomeostasisRegulator: check signal stability
    v
Heartbeat Step 4 (GATE)
    |
    |-- ContextDelta: compress context for LLM calls
    |-- EpisodicReplay: inject relevant past episodes
    v
Heartbeat Steps 5-8 (DELIBERATE through VERIFY)
    |
    |-- CompensationChain: track rollback points for actions
    v
Heartbeat Step 9 (REFLECT)
    |
    |-- StateSnapshot: periodic content-addressed checkpoint
    |-- MetricsEmitter: emit wide-event telemetry
```

---

## Knowledge Architecture

The Grimoire specification -- knowledge representation, learning processes, memory architecture, dream cycles, and the outer memory loop (Styx Archive/Clade/Lethe layers) -- lives in the dedicated memory section.

**See [04-memory/](../04-memory/) for the full specification.**

| File | Topic |
|------|-------|
| `04-memory/00-overview.md` | Memory architecture overview: forgetting-as-feature, two-loop learning model |
| `04-memory/01-grimoire.md` | The Grimoire (the agent's persistent knowledge base): episodes, insights, heuristics, PLAYBOOK, Curator pipeline |
| `04-memory/02-emotional-memory.md` | PAD vector (three-dimensional affect state) weighted retrieval, mood-congruent recall |
| `04-memory/03-mortal-memory.md` | Memory under mortality pressure: phase-dependent retrieval and consolidation regimes |
| `20-styx/00-architecture.md` | Styx (global knowledge relay at wss://styx.bardo.run) architecture: Vault (L0), Clade (L1), Lethe (L2) |
| `04-memory/06-economy.md` | Knowledge lifecycle economics: demurrage, Styx marketplace pricing |

---

## Events Emitted

Mind-related events track the thinking lifecycle, model selection, and cognitive mechanism state changes. All events are variants of the `GolemEvent` enum emitted to the Event Fabric.

| Event | Trigger | Payload |
|---|---|---|
| `GolemEvent::ThinkingStart` | Agent invocation begins | `{ invocation_id, trigger, model }` |
| `GolemEvent::ThinkingEnd` | Agent invocation completes | `{ invocation_id, duration, turn_count, cost }` |
| `GolemEvent::TurnStart` | Each LLM turn begins | `{ turn_number, model }` |
| `GolemEvent::TurnEnd` | Each LLM turn completes | `{ turn_number, usage, cost, cache_hit_rate }` |
| `GolemEvent::ModelSelected` | Model tier chosen | `{ tier, model_id, reason }` |
| `GolemEvent::SleepPressureThreshold` | Dream consolidation triggered | `{ pressure, threshold, complexity }` |
| `GolemEvent::HabituationDecay` | Pattern attenuation applied | `{ pattern_id, attenuation_factor }` |
| `GolemEvent::HomeostasisIntervention` | Regulator corrected a signal | `{ signal, deviation, correction }` |
| `GolemEvent::CompensationRollback` | Multi-step action rolled back | `{ chain_id, rolled_back_steps }` |
| `GolemEvent::StateSnapshotCreated` | Content-addressed checkpoint | `{ tick, blake3_hash, size_bytes }` |

---

## Runtime Primitives (from source 00-runtime-primitives)

The runtime primitives form the substrate on which all cognitive mechanisms operate. These are the low-level infrastructure types that handle session control, conversation management, tool authorization, model routing, event dispatch, and adaptive timing.

### Session Control: Intervention System

Interventions are typed interrupts that can preempt or queue messages for specific cognitive phases. The steer/followUp distinction is the most consequential intervention primitive.

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

Interventions from internal systems (the risk daemon, the mortality engine) use the same pathway as owner steers. Every intervention becomes a Grimoire episode so the agent learns from its response pattern.

### Session Tree: ConversationTree with Merkle Hash Chain

Conversation history is a tree, not a linear list. Each node carries a SHA-256 content hash chained to its parent, making post-hoc history editing detectable.

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

    /// Navigate to any node. Returns the path from root to target.
    pub fn navigate(&mut self, target: NodeId) -> Result<Vec<NodeId>> { /* ... */ }

    /// Create a typed branch from a checkpoint node.
    pub fn branch(
        &self, from: NodeId, kind: BranchKind,
    ) -> ConversationBranch { /* ... */ }

    /// Path from root to current head.
    pub fn current_path(&self) -> Vec<&ConversationNode> { /* ... */ }

    /// Content-addressable lookup by conversation hash.
    pub fn find_by_hash(
        &self, hash: [u8; 32],
    ) -> Option<&ConversationNode> { /* ... */ }

    /// Verify the hash chain from root to current head.
    pub fn verify_integrity(&self) -> bool { /* ... */ }
}
```

### Branch Sessions

Branches are typed, single-level only (no branch-of-branch), and cost-attributed. The `Sim` branch type connects to the prediction engine so simulation outcomes can be verified against reality.

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

impl BranchSession {
    pub fn from_checkpoint(
        parent: &InferenceSession,
        checkpoint_id: &str,
        kind: BranchKind,
        max_cost_usd: f64,
    ) -> Result<Self> { /* ... */ }

    /// Commit results back to parent. Err if parent moved past branch point.
    pub fn commit(
        self, parent: &mut InferenceSession,
    ) -> Result<CommitResult> { /* ... */ }

    /// Abandon and free resources.
    pub fn abort(self, reason: AbandonReason) { /* ... */ }
}
```

### ChronoNav (Time-Travel Navigation)

Navigation is explicit about direction (forward vs. backward in time), generates structured summaries, and records each navigation in the audit chain.

```rust
pub struct ChronoNavRequest {
    pub target: NodeId,
    pub direction: NavDirection,
    pub summary_policy: SummaryPolicy,
    pub instruction_update: Option<InstructionUpdate>,
}

#[derive(Debug, Clone)]
pub enum NavDirection {
    Backward,
    Forward,
}

#[derive(Debug, Clone)]
pub enum SummaryPolicy {
    None,
    Structural,
    Narrative { max_tokens: u32 },
    Custom(String),
}
```

### Context Management: CompactionPreserve

Bardo assembles a Cognitive Workspace fresh each tick from structured categories. Compaction applies only to the conversation sidecar. When it does compact, it preserves positions, PolicyCage hash, vitality, risk parameters, and top PLAYBOOK heuristics as structured data.

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

### Three-Tier Tool Authorization

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

### Model Routing: ProviderRouter

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
    /// Select model for tier, make the call, fall back on error or budget exhaustion.
    pub async fn complete(
        &self,
        tier: CognitiveTier,
        messages: Vec<AgentMessage>,
        tools: &[ToolDef],
    ) -> Result<ProviderResponse> { /* ... */ }

    pub fn budget_ok(&self, tier: CognitiveTier) -> bool { /* ... */ }
}
```

### Event Fabric

The Event Fabric is a `tokio::broadcast` channel with a 10,000-event ring buffer. 50+ typed event variants span 14 subsystem categories. Late subscribers can replay recent history for reconnection.

```rust
// crates/golem-core/src/event_fabric.rs

#[derive(Debug, Clone)]
pub enum GolemEvent {
    HeartbeatTick { tick_id: u64, tier: CognitiveTier, cost_usd: f64 },
    ToolCall { tool_id: String, tick_id: u64, params_hash: [u8; 32] },
    ToolResult { tool_id: String, tick_id: u64, success: bool, duration_ms: u64 },
    AffectUpdate { pad: PadVector, emotion: PlutchikEmotion },
    VitalityUpdate { economic: f32, epistemic: f32, composite: f32 },
    PhaseTransition { from: BehavioralPhase, to: BehavioralPhase },
    PositionOpened { protocol: String, asset: String, size_usd: f64 },
    PositionClosed { position_id: String, pnl_usd: f64 },
    // ... 40+ more variants across 14 categories
}

pub struct EventFabric {
    tx: tokio::sync::broadcast::Sender<GolemEvent>,
    ring: Arc<Mutex<VecDeque<GolemEvent>>>,
    ring_capacity: usize,
}

impl EventFabric {
    pub fn emit(&self, event: GolemEvent) { /* ... */ }
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<GolemEvent> { /* ... */ }
    pub fn replay_since(&self, since_event_id: Option<u64>) -> Vec<GolemEvent> { /* ... */ }
}
```

### Adaptive Clock

Three concurrent temporal scales, modeled after neural oscillatory hierarchies.

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

### Cortical State

A lock-free shared perception surface, ~256 bytes in 4 cache lines. Each signal group has exactly one writer. All reads use `Ordering::Relaxed`. Not transactionally consistent; used for TUI rendering, inference tier selection, attention allocation, and affect-based routing where slight staleness is acceptable. Safety-critical decisions use their own strongly-consistent state.

```rust
// crates/golem-core/src/cortical_state.rs

#[repr(C, align(64))]
pub struct CorticalState {
    // Affect (written by Daimon)
    pub(crate) pleasure: AtomicU32,
    pub(crate) arousal: AtomicU32,
    pub(crate) dominance: AtomicU32,
    pub(crate) primary_emotion: AtomicU8,

    // Prediction (written by Oracle)
    pub(crate) aggregate_accuracy: AtomicU32,
    pub(crate) accuracy_trend: AtomicI8,

    // Mortality (written by mortality engine)
    pub(crate) economic_vitality: AtomicU32,
    pub(crate) composite_vitality: AtomicU32,
    pub(crate) behavioral_phase: AtomicU8,

    // Environment (written by domain probes)
    pub(crate) regime: AtomicU8,

    // Inference (written by inference router)
    pub(crate) inference_budget_remaining: AtomicU32,
    pub(crate) current_tier: AtomicU8,
}
```

### Lifecycle Hooks: Extension Trait

Extensions are native Rust structs implementing an `Extension` trait with 20 lifecycle hooks, all defaulting to no-ops. Extensions are organized in a 7-layer dependency DAG; lower layers boot first and cannot depend on higher layers.

```rust
// crates/golem-runtime/src/extension.rs

#[async_trait]
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn layer(&self) -> u8;
    fn depends_on(&self) -> &[&str] { &[] }

    // Session lifecycle
    async fn on_session(
        &self, _reason: SessionReason, _ctx: &mut SessionCtx,
    ) -> Result<()> { Ok(()) }

    // Input processing
    async fn on_input(
        &self, _msg: &mut InputMessage, _ctx: &InputCtx,
    ) -> Result<InputAction> { Ok(InputAction::Pass) }

    // Turn lifecycle
    async fn on_turn_start(&self, _ctx: &TurnStartCtx) -> Result<()> { Ok(()) }
    async fn on_context(
        &self, _msgs: &mut Vec<AgentMessage>, _ctx: &ContextCtx,
    ) -> Result<()> { Ok(()) }
    async fn on_turn_end(&self, _ctx: &TurnEndCtx) -> Result<()> { Ok(()) }

    // Tool lifecycle
    async fn on_tool_call(
        &self, _call: &ToolCall, _ctx: &mut ToolCallCtx,
    ) -> Result<ToolAction> { Ok(ToolAction::Allow) }
    async fn on_tool_result(
        &self, _result: &mut ToolResult, _ctx: &ToolResultCtx,
    ) -> Result<()> { Ok(()) }

    // Post-turn learning
    async fn on_after_turn(
        &self, _ctx: &mut AfterTurnCtx,
    ) -> Result<()> { Ok(()) }

    // System
    async fn on_error(
        &self, _err: &GolemError, _ctx: &ErrorCtx,
    ) -> Result<()> { Ok(()) }
    async fn on_end(&self, _ctx: &EndCtx) -> Result<()> { Ok(()) }
}
```

The `ExtensionRegistry` validates the dependency graph, performs topological sort, and fires hooks in order. Short-circuit semantics apply: the most restrictive tool call action wins.

---

## Extension Hook Integration

Three hooks govern the thinking pipeline, implemented as extensions in the `golem-runtime` Extension trait system.

| Hook | Extension | Behavior |
|---|---|---|
| `before_agent_start` | `golem-context`, `golem-model-router` | Context assembly, model selection |
| `context` | `golem-turn-context` | Phase-aware context injection per turn |
| `after_tick` | `golem-cybernetics` | Learning loop execution |

---

## References

- [KAHNEMAN-2011] Kahneman, D. *Thinking, Fast and Slow*. FSG, 2011. — *Proposes dual-process theory (System 1/System 2): fast heuristic vs. slow deliberate reasoning. The cognitive model behind the T0/T1/T2 gating architecture.*
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer." *Trends in Cognitive Sciences*, 4(11), 2000. — *Adds an episodic buffer to working memory that integrates information across subsystems; the theoretical basis for the Cognitive Workspace.*
- [NEWELL-1990] Newell, A. *Unified Theories of Cognition*. Harvard, 1990. — *Foundational work on cognitive architectures as unified systems; Soar and ACT-R descend from this framework.*
- [LAIRD-2012] Laird, J.E. *The Soar Cognitive Architecture*. MIT Press, 2012. — *The canonical Soar reference: production-rule cognitive architecture with working memory, long-term stores, and impasses. One of CoALA's intellectual ancestors.*
- [PI-AGENT] Pi coding agent. GitHub: @mariozechner/pi-coding-agent. — *Reference implementation for session control, session tree, and extension system primitives. The Golem runtime's extension model evolved from Pi's plugin architecture.*
- [LANGGRAPH] LangGraph (LangChain). — *Directed-graph agent execution with database-backed checkpointing, time-travel debugging, and human-in-the-loop primitives. Comparison point for Golem's conversation tree and ChronoNav.*
- [TEMPORAL] Temporal. — *Durable workflow orchestration with deterministic replay, signals, queries, and updates. Comparison point for the compensation chain and saga-pattern rollback.*
- [AUTOGEN] Microsoft AutoGen v0.4. — *Actor-model agent runtime with standalone and distributed modes. Comparison point for the multi-agent delegation hierarchy.*
- [HUANG-2025] S. Huang et al. (2025). "Fork/Explore/Commit: OS primitives for agents." arXiv:2602.08199. — *Proposes OS-level fork/explore/commit primitives for agent branching; informs the BranchSession design.*

---

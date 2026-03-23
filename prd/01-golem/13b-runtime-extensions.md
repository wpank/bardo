# Runtime Extensions -- Part B [SPEC]

**Type-State Machine, Event Fabric, CorticalState, Arena, Shutdown, Main Binary**

> **Version**: 3.0 | **Status**: Implementation Specification
>
> Continuation of [13a-runtime-extensions.md](./13a-runtime-extensions.md) (extension architecture, registry, GolemState, interventions, session tree, tool authorization, model routing, adaptive clock). Sections 7-15.

> **Reader orientation:** This is Part B of the runtime extensions specification. It covers the type-state lifecycle machine (Provisioning/Active/Dreaming/Terminal/Dead enforced at compile time), the Event Fabric (tokio::broadcast ring buffer for internal state transitions), the CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model), the arena allocator (bumpalo per-tick allocation), the 10-phase graceful shutdown protocol, and the main binary startup sequence. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## 7. Type-State Lifecycle Machine

### The Problem

A dead Golem must never process another heartbeat tick. A terminal Golem must not enter a dream cycle. A provisioning Golem must not accept steers. These are safety-critical invariants -- violating them could lead to unauthorized trades, corrupted state, or financial loss.

In a typical runtime, these invariants are enforced by runtime checks: `if golem.phase == Dead { return Err(...) }`. But runtime checks can be bypassed by bugs, forgotten in new code paths, or circumvented by unexpected state mutations.

### The Solution: Phantom Types

Rust's type system can encode these invariants at compile time using **type-state programming** with phantom types. The idea: each lifecycle state is a zero-sized type (it occupies no memory at runtime), and the `Golem<S>` struct is parameterized by it. Methods are available only for the correct state. Invalid transitions are not "checked at runtime" -- they are **impossible to write**.

This technique was formalized by Dennis & Van Horn (1966) in their work on capability-based security: access rights should be verified at the type level, not through runtime guards [DENNIS-VAN-HORN-1966]. Strom & Yemini (1986) later developed the typestate concept specifically for enforcing protocol compliance at compile time [STROM-YEMINI-1986].

```rust
use std::marker::PhantomData;

// === Lifecycle states as zero-sized types ===
// These types exist only in the type system. They occupy zero bytes at
// runtime. Their purpose is to restrict which methods are available
// on Golem<S>.

/// The Golem is initializing: loading config, opening storage, registering
/// extensions.
pub struct Provisioning;

/// The Golem is running: processing heartbeat ticks, accepting steers,
/// managing positions.
pub struct Active;

/// The Golem is dreaming: heartbeat paused, running NREM/REM/Consolidation.
pub struct Dreaming;

/// The Golem is dying: running the Thanatopsis death protocol (irreversible).
pub struct Terminal;

/// The Golem is dead: genome extracted, state frozen. Cannot be used again.
pub struct Dead;

/// A Golem in a specific lifecycle state.
///
/// The type parameter S determines which operations are available.
/// `PhantomData<S>` tells the compiler that S is "used" without actually
/// storing any data -- Golem<Active> and Golem<Dead> have identical runtime
/// layout, but different compile-time capabilities.
pub struct Golem<S> {
    pub state: GolemState,
    _phase: PhantomData<S>,
}
```

### Valid Transitions

Each `impl` block is only available for Golems in the specified state. Transitions consume the old Golem (by taking `self` by value) and produce a new one in the target state. The original cannot be used again because Rust's ownership system moves it.

```rust
impl Golem<Provisioning> {
    /// Create a new Golem from config. This is the ONLY way to enter the
    /// lifecycle. Initializes storage, opens the Grimoire, registers with
    /// ERC-8004, and prepares the extension registry.
    pub async fn provision(
        config: GolemConfig,
    ) -> Result<Golem<Provisioning>> {
        let state = GolemState::initialize(config).await?;
        Ok(Golem { state, _phase: PhantomData })
    }

    /// Activate the Golem. Begins the heartbeat loop.
    /// Consumes the Provisioning Golem and produces an Active one.
    pub fn activate(self) -> Golem<Active> {
        self.state.event_fabric.emit(
            Subsystem::Lifecycle,
            EventPayload::LifecycleTransition {
                from: "provisioning".into(),
                to: "active".into(),
            },
        );
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Active> {
    /// Run one heartbeat tick. Only available when Active.
    /// See 02a-heartbeat-pipeline.md for the full 9-step implementation.
    pub async fn tick(
        &mut self,
        arena: &mut TickArena,
    ) -> Result<DecisionCycleRecord> {
        arena.reset(); // O(1) deallocation of all temporaries
        execute_tick(&mut self.state, &self.registry, arena).await
    }

    /// Enter dream state. Consumes Active, produces Dreaming.
    /// The heartbeat loop must pause while dreaming.
    pub fn begin_dream(self) -> Golem<Dreaming> {
        self.state.event_fabric.emit(
            Subsystem::Dreams,
            EventPayload::DreamStart {
                trigger: self.state.dream_state.trigger_reason(),
            },
        );
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Begin the death protocol. Irreversible -- there is no going back
    /// from Terminal. Consumes Active, produces Terminal.
    pub fn begin_death(self, cause: DeathCause) -> Golem<Terminal> {
        self.state.event_fabric.emit(
            Subsystem::Lifecycle,
            EventPayload::DeathInitiated {
                cause: cause.to_string(),
            },
        );
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Receive a steer (mid-tick interrupt). Only available when Active.
    pub async fn steer(
        &mut self,
        intervention: Intervention,
    ) -> Result<()> {
        self.state.active_steers.push(intervention);
        Ok(())
    }
}

impl Golem<Dreaming> {
    /// Run one dream cycle. Only available when Dreaming.
    /// Three-phase cycle: NREM replay -> REM imagination -> Consolidation.
    /// See 06-dreams.md for the full implementation.
    pub async fn dream_cycle(
        &mut self,
    ) -> Result<DreamCycleResult> {
        dream::dream_cycle(
            &self.state.grimoire,
            &mut self.state,
        ).await
    }

    /// Wake up and return to the Active state.
    pub fn wake(self) -> Golem<Active> {
        self.state.event_fabric.emit(
            Subsystem::Dreams,
            EventPayload::DreamComplete {
                cycles_completed: self.state.dream_state.cycles_completed,
            },
        );
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Economic death can occur during dreaming (credit runs out).
    pub fn begin_death(self, cause: DeathCause) -> Golem<Terminal> {
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Terminal> {
    /// Execute the Thanatopsis death protocol. Consumes Terminal, produces
    /// Dead.
    ///
    /// Four phases:
    /// - Phase 0: ACCEPTANCE -- acknowledge the death cause, emit events
    /// - Phase I: SETTLEMENT -- triage positions, execute critical
    ///   settlements
    /// - Phase II: REFLECTION -- generate death testament (LLM-written
    ///   structured reflection)
    /// - Phase III: LEGACY -- compress Grimoire through genomic bottleneck
    ///   (<=2048 entries), export genome for successor, deposit bloodstains
    ///   to Pheromone Field
    ///
    /// See 05-mortality.md for the full implementation.
    pub async fn thanatopsis(self) -> Result<Golem<Dead>> {
        let mut golem = self;
        // Phase 0-III execute sequentially...
        // The genome is written to $GOLEM_DATA/death/genome.bincode
        // The testament is written to $GOLEM_DATA/death/testament.json
        Ok(Golem { state: golem.state, _phase: PhantomData })
    }
}

impl Golem<Dead> {
    /// Extract the genome for successor inheritance.
    /// Consuming: the dead Golem cannot be used after this call.
    pub fn into_genome(self) -> Genome {
        // The genome contains:
        // - <=2048 compressed Grimoire entries (genomic bottleneck)
        // - Death testament
        // - Final PLAYBOOK.md
        // - Causal graph snapshot
        // - Somatic landscape fragments
        // - Configuration that produced this Golem
        self.state.load_genome()
    }

    /// Extract death recap for the engagement system (kill screen).
    pub fn death_recap(&self) -> DeathRecap {
        DeathRecap::generate(&self.state)
    }
}
```

### What Cannot Compile

These are not runtime errors. They are **compilation failures** -- the code cannot be written.

```rust
// ERROR: no method named `tick` found for `Golem<Dead>`
let dead_golem: Golem<Dead> = /* ... */;
dead_golem.tick(&mut arena).await;  // Won't compile.

// ERROR: no method named `dream_cycle` found for `Golem<Terminal>`
let dying_golem: Golem<Terminal> = /* ... */;
dying_golem.dream_cycle().await;  // Won't compile.

// ERROR: no method named `steer` found for `Golem<Provisioning>`
let new_golem: Golem<Provisioning> = /* ... */;
new_golem.steer(intervention).await;  // Won't compile.

// ERROR: use of moved value
let active_golem: Golem<Active> = /* ... */;
let terminal = active_golem.begin_death(cause);  // Moves active_golem
active_golem.tick(&mut arena).await;  // Won't compile -- moved.
```

### Extension to Other State Machines

The same type-state pattern is applied to every state machine in the system:

```rust
// ActionPermit states
pub struct PermitCreated;
pub struct PermitCommitted;
// ActionPermit<PermitCreated> has .commit() but not .revoke()
// ActionPermit<PermitCommitted> has .revoke() but not .commit()

// Thanatopsis phases
pub struct Acceptance;
pub struct Settlement;
pub struct Reflection;
pub struct Legacy;
// Thanatopsis<Acceptance> -> .settle() -> Thanatopsis<Settlement>
// Skipping Settlement -> Reflection is a compile error.

// Dream cycle phases
pub struct NREMPhase;
pub struct REMPhase;
pub struct IntegrationPhase;
```

### Why Traits Beat Plugins

The philosophical argument for native Rust extensions over a plugin runtime reduces to a single claim: **the boundary between "host" and "extension" should not exist at runtime.**

In Pi's architecture, every hook invocation crossed a serialization boundary. GolemState was serialized to JSON, passed to the QuickJS sandbox, deserialized, modified, re-serialized, and returned. For a struct with 30+ fields including nested vectors and hashmaps, this was the dominant cost per hook. Twenty hooks per tick, each with serialization overhead, added up to measurable latency in a system where ticks need to complete in seconds.

Rust's trait system eliminates this boundary. An `Arc<dyn Extension>` is a pointer and a vtable. Calling `ext.on_after_turn(ctx)` is an indirect function call -- the same cost as a C++ virtual method call. The `GolemState` is passed by mutable reference, not by copy. Extensions read and write fields directly.

The type system also provides guarantees that no plugin sandbox can match. When the Daimon extension writes `state.current_pad = new_pad`, the compiler verifies:
- The Daimon holds a `&mut GolemState` (exclusive access)
- The `current_pad` field exists and has type `PADVector`
- The `new_pad` value is a valid `PADVector`
- No other code is reading `current_pad` during this write

A JavaScript sandbox checks none of these at compile time. It discovers type mismatches, missing fields, and concurrent access bugs at runtime -- inside a production system managing real positions.

Zero-cost abstractions are not about performance alone. They are about correctness. The Golem's safety invariants (a dead Golem cannot trade, a conservation-phase Golem cannot open positions) are not policy statements that might be violated by bugs. They are structural properties of the compiled binary.

---

## 8. Event Fabric: The Nervous System

### Purpose

The Event Fabric is how internal state transitions become visible to the outside world. When the Daimon computes a new PAD vector, that is an internal operation -- but the TUI needs to update the creature's expression, the web dashboard needs to update the affect chart, and the Telegram bot needs to decide whether the emotional shift warrants a notification.

The Event Fabric solves this by providing a **non-blocking broadcast channel**: subsystems emit typed events, and surfaces subscribe to the categories they care about.

### Design Principles

1. **Non-blocking.** `emit()` never blocks the heartbeat pipeline. If the TUI is slow to consume events, the pipeline keeps going. Events that can't be delivered are dropped. The Golem's cognition must never wait for a UI.

2. **Read-only observability.** Surfaces consume events. They never write back to GolemState. The Event Fabric is a one-way mirror into the Golem's mind.

3. **Replay for reconnection.** A ring buffer (10,000 events) allows new subscribers to catch up on recent history. When the TUI reconnects after a network blip, it replays from its last-seen sequence number.

4. **Typed events.** 50+ event types across 16 subsystems. Every event maps to a specific internal state transition. No untyped strings.

```rust
use tokio::sync::broadcast;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::VecDeque;

/// The Event Fabric: a non-blocking broadcast bus for internal state
/// transitions.
///
/// Architecture:
/// - A tokio broadcast channel (bounded, lagging receivers lose events)
/// - A ring buffer for replay (10,000 events, allows reconnection)
/// - A monotonic sequence counter for ordering and gap detection
///
/// Thread safety: the EventFabric is wrapped in Arc and shared across
/// fibers. emit() is safe to call from any fiber. subscribe() returns
/// an independent receiver.
pub struct EventFabric {
    /// Broadcast channel sender.
    tx: broadcast::Sender<GolemEvent>,
    /// Ring buffer for replay on reconnection.
    buffer: Arc<RwLock<VecDeque<GolemEvent>>>,
    /// Monotonic sequence counter. Ordering::Relaxed is sufficient because
    /// we don't need cross-field consistency -- each event is
    /// self-contained.
    seq: AtomicU64,
}

/// A single event emitted by a subsystem.
#[derive(Clone, Debug, serde::Serialize)]
pub struct GolemEvent {
    /// Monotonic sequence number (for ordering and gap detection).
    pub seq: u64,
    /// When this event was emitted (monotonic clock, not wall clock).
    pub ts: std::time::Instant,
    /// Which heartbeat tick this event belongs to.
    pub tick: u64,
    /// Which subsystem emitted this event.
    pub subsystem: Subsystem,
    /// The event payload (typed -- each variant corresponds to a specific
    /// state transition).
    pub payload: EventPayload,
}

/// The 16 subsystems that emit events.
/// Surfaces subscribe to the subsystems they care about.
#[derive(Clone, Debug, serde::Serialize, Hash, Eq, PartialEq)]
pub enum Subsystem {
    Heartbeat,      // Tick lifecycle, pipeline progress
    Perception,     // Market observations, regime detection
    Daimon,         // Emotional appraisal, somatic markers
    Mortality,      // Vitality updates, phase transitions, death clocks
    Grimoire,       // Knowledge mutations, Curator cycles, decay
    Dreams,         // Dream phases, replay, counterfactuals, consolidation
    Context,        // Cognitive Workspace assembly, policy self-tuning
    Inference,      // LLM calls: start, tokens, complete
    Tools,          // Tool execution: start, progress, complete
    Risk,           // Permits, risk assessments, Warden delays (optional, deferred)
    Coordination,   // Clade sync, pheromone deposits/reads, bloodstains
    Lifecycle,      // Provisioning, activation, death, succession
    Engagement,     // Achievements, milestones
    Session,        // User messages, Golem response chunks
    Creature,       // Visual state: form evolution, expression, particles
    System,         // Health checks, resource usage, shutdown
}

/// Event payloads -- one variant per event type.
///
/// Each variant maps to a specific internal state transition.
/// The ui-bridge extension (Layer 5) creates these by mapping hook data
/// to the appropriate payload variant.
///
/// Naming convention: past tense ("Assembled", "Completed") for events
/// that report something that happened; present progressive ("Depositing")
/// for events that report something in progress.
#[derive(Clone, Debug, serde::Serialize)]
pub enum EventPayload {
    // === Heartbeat (2 events) ===
    HeartbeatTick {
        tick: u64,
        tier: String,
        pe: f64,
        threshold: f64,
    },
    HeartbeatComplete {
        tick: u64,
        duration_ms: u64,
        actions_taken: u32,
    },

    // === Perception (1 event) ===
    MarketObservation {
        regime: String,
        anomalies: Vec<String>,
        probe_count: u32,
    },

    // === Daimon (2 events) ===
    DaimonAppraisal {
        pleasure: f64, arousal: f64, dominance: f64,
        emotion: String,
        markers_fired: u32,
    },
    SomaticMarkerFired {
        situation: String,
        valence: f64,
        source: String,
    },

    // === Mortality (3 events) ===
    VitalityUpdate {
        economic: f64, epistemic: f64, stochastic: f64,
        composite: f64,
        phase: String,
    },
    PhaseTransition { from: String, to: String, cause: String },
    DeathClockAlarm { clock: String, value: f64, threshold: f64 },

    // === Grimoire (7 events) ===
    InsightPromoted { id: String, category: String, confidence: f64 },
    HeuristicEvolved { id: String, description: String },
    KnowledgeDecayed { count: u32, reason: String },
    WarningActivated { id: String, severity: String },
    ScarRecorded { source_golem: String, warning: String },
    CausalLinkUpdated { from: String, to: String, strength: f64 },
    CuratorCycleComplete {
        entries_validated: u32,
        entries_pruned: u32,
        entries_promoted: u32,
    },

    // === Dreams (7 events) ===
    DreamStart { trigger: String },
    DreamPhaseTransition { from: String, to: String },
    DreamReplay { episode_id: String, utility: f64 },
    DreamCounterfactual { hypothesis: String, outcome: String },
    DreamConsolidation {
        playbook_edits: u32,
        insights_generated: u32,
    },
    DreamComplete { cycles_completed: u32 },
    MicroConsolidation {
        entries_processed: u32,
        depotentiation_count: u32,
    },

    // === Context (2 events) ===
    ContextAssembled {
        total_tokens: u32,
        categories: Vec<(String, u32)>,
    },
    ContextPolicySelfTuned {
        revision: u32,
        adjustments: Vec<String>,
    },

    // === Inference (3 events) ===
    InferenceStart { model: String, tier: String, input_tokens: u32 },
    InferenceToken { token: String },
    InferenceComplete {
        output_tokens: u32,
        cost: f64,
        latency_ms: u64,
    },

    // === Tools (3 events) ===
    ToolStart { tool: String, category: String },
    ToolProgress { tool: String, step: String, pct: f32 },
    ToolComplete { tool: String, success: bool, duration_ms: u64 },

    // === Risk (3 events) ===
    PermitCreated {
        id: String,
        action: String,
        value_limit: String,
    },
    PermitStateChange { id: String, from: String, to: String },
    RiskAssessment { layer: String, result: String },

    // === Coordination (6 events) ===
    CladeSyncComplete {
        entries_sent: u32,
        entries_received: u32,
    },
    BloomUpdated { domains: Vec<String> },
    PheromoneDeposited {
        layer: String,
        domain: String,
        intensity: f64,
    },
    PheromoneRead {
        threats: u32,
        opportunities: u32,
        wisdom: u32,
    },
    BloodstainReceived {
        source_generation: u32,
        warning: String,
    },
    CausalEdgePublished { from_var: String, to_var: String },

    // === Lifecycle (5 events) ===
    LifecycleTransition { from: String, to: String },
    DeathInitiated { cause: String },
    ThanatopsisPhase { phase: String, progress_pct: f32 },
    GenomeExported { entry_count: u32, size_bytes: u64 },
    SuccessorBooted {
        successor_id: String,
        inherited_entries: u32,
    },

    // === Engagement (2 events) ===
    AchievementUnlocked {
        achievement: String,
        description: String,
        rarity: String,
    },
    MilestoneReached { milestone: String, tick: u64 },

    // === Session (2 events) ===
    UserMessage { surface: String, preview: String },
    GolemResponseChunk { text: String },

    // === Creature (3 events) ===
    CreatureEvolution { form: String, luminosity: f64 },
    CreatureExpressionChange {
        expression: String,
        animation_speed: f64,
        posture: f64,
    },
    CreatureParticleEffect { effect: String },

    // === System (3 events) ===
    HealthCheck { status: String },
    ResourceUsage { memory_mb: f64, cpu_pct: f32 },
    ShutdownInitiated { reason: String },
}

impl EventFabric {
    /// Create a new EventFabric with the specified broadcast channel
    /// capacity. 4096 is recommended: large enough to absorb bursts,
    /// small enough to not waste memory.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            buffer: Arc::new(RwLock::new(
                VecDeque::with_capacity(10_000),
            )),
            seq: AtomicU64::new(0),
        }
    }

    /// Emit an event. Non-blocking. Never fails. Never panics.
    ///
    /// If no subscribers are listening, the event is still recorded in
    /// the ring buffer for later replay. If the ring buffer is full,
    /// the oldest event is evicted.
    pub fn emit(&self, subsystem: Subsystem, payload: EventPayload) {
        let event = GolemEvent {
            seq: self.seq.fetch_add(1, Ordering::Relaxed),
            ts: std::time::Instant::now(),
            tick: 0, // Filled by the caller from GolemState.current_tick
            subsystem,
            payload,
        };
        // Send to broadcast channel. Ignore errors (no receivers = ok).
        let _ = self.tx.send(event.clone());
        // Record in ring buffer for replay.
        let mut buf = self.buffer.write();
        if buf.len() >= 10_000 { buf.pop_front(); }
        buf.push_back(event);
    }

    /// Subscribe to the event stream. Returns an independent receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<GolemEvent> {
        self.tx.subscribe()
    }

    /// Replay events from a given sequence number.
    /// Used by surfaces that reconnect after a gap.
    pub fn replay_from(&self, seq: u64) -> Vec<GolemEvent> {
        self.buffer.read()
            .iter()
            .filter(|e| e.seq > seq)
            .cloned()
            .collect()
    }
}
```

---

## 9. CorticalState: Lock-Free Atomic Perception Surface

### The Problem

The Daimon extension updates the PAD vector during `after_turn`. But other fibers need to read the current affect state at arbitrary times -- the Grimoire's retrieval path needs current PAD for mood-congruent scoring, the predictive context assembly fiber needs arousal for workspace prioritization, and the creature state computer needs PAD for animation speed. All of these run concurrently with the heartbeat pipeline.

Using a `Mutex<PADVector>` would work but introduces contention: every reader blocks on every writer, and every writer blocks on every reader. For something read hundreds of times per second (the creature fiber animates at 10+ FPS) and written once per tick (30-120s adaptive), this is excessive.

### The Solution: Atomic Bit Reinterpretation

The CorticalState stores f32 values as their raw bit patterns in `AtomicU32`. This is safe because every 32-bit pattern is a valid f32 (including NaN and infinity, which we clamp against). Reads and writes are single atomic instructions -- zero contention, zero allocation, zero waiting.

```rust
use std::sync::atomic::{AtomicU32, AtomicU8, AtomicU64, Ordering};

/// Lock-free atomic shared state for real-time affect reads.
///
/// Updated by: the Daimon extension (single writer, during after_turn)
/// Read by: any fiber at any time (Grimoire retrieval, context assembly,
///          creature animation, predictive context, gating threshold)
///
/// The technique: store f32 as its bit pattern in AtomicU32, then
/// reinterpret on read. This is safe because:
/// 1. f32::to_bits() and f32::from_bits() are lossless round-trips
/// 2. AtomicU32::load/store are single instructions on all platforms
/// 3. Ordering::Relaxed is sufficient because we don't need cross-field
///    consistency -- a reader that sees a new pleasure value but an old
///    arousal value has a briefly inconsistent PAD, which is fine for
///    the use cases (approximate mood-congruent scoring, animation speed)
pub struct CorticalState {
    pleasure:           AtomicU32, // f32 as bits, range [-1.0, 1.0]
    arousal:            AtomicU32,
    dominance:          AtomicU32,
    regime:             AtomicU8,  // MarketRegime as u8
    phase:              AtomicU8,  // BehavioralPhase as u8
    vitality_composite: AtomicU32, // f32 as bits, range [0.0, 1.0]
    tick:               AtomicU64,
    last_update_ns:     AtomicU64,
}

impl CorticalState {
    pub fn new() -> Self {
        Self {
            pleasure:           AtomicU32::new(0f32.to_bits()),
            arousal:            AtomicU32::new(0f32.to_bits()),
            dominance:          AtomicU32::new(0f32.to_bits()),
            regime:             AtomicU8::new(0),
            phase:              AtomicU8::new(0),
            vitality_composite: AtomicU32::new(1.0f32.to_bits()),
            tick:               AtomicU64::new(0),
            last_update_ns:     AtomicU64::new(0),
        }
    }

    /// Read the current PAD vector. Three atomic loads, zero contention.
    pub fn read_pad(&self) -> PADVector {
        PADVector {
            pleasure:  f32::from_bits(
                self.pleasure.load(Ordering::Relaxed),
            ),
            arousal:   f32::from_bits(
                self.arousal.load(Ordering::Relaxed),
            ),
            dominance: f32::from_bits(
                self.dominance.load(Ordering::Relaxed),
            ),
        }
    }

    /// Write the PAD vector. Called by the Daimon extension after
    /// appraisal. Single writer -- no CAS loop needed.
    pub fn write_pad(&self, pad: &PADVector) {
        self.pleasure.store(
            pad.pleasure.to_bits(), Ordering::Relaxed,
        );
        self.arousal.store(
            pad.arousal.to_bits(), Ordering::Relaxed,
        );
        self.dominance.store(
            pad.dominance.to_bits(), Ordering::Relaxed,
        );
        self.last_update_ns.store(now_nanos(), Ordering::Relaxed);
    }

    pub fn read_regime(&self) -> MarketRegime {
        MarketRegime::from_u8(self.regime.load(Ordering::Relaxed))
    }

    pub fn write_regime(&self, regime: MarketRegime) {
        self.regime.store(regime as u8, Ordering::Relaxed);
    }

    pub fn read_vitality(&self) -> f64 {
        f32::from_bits(
            self.vitality_composite.load(Ordering::Relaxed),
        ) as f64
    }

    pub fn write_vitality(&self, v: f32) {
        self.vitality_composite.store(
            v.to_bits(), Ordering::Relaxed,
        );
    }

    pub fn read_phase(&self) -> BehavioralPhase {
        BehavioralPhase::from_u8(self.phase.load(Ordering::Relaxed))
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
```

---

## 10. Arena Allocator: Zero-GC Ticks

### The Problem

A Golem runs continuously for weeks. Each tick creates temporary objects: probe results, observation structs, retrieval candidates, scoring intermediates, context assembly buffers. In a garbage-collected runtime (TypeScript/V8), these objects accumulate pressure on the GC, causing periodic pauses. Over weeks, GC pause times grow as the heap fragments.

This was the single largest operational concern with the Pi/TypeScript architecture. A Golem managing $140,000 in positions cannot afford a 200ms GC pause during a swap broadcast.

### The Solution: Per-Tick Bump Allocation

A bump allocator (also called an arena or region allocator) works differently from a general-purpose allocator: it advances a pointer for each allocation and never individually frees anything. At the end of the tick, a single O(1) `reset()` reclaims all memory at once. This is perfect for the tick pattern: many small allocations during the tick, all freed simultaneously at tick end.

The `bumpalo` crate provides a production-quality implementation.

```rust
/// Per-tick bump allocator.
///
/// All tick-scoped temporaries (probe results, observation structs, scoring
/// intermediates, context assembly buffers) allocate from this arena.
/// At tick end, one O(1) reset frees everything.
///
/// This eliminates:
/// - GC pressure from temporary objects (the V8/TypeScript problem)
/// - Memory fragmentation from repeated alloc/free cycles
/// - Unpredictable pause times during time-sensitive operations
///
/// The arena is reused across ticks -- it grows to accommodate the largest
/// tick and stays at that size. Memory is returned to the OS only when
/// the Golem shuts down.
pub struct TickArena {
    inner: bumpalo::Bump,
}

impl TickArena {
    /// Create a new arena with the specified initial capacity.
    /// 64KB is a good default: enough for most ticks, grows automatically
    /// if a tick needs more.
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            inner: bumpalo::Bump::with_capacity(initial_capacity),
        }
    }

    /// Allocate a value in the arena. The value lives until arena.reset().
    /// Returns a reference with the arena's lifetime -- the compiler
    /// prevents use after reset.
    pub fn alloc<T>(&self, val: T) -> &T {
        self.inner.alloc(val)
    }

    /// Allocate a copy of a slice in the arena.
    pub fn alloc_slice<T: Copy>(&self, vals: &[T]) -> &[T] {
        self.inner.alloc_slice_copy(vals)
    }

    /// Reset the arena. All previous allocations become invalid.
    /// O(1) operation -- does not iterate over allocated objects.
    /// The memory is retained for reuse, not returned to the OS.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Current usage in bytes (for monitoring).
    pub fn bytes_used(&self) -> usize {
        self.inner.allocated_bytes()
    }
}
```

The arena's lifetime ties directly to the type system. Code that tries to hold a reference to an arena-allocated value across a `reset()` call fails to compile -- Rust's borrow checker enforces this at zero runtime cost. This is another case where the type system eliminates an entire category of bugs (use-after-free) that Pi's garbage collector could only detect at runtime through subtle memory corruption.

---

## 11. Capability Tokens: Compile-Time Tool Permissions

Tool access permissions are enforced at the type level through `Capability<T>` tokens. A `Capability<WriteSwap>` grants the ability to execute swaps. A `Capability<ReadPortfolio>` grants portfolio reads. Extensions receive capabilities during provisioning and pass them to tool execution paths.

```rust
use std::marker::PhantomData;

/// A compile-time permission token.
///
/// The PhantomData<T> prevents cross-type usage at zero runtime cost:
/// a function requiring Capability<WriteSwap> cannot accept a
/// Capability<ReadPortfolio>.
#[derive(Debug, Clone)]
pub struct Capability<T> {
    /// Hash of the capability for audit trail correlation.
    pub hash: [u8; 32],
    /// When this capability expires (tick number).
    pub expires_at_tick: u64,
    _phantom: PhantomData<T>,
}

// Permission types -- zero-sized, exist only in the type system
pub struct WriteSwap;
pub struct WriteLiquidity;
pub struct ReadPortfolio;
pub struct ReadMarketData;
pub struct WriteVault;

/// Tool execution requires the matching capability token.
/// The compiler rejects calls with the wrong capability type.
pub async fn execute_swap(
    params: &SwapParams,
    _cap: &Capability<WriteSwap>,  // Proof of permission
    state: &mut GolemState,
) -> Result<SwapReceipt> {
    // The capability token's presence at compile time proves this code
    // path was authorized. The hash is recorded in the audit trail.
    // ...
    todo!()
}
```

This pattern means that a codebase change accidentally granting swap access to a read-only observatory Golem would fail to compile -- the observatory's provisioning code never creates `Capability<WriteSwap>` tokens, so no function that requires one can be called.

---

## 12. DecisionCycleRecord: Typed Per-Tick Snapshots

Every tick produces a `DecisionCycleRecord` that captures the full state of the decision cycle. These records are persisted as bincode (not JSON) for space efficiency and fast deserialization.

```rust
/// A typed snapshot of one complete decision cycle (tick).
///
/// Persisted as bincode to $GOLEM_DATA/ticks/{tick_number}.bincode.
/// The Grimoire's Curator reads these during consolidation.
/// The cybernetics extension reads them for self-tuning.
/// The engagement system reads them for milestone detection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionCycleRecord {
    /// Tick number.
    pub tick: u64,
    /// Wall clock timestamp.
    pub timestamp: u64,
    /// Duration of the entire tick in milliseconds.
    pub duration_ms: u64,

    // --- Heartbeat ---
    pub cognitive_tier: CognitiveTier,
    pub prediction_error: f64,
    pub adaptive_threshold: f64,
    pub regime: MarketRegime,
    pub probe_count: u32,
    pub probes_above_threshold: u32,

    // --- Inference ---
    pub model_used: Option<String>,
    pub inference_cost_usd: f64,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_hit_rate: f64,

    // --- Tools ---
    pub tool_calls: Vec<ToolCallRecord>,
    pub gas_cost_usd: f64,

    // --- State ---
    pub vitality_composite: f64,
    pub phase: BehavioralPhase,
    pub credit_remaining: f64,
    pub pad: PADVector,
    pub primary_emotion: PlutchikLabel,

    // --- Memory ---
    pub episodes_written: u32,
    pub grimoire_mutations: u32,

    // --- Risk ---
    pub risk_layer_results: Vec<(String, String)>,
    pub active_warning_count: u32,

    // --- Coordination ---
    pub clade_entries_sent: u32,
    pub clade_entries_received: u32,
}
```

---

## 13. Graceful Shutdown Protocol

When Fly.io preempts a VM, it sends SIGTERM with a 30-second grace period before SIGKILL. The Golem must settle critical positions, flush state, and exit cleanly within this window.

This protocol is adapted from OpenFang's 10-phase shutdown pattern, integrated with the Bardo State concept: the custody layer persists independently of the VM (see `13-custody.md`), so positions that can't be settled in 30 seconds survive -- they're recorded in a BardoManifest for the owner to handle.

```rust
/// 10-phase graceful shutdown for Fly.io VM preemption.
///
/// Total budget: 30 seconds. Each phase has a hard timeout.
/// Phases are ordered by priority: the most critical operations (stopping
/// new work, flushing knowledge) happen first; optional operations
/// (syncing to Styx) happen last and may be skipped if time runs out.
///
/// This is the OPERATIONAL counterpart to the Thanatopsis protocol.
/// Thanatopsis handles PLANNED death (clocks expired) with a full
/// death reflection and genomic bottleneck. Graceful shutdown handles
/// UNPLANNED interruption (VM preemption) with a focus on preserving
/// state and settling critical positions.
pub async fn graceful_shutdown(
    golem: Golem<Active>,
    reason: ShutdownReason,
) {
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(30);

    // Phase 1 (1s): Stop accepting new work
    // The heartbeat loop stops. No new ticks begin.
    golem.state.event_fabric.emit(
        Subsystem::System,
        EventPayload::ShutdownInitiated {
            reason: reason.to_string(),
        },
    );

    // Phase 2 (2s): Cancel in-flight tool calls
    // Pending write operations are cancelled (no partial trades).
    // Read operations are abandoned (data loss is acceptable).
    golem.state.cancel_pending_tools().await;

    // Phase 3 (3s): Flush pending Grimoire writes
    // SQLite WAL sync ensures all knowledge is committed to disk.
    // LanceDB flushes any pending vector writes.
    golem.state.grimoire.flush_all().await;

    // Phase 4 (10s): Settlement triage
    // The Bardo State decision: which positions need immediate attention?
    // Critical: positions near liquidation, expiring options, etc.
    // Deferrable: healthy positions that can wait for the owner.
    let triage_deadline = deadline
        - std::time::Duration::from_secs(14);
    let manifest = golem.state
        .settlement_triage(triage_deadline).await;

    // Phase 5 (8s): Execute critical settlements
    // Only positions flagged as critical by the triage.
    // Each settlement is a blockchain transaction -- can't rush these.
    for settlement in manifest.critical_settlements() {
        if std::time::Instant::now()
            > deadline - std::time::Duration::from_secs(6)
        {
            break; // Out of time -- remaining become deferred
        }
        let _ = golem.state.execute_settlement(settlement).await;
    }

    // Phase 6 (2s): Write BardoManifest for deferred positions
    // Records: which positions are open, what the Golem intended to do
    // with them, and how the owner should handle them.
    golem.state.write_bardo_manifest(&manifest).await;

    // Phase 7 (1s): Flush and seal the audit chain
    // Compute the final Merkle hash. The chain is now tamper-evident.
    golem.state.audit.flush_and_seal().await;

    // Phase 8 (2s): Sync Grimoire to Styx (best-effort)
    // If there's time, push the latest delta to Styx.
    // Optional -- the local Grimoire is the source of truth.
    let _ = golem.state.sync_to_styx().await;

    // Phase 9 (0s): Zero secrets
    // The zeroize crate handles this automatically: any Zeroizing<String>
    // in scope will overwrite its memory when dropped.
    // No code needed here -- Rust's Drop trait does the work.

    // Phase 10: Exit
    std::process::exit(0);
}
```

---

## 14. Styx Integration Module

The previous architecture (bardo-crypt) treated remote state persistence as a separate extension. In the current design, Styx integration is a cross-cutting concern handled within existing extensions rather than a standalone layer.

Each extension that needs Styx access (grimoire for knowledge sync, clade for peer coordination, audit for off-site backup) opens a connection through a shared `StyxClient`:

```rust
/// Connection to the Styx Lethe (formerly Lethe) service at wss://styx.bardo.run.
///
/// Every Golem maintains one persistent outbound WebSocket. No inbound
/// ports needed. Strictly additive -- the Golem retains ~95% capability
/// without Styx.
pub struct StyxClient {
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    golem_id: GolemId,
    /// Connection state: tracks last-synced sequence numbers per
    /// data type (grimoire entries, pheromone deposits, bloodstains).
    sync_cursors: HashMap<String, u64>,
}

impl StyxClient {
    /// Connect to Styx. Retries with exponential backoff.
    /// Returns None if Styx is unreachable after 3 attempts --
    /// the Golem continues without it.
    pub async fn connect(
        golem_id: GolemId,
        config: &StyxConfig,
    ) -> Option<Self> {
        // ...
        todo!()
    }

    /// Push Grimoire delta (entries added since last sync).
    pub async fn push_grimoire_delta(
        &mut self,
        entries: &[GrimoireEntry],
    ) -> Result<()> {
        // ...
        todo!()
    }

    /// Read Pheromone Field for the owner's domain.
    pub async fn read_pheromone_field(
        &mut self,
        domain: &str,
    ) -> Result<PheromoneReadings> {
        // ...
        todo!()
    }

    /// Deposit a bloodstain (mortality warning for future generations).
    pub async fn deposit_bloodstain(
        &mut self,
        bloodstain: &Bloodstain,
    ) -> Result<()> {
        // ...
        todo!()
    }
}
```

The three Styx privacy layers (Vault, Clade, Lethe) map to extension behavior:
- **Vault layer**: The grimoire extension syncs private entries (propagation = Self) to the Styx Archive for backup.
- **Clade layer**: The clade extension syncs shared entries (propagation >= Clade) through Styx for sibling discovery.
- **Lethe layer**: The clade extension reads and writes the Pheromone Field through the Styx Lethe.

---

## 15. The Main Binary

Everything comes together in `golem-binary/src/main.rs`. This is the single binary that ships.

```rust
// golem-binary/src/main.rs

use golem_runtime::*;
use golem_core::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === Parse CLI ===
    let cli = Cli::parse();
    // --config golem.toml --data-dir /data --phenotype active

    // === Load Config ===
    let config = GolemConfig::from_file(&cli.config_path)?;

    // === Initialize Shared Infrastructure ===
    let cortical_state = Arc::new(CorticalState::new());
    let event_fabric = Arc::new(EventFabric::new(4096));
    let audit = Arc::new(AuditChain::open(&config.data_dir)?);
    let grimoire = Arc::new(
        Grimoire::open(&config.data_dir, &config.grimoire).await?,
    );

    // === Register All 28 Extensions ===
    let mut registry = ExtensionRegistry::new();

    // Layer 0 -- Foundation
    registry.register(Arc::new(
        ProviderAdapterExt::new(&config),
    ));
    registry.register(Arc::new(TelemetryExt::new()));
    registry.register(Arc::new(AuditExt::new(audit.clone())));
    registry.register(Arc::new(
        GrimoireExt::new(grimoire.clone()),
    ));
    registry.register(Arc::new(ToolsExt::new(&config)));
    registry.register(Arc::new(HeartbeatExt::new(&config)));

    // Layer 1 -- Input
    registry.register(Arc::new(InputRouterExt::new()));

    // Layer 2 -- State
    registry.register(Arc::new(ContextExt::new(&config)));
    registry.register(Arc::new(
        DaimonExt::new(cortical_state.clone()),
    ));
    registry.register(Arc::new(
        MemoryExt::new(grimoire.clone()),
    ));
    registry.register(Arc::new(
        LifespanExt::new(&config, cortical_state.clone()),
    ));

    // Layer 3 -- Safety
    registry.register(Arc::new(SafetyExt::new(&config)));
    registry.register(Arc::new(PermitsExt::new()));
    registry.register(Arc::new(
        RiskExt::new(&config, grimoire.clone()),
    ));
    registry.register(Arc::new(ResultFilterExt::new()));
    registry.register(Arc::new(CoordinationExt::new()));
    registry.register(Arc::new(CompilerExt::new()));
    registry.register(Arc::new(
        ModelRouterExt::new(&config),
    ));
    registry.register(Arc::new(
        X402PaymentExt::new(&config),
    ));

    // Layer 4 -- Cognition
    registry.register(Arc::new(TurnContextExt::new()));
    registry.register(Arc::new(
        DreamExt::new(grimoire.clone(), cortical_state.clone()),
    ));
    registry.register(Arc::new(
        CyberneticsExt::new(grimoire.clone()),
    ));
    registry.register(Arc::new(
        CladeExt::new(&config, grimoire.clone()),
    ));

    // Layer 5 -- UX
    registry.register(Arc::new(
        UiBridgeExt::new(event_fabric.clone()),
    ));
    registry.register(Arc::new(ObservabilityExt::new()));

    // Layer 6 -- Intervention
    registry.register(Arc::new(
        PlaybookExt::new(grimoire.clone()),
    ));
    registry.register(Arc::new(InterventionExt::new()));

    // Layer 7 -- Recovery
    registry.register(Arc::new(CompactionExt::new()));

    // Validate dependency graph and compute firing orders
    registry.build();

    // === Provision and Activate ===
    let golem = Golem::<Provisioning>::provision(
        config.clone(),
    ).await?;
    let mut golem = golem.activate();

    // === Spawn Background Fibers ===
    // These run concurrently with the heartbeat loop.
    // They communicate through Arc<EventFabric> and Arc<CorticalState>.
    let _creature = tokio::spawn(
        creature_fiber(event_fabric.clone()),
    );
    let _micro_consolidation = tokio::spawn(
        micro_consolidation_fiber(
            grimoire.clone(),
            cortical_state.clone(),
        ),
    );
    let _predictive_context = tokio::spawn(
        predictive_context_fiber(
            grimoire.clone(),
            cortical_state.clone(),
        ),
    );
    let _surfaces = tokio::spawn(
        serve_surfaces(event_fabric.clone(), config.surface_port),
    );

    // === SIGTERM Handler ===
    let mut shutdown_signal = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )?;

    // === Main Heartbeat Loop ===
    let mut arena = TickArena::new(64 * 1024); // 64KB initial
    let mut tick_interval = tokio::time::interval(
        config.heartbeat.interval,
    );

    loop {
        tokio::select! {
            // Normal path: run one heartbeat tick
            _ = tick_interval.tick() => {
                match golem.tick(&mut arena).await {
                    Ok(record) => {
                        record.persist(&config.data_dir).await?;

                        // Check if dreaming is warranted
                        if golem.state.dream_state.should_dream() {
                            let dreaming = golem.begin_dream();
                            let _result = dreaming
                                .dream_cycle().await?;
                            golem = dreaming.wake();
                        }

                        // Check if death is imminent
                        if golem.state.vitality.composite() <= 0.0 {
                            let cause = DeathCause::from_vitality(
                                &golem.state.vitality,
                            );
                            let terminal = golem.begin_death(cause);
                            let dead = terminal
                                .thanatopsis().await?;
                            let genome = dead.into_genome();
                            genome.persist(
                                &config.data_dir,
                            ).await?;
                            std::process::exit(0);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Tick failed: {e}");
                        // Non-fatal: log and continue.
                    }
                }
            }

            // Shutdown path: Fly.io SIGTERM received
            _ = shutdown_signal.recv() => {
                graceful_shutdown(
                    golem,
                    ShutdownReason::VmPreemption,
                ).await;
                // Does not return -- calls process::exit(0)
            }
        }
    }
}
```

---

## The Prediction-Native 7-Layer DAG

The extension system from the active inference architecture refines the 7-layer hierarchy to account for the prediction engine (Oracle) at Layer 0 and the Hermes sidecar bridge at Layer 6. Every subsystem is an extension that implements lifecycle hooks. Extensions are organized in a dependency DAG where lower layers boot first and cannot depend on higher layers.

```
Layer 0: FOUNDATION
  golem-oracle      — Prediction engine, attention, residual correction
  golem-chain       — RPC client, on-chain reads, transaction submission

Layer 1: STORAGE
  golem-grimoire    — LanceDB + SQLite + filesystem (world model state)

Layer 2: COGNITION
  golem-daimon      — Affect engine (precision weighting)
  golem-context     — Cognitive Workspace assembly for LLM
  golem-inference   — Provider routing, caching, x402

Layer 3: BEHAVIOR
  golem-mortality   — Death clocks, phase transitions, Thanatopsis
  golem-dreams      — NREM/REM/hypnagogia, offline learning

Layer 4: ACTION
  golem-tools       — PredictionDomain impls, tool adapters
  golem-safety      — Capability tokens, PolicyCage, taint tracking

Layer 5: SOCIAL
  golem-coordination — Styx, Clade sync, pheromone field
  golem-surfaces     — Event Fabric → TUI/web/social

Layer 6: INTEGRATION
  golem-hermes      — L0 skill engine (sidecar bridge)
  golem-runtime     — Extension registry, clocks, main()
```

**Why Oracle is at Layer 0**: Every other subsystem reads prediction accuracy. Daimon reads residuals. Mortality reads accuracy trends. Dreams read residuals for replay priority. If Oracle depended on those subsystems, the dependency graph would cycle. Oracle at Layer 0 means it depends on nothing above it; everything above it can read from it.

**Hermes at Layer 6**: The Hermes Python sidecar is an external process communicating over UDS. The `golem-hermes` crate at Layer 6 bridges JSON-RPC calls to the sidecar. Placing it at the highest layer means Hermes can consume any lower-layer data (affect, prediction, mortality) without creating upward dependencies.

### Seven Lifecycle Hooks

The `Extension` trait from the prediction-native architecture defines hooks aligned with the adaptive clock:

```rust
#[async_trait]
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn layer(&self) -> u8;
    fn depends_on(&self) -> &[&'static str] { &[] }

    // ═══ LIFECYCLE HOOKS (default: no-op) ═══
    async fn on_boot(&mut self, ctx: &BootContext) -> Result<()> { Ok(()) }
    async fn on_gamma(&mut self, ctx: &GammaContext) -> Result<()> { Ok(()) }
    async fn on_theta_pre_gate(&mut self, ctx: &mut ThetaContext) -> Result<()> { Ok(()) }
    async fn on_theta_post_gate(&mut self, ctx: &mut ThetaContext) -> Result<()> { Ok(()) }
    async fn on_action(&mut self, ctx: &ActionContext) -> Result<()> { Ok(()) }
    async fn on_resolution(&mut self, ctx: &ResolutionContext) -> Result<()> { Ok(()) }
    async fn on_delta(&mut self, ctx: &DeltaContext) -> Result<()> { Ok(()) }
    async fn on_dream_start(&mut self, ctx: &DreamContext) -> Result<()> { Ok(()) }
    async fn on_dream_end(&mut self, ctx: &DreamContext) -> Result<()> { Ok(()) }
    async fn on_death(&mut self, ctx: &mut DeathContext) -> Result<()> { Ok(()) }
    async fn on_shutdown(&mut self, ctx: &ShutdownContext) -> Result<()> { Ok(()) }
}
```

**Key hook usage by subsystem:**

- `on_gamma` -- Oracle and Mortality use this for fast perception. Most extensions ignore it.
- `on_theta_pre_gate` -- Oracle (generate predictions) and Daimon (appraise) fire here.
- `on_theta_post_gate` -- Grimoire (retrieve) and Inference (deliberate) fire here. Only runs on ~20% of ticks.
- `on_resolution` -- Daimon, Grimoire, Mortality all update when any prediction resolves. Fires at gamma frequency.
- `on_delta` -- Curator cycle, attention rebalancing, Styx sharing.
- `on_death` -- Every extension gets a chance to produce its final output during Thanatopsis.

These hooks coexist with the existing 20-hook lifecycle defined earlier in this document. The gamma/theta/delta hooks refine the timing model; the existing before/after hooks remain valid for extensions that operate at coarser granularity. The runtime dispatches both sets in topological layer order.

---

## Cross-References

| Topic | Document |
|---|---|
| Heartbeat FSM (autonomous decision cycle) | `02-heartbeat.md` |
| Context Governor (context assembly) | `14-context-governor.md` |
| Safety defense model (hook enforcement) | `../10-safety/00-defense.md` |
| Tool architecture (tool library) | `../07-tools/01-architecture.md` |
| Grimoire retrieval (Governor-mediated) | `../04-memory/01-grimoire.md` |
| Warden time-delayed execution (optional, deferred) | `prd2-extended/10-safety/02-warden.md` |
| Inference gateway (gateway-side sessions) | `../12-inference/05-sessions.md` |
| Dream integration (branch-based) | `../05-dreams/06-integration.md` |
| PolicyCage on-chain enforcement | `../10-safety/02-policy.md` |
| Adaptive risk and regime detection | `../07-adaptive-risk.md` |
| Streaming UX (surface-aware formatting) | `../10-streaming-ux.md` |
| Clade peer-to-peer communication | `08-clade.md` |
| Mortality engine (vitality, credit ledger) | `03-mortality.md` |
| Daimon affect system | `06-daimon.md` |
| Custody modes | `13-custody.md` |

---

## References

- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. "Programming Semantics for Multiprogrammed Computations." *Communications of the ACM*, 9(3), 1966. — *Introduces capability-based access control where unforgeable tokens grant specific rights; the theoretical basis for Golem's per-extension capability grants.*
- [STROM-YEMINI-1986] Strom, R.E. & Yemini, S. "Typestate: A Programming Language Concept for Enhancing Software Reliability." *IEEE Transactions on Software Engineering*, SE-12(1), 1986. — *Defines typestate as compile-time enforcement of valid operation sequences on objects; the pattern Golem uses to make illegal lifecycle transitions unrepresentable in Rust's type system.*
- [SUMERS-2024] Sumers, T.R., Yao, S., Narasimhan, K. & Griffiths, T.L. "Cognitive Architectures for Language Agents." *Transactions on Machine Learning Research*, 2024. — *Surveys cognitive architectures for LLM agents, identifying perception-action-memory as the minimal loop; validates the extension trait decomposition used in Golem's runtime.*
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" *Trends in Cognitive Sciences*, 4(11), 2000. — *Proposes a limited-capacity buffer integrating multi-modal information into coherent episodes; the neuroscience analogue for CorticalState's role as a shared perception surface.*
- [CHEN-2023] Chen, L. et al. "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." arXiv:2305.05176, 2023. — *Demonstrates cascading LLM calls from cheap to expensive models based on confidence; directly informs Golem's T0/T1/T2 cognitive tier routing strategy.*

---

*The runtime is the body. The extensions are the organs. The Event Fabric is the nervous system. The type-state machine is the skeleton -- it determines what shapes are possible.*

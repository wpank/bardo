# golem-core

`golem-core` is the Layer 0 foundation crate for Bardo. It provides the shared Rust vocabulary that every later crate depends on: identity, runtime configuration, the lock-free cortical surface, the typed event bus, the extension hook skeleton, taint markers, HDC primitives, and the per-tick arena allocator.

## Features

- `GolemId` for ephemeral in-process process identity
- `GolemConfig` for the canonical `golem.toml` schema with `GOLEM_*` and `BARDO_*` overrides
- `CorticalState` and `CorticalSnapshot` for shared perception and state inspection
- `EventFabric` for non-blocking event emission plus bounded replay
- `Extension` and `ExtensionRegistry` for lifecycle hook orchestration
- `TaintLabel` and `TaintedString` for explicit information-flow tracking
- `CognitiveTier` for routing inference spend across `T0`, `T1`, and `T2`
- `HdcVector` for 10,240-bit hypervector operations
- `TickArena` for tick-scoped bump allocation
- `GolemError` and `Result` as the crate-wide error surface

## Getting Started

Import the crate root and use the re-exports directly:

```rust
use std::path::Path;

use golem_core::{
    CognitiveTier, CorticalState, EventFabric, EventPayload, GolemConfig, GolemId, Subsystem,
    TaintLabel, TaintedString, TickArena,
};

let golem_id = GolemId::new();
let config = GolemConfig::from_file(Path::new("golem.toml"))?;
let tier = CognitiveTier::try_from(1)?;

let cortical = CorticalState::new();
cortical.write_affect(0.5, -0.3, 0.1, 7);
let snapshot = cortical.snapshot();

let events = EventFabric::new(1_024);
events.emit(
    Subsystem::Heartbeat,
    42,
    EventPayload::HeartbeatComplete {
        tick: 42,
        duration_ms: 12,
        actions_taken: 3,
    },
);

let arena = TickArena::new();
let secret = TaintedString::new("0xabc".to_owned(), TaintLabel::WalletSecret);
let _ = arena.alloc(secret.value.clone());
```

`golem-core` is designed for direct root-level imports. Downstream crates should prefer the re-exported API instead of reaching into implementation modules.

## Configuration

`GolemConfig` is the canonical runtime schema loaded from `golem.toml`. The top-level sections are:

- `golem`
- `heartbeat`
- `inference`
- `safety`
- `custody`
- `styx`
- `succession`
- `daimon`
- `dreams`
- `oracle`
- `mortality`
- `compute`

Environment overrides use the `GOLEM_*` and `BARDO_*` prefixes. Example:

```rust
use golem_core::GolemConfig;

let config = GolemConfig::from_file(Path::new("golem.toml"))?;
let config = GolemConfig::from_str(
    r#"
    [golem]
    name = "oracle-3"
    "#,
)?;
```

Useful runtime overrides include `GOLEM_NAME`, `GOLEM_TICK_INTERVAL`, `GOLEM_MODE`, `GOLEM_CUSTODY_MODE`, `GOLEM_INFERENCE_PAYMENT`, `GOLEM_INFERENCE_DAILY_BUDGET`, `GOLEM_SUCCESSION_AUTO`, `GOLEM_SUCCESSION_BUDGET`, `GOLEM_DAIMON_ENABLED`, `GOLEM_DREAMS_ENABLED`, `GOLEM_ORACLE_ENABLED`, `GOLEM_COMPUTE_TIER`, `BARDO_STYX_ENABLED`, `BARDO_STYX_HOST`, `BARDO_CLADE_ENABLED`, `BARDO_STYX_DAILY_BUDGET`, `BARDO_STYX_MONTHLY_BUDGET`, `BARDO_IMMORTAL`, `BARDO_MORTALITY_ENABLED`, and `BARDO_STOCHASTIC_SEED`.

## API

### Identity and Errors

```rust
pub struct GolemId(uuid::Uuid);
impl GolemId {
    pub fn new() -> Self;
    pub const fn from_uuid(uuid: uuid::Uuid) -> Self;
    pub const fn as_uuid(&self) -> &uuid::Uuid;
}

pub type Result<T> = std::result::Result<T, GolemError>;

#[derive(thiserror::Error, Debug)]
pub enum GolemError {
    Config(String),
    Init(String),
    Extension { extension: String, source: anyhow::Error },
    EventFabric(String),
    CorticalState(String),
    Io(#[from] std::io::Error),
    TomlParse(#[from] toml::de::Error),
    Serde(#[from] serde_json::Error),
}
```

`GolemId` serializes transparently as a UUID and implements `Display` with the hyphenated UUID form. `GolemError` is the crate-wide error type used by config parsing, runtime plumbing, and downstream callers that want a single foundation error surface.

### Configuration

```rust
pub struct GolemConfig {
    pub golem: GolemSection,
    pub heartbeat: HeartbeatConfig,
    pub inference: InferenceConfig,
    pub safety: SafetyConfig,
    pub custody: CustodyConfig,
    pub styx: StyxConfig,
    pub succession: SuccessionConfig,
    pub daimon: DaimonConfig,
    pub dreams: DreamsConfig,
    pub oracle: OracleConfig,
    pub mortality: MortalityConfig,
    pub compute: ComputeConfig,
}

impl GolemConfig {
    pub fn from_file(path: &std::path::Path) -> Result<Self>;
    pub fn from_str(s: &str) -> Result<Self>;
    pub fn with_env_overrides(self) -> Result<Self>;
}
```

The configuration model is fully `serde`-driven and defaults missing sections so minimal files parse cleanly. The core schema is split into typed sections such as `GolemSection`, `HeartbeatConfig`, `InferenceConfig`, `SafetyConfig`, `CustodyConfig`, `StyxConfig`, `SuccessionConfig`, `DaimonConfig`, `DreamsConfig`, `OracleConfig`, `MortalityConfig`, and `ComputeConfig`.

### Cortical State

```rust
pub struct CorticalState;
impl CorticalState {
    pub fn new() -> std::sync::Arc<Self>;
    pub fn pad(&self) -> PadVector;
    pub fn prediction_accuracy(&self) -> f32;
    pub fn phase(&self) -> BehavioralPhase;
    pub fn snapshot(&self) -> CorticalSnapshot;
    pub fn write_affect(&self, pleasure: f32, arousal: f32, dominance: f32, emotion: u8);
    pub fn write_prediction(&self, accuracy: f32, trend: i8, categories: &[f32; 16], surprise: f32, pending: u32);
    pub fn write_attention(&self, universe: u32, active: u16, watched: u16, pending: u32);
    pub fn write_environment(&self, regime: u8, confidence: f32, gas_gwei: f32);
    pub fn write_mortality(&self, economic: f32, epistemic: f32, stochastic: f32, phase: u8);
    pub fn write_inference(&self, budget_remaining: f32, tier: u8);
    pub fn write_creative(&self, mode: u8, fragments: u32, last_novel_tick: u64);
    pub fn write_derived(&self, momentum: f32);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalSnapshot { /* point-in-time view of all signals */ }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PadVector {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum BehavioralPhase { Thriving, Stable, Conservation, Declining, Terminal }

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum PlutchikEmotion { Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation }
```

`CorticalState` is cache-aligned, lock-free, and intended to be shared across runtime consumers. `snapshot()` is a sequential read of the current signals and is useful for rendering and context assembly.

### Event Fabric

```rust
pub struct EventFabric;
impl EventFabric {
    pub fn new(capacity: usize) -> Self;
    pub fn emit(&self, subsystem: Subsystem, tick: u64, payload: EventPayload);
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<GolemEvent>;
    pub fn replay_from(&self, after_seq: u64) -> Vec<GolemEvent>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GolemEvent {
    pub seq: u64,
    pub ts_millis: u64,
    pub tick: u64,
    pub subsystem: Subsystem,
    pub payload: EventPayload,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Subsystem { Heartbeat, Perception, Daimon, Mortality, Grimoire, Dreams, Context, Inference, Tools, Risk, Coordination, Lifecycle, Engagement, Session, Creature, System }
```

`EventFabric` combines a live `tokio::sync::broadcast` channel with a bounded replay buffer. New subscribers can join live traffic and still catch up from the replay window if they need older events.

### Extension Hooks

```rust
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn layer(&self) -> u8;
    fn depends_on(&self) -> &[&str] { &[] }

    async fn on_session(&self, reason: SessionReason, ctx: &mut SessionCtx) -> anyhow::Result<()>;
    async fn on_input(&self, msg: &mut InputMessage, ctx: &InputCtx) -> anyhow::Result<InputAction>;
    async fn on_before_agent_start(&self, ctx: &mut AgentStartCtx) -> anyhow::Result<()>;
    async fn on_agent_start(&self, ctx: &AgentStartCtx) -> anyhow::Result<()>;
    async fn on_turn_start(&self, ctx: &TurnStartCtx) -> anyhow::Result<()>;
    async fn on_context(&self, messages: &mut Vec<AgentMessage>, ctx: &ContextCtx) -> anyhow::Result<()>;
    async fn on_before_provider_request(&self, ctx: &mut ProviderReqCtx) -> anyhow::Result<()>;
    async fn on_tool_call(&self, call: &ToolCall, ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction>;
    async fn on_tool_execution_start(&self, ctx: &ToolExecCtx) -> anyhow::Result<()>;
    async fn on_tool_execution_update(&self, ctx: &ToolExecCtx) -> anyhow::Result<()>;
    async fn on_tool_execution_end(&self, ctx: &ToolExecCtx) -> anyhow::Result<()>;
    async fn on_tool_result(&self, result: &mut ToolResult, ctx: &ToolResultCtx) -> anyhow::Result<()>;
    async fn on_turn_end(&self, ctx: &TurnEndCtx) -> anyhow::Result<()>;
    async fn on_agent_end(&self, ctx: &AgentEndCtx) -> anyhow::Result<()>;
    async fn on_after_turn(&self, ctx: &mut AfterTurnCtx) -> anyhow::Result<()>;
    async fn on_system_prompt(&self, prompt: &mut String, ctx: &PromptCtx) -> anyhow::Result<()>;
    async fn on_steer(&self, msg: &SteerMessage, ctx: &mut SteerCtx) -> anyhow::Result<()>;
    async fn on_send_message(&self, msg: &OutboundMessage, ctx: &MsgCtx) -> anyhow::Result<()>;
    async fn on_debug(&self, ctx: &DebugCtx) -> anyhow::Result<()>;
    async fn on_error(&self, err: &GolemError, ctx: &ErrorCtx) -> anyhow::Result<()>;
    async fn on_end(&self, ctx: &EndCtx) -> anyhow::Result<()>;
}

pub struct ExtensionRegistry;
impl ExtensionRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, ext: std::sync::Arc<dyn Extension>);
    pub fn build(&mut self);
    pub async fn fire_after_turn(&self, ctx: &mut AfterTurnCtx) -> anyhow::Result<()>;
    pub async fn fire_tool_call(&self, call: &ToolCall, ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction>;
    pub async fn fire_session(&self, reason: SessionReason, ctx: &mut SessionCtx) -> anyhow::Result<()>;
    pub async fn fire_end(&self, ctx: &EndCtx) -> anyhow::Result<()>;
}
```

The extension system gives later crates a typed hook surface without coupling them to one another. The registry validates dependency order before hooks fire.

### Taint, HDC, and Allocation

```rust
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaintLabel { Clean, Tainted, WalletSecret, LlmOutput, UserInput, ChainData }

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaintedString {
    pub value: String,
    pub label: TaintLabel,
}

pub struct HdcVector;
impl HdcVector {
    pub const fn zeros() -> Self;
    pub fn random() -> Self;
    pub fn bind(&self, other: &Self) -> Self;
    pub fn bundle(vectors: &[&Self]) -> Self;
    pub fn permute(&self, n: usize) -> Self;
    pub fn similarity(&self, other: &Self) -> f32;
}

pub struct TickArena;
impl TickArena {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn alloc<T>(&self, val: T) -> &T;
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &[T];
}
```

`TaintedString` keeps provenance visible at the type level. `HdcVector` is a simple 10,240-bit primitive for later HDC work, and `TickArena` gives the runtime a named arena allocator for tick-scoped temporary data.

## Architecture

```text
golem-core
├── identity: GolemId
├── config: GolemConfig and runtime sections
├── cortical: CorticalState, PadVector, BehavioralPhase, PlutchikEmotion
├── event: EventFabric, GolemEvent, EventPayload, Subsystem
├── extension: Extension trait, registry, hook contexts, hook actions
├── taint: TaintLabel, TaintedString
├── hdc: HdcVector
└── alloc: TickArena
```

Every later Rust crate imports `golem-core` either directly or through re-exports. That keeps foundational types centralized and prevents the workspace from growing a second copy of the same runtime vocabulary.

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout`, `Root Cargo.toml`, `Crate Inventory`, and `Dependency Rules`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `Workspace Dependency Inheritance`, and `Workspace Lints`
- `prd2/17-monorepo/03-conventions.md` sections `Rust Conventions` and `Workspace Dependency Inheritance`
- `prd2/01-golem/18-cortical-state.md` for the lock-free cortical surface, PAD vectors, and emotion classification
- `prd2/01-golem/19-config-and-operator-model.md` and `prd2/shared/config-reference.md` for the runtime configuration schema and environment overrides
- `prd2/01-golem/13a-runtime-extensions.md` and `prd2/01-golem/13b-runtime-extensions.md` for the extension hook surface and event bus
- `prd2/shared/glossary.md` for Bardo terminology used throughout the runtime vocabulary

# golem-core

## What It Is

`golem-core` is the Layer 0 foundation crate for Bardo. It defines the shared runtime vocabulary that every later crate imports: identity, `golem.toml` configuration, the lock-free cortical surface, the typed event fabric, the extension hook skeleton, taint labels, hyperdimensional primitives, and the per-tick arena allocator.

`GolemId` is the ephemeral in-process runtime identifier. It is distinct from the later chain-facing identity types and is safe to use for cross-crate references inside one running process.

## Features

- `GolemId` for UUID-backed runtime identity
- `GolemConfig` for the canonical `golem.toml` schema, including `mirage`
- `GolemError` and `Result` as the crate-wide typed error surface
- `CorticalState`, `CorticalSnapshot`, `PadVector`, `BehavioralPhase`, and `PlutchikEmotion` for zero-latency shared perception
- `EventFabric`, `GolemEvent`, `EventPayload`, and `Subsystem` for non-blocking broadcast plus bounded replay
- `Extension`, `ExtensionRegistry`, hook contexts, and hook action types for runtime lifecycle orchestration
- `TaintLabel` and `TaintedString` for explicit information-flow tracking
- `CognitiveTier` for routing inference spend across `T0`, `T1`, and `T2`
- `HdcVector` for 10,240-bit hypervector operations
- `TickArena` for tick-scoped bump allocation

## Getting Started

Import the crate root and use the re-exports directly:

```rust
use std::{convert::TryFrom, path::Path};

use golem_core::{
    CognitiveTier, CorticalState, EventFabric, EventPayload, GolemConfig, GolemId, Subsystem,
    TaintLabel, TaintedString, TickArena,
};

fn example() -> golem_core::Result<()> {
    let golem_id = GolemId::new();
    let tier = CognitiveTier::try_from(1)?;

    let config = GolemConfig::from_file(Path::new("golem.toml"))?;

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
    let copied = arena.alloc(secret.value.clone());

    let _ = (golem_id, tier, config, snapshot, copied);
    Ok(())
}
```

Downstream crates should prefer the root re-exports instead of reaching into implementation modules unless they need a specific module path for documentation or organization.

## Configuration

`GolemConfig` is the canonical runtime schema loaded from `golem.toml`. It follows the shared schema reference in `prd2/shared/config-reference.md`, and the live crate also carries the `mirage` sidecar section used by `mirage-rs`. All top-level sections have defaults, so an empty input string parses into a complete configuration. The current schema includes:

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
- `mirage`

Environment overrides use the `GOLEM_*` and `BARDO_*` prefixes. The most common runtime overrides include:

- `GOLEM_NAME`
- `GOLEM_TICK_INTERVAL`
- `GOLEM_MODE`
- `GOLEM_CUSTODY_MODE`
- `GOLEM_INFERENCE_PAYMENT`
- `GOLEM_INFERENCE_DAILY_BUDGET`
- `GOLEM_SPEND_LIMIT_TX`
- `GOLEM_SPEND_LIMIT_DAILY`
- `GOLEM_SUCCESSION_AUTO`
- `GOLEM_SUCCESSION_BUDGET`
- `GOLEM_DAIMON_ENABLED`
- `GOLEM_DREAMS_ENABLED`
- `GOLEM_ORACLE_ENABLED`
- `GOLEM_COMPUTE_TIER`
- `BARDO_STYX_ENABLED`
- `BARDO_STYX_HOST`
- `BARDO_CLADE_ENABLED`
- `BARDO_STYX_DAILY_BUDGET`
- `BARDO_STYX_MONTHLY_BUDGET`
- `BARDO_IMMORTAL`
- `BARDO_MIRAGE_URL`
- `BARDO_MIRAGE_HOST`
- `BARDO_MIRAGE_PORT`
- `BARDO_MIRAGE_TIMEOUT_MS`
- `BARDO_MIRAGE_RETRY_ATTEMPTS`
- `BARDO_MIRAGE_RETRY_BACKOFF_MS`

```rust
use std::path::Path;

use golem_core::GolemConfig;

fn load_configs() -> golem_core::Result<()> {
    let from_file = GolemConfig::from_file(Path::new("golem.toml"))?;
    let from_str = GolemConfig::from_str(
        r#"
        [golem]
        name = "oracle-3"
        "#,
    )?;

    let _ = (from_file, from_str);
    Ok(())
}
```

`GolemConfig::from_file`, `GolemConfig::from_str`, and `GolemConfig::with_env_overrides` all preserve the same schema and override behavior. Secrets are not stored in the config struct; they are loaded separately at runtime from the environment or a keystore. The full field table and environment-variable matrix live in `prd2/shared/config-reference.md`.

## API

### Identity And Errors

```rust
pub struct GolemId(uuid::Uuid);
impl GolemId {
    pub fn new() -> Self;
    pub fn from_uuid(uuid: uuid::Uuid) -> Self;
    pub fn as_uuid(&self) -> &uuid::Uuid;
}
impl std::fmt::Display for GolemId;
impl From<uuid::Uuid> for GolemId;
impl From<GolemId> for uuid::Uuid;

pub enum GolemError {
    Config(String),
    Init(String),
    Extension { extension: String, source: anyhow::Error },
    EventFabric(String),
    CorticalState(String),
    Io(std::io::Error),
    TomlParse(toml::de::Error),
    Serde(serde_json::Error),
}
pub type Result<T> = std::result::Result<T, GolemError>;
```

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
    pub mirage: MirageSection,
}
impl GolemConfig {
    pub fn from_file(path: &std::path::Path) -> Result<Self, GolemError>;
    pub fn from_str(s: &str) -> Result<Self, GolemError>;
    pub fn with_env_overrides(self) -> Result<Self, GolemError>;
}
```

The main config enums and supporting sections are:

- `StrategyCategory`
- `Network`
- `DeploymentMode`
- `CustodyMode`
- `TransferRestriction`
- `InferencePayment`
- `ProviderType`
- `AppraisalModel`
- `DreamSchedule`
- `ComputeTier`
- `MirageSection`

### Cortical State

```rust
#[repr(C, align(64))]
pub struct CorticalState { /* private atomic fields */ }
impl CorticalState {
    pub fn new() -> std::sync::Arc<Self>;
    pub fn pad(&self) -> PadVector;
    pub fn prediction_accuracy(&self) -> f32;
    pub fn phase(&self) -> BehavioralPhase;
    pub fn snapshot(&self) -> CorticalSnapshot;
    pub fn write_affect(&self, pleasure: f32, arousal: f32, dominance: f32, emotion: u8);
    pub fn write_prediction(
        &self,
        accuracy: f32,
        trend: i8,
        categories: &[f32; 16],
        surprise: f32,
        pending: u32,
    );
    pub fn write_attention(&self, universe: u32, active: u16, watched: u16, pending: u32);
    pub fn write_environment(&self, regime: u8, confidence: f32, gas_gwei: f32);
    pub fn write_mortality(&self, economic: f32, epistemic: f32, stochastic: f32, phase: u8);
    pub fn write_inference(&self, budget_remaining: f32, tier: u8);
    pub fn write_creative(&self, mode: u8, fragments: u32, last_novel_tick: u64);
    pub fn write_derived(&self, momentum: f32);
}
pub struct CorticalSnapshot { /* point-in-time readout */ }
pub struct PadVector { pub pleasure: f64, pub arousal: f64, pub dominance: f64 }
impl PadVector {
    pub const ZERO: Self;
    pub fn clamp(&self, min: f64, max: f64) -> Self;
}
pub enum BehavioralPhase { Thriving, Stable, Conservation, Declining, Terminal }
impl BehavioralPhase {
    pub fn from_u8(v: u8) -> Self;
}
pub enum PlutchikEmotion {
    Joy,
    Trust,
    Fear,
    Surprise,
    Sadness,
    Disgust,
    Anger,
    Anticipation,
}
impl PlutchikEmotion {
    pub fn from_pad(pad: &PadVector) -> Self;
}
```

`snapshot()` is a best-effort read of all 32 signals. It is not transactional, so the returned values can span two ticks if writers are active concurrently.

### Event Fabric

```rust
pub struct EventFabric { /* live broadcast sender + replay buffer + sequence counter */ }
impl EventFabric {
    pub fn new(capacity: usize) -> Self;
    pub fn emit(&self, subsystem: Subsystem, tick: u64, payload: EventPayload);
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<GolemEvent>;
    pub fn replay_from(&self, after_seq: u64) -> Vec<GolemEvent>;
}
pub struct GolemEvent {
    pub seq: u64,
    pub ts_millis: u64,
    pub tick: u64,
    pub subsystem: Subsystem,
    pub payload: EventPayload,
}
pub enum Subsystem {
    Heartbeat,
    Perception,
    Daimon,
    Mortality,
    Grimoire,
    Dreams,
    Context,
    Inference,
    Tools,
    Risk,
    Coordination,
    Lifecycle,
    Engagement,
    Session,
    Creature,
    System,
}
pub enum EventPayload { /* 50+ typed variants across 16 subsystems */ }
```

`emit` is non-blocking, `subscribe` attaches a live receiver, and `replay_from` returns buffered events with `seq >= after_seq`.

The live broadcast channel capacity is caller-supplied, while the replay ring buffer is fixed at 10,000 events.

### Extension Hooks

```rust
#[async_trait::async_trait]
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn layer(&self) -> u8;
    fn depends_on(&self) -> &[&str] { &[] }

    async fn on_session(&self, _reason: SessionReason, _ctx: &mut SessionCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_input(&self, _msg: &mut InputMessage, _ctx: &InputCtx) -> anyhow::Result<InputAction> { Ok(InputAction::Pass) }
    async fn on_before_agent_start(&self, _ctx: &mut AgentStartCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_agent_start(&self, _ctx: &AgentStartCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_turn_start(&self, _ctx: &TurnStartCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_context(&self, _messages: &mut Vec<AgentMessage>, _ctx: &ContextCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_before_provider_request(&self, _ctx: &mut ProviderReqCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_call(&self, _call: &ToolCall, _ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction> { Ok(ToolAction::Allow) }
    async fn on_tool_execution_start(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_execution_update(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_execution_end(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_result(&self, _result: &mut ToolResult, _ctx: &ToolResultCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_turn_end(&self, _ctx: &TurnEndCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_agent_end(&self, _ctx: &AgentEndCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_after_turn(&self, _ctx: &mut AfterTurnCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_system_prompt(&self, _prompt: &mut String, _ctx: &PromptCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_steer(&self, _msg: &SteerMessage, _ctx: &mut SteerCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_send_message(&self, _msg: &OutboundMessage, _ctx: &MsgCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_debug(&self, _ctx: &DebugCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_error(&self, _err: &GolemError, _ctx: &ErrorCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_end(&self, _ctx: &EndCtx) -> anyhow::Result<()> { Ok(()) }
}

pub struct ExtensionRegistry {
    /* registered extensions and precomputed firing orders */
}
impl ExtensionRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, ext: std::sync::Arc<dyn Extension>);
    pub fn build(&mut self);
    pub async fn fire_after_turn(&self, ctx: &mut AfterTurnCtx) -> anyhow::Result<()>;
    pub async fn fire_tool_call(&self, call: &ToolCall, ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction>;
    pub async fn fire_session(&self, reason: SessionReason, ctx: &mut SessionCtx) -> anyhow::Result<()>;
    pub async fn fire_end(&self, ctx: &EndCtx) -> anyhow::Result<()>;
}
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
```

The registry validates extension names, dependency edges, layer ordering, and cycles before dispatch. Once built, it uses the dependency-respecting order for all hook firing helpers. Tool-call hooks are merged conservatively: `Block` short-circuits and `Modify` is last-write-wins.

### Taint, HDC, and Allocation

```rust
pub enum TaintLabel {
    Clean,
    Tainted,
    WalletSecret,
    LlmOutput,
    UserInput,
    ChainData,
}
pub struct TaintedString {
    pub value: String,
    pub label: TaintLabel,
}
impl TaintedString {
    pub fn new(value: String, label: TaintLabel) -> Self;
    pub fn clean(value: String) -> Self;
    pub fn is_tainted(&self) -> bool;
}

pub struct HdcVector { /* 10,240-bit sparse distributed vector */ }
impl HdcVector {
    pub fn zeros() -> Self;
    pub fn random() -> Self;
    pub fn bind(&self, other: &Self) -> Self;
    pub fn bundle(vectors: &[&Self]) -> Self;
    pub fn permute(&self, n: usize) -> Self;
    pub fn similarity(&self, other: &Self) -> f32;
}

pub struct TickArena { /* bumpalo-backed per-tick arena */ }
impl TickArena {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn alloc<T>(&self, val: T) -> &T;
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &[T];
}
```

`TaintedString` keeps provenance explicit in the type system. `HdcVector` is a correctness-first stub for later hyperdimensional work. `TickArena` makes tick-scoped temporary allocation explicit and cheap to reset.

### Cognitive Tiers

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum CognitiveTier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
}
impl core::convert::TryFrom<u8> for CognitiveTier {
    type Error = GolemError;
}
impl From<CognitiveTier> for u8;
```

### Root Re-Exports

The crate root re-exports the documented public surface from each module, including the hook context types and action types used by later runtime crates:

- `GolemConfig`
- `GolemError`
- `Result`
- `GolemId`
- `CognitiveTier`
- `CorticalState`, `CorticalSnapshot`, `PadVector`, `BehavioralPhase`, `PlutchikEmotion`
- `EventFabric`, `GolemEvent`, `EventPayload`, `Subsystem`
- `Extension`, `ExtensionRegistry`, `HookId`
- `SessionReason`
- `SessionCtx`, `InputCtx`, `AgentStartCtx`, `TurnStartCtx`, `ContextCtx`, `ProviderReqCtx`, `ToolCallCtx`, `ToolExecCtx`, `ToolResultCtx`, `TurnEndCtx`, `AgentEndCtx`, `AfterTurnCtx`, `PromptCtx`, `SteerCtx`, `MsgCtx`, `DebugCtx`, `ErrorCtx`, `EndCtx`
- `InputMessage`, `InputAction`
- `ToolCall`, `ToolAction`, `ToolResult`
- `AgentMessage`, `SteerMessage`, `OutboundMessage`
- `TaintLabel`, `TaintedString`
- `HdcVector`
- `TickArena`

## Architecture

`golem-core` is the only crate in the workspace that other crates are expected to import freely without creating a dependency-layer violation. Its job is to keep foundational concepts centralized:

- configuration stays in one schema
- events stay typed and replayable
- shared perception stays atomic and cache-aligned
- runtime hooks stay on one extension trait
- taint and allocation contracts stay explicit

The design and terminology come from `prd2/01-golem/00-overview.md`, `prd2/01-golem/13a-runtime-extensions.md`, `prd2/01-golem/13b-runtime-extensions.md`, `prd2/01-golem/18-cortical-state.md`, `prd2/01-golem/19-config-and-operator-model.md`, and the shared reference files under `prd2/shared/`.

That keeps the rest of the workspace from re-defining the same concepts in incompatible ways.

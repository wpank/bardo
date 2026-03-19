# golem-core

`golem-core` is the Layer 0 foundation crate for Bardo. It defines the shared vocabulary every later crate imports: identity, configuration, error handling, taint labels, the lock-free cortical surface, the event fabric, the extension trait skeleton, HDC primitives, and the per-tick arena wrapper.

## Features

- `GolemId`: UUID-backed runtime identity for in-process references
- `GolemConfig`: TOML runtime schema with `GOLEM_*` and `BARDO_*` environment overrides
- `CognitiveTier`: `T0`, `T1`, `T2` inference gate for cost-aware routing
- `CorticalState`: cache-aligned atomic perception surface with writer helpers and snapshots
- `EventFabric`: non-blocking broadcast bus with a bounded replay buffer
- `Extension` and `ExtensionRegistry`: async hook trait plus dependency-validated dispatcher
- `TaintLabel` and `TaintedString`: explicit information-flow markers
- `HdcVector`: 10,240-bit hypervector primitive stub
- `TickArena`: `bumpalo`-backed arena that resets at tick boundaries
- `GolemError` and `Result`: crate-wide typed error surface

## Getting Started

Import the crate root and use the re-exported types directly:

```rust
use std::{path::Path, convert::TryFrom};

use golem_core::{CognitiveTier, EventFabric, GolemConfig, GolemId};

let id = GolemId::new();
let config = GolemConfig::from_file(Path::new("golem.toml"))?;
let tier = CognitiveTier::try_from(1)?;
let fabric = EventFabric::new(1_024);

fabric.emit(
    golem_core::Subsystem::Heartbeat,
    42,
    golem_core::EventPayload::HeartbeatComplete {
        tick: 42,
        duration_ms: 12,
        actions_taken: 3,
    },
);
```

`golem-core` is designed to be imported from the crate root. The root re-exports the full public surface, so downstream crates can stay on the stable `golem_core::...` path instead of reaching into implementation modules.

## Configuration

`GolemConfig` is the canonical `golem.toml` schema. It includes the top-level sections that Bardo uses at runtime:

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

Load configuration from a file or string, then apply the environment overlay:

```rust
use std::path::Path;

use golem_core::GolemConfig;

let config = GolemConfig::from_file(Path::new("golem.toml"))?;
let config = GolemConfig::from_str("[golem]\nname = \"oracle-3\"\n")?;
```

`from_file` reads TOML from disk and then calls `with_env_overrides`. `from_str` parses TOML and applies the same environment lookup without file I/O. The canonical field names, defaults, and environment mappings live in `prd2/shared/config-reference.md`, while the operator-facing file layout and hot-reload guidance are documented in `prd2/01-golem/19-config-and-operator-model.md`.

## API

### Identity and Errors

```rust
pub struct GolemId(uuid::Uuid);
impl GolemId {
    pub fn new() -> Self;
    pub fn from_uuid(uuid: uuid::Uuid) -> Self;
    pub fn as_uuid(&self) -> &uuid::Uuid;
}

#[derive(thiserror::Error, Debug)]
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

`GolemId` serializes transparently as a UUID. `GolemError::Extension` preserves the source error with `anyhow::Error` so hook failures keep their full context.

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

pub enum CognitiveTier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
}
```

Every config section derives `Serialize`, `Deserialize`, `Clone`, and `Debug`, and the optional sections use `Default` so minimal files parse cleanly.

### Cortical Surface

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

pub struct CorticalSnapshot { /* all 32 signals */ }
pub struct PadVector { pub pleasure: f64, pub arousal: f64, pub dominance: f64 }
pub enum BehavioralPhase { Thriving, Stable, Conservation, Declining, Terminal }
pub enum PlutchikEmotion { Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation }
```

`CorticalState` uses atomic loads and stores with Acquire/Release ordering. `PadVector::ZERO`, `PadVector::clamp`, `BehavioralPhase::from_u8`, and `PlutchikEmotion::from_pad` are all part of the public API. `snapshot()` reads all signals sequentially, so it is useful for rendering and context assembly, but it is not transactional across the full struct.

### Event Fabric

```rust
pub struct EventFabric;
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
```

`EventFabric` combines a live `tokio::sync::broadcast` channel with a replay buffer capped at 10,000 events. `emit()` never blocks; it appends to the buffer and forwards the event to live subscribers. `GolemEvent::ts_millis` is a wall-clock timestamp in Unix epoch milliseconds.

`EventPayload` covers the runtime event vocabulary across heartbeat, perception, daimon, mortality, grimoire, dreams, context, inference, tools, risk, coordination, lifecycle, engagement, session, creature, and system events. `Subsystem` identifies which part of the runtime emitted the event.

### Extension System

```rust
#[async_trait::async_trait]
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

The registry validates dependency names and layer order, then computes a stable firing order. For tool calls, `Block` short-circuits and `Modify` can be refined by later hooks.

### Taint, HDC, and Allocator

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

pub struct HdcVector { /* 10,240 bits */ }
impl HdcVector {
    pub fn zeros() -> Self;
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

`TaintedString` does not dereference to `String`, which keeps the taint boundary explicit. `HdcVector::bundle` uses strict majority voting, ties fall back to `0`, and `similarity` returns normalized Hamming similarity. `TickArena` is a thin named wrapper around `bumpalo::Bump`.

## Architecture

`golem-core` is the root of the workspace dependency graph. Later crates build on it, but it depends on no other workspace crates.

```
golem_core
├── id
├── error
├── taint
├── cognitive
├── config
├── cortical
├── event
├── extension
├── hdc
└── alloc
```

The crate root re-exports the stable public API so downstream crates can import from `golem_core` directly. That keeps the shared vocabulary small and avoids dependency drift across later layers.

## References

- `prd2/01-golem/00-overview.md` sections `The Golem Container`, `The CorticalState`, `Architecture: 7-Layer Dependency Hierarchy`, and `Key Architectural Decisions`
- `prd2/01-golem/13a-runtime-extensions.md` sections `2. The Extension Trait: 20 Lifecycle Hooks` and `5. The Extension Registry`
- `prd2/01-golem/13b-runtime-extensions.md` sections `8. Event Fabric: The Nervous System`, `9. CorticalState: Lock-Free Atomic Perception Surface`, and `10. Arena Allocator: Zero-GC Ticks`
- `prd2/01-golem/18-cortical-state.md` sections `The struct`, `Reading and writing`, `Initialization`, and `Plutchik Emotion Labels`
- `prd2/01-golem/19-config-and-operator-model.md` sections `Config Files Overview`, `golem.toml`, and `Environment Variable Substitution`
- `prd2/shared/config-reference.md` sections `Config Resolution Order`, `Env Var Naming Convention`, `[golem]`, `[heartbeat]`, `[inference]`, `[safety]`, `[custody]`, `[styx]`, `[succession]`, `[daimon]`, `[dreams]`, `[oracle]`, `[mortality]`, and `[compute]`
- `prd2/shared/glossary.md` entries for `Golem`, `CorticalState`, `Event Fabric`, `Extension`, `HDC`, `HyperVector`, and `TaintLabel`

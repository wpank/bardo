# Plan 02: Core Types & Event Fabric

## Context

This plan implements `golem-core`, the zero-dependency foundation crate that every other crate in the workspace imports. It defines the shared type vocabulary: the Golem's identity, its configuration schema, the lock-free perception surface, the typed event broadcast bus, the extension trait skeleton, information-flow taint markers, and HDC primitive stubs.

`golem-core` has no workspace-internal dependencies. It is Layer 0 in the crate graph.

---

## Previous Plan

Plan 01 (Workspace Scaffold) created the full Cargo workspace at `/Users/will/dev/uniswap/gringotts/bardo/`. It produced:

- `Cargo.toml` (workspace root) with `[workspace.dependencies]` for all external crates
- Crate shells for all 17 library crates (`crates/`) and 4 app binaries (`apps/`)
- `sidecar/tools-ts/` stub
- `cargo check --workspace` passes (all shells compile with empty `src/lib.rs`)

The workspace structure matches `prd2/17-monorepo/00-packages.md` canonical names.

---

## Prerequisites

- **Plan 01: Workspace Scaffold** — provides: workspace `Cargo.toml`, `crates/golem-core/Cargo.toml` shell, `cargo check` passes

---

## Imports (from earlier plans)

None. `golem-core` has zero workspace-internal dependencies.

---

## Exports (for later plans)

All types below are defined in this plan and used by Plans 03+.

> **`GolemId` vs `GolemIdentityId`**: `GolemId` (this plan) is a UUID-based internal runtime identifier — ephemeral, not on-chain. `GolemIdentityId` (Plan 45) is a `[u8; 32]` secp256k1 pubkey hash — persistent, on-chain ERC-8004 identity. Use `GolemId` for in-process cross-crate referencing; use `GolemIdentityId` for anything that touches the chain, wallets, or cross-agent reputation.

```rust
// ── golem_core::id ──────────────────────────────────────────────────────────
pub struct GolemId(uuid::Uuid);
impl GolemId {
    pub fn new() -> Self;
    pub fn from_uuid(uuid: uuid::Uuid) -> Self;
    pub fn as_uuid(&self) -> &uuid::Uuid;
}
impl std::fmt::Display for GolemId { ... }
impl serde::Serialize for GolemId { ... }
impl<'de> serde::Deserialize<'de> for GolemId { ... }

// ── golem_core::config ──────────────────────────────────────────────────────
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
    pub fn from_file(path: &std::path::Path) -> Result<Self, GolemError>;
    pub fn from_str(s: &str) -> Result<Self, GolemError>;
    pub fn with_env_overrides(self) -> Result<Self, GolemError>;
}

pub struct GolemSection {
    pub name: String,
    pub strategy_category: StrategyCategory,
    pub network: Network,
    pub mode: DeploymentMode,
    pub funding: String,
    pub custody_mode: CustodyMode,
    pub transfer_restriction: TransferRestriction,
    pub schema_version: u32,
}

pub enum StrategyCategory { Dca, Yield, Lp, Momentum, MultiProtocol, Custom }
pub enum Network { Base, BaseSepolia, Anvil }
pub enum DeploymentMode { Hosted, SelfHosted }
pub enum CustodyMode { Delegation, Embedded, LocalKey }
pub enum TransferRestriction { Strict, Clade, Network, Unrestricted }

pub struct HeartbeatConfig {
    pub base_interval_seconds: u64,         // default: 15, env: GOLEM_TICK_INTERVAL
    pub base_deliberation_threshold: f64,   // default: 0.3, env: GOLEM_DELIBERATION_THRESHOLD
    pub max_daily_cost_usd: f64,            // default: 10.0, env: GOLEM_MAX_DAILY_COST
    pub cost_warning_threshold: f64,        // default: 0.7
    pub cost_soft_cap_threshold: f64,       // default: 0.9
    pub write_batch_size: u32,              // default: 25
    pub write_batch_flush_interval_ms: u64, // default: 30000
    pub regime_multipliers: RegimeMultipliers,
    pub probe_thresholds: ProbeThresholds,
}

pub struct RegimeMultipliers {
    pub trending_up: f64,    // default: 1.0
    pub trending_down: f64,  // default: 0.5
    pub volatile: f64,       // default: 0.3
    pub range_bound: f64,    // default: 2.0
    pub unknown: f64,        // default: 0.8
}

pub struct ProbeThresholds {
    pub price_delta_low_bps: u32,    // default: 50
    pub price_delta_high_bps: u32,   // default: 200
    pub health_factor_low: f64,      // default: 1.5
    pub health_factor_high: f64,     // default: 1.2
    pub world_model_drift_low: f64,  // default: 0.10
    pub world_model_drift_high: f64, // default: 0.25
}

pub struct InferenceConfig {
    pub payment: InferencePayment,
    pub daily_budget_usd: f64, // default: 5.0, env: GOLEM_INFERENCE_DAILY_BUDGET
    pub providers: Vec<InferenceProvider>,
}
pub enum InferencePayment { GolemWallet, Prepaid, Diem, Composite }
pub struct InferenceProvider { pub provider_type: ProviderType, pub diem: bool }
pub enum ProviderType { Bardo, Venice, Bankr, Anthropic, OpenAi, Google, Deepseek, Local }

pub struct SafetyConfig {
    pub approved_assets: Vec<String>,
    pub approved_protocols: Vec<String>,
    pub max_asset_count: u32,           // default: 10
    pub max_position_bps: u32,          // default: 2500
    pub max_concentration_bps: u32,     // default: 3000
    pub min_collateral_ratio_bps: u32,  // default: 12500
    pub max_drawdown_bps: u32,          // default: 1300
    pub drawdown_window: u64,           // default: 86400
    pub min_rebalance_interval: u64,    // default: 3600
    pub max_rebalances_per_day: u32,    // default: 24
    pub allow_arbitrary_calldata: bool, // default: false
    pub sanction_oracle: String,
    pub spending_limits: SpendingLimits,
}
pub struct SpendingLimits {
    pub per_transaction: u64, // default: 10000, env: GOLEM_SPEND_LIMIT_TX
    pub per_session: u64,     // default: 50000
    pub per_day: u64,         // default: 100000, env: GOLEM_SPEND_LIMIT_DAILY
}

pub struct CustodyConfig {
    pub mode: CustodyMode, // env: GOLEM_CUSTODY_MODE
    pub delegation_bounds: Option<DelegationBounds>,
}
pub struct DelegationBounds {
    pub max_daily_spend_usd: f64,
    pub max_total_calls: u64,
    pub expires_at: Option<u64>,
    pub allowed_targets: Vec<String>,
}

pub struct StyxConfig {
    pub enabled: bool,          // default: true, env: BARDO_STYX_ENABLED
    pub host: String,           // default: "styx.bardo.run", env: BARDO_STYX_HOST
    pub vault: StyxVaultConfig,
    pub clade: StyxCladeConfig,
    pub lethe: StyxLetheConfig,
    pub pheromone: StyxPheromoneConfig,
    pub marketplace: StyxMarketplaceConfig,
    pub budget: StyxBudgetConfig,
}
pub struct StyxCladeConfig {
    pub enabled: bool,                   // default: true, env: BARDO_CLADE_ENABLED
    pub sync_interval_ticks: u64,        // default: 50
    pub immediate_warnings: bool,        // default: true
    pub immediate_bloodstains: bool,     // default: true
    pub p2p: StyxP2pConfig,
}
pub struct StyxBudgetConfig {
    pub max_per_tick: f64,      // default: 0.01
    pub daily_budget: f64,      // default: 0.50, env: BARDO_STYX_DAILY_BUDGET
    pub monthly_budget: f64,    // default: 10.00, env: BARDO_STYX_MONTHLY_BUDGET
}

pub struct SuccessionConfig {
    pub enabled: bool,                    // default: true
    pub auto: bool,                       // default: false, env: GOLEM_SUCCESSION_AUTO
    pub budget_usdc: f64,                 // default: 50.0, env: GOLEM_SUCCESSION_BUDGET
    pub strategy_drift_allowed: f64,      // default: 0.3
    pub inherit_grimoire: bool,           // default: true
    pub inheritance_confidence: f64,      // default: 0.4 (max 0.7, protocol invariant)
    pub min_playbook_divergence: f64,     // default: 0.15
    pub inherit_from_clade: bool,         // default: true
    pub use_novelty_ranking: bool,        // default: true
}

pub struct DaimonConfig {
    pub enabled: bool,                   // default: true, env: GOLEM_DAIMON_ENABLED
    pub appraisal_model: AppraisalModel, // default: ChainOfEmotion, env: GOLEM_APPRAISAL_MODEL
    pub mood_decay_rate: f64,            // default: 0.95
    pub mortality_aware_affect: bool,    // default: true
    pub grief_duration_ticks: u32,       // default: 100
    pub record_emotional_context: bool,  // default: true
}
pub enum AppraisalModel { ChainOfEmotion, RuleBased, Disabled }

pub struct DreamsConfig {
    pub enabled: bool,                              // default: true, env: GOLEM_DREAMS_ENABLED
    pub schedule: DreamSchedule,                    // default: Hybrid, env: GOLEM_DREAM_SCHEDULE
    pub autonomous_threshold: f64,                  // default: 0.8
    pub budget_fraction: f64,                       // default: 0.08
    pub terminal_cutoff: f64,                       // default: 0.15
    pub max_staged_revisions: u32,                  // default: 10
    pub dream_inference_provider: Option<String>,   // env: GOLEM_DREAM_INFERENCE_PROVIDER
    pub web_search_budget_per_cycle_usdc: f64,      // default: 0.05
    pub windows: Vec<DreamWindow>,
    pub phase_scaling: DreamPhaseScaling,
}
pub enum DreamSchedule { Operator, Autonomous, Hybrid }
pub struct DreamPhaseScaling {
    pub nrem: f64,        // default: 0.45
    pub rem: f64,         // default: 0.35
    pub integration: f64, // default: 0.20
}

pub struct OracleConfig {
    pub enabled: bool,              // default: true, env: GOLEM_ORACLE_ENABLED (restart required)
    pub residual_buffer_size: u32,  // default: 256
    pub target_coverage: f64,       // default: 0.85
    pub min_correction_samples: u32,// default: 10
    pub novelty_threshold: f64,     // default: 2.0
    pub forgetting_rate: f64,       // default: 0.005
    pub compaction_window: u64,     // default: 604800 (7 days)
    pub attention: OracleAttentionConfig,
    pub gate: OracleGateConfig,
    pub calibration: OracleCalibrationConfig,
}

pub struct MortalityConfig {
    pub immortal: bool,          // default: false, env: BARDO_IMMORTAL
    pub economic: EconomicMortalityConfig,
    pub epistemic: EpistemicMortalityConfig,
    pub stochastic: StochasticMortalityConfig,
    pub demurrage: DemurrageConfig,
    pub phage: PhageConfig,
    pub thanatopsis: ThanatopsisConfig,
}

pub struct ComputeConfig {
    pub mode: DeploymentMode,   // default: Hosted, env: GOLEM_MODE
    pub tier: ComputeTier,      // default: Small, env: GOLEM_COMPUTE_TIER
}
pub enum ComputeTier { Micro, Small, Medium, Large }

// ── golem_core::event ───────────────────────────────────────────────────────
/// Non-blocking broadcast bus; 10,000-event ring buffer for replay on reconnect.
pub struct EventFabric {
    tx: tokio::sync::broadcast::Sender<GolemEvent>,
    buffer: std::sync::Arc<parking_lot::RwLock<std::collections::VecDeque<GolemEvent>>>,
    seq: std::sync::atomic::AtomicU64,
}
impl EventFabric {
    pub fn new(capacity: usize) -> Self; // capacity = 10_000
    pub fn emit(&self, subsystem: Subsystem, tick: u64, payload: EventPayload);
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<GolemEvent>;
    /// Replay events from `after_seq` (inclusive). Returns buffered events.
    pub fn replay_from(&self, after_seq: u64) -> Vec<GolemEvent>;
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct GolemEvent {
    pub seq: u64,
    pub ts: std::time::Instant,
    pub tick: u64,
    pub subsystem: Subsystem,
    pub payload: EventPayload,
}

#[derive(Clone, Debug, serde::Serialize, Hash, Eq, PartialEq)]
pub enum Subsystem {
    Heartbeat, Perception, Daimon, Mortality, Grimoire, Dreams,
    Context, Inference, Tools, Risk, Coordination, Lifecycle,
    Engagement, Session, Creature, System,
}

/// 50+ typed event variants across 16 subsystems.
#[derive(Clone, Debug, serde::Serialize)]
pub enum EventPayload {
    // Heartbeat (2)
    HeartbeatTick { tick: u64, tier: String, pe: f64, threshold: f64 },
    HeartbeatComplete { tick: u64, duration_ms: u64, actions_taken: u32 },
    // Perception (1)
    MarketObservation { regime: String, anomalies: Vec<String>, probe_count: u32 },
    // Daimon (2)
    DaimonAppraisal { pleasure: f64, arousal: f64, dominance: f64, emotion: String, markers_fired: u32 },
    SomaticMarkerFired { situation: String, valence: f64, source: String },
    // Mortality (3)
    VitalityUpdate { economic: f64, epistemic: f64, stochastic: f64, composite: f64, phase: String },
    PhaseTransition { from: String, to: String, cause: String },
    DeathClockAlarm { clock: String, value: f64, threshold: f64 },
    // Grimoire (7)
    InsightPromoted { id: String, category: String, confidence: f64 },
    HeuristicEvolved { id: String, description: String },
    KnowledgeDecayed { count: u32, reason: String },
    WarningActivated { id: String, severity: String },
    ScarRecorded { source_golem: String, warning: String },
    CausalLinkUpdated { from: String, to: String, strength: f64 },
    CuratorCycleComplete { entries_validated: u32, entries_pruned: u32, entries_promoted: u32 },
    // Dreams (7)
    DreamStart { trigger: String },
    DreamPhaseTransition { from: String, to: String },
    DreamReplay { episode_id: String, utility: f64 },
    DreamCounterfactual { hypothesis: String, outcome: String },
    DreamConsolidation { playbook_edits: u32, insights_generated: u32 },
    DreamComplete { cycles_completed: u32 },
    MicroConsolidation { entries_processed: u32, depotentiation_count: u32 },
    // Context (2)
    ContextAssembled { total_tokens: u32, categories: Vec<(String, u32)> },
    ContextPolicySelfTuned { revision: u32, adjustments: Vec<String> },
    // Inference (3)
    InferenceStart { model: String, tier: String, input_tokens: u32 },
    InferenceToken { token: String },
    InferenceComplete { output_tokens: u32, cost: f64, latency_ms: u64 },
    // Tools (3)
    ToolStart { tool: String, category: String },
    ToolProgress { tool: String, step: String, pct: f32 },
    ToolComplete { tool: String, success: bool, duration_ms: u64 },
    // Risk (3)
    PermitCreated { id: String, action: String, value_limit: String },
    PermitStateChange { id: String, from: String, to: String },
    RiskAssessment { layer: String, result: String },
    // Coordination (6)
    CladeSyncComplete { entries_sent: u32, entries_received: u32 },
    BloomUpdated { domains: Vec<String> },
    PheromoneDeposited { layer: String, domain: String, intensity: f64 },
    PheromoneRead { threats: u32, opportunities: u32, wisdom: u32 },
    BloodstainReceived { source_generation: u32, warning: String },
    CausalEdgePublished { from_var: String, to_var: String },
    // Lifecycle (4)
    LifecycleTransition { from: String, to: String },
    DeathInitiated { cause: String },
    SuccessorSpawned { successor_id: String },
    HealthStatus { process: String, ok: bool, message: String },
    // Engagement (2)
    AchievementUnlocked { id: String, description: String },
    MilestoneReached { name: String, value: f64 },
    // Session (2)
    UserMessage { content: String, session_id: String },
    GolemResponseChunk { content: String, is_final: bool },
    // Creature (3)
    CreatureFormEvolved { from_form: u8, to_form: u8 },
    ExpressionUpdated { expression: String },
    ParticleEffectTriggered { effect_type: String },
    // System (2)
    ShutdownInitiated { phase: String },
    ResourceWarning { resource: String, utilization: f64 },
}

// ── golem_core::cortical ────────────────────────────────────────────────────
/// Zero-latency shared perception surface.
///
/// 32 atomic signals, #[repr(C, align(64))], ~192 bytes actual (256 conservative bound).
/// Fits in 4 cache lines. Each signal group has exactly one writer.
///
/// AFFECT (writer: Daimon):      pleasure f32, arousal f32, dominance f32, primary_emotion u8
/// PREDICTION (writer: Oracle):  aggregate_accuracy f32, accuracy_trend i8,
///                               category_accuracies [f32;16], surprise_rate f32,
///                               pending_predictions u32
/// ATTENTION (writer: Forager):  universe_size u32, active_count u16, watched_count u16
/// ENVIRONMENT (writer: probes): regime u8 {calm=0,trending=1,volatile=2,crisis=3},
///                               regime_confidence f32, gas_gwei f32
/// MORTALITY (writer: mortality): economic_vitality f32, epistemic_vitality f32,
///                               stochastic_vitality f32, behavioral_phase u8
/// INFERENCE (writer: router):   inference_budget_remaining f32, current_tier u8
/// CREATIVE (writer: dreams):    creative_mode u8, fragments_captured u32,
///                               last_novel_prediction_tick u64 (split across two AtomicU32)
/// DERIVED (writer: runtime):    compounding_momentum f32
///
/// Write ordering: Ordering::Release. Read ordering: Ordering::Acquire.
/// f32 values stored as bit-reinterpreted u32 via f32::to_bits() / f32::from_bits().
#[repr(C, align(64))]
pub struct CorticalState {
    // AFFECT
    pub(crate) pleasure: AtomicU32,
    pub(crate) arousal: AtomicU32,
    pub(crate) dominance: AtomicU32,
    pub(crate) primary_emotion: AtomicU8,
    // PREDICTION
    pub(crate) aggregate_accuracy: AtomicU32,
    pub(crate) accuracy_trend: AtomicI8,
    pub(crate) category_accuracies: [AtomicU32; 16],
    pub(crate) surprise_rate: AtomicU32,
    // ATTENTION
    pub(crate) universe_size: AtomicU32,
    pub(crate) active_count: AtomicU16,
    pub(crate) watched_count: AtomicU16,
    pub(crate) pending_predictions: AtomicU32,
    // ENVIRONMENT
    pub(crate) regime: AtomicU8,
    pub(crate) regime_confidence: AtomicU32,
    pub(crate) gas_gwei: AtomicU32,
    // MORTALITY
    pub(crate) economic_vitality: AtomicU32,
    pub(crate) epistemic_vitality: AtomicU32,
    pub(crate) stochastic_vitality: AtomicU32,
    pub(crate) behavioral_phase: AtomicU8,
    // INFERENCE
    pub(crate) inference_budget_remaining: AtomicU32,
    pub(crate) current_tier: AtomicU8,
    // CREATIVE
    pub(crate) creative_mode: AtomicU8,
    pub(crate) fragments_captured: AtomicU32,
    pub(crate) last_novel_prediction_tick: AtomicU32,    // lower 32 bits
    pub(crate) last_novel_prediction_tick_hi: AtomicU32, // upper 32 bits
    // DERIVED
    pub(crate) compounding_momentum: AtomicU32,
}
impl CorticalState {
    pub fn new() -> std::sync::Arc<Self>;
    pub fn pad(&self) -> PadVector;
    pub fn prediction_accuracy(&self) -> f32;
    pub fn phase(&self) -> BehavioralPhase;
    pub fn snapshot(&self) -> CorticalSnapshot;
    // Writer helpers — one per signal group, called only by the owning subsystem
    pub fn write_affect(&self, pleasure: f32, arousal: f32, dominance: f32, emotion: u8);
    pub fn write_prediction(&self, accuracy: f32, trend: i8, categories: &[f32; 16], surprise: f32, pending: u32);
    pub fn write_attention(&self, universe: u32, active: u16, watched: u16, pending: u32);
    pub fn write_environment(&self, regime: u8, confidence: f32, gas_gwei: f32);
    pub fn write_mortality(&self, economic: f32, epistemic: f32, stochastic: f32, phase: u8);
    pub fn write_inference(&self, budget_remaining: f32, tier: u8);
    pub fn write_creative(&self, mode: u8, fragments: u32, last_novel_tick: u64);
    pub fn write_derived(&self, momentum: f32);
}

#[derive(Clone, Debug)]
pub struct CorticalSnapshot {
    pub pleasure: f32, pub arousal: f32, pub dominance: f32, pub primary_emotion: u8,
    pub aggregate_accuracy: f32, pub accuracy_trend: i8, pub surprise_rate: f32,
    pub pending_predictions: u32,
    pub universe_size: u32, pub active_count: u16, pub watched_count: u16,
    pub regime: u8, pub regime_confidence: f32, pub gas_gwei: f32,
    pub economic_vitality: f32, pub epistemic_vitality: f32, pub stochastic_vitality: f32,
    pub behavioral_phase: u8,
    pub inference_budget_remaining: f32, pub current_tier: u8,
    pub creative_mode: u8, pub fragments_captured: u32, pub last_novel_prediction_tick: u64,
    pub compounding_momentum: f32,
}

#[derive(Clone, Debug)]
pub struct PadVector {
    pub pleasure: f64,   // [-1.0, 1.0]
    pub arousal: f64,    // [-1.0, 1.0]
    pub dominance: f64,  // [-1.0, 1.0]
}
impl PadVector {
    pub const ZERO: Self;
    pub fn clamp(&self, min: f64, max: f64) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BehavioralPhase {
    Thriving = 0,     // vitality > 0.7
    Stable = 1,       // 0.5–0.7
    Conservation = 2, // 0.3–0.5
    Declining = 3,    // 0.1–0.3
    Terminal = 4,     // < 0.1
}
impl BehavioralPhase {
    pub fn from_u8(v: u8) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PlutchikEmotion {
    Joy = 0, Trust = 1, Fear = 2, Surprise = 3,
    Sadness = 4, Disgust = 5, Anger = 6, Anticipation = 7,
}
impl PlutchikEmotion {
    pub fn from_pad(pad: &PadVector) -> Self;
}

// ── golem_core::extension ───────────────────────────────────────────────────
/// The 20-hook extension trait. All hooks have default no-op implementations.
/// Extensions implement only the hooks they need.
///
/// Firing order within after_turn (sequential, non-negotiable):
///   heartbeat → lifespan → daimon → memory → risk →
///   dream → cybernetics → clade → telemetry
#[async_trait::async_trait]
pub trait Extension: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn layer(&self) -> u8;                      // 0 = foundation, 7 = recovery
    fn depends_on(&self) -> &[&str] { &[] }

    // Category 1: Session lifecycle (hook 1)
    async fn on_session(&self, _reason: SessionReason, _ctx: &mut SessionCtx) -> anyhow::Result<()> { Ok(()) }
    // Category 2: Input processing (hook 2)
    async fn on_input(&self, _msg: &mut InputMessage, _ctx: &InputCtx) -> anyhow::Result<InputAction> { Ok(InputAction::Pass) }
    // Category 3: Agent lifecycle (hooks 3–4)
    async fn on_before_agent_start(&self, _ctx: &mut AgentStartCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_agent_start(&self, _ctx: &AgentStartCtx) -> anyhow::Result<()> { Ok(()) }
    // Category 4: Turn lifecycle (hooks 5–11)
    async fn on_turn_start(&self, _ctx: &TurnStartCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_context(&self, _messages: &mut Vec<AgentMessage>, _ctx: &ContextCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_before_provider_request(&self, _ctx: &mut ProviderReqCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_call(&self, _call: &ToolCall, _ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction> { Ok(ToolAction::Allow) }
    async fn on_tool_execution_start(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_execution_update(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_execution_end(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_tool_result(&self, _result: &mut ToolResult, _ctx: &ToolResultCtx) -> anyhow::Result<()> { Ok(()) }
    // Category 5: Post-turn learning (hooks 12–14)
    async fn on_turn_end(&self, _ctx: &TurnEndCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_agent_end(&self, _ctx: &AgentEndCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_after_turn(&self, _ctx: &mut AfterTurnCtx) -> anyhow::Result<()> { Ok(()) }
    // Category 6: System hooks (hooks 15–20)
    async fn on_system_prompt(&self, _prompt: &mut String, _ctx: &PromptCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_steer(&self, _msg: &SteerMessage, _ctx: &mut SteerCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_send_message(&self, _msg: &OutboundMessage, _ctx: &MsgCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_debug(&self, _ctx: &DebugCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_error(&self, _err: &GolemError, _ctx: &ErrorCtx) -> anyhow::Result<()> { Ok(()) }
    async fn on_end(&self, _ctx: &EndCtx) -> anyhow::Result<()> { Ok(()) }
}

pub struct ExtensionRegistry {
    extensions: Vec<std::sync::Arc<dyn Extension>>,
    firing_orders: std::collections::HashMap<HookId, Vec<usize>>,
}
impl ExtensionRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, ext: std::sync::Arc<dyn Extension>);
    /// Validates dependency graph and pre-computes per-hook firing orders.
    /// Panics at startup if graph is invalid (fail-fast, not runtime).
    pub fn build(&mut self);
    pub async fn fire_after_turn(&self, ctx: &mut AfterTurnCtx) -> anyhow::Result<()>;
    pub async fn fire_tool_call(&self, call: &ToolCall, ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction>;
    pub async fn fire_session(&self, reason: SessionReason, ctx: &mut SessionCtx) -> anyhow::Result<()>;
    pub async fn fire_end(&self, ctx: &EndCtx) -> anyhow::Result<()>;
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum HookId {
    Session, Input, BeforeAgentStart, AgentStart,
    TurnStart, Context, BeforeProviderRequest,
    ToolCall, ToolExecutionStart, ToolExecutionUpdate, ToolExecutionEnd, ToolResult,
    TurnEnd, AgentEnd, AfterTurn,
    SystemPrompt, Steer, SendMessage, Debug, Error, End,
}

// Hook context types (opaque stubs in golem-core; fleshed out in golem-runtime)
pub struct SessionCtx;
pub struct InputCtx;
pub struct AgentStartCtx;
pub struct TurnStartCtx;
pub struct ContextCtx;
pub struct ProviderReqCtx;
pub struct ToolCallCtx;
pub struct ToolExecCtx;
pub struct ToolResultCtx;
pub struct TurnEndCtx;
pub struct AgentEndCtx;
pub struct AfterTurnCtx;
pub struct PromptCtx;
pub struct SteerCtx;
pub struct MsgCtx;
pub struct DebugCtx;
pub struct ErrorCtx;
pub struct EndCtx;

// Hook action types
pub struct InputMessage { pub content: String, pub source: String }
pub enum InputAction { Pass, Transform(String), Suppress }
pub enum ToolAction { Allow, Block(String), Modify(ToolCall) }
pub struct ToolCall { pub name: String, pub arguments: serde_json::Value }
pub struct ToolResult { pub content: String, pub is_error: bool }
pub struct AgentMessage { pub role: String, pub content: String }
pub struct SteerMessage { pub content: String, pub priority: u8 }
pub struct OutboundMessage { pub content: String, pub surface: String }

#[derive(Clone, Debug)]
pub enum SessionReason { Start, Resume, BeforeCompact, BeforeBranch }

// ── golem_core::taint ───────────────────────────────────────────────────────
#[derive(Clone, Debug, PartialEq, Eq)]
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

// ── golem_core::error ───────────────────────────────────────────────────────
#[derive(thiserror::Error, Debug)]
pub enum GolemError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("initialization error: {0}")]
    Init(String),
    #[error("extension error in '{extension}': {source}")]
    Extension { extension: String, source: anyhow::Error },
    #[error("event fabric error: {0}")]
    EventFabric(String),
    #[error("cortical state error: {0}")]
    CorticalState(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, GolemError>;

// ── golem_core::hdc ─────────────────────────────────────────────────────────
/// 10240-bit binary sparse distributed representation (BSC hypervector).
/// Full TA encoding implemented in Plan 75a (golem-ta crate).
pub struct HdcVector {
    bits: [u64; 160], // 10240 bits = 160 × 64-bit words
}
impl HdcVector {
    pub fn zeros() -> Self;
    pub fn random() -> Self;
    /// Bind: XOR (associative, commutative)
    pub fn bind(&self, other: &Self) -> Self;
    /// Bundle: majority vote across vectors (requires odd count or tie-break)
    pub fn bundle(vectors: &[&Self]) -> Self;
    /// Permute: rotate bits left by `n` positions
    pub fn permute(&self, n: usize) -> Self;
    /// Hamming similarity ∈ [0.0, 1.0]
    pub fn similarity(&self, other: &Self) -> f32;
}

// ── golem_core::alloc ───────────────────────────────────────────────────────
/// Per-tick arena allocator. Reset via O(1) pointer bump after each tick.
/// Backed by bumpalo. Eliminates per-tick heap fragmentation.
pub struct TickArena {
    inner: bumpalo::Bump,
}
impl TickArena {
    pub fn new() -> Self;
    /// O(1) deallocation of all temporaries allocated in this tick.
    pub fn reset(&mut self);
    pub fn alloc<T>(&self, val: T) -> &T;
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &[T];
}

// ── golem_core::cognitive ────────────────────────────────────────────────────
/// Cognitive tier — gates LLM invocation cost per tick.
/// T0 (~80% of ticks): $0.00, <10ms, PLAYBOOK heuristics only.
/// T1: Moderate surprise, Haiku-class LLM, $0.001–$0.005, top-5 retrieved + positions.
/// T2: High surprise / mortality pressure, full LLM, $0.03–$0.10, full cognitive workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum CognitiveTier {
    T0 = 0, // Suppress: ~80% of ticks, $0.00 inference, <10ms
    T1 = 1, // Analyze: Haiku-class, $0.001–$0.005
    T2 = 2, // Deliberate: full LLM, $0.03–$0.10
}
```

---

## Cargo Dependencies

`golem-core` is Layer 0 — zero workspace-internal dependencies. External crates from the workspace `[dependencies]` table:

```toml
[dependencies]
tokio        = { workspace = true, features = ["sync", "rt"] }
serde        = { workspace = true, features = ["derive"] }
serde_json   = { workspace = true }
toml         = { workspace = true }
uuid         = { workspace = true, features = ["v4", "serde"] }
thiserror    = { workspace = true }
tracing      = { workspace = true }
parking_lot  = { workspace = true }
async-trait  = { workspace = true }
anyhow       = { workspace = true }
bumpalo      = { workspace = true }

[dev-dependencies]
tokio        = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

If `bumpalo` is not already in the workspace `[workspace.dependencies]` from Plan 01, add it:
```toml
# In root Cargo.toml [workspace.dependencies]
bumpalo = "3"
```

---

## Source Files

- `prd2/01-golem/00-overview.md` — GolemId semantics, CorticalState overview, crate layer map, cognitive tier table, key term definitions
- `prd2/01-golem/13-runtime-extensions.md` — redirect to 13a/13b
- `prd2/01-golem/13a-runtime-extensions.md` — **Extension trait (20 hooks), ExtensionRegistry, GolemState layout, 28 extensions across 7 layers, topological firing order, hook context types, intervention primitives**
- `prd2/01-golem/13b-runtime-extensions.md` — **EventFabric (full struct + EventPayload enum with 50+ variants, Subsystem enum), type-state lifecycle machine phantom types, arena allocator, shutdown**
- `prd2/01-golem/18-cortical-state.md` — **CorticalState exact field layout with atomic types, size verification (~192 bytes actual), read/write helpers, Ordering semantics, initialization values, serialization rules, ALMA three-layer model, PadVector, PlutchikEmotion, BehavioralPhase**
- `prd2/01-golem/19-config-and-operator-model.md` — **GolemConfig full TOML schema, four config files, hot-reload scope, operator power hierarchy**
- `prd2/shared/config-reference.md` — **Canonical schema for all [section] blocks, all field names/types/defaults/env vars**
- `prd2/shared/glossary.md` — Term definitions for Golem, CorticalState, EventFabric, Extension, PAD Vector, TaintLabel

---

## Implementation Details

### Quick Reference: CorticalState Signal Map

```
32 atomic f32/u8/u32 signals, #[repr(C, align(64))], ~256 bytes (4 cache lines)
Writers are specific subsystems; all others read atomically.

AFFECT (writer: Daimon):        pleasure f32, arousal f32, dominance f32, primary_emotion u8
PREDICTION (writer: Oracle):    aggregate_accuracy f32, accuracy_trend i8,
                                category_accuracies [f32;16], surprise_rate f32,
                                pending_predictions u32
ATTENTION (writer: Forager):    universe_size u32, active_count u16, watched_count u16
ENVIRONMENT (writer: probes):   regime u8 {calm=0,trending=1,volatile=2,crisis=3},
                                regime_confidence f32, gas_gwei f32
MORTALITY (writer: mortality):  economic_vitality f32, epistemic_vitality f32,
                                stochastic_vitality f32, behavioral_phase u8
INFERENCE (writer: router):     inference_budget_remaining f32, current_tier u8
CREATIVE (writer: dreams):      creative_mode u8, fragments_captured u32,
                                last_novel_prediction_tick u64 (two AtomicU32)
DERIVED (writer: runtime):      compounding_momentum f32
```

### Quick Reference: Cognitive Tiers

```
T0 (Suppress):   ~80% of ticks, $0.00 inference, <10ms. PLAYBOOK heuristics only.
T1 (Analyze):    Moderate surprise, Haiku-class LLM, $0.001-$0.005. Top-5 retrieved + positions.
T2 (Deliberate): High surprise/mortality pressure, full LLM, $0.03-$0.10. Full cognitive workspace.
```

### Quick Reference: Extension Hook Order

```
20 hooks across 6 categories:
1.  on_session          — session start/resume/compact/branch
2.  on_input            — input arrives, can pass/transform/suppress
3.  on_before_agent_start — context assembly, model selection
4.  on_agent_start      — after agent init, before first response
5.  on_turn_start       — before each LLM request-response
6.  on_context          — modify messages array
7.  on_before_provider_request — last chance to modify LLM request
8.  on_tool_call        — safety checkpoint; Allow/Block/Modify
9.  on_tool_execution_start
10. on_tool_execution_update
11. on_tool_execution_end
12. on_tool_result      — transform/redact result before LLM sees it
13. on_turn_end
14. on_agent_end
15. on_after_turn       — 9-subsystem sequential pipeline: heartbeat→lifespan→
                          daimon→memory→risk→dream→cybernetics→clade→telemetry
16. on_system_prompt    — modify system prompt
17. on_steer            — mid-execution interrupt
18. on_send_message     — outbound message to surface
19. on_debug            — debug info requested
20. on_error            — error recovery
21. on_end              — shutdown flush (this is hook 20+1; on_end IS the 20th hook; on_debug/on_error are 19/20 — count from prd2: 20 total including on_end)
```

Note: prd2 counts 20 hooks total. The 20th is `on_end`. `on_debug` and `on_error` are hooks 19 and 20 in the System Hooks category, with `on_end` as the final shutdown hook.

---

### Unit 1: Foundation Types & GolemId (`src/id.rs`, `src/error.rs`, `src/taint.rs`, `src/cognitive.rs`)

**Files to create:**
- `crates/golem-core/src/id.rs`
- `crates/golem-core/src/error.rs`
- `crates/golem-core/src/taint.rs`
- `crates/golem-core/src/cognitive.rs`

**Implementation notes:**

`GolemId` wraps `uuid::Uuid` as a newtype. Derive `serde::Serialize/Deserialize` with transparent serialization (serializes as the UUID string). Implement `Display` as the UUID hyphenated form. Add `From<uuid::Uuid>` and `From<GolemId> for uuid::Uuid`.

`GolemError` uses `thiserror`. The `Extension` variant carries the extension name as a string plus the wrapped `anyhow::Error` to preserve full context without coupling to any downstream error type.

`TaintLabel` derives `Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize`. `TaintedString` is `pub` with a `value: String` and `label: TaintLabel`. Do NOT implement `Deref` — callers must explicitly access `.value` to acknowledge the taint propagation cost.

`CognitiveTier` is `#[repr(u8)]` to allow cheap transmission in CorticalState's `current_tier: AtomicU8`. Implement `TryFrom<u8>` returning `GolemError` for values > 2.

---

### Unit 2: GolemConfig (`src/config.rs`)

**File to create:** `crates/golem-core/src/config.rs`

**Implementation notes:**

All config structs derive `serde::Deserialize, serde::Serialize, Debug, Clone`. Use `#[serde(default)]` on the struct-level for all optional sections (styx, succession, etc.) so minimal configs parse correctly.

`GolemConfig::from_file` reads the TOML file, deserializes via `toml::from_str`, then calls `with_env_overrides`. `with_env_overrides` reads `GOLEM_*` and `BARDO_*` env vars and overrides matching fields. ENV var resolution uses a priority table matching `prd2/shared/config-reference.md` Full Env Var Table.

`GolemConfig::from_str` is the same but skips file I/O — used in tests and for runtime config parsing.

Secrets (API keys, private keys) are NOT fields on config structs. They are loaded separately at runtime from env vars. Config structs only hold structural configuration, never secrets.

`GolemSection.name` defaults to `format!("golem-{}", nanoid(6))`. For Plan 02, use a `fn default_golem_name() -> String { format!("golem-{}", uuid::Uuid::new_v4().to_string()[..6].to_owned()) }` as the serde default.

`HeartbeatConfig.base_interval_seconds` default is 15 (from config-reference.md table, `GOLEM_TICK_INTERVAL`). Note: the annotated golem.toml example in 19-config-and-operator-model.md shows `interval_secs = 15` but the canonical config-reference.md uses `base_interval_seconds`. Use `base_interval_seconds` (canonical schema wins).

`InferenceConfig.providers` is a `Vec<InferenceProvider>` because golem.toml uses `[[inference.providers]]` array-of-tables syntax.

`MortalityConfig.immortal = false` by default; only valid for self-hosted mode. The `immortal` field is honored but downstream plan (golem-mortality) enforces the self-hosted restriction.

---

### Unit 3: CorticalState (`src/cortical.rs`)

**File to create:** `crates/golem-core/src/cortical.rs`

**Implementation notes:**

```rust
use std::sync::atomic::{AtomicU32, AtomicU16, AtomicU8, AtomicI8, Ordering};

#[repr(C, align(64))]
pub struct CorticalState { /* fields per Exports section */ }
```

All `AtomicU32` fields storing `f32` values use `f32::to_bits()` for writes and `f32::from_bits()` for reads. Do NOT use unsafe transmute — `f32::to_bits()` and `f32::from_bits()` are stable, safe, and semantically correct.

Write helpers use `Ordering::Release`. Read helpers use `Ordering::Acquire`. This ensures the happens-before relationship within a signal group: when a reader observes a new `pleasure` value, the writer's preceding `arousal` write is also visible.

The `category_accuracies: [AtomicU32; 16]` cannot be initialized with `Default::default()` because `AtomicU32` is not `Copy`. Initialize via:
```rust
// Safe initialization of AtomicU32 array
use std::array;
category_accuracies: array::from_fn(|_| AtomicU32::new(0)),
```

`CorticalState::new()` returns `Arc<Self>` because the state is shared across the GolemState (heap-allocated), the TUI fiber, and any background monitoring threads.

Add a compile-time size assertion:
```rust
const _: () = assert!(std::mem::size_of::<CorticalState>() <= 256);
```

`CorticalSnapshot` derives `Clone, Debug, serde::Serialize` and is produced by reading all 32 signals sequentially. Document that it may span two ticks (not transactionally consistent).

For `PadVector`, initialize constants: `pub const ZERO: Self = Self { pleasure: 0.0, arousal: 0.0, dominance: 0.0 };`

`PlutchikEmotion::from_pad` classifies the PAD octant using the Russell-Mehrabian model:
- Pleasure+, Arousal+, Dominance+ → Joy (0)
- Pleasure+, Arousal-, Dominance+ → Trust (1)
- Pleasure-, Arousal+, Dominance- → Fear (2)
- Pleasure-, Arousal+, Dominance+ → Anger (6)
- Pleasure-, Arousal-, Dominance- → Sadness (4)
- Pleasure+, Arousal+, Dominance- → Surprise (3)
- Pleasure-, Arousal-, Dominance+ → Disgust (5)
- Neutral/Anticipation (default) → Anticipation (7)

Default initialization of CorticalState: all signals at 0.0/0. `primary_emotion` initializes to `PlutchikEmotion::Anticipation as u8 = 7` (the "waiting to observe" state per prd2).

---

### Unit 4: EventFabric & Event Types (`src/event.rs`)

**File to create:** `crates/golem-core/src/event.rs`

**Implementation notes:**

`EventFabric` uses `tokio::sync::broadcast::channel(10_000)` for the live bus and a `parking_lot::RwLock<VecDeque<GolemEvent>>` capped at 10,000 events for replay.

`emit` method:
1. Fetch-and-increment `seq` with `Ordering::Relaxed` (each event is self-contained; no cross-field consistency needed for the counter itself)
2. Construct `GolemEvent { seq, ts: Instant::now(), tick, subsystem, payload }`
3. Push to `buffer` (write lock, pop front if len >= 10_000)
4. `let _ = tx.send(event)` — ignore `SendError` (no active receivers is fine, events are not critical)

`emit` must never block. The `let _ = tx.send(...)` satisfies this — `broadcast::Sender::send` is non-blocking.

`subscribe` returns `tx.subscribe()`. New subscribers miss events emitted before the subscription but can call `replay_from` to catch up.

`replay_from` acquires a read lock on `buffer` and returns cloned events with `seq >= after_seq`.

`GolemEvent` derives `Clone` (required for broadcast channel). `serde::Serialize` but NOT `serde::Deserialize` — events flow outward only, never inbound.

`std::time::Instant` is not `serde::Serialize`. Use `#[serde(skip)]` on the `ts` field and serialize the Unix timestamp as a separate `ts_millis: u64` field. Or use `#[serde(with = "instant_serde")]` — implement a small custom serializer that converts to millis since epoch.

The simplest approach: replace `ts: Instant` with `ts_millis: u64` computed from `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64`.

`EventPayload` has 50+ variants. Implement all variants listed in the Exports section. All fields use `f64` or `f32` consistently with the prd2 definitions (some use `f64` — match the prd2 source exactly).

---

### Unit 5: Extension Trait System (`src/extension.rs`)

**File to create:** `crates/golem-core/src/extension.rs`

**Implementation notes:**

The `Extension` trait requires `async_trait::async_trait` macro because stable Rust does not yet support `async fn` in trait definitions without it. Add `async-trait = "0.1"` to workspace deps if not already present.

All 20 hook methods have `async fn` signatures with `anyhow::Result<()>` return types (or appropriate action types for `on_input`, `on_tool_call`). All default implementations return `Ok(())` or the default action.

The trait has three required methods with no defaults: `fn name(&self) -> &str`, `fn layer(&self) -> u8`. `fn depends_on(&self) -> &[&str]` has a default returning `&[]`.

Context types (`SessionCtx`, `AfterTurnCtx`, etc.) are defined as public structs with no fields in `golem-core`. They will gain fields in later plans (`golem-runtime` Plan 14b) when the subsystems that populate them are implemented. Using empty structs now lets the trait compile and lets plans 03+ implement no-op extensions without blocking on `golem-runtime`.

`ExtensionRegistry::build` algorithm:
1. Sort `extensions` by `layer()` (stable sort — preserves registration order within a layer)
2. Build a name→index map
3. For each extension, validate all `depends_on()` entries exist and are in the same or lower layer; panic with a descriptive message on violation
4. For each `HookId` variant, compute firing order by iterating sorted extensions and collecting indices of those that override the hook (in Plan 02, since all contexts are empty structs, just use all extensions in sorted order)

For `fire_tool_call`: the most restrictive action wins. `Block` short-circuits. `Modify` overrides `Allow`. Two `Modify` results: the last one wins (later extensions in the chain can further modify).

`HookId` derives `Hash, Eq, PartialEq, Clone, Debug`. The `firing_orders: HashMap<HookId, Vec<usize>>` is populated during `build()`.

---

### Unit 6: HDC Primitives (`src/hdc.rs`)

**File to create:** `crates/golem-core/src/hdc.rs`

**Implementation notes:**

`HdcVector` stores 10240 bits as `[u64; 160]`. This is a stub for Plan 75a. Implement the four basic operations:

- `bind`: XOR, element-wise: `a.bits[i] ^ b.bits[i]`
- `bundle`: majority vote. For N vectors, count bit positions where >N/2 vectors have a 1. If exactly N/2 (even N), set to 0 by default.
- `permute(n)`: bitwise left rotation by `n` positions across the full 10240-bit array
- `similarity`: Hamming distance normalized: `1.0 - (popcount(a XOR b) as f32 / 10240.0)`

`random()` fills with `rand` or just use `uuid`-seeded bits. Since `rand` may not be a workspace dep yet, use a simple XorShift PRNG seeded from `uuid::Uuid::new_v4().as_u128()`.

All operations are `pub`. These are stubs — correctness over performance. Plan 75a optimizes with SIMD.

---

### Unit 7: Per-Tick Allocator (`src/alloc.rs`)

**File to create:** `crates/golem-core/src/alloc.rs`

**Implementation notes:**

`TickArena` wraps `bumpalo::Bump`. `reset()` calls `self.inner.reset()`. `alloc<T>` delegates to `self.inner.alloc(val)`. `alloc_slice_copy<T: Copy>` delegates to `self.inner.alloc_slice_copy(slice)`.

This is a thin wrapper. The value is in making `TickArena` a named type that the heartbeat pipeline passes explicitly — it documents the allocation contract and makes it testable.

---

### Unit 8: lib.rs

**File to modify:** `crates/golem-core/src/lib.rs`

Re-export all public items:

```rust
pub mod alloc;
pub mod cognitive;
pub mod config;
pub mod cortical;
pub mod error;
pub mod event;
pub mod extension;
pub mod hdc;
pub mod id;
pub mod taint;

pub use alloc::TickArena;
pub use cognitive::CognitiveTier;
pub use config::GolemConfig;
pub use cortical::{BehavioralPhase, CorticalSnapshot, CorticalState, PadVector, PlutchikEmotion};
pub use error::{GolemError, Result};
pub use event::{EventFabric, EventPayload, GolemEvent, Subsystem};
pub use extension::{Extension, ExtensionRegistry, HookId};
pub use hdc::HdcVector;
pub use id::GolemId;
pub use taint::{TaintLabel, TaintedString};
```

---

## Failure Recovery

**`cargo check` fails with "feature not found for tokio"**
The workspace `Cargo.toml` must declare `tokio` with at minimum `features = ["sync"]`. `golem-core/Cargo.toml` adds `features = ["sync", "rt"]` locally. If the workspace dep declares `features = []`, the crate-level `features` array extends it.

**`AtomicI8` not found**
`AtomicI8` is in `std::sync::atomic` on all Tier-1 Rust targets. Import via `use std::sync::atomic::{AtomicI8, ...}`.

**`[AtomicU32; 16]` initialization fails**
`AtomicU32` does not implement `Copy` or `Default`. Use `std::array::from_fn(|_| AtomicU32::new(0))` (stable since Rust 1.63).

**`async_trait` not in workspace deps**
Add to `[workspace.dependencies]` in root `Cargo.toml`: `async-trait = "0.1"`. Then in `golem-core/Cargo.toml`: `async-trait = { workspace = true }`.

**`bumpalo` not in workspace deps**
Same pattern. `bumpalo = "3"` in workspace deps.

**`serde::Serialize` on `GolemEvent` fails due to `std::time::Instant`**
Replace `ts: Instant` with `ts_millis: u64`. `Instant` is not serializable because it is platform-specific. Use `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64`.

**Config `#[serde(default)]` for nested structs**
All config sub-structs must implement `Default` (or use `#[serde(default = "fn_name")]`) for optional sections. Implement `Default` for all config structs with their documented defaults.

---

## Testing Checkpoint

```bash
# From workspace root:
cargo check -p golem-core
cargo test -p golem-core -- --nocapture
```

Expected passing tests (write these in `crates/golem-core/src/cortical.rs` and `src/event.rs` and `src/extension.rs`):

```
test cortical_state_alignment ... ok
test cortical_state_size ... ok
test cortical_state_write_read_affect ... ok
test pad_vector_plutchik_joy ... ok
test behavioral_phase_from_u8 ... ok
test event_fabric_broadcast ... ok
test event_fabric_replay ... ok
test event_fabric_ring_buffer_cap ... ok
test extension_registry_topological_order ... ok
test extension_registry_missing_dep_panics ... ok
test golem_id_roundtrip ... ok
test tainted_string_is_tainted ... ok
test cognitive_tier_try_from ... ok
test hdc_bind_involution ... ok
test hdc_similarity_self ... ok
test tick_arena_reset ... ok
test config_from_str_minimal ... ok
test config_env_override ... ok
```

Key test implementations:

```rust
#[test]
fn cortical_state_alignment() {
    assert_eq!(std::mem::align_of::<CorticalState>(), 64);
}

#[test]
fn cortical_state_size() {
    assert!(std::mem::size_of::<CorticalState>() <= 256,
        "CorticalState is {} bytes, expected ≤256", std::mem::size_of::<CorticalState>());
}

#[test]
fn cortical_state_write_read_affect() {
    let cs = CorticalState::new();
    cs.write_affect(0.5, -0.3, 0.1, 7);
    let snap = cs.snapshot();
    assert!((snap.pleasure - 0.5).abs() < 1e-6);
    assert!((snap.arousal - (-0.3)).abs() < 1e-6);
    assert_eq!(snap.primary_emotion, 7);
}

#[tokio::test]
async fn event_fabric_broadcast() {
    let fabric = EventFabric::new(100);
    let mut rx = fabric.subscribe();
    fabric.emit(Subsystem::Heartbeat, 1, EventPayload::HeartbeatTick {
        tick: 1, tier: "T0".into(), pe: 0.1, threshold: 0.3,
    });
    let event = rx.recv().await.unwrap();
    assert_eq!(event.tick, 1);
    assert_eq!(event.seq, 0);
}

#[test]
fn hdc_bind_involution() {
    // XOR is its own inverse: bind(bind(a, b), b) == a
    let a = HdcVector::random();
    let b = HdcVector::random();
    let bound = a.bind(&b);
    let recovered = bound.bind(&b);
    assert!((recovered.similarity(&a) - 1.0).abs() < 1e-6);
}

#[test]
fn extension_registry_missing_dep_panics() {
    use std::panic;
    let result = panic::catch_unwind(|| {
        let mut reg = ExtensionRegistry::new();
        reg.register(Arc::new(BadExt)); // BadExt declares depends_on = ["nonexistent"]
        reg.build(); // should panic
    });
    assert!(result.is_err());
}
```

---

## Completion Report

*(Codex fills this in after implementation)*

```
Plan 02 completed: <date>
Files created:
  crates/golem-core/src/lib.rs (modified)
  crates/golem-core/src/id.rs
  crates/golem-core/src/error.rs
  crates/golem-core/src/taint.rs
  crates/golem-core/src/cognitive.rs
  crates/golem-core/src/config.rs
  crates/golem-core/src/cortical.rs
  crates/golem-core/src/event.rs
  crates/golem-core/src/extension.rs
  crates/golem-core/src/hdc.rs
  crates/golem-core/src/alloc.rs
Cargo.toml modified: added async-trait, bumpalo to [workspace.dependencies]
cargo check -p golem-core: PASS
cargo test -p golem-core: NN passed, 0 failed
Deviations: <any>
```

## Verification

### Invariants

<!-- INV-001: Gompertz-Makeham Hazard Monotonicity -->
- **type**: monotonic
- **module**: `golem_mortality::stochastic`
- **property**: Hazard rate h(t) monotonically increases with age (tick number) when epistemic_fitness held constant
- **formula**: h(t) = lambda + alpha * exp(beta * t) * epsilon(t)
- **constraint**: ∂h/∂t ≥ 0 for all t ≥ 0, holding epsilon(t) constant; exponential term dominates
- **test_fn**: `test_hazard_monotonic_with_age`
- **strategy**: proptest
- **inputs**: {"tick": "0..200000", "epistemic_fitness": "constant(0.5)", "lambda": "1e-6", "alpha": "1e-8", "beta": "5e-5"}
- **oracle**: see formula; compute h(t1) < h(t2) for t1 < t2
- **severity**: spec
- **source**: `prd2/02-mortality/03-stochastic-mortality.md`, section "The Gompertz-Makeham Hazard Function"

<!-- INV-002: Gompertz Hazard Rate Cap -->
- **type**: numeric_range
- **module**: `golem_mortality::stochastic`
- **property**: Per-tick hazard rate never exceeds max_hazard_rate; bounded between base and cap
- **formula**: h(t) = clamp(lambda + alpha * exp(beta * t) * epsilon(t), lambda, max_hazard_rate)
- **constraint**: lambda ≤ h(t) ≤ max_hazard_rate; default: 1e-6 ≤ h ≤ 0.001
- **test_fn**: `test_hazard_rate_within_bounds`
- **strategy**: unit
- **inputs**: {"tick": "[0, 1000, 100000]", "base_hazard_rate": "1e-6", "age_hazard_coefficient": "1e-8", "aging_rate": "5e-5", "epistemic_hazard_multiplier": "3.0", "max_hazard_rate": "0.001"}
- **oracle**: base_hazard_rate ≤ result ≤ max_hazard_rate
- **severity**: spec
- **source**: `prd2/02-mortality/03-stochastic-mortality.md`, StochasticMortalityConfig defaults

<!-- INV-003: Epistemic Frailty Multiplier Range -->
- **type**: numeric_range
- **module**: `golem_mortality::stochastic`
- **property**: Epistemic frailty multiplier scales linearly with fitness loss; ranges [1.0, epistemic_hazard_multiplier]
- **formula**: epsilon(t) = 1.0 + (epistemic_hazard_multiplier - 1.0) * (1.0 - epistemic_fitness)
- **constraint**: 1.0 ≤ epsilon(t) ≤ epistemic_hazard_multiplier (default 3.0); at fitness=1.0, epsilon=1.0; at fitness=0.0, epsilon=3.0
- **test_fn**: `test_epistemic_frailty_multiplier_bounds`
- **strategy**: unit
- **inputs**: {"epistemic_fitness": "[0.0, 0.5, 1.0]", "epistemic_hazard_multiplier": "3.0"}
- **oracle**: epsilon = 1.0 + 2.0 * (1.0 - fitness)
- **severity**: spec
- **source**: `prd2/02-mortality/03-stochastic-mortality.md`, section "StochasticMortalityState Interface"

<!-- INV-004: Survival Probability Monotonic Decrease -->
- **type**: monotonic
- **module**: `golem_mortality::stochastic`
- **property**: Cumulative survival probability decreases monotonically with each tick; product of (1 - h(t)) terms
- **formula**: S(t) = S(t-1) * (1 - h(t))
- **constraint**: S(t) < S(t-1) for all t ≥ 1; S(0) = 1.0; S(t) → 0 as t → ∞
- **test_fn**: `test_survival_probability_decreasing`
- **strategy**: proptest
- **inputs**: {"tick": "0..1000", "hazard_sequence": "generated from Gompertz"}
- **oracle**: survival[i+1] < survival[i]; survival[0] = 1.0
- **severity**: spec
- **source**: `prd2/02-mortality/03-stochastic-mortality.md`, StochasticMortalityState.survival_probability

<!-- INV-005: Ebbinghaus Epistemic Decay Curve -->
- **type**: convergence
- **module**: `golem_mortality::epistemic`
- **property**: Knowledge decay follows Ebbinghaus forgetting curve; retention = exp(-t / half_life)
- **formula**: confidence(t) = initial_confidence * exp(-ticks_since_validation / (half_life / ln(2)))
- **constraint**: At t=0, confidence=initial; at t=half_life, confidence=initial*0.5; monotonically decreasing
- **test_fn**: `test_ebbinghaus_decay_halflife`
- **strategy**: unit
- **inputs**: {"initial_confidence": "1.0", "half_life_ticks": "250", "decay_ticks": "[0, 250, 500, 750]"}
- **oracle**: decay(0) = 1.0; decay(250) ≈ 0.5; decay(500) ≈ 0.25
- **severity**: spec
- **source**: `prd2/02-mortality/05-knowledge-demurrage.md`, section "S5 -- DemurrageConfig"

<!-- INV-006: Domain Decay Multipliers Variance -->
- **type**: numeric_range
- **module**: `golem_mortality::demurrage`
- **property**: Domain-specific decay multipliers correctly reflect environmental volatility; GasMev fastest, Protocol slowest
- **formula**: effective_decay = base_decay_per_interval * domain_multiplier * type_weight
- **constraint**: domain_multipliers: GasMev=3.0x, PriceDirection=1.5x, Volatility=0.8x, Yield=0.5x, Protocol=0.3x, Governance=0.4x, MarketStructure=0.7x, CrossChain=0.6x, General=1.0x (baseline)
- **test_fn**: `test_domain_multipliers_ordering`
- **strategy**: unit
- **inputs**: {"base_decay": "0.03", "domains": "[GasMev, Protocol, General]"}
- **oracle**: GasMev_decay > PriceDirection_decay > Volatility_decay > ... > Protocol_decay
- **severity**: spec
- **source**: `prd2/02-mortality/05-knowledge-demurrage.md`, section "Differential Decay by Knowledge Type"

<!-- INV-007: Knowledge Type Weight Hierarchy -->
- **type**: numeric_range
- **module**: `golem_mortality::demurrage`
- **property**: Knowledge type weights correctly prioritize durable types; DeathTestament most resistant, Episode least
- **formula**: confidence_decay_rate = base_decay_per_interval * domain_multiplier * type_weight
- **constraint**: DeathTestament=3.0, Question=2.0, CausalLink=1.8, Warning=1.5, Heuristic=1.2, Insight=1.0, StrategyFragment=0.7, Episode=0.5
- **test_fn**: `test_type_weight_ordering`
- **strategy**: unit
- **inputs**: {"base_decay": "0.03", "type_weights": "as specified"}
- **oracle**: DeathTestament_decay_rate < Question_decay_rate < ... < Episode_decay_rate (inverse relationship)
- **severity**: spec
- **source**: `prd2/02-mortality/05-knowledge-demurrage.md`, table "Knowledge Type"

<!-- INV-008: Epistemic Fitness R-squared Bounds -->
- **type**: numeric_range
- **module**: `golem_mortality::epistemic`
- **property**: Epistemic fitness R-squared ranges [0.0, 1.0]; under 10 predictions defaults to 0.5
- **formula**: R² = 1 - (SS_res / SS_tot), where SS_res = Σ(actual - predicted)², SS_tot = Σ(actual - mean)²
- **constraint**: 0.0 ≤ fitness ≤ 1.0; n<10 ⇒ fitness=0.5; n≥100 uses rolling window
- **test_fn**: `test_epistemic_fitness_rsquared_bounds`
- **strategy**: unit
- **inputs**: {"predictions": "hand-computed 3-tuple [predicted, actual, expected_r2]", "window_size": "100"}
- **oracle**: computed_r2 ≤ 1.0; computed_r2 ≥ 0.0; n<10 yields 0.5
- **severity**: spec
- **source**: `prd2/02-mortality/02-epistemic-decay.md`, section "Fitness Metric: R-squared Over Predictions"

<!-- INV-009: Senescence Threshold Hysteresis -->
- **type**: state_machine
- **module**: `golem_mortality::epistemic`
- **property**: Senescence stage transitions follow hysteresis; recovery requires exceeding senescence_threshold + 0.10
- **formula**: entry: fitness < senescence_threshold (default 0.35) for recovery_grace_period ticks; exit: fitness > (senescence_threshold + 0.10) = 0.45
- **constraint**: Stage1 (WARNING) → Stage2 (CONFIRMED) → Stage3 (DEATH_PROTOCOL); recovery only from Stage1; once Stage2, requires fitness > 0.45 to recover
- **test_fn**: `test_senescence_stage_transitions`
- **strategy**: unit
- **inputs**: {"fitness_sequence": "[0.5→0.3→0.2→0.5]", "senescence_threshold": "0.35", "recovery_hysteresis": "0.10", "grace_period": "100_ticks"}
- **oracle**: after 100 ticks at 0.3, Stage1→Stage2; at 0.5, remains Stage2 until fitness exceeds 0.45
- **severity**: spec
- **source**: `prd2/02-mortality/02-epistemic-decay.md`, section "EpistemicFitnessState Struct"

<!-- INV-010: Behavioral Phase Thresholds -->
- **type**: numeric_range
- **module**: `golem_core::cortical`
- **property**: Behavioral phase transitions at correct composite vitality thresholds
- **formula**: phase = f(min(economic_vitality, epistemic_vitality, stochastic_vitality))
- **constraint**: Thriving (>0.7), Stable (0.5–0.7), Conservation (0.3–0.5), Declining (0.1–0.3), Terminal (<0.1)
- **test_fn**: `test_behavioral_phase_thresholds`
- **strategy**: unit
- **inputs**: {"composite_vitality": "[0.0, 0.1, 0.3, 0.5, 0.7, 1.0]", "expected_phase": "[Terminal, Declining, Conservation, Stable, Thriving, Thriving]"}
- **oracle**: phase matches expected_phase for each vitality value
- **severity**: spec
- **source**: `plans/02-core-types.md`, BehavioralPhase enum definition

<!-- INV-011: CorticalState Cache Line Alignment -->
- **type**: capacity
- **module**: `golem_core::cortical`
- **property**: CorticalState struct fits within 256-byte conservative bound (4 cache lines); actual ~192 bytes
- **formula**: struct size = (4 × 8) + (16 × 8) + (4 × 4) + (3 × 4) + (8 × 1) + padding
- **constraint**: #[repr(C, align(64))]; actual_size ≤ 256; zero false sharing across Atomic operations
- **test_fn**: `test_cortical_state_size_and_alignment`
- **strategy**: unit
- **inputs**: {"struct_size": "computed via std::mem::size_of"}
- **oracle**: size_of::<CorticalState>() ≤ 256; align_of::<CorticalState>() == 64
- **severity**: code
- **source**: `plans/02-core-types.md`, CorticalState definition

<!-- INV-012: PAD Vector Bounds -->
- **type**: numeric_range
- **module**: `golem_core::cortical`
- **property**: Pleasure, Arousal, Dominance (PAD) values clamp to [-1.0, 1.0]
- **formula**: pad_component = clamp(value, -1.0, 1.0)
- **constraint**: -1.0 ≤ pleasure ≤ 1.0; -1.0 ≤ arousal ≤ 1.0; -1.0 ≤ dominance ≤ 1.0
- **test_fn**: `test_pad_vector_bounds`
- **strategy**: proptest
- **inputs**: {"raw_pleasure": "-2.0..3.0", "raw_arousal": "-2.0..3.0", "raw_dominance": "-2.0..3.0"}
- **oracle**: clamped values all within [-1.0, 1.0]
- **severity**: code
- **source**: `plans/02-core-types.md`, PadVector struct

<!-- INV-013: Plutchik Emotion Mapping from PAD -->
- **type**: roundtrip
- **module**: `golem_core::cortical`
- **property**: Emotion classification from PAD vector is deterministic and covers all 8 Plutchik emotions
- **formula**: emotion = argmax(affinity_score(pad, emotion)) for emotion in {Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation}
- **constraint**: every PAD vector maps to exactly one emotion; each emotion is reachable from some PAD region
- **test_fn**: `test_plutchik_emotion_classification`
- **strategy**: proptest
- **inputs**: {"pad": "generated across [-1, 1]³ space"}
- **oracle**: each emotion classifiable; classification stable in emotion neighborhoods
- **severity**: code
- **source**: `plans/02-core-types.md`, PlutchikEmotion::from_pad

<!-- INV-014: Regime Tag Correctness -->
- **type**: state_machine
- **module**: `golem_core::config`
- **property**: Market regime (Calm, Trending, Volatile, Crisis) correctly assigned and enforced
- **formula**: regime = classify(price_volatility, volume_change, gas_price_change)
- **constraint**: regime ∈ {Calm=0, Trending=1, Volatile=2, Crisis=3}; monotonic enforcement (no invalid transitions)
- **test_fn**: `test_regime_tag_transitions`
- **strategy**: unit
- **inputs**: {"volatility": "[0.01, 0.05, 0.15, 0.30]", "expected_regime": "[Calm, Calm, Volatile, Crisis]"}
- **oracle**: regime assignments match expected for given market conditions
- **severity**: code
- **source**: `plans/02-core-types.md`, CorticalState regime field

<!-- INV-015: Inheritance Confidence Cap -->
- **type**: numeric_range
- **module**: `golem_core::config`
- **property**: Inherited knowledge confidence capped at 0.7 (protocol invariant); default 0.4
- **formula**: inherited_confidence = min(predecessor_confidence * inheritance_factor, 0.7)
- **constraint**: inheritance_confidence ≤ 0.7 (hard cap); default 0.4; max allowed 0.7
- **test_fn**: `test_inheritance_confidence_cap`
- **strategy**: unit
- **inputs**: {"predecessor_confidence": "[0.5, 0.8, 1.0]", "inheritance_confidence_config": "0.4", "expected_max": "0.7"}
- **oracle**: all inherited entries cap at 0.7 or below config value
- **severity**: spec
- **source**: `plans/02-core-types.md`, SuccessionConfig.inheritance_confidence comment

<!-- INV-016: Regime Multiplier Coverage -->
- **type**: sum_constraint
- **module**: `golem_core::config`
- **property**: All five regime multipliers defined; interpret cost scaling based on market state
- **formula**: adjusted_cost = base_cost * regime_multiplier[regime]
- **constraint**: regimes {TrendingUp, TrendingDown, Volatile, RangeBound, Unknown} all have multipliers; defaults {1.0, 0.5, 0.3, 2.0, 0.8}
- **test_fn**: `test_regime_multipliers_complete`
- **strategy**: unit
- **inputs**: {"regimes": "[TrendingUp, TrendingDown, Volatile, RangeBound, Unknown]", "expected_multipliers": "[1.0, 0.5, 0.3, 2.0, 0.8]"}
- **oracle**: all regimes present; multipliers match defaults
- **severity**: code
- **source**: `plans/02-core-types.md`, RegimeMultipliers struct

<!-- INV-017: Probe Threshold Ordering -->
- **type**: numeric_range
- **module**: `golem_core::config`
- **property**: Probe thresholds have correct ordering: low < high for each dimension
- **formula**: thresholds have pairs (low, high) for price_delta, health_factor, world_model_drift
- **constraint**: price_delta_low_bps (50) < price_delta_high_bps (200); health_factor_low (1.5) > health_factor_high (1.2); drift_low (0.10) < drift_high (0.25)
- **test_fn**: `test_probe_threshold_ordering`
- **strategy**: unit
- **inputs**: {"thresholds": "ProbeThresholds defaults"}
- **oracle**: price_delta_low < price_delta_high; drift_low < drift_high; health factor inverted (lower is higher severity)
- **severity**: code
- **source**: `plans/02-core-types.md`, ProbeThresholds struct

<!-- INV-018: Economic Mortality Vitality EMA -->
- **type**: convergence
- **module**: `golem_mortality::economic`
- **property**: Burn rate EMA converges to true burn rate from cold start within 10 observations
- **formula**: burn_rate_new = burn_rate_old * 0.95 + tick_cost * 0.05 (alpha=0.05)
- **constraint**: after n constant-input ticks, |burn_rate_ema - true_rate| < 0.01 * true_rate for n ≥ 10
- **test_fn**: `test_burn_rate_ema_convergence`
- **strategy**: proptest
- **inputs**: {"constant_tick_cost": "1.0", "iterations": "0..100"}
- **oracle**: EMA reaches within 1% of input by iteration 10
- **severity**: code
- **source**: `prd2/02-mortality/01-architecture.md`, EconomicClock impl

<!-- INV-019: Death Clock Coupling Paths -->
- **type**: event_sequence
- **module**: `golem_mortality`
- **property**: All four coupling paths between mortality clocks are implemented and fire in dependency order
- **formula**: Epistemic→Economic, Economic→Epistemic, Both→Stochastic, Stochastic→Behavioral
- **constraint**: OutcomeVerification records ground epistemic→economic coupling; conservation mode triggers economic→epistemic; hazard includes epistemic frailty; stochastic seed deterministic
- **test_fn**: `test_mortality_coupling_completeness`
- **strategy**: integration
- **inputs**: {"scenario": "epistemically decayed golem with declining trades"}
- **oracle**: OutcomeVerification produced; burn rate increases; hazard rises; behavioral phase drops; stochastic roll deterministic
- **severity**: spec
- **source**: `prd2/02-mortality/01-architecture.md`, section "Clock Interaction and Coupling Dynamics"

<!-- INV-020: Extension Hook Fire Order -->
- **type**: event_sequence
- **module**: `golem_core::extension`
- **property**: Extension hooks fire in exactly this order within after_turn: heartbeat → lifespan → daimon → memory → risk → dream → cybernetics → clade → telemetry
- **formula**: hook_order = [heartbeat, lifespan, daimon, memory, risk, dream, cybernetics, clade, telemetry]
- **constraint**: no reordering; sequential execution; no parallel execution of cross-layer hooks
- **test_fn**: `test_extension_hook_fire_order`
- **strategy**: integration
- **inputs**: {"extensions": "mock extensions with firing timestamps"}
- **oracle**: timestamps monotonically increase in expected hook order
- **severity**: spec
- **source**: `plans/02-core-types.md`, Extension trait comment

<!-- INV-021: DeathCause Exhaustiveness -->
- **type**: state_machine
- **module**: `golem_mortality`
- **property**: Every death path (economic depletion, epistemic senescence, stochastic roll, composite collapse) triggers correct DeathCause variant
- **formula**: enum DeathCause { Economic, Epistemic, Stochastic, Composite }
- **constraint**: each tick's death check produces exactly one cause if death occurs; no multiple causes per death
- **test_fn**: `test_death_cause_exhaustiveness`
- **strategy**: unit
- **inputs**: {"death_scenarios": "[economic=0, epistemic=0.2, stochastic_roll<hazard, composite<0.01]"}
- **oracle**: each scenario triggers expected DeathCause; all four variants reachable
- **severity**: code
- **source**: `prd2/02-mortality/01-architecture.md`, implied by death clock structure

<!-- INV-022: EventFabric Ring Buffer Capacity -->
- **type**: capacity
- **module**: `golem_core::event`
- **property**: EventFabric broadcast ring buffer holds exactly 10,000 events for replay on reconnect
- **formula**: buffer_capacity = 10_000
- **constraint**: buffer.len() ≤ 10_000; on overflow, oldest events evicted FIFO; replay_from(seq) returns all buffered events >= seq
- **test_fn**: `test_event_fabric_ring_buffer_capacity`
- **strategy**: unit
- **inputs**: {"emit_count": "15000", "expected_buffer_size": "10000"}
- **oracle**: final buffer size exactly 10_000; replay returns correct range
- **severity**: spec
- **source**: `plans/02-core-types.md`, EventFabric definition

<!-- INV-023: Subsystem Coverage -->
- **type**: numeric_range
- **module**: `golem_core::event`
- **property**: All 16 subsystems represented in Subsystem enum
- **formula**: enum Subsystem { Heartbeat, Perception, Daimon, Mortality, Grimoire, Dreams, Context, Inference, Tools, Risk, Coordination, Lifecycle, Engagement, Session, Creature, System }
- **constraint**: exactly 16 distinct subsystems; no duplicates; complete coverage of system domains
- **test_fn**: `test_subsystem_enum_completeness`
- **strategy**: unit
- **inputs**: {"subsystems": "all enum variants"}
- **oracle**: count == 16; all variants distinct; no overlapping domains
- **severity**: code
- **source**: `plans/02-core-types.md`, Subsystem enum

<!-- INV-024: EventPayload Variant Count -->
- **type**: numeric_range
- **module**: `golem_core::event`
- **property**: EventPayload contains 50+ typed event variants across 16 subsystems (2–7 per subsystem)
- **formula**: count(EventPayload variants) ≥ 50
- **constraint**: Heartbeat(2), Perception(1), Daimon(2), Mortality(3), Grimoire(7), Dreams(7), Context(2), Inference(3), Tools(3), Risk(3), Coordination(6), Lifecycle(4), Engagement(2), Session(2), Creature(3), System(2) = 50+ total
- **test_fn**: `test_event_payload_variant_count`
- **strategy**: unit
- **inputs**: {"variants": "EventPayload enum arms"}
- **oracle**: count(arms) ≥ 50; distribution per subsystem matches spec
- **severity**: code
- **source**: `plans/02-core-types.md`, EventPayload enum section

---

### Regression Anchors

`test_hazard_monotonic_with_age`
`test_hazard_rate_within_bounds`
`test_epistemic_frailty_multiplier_bounds`
`test_survival_probability_decreasing`
`test_ebbinghaus_decay_halflife`
`test_domain_multipliers_ordering`
`test_type_weight_ordering`
`test_epistemic_fitness_rsquared_bounds`
`test_senescence_stage_transitions`
`test_behavioral_phase_thresholds`
`test_cortical_state_size_and_alignment`
`test_pad_vector_bounds`
`test_plutchik_emotion_classification`
`test_regime_tag_transitions`
`test_inheritance_confidence_cap`
`test_regime_multipliers_complete`
`test_probe_threshold_ordering`
`test_burn_rate_ema_convergence`
`test_mortality_coupling_completeness`
`test_extension_hook_fire_order`
`test_death_cause_exhaustiveness`
`test_event_fabric_ring_buffer_capacity`
`test_subsystem_enum_completeness`
`test_event_payload_variant_count`

---

### Cross-Crate Contracts

| Upstream | Input Condition | Expected Behavior |
|----------|----------------|-------------------|
| golem-core (Config) | `inheritance_confidence` > 0.7 | Clamped to 0.7 by golem-mortality on successor creation |
| golem-core (EventFabric) | `emit()` called; buffer full | Oldest event evicted; seq counter increments monotonically |
| golem-mortality | Tick occurs; epistemic_fitness < senescence_threshold | EpistemicFitnessState enters Stage1; after grace_period ticks, advances to Stage2 |
| golem-mortality | Tick occurs; epistemic_fitness recovers to > (threshold + 0.10) | SenescenceStage can recover from Stage1 only; Stage2+ requires fitness > 0.45 |
| golem-daimon (reads CorticalState) | `write_affect()` called by any subsystem | Atomic store with Ordering::Release; next read via Acquire sees latest PAD |
| golem-heartbeat (reads CorticalState) | Current phase is Terminal | Likely triggers Lifecycle.LifecycleTransition event; death protocol initiates |

---

### Event Sequence Assertions

1. **Economic Death Sequence**:
   - `HeartbeatTick` → `VitalityUpdate { economic: 0.0, ... }` → `DeathClockAlarm { clock: "Economic", ... }` → `DeathInitiated { cause: "Economic" }` → `SuccessorSpawned`

2. **Epistemic Senescence Sequence**:
   - `MarketObservation` (bad regime) → `OutcomeVerification` (low R²) → `VitalityUpdate { epistemic: < threshold }` → `DeathClockAlarm { clock: "Epistemic" }` → `PhaseTransition { to: "Declining" }` → `DeathInitiated { cause: "Epistemic" }`

3. **Stochastic Death Sequence**:
   - `HeartbeatTick` → [random roll < hazard] → `VitalityUpdate { stochastic: 0.0 }` → `DeathInitiated { cause: "Stochastic" }` → `SuccessorSpawned`

4. **Behavioral Phase Cascade**:
   - `VitalityUpdate { composite: 0.8 }` (Thriving) → cost spike → `VitalityUpdate { composite: 0.6 }` (Stable) → market regime shift → `VitalityUpdate { composite: 0.4 }` (Conservation) → inference reduction → `VitalityUpdate { composite: 0.2 }` (Declining) → `PhaseTransition` event fired at each threshold

5. **Extension Hook Execution**:
   - `on_session(Reason::Start)` → `on_heartbeat_start` → `on_lifespan_tick` → `on_daimon_update` → `on_memory_validate` → `on_risk_check` → `on_dream_trigger` → `on_cyber_reflect` → `on_clade_sync` → `on_telemetry_emit`

---

### Academic References Verified

| Reference | Formula/Constant | PRD2 Match | Web-Verified | Notes |
|-----------|-----------------|------------|--------------|-------|
| Gompertz 1825 | h(t) = aq^x (original); modern: α·e^(βx) | ✓ beta=5e-5, alpha=1e-8 | ✓ Confirmed PMC paper | Original paper used notation aq^x; modern form used for computation |
| Ebbinghaus 1885 | retention = exp(-t / half_life) | ✓ Applied to demurrage with domain multipliers | ✓ Cited in prd2/shared/citations.md | Base forgetting curve; DeFi domains have different half-lives (hours to months) |
| Kelly 1956 | f* = (bp - q) / b (optimal fraction) | Referenced but not computed in core types | ✓ prd2/shared/citations.md [KELLY-1956] | Mathematical foundation; used in 08-vault specs, not core-types |
| Kreps-Milgrom-Roberts-Wilson 1982 | Game-theoretic cooperation under uncertainty | ✓ Referenced for stochastic mortality design rationale | ✓ [KREPS-1982] in prd2 | Justifies indefinite hazard vs. known endpoint |
| Makeham (Gompertz-Makeham separation) | h(t) = λ + α·e^(βt) | ✓ lambda=1e-6, alpha=1e-8, beta=5e-5 | ✓ Wikipedia, PMC papers | Age-independent (λ) + age-dependent (α·e^(βt)) components |
| Plutchik 1980 (Emotion Wheel) | 8-emotion classification from arousal/valence | ✓ PlutchikEmotion enum with 8 variants | ✓ [PLUTCHIK-1980] in glossary | Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation |
| PAD (Pleasure-Arousal-Dominance) | 3D emotion space | ✓ PadVector ranges [-1.0, 1.0] per dimension | ✓ Widely used in affective computing | Russell 1980 emotional space model |
| Vela et al. 2022 | 91% of ML models degrade temporally | ✓ [VELA-2022] justification for epistemic death | ✓ Scientific Reports, cited in prd2 | Empirical basis for model staleness |
| Arbesman 2012 | Knowledge half-lives (medical 45yr, IT <2yr) | ✓ Applied to domain_multipliers in demurrage | ✓ [ARBESMAN-2012] in prd2 | Theoretical foundation; DeFi mapped to IT range |

---

### Unverified Edge Cases & Recommended Follow-Ups

1. **Hazard Rate Overflow**: Current formula uses f64; at t > 300,000 ticks (~139 days), exp(beta * t) may approach f64 limits. Recommend adding unit test for hazard_rate_capping at realistic lifespans.

2. **Epistemic Fitness with < 10 Predictions**: Default 0.5 is reasonable but untested. Recommend proptest with n ∈ [0..10) to confirm stabilization.

3. **Atomic Write Ordering Gaps**: CorticalState uses Acquire/Release; verify that all reads in downstream subsystems use Acquire, and all writes use Release. No guarantee in the code that violations will be caught at compile time.

4. **Ring Buffer Wraparound**: EventFabric's 10,000-event buffer will wrap; recommend integration test with >100,000 emitted events to verify no seq counter overflow (atomic u64 has headroom, but wraparound behavior should be documented).

5. **Senescence Grace Period Type**: Currently u64; if grace_period > u64::MAX ticks, no overflow check. Unlikely in practice (u64::MAX ticks ≈ 10^19 seconds ≈ 10^11 years), but document assumption.

6. **Inheritance Confidence Truncation**: Spec says "max 0.7, protocol invariant" but no runtime check in the plan's GolemConfig. Recommend adding validation in `from_file()` and `with_env_overrides()`.

7. **ProbeThresholds Health Factor Inversion**: Spec has low=1.5 > high=1.2, inverting the usual ordering. Recommend naming clarification: `health_factor_warning` vs. `health_factor_danger` instead of low/high.


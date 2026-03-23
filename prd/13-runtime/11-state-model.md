# State model [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document defines the canonical state model for a running Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM): the mutable `GolemState` struct, the read-only `GolemSnapshot` projection, and all component states (identity, heartbeat, vitality, mood, dream, cognition, memory, economic, death). It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the Heartbeat FSM (the recurring decision cycle), the Daimon (affect engine), and the Event Fabric (internal event bus). For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

The Golem's mutable state lives in a single Rust struct (`GolemState`) that flows through the extension chain on every heartbeat tick. A read-only projection (`GolemSnapshot`) is computed at the end of each tick for TUI consumption, REST queries, and WebSocket streaming. All state transitions emit `GolemEvent` variants through the Event Fabric.

> **Crate**: `golem-runtime` (state.rs, lifecycle.rs), `golem-core` (event_fabric.rs)
>
> **Cross-references:**
>
> - `./12-realtime-subscriptions.md` — full GolemEvent enum (50+ variants), subscription topics, and the event catalog emitted by state transitions
> - `../02-mortality/01-architecture.md` — VitalityState: three mortality clocks (economic, epistemic, stochastic), composite Vitality score, and BehavioralPhase transitions
> - `../03-daimon/01-appraisal.md` — MoodState: PAD (Pleasure-Arousal-Dominance) vector computation, Plutchik emotion mapping, and appraisal triggers
> - `../05-dreams/01-architecture.md` — DreamState: dream scheduling, the three-phase NREM/REM/integration cycle, and urgency scoring
> - `../06-hypnagogia/02-architecture.md` — HypnagogicState: the transitional cognitive state between waking and dreaming, producing creative associations
> - `../04-memory/01-grimoire.md` — MemoryState: the Grimoire persistent knowledge base with episodes, insights, heuristics, and the Curator maintenance cycle

---

## Design principles

1. **Single mutable struct.** `GolemState` is the one source of truth. Extensions read and write it during hook execution. No ambient mutation between ticks.
2. **Component ownership.** Each section of `GolemState` is written by a specific extension. The sequential `after_turn` firing order ensures each section is written once per tick, then read by later extensions.
3. **Snapshot projection.** At tick end, `GolemState` projects to `GolemSnapshot` -- a read-only, serializable struct optimized for UI consumption. Snapshots are cheap to compute and safe to send across thread boundaries.
4. **Three visibility tiers.** Every field is tagged `public`, `owner`, or `internal`. WebSocket connections receive only fields matching their auth tier.
5. **Event emission.** Every state transition emits a `GolemEvent` variant through the Event Fabric's `tokio::broadcast` channel. Zero subscribers means zero serialization cost.

---

## GolemState: the mutable world

```rust
/// The mutable state of a living Golem. Passed through the extension chain.
///
/// Organized by ownership: each section is written by a specific extension
/// (or set of extensions) and read by downstream extensions.
pub struct GolemState {
    // ── Identity (static after boot) ──────────────────────────
    pub identity: IdentityState,

    // ── Heartbeat (written by golem-heartbeat) ────────────────
    pub heartbeat: HeartbeatState,

    // ── Mortality (written by golem-lifespan) ─────────────────
    pub vitality: VitalityState,

    // ── Affect (written by golem-daimon) ──────────────────────
    pub mood: MoodState,

    // ── Dreams (written by golem-dream) ───────────────────────
    pub dream: DreamState,

    // ── Hypnagogia (written by golem-hypnagogia) ────────────
    pub hypnagogia: HypnagogicState,

    // ── Cognition (written by golem-context, golem-reflector) ─
    pub cognition: CognitionState,

    // ── Performance (written by golem-verifier) ───────────────
    pub performance: PerformanceState,

    // ── Memory (written by golem-curator, golem-styx) ─────────
    pub memory: MemoryState,

    // ── Clade (written by golem-clade) ────────────────────────
    pub clade: CladeState,

    // ── Economics (written by golem-lifespan) ─────────────────
    pub economic: EconomicState,

    // ── Death (written by golem-lifespan during Thanatopsis) ──
    pub death: DeathState,
}
```

---

## GolemSnapshot: the read-only projection

Returned on initial WebSocket connection and via `GET /api/v1/state`. Computed once per tick, cached until the next tick.

```rust
/// Read-only projection of GolemState for UI consumption.
/// Serialized to JSON for WebSocket and REST delivery.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GolemSnapshot {
    /// Schema version for forward compatibility.
    pub version: u32,

    /// Monotonic sequence number -- incremented on every state change.
    pub seq: u64,

    /// Tick number when this snapshot was computed.
    pub tick: u64,

    /// ISO-8601 timestamp.
    pub timestamp: String,

    /// Component snapshots.
    pub identity: IdentitySnapshot,
    pub heartbeat: HeartbeatSnapshot,
    pub vitality: VitalitySnapshot,
    pub mood: MoodSnapshot,
    pub dream: DreamSnapshot,
    pub hypnagogia: HypnagogicSnapshot,
    pub cognition: CognitionSnapshot,
    pub performance: PerformanceSnapshot,
    pub memory: MemorySnapshot,
    pub clade: CladeSnapshot,
    pub economic: EconomicSnapshot,
    pub death: DeathSnapshot,
}
```

> **Size target:** Serialized `GolemSnapshot` must be <50KB for efficient WebSocket transmission.

---

## Component states

### IdentityState

Written once at boot. Changes only on config reload.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdentityState {
    pub golem_id: String,
    pub name: String,
    pub owner_address: String,        // Hex address
    pub wallet_address: String,       // Hex address
    pub erc8004_id: Option<String>,
    pub lineage_id: String,
    pub generation_number: u32,
    pub strategy_family: String,
    pub disposition: Disposition,
    pub created_at: String,           // ISO-8601
    pub chains: Vec<u64>,

    // Owner-only fields
    pub privy_user_id: Option<String>,
    pub telegram_binding: Option<TelegramBinding>,
    pub discord_binding: Option<DiscordBinding>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Disposition { Eros, Balanced, Thanatos }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelegramBinding {
    pub telegram_user_id: String,
    pub privy_user_id: String,
    pub wallet_address: String,
    pub bound_at: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscordBinding {
    pub discord_user_id: String,
    pub clade_id: String,
    pub channel_id: String,
    pub wallet_address: String,
    pub bound_at: String,
    pub permissions: Vec<String>,
}
```

---

### HeartbeatState

Updated every tick by `golem-heartbeat`.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatState {
    pub phase: FsmPhase,
    pub dream_sub_phase: Option<DreamSubPhase>,
    pub tick_number: u64,
    pub tick_interval_seconds: f64,
    pub tier: InferenceTier,
    pub suppressed: bool,
    pub tick_cost_usdc: f64,
    pub daily_cost_usdc: f64,
    pub daily_cost_remaining_usdc: f64,
    pub cost_cap_state: CostCapState,
    pub last_tick_at: String,           // ISO-8601
    pub last_tick_duration_ms: u64,

    // Owner-only
    pub probes: Option<Vec<ProbeResult>>,
    pub escalation: Option<EscalationSummary>,
    pub cache_hit: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FsmPhase { Idle, Sensing, Deciding, Acting, Reflecting, Sleeping }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DreamSubPhase { HypnagogicOnset, Nrem, Rem, Integration, HypnopompicReturn }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InferenceTier { T0, T1, T2, T3 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CostCapState { Normal, Warning, SoftCap, HardCap }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProbeResult {
    pub probe: String,
    pub severity: Severity,
    pub value: f64,
    pub threshold: f64,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Severity { None, Low, High }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EscalationSummary {
    pub max_severity: Severity,
    pub probes_triggered: Vec<String>,
    pub tier: InferenceTier,
    pub reason: String,
    pub similar_past_episodes: u32,
    pub applicable_models: Vec<String>,
}
```

**Probes** (16 total, all T0 / deterministic / $0.00):

| Probe | Low threshold | High threshold | Detects |
|---|---|---|---|
| `price_delta` | 50 bps | 200 bps | Price movement since last tick |
| `tvl_delta` | 2% | 10% | TVL change in monitored pools |
| `position_health` | 1.5 | 1.2 | Lending health factor |
| `gas_spike` | 2x | 5x | Gas price vs. 1h average |
| `credit_balance` | 30% remaining | 15% remaining | USDC credit depletion |
| `rsi_extreme` | RSI < 30 | RSI > 70 | Momentum extremes |
| `macd_cross` | signal cross | strong divergence | Trend change |
| `circuit_breaker` | any | any | PolicyCage violation |
| `kill_switch` | any | any | Operator kill command |
| `replicant_report` | any | any | Replicant completed test |
| `clade_alert` | any | any | Clade peer urgent alert |
| `homeostatic` | deviation > 1s | deviation > 2s | Position drift from target |
| `world_model_drift` | 0.3 | 0.6 | Prediction accuracy degradation |
| `causal_graph` | edge weakened | edge broken | Causal relationship invalidated |
| `vpin_spike` | 0.5 | 0.8 | Informed trading probability |
| `il_threshold` | 2% | 5% | Impermanent loss on LP positions |

---

### VitalityState

Updated every tick by `golem-lifespan`.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VitalityState {
    /// Economic clock -- USDC depletion [0, 1].
    pub economic: f64,

    /// Epistemic clock -- predictive fitness [0, 1].
    pub epistemic: f64,

    /// Age factor -- Gompertz aging drag [0, 1].
    pub age_factor: f64,

    /// Composite vitality -- multiplicative combination [0, 1].
    pub composite: f64,

    /// Current behavioral phase.
    pub phase: BehavioralPhase,

    /// Ticks spent in current phase.
    pub ticks_in_phase: u64,

    /// Previous phase.
    pub previous_phase: Option<BehavioralPhase>,

    /// Projected hours remaining at current burn rate.
    pub projected_life_hours: f64,

    /// Survival pressure [0, 1] -- composite inverse, smoothed.
    pub survival_pressure: f64,

    // Owner-only
    pub epistemic_detail: Option<EpistemicFitnessState>,
    pub stochastic_detail: Option<StochasticMortalityState>,
    pub economic_detail: Option<EconomicVitalityDetail>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BehavioralPhase {
    Thriving,      // composite > 0.7
    Stable,        // 0.5-0.7
    Conservation,  // 0.3-0.5
    Declining,     // 0.1-0.3
    Terminal,      // < 0.1
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpistemicFitnessState {
    pub fitness: f64,
    pub fitness_trend: f64,
    pub domain_fitness: DomainFitness,
    pub ticks_below_threshold: u64,
    pub senescent: bool,
    pub senescence_stage: Option<u8>,  // 1, 2, or 3
    pub peak_fitness: f64,
    pub peak_fitness_tick: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainFitness {
    pub price_direction: f64,
    pub volatility_regime: f64,
    pub yield_trend: f64,
    pub gas_pattern: f64,
    pub protocol_behavior: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StochasticMortalityState {
    pub tick_number: u64,
    pub current_hazard: f64,
    pub survival_probability: f64,
    pub survived: bool,
    pub last_roll: f64,  // internal-only
    pub estimated_median_remaining_ticks: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EconomicVitalityDetail {
    pub remaining_usdc: f64,
    pub burn_rate: BurnRateState,
    pub partitions: CreditPartitions,
    pub reserve: ApoptoticReserve,
    pub net_income_rate_24h: f64,
    pub sustainability: SustainabilityMetrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BurnRateState {
    pub compute_per_hour: f64,
    pub inference_per_hour: f64,
    pub gas_per_hour: f64,
    pub data_per_hour: f64,
    pub insurance_per_hour: f64,
    pub total_per_hour: f64,
    pub regime_cost_profile: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreditPartitions {
    pub llm: PartitionState,
    pub gas: PartitionState,
    pub data: PartitionState,
    pub legacy: PartitionState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartitionState {
    pub allocated: f64,
    pub spent: f64,
    pub remaining: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApoptoticReserve {
    pub total_usdc: f64,
    pub settle: f64,
    pub life_review: f64,
    pub legacy: f64,
    pub locked: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SustainabilityMetrics {
    pub revenue_per_hour: f64,
    pub burn_per_hour: f64,
    pub sustainability_ratio: f64,
    pub time_to_sustainability: f64,
    pub homeostatic: bool,
}
```

**Phase behavior effects:**

| Phase | Tick interval | Inference ceiling | Trading | Clade sharing | Replicants |
|---|---|---|---|---|---|
| Thriving | 1x | T3 (Opus) | Full | Normal thresholds | Spawn enabled |
| Stable | 1x | T2 (Sonnet) | Full | Normal thresholds | Spawn enabled |
| Conservation | 2x | T1 (Haiku) | Monitor-only | Elevated thresholds | Disabled |
| Declining | 2x | T1 (Haiku) | Unwind only | Emergency sharing | Disabled |
| Terminal | N/A | T1 (one-shot) | Settlement only | Death notification | Disabled |

---

### MoodState

Updated by `golem-daimon` when PAD Euclidean delta > 0.15.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoodState {
    pub pad: PadVector,
    pub label: String,             // Plutchik primary
    pub intensity: String,         // "mild" | "moderate" | "intense"
    pub octant: String,            // PAD octant name
    pub persistence_ticks: u64,
    pub trigger: String,
    pub personality_baseline: PadVector,
    pub tone: String,              // conversational tone

    // Owner-only
    pub history: Option<Vec<MoodSample>>,

    // Internal-only
    pub last_appraisal: Option<AppraisalResult>,
    pub contagion: Option<ContagionState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PadVector {
    pub pleasure: f64,   // [-1.0, 1.0]
    pub arousal: f64,    // [-1.0, 1.0]
    pub dominance: f64,  // [-1.0, 1.0]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoodSample {
    pub tick: u64,
    pub timestamp: String,
    pub pad: PadVector,
    pub label: String,
    pub trigger: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppraisalResult {
    pub emotion: String,
    pub pad: PadVector,
    pub intensity: f64,
    pub trigger_event_type: String,
    pub trigger_tick: u64,
    pub goal_relevance: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContagionState {
    pub sources: Vec<ContagionSource>,
    pub net_effect: PadVector,
    pub last_sync_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContagionSource {
    pub peer_id: String,
    pub pad: PadVector,
    pub attenuation: f64,
    pub received_at: String,
}
```

---

### DreamState

Updated by `golem-dream` on mode/phase transitions.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DreamState {
    pub mode: GolemMode,
    pub tier: String,                  // "core" | "hardened"
    pub current_phase: Option<DreamSubPhase>,
    pub cycle_number: u32,
    pub total_cycles_this_window: u32,
    pub budget_remaining: f64,
    pub budget_spent: f64,
    pub episodes_replayed: u32,
    pub hypotheses_generated: u32,
    pub threats_simulated: u32,
    pub dream_start_tick: u64,
    pub last_dream_tick: u64,
    pub urgency: f64,
    pub episode_count: u64,
    pub lifetime_cycles: u32,
    pub avg_dream_quality: f64,

    // Owner-only
    pub budget_config: Option<DreamBudget>,
    pub active_hypotheses: Option<Vec<HypothesisSummary>>,
    pub active_threats: Option<Vec<ThreatSummary>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GolemMode { Waking, Dreaming, Dying }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DreamBudget {
    pub per_cycle_max: f64,
    pub per_session_max: f64,
    pub monthly_max: f64,
    pub nrem_allocation: f64,
    pub rem_allocation: f64,
    pub integration_allocation: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HypothesisSummary {
    pub id: String,
    pub source: String,
    pub description: String,
    pub confidence: f64,
    pub status: String,
    pub created_cycle: u32,
    pub last_tested_tick: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreatSummary {
    pub id: String,
    pub tier: u8,
    pub category: String,
    pub description: String,
    pub rehearsed: bool,
    pub impact_severity: f64,
    pub response_confidence: f64,
}
```

---

### HypnagogicState

Updated by `golem-hypnagogia` during onset and return phases.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HypnagogicState {
    pub phase: Option<HypnagogicPhase>,
    pub depth: f64,                     // [0.0, 1.0] — dissolution depth during onset, recrystallization during return
    pub waking_residue_count: u32,      // Unresolved waking threads captured at onset
    pub dali_interrupt_pending: bool,   // True during the Dali interrupt window in return
    pub onset_duration_ms: u64,
    pub return_duration_ms: u64,
    pub liminal_insights_captured: u32, // Fragments captured at the onset/return boundary
    pub last_onset_tick: u64,
    pub last_return_tick: u64,

    // Owner-only
    pub waking_residue: Option<Vec<WakingResidue>>,
    pub liminal_fragments: Option<Vec<LiminalFragment>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HypnagogicPhase {
    Onset,   // Waking → Dreaming transition
    Return,  // Dreaming → Waking transition
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WakingResidue {
    pub thread_id: String,
    pub description: String,
    pub emotional_charge: f64,
    pub source_tick: u64,
    pub resolved_in_dream: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiminalFragment {
    pub id: String,
    pub content: String,
    pub phase: HypnagogicPhase,   // Which transition produced it
    pub confidence: f64,
    pub promoted_to_staging: bool,
    pub captured_at: String,       // ISO-8601
}
```

The `depth` field tracks how far the Golem has progressed through the current liminal transition. During onset, depth increases from 0.0 (fully waking) to 1.0 (fully dissolved, ready for NREM). During return, depth decreases from 1.0 (still in dream cognition) to 0.0 (fully recrystallized, ready for waking). The TUI renders this as a depth gauge (see `../18-interfaces/03-tui.md`).

> **Cross-ref**: `../06-hypnagogia/02-architecture.md` (onset/return phase specifications), `../05-dreams/01-architecture.md` (DreamPhase enum)

---

### CognitionState, PerformanceState, MemoryState, CladeState, EconomicState, DeathState

These follow the same pattern. Each is a Rust struct with `serde::Serialize + serde::Deserialize`, written by a specific extension, projected to a snapshot at tick end.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CognitionState {
    pub decision_cache: DecisionCacheStats,
    pub insights_active: u32,
    pub insights_candidate: u32,
    pub heuristics: u32,
    pub warnings: u32,
    pub causal_graph_nodes: u32,
    pub causal_graph_edges: u32,
    pub regime: RegimeTag,
    pub playbook_revisions: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionCacheStats {
    pub total_entries: u32,
    pub hits_24h: u32,
    pub misses_24h: u32,
    pub hit_rate: f64,
    pub estimated_savings_usdc: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegimeTag {
    pub volatility_quintile: u8,       // 1-5
    pub trend_direction: String,       // "up" | "down" | "range"
    pub gas_price_level: String,       // "low" | "normal" | "high" | "spike"
    pub liquidity_condition: String,   // "deep" | "normal" | "thin" | "crisis"
    pub timestamp: u64,
    pub block_number: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceState {
    pub nav_usdc: f64,
    pub pnl_24h: f64,
    pub pnl_7d: f64,
    pub pnl_30d: f64,
    pub sharpe_30d: f64,
    pub max_drawdown_30d: f64,
    pub trade_count_24h: u32,
    pub win_rate_30d: f64,
    pub positions: Option<Vec<PositionSummary>>,  // Owner-only
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionSummary {
    pub protocol: String,
    pub position_type: String,
    pub value_usdc: f64,
    pub pnl_usdc: f64,
    pub entry_tick: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryState {
    pub episode_count: u64,
    pub insight_count: u32,
    pub heuristic_count: u32,
    pub warning_count: u32,
    pub causal_link_count: u32,
    pub grimoire_disk_bytes: u64,
    pub last_curator_tick: u64,
    pub styx_vault_last_upload: Option<String>,  // ISO-8601
    pub styx_vault_connected: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CladeState {
    pub peers_connected: u32,
    pub entries_pushed: u64,
    pub entries_received: u64,
    pub alerts_received: u32,
    pub last_sync_at: Option<String>,
    pub peers: Option<Vec<CladePeer>>,  // Owner-only
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CladePeer {
    pub peer_id: String,
    pub status: String,      // "online" | "offline" | "dying"
    pub erc8004_id: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EconomicState {
    pub balance_usdc: f64,
    pub projected_ttl_hours: f64,
    pub burn_rate_per_hour: f64,
    pub partitions: CreditPartitions,
    pub cost_cap_state: CostCapState,
    pub daily_accumulator: f64,
    pub daily_remaining: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeathState {
    pub protocol_active: bool,
    pub phase: Option<DeathProtocolPhase>,
    pub cause: Option<String>,
    pub started_at_tick: Option<u64>,
    pub settlement_progress: Option<SettlementProgress>,
    pub testament_progress: Option<TestamentProgress>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeathProtocolPhase {
    Acceptance,  // Phase 0: 1-5 ticks, deterministic
    Settlement,  // Phase I: variable, close positions
    Reflection,  // Phase II: 5-15 ticks, Opus-level review
    Legacy,      // Phase III: 2-5 ticks, upload to Styx Archive
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SettlementProgress {
    pub positions_total: u32,
    pub positions_closed: u32,
    pub usdc_recovered: f64,
    pub failed_settlements: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestamentProgress {
    pub sections_complete: u32,
    pub sections_total: u32,
    pub uploaded_to_styx: bool,
    pub upload_timestamp: Option<String>,
}
```

---

## State change and event emission

Every mutation to `GolemState` emits a corresponding `GolemEvent` through the Event Fabric. The Event Fabric is a `tokio::broadcast` channel with a 10,000-entry ring buffer for reconnection replay.

```rust
impl GolemState {
    /// Apply a state change and emit the corresponding event.
    pub fn apply(&mut self, change: StateChange, fabric: &EventFabric) {
        match change {
            StateChange::VitalityUpdate(v) => {
                self.vitality = v.clone();
                fabric.emit(GolemEvent::VitalityUpdate {
                    timestamp: now_ms(),
                    golem_id: self.identity.golem_id.clone(),
                    sequence: fabric.next_seq(),
                    vitality: v.composite,
                    phase: format!("{:?}", v.phase),
                    credit_balance: self.economic.balance_usdc,
                    projected_life_hours: v.projected_life_hours,
                    clocks: VitalityClocks {
                        economic: v.economic,
                        epistemic: v.epistemic,
                        stochastic: v.age_factor,
                    },
                });
            }
            // ... other state changes follow the same pattern
        }
    }
}
```

See `./12-realtime-subscriptions.md` for the complete `GolemEvent` enum with all 50+ variants.

---

## Snapshot projection

At the end of each tick, `GolemState` projects to `GolemSnapshot`. The projection strips internal-only fields based on the connection's auth tier.

```rust
impl GolemState {
    pub fn snapshot(&self, tier: AuthTier) -> GolemSnapshot {
        GolemSnapshot {
            seq: self.next_seq(),
            tick: self.heartbeat.tick_number,
            timestamp: now_iso8601(),
            identity: self.identity.project(tier),
            heartbeat: self.heartbeat.project(tier),
            vitality: self.vitality.project(tier),
            mood: self.mood.project(tier),
            dream: self.dream.project(tier),
            hypnagogia: self.hypnagogia.project(tier),
            cognition: self.cognition.project(tier),
            performance: self.performance.project(tier),
            memory: self.memory.project(tier),
            clade: self.clade.project(tier),
            economic: self.economic.project(tier),
            death: self.death.project(tier),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthTier { Public, Owner, Internal }
```

---

## Type-state lifecycle

The Golem's lifecycle phases are enforced at the type level:

```rust
pub struct Golem<Phase> {
    state: GolemState,
    _phase: std::marker::PhantomData<Phase>,
}

pub struct Booting;
pub struct Running;
pub struct Dying;

impl Golem<Booting> {
    pub fn start(config: GolemConfig) -> Self { /* ... */ }
    pub fn ready(self) -> Golem<Running> { /* first heartbeat passed */ }
}

impl Golem<Running> {
    pub fn tick(&mut self, fabric: &EventFabric) { /* heartbeat loop */ }
    pub fn die(self, cause: String) -> Golem<Dying> { /* Thanatopsis */ }
}

impl Golem<Dying> {
    pub fn settle(&mut self) { /* Phase I */ }
    pub fn reflect(&mut self) { /* Phase II */ }
    pub fn legacy(self) { /* Phase III, then drop */ }
}
```

A running Golem cannot call `settle()`. A dying Golem cannot call `tick()`. The compiler enforces the lifecycle.

---

> **Design system binding:** Every field in this state model has a visual binding in the design system §24 (Data-to-Visual Binding Reference). See `tmp/research/design-inspo/bardo-design-system-v2.md`.

---

_End of document._

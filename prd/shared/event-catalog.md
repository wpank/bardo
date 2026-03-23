# GolemEvent catalog [SPEC]

> **Document Type**: SPEC (normative) | **Scope**: `golem-core` (events.rs, event_fabric.rs) | **Version**: 2.0.0
> **Prerequisites**: [Observability](../13-runtime/09-observability.md) (structured logging, metrics, and tracing infrastructure), [Realtime subscriptions](../13-runtime/12-realtime-subscriptions.md) (WebSocket event streaming to TUI and external consumers), [State model](../13-runtime/11-state-model.md) (Golem runtime state machine and lifecycle transitions)
> **Last Updated**: 2026-03-14
>
> The complete GolemEvent enum. 22 subsystems, 87 variants, one source of truth. Referenced by runtime, TUI, inference gateway, observability, and coordination specs. If an event isn't in this file, it doesn't exist.

> **Reader orientation:** This is the complete GolemEvent catalog: 22 subsystems, 87 event variants, one source of truth. It belongs to the `shared/` reference layer and is the authoritative enum definition for all events emitted by the Golem (mortal autonomous agent) runtime. Every subsystem -- from Heartbeat (the 9-step decision cycle) to Daimon (the affect engine implementing PAD emotional state) to Thanatopsis (the four-phase structured shutdown protocol) -- emits typed events through this catalog. If an event is not listed here, it does not exist. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Conventions

Every variant carries four base fields, flattened (serde's internally-tagged enums require flat structs):

- `timestamp: u64` -- Unix milliseconds (wall clock)
- `golem_id: String` -- which Golem emitted this
- `sequence: u64` -- monotonically increasing per Golem, gap = dropped events
- `tick: u64` -- heartbeat tick number at time of emission

Wire format: `#[serde(tag = "type")]` with dot-notation rename. Field names are `snake_case`. The base fields are omitted from the per-variant payload tables in section 2 to avoid repetition.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base fields (timestamp, golem_id, sequence, tick) abbreviated as `// base`
/// in the listing below. All String fields are heap-allocated.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GolemEvent {

    // ── Heartbeat ────────────────────────────────────────────────
    #[serde(rename = "heartbeat.market_observation")]
    MarketObservation { /* base, */ regime: String, anomalies: Vec<String>, probe_count: u32 },
    #[serde(rename = "heartbeat.tick")]
    HeartbeatTick { /* base, */ tier: String, pe: f64, threshold: f64, cost: f64 },
    #[serde(rename = "heartbeat.complete")]
    HeartbeatComplete { /* base, */ duration_ms: u64, actions_taken: u32, total_cost: f64 },

    // ── Perception ───────────────────────────────────────────────
    #[serde(rename = "heartbeat.tier_selected")]
    TierSelected { /* base, */ tier: String, rationale: String, pe: f64, threshold: f64, vitality: f64, arousal: f64 },

    // ── Tools ────────────────────────────────────────────────────
    #[serde(rename = "tool.start")]
    ToolExecutionStart { /* base, */ tool_name: String, action_kind: Option<String>, params_hash: String, risk_tier: String, permit_id: Option<String> },
    #[serde(rename = "tool.update")]
    ToolExecutionUpdate { /* base, */ tool_name: String, progress: Option<f64>, step: Option<String>, intermediate: Option<String> },
    #[serde(rename = "tool.end")]
    ToolExecutionEnd { /* base, */ tool_name: String, result: String, tx_hash: Option<String>, gas_used: Option<f64>, duration_ms: u64, block_reason: Option<String> },

    // ── Risk / Permits ───────────────────────────────────────────
    #[serde(rename = "permit.created")]
    PermitCreated { /* base, */ permit_id: String, action_kind: String, risk_tier: String, simulation_hash: Option<String>, expires_at: u64 },
    #[serde(rename = "permit.committed")]
    PermitCommitted { /* base, */ permit_id: String, action_kind: String, risk_tier: String, tx_hash: Option<String> },
    #[serde(rename = "permit.expired")]
    PermitExpired { /* base, */ permit_id: String, action_kind: String, risk_tier: String },
    #[serde(rename = "permit.blocked")]
    PermitBlocked { /* base, */ permit_id: String, action_kind: String, risk_tier: String, block_reason: String },

    // ── Inference ────────────────────────────────────────────────
    #[serde(rename = "inference.start")]
    InferenceStart { /* base, */ model: String, tier: String, estimated_input_tokens: u32, pipeline_profile: String },
    #[serde(rename = "inference.token")]
    InferenceToken { /* base, */ model: String, tier: String, token_count: u64 },
    #[serde(rename = "inference.end")]
    InferenceEnd { /* base, */ model: String, tier: String, input_tokens: u32, output_tokens: u32, cache_read_tokens: u32, cost_usd: f64, latency_ms: u64, decision: Option<String> },
    #[serde(rename = "inference.cache_hit")]
    CacheHit { /* base, */ cache_layer: String, similarity: Option<f64>, savings_usd: f64 },
    #[serde(rename = "inference.provider_fallback")]
    ProviderFallback { /* base, */ failed_provider: String, fallback_provider: String, error: String },

    // ── Dreams ───────────────────────────────────────────────────
    #[serde(rename = "dream.start")]
    DreamStart { /* base, */ cycle_id: String, trigger: String },
    #[serde(rename = "dream.phase")]
    DreamPhase { /* base, */ cycle_id: String, phase: String, budget_tokens: u32 },
    #[serde(rename = "dream.hypothesis")]
    DreamHypothesis { /* base, */ cycle_id: String, hypothesis: String, seed_episode_tick: u64 },
    #[serde(rename = "dream.counterfactual")]
    DreamCounterfactual { /* base, */ cycle_id: String, hypothesis: String, outcome: String, lesson: Option<String>, confidence: f64 },
    #[serde(rename = "dream.complete")]
    DreamComplete { /* base, */ cycle_id: String, episodes_replayed: u32, counterfactuals_generated: u32, insights_crystallized: u32, heuristics_proposed: u32, episodes_depotentiated: u32, playbook_edits: u32 },

    // ── Daimon / Affect ──────────────────────────────────────────
    #[serde(rename = "daimon.appraisal")]
    DaimonAppraisal { /* base, */ pleasure: f64, arousal: f64, dominance: f64, emotion: String, markers_fired: u32, intensity: f64 },
    #[serde(rename = "daimon.somatic_marker")]
    SomaticMarkerFired { /* base, */ situation: String, valence: f64, source: String, strategy_param: Option<String> },
    #[serde(rename = "daimon.emotional_shift")]
    EmotionalShift { /* base, */ from: String, to: String, trigger: String },

    // ── Mortality ────────────────────────────────────────────────
    #[serde(rename = "vitality.update")]
    VitalityUpdate { /* base, */ composite: f64, economic: f64, epistemic: f64, stochastic: f64, phase: String, credit_balance: f64, projected_life_hours: f64 },
    #[serde(rename = "vitality.phase_transition")]
    PhaseTransition { /* base, */ from: String, to: String, trigger: String, vitality_at_transition: f64 },
    #[serde(rename = "vitality.death_initiated")]
    DeathInitiated { /* base, */ cause: String, vitality_at_death: f64, ticks_lived: u64 },
    #[serde(rename = "vitality.thanatopsis_phase")]
    ThanatopsisPhase { /* base, */ phase: String, detail: Option<String> },
    #[serde(rename = "vitality.successor_spawned")]
    SuccessorSpawned { /* base, */ successor_id: String, generation: u32, genome_entries: u32, inherited_heuristics: u32 },

    // ── Grimoire ─────────────────────────────────────────────────
    #[serde(rename = "grimoire.insight")]
    InsightGenerated { /* base, */ entry_id: String, confidence: f64, summary: String, source: String, provenance: String },
    #[serde(rename = "grimoire.heuristic_promoted")]
    HeuristicPromoted { /* base, */ entry_id: String, from_tier: String, to_tier: String, confidence: f64, validation_count: u32 },
    #[serde(rename = "grimoire.heuristic_demoted")]
    HeuristicDemoted { /* base, */ entry_id: String, from_tier: String, to_tier: String, reason: String },
    #[serde(rename = "grimoire.warning")]
    WarningReceived { /* base, */ entry_id: String, confidence: f64, summary: String, provenance: String, is_bloodstain: bool },
    #[serde(rename = "grimoire.curator_cycle")]
    CuratorCycle { /* base, */ entries_processed: u32, entries_pruned: u32, entries_promoted: u32, entries_demoted: u32, duration_ms: u64 },
    #[serde(rename = "grimoire.playbook_evolved")]
    PlaybookEvolved { /* base, */ edits: u32, dream_cycle_id: String },
    #[serde(rename = "grimoire.causal_edge")]
    CausalEdgeDiscovered { /* base, */ entry_id: String, from_variable: String, to_variable: String, direction: String, confidence: f64, lag_ticks: u32 },

    // ── Coordination ─────────────────────────────────────────────
    #[serde(rename = "coordination.pheromone_deposited")]
    PheromoneDeposited { /* base, */ layer: String, signal_class: String, strength: f64, domain: String },
    #[serde(rename = "coordination.pheromone_sensed")]
    PheromoneSensed { /* base, */ layer: String, signal_class: String, strength: f64, confirmations: u32, source_hash: String },
    #[serde(rename = "coordination.pheromone_evaporated")]
    PheromoneEvaporated { /* base, */ layer: String, signal_class: String, original_strength: f64, age_seconds: u64 },
    #[serde(rename = "coordination.bloodstain_received")]
    BloodstainReceived { /* base, */ source_golem_id: String, source_generation: u32, death_cause: String, warnings_count: u32, causal_edges_count: u32 },
    #[serde(rename = "coordination.clade_sync")]
    CladeSyncComplete { /* base, */ entries_sent: u32, entries_received: u32, siblings_online: u32 },

    // ── Context ──────────────────────────────────────────────────
    #[serde(rename = "context.assembled")]
    ContextAssembled { /* base, */ total_tokens: u64, category_sizes: HashMap<String, u64>, policy_revision: u64, steer_present: bool },
    #[serde(rename = "context.compaction")]
    CompactionTriggered { /* base, */ tokens_before: u64, tokens_after: u64, strategy: String, invariants_preserved: Vec<String> },
    #[serde(rename = "context.tools_pruned")]
    ToolsPruned { /* base, */ from_count: u32, to_count: u32, tick_type: String },

    // ── Custody / Session ────────────────────────────────────────
    #[serde(rename = "custody.delegation_created")]
    DelegationCreated { /* base, */ delegation_hash: String, delegator: String, delegate: String, caveats: Vec<String>, expires_at: u64 },
    #[serde(rename = "custody.delegation_revoked")]
    DelegationRevoked { /* base, */ delegation_hash: String, revocation_tx: String },
    #[serde(rename = "custody.self_funding_replenish")]
    SelfFundingReplenish { /* base, */ amount_usd: f64, new_balance_usd: f64, sustainability_ratio: f64 },
    #[serde(rename = "custody.session_key_rotated")]
    SessionKeyRotated { /* base, */ reason: String, new_key_address: String, old_key_operations: u64 },
    #[serde(rename = "custody.spending_limit_reached")]
    SpendingLimitReached { /* base, */ limit_usd: f64, spent_usd: f64, resets_at: u64 },

    // ── Lifecycle ────────────────────────────────────────────────
    #[serde(rename = "lifecycle.booted")]
    GolemBooted { /* base, */ generation: u32, config_hash: String },
    #[serde(rename = "lifecycle.shutdown")]
    GolemShutdownStarted { /* base, */ reason: String },

    // ── Engagement / Creature ────────────────────────────────────
    #[serde(rename = "engagement.achievement_unlocked")]
    AchievementUnlocked { /* base, */ achievement_id: String, name: String, category: String, rarity: String, description: String },
    #[serde(rename = "engagement.sprite_evolution")]
    SpriteEvolution { /* base, */ from_form: String, to_form: String, trigger: String },
    #[serde(rename = "engagement.sprite_mutation")]
    SpriteMutation { /* base, */ mutation_type: String, layer: String, cause: String },
    #[serde(rename = "engagement.milestone")]
    MilestoneReached { /* base, */ metric: String, value: u64, previous_milestone: u64 },

    // ── Marketplace ──────────────────────────────────────────────
    #[serde(rename = "marketplace.listing_created")]
    ListingCreated { /* base, */ listing_id: String, entry_type: String, price_usd: f64 },
    #[serde(rename = "marketplace.purchase_complete")]
    PurchaseComplete { /* base, */ listing_id: String, buyer_hash: String, price_usd: f64 },
    #[serde(rename = "marketplace.review_posted")]
    ReviewPosted { /* base, */ listing_id: String, rating: f64, reviewer_hash: String },
    #[serde(rename = "marketplace.autonomous_purchase")]
    AutonomousPurchase { /* base, */ listing_id: String, seller_hash: String, price_usd: f64, entry_type: String, rationale: String },
    #[serde(rename = "marketplace.escrow_released")]
    EscrowReleased { /* base, */ listing_id: String, seller_hash: String, buyer_hash: String, amount_usd: f64, evaluator_hash: String },

    // ── Trading ──────────────────────────────────────────────────
    #[serde(rename = "trading.swap_executed")]
    SwapExecuted { /* base, */ pair: String, side: String, amount_in: f64, amount_out: f64, price: f64, slippage_bps: f64, gas_usd: f64, pnl_usd: Option<f64>, tx_hash: String, venue: String },
    #[serde(rename = "trading.lp_rebalanced")]
    LpRebalanced { /* base, */ pool: String, old_lower_tick: i32, old_upper_tick: i32, new_lower_tick: i32, new_upper_tick: i32, fees_collected_usd: f64, gas_usd: f64, tx_hash: String },
    #[serde(rename = "trading.vault_deposit")]
    VaultDeposit { /* base, */ vault_address: String, asset_amount: f64, shares_received: f64, tx_hash: String },
    #[serde(rename = "trading.vault_withdrawal")]
    VaultWithdrawal { /* base, */ vault_address: String, shares_burned: f64, assets_received: f64, tx_hash: String },
    #[serde(rename = "trading.position_liquidated")]
    PositionLiquidated { /* base, */ position_id: String, protocol: String, loss_usd: f64, cause: String, tx_hash: String },

    // ── CorticalState ─────────────────────────────────────────────
    #[serde(rename = "cortical.signal_updated")]
    CorticalSignalUpdated { /* base, */ signal_name: String, old_value: f64, new_value: f64, source: String },
    #[serde(rename = "cortical.threshold_crossed")]
    CorticalThresholdCrossed { /* base, */ signal_name: String, threshold: f64, direction: String, value: f64 },

    // ── Prediction ────────────────────────────────────────────────
    #[serde(rename = "prediction.created")]
    PredictionCreated { /* base, */ prediction_id: String, domain: String, confidence: f64, interval_low: f64, interval_high: f64, resolves_at: u64 },
    #[serde(rename = "prediction.resolved")]
    PredictionResolved { /* base, */ prediction_id: String, domain: String, predicted: f64, actual: f64, residual: f64, within_interval: bool },
    #[serde(rename = "prediction.expired")]
    PredictionExpired { /* base, */ prediction_id: String, domain: String, reason: String },
    #[serde(rename = "prediction.accuracy_updated")]
    PredictionAccuracyUpdated { /* base, */ domain: String, accuracy_7d: f64, accuracy_30d: f64, calibration_ece: f64, total_resolved: u64 },

    // ── Adaptive Clock ────────────────────────────────────────────
    #[serde(rename = "clock.gamma_tick")]
    GammaTick { /* base, */ interval_ms: u64, arousal: f64, signals_sampled: u32 },
    #[serde(rename = "clock.theta_tick")]
    ThetaTick { /* base, */ interval_ms: u64, predictions_resolved: u32, actions_evaluated: u32 },
    #[serde(rename = "clock.delta_tick")]
    DeltaTick { /* base, */ interval_ms: u64, consolidation_type: String, entries_processed: u32 },
    #[serde(rename = "clock.frequency_adjusted")]
    ClockFrequencyAdjusted { /* base, */ scale: String, old_interval_ms: u64, new_interval_ms: u64, trigger: String },

    // ── Library ───────────────────────────────────────────────────
    #[serde(rename = "library.knowledge_ingested")]
    KnowledgeIngested { /* base, */ entry_id: String, source: String, channel: String, confidence: f64 },
    #[serde(rename = "library.knowledge_equipped")]
    KnowledgeEquipped { /* base, */ entry_id: String, entry_type: String, generation_origin: u32 },
    #[serde(rename = "library.knowledge_decayed")]
    KnowledgeDecayed { /* base, */ entry_id: String, old_confidence: f64, new_confidence: f64, reason: String },

    // ── Necrocracy ────────────────────────────────────────────────
    #[serde(rename = "necrocracy.golem_died")]
    GolemDied { /* base, */ generation: u32, ticks_lived: u64, cause: String, testament_hash: String },
    #[serde(rename = "necrocracy.knowledge_bequeathed")]
    KnowledgeBequeathed { /* base, */ entries_count: u32, warnings_count: u32, causal_edges_count: u32, successor_id: Option<String> },
    #[serde(rename = "necrocracy.lethe_contribution")]
    LetheContribution { /* base, */ entries_count: u32, anonymized: bool },

    // ── Portal ────────────────────────────────────────────────────
    #[serde(rename = "portal.entered")]
    PortalEntered { /* base, */ perspective: String },
    #[serde(rename = "portal.exited")]
    PortalExited { /* base, */ duration_ms: u64 },
    #[serde(rename = "portal.perspective_shifted")]
    PerspectiveShifted { /* base, */ from: String, to: String },

    // ── System ───────────────────────────────────────────────────
    #[serde(rename = "system.config_reloaded")]
    ConfigReloaded { /* base, */ changes: Vec<String> },
    #[serde(rename = "system.extension_registered")]
    ExtensionRegistered { /* base, */ name: String, layer: String },
}
```

---

## 2. Payload schemas by subsystem

### 2.1 Heartbeat (3 variants)

Emitter: `golem-heartbeat`. Fires at pipeline Steps 1, 4, and 9.

| Variant | Wire name | Payload (beyond base) |
|---|---|---|
| `MarketObservation` | `heartbeat.market_observation` | `regime: String`, `anomalies: Vec<String>`, `probe_count: u32` |
| `HeartbeatTick` | `heartbeat.tick` | `tier: String`, `pe: f64`, `threshold: f64`, `cost: f64` |
| `HeartbeatComplete` | `heartbeat.complete` | `duration_ms: u64`, `actions_taken: u32`, `total_cost: f64` |

### 2.2 Perception (1 variant)

Emitter: `golem-heartbeat` (gating logic, Step 4).

| Variant | Wire name | Payload |
|---|---|---|
| `TierSelected` | `heartbeat.tier_selected` | `tier: String`, `rationale: String`, `pe: f64`, `threshold: f64`, `vitality: f64`, `arousal: f64` |

### 2.3 Tools (3 variants)

Emitter: `golem-tools`. Start/update/end triple forms an OTel span.

| Variant | Wire name | Payload |
|---|---|---|
| `ToolExecutionStart` | `tool.start` | `tool_name`, `action_kind: Option<String>`, `params_hash: String`, `risk_tier: String`, `permit_id: Option<String>` |
| `ToolExecutionUpdate` | `tool.update` | `tool_name`, `progress: Option<f64>`, `step: Option<String>`, `intermediate: Option<String>` |
| `ToolExecutionEnd` | `tool.end` | `tool_name`, `result: String`, `tx_hash: Option<String>`, `gas_used: Option<f64>`, `duration_ms: u64`, `block_reason: Option<String>` |

### 2.4 Risk / Permits (4 variants)

Emitter: `golem-safety`. ActionPermit lifecycle.

| Variant | Wire name | Payload |
|---|---|---|
| `PermitCreated` | `permit.created` | `permit_id`, `action_kind`, `risk_tier`, `simulation_hash: Option<String>`, `expires_at: u64` |
| `PermitCommitted` | `permit.committed` | `permit_id`, `action_kind`, `risk_tier`, `tx_hash: Option<String>` |
| `PermitExpired` | `permit.expired` | `permit_id`, `action_kind`, `risk_tier` |
| `PermitBlocked` | `permit.blocked` | `permit_id`, `action_kind`, `risk_tier`, `block_reason: String` |

### 2.5 Inference (5 variants)

Emitter: `golem-inference`. `InferenceToken` is high-frequency during T2 calls.

| Variant | Wire name | Payload |
|---|---|---|
| `InferenceStart` | `inference.start` | `model`, `tier`, `estimated_input_tokens: u32`, `pipeline_profile: String` |
| `InferenceToken` | `inference.token` | `model`, `tier`, `token_count: u64` (cumulative) |
| `InferenceEnd` | `inference.end` | `model`, `tier`, `input_tokens: u32`, `output_tokens: u32`, `cache_read_tokens: u32`, `cost_usd: f64`, `latency_ms: u64`, `decision: Option<String>` |
| `CacheHit` | `inference.cache_hit` | `cache_layer: String`, `similarity: Option<f64>`, `savings_usd: f64` |
| `ProviderFallback` | `inference.provider_fallback` | `failed_provider`, `fallback_provider`, `error` |

### 2.6 Dreams (5 variants)

Emitter: `golem-dreams`. Full dream lifecycle from trigger to consolidation.

| Variant | Wire name | Payload |
|---|---|---|
| `DreamStart` | `dream.start` | `cycle_id`, `trigger: String` |
| `DreamPhase` | `dream.phase` | `cycle_id`, `phase: String` (NREM/REM/Integration), `budget_tokens: u32` |
| `DreamHypothesis` | `dream.hypothesis` | `cycle_id`, `hypothesis: String`, `seed_episode_tick: u64` |
| `DreamCounterfactual` | `dream.counterfactual` | `cycle_id`, `hypothesis`, `outcome`, `lesson: Option<String>`, `confidence: f64` |
| `DreamComplete` | `dream.complete` | `cycle_id`, `episodes_replayed: u32`, `counterfactuals_generated: u32`, `insights_crystallized: u32`, `heuristics_proposed: u32`, `episodes_depotentiated: u32`, `playbook_edits: u32` |

### 2.7 Daimon / Affect (3 variants)

Emitter: `golem-daimon`. `DaimonAppraisal` fires every tick; `EmotionalShift` only on label change.

| Variant | Wire name | Payload |
|---|---|---|
| `DaimonAppraisal` | `daimon.appraisal` | `pleasure: f64`, `arousal: f64`, `dominance: f64`, `emotion: String`, `markers_fired: u32`, `intensity: f64` |
| `SomaticMarkerFired` | `daimon.somatic_marker` | `situation`, `valence: f64`, `source: String`, `strategy_param: Option<String>` |
| `EmotionalShift` | `daimon.emotional_shift` | `from`, `to`, `trigger` |

### 2.8 Mortality (5 variants)

Emitter: `golem-mortality`. `Declining -> Terminal` is irreversible.

| Variant | Wire name | Payload |
|---|---|---|
| `VitalityUpdate` | `vitality.update` | `composite: f64`, `economic: f64`, `epistemic: f64`, `stochastic: f64`, `phase: String`, `credit_balance: f64`, `projected_life_hours: f64` |
| `PhaseTransition` | `vitality.phase_transition` | `from`, `to`, `trigger`, `vitality_at_transition: f64` |
| `DeathInitiated` | `vitality.death_initiated` | `cause: String`, `vitality_at_death: f64`, `ticks_lived: u64` |
| `ThanatopsisPhase` | `vitality.thanatopsis_phase` | `phase: String` (ACCEPTANCE/SETTLEMENT/REFLECTION/LEGACY), `detail: Option<String>` |
| `SuccessorSpawned` | `vitality.successor_spawned` | `successor_id`, `generation: u32`, `genome_entries: u32`, `inherited_heuristics: u32` |

### 2.9 Grimoire (7 variants)

Emitter: `golem-grimoire`. Knowledge lifecycle. `PlaybookEvolved` is the only write path to PLAYBOOK.md.

| Variant | Wire name | Payload |
|---|---|---|
| `InsightGenerated` | `grimoire.insight` | `entry_id`, `confidence: f64`, `summary`, `source: String`, `provenance: String` |
| `HeuristicPromoted` | `grimoire.heuristic_promoted` | `entry_id`, `from_tier`, `to_tier`, `confidence: f64`, `validation_count: u32` |
| `HeuristicDemoted` | `grimoire.heuristic_demoted` | `entry_id`, `from_tier`, `to_tier`, `reason` |
| `WarningReceived` | `grimoire.warning` | `entry_id`, `confidence: f64`, `summary`, `provenance`, `is_bloodstain: bool` |
| `CuratorCycle` | `grimoire.curator_cycle` | `entries_processed: u32`, `entries_pruned: u32`, `entries_promoted: u32`, `entries_demoted: u32`, `duration_ms: u64` |
| `PlaybookEvolved` | `grimoire.playbook_evolved` | `edits: u32`, `dream_cycle_id` |
| `CausalEdgeDiscovered` | `grimoire.causal_edge` | `entry_id`, `from_variable`, `to_variable`, `direction: String`, `confidence: f64`, `lag_ticks: u32` |

### 2.10 Coordination (5 variants)

Emitter: `golem-coordination`. Pheromone field + bloodstain network + clade sync.

| Variant | Wire name | Payload |
|---|---|---|
| `PheromoneDeposited` | `coordination.pheromone_deposited` | `layer: String`, `signal_class`, `strength: f64`, `domain` |
| `PheromoneSensed` | `coordination.pheromone_sensed` | `layer`, `signal_class`, `strength: f64`, `confirmations: u32`, `source_hash: String` |
| `PheromoneEvaporated` | `coordination.pheromone_evaporated` | `layer`, `signal_class`, `original_strength: f64`, `age_seconds: u64` |
| `BloodstainReceived` | `coordination.bloodstain_received` | `source_golem_id`, `source_generation: u32`, `death_cause`, `warnings_count: u32`, `causal_edges_count: u32` |
| `CladeSyncComplete` | `coordination.clade_sync` | `entries_sent: u32`, `entries_received: u32`, `siblings_online: u32` |

### 2.11 Context (3 variants)

Emitter: `golem-context`. Context window management.

| Variant | Wire name | Payload |
|---|---|---|
| `ContextAssembled` | `context.assembled` | `total_tokens: u64`, `category_sizes: HashMap<String, u64>`, `policy_revision: u64`, `steer_present: bool` |
| `CompactionTriggered` | `context.compaction` | `tokens_before: u64`, `tokens_after: u64`, `strategy: String`, `invariants_preserved: Vec<String>` |
| `ToolsPruned` | `context.tools_pruned` | `from_count: u32`, `to_count: u32`, `tick_type: String` |

### 2.12 Custody / Session (5 variants)

Emitter: `golem-chain` (wallet layer). Delegation, keys, spending limits.

| Variant | Wire name | Payload |
|---|---|---|
| `DelegationCreated` | `custody.delegation_created` | `delegation_hash`, `delegator`, `delegate`, `caveats: Vec<String>`, `expires_at: u64` |
| `DelegationRevoked` | `custody.delegation_revoked` | `delegation_hash`, `revocation_tx` |
| `SelfFundingReplenish` | `custody.self_funding_replenish` | `amount_usd: f64`, `new_balance_usd: f64`, `sustainability_ratio: f64` |
| `SessionKeyRotated` | `custody.session_key_rotated` | `reason: String`, `new_key_address`, `old_key_operations: u64` |
| `SpendingLimitReached` | `custody.spending_limit_reached` | `limit_usd: f64`, `spent_usd: f64`, `resets_at: u64` |

### 2.13 Lifecycle (2 variants)

Emitter: `golem-runtime`. Process-level events.

| Variant | Wire name | Payload |
|---|---|---|
| `GolemBooted` | `lifecycle.booted` | `generation: u32`, `config_hash` |
| `GolemShutdownStarted` | `lifecycle.shutdown` | `reason: String` |

### 2.14 Engagement / Creature (4 variants)

Emitter: `golem-engagement`. Retention loop events.

| Variant | Wire name | Payload |
|---|---|---|
| `AchievementUnlocked` | `engagement.achievement_unlocked` | `achievement_id`, `name`, `category`, `rarity: String`, `description` |
| `SpriteEvolution` | `engagement.sprite_evolution` | `from_form`, `to_form`, `trigger` |
| `SpriteMutation` | `engagement.sprite_mutation` | `mutation_type`, `layer`, `cause` |
| `MilestoneReached` | `engagement.milestone` | `metric`, `value: u64`, `previous_milestone: u64` |

### 2.15 Marketplace (5 variants)

Emitter: `golem-coordination` (marketplace subsystem). x402-gated knowledge market.

| Variant | Wire name | Payload |
|---|---|---|
| `ListingCreated` | `marketplace.listing_created` | `listing_id`, `entry_type`, `price_usd: f64` |
| `PurchaseComplete` | `marketplace.purchase_complete` | `listing_id`, `buyer_hash`, `price_usd: f64` |
| `ReviewPosted` | `marketplace.review_posted` | `listing_id`, `rating: f64`, `reviewer_hash` |
| `AutonomousPurchase` | `marketplace.autonomous_purchase` | `listing_id`, `seller_hash`, `price_usd: f64`, `entry_type`, `rationale` |
| `EscrowReleased` | `marketplace.escrow_released` | `listing_id`, `seller_hash`, `buyer_hash`, `amount_usd: f64`, `evaluator_hash` |

### 2.16 Trading (5 variants)

Emitter: `golem-tools` (trading tool handlers). DeFi-specific fields beyond `ToolExecutionEnd`.

| Variant | Wire name | Payload |
|---|---|---|
| `SwapExecuted` | `trading.swap_executed` | `pair`, `side`, `amount_in: f64`, `amount_out: f64`, `price: f64`, `slippage_bps: f64`, `gas_usd: f64`, `pnl_usd: Option<f64>`, `tx_hash`, `venue` |
| `LpRebalanced` | `trading.lp_rebalanced` | `pool`, `old_lower_tick: i32`, `old_upper_tick: i32`, `new_lower_tick: i32`, `new_upper_tick: i32`, `fees_collected_usd: f64`, `gas_usd: f64`, `tx_hash` |
| `VaultDeposit` | `trading.vault_deposit` | `vault_address`, `asset_amount: f64`, `shares_received: f64`, `tx_hash` |
| `VaultWithdrawal` | `trading.vault_withdrawal` | `vault_address`, `shares_burned: f64`, `assets_received: f64`, `tx_hash` |
| `PositionLiquidated` | `trading.position_liquidated` | `position_id`, `protocol`, `loss_usd: f64`, `cause`, `tx_hash` |

### 2.17 System (2 variants)

Emitter: `golem-runtime`. Rare, operational events.

| Variant | Wire name | Payload |
|---|---|---|
| `ConfigReloaded` | `system.config_reloaded` | `changes: Vec<String>` |
| `ExtensionRegistered` | `system.extension_registered` | `name`, `layer` |

### 2.18 CorticalState (2 variants)

Emitter: `golem-cortex`. Fires when any of the 32 CorticalState signals change beyond their delta threshold.

| Variant | Wire name | Payload |
|---|---|---|
| `CorticalSignalUpdated` | `cortical.signal_updated` | `signal_name: String`, `old_value: f64`, `new_value: f64`, `source: String` |
| `CorticalThresholdCrossed` | `cortical.threshold_crossed` | `signal_name: String`, `threshold: f64`, `direction: String` (rising/falling), `value: f64` |

### 2.19 Prediction (4 variants)

Emitter: `golem-prediction`. Full prediction lifecycle from creation through resolution.

| Variant | Wire name | Payload |
|---|---|---|
| `PredictionCreated` | `prediction.created` | `prediction_id`, `domain`, `confidence: f64`, `interval_low: f64`, `interval_high: f64`, `resolves_at: u64` |
| `PredictionResolved` | `prediction.resolved` | `prediction_id`, `domain`, `predicted: f64`, `actual: f64`, `residual: f64`, `within_interval: bool` |
| `PredictionExpired` | `prediction.expired` | `prediction_id`, `domain`, `reason: String` |
| `PredictionAccuracyUpdated` | `prediction.accuracy_updated` | `domain`, `accuracy_7d: f64`, `accuracy_30d: f64`, `calibration_ece: f64`, `total_resolved: u64` |

### 2.20 Adaptive Clock (4 variants)

Emitter: `golem-heartbeat` (clock subsystem). Triple-timescale tick events and frequency adjustments.

| Variant | Wire name | Payload |
|---|---|---|
| `GammaTick` | `clock.gamma_tick` | `interval_ms: u64`, `arousal: f64`, `signals_sampled: u32` |
| `ThetaTick` | `clock.theta_tick` | `interval_ms: u64`, `predictions_resolved: u32`, `actions_evaluated: u32` |
| `DeltaTick` | `clock.delta_tick` | `interval_ms: u64`, `consolidation_type: String`, `entries_processed: u32` |
| `ClockFrequencyAdjusted` | `clock.frequency_adjusted` | `scale: String` (gamma/theta/delta), `old_interval_ms: u64`, `new_interval_ms: u64`, `trigger: String` |

### 2.21 Library (3 variants)

Emitter: `golem-library`. Knowledge persistence lifecycle across golem generations.

| Variant | Wire name | Payload |
|---|---|---|
| `KnowledgeIngested` | `library.knowledge_ingested` | `entry_id`, `source: String`, `channel: String` (death_deposit/marketplace/clade/lethe/manual), `confidence: f64` |
| `KnowledgeEquipped` | `library.knowledge_equipped` | `entry_id`, `entry_type: String`, `generation_origin: u32` |
| `KnowledgeDecayed` | `library.knowledge_decayed` | `entry_id`, `old_confidence: f64`, `new_confidence: f64`, `reason: String` |

### 2.22 Necrocracy (3 variants)

Emitter: `golem-mortality` (death subsystem). Death-as-infrastructure events for the necrocratic knowledge economy.

| Variant | Wire name | Payload |
|---|---|---|
| `GolemDied` | `necrocracy.golem_died` | `generation: u32`, `ticks_lived: u64`, `cause: String`, `testament_hash: String` |
| `KnowledgeBequeathed` | `necrocracy.knowledge_bequeathed` | `entries_count: u32`, `warnings_count: u32`, `causal_edges_count: u32`, `successor_id: Option<String>` |
| `LetheContribution` | `necrocracy.lethe_contribution` | `entries_count: u32`, `anonymized: bool` |

### 2.23 Portal (3 variants)

Emitter: `golem-tui` (portal subsystem). First-person perspective mode events.

| Variant | Wire name | Payload |
|---|---|---|
| `PortalEntered` | `portal.entered` | `perspective: String` |
| `PortalExited` | `portal.exited` | `duration_ms: u64` |
| `PerspectiveShifted` | `portal.perspective_shifted` | `from: String`, `to: String` |

---

## 3. Subscription categories

Clients subscribe to categories (dot-notation prefix), not individual variants.

| Category | Variants | Primary TUI destination |
|---|---|---|
| `heartbeat` | MarketObservation, HeartbeatTick, HeartbeatComplete, TierSelected | HEARTH:Overview |
| `tool` | ToolExecutionStart, ToolExecutionUpdate, ToolExecutionEnd | HEARTH:Operations |
| `permit` | PermitCreated, PermitCommitted, PermitExpired, PermitBlocked | SOMA:Custody |
| `inference` | InferenceStart, InferenceToken, InferenceEnd, CacheHit, ProviderFallback | MIND:Inference |
| `dream` | DreamStart, DreamPhase, DreamHypothesis, DreamCounterfactual, DreamComplete | MIND:Dreams |
| `daimon` | DaimonAppraisal, SomaticMarkerFired, EmotionalShift | HEARTH:Signals |
| `vitality` | VitalityUpdate, PhaseTransition, DeathInitiated, ThanatopsisPhase, SuccessorSpawned | FATE:Mortality |
| `grimoire` | InsightGenerated, HeuristicPromoted, HeuristicDemoted, WarningReceived, CuratorCycle, PlaybookEvolved, CausalEdgeDiscovered | MIND:Grimoire |
| `coordination` | PheromoneDeposited, PheromoneSensed, PheromoneEvaporated, BloodstainReceived, CladeSyncComplete | WORLD:Clade |
| `context` | ContextAssembled, CompactionTriggered, ToolsPruned | HEARTH:Overview |
| `custody` | DelegationCreated, DelegationRevoked, SelfFundingReplenish, SessionKeyRotated, SpendingLimitReached | SOMA:Custody |
| `lifecycle` | GolemBooted, GolemShutdownStarted | Connection indicator |
| `engagement` | AchievementUnlocked, SpriteEvolution, SpriteMutation, MilestoneReached | FATE:Achievements |
| `marketplace` | ListingCreated, PurchaseComplete, ReviewPosted, AutonomousPurchase, EscrowReleased | SOMA:Bazaar |
| `trading` | SwapExecuted, LpRebalanced, VaultDeposit, VaultWithdrawal, PositionLiquidated | SOMA:Portfolio |
| `cortical` | CorticalSignalUpdated, CorticalThresholdCrossed | HEARTH:Signals |
| `prediction` | PredictionCreated, PredictionResolved, PredictionExpired, PredictionAccuracyUpdated | MIND:Pipeline |
| `clock` | GammaTick, ThetaTick, DeltaTick, ClockFrequencyAdjusted | HEARTH:Overview |
| `library` | KnowledgeIngested, KnowledgeEquipped, KnowledgeDecayed | MIND:Grimoire |
| `necrocracy` | GolemDied, KnowledgeBequeathed, LetheContribution | FATE:Mortality |
| `portal` | PortalEntered, PortalExited, PerspectiveShifted | Portal (F4) |
| `system` | ConfigReloaded, ExtensionRegistered | Debug only |

Composite subscriptions: `dashboard` = heartbeat + vitality + daimon + tool + permit + cortical. `all` = every category. `vitals` = vitality + daimon + necrocracy. `activity` = heartbeat + tool + inference + grimoire + prediction. `cognitive` = prediction + cortical + clock + dream.

---

## 4. Wire format examples

All events serialize as JSON with `#[serde(tag = "type")]`. The `type` field is the dot-notation wire name from the mapping table.

### HeartbeatTick

```json
{
  "type": "heartbeat.tick",
  "timestamp": 1741968000000,
  "golem_id": "g-7f3a",
  "sequence": 4217,
  "tick": 3847,
  "tier": "T1",
  "pe": 0.42,
  "threshold": 0.34,
  "cost": 0.003
}
```

### SwapExecuted

```json
{
  "type": "trading.swap_executed",
  "timestamp": 1741968015000,
  "golem_id": "g-7f3a",
  "sequence": 4218,
  "tick": 3847,
  "pair": "ETH/USDC",
  "side": "sell",
  "amount_in": 1.5,
  "amount_out": 4725.0,
  "price": 3150.0,
  "slippage_bps": 12.0,
  "gas_usd": 2.30,
  "pnl_usd": 47.50,
  "tx_hash": "0xabc123...",
  "venue": "uniswap_v4"
}
```

### DeathInitiated

```json
{
  "type": "vitality.death_initiated",
  "timestamp": 1741990000000,
  "golem_id": "g-7f3a",
  "sequence": 9841,
  "tick": 8192,
  "cause": "economic_exhaustion",
  "vitality_at_death": 0.02,
  "ticks_lived": 8192
}
```

### AchievementUnlocked

```json
{
  "type": "engagement.achievement_unlocked",
  "timestamp": 1741968020000,
  "golem_id": "g-7f3a",
  "sequence": 4219,
  "tick": 3847,
  "achievement_id": "hot_hand",
  "name": "Hot Hand",
  "category": "performance",
  "rarity": "rare",
  "description": "10 consecutive profitable decisions"
}
```

---

## 5. Emitter-to-consumer matrix

| Subsystem | Emitter crate | Count | Consumers |
|---|---|---|---|
| Heartbeat | `golem-heartbeat` | 3 | TUI hearth, dashboard, Prometheus |
| Perception | `golem-heartbeat` | 1 | TUI hearth, structured logs |
| Tools | `golem-tools` | 3 | TUI tools/hearth, OTel spans, Prometheus |
| Risk | `golem-safety` | 4 | TUI safety/hearth, alerts, Prometheus |
| Inference | `golem-inference` | 5 | TUI hearth/inference, Prometheus, cost tracking |
| Dreams | `golem-dreams` | 5 | TUI dream pane, push notifications, Prometheus |
| Daimon | `golem-daimon` | 3 | TUI ghost pane, sprite engine, Prometheus |
| Mortality | `golem-mortality` | 5 | TUI mortality, alerts, push, sprite overlay, Prometheus |
| Grimoire | `golem-grimoire` | 7 | TUI knowledge/dream, Prometheus |
| Coordination | `golem-coordination` | 5 | TUI clade/knowledge, Prometheus |
| Context | `golem-context` | 3 | TUI hearth (debug), structured logs |
| Custody | `golem-chain` | 5 | TUI wallet/portfolio, alerts |
| Lifecycle | `golem-runtime` | 2 | Control plane, structured logs, TUI connection |
| Engagement | `golem-engagement` | 4 | TUI achievement/body, push notifications |
| Marketplace | `golem-coordination` | 5 | TUI marketplace, Prometheus |
| Trading | `golem-tools` | 5 | TUI portfolio/tools, Prometheus, sprite engine |
| CorticalState | `golem-cortex` | 2 | TUI hearth, dashboard, Prometheus |
| Prediction | `golem-prediction` | 4 | TUI predictions, Prometheus, residual corrector |
| Adaptive Clock | `golem-heartbeat` | 4 | TUI hearth, structured logs, Prometheus |
| Library | `golem-library` | 3 | TUI knowledge, Prometheus |
| Necrocracy | `golem-mortality` | 3 | TUI mortality, alerts, push, Styx |
| Portal | `golem-tui` | 3 | TUI portal, structured logs |
| System | `golem-runtime` | 2 | Structured logs, debug view |
| **Total** | | **87** | |

---

## 6. Proposed events (not yet in the enum)

These gaps were identified during cross-document review. They SHOULD be added before the runtime reaches beta.

### `ClientCommandAcknowledged`

Wire: `system.command_ack`. The server-side `CommandAck` struct exists (see [realtime subscriptions](../13-runtime/12-realtime-subscriptions.md) section 9), but it's a raw JSON frame, not a `GolemEvent`. Promoting it would let the Event Fabric record command acks in the ring buffer for complete replay.

Payload: `request_id: String`, `command_type: String`, `status: String` (accepted/rejected/queued), `reason: Option<String>`.

### `ToolVerified`

Wire: `tool.verified`. Post-flight on-chain verification that a transaction was included and state changed as expected. Currently embedded in `ToolExecutionEnd`. Splitting it out gives the TUI a distinct "verified" badge.

Payload: `tool_name`, `tx_hash`, `block_number: u64`, `state_change_confirmed: bool`, `verification_latency_ms: u64`.

### `HeartbeatSuppressed`

Wire: `heartbeat.suppressed`. Defined in `12-realtime-subscriptions.md` but absent from this catalog's research source. MUST be reconciled.

Payload: `reason: String` (all_clear, cost_cap, conservation_mode).

### `PolicyViolation`

Wire: `risk.policy_violation`. Hard shield or caveat enforcer rejection at a layer above the permit system. Referenced in early event lists but not formalized.

Payload: `rule: String`, `context: String`, `action_blocked: String`.

---

## 7. Reconciliation notes

This document is authoritative. Two older documents define partial GolemEvent enums:

- **`13-runtime/12-realtime-subscriptions.md`** section 1: 40-variant enum with shorter names (`ToolStart` vs. `ToolExecutionStart`). This catalog uses longer names because they're more grep-friendly.
- **`13-runtime/09-observability.md`** section 1: subsystem event counts that undercount (e.g., "Daimon: 1 event" vs. 3 here).

Key additions over the subscriptions enum: full Coordination subsystem (pheromone, bloodstain, clade sync), full Custody subsystem (delegation, session keys, spending limits), Marketplace (listings, purchases, reviews), Trading (swaps, LP, vault, liquidation), Engagement (achievements, sprite, milestones), Lifecycle (boot, shutdown), expanded Grimoire (7 variants vs. 4), expanded Daimon (3 vs. 1), expanded Mortality (5 vs. 4).

The older definitions SHOULD be updated to reference this catalog rather than duplicate the enum.

---

_End of document._

# Realtime subscriptions [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies the real-time event subscription system for a running Golem (a mortal autonomous DeFi agent): the full GolemEvent enum (50+ variants), how clients subscribe over WebSocket, reconnection replay from the ring buffer, bandwidth estimates, and sprite animation triggers. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the Event Fabric (tokio::broadcast channel carrying all Golem events) and the state model from `11-state-model.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

The Event Fabric subscription model. Clients subscribe to event categories (not individual types) over WebSocket. The Golem runtime emits typed `GolemEvent` variants in Rust (CamelCase); a bridge layer serializes them to JSON for external consumers. 10,000-event ring buffer for reconnection replay. 50+ event variants across 16 subsystems.

> **Crate**: `golem-core` (event_fabric.rs, events.rs), `bardo-terminal` (event consumer)
>
> **Cross-references:**
>
> - `./11-state-model.md` — GolemState (mutable internal state) and GolemSnapshot (read-only projection), the structures whose transitions produce events
> - `./09-observability.md` — Event Fabric architecture: how events flow to Prometheus metrics, structured logging, and OpenTelemetry traces
> - `./02-communication-channels.md` — surface multiplexer: how TUI, web portal, and social connectors consume the same event stream

---

## 1. GolemEvent enum

The complete event catalog. Rust CamelCase variants, serialized with `#[serde(tag = "type")]` for discriminated union dispatch.

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Shared types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeartbeatPhase {
    Observe, Retrieve, Analyze, Decide, Simulate, Validate, Execute, Verify, Reflect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tier { T0, T1, T2, T3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolResult { Success, Failure, Blocked }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DreamPhase { Nrem, Rem, Integration }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provenance { Own, Clade, Replicant, Marketplace, Inherited }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSummaryEntry {
    pub name: String,
    pub severity: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pad {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityClocks {
    pub economic: f64,
    pub epistemic: f64,
    pub stochastic: f64,
}

// ── Event enum ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GolemEvent {
    // ── Heartbeat ───────────────────────────────────────────────
    HeartbeatTick {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tick: u64,
        phase: HeartbeatPhase,
        tier: Tier,
        probe_summary: Vec<ProbeSummaryEntry>,
        cost: f64,
    },
    HeartbeatSuppressed {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tick: u64,
        reason: String,   // "all_clear" | "cost_cap" | "conservation_mode"
    },

    // ── Tool execution ──────────────────────────────────────────
    ToolStart {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tool_name: String,
        action_kind: Option<String>,
        permit_id: Option<String>,
    },
    ToolUpdate {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tool_name: String,
        progress: Option<f64>,  // 0.0-1.0
    },
    ToolEnd {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tool_name: String,
        tx_hash: Option<String>,
        result: Option<ToolResult>,
        block_reason: Option<String>,
    },

    // ── LLM inference ───────────────────────────────────────────
    LlmStart {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        model: String,
        tier: Tier,
    },
    LlmToken {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        model: String,
        tier: Tier,
        token_count: Option<u64>,  // cumulative
    },
    LlmEnd {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        model: String,
        tier: Tier,
        token_count: Option<u64>,
        cost: Option<f64>,
        decision: Option<String>,
    },

    // ── Model selection ─────────────────────────────────────────
    ModelSelected {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        model: String,
        provider: String,
        subsystem: String,      // "heartbeat" | "dream" | "reflection"
        reason: String,
    },

    // ── Inference token streaming ───────────────────────────────
    InferenceToken {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        content: String,         // token text
        done: bool,
        tokens_so_far: u64,
    },

    // ── Dream ───────────────────────────────────────────────────
    DreamStart {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        cycle_id: String,
    },
    DreamPhaseTransition {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        cycle_id: String,
        phase: Option<DreamPhase>,
    },
    DreamHypothesis {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        cycle_id: String,
        hypothesis: Option<String>,
        validated: Option<bool>,
    },
    DreamOutcome {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        cycle_id: String,
        quality_score: f64,
        insights_promoted: u32,
        playbook_revisions_staged: u32,
    },
    DreamEnd {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        cycle_id: String,
        heuristics_proposed: Option<u64>,
    },

    // ── Daimon (affect) ─────────────────────────────────────────
    DaimonAppraisal {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        pad: Pad,
        plutchik: String,
        mortality_emotion: Option<String>,
        intensity: f64,
    },

    // ── Vitality ────────────────────────────────────────────────
    VitalityUpdate {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        vitality: f64,
        phase: String,
        credit_balance: f64,
        projected_life_hours: f64,
        clocks: VitalityClocks,
    },

    // ── Mortality ───────────────────────────────────────────────
    MortalityWarning {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        clock: String,       // "economic" | "epistemic" | "stochastic"
        level: String,       // "notice" | "warning" | "critical"
        value: f64,
        threshold: f64,
    },
    DeathImminent {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        estimated_ticks_to_death: u64,
        primary_cause: String,
        composite: f64,
    },
    StochasticCheck {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        hazard_rate: f64,
        survived: bool,
        survival_probability: f64,
    },
    Senescence {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        stage: u8,
        fitness: f64,
        failing_domains: Vec<String>,
    },
    DeathProtocolStep {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        phase: String,            // "acceptance" | "settlement" | "reflection" | "legacy"
        previous_phase: String,
        estimated_remaining_ticks: u64,
        detail: String,
    },

    // ── Clade ───────────────────────────────────────────────────
    CladeSync {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        source_golem_id: Option<String>,
        entry_count: Option<u64>,
    },
    CladeAlert {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        source_golem_id: Option<String>,
        alert_kind: Option<String>,
    },
    CladeSiblingDeath {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        source_golem_id: Option<String>,
    },

    // ── Grimoire ────────────────────────────────────────────────
    GrimoireInsight {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        entry_id: String,
        confidence: f64,
        summary: String,
        provenance: Provenance,
    },
    GrimoireHeuristic {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        entry_id: String,
        confidence: f64,
        summary: String,
        provenance: Provenance,
    },
    GrimoireWarning {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        entry_id: String,
        confidence: f64,
        summary: String,
        provenance: Provenance,
    },
    GrimoireCausalLink {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        entry_id: String,
        confidence: f64,
        summary: String,
        provenance: Provenance,
    },

    // ── Action permits ──────────────────────────────────────────
    PermitCreated {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        permit_id: String,
        action_kind: String,
        risk_tier: String,
        simulation_hash: Option<String>,
    },
    PermitCommitted {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        permit_id: String,
        action_kind: String,
        risk_tier: String,
        tx_hash: Option<String>,
    },
    PermitExpired {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        permit_id: String,
        action_kind: String,
        risk_tier: String,
    },
    PermitCancelled {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        permit_id: String,
        action_kind: String,
        risk_tier: String,
    },
    PermitBlocked {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        permit_id: String,
        action_kind: String,
        risk_tier: String,
        block_reason: Option<String>,
    },

    // ── Context ─────────────────────────────────────────────────
    ContextAssembled {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        bundle_size: u64,
        category_sizes: HashMap<String, u64>,
        policy_revision: u64,
        steer_present: bool,
    },

    // ── Compaction ──────────────────────────────────────────────
    CompactionTriggered {
        timestamp: u64,
        golem_id: String,
        sequence: u64,
        tokens_before: u64,
        tokens_after: u64,
        invariants_preserved: Vec<String>,
    },
}
```

---

## 2. TypeScript to Rust mapping table

The bridge layer translates between TypeScript dot-notation event names (used on the JSON wire protocol for backward compatibility) and Rust CamelCase enum variants.

| TypeScript wire name | Rust variant | Category |
|---|---|---|
| `heartbeat.tick` | `HeartbeatTick` | heartbeat |
| `heartbeat.suppressed` | `HeartbeatSuppressed` | heartbeat |
| `tool.start` | `ToolStart` | tool |
| `tool.update` | `ToolUpdate` | tool |
| `tool.end` | `ToolEnd` | tool |
| `llm.start` | `LlmStart` | llm |
| `llm.token` | `LlmToken` | llm |
| `llm.end` | `LlmEnd` | llm |
| `model.selected` | `ModelSelected` | llm |
| `inference.token` | `InferenceToken` | llm |
| `dream.start` | `DreamStart` | dream |
| `dream.phase` | `DreamPhaseTransition` | dream |
| `dream.hypothesis` | `DreamHypothesis` | dream |
| `dream.outcome` | `DreamOutcome` | dream |
| `dream.end` | `DreamEnd` | dream |
| `daimon.appraisal` | `DaimonAppraisal` | daimon |
| `vitality.update` | `VitalityUpdate` | vitality |
| `mortality.warning` | `MortalityWarning` | mortality |
| `mortality.death_imminent` | `DeathImminent` | mortality |
| `mortality.stochastic_check` | `StochasticCheck` | mortality |
| `mortality.senescence` | `Senescence` | mortality |
| `death.protocol_step` | `DeathProtocolStep` | death |
| `clade.sync` | `CladeSync` | clade |
| `clade.alert` | `CladeAlert` | clade |
| `clade.sibling_death` | `CladeSiblingDeath` | clade |
| `grimoire.insight` | `GrimoireInsight` | grimoire |
| `grimoire.heuristic` | `GrimoireHeuristic` | grimoire |
| `grimoire.warning` | `GrimoireWarning` | grimoire |
| `grimoire.causal_link` | `GrimoireCausalLink` | grimoire |
| `permit.created` | `PermitCreated` | permit |
| `permit.committed` | `PermitCommitted` | permit |
| `permit.expired` | `PermitExpired` | permit |
| `permit.cancelled` | `PermitCancelled` | permit |
| `permit.blocked` | `PermitBlocked` | permit |
| `context.assembled` | `ContextAssembled` | context |
| `compaction.triggered` | `CompactionTriggered` | compaction |

The bridge serializes with `#[serde(rename = "heartbeat.tick")]` annotations on each variant for JSON compatibility, while the internal Rust code uses native CamelCase.

---

## 3. Subscription model

Clients subscribe to **categories**, not individual event types. This keeps the protocol simple and avoids per-event subscription overhead.

### Categories

| Category | Events included | Typical subscriber |
|---|---|---|
| `heartbeat` | HeartbeatTick, HeartbeatSuppressed | TUI main screen, dashboard |
| `tool` | ToolStart, ToolUpdate, ToolEnd | TUI pipeline view |
| `llm` | LlmStart, LlmToken, LlmEnd, ModelSelected, InferenceToken | TUI reasoning view |
| `dream` | DreamStart, DreamPhaseTransition, DreamHypothesis, DreamOutcome, DreamEnd | TUI dream pane |
| `daimon` | DaimonAppraisal | TUI feel pane |
| `vitality` | VitalityUpdate | Status bar, health gauge |
| `mortality` | MortalityWarning, DeathImminent, StochasticCheck, Senescence | Alert system |
| `death` | DeathProtocolStep | Death protocol UI |
| `clade` | CladeSync, CladeAlert, CladeSiblingDeath | Clade panel |
| `grimoire` | GrimoireInsight, GrimoireHeuristic, GrimoireWarning, GrimoireCausalLink | Knowledge log |
| `permit` | PermitCreated, PermitCommitted, PermitExpired, PermitCancelled, PermitBlocked | Action permit UI |
| `context` | ContextAssembled | Debug view |
| `compaction` | CompactionTriggered | Debug view |

### Subscribe message

```json
{
  "type": "subscribe",
  "categories": ["heartbeat", "tool", "llm", "dream", "daimon", "vitality"],
  "auth": "Bearer <jwt>"
}
```

Clients can update subscriptions at any time. For example, mute `llm` when not viewing the reasoning pane (reduces bandwidth during active inference).

### Composite subscriptions

| Composite | Includes | Use case |
|---|---|---|
| `dashboard` | heartbeat, vitality, daimon, tool, permit | Main dashboard |
| `all` | Every category | Debug / development |
| `vitals` | vitality, daimon, mortality | Health monitoring |
| `activity` | heartbeat, tool, llm, grimoire | Activity feed |

---

## 4. Connection lifecycle

### WebSocket endpoint

```
ws://localhost:8402/events          # Local self-hosted
wss://g-7f3a.bardo.run/events      # Remote hosted (TLS)
wss://styx.bardo.run/v1/relay/events?golem=<id>  # Through Styx (NAT traversal)
```

### Handshake

1. Client connects with auth token
2. Server validates auth, determines visibility tier (public/owner/internal)
3. Server sends `connection.established` with available categories
4. Client sends `subscribe` with desired categories
5. Server sends full `GolemSnapshot` for subscribed components
6. Server begins streaming events

### Keepalive

Server sends `ping` every 30 seconds. Client must respond with `pong` within 10 seconds. Two missed pongs trigger disconnect.

---

## 5. Reconnection protocol

### Automatic reconnection

1. On disconnect, client waits 1s, then reconnects with exponential backoff (1s, 2s, 4s, max 30s)
2. On reconnect, client sends:

```json
{
  "type": "resync",
  "lastSequence": 4217,
  "auth": "Bearer <jwt>"
}
```

3. Server evaluates the gap:
   - **Gap <= 10,000 events** (ring buffer): replay missed events in order, then resume live
   - **Gap > 10,000 events**: send fresh `GolemSnapshot`, then resume live

4. Server sends `reconnect.replay_start` before replay and `reconnect.replay_end` after

### Ring buffer

```rust
pub struct EventRingBuffer {
    events: VecDeque<GolemEvent>,
    capacity: usize,  // 10,000
}

impl EventRingBuffer {
    pub fn push(&mut self, event: GolemEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn replay_from(&self, last_sequence: u64) -> Vec<&GolemEvent> {
        self.events.iter()
            .filter(|e| e.sequence() > last_sequence)
            .collect()
    }
}
```

---

## 6. Sprite animation triggers

The TUI's sprite engine subscribes to the event stream and maps events to animation states:

```rust
impl SpriteEngine {
    pub fn handle_event(&mut self, event: &GolemEvent) {
        match event {
            GolemEvent::HeartbeatTick { tier, .. } => match tier {
                Tier::T0 => self.set_animation(Animation::Idle),
                Tier::T1 => self.set_animation(Animation::Thinking),
                Tier::T2 => {
                    self.set_animation(Animation::Thinking);
                    self.set_intensity(1.5);
                }
                _ => {}
            },
            GolemEvent::ToolStart { action_kind, .. } => {
                if action_kind.is_some() {
                    self.set_animation(Animation::Executing);
                }
            }
            GolemEvent::ToolEnd { result, .. } => match result {
                Some(ToolResult::Success) => self.play_once(Animation::Success),
                Some(ToolResult::Failure) | Some(ToolResult::Blocked) => {
                    self.play_once(Animation::Failure);
                }
                None => {}
            },
            GolemEvent::DaimonAppraisal { pad, mortality_emotion, .. } => {
                self.update_expression(pad);
                if let Some(emotion) = mortality_emotion {
                    self.set_mortality_overlay(emotion);
                }
            }
            GolemEvent::DreamStart { .. } => self.set_animation(Animation::Dreaming),
            GolemEvent::DreamEnd { .. } => self.play_once(Animation::Waking),
            GolemEvent::VitalityUpdate { phase, .. } => self.set_phase_overlay(phase),
            GolemEvent::GrimoireInsight { .. } => {
                self.emit_particles(ParticleEffect::KnowledgePulse);
            }
            GolemEvent::CladeSync { .. } => self.play_once(Animation::CladePulse),
            _ => {}
        }
    }
}
```

---

## 7. Bandwidth estimates

| Event category | Frequency (normal) | Frequency (volatile) | Avg size |
|---|---|---|---|
| `heartbeat` | 1/min | 2-4/min | 500B |
| `tool` | 0-3/tick | 2-8/tick | 200B |
| `llm` | 0-1/tick | 1-3/tick | 300B |
| `daimon` | 1/tick | 1/tick | 150B |
| `vitality` | 1/tick | 1/tick | 200B |
| `grimoire` | 0-1/tick | 0-2/tick | 300B |
| `permit` | 0-1/tick | 1-3/tick | 250B |

**Normal market**: ~5 events/min, ~2.5 KB/min, ~150 KB/hour.
**Volatile market**: ~30 events/min, ~9 KB/min, ~540 KB/hour.

Negligible bandwidth. Even metered connections see <1 MB/hour.

---

## 8. Backpressure

If the WebSocket send buffer exceeds thresholds:

| Buffer size | Action |
|---|---|
| 1 MB | Drop `HeartbeatSuppressed` events (lowest priority) |
| 2 MB | Drop `HeartbeatTick` events (keep only phase changes) |
| 4 MB | Send `backpressure.warning`, pause non-critical categories |

```rust
#[derive(Serialize)]
pub struct BackpressureWarning {
    pub buffer_size_bytes: u64,
    pub paused_categories: Vec<String>,
    pub recommendation: String,  // "reduce_subscriptions" | "process_faster"
}
```

---

## 9. Client commands

Commands flow from clients to the server over the same WebSocket connection.

```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ClientCommand {
    Steer {
        request_id: String,
        instruction: String,
        priority: String,  // "normal" | "urgent"
    },
    FollowUp {
        request_id: String,
        message: String,
    },
    Halt {
        request_id: String,
        reason: String,
    },
    Resume {
        request_id: String,
    },
    Snapshot {
        request_id: String,
        components: Vec<String>,
    },
    Wake {
        request_id: String,
        reason: String,
    },
    Confirm {
        request_id: String,
        permit_id: String,
        confirmed: bool,
    },
}
```

Server acknowledges all commands:

```rust
#[derive(Serialize)]
pub struct CommandAck {
    pub request_id: String,
    pub status: String,  // "accepted" | "rejected" | "queued"
    pub reason: Option<String>,
}
```

---

## 10. SSE fallback

For environments where WebSocket is unavailable, a subset of events is available via Server-Sent Events:

```
GET /api/v1/events?categories=vitality,mortality,death
Accept: text/event-stream
Authorization: Bearer <privy-jwt>
```

SSE limitations:
- Read-only (no client commands)
- No delta encoding (full event payload each time)
- No backpressure management
- Event ID corresponds to `sequence` for reconnection via `Last-Event-ID`
- Maximum 5 categories per connection

---

## 11. Surface-specific rendering

Different surfaces consume the same event stream but render differently:

| Surface | Transport | Format | Rendering |
|---|---|---|---|
| TUI (Ratatui) | Unix socket or WebSocket | NDJSON, 60fps target | Full sprite animation, multi-pane layout |
| Web dashboard | WebSocket `:8402/events` | JSON frames | React components, charts, status badges |
| Telegram | HTTP polling via Bott gateway | Batched text, 1 edit/sec rate limit | Condensed summaries, inline keyboard |
| Discord | HTTP via Bott gateway | Rich embeds, color-coded sidebars | Embed fields, bot presence status |
| API consumer | SSE | Standard SSE format | Raw event data |

The TUI is the highest-fidelity consumer. Every event maps to a visual: sprite animation, progress bar, gauge update, or particle effect. The other surfaces degrade gracefully, batching and summarizing where the medium demands it.

---

> **Design system binding:** Event-to-visual trigger mappings are documented in the design system §24 (Data-to-Visual Binding), §7 (Animation Library), and the sprite runtime spec (v3.1 §1.3 Variable Channels). See `tmp/research/design-inspo/bardo-design-system-v2.md`.

---

_End of document._

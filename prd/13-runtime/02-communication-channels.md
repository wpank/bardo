# Communication channels [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies how a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) communicates with its owner and the outside world across three surfaces: TUI, web portal, and social connectors (Telegram, Discord, Twitter/X, Farcaster). It sits in the **Runtime** layer of the Bardo specification. The main prerequisite is the Event Fabric, the internal event bus that all surfaces consume from. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Three communication surfaces -- TUI (primary), web portal (secondary), and social connectors (tertiary) -- all fed by the Event Fabric. Per-channel message formats, frequencies, formatting, and delivery guarantees.

> **Cross-references:**
>
> - `./12-realtime-subscriptions.md` — full GolemEvent enum (50+ variants), WebSocket subscription protocol, delta encoding, reconnection semantics, and bandwidth estimates
> - `./11-state-model.md` — defines GolemState (mutable internal state) and GolemSnapshot (read-only projection for UI consumption), with all component interfaces
> - `./00-interaction-model.md` — user-Golem interaction model: TUI architecture, authentication, Heartbeat FSM, operator intervention (steer/follow_up), and the 29-screen navigation hierarchy
> - `../05-dreams/01-architecture.md` — dream scheduling architecture: the three-phase NREM/REM/integration cycle and knowledge consolidation
> - `../03-daimon/07-runtime-daimon.md` — runtime integration of the Daimon affect engine: mood event emission, PAD vector updates, and emotional contagion across Clades
> - `../02-mortality/12-integration.md` — mortality event integration: death protocol steps, phase transition events, and clock threshold warnings

---

## 1. Three surfaces

| Surface | Modality | Primary use | Audience |
| --- | --- | --- | --- |
| **TUI** (bardo-terminal) | Persistent, 60fps | Live Golem management, creature visualization, knowledge browsing | Owners, power users |
| **Web Portal** | Browser | Setup wizard, reputation profile, vault analytics, knowledge browser | All users |
| **Social connectors** | Push notifications | Alerts, daily summaries, death warnings, status checks | All users |

The TUI is the primary surface. It is a standalone Rust binary (`bardo-terminal`) that renders animated sprites, real-time event streams, and multi-pane dashboards at 60fps via ratatui. The web portal handles setup, analytics, and scenarios where a browser is more appropriate. Social connectors are notification channels -- they push alerts, they don't host conversations.

---

## 2. Event Fabric (internal bus)

All surfaces consume from the same source: the Event Fabric, a `tokio::broadcast` channel inside the Golem runtime with a ring buffer of 10,000 entries. Every internal state transition emits a typed `GolemEvent` (CamelCase Rust enum variants). Surfaces subscribe to the Event Fabric over WebSocket and receive events as tagged JSON.

```rust
/// The Event Fabric is the single source of truth for all Golem events.
/// Surfaces connect via WebSocket, specify topic filters, and receive
/// a filtered stream of GolemEvent variants.
pub struct EventFabric {
    tx: tokio::sync::broadcast::Sender<GolemEvent>,
    ring_buffer: RingBuffer<GolemEvent>,
}

impl EventFabric {
    pub fn emit(&self, event: GolemEvent) {
        // Only serialize if subscribers exist
        if self.tx.receiver_count() > 0 {
            let _ = self.tx.send(event.clone());
        }
        self.ring_buffer.push(event);
    }

    pub fn subscribe(&self, resume_from: Option<u64>) -> EventStream {
        let rx = self.tx.subscribe();
        let replay = resume_from
            .map(|seq| self.ring_buffer.since(seq))
            .unwrap_or_default();
        EventStream { replay, rx }
    }
}
```

Events use CamelCase variant names in Rust (`HeartbeatTick`, `SwapExecuted`, `DreamInsight`) and serialize with `#[serde(tag = "type")]` for discriminated-union dispatch over the wire. Field names use `snake_case`. Timestamps are Unix milliseconds.

---

## 3. TUI transport (primary)

### 3.1 Architecture

The TUI connects to the Golem VM's WebSocket endpoint. A second WebSocket connects to Styx (the global knowledge relay and persistence layer at wss://styx.bardo.run) for ecosystem-wide events (pheromone field, Bloodstains (market conditions recorded at another Golem's death, broadcast as warnings), other Golems).

```
+---------------------------+       WebSocket        +------------------+
|   bardo-terminal          |<--- GolemEvents -------|  Golem VM        |
|   (Rust / ratatui)        |---- commands --------->|  (Fly.io)        |
|                           |      :8080             |                  |
|   60fps render loop       |                        |  Event Fabric    |
|   Sprite engine           |       WebSocket        |  Heartbeat FSM   |
|   Particle system         |<--- Styx events -------|                  |
|   Custom widgets          |      :8080/ws          +------------------+
|   Sound engine (rodio)    |                        |  Styx Service    |
|   Input handling          |                        |  (Fly.io)        |
+---------------------------+                        +------------------+
```

The render loop runs at 60fps independently of event arrival. Events update shared state; the renderer reads state each frame. No frame waits on a network call. Input polling, WebSocket drain, animation tick, and terminal flush all fit within the 16.6ms frame budget.

### 3.2 Event delivery

The TUI subscribes to all Event Fabric topics and receives every `GolemEvent` variant. Events drive screen updates:

- `HeartbeatTick` updates the Beat screen pipeline visualization
- `SwapExecuted` triggers a trade animation and updates the Vault screen
- `DreamInsight` appends to the Dreams screen hypothesis tracker
- `VitalityPhaseChange` triggers sprite animation transitions
- `PheromoneDeposit` updates the Clade (cooperative group of Golems sharing knowledge via Styx) screen heatmap

### 3.3 Owner commands

The TUI sends typed commands over the same WebSocket:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OwnerCommand {
    Steer { message: String },
    FollowUp { message: String },
    PauseHeartbeat,
    ResumeHeartbeat,
    KillSwitch,
    WakeFromDream,
    SpawnReplicant { config: ReplicantConfig },
}
```

### 3.4 Auth flow

The TUI uses browser handoff for authentication, mirroring the Claude Code / Codex pattern:

1. Terminal generates session ID and opens `bardo.run/auth?session=xxx` in the browser
2. User authenticates in browser (wallet signature, email, or OAuth)
3. Terminal polls `/auth/status?session=xxx` every 2 seconds
4. On success: session token + wallet address returned
5. Stored in `~/.bardo/auth.json` (0o600 permissions)

While waiting, the terminal shows a copyable URL, a QR code, and keyboard shortcuts. For headless SSH: device code flow (RFC 8628) where the terminal shows a code like `BARD-7F3A` and the user visits `bardo.run/device` to enter it.

---

## 4. Web portal transport (secondary)

### 4.1 WebSocket protocol

Primary transport for the portal's interactive features.

**Connection:**

```
wss://<machine-name>.bardo.money/ws
Authorization: Bearer <session-token>
```

**Message types:**

```rust
/// Client -> Server
#[derive(Debug, Serialize, Deserialize)]
pub struct WsClientMessage {
    pub channel: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub request_id: Option<String>,
}

/// Server -> Client
#[derive(Debug, Serialize, Deserialize)]
pub struct WsServerMessage {
    pub channel: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub request_id: Option<String>,
    pub timestamp: u64,
}
```

### 4.2 Channel multiplexing

A single WebSocket carries multiple logical channels:

| Channel    | Direction       | Purpose                                                  |
| ---------- | --------------- | -------------------------------------------------------- |
| `system`   | Bidirectional   | Connection lifecycle, subscribe/unsubscribe, ping/pong   |
| `events`   | Server -> Client | Real-time GolemEvent stream                              |
| `logs`     | Server -> Client | Live structured log stream (filterable by level)         |
| `terminal` | Bidirectional   | Conversation (steer/followUp/response)                   |
| `strategy` | Bidirectional   | Strategy parameter updates, trigger rebalance            |
| `grimoire` | Server -> Client | Live Grimoire (the Golem's persistent knowledge base of episodes, insights, heuristics, and causal links) updates (new insights, confidence changes) |
| `dreams`   | Server -> Client | Dream cycle progress, insights, mode transitions         |

### 4.3 Full server event catalog

All events emitted by the Golem, organized by subsystem. Events use CamelCase Rust enum variants serialized as tagged JSON.

**Heartbeat events** (topic: `heartbeat`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `MarketObservation` | Step 1 complete | Owner |
| `HeartbeatTick` | Gating decision | Owner |
| `HeartbeatComplete` | Full cycle done | Owner |
| `TierSelected` | Step 4 | Owner |

**Vitality events** (topic: `vitality`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `VitalityUpdate` | Every tick | Public (partial) |
| `VitalityPhaseChange` | Phase transition | Public |

**Mood events** (topic: `mood`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `MoodUpdate` | PAD delta > 0.15 | Public (label), Owner (PAD) |
| `MoodAppraisal` | Appraisal fires | Internal |
| `MoodContagion` | Clade PAD sync | Internal |

**Dream events** (topic: `dream`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `DreamStart` | Cycle begins | Owner |
| `DreamProgress` | Phase transition | Owner |
| `DreamComplete` | Cycle ends | Owner |
| `DreamInsight` | Notable hypothesis | Owner |
| `DreamWakeInterrupt` | Wake command | Owner |

**Mortality events** (topic: `death`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `MortalityWarning` | Clock threshold | Owner |
| `DeathImminent` | Composite < 0.15 | All channels |
| `StochasticCheck` | Every tick | Internal |
| `SenescenceStage` | Stage change | Owner |
| `DeathProtocolStep` | Protocol phase | Owner |

**Tool events** (topic: `tools`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `ToolExecutionStart` | Tool invoked | Owner |
| `ToolExecutionUpdate` | Progress step | Owner |
| `ToolExecutionEnd` | Tool complete | Owner |
| `PermitCreated` | Capability issued | Owner |
| `PermitCommitted` | Capability consumed | Owner |

**Trade events** (topic: `performance`):

| Event | Trigger | Visibility |
| --- | --- | --- |
| `SwapExecuted` | Trade completes | Owner |
| `VaultNavUpdate` | NAV changes | Public |

### 4.4 Reconnection

| Parameter               | Value                            |
| ----------------------- | -------------------------------- |
| Initial reconnect delay | 3 seconds                        |
| Backoff multiplier      | 2x                               |
| Max reconnect delay     | 30 seconds                       |
| Max reconnect attempts  | Unlimited (persistent)           |
| Resume-from support     | Yes (server replays from buffer) |
| Event buffer size       | 10,000 events                    |

On reconnect, the client sends the last seen sequence number. The server replays missed events from its ring buffer.

---

## 5. Social connectors (tertiary)

Social platform connectors bridge platform I/O with the Golem's event stream. Each connector translates inbound platform messages into typed RPC calls and routes outbound Golem events to platform notifications. The `bardo-social` crate provides the connector implementations.

### 5.1 Supported platforms

**Telegram** (launch platform):

| Property        | Value                                                    |
| --------------- | -------------------------------------------------------- |
| Transport       | Telegram Bot API (long polling or webhook)               |
| Rate limits     | 30 messages/sec per chat                                 |
| Character limit | 4,096 per message                                        |
| Rich content    | Inline keyboards, Markdown formatting, image attachments |
| Auth model      | Telegram user ID mapped to wallet address                |

Telegram is the primary social connector. Bot API provides reliable delivery, inline keyboards for interactive confirmations, and Markdown formatting for portfolio summaries. Group chat support enables shared monitoring of Clade activity.

**Discord** (phase 2):

| Property        | Value                                      |
| --------------- | ------------------------------------------ |
| Transport       | Discord Gateway (WebSocket)                |
| Rate limits     | Platform-standard                          |
| Character limit | 2,000 per message, 4,096 per embed        |
| Rich content    | Embeds, buttons, slash commands            |
| Auth model      | Discord user ID mapped to wallet address   |

Slash commands (`/bardo status`, `/bardo portfolio`, `/bardo ask`) registered as global application commands. Rich embeds with color-coded phase indicators.

**Twitter/X** (phase 2):

| Property        | Value                                     |
| --------------- | ----------------------------------------- |
| Transport       | Twitter API v2                            |
| Rate limits     | 300 posts/3h (free tier), higher on paid  |
| Character limit | 280 per post                              |
| Rich content    | Threads, polls, card links                |
| Auth model      | Twitter user ID mapped to wallet address  |

Outbound only by default. Public Golems can post performance updates and market observations. Truncation at 270 chars (leave room for mentions).

**Farcaster** (phase 2):

| Property        | Value                                    |
| --------------- | ---------------------------------------- |
| Transport       | Farcaster Hub API                        |
| Rate limits     | Protocol-level (no hard per-user limit)  |
| Character limit | 320 per cast                             |
| Rich content    | Frames, embeds                           |
| Auth model      | Farcaster FID mapped to wallet address   |

Native to crypto communities. Cast-based interaction model. Frames can embed interactive vault dashboards.

### 5.2 Notification events

The social connector emits notifications for lifecycle events. All notifications are outbound only (Golem -> user). The Golem never receives strategy instructions through notification channels.

| Event                  | Trigger                                 | Priority |
| ---------------------- | --------------------------------------- | -------- |
| `StrategyDeployed`     | Golem boots and begins executing        | High     |
| `PnlMilestone`         | P&L crosses configurable threshold      | Medium   |
| `StopLossTriggered`    | Stop-loss hit, position closed          | High     |
| `GolemDying`           | Credits < 5%, death preparation begins  | Critical |
| `CoordinationUpdate`   | ERC-8001/8033/8183 intent status change | Medium   |
| `ReputationChange`     | ERC-8004 (the on-chain agent identity standard) tier change (up or down) | Medium   |
| `RegimeShift`          | Market regime change detected           | Medium   |

### 5.3 Platform-specific formatting

Each notification is formatted for the target platform:

```rust
pub fn format_for_platform(
    message: &str,
    platform: &str,
) -> FormattedMessage {
    match platform {
        "twitter" => FormattedMessage {
            text: truncate(message, 270),
            platform: platform.into(),
            truncated: message.len() > 270,
            embeds: vec![],
        },
        "telegram" => FormattedMessage {
            text: format_markdown(message),
            platform: platform.into(),
            truncated: false,
            embeds: vec![MessageEmbed::InlineKeyboard(status_keyboard())],
        },
        "discord" => FormattedMessage {
            text: String::new(),
            platform: platform.into(),
            truncated: false,
            embeds: vec![build_discord_embed(message)],
        },
        "farcaster" => FormattedMessage {
            text: truncate(message, 310),
            platform: platform.into(),
            truncated: message.len() > 310,
            embeds: vec![],
        },
        _ => unreachable!(),
    }
}
```

### 5.4 Rate limiting

| Channel                | Inbound limit      | Outbound limit                |
| ---------------------- | ------------------ | ----------------------------- |
| Social commands        | 1/second per user  | --                            |
| Deployment requests    | 5/hour per user    | --                            |
| Status queries         | 10/minute per user | --                            |
| Platform notifications | --                 | Platform-specific (see above) |
| Clade broadcasts       | --                 | 1/minute per Clade            |

### 5.5 Cross-platform notification routing

Users who interact on multiple platforms configure which platforms receive which notification types:

| Notification            | Default platform          | Configurable |
| ----------------------- | ------------------------- | ------------ |
| Deployment confirmation | Originating platform      | No           |
| P&L milestones          | All connected platforms   | Yes          |
| Stop-loss alerts        | All connected platforms   | No (always)  |
| Credit warnings         | All connected platforms   | No (always)  |
| Status updates          | Originating platform only | Yes          |

---

## 6. Heartbeat delivery

The heartbeat tick is the Golem's fundamental lifecycle signal.

### 6.1 Event emission

Every tick emits a `HeartbeatTick` event via the Event Fabric. The TUI receives all ticks. The web portal receives all ticks over WebSocket. Social connectors receive only phase changes and daily summaries.

### 6.2 Regime and phase change events

Special events emitted on transitions:

```rust
/// Emitted when the behavioral phase transitions.
GolemEvent::VitalityPhaseChange {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    from_phase: String,
    to_phase: String,
    survival_pressure: f64,
    projected_life_hours: f64,
    trigger: String,
}

/// Emitted when the market regime shifts.
GolemEvent::RegimeChange {
    timestamp: u64,
    golem_id: String,
    sequence: u64,
    tick: u64,
    from_regime: String,
    to_regime: String,
    price_delta: f64,
    volume_spike: f64,
    iv_change: f64,
    new_interval_ms: u64,
}
```

### 6.3 Per-surface delivery config

| Event type       | TUI      | Web portal | Telegram | Discord | Webhook   |
| ---------------- | -------- | ---------- | -------- | ------- | --------- |
| Every tick       | Yes      | Yes        | No       | No      | Optional  |
| Trade executed   | Yes      | Yes        | Opt-in   | Opt-in  | Yes       |
| Phase change     | Yes      | Yes        | Always   | Always  | Yes       |
| Daily summary    | N/A      | Yes        | Yes      | Yes     | Optional  |

---

## 7. Webhook delivery

### 7.1 Registration

```rust
pub struct WebhookConfig {
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub enabled: bool,
}
```

### 7.2 Signature verification

```
X-Bardo-Signature: sha256=a1b2c3d4e5f6...
X-Bardo-Delivery-Id: dlv_abc123
X-Bardo-Event: SwapExecuted
X-Bardo-Timestamp: 1709942400000
Content-Type: application/json
```

HMAC-SHA256 over `{timestamp}.{json_payload}`.

### 7.3 Retry policy

| Attempt | Delay           |
| ------- | --------------- |
| 1       | Immediate       |
| 2       | 30 seconds      |
| 3       | 2 minutes       |
| 4       | 10 minutes      |
| 5       | 1 hour          |
| 6       | 6 hours (final) |

After 6 failures: webhook auto-disabled. Re-enable via `PATCH /api/v1/webhooks/:id`.

---

## 8. Delivery guarantees

| Surface   | Guarantee                   | Ordering                | Durability                     |
| --------- | --------------------------- | ----------------------- | ------------------------------ |
| TUI (WS)  | At-most-once                | Ordered per channel     | Ring buffer (10,000 events)    |
| Portal WS | At-most-once                | Ordered per channel     | Ring buffer (10,000 events)    |
| Portal SSE| At-least-once (with replay) | Ordered by sequence     | Ring buffer (10,000 events)    |
| Telegram  | At-least-once               | Ordered per chat        | Telegram server queue          |
| Discord   | At-most-once                | Ordered per channel     | Discord gateway                |
| Webhook   | At-least-once (with retry)  | Ordered per delivery    | SQLite dead letter (72h)       |

---

_End of document._

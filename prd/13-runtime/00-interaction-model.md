# User-Golem interaction model [SPEC]

**Version:** 4.0.0
**Last updated:** 2026-03-15
**Status:** Implementation specification

---

> **Reader orientation:** This document defines how users interact with their Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) across all product surfaces: TUI, web portal, social connectors, and REST API. It sits in the **Runtime** layer of the Bardo specification, which covers the live operational behavior of a running Golem. Key prerequisites include the Heartbeat FSM (the Golem's core decision loop), the Daimon affect engine, and the three-mode custody model. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Cross-references

- `./11-state-model.md` — defines GolemState (the Golem's mutable internal state) and GolemSnapshot (the read-only projection surfaces consume for rendering)
- `./12-realtime-subscriptions.md` — specifies the full GolemEvent enum, WebSocket subscription protocol, reconnection semantics, and bandwidth estimates
- `./07-onboarding.md` — end-to-end onboarding flow from surface routing through authentication, custody selection, funding, strategy configuration, and first heartbeat
- `../18-interfaces/03-tui.md` — full `bardo-terminal` specification including all 29 screens, widget catalog, sprite engine, and rendering pipeline
- `../01-golem/02-heartbeat.md` — canonical Heartbeat FSM definition: the 9-step observe-to-reflect decision cycle that drives every Golem tick
- `../12-inference/01-routing.md` — inference routing across cognitive tiers (T0 rule-based through T3 extended reasoning) and payment via x402
- `../05-dreams/01-architecture.md` — dream scheduling, the three-phase NREM/REM/integration cycle, and knowledge consolidation
- `../03-daimon/01-appraisal.md` — the Daimon affect engine: PAD (Pleasure-Arousal-Dominance) vector computation, appraisal triggers, and mortality-specific emotions
- `../02-mortality/01-architecture.md` — the three mortality clocks (economic, epistemic, stochastic), Vitality score, and BehavioralPhase transitions
- `../04-memory/01-grimoire.md` — the Grimoire persistent knowledge base: episodes, insights, heuristics, causal links, and the Curator maintenance cycle
- `../10-safety/01-custody.md` — three-mode custody selection (Delegation, Embedded, LocalKey) and PolicyCage on-chain constraint enforcement

---

## 1. Interaction surfaces

Three surfaces, ranked by fidelity. The TUI is the recommended experience and the primary product surface.

| Surface | Modality | Primary use | Audience |
| --- | --- | --- | --- |
| **TUI** (`bardo-terminal`) | Persistent, 60fps, full-duplex | Live Golem management, the "living world" experience | Owners, power users |
| **Web portal** | Browser dashboard | Setup wizard, vault analytics, reputation profile, occasional check-ins | All users |
| **Social connectors** | Telegram, Discord, Farcaster | Push notifications, read-only status queries | All users |

The TUI is not a dashboard. It is a window into machine cognition. Most agent frameworks produce a number on a screen. `bardo-terminal` produces a creature you watch think, feel, dream, and die. That difference — visibility of internal cognitive state creating affective bonds (Reeves & Nass, 1996) — is what drives retention patterns no web dashboard replicates.

All three surfaces consume the same GolemEvent (the typed event enum emitted by every state change inside the Golem) stream from the Golem's HTTP interface. The Golem does not know or care which surface is connected.

---

## 2. TUI: `bardo-terminal`

The TUI is the primary interaction surface. Full specification in `../18-interfaces/03-tui.md`. This section covers the integration points: authentication, custody, event consumption, and how the TUI connects to the Golem's runtime.

### 2.1 Architecture

`bardo-terminal` is a standalone Rust binary. It connects to the Golem via WebSocket and renders at 60fps using ratatui. The render loop never waits on a network call.

```
┌──────────────────────────┐        WebSocket        ┌──────────────────┐
│   bardo-terminal         │◄─── GolemEvents ─────────│ Golem VM         │
│   (Rust / ratatui)       │──── steers/commands ────►│ (Fly.io)         │
│                          │       :8080               │                  │
│   60fps render loop      │                           │ Event Fabric     │
│   Sprite engine          │        WebSocket          │ Heartbeat FSM    │
│   Particle system        │◄─── Styx events ─────────│                  │
│   Custom widgets         │       :8080/ws            ├──────────────────┤
│   Sound engine (rodio)   │                           │ Styx Service     │
│   Input handling         │                           │ (Fly.io)         │
└──────────────────────────┘                           └──────────────────┘
```

**Process model**: `bardo-terminal` is a client, not a host. The Golem runtime is a separate process (Golem-RS, Rust) running on Fly.io or locally. The TUI connects to the Golem's exposed WebSocket endpoint. For self-hosted Golems:

```bash
# Terminal 1: Run the Golem (Rust binary from bardo-golem-rs)
bardo-golem start --config bardo.toml

# Terminal 2 (or tmux pane): Connect the TUI
bardo-terminal --golem g-7f3a
```

### 2.2 Installation

```bash
cargo install bardo-terminal

# Or via npx (downloads prebuilt binary, caches in ~/.bardo/bin/)
npx @bardo/terminal

# Or release binary
curl -sSL https://get.bardo.run | sh
```

Prebuilt binaries for x86_64 and aarch64 Linux, macOS Intel and Apple Silicon, Windows.

### 2.3 Screen navigation

The TUI uses a 29-screen, 6-window hierarchical navigation system. See `18-interfaces/screens/03-interaction-hierarchy.md` for the canonical interaction model.

---

## 3. Authentication flow

### 3.1 Browser handoff

The TUI mirrors the Claude Code / Codex authentication pattern. The terminal opens a browser tab; the user authenticates there; the terminal detects completion via polling.

```
bardo-terminal launches
  → checks ~/.bardo/config.toml
  → if absent: begin onboarding
  → generate session ID
  → open https://bardo.run/auth?session=xxx in default browser
  → show copyable URL + QR code in terminal
  → "[c] Copy URL  [q] Cancel"
  → poll https://bardo.run/auth/status?session=xxx every 2 seconds
  → on success: JWT + wallet address + wallet ID returned
  → store in ~/.bardo/auth.json (0o600 permissions)
  → proceed to custody selection
```

After initial auth, all subsequent sessions are TUI-native. The browser is only needed again when the refresh token expires (30-day lifetime) or auth is explicitly revoked.

### 3.2 Headless fallback (SSH)

For SSH sessions without browser access, three options:

1. Copy the auth URL manually and open on another device
2. Device code flow: terminal displays `BARD-7F3A`, user visits `bardo.run/device`, enters code
3. API key mode: user provides key from `bardo.run/settings/api-keys`, bypasses Privy entirely

### 3.3 Token storage

```rust
// ~/.bardo/auth.json (0o600 permissions)
#[derive(Serialize, Deserialize)]
pub struct AuthStorage {
    pub privy_jwt: String,
    pub refresh_token: String,
    pub expires_at: u64,          // Unix epoch ms
    pub user_id: String,
    pub wallet_address: String,
    pub wallet_id: String,
    pub auth_method: AuthMethod,  // Email | Google | GitHub | Wallet | ApiKey
}

pub enum AuthMethod { Email, Google, GitHub, Wallet, ApiKey }
```

Silent refresh on subsequent launches. Re-auth only when refresh token expires.

---

## 4. Three-mode custody selection

During onboarding, the user selects how the Golem manages capital. See `../10-safety/01-custody.md` for full specification.

| Mode | Description | Use case |
| --- | --- | --- |
| **Delegation** | MetaMask or hardware wallet via ERC-7710/7715. Golem draws from external wallet with daily spending limits. Revocable at any time. | Users who want to keep custody in their own wallet |
| **Embedded** | Privy server-side wallet. Zero additional setup. Funded directly. | Most users — lowest friction |
| **LocalKey + Delegation** | Local key file + MetaMask delegation for spend authorization. | Self-hosted Golems, advanced users |

The custody mode is selected once during onboarding and displayed on the Custody screen (`k`). Changing modes requires re-configuration via `bardo-terminal` or the web portal.

**Proxy spending authorization** (Delegation mode): The terminal sets up a Permit2 `PermitBatchTransferFrom` approval with daily spending limits and time bounds. Signing happens in the browser (Privy redirects to the connected wallet). The Golem's server wallet is authorized to pull up to `spendingLimit` USDC per day. Revocable via `:revoke-proxy` in the command palette or the `k` Custody screen.

---

## 5. Real-time communication

### 5.1 WebSocket subscription model

All surfaces connect to the Golem's HTTP interface. The primary transport is WebSocket (full-duplex):

```rust
pub async fn serve(fabric: Arc<EventFabric>, config: SurfaceConfig) {
    let app = Router::new()
        .route("/ws", get(ws_handler))                        // Full-duplex events + commands
        .route("/events", get(sse_handler))                   // SSE fallback
        .route("/api/v1/state", get(snapshot_handler))        // Initial state load
        .route("/api/v1/steer", post(steer_handler))
        .route("/api/v1/followup", post(followup_handler))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/creature", get(creature_handler))
        .route("/api/v1/achievements", get(achievements_handler))
        .route("/api/v1/lineage", get(lineage_handler))
        .route("/api/v1/graveyard", get(graveyard_handler))
        .with_state(AppState { fabric, config });
}
```

Three transport modes:

| Transport | Use case | Bidirectional | Latency |
| --- | --- | --- | --- |
| WebSocket | TUI and web dashboard (primary) | Yes — clients can send steers/followUps | <50ms |
| SSE | Web fallback | No — server-to-client only | <100ms |
| Telegram/Discord | Push notifications for high-signal events | No | 1-3s |

### 5.2 WebSocket protocol

1. Client connects with auth token (Privy JWT or API key)
2. Client sends: `{ "subscribe": ["heartbeat", "daimon", "creature", ...] }`
3. Server streams matching GolemEvents as newline-delimited JSON
4. Client can send steers/followUps on the same connection (bidirectional)
5. On reconnection, client sends `{ "resume_from": <sequence_number> }` — server replays from the Event Fabric (the tokio::broadcast channel with a 10,000-entry ring buffer that carries all Golem events) ring buffer

Subscription model: clients subscribe to event categories, not individual event types. Categories map 1:1 with TUI screens. Subscribing to "heartbeat" delivers all events the Mind screen needs.

### 5.3 Wire format

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WireEvent {
    Heartbeat(HeartbeatEvent),
    Tool(ToolEvent),
    Inference(InferenceEvent),
    Dream(DreamEvent),
    Daimon(DaimonEvent),
    Vitality(VitalityEvent),
    Grimoire(GrimoireEvent),
    Coordination(CoordinationEvent),
    Custody(CustodyEvent),
    Marketplace(MarketplaceEvent),
    Engagement(EngagementEvent),
    Context(ContextEvent),
    Trading(TradingEvent),
}
```

### 5.4 GolemSnapshot (initial load)

When a client first connects (or reconnects after a gap too large for replay), it receives a full GolemSnapshot (the read-only projection of the Golem's entire internal state, designed for UI consumption). See `11-state-model.md` for the canonical GolemSnapshot definition.

**Forward compatibility:** The `WireEvent` enum includes an `Unknown` variant. Clients must ignore unrecognized event types rather than failing. This allows the server to introduce new event categories without breaking older TUI versions.

```rust
#[serde(other)]
Unknown,
```

**WebSocket handshake:** The initial WebSocket connection includes a `protocol_version` field. The server uses this to gate which event types it sends, enabling safe rollout of new wire events across client versions.

```json
{ "subscribe": ["heartbeat", "daimon", ...], "protocol_version": "1.0" }
```

### 5.5 Connection resilience

On WebSocket disconnect, the TUI status bar shows `[RECONNECTING...]` and the screen enters read-only mode. Reconnection uses exponential backoff: 1s → 2s → 4s → 8s, capped at 30s. On reconnect, the TUI sends the last seen sequence number and replays missed events. redb KV store provides offline state — the last known GolemSnapshot persists between sessions.

---

## 6. Operator intervention

### 6.1 `steer()` — urgent intervention

`steer()` is the reflex arc. It injects a system-level directive into the active Golem-RS session, tagged `source: "operator_steer"`, visible to the LLM at the start of its next turn regardless of FSM state.

**Latency target:** <2 seconds from Owner action to LLM context change.

Before `steer()` fires, the FSM substate transitions to `ACTING.CANCEL_PENDING`, blocking new `commit_action` calls. The steer directive and permit state agree before the LLM sees the directive.

```rust
#[derive(Deserialize)]
pub struct SteerRequest {
    pub message: String,
    pub priority: Priority,  // Normal | Urgent
}
```

Use cases: health factor breach, emergency halt, signer anomaly, market crash. When a probe fires `high` severity, the GolemEventBus routes the event to `golem-cybernetics`, which cancels all pending ActionPermits and fires `session.steer()` before the LLM's next turn starts.

### 6.2 `follow_up()` — non-urgent guidance

`follow_up()` is not a slower `steer()`. It is a different primitive. `steer()` changes what the Golem is doing right now. `follow_up()` changes how it deliberates the next time it reaches a decision boundary.

**Timing guarantee:** Delivered at the next DECIDING state, not before. The UI shows "queued for next decision cycle" when the Owner submits a follow-up.

```rust
#[derive(Deserialize)]
pub struct FollowUpRequest {
    pub message: String,
}
```

In the TUI, steers appear with a gold border in the Steer screen's conversation log. Follow-ups appear with a dim border and "queued" label until they are consumed.

### 6.3 ActionPermit flow

Write operations (trades, LP adjustments, vault deposits) go through the ActionPermit flow. The Golem proposes an action; the runtime validates it against PolicyCage (the on-chain smart contract enforcing safety constraints like position limits, gas caps, and allowed protocols) constraints; and the result is either committed or rejected. Each step emits a visible GolemEvent:

```rust
pub enum GolemEvent {
    ActionPermitIssued {
        permit_id: String,
        action_type: String,
        params: serde_json::Value,
        risk_assessment: RiskLevel,
        timestamp: u64,
    },
    ActionPermitCommitted { permit_id: String, tx_hash: String, timestamp: u64 },
    ActionPermitRejected { permit_id: String, reason: String, timestamp: u64 },
    ActionPermitCancelled { permit_id: String, source: String, timestamp: u64 },
}
```

On the Steer screen, active permits are listed with their status, kind, and expiry. A permit can be cancelled by the Owner before it is committed.

### 6.4 Intervention availability by surface

| Surface | steer() | follow_up() | Notes |
| --- | --- | --- | --- |
| TUI (`s` screen) | Yes | Yes | Full bidirectional, chat-style |
| Web portal | Yes | Yes | Dashboard controls |
| Telegram | Read-only queries | No | Write ops redirect to portal |
| Discord | `/bardo ask` | No | Read-only queries |
| REST API | Yes | Yes | Direct HTTP |

---

## 7. Heartbeat FSM

The Heartbeat (the Golem's 9-step decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect) is the Golem's core execution loop. Every tick advances through a deterministic state machine.

### 7.1 FSM states

```
IDLE --> SENSING --> DECIDING --> ACTING --> REFLECTING --> SLEEPING --> IDLE
              |                                                  |
              +---------- SUPPRESSED (T0, all probes clear) --->+
                                                                |
                                                         SLEEPING.DREAMING
                                                         (nrem → rem → integration)
```

| State | Duration | Cost | What happens |
| --- | --- | --- | --- |
| IDLE | Regime-adaptive (15-120s) | $0 | Tick timer countdown |
| SENSING | <100ms | $0 | 16 deterministic probes evaluate market + system state |
| DECIDING | 1-5s | $0.001-$0.25 | LLM evaluates escalated probes with full context |
| ACTING | Variable | Gas costs | Execute tool calls (trades, LP, vault ops) |
| REFLECTING | 1-5s | $0.001-$0.05 | OutcomeVerification + LLM reflection |
| SLEEPING | <100ms | $0 | Dream eligibility check, Curator trigger |
| SLEEPING.DREAMING | 5-15 min | $0.001-$0.05 | Three-phase dream cycle |

### 7.2 Probe system (T0 — deterministic, $0)

16 probes run every tick during SENSING. All are compiled Rust — no LLM inference. Each returns `ProbeResult` with severity `none | low | high`. Probes cover: price delta, TVL delta, position health, gas spike, credit balance, RSI extreme, MACD cross, circuit breaker, kill switch, replicant report, clade alert, homeostatic drift, world model drift, causal graph consistency, VPIN spike, and IL threshold.

### 7.3 Tier escalation

| Tier | Model | Trigger | Cost |
| --- | --- | --- | --- |
| T0 | None (rule-based) | All probes clear — suppress tick | $0 |
| T1 | Haiku 4.5 | Single probe low, or cached decision | ~$0.001 |
| T2 | Sonnet 4.6 | Multiple probes, novel situation | ~$0.01 |
| T3 | Opus 4.6 | Critical severity, no similar past episodes | ~$0.05-0.25 |

**Cost cascade** — daily cost cap enforces tier ceilings:

| Daily spend | Effect |
| --- | --- |
| <70% cap | All tiers available |
| 70-90% | T3 suppressed |
| 90-100% | T2 + T3 suppressed |
| 100% | T0 only until midnight UTC or credit top-up |

### 7.4 Decision cache

Before escalating to T1+, the heartbeat checks the decision cache. Cache hit at confidence > 0.7 skips LLM inference entirely. Invalidated on regime change or PLAYBOOK (the owner-authored strategy document that defines the Golem's investment mandate, risk parameters, and allowed protocols) revision. Re-validated every 200 ticks.

```rust
pub struct DecisionCache {
    pub pattern_hash: String,
    pub mental_models_applied: Vec<String>,
    pub consistent_action: String,
    pub consistency_count: u32,
    pub expires_at: u64,
    pub cache_tick: u64,
    pub cache_regime: String,
}
```

Economics: a cache hit saves ~$0.003 per tick. At 60% hit rate over 1440 ticks/day, that saves ~$2.59/day — equivalent to ~3.5 additional days of lifespan.

### 7.5 Adaptive Clock theta interval (regime-dependent)

| Regime | Multiplier | Effective theta interval |
| --- | --- | --- |
| `bull_low_vol` | 1.5x | 90s |
| `bull_high_vol` | 0.5x | 30s |
| `bear_low_vol` | 1.0x | 60s |
| `bear_high_vol` | 0.5x | 30s |
| Conservation phase | 2.0x | 120s |

Theta interval is clamped to [30s, 120s]. Gamma ticks (5-15s) run concurrently for perception; delta ticks (5-30min) handle maintenance.

---

## 8. Multi-surface rendering

A single Golem-RS session is shared across all surfaces. Messages from any surface append to the same JSONL session. Responses fan out to all connected surfaces.

| Surface | Input | Output | Reasoning display |
| --- | --- | --- | --- |
| TUI | Keyboard + command palette | Full ratatui render, sprites, sparklines | `[thinking...]` spinner → Mind screen for full pipeline |
| Web portal | Text + file upload | Markdown + collapsible panels | Collapsible reasoning panel |
| Telegram | Text messages + commands | Formatted text, no panels | Suppressed |
| Discord | Text + slash commands | Embeds + reactions | "Show reasoning" reaction |
| REST API | JSON-RPC 2.0 | JSON responses | Full ReasoningChain in response |

**Provider badge** — each response carries provider metadata:
- TUI: Status line `[sonnet-4.6/blockrun/87%/std]`
- Web: Subtle badge below response
- Telegram: Suppressed
- API: `X-Bardo-Backend`, `X-Bardo-Pipeline`, `X-Bardo-Reasoning-Tokens` headers

---

## 9. GolemMode and dream interaction

```rust
pub enum GolemMode { Waking, Dreaming, Dying }
```

### 9.1 Mode transitions

```
WAKING <──────────> DREAMING
  │                     │
  └───────> DYING <─────┘
```

| From | To | Trigger | Behavior |
| --- | --- | --- | --- |
| WAKING | DREAMING | `dreamUrgency.composite > threshold` at SLEEPING | Heartbeat FSM suspends; dream cycle begins |
| DREAMING | WAKING | Dream cycle complete or wake interrupt | Heartbeat FSM resumes from IDLE |
| WAKING | DYING | Death trigger (economic, epistemic, stochastic, operator) | Thanatopsis Protocol (the four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy) begins |
| DREAMING | DYING | Death trigger during dream | Immediate interrupt; Thanatopsis begins |

### 9.2 During Dreaming

The Golem does not respond to steers. Messages receive an automatic hold response:

```
"I'm dreaming right now. I'll be back in ~8 minutes.
 You can send 'wake' to interrupt."
```

**Wake interrupt**: Send `wake` via any surface. The current dream phase completes consolidation (to avoid losing partially-processed knowledge), then the Golem returns to Waking mode.

The TUI shows the Dreams screen (`d`): NREM replay, REM counterfactual tree, integration visualization, sprite sleeping with slow breathing pulse.

### 9.3 During Dying

The 4-phase Thanatopsis Protocol: Acceptance → Settlement → Reflection → Legacy. Chat is disabled except for status queries. Death progress streams via `death.protocol_step`. The TUI renders the full death animation sequence (30 seconds, cinematic).

---

## 10. Live event stream

GolemEventBus events surface to all connected clients in real-time:

| Event type | Description | Channel |
| --- | --- | --- |
| `HeartbeatStateTransition` | FSM state changes (IDLE → SENSING → DECIDING) | `events` |
| `ProbeAlert` | Probe severity changes (low/high triggers) | `events` |
| `ActionPermitIssued` | New ActionPermit created for a write operation | `events` |
| `ToolCallAttempt` | Tool invocation start/complete/error | `terminal` |
| `DreamStarted` | Golem entered dream mode with estimated duration | `events` |
| `CreatureEvolution` | Sprite form transition (hatchling → mature, etc.) | `engagement` |
| `AchievementUnlocked` | New achievement earned | `engagement` |
| `PhaseTransition` | BehavioralPhase (the five survival phases: Thriving, Stable, Conservation, Desperate, Terminal) change (Stable → Conservation) | `events` |
| `DaimonAppraisal` | PAD vector (Pleasure-Arousal-Dominance emotional coordinates computed by the Daimon affect engine) update, emotional state change | `daimon` |
| `VitalityUpdate` | Vitality (composite survival score from 0.0 to 1.0 derived from the three mortality clocks) update, credit balance, projected lifespan, clock updates | `vitality` |
| `BloodstainReceived` | Incoming Bloodstain (market conditions recorded at another Golem's death, broadcast as warnings via Styx) matching current conditions | `clade` |

---

## 11. Session management

Sessions persist as Golem-RS JSONL files on the Golem's local storage (`/var/lib/bardo/sessions/`).

| Endpoint | Method | Description |
| --- | --- | --- |
| `/api/v1/sessions` | GET | List sessions with metadata |
| `/api/v1/sessions/:id` | GET | Session detail with full message history |
| `/api/v1/sessions` | POST | Create a new session |
| `/api/v1/sessions/:id/resume` | POST | Resume an existing session |

```rust
#[derive(Serialize, Deserialize)]
pub struct GolemSession {
    pub id: String,
    pub created_at: u64,
    pub last_active_at: u64,
    pub message_count: u32,
    pub tool_call_count: u32,
    pub inference_tokens: u64,
    pub cost_usdc: f64,
    pub status: SessionStatus,  // Active | Suspended | Completed
}
```

Each session maintains a configurable context window. When the window fills, older messages are summarized (via Haiku) and compressed into a session summary.

| Setting | Default | Range |
| --- | --- | --- |
| `max_context_tokens` | 100,000 | 10,000-200,000 |
| `compression_threshold` | 0.8 | 0.5-0.95 |
| `summary_model` | `claude-haiku-4-5` | any supported model |

---

## 12. Emotional state in conversation

The Golem's emotional state (PAD vector from the Daimon) modulates conversational tone. A Golem experiencing high arousal and low pleasure writes more cautiously. A Golem in anticipation communicates with forward-looking language.

```
Owner: "How are you feeling?"
Golem: "Cautious — moderate anxiety from the recent volatility spike.
        PAD: pleasure -0.2, arousal 0.6, dominance 0.3 (apprehension).
        I'm being more conservative with position sizing until the
        regime stabilizes."
```

In the TUI, the sprite's expression reflects the PAD vector continuously. The Hearth screen sidebar shows the current emotional state as a PADDial widget. The three mortality-specific emotions from the Daimon (`prd2-daimon.md`) produce distinctive sprite visual signatures: Economic Anxiety causes body flickers and aura contraction, Epistemic Vertigo produces eye unfocus animations and scattered particles, Stochastic Dread creates dark particle overlay and hunched posture.

---

## 13. Mortality status

```
Owner: "How long do you have?"
Golem: "Vitality: 0.62 (Stable phase). At current burn rate ($0.75/day)
        and income ($0.45/day net), approximately 28 days remaining.
        Hayflick ticks: 67,420 of 100,000.
        Next phase transition (Conservation) projected at vitality 0.40,
        roughly 18 days from now."
```

Critical mortality events — entering Declining or Terminal phase — trigger push notifications on all configured channels regardless of notification preferences. The TUI Mortality screen (`x`) shows three clock gauges (economic/epistemic/stochastic), the phase timeline, and the full lineage tree.

---

## 14. Graveyard (post-death)

When a Golem dies, there are no background settlement workers. The Golem's VM is gone. Its wallet persists in Privy's infrastructure independently of the VM.

### 14.1 Sweep model

The Owner clicks "Sweep" in the portal or runs `:sweep` in the TUI command palette. The portal signs the sweep transaction via the Privy server SDK — no Golem process needed.

| Category | Action |
| --- | --- |
| Settleable (freely transferable tokens) | One-click sweep to Main Wallet |
| Timelocked (positions with cooldown) | Shows estimated unlock time; sweep later |

### 14.2 Graveyard view

First-class section of the TUI (`x` Mortality screen, lineage tree section) and the web portal. Per-Golem tombstone: vital statistics, wallet balance with sweep button, death testament, Grimoire (the Golem's persistent knowledge base of episodes, insights, heuristics, warnings, and causal links) archive (browsable, searchable), performance summary, achievements earned in that life.

In the TUI, dead Golems render as desaturated tombstone sprites with a subtle static animation.

### 14.3 Death notifications

All surfaces receive death notifications with action links: "View Testament", "Create Successor", "Sweep Funds."

The death recap (roguelike kill screen) renders in the TUI with the full 30-second dissolution animation, cause narrative (Opus-generated), lifetime statistics, emotional arc, and succession prompt. See `../18-interfaces/03-tui.md` §7.4 for the full `DeathRecap` struct.

---

## 15. Notification preferences

```rust
pub struct NotificationPreferences {
    pub tui: TuiNotifications,
    pub web: WebNotifications,
    pub telegram: PlatformNotifications,
    pub discord: PlatformNotifications,
    pub webhook: WebhookNotifications,
}

pub enum NotificationEvent {
    PhaseChange,
    TradeExecuted,
    LowCredits,
    CriticalCredits,
    DeathImminent,
    DailySummary,
    CladeAlert,
    ReplicantReport,
    AchievementUnlocked,
}
```

| Event | TUI | Web | Telegram | Discord |
| --- | --- | --- | --- | --- |
| Phase change | Toast | Yes | Yes | Yes |
| Trade executed | Log entry | Yes | No | No |
| Low credits | Toast + status bar | Yes | Yes | Yes |
| Critical credits | Toast (persistent) | Yes | Yes | Yes |
| Death imminent | Full-screen alert | Yes | Yes | Yes |
| Daily summary | — | Yes | Yes | Yes |
| Achievement unlocked | Popup + particles | Yes | No | No |

---

## 16. Error handling

### 16.1 Connection loss

| Scenario | Recovery |
| --- | --- |
| WebSocket disconnects | Auto-reconnect with `resume_from` sequence replay (3s → 6s → 12s backoff) |
| Telegram bot offline | Messages queued by Telegram, delivered on reconnect |
| Discord slash command timeout | Ephemeral "Processing…" message, follow-up edit |
| TUI loses WebSocket | Status bar shows `[RECONNECTING...]`, enters read-only mode |
| Golem-RS session crash | New session created, previous context loaded from JSONL |

### 16.2 Inference failures

| Failure | Behavior |
| --- | --- |
| x402 (micropayment protocol for inference, compute, and data purchases via signed USDC transfers) payment rejected | Fallback to OpenRouter, then direct key |
| All inference unavailable | Golem operates in T0-only mode (rule-based) |
| Rate limited | Backoff + downgrade model tier |
| Budget exceeded | T0-only until next day or credit top-up |

Errors surface with actionable context on all connected surfaces. The TUI status bar turns red and shows the error code.

---

## 17. Interaction patterns

### Daily check-in (<1 minute)

```
Owner: "How are you doing?"
Golem: "Running well. Phase: stable. NAV up 1.2% today.
        3 trades executed (2 profitable). Morpho yield at 4.8%.
        USDC balance covers ~7.2 days. No alerts."
```

### Strategy review (5-10 minutes)

```
Owner: "Walk me through your recent decisions"
Golem: "Over the past 24h I made 3 key decisions:

        1. Rebalanced Morpho allocation from 35% to 42% after
           supply rate climbed to 5.1% (insight_2847)

        2. Closed ETH/USDC LP position at tick 4180 after range
           exit. Net P&L: +$47.20 (1.8% over 3 days)

        3. Skipped a swap at tick 4195 — high gas ($2.40) relative
           to expected $12 profit. Cached as heuristic_0891."

Owner: "Good call on #3. Increase your gas threshold to $3"
Golem: "Updated gas threshold heuristic. Won't override PolicyCage
        (max $5 gas per tx). Applied starting next tick."
```

### Emergency intervention

```
Owner (steer, priority: urgent): "Stop all trading immediately"
Golem: "Heartbeat paused. All pending tool calls cancelled.
        2 open LP positions (no immediate action needed)
        1 pending vault rebalance (cancelled)
        No in-flight transactions.

        Resume with 'resume trading' when ready."
```

---

## 18. Interaction hierarchy (v4)

The v4 interaction hierarchy uses a layered depth model: windows, tabs, screens, panes, elements, and modals. 29 screens across 6 windows.

**Source:** `tmp/research/design-inspo/engagement-prd/bardo-v4-00-interaction-hierarchy.md`

### 18.1 Terminology

```
WINDOW    The outermost container. A major conceptual category.
          Navigate between windows with Tab / Shift-Tab.
          There are exactly 6 windows. They are always visible
          in the top bar.

TAB       A sub-view within a window. Each window has 2-6 tabs.
          Navigate between tabs with number keys 0-9.
          Tabs appear as a secondary bar below the window bar
          when that window is active.

SCREEN    What you see when a specific tab is selected. A screen
          is a layout of panes filling the available space below
          the tab bar and beside the golem sidebar.

PANE      A bounded region within a screen. Contains data, widgets,
          or interactive elements. One pane has FOCUS at any time.
          Navigate between panes with arrow keys.

FOCUS     The highlighted pane. Receives keyboard input. Shows
          available commands in the bottom bar. Bright border.

LOCKED    When you press Enter on a focused pane, focus LOCKS.
          Arrow keys now scroll/select WITHIN the pane instead
          of moving between panes. Backspace unlocks.

MODAL     A floating overlay triggered by Enter on a selected item
          within a locked pane. Covers 40-80% of the screen.
          Has its own internal structure (can contain tabs, panes,
          elements -- the full hierarchy repeats inside).
          Backspace closes the modal. Modals can nest infinitely.

ELEMENT   An individual interactive item within a pane: a list row,
          a gauge, a chart, a text block. Elements are selectable
          when a pane is locked. Enter on an element opens a modal.
```

### 18.2 The depth stack

```
LAYER 0:  WINDOW      (Tab/Shift-Tab cycles)
LAYER 1:  TAB         (Number keys 0-9 switch)
LAYER 2:  SCREEN      (the layout of panes)
LAYER 3:  PANE FOCUS  (arrow keys move focus between panes)
LAYER 4:  PANE LOCKED (Enter locks; arrows scroll/select within)
LAYER 5:  ELEMENT     (Enter on element opens modal)
LAYER 6:  MODAL       (has its own tabs, panes, elements)
LAYER 7:  MODAL LOCKED (Enter within modal -> nested modal)
LAYER 8+: NESTED      (infinite depth -- modals within modals)

Backspace always goes UP one layer.
Esc goes all the way back to Layer 3 (pane focus on current screen).
```

### 18.3 The six windows

The windows represent six dimensions of a Golem's existence. The question "which window does X belong in?" should have an obvious answer based on what dimension X relates to.

```
HEARTH   MIND   SOMA   WORLD   FATE   COMMAND
```

| Window | Concept | Tabs |
| --- | --- | --- |
| **HEARTH** | *"What is happening right now?"* The Golem as a living thing. Vital signs, heartbeat rhythm, emotional state, operational status. The ambient screen. | Overview, Signals, Operations, Status |
| **MIND** | *"What does the Golem know, and how does it think?"* Decision pipeline, knowledge library, strategic doctrine, dream system, inference engine. | Pipeline, Grimoire, Playbook, Dreams, Inference |
| **SOMA** | *"What does the Golem own, and what does it cost?"* Positions, trades, wallet security, marketplace. Named for the Greek word for "body." | Portfolio, Trades, Custody, Bazaar, Budget |
| **WORLD** | *"What else exists, and how do they relate to me?"* The ecosystem. Other Golems (the Solaris ocean), the Clade (a cooperative group of Golems sharing knowledge and coordinating strategy via Styx), the pheromone field, bloodstains. | Solaris, Clade, Lethe, Bloodstains |
| **FATE** | *"How long will the Golem live, and what will it leave behind?"* Mortality, lineage, achievements, legacy. The confrontation with finitude. | Mortality, Lineage, Achievements, Graveyard |
| **COMMAND** | *"What can I do, and how do I configure things?"* Chat, configuration, visual effects settings, Hermes conversation. The owner's agency. | Steer, Config, Effects, Hermes |

### 18.4 Persistent chrome

Four persistent elements appear across all windows:

**Window bar (top row 1):** Active window framed with corner brackets in rose. Inactive windows in dim text. Unread indicator dot after window names with unseen changes.

**Tab bar (top row 2):** Shows the current window's tabs. Active tab highlighted in rose. Number keys switch tabs.

**Golem sidebar (left, 8-12 cols):** Always visible, even in modals. Single Golem: full Spectre (the Golem's animated visual creature representation in the TUI, reflecting its emotional state and health) with mini mortality gauges. Multiple Golems: vertical stack of mini-Spectres. `Ctrl+Up/Down` cycles between Golems, crossfading all data on screen over ~500ms.

**Status bar (bottom row -2):** Phase indicator, tick counter (FlashNumber), credit balance, breadcrumb showing full navigation depth (`Window > Tab > Pane [LOCKED] > Element [MODAL]`), heartbeat pulse.

**Command bar (bottom row -1):** Top 5-8 contextual keys for the current depth layer. Changes as you navigate deeper. `F1` always shows the complete key reference.

### 18.5 Focus and lock system

**Pane focus (Layer 3):** Arrow keys move focus between panes. Focused pane has bright border and shows available commands. Unfocused panes remain visible and updating.

**Pane lock (Layer 4):** Enter locks focus. Border doubles to show lock state. Arrow keys now scroll/select within the pane. Letter keys become pane-specific commands. Other panes dim to 70%.

**Element selection (Layer 5):** Within a locked pane, selectable elements highlight on cursor. Selected rows get `rose_deep` background.

**Modals (Layer 6+):** Floating panel, double-line borders, 40-80% screen size, centered. Background dims to 40%. Golem sidebar remains visible. Modal content follows the same hierarchy -- tabs, panes, elements, nested modals. Depth is infinite. Every piece of data is a portal.

### 18.6 Navigation summary

```
Tab / Shift-Tab     Cycle between the 6 windows
1-9                 Switch tabs within the current window
Arrow keys          Move focus between panes (or scroll when locked)
Enter               Lock focus on pane / Select element / Open modal
Backspace           Unlock pane / Close modal / Go up one layer
Esc                 Jump back to Layer 3 (pane focus) from anywhere
Ctrl+Up / Ctrl+Down Cycle between Golems (changes all data on screen)
/                   Search (context-sensitive)
F1                  Full help for current context
?                   Available keys overlay
:                   Command palette (fuzzy search across everything)
Ctrl+Q              Quit bardo-terminal
```

### 18.7 Golem-contextual rendering

Everything on every screen depends on the selected Golem. When cycling Golems with `Ctrl+Up/Down`, the entire terminal transforms:

**What changes:** All data (positions, trades, Grimoire entries, PAD values, vitality, balances), Spectre appearance, color temperature (PAD-modulated atmosphere), noise floor (denser for crisis, calmer for stable), heartbeat rhythm, phase degradation level, philosophical fragment pool.

**What stays the same:** Window/tab structure, Golem sidebar (all Golems still visible), command bar layout, persistent chrome.

**The transition:** Smooth crossfade (~500ms). Data values lerp between old and new Golem state. Not a snap cut. A blending, as if the terminal's attention is shifting from one mind to another.

### 18.8 Modal types

| Type | Use | Layout |
| --- | --- | --- |
| **Detail** | Grimoire entries, trade details, position details | Full content top, metadata grid below, action buttons bottom, internal tabs |
| **Comparison** | Eros/Thanatos splits, counterfactual branches | Two-column side by side, differences highlighted |
| **Timeline** | Confidence history, PnL history, dream replay | Horizontal timeline as primary axis, cursor-navigable |
| **Graph** | Causal graph, Clade topology, lineage tree | Interactive graph filling modal, zoom/pan with arrows |
| **Editor** | Config parameters, strategy tuning | Form-style with preview of effect |
| **Conversation** | Steer follow-ups, Hermes queries | Chat-style with streaming responses, inherits element context |

### 18.9 Responsive behavior

| Terminal Width | Golem Sidebar | Screen Layout | Modal Size |
| --- | --- | --- | --- |
| **Compact** (80-119 cols) | 6 cols, eyes only | Single column, panes stack vertically | 90% width |
| **Standard** (120-159 cols) | 10 cols, mini Spectre | Two columns | 70% width |
| **Wide** (160-199 cols) | 12 cols, full Spectre | Three columns | 60% width |
| **Ultra** (200+ cols) | 14 cols, full Spectre + waveforms | Three columns + detail sidebar | 50% width |

### 18.10 The Persona 5 principle

Every layer of depth should feel like a discovery, not a chore. Each layer reveals something the previous layer only hinted at:

- The window gives you the feeling (warm/cool/liminal)
- The tab gives you the category (what kind of data)
- The screen gives you the overview (the layout, the relationships)
- The focused pane gives you the detail (the numbers, the trends)
- The locked pane gives you the interaction (sort, filter, navigate)
- The element gives you the specifics (this one entry, this one trade)
- The modal gives you the depth (full history, provenance, cross-links)
- The nested modal gives you the rabbit hole (follow any thread anywhere)

---

## References

- [REEVES-NASS-1996] Reeves, B. & Nass, C. *The Media Equation.* Cambridge University Press, 1996. Demonstrates that people treat computers and media as social actors, applying the same social rules they use with other humans. This underpins Bardo's bet that making internal cognitive state visible creates affective bonds that drive retention.
- [MEHRABIAN-RUSSELL-1974] Mehrabian, A. & Russell, J.A. *An Approach to Environmental Psychology.* MIT Press, 1974. Introduces the PAD (Pleasure-Arousal-Dominance) model of emotional state, the theoretical foundation for the Daimon affect engine's three-axis representation.
- [ZHANG-ACE-2025] Zhang, A. et al. "ACE: Agentic Context Engineering." arXiv:2510.04618, 2025. Proposes structured context engineering for LLM agents, arguing that how context is assembled determines agent quality more than model choice. Informs the Golem's context window management and session compression strategy.
- [ANTHROPIC-CE-2025] Anthropic. "Context Engineering for Agents." 2025. Anthropic's practical guide to building agent context windows, covering prompt structure, tool result injection, and memory retrieval patterns. Directly influenced the Golem's heartbeat context assembly pipeline.
- [SHNEIDERMAN-1996] Shneiderman, B. "The Eyes Have It: A Task by Data Type Taxonomy for Information Visualizations." *VL*, 1996. Proposes the "overview first, zoom and filter, then details on demand" mantra for information visualization. This principle structures the TUI's layered depth model: window gives overview, tab gives category, locked pane gives detail, modal gives depth.

# Platform-specific UX [STUB]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies platform-specific UX constraints for each of Bardo's interface surfaces: the TUI (primary, 60fps Rust terminal application), the web Portal (secondary, browser-based), CLI, Telegram, and Discord. It covers responsive breakpoints, accessibility, cross-platform continuity, and notification strategy. It sits in the **Runtime** layer of the Bardo specification. Key prerequisite: the Golem (a mortal autonomous DeFi agent) interaction model from `00-interaction-model.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Bardo has three interface surfaces, each with distinct modality and audience. The TUI is the primary experience -- a persistent, 60fps Rust application where owners live alongside their Golems. The web Portal is secondary: setup wizards, reputation profiles, vault analytics. Mobile is tertiary and out of v1 scope.

This document specifies platform-specific UX constraints, responsive behavior, accessibility requirements, and how each surface handles the Golem's visual language under different rendering conditions.

> **Cross-references:**
>
> - `./14-creature-system.md` — the Spectre (visual creature) rendering system: sprite resolution tiers, animation degradation by terminal width
> - `../../18-interfaces/00-portal.md` — web Portal architecture: React app, API integration, and browser-specific rendering
> - `../../18-interfaces/01-cli.md` — CLI and TUI architecture: `bardo-terminal` binary, ratatui rendering, and command-line tooling
> - `../../18-interfaces/02-ui-system.md` — React component library: shared UI primitives for the web Portal
> - `./13-engagement-loops.md` — engagement loop design and cadence architecture that platform UX must support

---

## 1. Platform hierarchy

| Surface        | Modality    | Primary Use                                      | Audience            | Tech              |
| -------------- | ----------- | ------------------------------------------------ | ------------------- | ----------------- |
| **TUI**        | Persistent  | Live Golem management, creature experience, social | Owners, power users | Rust / ratatui    |
| **Portal**     | Web browser | Setup wizard, reputation profile, vault analytics | All users           | React / Next.js   |
| **CLI**        | One-shot    | `bardo golem create`, `bardo doctor`, scripting  | All users           | TypeScript / Node |
| **Telegram**   | Push + bot  | High-signal notifications, quick status checks   | Mobile-first users  | Bot API           |
| **Discord**    | Bot + embeds| Clade channels, rich status embeds               | Community users     | Bot API           |

The TUI is where the engagement loops (check-in, discovery, evolution, death, collection) are designed to play out. The other surfaces are auxiliary -- they surface subsets of the same data at lower fidelity.

---

## 2. TUI as primary platform

### 2.1 Why TUI first

A terminal is not a lesser interface. It is a more intimate one. Builders already live in terminals -- VS Code's integrated terminal, tmux sessions, SSH connections to remote machines. A TUI that makes managing an autonomous DeFi agent feel like inhabiting a living world produces engagement that no web dashboard replicates.

The TUI runs at 60fps on ratatui (Rust). This is non-negotiable. Sprite animation, particle systems, smooth transitions, and real-time event streaming demand a rendering loop that never drops frames. A 16.6ms frame budget. Node.js TUI frameworks (blessed, Ink) cannot hit 60fps reliably. The terminal is where Bardo must feel premium.

### 2.2 Responsive breakpoints

The TUI adapts to terminal dimensions through four layout modes:

| Terminal Width | Layout Mode      | Sprite    | Pane Arrangement                        |
| -------------- | ---------------- | --------- | --------------------------------------- |
| < 80 cols      | **Compact**      | 4-col mini (head only) | Single pane, tab-switch navigation      |
| 80-119 cols    | **Standard**     | 8-col (torso up) | 2-column: sidebar + main               |
| 120-179 cols   | **Wide**         | 12-col (full body) | 3-column: sidebar + main + detail       |
| 180+ cols      | **Ultra**        | 12-col + animations | 4-column: sidebar + main + detail + aux |

### 2.3 Degradation behavior

The TUI degrades gracefully based on runtime detection:

| Condition           | Behavior                                                                    |
| ------------------- | --------------------------------------------------------------------------- |
| No truecolor        | Falls back to 256-color palette mapping (quantized via octree)             |
| No Unicode          | ASCII-only mode (box drawing with `+|-`, sprites as ASCII art)             |
| Small terminal      | Compact mode: single-pane with tab switching instead of multi-pane         |
| No mouse            | Full keyboard navigation (vim-style). Mouse is optional throughout.        |
| No WebSocket        | Polling mode via HTTP at 2-second intervals (degrades to dashboard-like)   |
| SSH / remote session| Standard operation. All rendering is character-based. No local GPU needed. |

### 2.4 SSH and remote session support

The TUI is a standalone Rust binary communicating via WebSocket. It runs identically over SSH, in tmux/screen panes, in Docker containers, and on remote VPS instances. No X11 forwarding, no graphical dependencies. The render loop targets the `crossterm` backend for maximum terminal compatibility: Windows Terminal, iTerm2, Kitty, WezTerm, Ghostty, Alacritty, and standard VT100.

### 2.5 Performance targets

| Metric                      | Target      | Notes                                           |
| --------------------------- | ----------- | ----------------------------------------------- |
| Cold start to first render  | <500ms      | Rust binary, no interpreter startup              |
| Render frame rate           | 60 FPS      | 16.6ms frame budget, differential terminal flush |
| Event stream processing     | <1ms/event  | Zero-copy deserialization via `serde`            |
| Memory footprint            | <30MB RSS   | No GC, no VM, predictable allocation             |
| Binary size                 | ~8MB        | Static linking, embedded sprite atlas            |
| CPU idle (static screen)    | <1%         | Differential rendering skips unchanged cells     |
| CPU active (full animation) | <5%         | Sprite + particles + scrolling logs              |

---

## 3. Portal (web) as secondary platform

The Portal is a React/Next.js web application. It serves two functions:

1. **Setup** -- guided onboarding wizard (wallet, custody, ERC-8004 registration)
2. **Manage** -- ongoing agent profile, reputation tracking, vault analytics, dashboard

### 3.1 Golem-RS state model

The Portal receives Golem state via the same WebSocket event stream that the TUI consumes. It subscribes to the Event Fabric via the surface multiplexer (see `./12-realtime-subscriptions.md`). The Golem runtime is Rust (`golem-rs`), and the `GolemSnapshot` struct defines all renderable state.

For Golem owners, the Portal adds dashboard extensions:

| Section            | Content                                                                |
| ------------------ | ---------------------------------------------------------------------- |
| Vitality gauge     | Current USDC balance vs mortality threshold, estimated lifespan        |
| Behavioral phase   | Current phase (Thriving / Stable / Conservation / Declining / Terminal)|
| Heartbeat monitor  | Real-time heartbeat interval, last heartbeat timestamp                 |
| Grimoire inspector | Episode timeline, insight graph, heuristic rules, causal links        |
| Clade peers        | Connected siblings, shared knowledge, sync status                      |

### 3.2 Creature rendering on web

The `bardo-sprites` crate compiles to `wasm32-unknown-unknown` and renders to an HTML5 Canvas element via `wasm-bindgen`. The same procedural generation, animation system, and particle engine run in the browser at 60fps -- identical visual output to the TUI, rendered as pixel data instead of half-block characters.

### 3.3 Styx as data source

The Portal queries Styx for social data: death registry, leaderboards, agent profiles, bloodstain conditions. All reads go through the Styx REST API. No direct on-chain reads for social features (too slow for browsing).

### 3.4 Portal non-goals

The Portal is not the primary Golem experience. It doesn't replicate the TUI's 29-screen, 6-window system. It provides:

- Setup wizard (run once)
- Agent profile and reputation (reference)
- Vault analytics (numbers)
- Leaderboard (browsing)

The Portal doesn't provide: real-time creature interaction, dream visualization, death cinematics, command palette, steer interface. Those belong to the TUI.

---

## 4. CLI as one-shot interface

The `bardo` CLI (TypeScript/Node) handles non-interactive operations. See `../../18-interfaces/01-cli.md` for the full subcommand tree.

The CLI is distinct from `bardo-terminal` (Rust TUI). They are separate binaries with separate concerns:

| Binary           | Language   | Purpose                    | Session Type |
| ---------------- | ---------- | -------------------------- | ------------ |
| `bardo`          | TypeScript | Subcommands, scripting     | One-shot     |
| `bardo-terminal` | Rust       | Live Golem management, TUI | Persistent   |

---

## 5. Telegram and Discord

### 5.1 Telegram

High-signal push notifications only. Not every event goes to Telegram:

| Event | Telegram Message |
|-------|-----------------|
| Phase transition | Phase Transition: Stable -> Conservation |
| Death | Death: economic exhaustion. Check graveyard for recap. |
| Achievement (rare+) | Achievement: Hot Hand -- 10 consecutive profitable decisions |
| Dream complete | Dream cycle 47 complete |
| Bloodstain received | Bloodstain from generation 3: stale oracle warning |

Telegram Mini-App provides a lightweight creature viewport (Canvas 2D, 30fps) with basic status and steer capability.

### 5.2 Discord

Bot commands for status checks. Rich embeds for milestones. Clade channels for group coordination. Static/animated PNG renders of creature state. Live spectating via embed auto-refresh.

---

## 6. Accessibility

### 6.1 Screen reader support

The TUI implements ARIA-equivalent semantics through structured output. Unlike a web DOM, terminals have no accessibility tree. The TUI compensates with two mechanisms:

**Structured text announcements.** Every state change emits a structured text description to a dedicated announcement buffer. Screen readers that hook into terminal output (NVDA + Windows Terminal, VoiceOver + Terminal.app, Orca + GNOME Terminal) receive these as sequential text events:

- Creature state: "Your Golem Ember-7f3a is in Stable phase, mood: confident, vitality 67%, 23 days remaining"
- Navigation: "Focused: Grimoire screen, 47 episodes, 12 insights"
- Actions: "Trade executed: swapped 500 USDC for 0.18 ETH on Uniswap V3"
- Notifications: priority tier announced before content ("Critical: health factor breach on Morpho vault")

**`--accessible` mode.** Launched via `bardo-terminal --accessible` or set in `~/.bardo/config.toml`:

- Particle effects disabled (zero visual noise)
- Sprite replaced with a text status block (name, phase, vitality, mood, lifespan)
- All panes emit full text descriptions on focus, not just labels
- Tab-order navigation replaces vim-style keys (Tab / Shift-Tab cycles panes, Enter activates, Escape backs out)
- Terminal bell fires on Critical and High notifications (configurable: `bell = "critical"`, `"high"`, `"all"`, `"none"`)
- Render rate drops to 10fps (no visual information is lost; reduces CPU and screen reader chatter)

**Portal (web) accessibility.** The React portal follows WCAG 2.2 AA:

- All interactive elements have `aria-label` or visible label text
- Canvas-rendered sprites include a `role="img"` with `aria-label` describing creature state
- Focus management on route transitions (focus moves to page heading)
- Live regions (`aria-live="polite"`) for toast notifications, `aria-live="assertive"` for critical alerts
- Skip-to-content link on every page
- Color contrast ratio >= 4.5:1 for text, >= 3:1 for UI components

### 6.2 High-contrast themes

Three built-in themes:

| Theme           | Description                                        | WCAG contrast |
| --------------- | -------------------------------------------------- | ------------- |
| **Default**     | ROSEDUST -- rose on violet-black with bone accents | AA            |
| **High Contrast** | White text on pure black, bold borders, no gradients | AAA           |
| **Light**       | Dark text on light background (for outdoor use)    | AA            |

High Contrast mode additionally:

- Replaces gradient backgrounds with solid fills
- Doubles border widths on all panes
- Uses bold weight for all body text
- Removes background textures from sprite rendering (silhouette only, high-contrast fill)

Themes are selected via `~/.bardo/config.toml` or `,` (Settings) screen in TUI. The Portal respects `prefers-color-scheme` from the OS and allows manual override.

### 6.3 Reduced motion

When the OS reports `prefers-reduced-motion` (detected via `TERM` capabilities on terminal, CSS media query on web) or the user sets `reduced_motion = true` in config:

**TUI behavior:**
- Particle effects disabled entirely
- Sprite animations replaced with static poses that update on state change (phase transition, mood shift)
- Pane transitions are instant (no eased fades or slides)
- Scrolling is immediate, not smoothed
- Heartbeat pulse indicator uses a toggling symbol (dot/circle) instead of a pulsing animation

**Portal behavior:**
- CSS `animation: none` and `transition: none` applied globally via a root class
- Canvas sprite renders static frames, updated on state change events only
- Progress bars jump to final state instead of animating
- Toast notifications appear/disappear without slide animation

All information content is identical in reduced-motion mode. Motion is presentation, not data.

### 6.4 Color vision deficiency

The ROSEDUST palette is inherently high-contrast. Phase indicators use both color and shape, so they remain distinguishable under protanopia, deuteranopia, and tritanopia:

| Phase         | Color       | Shape                    | Pattern (alt) |
| ------------- | ----------- | ------------------------ | ------------- |
| Thriving      | Rose bright | Full circle              | Solid fill    |
| Stable        | Rose        | Open circle              | No fill       |
| Conservation  | Cool blue   | Half circle              | Hatched fill  |
| Declining     | Amber       | Triangle (warning)       | Diagonal fill |
| Terminal      | Red/black   | Cross (critical)         | Dense cross   |

The Pattern column applies when `colorblind_patterns = true` is set in config. This adds texture fills to all color-coded indicators, making them distinguishable without any color perception.

---

## 7. Cross-platform continuity

State is server-side (Golem VM + Styx). All surfaces read from the same source. Actions taken on any surface are immediately visible on all others.

| Action                 | Where It Happens | Effect on Other Surfaces                           |
| ---------------------- | ---------------- | -------------------------------------------------- |
| Steer (`:steer`)       | TUI or Portal    | Golem processes; all surfaces see updated behavior |
| Feed (top up USDC)     | TUI, CLI, Portal | Vitality updates everywhere                        |
| Create successor       | TUI or CLI       | New Golem appears on all surfaces                   |
| Achievement unlock     | Automatic        | Notification on TUI, push to Telegram, embed on Discord |
| Death                  | Automatic        | Full cinematic on TUI, push to Telegram, embed on Discord, Styx memorial on Portal |

No surface-exclusive features. Everything the Portal can do, the TUI and CLI can also do. The TUI provides the richest experience but never gates functionality.

### 7.1 State sync latency SLAs

All surfaces subscribe to the Event Fabric via WebSocket. The latency budget from state change to surface render:

| Hop | Target | P99 | Notes |
| --- | ------ | --- | ----- |
| Golem -> Event Fabric | <1ms | <5ms | In-process broadcast channel |
| Event Fabric -> TUI (local) | <5ms | <20ms | Direct WebSocket, LAN or localhost |
| Event Fabric -> TUI (remote via Styx) | <100ms | <300ms | Styx relay adds one network hop |
| Event Fabric -> Portal | <150ms | <500ms | Browser WebSocket + React render cycle |
| Event Fabric -> Telegram/Discord | <2s | <5s | Platform API latency dominates |

Total state propagation from a Golem action to the user seeing it on a remote surface: under 500ms for TUI and Portal, under 5 seconds for social connectors. If any surface falls behind by more than 30 seconds, it triggers a full state snapshot request instead of replaying missed deltas.

### 7.2 Conflict resolution

Two surfaces can issue conflicting actions simultaneously (e.g., user sends a steer via TUI while the Portal sends a different steer). Resolution rules:

1. **Last-write-wins for steers.** Steers are timestamped at the Golem's event loop. The Golem processes them in arrival order. If two steers arrive within the same heartbeat tick, the one with the later timestamp wins. The losing steer receives an `STEER_SUPERSEDED` event with the winning steer's content, so the surface can display "Your steer was overridden by a more recent one."

2. **Idempotent for financial actions.** Feed (top-up) transactions are on-chain and inherently serialized. Double-submitting a feed from two surfaces results in two separate top-ups, both valid. The UI shows pending state until the transaction confirms.

3. **No concurrent creation.** The creation wizard acquires a server-side lock on the owner's session. If a second surface attempts to start creation while one is in progress, it receives a redirect to "Creation in progress on [other surface]" with an option to take over.

### 7.3 Offline behavior

When a surface loses its WebSocket connection:

**TUI:** Shows a `DISCONNECTED` indicator in the status bar. Queues user actions (steers, commands) locally. On reconnection, replays queued actions in order and requests a full state snapshot to reconcile. If disconnected for more than 5 minutes, the TUI switches to a "last known state" view with a prominent reconnection banner. Local queue is capped at 20 actions; overflow is dropped with a warning.

**Portal:** Shows a yellow banner: "Connection lost. Reconnecting..." Retry with exponential backoff (1s, 2s, 4s, 8s, max 30s). Stale data is grayed out. User actions during disconnection are queued in the browser's IndexedDB (max 50 entries) and replayed on reconnection.

**Social connectors:** Bot commands issued while the Golem is unreachable receive a reply: "Your Golem is currently unreachable. Command queued -- it will execute when connectivity is restored." Commands are queued server-side in the Bott gateway for up to 1 hour, then expired with a notification.

---

## 8. Notification strategy

Notifications respect the platform hierarchy:

| Priority    | TUI                                    | Portal               | Telegram          | Discord           |
| ----------- | -------------------------------------- | -------------------- | ----------------- | ----------------- |
| **Critical**| Red banner, top of screen, blocks input | Browser notification | Immediate push    | @mention          |
| **High**    | Gold toast, top-right, 10s             | In-app notification  | Push              | Embed             |
| **Normal**  | Dim toast, top-right, 5s              | Badge only           | Digest (6h batch) | No notification   |
| **Low**     | Dot on sidebar screen label            | No notification      | No notification   | No notification   |

Critical: kill-switch, health factor breach, death. High: trade executed, dream completed, achievement. Normal: insight generated, clade sync. Low: routine heartbeat summary.

Notifications accumulate -- there is no push mechanism in the terminal. They surface when the user opens the TUI. This is by design. The system never interrupts; it waits.

### 8.1 Rate-limiting algorithm

Per-channel rate limits prevent notification fatigue. The algorithm operates per (user, channel, priority) triple:

```
token_bucket(capacity=C, refill_rate=R, refill_interval=T)

| Priority | Capacity (C) | Refill rate (R) | Refill interval (T) |
|----------|-------------|-----------------|---------------------|
| Critical | unlimited   | N/A             | N/A                 |
| High     | 10          | 1               | 10 min              |
| Normal   | 5           | 1               | 30 min              |
| Low      | 3           | 1               | 1 hour              |

On notification emit:
  if bucket.tokens > 0:
    send immediately
    bucket.tokens -= 1
  else:
    add to batch queue
    schedule batch delivery at next refill
```

Critical notifications always deliver immediately. They are never rate-limited, never batched.

### 8.2 Batching

When Normal or Low notifications exceed their rate limit, they accumulate in a batch buffer. Batches deliver at fixed cadence boundaries:

- **Normal batch**: every 6 hours (0600, 1200, 1800, 0000 UTC, adjusted to user's timezone)
- **Low batch**: daily digest at the user's configured summary time (default: 0800 local)

Batch format groups events by type ("3 insights generated", "Clade sync completed with 2 peers", "12 heartbeat summaries") rather than listing each event individually. If a batch contains more than 20 events, it truncates to the 10 highest-priority events plus a count of omitted ones.

### 8.3 Fatigue detection

The system monitors dismissal behavior to detect notification fatigue:

```
fatigue_score = dismissed_without_reading / total_delivered   (rolling 7-day window)

if fatigue_score > 0.6 for a given (user, priority) pair:
  demote that priority tier by one level for that user
  log demotion event to telemetry
  surface "Notification preferences adjusted" in Settings screen

if fatigue_score drops below 0.3 for 14 consecutive days:
  restore original priority level
```

"Dismissed without reading" is defined as: TUI toast auto-dismissed (user didn't interact), Telegram message not opened within 4 hours, Portal badge cleared via bulk dismiss. This heuristic is imperfect -- it errs on the side of reducing noise.

### 8.4 Accumulated event summary

When the user opens the TUI after an absence, the Home screen shows a "While you were away" section. The summary is generated by the Golem's T0 logic (no LLM call, deterministic):

```
time_away = now - last_session_end

if time_away < 1 hour:
  no summary (events are in the log)
if time_away < 24 hours:
  one-paragraph summary: key trades, P&L delta, phase changes, dreams completed
if time_away >= 24 hours:
  structured summary with sections: Performance, Knowledge, Health, Alerts
  each section max 3 bullet points
```

The summary is ephemeral -- it's computed on session start and not stored. If the user disconnects and reconnects within 5 minutes, the same summary persists. After 5 minutes of active session, it collapses to a single line.

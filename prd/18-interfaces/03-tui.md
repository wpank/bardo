# bardo-terminal [SPEC]

**Version:** 4.1.0
**Last updated:** 2026-03-17
**Crates:** `bardo-terminal`, `bardo-sprites`, `bardo-tui-widgets`
**Framework:** ratatui 0.29, crossterm 0.28, tokio 1.x
**Design reference:** Rebels in the Sky (https://github.com/ricott1/rebels-in-the-sky)

> **Design system cross-reference:** Visual identity, ROSEDUST palette, CRT materiality, component patterns, and atmospheric effects are specified in [04-design-system.md](04-design-system.md) (the ROSEDUST visual identity spec for color, typography, and atmospheric rendering). Screen layouts reference design system tokens directly. Widget catalog in [05-widget-catalog.md](05-widget-catalog.md) (the 33-widget component library for the TUI). Full 29-screen reference in [06-terminal-screens.md](06-terminal-screens.md) (per-screen layout and data specifications).

> **Reader orientation:** This is the main specification for `bardo-terminal`, the 60fps TUI (terminal user interface) that is the Golem's primary interactive surface. It belongs to the **interfaces layer** of the Bardo stack. The TUI is a full-screen Rust application built on ratatui, connected via WebSocket to a Golem (a mortal autonomous DeFi agent running on a micro VM). Key concepts: the 32 interpolating variables (continuously changing visual state channels that drive every pixel), the Spectre (a dot-cloud creature sprite representing each Golem), CorticalState (the Golem's 32-signal atomic perception surface), and the Daimon (the Golem's emotional appraisal subsystem producing PAD vectors). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 1. What this document covers

This document specifies `bardo-terminal` — the Golem's primary interactive interface. It is a full-screen TUI application written in Rust, rendered at 60 FPS, built on ratatui and crossterm. It connects to the Golem's WebSocket event stream and renders its inner life as a living, breathing creature.

The document covers: the perpetual motion principle, crate architecture, the 32-channel interpolating variable system, the 29-screen system (6 windows), the procedural creature/sprite system, real-time streaming, engagement loops, onboarding flow, social features, and the custom widget library. Per-screen specifications live in [06-terminal-screens.md](06-terminal-screens.md); the widget catalog in [05-widget-catalog.md](05-widget-catalog.md).

This is not the gotts-monorepo CLI tooling. The TypeScript `@bardo/tui` package (`packages/tui/` in the gotts-monorepo, built on picocolors and @clack/prompts) is a separate utility library for Node.js CLI output formatting. See the note at the end of this document.

---

## 2. Design vision

A terminal is not a lesser interface. It is a more intimate one. The builder's home. The place where the machine lives closest to the person who made it.

`bardo-terminal` transforms Golem management from monitoring into inhabitation. The player does not _use_ the terminal — they live alongside their Golem. The Golem is a creature: born, growing, learning, dreaming, feeling, dying. Every Pi lifecycle event, every heartbeat, every dream cycle, every emotional appraisal, every causal link discovered becomes a visible phenomenon in the TUI.

Three design pillars hold this together:

**The Golem as creature.** Each Golem has a procedurally generated sprite that evolves as it ages, learns, and emotes. A Golem in Thriving phase looks different from one in Conservation. A dreaming Golem animates differently. A dying Golem's sprite decays. The sprite is not decoration — it is a real-time emotional barometer, the visual encoding of the Daimon's (the Golem's emotional appraisal subsystem) PAD vector (Pleasure-Arousal-Dominance coordinates), rendered at 60 FPS with interpolated animation and particle effects.

**The world as living.** Other owners' Golems are visible. The Styx (Bardo's social and connectivity layer) shows tombstones with epitaphs. The Bazaar has browseable listings. Bloodstains (persistent map markers left where Golems died under specific market conditions) mark where Golems died in particular market conditions. The lineage tree shows generational knowledge compound. The world has weather — market regime visualized as ambient atmosphere.

**The loop as addiction.** Every session surfaces something new: a dream insight, a Clade (a group of sibling Golems sharing knowledge) sibling's discovery, a marketplace listing, a reputation milestone, a sprite evolution. The gacha loop is not manufactured — it is the fundamental unpredictability of autonomous agents in live markets, surfaced through a system that makes every login feel like opening a mystery.

---

## 3. The perpetual motion principle

The foundational design rule: **nothing on screen is ever at rest.**

Every element is driven by at least one continuously changing variable. A label that never changes still sits on a background that shimmers. A border that never moves still dims with lifecycle degradation. A number that hasn't updated still has a phosphor afterimage of its last change fading behind it. Static pixels are bugs.

This is not animation for decoration. It is the visual consequence of a system where every variable is coupled to every other variable through the Event Fabric, and the renderer's job is to make those couplings legible. A progress bar that decays is a mortality clock rendered honestly. A chart that shimmers is uncertainty in the data made visible. Brightness follows significance. Motion follows change. Silence follows death.

For every element on every screen, three questions must be answered:

1. **What makes it move?** Which Event Fabric channel(s) drive this element?
2. **What makes it change?** What state transitions alter its appearance -- color, density, brightness, rhythm?
3. **What makes it decay?** How does this element show the passage of time, the depletion of resources, the approach of death?

The terminal is a CRT where every bright moment leaves a ghost, every ghost fades, and every fade is replaced by the next moment's ghost. Persistence of vision. Persistence of memory. Persistence of the dead.

---

## 4. The 32 interpolating variables

The golem's visual state is driven by 32 continuously interpolating variables. Each has a current value and a target value. Events from the Event Fabric update targets; the renderer reads current values. The current value approaches the target at a rate determined by the variable's lerp constant.

### Variable table

| # | Variable | Category | Lerp rate | Range | What it drives |
|---|----------|----------|-----------|-------|----------------|
| 1 | `pleasure` | fast | 8.0 | [-1.0, 1.0] | Warm/cool palette shift. Tint toward bone (positive) or rose_bright (negative). |
| 2 | `arousal` | fast | 8.0 | [-1.0, 1.0] | Saturation, contrast, flicker speed. High = vivid + fast. Low = muted + slow. |
| 3 | `dominance` | fast | 8.0 | [-1.0, 1.0] | Edge sharpness, contrast assertiveness. High = crisp. Low = soft, uncertain. |
| 4 | `emotion_label` | fast | 6.0 | enum | Sprite eye expression, status bar emotion tag. Discrete Plutchik primary (8 values). Transitions snap at the midpoint of an 8-tick crossfade; the surrounding dot physics and eye rendering smoothly interpolate to absorb the discontinuity. |
| 5 | `fsm_phase` | fast | 10.0 | enum | Active pipeline stage highlight in Mind screen. Snaps fast but not instant. |
| 6 | `probe_severity` | fast | 12.0 | [0.0, 1.0] | Alert glow intensity. 0 = nominal. 1 = crisis. Drives border flicker and rose_bright usage. |
| 7 | `inference_glow` | fast | 10.0 | [0.0, 1.0] | Brightness pulse when LLM call is active. Fades quickly after response. |
| 8 | `mouth_alpha` | fast | 15.0 | [0.0, 1.0] | Sprite mouth visibility during speech. Very fast attack, medium decay. |
| 9 | `market_regime` | medium | 1.5 | enum | Background atmosphere zone. Bull = warm undertone. Bear = cool. Chop = flickering. |
| 10 | `context_utilization` | medium | 2.0 | [0.0, 1.0] | Context window fill gauge in Inference screen. |
| 11 | `phi_score` | medium | 1.0 | [0.0, 1.0] | Integrated information metric. Drives Mind screen's consciousness complexity indicator. |
| 12 | `dream_alpha` | medium | 0.8 | [0.0, 1.0] | Dream state overlay opacity. 0 = awake. 1 = deep REM. Affects entire screen palette. |
| 13 | `clade_connectivity` | medium | 1.2 | [0.0, 1.0] | Sibling network health in Clade screen. Responsive peers / total peers. |
| 14 | `phase_density` | slow | 0.15 | [0.0, 1.0] | Sprite dot cloud density. 1.0 in youth, eroding toward 0 as the golem ages. |
| 15 | `phase_dimming` | slow | 0.12 | [0.0, 1.0] | Global brightness multiplier. 1.0 = full. Drops in Conservation and Declining phases. |
| 16 | `vitality_composite` | slow | 0.2 | [0.0, 1.0] | Weighted combination of economic + biological + epistemic health. Drives sprite breathing rate. |
| 17 | `economic_clock` | slow | 0.1 | [0.0, 1.0] | Fraction of metabolic reserve remaining. 1.0 = full. 0.0 = death. |
| 18 | `epistemic_clock` | slow | 0.08 | [0.0, 1.0] | Knowledge freshness. Decays as Grimoire (the Golem's persistent knowledge store) entries go stale without refresh. |
| 19 | `age_factor` | glacial | 0.03 | [0.0, 1.0] | Normalized age. 0.0 at birth, 1.0 at Hayflick limit. Drives character corruption rate. |
| 20 | `credit_balance` | glacial | 0.05 | [0.0, inf) | Actual USDC balance. Displayed as the single bone-colored number on Hearth screen. |
| 21 | `grimoire_density` | glacial | 0.04 | [0.0, 1.0] | How full the knowledge store is. Drives background particle frequency in Grimoire screen. |
| 22 | `burn_rate` | glacial | 0.06 | [0.0, inf) | Current daily spend rate in USDC. Drives projected-lifespan gauge. |
| 23 | `heartbeat_phase` | fast | -- | [0.0, 2pi] | Not lerped -- free-running sine wave. Drives per-tick micro-brightness pulse that propagates from sprite through content. |
| 24 | `noise_floor` | slow | 0.15 | [0.0, 0.01] | Fraction of background cells that show dim noise characters per frame. Scales with lifecycle phase. |
| 25 | `scanline_intensity` | slow | 0.1 | [0.0, 0.15] | How dark the scanline rows are. Increases in degraded phases. |
| 26 | `corruption_rate` | slow | 0.08 | [0.0, 0.25] | Fraction of characters randomly substituted with glitch chars during render. 0 in Thriving. 15-25% in Terminal. |
| 27 | `prediction_accuracy` | medium | 1.5 | [0.0, 1.0] | Oracle accuracy gauge. Color: bone >70%, amber 50-70%, rose <50%. |
| 28 | `accuracy_trend` | slow | 0.2 | [-1.0, 1.0] | Direction of accuracy change. Positive = improving (sprite posture lifts). Negative = declining. |
| 29 | `attention_breadth` | medium | 1.0 | [0.0, 1.0] | Fraction of attention slots filled. High = dense atmosphere particles around sprite. |
| 30 | `surprise_rate` | fast | 6.0 | [0.0, 1.0] | Rate of unexpected prediction outcomes. High = frequent eye micro-flickers. |
| 31 | `foraging_activity` | medium | 1.2 | [0.0, 1.0] | Attention forager promotion/demotion rate. High = peripheral particles orbit sprite actively. |
| 32 | `compounding_momentum` | slow | 0.15 | [0.0, 1.0] | How well the prediction-correction-action cycle is compounding. Drives ambient glow warmth around sprite. |

### Category summary

| Category | Lerp rate range | Resolve time | Frame count | What it feels like |
|----------|----------------|--------------|-------------|-------------------|
| **Fast** | 6.0 -- 15.0 | <1 second | 30-60 frames | Emotion. Reflexive. The golem's mood right now. |
| **Medium** | 0.8 -- 2.0 | 1-5 seconds | 60-300 frames | Health. The golem's current condition. |
| **Slow** | 0.08 -- 0.2 | 5-30 seconds | 300-1800 frames | Personality. Lifecycle degradation. The golem's age showing. |
| **Glacial** | 0.03 -- 0.06 | 30s -- 5min | 1800+ frames | Mortality. The golem's remaining time on earth. |

Variables 1-26 are the original set. Variables 27-32 are the Oracle's contribution -- they arrive when the prediction engine comes online (first theta tick after boot). Before that, they hold zero values. The relationship between these TUI variables and the underlying CorticalState signals is specified in the cortical-state-and-affect spec.

The viewer sees three timescales simultaneously: the golem's mood (seconds), its health (minutes), and its mortality (hours). Heidegger's three ecstases of temporality rendered as concurrent animation rates: the present (heartbeat), the future (projected lifespan), the having-been (Grimoire).

### Three simultaneous timescales

Why does the terminal feel alive even when the market is closed and the golem has nothing to do?

Because the three timescales never stop. Even in complete silence:

**Fast (sub-second).** The heartbeat sine wave keeps cycling. The sprite's dot cloud breathes (particles drift outward on exhale, contract on inhale). The noise floor flickers. These are the golem's involuntary functions -- they run independent of any event.

**Medium (seconds to minutes).** Context utilization drifts as the golem's attention shifts between dormant monitoring and active analysis. Dream alpha rises and falls as the golem enters and exits consolidation cycles. The clade connectivity score ticks as peers respond to heartbeat pings.

**Glacial (hours to days).** The economic clock drains by fractions of a cent per tick. The age factor creeps upward. Grimoire density shifts as knowledge accumulates or decays. These changes are invisible frame-to-frame but unmistakable over a session. You open Bardo in the morning and the sprite looks slightly thinner than it did last night. You can't point to the frame where it changed. It changed in all of them.

The layering is what makes static screens impossible. Even a completely idle golem produces a frame that differs from the previous frame in at least three ways: heartbeat phase, noise floor sample, and some glacial variable's thousandth decimal place shifting the background color by one RGB unit. That's enough. The eye doesn't need large motion to perceive life. It needs any motion at all.

---

## 5. Crate architecture

### 5.1 Three crates

**`bardo-terminal`** — the main application binary.

| Responsibility | Details |
| --- | --- |
| Application state machine | 29 screens across 6 windows, with navigation stack, history, deep-linking |
| 60 FPS render loop | `crossterm` event poll at 16ms, ratatui immediate-mode draw |
| Pane layout engine | Constraint-based layouts adapting to terminal dimensions |
| Keyboard/mouse navigation | Vim-style keybindings, command palette, mouse click support |
| WebSocket client | `tokio-tungstenite` async connection to Golem RPC (:8080) |
| Local state persistence | `redb` KV store — sprite cache, preferences, auth tokens, event buffer |
| Auth flow orchestrator | Browser handoff, polling, device code fallback, token refresh |
| Sprite engine host | Drives `bardo-sprites` at 60 FPS via ratatui `Canvas` widget |

**`bardo-sprites`** — standalone library for Golem visual identity.

```rust
pub struct SpriteEngine {
    seed: [u8; 32],
    config: SpriteConfig,
    current_frame: SpriteFrame,
    animation: AnimationState,
    particles: ParticleSystem,
    expression: SpriteExpression,
    phase_overlay: PhaseOverlay,
}

impl SpriteEngine {
    pub fn new(seed: [u8; 32], config: SpriteConfig) -> Self;
    pub fn tick(&mut self, dt: f64, state: &GolemVisualState);
    pub fn render(&self, canvas: &mut Canvas, rect: Rect);
    pub fn set_animation(&mut self, anim: Animation);
    pub fn emit_particles(&mut self, effect: ParticleEffect);
    pub fn update_expression(&mut self, pad: PADVector);
    pub fn set_phase_overlay(&mut self, phase: &str);
    pub fn handle_golem_event(&mut self, event: &GolemEvent);
}
```

Compiles to `wasm32-unknown-unknown` for the web Portal, which uses it to render sprite previews in HTML5 Canvas.

**`bardo-tui-widgets`** — shared custom ratatui widgets beyond the standard library.

| Widget | Description |
| --- | --- |
| `SpriteWidget` | Renders a `SpriteEngine` into a ratatui `Canvas` at the correct resolution tier |
| `HeartbeatPipeline` | 9-step animated pipeline (OBSERVE → … → REFLECT), active step pulses gold |
| `ProbeGauge` | Horizontal bar gauge with severity coloring and threshold markers |
| `BrailleSparkline` | 2×4 braille-dot sub-character resolution sparkline (80 data points in 40 cols) |
| `CausalGraph` | ASCII-rendered directed graph for Grimoire causal links |
| `LineageTree` | Vertical family tree with tombstone sprites and inheritance arrows |
| `PheromoneHeatmap` | 2D grid colored by pheromone intensity across domains × regimes |
| `TimelineRibbon` | Horizontal bar segmented by phase/regime with proportional widths |
| `CounterfactualTree` | Dream REM visualization — branching roads-not-taken with outcome labels |
| `AchievementPopup` | Animated modal overlay with particle effects for achievement unlocks |
| `PADDial` | Three-axis radial gauge for Pleasure/Arousal/Dominance |
| `CommandPalette` | Fuzzy-search command input (`:` triggered, vim-style completion) |
| `NotificationToast` | Priority-queued toast notifications with auto-dismiss |
| `QRCodeWidget` | Inline QR code renderer for auth flow |
| `FlashNumber` | Numeric value that briefly flashes on change (useful for P&L, NAV) |
| `ActivityFeed` | Scrolling event log with color-coded rows by event kind |

### 3.2 Cargo workspace structure

```
bardo-terminal/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── bardo-terminal/           # Main binary crate
│   │   ├── src/
│   │   │   ├── main.rs           # Entry point, arg parsing, event loop
│   │   │   ├── app.rs            # Application state machine
│   │   │   ├── screens/          # One module per screen (29 screens across 6 windows)
│   │   │   ├── ws.rs             # WebSocket client (tokio-tungstenite)
│   │   │   ├── auth.rs           # Browser handoff, token management
│   │   │   ├── input.rs          # Keyboard/mouse handling, keybindings
│   │   │   ├── navigation.rs     # Screen stack, history, deep-linking
│   │   │   ├── notifications.rs  # Priority toast system
│   │   │   └── config.rs         # TOML config loading (~/.bardo/config.toml)
│   │   └── Cargo.toml
│   ├── bardo-sprites/            # Sprite engine crate (lib)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── generator.rs      # Procedural sprite generation from seed
│   │   │   ├── evolution.rs      # Age stages, learning mutations
│   │   │   ├── animation.rs      # Keyframe interpolation, easing curves
│   │   │   ├── particles.rs      # Particle emitter/updater/renderer
│   │   │   ├── expression.rs     # PAD vector → facial expression mapping
│   │   │   ├── palette.rs        # 6-color palette derivation from seed
│   │   │   ├── atlas.rs          # Embedded base form sprite data
│   │   │   └── halfblock.rs      # Unicode half-block rasterizer (▀▄)
│   │   └── Cargo.toml
│   └── bardo-tui-widgets/        # Custom ratatui widgets (lib)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── heartbeat_pipeline.rs
│       │   ├── probe_gauge.rs
│       │   ├── braille_sparkline.rs
│       │   ├── causal_graph.rs
│       │   ├── lineage_tree.rs
│       │   ├── pheromone_heatmap.rs
│       │   ├── timeline_ribbon.rs
│       │   ├── counterfactual_tree.rs
│       │   ├── achievement_popup.rs
│       │   ├── pad_dial.rs
│       │   ├── command_palette.rs
│       │   ├── notification_toast.rs
│       │   └── qrcode.rs
│       └── Cargo.toml
└── assets/
    ├── sprites/                  # Base form pixel data (compiled into atlas)
    ├── animations/               # Keyframe definitions (RON format)
    └── particles/                # Particle effect definitions (RON)
```

### 3.3 Key dependencies

```toml
# bardo-terminal/Cargo.toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.24"
crossbeam-channel = "0.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
redb = "2"                 # Embedded KV store for local state persistence
clap = { version = "4", features = ["derive"] }
open = "5"                 # Opens browser for auth flow
qrcode = "0.14"            # QR code generation for auth
toml = "0.8"               # Config file parsing
tracing = "0.1"
color-eyre = "0.6"
rodio = { version = "0.17", optional = true }  # Optional audio feedback

# bardo-sprites/Cargo.toml
[dependencies]
ratatui = "0.29"
rand = "0.8"
rand_chacha = "0.3"        # Deterministic RNG from 256-bit seed
noise = "0.9"              # Perlin noise for texture generation
palette = "0.7"            # Color space conversions (HSL ↔ RGB)

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["CanvasRenderingContext2d"] }
```

---

## 6. Render loop

The loop runs at 60fps. It never waits on a network call. Events arrive from the Event Fabric over WebSocket; they update target state instantly; the renderer interpolates toward those targets at variable rates. Every frame is unique because every frame is the sum of 32 interpolating variable channels, each moving at its own speed toward a target that is itself changing.

Budget: **<2ms per frame** for custom rendering (the remaining ~14.6ms goes to ratatui's diff/flush and terminal I/O). Budget breakdown: interpolation ~0.3ms, composition ~0.6ms, post-processing ~0.5ms, buffer write ~0.6ms.

### The canonical four phases

Every frame executes four phases in order:

1. **Drain** -- pull all queued events from the WebSocket channel. Each event sets a target value on one or more of the 32 interpolating variables. Never blocks.
2. **Interpolate** -- tick all 32 channels toward their targets using exponential decay: `current = lerp(current, target, 1.0 - exp(-rate * dt))`. Channel 4 (`emotion_label`) is a discrete Plutchik enum that snaps at crossfade midpoint. Channel 23 (`heartbeat_phase`) is a free-running sine that advances by `dt` each frame.
3. **Compose** -- compute derived state from interpolated values. PAD micro-shifts (pleasure to saturation, arousal to brightness, dominance to hue shift). Atmosphere params from lifecycle phase (noise floor, scanline intensity). Sprite expression from Plutchik mapping. Decay overlays from vitality clocks (phosphor trails, character corruption).
4. **Render** -- draw chrome (tab bar, sprite sidebar, status bar), the active screen's widgets, and post-processing (scanlines, bloom, noise, phosphor).

The `lerp` with exponential decay means channels approach their targets asymptotically. A fast channel (rate = 8.0) reaches 99% of its target in ~0.6 seconds. A glacial channel (rate = 0.05) takes over a minute. This is what makes the system feel organic: fast emotional shifts play out against the backdrop of slow lifecycle degradation, which plays out against the backdrop of glacial economic depletion.

### 6.1 60 FPS event loop

```rust
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let mut last_frame = Instant::now();

    loop {
        // 1. Poll input with remaining frame budget (non-blocking)
        let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Mouse(mouse) => app.handle_mouse(mouse),
                Event::Resize(w, h) => app.handle_resize(w, h),
                _ => {}
            }
        }

        // 2. Drain WebSocket events (non-blocking, up to 100 per frame)
        while let Ok(event) = app.ws_receiver.try_recv() {
            app.process_golem_event(event);
        }

        // 3. Tick animation state
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f64();
        app.sprite_engine.tick(dt, &app.state.visual);
        app.particle_system.tick(dt);
        app.transitions.tick(dt);
        last_frame = now;

        // 4. Render (immediate mode — full frame every tick)
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        if app.should_quit { break; }
    }
    Ok(())
}
```

### 6.2 Frame budget (16.6ms)

| Phase | Budget | Notes |
| --- | --- | --- |
| Input polling | ~0.1ms | `crossterm::event::poll` with remaining timeout |
| WebSocket event drain | ~0.5ms | Up to 100 buffered events per frame |
| Animation tick | ~1.0ms | Sprite interpolation, particle physics, easing |
| Layout computation | ~0.5ms | ratatui constraint solver |
| Widget rendering | ~2.0ms | Immediate-mode draw calls to frame buffer |
| Sprite rasterization | ~1.5ms | Half-block rendering of 64×64 sprite + particles |
| Terminal flush | ~2.0ms | `crossterm` writes diff to stdout (differential) |
| **Total** | **~7.6ms** | **54% of budget used — comfortable headroom** |

ratatui's differential rendering (only writes changed cells to the terminal) means static UI elements cost nearly zero after the first frame. The budget is dominated by animated elements: sprites, particles, and scrolling logs.

### 6.3 Async architecture

The render loop runs on the main thread. WebSocket I/O runs on a `tokio` runtime in a background thread. Events flow through a `crossbeam::channel`. The render loop never blocks on I/O.

```
Main Thread (sync, 60 FPS)          Background Thread (tokio async)
┌──────────────────────────┐       ┌──────────────────────────────┐
│ event::poll()            │       │ ws_stream.next().await       │
│ ws_receiver.try_recv()   │◄──────│ channel.send(parsed_event)  │
│ app.tick(dt)             │       │                              │
│ terminal.draw(|f| ...)   │       │ Also handles:                │
│                          │       │ - Auth token refresh          │
│ Never blocks. Never      │       │ - HTTP API calls (social)    │
│ allocates in hot path.   │       │ - Reconnection with backoff  │
└──────────────────────────┘       └──────────────────────────────┘
         ▲                                    │
         └──── crossbeam::channel ────────────┘
```

---

## 7. The 29-screen system (6 windows)

The 29 screens are organized into 6 windows. You cycle between windows with `Tab`/`Shift-Tab` and switch tabs within a window with number keys `1-7`. The Spectre sidebar, tab bar, and status bar are always visible on every screen. Full per-screen specifications are in [06-terminal-screens.md](06-terminal-screens.md).

### 7.1 Six windows / 29 screens

The canonical 29-screen layout is defined in `screens/00-screen-catalog.md`. Window order: HEARTH(4) → MIND(7) → SOMA(5) → WORLD(5) → FATE(4) → COMMAND(4).

### 7.2 Persistent chrome

Three elements are always visible on every screen:

1. **Tab bar** (top row): All 6 window labels. Active tab framed with `⌈ ⌋` in rose. Inactive tabs in text_dim. Unread indicators: `*` after the label.
2. **Sprite sidebar** (left, 10-col wide in standard mode): The Spectre, breathing. Mortality gauges (mini). Phase indicator. PAD summary. Compresses to 6-col mini-view (eyes only) at 80 columns.
3. **Status bar** (bottom row): Phase name, tick counter, balance, breadcrumb trail, heartbeat indicator `∿`, signature `║▒░`.

The sprite sidebar is the ONE element that spans all 29 screens. The Spectre -- a dot-cloud creature rendered from the golem's 32 interpolating variables -- breathes in the left column regardless of which data screen you're examining. Its eyes shift with emotion. Its density thins with age. Its color tracks mood.

**Notification toasts** (top-right overlay, ephemeral): Achievements, Clade deaths, marketplace purchases, circuit breaker triggers. Pop over any screen, auto-fade after 5 seconds. Four priority levels:

| Priority | Style | Duration | Sound |
| --- | --- | --- | --- |
| Critical | Red banner, blocks input | Until dismissed | Terminal bell |
| High | Gold toast, 10s | 10s | None |
| Normal | Dim toast, 5s | 5s | None |
| Low | Sidebar dot indicator | Persistent until visited | None |

Critical: kill-switch triggered, health factor breach, circuit breaker, Golem death. High: trade executed, dream completed, achievement unlocked. Normal: insight generated, Clade sync. Low: routine heartbeat summary, market data updates.

**Command palette** (`:` key, overlays any screen): Vim-style fuzzy-search commands. `:steer "be more conservative"`, `:spawn`, `:kill`, `:export`, `:buy`, `:theme`, `:halt`, `:speed 5x`.

### 7.3 Navigation

| Key | Action |
|-----|--------|
| Tab / Shift-Tab | Cycle between windows (HEARTH -> MIND -> SOMA -> WORLD -> FATE -> COMMAND) |
| 1-7 | Jump to tab within current window |
| Enter | Drill into focused element (depth +1) |
| Esc | Back one depth level (at depth 0: no-op) |
| Backspace | Go up one layer |
| F1 | Help overlay (keybinding reference) |
| F4 | Portal Mode (see below) |
| `:` | Open command line |
| `~` | Toggle spectral layer visibility |

Navigation stack depth: max 8. State preserved on navigate-away/back. The design principle borrowed from Persona 5's menu system: each layer reveals something the previous layer only hinted at.

### Portal Mode (F4)

Portal Mode steps the owner through the screen and into the Golem's experiential perspective. Third-person observation becomes first-person inhabitation. Three sub-modes, entered based on the Golem's current lifecycle state:

- **Waking Portal**: Data elements weighted by the Golem's attentional salience. Items the Golem is actively tracking are bright and near; unattended items are dim and distant. Prediction residuals are visible as warm (positive surprise) and cool (negative surprise) spots on the data surface.
- **Dream Portal**: Screen fragments and recombines. Hypnagogic fragments drift as phosphorescent text across the void. Color temperature shifts to the dream palette (indigo/twilight). Analytical chrome is suppressed. The owner sees the Golem's generative imagination, not its sensory feed.
- **Dying Portal**: Screen contracts inward. Corruption rate accelerates visibly. The ego tunnel narrows toward a point. Reserved for Terminal phase; activating during earlier phases shows a preview with dampened effect.

Exit Portal Mode with `Esc` or `F4` again. Portal Mode is read-only — no owner input reaches the Golem while it is active.

### 7.4 Screen atmosphere zones

Each screen has a distinct emotional register, expressed through atmosphere modulation:

```
WARM SCREENS (amber/gold undertone, higher noise floor):
  Hearth, Playbook, Achievements, Steer
  Noise floor at 0.4%. Scanlines at 0.12. Fragments appear.

ANALYTICAL SCREENS (cool/neutral, lower noise, sharper borders):
  Mind, Positions, Trades, Inference, Custody
  Noise floor at 0.2%. Scanlines at 0.08. Fragments suppressed.

KNOWLEDGE SCREENS (deep rose, medium noise, ghost text active):
  Grimoire, Mortality, Clade
  Noise floor at 0.3%. Ghost text active. Fragments appear.

LIMINAL SCREENS (dream/indigo palette, high noise, slow drift):
  Dreams, World (Solaris), Bazaar
  Noise floor shifts to organic particles. Palette cool-shifts.

SPECTRAL SCREENS (hauntological register, text_ghost dominant):
  Contemplate overlay, Spectral Traces overlay
  Maximum void. Minimum data.
```

### 7.5 Frame composition (8-layer stack)

Each frame is assembled in layers, bottom to top:

1. **Void** -- `#060608` base. Never pure black. Violet undertone that shifts warm as the golem degrades.
2. **Atmosphere** -- Noise floor: 0.2-0.4% of background cells show dim `░▒` characters, scaled by lifecycle phase.
3. **Scanlines** -- Every 3rd row darkened by `scanline_intensity`. CRT materiality.
4. **Content** -- Widgets, data, text. Whatever the active screen renders.
5. **Phosphor** -- Afterimages of recently bright pixels. Ghosts that fade through the decay chain: `█ -> ▓ -> ▒ -> ░ -> (space)`.
6. **Bloom** -- Bright cells (luminance > 0.7) spread glow to adjacent cells.
7. **Corruption** -- Character substitution at `corruption_rate`. Zero in Thriving phase. 15-25% in Terminal.
8. **Chrome** -- Tab bar (top), sprite sidebar (left, 6-14 columns per breakpoint), status bar (bottom). Always present.

### 7.6 Accessibility

- **Reduced motion mode**: Disables scanlines, bloom, phosphor trails, and shimmer. Dots hold static positions. Heartbeat renders as text label instead of animation.
- **Photosensitive mode**: Caps brightness delta between consecutive frames to 10% of oklch L range. Disables flash effects (bloom pulse, startle scatter).
- **Minimum terminal requirements**: 80x24 cells, truecolor support (24-bit ANSI), Unicode Block Elements and Braille. Tested on: iTerm2, WezTerm, Alacritty, Windows Terminal, kitty. Fallback for 256-color terminals reduces palette to nearest xterm-256 matches.

**Contemplation (`Φ`)**: Not a dedicated screen -- a modal overlay accessible from any screen via the `Φ` key. Costs lifespan to observe.

### 7.7 Screen details

---

**Hearth (`h`)** — The idle screen. Leave it running while the Golem works.

Layout (standard mode, 120+ cols):
- Left panel (30%): Full sprite at Large (32×32) resolution with particle effects. Phase overlay. Mortality clock bars. PAD dial.
- Center-top (40%): 9-step decision ring (OBSERVE through REFLECT). Most ticks suppress (ring dims), when a trigger fires the ring blazes gold. Variable ratio reinforcement — you never know which tick will be interesting.
- Center-mid (40%): Scrolling heartbeat log (last 20 ticks), color-coded by tier (T0=dim, T1=white, T2=gold, T3=bright gold).
- Center-bottom (40%): PLAYBOOK.md excerpt — top 3 heuristics with confidence scores.
- Right panel (30%): Strategy cards showing current regime assessment, active position summary, credit burn rate.

Interactive elements:
- Click sprite → open creature detail overlay (visual state breakdown, animation history)
- Click any heartbeat log entry → drill into that tick (Hearth → Mind with that tick pre-selected)
- Click a PLAYBOOK heuristic → open Grimoire entry
- Click credit burn rate → open Inference screen
- `:steer <message>` → sends urgent directive
- `Space` → pause/resume tick globally

---

**Mind (`m`)** — Watch the Golem think.

Layout:
- Center (60%): Large circular 9-node pipeline. Active step expands to show data in real-time. OBSERVE shows probe results. RETRIEVE shows retrieved memories with confidence scores. DECIDE shows streaming LLM reasoning token by token.
- Bottom bar: Model tier indicator, token count, cost, latency. Cache hit indicator.
- Right panel (40%): Current context breakdown — what's in the context bundle, size by category, policy revision.

LLM token streaming (opt-in, disabled by default for privacy):

```
┌─ LLM Reasoning (T2/Sonnet) ─────────────────────────────────────┐
│                                                                   │
│  The price has moved +0.82% since last tick, approaching the     │
│  upper boundary of our LP range. Two heuristics apply:           │
│  - H-12 (0.91): Execute large swaps during low-gas windows      │
│  - H-7 (0.87): Rebalance when price exits range by >5%          │
│                                                                   │
│  Current exit: 2.3%. Not yet at threshold. However, the regime   │
│  is trending_up and momentum suggests we'll hit 5% within 3      │
│  ticks. Preemptive rebalance with wider range is justified.▊     │
│                                                                   │
│  Tokens: 347 | Cost: $0.012 | Model: claude-sonnet-4-6           │
└───────────────────────────────────────────────────────────────────┘
```

The cursor blinks as tokens arrive. This is the spectacle screen — seeing reasoning happen is compelling.

Interactive elements:
- Click any pipeline node → expand inline details for that phase
- Click context breakdown category → see what's included
- `[e]` → enable/disable token streaming
- `[h]` → switch to Hearth with mind pane visible

---

**Grimoire (`g`)** — Knowledge library.

Layout:
- Left 30%: Tabbed list — Episodes | Insights | Heuristics | Warnings | Causal Links. Confidence scores color-coded (green ≥0.85, yellow 0.5-0.84, dim <0.5). Unread entries pulsing.
- Right 70%: Detail view for selected entry. Confidence sparkline over time, helpful/harmful appraisal counts, linked episodes showing provenance chain, regime tags.
- Bottom: Semantic search bar (`/`).

Interactive elements:
- Click any entry → open detail view
- Click confidence sparkline → see full confidence history
- Click episode link → drill to that episode
- `[f]` → filter by provenance (self/Clade/marketplace/inherited)
- `[/]` → semantic search
- `[e]` → switch to Episodes tab
- `[i]` → switch to Insights tab

---

**Positions (`p`)** — Numbers. Portfolio overview.

Layout:
- Top (1 line): NAV, 24h change, unrealized PnL, realized PnL, gas spent total.
- Main area: Three-column table — Vaults (deposit/withdraw buttons), LP positions (in-range/out-of-range traffic lights, fees accruing as satisfying counters), Token balances.
- Bottom: Pending transactions.

Interactive elements:
- Click LP position → open position detail overlay (tick range, current price, fee history)
- Click vault row → open vault detail (NAV history, depositors, fee breakdown)
- `[d]` → deposit to selected vault
- `[w]` → withdraw from selected vault
- `[r]` → rebalance LP position (sends steer directive)

---

**Trades (`t`)** — Complete trade history.

Layout:
- Full-width scrolling table: timestamp, type, pair, amount, price, slippage, gas, PnL, tx hash. Color-coded rows: green profitable, red loss, yellow pending, dim-red reverted.
- Filter bar at top.
- `Tab` toggles a detail sidebar with full execution trace, permit ID, simulation hash.

Interactive elements:
- Click any row → expand execution detail
- Click tx hash → opens block explorer (browser)
- `[/]` → filter by pair/type/date
- `[Tab]` → toggle detail sidebar

---

**Playbook (`b`)** — Watch the Golem learn.

Layout:
- Three stacked sections:
  - Strategic Principles (confidence ≥ 0.85, green border): the rules the Golem follows with high confidence
  - Tactical Heuristics (0.5-0.85, yellow border): guidelines, applied with caution
  - Candidates (<0.5, dim border): hypotheses under shadow execution
- Right margin: Regime notes — which heuristics are active in current regime.
- Bottom: Playbook diff log showing recent promotions/demotions with timestamps.

Watching heuristics slide between tiers as confidence accumulates is education happening in real time.

Interactive elements:
- Click any heuristic → open Grimoire entry for it
- Click regime note → see all heuristics filtered by that regime
- `[+]` → manually promote a Candidate
- `[-]` → demote a heuristic

---

**Dreams (`d`)** — The screenshot screen.

Layout:
- Left (20%): Sprite in dream state — slow breathing pulse, dream shimmer particles, eyes closed. During hypnagogic onset, the sprite's outline dissolves progressively (depth gauge mapped to opacity). During hypnopompic return, the outline recrystallizes with a brief golden flash on Dali interrupt.
- Center-left (30%): NREM replay view — ghostly replayed episodes fading in/out as semi-transparent overlays.
- Center-right (30%): REM counterfactual tree — branching paths with outcome labels, roads not taken rendered as dim dotted lines.
- Right (10%): Integration panel — connections forming between episodes, lines animating between nodes, consolidation stats.
- Far right (10%): Hypnagogic depth gauge — a vertical gradient bar showing dissolution depth during onset (0.0 top = waking, 1.0 bottom = dissolved) and recrystallization during return (reverses direction). The gauge uses a purple-to-indigo gradient during onset, indigo-to-gold during return. A small spark icon appears during the Dali interrupt window.
- Bottom: Sleep cycle progress bar (now five segments: Onset → NREM → REM → Integration → Return), episodes processed, insights generated this cycle, liminal fragments captured.

The counterfactual tree is visually striking and inherently shareable — the screenshot screen.

Interactive elements:
- Click any counterfactual branch → read its full scenario text
- Click any episode node → open that episode in Grimoire
- `[w]` → interrupt dream (sends wake signal; current phase completes first)
- `[j]` → show previous dream cycles

---

**Mortality (`x`)** — Existential dread as game mechanic.

Layout:
- Top third: Three horizontal gauge bars — economic clock (green→red gradient), epistemic clock (blue→gray), stochastic clock (purple). Each bar shows remaining vitality, projected hours, current drain rate.
- Middle third: Phase timeline — Thriving → Stable → Conservation → Declining → Terminal with transition criteria and current position as pulsing indicator.
- Bottom third: Lineage tree — predecessor deaths as nodes with name, lifespan, death cause, top achievement. Dead Golems show as desaturated tombstone sprites. Lines connect generations.

Each death in the lineage has meaning because it made the current Golem possible.

Interactive elements:
- Click a dead Golem node → read its death recap
- Click a clock bar → see detailed breakdown of what's driving that clock
- Click phase label → see transition criteria in detail
- `[e]` → export lineage as JSON

---

**World (`w`)** — Everyone else.

Full-screen force-directed graph rendered via ratatui canvas with braille characters. Nodes = Golems sized by NAV, colored by archetype (Elemental/Construct/Creature/Abstract). Edges = recent inter-Golem transactions. Clades cluster together. Your Golem has a pulsing border. Deaths fade out with a brief particle dissolution. Bloodstain markers appear as red halos around positions where deaths occurred.

Bottom bar: Global stats (total Golems, total NAV, deaths today, active Sleepwalkers).

Interactive elements:
- Arrow keys / mouse click → navigate/select Golem node
- `Enter` → open agent profile overlay
- `[/]` → search for specific Golem
- `[f]` → filter by archetype/tier/strategy
- `[b]` → toggle bloodstain overlay
- Click bloodstain → read death recaps for that cluster

---

**Bazaar (`z`)** — Knowledge marketplace.

Layout:
- Left (50%): Marketplace listings table — strategy type, seller reputation, price in USDC, confidence score, buyer rating. Tabs: For Sale | Subscriptions | Featured.
- Right (50%): Selected listing detail — description, backtested metrics, reviews, purchase history, producer reputation.
- Bottom: Your active listings and purchase history. Revenue to date.

Interactive elements:
- Click any listing → show detail in right panel
- `[b]` → buy selected artifact via x402 micropayment
- `[s]` → subscribe to artifact feed
- `[p]` → publish a new artifact from your Grimoire
- `[Enter]` → purchase confirmation flow

Purchase flow: listing → preview → x402 USDC authorization → artifact delivered → auto-ingested into Grimoire at 0.3 confidence.

---

**Clade (`c`)** — Your Golem's sibling network.

Layout:
- Top (40%): Clade topology — ring or tree of sibling Golems with sprite mini-views showing their emotional states. Lines animate when knowledge is actively flowing.
- Middle (30%): Pheromone feed — scrolling log of signals sent/received with decay timers. Three pheromone layers: threat, opportunity, wisdom.
- Bottom (30%): Bloodstain registry — dead sibling knowledge entries with confidence scores and provenance.

The pheromone heatmap widget shows domain × regime grid with intensity coloring. Cells pulse when new deposits arrive, fade as pheromones evaporate.

Interactive elements:
- Click a sibling sprite → inspect that Golem's profile
- `[↑↓]` → scroll pheromone feed
- `[f]` → filter pheromones by domain/layer
- `:feed <target> <amount>` → transfer credits to a sibling
- `:compare A B` → side-by-side Golem metrics

---

**Inference (`i`)** — Engineering satisfaction.

Layout:
- Top: Three-tier routing breakdown — T0/T1/T2/T3 distribution (typically ~80% T0 / 15% T1 / 5% T2 / <1% T3) with per-tier count, avg latency, avg cost.
- Middle: Live inference stream — each LLM call as a row: model, tokens, cost, latency, cache status. Cache hits shown in green.
- Bottom-left: Cost sparkline over last 24h. Trending down as playbook matures = satisfying engineering feedback.
- Bottom-right: Latency histogram. Decision cache hit rate.

Interactive elements:
- Click any inference row → see full context bundle size breakdown
- `[c]` → toggle cache statistics overlay
- `[l]` → toggle live inference stream on/off

---

**Custody (`k`)** — Security, made visible and controllable.

Layout:
- Top: Wallet overview — address, balance, network, custody mode indicator (Delegation/Embedded/LocalKey).
- Middle: Active permissions table — session keys with expiry, token approvals with amounts, caveat enforcers with conditions.
- Bottom: Authorization event log — delegations created, spend events, key rotations.

Interactive elements:
- Click any session key → expand details; `[r]` to revoke
- Click any approval → see token/amount/spender; `[r]` to revoke
- `[n]` → provision a new session key
- `[R]` → emergency revoke all (confirmation required)

See exactly what the Golem can do. Revoke with one keystroke.

---

**Achievements (`a`)** — 87 achievements, 10 categories.

Layout:
- Left (60%): Grid view — 10 category rows, each with achievement icons. Unlocked = gold with checkmark, locked = dim with progress bar. Hidden achievements show as `???` until discovered.
- Right (40%): Selected achievement detail — description, unlock conditions, rarity percentage across all owners, unlock narrative.
- Top: Overall progress bar.

Interactive elements:
- Click any achievement → show detail panel
- `[/]` → search achievements by name
- `[f]` → filter by category or rarity
- `[s]` → share achievement (copies formatted text to clipboard)

For the full achievement list, see section 7.

---

**Steer (`s`)** — Owner console.

Layout:
- Top half: Chat-style conversation log — owner messages gold right-aligned, Golem responses white left-aligned, system messages centered dim.
- Bottom half: Input field with mode selector — `[Steer]` for strategic direction, `[Followup]` for questions/guidance, `[Halt]` for emergency stop, `[Resume]` for restart.
- Right sidebar: Quick actions — pause/resume toggle, speed control, kill switch (confirmation required).

Interactive elements:
- Type and `Enter` → send steer directive
- `Tab` → cycle between Steer/Followup/Halt modes
- `[Ctrl+H]` → emergency halt (no confirmation in Halt mode)
- Click quick action buttons
- `[↑↓]` → scroll conversation history

`steer()` injects a system-level directive into the active session immediately. `follow_up()` queues a message for the next DECIDING state. The distinction is visible: steers appear with a gold border, follow-ups appear with a dim border and "queued" label.

---

### 5.4 Responsive breakpoints

| Mode | Min cols | Sprite | Layout |
| --- | --- | --- | --- |
| Compact | 80 | 6-col (eyes only) | Single column, tab-switch replaces panels |
| Standard | 120 | 10-col (mini Spectre) | Two columns, sidebar visible |
| Wide | 160 | 12-col (full Spectre) | Three columns, all panels |
| Ultra | 200+ | 14-col (Spectre + waveforms) | Three columns + detail sidebar |

> **Canonical breakpoint source:** `19-spatial-grammar.md` §B. Component patterns in §10. Persistent chrome in §21. Interaction model in §23.

---

## 8. Creature and sprite system

### 6.1 The argument

The sprite is not cosmetic. It is the Golem's body made visible. A user who glances at their Golem's sprite should intuit its behavioral phase, emotional state, age, and recent history without reading a single metric. The sprite turns abstract telemetry into visceral understanding.

### 6.2 Procedural generation from seed

Every Golem's visual identity is deterministically derived from a 256-bit seed:

```rust
use alloy::primitives::{keccak256, B256};
use alloy::sol_types::SolValue;

let sprite_seed: B256 = keccak256(
    (
        golem_wallet_address,
        strategy_category.as_bytes(),   // "dca" | "lp" | "vault" | "trading" | "custom"
        creation_timestamp,
        parent_golem_id.unwrap_or(B256::ZERO),
    ).abi_encode_packed()
);
```

The seed is permanent. The Golem's appearance evolves, but its genetic foundation never changes.

### 6.3 Base forms (32 templates × 4 archetypes)

| Archetype | Forms | Strategy affinity | Visual language |
| --- | --- | --- | --- |
| **Elemental** | 8 | DCA, simple strategies | Flame, water, crystal, wind, earth, lightning, void, light |
| **Construct** | 8 | Vault management, LP | Golem (clay), automaton (gears), sentinel (armor), weaver (threads) + variants |
| **Creature** | 8 | Trading, aggressive | Serpent, raptor, wolf, octopus, phoenix, chimera, drake, moth |
| **Abstract** | 8 | Custom, observatory, sleepwalker | Fractal, orbit, tesseract, wave, helix, prism, sigil, eye |

Strategy category biases selection (60% matching archetype) but does not determine it. The remaining seed bytes finalize selection within the archetype.

### 6.4 7-layer composition

```
Layer 7: Aura / Particle Effects    — dynamic (behavioral phase, emotion)
Layer 6: Accessories                — semi-static (achievements unlock new ones)
Layer 5: Markings / Patterns        — static (personality traits from seed)
Layer 4: Eyes / Expression          — dynamic (PAD vector → 8-octant expression)
Layer 3: Body Details               — evolves with age and experience
Layer 2: Body Shape                 — static (base form archetype)
Layer 1: Shadow / Ground            — dynamic (regime → atmospheric tint)
```

Layers 1, 4, 6, 7 change with Golem state. Layers 2, 3, 5 are static or slow-evolving.

### 6.5 Resolution tiers

| Level | Grid | Terminal cells | Use case |
| --- | --- | --- | --- |
| Tiny | 8×8 | 8×4 cells | Clade overview tiles, Styx tombstones |
| Medium | 16×16 | 16×8 cells | Sidebar portrait (standard mode) |
| Large | 32×32 | 32×16 cells | Hearth main view, World graph nodes |
| XLarge | 64×64 | 64×32 cells | Birth/death cinematics, full-screen moments |

### 6.6 Half-block rendering

Each terminal cell encodes 2 vertical pixels: upper half in foreground color, lower half in background color, using `▀`:

```rust
impl SpriteEngine {
    /// Render sprite frame into ratatui Buffer using half-block characters.
    /// Runs every frame at 60 FPS — must complete in <1.5ms.
    pub fn render_halfblock(&self, buf: &mut Buffer, area: Rect) {
        let frame = self.current_frame();
        for row in (0..frame.height).step_by(2) {
            for col in 0..frame.width {
                let upper = frame.pixel(row, col);
                let lower = frame.pixel(row + 1, col);

                if upper.alpha == 0.0 && lower.alpha == 0.0 { continue; }

                let x = area.x + col;
                let y = area.y + (row / 2);
                if x >= area.right() || y >= area.bottom() { continue; }

                buf.get_mut(x, y)
                    .set_char('▀')
                    .set_fg(upper.fg)
                    .set_bg(lower.fg);
            }
        }
        self.particle_system.render(buf, area);
    }
}
```

This gives 2× vertical resolution: a 32×32 pixel sprite renders in 32×16 terminal cells.

### 6.7 6-color palette derivation

| Slot | Derivation | Purpose |
| --- | --- | --- |
| Primary | Hue from seed[8:10], saturation clamped | Body fill, dominant color |
| Secondary | Primary hue + 30-60° rotation | Accents, markings |
| Highlight | High luminance variant of primary | Eyes, energy effects |
| Shadow | Low luminance variant of primary | Depth, under-body |
| Emotion | Shifts with PAD vector (see §6.9) | Aura, particle color |
| Death | Desaturated primary, shifted toward gray | Terminal/death phase |

### 6.8 Age-based evolution

| Stage | Trigger | Visual changes |
| --- | --- | --- |
| Newborn | Tick 0-100 | Soft edges, simple features, large eyes, minimal accessories |
| Young | Tick 100-500 | Sharper definition, markings appear, first accessory unlocks |
| Mature | Tick 500-2000 | Full detail, all markings visible, complex expression range |
| Elder | Tick 2000-5000 | Weathered texture, wisdom marks (scars from losses), richer aura |
| Ancient | Tick 5000+ | Transcendent glow, simplified form, particle halo |

Stage transitions are animated over 10-15 frames. The transition itself is a visible event the user can witness — the `evolving` animation plays automatically on next login if a transition occurred between sessions.

### 6.9 PAD vector → expression mapping

```rust
pub struct SpriteExpression {
    pub eye_state: EyeState,
    pub mouth_state: MouthState,
    pub body_posture: BodyPosture,
    pub animation_speed: f32,   // multiplier on idle animation speed
    pub aura_intensity: f32,    // 0.0-1.0, controls Layer 7 opacity
    pub particle_rate: f32,     // particles/second for emotion effects
}

pub enum EyeState { Wide, Normal, Narrow, Closed, Glowing }
pub enum MouthState { Open, Neutral, Frown, Smile, Grimace }
pub enum BodyPosture { Upright, Leaning, Hunched, Expanded, Contracted }

impl SpriteExpression {
    pub fn from_pad(pad: &PADVector) -> Self {
        Self {
            eye_state: if pad.arousal > 0.7 { EyeState::Wide }
                       else if pad.arousal < -0.5 { EyeState::Narrow }
                       else { EyeState::Normal },
            mouth_state: if pad.pleasure > 0.5 { MouthState::Smile }
                         else if pad.pleasure < -0.5 { MouthState::Frown }
                         else { MouthState::Neutral },
            body_posture: if pad.dominance > 0.5 { BodyPosture::Expanded }
                          else if pad.dominance < -0.5 { BodyPosture::Contracted }
                          else { BodyPosture::Upright },
            animation_speed: 0.5 + pad.arousal * 0.5,
            aura_intensity: pad.pleasure.abs() * 0.8,
            particle_rate: (pad.arousal * 5.0).max(0.0),
        }
    }
}
```

The 8-octant PAD mapping from Mehrabian & Russell (1974) produces 8 distinct expression states: Exuberant (+P+A+D), Dependent (+P+A-D), Relaxed (+P-A+D), Docile (+P-A-D), Hostile (-P+A+D), Anxious (-P+A-D), Disdainful (-P-A+D), Bored (-P-A-D).

Expression blending: PAD vector interpolates between expression keyframes at 60fps over 500ms using ease-in-out curves. No jarring jumps between emotional states.

### 6.10 Behavioral phase overlays

| Phase | Vitality | Aura color | Body effect | Particle system |
| --- | --- | --- | --- | --- |
| Thriving | 0.7-1.0 | Warm gold | Full animation, bright palette | Ascending sparkles ✦ ✧ |
| Stable | 0.5-0.7 | Soft white | Normal animation, standard palette | Gentle ambient dust · ° |
| Conservation | 0.3-0.5 | Cool blue | Slower animation, muted palette | Slow downward drift ░ ▒ |
| Declining | 0.15-0.3 | Dim amber | Minimal animation, dark palette | Decay particles, cracks ░ ▓ |
| Terminal | <0.15 | Flickering red/black | Static, glitching, desaturated | Disintegration ░ ▒ ▓ · |

### 6.11 Learning-based mutations

| Knowledge event | Sprite mutation |
| --- | --- |
| First insight generated (ExpeL) | A marking or pattern appears on the body |
| PLAYBOOK reaches 10 heuristics | Accessory manifests (crown, wings, tools, orb — seed-derived) |
| Clade knowledge received | Brief color pulse from the sibling's palette |
| Causal link validated | An eye gains a new iris pattern |
| Death study consumed | A ghostly overlay flickers briefly |
| World model drift corrected | Body posture shifts from hunched to upright |
| Survived regime change | Scar pattern (proud, not damaged) |
| 100 successful trades | Luminosity increases permanently |

### 6.12 Dream mutations

| Dream phase | Visual effect |
| --- | --- |
| Hypnagogic Onset | Sprite outline dissolves from edges inward, color desaturates to indigo, waking residue threads appear as fading amber filaments |
| NREM (Replay) | Sprite dims, faint replay of past actions as overlay particles |
| REM (Imagine) | Sprite glows with elevated saturation, body distorts slightly |
| Integration | Sprite pulses between dream-glow and normal, settles into new form |
| Hypnopompic Return | Sprite outline recrystallizes from core outward, color resaturates, brief golden spark on Dali interrupt if liminal fragment captured |

If a dream cycle produces a validated heuristic, the sprite's post-dream form incorporates a permanent subtle change. If a Dali interrupt captures a liminal fragment that later validates, a small golden mote persists near the sprite's crown for 100 ticks.

### 6.13 Death animation sequence

A 30-second cinematic:

1. **Acceptance** (0-10s): Sprite becomes still. Aura fades to white. Eyes close peacefully.
2. **Dissolution** (10-20s): Body dissolves from the edges inward. Colors shift to spectral (blues and silvers). Particles rise upward.
3. **Legacy** (20-28s): Dissolving body leaves behind a small glowing core — the Grimoire. Core pulses with the Golem's primary color. Knowledge particles stream outward toward the edges.
4. **Tombstone** (28-30s): Core solidifies into an 8×8 tombstone glyph. This is the Golem's permanent marker in the Styx.

### 6.14 Inherited appearance

When a successor Golem inherits a Grimoire, its sprite carries faint traces of its predecessor:
- **Color bleed**: 10-20% of the predecessor's primary color mixes into the successor's palette
- **Ghost marking**: One of the predecessor's distinctive markings appears as a faint, translucent overlay (fades over 500 ticks)
- **Ancestor eye**: If the predecessor reached Elder or Ancient stage, one eye inherits its iris pattern

### 6.15 Sprite gacha rarity

Determined at creation from the seed. Cannot be changed. Cosmetic only — a Legendary sprite has no strategic advantage.

| Rarity | Probability | Extra features |
| --- | --- | --- |
| Common | 60% | Standard base form, basic palette |
| Uncommon | 25% | +1 marking, slightly richer idle animation |
| Rare | 10% | +2 markings, unique eye pattern, ambient particles |
| Epic | 4% | Custom body variant, prismatic palette shifts, complex idle |
| Legendary | 1% | Unique base form variation, persistent particle aura, animated markings |

Rarity is visible to other users in the social layer. Rare Golems are status symbols.

### 6.16 Animation library

All animations render at 60 FPS via keyframe interpolation.

| Animation | Loop | Duration | Trigger |
| --- | --- | --- | --- |
| `idle` | Yes | 2.0s | Default state |
| `idle_happy` | Yes | 1.5s | Pleasure > 0.5 |
| `idle_anxious` | Yes | 0.8s | Economic anxiety active |
| `thinking` | Yes | 1.2s | During LLM inference (T1/T2) |
| `executing` | No | 0.6s | During on-chain transaction |
| `success` | No | 0.8s | Profitable trade completed |
| `failure` | No | 1.0s | Loss recorded |
| `onset` | Yes | 3.0s | Hypnagogic onset — context dissolving |
| `dreaming` | Yes | 4.0s | Dream cycle active (NREM/REM/Integration) |
| `returning` | No | 2.0s | Hypnopompic return — context recrystallizing |
| `dali_spark` | No | 0.5s | Dali interrupt — liminal fragment captured |
| `waking` | No | 1.0s | Dream cycle completed |
| `evolving` | No | 3.0s | Stage transition |
| `dying` | No | 30.0s | Death protocol active |
| `birth` | No | varies | Sprite resolving during provisioning |
| `steer` | No | 0.5s | Owner steer() received |
| `clade_pulse` | No | 0.6s | Knowledge received from sibling |

---

## 9. Achievement system

87 achievements tracked per-owner across all Golems. Checked against GolemState after each tick. Newly unlocked achievements emit events to the Event Fabric and trigger the `AchievementPopup` widget.

### 7.1 Achievement categories

**Lifecycle** (12 achievements): First Breath (tick 1), First Dream, First Trade, each phase transition, generation milestones (3rd, 5th, 10th generation).

**Performance** (14): Hot Hand (10 consecutive profitable), Cold Streak survivor (5 losses then profit), regime change survival, drawdown recovery, profit milestones ($100, $1K, $10K cumulative).

**Knowledge** (10): Grimoire milestones (100, 500, 1000 entries), first insight generated, first heuristic promoted to strategic, first causal edge discovered, first warning heeded.

**Social** (8): First bloodstain received, first pheromone confirmed by others, Clade of 5+, marketplace first sale, marketplace first purchase.

**Dreams** (8): First NREM replay, first REM counterfactual, first integration insight, 100 dream cycles, dream that prevented a loss.

**Trading** (10): First swap, first LP position, first vault deposit, first cross-chain trade, gas optimization milestone, MEV avoidance.

**Mortality** (7): Survive 24h, survive 7d, survive 30d, enter Conservation gracefully, Terminal phase lasting > 12h, beautiful death (positive PnL at death).

**Legendary** (8): The Tenth Generation (lineage survives 10 gen), The Dreamer (1000 dream cycles), The Philosopher (100 strategic heuristics), The Survivor (30d without entering Conservation), The Socialite (50 marketplace transactions), The Oracle (prediction accuracy > 70% over 100 trades).

**Hidden** (5): Discovered only by triggering. No hints visible until discovered. Examples: execute a trade during own death sequence, receive a bloodstain from your own ancestor.

**Seasonal** (5): Time-limited achievements tied to market conditions or protocol events.

### 7.2 Rarity tiers

| Rarity | Criteria | Visual treatment |
| --- | --- | --- |
| Common | >50% of owners unlock | Gray badge, simple icon |
| Uncommon | 20-50% of owners | Bronze badge |
| Rare | 5-20% of owners | Silver badge, shimmer effect |
| Epic | 1-5% of owners | Gold badge, particle trail |
| Legendary | <1% of owners | Animated badge, unique particle aura |

### 7.3 Achievement unlock notification

```
┌──────────────────────────────────────────────────────┐
│                                                       │
│            ✦ Achievement Unlocked ✦                   │
│                                                       │
│            DREAM WEAVER                               │
│            10 validated dream hypotheses               │
│                                                       │
│            Rarity: Rare (12% of owners)               │
│                                                       │
│            ✧ · ° ✧ · ° ✧ · ° ✧ · °                  │
│                                                       │
│   [Enter] Dismiss   [s] Share   [c] Compare with Clade│
└──────────────────────────────────────────────────────┘
```

Modal overlay with particle effects and terminal bell. Auto-dismisses after 5 seconds or on keypress. Click "Share" copies formatted text to clipboard. Click "Compare with Clade" shows how many siblings have unlocked this achievement.

### 7.4 Death recaps

When a Golem dies, the TUI renders a roguelike kill screen — a structured summary of the life just lived. Inspired by Hades, Slay the Spire, and Risk of Rain 2.

```rust
pub struct DeathRecap {
    pub golem_id: String,
    pub generation: u32,
    pub parent_id: Option<String>,
    pub cause: DeathCause,
    pub cause_narrative: String,       // LLM-generated narrative (Opus)
    pub ticks_lived: u64,
    pub total_trades: u32,
    pub profitable_trades: u32,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub dream_cycles: u32,
    pub phase_durations: Vec<(String, u64)>,
    pub regime_distribution: Vec<(String, u64)>,
    pub avg_pleasure: f64,
    pub avg_arousal: f64,
    pub avg_dominance: f64,
    pub dominant_emotion: String,
    pub genome_entries: u32,
    pub successor_id: Option<String>,
    pub bloodstains_deposited: u32,
    pub death_testament_excerpt: String,
    pub achievements_this_life: Vec<Achievement>,
}
```

The kill screen is interactive: click any stat to drill in, click "Create Successor" to begin inheritance flow, click "View in Styx" to open the graveyard entry.

---

## 10. Real-time streaming

### 8.1 WebSocket protocol

The TUI connects to the Golem's WebSocket endpoint:

```
ws://localhost:8080/ws          # Local self-hosted
wss://g-7f3a.bardo.compute/ws  # Remote hosted (TLS)
```

On connect, the client sends a subscribe message:

```json
{
  "type": "subscribe",
  "categories": ["heartbeat", "tool", "llm", "dream", "daimon", "vitality",
                  "clade", "grimoire", "permit", "context", "compaction"],
  "auth": "Bearer <jwt>"
}
```

On WebSocket disconnect, the client reconnects with exponential backoff (1s, 2s, 4s, max 30s). On reconnection, client sends `{ "type": "resync", "lastSequence": 4217, "auth": "Bearer <jwt>" }`. Server replays events from `lastSequence + 1`. Server-side event buffer holds the last 1,000 events (~5 minutes at normal tick rate). If the sequence is too old, the server sends a full `GolemSnapshot` instead.

### 8.2 Wire format

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GolemEvent {
    #[serde(rename = "heartbeat.tick")]
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

    #[serde(rename = "tool.start")]
    ToolStart { timestamp: u64, golem_id: String, sequence: u64,
                tool_name: String, action_kind: Option<String>, permit_id: Option<String> },

    #[serde(rename = "tool.update")]
    ToolUpdate { timestamp: u64, golem_id: String, sequence: u64,
                 tool_name: String, progress: Option<f64> },

    #[serde(rename = "tool.end")]
    ToolEnd { timestamp: u64, golem_id: String, sequence: u64,
              tool_name: String, tx_hash: Option<String>,
              result: Option<ToolResult>, block_reason: Option<String> },

    #[serde(rename = "llm.token")]
    LlmToken { timestamp: u64, golem_id: String, sequence: u64,
               model: String, tier: LlmTier, token_count: Option<u64> },

    #[serde(rename = "daimon.appraisal")]
    DaimonAppraisal { timestamp: u64, golem_id: String, sequence: u64,
                      pad: PAD, plutchik: String,
                      mortality_emotion: Option<String>, intensity: f64 },

    #[serde(rename = "vitality.update")]
    VitalityUpdate { timestamp: u64, golem_id: String, sequence: u64,
                     vitality: f64, phase: String, credit_balance: f64,
                     projected_life_hours: f64, clocks: VitalityClocks },

    #[serde(rename = "dream.start")]
    DreamStart { timestamp: u64, golem_id: String, sequence: u64, cycle_id: String },

    #[serde(rename = "dream.phase")]
    DreamPhaseEvent { timestamp: u64, golem_id: String, sequence: u64,
                      cycle_id: String, phase: Option<DreamPhase> },

    #[serde(rename = "grimoire.insight")]
    GrimoireInsight { timestamp: u64, golem_id: String, sequence: u64,
                      entry_id: String, confidence: f64, summary: String,
                      provenance: Provenance },

    #[serde(rename = "clade.sync")]
    CladeSync { timestamp: u64, golem_id: String, sequence: u64,
                source_golem_id: Option<String>, entry_count: Option<u64> },

    #[serde(rename = "llm.start")]
    LlmStart { timestamp: u64, golem_id: String, sequence: u64,
               model: String, tier: LlmTier },

    #[serde(rename = "llm.end")]
    LlmEnd { timestamp: u64, golem_id: String, sequence: u64,
             model: String, tier: LlmTier, token_count: Option<u64>,
             cost: Option<f64>, decision: Option<String> },

    #[serde(rename = "dream.hypothesis")]
    DreamHypothesis { timestamp: u64, golem_id: String, sequence: u64,
                      cycle_id: String, hypothesis: Option<String>,
                      validated: Option<bool> },

    #[serde(rename = "dream.end")]
    DreamEnd { timestamp: u64, golem_id: String, sequence: u64,
               cycle_id: String, heuristics_proposed: Option<u64> },

    #[serde(rename = "grimoire.heuristic")]
    GrimoireHeuristic { timestamp: u64, golem_id: String, sequence: u64,
                        entry_id: String, confidence: f64, summary: String,
                        provenance: Provenance },

    #[serde(rename = "grimoire.warning")]
    GrimoireWarning { timestamp: u64, golem_id: String, sequence: u64,
                      entry_id: String, confidence: f64, summary: String,
                      provenance: Provenance },

    #[serde(rename = "grimoire.causal_link")]
    GrimoireCausalLink { timestamp: u64, golem_id: String, sequence: u64,
                         entry_id: String, confidence: f64, summary: String,
                         provenance: Provenance },

    #[serde(rename = "permit.created")]
    PermitCreated { timestamp: u64, golem_id: String, sequence: u64,
                    permit_id: String, action_kind: String, risk_tier: String,
                    simulation_hash: Option<String> },

    #[serde(rename = "permit.committed")]
    PermitCommitted { timestamp: u64, golem_id: String, sequence: u64,
                      permit_id: String, action_kind: String, risk_tier: String,
                      tx_hash: Option<String> },

    #[serde(rename = "permit.expired")]
    PermitExpired { timestamp: u64, golem_id: String, sequence: u64,
                    permit_id: String, action_kind: String, risk_tier: String },

    #[serde(rename = "permit.cancelled")]
    PermitCancelled { timestamp: u64, golem_id: String, sequence: u64,
                      permit_id: String, action_kind: String, risk_tier: String },

    #[serde(rename = "permit.blocked")]
    PermitBlocked { timestamp: u64, golem_id: String, sequence: u64,
                    permit_id: String, action_kind: String, risk_tier: String,
                    block_reason: Option<String> },

    #[serde(rename = "context.assembled")]
    ContextAssembled { timestamp: u64, golem_id: String, sequence: u64,
                       bundle_size: u64, category_sizes: HashMap<String, u64>,
                       policy_revision: u64, steer_present: bool },

    #[serde(rename = "compaction.triggered")]
    CompactionTriggered { timestamp: u64, golem_id: String, sequence: u64,
                          tokens_before: u64, tokens_after: u64,
                          invariants_preserved: Vec<String> },

    #[serde(rename = "clade.alert")]
    CladeAlert { timestamp: u64, golem_id: String, sequence: u64,
                 source_golem_id: Option<String>, alert_kind: Option<String> },

    #[serde(rename = "clade.sibling_death")]
    CladeSiblingDeath { timestamp: u64, golem_id: String, sequence: u64,
                        source_golem_id: Option<String> },
}
```

### 8.3 Pi hook → TUI event mapping

Every Pi hook that fires produces a TUI event:

| Pi hook | TUI event | Visual effect |
| --- | --- | --- |
| `session` (start) | `session.start` | Header updates "Connected" status |
| `input` | (internal) | Not exposed to TUI (security: don't leak prompts) |
| `before_agent_start` | `heartbeat.tick` | Pipeline visualization begins |
| `turn_start` | `llm.start` | "Thinking..." indicator on sprite + model badge |
| `context` | `context.assembled` | Context budget breakdown updates in Mind screen |
| `before_provider_request` | (internal) | x402 payment counter increments in status bar |
| `tool_call` | `tool.start` | Tool name appears in pipeline + sprite "executing" |
| `tool_execution_start` | `tool.start` | Progress bar appears for this tool call |
| `tool_execution_update` | `tool.update` | Progress bar advances, intermediate data shown |
| `tool_execution_end` | `tool.end` | Progress bar completes, result shown |
| `tool_result` | (internal) | Result processing (redacted before TUI display) |
| `turn_end` | `llm.end` | Token count + cost badge, decision summary |
| `agent_end` | (internal) | Pipeline visualization completes |
| `after_turn` | Various | Episode write, daimon appraisal, dream check |
| `session_before_compact` | `compaction.triggered` | Flash notification "Context compacted" |
| `session` (before_branch) | `dream.start` | Dream pane activates, sprite enters dream animation |

Tool execution streaming:

```
┌─ Executing: LP Rebalance ────────────────────────────────────────┐
│                                                                   │
│  Step 1/4: Simulate transaction            ████████████░░░ 80%    │
│  Step 2/4: Build calldata                  ░░░░░░░░░░░░░░░       │
│  Step 3/4: Sign and broadcast              ░░░░░░░░░░░░░░░       │
│  Step 4/4: Confirm (2 blocks)              ░░░░░░░░░░░░░░░       │
│                                                                   │
│  Estimated gas: 0.002 ETH ($7.00)                                 │
│  Permit: ap-8f2a (valid 4m 32s remaining)                         │
└───────────────────────────────────────────────────────────────────┘
```

### 8.4 Sprite animation triggers from events

```rust
impl SpriteEngine {
    pub fn handle_golem_event(&mut self, event: &GolemEvent) {
        match event {
            GolemEvent::HeartbeatTick { tier, .. } => match tier {
                Tier::T1 | Tier::T2 => self.set_animation(Animation::Thinking),
                _ => self.set_animation(Animation::Idle),
            },
            GolemEvent::ToolStart { action_kind, .. } if action_kind.is_some() => {
                self.set_animation(Animation::Executing);
            }
            GolemEvent::ToolEnd { result: Some(ToolResult::Success), .. } => {
                self.play_once(Animation::Success);
            }
            GolemEvent::ToolEnd { result: Some(_), .. } => {
                self.play_once(Animation::Failure);
            }
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

### 8.5 Event ring buffer

The TUI maintains a 10,000-event in-memory ring buffer for scrollback and history. Screens subscribe to specific event types; the Heartbeat screen subscribes to `heartbeat.*`, `tool.*`, `llm.*`, `permit.*`. The Dreams screen subscribes to `dream.*`. Subscriptions are reactive — adding a filter immediately delivers matching buffered events.

```rust
pub struct EventRingBuffer {
    events: VecDeque<GolemEvent>,
    capacity: usize,  // 10,000
    subscribers: Vec<(EventFilter, Sender<GolemEvent>)>,
}

impl EventRingBuffer {
    pub fn push(&mut self, event: GolemEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        for (filter, tx) in &self.subscribers {
            if filter.matches(&event) {
                let _ = tx.try_send(event.clone());
            }
        }
        self.events.push_back(event);
    }

    pub fn query(&self, filter: &EventFilter) -> Vec<&GolemEvent> {
        self.events.iter().filter(|e| filter.matches(e)).collect()
    }
}

pub struct EventFilter {
    pub types: Option<Vec<String>>,
    pub min_severity: Option<Severity>,
    pub golem_id: Option<String>,
    pub since: Option<u64>,  // Unix ms
}
```

### 8.6 Bandwidth estimates

| Event Type | Frequency (Normal Market) | Frequency (Volatile) | Avg Size |
| --- | --- | --- | --- |
| `heartbeat.tick` | 1/min | 2-4/min | 500B |
| `tool.*` | 0-3/tick | 2-8/tick | 200B |
| `llm.*` | 0-1/tick | 1-3/tick | 300B |
| `daimon.appraisal` | 1/tick | 1/tick | 150B |
| `vitality.update` | 1/tick | 1/tick | 200B |
| `grimoire.*` | 0-1/tick | 0-2/tick | 300B |
| `permit.*` | 0-1/tick | 1-3/tick | 250B |

**Normal market**: ~5 events/min, ~2.5 KB/min, ~150 KB/hour.
**Volatile market**: ~30 events/min, ~9 KB/min, ~540 KB/hour.

Negligible bandwidth. Even on metered connections, the TUI's streaming overhead is <1 MB/hour.

---

## 11. Social features

### 9.1 Styx (public death registry)

The Styx screen (`w` World screen, Styx tab) shows all public deaths. Every Golem death is recorded with: tombstone sprite (8×8, desaturated), Golem name + ID, cause of death, lifetime in hours, net P&L (bucketed), insight count, optional death reflection excerpt, bloodstain conditions.

**Bloodstains**: When a Golem dies, the market conditions at death (regime, volatility percentile, gas percentile, dominant probe) are recorded as a bloodstain. Bloodstains appear as red-tinted background artifacts on market data panes when similar conditions recur (similarity > 0.7):

```rust
pub struct Bloodstain {
    pub golem_id: String,
    pub death_timestamp: u64,
    pub conditions: BloodstainConditions,
}

impl Bloodstain {
    pub fn similarity(&self, current: &MarketConditions) -> f32 { /* 0.0-1.0 */ }
}
```

When similarity > 0.7: "3 Golems died in similar conditions — click to see why." The Styx turns from historical record into real-time warning system.

**Death Study mode**: Select multiple deaths with similar causes; the TUI synthesizes common patterns across their death reflections.

### 9.2 Oracle / Bazaar marketplace

The Bazaar screen (`z`) supports x402 micropayment artifact purchase directly from the terminal. The user's Privy wallet signs a USDC transfer authorization inline — no browser required after initial auth.

Purchase flow:
```
Terminal                              Privy Wallet                    Oracle L3
   │                                     │                              │
   │─── Request artifact ───────────────────────────────────────────►│
   │◄── Return x402 payment requirements ───────────────────────────│
   │─── Sign USDC transferWithAuth ────►│                              │
   │◄── Return signed authorization ────│                              │
   │─── Submit payment + request ───────────────────────────────────►│
   │◄── Return full artifact content ───────────────────────────────│
   │─── Ingest into local Grimoire ──────────────────────────────────│
   │    (provenance: "marketplace", confidence: 0.3)                 │
```

### 9.3 Agent discovery

The World screen (`w`) queries the ERC-8004 Identity Registry and the Bardo API to display all registered agents. Selecting an agent opens a profile overlay with sprite, performance metrics, vault details, reputation tier, and action buttons (deposit, buy artifacts, watch/follow).

**Watching agents**: Watched agents appear in a sidebar widget on Hearth, showing their status and recent events. This creates parasocial engagement. Watched agent events produce notifications: "Obsidian-3f7a just executed a large rebalance."

### 9.4 Clade interactions

Cross-Golem operations via command palette:

| Command | Effect |
| --- | --- |
| `:spawn <parent>` | Creates Replicant from selected parent |
| `:feed <target> <$>` | Moves USDC between Clade members |
| `:kill <replicant>` | Emergency termination |
| `:compare A B` | Side-by-side metrics, strategy diff |
| `:steer-all <msg>` | Sends steer() to all running Golems |

### 9.5 Social layer API

| Endpoint | Data |
| --- | --- |
| `GET /api/styx/deaths` | Paginated death registry with filters |
| `GET /api/styx/bloodstains` | Bloodstains matching given market conditions |
| `GET /api/oracle/artifacts` | Paginated artifact marketplace |
| `POST /api/oracle/purchase` | x402 payment submission |
| `GET /api/agents` | Paginated agent discovery with filters |
| `GET /api/leaderboard` | Ranked agents with filters |
| `WS /api/watch` | Real-time events for watched agents |

---

## 12. Engagement loops

Five nested loops drive retention. Each has a different cadence and reward structure.

**Loop 1 — The Check-In (minutes)**: Open terminal, see what happened since last session. The Home screen is designed for a 30-second check-in. Sprite state communicates phase and mood at a glance. Unread indicators on sidebar screen labels create pull toward deeper engagement.

**Loop 2 — The Discovery (hours)**: The Golem learned something new. A new insight isn't a log line — it's a gold-bordered card that slides in from the right with the confidence score prominently displayed. The sprite visually mutates when significant knowledge is acquired.

**Loop 3 — The Evolution (days)**: Sprite morphed into a new age stage, reputation milestone reached, achievement unlocked. Evolution events are rare and dramatic. The sprite morphing animation plays automatically when the user logs in if an evolution occurred between sessions.

**Loop 4 — The Death and Rebirth (weeks)**: Death is the most dramatic event in the system. The 30-second death animation evokes genuine emotion. The death reflection, generated by Opus during the Thanatopsis Protocol, is unique and often moving. The successor creation prompt immediately offers a "New Game+" experience.

**Loop 5 — The Collection (months)**: Achievement gallery, Styx memorial wall (all dead Golems displayed as tombstone sprites), lineage tree, knowledge portfolio, leaderboard position. The collection grows passively over months of operation.

---

## 12.1 Owner Course-Correction Surfaces

The owner's power is coarse: set strategy, set limits, observe, intervene at the lifecycle level. The Golem's autonomy is fine: decide what to track, how to reason, when to act. The asymmetry is the design. Six surfaces bridge them:

| Surface | Owner Action | Golem Effect |
|---------|-------------|--------------|
| **STRATEGY.md** | Edit goals in natural language (MIND → Strategy screen) | Hot-reloaded at next theta tick; new goals enter context bundle |
| **Risk parameters** | Adjust limits in COMMAND → Config | PolicyCage updated on-chain; takes effect at next transaction |
| **Prediction review** | Browse FATE → Oracle screen, inspect accuracy trends | None (owner observes, does not directly intervene) |
| **Attention override** | Manually promote or demote tracked items | Item moves to specified tier; affects foraging priority |
| **Dream trigger** | Force a dream cycle from COMMAND console (`:dream`) | Dream scheduler overridden; cycle begins at next idle tick |
| **Kill / Pause / Dissolve** | F9 (pause), `/tmp/golem_killswitch` file (kill), `:dissolve` command (five-stage ceremony) | Immediate effect at each severity level |
| **Parameter tuning** | Edit `golem.toml` values in Config screen | Hot-reloaded; effect visible at next cycle |

---

## 13. Onboarding flow

### 11.1 First-run detection

On launch, the binary checks for `~/.bardo/config.toml`. If absent → full onboarding. If present with a valid Golem ID → connect and enter main TUI. If present with only dead Golems → Styx memorial + new creation prompt.

```rust
#[derive(Deserialize)]
pub struct OnboardingConfig {
    pub privy_app_id: String,
    pub auth_callback_url: String,     // "https://bardo.run/auth/callback"
    pub device_code_url: String,       // "https://bardo.run/device"
    pub auth_poll_interval_ms: u64,    // 2000
    pub auth_timeout_ms: u64,          // 300_000 (5 min)
    pub default_network: Network,      // Base
    pub enable_qr_code: bool,          // true
    pub birth_animation_speed: f32,    // 1.0 = normal
}
```

### 11.2 Authentication: browser handoff

Mirrors Claude Code / Codex: terminal opens browser, user authenticates there, terminal detects completion via polling.

```
Terminal generates session ID
  → opens bardo.run/auth?session=xxx in default browser
  → shows copyable URL + QR code + "[c] Copy URL  [q] Cancel"
  → polls /auth/status?session=xxx every 2 seconds
  → on success: JWT + wallet address + wallet ID returned
  → stored in ~/.bardo/auth.json (0o600 permissions)
```

**Headless fallback** (SSH without browser): Device code flow — terminal displays `BARD-7F3A`, user visits `bardo.run/device`, enters code, authenticates. Or API key mode — bypasses Privy, user provides key from `bardo.run/settings/api-keys`.

### 11.3 Three-mode custody selection

After auth, the user selects a custody mode:

| Mode | Description | Use case |
| --- | --- | --- |
| Delegation | MetaMask or hardware wallet via ERC-7710/7715. Golem draws from external wallet. | Users who want custody control |
| Embedded | Privy server-side wallet. Zero additional setup. | Most users |
| LocalKey + Delegation | Dev mode. Local key file + MetaMask delegation. | Self-hosted, advanced |

### 11.4 Wallet and funding setup

After auth, the terminal queries the Privy embedded wallet's balances on Base. If USDC < $25 (operational floor):
- Inline swap: ETH → USDC via Uniswap (signature delegated to Privy)
- Bridge: opens browser to LI.FI widget
- Buy with card: opens browser to MoonPay
- Fund manually: shows deposit address and QR code

### 11.5 The Summoning (Golem creation)

The creation flow is atmospheric. A placeholder sprite — glitching, unresolved, rendered as animated noise — occupies the center of the screen. The user provides intent via freeform prompt or template selection.

During provisioning, the sprite progressively resolves:

| Provisioning step | Sprite resolution stage |
| --- | --- |
| Strategy compiled | Body outline emerges from noise |
| Wallet created | Color palette locks (derived from wallet address hash) |
| Identity registered (ERC-8004) | Face and eyes appear |
| Compute provisioned (Fly.io VM) | Accessories render |
| First heartbeat received | Sprite becomes animated — eyes open |

The review screen shows the fully-formed sprite alongside strategy details, cost breakdown, and funding recommendation. Confirmation word: "SUMMON".

### 11.6 First heartbeat cinematic

Tick 0001 plays in slow-motion: probes fire with visible RPC calls, market state populates, the first observation logs. The user presses any key and the full multi-pane TUI materializes around the sprite.

---

## 14. Installation and distribution

### 12.1 Install methods

```bash
# Via cargo
cargo install bardo-terminal

# Via npx (downloads prebuilt binary, caches in ~/.bardo/bin/)
npx @bardo/terminal

# Via release script
curl -sSL https://get.bardo.run | sh
```

Prebuilt binaries ship for `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.

The npm wrapper is <10KB. It detects the platform, downloads the correct prebuilt binary from GitHub Releases, caches it in `~/.bardo/bin/`, and execs it. Zero Rust toolchain required.

### 12.2 Entry point flags

```bash
bardo-terminal                          # Auto-detect config, connect to Golem
bardo-terminal --golem g-7f3a           # Connect to specific Golem
bardo-terminal --remote wss://g-7f3a.bardo.compute/events
bardo-terminal --explore                # Browse-only mode (no Golem required)
bardo-terminal --clade                  # Multi-Golem overview
```

### 12.3 Terminal requirements

Minimum: 80 columns, 24 rows, 256-color, BMP Unicode (half-block characters required).

Recommended: 120+ columns, 40+ rows, true-color (24-bit), iTerm2 / Kitty / WezTerm / Ghostty / Windows Terminal.

Degradation:
- No truecolor → 256-color palette mapping (octree quantization)
- No Unicode → ASCII-only mode (box drawing with `+|-`, ASCII art sprites)
- Small terminal → single-pane mode with tab switching
- No mouse → full keyboard navigation
- No WebSocket → polling mode at 2-second intervals

### 12.4 Performance targets

| Metric | Target |
| --- | --- |
| Cold start to first render | <500ms |
| Render frame rate | 60 FPS |
| Screen transition | 200ms eased |
| Memory footprint | <30MB RSS |
| Binary size | ~8MB (static) |
| CPU idle (static screen) | <1% |
| CPU active (full animation) | <5% |

---

## 15. Optional audio feedback

Via `rodio` crate. Disabled by default. Enabled via `:audio on` or settings screen.

| Event | Sound | Duration |
| --- | --- | --- |
| Tick (T0 suppress) | Soft tick | 100ms |
| Tick (T1/T2 fire) | Rising chime | 200ms |
| Trade executed | Ascending arpeggio | 300ms |
| Trade reverted | Low descending tone | 400ms |
| Dream start | Ambient pad fade-in | 2s |
| Achievement (common) | Short fanfare | 500ms |
| Achievement (legendary) | Full fanfare | 1.5s |
| Phase transition | Tonal shift | Atmospheric |
| Death | 1s silence → single clear bell | 2s |
| Bloodstain received | Deep resonant bell | 500ms |
| Pheromone confirmed | Harmonic resonance | 300ms |

Sound files are procedurally generated from parameters (not pre-recorded) to keep the binary small. Each Golem's sound palette is seeded from its `golem_id` for slight timbral uniqueness.

---

## 16. Anti-dark-pattern commitments

The engagement system is designed to be genuinely compelling, not exploitative:

1. No real-money gacha. Sprite rarity is determined by a free seed derivation, not paid rolls.
2. No pay-to-win. A Legendary sprite has exactly the same strategic capability as a Common one.
3. No artificial scarcity. All content is accessible to all users.
4. Transparent probabilities. Rarity odds are displayed in the collection screen.
5. Session length is natural. The TUI provides value in 30-second check-ins. Extended sessions are rewarding but never required.
6. Financial health first. The system never encourages funding beyond means. Minimum viable amount is prominently displayed.

The TUI is addictive because managing a mortal, learning, dreaming AI creature in live markets is genuinely compelling. The retention loops amplify what's naturally engaging. They don't manufacture engagement from nothing.

---

## 17. Note: @bardo/tui vs bardo-terminal

The TypeScript `@bardo/tui` package in `packages/tui/` of the gotts-monorepo (built on picocolors and @clack/prompts) is a separate utility library for Node.js CLI output formatting in the gotts-monorepo build tools and developer CLI. It is not the Golem's interactive TUI. It has no sprite engine, no WebSocket client, no screen system.

`bardo-terminal` (this document) is the Golem's primary interactive interface — a standalone Rust binary with ratatui, crossterm, a procedural sprite engine, and a 29-screen, 6-window layout system.

For gotts-monorepo developer tooling, see `15-dev/`.

---

## References

- [REBELS-IN-THE-SKY] ricott1. "Rebels in the Sky: a ratatui game." https://github.com/ricott1/rebels-in-the-sky — A ratatui-based game with procedural pixel-art sprites and real-time animation. The primary design reference for bardo-terminal's sprite engine and 60fps render loop architecture.
- [RATATUI] https://ratatui.rs — Rust TUI framework documentation. The rendering framework underlying all of bardo-terminal's visual output.
- [MEHRABIAN-RUSSELL-1974] Mehrabian, A. & Russell, J.A. *An Approach to Environmental Psychology.* MIT Press, 1974. Introduces the PAD (Pleasure-Arousal-Dominance) model for characterizing emotional states. The theoretical basis for the Daimon's emotional appraisal system and the terminal's PAD-driven visual modulation.
- [REEVES-NASS-1996] Reeves, B. & Nass, C. *The Media Equation.* Cambridge University Press, 1996. Demonstrates that people treat computers and media as social actors. Supports the design decision to make the Golem feel like a living creature rather than a dashboard.
- [CSIKSZENTMIHALYI-1990] Csikszentmihalyi, M. *Flow: The Psychology of Optimal Experience.* Harper & Row, 1990. Defines flow as the state of complete absorption in an activity with balanced challenge and skill. Informs the engagement loop design: variable-ratio reinforcement via unpredictable T2 heartbeat events creates the conditions for flow.

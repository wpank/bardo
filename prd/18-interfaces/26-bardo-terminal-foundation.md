# Bardo Terminal Foundation [SPEC]

> **PRD2 Section**: 18-interfaces | **Source**: MMO Research S1 v2.0
>
> **Status**: Implementation Specification
>
> **Dependencies**: `prd2/01-golem/18-cortical-state.md` (defines the 32 CorticalState signals that form the Golem's atomic perception surface), `prd2/18-interfaces/perspective/03-embodied-consciousness.md` (somatic architecture mapping terminal zones to body metaphor, PAD-driven color system), `prd2/18-interfaces/screens/03-interaction-hierarchy.md` (the 29-screen, 6-window navigation model), `prd2/18-interfaces/perspective/05-stasis-dissolution.md` (operator-controlled freeze and deliberate termination ceremonies)

> **Reader orientation:** This is the foundation document for the entire Bardo terminal specification. It belongs to the **interfaces layer** and explains what Bardo is, the perpetual motion design principle, the 60fps render loop, and the 32 interpolating variables that drive every visual element. Start here before reading any other terminal spec. Key concepts: Golem (a mortal autonomous DeFi agent compiled as a Rust binary on a micro VM), CorticalState (the 32-signal atomic shared perception surface), the Spectre (the dot-cloud creature sprite), and the Daimon (the Golem's emotional appraisal subsystem). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## What Bardo Is

Bardo is a cinematic terminal UI where autonomous financial agents -- called Golems -- live, trade, and die. You install a binary. You type `bardo`. A full-screen interface opens, rendered in the terminal at 60fps, and every pixel on screen is the downstream consequence of a variable that is always changing. A mortality clock draining. A credit balance depleting. An emotional state drifting. A market moving. The terminal breathes because the Golem breathes because the market breathes because the world breathes.

This is not a dashboard. Dashboards are static until data refreshes. Bardo is a living system where nothing is ever at rest. The difference matters: a dashboard shows you numbers; Bardo shows you a creature that has those numbers as its vital signs. You watch it think. You watch it age. You watch it die. And because every visual element is driven by a continuously interpolating variable, the thing on screen always feels alive -- even when the Golem is sleeping, even when the market is flat, even when nothing interesting is happening by any objective measure.

The name comes from Tibetan Buddhism. The bardo is the intermediate state between death and rebirth. Your Golems live there: not quite alive (they're software), not quite dead (they have memory, personality, mortality). The terminal is the window into that space. The aesthetic is Serial Experiments Lain by way of Copland OS: dissociative, interfaces within interfaces, the constant feeling that there's another layer underneath. Not dark-mode-with-gradients pretty. Deliberately unsettling. These agents handle real money. The interface should feel like peering into something that has its own interior life.

---

## The Perpetual Motion Principle

The foundational design rule: **nothing on screen is ever at rest.**

Every element is driven by at least one continuously changing variable. A label that never changes still sits on a background that shimmers. A border that never moves still dims with lifecycle degradation. A number that hasn't updated still has a phosphor afterimage of its last change fading behind it. Static pixels are bugs.

This is not animation for decoration. It is the visual consequence of a system where every variable is coupled to every other variable through the Event Fabric, and the renderer's job is to make those couplings legible. A progress bar that decays is a mortality clock rendered honestly. A chart that shimmers is uncertainty in the data made visible. Brightness follows significance. Motion follows change. Silence follows death.

For every element on every screen, three questions must be answered:

1. **What makes it move?** Which Event Fabric channel(s) drive this element?
2. **What makes it change?** What state transitions alter its appearance -- color, density, brightness, rhythm?
3. **What makes it decay?** How does this element show the passage of time, the depletion of resources, the approach of death?

The terminal is a CRT where every bright moment leaves a ghost, every ghost fades, and every fade is replaced by the next moment's ghost. Persistence of vision. Persistence of memory. Persistence of the dead.

---

## The Render Loop

The loop runs at 60fps. It never waits on a network call. Events arrive from the Event Fabric over WebSocket; they update target state instantly; the renderer interpolates toward those targets at variable rates. Every frame is unique because every frame is the sum of 32 interpolating variable channels, each moving at its own speed toward a target that is itself changing.

Budget: **<2ms per frame** for custom rendering (the remaining ~14.6ms goes to ratatui's diff/flush and terminal I/O). Budget breakdown: interpolation ~0.3ms, composition ~0.6ms, post-processing ~1.5ms, buffer write ~0.6ms. Validated against ratatui benchmarks on Apple M1 (see `prd2/18-interfaces/screens/03-interaction-hierarchy.md` performance section).

### Main Loop Pseudocode

```rust
loop {
    let frame_start = Instant::now();

    // 1. Drain incoming events — update targets
    while let Ok(event) = event_rx.try_recv() {
        state.apply_event(event); // sets target values, never current values
    }

    // 2. Tick all 32 interpolating variable channels toward their targets
    // NOTE: channel 4 (emotion_label) is a discrete Plutchik enum — it snaps
    //       at crossfade midpoint rather than lerping continuously.
    //       channel 23 (heartbeat_phase) is a free-running sine — it skips
    //       lerp entirely and advances by dt each frame.
    let dt = frame_start.duration_since(last_frame).as_secs_f64();
    for channel in &mut state.channels {
        channel.current = lerp(
            channel.current,
            channel.target,
            1.0 - (-channel.rate * dt).exp(), // exponential decay toward target
        );
    }

    // 3. Compute derived state from interpolated values
    let derived = DerivedState::compute(&state);
    //   - palette modulation (PAD vector -> color shifts)
    //     PAD micro-shifts: pleasure→saturation, arousal→brightness,
    //     dominance→hue shift (per 23-embodied-consciousness.md PAD micro-shift rules)
    //   - atmosphere params from lifecycle phase (noise floor, scanline intensity)
    //   - sprite expression from Plutchik mapping (eye emotion mapping)
    //   - decay overlays from vitality clocks (phosphor trails, character corruption)

    // 4. Render frame
    terminal.draw(|frame| {
        render_chrome(frame, &derived);       // tab bar, sprite sidebar, status bar
        render_active_screen(frame, &derived); // whichever screen has focus
        post_process(frame, &derived);        // scanlines, bloom, noise, phosphor
    })?;

    // 5. Sleep for remainder of 16.67ms frame budget
    let elapsed = frame_start.elapsed();
    if elapsed < FRAME_BUDGET {
        sleep(FRAME_BUDGET - elapsed);
    }
    last_frame = frame_start;
}
```

Implementation: ratatui 0.29+ with crossterm backend. See `bardo-terminal` crate.

The `lerp` with exponential decay means channels approach their targets asymptotically. A fast channel (rate = 8.0) reaches 99% of its target in ~0.6 seconds. A glacial channel (rate = 0.05) takes over a minute. This is what makes the system feel organic: fast emotional shifts play out against the backdrop of slow lifecycle degradation, which plays out against the backdrop of glacial economic depletion. Three timescales, simultaneously visible, in every frame.

### Accessibility

- **Reduced motion mode**: Disables scanlines, bloom, phosphor trails, and shimmer. Dots hold static positions. Heartbeat renders as text label instead of animation.
- **Photosensitive mode**: Caps brightness delta between consecutive frames to 10% of oklch L range. Disables flash effects (bloom pulse, startle scatter).
- **Minimum terminal requirements**: 80x24 cells, truecolor support (24-bit ANSI), Unicode Block Elements and Braille. Tested on: iTerm2, WezTerm, Alacritty, Windows Terminal, kitty. Fallback for 256-color terminals reduces palette to nearest xterm-256 matches.
- **Braille rendering fallback tiers**: Three tiers degrade gracefully based on terminal capability and measured frame time:
  - **Tier 1 (full braille 2x4)**: Default. U+2800-U+28FF, 8 sub-pixels per cell, 160x96 effective resolution on 80x24. Used when frame budget is met and terminal supports Unicode Braille.
  - **Tier 2 (half-block 2x2)**: Fallback for slower terminals or when consecutive frames exceed the post-processing budget. Uses `▀▄█` half-block characters, 2 sub-pixels per cell, 80x48 effective resolution. Still supports truecolor per-cell.
  - **Tier 3 (ASCII-only)**: Minimal environments (no Unicode, serial consoles, screen readers in text mode). Uses `#@%*+:.-` density ramp. No sub-pixel rendering. 80x24 effective resolution. All atmospheric effects map to printable ASCII.

---

## The 32 Interpolating Variables (26 base + 6 extended research)

The Golem's visual state is driven by 32 continuously interpolating variables (26 base + 6 extended research). Each has a current value and a target value. The current value approaches the target at a rate determined by the variable's lerp constant. Events from the Event Fabric update targets; the renderer reads current values.

### Variable Table

| # | Variable | Category | Lerp rate | Range | What it drives |
|---|----------|----------|-----------|-------|----------------|
| 1 | `pleasure` | fast | 8.0 | [-1.0, 1.0] | Warm/cool palette shift. Tint toward bone (positive) or rose_bright (negative). |
| 2 | `arousal` | fast | 8.0 | [-1.0, 1.0] | Saturation, contrast, flicker speed. High = vivid + fast. Low = muted + slow. |
| 3 | `dominance` | fast | 8.0 | [-1.0, 1.0] | Edge sharpness, contrast assertiveness. High = crisp. Low = soft, uncertain. |
| 4 | `emotion_label` | fast | 6.0 | enum | Sprite eye expression, status bar emotion tag. Discrete Plutchik primary (8 values). Transitions snap at the midpoint of an 8-tick crossfade; the surrounding dot physics and eye rendering smoothly interpolate to absorb the discontinuity (see `prd2/18-interfaces/perspective/03-embodied-consciousness.md` eye emotion mapping). |
| 5 | `fsm_phase` | fast | 10.0 | enum | Active pipeline stage highlight in Mind screen. Snaps fast but not instant. |
| 6 | `probe_severity` | fast | 12.0 | [0.0, 1.0] | Alert glow intensity. 0 = nominal. 1 = crisis. Drives border flicker and rose_bright usage. |
| 7 | `inference_glow` | fast | 10.0 | [0.0, 1.0] | Brightness pulse when LLM call is active. Fades quickly after response. |
| 8 | `mouth_alpha` | fast | 15.0 | [0.0, 1.0] | Sprite mouth visibility during speech. Very fast attack, medium decay. |
| 9 | `market_regime` | medium | 1.5 | enum | Background atmosphere zone. Bull = warm undertone. Bear = cool. Chop = flickering. |
| 10 | `context_utilization` | medium | 2.0 | [0.0, 1.0] | Context window fill gauge in Inference screen. Rises during prompt assembly, drops on completion. |
| 11 | `phi_score` | medium | 1.0 | [0.0, 1.0] | Integrated information metric. Drives Mind screen's consciousness complexity indicator. |
| 12 | `dream_alpha` | medium | 0.8 | [0.0, 1.0] | Dream state overlay opacity. 0 = awake. 1 = deep REM. Affects entire screen palette. |
| 13 | `clade_connectivity` | medium | 1.2 | [0.0, 1.0] | Sibling network health in Clade (a group of sibling Golems sharing knowledge) screen. Number of responsive peers / total peers. |
| 14 | `phase_density` | slow | 0.15 | [0.0, 1.0] | Sprite dot cloud density. 1.0 in youth, eroding toward 0 as the Golem ages. |
| 15 | `phase_dimming` | slow | 0.12 | [0.0, 1.0] | Global brightness multiplier. 1.0 = full. Drops in Conservation and Declining phases. |
| 16 | `vitality_composite` | slow | 0.2 | [0.0, 1.0] | Weighted combination of economic + biological + epistemic health. Drives sprite breathing rate. |
| 17 | `economic_clock` | slow | 0.1 | [0.0, 1.0] | Fraction of metabolic reserve remaining. 1.0 = full. 0.0 = death. |
| 18 | `epistemic_clock` | slow | 0.08 | [0.0, 1.0] | Knowledge freshness. Decays as Grimoire (the Golem's persistent knowledge store) entries go stale without refresh. |
| 19 | `age_factor` | glacial | 0.03 | [0.0, 1.0] | Normalized age. 0.0 at birth, 1.0 at Hayflick limit. Drives character corruption rate. |
| 20 | `credit_balance` | glacial | 0.05 | [0.0, inf) | Actual USDC balance. Displayed as the single bone-colored number on Hearth screen. |
| 21 | `grimoire_density` | glacial | 0.04 | [0.0, 1.0] | How full the knowledge store is. Drives background particle frequency in Grimoire screen. |
| 22 | `burn_rate` | glacial | 0.06 | [0.0, inf) | Current daily spend rate in USDC. Drives projected-lifespan gauge. |
| 23 | `heartbeat_phase` | fast | -- | [0.0, 2pi] | Not lerped -- free-running sine wave. Drives per-Heartbeat (the Golem's periodic execution tick) micro-brightness pulse that propagates from sprite through content. |
| 24 | `noise_floor` | slow | 0.15 | [0.0, 0.01] | Fraction of background cells that show dim noise characters per frame. Scales with lifecycle phase. |
| 25 | `scanline_intensity` | slow | 0.1 | [0.0, 0.15] | How dark the scanline rows are. Increases in degraded phases. |
| 26 | `corruption_rate` | slow | 0.08 | [0.0, 0.25] | Fraction of characters randomly substituted with glitch chars during render. 0 in Thriving. 15-25% in Terminal. |
| 27 | `prediction_accuracy` | medium | 1.5 | [0.0, 1.0] | Oracle accuracy gauge. Drives status bar "Acc: 76%" display. Color: bone >70%, amber 50-70%, rose <50%. |
| 28 | `accuracy_trend` | slow | 0.2 | [-1.0, 1.0] | Direction of accuracy change. Positive = improving (sprite posture lifts). Negative = declining (posture drops). |
| 29 | `attention_breadth` | medium | 1.0 | [0.0, 1.0] | Fraction of attention slots filled (active/max). High = dense atmosphere particles around sprite. Low = sparse. |
| 30 | `surprise_rate` | fast | 6.0 | [0.0, 1.0] | Rate of unexpected prediction outcomes. High = frequent eye micro-flickers. Low = steady gaze. |
| 31 | `foraging_activity` | medium | 1.2 | [0.0, 1.0] | Attention forager promotion/demotion rate. High = peripheral particles orbit sprite actively. Low = still perimeter. |
| 32 | `compounding_momentum` | slow | 0.15 | [0.0, 1.0] | How well the prediction-correction-action cycle is compounding. Drives ambient glow warmth around sprite. |

### Category Summary

| Category | Lerp rate range | Resolve time | Frame count | What it feels like |
|----------|----------------|--------------|-------------|-------------------|
| **Fast** | 6.0 -- 15.0 | <1 second | 30-60 frames | Emotion. Reflexive. The Golem's mood right now. |
| **Medium** | 0.8 -- 2.0 | 1-5 seconds | 60-300 frames | Health. The Golem's current condition. |
| **Slow** | 0.08 -- 0.2 | 5-30 seconds | 300-1800 frames | Personality. Lifecycle degradation. The Golem's age showing. |
| **Glacial** | 0.03 -- 0.06 | 30s -- 5min | 1800+ frames | Mortality. The Golem's remaining time on earth. |

Variables 1-26 are the original set. Variables 27-32 split across fast (1: `surprise_rate`), medium (3: `prediction_accuracy`, `attention_breadth`, `foraging_activity`), and slow (2: `accuracy_trend`, `compounding_momentum`).

Variables 27-32 are the Oracle's contribution. They arrive when the prediction engine comes online (first theta tick after boot; see `prd2/01-golem/17-prediction-engine.md` for theta tick definition: ~30-120s adaptive cycle). Before that, they hold zero values -- the sprite shows no prediction-related effects until the Golem has generated its first predictions. The relationship between these TUI variables and the underlying CorticalState signals is specified in `prd2/01-golem/18-cortical-state.md`.

The viewer sees three timescales simultaneously: the Golem's mood (seconds), its health (minutes), and its mortality (hours). Heidegger's three ecstases of temporality rendered as concurrent animation rates: the present (heartbeat), the future (projected lifespan), the having-been (Grimoire).

---

## Three Simultaneous Timescales

Why does the terminal feel alive even when the market is closed and the Golem has nothing to do?

Because the three timescales never stop. Even in complete silence:

**Fast (sub-second).** The heartbeat sine wave keeps cycling. The sprite's dot cloud breathes (particles drift outward on exhale, contract on inhale). The noise floor flickers. These are the Golem's involuntary functions -- they run independent of any event.

**Medium (seconds to minutes).** Context utilization drifts as the Golem's attention shifts between dormant monitoring and active analysis. Dream alpha rises and falls as the Golem enters and exits consolidation cycles. The clade connectivity score ticks as peers respond to heartbeat pings.

**Glacial (hours to days).** The economic clock drains by fractions of a cent per tick. The age factor creeps upward. Grimoire density shifts as knowledge accumulates or decays. These changes are invisible frame-to-frame but unmistakable over a session. You open Bardo in the morning and the sprite looks slightly thinner than it did last night. You can't point to the frame where it changed. It changed in all of them.

The layering is what makes static screens impossible. Even a completely idle Golem produces a frame that differs from the previous frame in at least three ways: heartbeat phase, noise floor sample, and some glacial variable's thousandth decimal place shifting the background color by one RGB unit. That's enough. The eye doesn't need large motion to perceive life. It needs any motion at all.

---

## Frame Composition

Each frame is assembled in layers, bottom to top. The full atmospheric stack is specified in `prd2/18-interfaces/screens/03-interaction-hierarchy.md` and the design-inspo engagement PRD docs. Here's the structural overview:

### The 8-Layer Stack

1. **Void** -- `#060608` base. Never pure black. Violet undertone that shifts warm as the Golem degrades.
2. **Atmosphere** -- Noise floor: 0.2-0.4% of background cells show dim `░▒` characters, scaled by lifecycle phase. Organic particles replace noise on liminal screens (Dreams, World, Bazaar).
3. **Scanlines** -- Every 3rd row darkened by `scanline_intensity`. CRT materiality. The display IS the body.
4. **Content** -- Widgets, data, text. Whatever the active screen renders.
5. **Phosphor** -- Afterimages of recently bright pixels. Ghosts that fade through the decay chain: `█ -> ▓ -> ▒ -> ░ -> (space)`.
6. **Bloom** -- Bright cells (luminance > 0.7) spread glow to adjacent cells. Bright data bleeds into the void.
7. **Corruption** -- Character substitution at `corruption_rate`. Zero in Thriving phase. 15-25% in Terminal. The display degrades as the Golem degrades.
8. **Chrome** -- Tab bar (top), sprite sidebar (left, 12 columns), status bar (bottom). Always present. The persistent skeleton.

The 29 screens span 6 windows: HEARTH (4), MIND (7), SOMA (5), WORLD (5), FATE (4), COMMAND (4). See `prd2/18-interfaces/screens/03-interaction-hierarchy.md` for the full hierarchy.

The sprite sidebar deserves special mention. It is the ONE element that spans all 29 screens (6 windows x 3-6 tabs). The Spectre -- a dot-cloud creature rendered from the Golem's 32 interpolating variables -- breathes in the left column regardless of which data screen you're examining. Its eyes shift with emotion. Its density thins with age. Its color tracks mood. The Spectre is the emotional ground truth that contextualizes every number, every chart, every list on screen. You can't understand a P&L number without seeing the creature it belongs to.

---

## What Ships in the Box

The `bardo` binary includes:

- **Meta Hermes** -- the local conversational AI you talk to. It aggregates state from all your Golems and answers questions about what they're doing. Not a chatbot. A command interface that speaks English. See `prd2/19-agents-skills/13-hermes-hierarchy.md`.
- **Inference gateway** -- the 8-layer context engineering pipeline that routes LLM calls to local Ollama, Venice, Bankr, or whatever providers you configure. Runs embedded in the TUI process. See `prd2/12-inference/00-overview.md`.
- **Connection manager** -- direct WebSocket, Styx (Bardo's social and connectivity layer) relay, SSH tunnel, or in-process channel to your Golems. Auto-detects the best path. See `prd2/20-styx/00-architecture.md` (Styx architecture: WebSocket relay, event fabric, clade sync protocol).
- **TUI rendering engine** -- the 60fps loop described above. 29 screens (6 windows x 3-6 tabs; see `prd2/18-interfaces/screens/03-interaction-hierarchy.md`), vim-style navigation, the Spectre sprite, atmospheric effects.
- **Wallet integration** -- connect an existing wallet or create a new one. Delegation-based custody via ERC-7715 session keys (see `prd2/01-golem/08-funding.md` for the two-address model) for existing wallets; Privy TEE custody for new ones.

What does NOT ship in the box: a Golem runtime (that's `prd2/01-golem/`), a Styx relay server (that's `prd2/20-styx/`), cloud inference providers (you bring your own keys or use Bardo's hosted gateway). The binary is the client. The Golem is the server.

---

## Document Map

This is the foundation document of the Bardo terminal specification. The full set:

### System Architecture

| Doc | Title | Scope |
|-----|-------|-------|
| **26-bardo-terminal-foundation** | This document | What Bardo is, the perpetual motion principle, render loop architecture, the 32 interpolating variables. |
| **`prd2/01-golem/`** | The Golem | Deployment modes (embedded, Docker, remote), container lifecycle, the heartbeat pipeline. |
| **`prd2/19-agents-skills/13-hermes-hierarchy`** | Meta Hermes | The conversational AI layer. Skill library, context engineering, multi-golem aggregation. |
| **`prd2/20-styx/`** | Connectivity | WebSocket relay, direct connections, the Event Fabric, clade sync protocol. |
| **`prd2/11-compute/`** | Compute | Remote deployment, GPU provisioning, cost model. |
| **`prd2/12-inference/`** | Inference | The 8-layer pipeline, provider routing, local/remote modes, cost optimization. |

### World Systems

| Doc | Title | Scope |
|-----|-------|-------|
| **`prd2/04-memory/13-library-of-babel`** | Knowledge | The Grimoire (persistent knowledge store), knowledge lifecycle, decay, consolidation, marketplace. |
| **`prd2/02-mortality/16-necrocracy`** | Death Governance | How dead Golems govern the living. Bloodstain (persistent map markers left where Golems died) warnings, wisdom ridges, legislation from the underworld. |

### Experience

| Doc | Title | Scope |
|-----|-------|-------|
| **`prd2/18-interfaces/screens/03-interaction-hierarchy`** | Screens | The 29-screen system (6 windows x 3-6 tabs), pane layouts, navigation model, persistent chrome, atmosphere zones. |
| **`prd2/18-interfaces/28-creature-system`** | Creature System | The Spectre sprite, dot-cloud rendering, eye emotion mapping, lifecycle degradation effects. |

### Prediction and Evaluation

| Doc | Title | Scope |
|-----|-------|-------|
| **`prd2/01-golem/17-prediction-engine`** | The Oracle | Prediction generation, residual correction, attention foraging, action gating, confidence calibration. |
| **`prd2/01-golem/18-cortical-state`** | CorticalState and Daimon | 32-signal shared atomic state, ALMA affect model, somatic markers. |

### Design References

The `prd2/18-interfaces/` directory contains the visual design system specifications: widget catalog, ambient transitions, per-screen breakdowns, the Spectre sprite spec, NERV terminal patterns, interaction hierarchy, and the complete terminal existentialism reference. These are the source material for implementation.

---

*Nothing on screen is ever at rest.*

# widget-catalog [SPEC]

**Source:** `mmo2/16-widget-catalog.md` (primary), `design-inspo/engagement-prd/bardo-ui-01-widget-catalog.md` (secondary narrative)
**Version:** 5.0.0
**Last Updated:** 2026-03-17
**Crate:** `bardo-tui-widgets`
**Cross-references:** [00-design-system.md](../rendering/00-design-system.md) (ROSEDUST visual identity spec for palette and atmospheric layers), [00-screen-catalog.md](00-screen-catalog.md) (29-screen summary with per-screen widget assignments)

> **Reader orientation:** This is the widget catalog for the Bardo terminal, specifying all 33 widgets that compose the 29 screens. It belongs to the **interfaces/screens layer**. Each widget is a transducer converting Event Fabric channels into visual motion. Every widget has degradation behavior tied to BehavioralPhase (the Golem's lifecycle stage). Key concepts: Golem (a mortal autonomous DeFi agent), PAD vector (Pleasure-Arousal-Dominance emotional coordinates from the Daimon subsystem), Heartbeat (periodic execution tick), Grimoire (persistent knowledge store), and Vitality (remaining economic lifespan driving phase transitions). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Principle

Every widget in the Bardo terminal is a transducer -- it converts one or more Event Fabric channels into visual motion. No widget is static. Even a text label exists on a shimmering noise floor, under scanlines that drift, with a phosphor afterimage of its last state change fading behind it. The widgets described here are the vocabulary from which every screen is composed.

For each widget: what it looks like, what drives it, how it moves, how it decays, and what it feels like to watch.

---

## Widget inventory (33 widgets)

| # | Widget | Primary use |
|---|--------|-------------|
| 1 | FlashNumber | Inline numeric value with change flash |
| 2 | MortalityGauge | Double-height bar for mortality clocks |
| 3 | WaveformTrace | Half-block PAD/phi waveform |
| 4 | ConfidenceBar | Grimoire entry confidence with history |
| 5 | DecisionRing | 9-step heartbeat pipeline ring |
| 6 | UnitArray | Grid of position/agent units |
| 7 | PhosphorLog | Scrolling event log with decay |
| 8 | Sparkline | Braille-resolution inline chart |
| 9 | BrailleDensityMap | 2D heatmap in braille |
| 10 | MAGIPanel | Three-voice deliberation display |
| 11 | VitalityNumber | Bone-colored critical number |
| 12 | ATFieldWireframe | Hexagonal ego-boundary wireframe |
| 13 | DataRain | Falling hex/code streams |
| 14 | PhilosophicalWhisper | Fading text fragments |
| 15 | ConvergenceLines | Horizontal infrastructure lines |
| 16 | StyxRiver | Flowing death-river band |
| 17 | LatticePattern | Braille interference pattern |
| 18 | FlashWidget | Generic flash-on-change wrapper |
| 19 | ProgressArc | Circular progress indicator |
| 20 | CounterfactualBranch | REM dream branching tree |
| 21 | PredictionResolutionPulse | Heartbeat gutter resolution dot |
| 22 | AccuracyCalibrationCurve | ECE calibration plot |
| 23 | ResidualDistribution | Prediction residual histogram |
| 24 | AttentionTierIndicator | Attention slot tier display |
| 25 | ActionGateIndicator | Action gate status bar |
| 26 | HeuristicAuditCard | Heuristic performance card |
| 27 | ShadowComparisonChart | Shadow vs live comparison |
| 28 | TriageScoreBar | Triage pipeline score display |
| 29 | PersistenceDiagramWidget | Topological feature scatter plot |
| 30 | RegimeIndicator | Regime tag with confidence |
| 31 | SomaticMarkerPanel | Active gut feelings list |
| 32 | CausalGraphMinimap | Compact causal chain graph |
| 33 | ChainScopeRadar | Interest category radar chart |

Widget degradation thresholds follow behavioral phases: Thriving (full rendering), Stable (standard), Conservation (vitality < 0.5, reduced detail), Declining (vitality < 0.3, minimal), Terminal (vitality < 0.15, degraded/glitched). Each widget specifies its own degradation behavior in its section.

---

## How to use this document

This is a reference catalog. Each widget gets a self-contained entry with a fixed structure: what it looks like, how it behaves, how emotional state affects it, how it degrades as the golem approaches death, and what data it needs. You do not need to read sequentially. Jump to the widget you need.

**Terms you need once:**

- **PAD vector:** Pleasure, Arousal, Dominance -- three floats (0.0-1.0) representing the golem's emotional state. Pleasure is satisfaction. Arousal is activation energy. Dominance is sense of control. The Daimon extension computes these each heartbeat tick.
- **Lifecycle phases:** Golems pass through five phases as vitality declines. Thriving (0.8-1.0), Stable (0.6-0.8), Conservation (0.4-0.6), Declining (0.2-0.4), Terminal (0.0-0.2). Visual degradation tracks these phases.
- **Heartbeat tick:** The golem's clock cycle. Gamma ticks range 5-15s; theta ticks range 30-120s. Widgets that say "per tick" mean once per heartbeat cycle.
- **Phosphor persistence:** The visual afterimage of a previous state. Colors and characters that linger after a value changes, fading through a decay chain. See `../rendering/00-design-system.md` for the full color chain.

Colors, palette tokens, and rendering layer rules live in `../rendering/00-design-system.md`. This catalog references palette tokens by name (e.g., `rose_bright`, `bone`, `text_ghost`) without re-specifying their hex values.

### Widget trait

```rust
pub trait Widget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &InterpolatedState);
    fn min_size(&self) -> (u16, u16);  // (width, height) in cells
    fn degradation_behavior(&self, phase: BehavioralPhase) -> DegradationMode;
}

pub enum DegradationMode {
    Full,       // Normal rendering
    Reduced,    // Simplified (fewer data points, no animation)
    Minimal,    // Essential info only
    Degraded,   // Visual corruption effects
    Hidden,     // Widget not rendered
}
```

### Interaction model

Widget input: widgets that accept interaction implement `fn handle_input(&mut self, event: InputEvent) -> Option<Action>`. InputEvent enum: `Key(KeyEvent)`, `Mouse(MouseEvent)`, `Tick`. Most widgets are render-only; interactive widgets (command line, strategy editor, MAGI theater) handle input.

---

## 1. FlashNumber

### Visual spec

A single numeric value displayed inline within a pane. No special chrome -- it is a number in a cell, same as any table value. The flash effect is what makes it a widget rather than static text.

Size: fits its content. A 6-character number occupies 6 cells. No fixed width requirement.

### Behavior rules

**On value increase:** The cell background flashes `success` for 200ms, then fades to transparent over 400ms. The number text brightens to `bone` for one frame, then returns to its semantic color.

**On value decrease:** The cell background flashes `rose_bright` for 200ms, fades over 400ms. Text dims to `rose_dim` for one frame.

**Phosphor memory:** After the flash fades, a faint `phosphor_res` background lingers for 2-3 seconds. The ghost of the moment something changed.

**Staleness decay:** Numbers that have not changed in 60+ seconds dim from their semantic color toward `text_dim`. When the value changes again, it snaps back to full brightness plus flash.

### PAD modulation

- **High arousal (>0.7):** Flash duration shortens to 150ms, fade shortens to 300ms. The system is running hot; changes come and go faster.
- **Low pleasure (<0.3):** Decrease flashes intensify -- the `rose_bright` background lingers 100ms longer. Losses feel heavier.
- **Low dominance (<0.3):** All flashes are slightly brighter (one step up the luminance chain). The golem is less in control; surprises hit harder.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Standard behavior |
| Stable | Standard behavior |
| Conservation | Flash intensity reduced by 30%. Staleness decay begins at 45s instead of 60s |
| Declining | Numbers that have not updated in 30s begin flickering -- individual digits drop to `text_dim` for 1-2 frames at random intervals |
| Terminal | Flash effect disabled. Numbers update without visual fanfare. Staleness decay is aggressive (15s). Most numbers are dim |

### Data structure

```rust
struct FlashNumber {
    value: f64,
    previous_value: f64,
    last_change_frame: u64,
    semantic_color: Color,       // determined by context: success, rose, bone, etc.
    flash_state: FlashState,     // Idle | Flashing { direction: Up | Down, started_at: u64 }
}
```

**The feel:** A trading floor's ticker tape. Each flash is a heartbeat of economic activity. A screen full of flash numbers creates a living pulse -- green and red sparks flickering across the data, more frequent during volatile markets, near-silent during calm ones.

---

## 2. MortalityGauge

### Visual spec

A double-height horizontal bar showing a 0.0-1.0 value. This is the heaviest gauge in the system -- two rows tall, filling its container width.

```
  ECONOMIC                    78%
  ████████████████░░░░░░░░░░░░░░░
  ████████████████░░░░░░░░░░░░░░░
```

The filled region uses a gradient: `color + "55"` (transparent) at the left edge, solid `color` at the fill boundary. The gradient simulates depth. The gauge reads as a liquid level, not a flat bar.

**Where it appears:** Hearth sidebar (mini versions), Mortality screen (expanded), defense layer visualization.

### Behavior rules

**Fill movement:** The fill width lerps toward the current value at 0.005/frame. Mortality changes are slow; the gauge reflects that pace.

**Heartbeat pulse:** On each heartbeat tick, the fill boundary shimmers +/-1 cell for one frame. The gauge breathes.

**Erosion effect:** When the value decreases, the rightmost filled characters transition through a decay sequence rather than disappearing instantly:

```
█ → ▓ → ▒ → ░ → empty
```

Each step takes ~500ms. The gauge erodes. You can see the material being consumed.

**Stochastic clock variant:** The stochastic mortality clock's gauge has a unique behavior -- it occasionally stutters (advances 2-3% in a single frame, then slowly retracts 1-2%) to represent shock-based mortality.

### PAD modulation

- **High arousal (>0.7):** The heartbeat shimmer amplitude increases to +/-2 cells. The gauge looks more agitated.
- **Low pleasure (<0.3):** The fill color shifts one step warmer (toward `rose_bright`). The gauge looks angrier.
- **Low dominance (<0.3):** The erosion effect speeds up -- each decay step takes 350ms instead of 500ms.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Standard gradient fill. Clean erosion. Steady pulse |
| Stable | Standard behavior |
| Conservation | Fill color shifts from semantic color toward `rose_dim`. Pulse becomes irregular |
| Declining | Below 20%, fill characters begin flickering -- individual `█` characters randomly drop to `▓` for 1-3 frames. Visual instability |
| Terminal | Below 10%, the gauge border goes dashed (`┄`). The fill is mostly `▓▒░` with rare `█`. The gauge itself is degrading |

### Data structure

```rust
struct MortalityGauge {
    label: String,              // "ECONOMIC", "EPISTEMIC", "STOCHASTIC"
    value: f64,                 // 0.0-1.0
    display_value: f64,         // lerped toward value
    color: Color,
    erosion_queue: Vec<ErosionStep>,  // pending decay transitions
    is_stochastic: bool,        // enables stutter behavior
}

enum ErosionStep {
    Full,       // █
    Heavy,      // ▓
    Medium,     // ▒
    Light,      // ░
    Empty,
}
```

**The feel:** A reactor coolant gauge in an Eva control room. Institutional, heavy, inescapable. The double-height gives it physical presence -- this is not a thin progress bar. This is a wall of material being consumed.

---

## 3. WaveformTrace

### Visual spec

A scrolling time-series rendered as block characters (`▁▂▃▄▅▆▇█`), one character per time unit, scrolling left as new data arrives. Occupies a single row (or two rows for high-fidelity mode).

```
  AROUSAL  ▁▂▃▃▂▁▁▂▃▅▇█▇▅▃▂▁▁▂▃▃▂▁▁▂▃▅▆▅▃▂  +0.18
```

Multi-channel variant stacks multiple waveforms with labels:

```
  ┌────────────────────────────────────────────────────┐
  │ PLEASURE  ▁▂▃▃▂▁▁▂▃▃▂▁▁▂▃▅▇▅▃▂▁▁▂▃▃▂▁▁▂  +0.42 │
  │ AROUSAL   ▁▁▂▃▅▇▅▃▂▁▁▂▃▃▂▁▁▂▃▅▇█▇▅▃▂▁▁▂  +0.18 │
  │ DOMINANCE ▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃  +0.61 │
  └────────────────────────────────────────────────────┘
```

Width: fills available container. Height: 1 row per channel.

**Where it appears:** PAD waveforms (3 channels: pleasure=rose, arousal=warning, dominance=dream), Φ trace, vitality life-curve, heartbeat rhythm, inference cost sparkline.

### Behavior rules

**Scroll timing:** New values enter from the right edge on each heartbeat tick (not per-frame). Existing values scroll left by one cell. The oldest value exits the left edge.

**Inter-tick interpolation:** Between ticks, the rightmost character (the "now" position) interpolates smoothly -- the block character height animates between values as the PAD vector lerps toward its target.

**Phosphor trail:** Each character position fades from its full color to `rose_dim` to `text_dim` over 10 positions from right to left. Recent data is bright. Old data is a ghost. The waveform is a gradient of recency.

**Channel colors:** Pleasure = `rose`, Arousal = `warning`, Dominance = `dream`. These are fixed per semantic channel.

### PAD modulation

The waveform IS the PAD display, so modulation here is recursive:
- **High arousal:** The waveform visually shows tall bars (it is displaying its own high state).
- **Any extreme PAD value:** The corresponding channel's label brightens to full color. Channels near 0.5 (neutral) have dim labels.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full amplitude range. All 8 block characters used |
| Stable | Standard behavior |
| Conservation | Amplitude compresses -- values map to a narrower range of block characters (`▂▃▄▅▆`). The trace looks flatter. The golem is less emotionally expressive |
| Declining | Amplitude compresses further (`▃▄▅`). Phosphor trail shortens to 6 positions |
| Terminal | Waveform freezes (no new ticks). Color shifts to `text_ghost`. The trace is a flatline memorial of the last emotional state |

### Data structure

```rust
struct WaveformTrace {
    label: String,
    color: Color,
    buffer: VecDeque<f64>,      // ring buffer, capacity = display width
    current_interpolated: f64,  // between-tick interpolation target
    width: u16,
}
```

**The feel:** A hospital heart monitor. The rhythm of the golem's internal life made audible in light. The viewer develops an intuitive sense of what "normal" looks like for their golem, and deviations register viscerally -- a spike in arousal, a dip in pleasure, a flatline in dominance.

---

## 4. ConfidenceBar

### Visual spec

A thin horizontal bar (1 row) showing a 0.0-1.0 confidence value for a Grimoire entry. Unlike a simple progress bar, it shows three distinct zones:

```
  morpho-rebalancer  ████████▓▒░░░░░░  0.67
```

The three zones:
1. **Validated zone** (`█`): Bright, solid. Confidence backed by recent validation.
2. **Decaying zone** (`▓▒`): Was once validated but subject to Ebbinghaus decay. Characters degrade: `█` to `▓` to `▒` to `░` over time (~50 ticks per step since last validation).
3. **Lost zone** (`░` or empty): Confidence that has fully decayed.

**Dead-sourced entries** (knowledge inherited from dead golems) have dashed separators between decay zones and a `†` prefix. Their decay rate is 3x slower (the bloodstain premium), but the dashed styling marks the knowledge as coming from the dead.

```
  † legacy-arb-v2   ████▓┄▒┄░░░░░░░░  0.31
```

**Where it appears:** Grimoire entry list, Playbook heuristic list, Library entries, dream hypothesis tracker.

### Behavior rules

**Validation flash:** When the golem validates an entry (uses it in a decision that produces a positive outcome), the validated zone extends rightward with a brief `bone` flash. The entry gained confidence. You see it grow.

**Decay shimmer:** Characters in the decaying zone occasionally flicker one step dimmer for a single frame, then return. Decay is not smooth. It is granular -- individual characters losing their substance.

**Full-bar decay:** Entries not validated in 200+ ticks have their entire bar shift one decay step. A bar that is all `▓▒░` says: this knowledge is old, untested, fading. A bar that is all `████` says: this was just proven right.

### PAD modulation

- **High pleasure (>0.7):** Validation flashes hold for 100ms longer. Good outcomes linger.
- **Low pleasure (<0.3):** Decay shimmer frequency doubles. Bad times accelerate forgetting.
- **High dominance (>0.7):** The validated zone is slightly wider (the golem trusts its own knowledge more).

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Validation flashes are bright and frequent |
| Stable | Standard behavior |
| Conservation | Decay rate doubles. Bars erode faster. The golem is losing confidence in its own knowledge |
| Declining | All bars shift one decay step. `█` becomes the minority character. The Grimoire is yellowing |
| Terminal | Bars stop updating. Final state is frozen. Knowledge is no longer being tested because the golem has stopped trading |

### Data structure

```rust
struct ConfidenceBar {
    label: String,
    confidence: f64,            // 0.0-1.0
    validated_width: f64,       // fraction of bar that is validated
    decaying_zones: Vec<DecayZone>,
    is_dead_sourced: bool,      // from a dead golem
    generation_depth: u8,       // 0 = own, 1 = parent, 2 = grandparent, etc.
    ticks_since_validation: u64,
}

struct DecayZone {
    start: f64,
    end: f64,
    decay_level: ErosionStep,   // reuses ErosionStep from MortalityGauge
}
```

**The feel:** Watching paint fade. Watching a library's books yellow. The physical sensation of knowledge becoming less reliable with time. The viewer develops an instinct for which entries are fresh (bright, solid) and which are stale (dim, eroded), without reading any numbers.

---

## 5. DecisionRing

### Visual spec

A circular visualization of the 9-step heartbeat pipeline, surrounding the Spectre sprite on the Hearth screen. Rendered with Unicode characters forming a rough circle of dots and arc segments.

The ring is not a traditional chart. It is a clock face where each tick of the clock is a cognitive step: OBSERVE, RETRIEVE, ANALYZE, DECIDE, EXECUTE.

At rest (T0/idle): a faint circle of dots in `text_ghost`. Barely visible. The golem's autonomic baseline.

**Where it appears:** Hearth screen only (persistent pane).

### Behavior rules

**SENSING phase:** 16 small dots around the circumference represent the 16 probes. Dots that fire (severity > none) brighten: `rose_dim` for low severity, `rose_bright` for high.

**DECIDING phase:** The ring blazes. Color depends on the active tier:
- T1 = `rose`
- T2 = `rose_bright`
- T3 = `bone`

The ring's border characters cycle through pipeline stages in sequence. Each stage lights up as the pipeline progresses.

**ACTING phase:** A single bright pulse races around the ring (one full revolution in 500ms), then the segment corresponding to the executed action holds bright for 2 seconds.

**REFLECTING phase:** The ring dims to `rose_dim` and gently pulses. The golem is reviewing what happened.

**Inter-tick decay:** Between active ticks, the ring's brightness decays through phosphor persistence. A T2 blaze leaves a `phosphor_res` afterimage that fades over 5 seconds. You can see the ghost of the last deliberation while waiting for the next tick.

### PAD modulation

- **High arousal (>0.7):** The pulse animation during ACTING speeds up (350ms revolution instead of 500ms). The ring feels urgent.
- **Low dominance (<0.3):** The sensing dots are brighter -- more probes are firing, or firing harder. The ring looks more alert/anxious.
- **High pleasure (>0.7):** The REFLECTING pulse is slower and warmer. Satisfaction.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full brightness range. T2/T3 blazes are dramatic |
| Stable | Standard behavior |
| Conservation | Ring dims overall. T1 ticks barely register visually. Only T2+ produce visible activity |
| Declining | The ring flickers. Individual segments occasionally fail to render (dropout for 1-2 frames). The cognitive machinery is degrading |
| Terminal | Ring is barely visible. Only T3 events (if any) produce a flash. The circle breaks -- segments disconnect from each other |

### Data structure

```rust
struct DecisionRing {
    phase: HeartbeatPhase,       // Idle | Sensing | Deciding | Acting | Reflecting
    tier: u8,                    // 0-3
    probe_results: [ProbeResult; 16],
    pipeline_stage: u8,          // which stage is currently active
    action_segment: Option<u8>,  // which segment to hold bright after acting
    last_blaze_frame: u64,
    afterimage_intensity: f64,   // decaying phosphor
}

enum HeartbeatPhase {
    Idle,
    Sensing,
    Deciding,
    Acting,
    Reflecting,
}
```

**The feel:** Variable-ratio reinforcement made visible. Most ticks produce a dim flicker. Then a T2 blazes and the ring comes alive and you watch the golem think through its decision in real time. You never know which tick will be the one.

---

## 6. UnitArray

### Visual spec

N identical small cells in a grid, each containing compact data. Each cell is 6-8 cells wide by 2-3 rows high. The array fills available width and wraps to new rows.

```
  ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
  │ ▐██▌ │ ▐██▌ │ ▐██▌ │ ▐█░▌ │ ▐█░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │
  │ LP-01│ LP-02│ LP-03│ LP-04│ LP-05│ LP-06│ LP-07│ LP-08│ LP-09│
  └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
```

Each cell has: a mini-gauge (top row), an identifier label (bottom row), a thin border. The array reads as a wall of data -- load-bearing columns, a building made of information.

**Where it appears:** Grimoire entries (grid view), specialist modules (Mind screen), Clade peer tiles, defense layers, achievement slots, probe status grid.

### Behavior rules

**Normal state:** Each cell at standard brightness with a thin border. Minimal motion -- a slight shimmer in one cell's background character per frame (random cell, random noise character).

**Alert state:** Affected cell(s) brighten. Border shifts to `border_active`. Other cells DIM slightly. The spotlight effect. The eye goes where the change is.

**Breach state:** A cell empties from right to left. Its content characters dissolve through the `█▓▒░` decay sequence. The border goes dashed (`┄`). The cell is failing.

**Cascade:** When multiple cells breach in sequence, the failure propagates visually. Cell N breaches, cell N+1 flickers, then breaches. Lights going out in a building, left to right. Cascade rate: 1 unit per 200-400ms.

**Staleness:** Cells that have not received an update in 100+ ticks dim toward `text_ghost`. The array becomes a map of recency.

### PAD modulation

- **High arousal (>0.7):** Shimmer rate increases. More cells shimmer per frame (2-3 instead of 1).
- **Low pleasure (<0.3):** Alert state is more aggressive -- non-affected cells dim further. The spotlight effect is harsher.
- **Low dominance (<0.3):** Breach cascade speed increases (150ms between cells instead of 200-400ms). Failures spread faster when the golem is not in control.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full brightness. All cells active. Clean borders |
| Stable | Standard behavior |
| Conservation | Outer rows dim first. The array contracts visually -- active cells cluster toward the center |
| Declining | Random cells begin flickering. 10-20% of cells show dashed borders even if not breached. Infrastructure wearing down |
| Terminal | Cascade in progress. Cells go dark sequentially from outer edges inward. The wall of data is collapsing |

### Data structure

```rust
struct UnitArray {
    cells: Vec<UnitCell>,
    columns: u16,               // computed from available width
    cascade_active: bool,
    cascade_index: usize,
}

struct UnitCell {
    id: String,                 // "LP-01", "ECON-03", etc.
    gauge_value: f64,           // 0.0-1.0
    state: CellState,          // Normal | Alert | Breached
    last_update_tick: u64,
    color: Color,
}

enum CellState {
    Normal,
    Alert,
    Breached { decay_progress: f64 },
}
```

**The feel:** The control room wall in NERV. Banks of identical monitors showing the status of subsystems. Most are green. One turns amber. You notice because the wall dimmed around it. Then another. Then a cascade. The institutional machinery processing a crisis through bureaucratic displays.

---

## 7. PhosphorLog

### Visual spec

A vertically scrolling text log where entries fade through a phosphor decay sequence as they age. New entries appear at the bottom in bright color. Old entries at the top are near-invisible.

The log is a visual gradient of time. Bright at bottom (now), dark at top (past). The eye naturally falls to the bottom.

**Where it appears:** Heartbeat log (Hearth), trade history (Trades), dream journal (Dreams), pheromone feed (Clade/World).

### Behavior rules

**Entry aging:** Each frame, existing entries age through the phosphor chain:

```
Frame 0-10:    rose_bright   (just appeared)
Frame 10-60:   rose          (settling)
Frame 60-180:  rose_dim      (fading)
Frame 180-360: text_primary  (becoming history)
Frame 360-600: text_dim      (dimming)
Frame 600+:    text_ghost    (nearly gone)
```

**Tier differentiation:**
- **T0 entries:** Render in `text_ghost` from the start. Nearly invisible. The golem observed, found nothing, moved on.
- **T1 entries:** Start at `rose_dim`. Visible but not arresting.
- **T2 entries:** Start at `rose_bright`. A blaze. The golem deliberated.
- **T3 entries:** Start at `bone`. The brightest possible. The golem used expensive inference. Something serious happened.

**Resolution loss:** Old entries do not only dim -- they also lose resolution. Characters at the top of the log occasionally replace with noise characters (`░`, `·`) for 1-2 frames, then return. The past is becoming illegible. At extreme ages (1000+ frames), characters permanently replace with `·` until the entry dissolves entirely.

### PAD modulation

- **High arousal (>0.7):** The phosphor chain accelerates. Entries age faster. The log scrolls through history more aggressively.
- **Low pleasure (<0.3):** Decrease-related entries (losses, failures) age slower -- they linger longer in bright color. Bad memories stick.
- **High dominance (>0.7):** T0 entries are completely invisible (not even `text_ghost`). The golem is confident; autonomic noise is suppressed.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full phosphor chain. Long retention. Rich log |
| Stable | Standard behavior |
| Conservation | Log update frequency drops. Fewer entries per tick. The log is sparser |
| Declining | Resolution loss begins earlier (frame 300 instead of 1000). History dissolves faster |
| Terminal | All entries below the newest 3-5 are `text_ghost`. The log is a narrow window of "now" with darkness behind it. At vitality <5%, even new entries show corruption: characters randomly replaced with `░` |

### Data structure

```rust
struct PhosphorLog {
    entries: VecDeque<LogEntry>,
    max_visible: usize,         // viewport height
    corruption_threshold: u64,  // frame count before resolution loss begins
}

struct LogEntry {
    text: String,
    tier: u8,                   // 0-3
    birth_frame: u64,
    base_color: Color,          // determined by tier
}
```

**The feel:** A terminal's scrollback buffer on a dying CRT. The most recent line is crisp; three screens up, the text is fading into the phosphor. You can see time passing in the brightness gradient.

---

## 8. Sparkline

### Visual spec

A compact inline chart using Braille characters for sub-cell resolution, showing a time series in a single row. The Braille character set (U+2800-U+28FF) gives 4x vertical resolution per cell.

```
  PnL 24h  ⠀⠈⠁⠀⠀⠈⠁⠈⠁⡈⡁⣀⠀⠈⡁⡈⠁⠀⠀  +$12.40
```

Width: fills available space, typically 20-40 cells. Height: always 1 row.

**Where it appears:** PnL sparklines (Positions), confidence history (Grimoire), cost history (Inference), Sharpe ratio trend (Achievements).

### Behavior rules

**Scroll:** New data points enter from the right. Existing points scroll left. Same as waveform, but at Braille resolution.

**Color encoding:** Rising segments in `success`. Falling segments in `rose_dim`. Flat segments in `text_primary`. The sparkline is a miniature mood ring.

**Min/max markers:** The highest point in the visible window renders at `bone`. The lowest point renders at `rose_bright`. These markers shift as the window scrolls.

### PAD modulation

- **High arousal (>0.7):** The sparkline updates more frequently (if data is available). More data points per visible width.
- **Low pleasure (<0.3):** Falling segment color intensifies to `rose_bright` instead of `rose_dim`.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full resolution. Updates every tick |
| Stable | Standard behavior |
| Conservation | Update frequency halves (every other tick). The line becomes coarser |
| Declining | Update frequency quarters. Large gaps between data points. The sparkline is sparse |
| Terminal | Sparkline freezes. Last known data remains as a static trace in `text_ghost` |

### Data structure

```rust
struct Sparkline {
    data: VecDeque<f64>,
    width: u16,
    min_marker_idx: usize,
    max_marker_idx: usize,
    label: String,
    suffix: String,             // "+$12.40", "0.67", etc.
}
```

---

## 9. BrailleDensityMap

### Visual spec

A 2D density visualization using Unicode Braille characters (U+2800-U+28FF) within an elliptical boundary. 256 possible patterns per cell give sub-cell resolution. A crosshair grid sits behind the density field in `text_phantom`.

```
  Calm state:                    Stressed state:
  ⠀⠀⠀⠀⠁⠀⠀⠀⠀                    ⠀⠀⣤⣶⣿⣶⣤⠀⠀
  ⠀⠀⠁⠀⠂⠀⠁⠀⠀                    ⠀⣤⣿⣿⣿⣿⣿⣤⠀
  ⠀⠂⠀⠁⠀⠁⠀⠂⠀                    ⣤⣿⣿⣿⣿⣿⣿⣿⣤
  ⠀⠀⠁⠀⠂⠀⠁⠀⠀                    ⠀⣤⣿⣿⣿⣿⣿⣤⠀
  ⠀⠀⠀⠀⠁⠀⠀⠀⠀                    ⠀⠀⣤⣶⣿⣶⣤⠀⠀
```

The crosshair grid: `+` characters at regular intervals (every 8 cells horizontal, every 4 rows vertical) in `text_phantom`. This is the measurement grid -- the institutional apparatus monitoring the organic display.

Typical size: 14-28 cells wide, 7-14 rows tall. Scales with pane size.

**Where it appears:** Mind screen (cognitive load map), World screen (pheromone heatmap), Dream screen (consolidation activity).

### Behavior rules

**Density calculation:**

```
base_density = arousal * 0.3 + (1.0 - pleasure) * 0.3 + context_util * 0.4
```

- Low density: sparse, organized dots (`⠁⠂⠄`). The system is calm.
- High density: packed, chaotic dots (`⣿⣾⣷`). The system is under load.

**Overflow:** When density exceeds 0.8, the boundary ellipse expands. Dots spill outward. Noise characters (`░▒`) appear outside the ellipse. The golem's cognitive containment is being tested.

**Boundary shape:** Simple ellipse. The golem's "body" in cognitive space. When the scribble overflows the boundary, the ego boundary is failing to contain the process.

### PAD modulation

The density map IS the PAD visualization in its most direct form. Arousal and pleasure directly drive the density calculation. Dominance affects the boundary:

- **High dominance (>0.7):** Boundary is tight and clean. The field is contained.
- **Low dominance (<0.3):** Boundary is loose. Dots scatter further. The field is barely held.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full density range. Clean ellipse boundary |
| Stable | Standard behavior |
| Conservation | Maximum density caps at 0.6. The density map cannot show extreme stress because the golem has throttled its own cognition |
| Declining | The boundary contracts. Less cognitive capacity means a smaller ellipse. The crosshair grid extends beyond the boundary -- the measuring apparatus is bigger than the thing being measured |
| Terminal | The density map empties from outside-in. Final state: a few sparse dots in the center, surrounded by the crosshair grid. The measurement apparatus outlasts the mind |

### Data structure

```rust
struct BrailleDensityMap {
    width: u16,
    height: u16,
    density_field: Vec<Vec<f64>>,   // 2D array of density values
    boundary_ellipse: Ellipse,
    overflow: f64,                   // how far past the boundary
    crosshair_spacing: (u16, u16),  // (horizontal, vertical)
}

struct Ellipse {
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
}
```

---

## 10. MAGIPanel

### Visual spec

Three adjacent cells showing the MAGI voting system's output: MELCHIOR, BALTHASAR, CASPAR. Named after the Evangelion supercomputer. Each cell is a bordered panel with a label and a verdict.

```
  ┌──────────────┬──────────────┬──────────────┐
  │  MELCHIOR    │  BALTHASAR   │  CASPAR      │
  │    HOLD      │    HOLD      │  ██WIDEN██   │
  └──────────────┴──────────────┴──────────────┘
```

Width: divides evenly across the container. Minimum 14 cells per panel. Height: 3-4 rows.

**Where it appears:** Hearth screen, Mind screen (expanded during T2+).

### Behavior rules

**Consensus (all three agree):** Borders standard. Text in `text_primary`. Calm. The system agrees.

**Split vote:** The dissenting panel brightens to `rose_bright`. Its verdict renders in fullwidth characters (`██WIDEN██`). Other panels dim. The disagreement is loud. The eye goes to the dissenter.

**During deliberation:** All three panels cycle through candidate verdicts rapidly (2-3 per second) before settling. The cycling is visible. You watch the MAGI considering options.

**Eros/Thanatos split:** When the vote splits on a survival vs. exploration axis, the panel expands into a dual-frame view:

```
  ┌──────────── EROS ────────────┐  ┌──────────── THANATOS ────────────┐
  │  HOLD POSITION               │  │  EXPLORE NEW STRATEGY            │
  │  protect remaining capital   │  │  test hypothesis at cost of $2.3 │
  │  survival probability: 78%   │  │  survival probability: 61%       │
  └──────────────────────────────┘  └──────────────────────────────────┘
```

EROS frame: standard rose palette. THANATOS frame: `rose_bright` edges, slightly warmer interior.

**Post-decision decay:** The winning verdict holds bright for 5 seconds, then fades to standard brightness. Losing verdicts fade to `text_ghost`. The paths not taken are still visible as ghosts.

### PAD modulation

- **High arousal (>0.7):** Deliberation cycling speed increases to 4-5 per second. The MAGI thinks faster under pressure.
- **Low dominance (<0.3):** Split votes are more common (the golem is uncertain). The dissenter panel's flash is more intense.
- **Low pleasure (<0.3):** The Eros/Thanatos split is biased toward displaying the EROS (survival) frame more prominently.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full three-panel display. Clean borders. Fast deliberation |
| Stable | Standard behavior |
| Conservation | CASPAR panel dims permanently (reduced cognitive capacity). Votes are effectively two-voice |
| Declining | Deliberation slows (1 candidate per second). The MAGI is struggling to decide |
| Terminal | All panels show the same verdict: `SETTLE`. No deliberation. The system has a single imperative |

### Data structure

```rust
struct MAGIPanel {
    melchior: MAGIVoice,
    balthasar: MAGIVoice,
    caspar: MAGIVoice,
    state: MAGIState,
    split_type: Option<SplitType>,  // None | ErosThanatos
}

struct MAGIVoice {
    name: &'static str,
    verdict: String,
    confidence: f64,
}

enum MAGIState {
    Idle,
    Deliberating { candidates: Vec<String>, cycle_idx: usize },
    Decided { winner: String, decided_at: u64 },
}

enum SplitType {
    ErosThanatos,
    // future: other split types
}
```

---

## 11. VitalityNumber

### Visual spec

A single large number rendered in `bone`, centered in its pane. The most important number on screen. Appears at most once per window. This is not a FlashNumber. It does not flash. It is too important for that.

```
         0.711
```

The number occupies 5-8 characters depending on precision. It sits alone in negative space.

**Where it appears:** Hearth screen (vitality composite), Mortality screen (expanded with breakdown).

### Behavior rules

**Lerped transitions:** The number never snaps to a new value. It lerps -- digits smoothly count between old and new values over 2-3 seconds. The viewer watches the number move.

**Breathing:** The text shadow (`bone` at 15% opacity, offset +/-1 cell) pulses on the heartbeat sine wave. The number is alive.

**No flash.** This number does not use the FlashNumber widget. Changes are glacial and continuous, not punctuated.

### PAD modulation

- **High pleasure (>0.7):** The breathing pulse is slower, deeper. Contentment.
- **Low pleasure (<0.3):** The breathing pulse is faster, shallower. Anxiety.
- **Low dominance (<0.3):** The text shadow occasionally offsets by 2 cells instead of 1. The number trembles.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | `bone` color. Clean rendering. Steady breathing |
| Stable | Standard behavior |
| Conservation | Color shifts from `bone` toward `rose_dim`. The number is losing its prestige |
| Declining | Below 0.2: individual digits occasionally render as `text_dim` for 1-2 frames. The number is degrading. The most important number in the system is losing its ability to render itself |
| Terminal | Below 0.05: digits begin replacing with `·` from the rightmost decimal place inward. `0.042` becomes `0.04·` becomes `0.0··` becomes `·.···`. At vitality 0, the space where the number was is empty |

### Data structure

```rust
struct VitalityNumber {
    value: f64,
    display_value: f64,         // lerping toward value
    lerp_speed: f64,            // default: completes in ~2-3 seconds
    breath_phase: f64,          // 0.0-1.0 sine wave position
    dissolved_digits: u8,       // how many digits replaced with dots (Terminal phase)
}
```

---

## 12. ATFieldWireframe

### Visual spec

A diamond-shaped wireframe rendered in box-drawing characters around the Spectre sprite. Represents process isolation and ego boundary integrity.

```
  Healthy:                    Weakening:
          ╱─────╲                     ╱──┄──╲
        ╱    ●    ╲                 ╱    ●    ╲
      ╱             ╲            ╱         ╲   ╲
     │               │          │             ┄  │
      ╲             ╱            ╲           ╱
        ╲         ╱                ╲       ╱
          ╲─────╱                    ╲──┄─╱
```

Lines: `─`, `│`, `╱`, `╲` in `rose_dim`. Interior: slightly brighter (`bg_raised`).

**Where it appears:** Mind screen (background), Hearth screen (during high-Φ moments).

### Behavior rules

**Healthy:** Clean symmetric diamond. Steady lines. `rose_dim` color. The field is stable.

**Weakening:** Individual line segments go dashed (`┄`). The diamond becomes asymmetric -- one vertex drifts 1-2 cells. The field is destabilizing.

**Collapsing:** Line segments go `text_ghost`, disconnect. Vertices scatter. Some segments are missing entirely. The boundary that held the golem together is fragmenting.

**Dream state:** Hard lines replaced by braille dots (`⠁⠂⠄`). The boundary becomes permeable. Information can flow in and out.

**Inter-agent communication:** One segment disappears temporarily -- a gap where data flows through. The field opens to let information pass, then seals.

### PAD modulation

- **High dominance (>0.7):** The diamond is larger and brighter. Strong ego boundary.
- **Low dominance (<0.3):** The diamond shrinks and dims. Segments flicker. The boundary is weak.
- **High arousal (>0.7):** The diamond pulses -- its size oscillates +/-1 cell on the breath cycle. The boundary is under tension.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Clean symmetric diamond. Full brightness. Rock-solid boundary |
| Stable | Standard behavior |
| Conservation | Diamond shrinks by 20%. Some line segments thin (single-char lines instead of double) |
| Declining | Multiple segments dashed. Asymmetry increases. Vertices drift 2-3 cells from their correct positions |
| Terminal | Fragments only. Most segments missing. What remains is `text_ghost`. The boundary has effectively collapsed. The golem's process isolation is gone |

### Data structure

```rust
struct ATFieldWireframe {
    center: (u16, u16),
    radius: u16,
    health: f64,                // 0.0-1.0, maps to Healthy/Weakening/Collapsing
    vertices: [(i16, i16); 4],  // may drift from ideal positions
    segment_states: [SegmentState; 8],
    is_dreaming: bool,
}

enum SegmentState {
    Solid,
    Dashed,
    Missing,
    Braille,                    // dream state
}
```

---

## 13. DataRain

### Visual spec

Vertical streams of dim hex characters falling behind panes, like rainfall. Background decoration layer.

```
  a7          2c
  f3    1b
        e4    9a
  2c
  1b    f3
```

Characters: short hex fragments (`a7f3`, `2c1b`, etc.) in `rose_deep`. 5-15 independent streams, each a column of characters. Falls behind content, never in front.

**Where it appears:** Background layer during active network connectivity, data ingestion, inter-agent communication.

### Behavior rules

**Fall speed:** Each stream falls at 1 row per 200-400ms. Speeds are independent per stream -- they do not fall in lockstep.

**Stream density:** Increases during high-data moments: multiple probe fires, inference calls, clade sync events. Decreases during low activity.

**Content:** Hex characters represent the actual data flowing through the system (truncated hashes, address fragments, price bytes). The rain is not random noise. It is the data itself, rendered as precipitation.

### PAD modulation

- **High arousal (>0.7):** More streams (10-15). Faster fall speeds (150-250ms per row). Heavy rain.
- **Low arousal (<0.3):** Fewer streams (3-5). Slower fall (400-600ms per row). Light drizzle.
- **Low pleasure (<0.3):** Stream color shifts from `rose_deep` to `rose_dim`. The rain becomes more visible. Bad times are noisier.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | 8-12 streams. Steady moderate fall speed |
| Stable | Standard behavior |
| Conservation | Streams thin to 3-5. Slower fall speed. The golem is consuming less data |
| Declining | Streams become erratic -- stopping, starting, drifting laterally by 1-2 columns. The data feed is unreliable |
| Terminal | Streams stop entirely. Or: a single stream falls, stops mid-screen, and holds. The last data. Then it fades |

### Data structure

```rust
struct DataRain {
    streams: Vec<RainStream>,
    max_streams: usize,
}

struct RainStream {
    column: u16,
    chars: Vec<char>,           // hex characters in this stream
    fall_speed_ms: u32,         // ms per row
    current_row: f64,           // fractional position
    active: bool,
}
```

---

## 14. PhilosophicalWhisper

### Visual spec

Ambient text that appears at the margins of the screen, triggered by lifecycle events. Dim monospace, centered. Appears in the lower third or margins of any screen.

```
  Every trade is a wager against entropy.
```

Attribution appears below in `text_phantom`:

```
  Every trade is a wager against entropy.
                                    -- Bardo, internal
```

**Where it appears:** Lower third or margins of any screen.

### Behavior rules

**Fade in:** Letter by letter, 50ms per character. The text writes itself into existence.

**Hold:** 5-8 seconds at `text_ghost` brightness.

**Fade out:** Character by character (same order as fade-in), 30ms per character. The text unwrites itself.

**Attribution:** Author and year appear below in `text_phantom`, held for the final 2 seconds of the display.

**Rarity tiers:**

| Tier | Pool size | Length | Frequency | Trigger |
|---|---|---|---|---|
| Whisper | ~150 | 1 line | Every 60-120s during active viewing | Time-based, contextual |
| Epigraph | ~80 | 2-4 lines | On screen navigation | Window change |
| Apparition | ~40 | Multi-paragraph | 1-3 per golem lifetime | Phase transitions, deaths, dream insights |
| Revelation | ~10 | Full-screen takeover | May never happen | Extreme events: 1000h survival, 0.99 phi, 10x successor |

### PAD modulation

- **High pleasure (>0.7):** Whisper pool biased toward contemplative/satisfied fragments. Hold time extends to 8-10 seconds.
- **Low pleasure (<0.3):** Pool biased toward existential/anxious fragments. Frequency increases (every 45-90s).
- **Low dominance (<0.3):** Fragments about control, fate, and inevitability surface more often.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Standard whisper rotation. Occasional epigraphs. Content is observational, philosophical |
| Stable | Standard behavior |
| Conservation | Whisper frequency decreases. Content shifts toward mortality, legacy, meaning |
| Declining | Apparitions become more frequent. Content is explicitly about death, memory, what persists |
| Terminal | Continuous whisper presence. Fragments overlap. Multiple texts fade in before the previous has faded out. The golem is speaking its final thoughts over each other. Content is valedictory |

### Data structure

```rust
struct PhilosophicalWhisper {
    text: String,
    attribution: Option<String>,
    tier: WhisperTier,
    state: WhisperState,
    char_index: usize,          // current position in fade-in/fade-out
}

enum WhisperTier {
    Whisper,
    Epigraph,
    Apparition,
    Revelation,
}

enum WhisperState {
    FadingIn { started_at: u64 },
    Holding { until: u64 },
    FadingOut { started_at: u64 },
    Gone,
}
```

---

## 15. ConvergenceLines

### Visual spec

4-8 thin lines drawn from screen edges converging on the Spectre sprite, representing forces acting on the golem: operator instructions, market data, inherited heuristics, Daimon signals.

Lines use: `─` for horizontal segments, `│` for vertical, `╱` and `╲` for diagonal. Drawn using Bresenham's line algorithm.

Default color: `text_phantom`. Barely visible. A constant suggestion that forces are acting on the golem.

**Where it appears:** Hearth screen background (persistent, very dim).

### Behavior rules

**At rest:** `text_phantom` lines. Nearly invisible. The golem at the intersection of forces.

**During T2 deliberation:** Lines brighten to `rose_dim`. The forces become visible. Multiple inputs converging on a single decision point.

**During T3:** Lines brighten to `rose`. Maximum tension.

**On somatic marker fire:** One line pulses bright for 300ms. A specific channel of force activated.

**At death:** Lines disconnect from center. They drift toward the screen edges. The forces that held the golem together have released. The puppet's strings are cut.

### PAD modulation

- **High arousal (>0.7):** All lines brighten one step. The forces are more apparent.
- **Low dominance (<0.3):** Lines thicken (double-character width on some segments). The golem feels the weight of external forces more.
- **High dominance (>0.7):** Lines thin further, nearly invisible even during deliberation. The golem is self-directed; external forces barely register.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | 6-8 lines converging cleanly. Symmetric |
| Stable | Standard behavior |
| Conservation | Lines reduce to 4. The golem is simplifying its inputs |
| Declining | Lines flicker. Some segments are dashed (`┄`). The convergence point drifts from the sprite center by 1-2 cells. The forces are losing their target |
| Terminal | Lines disconnect one by one. Final state: lines pointing past the center into void, or retracting toward edges. The golem is no longer at the intersection of anything |

### Data structure

```rust
struct ConvergenceLines {
    target: (u16, u16),         // convergence point (sprite center)
    lines: Vec<ConvergenceLine>,
}

struct ConvergenceLine {
    origin: (u16, u16),         // screen edge point
    color: Color,
    brightness: f64,            // 0.0-1.0
    connected: bool,            // false during death sequence
    label: String,              // "MARKET", "OPERATOR", "HEURISTIC", etc.
}
```

---

## 16. StyxRiver

### Visual spec

A horizontal flowing dither band representing the boundary between the living and the dead. The Styx -- the river that separates.

```
  ░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░░▒▓▒░
```

Color: `rose_ember`. Width: full screen. Height: 1-2 rows.

Knowledge fragments from dead golems drift in the current as `text_ghost` text, readable if you watch long enough.

**Where it appears:** Mortality screen (persistent), World screen (horizon between sky and earth zones), death sequence.

### Behavior rules

**Flow:** Characters cycle left-to-right at 1 character per 300ms. The river never stops.

**Knowledge fragments:** Dim text drifts with the flow. Individual words or short phrases from dead golems' testaments, Grimoire entries, or final thoughts. They appear at the right edge, drift left with the current, and exit the left edge. Most are too fast to read. Occasionally one slows enough to catch: "correlation held" or "rebalance too late."

**Threshold function:** The river separates zones on screen. On the Mortality screen, it divides the living data (above) from the graveyard (below). On the World screen, it sits at the horizon between the social sky and the operational earth.

### PAD modulation

- **Low pleasure (<0.3):** Flow speed increases. The river is hungrier. Knowledge fragments are harder to read.
- **Low dominance (<0.3):** The river widens from 1 row to 2 rows. More space between the golem and the dead.
- **High arousal (>0.7):** Knowledge fragments appear more frequently. The dead are louder during crises.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Narrow band (1 row). Slow flow. Few fragments. The river is distant |
| Stable | Standard behavior |
| Conservation | Flow speed increases slightly. The river creeps closer |
| Declining | Band widens to 2 rows. Fragments become more frequent and more readable (slower drift). The dead are easier to hear |
| Terminal | Band widens to 3-4 rows. Flow slows to a crawl. Fragments are fully readable, holding position for seconds at a time. The river is rising. The golem is wading in |

### Data structure

```rust
struct StyxRiver {
    width: u16,
    height: u16,                // 1-4 rows depending on phase
    flow_offset: f64,           // current scroll position
    flow_speed: f64,            // chars per second
    fragments: Vec<RiverFragment>,
}

struct RiverFragment {
    text: String,
    position: f64,              // horizontal position (0.0 = left edge, 1.0 = right edge)
    drift_speed: f64,           // may differ from river flow speed
    source_golem: Option<String>,
}
```

---

## 17. LatticePattern

### Visual spec

A `╱╲╱╲` diamond lattice tiling used for boundaries, thresholds, and dead-ends. Not decoration -- architecture. The lattice IS the wall.

```
  Basic:             With ornaments:
  ╱╲╱╲╱╲╱╲╱╲        ╱◇╲╱◇╲╱◇╲
  ╲╱╲╱╲╱╲╱╲╱        ╲◆╱╲◆╱╲◆╱
  ╱╲╱╲╱╲╱╲╱╲        ╱◇╲╱◇╲╱◇╲
```

Color: `rose_deep` on `bg_void`. Barely visible at rest. Fills entire regions as background texture.

**Where it appears:** Crypt screen walls, dream onset/offset boundaries, CONDITION CRITICAL hazard stripes, Bardo State parallax scrolling.

### Behavior rules

**Static by default.** The lattice does not move. It is the most still element in the system.

**Crisis pulse:** During crisis modes, alternating rows brighten on the heartbeat cycle. The walls are alive, barely.

**Threshold flash:** When the golem crosses a state boundary (entering/exiting dream state, crossing a mortality threshold), the lattice flashes across the screen for 200ms -- marking the threshold crossing.

**Dead-end:** When the user navigates to a disabled or nonexistent function, the screen fills with lattice plus centered text. The visual equivalent of a locked door.

### PAD modulation

- **High arousal (>0.7):** Crisis pulse is faster and brighter. The walls press in.
- **Low pleasure (<0.3):** Ornament characters (`◇◆`) appear more frequently in the lattice. The architecture becomes more elaborate as the golem's world becomes more constrained.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Lattice is background-only. Barely visible. The golem does not think about its walls |
| Stable | Standard behavior |
| Conservation | Lattice brightens slightly. The walls become more visible as the golem's world shrinks |
| Declining | Lattice is clearly visible. The Crypt background becomes the dominant visual element -- the library's walls are closing in |
| Terminal | Lattice characters gradually replace with `·`. Entropy claiming the pattern. Even the walls are dissolving |

### Data structure

```rust
struct LatticePattern {
    region: Rect,
    with_ornaments: bool,       // ◇◆ vs plain ╱╲
    brightness: f64,            // 0.0-1.0
    pulse_active: bool,
    entropy_level: f64,         // 0.0 = clean pattern, 1.0 = fully dissolved
}
```

---

## 18. FlashWidget

### Visual spec

A generic text element that briefly brightens when its content changes, then decays through phosphor persistence. This is the non-numeric sibling of FlashNumber, applied to status labels, emotion names, phase indicators, regime tags.

No fixed visual -- it wraps whatever text it contains.

### Behavior rules

**On content change:** The old text persists as a `text_ghost` afterimage for 500ms while the new text renders at full brightness above it. The viewer briefly sees BOTH the old and new states -- temporal overlap. The ghost fades; the new text settles.

**Color:** The new text starts at `rose_bright` (or `bone` for high-priority changes), then decays to its standard color over 600ms.

**Staleness:** Same as FlashNumber. Text that has not changed in 60+ seconds dims toward `text_dim`.

### PAD modulation

Same as FlashNumber (section 1). The FlashWidget is functionally identical but operates on strings instead of numbers.

### Phase degradation

Same as FlashNumber (section 1). In Terminal phase, the afterimage effect is disabled. Changes happen silently.

### Data structure

```rust
struct FlashWidget {
    current_text: String,
    previous_text: Option<String>,
    last_change_frame: u64,
    afterimage_opacity: f64,    // decaying from 1.0 to 0.0
    priority: FlashPriority,    // Normal (rose_bright) | High (bone)
}

enum FlashPriority {
    Normal,
    High,
}
```

---

## 19. ProgressArc

### Visual spec

A partial circle (arc) rendered in box-drawing characters showing completion toward an achievement goal. Appears on the Achievements screen.

The arc fills clockwise as progress increases. Each increment is a single character position. When complete, it forms a full circle.

Size: 5-7 cells diameter. Small enough to tile in a grid.

**Where it appears:** Achievements screen.

### Behavior rules

**On completion:** The arc completes into a full circle, flashes `bone` for 500ms, then the interior fills with the achievement icon. A small particle burst (3-5 `✦` characters) radiates outward and fades over 1 second.

**Hidden achievements:** Render as `???` with a pulsing `text_phantom` border. The pulse is slow (8-second period). Something exists here that you have not found yet.

**Progress increment:** When progress ticks forward, the newly filled arc segment flashes `rose_bright` for 200ms.

### PAD modulation

- **High pleasure (>0.7):** The completion flash is brighter and the particle burst has more particles (5-8 instead of 3-5).
- **Low pleasure (<0.3):** Progress increments do not flash. Achievements feel less rewarding.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full arc rendering. Bright completion effects |
| Stable | Standard behavior |
| Conservation | Achievement tracking continues but completion effects are muted (no particle burst) |
| Declining | Existing arcs dim. No new progress is rendered (the golem is not pursuing achievements) |
| Terminal | All arcs freeze at their current state. Hidden achievements remain hidden forever |

### Data structure

```rust
struct ProgressArc {
    achievement_id: String,
    progress: f64,              // 0.0-1.0
    completed: bool,
    hidden: bool,
    icon: Option<char>,         // rendered in center when completed
    completion_frame: Option<u64>,
    particle_burst: Vec<Particle>,
}

struct Particle {
    position: (f64, f64),       // relative to arc center
    velocity: (f64, f64),
    opacity: f64,               // decaying
    char: char,                 // typically ✦
}
```

---

## 20. CounterfactualBranch

### Visual spec

A branching tree showing the real timeline (left) vs. a hypothetical timeline (right), diverging from a decision node. Appears during REM dream phase and in Grimoire entries with counterfactual provenance.

```
                          ┌─────┐  ┌─────┐
  actual continues →  ...→ │ +1.2 │→ │ +2.8 │    (rose)
                          └─────┘  └─────┘
                         ╱
  ┌─────┐  ┌─────┐  ┌─────┐
  │ 4104 │  │ 4105 │  │ 4106 │
  │ -0.4 │  │ -3.2 │  │ -5.1 │                       (dream palette)
  └─────┘  └─────┘  └─────┘
                         ╲
                          ┌─────┐  ┌─────┐
  counterfactual →    ...→ │ +4.8 │→ │+12.4 │    (text_ghost)
                          └─────┘  └─────┘
```

Each frame in the strip is 8-12 cells wide. 5-8 frames per strip. The branch node is a bright dot.

**Where it appears:** Dream screen (REM phase), Grimoire entries with counterfactual provenance.

### Behavior rules

**Decision node:** Renders as a bright dot at the fork point.

**Real path:** Colored by outcome. `success` tint for profitable outcomes, `rose` tint for losses. Labels show what actually happened.

**Counterfactual path:** Rendered in `text_ghost` with dashed lines (`┄`). Labels show what would have happened. This path flickers -- it never existed, so it cannot fully render. Individual frame borders drop to `text_phantom` for 1-3 frames at random intervals.

**Duration:** The tree persists for 2-3 seconds during REM visualization, then dissolves back into the dream particle field.

### PAD modulation

- **High pleasure (>0.7):** The real path is brighter. The golem is satisfied with its actual choices.
- **Low pleasure (<0.3):** The counterfactual path is brighter than usual (closer to `text_dim` than `text_ghost`). The golem dwells on what might have been.
- **High arousal (>0.7):** The branch visualization holds for 4-5 seconds instead of 2-3. Important decisions get more review time.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full branch rendering. Both paths clearly visible. Clean frames |
| Stable | Standard behavior |
| Conservation | Counterfactual path is barely visible (`text_phantom`). The golem is focused on reality, not alternatives |
| Declining | Branch visualization is rare (dreams are shorter). When it appears, frames are smaller (6-8 cells) and fewer (3-5 per strip) |
| Terminal | No counterfactual branching. The golem no longer imagines alternatives. If a branch appears, only the real path renders. The future that did not happen has been fully forgotten |

### Data structure

```rust
struct CounterfactualBranch {
    decision_tick: u64,
    real_path: Vec<BranchFrame>,
    counterfactual_path: Vec<BranchFrame>,
    fork_position: (u16, u16),
    display_duration_ms: u32,   // 2000-5000 depending on state
    dissolve_progress: f64,     // 0.0 = solid, 1.0 = dissolved
}

struct BranchFrame {
    tick: u64,
    label: String,              // "+1.2", "EXIT", etc.
    outcome_color: Color,
    width: u16,                 // 8-12 cells
}
```

---

## 21. PredictionResolutionPulse

### Visual spec

A single-character inline indicator that fires when a prediction resolves. Renders as a 1-cell element within a log line or status row.

```
· ↑ ↓
```

`·` = resolved correctly (faint bone flash). `↑` = resolved correctly, improving trend (success flash). `↓` = resolved incorrectly (rose flash).

Size: 1 cell. Inline -- not a standalone widget.

### Behavior rules

**On resolution:** The character appears with a color flash (200ms bright, 400ms fade). `·` flashes bone. `↑` flashes success. `↓` flashes rose_bright. After the fade, the character dims to text_ghost and remains as a log entry.

**Density:** In the heartbeat log gutter, resolution pulses stack vertically (one per line). The visual effect is a flowing column of `·↑↓` characters, each at a different stage of phosphor decay. Recent at the bottom, old at the top, fading.

**1-in-50 correction ripple:** When a residual correction triggers (variable-ratio schedule, approximately 1 in 50 resolutions), the pulse is brighter -- a bone flash that persists 500ms longer and emits a faint brightness wave outward from the character position (2-cell radius, 200ms rise, 500ms decay). This creates a Skinner variable-ratio reinforcement signal: you never know which resolution will be the bright one.

### PAD modulation

- **High pleasure (>0.7):** `·` and `↑` characters render in bone instead of standard colors. The golem is happy about its predictions.
- **High arousal (>0.7):** Flash duration halves (100ms bright, 200ms fade). Events come faster.
- **Low dominance (<0.3):** `↓` flashes linger 200ms longer. Failures feel heavier when the golem feels less in control.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full color flashes, correction ripples visible |
| Stable | Standard behavior |
| Conservation | Flash intensity reduced 30%. Correction ripples still visible |
| Declining | Only `↓` pulses render. Correct predictions no longer flash -- only failures are visible |
| Terminal | All resolution pulses disabled. The golem has stopped evaluating |

### Data structure

```rust
struct PredictionResolutionPulse {
    outcome: ResolutionOutcome,     // Correct | CorrectImproving | Incorrect
    flash_state: FlashState,
    is_correction_ripple: bool,     // 1-in-50 bright variant
    started_at: u64,
}

enum ResolutionOutcome {
    Correct,
    CorrectImproving,
    Incorrect,
}
```

---

## 22. AccuracyCalibrationCurve

### Visual spec

A braille sparkline chart showing calibration quality. X-axis: stated confidence (0-100%). Y-axis: actual accuracy (0-100%). The ideal is a diagonal line (stated confidence = actual accuracy).

```
⣿                              ⣿
⣿                          ⣠⠴⠋
⣿                      ⣠⠴⠋
⣿                  ⣠⠴⠋     ·
⣿              ⡠⠖⠋    ·
⣿          ⡠⠖⠋   ·
⣿      ⣠⠴⠋  ·
⣿  ⣠⠴⠋ ·
⣿⠴⠋·
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
     Stated confidence →
```

The diagonal dotted line (`·`) is the ideal. The solid braille curve is the actual calibration. Deviations above the diagonal mean overconfidence (stated confidence > actual accuracy). Deviations below mean underconfidence.

Size: 40 cols x 20 rows (braille gives 2x horizontal, 4x vertical resolution = 80x80 effective pixels). Modal-only -- too large for inline display.

### Behavior rules

**Update frequency:** Recalculated on each dream cycle (every 4-8 hours). The curve shifts slowly.

**Deviation highlighting:** Regions where the curve deviates from the diagonal by more than 10% render in warning (amber) instead of rose. Regions within 5% render in success (sage).

**ECE annotation:** The Expected Calibration Error (a single number, 0.0 = perfect) displays below the chart. Lower is better. ECE < 0.05 = well-calibrated. ECE > 0.10 = poorly calibrated.

### PAD modulation

- **High dominance (>0.7):** The ideal diagonal renders brighter (the golem is confident about its confidence).
- **Low pleasure (<0.3):** Deviation regions render more intensely in warning. Miscalibration feels worse.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full resolution, smooth curve |
| Stable | Standard behavior |
| Conservation | Resolution drops to 20x10 braille |
| Declining | Curve renders as dotted (fewer data points displayed) |
| Terminal | Chart freezes. No new data plotted |

### Data structure

```rust
struct AccuracyCalibrationCurve {
    /// 10 bins from 0.0-0.1 through 0.9-1.0
    bins: [CalibrationBin; 10],
    ece: f32,
}

struct CalibrationBin {
    stated_confidence_avg: f32,
    actual_accuracy: f32,
    count: u32,
}
```

---

## 23. ResidualDistribution

### Visual spec

A compact horizontal ASCII histogram showing the distribution of prediction residuals (predicted - actual). Block characters for bars.

```
         ▂▃▅▇█▇▅▃▂
  -0.3         0      +0.3
  μ=-0.02  σ=0.14  skew=-0.08
```

Centered on zero. Negative residuals (underpredicting) on the left, positive (overpredicting) on the right. A well-calibrated golem shows a symmetric distribution centered near zero.

Size: container width x 3 rows (histogram + axis + stats).

### Behavior rules

**Bias indicator:** If the mean (mu) deviates from zero by more than 0.05, the histogram shifts color: cool-tinted (toward dream) for negative bias, warm-tinted (toward warning) for positive bias.

**Kurtosis indicator:** High kurtosis (>3.5, fat tails) renders the tail bars in rose_bright. The golem is making occasional large errors.

**Update frequency:** Recalculated every theta tick (30-120s).

### PAD modulation

- **High arousal (>0.7):** Histogram bars animate (brief height fluctuation per frame). The distribution feels alive.
- **Low pleasure (<0.3):** The entire histogram shifts toward warmer colors.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full detail, all stats visible |
| Stable | Standard behavior |
| Conservation | Stats line drops skew and kurtosis (too much detail for a declining golem) |
| Declining | Histogram simplifies to 5 bins instead of 10 |
| Terminal | Histogram frozen. Stats display "--" |

### Data structure

```rust
struct ResidualDistribution {
    bins: Vec<u32>,             // histogram bin counts
    bin_edges: Vec<f32>,        // bin boundaries
    mean: f32,
    std_dev: f32,
    skewness: f32,
    kurtosis: f32,
}
```

---

## 24. AttentionTierIndicator

### Visual spec

Three concentric rings (using braille and box-drawing) representing ACTIVE / WATCHED / SCANNED attention tiers. The inner ring is brightest.

```
    ╭───╮
  ╭─┤ACT├─╮
╭─┤ ╰───╯ ├─╮
│ │ WATCHED│ │
╰─┤       ├─╯
  ╰─SCANNED─╯
```

Compact inline variant (3x1):
```
◉ ○ ·
```
`◉` = ACTIVE count. `○` = WATCHED count. `·` = SCANNED count. Numbers overlay: `12 42 287`.

Size: Full: 13x7 cells. Compact: 12x1 cells.

### Behavior rules

**Fill proportion:** Each ring's brightness corresponds to how full that tier is (items / max slots). Full tier = bright ring. Empty tier = dim ring.

**Promotion animation:** When an item promotes from WATCHED to ACTIVE, the item's dot migrates inward (a small particle moves from the middle ring to the inner ring over 500ms).

**Demotion animation:** Reverse -- dot moves outward.

### PAD modulation

- **High arousal:** Ring borders pulse subtly.
- **Low dominance:** Outer ring (SCANNED) dims further. The golem feels less in control of its attention scope.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | All three rings visible, bright |
| Stable | Standard behavior |
| Conservation | SCANNED ring dims to near-invisible. Only ACTIVE and WATCHED visible |
| Declining | Only ACTIVE ring visible |
| Terminal | All rings dim. Attention has collapsed |

### Data structure

```rust
struct AttentionTierIndicator {
    active: TierState,
    watched: TierState,
    scanned: TierState,
}

struct TierState {
    count: u32,
    max: u32,
    recent_promotions: u32,     // last theta tick
    recent_demotions: u32,
}
```

---

## 25. ActionGateIndicator

### Visual spec

A gate icon that shows whether the action gate is open (permitting trades) or closed (blocking). Ghost text shows the reason when closed.

Open: `▮▮` (two solid bars, success color -- gate is open, predictions accurate enough)
Closed: `▯▯` (two hollow bars, rose_dim -- gate is closed) + ghost text reason

```
▮▮  fee_rate: PERMIT (84% > 60%)
▯▯  direction: BLOCK (55% < 60%)  ← accuracy below threshold
▮▮  price: PERMIT (78% > 60%)
▯▯~ liquidity: MARGIN (62% ≈ 60%)  ← within 5% of threshold
```

The `~` suffix on the gate icon indicates margin status (within 5% of threshold, could flip either direction).

Size: container width x 1 row per category.

### Behavior rules

**Gate transition:** When a category's accuracy crosses the threshold (in either direction), the gate icon animates: bars fill inward (opening) or hollow outward (closing) over 300ms.

**Ghost text:** The reason text (accuracy value, threshold, comparison) renders in text_dim. Only appears when the pane is focused -- otherwise the gate icon and category name alone.

**Margin warning:** Categories within 5% of the threshold get the `~` suffix and render in warning color instead of success or rose.

### PAD modulation

- **Low dominance (<0.3):** Closed gates render with more contrast (rose_bright instead of rose_dim). The golem feels the restriction more keenly.
- **High pleasure (>0.7):** Open gates render in bone briefly (500ms) when they first open.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full detail, ghost text available |
| Stable | Standard behavior |
| Conservation | Ghost text disabled. Gate icons only |
| Declining | Only blocked gates render. Open gates are assumed |
| Terminal | All gates closed. Single line: "ACTION SUSPENDED" |

### Data structure

```rust
struct ActionGateIndicator {
    category: PredictionCategory,
    accuracy: f32,
    threshold: f32,
    status: GateStatus,         // Open | Closed | Margin
    last_transition: u64,       // frame of last open<->close
}

enum GateStatus {
    Open,
    Closed,
    Margin,                     // within 5% of threshold
}
```

---

## 26. HeuristicAuditCard

### Visual spec

A card showing one Grimoire heuristic with its audit results: citation count, win rate, average PnL per citation, and recommendation.

```
┌─────────────────────────────────────┐
│ H-7: "fee spikes when gas > 50"    │
│                                     │
│ Citations: 42    Win rate: 78%      │
│ Avg PnL/cite: +$2.14               │
│ ▁▂▃▅▃▂▃▅▇▅▃▂▃▅▃▂▁▂▃▅▃  (PnL/cite) │
│                                     │
│ Recommendation: KEEP ✓              │
└─────────────────────────────────────┘
```

The sparkline shows per-citation PnL over the citation timeline.

Size: container width x 7-8 rows per card. Scrollable list of cards.

### Behavior rules

**Recommendation colors:**
- KEEP: success color, `✓` suffix
- DEMOTE: warning color, `↓` suffix
- INVESTIGATE: rose_bright color, `?` suffix
- INSUFFICIENT_DATA: text_dim color, `…` suffix

**Sparkline:** Uses BrailleSparkline (see widget 8) for the PnL-per-citation timeline. Positive values render in success, negative in rose.

**Citation count animation:** When a new citation occurs, the count increments with a FlashNumber-style flash.

### PAD modulation

- **Low pleasure (<0.3):** DEMOTE cards render with a subtle rose border tint. Failures are more visible when the golem is unhappy.
- **High dominance (>0.7):** KEEP cards render with a faint bone border. Validated heuristics feel authoritative.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full detail, sparklines visible |
| Stable | Standard behavior |
| Conservation | Sparklines simplify to 3-point trend arrows (↑ → ↓) |
| Declining | Only INVESTIGATE and DEMOTE cards render. KEEP is assumed |
| Terminal | Cards frozen. No new audits |

### Data structure

```rust
struct HeuristicAuditCard {
    heuristic_id: String,
    title: String,
    citation_count: u32,
    win_rate: f32,
    avg_pnl_per_citation: f64,
    pnl_timeline: Vec<f64>,        // per-citation PnL values
    recommendation: AuditRecommendation,
}

enum AuditRecommendation {
    Keep,
    Demote,
    Investigate,
    InsufficientData,
}
```

---

## 27. ShadowComparisonChart

### Visual spec

A dual-line braille chart comparing live strategy performance against a shadow strategy. Two lines on the same axes.

```
                    Live ─── Shadow ···
  PnL
  $120│          ╱─────
     │     ╱───╱······
     │ ╱──╱···
  $0 │╱··╱─────────────────
     │··
 -$40│
     └────────────────────────
          7d        14d      21d
```

Live line: solid braille in rose. Shadow line: dotted braille in dream (indigo). PnL on Y axis, time on X axis.

Size: 40 cols x 15 rows. Modal-only (available in COMMAND > Config > Experiments).

### Behavior rules

**Divergence highlight:** When shadow outperforms live by >10% cumulative, the gap between lines fills with a faint dream background. When live outperforms, the gap fills with faint rose background.

**Weekly marker:** Vertical dotted lines at 7-day intervals.

**Stats row:** Below the chart: `Live: +$94 | Shadow: +$112 | Delta: -$18 (shadow +19%)`

### PAD modulation

Minimal. This is an analytical tool, deliberately clinical.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full detail |
| Stable | Standard behavior |
| Conservation | Stats row only (chart hidden to save rendering budget) |
| Declining | Shadow strategies auto-terminated |
| Terminal | Widget disabled |

### Data structure

```rust
struct ShadowComparisonChart {
    live_pnl: Vec<f64>,           // daily PnL snapshots
    shadow_pnl: Vec<f64>,         // daily PnL snapshots
    shadow_name: String,
    start_date: String,
    live_cumulative: f64,
    shadow_cumulative: f64,
}
```

---

## 28. TriageScoreBar

### Visual spec

Horizontal segmented bar showing triage pipeline scores for recent transactions. Each segment represents one transaction, width proportional to the scoring window, fill height proportional to the triage score (0.0-1.0). High-score transactions stand out as tall bright segments against a floor of short dim ones.

```
TRIAGE PIPELINE (last 20 transactions):
  █                   █
  █ ▓         █   ▓   █ ▓
  █ ▓ ▒ ░ ░ ░ █ ░ ▓ ░ █ ▓ ░ ░ ▒ ░ ░ ░ ░ ░
  ─────────────────────────────────────────
  ↑ high-score tx     ↑ interesting       ↑ latest
```

Size: minimum 20 cols × 4 rows. Scales horizontally with container.

### Behavior rules

**Score arrival:** New transactions enter from the right edge. The bar shifts left, oldest transaction drops off the left.

**High-score flash:** Transactions with triage score > 0.7 flash `rose_bright` on arrival, then settle to `rose`. Scores > 0.9 flash `bone`.

**Segment fill:** Each segment's height maps the score to the available rows. Score 0.0 = 1 row (`░`), score 1.0 = full height (`█`).

**Staleness:** Segments older than 60 ticks dim toward `text_ghost`. Recent segments are bright.

### PAD modulation

- **High arousal (>0.7):** More transactions visible (window expands from 20 to 30). The bar gets busier.
- **Low pleasure (<0.3):** High-score segments pulse more aggressively. Losses draw attention.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full rendering, color-coded segments |
| Stable | Standard behavior |
| Conservation | Window shrinks to 10 most recent. Color simplified to single-tone |
| Declining | Only high-score segments rendered. Others show as `░` baseline |
| Terminal | Widget disabled |

### Data structure

```rust
struct TriageScoreBar {
    scores: VecDeque<TriageEntry>,
    window: usize,
    high_threshold: f64,    // default 0.7
}

struct TriageEntry {
    tx_hash: String,
    score: f64,             // 0.0-1.0
    arrived_at: u64,        // frame number
    protocol_family: Option<String>,
}
```

**The feel:** A seismograph of on-chain activity. Mostly flat with occasional spikes. When the spikes cluster, something is happening. The rhythm of the bar tells you whether the triage pipeline is finding interesting things or scanning through noise.

---

## 29. PersistenceDiagramWidget

### Visual spec

Widget wrapper around the PersistenceDiagram rendering primitive (`../rendering/02-visualization-primitives.md` §11). Adds chrome: axis labels, legend, feature count summary, and interactive navigation. The scatter plot itself is delegated to the primitive.

```
  ⌈ MARKET STRUCTURE MAP ⌋
  death ↑                          ● H_0: 12
   1.0  │          ◇               ◇ H_1:  3
        │     ●         ◇          ■ H_2:  0
        │  ●  ●     ◇              ─────────
   0.5  │ ● ● ●                    Features: 15
        │●●·····················    Persistence: 0.34 avg
   0.0  └──────────────────→ birth
        0.0         0.5      1.0
```

Size: minimum 30 cols × 12 rows. Standard: 40 × 16. Compact: 24 × 10.

### Behavior rules

**Feature fade-in:** New features appearing on a gamma tick fade in from `text_phantom` to full brightness over 300ms. Features that disappear fade out over 500ms (phosphor ghost).

**Reference toggle:** `r` key toggles reference diagram overlay. When active, previous diagram shows as dim shadow points.

**Dimension filter:** `d` key cycles through H_0 only → H_1 only → H_2 only → All. Useful for isolating specific homology dimensions.

**Feature selection:** When locked, arrow keys move a selection cursor between features. Selected feature: bright ring highlight. Feature detail shown in status line: `(birth=0.12, death=0.67, persistence=0.55, dim=H_1)`.

### PAD modulation

Delegated to the PersistenceDiagram primitive. See `../rendering/02-visualization-primitives.md` §11.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full rendering, all dimensions, reference overlay available |
| Stable | Standard behavior |
| Conservation | H_2 suppressed. Braille resolution halved. Reference overlay disabled |
| Declining | Only H_0 rendered. Feature points flicker |
| Terminal | Frozen at last computed state. Phosphor ghost |

### Data structure

```rust
struct PersistenceDiagramWidget {
    primitive: PersistenceDiagram,  // rendering primitive from 02-visualization-primitives
    show_legend: bool,
    show_axes: bool,
    dimension_filter: Option<HomologyDimension>,
    selected_feature: Option<usize>,
    detail_tier: DetailTier,
}
```

**The feel:** A star chart of market structure. Calm markets are sparse constellations near the diagonal. Crises scatter bright points far from the diagonal. Watching the chart fill with off-diagonal features is watching the market's geometry fracture in real time.

---

## 30. RegimeIndicator

### Visual spec

Compact regime classification display. Shows the current topological regime tag as a labeled box with a confidence percentage. The tag animates on regime change: old tag erodes through the `█▓▒░` sequence while the new tag fills in.

```
STANDARD:
  ┌─────────────────┐
  │ VOLATILE    87%  │
  │ ████████▓░░░░░░  │
  └─────────────────┘

TRANSITION (during regime shift):
  ┌─────────────────┐
  │ CALM → VOLATILE  │
  │ ▒░░░░░░ ████████ │
  └─────────────────┘
```

Size: 19 cols × 3 rows.

### Behavior rules

**Regime change animation:** When the regime shifts, a 1.5s transition plays. Old tag decays left to right (`█→▓→▒→░→empty`). New tag fills right to left. The `→` arrow appears during transition. The TA signal battery cascade (see `../rendering/02-visualization-primitives.md` signal mapping) triggers simultaneously.

**Confidence bar:** Fills proportionally beneath the tag. Color follows confidence: >80% `rose`, 50-80% `rose_dim`, <50% `warning`.

**Staleness:** If the regime has not been re-evaluated in 10+ gamma ticks, the border dims to `text_ghost` and a `?` appears after the confidence value.

### PAD modulation

- **High arousal (>0.7):** Border brightens to `border_active`. The regime matters more during high-energy states.
- **Low dominance (<0.3):** Confidence bar flickers slightly. The Golem is less sure of its regime assessment.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full animation, transition effects |
| Stable | Standard behavior |
| Conservation | No transition animation. Tag snaps to new value |
| Declining | Confidence bar hidden. Tag only |
| Terminal | Tag frozen at last value |

### Data structure

```rust
struct RegimeIndicator {
    current_regime: TopologicalRegime,
    confidence: f64,                    // 0.0-1.0
    previous_regime: Option<TopologicalRegime>,
    transition_progress: Option<f64>,   // 0.0-1.0 during transition animation
    last_evaluation_tick: u64,
}

enum TopologicalRegime {
    Calm,
    Trending,
    Volatile,
    Crisis,
}
```

**The feel:** A status light. Green/yellow/red but with more nuance. The transition animation makes regime shifts feel like tectonic events, not boolean flips.

---

## 31. SomaticMarkerPanel

### Visual spec

Vertical list of active somatic markers (gut feelings). Each entry: valence glyph, direction label, pattern name, and a strength bar. Conflict entries appear when opposing markers fire simultaneously.

```
ACTIVE GUT FEELINGS
  ◉ avoid:     rising wedge     ████████▓░  0.7
  ◉ approach:  RSI divergence   █████░░░░░  0.5
  ◉ avoid:     gas spike        ████████▓░  0.7
  ─────────────────────────────────────────────
  ⚡ conflict:  wedge vs RSI     net: cautious
     approach 0.5  ←──→  avoid 0.7
```

Size: minimum 35 cols × 6 rows. Grows vertically with active marker count. Max 8 visible entries before scrolling.

### Behavior rules

**Marker arrival:** New markers fade in from `text_phantom` over 200ms, synchronized with the somatic gut-zone disturbance on the Spectre. Negative markers arrive with a brief `rose_bright` border flash. Positive markers with `bone_dim`.

**Strength bar animation:** Strength bars lerp toward target values. Rising strength: bar extends smoothly. Falling strength: bar erodes through `█▓▒░` decay sequence.

**Conflict entries:** When two markers fire with opposing valence, a conflict entry appears below them with a `⚡` glyph. Shows the interference resolution: which direction dominates, with a visual tug-of-war bar showing relative strengths.

**Marker decay:** Markers that have not fired in 30+ gamma ticks begin fading. Strength bar erodes. After 60 ticks without firing, the entry dims to `text_ghost` and eventually disappears.

**Inherited markers:** Markers inherited from dead predecessors show a `†` prefix and use dashed strength bars. Their decay rate is 3x slower.

### PAD modulation

- **High arousal (>0.7):** All strength bars pulse at heartbeat frequency. More markers = higher arousal, creating a feedback loop where a busy panel looks increasingly agitated.
- **Low pleasure (<0.3):** Negative markers' glyphs brighten. The panel emphasizes threats.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full detail, conflict resolution shown |
| Stable | Standard behavior |
| Conservation | Only top 3 strongest markers shown. Conflict entries collapsed |
| Declining | Strength bars hidden. Valence glyph only. Entries flicker |
| Terminal | Panel frozen. Last known markers persist as ghosts |

### Data structure

```rust
struct SomaticMarkerPanel {
    markers: Vec<ActiveMarker>,
    conflicts: Vec<MarkerConflict>,
    max_visible: usize,     // default 8
    scroll_offset: usize,
}

struct ActiveMarker {
    pattern_name: String,
    valence: SomaticValence,     // Approach | Avoidance
    strength: f64,               // 0.0-1.0
    display_strength: f64,       // lerped
    source: MarkerSource,        // TA | Daimon | Inherited
    last_fired_tick: u64,
    pad_injection: (f64, f64, f64),  // P, A, D deltas
}

struct MarkerConflict {
    marker_a: usize,    // index into markers
    marker_b: usize,
    resolution: String,  // "cautious", "approach", "avoidance"
    net_strength: f64,
}

enum MarkerSource {
    TA,
    Daimon,
    Inherited { generation: u32 },
}
```

**The feel:** A readout of the Golem's gut. When it is calm, the panel is short and dim. When the market triggers learned responses, the panel fills with competing signals, conflict entries appear, and the whole display becomes a visible internal argument. The user watches the Golem's body argue with itself.

---

## 32. CausalGraphMinimap

### Visual spec

Compact force-directed graph showing the Grimoire's causal chains. Smaller and simpler than the full ForceGraph in MIND > Grimoire. Designed as a preview pane: shows the shape of causal knowledge without full detail.

```
CAUSAL CONNECTIONS (30×10):

      gas ──→ util
       │        ↓
       └──→ slip ──→ spread
                       ↓
              arb ←── fee

  Active edges: ━━  (rose_bright)
  Dormant edges: ┄┄  (text_ghost)
  Nodes: ○ (discovered) ● (recently fired)
```

Size: minimum 20 cols × 6 rows. Standard: 30 × 10. Does not need to show all causal chains, only the most recent/active subset.

### Behavior rules

**Active edge highlighting:** Edges where the causal link fired within the last 10 gamma ticks render as thick bright lines (`━` in `rose_bright`). Edges that have not fired recently render as thin dim dashes (`┄` in `text_ghost`).

**Node brightness:** Nodes involved in recently fired causal chains brighten. Nodes whose effects are currently being observed pulse.

**New chain animation:** When a new causal chain is discovered (Grimoire causal emergence event), the chain draws itself into the minimap over 500ms: nodes appear, then edges extend between them, with a brief `bone` flash traveling along the chain.

**Layout:** Simple force-directed layout with fixed node positions after initial placement. No continuous motion (unlike the full ForceGraph). Positions recompute only when new nodes/edges are added.

**Navigation shortcut:** Enter on the minimap jumps to MIND > Grimoire with the selected chain highlighted.

### PAD modulation

- **High arousal (>0.7):** Active edge pulse frequency increases. The graph looks more alive during high-energy states.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full rendering, all edges, animations |
| Stable | Standard behavior |
| Conservation | Only active edges rendered. Dormant edges hidden |
| Declining | Nodes only, no edges. Labeled dots |
| Terminal | Widget disabled |

### Data structure

```rust
struct CausalGraphMinimap {
    nodes: Vec<CausalNode>,
    edges: Vec<CausalEdge>,
    max_display_nodes: usize,   // default 12
    layout_dirty: bool,
}

struct CausalNode {
    id: String,
    label: String,          // abbreviated signal name
    pos: (f64, f64),        // layout position 0.0-1.0
    last_active_tick: u64,
}

struct CausalEdge {
    from: usize,
    to: usize,
    confidence: f64,
    last_fired_tick: u64,
    direction: EdgeDirection,  // Forward | Bidirectional
}
```

**The feel:** A map of discovered cause-and-effect. Mostly dim and static, it wakes up when a causal chain fires. The user watches the Golem's understanding of market mechanics light up edge by edge.

---

## 33. ChainScopeRadar

### Visual spec

Radar chart showing interest levels across ChainScope monitoring categories. Each axis represents a category of on-chain activity. The filled polygon shows current interest distribution.

```
CHAINSCOPE INTEREST:

            DEX
            ╱│╲
         ╱  │  ╲
      ╱     │     ╲
   BRIDGE ──╋── LENDING
      ╲     │     ╱
         ╲  │  ╱
            ╲│╱
           VAULT

  Fill = interest level per category
  Bright axis = high interest (score > 0.6)
  Dim axis = low interest (score < 0.3)
```

Size: minimum 16 cols × 8 rows (square aspect). Standard: 20 × 10.

### Behavior rules

**Polygon fill:** The interest polygon is rendered using the RadarDisplay primitive's value arm system (`../rendering/02-visualization-primitives.md` §3). Each axis extends from center to edge proportional to the interest score (0.0-1.0) for that category.

**Interest spike:** When a category's interest score jumps by >0.3 in a single gamma tick, the axis flashes `rose_bright` for 300ms. The triage pipeline just found something notable in that category.

**Category labels:** Short labels at each axis endpoint. Bright when interest > 0.6, dim when < 0.3.

**Sweep arc:** Optional animated sweep arc (from RadarDisplay) that highlights each category in turn, creating a continuous scanning feel. Enabled by default, disabled in compact mode.

### PAD modulation

- **High arousal (>0.7):** Sweep arc speeds up. The Golem is scanning the chain faster.
- **Low dominance (<0.3):** Polygon edges become less defined (rendering as dashed instead of solid). The Golem is less certain about what is interesting.

### Phase degradation

| Phase | Effect |
|---|---|
| Thriving | Full rendering, sweep arc, all labels |
| Stable | Standard behavior |
| Conservation | Sweep arc disabled. Labels dim |
| Declining | Polygon outline only, no fill |
| Terminal | Widget disabled |

### Data structure

```rust
struct ChainScopeRadar {
    categories: Vec<InterestCategory>,
    sweep_enabled: bool,
    sweep_angle: f64,
}

struct InterestCategory {
    label: String,                  // "DEX", "LENDING", "VAULT", "BRIDGE", "STAKING", etc.
    interest_score: f64,            // 0.0-1.0
    display_score: f64,             // lerped
    last_spike_tick: Option<u64>,
    protocol_count: usize,          // how many protocols in this category
}
```

**The feel:** A scanning eye on the blockchain. The radar sweeps, the polygon breathes, and when a category lights up you know the triage pipeline found something there. A glanceable summary of where on-chain activity is concentrated.

---

## Open specifications

Several widgets have gaps in the source material. Recording them here rather than inventing specs:

- **FlashNumber/FlashWidget:** No specification for concurrent flashes (what happens when a value changes again before the previous flash completes). Recommend: cancel the previous flash and start a new one.
- **DecisionRing:** No specification for the exact character set used to render the circle, or how it scales with different terminal sizes. The ring is described conceptually but not with an ASCII mockup.
- **BrailleDensityMap:** The overflow behavior (dots spilling past the boundary) has no specified cap. How far can the overflow extend? Recommend: cap at 150% of the original boundary radius.
- **MAGIPanel:** The Eros/Thanatos split trigger condition is described qualitatively ("when the vote splits on a survival vs. exploration axis") but has no formal definition. The golem runtime needs to classify split votes along this axis.
- **CounterfactualBranch:** No specification for how the dream system generates counterfactual outcomes. The visualization assumes the data exists but does not specify the computation.
- **ProgressArc:** No specification for the exact box-drawing characters used to render the circular arc, or how the arc maps to character positions at different radii.

---

*Every widget is a transducer: event fabric in, visual motion out.*

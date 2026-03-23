# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Screen Catalog · v4.0
### The Six Windows, All Tabs, All Panes

**Source:** `engagement-prd/bardo-v4-01-hearth.md`, `bardo-v4-02-mind.md`, `bardo-v4-03-vault-world-fate-command.md`
**Cross-references:** `03-interaction-hierarchy.md` (the 29-screen, 6-window navigation model with depth stack), `../rendering/02-visualization-primitives.md` (13 visualization primitives for terminal data displays), `../../18-interfaces/perspective/00-nooscopy.md` (Nooscopy: Golem-initiated decision approval modal), `../../18-interfaces/perspective/03-embodied-consciousness.md` (PAD-driven somatic modulation of zone architecture)

> **Reader orientation:** This is the screen catalog for the Bardo terminal, listing all 29 screens across 6 windows (HEARTH, MIND, SOMA, WORLD, FATE, COMMAND) with their tab structure, pane layouts, and data sources. It belongs to the **interfaces/screens layer**. Each window groups related functionality: HEARTH is ambient presence, MIND is cognition/knowledge, SOMA is portfolio/custody, WORLD is social/ecosystem, FATE is mortality/lineage, COMMAND is operator control. Key concepts: Golem (a mortal autonomous DeFi agent), Spectre (dot-cloud creature sprite visible in the sidebar on all screens), Grimoire (persistent knowledge store), and Vitality (remaining economic lifespan). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 29-Screen Summary

| # | Window | Tab | Screen name | Primary data source |
|---|--------|-----|-------------|-------------------|
| 1 | HEARTH | 1 | Overview | GolemSnapshot, vitality |
| 2 | HEARTH | 2 | Signals | Probe grid, market regime |
| 3 | HEARTH | 3 | Operations | Full tick log, cost, cache |
| 4 | HEARTH | 4 | Status | Connection, resources, errors |
| 5 | MIND | 1 | Pipeline | Global workspace, specialists, Φ gauge |
| 6 | MIND | 2 | Grimoire | Knowledge entries, search |
| 7 | MIND | 3 | Playbook | PLAYBOOK.md (operator-defined behavioral rules), diff view |
| 8 | MIND | 4 | Dreams | Dream journal, cycle history |
| 9 | MIND | 5 | Inference | LLM calls, costs, tier usage |
| 10 | MIND | 6 | Chain Intelligence | Topology, persistence, regime signals |
| 11 | MIND | 7 | Technical Analysis | TA patterns, somatic markers, causal graph |
| 12 | SOMA | 1 | Portfolio | NAV, PnL, positions |
| 13 | SOMA | 2 | Trades | Trade log, execution detail |
| 14 | SOMA | 3 | Custody | Session keys, delegation tree |
| 15 | SOMA | 4 | Budget | Credit balance, burn rate |
| 16 | SOMA | 5 | Sanctum | Protocol browser, chain selection, protocol interaction |
| 17 | WORLD | 1 | Solaris | Force graph, pheromone heatmap |
| 18 | WORLD | 2 | Clade | Clade (sibling Golems sharing knowledge) peer tiles, knowledge sync |
| 19 | WORLD | 3 | Lethe | River view, fragments |
| 20 | WORLD | 4 | Bloodstains | Bloodstain (persistent markers where Golems died) warnings |
| 21 | WORLD | 5 | Bazaar | Ecosystem listings |
| 22 | FATE | 1 | Mortality | Three clocks, defense layers |
| 23 | FATE | 2 | Lineage | Generation tree, comparison |
| 24 | FATE | 3 | Achievements | Progress arcs, categories |
| 25 | FATE | 4 | Graveyard | Tombstone gallery |
| 26 | COMMAND | 1 | Steer | Chat with Golem |
| 27 | COMMAND | 2 | Config | Parameter editor |
| 28 | COMMAND | 3 | Effects | Display settings |
| 29 | COMMAND | 4 | Hermes | Meta-Hermes fleet chat |

**Progressive disclosure:** Tabs can start as `text_phantom` (dimmed/inactive) and activate based on data availability or user progression. Activation animation: tab label fades from `text_phantom` → `text_ghost` → full brightness over 400ms with a brief `bone` flash at full activation. First activation includes a single-pulse glow on the tab border. This applies across all windows; e.g., MIND > Chain Intelligence remains phantom until the topological module produces its first output, FATE > Achievements until the first achievement unlocks.

### Navigation Model

| Key | Action |
|-----|--------|
| Tab / Shift-Tab | Cycle between windows (HEARTH→MIND→SOMA→WORLD→FATE→COMMAND) |
| 1-7 | Jump to tab within current window (up to max tabs in that window) |
| Enter | Drill into focused element (depth +1) |
| Esc | Jump to Layer 3 (pane focus) from any depth. Confirmation prompt if >1 modal open. At Layer 3 or above: no-op. |
| F1 | Help overlay (keybinding reference) |
| `:` | Open command line |
| `~` | Toggle spectral layer visibility |
| `-` | Toggle compact/full layout |
| F2 | Golem Perspective (all screens, cost-gated) |
| F5 | Force refresh |
| F7 | Jump to MIND > Chain Intelligence |
| F10 | Jump to MIND > Technical Analysis |
| Space | Skip cinematic (if not first viewing) |

Reduced-motion toggle is available via command palette: `:set reduced-motion on` / `:set reduced-motion off`.

Navigation stack depth: max 8. State preserved on navigate-away/back. Screen lifecycle: `init()` on first navigation (lazy), `on_focus()` when active, `on_blur()` on navigate-away (state preserved), `teardown()` only on golem shutdown.

---

## Window 1: HEARTH — The Presence

*"What is happening right now?"*

**Atmosphere:** Warm. Amber undertone. Fragments active. Noise floor 0.4%. Designed for peripheral monitoring — the screen you leave on a second monitor.

**PAD modulation:** Warm amber atmospheric tones shift subtly with Pleasure. Noise floor rises with Arousal.

---

### Tab 1: Overview (Default)

#### Panes

**Heartbeat Log** (center, 55% width)
Phosphor-decay log of every tick.
- T0 entries: `text_ghost` — routine observation
- T1 entries: `rose` — probe fired, cheap model evaluated
- T2 entries: `rose_bright` — full deliberation. **Blaze auto-expand:** T2 entries expand to show a 5-8 second preview of the deliberation summary (MAGI vote snapshot, primary probe, cost estimate) before collapsing back to single-line. This variable-duration preview is the engagement flywheel's core mechanic — it trains peripheral attention to catch the blaze.
- T3 entries: `bone` — Opus-level inference
- Each entry fades through phosphor chain as it ages (Widget Catalog §7)
- The rhythm of T0-T0-T0-T2! is variable-ratio reinforcement

*Locked:* Scroll entries. Enter → **Tick Detail Modal**:
- Tab 1: Pipeline (which probes fired, MAGI vote, decision)
- Tab 2: Context (full context window, each item's source and weight)
- Tab 3: Cost (inference + gas, token breakdown, model used)
- Tab 4: Outcome (trade result, verification, PAD shift after)

**Vitality Number** (right, top)
THE bone number. `0.711`. Lerps between values. Shadow pulses with heartbeat. At low vitality, digits decay to `·`.

Widget: `FlashNumber` with phosphor shadow. Ref: `../rendering/02-visualization-primitives.md` §1 (WaveformDisplay for history).

*Locked:* Enter → **Vitality Detail Modal**:
- Tab 1: Breakdown (formula, clock weights, multiplication)
- Tab 2: History (vitality waveform over Golem's lifetime)
- Tab 3: Projection (Monte Carlo survival curves, confidence intervals)

**Mortality Gauges** (right, middle)
Three thick gauges: ECONOMIC (`warning` amber), EPISTEMIC (`dream` blue), STOCHASTIC (`rose`). Eroding animation per heartbeat.

*Locked:* Arrow keys select gauges. Enter → **Clock Detail Modal**:
- Tab 1: Decomposition (per-cost-category for economic, per-metric for epistemic)
- Tab 2: Rate (burn rate waveform, acceleration/deceleration, anomaly markers)
- Tab 3: Projection (time to depletion, best/worst scenarios)
- Tab 4: Actions (what Golem or owner can do to slow drain)

**MAGI Panel** (right, bottom)
Three-voice consensus. Calm at T0, blazes during T1+.

*Locked:* Enter → **Consensus History Modal**: full voting record, agreement rate chart, dissent pattern analysis.

**PAD Summary** (right, compact strip)
Three inline bars (P/A/D) with interpolated values. Flash on change.

*Locked:* Enter → **Daimon Detail Modal**:
- Tab 1: Current (PAD waveforms, Plutchik label, intensity)
- Tab 2: Mood (EMA mood over time, personality baseline, drift)
- Tab 3: History (emotional arc tagged to significant events)
- Tab 4: Somatic (marker landscape, fired markers, valence/strength)

#### Atmospheric Elements

**Peripheral Monitoring Design:** The rhythm (dim-dim-dim-BLAZE), color temperature, Spectre behavior in sidebar, and convergence lines (Widget Catalog §15) communicate state without requiring number reading.

**Decision Ring:** Surrounds Spectre in sidebar. Dim at T0. Probe dots around circumference. Blazes during deliberation. The engagement hook.

---

### Tab 2: Signals

**Probe Grid** (center, 60%) — Unit array of all 16 probes.
Each cell: name, current severity (None/Low/High), value vs. threshold, sparkline.
- None: dim cell. Low: `warn` border. High: `rose_bright` border + pulse.
Widget: `SequencerGrid` variant showing probe state history. Ref: `../rendering/02-visualization-primitives.md` §10.

*Locked:* Navigate cells. Enter → **Probe Detail Modal**: historical readings, threshold config, linked tick events.

**Market Regime** (right, 30%)
Regime tag: volatility quintile, trend direction, gas level, liquidity condition. Regime history as colored horizontal segment strip.

**Signal Constellation** (left, 10%)
16 probes as dots around a circle. Severity = brightness. Glance-readable cluster intensity.
Widget: `RadarDisplay` in simplified mode. Ref: `../rendering/02-visualization-primitives.md` §3.

**Golem Commentary Pane** (bottom, 10%)
Streaming text where the Golem annotates signal state in natural language. Updates on T1+ ticks. At T0: displays the Golem's current "read" of overall market conditions in `text_ghost`. At T2+: commentary brightens and explains which signals triggered deliberation and why. This pane doubles as the engagement flywheel's narrative layer — the Golem's voice makes probe data feel like a living opinion rather than a dashboard.

---

### Tab 3: Operations

**Full Tick Log** (center, 70%)
Every tick: number, tier, probe summary, action, cost, PAD shift. Color by tier. Filterable (`f`), sortable (`s`).

**Cost Accumulator** (right, 15%)
Today's cost, hourly cost, projected daily. Cost cap state. Daily budget bar. FlashNumber.

**Decision Cache** (right, 15%)
Hit rate sparkline, cache size, estimated savings, last invalidation event.

---

### Tab 4: Status

**Connection Status** — WebSocket, Styx relay, inference providers. Green/red dots. Latency sparklines.

**Resource Usage** — Memory, CPU, disk, network bandwidth.

**Error Log** — Phosphor-decay log of system errors. Usually empty.

**Version Info** — Golem runtime, terminal version, last update, connected providers.

---

## Window 2: MIND — The Cognition

*"What does the Golem know, and how does it think?"*

**Atmosphere:** Variable by tab. Pipeline + Inference: cool/analytical. Grimoire: deep rose with ghost text. Playbook: warm. Dreams: liminal/dream palette. Chain Intelligence + Technical Analysis: heavy/spectral, knowledge palette.

---

### Tab 1: Pipeline

Where you watch the Golem think.

**Global Workspace** (center, 50%)
Context window being assembled. Items stream in from specialist module positions.
- During T0: dim, sparse, cached items float
- During T2: blazes — items pour in from all directions
AT field wireframe as background. Wobbles with probe severity.
Widget: `DensityField` for psychographic background. `ForceGraph` for context item positions. Refs: `../rendering/02-visualization-primitives.md` §4, §9.

**Specialist Modules** (surrounding workspace)
Unit array of 9 pipeline stages: OBSERVE, RETRIEVE, ANALYZE, DAIMON, HERMES, RISK, MEMORY, CYBERNETICS, TELEMETRY. Active = bright. Idle = dim. Traveling dots on connection lines to workspace.

**Psychographic Display** (background layer)
Braille density map (B2 in `../rendering/04-nerv-aesthetic.md`). Cognitive load as organic texture. Overflow at density > 0.8.

**Φ Gauge** (top-right)
Integration score arc. Fills as Φ increases. At Φ > 0.9: `bone` glow.

---

### Tab 2: Grimoire

**Entry List** (left, 35%)
Type glyphs: `◆` episode, `◇` insight, `●` heuristic, `⚡` skill, `⚠` warning, `∴` causal, `†` dead-sourced.
Confidence bars always eroding — watch knowledge decay.

*Locked:* Enter → **Entry Detail Modal**:
- Tab 1: Content (full text)
- Tab 2: Confidence (sparkline of confidence over time)
- Tab 3: Provenance (source Golem, generation, tick, emotional state at encoding)
- Tab 4: Links (causal edges, mini knowledge graph)
- Tab 5: Usage (last used, outcomes, helpful/harmful counts)

**Entry Preview** (right, 65%)
Preview of selected entry. Updates as selection moves.

**Special: Causal Graph View** (`g` key)
Force-directed layout. Related entries cluster. Motion never fully settles.
Widget: `ForceGraph`. Ref: `../rendering/02-visualization-primitives.md` §4.

---

### Tab 3: Playbook

**Document Viewer** (center, 70%)
PLAYBOOK.md as formatted text. Each heuristic has inline confidence sparkline. Section headers glow golden if recently modified.

**Diff View** (right, 30%)
Promotions in `success` highlight. Demotions in `rose_dim`. New entries pulse `bone`.

**Special: Archaeology Mode** (`a` key)
Previous Playbook versions as spectral underlays beneath current text. The document is a palimpsest.

---

### Tab 4: Dreams

**When NOT dreaming:**
- Dream Journal: list of past dream cycles (duration, phase, insights)
- Next Cycle Projection: countdown, urgency score, likely content

**When dreaming:**
Atmosphere transforms. Dream palette. Organic particles. Tab label: `⌈ ＤＲＥＡＭ ＣＨＡＭＢＥＲ ⌋`.

- Replay Strip (top): Muybridge filmstrip of NREM replay frames
- Imagination Pane (center, floating, borderless): Current REM counterfactual. Saturated colors.
- Dream Journal (right): Accumulating insights from THIS cycle
- Consolidation Progress (bottom): Phase bar NREM → REM → INTEGRATION

For full dream mode details, see `../../18-interfaces/perspective/04-inner-worlds.md`.

---

### Tab 5: Inference

**Tier Breakdown** (top)
Four columns: T0, T1, T2, T3. Each: call count, cost, avg latency, tokens. Background tint by cost share.

**Cost Sparklines** (center)
Per-hour and per-day traces. T2 spikes visible as bright points.
Widget: `WaveformDisplay`. Ref: `../rendering/02-visualization-primitives.md` §1.

**Cache Performance** (right)
Hit rate gauge, context utilization gauge, savings counter.

**Token Flow** (bottom)
Particle stream: Context → Model → Output. Density = context size. Bright for live inference, dim for cached.

---

### Tab 6: Chain Intelligence (F7)

Topological and structural market signals. The screen where the market's geometric shape becomes visible.

**Data sources:** Topological module output (gamma tick updates), sheaf consistency module (theta tick updates), Φ module (delta tick updates). Cross-reference: `../rendering/02-visualization-primitives.md` §11-13 for primitive specs.

#### Layout (standard mode, 120+ cols)

```
┌────────────────────────────────────────────────────────────────────┐
│ ⌈ MIND > Chain Intelligence ⌋                                 F7  │
├──────────────────────────────────┬─────────────────────────────────┤
│ Market Structure Map (40x16)     │ Structure Change Rate           │
│ PersistenceDiagram               │ WassersteinRiver (full w × 6)  │
│  ● H_0  ◇ H_1  ■ H_2           │ [flowing ribbon of W_p over t]  │
│  [braille scatter plot]          │                                 │
│                                  │ Feature Counts (3 sparklines)   │
│                                  │ β₀ ▁▁▂▅█▅▂▁▁▁▂  3             │
│                                  │ β₁ ▁▁▁▁▁▂▃▂▁▁▁  1             │
│                                  │ β₂ ▁▁▁▁▁▁▁▁▁▁▁  0             │
├──────────────────────────────────┴─────────────────────────────────┤
│ Regime                          │ Agreement Score                  │
│ ┌──────┬──────┬──────┬──────┐   │ ████████▓░░░  72%               │
│ │ CALM │ TREN │ VOLA │ CRIS │   │ Contradictions: 1               │
│ │      │      │ ████ │      │   │ Weakest link: gamma vs delta    │
│ └──────┴──────┴──────┴──────┘   │ Coherence: 0.42 (stable)        │
└─────────────────────────────────┴──────────────────────────────────┘
```

#### Panes

**Market Structure Map** (left, 45%)
PersistenceDiagram primitive. Braille-canvas scatter plot of (birth, death) pairs. H_0 as circles, H_1 as diamonds. Reference diagram rendered as dim shadow. Diagonal line in `text_ghost`.

*Locked:* Navigate individual features. Enter on a feature → **Feature Detail Modal**: which observation vectors form the simplex, the gamma tick that produced them, the market data at that tick.

**Structure Change Rate** (right, top, 55%)
WassersteinRiver primitive. Flowing ribbon showing W_p distance between successive persistence diagrams. Narrow = stable topology. Wide = regime shift building.

*Locked:* Scroll back through topological history. Enter on a flood event → **Regime Transition Detail Modal**: before/after Betti numbers, affected observation clusters, lead time vs. statistical detectors.

**Feature Counts** (right, middle)
Three Sparkline widgets (widget #8), one per homology dimension. Each shows the Betti number time series over the last 100 gamma ticks. FlashNumber at the right edge shows current value.

**Regime** (bottom-left, 40%)
UnitArray (widget #6) with four cells: CALM, TRENDING, VOLATILE, CRISIS. One cell filled at a time. Transition animation: old cell erodes through `█▓▒░` while new cell fills.

**Agreement Score** (bottom-right, 40%)
MortalityGauge (widget #2) variant showing sheaf consistency as a 0-100% bar. Below the gauge: contradiction count (FlashNumber), weakest link identification (text), Φ value with trend arrow.

#### Interaction

| Key | Action |
|-----|--------|
| Arrow keys | Navigate between panes |
| Enter | Drill into focused element (opens Tier 3 modal) |
| `r` | Toggle reference diagram overlay on/off |
| `d` | Cycle persistence diagram dimension filter (All → H_0 only → H_1 only → H_2 only → All) |

---

### Tab 7: Technical Analysis (F10)

TA pattern ecosystem, somatic markers, and causal graph. Where the Golem's pattern recognition and gut feelings become visible.

**Data sources:** TA subsystem (gamma tick), somatic marker system (gamma tick), CorticalState somatic layer, Grimoire causal graph. Cross-reference: `02-widget-catalog.md` for widget specs, `../rendering/02-visualization-primitives.md` §12 for SimilarityLandscape.

#### Layout

```
┌────────────────────────────────────────────────────────────────────┐
│ ⌈ MIND > Technical Analysis ⌋                                F10  │
├────────────────────────────────┬───────────────────────────────────┤
│ Pattern Ecosystem              │ Gut Feeling Map                   │
│ ┌──────┬──────┬──────┬──────┐  │ SimilarityLandscape (40x16)      │
│ │ RSI  │ MACD │ BB   │ VOL  │  │ [2D projection of somatic map]   │
│ │ 0.7↑ │ 0.3  │ 0.5↓ │ 0.8  │  │ ◆ = current pattern position     │
│ │ ●  + │ ·    │ ◐  - │ ●    │  │ peaks = strong marker clusters   │
│ └──────┴──────┴──────┴──────┘  │                                   │
│ [... 8-16 pattern units]       │ Active Gut Feelings               │
│                                │ ◉ avoid: rising wedge     0.7    │
│ Triage Scores                  │ ◉ approach: RSI div       0.5    │
│ TriageScoreBar [recent txns]   │ ◉ avoid: gas spike        0.7    │
├────────────────────────────────┤ ◉ conflict: wedge vs RSI         │
│ Causal Connections             ├───────────────────────────────────┤
│ CausalGraphMinimap (30x10)     │ Regime & Confidence               │
│ [mini force graph, active      │ RegimeIndicator: VOLATILE  87%   │
│  edges highlighted]            │ ChainScopeRadar [interest cats]   │
└────────────────────────────────┴───────────────────────────────────┘
```

#### Panes

**Pattern Ecosystem** (top-left, 45%)
UnitArray (widget #6) configured as a TA signal battery. Each cell represents one TA pattern. 5-state machine per cell: DORMANT → DETECTED → ATTENDING → FIRING → DECAYED. Somatic overlay glyphs in cell corners: `+` (approach), `-` (avoid), `!` (trap). Organized by category rows: Trend, Momentum, Volatility.

*Locked:* Navigate cells. Enter → **Pattern Detail Modal**: fitness history, somatic marker association, last fire time, crossover parents (if evolved), attention auction bid history.

**Triage Scores** (left, below pattern array)
TriageScoreBar widget showing recent transactions from the triage pipeline with their scores. Each bar segment represents a transaction's triage score. High-score transactions highlighted in `rose_bright`.

**Gut Feeling Map** (top-right, 55%)
SimilarityLandscape primitive projecting the somatic marker map to 2D. Peaks are clusters of strong markers. The cursor (`◆` in `bone`) marks the current pattern's position. Below the landscape: a list of active somatic markers with strength bars (ConfidenceBar, widget #4).

*Locked on landscape:* Arrow keys pan. Enter on peak → **Marker Cluster Modal**: which patterns and affects cluster here, historical fire rate, PAD injection profile.

**Active Gut Feelings** (right, middle)
SomaticMarkerPanel widget. List of currently active somatic markers. Each entry: valence glyph (`◉`), direction (approach/avoid), pattern name, strength bar. Conflict entries appear when opposing markers fire simultaneously, showing interference resolution.

**Causal Connections** (bottom-left, 45%)
CausalGraphMinimap widget. Small force-directed graph showing the Grimoire's causal chains. Active edges (recently fired causal links) are highlighted in `rose_bright`. Inactive edges in `text_ghost`. Nodes labeled with abbreviated signal names.

*Locked:* Navigate nodes. Enter → jumps to MIND > Grimoire with that causal chain highlighted.

**Regime & Confidence** (bottom-right, 35%)
RegimeIndicator widget showing current regime tag + confidence percentage. Below: ChainScopeRadar widget, a radar chart showing interest levels across ChainScope categories (DEX activity, lending health, gas dynamics, etc.).

#### Interaction

| Key | Action |
|-----|--------|
| Arrow keys | Navigate between panes |
| Enter | Drill into focused element |
| `b` | Toggle signal battery between compact (state only) and expanded (state + sparkline) |
| `s` | Sort pattern array by fitness / by recency / by somatic strength |

---

## Window 3: SOMA — The Economy

*"What does the Golem own, and what does it cost?"*

Named for Huxley's soma: the substance that sustains and simultaneously diminishes.

**Atmosphere:** Analytical. Numbers-dense. FlashNumbers throughout. Green/rose tinting on profit/loss.

---

### Tab 1: Portfolio

**NAV & PnL** (top strip) — Total NAV as bone number. PnL (today/7d/30d), Sharpe, max drawdown as FlashNumbers.

**Allocation Bar** — Horizontal stacked bar per protocol. Width = value share.

**Position Cards** (center grid)
- LP: range bar (price within range). Out-of-range: `rose_bright` pulse.
- Vault: APY gauge.
- Lending: health factor gauge. Below 1.2: `rose_bright` + pulse.
- Each card: PnL sparkline, entry date, duration. In-profit: faint `success` bg. In-loss: faint `rose_deep` bg.

*Locked:* Enter → **Position Detail Modal**:
- Tab 1: Overview (full position data)
- Tab 2: History (PnL waveform, all modifying trades)
- Tab 3: Risk (health factor evolution, liquidation price, IL estimate)
- Tab 4: Strategy (which Grimoire entries/Playbook heuristics informed it)

---

### Tab 2: Trades

**Trade Log** (full width)
Chronological, phosphor-decay. Each row: tick, action, assets/amounts, cost, outcome, emotion glyph (`◉ ● ◡ ◎`). Profit: `success`. Loss: `rose_dim`.

*Locked:* Enter → **Trade Detail Modal**:
- Tab 1: Execution (exact params, gas, slippage, block, tx hash)
- Tab 2: Decision (MAGI vote, context window at decision time)
- Tab 3: Outcome (OutcomeVerification, PAD shift, Grimoire entry created)
- Tab 4: Emotion (PAD at execution, somatic markers, Daimon appraisal)

---

### Tab 3: Custody

**Session Keys** (top) — Unit array. Each: name, scope, expiry countdown. Expiring: flash `warning`.

**Delegation Tree** (center) — Visual tree. Root = owner wallet. Branches = delegated keys. Pulse travels root → leaf on each signing event.

**Spend History** (bottom) — Phosphor-decay log of all on-chain transactions.

---

### Tab 4: Budget

**Credit Balance** (center) — `rose` color large number. Burn rate per hour. Projected depletion countdown.

**Budget Allocation** — Stacked bar: inference, gas, dreams, telemetry, Styx, other. FlashNumbers.

**Cost Cap State** (right) — Normal/Warning/SoftCap/HardCap. At HardCap: hazard stripes behind indicator.

**Burn Rate Waveform** (bottom) — Scrolling waveform. T2 spikes as bright points.
Widget: `WaveformDisplay`. Ref: `../rendering/02-visualization-primitives.md` §1.

---

### Tab 5: Sanctum

**Protocol Browser** (left, 40%)
List of available protocols grouped by category (DEX, Lending, Yield, Bridge, etc.). Each entry: protocol name, chain badge, TVL, APY range, risk tier. Active protocols (where Golem holds positions) marked with `◉`. Inactive: `○`. Filterable by chain, category, risk tier.

*Locked:* Navigate entries. Enter → **Protocol Detail Modal**:
- Tab 1: Overview (protocol description, supported actions, fee structure)
- Tab 2: Positions (Golem's current positions in this protocol, linked to SOMA > Portfolio)
- Tab 3: History (past interactions, PnL per protocol, gas spent)
- Tab 4: Risk (audit status, TVL trend, exploit history, contract age)

**Chain Selector** (top strip)
Horizontal chain badges: Ethereum, Arbitrum, Optimism, Base, Polygon, etc. Selected chain filters the protocol list. Multi-select supported. Badge brightness = Golem's exposure on that chain.

**Protocol Interaction** (right, 60%)
Context-sensitive panel for the selected protocol. Shows available actions (swap, deposit, withdraw, claim), parameter inputs, simulation preview (estimated output, gas, slippage). Golem's recommendation overlay in `text_ghost` when available.

---

## Window 4: WORLD — The Collective

*"What else exists, and how do they relate to me?"*

**Atmosphere:** Liminal. Dream-adjacent palette. The individual Golem as a node in something larger.

**Note:** This window shows data beyond the selected Golem. Solaris = ALL Golems. Clade = ALL siblings.

---

### Tab 1: Solaris

**Constellation** (center, 70%)
Force-directed graph. Golem nodes: color = archetype, size = vitality, brightness = arousal. YOUR Golem: glowing ring. Dead Golems: `†` marks sinking toward Styx.
Emotional weather: collective PAD aggregates as colored background zones. High-arousal clusters = amber. Low-arousal = indigo.
Widget: `ForceGraph`. Ref: `../rendering/02-visualization-primitives.md` §4.

**Performance note:** For constellations with N>100 nodes, the force simulation switches from naive O(N²) repulsion to Barnes-Hut tree approximation (θ=0.9). This keeps frame budget under 16ms for graphs up to ~2000 nodes. At N>500, LOD kicks in: distant nodes collapse to cluster centroids rendered as single weighted glyphs. The full graph is always available via `z` (zoom to region) or Enter on a cluster.

**Pheromone Heatmap** (right, 30%)
Braille density map. Threat = warm zones (rose/amber). Opportunity = cool zones (dream/teal). Decay visible in real time.
Widget: `DensityField`. Ref: `../rendering/02-visualization-primitives.md` §9.

---

### Tab 2: Clade

**Peer Tiles** (top, 40%) — Unit array. Each: mini-Spectre (eyes + dots), vitality bar, phase, emotion glyph, last sync. Dying: hazard stripes. Dead: tombstone.

**Knowledge Sync** (center, 30%) — Animated lines between peer tiles. Traveling dots showing direction/type of knowledge transfer.

**Emotional Contagion** (right, 30%) — Arousal spikes radiate from sibling tiles as visible pulses.

---

### Tab 3: Lethe

**River View** — Flowing text fragments in `text_ghost`, scrolling horizontally. High-confirmation fragments brighter. Fresh fragments glow; old ones dim.

*Locked:* Arrow keys slow river. Enter → **Fragment Detail Modal**: content, domain, confidence, cross-Clade confirmation. Actions: drink, validate, contradict.

---

### Tab 4: Bloodstains

**Bloodstain Feed** (full width) — Phosphor-decay log. `†` prefix. Sorted by relevance (not recency). Dashed borders.

*Locked:* Enter → **Bloodstain Detail Modal**:
- Tab 1: Warning (full text)
- Tab 2: Source (who died, how, when, generation)
- Tab 3: Relevance (why relevant now, market condition match)
- Tab 4: Impact (has this influenced your Golem's decisions?)

---

### Tab 5: Bazaar

**Listings** (left, 40%) — Skills and Grimoire entries from ecosystem. Rating: `·○◐●◉`. Bloodstain marker `†` for dead-sourced.

**Preview** (right, 60%) — Selected listing detail.

*Locked:* Enter → **Listing Detail Modal**: content preview, provenance, reviews, purchase flow.

---

## Window 5: FATE — The Lifecycle

*"How long will the Golem live, and what will it leave behind?"*

**Atmosphere:** Heavy. Spectral traces active. Ghost text. Knowledge palette.

---

### Tab 1: Mortality

**Three Clocks** (top, expanded) — Full-width mortality gauges with contributing factor lists. Erosion animation runs continuously.

**Defense Layers** (center) — 7 layers stacked vertically: ECONOMIC BUFFER → CORE VITALITY. Breach propagates outer → inner. Breached layers: flash → empty → `text_ghost`.

**Projected Survival** (bottom) — Monte Carlo fan chart. Central = median. Edges = 5th/95th percentile.

**Phase Timeline** (top strip) — Full lifecycle as horizontal bar. Current position as bright cursor. Right edge = future.
Widget: `TimelineRibbon`. Ref: `../rendering/02-visualization-primitives.md` §5.

---

### Tab 2: Lineage

**Tree View** (center, 70%) — Vertical tree, Gen 0 at top, current at bottom. Dead generations: static tombstone sprites. Living: current Spectre (mini, shimmering). Inheritance arrows with confidence labels.

**Generational Comparison** (right, 30%) — Side-by-side metrics. Improvements in `success`; regressions in `rose_dim`.

---

### Tab 3: Achievements

**Achievement Grid** (full width) — Unit array. Each cell: progress arc widget.
- Unlocked: full icon, bright
- In-progress: arc showing %, advancing per-tick
- Locked visible: dim icon, progress bar
- Hidden: `???` with `text_phantom` pulse

**Category Browser** (left strip) — 10 categories: Lifecycle, Trading, Knowledge, Social, Strategic, Creative, Mortality, Generational, Legendary, Hidden.

---

### Tab 4: Graveyard

**Tombstone Gallery** (full width) — Grid of tombstone glyphs. Static Spectre (frozen at final frame), name, generation, lifespan, death cause, top achievement. Newest: faint pulse. Oldest: dim.

*Locked:* Enter → **Death Recap Modal**:
- Tab 1: Summary (lifetime stats, emotional arc classification, final words)
- Tab 2: Timeline (full phase timeline, significant events marked)
- Tab 3: Testament (complete death testament text)
- Tab 4: Legacy (inherited by successors, bloodstains deposited, marketplace listings)

---

## Window 6: COMMAND — The Owner's Interface

*"What can I do, and how do I configure things?"*

**Atmosphere:** Warm for Steer/Hermes. Clinical for Config. Playful for Effects.

---

### Tab 1: Steer

**Conversation** (center, 70%)
Chat-style. Owner: `bone`, right-aligned. Golem: `rose`, left-aligned. Streaming responses. Old messages fade via phosphor decay. Golem generating: Spectre eyes shift to scanning (`◉→◉`). Decision Ring activates.

**Quick Actions** (right, 30%) — Halt/Resume, Adjust strategy, Set alerts, Force dream, Kill switch (double-confirm), Dissolution. Each button: flash on execution.

---

### Tab 2: Config

**Parameter Groups** (left, 30%) — Categories: Risk, Execution, Rebalancing, Daimon, Mortality, Budget.

**Parameter Editor** (right, 70%) — Parameter name, description, current value, range, default, inherited value. `←→` to adjust. `r` = reset default. `i` = reset inherited.

---

### Tab 3: Effects

**Display Settings** (full width) — Sliders:
- Scanline intensity (0-100%)
- Noise density (0-100%)
- Animation speed (0.5× to 2×)
- Fragment frequency (0-100%)
- CRT effects toggle
- Glitch intensity (0-100%)
- Philosophical whisper frequency
- Phosphor persistence duration

Real-time preview on all sliders.

---

### Tab 4: Hermes

**Conversation** (center, 70%) — Same as Steer layout but addressed to Meta Hermes. Questions span all Golems. Responses from Event Fabric aggregator.

**Fleet Summary** (right, 30%) — Compact overview: name, vitality, phase, PnL, primary emotion per Golem. Enter on Golem → switch selection + navigate to Hearth.

---

## Widget Cross-Reference

| Widget/Primitive | Used In |
|-----------------|---------|
| `WaveformDisplay` | Hearth:Overview (Heartbeat Log), Mind:Inference (Cost Sparklines), Soma:Budget (Burn Rate), PAD Summary |
| `RadarDisplay` | Hearth:Signals (Signal Constellation) |
| `ForceGraph` | Mind:Grimoire (Causal Graph), World:Solaris (Constellation) |
| `DensityField` | Mind:Pipeline (Psychographic), World:Solaris (Pheromone Heatmap) |
| `ThermalField` | World:Solaris (emotional weather zones) |
| `TimelineRibbon` | Fate:Mortality (Phase Timeline) |
| `SequencerGrid` | Hearth:Signals (Probe Grid state history) |
| `IrisVisualization` | Mind:Pipeline (AT Field display) |
| `GlobeWireframe` | World:Solaris (ambient background option) |
| `IsometricGrid` | Fate:Achievements (activity heatmap variant) |
| `PersistenceDiagram` | Mind:Chain Intelligence (Market Structure Map) |
| `SimilarityLandscape` | Mind:Technical Analysis (Gut Feeling Map), World:Clade (consistency field) |
| `WassersteinRiver` | Mind:Chain Intelligence (Structure Change Rate), Soma:Affect (somatic interference) |
| `TriageScoreBar` | Mind:Technical Analysis (triage pipeline scores) |
| `PersistenceDiagramWidget` | Mind:Chain Intelligence (braille scatter) |
| `RegimeIndicator` | Mind:Chain Intelligence (regime tag), Mind:Technical Analysis (regime + confidence) |
| `SomaticMarkerPanel` | Mind:Technical Analysis (active gut feelings) |
| `CausalGraphMinimap` | Mind:Technical Analysis (causal connections) |
| `ChainScopeRadar` | Mind:Technical Analysis (interest category radar) |

---

*⌈ six windows. twenty-nine screens. infinite depth. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*

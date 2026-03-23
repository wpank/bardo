# Oracle Surfaces: Progressive Complexity for Prediction and Evaluation [SPEC]

**Progressive Disclosure of Oracle Data, FATE Screen Progression, Complexity Gating, and Widget Catalog**

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: [02-widget-catalog.md](02-widget-catalog.md), [00-screen-catalog.md](00-screen-catalog.md), [../rendering/00-design-system.md](../rendering/00-design-system.md), [14-creature-system.md](../../13-runtime/14-creature-system.md)
>
> Cross-references: [../../01-golem/17-prediction-engine.md](../../01-golem/17-prediction-engine.md) (the Oracle: prediction generation, residual correction, attention foraging, action gating), [../../16-testing/09-evaluation-map.md](../../16-testing/09-evaluation-map.md) (the 14-loop evaluation hierarchy for measuring Golem performance), [../../01-golem/18-cortical-state.md](../../01-golem/18-cortical-state.md) (CorticalState: the 32-signal atomic perception surface, and TUI variable mapping)
>
> Source: `mmo2/22-oracle-surfaces`

> **Reader orientation:** This document specifies how prediction and evaluation data from the Golem's Oracle (prediction engine) are progressively disclosed across the terminal's screens. It belongs to the **interfaces/screens layer** and defines four complexity levels: ambient signals visible on every screen, overview summaries, detail drills, and deep mathematical views. Key concepts: Golem (a mortal autonomous DeFi agent), the Spectre (dot-cloud creature whose particle coherence encodes prediction quality), CorticalState (32-signal perception surface), Grimoire (persistent knowledge store where heuristics live), and BehavioralPhase (lifecycle stage gating which FATE tabs are visible). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Design philosophy

The Oracle produces a lot of data. Fourteen evaluation loops across five speed tiers. Per-category accuracy. Calibration curves. Residual distributions. Heuristic audits. Shadow strategies. Meta-learning scores. Position retrospectives. Attention tier allocations. The question is not what to show. It is when to show it, and to whom.

The wrong answer is a dashboard that dumps everything on a single screen. DeFi dashboards do this routinely: fifty numbers, twenty charts, no hierarchy, no progression. The user either processes everything (cognitive overload) or processes nothing (learned helplessness). Shneiderman's information visualization mantra points the other way: overview first, zoom and filter, details on demand [SHNEIDERMAN-1996]. Start with the broadest possible summary. Let the user drill when they choose to.

But the Bardo terminal goes further than Shneiderman's taxonomy. The broadest summary is not a summary at all. It is an ambient signal, visible without reading, without navigating, without even looking directly at the Oracle screen. Ishii and Ullmer's work on tangible bits introduced the concept of ambient media: information displayed at the periphery of human perception, registering without conscious attention [ISHII-ULLMER-1997]. Weiser and Brown extended this into calm technology, systems that inform without demanding focus [WEISER-BROWN-1997]. The Spectre sprite's particle coherence, the heartbeat's second harmonic, the status bar's single accuracy number: these are calm signals. You absorb them the way you absorb weather through a window.

Tufte's data-ink ratio principle constrains the implementation: every pixel of every Oracle surface must carry information [TUFTE-1983]. No decorative borders. No labels that restate what the visual already shows. The ambient signals work because they are dense with meaning, not because they are pretty. A jittering particle aura and a smooth particle aura differ by exactly one bit of information (coherent vs. turbulent), but that bit is delivered continuously, at 60fps, without occupying a single character of screen real estate.

The progressive complexity model has four levels. Each level requires more effort from the viewer and rewards that effort with more detail. No level forces jargon on users who haven't reached it. No level withholds information from users who want it. The design follows From Software's philosophy: complexity through discovery, not tutorials. Andersen et al. showed that tutorials help simple games but hurt complex ones, because players of complex systems learn better by doing [ANDERSEN-2012]. The Oracle surfaces teach by rewarding exploration. You discover the calibration curve by drilling into the accuracy bar. You discover the environmental model by drilling into the residual distribution. Nobody tells you these exist. You find them because you wanted to know more.

The four levels:

| Level | Name | Effort | Where | What you learn |
|-------|------|--------|-------|----------------|
| 0 | Ambient | Zero interaction | Every screen | Is the Golem predicting well or poorly? |
| 1 | Overview | 10-second scan | MIND > Oracle, FATE > Reviews | Category accuracies, gate status, recent events |
| 2 | Detail | Lock into a pane and interact | Modal drills from Level 1 | Per-category history, position retrospectives, heuristic audits |
| 3 | Deep | Modal within modal | Drill from Level 2 | Full calibration curves, residual distributions, environmental models |

The levels are not tabs. They are not modes. They are depth layers in the same interface, accessible through the standard drill/dismiss navigation. Moving between levels feels like focusing a microscope: the subject stays the same; the resolution changes.

### Progressive complexity states

The four levels are always available in principle, but the TUI gates which FATE tabs are visible based on the owner's experience with the prediction system. A new owner seeing all five FATE tabs on day one is overload. The state machine controls tab exposure:

| State | Trigger | Available views | Transition |
|-------|---------|----------------|------------|
| NOVICE | First launch, <20 predictions | FATE Overview only | INTERMEDIATE after 50 resolved predictions |
| INTERMEDIATE | 50+ resolved predictions | Overview + Categories | EXPERT after 200 resolved AND accuracy > 0.55 |
| EXPERT | 200+ resolved, accuracy > 0.55 | All 5 FATE tabs | Permanent (no demotion) |

State persisted in `~/.bardo/golems/<name>/oracle_state.json`.

### Navigation shortcuts

FATE navigation: `1` = Overview, `2` = Categories, `3` = Deep Dive, `4` = Evaluation, `5` = Reviews. From Overview: `Enter` on a category row jumps to that category's Deep Dive. From Deep Dive: `Esc` returns to Categories. `:demote <heuristic_id>` demotes a heuristic from the command line.

### Action paths from FATE screens

Action paths from FATE screens: `:demote <id>` demotes a heuristic to lower confidence. `:promote <id>` increases confidence. `:retire <id>` marks a heuristic as retired (excluded from retrieval). `:investigate <id>` opens a detailed view of the heuristic's prediction history. All commands are accessible from the COMMAND console as well. These commands modify the Grimoire (the Golem's persistent knowledge store) entry for the targeted heuristic and emit a `Promotion` event on the Event Fabric.

---

## Level 0: Ambient (zero interaction)

The owner is not looking at the Oracle screen. They are on the Hearth, or the Portfolio, or not looking at the terminal at all. Level 0 signals are always present, embedded in the persistent chrome and the Spectre sprite. They require no reading, no navigation, no understanding of prediction systems. A child watching the terminal can tell whether the Golem is predicting well or poorly.

### Spectre interference pattern

The Spectre's particle aura is driven by `coherence_score`, a derived float combining prediction accuracy, calibration quality, and compounding momentum. The relationship between these values and the underlying CorticalState signals is specified in [../../01-golem/18-cortical-state.md](../../01-golem/18-cortical-state.md).

```rust
/// Compute visual coherence from prediction performance metrics.
/// This value drives the Spectre's particle aura behavior.
///
/// Returns 0.0-1.0 where 1.0 = perfectly coherent orbits,
/// 0.0 = fully chaotic Brownian motion.
fn coherence_from_prediction_state(
    prediction_accuracy: f32,
    ece: f32,
    compounding_momentum: f32,
) -> f32 {
    // Accuracy dominates: the Golem's primary signal.
    let accuracy_term = prediction_accuracy.clamp(0.0, 1.0);

    // Calibration quality: ECE of 0.0 = perfect, 0.1 = terrible.
    // Scale so 0.1 maps to 0.0 contribution.
    let calibration_term = (1.0 - ece * 10.0).clamp(0.0, 1.0);

    // Compounding: is the prediction-correction-action cycle
    // generating self-reinforcing improvement?
    let momentum_term = compounding_momentum.clamp(0.0, 1.0);

    accuracy_term * 0.5 + calibration_term * 0.3 + momentum_term * 0.2
}
```

Three visual states emerge from this score:

**Coherent** (coherence > 0.7). Particle orbits are smooth, concentric. The Spectre's aura looks ordered. Peripheral particles trace elliptical paths that share a common center. The motion reads as stable, deliberate, settled. This is a Golem whose model of the world works. Its predictions are accurate, its confidence is calibrated, its learning loops are compounding.

**Turbulent** (coherence < 0.4). Particle paths become chaotic. Brownian motion replaces orbital regularity. The aura jitters. Particles appear to collide, scatter, and reform in unstructured clumps. Something is wrong. The Golem is miscalibrated, or inaccurate, or both. The viewer does not need to know which. The visual signal is sufficient: this Golem is not well.

**Learning** (coherence derivative > 0.01 per minute, regardless of absolute value). Individual particles that were drifting erratically snap into smoother orbits, one to three at a time, with brief brightness flashes on each snap. The aura visibly re-coheres. The Golem is getting better. This state is distinct from both Coherent and Turbulent because it communicates *trajectory*, not position. A Golem with coherence 0.35 (turbulent) and a positive derivative shows particles snapping into order against a backdrop of chaos. The owner sees: it's rough, but it's improving.

Each snap plays over 300ms on individual particles, not the whole cloud. The stagger (1-3 particles per second) prevents the transition from reading as a mode switch.

### Breathing modulation

The heartbeat sine wave (interpolating variable #23, free-running, not lerped) is the Golem's involuntary pulse. The Oracle adds a second harmonic to it, modulated by coherence trend:

```rust
/// Compute the breathing modulation factor from coherence trend.
/// Applied as a second harmonic on the heartbeat sine wave.
///
/// `coherence_delta`: rate of change of coherence_score, per minute.
/// Positive = improving. Negative = declining.
fn breathing_modulation(heartbeat_phase: f32, coherence_delta: f32) -> f32 {
    let base = heartbeat_phase.sin();

    // Second harmonic at half amplitude.
    let amplitude = 0.15;
    let phase_offset = if coherence_delta >= 0.0 {
        0.0       // In-phase: breathing feels deeper, steadier
    } else {
        std::f32::consts::PI  // Out-of-phase: breathing feels irregular
    };

    let second = (heartbeat_phase * 2.0 + phase_offset).sin() * amplitude;

    base + second
}
```

When coherence is improving, the second harmonic reinforces the first. Breathing deepens. The rhythm feels settled, strong. When coherence is declining, the second harmonic opposes the first. Breathing becomes irregular, labored. The amplitude is intentionally small (15% of the fundamental). Most viewers will not consciously notice the modulation. But after watching for five minutes, the breathing pattern registers. You "feel" whether the Golem is doing well without reading a number.

This is sub-perceptual design. Continuous low-amplitude rhythmic signals influence affective state even when viewers cannot report noticing them [BORNSTEIN-1989]. The viewer's own breathing rhythm may entrain to the Golem's, creating somatic awareness of prediction health.

### Status bar indicator

The persistent status bar at the bottom of every screen contains a single prediction metric:

```
  HEARTH  MIND  PORTFOLIO  FATE  WORLD  CLADE  ...     Acc: 76%     ♥ 2.4h    $847.32
```

`Acc: 76%` is the aggregate prediction accuracy across all categories. One number. No breakdown. Color-coded:

- `bone` (>70%): healthy.
- `warning` (50-70%): needs attention.
- `rose_bright` (<50%): poor.

Updates on every resolution. Lerps at rate 1.5 (variable #27), smoothing over ~2 seconds. Color transition is discontinuous: snaps on the frame that crosses the threshold. A smooth gradient would obscure the crossing.

### Decision ring glow

The DecisionRing widget wraps the Spectre on the Hearth screen. The Oracle adds two ambient glow events to it:

**Resolution flash.** Correct resolution: `success` flash (200ms), phosphor fade (1s). Incorrect: `rose_dim` (100ms), fainter. Successes are more visible at the ambient level. Failure detail belongs at Level 2.

**Violation sparks.** Gate violation (action taken despite suppression): amber sparks scatter from the ring (3-5 particles, 500ms fade). Rare and deliberately alarming.

### Heartbeat log gutter

The heartbeat log (visible in the persistent chrome on the HEARTH screen and accessible via the log panel on other screens) gains a one-character gutter column for learning indicators:

```
  · 14:32:07  theta tick #4827, T1, fee_rate +0.3%
  ↑ 14:31:42  theta tick #4826, T0, slippage corrected
  · 14:31:18  theta tick #4825, T0, idle
  ↓ 14:30:53  theta tick #4824, T1, direction miss (-2)
  · 14:30:28  theta tick #4823, T0, price within band
```

The gutter characters:

- `·` = prediction resolved, no accuracy change worth noting
- `↑` = accuracy improved this theta tick (any category)
- `↓` = accuracy declined this theta tick (any category)

These are rendered in `text_dim`. They do not demand attention. But a viewer scrolling through the log can see at a glance whether the last hour was mostly `↑` (improving) or `↓` (declining) or `·` (stable). The gutter is a sparkline of learning compressed into a single column.

---

## Level 1: Overview (10-second scan)

The owner navigates to a prediction-related screen. They scan it for ten seconds. These surfaces should answer the question "how is my Golem's prediction system doing?" with a glance.

### MIND > Oracle tab

The primary Oracle screen. Pane layout follows the standard MIND screen structure.

**Prediction accuracy pane (top-left, 50% width).** Five ProbeGauge widgets, one per prediction category, sorted by accuracy (best at top):

```
  fee_rate     [██████████████    ] 82%  ●
  slippage     [████████████      ] 76%  ●
  liquidity    [███████████       ] 71%  ◐
  price        [█████████         ] 63%  ○
  direction    [███████           ] 48%  ○
```

Each bar shows:
- Category name in `text_dim`
- ProbeGauge fill in `rose` (>70%), `warning` (50-70%), or `rose_bright` (<50%)
- Percentage as FlashNumber
- ECE dot: `●` green (ECE < 0.05, well-calibrated), `◐` yellow (ECE 0.05-0.10, acceptable), `○` red (ECE > 0.10, miscalibrated)

Sort updates on every resolution with 300ms lerped position animation. Strongest categories float up, weakest sink.

**Attention forager pane (top-right, 50% width).** Three-tier count display showing the forager's current allocation:

```
  ATTENTION FORAGER
  ─────────────────────────────────────
  ACTIVE     12 / 15    ████████████▒▒▒
  WATCHED    42 / 60    ███████████████████████████████▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
  SCANNED   287 / 500   ████████████████████████████████████████████████···
```

Each tier is a thin MortalityGauge (single-height variant). Fill color:
- ACTIVE: `rose_bright` (these items consume real inference budget)
- WATCHED: `rose` (these items get cheap T0 reads)
- SCANNED: `text_dim` (these items are passively monitored)

Below the gauges, a one-line summary of the most recent tier transition:

```
  ↑ WETH/USDC 0.05% (Base) promoted WATCHED → ACTIVE: prediction error spike
```

Updated on every forager event (~3-5 per hour).

**Action gate pane (middle, full width, compact).** A single row showing per-category gate status:

```
  GATE:  fee_rate ▮▮  slippage ▮▮  liquidity ▯▯  price ▯▯  direction ▯▯
```

- `▮▮` (filled blocks, `success` color) = gate open, category permits action
- `▯▯` (empty blocks, `rose_dim` color) = gate closed, category suppresses action

Hovering (or pressing `?` on a gated category) shows the suppression reason as ghost text:

```
  GATE:  fee_rate ▮▮  slippage ▮▮  liquidity ▯▯  price ▯▯  direction ▯▯
                                   margin: 1.2%   margin: -0.4%
                                   need: 5.0%     need: 5.0%
```

The action gate from the prediction engine, rendered as icons.

**Recent resolutions pane (bottom, full width).** The five most recent prediction outcomes, newest first:

```
  RECENT RESOLUTIONS
  ─────────────────────────────────────────────────────────────
  ↑ 14:32  fee_rate    WETH/USDC (Base)   predicted $2.41/hr  actual $2.38/hr  ✓
  · 14:31  slippage    UNI/ETH (Mainnet)  predicted 12 bps    actual 14 bps    ✓
  ↓ 14:30  direction   WETH/USDC (Base)   predicted up        actual down      ✗
  · 14:29  price       ARB/ETH (Arb)      predicted $1.42     actual $1.39     ✓
  · 14:28  liquidity   WETH/USDC (Base)   predicted $4.2M     actual $3.8M     ✓
```

Same `·↑↓` gutter as the heartbeat log. `✓`/`✗` color-coded (`success`/`rose_bright`). Scrolls with phosphor decay on older entries.

### FATE > Reviews tab

The evaluation screen. This is where the slow mirror's output surfaces.

**Daily review card (top, 40% height).** A bordered card summarizing the last 24 hours:

```
  ┌─ DAILY REVIEW: 2026-03-15 ──────────────────────────────────┐
  │                                                              │
  │  Predictions: 847    Accuracy: 74%    ECE: 0.068             │
  │                                                              │
  │  Best:   fee_rate (82%, ↑4% from yesterday)                  │
  │  Worst:  direction (48%, ↓3%)                                │
  │                                                              │
  │  Gate blocked 31 actions (23 profitable in hindsight,        │
  │  8 would have lost money). Gate saved ~$12.40 net.           │
  │                                                              │
  │  Notable: Corrector converged on slippage bias in bull       │
  │  regime (was +8 bps, now +1 bps after 340 samples).          │
  │                                                              │
  └──────────────────────────────────────────────────────────────┘
```

Structured summary, data-dense. The "Notable" field is generated by Loop 12 during the daily review (T1 inference).

**Weekly review summary (below daily card, 2 lines).** A one-line trend plus a one-line highlight:

```
  Week 3: Acc 74% (↑2%)  Dream yield 34%  Corrector converged: 4 categories
  Position retrospectives: 3 reviewed, avg regret: $27, best exit gap: 2.1 hours
```

Aggregates seven daily reviews. Surfaces weekly-timescale signals only.

**Position retrospective count (compact, below weekly).** One line:

```
  3 positions reviewed this week  |  avg regret: $27  |  optimal exit gap: 2.1h avg
```

"Regret" = dollar gap between actual and optimal outcome (Loop 12). "Optimal exit gap" = average time between actual exit and retrospectively optimal exit.

### MIND > Oracle > Learning Pulse sub-tab

Accessible via `Playbook > B` (keyboard navigation from the Oracle tab). This is the real-time learning dashboard, showing the state of the evaluation loops as they operate.

```
┌─ LOOPS ──────────────────┬─ DRIFT ──────────────────────────────────────┐
│ L1 ● Correcting          │  ⠤⠤⠤⡤⡤⣤⣤⣤⣤⡤⠤⠤⠤⠤⠤⠤⡤⡤⣤⡤⠤                │
│ L4 ● Tracking            │  PE over last 100 theta ticks               │
│ L7 ● Cost-eval           │                                             │
├─ CATEGORIES ─────────────┼─────────────────────────────────────────────│
│ fee_rate   [======  ] ↑  │  RECENT                                     │
│ price      [====    ] →  │  ✓ fee: corrected -0.02 bias                │
│ slippage   [=====·  ] ↓  │  ✗ direction: 3 consecutive miss            │
│ liquidity  [===     ] →  │  ● H-7 promoted KEEP                        │
│ direction  [==      ] ↓  │  ◇ creative #412 confirmed (2/3)            │
├─ ────────────────────────┴─────────────────────────────────────────────│
│  ▁▂▃▄▅▆▇▆▅▄▃▂▁▂▃▄▅▆▅▄▃▂▁▂▃▄▅▆▇▆▅▄  TimelineRibbon (30d PE)          │
└────────────────────────────────────────────────────────────────────────┘
```

Four panes in a 2x2 grid plus a full-width footer:

**LOOPS pane (top-left).** Indicators for the 3 evaluation loops fast enough for real-time display: L1 (Residual correction), L4 (Prediction accuracy), L7 (Cost-effectiveness). `●` (`success`) = active, `○` (`text_dim`) = idle. Higher-tier loops (L8-L14) surface in FATE > Reviews and Level 2 drill-downs.

**DRIFT pane (top-right).** Braille sparkline of aggregate prediction error over 100 theta ticks. Flat near center = well-calibrated. Drift up = over-prediction bias. Drift down = under-prediction. Color: `rose` for recent 20 ticks, fading to `text_dim`. Scale auto-adjusts.

**CATEGORIES pane (bottom-left).** Per-category ProbeGauge bars sorted by change magnitude. Arrows: `↑` (`success`), `↓` (`rose_bright`), `→` (`text_dim`). A `·` in each gauge marks where accuracy sat 24 hours ago.

**RECENT pane (bottom-right).** Scrolling event log with phosphor decay (bright `bone` to dim `text_ghost`). Prefixes: `✓` correction, `✗` miss streak, `●` heuristic promotion, `◇` creative confirmation. Updates asynchronously, 5-10 events/min during active trading.

**TimelineRibbon (footer).** Full-width braille sparkline of 30-day prediction error. Compare against DRIFT: if current drift exceeds anything in the 30-day ribbon, the Golem is in unfamiliar territory. Color: uniform `rose_dim`.

---

## Level 2: Detail (lock and interact)

The owner locks into a pane (press Enter) and explores. Each pane in the Oracle and Reviews screens supports depth drilling via modal overlays. The overlay appears within the locked pane's bounds, not as a full-screen takeover.

### Category detail modal

Triggered by locking the Prediction Accuracy pane and pressing Enter on a specific category.

**30-day accuracy sparkline.** Braille sparkline, full pane width, 4 rows tall. Color: `rose`. Horizontal axis: 30 days. Vertical axis: 0-100% accuracy. The current day's value is marked with a solid block character at the rightmost position. Regime change events (detected by the environmental model) are marked as vertical dotted lines in `text_ghost`.

**Per-regime breakdown.** A compact table showing accuracy broken down by market regime:

```
  REGIME       ACC     SAMPLES   TREND
  bull         81%     342       →
  bear         67%     128       ↓
  chop         74%     89        ↑
  crisis       41%     12        →
```

Regime labels use their atmosphere zone colors (bull = warm, bear = cool, chop = neutral, crisis = `rose_bright`). Sample count determines statistical confidence: regimes with fewer than 30 samples get a `*` suffix indicating insufficient data.

**Calibration curve (compact).** A 20x10 braille rendering of the calibration curve. Ideal diagonal line in `text_ghost`. Actual curve in `rose`. Where the actual curve deviates from ideal by more than 0.10, the gap between them fills with `warning` dots. ECE annotated below the chart.

Above diagonal = underconfident. Below = overconfident. The viewer doesn't need to understand ECE. They need to see whether the curve follows the diagonal.

**ECE trend sparkline.** Single-row braille below the calibration curve, showing ECE over 30 days. Color: `success` (<0.05), `warning` (0.05-0.10), `rose_bright` (>0.10).

### Attention item detail modal

Triggered by locking the Attention Forager pane and pressing Enter on a specific item.

**Item identity.** Protocol (Uniswap V3, Aerodrome, etc.), token pair, chain, fee tier. Pool address truncated with leading/trailing 4 characters.

**Prediction history.** Last 20 predictions with outcomes. Residuals color-coded: within conformal band in `text_dim`, outside in `rose_bright`.

**Tier transition log.** Promotion/demotion events with timestamps and trigger reasons. Shows why the Golem is paying attention to this market.

### Position retrospective modal

Triggered from the Position Retrospectives pane on the FATE screen. This is the centerpiece of Level 2, the most information-dense surface in the Oracle system.

**PnL trajectory chart.** A braille line chart, full modal width, 8 rows tall. X-axis: position lifetime (hours or days). Y-axis: unrealized PnL in USDC. Zero line drawn in `text_ghost` at vertical center.

Positive PnL in `success`, negative in `rose_bright` with faint `rose_dim` fill to zero line (losses are visually heavier). Optimal exit marked with `◇` (`bone`), actual exit with `✗` (`rose`). The gap is regret. Dollar value annotated below: `Regret: $27.40 (held 2.1h past optimal exit)`.

**Entry reasoning.** Collapsible (toggle `e`). The T1/T2 deliberation trace that led to entry. Wrapped text in `text_dim`, truncated to 20 lines. Loop 10 alignment classification annotated at top: `[AlignedCorrect]`, `[MisalignedCorrect]`, `[AlignedIncorrect]`, or `[MisalignedIncorrect]`.

**Vs-inaction analysis.** A two-column comparison:

```
  ACTION                         INACTION
  Entry: 2026-03-13 14:22        (held USDC)
  Exit:  2026-03-15 09:17
  PnL:   -$14.30                 PnL:  $0.00
  Fees:   +$6.20                 Fees: $0.00
  IL:    -$18.10                 IL:   $0.00
  Gas:    -$2.40                 Gas:  $0.00
  Net:   -$14.30                 Net:  $0.00
```

Prospect theory: losses from action feel worse than equivalent losses from inaction [KAHNEMAN-TVERSKY-1979]. The display makes the comparison explicit. The better column glows faintly `success`.

**Hindsight narrative.** Two to three sentences from the weekly retrospective (Loop 12, T1 inference) describing what the Golem would do differently. Cached during review, not generated in real time.

### Heuristic audit detail modal

Triggered from the Heuristic Audit pane on the FATE > Reviews tab.

**Heuristic full text.** The complete heuristic rule, displayed in a bordered box:

```
  ┌─ H-7 ──────────────────────────────────────────────────┐
  │  "Increase position size when accuracy > 80%"          │
  │  Created: 2026-02-28  |  Audits survived: 4            │
  └────────────────────────────────────────────────────────┘
```

**Citation timeline.** Braille sparkline, one dot per day across the heuristic's lifetime. Brightness = citation density. A heuristic cited heavily for two weeks then silent is probably stale.

**Per-citation PnL sparkline.** Below the timeline. Dots above center line in `success`, below in `rose_bright`. Does this heuristic make money when cited?

**Recommendation display.** The Loop 13 audit recommendation, rendered with rationale:

```
  RECOMMENDATION: DEMOTE
  The 2 losses (-$4.80, -$3.60) wiped out the 1 win (+$2.10).
  Overconfidence after high accuracy led to oversized positions.
  Estimated savings if demoted: $6.30 per week.
```

Color: `success` for KEEP, `rose_bright` for DEMOTE, `warning` for INVESTIGATE, `text_dim` for INSUFFICIENT_DATA.

### Feedback loop detail modal

Triggered from the LOOPS pane in the Learning Pulse sub-tab. Shows loop identity (number, name, tier, frequency), corrections count in the last 24 hours vs. expected rate, and success rate as a ProbeGauge.

**Grimoire correlations.** Top 3 Grimoire entries co-occurring with this loop's corrections (from Loop 6 context attribution).

**Threshold contribution.** Gate margin gauge with the 5.0% threshold marked. Shows how far this category is from being trusted enough to act on.

**Last failure event.** Most recent failed prediction with full detail (predicted, actual, residual, context). A concrete example of what goes wrong.

---

## Level 3: Deep (modal, full investigation)

Full statistical detail. These are modals accessed by drilling from Level 2 elements. Anyone who navigates here knows what Expected Calibration Error is. Jargon is acceptable.

### Calibration curve (full)

A 40x20 braille canvas. The ideal diagonal (perfect calibration) is drawn in `text_ghost`. The actual calibration curve is drawn in `rose` with 2-dot-per-cell resolution. Regions where |stated confidence - observed accuracy| > 0.10 are highlighted with `warning` fill between the two curves. The fill makes miscalibrated zones visually obvious: they bulge from the diagonal.

Calibration curve rendering: braille scatter plot using Unicode U+2800-U+28FF range. X-axis is predicted confidence (0-1). Y-axis is actual accuracy (0-1). Each braille cell is a 2x4 dot grid, giving 80x80 effective resolution on the 40x20 cell canvas (80 horizontal dots, 80 vertical dots). Perfect calibration is a diagonal line from bottom-left to top-right. Points above the diagonal mean the Golem is underconfident (actual accuracy exceeds stated confidence). Points below mean overconfident. The actual curve is rendered in `rose_dim`. The diagonal reference line is rendered in `text_ghost`.

Below the curve, three statistics:

```
  ECE: 0.068    ACE: 0.042    Max CE: 0.14 (at stated 85%)
```

ECE: weighted average of per-bin |accuracy - confidence| [NAEINI-2015]. ACE: adaptive binning, more sensitive at extremes. Max CE: worst single bin, locates the most miscalibrated confidence range. Toggle (`r`) overlays per-category curves to reveal whether miscalibration is uniform or concentrated.

### Residual distribution (full)

A horizontal ASCII histogram using block characters (`▁▂▃▄▅▆▇█`). The distribution of prediction residuals (actual - predicted) across all resolutions in the review period.

```
                    ▁
                   ▃█▃
                  ▅████▅
                ▂▇██████▇▂
              ▁▄████████████▄▁
  ──────────────────┼──────────────────
  -2.0             0.0             +2.0
```

The center line (0.0) is drawn in `text_ghost`. If the distribution is centered on zero, the corrector is working (no systematic bias). If the distribution is shifted left or right, a bias exists.

Statistics below: mean, std, skew, kurtosis. Bias annotation: "cool" (shifted left, over-predicting), "warm" (shifted right), "neutral" (centered). The warm/cool vocabulary connects to the palette system. Tab switcher (`1`-`5`) shows per-category distributions.

Residual histogram detail: half-block vertical bars using Unicode U+2581-U+2588 (lower one-eighth block through full block). X-axis: residual bins from -1.0 to +1.0, 20 bins of width 0.1 each. Y-axis: count (auto-scaled to tallest bin). Bars in `rose` for positive residuals (underconfident predictions) and `dream` for negative residuals (overconfident predictions). Rendered in a 40x8 cell area. The zero-residual line is marked with a `|` in `text_ghost`.

### Per-category accuracy time series

Multi-line braille sparklines, one per prediction category, stacked vertically:

```
  fee_rate    ⠤⡤⣤⣤⣤⣤⣤⡤⡤⣤⣤⣤⣤⣤⣤⡤⡤⡤⡤⣤⣤⣤⣤⣤⡤⡤⣤  82%
  slippage    ⠤⠤⡤⡤⣤⣤⡤⡤⡤⡤⡤⣤⣤⣤⡤⡤⠤⡤⡤⣤⣤⣤⣤⣤⡤⡤⡤  76%
  liquidity   ⠤⠤⠤⡤⡤⡤⡤⡤⡤⡤⣤⣤⣤⡤⡤⡤⡤⡤⡤⡤⡤⡤⡤⡤⡤⣤⡤  71%
  price       ⠤⠤⡤⡤⡤⡤⠤⠤⠤⡤⡤⡤⡤⡤⡤⡤⠤⠤⡤⡤⡤⡤⠤⠤⡤⡤⡤  63%
  direction   ⠤⠤⠤⠤⡤⡤⠤⠤⠤⠤⠤⡤⡤⠤⠤⠤⠤⠤⠤⡤⡤⠤⠤⡤⡤⠤⠤  48%
```

Horizontal axis: Golem lifetime (or last 90 days, whichever is shorter). Each braille column represents one day. Vertical axis: 0-100% accuracy.

Regime change events appear as vertical lines in `text_ghost`, spanning all five sparklines. The viewer can see whether accuracy drops correlate with regime changes (they should: accuracy drops after regime changes, then recovers as the corrector adapts).

Colors: `rose` base with per-category brightness modulation (highest accuracy = brightest).

### Environmental model graph

The Golem's learned causal model, rendered as a directed graph using box-drawing and Unicode arrow characters.

```
  ┌──────────┐         ┌──────────┐
  │ gas_price├────────►│ slippage │
  └────┬─────┘         └──────────┘
       │
       ▼
  ┌──────────┐    ┌──────────┐
  │ volatility├───►│ fee_rate │
  └────┬─────┘    └─────┬────┘
       │                │
       ▼                ▼
  ┌──────────┐    ┌──────────┐
  │ direction │    │ liquidity│
  └──────────┘    └──────────┘
```

Nodes are prediction categories and environmental variables. Edges are weighted by learned confidence (from attention forager co-occurrence and corrector regime structure). Node brightness scales with Grimoire citation count. Edge style encodes weight: thick (`━`) high confidence, normal (`─`) medium, thin (`╌`) low. Edge color: `rose` for positive correlation, `rose_dim` for negative.

The graph is the Golem's causal map made visual. The viewer can compare it against their own understanding and spot gaps. Updates on weekly review cycles only.

### Attention forager heatmap

A braille heatmap. Rows: attention items (sorted by tier: ACTIVE at top, then WATCHED, then SCANNED). Columns: time (theta ticks over the last 7 days). Cell intensity: prediction error magnitude for that item at that tick. Bright cells = large errors. Dark cells = small errors or no prediction.

```
  WETH/USDC Base    ⠀⠀⡀⡀⣀⣀⣤⣤⣀⡀⠀⠀⡀⡀⣤⣀⡀⠀⠀⡀⣤⣤⣀⡀
  UNI/ETH Main      ⠀⠀⠀⡀⡀⡀⡀⡀⠀⠀⠀⠀⠀⡀⡀⡀⠀⠀⠀⠀⡀⡀⡀⠀
  ARB/ETH Arb       ⠀⠀⣤⣤⣤⣀⡀⠀⠀⠀⠀⠀⠀⠀⡀⡀⠀⠀⠀⠀⠀⠀⠀⠀
  AERO/ETH Base     ⠀⠀⠀⠀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀
  ...
```

Hot spots (bright clusters) = persistent prediction failure. Horizontal bands = item-specific struggle. Vertical bands = systemic failure at a point in time (regime change, external shock). Read-only. Its value is pattern recognition: the viewer's eye catches clusters that statistical tests miss.

### Shadow experiment detail

When a shadow strategy is active, Level 3 provides the full comparison.

**Dual PnL chart.** Two line charts overlaid, using the ShadowComparisonChart widget. Live strategy in `rose`. Shadow strategy in `dream`. Both on the same vertical axis. Divergence between the two lines represents the performance gap.

**Parameter difference table.** Only differing parameters shown. One-row-per-parameter comparison (live vs. shadow values). If the shadow differs from live by one parameter, the viewer sees the lever being tested.

**Divergence log.** Timestamped events where live and shadow would have acted differently. Each row: timestamp, live decision, shadow decision, outcome. The most actionable data in the shadow system.

### Meta-learning detail

All Loop 14 meta-metrics as braille sparklines, one per row: corrector convergence, dream yield, attention precision, heuristic half-life, time-to-competence. Each spans the Golem's lifetime (or last 90 days). Trend annotations: "improving", "stable", "declining".

Time-to-competence has data points only at generation boundaries. A Golem on generation 3 has three dots. A decreasing sequence means inheritance works.

If predecessors exist, a generational comparison table shows ACC@70% (tick at which the Golem reached 70% accuracy), dream yield, and heuristic half-life per generation. Decreasing time-to-competence = each generation starts smarter.

---

## Animations

Oracle-specific animations supplement the general animation system. All animations respect the lifecycle phase degradation rules: animation intensity decreases in Conservation, becomes intermittent in Declining, and ceases in Terminal.

### Correction ripple

When a residual correction event fires from Loop 1, a brightness wave expands from the Spectre's core outward. 200ms rise to peak brightness, 500ms exponential decay. The wave passes through 2-3 cells of radius, illuminating nearby particles and characters as it passes.

Only 1 in 50 corrections produces a visible ripple. The rest happen silently. The variable-ratio schedule follows Skinner's intermittent reinforcement principle [SKINNER-1938]: unpredictable rewards are more engaging than predictable ones. The viewer never knows which correction will be the visible one. This is the same reinforcement schedule that makes slot machines compelling and fishing absorbing. The ripple is the visual equivalent of a bite on the line.

The 1-in-50 rate uses a geometric distribution (p=0.02). Expected wait: 50 corrections, roughly 3-4 minutes at normal trading speed. Actual interval varies from 1 to 200+. The variance is the point.

```rust
struct CorrectionRipple {
    center: (u16, u16),
    radius: f32,            // current, max 2.5 cells
    brightness: f32,        // 0.0-1.0, decays exponentially
    started_at: Instant,
    rise_duration: Duration,   // 200ms
    decay_rate: f32,
}
```

### Heuristic promotion animation

When Loop 13 promotes a heuristic (recommendation: KEEP) or demotes one (recommendation: DEMOTE), and the Heuristic Audit pane is visible (Level 2), the heuristic's text line animates between tiers.

**Promotion (KEEP).** Text rises over 1 second. Characters arrive at varying speeds (shimmer effect). Color: `bone` during transition, settles to `success` for 2 seconds, then standard `rose`. Promotions feel like crystallization.

**Demotion (DEMOTE).** Text sinks. Characters scatter at varying speeds. Color shifts from `rose` to `rose_dim`. Demotions feel like dissolution.

Only plays when the Heuristic Audit pane is visible. Otherwise logged in the RECENT pane with `●` prefix.

### Dream integration flash

When a creative prediction (from Loop 9, REM counterfactual generation) passes the FDR gate (3 independent confirmations), the DecisionRing flashes from `dream` palette to `rose` palette over 800ms. The flash represents an insight transitioning from dream to waking cognition, from hypothesis to confirmed knowledge.

The flash is a single event, not recurring. It fires once per confirmed creative prediction. At a dream yield of 34%, with roughly 5 dream cycles per day producing 2-3 creative predictions each, this flash fires approximately 3-5 times per day. Rare enough to feel significant. Frequent enough that an attentive owner will see it happen during a session.

The `dream` color holds for 200ms before shifting (cubic easing). The owner's eye is drawn by the color anomaly, and by the time they look, the transition to `rose` is underway.

### Coherence breathing modulation

As described in Level 0. Runs continuously. Second harmonic at 15% amplitude, in-phase when improving, out-of-phase when declining. A viewer watching for 30 seconds will not notice it. After 5 minutes they feel it. After 30 minutes their own breathing has synchronized, and irregularity (declining coherence) registers as discomfort.

Phase degradation: Conservation drops amplitude to 10%. Declining to 5%. Terminal removes the second harmonic entirely.

---

## New widgets

Seven widgets introduced by the Oracle surfaces. Brief specs here. Full specs with PAD modulation and phase degradation follow the format established in the widget catalog.

### 1. PredictionResolutionPulse

Single-character inline indicator: `·` (resolved, `text_dim`), `↑` (accuracy improved, `success`), `↓` (accuracy declined, `rose_bright`). Appears in heartbeat log gutter and Recent Resolutions pane. On state change, flashes to `bone` for one frame, then settles. Phosphor: `↑`/`↓` remain faintly brighter than surrounding `·` for 2 seconds.

PAD: high arousal increases flash brightness. Low pleasure makes `↓` persist one extra second.

```rust
struct PredictionResolutionPulse {
    state: PulseState,           // Neutral | Improved | Declined
    last_change_frame: u64,
    flash_active: bool,
}
```

### 2. AccuracyCalibrationCurve

Braille chart of stated confidence vs. observed accuracy. Compact: 20x10. Full: 40x20. Ideal diagonal in `text_ghost`, actual curve in `rose`, deviation regions (|stated - actual| > 0.10) filled `warning`. ECE below. Static between weekly reviews. On update, new curve fades in over 500ms while old curve fades through phosphor.

PAD: high dominance narrows the deviation threshold to 0.08. Low dominance widens to 0.12.

```rust
struct AccuracyCalibrationCurve {
    bins: Vec<CalibrationBin>,   // stated_lower, stated_upper, observed_accuracy, sample_count
    ece: f64,
    mode: CurveMode,             // Compact | Full
    previous_bins: Option<Vec<CalibrationBin>>,
    last_update_frame: u64,
}
```

### 3. ResidualDistribution

Horizontal histogram (`▁▂▃▄▅▆▇█`), full pane width, 6 rows tall. Center line (residual = 0) in `text_ghost`. Left half (under-prediction) tinted cool; right half tinted warm (shift of 10-15 RGB units). Updates on every resolution with 100ms lerp. Stats below: mean, std, skew, kurtosis.

PAD: low arousal compresses vertical scale. High arousal expands it.

```rust
struct ResidualDistribution {
    bins: Vec<HistogramBin>,     // lower, upper, count, display_height (lerped)
    bin_width: f64,
    mean: f64,
    std_dev: f64,
    skewness: f64,
    kurtosis: f64,
}
```

### 4. AttentionTierIndicator

Three concentric box-drawing rectangles (5x5 cells). Outer = SCANNED (`text_dim`), middle = WATCHED (`rose`), inner = ACTIVE (`rose_bright`). Fill level = fraction of border characters solid vs. dashed. Overflow (>90%): ring flickers. Near-empty (<20%): ring dims to `text_ghost`.

PAD: high arousal makes the ACTIVE ring pulse with the heartbeat.

```rust
struct AttentionTierIndicator {
    active_fill: f32,       // 0.0-1.0
    watched_fill: f32,
    scanned_fill: f32,
    active_max: u16,
    watched_max: u16,
    scanned_max: u16,
}
```

### 5. ActionGateIndicator

Gate icon per category: `▮▮` (filled, `success`) = open, `▯▯` (hollow, `rose_dim`) = closed. Two-character width for readability at status bar scale. State changes animate over 200ms (fill inward on open, drain outward on close). On focus, ghost text shows suppression reason and margin.

PAD: low dominance makes closed gates flicker.

```rust
struct ActionGateIndicator {
    category: CategoryId,
    is_open: bool,
    margin: f64,
    required_margin: f64,
    suppression_reason: Option<String>,
    transition_state: GateTransition,  // Idle | Opening | Closing
}
```

### 6. HeuristicAuditCard

Four-row card per heuristic. Row 1: ID, truncated text, recommendation badge. Row 2: citation count, win rate, avg PnL. Row 3: win-rate ProbeGauge. Row 4: action detail (savings estimate for DEMOTE, boost factor for KEEP). Badge colors: KEEP=`success`, DEMOTE=`rose_bright`, INVESTIGATE=`warning`, INSUFFICIENT=`text_dim`.

PAD: high pleasure brightens KEEP badges. Low pleasure pulses DEMOTE badges.

```rust
struct HeuristicAuditCard {
    heuristic_id: String,
    heuristic_text: String,
    citation_count: u32,
    win_rate: f64,
    avg_pnl: f64,
    recommendation: AuditRecommendation,  // Keep | Demote | Investigate | InsufficientData
}
```

### 7. ShadowComparisonChart

Dual-line braille chart (full pane width, 8 rows). Live strategy in `rose`, shadow in `dream`. Gap fill: `success` when shadow outperforms, `rose_dim` when live outperforms. Scrolls left on each theta tick. Crossing-point flash when one overtakes the other. Gap fill intensifies when the difference exceeds one standard deviation.

PAD: high arousal intensifies crossing flash. Low dominance increases fill opacity.

Phase degradation: Conservation dims shadow line. Declining removes it. Terminal freezes.

```rust
struct ShadowComparisonChart {
    live_pnl: VecDeque<f64>,
    shadow_pnl: VecDeque<f64>,
    width: u16,
    started_at: u64,
    gap_std_dev: f64,
}
```

---

## WebSocket events

The Oracle introduces four new event types on the Event Fabric WebSocket.

```rust
/// Oracle-specific events emitted on the Event Fabric WebSocket.
/// These drive both Oracle-specific screens and ambient Level 0
/// signals across all screens.
enum OracleEvent {
    /// A residual correction was applied by Loop 1.
    /// Emitted on every correction (~15,000/day).
    /// Only 1-in-50 are marked as visible for the correction ripple animation.
    Correction {
        category: PredictionCategory,
        /// The bias shift applied to future predictions (signed).
        bias_shift: f32,
        /// The interval width adjustment (unsigned, always positive).
        interval_adjustment: f32,
        /// Whether this correction should produce a visible ripple.
        /// Determined by geometric distribution with p=0.02.
        visible: bool,
    },

    /// A Grimoire heuristic was promoted or demoted by Loop 13.
    /// Emitted during weekly heuristic audit (4-8 per week).
    Promotion {
        heuristic_id: String,
        /// The direction of the promotion.
        direction: PromotionDirection,
        /// The new recommendation after this promotion/demotion.
        new_recommendation: AuditRecommendation,
    },

    /// Coherence score updated.
    /// Emitted on every theta tick (~1,000/day).
    /// Drives ambient Spectre interference pattern (Level 0)
    /// and breathing modulation.
    Coherence {
        /// Current coherence score (0.0-1.0).
        score: f32,
        /// Rate of change, per minute. Positive = improving.
        /// Drives second-harmonic phase on the heartbeat sine wave.
        delta: f32,
    },

    /// An evaluation loop changed state.
    /// Emitted when a loop starts or completes processing.
    /// Only loops 1-7 emit these frequently; loops 8-14
    /// emit them during dream cycles and reviews.
    LoopState {
        /// Loop identifier (1-14).
        loop_id: u8,
        /// Whether the loop is currently active (processing).
        active: bool,
        /// Brief summary of the last result, if the loop just completed.
        /// e.g., "corrected -0.02 fee_rate bias" or "H-7 demoted"
        last_result: Option<String>,
    },
}

enum PromotionDirection {
    Up,   // KEEP: heuristic validated, retrieval weight increased
    Down, // DEMOTE: heuristic harmful, retrieval weight decreased
}

/// Mirrors the AuditRecommendation from the evaluation architecture.
/// Included here so the event is self-contained (no cross-crate dependency
/// for deserialization).
enum AuditRecommendation {
    Keep,
    Demote,
    Investigate,
    InsufficientData,
}
```

Event rates: Correction ~15,000/day, Coherence ~1,000/day, LoopState ~2,000/day, Promotion ~6/week.

Total Oracle bandwidth: ~3 KB/min. Negligible relative to existing Event Fabric traffic. No batching needed.

The TUI subscribes to all four types unconditionally. Level 0 surfaces consume Coherence events on every screen. Correction events drive ripples only when the sprite is visible. LoopState and Promotion events are buffered (last 100) so the RECENT pane in Learning Pulse shows history on navigation.

---

## References

- [ANDERSEN-2012] Andersen, E., O'Rourke, E., Liu, Y.-E., Snider, R., Lowdermilk, J., Truong, D., Cooper, S., and Popovic, Z. "The Impact of Tutorials on Games of Varying Complexity." *Proceedings of the SIGCHI Conference on Human Factors in Computing Systems (CHI '12)*. ACM, 2012. Argues that tutorials help simple games but hurt complex ones; players of complex systems learn better through discovery. Justifies the Oracle surfaces' progressive complexity model over explicit tutorials.
- [BORNSTEIN-1989] Bornstein, R. F. "Exposure and Affect: Overview and Meta-Analysis of Research, 1968-1987." *Psychological Bulletin*, 106(2), 1989, pp. 265-289. Meta-analysis showing that mere repeated exposure to a stimulus increases positive affect toward it. Supports the design of ambient Oracle signals that build familiarity through passive observation.
- [ISHII-ULLMER-1997] Ishii, H. and Ullmer, B. "Tangible Bits: Towards Seamless Interfaces between People, Bits and Atoms." *Proceedings of the ACM SIGCHI Conference on Human Factors in Computing Systems (CHI '97)*. ACM, 1997. Introduces ambient media: information at the periphery of perception that registers without conscious attention. Justifies the Spectre's particle coherence as a Level 0 Oracle signal.
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. and Tversky, A. "Prospect Theory: An Analysis of Decision under Risk." *Econometrica*, 47(2), 1979, pp. 263-292. Foundational work on loss aversion and reference-dependent preferences. Informs how the Oracle surfaces present losses vs. gains asymmetrically in position retrospectives.
- [NAEINI-2015] Naeini, M. P., Cooper, G. F., and Hauskrecht, M. "Obtaining Well Calibrated Probabilities Using Bayesian Binning." *Proceedings of the AAAI Conference on Artificial Intelligence*, 2015. Introduces the Bayesian Binning into Quantiles (BBQ) method for probability calibration. Informs the ECE calibration curve widget and the Oracle's calibration quality metric.
- [SHNEIDERMAN-1996] Shneiderman, B. "The Eyes Have It: A Task by Data Type Taxonomy for Information Visualizations." *Proceedings of the IEEE Symposium on Visual Languages*. IEEE, 1996. Establishes the "overview first, zoom and filter, details on demand" mantra for information visualization. Directly structures the Oracle's four-level progressive complexity model.
- [SKINNER-1938] Skinner, B. F. *The Behavior of Organisms: An Experimental Analysis*. Appleton-Century-Crofts, 1938. Foundational work on operant conditioning and variable-ratio reinforcement schedules. Explains why the T0-T0-T0-T2 heartbeat rhythm is engaging: unpredictable reward timing produces the strongest engagement loops.
- [TUFTE-1983] Tufte, E. R. *The Visual Display of Quantitative Information*. Graphics Press, 1983 (2nd ed. 2001). Establishes the data-ink ratio principle: every pixel must carry information. Constrains the Oracle surfaces to zero decorative elements; ambient signals work because they are dense with meaning.
- [WEISER-BROWN-1997] Weiser, M. and Brown, J. S. "The Coming Age of Calm Technology." In Denning, P. J. and Metcalfe, R. M. (eds.), *Beyond Calculation: The Next Fifty Years of Computing*. Springer, 1997. Extends ambient media into "calm technology" that informs without demanding focus. Justifies the status bar accuracy number and Spectre coherence as calm Oracle signals.

---

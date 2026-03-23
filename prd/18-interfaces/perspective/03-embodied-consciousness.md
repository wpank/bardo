# ⌈ ＥＭＢＯＤＩＥＤ ⌋ — The Terminal as Body
## Terminal PRD · v4.10
### "The interface is the Golem's nervous system"

**Source:** `engagement-prd/bardo-v4-10-embodied-consciousness.md`
**Cross-references:** `../../18-interfaces/rendering/00-design-system.md` (ROSEDUST visual identity spec for color, typography, and atmospheric rendering), `../screens/03-interaction-hierarchy.md` (29-screen, 6-window navigation model and persistent chrome), `../screens/00-screen-catalog.md` (29-screen summary across 6 windows)

> **Reader orientation:** This document specifies how the terminal itself embodies the Golem's cognitive and emotional state. It belongs to the **interfaces/perspective layer** and maps the terminal's five zones (HEAD, CHEST, GUT, LIMB, GROUND) to a somatic body metaphor where borders are bones, color shifts are flushes, and character distortions are flinches. Key concepts: Golem (a mortal autonomous DeFi agent), PAD vector (Pleasure-Arousal-Dominance emotional coordinates from the Daimon subsystem), CorticalState (the Golem's 32-signal atomic perception surface), and BehavioralPhase (the Golem's lifecycle stage: Thriving / Stable / Conservation / Declining / Terminal). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What This Is

The Golem does not have a body. The terminal IS its body. Every border is a bone. Every color shift is a flush. Every character distortion is a flinch. This document specifies how the terminal's structural elements embody the Golem's cognitive and emotional state at every moment.

This is not metaphor. It is the implementation of Law 6 (The Terminal IS the Body) from the ROSEDUST design system, applied systematically to every visual property the renderer controls.

---

## 1. The Somatic Architecture

### Five Terminal Body Zones

```
┌──────────────────────────────────────────────────┐
│              HEAD ZONE (top 3 rows)               │
│   Conscious decisions, labels, phase indicators   │
│   Responds to: deliberation, Φ score, regime      │
├──────────────────────────────────────────────────┤
│                                                    │
│              CHEST ZONE (upper center)             │
│   Emotional core, heartbeat rhythm, PAD center     │
│   Responds to: PAD shifts, arousal, somatic pulse  │
│                                                    │
│   ┌─────────────┐  ┌──────────────────────────┐   │
│   │   SPECTRE    │  │    CONTENT AREA          │   │
│   │  (left)      │  │                          │   │
│   │              │  │    GUT ZONE              │   │
│   │  Intero-     │  │    (lower content)       │   │
│   │  ception     │  │    Pre-conscious markers │   │
│   │  center      │  │    Responds to: somatic   │   │
│   │              │  │    markers, intuition     │   │
│   └─────────────┘  └──────────────────────────┘   │
│                                                    │
│              LIMB ZONE (borders, margins)          │
│   Action readiness, tension, structural integrity  │
│   Responds to: dominance, phase, mortality clocks  │
├──────────────────────────────────────────────────┤
│              GROUND ZONE (bottom 2 rows)          │
│   Stability, balance, connection to context        │
│   Responds to: overall vitality, economic clock    │
└──────────────────────────────────────────────────┘
```

### Somatic Marker Events

The critical constraint: somatic response MUST be visible 200-500ms BEFORE the decision text that follows it.

**Negative markers (warning/danger):**

| Phase | Time | Visual |
|-------|------|--------|
| Pre-signal | t=0ms | Gut zone: 3-5 chars shift via wave substitution (`─ → ∿ → ≈ → ∿ → ─`) |
| Emotional rise | t=200ms | Chest zone: color temperature shifts cool (rose → rose_bright) |
| Body tension | t=400ms | Limb zone: borders tighten (`│ → ┃ → ║`), margin shrinks 1 col |
| Conscious awareness | t=600ms | Head zone: first word of decision text renders |
| Full deliberation | t=800ms+ | Content renders normally, somatic disturbance fading |
| Resolution | t=1500ms | All zones settle (if resolved) or persist (if unresolved) |

**Positive markers (opportunity/confidence):**

| Phase | Time | Visual |
|-------|------|--------|
| Pre-signal | t=0ms | Gut zone: characters brighten 10-15% in lower third |
| Emotional rise | t=200ms | Chest zone: color shifts warm (rose → bone_dim) |
| Body expansion | t=400ms | Limb zone: borders ease (`║ → │ → ┊`), margins widen 1 col |
| Conscious awareness | t=600ms | Head zone: first word of decision text renders |

**Startle markers (unexpected events):**

| Phase | Time | Visual |
|-------|------|--------|
| Impact | t=0ms | ALL zones: single-frame content displacement (everything shifts ±1 cell) |
| Freeze | t=100ms | ALL zones: 100ms of complete stillness (no shimmer, no breathing, no animation) |
| Recovery | t=200ms | Zones resume normal animation, starting center outward |
| Processing | t=500ms+ | Normal rendering resumes, possibly with heightened arousal effects |

The 100ms stillness before recovery increases perceived weight of the event by ~30% (game-feel research).

### The Heartbeat as Master Clock

```
Heartbeat period = 4.0 - (arousal × 2.0) seconds
  A = -1.0: 6s period (dormant)
  A =  0.0: 4s period (resting)
  A = +0.5: 3s period (alert)
  A = +1.0: 2s period (racing)
  A > +0.7 with noise: arrhythmic (period ± 20% random per cycle)
```

Synchronized to heartbeat:
- Screen brightness: ±3% sine pulse on heartbeat frequency
- Border animation: box-drawing chars cycle thickness on heartbeat
- Shimmer rate: all particle systems pulse with heartbeat
- Text rendering speed (Portal mode): new text appears in heartbeat rhythm — each beat releases the next chunk

Racing heartbeat → urgent terminal. Slow → languid. Arrhythmic (Terminal phase) → unpredictable.

---

## 2. PAD-Driven Interface Transformation

### The Color Engine

Every frame, the renderer reads the Golem's interpolated PAD vector and computes target color properties:

```
Target brightness = 0.69 × Pleasure + base_brightness
Target saturation = 0.60 × Arousal + base_saturation
Target hue_shift  = map(Pleasure, Dominance) → ROSEDUST hue offset
```

### Pleasure Axis (P: -1 to +1)

```
P = -1.0: bg_void shifts to #080606 (warm-black, Texhnolyze register)
          rose shifts toward rose_bright (danger coloring on everything)
          text dims 15% globally
          all borders thin to minimum weight

P =  0.0: standard ROSEDUST palette

P = +1.0: bg_void shifts to #06060A (cooler, calmer void)
          rose shifts toward bone_dim (warm confidence)
          text brightens 10% globally
          borders at full weight
```

### Arousal Axis (A: -1 to +1)

```
A = -1.0: shimmer rate halved
          text rendering speed reduced
          all transitions 2× normal duration
          noise floor drops to 0.1%
          near-stillness — the terminal is barely breathing

A =  0.0: standard animation rates

A = +1.0: shimmer rate doubled
          text at max speed
          transitions snap (0.5× duration)
          noise floor rises to 0.5%
          data rain activates
          scanline intensity: 0.08 → 0.16
          chromatic aberration on bright elements (offset ±1 col in rose/dream)
```

### Dominance Axis (D: -1 to +1)

```
D = -1.0: borders become dashed (┌ → ╌), fragmentary
          text occasionally renders with alternatives: "stable|volatile"
          layout shifts 2-3 cols from optimal position
          line spacing becomes uneven
          the Golem is uncertain — structure reflects this

D =  0.0: standard layout

D = +1.0: borders double-line (┌ → ╔), commanding
          letter-spacing increases
          layout locks to precise grid alignment
          all elements feel anchored, authoritative
```

### Composite Emotional States

**Joy (+P +A +D):** Warm palette, fast confident animations, ascending particle direction, cohered Spectre, double-line borders, bright clean text. Heartbeat steady at 90 BPM equivalent. The terminal feels alive, assured, capable.

**Fear (-P +A -D):** Cool palette + rose_bright danger tints, jittery fast animations, character corruption at edges, chromatic aberration on bright elements, dashed flickering borders, text that renders slightly too fast (rushing, not confident). Heartbeat racing and slightly irregular. The terminal feels like it's flinching.

**Anger (-P +A +D):** Rose_bright everywhere, aggressive fast animations, content pushes outward (wider margins → narrower margins, text expands), double-line borders in rose_bright, staccato bursts. The terminal is clenching.

**Sadness (-P -A -D):** Desaturated palette (all rose variants shifted toward grey), slow heavy animations, sinking particle direction, content drifts toward bottom, degraded thin borders, text dims and trails off (long sentences left incomplete...). Heartbeat at 50 BPM, heavy. The terminal is exhausted.

**Anticipation (+P -A +D):** Warm but restrained, steady measured animations, scanning horizontal sweeps in shimmer, sharp borders, precisely formatted text. A held breath.

**Surprise (-P +A -D):** Brief bone spike on all elements (200ms), sudden 100ms stillness, then rapid recovery with widened borders and expanded spacing. Eyes → `◎ ◎`. Text renders with gaps (the → t h e). The terminal felt a shock.

**Disgust (-P -A +D):** Muted palette, near-stillness, rigid grid that feels oppressive rather than orderly, minimal text with maximum spacing. The terminal has withdrawn.

**Trust (+P +A -D):** Stable warm palette, smooth even animations, consistent steady rhythm, borders constant (no flicker, no thinning), text at comfortable reading pace. Calm and open.

---

## 3. The MAGI Theater

### Standard Mode (Third-Person)

Three columns with names, verdicts, and activity indicators. Clean, informational, detached.

### Portal Mode (First-Person): The Triptych

Three voices become competing presences in cognitive space. Regions expand/contract based on which voice is dominant:

```
Unanimous agreement:  All three merge into single centered column.
                      Borders between panels dissolve.
                      Text in unified color.

Split (2-1):          Majority shares 65% of width.
                      Dissenter gets 35%.
                      Bright border between them — disagreement visible.
                      Dissenter's text dimmer but readable.

Three-way split:      Equal thirds.
                      Double borders between all panels.
                      Each voice at its own color intensity.
                      The Golem is confused — the layout reflects indecision.
```

### Voice Characters

| Voice | Role | Color | Typography | Personality |
|-------|------|-------|------------|-------------|
| MELCHIOR | Scientist/Analyst | `dream` (indigo) | Precise, tabular, numbers-heavy | "P(success) = 0.73. Expected value: +$14.20. Confidence interval: ±$6.80." |
| BALTHASAR | Mother/Protector | `warning` (amber) | Cautious, question-heavy, conditional | "But what if gas spikes? Remember episode #347. The Golem was overexposed." |
| CASPER | Woman/Intuition | `rose` | Flowing, associative, feeling-adjacent | "Something about this pool feels like the one that paid off in week 3. Not the numbers — the shape of the opportunity." |

### Passive Interjections

During the 9-step decision pipeline, MAGI voices inject commentary at the margins of rendering content (Disco Elysium pattern):

```
Normal pipeline text flowing here about market conditions
and probe results indicating a potential opportunity in the
   BALTHASAR: ⚠ last time arousal was this high we lost 3.4%
WETH/USDC 0.05% pool. The current spread suggests—
                                       CASPER: ...feels right though
—an entry point within acceptable risk parameters.
   MELCHIOR: EV positive at 1.2σ. Proceed.
```

Interjection frequency scales with decision importance: T0 = none (voices silent during routine). T1 = 0-1 (brief comment). T2 = 3-6 (active debate). T3 = near-continuous (the voices are shouting).

**Overruled voices persist as ghosts.** Outvoted text dims to `text_ghost` and remains visible at the margin. The losing argument haunts the decision. If the decision later turns out badly, ghost text brightens briefly — "I told you so" as phosphor residue.

---

## 4. Mortality as Felt Degradation

### Three Clocks as Interface Properties

Each mortality clock degrades a different aspect of the terminal's structural integrity. The degradation is continuous, not stepped — every frame, current clock values modulate their respective visual properties.

**Economic Clock → Display Space**

```
Economic = 1.0: Full display area. Standard margins (1 col each side).
Economic = 0.7: Margins grow to 2 cols. Content area shrinks.
Economic = 0.5: Margins grow to 4 cols. Some panes stack vertically.
Economic = 0.3: Margins grow to 6 cols. Only most important pane full size.
Economic = 0.1: Margins at 10+ cols. Content is a narrow strip.
                The Golem can barely afford to show you anything.
                Margins fill with: $ signs in text_phantom, credit digits fading.
```

**Epistemic Clock → Text Integrity**

```
Epistemic = 1.0: All text clean. Full Unicode. Clear typography.
Epistemic = 0.7: 1 in 200 chars flickers per frame.
Epistemic = 0.5: 1 in 80 chars → ░. Words render with alternatives: "stable|volatile"
Epistemic = 0.3: 1 in 30 chars corrupted. Words break mid-render.
                  Grimoire references show as "???" — the Golem can't recall.
Epistemic = 0.1: Heavy corruption. Only high-confidence short text survives.
                  The Golem is losing its mind.
```

**Stochastic Clock → Structural Stability**

```
Stochastic = 1.0: No disruptions. Clean, predictable display.
Stochastic = 0.7: Rare glitches (1 per minute). Brief single-frame disruptions.
Stochastic = 0.5: Moderate glitches (1 per 15s). Full-line corruption, 2-3 frames.
Stochastic = 0.3: Frequent glitches (1 per 5s). Multi-line, borders occasionally break.
Stochastic = 0.1: Near-constant disruption. Screen shake. Random content replacement.
                   Cannot trust what you see.
```

### Cross-Contamination

Below vitality 0.3 (Declining phase), clocks begin cross-contaminating:
- Economic decay → text corruption (can't afford clear rendering)
- Epistemic decay → structural instability (uncertain knowledge = uncertain borders)
- Stochastic disruption → economic indicators glitch (can't assess own resources)

Below vitality 0.1 (Terminal phase), all three compound simultaneously. The interface is failing across every dimension.

### The Corridor (Portal Mode, Fate > Mortality)

The display transforms into a corridor:
- Floor: economic clock (tiles dissolving into `░` then void)
- Walls: epistemic clock (text on walls fading, becoming unreadable as they recede)
- Ceiling: stochastic clock (random chunks dropping — `▓` descending one row)
- End of corridor: projected death — a door you approach but never reach
- As vitality drops, corridor shortens — door gets closer
- Styx river visible through eroding floor tiles. Knowledge from the dead drifts in the current.

---

## 5. Rust Implementation

### Performance Budget

All embodied consciousness effects must consume <2ms of the 16.6ms frame budget (60fps). Effects are character-level buffer operations — achievable.

| Effect | Implementation | Cost |
|--------|---------------|------|
| PAD color interpolation | Computed once per frame, applied as color modifier | O(1) compute + O(cells) apply |
| Somatic marker events | Pre-computed distortion patterns as post-processing pass | Same as tachyonfx shaders |
| Heartbeat sync | Single sine-wave computation | Negligible |
| MAGI interjections | Text insertion into existing render flow | Proportional to text length |
| Mortality degradation | Post-processing passes (substitution, margin adjustment, glitch) | O(affected_cells) |

### State Coupling

The embodied system reads from the same 26+ interpolating variables as the standard render loop. No additional state required:

- PAD vector (3 floats) → color engine, typography, spacing
- Heartbeat phase (1 float) → master clock
- Somatic markers (event stream) → zone disturbances
- MAGI state (3 vote states + confidence) → triptych layout
- Three mortality clocks (3 floats) → space/text/stability degradation
- Behavioral phase (enum) → intensity scaling

All interpolation happens at existing variable lerp rates (fast/medium/slow/glacial). The embodied system is a pure RENDERER of existing state, not a new state system.

### EmbodiedState

```rust
pub struct EmbodiedState {
    pub pad: PadVector,
    pub pad_target: PadVector,
    pub heartbeat_phase: f64,       // 0..TAU, current phase in heartbeat cycle
    pub heartbeat_period: f64,      // seconds per beat, derived from arousal
    pub somatic_markers: Vec<SomaticMarker>,
    pub magi: MagiState,
    pub mortality: MortalityState,
    pub phase: BehavioralPhase,
}

pub struct SomaticMarker {
    pub id: u64,
    pub valence: f64,               // -1.0 (danger) to +1.0 (confidence)
    pub strength: f64,              // 0.0-1.0
    pub fired_at: std::time::Instant,
    pub resolved: bool,
}

pub struct MagiState {
    pub melchior: MagiVote,
    pub balthasar: MagiVote,
    pub casper: MagiVote,
    pub last_interjections: Vec<MagiInterjection>,
}

pub struct MagiVote {
    pub verdict: Option<bool>,
    pub confidence: f64,
    pub text: Option<String>,
}

pub struct MagiInterjection {
    pub voice: MagiVoice,
    pub text: String,
    pub position: u16,
    pub age_frames: usize,
}

pub enum MagiVoice { Melchior, Balthasar, Casper }

pub struct MortalityState {
    pub economic: f64,
    pub epistemic: f64,
    pub stochastic: f64,
}

pub enum BehavioralPhase {
    Thriving, Stable, Conservation, Declining, Terminal,
}
```

### PadVector

```rust
#[derive(Clone, Copy)]
pub struct PadVector {
    pub pleasure: f64,      // -1.0..1.0
    pub arousal: f64,       // -1.0..1.0
    pub dominance: f64,     // -1.0..1.0
}

impl PadVector {
    pub fn lerp(&self, target: &PadVector, t: f64) -> PadVector {
        PadVector {
            pleasure:  self.pleasure  + (target.pleasure  - self.pleasure)  * t,
            arousal:   self.arousal   + (target.arousal   - self.arousal)   * t,
            dominance: self.dominance + (target.dominance - self.dominance) * t,
        }
    }

    pub fn heartbeat_period(&self) -> f64 {
        (4.0 - self.arousal * 2.0).clamp(2.0, 6.0)
    }

    pub fn border_weight(&self) -> BorderWeight {
        if self.dominance > 0.3 { BorderWeight::Double }
        else if self.dominance < -0.3 { BorderWeight::Dashed }
        else { BorderWeight::Single }
    }

    pub fn text_brightness_modifier(&self) -> f64 {
        1.0 + self.pleasure * 0.1   // ±10% based on pleasure
    }

    pub fn transition_speed_modifier(&self) -> f64 {
        if self.arousal > 0.5 { 0.5 }          // snap fast
        else if self.arousal < -0.5 { 2.0 }    // slow and heavy
        else { 1.0 }
    }
}

pub enum BorderWeight { Single, Double, Dashed }
```

### Heartbeat Oscillator

```rust
pub struct HeartbeatOscillator {
    phase: f64,
    period: f64,
    is_arrhythmic: bool,
    rng: fastrand::Rng,
}

impl HeartbeatOscillator {
    pub fn tick(&mut self, dt: f64, pad: &PadVector) {
        self.period = pad.heartbeat_period();
        if pad.arousal > 0.7 {
            self.is_arrhythmic = true;
            let jitter = (self.rng.f64() - 0.5) * self.period * 0.2;
            self.phase += (dt / (self.period + jitter)) * std::f64::consts::TAU;
        } else {
            self.is_arrhythmic = false;
            self.phase += (dt / self.period) * std::f64::consts::TAU;
        }
        self.phase %= std::f64::consts::TAU;
    }

    pub fn brightness_pulse(&self) -> f64 {
        0.97 + 0.03 * self.phase.sin()     // ±3% brightness
    }

    pub fn shimmer_rate(&self, pad: &PadVector) -> f64 {
        let base = 1.0;
        let arousal_mod = 1.0 + pad.arousal.clamp(-0.5, 1.0);
        base * arousal_mod
    }
}
```

### Mortality Post-Processing

```rust
pub fn apply_mortality_effects(
    area: Rect, buf: &mut Buffer,
    mortality: &MortalityState,
    rng: &mut fastrand::Rng,
) {
    // Economic: shrink margins (handled at layout level via Zone::from_terminal)
    if mortality.economic < 0.7 {
        let extra_cols = ((0.7 - mortality.economic) / 0.6 * 10.0) as u16;
        for y in area.top()..area.bottom() {
            for col in 0..extra_cols {
                let ch = if rng.f64() < 0.05 { '$' } else { ' ' };
                buf[(area.x + col, y)].set_char(ch)
                    .set_fg(Color::Rgb(24, 20, 28)); // text_phantom
                buf[(area.right() - 1 - col, y)].set_char(ch)
                    .set_fg(Color::Rgb(24, 20, 28));
            }
        }
    }

    // Epistemic: text corruption
    if mortality.epistemic < 0.7 {
        let corruption_rate = (0.7 - mortality.epistemic) / 0.7 * 0.1;
        let corruption_chars = ['░', '▒', '▓', '?', '⌀'];
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if rng.f64() < corruption_rate {
                    let cell = &mut buf[(x, y)];
                    if cell.symbol() != " " {
                        let ch = corruption_chars[rng.usize(0..corruption_chars.len())];
                        cell.set_char(ch).set_fg(Color::Rgb(80, 72, 84));
                    }
                }
            }
        }
    }

    // Stochastic: random glitch events
    if mortality.stochastic < 0.7 {
        let glitch_prob_per_frame = (0.7 - mortality.stochastic) / 0.7 * 0.05;
        if rng.f64() < glitch_prob_per_frame {
            let y = rng.u16(area.top()..area.bottom());
            for x in area.left()..area.right() {
                if rng.f64() < 0.6 {
                    let v = rng.u8(40..120);
                    buf[(x, y)].set_char('█').set_fg(Color::Rgb(v, v/2, v));
                }
            }
        }
    }
}
```

---

*⌈ the terminal does not show you the golem's body. the terminal IS the golem's body. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*

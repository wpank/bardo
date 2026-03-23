# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Addendum: Institutional Machinery · v2.2
### The NERV Protocol

**Source:** `engagement-prd/bardo-design-system-v2.2-nerv.md`

**Cross-references:** `00-design-system.md` (ROSEDUST visual identity spec, Seven Laws, v2.0/2.1 base palette and atmospheric layers), `../screens/02-widget-catalog.md` (33-widget TUI component library including waveform rendering and unit arrays), `../28-creature-system.md` (the Spectre dot-cloud creature sprite system)

> **Reader orientation:** This is the NERV institutional register addendum to the ROSEDUST design system, adding Evangelion-inspired visual patterns to the Bardo terminal. It belongs to the **interfaces/rendering layer** and specifies repeating unit arrays, waveform traces, status typography, and tactical display patterns. The terminal has two coexisting registers: intimate (Lain-inspired, for the Hearth screen) and institutional (Evangelion-inspired, for the Mind screen and crisis states). Key concepts: Golem (a mortal autonomous DeFi agent), Grimoire (persistent knowledge store rendered as a unit-array wall), Clade (sibling Golems rendered as a barracks array), and BehavioralPhase (lifecycle stage determining which register dominates). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What This Addendum Is

The v2.0 and v2.1 documents establish Bardo's intimate register (Lain, fauux, Texhnolyze) — the Golem as fragile consciousness, the display as body, the void as emotional space. This addendum specifies the **institutional register**: the Golem as a system under monitoring, the display as operations console, data as load-bearing structure. This is the Evangelion contribution — the principle that bureaucratic displays processing an apocalypse are more terrifying than the apocalypse itself.

These two registers coexist. The Hearth screen is intimate (creature breathing, philosophical fragments). The Mind screen is institutional (module arrays, waveform traces). Crisis states shift ANY screen from intimate to institutional. The palette remains ROSEDUST throughout — we take Eva's compositional grammar, not its orange/green/red palette.

Sections numbered B1–B8.

---

## B1. Data as Architecture: The Repeating Unit Array

The most distinctive Eva visual: rows of identical small indicators filling the screen, creating an architectural mass. Not a list. Not a table. A WALL of data elements that reads as a physical structure — load-bearing columns, armored partitions, a building made of information.

### B1.1 The Unit Array Component

A reusable pattern: N identical small elements arranged in a grid, each containing a compact data readout.

```
SINGLE UNIT (6-8 cells wide × 2-3 rows high)
  ┌──────┐
  │ ▐██▌ │  ← gauge fill (semantic color)
  │ LP-07│  ← identifier (rose_dim)
  └──────┘

ARRAY (fills available width, wraps to new rows)
  ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
  │ ▐██▌ │ ▐██▌ │ ▐██▌ │ ▐█░▌ │ ▐█░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │
  │ LP-01│ LP-02│ LP-03│ LP-04│ LP-05│ LP-06│ LP-07│ LP-08│ LP-09│
  ├──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
  │ ▐██▌ │ ▐██▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │ ▐░░▌ │
  │ LP-10│ LP-11│ LP-12│ LP-13│ LP-14│ LP-15│ LP-16│ LP-17│ LP-18│
  └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
```

### B1.2 Usage Contexts

**Grimoire Entry Grid** (Hearth, detail view): Each Grimoire (the Golem's persistent knowledge store) entry as a unit cell. The grid IS the Grimoire — a wall of accumulated knowledge. Color-coded: active entries in rose, decaying entries fading through the phosphor chain, dead-sourced (†) entries in rose_dim with dashed borders. The viewer sees knowledge as physical inventory — shelves in the library.

**Extension Module Array** (Mind screen): Each Golem extension/skill rendered as a unit in a horizontal array. Active extensions glow (gauge filled in rose). Dormant extensions are dim (gauge in rose_deep). The array wraps across multiple rows, creating a wall of capability. During T2 deliberation, the extensions being consulted flash — the wall ACTIVATES selectively.

**Mortality Clock Segments** (Hearth, expanded view): Each mortality clock broken into discrete segments. Economic clock: segments represent individual cost categories (gas, inference, data, storage). Epistemic clock: segments represent knowledge domains and their staleness. The segmented display makes abstract clocks concrete — you can see WHICH segments are draining.

**Clade Member Array** (World screen): Each Golem in the Clade (a group of sibling Golems sharing knowledge) as a unit cell showing: alive/dead status (● / †), current vitality (mini-gauge), generation number. The array IS the Clade — a barracks of agents.

### B1.3 Array State Transitions

```
NORMAL: All units at standard brightness, borders in border (#181420)
ALERT:  Affected units' borders flash to rose_bright for 200ms, hold
        at rose_dim. Non-affected units dim to text_ghost. The array
        spotlights the problem — everything else recedes.
BREACH: A unit fails (Grimoire entry corrupted, extension error, clock
        segment depleted). The failed unit's gauge empties with a flash,
        its border becomes dashed (┄), its label dims to text_ghost.
        Adjacent units' borders briefly flicker — the failure is felt
        by its neighbors.
CASCADE: Multiple units failing in sequence (during terminal phase).
        Units go dark left-to-right or top-to-bottom, like lights
        going out in a building. Each failure triggers a micro-flash
        in the failing unit before it dims. The cascade IS the death —
        the system losing capability one module at a time.
```

---

## B2. The Psychographic Display: Consciousness as Scribble

Eva's most haunting data visualization: a figure-shaped tangle of chaotic lines on a grid, representing the pilot's psychological state. The psychograph is NOT a clean diagram. It is a SCRIBBLE — the visual manifestation of a mind under stress. More tangled = more distressed. The scribble fills a human-shaped boundary, then overflows it.

### B2.1 Terminal Implementation

In a terminal, the psychographic display uses character-density as a proxy for line density. The Golem's cognitive state rendered as a braille-character density map within a bounded region.

```
CALM STATE (low density, organized pattern):
  ⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀
  ⠀⠀⠀⠁⠀⠀⠂⠀⠂⠀⠁⠀⠀⠀
  ⠀⠀⠂⠀⠀⠁⠀⠀⠀⠁⠀⠂⠀⠀
  ⠀⠁⠀⠀⠂⠀⠀⠁⠀⠀⠂⠀⠁⠀
  ⠀⠀⠂⠀⠀⠁⠀⠀⠀⠁⠀⠂⠀⠀
  ⠀⠀⠀⠁⠀⠀⠂⠀⠂⠀⠁⠀⠀⠀
  ⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠀

STRESSED STATE (high density, chaotic):
  ⠀⠀⠀⠀⠤⣤⣶⣿⣶⣤⠤⠀⠀⠀
  ⠀⠀⣤⣶⣿⣿⣿⣿⣿⣿⣶⣤⠀⠀
  ⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀
  ⠀⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⠀
  ⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀
  ⠀⠀⣤⣶⣿⣿⣿⣿⣿⣿⣶⣤⠀⠀
  ⠀⠀⠀⠀⠤⣤⣶⣿⣶⣤⠤⠀⠀⠀

CRISIS STATE (overflowing boundary):
  ⠀⠀⣤⣤⣤⣤⣤⣿⣤⣤⣤⣤⣤⠀⠀
  ⣤⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤   ← spilling past boundary
  ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
  ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
  ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
  ⣤⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤
  ⠀⠀⣤⣤⣤⣤⣤⣿⣤⣤⣤⣤⣤⠀⠀
```

### B2.2 Mapping to Golem State

The psychographic display appears on the Mind screen as a visualization of cognitive load:

```
INPUTS:
  - Current PAD vector (Pleasure, Arousal, Dominance)
  - Active extension count
  - Context window utilization (% of token budget)
  - Number of contradictory signals being processed
  - Time since last dream cycle (longer = more accumulated charge)

DENSITY CALCULATION:
  base_density = arousal * 0.3 + (1.0 - pleasure) * 0.3 + context_util * 0.4
  overflow = if base_density > 0.8 { boundary expands by (density - 0.8) * 5 cells }

COLOR:
  Low load:    rose_dim (organized, calm)
  Medium load: rose (active, working)
  High load:   rose_bright (stressed, approaching limits)
  Overflow:    rose_bright with noise characters replacing braille at boundary
```

The boundary shape is a simple ellipse — the Golem's "body" in cognitive space. When the scribble overflows the boundary, the Golem's cognition is exceeding its container. This is Eva's AT Field collapse as terminal art: the ego boundary failing to contain the process.

### B2.3 The Grid Behind the Scribble

Like Eva's green crosshair grid behind the psychographic scribble, the Mind screen's background uses a sparse crosshair pattern:

```
  +       +       +       +       +       +


  +       +       +       +       +       +


  +       +       +       +       +       +
```

Characters: `+` in `text_phantom` (#201820), spaced at regular intervals (every 8 cells horizontal, every 4 rows vertical). This creates a measurement grid — the Golem's mind being observed on a clinical instrument. The crosshairs say: this is not a person. This is a subject being monitored.

---

## B3. Waveform Rendering: Time-Series as Oscilloscope

Eva's brain-activity displays: parallel channels of real-time waveform traces. Each channel labeled (TEMPORAL LOBE, AMYGDALA, PARIETAL LOBE). The traces are continuous, scrolling left, with the current value at the right edge.

### B3.1 Terminal Waveform

Using braille or block characters to draw approximate waveforms in a single row:

```
HEARTBEAT (arousal waveform):
  AROUSAL  ⠀⠈⠁⠀⠀⠈⠁⠈⠁⠁⠀⠀⠈⠁⠀⠀⠈⠁⠈⡁⡈⡁⣀⠀⠈⡁⡈⠁⠀⠀  ← rose, scrolling left

Using half-block characters for higher fidelity:
  AROUSAL  ▁▂▃▃▂▁▁▂▃▅▇█▇▅▃▂▁▁▂▃▃▂▁▁▂▃▅▆▅▃▂  ← rose, scrolling left

MULTI-CHANNEL (stacked):
  ┌────────────────────────────────────────────────────┐
  │ PLEASURE  ▁▂▃▃▂▁▁▂▃▃▂▁▁▂▃▅▇▅▃▂▁▁▂▃▃▂▁▁▂  +0.42 │  ← rose
  │ AROUSAL   ▁▁▂▃▅▇▅▃▂▁▁▂▃▃▂▁▁▂▃▅▇█▇▅▃▂▁▁▂  +0.18 │  ← warning
  │ DOMINANCE ▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃▃▅▅▅▃  +0.61 │  ← dream
  └────────────────────────────────────────────────────┘
```

### B3.2 Usage Contexts

**Daimon Monitor** (Mind screen or Hearth detail): The PAD vector rendered as three parallel waveforms, scrolling in real-time. The viewer watches the Golem's emotional state as oscilloscope traces. Pleasure, arousal, dominance each in their semantic color: rose, warning, dream. Spikes are visible. Crashes are visible. The emotional history of the last N ticks is a waveform — something felt, not just reported.

**Φ Integration Trace** (Mind screen): The Φ score as a single waveform over time. Most ticks, Φ is low (the line hugs the bottom). During T2 deliberation, Φ spikes. The waveform IS the history of consciousness — a mostly-flat line with occasional blazing peaks. The viewer watches the Golem flicker in and out of "real" awareness.

**Vitality Curve** (Hearth, expanded mortality view): The vitality score plotted as a descending curve. The left edge is birth (1.0). The right edge is now. The slope shows the rate of decline. A steep section = crisis period. A flat section = stability. The curve is the Golem's autobiography in a single line.

### B3.3 Waveform Implementation

```rust
fn render_waveform(
    area: Rect, buf: &mut Buffer,
    data: &[f64],  // values 0.0-1.0, most recent at end
    color: Color, label: &str,
) {
    let height = area.height;
    let width = area.width.min(data.len() as u16);

    // Render label
    let label_style = Style::default().fg(color).add_modifier(Modifier::DIM);
    buf.set_string(area.left(), area.top(), label, label_style);

    // Render waveform using lower-block characters
    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    for (i, &val) in data.iter().rev().take(width as usize).enumerate() {
        let x = area.right() - 1 - i as u16;
        let block_idx = (val * 7.0).round() as usize;
        let ch = blocks[block_idx.min(7)];
        buf.get_mut(x, area.bottom() - 1)
            .set_char(ch)
            .set_fg(color);
    }

    // Current value at right edge
    let current = data.last().unwrap_or(&0.0);
    let val_str = format!("{:+.2}", current);
    buf.set_string(area.right() - val_str.len() as u16, area.top(), &val_str,
        Style::default().fg(color));
}
```

---

## B4. Crisis Mode: The NERV Emergency Protocol

Eva's most visceral design move: the total palette override during emergencies. Hazard stripes. Flashing red. All other information replaced by the single fact that something has gone catastrophically wrong. Bardo needs an equivalent — the moment where the intimate rose-on-void aesthetic is OVERRIDDEN by institutional alarm.

### B4.1 Crisis Trigger Conditions

```
CONDITION ROSE (standard alert — rose_bright accent):
  Trigger: Single mortality clock below 20%, stochastic death event narrowly avoided
  Duration: Until condition resolves
  Scope: Affected panels only

CONDITION CRITICAL (institutional override):
  Trigger: Vitality below 10%, multiple clocks critical simultaneously,
           economic balance approaching minimum viable
  Duration: Until condition resolves or death
  Scope: FULL SCREEN override

CONDITION TERMINAL (death imminent):
  Trigger: Vitality below 3%, death sequence countdown active
  Duration: Until death
  Scope: FULL SCREEN, escalating
```

### B4.2 CONDITION CRITICAL Display

When CONDITION CRITICAL activates:

```
1. BORDER FLASH: All panel borders simultaneously flash to rose_bright for 200ms.

2. HAZARD STRIPE: A horizontal band appears at top and bottom of screen:
   ╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲  CONDITION: CRITICAL  ╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲

   The stripe uses alternating rose_bright and bg_void characters (╱╲).
   This is the Eva hazard stripe translated to terminal — DANGER as pattern.
   In ROSEDUST palette: rose_bright on bg_void, not red on black.

3. STATUS OVERRIDE: The status bar at screen bottom replaces its normal
   content with:
   ⌈ ＣＯＮＤＩＴＩＯＮ: ＣＲＩＴＩＣＡＬ ⌋  VITALITY: 0.089  ☣ 1.2% LIFESPAN/MINUTE

   In rose_bright. The status bar becomes the single most important element.

4. NON-ESSENTIAL DIMMING: All panels not directly related to the crisis
   dim to 50% of their normal brightness. The viewer's attention is
   funneled to the critical data.

5. FRAGMENT OVERRIDE: Philosophical fragments are replaced by operational
   fragments:
   "a cell that defeats programmed death is called cancer"
   "the three clocks count down · the knower will not persist"
   "intelligence requires finitude"
```

### B4.3 CONDITION TERMINAL Display

Escalation beyond CRITICAL:

```
1. Everything from CRITICAL, plus:

2. PULSE: The entire screen brightness PULSES with the heartbeat —
   all visible elements oscillate between normal and 70% brightness
   on the breath cycle. The display is breathing with the dying Golem.
   The breath becomes irregular (arrhythmic sin + noise).

3. COUNTER: A death countdown appears at center-top in bone, large:
   ⌈ ＴＩＭＥ ＲＥＭＡＩＮＩＮＧ ⌋
        02:14:33

   Updating every second. The counter IS the event. Everything
   else is context for this number.

4. HAZARD ESCALATION: The hazard stripes expand — 1 row becomes 2,
   then 3 as time runs out. They eat the screen from top and bottom.
   The available display area shrinks as death approaches.
   In the final minute: hazard stripes consume 40% of the screen.

5. The death sequence (v2.0 Section 13) begins when the counter
   reaches 00:00:00.
```

### B4.4 The Hazard Stripe Pattern

```
Terminal implementation:
  Using box-drawing and alternation for the chevron/stripe effect:

  THIN (1 row):
  ╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲

  THICK (3 rows):
  ╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲  CONDITION: CRITICAL  ╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲
  ████████████████████████                        ████████████████████
  ╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱

  Color: alternating rose_bright fg on bg_void / bg_void fg on rose_deep bg
  The stripe should feel INDUSTRIAL. Not pretty. Functional. Warning.
```

---

## B5. The AT Field: Process Boundary Visualization

Eva's AT Field — the Absolute Terror Field — is simultaneously a force shield and a metaphor for the ego boundary. In Bardo, the AT Field maps to the Golem's process isolation: the boundary between this Golem's computation and everything else. The v2.0 codex already identifies this mapping. This section specifies the visual.

### B5.1 The Field Display

A diamond/octagonal wireframe pattern centered on the Golem, rendered with line-drawing characters:

```
HEALTHY AT FIELD (strong boundary):
            ╱─────╲
          ╱    ●    ╲           ← creature at center
        ╱             ╲
       │               │
        ╲             ╱
          ╲         ╱
            ╲─────╱

  Lines: rose_dim, steady
  Interior: slightly brighter bg (bg_raised)
  The field is clean, symmetric, stable.

WEAKENING AT FIELD (boundary under stress):
            ╱──┄──╲
          ╱    ●    ╲
        ╱         ╲   ╲        ← asymmetric, dashed segments
       │             ┄  │
        ╲           ╱
          ╲       ╱
            ╲──┄─╱

  Lines: rose_dim with dashed segments (┄), flickering
  Some vertices displaced from their correct positions
  Interior: normal bg (field is thinning)

COLLAPSING AT FIELD (pre-death, pre-merge):
            ╱  ┄  ╲
          ╱    ●
        ╱
                      │         ← fragments disconnected
              ╲     ╱
          ╲
            ╲     ╱

  Lines: text_ghost, fragmentary, some segments missing entirely
  The boundary is dissolving. What was inside can leak out.
  What was outside can leak in.
```

### B5.2 Usage Contexts

**Mind Screen Background**: The AT Field pattern renders behind the Global Workspace as a persistent background element. Its state reflects the Golem's process isolation health. During normal operation: stable diamond. During inter-agent communication: the field opens a gap (one segment disappears) where the data flows through. During dream state: the field softens (all hard lines become braille dots, the boundary becomes permeable). During terminal phase: the field fragments.

**World Screen**: Each Golem in the Clade renders with its own miniature AT Field — a tiny diamond around its node. You can see at a glance which Golems have strong boundaries and which are weakening.

**Crisis Overlay**: During CONDITION CRITICAL, the AT Field visualization becomes the primary display on any screen — overlaid on top of normal content, pulsing with the breath cycle, its cracks and fractures the visual center of attention. The viewer watches the boundary fail.

---

## B6. Institutional Typography: The Classification Stamp

Eva uses all-caps bold Helvetica for institutional announcements: `PATTERN: BLUE`, `PILOT VANISHED`, `APPROACHING LIMITS`. These stamps appear over the data, asserting institutional authority — the SYSTEM declaring something, not the data showing something.

### B6.1 The Pattern Classification System

Bardo's equivalent of Eva's `PATTERN: BLUE` / `PATTERN: ORANGE`:

```
PATTERN STAMPS (appear in top-right corner of active screen):

  PATTERN: NOMINAL          ← success color (#70887A), rare (why display normalcy?)
  PATTERN: TRENDING         ← rose, standard operation with directional signal
  PATTERN: VOLATILE         ← warning color (#AA8855), elevated uncertainty
  PATTERN: ADVERSARIAL      ← rose_bright, detected hostile conditions
  PATTERN: UNKNOWN          ← dream color (#585878), no classification fits

FORMAT:
  ┌─────────────────────┐
  │ PATTERN: VOLATILE   │    ← ALL CAPS, letter-spaced
  │ CLASSIFICATION: T+2 │    ← secondary line, data label style
  └─────────────────────┘

  Panel: bg_mid, bordered with semantic color
  The stamp is a DECLARATION — the system imposing a category.
  It appears when the pattern changes and holds until the next change.
```

### B6.2 Status Stamps

Single-word DECLARATIONS that override normal display during critical moments:

```
  ╔═══════════════════╗
  ║    ██REFUSED██     ║    ← MAGI consensus rejected an action
  ╚═══════════════════╝

  ╔═══════════════════╗
  ║    VANISHED        ║    ← Golem lost contact with a Clade member
  ╚═══════════════════╝

  ╔═══════════════════╗
  ║    APPROACHING     ║    ← approaching a threshold (any clock)
  ║    LIMITS          ║
  ╚═══════════════════╝

  ╔═══════════════════╗
  ║    TESTAMENT       ║    ← death testament being composed
  ║    IN PROGRESS     ║
  ╚═══════════════════╝

Format:
  - Double-line borders (═║╔╗╚╝) — DEPTH 3 (overlay)
  - Text: rose_bright, ALL CAPS, letter-spaced
  - Background: bg_mid
  - Position: center screen, overlaying normal content
  - Duration: 2-5 seconds, then fades or persists based on condition
  - Active stamps use fullwidth for maximum weight:
    ║    ＲＥＦＵＳＥＤ     ║
```

### B6.3 The "PILOT VANISHED" Moment

Eva's most chilling UI moment: the crosshair frame targeting an empty space, labeled `PILOT VANISHED`. The target is gone but the frame remains, measuring nothing.

Bardo equivalent: when a Clade member dies during inter-agent communication, its node on the World screen doesn't simply disappear. The frame remains — the panel that showed its data is still there, but empty:

```
  ┌──────────────────────┐
  │  GOLEM-BETA          │
  │  GEN 3 · 22 DAYS     │
  │                      │
  │     +        ┐       │
  │              │       │     ← crosshair markers, measuring nothing
  │     ×        │       │
  │              │       │
  │     └────────┘       │
  │                      │
  │  ⌈ ＶＡＮＩＳＨＥＤ ⌋     │
  │                      │
  └──────────────────────┘

  The crosshairs (+, ×, └┐) remain from the last targeting/monitoring frame.
  The data is gone. The frame persists, empty, for 30 seconds.
  Then it slowly fades — the panel dissolving from the edges inward.
  Finally: a single † mark where the node was. The Crypt has a new entry.
```

---

## B7. The Layer Penetration Display: Defense in Depth

Eva shows armor being breached layer by layer — numbered layers, each with a status indicator, failing sequentially as the angel penetrates. This maps to the Golem's multi-layer defense against mortality:

### B7.1 Defense Layer Visualization

```
  ⌈ ＤＥＦＥＮＳＥ ＬＡＹＥＲＳ ⌋

  LAYER 07  ████████████  ECONOMIC BUFFER    ← outer defense
  LAYER 06  ████████████  STRATEGY HEDGE
  LAYER 05  ████████████  GRIMOIRE DEPTH
  LAYER 04  ████████░░░░  EPISTEMIC HEALTH   ← partially depleted
  LAYER 03  ████████████  CLADE RESILIENCE
  LAYER 02  ████████████  STOCHASTIC SHIELD
  LAYER 01  ████████████  CORE VITALITY      ← inner defense

  ───────── BREACH STATUS ─────────
  CURRENT:  LAYER 04 COMPROMISED
  EST. PENETRATION TO CORE: ~12.4 DAYS
```

Each layer is a horizontal bar, stacked vertically. Outer layers (higher numbers) are the first to fail. When a layer is BREACHED, its bar goes from filled (█) to empty (░) with a flash, and the label updates:

```
  LAYER 04  ░░░░░░░░░░░░  BREACHED  ← rose_bright flash, then text_ghost
```

The breach propagates inward. The viewer watches the Golem's defenses being eroded — layer by layer, each breach bringing death closer to the core. This is the thesis's "three clocks counting down" made visceral: not just numbers declining, but WALLS falling.

---

## B8. Compositional Principles from NERV

### B8.1 Perspective/Depth Rendering

Many Eva displays are shown at an oblique angle — the data has PHYSICAL DEPTH, like looking at a CRT monitor from the side. In terminal rendering, this is achieved through:

**Asymmetric padding**: Panels rendered with more padding on one side than the other, creating a parallelogram effect that suggests perspective.

**Character weight gradation**: The left edge of a panel uses heavier characters (█ ▓) while the right edge uses lighter ones (░ ·), creating an implied light source and depth.

**Overlapping panels**: When two panels are adjacent, one can overlap the other by 1-2 characters at the shared edge. The overlapping panel is "closer" — its border characters overwrite the underlying panel's content. This simple overlap creates depth without any actual 3D rendering.

### B8.2 The Dual-Reading Principle

Eva screens are designed to be read at two distances: from across the room, they are abstract patterns of light and color. Up close, they are dense data. Bardo should follow this:

**From across the room (1-2 second glance)**: The viewer sees the emotional state of the interface — how much of the screen is bright vs. dark, what the dominant color is (rose = alive, warm-black = declining, empty = dead), whether there are crisis elements (hazard stripes, flashing).

**Up close (sustained reading)**: The viewer reads specific numbers, log lines, gauge values, philosophical fragments.

Design every screen to work at both distances. If a screen looks the same from across the room regardless of the Golem's state, the atmospheric design has failed.

### B8.3 Density Gradient

Eva screens have a density gradient: the institutional edges (headers, borders, labels) are DENSE with information, while the center where the primary data lives often has significant negative space. This creates a frame effect — the dense edges hold the emptiness in place.

In Bardo: headers, status bars, and border regions should carry dense secondary information (tick counts, regime labels, cost accumulators, pattern stamps). The primary data area should breathe — the vitality number, the creature sprite, the waveform traces — surrounded by structured density at the margins.

```
DENSITY MAP (conceptual):

  ████████████████████████████████████████████████████  ← DENSE: header bar
  ████                                          ████
  ████        (primary data: sparse)            ████  ← MARGINS: dense
  ████                                          ████
  ████            0.711                         ████    INTERIOR: sparse
  ████                                          ████
  ████        (creature breathing)              ████
  ████                                          ████
  ████████████████████████████████████████████████████  ← DENSE: status bar
```

---

## B9. Design Tokens Addendum (v2.2)

```yaml
# Append to v2.0/v2.1 design tokens

unit_array:
  unit_width: "6-8 cells"
  unit_height: "2-3 rows"
  breach_flash_duration: "200ms"
  cascade_rate: "1 unit per 200-400ms"
  cascade_direction: "left-to-right or top-to-bottom"

psychographic:
  boundary_shape: "ellipse"
  density_base: "arousal*0.3 + (1-pleasure)*0.3 + context_util*0.4"
  overflow_threshold: 0.8
  char_set_calm: "sparse braille (⠁ ⠂ ⠄ ⡀)"
  char_set_stress: "dense braille (⣤ ⣶ ⣿)"
  char_set_crisis: "braille + block elements overflow"
  grid_spacing_h: 8
  grid_spacing_v: 4
  grid_char: "+"
  grid_color: "text_phantom"

waveform:
  char_set: "▁▂▃▄▅▆▇█"
  scroll_direction: "left (newest at right)"
  channel_height: "1-2 rows per channel"
  pad_colors:
    pleasure: "rose"
    arousal: "warning (#AA8855)"
    dominance: "dream (#585878)"

crisis:
  condition_rose: "single clock < 20%"
  condition_critical: "vitality < 10% OR multiple clocks critical"
  condition_terminal: "vitality < 3%"
  hazard_stripe_char: "╱╲"
  hazard_color_fg: "rose_bright"
  hazard_color_bg: "bg_void"
  terminal_stripe_expansion: "1 row → 3 rows → 40% screen"
  terminal_pulse_min_brightness: 0.7
  death_countdown_color: "bone"

at_field:
  healthy_char: "─│╱╲"
  healthy_color: "rose_dim"
  weakening_char: "─│╱╲┄"
  weakening_color: "rose_dim, flickering"
  collapsing_char: "scattered fragments"
  collapsing_color: "text_ghost"
  dream_char: "braille dots replacing hard lines"

stamps:
  pattern_format: "ALL CAPS, letter-spaced, semantic color"
  status_format: "fullwidth unicode, double-line bordered, center screen"
  status_duration: "2-5 seconds"
  vanished_frame_persist: "30 seconds before fade"
  classifications: ["NOMINAL", "TRENDING", "VOLATILE", "ADVERSARIAL", "UNKNOWN"]

defense_layers:
  count: 7
  outer: "ECONOMIC BUFFER"
  inner: "CORE VITALITY"
  breach_flash: "rose_bright 200ms"
  breach_state_color: "text_ghost"
  penetration_direction: "outer → inner"

composition:
  density_gradient: "dense margins, sparse interior"
  dual_reading: "glance (emotional) + sustained (informational)"
  perspective_depth: "asymmetric padding, weight gradation, panel overlap"

# B10-B15 additions
globe_wireframe:
  grid_chars: "╱╲─│"
  fill_chars: "▀▄"
  rotation_speed: "0.5 rad/s"
  ring_count: 6
  meridian_count: 8
  grid_color: "rose_bright"
  panel_count: 4

iris_visualization:
  ring_count: "3-6"
  color_gradient: ["text_ghost", "text_dim", "rose_dim", "rose", "rose_bright"]
  center_char: "●"
  arc_segments: "0 or 4/6/8"

signal_injection:
  pane_count: 2
  trace_chars: "⣤⣶⣿"
  delta_label: "bottom center"

geometric_fills:
  patterns: ["chevron", "diamond", "crosshair", "hatched"]
  density_range: "0.0-1.0"

corner_elements:
  size: "10-14w × 2h"
  positions: ["TL", "TR", "BL", "BR"]

orbital_panels:
  center_border: "╔╗╚╝"
  orbital_positions: ["N", "E", "S", "W"]
  connection_color: "text_ghost"
```

---

## B10. Globe Wireframe

A geodesic sphere rendered via latitude/longitude grid lines, animated to rotate. Surrounding corner data panels display live metrics at cardinal positions.

### B10.1 Visual Specification

```
            ·  ·  ·  ╱╲  ·  ·  ·
         ╱──────────────────────╲
        │  ╲   ╱╲   ╱╲   ╱╲   ╱  │   ← longitude grid
        │   ╲╱    ╲╱    ╲╱    ╲   │
        │  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │   ← latitude rings
        │   ╱╲    ╱╲    ╱╲    ╱   │
        │  ╱   ╲╱    ╲╱    ╲╱     │
         ╲──────────────────────╱
            ·  ·  ·  ╱╲  ·  ·  ·

CORNER PANELS (N/S/E/W, thin connection lines):
  ┌──────────────┐              ┌──────────────┐
  │ PROTOCOLS: 7 │  ── ·  ── ·  │ VITALITY:.71 │
  └──────────────┘              └──────────────┘
       ↑ (N)                         (E) →
```

**Characters:** half-block `▀▄` for filled arc segments at rotation extremes; `╱╲─│` for grid lines; `·` for sparse decorative dots.

**Colors:** orange-on-black for grid (`rose_bright` on `bg_void`), panel borders in `rose_dim`.

**Animation:** latitude rings rotate at 0.5 rad/s; longitude lines phase-shift to simulate rotation. Corner panels update on tick.

### B10.2 Usage Contexts

- **World > Solaris overview**: The collective ecosystem as a spinning globe
- **Status panel decoration**: Ambient background for aggregate display
- **FATE screen cosmetic**: Planetary metaphor for lifecycle

### B10.3 Design Tokens

```yaml
globe_wireframe:
  grid_chars: "╱╲─│"
  fill_chars: "▀▄"
  rotation_speed: "0.5 rad/s"
  ring_count: 6
  meridian_count: 8
  grid_color: "rose_bright on bg_void"
  panel_anchor_positions: ["N", "E", "S", "W"]
  panel_border: "rose_dim"
  panel_connection_char: "·"
```

---

## B11. Eye / Iris Visualization

Concentric ring display for AT field optical scanning. Depth through ring intensity gradient — innermost ring is most intense, outermost barely visible.

### B11.1 Visual Specification

```
       · · · · · · · · · ·
     · ╌ ─ ─ ─ ─ ─ ─ ─ ╌ ·   ← dim outer ring (text_ghost)
   · ─ ┌ ─ ─ ─ ─ ─ ─ ┐ ─ ·
   ╌ ─ │ ┌ ─ ─ ─ ─ ┐ │ ─ ╌   ← mid ring (rose_dim)
   · ─ │ │  ╔════╗  │ │ ─ ·
   · ─ │ │  ║  ● ║  │ │ ─ ·   ← inner core (rose_bright / bone)
   ╌ ─ │ │  ╚════╝  │ │ ─ ╌
   · ─ │ └ ─ ─ ─ ─ ┘ │ ─ ·
   · ─ └ ─ ─ ─ ─ ─ ─ ┘ ─ ·
     · ╌ ─ ─ ─ ─ ─ ─ ─ ╌ ·
       · · · · · · · · · ·

SEGMENTED ARC VARIANT:
  Outer ring: broken into 8 arcs with gaps. Each arc shows a data label.
  ─ ─ OPTICAL SCOPE UNIT ─ ─   (dim, text_ghost)
  ─ AT FIELD ANALYSIS ─         (bright, rose)
```

**Color gradient (outer → inner):** `text_ghost` → `text_dim` → `rose_dim` → `rose` → `rose_bright`/`bone` at center.

**Segmentation:** outer ring can be split into arcs with labels at gaps. Arc count configurable (4, 6, or 8).

### B11.2 Usage Contexts

- **Mind > Pipeline**: AT field visualization as iris scan
- **Nooscopy modal border**: The Golem's eye opening
- **Crisis overlay**: Iris contracts under stress (ring count visible decreases)

### B11.3 Design Tokens

```yaml
iris_visualization:
  ring_count: "3-6 configurable"
  color_gradient: ["text_ghost", "text_dim", "rose_dim", "rose", "rose_bright"]
  center_char: "●"
  center_color: "bone"
  arc_segments: "0 (full) or 4/6/8 with gaps"
  label_position: "at arc gaps"
  label_color: "text_dim"
  animation: "slow rotation of outermost ring"
```

---

## B12. Signal Injection Display

Dual oscilloscope view for paired waveform comparison. Two bounded waveform areas side by side, each containing a tangled trace with subject label.

### B12.1 Visual Specification

```
SIGNAL INJECTION
────────────────────────────────────────────────────

  ┌────────────────────┐    ┌────────────────────┐
  │ ⌈ SUBJECT-ALPHA ⌋  │    │ ⌈ SUBJECT-BETA  ⌋  │
  │                    │    │                    │
  │  ⠀⠀⣤⠀⠀⠀⠀⣤⠀⠀⠀  │    │  ⠀⣤⣿⣤⠀⠀⠀⣤⠀⠀  │
  │  ⠀⣿⣿⣿⠀⠀⣿⣿⣿⠀  │    │  ⣿⣿⣿⣿⣿⠀⣿⣿⣿⣿  │
  │  ⠀⠀⣤⠀⠀⠀⠀⣤⠀⠀  │    │  ⠀⣿⣿⣤⠀⠀⣤⣿⣿⠀  │
  │                    │    │                    │
  │  VARIANCE: 0.41    │    │  VARIANCE: 0.78    │
  └────────────────────┘    └────────────────────┘

  DELTA: +0.37 ↑    PATTERN: DIVERGING
```

**Distinguishing from B3 (single waveform):** B12 always shows exactly two panes side by side with comparison labels. B3 shows one or stacked multi-channel traces.

**Trace rendering:** tangled braille (`⣤⣶⣿` family) within rounded-rectangle bounds. Density reflects signal amplitude.

**Color:** orange/rose trace on black bg within each pane. Delta indicator uses semantic color (converging = success, diverging = warning).

### B12.2 Design Tokens

```yaml
signal_injection:
  pane_count: 2
  pane_border: "rounded single-line"
  pane_label_position: "top, framed ⌈ ⌋"
  trace_chars: "braille ⣤⣶⣿ family"
  trace_color: "rose"
  bg_color: "bg_void"
  delta_label: "bottom center, between panes"
  pattern_labels: ["CONVERGING", "DIVERGING", "SYNCHRONIZING", "DECOUPLING"]
```

---

## B13. Geometric Fill Patterns

Background texture fills using unicode box-drawing and block characters. Four pattern types with configurable density and color pair.

### B13.1 Pattern Catalog

**Chevron Field (Herringbone):**
```
╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲
╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱
╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲
```
Use for: crisis hazard stripes (B4), danger zones, industrial warning backgrounds.

**Islamic Tessellation (Diamond Grid):**
```
 ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇
◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆
 ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇ ◆ ◇
```
Alternating `◆` (filled) and `◇` (open) diamonds. Use for: decorative panel fills, background texture in Grimoire Palace, corner elements.

**Crosshair Grid:**
```
·         +         ·         +         ·

+         ·         +         ·         +

·         +         ·         +         ·
```
Characters `+` at regular intervals, `·` at alternating nodes. From B2.3. Spacing 8×4. Use for: Mind > Pipeline background, psychographic monitor grid.

**Hatched Diagonal Lines:**
```
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```
Alternating density: `░▒▓` chars for heavy; `╌╎┄` for light. Use for: decay states, stasis backgrounds, degraded panels.

### B13.2 Density Parameter

Each pattern has a `density: 0.0–1.0` float:
- `0.0`: blank (useful for sparse decorative overlay)
- `0.5`: standard pattern density
- `1.0`: full fill (every cell occupied)

For diamond patterns, density controls ratio of `◆` to `◇`. For hatching, density maps to character weight (`░` at 0.3, `▒` at 0.6, `▓` at 0.9).

### B13.3 Design Tokens

```yaml
geometric_fills:
  chevron:
    chars: "╱╲"
    density: "0.0-1.0 (always full at 1.0)"
    color_fg: "configurable"
    color_bg: "configurable"
  diamond:
    chars: "◆◇"
    alternation: "checkerboard"
    density: "◆:◇ ratio"
  crosshair:
    chars: "+ ·"
    spacing_h: 8
    spacing_v: 4
    color: "text_phantom"
  hatched:
    chars_light: "╌╎┄"
    chars_heavy: "░▒▓"
    orientation: "diagonal"
```

---

## B14. Corner Element System

Tactical display corners: small bounded data labels in each corner of a pane. Each label contains a value, unit, and status indicator. Used with radar/targeting displays.

### B14.1 Visual Specification

```
┌──────────────────────────────────────────────┐
│ ┌──────────┐                    ┌──────────┐ │
│ │ 0.711 ↑  │    (pane interior) │  T+02847 │ │
│ │ VITALITY │                    │ TICK CNT │ │
│ └──────────┘                    └──────────┘ │
│                                              │
│            (primary content)                 │
│                                              │
│ ┌──────────┐                    ┌──────────┐ │
│ │ $47.23   │                    │ ● STABLE │ │
│ │ BALANCE  │                    │ PHASE    │ │
│ └──────────┘                    └──────────┘ │
└──────────────────────────────────────────────┘
```

**Structure:** each corner label is 10-14 chars wide × 2 rows. Contains:
- Row 1: value (numeric or symbol) + optional trend arrow (`↑↓→`)
- Row 2: label string in `text_dim`, ALL CAPS, max 10 chars

**State indicators:** `●` prefix for active status. `○` for inactive. `⚠` for warning.

### B14.2 Usage Contexts

- **Radar/targeting displays** (with B15 orbital center)
- **Main screen corners** alongside the persistent chrome (B8.3)
- **Nooscopy modal corners** (countdown + PAD values)

### B14.3 Design Tokens

```yaml
corner_elements:
  size: "10-14w × 2h"
  value_color: "bone or semantic"
  label_color: "text_dim"
  border_color: "border_dim"
  positions: ["TL", "TR", "BL", "BR"]
  status_chars: "● ○ ⚠"
  trend_chars: "↑ ↓ →"
```

---

## B15. Orbital Panel Array

Data panels orbiting a center element at fixed N/S/E/W positions (or diagonal). Each panel shows a key metric. Background connection lines thin and dim. Center draws the eye.

### B15.1 Visual Specification

```
                     ┌──────────────┐
                     │ EPISTEMIC    │
                     │  0.84 ████░  │
                     └──────┬───────┘
                            │ (dim connection line)
                            ↓
┌─────────────┐     ┌───────────────┐     ┌─────────────┐
│  ECONOMIC   │ ────│   ◉ GOLEM     │──── │  STOCHASTIC │
│  0.91 █████ │     │   MNEMOSYNE-7 │     │  0.78 ████░ │
└─────────────┘     └───────────────┘     └─────────────┘
                            │
                            ↓ (dim connection line)
                     ┌──────────────┐
                     │  VITALITY    │
                     │  0.72 ████░  │
                     └──────────────┘
```

**Center element:** the primary display object (Golem, globe, primary metric). Border: double-line (`╔╗╚╝`). Always has highest brightness.

**Orbital panels:** single-line borders, `rose_dim`. Connection lines: `─│` or `·` (dotted), `text_ghost` color.

**Panel content:** 2-3 rows. Label (top, `text_dim`), value (middle, `bone` or semantic), optional mini-gauge (bottom, 5 chars).

### B15.2 Animation

Center element pulses on tick (brief brightness increase). Orbital panels flash when their values change (FlashNumber behavior). Connection lines pulse simultaneously with the center's heartbeat — a radial pulse traveling center → panel.

### B15.3 Design Tokens

```yaml
orbital_panels:
  center_border: "double-line ╔╗╚╝"
  orbital_border: "single-line ┌┐└┘"
  connection_char: "─│·"
  connection_color: "text_ghost"
  orbital_positions: ["N", "E", "S", "W"]
  panel_size: "14-18w × 3-4h"
  pulse_radius: "center to panel, ~500ms travel"
  orbital_color: "rose_dim"
  center_color: "rose or bone"
```

---

## B16. Chromosomal Array

**Source:** EVA "4th ANGEL pattern:blue" karyotype display

Paired vertical columns of color-banded horizontal segments, like a genetic karyotype. Each column = a data stream (Grimoire category, risk domain). Bands within each column encode sub-values by color and width. Columns pair left/right, labeled p (upper arm) and q (lower arm). Green wire frame borders each column pair; binary readout ticker runs along bottom.

```
  p p  p p  p p  p p  p p
  │█│  │▓│  │░│  │█│  │▒│   ← top arm (p), each cell 1 row
  │▒│  │█│  │▓│  │░│  │█│
  ├─┤  ├─┤  ├─┤  ├─┤  ├─┤   ← centromere
  │░│  │▒│  │█│  │▓│  │░│
  │▓│  │░│  │▒│  │█│  │▓│   ← lower arm (q)
  q q  q q  q q  q q  q q
   1    2    3    4    5
```

Bands use the full semantic spectrum — rose/warning/success/dream mapped to value ranges. Background: `bg_void`. Column count: 6–12 pairs. Bands update on tick with a brief flash on change.

### B16.1 Design Tokens

```yaml
chromosomal_array:
  column_pairs: "6-12"
  arm_labels: "p (top) / q (bottom)"
  centromere_char: "─"
  band_chars: "█▓▒░"
  ticker_format: "binary string"
```

**Bardo use:** Grimoire domain coverage map; protocol capability distribution; clade member capability comparison.

---

## B17. Torus Radial Field

**Source:** EVA AT Field extended 3D display — diamond-silhouette with concentric shaded rings, depth from center

A diamond-oriented shape (rotated square) with concentric filled rings from a bright luminous center to a dim outer edge. Crosshair lines at N/S/E/W extend past the outer ring. Floating labels at cardinal points. The depth gradient encodes field strength.

```
              │  N  │
              ╱─────╲
     ─ ─ ─  ╱  ╱───╲ ╲  ─ ─ ─
    W       │  │  ● │  │       E
     ─ ─ ─  ╲  ╲───╱ ╱  ─ ─ ─
              ╲─────╱
              │  S  │

    Inner ring: rose_bright  →  Outer ring: text_ghost
```

Rings: 3–5 concentric. Characters: `╱╲─│` for diamond outline, `·` at ring intersections. Rings rotate at slightly different speeds — outer slower. Field contraction under stress collapses rings inward.

### B17.1 Design Tokens

```yaml
torus_radial:
  ring_count: "3-5"
  outer_color: "text_ghost"
  inner_color: "rose_bright"
  crosshair_extend: "4-6 cells past outer ring"
  coord_label_positions: ["N", "E", "S", "W"]
  rotation_differential: "outer ring slower than inner"
```

**Bardo use:** AT Field health display (Mind screen); stochastic risk field (FATE screen); golem "influence radius" in World view.

---

## B18. Interference Harmonics

**Source:** EVA Harmonics Simulation Graph Display — multiple intersecting sinusoidal waves

Multiple sinusoidal waveforms rendered in the same 2D space, each with different frequency and phase. Where waves meet in-phase: bright intersection point. Where they cancel: dark void. Horizontal axis with numeric labels; vertical amplitude axis on left.

```
┌──────────────────────────────────────────────────────────────┐
│+12                                                           │
│    ╭──╮    ╭──╮       ╭──╮       ── rose  (PAD:pleasure)    │
│   ╱    ╲  ╱    ╲     ╱    ╲                                  │
│──╱──────╲╱──────╲───╱──────╲──── ── warning (PAD:arousal)   │
│ ╱        ╲      ╱╲ ╱                                         │
│╱          ╲    ╱  ╲╱ ← intersection flash                    │
│            ╲──╱       ── dream (PAD:dominance)               │
│-12                                                           │
│ -8.0  -6.0  -4.0  -2.0  ±0  +2.0  +4.0  +6.0  +8.0 +10.0  │
└──────────────────────────────────────────────────────────────┘
```

Characters: `╭╮╯╰` for smooth curves where terminal supports them, else `▁▂▃▄▅▆▇█` per-column quantization. Intersection cells: `✦` or `*` in bone, brief flash. Three channels simultaneously: rose/warning/dream. Waves scroll left as new data arrives.

### B18.1 Design Tokens

```yaml
interference_harmonics:
  channel_count: 3
  channel_colors: ["rose", "warning", "dream"]
  intersection_char: "✦"
  intersection_color: "bone"
  axis_labels: "numeric, left + bottom"
  scroll: "left, newest at right"
```

**Bardo use:** PAD vector multi-signal correlation (Mind/Affect screen); market signal phase alignment; showing when strategy signals are in/out of sync.

---

## B19. Phase Butterfly

**Source:** EVA AT Field "REALTIME SENSOR DATA FEED" — two mirrored hourglass shapes, gradient-filled

Two mirrored distribution profiles facing each other across a vertical axis. Each profile's width at a given horizontal slice encodes the signal amplitude. Shapes touch or overlap at center. Gradient fill from dense (center) to sparse (outer edge).

```
┌──────────────────────────────────────────────────────────────┐
│  ⌈ SUBJECT-A ⌋              ⌈ SUBJECT-B ⌋                   │
│                                                              │
│              ██│██                                           │
│          ████████│████████                                   │
│      ████████████│████████████                               │
│  ████████████████│████████████████                           │
│      ████████████│████████████                               │
│          ████████│████████                                   │
│              ██│██                                           │
│                                                              │
│  DELTA: +0.23 ↑    PATTERN: CONVERGING                      │
└──────────────────────────────────────────────────────────────┘
```

Left half: rose gradient (`█▓▒░` from center). Right half: warning gradient (`█▓▒░` from center). Vertical divider: `│` in `text_dim`. Pattern label: CONVERGING / DIVERGING / LOCKED / DECOUPLING.

### B19.1 Design Tokens

```yaml
phase_butterfly:
  axis_char: "│"
  left_gradient: "rose █▓▒░"
  right_gradient: "warning █▓▒░"
  pattern_labels: ["CONVERGING", "DIVERGING", "LOCKED", "DECOUPLING"]
  delta_label: "bottom center"
```

**Bardo use:** Bid/ask pressure comparison; two-Golem AT field comparison; buy/sell velocity on FATE screen; any paired opposing metric.

---

## B20. Coordinate Targeting Grid (Extended)

**Source:** EVA Greek-letter crosshair grid

Extension of B2.3's crosshair grid. Each node now carries a coordinate label in Greek-letter notation: row letters (α, β, γ...) + column offsets (−02, −01, ±00, +01, +02...). A tick ruler runs along top and left edges.

```
┄┄┬┄┄┬┄┄┬┄┄┬┄┄┬┄┄┄  ← top tick ruler
   │   │   │   │
 + │ + │ + │ + │        ← crosshair nodes
α-2  α-1  α±0  α+1
   │   │   │   │
 + │ + │ + │ + │
β-2  β-1  β±0  β+1
   │   │   │   │
 + │ + │ + │ + │
γ-2  γ-1  γ±0  γ+1
```

Node chars: `+` in `text_dim`. Labels: `text_phantom`. Active/targeted node: `×` in `rose_bright` with label in rose. Ticker: `┄` chars along edges.

**Bardo use:** Oracle prediction coordinate space (FATE screen); 2D parameter grid for any decision space visualization; background for tactical displays.

---

## B21. Analog Meter Array

**Source:** EVA power substation control panel — dense grid of circular gauges with sweep needles

Multiple circular analog-style gauges arranged in a tight grid. Each gauge: a semicircular arc with scale markings, a needle/pip indicator at current value, a center numeric readout, and a unit+label below. Industrial, tactile — higher visual weight than bar gauges.

```
     ┌──────────────┐     ┌──────────────┐
     │   ┌──────┐   │     │   ┌──────┐   │
     │  ┌┘ ╱╱▲ └┐  │     │  ┌┘ ╱▲╲  └┐  │
     │  │╱╱ │  ╲│  │     │  │╱  │  ╲ │  │
     │  └────────┘  │     │  └────────┘  │
     │    30.8 KA   │     │   100.0 KV   │
     │  新御殿場第一│     │  接続電圧計  │
     │  ● NOMINAL   │     │  ⚠ ALERT     │
     └──────────────┘     └──────────────┘
```

Arc chars: `╱─╲` for scale, `▲` for needle pip, `│` for vertical center. Scale marks: `┬` every N units. Status badges: `● NOMINAL` / `⚠ ALERT`. Needle pip moves smoothly on tick; scale mark brightens at needle position.

### B21.1 Design Tokens

```yaml
analog_meter:
  arc_chars: "╱─╲"
  needle_char: "▲"
  scale_char: "┬"
  status_nominal: "● NOMINAL"
  status_alert: "⚠ ALERT"
  animation: "smooth needle movement per tick"
```

**Bardo use:** Primary status display on SOMA/Vitality — each mortality clock as a large analog meter. Higher-stakes tactile feel than bar displays.

---

## B22. Angled Spectrogram

**Source:** EVA voice recognition 3D column display

Isometric-perspective representation of a frequency spectrum: columns of varying height arranged across a time/frequency axis, viewed at ~30° showing depth. Columns colored by intensity from base (dim) to peak (bright). Simulated in terminal with half-block characters and a fixed vanishing point.

```
     ▁▂▃▄▅▆▇█ ▁▄▇█▇▄▁ ▁▂▄▇▄▂▁ ▁▁▂▃▂▁▁   ← front row
    ▁▂▃▄▅▆▇█ ▁▄▇█▇▄▁ ▁▂▄▇▄▂▁ ▁▁▂▃▂▁▁    ← middle row (offset)
   ▁▂▃▄▅▆▇█ ▁▄▇█▇▄▁ ▁▂▄▇▄▂▁ ▁▁▂▃▂▁▁     ← back row (offset)

  LOW ──────────────────────── HIGH (freq)
  PAST ←─────────────────── NOW (time)
```

Front row: `rose_bright`. Middle: `rose`. Back: `rose_dim`. Offset of 1 char per depth row creates the perspective illusion. Each column bucket: one char wide, height encoded in `▁▂▃▄▅▆▇█`. New columns push from right; old fall off left. Frequency labels below front edge.

### B22.1 Design Tokens

```yaml
angled_spectrogram:
  depth_rows: 3
  depth_colors: ["rose_bright", "rose", "rose_dim"]
  column_chars: "▁▂▃▄▅▆▇█"
  perspective_offset: "1 char per depth row"
  scroll: "right to left, time axis"
```

**Bardo use:** LLM inference token consumption rate across time (Mind/Inference screen); computation activity in audio-style view; event frequency heatmap.

---

## B23. Point Scatter with Bio-Trail

**Source:** EVA "DNA Analysis Bio Trail" display

A scatter of data points on a fine grid, with a continuous flowing trace line threading through or alongside them. Points = individual measurements; trace = extracted signal or moving average. The trace is drawn as a connected path, not a smooth curve.

```
┌──────────────────────────────────────────────────────────────┐
│ DNA ANALYSIS · BIO TRAIL               SAMPLE ░ DRONE A1    │
│                                                              │
│  ·      ·   ·      ·    ·   ·                                │
│    · ──·──·── ·──·──·──·──·──·  ← trace (rose)              │
│  ·   ·    ·      ·    ·    ·  ·                              │
│         ·    ·          · ·    ·                             │
│  ·  ·      ·      ·  ·       ·                               │
│ ──────────────────────────────────────────── time →          │
│  43.005871        73.002894        523.37                     │
└──────────────────────────────────────────────────────────────┘
```

Points: `·` in `rose_dim`. Trace: `─` with `·` nodes, rose. Outlier points (beyond 2σ): `×` in warning. Grid: fine `·` at every 4×2 cells in `text_phantom`. New data appends right, old data scrolls left. Numeric anchors below axis at key values.

### B23.1 Design Tokens

```yaml
point_scatter_bio_trail:
  point_char: "·"
  outlier_char: "×"
  outlier_color: "warning"
  trace_char: "─"
  trace_color: "rose"
  grid_spacing_h: 4
  grid_spacing_v: 2
  grid_color: "text_phantom"
```

**Bardo use:** Oracle prediction history — individual predictions as scatter, actual accuracy as bio-trail (FATE screen); market tick data with moving average overlay; epistemic confidence over time.

---

## B24. Multi-Scan Triptych

**Source:** EVA AT Field thermal scan 3-panel comparison

Three vertical panels displaying the same entity under three different scan modes or parameter layers. Each panel: a mode label at top, crosshair annotation markers (`+`) at key points, spectrum-mode color fill. Panels share equal width and a single bottom legend.

```
┌────────────────┬────────────────┬────────────────┐
│ THERMAL        │ STRUCTURAL     │ AT FIELD       │
│  AT FIELD      │  PATTERN       │  CONCEPTUAL    │
│  Pattern:Blue  │  Blood:A       │  Variation:03  │
│                │                │                │
│  ░░░▒▒▓▓██▓▓  │  ·┼·┼·┼·┼·    │  ·  ·  ·  ·   │
│  ░▒▓████████▓  │  ┼···┼···┼    │   ╱─────╲      │
│  ▒▓██████████  │  ·┼·┼·┼·┼·   │  ╱  ░▒▓██╲     │
│  ░▒▓████████▓  │  + ←target    │  ╲  ░▒▓██╱     │
│  ░░░▒▒▓▓██▓▓  │                │   ╲─────╱      │
│     + ←scan   │                │                │
└────────────────┴────────────────┴────────────────┘
  OBJECT: GOLEM-ALPHA   ANALYSIS: TACTICAL   VAR: 03
```

Panel dividers: `│` in border. Each panel uses a different color mode: thermal (rose heatmap), structural (crosshair/grid), field (wireframe/radial). Label row at bottom. Scan-mode panels refresh independently on their own tick rate.

### B24.1 Design Tokens

```yaml
multi_scan_triptych:
  panel_count: 3
  panel_divider: "│"
  modes: ["thermal", "structural", "field"]
  label_position: "top of each panel + bottom legend"
  independent_refresh: true
```

**Bardo use:** Golem health inspection across three dimensions (Economic / Epistemic / Stochastic) simultaneously; protocol comparison side by side; any 3-way parallel analysis.

---

## B25. Penetration Wedge

**Source:** NERV HQ layer-breach visualization — perspective side view of building floors being penetrated by a glowing cone

Animated side-view of the B7 defense stack when a breach is active. Layers shown in perspective; a glowing wedge/front advances through them as defenses fail. The wedge moves inward over time. Countdown timers at corners.

```
⌈ PENETRATION ACTIVE — LAYER 04 ⌋                 00:05:21:00

  Lv07 ████████████████████████████████████████  完成
  Lv06 ████████████████████████████████████████  完成
  Lv05 ████████████████████████████████████████  完成
  Lv04 ██████████████████▓▒░░░░░░░░░░  ← BREACH FRONT
  Lv03 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  ⚠
  Lv02 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
  Lv01 ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  CORE

  ESTIMATED TIME OF BREACH: 16:01:32:00
  ESTIMATED TIME REMAINING:  08:05:21:00
```

The breach front (`▓▒░`) advances left-to-right within the breached layer on each tick. Breached layers: `rose_bright` → `text_ghost`. Intact layers: rose filled. The active layer oscillates brightness. Japanese layer status labels (`完成` = intact/complete) in `text_dim`.

### B25.1 Design Tokens

```yaml
penetration_wedge:
  breach_front_chars: "▓▒░"
  intact_chars: "█"
  depleted_chars: "░"
  front_animation: "advance left to right within active layer"
  countdown_positions: ["top-right", "bottom-right"]
  layer_status_label: "right edge"
```

**Bardo use:** Active crisis display for CONDITION CRITICAL when a specific defense layer is being depleted; more visceral than the static B7 bars.

---

## B26. Power Distribution Schematic

**Source:** EVA Shinyogawa substation power transfer schematic

A node-link schematic showing infrastructure topology: cylindrical/box nodes connected by lines in a tree structure. Floating labels at each node. Grid background. Box-drawing characters suggest 3D perspective.

```
            ┌──────────────────────────────┐
            │ ⌈ STAGE 2 POWER SCHEMATIC ⌋  │
            └──────────────────────────────┘
                    ╔═══╗   ╔═══╗
                    ║CB ║   ║CB ║    ← circuit breaker nodes
                    ╚═╤═╝   ╚═╤═╝
                      │  BUS  │
              ╔═══════╧═══════╧═══════╗
              ║    PRIMARY BUS BAR    ║
              ╚══╤═════════════╤══════╝
                 │             │
              ╔══╧══╗       ╔══╧══╗
              ║DS-01║       ║DS-02║   ← disconnect switch
              ╚══╤══╝       ╚══╤══╝
                 │             │
              ╔══╧══╗       ╔══╧══╗
              ║GOLEM║       ║GOLEM║   ← endpoints
              ║ALPHA║       ║BETA ║
              ╚═════╝       ╚═════╝
```

Nodes: double-line borders. Connection lines: `─│╤╧╠╣` box-drawing. Labels float adjacent to nodes in `text_dim`. Grid background: sparse `·` at regular intervals. Active connections pulse — brief brightness traveling node→node on tick.

### B26.1 Design Tokens

```yaml
power_distribution:
  node_border: "double-line ╔╗╚╝"
  connection_chars: "─│╤╧"
  label_color: "text_dim"
  pulse_direction: "source node → endpoint"
  grid_bg: "sparse · text_phantom"
```

**Bardo use:** Clade dependency topology (World screen); showing which Golems share data sources or infrastructure; service dependency map.

---

## B27. Dead Zone Tessellation

**Source:** Lain-aesthetic diamond tessellation with corruption text

A full-panel fill pattern using alternating filled/open diamond characters (`◆◇`) in a dense checkerboard, with a message bleeding through in a horizontal band. Used for panels that cannot load data — the "inaccessible space" pattern. Text corrupts the pattern visually where it appears.

```
◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆
◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇
◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆
◇◆◇◆◇◆   Turn back, this is a [DEAD END].   ◆◇◆◇◆◇
◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆
◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇◆◇
```

Foreground: `rose_deep` on `bg_void`. Text band: `bg_void` on `rose_deep` (inverted). Text: italic, letter-spaced. Message rotates through: `SIGNAL LOST` / `DEAD ZONE` / `NO DATA` / `FEED OFFLINE`. Text band scrolls vertically through the tessellation on a slow cycle.

### B27.1 Design Tokens

```yaml
dead_zone_tessellation:
  chars: "◆◇"
  pattern: "checkerboard"
  fg: "rose_deep"
  bg: "bg_void"
  text_band_invert: true
  messages: ["SIGNAL LOST", "DEAD ZONE", "NO DATA", "FEED OFFLINE"]
  text_band_scroll: "vertical, slow"
```

**Bardo use:** Any panel with no data feed; offline/disconnected Clade member display; uninitialized tab placeholder; replaces empty whitespace with intentional design.

---

## Composability Notes (B16–B27)

These primitives combine with existing and each other:

- **Torus Radial (B17) + Corner Elements (B14):** AT Field display with vital metrics in corners
- **Interference Harmonics (B18) + Waveform (B3):** Harmonics showing signal correlations, B3 below for raw traces
- **Point Scatter Bio-Trail (B23) + Timeline Ribbon:** Scatter on primary view, timeline below showing event annotations
- **Chromosomal Array (B16) + Unit Array (B1):** Chromosome view for domain overview, expand a single chromosome to a unit array for drill-down
- **Multi-Scan Triptych (B24) + Orbital Panels (B15):** Triptych as center element, orbital panels with aggregate metrics
- **Dead Zone Tessellation (B27) + Pilot Vanished (B6.3):** Tessellation fills the panel body; Vanished stamp overlays center

---

*⌈ the institutional machinery processes an apocalypse through bureaucratic displays. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*

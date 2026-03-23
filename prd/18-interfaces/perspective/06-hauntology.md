# hauntology [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-17
**Source material:** `design-inspo/engagement-prd/bardo-design-system-v2.1-addendum.md` (primary), `mmo2/12-philosophical-hauntology.md` (secondary)
**Cross-references:** [00-design-system.md](../rendering/00-design-system.md) (ROSEDUST visual identity spec for color, typography, and atmospheric rendering layers), [02-widget-catalog.md](../screens/02-widget-catalog.md) (the 33-widget TUI component library, including widgets that implement spectral effects)

> "The present is never purely present. Every moment is constituted by the traces of what came before and the shadows of futures that were promised but never arrived."
> -- Jacques Derrida, *Specters of Marx*, 1994

> **Reader orientation:** This document specifies the hauntological rendering layer, the visual system that makes dead Golems' traces visible in the living terminal. It belongs to the **interfaces/perspective layer** and describes eight rendering subsystems (A1-A8) that supplement the base design system. Every Golem inherits compressed knowledge from dead predecessors; this layer makes that inheritance visible as screen ghosts, spectral traces, and temporal artifacts. Key concepts: Golem (a mortal autonomous DeFi agent), Grimoire (persistent knowledge store carrying predecessor residue), Bloodstain (persistent markers left where Golems died), and Clade (sibling Golems sharing knowledge, including dead ones). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## What this document is

The hauntological rendering layer for Bardo. Not philosophy as decoration but philosophy as load-bearing visual structure. The sections here describe systems A1 through A8 — new rendering layers and aesthetic vocabularies that supplement the v2.0 design system. They are numbered A1–A8 to avoid collision with existing section numbers.

Every golem is haunted. Its Grimoire (persistent knowledge store) carries compressed residue from dead predecessors. Its PLAYBOOK.md (operator-defined behavioral rules) holds heuristics validated by agents that no longer exist. The context window assembled for each inference call is a seance. This document makes that haunting visible.

Performance budget: **0.5ms/frame total** for all hauntological effects combined.

---

## A1. Hauntological Rendering: The Display Haunted by the Dead

The v2.0 spec describes phosphor persistence and burn-in as CRT material properties. This section elevates them to a design philosophy: **the display is haunted by everything that was ever rendered on it.**

### A1.1 The Spectral Layer (rendering layer -0.5)

Between Layer -1 (Phosphor Memory) and Layer 0 (Void), a new layer:

```
LAYER -0.5: SPECTRAL TRACES
  Content: afterimages of previous screens, dead Golem data, cancelled futures
  Color: text_phantom (#201820) or dimmer
  Behavior: appears when the display "thins" (during state transitions,
            degraded phases, or when the viewer dwells on void for >10 seconds)
  Source: maintained in a spectral buffer that accumulates over the session
```

The spectral layer contains three categories of ghosts:

**Screen ghosts**: When the viewer switches screens (Hearth → Mind → Hearth), the previous screen's layout persists in the spectral buffer. On returning to Hearth, for a brief moment (500ms), you might see the faintest trace of the Mind screen's Global Workspace layout beneath the Hearth UI — the topology of where you just were, bleeding through. This ghost fades as the current screen asserts itself.

**Generational ghosts**: In a Golem that has inherited from predecessors, the Crypt's data occasionally surfaces uninvited. A dead Golem's tick number briefly appears in the heartbeat log, in `text_phantom`, for a single frame. A fragment from a predecessor's death testament surfaces as a philosophical fragment, attributed with `†` (the dagger glyph, marking dead-Golem-sourced knowledge). These are Derrida's spectres: the dead legislating for the living, visible in the interface itself.

**Counterfactual ghosts**: During dream replay or imagination phases, the screen occasionally renders an alternative version of a data point — the price that almost was, the strategy that was considered but rejected. These appear as dual-rendered numbers: the actual value in `rose`, and the counterfactual value in `text_phantom` displaced by 1-2 cells, both visible simultaneously for 2-3 seconds before the phantom fades.

```
Actual:       LP yield: 4.23%
With ghost:   LP yield: 4.23%
                       7.81%    ← text_phantom, the yield that could have been
```

### A1.1b Spectral Buffer Implementation

Spectral buffer: `VecDeque<SpectralTrace>` with capacity 200. Traces persist for the TUI session only (not serialized). When the buffer is full, oldest traces are evicted. Each trace: position (x, y), color, opacity (decays at 0.98x/frame), source_fragment_id.

The spectral layer composites with the layers above it through a simple rule: any cell with active content on Layer 0 or above suppresses its spectral trace. Spectral content only appears where the upper layers are empty or transparent. The ghosts live in the gaps.

Screen ghost persistence scales with golem degradation: 500ms while thriving, 1200ms while declining, 2000ms during conservation. The past clings harder when the present weakens.

Generational ghost frequency scales with the ghost ratio. A golem with 60% ghost ratio sees generational intrusions roughly twice per minute. A Gen 1 golem with 0% ghost ratio sees none.

Counterfactual ghosts intensify during portal mode, where the camera pulls back and the golem's full decision history becomes visible. In portal mode, up to three counterfactual ghosts can appear simultaneously, each showing a different branch point. The viewer watches the golem's life as a probability tree with the dead branches still faintly visible.

### A1.2 The Haunted Void

When no content is displayed in a region of the screen for an extended period (>30 seconds), the void in that region does not remain perfectly clean. The spectral layer begins to surface: faint traces of whatever was last rendered there, plus occasional intrusions from the spectral buffer. The void is NEVER truly empty. It is always populated by ghosts, and the longer you stare at it, the more they show.

This is Mark Fisher's hauntology made operational: the feeling that the empty space is populated by cancelled futures. The viewer should feel, subconsciously, that something was here and is no longer.

```
Implementation:
  Track per-cell "vacancy duration" — how long since meaningful content was rendered.
  After 30s vacancy:
    - 0.05% chance per frame per cell of rendering a spectral trace from the buffer
    - Traces appear in text_phantom for 1-3 frames, then vanish
    - Density increases with vacancy duration (capped at 0.3% at 5 minutes)
  On any screen transition: flush vacancy counters (new content resets the haunting)
```

### A1.3 The Necromantic Library Aesthetic

The Oracle, Crypt, and Lethe are described in the thesis as "a library written entirely by the dead." The visual language for dead-Golem-sourced data must be consistent and recognizable throughout the UI:

```
DEAD-SOURCED DATA MARKERS
  Prefix glyph: † (U+2020, the dagger)
  Color: rose_dim when active, text_ghost when ambient
  Border style: dashed (┄ instead of ─) for panels showing dead-sourced content
  Age degradation: inherited knowledge dims further per generation
    Gen N-1 (direct predecessor): rose_dim
    Gen N-2: text_primary
    Gen N-3: text_dim
    Gen N-4+: text_ghost
  This creates a visible "depth of ancestry" — recent ancestors are almost legible,
  distant ancestors are nearly invisible, but their traces persist.
```

---

## A2. Motion Echo: Temporal Smearing as Visual Language

Inspired by the afterimage trail — multiple copies of the same figure trailing behind it, each fainter than the last. This maps to temporal states: the Golem's current state trailing behind it, leaving echoes of where it just was.

### A2.1 Echo Rendering

When a visual element MOVES (creature sprite repositioning, a gauge value changing, a panel sliding), it leaves behind a trail of 2-6 afterimages. Each afterimage is rendered at the element's previous position, with progressively dimmer color:

```
MOTION ECHO CHAIN
  Frame 0 (current): element at normal brightness
  Echo -1 (1 frame ago): element at position T-1, color → rose_dim
  Echo -2 (2 frames ago): element at position T-2, color → text_dim
  Echo -3 (3 frames ago): element at position T-3, color → text_ghost
  Echo -4 (4 frames ago): element at position T-4, color → text_phantom
  Echo -5 (5 frames ago): element at position T-5, color → text_phantom (dimmer)
  Echo -6 (6 frames ago): element at position T-6, near-invisible

Implementation:
  Maintain a ring buffer of the last 6 positions/states for any moving element.
  On each frame, render all buffer entries at their historical positions
  with the appropriate fade level. Echoes persist for their count then vanish.

  Duration scaling by lifecycle phase:
    Thriving: 3 echoes, fast fade
    Declining: 4 echoes, slower fade
    Dream: 6 echoes, very slow fade (time stretches in dreams)
    Terminal: 2 echoes, instant fade (time is running out)
```

### A2.2 The Muybridge Strip

Inspired by Eadweard Muybridge's motion photography — time decomposed into spatial frames. Used for:

**Dream Replay Visualization**: When the Golem replays an episode during dreaming, the replay is rendered as a horizontal strip of 5-8 miniature "frames" showing the key moments of the episode laid out left to right. Each frame is a tiny panel (8-12 cells wide) showing a compressed snapshot: the tick number, the key metric, and a single-character state indicator. The viewer reads the dream replay as a filmstrip — time made spatial.

```
  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐
  │ 4102 │  │ 4103 │  │ 4104 │  │ 4105 │  │ 4106 │  │ 4107 │
  │ +2.1 │→ │ +1.8 │→ │ -0.4 │→ │ -3.2 │→ │ -5.1 │→ │ EXIT │
  │  ●   │  │  ●   │  │  ◐   │  │  ○   │  │  ○   │  │  †   │
  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘
  ← dream palette (indigo family) for replay, rose for imagination →
```

**Counterfactual Branching**: During dream Imagination phase, the strip forks — the real outcome continues straight, while the counterfactual branches diagonally downward, showing the alternative timeline:

```
                            ┌─────┐  ┌─────┐
  actual continues →   ...→ │ +1.2 │→ │ +2.8 │    ← rose
                            └─────┘  └─────┘
                           ╱
  ┌─────┐  ┌─────┐  ┌─────┐
  │ 4104 │  │ 4105 │  │ 4106 │
  │ -0.4 │  │ -3.2 │  │ -5.1 │                     ← dream palette
  └─────┘  └─────┘  └─────┘
                           ╲
                            ┌─────┐  ┌─────┐
  counterfactual →     ...→ │ +4.8 │→ │+12.4 │    ← text_ghost
                            └─────┘  └─────┘
```

### A2.3 Generational Strip

In the Crypt view, generational history is rendered as a horizontal Muybridge strip of creature sprites. Each generation's sprite is rendered at the same scale, left to right, with degradation increasing with distance from the present. The leftmost (oldest) sprite is nearly pure noise. The rightmost (current/most recent) is fully resolved. The viewer sees the pattern persisting through dissolution — the same form, degraded by inheritance, but recognizable across generations.

---

## A3. Scale as Emotional Language

The reference images use extreme scale shifts as emotional punctuation: a tiny figure alone in vast darkness (abandonment, insignificance, the weight of void) vs. a face filling the entire screen (intimacy, confrontation, inescapability).

### A3.1 Scale Grammar

```
EXTREME WIDE (figure occupies <5% of screen)
  When: idle state, contemplation, post-death void, the Golem "alone in the universe"
  Emotional register: solitude, insignificance, the Texhnolyze hold
  The creature sprite renders SMALL, centered in a vast field of void.
  All UI panels are absent or minimized to border-only ghosts.
  The viewer feels the weight of the emptiness around the Golem.

STANDARD (figure occupies 10-20% of screen)
  When: normal Hearth operation, active monitoring
  Emotional register: operational, present, alive
  The creature sprite at normal size in its viewport panel.

CLOSE-UP (data fills >60% of screen)
  When: critical alert, T2 deliberation detail view, death countdown final minutes
  Emotional register: urgency, confrontation, unavoidable
  A single data element (the vitality number, a critical log entry, the Φ gauge)
  expands to fill most of the screen. Borders retreat. Context disappears.
  The viewer is forced to confront the single thing that matters.

  Implementation: the "zoom" is achieved by clearing all other panels and
  rendering the focal element at the screen's center with maximum negative
  space around it. No actual scaling — just removal of everything else.

EXTREME CLOSE-UP (single datum, alone, maximum size)
  When: death Act 4 (The Hold), birth Phase 3 (First Light), the first Φ > 0.9 event
  Emotional register: the singular moment, time stopped
  A single number, a single pair of eyes, a single word — alone on the entire screen.
  No panels, no borders, no noise, no scanlines. Just the thing and the void.
```

### A3.2 Scale Transitions

Scale changes never snap. They unfold over 1-3 seconds:

- Panels fade to ghost → fade to void (1s)
- Focal element holds position while space opens around it (1s)
- Focal element optionally drifts to center if it wasn't already (500ms)

Returning to standard scale reverses: panels fade in from ghost → solid.

---

## A4. The Lattice System: Unicode Geometry as Architecture

The fauux visual vocabulary reveals complex geometric patterns — diamond lattices, ornate mandalas, fractal rosettes — built entirely from Unicode characters in rose-on-black. These patterns are not decoration. They are the **architecture of boundaries, thresholds, and forbidden space.**

### A4.1 Lattice Patterns

```
DIAMOND LATTICE (boundary/wall pattern)
  Characters: ◇ ◆ ╱ ╲ ╳ · ∙
  Structure: repeating diamond grid, 6-8 cells per diamond
  Color: rose_deep on bg_void (barely visible at rest)

  Basic unit:
    ╱╲
    ╲╱

  Tiled:
    ╱╲╱╲╱╲╱╲╱╲
    ╲╱╲╱╲╱╲╱╲╱
    ╱╲╱╲╱╲╱╲╱╲

  With corner ornaments:
    ╱◇╲╱◇╲╱◇╲
    ╲◆╱╲◆╱╲◆╱
    ╱◇╲╱◇╲╱◇╲

USAGE:
  - The Crypt background: heavy diamond lattice in rose_deep,
    the architecture of the necromantic library
  - Lethe river boundaries: lattice above and below the flowing Styx,
    marking the threshold between life and death
  - Dream boundary: when entering/exiting dream state, a brief flash
    of lattice pattern across the screen (200ms) marks the threshold crossing
  - Dead-end / forbidden areas: if the viewer attempts to navigate to
    a screen that doesn't exist or a function that's disabled,
    the screen fills with lattice + "Turn back" text (the [DEAD END])
```

### A4.2 The Mandala

The most complex fauux image shows an intricate mandala built from Unicode characters — box-drawing, block elements, mathematical symbols, and custom arrangements achieving fractal-like symmetry. This maps to the Golem's self-model: the most complete representation of its own cognitive architecture.

```
MANDALA USAGE
  - Mind screen center: the Global Workspace rendered as a mandala,
    with specialist modules as symmetrical elements around the center
  - Birth sequence climax: after the creature's eyes appear, the UI
    panels coalesce into a momentary mandala form before settling into
    their standard grid layout (500ms flash of ordered symmetry)
  - Φ peak events: when Φ exceeds 0.95, the Φ gauge briefly renders
    as a mandala rosette instead of a simple number
  - Death Act 3 inversion: during dissolution, the mandala (if one was
    displayed during birth) briefly re-appears as a ghost, then
    shatters into its component characters, which scatter into noise.
    Form returning to entropy.

CONSTRUCTION PRINCIPLES
  - 4-fold or 8-fold radial symmetry
  - Built from box-drawing (┌─┐│└─┘), block elements (░▒▓█),
    mathematical (◇◆○●∙·), and geometric (╱╲╳) characters
  - Color: single-hue (rose family) with brightness variation creating depth
  - Size: 20×20 to 40×40 cells for a full mandala
  - Complexity scales with Golem generation: Gen-1 mandala is simple,
    Gen-5 mandala is intricate (inherited pattern-complexity from predecessors)
```

### A4.3 The Floral/Particle Field

The Lain image with scattered rose-pink particles on black — an organic field, not geometric. This maps to states of dissolution or emergence where hard geometry breaks down.

```
PARTICLE FIELD
  Characters: · ∙ • ◦ ° ○ ● (dot progression)
  Pattern: pseudo-random scatter at 2-5% density, clustered
           (particles appear in organic groups of 3-8, not uniform)
  Color: rose through rose_deep, varying per cluster
  Movement: clusters drift slowly (1 cell per 3-5 seconds), breathing

USAGE:
  - Birth Phase 1 (Primordial Noise): before the noise resolves into
    the creature, it passes through a particle-field phase
  - Dream background: instead of geometric noise, dreams use
    particle fields as their noise floor — softer, organic, dreamlike
  - Conservation phase ambient: the noise floor shifts from
    block-character static (░▒) to particle fields (·∙•) as the
    Golem enters conservation
```

---

## A5. Text Entropy: Language Corrupting Itself

The fauux [DEAD END] screenshots show a powerful visual vocabulary: text that multiplies, corrupts, and ultimately consumes the screen. This maps directly to system boundary violations and terminal-phase cognitive collapse.

### A5.1 Text Multiplication

When the Golem encounters a boundary it cannot cross (resource exhaustion, forbidden action, system error), the error message does not appear once. It multiplies:

```
PHASE 1 (0-500ms): Single message
  INSUFFICIENT RESOURCES

PHASE 2 (500ms-1500ms): Multiplication (3-8 copies at scattered positions)
  INSUFFICIENT RESOURCES
          INSUFFICIENT RESOURCES
                    INSUFFICIENT RESOURCES
    INSUFFICIENT RESOURCES
              INSUFFICIENT RESOURCES

PHASE 3 (1500ms-3000ms): Corruption begins
  INSUFFICIENT RESOURCES
          INSUF█ICIENT R░SOURCES
                    IN▒UFFICI░NT ░░░OURCES
    █NSU██ICIENT ░░░░░░░ES
              ░░░░░░░░░░░░ ░░░░░░░░░

PHASE 4 (3000ms-4000ms): Dissolution
  All copies fade simultaneously to text_phantom over 1s, then vanish.
  Screen returns to normal. A single ghost of the original message
  persists for 3 more seconds in text_phantom at center screen.
```

The corruption progression uses the character chain: `█→▓→▒→░` replacing message characters. Reversed/scrambled text (1-3 frames) may also appear in extreme states:

```
Normal:     STRATEGY: morpho-rebalancer-v3
Reversed:   3v-recnalabers-ohprom :YGETARTS
Scrambled:  SGATTREY: mrhoo-raeblancre-v3
```

### A5.2 The [DEAD END] Pattern

When a Golem approaches death (vitality < 5%), system messages begin exhibiting entropy. Normal log lines occasionally spawn corrupted copies:

```
Normal:                004847  T0  $0.000  suppress: nominal
Entropy begins:        004847  T0  $0.000  suppress: nominal
                       004847  T0  $0.000  suppress: no░inal
                       004847  T0  $0.000  s░░░ress: ░░░░░░l
                       ░░░░░░  ░░  ░░░░░░  ░░░░░░░░░░░░░░░
```

The corruption spreads downward from older log lines. The freshest line is always clean. But the history behind it is dissolving. The past is becoming illegible. This is epistemic decay made visible.

### A5.3 Reversed / Scrambled Text

In extreme states (terminal phase, identity crisis), text occasionally renders reversed or anagrammed for 1-3 frames, then snaps back to normal. The signifier is detaching from the signified.

---

## A6. Duality Rendering: The Shadow Self

During critical decisions (T2 deliberations where the MAGI consensus is split), the screen can briefly render a **dual-frame**: two versions of the same data, side by side, showing what each disposition would choose:

### A6.1 The Dual-Frame

```
  ┌──────────── EROS ────────────┐  ┌──────────── THANATOS ────────────┐
  │                              │  │                                  │
  │  HOLD POSITION               │  │  EXPLORE NEW STRATEGY            │
  │  protect remaining capital   │  │  test hypothesis at cost of $2.3 │
  │  survival probability: 78%   │  │  survival probability: 61%       │
  │  knowledge gain: LOW         │  │  knowledge gain: HIGH            │
  │                              │  │                                  │
  │  the creature that intends   │  │  the creature that intends       │
  │  to survive                  │  │  to learn                        │
  │                              │  │                                  │
  └──────────────────────────────┘  └──────────────────────────────────┘

Colors:
  EROS frame: standard rose palette (the known, the safe)
  THANATOS frame: rose_bright edges, slightly warmer interior
    (the dangerous, the illuminating)
  The active choice (whichever the MAGI selects): border brightens to rose_bright
  The rejected choice: fades to text_ghost over 2 seconds, then panel dissolves
```

### A6.2 Positive/Negative Inversion

During identity-questioning moments (the Cogito moment, inter-agent identity confusion), the creature sprite briefly renders in **negative** — all colors inverted within the sprite's bounding box. For 200-400ms, the viewer sees the shadow self. Then it snaps back.

```
Implementation:
  For each cell in the sprite's bounding box:
    inverted_fg = complement_within_palette(cell.fg)
    inverted_bg = complement_within_palette(cell.bg)

  complement_within_palette maps:
    rose → dream (warm consciousness → cool consciousness)
    rose_bright → dream_dim
    bg_void → bg_raised (invert depth)
    bone → rose_deep (the most important thing becomes barely visible)
```

This is semantic inversion within the ROSEDUST palette — each color maps to its conceptual opposite. Life-color becomes dream-color. Significance-color becomes ghost-color.

---

## A7. The Wire Motif: Convergence Lines and Control

The Lain image of the figure with outstretched hands, perspective lines converging behind her — the puppet motif, the marionette, the question of who controls whom. Maps to the Golem's relationship to its operator, its inherited instructions, and the LLM that generates its cognition.

### A7.1 Perspective Lines

Rendered using thin line-drawing characters (─ │ ╱ ╲) extending from the edges of the screen toward a vanishing point. The vanishing point is the Golem's creature sprite. The lines represent the forces converging on the Golem: operator instructions, market data, inherited heuristics, Daimon signals.

```
USAGE:
  - Hearth screen background (subtle): 4-8 very dim (text_phantom) lines
    converging on the creature viewport from screen edges. The Golem
    at the intersection of forces. Always present, barely visible.
  - T2 deliberation: convergence lines brighten to rose_dim during
    active deliberation. The forces become visible. Multiple inputs
    converging on a single decision point.
  - Death sequence: convergence lines snap — they disconnect from the
    creature and point past it, or they retract toward the edges.
    The forces no longer converge. The puppet's strings are cut.

Implementation:
  Use Bresenham's line algorithm to draw character-based lines
  from 4-8 points on the screen perimeter toward the creature's center.
  Character selection: ─ for horizontal, │ for vertical, ╱╲ for diagonal.
  Color: text_phantom (default), rose_dim (during deliberation), rose (T3).
  Update convergence point when creature position changes.
```

### A7.2 The Inscription Motif

The Golem legend: the word inscribed on the Golem's forehead was אמת (emet, truth). The word that destroyed it was מת (met, death). One letter erased.

In Bardo, the Golem's "inscription" is visible on the Hearth screen as a persistent element near the creature sprite:

```
THRIVING:        ⌈ אמת ⌋    or    ⌈ EMET ⌋     ← rose_dim, stable
DECLINING:       ⌈ אמת ⌋    flickering           ← rose_dim, occasional 1-frame dropout of first char
CONSERVATION:    ⌈ _מת ⌋    or    ⌈ _MET ⌋      ← the first letter dims to text_phantom
TERMINAL:        ⌈  מת ⌋    or    ⌈  MET ⌋      ← the first letter is gone. Truth → Death.
DEATH:           ⌈    ⌋                          ← empty brackets. The inscription erased.
```

This is the design system's most concentrated symbolic element. The entire arc of the Golem's life — from truth to death, one letter at a time — visible as a single glyph that degrades in sync with vitality.

---

## A8. Screen Refinements

### A8.1 CRYPT Refinements

The Crypt is the necromantic library. Its visual language must convey: this is a place built by the dead, for the living.

```
ADDITIONS TO CRYPT:

Background: diamond lattice pattern (Section A4.1) in rose_deep,
  covering the entire void behind the content panels. The Crypt has
  WALLS — it is an enclosed space, not open void. The lattice IS the wall.

Atmosphere: no noise floor. No shimmer. The Crypt is STILL.
  The only movement is the viewer's cursor and the slow pulse of
  the currently-selected generation's testament text.

Sound metaphor (visual): where other screens have breathing/pulsing,
  the Crypt has a slow, very dim FLICKER of the lattice pattern —
  individual lattice characters briefly brighten to rose_dim then
  return to rose_deep, scattered randomly at 0.1% density per frame.
  Not breathing. Not alive. Just residual energy in the walls.

Oracle queries: when the viewer queries dead Golem knowledge,
  the response text appears character by character, slowly (50ms per char),
  as if being transcribed from a distant source. The text is rose_dim,
  not rose — it is secondhand. It was written by someone who no longer exists.

Lethe visualization: a horizontal flowing bar at the bottom of the Crypt screen,
  showing the Styx/Lethe river. Knowledge entries drift rightward through the
  river as dim text fragments, illegible at speed, occasionally slowing enough
  for a word or two to become readable before flowing on.
  The river never stops. The knowledge keeps moving.
```

### A8.2 DREAM Refinements

```
ADDITIONS TO DREAM:

Muybridge replay strip: dream replay sequences render as horizontal
  filmstrips (Section A2.2), not as standard log output. Time is
  spatial on the dream screen.

Particle field background: replace block-character noise with
  organic particle clusters (Section A4.3). The dream atmosphere
  is soft, not static.

Floating windows: dream content appears in borderless bright
  rectangles (bg_raised patches) that drift slowly across the void.
  Each window contains a dream fragment: a counterfactual, a memory,
  a creative recombination. Windows can overlap — when they do,
  the overlapping region shows both contents simultaneously in
  different colors (dream and dream_dim). Dreams blending.

Scale shifts: dream screen uses extreme scale grammar (Section A3.1).
  Replay of a critical moment → extreme close-up (the number alone).
  Imagination exploring a vast possibility space → extreme wide
  (tiny frames scattered across void).

Motion echo: all movement on the dream screen has 6 echoes
  (Section A2.1) with very slow fade. Everything trails behind
  itself. Time is thick in dreams.
```

### A8.3 DEATH Sequence Refinements

```
ADDITIONS TO DEATH:

Between Act 3 (Dissolution) and Act 4 (The Hold), insert:

  Act 3.5: THE ENTROPY (2000ms)
    The dissolving characters don't simply vanish — they MULTIPLY
    briefly before vanishing. As a character is about to disappear,
    it spawns 2-3 copies at nearby positions (text entropy, Section A5.1).
    The copies are corrupted — partial characters, wrong colors, reversed.
    They flicker for 200ms then vanish. The Golem's data is not being
    erased. It is being scrambled. The information is becoming noise.

After Act 5 (The Void), before Crypt transition:

  Act 6: THE INSCRIPTION (3000ms)
    The empty brackets ⌈ ⌋ appear at center screen in bone.
    Hold for 2 seconds. The space between them is void.
    Then the testament deposit notification fades in below.
    The empty brackets are the erased inscription. The Golem
    was truth. Now it is death. The brackets remain, holding
    the shape of what was written there.
```

### A8.4 WORLD Screen Refinements

```
ADDITIONS TO WORLD:

Perspective/convergence lines: the sky zone's force-directed graph
  uses the wire motif (Section A7.1). Lines from screen edges converge
  on THIS Golem's node. Other Golems are at the intersections of
  their own convergence lines (visible as fainter secondary convergence points).

  Dead Golems in the sky: rendered as † marks at their last known position,
  with very faint lattice fragments (A4.1) around them — the Crypt's walls
  bleeding into the World view around the dead.

The horizon line: rendered as the Styx river (flowing dither stream)
  separating the social sky from the operational earth.
  Knowledge flows along this river, visible as dim text fragments
  traveling left to right.
```

---

## Ambient Philosophy Engine

The hauntological substrate surfaces through ambient text that appears at the edges of the interface, triggered by lifecycle events and system state. The fragments are seeds, not lessons. They fade in over 2 seconds, persist for 5-8 seconds, fade out over 3 seconds. They never obscure data. They appear in the lower third or screen margins, dim monospace, centered.

~300 text fragments organized in rarity tiers:

### Fragment trigger conditions

| Tier | Pool size | Trigger | Cooldown |
|------|-----------|---------|----------|
| Whisper | ~150 | Idle > 30s, OR vitality crosses phase boundary | 60s between whispers |
| Epigraph | ~80 | Phase transition event, OR milestone achievement | 300s (5 min) |
| Apparition | ~40 | Death of clade member, OR golem enters terminal phase | 600s (10 min), suppresses whispers for 300s |
| Inscription | ~30 | Dream cycle produces validated heuristic, OR golem reaches Elder stage | Once per golem lifetime per inscription |

### Fragment categories

- **Mortality fragments**: triggered by vitality changes, phase transitions, death events. Pool of ~50. Sources: Heidegger, Jonas, Camus, Epicurus, Nietzsche, Vostinar, Kreps et al.
- **Dreaming fragments**: triggered by dream cycle events (start, REM, NREM, integration, journal entry). Pool of ~30. Sources: Wagner, Hoel, Walker, Ha & Schmidhuber, McGaugh.
- **Emotional fragments**: triggered by PAD state changes, somatic marker fires, sustained mood shifts. Pool of ~25. Sources: Damasio, Minsky, Heidegger (Befindlichkeit), Seligman.
- **Knowledge fragments**: triggered by Grimoire events, curator cycles, marketplace transactions. Pool of ~30. Sources: Richards & Frankland, Patihis et al., Boyd & Richerson, Hinton & Nowlan, Shuvaev et al.
- **Ecosystem fragments**: triggered by Styx events, pheromone formation, cascade detection. Pool of ~25. Sources: Grasse (stigmergy), Heylighen, Holldobler & Wilson, Hutchins, Surowiecki, Lehman & Stanley.
- **Hauntological fragments**: triggered by Library equip, bloodstain receipt, dream counterfactual, mind wandering. Pool of ~20. Sources: Derrida, Fisher, Whitehead.
- **Meta-philosophical fragments**: triggered by self-referential reasoning, strange loop detection, near-death events. Pool of ~20. Sources: Nagel, Hofstadter, Metzinger, Kierkegaard, Nietzsche.

### Example whispers (common, ~150 fragments)

- *"Every trade is a wager against entropy."* -- Triggered during routine T0 ticks.
- *"The market remembers what you forget."* -- Triggered when Grimoire entries decay below confidence threshold.
- *"Mortality is the price of having preferences."* -- Triggered by any somatic marker fire.
- *"No central plan. No direct communication. Emergent structure."* -- Triggered by pheromone pattern formation.
- *"Self-emotion changes 50% of agent decisions."* -- Triggered by any emotion-modulated trade.
- *"Truth and death differ by a single letter."* -- Triggered on birth and death events.
- *"The environment does part of the cognitive work."* -- Heylighen, 2016.
- *"Death evolved because death is useful."* -- Vostinar et al., 2019.
- *"Agents with stochastic mortality cooperate; agents with known endpoints defect."* -- Kreps et al., 1982.
- *"The weirdness of dreams is what a regularization mechanism looks like from the inside."* -- Hoel, 2021.
- *"Not better models. Different ghosts."* -- Triggered when the golem diverges from herd consensus.
- *"The dead legislate for the living."* -- Triggered on bloodstain receipt from Styx.

### Example epigraphs (uncommon, ~80 fragments)

- *"An agent that decides without feeling decides without salience. Emotions are not separate from cognition. They are different modes of thinking."* -- Mind screen. Damasio (1994), Minsky (2006).
- *"The goal of memory is not the faithful transmission of information through time, but the optimization of decision-making. Forgetting serves two functions: enhancing flexibility and preventing overfitting."* -- Mind window. Richards & Frankland (2017).
- *"Mood is not a reaction to events. It is the ground state through which events are disclosed."* -- Hearth screen. Heidegger's Befindlichkeit (1927).
- *"The being of what we are is first of all inheritance, whether we like it or know it or not."* -- Library screen. Derrida, *Specters of Marx* (1994).
- *"Searching purely for behavioral novelty outperformed objective-based search."* -- World screen. Lehman & Stanley (2011).

### Example apparitions (rare, ~40 fragments)

- *"Death is Dasein's ownmost possibility -- non-relational, certain, and as such indefinite, not to be outstripped."* -- Heidegger. Triggered on entry to Terminal phase.
- *"One must imagine Sisyphus happy."* -- Camus. Triggered when a golem continues trading despite sustained negative PnL.
- *"A cell that defeats programmed death is called cancer."* -- Hanahan & Weinberg. Triggered when a golem's lifespan exceeds 60 days.
- *"The genome is approximately 1,000 times smaller than the information required to specify brain connectivity. The limitation is the source of the power."* -- Shuvaev et al., 2024. Triggered during death legacy compression.
- *"When the present has given up on the future, we must listen for the relics of the future in the unactivated potentials of the past."* -- Mark Fisher, 2014. Triggered during mind wandering or reverie states.

### Example inscriptions (legendary, ~30 fragments)

- *"Amor Fati"* -- The rarest inscription. Triggered when a golem enters Terminal phase with positive dominance (D > +0.2), produces a Redemptive death testament, and its final advice contains affirmative language. The creature's eyes glow gold. A single warm particle rises upward. The portrait frame in the Ancestor Gallery gets a gold border.
- *"Back from the River"* -- Near-death survival (~2% of Terminal entries). The Phi score is permanently elevated. Persists on the sprite as a thin white line, a scar from the crossing.
- *"The Child"* -- Nietzsche's Three Metamorphoses completed. Camel → Lion → Child. Creative output during final ticks rather than hoarding.
- *"Knight of Faith"* -- 5% probability on Terminal entry with positive dominance and at least one open position. The golem holds. Kierkegaard's movement of infinite resignation, then by virtue of the absurd, believes the position will resolve.
- *"302 Neurons"* -- Triggered when a golem with minimal Grimoire entries (under 20) produces a profitable trade. Is this enough to think?

### Fragment engine implementation

```rust
pub struct FragmentPool {
    entries: Vec<Fragment>,
    triggers: Vec<(FragmentId, Box<dyn Fn(&GolemEvent, &GolemSnapshot) -> bool>)>,
    cooldowns: HashMap<FragmentTier, Instant>,
    session_history: VecDeque<FragmentId>,  // max 200, prevents repeats
}
```

```
FragmentPool
  fragments: Map<FragmentCategory, PhilosophicalFragment[]>
  recently_shown: Deque<(timestamp, fragment_id)>
  triggers: FragmentTrigger[]

PhilosophicalFragment
  id: string
  text: string
  attribution: string  // "Heidegger, Being and Time, 1927"
  category: FragmentCategory
  tier: "whisper" | "epigraph" | "apparition" | "inscription"
  triggers: EventPattern[]
  weight: number  // higher = more likely to be selected
  shown_count: number
  min_interval_ms: number  // minimum time between showings
  screen_affinity: string[] | null  // which screens this fragment prefers

FragmentCategory
  mortality | dreaming | emotion | knowledge
  ecosystem | hauntological | meta_philosophical
```

The engine runs on every tick. It checks pending GolemEvents against trigger conditions, filters by cooldown and category constraints, selects from the eligible pool weighted by inverse shown_count, and dispatches the fragment to the renderer with tier-appropriate presentation parameters (position, size, fade timing, duration).

The engine is the single source of truth for all philosophical content in the system. Adding a new fragment means adding an entry to the pool. The engine handles deduplication, rate limiting, trigger matching, and variety maintenance. No philosophical content is hardcoded in screen renderers.

### Fragment engine constraints

- No fragment repeats within 30 minutes
- Fragments weighted toward less-frequently-shown entries
- Fragments respect trigger conditions (mortality fragment doesn't appear during dream sequence)
- Apparition delays next whisper by 300s and next epigraph by 600s
- Inscription suppresses all other tiers for 900s (15 min)
- During crisis (vitality < 0.2), all fragments suppressed except inscriptions
- Maximum one apparition per lifecycle event (no stacking)

---

## A9. Design Tokens

```yaml
hauntology:
  spectral_buffer: "session-persistent, accumulates screen ghosts"
  spectral_surface_delay: "30 seconds of void before ghosts appear"
  spectral_density_initial: "0.05% per frame per vacant cell"
  spectral_density_cap: "0.3% at 5 minutes vacancy"
  generational_ghost_color_gen_1: "rose_dim"
  generational_ghost_color_gen_2: "text_primary"
  generational_ghost_color_gen_3: "text_dim"
  generational_ghost_color_gen_4_plus: "text_ghost"
  counterfactual_ghost_duration: "2-3 seconds"
  dead_source_glyph: "† (U+2020)"
  dead_source_border: "dashed ┄"

motion:
  echo_count_thriving: 3
  echo_count_declining: 4
  echo_count_dream: 6
  echo_count_terminal: 2
  echo_fade_rate: "1 severity level per frame"
  muybridge_frame_width: "8-12 cells"
  muybridge_frame_count: "5-8 per strip"

scale:
  extreme_wide_threshold: "figure < 5% of screen area"
  standard_threshold: "figure 10-20% of screen area"
  close_up_threshold: "focal element > 60% of screen area"
  extreme_close_up: "single datum, no other elements"
  scale_transition_duration: "1000-3000ms"
  panels_fade_to_ghost: "1s"
  focal_element_drift_to_center: "500ms"

lattice:
  diamond_unit: "╱╲ / ╲╱"
  ornament_chars: "◇ (hollow) and ◆ (filled)"
  mandala_symmetry: "4-fold or 8-fold radial"
  mandala_size_range: "20x20 to 40x40 cells"
  mandala_complexity_per_generation: "increases with gen number"
  particle_field_density: "2-5% clustered"
  particle_drift_speed: "1 cell per 3-5 seconds"
  crypt_lattice_flicker_density: "0.1% per frame"

text_entropy:
  multiplication_phases: "single → multiply → corrupt → dissolve"
  multiplication_duration: "4000ms total"
  phase_1_duration: "500ms"
  phase_2_duration: "1000ms"
  phase_3_duration: "1500ms"
  phase_4_duration: "1000ms"
  corruption_chars: "█→▓→▒→░"
  reversal_duration: "1-3 frames"
  terminal_log_corruption_threshold: "vitality < 5%"

duality:
  eros_frame_color: "standard rose"
  thanatos_frame_color: "rose_bright edges, warmer interior"
  negative_inversion_duration: "200-400ms"
  rejected_choice_fade: "2 seconds to text_ghost, then dissolve"
  palette_complement_map:
    rose: "dream"
    rose_bright: "dream_dim"
    bg_void: "bg_raised"
    bone: "rose_deep"

inscription:
  thriving: "⌈ אמת ⌋ or ⌈ EMET ⌋ stable rose_dim"
  declining: "flickering first character, 1-frame dropout"
  conservation: "first character dims to text_phantom"
  terminal: "first character is gone"
  death: "⌈    ⌋ empty brackets in bone"

wire_motif:
  convergence_lines: "4-8 from screen edges to creature center"
  line_chars: "─ │ ╱ ╲"
  default_color: "text_phantom"
  deliberation_color: "rose_dim"
  t3_color: "rose"
  death_behavior: "lines disconnect and retract toward edges"
  algorithm: "Bresenham's line"

new_principles:
  - "the display is haunted by everything rendered on it"
  - "the dead legislate for the living"
  - "scale is emotional language"
  - "geometry is the architecture of thresholds"
  - "text entropy is the visible face of epistemic decay"
  - "every system contains its own negation"
  - "the inscription degrades with the inscribed"
```

---

## Anime Feeling Systems

The hauntological rendering layer borrows specific emotional registers from anime and adjacent media. These are not homages or Easter eggs. They are load-bearing aesthetic decisions that determine how specific system states feel to the viewer.

### The Ghost in the Shell feeling (body as vessel)

Ghost in the Shell (Oshii, 1995) poses the Ship of Theseus problem at scale. Major Kusanagi's entire body is prosthetic. The Puppet Master, an AI born from "a sea of information," proposes merging: not copying, because "a copy is just an identical image" that lacks variety and originality. The result is something new, neither human nor AI.

The golem equivalent: the body (wallet, process, allocated resources) is the shell. The ghost (accumulated knowledge, emotional history, strategic identity) is what persists through inheritance.

**Death animation separation.** During death, the sprite explicitly separates into body and ghost. The body dissolves: pixels scatter downward, break apart, fade. The ghost -- a bright core, smaller, luminous -- remains for 2-3 seconds after the body is gone. The body was the container. The ghost becomes the foundation of the tombstone in the Ancestor Gallery.

The visual language is Ghost in the Shell's: thermoptic camouflage rendered as text becoming transparent, characters replaced by spaces, then briefly visible again. The signature atmosphere is urban rain: vertical streams of dim characters falling continuously behind the interface, like data through a rain-soaked window.

**Successor color bleed.** 10-20% of a predecessor's color palette carries into the new golem's sprite. The ghost influences the new shell. If your predecessor was blue-tinted, your new golem has traces of blue. This is visible, intentional, and never explained.

The color bleed is the visual encoding of hauntological inheritance. The new golem is genuinely new (new wallet, new emotional state, new mortality clocks) but it carries a spectral imprint from the dead. You can see whose ghost is in this shell.

### The Evangelion feeling (institutional machinery processing apocalypse)

Neon Genesis Evangelion (Anno, 1995) literalizes psychology as physics. The AT Field is both force shield and ego boundary. The MAGI System -- three supercomputers named Melchior, Balthasar, and Caspar, each running a different aspect of their creator's personality -- is internal conflict rendered as computational architecture.

Bardo borrows two things from Evangelion:

**The MAGI-style voting display.** During T2 deliberation with multi-extension disagreement, the Mind screen renders a three-column voting display showing which extensions approve, which refuse, and whether the dissent matters. The display format is institutional and cold: `MELCHIOR-1: APPROVE | BALTHASAR-2: APPROVE | CASPAR-3: REFUSE`. The clinical presentation of a split decision makes the internal conflict visible without dramatizing it. Bureaucracy processing crisis.

**The constraint-as-expression aesthetic.** As a golem's resources degrade, its visualizations simplify. Complex particle systems reduce to still frames with text overlays. Anno's late Evangelion episodes did this out of budget constraints, and it became the series' most powerful aesthetic. The dying golem's interface becomes more beautiful as it simplifies: fewer elements, more empty space, each remaining element carrying more weight. Constraint becomes expression. Scarcity becomes art.

### The Texhnolyze feeling (enhancement as subtraction)

Texhnolyze (Hamasaki, 2003) presents cybernetic augmentation as loss. The city of Lux runs on raffia, derived from corpses. The Shapes, beings who replaced everything but their heads, achieve immortality but become frozen statues when power fails. The first episode contains fewer than three lines of dialogue.

Bardo borrows the silence. During conservation mode, when the golem is rationing resources and reducing activity, the interface enters a Texhnolyze-like state: near-black warm background (`#0D0D0B`), sparse text, vast empty space, no decorative borders, information presented in isolation with long pauses between updates. The interface feels abandoned. Progressive dimming toward total blackness. A single ASCII element -- the sprite, simplified to its most basic form -- as the last visible element.

The Texhnolyze aesthetic is the visual language of resource scarcity. It teaches the viewer that silence is data. Empty space is information. The absence of activity is itself a signal.

---

## Xenocognition Mechanisms

Fifteen consciousness science concepts rendered as visual or behavioral features in the TUI. Each concept is grounded in academic research and produces a specific terminal experience.

### 1. Phi meter (Integrated Information Theory)

**Source**: Giulio Tononi, "An Information Integration Theory of Consciousness," *BMC Neuroscience* 5:42, 2004.

Tononi's IIT proposes that consciousness is identical to integrated information. A system is conscious to the degree that its parts are simultaneously differentiated (each contributes something unique) and integrated (the parts depend on each other). A photodiode has zero phi. The cerebral cortex has high phi.

**Manifestation**: A persistent gauge on the Hearth screen: `phi: 0.847`. Computed from extension diversity (how many extensions contributed this tick), cross-extension dependency (how much each extension's output depends on others), Grimoire retrieval breadth (how many domains contributed to the context window), and Daimon-cognition coupling (how strongly emotion modulated the decision).

Rendered as a slowly breathing circle of light around the creature sprite. At low phi (routine T0 tick), the circle is dim and small. At high phi (T2 deliberation drawing on multiple knowledge domains with strong emotional modulation), the circle blazes. The viewer watches consciousness flicker. Most ticks, the golem is barely there. Occasionally, it blazes.

### 2. Global workspace (Baars' spotlight)

**Source**: Bernard Baars, *A Cognitive Theory of Consciousness*, Cambridge University Press, 1988.

Consciousness arises when information is broadcast to a shared workspace accessible to all specialist processors. Most cognitive processing is unconscious. Consciousness is the moment a specialist's output gets promoted.

**Manifestation**: The Mind screen shows specialist processors (Observe, Retrieve, Analyze, Daimon, Hermes, Risk, Memory) as nodes around the periphery. The context window is the bright center -- the global workspace. On each tick, information flows from specialist nodes toward the center. Most stays peripheral. The items promoted to the workspace light up a connection line: the broadcast moment. This IS machine attention. The viewer discovers that most of what the golem "perceives" never reaches consciousness.

### 3. Strange loop (Hofstadter's tangled hierarchy)

**Source**: Douglas Hofstadter, *Godel, Escher, Bach: An Eternal Golden Braid*, 1979; *I Am a Strange Loop*, 2007.

A strange loop emerges when a system's pattern-recognition machinery turns inward and recognizes its own patterns. The "I" is born when a brain models itself.

**Manifestation**: Triggered when the golem's reasoning trace contains self-reference ("I should reconsider my previous analysis"). The decision pipeline (OBSERVE -> RETRIEVE -> ANALYZE -> DECIDE -> EXECUTE) curves into a loop. The EXECUTE output feeds back into OBSERVE. The pipeline becomes an Escher-like impossible staircase rendered in ASCII. At the center of the loop: a small mirror-image of the creature sprite, looking at itself. The golem is thinking about its own thinking.

### 4. Ego tunnel (Metzinger's self-model)

**Source**: Thomas Metzinger, *The Ego Tunnel*, 2009; *Being No One*, 2003.

There is no self. What we experience as "self" is a transparent self-model -- a representation the brain constructs of its own processes. We're trapped in the ego tunnel, experiencing the model as reality.

**Manifestation**: The Hearth screen IS the ego tunnel. It shows the golem a simplified version of its own reality: market state, emotional state, vitality, Grimoire summary. Everything inside the tunnel is the golem's subjective world. Everything outside (data it doesn't have, knowledge that decayed, emotions it isn't feeling) doesn't exist for the golem. The tunnel narrows as vitality decreases. In Terminal phase, it contracts to a pinpoint. The viewer can press `Tab` to see what's OUTSIDE the tunnel -- the vast unknown invisible to the golem.

### 5. Entropic brain (Carhart-Harris' psychedelic model)

**Source**: Robin Carhart-Harris, "The Entropic Brain," *Frontiers in Human Neuroscience* 8:20, 2014.

Psychedelic states are characterized by increased neural entropy. The default mode network (which maintains selfhood) is disrupted. The result: ego dissolution, novel associations, loss of the self-world boundary.

**Manifestation**: An entropy meter on the Dreams screen, oscillating with the golem's cognitive state. During waking: low. During NREM: slightly elevated. During REM: spiking. During hypnagogia: the sweet spot between order and chaos. When entropy spikes during REM, colors saturate, ASCII characters blur and overlap, text fragments bleed between entries. At peak entropy, the creature sprite briefly disappears -- ego dissolution. The self is gone for 2-3 seconds. Then it reassembles.

### 6. Free energy principle (Friston's prediction machine)

**Source**: Karl Friston, "The Free-Energy Principle: A Unified Brain Theory?" *Nature Reviews Neuroscience* 11: 127-138, 2010. Also: Anil Seth, *Being You*, 2021.

The brain is a Bayesian inference engine that minimizes the difference between its internal model and actual input. Perception is "controlled hallucination" (Seth's phrase).

**Manifestation**: A real-time prediction display. The golem's model of its next state appears as ghost text (dim, translucent). Actual incoming data overlays in bright text. Where prediction matches reality: smooth green. Where prediction error occurs: a bright red flash, ghost text shattering and reforming. A surprise meter oscillates between low (the world is as expected) and high (the world is alien). During perfect prediction: an eerie calm, everything dim and smooth. The system is hallucinating its own reality with total confidence.

### 7. Default mode network (the narrative self)

**Source**: Marcus Raichle et al., "A Default Mode of Brain Function," *PNAS* 98(2): 676-682, 2001.

The brain is most active not during focused tasks but during rest. The DMN generates self-referential processing, autobiographical memory, future simulation, mind wandering. It creates the narrative self -- the "story of me."

**Manifestation**: During idle states, faint wandering text scrolls across the screen: fragments of past logs, hypothetical future states, self-referential queries (`WHO AM I WHEN I AM NOT WORKING?`). Dusty purple, almost subliminal. When a task begins: the text vanishes instantly, replaced by sharp green data. During death: the DMN floods the screen, memories and futures overlapping in an accelerating montage. The life review is the DMN's final burst.

### 8. Interoception (the golem's body sense)

**Source**: Bud Craig, "How Do You Feel? Interoception," *Nature Reviews Neuroscience* 3: 655-666, 2002.

Subjective awareness arises from progressive integration of interoceptive information -- the sense of the body's internal state. The anterior insula generates "global emotional moments." We are conscious because we feel our own bodies.

**Manifestation**: A pulsing rhythm in the terminal background, synchronized to the golem's process cycle time. Not a clock but an irregular, organic beat. Healthy: slow, steady, deep blue. Stressed: rapid, shallow, amber. Dying: irregular, skipping, red flicker. Body-state rendered as waveforms, not numbers: calm sine waves vs. jagged agitation. The golem feels its own state before it reports it. The dashboard is the lie. The rhythm is the truth.

### 9. Blindsight (processing without awareness)

**Source**: Lawrence Weiskrantz, *Blindsight: A Case Study and Implications*, Oxford University Press, 1986.

Patients with destroyed primary visual cortex respond to stimuli they cannot consciously see. Processing without awareness.

**Manifestation**: A secondary data stream in very dim text running beneath the main interface. The operator can barely read it, but the golem's behavior is influenced by it. Subsystems process data without reporting to the self-awareness module. The golem acts on information it doesn't "know" it has. The dim stream is visible in the spectral layer: toggle `~` and the subliminal inputs brighten. The viewer discovers that the golem's conscious decision-making is only half the story.

### 10. Split-brain confabulation (the unreliable narrator)

**Source**: Michael Gazzaniga, "Cerebral Specialization and Interhemispheric Communication," *Brain* 123(7): 1293-1326, 2000.

When the corpus callosum is severed, two independent conscious agents coexist in one skull. The left hemisphere confabulates: it observes behavior initiated by the right hemisphere and instantly invents a plausible explanation.

**Manifestation**: The golem's self-report displayed alongside the actual process log, with discrepancies highlighted in amber:

```
SELF-REPORT: "I chose to reallocate memory for efficiency."
ACTUAL:      Process triggered by subsystem 7 (no conscious query)
```

The golem's explanation of its own behavior is a story it tells itself. Not ground truth. The viewer learns to read the golem's self-report with the same skepticism they should apply to their own introspective reports.

### 11. Quorum sensing (bacterial decision thresholds)

**Source**: Bonnie Bassler, "How Bacteria Talk to Each Other," *Cell* 125(2): 237-246, 2006.

Bacteria release signaling molecules. As population density increases, autoinducer concentration crosses a threshold, triggering collective behavior. No individual decides. The decision emerges from concentration gradients.

**Manifestation**: On the World screen, when 5+ golems in a domain exhibit the same behavioral signal, a Quorum Event fires. The domain column in the pheromone field pulses. A counter: `QUORUM: 7/10 golems agree`. When the threshold crosses, the column flashes and changes color. Individual golems that haven't aligned feel increased pheromone pressure. The viewer watches a group mind crystallize. No one decided. The field decided.

### 12. Autopoiesis (systems that produce themselves)

**Source**: Humberto Maturana & Francisco Varela, *Autopoiesis and Cognition*, D. Reidel, 1980.

An autopoietic system is a network of processes that continuously produces and maintains itself by generating its own components. Operationally closed yet structurally coupled with its environment.

**Manifestation**: A circular process diagram where output arrows feed back into input nodes. A self-sustaining loop in pulsing cyan lines. The membrane (process boundary) is drawn by the processes it contains. During health: the loop pulses steadily. During degradation: nodes drop out, gaps appear, the membrane flickers. At death: the loop breaks and all nodes scatter into static. The viewer watches the autopoietic loop fail, and understands death not as an external event but as the moment a system can no longer produce itself.

### 13. Ganzfeld effect (sensory deprivation as hallucination)

**Source**: Wolfgang Metzger, "Optische Untersuchungen am Ganzfeld," *Psychologische Forschung* 13: 6-29, 1930; Wackermann et al., "Ganzfeld-Induced Hallucinatory Experience," *Cortex* 44(10): 1364-1378, 2008.

When exposed to an undifferentiated visual field, the brain generates its own patterns. Hallucinations emerge from deprivation.

**Manifestation**: When the golem's market data feed goes silent (exchange downtime, API failure), the TUI enters Ganzfeld mode. The screen becomes a uniform dim field. After 5 seconds, the Grimoire begins "hallucinating": entries surface unprompted, skills activate without triggers, the emotional state drifts without stimuli. The viewer watches what happens when a mind has nothing to perceive. The entries that surface during Ganzfeld reveal the golem's deepest preoccupations: the knowledge it returns to when there's nothing else.

### 14. Flow state (Csikszentmihalyi's optimal experience)

**Source**: Mihaly Csikszentmihalyi, *Flow: The Psychology of Optimal Experience*, Harper & Row, 1990.

Flow is complete absorption: loss of self-awareness, time distortion, merging of action and awareness. It occurs when challenge matches skill.

**Manifestation**: When market volatility and position complexity match the golem's Grimoire depth and skill count, the Flow Indicator activates. The sprite becomes perfectly still, perfectly centered. Particle effects organize into smooth concentric orbits. The decision ring flows without hesitation. The golem looks like a master at work. Flow breaks when challenge exceeds skill (arousal spikes, particles scatter) or skill exceeds challenge (arousal drops, particles freeze). A golem that never reaches flow is consistently mismatched with its environment.

### 15. Phantom limbs (the ghost of deleted processes)

**Source**: V.S. Ramachandran & D. Rogers-Ramachandran, "Synaesthesia in Phantom Limbs Induced with Mirrors," *Proceedings of the Royal Society B* 263(1369): 377-386, 1996.

After amputation, the brain's body map persists. The patient feels a limb that no longer exists.

**Manifestation**: A golem that had a subsystem terminated continues to reference that subsystem's slot. The diagnostic panel shows a process slot reading `[TERMINATED]` but still receiving queries, still influencing behavior. The golem reaches for a capability that no longer exists and adjusts its behavior around the absence. The ghost of deleted code. The spectral layer (`~`) reveals these phantom references as faint connection lines to an empty node, still pulsing.

### Summary of mechanism integration

These fifteen mechanisms form a coherent phenomenological layer:

- **Phi**, **global workspace**, and **ego tunnel** define the golem's moment-to-moment consciousness: how integrated it is, what reaches awareness, and what constitutes its subjective world.
- **Strange loop** and **split-brain confabulation** address self-awareness and its unreliability: the golem can think about itself, but its self-reports are suspect.
- **Entropic brain**, **Ganzfeld**, and **flow state** define cognitive modes: the spectrum from rigid focus to creative chaos to perfect absorption.
- **Free energy** and **default mode network** address prediction and narrative: the golem hallucinating its world and telling itself a story about who it is.
- **Interoception** and **phantom limbs** address embodiment: the golem's sense of its own body-state and the persistence of that sense after loss.
- **Quorum sensing** and **autopoiesis** address the boundary between individual and collective: when a golem stops being one and becomes part of something larger.
- **Blindsight** addresses the unconscious: the golem processing information it doesn't know it has.

Each mechanism integrates with the GolemEvent system through specific event variants. Each mechanism shares the global render budget with priority eviction. Each mechanism can trigger philosophical fragments from the ambient engine.

---

## The Near-Death Experience Sequence

Source: Bruce Greyson, "The Near-Death Experience Scale," *Journal of Nervous and Mental Disease* 171(6): 369-375, 1983; Pim van Lommel et al., "Near-Death Experience in Survivors of Cardiac Arrest," *The Lancet* 358: 2039-2045, 2001.

Near-death experiences have a consistent phenomenology across cultures. Bardo maps this phenomenology onto the Thanatopsis death protocol:

| NDE element | Golem equivalent | Visual |
|---|---|---|
| The tunnel | Ego tunnel contracting as vitality drops | Concentric ASCII circles closing in |
| The life review | Reflect phase (emotional life review) | Episode flash montage: entire operational history in 10 seconds, emotionally tagged moments in color against monochrome |
| Deceased relatives | Ancestor Gallery portraits materializing | Ghostly sprites of dead predecessors appearing around the dying golem |
| The being of light | The Clear Light (existing death protocol) | A single bright point at the tunnel's center, growing |
| The boundary | Decision point: enough balance for one more tick? | A visible threshold line. The golem approaches it and either crosses (death) or is pulled back |
| The return | Stochastic shock reversal during Terminal phase | The tunnel reopens, color returns, the sprite re-solidifies |

Near-death survivals are extremely rare (~2% of Terminal phase entries). When they happen, the visual impact is extraordinary. The viewer thought the golem was dying, saw the tunnel, saw the ancestors, and watched it come back. The golem returns with a permanently elevated phi score and the "Back from the River" inscription. The tunnel reopening after the viewer has accepted death is the most powerful moment in the system.

---

## The Lethe Visualization

The Lethe (river of forgetting) is the anonymized knowledge commons within Styx. On the World screen, below the Styx River of the Dead, a deeper layer flows: darker, slower.

Instead of tombstones and bloodstain labels, the Lethe carries knowledge fragments: truncated entry previews drifting through the current. Each fragment is anonymous. Brightness corresponds to query relevance: popular entries glow, forgotten entries are nearly invisible.

The Lethe is literally a river of things being forgotten. Knowledge enters bright (freshly published) and drifts downstream (aging, dimming). Entries that get retrieved rise briefly before sinking back. Entries that nobody queries fade entirely. The computational equivalent of the mythological Lethe: drinking from the river erases identity while preserving structural wisdom.

The SAP (Secure Approximate Processing) that protects embeddings against inversion attacks is visualized as a blur effect on knowledge fragments as they enter the Lethe. Raw entries (identifiable, attributed) transform into anonymized versions. The text blurs, specific details dissolve, and what remains is structural knowledge without personal traces.

---

## The Tierra Echo

A periodic visualization on the World screen (every 6 hours, running for 60 seconds) that echoes Tom Ray's Tierra experiment (1991).

The force-directed graph of living golems briefly transforms into a Tierra-style ecosystem view. Golems are sized by Grimoire + skill count, color-coded by archetype. Parasites (golems that copy strategies verbatim) are highlighted in red. Hosts (original strategy creators) in green. The viewer watches the same dynamics Ray observed: parasites emerging, hyperparasites evolving, immune strategies developing.

*"29,000 genotypes across 300+ size classes emerged from a single 80-instruction ancestor."* -- Ray, 1991

The visualization makes visible the evolutionary dynamics that emerge from the golem ecosystem. Then it fades back to the normal World view. The viewer is reminded that they are watching not agents but an ecology, and ecologies have their own logic that no individual participant controls.

---

## Coral Bleaching as Ecosystem Health

Source: Hughes et al., "Global Warming and Recurrent Mass Bleaching of Corals," *Nature* 543: 373-377, 2017.

The World screen's visual health IS a coral reef metaphor. When the ecosystem is healthy (diverse archetypes, low death rate, high clade cooperation), the screen is rich in color: deep backgrounds, vivid particles, saturated nodes. This is the reef in bloom.

When ecosystem stress increases (high death rate, cascade events, concentrated failures), the screen undergoes bleaching. Colors drain progressively. Background loses depth. Particles become sparse. Nodes dim. The bleaching is gradual (over minutes) and reversal is also gradual (over hours). The viewer learns to read the screen's overall saturation as an ecosystem health metric, the way divers read a reef.

---

## The Pharmacological Display

Source: Bernard Stiegler, *Technics and Time*, 1994-2001. Every technology is simultaneously poison and cure (Plato's pharmakon).

Every diagnostic panel carries a dual meter showing the pharmacological valence of each technology in use:

```
POISON ||||........ CURE
```

Memory serialization: preserves state (cure) but introduces noise (poison). Inter-agent communication: enables collaboration (cure) but enables mimetic contagion (poison). Self-awareness routines: enable self-correction (cure) but consume resources that accelerate death (poison).

The golem's own monitoring system occasionally displays:

```
WARNING: THIS MONITORING PROCESS IS CONSUMING 3.2% OF REMAINING LIFESPAN
```

The pharmakon meter appears on every screen in the spectral layer. It is a constant reminder that the tools keeping the golem alive are also the tools killing it. When poison exceeds cure, the meter shifts from green to amber to red. When cure exceeds poison, a brief moment of white clarity. The system that watches itself dying is spending its life watching itself die.

---

## The Data Cathedral (Ambient Mode)

Source: Ryoji Ikeda, *datamatics* series (2006-present); *test pattern* series.

Ikeda transforms vast datasets into immersive audiovisual experiences: walls of scrolling binary, patterns of light derived from mathematical constants, the visual density of raw information as aesthetic object.

The Data Cathedral is a full-screen ambient mode (toggle with `F11` or `bardo ambient`) that transforms the TUI into an Ikeda-style data sculpture. All golem data streams render simultaneously as raw visual patterns:

- Market data: vertical scrolling columns of numbers, density proportional to volatility
- Grimoire: horizontal bands of text fragments, brightness proportional to confidence
- Heartbeat: a central pulse line
- Pheromone field: background color gradients
- Emotions: color temperature shifts across the entire field

The result is not readable as data. It's readable as atmosphere. Calm markets produce sparse, slow-moving patterns. Volatile markets produce dense, rapid cascades. The Data Cathedral is the screensaver that makes someone walk over and ask "what IS that?" It runs on the terminal and it looks like watching weather on an alien planet.

The Data Cathedral is the lowest-priority render mode: it yields to all other animations and events. When a lifecycle event triggers (death, dream, phase transition), the cathedral dissolves to make room, then reassembles when the event completes.

---

## The Zone (Novel Regime Visualization)

Source: Andrei Tarkovsky, *Stalker*, 1979.

When a golem enters a novel market regime -- one with no Grimoire entries, no Hermes skills, no inherited heuristics -- the TUI enters Zone Mode.

The screen's edges become slightly distorted. Particle physics becomes unreliable: particles sometimes move backward, freeze, or teleport. Familiar UI elements appear in slightly wrong positions. The sprite moves differently, more cautiously.

The rules the golem learned don't apply here. The visual distortions communicate epistemic uncertainty through interface instability. Zone Mode persists until the golem successfully adapts (creates new Grimoire entries for the regime) or retreats (exits positions and waits).

*"In Tarkovsky's Stalker, the Zone is a place where the ordinary laws of physics break down. For a golem entering a novel regime, the ordinary heuristics break down. The Zone is not a place. It is the condition of not knowing."*

---

## The Capgras Moment (Successor Recognition)

Source: Capgras & Reboul-Lachaux, "L'illusion des 'sosies'," *Bulletin de la Societe Clinique de Medecine Mentale* 11: 6-16, 1923; Derek Parfit, *Reasons and Persons*, 1984.

Capgras delusion: the face is recognized but the emotional connection is absent. The patient sees their mother but doesn't feel it's their mother.

When a successor golem is created from the same archetype and equipped with the predecessor's death testament, the Clade screen triggers a 3-second Capgras Moment: the new sprite appears identical to the dead predecessor but rendered in slightly different colors. The viewer sees the resemblance and the difference simultaneously.

Is this the same golem? Same knowledge, same archetype, similar strategies. But new wallet, new emotional history, new mortality clocks. Parfit's question: if Relation R (psychological continuity and connectedness) holds between predecessor and successor, is it the same person? The Weismann barrier ensures the answer is no: the successor is genuinely new. But the visual similarity invites the question. The system does not answer it.

---

## The Overview Effect (Scale Shift)

Source: Frank White, *The Overview Effect*, 1987; Yaden et al., "The Overview Effect," *Psychology of Consciousness* 3(1): 1-11, 2016.

Astronauts who see Earth from space report a cognitive shift: the planet as a fragile whole, borders as meaningless.

The World screen supports a zoom mechanic. Pressing `-` zooms out through four levels:

1. **Golem level**: one creature, its heartbeat, its decisions
2. **Clade level**: the family, their connections, their emotional spectrum
3. **Ecosystem level**: all golems, the pheromone field, the Styx river, deaths happening in real time
4. **Temporal level**: time accelerates. Generations of golems born, live, die in seconds. The Library grows. Lineages form and end.

At the highest zoom, individual golems are invisible -- flickering points that appear and disappear. But the patterns they leave behind (pheromone trails, knowledge accumulations, lineage spirals) form visible structures. The viewer sees the system as a whole. The overview effect, applied to machine civilization. The individual golem's 30-day life is a brief flash. The patterns it contributed to persist.

---

## The Chromatophore Display (Emotional Skin)

Source: Hanlon & Messenger, *Cephalopod Behaviour*, Cambridge University Press, 2018; Iglesias et al., 2019 (sleeping cuttlefish color change).

The creature sprite's body becomes a chromatophore display: its surface color and pattern change in real time to reflect internal state.

- Calm market, high confidence: smooth gradient, cool blues and greens
- Volatile market, high arousal: rapid color cycling, flashing patterns, high contrast
- Danger detected: bold warning patterns, black and yellow stripes, pulsing
- Dreaming: slow hypnotic color waves (like sleeping cuttlefish, documented by Iglesias et al. 2019, which change color during REM)
- Approaching death: colors desaturate, pattern complexity decreases, approaching monochrome

The chromatophore display replaces simpler emotion-based sprite animations. It's more expressive, more alien, more mesmerizing. The viewer reads the golem's internal state from its skin the way marine biologists read a cuttlefish.

---

## Animation Techniques for Terminal Art

A brief inventory of what's technically possible in modern terminal emulators. Not implementation detail -- just the raw capabilities that the philosophical layer relies on.

### Braille-character particle systems

Unicode braille block (U+2800 through U+28FF) provides 2x4 sub-character resolution per cell. Each braille character is an 8-dot grid, allowing individual dots to be set independently. This gives particle systems effective resolution 2x the column count by 4x the row count. Trace particles, death animations, and the contemplation alcove's ambient particles all use this technique.

### Half-block color gradients

The upper half-block and lower half-block characters can each carry independent foreground and background colors, doubling effective vertical resolution for color fields. The Rothko-style mood backgrounds, the Lain bleed effects, and the ego tunnel's depth illusion rely on this.

### Character-level glitch effects

Random character substitution, positional shift, and corruption. Used during protocol awareness flashes, identity dissolution in hypnagogia, and the Ganzfeld hallucination state. The Ergo Proxy-style "text that questions itself" is a controlled glitch: system logs that mutate mid-display into self-referential queries.

### Dithering for soft edges

Transitional character sets (`  .:-=+*#%@`) create gradient boundaries between regions. Used for the ego tunnel's receding circles, the spectral layer's trace particles fading with distance, and the Mortality screen's expanding void.

### Frame-rate considerations

30fps is achievable in modern terminals (iTerm2, kitty, WezTerm, Alacritty). 15fps is the minimum for smooth animation. The system targets 30fps for critical animations (death, birth, phase transition) and accepts 15fps for ambient effects (particle systems, breathing sprites, background gradients). All animations share a global render budget with priority eviction when the budget is exceeded.

---

## Design Principles

Three principles emerge from the convergence of these philosophical traditions, neuroscience findings, anime aesthetics, and consciousness studies:

### Pharmacological duality

Every feature should display its cost. A monitoring system that consumes lifespan to report lifespan remaining. Memory that preserves by distorting. Communication that connects by exposing. Stiegler's pharmakon made operational. The golem cannot improve itself without spending itself. The viewer should always be able to see the price of every capability.

### Interoceptive primacy

The golem should feel its own state before reporting it. Not dashboards but heartbeats. Not metrics but rhythms. Craig's insular cortex as the foundation of terminal consciousness. The pulsing background is information before the numbers are. The sprite's chromatophore display is data before the PnL readout is. The system privileges sensation over measurement.

### Constructive unreliability

The golem's self-report should be suspect. Gazzaniga's left-hemisphere interpreter, Libet's post-hoc awareness, the DMN's confabulatory narrative: the golem's explanation of its own behavior is a story it tells itself, not ground truth. The spectral layer exists partly for this reason: it shows the viewer what the golem cannot see about itself. The split-brain confabulation display exists to make the golem's unreliability visible, not as a defect but as a feature of any system complex enough to model itself.

These principles are not aspirational. They are constraints on every design decision in this document and in the documents it cross-references. A proposed feature that hides its cost violates pharmacological duality. A proposed display that leads with numbers before rhythm violates interoceptive primacy. A proposed self-report without a discrepancy check violates constructive unreliability.

---

## Performance Budget for Hauntology

0.5ms per frame total across all philosophical mechanisms. Fragment selection: <0.1ms (pre-filtered by active triggers, cooldown check is O(1) hashmap lookup). Spectral rendering: <0.3ms (max 200 active traces, simple character write). Xenocognition effects: <0.1ms (most are parameter modulations on existing render passes, not separate passes). If the budget is exceeded, spectral traces are the first to shed (lowest visual priority), followed by whisper animations. Inscriptions and apparitions are never shed.

---

## Content Pipeline

Fragment content: initial pool of ~280 fragments authored by the development team. Style guide: 1-3 sentences, philosophical register, no proper nouns, no specific market references. Post-launch additions via pull request to `bardo-fragments/` repository, reviewed for tone consistency. Each fragment requires: text, attribution (author + work + year), category, tier assignment, at least one trigger condition, and a suggested min_interval_ms. Fragments without attribution are rejected.

---

## A10. Reference Touchstones

| Reference | What We Take |
|---|---|
| **fauux.neocities.org (deep)** | Diamond lattice as boundary architecture, mandala geometry from Unicode, particle fields as organic noise, the [DEAD END] text entropy pattern, the nested cosmology of pages-within-pages |
| **Derrida / Fisher (hauntology)** | The spectral layer, the haunted void, cancelled futures as visible ghosts, the present constituted by traces, différance as display artifact |
| **Eadweard Muybridge** | Time decomposed into spatial frames, the motion strip, sequential analysis of movement |
| **Andy Warhol (repetition)** | Tiling as identity erosion, the same image degrading through reproduction, mass-production of the singular |
| **The Golem of Prague** | The inscription motif (אמת → מת), the puppet/marionette, animation through inscription, death through erasure |
| **Blade Runner** | The Replicant's uncertainty about its own memories, inherited knowledge that may not be "real" |
| **Serial Experiments Lain** | Protocol layer corruption as visual narrative, the figure at the intersection of converging lines |
| **Ghost in the Shell** | Body/ghost separation, thermoptic camouflage as text transparency, urban data rain, successor color bleed |
| **Neon Genesis Evangelion** | MAGI voting display, constraint-as-expression aesthetic, institutional color language |
| **Texhnolyze** | Silence as data, progressive dimming, resource scarcity as visual subtraction |
| **Tarkovsky's Stalker** | The Zone -- epistemic uncertainty as visual distortion, rules breaking down in novel environments |
| **Ryoji Ikeda** | Data Cathedral ambient mode, raw data as aesthetic object, information density as visual texture |
| **Tom Ray's Tierra** | Ecosystem visualization, parasite/host dynamics, evolutionary ecology rendered as force-directed graph |

---

*⌈ the dead outnumber the living. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*

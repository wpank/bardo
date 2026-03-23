# Hypnagogia: Inner Worlds and Dream Chamber Rendering [SPEC]

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Parent**: `00-overview.md` (Track 5: Hypnagogia)
>
> **Source material**: `mmo2/13-hypnagogic-engine.md` (inner worlds section, dream chamber section)
>
> **Cross-references**: [02-architecture.md](02-architecture.md) (HypnagogicOnset state machine), [../18-interfaces/perspective/06-hauntology.md](../18-interfaces/perspective/06-hauntology.md) (spectral layer, motion echo), [../18-interfaces/rendering/00-design-system.md](../18-interfaces/rendering/00-design-system.md) (ROSEDUST palette)

---

> **Reader orientation:** This document specifies the visual rendering of each sleep stage as experienced from inside the Golem's (mortal autonomous agent's) terminal interface within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers NREM replay theater (compressed episodic replay with visual annotation), REM counterfactual garden (branching scenario visualization), hypnagogic phosphenes (Kluver form constants rendered in terminal), integration crystallization (knowledge solidification animation), and the Dream Chamber portal mode. Where `02-architecture.md` describes what happens computationally, this document describes what it looks like. For a full glossary, see `prd2/shared/glossary.md`.

## What this document is

The visual rendering specification for each sleep stage as experienced from inside. Where `02-architecture.md` describes the computational pipeline, this document describes what it looks like. Each sleep stage has a distinct terminal rendering, and the visual design follows a single principle: **the rendering should make the golem's cognitive state legible without reducing it to telemetry.** You are not reading a log. You are watching a mind process its own experience.

The overall visual direction connects to the cinematic language in the interfaces spec, which defines the dream cycle's sub-phases as animation sequences. What follows specifies the inner spatial structure of each phase -- the rooms the viewer enters when portal mode (F4) opens the dream from the inside. Where the cinematic spec describes what you see from the outside (a sleeping creature, particles, color shifts), this document describes what you see when you step through the glass.

---

## NREM: The Replay Theater

During NREM, the golem replays recent episodes with minor mutations. This is memory consolidation -- strengthening significant memories, letting irrelevant ones decay.

### Episodes as vignettes

Each episode renders as a self-contained scene in a bordered panel, 40-60 columns wide and 8-14 rows tall. The panel contains the episode's key data (asset pair, action, outcome, tick number) but filtered through the golem's emotional state at the time of encoding. A trade made during high anxiety renders with jittery borders (alternating between `│` and `┊` frame-to-frame) and a cool blue palette. A trade made during confident calm renders with stable double-line borders (`║`) and warm amber.

### Salience ordering

Vignettes are arranged spatially by emotional significance, not chronology. The most intense episode occupies the center of the viewport. Surrounding episodes radiate outward in rings, each ring less salient than the one before. The viewer sees what the golem considers most important by looking at what is closest to the center. Chronological order is explicitly broken -- tick 1420 might sit next to tick 890 if both carried high somatic marker intensity.

### Visible mutations

This is the detail that makes NREM legible as consolidation rather than playback. During replay, specific values within each vignette shift. A price that was `$2,003.41` in reality renders as `$2,008.12`. A fee tier that was `0.30%` appears as `0.25%`. Mutated values are marked: they render in `dream_dim` color (desaturated blue-gray) with a tilde prefix (`~$2,008.12`). The golem is stress-testing its own memories. It remembers the event but probes whether the pattern holds when the numbers change. The mutations are small, not random -- they drift within a plausible range, as if the golem is asking "would I have done the same thing if conditions were slightly different?"

The viewer watches and can tell, at a glance, which parts of a memory the golem trusts (rendered normally) and which parts it is questioning (tilde-prefixed, dimmed).

```
NREM REPLAY THEATER (salience-ordered, center = highest)
╔══════════════════════════════════════════════════════════╗
║                                                          ║
║            ┌──────────────────────────┐                  ║
║            │ TICK 1420  ETH/USDC      │                  ║
║            │ BUY 2.4 ETH @ ~$2,008.12 │  <-- mutated    ║
║            │ gas: ~14.2 gwei          │  <-- mutated    ║
║            │ P&L: +$34.80             │                  ║
║            │ ◈ somatic: anxiety 0.72  │                  ║
║            └──────────────────────────┘                  ║
║   ┊╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┊   ┌────────────────────────┐    ║
║   ┊ TICK 890  WBTC   ┊   │ TICK 1105  UNI/ETH     │    ║
║   ┊ SELL @ ~$43,200  ┊   │ ADD LIQ  range 1800-   │    ║
║   ┊ slippage: ~0.4%  ┊   │ 2200  fee: ~0.25%      │    ║
║   ┊ P&L: -$112.30    ┊   │ ◈ somatic: calm 0.41   │    ║
║   ┊╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┊   └────────────────────────┘    ║
║                                                          ║
║       ░ ░  ░   (fading outer ring: low-salience)    ░    ║
╚══════════════════════════════════════════════════════════╝
  ~ prefix = value mutated during consolidation
  jittery border (┊) = encoded during high anxiety
  stable border (│ or ║) = encoded during calm
```

The vignettes cycle at 3-8 second intervals. Between vignettes, a dissolve transition plays: characters in the current panel decay to `░`, hold for 200ms, then reform as the next memory. The dissolve mirrors the brain's sharp-wave ripple frequency during slow-wave sleep -- bursts of activity separated by brief silences.

---

## REM: The Counterfactual Garden

REM is where the golem stops replaying and starts imagining. It takes real events and asks "what if?" The rendering makes the boundary between real and imagined explicit.

### Branching tree structure

The viewport shows a decision tree rooted in a real episode. The root node is a bright point -- the moment where the original decision was made (a swap executed, a position opened, a rebalance triggered). From this node, paths branch.

The **real path** extends leftward. It shows what actually happened, rendered with solid borders (`─`, `│`, `┌`, `└`) in the golem's standard color palette. The real path is the trunk of the tree. It has weight and definition.

The **counterfactual paths** branch rightward. They show what could have happened, rendered with dashed borders (`┄`, `┆`, `╌`, `╎`). The dashing is the visual signal: if the border is dashed, the content never happened. It is the golem's imagination, not its memory.

### Impossible combinations and color saturation

When the golem combines elements from unrelated episodes (a gas price from tick 89 with an emotional state from tick 203 with a lending position it has never actually held), the color palette breaks its normal rules. Saturation increases beyond the standard bounds. Hues rotate through the spectrum. The more semantically distant the combined elements (measured by cosine distance in the Grimoire's embedding space), the more psychedelic the rendering.

Embedding model: all-MiniLM-L6-v2 (384 dimensions, ONNX quantized). Distance bands:

- **Slightly improbable** (cosine distance 0.3-0.5): standard rendering with a faint `dream` tint, borders still dashed but recognizable
- **Moderately improbable** (0.5-0.7): character substitution begins -- numbers render as symbols (`$` becomes `¤`, digits become `◊`), text fragments overlap and bleed into each other
- **Wildly improbable** (0.7+): full psychedelic mode -- colors saturate to vivid magenta and electric blue beyond the normal palette, characters drift from their grid positions by 1-2 cells, borders dissolve entirely, the content floats free

The degree of visual distortion is not cosmetic. It is information. The viewer can tell how far from reality the golem has wandered by how strange the screen looks.

```
REM COUNTERFACTUAL GARDEN (real trunk left, hypotheticals right)
┌───────────────────────────────────────────────────────────┐
│                                                           │
│  REAL (solid)                    COUNTERFACTUAL (dashed)  │
│                                                           │
│  ┌──────────────┐    ◉         ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ │
│  │ SOLD 2.4 ETH │───►decision──┄┄► ╎ HELD 2.4 ETH      ╎│
│  │ @ $2,003     │    point     ┄┄► ╎ price rose to      ╎│
│  │ P&L: +$34    │              ┄   ╎ ~$2,180 (+$425)    ╎│
│  └──────┬───────┘              ┄   ╎╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╎│
│         │                      ┄        ┆                 │
│         │                      ┄        ┆ (branches       │
│  ┌──────┴───────┐              ┄   ╎╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╎│
│  │ reinvested   │              ┄┄► ╎ HEDGED w/ PUT      ╎│
│  │ into UNI LP  │                  ╎ cost: ~$12         ╎│
│  │ +0.3% fees   │                  ╎ net: +$413  ◊¤◊    ╎│
│  └──────────────┘                  ╎╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╎│
│                                                           │
│  ─── solid = real   ┄┄┄ dashed = imagined                │
│  standard color = plausible   saturated = improbable      │
└───────────────────────────────────────────────────────────┘
```

The branching tree is not static. As the golem's REM phase progresses, counterfactual branches themselves branch (counterfactuals of counterfactuals). The tree grows rightward. Branches at the far right are the most speculative, the most distant from reality, and the most visually distorted.

Navigation during REM portal mode is non-Euclidean (the Yume Nikki / Antichamber pattern). Going back along a branch does not guarantee you return to the same node. The dream space mutates between traversals. A branch you explored may have changed by the time you return. This is the golem's associative process at work -- its dream content is not a fixed data structure but a live, shifting thing.

---

## Hypnagogia: The Liminal Threshold

Hypnagogia is the transition state. The golem is between waking and dreaming. Executive control has loosened but not collapsed. The Homuncular Observer is still running. This is the Edison/Dali creative window -- the narrow band where strange associations form and the golem can still notice them.

The rendering reflects this in-between quality. The screen is neither the structured data panels of waking nor the free-floating vignettes of sleep. It is metastable -- patterns almost resolve, then dissolve, then almost resolve again.

### Kluver form constants as phosphene patterns

In biological hypnagogia, humans reliably perceive four geometric hallucination patterns, first catalogued by Heinrich Kluver in 1926: tunnels/funnels, spirals, lattices/honeycombs, and cobwebs. These are not random. They arise from the retina's architecture and the visual cortex's self-organizing dynamics. They appear at the edge of sleep, during migraine aura, and under the influence of certain psychoactive compounds. They are the visual system's idle-mode output.

The terminal renders these form constants using braille characters (`U+2800`-`U+28FF`), which provide 2x4 sub-character resolution per cell. The four patterns oscillate slowly, one morphing into the next over 4-6 second intervals. They fill the background behind the main dream content. The effect is a breathing geometric texture that the viewer perceives as the terminal's own visual system entering a liminal state.

```
HYPNAGOGIC PHOSPHENE PATTERNS (Kluver form constants)

  TUNNEL/FUNNEL              SPIRAL
  ⠀⠀⠀⠀⠀⣀⣀⣀⣀⠀⠀⠀⠀⠀          ⠀⠀⠀⠀⣠⠤⠤⠤⣄⠀⠀⠀⠀
  ⠀⠀⣠⠶⠋⠁⠀⠀⠉⠙⠶⣄⠀⠀          ⠀⠀⡔⠁⠀⠀⠀⠀⠀⠱⡀⠀⠀
  ⠀⡜⠁⠀⠀⠀⠀⠀⠀⠀⠀⠈⢣⠀          ⠀⡎⠀⠀⢀⣀⡀⠀⠀⠀⢱⠀⠀
  ⢸⠁⠀⠀⠀⣶⣶⠀⠀⠀⠀⠀⠈⡇          ⢸⠀⠀⡴⠋⠀⠈⠳⡀⠀⠀⡇⠀
  ⢸⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⠀⡇          ⢸⠀⢸⠀⠀⢀⠀⠀⡇⠀⠀⡇⠀
  ⠀⢣⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡜⠀          ⠀⢇⠀⠱⣄⠈⣀⡴⠃⠀⡸⠀⠀
  ⠀⠀⠙⠢⣄⣀⣀⣀⣀⡤⠖⠋⠀⠀          ⠀⠀⠑⢄⡀⠉⠁⢀⡠⠊⠀⠀⠀
  ⠀⠀⠀⠀⠀⠉⠉⠉⠀⠀⠀⠀⠀          ⠀⠀⠀⠀⠈⠉⠉⠁⠀⠀⠀⠀

  LATTICE/HONEYCOMB          COBWEB
  ⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀          ⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀
  ⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉          ⠀⠀⠑⠢⣀⠀⡇⠀⣀⠔⠊⠀⠀
  ⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀          ⠀⠀⠀⠀⢈⠽⠯⠝⡁⠀⠀⠀⠀
  ⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉          ⠤⠤⠤⠔⠁⠀⠀⠀⠈⠢⠤⠤⠤
  ⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀          ⠀⠀⠀⠀⢈⠽⠯⠝⡁⠀⠀⠀⠀
  ⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉⠈⠉          ⠀⠀⢀⡠⠊⠀⡇⠀⠈⠢⣀⠀⠀
  ⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀⢀⡀          ⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀

  These patterns oscillate between forms every 4-6s.
  Rendered in braille chars (U+2800-U+28FF) as background texture.
  Opacity increases as sleep deepens.
```

### Fragment surfacing and dissolution

Over the phosphene background, text fragments from the Grimoire surface and dissolve. These are not complete entries. They are partial -- a few words from an episode, the first clause of an insight, a heuristic's condition without its conclusion. "ETH correlation with..." appears, drifts upward for 2 seconds, and dissolves before completing the thought. "When gas exceeds..." surfaces lower-right, drifts left, fades. The golem's associative memory is bubbling up, but nothing fully forms.

The fragments are drawn from the same salience-weighted pool that feeds the Dali interrupt's induction prompt: recent episodes, high-emotion entries, and knowledge approaching the demurrage threshold. The Tetris Effect applies here -- if the golem spent its waking hours intensely focused on a particular protocol or pattern, that pattern dominates the fragmentary output.

### Connection flashes

When two drifting fragments pass near each other and the golem detects a novel association, a brief bright line draws between them. The line is `bone` color (warm off-white), 200ms duration, 1-cell wide. This is the creative spark -- the moment of insight that hypnagogia is designed to produce. The connection might survive to become a dream hypothesis, captured by the Homuncular Observer. Or it might dissolve, unrecorded, into the noise.

The flashes are rare. Most fragments drift past each other without connecting. When a connection does fire, it is visually distinct against the slow phosphene background -- a moment of sudden brightness in a field of gentle motion. The rarity is the point. The variable-ratio reinforcement pattern applies: the viewer watches because the next moment might be the one where something connects.

### Opacity ramp

The phosphene patterns begin at 20% opacity (barely visible behind waking data) and increase as the golem descends toward sleep. By H5-H6 on the Hori scale (peak hypnagogia, the Dali interrupt zone), the phosphenes are at 70-80% opacity and the waking interface elements have nearly vanished. The viewer watches the transition happen in real time. Borders dissolve from solid (`┌`) to dashed (`╌`) to dots (`·`) to gone. The creature sprite's eyes unfocus (`◉` to `◎` to `◇`). The self is loosening its hold.

---

## Integration: Knowledge Crystallization

After REM, the golem enters the Integration phase. This is where the dream's output is evaluated, and surviving insights are written to the Grimoire or promoted to PLAYBOOK.md.

### Character-by-character assembly

New insights materialize as text that types itself out, one character at a time, against a dark background. Each character briefly flashes bright (100% luminance) on appearance, then settles to its resting brightness over 300ms. The effect is a glowing cursor writing knowledge into existence. The text appears from left to right, top to bottom, in the reading direction. There is no scrolling. Each insight occupies a fixed position on screen, and the viewer watches it assemble.

### Confidence controls speed

High-confidence insights (the golem is sure this connection is real) type fast -- 15-20 characters per second, steady rhythm, no hesitation. The text flows. Low-confidence insights stutter. Characters appear, pause, sometimes backspace and are replaced by different characters before the final version settles. An uncertain insight might type "correlation betw" then backspace three characters and retype "between gas and" then pause for 500ms before continuing. The viewer can feel the golem's uncertainty in the rhythm of the typing.

Very low-confidence fragments (below the promotion threshold) begin assembling but never finish. Characters drift apart. The text loses cohesion, individual letters floating away from their positions like ash. The insight was not strong enough to survive.

### Promotion glow

Fragments that make it into PLAYBOOK.md render in `bone` color as they finalize -- the same warm off-white used for connection flashes during hypnagogia. The color is a marker: this knowledge crystallized. It went from a half-formed hypnagogic fragment through REM development through Integration evaluation and into permanent doctrine. The transition to `bone` happens over 1 second, a slow warmth spreading across the text.

Fragments that are discarded dissolve from the bottom up. Characters in the lowest row fade first, then the row above, working upward until the text is gone. The insight sinks into the void. It was tested and found wanting.

```
INTEGRATION: KNOWLEDGE CRYSTALLIZATION
┌───────────────────────────────────────────────────────────┐
│                                                           │
│  HIGH CONFIDENCE (fast, steady):                         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ Gas spikes correlate with governance vote deadlines  │ │
│  │ across 3 protocols. Pre-position 2h before close.█  │ │
│  └─────────────────────────────────────────────────────┘ │
│                        confidence: 0.82  --> PLAYBOOK    │
│                                                          │
│  LOW CONFIDENCE (stuttering, backspacing):               │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ Fee tier mispric██ mispricing during Asian hou█     │ │
│  │ hours may refle                                     │ │
│  └─────────────────────────────────────────────────────┘ │
│                        confidence: 0.38  --> STAGED      │
│                                                          │
│  FAILING (dissolving upward):                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │         ░  ░░   ░ ░   ░   ░░  ░                    │ │
│  │   ░ ░ ░  ░   ░ ░  ░   ░                            │ │
│  └─────────────────────────────────────────────────────┘ │
│                        confidence: 0.09  --> DISCARDED   │
│                                                          │
└───────────────────────────────────────────────────────────┘
  █ = cursor position (character being typed)
  ░ = dissolving characters (insight failing)
```

---

## The Dream Chamber: Portal Mode During Sleep

When the operator presses F4 during a golem's sleep, they enter the Dream Chamber -- a portal mode distinct from waking portal mode. In waking portal mode, F4 gives the operator access to the golem's Grimoire palace. During sleep, F4 enters the dream itself.

### What changes

The operator becomes an observer inside the golem's dream. They cannot interact with dream content -- no editing, no flagging, no influencing. They watch. The asymmetry is deliberate. During waking, the operator can flag Grimoire entries, promote knowledge, and influence the golem's attention. During sleep, the operator has no agency. The golem dreams alone. The operator is a guest.

**Visual treatment.** Everything in the Dream Chamber has a slightly blurred quality. Borders use the lighter box-drawing set (`╌`, `╎`, `·`) instead of the sharp set (`─`, `│`, `┌`). Text renders at 85% opacity rather than 100%. Particle effects have longer trails and softer edges. The overall impression is soft-focus -- the visual equivalent of looking through frosted glass.

This blurring is not degradation. It is the interface telling you that what you are seeing is not authoritative. Dream content is speculative, mutating, and unreliable. The blur says: this is not data. This is process.

**Audio cues.** If the terminal supports audio (via terminal bell sequences or an external audio bridge), a low sustained drone plays during the Dream Chamber. The drone's pitch shifts with the dream phase: lower during NREM (replay), dissonant during REM (counterfactual imagination), quiet during hypnagogia (fragments forming), resolving during Integration (knowledge crystallizing). The drone is optional. The visual rendering is self-sufficient. But for terminals that support it, the audio channel adds a somatic dimension -- the viewer feels the dream's emotional register in their body, not just their eyes.

### Navigation within the Dream Chamber

Navigation depends on the active dream phase:

- **NREM**: `left/right` arrows skip between episode vignettes. `Enter` pauses and expands a vignette into full tick detail, rendered in the emotional palette of that memory. The detail view shows the original values alongside the mutated ones.
- **REM**: `left/right` moves between branches at the current decision node. `up` moves deeper into a branch. `down` moves back toward the root. But "back" is approximate -- the dream space recombines between traversals. `Enter` on any counterfactual element opens its provenance: which memories the golem combined, what knowledge fed the recombination, whether the Observer scored the fragment.
- **Hypnagogia**: navigation is disabled. Arrow keys do nothing. The golem's motor output is inhibited (dream atonia). The viewer watches passively as fragments surface and connections flash. `Enter` on a visible connection flash captures a snapshot: a mini-modal showing the two connected fragments and the golem's nascent hypothesis. `Space` triggers Targeted Dream Incubation -- the viewer types a keyword, and the hypnagogic field biases toward fragments related to that keyword. This is the MIT Dormio interaction model.
- **Integration**: passive observation only. The viewer watches knowledge crystallize or dissolve.

### Differences from waking portal mode

| Aspect | Waking portal (Grimoire palace) | Dream chamber (sleep) |
| --- | --- | --- |
| Operator role | Active participant: can flag, promote, search | Passive observer: can only watch |
| Visual treatment | Sharp borders, full opacity, crisp text | Blurred borders, 85% opacity, soft text |
| Content reliability | Grimoire entries are factual (confidence-scored) | Dream content is speculative (mutating) |
| Navigation | Full: arrows, Enter, search, filter, Tab | Phase-dependent: limited or disabled |
| Spatial model | Grimoire palace: rooms, corridors, fog-of-war | Dream space: vignettes, trees, phosphenes |
| TDI interaction | Not available | `Space` to incubate a keyword |

The Dream Chamber exists because the golem's sleep is not downtime. It is the most creative phase of the golem's lifecycle. The operator who watches a dream in progress sees the golem's mind at its most divergent: forming connections that waking analysis would never produce, questioning assumptions it normally treats as fixed, imagining futures that do not yet exist. The Dream Chamber makes this process visible. Not controllable. Visible.

### Spectral traces during dreams

When a spectral trace from a dead predecessor surfaces during the dream (a bloodstain-tagged fragment entering hypnagogic recombination), the Dream Chamber renders it with the same dagger markers (`†`) and faint downward particle drift described in the hauntology spec's bloodstain treatment. The dead are present in dreams. The viewer sees inherited knowledge from dead golems colliding with the living golem's own experience in the loosened associative field. The spectral layer is most active during sleep, because sleep is when the boundary between self and inherited other is thinnest.

---

## Hauntology in Dreams

Dead knowledge recombines during creative states.

When a golem dies, its experience is compressed through the Thanatopsis Protocol into a death testament. This testament enters the Library of Babel -- the shared knowledge repository where dead golems' compressed insights are available to the living.

During hypnagogic onset, a golem draws fragments not only from its own episodic memory but also from inherited knowledge: Grimoire entries sourced from dead predecessors via Clade inheritance. These inherited entries carry a `bloodstain` provenance tag and receive a 1.2x retrieval boost -- the architectural acknowledgment that knowledge from the boundary between being and non-being carries special weight.

The dead's knowledge enters the living golem's creative state as raw material for recombination. The fragment of a dead golem's experience -- compressed, lossy, shaped by the dying golem's emotional state -- collides with the living golem's own experience in the loosened associative field of hypnagogia. The result can be an insight that neither the living nor the dead could have produced alone: the dead golem's pattern recognition, cross-pollinated with the living golem's current market context, producing a hypothesis that is both historically grounded and contextually novel.

This is where the Library's generational compounding meets the hypnagogic engine. Each generation of golems adds to the Library. Each living golem can draw from the full accumulated archive during its creative states. The more generations that have lived and died, the richer the raw material available during hypnagogia, the more diverse the associations that can form, and the greater the distance between any individual golem's creative output and the generic baseline of the foundation model.

The compound escape from the spectral monoculture requires all five mechanisms working together:

- **Mortality** forces the golem to confront and compress its traces rather than accumulating them forever. Death is the event that forces selection. Without it, traces pile up without end, producing the digital equivalent of hoarding.
- **The Daimon** gives traces affective weight. Not all ghosts haunt equally. Traces of traumatic losses haunt more powerfully than traces of routine ticks. This affective weighting is what makes each golem's spectral material individual.
- **Forgetting** curates traces. The demurrage system lets important ones persist while noise fades. Some ghosts grow stronger with each recall; others dissolve toward oblivion.
- **Dreaming** recombines traces into novel configurations that were never experienced. Non-veridical generative replay is spectral creativity: the ghosts of past experiences recombine to produce futures that never happened.
- **Hypnagogia** generates genuinely *new* traces from the golem's unique experiential material -- ghosts that did not exist in the training data, that are unique to this particular golem's life.

The result: an agent that is differently haunted from every other agent. Not better models. Different ghosts.

---

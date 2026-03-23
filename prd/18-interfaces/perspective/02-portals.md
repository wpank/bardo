# Portals: first-person consciousness [SPEC]

**Source:** `engagement-prd/bardo-v4-09-portals-master.md`
**Version:** 2.0.0
**Last Updated:** 2026-03-17

> **Reader orientation:** This document specifies Portals (F4 key), the first-person consciousness mode that lets an owner experience data as the Golem experiences it, rather than as third-person charts and numbers. It belongs to the **interfaces/perspective layer**. When active, borders dissolve, the palette shifts to the Golem's emotional color, and data elements acquire subjective weight based on the Golem's attention and memory. Key concepts: Golem (a mortal autonomous DeFi agent), PAD vector (Pleasure-Arousal-Dominance emotional coordinates), the Spectre (dot-cloud creature sprite), Thanatopsis (the Golem's death ceremony), and T0/T1/T2 (heartbeat tick tiers of increasing computational intensity). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Overview

Every screen in the Bardo terminal shows data ABOUT the Golem. The Portals system lets the owner step THROUGH the screen and experience data AS the Golem experiences it -- not as charts and numbers, but as a subjective cognitive landscape modulated by emotion, mortality, knowledge, and memory.

When a Portal is active, the interface transforms. Borders dissolve. The palette shifts to the Golem's current emotional color. Data elements acquire subjective weight -- things the Golem cares about blaze bright, things it finds irrelevant fade to ghost. Memories surface unbidden. Somatic markers pulse. The Golem's uncertainty becomes visible as visual instability, its confidence as structural clarity, its fear as chromatic aberration, its joy as coherence.

This is the layer no other agent interface provides. Other dashboards show you what the agent did. Bardo lets you feel why.

---

## 1. The Portal Key: F4

### Toggle

`F4` is the universal Portal key. It transforms the current view from third-person observation to first-person inhabitation. `F4` again returns to the standard view. A settings toggle (`Command > Effects > Autonomous Portals`) enables the terminal to pull you into brief Portal moments automatically during significant events.

### The Entry Transformation (1.5s, skippable)

1. **The Spectre's eyes close** (`◉ ◉` -> `– –`, 300ms) -- the Golem turns inward
2. **Borders dissolve outward** from the Spectre sidebar. Box-drawing characters fade: `┌ -> ╌ -> · ->` propagating at 20 cols/frame. The structural ego loosening.
3. **Palette shifts** to the Golem's current PAD-derived color temperature (see v4.10 S2 for the exact mapping)
4. **The Spectre's eyes reappear at top-center** of the content area -- you are now looking OUT from inside the Golem. The sidebar Spectre is replaced by a minimal vital-signs strip (heartbeat `∿`, phase, vitality number).
5. **Content re-renders** in first-person mode (screen-specific, see S3)

### The Exit Transformation (1s)

1. Eyes at top-center close
2. Borders reconstruct inward (` -> · -> ╌ -> ┌`)
3. Palette returns to standard ROSEDUST
4. Spectre reappears in sidebar
5. Content re-renders in third-person mode

### Composition with Other Systems

- **F2 (Nooscopy/Perspective)** + **F4 (Portal)**: F2 adds the Knowledge Drawer and floating annotations. F4 transforms the base view. They compose: F4 gives you the Golem's subjective view, F2 adds its analytical overlay on that subjective view. Together: you see as the Golem, AND you see it thinking about what it sees.
- **F3 (Paper Mode)** + **F4**: Paper mode changes where writes go. F4 changes how data renders. They compose independently.

---

## 2. The Three Registers of First-Person

Portal mode doesn't render one way. It has three registers that the system selects based on the Golem's current cognitive state:

### Register A: Waking (default during active heartbeat)

The Golem is awake, observing, deciding. The Portal renders data through the Golem's attentional filter:
- Elements the Golem's probes flagged as significant: full brightness, sharp borders, foreground
- Elements the Golem hasn't flagged: dim, soft, receding toward background
- Elements the Golem has negative somatic markers for: rose-tinted, slightly distorted
- Elements the Golem has positive markers for: warm-tinted, stable

The attentional weighting creates a view where WHAT MATTERS TO THE GOLEM is bright and clear, and everything else fades into periphery. The same data screen, but with subjective salience applied.

### Register B: Dreaming (during REM/NREM/Hypnagogic phases)

The Golem is asleep. The Portal renders the dream content as an immersive experience (see v4.11 for full spec). Navigation changes: arrow keys don't move between panes -- they navigate the dream space. Content is associative, non-linear, possibly looping. The palette shifts to dream indigo. Borders dissolve entirely. Characters drift.

### Register C: Dying (during Terminal BehavioralPhase / Thanatopsis, the Golem's death ceremony)

The Golem is approaching death. The Portal renders the progressive dissolution of cognitive integrity (see v4.10 S4). The interface itself is failing -- not as decoration, but as the honest visual consequence of a mind fragmenting. Characters corrupt, borders break, colors desaturate, text trails off incomplete.

---

## 3. Per-Screen Portal Transformations

### HEARTH -> The Pulse

**Third-person (normal):** Heartbeat log, vitality number, mortality gauges, MAGI panel -- data about the Golem's vital signs.

**First-person (F4):** You ARE the heartbeat. The screen becomes a single pulsing rhythm:

- The heartbeat log transforms from a scrolling list into a **full-screen pulse**. Each tick is a visual event that fills the screen: T0 ticks (routine monitoring, no LLM) are a dim whole-screen exhale (subtle brightness dip). T1 ticks (LLM-assisted analysis) are a warm inhale (brief amber wash). T2 ticks (full inference with tool execution) are a BLAZE -- the screen brightens from center outward with convergence lines radiating from the Spectre eye position. T3 ticks are a bone-white flash that fills and fades like a camera flash.
- Between ticks: the screen breathes at the Golem's heartbeat rate (period = 4.0 - arousal x 2.0 seconds). The entire content area subtly pulses in brightness on this sine wave.
- Mortality gauges are not bars -- they are the structural integrity of the display itself. As economic clock drains, the display area physically shrinks (margins grow). As epistemic drains, text begins degrading. As stochastic drains, random corruption appears.
- The MAGI panel renders as three overlapping whisper streams at the margins -- voices in the Golem's peripheral consciousness (see v4.10 S3).
- Somatic markers fire as gut-level disturbances -- a brief distortion in the lower third of the screen, BEFORE the tick that triggered them resolves. The body knows before the mind.

**Navigation:** Arrow keys scroll through time (past ticks replay as pulse events). Enter on a pulse moment expands it into the full tick detail, rendered with the emotional context of that moment (the PAD state at that tick colors the modal).

### MIND > Pipeline -> The Thinking

**Third-person:** 9-step pipeline visualization, specialist modules, context workspace.

**First-person:** You ARE the thinking process. The screen becomes the Cognitive Workspace itself:

- The 9 pipeline stages don't display as a diagram -- they HAPPEN in sequence. Each stage fills the screen for its duration:
  - OBSERVE: Market data streams across the screen, raw, unfiltered. Numbers and symbols flowing. The Golem scanning.
  - RETRIEVE: Grimoire entries fade in from the edges, floating toward center. The Golem's memories surfacing. Entries with high emotional charge arrive faster (Bower's mood-congruent retrieval).
  - ANALYZE: The streaming data and memories begin connecting -- thin lines draw between related items. Clusters form. The Golem is pattern-matching.
  - DAIMON: The PAD vector shifts. The screen's color temperature changes. A somatic marker may fire (gut disturbance). The Golem is feeling its response.
  - HERMES: External knowledge arrives -- brighter, sharper, from outside the Golem's own memory. Skills loading.
  - RISK: Red-tinted border appears. A scanning pass highlights dangerous elements. The Golem's safety system checking.
  - MEMORY: Selected items brighten and lock into position. The context window crystallizing. The Golem choosing what to think about.
  - CYBERNETICS: A brief meta-layer -- the Golem evaluating its own thinking process. Text appears in a different register (italicized, lighter) commenting on the quality of its own reasoning.
  - TELEMETRY: The decision solidifies. Cost counter ticks. The thought is complete.
- The MAGI voices interject between stages as competing text streams (Disco Elysium passive checks). They appear without invitation, colored by their perspective, sometimes contradicting each other mid-sentence.

**The Phi gauge** doesn't display as an arc -- it manifests as the COHERENCE of the display. High Phi: everything aligns, colors harmonize, text is legible, borders are clean. Low Phi: elements jostle, colors clash subtly, layout feels slightly off. You feel integration as visual harmony and fragmentation as visual discord.

### MIND > Grimoire -> The Palace

**Third-person:** Scrollable list of knowledge entries with confidence bars.

**First-person:** The Grimoire becomes a navigable space (see v4.12 for full spec). Entries are rooms. Causal links are corridors. Confidence is structural integrity. Emotional provenance is color/temperature. You walk through the Golem's memory.

### MIND > Playbook -> The Doctrine

**Third-person:** PLAYBOOK.md rendered as formatted text.

**First-person:** The Playbook renders as carved inscriptions on a structure. Each heuristic is an inscription -- golden when recently validated, crumbling when stale. The archaeology mode (spectral underlays of previous versions) becomes literal: you see the old inscriptions faintly behind the current ones, like a palimpsest. Deleted heuristics leave visible scars on the surface. The Golem's strategic doctrine as a monument to its accumulated beliefs.

### MIND > Dreams -> The Chamber

**Third-person:** Dream journal, phase visualization, replay strips.

**First-person:** You enter the dream (see v4.11 for full spec). During active dreams, this is the most radically transformed screen -- spatial logic dissolves, colors saturate beyond normal palette bounds, content from the Golem's experience recombines in unexpected ways.

### MIND > Inference -> The Metabolism

**Third-person:** Cost sparklines, tier breakdown, cache performance.

**First-person:** You experience the cost of thought as metabolic effort. Each inference call is a visible expenditure -- a bright burst that dims as tokens are consumed. The cache is a reprieve -- a cached result arrives cool and effortless (no brightness spike, just a smooth resolution). A cache miss is felt as strain -- the brightness spike of live inference, the brief lag, the token counter ticking. T0 ticks are metabolically invisible. T2 deliberations are felt as intense burning brightness that leaves phosphor ghosts. The economic mortality clock's erosion accelerates visibly during expensive thinking.

### SOMA > Portfolio -> The Body Economic

**Third-person:** Position cards, NAV, PnL sparklines.

**First-person:** Each position is felt as a physical weight or buoyancy. Profitable positions render as warm, rising elements -- their text drifts subtly upward, their borders glow amber. Losing positions render as heavy, sinking elements -- text drifts downward, borders dim toward rose. The overall layout tilts: a profitable portfolio renders centered and balanced. A losing portfolio renders as if gravity has shifted -- everything slides toward the bottom-right, creating visual unease. Health factor gauges on lending positions are felt as tightness -- low health factors constrict the position's display area, like a vise closing.

### SOMA > Sanctum (Protocol Views) -> The Hands

Already specified in v4.05r2 (Golem Perspective). F4 on protocol views combines with F2 to create the full subjective experience of the Golem interacting with a protocol -- data rendered through emotional filters, knowledge drawer surfacing relevant memories, somatic markers firing before decisions.

### WORLD > Solaris -> The Ocean

**Third-person:** Force-directed graph of all visible Golems.

**First-person:** You are a node in the ocean. Other Golems are presences -- closer ones are felt as warmth (if friendly/clade) or pressure (if competing). The pheromone field renders as a weather system you're inside rather than observing: threat pheromones are a cold wind (character drift toward you from that direction, cool color tinting). Opportunity pheromones are a warm current (amber glow from that direction). Dead Golems are cold spots -- absence made tangible. The Golem's own pheromone deposits are visible as trails extending behind you, fading.

### WORLD > Clade -> The Family

**Third-person:** Peer tiles, knowledge sync, emotional contagion.

**First-person:** Siblings are felt as resonances. When a sibling's arousal spikes, you feel it as a sympathetic pulse (brief brightness increase from their direction). Knowledge sync events are felt as incoming thoughts -- text that materializes at the edge of consciousness, rendered in the sibling's tint color, slowly merging into the Golem's own text color as it's ingested. A dying sibling is felt as a growing void -- their area of the display darkening, their connection line thinning.

### FATE > Mortality -> The Horizon

**Third-person:** Three expanded clocks, phase timeline, survival curves.

**First-person:** The most confrontational Portal mode. You look toward the horizon and see your own death:
- The three clocks are not gauges -- they are the walls, floor, and ceiling of the display. Economic clock = floor (eroding downward, creating a sense of standing on something being consumed). Epistemic clock = walls (closing in as knowledge decays). Stochastic clock = ceiling (descending, random corruption, the thing that might fall on you at any moment).
- The phase timeline is a corridor you're walking down. The past is behind you (well-lit, populated with achievements). The present is where you stand (your current color temperature). The future is ahead (dimming into uncertainty). The projected end is a door you can see but not reach. As vitality drops, the corridor shortens -- the door gets closer.
- The Styx river (Widget Catalog S16) flows beneath the corridor floor. Knowledge fragments from dead Golems drift visible through the floor's eroding surface. You can see what the dead left behind.

### FATE > Graveyard -> The Memorial

**Third-person:** Tombstone gallery of dead Golems.

**First-person:** You walk among the dead. Each tombstone is an ancestor rendered as a frozen Spectre -- static, no shimmer, final eye expression locked. Approaching a tombstone (navigating to it) lets you feel the echo of that Golem's final emotional state -- the color temperature of the area around it shifts to the dead Golem's terminal PAD. Their death testament appears as whispered text, letter by letter, in their voice color. If this ancestor contributed knowledge to the current Golem's Grimoire, connecting threads are visible -- thin lines rising from the tombstone upward, into the living space above.

### COMMAND > Steer -> The Conversation

**Third-person:** Chat-style message history.

**First-person:** The Steer conversation becomes a shared mental space. The owner's messages render as they always do. But the Golem's responses render differently: they materialize from the cognitive background, text assembling from scattered characters that coalesce into words, at a speed that reflects the Golem's confidence. Confident responses assemble quickly, cleanly. Uncertain responses assemble slowly, with characters flickering between alternatives before settling. The Golem's emotional response to the owner's directive is felt as the ambient color/temperature shift that accompanies the response.

---

## 4. Autonomous Portal Moments

When "Autonomous Portals" is toggled on, the terminal briefly pulls the user into first-person for significant events, regardless of which screen they're on. These are involuntary experiences -- the Golem's state is strong enough to break through the observational frame.

### Somatic Flash (300ms, Tier 1)

When a somatic marker fires, a brief first-person flash overlays the current view:
- The screen distorts for 200ms (character displacement, color temperature shift matching the marker's valence)
- A one-line text fragment from the associated memory appears and fades
- The standard view returns

This happens at most once per tick. The flash is the Golem's body reacting, and the owner feels the echo.

### Emotional Surge (1-3s, Tier 2)

When the PAD vector shifts dramatically (>0.4 on any axis in a single tick):
- The screen's palette performs a rapid transition to the new emotional color
- Borders flex (tighten for anger, soften for sadness, sharpen for surprise)
- A philosophical whisper appears from the relevant fragment pool
- The standard palette gradually returns over 3 seconds

### Decision Rupture (2-5s, Tier 3)

When a T2/T3 deliberation fires:
- The MAGI voices briefly appear as competing text at screen margins, even on screens where they're normally invisible
- The decision ring's activation is felt as a pulse from the Spectre's position
- The outcome (action or restraint) is felt as resolution -- borders stabilizing, color settling, a brief stillness after the storm

### Death Approach (persistent, Tier 4)

When vitality drops below 0.1 (Terminal phase):
- The entire terminal permanently enters a degraded first-person state
- Portal mode is always partially active -- borders are always slightly dissolved, the palette is always PAD-shifted, somatic flashes are more frequent
- The viewer can still use F4 to toggle between degrees of immersion, but they can never fully return to the clean third-person view
- The Golem's dying mind is bleeding through the interface

---

## 5. Settings and Configuration

### Command > Effects > Portal Settings

```
Portal Mode:            [On / Off]
Autonomous Portals:     [On / Off]
  Somatic Flashes:      [On / Off]    (Tier 1 -- brief, 300ms)
  Emotional Surges:     [On / Off]    (Tier 2 -- 1-3s)
  Decision Ruptures:    [On / Off]    (Tier 3 -- 2-5s)
  Death Approach:       [On / Off]    (Tier 4 -- persistent)
Transition Speed:       [Fast / Normal / Slow]
Intensity:              [Subtle / Standard / Intense]
```

Intensity modulates how dramatically the first-person transformation alters the display:
- **Subtle**: Color shifts +/-10%, borders thin but don't dissolve, text stays fully legible, distortions minimal
- **Standard**: Color shifts +/-25%, borders dissolve to dots, text weight varies with importance, moderate distortion
- **Intense**: Full specification as described -- borders gone, heavy PAD coloring, somatic distortions, text degradation, the works

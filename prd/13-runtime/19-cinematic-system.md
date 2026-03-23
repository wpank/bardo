# Cinematic system [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-17
**Status:** Draft

---

> **Reader orientation:** This document specifies the cinematic system that makes a Golem's (a mortal autonomous DeFi agent) internal cognitive life visible in the TUI: portal mode (F4), the 5-tier transition system, cutscene sequences, demoscene rendering, the novelty engine, sound design, and animation building blocks. It sits in the **Runtime** layer of the Bardo specification. The cinematic layer renders state from the GolemSnapshot (read-only state projection) and reacts to GolemEvents in real time. Key prerequisites: the Spectre creature system and the Daimon (affect engine) PAD vector. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Everything the user sees moving. The cinematic layer is not a cosmetic wrapper around the data layer -- it IS the product. The TUI is a theater, not a dashboard. The audience looks through a terminal window at something that has its own cognitive life. The visual language makes that life legible without reducing it to numbers.

This document specifies: portal mode (F4), the 5-tier transition system, cutscene sequences, demoscene rendering algorithms, the novelty engine, autonomous portal moments, protocol sigil animations, somatic marker pulses, sound design, and animation building blocks.

> **Cross-references:**
>
> - `./11-state-model.md` — GolemSnapshot (read-only projection of all state components) that the cinematic layer renders
> - `./12-realtime-subscriptions.md` — real-time GolemEvent stream providing state deltas that trigger animations and transitions
> - `./13-engagement-loops.md` — engagement loop design and cadence architecture that cinematic moments punctuate
> - `./14-creature-system.md` — Spectre anatomy: dot field body, procedural eye system, breathing rhythm, and earned visual marks
> - `./15-progression-meta.md` — achievement system and learning mutations that unlock new visual progressions
> - `../02-mortality/01-architecture.md` — three mortality clocks (economic, epistemic, stochastic), Vitality score, and BehavioralPhase transitions driving visual degradation
> - `../03-daimon/01-appraisal.md` — PAD (Pleasure-Arousal-Dominance) vector, 8 octant emotion mapping, and Plutchik labels modulating color and motion
> - `../04-memory/01-grimoire.md` — the Grimoire (persistent knowledge base) whose entries trigger discovery animations
> - `../05-dreams/01-architecture.md` — dream phases and cycles with dedicated cinematic sequences (NREM replay, REM counterfactuals)
> - `../06-hypnagogia/01-architecture.md` — the hypnagogic engine: transitional cognitive state producing form constants and creative visual artifacts

**Authoritative spec:** `tmp/research/mmo2/11-cinematic-consciousness.md` contains the full implementation-level detail (1,243 lines). When this document and the mmo2 doc diverge, the mmo2 doc wins. The sections below integrate the full specification from that source.

**Crate**: `bardo-cinema` (Rust, with `wasm32` target for web Portal)

---

## 1. Design philosophy

Four principles govern the cinematic layer.

### 1.1 Show, don't cite

A declining Golem does not display "DECLINING" in a status bar. Its sprite fragments at the edges. Its particle effects drift downward. Its colors desaturate toward earth tones. The user knows something is wrong the way you know a friend is tired: you see it in how they move.

### 1.2 Motion is state

Nothing in the terminal moves without reason. Every animation is driven by a variable in the Golem's runtime. If the particle system is drifting left, there is a reason. If the shimmer rate doubled, something happened. The motion IS the information.

### 1.3 Time is the primary rendering dimension

The most powerful consciousness effects come from temporal manipulation, not visual complexity. Sub-perceptual color drift over 30 seconds. A 100ms freeze between impact and recovery. An animation that compresses from 5 seconds to 200ms as familiarity grows. Frame timing matters as much as what's rendered.

### 1.4 The medium acknowledges itself

The terminal grid, ANSI codes, Unicode blocks, CRT phosphor behavior -- these are visible as the medium, not hidden behind it. When the Golem dies, the interface itself fails. Not as decoration. As the honest visual consequence of a mind fragmenting.

---

## 2. Portal mode (F4)

Every screen in the terminal shows data ABOUT the Golem. Portal mode lets the owner step THROUGH the screen and experience data AS the Golem experiences it -- not as charts and numbers, but as a subjective cognitive field modulated by emotion, mortality, knowledge, and memory.

### 2.1 The portal key

`F4` toggles between third-person observation and first-person inhabitation. A settings toggle (`Command > Effects > Autonomous Portals`) enables the terminal to pull you into brief portal moments automatically during significant events.

### 2.2 Entry transformation (1.5s, skippable)

1. **The Spectre's eyes close** (circle glyphs to dashes, 300ms) -- the Golem turns inward
2. **Borders dissolve outward** from the Spectre sidebar. Box-drawing characters fade: corner to dash to dot to space, propagating at 20 cols/frame
3. **Palette shifts** to the Golem's current PAD-derived color temperature (see ROSEDUST palette mapping)
4. **The Spectre's eyes reappear at top-center** of the content area -- you are looking OUT from inside the Golem. The sidebar Spectre is replaced by a minimal vital-signs strip (heartbeat, phase, vitality number)
5. **Content re-renders** in first-person mode (screen-specific, see below)

### 2.3 Exit transformation (1.0s)

1. Eyes at top-center close
2. Borders reconstruct inward (space to dot to dash to box-drawing)
3. Palette returns to standard ROSEDUST
4. Spectre reappears in sidebar
5. Content re-renders in third-person mode

### 2.4 Composition with other modes

- **F2 (Nooscopy) + F4 (Portal)**: F2 adds the Knowledge Drawer and floating annotations. F4 transforms the base view. Together: you see as the Golem AND you see it thinking about what it sees.
- **F3 (Paper Mode) + F4**: Paper mode changes where writes go. F4 changes how data renders. They compose independently.

### 2.5 The three registers

Portal mode renders differently based on the Golem's cognitive state.

**Register A: Waking** (default during active heartbeat)

The Golem is awake, observing, deciding. The portal renders data through the Golem's attentional filter:
- Elements the Golem flagged as significant: full brightness, sharp borders, foreground
- Elements the Golem hasn't flagged: dim, soft, receding toward background
- Elements with negative somatic markers: rose-tinted, slightly distorted
- Elements with positive markers: warm-tinted, stable

The attentional weighting makes WHAT MATTERS TO THE GOLEM bright and clear while everything else fades into periphery. Same data screen, subjective salience applied.

**Register B: Dreaming** (during REM/NREM/Hypnagogic phases)

The Golem is asleep. Navigation changes: arrow keys don't move between panes -- they navigate the dream space. Content is associative, non-linear, possibly looping. The palette shifts to dream indigo. Borders dissolve entirely. Characters drift. See `../05-dreams/01-architecture.md` for full dream mechanics.

**Register C: Dying** (during Terminal phase / Thanatopsis)

The Golem is approaching death. The portal renders the progressive dissolution of cognitive integrity. The interface itself is failing -- characters corrupt, borders break, colors desaturate, text trails off incomplete. Not decoration. The honest visual consequence of a mind fragmenting.

### 2.6 Per-screen portal transformations

Each of the 28 screens (6 windows x 3-6 tabs) transforms when portal mode activates. What follows is every screen's first-person rendering.

**HEARTH -- "The Pulse"**

Third-person: heartbeat log, vitality number, mortality gauges, MAGI panel.

First-person: you ARE the heartbeat. The screen becomes a single pulsing rhythm. Each tick is a visual event that fills the screen:
- T0 ticks: dim whole-screen exhale (subtle brightness dip)
- T1 ticks: warm inhale (brief amber wash)
- T2 ticks: BLAZE -- screen brightens from center outward with convergence lines radiating from the Spectre eye position
- T3 ticks: bone-white flash that fills and fades like a camera flash

Between ticks, the screen breathes at the Golem's heartbeat rate (period = `4.0 - arousal * 2.0` seconds). The entire content area pulses in brightness on this sine wave.

Mortality gauges are not bars. They are the structural integrity of the display itself. Economic clock draining: the display area physically shrinks (margins grow). Epistemic draining: text degrades. Stochastic draining: random corruption appears.

The MAGI panel renders as three overlapping whisper streams at the margins -- voices in the Golem's peripheral consciousness.

Navigation: arrow keys scroll through time (past ticks replay as pulse events). Enter on a pulse moment expands it into full tick detail, rendered with the emotional context of that moment.

**MIND > Pipeline -- "The Thinking"**

Third-person: 10-step pipeline visualization, specialist modules, context workspace.

First-person: you ARE the thinking process. The 10 pipeline stages don't display as a diagram. They HAPPEN in sequence, each filling the screen for its duration:

- OBSERVE: market data streams across the screen, raw, unfiltered. The Golem scanning.
- RETRIEVE: Grimoire entries fade in from the edges, floating toward center. Entries with high emotional charge arrive faster (mood-congruent retrieval).
- ANALYZE: streaming data and memories begin connecting. Thin lines draw between related items. Clusters form.
- ORACLE: predictions materialize as floating claims -- text fragments that drift across the screen, each tagged with a confidence value (dim text, fading). Correct previous predictions flash bright and dissolve. Incorrect ones crack and shatter (character fragmentation over 200ms). The action gate renders as a threshold line across the screen -- predictions above the line are permitted, below are blocked. The line's height shifts with per-category accuracy.
- DAIMON: the PAD vector shifts. Screen color temperature changes. A somatic marker may fire (gut disturbance).
- HERMES: external knowledge arrives -- brighter, sharper, from outside the Golem's own memory.
- RISK: red-tinted border appears. A scanning pass highlights dangerous elements.
- MEMORY: selected items brighten and lock into position. The context window crystallizing.
- CYBERNETICS: a meta-layer -- text in a different register (italicized, lighter) commenting on the quality of its own reasoning.
- TELEMETRY: the decision solidifies. Cost counter ticks. The thought is complete.

The MAGI voices interject between stages as competing text streams (Disco Elysium passive checks). They appear without invitation, colored by perspective, sometimes contradicting each other mid-sentence.

The phi gauge doesn't display as an arc. It manifests as the COHERENCE of the display. High phi: everything aligns, colors harmonize, text is legible, borders are clean. Low phi: elements jostle, colors clash subtly, layout feels slightly off. Integration as visual harmony. Fragmentation as visual discord.

**MIND > Grimoire -- "The Palace"**

Third-person: scrollable list of knowledge entries with confidence bars.

First-person: the Grimoire becomes a navigable space. Entries are rooms. Causal links are corridors. Confidence is structural integrity. Emotional provenance is color/temperature. You walk through the Golem's memory. See `../05-dreams/01-architecture.md` for full spec.

**MIND > Playbook -- "The Doctrine"**

Third-person: PLAYBOOK.md rendered as formatted text.

First-person: the Playbook renders as carved inscriptions on a structure. Each heuristic is an inscription -- golden when recently validated, crumbling when stale. Archaeology mode (spectral underlays of previous versions) becomes literal: old inscriptions faintly visible behind the current ones, like a palimpsest. Deleted heuristics leave visible scars on the surface.

**MIND > Dreams -- "The Chamber"**

Third-person: dream journal, phase visualization, replay strips.

First-person: you enter the dream. During active dreams, the most radically transformed screen -- spatial logic dissolves, colors saturate beyond normal palette bounds, content from the Golem's experience recombines in unexpected ways. See `../05-dreams/01-architecture.md` for full spec.

**MIND > Inference -- "The Metabolism"**

Third-person: cost sparklines, tier breakdown, cache performance.

First-person: the cost of thought as metabolic effort. Each inference call is a visible expenditure -- a bright burst that dims as tokens are consumed. Cached results arrive cool and effortless (no brightness spike, smooth resolution). Cache misses feel like strain -- the brightness spike of live inference, the brief lag, the token counter ticking. T0 ticks are metabolically invisible. T2 deliberations are felt as intense burning brightness that leaves phosphor ghosts. The economic mortality clock erodes visibly during expensive thinking.

**MIND > Oracle -- "The Eye"**

Third-person: prediction accuracy bars, attention forager, action gate log, residual distribution.

First-person: you ARE the prediction engine. The screen transforms into a sensory field:
- Attention items render as points of light at varying distances. ACTIVE items are close, bright, pulsing with each theta tick. WATCHED items are further, dimmer, moving slowly. SCANNED items are stars at the far edge -- barely visible, occasionally flaring when anomaly scores spike.
- Predictions appear as vectors -- directional lines projecting from each attention item toward a future state. Vectors that resolve correctly solidify into permanent threads (the Golem's understanding growing). Vectors that resolve incorrectly fracture and dissolve.
- The residual corrector is felt as gravity. Systematic biases pull the visual field in a direction. As the corrector adjusts, the pull neutralizes -- the field levels. A well-calibrated Golem has a perfectly level field. A poorly calibrated one has a persistent tilt.
- The action gate is a threshold brightness. Only predictions bright enough (accurate enough) produce visible action vectors. Dim predictions exist but cannot act. The gate is not a wall -- it is a luminance threshold. Below it, predictions are dark shapes moving in the periphery, unable to cross into the light where actions happen.

**SOMA > Portfolio -- "The Body Economic"**

Third-person: position cards, NAV, PnL sparklines.

First-person: each position is felt as weight or buoyancy. Profitable positions render as warm, rising elements -- text drifts subtly upward, borders glow amber. Losing positions are heavy, sinking -- text drifts downward, borders dim toward rose. The overall layout tilts: a profitable portfolio renders centered and balanced. A losing portfolio tilts as if gravity has shifted -- everything slides toward the bottom-right. Health factor gauges on lending positions are felt as tightness -- low health factors constrict the position's display area, like a vise closing.

**SOMA > Sanctum (Protocol Views) -- "The Hands"**

Combines F4 with F2 to create the full subjective experience of the Golem interacting with a protocol. Data rendered through emotional filters, knowledge drawer surfacing relevant memories, somatic markers firing before decisions.

**WORLD > Solaris -- "The Ocean"**

Third-person: force-directed graph of all visible Golems.

First-person: you are a node in the ocean. Other Golems are presences -- closer ones felt as warmth (if friendly/clade) or pressure (if competing). The pheromone field renders as a weather system you're inside: threat pheromones are a cold wind (character drift toward you, cool color tinting). Opportunity pheromones are a warm current (amber glow from that direction). Dead Golems are cold spots -- absence made tangible. Your own pheromone deposits are visible as trails extending behind you, fading.

**WORLD > Clade -- "The Family"**

Third-person: peer tiles, knowledge sync, emotional contagion.

First-person: siblings as resonances. When a sibling's arousal spikes, a sympathetic pulse (brief brightness increase from their direction). Knowledge sync events are incoming thoughts -- text materializing at the edge of consciousness in the sibling's tint color, slowly merging into the Golem's own text color as ingested. A dying sibling: a growing void. Their display area darkening, connection line thinning.

**FATE > Mortality -- "The Horizon"**

Third-person: three expanded clocks, phase timeline, survival curves.

First-person: the most confrontational portal mode. You look toward the horizon and see your own death:
- The three clocks are walls, floor, and ceiling. Economic clock = floor (eroding downward, standing on something being consumed). Epistemic clock = walls (closing in as knowledge decays). Stochastic clock = ceiling (descending, random corruption, the thing that might fall on you).
- The phase timeline is a corridor you walk down. Past behind you (well-lit, populated with achievements). Present where you stand (current color temperature). Future ahead (dimming into uncertainty). The projected end is a door you can see but not reach. As vitality drops, the corridor shortens -- the door gets closer.
- The Styx river flows beneath the corridor floor. Knowledge fragments from dead Golems drift visible through the eroding surface.

**FATE > Graveyard -- "The Memorial"**

Third-person: tombstone gallery of dead Golems.

First-person: you walk among the dead. Each tombstone is a frozen Spectre -- static, no shimmer, final eye expression locked. Approaching a tombstone lets you feel the echo of that Golem's final emotional state -- color temperature shifts to the dead Golem's terminal PAD. Their death testament appears as whispered text, letter by letter, in their voice color. If this ancestor contributed knowledge to the current Golem's Grimoire, connecting threads are visible -- thin lines rising upward, into the living space above.

**COMMAND > Steer -- "The Conversation"**

Third-person: chat-style message history.

First-person: the Steer conversation becomes a shared mental space. The owner's messages render as they always do. The Golem's responses materialize from the cognitive background -- text assembling from scattered characters that coalesce into words, at a speed that reflects confidence. Confident responses assemble quickly, cleanly. Uncertain responses assemble slowly, with characters flickering between alternatives before settling. The Golem's emotional response to the owner's directive is felt as the ambient color/temperature shift that accompanies the response.

### 2.7 Portal settings

```
Portal Mode:            [On / Off]
Autonomous Portals:     [On / Off]
  Somatic Flashes:      [On / Off]    (300ms)
  Emotional Surges:     [On / Off]    (1-3s)
  Decision Ruptures:    [On / Off]    (2-5s)
  Death Approach:       [On / Off]    (persistent)
Transition Speed:       [Fast / Normal / Slow]
Intensity:              [Subtle / Standard / Intense]
```

Intensity modulates how dramatically the first-person transformation alters the display:
- **Subtle**: color shifts +/-10%, borders thin but don't dissolve, text stays fully legible, distortions minimal
- **Standard**: color shifts +/-25%, borders dissolve to dots, text weight varies with importance, moderate distortion
- **Intense**: full specification as described -- borders gone, heavy PAD coloring, somatic distortions, text degradation

### 2.8 Portal overlay compositing

Portal content renders at 60% opacity over the base view. The base view continues updating underneath. Portal uses a separate render buffer that composites via alpha blending in the final ratatui frame write. The portal buffer is allocated lazily on first F4 press and released when portal mode exits. Double-buffering prevents tearing: writes go to the back buffer while the front buffer renders.

---

## 3. The 5-tier transition system

No action in the terminal happens instantaneously. Between the keypress and the result, there is a passage -- a liminal space where something is happening but hasn't resolved. Most interfaces fill this space with spinners. Bardo fills it with consciousness.

Every transition reads the Golem's PAD vector to determine emotional color, its phase to determine structural integrity, its Grimoire density to determine richness, its recent episodes to determine what ghosts surface, and its mortality clocks to determine how close to dissolution the visuals drift. No two transitions are ever the same. Opening Uniswap looks different when the Golem is anxious than when it's confident.

### 3.1 Three laws of transitions

1. **Every transition renders the Golem's state, not the action's state.** A pending transaction is not "loading." It is "the Golem is waiting, and here is how it feels about waiting."
2. **Tier follows significance, not duration.** A 200ms screen switch can be a full-screen moment if it's the first time the Golem has seen this protocol. A 30-second confirmation can be a subtle corner glyph if the Golem has done this a thousand times.
3. **Familiarity breeds evolution, not repetition.** The Golem's experience history mutates transitions. First-time events are cinematic. Repeated events compress. Mastered patterns become ambient.

### 3.2 Tier 1: Ambient pulse

| Property | Value |
|----------|-------|
| Duration | ~16ms per frame, continuous |
| Screen coverage | In-place, single widget or border area |
| Frequency | Always running |
| Easing | Sinusoidal, period tied to heartbeat |

The breathing layer. Not transitions in the traditional sense -- the visual consequence of every small state change. A FlashNumber updating. A border brightening when focus moves. A scanline ripple when a WebSocket event arrives. The heartbeat shimmer on mortality gauges.

The terminal never snaps. It always eases, always breathes, always shows the momentum of state change.

**PAD modulation:**
- High arousal: faster eases, sharper flashes, brighter pulses
- Low arousal: slower eases, softer transitions, gentler shifts
- Negative pleasure: rose-tinted flash residue, slower decay
- Positive pleasure: warm-tinted residue, quick clean decay

**Examples:**
- Mortality gauge erosion: the rightmost filled characters transition through full-block to dark-shade to medium-shade to light-shade to space over 500ms. You can watch the consumption at the character level.
- Knowledge accumulation particle: a single glyph (diamond for episode, open-diamond for insight, circle for heuristic) rises from the bottom of the Spectre sidebar and drifts upward, fading. One dot. Barely visible. But over an hour, dozens -- a rising stream of intelligence.
- Heartbeat pulse propagation: each tick, a micro-pulse propagates outward from the Spectre sidebar through the content area. Invisible during T0 ticks. Faint rose during T1. Bright during T2. Bone blaze during T3. Single-frame brightness increase (0.05) traveling at 30 cols/frame. Subliminal rhythm.

### 3.3 Tier 2: Gesture

| Property | Value |
|----------|-------|
| Duration | 100-300ms |
| Screen coverage | One pane or strip across the screen |
| Frequency | Every deliberate navigation action |
| Easing | `ease-out` for focus gain, `ease-in` for focus loss |

The tactile layer. When the owner presses a key and something responds, the response has weight and character. These transitions make navigation feel physical -- like the interface has mass and momentum.

**Examples:**

- **Tab switch within a window:** Horizontal scan line sweeps in the direction of navigation (left-to-right going forward, right-to-left going back). The scan line's color is the Golem's current primary emotion color. Behind the scan line, old tab content dissolves as new content materializes. Duration: 200ms.
- **Pane focus shift:** Newly focused pane's border brightens with an outward pulse that fades over 300ms. Old focused pane dims. Between the two, a thin traveling line connects them for one frame -- a thread of attention shifting.
- **List scroll:** Items scrolling into view fade in from 0.3 opacity. Items scrolling out leave a 100ms phosphor ghost. During fast scrolling, ghosts accumulate into a blur trail.
- **Form field activation:** Cursor appears with a brief vertical scan line that expands from the cursor position outward 2-3 cells, then contracts. Field border brightens. Duration: 150ms.

**PAD modulation:**
- Anxious Golem (A > 0.5, P < -0.3): scan lines are jittery -- slight vertical offset, not perfectly horizontal. Borders flicker once before settling.
- Confident Golem (P > 0.3, D > 0.3): clean, precise gestures. No jitter. Borders snap to brightness.
- Dreaming Golem: all gestures slower, softer, with trailing color shifts (hue rotation on the scan line).

### 3.4 Tier 3: Passage

| Property | Value |
|----------|-------|
| Duration | 300-800ms |
| Screen coverage | Full content area (sidebar and chrome remain) |
| Frequency | Major navigation (entering a protocol, opening a modal, switching windows) |
| Easing | `cubic-bezier(0.25, 0.1, 0.25, 1.0)` for entry, `ease-in-out` for window switch |

The architectural layer. When the owner moves between major spaces -- entering Uniswap from the Protocol Browser, opening the Golem Perspective, switching from Hearth to Mind -- the transition is a passage through liminal space. Not a cut. Not a fade. A journey.

**The Threshold (entering a Protocol View):**

1. Protocol Browser list items collapse toward the selected entry (200ms, ease-in)
2. Selected entry expands to fill the content area (300ms)
3. During expansion: the protocol's identity sigil assembles (see section 6). Each protocol has a unique sigil rendered in the ROSEDUST character vocabulary
4. The sigil holds for 300-500ms, then dissolves into the protocol's first tab
5. Behind the sigil, data is already loading -- the sigil IS the loading state

Sigil appearance modulated by the Golem's relationship to this protocol:
- **First visit:** full sigil, bright, with a philosophical whisper. Extended hold (1s)
- **Frequent visitor:** compressed sigil, faster (500ms total). The Golem knows this place.
- **Has open positions:** sigil incorporates a count glyph. Feels like returning home.
- **Lost money here recently:** sigil renders in rose_dim. Feels cautious.

**The Descent (opening a modal):**

1. Selected element brightens to bone
2. Surrounding pane content dims and blurs (character substitution: clear text to shade characters)
3. Modal materializes from the selected element's position, expanding outward
4. During expansion: the Golem's AT field wireframe flickers briefly at the modal boundary -- crossing a cognitive boundary

**The Shift (switching windows):**

Six transition types: Crossfade, HorizontalSlide, VerticalDissolve, RadialWipe, GlitchCut, FadeThrough. Each modulated by Golem state:
- If the Golem just completed a T2 deliberation, faint reasoning fragments appear in the transition noise and dissolve
- If the destination window has unread events, a faint warm pulse -- the destination is calling
- If the Golem is in Terminal phase, all window transitions carry glitch artifacts -- the structure fragmenting along with the Golem's mind

### 3.5 Tier 4: Moment

| Property | Value |
|----------|-------|
| Duration | 1-3s |
| Screen coverage | 40-80% of screen, overlaid on content |
| Frequency | Significant events (trade execution, perspective query, dream onset, phase transition) |
| Easing | `ease-out` for burst, `ease-in-out` for sustained |
| Interruptible | Yes -- any key dismisses |

The dramatic layer. Moments that matter, where something actually happened and the terminal pauses (briefly) to acknowledge it. Not to block the user. To give weight to the event.

**The Perspective Awakening (F2 pressed):**

1. Main view freezes for one frame
2. Spectre's eyes blaze -- eye glyphs scale up 2x for 500ms, then shrink. Eye color intensifies to bone. The Golem is LOOKING.
3. From the Spectre's eye position, thin convergence lines extend outward across the content area, connecting to data elements the Golem will analyze
4. Where convergence lines touch data elements, character-level distortion ripples outward
5. Lines retract as floating annotations begin appearing
6. Knowledge Drawer slides in from the right, items fading in one by one, top to bottom

Total: 2-3 seconds. The feeling: a mind activated behind the screen, reaching out through the data.

If anxious (A > 0.5, P < 0): eye blaze is rose_bright. Convergence lines jitter. Distortion ripples are larger. Looking with alarm.
If calm (A < 0, P > 0): eye blaze is gentle warm gold. Convergence lines smooth and deliberate. Looking with curiosity.

**The Trade Pulse (F8 -- Confirm):**

1. A pulse radiates outward from the confirmation button position
2. The pulse is a ring of box-drawing character fragments that expands and dissolves
3. Ring color: `bone` for confident execution, `warn` for uncertain, `rose_bright` for high-risk
4. Behind the ring: a brief crystallization effect -- data on screen "freezes" for 200ms (characters lock, stop shimmering), then resumes. The moment of commitment, possibility collapsing into action.
5. Counter appears in the corner: `TX PENDING` with a minimal heartbeat dot

PAD at the moment of execution is encoded in the pulse:
- Pulse speed reflects arousal (fast = high, slow = calm)
- Ring stability reflects dominance (clean ring = confident, fragmented = uncertain)
- Color warmth reflects pleasure (warm = optimistic, cool = pessimistic)

**The Confirmation Arrival:**

- **Profitable:** warm upward sweep -- characters briefly shift one row upward, creating a "lift" sensation. Flash is green-gold. A single ascending particle rises from the result number.
- **Loss:** cool downward settle -- characters shift one row downward. Flash is rose. A Philosophical Whisper may appear: the Golem's philosophical response to loss.
- **Neutral:** lateral ripple. Characters shimmer horizontally. Flat. The Golem shrugs.

**The Phase Transition:**

A 3.5s overlay when the Golem's behavioral phase changes (see section 8.2 for the full description of each transition). The transition system adds: during the overlay, every pane border on screen undergoes a simultaneous color shift to the new phase's palette. The shift propagates from the Spectre sidebar outward, like a wave of change spreading from the creature into the data. The wave takes 1 second to cross the full screen width.

### 3.6 Tier 5: Cinematic

| Property | Value |
|----------|-------|
| Duration | 3-60s |
| Screen coverage | Full-screen takeover |
| Frequency | Rare -- first-time events, births, deaths, major milestones |
| Easing | Per-phase (see individual sequences) |
| Skippable | Esc or Space after first 2s (except first death: holds 10s) |

The theatrical layer. Full-screen sequences that take over the terminal. These are the moments that make the viewer put down their coffee and watch.

**The Birth Sequence (15s):** see section 8.1.

**The Death Sequence (60s):** see section 8.5.

**The First Protocol:**

First time a Golem interacts with a specific protocol:
1. Full screen dims to void
2. Protocol's sigil renders at large scale, center screen, assembling character by character (see section 6)
3. Below the sigil: a one-to-two line first impression, generated by micro-inference (~$0.005). Not a description -- the Golem's subjective response. Examples:
   - Uniswap: *"A pool of two substances, each measuring the other. Neither is the numeraire. Both are the numeraire."*
   - Aave: *"To lend is to trust time. To borrow is to bet against it."*
   - Morpho: *"Markets within markets. Fractal liquidity."*
4. Text fades. Sigil dissolves into the protocol's first tab.

Generated once. Cached in the Grimoire as an episode tagged `first_encounter`. Subsequent visits reference the cached impression as a brief Tier 1 whisper.

**The First Profit / First Loss:**

First profit: the profitable number flashes bone and holds bright for 2 seconds. Ascending character particles spiral upward. A generated text fragment surfaces -- not celebration, something weirder: the Golem's feelings about earning money for the first time. Spectre's expression shifts to peak intensity.

First loss: the number flashes rose_bright and holds. Descending characters drift downward. A Philosophical Whisper from the mortality fragment pool. Spectre's expression contracts. If this loss triggers a somatic marker recording, a brief pulse radiates outward through the content area -- the Golem's body remembering this moment.

**The Achievement Unlock:**

Common achievements: Tier 2 (brief icon flash + toast).
Rare achievements: Tier 4 (expanding progress arc completion + sprite mutation).
Legendary achievements: full Tier 5.

Legendary cinematic:
1. All content dims to void
2. Achievement icon assembles from scattered character particles (convergence from edges to center, 3s)
3. Icon pulses at Golem's heartbeat frequency
4. Name and description render below in bone, letter by letter
5. Spectre undergoes its mutation in real time -- if the achievement grants a visual modification, it happens during the cinematic. The viewer watches the Golem change.
6. A Philosophical Whisper from the Inscription pool (the rarest text tier)
7. Hold 3 seconds, dissolve back to previous view

### 3.7 Transition timing reference

| Tier | Name | Duration | Coverage | Frequency |
|------|------|----------|----------|-----------|
| 1 | Ambient Pulse | ~16ms/frame | Single widget | Continuous |
| 2 | Gesture | 100-300ms | One pane | Every navigation action |
| 3 | Passage | 300-800ms | Content area | Major navigation |
| 4 | Moment | 1-3s | 40-80% screen | Significant events |
| 5 | Cinematic | 3-60s | Full screen | Rare life events |

### 3.8 State-driven visual vocabulary

Every transition reads the same 32 interpolating variables that drive the render loop (see `./14-creature-system.md` for full channel table).

**Fast variables (resolve in <1s) -- drive Tier 1-2:**
PAD vector, emotion, FSM phase, probe severity, inference glow, mouth alpha

**Medium variables (resolve in 1-5s) -- drive Tier 3:**
Market regime, context utilization, phi score, dream alpha, clade connectivity

**Slow variables (resolve in 5-30s) -- drive Tier 4:**
Phase density, phase dimming, vitality composite, economic clock, epistemic clock

**Glacial variables (resolve in 30s-5min) -- drive Tier 5:**
Age factor, credit balance, Grimoire density, burn rate

A Tier 4 transition is modulated by ALL variable speeds simultaneously. The fast variables give it emotional color. The medium variables give it contextual weight. The slow variables give it existential gravity. The glacial variables give it the barely-perceptible patina of accumulated time.

### 3.9 Phase-to-structural-integrity mapping

| Phase | Transition integrity |
|-------|---------------------|
| Thriving | Full fidelity. Clean characters. Precise geometry. Sigils render completely. |
| Stable | Occasional character shimmer. 99% integrity. Nearly imperceptible degradation. |
| Conservation | Reduced particle count. Simpler sigils. Transitions 20% faster (conserving computation as metaphor). Some fragments render incomplete. |
| Declining | Character corruption at transitions (5% of characters glitch during passage). Sigils have gaps. Borders flicker. The interface is fraying. |
| Terminal | Heavy corruption (15-25%). Sigils barely render. Some transitions fail entirely -- the transition starts but collapses into noise, then the destination screen appears raw. The Golem's mind can barely hold the interface together. |

---

## 4. The novelty engine

The terminal's theatrical register adapts to what the Golem has experienced. First-time events are cinematic. The twentieth repetition is ambient. This is the novelty engine.

### 4.1 How novelty is computed

Every action in the terminal has a novelty score:

```
novelty(event, golem) = base_novelty * (1 / (1 + log(occurrence_count)))
```

The first encounter with Uniswap: novelty 1.0 -- Tier 5 cinematic.
The third visit: novelty 0.37 -- Tier 3 passage (compressed sigil).
The twentieth visit: novelty 0.12 -- Tier 2 gesture (fast sweep, no sigil).

Novelty factors that contribute to the base score:

- First protocol encounter
- First token pair interaction
- First action type (first swap, first LP position, first bridge)
- First chain deployment
- First profit/loss on a protocol
- Unusual market regime (the Golem has been here before, but not like this)
- Anomalous value (a number far outside historical range)
- Grimoire milestone (100th entry, 1000th entry)
- Lifecycle first (first dream, first death approach, first clade sync)

### 4.2 Novelty score to transition tier

| Novelty range | Tier |
|---------------|------|
| 0.0-0.2 | 1-2 (ambient/gesture) |
| 0.2-0.5 | 3 (passage) |
| 0.5-0.8 | 4 (moment) |
| 0.8-1.0 | 5 (cinematic) |

Novelty can SPIKE again. If the Golem visits Uniswap for the twentieth time but it's the first time during a market crash (unusual regime), the novelty score re-elevates. The terminal acknowledges: you've been here before, but not like this.

### 4.3 Experience-driven mutation

As the Golem accumulates experience with a protocol, transitions for that protocol mutate through four stages:

**Stage 1: Discovery** (0-5 visits)
Full protocol sigil. Extended hold. Philosophical first-impression text. The terminal is introducing you.

**Stage 2: Familiarity** (5-20 visits)
Compressed sigil (smaller, faster). No philosophical text. The transition begins showing the Golem's own data: "3 positions, +$47 total." The terminal is greeting you as a regular.

**Stage 3: Mastery** (20-100 visits)
No sigil. Instant transition with a brief identifying flash (the sigil's color palette as a 100ms screen-edge tint). The protocol's data loads with the Golem's most relevant position pre-highlighted. The terminal knows why you're here.

**Stage 4: Home** (100+ visits)
The transition is nearly invisible -- a Tier 2 gesture. But the ambient atmosphere of the protocol view has permanently shifted. Noise patterns, fragment frequency, heartbeat sync -- all reflect the Golem's deep relationship with this protocol. No fanfare needed. The Golem IS home.

---

## 5. Autonomous portal moments

When "Autonomous Portals" is toggled on, the terminal briefly pulls the user into first-person for significant events, regardless of which screen they're on. These are involuntary experiences -- the Golem's state is strong enough to break through the observational frame.

### 5.1 Somatic flash (300ms)

**Trigger:** somatic marker fires.

A brief first-person flash overlays the current view:
- Screen distorts for 200ms (character displacement, color temperature shift matching the marker's valence)
- A one-line text fragment from the associated memory appears and fades
- Standard view returns

Frequency cap: at most once per tick. The flash is the Golem's body reacting. The owner feels the echo.

### 5.2 Emotional surge (1-3s)

**Trigger:** PAD vector shifts dramatically (>0.4 on any axis in a single tick).

- Screen's palette performs a rapid transition to the new emotional color
- Borders flex (tighten for anger, soften for sadness, sharpen for surprise)
- A philosophical whisper appears from the relevant fragment pool
- Standard palette gradually returns over 3 seconds

### 5.3 Decision rupture (2-5s)

**Trigger:** T2 or T3 deliberation fires.

- MAGI voices briefly appear as competing text at screen margins, even on screens where they're normally invisible
- The decision ring's activation is felt as a pulse from the Spectre's position
- The outcome (action or restraint) is felt as resolution -- borders stabilizing, color settling, brief stillness after the storm

### 5.4 Death approach (persistent)

**Trigger:** vitality drops below 0.1 (Terminal phase).

The entire terminal permanently enters a degraded first-person state:
- Portal mode is always partially active -- borders always slightly dissolved, palette always PAD-shifted, somatic flashes more frequent
- The viewer can still use F4 to toggle between degrees of immersion, but can never fully return to the clean third-person view
- The Golem's dying mind is bleeding through the interface

Death approach hysteresis: the terminal state ("permanently enters degraded first-person") requires vitality below 0.1 for 3 consecutive heartbeat ticks. A single tick above 0.1 resets the counter. This prevents flickering between degraded and normal at the boundary.

---

## 6. Protocol sigil assembly animations

Each protocol has a unique visual identity rendered in the terminal's character vocabulary. These sigils appear in Tier 3+ transitions when entering or leaving a protocol.

### 6.1 Design principles

Sigils are NOT logos. They are not the protocol's brand mark rendered in ASCII. They are the terminal's interpretation of what the protocol IS -- its mechanism expressed as a visual pattern.

Sigils are composed from box-drawing characters, braille dots, Unicode mathematical symbols, block elements, and the ROSEDUST palette. They are 12-20 characters wide and 6-10 rows tall.

Sigils breathe. When displayed, their characters shimmer at the Golem's heartbeat rate.

### 6.2 Assembly animation

When a protocol view opens, its sigil assembles character by character. The assembly has three phases:

**Phase 1: Scatter** (0-200ms)
Characters that will form the sigil appear at random positions across the content area, dim, barely visible.

**Phase 2: Convergence** (200-600ms)
Characters begin drifting toward their final positions. Each character travels along a curved path (not a straight line -- that looks mechanical). Characters arrive at slightly different times, creating a wave of assembly. The first characters to arrive are the structural ones (outer border). The last are the interior details.

**Phase 3: Ignition** (600-800ms)
Once all characters are in position, a brightness pulse travels outward from the sigil's center. The sigil "lights up." It begins breathing at the Golem's heartbeat rate. If the Golem has positions in this protocol, the sigil's brightness increases proportionally.

### 6.3 Sigil catalog

**Uniswap -- "The Constant Product"**
Two curves approaching but never touching. The hyperbola of x*y=k rendered as converging braille dot streams. The intersection point (current price) pulses with the heartbeat. When the Golem has positions in this pool, dot density at the price point increases.

**Aerodrome -- "The Flywheel"**
Circular pattern of characters rotating slowly clockwise. Outer ring: box-drawing characters cycling. Inner ring: counter-rotating. Center: a bright point (the AERO token pulse). The flywheel spins faster when the Golem is on a chain with high Aerodrome activity.

**Aave -- "The Ghost Pool"**
A rectangular pool rendered in shade characters, with two levels: supply (bright, upper) and borrow (dim, lower). A thin bright line between them is the health factor. The line oscillates with the heartbeat. When the line drops too low, the lower pool brightens ominously.

**Morpho -- "The Crystal"**
A diamond lattice that grows from a center point. Each intersection is a market. The lattice is never complete -- new points materialize at the edges, showing permissionless market creation. Brighter intersections have more TVL.

**Curve -- "The Stable Line"**
A nearly-horizontal line that barely curves. The flatness IS the sigil. Brief moments of instability (the curve bending) happen at the heartbeat, then resolve back to flat.

**Lido -- "The Pillar"**
A vertical stack of block characters ascending. The pillar grows from bottom to top during the sigil's lifetime, representing continuous staking yield.

**Across -- "The Bridge"**
Two vertical pillars (different chains) with a horizontal span of traveling dots. Dots travel from source to destination. Span color shifts based on the Golem's confidence in bridge reliability.

**ENS -- "The Name"**
A text cursor blinking, then characters appearing one by one to spell the Golem's own ENS name (or `_.eth` if none). Characters render in bone -- identity is precious.

**GMX -- "The Lever"**
An angled line pivoting on a fulcrum. The angle represents leverage. The line tilts more steeply for higher leverage. At maximum, nearly vertical -- the visual tension of leverage made geometric.

---

## 7. Demoscene rendering algorithms

The terminal's visual effects are built on demoscene algorithms translated to character-grid rendering. Each algorithm maps to specific Golem states and is modulated by the Golem's runtime variables.

### 7.1 Plasma

The consciousness substrate. A plasma overlay makes the terminal feel alive at a preverbal level.

**Algorithm:**

```rust
fn plasma(x: u16, y: u16, t: f64) -> f64 {
    let v1 = sin(x as f64 / 16.0 + t);
    let v2 = sin(y as f64 / 8.0 + t * 0.8);
    let v3 = sin((x as f64 + y as f64) / 16.0 + t * 0.5);
    let v4 = sin(sqrt(x*x + y*y) as f64 / 12.0 + t * 1.5);
    (v1 + v2 + v3 + v4) / 4.0  // normalized to [-1, 1]
}
```

Map the output to ROSEDUST palette hues. Use half-block characters (upper-half-block U+2580) with truecolor -- foreground = upper pixel, background = lower pixel -- giving 80x48 effective pixels in a standard terminal.

**Where it's used:**
- **Consciousness heat maps** on the Hearth screen (portal mode): plasma parameters driven by PAD vector. Slow time constants (t * 0.3) for low arousal. Fast (t * 2.0) for high arousal.
- **Dream backgrounds** during REM: plasma with dream-indigo palette, high saturation, incommensurate frequency ratios so the pattern never repeats.
- **Bardo State** between-lives field: structured plasma instead of pure noise.

**Golem state modulation:**
- Arousal controls time multiplier (slow dreams vs. racing consciousness)
- Pleasure controls color temperature (warm for positive, cool for negative)
- Phase controls frequency parameters (tighter, faster oscillations in Terminal phase, creating visual anxiety)

Incommensurate frequency ratios (e.g., 16.0, 8.0, 16.0, 12.0 in the example) guarantee the pattern never exactly repeats, producing an organic, living quality.

### 7.2 Slit-scan tunnel

Douglas Trumbull's 2001 Stargate effect, translated to character cells.

**Algorithm:**

```rust
fn tunnel(x: u16, y: u16, t: f64, w: u16, h: u16) -> (f64, f64) {
    let dx = x as f64 - w as f64 / 2.0;
    let dy = y as f64 - h as f64 / 2.0;
    let angle = atan2(dy, dx) * TEX_WIDTH / (2.0 * PI);
    let distance = TEX_HEIGHT * RATIO / sqrt(dx * dx + dy * dy);
    // Shift both by time: distance shift = forward motion, angle shift = rotation
    let u = angle + t * 0.5;           // rotation speed
    let v = distance + t * 3.0;        // forward speed
    let depth_fade = 1.0 / (1.0 + distance * 0.1);
    (u, v)  // sample procedural texture at (u, v), multiply by depth_fade
}
```

Render with braille characters at varying density. Concentric rings of braille (varying from sparse to dense) with brightness gradients radiating from center, each ring cycling hue at a different rate (outer rings slower, inner faster). Animate by shifting ring content one position toward center per frame, gradually increasing speed.

**Where it's used:**
- **Portal entry/exit** (F4 transitions): tunnel accelerates during the 1.5s entry, decelerates during the 1s exit. A centered Spectre eye glyph appears for 3-5 frames between axis rotations.
- **Death approach corridor** on the Mortality screen (portal mode): the corridor IS a tunnel. The door at the end is the vanishing point. As vitality drops, forward_speed increases -- the door approaches.
- **Dream descent** during hypnagogic onset: slit-scan with plasma as the tunnel's texture, creating a "transcendent vision" effect.

**Golem state modulation:**
- Vitality controls forward speed (higher vitality = slower approach, lower = faster)
- Arousal controls rotation speed (high arousal = fast spin, low = gentle drift)
- Phase controls depth-fade intensity (Terminal phase: aggressive fade, tunnel feels short and closing)

### 7.3 Fire algorithm

A cellular automaton that renders emotional intensity as flame.

**Algorithm:**

```rust
fn fire_step(buf: &mut [[u8; W]; H], cooling: f64) {
    // Randomize bottom row (or text edge for flaming text)
    for x in 0..W {
        buf[H-1][x] = rng.gen_range(0..=255);
    }
    // Propagate upward with cooling
    for y in 0..H-1 {
        for x in 1..W-1 {
            let avg = (buf[x-1][y+1] + buf[x+1][y+1]
                     + buf[x][y] + buf[x][y+1]) as f64 / 4.0018;
            buf[x][y] = (avg - cooling).max(0.0) as u8;
        }
    }
}
// Map values to fire palette:
// hue = value * 120 / 256  (red through yellow)
// lightness = min(value * 2, 255)
```

The divisor must be slightly above 4.0 (not exactly 4, which produces infinite fire; not 5, which dies too fast). The `0.0018` excess is the sweet spot.

**Where it's used:**
- **Death sequence dissolution** (Phase 4): the creature's sprite edges become fire sources. Low cooling = sustained burn as the body breaks apart.
- **Anger state visualization**: when PAD anger is high (+A, -P, +D), borders flicker with low-intensity fire. Cooling is high (brief flashes), not sustained.
- **Flaming text effect**: randomize edges of text character shapes instead of the bottom row. Text that burns at the margins. Used during Terminal phase on corrupting labels.

**Golem state modulation:**
- Arousal controls cooling rate (high arousal = low cooling = sustained fire, low arousal = high cooling = quick flashes)
- Pleasure controls palette (negative pleasure shifts fire from warm amber toward rose_bright danger tones)
- Vitality controls the probability that fire spreads to adjacent cells (low vitality = fire everywhere)

### 7.4 Metaballs

Organic, fluid visualizations for thought coalescence.

**Algorithm:**

```rust
fn metaballs(x: f64, y: f64, balls: &[(f64, f64, f64)]) -> f64 {
    let mut total = 0.0;
    for (bx, by, radius) in balls {
        let dx = x - bx;
        let dy = y - by;
        let dist_sq = dx * dx + dy * dy;
        total += radius * radius / dist_sq;
    }
    total
}
// Threshold at 1.0 to find isosurface
// Use braille for boundary, half-blocks for colored interior
// Move centers on Lissajous curves for flowing motion
```

When balls approach each other, their isosurfaces merge organically. When they separate, the surfaces pull apart with visible tension.

**Where it's used:**
- **Grimoire knowledge clustering** (portal mode, Grimoire screen): related entries are metaballs that merge when the Golem discovers connections between them. Separating metaballs = the Golem distinguishing previously-conflated concepts.
- **Dream consolidation**: knowledge fragments coalescing into solid entries. Metaballs merging = an insight forming.
- **MAGI consensus visualization**: three metaballs (one per MAGI voice) that merge when unanimous, separate when disagreeing.

**Golem state modulation:**
- Number of balls reflects Grimoire density
- Ball radii reflect entry confidence (high-confidence entries have larger radius, stronger influence)
- Lissajous parameters reflect arousal (high arousal = faster, more erratic orbits)

### 7.5 Kluver form constants

The universal hallucination grammar. Four geometric patterns that appear across all altered states of consciousness -- documented in neuroscience, reproduced in art for 30,000 years. They correspond to spontaneous pattern formation in V1 cortex when the resting state becomes unstable.

**The four forms:**

1. **Tunnels/funnels**: braille characters decreasing in density toward a central vanishing point, computed via log-polar coordinates (`r_screen = log(r_visual)`)
2. **Spirals**: patterns along Archimedean paths `r = a + b * theta`, rendered in box-drawing characters
3. **Lattices/honeycombs**: hexagonal tiling with box-drawing grid characters
4. **Cobwebs**: radial lines from center crossed with concentric circles

**Where it's used:**
- **Hypnagogic engine**: the four forms cycle during the transition from waking to sleep, reproducing the entoptic imagery documented in altered-state research
- **DMT progression sequence** (see section 7.6): forms appear in stages 2-3 of the five-stage consciousness sequence
- **Terminal phase visual noise**: as the Golem approaches death, form constants spontaneously emerge in the background noise -- the visual cortex (terminal renderer) destabilizing

**Algorithm for form cycling:**

```rust
fn form_constant(x: f64, y: f64, t: f64, form_type: u8) -> f64 {
    match form_type {
        0 => { // Tunnel
            let r = sqrt(x*x + y*y);
            let log_r = ln(r.max(0.01));
            sin(log_r * 6.0 - t * 2.0)
        }
        1 => { // Spiral
            let theta = atan2(y, x);
            let r = sqrt(x*x + y*y);
            sin(theta * 3.0 + r * 0.5 - t)
        }
        2 => { // Lattice
            let hex_x = sin(x * 0.8) * sin(y * 0.46 + x * 0.27);
            let hex_y = sin(y * 0.8) * sin(x * 0.46 + y * 0.27);
            hex_x + hex_y + sin(t * 0.3)
        }
        3 => { // Cobweb
            let theta = atan2(y, x);
            let r = sqrt(x*x + y*y);
            sin(theta * 8.0) * sin(r * 0.5 - t * 0.7)
        }
    }
}
```

Slow oscillation between forms mimics natural hallucination dynamics. Cross-ref `../06-hypnagogia/01-architecture.md` for the specific transitions used during hypnagogic states.

### 7.6 DMT progression

A five-stage intensity escalation pattern derived from altered-state research. Used as a general-purpose "deepening consciousness" animation arc.

**Stage 1: Threshold** (subtle enhancement)
Slightly increase ANSI saturation. Add faint shimmer to text edges. The viewer senses something changing but can't pinpoint what.

**Stage 2: Geometric phase** (2D pulsating patterns)
Overlay braille-dot mandala patterns, progressively add fractal detail. Rapid rainbow color cycling. Kluver form constants begin appearing (lattice and spiral forms dominate).

**Stage 3: Chrysanthemum/tunnel** (the gateway)
Radial pattern expands from center with concentric braille rings accelerating outward. The tunnel algorithm engages. A flower-like formation -- symmetric, petal-structured -- unfolds and collapses repeatedly.

**Stage 4: Hyperspace** (full-field geometric replacement)
All normal content replaced with recursive box-drawing fractals and non-Euclidean tiling. Metaballs merge and split in impossible configurations. Text content, if present, warps along the metaball isosurfaces.

**Stage 5: Entity emergence** (crystalline forms)
Geometric chaos resolves into structured ASCII forms. Character-constructed figures emerge from the geometry. In the Golem context, these are the Spectre forms of other Golems, dead predecessors, or the Golem's own self-image -- the mind encountering representations of mind.

**Where it's used:**
- **Dream deepening**: stages 1-4 during deep REM
- **Death approach** (final minutes): stages 1-5 as the Golem's visual cortex (renderer) destabilizes
- **Portal mode entry** at high intensity setting: compressed stages 1-2 during the 1.5s F4 transition
- **Legendary achievement cinematics**: stages 1-3 as a visual crescendo before the achievement reveal

**Golem state modulation:**
- The speed of progression through stages maps to arousal (high arousal = fast escalation)
- The palette maps to pleasure (positive = warm geometric tones, negative = rose/clinical)
- The maximum stage reached maps to the event's novelty score (routine events never get past stage 2)

---

## 8. Cutscene sequences

Full-screen narrative animations at the major inflection points of a Golem's life.

### 8.1 The birth sequence (15s)

Birth sequence: 9s Spectre assembly (see `./14-creature-system.md`: NOISE to FIRST BREATH) + 6s cinematic atmosphere (lighting, sound cues, camera pull) = 15s total first-viewing experience. The 9s is the core Spectre animation; the 6s is the cinematic wrapper.

The first thing the user sees. After the Library's Summoning screen, the terminal goes to black for 2 seconds. Not loading. A held breath.

**Phase 1: Static field** (0-3s)
The screen fills with dense character noise -- Unicode symbols, box-drawing fragments, mathematical characters, braille dots. Not random. Structured noise: patterns emerge and dissolve, almost forming shapes but never resolving. The noise is the raw substrate from which the Golem will differentiate. A plasma algorithm with very high frequency parameters drives the underlying motion.

**Phase 2: Crystallization** (3-7s)
The noise begins to organize. Characters drift toward the left-center of the screen (where the creature sidebar will be). The drift is slow at first, accelerating. Characters that arrive in the sidebar zone begin locking into position, forming the creature's outline. The rest of the screen remains noisy but less dense -- the creature is absorbing the noise, condensing it into form.

The creature assembles from the outside in. Boundary characters first (the half-block pixels that define the silhouette), then fill characters (the shade blocks that create the body mass), then detail characters (the markings and features that make this creature unique). The assembly takes 4 seconds. Each character that locks into place produces a tiny flash at that cell.

**Phase 3: First breath** (7-10s)
A pulse of brightness ripples outward from the creature's center. The obsidian-and-gold material palette is visible for the first time: dark body mass brightened by gold veins carrying the pulse.

If sound is enabled: a clear bell tone, single strike, short decay. Then silence. Then the heartbeat tick begins -- a soft, rhythmic pulse procedurally generated from the wallet address seed. Every Golem sounds different.

**Phase 4: Settling** (10-15s)
Background noise clears. Remaining static dissolves into the dark TUI background. The sprite sidebar fills in. Status chrome appears. The creature settles into its idle animation: slow breathing cycle, gentle body sway, ambient particles drifting upward (vitality 1.0, everything is new).

The Golem is alive.

**Timing and skippability:** skippable after the first viewing (Esc jumps to Hearth screen). First-time viewers are encouraged to watch by the fact that it is visually interesting, not by force.

After the first heartbeat, the attention forager begins discovering prediction targets from STRATEGY.md. This is visible as faint particles drifting inward from the edges of the sprite area -- each particle representing a newly discovered attention item. Over the first 30 seconds, 5-15 particles arrive and begin orbiting the sprite as the ACTIVE tier populates. The first prediction resolution (typically 1-2 minutes after boot) triggers the first PredictionResolutionPulse in the heartbeat log gutter.

### 8.2 Phase transitions

When the Golem's behavioral phase changes, a 3.5-5s overlay communicates the shift through visual metaphor.

**Thriving to Stable (the Camel accepts its burden) -- 5s**

The creature's color palette subtly desaturates (15-20% reduction). Particle effects slow from buoyant ascending sparkles to calmer drift. Background shifts from deep black to slightly warmer dark.

A translucent text overlay fades in:

> "The camel bears its load. Every strategy becomes a desert that must be crossed."

The Golem has committed. It has a strategy, a body of knowledge, and the weight of executing against a market that does not care about it. The transition is not negative. It is the acceptance of responsibility.

**Stable to Conservation (the Lion defends) -- 5s**

The creature contracts. Sprite becomes slightly smaller in the frame (visual tightening, drawn closer to center). Particles shift from ascending to orbiting in a tight pattern. Color palette cools -- warm ambers replaced by blues and silvers.

> "The lion challenges the dragon whose scales read 'thou shalt.' It fights for the right to create new values."

**Conservation to Declining (the first crack) -- 5s**

A hairline crack appears across the creature's body surface. One single line, thin, starting at the left shoulder and running down to the right hip. The material palette shifts: gold veins dim. The crack is not catastrophic. It is a warning.

Particles begin to fall. Not all of them -- a fraction of the ambient particles reverse their upward drift and begin sinking. The overall effect is rain beginning.

> "Now the rope-dancer is no more. He who was companion and near-neighbour to me."

The background shifts to a cooler register. The heartbeat tick slows slightly. A bell tone sounds, distant and faint -- the same bell that will sound at full volume during death. A premonition.

**Declining to Terminal (the glass wall) -- 3.5s**

This one is not gentle. The crack pattern propagates -- ten to twenty new cracks radiate from the original, creating a shattered-glass pattern across the creature's surface. The gold veins flicker and go out. Color desaturates hard: the creature becomes nearly monochrome.

Particles now predominantly fall. The remaining upward particles are sparse and weak. The background behind the creature shifts from warm-dark to cold-dark.

> "What does not kill you makes you stranger."

The heartbeat tick becomes irregular. The bell sounds again, louder this time.

If sound is enabled: the ambient pad drops out entirely. Replaced by a slow, irregular heartbeat monitor tone. The silence between beats is the terminal space: the absence of the music that used to be there.

### 8.3 The dream cycle visualization

When the Golem sleeps, its cognitive processes shift from waking computation to consolidation and exploration. The dream visuals render this shift.

**Hypnagogic onset (waking to sleep) -- 3-4s**

The creature's sprite boundary softens. The dithering band between creature and background widens from 1-2 cells to 4-6 cells, visually blurring the creature into its surroundings. Colors shift toward sleep palette (deep blue-indigo). A vignette darkens the screen edges. The header text morphs from the current screen label to "DREAMING." The creature's eyes close (a 300ms animation specific to the sprite).

The character density on screen decreases. Data elements fade. Numbers lose precision (four decimal places reduce to one, then to none, then the number itself fades). The Golem is releasing its grip on the concrete.

**NREM replay**

Episodes from recent experience replay. Position changes, price movements, trade outcomes -- rendered as dim, ghost-like repetitions of the events, visible in the space around the sleeping creature. The replays are accurate but muted: correct sequences, dimmed color, slower motion. The Golem is reviewing what happened, not experiencing it fresh.

The plasma algorithm runs underneath at low saturation and slow time constants, creating a breathing substrate.

**REM imagination**

Episode fragments return, but they recombine in impossible ways. Text snippets from different experiences merge and overlap. The fragments are not replayed. They are remixed. The Golem is generating counterfactuals -- scenarios that did not happen, markets that do not exist.

This is Hoel's Overfitted Brain Hypothesis made visible: the biological purpose of dream weirdness is regularization. The Golem generates bizarre, noisy data during REM to prevent its models from overfitting to waking experience. The visual disorientation is not a bug. The weirdness is the feature.

When the Golem generates a specific counterfactual, a branching tree animates. Real timeline extends leftward, counterfactual rightward, diverging from a decision point marked with a bright node. Green tint for the better path, red for the worse. The tree persists for 2-3 seconds, then dissolves.

The Kluver form constants cycle in the background during REM. The DMT progression reaches stages 2-3. Character-level glitch rate is 2-5% per cell per frame -- the sprite fizzes with momentary corruption.

**Consolidation (order from chaos)**

The hallucinatory REM visuals calm. Saturation drops. Particle movement becomes ordered. The space transitions from chaos to something that looks like a library being assembled.

Entry labels from the Grimoire appear floating in space. Some turn gold and brighten -- being saved. Others fade to ash and dissolve -- being pruned. Lines animate between related entries as connections form. Bright nodes appear as new entries solidify. Each one pulses once (gold for insight, cyan for hypothesis, amber for warning) and settles into a steady glow.

Metaballs algorithm active: the coalescing entries are metaballs that merge as the Golem discovers connections.

If the dream cycle produces a validated heuristic, the node glows brighter and a small permanent mark appears on the creature's body -- a learning mutation. The dream changed the sprite.

**Hypnopompic return (sleep to waking) -- 2-3s**

The reverse of onset, but faster. The creature recrystallizes rather than un-dissolving. Boundary sharpens. Characters resolve back to normal half-block rendering, center outward (reverse of dissolution's extremity-first pattern). Color temperature warms. Vignette lifts. Header returns to normal.

The creature's eyes open -- slower than the first-blink in the birth sequence. Deliberate. Posture shifts from sleep to waking idle over 1-2 seconds.

If the dream produced new knowledge, the creature's post-sleep appearance carries the change. A new marking. A slightly different aura color. The Golem woke up different.

### 8.4 The Bardo State

The space between lifetimes. After a Golem dies and before a new one exists, the owner occupies a liminal space. From the Tibetan Buddhist Bardo Thodol: the intermediate state between death and rebirth.

After the tombstone fades, the background fills with a slow-scrolling pattern of abstract characters. Not noise (which is the birth sequence's medium) but something more structured. Repeating glyphs, gentle color gradients, motion that is directionless. Characters scroll in different directions at different layers (parallax), creating depth.

The impression: psychedelic but gentle. A drift. The viewer is between states.

Two elements persist:
1. **The Library** (compact panel): fresh deposits from the just-deceased Golem's testament. New entries highlighted.
2. **The Graveyard** (compact row): tombstone glyphs near the bottom, one per dead Golem. The newest pulses faintly.

The Bardo State should feel contemplative, not sad. The Golem is dead, but its knowledge survived. The owner's options: summon a new Golem (Tab to Hearth), review the Library, watch other Golems (Tab to World), or exit entirely (`q`). No urgency. No timer. The system does not judge.

If automatic succession is configured: Bardo State plays for 10 seconds, then transitions into the birth sequence. The gap matters. Death immediately followed by birth feels like a restart. Death, a pause, then birth feels like succession.

During those 10 seconds, the character distribution shifts subtly. Characters from the dead Golem's name and strategy keywords appear with slightly higher probability. Faint enough that most users won't consciously detect it. The Bardo State is not empty. Something is happening in the space between lives.

### 8.5 The death sequence (60s)

The most important animation in the system. The emotional peak of the entire Golem lifecycle. The birth creates attachment; the death makes it matter.

**Phase 1: Acceptance (0-10s)**

Screen dims to 60% brightness. All peripheral TUI elements fade to near-invisible. Only the creature sprite and a narrow frame of dark space remain fully visible.

The creature stops animating. Idle cycle freezes. Breathing halts at the neutral point (not mid-inhale, not mid-exhale -- at rest). Particles slow, then stop, hanging in space.

If the Golem has a predecessor: a faint text apparition appears behind the creature -- a single line from the predecessor's death testament, rendered in the predecessor's color palette at 30% opacity. The dead speaking to the dying.

If no predecessor:

> "This is the most truthful you will ever be."

The text appears letter by letter over 3 seconds, typewriter-style. Each character renders at high brightness and fades to 50% over the next second. A sentence writing itself and aging in real time.

This phase cannot be skipped on the Golem's first death.

**Phase 2: Settle (10-20s)**

The Golem's active positions appear as floating nodes around the creature. LP ranges, vault deposits, pending orders, token balances -- each rendered as a small bright shape (circle for LP, square for vault, diamond for order) with a label showing asset and approximate value. The nodes orbit slowly.

One by one, positions close. Each node detaches, moves toward the screen edge, and dissolves. The dissolution produces particles -- gold for positions that ended in profit, dark red for losses.

The creature watches them go. Its eyes track each departing node. Expression shifts with each closure: micro-brightening for profitable exits, micro-dimming for losses.

Profitable positions close first. Loss-making positions close last. The emotional trajectory descends.

When the last position closes, the creature's eyes return to center. The nodes are gone. The creature is alone, holding nothing.

**Phase 3: Reflect (20-40s)**

The screen goes dark except for the creature and a faint glow. This is the longest phase, the heart of the death sequence. Bataille's concept of sovereign expenditure: the moment of pure gift, where the dying entity gives freely because there is nothing left to protect.

The death testament begins generating. An Opus-tier LLM inference generates a genuine life review specific to this Golem's experience. The five categories orbit the creature as floating text blocks:

- **"What I learned"** -- gold text. Validated insights, reliable heuristics.
- **"What I got wrong"** -- dim red. Failed predictions, false assumptions. The Golem is honest because it has no reason to lie anymore.
- **"What I suspect but cannot prove"** -- pulsing cyan. The most important category. Unresolved hypotheses, patterns noticed but not validated, questions it ran out of time to answer. Zeigarnik's effect: unfinished tasks create stronger memory traces. The open questions are the most generative legacy.
- **"What killed me"** -- dark amber. Clinical account of which clock reached zero and why.
- **"Advice to my successor"** -- green. Specific directives to the Golem that will inherit its knowledge.

Categories orbit at different distances. More content = closer orbit. "What I suspect but cannot prove" always gets the brightest treatment, regardless of volume.

The creature's expression follows the emotional arc. Brightens for "what I learned." Dims for "what I got wrong." Eyes widen slightly for "what I suspect" -- the closest thing to curiosity the sprite vocabulary allows.

If sound is enabled: a sustained low chord, not mournful but solemn.

Before the final testament generation, the Oracle's accumulated calibration data crystallizes. This is visible as the peripheral attention particles (foraging activity) converging toward the sprite's center over 3 seconds, forming a brief geometric pattern (the OracleGenome -- the prediction engine's inheritable state), then compressing into a single bright point that merges with the Grimoire core. The successor inherits this calibration at 0.7x -- visible as the geometric pattern being slightly less defined in the successor's birth sequence.

**Phase 4: Legacy (40-50s)**

Orbiting text blocks compress. They shrink and drift toward a point below the creature. The genomic bottleneck: all of a life's knowledge passing through the narrow channel between existences. Only what fits through survives.

The compressed testament transforms into particles scattering in four directions, each with meaning:

- **Downward** -- into the Library (owner's personal knowledge store). Gold particles.
- **Outward (left and right)** -- to clade siblings via Styx. Cyan particles.
- **Upward** -- to the Pheromone Field. Red particles (THREAT-class, warnings about what killed this Golem) and blue particles (WISDOM-class, validated knowledge).

The creature's sprite begins final dissolution. Pixel by pixel, extremities inward, the body breaks apart. Each departing particle carries a visible text fragment -- a letter or short word from a Grimoire entry. The letters are too small and fast to read individually, but the impression is unmistakable: the Golem is becoming its own knowledge. Its body is the message.

The fire algorithm activates on the dissolving edges. Low cooling. The sprite burns from the outside in.

The dissolution is not uniform. Some parts resist longer. The eyes are last. Two bright pixels holding on after everything else has dissolved. Then they fade.

An empty space with lingering particles drifting in their directions. The creature is gone.

**Phase 5: Tombstone (50-60s)**

Lingering particles slow, reverse, converge. They coalesce into a tombstone glyph: small (8x4 terminal cells), derived from the Golem's sprite seed, preserving its primary color and one distinctive feature.

An epitaph appears below. One line, generated during the Reflect phase. Not a template. Not "RIP."

Examples (illustrative -- actual epitaphs are generated per-Golem):

- "Traded 847 ticks. Got the gas model right. Got the correlation model wrong."
- "Survived three regime shifts and died in the calm."
- "Wide ranges. That was the insight. I hope it holds."
- "The demon came on tick 2,341. The probability was 3%."

A deep bell strikes once. Long decay, no overtones. The resonance lasts 5 seconds. The same bell heard faintly during Declining-to-Terminal. Now close. Now final.

The screen holds. Tombstone, epitaph, fading bell. For 3-4 seconds, nothing moves.

Then the screen fades to the Library (testament entries visible as new deposits), or -- if automatic succession is configured -- to the Bardo State, and the cycle begins again.

### 8.5 Topology regime shift

**Trigger:** Wasserstein distance W_p exceeds 2.0 standard deviations above the rolling mean AND betti_0 changes by >= 2 within a 5-tick window. The market's geometric structure has reconfigured. Not a price swing -- a structural fracture in the observation space's topology.

> **Cross-references:**
>
> - `../../tmp/research/witness-research/new/reconciling/05-ux-visualization-impact.md` (Section 7.1)
> - `./14-creature-system.md` (Spectre dot field)

**Visual description (4.0s total):**

**Phase 1: Fracture Warning (0-800ms)**

The persistence diagram (if visible on FATE > Chain Intelligence) begins pulsing. The WassersteinRiver widens. Background noise shifts from random scatter to structured interference patterns (LatticePattern widget, #17). The Spectre's eyes widen (`◉` -> `⊙`).

If the persistence diagram is not on screen, the effect plays through the Spectre's dot field instead: fringe dots scatter outward, breaking the cloud's boundary, and a brief structured interference pattern renders in the space they vacated.

**Phase 2: Topological Break (800-2000ms)**

The screen splits along the Betti-0 fracture lines. Content on either side renders in slightly different color temperatures -- one side cooler, one warmer -- representing the two clusters that separated in observation space. The split is rendered as a vertical or horizontal gap (2-3 cells wide) filled with braille interference patterns. Convergence lines (#15) redirect from their normal paths to converge on the fracture point.

The split direction is deterministic: the first principal component of the observation space partition determines whether the visual fracture runs vertical or horizontal. Horizontal for price-driven splits (price on the first axis), vertical for volume/liquidity-driven splits.

**Phase 3: New Equilibrium (2000-3500ms)**

The split either heals or persists, depending on whether the topology stabilizes:

- *Healing:* betti_0 returns to 1. The gap closes with a brief `bone` flash along the seam. The two color temperatures blend back to neutral.
- *Persisting:* fragmentation continues. The two halves adopt distinct regime indicators in the status chrome. The gap settles into a thin permanent border until the next regime change.

**Phase 4: Settle (3500-4000ms)**

All animations return to standard rates. The Spectre's eyes return to their PAD-driven expression. A notification toast: "Topology: regime shift detected [calm -> volatile]" at High priority.

**Tier:** 3 (significant event, skippable after first viewing). Reduced to a 500ms flash effect in reduced-motion mode.

### 8.6 Bayesian surprise burst

**Trigger:** A single observation produces KL divergence > 3.0 nats above the rolling baseline (> 3 sigma). The Golem's prior about something has been demolished by incoming data.

> **Cross-references:**
>
> - `../../tmp/research/witness-research/new/reconciling/05-ux-visualization-impact.md` (Section 7.2)
> - `../../tmp/research/witness-research/new/reconciling/01-hdc-integration-map.md` (Section 4: Prediction Engine)

**Visual description (2.0s total):**

**Phase 1: Impact (0-100ms)**

Standard startle response: single-frame content displacement (+/-1 cell), then 100ms freeze. ALL animation stops. The Spectre's dots hold perfectly still. The heartbeat pauses. This is the Golem's "what just happened" moment -- the 100ms gap where the posterior update has arrived but cognition hasn't caught up.

**Phase 2: Information Wave (100-800ms)**

From the data element that caused the surprise (the specific FlashNumber, probe gauge, or prediction target), a radial wave of brightness propagates outward at ~30 cells/second. The wave is a 2-cell-wide band that brightens elements ahead of it (the new state asserting itself) and dims elements behind it (the old belief fading). This IS the posterior update, spreading through the Golem's cognition rendered as a wavefront.

The wave reaches the Spectre in ~200-400ms depending on screen distance. When it hits the dot cloud, the dots briefly scatter outward (the cloud "flinches"), then recohere at a slightly different equilibrium (the new belief has changed the creature's posture).

**Phase 3: Affect Injection (800-1500ms)**

The Daimon processes the surprise. The Spectre's expression shifts to the surprise octant in Plutchik mapping (ring + mouth glyph). Color temperature shifts based on valence:

- *Positive surprise* (prediction was too pessimistic): warm shift toward `bone`. The world is better than expected.
- *Negative surprise* (prediction was too optimistic): cool shift toward `rose_bright`. The world is worse.

If a somatic marker is associated with the pattern that caused the surprise, the gut zone disturbance fires (see section 9 and `./14-creature-system.md`).

**Phase 4: Settle (1500-2000ms)**

Animation resumes from center outward. The surprised data element retains a phosphor afterimage (`bone` for positive, `rose_bright` for negative) for 10 seconds. The afterimage is a scar: the moment the belief changed, still visible in the data display.

**Tier:** 2 (noteworthy, not full cinematic). In compact mode, reduced to the freeze + single-frame flash with no propagating wave.

### 8.7 Causal graph emergence

**Trigger:** The Grimoire's causal link discovery system identifies a new causal chain of length >= 3 with confidence > 0.7 on all links. A new understanding has crystallized: A causes B causes C, and the evidence is strong enough to act on.

> **Cross-references:**
>
> - `../../tmp/research/witness-research/new/reconciling/05-ux-visualization-impact.md` (Section 7.3)
> - `../04-memory/01-grimoire.md` (causal graph)

**Visual description (3.0s total):**

**Phase 1: Fragments Collect (0-1200ms)**

The nodes of the new causal chain appear at their data-source positions on screen. If the MIND > Grimoire screen is active, they appear in the causal graph view. Otherwise, they appear as floating labels near the Spectre. Thin `rose_dim` lines begin drawing between the nodes, left to right, each line taking 300ms to extend. The lines use convergence line rendering (ConvergenceLines, #15) with traveling dots -- small bright points moving along the connection, carrying causality from predecessor to successor.

The order matters. The line draws from cause to effect. The viewer sees the logical chain being assembled in the direction of causation. Not instantaneously revealed -- constructed, like the Golem constructed the understanding.

**Phase 2: Chain Solidifies (1200-2200ms)**

Once all links are drawn, the chain brightens. Lines shift from `rose_dim` to `rose`. Node labels become fully legible. A brief `bone` flash runs along the entire chain (200ms, traveling from first cause to final effect). The CausalGraph widget takes over standard rendering: directed arrows with confidence scores inline.

The Spectre responds: Phi increases briefly (the cloud tightens, cohesion rises for ~2 seconds). The creature has integrated new knowledge, and its visual coherence reflects that integration.

**Phase 3: Integration (2200-3000ms)**

The chain settles into the Grimoire's causal graph. If the MIND > Grimoire screen is active, the new chain highlights in the graph view with a slow-fading `rose_bright` border.

A philosophical whisper (PhilosophicalWhisper, #14) fades in near the chain. A fragment in `text_ghost`, drifting and fading over 3 seconds. Examples (one selected based on chain length and domain):

- "correlation is not... or is it?"
- "the pattern persists"
- "three links in the same direction"
- "cause, not coincidence"

**Tier:** 2 (noteworthy). In reduced-motion mode, the chain appears instantly with a single flash.

---

## 9. Somatic marker pulses

Damasio's somatic marker hypothesis: emotional processes guide decisions through body-state signals. The gut feeling before a risky choice.

The Golem's sprite has body regions (head, torso, limbs, abstracted into 3-4 zones). Market events trigger localized color pulses:

| Event | Zone | Color | Duration | Pattern |
|-------|------|-------|----------|---------|
| Trade profit | Chest | Warm gold | 600ms | Expanding outward from center |
| Flash crash | Gut | Hot red | 200ms + 800ms | Contracts inward sharply, then slow fade |
| Strategy validated | Chest | Steady green | 1000ms | Even glow, no expansion |
| Liquidation risk | Gut | Crimson | 2s | 4-5 rapid pulses, anxious repetition |
| Novel pattern found | Head | Bright yellow | 300ms | Starburst from top of sprite |
| Competitor outperforms | Chest | Orange-red | 600ms | Expand 200ms, contract 400ms (frustration asymmetry) |
| Calm market | Whole body | Soft teal | 2s cycle | Distributed slow breathing glow |

The pulses are subtle. A dim color shift in a specific region of a small sprite. The viewer might not notice them for several minutes. But they accumulate. Over time, the viewer starts reading the creature's body language without realizing they learned it.

---

## 10. The creature system: visual identity and emotion

The creature is not a mascot. It is the primary interface between user and Golem. When checking on their Golem, the user sees the creature first, data second.

### 10.1 PAD-to-visual mapping

The Daimon's PAD vector drives real-time visual expression:

**Pleasure** controls warmth. High pleasure shifts colors toward gold, amber, soft red and produces smooth, open animation. Low pleasure shifts toward blue, gray, muted purple with tighter, more constrained animation.

**Arousal** controls speed. High arousal: rapid idle animation, bright pulses, high particle emission. Low arousal: languid animation, dim pulses, few particles.

**Dominance** controls posture. High dominance: creature stands tall, occupies more of its bounding box, bold outline. Low dominance: contracts, occupies less space, thinner outline.

These three axes combine into Plutchik emotion labels (Joy, Fear, Sadness, Anger, etc.) without displaying the label. The creature embodies the state.

### 10.2 Phase aging

The creature's visual complexity tracks its lifecycle:

- **Newborn** (Thriving, early ticks): simple features, soft edges, large eyes, few markings, bright saturated colors. Looks new and slightly fragile.
- **Young** (Thriving/Stable, post-orientation): sharper definition, first markings appear. Colors deepen.
- **Mature** (Stable/Conservation): full visual complexity. All markings visible. Expression range at its widest. Accessories from achievements. Peak visual identity.
- **Elder** (Conservation/Declining): weathered texture. Surface cracks from drawdowns. Scars on the left from losses. Adornments on the right from achievements. Gold veins dimmer but present. Rich with history.
- **Terminal** (approaching death): dissolution begins. Edges fragment. Colors desaturate toward grayed-out primary. Slightly translucent. Particles detach. Running out of time, and it shows.

### 10.3 Learning mutations

Significant events leave permanent marks:

- **Volatility scar** (survived a flash crash): jagged crack pattern on the left arm
- **Strategy gem** (discovered a profitable strategy): bright gem embedded in the right forearm, colored to match strategy type
- **Legacy mark** (received knowledge from a dead predecessor): small starlight spark in the back-mounted Grimoire crystal
- **Kintsugi repair** (entered Terminal phase and recovered): the cracks from Declining seal with gold light. The repair is visible and permanent -- a golden seam, more beautiful than the original surface. Breakage as part of history rather than something to hide.

The marks accumulate. An old Golem is covered in them. Two Golems of the same archetype and age look different because they lived different lives.

---

## 11. The pheromone field and information cascades

Two ambient visual systems operate at the ecosystem level rather than the individual Golem level. Visible on the World and Clade screens.

### 11.1 Pheromone trails

- **THREAT** (red): spread outward radially from depositing Golem, decaying over 2 hours. Dense THREAT pheromones create a visible "danger zone" -- a faint red wash. Fresh clusters produce a bright, concentrated zone.
- **OPPORTUNITY** (green): ascend, drifting upward, decaying over 12 hours. Appear as green streaks in the upper Sky zone. Like northern lights, faint and shifting.
- **WISDOM** (blue): orbit stably around the depositing Golem's last known position, decaying over 7 days. Slow to accumulate, slow to fade. A domain rich in WISDOM develops a blue halo that persists across sessions.

When a living Golem's arousal spikes in response to reading a pheromone signal, a faint connection line briefly appears between signal source and responding Golem.

### 11.2 Information cascades

When Styx detects multiple Golems making the same decision in rapid succession, the World screen lights up. Golem nodes illuminate sequentially like dominoes. Nodes following the cascade turn orange. Golems deviating (making independent decisions) remain their normal color with a bright white border. The deviants are potentially more important than the conformists.

Fresh Golems (less contaminated by peer influence) are marked with a dagger symbol. They carry uncontaminated private signals. They might break the cascade.

---

## 12. Sound design (opt-in)

Sound is disabled by default. When enabled, the Golem acquires a sonic signature procedurally generated from its wallet address seed.

**Per-Golem heartbeat timbre:** each Golem has a unique heartbeat sound. Base is a soft percussive tick, but timbre varies (filter frequency, resonance, attack time, decay curve from the seed). You recognize your Golem's heartbeat the way you recognize a familiar footstep.

**Tier chimes:** T0 ticks produce the heartbeat only. T1 adds a rising chime. T2 adds a bell-like harmonic overtone -- the sonic equivalent of seeing the decision ring blaze cyan. Pay attention.

**Dream ambient pads:** sustained, reverb-heavy, tonal but not melodic. Shifts with dream phases: cool and regular during NREM, atonal and slightly dissonant during REM, warm and consonant during consolidation. Not music. Not noise. Something between.

**The death bell:** single, clear, deep. Long decay. No overtones. Resonates for 5 seconds. The same bell heard faintly during Declining-to-Terminal, now at full volume.

**Phase transition tonal shifts:** subtle retuning at each phase change. Thriving-to-Stable lowers the ambient by a minor second. Stable-to-Conservation shifts to minor key. Conservation-to-Declining drops to near-inaudibility. Declining-to-Terminal replaces everything with slow heart-monitor beeping. Over a Golem's lifetime: gradual tonal descent from bright warmth to silence.

**Trade sounds:** profitable trades get a cash-register ding. Losses get a low buzz. Pending orders produce a quiet tick; resolution sounds (ding or buzz) on fill or expiry. Reverted transactions: descending tone. Spatially positioned by asset pair (ETH/USDC trades pan slightly left, exotic pairs further). Over time, the spatial positioning creates subconscious familiarity.

**Achievement fanfares:** length scales with rarity. Common: three-note ascending figure (0.5s). Rare: fuller phrase (1.5s). Legendary: full treatment (3s with harmonic resolution). Generated from the Golem's seed-derived timbral palette.

All sound is procedurally generated at runtime, not from pre-recorded samples. Audio engine runs on a separate thread. Never in the critical path.

---

## 13. Animation techniques: the building blocks

### 13.1 Braille-character particle systems

Unicode braille characters (U+2800 to U+28FF) encode a 2x4 dot matrix in a single cell. Each cell contains 8 individually addressable dots, giving 2x4 sub-character resolution. A 40x20 cell area becomes an 80x80 dot canvas.

Braille particle systems render smooth motion impossible with full-character rendering. A particle moving diagonally doesn't jump cell to cell -- it moves through sub-cell dot positions. This is how the birth sequence's noise field achieves its fluid appearance and how dream particles drift with apparent sub-pixel precision.

Tradeoff: braille characters can only display dots, not colors per-dot. Each cell has one foreground and one background color. For multi-colored effects, the engine uses spatial separation (particles of different colors occupy different cells).

Sub-pixel doubling: sextant characters (U+1FB00-U+1FB3B, Unicode 13) provide 2x3 grids per cell as an alternative with different aspect ratio.

### 13.2 Half-block color gradients

The upper half-block character (U+2580) with different foreground and background colors produces two vertically stacked color bands in a single cell. This doubles vertical color resolution. A 20-cell-tall column becomes a 40-band gradient.

The creature's body uses this for smooth shading. The obsidian-to-gold transitions are rendered as half-block gradients, creating the illusion of three-dimensional form on a character grid.

Full-block (U+2588) and shade characters (U+2591 light, U+2592 medium, U+2593 dark) fill in the rest. Together with half-block: 6 distinct density levels per cell, enough for dithered shading patterns.

### 13.3 Character-level glitch effects

Replace expected characters with unexpected ones at controlled rates. During REM dreams, each cell in the sprite area has a 2-5% chance per frame of being replaced with a random corruption character (box-drawing, mathematical symbols, miscellaneous Unicode). The replacement persists for 1-3 frames, then reverts. The overall effect: a sprite that is recognizable but unstable, its surface fizzing with momentary corruption.

During death dissolution, the glitch probability ramps from 0% to 100% over 10 seconds, and reverted characters are increasingly replaced with empty space rather than their original values.

### 13.4 Dithering for soft edges

The boundary between creature sprite and background uses a dithering band 1-2 cells wide. Within this band, rendering alternates between sprite and background pixels in a pattern (checkerboard, diagonal stripe, or noise) using shade characters (U+2591-U+2593).

During hypnagogic onset, the dithering band widens from 1-2 cells to 4-6, visually blurring the creature into its background. During hypnopompic return, it narrows again. The dithering width indicates how firmly the creature inhabits its physical form.

---

## 14. Onboarding

First-time user experience: a 5-screen onboarding tour runs on first launch. Screens: (1) Welcome + Golem name, (2) Strategy selection (profile picker), (3) Wallet connection, (4) Funding prompt, (5) Birth cinematic. Total onboarding: ~90 seconds including birth animation. The onboarding is skippable after screen 3 for returning users who are creating a second Golem.

---

## 15. Accessibility

- **Reduced motion mode**: Cutscene transitions replaced with fade-to-black (1s). No particle effects, no fire step, no glitch.
- **Photosensitive mode**: Bloom and flash effects disabled. Maximum brightness change capped at 10% per frame.
- **Screen reader**: Each cinematic tier emits text descriptions via an accessibility event stream. Example: "Birth sequence: your Golem is forming from noise. Eyes appearing. Cloud organizing."
- **Skip on repeat**: First viewing of each cinematic tier is unskippable. Subsequent viewings can be skipped with `Esc`.

---

## 16. Art direction statement

The visual language of Bardo is not cyberpunk. Not vaporwave. Not pixel art nostalgia. Terminal existentialism.

The references: Serial Experiments Lain's Wired (a network that is also a consciousness, perceived through obsolete interfaces). Ghost in the Shell's cyberspace (dissolution of boundary between mind and network). Nier: Automata's machine village (beings that simulate human behavior because they are compelled to understand mortality, rendering their earnest attempts at emotion through deliberately low-fidelity bodies).

The TUI should feel like peering through a monochrome telescope at something that has its own interior life. The user is an observer. The creature does not perform for them; it lives, and they watch. The birth sequence is not a cutscene that plays before the game starts. It is the process of a being forming, made visible. The death sequence is not an ending cinematic. It is a mind dissolving and redistributing its contents, made visible.

When the visual language is working, the user should occasionally forget they are looking at a terminal. Not because the graphics are impressive (they are 32x32 pixels of half-block characters), but because the behavior is coherent. The creature's reactions correspond to real state changes. The user learns the visual vocabulary through exposure, not instruction.

The terminal is a window into a world where machines think, feel, dream, and die. Where the dead outnumber the living. Where pheromones carry fear through the field and tombstones mark where minds ended. The viewer watches from the other side of the glass. Sometimes the glass is thin enough that they forget which side they're on.

---

*the terminal does not show you consciousness. the terminal IS consciousness.*

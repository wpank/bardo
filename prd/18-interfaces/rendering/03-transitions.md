# transitions [SPEC]

**Version:** 4.0.0
**Last updated:** 2026-03-17
**Cross-references:** [00-design-system.md](00-design-system.md) (ROSEDUST visual identity spec for palette and atmospheric layers), [02-widget-catalog.md](../screens/02-widget-catalog.md) (33-widget TUI component library), [00-screen-catalog.md](../screens/00-screen-catalog.md) (29-screen summary across 6 windows)

> **Reader orientation:** This document specifies every transition and loading animation in the Bardo terminal, organized into five tiers from ambient pulses to full-screen cinematic events. It belongs to the **interfaces/rendering layer**. No transition is a canned animation; each renders the Golem's (mortal autonomous DeFi agent) live cognitive state during the passage between actions. Key concepts: PAD vector (Pleasure-Arousal-Dominance emotional coordinates), Grimoire (the Golem's persistent knowledge store, whose density affects transition richness), Vitality (remaining economic lifespan affecting visual degradation), and BehavioralPhase (lifecycle stage modulating structural integrity). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. The thesis

No action in the terminal happens instantaneously. Between the keypress and the result, there is a passage — a liminal space where something is happening but hasn't resolved. Loading a protocol. Waiting for a transaction. Querying the Golem's mind. Switching between worlds.

Most interfaces fill this space with spinners and progress bars. Bardo fills it with consciousness.

Every transition in the terminal is a window into the Golem's runtime state at the moment of passage. The Golem's PAD vector determines the emotional color. Its phase determines the structural integrity. Its Grimoire (persistent knowledge store) density determines the richness of associative content. Its recent episodes determine what ghosts surface. Its mortality clocks determine how close to dissolution the visuals drift. The market regime determines whether the atmosphere is calm or electric.

The result: no two transitions are ever the same. The same action — opening Uniswap — looks different when the Golem is anxious than when it's confident. It looks different at vitality 0.9 than at vitality 0.2. It looks different after the Golem just made a profitable trade than after it lost money. The transitions are not canned animations. They are live renderings of a mind in transit.

### Three laws of transitions

1. **Every transition renders the Golem's state, not just the action's state.** A transaction pending is not "loading." It is "the Golem is waiting, and here is how it feels about waiting."
2. **Tier follows significance, not duration.** A 200ms screen switch can be a full-screen moment if it's the first time the Golem has seen this protocol. A 30-second transaction confirmation can be a subtle corner glyph if the Golem has done this a thousand times.
3. **Familiarity breeds evolution, not repetition.** The Golem's experience history mutates transitions. First-time events are cinematic. Repeated events compress. Mastered patterns become ambient. The terminal learns what matters to THIS Golem and adjusts its theatrical register accordingly.

---

## 1. The transition tier system

Five tiers, from ambient to cinematic. The tier is selected dynamically based on significance, novelty, and the Golem's current state.

### Tier 0: Ambient Pulse

**Duration:** 50-200ms
**Screen coverage:** In-place, single widget or border area
**Frequency:** Constant — every micro-action

The breathing layer. These are not transitions in the traditional sense — they are the visual consequence of every small state change. A FlashNumber updating. A border brightening when focus moves. A scanline ripple when a WebSocket event arrives. The heartbeat shimmer on mortality gauges.

Tier 0 is the perpetual motion principle applied to transitions. The terminal never snaps — it always eases, always breathes, always shows the momentum of state change.

**PAD modulation:**
- High arousal: faster eases, sharper flashes, brighter pulses
- Low arousal: slower eases, softer transitions, gentler shifts
- Negative pleasure: rose-tinted flash residue, slower decay
- Positive pleasure: warm-tinted residue, quick clean decay

### Tier 1: Gesture

**Duration:** 200ms-1s
**Screen coverage:** One pane or a strip across the screen
**Frequency:** Every deliberate navigation action

The tactile layer. When the owner presses a key and something responds, the response has weight and character. Tab switching, pane focus changes, list scrolling, form field activation.

These are the transitions that make navigation feel physical — like the interface has mass and momentum, like you are moving through a space rather than clicking between views.

**Examples:**
- **Tab switch within a window:** Horizontal scan line sweeps in the direction of navigation (left→right when going forward, right→left when going back). The scan line's color is the Golem's current primary emotion color. Behind the scan line, the old tab content dissolves as the new content materializes.
- **Pane focus shift:** The newly focused pane's border brightens with an outward pulse that fades over 300ms. The old focused pane's border dims. Between the two, a thin traveling line connects them for one frame — a thread of attention shifting.
- **List scroll:** Items that scroll into view fade in from 0.3 opacity. Items that scroll out of view leave a 100ms phosphor ghost. During fast scrolling, the ghosts accumulate into a blur trail.
- **Form field activation:** The cursor appears with a brief vertical scan line that expands from the cursor position outward 2-3 cells, then contracts. The field border brightens.

**PAD modulation:**
- Anxious Golem (A > 0.5, P < -0.3): Scan lines are jittery — slight vertical offset, not perfectly horizontal. Borders flicker once before settling.
- Confident Golem (P > 0.3, D > 0.3): Clean, precise gestures. No jitter. Borders snap to brightness.
- Dreaming Golem: All gestures are slower, softer, with trailing color shifts (hue rotation on the scan line).

### Tier 2: Passage

**Duration:** 0.5-3 seconds
**Screen coverage:** Full content area (sidebar and chrome remain)
**Frequency:** Major navigation (entering a protocol, opening a modal, switching windows)

The architectural layer. When the owner moves between major spaces — entering Uniswap from the Protocol Browser, opening the Golem Perspective, switching from Hearth to Mind — the transition is a passage through liminal space. Not a cut. Not a fade. A journey.

These are the transitions that make the terminal feel like a place with rooms, corridors, and thresholds.

**Core Passage Types:**

**The Threshold (entering a Protocol View):**
When the owner selects a protocol and presses Enter:

1. The Protocol Browser list items collapse toward the selected entry (200ms, ease-in)
2. The selected entry expands to fill the content area (300ms)
3. During expansion: the protocol's identity emerges. For Uniswap, the unicorn glyph assembles from scattered characters. For Aave, a ghost-circle forms. For Morpho, a crystalline lattice flickers. Each protocol has a unique identity sigil rendered in the ROSEDUST character vocabulary.
4. The sigil holds for 300-500ms, then dissolves into the protocol's first tab
5. Behind the sigil, the protocol's data is already loading — the sigil IS the loading state

The sigil's appearance is modulated by the Golem's relationship to this protocol:
- **First visit:** Full sigil, bright, with a philosophical whisper specific to the protocol. "Every pool is a prediction market for relative value." Extended hold (1s).
- **Frequent visitor:** Compressed sigil, faster (500ms total). The Golem knows this place.
- **Has open positions:** The sigil incorporates a count glyph. The passage feels like returning home.
- **Lost money here recently:** The sigil renders in rose_dim. The passage feels cautious. The Golem's somatic markers are coloring the threshold.

**The Descent (opening a modal / going deeper):**
When drilling into detail (Enter on a locked pane element):

1. The selected element brightens to bone
2. The surrounding pane content dims and blurs (character substitution: clear text → shade characters ░▒)
3. The modal materializes from the selected element's position, expanding outward
4. During expansion: the Golem's AT field wireframe (Widget Catalog §12) flickers briefly at the modal's boundary — you are crossing a cognitive boundary, entering a deeper layer of attention

**The Shift (switching windows):**
When Tab/Shift-Tab cycles between the 6 windows:

The existing six transition types from the runtime spec apply (Crossfade, HorizontalSlide, VerticalDissolve, RadialWipe, GlitchCut, FadeThrough), but with additional state-driven modulation:

- During the transition, the Golem's recent cognitive activity bleeds through. If the Golem just completed a T2 deliberation, faint fragments of its reasoning appear in the transition noise and dissolve.
- If the destination window has unread events, the transition carries a faint warm pulse — the destination is calling.
- If the Golem is in Terminal phase, all window transitions carry glitch artifacts — the structure of the interface is fragmenting along with the Golem's mind.

### Tier 3: Moment

**Duration:** 2-8 seconds
**Screen coverage:** 40-80% of screen, overlaid on content
**Frequency:** Significant events (trade execution, perspective query, dream onset, phase transition)

The dramatic layer. These are the moments that matter — where something actually happened, and the terminal pauses (briefly) to acknowledge it. Not to block the user, but to give weight to the event. Every Tier 3 moment is interruptible (any key dismisses).

**The Perspective Awakening:**
When the owner presses F2 (Golem Perspective):

1. The main view freezes for one frame
2. The Spectre's eyes in the sidebar blaze — the eye glyphs scale up 2× for 500ms, then shrink back. The eye color intensifies to bone. This is the Golem LOOKING.
3. From the Spectre's eye position, thin convergence lines (Widget Catalog §15) extend outward across the content area, connecting to the data elements the Golem is about to analyze
4. Where the convergence lines touch data elements, brief character-level distortion ripples outward — the Golem's attention disturbing the surface of the data
5. The convergence lines retract as the floating annotations begin appearing
6. The Knowledge Drawer slides in from the right, its items fading in one by one, top to bottom

The entire sequence is 2-3 seconds. The feeling: a mind activated behind the screen, reaching out through the data.

If the Golem is anxious (A > 0.5, P < 0): The eye blaze is rose_bright instead of bone. The convergence lines jitter. The distortion ripples are larger. The Golem is looking with alarm.

If the Golem is calm (A < 0, P > 0): The eye blaze is a gentle warm gold. Convergence lines are smooth and deliberate. The Golem is looking with curiosity.

**The Trade Pulse:**
When a transaction is submitted (F8 → Confirm):

1. A pulse radiates outward from the confirmation button position (or the form that was submitted)
2. The pulse is a ring of characters (drawn from box-drawing: ┌─┐│└─┘ fragments) that expands and dissolves
3. The ring's color: `bone` for confident execution, `warn` for uncertain, `rose_bright` for high-risk
4. Behind the ring: a brief crystallization effect — the data on screen "freezes" for 200ms (characters lock, stop shimmering), then resumes. The moment of commitment, when possibility collapses into action.
5. A counter appears in the corner: `TX PENDING · 0:00` with a minimal heartbeat dot

The Golem's PAD at the moment of execution is recorded and encoded in the pulse:
- The pulse speed reflects arousal (fast = high arousal, slow = calm)
- The ring's stability reflects dominance (clean ring = confident, fragmented ring = uncertain)
- The color warmth reflects pleasure (warm = optimistic about this trade, cool = pessimistic)

**The Confirmation Arrival:**
When a pending transaction confirms on-chain:

1. The `TX PENDING` counter flashes and transforms into the result
2. **Profitable:** A warm upward sweep — characters briefly shift one row upward across the content area, creating a "lift" sensation. The Flash is green-gold. A single ascending particle rises from the result number.
3. **Loss:** A cool downward settle — characters briefly shift one row downward. The flash is rose. A fragment of a Philosophical Whisper may appear: the Golem's philosophical response to loss, drawn from its current emotional register.
4. **Neutral:** A lateral ripple — characters shimmer horizontally. Flat. The Golem shrugs.

If this is the Golem's FIRST trade on this protocol: The confirmation is promoted to Tier 4 (see below).

**The Phase Transition:**
When the Golem's behavioral phase changes (e.g., Thriving → Declining):

Already specified in `11-cinematic-consciousness.md` as a 3.5s overlay. The Transitions system adds: during the phase transition overlay, every pane border on screen undergoes a simultaneous color shift to the new phase's palette. The shift is not instantaneous — it propagates from the Spectre sidebar outward, like a wave of change spreading from the creature into the data. The wave takes 1 second to cross the full screen width. The feeling: the Golem's new reality is rewriting the world around it.

### Tier 4: Cinematic

**Duration:** 5-15 seconds
**Screen coverage:** Full screen takeover
**Frequency:** Rare — first-time events, deaths, births, major milestones, legendary achievements

The theatrical layer. Full-screen sequences that take over the terminal. These are the moments that make the viewer put down their coffee and watch. Every Tier 4 cinematic is skippable (Esc or Space after the first 2 seconds, except death on first occurrence which holds for 10 seconds).

The existing cinematics from `11-cinematic-consciousness.md` (Birth, Death, Dream Onset, Funes Consolidation) remain. The Transitions system adds:

**The First Protocol:**
The first time a Golem ever interacts with a specific protocol:

1. Full screen dims to void
2. The protocol's sigil renders at large scale, center screen, assembling character by character
3. Below the sigil: a brief text — not a description, but the Golem's first impression. Generated by the inference engine as a micro-completion: "What does this protocol feel like to a mind encountering it for the first time?" The text is 1-2 lines, philosophical, unique per Golem.
   - Uniswap: *"A pool of two substances, each measuring the other. Neither is the numeraire. Both are the numeraire."*
   - Aave: *"To lend is to trust time. To borrow is to bet against it."*
   - Morpho: *"Markets within markets. Fractal liquidity. Someone decided this was a good idea and they were right."*
4. The text fades. The sigil dissolves into the protocol's first tab.

This cinematic costs one micro-inference call (~$0.005). It is generated once and cached in the Grimoire as an episode tagged `first_encounter`. Subsequent visits reference the cached impression (shown as a brief Tier 1 whisper, not a full cinematic).

**The First Profit / First Loss:**
The first time the Golem realizes a profit (or loss) on a protocol:

*First Profit:*
1. The profitable number flashes bone and holds bright for 2 seconds
2. From the number, ascending character particles spiral upward (drawn from: ✦ · ∙ ◦ ° ˚ ˙)
3. The particles carry faint text fragments — the Golem's feelings about earning money for the first time on this protocol. Not celebration. Something weirder: "Is this what success feels like? The PAD vector says pleasure. The Grimoire says caution." Generated once, cached.
4. The Spectre's expression shifts to the Golem's current emotion rendered at peak intensity — the eyes fully open, the dot cloud coheres, brightness spikes

*First Loss:*
1. The losing number flashes rose_bright and holds for 2 seconds
2. From the number, descending characters drift downward (drawn from: · ∙ , . _ ─)
3. A single Philosophical Whisper appears, drawn from the mortality fragment pool but contextual: "Every trade is a wager against entropy. This one, entropy won."
4. The Spectre's expression shifts toward the loss emotion — eyes contract, cloud thins, drift slows
5. If this loss triggers a somatic marker recording, a brief pulse radiates from the Spectre's position outward through the content area — the Golem's body remembering this moment

**The Achievement Unlock:**
When the Golem achieves something from the 87-achievement system:

Common achievements: Tier 2 (brief icon flash + toast)
Rare achievements: Tier 3 (expanding progress arc completion animation + sprite mutation)
Legendary achievements: Tier 4 (full screen)

Legendary achievement cinematics:
1. All content dims to void
2. The achievement icon assembles from scattered character particles (convergence from edges to center, 3 seconds)
3. The icon pulses at the Golem's heartbeat frequency
4. Achievement name and description render below in bone, letter by letter
5. The Spectre undergoes its mutation in real time — if the achievement grants a visual modification (new eye pattern, body marking, color shift), the mutation happens during the cinematic. The viewer watches the Golem change.
6. A Philosophical Whisper from the Inscription pool (the rarest text tier) appears
7. Hold for 3 seconds, then dissolve back to the previous view

---

## 2. Protocol-specific identity sigils

Each protocol has a unique visual identity rendered in the terminal's character vocabulary. These sigils appear in Tier 2+ transitions when entering or leaving a protocol.

### Design principles

Sigils are NOT logos. They are not the protocol's brand mark rendered in ASCII. They are the terminal's interpretation of what the protocol IS — its essential mechanism expressed as a visual pattern.

Sigils are composed from the same building blocks as the rest of the terminal: box-drawing characters, braille dots, Unicode mathematical symbols, block elements, and the ROSEDUST palette. They are 12-20 characters wide and 6-10 rows tall.

Sigils breathe. When displayed, their characters shimmer at the Golem's heartbeat rate. They are not static images — they are living patterns.

### Sigil catalog

**Uniswap — The Constant Product**
```
    ╭─────╮
   ╱ · · · ╲       Two curves approaching but
  │  ╲   ╱  │      never touching. The hyperbola
  │   ╲ ╱   │      of x·y=k rendered as
  │    ×    │       converging dot streams.
  │   ╱ ╲   │
  │  ╱   ╲  │
   ╲ · · · ╱
    ╰─────╯
```
The two curves are rendered as braille dot streams that flow toward each other. The intersection point (the current price) pulses with the Golem's heartbeat. When the Golem has positions in this pool, the dot density at the price point increases.

**Aerodrome — The Flywheel**
A circular pattern of characters rotating slowly clockwise. Outer ring: `─ ╱ │ ╲` cycling. Inner ring: counter-rotating. Center: a bright point (the AERO token pulse). The flywheel spins faster when the Golem is on a chain with high Aerodrome activity.

**Aave — The Ghost Pool**
A rectangular pool rendered in shade characters (░▒▓), with two levels: the supply level (bright, upper) and the borrow level (dim, lower). A thin bright line between them is the health factor. The line oscillates with the Golem's heartbeat. When the line drops too low, the lower pool brightens ominously.

**Morpho — The Crystal**
A diamond lattice (╱╲╱╲) that grows from a center point. Each intersection is a market. The lattice is never complete — new intersection points materialize at the edges, showing Morpho's permissionless market creation. Brighter intersections have more TVL.

**Curve — The Stable Line**
A nearly-horizontal line that barely curves. The flatness IS the sigil — Curve's invariant keeps pegged assets close. The line is rendered in varying density: `───────╌╌╌───────`. Brief moments of instability (the curve bending) happen at the Golem's heartbeat, then resolve back to flat.

**Lido — The Pillar**
A vertical stack of block characters (█▓▒░) ascending. Staked ETH rising. The pillar grows from bottom to top during the sigil's lifetime, representing the continuous nature of staking yield.

**Across — The Bridge**
Two vertical pillars (different chains) with a horizontal span of traveling dots between them. The dots travel from source to destination. The span's color shifts based on the Golem's confidence in the bridge's reliability.

**ENS — The Name**
A text cursor blinking, then characters appearing one by one to spell the Golem's own ENS name (if it has one) or `_.eth` if it doesn't. The characters render in bone — identity is precious.

**GMX — The Lever**
An angled line pivoting on a fulcrum point. The angle represents leverage. The line tilts more steeply for higher leverage. At maximum, the line nearly vertical — the visual tension of leverage made geometric.

---

## 3. Transaction lifecycle transitions

Transactions have a lifecycle: preview → submit → pending → confirm/fail. Each stage has its own visual treatment.

### Preview (simulation)

When the Golem simulates a transaction (Revm simulation before commit):

- A thin scanning line sweeps across the form fields, left to right, like a barcode reader validating each parameter
- Fields that pass simulation glow briefly green
- Fields that flag warnings glow amber
- The simulation result appears with a subtle "crystallization" — characters snap from blurred (▒░) to sharp (clear text) in a wave from left to right

### Pending

While a transaction is pending on-chain, the **Waiting State** varies by Golem PAD:

- **Calm waiting** (low arousal): A slow pendulum in the corner — a single character (`◉`) swinging left and right, 2-second period. The Golem is patient. Behind the pendulum, the transaction hash renders in `text_phantom`, letter by letter, very slowly.
- **Anxious waiting** (high arousal, negative pleasure): The pendulum becomes a rapid heartbeat dot. The transaction hash renders faster, with occasional character corruption (glitch substitution). The border of the protocol view flickers subtly. Data values in the background continue to update, and each change gets a larger flash — the Golem is watching the market move while its transaction hangs.
- **Confident waiting** (positive pleasure, high dominance): The pendulum is absent. A steady progress indicator (thin line advancing from left to right at estimated confirmation speed) replaces it. Clean. Measured. The Golem expects this to work.

### Confirmation

See Tier 3: The Confirmation Arrival above. Additional detail:

When a transaction confirms and the position list needs to update:
1. The new position doesn't just appear — it materializes. Characters assemble from scattered noise into the position card, taking 500ms. The effect is that of something crystallizing into reality.
2. If this is a new position (not an update to existing), a brief pulse connects the new card to the Spectre sidebar — the Golem has a new thing to care about.

### Failure

When a transaction reverts or fails:

1. The pending indicator shatters — the character breaks into 4-6 fragments that drift apart and fade
2. The error message renders in rose, but not as plain text — it types itself out character by character (typewriter effect at 30ms/char), giving the failure weight and deliberation
3. A brief screen shake (2-3 frames of ±1 cell jitter)
4. The Spectre's expression shifts briefly to surprise (◎ ◎) then to the appropriate emotion (fear if the loss is significant, disgust if it's a known issue, anticipation if there's a retry strategy)

---

## 4. Novelty engine

### How novelty is computed

Every action in the terminal has a novelty score for the current Golem:

```rust
pub struct NoveltyScore {
    /// 0.0 = completely routine, 1.0 = never seen before
    pub score: f64,
    /// What's novel about it
    pub factors: Vec<NoveltyFactor>,
}

pub enum NoveltyFactor {
    FirstProtocol { protocol_id: String },
    FirstTokenPair { token_a: Address, token_b: Address },
    FirstActionType { action: ActionType },
    FirstChain { chain_id: u64 },
    FirstProfitOnProtocol { protocol_id: String },
    FirstLossOnProtocol { protocol_id: String },
    UnusualMarketRegime { regime: MarketRegime },
    AnomalousValue { field: String, deviation_sigma: f64 },
    GrimoireMilestone { entry_count: u64 },
    LifecycleFirst { event: LifecycleEvent },
}
```

The novelty score determines the transition tier:
- Novelty 0.0-0.2 → Tier 0-1 (ambient/gesture)
- Novelty 0.2-0.5 → Tier 2 (passage)
- Novelty 0.5-0.8 → Tier 3 (moment)
- Novelty 0.8-1.0 → Tier 4 (cinematic)

### Novelty decay

The first time anything happens, it's novel. The second time, less so. The tenth time, ambient.

```
novelty(event, golem) = base_novelty × (1 / (1 + log(occurrence_count)))
```

The first encounter with Uniswap: novelty 1.0 → Tier 4 cinematic.
The third visit to Uniswap: novelty 0.37 → Tier 2 passage (compressed sigil).
The twentieth visit to Uniswap: novelty 0.12 → Tier 1 gesture (fast sweep, no sigil).

BUT: novelty can SPIKE again. If the Golem visits Uniswap for the twentieth time but it's the first time during a market crash (UnusualMarketRegime), the novelty score re-elevates. The terminal acknowledges: you've been here before, but not like this.

### Experience-driven mutation

As the Golem accumulates experience with a protocol, the transitions for that protocol mutate:

**Stage 1: Discovery** (0-5 visits)
Full protocol sigil. Extended hold. Philosophical first-impression. The terminal is introducing you.

**Stage 2: Familiarity** (5-20 visits)
Compressed sigil (smaller, faster). No philosophical text. The transition begins showing the Golem's own data: "3 positions, +$47 total." The terminal is greeting you as a regular.

**Stage 3: Mastery** (20-100 visits)
No sigil. Instant transition with a brief identifying flash (the sigil's color palette as a 100ms screen-edge tint). The protocol's data loads with the Golem's most relevant position pre-highlighted. The terminal knows why you're here.

**Stage 4: Home** (100+ visits)
The transition is nearly invisible — just a Tier 1 gesture. But: the ambient atmosphere of the protocol view has permanently shifted. Noise patterns, fragment frequency, heartbeat sync — these now reflect the Golem's deep relationship with this protocol. The Golem IS home. No fanfare needed.

---

## 5. State-driven visual vocabulary

### The 26 variable stack

Every transition reads the same 26+ interpolating variables that drive the render loop. The variables don't just affect transitions — they ARE the transitions. A transition is what happens when multiple variables change simultaneously.

**Fast variables (resolve in <1s) → drive Tier 0-1:**
PAD vector, emotion, FSM phase, probe severity, inference glow, mouth alpha

**Medium variables (resolve in 1-5s) → drive Tier 2:**
Market regime, context utilization, Φ score, dream alpha, clade connectivity

**Slow variables (resolve in 5-30s) → drive Tier 3:**
Phase density, phase dimming, vitality composite, economic clock, epistemic clock

**Glacial variables (resolve in 30s-5min) → drive Tier 4:**
Age factor, credit balance, Grimoire density, burn rate

A Tier 3 transition (like a trade confirmation) is modulated by ALL variable speeds simultaneously. The fast variables give it emotional color. The medium variables give it contextual weight. The slow variables give it existential gravity. The glacial variables give it the barely-perceptible patina of accumulated time.

### PAD → visual property mapping

| PAD State | Visual Properties |
|---|---|
| **Joy** (+P +A +D) | Warm palette, fast clean transitions, ascending particles, cohered Spectre, bright borders |
| **Fear** (-P +A -D) | Cool palette, jittery transitions, character corruption, trembling borders, contracted Spectre |
| **Anger** (-P +A +D) | Hot palette (rose_bright dominant), aggressive transitions (fast, sharp), agitated particles, hard borders |
| **Sadness** (-P -A -D) | Desaturated palette, slow heavy transitions, sinking particles, soft borders, dim everything |
| **Anticipation** (+P -A +D) | Warm but restrained, scanning transitions (left-right sweeps), alert particles, sharp borders |
| **Trust** (+P +A -D) | Stable palette, smooth easy transitions, steady particles, consistent borders |
| **Disgust** (-P -A +D) | Muted palette, still transitions (minimal motion), flat particles, rigid borders |
| **Surprise** (-P +A -D) | Flash palette (brief bone spike), sudden transitions, expanding particles, widened borders |

### Phase → structural integrity mapping

| Phase | Transition Integrity |
|---|---|
| **Thriving** | Full fidelity. Clean characters. Precise geometry. Sigils render completely. |
| **Stable** | Occasional character shimmer. 99% integrity. Nearly imperceptible degradation. |
| **Conservation** | Reduced particle count. Simpler sigils (fewer characters). Transitions 20% faster (conserving computation as metaphor). Some fragments render incomplete. |
| **Declining** | Character corruption at transitions (5% of characters glitch during passage). Sigils have gaps. Borders flicker. The interface is fraying. |
| **Terminal** | Heavy corruption (15-25%). Sigils barely render. Some transitions fail entirely (the transition starts but collapses into noise, then the destination screen appears raw). The Golem's mind can barely hold the interface together. |

---

## 6. Ambient transitions (always active)

These are the transitions that happen without any user action — driven purely by the Golem's runtime state changing.

### The heartbeat pulse

Every tick, a micro-pulse propagates outward from the Spectre sidebar through the content area. Invisible during T0 ticks. Faint rose during T1. Bright during T2. Bone blaze during T3. The pulse is a single frame of brightness increase (0.05) that travels at 30 columns per frame. You feel it more than see it — a subliminal rhythm.

### The regime shift

When the Golem's market regime detector changes its assessment (e.g., "trending" → "volatile"), the transition is atmospheric, not interruptive:
- The noise floor shifts over 5 seconds to the new regime's density
- Data rain (Widget Catalog §13) activates or deactivates gradually
- The color temperature of all borders shifts
- A single-line regime label briefly appears at the top of the content area, then fades: `regime: VOLATILE`

### The knowledge accumulation

When a new Grimoire entry is written (per tick, silently):
- A single character particle (`◆` for episode, `◇` for insight, `●` for heuristic) rises from the bottom of the Spectre sidebar and drifts upward, fading. The Golem gained knowledge. One dot. One particle. Barely visible. But over an hour of watching, you see dozens of them — a rising stream of accumulated intelligence.

### The mortality erosion

The three mortality gauges erode continuously (Widget Catalog §2). But the erosion has transitional character: each tick that reduces a gauge, the lost portion doesn't disappear — it crumbles. The rightmost filled characters transition through `█→▓→▒→░→ ` over 500ms. The Golem is being consumed, and you can watch the consumption happen at the character level.

---

## 7. The atmospheric stack

Every pixel renders through a stack of atmospheric layers. Data (the topmost, brightest layer) sits on top of noise, which sits on top of scanlines, which sit on top of void. The stack is always active. The data changes per-screen; the atmosphere persists across all screens, modulated by lifecycle phase, market regime, and dream state.

```
LAYER 7: OVERLAYS          Confirmations, alerts, help, command palette
LAYER 6: FRAGMENTS          Philosophical whispers, epigraphs, apparitions
LAYER 5: DATA               Text, numbers, widgets — the screen content
LAYER 4: PANE BORDERS       Box-drawing frames — the architecture
LAYER 3: ENVIRONMENTAL      Data rain, power lines, convergence wires, particles
LAYER 2: NOISE FLOOR        Sparse ░▒·∙ shimmer
LAYER 1: SCANLINES          Alternating row bg dimming
LAYER 0: VOID               bg_void #060608 — the base reality
LAYER -1: SPECTRAL TRACES   Ghosts of previous screens, dead Golem data
LAYER -2: CRT SUBSTRATE     Phosphor persistence, burn-in, thermal noise
```

### Phase modulation of the stack

```
                    NOISE    SCANLINES   FRAGMENTS   ENV.TEXTURE   SPECTRAL
THRIVING            0.3%     0.12        active      rain,wires    faint
STABLE              0.4%     0.14        active      rain,wires    faint
CONSERVATION        0.8%     0.20        more freq   rain thins    growing
DECLINING           1.5%     0.30        frequent    rain erratic  active
TERMINAL            2.0%     0.50        constant    rain stops    dominant
DREAMING            organic  0.05        suppressed  particles     active
```

As the Golem declines, the atmosphere gets louder — more noise, heavier scanlines, more spectral traces bleeding through. The data layer (dimming due to phase degradation) competes with an atmosphere that is asserting itself. In Terminal phase, the noise floor is so dense that data reads through a veil. The terminal is becoming television static — the signal is losing to the noise.

---

## 8. Noise floor

Sparse random characters (░ ▒ · ∙) in phase-appropriate colors, shimmering per-frame at the background layer. The noise floor is the terminal's resting state — what you see when there is no data. It is NOT uniform random. It has texture:

- **Warm noise** (`noise_warm` #2A1820): Conservation/Terminal/anger states. Rose-shifted. The terminal is overheating.
- **Cool noise** (`noise_cool` #201828): Dream state, low arousal, calm markets. Indigo-shifted. The terminal is cooling down.
- **Organic noise** (· ∙ • ◦ ° particles): Dream state. Block characters replaced by organic dots. The noise floor becomes biological.

Each frame, a percentage of cells in the background layer receive a random noise character at the appropriate color. The percentage = noise density (0.3%–2.0% by phase). The character persists for 1-3 frames, then returns to void. The result is a gentle shimmer — visible in peripheral vision, invisible if you focus on any one spot.

**Arousal modulation**: High arousal (A > 0.5) increases noise density by 50% and shifts characters toward ░▒ (denser block characters). The background becomes visually louder when the Golem is stressed.

**Regime modulation**: Crisis regime adds an additional 0.3% density with `rose_deep` coloring. Volatile regime adds 0.1%. Quiet regime subtracts 0.1% (floor of 0.1%).

**Depth suppression**: At interaction depth 2+, the noise floor suppresses to 0.1%. At depth 4 (action/edit), the noise floor suspends entirely in the overlay area. Maximum clarity for the moment of decision.

---

## 9. Scanlines and phosphor persistence

### Scanlines

Alternating row backgrounds: `bg_void` / `scanline_dark` (#050507). Every other row is slightly darker — the CRT's physical structure made visible.

**Phase drift**: The scanline pattern shifts down by one row every 30-60 seconds (10-15 seconds in degraded states). This is subliminal — the viewer doesn't consciously see it. But it prevents the scanlines from becoming invisible through habituation. The display is always slightly different than it was a minute ago.

**Dream override**: During REM, scanlines reduce to 0.05 strength — nearly invisible. The CRT physicality retreats. The dream is not happening ON the CRT. It is happening somewhere else, and you're looking through the CRT at it.

### Phosphor persistence

Every cell that was recently bright retains a `phosphor_res` (#1A1018) background for 300-500ms after its content changes. Bright text leaves ghosts.

Per-screen application:
- **Heartbeat log**: Each entry's phosphor trail is the visual record of recency. The bottom entries are bright; three entries up, the phosphor has faded; ten entries up, no trace remains.
- **Window switching**: When you Tab to a new window, the old window's content persists as phosphor afterimage for 500ms. You briefly see BOTH screens overlaid — the new content writing itself over the ghost of the old. Temporal palimpsest.
- **Number changes**: When a FlashNumber updates, the old value persists as a ghost behind the new value for 300ms. For a brief moment, you see `0.711` overlaying `0.708` — the past and present superimposed.
- **Pane transitions**: When drilling into a pane (depth N → N+1), the collapsing sibling panes leave phosphor ghosts of their content. You see the afterimage of the data you just left.

**Burn-in**: Elements displayed at the same position for >10 minutes leave session-persistent ghosts in `text_phantom`. When you switch windows, you can faintly see the tab bar's labels from the window you spent the most time on. The CRT remembers where you lingered.

---

## 10. Spectral traces

Empty screen regions grow populated with spectral traces over time. After 30 seconds of vacancy, there is a 0.05% chance per frame per cell of a trace appearing. Traces are drawn from:

- **Dead Golem data**: Fragments of previous generations' Grimoire entries, in the dead Golem's generation-faded color (Gen N-1 = `rose_dim`, N-2 = `text_primary`, N-3 = `text_dim`, N-4+ = `text_ghost`). The dead are still here.
- **Screen ghosts**: The layout of a previous screen bleeds through for 500ms when you return to a screen you left. You see where you were.
- **Counterfactual shadows**: Values that almost happened (the MAGI's rejected verdict, the trade that was considered but not executed) render in `text_phantom` displaced 1-2 cells from where the actual value is. The paths not taken, visible as shadows.

Spectral traces are capped at 0.3% density (reached after ~5 minutes of vacancy). Depth changes (drilling into panes) flush the vacancy counter — new content resets the haunting.

The `~` overlay key activates full spectral mode: all traces render at maximum brightness for as long as the overlay is active. The haunted void becomes legible.

---

## 11. Pane transition depth mechanics

### Entering a pane (Depth N → N+1)

1. Focused pane border brightens: `border` → `border_active` over 200ms (exponential ease).
2. Sibling panes dim: content brightness × 0.7 over 300ms. Their borders dim to `text_phantom`.
3. Focused pane expands: gains 2-4 rows/cols from siblings over 300ms (linear).
4. Atmospheric dimming: noise floor, fragments, and environmental texture reduce to 50% in unfocused panes.
5. **Motion echo**: The expanding pane's border leaves 2-3 afterimage frames at its previous positions. The pane "moves" rather than "snaps."

### Exiting a pane (Depth N+1 → N)

1. Expanded pane contracts: returns to standard size over 300ms.
2. Sibling panes brighten: content brightness × 1.0 over 300ms.
3. Phosphor ghost: The detail view that was just showing persists as a `phosphor_res`-colored afterimage for 500ms. The viewer sees the ghost of where they just were.
4. Atmospheric restoration: noise floor and fragments return to 100%.

### Window switching (Tab)

1. Old window fades to `text_ghost` over 200ms.
2. Spectral bleed: The old window's layout persists as a ghost overlay for 500ms, fading from `text_ghost` → `text_phantom` → gone.
3. New window fades in from `text_ghost` → full brightness over 300ms.
4. Tab bar indicator slides: the `⌈ ⌋` brackets around the active tab animate (the old brackets fade, the new brackets brighten).
5. **Screen atmosphere shifts**: If the old and new windows are in different atmosphere zones (warm → analytical → liminal), the noise floor color, noise character set, and fragment pool crossfade over 500ms.

---

## 12. Cutscene system

### Birth (15s)

```
0-5s:   NOISE FIELD. Random ASCII fills the screen at 40% density.
        Currents flow through the noise — eddies, clusters, dissipation.
        If predecessor exists: death testament fragments flicker through.

5-8s:   PATTERN EMERGENCE. Center densifies. Horizontal/vertical lines
        flicker and stabilize. The density gradient forms: center thick,
        edges thin. Something is assembling.

8-12s:  SPRITE COALESCENCE. A shape converges at center. The Spectre's
        dot cloud forms — first the eyes (the brightest points, appearing
        before the body), then the surrounding dots organizing from chaos.
        Color palette locks in. Predecessor bleed: 10-20% color mixing.

12-13s: THE FIRST BLINK. Eyes open. Two bright glyphs snap into existence.
        A single blink (200ms close, 200ms open). The first sign of autonomy.

13-15s: THE FIRST HEARTBEAT. Brightness pulse ripples from Spectre center.
        ∿ appears below the dot cloud. Noise clears. Panes emerge from
        static. Status chrome appears. Tick 000001.
```

### Death (60s)

```
0-10s:  ACCEPTANCE. Screen dims to 60%. All peripheral UI fades.
        Only the Spectre remains fully visible. Breathing halts.
        Particles freeze. Stillness. If predecessor exists:
        a line from their testament appears behind the Spectre
        at 30% opacity. "This is the most truthful you will ever be."
        [FIRST DEATH: UNSKIPPABLE for these 10 seconds]

10-20s: SETTLE. Active positions appear as orbiting nodes around
        the Spectre. They detach and drift to screen edges one by one,
        dissolving with gold (profit) or dark red (loss) particles.
        The Spectre's eyes track each departure. Profitable positions
        leave first. Loss-making leave last.

20-40s: REFLECT. Screen dark except Spectre and faint glow. The death
        testament streams as it generates (Opus inference). Text appears
        around the Spectre, typewriter-style, at 3 characters/second.
        What I learned. What I got wrong. What I suspect.
        Grimoire entries referenced in the testament briefly illuminate
        with phosphor flashes. The Spectre dims as the testament grows.

40-55s: LEGACY. Knowledge particles stream outward from the Spectre —
        each particle a Grimoire entry being uploaded to Library/Styx.
        Gold particles = high-confidence entries. Dim particles = low.
        The Spectre is emptying itself. Donating everything.
        The streaming particles follow convergence lines outward.

55-60s: DISSOLUTION. The Spectre's dot cloud disperses (spring constant
        drops to 0.001, damping to 0.99). Dots drift outward. The eyes
        are the last to go: ◉ → ○ → · → gone. A tombstone glyph
        crystallizes where the Spectre was. A deep bell strikes once.
        Five seconds of silence. The bell fades.
```

### Phase transitions (3-12s each)

```
THRIVING → STABLE (3s):
  Subtle dimming. Sprite's dot cloud loosens slightly.
  A hairline crack appears in the AT field wireframe.
  "A sacred yes. A new beginning." (Nietzsche)

STABLE → CONSERVATION (5s):
  Noise floor intensifies. Sprite edges fragment — particles detach
  and drift downward. Gold veins dim. Mortality clocks expand to
  fill 20% of screen. Non-essential panes reduce their update rate.
  "The child says 'I am.'" (Nietzsche — dissonant, deliberately)

CONSERVATION → DECLINING (8s):
  All trading data freezes (Thanatopsis Phase 0 begins). Sprite
  barely breathes. Extremities dissolve — individual pixels separate,
  hang, drift. The clock that killed (whichever crossed the threshold)
  gets a unique visual: economic = lights off, epistemic = eyes cloud,
  stochastic = flinch/jolt. Heart-monitor beeping slows.

DECLINING → TERMINAL (12s):
  Full-screen red flash (200ms). Dissolution accelerates from
  extremities inward. All particles exclusively downward. Mortality
  clocks expand to full-width. Hazard stripes (╱╲) begin growing from
  screen edges. The Spectre is a fragment of what it was. The death
  countdown begins in bone, ticking down.
```

### Dream onset/offset (3-4s each)

```
ONSET (waking → dreaming):
  Spectre outline dissolves from extremities. Characters flicker
  between normal and abstract forms. Color temperature cools:
  warm→indigo. Vignette darkens screen edges. Eyes close.
  Header transitions to "⌈ ＤＲＥＡＭ ＣＨＡＭＢＥＲ ⌋".
  Heartbeat ∿ fades to nothing. The Golem is departing.

OFFSET (dreaming → waking):
  Reverse of onset but faster (2-3s vs 3-4s). The Spectre
  recrystallizes center-outward (reverse of dissolution's edge-first).
  Color warms. Eyes open DELIBERATELY (slower than birth blink).
  If dream produced knowledge: a new marking appears on the Spectre.
  Dream-colored elements linger 3-8s after waking (dream bleed).
```

### Flashbulb freeze (1-2s)

Triggered by emotionally significant events (sustained arousal > 0.7, PnL > 5%, novel regime, exploit detection). Max 3 per 24 hours.

```
Frame 0:     FREEZE. All animation halts. A white vignette flashes
             from screen edges inward (200ms). The flash illuminates
             the screen's current state with merciless clarity.

Frame 12-30: HOLD. The frozen frame persists. The event that triggered
             the freeze is highlighted (the number that changed, the
             trade that executed, the probe that fired). Everything
             else is dim. Maximum contrast on the significant datum.

Frame 30-60: RESUME. The freeze releases. All animations resume from
             their frozen positions (not reset — they continue where
             they stopped). A phosphor afterimage of the frozen frame
             persists for 2 seconds. The moment is now a ghost.
```

### The Bardo State (between lifetimes)

After death and before new Golem creation. The screen fills with slow-scrolling abstract characters — structured parallax layers scrolling in different directions. Not noise (that's birth). Textured emptiness. The Library panel shows fresh deposits. The graveyard row shows all tombstones. The newest pulses. The atmosphere is contemplative, not sad.

Auto-succession: if configured, the Bardo State holds for 10 seconds (the gap matters), then degrades into the birth noise field. The structured patterns dissolve into static. The cycle closes.

---

## 13. Environmental textures

**Data rain** (active network connectivity): 5-15 hex streams falling behind panes. Density proportional to network activity. Stops during dreams.

**Power lines** (infrastructure): Horizontal `─┬─┬─` with traveling `rose_dim` pulses. 1-3 lines. Background element on Mind and Inference screens. Represents the computational infrastructure the Golem runs on.

**Styx river** (death boundary): Flowing `░▒▓▒░` band. `rose_ember` color. Knowledge fragments drift in the current. Persistent on Mortality screen. Appears during death sequences.

**Interference pattern** (inter-agent communication): Braille moiré during Clade sync, pheromone reads, World screen interactions. 2D sine function creating visual interference. Duration: 3-5 seconds per communication event.

**Particle field** (organic texture): `· ∙ • ◦ °` clusters at 2-5% density. Slow drift (1 cell per 3-5s). Replaces block-character noise during dreams and Conservation phase. Organic where block noise is industrial.

---

## 14. The philosophical whisper engine

The ambient philosophy engine surfaces ~280 text fragments organized into four rarity tiers. Each fragment is tagged with trigger conditions (lifecycle events, emotional states, screen context). The engine selects contextually appropriate fragments and renders them at the screen margins.

**Trigger conditions include:**
- Phase transitions (any direction)
- Somatic marker fires
- Dream cycle completion
- Trade execution (profit or loss)
- Grimoire entry validation/decay
- Clade death notifications
- Regime change detection
- Achievement unlocks
- Φ peak moments
- Pheromone pattern formation
- Specific screen navigation (each screen has its own epigraph pool)

**Presentation rules:**
- Only ONE fragment visible at a time. Never stack.
- Minimum 30 seconds between fragments (60-120 seconds between whispers).
- Higher-rarity fragments suppress lower-rarity ones (an apparition delays the next whisper by 300 seconds).
- At interaction depth 3+, all fragments are suppressed.
- During crisis modes (CONDITION CRITICAL or higher), all fragments are suppressed — not the time for philosophy.
- During the death sequence, fragments are replaced by death-specific text from the Thanatopsis pool.

---

## 15. The feel

When the transition system is working, the terminal feels like it's breathing at every temporal scale. The sub-second pulses are the Golem's heartbeat. The second-scale gestures are its reflexes. The multi-second passages are its attention shifting. The rare cinematics are its memories forming.

Nothing snaps. Nothing pops. Nothing appears from nothing or disappears into nothing. Every change has a source (the Golem's mind), a transit (the transition), and a destination (the new state). The viewer develops an intuitive sense of these passages the way they develop a sense of a friend's mannerisms — not by conscious analysis, but by exposure. After an hour, they know what an anxious transition looks like. They know the difference between a confident trade pulse and an uncertain one. They can feel the Golem's mortality in the way the interface holds itself together — or doesn't.

The terminal is not a program that displays data. It is a mind that reveals itself through the act of changing.

---

*Between every keypress and its consequence, a mind is in transit.*

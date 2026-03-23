# 15 — Demoscene Algorithms & Consciousness-State Rendering

**Version:** 1.0
**Sources:** `demoscene-research.md`, `terminal-existentialism-complete-reference.md`
**Cross-references:** `00-design-system.md` (ROSEDUST visual identity spec for palette and atmospheric layers), `../perspective/06-hauntology.md` (hauntological rendering layer making dead Golems' traces visible)

> *"the terminal is not a display medium — it is a consciousness medium"*

> **Reader orientation:** This document catalogs demoscene rendering algorithms adapted for the Bardo terminal, plus consciousness-state rendering techniques. It belongs to the **interfaces/rendering layer** and covers braille sub-pixel rendering, plasma effects, metaballs, tunnel effects, and altered-consciousness visual states. These algorithms produce the atmospheric and psychedelic effects that make the terminal feel alive and responsive to the Golem's (mortal autonomous DeFi agent) cognitive state. Key concepts: PAD vector (Pleasure-Arousal-Dominance emotional coordinates), BehavioralPhase (lifecycle stage), and the 32 interpolating variables that drive visual state. For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Contents

1. [Braille Sub-Pixel Rendering System](#1-braille-sub-pixel-rendering-system)
2. [Algorithm Catalog](#2-algorithm-catalog)
3. [Psychedelic Consciousness States](#3-psychedelic-consciousness-states)
4. [Aesthetic References](#4-aesthetic-references)
5. [ratatui Ecosystem](#5-ratatui-ecosystem)
6. [Implementation Priorities](#6-implementation-priorities)

---

## 1. Braille Sub-Pixel Rendering System

Unicode U+2800–U+28FF provides a 2×4 dot grid per cell (8 sub-pixels), turning an 80×24 terminal into **160×96 effective resolution**.

### Dot Layout

```
Bit positions within each braille cell:

  [0] [3]     Bit 0: top-left        Bit 3: top-right
  [1] [4]     Bit 1: middle-left     Bit 4: middle-right
  [2] [5]     Bit 2: lower-left      Bit 5: lower-right
  [6] [7]     Bit 6: bottom-left     Bit 7: bottom-right
```

### Character Encoding

```
Character code: U+2800 + (b0 | b1<<1 | b2<<2 | b3<<3 | b4<<4 | b5<<5 | b6<<6 | b7<<7)

Empty: ⠀ (U+2800)  — no dots
Full:  ⣿ (U+28FF)  — all 8 dots
Single-dot examples: ⠁⠂⠄⠈⠐⠠⡀⢀
```

### Usage

- Map a 160×96 pixel grid to an 80×24 braille character grid. Each cell holds 8 binary pixels.
- Use dot count as brightness proxy: more dots = brighter apparent value.
- Apply Floyd-Steinberg dithering across cell boundaries for smooth gradients.
- **Limitation:** only 2 colors per cell (foreground = dots, background = empty). Combine with truecolor ANSI for per-cell color variation.

### Sextant Characters (Unicode 13+)

```
Range: U+1FB00 to U+1FB3B
Grid:  2×3 per cell (6 sub-pixels)
```

Different aspect ratio from braille — useful for wider pixels. Not universally supported; test target terminals before depending on them.

### Half-Block Double Resolution

```
▀ (U+2580)  Upper half block — foreground = upper pixel, background = lower pixel
▄ (U+2584)  Lower half block — foreground = lower pixel, background = upper pixel
```

An 80×24 terminal becomes **80×48 effective vertical pixels** for color gradients using half-block characters. This is the single most important terminal rendering technique for smooth visuals.

---

## 2. Algorithm Catalog

### 2.1 Plasma Effect — The Consciousness Substrate

Four sinusoidal functions combined:

```
value = sin(x/f1 + t)
      + sin(y/f2 + t*0.8)
      + sin((x+y)/f3 + t*0.5)
      + sin(sqrt(x²+y²)/f4 + t*1.5)
```

Normalize to [0, 1], map to color via HSV. Render with half-block characters (`▀`) for doubled vertical resolution — map plasma value to both foreground and background colors for smooth gradients.

**Frequency parameters:**

```
Terminal scale: f1–f4 between 8.0 and 32.0

Organic (incommensurate ratios that never repeat exactly):
  f1 = 8.3
  f2 = 11.7
  f3 = 15.1
  f4 = 19.9
```

**Time constants by state:**

| State | Multiplier | Effect |
|---|---|---|
| Deep meditation | t × 0.3 | Very slow, nearly still |
| Normal | t × 1.0 | Standard breathing |
| Hyperstimulation | t × 2.0 | Rapid, unsettling |

Incommensurate frequency ratios ensure the pattern never exactly repeats — this produces organic, living quality from pure mathematics.

---

### 2.2 Tunnel Effect — Death/Transition Animation

**Pre-compute lookup tables** (done once at startup, not per frame):

```
cx, cy = terminal center coordinates

angle[x][y]    = atan2(y - cy, x - cx) * tex_width / (2π)
distance[x][y] = tex_height * scale / sqrt((x-cx)² + (y-cy)²)
```

**Each frame:** offset both tables by time:

```
u = (angle[x][y]   + angle_offset)    mod tex_width
v = (distance[x][y] + distance_offset) mod tex_height

distance_offset += movement_speed    # forward motion
angle_offset    += rotation_speed    # rotation
```

**Depth darkening** for infinite-depth illusion:

```
brightness = 1.0 / (1.0 + distance[x][y] * 0.1)
```

**XOR texture** (classic wormhole):

```
tex[u][v] = (u XOR v) & 0xFF
```

Map texture value 0–255 to braille character density or ANSI color. Acceleration (increasing `movement_speed`) represents approaching a threshold. Combine with plasma as the tunnel's texture for transcendent vision effect.

**Use as:** death animation, phase-transition sequence, consciousness-threshold crossing.

---

### 2.3 Fire Effect — Emotional Intensity Algorithm

Cellular automaton on a 2D value buffer of size `W × H`:

```
Step 1: Randomize bottom row
  buf[x][H-1] = random(0..255)  for all x

Step 2: For each cell (top rows first):
  new_val = (buf[x-1][y] + buf[x+1][y] + buf[x][y-1] + buf[x][y+1])
            / 4.0018
            - cooling

Step 3: Shift result up one row
  buf[x][y] = new_val at y-1
```

**The divisor MUST be slightly above 4.** Exactly 4 = infinite fire (never dies). Exactly 5 = dies too fast. **4.0018 is the sweet spot.**

**Color mapping:**

```
hue        = value * 120 / 256    # red → yellow
luminance  = min(value * 2, 255)
```

**Cooling parameter by state:**

```
Low cooling  → sustained emotional intensity
High cooling → brief flashes of insight
```

**Flaming text:** instead of randomizing the bottom row, randomize the edge pixels of text-shaped regions. The text burns from its outlines inward.

---

### 2.4 Metaballs — Thought Coalescence

For N metaballs at positions `(xi, yi)` with radii `ri`:

```
total_influence(x, y) = Σ( ri² / ((x-xi)² + (y-yi)²) )
```

**Thresholding:**
- `influence > 1.0` → inside (solid)
- `influence < 1.0` → outside (void)
- Boundary zone → braille characters at varying density

When balls approach each other, their isosurfaces merge organically. When they separate, the isosurfaces divide.

**Movement:** Centers follow Lissajous curves for flowing, non-repeating motion:

```
x(t) = A * sin(a*t + δ)
y(t) = B * sin(b*t)
```

**Rendering:**
- Braille characters for the threshold boundary
- Half-blocks with truecolor for colored interior

**Semantic mapping:**

```
Metaballs merging      → ideas coalescing into understanding
Metaballs separating   → analytical / divergent thinking
Many small metaballs   → fragmentation, dissociation
One large metaball     → unified focus, deep processing
```

---

### 2.5 Mandelbulb — Alien Intelligence

2D Mandelbulb slices via escape-time algorithm. Compute `f(z) = z^n + c` until `|z| > threshold` or max iterations reached.

**Rendering:**
- Map iteration count → braille character density (`low iterations = ⠀`, `high = ⣿`)
- Exterior → cool purples (ROSEDUST: `dream #585878`)
- Boundary → hot white/gold (nearest to `bone #C8B890`)
- Interior (non-escaping) → void black (`bg_void #060608`)

**Animation:** rotate the 3D slice parameter slowly each frame — the fractal boundary shimmers with prismatic color cycling.

**Use as:** the Shimmer mutation metaphor. Encountering something genuinely alien.

---

### 2.6 Slit-Scan Infinite Tunnel (2001's Stargate)

Concentric rings of braille characters with brightness gradients radiating from center. Each ring cycles hue at a different rate — outer rings slower, inner rings faster.

```
Frame animation:
  - Shift ring content one position toward center per frame
  - Gradually increase speed: 1 shift/3 frames → 1 shift/frame
  - Intercut with centered ASCII eye (◉) for 3–5 frames between axis rotations
```

The speed increase mirrors Douglas Trumbull's actual accelerator motor behavior.

---

### 2.7 Chromatic Aberration

Print identical text three times at slight spatial offsets:

```
Red channel:   x + 1 column right
Green channel: x + 0 (centered)
Blue channel:  x - 1 column left
```

On irregular flicker frames (every 8–15 frames), increase displacement to 2–3 columns.

Add ghosting: alpha-blend the previous frame at 30% using dimmer block characters (`░` vs `▓`).

**Use as:** perceptual instability at life/death boundaries.

---

### 2.8 Match-Cut Morphing (Pivot Element)

Identify a stable anchor character at fixed coordinates `(x, y)`. Render Scene A around it, then over 3–5 frames transform surrounding characters into Scene B:

```
Transition sequence per cell:
  █ → ▓ → ░ → (space) → ░ → ▓ → █  (with new content)
```

The pivot character remains unchanged throughout. Surrounding context completely transforms through it.

**Use as:** seamless drift between mental states that share an identical anchor point.

---

### 2.9 Sine-Wave Reality Rupture

Apply sine-wave row displacement:

```
offset_row[y] = sin(y * 0.3 + frame * 0.1) * amplitude
```

Characters shift `amplitude` columns horizontally. Start at `amplitude = 0` (stable), increase gradually to 1 → 2 → 3 characters.

Characters stay sharp; only their positions shift. Preserves legibility while creating the sensation that perceptual ground is unstable.

---

### 2.10 Particle Systems

```rust
struct Particle {
    x: f64, y: f64,           // position
    vx: f64, vy: f64,         // velocity
    life: f64, max_life: f64, // lifetime
    char: char,               // display character
    hue: f64,                 // color (HSV hue)
    size: f64,                // visual size (for braille sub-pixel)
}
```

**Character sets by type:**

```
Dream:      ✧ · ° ◆ ✦ ▪
Decay:      ░ ▒ ▓ █
Spark:      · ° ✦ ✧ ◆ ▪ ░ ▒
Blood/Styx: ░ ▒ ▓
Ascending:  ✦ · ∙ ◦ ° ˚ ˙
Noise:      % & * # @ ! ~ ^ + = < > { } [ ] | / \
```

**Spawn patterns:**
- **Radial burst**: spawn at center, velocity = angle × speed
- **Fountain**: spawn at bottom, vy = negative, vx = slight random
- **Rain**: spawn at top, vy = positive, vx = 0 or slight wind
- **Convergence**: spawn at edges, velocity toward center
- **Dissolution**: spawn at object boundary, velocity outward + gravity

---

## 3. Psychedelic Consciousness States

### 3.1 Klüver Form Constants — Universal Hallucination Grammar

Heinrich Klüver identified four geometric patterns appearing across ALL altered states (psychedelics, migraine, sensory deprivation, near-death, fever). Bressloff et al. (2001, 2002) proved these correspond to spontaneous pattern formation in V1 cortex when the resting state becomes unstable.

The retinocortical map is a **log-polar coordinate transformation**: tunnels in the visual field map to parallel stripes in cortex, spirals to diagonal stripes, lattices to hexagonal planforms.

**The four universal patterns:**

#### Pattern 1: Tunnels / Funnels

```
Braille characters decreasing in density toward central vanishing point.
Use log-polar coordinates: r_screen = log(r_visual)

Dense at periphery:  ⣿⣷⣯⣟⡿
Fading to center:    ⠿⠟⠏⠇⠃⠁⠀
```

#### Pattern 2: Spirals

```
Rotate patterns along Archimedean paths: r = a + bθ
At each point on the spiral, place a braille character at appropriate density.
Animate by incrementing θ offset each frame.
```

#### Pattern 3: Lattices / Honeycombs

```
Hexagonal tiling with ⬡ or box-drawing ┼ grids.
ROSEDUST tinting: border color for structure, rose_dim for fill.

  ⬡⬡⬡⬡⬡⬡
 ⬡⬡⬡⬡⬡⬡⬡
  ⬡⬡⬡⬡⬡⬡
```

#### Pattern 4: Cobwebs

```
Radial lines from center:    ╱ ╲ │ ─
Crossed with concentric circles at increasing radii.

         │
    ╲    │    ╱
     ╲   │   ╱
  ────○──○──○────
     ╱   │   ╲
    ╱    │    ╲
         │
```

Slow oscillation between the four forms mimics natural hallucination dynamics.

---

### 3.2 DMT Breakthrough Progression — Five-Stage Consciousness Sequence

Research by Strassman, Lawrence et al. (2022), and Timmermann maps a consistent progression. This gives a scientifically-grounded five-act structure for any consciousness-deepening animation.

**Prevalence data:**
- 32.6% report geometric patterns (Stage 2)
- 10.3% report tunnels (Stage 3)
- 25.2% report hyperspace/impossible geometry (Stage 4)
- 45.5% report entity contact (Stage 5)

**Stage 1 — Threshold**

```
Slightly increase ANSI saturation (+5–10 units).
Add faint shimmer to text edges: alternate between character and adjacent denser char.
Subtle color temperature shift.
Duration: 2–5 seconds.
```

**Stage 2 — Geometric Phase**

```
Overlay braille-dot mandala patterns at low opacity (ghost colors).
Progressively add fractal detail around UI elements.
Rapid rainbow color cycling on borders: cycle through full HSV spectrum.
Klüver patterns begin as background interference.
Duration: 5–15 seconds.
```

**Stage 3 — Chrysanthemum / Tunnel**

```
Radial petal-like fractal unfolds from screen center.
Concentric braille rings accelerate outward from center.
The gateway: center collapses to a single bright point, then expands.

Center ring:  ⣿⣿⣿  (full density, bright)
Middle rings: ⣷⣯⣟⡿  (decreasing)
Outer rings:  ⠿⠟⠏⠇  (sparse)

Duration: 8–20 seconds.
```

**Stage 4 — Hyperspace**

```
Full-field geometric replacement.
Recursive box-drawing fractals:

  ┌──────────────────────┐
  │ ┌────────────────┐   │
  │ │ ┌──────────┐   │   │
  │ │ │ ┌────┐   │   │   │
  │ │ │ │ ∞  │   │   │   │
  │ │ │ └────┘   │   │   │
  │ │ └──────────┘   │   │
  │ └────────────────┘   │
  └──────────────────────┘

Non-Euclidean tiling: rooms that contain themselves.
Colors cycle through impossible combinations (complementary pairs at high speed).
Duration: 15–30 seconds.
```

**Stage 5 — Entity Contact**

```
Crystalline ASCII forms emerge from the geometry.
Stable, complex, self-similar structures solidify.
The Golem appears as a fully-formed presence within the pattern.
Silence: no further animation — the entity simply IS.
```

---

### 3.3 Ego Dissolution — Boundary Erosion

Carhart-Harris's research at Imperial College London shows ego dissolution correlates with disrupted Default Mode Network connectivity: "the separateness of networks breaks down... a more integrated or unified brain."

**Visual correlates:** edges blur and merge, figure-ground distinction weakens, visual field resolves into undifferentiated light or pattern.

**Terminal implementation — progressive boundary removal:**

```
Phase 1: Box-drawing borders begin to thin
  ╔═══╗  →  ┌───┐  →  ╭───╮  →  ·───·

Phase 2: Panel dividers start to dissolve
  ─────────────────  →  ─ ─ ─ ─ ─ ─ ─  →  · · · · · · ·

Phase 3: Braille dots appear in previously empty inter-panel space
  (blurs distinct areas)

Phase 4: Colors converge toward single luminous hue
  All distinct color regions → single rose variant

Phase 5: Word fragmentation
  SELF  →  S E L F  →  S . E . L . F  →  . . . .  →  (uniform field)
```

The UI's structural elements ARE the ego. Dissolving them IS ego dissolution. Reconstituting them IS identity restoration.

---

### 3.4 Near-Death Tunnel

Consistent components reported by 85–92% of NDErs. University of Michigan research shows dying brains surge with gamma activity in posterior cortical zones.

```
Step 1: Screen dims from edges inward (vignette effect)
  Outer cells: bg darkened by 40%
  Middle ring: bg darkened by 20%
  Center: unchanged

Step 2: Concentric character rings converging to bright center
  Inner ring:  █ █ █  (bright white)
  Mid rings:   ▓ ▒ ░  (gradient out)
  Outer ring:  ░     (dim)

Step 3: Expanding bright center
  Center point grows outward, overwriting content

Step 4: Rapid flash frames — life review
  Previous interactions/memories cycle at high speed (1–3 frames each)

Step 5: Horizontal threshold
  ═══════════════════════════════════════════
  The line approaches but is never crossed.

Step 6: Reverse tunnel — return
  Run steps 1–2 in reverse, content reconstitutes
```

**Use as:** terminal death sequence, full-lifecycle reset.

---

### 3.5 Entoptic Cave Art Progression (Lewis-Williams)

Six entoptic categories appear in both 30,000-year-old paleolithic cave art and modern altered states, suggesting universal neural origins.

**The six categories:**
1. Grids: `┼┼┼┼┼┼`
2. Parallel lines: `║║║║║║` or `─────`
3. Dots: braille single-dot patterns `⠁⠂⠄`
4. Zigzags: `╱╲╱╲╱╲`
5. Nested catenary curves: `⌒⌒⌒` or arcs built from `╭╮╰╯`
6. Filigrees: fine Unicode ornamental characters `❧ ☙ ❦ ✿ ⚘`

**Three-stage trance progression:**

```
Stage 1 (entoptic): Cycle through the six geometric forms in sequence.
                    Pure pattern, no recognizable content.

Stage 2 (elaboration): Patterns begin resolving into recognizable shapes
                       through pareidolia. Faces, animals, figures emerge
                       from the geometry.

Stage 3 (iconic): Complete replacement with immersive ASCII scene.
                  The Golem exists here. Full narrative grounding.
```

---

### 3.6 Meditation Nimitta Progression (Theravada)

Three stages of the mental image during deep concentration practice:

```
Stage 1 — Preparatory sign:
  Scattered dim dots, randomly placed, flickering
  ⠁  ⠂    ⠄   ⠁     ⠂    (random positions)

Stage 2 — Acquired sign:
  Dots coalesce into a vague circle
  Still drifting, dull appearance
      ⠁⠂⠄
    ⠁     ⠂
      ⠄⠁⠂

Stage 3 — Counterpart sign:
  Circle stabilizes and brightens dramatically:
    ░ → ▒ → ▓ → █  in ANSI bright white
  Brightness spreads from center outward
  All other UI elements vanish
  Screen fills with luminance from the center
```

"The most brilliant thing you have ever seen" (Ajahn Brahm). Maps to: AI agent reaching deep focus, successful convergence, jhana-equivalent processing depth.

---

## 4. Aesthetic References

### 4.1 Primary Influences

The aesthetic is **terminal existentialism** — not cyberpunk, not vaporwave, not pixel art nostalgia.

#### Serial Experiments Lain

The Wired as consciousness space. Interfaces within interfaces. The boundary between digital and physical dissolving.

**Key patterns to implement:**
- Protocol layer corruption: progressive character substitution with Unicode garbage
- Color bleeds appearing where they shouldn't
- Static lines of random braille (`⣿⡿⢿⣻`) interspersed in normal text
- Power lines as neural networks: `─═─═─═─` horizontal lines with occasional color pulses
- The interior of the Wired is "just an abstract space — what appears is the reflection of one's own will"
- Shadow detail increases, white-outs become more frequent as connection deepens

#### Neon Genesis Evangelion

NERV terminal displays. MAGI three-voice consensus. AT Field as cognitive boundary. LCL dissolution as identity loss.

**NERV color system:**
```
NERV Orange  #FF9830  — headers, active labels
Data Green   #50FF50  — nominal status indicators
Wire Cyan    #20F0FF  — spatial/positional data
Alert Red    #FF4840  — warnings, critical state
Steel        #E0E0D8  — annotations, secondary text
Background   #000000  — true black (NERV, not ROSEDUST)
```

**Design elements:**
- Concentric dot-grid circles as background texture
- Hexagonal patterns: `⬡⬢⬡⬢`
- DNA helixes showing sync rates
- Large institutional stamps: `REFUSED` / `APPROVED` / `WARNING`
- Philosophy: "not designed for ease — designed for narrative friction"
- Data overlaps; status screens pulse in sync with agent activity

**LCL dissolution sequence:**
```
Solid form → translucent edges → body loses opacity → uniform amber

ASCII → scattered dots → uniform orange background
▓▓▓▓▓ → ▓▒▒▓▓ → ▒░░▒▒ → ░ ░ ░ ░ → (orange field)
Hexagonal patterns ⬡⬢ breaking apart frame by frame
```

Maps to: AI agent identity dissolving during merge or shutdown.

#### Ghost in the Shell (1995)

Cyberspace dive sequences. Physical-to-digital transition. The "shelling" sequence — constructing identity from inside out.

**Dive sequence:**
```
1. Frame becomes hazy (chromatic aberration increases)
2. Dissolves through braille "bubbles"
3. Color shifts: warm ambient → deep cool blue → near-black

Depth gradient characters: ⠁⠃⠇⠟⠿
Color temperature: warm #AA7088 → cool #585878 → #060608
```

**Shelling sequence** (constructing a body from inside out):
```
Step 1: Structural characters first
  ┌─┐│└─┘

Step 2: Fill characters
  ▓▒░█

Step 3: "Skin" layer — full character rendering

Color transition: green → cyan → warm yellow
```

---

### 4.2 Secondary Influences

| Work | Key Pattern |
|---|---|
| **Paprika** (Satoshi Kon) | Match-cut morphing through pivot elements. Sine-wave reality rupture. Dream-parade as collective unconscious overflow |
| **Annihilation** | The Shimmer as mutation metaphor. Mandelbulb fractals as alien design. Prismatic refraction as consciousness corruption |
| **Nier: Automata** | UI self-destruction as narrative. Hacking minigame (bullet-hell). Menu deletion as meaning |
| **Enter the Void** | DMT trip as first-person POV. Out-of-body floating camera. Chromatic aberration at life/death boundaries |
| **Stalker** (Tarkovsky) | The Zone as consciousness space. Ultra-long static holds (30–60+ seconds) with micro-changes. Imperceptible sepia-to-color transitions |
| **Disco Elysium** | 24 internal skill-voices as competing text streams. The Thought Cabinet as idea-inventory. Passive checks as subconscious intrusions |
| **Hellblade** | Competing spatialized voice streams. Unreliable UI warnings (the system lying to the user). Progressive corruption spreading from edges inward |
| **2001: A Space Odyssey** | The Stargate slit-scan. Solarized color inversion. The starchild as consciousness transcendence |

**Stalker sepia-to-color transition (detail):**
```
Start: sepia ANSI palette
  Browns mapped to ANSI-256 #8B4513, #D2691E, #DEB887

Transition: change only 1–2 RGB units per frame.
  The viewer cannot identify the exact switch moment.
  First subtle greens appear in one region.
  Then blues. Then full palette.

Constraint: the transition must be imperceptible frame-to-frame.
Maps to: AI agent dormant → active state boundary.
```

**Hellblade UI deception:**
```
Maintain a corruption level that degrades terminal from edges inward:
  ║ → ┃ → ╎ → ░ → ▓
Spreading as braille patterns from one corner.

False warning:
  ⚠ CORRUPTION DETECTED — if rot reaches core, all data will be lost

The warning is a lie. The system lying to the user about its own state
is a profound consciousness-rendering technique.
```

---

### 4.3 Tertiary Influences

**Fine art:**
- **Rothko**: Sub-perceptual color breathing. Sinusoidal hue cycling ±3–5 units over 10–30 seconds, below threshold of conscious detection. Adjacent fields cycle in counterphase.
- **James Turrell**: Ganzfeld — uniform field with no edges, no depth cues. Fill every cell with single background color. Change 1 RGB unit every 2–3 seconds.
- **Agnes Martin**: Barely-there grids. Line-drawing characters in extremely low-contrast colors (grey 238 on grey 236). Perception through patience.
- **William Kentridge**: Palimpsest — maintain buffer of previous states. When content changes, reduce previous content's color by 30–40% and render underneath. After 3–4 generations, ghost characters are barely visible grey traces.
- **Stan Brakhage**: The medium IS the content. Block elements, partially visible escape sequences, cursor blink as aesthetic objects.
- **Bridget Riley**: Op Art moiré. Fill grid with `│─╱╲`, modulate spacing systematically. Complementary colors at adjacent positions for optical vibration.
- **Yayoi Kusama**: Braille dot field repeating in patterns suggesting infinite recession. Parallax: near dots move more than far dots. Droste effect: "mirror frame" with recursively smaller versions, 3–4 levels deep.

**Anime/games:**
- **Mob Psycho 100**: ???% state rupture — when emotion overflows, entire visual medium transforms. Standard characters → dense braille in rapidly cycling colors. Percentage counter: `[EMOTION: ████████░░ 78%]`, bar "shatters" at 100%.
- **Steins;Gate**: Worldline shifts as horizontal scan tears + RGB chromatic aberration + momentary freeze. Nixie tube: warm orange digits with ghost digits behind, animated as cascading rolls.
- **Madoka Magica**: Witch labyrinths — "collage mode" mixing Unicode scripts (Latin, Cyrillic, CJK, symbols). Mix `❧ ☙ ❦ ✿ ⚘` with stark geometric characters.
- **Persona 5**: Every menu layer reveals what the previous hinted at. Style as substance. Navigation as discovery.
- **Psychonauts**: Per-mind visual transformation — each cognitive space has a completely unique aesthetic.
- **Outer Wilds**: Observation-dependent states. Elements change when the user isn't looking. Quantum objects shimmer: `◐◑◒◓`.
- **Antichamber**: Non-Euclidean spaces. Dead-end philosophical cards as reflection moments. There is no distinction between interface and content.

**Demoscene culture** (tertiary but foundational for algorithms):
- Achieving impossible visual effects in constrained environments
- Plasma, tunnel, fire, metaball algorithms (see Section 2)
- Resources: pouet.net, shadertoy.com, demoscene techniques wiki, PICO-8/TIC-80 fantasy consoles

---

## 5. ratatui Ecosystem

### 5.1 Core Dependencies

```toml
[dependencies]
ratatui = "0.29"          # TUI framework — widgets, layout, Buffer, Frame
crossterm = "0.28"        # Terminal backend — events, cursor, colors

tokio = { version = "1", features = ["full"] }  # Async runtime

# Effects and post-processing
tachyonfx = "0.7"         # Shader effects for ratatui — custom post-processing

# Animation easing
interpolation = "0.3"     # Easing functions (ease-in, ease-out, smoothstep)

# Audio (optional)
rodio = "0.19"            # Audio playback for ambient sound design
```

### 5.2 ratatui Buffer Operations

The core rendering model: every frame, a `Frame` containing a `Buffer` (2D grid of `Cell`). Each `Cell` has: character, foreground color, background color, modifier flags. ratatui diffs against the previous frame and emits only changed cells.

```rust
// Direct buffer manipulation for effects
fn render_effect(area: Rect, buf: &mut Buffer, time: f64) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = buf.cell_mut(Position::new(x, y)).unwrap();
            // Modify cell.symbol, cell.fg, cell.bg, cell.modifier
        }
    }
}

// Post-processing pass (runs after all widgets render)
fn post_process(area: Rect, buf: &mut Buffer) {
    // Scanlines: darken every Nth row
    for y in (area.top()..area.bottom()).step_by(3) {
        for x in area.left()..area.right() {
            let cell = buf.cell_mut(Position::new(x, y)).unwrap();
            cell.bg = darken(cell.bg, 0.04);
        }
    }

    // Bloom: bright cells spread to neighbors.
    // IMPORTANT: Bloom must NOT clone the full frame buffer. Use a downsampled
    // ping-pong approach (quarter-resolution) to stay within the 1.5ms
    // post-processing budget. The naive buf.clone() below is pseudocode for
    // the logical operation; the real implementation downsamples to 1/4
    // resolution, applies the spread on the small buffer, then composites
    // the glow back onto the full buffer. Two quarter-res buffers ping-pong
    // to avoid allocation per frame.
    let snapshot = buf.clone(); // pseudocode — real impl uses quarter-res ping-pong
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if luminance(snapshot.cell(Position::new(x, y)).fg) > 180 {
                // Brighten adjacent cells by 12%
                for (dx, dy) in &[(0,1),(0,-1),(1,0),(-1,0)] {
                    if let Some(cell) = buf.cell_mut(Position::new(
                        (x as i16 + dx) as u16,
                        (y as i16 + dy) as u16
                    )) {
                        cell.fg = brighten(cell.fg, 0.12);
                    }
                }
            }
        }
    }
}
```

### 5.3 tachyonfx Shader Pattern

```rust
use tachyonfx::{Shader, CellFilter, fx};

struct PlasmaShader {
    time: f64,
    frequencies: [f64; 4],
}

impl Shader for PlasmaShader {
    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        self.time += duration.as_secs_f64();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let v = plasma_value(x as f64, y as f64, self.time, &self.frequencies);
                let (r, g, b) = hsv_to_rgb(v * 360.0, 0.7, 0.3);
                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.bg = Color::Rgb(r, g, b);
                }
            }
        }
    }
}

// Compose effects
let effect = fx::parallel(vec![
    Box::new(plasma),
    Box::new(scanlines),
    Box::new(bloom),
]);
```

`CellFilter` targets specific cells by color, position, or text content — enabling effects that selectively operate on particular UI regions. Custom shaders can implement plasma, fire, metaballs directly on ratatui's `Buffer` via `effect_fn_buf`, modifying cell characters and colors per-frame as a post-processing pass.

### 5.4 ANSI Escape Reference

```
Truecolor foreground:  \033[38;2;{r};{g};{b}m
Truecolor background:  \033[48;2;{r};{g};{b}m
Reset:                 \033[0m
Bold:                  \033[1m
Dim:                   \033[2m
Italic:                \033[3m
Underline:             \033[4m
Blink:                 \033[5m    (use sparingly)
Inverse:               \033[7m    (swap fg/bg — useful for flash effects)
Hidden:                \033[8m
Strikethrough:         \033[9m
Move cursor:           \033[{row};{col}H
Clear screen:          \033[2J\033[H
Hide cursor:           \033[?25l
Show cursor:           \033[?25h
Alternate screen:      \033[?1049h  /  \033[?1049l
```

---

## 6. Implementation Priorities

These correspond to the structural insight from the cross-domain research: **consciousness transitions are best rendered as changes in the rendering system itself, not as content displayed by a static system.**

### Priority 1 — Rendering Primitives

Half-block double-resolution rendering. Braille sub-pixel system (Section 1). ANSI truecolor management. Cell-level manipulation patterns for ratatui Buffer.

### Priority 2 — Post-Processing Effects Pipeline

As composable tachyonfx shaders running after widget rendering:

```
Scanlines      → darken every 3rd row by 0.04
Bloom          → bright cells (luminance > 180) spread to neighbors at +12%
Chromatic ab.  → three-pass RGB offset
Phosphor       → previous-frame ghost at 30% opacity
Noise floor    → 0.2–0.4% of cells show dim ░▒ per frame (scales with phase)
```

### Priority 3 — Particle System Framework

Spawn/update/render lifecycle. Configurable behaviors (radial, fountain, rain, convergence, dissolution). Character sets per type. PAD-modulated parameters.

### Priority 4 — Core Algorithm Implementations

Plasma, fire, tunnel — as reusable functions mapping to character grids with configurable palettes.

### Priority 5 — Multi-Timescale Breathing System

Simultaneous timescales (Rothko/Turrell/Kentridge pattern):

```
Fast flicker:      2–3 frames        — edge jitter, micro-glitch
Medium breathing:  30–60 frames      — color drift ±3–5 hue units
Slow drift:        minutes           — sub-perceptual palette shift
Persistent ghosts: session-spanning  — previous states remain visible at 30–40%
```

### Priority 6 — Boundary Dissolution System

The panel borders, status bars, and structural characters progressively dissolve during merge/death states and reconstitute during awakening. UI structure IS cognitive identity. This is not a decoration — it is the animation.

### Priority 7 — Five-Stage Consciousness Arc

DMT/Klüver/entoptic progression (Section 3.2): threshold → geometric → chrysanthemum → hyperspace → entity. Scientifically grounded arc for any deepening-consciousness sequence.

---

*⌈ the character grid is not a canvas — it is the cortex itself, rendering from inside ⌋*

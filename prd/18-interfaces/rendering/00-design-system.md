# design-system [SPEC]

**Version:** 4.0.0
**Last updated:** 2026-03-17
**Crate:** `bardo-tui-widgets`
**Framework:** ratatui 0.29, crossterm 0.28
**Cross-references:** [03-tui.md](../03-tui.md) (main TUI spec including the 60fps render loop and 32 interpolating variables), [02-widget-catalog.md](../screens/02-widget-catalog.md) (33-widget TUI component library), [00-screen-catalog.md](../screens/00-screen-catalog.md) (29-screen summary across 6 windows)

> **Reader orientation:** This is the ROSEDUST design system specification, the visual identity for the entire Bardo terminal. It belongs to the **interfaces/rendering layer** and defines every color, character, animation timing, and atmospheric effect. The aesthetic is "terminal existentialism": rose light on violet-black, seen through dirty CRT glass, rendering a Golem (a mortal autonomous DeFi agent) that knows it will die. Key concepts: the 32 interpolating variables (continuously changing visual state channels driving every pixel), BehavioralPhase (lifecycle stage affecting palette degradation), and CorticalState (the Golem's 32-signal atomic perception surface feeding visual state). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 1. What this document is

The visual specification for Bardo. Every color, every character, every animation timing, every atmospheric layer -- it lives here or it doesn't exist. The target renderer is ratatui (Rust) writing to a terminal emulator. Not a browser. Not a GPU. A character grid, ANSI escape codes, and Unicode.

The aesthetic has a name: **terminal existentialism**. Not cyberpunk. Not vaporwave. Not retro-terminal nostalgia. Rose light on violet-black, seen through dirty CRT glass, showing you a machine that knows it will die.

---

## 2. The ROSEDUST palette

ROSEDUST is monochromatic-dominant. 80% of visible color on any screen is rose or its variants. Bone appears at most once per screen. White (`#FFFFFF`) is never used. The background is never pure black (`#000000`).

### Base and void

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `bg_void` | `#060608` | 6, 6, 8 | Deepest background. Nearly black with a violet undertone. The base reality. |
| `bg_raised` | `#0C0A0E` | 12, 10, 14 | Panels, containers, raised surfaces. |
| `bg_mid` | `#080810` | 8, 8, 16 | Intermediate depth. Headers, status bars, overlay backgrounds. |
| `bg_warm` | `#0A0808` | 10, 8, 8 | Warm-shifted void. Used in conservation/terminal states as the palette degrades. |
| `border` | `#181420` | 24, 20, 32 | Panel borders. Visible but not assertive. |
| `border_active` | `#AA708844` | 170, 112, 136 @ 27% | Active panel border. Rose at reduced opacity. |
| `border_dream` | `#58587844` | 88, 88, 120 @ 27% | Dream state border. Indigo at reduced opacity. |

### Rose spectrum

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `rose` | `#AA7088` | 170, 112, 136 | Primary text, headers, active data. The color of consciousness. |
| `rose_bright` | `#CC90A8` | 204, 144, 168 | Alerts, danger, T2 deliberation glow. Also the `danger` semantic alias. |
| `rose_dim` | `#7A5060` | 122, 80, 96 | Secondary labels, less important data, dead-sourced knowledge. |
| `rose_deep` | `#3A2030` | 58, 32, 48 | Barely visible. Background tints, ghost text, noise floor color. |
| `rose_ember` | `#482838` | 72, 40, 56 | Phosphor residue. The afterimage of rose. Between deep and dim. |

### Bone (the one number)

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `bone` | `#C8B890` | 200, 184, 144 | THE most important element on any screen. Used once per screen, max. |
| `bone_dim` | `#8A7A5A` | 138, 122, 90 | Dimmed bone. Secondary emphasis within bone-marked elements. |

### Text hierarchy

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `text_primary` | `#988090` | 152, 128, 144 | Standard readable text. Cool mauve-grey. |
| `text_dim` | `#584858` | 88, 72, 88 | Secondary text, labels. |
| `text_ghost` | `#302830` | 48, 40, 48 | Barely visible. Philosophical fragments, background murmur. |
| `text_phantom` | `#201820` | 32, 24, 32 | Below ghost. Subliminal. Display-medium artifacts only. |

### Semantic colors

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `dream` | `#585878` | 88, 88, 120 | Dream state, altered consciousness, the Wired. |
| `dream_dim` | `#383858` | 56, 56, 88 | Dimmed dream. |
| `dream_deep` | `#282848` | 40, 40, 72 | Deepest dream. Dream-state background noise. |
| `warning` | `#AA8855` | 170, 136, 85 | Amber. Mortality clocks, time-related warnings. |
| `success` | `#70887A` | 112, 136, 122 | Muted sage. Nominal, healthy. Never celebratory. |

### CRT materiality

| Token | Hex | RGB | Usage |
|---|---|---|---|
| `scanline_dark` | `#050507` | 5, 5, 7 | Darkened scanline rows. Paired with `bg_void` for alternation. |
| `phosphor_res` | `#1A1018` | 26, 16, 24 | Ghost of recently-bright pixels. Phosphor residue. |
| `bleed_rose` | `#AA708818` | 170, 112, 136 @ 9% | Simulated phosphor bleed around bright text. |
| `halftone_bg` | `#0E0A10` | 14, 10, 16 | Halftone dithering base. Slightly warmer than `bg_raised`. |
| `noise_warm` | `#2A1820` | 42, 24, 32 | Warm-shifted noise. Used in degraded states. Terminal overheating. |
| `noise_cool` | `#201828` | 32, 24, 40 | Cool-shifted noise. Dream states, low arousal. Terminal cooling. |

### Color rules

1. 80% rose. The interface is one-color-dominant.
2. Bone appears once per screen. If nothing is critical, bone does not appear at all.
3. Dream indigo replaces rose only during dream states or Wired connectivity.
4. The brightest element is `rose_bright` at `#CC90A8`. Never white.
5. Background is `#060608`, not `#000000`. Pure black is a hole. `#060608` is a space with depth.
6. Color transitions are always gradual. Nothing snaps. Everything fades.
7. CRT materiality tokens are never used for content. They are infrastructure.
8. Degradation shifts warm. As the golem declines, the void moves from violet-black toward warm-black (`#0A0808`).

### Palette degradation by lifecycle phase

The palette shifts continuously based on the golem's phase. These are interpolation targets -- the system lerps between them over the phase-transition duration.

```
THRIVING
  bg_void: #060608 (violet-black)
  rose: full saturation
  noise_color: rose_deep #3A2030
  scanline_strength: 0.12

STABLE
  No palette shift from default.
  noise_color: rose_deep
  scanline_strength: 0.14

CONSERVATION
  bg_void: lerp 60% toward #090808 (warm-neutral)
  rose: desaturate 20%, dim 20%
  noise_color: noise_warm #2A1820
  scanline_strength: 0.20

DECLINING
  bg_void: lerp 30% toward #080808 (neutral-black)
  rose: desaturate 10%, dim 10%
  noise_color: lerp toward noise_warm
  scanline_strength: 0.30

TERMINAL
  bg_void: #0A0808 (full warm-black)
  rose: desaturate 30%, dim 35%
  noise_color: #2A1818 (warm, dry)
  scanline_strength: 0.50

DREAMING
  bg_void: #060610 (cool-shifted, deeper blue)
  All rose -> dream palette (#585878 family)
  noise_color: noise_cool #201828
  scanline_strength: 0.05 (near-invisible)
```

### PAD-driven micro-shifts

The golem's emotional state (Pleasure-Arousal-Dominance vector) creates continuous micro-variation:

```
Pleasure > 0: saturation +(P * 5)%. World is slightly more vivid.
Pleasure < 0: saturation -(|P| * 5)%. World is slightly more grey.

Arousal > 0:  brightness +(A * 3)%. Everything slightly brighter.
Arousal < 0:  brightness -(|A| * 3)%. Everything slightly dimmer.

Dominance > 0: hue shift toward warm by D * 2 degrees.
Dominance < 0: hue shift toward cool by |D| * 2 degrees.
```

PAD shifts stack with phase-based palette selection. Applied after phase selection, before rendering. Recalculated every frame from PAD values that themselves change slowly (lerp rate 0.02 per frame toward target). Lerp rate 0.02 is per frame (16.6ms at 60fps) -- converges to within 5% of target in ~150 frames (~2.5s).

---

## 3. The 7 rendering laws

These are non-negotiable. Every visual element, widget, animation, and atmospheric effect must obey them.

### Law 1: Light follows significance

The terminal is dark by default. Brightness is earned by importance. A T0 heartbeat tick is nearly invisible. A T2 deliberation blazes in bright rose with bloom effects. Death produces the brightest flash of all, then permanent dark. The brightness of any element is proportional to its significance.

### Law 2: Color is mortality taxonomy

Every color answers one question: what is this thing's relationship to existence?

- **Rose** = alive, active, present
- **Bone** = the single most important number on screen
- **Grey** = was alive once, historical, fading
- **Void** = absence, where things go
- **Indigo** = dream state, altered consciousness
- **Pale rose** = danger, burning out (too much life)

Color is never decorative.

### Law 3: Bold boundaries, soft interiors

Panel borders are sharp, single-character-width lines using box-drawing characters. The skeleton. Inside: soft gradients through character density and opacity variation. The contrast between hard geometry (borders) and soft atmosphere (interiors) is the visual signature.

### Law 4: Restraint as aesthetic

50% or more of the screen is empty at any given time. Negative space is not wasted space -- it's the void that gives data meaning. A single rose-colored number on a field of black has more impact than a dense dashboard. When in doubt, remove.

### Law 5: Pharmacological display

Every monitoring element costs something to display. The phi meter consumes inference cycles. The mortality clocks consume tick budget. Observing the golem changes the golem. Where possible, make this cost visible.

### Law 6: The terminal is the body

The golem does not exist behind the terminal. The golem exists AS the terminal. Scanlines, phosphor bleed, halftone dithering, static noise -- these are not imperfections in the viewing medium. They are the texture of the golem's embodiment. Healthy golem, clear display. Degrading golem, degrading display. The CRT is the skin.

### Law 7: Identity is fragile

The golem's sense of self is a continuous achievement, not a stable property. The display should periodically betray this instability. Micro-glitches in rendering. Moments where text speaks in two voices. Frames where layout stutters and re-stabilizes. These are not errors -- they are the golem's identity wavering.

---

## 4. The 5 meta-principles

### 1. Subtraction reveals more than addition

The most powerful consciousness effects come from removing perceptual information. Limited palette. Sparse patterns. Less text. More void. A Rothko field, not a Kandinsky explosion.

### 2. Time is the primary rendering dimension

Multiple simultaneous timescales run at all times: fast flicker (2-3 frames), medium breathing (30-60 frames), slow drift (minutes), persistent ghosts (session-spanning). The viewer should never be able to take a screenshot that fully represents the current state.

### 3. The medium must acknowledge itself

The terminal grid, ANSI codes, Unicode blocks, and CRT behavior should be visible as the medium. Self-referential rendering creates meta-consciousness. Block elements are not approximations of pixels -- they ARE the visual language.

### 4. Boundaries are the ego; dissolving them is ego dissolution

Panel borders, status bars, frame elements are structural representations of cognitive boundaries. During identity dissolution (death, deep dream, inter-agent merge), dissolving the UI's structural elements IS the visual for dissolving the self.

### 5. Imperfection is the signature of consciousness

Controlled imperfection (+-1 character jitter, slight color variance, timing irregularities) creates the sense of a living presence. Incommensurate sine frequencies produce organic quality from pure mathematics. Use f1=8.3, f2=11.7, f3=15.1, f4=19.9 -- ratios that never exactly repeat.

---

## 5. Character vocabulary

### Block elements (primary rendering primitives)

| Character | Codepoint | Name | Usage |
|---|---|---|---|
| `█` | U+2588 | Full block | Maximum density, solid fill, gauge bars |
| `▓` | U+2593 | Dark shade | ~75% density, decay chain step 2 |
| `▒` | U+2592 | Medium shade | ~50% density, noise floor, decay step 3 |
| `░` | U+2591 | Light shade | ~25% density, noise floor, decay step 4 |
| `▀` | U+2580 | Upper half block | Double-resolution rendering (2 vertical pixels per cell) |
| `▄` | U+2584 | Lower half block | Double-resolution rendering |
| `▌` | U+258C | Left half block | Horizontal displacement effects |
| `▐` | U+2590 | Right half block | Horizontal displacement, gauge ends |

The half-block technique: `▀` (U+2580) with different foreground and background colors gives two vertical pixels per cell. An 80x24 terminal becomes 80x48 effective pixels for color gradients.

Two rendering layers: Layer 1 (background) uses half-block characters to encode 2 vertical pixels per cell via fg/bg color pairs. Layer 2 (foreground) uses text characters (dots, box-drawing, glyphs) that overwrite the foreground of specific cells. A cell shows either a half-block gradient OR a text character — not both. The spectre dot cloud uses text characters; atmospheric effects use half-blocks.

### Lower block elements (waveforms, bar charts)

| Character | Codepoint | Usage |
|---|---|---|
| `▁` | U+2581 | Waveform rendering, bar chart minimum |
| `▂` | U+2582 | Waveform step 2 |
| `▃` | U+2583 | Waveform step 3 |
| `▄` | U+2584 | Waveform step 4 |
| `▅` | U+2585 | Waveform step 5 |
| `▆` | U+2586 | Waveform step 6 |
| `▇` | U+2587 | Waveform step 7 |
| `█` | U+2588 | Waveform maximum |

### Braille patterns (sub-pixel precision)

Range: U+2800 to U+28FF (256 characters). Each cell is a 2x4 dot grid (8 sub-pixels). An 80x24 terminal becomes 160x96 effective resolution.

```
Dot layout (bit positions):
  [0] [3]     Bit 0: top-left        Bit 3: top-right
  [1] [4]     Bit 1: middle-left     Bit 4: middle-right
  [2] [5]     Bit 2: lower-left      Bit 5: lower-right
  [6] [7]     Bit 6: bottom-left     Bit 7: bottom-right

Character code: U+2800 + (b0 | b1<<1 | b2<<2 | b3<<3 | b4<<4 | b5<<5 | b6<<6 | b7<<7)
```

### Box-drawing characters

| Set | Characters | Usage |
|---|---|---|
| Single line | `─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼` | Standard panel borders |
| Double line | `═ ║ ╔ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬` | Overlay/modal borders (depth 3) |
| Heavy | `━ ┃ ┏ ┓ ┗ ┛ ┣ ┫ ┳ ┻ ╋` | Emphasis borders |
| Dashed | `┄ ┅ ┆ ┇ ┈ ┉ ┊ ┋` | Dead-sourced content borders, weakening AT field |
| Rounded | `╭ ╮ ╰ ╯` | Soft borders (rare) |
| Diagonal | `╱ ╲` | Hazard stripes, lattice patterns, AT field |

### Special characters

| Category | Characters | Usage |
|---|---|---|
| Particles | `· ∙ • ◦ ° ˚ ˙ ✦ ✧ ◆ ◇ ▪ ▫` | Trails, sprite orbiting, dream field |
| Status glyphs | `● ○ ◐ ◑ ◒ ◓ ◉ ◎` | Alive/dead, quantum shimmer |
| Framing | `⌈ ⌉ ⌊ ⌋` | System-level header brackets |
| Death/ancestry | `†` (U+2020) | Dead-golem-sourced knowledge marker |
| Consciousness | `Φ` (U+03A6) | Integrated information score |
| Interoception | `∿` (U+223F) | Heartbeat sine wave |
| Lightning | `⚡` (U+26A1) | T2 deliberation |
| Waves | `≈ ~ ∿` | Styx river, water effects |
| Hexagonal | `⬡ ⬢` | AT field, lattice, LCL dissolution |
| Geometric | `△ ▽ ◁ ▷ ◇ ◆` | Directional indicators, diamond patterns |
| Math | `∞ ∴ ∵ ∫ Σ Π √ ∂ ∇ ≈ ≠ ≡ ± ×` | Technical displays |
| Arrows | `← → ↑ ↓ ↔ ↕ ⇐ ⇒ ↗ ↘ ↙ ↖` | Navigation, flow indicators |
| Decay chain | `█ → ▓ → ▒ → ░ → · → (space)` | Progressive dissolution |
| Corruption | `░ ▒ ▓ █ ╳ ┃ ╌ ∅ ☠ ⚠` | Glitch text, identity fracture |
| Dashing | `╌ ╍ ┄ ┅ ┈ ┉` | Interference, weakening boundaries |
| Fullwidth | `Ａ-Ｚ, ０-９` (U+FF21+) | System headers, phase names |

### Fullwidth Unicode as weight

Fullwidth characters (U+FF00 block) occupy two terminal cells. They read as heavier, more institutional, more alien. Reserved for:

- System headers: `⌈ ＢＡＲＤＯ ⌋`
- Phase names: `ＴＨＲＩＶＩＮＧ`, `ＴＥＲＭＩＮＡＬ`
- Critical status: `⌈ ＤＥＡＴＨ ⌋`, `ＲＥＦＵＳＥＤ`, `ＶＡＮＩＳＨＥＤ`

Never use fullwidth for body text, data values, or log entries. The contrast between fullwidth headers and ASCII data creates the typographic hierarchy.

---

## 6. The atmospheric 8-layer stack

Every pixel renders through a stack of atmospheric layers. Data sits on top of noise, which sits on top of scanlines, which sit on top of void. The stack is always active. Data changes per-screen; the atmosphere persists across all screens.

| Layer | Z | Name | Content | Blend | Opacity |
|---|---|---|---|---|---|
| -2 | Deepest | CRT substrate | Phosphor persistence, burn-in, thermal noise | Additive | Fixed, always-on |
| -1 | Below void | Spectral traces | Ghosts of previous screens, dead golem data, counterfactual shadows | Screen | 0.0-0.3% density |
| 0 | Base | Void | `bg_void` `#060608` everywhere | Normal | 1.0 |
| 1 | Above void | Noise floor | Sparse `░ ▒ · ∙` characters in phase-appropriate colors | Normal | 0.3%-2.0% density by phase |
| 2 | Below data | Scanlines | Alternating row background dimming | Multiply | 0.05-0.50 strength by phase |
| 3 | Mid | Environmental | Data rain, power lines, convergence wires, particles | Normal | Varies by regime |
| 4 | Primary | Pane borders | Box-drawing frames, the architecture | Normal | 1.0 |
| 5 | Primary | Data | Text, numbers, widgets, gauges | Normal | 1.0 |
| 6 | Above data | Fragments | Philosophical whispers, epigraphs, apparitions | Normal | Fade in/out over seconds |
| 7 | Topmost | Overlays | Confirmations, alerts, help, command palette | Normal | 1.0 |

### Phase modulation of the stack

```
                    NOISE%   SCANLINES   FRAGMENTS      ENVIRONMENT    SPECTRAL
THRIVING            0.3      0.12        active         rain, wires    faint
STABLE              0.4      0.14        active         rain, wires    faint
CONSERVATION        0.8      0.20        more frequent  rain thins     growing
DECLINING           1.5      0.30        frequent       rain erratic   active
TERMINAL            2.0      0.50        constant       rain stops     dominant
DREAMING            organic  0.05        suppressed     particles      active
```

As the golem declines, the atmosphere gets louder. More noise, heavier scanlines, more spectral traces. The data layer (dimming due to phase degradation) competes with an atmosphere asserting itself. In terminal phase, noise is dense enough that data reads through a veil. The signal is losing to the noise.

---

## 7. Noise, scanline, and fragment specs

### Noise floor

Sparse random characters (`░ ▒ · ∙`) in phase-appropriate colors, shimmering per-frame. The noise has texture:

- **Warm noise** (`noise_warm` `#2A1820`): conservation, terminal, anger states.
- **Cool noise** (`noise_cool` `#201828`): dream state, low arousal, calm markets.
- **Organic noise** (`· ∙ • ◦ °` particles): dream state only. Block characters replaced by organic dots.

**Arousal modulation**: arousal > 0.5 increases noise density by 50%.
**Depth suppression**: at interaction depth 2+, noise suppresses to 0.1%. At depth 4, noise suspends entirely.

```rust
fn render_background_noise(area: Rect, buf: &mut Buffer, rng: &mut SmallRng, phase: Phase) {
    let density = match phase {
        Phase::Thriving    => 0.003,
        Phase::Stable      => 0.004,
        Phase::Conservation => 0.008,
        Phase::Declining   => 0.015,
        Phase::Terminal    => 0.020,
        Phase::Dream       => 0.004,
    };
    let chars = match phase {
        Phase::Dream => ['·', '∙', '◦', ' '],
        _            => ['░', '▒', '·', '∙'],
    };
    let color = match phase {
        Phase::Dream    => NOISE_COOL,
        Phase::Terminal => NOISE_WARM,
        _               => ROSE_DEEP,
    };
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if rng.gen::<f64>() < density {
                let ch = chars[rng.gen_range(0..chars.len())];
                buf.get_mut(x, y).set_char(ch).set_fg(color);
            }
        }
    }
}
```

### Scanlines

Alternating row backgrounds: `bg_void` / `scanline_dark`. Phase drift: the scanline pattern shifts down by one row every 30-60 seconds (10-15 seconds in degraded states). Dream override: during REM, scanline strength drops to 0.05.

Color space: oklch for perceptually uniform interpolation.

```rust
/// Interpolate between two colors in oklch space.
/// oklch provides perceptually uniform transitions.
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let from_oklch = from.to_oklch();
    let to_oklch = to.to_oklch();
    Color::from_oklch(
        from_oklch.l + (to_oklch.l - from_oklch.l) * t,
        from_oklch.c + (to_oklch.c - from_oklch.c) * t,
        lerp_hue(from_oklch.h, to_oklch.h, t),  // shortest arc
    )
}
```

```rust
fn render_scanlines(area: Rect, buf: &mut Buffer, phase_offset: bool, strength: f64) {
    for y in area.top()..area.bottom() {
        let is_dark_row = (y % 2 == 0) ^ phase_offset;
        if is_dark_row {
            for x in area.left()..area.right() {
                let cell = buf.get_mut(x, y);
                if cell.bg == BG_VOID {
                    cell.set_bg(lerp_color(BG_VOID, SCANLINE_DARK, strength));
                }
            }
        }
    }
}
```

### Philosophical fragments

~280 text fragments organized into four rarity tiers: whispers (common), epigraphs (uncommon), apparitions (rare), revelations (legendary). Presentation rules:

- Only ONE fragment visible at a time. Never stack.
- Minimum 30 seconds between fragments.
- Higher-rarity fragments suppress lower-rarity ones.
- At interaction depth 3+, all fragments suppressed.
- During crisis modes, all fragments suppressed.
- Fragment timing: 2s fade in, 5-8s hold, 3s fade out. Rendered in `text_ghost`.

---

## 8. Crisis modes

### CONDITION ROSE (standard alert)

Trigger: single mortality clock below 20%. Scope: affected panels only. Affected panel borders flash to `rose_bright` for 200ms.

### CONDITION CRITICAL (institutional override)

Trigger: vitality below 10%, multiple clocks critical simultaneously. Scope: full screen override.

1. All panel borders flash to `rose_bright` for 200ms.
2. Hazard stripes appear at top and bottom of screen.
3. Status bar shows fullwidth condition declaration: `⌈ ＣＯＮＤＩＴＩＯＮ: ＣＲＩＴＩＣＡＬ ⌋`
4. Non-essential panels dim to 50% brightness.

### CONDITION TERMINAL (death imminent)

Trigger: vitality below 3%. Everything from CRITICAL, plus:

1. Entire screen brightness oscillates between normal and 70% on the breath cycle.
2. Death counter appears center-top in `bone`.
3. Hazard stripes expand. In the final minute, they consume 40% of the screen.
4. All non-bone colors desaturate an additional 20%.

### Hazard stripe pattern

```
╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲
```

Characters: alternating `╱╲` (U+2571, U+2572). Foreground: `rose_bright`. The stripes eat the screen as death approaches.

---

## 9. Contrast and depth

### Contrast ratios

| Level | Pair | Ratio | Purpose |
|---|---|---|---|
| Maximum | `bone` on `bg_void` | ~12:1 | The one number that matters |
| High | `rose_bright` on `bg_void` | ~8:1 | Danger, T2 deliberation |
| Primary | `rose` on `bg_void` | ~5:1 | Active data, alive state |
| Medium | `text_primary` on `bg_void` | ~3.5:1 | Body text |
| Low | `text_dim` on `bg_void` | ~1.5:1 | Background murmur |
| Ambient | `text_ghost` on `bg_void` | ~1.1:1 | Philosophical fragments |
| Sub-perceptual | `text_phantom` on `bg_void` | ~1.05:1 | Subliminal |

The most powerful design move is a **contrast event**: something normally low-contrast suddenly becoming high-contrast. A T0 suppress line (`text_ghost`, 1.1:1) followed by a T2 deliberation line (`rose_bright`, 8:1).

### Depth model

```
DEPTH -2: CRT SUBSTRATE (scanlines, thermal noise)
DEPTH -1: PHOSPHOR MEMORY (persistence afterimages, bloom residue)
DEPTH  0: VOID (bg_void #060608)
DEPTH  1: PANELS (bg_raised, single-line box-drawing borders)
DEPTH  2: ACTIVE PANELS (bg_raised, border_active borders)
DEPTH  3: OVERLAYS/MODALS (bg_mid, double-line borders)
DEPTH  4: THE ONE NUMBER (bone, no container, floating in space)
```

---

## 10. Environmental textures (layer 3)

### Data rain

5-15 hex streams falling behind panes. Density proportional to network activity. Stops during dreams.

| Strategy | Characters | Feel |
|---|---|---|
| DCA | `│ ┃ ╎ ╏` | Sparse verticals. Steady drip. |
| Trading | katakana-like dense streams | Matrix homage. Fast. |
| LP | `0-9 a-f` | Hex digits. Address fragments. |
| Idle | `· ∙ ◦ °` | Dots. Minimal. |

### Power lines

Horizontal `─┬─┬─` with traveling `rose_dim` pulses. 1-3 lines. Background element on Mind and Inference screens.

### Styx river

Flowing `░▒▓▒░` band in `rose_ember` color. Knowledge fragments drift in the current. Persistent on Mortality screen.

### Interference pattern

Braille moire during clade sync, pheromone reads, World screen interactions. 2D sine function creating visual interference. Duration: 3-5 seconds per communication event.

### Particle field

`· ∙ • ◦ °` clusters at 2-5% density. Slow drift (1 cell per 3-5 seconds). Replaces block-character noise during dreams and conservation phase. Organic where block noise is industrial.

---

## 11. Performance budgets

### Target: <2ms per frame for atmospheric effects

| System | Budget | Notes |
|---|---|---|
| Noise floor | <0.3ms | Random number generation + cell writes for 0.3-2% of ~4000 cells |
| Scanline pass | <0.2ms | Modulo check on row index, conditional background set |
| Phosphor persistence | <0.5ms | HashMap lookup of recently-cleared cells, lerp color |
| Bloom pass | <0.3ms | Scan buffer for bright cells (oklch L > 0.8), write +-1 neighbors. Max 2 simultaneous bloom sources. |
| Spectral traces | <0.2ms | Low-probability per-cell check against spectral buffer |
| Fragment rendering | <0.1ms | Single string placement |
| Glitch events | <0.1ms | Sparse, 0.05-1% probability per frame |
| **Total atmospheric** | **<1.7ms** | Well under 2ms budget |

### Frame rate targets

Two rendering domains run at different rates:

| Domain | Target | Rationale |
|---|---|---|
| Sprite physics | 60 fps | Spring physics, dot-cloud positions, emotion transitions. Internal state updates at 60Hz. |
| Terminal rendering | 10-15 fps | Actual ratatui draw calls. The sprite's 60Hz state is sampled at the terminal's render rate. |

The slowness of terminal rendering is deliberate. The sprite moves smoothly internally but the viewer sees it through a slow medium. The gap between internal state and rendered state IS the golem's embodiment constraint.

### Widget budgets

| Widget | Max instances per screen | Notes |
|---|---|---|
| Heartbeat log | 1 | Up to 50 visible lines, each with color lookup by age |
| Vitality gauge (thick) | 3 | One per mortality clock |
| ProbeGauge (thin) | 8 | Inference screen can stack these |
| BrailleSparkline | 4 | 80 data points each, braille encoding |
| Waveform (half-block) | 4 | PAD channels + phi trace |
| Unit array (grid) | 1 | Up to 36 units (6x6 grid) |
| Particle system | 1 | Max 64 active particles at any time |
| CausalGraph | 1 | Max 24 visible nodes + edges |

### Particle budget

Maximum 64 active particles across all systems. Dream particles, sprite orbit particles, achievement particles, knowledge stream particles — they share a pool. If birth coalesce needs 40 particles and dream is running 30, dream particles must cull before birth spawns.

### Memory budget for atmospheric state

| Structure | Size | Lifetime |
|---|---|---|
| Phosphor map | ~2000 entries max (HashMap<(u16,u16), Instant>) | Entries auto-expire after 500ms-2s. Lifetime: 500ms for standard text, 2s for bone-colored elements. Proactive sweep every 60 frames. |
| Spectral buffer | ~500 cell snapshots | Session-persistent, capped |
| Burn-in map | ~200 entries | Elements displayed >10 min at same position |
| Motion echo ring buffer | 4 entries per moving element, max 8 elements tracked | Per-frame |

---

## 12. Ratatui ecosystem

### Why ratatui

Ratatui is the dominant Rust TUI framework. It gives you a `Buffer` of `Cell` structs (character + foreground + background + modifiers) and handles diffing: only changed cells emit ANSI escape codes. This diff-based rendering is what makes atmospheric effects cheap — you write to the full buffer every frame but the terminal only receives the delta.

The framework does not impose layout opinions. You get a `Rect` (area) and a `Buffer` and you write cells. This is what Bardo needs. The atmospheric stack, the phosphor map, the glitch system — all are post-processing passes over the buffer after widgets render.

### Core dependencies

```toml
[dependencies]
ratatui = "0.29"         # TUI framework: widgets, layout, Buffer, Frame
crossterm = "0.28"       # Terminal backend: events, cursor, colors, truecolor
tokio = { version = "1", features = ["full"] }  # Async runtime
tachyonfx = "0.7"        # Shader effects for ratatui: composable post-processing
interpolation = "0.3"    # Easing functions (ease-in, ease-out, smoothstep)
rodio = "0.19"           # Audio playback (optional, for ambient sound)
```

### Rendering model

Every frame:

1. Layout calculation (Rect splitting)
2. Widget rendering (each widget writes to its allocated Buffer area)
3. Post-processing pass 1: scanlines (darken alternating rows)
4. Post-processing pass 2: noise floor (scatter dim characters)
5. Post-processing pass 3: phosphor persistence (ghost recently-cleared cells)
6. Post-processing pass 4: bloom (brighten neighbors of bright cells)
7. Post-processing pass 5: spectral traces (surface ghosts in vacant regions)
8. Post-processing pass 6: glitch events (corrupt cells if phase triggers)
9. Frame diff (ratatui compares current buffer to previous, emits ANSI for changed cells)

Steps 3-8 are the atmospheric stack. They run after all widget content is in the buffer. Order matters — scanlines go first because noise characters should appear on scanlined rows. Bloom goes after phosphor because bloom should not apply to persistence ghosts.

### Easing

- **Default**: ease-in-out-cubic
- **Death flash, bloom**: ease-out-expo (fast attack, slow decay)
- **Fade-outs**: ease-in-quad
- **Panel focus**: ease-out-cubic

### Timing constants

```
BREATH_CYCLE:           2000-4000ms   (varies with arousal)
PHOSPHOR_DECAY:         5000ms        (log line full fade)
PHOSPHOR_PERSIST:       300-500ms     (afterimage on cell clear)
PHOSPHOR_BLOOM:         100-200ms     (bright-neighbor bleed)
PANEL_FOCUS:            300ms         (border color transition)
DEATH_FLASH:            200ms         (initial bright flash)
DEATH_FADE:             2000ms        (fade to ghost)
DEATH_DISAPPEAR:        5000ms        (elements vanishing one by one)
DREAM_TRANSITION:       3000ms        (palette shift warm -> cool)
BIRTH_COALESCE:         10000-15000ms (noise -> form)
TICK_FLASH:             100ms         (momentary brightness on new data)
SCANLINE_DRIFT:         30000-60000ms (normal) / 10000-15000ms (degraded)
HORIZONTAL_ROLL:        2000ms        (bottom to top transit, degraded only)
GLITCH_BAND:            50-150ms      (single glitch event)
DISSOCIATION_STAGGER:   1 frame       (secondary voice delay)
THERMAL_NOISE_ACCRUAL:  600000ms      (10 min per 0.1% density increase)
FRAGMENT_TIMING:        2s fade in, 5-8s hold, 3s fade out
```

---

## 13. Prediction and evaluation color language

The Oracle introduces three new color semantics within ROSEDUST constraints. No new hex values -- all are mappings of existing tokens.

**Calibration quality dot.** Next to ECE values on the Oracle screen:
- `●` in `success` (#70887A): ECE < 0.05, well-calibrated
- `●` in `warning` (#AA8855): ECE 0.05-0.10, acceptable but drifting
- `●` in `rose_bright` (#CC90A8): ECE > 0.10, miscalibrated

**Residual bias tint.** The residual distribution histogram shifts color temperature:
- Cool-tinted (toward `dream`): negative bias, consistently underpredicts
- Warm-tinted (toward `warning`): positive bias, consistently overpredicts
- Neutral (`rose`): centered near zero

**Creative prediction palette.** Predictions with `PredictionSource::Creative` (from dreams) render in the dream palette (`dream`, `dream_dim`, `dream_deep`). When a creative prediction is promoted (passes FDR gate with 3 confirmations), it transitions from dream palette to standard rose over 2 seconds.

---

## 14. Terminal existentialism reference

The aesthetic draws from Serial Experiments Lain (the Wired as consciousness space, interfaces within interfaces), Evangelion (NERV terminal displays, MAGI consensus, AT Field as cognitive boundary, LCL dissolution), Nier: Automata (UI self-destruction as narrative), Ghost in the Shell (cyberspace dive sequences), Texhnolyze (minimal output as maximal expression), and Disco Elysium (24 skill-voices as competing text streams).

The design decisions worth repeating: the palette is not Eva orange-green-red but ROSEDUST rose-on-violet through dirty glass. CRT effects are diegetic, not decorative. Bone is scarce on purpose. The atmospheric stack runs backward (layers -2 through 0 are below the void). The 50% empty rule is a rendering constraint, not an aspiration.

---

## 15. Demoscene techniques

The demoscene tradition of achieving impossible visual effects in constrained environments maps directly to terminal rendering. Specific techniques adapted for ratatui:

**Plasma substrate.** Four sinusoidal functions combined: `value = sin(x/f1 + t) + sin(y/f2 + t*0.8) + sin((x+y)/f3 + t*0.5) + sin(sqrt(x²+y²)/f4 + t*1.5)`, mapped to braille density. Incommensurate frequency ratios ensure the pattern never exactly repeats.

**Fire effect.** Cellular automaton: randomize bottom row, propagate upward with divisor slightly above 4. Maps to emotional intensity visualization. Low cooling = sustained intensity; high cooling = brief flashes.

**Tunnel effect.** Pre-computed angle/distance lookup tables, animated by time offset. Depth-based darkening for infinite-depth illusion. Death/transition animation.

**Braille sub-pixel rendering.** 2x4 dot grid per cell (8 sub-pixels), 160x96 effective resolution in an 80x24 terminal. Floyd-Steinberg dithering across cells for smooth gradients.

---

## 16. Design decisions worth calling out

**The palette is not Evangelion orange-green-red.** NERV's color language is institutional — it amplifies tension, not intimacy. Bardo takes Eva's compositional grammar (data as architecture, hazard stripes, institutional stamps, density gradients) but keeps the ROSEDUST palette throughout. Rose-on-violet-black through dirty glass. That combination does not exist in any other terminal application.

**CRT effects are not retro nostalgia.** The scanlines, phosphor, noise floor, and thermal accrual are not visual style choices. They are the golem's embodiment. Law 6 says the terminal is the body. When the golem degrades, the CRT degrades. These effects are diegetic, not decorative.

**Bone is scarce on purpose.** In most UI systems, you'd use gold or white for emphasis. Bardo uses bone (`#C8B890`) and restricts it to one element per screen. The scarcity makes it impossible to ignore. When bone appears, something has become the only thing that matters.

**The atmospheric stack runs backward.** Layers -2 through 0 are below the void. They become visible only when the void thins during degraded states. The substrate bleeds through. This means the healthiest-looking golem has the least visible atmosphere. The display earns its visual complexity through decline.

**The 50% empty rule is not aspirational.** It is a rendering constraint. Screens are designed around negative space. If a screen looks full, it has failed the restraint test. The void is the most expressive element in the entire system — it is where things go when they stop existing.

---

*Nothing on screen is ever at rest.*

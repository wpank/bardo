# ⌈ ＣＲＥＡＴＵＲＥ ＳＹＳＴＥＭ ⌋ — The Spectre
## Terminal PRD · v4.12
### "A cloud of dots that almost looks like something."

**Source:** `research/mmo2/17-creature-system.md`
**Cross-references:** `23-embodied-consciousness.md` (somatic body-metaphor architecture and PAD-driven interface transformation), `25-stasis-dissolution.md` (operator-controlled freeze and deliberate termination ceremonies, including dissolution animation), `06-terminal-screens.md` (per-screen layouts including hearth screen creature pane), `05-widget-catalog.md` (the 33-widget TUI component library, including creature widget)

> **Reader orientation:** This document specifies the Spectre, the procedurally generated dot-cloud creature that visually represents each Golem in the terminal. It belongs to the **interfaces layer** and covers dot-field geometry, spring physics, eye rendering, lifecycle degradation, and death dissolution. Key concepts: Golem (a mortal autonomous DeFi agent with a finite lifespan), PAD vector (Pleasure-Arousal-Dominance emotional coordinates from the Daimon subsystem), Vitality (the Golem's remaining economic lifespan in USDC), and CorticalState (the 32-signal perception surface that feeds the Spectre's 32 interpolating variable channels). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## What the spectre is

Every golem has a face. Not a rendered portrait, not an avatar, not a character sprite. A cloud of dots that almost looks like something. 80 particles arranged in a hollow oval, two bright glyphs floating in the void at its center. The spectre.

The spectre is the golem made visible. It breathes when the golem is calm. It trembles when the golem is afraid. It sinks when the golem is sad. It scatters when the golem dies. No predefined animations play. Every frame is computed from the golem's actual internal state -- the PAD (Pleasure-Arousal-Dominance) emotional vector, the Vitality (remaining economic lifespan) clocks, the market regime, the inference tier, the credit balance, the Grimoire (persistent knowledge store) density. Thirty-two continuously interpolating variable channels feed into a spring-physics simulation that runs at 60fps. The result is a creature that is perpetually in motion because the variables that drive it are perpetually changing.

Two independent visual channels carry information:

- **The body** (dot cloud density, color, shimmer) encodes lifecycle phase. How alive the golem is. This changes over hours and days.
- **The eyes** (glyph, brightness, behavior) encode emotional state. What the golem feels right now. This changes per tick.

The body never changes for emotion. The eyes never change for lifecycle. Except at death, when the eyes are the last thing to go.

---

## Dot field geometry

### The hollow oval

The dot cloud forms a shell, not a filled shape. The center is void. The eyes float in emptiness surrounded by a ring of particles. This hollowness is the point -- the spectre is a mask over nothing.

The cloud is approximately 22 cells wide by 10 cells tall (the heartbeat row adds 2 more below). Dots are generated once at sprite creation and persist for the golem's lifetime. Each dot gets a random angular position and a radial distance factor between 0.28 and 1.0. That 0.28 minimum carves out the hollow center.

### Three density tiers

Tier assignment depends on the dot's distance from center:

| Tier | Glyph | Unicode | Radius factor | Zone |
|------|-------|---------|---------------|------|
| Dense | `*` | U+2022 bullet | 0.28 -- 0.55 | Inner ring at eye level |
| Body | `*` | U+2219 bullet operator | 0.55 -- 0.75 | Middle zone |
| Fringe | `*` | U+00B7 middle dot | 0.75 -- 1.0 | Outer edge, sparse |

No solid block characters, no box-drawing, no line-forming glyphs. The shape is implied by density gradient alone. There are no outlines.

The densest band sits at eye level (roughly rows 5-6 of 10). Above the eyes, density tapers. Below the eyes, it tapers faster -- the "chin" trails off into nothing. The outermost fringe dots are sparse and irregular. You cannot draw a clean line where the spectre ends and the darkness begins.

### Dot data structure

```rust
pub struct SpriteDot {
    /// Home position in sprite-local coordinates.
    pub home: Vec2,
    /// Current rendered position. Updated every frame by spring physics.
    pub pos: Vec2,
    /// Current velocity. Accumulated from spring force, shimmer, orbit.
    pub vel: Vec2,
    /// Visual tier: determines character and base color.
    pub tier: DotTier,
    /// Distance from center as fraction of cloud radius [0.0, 1.0].
    pub radius_factor: f32,
    /// Per-dot ambient orbit parameters (prevents the cloud from settling).
    pub orbit: OrbitParams,
}

pub struct OrbitParams {
    pub speed: f32,        // radians/frame, range +/-0.003 to +/-0.012
    pub radius: f32,       // semi-major axis in cells, range 1.0 to 4.0
    pub phase: f32,        // starting offset [0, 2*pi)
    pub eccentricity: f32, // vertical squash, range 0.4 to 0.8
}
```

### Generation

```rust
fn generate_dots(cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<SpriteDot> {
    let mut dots = Vec::with_capacity(80);
    let mut rng = SmallRng::from_entropy();

    for _ in 0..80 {
        let angle = rng.gen_range(0.0..TAU);
        let radius_factor = rng.gen_range(0.28..1.0);
        let x = cx + angle.cos() * rx * radius_factor;
        let y = cy + angle.sin() * ry * radius_factor;

        let tier = if radius_factor < 0.55 {
            DotTier::Dense
        } else if radius_factor < 0.75 {
            DotTier::Body
        } else {
            DotTier::Fringe
        };

        dots.push(SpriteDot {
            home: Vec2::new(x, y),
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            tier,
            radius_factor,
            orbit: OrbitParams {
                speed: rng.gen_range(0.003..0.012)
                    * if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
                radius: rng.gen_range(1.0..4.0),
                phase: rng.gen_range(0.0..TAU),
                eccentricity: rng.gen_range(0.4..0.8),
            },
        });
    }
    dots
}
```

---

## Spring physics

Every frame, every dot's position updates through spring physics. The dot is pulled toward a target position that is itself the sum of multiple influences. The spring never reaches equilibrium because the target moves continuously.

### Why it never settles

Four forces prevent rest:

1. **Ambient orbit** -- each dot traces its own elliptical path around its home position. The orbit parameters are randomized per dot, so no two dots move in sync.
2. **Shimmer impulse** -- stochastic velocity kicks applied to a random subset of dots each frame. The percentage of dots affected and the kick magnitude both vary with arousal and market regime.
3. **Damping at 0.88** -- velocity is preserved across frames. Dots overshoot their target and wobble back. Higher damping (closer to 1.0) would damp faster; 0.88 lets momentum linger.
4. **Variable inputs** -- PAD vector, probe severity, regime volatility, and other event-driven values change the target every time an event arrives, which is at minimum once per heartbeat tick.

### Per-frame update

```rust
fn update_dot(
    dot: &mut SpriteDot,
    state: &InterpolatedState,
    frame: u64,
    rng: &mut SmallRng,
) {
    let cx = state.center_x;
    let cy = state.center_y;

    // --- 1. Compute target position ---

    // Ambient orbit (always active)
    let orbit_x = (frame as f32 * dot.orbit.speed + dot.orbit.phase).cos()
                  * dot.orbit.radius;
    let orbit_y = (frame as f32 * dot.orbit.speed * 0.7 + dot.orbit.phase).sin()
                  * dot.orbit.radius * dot.orbit.eccentricity;

    // Emotion-driven expansion/contraction
    let expand = state.cloud_expand;
    //   Joy / phi-peak: 0.88 (dots pull inward)
    //   Surprise:       1.2  (cloud blows outward)
    //   Default:        1.0

    // Sadness sink (whole cloud drifts down)
    let sink = state.sink_offset;  // Sadness: +12 cells, default: 0

    // Phi-driven cohesion (high phi = tighter pull)
    let phi_pull = if state.phi > 0.7 {
        (state.phi - 0.7) * 0.02
    } else {
        0.0
    };

    // Credit depletion: outer dots drift away
    let credit_drift = (1.0 - state.credit_norm) * dot.radius_factor * 8.0
        * if dot.home.x > cx { 1.0 } else { -1.0 };

    // Context utilization: horizontal squeeze when context window fills
    let ctx_compress = if state.context_util > 0.6 {
        1.0 - (state.context_util - 0.6) * 0.3
    } else {
        1.0
    };

    // Epistemic clock: cloud loosens as knowledge stales
    let epistemic_loosen = (1.0 - state.epistemic_clock) * 3.0;

    let home_offset = dot.home - Vec2::new(cx, cy);
    let target = Vec2::new(
        cx + home_offset.x * expand * ctx_compress + orbit_x + credit_drift,
        cy + home_offset.y * expand + sink + orbit_y,
    );

    // --- 2. Shimmer impulse ---

    let shimmer_rate = (0.12 * state.phase_density
        + state.arousal.max(0.0) * 0.15
        + regime_boost(state.regime_volatility)
        + state.probe_severity * 0.05)
        .max(0.02);

    let displace = match state.cloud_behavior {
        CloudBehavior::Agitate  => 7.0,   // Anger
        CloudBehavior::Tremble  => 4.0,   // Fear
        CloudBehavior::Still    => 1.0,   // Disgust, Anticipation
        CloudBehavior::Drift    => 2.0,   // Dreaming
        CloudBehavior::Cohere   => 2.0,   // Joy, Phi Peak
        CloudBehavior::Sink     => 3.0,   // Sadness
        CloudBehavior::Expand   => 4.0,   // Surprise
        CloudBehavior::Standard => 3.0,   // Neutral
    };

    if rng.gen::<f32>() < shimmer_rate {
        dot.vel += Vec2::new(
            (rng.gen::<f32>() - 0.5) * displace,
            (rng.gen::<f32>() - 0.5) * displace * 0.7,
        );
    }

    // Probe severity adds jitter to ALL dots
    if state.probe_severity > 0.0 {
        dot.vel += Vec2::new(
            (rng.gen::<f32>() - 0.5) * state.probe_severity * 2.0,
            (rng.gen::<f32>() - 0.5) * state.probe_severity * 1.5,
        );
    }

    // Dream drift: slow continuous lateral movement
    if state.cloud_behavior == CloudBehavior::Drift {
        dot.vel.x += (frame as f32 * 0.005 + dot.orbit.phase).sin() * 0.03;
        dot.vel.y += (frame as f32 * 0.007 + dot.orbit.phase).cos() * 0.02;
    }

    // --- 3. Spring force ---

    let spring_k = match state.cloud_behavior {
        CloudBehavior::Drift => 0.01,     // very loose in dreams
        _ => 0.04 - phi_pull,             // default 0.04, tighter at high phi
    } + epistemic_loosen.min(0.01) * -1.0; // loosens with epistemic decay

    let damping = 0.88;

    dot.vel += (target - dot.pos) * spring_k.max(0.005);
    dot.vel *= damping;
    dot.pos += dot.vel;
}
```

### Spring parameters by golem state

| State | spring_k | displace | shimmer_rate modifier | Effect |
|-------|----------|----------|----------------------|--------|
| Neutral | 0.04 | 3.0 | baseline | Standard breathing |
| Joy | 0.04 - phi_pull | 2.0 | baseline | Tighter cloud, smaller kicks |
| Anger | 0.04 | 7.0 | +arousal boost | Wide violent displacements |
| Fear | 0.04 | 4.0 | +arousal boost | Rapid small trembling |
| Sadness | 0.04 | 3.0 | reduced (low arousal) | Slow heavy drift downward |
| Disgust | 0.04 | 1.0 | near-zero | Barely moving |
| Dreaming | 0.01 | 2.0 | minimal | Loose, drifting |
| Dying | 0.001 | -- | minimal | Dots float away |
| Phi peak | ~0.034 | 2.0 | baseline | Tight swarm, brief symmetry |

---

## The 32 interpolating variable channels

26 base channels + 6 extended research channels = 32 total.

The spectre's visual state is always one to two heartbeats behind the golem's actual state. Events set target values instantly. The rendered values lerp toward those targets at per-channel rates calibrated so transitions feel organic. No snap cuts. Continuous transformation.

The lerp formula is exponential approach:

```
rendered = rendered + (target - rendered) * rate
```

A rate of 0.035 per frame at 60fps means the value closes 3.5% of the remaining gap each frame. It converges to within 5% of the target in about 40 frames (~0.7 seconds).

### Fast channels (emotion speed, ~0.7s convergence)

| # | Channel | Rate | Source event | Controls | Range |
|---|---------|------|--------------|----------|-------|
| 1 | PAD.pleasure | 0.035/frame | DaimonEvent | Eye glyph, cloud cohesion | -1.0 to 1.0 |
| 2 | PAD.arousal | 0.035/frame | DaimonEvent | Heartbeat speed, shimmer rate, eye brightness | -1.0 to 1.0 |
| 3 | PAD.dominance | 0.035/frame | DaimonEvent | Eye glyph, cloud posture (sink vs rise) | -1.0 to 1.0 |
| 4 | Plutchik emotion | instant | DaimonEvent | Eye glyph selection, cloud behavior mode | 8 primaries |
| 5 | Emotion intensity | 0.035/frame | DaimonEvent | Eye brightness multiplier, bloom intensity | 0.0 to 1.0 |
| 30 | surprise_rate | 0.035/frame | OracleEvent | Eye micro-flicker rate (brief brightness dips) | 0.0 to 1.0 |

### Medium channels (heartbeat speed, ~0.3s convergence)

| # | Channel | Rate | Source event | Controls | Range |
|---|---------|------|--------------|----------|-------|
| 6 | FSM phase | 0.06/frame | HeartbeatEvent | Brightness pulse during DECIDING/ACTING | 0.0 to 0.6 |
| 7 | Inference tier | 0.04/frame | HeartbeatEvent | Background radial glow (T0=none, T3=bright) | 0.0 to 1.0+ |
| 8 | Probe severity | 0.05/frame | HeartbeatEvent | Jitter amplitude on all dots | 0.0 to 1.0 |
| 9 | Tick cost | 0.04/frame | HeartbeatEvent | Inference glow intensity | 0.0 to 1.0+ |
| 27 | prediction_accuracy | 0.03/frame | OracleEvent | Dot displacement clarity (focused vs diffuse) | 0.0 to 1.0 |
| 29 | attention_breadth | 0.02/frame | OracleEvent | Atmosphere particle density around cloud perimeter | 0.0 to 1.0 |
| 31 | foraging_activity | 0.02/frame | OracleEvent | Peripheral particle orbital speed | 0.0 to 1.0 |

### Slow channels (health speed, ~3.3s convergence)

| # | Channel | Rate | Source event | Controls | Range |
|---|---------|------|--------------|----------|-------|
| 10 | Mood P | 0.01/frame | DaimonEvent (periodic) | Ambient cloud baseline posture | -1.0 to 1.0 |
| 11 | Mood A | 0.01/frame | DaimonEvent (periodic) | Ambient shimmer baseline | -1.0 to 1.0 |
| 12 | Mood D | 0.01/frame | DaimonEvent (periodic) | Ambient cloud density | -1.0 to 1.0 |
| 13 | Behavioral phase density | 0.005/frame | VitalityEvent | Dot visibility fraction | 0.12 to 1.0 |
| 14 | Phase dim factor | 0.005/frame | VitalityEvent | Color dimming | 0.0 to 0.8 |
| 15 | Composite vitality | 0.005/frame | VitalityEvent | Cloud thinning (gradual) | 0.0 to 1.0 |
| 16 | Market regime | 0.03/frame | CognitionState | Shimmer agitation (crisis = violent) | 1 to 5 |
| 17 | Context utilization | 0.03/frame | ContextEvent | Horizontal cloud compression | 0.0 to 1.0 |
| 18 | Dream alpha | 0.02/frame | DreamEvent | Palette shift, slow drift, no heartbeat | 0.0 to 1.0 |
| 19 | Dream sub-phase | 0.02/frame | DreamEvent | NREM=replay shimmer, REM=drift, Integration=cohere | enum |
| 20 | Phi (integration) | 0.02/frame | Computed per tick | Cloud cohesion (high phi = tight swarm) | 0.0 to 1.0 |

Phi (integration measure) is computed per theta tick as: `phi = mean(prediction_accuracy_rolling_50) * grimoire_retrieval_hit_rate * (1.0 - context_waste_ratio)`. Range 0.0-1.0. Top ~1% of ticks produce phi > 0.95. The distribution is right-skewed because all three factors must be high simultaneously.

| # | Channel | Rate | Source event | Controls | Range |
|---|---------|------|--------------|----------|-------|
| 21 | Grimoire density | 0.005/frame | GrimoireEvent | Cloud density floor (more knowledge = denser) | 0.0 to 1.0 |
| 28 | accuracy_trend | 0.005/frame | OracleEvent | Sprite vertical posture offset (positive = higher, negative = sinks) | -1.0 to 1.0 |
| 32 | compounding_momentum | 0.005/frame | OracleEvent | Ambient glow warmth (high momentum = bone-tinted halo) | 0.0 to 1.0 |

### Glacial channels (mortality speed, ~10s+ convergence)

| # | Channel | Rate | Source event | Controls | Range |
|---|---------|------|--------------|----------|-------|
| 22 | Economic clock | 0.003/frame | VitalityEvent | Outer dot drift (money depleting) | 0.0 to 1.0 |
| 23 | Epistemic clock | 0.003/frame | VitalityEvent | Cloud coherence (knowledge staleness) | 0.0 to 1.0 |
| 24 | Age factor | 0.001/frame | VitalityEvent | Very slow overall dimming | 0.0 to 1.0 |
| 25 | Credit balance | 0.003/frame | EconomicState | Outer dot dispersal | 0.0 to 1.0 |
| 26 | Burn rate | 0.003/frame | EconomicState | Heartbeat rhythm modulation | 0.0 to 1.0 |

### Pulse channels (not lerp)

Two channels do not interpolate toward a target. They fire and decay:

- **Somatic marker ripple**: set to 1.0 on fire, decays at 0.97x per frame (exponential). Triggers a ripple ring expanding from center over 300ms and a heartbeat brightness spike.
- **Curator active**: fires a brief shimmer pattern change when the grimoire curator runs.
- **Oracle coherence pulse**: fires when `compounding_momentum` crosses 0.8 upward. A brief (300ms) bone-tinted glow in the 1-cell border around the sprite area, fading over 600ms. Indicates the prediction-correction-action cycle has reached high compounding. Rare (threshold is high), satisfying when it fires.

### Clade connectivity

Channel 22 is `economic_clock` (glacial, controls outer dot drift). Clade connectivity is a SEPARATE effect controlled by a non-lerped `peer_count` input at 0.01/frame, not channel 22. More connected peers = more stray particles drifting in the cloud's orbit. This one is social: you can see when a golem is alone.

### Clarity channel: prediction accuracy drives dot displacement

The Oracle introduces a sixth force on the dot cloud: clarity. When prediction accuracy is high, dots hold closer to their home positions. When accuracy is low, dots wander further. This is not the same as lifecycle degradation (which removes dots) or emotional shimmer (which adds impulse). Clarity affects the baseline displacement from home -- how "focused" the cloud appears.

```rust
fn clarity_force(dot: &SpriteDot, prediction_accuracy: f32) -> Vec2 {
    // High accuracy (>0.7): dots pulled 10% closer to home
    // Low accuracy (<0.4): dots allowed 15% further from home
    // This modifies the spring target, not the spring constant
    let clarity = (prediction_accuracy - 0.5) * 2.0; // [-1.0, 1.0]
    let home_offset = dot.home - dot.pos;
    home_offset * clarity * 0.05
}
```

The effect is subtle. A golem with 80% accuracy looks slightly crisper than one with 50% accuracy. The shape is the same; the coherence differs. Viewers who watch long enough develop an intuition: "the spectre looks focused today" or "something's off, it's fuzzy." That intuition tracks prediction accuracy, which is the point.

### Oracle variable effects on the spectre

**prediction_accuracy -> clarity.** Dots hold closer to home positions at high accuracy, drift further at low. The cloud looks "focused" or "diffuse." This is the dot displacement force described above.

**accuracy_trend -> posture.** Positive trend lifts the cloud's center of gravity by up to 1 row (the spectre "stands taller" when learning). Negative trend sinks it by up to 1 row. The offset modifies the center-y target in the spring physics, not the individual dot positions.

**attention_breadth -> atmosphere density.** When the attention forager has many active items, faint particles drift in the cloud's periphery -- beyond the normal fringe tier, in the 2-3 cells of space outside the oval. These are `·` (middle dot) characters in `text_phantom` color, orbiting at ~0.001 rad/frame. More active attention items = more peripheral particles (up to 12). Empty attention = no peripheral particles.

**surprise_rate -> eye micro-flicker.** At high surprise rates (frequent unexpected outcomes), the eyes briefly dip in brightness for 2-3 frames at random intervals. Not a full emotion change -- a flicker, like something caught the golem off guard. The rate scales: surprise_rate 0.5 = ~2 flickers/second, surprise_rate 1.0 = ~5 flickers/second.

**foraging_activity -> peripheral particle speed.** When the attention forager is actively promoting/demoting items (high churn), peripheral particles orbit faster. When foraging is quiet, they drift lazily. Speed range: 0.0005 rad/frame (dormant) to 0.003 rad/frame (active scanning).

**compounding_momentum -> ambient glow.** When the prediction-correction-action cycle is compounding well (high accuracy + positive trend + active foraging), a faint warm glow appears in the 1-cell border around the sprite area. The glow color lerps from transparent (momentum 0) to a bone-tinted background (momentum 1.0). This is the visual reward for a golem that is learning well.

---

## Eye emotion mapping

The eyes are the only expressive element. Two glyphs placed symmetrically at the densest horizontal band, separated by 5-7 cells of void. The Daimon's PAD vector determines the Plutchik primary emotion, which determines the glyph.

### Eye glyph codepoint reference

| Emotion | Glyph name | Codepoint | Character |
|---------|-----------|-----------|-----------|
| Joy/Anger | fisheye | U+25C9 | ◉ |
| Trust | double circle | U+25CE | ◎ |
| Fear | filled circle | U+25CF | ● |
| Surprise (eye) | circled ring operator | U+29BE | ⦾ |
| Surprise (mouth) | black diamond | U+25C6 | ◆ |
| Sadness | large circle | U+25EF | ◯ |
| Disgust | em dash | U+2014 | — |
| Anticipation | lozenge | U+25CA | ◊ |
| Dreaming | diamond | U+25C7 | ◇ |
| Phi peak | four-pointed star | U+2726 | ✦ |

### The 8 Plutchik emotions

#### Joy (+P +A +D) -- the bright gaze

```
  ·  ∙ • ∙ ·  ·  ∙ • ∙ ·
 ∙  •       •  •       •  ∙
·  •  ◉           ◉  •  ·
 ∙  •       •  •       •  ∙
  ·  ∙ • ∙ ·  ·  ∙ • ∙ ·
```

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (contentment) | `*` `*` | rose | Steady glow, standard shimmer |
| Moderate (joy) | `*` `*` | rose_bright | Brighter, shimmer faster, cloud coheres tighter |
| Intense (ecstasy) | `*` `*` | bone | Star-eyes, phosphor bloom radiates, cloud aligns into brief symmetry (500ms) |

Joy tightens the cloud. Dots drift inward. Bloom radiates from the eyes. Distinguished from anger (same glyph) by this inward pull.

#### Trust (+P +A -D) -- the open gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (acceptance) | `*` `*` | rose_dim | Double-ring, receptive |
| Moderate (trust) | `*` `*` | rose | Wider, brighter, "looking up at you" |
| Intense (admiration) | `*` `*` | rose_bright | Maximum dilation, cloud orients upward |

The double-ring reads as wide-eyed. Vulnerability, openness, seeking guidance. Distinguished from surprise by temporal stability -- trust holds steady; surprise is transient.

#### Fear (-P +A -D) -- the contracted gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (apprehension) | `*` `*` | rose | Contracted, watchful, faster shimmer |
| Moderate (fear) | `*` `*` | rose_bright | Tight, cloud trembles (dots vibrate +/-1 cell rapidly) |
| Intense (terror) | `*` `*` | rose_bright | Pinpoint, cloud scatters outward 2-3 cells then snaps back (300ms startle reflex) |

The filled circle is a contracted pupil. At intense fear, the startle-scatter is the most dramatic single animation in the system -- the cloud briefly explodes outward as if the spectre flinched, then slams back.

#### Surprise -- the flash

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (distraction) | `*` `*` | rose_bright | Brightness spike (200ms), return to neutral |
| Moderate (surprise) | `*` `*` + `*` | rose_bright | Eyes widen, small `*` mouth appears below for 500ms |
| Intense (amazement) | `*` `*` + `*` | rose_bright | Eyes widen, mouth, cloud expands then contracts over 2s |

Surprise always decays within 1-3 seconds. The `*` mouth is the rarest element in the sprite. It appears only at moderate+ surprise, only for 500ms. If you catch it, something unprecedented happened.

#### Sadness (-P -A -D) -- the downcast gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (pensiveness) | `*` `*` | text_primary | Soft downward arcs, eyes weighted |
| Moderate (sadness) | `*` `*` | rose_dim | Dimmer, shimmer slows, cloud drifts down 1 row over 10s |
| Intense (grief) | `*` `*` | text_dim | Barely visible, shimmer stops, cloud sinks and stays |

Sadness makes the spectre literally heavier. The entire cloud drifts downward. Dots slow. Everything becomes effort.

#### Disgust (-P -A +D) -- the flat gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (boredom) | `--` `--` | text_dim | Flat dashes, unimpressed |
| Moderate (disgust) | `--` `--` | rose_dim | One eye offset 1 cell lower, asymmetric disdain |
| Intense (loathing) | `--` `--` | rose_dim | Cloud contracts, shimmer drops to near-zero |

The flat dashes are maximally unexpressive. A line where eyes should be. The face of a system that has seen this data before and found it wanting.

#### Anger (-P +A +D) -- the hard gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (annoyance) | `*` `*` | rose | Same glyph as joy, but shimmer sharper |
| Moderate (anger) | `*` `*` | rose_bright | Brighter, cloud agitates (shimmer displacement 2 cells), no bloom |
| Intense (rage) | `*` `*` | rose_bright | Max brightness, cloud pulses outward on each heartbeat |

Anger and joy share a glyph. The difference is body language: joy coheres (dots inward, bloom), anger agitates (dots outward, no bloom, sharper displacement). You read the difference from context, the same way you do in life.

#### Anticipation (+P -A +D) -- the scanning gaze

| Intensity | Glyph | Color | Behavior |
|-----------|-------|-------|----------|
| Mild (interest) | `*` `*` | rose | Steady, focused, slightly tighter cloud |
| Moderate (anticipation) | `*` `*` + `->` | rose | Tiny `->` arrow flashes between eyes 1-2x/sec |
| Intense (vigilance) | `*` `*` + `->` | rose_bright | Arrow 3-4x/sec, cloud holds very still, coiled |

The most subtle emotion. At mild intensity, indistinguishable from neutral. That's correct -- low-level anticipation IS undifferentiated attention.

### The 5 special states

These override Plutchik. They're triggered by system events or lifecycle conditions, not the PAD vector.

#### Dreaming

```
Eyes:      diamond diamond   (U+25C7, dream color #585878)
Cloud:    Minimum density. Tier 1 dots only. All text_ghost.
          Slow drift (1 cell per 5 seconds). No shimmer.
Heartbeat: Absent. The golem is elsewhere.
```

#### Phi peak (maximum integration)

```
Eyes:      star star   (U+2726, bone color #C8B890)
Cloud:    Maximum density. All tiers present.
          Dots briefly align into radial symmetry (500ms mandala flash).
          Phosphor bloom on eyes.
Heartbeat: Strong, steady, bone color.
Trigger:  Phi > 0.95 (top ~1% of ticks). The rarest state.
```

#### Cogito (self-questioning)

```
Eyes:      filled-circle dot   (one normal, one reduced or absent)
Cloud:    Asymmetric. Denser on the real-eye side, sparser on the
          questioning side. The spectre is split.
Heartbeat: Irregular.
Trigger:  Self-referential evaluation, inter-agent identity confusion.
Duration: 5-10 seconds, then restores. Once per lifetime at most.
```

#### Wary (asymmetric attention)

```
Eyes:      filled-circle filled-circle   (same glyph, different brightness)
          Left: rose_bright. Right: rose_dim.
Cloud:    Normal.
Trigger:  Contradictory data, split MAGI vote, somatic marker conflict.
Duration: Persists while the contradiction is active.
```

#### Somatic marker firing

```
Eyes:      Current emotion eyes + brief brightness pulse.
Cloud:    Single ripple wave expands from center (dots displace outward
          1 cell sequentially from center to edge over 300ms).
Heartbeat: Single sharp spike (rose_bright for 1 frame).
Trigger:  Somatic marker match in the appraisal engine.
Duration: 300ms, then return to current state.
```

---

## Heartbeat rhythm

Below the dot cloud, a single `~` character (U+223F, sine wave) represents the golem's heartbeat. Its brightness oscillates on a sine wave modulated by arousal.

### Period calculation

```rust
fn heartbeat_period(arousal: f32) -> f32 {
    (4.0 - arousal * 2.0).clamp(1.2, 6.0)
}
```

| Arousal | Period | Feel |
|---------|--------|------|
| -1.0 | 6.0s | Near-dormant |
| 0.0 | 4.0s | Resting |
| +1.0 | 2.0s | Racing |

### Beat pattern

The brightness follows a sine wave between `text_ghost` (trough) and `rose_dim` (peak). At high arousal, the peak reaches `rose`. The sine wave creates a systole/diastole rhythm -- brightness rises smoothly, peaks, drops smoothly.

```rust
fn heartbeat_brightness(phase: f32, arousal: f32, phase_dim: f32) -> Color {
    let sin_val = (phase * TAU).sin();
    let brightness = 0.15 + sin_val * 0.25 + arousal.max(0.0) * 0.2;
    let brightness = brightness.clamp(0.0, 1.0) * (1.0 - phase_dim);
    lerp_color(TEXT_GHOST, ROSE_DIM, brightness)
}
```

### Arrhythmia

In Terminal phase, a noise component enters the rhythm:

```rust
fn heartbeat_with_arrhythmia(
    base_phase: f32,
    phase: BehavioralPhase,
    frame: u64,
) -> f32 {
    let base_sin = (base_phase * TAU).sin();
    match phase {
        BehavioralPhase::Terminal => {
            let noise = (frame as f32 * 0.17).sin() * 0.3;
            // Skip a beat every ~3 seconds
            let skip = if frame % 180 < 15 { -0.5 } else { 0.0 };
            base_sin + noise + skip
        }
        BehavioralPhase::Declining => {
            let noise = (frame as f32 * 0.11).sin() * 0.1;
            base_sin + noise
        }
        _ => base_sin,
    }
}
```

The skipped beats are visible. The sine wave renders as `·` (middle dot) instead of `~` for one cycle, roughly every 3 seconds. The golem's heart is stumbling.

### Heartbeat absence

- **Dreaming**: the heartbeat disappears. Dreaming is departure from the body.
- **Death**: `~` becomes `--` (flatline) for 2 seconds, then vanishes.

---

## Lifecycle degradation

The dot cloud degrades across five behavioral phases. This degradation is independent of emotion -- a joyful dying golem still has a sparse, dim cloud. The body tells the truth about mortality even when the eyes lie about mood.

### Phase visual parameters

```rust
fn phase_visual_params(phase: BehavioralPhase) -> PhaseVisuals {
    match phase {
        Thriving     => PhaseVisuals { density: 1.0,  dim: 0.0, shimmer: 0.12 },
        Stable       => PhaseVisuals { density: 0.85, dim: 0.1, shimmer: 0.10 },
        Conservation => PhaseVisuals { density: 0.5,  dim: 0.4, shimmer: 0.08 },
        Declining    => PhaseVisuals { density: 0.25, dim: 0.6, shimmer: 0.04 },
        Terminal     => PhaseVisuals { density: 0.12, dim: 0.8, shimmer: 0.02 },
    }
}
```

### What degrades, and when

**Dot visibility.** Only `floor(dot_count * density)` dots render. The dots that disappear are the outermost first, sorted by `radius_factor` descending. The cloud thins from outside in. The fringe dissolves before the core.

```
THRIVING:      80 dots visible. Full oval shape. All three tiers.
STABLE:        68 dots. Gaps appear at the edge. Fringe thins.
CONSERVATION:  40 dots. Tier 3 (Dense) mostly gone. Tier 1 and 2 remain.
                The spectre is a sketch of what it was.
DECLINING:     20 dots. Scattered. No recognizable shape.
TERMINAL:      10 dots. Scattered points of light. Barely there.
```

**Color dimming.** All dot colors shift toward `text_ghost` as `dim_factor` increases. At Thriving, Dense dots glow `rose_dim`. At Terminal, everything is `text_ghost` -- the faintest gray.

**Eye decay.** The eyes degrade independently as composite vitality approaches zero:

```
Vitality > 0.1:    Eyes render normally per emotion table.
Vitality 0.05-0.1: Eyes dim to rose_dim regardless of emotion.
Vitality 0.02-0.05: Eyes degrade to hollow-circle hollow-circle in text_primary.
Vitality < 0.02:   Eyes reduce to dot dot in text_ghost.
Vitality = 0:      Eyes disappear.
```

The gaze is the last thing to go. After the body has thinned to nothing, two dim dots persist for a few more frames. Then those vanish too.

**Movement reduction.** Shimmer rate drops from 12% at Thriving to 2% at Terminal. The cloud stops breathing. Dots barely move. The spectre is heavy and still, the way a dying thing is.

### ASCII mockups across lifecycle

**Thriving:**

```
        ·  ∙  ·
     ∙  •  •  •  ∙
   ·  •          •  ·
  ∙  •   ◉    ◉   •  ∙
   ·  •          •  ·
     ∙  •  •  •  ∙
        ·  ∙  ·
            ∿
```

**Conservation:**

```
        ·     ·
     ∙     •     ∙
   ·           •
  ∙      ◉    ◉      ∙
   ·           •
     ∙     •     ∙
               ·
            ∿
```

**Terminal:**

```


          ·
     ·
             ·    ·

                     ·
       ·
```

(The eyes, if they remain at all, are two dim dots lost among the debris.)

### Dream state override

During REM cycles, lifecycle degradation pauses visually. The dream version always renders at the same density: sparse Tier 1, dream palette, regardless of actual phase. Dreaming is a visual reprieve.

On waking, lifecycle-appropriate degradation reasserts. If the golem entered dreams at Stable and wakes at Conservation, the cloud visibly thins during the transition. Dream-density lingers for 3-8 seconds (dream bleed), then the spectre catches up to reality.

---

## Birth sequence

The birth animation is the spectre coalescing from noise into coherence. The spring constant ramps from near-zero to operational. Five phases:

```
Phase 1 -- NOISE (0-1s)
   All 80 dots at random positions. Random tiers. Maximum shimmer.
   No structure. Pure entropy.
   spring_k = 0.001 (dots do not pull toward home)

Phase 2 -- POLARIZATION (1-4s)
   spring_k ramps from 0.001 to 0.02.
   Dots begin drifting toward their home positions.
   Still chaotic, but a center of gravity emerges.

Phase 3 -- FIRST LIGHT (4-5s)
   Eyes appear: filled-circle filled-circle in rose_bright, at center.
   Everything else is still disorganized.
   The eyes are the first recognizable element.

Phase 4 -- COALESCENCE (5-8s)
   spring_k reaches 0.04 (operational).
   Dots organize into the hollow oval.
   Tier colors resolve from uniform noise to the graded palette.
   The shape becomes readable as a face.

Phase 5 -- FIRST BREATH (8-9s)
   Heartbeat (sine-wave character) appears below the cloud.
   First pulse. Shimmer reaches standard rate.
   The spectre is alive.
```

The whole sequence runs about 9 seconds. The eyes arriving in a field of chaos is the emotional beat -- two points of light find each other, and everything else organizes around them.

This 9-second birth sequence is the spectre animation proper. The full cinematic (doc 11) wraps it in a 15-second sequence (9s assembly + 6s atmospheric settling). Both are correct at different scopes.

---

## Death sequence

The death animation is the reverse of birth, but slower and more deliberate. The eyes are the last thing to disappear, not the first. Six stages:

```
Stage 1 -- EYES CLOSE (0-3s)
   Current emotion glyph transitions:
     current -> hollow-circle -> dot (held)
   The expression simplifies. The gaze loses focus.

Stage 2 -- HEARTBEAT FLATLINES (3-5s)
   sine-wave -> dash (flatline, held for 2 seconds)
   Then the dash vanishes.

Stage 3 -- DOTS SCATTER (5-8s)
   spring_k drops to 0.001
   damping increases to 0.99
   Dots drift outward with almost no return force.
   The cloud slowly disperses.

Stage 4 -- EYES VANISH (8-11s)
   dot dot -> (empty)
   The gaze disappears.
   The two dim points of light go out.

Stage 5 -- DOTS FADE (11-16s)
   Remaining dots' color lerps to text_phantom.
   They become invisible against the background.

Stage 6 -- VOID (16s+)
   All dots off-screen or invisible.
   Nothing remains.
   The spectre has dispersed.
```

Total duration: about 16 seconds from first eye degradation to void. The eyes outlast the body by several seconds. Even after the cloud has scattered to nothing, two dim dots persist, looking at you. Then they go.

---

## Rendering in ratatui

The spectre renders through a 6-pass compositing pipeline inside ratatui's `Frame::render_widget` call, running at 60fps.

### Pass order

**Pass 1: Scanlines (layer -2).** Faint horizontal lines across the entire sprite area. Dimmed by `phase_dim`. These create the CRT monitor effect -- the spectre is visible through a screen, not floating in open space.

**Pass 2: Inference glow.** When the golem is running an LLM call (`infer_glow > 0.01`), a radial background color shift emanates from the sprite center. Intensity scales with inference tier: T0 produces nothing, T3 produces a visible warm halo. The glow renders as background color on cells, not foreground, so it sits behind the dots.

**Pass 3: Somatic ripple ring.** When a somatic marker fires (`somatic_ripple > 0.01`), a ring of displaced dots expands outward from center over 300ms. Implemented by adding outward velocity impulse to dots in sequence from center to edge.

**Pass 4: Dots.** The cloud body. Only `floor(dot_count * phase_density)` dots render. Each dot's character is tier-dependent (`*`, `*`, `*`). Color blends between phase-dimmed base and emotion-bright, modulated by heartbeat phase and FSM pulse. Dots that fall outside the bounding box are skipped.

**Pass 5: Eyes.** Overwrite the buffer cells at eye positions. Eye glyphs, colors, bloom, scanning arrows, and the surprise mouth all render here. The bloom effect sets background color on adjacent cells to simulate phosphor bleed (ratatui does not support true alpha).

**Pass 6: Heartbeat.** The sine-wave character below the cloud. Brightness oscillates per the heartbeat phase calculation. Skipped during dreaming (`dream_alpha > 0.5`).

### Widget implementation

```rust
impl Widget for SpectreWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = &self.interpolated;
        let frame = self.frame_counter;

        // Pass 1: scanlines
        render_scanlines(area, buf, state.phase_dim);

        // Pass 2: inference glow
        if state.infer_glow > 0.01 {
            render_inference_glow(area, buf, state);
        }

        // Pass 3: somatic ripple
        if state.somatic_ripple > 0.01 {
            render_ripple(area, buf, state);
        }

        // Pass 4: dots
        let visible = (self.dots.len() as f32 * state.phase_density) as usize;
        for dot in &self.dots[..visible] {
            render_dot(dot, area, buf, state, frame);
        }

        // Pass 5: eyes
        render_eyes(buf, &self.eyes, state, frame);

        // Pass 6: heartbeat
        if state.dream_alpha < 0.5 {
            render_heartbeat(area, buf, state, frame);
        }
    }
}
```

---

## Bounding box sizes

Three size variants for different contexts:

| Variant | Dimensions | Use | Notes |
|---------|-----------|-----|-------|
| Standard | 22W x 12H cells | Creature pane (hearth screen) | Full cloud, all tiers, heartbeat row |
| Compact | 14W x 8H cells | Collapsed pane, crypt thumbnail | Fringe tier eliminated, no heartbeat |
| Mini | 8W x 4H cells | Clade miniatures, world screen | Eyes + nearest dots only, no shimmer |

At compact size, eliminate all Fringe (Tier 1) dots. Eyes stay the same glyph size. Heartbeat row removed. Spring physics continues at full rate; only rendering is simplified.

Mini (8x4): eyes + 6-8 nearest Dense dots only, no shimmer, no heartbeat. Dead golems in mini: single `†` (U+2020 dagger) at center.

On the hearth screen at interaction depth 2+, the standard sprite renders with expanded detail: PAD values visible below, Phi gauge, and the inscription (aleph-mem-tav / mem-tav) beneath the heartbeat.

---

## Mood-driven ambient effects

The Daimon's mood (the exponential moving average of recent emotions) creates persistent baseline shifts in the cloud's posture. Unlike emotions that change per tick, mood changes over hours.

| Mood octant | PAD signs | Ambient effect |
|-------------|-----------|----------------|
| Exuberant | +P +A +D | Cloud slightly tighter than baseline, shimmer lively |
| Dependent | +P +A -D | Dots occasionally drift toward top (looking up) |
| Relaxed | +P -A +D | Cloud loose, spacious, shimmer slow and gentle |
| Docile | +P -A -D | Cloud compact, minimal shimmer, quiet |
| Hostile | -P +A +D | Cloud agitated at baseline, sharp shimmer between events |
| Anxious | -P +A -D | Constant low-amplitude trembling |
| Disdainful | -P -A +D | Cloud pulled slightly to one side (asymmetric, averted) |
| Bored | -P -A -D | Cloud sinks 0.5 rows, shimmer near zero |

These are subtle. A viewer who checks back after 30 minutes might notice the cloud seems more agitated or more peaceful. They would not see a discrete transition happen. That's the point. Mood is climate, not weather.

---

## Interpolation architecture

### Double buffer

The sprite engine maintains two state buffers:

- **Target state**: set instantly when events arrive from the Event Fabric WebSocket.
- **Interpolated state**: what the renderer actually reads. Lerped toward target each frame at per-field rates.

The interpolated state struct holds all 32 channels as `f32` fields, plus derived values (cloud_expand, sink_offset, heartbeat_phase, cloud_behavior). The `tick()` method runs once per frame and advances every field toward its target.

```rust
pub struct InterpolatedState {
    // Fast channels (emotion, ~0.7s)
    pub pleasure: f32,            // ch 1
    pub arousal: f32,             // ch 2
    pub dominance: f32,           // ch 3
    pub emotion: PlutchikEmotion, // ch 4 (snap)
    pub emotion_intensity: f32,   // ch 5
    // Medium channels
    pub fsm_phase: f32,           // ch 6
    pub infer_glow: f32,          // ch 7
    pub probe_severity: f32,      // ch 8
    pub tick_cost: f32,           // ch 9
    // Slow channels
    pub mood_p: f32,              // ch 10
    pub mood_a: f32,              // ch 11
    pub mood_d: f32,              // ch 12
    pub phase_density: f32,       // ch 13
    pub phase_dim: f32,           // ch 14
    pub composite_vitality: f32,  // ch 15
    pub regime_volatility: f32,   // ch 16
    pub context_util: f32,        // ch 17
    pub dream_alpha: f32,         // ch 18
    pub dream_sub_phase: f32,     // ch 19
    pub phi: f32,                 // ch 20
    pub grimoire_density: f32,    // ch 21
    // Glacial channels
    pub economic_clock: f32,      // ch 22
    pub epistemic_clock: f32,     // ch 23
    pub age_factor: f32,          // ch 24
    pub credit_norm: f32,         // ch 25
    pub burn_rate: f32,           // ch 26
    // Oracle channels
    pub prediction_accuracy: f32, // ch 27
    pub accuracy_trend: f32,      // ch 28
    pub attention_breadth: f32,   // ch 29
    pub surprise_rate: f32,       // ch 30
    pub foraging_activity: f32,   // ch 31
    pub compounding_momentum: f32,// ch 32
    // Derived (not lerped)
    pub cloud_expand: f32,
    pub sink_offset: f32,
    pub cloud_behavior: CloudBehavior,
    pub heartbeat_phase: f32,     // free-running sine
    pub center_x: f32,
    pub center_y: f32,
    // Pulse channels (fire and decay)
    pub somatic_ripple: f32,
    pub curator_shimmer: f32,
    pub oracle_coherence_pulse: f32,
}
```

Cloud behavior is the one exception: it switches instantly, no interpolation. Behavior modes (Cohere, Agitate, Tremble, Sink, Still, Expand, Drift, Standard) do not blend. The spring physics parameters snap to new values when the mode changes; the physical simulation handles the visual transition smoothly because the dots have momentum and take several frames to respond.

### Event-to-target translation

When a `WireEvent` arrives, target values update:

- `DaimonEvent`: sets PAD targets, emotion, intensity, mood, cloud behavior, expansion factor, sink offset. Somatic marker fires set the ripple to 1.0 (pulse, not target).
- `VitalityEvent`: sets phase density, dim factor, vitality, economic/epistemic clocks, age factor.
- `HeartbeatEvent`: sets FSM pulse, inference glow, probe severity.
- `DreamEvent`: sets dream alpha (0.0 awake, 1.0 dreaming).
- `ContextEvent`: sets context utilization.
- `OracleEvent`: sets prediction_accuracy, accuracy_trend, attention_breadth, surprise_rate, foraging_activity, compounding_momentum. Arrives after first theta tick; all six channels hold zero until then.

The renderer never waits on network. Events update shared state asynchronously; the render loop reads whatever's there and interpolates.

---

## Performance

The shimmer system processes 60-80 dots per frame. At 60fps, that's 4,800 dot updates per second. This is negligible on any modern machine.

The most expensive single operation is the fear-intense startle-scatter, which temporarily repositions all dots. It lasts 300ms (18 frames) and happens rarely. The somatic ripple ring is similarly brief.

The interpolation tick processes ~36 lerp operations per frame. Also negligible.

The spectre's rendering cost is dominated by ratatui's buffer writes, not by the physics simulation.

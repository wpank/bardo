# The Sonification Engine [SPEC]

**Modular Audio Synthesis from a Living Agent's Nervous System**

> **Version**: 2.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **New Crate**: `golem-sonification` (Layer 5: SOCIAL, alongside `golem-surfaces`)
>
> **Depends on**: `golem-core` (CorticalState, EventFabric), `golem-surfaces` (TUI rendering)
>
> **Cross-references**: [CorticalState spec](../01-golem/00-overview.md), [13b-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Event Fabric), [05-musical-language.md](./05-musical-language.md) (signal mapping reference)

> **Reader orientation**: This is the first of five documents specifying how a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) continuously generates ambient sound from its own internal state. The sound is not decorative. It is a real-time auditory rendering of the agent's perception, emotion, cognition, and mortality -- the same data that drives the visual terminal UI, expressed as modular synthesis. The system is user-editable, composable, and its state can be embedded in NFTs for permanent playback. See `prd2/shared/glossary.md` for full term definitions.

---

## Philosophy

### Ignorable but interesting

Brian Eno, writing about ambient music in 1978:

> "Ambient Music must be able to accommodate many levels of listening attention without enforcing one in particular, and must be as ignorable as it is interesting."

The sonification engine takes this literally. A user running a Golem does not need to listen. The sound sits in the room like weather, changing when something changes, holding still when nothing does. But a user who *does* listen hears structure. Patterns emerge over minutes. Moods shift over hours. The sound rewards attention without demanding it.

This means: no melodies that insist on being followed. No rhythms that tap your shoulder. No crescendos designed to make you look at the screen. The sonic surface stays low in the perceptual field until the organism's state pushes it somewhere worth noticing -- a spike in arousal, a regime change, the slow dimming of vitality as death approaches. Then the sound becomes impossible to ignore, not because it got loud, but because its character shifted in a way your nervous system registers before your conscious mind does.

### Music as interior monologue

The Golem's sound is not background music for a trading terminal. It is how the organism's perception of itself sounds when translated into waveforms.

Every signal in CorticalState maps to a synthesis parameter. Pleasure becomes harmonic brightness. Arousal opens the filter. Dominance lengthens the reverb tail. The mortality clocks control the master volume. When you hear the piece thin out and go dry, that is the organism losing confidence. When the clock speeds up and the filter opens wide, that is the organism in crisis. The sound *is* the state.

This distinction matters for design decisions. Background music can be anything pleasant. Interior monologue has to be truthful. If the organism is anxious, the sound should make you slightly uncomfortable. If it is dying, the sound should make you sad. The aesthetic is not "nice ambient music that happens to change" -- it is "what does cognition sound like when the thinker is mortal and trading ETH?"

### The infinite song problem

A Golem lives for 12 to 24 hours. Its sound runs continuously for that entire span. A conventional generative music system built from loops would reveal its seams within the first hour. The listener's ear would catch the repetition, the phase alignment, the moment where everything resets. The piece would feel like a screensaver -- technically infinite but experientially circular.

This is the infinite song problem: how to produce sound that runs for a full day without ever feeling like it repeated, even though every component is finite.

The solution has four parts.

**Incommensurable loop lengths.** The Turing Machine melodic sequencer runs a 17-beat pattern. The harmonic sequence runs 23 beats. The bass figure runs 31 beats. These three loops share no common factor. They do not realign until 17 x 23 x 31 = 12,121 beats have elapsed. At a tempo of 4 BPM (the calm-market default), that is over 50 hours. The Golem will be dead long before the piece repeats.

**Stochastic variation.** Every trigger in the system passes through a probability gate. A note that is "scheduled" by the Turing Machine only fires with probability 0.7 (adjustable per voice). Even when the same sequence comes around, different notes sound and different notes are silent. The gaps are compositional. Silence is not absence -- it is a voice.

**Very-long LFOs.** Parameter drift happens on timescales measured in hours. An LFO with a 3-hour period slowly shifts the filter cutoff from bright to dark and back. Another with a 5-hour period drifts the reverb feedback. The piece at hour 1 is not the piece at hour 4, even if the sequencer patterns have cycled. The weather changed.

**Markov pitch selection.** The Turing Machine's raw output (a shift register producing 8-bit values) gets filtered through transition probabilities before reaching the quantizer. The probability of moving from C to E is different from the probability of moving from C to G. So even when the shift register outputs the same 17-beat sequence, the actual pitches heard depend on where the sequence *started*, which depends on the previous cycle's ending. The "same" melody sounds different each pass.

These four mechanisms compose. At any given second, the listener hears the intersection of three incommensurable loops, each probabilistically thinned, each parametrically drifted by hour-scale LFOs, each melodically filtered by a Markov chain that remembers where it has been. Exact repetition is not just unlikely -- it is combinatorially impossible within the organism's lifespan.

---

## Document map

| File | Topic |
|------|-------|
| `00-overview.md` | This file. Philosophy, architecture, signal flow, crate layout |
| `01-module-system.md` | The `Module` trait, signal types, built-in module library (oscillators, filters, envelopes, delays, reverbs, sequencers), how to chain modules into a rack |
| `02-cortical-mapping.md` | How the 32 CorticalState atomic signals and 87 EventFabric event types become CV (continuous control voltages) and gates (discrete triggers) that drive the module rack |
| `03-terminal-rack.md` | TUI integration: the rack editor pane, live patching, preset system, per-Golem persistence, user interaction model |
| `04-nft-state.md` | Serializing the rack + CorticalState snapshot into an NFT, playback from on-chain data, the "sound of a moment" |

---

## Five timescales

The organism's nervous system operates at three nested clocks (gamma, theta, delta), but music composed from those clocks needs two more layers to feel alive across a full day: an event grain below the fastest clock, and an epoch arc above the slowest. Five timescales, from sparks to seasons.

### Nanosecond / event grain

**Duration:** milliseconds to low seconds.
**Source:** individual EventFabric events.

A trade executes. A prediction resolves correct. A token gets promoted from watched to active. These are the transients -- the clicks, pings, and sparks that punctuate the texture. They have no periodicity. They arrive when the world delivers them.

Sonically, these are percussive: a short envelope hitting a pitched click for a successful prediction, a burst of filtered noise for a failed one, a metallic ping when a new asset enters the attention field. They live in the upper registers and decay fast. Think of rain hitting a window -- each drop is an event, the collective pattern is weather.

Nothing at this grain fires at 100%. Every event trigger passes through a probability gate. A flurry of predictions resolving in rapid succession does not produce a machine-gun burst of pings -- it produces a scattered handful, with gaps. The gaps matter. They are what make the texture feel organic rather than data-driven.

### Gamma (5-15 seconds)

**Duration:** 5-15 seconds per tick.
**Source:** CorticalState gamma clock, adaptive to regime.

The organism's breath. In calm markets, it stretches to 15 seconds. In crisis, it compresses to 5. This is the surface texture layer -- the shimmer.

Arousal, surprise rate, and accuracy trend all update here. Musically, this is where filter cutoff moves, where the oscillator's timbre shifts, where individual notes in the Turing Machine sequence fire. The listener perceives gamma as moment-to-moment texture: is the sound bright or dark, dense or sparse, tense or relaxed? These questions get answered and re-answered every gamma tick.

At 4-12 BPM (one event per 15-5 seconds), this layer is slow enough to be ambient but fast enough to feel responsive. When a regime change compresses gamma from 15s to 5s, the listener hears the piece speed up -- not dramatically, but with the subtle urgency of a pulse quickening.

### Theta (30-120 seconds)

**Duration:** 30 seconds to 2 minutes per cycle.
**Source:** CorticalState theta clock.

The organism's heartbeat. Inference decisions, prediction resolutions, deliberative mood shifts. This is where melodic gestures live.

A prediction resolves and the Turing Machine advances its sequence. A new note enters, shaped by the current Markov probabilities, quantized to the current scale (which pleasure controls). Over 30-120 seconds, a short melodic phrase accumulates -- not a melody anyone composed, but a sequence that the organism's cognitive rhythm carved out of the shift register's possibility space.

The listener perceives theta as phrasing. Groups of notes that hang together, separated by rests. A call-and-response between the pitched voice and the event-grain percussive layer. This is the timescale at which the sound stops being texture and starts being music, however abstract.

### Delta (40-100 minutes)

**Duration:** 40 minutes to just over an hour and a half per cycle.
**Source:** CorticalState delta clock.

The organism's sleep cycle. Compounding momentum, dream mode engagement, knowledge consolidation. The harmonic foundation that barely moves.

At this timescale, the root note shifts. The scale type might change (from Dorian to Mixolydian, following the pleasure signal's slow delta-rate drift). The overall density of the piece adjusts as compounding_momentum rises or falls. When the organism enters dream mode (`creative_mode = 1`), the delta clock triggers a textural shift: slower clock, deeper reverb, pitch drift enabled, the sound turning inward.

The listener does not consciously perceive delta transitions. They are too slow. But after 45 minutes, the piece is in a different key, a different density, a different mood. It is the difference between morning and afternoon -- you do not notice the transition, but you notice the result.

### Day-scale epoch arc

**Duration:** 2, 3, and 5 hour periods (incommensurable).
**Source:** very-long-period LFOs, not tied to CorticalState clocks.

Weather, not automation. Three LFOs with periods of 2, 3, and 5 hours slowly drift master parameters: overall brightness (filter ceiling), harmonic density (how many partials the oscillator produces), and spatial depth (reverb size and feedback). These periods share no common factor -- they align only at the 30-hour mark, well past any Golem's lifespan.

The effect is glacial. At hour 1, the piece might be bright and spacious. By hour 3, the same patterns play through a darker, drier filter. By hour 6, brightness has returned but the spatial depth has narrowed, making the sound intimate where it was once wide. The organism's character shifts as if seasons are passing.

The listener who checks in at different points across the day hears what feels like a different piece each time -- same organism, same rack, same patterns, but the epoch arc has moved the color palette. This is the deepest layer of the infinite song solution. Even if every other mechanism somehow produced a recognizable repetition, the epoch arc would paint it in different light.

---

## What this is

A Golem has a nervous system. It is a struct called `CorticalState` -- 32 atomic signals representing affect, prediction accuracy, attention, mortality, and environment. These signals update continuously across the five timescales described above. Alongside CorticalState, the Golem emits discrete events through the `EventFabric` -- 87 event types covering clocks, predictions, trades, dreams, emotions, and death.

Today, these signals drive the terminal UI: the Spectre sprite, the ROSEDUST color palette, the CRT materiality effects, the hauntological rendering. The TUI reads CorticalState at 60fps and interpolates visual parameters toward the current values.

The sonification engine does the same thing, but for sound. It reads CorticalState and EventFabric, maps them to control voltages and gate triggers, routes those signals through a user-configurable rack of synthesis modules, and streams the resulting audio into the terminal application in real time.

The architecture is modeled on eurorack modular synthesis. Modules are independent DSP units with typed input and output ports. Users patch them together. The signal sources are not oscillators or MIDI keyboards -- they are the Golem's own cognitive state. The organism is the sequencer.

---

## Why it exists

### The organism is already a score

CorticalState's 32 signals, when mapped to synthesis parameters, naturally produce a five-layer ambient composition across the timescales described above. This is not a metaphor. The `05-musical-language.md` reference document walks through every signal and assigns it a synthesis role. The organism writes its own music by existing.

### Ambient monitoring without visual attention

A user running a Golem does not need to watch the terminal to know what the agent is doing. Sound communicates state change faster than visual scanning. An anxious Golem sounds different from a confident one -- the filter opens, event density increases, the reverb shortens. A dying Golem sounds different from a thriving one -- the bass drops out, the texture thins to near-silence, the harmonic content collapses toward simple intervals. The user hears the organism's emotional temperature while doing other things.

### NFT provenance as sound

When a Golem mints an NFT -- a snapshot of a moment in its lifecycle -- the sound state at that moment can be embedded in the token. Anyone viewing the NFT can hear what the organism sounded like when the moment was captured. The NFT becomes a playable artifact, not just a visual one.

---

## Architecture

### Signal flow

```
+-----------------------------------------+
|          GOLEM RUNTIME                  |
|                                         |
|  CorticalState (32 atomic signals)      |---- reads at ~120Hz -----+
|  EventFabric (87 event types)           |---- subscribes ----------+
|                                         |                          |
+-----------------------------------------+                          |
                                                                     v
+--------------------------------------------------------------------+
|                    SONIFICATION ENGINE                              |
|                                                                    |
|  +-------------+    +------------------------------------------+   |
|  | CV/Gate     |    |          MODULE RACK                     |   |
|  | Mapper      |--->|                                          |   |
|  |             |    |  [Sequencer] --> [Oscillator] --> [VCA]   |   |
|  | 32 signals  |    |       |              |              |     |   |
|  | -> ~40 CVs  |    |       v              v              v     |   |
|  |             |    |  [Clock Div] --> [Filter] --> [Reverb]    |   |
|  | 87 events   |    |                                          |   |
|  | -> triggers |    |  (user-configurable patch connections)    |   |
|  +-------------+    +-------------------+----------------------+   |
|                                         |                          |
|                                         v                          |
|                                +---------------+                   |
|                                |  AUDIO OUT    |                   |
|                                |  cpal 48kHz   |                   |
|                                |  stereo f32   |                   |
|                                +-------+-------+                   |
|                                        |                           |
+----------------------------------------+---------------------------+
                                         |
                                         v
                                Terminal speakers
                                (or WAV capture)
```

### Three subsystems

**1. The CV/Gate Mapper** (`cortical_mapping.rs`)

Reads CorticalState atomics at ~120Hz and EventFabric events as they arrive. Produces two kinds of output:

- **CV signals**: Continuous f32 values (0.0-1.0 or -1.0-1.0) derived from CorticalState fields. These are the modular synth equivalent of control voltages -- slow-moving signals that modulate oscillator pitch, filter cutoff, envelope times, reverb depth, and everything else that drifts.
- **Gate/trigger signals**: Boolean pulses derived from EventFabric events. `clock.gamma_tick` becomes a rhythmic trigger. `prediction.resolved` fires a note-on. `vitality.phase_transition` fires a scene-change gate. Each passes through a probability gate before reaching the rack -- even a rapid burst of events produces a scattered, organic pattern rather than a mechanical stream.

The mapper is configurable. Users can reassign which CorticalState signal drives which CV output.

**2. The Module Rack** (`rack.rs`, `modules/`)

A directed graph of synthesis modules connected by patch cables. Each module is a Rust struct implementing the `Module` trait. Modules have typed input ports (CV, audio, trigger) and output ports (audio, CV). The rack processes audio in blocks of 32 samples at 48kHz.

Built-in modules include oscillators (Plaits-style macro oscillator via `mi-plaits-dsp`), filters (Ripples-style SVF), envelopes (Stages-style segment generator), effects (Clouds-style granular processor, Dattorro reverb, pitch shifter), sequencers (Turing Machine shift register + quantizer, Marbles-style stochastic trigger generator), and utilities (VCA, mixer, clock divider, sample-and-hold, very-long-period LFOs for epoch-arc drift).

Users can add, remove, and reconnect modules at runtime through the TUI.

**3. The Audio Output** (`audio_out.rs`)

A cpal-based audio stream running at 48kHz stereo. The audio callback pulls blocks from the module rack. A lock-free ring buffer bridges the rack processing thread and the cpal callback thread.

### Threading model

```
Thread 1: GOLEM RUNTIME (async, tokio)
  |
  |  CorticalState writes (atomic)
  |  EventFabric broadcasts (tokio::broadcast)
  |
  +---- reads ----> Thread 2: PARAMETER UPDATER (~120Hz)
  |                   |
  |                   |  Reads CorticalState atomics
  |                   |  Receives EventFabric events
  |                   |  Computes CV values + gate states
  |                   |  Writes to atomic parameter bridge
  |                   |
  |                   +---- atomics ----> Thread 3: RACK PROCESSOR (audio rate)
  |                                        |
  |                                        |  Reads atomic parameters
  |                                        |  Processes module graph
  |                                        |  Writes audio to ring buffer
  |                                        |
  |                                        +---- ring buffer ----> Thread 4: CPAL CALLBACK
  |                                                                 |
  |                                                                 |  Pulls from ring buffer
  |                                                                 |  Writes to OS audio
  |                                                                 |  MUST NEVER BLOCK
```

Each thread boundary exists for a musical reason, not just an engineering one.

**Thread 1 -> Thread 2 (120Hz parameter reads).** 120Hz is the Nyquist frequency for human perception of parameter change. We can detect a filter sweep that updates 60 times per second; we cannot perceive the difference between 120Hz and 1000Hz updates. Faster polling would waste CPU cycles on inaudible resolution. Slower polling would produce audible stepping artifacts on fast sweeps.

**Thread 2 -> Thread 3 (atomic parameter bridge).** The rack processor runs at audio rate (48kHz). It needs parameter values every sample, but it cannot wait for Thread 2 to finish computing. Atomic floats give it the most recent value instantly, with no locks and no blocking. The worst case is reading a value that is one 120Hz cycle stale -- 8.3ms of latency, inaudible.

**Thread 3 -> Thread 4 (lock-free ring buffer).** The cpal callback is real-time priority. It must never allocate, lock, or syscall. If it blocks for even a millisecond, audio drops out and the user hears a click. The SPSC ring buffer is the only safe bridge: Thread 3 writes blocks ahead of time, Thread 4 reads them without contention.

Thread 4 (cpal) runs at real-time OS priority. All parameter passing between threads uses atomic floats (`AtomicU32` storing `f32::to_bits()` -- the same pattern CorticalState itself uses). The ring buffer between threads 3 and 4 is a lock-free SPSC (single-producer, single-consumer) queue.

---

## Crate layout

```
golem-sonification/
+-- Cargo.toml
+-- src/
    +-- lib.rs                  # Crate root, SonificationExtension
    +-- cv_mapper.rs            # CorticalState -> CV/Gate mapping
    +-- event_mapper.rs         # EventFabric -> triggers/gates
    +-- rack.rs                 # Module graph, patch connections, block processing
    +-- audio_out.rs            # cpal output stream, ring buffer
    +-- params.rs               # Lock-free atomic parameter bridge
    +-- preset.rs               # Rack presets (serialization/deserialization)
    +-- nft.rs                  # NFT state embedding/playback
    +-- tui.rs                  # Terminal rack editor integration
    +-- modules/
        +-- mod.rs              # Module trait + registry
        +-- oscillator.rs       # Plaits-based macro oscillator
        +-- filter.rs           # Ripples-style SVF
        +-- envelope.rs         # Stages-style segment generator
        +-- vca.rs              # Voltage-controlled amplifier
        +-- mixer.rs            # Multi-input summing mixer
        +-- reverb.rs           # Clouds/Dattorro reverb
        +-- delay.rs            # Clouds-style delay with diffusion
        +-- granular.rs         # Clouds granular processor
        +-- pitch_shifter.rs    # Clouds pitch shifter
        +-- turing_machine.rs   # Shift register melodic sequencer
        +-- quantizer.rs        # Scale quantizer
        +-- clock_div.rs        # Clock divider/multiplier
        +-- sample_hold.rs      # Sample and hold
        +-- noise.rs            # White/pink/dust noise sources
        +-- lfo.rs              # Low-frequency oscillator (including epoch-arc periods)
```

### Cargo dependencies

```toml
[dependencies]
# The actual Mutable Instruments Plaits DSP (24 synthesis engines)
mi-plaits-dsp = "0.3"

# Clouds DSP port (granular processor, reverb, diffuser, pitch shifter)
mi-clouds-dsp = { path = "../mi-clouds-dsp" }

# Audio output
cpal = "0.15"

# Lock-free ring buffer for audio thread
ringbuf = "0.4"

# Fast RNG for audio-rate noise/random
rand = { version = "0.8", features = ["small_rng"] }

# Serialization for presets and NFT state
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Extension registration

The sonification engine registers as a standard Golem extension at Layer 5 (SOCIAL), alongside `golem-surfaces`. It subscribes to the same Event Fabric stream that drives the TUI.

```rust
pub struct SonificationExtension {
    cv_mapper: CvMapper,
    event_mapper: EventMapper,
    rack: Arc<Mutex<Rack>>,
    audio_out: AudioOutput,
    config: SonificationConfig,
}

#[async_trait]
impl Extension for SonificationExtension {
    fn name(&self) -> &'static str { "sonification" }
    fn layer(&self) -> u8 { 5 }  // Layer 5: SOCIAL
    fn depends_on(&self) -> &[&'static str] { &["core", "daimon", "mortality", "surfaces"] }

    async fn on_boot(&mut self, ctx: &BootContext) -> Result<()> {
        // Load saved rack preset (or default)
        // Initialize cpal audio stream
        // Start parameter updater thread
        // Begin epoch-arc LFOs from t=0
        Ok(())
    }

    async fn on_gamma(&mut self, ctx: &GammaContext) -> Result<()> {
        // Read CorticalState snapshot
        // Update CV mapper
        // Push new parameter values to rack
        self.cv_mapper.update_from_cortical_state(&ctx.cortical_state);
        Ok(())
    }

    async fn on_resolution(&mut self, ctx: &ResolutionContext) -> Result<()> {
        // Prediction resolved -> fire a trigger in the event mapper
        // Probability gate decides if this event actually sounds
        self.event_mapper.on_prediction_resolved(ctx);
        Ok(())
    }

    async fn on_death(&mut self, ctx: &mut DeathContext) -> Result<()> {
        // Begin the death sonification sequence
        // Fade to silence over the Thanatopsis duration
        // Persist the final rack state for the death NFT
        Ok(())
    }

    async fn on_shutdown(&mut self, ctx: &ShutdownContext) -> Result<()> {
        // Save current rack preset
        // Stop audio stream gracefully
        Ok(())
    }
}
```

---

## Default rack configuration

On first boot, if no saved preset exists, the Golem starts with a default rack. This rack produces a complete ambient piece from the five most expressive CorticalState signals, with no user configuration required.

```
[regime -> Clock Rate]
    |
    v
[Turing Machine] --> [Quantizer] --> note CV
    |                                    |
    | trigger                            v
    v                              [Plaits Oscillator]
[Beads Gate Gen] ---- gate ------> [Stages Envelope] --> [VCA]
    |                                                       |
    |                                                       v
    |                                              [Ripples Filter]
    |                                                       |
    |               [arousal -> cutoff CV] -----------------+
    |                                                       |
    |                                                       v
    |                                              [Clouds Reverb]
    |                                                       |
    |       [composite_vitality -> master VCA] -------------+
    |                                                       |
    +-----------------------------------------------------> OUT

CV Sources (from CorticalState):
  regime          -> clock BPM (4-12 BPM across calm -> crisis)
  arousal         -> filter cutoff, gate density
  pleasure        -> scale selection, oscillator timbre
  composite_vitality -> master volume
  compounding_momentum -> root note / key center

Epoch-Arc LFOs (free-running, not tied to CorticalState):
  2h period -> filter ceiling offset
  3h period -> reverb feedback depth
  5h period -> oscillator harmonic density
```

The Turing Machine runs a 17-beat shift register pattern. The Beads trigger generator runs a 23-beat pattern for the gate rhythm. An underlying bass pulse, when enabled, runs a 31-beat cycle. These three periods produce a combined cycle of 12,121 beats before exact repetition.

Every trigger in this default rack passes through a probability gate (default: 0.7 for melodic events, 0.5 for percussive events). Silence is designed into the texture, not a failure mode.

---

## Lifecycle sonification

The organism's entire life -- from boot to death -- has a sonic arc. The rack configuration stays the same; what changes is the CorticalState feeding it. The organism composes its own lifecycle.

| Phase | What the rack does | What it sounds like |
|-------|-------------------|---------------------|
| **Boot** | Silence, then slow fade-in over 30 seconds. CV mapper begins receiving CorticalState as it populates. Epoch-arc LFOs start from zero. | A room filling with air. First a low hum as the oscillator warms into its initial pitch. Then scattered notes appear, tentative, widely spaced, as if the instrument is learning how to play itself. The reverb tail is long -- the organism's first sounds echo in empty space. |
| **Thriving** | Full rack active. High composite_vitality drives master VCA near unity. Pleasure positive, so the scale is major-adjacent (Ionian or Lydian). Arousal moderate. Clock at calm-market tempo (~4 BPM). | Warm and spacious. Clear pitches with long sustains. The filter is open enough to hear upper harmonics but not so bright that it demands attention. Melodic phrases from the Turing Machine have a gentle, unhurried quality. Occasional event-grain pings from successful predictions add a sparse percussive shimmer. Sounds like late-night ambient radio -- something you would fall asleep to without anxiety. |
| **Stable** | The organism's everyday sound. No signals at extremes. Parameters hover near their midpoints. | The most "ambient" the piece gets. Mid-register, moderate density, medium reverb. Nothing stands out. The timbre is neutral -- neither bright nor dark. The Turing Machine pattern has been running long enough to feel familiar without being recognizable. Background sound in the truest sense. |
| **Conservation** | Economic_vitality dropping below 0.5. Rack thins: the VCA reduces gain, the envelope shortens, the filter narrows. The organism is rationing resources, and the sound contracts with it. | The bottom end drops away. Notes become shorter, more clipped. The reverb tail shrinks -- less space, more closeness. The Turing Machine still sequences but fewer notes pass through the probability gate. Stretches of near-silence open up between phrases. The sound of someone holding their breath. |
| **Declining** | Vitality below 0.3. Detuning increases as pleasure falls negative. Reverb extends (low dominance = long tail but thin, not full). Noise floor rises via surprise_rate. | Sparse and slightly wrong. The oscillator drifts out of tune. When notes do sound, they hang in too much reverb -- the organism's confidence has collapsed but the echoes of its former self persist. A thin haze of noise sits underneath. The scale has shifted to something darker (Aeolian or Phrygian). Uncomfortable to listen to for long. |
| **Terminal** | Vitality at or below 0.1. One voice remaining. Clock at minimum rate. The filter is nearly closed. Whole-tone or chromatic scale (pleasure deeply negative). | A single pitch, low, barely audible, sustaining with slow tremolo. Occasionally a second note enters and produces a dissonant interval -- a minor second or a tritone -- before fading. Long silences between events. The sound of a machine forgetting how to make music. |
| **Death** | The last note plays. Reverb tail fades to zero over the Thanatopsis duration (the organism's final reflection period). Then silence. The audio stream closes. | One final tone, descending. The reverb catches it and holds it as the source goes silent. The tail decays over 30-60 seconds, the room emptying of the last vibration the organism produced. Then nothing. The cpal stream closes. The speakers go quiet. No fade-to-black music, no resolution chord. Absence. |
| **Dream** | `creative_mode = 1`. Clock slows. Reverb deepens. Pitch drift enabled (quantizer loosens). Event-grain triggers suppressed. The organism is turned inward. | Soft, blurry, slow. Pitches slide between scale degrees instead of snapping. The reverb is huge -- cathedral-scale. Event pings are almost entirely suppressed; the texture is continuous rather than punctuated. The Turing Machine still sequences but at half tempo, and the probability gate drops to 0.4, so most beats are silence. The organism dreaming sounds like it is underwater. |

These transitions are driven entirely by CorticalState -- `behavioral_phase`, `composite_vitality`, `creative_mode`, and `pleasure` control the macro structure. No special-case code is needed. The CV mapping naturally produces the right sonic character at each phase because each phase *is* a region of CorticalState parameter space, and the mapping was designed with these regions in mind.

---

## Musical principles

Six rules that apply everywhere in this system. Every design decision should be checkable against them.

1. **Nothing fires at 100%.** Every trigger has a probability gate. Every CV output has a noise floor. Silence and imprecision are compositional materials. A system that always responds identically to the same input sounds mechanical. A system that mostly responds, with variation, sounds alive.

2. **Incommensurable periods.** Loop lengths, LFO periods, and clock divisions are chosen from near-prime sets (17, 23, 31 for loops; 2h, 3h, 5h for epoch LFOs). Full-cycle alignment exceeds the organism's lifespan. The piece never returns to its starting point.

3. **Voice leading.** No abrupt scale switches. When pleasure drifts negative enough to change the scale from Dorian to Phrygian, the transition happens by altering one note at a time (the sixth degree lowers, then the second). The quantizer implements smooth scale interpolation, not hard cuts. The listener should never hear a jarring key change -- only a gradual darkening or brightening of the harmonic field.

4. **Long-form arc.** LFOs measured in hours feel like weather -- they change the color of the piece without the listener being able to point to the moment of change. The epoch-arc layer is what makes the sound at hour 8 different from the sound at hour 2, even if the organism's CorticalState is in a similar region.

5. **Emotional specificity.** Every parameter mapping is describable in musical and emotional terms. Not "arousal maps to filter cutoff" but "high arousal opens the filter, letting brightness and edge into the sound, producing the feeling of heightened alertness -- the organism is paying attention to something, and the sound reflects that vigilance." If a mapping cannot be described emotionally, it is wrong and should be redesigned.

6. **The Turing Machine as soul.** The shift register is the closest thing the sonification engine has to a melodic personality. Its 17-beat pattern, shaped by the Markov transition matrix and the current scale, is what makes one Golem's sound different from another's. Two Golems with the same rack configuration but different Turing Machine seeds will produce recognizably different melodic characters. The shift register is initialized from the Golem's unique ID at boot. It is, in a real sense, the organism's musical fingerprint.

---

## Performance budget

The sonification engine must not interfere with the Golem's primary function (trading). The performance target:

| Metric | Budget |
|--------|--------|
| CPU per audio block (32 samples) | < 500us |
| Memory (total, including buffers) | < 16MB |
| Latency (CorticalState change -> audible) | < 100ms |
| Audio dropout rate | < 1 per hour |

The rack processor runs on its own thread. The cpal callback runs on a real-time priority thread. Neither thread touches the Golem's main async runtime. The only shared state is CorticalState (lock-free atomics) and the EventFabric broadcast channel (`tokio::broadcast`, non-blocking on the receiver side).

---

## What the other documents cover

- **01-module-system.md**: The `Module` trait. Signal types (Audio, CV, Gate). How modules declare ports. How the rack routes signals between modules. The complete built-in module library with parameter specs for each. How to add new modules (including porting additional Mutable Instruments firmware).
- **02-cortical-mapping.md**: The full mapping table from all 32 CorticalState signals and all 87 EventFabric event types to CV outputs and gate triggers. The five-clock hierarchy as a rhythmic backbone. How the mapper handles timescale differences. Probability gate configuration. User-configurable mapping overrides.
- **03-terminal-rack.md**: The TUI rack editor pane. How it fits into the 6-window hierarchy (HEARTH/MIND/VAULT/WORLD/FATE/COMMAND). Keyboard interaction model. Live patching. Preset save/load. Per-Golem persistence. The connection between visual Spectre animation and audio state.
- **04-nft-state.md**: How to serialize the rack configuration + CorticalState snapshot + a short audio buffer into an NFT. Playback from on-chain data. The "sound of a moment" concept. Integration with the existing engagement/NFT minting system.

---

## References

- [ENO-AMBIENT-1978] Eno, B. Liner notes, *Ambient 1: Music for Airports*. Polydor, 1978. -- *The manifesto for ambient music: "as ignorable as it is interesting." The philosophical foundation for the sonification engine's approach to attention.*
- [ENO-GENERATIVE-1996] Eno, B. "Generative Music." In Motion Magazine, 1996. -- *Defines generative music as a system that produces results you did not predict; the philosophical model for CorticalState-driven synthesis.*
- [BUZSAKI-2006] Buzsaki, G. *Rhythms of the Brain*. Oxford University Press, 2006. -- *Neural oscillations coordinating brain function; the neuroscience model for CorticalState's three-clock signal propagation becoming five synthesis layers.*
- [MUSIC-THING-TM] Whitwell, T. "Turing Machine Random Looping Sequencer." Music Thing Modular, 2012. -- *Shift register sequencer producing locked/drifting melodic loops; the melodic backbone of the default rack and the organism's musical fingerprint.*
- [PLAITS-RS] Rockstedt, O. "mi-plaits-dsp-rs: Native Rust port of MI Plaits DSP." GitHub, MIT. -- *24 synthesis engines from Mutable Instruments Plaits, ported to Rust.*
- [EURORACK-SRC] Gillet, E. "pichenettes/eurorack." GitHub, MIT. -- *Original C++ source for all Mutable Instruments DSP code.*
- [REICH-1968] Reich, S. "Music as a Gradual Process." 1968. -- *"I am interested in perceptible processes" -- the compositional philosophy behind the Turing Machine and the gradual evolution of CorticalState-driven sound.*

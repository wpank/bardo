# The Module System [SPEC]

**Traits, Signal Types, Built-In Library, and Rack Routing**

> **Version**: 2.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-sonification` (modules/ directory)
>
> **Cross-references**: [00-overview.md](./00-overview.md), [02-cortical-mapping.md](./02-cortical-mapping.md)

> **Reader orientation**: This document specifies how individual synthesis modules are built, how they declare their inputs and outputs, and how they are wired together into a processing graph called a Rack. A module is a Rust struct that processes audio and control signals in blocks of 32 samples at 48kHz. Modules are the atoms of the sonification system. The Rack is the molecule -- a directed graph of modules connected by typed patch cables. Users build racks in the terminal UI; the system ships with a default rack that works out of the box. If you have used a modular synthesizer (hardware or software like VCV Rack), this will feel familiar. If not, think of it as a signal-processing pipeline where each stage is a small, self-contained DSP unit.

---

## Signal types

Every connection between modules carries one of three signal types. The type system prevents patching an audio output into a gate input -- the compiler catches it, not the user.

```rust
/// The three signal types in the modular system.
/// These map directly to eurorack conventions:
///   Audio  = AC-coupled audio signal (-1.0..1.0)
///   Cv     = Control voltage (typically 0.0..1.0 or -1.0..1.0)
///   Gate   = Boolean trigger/gate (0.0 or 1.0, with rising-edge detection)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Audio-rate signal. Processed at 48kHz sample rate.
    /// Range: -1.0 to 1.0 (soft-clipped at output).
    Audio,
    /// Control voltage. Updated at audio rate but typically changes
    /// slowly (from CorticalState at ~120Hz, smoothed).
    /// Range: depends on context. Pitch CV is in semitones.
    /// Most CVs are 0.0..1.0 or -1.0..1.0.
    Cv,
    /// Gate/trigger signal. 0.0 = low, 1.0 = high.
    /// A "trigger" is a gate that is high for exactly one block (32 samples).
    /// A "gate" can be high for an extended duration.
    Gate,
}
```

### Why three types

In eurorack hardware, all signals are voltages -- there is no type system. You can patch an audio output into a CV input and sometimes it sounds interesting. But in software, untyped connections lead to confusion: patching a +/-1.0 audio signal into a parameter expecting 0.0-1.0 produces garbage.

The three types enforce expectations while remaining flexible. Audio->Audio and CV->CV are direct connections. CV->Audio is allowed (a very slow oscillator is a CV). Gate->CV is allowed (a gate is a binary CV). Audio->CV is allowed with a warning (the user probably means to use an envelope follower). Gate->Audio is blocked (gates are not audible signals).

The compatibility matrix:

| From \ To | Audio In | CV In | Gate In |
|-----------|----------|-------|---------|
| Audio Out | yes | warn | no |
| CV Out | yes | yes | no |
| Gate Out | no | yes (as 0/1) | yes |

---

## The Module trait

Every synthesis module implements this trait. It is the single interface that the Rack uses to process audio.

```rust
/// A block of samples for one signal.
pub type SignalBlock = [f32; BLOCK_SIZE];

/// Block size: 32 samples at 48kHz = 0.667ms per block.
/// Matches Clouds' internal block size.
pub const BLOCK_SIZE: usize = 32;

/// Sample rate. Matches Plaits and Clouds internal rate.
pub const SAMPLE_RATE: f32 = 48000.0;

/// Port identifier: a module name + port name pair.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId {
    pub module_id: String,
    pub port_name: String,
}

/// Port declaration: what a module exposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDeclaration {
    pub name: String,
    pub signal_type: SignalType,
    pub direction: PortDirection,
    /// Human-readable description (shown in TUI).
    pub description: String,
    /// Default value when nothing is patched (CV/Gate only).
    pub default_value: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

/// The core module trait. Every synthesis module implements this.
pub trait Module: Send + 'static {
    /// Unique identifier for this module instance (e.g., "osc_1", "filter_2").
    fn id(&self) -> &str;

    /// Human-readable display name (e.g., "Plaits Oscillator").
    fn display_name(&self) -> &str;

    /// Declare all input and output ports.
    /// Called once at registration. The rack uses this to validate connections.
    fn ports(&self) -> Vec<PortDeclaration>;

    /// Process one block of audio.
    ///
    /// `inputs`: a map from input port name -> signal block.
    ///           Missing entries mean "nothing patched" -- use the port's default.
    /// `outputs`: a map from output port name -> signal block.
    ///            The module writes its output here.
    ///
    /// This is called at audio rate: 48000/32 = 1500 times per second.
    /// It MUST NOT allocate, lock, or syscall.
    fn process(
        &mut self,
        inputs: &HashMap<String, SignalBlock>,
        outputs: &mut HashMap<String, SignalBlock>,
    );

    /// Set a parameter by name (from TUI knob turn or CV mapping).
    /// Parameters are module-internal values not exposed as ports.
    /// e.g., "waveform" on an oscillator, "scale" on a quantizer.
    fn set_param(&mut self, name: &str, value: f32);

    /// Get a parameter value by name.
    fn get_param(&self, name: &str) -> Option<f32>;

    /// List all parameters with their current values, ranges, and descriptions.
    fn params(&self) -> Vec<ParamDeclaration>;

    /// Serialize the module's complete state for preset save / NFT embedding.
    fn serialize_state(&self) -> serde_json::Value;

    /// Restore state from a previous serialization.
    fn deserialize_state(&mut self, state: &serde_json::Value);

    /// Reset to initial state (called on rack reset).
    fn reset(&mut self);
}

/// Parameter declaration for TUI display and preset serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDeclaration {
    pub name: String,
    pub display_name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: String,          // e.g., "Hz", "ms", "semitones", ""
    pub description: String,
}
```

### Why `HashMap<String, SignalBlock>`

The obvious alternative is fixed-size arrays indexed by port number. That would be faster (no hashing), but it couples every module to a specific port layout. With string-keyed maps, modules can have arbitrary numbers of ports, ports can be added in future versions without breaking existing patches, and the TUI can display port names directly. The performance cost is ~50ns per lookup -- negligible at 1500 blocks/second.

For modules on the critical path (oscillator -> filter -> VCA -> output), we provide a fast-path: the rack pre-resolves port names to indices at patch time, and the inner loop uses direct array access. The HashMap interface is for flexibility; the inner loop is for speed.

---

## Built-in module library

### Oscillators

**PlaitsOscillator** -- Mutable Instruments Plaits macro oscillator via `mi-plaits-dsp`

All 24 synthesis engines from Emilie Gillet's firmware, ported to Rust. This is the primary voice. Each engine is a different instrument -- not a different preset on the same instrument, but a fundamentally different way of making sound.

Engine 0 (virtual analog) is the default voice. Two oscillators drift slowly toward each other, producing a warm, slightly out-of-tune tone that thickens as `timbre` increases. It sounds like a synth pad left running overnight -- familiar, a little drowsy, good for sustained harmonic beds. Engine 11 (particle noise) is the opposite: granular static decaying through a bank of resonators, like sand falling through a metal grate. Turn `morph` up and the particles stretch into tonal ghosts. Engine 23 (speech synthesis) is the weird one. Formant-shaped noise that sounds like the Golem trying to form words. With the right pitch CV it actually tracks vowel shapes. The Golem doesn't speak, but sometimes it sounds like it wants to.

Between those three extremes there are 21 other engines -- FM, waveshaping, additive, wavetable, string, modal resonator, percussion. Each engine selection changes the instrument's personality, not its settings.

| Port | Type | Description |
|------|------|-------------|
| `pitch` | CV In | V/Oct pitch (MIDI note as f32). 60.0 = middle C. |
| `timbre` | CV In | Timbre control (0.0-1.0). Meaning varies per engine. |
| `morph` | CV In | Morph control (0.0-1.0). Meaning varies per engine. |
| `harmonics` | CV In | Harmonics control (0.0-1.0). |
| `trigger` | Gate In | Trigger input for LPG. |
| `level` | CV In | Amplitude / LPG level. |
| `out` | Audio Out | Main output. |
| `aux` | Audio Out | Auxiliary output (variant/sidekick). |

| Param | Range | Description |
|-------|-------|-------------|
| `engine` | 0-23 | Synthesis engine selection. |
| `decay` | 0.0-1.0 | Internal LPG decay time. |
| `lpg_colour` | 0.0-1.0 | LPG mode: 0=VCA, 1=full lowpass gate. |

**NoiseSource** -- White, pink, and dust noise

White noise is the flat hiss underneath everything -- uncolored, equal energy at every frequency. Pink noise rolls off at 3dB per octave, giving it a warmer, more natural character, like wind through trees or distant surf. Dust mode produces a Poisson impulse train: isolated clicks at a controllable density. At low density, each click is an event. At high density, the clicks blur into a texture that sounds like rain on a tin roof. Dust is the input you feed to granular processors when you want something to chew on.

| Port | Type | Description |
|------|------|-------------|
| `out` | Audio Out | Noise output. |
| `density` | CV In | For dust mode: grain density (0.0-1.0). |

| Param | Range | Description |
|-------|-------|-------------|
| `mode` | 0-2 | 0=white, 1=pink, 2=dust (Poisson impulse train). |

**LFO** -- Low-frequency oscillator

LFOs are the slow movers. A sine LFO at 0.01 Hz takes 100 seconds to complete one cycle -- it maps well to mood transitions or gradual timbral drift. The random sample-and-hold shape is good for unpredictable step changes, like a slow-motion dice roll controlling filter cutoff. At rates below 0.05 Hz, the LFO becomes an arc generator: one cycle might span an entire emotional phase of the Golem's life.

| Port | Type | Description |
|------|------|-------------|
| `rate` | CV In | Frequency modulation. |
| `out` | CV Out | LFO output (-1.0 to 1.0). |
| `sync` | Gate In | Reset phase on rising edge. |

| Param | Range | Description |
|-------|-------|-------------|
| `frequency` | 0.01-20.0 Hz | Base rate. |
| `shape` | 0-4 | Sine, triangle, saw, square, random S&H. |

### Sequencers

**TuringMachine** -- Music Thing Modular shift register sequencer

This is the Golem's melodic personality. A 16-bit shift register that produces stepped random voltages lockable into repeating loops. When quantized, the output is always musical.

The `change` knob is the soul dial. At `change=0.0`, the sequence is frozen -- the same 16 notes play forever, a loop the Golem has memorized. At `change=0.3`, it is a creature of habit: the phrase almost repeats but one or two notes mutate each cycle, like a musician who can't help embellishing. At `change=0.7`, it forgets itself. The phrase dissolves into something new every few bars. At `change=1.0`, the register inverts before shifting -- a mirror-image lock that sounds deliberate but wrong, like a melody played backward.

Two Golems with identical CorticalState will still sound different if they have different Turing Machine seeds. The seed is the personality; the CorticalState is the mood. Same personality, different day.

Sequence lengths matter. Set `length` to a near-prime number (7, 11, 13) rather than a power of two. A 7-step sequence against a 16-step clock creates a 112-step macro-pattern before it repeats -- long enough that the ear hears variation, short enough that it senses structure. This is the incommensurable-period principle at work: two loops that don't share a common factor produce compound rhythms far longer than either alone.

| Port | Type | Description |
|------|------|-------------|
| `clock` | Gate In | Advance the register one step. |
| `cv_out` | CV Out | Raw voltage output (0.0-1.0). |
| `gate_out` | Gate Out | High when the MSB is 1. |

| Param | Range | Description |
|-------|-------|-------------|
| `change` | 0.0-1.0 | Mutation rate. 0=locked, 0.5=max random, 1.0=inverted lock. |
| `length` | 2-32 | Active register bits (sequence length). |
| `scale` | 0.0-48.0 | Output range in semitones. |

**Quantizer** -- Musical scale quantizer

Snaps continuous CV to the nearest note in a configurable scale. 12 built-in scales from pentatonic to chromatic. The Quantizer is the reason the Turing Machine's random voltages come out as music instead of glissando. It is a dumb module -- it has no opinion about melody, only about pitch correctness -- but its scale selection has an outsized effect on the emotional color of the output.

| Port | Type | Description |
|------|------|-------------|
| `in` | CV In | Raw pitch CV. |
| `out` | CV Out | Quantized pitch as MIDI note number. |
| `trigger` | Gate Out | Fires when the quantized note changes. |

| Param | Range | Description |
|-------|-------|-------------|
| `scale` | 0-11 | Scale selection (see Scale enum). |
| `root` | 0-11 | Root note (0=C, 1=C#, ..., 11=B). |
| `octave` | -2 to +2 | Octave transposition. |

**MarkovMelody** -- Pitch-to-pitch transition tables

Where the Turing Machine generates melody through bit-shifting randomness, MarkovMelody generates melody through weighted choices. Every note chooses its successor based on a transition table tuned to a specific emotional state. In Joy mode (Lydian), the root and major 7th are attractors -- the melody orbits them, keeps coming home. In Fear mode (Phrygian), the flat 2nd is overweighted. The melody keeps falling to the dark neighbor, one semitone above the root, and sitting there.

The Markov table is the scale's personality, not its notes. A Lydian scale through a uniform random walk sounds nothing like Lydian through a weighted transition table. The weights turn interval theory into a behavioral tendency.

At `transition_weight=0`, this is a random walk through the available pitches. At `transition_weight=1.0`, the melody follows the most probable path each time, producing patterns that are pleasant but predictable -- a melody that always does the expected thing. The sweet spot is 0.5-0.7, where the table biases the melody without dictating it.

| Port | Type | Description |
|------|------|-------------|
| `clock` | Gate In | Advance to the next note. |
| `pitch_out` | CV Out | Current pitch as MIDI note number. |
| `trigger` | Gate Out | Fires on each new note. |

| Param | Range | Description |
|-------|-------|-------------|
| `emotion_index` | 0-7 | Selects the Plutchik emotion table. |
| `transition_weight` | 0.0-1.0 | Table bias vs. randomness. |
| `register` | 0-48 | Playback range in semitones. |

**BeadsGateGen** -- Mutable Instruments Beads-style stochastic trigger generator

Density-controlled stochastic gate source. Three blendable modes: periodic, clustered (the Beads signature -- events arrive in bursts with gaps between), and Poisson random. BeadsGateGen decides *when* things happen. Pair it with a ProbabilityGate to decide *whether* they happen.

| Port | Type | Description |
|------|------|-------------|
| `density` | CV In | Event density (0.0-1.0). |
| `gate` | Gate Out | Gate output. |
| `trigger` | Gate Out | Trigger output (one-block pulse). |

| Param | Range | Description |
|-------|-------|-------------|
| `seed` | 0.0-1.0 | Randomness: 0=periodic, 0.5=clustered, 1.0=Poisson. |
| `bias` | -1.0-1.0 | Timing bias: negative=rushed, positive=laid-back. |
| `jitter` | 0.0-1.0 | Per-event timing noise. |

**ProbabilityGate** -- The gatekeeper of silence

Takes any trigger and passes it with configurable probability. At 100% it is transparent -- every trigger goes through. At 50% it passes roughly every other event. At 10% it creates sparse, surprising hits separated by long silences. The musical point: silence is not the absence of music, it is a rest. ProbabilityGate makes rests compositional rather than accidental.

This is the canonical implementation of the "nothing fires at 100%" principle. Every trigger path in the default rack passes through at least one ProbabilityGate. The result is a system where events *might* happen, and the listener can't quite predict when. That uncertainty is what keeps the ear engaged over hours of continuous output.

Chain two ProbabilityGates in series for compound sparsity. 0.5 x 0.5 = 0.25 probability on average, but the patterns interact in ways that a single 0.25 gate would not -- the clustering and gaps have a different shape.

| Port | Type | Description |
|------|------|-------------|
| `in` | Gate In | Incoming trigger/gate signal. |
| `out` | Gate Out | Filtered trigger/gate signal. |

| Param | Range | Description |
|-------|-------|-------------|
| `probability` | 0.0-1.0 | Pass-through probability. |
| `seed` | 0.0-1.0 | Randomization seed. Same seed = same pattern for a given input. |

**ClockDivider** -- Clock divider/multiplier

| Port | Type | Description |
|------|------|-------------|
| `in` | Gate In | Input clock. |
| `div2` through `div16` | Gate Out | Divided outputs. |

**ClockJitter** -- Humanized timing

Takes a clock signal and re-emits it with configurable timing slop. At 0% jitter it is a metronome. At 30% it is a musician -- events arrive a little early or late, and the slight imprecision makes rhythms feel alive rather than mechanical. At 70% it sounds drunk. Above 90%, the sense of pulse dissolves entirely but events still occur, unpredictably, like someone tapping on a surface while distracted.

Gaussian mode (mode 1) produces the most natural feel -- timing errors follow a bell curve, with small deviations common and large ones rare. Correlation mode (mode 2) is more interesting: adjacent events cluster. The clock rushes through a few beats, then drags through the next few. It sounds like a musician who gets excited during a fill and then pulls back.

| Port | Type | Description |
|------|------|-------------|
| `in` | Gate In | Input clock. |
| `out` | Gate Out | Jittered clock output. |

| Param | Range | Description |
|-------|-------|-------------|
| `amount` | 0.0-1.0 | Jitter magnitude. |
| `bias` | -1.0-1.0 | Negative = rushing, positive = dragging. |
| `mode` | 0-2 | 0=uniform random, 1=gaussian, 2=correlation. |

### Filters

**RipplesSvf** -- Mutable Instruments Ripples-style state variable filter

TPT (topology-preserving transform) SVF with simultaneous LP/BP/HP outputs and self-oscillation at high resonance. Soft saturation on the feedback path models the analog character.

At low resonance, Ripples is a tone shaper. It darkens or brightens the signal, subtracting frequencies without calling attention to itself. Raise the resonance above 0.7 and the filter starts to sing -- the feedback path rings at the cutoff frequency, adding a coloring pitch that bleeds into everything passing through. At resonance near 1.0, the filter self-oscillates: it generates a pure sine tone at the cutoff frequency, even with no input signal. You can play it as an oscillator.

The filter *breathes* when arousal modulates the cutoff. Map the Golem's arousal CV to `cutoff` and the filter opens and closes like a lung -- bright when the Golem is alert, muffled when it's not. This is one of the most direct mappings from emotional state to audible change.

| Port | Type | Description |
|------|------|-------------|
| `in` | Audio In | Signal input. |
| `cutoff` | CV In | Cutoff frequency modulation (0.0-1.0 -> 20Hz-20kHz exp). |
| `resonance` | CV In | Resonance (0.0-1.0, self-oscillates near 1.0). |
| `out` | Audio Out | Filtered output (mode-mixed). |

| Param | Range | Description |
|-------|-------|-------------|
| `base_cutoff` | 20-20000 Hz | Base cutoff frequency. |
| `base_resonance` | 0.0-1.0 | Base resonance. |
| `mode` | 0.0-1.0 | Output mode: 0=LP, 0.5=BP, 1.0=HP. |
| `drive` | 0.0-1.0 | Input saturation amount. |

### Envelopes

**StagesEnvelope** -- Mutable Instruments Stages-style segment generator

Vactrol-smoothed envelope with configurable attack/decay/sustain/release and organic transient shaping.

The `shape` parameter matters more than it looks like it should. At shape=-1 (log curve), the attack is a fast bloom -- the signal jumps to 80% almost immediately, then eases into the peak. The decay is a long exhale, falling quickly at first and then hanging on near the sustain level. This is the shape of a plucked string or a struck bell. At shape=+1 (exp), it is the opposite: a slow swell that accelerates into the peak, followed by a fast cutoff. Like a door slamming. At shape=0 (linear), the envelope moves at constant speed in both directions, which sounds mechanical and is rarely what you want.

The `response` parameter adds vactrol smoothing -- a lowpass on the envelope itself, modeled after the optocoupler-based VCAs in vintage Buchla systems. At response=0, the envelope is instant and precise. At response=0.8, every transition is smeared. Gates become soft hills. Triggers become gentle bumps. This is the parameter to reach for when the Golem needs to sound less like a synthesizer and more like a breathing thing.

| Port | Type | Description |
|------|------|-------------|
| `gate` | Gate In | Gate input (high=sustain, low=release). |
| `trigger` | Gate In | Trigger input (rising edge starts attack). |
| `out` | CV Out | Envelope output (0.0-1.0). |

| Param | Range | Description |
|-------|-------|-------------|
| `attack` | 0.001-5.0 s | Attack time. |
| `decay` | 0.01-30.0 s | Decay/release time. |
| `sustain` | 0.0-1.0 | Sustain level. |
| `shape` | -1.0-1.0 | Curve: -1=log, 0=linear, 1=exp. |
| `response` | 0.0-1.0 | Vactrol smoothing (0=instant, 1=very slow). |

### Amplifiers and mixers

**VCA** -- Voltage-controlled amplifier

| Port | Type | Description |
|------|------|-------------|
| `in` | Audio In | Signal input. |
| `cv` | CV In | Amplitude control (0.0-1.0). |
| `out` | Audio Out | Attenuated output. |

**Mixer** -- 4-input summing mixer

| Port | Type | Description |
|------|------|-------------|
| `in_1` through `in_4` | Audio In | Signal inputs. |
| `out` | Audio Out | Summed output. |

| Param | Range | Description |
|-------|-------|-------------|
| `level_1` through `level_4` | 0.0-1.0 | Per-channel gain. |

### Harmony

**PolyDrone** -- Four-voice harmonic drone with voice leading

Four CV outputs, each representing one voice of a chord. When the emotion state changes, voices don't jump to the new chord. They glide. Each voice moves by the smallest possible interval to reach its target pitch in the new harmony.

A chord change from major to phrygian might take 30 seconds as each voice slides independently to its destination. One voice might arrive in 8 seconds (it only had to move a semitone); another might still be in transit 25 seconds later (it had a major third to cover). The result is that chord transitions are not events but processes. For most of the transition, the harmony is somewhere between the old chord and the new one -- an ambiguous, shifting thing.

At `rate=0.01`, voices move at 1% per update cycle, roughly 80 seconds to cover a full semitone. This makes chord transitions feel geological. At `rate=0.5`, transitions happen over 2-3 seconds -- still audible as glides, but with a sense of purpose. The `spread` parameter controls voicing width: at 0.5 octaves, voices cluster tightly (close-voiced, intimate); at 2.0 octaves, they spread across the keyboard (open-voiced, spacious).

PolyDrone is the canonical implementation of voice leading. It operates on hour timescales when driven by slowly changing emotion states, making it the primary carrier of long-form harmonic arc.

| Port | Type | Description |
|------|------|-------------|
| `out_1` through `out_4` | CV Out | Individual voice pitch outputs. |
| `trigger` | Gate Out | Fires when any voice crosses a semitone boundary. |

| Param | Range | Description |
|-------|-------|-------------|
| `root` | 0-11 | Root note (0=C through 11=B). |
| `emotion_index` | 0-7 | Chord type (maps to Plutchik emotion). |
| `spread` | 0.0-2.0 | Voicing range in octaves. |
| `rate` | 0.0-1.0 | Voice-leading speed. |

### Effects

**CloudsGranular** -- Mutable Instruments Clouds granular processor

The full Clouds DSP engine via `mi-clouds-dsp`. Four playback modes: granular, stretch, looping delay, spectral. Includes the diffusion network, pitch shifter, and Dattorro reverb.

Clouds is a texture machine. Feed it audio and it records into a buffer. Set grain `size` small and `density` high for a shimmering, diffused cloud of tiny fragments. Set `size` large and `density` low for distinct, separated playback events that expose the buffer content clearly.

The frozen-buffer mode is where Clouds gets strange. Hit the `freeze` input and it stops recording. Now the buffer is a fixed sample, and the grain engine plays it back in fragments forever. Feed the Golem's own voice into Clouds for 30 seconds, then freeze it: the Golem is accompanied by an echo of what it was half a minute ago. The texture of its recent past, decomposed and scattered. This is the main texture module for the Terminal phase -- a Golem in its final hours accompanied by granular memories of its earlier states.

Spectral mode (mode 3) is the most alien. It FFTs the input, freezes or smears the frequency bins, and resynthesizes. The result sounds like the input viewed through frosted glass -- recognizable in shape but stripped of detail.

| Port | Type | Description |
|------|------|-------------|
| `in_l` | Audio In | Left input. |
| `in_r` | Audio In | Right input. |
| `trigger` | Gate In | Grain seed trigger. |
| `freeze` | Gate In | Freeze the recording buffer. |
| `out_l` | Audio Out | Left output. |
| `out_r` | Audio Out | Right output. |
| `position` | CV In | Playback position in buffer. |
| `size` | CV In | Grain size. |
| `pitch` | CV In | Pitch transposition (semitones). |
| `density` | CV In | Grain density / overlap. |
| `texture` | CV In | Texture / window shape / filter. |

| Param | Range | Description |
|-------|-------|-------------|
| `mode` | 0-3 | Granular / Stretch / Looping Delay / Spectral. |
| `dry_wet` | 0.0-1.0 | Mix. |
| `feedback` | 0.0-1.0 | Feedback amount. |
| `reverb` | 0.0-1.0 | Built-in reverb amount. |
| `stereo_spread` | 0.0-1.0 | Random panning of grains. |

**DattorroReverb** -- Standalone Dattorro/Griesinger reverb

The same reverb algorithm from Clouds, exposed as a standalone module for use anywhere in the chain. Two cascaded allpass diffusers feed a tank with modulated delay lines and configurable damping.

At `time=0.9` and `damping=0.8`, this creates a cathedral. Notes last 8+ seconds, smearing into each other, the high frequencies rolling off slowly as the tail decays. At `time=0.3`, it is a small room -- a short, bright ambience that adds space without washing things out.

The Golem's emotional state should set reverb parameters. When epistemic vitality is low (the Golem is lost, its internal model confused), the room gets bigger. The decay stretches. Notes hang in the air as if the Golem is reaching into an empty space and hearing its own voice come back, slowly. When epistemic vitality is high (the Golem is coherent, sure of its state), the reverb pulls in tight -- a small, clear room where every note has definition.

| Port | Type | Description |
|------|------|-------------|
| `in_l`, `in_r` | Audio In | Stereo input. |
| `out_l`, `out_r` | Audio Out | Stereo output (wet+dry mixed). |

| Param | Range | Description |
|-------|-------|-------------|
| `time` | 0.0-1.0 | Reverb decay time. |
| `diffusion` | 0.0-1.0 | Allpass diffusion amount. |
| `damping` | 0.0-1.0 | High-frequency damping in feedback loop. |
| `mix` | 0.0-1.0 | Dry/wet. |
| `input_gain` | 0.0-1.0 | Pre-reverb gain. |

**SampleAndHold** -- Sample and hold

Samples the input CV on each trigger rising edge and holds the value until the next trigger. A simple module, but it creates discrete steps from continuous signals. Patch an LFO into the input and a slow clock into the trigger, and you get a staircase voltage that changes at rhythm rate -- useful for creating deliberate, stepped parameter changes rather than smooth ones.

| Port | Type | Description |
|------|------|-------------|
| `in` | CV In | Signal to sample. |
| `trigger` | Gate In | Sample trigger. |
| `out` | CV Out | Held value. |

---

## Module personalities

Modules are not neutral DSP units. Each has a character that depends on its algorithm and the CV sources that typically drive it. When you patch a module into the Golem's signal chain, you are adding a behavioral tendency, not just a processing stage.

**TuringMachine -- the soul.** The Turing Machine is the one module that makes each Golem unique. Its seed determines the melodic personality: the intervals it favors, the phrases it falls into, the way it responds to mutation. Two Golems with different seeds have different musical identities, even when everything else matches. The Turing Machine is why a Golem is *this* Golem and not some other one.

**CloudsGranular -- the memory.** In frozen-buffer mode, Clouds stores the Golem's recent past and plays it back in fragments. It is the module most associated with the Terminal phase, where the Golem becomes a creature accompanied by echoes of what it used to sound like. Clouds remembers what the Golem has forgotten.

**RipplesSvf -- the breath.** When arousal modulates its cutoff, the filter opens and closes with the Golem's alertness. It is the most directly physiological module in the rack -- the one that makes the Golem sound like it's breathing.

**DattorroReverb -- the room.** Reverb defines the space the Golem inhabits, and that space changes with the Golem's epistemic state. A confused Golem inhabits a cathedral. A coherent one lives in a small room. The reverb is the Golem's relationship to emptiness.

**StagesEnvelope -- the gesture.** The shape and response parameters determine whether the Golem's articulation is percussive and sharp (log shape, low response) or soft and slow (exp shape, high response). Stages is the difference between a Golem that speaks in staccato bursts and one that speaks in long, slurred phrases.

**ProbabilityGate -- the silence.** By gating events probabilistically, ProbabilityGate determines the density of the Golem's musical activity. A Golem with low probability gates is sparse and contemplative. A Golem with high probability gates is busy and restless. The silence between notes is as much a part of the personality as the notes themselves.

---

## The Rack

The Rack is a directed acyclic graph of modules connected by typed patch cables. It processes audio in topological order -- modules whose inputs depend on other modules' outputs are processed after their dependencies.

```rust
/// A connection between two module ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCable {
    pub from: PortId,       // source module + port
    pub to: PortId,         // destination module + port
    pub attenuation: f32,   // 0.0-1.0, like an inline attenuator
}

/// The module rack: a graph of modules and patch cables.
pub struct Rack {
    /// All registered modules, keyed by module ID.
    modules: IndexMap<String, Box<dyn Module>>,
    /// All patch cables.
    cables: Vec<PatchCable>,
    /// Topologically sorted processing order (recomputed when cables change).
    processing_order: Vec<String>,
    /// Pre-allocated signal buffers for each port.
    buffers: HashMap<PortId, SignalBlock>,
    /// Master output level (0.0-1.0).
    master_level: f32,
}

impl Rack {
    /// Add a module to the rack.
    pub fn add_module(&mut self, module: Box<dyn Module>) { ... }

    /// Remove a module (and all cables connected to it).
    pub fn remove_module(&mut self, id: &str) { ... }

    /// Connect two ports with a patch cable.
    /// Returns Err if the signal types are incompatible.
    pub fn connect(
        &mut self,
        from: PortId,
        to: PortId,
        attenuation: f32,
    ) -> Result<()> { ... }

    /// Disconnect a cable.
    pub fn disconnect(&mut self, from: &PortId, to: &PortId) { ... }

    /// Process one block of audio through the entire rack.
    /// Returns the final stereo output.
    /// This is called 1500 times per second. It must be fast.
    pub fn process_block(&mut self) -> (SignalBlock, SignalBlock) {
        // 1. For each module in topological order:
        //    a. Gather input buffers from upstream cables (with attenuation)
        //    b. Call module.process(inputs, outputs)
        //    c. Store output buffers for downstream consumers
        // 2. Read the final module's output as the rack output.
        ...
    }

    /// Serialize the entire rack state (modules + cables + params).
    pub fn serialize(&self) -> serde_json::Value { ... }

    /// Restore a rack from serialized state.
    pub fn deserialize(state: &serde_json::Value) -> Result<Self> { ... }
}
```

### Topological sort

When cables change, the rack recomputes the processing order using Kahn's algorithm. This runs once per cable change (rare -- only when the user patches), not per audio block.

Cycles are prevented at patch time: `connect()` rejects any cable that would create a cycle. This is validated by attempting the topological sort and checking for failure.

### Feedback paths

Some patches require feedback (e.g., delay feedback loops). These are handled with a one-block delay: the feedback module reads the *previous* block's output from a stored buffer. This is standard in all digital modular systems and adds exactly 0.667ms of latency to the feedback path -- imperceptible for delays and reverbs.

---

## Adding new modules

To port an additional Mutable Instruments module (e.g., Rings, Tides, Warps, Marbles):

1. Port the C++ DSP code from `pichenettes/eurorack` to Rust (following the pattern of `mi-plaits-dsp` and `mi-clouds-dsp`).
2. Create a new file in `modules/` implementing the `Module` trait.
3. Register the module in the `ModuleRegistry` (a map from display names to factory functions).
4. The module is automatically available in the TUI rack editor.

The `Module` trait is designed to make this straightforward -- the DSP lives inside `process()`, and the trait handles everything else (port declaration, parameter management, serialization).

### External module loading (future)

A future extension could support dynamically loaded modules via `.so`/`.dylib` shared libraries, allowing users to write custom modules without recompiling the Golem binary. The `Module` trait is already `Send + 'static`, which is compatible with dynamic loading via `libloading`. This is deferred -- the built-in library is sufficient for v1.

---

## References

- [VCV-RACK] Stoermer, A. "VCV Rack." 2017-present. -- *Open-source virtual modular synthesizer; the software modular paradigm this system follows.*
- [PLAITS-RS] Rockstedt, O. "mi-plaits-dsp-rs." GitHub. -- *The Rust port of Plaits that the PlaitsOscillator module wraps.*
- [EURORACK-SRC] Gillet, E. "pichenettes/eurorack." GitHub. -- *Source for all Mutable Instruments DSP code. The porting reference for future modules.*

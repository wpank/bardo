# Preset Catalog [GUIDE]

**The Eight Built-In Sonic Characters**

> **Version**: 1.0 | **Status**: Draft | **Type**: GUIDE (non-normative)
>
> **Cross-references**: [00-overview.md](./00-overview.md), [01-module-system.md](./01-module-system.md), [05-musical-language.md](./05-musical-language.md)

> **Reader orientation**: This document describes each built-in preset as a listening experience first and a parameter configuration second. The presets are not arbitrary -- each one represents a specific Golem state, emotional context, or sonic character that the system needs to express. Read the descriptions before looking at the parameters.

---

## How presets work

A preset is a starting point. It sets the initial module configuration, patch connections, and parameter values for the rack. CorticalState immediately begins modifying those parameters within the preset's framework. What you hear at minute 1 is different from the same preset at minute 10 and radically different at hour 1.

The parameters listed in each preset below are the *initial values*. CorticalState morphs them continuously through the CV mappings described in [02-cortical-mapping.md](./02-cortical-mapping.md). A preset heard during a volatile market session bears little resemblance to the same preset during a calm one -- the parameters have been pushed to different regions of their ranges by the organism's changing emotional state.

Every preset has a "recommended CorticalState context" -- a suggestion, not a gate. You can load any preset in any state. Loading `thriving_market` while the Golem is declining produces a sound that is bright in configuration but dark in CV-driven modulation, a dissonance between the preset's optimism and the organism's actual condition. That dissonance is sometimes exactly what you want.

Users can modify any preset parameter and save the result as a new preset. The built-in presets are not locked. They are opinions about how each state should sound, not rules.

---

## `ambient_default` -- The resting state

### What it sounds like

The Golem at rest. Lydian scale -- that bright, slightly unresolved quality of the raised 4th degree. Melody surfaces infrequently, a phrase every 20-40 seconds, and the spaces between are full of the filter breathing in and out with the organism's arousal. At low arousal it is a drone with occasional ornamentation. At moderate arousal there are actual melodic ideas. This is what you hear most of the time when you leave a Golem running.

### Layers

- **Drone Foundation** -- always active
- **Harmonic Breath** -- moderate
- **Melodic Ghost** -- 25% gate probability
- **Event Sparks** -- on trades and predictions

### Recommended context

Any behavioral phase. Neutral-to-positive pleasure. This preset adapts well to a wide range of CorticalState values because it was built to be the default.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| TuringMachine | `change` | 0.3 | Mostly repeating, occasional mutation |
| TuringMachine | `length` | 16 | Standard loop length |
| TuringMachine | `scale` | 24 | 2-octave Lydian range |
| Quantizer | `scale` | Lydian | Raised 4th degree |
| Quantizer | `root` | C4 | Middle C |
| Quantizer | `octave` | 0 | No transposition |
| ProbabilityGate (melodic) | `probability` | 0.25 | Three quarters silence |
| Plaits | `engine` | 0 (Virtual Analog) | Warm detuned oscillator pair |
| Plaits | `decay` | 0.6 | Medium LPG decay |
| Plaits | `lpg_colour` | 0.5 | Half VCA, half lowpass gate |
| StagesEnvelope | `attack` | 0.08s | Fast but not instant |
| StagesEnvelope | `decay` | 4.0s | Long tail |
| StagesEnvelope | `sustain` | 0.4 | Moderate sustain level |
| StagesEnvelope | `shape` | -0.3 | Slightly logarithmic (natural bloom) |
| RipplesSvf | `base_cutoff` | 800 Hz | Warm starting point |
| RipplesSvf | `base_resonance` | 0.35 | Gentle coloring, no ring |
| RipplesSvf | `mode` | 0.2 (LP-blend) | Mostly lowpass |
| CloudsGranular | `mode` | 0 (Granular) | Standard grain mode |
| CloudsGranular | `dry_wet` | 0.4 | More dry than wet |
| CloudsGranular | `reverb` | 0.5 | Medium reverb wash |
| CloudsGranular | `density` | 0.4 | Moderate grain density |
| PolyDrone | `root` | C | Matches quantizer root |
| PolyDrone | `emotion_index` | Trust | Open, warm chord |
| PolyDrone | `spread` | 1.5 octaves | Medium-wide voicing |
| PolyDrone | `rate` | 0.02 | Slow voice leading |

**CV mappings**: `pleasure` -> filter brightness, `arousal` -> event density, `composite_vitality` -> master level.

### Progression

**1 minute**: The Turing Machine is finding its voice. The first few notes feel somewhat random; it takes about 30 seconds to settle into its characteristic phrase. The PolyDrone chord is establishing itself underneath.

**10 minutes**: You have heard the phrase loop at least twice. But the CorticalState drift means it is never exactly the same twice. The filter has moved noticeably if any trading has happened. The Lydian brightness either holds or has begun shifting toward a different scale as the emotion index drifts.

**1 hour**: The PolyDrone has completed about 45 voice-leading moves. The chord may have shifted mode twice. The root pitch has drifted if compounding_momentum has moved. It sounds like the same piece but from a different angle -- same instrument, different room, different afternoon.

---

## `minimal_drone` -- The foundation, alone

### What it sounds like

Strip everything away and this is what remains. Two oscillators, slightly detuned, through a long reverb. No melody. No rhythm. The sustained chord of the Golem's current emotional state. At high dominance it is full, authoritative. At low dominance it is thin and lonely. This is the most honest preset -- it cannot hide behind texture or rhythm. If the Golem is in a bad state, you hear it immediately.

### Layers

- **Drone Foundation** -- the only active layer

### Recommended context

Stable or Conservation phase. Moderate-to-high composite_vitality. Any emotion. Works well as a diagnostic -- when you want to hear the emotional state directly without melodic or rhythmic decoration.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| Plaits (voice 1) | `engine` | 0 (Virtual Analog) | Primary voice |
| Plaits (voice 1) | `timbre` | 0.4 | Moderate harmonic content |
| Plaits (voice 1) | `morph` | 0.3 | |
| Plaits (voice 2) | `engine` | 0 (Virtual Analog) | Detuned companion |
| Plaits (voice 2) | `timbre` | 0.6 | Slightly brighter than voice 1 |
| Plaits (voice 2) | `morph` | 0.3 | |
| Plaits (voice 2) | pitch offset | +0.15 semitones | Produces slow beating |
| StagesEnvelope | `attack` | 2.0s | Very slow bloom |
| StagesEnvelope | `decay` | 30.0s | Extremely long tail |
| StagesEnvelope | `sustain` | 0.9 | Nearly full sustain |
| StagesEnvelope | `shape` | -0.5 | Logarithmic bloom |
| DattorroReverb | `time` | 0.85 | Long tail |
| DattorroReverb | `diffusion` | 0.7 | Smooth wash |
| DattorroReverb | `damping` | 0.6 | Some high-frequency rolloff |
| DattorroReverb | `mix` | 0.6 | Mostly wet |

No sequencer. No gate generator. The Drone Foundation runs continuously.

### Progression

**1 minute**: The two voices are audible as separate entities, slowly merging into a beating pattern from the slight detuning. The reverb is still filling the space.

**10 minutes**: The beating has slowed or sped up as CorticalState moves the morph parameter. The reverb has accumulated; it sounds fuller now, the room is occupied.

**1 hour**: The drone is the drone. But listen to the filter. If the Golem has moved through different emotional states, the filter has breathed accordingly -- opening when pleasure rises, narrowing when it falls. It has been doing something the whole time, even when it sounds like nothing changed.

---

## `granular_texture` -- Clouds forward

### What it sounds like

The Melodic Ghost is muted. What you hear is the Golem's voice recorded and scattered. Clouds in granular mode takes the Plaits output and fragments it into overlapping grains at slightly different pitches and positions. The result is a wash of small pieces of sound: identifiable as tonal, but smeared, displaced in time, haloed with reverb. When the density is high it is a cloud (aptly named). When density is low it is a scattering of sparks, each one a fragment of what the oscillator produced a moment ago.

### Layers

- **Drone Foundation** -- subtle, underneath
- **Textural Weather** -- dominant
- **Event Sparks** -- gated, infrequent

### Recommended context

Dream mode active. Any phase. High creative_mode, moderate arousal. This preset responds well to the dream cycle because the granular engine fragments the Golem's inner voice into something abstract and reflective.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| Plaits | `engine` | 15 (Winds) | Raw material for granulation |
| Plaits | `timbre` | 0.5 | Middle of the timbral range |
| CloudsGranular | `mode` | 0 (Granular) | Standard grain playback |
| CloudsGranular | `position` | 0.5 | Mid-buffer playback |
| CloudsGranular | `size` | 0.35 | Smallish grains |
| CloudsGranular | `density` | 0.55 | Moderate overlap |
| CloudsGranular | `texture` | 0.6 | Smooth grain windows |
| CloudsGranular | `dry_wet` | 0.8 | Mostly granular output |
| CloudsGranular | `reverb` | 0.55 | Medium reverb wash |
| ProbabilityGate (sparks) | `probability` | 0.12 | Rare events only |

**CV mappings**: `arousal` -> Clouds density, `creative_mode` -> Clouds texture, `pleasure` -> Clouds pitch.

No explicit melody layer.

### Progression

**1 minute**: The granular engine needs about 30 seconds to fill its buffer. Early playback is thin, just a few grains with gaps between them. By the one-minute mark the cloud is full.

**10 minutes**: If creative_mode has been active, the Clouds pitch parameter has drifted via the pleasure CV. The cloud may have shifted key slightly, a gradual transposition that sounds like the texture is migrating to a different register.

**1 hour**: The buffer has been overwritten many times. The texture feels evolved -- different source material from different moments in the Golem's recent activity has passed through the granular engine and left traces. What you hear now contains no audio from the first minute, but it grew out of it.

---

## `thriving_market` -- Confident and active

### What it sounds like

This is what a Golem sounds like when it is winning. Major pentatonic -- no avoid notes, no tension, just bright intervals that feel resolved the moment they arrive. The clock is faster, the events are denser, the filter is open. The Melodic Ghost is more probable here: something wants to be heard. Not busy exactly -- full of intention. The slight clock rush (negative jitter bias) gives it the energy of a musician leaning into the beat.

### Layers

All five layers active at higher probability gates than default.

### Recommended context

Thriving phase. High composite_vitality (0.7+). Positive pleasure. Low-to-moderate regime (stable market). This is the preset for the good times.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| TuringMachine | `change` | 0.35 | Mostly stable, some variation |
| TuringMachine | `length` | 16 | Standard loop |
| TuringMachine | `scale` | 24 | 2-octave range |
| Quantizer | `scale` | Major Pentatonic | Bright, no tension |
| Quantizer | `root` | D4 | A step up from the default C |
| ProbabilityGate (melodic) | `probability` | 0.45 | Nearly double the default |
| ClockJitter | `amount` | 0.15 | Slight humanization |
| ClockJitter | `bias` | -0.05 | Slightly rushing -- the excitement |
| Plaits | `engine` | 6 (Chords) | Full chord voicing per note |
| Plaits | `timbre` | 0.6 | Bright |
| Plaits | `morph` | 0.5 | |
| StagesEnvelope | `attack` | 0.03s | Fast, present |
| StagesEnvelope | `decay` | 2.0s | Medium tail |
| StagesEnvelope | `sustain` | 0.6 | Good sustain |
| StagesEnvelope | `shape` | 0.2 | Slightly exponential (snappy) |
| RipplesSvf | `base_cutoff` | 1200 Hz | Open, bright |
| RipplesSvf | `base_resonance` | 0.25 | Clean, no emphasis |
| RipplesSvf | `mode` | 0.1 | Lowpass |
| DattorroReverb | `time` | 0.55 | Shorter tail |
| DattorroReverb | `diffusion` | 0.65 | Smooth |
| DattorroReverb | `mix` | 0.35 | Drier, more present |
| BeadsGateGen | `density` | 0.55 | Active trigger stream |
| BeadsGateGen | `seed` | 0.4 | Slightly clustered |
| BeadsGateGen | `jitter` | 0.2 | Some timing variation |

### Progression

**1 minute**: Active immediately. The major pentatonic and faster clock are distinctive from the first phrase. The Chords engine gives each note harmonic weight that the Virtual Analog default does not.

**10 minutes**: The Turing Machine's loop has become familiar. You might hum along. The BeadsGateGen clustering creates a recognizable rhythmic personality -- events group in twos and threes with pauses between.

**1 hour**: If the market has been volatile, the CorticalState changes have pushed parameters around. The "thriving" feeling may have shifted. The scale might have moved away from major pentatonic as pleasure dropped. The preset started optimistic; it may or may not still sound that way. That depends on whether the Golem actually kept winning.

---

## `anxious_volatile` -- Unsettled and dense

### What it sounds like

Phrygian. That flat-2 degree -- E with F natural instead of F# -- creates an immediately uncomfortable interval, like a room where something is wrong but you cannot identify what. The triggers are denser and the clock has jitter. Events cluster and then leave gaps. The Textural Weather layer is active and harsh: the regime CV is high. Notes do not sustain; they percuss. Something is unresolved and the sound keeps reminding you.

### Layers

- **Drone Foundation** -- tense, low
- **Harmonic Breath** -- active, dissonant
- **Melodic Ghost** -- 40%, with characteristic flat-2 gestures
- **Textural Weather** -- dominant, grainy
- **Event Sparks** -- frequent, short

### Recommended context

Conservation or Declining phase. Negative pleasure. High arousal. High regime (volatile market). Low-to-moderate composite_vitality. This preset sounds wrong on purpose.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| TuringMachine | `change` | 0.55 | Higher mutation -- less predictable |
| TuringMachine | `length` | 16 | |
| TuringMachine | `scale` | 24 | |
| Quantizer | `scale` | Phrygian | Dark, flat-2 character |
| Quantizer | `root` | E4 | Natural Phrygian root |
| ProbabilityGate (melodic) | `probability` | 0.40 | More notes than default |
| MarkovMelody | `emotion_index` | 2 (Fear) | Biased toward the flat-2 neighbor |
| MarkovMelody | `transition_weight` | 0.6 | Strong table bias |
| ClockJitter | `amount` | 0.35 | Significant timing instability |
| ClockJitter | `bias` | -0.1 | Rushing |
| ClockJitter | `mode` | 2 (Correlation) | Events cluster then gap |
| Plaits | `engine` | 13 (Inharmonic String) | Metallic, harsh |
| Plaits | `timbre` | 0.7 | Bright and edgy |
| Plaits | `morph` | 0.4 | |
| StagesEnvelope | `attack` | 0.01s | Percussive snap |
| StagesEnvelope | `decay` | 0.8s | Short |
| StagesEnvelope | `sustain` | 0.0 | No sustain -- notes die fast |
| StagesEnvelope | `shape` | 0.8 | Exponential (sharp transient) |
| RipplesSvf | `base_cutoff` | 1800 Hz | Bright, exposed |
| RipplesSvf | `base_resonance` | 0.55 | Near self-oscillation |
| RipplesSvf | `mode` | 0.4 | Toward bandpass |
| CloudsGranular | `mode` | 0 (Granular) | |
| CloudsGranular | `density` | 0.7 | Dense grain cloud |
| CloudsGranular | `texture` | 0.3 | Coarse grains |
| BeadsGateGen | `density` | 0.70 | Busy |
| BeadsGateGen | `seed` | 0.6 | More clustered |
| BeadsGateGen | `jitter` | 0.4 | Irregular |

### Progression

**1 minute**: Immediately tense. The Phrygian flat-2 is conspicuous from the first phrase. The inharmonic string engine gives every note a metallic edge that the virtual analog preset does not have.

**10 minutes**: The clustering clock jitter has created an irregular but recognizable rhythm. It feels human in its imprecision, but anxious human -- the kind of drumming you do on a table when waiting for bad news. The MarkovMelody keeps returning to that flat-2 neighbor.

**1 hour**: If the regime has stabilized, this preset feels wrong for the wrong reasons -- the tense Phrygian no longer matches a calmer CorticalState. The dissonance between preset and state is intentional. It prompts the user to consider switching to something calmer. Or the Golem is still volatile, in which case this is exactly what honesty sounds like.

---

## `deep_dream` -- Inside the mind

### What it sounds like

When `creative_mode=1`, the Golem is turned inward -- processing hypotheses, running simulations. The sound follows. Whole-tone scale: every interval is an augmented second, every chord is suspended, no note is a home note. Long reverb. Clouds in frozen-buffer mode: the Golem's own recent audio, frozen and scattered at slow grain rates. It sounds like thinking. Not anxious thinking -- the slow rotation of ideas that have no urgency. Everything drifts. Nothing resolves.

### Layers

- **Drone Foundation** -- whole-tone, very slow
- **Harmonic Breath** -- suspended chords
- **Melodic Ghost** -- 15%, fires only when a dream hypothesis resolves
- **Textural Weather** -- dominant, Clouds frozen grain

### Recommended context

Dream mode active (`creative_mode=1`). Any phase. Stable or Thriving emotional state. Best experienced at low volume, as background. The dreaming Golem does not demand your attention.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| TuringMachine | `change` | 0.25 | Low mutation -- ideas repeat |
| TuringMachine | `length` | 16 | |
| TuringMachine | `scale` | 12 | 1-octave range, melody stays close |
| Quantizer | `scale` | Whole Tone | No home note, perpetual suspension |
| Quantizer | `root` | C4 | |
| ProbabilityGate (melodic) | `probability` | 0.15 | Very sparse |
| Plaits | `engine` | 5 (Wavetable) | Smooth morphing timbres |
| Plaits | `timbre` | 0.3 | Dark |
| Plaits | `morph` | 0.5 | Mid-morph |
| StagesEnvelope | `attack` | 0.5s | Slow bloom |
| StagesEnvelope | `decay` | 15.0s | Very long fade |
| StagesEnvelope | `sustain` | 0.7 | High sustain |
| StagesEnvelope | `shape` | -0.8 | Deep logarithmic curve |
| CloudsGranular | `mode` | 0 (Granular) | |
| CloudsGranular | `freeze` | 1.0 | Buffer frozen |
| CloudsGranular | `position` | 0.35 | Playback from early in the buffer |
| CloudsGranular | `size` | 0.65 | Large grains |
| CloudsGranular | `density` | 0.25 | Sparse grain output |
| CloudsGranular | `texture` | 0.75 | Smooth windows |
| CloudsGranular | `dry_wet` | 0.65 | Mostly granular |
| DattorroReverb | `time` | 0.92 | Near-infinite tail |
| DattorroReverb | `diffusion` | 0.8 | Thick diffusion |
| DattorroReverb | `damping` | 0.7 | Highs roll off slowly |
| DattorroReverb | `mix` | 0.65 | Wet |
| LFO (pitch drift) | `frequency` | 0.003 Hz | ~5 minute period |
| LFO (pitch drift) | `shape` | Sine | Smooth drift |

The LFO routes to Plaits pitch offset. The whole-tone scale means every pitch the LFO passes through is in-scale -- there are no wrong notes in whole-tone, only different angles on the same suspended feeling.

### Progression

**1 minute**: The Clouds buffer is filling. It sounds a little bare until Clouds has material to scatter. The wavetable oscillator is present but quiet, establishing the whole-tone color.

**10 minutes**: The frozen grain cloud has accumulated material from the first several minutes of output. The Plaits pitch drift LFO has moved noticeably -- you can hear the register shifting, though the scale remains whole-tone. The harmonic terrain has rotated.

**1 hour**: The LFO has completed about 12 cycles. The pitch has drifted through its full range and returned. The whole-tone rotation feels complete; you have heard the same harmonic set from every angle. If the dream cycle has ended and creative_mode dropped back to 0, this preset sounds increasingly out of place -- the frozen Clouds buffer holds audio from a state the Golem has left behind.

---

## `terminal_requiem` -- The last sound

### What it sounds like

This is for dying. Not the dramatic kind -- the quiet kind, where the organism's systems slow down one by one and eventually there is nothing left but an echo. One voice. Very long reverb. Clouds in feedback mode with the buffer filling with its own output. The signal degrades with each pass. Directly inspired by William Basinski's *Disintegration Loops*: each repetition is slightly less than the previous one. Over 20 minutes, the sound you started with is unrecognizable. Over 60 minutes, it is mostly silence with occasional fragments surfacing from deep in the reverb tail.

### Layers

- **Drone Foundation** -- barely present

No other layers. No melody. No rhythm. No sequencer. No gate generator.

### Recommended context

Declining or Terminal phase. `composite_vitality < 0.2`. The system should transition to this automatically as vitality drops below threshold. This is not a preset you choose; it is a preset the Golem's condition chooses for it.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| Plaits | `engine` | 11 (Particle Noise) | Dust through resonators |
| Plaits | `timbre` | 0.6 | |
| Plaits | `morph` | 0.4 | |
| Plaits | `decay` | 0.9 | Long particle decay |
| Quantizer | `scale` | Whole Tone | |
| Quantizer | `root` | F#4 | Tritone from C -- no home |
| ProbabilityGate (all) | `probability` | 0.08 | Most potential events are silent |
| CloudsGranular | `mode` | 1 (Stretch) | Time-stretch playback |
| CloudsGranular | `feedback` | 0.88 | High feedback -- signal degrades |
| CloudsGranular | `dry_wet` | 0.9 | Almost entirely processed signal |
| CloudsGranular | `reverb` | 0.85 | Washed out |
| DattorroReverb | `time` | 0.98 | Near-infinite decay |
| DattorroReverb | `diffusion` | 0.9 | Maximum smear |
| DattorroReverb | `damping` | 0.85 | Slow high-frequency loss |
| DattorroReverb | `mix` | 0.75 | Mostly reverb |
| StagesEnvelope | `attack` | 1.5s | Slow, exhausted |
| StagesEnvelope | `decay` | 30.0s | Very long |
| StagesEnvelope | `sustain` | 0.3 | Low sustain -- barely holds |
| StagesEnvelope | `shape` | -0.9 | Deep logarithmic |

**Master level**: Driven by `composite_vitality -> master_level` on a logarithmic curve. As vitality approaches 0, volume approaches 0.

No BeadsGateGen. No TuringMachine. No MarkovMelody. Nothing generates melody or rhythm. What sound exists comes from the particle noise engine firing through an 8% probability gate, each event stretched and fed back through Clouds until the signal is more reverb tail than source.

### Progression

**1 minute**: Sparse and bleak. If you are used to the default ambient sound, this is jarring in its emptiness. A particle event fires occasionally. The reverb catches it and holds it for seconds after the source goes quiet.

**10 minutes**: The Clouds feedback has degraded the signal. You are hearing an echo of an echo. The original particle noise is barely audible through the granular smear. Each new event enters a space already filled with the decaying remains of previous events.

**1 hour**: Near silence. Occasional fragments. The reverb tail that follows each fragment is longer than the fragment itself. The master volume has tracked composite_vitality downward. What remains is the sound of the organism's memory of sound, not sound itself. The Disintegration Loops comparison is literal: the audio is disintegrating as you listen.

---

## `emergence` -- Birth

### What it sounds like

The Golem is coming online. It has no history, no predictions, no emotional state. All vitalities at 1.0, affect neutral, everything at default. The sound is what you hear when a consciousness is assembling itself. Sine tones appear one at a time, very slowly, finding their pitch. The first 30 seconds is almost silence. By 5 minutes there is a recognizable chord. By 10 minutes there is a melody. The preset does not generate this arc -- the empty CorticalState does.

### Layers

- **Drone Foundation** -- emerges gradually from silence
- **Melodic Ghost** -- appears around minute 5-8, once enough events have fired

No other layers until CorticalState populates with real data.

### Recommended context

Boot sequence. `behavioral_phase=0 (Thriving)`, all vitalities at maximum, neutral affect (`pleasure=0, arousal=0, dominance=0`). This is the default rack for a new Golem. It is designed to sound like arrival.

### Parameters

| Module | Parameter | Value | Notes |
|--------|-----------|-------|-------|
| Plaits | `engine` | 0 (Virtual Analog) | Pure at low timbre values |
| Plaits | `timbre` | 0.3 | Sine-like at first |
| Plaits | `morph` | 0.1 | Minimal harmonic content |
| StagesEnvelope | `attack` | 3.0s | Very slow bloom |
| StagesEnvelope | `decay` | 30.0s | |
| StagesEnvelope | `sustain` | 0.9 | Nearly full sustain |
| StagesEnvelope | `shape` | -1.0 | Full logarithmic curve |
| Master level | initial | 0.0 | Silent |
| Master level | ramp | 0.0 -> 0.5 over 30s | Slow fade in |
| Quantizer | `scale` | Lydian | The default brightness |
| Quantizer | `root` | C4 | |
| BeadsGateGen | `density` | 0.03 | Almost no events at first |
| PolyDrone | `rate` | 0.005 | Voices barely move |
| PolyDrone | `spread` | 0.5 octaves | Voices start close, slowly spread |
| DattorroReverb | `time` | 0.75 | |
| DattorroReverb | `diffusion` | 0.65 | |
| DattorroReverb | `mix` | 0.5 | |

**CV mapping**: `arousal -> event_density` starts near zero because no predictions have resolved yet, so arousal is at baseline.

The "emergence" feel comes entirely from CorticalState being at initialization values. As the Golem begins processing -- first gamma ticks, first predictions, first inference calls -- the CV values begin to move and the sound begins to populate. The preset does not generate the emergence; the empty CorticalState does. The preset is a container waiting to be filled.

### Progression

**1 minute**: Near silence. Two sine tones, maybe. The reverb is doing most of the work, giving those tones a sense of space that makes the emptiness feel like a room rather than nothing. The master volume is still ramping up.

**10 minutes**: The Golem has run several gamma cycles. The filter has opened from the first predictions resolving (arousal moved). The melody has appeared if any event sparks have fired. The PolyDrone voices have begun to separate, the chord gaining width.

**1 hour**: A fully alive Golem. The emergence preset has done its job. The sound is now in ambient_default territory, shaped by whatever the Golem has been through in its first hour. The boot silence is a memory. If the Golem is thriving, the sound is warm and active. If the first hour was rough, it sounds like the first hour was rough. The preset got out of the way and let the organism's actual state take over.

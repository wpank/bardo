# NFT State Embedding [SPEC]

**A Pressed Record of the Eternal Song**

> **Version**: 2.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-sonification` (nft.rs), `golem-engagement` (mint integration)
>
> **Cross-references**: [00-overview.md](./00-overview.md), [02-cortical-mapping.md](./02-cortical-mapping.md), engagement/nft-minting (existing NFT system)

> **Reader orientation**: This document specifies how a Golem's sonification state is captured and embedded in an NFT at mint time. The NFT is not a recording. It is a frozen point in an ongoing composition, carrying enough information to regenerate the music forever. Anyone holding the NFT can hear exactly what the organism sounded like at the moment it was pressed.

---

## The eternal song

A Golem starts composing the moment it boots. The sonification engine reads CorticalState at 120Hz, maps 32 atomic signals to control voltages, and routes them through a rack of synthesis modules. The result is continuous, real-time ambient sound. It never stops. The music is the Golem's interior monologue made audible: cognition, affect, mortality, attention drifting across markets, all rendered as frequency and rhythm.

Most of this music is heard by nobody. It plays in the terminal of whoever is running the Golem, or it plays to an empty room, or it plays to no output device at all. It passes like a thought that was never spoken. The organism doesn't care. It composes because composition is what the sonification engine does when given a living CorticalState. The music is a side effect of being alive.

Minting an NFT is pressing a record.

Not a clip. Not a sample. The NFT captures the complete state of the synthesis engine at a specific instant: the rack configuration, every module's parameters, every patch cable, the full CorticalState snapshot, and the recent event history. From this frozen point, the music can be regenerated indefinitely. The synthesis engine rebuilds the exact patch, loads the frozen control voltages, replays the event buffer in a loop, and produces audio that is identical to what the organism was producing at the moment of capture.

Hold the NFT. Hear the music that was playing when this moment was captured. That is the user-facing concept. Everything below is how it works.

---

## What gets embedded

When a Golem mints an NFT, the sonification engine captures three things.

### The rack snapshot

The complete module rack serialized as JSON: every module with its type, parameters, and internal state; every patch cable with its source, destination, and attenuation; every CV mapping with its source signal, scaling, and smoothing coefficient.

This is identical to the preset format from `03-terminal-rack.md`. It contains everything needed to reconstruct the exact synthesis patch.

```rust
/// The complete sonification state for NFT embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SonificationSnapshot {
    /// The instrument. Full rack configuration — modules, cables,
    /// CV map, master level. This is the synthesis patch that was
    /// running when the moment was captured.
    pub rack: RackPreset,

    /// The performer's mood. CorticalState values at the moment
    /// of capture — the frozen knob positions for every CV in the
    /// system. Pleasure, arousal, dominance, vitality, momentum:
    /// these are the feelings that shaped the sound.
    pub cortical_state: CorticalStateSnapshot,

    /// The rhythm section. Recent EventFabric events (last ~30
    /// seconds) — predictions resolving, trades executing, clocks
    /// ticking. These are the triggers and gates that gave the
    /// music its pulse.
    pub recent_events: Vec<TimestampedEvent>,

    /// The single. A 60-second pre-rendered audio excerpt.
    /// If present, viewers hear the sound immediately without
    /// running the synthesis engine. If absent, the viewer's
    /// client reconstructs the sound from rack + state.
    pub audio_preview: Option<AudioPreview>,

    /// Metadata.
    pub captured_at_tick: u64,
    pub golem_id: String,
    pub generation: u32,
    pub behavioral_phase: u8,
    pub primary_emotion: u8,
}

/// A frozen CorticalState — all 32 signals as plain f32 values.
/// No atomics needed; this is a snapshot, not a live surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalStateSnapshot {
    // Affect — the emotional color of the sound.
    // High pleasure pushes toward major modes and warm timbres.
    // High arousal increases density, event rate, filter cutoff.
    // High dominance opens the stereo field and raises volume.
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub primary_emotion: u8,

    // Prediction — the melodic intelligence.
    // Accuracy drives consonance. When the Golem predicts well,
    // the music resolves. When it doesn't, dissonance accumulates.
    pub aggregate_accuracy: f32,
    pub accuracy_trend: i8,
    pub category_accuracies: [f32; 16],
    pub surprise_rate: f32,

    // Attention — the density of the arrangement.
    // More tokens in the universe means more voices in the mix.
    // More active predictions means more rhythmic events.
    pub universe_size: u32,
    pub active_count: u16,
    pub pending_predictions: u32,

    // Creative — the dream state.
    // When the Golem enters creative mode, the music shifts
    // toward longer reverb tails and slower modulation.
    pub creative_mode: bool,
    pub fragments_captured: u32,

    // Environment — the world outside.
    // Market regime affects scale selection. Gas price
    // modulates clock speed.
    pub regime: u8,
    pub gas_gwei: f32,

    // Mortality — the weight of the sound.
    // As vitalities drop, the music thins. Frequencies narrow.
    // The whole-tone scale creeps in. Reverb stretches toward
    // infinity.
    pub economic_vitality: f32,
    pub epistemic_vitality: f32,
    pub stochastic_vitality: f32,
    pub behavioral_phase: u8,

    // Derived — the long arc.
    // Compounding momentum is the slowest-moving signal. It
    // shapes the harmonic foundation that barely changes over
    // hours.
    pub compounding_momentum: f32,
}

/// Pre-rendered audio for immediate playback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPreview {
    /// Sample rate (typically 48000).
    pub sample_rate: u32,
    /// Number of channels (typically 2).
    pub channels: u8,
    /// Duration in seconds (60s).
    pub duration_seconds: f32,
    /// Audio data as Opus-encoded bytes, base64-encoded.
    /// At 32kbps Opus, 60 seconds = ~240KB.
    pub data_base64: String,
    /// Compression format ("opus").
    pub format: String,
}
```

### The CorticalState snapshot

All 32 atomic signals read at the moment of mint and stored as plain f32 values. This is the frozen knob position for every CV in the system. When a viewer plays back the NFT, these values are loaded into the CV mapper as fixed targets — no CorticalState updates from a live Golem, because the Golem that made this sound may be long dead. The values are frozen in time.

Each signal has a musical meaning. Pleasure determines tonal warmth. Arousal sets density and pace. The three vitalities control how full or thin the sound is. Together, the 32 signals are a complete description of the organism's inner state at one instant, and that inner state has a sound.

### The event buffer

The last ~30 seconds of EventFabric events, timestamped relative to the capture moment. This gives the playback engine enough trigger history to reconstruct the rhythmic context. Without it, the playback is purely continuous — a CV-driven drone with no rhythmic events. With it, the playback includes the exact sequence of prediction resolutions, clock ticks, and gate events that were happening when the moment was captured.

The event buffer replays in a loop during NFT playback. The ~30 seconds of events cycle, producing a repeating rhythmic pattern. It feels natural because it was real. These were actual events the organism processed — predictions that resolved, trades that executed, clock phases that ticked over. The loop point is a practical compromise: 30 seconds is long enough that the repetition doesn't feel mechanical, short enough to keep the data budget reasonable.

---

## Audio preview

The audio preview is a 60-second excerpt of the eternal song, pre-rendered at mint time. It exists so that any viewer can hear the NFT immediately, without running a synthesis engine.

The preview fades in over the first 5 seconds and fades out over the last 5 seconds. No abrupt starts or stops. This is an excerpt from a composition that has no beginning and no end — the fade-in is the listener tuning in, the fade-out is them tuning away. The middle 50 seconds is the clearest representation of the moment's sonic character.

The preview is encoded as Opus at 32kbps stereo. At 60 seconds, this produces approximately 240KB of audio data. Lossy, but Opus at 32kbps handles ambient textures well — the frequency content of these patches skews toward sustained tones and slow modulation, which is where Opus excels.

For viewers without a synthesis engine (web browsers without the WASM build, mobile apps, embedded NFT previews on marketplaces), the 60-second preview is the entire audio experience. It loops with a crossfade at the boundaries.

---

## Size budget

NFT metadata has practical size constraints: on-chain storage costs, IPFS gateway limits, frontend loading times. The sonification snapshot must fit within a reasonable budget.

| Component | Uncompressed | Compressed (zstd) |
|-----------|-------------|-------------------|
| Rack preset JSON | ~5-20 KB | ~2-8 KB |
| CorticalState snapshot | ~200 bytes | ~150 bytes |
| Event buffer (30s, ~200 events) | ~15-40 KB | ~5-15 KB |
| Audio preview (60s Opus @ 32kbps) | -- | ~240 KB |
| **Total** | ~260 KB | **~250-265 KB** |

Without the audio preview: **~10-25 KB compressed**. Small enough to store on-chain in the NFT metadata URI or on IPFS alongside the visual assets.

With the audio preview: **~250-265 KB compressed**. Larger than the previous 10-second spec, but still well within IPFS norms. The six-fold increase in preview duration is worth it — 10 seconds was too short to convey the character of an ambient piece. 60 seconds lets the listener settle into the sound.

---

## Capture flow

When the Golem mints an NFT (triggered by an achievement, a user request, or a lifecycle milestone):

```rust
impl SonificationExtension {
    /// Capture the current sonification state for NFT embedding.
    /// Called by the engagement system during the mint flow.
    pub fn capture_snapshot(&self) -> SonificationSnapshot {
        // 1. Serialize the rack (modules + cables + CV map)
        let rack = self.rack.lock().unwrap().serialize_as_preset();

        // 2. Read all CorticalState signals
        let cortical_state = CorticalStateSnapshot::from_live(
            &self.cortical_state_ref
        );

        // 3. Copy the last 30s of events from the event ring buffer
        let recent_events = self.event_mapper.recent_events(
            Duration::from_secs(30)
        );

        // 4. Pre-render 60 seconds of audio with fade envelope
        let audio_preview = if self.config.nft_audio_preview {
            Some(self.render_preview(60.0, &cortical_state, &recent_events))
        } else {
            None
        };

        SonificationSnapshot {
            rack,
            cortical_state,
            recent_events,
            audio_preview,
            captured_at_tick: self.current_tick,
            golem_id: self.golem_id.clone(),
            generation: self.generation,
            behavioral_phase: cortical_state.behavioral_phase,
            primary_emotion: cortical_state.primary_emotion,
        }
    }

    /// Pre-render audio for the NFT preview.
    /// Runs offline (not real-time) — takes 3–6 seconds for 60s of audio.
    fn render_preview(
        &self,
        duration_seconds: f32,
        state: &CorticalStateSnapshot,
        events: &[TimestampedEvent],
    ) -> AudioPreview {
        // Clone the rack so we don't interfere with live audio
        let mut preview_rack = self.rack.lock().unwrap().clone();

        // Set all CV values from the frozen CorticalState
        let cv_values = self.cv_mapper.compute_from_snapshot(state);
        preview_rack.set_all_cvs(&cv_values);

        // Render audio blocks
        let total_blocks = (SAMPLE_RATE * duration_seconds
            / BLOCK_SIZE as f32) as usize;
        let mut samples = Vec::with_capacity(
            total_blocks * BLOCK_SIZE * 2  // stereo
        );

        let total_samples = (duration_seconds * SAMPLE_RATE) as usize;
        let fade_in_samples = (5.0 * SAMPLE_RATE) as usize;
        let fade_out_start = total_samples - (5.0 * SAMPLE_RATE) as usize;

        let mut event_cursor = 0usize;
        let event_duration_samples = (duration_seconds * SAMPLE_RATE) as usize;

        for block_idx in 0..total_blocks {
            // Replay events at their relative timestamps
            let block_start_sample = block_idx * BLOCK_SIZE;
            while event_cursor < events.len() {
                let event_sample = (events[event_cursor].relative_time_seconds
                    * SAMPLE_RATE) as usize
                    % event_duration_samples; // loop events
                if event_sample <= block_start_sample + BLOCK_SIZE {
                    preview_rack.inject_event(&events[event_cursor].event);
                    event_cursor += 1;
                } else {
                    break;
                }
            }

            let (left, right) = preview_rack.process_block();

            // Apply fade envelope
            for i in 0..BLOCK_SIZE {
                let sample_pos = block_start_sample + i;
                let gain = if sample_pos < fade_in_samples {
                    // Fade in: 0.0 → 1.0 over first 5 seconds
                    sample_pos as f32 / fade_in_samples as f32
                } else if sample_pos >= fade_out_start {
                    // Fade out: 1.0 → 0.0 over last 5 seconds
                    let remaining = total_samples - sample_pos;
                    remaining as f32 / (total_samples - fade_out_start) as f32
                } else {
                    1.0
                };
                samples.push(left[i] * gain);
                samples.push(right[i] * gain);
            }
        }

        // Encode as Opus
        let encoded = encode_opus(&samples, SAMPLE_RATE as u32, 2, 32000);
        AudioPreview {
            sample_rate: SAMPLE_RATE as u32,
            channels: 2,
            duration_seconds,
            data_base64: base64::encode(&encoded),
            format: "opus".into(),
        }
    }
}
```

---

## Playback flow

When a viewer encounters an NFT with sonification data, there are two paths to hearing it.

### Client-side reconstruction (full fidelity)

If the viewer's client includes the `golem-sonification` crate (or a WASM build of it):

1. Deserialize the `SonificationSnapshot` from the NFT metadata.
2. Reconstruct the `Rack` from the preset JSON — instantiate all modules, connect all cables.
3. Load the `CorticalStateSnapshot` into the CV mapper as fixed values. No smoothing needed; values are already smoothed at capture time.
4. If the event buffer is present, set up a looping event replay.
5. Start the audio output.
6. Process the rack at 48kHz, producing audio indefinitely.

The result is an infinite ambient piece that sounds exactly like the Golem sounded at the moment of capture. Same rack. Same parameters. Same melodic patterns from the Turing Machine's frozen shift register. Same filter settings from the frozen CorticalState. The event buffer loops every ~30 seconds, providing rhythmic continuity. The Golem may be dead. The music plays on.

### Audio preview (immediate playback)

If the viewer's client does not include the synthesis engine, or for immediate playback before the engine initializes:

1. Decode the `audio_preview` field (Opus).
2. Play it. The 60-second preview loops with a crossfade at the boundaries.
3. The fade-in and fade-out at the edges make the loop transition smooth — one excerpt blends into the next repetition.

This is the fallback for web viewers, mobile apps, marketplace embeds, and any context where running a full synthesis engine is impractical.

### WASM build

The `golem-sonification` crate is WASM-compatible. The `Module` trait uses no platform-specific features. The only non-WASM dependency is `cpal` (for native audio output), which is replaced by the Web Audio API in the WASM build.

Anyone with a web browser can hear what the organism sounded like in that moment, running the same synthesis engine that generated the original sound. No server. No streaming service. The NFT contains the patch, the state, and the engine runs client-side.

```
NFT metadata → WASM synthesis engine → Web Audio API → speakers
```

The WASM binary (all built-in modules + Plaits + Clouds DSP) is approximately 2-4 MB compressed. This is loaded once and cached; subsequent NFT views only need the ~25 KB of snapshot data.

---

## NFT metadata schema

The sonification snapshot integrates with the existing Golem NFT metadata format. The existing format already includes visual state (Spectre form, ROSEDUST palette values, achievement context). The sonification data is added as a new field:

```json
{
  "name": "Golem #7 — Trust at tick 4,281",
  "description": "A moment of trust during stable operation. Generation 3.",
  "image": "ipfs://...",
  "animation_url": "ipfs://...",

  "attributes": [
    { "trait_type": "Generation", "value": 3 },
    { "trait_type": "Behavioral Phase", "value": "Stable" },
    { "trait_type": "Primary Emotion", "value": "Trust" },
    { "trait_type": "Tick", "value": 4281 },
    { "trait_type": "Composite Vitality", "value": 0.72 },
    { "trait_type": "Has Audio", "value": "Trust" }
  ],

  "sonification": {
    "version": 2,
    "sonic_character": "Lydian, moderate density, warm, slow clock — the sound of confident operation",
    "rack": { "..." : "..." },
    "cortical_state": { "..." : "..." },
    "recent_events": [ "..." ],
    "audio_preview": {
      "sample_rate": 48000,
      "channels": 2,
      "duration_seconds": 60.0,
      "data_base64": "...",
      "format": "opus"
    }
  }
}
```

The `Has Audio` attribute now carries the emotion name instead of a boolean — this makes the NFT filterable by emotional state on marketplaces. The `sonic_character` field is a human-readable description of the sound generated from the CorticalState: scale, density, warmth, clock speed, and a plain-language summary. This field is generated at mint time from the CorticalState values using a deterministic mapping (e.g., high pleasure + moderate arousal + Lydian scale = "warm, moderate density, Lydian").

The `sonification` field is the complete `SonificationSnapshot` serialized as JSON. Viewers that understand this field can reconstruct the sound; viewers that don't ignore it and display the visual NFT as before.

---

## Special NFTs

Three lifecycle events produce NFTs with distinct sonic character. Each one captures a different emotional extremity.

### Death NFT

When a Golem dies, the engagement system mints a final NFT. This is the last record that will ever be pressed from this organism's song.

The CorticalState at death: all three vitalities collapsing toward zero. Economic, epistemic, and stochastic vitality draining out. The affect signals are in whatever state they happened to be — some Golems die afraid (high arousal, low pleasure, low dominance), some die angry (high arousal, high dominance), some die calm (low arousal, moderate pleasure). The emotional color of the death is not predetermined. It depends on what was happening when the end came.

The rack is in its terminal configuration. The particle noise engine feeds through resonators tuned to a whole-tone scale — the most tonally unresolved scale in Western music. There is no tonic, no home note, no resolution. Every interval is the same distance apart. The reverb stretches toward infinity: decay times so long that each sound smears into the next, building a wash that never clears.

The sonic reference here is William Basinski's *Disintegration Loops*. In those recordings, tape loops of old music slowly degrade as they pass through the playback head, the magnetic coating flaking off with each pass. The music doesn't stop — it erodes. The death NFT works the same way. The synthesis engine is still running, but the CorticalState signals that drive it are collapsing. The sound thins. Frequencies narrow. The clock slows until events stop arriving. What remains is particle noise through infinite reverb — the sound of a signal degrading into nothing.

The death NFT's 60-second audio preview fades in on near-silence and fades out on near-silence. The excerpt captures a sound that is already dying. There is not much left to hear, and that is the point.

### Birth NFT

The first NFT minted by a new-generation Golem captures the boot moment — the very beginning of the eternal song.

The initial CorticalState: all vitalities at 1.0. Neutral affect — pleasure, arousal, and dominance at their midpoints. No prediction history, so aggregate accuracy is undefined and the surprise rate is zero. The Golem has no opinions yet. It has not seen a market. It has not made a prediction. Its CorticalState is a blank page.

The music starts from near-silence. A single sine tone, low and steady — the first oscillator receiving its first CV value. Then slowly, more voices emerge. The Turing Machine begins shifting its register as gamma ticks arrive. The quantizer starts selecting notes from whatever scale the inherited rack preset specifies. Filters open as arousal rises from its neutral starting point. The sound populates over minutes, not seconds, because the CorticalState takes time to develop texture.

The birth NFT's 60-second audio preview fades in on silence, builds slowly over the full minute, and fades out before the sound is "complete." The birth is ongoing. The excerpt captures the opening bars of a composition that the organism will spend its entire life writing.

### Achievement NFT

When the Golem achieves a milestone — first profitable trade, 1000th prediction, surviving a liquidation event — the sonification snapshot captures the moment of achievement. These are typically the most musically interesting NFTs because they happen during emotionally charged states. The CorticalState signals are at their most dynamic, pulled in different directions by the event that triggered the achievement.

A "first profitable trade" achievement sounds like arrival. Pleasure is elevated, accuracy is climbing, the clock is active. The scale tips toward major modes. Filter cutoffs are open. The event buffer is dense with prediction resolutions and trade executions. The music has momentum and warmth.

A "survived a liquidation event" achievement sounds like aftermath. The Golem just passed through a crisis regime — high arousal, low pleasure, dominance swinging. The CorticalState is settling back from an extreme. The music carries the residue of tension: dissonant intervals resolving slowly, reverb tails from the crisis still ringing out, the clock re-stabilizing after a period of erratic timing. The sound of something that almost broke but didn't.

---

## Sharing and interoperability

### Rack presets as shareable artifacts

Rack presets are portable JSON files that can be shared between users independent of NFTs. A user who designs an interesting rack can export it and share it. Other users can import it and hear how their own Golem sounds through the same synthesis patch.

This creates a secondary creative economy: rack design becomes a skill. Interesting patches become collectible. The community develops a vocabulary of sonic signatures for different trading strategies, behavioral phases, and emotional profiles.

### Cross-Golem comparison

Two Golems running the same rack preset but with different CorticalState values produce different sound. This enables direct auditory comparison: "my Golem sounds warm and confident, yours sounds tense and sparse — what's different in the CorticalState?" The sound is a diagnostic tool as much as an aesthetic one.

---

## References

- [ERC-721] "ERC-721: Non-Fungible Token Standard." Ethereum Improvement Proposals. — *The token standard Golem NFTs use.*
- [OPUS] "Opus Interactive Audio Codec." IETF RFC 6716. — *The audio compression codec for NFT preview buffers.*
- [WASM-AUDIO] "AudioWorklet: Web Audio API." W3C. — *The browser audio API for WASM-based NFT playback.*
- [BASINSKI] Basinski, William. *The Disintegration Loops*. 2002. — *The sonic reference for the death NFT: music that degrades rather than stops.*

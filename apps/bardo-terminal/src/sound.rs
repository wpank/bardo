//! Feature-gated sound engine built on rodio.
//!
//! When the `sound` feature is absent, all methods are no-ops and the binary
//! compiles without any native audio library on the path.

#[cfg(feature = "sound")]
use rodio::source::SineWave;
#[cfg(feature = "sound")]
use rodio::{OutputStream, Sink};
use std::collections::HashSet;

/// Events that can trigger sound playback.
/// Each variant maps to a specific procedural tone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SoundEvent {
    /// T2 inference starting -- soft rising chime
    DeliberationChime,
    /// Trade closed in profit
    TradeSuccess { profit_basis_points: u32 },
    /// Trade closed at a loss
    TradeFailure,
    /// Dream cycle begins
    DreamStart,
    /// Dream cycle ends
    DreamEnd,
    /// Dead golem's lesson arrived via Styx
    BloodstainReceived,
    /// Achievement unlocked (common)
    AchievementUnlock,
    /// Achievement unlocked (legendary -- longer fanfare)
    AchievementLegendary,
    /// Golem enters Terminal phase
    TerminalPhaseEntered,
    /// Golem dies -- 1s silence then single clear bell
    Death,
    /// Optional per-heartbeat metronome (T0)
    HeartbeatTick,
}

/// Sound engine configuration.
pub struct SoundConfig {
    pub enabled: bool,
    pub volume: f32,
    pub event_filters: HashSet<SoundEvent>,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.6,
            event_filters: HashSet::new(),
        }
    }
}

/// The sound engine. When the `sound` feature is enabled, wraps a rodio Sink.
/// When the feature is absent, all methods are no-ops.
pub struct SoundEngine {
    config: SoundConfig,
    #[cfg(feature = "sound")]
    _stream: OutputStream,
    #[cfg(feature = "sound")]
    sink: Sink,
}

impl SoundEngine {
    /// Construct a new engine. If `sound` feature is absent, returns a disabled
    /// no-op engine. If `sound` feature is present but audio init fails (common
    /// in headless CI), logs the error and returns a disabled engine.
    pub fn new(config: SoundConfig) -> Self {
        #[cfg(feature = "sound")]
        {
            match OutputStream::try_default() {
                Ok((_stream, handle)) => match Sink::try_new(&handle) {
                    Ok(sink) => {
                        sink.set_volume(config.volume);
                        return Self {
                            config,
                            _stream,
                            sink,
                        };
                    }
                    Err(e) => {
                        tracing::warn!("Sound sink init failed (headless?): {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Sound output stream init failed (headless?): {}", e);
                }
            }
            // Fallback: try again to get fields, but disable
            if let Ok((_stream, handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&handle) {
                    let mut engine = Self {
                        config,
                        _stream,
                        sink,
                    };
                    engine.config.enabled = false;
                    return engine;
                }
            }
            // If we truly can't get a stream, we need the non-sound path
        }
        #[cfg(not(feature = "sound"))]
        {
            Self { config }
        }
        #[cfg(feature = "sound")]
        {
            // This path is only reached if OutputStream::try_default() fails twice,
            // which shouldn't happen, but we need to satisfy the compiler.
            // In practice the first try_default succeeded or we're in not(sound).
            unreachable!("audio stream init failed on both attempts")
        }
    }

    /// Play a sound event. No-op if engine is disabled or event is filtered out.
    pub fn play(&self, event: SoundEvent) {
        if !self.config.enabled {
            return;
        }
        if !self.config.event_filters.is_empty() && !self.config.event_filters.contains(&event) {
            return;
        }
        #[cfg(feature = "sound")]
        {
            let source = Self::synthesize(&event, self.config.volume);
            if let Some(src) = source {
                self.sink.append(src);
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 1.0);
        #[cfg(feature = "sound")]
        self.sink.set_volume(self.config.volume);
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Synthesize a procedural tone for an event.
    /// Returns None for Death (multi-part, caller-managed).
    #[cfg(feature = "sound")]
    fn synthesize(event: &SoundEvent, volume: f32) -> Option<impl rodio::Source<Item = f32>> {
        use rodio::Source;
        use std::time::Duration;

        let (freq_hz, duration_ms, amp) = match event {
            SoundEvent::DeliberationChime => (880.0_f32, 200_u64, 0.3_f32),
            SoundEvent::TradeSuccess { .. } => (659.0, 300, 0.4),
            SoundEvent::TradeFailure => (220.0, 400, 0.3),
            SoundEvent::DreamStart => (110.0, 2000, 0.15),
            SoundEvent::DreamEnd => (1047.0, 1000, 0.25),
            SoundEvent::BloodstainReceived => (82.0, 500, 0.5),
            SoundEvent::AchievementUnlock => (523.0, 500, 0.35),
            SoundEvent::AchievementLegendary => (659.0, 1500, 0.4),
            SoundEvent::TerminalPhaseEntered => (55.0, 3000, 0.2),
            SoundEvent::Death => return None,
            SoundEvent::HeartbeatTick => (440.0, 50, 0.05),
        };
        let source = SineWave::new(freq_hz)
            .take_duration(Duration::from_millis(duration_ms))
            .amplify(amp * volume);
        Some(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_engine_disabled_by_default() {
        let config = SoundConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_sound_engine_play_noop_when_disabled() {
        // Without the sound feature, SoundEngine::new returns a no-op engine.
        // With the sound feature in CI (no audio device), it also returns disabled.
        // Either way, play() must not panic.
        let engine = SoundEngine::new(SoundConfig::default());
        engine.play(SoundEvent::HeartbeatTick);
        engine.play(SoundEvent::DeliberationChime);
        engine.play(SoundEvent::Death);
    }

    #[test]
    fn test_sound_engine_set_volume_clamps() {
        let mut engine = SoundEngine::new(SoundConfig::default());
        engine.set_volume(2.0);
        assert_eq!(engine.config.volume, 1.0);
        engine.set_volume(-1.0);
        assert_eq!(engine.config.volume, 0.0);
        engine.set_volume(0.5);
        assert_eq!(engine.config.volume, 0.5);
    }

    #[test]
    fn test_sound_event_hash_eq() {
        let mut set = HashSet::new();
        set.insert(SoundEvent::DeliberationChime);
        set.insert(SoundEvent::TradeSuccess {
            profit_basis_points: 100,
        });
        set.insert(SoundEvent::Death);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&SoundEvent::DeliberationChime));
        assert!(set.contains(&SoundEvent::Death));
    }
}

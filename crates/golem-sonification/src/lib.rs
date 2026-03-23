//! Sonification engine: modular synthesis driven by `CorticalState`.
//!
//! Produces real-time audio output through a rack of interconnected modules,
//! with parameters driven by the golem's cortical signals.
//!
//! ## Threading model
//!
//! - **Thread 1**: Golem runtime (tokio) — calls Extension hooks
//! - **Thread 2**: Parameter updater at 120Hz — reads `CorticalState`, writes `AtomicParameterBridge`
//! - **Thread 3**: Rack processor at audio rate — reads bridge, runs `rack.process_block()`, writes ring buffer
//! - **Thread 4**: cpal callback — reads ring buffer, writes to OS audio (must never allocate)

pub mod audio_out;
pub mod cv_mapper;
pub mod event_mapper;
pub mod modules;
pub mod nft;
pub mod params;
pub mod preset;
pub mod rack;
pub mod tui;

pub use modules::{
    BLOCK_SIZE, Module, PatchCable, PortDeclaration, PortDirection, PortId, SAMPLE_RATE,
    SignalBlock, SignalType,
};
pub use params::{AtomicParameterBridge, ParamDeclaration};
pub use rack::Rack;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use golem_core::cortical::CorticalState;
use golem_core::extension::{EndCtx, Extension, SessionCtx, SessionReason};
use tracing::{info, warn};

use audio_out::AudioOutput;
use cv_mapper::CvMapper;
use modules::{mixer::Mixer, noise::NoiseSource, vca::Vca};

/// Sonification extension: boots the audio engine, wires a default patch,
/// and bridges cortical state into audio parameters.
pub struct SonificationExtension {
    cortical: Arc<CorticalState>,
    bridge: Arc<AtomicParameterBridge>,
    rack: Arc<Mutex<Rack>>,
    shutdown: Arc<AtomicBool>,
    threads_started: Mutex<bool>,
}

impl SonificationExtension {
    /// Create a new sonification extension bound to the given cortical state.
    pub fn new(cortical: Arc<CorticalState>) -> Self {
        Self {
            cortical,
            bridge: Arc::new(AtomicParameterBridge::new()),
            rack: Arc::new(Mutex::new(Rack::new())),
            shutdown: Arc::new(AtomicBool::new(false)),
            threads_started: Mutex::new(false),
        }
    }

    /// Boot the audio engine: wire default patch, start threads 2-4.
    fn boot(&self) -> anyhow::Result<()> {
        {
            let mut started = self
                .threads_started
                .lock()
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if *started {
                return Ok(());
            }
            *started = true;
        }

        // Wire default patch: NoiseSource → VCA → Mixer → AudioOutput
        {
            let mut rack = self.rack.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            rack.add_module(Box::new(NoiseSource::new("noise_1")));
            rack.add_module(Box::new(Vca::new("vca_1")));
            rack.add_module(Box::new(Mixer::new("mixer_1")));

            rack.connect(
                PortId {
                    module_id: "noise_1".into(),
                    port_name: "out".into(),
                },
                PortId {
                    module_id: "vca_1".into(),
                    port_name: "in".into(),
                },
                1.0,
            );
            rack.connect(
                PortId {
                    module_id: "vca_1".into(),
                    port_name: "out".into(),
                },
                PortId {
                    module_id: "mixer_1".into(),
                    port_name: "in_1".into(),
                },
                1.0,
            );
        }

        // Thread 2: Parameter updater at 120Hz
        let bridge_t2 = self.bridge.clone();
        let cortical_t2 = self.cortical.clone();
        let shutdown_t2 = self.shutdown.clone();
        thread::Builder::new()
            .name("sono-param-updater".into())
            .spawn(move || {
                let interval = Duration::from_micros(8_333); // ~120Hz
                while !shutdown_t2.load(Ordering::Acquire) {
                    let snapshot = cortical_t2.snapshot();
                    CvMapper::update_from_snapshot(&snapshot, &bridge_t2);
                    thread::sleep(interval);
                }
            })
            .map_err(|e| anyhow::anyhow!("failed to spawn param updater: {e}"))?;

        // Thread 3: Rack processor + audio output
        // AudioOutput (cpal::Stream) is !Send, so it must be created on
        // the thread that will use it.
        let rack_t3 = self.rack.clone();
        let bridge_t3 = self.bridge.clone();
        let shutdown_t3 = self.shutdown.clone();
        thread::Builder::new()
            .name("sono-rack-proc".into())
            .spawn(move || {
                // Open audio on this thread (Thread 4 / cpal callback starts here)
                let mut audio = match AudioOutput::open() {
                    Ok(a) => Some(a),
                    Err(e) => {
                        warn!("sonification: audio output unavailable: {e}");
                        None
                    }
                };

                // Block duration at 48kHz with BLOCK_SIZE=32: ~667µs
                let block_interval = Duration::from_micros(667);

                while !shutdown_t3.load(Ordering::Acquire) {
                    // Copy bridge values into rack
                    let master = bridge_t3.read(params::cv_index::MASTER_LEVEL);
                    let (left, right) = {
                        let mut rack = match rack_t3.lock() {
                            Ok(r) => r,
                            Err(_) => continue,
                        };
                        rack.master_level = master;
                        // Feed master_level as VCA cv input
                        let cv_block = [master; BLOCK_SIZE];
                        rack.buffers.insert(
                            PortId {
                                module_id: "vca_1".into(),
                                port_name: "cv".into(),
                            },
                            cv_block,
                        );
                        rack.process_block()
                    };

                    if let Some(ref mut audio_out) = audio {
                        audio_out.write_block(&left, &right);
                    }

                    thread::sleep(block_interval);
                }

                // Graceful shutdown
                if let Some(audio_out) = audio {
                    audio_out.stop();
                }
            })
            .map_err(|e| anyhow::anyhow!("failed to spawn rack processor: {e}"))?;

        info!("sonification: boot complete, default patch wired");
        Ok(())
    }
}

#[async_trait]
impl Extension for SonificationExtension {
    fn name(&self) -> &str {
        "sonification"
    }

    fn layer(&self) -> u8 {
        6
    }

    async fn on_session(&self, reason: SessionReason, _ctx: &mut SessionCtx) -> anyhow::Result<()> {
        if matches!(reason, SessionReason::Start) {
            self.boot()?;
        }
        Ok(())
    }

    async fn on_end(&self, _ctx: &EndCtx) -> anyhow::Result<()> {
        self.shutdown.store(true, Ordering::Release);
        // Give threads time to wind down
        tokio::time::sleep(Duration::from_millis(50)).await;
        info!("sonification: shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sonification_extension_on_boot_smoke() {
        let cortical = CorticalState::new();
        let ext = SonificationExtension::new(cortical);

        // Boot may fail if no audio device (CI), but should not panic
        let result = ext.boot();
        match &result {
            Ok(()) => {
                // Shut down cleanly
                ext.shutdown.store(true, Ordering::Release);
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                // Acceptable on headless CI
                eprintln!("boot failed (expected on CI): {e}");
            }
        }
    }
}

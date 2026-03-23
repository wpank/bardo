//! Audio output via cpal + lock-free SPSC ring buffer.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};
use thiserror::Error;
use tracing::warn;

use crate::modules::BLOCK_SIZE;

/// Ring buffer capacity in samples (interleaved stereo).
/// Two 128-sample cpal periods = 256 frames = 512 stereo samples.
const RING_BUFFER_CAPACITY: usize = 512;

/// Errors from audio output initialization.
#[derive(Debug, Error)]
pub enum AudioError {
    /// No audio output device found.
    #[error("no audio output device found")]
    NoDevice,
    /// Failed to query supported stream configs.
    #[error("failed to get supported configs: {0}")]
    ConfigError(String),
    /// Failed to build or start the audio stream.
    #[error("failed to build stream: {0}")]
    StreamError(String),
}

/// Handle to the cpal audio output stream and the producer side of the ring buffer.
pub struct AudioOutput {
    _stream: cpal::Stream,
    producer: ringbuf::HeapProd<f32>,
    running: Arc<AtomicBool>,
}

impl AudioOutput {
    /// Open the default audio output device at 48kHz stereo f32.
    ///
    /// Returns the `AudioOutput` handle. Write interleaved stereo samples
    /// via `write_block()`.
    pub fn open() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(AudioError::NoDevice)?;

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(48_000),
            buffer_size: cpal::BufferSize::Fixed(128),
        };

        let rb = HeapRb::<f32>::new(RING_BUFFER_CAPACITY);
        let (producer, mut consumer) = rb.split();

        let running = Arc::new(AtomicBool::new(true));
        let running_cb = running.clone();

        // Thread 4 callback: reads from ring buffer, writes to OS audio.
        // MUST NOT allocate, lock, or syscall.
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    audio_callback(&mut consumer, data, &running_cb);
                },
                move |err| {
                    warn!("cpal stream error: {err}");
                },
                None,
            )
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            producer,
            running,
        })
    }

    /// Write a stereo block to the ring buffer.
    ///
    /// Interleaves left and right channels. If the ring buffer is full,
    /// excess samples are silently dropped (preferable to blocking).
    pub fn write_block(&mut self, left: &[f32; BLOCK_SIZE], right: &[f32; BLOCK_SIZE]) {
        for i in 0..BLOCK_SIZE {
            // Best-effort push; drop on overflow rather than blocking
            let _ = self.producer.try_push(left[i]);
            let _ = self.producer.try_push(right[i]);
        }
    }

    /// Signal the stream to stop and drop the cpal stream handle.
    pub fn stop(self) {
        self.running.store(false, Ordering::Release);
        // Stream is stopped on drop via cpal's Drop impl
    }
}

/// The callback body shared between cpal and tests.
///
/// Reads interleaved stereo samples from the consumer. Fills any shortfall
/// with silence (0.0). This function is `#[inline]` so it can be called
/// directly in allocation-free tests.
#[inline]
pub fn audio_callback(
    consumer: &mut ringbuf::HeapCons<f32>,
    data: &mut [f32],
    _running: &AtomicBool,
) {
    // Read available samples from ring buffer
    let available = consumer.occupied_len();
    let to_read = data.len().min(available);

    // Pop samples one at a time (no allocation, fixed stack ops)
    for sample in data.iter_mut().take(to_read) {
        *sample = consumer.try_pop().unwrap_or(0.0);
    }

    // Fill remainder with silence
    for sample in data.iter_mut().skip(to_read) {
        *sample = 0.0;
    }
}

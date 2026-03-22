//! Low-frequency oscillator module.

use std::collections::HashMap;

use serde_json::json;

use super::{BLOCK_SIZE, Module, PortDeclaration, PortDirection, SAMPLE_RATE, SignalBlock, SignalType};
use crate::params::ParamDeclaration;

/// LFO waveform shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LfoShape {
    /// Sine wave.
    Sine = 0,
    /// Triangle wave.
    Triangle = 1,
    /// Sawtooth (rising ramp).
    Saw = 2,
    /// Square wave.
    Square = 3,
    /// Random sample-and-hold.
    RandomSH = 4,
}

impl LfoShape {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Sine,
            1 => Self::Triangle,
            2 => Self::Saw,
            3 => Self::Square,
            _ => Self::RandomSH,
        }
    }
}

/// Low-frequency oscillator (0.01-20 Hz).
///
/// Produces CV-rate output driven by a phase accumulator.
/// Supports sync input (gate) for phase reset.
pub struct Lfo {
    id: String,
    phase: f32,
    frequency: f32,
    shape: u8,
    sh_value: f32,
    prev_sync: f32,
    ports: Vec<PortDeclaration>,
}

impl Lfo {
    /// Creates a new LFO with the given instance id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            phase: 0.0,
            frequency: 1.0,
            shape: 0,
            sh_value: 0.0,
            prev_sync: 0.0,
            ports: vec![
                PortDeclaration {
                    name: "rate".into(),
                    signal_type: SignalType::Cv,
                    direction: PortDirection::Input,
                    description: "Frequency modulation CV".into(),
                    default_value: 0.0,
                },
                PortDeclaration {
                    name: "sync".into(),
                    signal_type: SignalType::Gate,
                    direction: PortDirection::Input,
                    description: "Phase reset on rising edge".into(),
                    default_value: 0.0,
                },
                PortDeclaration {
                    name: "out".into(),
                    signal_type: SignalType::Cv,
                    direction: PortDirection::Output,
                    description: "LFO output (0.0-1.0 unipolar)".into(),
                    default_value: 0.0,
                },
            ],
        }
    }

    fn sample_shape(&self, phase: f32) -> f32 {
        match LfoShape::from_u8(self.shape) {
            LfoShape::Sine => {
                (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5
            }
            LfoShape::Triangle => {
                if phase < 0.5 {
                    phase * 2.0
                } else {
                    2.0 - phase * 2.0
                }
            }
            LfoShape::Saw => phase,
            LfoShape::Square => {
                if phase < 0.5 { 1.0 } else { 0.0 }
            }
            LfoShape::RandomSH => self.sh_value,
        }
    }
}

impl Module for Lfo {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        "LFO"
    }

    fn ports(&self) -> &[PortDeclaration] {
        &self.ports
    }

    fn process(
        &mut self,
        inputs: &HashMap<String, SignalBlock>,
        outputs: &mut HashMap<String, SignalBlock>,
    ) {
        let rate_cv = inputs.get("rate");
        let sync = inputs.get("sync");
        let mut out = [0.0_f32; BLOCK_SIZE];
        let phase_inc_base = self.frequency / SAMPLE_RATE;

        for i in 0..BLOCK_SIZE {
            // Check sync rising edge
            let sync_val = sync.map_or(0.0, |s| s[i]);
            if sync_val > 0.5 && self.prev_sync <= 0.5 {
                self.phase = 0.0;
            }
            self.prev_sync = sync_val;

            // Apply rate CV modulation (adds to base frequency)
            let rate_mod = rate_cv.map_or(0.0, |r| r[i]);
            let freq = (self.frequency + rate_mod * 10.0).clamp(0.01, 20.0);
            let phase_inc = freq / SAMPLE_RATE;

            // Advance phase
            let old_phase = self.phase;
            self.phase += phase_inc;

            // Wrap phase and update S&H on wraparound
            if self.phase >= 1.0 {
                self.phase -= 1.0;
                // New random value for S&H on each cycle
                // Use a simple deterministic hash since we can't easily use rand here
                // without storing an Rng (which we do in NoiseSource)
                self.sh_value = ((old_phase * 12345.6789).sin().abs()) % 1.0;
            }

            out[i] = self.sample_shape(self.phase);
        }

        outputs.insert("out".into(), out);
    }

    fn set_param(&mut self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency = value.clamp(0.01, 20.0),
            "shape" => self.shape = (value as u8).min(4),
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<f32> {
        match name {
            "frequency" => Some(self.frequency),
            "shape" => Some(f32::from(self.shape)),
            _ => None,
        }
    }

    fn params(&self) -> Vec<ParamDeclaration> {
        vec![
            ParamDeclaration {
                name: "frequency".into(),
                display_name: "Frequency".into(),
                value: self.frequency,
                min: 0.01,
                max: 20.0,
                default: 1.0,
                unit: "Hz".into(),
                description: "Oscillation rate".into(),
            },
            ParamDeclaration {
                name: "shape".into(),
                display_name: "Shape".into(),
                value: f32::from(self.shape),
                min: 0.0,
                max: 4.0,
                default: 0.0,
                unit: "".into(),
                description: "Waveform: 0=sine, 1=tri, 2=saw, 3=square, 4=S&H".into(),
            },
        ]
    }

    fn serialize_state(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "phase": self.phase,
            "frequency": self.frequency,
            "shape": self.shape,
            "sh_value": self.sh_value,
        })
    }

    fn deserialize_state(&mut self, state: &serde_json::Value) {
        if let Some(v) = state.get("phase").and_then(|v| v.as_f64()) {
            self.phase = v as f32;
        }
        if let Some(v) = state.get("frequency").and_then(|v| v.as_f64()) {
            self.frequency = v as f32;
        }
        if let Some(v) = state.get("shape").and_then(|v| v.as_u64()) {
            self.shape = v as u8;
        }
        if let Some(v) = state.get("sh_value").and_then(|v| v.as_f64()) {
            self.sh_value = v as f32;
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.frequency = 1.0;
        self.shape = 0;
        self.sh_value = 0.0;
        self.prev_sync = 0.0;
    }
}

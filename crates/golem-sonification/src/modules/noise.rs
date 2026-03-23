//! Noise source module: white, pink, and dust modes.

use std::collections::HashMap;

use rand::{Rng, SeedableRng, rngs::SmallRng};
use serde_json::json;

use super::{BLOCK_SIZE, Module, PortDeclaration, PortDirection, SignalBlock, SignalType};
use crate::params::ParamDeclaration;

/// Noise generation mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NoiseMode {
    /// Uniform white noise.
    White = 0,
    /// Pink (1/f) noise via Paul Kellet approximation.
    Pink = 1,
    /// Sparse random clicks with CV-controlled density.
    Dust = 2,
}

/// Noise source module.
///
/// Generates three flavours of noise. Dust mode density is CV-controllable
/// for rhythmic event generation.
pub struct NoiseSource {
    id: String,
    mode: u8,
    rng: SmallRng,
    // Pink noise filter state (Paul Kellet approximation)
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    ports: Vec<PortDeclaration>,
}

impl NoiseSource {
    /// Creates a new noise source with the given instance id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            mode: 0,
            rng: SmallRng::seed_from_u64(0xDEAD_BEEF),
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            ports: vec![
                PortDeclaration {
                    name: "density".into(),
                    signal_type: SignalType::Cv,
                    direction: PortDirection::Input,
                    description: "Click density for dust mode (0.0-1.0)".into(),
                    default_value: 0.5,
                },
                PortDeclaration {
                    name: "out".into(),
                    signal_type: SignalType::Audio,
                    direction: PortDirection::Output,
                    description: "Noise output".into(),
                    default_value: 0.0,
                },
            ],
        }
    }

    fn white_sample(&mut self) -> f32 {
        self.rng.r#gen::<f32>() * 2.0 - 1.0
    }

    fn pink_sample(&mut self) -> f32 {
        let white = self.white_sample();
        self.b0 = 0.99886 * self.b0 + white * 0.0555179;
        self.b1 = 0.99332 * self.b1 + white * 0.0750759;
        self.b2 = 0.96900 * self.b2 + white * 0.1538520;
        self.b3 = 0.86650 * self.b3 + white * 0.3104856;
        self.b4 = 0.55000 * self.b4 + white * 0.5329522;
        self.b5 = -0.7616 * self.b5 - white * 0.0168980;
        let pink =
            self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362;
        self.b6 = white * 0.115926;
        // Normalize: the sum has roughly +-4.5 range
        (pink * 0.11).clamp(-1.0, 1.0)
    }

    fn dust_sample(&mut self, density: f32) -> f32 {
        let threshold = density.clamp(0.0, 1.0);
        if self.rng.r#gen::<f32>() < threshold * 0.01 {
            self.rng.r#gen::<f32>() * 2.0 - 1.0
        } else {
            0.0
        }
    }
}

impl Module for NoiseSource {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        "Noise"
    }

    fn ports(&self) -> &[PortDeclaration] {
        &self.ports
    }

    fn process(
        &mut self,
        inputs: &HashMap<String, SignalBlock>,
        outputs: &mut HashMap<String, SignalBlock>,
    ) {
        let density_cv = inputs.get("density");
        let mut out = [0.0_f32; BLOCK_SIZE];

        for i in 0..BLOCK_SIZE {
            out[i] = match self.mode {
                0 => self.white_sample(),
                1 => self.pink_sample(),
                _ => {
                    let d = density_cv.map_or(0.5, |cv| cv[i]);
                    self.dust_sample(d)
                }
            };
        }

        outputs.insert("out".into(), out);
    }

    fn set_param(&mut self, name: &str, value: f32) {
        if name == "mode" {
            self.mode = (value as u8).min(2);
        }
    }

    fn get_param(&self, name: &str) -> Option<f32> {
        if name == "mode" {
            Some(f32::from(self.mode))
        } else {
            None
        }
    }

    fn params(&self) -> Vec<ParamDeclaration> {
        vec![ParamDeclaration {
            name: "mode".into(),
            display_name: "Mode".into(),
            value: f32::from(self.mode),
            min: 0.0,
            max: 2.0,
            default: 0.0,
            unit: "".into(),
            description: "0=white, 1=pink, 2=dust".into(),
        }]
    }

    fn serialize_state(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "mode": self.mode,
        })
    }

    fn deserialize_state(&mut self, state: &serde_json::Value) {
        if let Some(v) = state.get("mode").and_then(|v| v.as_u64()) {
            self.mode = (v as u8).min(2);
        }
    }

    fn reset(&mut self) {
        self.mode = 0;
        self.b0 = 0.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.b3 = 0.0;
        self.b4 = 0.0;
        self.b5 = 0.0;
        self.b6 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_white_no_panic() {
        let mut noise = NoiseSource::new("test_noise");
        noise.set_param("mode", 0.0); // white
        let inputs = HashMap::new();
        let mut outputs = HashMap::new();
        noise.process(&inputs, &mut outputs);

        let out = outputs
            .get("out")
            .expect("NoiseSource should produce 'out'");
        // White noise should have non-zero output
        let max_abs = out.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max_abs > 0.0, "White noise should produce non-zero output");
    }

    #[test]
    fn test_noise_pink_no_panic() {
        let mut noise = NoiseSource::new("test_noise");
        noise.set_param("mode", 1.0); // pink
        let inputs = HashMap::new();
        let mut outputs = HashMap::new();
        noise.process(&inputs, &mut outputs);
        assert!(outputs.contains_key("out"));
    }

    #[test]
    fn test_noise_dust_no_panic() {
        let mut noise = NoiseSource::new("test_noise");
        noise.set_param("mode", 2.0); // dust
        let inputs = HashMap::new();
        let mut outputs = HashMap::new();
        noise.process(&inputs, &mut outputs);
        assert!(outputs.contains_key("out"));
    }
}

//! Voltage-controlled amplifier module.

use std::collections::HashMap;

use serde_json::json;

use super::{BLOCK_SIZE, Module, PortDeclaration, PortDirection, SignalBlock, SignalType};
use crate::params::ParamDeclaration;

/// Voltage-controlled amplifier.
///
/// Multiplies the audio input by the CV input to produce amplitude modulation.
/// When no CV is connected, the input passes through at unity gain.
pub struct Vca {
    id: String,
    ports: Vec<PortDeclaration>,
}

impl Vca {
    /// Creates a new VCA with the given instance id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ports: vec![
                PortDeclaration {
                    name: "in".into(),
                    signal_type: SignalType::Audio,
                    direction: PortDirection::Input,
                    description: "Audio input".into(),
                    default_value: 0.0,
                },
                PortDeclaration {
                    name: "cv".into(),
                    signal_type: SignalType::Cv,
                    direction: PortDirection::Input,
                    description: "Amplitude control voltage".into(),
                    default_value: 1.0,
                },
                PortDeclaration {
                    name: "out".into(),
                    signal_type: SignalType::Audio,
                    direction: PortDirection::Output,
                    description: "Audio output".into(),
                    default_value: 0.0,
                },
            ],
        }
    }
}

impl Module for Vca {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        "VCA"
    }

    fn ports(&self) -> &[PortDeclaration] {
        &self.ports
    }

    fn process(
        &mut self,
        inputs: &HashMap<String, SignalBlock>,
        outputs: &mut HashMap<String, SignalBlock>,
    ) {
        let mut out = [0.0_f32; BLOCK_SIZE];

        // Default CV is 1.0 (unity) when unconnected
        let default_cv = [1.0_f32; BLOCK_SIZE];
        let audio_in = inputs.get("in");
        let cv = inputs.get("cv").unwrap_or(&default_cv);

        if let Some(audio) = audio_in {
            for i in 0..BLOCK_SIZE {
                out[i] = audio[i] * cv[i].clamp(0.0, 1.0);
            }
        }

        outputs.insert("out".into(), out);
    }

    fn set_param(&mut self, _name: &str, _value: f32) {
        // VCA has no parameters, gain is CV-controlled
    }

    fn get_param(&self, _name: &str) -> Option<f32> {
        None
    }

    fn params(&self) -> Vec<ParamDeclaration> {
        Vec::new()
    }

    fn serialize_state(&self) -> serde_json::Value {
        json!({ "id": self.id })
    }

    fn deserialize_state(&mut self, _state: &serde_json::Value) {}

    fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::BLOCK_SIZE;

    #[test]
    fn test_vca_multiplication() {
        let mut vca = Vca::new("test_vca");
        let mut inputs = HashMap::new();

        // Audio input of 0.5 across the block
        let audio_in = [0.5_f32; BLOCK_SIZE];
        let cv_in = [0.5_f32; BLOCK_SIZE];
        inputs.insert("in".to_string(), audio_in);
        inputs.insert("cv".to_string(), cv_in);

        let mut outputs = HashMap::new();
        vca.process(&inputs, &mut outputs);

        let out = outputs.get("out").expect("VCA should produce 'out'");
        for &sample in out {
            assert!(
                (sample - 0.25).abs() < f32::EPSILON,
                "VCA output should be 0.5 * 0.5 = 0.25, got {sample}"
            );
        }
    }

    #[test]
    fn test_vca_unity_when_no_cv() {
        let mut vca = Vca::new("test_vca");
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), [0.7_f32; BLOCK_SIZE]);
        // No "cv" input — should default to 1.0 (unity)

        let mut outputs = HashMap::new();
        vca.process(&inputs, &mut outputs);

        let out = outputs.get("out").expect("VCA should produce 'out'");
        for &sample in out {
            assert!(
                (sample - 0.7).abs() < f32::EPSILON,
                "VCA with no CV should pass through at unity, got {sample}"
            );
        }
    }
}

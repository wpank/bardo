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

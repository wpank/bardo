//! Four-input summing mixer module.

use std::collections::HashMap;

use serde_json::json;

use super::{BLOCK_SIZE, Module, PortDeclaration, PortDirection, SignalBlock, SignalType};
use crate::params::ParamDeclaration;

/// Four-input summing mixer.
///
/// Each input has an independent level control (0.0-1.0).
/// Output is the weighted sum of all connected inputs.
pub struct Mixer {
    id: String,
    levels: [f32; 4],
    ports: Vec<PortDeclaration>,
}

impl Mixer {
    /// Creates a new mixer with the given instance id. All levels default to 1.0.
    pub fn new(id: impl Into<String>) -> Self {
        let mut ports = Vec::with_capacity(5);
        for i in 1..=4 {
            ports.push(PortDeclaration {
                name: format!("in_{i}"),
                signal_type: SignalType::Audio,
                direction: PortDirection::Input,
                description: format!("Audio input {i}"),
                default_value: 0.0,
            });
        }
        ports.push(PortDeclaration {
            name: "out".into(),
            signal_type: SignalType::Audio,
            direction: PortDirection::Output,
            description: "Mixed audio output".into(),
            default_value: 0.0,
        });

        Self {
            id: id.into(),
            levels: [1.0; 4],
            ports,
        }
    }
}

impl Module for Mixer {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        "Mixer"
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

        for ch in 0..4 {
            let port_name = format!("in_{}", ch + 1);
            if let Some(input) = inputs.get(&port_name) {
                let level = self.levels[ch];
                for i in 0..BLOCK_SIZE {
                    out[i] += input[i] * level;
                }
            }
        }

        // Soft-clip to prevent overflow
        for sample in &mut out {
            *sample = sample.clamp(-1.0, 1.0);
        }

        outputs.insert("out".into(), out);
    }

    fn set_param(&mut self, name: &str, value: f32) {
        let idx = match name {
            "level_1" => Some(0),
            "level_2" => Some(1),
            "level_3" => Some(2),
            "level_4" => Some(3),
            _ => None,
        };
        if let Some(i) = idx {
            self.levels[i] = value.clamp(0.0, 1.0);
        }
    }

    fn get_param(&self, name: &str) -> Option<f32> {
        match name {
            "level_1" => Some(self.levels[0]),
            "level_2" => Some(self.levels[1]),
            "level_3" => Some(self.levels[2]),
            "level_4" => Some(self.levels[3]),
            _ => None,
        }
    }

    fn params(&self) -> Vec<ParamDeclaration> {
        (1..=4)
            .map(|i| ParamDeclaration {
                name: format!("level_{i}"),
                display_name: format!("Level {i}"),
                value: self.levels[i - 1],
                min: 0.0,
                max: 1.0,
                default: 1.0,
                unit: "".into(),
                description: format!("Input {i} level"),
            })
            .collect()
    }

    fn serialize_state(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "levels": self.levels,
        })
    }

    fn deserialize_state(&mut self, state: &serde_json::Value) {
        if let Some(levels) = state.get("levels").and_then(|v| v.as_array()) {
            for (i, val) in levels.iter().enumerate().take(4) {
                if let Some(f) = val.as_f64() {
                    self.levels[i] = f as f32;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.levels = [1.0; 4];
    }
}

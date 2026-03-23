//! Modular synthesis rack: module graph, patch cables, and block processing.

use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::modules::{Module, PatchCable, PortDirection, PortId, SignalBlock, zero_block};

/// The central signal-processing graph.
///
/// Holds modules connected by patch cables. On each `process_block()` call,
/// modules are processed in topological order (Kahn's algorithm) so that
/// every module reads only from outputs that have already been computed
/// in the current block.
pub struct Rack {
    modules: IndexMap<String, Box<dyn Module>>,
    cables: Vec<PatchCable>,
    processing_order: Vec<String>,
    /// Per-port signal buffers, pre-allocated.
    pub buffers: HashMap<PortId, SignalBlock>,
    /// Master output level (0.0-1.0).
    pub master_level: f32,
}

impl Rack {
    /// Create an empty rack.
    pub fn new() -> Self {
        Self {
            modules: IndexMap::new(),
            cables: Vec::new(),
            processing_order: Vec::new(),
            buffers: HashMap::new(),
            master_level: 1.0,
        }
    }

    /// Add a module to the rack. Recomputes processing order.
    pub fn add_module(&mut self, module: Box<dyn Module>) {
        let id = module.id().to_owned();
        // Pre-allocate buffers for all ports
        for port in module.ports() {
            let port_id = PortId {
                module_id: id.clone(),
                port_name: port.name.clone(),
            };
            self.buffers.entry(port_id).or_insert_with(zero_block);
        }
        self.modules.insert(id, module);
        self.recompute_processing_order();
    }

    /// Remove a module and all its cables. Recomputes processing order.
    pub fn remove_module(&mut self, id: &str) {
        self.modules.swap_remove(id);
        self.cables
            .retain(|c| c.from.module_id != id && c.to.module_id != id);
        self.buffers.retain(|k, _| k.module_id != id);
        self.recompute_processing_order();
    }

    /// Connect two ports with a patch cable. Recomputes processing order.
    pub fn connect(&mut self, from: PortId, to: PortId, attenuation: f32) {
        self.cables.push(PatchCable {
            from,
            to,
            attenuation,
        });
        self.recompute_processing_order();
    }

    /// Remove a cable between two ports. Recomputes processing order.
    pub fn disconnect(&mut self, from: &PortId, to: &PortId) {
        self.cables.retain(|c| &c.from != from || &c.to != to);
        self.recompute_processing_order();
    }

    /// Process one block through the entire rack.
    ///
    /// Returns a stereo pair (left, right). The master output is taken from
    /// the first module whose id starts with "mixer" (convention), or the
    /// last module in processing order if no mixer is found. The mono output
    /// is duplicated to both channels.
    pub fn process_block(&mut self) -> (SignalBlock, SignalBlock) {
        let order: Vec<String> = self.processing_order.clone();

        for module_id in &order {
            // Gather inputs for this module from cables
            let mut inputs: HashMap<String, SignalBlock> = HashMap::new();

            // Find all cables targeting this module
            let incoming: Vec<(String, String, f32)> = self
                .cables
                .iter()
                .filter(|c| c.to.module_id == *module_id)
                .map(|c| {
                    (
                        c.from.port_name.clone(),
                        c.to.port_name.clone(),
                        c.attenuation,
                    )
                })
                .collect();

            for (from_port, to_port, attenuation) in &incoming {
                // Find the source module id from cables
                let source_module_id = self
                    .cables
                    .iter()
                    .find(|c| {
                        c.to.module_id == *module_id
                            && c.from.port_name == *from_port
                            && c.to.port_name == *to_port
                            && (c.attenuation - attenuation).abs() < f32::EPSILON
                    })
                    .map(|c| c.from.module_id.clone());

                if let Some(src_id) = source_module_id {
                    let src_port_id = PortId {
                        module_id: src_id,
                        port_name: from_port.clone(),
                    };
                    if let Some(src_buf) = self.buffers.get(&src_port_id) {
                        let src_data = *src_buf;
                        // Multiple cables to the same input port: sum with attenuation
                        let entry = inputs.entry(to_port.clone()).or_insert_with(zero_block);
                        for i in 0..src_data.len() {
                            entry[i] += src_data[i] * attenuation;
                        }
                    }
                }
            }

            // Fill missing inputs with port defaults
            if let Some(module) = self.modules.get(module_id) {
                for port in module.ports() {
                    if port.direction == PortDirection::Input && !inputs.contains_key(&port.name) {
                        let mut block = zero_block();
                        for sample in &mut block {
                            *sample = port.default_value;
                        }
                        inputs.insert(port.name.clone(), block);
                    }
                }
            }

            // Process the module
            let mut outputs: HashMap<String, SignalBlock> = HashMap::new();
            if let Some(module) = self.modules.get_mut(module_id) {
                module.process(&inputs, &mut outputs);
            }

            // Store outputs in buffers
            for (port_name, data) in outputs {
                let port_id = PortId {
                    module_id: module_id.clone(),
                    port_name,
                };
                self.buffers.insert(port_id, data);
            }
        }

        // Extract master output: look for mixer "out" port, fall back to last module
        let master = self.find_master_output();
        let mut left = master;
        let right = master;

        // Apply master level
        for sample in &mut left {
            *sample *= self.master_level;
        }
        let mut right = right;
        for sample in &mut right {
            *sample *= self.master_level;
        }

        (left, right)
    }

    /// Serialize rack state to JSON.
    pub fn serialize(&self) -> serde_json::Value {
        let module_states: Vec<serde_json::Value> = self
            .modules
            .iter()
            .map(|(id, m)| {
                json!({
                    "id": id,
                    "display_name": m.display_name(),
                    "state": m.serialize_state(),
                })
            })
            .collect();

        let cable_data: Vec<serde_json::Value> = self
            .cables
            .iter()
            .map(|c| {
                json!({
                    "from_module": c.from.module_id,
                    "from_port": c.from.port_name,
                    "to_module": c.to.module_id,
                    "to_port": c.to.port_name,
                    "attenuation": c.attenuation,
                })
            })
            .collect();

        json!({
            "modules": module_states,
            "cables": cable_data,
            "master_level": self.master_level,
        })
    }

    /// Restore rack state from JSON.
    ///
    /// Note: this restores cables and master_level. Module state restoration
    /// requires the modules to already exist in the rack (call `add_module`
    /// first with the appropriate module instances, then `deserialize`).
    /// This is a limitation until a module factory/registry is implemented.
    pub fn deserialize(&mut self, state: &serde_json::Value) {
        if let Some(ml) = state.get("master_level").and_then(|v| v.as_f64()) {
            self.master_level = ml as f32;
        }

        // Restore cables
        if let Some(cables) = state.get("cables").and_then(|v| v.as_array()) {
            self.cables.clear();
            for c in cables {
                let from_module = c.get("from_module").and_then(|v| v.as_str()).unwrap_or("");
                let from_port = c.get("from_port").and_then(|v| v.as_str()).unwrap_or("");
                let to_module = c.get("to_module").and_then(|v| v.as_str()).unwrap_or("");
                let to_port = c.get("to_port").and_then(|v| v.as_str()).unwrap_or("");
                let attenuation =
                    c.get("attenuation").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

                self.cables.push(PatchCable {
                    from: PortId {
                        module_id: from_module.to_owned(),
                        port_name: from_port.to_owned(),
                    },
                    to: PortId {
                        module_id: to_module.to_owned(),
                        port_name: to_port.to_owned(),
                    },
                    attenuation,
                });
            }
            self.recompute_processing_order();
        }

        // Restore per-module state (modules must already exist)
        if let Some(modules) = state.get("modules").and_then(|v| v.as_array()) {
            for m in modules {
                let id = m.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(module_state) = m.get("state") {
                    if let Some(module) = self.modules.get_mut(id) {
                        module.deserialize_state(module_state);
                    }
                }
            }
        }
    }

    /// Kahn's topological sort over the module dependency graph.
    ///
    /// Builds a DAG where cables create edges from source module to
    /// destination module, then processes modules in zero-in-degree order.
    /// For acyclic graphs, the result includes all modules.
    fn recompute_processing_order(&mut self) {
        let module_ids: Vec<String> = self.modules.keys().cloned().collect();

        // Build in-degree map
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for id in &module_ids {
            in_degree.insert(id.as_str(), 0);
        }

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for cable in &self.cables {
            let from = cable.from.module_id.as_str();
            let to = cable.to.module_id.as_str();
            if from != to && in_degree.contains_key(from) && in_degree.contains_key(to) {
                adj.entry(from).or_default().push(to);
                *in_degree.entry(to).or_default() += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        // Sort for deterministic ordering among equal-degree nodes
        queue.sort();

        let mut order = Vec::with_capacity(module_ids.len());
        while let Some(node) = queue.first().copied() {
            queue.remove(0);
            order.push(node.to_owned());

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            // Insert sorted to keep deterministic
                            let pos = queue.partition_point(|&x| x < neighbor);
                            queue.insert(pos, neighbor);
                        }
                    }
                }
            }
        }

        self.processing_order = order;
    }

    /// Find the master output signal block.
    /// Looks for a module whose id starts with "mixer" and reads its "out" port.
    /// Falls back to the last module in processing order.
    fn find_master_output(&self) -> SignalBlock {
        // Try mixer first
        for id in &self.processing_order {
            if id.starts_with("mixer") {
                let port_id = PortId {
                    module_id: id.clone(),
                    port_name: "out".into(),
                };
                if let Some(buf) = self.buffers.get(&port_id) {
                    return *buf;
                }
            }
        }

        // Fall back to last module's "out" port
        if let Some(last_id) = self.processing_order.last() {
            let port_id = PortId {
                module_id: last_id.clone(),
                port_name: "out".into(),
            };
            if let Some(buf) = self.buffers.get(&port_id) {
                return *buf;
            }
        }

        zero_block()
    }
}

impl Default for Rack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::{mixer::Mixer, noise::NoiseSource, vca::Vca};

    #[test]
    fn test_rack_topological_sort() {
        let mut rack = Rack::new();
        rack.add_module(Box::new(NoiseSource::new("A")));
        rack.add_module(Box::new(Vca::new("B")));
        rack.add_module(Box::new(Mixer::new("C")));

        // Wire A → B → C
        rack.connect(
            PortId {
                module_id: "A".into(),
                port_name: "out".into(),
            },
            PortId {
                module_id: "B".into(),
                port_name: "in".into(),
            },
            1.0,
        );
        rack.connect(
            PortId {
                module_id: "B".into(),
                port_name: "out".into(),
            },
            PortId {
                module_id: "C".into(),
                port_name: "in_1".into(),
            },
            1.0,
        );

        assert_eq!(rack.processing_order, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_rack_process_no_panic() {
        let mut rack = Rack::new();
        rack.add_module(Box::new(NoiseSource::new("noise")));
        let (_left, _right) = rack.process_block();
    }

    #[test]
    fn test_rack_default_patch() {
        let mut rack = Rack::new();
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

        rack.master_level = 0.5;
        let (left, right) = rack.process_block();

        // Noise through VCA (default cv=1.0) through mixer should produce non-silent output
        let max_abs = left.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(
            max_abs > 0.0,
            "Default patch should produce non-silent output"
        );
        // Both channels should match (mono duplicate)
        assert_eq!(left, right);
    }

    #[test]
    fn test_default_patch_silent_when_vitality_zero() {
        let mut rack = Rack::new();
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

        rack.master_level = 0.0;
        let (left, _right) = rack.process_block();
        let max_abs = left.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(
            max_abs < f32::EPSILON,
            "master_level=0 should produce silence"
        );
    }

    #[test]
    fn test_rack_serialize_deserialize_roundtrip() {
        let mut rack = Rack::new();
        rack.add_module(Box::new(NoiseSource::new("n1")));
        rack.add_module(Box::new(Vca::new("v1")));
        rack.connect(
            PortId {
                module_id: "n1".into(),
                port_name: "out".into(),
            },
            PortId {
                module_id: "v1".into(),
                port_name: "in".into(),
            },
            0.8,
        );
        rack.master_level = 0.7;

        let state = rack.serialize();

        // Build a fresh rack with same modules, then deserialize
        let mut rack2 = Rack::new();
        rack2.add_module(Box::new(NoiseSource::new("n1")));
        rack2.add_module(Box::new(Vca::new("v1")));
        rack2.deserialize(&state);

        assert!((rack2.master_level - 0.7).abs() < f32::EPSILON);
        assert_eq!(rack2.cables.len(), 1);
        assert!((rack2.cables[0].attenuation - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rack_process_block_under_500us() {
        let mut rack = Rack::new();
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

        // Warm up
        for _ in 0..100 {
            rack.process_block();
        }

        let start = std::time::Instant::now();
        for _ in 0..100 {
            rack.process_block();
        }
        let elapsed = start.elapsed();
        let per_block = elapsed / 100;

        assert!(
            per_block.as_micros() < 500,
            "process_block took {}µs, must be < 500µs",
            per_block.as_micros()
        );
    }
}

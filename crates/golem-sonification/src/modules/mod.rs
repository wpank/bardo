//! Module system: trait definition, signal types, and port declarations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::params::ParamDeclaration;

pub mod lfo;
pub mod mixer;
pub mod noise;
pub mod vca;

/// Number of samples per processing block.
pub const BLOCK_SIZE: usize = 32;

/// Audio sample rate in Hz.
pub const SAMPLE_RATE: f32 = 48_000.0;

/// A fixed-size block of samples.
pub type SignalBlock = [f32; BLOCK_SIZE];

/// Returns a zeroed signal block.
#[must_use]
pub fn zero_block() -> SignalBlock {
    [0.0; BLOCK_SIZE]
}

/// Signal type classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// Audio-rate signal, nominally -1.0 to 1.0.
    Audio,
    /// Control voltage, 0.0 to 1.0 unipolar or -1.0 to 1.0 bipolar.
    Cv,
    /// Gate/trigger, 0.0 or 1.0.
    Gate,
}

/// Port direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    /// Receives signal.
    Input,
    /// Produces signal.
    Output,
}

/// Identifies a specific port on a specific module instance.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId {
    /// Module instance identifier.
    pub module_id: String,
    /// Port name within the module.
    pub port_name: String,
}

/// Declares a port's properties.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortDeclaration {
    /// Port name (matches keys in process I/O maps).
    pub name: String,
    /// Signal type.
    pub signal_type: SignalType,
    /// Direction.
    pub direction: PortDirection,
    /// Human-readable description.
    pub description: String,
    /// Default value when no cable is connected.
    pub default_value: f32,
}

/// A cable connecting one module's output port to another module's input port.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchCable {
    /// Source port.
    pub from: PortId,
    /// Destination port.
    pub to: PortId,
    /// Signal attenuation factor (0.0-1.0).
    pub attenuation: f32,
}

/// The processing interface for all rack modules.
///
/// Modules read from named input ports and write to named output ports.
/// The `Rack` handles routing between modules via patch cables.
pub trait Module: Send {
    /// Unique instance identifier.
    fn id(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// Port declarations describing inputs and outputs.
    fn ports(&self) -> &[PortDeclaration];

    /// Process one block. Reads from `inputs` keyed by port name,
    /// writes results into `outputs` keyed by port name.
    fn process(
        &mut self,
        inputs: &HashMap<String, SignalBlock>,
        outputs: &mut HashMap<String, SignalBlock>,
    );

    /// Set a named parameter to a new value.
    fn set_param(&mut self, name: &str, value: f32);

    /// Get a named parameter's current value.
    fn get_param(&self, name: &str) -> Option<f32>;

    /// Parameter declarations.
    fn params(&self) -> Vec<ParamDeclaration>;

    /// Serialize module state to JSON.
    fn serialize_state(&self) -> serde_json::Value;

    /// Restore module state from JSON.
    fn deserialize_state(&mut self, state: &serde_json::Value);

    /// Reset to initial state.
    fn reset(&mut self);
}

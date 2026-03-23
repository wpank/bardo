//! Rack preset save/load.

use crate::rack::Rack;

/// A named snapshot of a rack configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Preset {
    /// Preset name.
    pub name: String,
    /// Serialized rack state.
    pub rack_state: serde_json::Value,
}

impl Preset {
    /// Capture the current rack state as a preset.
    pub fn save(name: impl Into<String>, rack: &Rack) -> Self {
        Self {
            name: name.into(),
            rack_state: rack.serialize(),
        }
    }

    /// Restore a rack from this preset's saved state.
    ///
    /// Note: module deserialization requires a module factory/registry
    /// to reconstruct `Box<dyn Module>` instances. This stub deserializes
    /// cables and master_level but cannot rehydrate module state without
    /// a factory. See `Rack::deserialize` for details.
    pub fn load(&self, rack: &mut Rack) {
        rack.deserialize(&self.rack_state);
    }
}

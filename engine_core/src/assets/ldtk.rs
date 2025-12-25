use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rules for processing IntGrid layers (e.g., collisions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntGridStrategy {
    Ignore,
    SolidCollider, // Value 1 = Solid
    Trigger,       // Creates trigger zones
}

impl Default for IntGridStrategy {
    fn default() -> Self {
        Self::SolidCollider
    }
}

/// Settings for importing LDtk projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdtkImportSettings {
    /// World scale: How many pixels constitute one world unit.
    /// Default: 16.0 (1 tile = 1 unit for 16px tiles) or 100.0 (Unity standard)
    pub pixels_per_unit: f32,
    
    /// Global toggle for collider generation
    pub generate_colliders: bool,
    
    /// Per-layer strategies (Layer Name -> Strategy)
    #[serde(default)]
    pub layer_strategies: HashMap<String, IntGridStrategy>,
}

impl Default for LdtkImportSettings {
    fn default() -> Self {
        Self {
            pixels_per_unit: 16.0,
            generate_colliders: true,
            layer_strategies: HashMap::new(),
        }
    }
}

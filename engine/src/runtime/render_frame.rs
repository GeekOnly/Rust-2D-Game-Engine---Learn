use render::{MeshInstance, CameraUniform, LightUniform};
use std::collections::HashMap;

// We need a key to batch meshes. 
// Ideally "Asset ID" (String) for Mesh and "Material ID" (String).
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BatchKey {
    pub mesh_id: String,
    pub material_id: String, // Could be texture path or material name
}

pub struct RenderFrame {
    pub camera: CameraUniform,
    pub lights: Vec<LightUniform>, 
    // Batches for Opaque objects
    pub opaque_batches: HashMap<BatchKey, Vec<MeshInstance>>,
    // Transparent objects might need sorting, so we might store them differently or just sorted keys
    pub transparent_batches: HashMap<BatchKey, Vec<MeshInstance>>,
    
    // UI Commands or other stuff can go here
}

impl RenderFrame {
    pub fn new() -> Self {
        Self {
            camera: CameraUniform::new(), 
            lights: Vec::new(),
            opaque_batches: HashMap::new(),
            transparent_batches: HashMap::new(),
        }
    }

    pub fn push_instance(&mut self, mesh_id: String, material_id: String, instance: MeshInstance) {
        let key = BatchKey { mesh_id, material_id };
        self.opaque_batches
            .entry(key)
            .or_insert_with(Vec::new)
            .push(instance);
    }
}

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod traits;
pub mod component_manager;
pub mod components;
pub mod loaders;

// Re-export สำหรับใช้งานง่าย
pub use component_manager::{ComponentType, ComponentManager};
pub use components::*;

pub type Entity = u32;

/// Prefab system for reusable entity templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub transform: Transform,
    pub sprite: Option<Sprite>,
    pub collider: Option<Collider>,
    pub rigidbody: Option<Rigidbody2D>,
    pub velocity: Option<(f32, f32)>,  // Legacy - kept for backward compatibility
    pub tag: Option<EntityTag>,
    pub script: Option<Script>,
    pub camera: Option<Camera>,
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: None,
        }
    }

    /// Create a Player prefab with Rigidbody
    pub fn player() -> Self {
        Self {
            name: "Player".to_string(),
            transform: Transform::default(), // Use default transform (0, 0, 0) to work in both 2D and 3D
            sprite: Some(Sprite {
                texture_id: "player".to_string(),
                width: 40.0,
                height: 40.0,
                color: [0.2, 0.6, 1.0, 1.0],
                billboard: true, // Player sprite faces camera (good for 3D mode)
            }),
            collider: Some(Collider { width: 40.0, height: 40.0 }),
            rigidbody: Some(Rigidbody2D::default()),  // Add rigidbody with default values
            velocity: Some((0.0, 0.0)),
            tag: Some(EntityTag::Player),
            script: None,
            camera: None,
        }
    }

    /// Create an Item prefab
    pub fn item() -> Self {
        Self {
            name: "Item".to_string(),
            transform: Transform::default(), // Use default transform (0, 0, 0) to work in both 2D and 3D
            sprite: Some(Sprite {
                texture_id: "item".to_string(),
                width: 30.0,
                height: 30.0,
                color: [1.0, 0.8, 0.2, 1.0],
                billboard: true, // Item sprite faces camera
            }),
            collider: Some(Collider { width: 30.0, height: 30.0 }),
            rigidbody: None,
            velocity: None,
            tag: Some(EntityTag::Item),
            script: None,
            camera: None,
        }
    }

    /// Create a 2D Camera prefab
    pub fn camera_2d() -> Self {
        Self {
            name: "Camera 2D".to_string(),
            transform: Transform::with_position(0.0, 0.0, -10.0), // Camera behind objects
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: Some(Camera::orthographic_2d()),
        }
    }

    /// Create a 3D Camera prefab
    pub fn camera_3d() -> Self {
        Self {
            name: "Camera 3D".to_string(),
            transform: Transform::with_position(0.0, 5.0, -10.0), // Camera above and behind
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: Some(Camera::perspective_3d()),
        }
    }

    /// Spawn this prefab into the world
    pub fn spawn(&self, world: &mut World) -> Entity {
        let entity = world.spawn();
        world.transforms.insert(entity, self.transform.clone());
        world.names.insert(entity, self.name.clone());

        if let Some(sprite) = &self.sprite {
            world.sprites.insert(entity, sprite.clone());
        }
        if let Some(collider) = &self.collider {
            world.colliders.insert(entity, collider.clone());
        }
        if let Some(rigidbody) = &self.rigidbody {
            world.rigidbodies.insert(entity, rigidbody.clone());
            // Sync velocity from rigidbody
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = self.velocity {
            // Legacy velocity support
            world.velocities.insert(entity, velocity);
        }
        if let Some(tag) = &self.tag {
            world.tags.insert(entity, tag.clone());
        }
        if let Some(script) = &self.script {
            world.scripts.insert(entity, script.clone());
        }
        if let Some(camera) = &self.camera {
            world.cameras.insert(entity, camera.clone());
        }

        entity
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],  // X, Y, Z
    pub rotation: [f32; 3],  // Euler angles: X, Y, Z (in degrees)
    pub scale: [f32; 3],     // X, Y, Z
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

// Helper methods for backward compatibility and convenience
impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    pub fn with_position_2d(x: f32, y: f32) -> Self {
        Self::with_position(x, y, 0.0)
    }

    // Getters for convenience
    pub fn x(&self) -> f32 { self.position[0] }
    pub fn y(&self) -> f32 { self.position[1] }
    pub fn z(&self) -> f32 { self.position[2] }

    // Setters for convenience
    pub fn set_x(&mut self, x: f32) { self.position[0] = x; }
    pub fn set_y(&mut self, y: f32) { self.position[1] = y; }
    pub fn set_z(&mut self, z: f32) { self.position[2] = z; }
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = [x, y, z];
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_id: String,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4], // RGBA
    pub billboard: bool, // If true, sprite always faces camera (3D mode only)
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_id: String::new(),
            width: 0.0,
            height: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false, // Default: not a billboard
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}

/// Rigidbody 2D properties (Unity-like)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rigidbody2D {
    pub velocity: (f32, f32),      // Current velocity (vx, vy)
    pub gravity_scale: f32,         // Gravity multiplier (0 = no gravity, 1 = normal)
    pub mass: f32,                  // Mass (affects collision response)
    pub is_kinematic: bool,         // If true, not affected by physics (but still collides)
    pub freeze_rotation: bool,      // Prevent rotation (for 2D games)
}

impl Default for Rigidbody2D {
    fn default() -> Self {
        Self {
            velocity: (0.0, 0.0),
            gravity_scale: 1.0,
            mass: 1.0,
            is_kinematic: false,
            freeze_rotation: true,
        }
    }
}

/// 3D Mesh component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_type: MeshType,
    pub color: [f32; 4], // RGBA
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MeshType {
    Cube,
    Sphere,
    Cylinder,
    Plane,
    Capsule,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EntityTag {
    Player,
    Item,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub script_name: String,
    pub enabled: bool,
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, ScriptParameter>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ScriptParameter {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
}

/// Camera component for view control (Unity-like)
/// Can be attached to an entity to create game cameras
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    // Camera projection settings
    pub projection: CameraProjection,

    // Field of view (for perspective projection, in degrees)
    pub fov: f32,

    // Orthographic size (for orthographic projection, half-height in world units)
    pub orthographic_size: f32,

    // Near and far clip planes
    pub near_clip: f32,
    pub far_clip: f32,

    // Viewport settings (normalized 0-1)
    pub viewport_rect: [f32; 4], // x, y, width, height

    // Camera depth (lower renders first, like Unity)
    pub depth: i32,

    // Clear flags
    pub clear_flags: CameraClearFlags,
    pub background_color: [f32; 4], // RGBA
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CameraProjection {
    Orthographic, // 2D camera
    Perspective,  // 3D camera
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CameraClearFlags {
    SolidColor,
    Skybox,
    DepthOnly,
    DontClear,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            fov: 60.0,
            orthographic_size: 5.0,
            near_clip: 0.3,
            far_clip: 1000.0,
            viewport_rect: [0.0, 0.0, 1.0, 1.0], // Full screen
            depth: 0,
            clear_flags: CameraClearFlags::SolidColor,
            background_color: [0.15, 0.16, 0.18, 1.0], // Dark gray (Unity default)
        }
    }
}

impl Camera {
    /// Create a 2D orthographic camera (Unity 2D style)
    pub fn orthographic_2d() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            orthographic_size: 5.0,
            ..Default::default()
        }
    }

    /// Create a 3D perspective camera (Unity 3D style)
    pub fn perspective_3d() -> Self {
        Self {
            projection: CameraProjection::Perspective,
            fov: 60.0,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct World {
    next_entity: Entity,
    pub transforms: HashMap<Entity, Transform>,
    pub velocities: HashMap<Entity, (f32, f32)>,  // Legacy - kept for backward compatibility
    pub rigidbodies: HashMap<Entity, Rigidbody2D>, // New Rigidbody2D component
    pub sprites: HashMap<Entity, Sprite>,
    pub colliders: HashMap<Entity, Collider>,
    pub meshes: HashMap<Entity, Mesh>,      // 3D meshes
    pub cameras: HashMap<Entity, Camera>,   // Camera components
    pub tags: HashMap<Entity, EntityTag>,
    pub scripts: HashMap<Entity, Script>,
    pub active: HashMap<Entity, bool>,      // Active state (Unity-like)
    pub layers: HashMap<Entity, u8>,        // Layer (0-31, Unity has 32 layers)
    pub parents: HashMap<Entity, Entity>,   // Parent entity
    pub children: HashMap<Entity, Vec<Entity>>, // Children entities
    pub names: HashMap<Entity, String>,     // Entity names (for editor)
    // Sprite sheet and tilemap components
    pub sprite_sheets: HashMap<Entity, SpriteSheet>,
    pub animated_sprites: HashMap<Entity, AnimatedSprite>,
    pub tilemaps: HashMap<Entity, Tilemap>,
    pub tilesets: HashMap<Entity, TileSet>,
}

impl World {
    pub fn new() -> Self { Self::default() }

    pub fn spawn(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        // New entities are active by default (Unity behavior)
        self.active.insert(id, true);
        // New entities start on layer 0 (Default layer)
        self.layers.insert(id, 0);
        id
    }

    pub fn despawn(&mut self, e: Entity) {
        // Recursively despawn children
        if let Some(children) = self.children.remove(&e) {
            for child in children {
                self.despawn(child);
            }
        }

        // Remove from parent's children list
        if let Some(parent) = self.parents.remove(&e) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&x| x != e);
            }
        }

        self.transforms.remove(&e);
        self.velocities.remove(&e);
        self.rigidbodies.remove(&e);
        self.sprites.remove(&e);
        self.colliders.remove(&e);
        self.meshes.remove(&e);
        self.cameras.remove(&e);
        self.tags.remove(&e);
        self.scripts.remove(&e);
        self.active.remove(&e);
        self.layers.remove(&e);
        self.names.remove(&e);
        self.sprite_sheets.remove(&e);
        self.animated_sprites.remove(&e);
        self.tilemaps.remove(&e);
        self.tilesets.remove(&e);
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.velocities.clear();
        self.rigidbodies.clear();
        self.sprites.clear();
        self.colliders.clear();
        self.meshes.clear();
        self.cameras.clear();
        self.tags.clear();
        self.scripts.clear();
        self.active.clear();
        self.layers.clear();
        self.parents.clear();
        self.children.clear();
        self.names.clear();
        self.sprite_sheets.clear();
        self.animated_sprites.clear();
        self.tilemaps.clear();
        self.tilesets.clear();
        self.next_entity = 0;
    }

    pub fn set_parent(&mut self, child: Entity, parent: Option<Entity>) {
        // Remove from old parent
        if let Some(old_parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&x| x != child);
            }
        }

        // Add to new parent
        if let Some(new_parent) = parent {
            self.parents.insert(child, new_parent);
            self.children.entry(new_parent).or_default().push(child);
        }
    }

    pub fn get_children(&self, entity: Entity) -> &[Entity] {
        self.children.get(&entity).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.parents.get(&entity).copied()
    }

    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct SceneData {
            next_entity: Entity,
            transforms: Vec<(Entity, Transform)>,
            velocities: Vec<(Entity, (f32, f32))>,
            sprites: Vec<(Entity, Sprite)>,
            colliders: Vec<(Entity, Collider)>,
            cameras: Vec<(Entity, Camera)>,  // Added camera serialization
            meshes: Vec<(Entity, Mesh)>,     // Added mesh serialization
            tags: Vec<(Entity, EntityTag)>,
            scripts: Vec<(Entity, Script)>,
            active: Vec<(Entity, bool)>,
            layers: Vec<(Entity, u8)>,
            parents: Vec<(Entity, Entity)>,
            names: Vec<(Entity, String)>,
        }

        let data = SceneData {
            next_entity: self.next_entity,
            transforms: self.transforms.iter().map(|(k, v)| (*k, v.clone())).collect(),
            velocities: self.velocities.iter().map(|(k, v)| (*k, *v)).collect(),
            sprites: self.sprites.iter().map(|(k, v)| (*k, v.clone())).collect(),
            colliders: self.colliders.iter().map(|(k, v)| (*k, v.clone())).collect(),
            cameras: self.cameras.iter().map(|(k, v)| (*k, v.clone())).collect(),
            meshes: self.meshes.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tags: self.tags.iter().map(|(k, v)| (*k, v.clone())).collect(),
            scripts: self.scripts.iter().map(|(k, v)| (*k, v.clone())).collect(),
            active: self.active.iter().map(|(k, v)| (*k, *v)).collect(),
            layers: self.layers.iter().map(|(k, v)| (*k, *v)).collect(),
            parents: self.parents.iter().map(|(k, v)| (*k, *v)).collect(),
            names: self.names.iter().map(|(k, v)| (*k, v.clone())).collect(),
        };

        serde_json::to_string_pretty(&data)
    }

    pub fn load_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        #[derive(Deserialize)]
        struct SceneData {
            #[serde(default)]
            next_entity: Entity,
            #[serde(default)]
            transforms: Vec<(Entity, Transform)>,
            #[serde(default)]
            velocities: Vec<(Entity, (f32, f32))>,
            #[serde(default)]
            sprites: Vec<(Entity, Sprite)>,
            #[serde(default)]
            colliders: Vec<(Entity, Collider)>,
            #[serde(default)]
            cameras: Vec<(Entity, Camera)>,  // Added camera deserialization
            #[serde(default)]
            meshes: Vec<(Entity, Mesh)>,     // Added mesh deserialization
            #[serde(default)]
            tags: Vec<(Entity, EntityTag)>,
            #[serde(default)]
            scripts: Vec<(Entity, Script)>,
            #[serde(default)]
            active: Vec<(Entity, bool)>,
            #[serde(default)]
            layers: Vec<(Entity, u8)>,
            #[serde(default)]
            parents: Vec<(Entity, Entity)>,
            #[serde(default)]
            names: Vec<(Entity, String)>,
        }

        let data: SceneData = serde_json::from_str(json)?;

        self.clear();
        
        // Set next_entity, or calculate from existing entities if not provided
        if data.next_entity > 0 {
            self.next_entity = data.next_entity;
        } else {
            // Calculate next_entity from max entity ID + 1
            let max_entity = data.transforms.iter()
                .map(|(e, _)| *e)
                .max()
                .unwrap_or(0);
            self.next_entity = max_entity + 1;
        }

        for (entity, transform) in data.transforms {
            self.transforms.insert(entity, transform);
        }
        for (entity, velocity) in data.velocities {
            self.velocities.insert(entity, velocity);
        }
        for (entity, sprite) in data.sprites {
            self.sprites.insert(entity, sprite);
        }
        for (entity, collider) in data.colliders {
            self.colliders.insert(entity, collider);
        }
        for (entity, camera) in data.cameras {
            self.cameras.insert(entity, camera);
        }
        for (entity, mesh) in data.meshes {
            self.meshes.insert(entity, mesh);
        }
        for (entity, name) in data.names {
            self.names.insert(entity, name);
        }
        for (entity, tag) in data.tags {
            self.tags.insert(entity, tag);
        }
        for (entity, script) in data.scripts {
            self.scripts.insert(entity, script);
        }
        for (entity, is_active) in data.active {
            self.active.insert(entity, is_active);
        }
        for (entity, layer) in data.layers {
            self.layers.insert(entity, layer);
        }
        
        // Reconstruct hierarchy
        for (child, parent) in data.parents {
            self.parents.insert(child, parent);
            self.children.entry(parent).or_default().push(child);
        }

        // Ensure all entities have active and layer (backward compatibility)
        for &entity in self.transforms.keys() {
            self.active.entry(entity).or_insert(true);
            self.layers.entry(entity).or_insert(0);
        }

        Ok(())
    }
}

// ============================================================================
// Trait Implementations for World
// ============================================================================

use traits::{EcsWorld, EcsError, Serializable};

impl EcsWorld for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        World::spawn(self)
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        // Check if entity exists
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }
        
        World::despawn(self, entity);
        Ok(())
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        // An entity is alive if it has at least one component or is in the active map
        self.transforms.contains_key(&entity) ||
        self.sprites.contains_key(&entity) ||
        self.colliders.contains_key(&entity) ||
        self.meshes.contains_key(&entity) ||
        self.cameras.contains_key(&entity) ||
        self.active.contains_key(&entity)
    }
    
    fn clear(&mut self) {
        World::clear(self);
    }
    
    fn entity_count(&self) -> usize {
        self.active.len()
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        // Check for circular reference if setting a parent
        if let Some(p) = parent {
            // Check if parent is actually a descendant of child
            let mut current = Some(p);
            while let Some(ancestor) = current {
                if ancestor == child {
                    return Err(EcsError::InvalidHierarchy);
                }
                current = self.get_parent(ancestor);
            }
        }
        
        World::set_parent(self, child, parent);
        Ok(())
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        World::get_parent(self, entity)
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        World::get_children(self, entity).to_vec()
    }
}

impl Serializable for World {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        World::save_to_json(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        World::load_from_json(self, json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

// Implement ComponentAccess for all component types using the macro
impl_component_access!(World, Transform, transforms);
impl_component_access!(World, Sprite, sprites);
impl_component_access!(World, Collider, colliders);
impl_component_access!(World, Mesh, meshes);
impl_component_access!(World, Camera, cameras);
impl_component_access!(World, Script, scripts);
impl_component_access!(World, EntityTag, tags);
impl_component_access!(World, SpriteSheet, sprite_sheets);
impl_component_access!(World, AnimatedSprite, animated_sprites);
impl_component_access!(World, Tilemap, tilemaps);
impl_component_access!(World, TileSet, tilesets);

// Manual implementations for tuple and primitive types
impl traits::ComponentAccess<(f32, f32)> for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn insert(&mut self, entity: Self::Entity, component: (f32, f32)) 
        -> Result<Option<(f32, f32)>, Self::Error> 
    {
        Ok(self.velocities.insert(entity, component))
    }
    
    fn get(&self, entity: Self::Entity) -> Option<&(f32, f32)> {
        self.velocities.get(&entity)
    }
    
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut (f32, f32)> {
        self.velocities.get_mut(&entity)
    }
    
    fn remove(&mut self, entity: Self::Entity) 
        -> Result<Option<(f32, f32)>, Self::Error> 
    {
        Ok(self.velocities.remove(&entity))
    }
    
    fn has(&self, entity: Self::Entity) -> bool {
        self.velocities.contains_key(&entity)
    }
}

impl traits::ComponentAccess<bool> for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn insert(&mut self, entity: Self::Entity, component: bool) 
        -> Result<Option<bool>, Self::Error> 
    {
        Ok(self.active.insert(entity, component))
    }
    
    fn get(&self, entity: Self::Entity) -> Option<&bool> {
        self.active.get(&entity)
    }
    
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut bool> {
        self.active.get_mut(&entity)
    }
    
    fn remove(&mut self, entity: Self::Entity) 
        -> Result<Option<bool>, Self::Error> 
    {
        Ok(self.active.remove(&entity))
    }
    
    fn has(&self, entity: Self::Entity) -> bool {
        self.active.contains_key(&entity)
    }
}

impl traits::ComponentAccess<u8> for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn insert(&mut self, entity: Self::Entity, component: u8) 
        -> Result<Option<u8>, Self::Error> 
    {
        Ok(self.layers.insert(entity, component))
    }
    
    fn get(&self, entity: Self::Entity) -> Option<&u8> {
        self.layers.get(&entity)
    }
    
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut u8> {
        self.layers.get_mut(&entity)
    }
    
    fn remove(&mut self, entity: Self::Entity) 
        -> Result<Option<u8>, Self::Error> 
    {
        Ok(self.layers.remove(&entity))
    }
    
    fn has(&self, entity: Self::Entity) -> bool {
        self.layers.contains_key(&entity)
    }
}

impl traits::ComponentAccess<String> for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn insert(&mut self, entity: Self::Entity, component: String) 
        -> Result<Option<String>, Self::Error> 
    {
        Ok(self.names.insert(entity, component))
    }
    
    fn get(&self, entity: Self::Entity) -> Option<&String> {
        self.names.get(&entity)
    }
    
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut String> {
        self.names.get_mut(&entity)
    }
    
    fn remove(&mut self, entity: Self::Entity) 
        -> Result<Option<String>, Self::Error> 
    {
        Ok(self.names.remove(&entity))
    }
    
    fn has(&self, entity: Self::Entity) -> bool {
        self.names.contains_key(&entity)
    }
}


// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use traits::EcsWorld;
    
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        
        // **Feature: ecs-abstraction, Property 2: Entity spawn increases count**
        // **Validates: Requirements 1.1**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            
            #[test]
            fn entity_spawn_increases_count(spawn_count in 1usize..100) {
                let mut world = World::new();
                let initial_count = world.entity_count();
                
                // Spawn entities
                for _ in 0..spawn_count {
                    world.spawn();
                }
                
                let final_count = world.entity_count();
                
                // Property: spawning N entities should increase count by exactly N
                prop_assert_eq!(final_count, initial_count + spawn_count);
            }
        }
        
        // **Feature: ecs-abstraction, Property 3: Entity despawn decreases count**
        // **Validates: Requirements 1.1**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            
            #[test]
            fn entity_despawn_decreases_count(
                spawn_count in 1usize..50,
                despawn_indices in prop::collection::vec(any::<prop::sample::Index>(), 1..20)
            ) {
                let mut world = World::new();
                
                // Spawn entities
                let mut entities = Vec::new();
                for _ in 0..spawn_count {
                    entities.push(world.spawn());
                }
                
                let count_after_spawn = world.entity_count();
                prop_assert_eq!(count_after_spawn, spawn_count);
                
                // Despawn some entities (without duplicates)
                let mut despawned = std::collections::HashSet::new();
                for index in despawn_indices {
                    let entity = entities[index.index(entities.len())];
                    if despawned.insert(entity) {
                        let _ = world.despawn(entity);
                    }
                }
                
                let final_count = world.entity_count();
                let expected_count = spawn_count - despawned.len();
                
                // Property: despawning entities should decrease count appropriately
                // Note: This accounts for children being despawned recursively
                prop_assert!(final_count <= expected_count, 
                    "Expected count <= {}, got {}", expected_count, final_count);
            }
        }
    }
}

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type Entity = u32;

/// Prefab system for reusable entity templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub transform: Transform,
    pub sprite: Option<Sprite>,
    pub collider: Option<Collider>,
    pub velocity: Option<(f32, f32)>,
    pub tag: Option<EntityTag>,
    pub script: Option<Script>,
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            sprite: None,
            collider: None,
            velocity: None,
            tag: None,
            script: None,
        }
    }

    /// Create a Player prefab
    pub fn player() -> Self {
        Self {
            name: "Player".to_string(),
            transform: Transform::with_position_2d(0.0, 0.0),
            sprite: Some(Sprite {
                texture_id: "player".to_string(),
                width: 40.0,
                height: 40.0,
                color: [0.2, 0.6, 1.0, 1.0],
            }),
            collider: Some(Collider { width: 40.0, height: 40.0 }),
            velocity: Some((0.0, 0.0)),
            tag: Some(EntityTag::Player),
            script: None,
        }
    }

    /// Create an Item prefab
    pub fn item() -> Self {
        Self {
            name: "Item".to_string(),
            transform: Transform::with_position_2d(0.0, 0.0),
            sprite: Some(Sprite {
                texture_id: "item".to_string(),
                width: 30.0,
                height: 30.0,
                color: [1.0, 0.8, 0.2, 1.0],
            }),
            collider: Some(Collider { width: 30.0, height: 30.0 }),
            velocity: None,
            tag: Some(EntityTag::Item),
            script: None,
        }
    }

    /// Spawn this prefab into the world
    pub fn spawn(&self, world: &mut World) -> Entity {
        let entity = world.spawn();
        world.transforms.insert(entity, self.transform.clone());

        if let Some(sprite) = &self.sprite {
            world.sprites.insert(entity, sprite.clone());
        }
        if let Some(collider) = &self.collider {
            world.colliders.insert(entity, collider.clone());
        }
        if let Some(velocity) = self.velocity {
            world.velocities.insert(entity, velocity);
        }
        if let Some(tag) = &self.tag {
            world.tags.insert(entity, tag.clone());
        }
        if let Some(script) = &self.script {
            world.scripts.insert(entity, script.clone());
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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_id: String,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4], // RGBA
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
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

/// Camera for view control (Unity-like)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct World {
    next_entity: Entity,
    pub transforms: HashMap<Entity, Transform>,
    pub velocities: HashMap<Entity, (f32, f32)>,
    pub sprites: HashMap<Entity, Sprite>,
    pub colliders: HashMap<Entity, Collider>,
    pub tags: HashMap<Entity, EntityTag>,
    pub scripts: HashMap<Entity, Script>,
    pub active: HashMap<Entity, bool>,      // Active state (Unity-like)
    pub layers: HashMap<Entity, u8>,        // Layer (0-31, Unity has 32 layers)
    pub parents: HashMap<Entity, Entity>,   // Parent entity
    pub children: HashMap<Entity, Vec<Entity>>, // Children entities
    pub names: HashMap<Entity, String>,     // Entity names (for editor)
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
        self.sprites.remove(&e);
        self.colliders.remove(&e);
        self.tags.remove(&e);
        self.scripts.remove(&e);
        self.active.remove(&e);
        self.layers.remove(&e);
        self.names.remove(&e);
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.velocities.clear();
        self.sprites.clear();
        self.colliders.clear();
        self.tags.clear();
        self.scripts.clear();
        self.active.clear();
        self.layers.clear();
        self.parents.clear();
        self.children.clear();
        self.names.clear();
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

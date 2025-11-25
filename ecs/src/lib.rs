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
            transform: Transform { x: 0.0, y: 0.0, rotation: 0.0, scale: 1.0 },
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
            transform: Transform { x: 0.0, y: 0.0, rotation: 0.0, scale: 1.0 },
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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale: f32,
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
}

impl World {
    pub fn new() -> Self { Self::default() }

    pub fn spawn(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        id
    }

    pub fn despawn(&mut self, e: Entity) {
        self.transforms.remove(&e);
        self.velocities.remove(&e);
        self.sprites.remove(&e);
        self.colliders.remove(&e);
        self.tags.remove(&e);
        self.scripts.remove(&e);
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.velocities.clear();
        self.sprites.clear();
        self.colliders.clear();
        self.tags.clear();
        self.scripts.clear();
        self.next_entity = 0;
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
        }

        let data = SceneData {
            next_entity: self.next_entity,
            transforms: self.transforms.iter().map(|(k, v)| (*k, v.clone())).collect(),
            velocities: self.velocities.iter().map(|(k, v)| (*k, *v)).collect(),
            sprites: self.sprites.iter().map(|(k, v)| (*k, v.clone())).collect(),
            colliders: self.colliders.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tags: self.tags.iter().map(|(k, v)| (*k, v.clone())).collect(),
            scripts: self.scripts.iter().map(|(k, v)| (*k, v.clone())).collect(),
        };

        serde_json::to_string_pretty(&data)
    }

    pub fn load_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        #[derive(Deserialize)]
        struct SceneData {
            next_entity: Entity,
            transforms: Vec<(Entity, Transform)>,
            velocities: Vec<(Entity, (f32, f32))>,
            sprites: Vec<(Entity, Sprite)>,
            colliders: Vec<(Entity, Collider)>,
            tags: Vec<(Entity, EntityTag)>,
            scripts: Vec<(Entity, Script)>,
        }

        let data: SceneData = serde_json::from_str(json)?;

        self.clear();
        self.next_entity = data.next_entity;

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
        for (entity, tag) in data.tags {
            self.tags.insert(entity, tag);
        }
        for (entity, script) in data.scripts {
            self.scripts.insert(entity, script);
        }

        Ok(())
    }
}

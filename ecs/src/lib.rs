use std::collections::HashMap;

pub type Entity = u32;

#[derive(Default, Clone, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Sprite {
    pub texture_id: String,
    // potentially rect, color, etc.
}

#[derive(Default)]
pub struct World {
    next_entity: Entity,
    pub transforms: HashMap<Entity, Transform>,
    pub velocities: HashMap<Entity, (f32, f32)>,
    pub sprites: HashMap<Entity, Sprite>,
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
    }
    
    pub fn clear(&mut self) {
        self.transforms.clear();
        self.velocities.clear();
        self.sprites.clear();
        self.next_entity = 0;
    }
}

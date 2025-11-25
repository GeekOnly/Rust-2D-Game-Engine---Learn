use ecs::{World, Entity};

pub struct AABB {
    pub min: (f32, f32),
    pub max: (f32, f32),
}

impl AABB {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            min: (x, y),
            max: (x + w, y + h),
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.0 < other.max.0 && self.max.0 > other.min.0 &&
        self.min.1 < other.max.1 && self.max.1 > other.min.1
    }
}

pub struct PhysicsWorld {
    pub gravity: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self { Self { gravity: 980.0 } }

    pub fn step(&mut self, dt: f32, world: &mut World) {
        // Simple Euler integration
        let entities: Vec<Entity> = world.velocities.keys().cloned().collect();

        for e in entities {
            if let Some(vel) = world.velocities.get(&e) {
                if let Some(transform) = world.transforms.get_mut(&e) {
                    transform.position[0] += vel.0 * dt;
                    transform.position[1] += vel.1 * dt;
                }
            }
        }
    }

    /// Check collisions between two entities using AABB
    pub fn check_collision(world: &World, e1: Entity, e2: Entity) -> bool {
        let t1 = world.transforms.get(&e1);
        let t2 = world.transforms.get(&e2);
        let c1 = world.colliders.get(&e1);
        let c2 = world.colliders.get(&e2);

        if let (Some(t1), Some(t2), Some(c1), Some(c2)) = (t1, t2, c1, c2) {
            let aabb1 = AABB::new(t1.x() - c1.width/2.0, t1.y() - c1.height/2.0, c1.width, c1.height);
            let aabb2 = AABB::new(t2.x() - c2.width/2.0, t2.y() - c2.height/2.0, c2.width, c2.height);
            return aabb1.intersects(&aabb2);
        }
        false
    }
}

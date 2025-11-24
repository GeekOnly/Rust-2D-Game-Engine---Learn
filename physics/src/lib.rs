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
        // We need to iterate entities that have both Position (Transform) and Velocity
        // Since our ECS is simple HashMaps, we can iterate one and check the other.
        
        let mut entities: Vec<Entity> = world.velocities.keys().cloned().collect();
        
        for e in entities {
            if let Some(vel) = world.velocities.get(&e) {
                if let Some(transform) = world.transforms.get_mut(&e) {
                    transform.x += vel.0 * dt;
                    transform.y += vel.1 * dt;
                    
                    // Gravity could be applied here to velocity if we had mass/rigid body component
                    // For now, just kinematic movement
                }
            }
        }
        
        // Collision detection (naive O(N^2))
        // Placeholder for now as we don't have Collider components yet
    }
}

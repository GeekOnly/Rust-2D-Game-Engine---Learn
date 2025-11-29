//! 2D Physics System for Game Runtime
//! 
//! Provides gravity, velocity, and collision detection for 2D games

use ecs::{World, Entity};

/// Physics World - manages physics simulation
pub struct PhysicsWorld {
    pub gravity: f32,           // Gravity acceleration (pixels/s²)
    pub enabled: bool,          // Enable/disable physics
    pub time_scale: f32,        // Time scale for slow motion effects
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: 980.0,     // Unity-like gravity (9.8 m/s² = 980 pixels/s²)
            enabled: true,
            time_scale: 1.0,
        }
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update physics simulation
    pub fn step(&mut self, dt: f32, world: &mut World) {
        if !self.enabled {
            return;
        }

        let scaled_dt = dt * self.time_scale;

        // Apply gravity to all entities with Rigidbody (velocity component)
        self.apply_gravity(scaled_dt, world);

        // Update positions based on velocity
        self.update_positions(scaled_dt, world);

        // Apply world bounds to prevent objects from falling infinitely
        self.apply_world_bounds(world);

        // Check and resolve collisions
        self.check_collisions(world);
    }

    /// Apply gravity to all entities with Rigidbody
    fn apply_gravity(&self, dt: f32, world: &mut World) {
        let entities: Vec<Entity> = world.rigidbodies.keys().cloned().collect();

        for entity in entities {
            // Skip if entity is not active
            if !world.active.get(&entity).copied().unwrap_or(true) {
                continue;
            }

            if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                // Skip if kinematic (not affected by physics)
                if rigidbody.is_kinematic {
                    continue;
                }

                // Apply gravity to Y velocity with gravity scale
                rigidbody.velocity.1 -= self.gravity * rigidbody.gravity_scale * dt;

                // Sync with legacy velocity
                world.velocities.insert(entity, rigidbody.velocity);
            }
        }
    }

    /// Update positions based on velocity (Euler integration)
    fn update_positions(&self, dt: f32, world: &mut World) {
        let entities: Vec<Entity> = world.rigidbodies.keys().cloned().collect();

        for entity in entities {
            // Skip if entity is not active
            if !world.active.get(&entity).copied().unwrap_or(true) {
                continue;
            }

            if let Some(rigidbody) = world.rigidbodies.get(&entity) {
                // Skip if kinematic (controlled manually)
                if rigidbody.is_kinematic {
                    continue;
                }

                if let Some(transform) = world.transforms.get_mut(&entity) {
                    // Update position
                    transform.position[0] += rigidbody.velocity.0 * dt;
                    transform.position[1] += rigidbody.velocity.1 * dt;
                }
            }
        }
    }

    /// Apply world bounds to prevent objects from falling infinitely
    fn apply_world_bounds(&self, world: &mut World) {
        // Define world bounds (can be made configurable later)
        let min_y = -100.0;  // Bottom bound
        let max_y = 100.0;   // Top bound (optional)
        let min_x = -100.0;  // Left bound (optional)
        let max_x = 100.0;   // Right bound (optional)

        let entities: Vec<Entity> = world.rigidbodies.keys().cloned().collect();

        for entity in entities {
            if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                // Skip kinematic bodies
                if rigidbody.is_kinematic {
                    continue;
                }

                if let Some(transform) = world.transforms.get_mut(&entity) {
                    let mut clamped = false;

                    // Clamp Y position (prevent falling infinitely)
                    if transform.position[1] < min_y {
                        transform.position[1] = min_y;
                        // Stop downward velocity
                        if rigidbody.velocity.1 < 0.0 {
                            rigidbody.velocity.1 = 0.0;
                            world.velocities.insert(entity, rigidbody.velocity);
                        }
                        clamped = true;
                    } else if transform.position[1] > max_y {
                        transform.position[1] = max_y;
                        // Stop upward velocity
                        if rigidbody.velocity.1 > 0.0 {
                            rigidbody.velocity.1 = 0.0;
                            world.velocities.insert(entity, rigidbody.velocity);
                        }
                        clamped = true;
                    }

                    // Clamp X position (optional side bounds)
                    if transform.position[0] < min_x {
                        transform.position[0] = min_x;
                        // Stop leftward velocity
                        if rigidbody.velocity.0 < 0.0 {
                            rigidbody.velocity.0 = 0.0;
                            world.velocities.insert(entity, rigidbody.velocity);
                        }
                        clamped = true;
                    } else if transform.position[0] > max_x {
                        transform.position[0] = max_x;
                        // Stop rightward velocity
                        if rigidbody.velocity.0 > 0.0 {
                            rigidbody.velocity.0 = 0.0;
                            world.velocities.insert(entity, rigidbody.velocity);
                        }
                        clamped = true;
                    }

                    // Optional: Add bounce effect
                    // if clamped {
                    //     rigidbody.velocity.0 *= -0.5; // Bounce with 50% energy loss
                    //     rigidbody.velocity.1 *= -0.5;
                    // }
                }
            }
        }
    }

    /// Check collisions between all entities with colliders and resolve them
    fn check_collisions(&self, world: &mut World) {
        let entities_with_colliders: Vec<Entity> = world.colliders.keys().cloned().collect();

        // Simple O(n²) collision detection and response
        for i in 0..entities_with_colliders.len() {
            for j in (i + 1)..entities_with_colliders.len() {
                let e1 = entities_with_colliders[i];
                let e2 = entities_with_colliders[j];

                // Skip if either entity is not active
                if !world.active.get(&e1).copied().unwrap_or(true) ||
                   !world.active.get(&e2).copied().unwrap_or(true) {
                    continue;
                }

                if Self::check_collision(world, e1, e2) {
                    // Collision detected - resolve it
                    Self::resolve_collision(world, e1, e2);
                }
            }
        }
    }

    /// Resolve collision between two entities (separate them)
    fn resolve_collision(world: &mut World, e1: Entity, e2: Entity) {
        // Get collision data
        let t1 = world.transforms.get(&e1).cloned();
        let t2 = world.transforms.get(&e2).cloned();
        let c1 = world.colliders.get(&e1).cloned();
        let c2 = world.colliders.get(&e2).cloned();

        if let (Some(t1), Some(t2), Some(c1), Some(c2)) = (t1, t2, c1, c2) {
            // Calculate overlap
            let x1 = t1.position[0];
            let y1 = t1.position[1];
            let x2 = t2.position[0];
            let y2 = t2.position[1];

            // Calculate centers
            let center1_x = x1;
            let center1_y = y1;
            let center2_x = x2;
            let center2_y = y2;

            // Calculate overlap on each axis
            let overlap_x = (c1.width / 2.0 + c2.width / 2.0) - (center1_x - center2_x).abs();
            let overlap_y = (c1.height / 2.0 + c2.height / 2.0) - (center1_y - center2_y).abs();

            // Determine which axis has less overlap (separate on that axis)
            let has_rigidbody1 = world.rigidbodies.contains_key(&e1);
            let has_rigidbody2 = world.rigidbodies.contains_key(&e2);
            
            // Check if kinematic
            let is_kinematic1 = world.rigidbodies.get(&e1).map(|rb| rb.is_kinematic).unwrap_or(false);
            let is_kinematic2 = world.rigidbodies.get(&e2).map(|rb| rb.is_kinematic).unwrap_or(false);

            if overlap_x < overlap_y {
                // Separate on X axis
                let direction = if center1_x < center2_x { -1.0 } else { 1.0 };
                
                if has_rigidbody1 && has_rigidbody2 && !is_kinematic1 && !is_kinematic2 {
                    // Both have rigidbody and not kinematic - push both
                    let push = overlap_x / 2.0;
                    if let Some(transform) = world.transforms.get_mut(&e1) {
                        transform.position[0] += direction * push;
                    }
                    if let Some(transform) = world.transforms.get_mut(&e2) {
                        transform.position[0] -= direction * push;
                    }
                    
                    // Stop velocity on collision axis
                    if let Some(rb) = world.rigidbodies.get_mut(&e1) {
                        if direction > 0.0 && rb.velocity.0 < 0.0 || direction < 0.0 && rb.velocity.0 > 0.0 {
                            rb.velocity.0 = 0.0;
                        }
                        world.velocities.insert(e1, rb.velocity);
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e2) {
                        if direction > 0.0 && rb.velocity.0 > 0.0 || direction < 0.0 && rb.velocity.0 < 0.0 {
                            rb.velocity.0 = 0.0;
                        }
                        world.velocities.insert(e2, rb.velocity);
                    }
                } else if has_rigidbody1 && !is_kinematic1 {
                    // Only e1 has rigidbody and not kinematic - push e1 only
                    if let Some(transform) = world.transforms.get_mut(&e1) {
                        transform.position[0] += direction * overlap_x;
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e1) {
                        if direction > 0.0 && rb.velocity.0 < 0.0 || direction < 0.0 && rb.velocity.0 > 0.0 {
                            rb.velocity.0 = 0.0;
                        }
                        world.velocities.insert(e1, rb.velocity);
                    }
                } else if has_rigidbody2 && !is_kinematic2 {
                    // Only e2 has rigidbody and not kinematic - push e2 only
                    if let Some(transform) = world.transforms.get_mut(&e2) {
                        transform.position[0] -= direction * overlap_x;
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e2) {
                        if direction > 0.0 && rb.velocity.0 > 0.0 || direction < 0.0 && rb.velocity.0 < 0.0 {
                            rb.velocity.0 = 0.0;
                        }
                        world.velocities.insert(e2, rb.velocity);
                    }
                }
            } else {
                // Separate on Y axis
                let direction = if center1_y < center2_y { -1.0 } else { 1.0 };
                
                if has_rigidbody1 && has_rigidbody2 && !is_kinematic1 && !is_kinematic2 {
                    // Both have rigidbody and not kinematic - push both
                    let push = overlap_y / 2.0;
                    if let Some(transform) = world.transforms.get_mut(&e1) {
                        transform.position[1] += direction * push;
                    }
                    if let Some(transform) = world.transforms.get_mut(&e2) {
                        transform.position[1] -= direction * push;
                    }
                    
                    // Stop velocity on collision axis
                    if let Some(rb) = world.rigidbodies.get_mut(&e1) {
                        if direction > 0.0 && rb.velocity.1 < 0.0 || direction < 0.0 && rb.velocity.1 > 0.0 {
                            rb.velocity.1 = 0.0;
                        }
                        world.velocities.insert(e1, rb.velocity);
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e2) {
                        if direction > 0.0 && rb.velocity.1 > 0.0 || direction < 0.0 && rb.velocity.1 < 0.0 {
                            rb.velocity.1 = 0.0;
                        }
                        world.velocities.insert(e2, rb.velocity);
                    }
                } else if has_rigidbody1 && !is_kinematic1 {
                    // Only e1 has rigidbody and not kinematic - push e1 only
                    if let Some(transform) = world.transforms.get_mut(&e1) {
                        transform.position[1] += direction * overlap_y;
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e1) {
                        if direction > 0.0 && rb.velocity.1 < 0.0 || direction < 0.0 && rb.velocity.1 > 0.0 {
                            rb.velocity.1 = 0.0;
                        }
                        world.velocities.insert(e1, rb.velocity);
                    }
                } else if has_rigidbody2 && !is_kinematic2 {
                    // Only e2 has rigidbody and not kinematic - push e2 only
                    if let Some(transform) = world.transforms.get_mut(&e2) {
                        transform.position[1] -= direction * overlap_y;
                    }
                    if let Some(rb) = world.rigidbodies.get_mut(&e2) {
                        if direction > 0.0 && rb.velocity.1 > 0.0 || direction < 0.0 && rb.velocity.1 < 0.0 {
                            rb.velocity.1 = 0.0;
                        }
                        world.velocities.insert(e2, rb.velocity);
                    }
                }
            }
        }
    }

    /// Check collision between two entities using AABB
    pub fn check_collision(world: &World, e1: Entity, e2: Entity) -> bool {
        let t1 = world.transforms.get(&e1);
        let t2 = world.transforms.get(&e2);
        let c1 = world.colliders.get(&e1);
        let c2 = world.colliders.get(&e2);

        if let (Some(t1), Some(t2), Some(c1), Some(c2)) = (t1, t2, c1, c2) {
            // Calculate AABB bounds
            let x1 = t1.position[0] - c1.width / 2.0;
            let y1 = t1.position[1] - c1.height / 2.0;
            let x2 = t2.position[0] - c2.width / 2.0;
            let y2 = t2.position[1] - c2.height / 2.0;

            // AABB collision test
            let collision = x1 < x2 + c2.width &&
                           x1 + c1.width > x2 &&
                           y1 < y2 + c2.height &&
                           y1 + c1.height > y2;

            return collision;
        }
        false
    }

    /// Get all entities colliding with a specific entity
    pub fn get_collisions(world: &World, entity: Entity) -> Vec<Entity> {
        let mut collisions = Vec::new();

        if !world.colliders.contains_key(&entity) {
            return collisions;
        }

        for (other_entity, _) in &world.colliders {
            if *other_entity != entity {
                if Self::check_collision(world, entity, *other_entity) {
                    collisions.push(*other_entity);
                }
            }
        }

        collisions
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    /// Enable/disable physics
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set time scale (for slow motion effects)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }
}

/// Physics helper functions
pub mod helpers {
    use ecs::{World, Entity};

    /// Apply impulse to an entity (instant velocity change)
    pub fn apply_impulse(world: &mut World, entity: Entity, impulse_x: f32, impulse_y: f32) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 += impulse_x;
            velocity.1 += impulse_y;
        }
    }

    /// Set velocity directly
    pub fn set_velocity(world: &mut World, entity: Entity, vel_x: f32, vel_y: f32) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 = vel_x;
            velocity.1 = vel_y;
        }
    }

    /// Get velocity
    pub fn get_velocity(world: &World, entity: Entity) -> Option<(f32, f32)> {
        world.velocities.get(&entity).copied()
    }

    /// Stop entity movement
    pub fn stop(world: &mut World, entity: Entity) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 = 0.0;
            velocity.1 = 0.0;
        }
    }

    /// Apply force (continuous acceleration)
    pub fn apply_force(world: &mut World, entity: Entity, force_x: f32, force_y: f32, dt: f32) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 += force_x * dt;
            velocity.1 += force_y * dt;
        }
    }

    /// Clamp velocity to max speed
    pub fn clamp_velocity(world: &mut World, entity: Entity, max_speed: f32) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
            if speed > max_speed {
                let scale = max_speed / speed;
                velocity.0 *= scale;
                velocity.1 *= scale;
            }
        }
    }

    /// Apply friction/damping
    pub fn apply_damping(world: &mut World, entity: Entity, damping: f32, dt: f32) {
        if let Some(velocity) = world.velocities.get_mut(&entity) {
            let factor = 1.0 - (damping * dt).min(1.0);
            velocity.0 *= factor;
            velocity.1 *= factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::{World, ComponentType, ComponentManager};

    #[test]
    fn test_gravity_application() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        let entity = world.spawn();
        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Rigidbody).unwrap();

        // Initial velocity
        world.velocities.insert(entity, (0.0, 0.0));

        // Apply physics for 1 second
        physics.step(1.0, &mut world);

        // Check that gravity was applied
        let velocity = world.velocities.get(&entity).unwrap();
        assert!(velocity.1 < 0.0, "Gravity should pull down (negative Y)");
    }

    #[test]
    fn test_position_update() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        physics.gravity = 0.0; // Disable gravity for this test

        let entity = world.spawn();
        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Rigidbody).unwrap();

        // Set initial position and velocity
        world.transforms.get_mut(&entity).unwrap().position = [0.0, 0.0, 0.0];
        world.velocities.insert(entity, (100.0, 50.0));

        // Update for 1 second
        physics.step(1.0, &mut world);

        // Check position changed
        let transform = world.transforms.get(&entity).unwrap();
        assert_eq!(transform.position[0], 100.0);
        assert_eq!(transform.position[1], 50.0);
    }

    #[test]
    fn test_collision_detection() {
        let mut world = World::new();

        // Create two entities with colliders
        let e1 = world.spawn();
        world.add_component(e1, ComponentType::Transform).unwrap();
        world.add_component(e1, ComponentType::BoxCollider).unwrap();
        world.transforms.get_mut(&e1).unwrap().position = [0.0, 0.0, 0.0];
        world.colliders.get_mut(&e1).unwrap().width = 32.0;
        world.colliders.get_mut(&e1).unwrap().height = 32.0;

        let e2 = world.spawn();
        world.add_component(e2, ComponentType::Transform).unwrap();
        world.add_component(e2, ComponentType::BoxCollider).unwrap();
        world.transforms.get_mut(&e2).unwrap().position = [10.0, 10.0, 0.0];
        world.colliders.get_mut(&e2).unwrap().width = 32.0;
        world.colliders.get_mut(&e2).unwrap().height = 32.0;

        // Should collide (overlapping)
        assert!(PhysicsWorld::check_collision(&world, e1, e2));

        // Move e2 far away
        world.transforms.get_mut(&e2).unwrap().position = [100.0, 100.0, 0.0];

        // Should not collide
        assert!(!PhysicsWorld::check_collision(&world, e1, e2));
    }

    #[test]
    fn test_physics_helpers() {
        let mut world = World::new();

        let entity = world.spawn();
        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Rigidbody).unwrap();

        // Test apply impulse
        helpers::apply_impulse(&mut world, entity, 10.0, 20.0);
        let vel = helpers::get_velocity(&world, entity).unwrap();
        assert_eq!(vel, (10.0, 20.0));

        // Test set velocity
        helpers::set_velocity(&mut world, entity, 5.0, 15.0);
        let vel = helpers::get_velocity(&world, entity).unwrap();
        assert_eq!(vel, (5.0, 15.0));

        // Test stop
        helpers::stop(&mut world, entity);
        let vel = helpers::get_velocity(&world, entity).unwrap();
        assert_eq!(vel, (0.0, 0.0));
    }
}

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

        // Debug: Log entities with rigidbodies
        let rb_count = world.rigidbodies.len();
        if rb_count > 0 {
            log::debug!("Physics step: {} rigidbodies, dt={:.4}", rb_count, scaled_dt);
            for (entity, rb) in &world.rigidbodies {
                log::trace!("  Entity {}: vel={:?}, kinematic={}, gravity_scale={}", 
                    entity, rb.velocity, rb.is_kinematic, rb.gravity_scale);
            }
        }

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
            let is_active = world.active.get(&entity).copied().unwrap_or(true);
            if !is_active {
                log::trace!("Entity {} skipped (not active)", entity);
                continue;
            }

            if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                // Skip if kinematic (not affected by physics)
                if rigidbody.is_kinematic {
                    log::trace!("Entity {} skipped (kinematic)", entity);
                    continue;
                }

                let old_vel = rigidbody.velocity.1;
                // Apply gravity to Y velocity with gravity scale
                rigidbody.velocity.1 -= self.gravity * rigidbody.gravity_scale * dt;
                log::debug!("Entity {}: gravity applied, vel.y {} -> {}", entity, old_vel, rigidbody.velocity.1);
            }

            // Sync rigidbody velocity to world velocities (after mutable borrow ends)
            if let Some(rigidbody) = world.rigidbodies.get(&entity) {
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

            // Get velocity from rigidbody
            let velocity = if let Some(rigidbody) = world.rigidbodies.get(&entity) {
                // Skip if kinematic (controlled manually)
                if rigidbody.is_kinematic {
                    continue;
                }
                rigidbody.velocity
            } else {
                continue;
            };

            // Update position using the velocity
            if let Some(transform) = world.transforms.get_mut(&entity) {
                transform.position[0] += velocity.0 * dt;
                transform.position[1] += velocity.1 * dt;
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

        log::trace!("World bounds: x=[{}, {}], y=[{}, {}]", min_x, max_x, min_y, max_y);

        let entities: Vec<Entity> = world.rigidbodies.keys().cloned().collect();

        for entity in entities {
            // Check if kinematic first
            let is_kinematic = world.rigidbodies.get(&entity)
                .map(|rb| rb.is_kinematic)
                .unwrap_or(true);
            
            if is_kinematic {
                continue;
            }

            // Get current position
            let position = if let Some(transform) = world.transforms.get(&entity) {
                transform.position
            } else {
                continue;
            };

            let mut needs_clamp = false;
            let mut new_velocity = world.rigidbodies.get(&entity)
                .map(|rb| rb.velocity)
                .unwrap_or((0.0, 0.0));

            // Check and adjust Y bounds
            if position[1] < min_y {
                log::warn!("Entity {} hit bottom bound: pos.y={:.2} < {:.2}, vel={:?}", entity, position[1], min_y, new_velocity);
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.position[1] = min_y;
                }
                if new_velocity.1 < 0.0 {
                    new_velocity.1 = 0.0;
                    needs_clamp = true;
                }
            } else if position[1] > max_y {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.position[1] = max_y;
                }
                if new_velocity.1 > 0.0 {
                    new_velocity.1 = 0.0;
                    needs_clamp = true;
                }
            }

            // Check and adjust X bounds
            if position[0] < min_x {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.position[0] = min_x;
                }
                if new_velocity.0 < 0.0 {
                    new_velocity.0 = 0.0;
                    needs_clamp = true;
                }
            } else if position[0] > max_x {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.position[0] = max_x;
                }
                if new_velocity.0 > 0.0 {
                    new_velocity.0 = 0.0;
                    needs_clamp = true;
                }
            }

            // Update velocity if clamped
            if needs_clamp {
                if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = new_velocity;
                }
                world.velocities.insert(entity, new_velocity);
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
            // Get world-space collider dimensions
            let width1 = c1.get_world_width(t1.scale[0]);
            let height1 = c1.get_world_height(t1.scale[1]);
            let width2 = c2.get_world_width(t2.scale[0]);
            let height2 = c2.get_world_height(t2.scale[1]);
            
            // Get world-space offsets
            let offset1 = c1.get_world_offset(t1.scale[0], t1.scale[1]);
            let offset2 = c2.get_world_offset(t2.scale[0], t2.scale[1]);
            
            // Calculate centers (position + offset)
            let center1_x = t1.position[0] + offset1[0];
            let center1_y = t1.position[1] + offset1[1];
            let center2_x = t2.position[0] + offset2[0];
            let center2_y = t2.position[1] + offset2[1];

            // Calculate overlap on each axis
            let overlap_x = (width1 / 2.0 + width2 / 2.0) - (center1_x - center2_x).abs();
            let overlap_y = (height1 / 2.0 + height2 / 2.0) - (center1_y - center2_y).abs();

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
            // Get world-space collider dimensions (size * transform.scale)
            let width1 = c1.get_world_width(t1.scale[0]);
            let height1 = c1.get_world_height(t1.scale[1]);
            let width2 = c2.get_world_width(t2.scale[0]);
            let height2 = c2.get_world_height(t2.scale[1]);
            
            // Get world-space offsets
            let offset1 = c1.get_world_offset(t1.scale[0], t1.scale[1]);
            let offset2 = c2.get_world_offset(t2.scale[0], t2.scale[1]);
            
            // Calculate AABB bounds (with offset)
            let x1 = t1.position[0] + offset1[0] - width1 / 2.0;
            let y1 = t1.position[1] + offset1[1] - height1 / 2.0;
            let x2 = t2.position[0] + offset2[0] - width2 / 2.0;
            let y2 = t2.position[1] + offset2[1] - height2 / 2.0;

            // AABB collision test
            let collision = x1 < x2 + width2 &&
                           x1 + width1 > x2 &&
                           y1 < y2 + height2 &&
                           y1 + height1 > y2;

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

        for other_entity in world.colliders.keys() {
            if *other_entity != entity
                && Self::check_collision(world, entity, *other_entity) {
                    collisions.push(*other_entity);
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
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity.0 += impulse_x;
            rigidbody.velocity.1 += impulse_y;
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 += impulse_x;
            velocity.1 += impulse_y;
        }
    }

    /// Set velocity directly
    pub fn set_velocity(world: &mut World, entity: Entity, vel_x: f32, vel_y: f32) {
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity.0 = vel_x;
            rigidbody.velocity.1 = vel_y;
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 = vel_x;
            velocity.1 = vel_y;
        }
    }

    /// Get velocity
    pub fn get_velocity(world: &World, entity: Entity) -> Option<(f32, f32)> {
        world.rigidbodies.get(&entity)
            .map(|rb| rb.velocity)
            .or_else(|| world.velocities.get(&entity).copied())
    }

    /// Stop entity movement
    pub fn stop(world: &mut World, entity: Entity) {
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity.0 = 0.0;
            rigidbody.velocity.1 = 0.0;
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 = 0.0;
            velocity.1 = 0.0;
        }
    }

    /// Apply force (continuous acceleration)
    pub fn apply_force(world: &mut World, entity: Entity, force_x: f32, force_y: f32, dt: f32) {
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity.0 += force_x * dt;
            rigidbody.velocity.1 += force_y * dt;
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
            velocity.0 += force_x * dt;
            velocity.1 += force_y * dt;
        }
    }

    /// Clamp velocity to max speed
    pub fn clamp_velocity(world: &mut World, entity: Entity, max_speed: f32) {
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            let speed = (rigidbody.velocity.0 * rigidbody.velocity.0 + rigidbody.velocity.1 * rigidbody.velocity.1).sqrt();
            if speed > max_speed {
                let scale = max_speed / speed;
                rigidbody.velocity.0 *= scale;
                rigidbody.velocity.1 *= scale;
            }
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
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
        let factor = 1.0 - (damping * dt).min(1.0);
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity.0 *= factor;
            rigidbody.velocity.1 *= factor;
            world.velocities.insert(entity, rigidbody.velocity);
        } else if let Some(velocity) = world.velocities.get_mut(&entity) {
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

        // Initial velocity - set in rigidbody
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity = (0.0, 0.0);
        }
        world.velocities.insert(entity, (0.0, 0.0));

        // Apply physics for a small time step (0.016s ~= 60fps)
        // Using small dt to avoid hitting world bounds
        physics.step(0.016, &mut world);

        // Check that gravity was applied
        let velocity = world.velocities.get(&entity).unwrap();
        assert!(velocity.1 < 0.0, "Gravity should pull down (negative Y), got: {:?}", velocity);
        
        // Verify the velocity is approximately correct
        // Expected: -980 * 0.016 = -15.68
        assert!((velocity.1 + 15.68).abs() < 0.1, "Velocity should be approximately -15.68, got: {}", velocity.1);
    }

    #[test]
    fn test_position_update() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        physics.gravity = 0.0; // Disable gravity for this test

        let entity = world.spawn();
        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Rigidbody).unwrap();

        // Set initial position and velocity - set in rigidbody
        world.transforms.get_mut(&entity).unwrap().position = [0.0, 0.0, 0.0];
        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
            rigidbody.velocity = (100.0, 50.0);
        }
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

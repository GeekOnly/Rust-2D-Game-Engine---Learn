//! Rapier Physics Backend - Production-ready 2D physics
//! 
//! This backend uses Rapier2D for robust collision detection and physics simulation
//! 
//! ## Axis Mapping
//! The engine uses a 3D coordinate system (X, Y, Z) where:
//! - X = horizontal (left/right)
//! - Y = vertical (up/down)
//! - Z = depth (into/out of screen) - unused in 2D physics
//! 
//! Rapier2D uses a 2D coordinate system (X, Y) where:
//! - X = horizontal
//! - Y = vertical (positive = down)
//! 
//! This backend maps:
//! - Engine X → Rapier X (horizontal)
//! - Engine Y → Rapier Y (vertical)
//! - Engine Z → ignored (depth is not simulated)

use ecs::{World, Entity};
use rapier2d::prelude::*;
use std::collections::HashMap;

/// Physics World using Rapier
pub struct RapierPhysicsWorld {
    pub gravity: Vector<Real>,
    pub enabled: bool,
    pub time_scale: f32,
    
    // Rapier components
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    
    // Mapping between ECS entities and Rapier handles
    entity_to_body: HashMap<Entity, RigidBodyHandle>,
    body_to_entity: HashMap<RigidBodyHandle, Entity>,
}

impl Default for RapierPhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl RapierPhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, 150.0], // Positive Y is down in Rapier
            enabled: true,
            time_scale: 1.0,
            
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
        }
    }
    
    /// Sync ECS world to Rapier world
    pub fn sync_from_ecs(&mut self, world: &World) {
        // Add/update rigid bodies from ECS
        for (entity, rigidbody) in &world.rigidbodies {
            if !self.entity_to_body.contains_key(entity) {
                // Create new rigid body
                let rb_type = if rigidbody.is_kinematic {
                    RigidBodyType::KinematicPositionBased
                } else {
                    RigidBodyType::Dynamic
                };
                
                // Map 3D position to 2D: X stays X, Y inverted
                // Engine: -Y=down, Rapier: +Y=down, so negate Y
                let position = world.transforms.get(entity)
                    .map(|t| vector![t.position[0], -t.position[1]])  // Negate Y
                    .unwrap_or(vector![0.0, 0.0]);
                
                let rigid_body = RigidBodyBuilder::new(rb_type)
                    .translation(position)
                    .linvel(vector![rigidbody.velocity.0, -rigidbody.velocity.1])  // Negate Y velocity
                    .gravity_scale(rigidbody.gravity_scale)
                    .ccd_enabled(true) // Enable continuous collision detection
                    .build();
                
                let handle = self.rigid_body_set.insert(rigid_body);
                self.entity_to_body.insert(*entity, handle);
                self.body_to_entity.insert(handle, *entity);
                
                log::info!("🔧 Rapier: Created rigidbody for entity {}, type={:?}, pos=({:.2}, {:.2})", 
                    entity, rb_type, position.x, position.y);
                
                // Add collider if exists
                if let Some(collider) = world.colliders.get(entity) {
                    let transform = world.transforms.get(entity).unwrap();
                    // Map 3D to 2D: width uses scale[0] (X), height uses scale[1] (Y)
                    let half_width = collider.get_world_width(transform.scale[0]) / 2.0;
                    let half_height = collider.get_world_height(transform.scale[1]) / 2.0;
                    let offset = collider.get_world_offset(transform.scale[0], transform.scale[1]);
                    
                    log::info!("🔧 Rapier: Creating collider for entity {}, half_size=({:.2}, {:.2}), offset=({:.2}, {:.2})", 
                        entity, half_width, half_height, offset[0], offset[1]);
                    
                    let collider_shape = ColliderBuilder::cuboid(half_width, half_height)
                        .translation(vector![offset[0], -offset[1]]) // Negate Y offset for Rapier
                        .friction(0.0) // No friction for platformer
                        .restitution(0.0) // No bounce
                        .build();
                    
                    self.collider_set.insert_with_parent(collider_shape, handle, &mut self.rigid_body_set);
                    log::info!("✅ Rapier: Collider created for entity {}", entity);
                } else {
                    log::warn!("⚠️ Rapier: Entity {} has rigidbody but NO collider!", entity);
                }
            } else {
                // Update existing rigid body velocity only (don't update position - let Rapier handle it)
                let handle = self.entity_to_body[entity];
                if let Some(rb) = self.rigid_body_set.get_mut(handle) {
                    // Only update velocity if it changed significantly
                    let current_vel = rb.linvel();
                    let new_vel = vector![rigidbody.velocity.0, -rigidbody.velocity.1];
                    if (current_vel.x - new_vel.x).abs() > 0.01 || (current_vel.y - new_vel.y).abs() > 0.01 {
                        rb.set_linvel(new_vel, true);
                    }
                }
            }
        }
    }
    
    /// Sync Rapier world back to ECS
    pub fn sync_to_ecs(&self, world: &mut World) {
        for (handle, entity) in &self.body_to_entity {
            if let Some(rb) = self.rigid_body_set.get(*handle) {
                // Update transform: Map 2D back to 3D (X stays X, Y inverted)
                if let Some(transform) = world.transforms.get_mut(entity) {
                    let translation = rb.translation();
                    let old_y = transform.position[1];
                    transform.position[0] = translation.x;
                    transform.position[1] = -translation.y; // Negate Y back to engine convention
                    // Keep Z (depth) unchanged
                    
                    // Debug: log significant Y changes
                    if (old_y - (-translation.y)).abs() > 0.5 {
                        log::info!("📍 Entity {} Y changed: {:.2} -> {:.2}", entity, old_y, -translation.y);
                    }
                }
                
                // Update velocity: Map 2D velocity back (negate Y)
                let linvel = rb.linvel();
                log::debug!("🔧 Entity {} velocity: ({:.2}, {:.2})", entity, linvel.x, -linvel.y);
                if let Some(rigidbody) = world.rigidbodies.get_mut(entity) {
                    rigidbody.velocity = (linvel.x, -linvel.y); // Negate Y velocity back to engine convention
                }
                world.velocities.insert(*entity, (linvel.x, -linvel.y));
            }
        }
    }
    
    /// Physics step
    pub fn step(&mut self, dt: f32, world: &mut World) {
        if !self.enabled {
            log::warn!("⚠️ Rapier: Physics disabled!");
            return;
        }
        
        let scaled_dt = dt * self.time_scale;
        
        // Sync from ECS to Rapier
        self.sync_from_ecs(world);
        
        log::info!("🔧 Rapier: Running physics step, dt={:.4}, bodies={}, colliders={}", 
            scaled_dt, self.rigid_body_set.len(), self.collider_set.len());
        
        // Run physics simulation
        self.integration_parameters.dt = scaled_dt;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
        
        // Update query pipeline for raycasts (Rapier 0.22 only needs colliders)
        self.query_pipeline.update(&self.collider_set);
        
        // Log collision info
        let contact_count = self.narrow_phase.contact_pairs().count();
        if contact_count > 0 {
            log::info!("🔧 Rapier: {} contact pairs after physics step", contact_count);
        }
        
        // Sync back to ECS
        self.sync_to_ecs(world);
    }
    
    /// Check if entity is grounded (touching ground below)
    pub fn is_grounded(&self, entity: Entity, _world: &World) -> bool {
        log::info!("🔍 is_grounded called for entity {}", entity);
        
        if let Some(body_handle) = self.entity_to_body.get(&entity) {
            log::info!("  ✅ Found body_handle for entity {}", entity);
            
            // Get all colliders attached to this rigid body
            if let Some(rb) = self.rigid_body_set.get(*body_handle) {
                let collider_count = rb.colliders().len();
                log::info!("  ✅ Entity {} has {} colliders", entity, collider_count);
                
                if collider_count == 0 {
                    log::warn!("  ⚠️ Entity {} has NO colliders!", entity);
                    return false;
                }
                
                for collider_handle in rb.colliders() {
                    // Check contacts for this collider
                    let mut contact_count = 0;
                    for contact_pair in self.narrow_phase.contact_pairs_with(*collider_handle) {
                        contact_count += 1;
                        let (h1, h2) = (contact_pair.collider1, contact_pair.collider2);
                        
                        log::info!("🔍 Entity {} contact pair: h1={:?}, h2={:?}, active={}", 
                            entity, h1, h2, contact_pair.has_any_active_contact);
                        
                        // Check if contact is active
                        if !contact_pair.has_any_active_contact {
                            log::info!("  ⚠️ Contact not active, skipping");
                            continue;
                        }
                        
                        // Check contact manifolds
                        for manifold in &contact_pair.manifolds {
                            // manifold.local_n1 always points from collider1 to collider2
                            // We want the normal pointing FROM the other object TO our object
                            // If we are collider1, the normal points away from us (to collider2)
                            // If we are collider2, the normal points away from collider1 (towards us)
                            let normal = if h1 == *collider_handle {
                                -manifold.local_n1  // We are collider1, flip to get normal pointing TO us
                            } else {
                                manifold.local_n1   // We are collider2, normal already points TO us
                            };
                            
                            log::info!("  🔍 Entity {} contact: we_are={}, normal=({:.3}, {:.3}), local_n1=({:.3}, {:.3})", 
                                entity, 
                                if h1 == *collider_handle { "h1" } else { "h2" },
                                normal.x, normal.y, 
                                manifold.local_n1.x, manifold.local_n1.y);
                            
                            // In Rapier: +Y=down, gravity pulls down (+Y), ground pushes up (-Y)
                            // So ground contact normals point up (negative Y in Rapier)
                            // Allow some tolerance (not exactly vertical)
                            // Check both the normal and its inverse to handle both cases
                            if normal.y < -0.5 || normal.y > 0.5 {
                                // This is a vertical contact
                                if normal.y < -0.5 {
                                    log::info!("  ✅ Ground contact detected for entity {}: normal.y = {:.3} (upward)", entity, normal.y);
                                    return true;
                                } else {
                                    log::info!("  ⚠️ Ceiling contact: normal.y = {:.3} (downward)", normal.y);
                                }
                            } else {
                                log::info!("  ❌ Not vertical contact: normal.y = {:.3}", normal.y);
                            }
                        }
                    }
                    
                    if contact_count > 0 {
                        log::info!("🔍 Entity {} has {} contact pairs but none are ground contacts", entity, contact_count);
                    } else {
                        log::info!("🔍 Entity {} has NO contact pairs", entity);
                    }
                }
            } else {
                log::warn!("⚠️ Entity {} has body handle but no rigidbody in set!", entity);
            }
        } else {
            log::warn!("⚠️ Entity {} has no body handle!", entity);
        }
        false
    }
    
    /// Raycast downward to check ground
    pub fn raycast_ground(&self, entity: Entity, world: &World, distance: f32) -> bool {
        if let Some(transform) = world.transforms.get(&entity) {
            // Map 3D to 2D: X stays X, Y inverted
            let ray_origin = point![transform.position[0], -transform.position[1]];  // Negate Y
            let ray_dir = vector![0.0, 1.0]; // Down in Rapier (positive Y)
            let max_toi = distance;
            
            let filter = QueryFilter::default();
            
            if let Some(_hit) = self.query_pipeline.cast_ray(
                &self.rigid_body_set,
                &self.collider_set,
                &Ray::new(ray_origin, ray_dir),
                max_toi,
                true,
                filter,
            ) {
                return true;
            }
        }
        false
    }
    
    /// Set gravity
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = vector![0.0, gravity]; // Positive Y is down in Rapier
    }
    
    /// Enable/disable physics
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Set time scale
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }
}

/// Helper functions for Rapier backend
pub mod helpers {
    use super::*;
    use ecs::{World, Entity};
    
    /// Apply impulse to entity
    /// impulse_x: horizontal impulse (X axis)
    /// impulse_y: vertical impulse (Y axis, engine convention: negative=down)
    pub fn apply_impulse(physics: &mut RapierPhysicsWorld, world: &mut World, entity: Entity, impulse_x: f32, impulse_y: f32) {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get_mut(*handle) {
                rb.apply_impulse(vector![impulse_x, -impulse_y], true);  // Negate Y for Rapier
                
                // Sync back to ECS: negate Y velocity back to engine convention
                let linvel = rb.linvel();
                if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = (linvel.x, -linvel.y);
                }
                world.velocities.insert(entity, (linvel.x, -linvel.y));
            }
        }
    }
    
    /// Set velocity directly
    /// vel_x: horizontal velocity (X axis)
    /// vel_y: vertical velocity (Y axis, engine convention: negative=down)
    pub fn set_velocity(physics: &mut RapierPhysicsWorld, world: &mut World, entity: Entity, vel_x: f32, vel_y: f32) {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get_mut(*handle) {
                rb.set_linvel(vector![vel_x, -vel_y], true);  // Negate Y for Rapier
                
                // Sync back to ECS: keep engine convention
                if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = (vel_x, vel_y);
                }
                world.velocities.insert(entity, (vel_x, vel_y));
            }
        }
    }
    
    /// Get velocity
    /// Returns (vx, vy) where vx is horizontal and vy is vertical (engine convention)
    pub fn get_velocity(physics: &RapierPhysicsWorld, entity: Entity) -> Option<(f32, f32)> {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get(*handle) {
                let linvel = rb.linvel();
                return Some((linvel.x, -linvel.y)); // Negate Y back to engine convention
            }
        }
        None
    }
}

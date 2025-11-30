//! Rapier Physics Backend - Production-ready 2D physics
//! 
//! This backend uses Rapier2D for robust collision detection and physics simulation

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
            gravity: vector![0.0, 150.0], // Positive Y is down
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
                
                let position = world.transforms.get(entity)
                    .map(|t| vector![t.position[0], t.position[1]])
                    .unwrap_or(vector![0.0, 0.0]);
                
                let rigid_body = RigidBodyBuilder::new(rb_type)
                    .translation(position)
                    .linvel(vector![rigidbody.velocity.0, rigidbody.velocity.1])
                    .gravity_scale(rigidbody.gravity_scale)
                    .ccd_enabled(true) // Enable continuous collision detection
                    .build();
                
                let handle = self.rigid_body_set.insert(rigid_body);
                self.entity_to_body.insert(*entity, handle);
                self.body_to_entity.insert(handle, *entity);
                
                // Add collider if exists
                if let Some(collider) = world.colliders.get(entity) {
                    let transform = world.transforms.get(entity).unwrap();
                    let half_width = collider.get_world_width(transform.scale[0]) / 2.0;
                    let half_height = collider.get_world_height(transform.scale[1]) / 2.0;
                    let offset = collider.get_world_offset(transform.scale[0], transform.scale[1]);
                    
                    let collider_shape = ColliderBuilder::cuboid(half_width, half_height)
                        .translation(vector![offset[0], offset[1]])
                        .friction(0.0) // No friction for platformer
                        .restitution(0.0) // No bounce
                        .build();
                    
                    self.collider_set.insert_with_parent(collider_shape, handle, &mut self.rigid_body_set);
                }
            } else {
                // Update existing rigid body
                let handle = self.entity_to_body[entity];
                if let Some(rb) = self.rigid_body_set.get_mut(handle) {
                    rb.set_linvel(vector![rigidbody.velocity.0, rigidbody.velocity.1], true);
                    rb.set_gravity_scale(rigidbody.gravity_scale, true);
                }
            }
        }
    }
    
    /// Sync Rapier world back to ECS
    pub fn sync_to_ecs(&self, world: &mut World) {
        for (handle, entity) in &self.body_to_entity {
            if let Some(rb) = self.rigid_body_set.get(*handle) {
                // Update transform
                if let Some(transform) = world.transforms.get_mut(entity) {
                    let translation = rb.translation();
                    transform.position[0] = translation.x;
                    transform.position[1] = translation.y;
                }
                
                // Update velocity
                let linvel = rb.linvel();
                if let Some(rigidbody) = world.rigidbodies.get_mut(entity) {
                    rigidbody.velocity = (linvel.x, linvel.y);
                }
                world.velocities.insert(*entity, (linvel.x, linvel.y));
            }
        }
    }
    
    /// Physics step
    pub fn step(&mut self, dt: f32, world: &mut World) {
        if !self.enabled {
            return;
        }
        
        let scaled_dt = dt * self.time_scale;
        
        // Sync from ECS to Rapier
        self.sync_from_ecs(world);
        
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
        
        // Sync back to ECS
        self.sync_to_ecs(world);
    }
    
    /// Check if entity is grounded (touching ground below)
    pub fn is_grounded(&self, entity: Entity, _world: &World) -> bool {
        if let Some(body_handle) = self.entity_to_body.get(&entity) {
            // Get all colliders attached to this rigid body
            if let Some(rb) = self.rigid_body_set.get(*body_handle) {
                for collider_handle in rb.colliders() {
                    // Check contacts for this collider
                    for contact_pair in self.narrow_phase.contact_pairs_with(*collider_handle) {
                        let (h1, h2) = (contact_pair.collider1, contact_pair.collider2);
                        
                        // Find which collider is ours
                        let our_collider = if h1 == *collider_handle { h1 } else { h2 };
                        
                        // Check contact manifolds
                        for manifold in &contact_pair.manifolds {
                            // Normal points from collider1 to collider2
                            let normal = if our_collider == h1 {
                                manifold.local_n1
                            } else {
                                -manifold.local_n1
                            };
                            
                            // If normal points up (negative Y), we're on ground
                            // Allow some tolerance (not exactly vertical)
                            if normal.y < -0.7 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    
    /// Raycast downward to check ground
    pub fn raycast_ground(&self, entity: Entity, world: &World, distance: f32) -> bool {
        if let Some(transform) = world.transforms.get(&entity) {
            let ray_origin = point![transform.position[0], transform.position[1]];
            let ray_dir = vector![0.0, 1.0]; // Down
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
        self.gravity = vector![0.0, gravity];
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
    pub fn apply_impulse(physics: &mut RapierPhysicsWorld, world: &mut World, entity: Entity, impulse_x: f32, impulse_y: f32) {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get_mut(*handle) {
                rb.apply_impulse(vector![impulse_x, impulse_y], true);
                
                // Sync back to ECS
                let linvel = rb.linvel();
                if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = (linvel.x, linvel.y);
                }
                world.velocities.insert(entity, (linvel.x, linvel.y));
            }
        }
    }
    
    /// Set velocity directly
    pub fn set_velocity(physics: &mut RapierPhysicsWorld, world: &mut World, entity: Entity, vel_x: f32, vel_y: f32) {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get_mut(*handle) {
                rb.set_linvel(vector![vel_x, vel_y], true);
                
                // Sync back to ECS
                if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = (vel_x, vel_y);
                }
                world.velocities.insert(entity, (vel_x, vel_y));
            }
        }
    }
    
    /// Get velocity
    pub fn get_velocity(physics: &RapierPhysicsWorld, entity: Entity) -> Option<(f32, f32)> {
        if let Some(handle) = physics.entity_to_body.get(&entity) {
            if let Some(rb) = physics.rigid_body_set.get(*handle) {
                let linvel = rb.linvel();
                return Some((linvel.x, linvel.y));
            }
        }
        None
    }
}

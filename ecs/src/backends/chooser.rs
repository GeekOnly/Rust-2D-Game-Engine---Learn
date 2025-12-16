//! ECS Backend Chooser
//!
//! This module provides a system to choose between different ECS backends at runtime.

use std::fmt;
use serde::{Serialize, Deserialize};

use crate::traits::{EcsWorld, EcsError, Serializable};

// Always include CustomWorld
use crate::CustomWorld;

#[cfg(feature = "hecs")]
use crate::backends::hecs_minimal::HecsMinimal;

#[cfg(feature = "specs")]
use crate::backends::specs_backend::SpecsBackend;
#[cfg(feature = "specs")]
use specs::WorldExt;

#[cfg(feature = "bevy")]
use crate::backends::bevy_backend::BevyBackend;

/// Available ECS backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcsBackendType {
    Custom,
    #[cfg(feature = "hecs")]
    Hecs,
    #[cfg(feature = "specs")]
    Specs,
    #[cfg(feature = "bevy")]
    Bevy,
}

impl fmt::Display for EcsBackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsBackendType::Custom => write!(f, "Custom HashMap Backend"),
            #[cfg(feature = "hecs")]
            EcsBackendType::Hecs => write!(f, "Hecs ECS Backend"),
            #[cfg(feature = "specs")]
            EcsBackendType::Specs => write!(f, "Specs ECS Backend"),
            #[cfg(feature = "bevy")]
            EcsBackendType::Bevy => write!(f, "Bevy ECS Backend"),
        }
    }
}

impl EcsBackendType {
    /// Get all available backends
    pub fn available_backends() -> Vec<EcsBackendType> {
        let mut backends = Vec::new();
        
        // Always include Custom
        backends.push(EcsBackendType::Custom);
        
        #[cfg(feature = "hecs")]
        backends.push(EcsBackendType::Hecs);
        
        #[cfg(feature = "specs")]
        backends.push(EcsBackendType::Specs);
        
        #[cfg(feature = "bevy")]
        backends.push(EcsBackendType::Bevy);
        
        backends
    }
    
    /// Get the default backend
    pub fn default() -> Self {
        #[cfg(feature = "hecs")]
        return EcsBackendType::Hecs;
        
        #[cfg(all(not(feature = "hecs"), feature = "specs"))]
        return EcsBackendType::Specs;
        
        #[cfg(all(not(feature = "hecs"), not(feature = "specs"), feature = "bevy"))]
        return EcsBackendType::Bevy;
        
        #[cfg(all(not(feature = "hecs"), not(feature = "specs"), not(feature = "bevy")))]
        return EcsBackendType::Custom;
    }
    
    /// Get backend description
    pub fn description(&self) -> &'static str {
        match self {
            EcsBackendType::Custom => "Simple HashMap-based ECS implementation. Good for prototyping and small games.",
            #[cfg(feature = "hecs")]
            EcsBackendType::Hecs => "Hecs is a fast, minimal, and flexible ECS library. Good balance of performance and simplicity.",
            #[cfg(feature = "specs")]
            EcsBackendType::Specs => "Specs is a mature, parallel ECS library. Excellent for complex games with many systems.",
            #[cfg(feature = "bevy")]
            EcsBackendType::Bevy => "Bevy ECS is a modern, high-performance ECS with excellent ergonomics and scheduling.",
        }
    }
    
    /// Get performance characteristics
    pub fn performance_info(&self) -> BackendPerformanceInfo {
        match self {
            EcsBackendType::Custom => BackendPerformanceInfo {
                entity_spawn_speed: PerformanceLevel::Medium,
                component_access_speed: PerformanceLevel::Medium,
                query_speed: PerformanceLevel::Low,
                memory_usage: PerformanceLevel::Medium,
                parallel_systems: false,
                archetype_based: false,
            },
            #[cfg(feature = "hecs")]
            EcsBackendType::Hecs => BackendPerformanceInfo {
                entity_spawn_speed: PerformanceLevel::High,
                component_access_speed: PerformanceLevel::High,
                query_speed: PerformanceLevel::High,
                memory_usage: PerformanceLevel::High,
                parallel_systems: false,
                archetype_based: true,
            },
            #[cfg(feature = "specs")]
            EcsBackendType::Specs => BackendPerformanceInfo {
                entity_spawn_speed: PerformanceLevel::Medium,
                component_access_speed: PerformanceLevel::High,
                query_speed: PerformanceLevel::High,
                memory_usage: PerformanceLevel::Medium,
                parallel_systems: true,
                archetype_based: false,
            },
            #[cfg(feature = "bevy")]
            EcsBackendType::Bevy => BackendPerformanceInfo {
                entity_spawn_speed: PerformanceLevel::High,
                component_access_speed: PerformanceLevel::High,
                query_speed: PerformanceLevel::VeryHigh,
                memory_usage: PerformanceLevel::High,
                parallel_systems: true,
                archetype_based: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl fmt::Display for PerformanceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerformanceLevel::Low => write!(f, "Low"),
            PerformanceLevel::Medium => write!(f, "Medium"),
            PerformanceLevel::High => write!(f, "High"),
            PerformanceLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendPerformanceInfo {
    pub entity_spawn_speed: PerformanceLevel,
    pub component_access_speed: PerformanceLevel,
    pub query_speed: PerformanceLevel,
    pub memory_usage: PerformanceLevel,
    pub parallel_systems: bool,
    pub archetype_based: bool,
}

/// Dynamic ECS World that can switch between backends
// Enable Custom backend always
pub enum DynamicWorld {
    Custom(CustomWorld),
    #[cfg(feature = "hecs")]
    Hecs(HecsMinimal),
    #[cfg(feature = "specs")]
    Specs(SpecsBackend),
    #[cfg(feature = "bevy")]
    Bevy(BevyBackend),
}

impl DynamicWorld {
    /// Create a new world with the specified backend
    pub fn new(backend_type: EcsBackendType) -> Result<Self, EcsError> {
        match backend_type {
            EcsBackendType::Custom => Ok(DynamicWorld::Custom(CustomWorld::new())),
            #[cfg(feature = "hecs")]
            EcsBackendType::Hecs => Ok(DynamicWorld::Hecs(HecsMinimal::new())),
            #[cfg(feature = "specs")]
            EcsBackendType::Specs => Ok(DynamicWorld::Specs(SpecsBackend::new())),
            #[cfg(feature = "bevy")]
            EcsBackendType::Bevy => Ok(DynamicWorld::Bevy(BevyBackend::new())),
        }
    }
    
    /// Get the current backend type
    pub fn backend_type(&self) -> EcsBackendType {
        match self {
            DynamicWorld::Custom(_) => EcsBackendType::Custom,
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(_) => EcsBackendType::Hecs,
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(_) => EcsBackendType::Specs,
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(_) => EcsBackendType::Bevy,
        }
    }
    
    /// Switch to a different backend (this will clear all entities)
    pub fn switch_backend(&mut self, new_backend: EcsBackendType) -> Result<(), EcsError> {
        if self.backend_type() == new_backend {
            return Ok(());
        }
        
        *self = Self::new(new_backend)?;
        Ok(())
    }
}

// Implement EcsWorld for DynamicWorld by delegating to the appropriate backend
impl EcsWorld for DynamicWorld {
    type Entity = u64; // Use u64 to support hecs/bevy generation
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        match self {
            DynamicWorld::Custom(world) => world.spawn() as u64,
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => world.spawn().to_bits().get(),
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => world.spawn().id() as u64,
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => world.spawn().to_bits(),
        }
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        match self {
            DynamicWorld::Custom(world) => EcsWorld::despawn(world, entity as u32).map_err(|_| EcsError::EntityNotFound),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => {
                let hecs_entity = hecs::Entity::from_bits(entity).ok_or(EcsError::EntityNotFound)?;
                EcsWorld::despawn(world, hecs_entity)
            },
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => {
                let specs_entity = world.specs_world().entities().entity(entity as u32);
                EcsWorld::despawn(world, specs_entity)
            },
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => {
                let bevy_entity = bevy_ecs::entity::Entity::from_bits(entity);
                EcsWorld::despawn(world, bevy_entity)
            },
        }
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        match self {
            DynamicWorld::Custom(world) => world.is_alive(entity as u32),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => {
                if let Some(hecs_entity) = hecs::Entity::from_bits(entity) {
                    world.is_alive(hecs_entity)
                } else {
                    false
                }
            },
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => {
                let specs_entity = world.specs_world().entities().entity(entity as u32);
                world.is_alive(specs_entity)
            },
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => {
                let bevy_entity = bevy_ecs::entity::Entity::from_bits(entity);
                world.is_alive(bevy_entity)
            },
        }
    }
    
    fn clear(&mut self) {
        match self {
            DynamicWorld::Custom(world) => world.clear(),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => world.clear(),
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => world.clear(),
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => world.clear(),
        }
    }
    
    fn entity_count(&self) -> usize {
        match self {
            DynamicWorld::Custom(world) => world.entity_count(),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => world.entity_count(),
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => world.entity_count(),
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => world.entity_count(),
        }
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        match self {
            DynamicWorld::Custom(world) => {
                EcsWorld::set_parent(world, child as u32, parent.map(|p| p as u32)).map_err(|_| EcsError::InvalidHierarchy)
            },
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => {
                let hecs_child = hecs::Entity::from_bits(child).ok_or(EcsError::EntityNotFound)?;
                let hecs_parent = if let Some(p) = parent {
                    Some(hecs::Entity::from_bits(p).ok_or(EcsError::EntityNotFound)?)
                } else {
                    None
                };
                EcsWorld::set_parent(world, hecs_child, hecs_parent)
            },
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => {
                let specs_child = world.specs_world().entities().entity(child as u32);
                let specs_parent = parent.map(|p| world.specs_world().entities().entity(p as u32));
                EcsWorld::set_parent(world, specs_child, specs_parent)
            },
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => {
                let bevy_child = bevy_ecs::entity::Entity::from_bits(child);
                let bevy_parent = parent.map(bevy_ecs::entity::Entity::from_bits);
                EcsWorld::set_parent(world, bevy_child, bevy_parent)
            },
        }
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        match self {
            DynamicWorld::Custom(world) => world.get_parent(entity as u32).map(|e| e as u64),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => {
                if let Some(hecs_entity) = hecs::Entity::from_bits(entity) {
                    world.get_parent(hecs_entity).map(|e| e.to_bits().get())
                } else {
                    None
                }
            },
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => {
                let specs_entity = world.specs_world().entities().entity(entity as u32);
                world.get_parent(specs_entity).map(|e| e.id() as u64)
            },
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => {
                let bevy_entity = bevy_ecs::entity::Entity::from_bits(entity);
                world.get_parent(bevy_entity).map(|e| e.to_bits())
            },
        }
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        match self {
            DynamicWorld::Custom(world) => EcsWorld::get_children(world, entity as u32).into_iter().map(|e| e as u64).collect(),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => {
                if let Some(hecs_entity) = hecs::Entity::from_bits(entity) {
                    world.get_children(hecs_entity)
                        .into_iter()
                        .map(|e| e.to_bits().get())
                        .collect()
                } else {
                    Vec::new()
                }
            },
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => {
                let specs_entity = world.specs_world().entities().entity(entity as u32);
                world.get_children(specs_entity)
                    .into_iter()
                    .map(|e| e.id() as u64)
                    .collect()
            },
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => {
                let bevy_entity = bevy_ecs::entity::Entity::from_bits(entity);
                world.get_children(bevy_entity)
                    .into_iter()
                    .map(|e| e.to_bits())
                    .collect()
            },
        }
    }
}

impl Serializable for DynamicWorld {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            DynamicWorld::Custom(world) => Serializable::save_to_json(world),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => Serializable::save_to_json(world),
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => Serializable::save_to_json(world),
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => Serializable::save_to_json(world),
        }
    }
    
    fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DynamicWorld::Custom(world) => Serializable::load_from_json(world, json),
            #[cfg(feature = "hecs")]
            DynamicWorld::Hecs(world) => Serializable::load_from_json(world, json),
            #[cfg(feature = "specs")]
            DynamicWorld::Specs(world) => Serializable::load_from_json(world, json),
            #[cfg(feature = "bevy")]
            DynamicWorld::Bevy(world) => Serializable::load_from_json(world, json),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_available_backends() {
        let backends = EcsBackendType::available_backends();
        assert!(!backends.is_empty());
        assert!(backends.contains(&EcsBackendType::Custom));
    }
    
    #[test]
    fn test_backend_descriptions() {
        for backend in EcsBackendType::available_backends() {
            let description = backend.description();
            assert!(!description.is_empty());
            
            let perf_info = backend.performance_info();
            println!("{}: {}", backend, description);
            println!("  Performance: {:?}", perf_info);
        }
    }
    
    #[test]
    fn test_dynamic_world_creation() {
        for backend_type in EcsBackendType::available_backends() {
            let world = DynamicWorld::new(backend_type);
            assert!(world.is_ok(), "Failed to create world with backend: {}", backend_type);
            
            let world = world.unwrap();
            assert_eq!(world.backend_type(), backend_type);
        }
    }
}
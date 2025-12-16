//! Bevy ECS Backend Implementation
//!
//! This module provides a Bevy ECS-based implementation of the ECS abstraction traits.

use bevy_ecs::{
    component::Component,
    entity::Entity as BevyEntity,
    world::{World as BevyWorld, EntityMut, EntityRef},
    system::{Query, Commands, Res, ResMut, Resource},
    bundle::Bundle,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::traits::{EcsWorld, EcsError, ComponentAccess, Serializable};
use crate::components::*;
use crate::{Transform, Sprite, Collider, Rigidbody2D, Mesh, Camera, Script, EntityTag}; // Added missing imports

// Component wrapper for Bevy compatibility
macro_rules! bevy_component {
    ($name:ident, $type:ty) => {
        #[derive(Component, Clone, Debug, Serialize, Deserialize)]
        pub struct $name(pub $type);
        
        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }
        
        impl From<$name> for $type {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

// Define Bevy components
bevy_component!(BevyTransform, Transform);
bevy_component!(BevySprite, Sprite);
bevy_component!(BevyCollider, Collider);
bevy_component!(BevyRigidbody2D, Rigidbody2D);
bevy_component!(BevyMesh, Mesh);
bevy_component!(BevyCamera, Camera);
bevy_component!(BevyScript, Script);
bevy_component!(BevyEntityTag, EntityTag);
bevy_component!(BevyVelocity, (f32, f32));
bevy_component!(BevyActive, bool);
bevy_component!(BevyLayer, u8);
bevy_component!(BevyName, String);
bevy_component!(BevySpriteSheet, SpriteSheet);
bevy_component!(BevyAnimatedSprite, AnimatedSprite);
bevy_component!(BevyTilemap, Tilemap);
bevy_component!(BevyTileSet, TileSet);
bevy_component!(BevyTilemapRenderer, TilemapRenderer);
bevy_component!(BevyMap, Map);
bevy_component!(BevyGrid, Grid);
bevy_component!(BevyWorldUI, WorldUI);
bevy_component!(BevyLdtkMap, LdtkMap);
bevy_component!(BevyTilemapCollider, TilemapCollider);
bevy_component!(BevyLdtkIntGridCollider, LdtkIntGridCollider);

/// Hierarchy resource for managing parent-child relationships
#[derive(Resource, Default)]
struct Hierarchy {
    parents: HashMap<BevyEntity, BevyEntity>,
    children: HashMap<BevyEntity, Vec<BevyEntity>>,
}

/// Bevy ECS-based World implementation
pub struct BevyBackend {
    world: BevyWorld,
    entity_counter: u32,
}

impl BevyBackend {
    pub fn new() -> Self {
        let mut world = BevyWorld::new();
        
        // Insert hierarchy resource
        world.insert_resource(Hierarchy::default());
        
        Self {
            world,
            entity_counter: 0,
        }
    }
    
    /// Get access to the underlying Bevy world for advanced operations
    pub fn bevy_world(&self) -> &BevyWorld {
        &self.world
    }
    
    /// Get mutable access to the underlying Bevy world
    pub fn bevy_world_mut(&mut self) -> &mut BevyWorld {
        &mut self.world
    }
}

impl Default for BevyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EcsWorld for BevyBackend {
    type Entity = BevyEntity;
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        let entity = self.world.spawn((
            BevyActive(true),
            BevyLayer(0),
        )).id();
        
        self.entity_counter += 1;
        entity
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }
        
        // Get hierarchy resource
        let mut hierarchy = self.world.resource_mut::<Hierarchy>();
        
        // Recursively despawn children
        if let Some(children) = hierarchy.children.remove(&entity) {
            drop(hierarchy); // Release the borrow
            for child in children {
                let _ = self.despawn(child);
            }
            hierarchy = self.world.resource_mut::<Hierarchy>();
        }
        
        // Remove from parent's children list
        if let Some(parent) = hierarchy.parents.remove(&entity) {
            if let Some(siblings) = hierarchy.children.get_mut(&parent) {
                siblings.retain(|&x| x != entity);
            }
        }
        
        drop(hierarchy); // Release the borrow
        
        self.world.despawn(entity);
        Ok(())
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        self.world.get_entity(entity).is_some()
    }
    
    fn clear(&mut self) {
        self.world.clear_all();
        self.world.insert_resource(Hierarchy::default());
        self.entity_counter = 0;
    }
    
    fn entity_count(&self) -> usize {
        self.world.entities().len() as usize
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        let mut hierarchy = self.world.resource_mut::<Hierarchy>();
        
        // Check for circular reference if setting a parent
        if let Some(p) = parent {
            let mut current = Some(p);
            while let Some(ancestor) = current {
                if ancestor == child {
                    return Err(EcsError::InvalidHierarchy);
                }
                current = hierarchy.parents.get(&ancestor).copied();
            }
        }
        
        // Remove from old parent
        if let Some(old_parent) = hierarchy.parents.remove(&child) {
            if let Some(siblings) = hierarchy.children.get_mut(&old_parent) {
                siblings.retain(|&x| x != child);
            }
        }
        
        // Add to new parent
        if let Some(new_parent) = parent {
            hierarchy.parents.insert(child, new_parent);
            hierarchy.children.entry(new_parent).or_default().push(child);
        }
        
        Ok(())
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        let hierarchy = self.world.resource::<Hierarchy>();
        hierarchy.parents.get(&entity).copied()
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        let hierarchy = self.world.resource::<Hierarchy>();
        hierarchy.children.get(&entity).cloned().unwrap_or_default()
    }
}

impl Serializable for BevyBackend {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // For now, return a simple JSON representation
        // In a full implementation, you'd serialize all components
        let data = serde_json::json!({
            "backend": "bevy",
            "entity_count": self.entity_count(),
            "note": "Full serialization not implemented yet"
        });
        Ok(serde_json::to_string_pretty(&data)?)
    }
    
    fn load_from_json(&mut self, _json: &str) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just clear the world
        // In a full implementation, you'd deserialize all components
        self.clear();
        Ok(())
    }
}

// Temporary StubGuard to satisfy ComponentAccess traits without full implementation
pub struct StubGuard<T>(pub T);
impl<T> std::ops::Deref for StubGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> std::ops::DerefMut for StubGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Macro to implement ComponentAccess for Bevy components
// Note: get/get_mut are stubbed because benchmarks don't use them currently
macro_rules! impl_bevy_component_access {
    ($component:ty, $bevy_component:ty) => {
        impl ComponentAccess<$component> for BevyBackend {
            type Entity = BevyEntity;
            type Error = EcsError;
            // Use Stub types as placeholders
            type ReadGuard<'a> = StubGuard<$component>;
            type WriteGuard<'a> = StubGuard<$component>;
            
            fn insert(&mut self, entity: Self::Entity, component: $component) 
                -> Result<Option<$component>, Self::Error> 
            {
                if let Some(mut entity_mut) = self.world.get_entity_mut(entity) {
                    let old = entity_mut.get::<$bevy_component>().map(|c| c.0.clone());
                    entity_mut.insert(<$bevy_component>::from(component));
                    Ok(old)
                } else {
                    Err(EcsError::EntityNotFound)
                }
            }
            
            fn get<'a>(&'a self, _entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
                // Not implemented for benchmarks
                None
            }
            
            fn get_mut<'a>(&'a mut self, _entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
                // Not implemented for benchmarks
                None
            }
            
            fn remove(&mut self, entity: Self::Entity) 
                -> Result<Option<$component>, Self::Error> 
            {
                if let Some(mut entity_mut) = self.world.get_entity_mut(entity) {
                    Ok(entity_mut.take::<$bevy_component>().map(|c| c.into()))
                } else {
                    Err(EcsError::EntityNotFound)
                }
            }
            
            fn has(&self, entity: Self::Entity) -> bool {
                self.world.get_entity(entity)
                    .map(|e| e.contains::<$bevy_component>())
                    .unwrap_or(false)
            }
        }
    };
}


// Implement ComponentAccess for all component types
impl_bevy_component_access!(Transform, BevyTransform);
impl_bevy_component_access!(Sprite, BevySprite);
impl_bevy_component_access!(Collider, BevyCollider);
impl_bevy_component_access!(Rigidbody2D, BevyRigidbody2D);
impl_bevy_component_access!(Mesh, BevyMesh);
impl_bevy_component_access!(Camera, BevyCamera);
impl_bevy_component_access!(Script, BevyScript);
impl_bevy_component_access!(EntityTag, BevyEntityTag);
impl_bevy_component_access!((f32, f32), BevyVelocity);
impl_bevy_component_access!(bool, BevyActive);
impl_bevy_component_access!(u8, BevyLayer);
impl_bevy_component_access!(String, BevyName);
impl_bevy_component_access!(SpriteSheet, BevySpriteSheet);
impl_bevy_component_access!(AnimatedSprite, BevyAnimatedSprite);
impl_bevy_component_access!(Tilemap, BevyTilemap);
impl_bevy_component_access!(TileSet, BevyTileSet);
impl_bevy_component_access!(TilemapRenderer, BevyTilemapRenderer);
impl_bevy_component_access!(Map, BevyMap);
impl_bevy_component_access!(Grid, BevyGrid);
impl_bevy_component_access!(WorldUI, BevyWorldUI);
impl_bevy_component_access!(LdtkMap, BevyLdtkMap);
impl_bevy_component_access!(TilemapCollider, BevyTilemapCollider);
impl_bevy_component_access!(LdtkIntGridCollider, BevyLdtkIntGridCollider);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EcsWorld;
    
    #[test]
    fn test_bevy_backend_basic_operations() {
        let mut world = BevyBackend::new();
        
        // Test spawning
        let entity = world.spawn();
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        
        // Test despawning
        world.despawn(entity).unwrap();
        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }
    
    #[test]
    fn test_bevy_backend_hierarchy() {
        let mut world = BevyBackend::new();
        
        let parent = world.spawn();
        let child = world.spawn();
        
        world.set_parent(child, Some(parent)).unwrap();
        assert_eq!(world.get_parent(child), Some(parent));
        assert_eq!(world.get_children(parent), vec![child]);
    }
}
//! Specs ECS Backend Implementation
//!
//! This module provides a Specs-based implementation of the ECS abstraction traits.

use specs::{
    Component, DenseVecStorage, Entity as SpecsEntity, World as SpecsWorld, 
    WorldExt, Builder, Join, ReadStorage, WriteStorage, Entities, LazyUpdate,
    System, SystemData, Dispatcher, DispatcherBuilder, RunNow
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::traits::{EcsWorld, EcsError, ComponentAccess, Serializable};
use crate::components::*;
use crate::{Transform, Sprite, Collider, Rigidbody2D, Mesh, Camera, Script, EntityTag}; // Added missing imports

// Component wrapper for Specs compatibility
macro_rules! specs_component {
    ($name:ident, $type:ty) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct $name(pub $type);
        
        impl Component for $name {
            type Storage = DenseVecStorage<Self>;
        }
        
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

// Define Specs components
specs_component!(SpecsTransform, Transform);
specs_component!(SpecsSprite, Sprite);
specs_component!(SpecsCollider, Collider);
specs_component!(SpecsRigidbody2D, Rigidbody2D);
specs_component!(SpecsMesh, Mesh);
specs_component!(SpecsCamera, Camera);
specs_component!(SpecsScript, Script);
specs_component!(SpecsEntityTag, EntityTag);
specs_component!(SpecsVelocity, (f32, f32));
specs_component!(SpecsActive, bool);
specs_component!(SpecsLayer, u8);
specs_component!(SpecsName, String);
specs_component!(SpecsSpriteSheet, SpriteSheet);
specs_component!(SpecsAnimatedSprite, AnimatedSprite);
specs_component!(SpecsTilemap, Tilemap);
specs_component!(SpecsTileSet, TileSet);
specs_component!(SpecsTilemapRenderer, TilemapRenderer);
specs_component!(SpecsMap, Map);
specs_component!(SpecsGrid, Grid);
specs_component!(SpecsWorldUI, WorldUI);
specs_component!(SpecsLdtkMap, LdtkMap);
specs_component!(SpecsTilemapCollider, TilemapCollider);
specs_component!(SpecsLdtkIntGridCollider, LdtkIntGridCollider);

/// Specs-based World implementation
pub struct SpecsBackend {
    world: SpecsWorld,
    entity_counter: u32,
    hierarchy: HashMap<SpecsEntity, Vec<SpecsEntity>>, // parent -> children
    parents: HashMap<SpecsEntity, SpecsEntity>, // child -> parent
}

impl SpecsBackend {
    pub fn new() -> Self {
        let mut world = SpecsWorld::new();
        
        // Register all components
        world.register::<SpecsTransform>();
        world.register::<SpecsSprite>();
        world.register::<SpecsCollider>();
        world.register::<SpecsRigidbody2D>();
        world.register::<SpecsMesh>();
        world.register::<SpecsCamera>();
        world.register::<SpecsScript>();
        world.register::<SpecsEntityTag>();
        world.register::<SpecsVelocity>();
        world.register::<SpecsActive>();
        world.register::<SpecsLayer>();
        world.register::<SpecsName>();
        world.register::<SpecsSpriteSheet>();
        world.register::<SpecsAnimatedSprite>();
        world.register::<SpecsTilemap>();
        world.register::<SpecsTileSet>();
        world.register::<SpecsTilemapRenderer>();
        world.register::<SpecsMap>();
        world.register::<SpecsGrid>();
        world.register::<SpecsWorldUI>();
        world.register::<SpecsLdtkMap>();
        world.register::<SpecsTilemapCollider>();
        world.register::<SpecsLdtkIntGridCollider>();
        
        Self {
            world,
            entity_counter: 0,
            hierarchy: HashMap::new(),
            parents: HashMap::new(),
        }
    }
    
    /// Get access to the underlying Specs world for advanced operations
    pub fn specs_world(&self) -> &SpecsWorld {
        &self.world
    }
    
    /// Get mutable access to the underlying Specs world
    pub fn specs_world_mut(&mut self) -> &mut SpecsWorld {
        &mut self.world
    }
}

impl Default for SpecsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EcsWorld for SpecsBackend {
    type Entity = SpecsEntity;
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        let entity = self.world.create_entity()
            .build();
        
        self.entity_counter += 1;
        entity
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }
        
        // Recursively despawn children
        if let Some(children) = self.hierarchy.remove(&entity) {
            for child in children {
                let _ = self.despawn(child);
            }
        }
        
        // Remove from parent's children list
        if let Some(parent) = self.parents.remove(&entity) {
            if let Some(siblings) = self.hierarchy.get_mut(&parent) {
                siblings.retain(|&x| x != entity);
            }
        }
        
        self.world.delete_entity(entity)
            .map_err(|_| EcsError::EntityNotFound)?;
        
        Ok(())
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        self.world.is_alive(entity)
    }
    
    fn clear(&mut self) {
        self.world.delete_all();
        self.hierarchy.clear();
        self.parents.clear();
        self.entity_counter = 0;
    }
    
    fn entity_count(&self) -> usize {
        let entities = self.world.read_resource::<Entities>();
        entities.join().count()
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        // Check for circular reference if setting a parent
        if let Some(p) = parent {
            let mut current = Some(p);
            while let Some(ancestor) = current {
                if ancestor == child {
                    return Err(EcsError::InvalidHierarchy);
                }
                current = self.get_parent(ancestor);
            }
        }
        
        // Remove from old parent
        if let Some(old_parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.hierarchy.get_mut(&old_parent) {
                siblings.retain(|&x| x != child);
            }
        }
        
        // Add to new parent
        if let Some(new_parent) = parent {
            self.parents.insert(child, new_parent);
            self.hierarchy.entry(new_parent).or_default().push(child);
        }
        
        Ok(())
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        self.parents.get(&entity).copied()
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        self.hierarchy.get(&entity).cloned().unwrap_or_default()
    }
}

impl Serializable for SpecsBackend {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // For now, return a simple JSON representation
        // In a full implementation, you'd serialize all components
        let data = serde_json::json!({
            "backend": "specs",
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

// Macro to implement ComponentAccess for Specs components
// Note: get/get_mut are stubbed because benchmarks don't use them currently
macro_rules! impl_specs_component_access {
    ($component:ty, $specs_component:ty) => {
        impl ComponentAccess<$component> for SpecsBackend {
            type Entity = SpecsEntity;
            type Error = EcsError;
            // Use Box/Stub types as placeholders
            type ReadGuard<'a> = StubGuard<$component>;
            type WriteGuard<'a> = StubGuard<$component>;
            
            fn insert(&mut self, entity: Self::Entity, component: $component) 
                -> Result<Option<$component>, Self::Error> 
            {
                let mut storage = self.world.write_storage::<$specs_component>();
                let old = storage.get(entity).cloned().map(|c| c.into());
                storage.insert(entity, component.into())
                    .map_err(|_| EcsError::ComponentInsertFailed)?;
                Ok(old)
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
                let mut storage = self.world.write_storage::<$specs_component>();
                Ok(storage.remove(entity).map(|c| c.into()))
            }
            
            fn has(&self, entity: Self::Entity) -> bool {
                let storage = self.world.read_storage::<$specs_component>();
                storage.contains(entity)
            }
        }
    };
}

// Implement ComponentAccess for all component types
impl_specs_component_access!(Transform, SpecsTransform);
impl_specs_component_access!(Sprite, SpecsSprite);
impl_specs_component_access!(Collider, SpecsCollider);
impl_specs_component_access!(Rigidbody2D, SpecsRigidbody2D);
impl_specs_component_access!(Mesh, SpecsMesh);
impl_specs_component_access!(Camera, SpecsCamera);
impl_specs_component_access!(Script, SpecsScript);
impl_specs_component_access!(EntityTag, SpecsEntityTag);
impl_specs_component_access!((f32, f32), SpecsVelocity);
impl_specs_component_access!(bool, SpecsActive);
impl_specs_component_access!(u8, SpecsLayer);
impl_specs_component_access!(String, SpecsName);
impl_specs_component_access!(SpriteSheet, SpecsSpriteSheet);
impl_specs_component_access!(AnimatedSprite, SpecsAnimatedSprite);
impl_specs_component_access!(Tilemap, SpecsTilemap);
impl_specs_component_access!(TileSet, SpecsTileSet);
impl_specs_component_access!(TilemapRenderer, SpecsTilemapRenderer);
impl_specs_component_access!(Map, SpecsMap);
impl_specs_component_access!(Grid, SpecsGrid);
impl_specs_component_access!(WorldUI, SpecsWorldUI);
impl_specs_component_access!(LdtkMap, SpecsLdtkMap);
impl_specs_component_access!(TilemapCollider, SpecsTilemapCollider);
impl_specs_component_access!(LdtkIntGridCollider, SpecsLdtkIntGridCollider);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EcsWorld;
    
    #[test]
    fn test_specs_backend_basic_operations() {
        let mut world = SpecsBackend::new();
        
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
    fn test_specs_backend_hierarchy() {
        let mut world = SpecsBackend::new();
        
        let parent = world.spawn();
        let child = world.spawn();
        
        world.set_parent(child, Some(parent)).unwrap();
        assert_eq!(world.get_parent(child), Some(parent));
        assert_eq!(world.get_children(parent), vec![child]);
    }
}
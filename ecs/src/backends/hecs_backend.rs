#[cfg(feature = "hecs")]
mod hecs_impl {
    use crate::traits::{EcsWorld, ComponentAccess, EcsError, Serializable};
    use hecs::{self, Component};
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};

    /// Wrapper around hecs::World to implement our EcsWorld trait
    pub struct HecsWorld {
        inner: hecs::World,
        // Manual hierarchy tracking for better performance
        parents: HashMap<hecs::Entity, hecs::Entity>,
        children: HashMap<hecs::Entity, Vec<hecs::Entity>>,
    }

impl HecsWorld {
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

impl Default for HecsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl EcsWorld for HecsWorld {
    type Entity = hecs::Entity;
    type Error = EcsError;

    fn spawn(&mut self) -> Self::Entity {
        self.inner.spawn(())
    }

    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }
        
        // Recursively despawn children
        if let Some(children) = self.children.remove(&entity) {
            for child in children {
                let _ = self.despawn(child);
            }
        }
        
        // Remove from parent's children list
        if let Some(parent) = self.parents.remove(&entity) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&x| x != entity);
            }
        }
        
        self.inner.despawn(entity).map_err(|_| EcsError::EntityNotFound)
    }

    fn is_alive(&self, entity: Self::Entity) -> bool {
        self.inner.contains(entity)
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.parents.clear();
        self.children.clear();
    }

    fn entity_count(&self) -> usize {
        self.inner.len() as usize
    }

    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        if !self.is_alive(child) {
            return Err(EcsError::EntityNotFound);
        }
        
        // Check for circular reference if setting a parent
        if let Some(p) = parent {
            if !self.is_alive(p) {
                return Err(EcsError::EntityNotFound);
            }
            
            let mut current = Some(p);
            while let Some(ancestor) = current {
                if ancestor == child {
                    return Err(EcsError::InvalidHierarchy);
                }
                current = self.parents.get(&ancestor).copied();
            }
        }
        
        // Remove from old parent
        if let Some(old_parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&x| x != child);
            }
        }
        
        // Add to new parent
        if let Some(new_parent) = parent {
            self.parents.insert(child, new_parent);
            self.children.entry(new_parent).or_default().push(child);
        }
        
        Ok(())
    }

    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        self.parents.get(&entity).copied()
    }

    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        self.children.get(&entity).cloned().unwrap_or_default()
    }
}

impl Serializable for HecsWorld {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Simple serialization - in a full implementation you'd serialize all components
        let data = serde_json::json!({
            "backend": "hecs",
            "entity_count": self.entity_count(),
            "note": "Full serialization not implemented yet"
        });
        Ok(serde_json::to_string_pretty(&data)?)
    }
    
    fn load_from_json(&mut self, _json: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Simple deserialization - clear the world for now
        self.clear();
        Ok(())
    }
}

// Implement ComponentAccess for specific component types we use
// We need to implement this for each component type individually due to Rust's orphan rules

macro_rules! impl_hecs_component_access {
    ($component:ty) => {
        impl ComponentAccess<$component> for HecsWorld {
            type Entity = hecs::Entity;
            type Error = EcsError;
            
            type ReadGuard<'a> = hecs::Ref<'a, $component>;
            type WriteGuard<'a> = hecs::RefMut<'a, $component>;

            fn insert(&mut self, entity: Self::Entity, component: $component) -> Result<Option<$component>, Self::Error> {
                if !self.inner.contains(entity) {
                    return Err(EcsError::EntityNotFound);
                }
                
                let prev = self.inner.get::<&$component>(entity).ok().map(|c| (*c).clone());
                self.inner.insert_one(entity, component).map_err(|_| EcsError::ComponentInsertFailed)?;
                Ok(prev)
            }

            fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
                self.inner.get::<&$component>(entity).ok()
            }

            fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
                self.inner.get::<&mut $component>(entity).ok()
            }

            fn remove(&mut self, entity: Self::Entity) -> Result<Option<$component>, Self::Error> {
                if !self.inner.contains(entity) {
                    return Err(EcsError::EntityNotFound);
                }
                Ok(self.inner.remove_one::<$component>(entity).ok())
            }

            fn has(&self, entity: Self::Entity) -> bool {
                self.inner.get::<&$component>(entity).is_ok()
            }
        }
    };
}

// Implement for our component types
use crate::{Transform, Sprite, Collider, Rigidbody2D, Mesh, Camera, Script, EntityTag};

impl_hecs_component_access!(Transform);
impl_hecs_component_access!(Sprite);
impl_hecs_component_access!(Collider);
impl_hecs_component_access!(Rigidbody2D);
impl_hecs_component_access!(Mesh);
impl_hecs_component_access!(Camera);
impl_hecs_component_access!(Script);
impl_hecs_component_access!(EntityTag);
impl_hecs_component_access!((f32, f32));
impl_hecs_component_access!(bool);
impl_hecs_component_access!(u8);
impl_hecs_component_access!(String);
}

#[cfg(feature = "hecs")]
pub use hecs_impl::HecsWorld;
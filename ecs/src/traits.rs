//! ECS Abstraction Traits
//!
//! Provides a thin abstraction layer over different ECS backends.
//! This allows switching between Custom HashMap-based, hecs, Specs, or Bevy ECS.

use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during ECS operations
#[derive(Debug, Clone)]
pub enum EcsError {
    /// Entity was not found in the world
    EntityNotFound,
    /// Component was not found on the entity
    ComponentNotFound,
    /// Invalid parent-child hierarchy operation (e.g., circular reference)
    InvalidHierarchy,
    /// Serialization or deserialization error
    SerializationError(String),
    /// Failed to insert component
    ComponentInsertFailed,
    /// Backend is not available (feature not enabled)
    BackendNotAvailable,
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::EntityNotFound => write!(f, "Entity not found"),
            EcsError::ComponentNotFound => write!(f, "Component not found"),
            EcsError::InvalidHierarchy => write!(f, "Invalid parent-child hierarchy"),
            EcsError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            EcsError::ComponentInsertFailed => write!(f, "Failed to insert component"),
            EcsError::BackendNotAvailable => write!(f, "ECS backend is not available"),
        }
    }
}

impl std::error::Error for EcsError {}

// ============================================================================
// Core Traits
// ============================================================================

/// ECS World trait - defines fundamental operations for entity and world management
pub trait EcsWorld {
    /// The entity type used by this world
    type Entity: Copy + Clone + PartialEq + Eq + std::hash::Hash + fmt::Debug;
    
    /// The error type used by this world
    type Error: std::error::Error;
    
    // ========================================
    // Entity Lifecycle
    // ========================================
    
    /// Spawn a new entity and return its ID
    fn spawn(&mut self) -> Self::Entity;
    
    /// Despawn an entity, removing it and all its components
    /// Returns an error if the entity doesn't exist
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error>;
    
    /// Check if an entity is alive (exists in the world)
    fn is_alive(&self, entity: Self::Entity) -> bool;
    
    // ========================================
    // World Operations
    // ========================================
    
    /// Clear all entities and components from the world
    fn clear(&mut self);
    
    /// Get the total number of entities in the world
    fn entity_count(&self) -> usize;
    
    // ========================================
    // Hierarchy Operations
    // ========================================
    
    /// Set the parent of a child entity
    /// Pass None to remove the parent
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error>;
    
    /// Get the parent of an entity, if it has one
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity>;
    
    /// Get all direct children of an entity
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity>;
}

/// ComponentAccess trait - provides type-safe component operations
pub trait ComponentAccess<T> {
    /// The entity type
    type Entity;
    
    /// The error type
    type Error;
    
    /// The read guard type (allows supporting both &T and Ref<T>)
    type ReadGuard<'a>: std::ops::Deref<Target = T> where Self: 'a;
    
    /// The write guard type (allows supporting both &mut T and RefMut<T>)
    type WriteGuard<'a>: std::ops::DerefMut<Target = T> where Self: 'a;
    
    /// Insert a component for an entity
    /// Returns the previous component value if one existed
    fn insert(&mut self, entity: Self::Entity, component: T) -> Result<Option<T>, Self::Error>;
    
    /// Get an immutable reference to a component
    fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>>;
    
    /// Get a mutable reference to a component
    fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>>;
    
    /// Remove a component from an entity
    /// Returns the component value if it existed
    fn remove(&mut self, entity: Self::Entity) -> Result<Option<T>, Self::Error>;
    
    /// Check if an entity has a component
    fn has(&self, entity: Self::Entity) -> bool;
}

/// Query trait - enables efficient iteration over entities with specific components (immutable)
pub trait Query<'w> {
    /// The item type yielded by the iterator
    type Item;
    
    /// The iterator type
    type Iter: Iterator<Item = Self::Item>;
    
    /// Create an iterator over the query results
    fn iter(&'w self) -> Self::Iter;
}

/// QueryMut trait - enables efficient iteration over entities with specific components (mutable)
pub trait QueryMut<'w> {
    /// The item type yielded by the iterator
    type Item;
    
    /// The iterator type
    type Iter: Iterator<Item = Self::Item>;
    
    /// Create a mutable iterator over the query results
    fn iter_mut(&'w mut self) -> Self::Iter;
}

/// Serializable trait - handles world persistence
pub trait Serializable {
    /// Serialize the world to JSON format
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>>;
    
    /// Deserialize the world from JSON format
    fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>>;
}

// ============================================================================
// Macro for implementing ComponentAccess
// ============================================================================

/// Macro to reduce boilerplate when implementing ComponentAccess for HashMap-based storage
#[macro_export]
macro_rules! impl_component_access {
    ($world_type:ty, $component_type:ty, $field:ident, $entity_type:ty) => {
        impl $crate::traits::ComponentAccess<$component_type> for $world_type {
            type Entity = $entity_type;
            type Error = $crate::traits::EcsError;
            
            type ReadGuard<'a> = &'a $component_type;
            type WriteGuard<'a> = &'a mut $component_type;
            
            fn insert(&mut self, entity: Self::Entity, component: $component_type) 
                -> Result<Option<$component_type>, Self::Error> 
            {
                Ok(self.$field.insert(entity, component))
            }
            
            fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
                self.$field.get(&entity)
            }
            
            fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
                self.$field.get_mut(&entity)
            }
            
            fn remove(&mut self, entity: Self::Entity) 
                -> Result<Option<$component_type>, Self::Error> 
            {
                Ok(self.$field.remove(&entity))
            }
            
            fn has(&self, entity: Self::Entity) -> bool {
                self.$field.contains_key(&entity)
            }
        }
    };
}

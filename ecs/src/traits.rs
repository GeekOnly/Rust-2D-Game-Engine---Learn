//! ECS Abstraction Traits
//!
//! Provides a thin abstraction layer over different ECS backends.
//! This allows switching between Custom HashMap-based, hecs, Specs, or Bevy ECS.

use std::any::TypeId;

/// Entity ID trait - must be Copy, Eq, Hash for lookups
pub trait EntityId: Copy + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static {
    /// Create an entity ID from a u32 (for compatibility)
    fn from_u32(id: u32) -> Self;

    /// Convert entity ID to u32 (for serialization)
    fn to_u32(&self) -> u32;
}

// Implement for u32 (our current Entity type)
impl EntityId for u32 {
    fn from_u32(id: u32) -> Self { id }
    fn to_u32(&self) -> u32 { *self }
}

/// Component marker trait
/// All components must be Send + Sync + 'static for thread safety
pub trait Component: Send + Sync + 'static {}

// Blanket implementation for all types that meet requirements
impl<T: Send + Sync + 'static> Component for T {}

/// ECS World trait - core abstraction over different backends
pub trait EcsWorld: Send + Sync {
    type Entity: EntityId;

    // ========================================
    // Entity Management
    // ========================================

    /// Spawn a new entity
    fn spawn(&mut self) -> Self::Entity;

    /// Despawn an entity (removes all components)
    fn despawn(&mut self, entity: Self::Entity);

    /// Check if entity is alive
    fn is_alive(&self, entity: Self::Entity) -> bool;

    /// Clear all entities and components
    fn clear(&mut self);

    // ========================================
    // Component Access (Type-erased for flexibility)
    // ========================================

    /// Insert a component (type-erased)
    fn insert_component<C: Component>(&mut self, entity: Self::Entity, component: C);

    /// Remove a component (type-erased)
    fn remove_component<C: Component>(&mut self, entity: Self::Entity) -> Option<C>;

    /// Get immutable reference to component
    fn get_component<C: Component>(&self, entity: Self::Entity) -> Option<&C>;

    /// Get mutable reference to component
    fn get_component_mut<C: Component>(&mut self, entity: Self::Entity) -> Option<&mut C>;

    /// Check if entity has component
    fn has_component<C: Component>(&self, entity: Self::Entity) -> bool;
}

/// Backend factory trait
pub trait EcsBackend {
    type World: EcsWorld;

    /// Create a new world instance
    fn create_world() -> Self::World;

    /// Get backend name (for debugging/benchmarking)
    fn name() -> &'static str;

    /// Get backend description
    fn description() -> &'static str;
}

/// Query trait for iterating entities with specific components
pub trait Query<'w> {
    type Item;

    fn iter(&self) -> Box<dyn Iterator<Item = Self::Item> + 'w>;
}

// ============================================================================
// Optional: System trait (for future System scheduling)
// ============================================================================

/// System trait for automatic scheduling (future feature)
pub trait System: Send + Sync {
    fn run(&mut self, world: &mut dyn EcsWorld<Entity = u32>);
    fn name(&self) -> &str;
}

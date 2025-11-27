# Design Document: ECS Abstraction Layer

## Overview

This design document describes an abstraction layer for the Entity Component System (ECS) that enables swapping between different ECS backend implementations without modifying game or editor code. The abstraction uses Rust traits to define a common interface that any ECS backend can implement, providing flexibility while maintaining type safety and performance.

The current implementation uses a HashMap-based ECS stored in the `World` struct. This design will create trait-based abstractions that allow this implementation to coexist with alternative backends like Bevy ECS, hecs, specs, or legion.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Game/Editor Code                      │
│              (Uses trait-based interfaces)               │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  ECS Abstraction Layer                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  EcsWorld    │  │  Component   │  │    Query     │  │
│  │    Trait     │  │    Access    │  │    Trait     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   HashMap    │    │  Bevy ECS    │    │    hecs      │
│   Backend    │    │   Backend    │    │   Backend    │
│  (Current)   │    │   (Future)   │    │   (Future)   │
└──────────────┘    └──────────────┘    └──────────────┘
```

### Design Principles

1. **Zero-Cost Abstraction**: Use static dispatch (generics) where possible to avoid runtime overhead
2. **Type Safety**: Leverage Rust's type system to catch errors at compile time
3. **Backward Compatibility**: Existing HashMap-based implementation continues to work
4. **Extensibility**: Easy to add new ECS backends by implementing traits
5. **Minimal Boilerplate**: Use macros to reduce repetitive implementation code

## Components and Interfaces

### Core Traits

#### 1. EcsWorld Trait

The `EcsWorld` trait defines the fundamental operations for entity and world management:

```rust
pub trait EcsWorld {
    type Entity: Copy + Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;
    type Error: std::error::Error;
    
    // Entity lifecycle
    fn spawn(&mut self) -> Self::Entity;
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error>;
    fn is_alive(&self, entity: Self::Entity) -> bool;
    
    // World operations
    fn clear(&mut self);
    fn entity_count(&self) -> usize;
    
    // Hierarchy operations
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error>;
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity>;
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity>;
}
```

#### 2. ComponentAccess Trait

The `ComponentAccess` trait provides type-safe component operations:

```rust
pub trait ComponentAccess<T> {
    type Entity;
    type Error;
    
    fn insert(&mut self, entity: Self::Entity, component: T) -> Result<Option<T>, Self::Error>;
    fn get(&self, entity: Self::Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut T>;
    fn remove(&mut self, entity: Self::Entity) -> Result<Option<T>, Self::Error>;
    fn has(&self, entity: Self::Entity) -> bool;
}
```

#### 3. Query Trait

The `Query` trait enables efficient iteration over entities with specific components:

```rust
pub trait Query<'w> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    
    fn iter(&'w self) -> Self::Iter;
}

pub trait QueryMut<'w> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    
    fn iter_mut(&'w mut self) -> Self::Iter;
}
```

#### 4. Serializable Trait

The `Serializable` trait handles world persistence:

```rust
pub trait Serializable {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

### Component Macro

To reduce boilerplate, we provide a macro for implementing `ComponentAccess`:

```rust
#[macro_export]
macro_rules! impl_component_access {
    ($world_type:ty, $component_type:ty, $field:ident) => {
        impl ComponentAccess<$component_type> for $world_type {
            type Entity = Entity;
            type Error = EcsError;
            
            fn insert(&mut self, entity: Self::Entity, component: $component_type) 
                -> Result<Option<$component_type>, Self::Error> 
            {
                Ok(self.$field.insert(entity, component))
            }
            
            fn get(&self, entity: Self::Entity) -> Option<&$component_type> {
                self.$field.get(&entity)
            }
            
            fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut $component_type> {
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
```

## Data Models

### Error Type

```rust
#[derive(Debug, Clone)]
pub enum EcsError {
    EntityNotFound,
    ComponentNotFound,
    InvalidHierarchy,
    SerializationError(String),
}

impl std::fmt::Display for EcsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcsError::EntityNotFound => write!(f, "Entity not found"),
            EcsError::ComponentNotFound => write!(f, "Component not found"),
            EcsError::InvalidHierarchy => write!(f, "Invalid parent-child hierarchy"),
            EcsError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for EcsError {}
```

### HashMap Backend Implementation

The existing `World` struct will implement all traits:

```rust
impl EcsWorld for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        self.active.insert(id, true);
        self.layers.insert(id, 0);
        id
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        // Recursively despawn children
        if let Some(children) = self.children.remove(&entity) {
            for child in children {
                self.despawn(child)?;
            }
        }
        
        // Remove from parent
        if let Some(parent) = self.parents.remove(&entity) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&x| x != entity);
            }
        }
        
        // Remove all components
        self.transforms.remove(&entity);
        self.velocities.remove(&entity);
        self.sprites.remove(&entity);
        self.colliders.remove(&entity);
        self.meshes.remove(&entity);
        self.cameras.remove(&entity);
        self.tags.remove(&entity);
        self.scripts.remove(&entity);
        self.active.remove(&entity);
        self.layers.remove(&entity);
        self.names.remove(&entity);
        
        Ok(())
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        self.transforms.contains_key(&entity) ||
        self.sprites.contains_key(&entity) ||
        self.colliders.contains_key(&entity) ||
        self.meshes.contains_key(&entity) ||
        self.cameras.contains_key(&entity) ||
        self.active.contains_key(&entity)
    }
    
    fn clear(&mut self) {
        World::clear(self);
    }
    
    fn entity_count(&self) -> usize {
        self.active.len()
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) 
        -> Result<(), Self::Error> 
    {
        World::set_parent(self, child, parent);
        Ok(())
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        World::get_parent(self, entity)
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        World::get_children(self, entity).to_vec()
    }
}

// Implement ComponentAccess for all component types
impl_component_access!(World, Transform, transforms);
impl_component_access!(World, Sprite, sprites);
impl_component_access!(World, Collider, colliders);
impl_component_access!(World, Mesh, meshes);
impl_component_access!(World, Camera, cameras);
impl_component_access!(World, Script, scripts);
impl_component_access!(World, EntityTag, tags);

// Special implementations for tuple components
impl ComponentAccess<(f32, f32)> for World {
    type Entity = Entity;
    type Error = EcsError;
    
    fn insert(&mut self, entity: Self::Entity, component: (f32, f32)) 
        -> Result<Option<(f32, f32)>, Self::Error> 
    {
        Ok(self.velocities.insert(entity, component))
    }
    
    fn get(&self, entity: Self::Entity) -> Option<&(f32, f32)> {
        self.velocities.get(&entity)
    }
    
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut (f32, f32)> {
        self.velocities.get_mut(&entity)
    }
    
    fn remove(&mut self, entity: Self::Entity) 
        -> Result<Option<(f32, f32)>, Self::Error> 
    {
        Ok(self.velocities.remove(&entity))
    }
    
    fn has(&self, entity: Self::Entity) -> bool {
        self.velocities.contains_key(&entity)
    }
}

// Similar implementations for bool (active), u8 (layers), String (names)
```

### Query Implementation

```rust
pub struct SingleQuery<'w, T> {
    components: &'w HashMap<Entity, T>,
}

impl<'w, T> Query<'w> for SingleQuery<'w, T> {
    type Item = (Entity, &'w T);
    type Iter = Box<dyn Iterator<Item = Self::Item> + 'w>;
    
    fn iter(&'w self) -> Self::Iter {
        Box::new(self.components.iter().map(|(&e, c)| (e, c)))
    }
}

pub struct SingleQueryMut<'w, T> {
    components: &'w mut HashMap<Entity, T>,
}

impl<'w, T> QueryMut<'w> for SingleQueryMut<'w, T> {
    type Item = (Entity, &'w mut T);
    type Iter = Box<dyn Iterator<Item = Self::Item> + 'w>;
    
    fn iter_mut(&'w mut self) -> Self::Iter {
        Box::new(self.components.iter_mut().map(|(&e, c)| (e, c)))
    }
}
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Component insert-get round trip

*For any* entity and component value, inserting a component and then immediately getting it should return an equivalent value.

**Validates: Requirements 1.2, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7**

### Property 2: Entity spawn increases count

*For any* world state, spawning an entity should increase the entity count by exactly one.

**Validates: Requirements 1.1**

### Property 3: Entity despawn decreases count

*For any* world with at least one entity, despawning an entity should decrease the entity count by at least one (accounting for children).

**Validates: Requirements 1.1**

### Property 4: Despawn removes all components

*For any* entity with components, after despawning that entity, all component queries for that entity should return None.

**Validates: Requirements 1.1**

### Property 5: Recursive despawn removes children

*For any* entity with children, despawning the parent entity should also despawn all children recursively.

**Validates: Requirements 7.2**

### Property 6: Parent-child relationship consistency

*For any* child entity with a parent, the parent's children list should contain that child entity.

**Validates: Requirements 7.1, 7.4**

### Property 7: Get children returns all direct children

*For any* entity with known children, querying for children should return exactly those children and no others.

**Validates: Requirements 7.3**

### Property 8: Single component query correctness

*For any* set of entities where some have component T and some don't, querying for component T should return exactly the entities that have component T.

**Validates: Requirements 8.1**

### Property 9: Multi-component query correctness

*For any* set of entities with various component combinations, querying for multiple components should return only entities that have all specified components.

**Validates: Requirements 8.2**

### Property 10: Mutable query modifies components

*For any* entity with a component, modifying the component through a mutable query should persist the changes.

**Validates: Requirements 8.3**

### Property 11: Serialization round trip preserves world state

*For any* world state, serializing to JSON and then deserializing should produce an equivalent world with the same entities, components, and hierarchy.

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 12: Backend-agnostic operations

*For any* ECS backend implementation, the same sequence of operations (spawn, insert component, query) should produce equivalent results.

**Validates: Requirements 1.5**

## Error Handling

### Error Categories

1. **Entity Errors**
   - `EntityNotFound`: Attempting to operate on a non-existent entity
   - Handle by returning `Result<T, EcsError>` from operations

2. **Component Errors**
   - `ComponentNotFound`: Attempting to access a component that doesn't exist
   - Handle by returning `Option<T>` for get operations

3. **Hierarchy Errors**
   - `InvalidHierarchy`: Attempting to create circular parent-child relationships
   - Handle by validating hierarchy operations before applying

4. **Serialization Errors**
   - `SerializationError`: JSON parsing or generation failures
   - Handle by wrapping serde errors in `EcsError`

### Error Handling Strategy

- Use `Result<T, EcsError>` for operations that can fail
- Use `Option<T>` for queries that may not find components
- Provide clear error messages with context
- Document error conditions in trait method documentation
- Ensure errors don't leave the world in an inconsistent state

## Testing Strategy

### Unit Testing

Unit tests will verify specific behaviors and edge cases:

1. **Entity Lifecycle Tests**
   - Spawn creates unique entities
   - Despawn removes entity and components
   - Clear removes all entities

2. **Component Tests**
   - Insert adds component
   - Get retrieves correct component
   - Remove deletes component
   - Has checks component existence

3. **Hierarchy Tests**
   - Set parent creates relationship
   - Get children returns correct list
   - Despawn parent removes children

4. **Serialization Tests**
   - Empty world serialization
   - World with entities serialization
   - Deserialization error handling

### Property-Based Testing

Property-based tests will verify universal properties across many random inputs using the `proptest` crate:

1. **Property Test: Component Round Trip**
   - Generate random entities and components
   - Insert components
   - Verify get returns same values
   - **Feature: ecs-abstraction, Property 1: Component insert-get round trip**

2. **Property Test: Entity Count Consistency**
   - Generate random spawn/despawn sequences
   - Track expected count
   - Verify actual count matches expected
   - **Feature: ecs-abstraction, Property 2: Entity spawn increases count**
   - **Feature: ecs-abstraction, Property 3: Entity despawn decreases count**

3. **Property Test: Despawn Cleanup**
   - Generate entities with random components
   - Despawn entities
   - Verify all components removed
   - **Feature: ecs-abstraction, Property 4: Despawn removes all components**

4. **Property Test: Recursive Despawn**
   - Generate random entity hierarchies
   - Despawn parent entities
   - Verify all descendants removed
   - **Feature: ecs-abstraction, Property 5: Recursive despawn removes children**

5. **Property Test: Hierarchy Consistency**
   - Generate random parent-child relationships
   - Verify bidirectional consistency
   - **Feature: ecs-abstraction, Property 6: Parent-child relationship consistency**
   - **Feature: ecs-abstraction, Property 7: Get children returns all direct children**

6. **Property Test: Query Correctness**
   - Generate entities with random component combinations
   - Query for specific components
   - Verify only matching entities returned
   - **Feature: ecs-abstraction, Property 8: Single component query correctness**
   - **Feature: ecs-abstraction, Property 9: Multi-component query correctness**

7. **Property Test: Mutable Query**
   - Generate entities with components
   - Modify through mutable query
   - Verify changes persist
   - **Feature: ecs-abstraction, Property 10: Mutable query modifies components**

8. **Property Test: Serialization Round Trip**
   - Generate random world states
   - Serialize and deserialize
   - Verify world state unchanged
   - **Feature: ecs-abstraction, Property 11: Serialization round trip preserves world state**

9. **Property Test: Backend Equivalence**
   - Run same operations on different backends
   - Verify equivalent results
   - **Feature: ecs-abstraction, Property 12: Backend-agnostic operations**

### Integration Testing

Integration tests will verify the abstraction works with real game/editor code:

1. Test editor entity creation through abstraction
2. Test scene save/load through abstraction
3. Test game systems using abstraction
4. Test switching between backends

### Testing Configuration

- Use `proptest` crate for property-based testing
- Configure each property test to run minimum 100 iterations
- Use `cargo test` for running all tests
- Use `cargo test --release` for performance testing

## Implementation Notes

### Phase 1: Core Traits (Current Implementation)

Based on the context provided, the following has already been implemented:

- `EcsWorld` trait with basic operations
- `ComponentStorage` trait
- `ComponentAccess` trait with macro
- `System` trait for system execution
- `SystemScheduler` for running systems
- Example `VecEcsWorld` as alternative backend
- Basic tests for abstraction layer

### Phase 2: Complete HashMap Backend (Next Steps)

- Implement all traits for existing `World` struct
- Add query support for HashMap backend
- Implement `Serializable` trait
- Add comprehensive component access implementations

### Phase 3: Advanced Features

- Multi-component query support
- Query builder pattern
- System dependencies and scheduling
- Parallel system execution (optional)

### Phase 4: Alternative Backends

- Bevy ECS integration
- hecs integration
- specs integration
- Performance benchmarking

### Migration Strategy

1. Keep existing `World` API intact for backward compatibility
2. Add trait implementations alongside existing methods
3. Gradually migrate editor/game code to use traits
4. Deprecate direct `World` usage in favor of trait-based code
5. Eventually make `World` an implementation detail

### Performance Considerations

- Use static dispatch (generics) for zero-cost abstraction
- Avoid boxing iterators where possible
- Use `&mut` references to avoid cloning
- Consider `unsafe` for performance-critical paths (with careful review)
- Benchmark against baseline before/after abstraction

## Future Enhancements

1. **Archetype-based Storage**: Migrate to archetype storage for better cache locality
2. **Parallel Queries**: Support parallel iteration over entities
3. **Change Detection**: Track component modifications for efficient updates
4. **Event System**: Integrate event handling with ECS
5. **Reflection**: Runtime component type information
6. **Prefab System**: Enhanced prefab support through abstraction
7. **Component Bundles**: Group related components for easier insertion

## References

- [Bevy ECS Documentation](https://docs.rs/bevy_ecs/)
- [hecs Documentation](https://docs.rs/hecs/)
- [specs Documentation](https://docs.rs/specs/)
- [ECS FAQ](https://github.com/SanderMertens/ecs-faq)

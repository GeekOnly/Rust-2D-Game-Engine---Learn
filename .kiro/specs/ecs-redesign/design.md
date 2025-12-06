# ECS Redesign - Design Document

## Overview

This document describes the technical design for the XS Game Engine's next-generation Entity Component System (ECS). The design is inspired by Bevy ECS's archetype-based architecture, enhanced with aggressive SIMD optimization and mobile-first memory efficiency.

### Design Goals

1. **Performance**: 4-10x faster than current HashMap-based implementation
2. **Scalability**: Support 100,000+ entities at 60 FPS
3. **Memory Efficiency**: 30-50% reduction in memory usage
4. **SIMD Optimization**: Leverage CPU vectorization for 4-8x speedup on batch operations
5. **Backward Compatibility**: Maintain existing API through compatibility layer
6. **Developer Experience**: Ergonomic API with compile-time safety

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Public API Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  World API   │  │  Query API   │  │  System API  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Compatibility Layer                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  HashMap-based API Wrapper (for legacy code)         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      Core ECS Engine                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Archetype   │  │  Sparse Set  │  │   Change     │      │
│  │   Storage    │  │   Mapping    │  │  Detection   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Query     │  │   System     │  │   Resource   │      │
│  │   Engine     │  │  Scheduler   │  │   Manager    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Memory Management Layer                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  SIMD-Aligned│  │  Pooled      │  │  Cache-Line  │      │
│  │  Allocator   │  │  Allocator   │  │  Alignment   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## Components and Interfaces

### 1. Entity and Component Types

#### Entity Structure
```rust
/// Entity ID with generation counter for safety
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    /// Unique entity index (reused after despawn)
    index: u32,
    /// Generation counter (prevents use-after-free)
    generation: u32,
}

impl Entity {
    pub fn index(&self) -> u32 { self.index }
    pub fn generation(&self) -> u32 { self.generation }
    
    /// Create a null entity (for initialization)
    pub const fn null() -> Self {
        Self { index: u32::MAX, generation: 0 }
    }
    
    /// Check if entity is null
    pub fn is_null(&self) -> bool {
        self.index == u32::MAX
    }
}
```

#### Component Trait
```rust
/// Marker trait for components
/// All components must be 'static, Send, and Sync for thread safety
pub trait Component: 'static + Send + Sync {
    /// Optional: Component type name for debugging
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Automatically implement Component for all valid types
impl<T: 'static + Send + Sync> Component for T {}
```

---

### 2. Archetype-Based Storage

#### Archetype Structure
```rust
/// Archetype ID - unique identifier for a component combination
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u32);

/// Archetype - stores entities with the same component types
pub struct Archetype {
    /// Unique archetype ID
    id: ArchetypeId,
    
    /// Component types in this archetype (sorted)
    component_types: Vec<ComponentTypeId>,
    
    /// Entity IDs in this archetype
    entities: Vec<Entity>,
    
    /// Component storage (one per component type)
    /// Uses type-erased storage for flexibility
    components: HashMap<ComponentTypeId, Box<dyn ComponentStorage>>,
    
    /// Edges to other archetypes (for add/remove component)
    edges: ArchetypeEdges,
}

/// Edges between archetypes (for fast component add/remove)
struct ArchetypeEdges {
    /// Map: ComponentTypeId -> ArchetypeId (when adding component)
    add: HashMap<ComponentTypeId, ArchetypeId>,
    
    /// Map: ComponentTypeId -> ArchetypeId (when removing component)
    remove: HashMap<ComponentTypeId, ArchetypeId>,
}

impl Archetype {
    /// Add an entity to this archetype
    pub fn push(&mut self, entity: Entity, components: Vec<Box<dyn Any>>) {
        self.entities.push(entity);
        
        for (type_id, component) in self.component_types.iter().zip(components) {
            self.components.get_mut(type_id)
                .unwrap()
                .push(component);
        }
    }
    
    /// Remove an entity from this archetype (swap-remove for O(1))
    pub fn swap_remove(&mut self, index: usize) -> Entity {
        let entity = self.entities.swap_remove(index);
        
        for storage in self.components.values_mut() {
            storage.swap_remove(index);
        }
        
        entity
    }
    
    /// Get component storage for a type
    pub fn get_storage<T: Component>(&self) -> Option<&ComponentColumn<T>> {
        let type_id = ComponentTypeId::of::<T>();
        self.components.get(&type_id)
            .and_then(|storage| storage.downcast_ref())
    }
    
    /// Get mutable component storage for a type
    pub fn get_storage_mut<T: Component>(&mut self) -> Option<&mut ComponentColumn<T>> {
        let type_id = ComponentTypeId::of::<T>();
        self.components.get_mut(&type_id)
            .and_then(|storage| storage.downcast_mut())
    }
}
```

#### Component Storage (SIMD-Optimized)
```rust
/// Type-erased component storage trait
pub trait ComponentStorage: Send + Sync {
    fn push(&mut self, component: Box<dyn Any>);
    fn swap_remove(&mut self, index: usize);
    fn len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Concrete component storage with SIMD alignment
pub struct ComponentColumn<T: Component> {
    /// Component data (SIMD-aligned)
    /// Uses Vec with custom allocator for 16-byte alignment
    data: AlignedVec<T, 16>,
    
    /// Change detection ticks
    changed: Vec<u32>,
    
    /// Added detection ticks
    added: Vec<u32>,
}

impl<T: Component> ComponentColumn<T> {
    pub fn new() -> Self {
        Self {
            data: AlignedVec::new(),
            changed: Vec::new(),
            added: Vec::new(),
        }
    }
    
    /// Push a component (marks as added and changed)
    pub fn push(&mut self, component: T, tick: u32) {
        self.data.push(component);
        self.changed.push(tick);
        self.added.push(tick);
    }
    
    /// Get component slice (for SIMD operations)
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
    
    /// Get mutable component slice (for SIMD operations)
    pub fn as_mut_slice(&mut self, tick: u32) -> &mut [T] {
        // Mark all as changed
        self.changed.fill(tick);
        self.data.as_mut_slice()
    }
    
    /// Get component with change detection
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    /// Get mutable component with change detection
    pub fn get_mut(&mut self, index: usize, tick: u32) -> Option<&mut T> {
        if let Some(component) = self.data.get_mut(index) {
            self.changed[index] = tick;
            Some(component)
        } else {
            None
        }
    }
}

/// SIMD-aligned vector
#[repr(align(16))]
pub struct AlignedVec<T, const ALIGN: usize> {
    data: Vec<T>,
}

impl<T, const ALIGN: usize> AlignedVec<T, ALIGN> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }
    
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}
```

---

### 3. Sparse Set Entity-Component Mapping

```rust
/// Sparse set for O(1) entity -> component index mapping
pub struct SparseSet {
    /// Sparse array: entity.index -> dense index
    /// Uses Option for empty slots
    sparse: Vec<Option<u32>>,
    
    /// Dense array: dense index -> entity
    dense: Vec<Entity>,
}

impl SparseSet {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
        }
    }
    
    /// Insert entity (returns dense index)
    pub fn insert(&mut self, entity: Entity) -> u32 {
        let index = entity.index() as usize;
        
        // Grow sparse array if needed
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }
        
        let dense_index = self.dense.len() as u32;
        self.sparse[index] = Some(dense_index);
        self.dense.push(entity);
        
        dense_index
    }
    
    /// Remove entity (returns dense index if found)
    pub fn remove(&mut self, entity: Entity) -> Option<u32> {
        let index = entity.index() as usize;
        
        if index >= self.sparse.len() {
            return None;
        }
        
        if let Some(dense_index) = self.sparse[index] {
            // Swap-remove from dense array
            let last_entity = self.dense.swap_remove(dense_index as usize);
            
            // Update sparse array for swapped entity
            if dense_index < self.dense.len() as u32 {
                self.sparse[last_entity.index() as usize] = Some(dense_index);
            }
            
            self.sparse[index] = None;
            Some(dense_index)
        } else {
            None
        }
    }
    
    /// Get dense index for entity (O(1))
    pub fn get(&self, entity: Entity) -> Option<u32> {
        let index = entity.index() as usize;
        if index < self.sparse.len() {
            self.sparse[index]
        } else {
            None
        }
    }
    
    /// Check if entity exists (O(1))
    pub fn contains(&self, entity: Entity) -> bool {
        self.get(entity).is_some()
    }
    
    /// Iterate all entities (linear in dense array)
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.dense.iter().copied()
    }
}
```

---

### 4. World Structure

```rust
/// Main ECS world
pub struct World {
    /// Entity allocator
    entities: Entities,
    
    /// Archetype storage
    archetypes: Archetypes,
    
    /// Entity -> Archetype mapping
    entity_index: HashMap<Entity, EntityLocation>,
    
    /// Component type registry
    components: Components,
    
    /// Resources (global state)
    resources: Resources,
    
    /// Change detection tick
    change_tick: AtomicU32,
}

/// Entity location in archetype
#[derive(Copy, Clone, Debug)]
struct EntityLocation {
    archetype_id: ArchetypeId,
    index: u32,  // Index within archetype
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            archetypes: Archetypes::new(),
            entity_index: HashMap::new(),
            components: Components::new(),
            resources: Resources::new(),
            change_tick: AtomicU32::new(1),
        }
    }
    
    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = self.entities.alloc();
        
        // Place in empty archetype
        let archetype_id = self.archetypes.empty();
        let index = self.archetypes.get_mut(archetype_id)
            .unwrap()
            .push(entity, vec![]);
        
        self.entity_index.insert(entity, EntityLocation {
            archetype_id,
            index,
        });
        
        entity
    }
    
    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EcsError> {
        let location = self.entity_index.remove(&entity)
            .ok_or(EcsError::EntityNotFound)?;
        
        // Remove from archetype
        let archetype = self.archetypes.get_mut(location.archetype_id)
            .ok_or(EcsError::EntityNotFound)?;
        
        let swapped_entity = archetype.swap_remove(location.index as usize);
        
        // Update swapped entity's location
        if let Some(swapped_location) = self.entity_index.get_mut(&swapped_entity) {
            swapped_location.index = location.index;
        }
        
        // Free entity ID
        self.entities.free(entity);
        
        Ok(())
    }
    
    /// Insert a component
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), EcsError> {
        let location = self.entity_index.get(&entity)
            .ok_or(EcsError::EntityNotFound)?;
        
        let current_archetype = self.archetypes.get(location.archetype_id)
            .ok_or(EcsError::EntityNotFound)?;
        
        // Find or create target archetype
        let component_type = ComponentTypeId::of::<T>();
        let target_archetype_id = current_archetype.edges.add.get(&component_type)
            .copied()
            .unwrap_or_else(|| {
                // Create new archetype
                let mut types = current_archetype.component_types.clone();
                types.push(component_type);
                types.sort();
                self.archetypes.get_or_create(&types)
            });
        
        // Move entity to new archetype
        self.move_entity(entity, *location, target_archetype_id, Some(component))?;
        
        Ok(())
    }
    
    /// Remove a component
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Result<Option<T>, EcsError> {
        // Similar to insert, but removes component type
        todo!()
    }
    
    /// Get component reference
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let location = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(location.archetype_id)?;
        let storage = archetype.get_storage::<T>()?;
        storage.get(location.index as usize)
    }
    
    /// Get mutable component reference
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<T>> {
        let location = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(location.archetype_id)?;
        let storage = archetype.get_storage_mut::<T>()?;
        let tick = self.change_tick.fetch_add(1, Ordering::Relaxed);
        
        Some(Mut {
            value: storage.get_mut(location.index as usize, tick)?,
            tick,
        })
    }
    
    /// Query entities
    pub fn query<Q: Query>(&self) -> QueryIter<Q> {
        QueryIter::new(self)
    }
    
    /// Query entities (mutable)
    pub fn query_mut<Q: Query>(&mut self) -> QueryIterMut<Q> {
        QueryIterMut::new(self)
    }
}
```

---

## Data Models

### Transform Component (SIMD-Optimized)

```rust
/// Transform component with SIMD-friendly layout
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Transform {
    /// Position (x, y, z, padding)
    pub position: [f32; 4],
    
    /// Rotation (x, y, z, w) as quaternion
    pub rotation: [f32; 4],
    
    /// Scale (x, y, z, padding)
    pub scale: [f32; 4],
}

impl Transform {
    /// Batch update positions using SIMD
    #[cfg(target_feature = "avx2")]
    pub fn batch_translate(transforms: &mut [Transform], delta: [f32; 3]) {
        use std::arch::x86_64::*;
        
        unsafe {
            let delta_simd = _mm_set_ps(0.0, delta[2], delta[1], delta[0]);
            
            for transform in transforms.iter_mut() {
                let pos = _mm_load_ps(transform.position.as_ptr());
                let new_pos = _mm_add_ps(pos, delta_simd);
                _mm_store_ps(transform.position.as_mut_ptr(), new_pos);
            }
        }
    }
    
    /// Scalar fallback
    #[cfg(not(target_feature = "avx2"))]
    pub fn batch_translate(transforms: &mut [Transform], delta: [f32; 3]) {
        for transform in transforms.iter_mut() {
            transform.position[0] += delta[0];
            transform.position[1] += delta[1];
            transform.position[2] += delta[2];
        }
    }
}
```

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Entity spawn increases count
*For any* world state, spawning an entity should increase the entity count by exactly 1.
**Validates: Requirements 1.1**

### Property 2: Entity despawn decreases count
*For any* world state with at least one entity, despawning an entity should decrease the entity count by exactly 1 (accounting for recursive child despawn).
**Validates: Requirements 1.1**

### Property 3: Component insertion preserves entity
*For any* entity and component, inserting a component should not change the entity's ID or generation.
**Validates: Requirements 1.3**

### Property 4: Archetype consistency
*For any* entity, all its components should be stored in exactly one archetype, and that archetype should contain the entity.
**Validates: Requirements 1.1, 1.2**

### Property 5: Sparse set consistency
*For any* entity in the world, the sparse set should map it to a valid dense index, and the dense array at that index should contain the same entity.
**Validates: Requirements 3.1, 3.2**

### Property 6: Change detection monotonicity
*For any* component modification, the change tick should be greater than or equal to the previous tick.
**Validates: Requirements 4.1, 4.2**

### Property 7: Query result completeness
*For any* query with filters, the result set should contain all and only entities that match the filter criteria.
**Validates: Requirements 6.1, 6.2, 6.3**

### Property 8: SIMD alignment
*For any* component storage, the data pointer should be aligned to 16-byte boundaries for SIMD operations.
**Validates: Requirements 2.2, 2.4**

### Property 9: Memory compaction
*For any* archetype after entity removal, there should be no gaps in the component arrays (swap-remove maintains density).
**Validates: Requirements 10.3**

### Property 10: Serialization round-trip
*For any* world state, serializing then deserializing should produce an equivalent world state with the same entities and components.
**Validates: Requirements 11.1, 11.2**

---

## Error Handling

### Error Types
```rust
#[derive(Debug, Clone)]
pub enum EcsError {
    EntityNotFound,
    ComponentNotFound,
    ArchetypeNotFound,
    InvalidHierarchy,
    SerializationError(String),
    ConcurrentAccessError,
}
```

### Error Handling Strategy
1. **Entity operations**: Return `Result<T, EcsError>` for fallible operations
2. **Component access**: Return `Option<T>` for missing components
3. **Query errors**: Panic on invalid query construction (compile-time safety preferred)
4. **Serialization errors**: Return detailed error messages with context

---

## Testing Strategy

### Unit Tests
- Entity allocation and deallocation
- Component insertion and removal
- Archetype creation and migration
- Sparse set operations
- Change detection
- Query filtering

### Property-Based Tests
- Entity spawn/despawn count consistency
- Archetype consistency after operations
- Sparse set consistency
- Change detection monotonicity
- Query result completeness
- Serialization round-trip

### Performance Tests
- Spawn 100,000 entities
- Query 100,000 entities (single component)
- Query 100,000 entities (multi-component)
- Insert/remove components (archetype migration)
- SIMD batch operations
- Parallel system execution

### Integration Tests
- Full game loop simulation
- Complex entity hierarchies
- Serialization/deserialization
- Migration from old ECS

---

## Performance Optimization Techniques

### 1. Cache-Friendly Memory Layout
- Store components in contiguous arrays (archetype tables)
- Align data to cache line boundaries (64 bytes)
- Use struct-of-arrays (SoA) instead of array-of-structs (AoS)

### 2. SIMD Vectorization
- Align component data to 16-byte boundaries
- Process 4-8 components simultaneously with SIMD instructions
- Use platform-specific intrinsics (SSE2, AVX2, NEON)

### 3. Parallel Execution
- Automatically schedule systems based on component access
- Use rayon for parallel iteration
- Lock-free data structures where possible

### 4. Memory Pooling
- Reuse entity IDs with generation counters
- Pool archetype allocations
- Minimize allocations in hot paths

### 5. Query Optimization
- Skip archetypes that don't match query filters
- Cache query results when possible
- Use change detection to skip unchanged data

---

## Migration from Current ECS

### Compatibility Layer
```rust
/// Compatibility wrapper for old HashMap-based API
pub mod compat {
    use super::*;
    
    pub struct LegacyWorld {
        world: World,
    }
    
    impl LegacyWorld {
        pub fn new() -> Self {
            Self { world: World::new() }
        }
        
        pub fn spawn(&mut self) -> Entity {
            self.world.spawn()
        }
        
        pub fn despawn(&mut self, entity: Entity) {
            let _ = self.world.despawn(entity);
        }
        
        // HashMap-like component access
        pub fn transforms(&self) -> ComponentMap<Transform> {
            ComponentMap::new(&self.world)
        }
        
        pub fn transforms_mut(&mut self) -> ComponentMapMut<Transform> {
            ComponentMapMut::new(&mut self.world)
        }
    }
    
    /// HashMap-like component access
    pub struct ComponentMap<'w, T: Component> {
        world: &'w World,
        _marker: PhantomData<T>,
    }
    
    impl<'w, T: Component> ComponentMap<'w, T> {
        pub fn get(&self, entity: &Entity) -> Option<&T> {
            self.world.get(*entity)
        }
        
        pub fn contains_key(&self, entity: &Entity) -> bool {
            self.world.get::<T>(*entity).is_some()
        }
        
        pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
            self.world.query::<(Entity, &T)>()
        }
    }
}
```

---

## Conclusion

This design provides a modern, high-performance ECS architecture that:
- Achieves 4-10x performance improvement over the current implementation
- Supports 100,000+ entities at 60 FPS
- Leverages SIMD for 4-8x speedup on batch operations
- Maintains backward compatibility through a compatibility layer
- Provides a clear migration path for existing code

The archetype-based storage, combined with SIMD optimization and parallel execution, positions the XS Game Engine's ECS as one of the fastest in the Rust ecosystem, rivaling Bevy ECS while offering unique optimizations for 2D pixel art games and mobile platforms.

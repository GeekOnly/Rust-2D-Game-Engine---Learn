# ECS Redesign - Requirements Document

## Introduction

This document outlines the requirements for redesigning the XS Game Engine's Entity Component System (ECS) to achieve AAA-level performance through modern best practices, memory optimization, and SIMD acceleration. The redesign will be inspired by Bevy ECS's archetype-based architecture while maintaining compatibility with our existing API.

## Glossary

- **ECS**: Entity Component System - A design pattern for game engines
- **Entity**: A unique identifier (ID) representing a game object
- **Component**: Data associated with an entity (e.g., Transform, Sprite)
- **System**: Logic that operates on entities with specific components
- **Archetype**: A unique combination of component types
- **Table**: Storage for entities sharing the same archetype
- **SIMD**: Single Instruction Multiple Data - CPU parallelization
- **SoA**: Struct of Arrays - Memory layout for SIMD optimization
- **Cache Line**: 64-byte CPU cache unit
- **Sparse Set**: Data structure for fast entity-component lookup

---

## Requirements

### Requirement 1: High-Performance Archetype-Based Storage

**User Story:** As a game developer, I want the ECS to handle 100,000+ entities at 60 FPS, so that I can create large-scale games without performance bottlenecks.

#### Acceptance Criteria

1. WHEN the system stores entities THEN it SHALL use archetype-based table storage for cache-friendly memory layout
2. WHEN entities share the same component types THEN the system SHALL store them contiguously in the same archetype table
3. WHEN a component is added or removed THEN the system SHALL move the entity to the appropriate archetype table
4. WHEN iterating over components THEN the system SHALL achieve linear memory access patterns for optimal CPU cache utilization
5. WHEN querying 100,000 entities THEN the system SHALL complete iteration in under 1ms on modern hardware

---

### Requirement 2: SIMD-Optimized Component Storage

**User Story:** As a performance-conscious developer, I want component data to be SIMD-optimized, so that batch operations run 4-8x faster through CPU vectorization.

#### Acceptance Criteria

1. WHEN storing component data THEN the system SHALL use Struct-of-Arrays (SoA) layout instead of Array-of-Structs (AoS)
2. WHEN processing Transform components THEN the system SHALL align data to 16-byte boundaries for SIMD operations
3. WHEN updating positions in batch THEN the system SHALL use SIMD instructions to process 4-8 components simultaneously
4. WHEN allocating component storage THEN the system SHALL align memory to cache line boundaries (64 bytes)
5. WHEN accessing component arrays THEN the system SHALL ensure zero padding overhead between elements

---

### Requirement 3: Sparse Set Entity-Component Mapping

**User Story:** As a developer, I want O(1) component access by entity ID, so that random entity lookups remain fast even with millions of entities.

#### Acceptance Criteria

1. WHEN looking up a component by entity ID THEN the system SHALL use sparse set data structure for O(1) access
2. WHEN an entity is despawned THEN the system SHALL remove it from sparse sets in O(1) time
3. WHEN checking if an entity has a component THEN the system SHALL complete the check in constant time
4. WHEN iterating all entities with a component THEN the system SHALL use the dense array for linear iteration
5. WHEN memory usage is measured THEN sparse sets SHALL use less than 16 bytes per entity on average

---

### Requirement 4: Change Detection System

**User Story:** As a developer, I want to detect when components change, so that I can optimize systems to only process modified data.

#### Acceptance Criteria

1. WHEN a component is modified THEN the system SHALL mark it as changed with a generation counter
2. WHEN a system queries for changed components THEN it SHALL receive only entities with modifications since last check
3. WHEN multiple systems read the same component THEN each system SHALL track changes independently
4. WHEN a component is added or removed THEN the system SHALL trigger change detection
5. WHEN querying unchanged data THEN the system SHALL skip processing to save CPU cycles

---

### Requirement 5: Parallel System Execution

**User Story:** As a developer, I want systems to run in parallel, so that I can utilize multi-core CPUs for maximum performance.

#### Acceptance Criteria

1. WHEN systems have no conflicting component access THEN they SHALL execute in parallel automatically
2. WHEN a system reads a component THEN multiple systems SHALL read it simultaneously
3. WHEN a system writes a component THEN no other system SHALL access it during execution
4. WHEN scheduling systems THEN the system SHALL detect read/write conflicts at compile time
5. WHEN running on a 4-core CPU THEN parallel systems SHALL achieve 3-4x speedup over sequential execution

---

### Requirement 6: Query Filtering and Iteration

**User Story:** As a developer, I want powerful query capabilities, so that I can efficiently filter and iterate entities based on complex criteria.

#### Acceptance Criteria

1. WHEN querying entities THEN the system SHALL support With<T> filters for required components
2. WHEN querying entities THEN the system SHALL support Without<T> filters for excluded components
3. WHEN querying entities THEN the system SHALL support Optional<T> for components that may or may not exist
4. WHEN combining filters THEN the system SHALL optimize query execution to skip irrelevant archetypes
5. WHEN iterating query results THEN the system SHALL provide both immutable and mutable access patterns

---

### Requirement 7: Component Bundles

**User Story:** As a developer, I want to add multiple components at once, so that entity creation is fast and ergonomic.

#### Acceptance Criteria

1. WHEN spawning an entity with multiple components THEN the system SHALL support bundle insertion in a single operation
2. WHEN a bundle is defined THEN it SHALL be a struct containing multiple components
3. WHEN inserting a bundle THEN the system SHALL place the entity in the correct archetype immediately
4. WHEN removing a bundle THEN the system SHALL remove all components in the bundle atomically
5. WHEN using bundles THEN entity creation SHALL be 2-3x faster than individual component insertion

---

### Requirement 8: Resource Management

**User Story:** As a developer, I want global resources accessible to all systems, so that I can share state like time, input, and configuration.

#### Acceptance Criteria

1. WHEN a resource is registered THEN it SHALL be accessible to all systems
2. WHEN multiple systems read a resource THEN they SHALL access it concurrently
3. WHEN a system writes a resource THEN it SHALL have exclusive access
4. WHEN a resource doesn't exist THEN the system SHALL provide a clear error message
5. WHEN resources are accessed THEN the system SHALL use the same borrow checking as components

---

### Requirement 9: Backward Compatibility Layer

**User Story:** As an existing user, I want the new ECS to support the old API, so that I can migrate gradually without breaking existing code.

#### Acceptance Criteria

1. WHEN using the old HashMap-based API THEN it SHALL continue to work with a compatibility wrapper
2. WHEN migrating code THEN the system SHALL provide clear deprecation warnings
3. WHEN both APIs are used THEN they SHALL operate on the same underlying data
4. WHEN serialization is used THEN both old and new formats SHALL be supported
5. WHEN performance is measured THEN the compatibility layer SHALL add less than 10% overhead

---

### Requirement 10: Memory Efficiency

**User Story:** As a developer targeting mobile platforms, I want minimal memory overhead, so that my game runs well on devices with limited RAM.

#### Acceptance Criteria

1. WHEN storing 100,000 entities THEN the system SHALL use less than 50MB of memory for entity metadata
2. WHEN archetypes are empty THEN the system SHALL deallocate their storage automatically
3. WHEN components are removed THEN the system SHALL compact storage to avoid fragmentation
4. WHEN measuring overhead THEN entity IDs SHALL use 4 bytes and generation counters SHALL use 4 bytes
5. WHEN allocating tables THEN the system SHALL use a pooled allocator to reduce allocation overhead

---

### Requirement 11: Serialization and Reflection

**User Story:** As a developer, I want to serialize and deserialize the entire world, so that I can save/load game state and support hot-reloading.

#### Acceptance Criteria

1. WHEN serializing the world THEN the system SHALL save all entities, components, and archetypes to JSON/binary
2. WHEN deserializing the world THEN the system SHALL restore the exact state including entity IDs
3. WHEN components are registered THEN they SHALL provide reflection metadata for serialization
4. WHEN a component type is unknown THEN the system SHALL skip it with a warning
5. WHEN serializing 10,000 entities THEN it SHALL complete in under 100ms

---

### Requirement 12: Debugging and Profiling Tools

**User Story:** As a developer, I want built-in debugging tools, so that I can diagnose performance issues and inspect ECS state.

#### Acceptance Criteria

1. WHEN debugging is enabled THEN the system SHALL provide entity inspector with component values
2. WHEN profiling systems THEN the system SHALL report execution time per system
3. WHEN analyzing memory THEN the system SHALL show archetype table sizes and fragmentation
4. WHEN querying is slow THEN the system SHALL provide query performance statistics
5. WHEN visualizing the ECS THEN the system SHALL export archetype graph in DOT format

---

### Requirement 13: Entity Relationships (Flecs-Inspired)

**User Story:** As a developer, I want native entity relationships, so that I can model complex hierarchies and inheritance without manual HashMap management.

#### Acceptance Criteria

1. WHEN adding a relationship THEN the system SHALL support ChildOf, IsA, and custom relationship types
2. WHEN querying relationships THEN the system SHALL provide O(1) lookup for relationship queries
3. WHEN despawning a parent entity THEN the system SHALL automatically despawn all children
4. WHEN querying children THEN the system SHALL return all entities with ChildOf relationship to parent
5. WHEN using IsA relationships THEN the system SHALL support component inheritance

---

### Requirement 14: Network Replication System

**User Story:** As a multiplayer developer, I want automatic network replication, so that entity state synchronizes across clients without manual code.

#### Acceptance Criteria

1. WHEN an entity has NetworkId component THEN the system SHALL replicate it to connected clients
2. WHEN a component changes THEN the system SHALL send delta updates to reduce bandwidth
3. WHEN replication mode is Proximity THEN the system SHALL only replicate to nearby clients
4. WHEN bandwidth is limited THEN the system SHALL prioritize high-priority entities
5. WHEN packet loss occurs THEN the system SHALL use redundancy to ensure delivery

---

### Requirement 15: Client-Side Prediction and Rollback

**User Story:** As a multiplayer developer, I want client-side prediction, so that player input feels responsive despite network latency.

#### Acceptance Criteria

1. WHEN player input is received THEN the system SHALL predict entity state immediately
2. WHEN server state arrives THEN the system SHALL compare with predicted state
3. WHEN prediction error exceeds threshold THEN the system SHALL rollback and replay inputs
4. WHEN replaying inputs THEN the system SHALL use stored input buffer from confirmed tick
5. WHEN prediction is accurate THEN the system SHALL skip rollback to save CPU

---

### Requirement 16: Lag Compensation

**User Story:** As a multiplayer developer, I want server-side lag compensation, so that hit detection is fair for all players regardless of latency.

#### Acceptance Criteria

1. WHEN processing player action THEN the system SHALL rewind world to player's view time
2. WHEN rewinding THEN the system SHALL use historical entity states from snapshot buffer
3. WHEN hit detection completes THEN the system SHALL restore world to current state
4. WHEN latency exceeds 200ms THEN the system SHALL cap compensation to prevent abuse
5. WHEN recording history THEN the system SHALL store last 256 ticks (4 seconds @ 60Hz)

---

### Requirement 17: Interest Management (Spatial Partitioning)

**User Story:** As a multiplayer developer, I want interest management, so that clients only receive updates for nearby entities in large worlds.

#### Acceptance Criteria

1. WHEN an entity enters client's interest radius THEN the system SHALL send full entity state
2. WHEN an entity exits interest radius THEN the system SHALL stop sending updates
3. WHEN querying nearby entities THEN the system SHALL use spatial grid for O(1) lookup
4. WHEN interest radius is 100m THEN the system SHALL update interest area every frame
5. WHEN world is large THEN the system SHALL support millions of entities with spatial partitioning

---

### Requirement 18: Deterministic Simulation (Lockstep)

**User Story:** As an RTS/Fighting game developer, I want deterministic simulation, so that all clients stay in perfect sync using lockstep netcode.

#### Acceptance Criteria

1. WHEN running simulation THEN the system SHALL use fixed timestep (1/60 second)
2. WHEN applying inputs THEN the system SHALL wait for all client inputs before advancing tick
3. WHEN calculating physics THEN the system SHALL use fixed-point math for determinism
4. WHEN replaying inputs THEN the system SHALL produce identical results on all clients
5. WHEN detecting desync THEN the system SHALL provide hash comparison for debugging

---

### Requirement 19: Anti-Cheat System

**User Story:** As a multiplayer developer, I want server-side validation, so that cheaters cannot exploit the game.

#### Acceptance Criteria

1. WHEN validating movement THEN the system SHALL reject speeds exceeding max speed + 10% tolerance
2. WHEN validating actions THEN the system SHALL enforce cooldowns server-side
3. WHEN detecting anomalies THEN the system SHALL track statistics (headshot ratio, wall shots)
4. WHEN violations occur THEN the system SHALL log violations with severity levels
5. WHEN cheat is detected THEN the system SHALL provide ban/kick functionality

---

### Requirement 20: Bandwidth Optimization

**User Story:** As a multiplayer developer, I want adaptive quality, so that the game works well on slow connections.

#### Acceptance Criteria

1. WHEN bandwidth is low THEN the system SHALL reduce update frequency automatically
2. WHEN packet loss is high THEN the system SHALL increase compression level
3. WHEN bandwidth improves THEN the system SHALL increase quality gradually
4. WHEN measuring bandwidth THEN the system SHALL track current, average, and packet loss
5. WHEN optimizing THEN the system SHALL achieve <100 KB/s per client with compression

---

### Requirement 21: Production Save/Load System

**User Story:** As a game developer, I want robust save/load, so that players can save progress reliably.

#### Acceptance Criteria

1. WHEN saving world THEN the system SHALL serialize all entities and components to disk
2. WHEN loading world THEN the system SHALL restore exact state including entity IDs
3. WHEN saving THEN the system SHALL compress data to reduce file size
4. WHEN saving THEN the system SHALL encrypt data to prevent tampering
5. WHEN auto-saving THEN the system SHALL save every N seconds without blocking gameplay

---

### Requirement 22: Platform-Specific Optimizations

**User Story:** As a cross-platform developer, I want platform-specific optimizations, so that the game runs well on all devices.

#### Acceptance Criteria

1. WHEN running on mobile THEN the system SHALL adjust quality based on thermal state
2. WHEN running on Switch THEN the system SHALL optimize for docked vs handheld mode
3. WHEN thermal throttling occurs THEN the system SHALL reduce entity count and physics substeps
4. WHEN battery is low THEN the system SHALL enable battery-saver mode
5. WHEN platform is detected THEN the system SHALL apply appropriate SIMD instructions (SSE2, AVX2, NEON)

---

## Performance Targets

### Baseline Performance (Current HashMap-based ECS)
- Spawn 10,000 entities: ~530 µs (53 ns/entity)
- Query single component (10,000 entities): ~23 µs (2.3 ns/entity)
- Query multi-component (10,000 entities): ~203 µs (20.3 ns/entity)
- Game scenario (1,000 entities × 60 frames): ~40 µs/frame

### Target Performance (New Archetype-based ECS)

#### Single-Player (Offline)
- Spawn 10,000 entities: **<200 µs** (2.6x faster)
- Query single component (10,000 entities): **<5 µs** (4.6x faster)
- Query multi-component (10,000 entities): **<20 µs** (10x faster)
- Game scenario (1,000 entities × 60 frames): **<10 µs/frame** (4x faster)
- Support 100,000+ entities at 60 FPS
- Save/Load time: **<5 seconds** for full world state

#### Multiplayer (Online)
- Max concurrent players: **100+**
- Server tick rate: **60 Hz**
- Client update rate: **20-60 Hz** (adaptive)
- Bandwidth per client: **<100 KB/s** (with compression)
- Latency compensation: **200ms max**
- Interest radius: **100m** (spatial partitioning)
- Snapshot buffer: **256 ticks** (~4 seconds @ 60 Hz)
- Packet size: **<1 KB** average
- Compression ratio: **3:1** (delta compression)
- Prediction error: **<10cm** position accuracy

### SIMD Performance Targets
- Transform batch update (10,000 entities): **<50 µs** with SIMD vs ~200 µs scalar
- Physics integration (10,000 entities): **<100 µs** with SIMD vs ~400 µs scalar
- Collision broad-phase (10,000 entities): **<200 µs** with SIMD vs ~800 µs scalar

---

## Comparison with Bevy ECS

### Features to Adopt from Bevy
1. ✅ Archetype-based storage
2. ✅ Sparse set entity-component mapping
3. ✅ Change detection with generation counters
4. ✅ Parallel system execution with automatic scheduling
5. ✅ Query filtering (With, Without, Optional)
6. ✅ Component bundles
7. ✅ Resource management
8. ✅ System parameters and dependency injection

### Features to Keep from Current ECS
1. ✅ Simple HashMap-based API (as compatibility layer)
2. ✅ Unity-like component names (Transform, Sprite, Camera)
3. ✅ Parent-child hierarchy
4. ✅ Entity tags and layers
5. ✅ Prefab system
6. ✅ JSON serialization

### Unique Features for XS Engine
1. ✅ SIMD-optimized component storage (more aggressive than Bevy)
2. ✅ Mobile-first memory optimization
3. ✅ Pixel art specific components (SpriteSheet, Tilemap)
4. ✅ Lua script integration
5. ✅ LDtk/Tiled map loader integration

---

## Non-Functional Requirements

### Performance
- Support 100,000+ active entities at 60 FPS
- Query iteration: <10 ns per entity
- System execution: Parallel on multi-core CPUs
- Memory usage: <500 bytes per entity average

### Compatibility
- Maintain API compatibility with existing code
- Support gradual migration path
- Preserve serialization format compatibility

### Maintainability
- Clear separation between storage and API layers
- Comprehensive benchmarks and tests
- Property-based testing for correctness
- Documentation with examples

### Platform Support
- Windows, macOS, Linux (desktop)
- iOS, Android (mobile)
- WebAssembly (web)
- SIMD support: SSE2, AVX2, NEON

---

## Success Criteria

The ECS redesign will be considered successful when:

1. ✅ All performance targets are met or exceeded
2. ✅ Existing code continues to work with compatibility layer
3. ✅ Benchmarks show 4-10x improvement over current implementation
4. ✅ Memory usage is reduced by 30-50%
5. ✅ 100% of existing tests pass
6. ✅ New property-based tests validate correctness
7. ✅ Documentation is complete with migration guide
8. ✅ Example projects demonstrate new features

---

## Migration Strategy

### Phase 1: Core Architecture (Month 1-2)
- Implement archetype-based storage
- Implement sparse set entity mapping
- Basic query system
- Compatibility layer for old API

### Phase 2: Performance Optimization (Month 2-3)
- SIMD-optimized component storage
- Parallel system execution
- Change detection
- Memory optimization

### Phase 3: Advanced Features (Month 3-4)
- Component bundles
- Resource management
- Query filtering
- Debugging tools

### Phase 4: Migration and Polish (Month 4-5)
- Migrate existing code
- Performance benchmarking
- Documentation
- Example projects

---

## References

- [Bevy ECS Architecture](https://bevyengine.org/learn/book/getting-started/ecs/)
- [hecs Documentation](https://docs.rs/hecs/)
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/)
- [SIMD Programming Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/)
- [Cache-Friendly Code](https://www.aristeia.com/TalkNotes/codedive-CPUCachesHandouts.pdf)

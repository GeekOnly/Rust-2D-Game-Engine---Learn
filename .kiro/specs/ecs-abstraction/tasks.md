# Implementation Plan

- [x] 1. Define core abstraction traits and error types





  - Create `EcsError` enum with all error variants
  - Define `EcsWorld` trait with entity lifecycle methods
  - Define `ComponentAccess` trait for type-safe component operations
  - Define `Query` and `QueryMut` traits for entity iteration
  - Define `Serializable` trait for persistence
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2_

- [x] 1.1 Write property test for entity spawn count


  - **Property 2: Entity spawn increases count**
  - **Validates: Requirements 1.1**

- [x] 1.2 Write property test for entity despawn count


  - **Property 3: Entity despawn decreases count**
  - **Validates: Requirements 1.1**

- [ ] 2. Implement component access macro
  - Create `impl_component_access!` macro to reduce boilerplate
  - Add documentation and usage examples
  - Test macro with sample component types
  - _Requirements: 1.2, 2.1-2.7_

- [ ] 3. Implement EcsWorld trait for HashMap-based World
  - Implement `spawn()` method
  - Implement `despawn()` method with recursive child removal
  - Implement `is_alive()` method
  - Implement `clear()` method
  - Implement `entity_count()` method
  - Implement hierarchy methods: `set_parent()`, `get_parent()`, `get_children()`
  - _Requirements: 1.1, 5.1, 5.2, 7.1, 7.2, 7.3, 7.4_

- [ ] 3.1 Write property test for despawn cleanup
  - **Property 4: Despawn removes all components**
  - **Validates: Requirements 1.1**

- [ ] 3.2 Write property test for recursive despawn
  - **Property 5: Recursive despawn removes children**
  - **Validates: Requirements 7.2**

- [ ] 3.3 Write property test for hierarchy consistency
  - **Property 6: Parent-child relationship consistency**
  - **Validates: Requirements 7.1, 7.4**

- [ ] 3.4 Write property test for get children
  - **Property 7: Get children returns all direct children**
  - **Validates: Requirements 7.3**

- [ ] 4. Implement ComponentAccess for all component types
  - Use macro to implement for: Transform, Sprite, Collider, Mesh, Camera, Script, EntityTag
  - Manually implement for tuple types: velocity (f32, f32)
  - Manually implement for primitive types: active (bool), layers (u8), names (String)
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

- [ ] 4.1 Write property test for component round trip
  - **Property 1: Component insert-get round trip**
  - **Validates: Requirements 1.2, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7**

- [ ] 5. Implement query system for HashMap backend
  - Create `SingleQuery` struct for immutable single-component queries
  - Create `SingleQueryMut` struct for mutable single-component queries
  - Implement `Query` trait for `SingleQuery`
  - Implement `QueryMut` trait for `SingleQueryMut`
  - Add helper methods on `World` to create queries
  - _Requirements: 1.3, 8.1, 8.3, 8.4_

- [ ] 5.1 Write property test for single component query
  - **Property 8: Single component query correctness**
  - **Validates: Requirements 8.1**

- [ ] 5.2 Write property test for mutable query
  - **Property 10: Mutable query modifies components**
  - **Validates: Requirements 8.3**

- [ ] 6. Implement multi-component query support
  - Create `DoubleQuery` for two-component queries
  - Create `TripleQuery` for three-component queries
  - Implement query filtering logic
  - Add query builder pattern for complex queries
  - _Requirements: 8.2_

- [ ] 6.1 Write property test for multi-component query
  - **Property 9: Multi-component query correctness**
  - **Validates: Requirements 8.2**

- [ ] 7. Implement Serializable trait for World
  - Implement `save_to_json()` using existing serialization code
  - Implement `load_from_json()` using existing deserialization code
  - Ensure all component types are included in serialization
  - Handle hierarchy serialization (parents/children)
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 7.1 Write property test for serialization round trip
  - **Property 11: Serialization round trip preserves world state**
  - **Validates: Requirements 4.1, 4.2, 4.3, 4.4**

- [ ] 8. Add comprehensive documentation
  - Document all traits with usage examples
  - Document error types and when they occur
  - Create guide for implementing new backends
  - Add inline examples in trait documentation
  - Document performance characteristics
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 9. Create example alternative backend
  - Implement a simple Vec-based ECS backend
  - Implement all required traits for the Vec backend
  - Demonstrate backend switching in examples
  - _Requirements: 1.5_

- [ ] 9.1 Write property test for backend equivalence
  - **Property 12: Backend-agnostic operations**
  - **Validates: Requirements 1.5**

- [ ] 10. Update existing code to use abstraction
  - Identify editor code that directly uses World
  - Refactor to use trait-based interfaces where beneficial
  - Ensure backward compatibility is maintained
  - _Requirements: 5.2, 5.3, 5.4_

- [ ] 10.1 Write unit tests for editor integration
  - Test entity creation through abstraction
  - Test entity modification through abstraction
  - Test entity deletion through abstraction
  - _Requirements: 5.4_

- [ ] 11. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 12. Add benchmarks for performance validation
  - Create benchmarks for entity spawn/despawn
  - Create benchmarks for component access
  - Create benchmarks for queries
  - Compare HashMap backend with and without abstraction
  - Document performance characteristics
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

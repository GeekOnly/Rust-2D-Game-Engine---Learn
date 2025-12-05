# Implementation Plan

- [x] 1. Set up project structure and testing framework




  - Create test directory structure for tilemap management tests
  - Add QuickCheck dependency to Cargo.toml for property-based testing
  - Create test utilities module for generating mock LDtk data
  - _Requirements: All (testing infrastructure)_

- [ ] 2. Implement core Grid and Tilemap components
  - [ ] 2.1 Verify Grid component implementation
    - Review existing Grid component in ecs/src/components/grid.rs
    - Ensure all coordinate conversion methods are correct
    - _Requirements: 7.1_

  - [ ]* 2.2 Write property test for Grid coordinate conversions
    - **Property: Grid Coordinate Round Trip**
    - **Validates: Requirements 7.1**
    - Test that world_to_cell(cell_to_world(x, y)) == (x, y) for all coordinates

  - [ ] 2.3 Verify Tilemap component implementation
    - Review existing Tilemap component
    - Ensure tile storage and retrieval works correctly
    - _Requirements: 7.2_

  - [ ]* 2.4 Write property test for Tilemap tile operations
    - **Property: Tilemap Tile Round Trip**
    - **Validates: Requirements 7.2**
    - Test that get_tile(x, y) returns the tile set by set_tile(x, y, tile)

- [ ] 3. Implement LdtkLoader with Grid hierarchy
  - [ ] 3.1 Implement load_project_with_grid method
    - Create Grid Entity as root
    - Load layers as children of Grid
    - Set up proper parent-child relationships
    - _Requirements: 1.2, 7.1, 7.2_

  - [ ]* 3.2 Write property test for map loading hierarchy
    - **Property 1: Map Loading Creates Hierarchy**
    - **Validates: Requirements 1.2, 7.1, 7.2**
    - Test that loading creates exactly one Grid with all layers as children

  - [ ] 3.3 Implement load_project_with_grid_and_colliders method
    - Call load_project_with_grid
    - Generate colliders automatically
    - Set colliders as children of Grid
    - _Requirements: 2.1, 2.2, 7.3_

  - [ ]* 3.4 Write property test for auto-generated colliders
    - **Property 5: Auto-Generate Colliders**
    - **Validates: Requirements 2.1**
    - Test that maps with IntGrid layers automatically generate colliders

  - [ ]* 3.5 Write property test for collider parent-child relationship
    - **Property 6: Colliders Are Grid Children**
    - **Validates: Requirements 2.2, 7.3**
    - Test that all colliders have Grid Entity as parent

- [ ] 4. Implement composite collider optimization
  - [ ] 4.1 Implement greedy meshing algorithm
    - Implement find_rectangles function
    - Implement find_largest_rectangle function
    - Optimize for minimal collider count
    - _Requirements: 2.4_

  - [ ]* 4.2 Write property test for composite optimization
    - **Property 8: Composite Collider Optimization**
    - **Validates: Requirements 2.4**
    - Test that N adjacent tiles produce fewer than N colliders

  - [ ] 4.3 Implement generate_composite_colliders_from_intgrid
    - Parse IntGrid CSV data
    - Apply greedy meshing algorithm
    - Create collider entities with correct sizes
    - _Requirements: 2.1, 2.4_

  - [ ]* 4.4 Write property test for collision value filtering
    - **Property 30: Collision Value Filtering**
    - **Validates: Requirements 9.4**
    - Test that only tiles with specified collision value get colliders

- [ ] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Implement MapManager core functionality
  - [ ] 6.1 Implement MapManager structure and basic methods
    - Create MapManager struct with HashMap for loaded maps
    - Implement new(), scan_ldtk_files(), and file scanning
    - _Requirements: 1.1, 1.5_

  - [ ] 6.2 Implement load_map method
    - Call LdtkLoader::load_project_with_grid_and_colliders
    - Store LoadedMap with Grid, layers, and colliders
    - Track file metadata (path, last_modified)
    - _Requirements: 1.2, 2.1_

  - [ ]* 6.3 Write property test for multiple maps display
    - **Property 4: Multiple Maps Display Correctly**
    - **Validates: Requirements 1.5**
    - Test that loading N maps results in N entries in loaded_maps

  - [ ] 6.4 Implement reload_map method
    - Preserve Grid Entity ID during reload
    - Update layers and colliders
    - Maintain layer visibility states
    - _Requirements: 1.3, 3.5, 6.3_

  - [ ]* 6.5 Write property test for reload preserves Grid
    - **Property 2: Reload Preserves Grid Entity**
    - **Validates: Requirements 1.3**
    - Test that Grid Entity ID remains the same after reload

  - [ ]* 6.6 Write property test for visibility persists across reload
    - **Property 11: Visibility Persists Across Reload**
    - **Validates: Requirements 3.5, 6.3**
    - Test that custom visibility states are preserved after reload

  - [ ] 6.7 Implement unload_map method
    - Despawn Grid Entity
    - Verify all children are automatically despawned
    - Remove from loaded_maps HashMap
    - _Requirements: 1.4, 2.5_

  - [ ]* 6.8 Write property test for unload cleanup
    - **Property 3: Unload Cleans Up Hierarchy**
    - **Validates: Requirements 1.4, 2.5**
    - Test that unloading despawns Grid and all children

- [ ] 7. Implement collider management operations
  - [ ] 7.1 Implement regenerate_colliders method
    - Despawn existing colliders
    - Generate new colliders from current IntGrid data
    - Set as children of Grid
    - Update LoadedMap tracking
    - _Requirements: 2.3_

  - [ ]* 7.2 Write property test for regenerate colliders
    - **Property 7: Regenerate Colliders Round Trip**
    - **Validates: Requirements 2.3**
    - Test that regenerating produces colliders matching current data

  - [ ] 7.3 Implement clean_up_colliders and clean_up_all_colliders
    - Remove colliders for specific map or all maps
    - Update LoadedMap tracking
    - _Requirements: 2.3_

  - [ ] 7.4 Implement is_map_entity helper
    - Check if entity belongs to any loaded map
    - Used for hierarchy filtering
    - _Requirements: 7.4_

- [ ] 8. Implement layer visibility management
  - [ ] 8.1 Implement toggle_layer_visibility method
    - Toggle active component on layer entity
    - Update LayerInfo tracking
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ]* 8.2 Write property test for visibility toggle idempotence
    - **Property 9: Visibility Toggle Idempotence**
    - **Validates: Requirements 3.1**
    - Test that toggling twice returns to original state

  - [ ]* 8.3 Write property test for visibility state synchronization
    - **Property 10: Visibility State Synchronization**
    - **Validates: Requirements 3.2, 3.3**
    - Test that hiding sets active=false and showing sets active=true

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement Maps Panel UI
  - [ ] 10.1 Create render_maps_panel function
    - Display window with file list
    - Show loaded maps section
    - Add actions section
    - _Requirements: 1.1, 1.5_

  - [ ] 10.2 Implement LDtk files section
    - Display available .ldtk files
    - Show load status indicators
    - Handle file selection and loading
    - _Requirements: 1.1, 1.2_

  - [ ] 10.3 Implement loaded maps section
    - Display Grid entity and layers
    - Show layer visibility toggles
    - Display layer information (size, name)
    - _Requirements: 1.5, 3.1_

  - [ ] 10.4 Implement actions section
    - Add Reload Map button
    - Add Regenerate Colliders button
    - Add Clean Up Colliders buttons
    - _Requirements: 1.3, 2.3_

  - [ ] 10.5 Implement statistics section
    - Display entity counts
    - Show tilemap and collider counts
    - _Requirements: 8.1, 8.2, 8.3_

- [ ] 11. Implement Layer Properties Panel
  - [ ] 11.1 Create LayerPropertiesPanel struct and basic rendering
    - Create panel structure with selected_layer field
    - Implement render method with window
    - Handle no selection state
    - _Requirements: 4.1_

  - [ ]* 11.2 Write property test for layer properties synchronization
    - **Property 12: Layer Properties Synchronization**
    - **Validates: Requirements 4.1**
    - Test that selecting a layer displays correct current values

  - [ ] 11.3 Implement transform editing section
    - Add drag values for position, rotation, scale
    - Implement reset buttons
    - Update transform component on change
    - _Requirements: 4.2, 4.5_

  - [ ]* 11.4 Write property test for transform updates
    - **Property 13: Transform Updates Apply Immediately**
    - **Validates: Requirements 4.2**
    - Test that modifying properties updates the component

  - [ ]* 11.5 Write property test for reset restores defaults
    - **Property 15: Reset Restores Defaults**
    - **Validates: Requirements 4.5**
    - Test that reset button restores default values

  - [ ] 11.6 Implement rendering section
    - Add visibility checkbox
    - Add Z-Order drag value
    - Add opacity slider
    - Add color tint picker
    - _Requirements: 3.1, 4.3, 4.4_

  - [ ]* 11.7 Write property test for Z-Order affects rendering
    - **Property 14: Z-Order Affects Rendering**
    - **Validates: Requirements 4.3**
    - Test that higher Z-Order renders on top

  - [ ] 11.8 Implement tilemap info section
    - Display tilemap size and tileset
    - Show tile count and memory usage
    - _Requirements: 8.3_

  - [ ] 11.9 Implement advanced section
    - Display entity ID and parent info
    - Show children count and components list
    - _Requirements: 7.5_

- [ ] 12. Implement Layer Ordering Panel
  - [ ] 12.1 Create LayerOrderingPanel struct and basic rendering
    - Create panel structure with drag state
    - Implement map selection dropdown
    - Display layer list
    - _Requirements: 5.1, 5.2_

  - [ ] 12.2 Implement drag and drop functionality
    - Track drag start and current position
    - Display visual feedback during drag
    - Calculate drop index from drag position
    - _Requirements: 5.1, 5.2_

  - [ ] 12.3 Implement reorder_layers method
    - Move layer in list
    - Update all Z-Order values
    - Update tilemap components
    - _Requirements: 5.2, 5.3_

  - [ ]* 12.4 Write property test for layer reordering
    - **Property 16: Layer Reordering Updates Z-Orders**
    - **Validates: Requirements 5.2**
    - Test that moving a layer updates all Z-Orders

  - [ ]* 12.5 Write property test for Z-Order monotonic increase
    - **Property 17: Z-Order Monotonic Increase**
    - **Validates: Requirements 5.3**
    - Test that Z-Orders are monotonically increasing after reorder

  - [ ] 12.6 Implement move up/down buttons
    - Increment Z-Order for move up
    - Decrement Z-Order for move down (min -100)
    - _Requirements: 5.4, 5.5_

  - [ ]* 12.7 Write property test for move up increments
    - **Property 18: Move Up Increments Z-Order**
    - **Validates: Requirements 5.4**
    - Test that move up increments by exactly 1

  - [ ]* 12.8 Write property test for move down with bounds
    - **Property 19: Move Down Decrements With Bounds**
    - **Validates: Requirements 5.5**
    - Test that move down decrements by 1 but never below -100

  - [ ] 12.9 Add visibility and lock toggles
    - Implement visibility toggle button
    - Implement lock/unlock button
    - _Requirements: 3.1_

- [ ] 13. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Implement hot-reload system
  - [ ] 14.1 Set up file watching with notify crate
    - Create file watcher for .ldtk files
    - Handle Create, Modify, Remove events
    - Debounce rapid file changes
    - _Requirements: 6.1, 6.2_

  - [ ] 14.2 Implement hot-reload logic
    - Detect file changes
    - Call reload_map on MapManager
    - Preserve layer states
    - _Requirements: 6.2, 6.3, 6.4_

  - [ ]* 14.3 Write property test for hot-reload regenerates entities
    - **Property 20: Hot-Reload Regenerates Entities**
    - **Validates: Requirements 6.2**
    - Test that hot-reload regenerates all entities from new data

  - [ ]* 14.4 Write property test for hot-reload regenerates colliders
    - **Property 21: Hot-Reload Regenerates Colliders**
    - **Validates: Requirements 6.4**
    - Test that colliders are regenerated during hot-reload

  - [ ] 14.3 Implement error recovery for hot-reload
    - Catch file corruption errors
    - Preserve last valid state on error
    - Display error message to user
    - _Requirements: 6.5_

  - [ ]* 14.6 Write property test for hot-reload error recovery
    - **Property 22: Hot-Reload Error Recovery**
    - **Validates: Requirements 6.5**
    - Test that corrupted files preserve last valid state

- [ ] 15. Implement hierarchy filtering
  - [ ] 15.1 Update hierarchy rendering to filter map entities
    - Use MapManager::is_map_entity to filter
    - Show Grid and layers
    - Hide collider entities
    - _Requirements: 7.4_

  - [ ]* 15.2 Write property test for hierarchy filtering
    - **Property 23: Hierarchy Filtering**
    - **Validates: Requirements 7.4**
    - Test that hierarchy shows Grid/layers but hides colliders

- [ ] 16. Implement performance monitoring
  - [ ] 16.1 Create PerformancePanel struct
    - Track draw calls, triangles, vertices
    - Monitor memory usage
    - Display real-time metrics
    - _Requirements: 8.1, 8.2, 8.3_

  - [ ]* 16.2 Write property test for performance metrics display
    - **Property 24: Performance Metrics Display**
    - **Validates: Requirements 8.1, 8.2**
    - Test that metrics are non-negative and displayed

  - [ ]* 16.3 Write property test for memory usage increases
    - **Property 25: Memory Usage Increases With Maps**
    - **Validates: Requirements 8.3**
    - Test that memory usage increases with more loaded maps

  - [ ] 16.4 Implement warning thresholds
    - Define thresholds for draw calls, memory, etc.
    - Display visual indicators when exceeded
    - _Requirements: 8.5_

  - [ ]* 16.5 Write property test for warning indicators
    - **Property 26: Performance Warning Indicators**
    - **Validates: Requirements 8.5**
    - Test that indicators appear when thresholds are exceeded

- [ ] 17. Implement collider configuration UI
  - [ ] 17.1 Create ColliderSettingsPanel struct
    - Display current configuration
    - Show collider type selection
    - Add collision value input
    - _Requirements: 9.1_

  - [ ]* 17.2 Write property test for collider settings display
    - **Property 27: Collider Settings Display**
    - **Validates: Requirements 9.1**
    - Test that opening settings displays current configuration

  - [ ] 17.3 Implement collider type selection
    - Add radio buttons for Composite/Individual/Polygon
    - Update configuration on change
    - _Requirements: 9.2, 9.3_

  - [ ]* 17.4 Write property test for composite type merges tiles
    - **Property 28: Composite Type Merges Tiles**
    - **Validates: Requirements 9.2**
    - Test that Composite type produces fewer colliders than tiles

  - [ ]* 17.5 Write property test for individual type one per tile
    - **Property 29: Individual Type One Per Tile**
    - **Validates: Requirements 9.3**
    - Test that Individual type produces one collider per tile

  - [ ] 17.6 Implement auto-regenerate toggle
    - Add checkbox for auto-regenerate on reload
    - Store setting in configuration
    - _Requirements: 9.5_

  - [ ]* 17.7 Write property test for auto-regenerate on reload
    - **Property 31: Auto-Regenerate On Reload**
    - **Validates: Requirements 9.5**
    - Test that enabling auto-regenerate regenerates on reload

- [ ] 18. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 19. Implement error handling and logging
  - [ ] 19.1 Create TilemapError enum
    - Define error types (FileNotFound, InvalidFormat, etc.)
    - Implement display_message method
    - Implement log_error method
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [ ] 19.2 Add error handling to load_map
    - Catch file not found errors
    - Catch invalid format errors
    - Display error messages in UI
    - _Requirements: 11.1, 11.2_

  - [ ]* 19.3 Write property test for invalid file error handling
    - **Property 32: Invalid File Error Handling**
    - **Validates: Requirements 11.1**
    - Test that invalid files display error without crashing

  - [ ]* 19.4 Write property test for missing file error handling
    - **Property 33: Missing File Error Handling**
    - **Validates: Requirements 11.2**
    - Test that missing files display "file not found" error

  - [ ] 19.5 Add error handling to regenerate_colliders
    - Catch generation failures
    - Maintain previous collider state on error
    - Display error message
    - _Requirements: 11.3_

  - [ ]* 19.6 Write property test for collider generation error recovery
    - **Property 34: Collider Generation Error Recovery**
    - **Validates: Requirements 11.3**
    - Test that generation failures maintain previous state

  - [ ] 19.7 Implement comprehensive error logging
    - Log all errors to console with context
    - Include stack traces for debugging
    - _Requirements: 11.5_

  - [ ]* 19.8 Write property test for error logging
    - **Property 35: Error Logging**
    - **Validates: Requirements 11.5**
    - Test that errors are logged to console

- [ ] 20. Implement data persistence
  - [ ] 20.1 Implement scene file serialization
    - Serialize layer visibility states
    - Serialize layer Z-Orders
    - Serialize layer transforms
    - _Requirements: 12.1, 12.2, 12.3_

  - [ ]* 20.2 Write property test for visibility persistence
    - **Property 36: Visibility Persistence**
    - **Validates: Requirements 12.1, 12.4**
    - Test that visibility is saved and restored

  - [ ]* 20.3 Write property test for Z-Order persistence
    - **Property 37: Z-Order Persistence**
    - **Validates: Requirements 12.2, 12.4**
    - Test that Z-Order is saved and restored

  - [ ]* 20.4 Write property test for transform persistence
    - **Property 38: Transform Persistence**
    - **Validates: Requirements 12.3, 12.4**
    - Test that transforms are saved and restored

  - [ ] 20.5 Implement project settings serialization
    - Create tilemap.json settings file
    - Save collider configuration
    - Load settings on startup
    - _Requirements: 12.5_

  - [ ]* 20.6 Write property test for collider configuration persistence
    - **Property 39: Collider Configuration Persistence**
    - **Validates: Requirements 12.5**
    - Test that configuration is saved to project settings

- [ ] 21. Integration and polish
  - [ ] 21.1 Integrate all panels into dock system
    - Add Maps Panel to dock layout
    - Add Layer Properties Panel to dock layout
    - Add Layer Ordering Panel to dock layout
    - Add Performance Panel to dock layout
    - _Requirements: All UI requirements_

  - [ ] 21.2 Add keyboard shortcuts
    - Ctrl+R for reload
    - Ctrl+G for regenerate colliders
    - Ctrl+H for toggle visibility
    - _Requirements: 10.1_

  - [ ] 21.3 Implement tooltips and help text
    - Add tooltips to all buttons
    - Add help text for complex features
    - _Requirements: 10.1_

  - [ ] 21.4 Performance optimization pass
    - Profile map loading
    - Profile collider generation
    - Optimize hot paths
    - _Requirements: 10.2, 10.3, 10.4, 10.5_

  - [ ] 21.5 Documentation and examples
    - Write user guide for tilemap management
    - Create example project with sample maps
    - Document API for extending the system
    - _Requirements: All_

- [ ] 22. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

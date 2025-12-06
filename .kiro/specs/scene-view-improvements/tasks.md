# Implementation Plan

- [x] 1. Enhance camera system with smooth damping and inertia





  - Add CameraSettings struct with sensitivity and damping parameters
  - Add CameraVelocity struct to track movement momentum
  - Implement exponential damping for smooth deceleration
  - Implement inertia system for weighted camera feel
  - Add target values for smooth interpolation (target_position, target_rotation, target_zoom)
  - _Requirements: 2.1, 2.2, 2.5, 5.1, 5.2, 5.3, 5.5_

- [x] 1.1 Write property test for damped pan movement


  - **Property 1: Damped pan movement is smooth**
  - **Validates: Requirements 2.1, 5.1**


- [x] 1.2 Write property test for orbit distance maintenance

  - **Property 2: Orbit maintains constant distance**
  - **Validates: Requirements 2.2, 5.2**

- [x] 1.3 Write property test for velocity decay


  - **Property 4: Velocity decays exponentially**
  - **Validates: Requirements 2.5, 5.5**

- [x] 1.4 Write property test for inertia momentum


  - **Property 6: Inertia maintains momentum**
  - **Validates: Requirements 5.1, 5.3**

- [ ] 2. Implement cursor-based zoom with world-space tracking
  - Enhance zoom() method to track cursor world position
  - Calculate world position under cursor before zoom
  - Apply zoom transformation
  - Adjust camera position to keep cursor point stationary
  - Add smooth zoom interpolation
  - _Requirements: 2.3, 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 2.1 Write property test for zoom cursor convergence
  - **Property 3: Zoom converges to cursor point**
  - **Validates: Requirements 2.3, 8.1, 8.2, 8.3**

- [ ] 3. Add configurable sensitivity settings
  - Create CameraSettings struct with pan/rotation/zoom sensitivity
  - Implement sensitivity scaling in all camera operations
  - Add methods to load/save settings from JSON
  - Add reset_settings_to_default() method
  - Store settings in .kiro/settings/camera_settings.json
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 3.1 Write property test for sensitivity scaling
  - **Property 5: Sensitivity scales linearly**
  - **Validates: Requirements 3.1, 3.2, 3.3**

- [ ] 3.2 Write unit tests for settings persistence
  - Test save/load camera settings
  - Test default value restoration
  - Test invalid value handling
  - _Requirements: 3.4, 3.5_

- [ ] 4. Implement infinite grid system
  - Create InfiniteGrid struct with enhanced configuration
  - Add multi-level grid support (minor, major, axis lines)
  - Implement adaptive grid level calculation based on zoom
  - Add smooth fade-in/fade-out for grid levels
  - Implement distance-based alpha fading
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.3, 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 4.1 Write property test for grid perspective convergence
  - **Property 7: Grid lines converge with perspective**
  - **Validates: Requirements 1.2, 7.1, 7.2**

- [ ] 4.2 Write property test for monotonic fade
  - **Property 8: Grid fade is monotonic with distance**
  - **Validates: Requirements 1.3**

- [ ] 4.3 Write property test for smooth level transitions
  - **Property 9: Grid level transitions maintain constant alpha**
  - **Validates: Requirements 6.3, 6.5**

- [ ] 4.4 Write property test for visual density maintenance
  - **Property 10: Grid spacing maintains visual density**
  - **Validates: Requirements 6.1, 6.2, 6.4**

- [ ] 4.5 Write property test for axis line visibility
  - **Property 11: Axis lines have full opacity at origin**
  - **Validates: Requirements 4.3**

- [ ] 4.6 Write property test for grid horizon extension
  - **Property 12: Grid extends to horizon**
  - **Validates: Requirements 1.1, 1.4, 1.5**

- [ ] 4.7 Write property test for grid orientation
  - **Property 13: Grid orientation matches camera rotation**
  - **Validates: Requirements 7.4**

- [ ] 5. Implement enhanced grid rendering with proper perspective
  - Implement generate_geometry() method for grid line generation
  - Add proper 3D perspective projection for grid points
  - Extend grid lines far into distance (e.g., 1000+ units)
  - Implement smooth alpha blending for multiple grid levels
  - Add axis line highlighting (red X, blue Z)
  - Ensure grid lines converge at vanishing points
  - _Requirements: 1.1, 1.2, 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 5.1 Write unit tests for grid projection
  - Test perspective projection calculations
  - Test vanishing point convergence
  - Test grid line generation
  - _Requirements: 7.1, 7.2, 7.5_

- [ ] 6. Implement grid caching for performance
  - Add GridGeometry struct to cache generated lines
  - Add CameraState comparison for cache invalidation
  - Implement needs_regeneration() method with threshold checking
  - Cache grid geometry when camera is static
  - Invalidate cache only when camera moves significantly
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 6.1 Write property test for grid caching
  - **Property 14: Grid caching reduces regeneration**
  - **Validates: Requirements 10.2**

- [ ] 6.2 Write property test for line batching efficiency
  - **Property 15: Line batching is efficient**
  - **Validates: Requirements 10.1**

- [ ] 7. Implement line batching for efficient rendering
  - Collect all grid lines into single batch
  - Group lines by color and width for efficient rendering
  - Submit all lines in minimal draw calls
  - Implement spatial culling to skip off-screen lines
  - _Requirements: 10.1, 10.4_

- [ ] 7.1 Write unit tests for line batching
  - Test line grouping by properties
  - Test spatial culling logic
  - Test draw call minimization
  - _Requirements: 10.1_

- [ ] 8. Add camera state display UI
  - Create CameraStateDisplay struct
  - Display camera distance from origin
  - Display camera rotation angles (yaw and pitch)
  - Display current grid unit size
  - Display FPS counter
  - Add tooltips for scene gizmo axes
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [ ] 8.1 Write unit tests for state display
  - Test distance calculation display
  - Test angle display formatting
  - Test grid size display
  - _Requirements: 9.1, 9.2, 9.4_

- [ ] 9. Integrate enhanced camera and grid into scene view
  - Update scene_view.rs to use new camera system
  - Replace old grid rendering with infinite grid
  - Wire up camera state display
  - Ensure smooth transitions between 2D and 3D modes
  - Test all camera controls work correctly
  - _Requirements: All_

- [ ] 10. Add error handling and validation
  - Add sensitivity value clamping [0.01, 10.0]
  - Add NaN/Inf checks in all calculations
  - Add cursor position validation
  - Add grid spacing bounds checking
  - Add projection error handling for points behind camera
  - Implement graceful degradation for extreme zoom levels
  - _Requirements: All_

- [ ] 10.1 Write unit tests for error handling
  - Test invalid sensitivity values
  - Test NaN/Inf handling
  - Test extreme zoom levels
  - Test projection edge cases
  - _Requirements: All_

- [ ] 11. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 12. Performance optimization and polish
  - Optimize grid line generation algorithm
  - Implement aggressive culling for distant lines
  - Fine-tune damping and sensitivity default values
  - Adjust grid colors for professional appearance
  - Test with various camera positions and angles
  - _Requirements: 4.1, 4.2, 10.1, 10.4_

- [ ] 12.1 Write performance benchmarks
  - Benchmark grid rendering time
  - Benchmark cache hit rate
  - Benchmark frame time consistency
  - _Requirements: 10.4_

- [ ] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Implement snapping system
  - Create SnapSettings struct with position/rotation/scale increments
  - Implement snap_position(), snap_rotation(), snap_scale() methods
  - Add Ctrl key detection for snap activation
  - Integrate snapping into transform gizmo interactions
  - Add visual snap indicators (grid highlights)
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [ ] 14.1 Write property test for grid snapping consistency
  - **Property 16: Grid snapping is consistent**
  - **Validates: Requirements 11.1**

- [ ] 14.2 Write property test for snap increments
  - **Property 17: Snap increments are configurable**
  - **Validates: Requirements 11.4**

- [ ]* 14.3 Write unit tests for snapping
  - Test position snapping with various grid sizes
  - Test rotation snapping with various angles
  - Test scale snapping with various increments
  - Test relative vs absolute snap modes
  - _Requirements: 11.1, 11.2, 11.3_

- [ ] 15. Implement multi-selection system
  - Create Selection struct with entity list management
  - Implement box selection (drag to select)
  - Add Ctrl+Click to add/remove from selection
  - Add Shift+Click to add to selection
  - Implement Ctrl+A to select all
  - Add selection outline rendering
  - _Requirements: 12.1, 12.2, 12.3, 12.5_

- [ ] 15.1 Write property test for box selection
  - **Property 18: Box selection is inclusive**
  - **Validates: Requirements 12.1**

- [ ] 15.2 Write property test for multi-selection order
  - **Property 19: Multi-selection preserves order**
  - **Validates: Requirements 12.2, 12.3**

- [ ] 15.3 Write property test for select all
  - **Property 20: Select all includes all entities**
  - **Validates: Requirements 12.5**

- [ ]* 15.4 Write unit tests for selection
  - Test single selection
  - Test multi-selection operations
  - Test selection clearing
  - Test selection bounds calculation
  - _Requirements: 12.1, 12.2, 12.3, 12.5_

- [ ] 16. Implement enhanced gizmo system
  - Create EnhancedGizmo struct with handle types
  - Add planar movement handles (XY, XZ, YZ planes)
  - Implement screen-constant gizmo sizing
  - Add hover highlighting (yellow on hover)
  - Implement uniform scale handle (center cube)
  - Add proper 3D gizmo rendering
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

- [ ] 16.1 Write property test for gizmo screen-constant size
  - **Property 21: Gizmo size is screen-constant**
  - **Validates: Requirements 13.3**

- [ ] 16.2 Write property test for planar movement
  - **Property 22: Planar handles move in plane**
  - **Validates: Requirements 13.1**

- [ ]* 16.3 Write unit tests for gizmo rendering
  - Test gizmo size calculation at various zoom levels
  - Test hover detection for handles
  - Test planar handle rendering
  - _Requirements: 13.1, 13.2, 13.3_

- [ ] 17. Implement multi-selection gizmo
  - Calculate center point of selected entities
  - Render gizmo at multi-selection center
  - Apply transforms to all selected entities
  - Maintain relative positions during transform
  - _Requirements: 12.4_

- [ ]* 17.1 Write unit tests for multi-selection transforms
  - Test moving multiple entities
  - Test rotating multiple entities
  - Test scaling multiple entities
  - Test relative position preservation
  - _Requirements: 12.4_

- [ ] 18. Implement 2.5D support enhancements
  - Create Scene25DSettings struct
  - Implement sprite Z-depth sorting
  - Add Z-depth visualization for selected entities
  - Implement billboard sprite mode
  - Add isometric grid rendering for 2.5D mode
  - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_

- [ ] 18.1 Write property test for Z-depth sorting
  - **Property 23: Z-depth sorting is correct**
  - **Validates: Requirements 14.2**

- [ ] 18.2 Write property test for orthographic projection
  - **Property 24: Orthographic projection preserves parallels**
  - **Validates: Requirements 14.1**

- [ ]* 18.3 Write unit tests for 2.5D rendering
  - Test sprite sorting by Z-position
  - Test billboard sprite orientation
  - Test isometric grid rendering
  - _Requirements: 14.1, 14.2, 14.4, 14.5_

- [ ] 19. Implement enhanced scene view toolbar
  - Add shading mode dropdown (Wireframe, Shaded, Textured)
  - Add gizmos visibility dropdown
  - Add scene view options menu
  - Implement shading mode rendering
  - Add toolbar icons and styling
  - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_

- [ ]* 19.1 Write unit tests for toolbar
  - Test shading mode switching
  - Test gizmo visibility toggles
  - Test toolbar state persistence
  - _Requirements: 15.1, 15.2, 15.3_

- [ ] 20. Implement viewport statistics overlay
  - Create ViewportStats struct
  - Track FPS and frame time
  - Count entities and visible entities
  - Track draw calls (estimate)
  - Add toggle for detailed/minimal view
  - Render stats overlay in corner
  - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_

- [ ]* 20.1 Write unit tests for stats tracking
  - Test FPS calculation
  - Test entity counting
  - Test stats overlay rendering
  - _Requirements: 16.1, 16.2, 16.3_

- [ ] 21. Implement camera speed modifiers
  - Add Shift key detection for 3x speed
  - Add Ctrl key detection for 0.3x speed
  - Apply speed modifiers to all camera movements
  - Implement smooth speed transitions
  - Combine modifiers with sensitivity settings
  - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5_

- [ ] 21.1 Write property test for speed modifiers
  - **Property 25: Speed modifiers multiply correctly**
  - **Validates: Requirements 17.1, 17.2, 17.3**

- [ ]* 21.2 Write unit tests for speed modifiers
  - Test Shift modifier (3x speed)
  - Test Ctrl modifier (0.3x speed)
  - Test smooth speed transitions
  - Test modifier combination with sensitivity
  - _Requirements: 17.1, 17.2, 17.3, 17.4_

- [ ] 22. Implement flythrough camera mode
  - Create FlythroughMode struct
  - Detect right-click to enter flythrough mode
  - Implement WASD movement in view direction
  - Implement mouse look (rotate view)
  - Add Q/E for up/down movement
  - Apply speed modifiers in flythrough mode
  - Exit flythrough on right-click release
  - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5, 18.6, 18.7_

- [ ] 22.1 Write property test for flythrough movement
  - **Property 26: Flythrough movement is view-relative**
  - **Validates: Requirements 18.2, 18.3, 18.4, 18.5**

- [ ]* 22.2 Write unit tests for flythrough mode
  - Test WASD movement calculations
  - Test mouse look rotation
  - Test flythrough activation/deactivation
  - Test speed modifiers in flythrough
  - _Requirements: 18.1, 18.2, 18.3, 18.6, 18.7_

- [ ] 23. Implement frame all functionality
  - Add A key detection for frame all
  - Calculate bounds of all entities in scene
  - Calculate optimal camera position and zoom
  - Animate camera to frame all entities
  - Handle empty scene (frame origin)
  - Handle very spread out entities
  - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5_

- [ ] 23.1 Write property test for frame all
  - **Property 27: Frame all includes all entities**
  - **Validates: Requirements 19.1, 19.2**

- [ ]* 23.2 Write unit tests for frame all
  - Test bounds calculation for multiple entities
  - Test camera positioning for various bounds
  - Test empty scene handling
  - Test animation to target position
  - _Requirements: 19.1, 19.2, 19.3, 19.4_

- [ ] 24. Implement enhanced scene gizmo
  - Create EnhancedSceneGizmo struct
  - Make axis labels clickable
  - Add center cube for perspective toggle
  - Implement smooth camera transitions (0.3s)
  - Add hover tooltips for axes
  - Highlight axes on hover
  - Make axis arrows cone-shaped (not circles)
  - _Requirements: 20.1, 20.2, 20.3, 20.4, 20.5_

- [ ] 24.1 Write property test for axis alignment
  - **Property 28: Axis click aligns view**
  - **Validates: Requirements 20.1**

- [ ]* 24.2 Write unit tests for scene gizmo
  - Test axis click detection
  - Test camera view transitions
  - Test perspective/orthographic toggle
  - Test hover detection and tooltips
  - _Requirements: 20.1, 20.2, 20.3, 20.5_

- [ ] 25. Checkpoint - Ensure all new features work together
  - Test snapping with multi-selection
  - Test gizmos with 2.5D mode
  - Test flythrough with speed modifiers
  - Test frame all with various entity configurations
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 26. Polish and integration
  - Fine-tune all animation timings
  - Adjust colors for professional appearance
  - Optimize rendering performance
  - Add keyboard shortcut hints to UI
  - Test all features in combination
  - _Requirements: All_

- [ ]* 26.1 Write integration tests
  - Test complete workflows (select, move, snap)
  - Test mode switching (2D, 3D, 2.5D)
  - Test camera navigation workflows
  - Test multi-selection workflows
  - _Requirements: All_

- [ ] 27. Final checkpoint - Complete Unity-like scene editor
  - Ensure all tests pass, ask the user if questions arise.

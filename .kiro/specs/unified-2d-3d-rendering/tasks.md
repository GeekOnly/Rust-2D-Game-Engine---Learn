# Implementation Plan: Unified 2D/3D Rendering System

- [x] 1. Set up core unified rendering infrastructure





  - Create ViewMode enum and UnifiedCamera component structures
  - Extend existing Camera component with unified 2D/3D capabilities
  - Set up WGPU pipeline modifications for unified rendering
  - _Requirements: 1.1, 1.2, 1.3, 5.1_

- [ ]* 1.1 Write property test for mode toggle projection switching
  - **Property 1: Mode toggle switches projection correctly**
  - **Validates: Requirements 1.1, 1.2, 1.3, 1.4**

- [ ] 2. Implement perfect pixel rendering system
  - [ ] 2.1 Create PerfectPixelSettings component and pixel snapping algorithms
    - Implement pixel boundary snapping calculations
    - Create pixel-perfect transform utilities
    - Add pixels-per-unit configuration system
    - _Requirements: 4.1, 4.2, 2.2, 3.2_

  - [ ]* 2.2 Write property test for perfect pixel rendering in 2D mode
    - **Property 3: Perfect pixel rendering in 2D mode**
    - **Validates: Requirements 2.2, 3.2, 4.1, 4.2**

  - [ ] 2.3 Implement viewport consistency for perfect pixel rendering
    - Add viewport size change handling
    - Implement consistent pixel ratio maintenance
    - Create reference resolution scaling system
    - _Requirements: 4.4_

  - [ ]* 2.4 Write property test for viewport consistency
    - **Property 8: Viewport consistency**
    - **Validates: Requirements 4.4**

- [ ] 3. Enhance sprite rendering for unified 2D/3D support
  - [ ] 3.1 Extend sprite component with 2D/3D rendering options
    - Add billboard and world-space quad rendering modes
    - Implement depth sorting integration
    - Create perfect pixel sprite positioning
    - _Requirements: 2.1, 2.3, 2.4_

  - [ ]* 3.2 Write property test for unified world space positioning
    - **Property 2: Unified world space positioning**
    - **Validates: Requirements 1.5, 2.1, 3.1**

  - [ ]* 3.3 Write property test for 3D mode rendering behavior
    - **Property 4: 3D mode rendering behavior**
    - **Validates: Requirements 2.3, 3.3**

  - [ ] 3.4 Implement perfect pixel scaling for sprites
    - Add integer scale factor detection
    - Implement nearest-neighbor filtering for pixel-perfect scaling
    - Create crisp edge preservation system
    - _Requirements: 2.5, 4.3_

  - [ ]* 3.5 Write property test for perfect pixel scaling preservation
    - **Property 6: Perfect pixel scaling preservation**
    - **Validates: Requirements 2.5, 4.3**

- [ ] 4. Enhance tilemap rendering for unified 2D/3D support
  - [ ] 4.1 Extend tilemap component with world-space 3D rendering
    - Implement tilemap as world-space geometry in 3D mode
    - Add multi-layer depth sorting for tilemaps
    - Create grid coordinate to world space positioning
    - _Requirements: 3.1, 3.3, 3.4_

  - [ ] 4.2 Implement animated tile support with perfect pixel preservation
    - Add texture coordinate animation system
    - Ensure animations maintain pixel alignment
    - Create frame-based animation timing
    - _Requirements: 3.5_

  - [ ]* 4.3 Write property test for animation pixel preservation
    - **Property 7: Animation pixel preservation**
    - **Validates: Requirements 3.5**

- [ ] 5. Implement depth sorting and mixed content rendering
  - [ ] 5.1 Create unified depth sorting system
    - Implement depth buffer integration for sprites and 3D objects
    - Add manual sort order support for sprites
    - Create depth-based rendering order calculation
    - _Requirements: 2.4, 3.4_

  - [ ]* 5.2 Write property test for depth sorting consistency
    - **Property 5: Depth sorting consistency**
    - **Validates: Requirements 2.4, 3.4**

  - [ ] 5.3 Implement mixed 2D/3D content rendering
    - Create unified rendering pipeline for mixed content
    - Ensure proper depth testing between 2D and 3D objects
    - Add support for rendering both types in same frame
    - _Requirements: 4.5_

  - [ ]* 5.4 Write property test for mixed content rendering
    - **Property 9: Mixed content rendering**
    - **Validates: Requirements 4.5**

- [ ] 6. Optimize WGPU rendering pipeline
  - [ ] 6.1 Implement shader selection system
    - Create 2D-optimized shaders for sprites and tilemaps
    - Ensure 3D shaders include full lighting and materials
    - Add automatic shader selection based on content type
    - _Requirements: 5.2, 5.3_

  - [ ]* 6.2 Write property test for appropriate shader usage
    - **Property 10: Appropriate shader usage**
    - **Validates: Requirements 5.2, 5.3**

  - [ ] 6.3 Implement draw call batching optimization
    - Group similar objects to minimize WGPU state changes
    - Create batching system for sprites and tilemaps
    - Add performance monitoring for batch efficiency
    - _Requirements: 5.4_

  - [ ]* 6.4 Write property test for draw call batching
    - **Property 11: Draw call batching**
    - **Validates: Requirements 5.4**

  - [ ] 6.5 Unify texture management system
    - Ensure sprites and tilemaps use same WGPU texture management
    - Create consistent texture update pipeline
    - Add texture streaming and memory management
    - _Requirements: 5.5_

  - [ ]* 6.6 Write property test for texture management consistency
    - **Property 12: Texture management consistency**
    - **Validates: Requirements 5.5**

- [ ] 7. Checkpoint - Ensure all core rendering tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement editor view mode switching
  - [ ] 8.1 Create 2D/3D mode toggle UI in Scene View
    - Add toggle button to Scene View toolbar
    - Implement mode switching logic
    - Create visual indicators for current mode
    - _Requirements: 1.1_

  - [ ] 8.2 Implement smooth camera transitions between modes
    - Add transition animation system
    - Preserve camera context during mode switches
    - Create smooth interpolation between projection modes
    - _Requirements: 7.3_

  - [ ]* 8.3 Write property test for smooth mode transitions
    - **Property 18: Smooth mode transitions**
    - **Validates: Requirements 7.3**

- [ ] 9. Implement mode-specific navigation controls
  - [ ] 9.1 Create 2D navigation controls
    - Implement optimized pan and zoom for 2D mode
    - Add 2D-specific keyboard shortcuts
    - Create 2D viewport framing functionality
    - _Requirements: 7.1, 7.4_

  - [ ]* 9.2 Write property test for 2D navigation controls
    - **Property 16: 2D navigation controls**
    - **Validates: Requirements 7.1**

  - [ ] 9.3 Create 3D navigation controls
    - Implement orbit, pan, and zoom for 3D mode
    - Add 3D-specific camera controls
    - Create 3D viewport framing functionality
    - _Requirements: 7.2, 7.4_

  - [ ]* 9.4 Write property test for 3D navigation controls
    - **Property 17: 3D navigation controls**
    - **Validates: Requirements 7.2**

  - [ ]* 9.5 Write property test for object framing
    - **Property 19: Object framing**
    - **Validates: Requirements 7.4**

  - [ ] 9.6 Ensure keyboard shortcut consistency across modes
    - Implement consistent shortcut behavior in both modes
    - Add mode-aware shortcut handling
    - Create unified shortcut configuration system
    - _Requirements: 7.5_

  - [ ]* 9.7 Write property test for keyboard shortcut consistency
    - **Property 20: Keyboard shortcut consistency**
    - **Validates: Requirements 7.5**

- [ ] 10. Implement editor-runtime rendering consistency
  - [ ] 10.1 Unify Scene View and Game View rendering pipelines
    - Ensure both views use identical rendering code
    - Create shared rendering context between views
    - Add rendering consistency validation
    - _Requirements: 6.1, 6.3_

  - [ ]* 10.2 Write property test for view rendering consistency
    - **Property 13: View rendering consistency**
    - **Validates: Requirements 6.1, 6.3**

  - [ ] 10.3 Implement gizmo rendering isolation
    - Ensure gizmos don't affect game content rendering
    - Create separate rendering pass for editor gizmos
    - Add gizmo depth testing and blending
    - _Requirements: 6.2_

  - [ ]* 10.4 Write property test for gizmo isolation
    - **Property 14: Gizmo isolation**
    - **Validates: Requirements 6.2**

  - [ ] 10.5 Ensure mode transition visual consistency
    - Maintain visual quality during editor/play mode switches
    - Add consistency validation between modes
    - Create visual quality preservation system
    - _Requirements: 6.4_

  - [ ]* 10.6 Write property test for mode transition consistency
    - **Property 15: Mode transition consistency**
    - **Validates: Requirements 6.4**

- [ ] 11. Final integration and testing
  - [ ] 11.1 Integrate all systems with existing engine architecture
    - Connect unified rendering to existing ECS systems
    - Ensure compatibility with current project structure
    - Add migration path for existing 2D/3D content
    - _Requirements: All_

  - [ ] 11.2 Create comprehensive integration tests
    - Test complete 2D to 3D workflow
    - Verify perfect pixel rendering in real scenarios
    - Test performance with complex mixed scenes
    - _Requirements: All_

- [ ] 12. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
# Implementation Plan

- [x] 1. Enhance camera control system





  - Improve SceneCamera with better state management and smooth interpolation
  - Implement proper coordinate transformations for 2D and 3D modes
  - Add view and projection matrix calculations
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.2, 5.4_

- [x] 1.1 Write property test for pan camera position updates


  - **Property 1: Pan updates camera position**
  - **Validates: Requirements 1.1, 1.4, 5.4**

- [x] 1.2 Write property test for orbit distance maintenance


  - **Property 2: Orbit maintains pivot distance**
  - **Validates: Requirements 1.2, 5.3**

- [x] 1.3 Write property test for free-look rotation


  - **Property 3: Free-look rotation updates camera orientation**
  - **Validates: Requirements 1.3**

- [x] 1.4 Write property test for zoom toward cursor


  - **Property 4: Zoom scales view toward cursor**
  - **Validates: Requirements 1.5**

- [x] 1.5 Write property test for smooth zoom interpolation


  - **Property 5: Zoom interpolation is smooth**
  - **Validates: Requirements 5.2**

- [x] 1.6 Write property test for entity focus framing


  - **Property 6: Focus frames entity appropriately**
  - **Validates: Requirements 1.6**

- [x] 2. Implement enhanced grid system





  - Add adaptive grid spacing based on zoom level
  - Implement perspective-correct 3D grid rendering
  - Add distance-based fading for 3D grid lines
  - Implement axis highlighting (red X, blue Z)
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 7.2, 7.3_

- [x] 2.1 Write property test for 2D grid orthogonality


  - **Property 7: 2D grid lines are orthogonal**
  - **Validates: Requirements 2.1**

- [x] 2.2 Write property test for 3D grid perspective correctness


  - **Property 8: 3D grid has correct perspective**
  - **Validates: Requirements 2.2**


- [x] 2.3 Write property test for grid distance fading

  - **Property 9: Grid fades with distance**
  - **Validates: Requirements 2.3**

- [x] 2.4 Write property test for adaptive grid density


  - **Property 10: Adaptive grid maintains visual density**
  - **Validates: Requirements 2.5, 7.1**


- [x] 2.5 Write property test for grid subdivision adaptation

  - **Property 11: Grid subdivisions adapt to zoom**
  - **Validates: Requirements 7.2, 7.3**

- [x] 3. Implement 2D/3D mode switching






  - Add mode switching logic with state preservation
  - Implement camera state save/restore for mode transitions
  - Add toolbar UI for 2D/3D toggle
  - Ensure smooth transitions between modes
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 3.1 Write property test for camera state preservation during mode switch


  - **Property 12: Mode switching preserves camera state**
  - **Validates: Requirements 3.3**

- [x] 3.2 Write property test for 3D orientation restoration


  - **Property 13: 3D mode restores or initializes orientation**
  - **Validates: Requirements 3.4**

- [ ] 4. Implement 3D rendering improvements
  - Add Point3D structure with rotation and projection methods
  - Implement perspective and isometric projection
  - Add back-face culling for mesh rendering
  - Implement depth-based face sorting (painter's algorithm)
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 4.1 Write property test for perspective depth scaling
  - **Property 14: Perspective projection scales with depth**
  - **Validates: Requirements 4.1**

- [ ] 4.2 Write property test for back-face culling
  - **Property 15: Back-face culling hides non-visible faces**
  - **Validates: Requirements 4.2**

- [ ] 4.3 Write property test for face depth sorting
  - **Property 16: Faces are depth-sorted**
  - **Validates: Requirements 4.3**

- [ ] 5. Implement scene gizmo
  - Create scene gizmo visual component (XYZ axes)
  - Add clickable axis indicators for camera snapping
  - Implement gizmo rotation to match camera orientation
  - Add projection mode toggle button near gizmo
  - _Requirements: 3.5, 4.5, 6.1, 6.2, 6.3, 6.5_

- [ ] 5.1 Write property test for gizmo orientation reflection
  - **Property 17: Gizmo reflects camera orientation**
  - **Validates: Requirements 6.2**

- [ ] 6. Implement depth sorting for entities
  - Add Z-position based sorting for entities in 3D mode
  - Implement separate sorting for transparent objects
  - Ensure selection outlines and gizmos render on top
  - Add proper render order for all scene elements
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 6.1 Write property test for entity depth ordering
  - **Property 18: Entities render in depth order**
  - **Validates: Requirements 8.1**

- [ ] 6.2 Write property test for transparent object sorting
  - **Property 19: Transparent objects are sorted correctly**
  - **Validates: Requirements 8.2**

- [ ] 7. Integrate all components in scene view
  - Wire up enhanced camera controls to scene view
  - Integrate new grid rendering
  - Connect mode switching to UI
  - Ensure all components work together seamlessly
  - _Requirements: All_

- [ ] 8. Add error handling and validation
  - Add input validation for camera operations
  - Implement bounds checking for zoom and rotation
  - Add error handling for invalid mesh data
  - Ensure graceful degradation for edge cases
  - _Requirements: All_

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Polish and optimize
  - Optimize grid rendering performance
  - Add caching for static camera states
  - Implement line batching for grid rendering
  - Fine-tune camera sensitivity and feel
  - _Requirements: 5.1_

- [ ] 10.1 Write unit tests for coordinate transformations
  - Test world-to-screen and screen-to-world conversions
  - Test rotation matrix calculations
  - Test projection matrix generation
  - _Requirements: 1.1, 1.5, 4.1_

- [ ] 10.2 Write unit tests for grid calculations
  - Test grid line generation
  - Test fade alpha calculations
  - Test adaptive spacing selection
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [ ] 11. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

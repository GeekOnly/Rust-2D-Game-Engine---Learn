# Implementation Plan

- [x] 1. Set up UI crate structure and core types
  - Create new `ui` crate with module structure
  - Define core types: Vec2, Vec4, Rect, Color
  - Set up dependencies (ecs, render, serde, glam, proptest)
  - Create public API in lib.rs with re-exports
  - _Requirements: All requirements - foundation_

- [x] 2. Implement RectTransform system
  - [x] 2.1 Create RectTransform component
    - Define RectTransform struct with anchor, pivot, position, size
    - Implement helper methods (anchored, stretched, get_size, set_size)
    - Implement contains_point for raycasting
    - Implement anchor clamping in setters
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [ ]* 2.2 Write property test for anchor clamping
    - **Property 4: RectTransform anchor clamping**
    - **Validates: Requirements 2.7**
  
  - [ ]* 2.3 Write property test for fixed anchor positioning
    - **Property 5: Fixed anchor positioning**
    - **Validates: Requirements 2.2**
  
  - [ ]* 2.4 Write property test for stretched anchor sizing
    - **Property 6: Stretched anchor sizing**
    - **Validates: Requirements 2.3, 2.4**
  
  - [ ]* 2.5 Write property test for pivot change invariant
    - **Property 7: Pivot change preserves visual position**
    - **Validates: Requirements 2.5**
  
  - [x] 2.6 Implement RectTransform calculation system
    - Create system to calculate world corners and rects
    - Handle parent-child transform calculations
    - Implement dirty flagging for efficient updates
    - _Requirements: 2.1, 2.6, 3.1_

- [x] 3. Implement Canvas system
  - [x] 3.1 Create Canvas and CanvasScaler components
    - Define Canvas struct with render mode and sort order
    - Define CanvasScaler with all scale modes
    - Implement scale factor calculation for each mode
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
  
  - [ ]* 3.2 Write property test for canvas initialization
    - **Property 1: Canvas initialization completeness**
    - **Validates: Requirements 1.1**
  
  - [ ]* 3.3 Write property test for resolution change
    - **Property 2: Resolution change preserves anchored layout**
    - **Validates: Requirements 1.5, 2.6, 7.5**
  
  - [ ]* 3.4 Write property test for canvas sort order
    - **Property 3: Canvas sort order determines render priority**
    - **Validates: Requirements 1.6**
  
  - [ ]* 3.5 Write property tests for canvas scaler modes
    - **Property 24: Canvas scaler maintains pixel size**
    - **Property 25: Canvas scaler proportional scaling**
    - **Property 26: Canvas scaler DPI consistency**
    - **Property 27: Canvas scaler aspect ratio matching**
    - **Property 28: Canvas scaler clamps scale factor**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.6**
  
  - [x] 3.6 Create Canvas management system
    - Implement canvas creation and initialization
    - Handle screen resolution changes
    - Update scale factors and mark canvases dirty
    - _Requirements: 1.1, 1.5, 7.5_

- [x] 4. Implement UI hierarchy system
  - [x] 4.1 Create UIElement base component
    - Define UIElement struct with raycast, color, alpha, interactable
    - Implement z-order and canvas caching
    - _Requirements: 3.1, 3.7, 6.1, 6.7_
  
  - [x] 4.2 Implement hierarchy propagation systems
    - Create system for canvas entity propagation
    - Create system for visibility propagation
    - Create system for destruction propagation
    - Handle sibling index ordering and render order
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_
  
  - [ ]* 4.3 Write property test for transform propagation
    - **Property 8: Hierarchical transform propagation**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
  
  - [ ]* 4.4 Write property test for visibility propagation
    - **Property 9: Hierarchical visibility propagation**
    - **Validates: Requirements 3.5**
  
  - [ ]* 4.5 Write property test for destruction propagation
    - **Property 10: Hierarchical destruction propagation**
    - **Validates: Requirements 3.6**
  
  - [ ]* 4.6 Write property test for sibling render order
    - **Property 11: Sibling index determines render order**
    - **Validates: Requirements 3.7**

- [x] 5. Define core UI component types
  - [x] 5.1 Create UIImage component type
    - Define UIImage struct with sprite, image type, 9-slice
    - Define fill methods (horizontal, vertical, radial)
    - Define preserve aspect ratio
    - _Requirements: 4.1, 4.5_
  
  - [x] 5.2 Create UIText component type
    - Define UIText struct with font, size, color, alignment
    - Define overflow modes (wrap, truncate, overflow)
    - Define rich text support
    - _Requirements: 4.2, 4.6_
  
  - [x] 5.3 Create UIButton component type
    - Define UIButton struct with states and transitions
    - Define color tint transition
    - Define sprite swap transition
    - Define button state changes
    - _Requirements: 4.3_
  
  - [x] 5.4 Create UIPanel component type
    - Define UIPanel struct with background and padding
    - Support 9-slice backgrounds
    - _Requirements: 4.4_
  
  - [x] 5.5 Create advanced component types
    - Define UISlider, UIToggle, UIDropdown, UIInputField, UIScrollView
    - _Requirements: 9.1-9.8, 10.1-10.8_

- [x] 6. Define layout system types
  - [x] 6.1 Create layout group component types
    - Define HorizontalLayoutGroup struct
    - Define VerticalLayoutGroup struct
    - Define GridLayoutGroup struct
    - Define Alignment, Corner, Axis enums
    - _Requirements: 5.1, 5.2, 5.3_

- [x] 7. Implement layout calculation system








  - [x] 7.1 Implement horizontal layout algorithm


    - Create system to arrange children horizontally
    - Handle padding, spacing, and alignment
    - Handle force expand and child control
    - _Requirements: 5.1, 5.4, 5.5, 5.6, 5.7_
  
  - [x] 7.2 Implement vertical layout algorithm


    - Create system to arrange children vertically
    - Handle padding, spacing, and alignment
    - Handle force expand and child control
    - _Requirements: 5.2, 5.4, 5.5, 5.6, 5.7_
  
  - [x] 7.3 Implement grid layout algorithm


    - Create system to arrange children in grid
    - Handle padding, spacing, cell size, and alignment
    - Handle constraints (flexible, fixed column/row count)
    - _Requirements: 5.3, 5.4, 5.5, 5.6, 5.7_
  
  - [ ]* 7.4 Write property test for layout spacing
    - **Property 15: Layout spacing consistency**
    - **Validates: Requirements 5.1, 5.2, 5.3**
  
  - [ ]* 7.5 Write property test for layout padding
    - **Property 16: Layout padding application**
    - **Validates: Requirements 5.4**
  
  - [ ]* 7.6 Write property test for layout alignment
    - **Property 17: Layout child alignment**
    - **Validates: Requirements 5.5**
  
  - [ ]* 7.7 Write property test for layout force expand
    - **Property 18: Layout force expand**
    - **Validates: Requirements 5.6**
  
  - [ ]* 7.8 Write property test for layout recalculation
    - **Property 19: Layout recalculation on size change**
    - **Validates: Requirements 5.7**

- [x] 8. Implement event system





  - [x] 8.1 Implement UI raycasting


    - Create raycast system to find elements at point
    - Handle raycast target filtering
    - Handle raycast blocking
    - Sort by Z-order for correct event delivery
    - _Requirements: 6.1, 6.6, 6.7_
  
  - [x] 8.2 Implement input event processing


    - Process mouse/touch input
    - Generate UI events (click, hover, drag)
    - Dispatch events to elements
    - Update button states based on events
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.8_
  
  - [ ]* 8.3 Write property test for raycast target inclusion
    - **Property 20: Raycast target inclusion**
    - **Validates: Requirements 6.1**
  
  - [ ]* 8.4 Write property test for event delivery to topmost
    - **Property 21: Event delivery to topmost element**
    - **Validates: Requirements 6.6**
  
  - [ ]* 8.5 Write property test for raycast blocking
    - **Property 22: Raycast blocking**
    - **Validates: Requirements 6.7**
  
  - [ ]* 8.6 Write property test for event callback invocation
    - **Property 23: Event callback invocation**
    - **Validates: Requirements 6.2, 6.3, 6.4, 6.5, 6.8**

- [x] 9. Implement animation system





  - [x] 9.1 Implement easing function calculations


    - Implement all easing functions (linear, quad, cubic, etc.)
    - Create easing function evaluator
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_
  
  - [x] 9.2 Implement animation update system


    - Update animation elapsed time
    - Calculate interpolated values using easing
    - Apply values to RectTransform and UIElement components
    - Handle animation completion and callbacks
    - Handle loop modes (once, loop, ping-pong)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.8_
  
  - [ ]* 9.3 Write property test for animation interpolation
    - **Property 29: Animation interpolation correctness**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7**
  
  - [ ]* 9.4 Write property test for animation completion callback
    - **Property 30: Animation completion callback**
    - **Validates: Requirements 8.8**

- [x] 10. Implement scroll view system





  - [x] 10.1 Implement viewport clipping system


    - Create clipping mask for viewport
    - Cull content outside viewport bounds
    - _Requirements: 9.1, 9.8_
  
  - [x] 10.2 Implement scroll view interaction


    - Implement drag scrolling
    - Implement scrollbar updates
    - Implement programmatic scrolling
    - Implement elastic spring-back
    - Implement inertia deceleration
    - _Requirements: 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_
  
  - [ ]* 10.3 Write property test for viewport clipping
    - **Property 31: Scroll view viewport clipping**
    - **Validates: Requirements 9.1, 9.8**
  
  - [ ]* 10.4 Write property test for drag scrolling
    - **Property 32: Scroll view drag scrolling**
    - **Validates: Requirements 9.3**
  
  - [ ]* 10.5 Write property test for scrollbar position
    - **Property 33: Scrollbar position reflects content**
    - **Validates: Requirements 9.4**
  
  - [ ]* 10.6 Write property test for programmatic scrolling
    - **Property 34: Programmatic scroll positioning**
    - **Validates: Requirements 9.5**
  
  - [ ]* 10.7 Write property test for elastic spring-back
    - **Property 35: Elastic scroll spring-back**
    - **Validates: Requirements 9.6**
  
  - [ ]* 10.8 Write property test for scroll inertia
    - **Property 36: Scroll inertia deceleration**
    - **Validates: Requirements 9.7**

- [x] 11. Implement advanced component systems





  - [x] 11.1 Implement slider interaction system


    - Implement value clamping
    - Implement handle positioning based on value
    - Handle drag interaction to update value
    - _Requirements: 10.1, 10.5**
  
  - [ ]* 11.2 Write property tests for slider
    - **Property 37: Slider value clamping**
    - **Property 38: Slider handle position reflects value**
    - **Validates: Requirements 10.1**
  
  - [x] 11.3 Implement toggle interaction system


    - Implement state toggling on click
    - Update visual state (checkmark visibility)
    - _Requirements: 10.2, 10.6**
  
  - [ ]* 11.4 Write property test for toggle state consistency
    - **Property 39: Toggle state consistency**
    - **Validates: Requirements 10.2, 10.6**
  
  - [x] 11.5 Implement dropdown interaction system


    - Implement dropdown list display/hide
    - Handle option selection
    - Update caption text
    - _Requirements: 10.3, 10.7**
  
  - [ ]* 11.6 Write property test for dropdown display
    - **Property 40: Dropdown displays selected option**
    - **Validates: Requirements 10.3, 10.7**
  
  - [x] 11.7 Implement input field interaction system



    - Implement text input handling
    - Implement cursor positioning and selection
    - Implement content type validation
    - _Requirements: 10.4, 10.8**
  
  - [ ]* 11.8 Write property test for input field validation
    - **Property 41: Input field content type validation**
    - **Validates: Requirements 10.8**

- [x] 12. Implement masking system





  - [x] 12.1 Implement stencil-based clipping


    - Create stencil buffer setup for masks
    - Clip children to mask bounds
    - Handle nested masks (intersection)
    - _Requirements: 11.1, 11.3_
  
  - [x] 12.2 Implement sprite alpha masking


    - Implement alpha-based masking using sprite alpha channel
    - _Requirements: 11.2_
  
  - [x] 12.3 Implement mask graphic visibility control


    - Control whether mask graphic itself is rendered
    - _Requirements: 11.4, 11.5_
  
  - [ ]* 12.4 Write property test for mask clipping
    - **Property 42: Mask clips children to bounds**
    - **Validates: Requirements 11.1**
  
  - [ ]* 12.5 Write property test for sprite alpha masking
    - **Property 43: Sprite alpha masking**
    - **Validates: Requirements 11.2**
  
  - [ ]* 12.6 Write property test for nested masks
    - **Property 44: Nested mask intersection**
    - **Validates: Requirements 11.3**
  
  - [ ]* 12.7 Write property test for mask graphic visibility
    - **Property 45: Mask graphic visibility**
    - **Validates: Requirements 11.4, 11.5**

- [x] 13. Implement UI rendering system











  - [x] 13.1 Implement 9-slice mesh generation


    - Create 9-slice mesh generator for UIImage
    - Handle border preservation
    - _Requirements: 4.5_
  
  - [ ]* 13.2 Write property test for 9-slice corner preservation
    - **Property 12: 9-slice corner preservation**
    - **Validates: Requirements 4.5**
  
  - [x] 13.3 Create UI batch builder



    - Collect all visible UI elements from hierarchy
    - Sort by canvas sort order, then Z-order
    - Group elements for batching by material/texture
    - Generate vertex and index buffers
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6_
  
  - [x] 13.4 Implement batching optimization


    - Batch elements with same material/texture
    - Break batches for Z-order changes
    - Handle dirty flagging for efficient updates
    - Implement culling for off-screen elements
    - _Requirements: 12.1, 12.2, 12.3, 12.6_
  
  - [ ]* 13.5 Write property test for UI batching
    - **Property 46: UI batching reduces draw calls**
    - **Validates: Requirements 12.1**
  
  - [ ]* 13.6 Write property test for Z-order batch breaking
    - **Property 47: Z-order breaks batches**
    - **Validates: Requirements 12.2**
  
  - [ ]* 13.7 Write property test for dirty marking
    - **Property 48: Property changes mark dirty**
    - **Validates: Requirements 12.3**
  
  - [ ]* 13.8 Write property test for transparent render order
    - **Property 49: Transparent elements render back-to-front**
    - **Validates: Requirements 12.4**
  
  - [ ]* 13.9 Write property test for canvas dirty rebuild
    - **Property 50: Canvas dirty triggers rebuild**
    - **Validates: Requirements 12.5**
  
  - [ ]* 13.10 Write property test for culling
    - **Property 51: Culled elements excluded from rendering**
    - **Validates: Requirements 12.6**
  
  - [x] 13.11 Integrate with render crate


    - Create UI render pass
    - Submit batches to sprite renderer
    - Handle multiple canvases with different render modes
    - _Requirements: 1.2, 1.3, 1.4, 1.6_

- [x] 14. Implement text rendering





  - [x] 14.1 Create text rendering system


    - Load and cache fonts (integrate with existing font system)
    - Generate text meshes from UIText components
    - Handle text alignment (9 positions)
    - Handle text overflow modes (wrap, truncate, overflow)
    - _Requirements: 4.2, 4.6_
  
  - [ ]* 14.2 Write property test for text overflow handling
    - **Property 13: Text overflow handling**
    - **Validates: Requirements 4.6**
  
  - [x] 14.3 Integrate text with UI rendering


    - Add text quads to UI batches
    - Handle text color and alpha
    - Apply UIElement color tint to text
    - _Requirements: 4.2, 4.7_
  
  - [ ]* 14.4 Write property test for color tint application
    - **Property 14: Color tint application**
    - **Validates: Requirements 4.7**

- [x] 15. Checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

- [x] 16. Implement prefab instantiation system




  - [x] 16.1 Implement prefab instantiation

    - Create entities from UIPrefab hierarchy
    - Apply all component values from prefab
    - Set up parent-child relationships
    - Support parameterization (override specific values)
    - _Requirements: 14.1, 14.2, 14.3_
  
  - [ ]* 16.2 Write property test for prefab serialization round-trip
    - **Property 58: Prefab serialization round-trip**
    - **Validates: Requirements 14.4, 14.5**
  
  - [ ]* 16.3 Write property test for prefab instantiation
    - **Property 59: Prefab instantiation completeness**
    - **Validates: Requirements 14.1, 14.2**
  
  - [ ]* 16.4 Write property test for prefab parameterization
    - **Property 60: Prefab parameterization**
    - **Validates: Requirements 14.3**

- [x] 17. Implement UI styling system




  - [x] 17.1 Implement style application system

    - Apply UIStyle to UI elements
    - Handle style inheritance from parent
    - Handle theme changes (update all elements)
    - Support style animations (smooth transitions)
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_
  
  - [ ]* 17.2 Write property test for style application
    - **Property 61: Style application updates visuals**
    - **Validates: Requirements 15.1, 15.2**
  
  - [ ]* 17.3 Write property test for theme changes
    - **Property 62: Theme change updates all elements**
    - **Validates: Requirements 15.3**
  
  - [ ]* 17.4 Write property test for style inheritance
    - **Property 63: Style inheritance**
    - **Validates: Requirements 15.4**
  
  - [ ]* 17.5 Write property test for style animations
    - **Property 64: Style animation transitions**
    - **Validates: Requirements 15.5**

- [x] 18. Implement Lua bindings






  - [x] 18.1 Create Lua API for UI creation


    - Bind Canvas creation functions
    - Bind UI element creation (Image, Text, Button, Panel, etc.)
    - Bind hierarchy operations (set_parent, get_children, destroy)
    - _Requirements: 13.1, 13.4_
  
  - [ ]* 18.2 Write property test for Lua element creation
    - **Property 52: Lua element creation**
    - **Validates: Requirements 13.1**
  
  - [ ]* 18.3 Write property test for Lua element destruction
    - **Property 55: Lua element destruction**
    - **Validates: Requirements 13.4**
  
  - [x] 18.4 Create Lua API for UI manipulation




    - Bind property getters and setters for all components
    - Bind animation functions (animate_position, animate_color, etc.)
    - Bind event callback registration
    - Bind element queries (find_by_name, find_by_tag)
    - _Requirements: 13.2, 13.3, 13.5, 13.6_
  
  - [ ]* 18.5 Write property test for Lua property modification
    - **Property 53: Lua property modification**
    - **Validates: Requirements 13.2**
  
  - [ ]* 18.6 Write property test for Lua callback registration
    - **Property 54: Lua callback registration**
    - **Validates: Requirements 13.3**
  
  - [ ]* 18.7 Write property test for Lua property queries
    - **Property 56: Lua property queries**
    - **Validates: Requirements 13.5**
  
  - [ ]* 18.8 Write property test for Lua animation execution
    - **Property 57: Lua animation execution**
    - **Validates: Requirements 13.6**

- [x] 19. Final checkpoint - Ensure all tests pass





  - Ensure all tests pass, ask the user if questions arise.

- [x] 20. Create UI examples and documentation





  - [x] 20.1 Create basic UI example


    - Create Rust example showing Canvas, Image, Text, Button
    - Demonstrate anchoring and layout
    - _Requirements: All core requirements_
  
  - [x] 20.2 Create advanced UI example


    - Create Rust example with Scroll View, Slider, Toggle, Dropdown, Input Field
    - Demonstrate animations and events
    - _Requirements: Advanced component requirements_
  
  - [x] 20.3 Create Lua UI example


    - Create Lua script that builds UI dynamically
    - Demonstrate Lua API usage
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6_
  
  - [x] 20.4 Write API documentation


    - Document all public types and functions with doc comments
    - Add usage examples to doc comments
    - Create getting started guide in README


---

## Migration Tasks (Legacy HUD System → UI Crate)

**Note:** These tasks handle the migration from `engine/src/hud` and `engine/src/editor/widget_editor` to the new `ui` crate system. See `MIGRATION_PLAN.md` for detailed migration strategy. **These tasks should only be started after tasks 1-20 are complete and the core UI system is functional.**

- [x] 21. Create HUD to UIPrefab converter




  - [x] 21.1 Implement converter core


    - Create HudToUIPrefabConverter struct
    - Implement HudAsset → UIPrefab conversion
    - Implement HudElement → UIPrefabElement conversion
    - Handle all HudElementType variants
    - _Migration Phase 3_
  
  - [x] 21.2 Implement anchor conversion

    - Convert Anchor enum to RectTransform
    - Map all 9 anchor positions correctly
    - Handle offset and size conversion
    - _Migration Phase 3_
  
  - [x] 21.3 Implement component mapping

    - Map Text → UIText
    - Map DynamicText → UIText (with notes for Lua binding)
    - Map HealthBar → UIImage (background) + UIImage (fill)
    - Map ProgressBar → UIImage (background) + UIImage (fill)
    - Map Image → UIImage
    - Map Container → parent-child hierarchy
    - Handle Minimap (custom component or notes)
    - _Migration Phase 3_
  
  - [ ]* 21.4 Write unit tests for converter
    - Test anchor conversion for all 9 positions
    - Test component mapping for all types
    - Test hierarchy preservation
    - Test property preservation
    - _Migration Phase 3_

- [x] 22. Create migration script





  - [x] 22.1 Implement file discovery


    - Recursively find all .hud files in project
    - Support multiple project directories
    - _Migration Phase 3_
  


  - [x] 22.2 Implement batch conversion




    - Load each .hud file
    - Convert to UIPrefab
    - Save as .uiprefab file
    - Generate migration report


    - _Migration Phase 3_
  
  - [x] 22.3 Create migration CLI tool





    - Add command-line arguments
    - Support dry-run mode


    - Support backup creation
    - Add progress reporting
    - _Migration Phase 3_
  
  - [x] 22.4 Test migration on sample HUD files




    - Test with simple HUD
    - Test with complex HUD
    - Test with nested containers
    - Verify visual output matches
    - _Migration Phase 3_

- [ ] 23. Refactor Widget Editor → UI Prefab Editor
  - [ ] 23.1 Update editor data structures
    - Replace HudAsset with UIPrefab
    - Replace HudElement with UIPrefabElement
    - Update WidgetEditorState to PrefabEditorState
    - _Migration Phase 4_
  
  - [ ] 23.2 Implement prefab loading/saving
    - Load .uiprefab files
    - Save .uiprefab files
    - Handle file format validation
    - _Migration Phase 4_
  
  - [ ] 23.3 Update canvas rendering
    - Render UIPrefabElements with RectTransform
    - Show anchor visualization
    - Show pivot point
    - Handle all UI component types
    - _Migration Phase 4_
  
  - [ ] 23.4 Implement RectTransform visual editing
    - Anchor point handles
    - Pivot point manipulation
    - Size handles (corners and edges)
    - Position dragging
    - _Migration Phase 4_
  
  - [ ] 23.5 Create component palette
    - List all available UI components
    - Drag-and-drop to add components
    - Component icons and descriptions
    - _Migration Phase 4_
  
  - [ ] 23.6 Create hierarchy panel
    - Tree view of UI element hierarchy
    - Drag-and-drop to reparent
    - Show/hide elements
    - Delete elements
    - Duplicate elements
    - _Migration Phase 4_
  
  - [ ] 23.7 Enhance properties panel
    - Edit RectTransform properties
    - Edit all component properties
    - Color pickers
    - Sprite selectors
    - Event callback editors
    - _Migration Phase 4_
  
  - [ ] 23.8 Implement layout preview
    - Preview different resolutions
    - Show layout group effects
    - Toggle grid and safe area
    - _Migration Phase 4_
  
  - [ ] 23.9 Add undo/redo system
    - Command pattern implementation
    - Undo/redo stack
    - Keyboard shortcuts (Ctrl+Z, Ctrl+Y)
    - _Migration Phase 4_

- [ ] 24. Update engine integration
  - [ ] 24.1 Remove HUD system from engine
    - Delete engine/src/hud module
    - Remove HudManager from engine state
    - Remove HUD rendering code
    - _Migration Phase 5_
  
  - [ ] 24.2 Integrate UI system with engine
    - Add ui crate dependency to engine
    - Create UI system manager
    - Integrate with ECS world
    - Integrate with rendering pipeline
    - _Migration Phase 5_
  
  - [ ] 24.3 Update Lua bindings
    - Remove old HUD Lua API
    - Add new UI Lua API
    - Update Lua scripts in examples
    - _Migration Phase 5_
  
  - [ ] 24.4 Update examples
    - Convert example HUD files to prefabs
    - Update example Lua scripts
    - Update example documentation
    - _Migration Phase 5_

- [ ] 25. Create migration documentation
  - [ ] 25.1 Write migration guide
    - Step-by-step migration instructions
    - Before/after code examples
    - Common issues and solutions
    - _Migration Phase 5_
  
  - [ ] 25.2 Document API changes
    - Old API → New API mapping
    - Breaking changes list
    - Deprecation notices
    - _Migration Phase 5_
  
  - [ ] 25.3 Create video tutorials (optional)
    - Using the new UI system
    - Using the UI Prefab Editor
    - Migrating existing HUD files
    - _Migration Phase 5_
  
  - [ ] 25.4 Update README files
    - Update main README
    - Update UI system README
    - Add migration notes
    - _Migration Phase 5_

- [ ] 26. Final migration verification
  - [ ] 26.1 Verify all HUD files migrated
    - Check all .hud files converted
    - Verify no references to old system
    - Test all converted prefabs
    - _Migration Phase 5_
  
  - [ ] 26.2 Performance testing
    - Benchmark UI rendering
    - Compare with legacy system
    - Optimize if needed
    - _Migration Phase 5_
  
  - [ ] 26.3 Visual regression testing
    - Screenshot comparison
    - Verify layouts match
    - Test multiple resolutions
    - _Migration Phase 5_
  
  - [ ] 26.4 Final cleanup
    - Remove migration tools (or move to tools/)
    - Remove temporary compatibility code
    - Update version numbers
    - Create migration completion report
    - _Migration Phase 5_

---

## Migration Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| Phase 1: Foundation | Tasks 1-7 | 2 weeks | None |
| Phase 2: Advanced Features | Tasks 8-18 | 2 weeks | Phase 1 |
| Phase 3: Migration Tools | Tasks 21-22 | 1 week | Phase 2 |
| Phase 4: UI Prefab Editor | Task 23 | 3 weeks | Phase 3 |
| Phase 5: Cleanup | Tasks 24-26 | 1 week | Phase 4 |
| **Total** | **26 task groups** | **9 weeks** | Sequential |

**Note:** Migration tasks (21-26) should only be started after the core UI system (tasks 1-20) is functional and tested.

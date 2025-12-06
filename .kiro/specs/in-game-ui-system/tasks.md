# Implementation Plan

- [x] 1. Set up UI crate structure and core types





  - Create new `ui` crate with module structure
  - Define core types: Vec2, Vec4, Rect, Color
  - Set up dependencies (ecs, render, serde, glam)
  - Create public API in lib.rs with re-exports
  - _Requirements: All requirements - foundation_

- [x] 2. Implement RectTransform system




  - [x] 2.1 Create RectTransform component


    - Define RectTransform struct with anchor, pivot, position, size
    - Implement helper methods (anchored, stretched, get_size, set_size)
    - Implement contains_point for raycasting
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
  

  - [ ] 4.2 Implement hierarchy propagation systems
    - Create system for transform propagation
    - Create system for visibility propagation
    - Create system for destruction propagation
    - Handle sibling index ordering
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

- [ ] 5. Implement core UI components
  - [ ] 5.1 Create UIImage component
    - Define UIImage struct with sprite, image type, 9-slice
    - Implement fill methods (horizontal, vertical, radial)
    - Implement preserve aspect ratio
    - _Requirements: 4.1, 4.5_
  
  - [ ] 5.2 Implement 9-slice rendering
    - Create 9-slice mesh generation
    - Handle border preservation
    - _Requirements: 4.5_
  
  - [ ]* 5.3 Write property test for 9-slice corner preservation
    - **Property 12: 9-slice corner preservation**
    - **Validates: Requirements 4.5**
  
  - [ ] 5.4 Create UIText component
    - Define UIText struct with font, size, color, alignment
    - Implement overflow modes (wrap, truncate, overflow)
    - Implement rich text support
    - _Requirements: 4.2, 4.6_
  
  - [ ]* 5.5 Write property test for text overflow handling
    - **Property 13: Text overflow handling**
    - **Validates: Requirements 4.6**
  
  - [ ] 5.6 Create UIButton component
    - Define UIButton struct with states and transitions
    - Implement color tint transition
    - Implement sprite swap transition
    - Handle button state changes
    - _Requirements: 4.3_
  
  - [ ] 5.7 Create UIPanel component
    - Define UIPanel struct with background and padding
    - Support 9-slice backgrounds
    - _Requirements: 4.4_
  
  - [ ]* 5.8 Write property test for color tint application
    - **Property 14: Color tint application**
    - **Validates: Requirements 4.7**

- [ ] 6. Implement layout system
  - [ ] 6.1 Create layout group components
    - Define HorizontalLayoutGroup struct
    - Define VerticalLayoutGroup struct
    - Define GridLayoutGroup struct
    - _Requirements: 5.1, 5.2, 5.3_
  
  - [ ] 6.2 Implement layout calculation system
    - Create horizontal layout algorithm
    - Create vertical layout algorithm
    - Create grid layout algorithm
    - Handle padding, spacing, and alignment
    - Handle force expand and child control
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_
  
  - [ ]* 6.3 Write property test for layout spacing
    - **Property 15: Layout spacing consistency**
    - **Validates: Requirements 5.1, 5.2, 5.3**
  
  - [ ]* 6.4 Write property test for layout padding
    - **Property 16: Layout padding application**
    - **Validates: Requirements 5.4**
  
  - [ ]* 6.5 Write property test for layout alignment
    - **Property 17: Layout child alignment**
    - **Validates: Requirements 5.5**
  
  - [ ]* 6.6 Write property test for layout force expand
    - **Property 18: Layout force expand**
    - **Validates: Requirements 5.6**
  
  - [ ]* 6.7 Write property test for layout recalculation
    - **Property 19: Layout recalculation on size change**
    - **Validates: Requirements 5.7**

- [ ] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement event system
  - [ ] 8.1 Create UI event types and handler
    - Define UIEvent enum with all event types
    - Define UIEventHandler with listener registration
    - Track hover, pressed, and drag state
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_
  
  - [ ] 8.2 Implement UI raycasting
    - Create raycast system to find elements at point
    - Handle raycast target filtering
    - Handle raycast blocking
    - Sort by Z-order for correct event delivery
    - _Requirements: 6.1, 6.6, 6.7_
  
  - [ ]* 8.3 Write property test for raycast target inclusion
    - **Property 20: Raycast target inclusion**
    - **Validates: Requirements 6.1**
  
  - [ ]* 8.4 Write property test for event delivery to topmost
    - **Property 21: Event delivery to topmost element**
    - **Validates: Requirements 6.6**
  
  - [ ]* 8.5 Write property test for raycast blocking
    - **Property 22: Raycast blocking**
    - **Validates: Requirements 6.7**
  
  - [ ] 8.3 Implement input event processing
    - Process mouse/touch input
    - Generate UI events (click, hover, drag)
    - Dispatch events to elements
    - Invoke Lua callbacks
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.8_
  
  - [ ]* 8.7 Write property test for event callback invocation
    - **Property 23: Event callback invocation**
    - **Validates: Requirements 6.2, 6.3, 6.4, 6.5, 6.8**

- [ ] 9. Implement animation system
  - [ ] 9.1 Create animation types and system
    - Define UIAnimation struct with property, values, easing
    - Define EasingFunction enum with all easing types
    - Implement easing function calculations
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_
  
  - [ ] 9.2 Implement animation update system
    - Update animation elapsed time
    - Calculate interpolated values
    - Apply values to components
    - Handle animation completion
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.8_
  
  - [ ]* 9.3 Write property test for animation interpolation
    - **Property 29: Animation interpolation correctness**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7**
  
  - [ ]* 9.4 Write property test for animation completion callback
    - **Property 30: Animation completion callback**
    - **Validates: Requirements 8.8**

- [ ] 10. Implement scroll view system
  - [ ] 10.1 Create UIScrollView component
    - Define UIScrollView struct with content, viewport, scrollbars
    - Define movement types (unrestricted, elastic, clamped)
    - Track scroll position and velocity
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8_
  
  - [ ] 10.2 Implement scroll view systems
    - Create viewport clipping system
    - Implement drag scrolling
    - Implement scrollbar updates
    - Implement programmatic scrolling
    - Implement elastic spring-back
    - Implement inertia deceleration
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8_
  
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

- [ ] 11. Implement advanced UI components
  - [ ] 11.1 Create UISlider component
    - Define UISlider struct with fill, handle, min/max
    - Implement value clamping
    - Implement handle positioning
    - Handle drag interaction
    - _Requirements: 10.1, 10.5_
  
  - [ ]* 11.2 Write property tests for slider
    - **Property 37: Slider value clamping**
    - **Property 38: Slider handle position reflects value**
    - **Validates: Requirements 10.1**
  
  - [ ] 11.3 Create UIToggle component
    - Define UIToggle struct with graphic and state
    - Implement state toggling
    - Update visual state
    - _Requirements: 10.2, 10.6_
  
  - [ ]* 11.4 Write property test for toggle state consistency
    - **Property 39: Toggle state consistency**
    - **Validates: Requirements 10.2, 10.6**
  
  - [ ] 11.5 Create UIDropdown component
    - Define UIDropdown struct with template, options
    - Implement dropdown list display
    - Handle option selection
    - _Requirements: 10.3, 10.7_
  
  - [ ]* 11.6 Write property test for dropdown display
    - **Property 40: Dropdown displays selected option**
    - **Validates: Requirements 10.3, 10.7**
  
  - [ ] 11.7 Create UIInputField component
    - Define UIInputField struct with text, validation
    - Implement text input handling
    - Implement cursor and selection
    - Implement content type validation
    - _Requirements: 10.4, 10.8_
  
  - [ ]* 11.8 Write property test for input field validation
    - **Property 41: Input field content type validation**
    - **Validates: Requirements 10.8**

- [ ] 12. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Implement masking system
  - [ ] 13.1 Create UIMask component
    - Define UIMask struct with show_mask_graphic, use_sprite_alpha
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_
  
  - [ ] 13.2 Implement masking rendering
    - Create stencil-based clipping
    - Implement sprite alpha masking
    - Handle nested masks
    - Control mask graphic visibility
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_
  
  - [ ]* 13.3 Write property test for mask clipping
    - **Property 42: Mask clips children to bounds**
    - **Validates: Requirements 11.1**
  
  - [ ]* 13.4 Write property test for sprite alpha masking
    - **Property 43: Sprite alpha masking**
    - **Validates: Requirements 11.2**
  
  - [ ]* 13.5 Write property test for nested masks
    - **Property 44: Nested mask intersection**
    - **Validates: Requirements 11.3**
  
  - [ ]* 13.6 Write property test for mask graphic visibility
    - **Property 45: Mask graphic visibility**
    - **Validates: Requirements 11.4, 11.5**

- [ ] 14. Implement UI rendering system
  - [ ] 14.1 Create UI batch builder
    - Collect all visible UI elements
    - Sort by canvas, Z-order
    - Group elements for batching
    - Generate vertex and index buffers
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6_
  
  - [ ] 14.2 Implement batching optimization
    - Batch elements with same material/texture
    - Break batches for Z-order changes
    - Handle dirty flagging
    - Implement culling
    - _Requirements: 12.1, 12.2, 12.3, 12.6_
  
  - [ ]* 14.3 Write property test for UI batching
    - **Property 46: UI batching reduces draw calls**
    - **Validates: Requirements 12.1**
  
  - [ ]* 14.4 Write property test for Z-order batch breaking
    - **Property 47: Z-order breaks batches**
    - **Validates: Requirements 12.2**
  
  - [ ]* 14.5 Write property test for dirty marking
    - **Property 48: Property changes mark dirty**
    - **Validates: Requirements 12.3**
  
  - [ ]* 14.6 Write property test for transparent render order
    - **Property 49: Transparent elements render back-to-front**
    - **Validates: Requirements 12.4**
  
  - [ ]* 14.7 Write property test for canvas dirty rebuild
    - **Property 50: Canvas dirty triggers rebuild**
    - **Validates: Requirements 12.5**
  
  - [ ]* 14.8 Write property test for culling
    - **Property 51: Culled elements excluded from rendering**
    - **Validates: Requirements 12.6**
  
  - [ ] 14.9 Integrate with render crate
    - Create UI render pass
    - Submit batches to sprite renderer
    - Handle multiple canvases
    - _Requirements: 1.2, 1.3, 1.4, 1.6_

- [ ] 15. Implement text rendering
  - [ ] 15.1 Create text rendering system
    - Load and cache fonts
    - Generate text meshes
    - Handle text alignment
    - Handle text overflow modes
    - Support rich text markup
    - _Requirements: 4.2, 4.6_
  
  - [ ] 15.2 Integrate text with UI rendering
    - Add text to UI batches
    - Handle text color and alpha
    - _Requirements: 4.2_

- [ ] 16. Implement Lua bindings
  - [ ] 16.1 Create Lua API for UI creation
    - Bind Canvas creation
    - Bind UI element creation (Image, Text, Button, Panel, etc.)
    - Bind hierarchy operations (set parent, get children)
    - _Requirements: 13.1, 13.4_
  
  - [ ]* 16.2 Write property test for Lua element creation
    - **Property 52: Lua element creation**
    - **Validates: Requirements 13.1**
  
  - [ ]* 16.3 Write property test for Lua element destruction
    - **Property 55: Lua element destruction**
    - **Validates: Requirements 13.4**
  
  - [ ] 16.4 Create Lua API for UI manipulation
    - Bind property getters and setters
    - Bind animation functions
    - Bind event callback registration
    - Bind element queries (by name, tag)
    - _Requirements: 13.2, 13.3, 13.5, 13.6_
  
  - [ ]* 16.5 Write property test for Lua property modification
    - **Property 53: Lua property modification**
    - **Validates: Requirements 13.2**
  
  - [ ]* 16.6 Write property test for Lua callback registration
    - **Property 54: Lua callback registration**
    - **Validates: Requirements 13.3**
  
  - [ ]* 16.7 Write property test for Lua property queries
    - **Property 56: Lua property queries**
    - **Validates: Requirements 13.5**
  
  - [ ]* 16.8 Write property test for Lua animation execution
    - **Property 57: Lua animation execution**
    - **Validates: Requirements 13.6**

- [ ] 17. Implement UI prefab system
  - [ ] 17.1 Create UIPrefab types
    - Define UIPrefab and UIPrefabElement structs
    - Support all UI component types
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_
  
  - [ ] 17.2 Implement prefab serialization
    - Serialize UI hierarchy to JSON
    - Deserialize JSON to UI hierarchy
    - _Requirements: 14.4, 14.5_
  
  - [ ]* 17.3 Write property test for prefab serialization round-trip
    - **Property 58: Prefab serialization round-trip**
    - **Validates: Requirements 14.4, 14.5**
  
  - [ ] 17.4 Implement prefab instantiation
    - Create entities from prefab hierarchy
    - Apply component values
    - Support parameterization
    - _Requirements: 14.1, 14.2, 14.3_
  
  - [ ]* 17.5 Write property test for prefab instantiation
    - **Property 59: Prefab instantiation completeness**
    - **Validates: Requirements 14.1, 14.2**
  
  - [ ]* 17.6 Write property test for prefab parameterization
    - **Property 60: Prefab parameterization**
    - **Validates: Requirements 14.3**

- [ ] 18. Implement UI styling system
  - [ ] 18.1 Create UIStyle and UITheme types
    - Define UIStyle struct with colors, fonts, sprites
    - Define UITheme struct with style collection
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_
  
  - [ ] 18.2 Implement style application system
    - Apply styles to UI elements
    - Handle style inheritance
    - Handle theme changes
    - Support style animations
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_
  
  - [ ]* 18.3 Write property test for style application
    - **Property 61: Style application updates visuals**
    - **Validates: Requirements 15.1, 15.2**
  
  - [ ]* 18.4 Write property test for theme changes
    - **Property 62: Theme change updates all elements**
    - **Validates: Requirements 15.3**
  
  - [ ]* 18.5 Write property test for style inheritance
    - **Property 63: Style inheritance**
    - **Validates: Requirements 15.4**
  
  - [ ]* 18.6 Write property test for style animations
    - **Property 64: Style animation transitions**
    - **Validates: Requirements 15.5**

- [ ] 19. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 20. Create UI examples and documentation
  - [ ] 20.1 Create basic UI example
    - Create example showing Canvas, Image, Text, Button
    - Demonstrate anchoring and layout
    - _Requirements: All core requirements_
  
  - [ ] 20.2 Create advanced UI example
    - Create example with Scroll View, Slider, Toggle, Dropdown, Input Field
    - Demonstrate animations and events
    - _Requirements: Advanced component requirements_
  
  - [ ] 20.3 Create Lua UI example
    - Create Lua script that builds UI dynamically
    - Demonstrate Lua API usage
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6_
  
  - [ ] 20.4 Write API documentation
    - Document all public types and functions
    - Add usage examples to doc comments
    - Create getting started guide


---

## Migration Tasks (Legacy HUD System → UI Crate)

**Note:** These tasks handle the migration from `engine/src/hud` and `engine/src/editor/widget_editor` to the new `ui` crate system. See `MIGRATION_PLAN.md` for detailed migration strategy.

- [ ] 21. Create HUD to UIPrefab converter
  - [ ] 21.1 Implement converter core
    - Create HudToUIPrefabConverter struct
    - Implement HudAsset → UIPrefab conversion
    - Implement HudElement → UIPrefabElement conversion
    - Handle all HudElementType variants
    - _Migration Phase 3_
  
  - [ ] 21.2 Implement anchor conversion
    - Convert Anchor enum to RectTransform
    - Map all 9 anchor positions correctly
    - Handle offset and size conversion
    - _Migration Phase 3_
  
  - [ ] 21.3 Implement component mapping
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

- [ ] 22. Create migration script
  - [ ] 22.1 Implement file discovery
    - Recursively find all .hud files in project
    - Support multiple project directories
    - _Migration Phase 3_
  
  - [ ] 22.2 Implement batch conversion
    - Load each .hud file
    - Convert to UIPrefab
    - Save as .uiprefab file
    - Generate migration report
    - _Migration Phase 3_
  
  - [ ] 22.3 Create migration CLI tool
    - Add command-line arguments
    - Support dry-run mode
    - Support backup creation
    - Add progress reporting
    - _Migration Phase 3_
  
  - [ ] 22.4 Test migration on sample HUD files
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

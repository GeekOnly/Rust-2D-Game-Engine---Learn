# Sprite Editor Implementation Plan

- [x] 1. Create core data structures and file I/O





  - Create `SpriteDefinition` struct with name, x, y, width, height
  - Create `SpriteMetadata` struct with texture path, dimensions, and sprite list
  - Implement JSON serialization/deserialization for sprite metadata
  - Implement save/load functions for .sprite files
  - Add backup creation before overwriting existing files
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ]* 1.1 Write property test for sprite metadata serialization
  - **Property 1: Metadata loading preserves sprite definitions**
  - **Validates: Requirements 1.4**

- [ ]* 1.2 Write property test for save includes all data
  - **Property 15: Save includes all sprite data**
  - **Property 16: Save includes texture path**
  - **Validates: Requirements 5.2, 5.3**

- [x] 2. Create SpriteEditorState and window structure










  - Create `SpriteEditorState` struct with editor state fields
  - Create `SpriteEditorWindow` struct with render method
  - Implement texture loading using TextureManager
  - Add zoom and pan state management
  - Initialize undo/redo stacks
  - _Requirements: 1.2, 1.3_

- [ ] 3. Implement sprite canvas rendering





  - Render loaded texture on canvas
  - Implement zoom controls (mouse wheel)
  - Implement pan controls (middle mouse drag)
  - Render sprite rectangles with borders
  - Render sprite name labels on canvas
  - _Requirements: 1.3, 3.5_

- [ ]* 3.1 Write property test for label display
  - **Property 10: Labels display sprite names**
  - **Validates: Requirements 3.5**

- [ ] 4. Implement sprite rectangle creation
  - Handle click-and-drag to create new sprite rectangle
  - Validate rectangle has positive dimensions
  - Assign default sequential name (sprite_0, sprite_1, etc.)
  - Add new sprite to metadata
  - Push state to undo stack
  - _Requirements: 2.1, 2.2, 3.1_

- [ ]* 4.1 Write property test for rectangle creation
  - **Property 2: Rectangle creation produces valid sprites**
  - **Property 6: Default naming is sequential**
  - **Validates: Requirements 2.1, 2.2, 3.1**

- [ ] 5. Implement sprite selection and highlighting
  - Handle click to select sprite
  - Render selected sprite with yellow border
  - Render hovered sprite with white border
  - Render unselected sprites with blue border
  - Update selected_sprite in state
  - _Requirements: 6.1_

- [ ]* 5.1 Write property test for selection highlighting
  - **Property 18: Selection highlights sprite**
  - **Validates: Requirements 6.1**

- [ ] 6. Implement sprite rectangle editing
  - Render resize handles at corners (8x8px squares)
  - Handle corner drag to resize sprite
  - Handle center drag to move sprite
  - Clamp sprite to texture bounds
  - Validate sprite maintains positive dimensions
  - Push state to undo stack after edit
  - _Requirements: 2.3, 2.4_

- [ ]* 6.1 Write property test for handle dragging
  - **Property 3: Handle dragging maintains rectangle validity**
  - **Property 4: Center dragging preserves sprite dimensions**
  - **Validates: Requirements 2.3, 2.4**

- [ ] 7. Implement sprite deletion
  - Handle Delete key press
  - Remove selected sprite from metadata
  - Clear selection
  - Push state to undo stack
  - _Requirements: 2.5_

- [ ]* 7.1 Write property test for deletion
  - **Property 5: Delete removes sprite from list**
  - **Validates: Requirements 2.5**

- [ ] 8. Create properties panel UI
  - Display selected sprite properties (name, x, y, width, height)
  - Add text input for sprite name editing
  - Validate name is not duplicate
  - Show warning for duplicate names
  - Update sprite name on edit
  - Display sprite preview image
  - Display sprite dimensions in pixels
  - _Requirements: 3.2, 3.3, 3.4, 6.2, 6.5_

- [ ]* 8.1 Write property test for properties display
  - **Property 7: Selection displays properties**
  - **Property 8: Name editing updates sprite**
  - **Property 9: Duplicate names are rejected**
  - **Validates: Requirements 3.2, 3.3, 3.4**

- [ ]* 8.2 Write property test for preview
  - **Property 19: Selection shows preview**
  - **Property 21: Preview displays dimensions**
  - **Validates: Requirements 6.2, 6.5**

- [ ] 9. Create sprite list panel UI
  - Display list of all sprites with thumbnails
  - Handle click to select sprite from list
  - Highlight selected sprite in list
  - Show sprite count in header
  - _Requirements: 9.2_

- [ ]* 9.1 Write property test for sprite count
  - **Property 29: Sprite count is accurate**
  - **Validates: Requirements 9.2**

- [ ] 10. Implement auto-slice functionality
  - Create AutoSlicer struct with grid slicing methods
  - Add "Auto Slice" button to toolbar
  - Show dialog with grid options (columns, rows, padding, spacing)
  - Calculate sprite dimensions from grid parameters
  - Create sprite rectangles in grid layout
  - Name sprites sequentially
  - Push state to undo stack
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [ ]* 10.1 Write property test for grid slicing
  - **Property 11: Grid slicing calculates correct dimensions**
  - **Property 12: Grid slicing creates correct count**
  - **Property 13: Auto-slice naming is sequential**
  - **Property 14: Padding affects sprite positions**
  - **Validates: Requirements 4.2, 4.3, 4.4, 4.5**

- [ ] 11. Implement undo/redo system
  - Handle Ctrl+Z to undo last action
  - Handle Ctrl+Y to redo last undone action
  - Maintain undo stack (limit to 50 actions)
  - Maintain redo stack
  - Clear redo stack on new action
  - _Requirements: 8.3, 8.4_

- [ ]* 11.1 Write property test for undo/redo
  - **Property 27: Undo reverses last action**
  - **Property 28: Redo restores undone action**
  - **Validates: Requirements 8.3, 8.4**

- [ ] 12. Implement keyboard shortcuts
  - Handle Ctrl+S to save
  - Handle Delete to remove selected sprite
  - Handle Escape to deselect
  - Handle Tab to cycle through overlapping sprites
  - Display keyboard hints in UI
  - _Requirements: 8.1, 8.2, 8.5, 6.4_

- [ ] 13. Implement sprite statistics and validation
  - Calculate and display texture coverage percentage
  - Detect overlapping sprites and show warning
  - Detect out-of-bounds sprites and show error
  - Display texture dimensions
  - Update statistics when sprites change
  - _Requirements: 9.1, 9.3, 9.4, 9.5_

- [ ]* 13.1 Write property test for statistics
  - **Property 30: Coverage calculation is correct**
  - **Property 31: Overlapping sprites show warning**
  - **Property 32: Out-of-bounds sprites show error**
  - **Validates: Requirements 9.3, 9.4, 9.5**

- [ ] 14. Implement hover tooltips
  - Detect mouse hover over sprite rectangles
  - Display tooltip with sprite name
  - Position tooltip near cursor
  - Hide tooltip when not hovering
  - _Requirements: 6.3_

- [ ]* 14.1 Write property test for tooltips
  - **Property 20: Hover shows tooltip**
  - **Validates: Requirements 6.3**

- [ ] 15. Integrate with asset browser
  - Add "Open in Sprite Editor" to PNG context menu
  - Handle menu selection to open sprite editor window
  - Display .sprite files with sprite icon in asset browser
  - Show sprite count badge on .sprite files
  - Allow expanding .sprite files to show individual sprites
  - _Requirements: 1.1, 7.1_

- [ ]* 15.1 Write property test for asset browser integration
  - **Property 22: Sprite files list in asset browser**
  - **Validates: Requirements 7.1**

- [ ] 16. Implement drag-drop sprites to scene
  - Handle drag start from asset browser
  - Handle drop onto scene view
  - Create entity with Transform, Sprite, SpriteSheet, AnimatedSprite components
  - Set sprite frame index to match dropped sprite
  - _Requirements: 7.2_

- [ ]* 16.1 Write property test for drag-drop
  - **Property 23: Drag-drop creates entity with sprite**
  - **Validates: Requirements 7.2**

- [ ] 17. Update sprite rendering to use sprite regions
  - Modify renderer to load .sprite metadata
  - Calculate UV coordinates from sprite definition
  - Render only sprite region from texture
  - Update both scene view and game view renderers
  - _Requirements: 7.3_

- [ ]* 17.1 Write property test for sprite rendering
  - **Property 24: Entity renders only sprite region**
  - **Validates: Requirements 7.3**

- [ ] 18. Update inspector to show sprite info
  - Display sprite name in SpriteSheet component
  - Display source texture path
  - Add "Edit Sprite Sheet" button
  - Handle button click to open sprite editor
  - _Requirements: 7.4_

- [ ]* 18.1 Write property test for inspector display
  - **Property 25: Inspector shows sprite info**
  - **Validates: Requirements 7.4**

- [ ] 19. Implement sprite definition hot-reloading
  - Watch .sprite files for changes
  - Reload sprite metadata when file changes
  - Update all entities using changed sprites
  - Refresh sprite editor if open
  - _Requirements: 7.5_

- [ ]* 19.1 Write property test for hot-reloading
  - **Property 26: Sprite changes update entities**
  - **Validates: Requirements 7.5**

- [ ] 20. Implement export functionality
  - Add "Export" button to toolbar
  - Show dialog with format options (JSON, XML, TexturePacker)
  - Implement JSON export with standard format
  - Implement XML export
  - Implement TexturePacker format export
  - Save exported file to project directory
  - Show success/error message
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ]* 20.1 Write property test for export
  - **Property 33: Export includes all metadata**
  - **Validates: Requirements 10.3**

- [ ] 21. Add toolbar with action buttons
  - Create toolbar UI at top of window
  - Add Save button (with Ctrl+S hint)
  - Add Auto Slice button
  - Add Export button
  - Add Undo button (with Ctrl+Z hint)
  - Add Redo button (with Ctrl+Y hint)
  - Disable buttons when actions are unavailable
  - _Requirements: 8.1_

- [ ] 22. Implement error handling and user feedback
  - Show error dialog for missing texture files
  - Show error dialog for invalid JSON
  - Show error dialog for write permission denied
  - Show warning for backup creation failure
  - Show success message after save
  - Show inline errors for validation issues
  - _Requirements: 5.5_

- [ ] 23. Add status bar with information
  - Display sprite count
  - Display texture coverage percentage
  - Display current zoom level
  - Display texture dimensions
  - Update status bar when state changes
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 24. Optimize performance for large textures
  - Implement viewport culling for sprite rectangles
  - Batch sprite rectangle rendering
  - Use texture mipmaps for zoomed-out views
  - Limit undo stack to 50 actions
  - Debounce auto-save (2 seconds after last edit)
  - _Requirements: Performance considerations from design_

- [ ]* 24.1 Write performance tests
  - Test with 2048x2048 texture
  - Test with 100+ sprites
  - Verify UI remains responsive

- [ ] 25. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

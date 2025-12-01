# Sprite Editor Design

## Overview

The Sprite Editor is a visual tool integrated into the game engine that allows developers to slice sprite sheet textures into individual named sprites. It provides an interactive canvas for drawing sprite rectangles, automatic grid-based slicing, and exports sprite metadata to JSON files that can be used at runtime.

The editor follows a similar workflow to Unity's Sprite Editor and Aseprite's sprite sheet tools, providing both manual and automatic slicing capabilities.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Sprite Editor Window                     │
├──────────────────────┬──────────────────────────────────────┤
│                      │                                       │
│   Toolbar            │   Canvas Area                         │
│   - Save             │   - Texture Display                   │
│   - Auto Slice       │   - Sprite Rectangles                 │
│   - Export           │   - Zoom/Pan Controls                 │
│   - Undo/Redo        │   - Selection Handles                 │
│                      │                                       │
├──────────────────────┼──────────────────────────────────────┤
│                      │                                       │
│   Sprite List        │   Properties Panel                    │
│   - sprite_0         │   - Name: [text input]                │
│   - sprite_1         │   - X: 0                              │
│   - sprite_2         │   - Y: 0                              │
│   - ...              │   - Width: 32                         │
│                      │   - Height: 32                        │
│                      │   - Preview: [image]                  │
└──────────────────────┴──────────────────────────────────────┘
```

### Component Structure

1. **SpriteEditorWindow**: Main window container (egui window)
2. **SpriteCanvas**: Interactive canvas for drawing/editing sprite rectangles
3. **SpriteDefinition**: Data structure for individual sprite metadata
4. **SpriteMetadata**: Container for all sprites in a sheet
5. **SpriteEditorState**: Editor state (selected sprite, tool mode, undo stack)
6. **AutoSlicer**: Grid-based automatic slicing algorithm
7. **SpriteExporter**: Export to various formats

## Components and Interfaces

### SpriteDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteDefinition {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

### SpriteMetadata

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteMetadata {
    pub texture_path: String,
    pub texture_width: u32,
    pub texture_height: u32,
    pub sprites: Vec<SpriteDefinition>,
}
```

### SpriteEditorState

```rust
pub struct SpriteEditorState {
    // File management
    pub texture_path: PathBuf,
    pub metadata_path: PathBuf,
    pub metadata: SpriteMetadata,
    
    // Editor state
    pub selected_sprite: Option<usize>,
    pub hovered_sprite: Option<usize>,
    pub is_drawing: bool,
    pub draw_start: Option<(f32, f32)>,
    pub draw_current: Option<(f32, f32)>,
    
    // View state
    pub zoom: f32,
    pub pan_offset: (f32, f32),
    
    // Undo/Redo
    pub undo_stack: Vec<SpriteMetadata>,
    pub redo_stack: Vec<SpriteMetadata>,
    
    // Texture
    pub texture_handle: Option<TextureHandle>,
}
```

### SpriteEditorWindow

```rust
pub struct SpriteEditorWindow {
    pub state: SpriteEditorState,
    pub is_open: bool,
}

impl SpriteEditorWindow {
    pub fn new(texture_path: PathBuf) -> Self;
    pub fn render(&mut self, ctx: &egui::Context, texture_manager: &mut TextureManager);
    pub fn save(&mut self) -> Result<(), String>;
    pub fn load_metadata(&mut self) -> Result<(), String>;
    pub fn auto_slice(&mut self, columns: u32, rows: u32, padding: u32, spacing: u32);
    pub fn export(&self, format: ExportFormat) -> Result<(), String>;
}
```

### AutoSlicer

```rust
pub struct AutoSlicer;

impl AutoSlicer {
    pub fn slice_by_grid(
        texture_width: u32,
        texture_height: u32,
        columns: u32,
        rows: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition>;
    
    pub fn slice_by_cell_size(
        texture_width: u32,
        texture_height: u32,
        cell_width: u32,
        cell_height: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition>;
}
```

## Data Models

### File Format (.sprite JSON)

```json
{
  "texture_path": "assets/characters/knight.png",
  "texture_width": 512,
  "texture_height": 256,
  "sprites": [
    {
      "name": "knight_idle_0",
      "x": 0,
      "y": 0,
      "width": 32,
      "height": 32
    },
    {
      "name": "knight_run_0",
      "x": 32,
      "y": 0,
      "width": 32,
      "height": 32
    }
  ]
}
```

### Integration with ECS

The existing `SpriteSheet` component in ECS will be updated to support loading from `.sprite` files:

```rust
// In ecs/src/components/sprite_sheet.rs
impl SpriteSheet {
    pub fn from_sprite_file(sprite_file_path: &Path) -> Result<Self, String> {
        // Load .sprite JSON
        // Create SpriteSheet with frames from sprite definitions
    }
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Metadata loading preserves sprite definitions
*For any* valid .sprite metadata file, loading it should result in a sprite list that matches the file contents exactly
**Validates: Requirements 1.4**

### Property 2: Rectangle creation produces valid sprites
*For any* click-drag operation on the texture canvas, the created sprite rectangle should have positive width and height
**Validates: Requirements 2.1, 2.2**

### Property 3: Handle dragging maintains rectangle validity
*For any* corner handle drag operation, the resulting sprite rectangle should remain within texture bounds and have positive dimensions
**Validates: Requirements 2.3**

### Property 4: Center dragging preserves sprite dimensions
*For any* center drag operation, the sprite's width and height should remain unchanged
**Validates: Requirements 2.4**

### Property 5: Delete removes sprite from list
*For any* selected sprite, pressing Delete should result in that sprite no longer appearing in the sprite list
**Validates: Requirements 2.5**

### Property 6: Default naming is sequential
*For any* newly created sprite, its default name should follow the pattern "sprite_N" where N is unique
**Validates: Requirements 3.1**

### Property 7: Selection displays properties
*For any* sprite selection, the properties panel should display that sprite's name, x, y, width, and height
**Validates: Requirements 3.2**

### Property 8: Name editing updates sprite
*For any* name edit in the properties panel, the corresponding sprite's name should be updated
**Validates: Requirements 3.3**

### Property 9: Duplicate names are rejected
*For any* attempt to set a sprite name to an existing name, the system should reject it and show a warning
**Validates: Requirements 3.4**

### Property 10: Labels display sprite names
*For any* sprite with a name, that name should be rendered as a label on the canvas
**Validates: Requirements 3.5**

### Property 11: Grid slicing calculates correct dimensions
*For any* grid dimensions (columns, rows), the calculated sprite width should equal texture_width / columns and height should equal texture_height / rows (accounting for padding/spacing)
**Validates: Requirements 4.2**

### Property 12: Grid slicing creates correct count
*For any* grid dimensions (columns, rows), the number of created sprites should equal columns × rows
**Validates: Requirements 4.3**

### Property 13: Auto-slice naming is sequential
*For any* auto-slice operation, sprites should be named sprite_0, sprite_1, ..., sprite_N in order
**Validates: Requirements 4.4**

### Property 14: Padding affects sprite positions
*For any* padding value P, sprites should start at position (P, P) and be separated by P pixels
**Validates: Requirements 4.5**

### Property 15: Save includes all sprite data
*For any* sprite in the editor, saving should write its name, x, y, width, and height to the JSON file
**Validates: Requirements 5.2**

### Property 16: Save includes texture path
*For any* save operation, the output JSON should contain the source texture path
**Validates: Requirements 5.3**

### Property 17: Save creates backup of existing file
*For any* existing .sprite file, saving should create a backup file before overwriting
**Validates: Requirements 5.4**

### Property 18: Selection highlights sprite
*For any* selected sprite, it should be rendered with a distinct highlight color different from unselected sprites
**Validates: Requirements 6.1**

### Property 19: Selection shows preview
*For any* selected sprite, a preview image showing only that sprite's region should be displayed
**Validates: Requirements 6.2**

### Property 20: Hover shows tooltip
*For any* sprite rectangle being hovered, a tooltip containing the sprite's name should be displayed
**Validates: Requirements 6.3**

### Property 21: Preview displays dimensions
*For any* sprite preview, the width and height in pixels should be displayed
**Validates: Requirements 6.5**

### Property 22: Sprite files list in asset browser
*For any* .sprite file in the project, individual sprites should appear as sub-items in the asset browser
**Validates: Requirements 7.1**

### Property 23: Drag-drop creates entity with sprite
*For any* sprite dragged onto the scene, an entity should be created with a SpriteSheet component referencing that sprite
**Validates: Requirements 7.2**

### Property 24: Entity renders only sprite region
*For any* entity with a sprite, rendering should display only the pixels within that sprite's rectangle from the sheet
**Validates: Requirements 7.3**

### Property 25: Inspector shows sprite info
*For any* entity with a sprite, the inspector should display the sprite name and source texture path
**Validates: Requirements 7.4**

### Property 26: Sprite changes update entities
*For any* sprite definition change, all entities using that sprite should reflect the updated region
**Validates: Requirements 7.5**

### Property 27: Undo reverses last action
*For any* undoable action, pressing Ctrl+Z should restore the state before that action
**Validates: Requirements 8.3**

### Property 28: Redo restores undone action
*For any* undone action, pressing Ctrl+Y should restore the state after that action
**Validates: Requirements 8.4**

### Property 29: Sprite count is accurate
*For any* set of sprites, the displayed count should equal the number of sprites in the list
**Validates: Requirements 9.2**

### Property 30: Coverage calculation is correct
*For any* set of sprites, the coverage percentage should equal (sum of sprite areas) / (texture area) × 100
**Validates: Requirements 9.3**

### Property 31: Overlapping sprites show warning
*For any* pair of sprites with overlapping rectangles, a warning should be displayed
**Validates: Requirements 9.4**

### Property 32: Out-of-bounds sprites show error
*For any* sprite with x+width > texture_width or y+height > texture_height, an error should be displayed
**Validates: Requirements 9.5**

### Property 33: Export includes all metadata
*For any* export operation, the output file should contain all sprite names, positions, and dimensions
**Validates: Requirements 10.3**

## Error Handling

### File Operations
- **Missing texture file**: Show error dialog with file path
- **Invalid .sprite JSON**: Show parse error with line number
- **Write permission denied**: Show error and suggest checking file permissions
- **Backup creation failed**: Warn user but allow save to proceed

### User Input Validation
- **Duplicate sprite names**: Show inline error, prevent save
- **Empty sprite name**: Show warning, use default name
- **Invalid grid dimensions**: Show error, disable auto-slice button
- **Negative coordinates**: Clamp to 0
- **Sprites outside bounds**: Show error indicator, allow but warn

### Runtime Errors
- **Texture loading failed**: Show placeholder texture with error message
- **Out of memory**: Show error, suggest reducing texture size
- **Undo stack overflow**: Limit stack to 50 actions, remove oldest

## Testing Strategy

### Unit Tests
- Test `SpriteDefinition` serialization/deserialization
- Test `AutoSlicer` grid calculations with various dimensions
- Test sprite name validation (duplicates, empty names)
- Test coordinate clamping and bounds checking
- Test coverage percentage calculation
- Test overlap detection algorithm

### Property-Based Tests
- Use `quickcheck` or `proptest` for Rust
- Generate random sprite definitions and verify:
  - Serialization round-trip (save then load produces same data)
  - Grid slicing produces correct count and dimensions
  - Coverage calculation is always between 0-100%
  - Undo/redo maintains state consistency
  - Sprite rectangles never have negative dimensions

### Integration Tests
- Test opening sprite editor from asset browser
- Test saving and loading .sprite files
- Test drag-drop sprite to scene creates entity
- Test entity rendering with sprite regions
- Test keyboard shortcuts (Ctrl+S, Delete, Ctrl+Z, Ctrl+Y)

### Manual Testing
- Test UI responsiveness with large sprite sheets (2048x2048)
- Test zoom and pan controls feel smooth
- Test selection and handle dragging is intuitive
- Test tooltip display timing
- Test error messages are clear and helpful

## Performance Considerations

### Texture Loading
- Load textures asynchronously to avoid blocking UI
- Cache loaded textures in TextureManager
- Use mipmaps for zoomed-out views

### Rendering
- Only render visible sprites (viewport culling)
- Batch sprite rectangle rendering
- Update canvas only when state changes (not every frame)

### Memory
- Limit undo stack to 50 actions
- Release texture when editor closes
- Use texture compression for large sprite sheets

### File I/O
- Save asynchronously to avoid UI freeze
- Debounce auto-save (wait 2 seconds after last edit)
- Use incremental backup (keep last 3 versions)

## UI/UX Design

### Layout
- **Left Panel (200px)**: Sprite list with thumbnails
- **Center Panel**: Canvas with texture and sprite rectangles
- **Right Panel (300px)**: Properties and preview
- **Top Toolbar**: Save, Auto Slice, Export, Undo, Redo buttons
- **Bottom Status Bar**: Sprite count, coverage %, zoom level

### Visual Design
- **Selected sprite**: Yellow border (2px)
- **Hovered sprite**: White border (1px)
- **Unselected sprite**: Semi-transparent blue border (1px)
- **Resize handles**: Small squares at corners (8x8px)
- **Grid lines**: Dotted gray lines when auto-slicing

### Interactions
- **Click**: Select sprite
- **Click + Drag**: Create new sprite rectangle
- **Drag handle**: Resize sprite
- **Drag center**: Move sprite
- **Double-click**: Rename sprite (inline edit)
- **Right-click**: Context menu (Delete, Duplicate, Rename)
- **Mouse wheel**: Zoom in/out
- **Middle mouse drag**: Pan canvas
- **Ctrl+Click**: Multi-select (future feature)

## Integration Points

### Asset Browser
- Add "Open in Sprite Editor" to PNG file context menu
- Display .sprite files with sprite icon
- Show sprite count badge on .sprite files
- Allow expanding .sprite files to show individual sprites

### Scene Editor
- Support drag-drop sprites from asset browser to scene
- Create entity with SpriteSheet and AnimatedSprite components
- Set sprite frame index to match dropped sprite

### Inspector
- Show sprite name and source texture in SpriteSheet component
- Add "Edit Sprite Sheet" button that opens sprite editor
- Display sprite preview thumbnail

### Texture Manager
- Share texture cache between sprite editor and runtime
- Support loading textures from project-relative paths
- Handle texture reloading when files change

## Future Enhancements

### Phase 2 Features
- **9-Slice support**: Define border regions for UI scaling
- **Pivot point editing**: Set sprite origin point
- **Animation sequence editor**: Group sprites into animations
- **Automatic sprite detection**: Detect sprites by transparency
- **Sprite packing**: Optimize texture atlas layout
- **Multi-texture support**: Edit multiple sheets in tabs

### Phase 3 Features
- **Collaborative editing**: Multiple users edit same sprite sheet
- **Version control integration**: Git diff for .sprite files
- **Import from external tools**: TexturePacker, Aseprite
- **Sprite optimization**: Remove duplicate sprites, trim transparency
- **Batch processing**: Apply operations to multiple sprite sheets

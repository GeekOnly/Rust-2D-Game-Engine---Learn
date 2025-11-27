# Design Document: Unity-Like Scene View

## Overview

This design document outlines the architecture and implementation strategy for upgrading the Scene View system to match Unity Editor's quality and user experience. The system builds upon the existing `SceneCamera` and `SceneGrid` components while adding enhanced rendering, improved camera controls, and professional-grade visual feedback.

The core philosophy is to provide an intuitive, responsive editing experience that feels natural to developers familiar with Unity while maintaining the flexibility to work with both 2D and 3D content seamlessly.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Scene View UI                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Toolbar    │  │ Scene Gizmo  │  │  Grid Overlay│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼────────┐ ┌──────▼──────┐ ┌───────▼────────┐
│ Camera Control │ │   Renderer  │ │  Grid System   │
│   - Pan        │ │  - 2D/3D    │ │  - Adaptive    │
│   - Orbit      │ │  - Depth    │ │  - Perspective │
│   - Zoom       │ │  - Culling  │ │  - Fading      │
└────────────────┘ └─────────────┘ └────────────────┘
```

### Component Responsibilities

1. **Scene View UI** (`scene_view.rs`)
   - Main rendering loop and event handling
   - Toolbar management (2D/3D toggle, transform tools)
   - Coordinate transformation between screen and world space
   - Entity selection and interaction

2. **Camera Control** (`camera.rs`)
   - Camera state management (position, rotation, zoom)
   - Input handling for navigation (pan, orbit, zoom)
   - Smooth interpolation and constraints
   - View matrix calculations

3. **Grid System** (`grid.rs`)
   - Grid line generation and rendering
   - Adaptive grid spacing based on zoom
   - Perspective-correct 3D grid rendering
   - Distance-based fading

4. **Renderer**
   - Entity rendering with proper depth sorting
   - 3D mesh rendering with back-face culling
   - Gizmo and overlay rendering
   - Scene gizmo visualization

## Components and Interfaces

### Enhanced SceneCamera

```rust
pub struct SceneCamera {
    // Position and orientation
    pub position: Vec2,        // World position (XZ plane in 3D)
    pub zoom: f32,             // Zoom level (pixels per world unit)
    pub rotation: f32,         // Yaw angle in degrees
    pub pitch: f32,            // Pitch angle in degrees
    
    // Orbit mode
    pub pivot: Vec2,           // Pivot point for orbit
    pub distance: f32,         // Distance from pivot
    
    // Constraints
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    
    // State tracking
    is_panning: bool,
    is_rotating: bool,
    is_orbiting: bool,
    last_mouse_pos: Vec2,
    
    // Settings
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub pan_speed: f32,
}

impl SceneCamera {
    // Navigation methods
    pub fn start_pan(&mut self, mouse_pos: Vec2);
    pub fn update_pan(&mut self, mouse_pos: Vec2);
    pub fn stop_pan(&mut self);
    
    pub fn start_orbit(&mut self, mouse_pos: Vec2, pivot: Vec2);
    pub fn update_orbit(&mut self, mouse_pos: Vec2);
    pub fn stop_orbit(&mut self);
    
    pub fn start_rotate(&mut self, mouse_pos: Vec2);
    pub fn update_rotate(&mut self, mouse_pos: Vec2);
    pub fn stop_rotate(&mut self);
    
    pub fn zoom(&mut self, delta: f32, mouse_pos: Vec2);
    pub fn focus_on(&mut self, target: Vec2, size: f32);
    
    // Coordinate transformation
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2;
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2;
    
    // View matrix for 3D rendering
    pub fn get_view_matrix(&self) -> Mat4;
    pub fn get_projection_matrix(&self, aspect: f32, mode: ProjectionMode) -> Mat4;
}
```

### Enhanced SceneGrid

```rust
pub struct SceneGrid {
    pub enabled: bool,
    pub size: f32,              // Base grid cell size
    pub snap_enabled: bool,
    
    // Visual settings
    pub color: [f32; 4],
    pub axis_color_x: [f32; 4], // Red for X axis
    pub axis_color_z: [f32; 4], // Blue for Z axis
    pub fade_distance: f32,      // Distance at which grid starts fading
    pub fade_range: f32,         // Range over which fade occurs
    
    // Adaptive grid
    pub subdivision_levels: Vec<f32>, // [1.0, 0.1, 0.01] for multi-scale grid
    pub min_line_spacing: f32,        // Minimum pixels between lines
}

impl SceneGrid {
    pub fn new() -> Self;
    
    // Grid rendering
    pub fn render_2d(&self, painter: &Painter, rect: Rect, camera: &SceneCamera);
    pub fn render_3d(&self, painter: &Painter, rect: Rect, camera: &SceneCamera);
    
    // Adaptive grid calculation
    pub fn calculate_visible_range(&self, camera: &SceneCamera, viewport: Rect) -> (f32, f32);
    pub fn select_grid_level(&self, camera: &SceneCamera) -> f32;
    
    // Utility
    pub fn snap(&self, position: Vec2) -> Vec2;
    pub fn calculate_fade_alpha(&self, distance: f32) -> f32;
}
```

### Scene View State

```rust
pub struct SceneViewState {
    pub mode: SceneViewMode,
    pub projection: ProjectionMode,
    pub transform_space: TransformSpace,
    pub current_tool: TransformTool,
    
    // Selection
    pub selected_entity: Option<Entity>,
    pub hovered_entity: Option<Entity>,
    
    // Gizmo interaction
    pub dragging_entity: Option<Entity>,
    pub drag_axis: Option<u8>,
    pub drag_start_pos: Vec2,
    
    // Visual settings
    pub show_grid: bool,
    pub show_gizmo: bool,
    pub show_colliders: bool,
    pub show_velocities: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneViewMode {
    Mode2D,
    Mode3D,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionMode {
    Perspective,
    Isometric,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformSpace {
    Local,
    World,
}
```

## Data Models

### 3D Point and Projection

```rust
#[derive(Clone, Copy, Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self;
    
    // Rotation methods
    pub fn rotate_x(&self, angle: f32) -> Self;
    pub fn rotate_y(&self, angle: f32) -> Self;
    pub fn rotate_z(&self, angle: f32) -> Self;
    pub fn rotate(&self, rotation: &[f32; 3]) -> Self;
    
    // Projection methods
    pub fn project_perspective(&self, fov: f32, distance: f32) -> (f32, f32);
    pub fn project_isometric(&self) -> (f32, f32);
}
```

### Grid Line Data

```rust
pub struct GridLine {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Color32,
    pub width: f32,
    pub is_axis: bool,
}

pub struct GridRenderData {
    pub lines: Vec<GridLine>,
    pub visible_range: (f32, f32),
    pub grid_level: f32,
}
```

## Correctn
ess Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Camera Control Properties

Property 1: Pan updates camera position
*For any* initial camera state and mouse drag input, panning should update the camera position proportionally to the mouse movement and inversely proportional to the zoom level
**Validates: Requirements 1.1, 1.4, 5.4**

Property 2: Orbit maintains pivot distance
*For any* pivot point and sequence of mouse movements during orbit, the distance between the camera and the pivot point should remain constant throughout the orbit operation
**Validates: Requirements 1.2, 5.3**

Property 3: Free-look rotation updates camera orientation
*For any* mouse drag input in free-look mode, the camera's yaw and pitch should update proportionally to the horizontal and vertical mouse movement respectively
**Validates: Requirements 1.3**

Property 4: Zoom scales view toward cursor
*For any* scroll delta and cursor position, zooming should scale the view and adjust the camera position such that the world point under the cursor remains stationary in screen space
**Validates: Requirements 1.5**

Property 5: Zoom interpolation is smooth
*For any* sequence of zoom operations, the zoom factor should change gradually without abrupt jumps, maintaining a consistent rate of change
**Validates: Requirements 5.2**

Property 6: Focus frames entity appropriately
*For any* entity with position and size, the focus operation should move the camera to a position where the entity is centered and fully visible within the viewport
**Validates: Requirements 1.6**

### Grid System Properties

Property 7: 2D grid lines are orthogonal
*For any* camera position in 2D mode, all generated grid lines should be either perfectly horizontal or perfectly vertical in world space, aligned to the X and Y axes
**Validates: Requirements 2.1**

Property 8: 3D grid has correct perspective
*For any* camera position and orientation in 3D mode, grid lines on the ground plane should be projected with mathematically correct perspective transformation
**Validates: Requirements 2.2**

Property 9: Grid fades with distance
*For any* grid line at distance D from the camera, the alpha value should decrease monotonically as D increases beyond the fade distance threshold
**Validates: Requirements 2.3**

Property 10: Adaptive grid maintains visual density
*For any* zoom level change, the grid spacing should adjust such that the screen-space distance between adjacent grid lines remains within a target range (e.g., 20-100 pixels)
**Validates: Requirements 2.5, 7.1**

Property 11: Grid subdivisions adapt to zoom
*For any* zoom level, when zoomed in close, finer grid subdivisions should appear, and when zoomed out far, only major grid lines should be visible
**Validates: Requirements 7.2, 7.3**

### Mode Switching Properties

Property 12: Mode switching preserves camera state
*For any* camera position and zoom level, switching between 2D and 3D modes should preserve these values
**Validates: Requirements 3.3**

Property 13: 3D mode restores or initializes orientation
*For any* previous 3D camera orientation, switching to 3D mode should either restore that orientation or initialize to a default isometric view if no previous state exists
**Validates: Requirements 3.4**

### 3D Rendering Properties

Property 14: Perspective projection scales with depth
*For any* 3D point with depth Z, the screen-space size of an object at that point should be inversely proportional to Z (objects farther away appear smaller)
**Validates: Requirements 4.1**

Property 15: Back-face culling hides non-visible faces
*For any* mesh face with normal vector N and camera view direction V, if the dot product N·V > 0, the face should not be rendered
**Validates: Requirements 4.2**

Property 16: Faces are depth-sorted
*For any* set of mesh faces, they should be rendered in order from farthest to nearest based on their average Z depth in camera space
**Validates: Requirements 4.3**

Property 17: Gizmo reflects camera orientation
*For any* camera rotation (yaw, pitch), the scene gizmo axes should be rotated to match the camera's orientation, providing accurate visual feedback
**Validates: Requirements 6.2**

### Depth Sorting Properties

Property 18: Entities render in depth order
*For any* set of entities with different Z positions, they should be rendered in back-to-front order such that entities with smaller Z values appear in front of those with larger Z values
**Validates: Requirements 8.1**

Property 19: Transparent objects are sorted correctly
*For any* set of entities with transparency (alpha < 1.0), they should be depth-sorted and rendered back-to-front to ensure correct alpha blending
**Validates: Requirements 8.2**

## Error Handling

### Camera Control Errors

1. **Invalid Zoom Levels**
   - Clamp zoom to [min_zoom, max_zoom] range
   - Handle division by zero in zoom calculations
   - Prevent negative zoom values

2. **Invalid Rotation Angles**
   - Clamp pitch to [-89°, 89°] to prevent gimbal lock
   - Normalize yaw to [0°, 360°] range
   - Handle NaN values from trigonometric functions

3. **Invalid Mouse Input**
   - Validate mouse positions are within viewport bounds
   - Handle missing or invalid pointer events
   - Prevent state corruption from rapid input changes

### Grid Rendering Errors

1. **Invalid Grid Parameters**
   - Ensure grid size is positive and non-zero
   - Clamp fade distances to reasonable ranges
   - Handle edge cases where grid spacing becomes too small or large

2. **Projection Errors**
   - Handle degenerate cases in perspective projection (Z near zero)
   - Validate transformation matrices are non-singular
   - Prevent overflow in distance calculations

### 3D Rendering Errors

1. **Mesh Data Errors**
   - Validate mesh has at least 3 vertices
   - Handle degenerate triangles (zero area)
   - Ensure face indices are within vertex array bounds

2. **Depth Sorting Errors**
   - Handle entities with identical Z values
   - Prevent infinite loops in sorting algorithms
   - Handle NaN or infinite depth values

## Testing Strategy

### Unit Testing

Unit tests will cover:

1. **Camera Mathematics**
   - Coordinate transformations (world-to-screen, screen-to-world)
   - Rotation matrix calculations
   - Zoom factor calculations
   - Focus framing calculations

2. **Grid Calculations**
   - Grid line generation for various zoom levels
   - Fade alpha calculations
   - Adaptive spacing selection
   - Perspective projection of grid points

3. **3D Projection**
   - Perspective projection correctness
   - Isometric projection correctness
   - Back-face culling logic
   - Depth sorting algorithms

4. **State Management**
   - Mode switching state preservation
   - Camera state transitions
   - Input state tracking

### Property-Based Testing

Property-based tests will use the **proptest** crate for Rust. Each test will run a minimum of 100 iterations with randomly generated inputs.

**Test Configuration:**
```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn test_property_name(/* generated inputs */) {
        // Test implementation
    }
}
```

**Property Test Requirements:**
- Each property-based test MUST be tagged with a comment referencing the design document property
- Tag format: `// Feature: unity-scene-view, Property {number}: {property_text}`
- Each correctness property MUST be implemented by a SINGLE property-based test
- Tests MUST use appropriate generators for input types (positions, angles, zoom levels, etc.)

**Example Property Test:**
```rust
// Feature: unity-scene-view, Property 1: Pan updates camera position
#[test]
fn prop_pan_updates_camera_position(
    initial_pos in prop_vec2(),
    initial_zoom in 5.0f32..200.0f32,
    mouse_delta in prop_vec2(),
) {
    let mut camera = SceneCamera::new();
    camera.position = initial_pos;
    camera.zoom = initial_zoom;
    
    let start_pos = Vec2::new(0.0, 0.0);
    let end_pos = start_pos + mouse_delta;
    
    camera.start_pan(start_pos);
    camera.update_pan(end_pos);
    
    // Verify camera moved proportionally
    let expected_delta = mouse_delta / initial_zoom;
    assert!((camera.position - (initial_pos + expected_delta)).length() < 0.01);
}
```

### Integration Testing

Integration tests will verify:

1. **End-to-End Camera Control**
   - Complete pan, orbit, and zoom workflows
   - Mode switching with state preservation
   - Multiple simultaneous camera operations

2. **Rendering Pipeline**
   - Complete scene rendering with entities, grid, and gizmos
   - Depth sorting with mixed entity types
   - UI overlay rendering order

3. **User Interaction Flows**
   - Toolbar interactions
   - Gizmo axis clicking
   - Entity selection and manipulation

### Visual Testing

Manual visual testing will verify:

1. **Grid Appearance**
   - Grid looks professional and clean
   - Fading is smooth and natural
   - Axis colors are distinct and visible

2. **Camera Feel**
   - Navigation feels smooth and responsive
   - Zoom centers on cursor correctly
   - Orbit maintains proper distance

3. **3D Rendering Quality**
   - Perspective looks correct
   - Back-face culling works properly
   - Depth sorting has no artifacts

## Implementation Notes

### Performance Considerations

1. **Grid Rendering Optimization**
   - Cull grid lines outside viewport
   - Use line batching to reduce draw calls
   - Cache grid geometry when camera is static

2. **Depth Sorting Optimization**
   - Only sort when camera moves or entities change
   - Use spatial partitioning for large scenes
   - Consider Z-buffer for hardware-accelerated sorting

3. **Camera Update Optimization**
   - Only recalculate matrices when camera changes
   - Use dirty flags to track state changes
   - Batch multiple input events per frame

### Coordinate System Conventions

- **World Space**: Right-handed coordinate system (X right, Y up, Z forward)
- **Screen Space**: Origin at top-left, Y down
- **Camera Space**: Z-axis points away from camera (into the scene)
- **2D Mode**: Uses XY plane, Z is ignored
- **3D Mode**: Camera position.y maps to world Z-axis for ground plane

### Rendering Order

1. Background
2. Grid (if enabled)
3. Entities (back-to-front sorted)
4. Collider gizmos (if enabled)
5. Velocity gizmos (if enabled)
6. Transform gizmos (for selected entity)
7. Scene gizmo (top-right corner)
8. Toolbar and UI overlays


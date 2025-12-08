# Design Document: Scene View Improvements

## Overview

This design document outlines the technical approach for upgrading the 3D Scene View from its current 1.5/10 rating to Unity-level quality. The focus is on two critical areas: professional infinite grid rendering and smooth, responsive camera controls.

The current implementation suffers from:
- Grid that looks like an oval/ellipse instead of extending to infinity
- Jerky, unresponsive camera controls
- Poor visual feedback
- Grid that fades too quickly

This upgrade will transform the scene view into a professional tool that developers trust and enjoy using.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Scene View UI                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Toolbar    │  │ Scene Gizmo  │  │  Info Overlay│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼────────┐ ┌──────▼──────┐ ┌───────▼────────┐
│ Camera System  │ │   Renderer  │ │  Grid System   │
│  - Damping     │ │  - Batching │ │  - Infinite    │
│  - Inertia     │ │  - Caching  │ │  - Adaptive    │
│  - Sensitivity │ │  - Culling  │ │  - Smooth Fade │
└────────────────┘ └─────────────┘ └────────────────┘
```

### Component Responsibilities

1. **Enhanced Camera System**
   - Smooth damping for all movements
   - Configurable sensitivity settings
   - Inertia and momentum
   - Cursor-based zoom with proper world-space tracking

2. **Infinite Grid System**
   - Shader-based or geometry-based infinite grid
   - Multi-level adaptive grid with smooth transitions
   - Proper perspective projection
   - Efficient rendering with batching

3. **Visual Feedback**
   - Real-time camera state display
   - Grid unit size indicator
   - Smooth transitions and animations

## Components and Interfaces

### Enhanced Camera System

```rust
pub struct CameraSettings {
    // Sensitivity settings
    pub pan_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    
    // Damping settings (0.0 = no damping, 1.0 = maximum damping)
    pub pan_damping: f32,
    pub rotation_damping: f32,
    pub zoom_damping: f32,
    
    // Inertia settings
    pub enable_inertia: bool,
    pub inertia_decay: f32,  // How quickly momentum decays
    
    // Zoom settings
    pub zoom_to_cursor: bool,
    pub zoom_speed: f32,
}

pub struct CameraVelocity {
    pub pan_velocity: Vec2,
    pub rotation_velocity: Vec2,  // (yaw_velocity, pitch_velocity)
    pub zoom_velocity: f32,
}

pub struct SceneCamera {
    // ... existing fields ...
    
    // New fields for smooth controls
    pub settings: CameraSettings,
    velocity: CameraVelocity,
    
    // Target values for smooth interpolation
    target_position: Vec2,
    target_rotation: f32,
    target_pitch: f32,
    target_zoom: f32,
    
    // Cursor tracking for zoom
    last_cursor_world_pos: Option<Vec2>,
}

impl SceneCamera {
    /// Update camera with delta time for smooth interpolation
    pub fn update(&mut self, delta_time: f32);
    
    /// Apply damping to velocity
    fn apply_damping(&mut self, delta_time: f32);
    
    /// Apply inertia when input stops
    fn apply_inertia(&mut self, delta_time: f32);
    
    /// Smooth interpolation toward target values
    fn interpolate_to_targets(&mut self, delta_time: f32);
    
    /// Enhanced zoom with cursor tracking
    pub fn zoom_to_cursor(&mut self, delta: f32, cursor_screen_pos: Vec2, viewport_center: Vec2);
    
    /// Load/save settings
    pub fn load_settings(&mut self) -> Result<(), Error>;
    pub fn save_settings(&self) -> Result<(), Error>;
    pub fn reset_settings_to_default(&mut self);
}
```

### Infinite Grid System

```rust
pub struct InfiniteGrid {
    pub enabled: bool,
    
    // Grid appearance
    pub base_unit: f32,  // Base grid unit (e.g., 1.0 meter)
    pub major_line_every: u32,  // Major line every N units (e.g., 10)
    
    // Colors
    pub minor_line_color: [f32; 4],
    pub major_line_color: [f32; 4],
    pub x_axis_color: [f32; 4],
    pub z_axis_color: [f32; 4],
    
    // Line widths
    pub minor_line_width: f32,
    pub major_line_width: f32,
    pub axis_line_width: f32,
    
    // Fade settings
    pub fade_start_distance: f32,  // Distance where fade begins
    pub fade_end_distance: f32,    // Distance where grid disappears
    pub near_fade_start: f32,      // Distance where near fade begins
    pub near_fade_end: f32,        // Distance where near fade completes
    
    // Adaptive grid
    pub min_pixel_spacing: f32,    // Minimum pixels between lines
    pub max_pixel_spacing: f32,    // Maximum pixels between lines
    pub level_transition_range: f32, // Range for smooth level transitions
    
    // Performance
    cached_geometry: Option<GridGeometry>,
    last_camera_state: Option<CameraState>,
}

pub struct GridGeometry {
    pub lines: Vec<GridLine>,
    pub generation_time: std::time::Instant,
}

pub struct GridLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 4],
    pub width: f32,
    pub line_type: GridLineType,
}

pub enum GridLineType {
    Minor,
    Major,
    XAxis,
    ZAxis,
}

impl InfiniteGrid {
    pub fn new() -> Self;
    
    /// Generate grid geometry for current camera view
    pub fn generate_geometry(
        &mut self,
        camera: &SceneCamera,
        viewport_size: Vec2,
    ) -> &GridGeometry;
    
    /// Calculate appropriate grid level for current zoom
    fn calculate_grid_level(&self, camera: &SceneCamera) -> f32;
    
    /// Calculate fade alpha for a point based on distance
    fn calculate_fade_alpha(&self, point: Vec3, camera_pos: Vec3) -> f32;
    
    /// Check if grid geometry needs regeneration
    fn needs_regeneration(&self, camera: &SceneCamera) -> bool;
    
    /// Render grid using batched lines
    pub fn render(
        &self,
        painter: &egui::Painter,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
    );
    
    /// Project 3D grid point to screen space
    fn project_to_screen(
        &self,
        point: Vec3,
        camera: &SceneCamera,
        viewport_center: Vec2,
    ) -> Vec2;
}
```

### Camera State Display

```rust
pub struct CameraStateDisplay {
    pub show_distance: bool,
    pub show_angles: bool,
    pub show_grid_size: bool,
    pub show_fps: bool,
}

impl CameraStateDisplay {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        camera: &SceneCamera,
        grid: &InfiniteGrid,
        fps: f32,
    );
}
```

### Snapping System

```rust
pub struct SnapSettings {
    pub enabled: bool,
    pub mode: SnapMode,
    
    // Snap increments
    pub position_snap: f32,    // Grid size for position (e.g., 1.0)
    pub rotation_snap: f32,    // Degrees for rotation (e.g., 15.0)
    pub scale_snap: f32,       // Increment for scale (e.g., 0.1)
    
    // Visual feedback
    pub show_snap_indicators: bool,
    pub snap_indicator_color: [f32; 4],
}

pub enum SnapMode {
    Relative,  // Snap relative to drag start position
    Absolute,  // Snap to absolute grid positions
}

impl SnapSettings {
    pub fn snap_position(&self, value: Vec3, original: Vec3) -> Vec3;
    pub fn snap_rotation(&self, value: f32, original: f32) -> f32;
    pub fn snap_scale(&self, value: Vec3, original: Vec3) -> Vec3;
    
    /// Check if snap key (Ctrl) is pressed
    pub fn is_snap_active(&self, modifiers: &egui::Modifiers) -> bool;
}
```

### Selection System

```rust
pub struct Selection {
    pub entities: Vec<Entity>,
    pub active_entity: Option<Entity>,  // The last selected entity
    
    // Box selection state
    box_selection_start: Option<Vec2>,
    box_selection_current: Option<Vec2>,
}

impl Selection {
    pub fn new() -> Self;
    
    // Single selection
    pub fn select(&mut self, entity: Entity);
    pub fn deselect(&mut self, entity: Entity);
    pub fn toggle(&mut self, entity: Entity);
    pub fn clear(&mut self);
    
    // Multi-selection
    pub fn add_to_selection(&mut self, entity: Entity);
    pub fn remove_from_selection(&mut self, entity: Entity);
    pub fn select_all(&mut self, world: &World);
    
    // Box selection
    pub fn start_box_selection(&mut self, screen_pos: Vec2);
    pub fn update_box_selection(&mut self, screen_pos: Vec2);
    pub fn finish_box_selection(&mut self, world: &World, camera: &SceneCamera) -> Vec<Entity>;
    pub fn cancel_box_selection(&mut self);
    
    // Queries
    pub fn is_selected(&self, entity: Entity) -> bool;
    pub fn count(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get_bounds(&self, world: &World) -> Option<Bounds>;
    pub fn get_center(&self, world: &World) -> Option<Vec3>;
    
    // Rendering
    pub fn render_box_selection(&self, painter: &egui::Painter);
}
```

### Enhanced Gizmo System

```rust
pub struct GizmoSettings {
    pub size: f32,              // Base size in pixels
    pub hover_scale: f32,       // Scale multiplier on hover (e.g., 1.2)
    pub selected_color: [f32; 4],
    pub hover_color: [f32; 4],
    
    // Visibility toggles
    pub show_move_gizmo: bool,
    pub show_rotate_gizmo: bool,
    pub show_scale_gizmo: bool,
    pub show_planar_handles: bool,
}

pub enum GizmoHandle {
    // Move handles
    MoveX,
    MoveY,
    MoveZ,
    MovePlaneXY,
    MovePlaneXZ,
    MovePlaneYZ,
    MoveCenter,
    
    // Rotate handles
    RotateX,
    RotateY,
    RotateZ,
    RotateScreen,
    
    // Scale handles
    ScaleX,
    ScaleY,
    ScaleZ,
    ScaleUniform,
}

pub struct EnhancedGizmo {
    pub settings: GizmoSettings,
    pub hovered_handle: Option<GizmoHandle>,
    pub active_handle: Option<GizmoHandle>,
    
    // Interaction state
    drag_start_pos: Option<Vec3>,
    drag_start_value: Option<Vec3>,
}

impl EnhancedGizmo {
    pub fn render(
        &mut self,
        painter: &egui::Painter,
        entity: Entity,
        world: &World,
        camera: &SceneCamera,
        tool: TransformTool,
        space: TransformSpace,
    );
    
    pub fn handle_input(
        &mut self,
        response: &egui::Response,
        entity: Entity,
        world: &mut World,
        camera: &SceneCamera,
        snap_settings: &SnapSettings,
    ) -> bool;  // Returns true if transform was modified
    
    /// Calculate screen-constant size for gizmo
    fn calculate_gizmo_scale(&self, camera: &SceneCamera, world_pos: Vec3) -> f32;
    
    /// Check if mouse is hovering over a handle
    fn check_hover(&mut self, mouse_pos: Vec2, gizmo_pos: Vec2, camera: &SceneCamera);
    
    /// Render planar movement handles (colored squares)
    fn render_planar_handles(&self, painter: &egui::Painter, center: Vec2, scale: f32);
}
```

### 2.5D Support

```rust
pub struct Scene25DSettings {
    pub enabled: bool,
    pub orthographic_size: f32,
    pub show_z_depth_indicators: bool,
    pub z_depth_indicator_color: [f32; 4],
}

pub struct SpriteDepthInfo {
    pub entity: Entity,
    pub z_position: f32,
    pub render_order: i32,
}

impl Scene25DSettings {
    /// Sort sprites by Z-depth for rendering
    pub fn sort_sprites_by_depth(&self, sprites: &mut Vec<SpriteDepthInfo>);
    
    /// Render Z-depth indicator for selected entity
    pub fn render_z_depth_indicator(
        &self,
        painter: &egui::Painter,
        entity: Entity,
        world: &World,
        camera: &SceneCamera,
    );
}
```

### Flythrough Camera Mode

```rust
pub struct FlythroughMode {
    pub active: bool,
    pub move_speed: f32,
    pub look_sensitivity: f32,
    pub smooth_movement: bool,
    
    // Movement state
    forward_pressed: bool,
    backward_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    
    // Look state
    last_mouse_pos: Option<Vec2>,
}

impl FlythroughMode {
    pub fn new() -> Self;
    
    pub fn activate(&mut self);
    pub fn deactivate(&mut self);
    
    pub fn update(
        &mut self,
        camera: &mut SceneCamera,
        ui: &egui::Ui,
        delta_time: f32,
    );
    
    /// Handle WASD movement
    fn handle_movement(&mut self, camera: &mut SceneCamera, delta_time: f32);
    
    /// Handle mouse look
    fn handle_look(&mut self, camera: &mut SceneCamera, mouse_delta: Vec2);
}
```

### Viewport Statistics

```rust
pub struct ViewportStats {
    pub show_stats: bool,
    pub detailed_view: bool,
    
    // Performance metrics
    pub fps: f32,
    pub frame_time_ms: f32,
    pub entity_count: usize,
    pub visible_entity_count: usize,
    pub draw_call_count: usize,
    
    // Update tracking
    last_update_time: std::time::Instant,
    frame_times: Vec<f32>,  // Rolling window for smoothing
}

impl ViewportStats {
    pub fn new() -> Self;
    
    pub fn update(&mut self, world: &World, camera: &SceneCamera);
    
    pub fn render(&self, ui: &mut egui::Ui, position: egui::Pos2);
    
    pub fn toggle_detailed_view(&mut self);
}
```

### Enhanced Scene Gizmo

```rust
pub struct EnhancedSceneGizmo {
    pub size: f32,
    pub position: SceneGizmoPosition,
    
    // Interaction state
    pub hovered_axis: Option<Axis3D>,
    pub hovered_center: bool,
    
    // Animation state
    transition_progress: f32,
    target_view: Option<CameraView>,
}

pub enum SceneGizmoPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

pub enum Axis3D {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

pub enum CameraView {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
    Perspective,
}

impl EnhancedSceneGizmo {
    pub fn render(
        &mut self,
        painter: &egui::Painter,
        viewport_rect: egui::Rect,
        camera: &SceneCamera,
    );
    
    pub fn handle_input(
        &mut self,
        ui: &egui::Ui,
        viewport_rect: egui::Rect,
        camera: &mut SceneCamera,
    ) -> bool;  // Returns true if view changed
    
    pub fn update(&mut self, camera: &mut SceneCamera, delta_time: f32);
    
    /// Animate camera to target view
    fn animate_to_view(&mut self, camera: &mut SceneCamera, delta_time: f32);
    
    /// Check if mouse is over an axis
    fn check_axis_hover(&mut self, mouse_pos: Vec2, gizmo_center: Vec2) -> Option<Axis3D>;
    
    /// Render axis with label and tooltip
    fn render_axis(
        &self,
        painter: &egui::Painter,
        center: Vec2,
        axis: Axis3D,
        camera: &SceneCamera,
    );
}
```

## Data Models

### Grid Level System

```rust
pub struct GridLevel {
    pub unit_size: f32,      // Size of one grid cell at this level
    pub alpha: f32,          // Current alpha for smooth transitions
    pub is_active: bool,     // Whether this level is currently visible
}

pub struct AdaptiveGridLevels {
    pub levels: Vec<GridLevel>,
    pub current_primary: usize,
    pub transition_progress: f32,
}

impl AdaptiveGridLevels {
    /// Update grid levels based on camera zoom
    pub fn update(&mut self, camera: &SceneCamera, delta_time: f32);
    
    /// Get all active levels with their alphas
    pub fn get_active_levels(&self) -> Vec<(f32, f32)>;  // (unit_size, alpha)
}
```

### Camera State for Caching

```rust
#[derive(Clone, PartialEq)]
pub struct CameraState {
    pub position: Vec2,
    pub rotation: f32,
    pub pitch: f32,
    pub zoom: f32,
}

impl CameraState {
    /// Check if camera has moved significantly
    pub fn has_changed_significantly(&self, other: &CameraState, threshold: f32) -> bool;
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Camera Control Properties

Property 1: Damped pan movement is smooth
*For any* sequence of pan inputs with damping enabled, the camera position should change gradually following exponential smoothing, with position delta decreasing over time when input stops
**Validates: Requirements 2.1, 5.1**

Property 2: Orbit maintains constant distance
*For any* orbit operation around a pivot point, the distance between camera position and pivot point should remain constant throughout the operation (within numerical tolerance)
**Validates: Requirements 2.2, 5.2**

Property 3: Zoom converges to cursor point
*For any* zoom operation with cursor position, the world point under the cursor before zoom should remain under the cursor after zoom (within numerical precision of 0.1 units)
**Validates: Requirements 2.3, 8.1, 8.2, 8.3**

Property 4: Velocity decays exponentially
*For any* camera velocity when input stops, the velocity magnitude should decrease exponentially toward zero, with each frame reducing velocity by a constant factor
**Validates: Requirements 2.5, 5.5**

Property 5: Sensitivity scales linearly
*For any* sensitivity setting S and input delta D, the camera response should be proportional to S × D (doubling sensitivity doubles response)
**Validates: Requirements 3.1, 3.2, 3.3**

Property 6: Inertia maintains momentum
*For any* camera movement with inertia enabled, when input stops, the camera should continue moving in the same direction with decaying velocity
**Validates: Requirements 5.1, 5.3**

### Grid Rendering Properties

Property 7: Grid lines converge with perspective
*For any* two parallel grid lines in world space, their screen-space projections should converge toward a common vanishing point when extended
**Validates: Requirements 1.2, 7.1, 7.2**

Property 8: Grid fade is monotonic with distance
*For any* grid point at distance D from camera, as D increases beyond fade_start_distance, the alpha value should decrease monotonically (never increase)
**Validates: Requirements 1.3**

Property 9: Grid level transitions maintain constant alpha
*For any* transition between grid levels, the sum of alpha values for all visible grid levels should remain constant (approximately 1.0) to avoid visual popping
**Validates: Requirements 6.3, 6.5**

Property 10: Grid spacing maintains visual density
*For any* zoom level, the screen-space distance between adjacent grid lines should remain within the configured min_pixel_spacing and max_pixel_spacing range
**Validates: Requirements 6.1, 6.2, 6.4**

Property 11: Axis lines have full opacity at origin
*For any* camera position and orientation where the world origin (0, 0, 0) is within the viewport, the X and Z axis lines should be rendered with alpha = 1.0
**Validates: Requirements 4.3**

Property 12: Grid extends to horizon
*For any* camera position and orientation, grid lines should extend far enough that they reach the viewport edges or fade to zero before ending
**Validates: Requirements 1.1, 1.4, 1.5**

Property 13: Grid orientation matches camera rotation
*For any* camera yaw angle, the grid should rotate around the Y-axis by the same angle, maintaining proper 3D orientation
**Validates: Requirements 7.4**

### Performance Properties

Property 14: Grid caching reduces regeneration
*For any* sequence of frames where camera state changes by less than threshold, grid geometry should be reused from cache without regeneration
**Validates: Requirements 10.2**

Property 15: Line batching is efficient
*For any* grid with N lines, all lines should be submitted in a single batched draw call or a small constant number of draw calls
**Validates: Requirements 10.1**

### Snapping Properties

Property 16: Grid snapping is consistent
*For any* position value P and grid size G, snapping P to grid should produce a value that is an exact multiple of G (within floating point precision)
**Validates: Requirements 11.1**

Property 17: Snap increments are configurable
*For any* configured snap increment I and transform operation, the resulting value should be quantized to multiples of I
**Validates: Requirements 11.4**

### Selection Properties

Property 18: Box selection is inclusive
*For any* box selection rectangle R and entity with bounds B, if B intersects R, then the entity should be included in the selection
**Validates: Requirements 12.1**

Property 19: Multi-selection preserves order
*For any* sequence of selection operations, the order of selected entities should match the order they were selected
**Validates: Requirements 12.2, 12.3**

Property 20: Select all includes all entities
*For any* scene with N entities, selecting all should result in exactly N entities being selected
**Validates: Requirements 12.5**

### Gizmo Properties

Property 21: Gizmo size is screen-constant
*For any* camera zoom level Z, the screen-space size of gizmo handles should remain constant (within 5% tolerance)
**Validates: Requirements 13.3**

Property 22: Planar handles move in plane
*For any* planar handle drag operation, the resulting position change should have zero component perpendicular to the plane
**Validates: Requirements 13.1**

### 2.5D Properties

Property 23: Z-depth sorting is correct
*For any* two sprites with Z-positions Z1 and Z2 where Z1 < Z2, sprite 1 should be rendered before sprite 2
**Validates: Requirements 14.2**

Property 24: Orthographic projection preserves parallels
*For any* two parallel lines in world space under orthographic projection, their screen-space projections should also be parallel
**Validates: Requirements 14.1**

### Camera Speed Properties

Property 25: Speed modifiers multiply correctly
*For any* base camera speed S and modifier M (Shift=3x, Ctrl=0.3x), the resulting speed should equal S × M
**Validates: Requirements 17.1, 17.2, 17.3**

Property 26: Flythrough movement is view-relative
*For any* flythrough movement in direction D, the camera should move in world space along the direction obtained by rotating D by the camera's current yaw and pitch
**Validates: Requirements 18.2, 18.3, 18.4, 18.5**

### Frame All Properties

Property 27: Frame all includes all entities
*For any* scene with entities at positions P1, P2, ..., Pn, framing all should result in a camera position and zoom where all positions are visible in the viewport
**Validates: Requirements 19.1, 19.2**

### Scene Gizmo Properties

Property 28: Axis click aligns view
*For any* axis click on the scene gizmo, the resulting camera orientation should align the view direction with that axis (within 1 degree)
**Validates: Requirements 20.1**

## Error Handling

### Camera Control Errors

1. **Invalid Sensitivity Values**
   - Clamp sensitivity to reasonable range [0.01, 10.0]
   - Validate on load, use defaults if invalid
   - Prevent division by zero in calculations

2. **Numerical Instability**
   - Check for NaN/Inf in all calculations
   - Clamp velocities to prevent runaway values
   - Use epsilon comparisons for floating point

3. **Cursor Position Errors**
   - Handle missing cursor position gracefully
   - Fall back to viewport center if cursor unavailable
   - Validate cursor is within viewport bounds

### Grid Rendering Errors

1. **Extreme Zoom Levels**
   - Prevent grid spacing from becoming too small (< 0.001)
   - Prevent grid spacing from becoming too large (> 10000)
   - Disable grid rendering if calculations become unstable

2. **Projection Errors**
   - Handle points behind camera (negative Z)
   - Clamp projected coordinates to reasonable range
   - Skip lines that project outside viewport by large margin

3. **Performance Degradation**
   - Limit maximum number of grid lines per frame
   - Implement aggressive culling for distant lines
   - Fall back to simpler rendering if FPS drops below threshold

## Testing Strategy

### Unit Testing

Unit tests will cover:

1. **Camera Mathematics**
   - Damping calculations
   - Velocity decay
   - Smooth interpolation
   - Cursor-to-world coordinate conversion

2. **Grid Calculations**
   - Grid level selection
   - Fade alpha calculations
   - Perspective projection
   - Line culling logic

3. **Settings Persistence**
   - Save/load camera settings
   - Default value restoration
   - Invalid value handling

### Property-Based Testing

Property-based tests will use the **proptest** crate for Rust. Each test will run a minimum of 100 iterations.

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
- Each property-based test MUST be tagged with: `// Feature: scene-view-improvements, Property {number}: {property_text}`
- Each correctness property MUST be implemented by a SINGLE property-based test
- Tests MUST use appropriate generators for camera states, zoom levels, positions, etc.

**Example Property Test:**
```rust
// Feature: scene-view-improvements, Property 3: Zoom converges to cursor point
#[test]
fn prop_zoom_converges_to_cursor(
    initial_zoom in 5.0f32..200.0f32,
    zoom_delta in -5.0f32..5.0f32,
    cursor_x in -500.0f32..500.0f32,
    cursor_y in -500.0f32..500.0f32,
) {
    let mut camera = SceneCamera::new();
    camera.zoom = initial_zoom;
    
    let cursor_screen = Vec2::new(cursor_x, cursor_y);
    let viewport_center = Vec2::ZERO;
    
    // Get world position under cursor before zoom
    let world_before = camera.screen_to_world(cursor_screen - viewport_center);
    
    // Perform zoom
    camera.zoom_to_cursor(zoom_delta, cursor_screen, viewport_center);
    
    // Get world position under cursor after zoom
    let world_after = camera.screen_to_world(cursor_screen - viewport_center);
    
    // World point should remain stationary (within tolerance)
    assert!((world_before - world_after).length() < 0.1);
}
```

### Integration Testing

Integration tests will verify:

1. **Complete Camera Workflows**
   - Pan → Orbit → Zoom sequences
   - Rapid input changes
   - Settings changes during movement

2. **Grid Rendering Pipeline**
   - Grid generation → Projection → Rendering
   - Level transitions during zoom
   - Cache invalidation and regeneration

3. **Performance Benchmarks**
   - Grid rendering time for various camera positions
   - Cache hit rate
   - Frame time consistency

### Visual Testing

Manual visual testing will verify:

1. **Grid Appearance**
   - Grid extends to horizon naturally
   - No visual artifacts or popping
   - Smooth fading at distance
   - Professional color scheme

2. **Camera Feel**
   - Smooth, weighted movement
   - Responsive but not twitchy
   - Natural deceleration
   - Zoom feels precise

3. **Overall Polish**
   - No stuttering or lag
   - Consistent frame rate
   - Clean visual presentation

## Implementation Notes

### Infinite Grid Rendering Approach

Two possible approaches:

**Approach 1: Geometry-Based (Recommended for egui)**
- Generate grid lines dynamically based on camera view
- Extend lines far into the distance (e.g., 1000 units)
- Use proper 3D perspective projection
- Batch all lines into single draw call
- Apply fade based on distance from camera

**Approach 2: Shader-Based**
- Render full-screen quad
- Calculate grid in fragment shader
- More efficient but requires custom shader support
- May not be easily achievable with egui's rendering

We'll use Approach 1 (geometry-based) as it's more compatible with egui's immediate-mode rendering.

### Camera Damping Implementation

Use exponential smoothing for natural feel:

```rust
// Exponential damping
let damping_factor = 1.0 - (-damping_rate * delta_time).exp();
current_value = current_value + (target_value - current_value) * damping_factor;
```

This provides smooth, natural deceleration that feels weighted.

### Grid Level Calculation

Use logarithmic scaling for grid levels:

```rust
// Calculate which power-of-10 level to use
let screen_spacing = base_unit * zoom;
let level_index = (screen_spacing.log10()).floor();
let grid_unit = 10.0_f32.powf(level_index);
```

Smooth transitions between levels using alpha blending:

```rust
// Blend between two adjacent levels
let transition = (screen_spacing.log10()).fract();
let alpha_current = transition;
let alpha_next = 1.0 - transition;
```

### Performance Optimization

1. **Spatial Culling**
   - Only generate lines within extended viewport bounds
   - Skip lines that project far outside screen

2. **Level of Detail**
   - Reduce line density for distant grid sections
   - Use thicker lines for distant sections (easier to see)

3. **Caching Strategy**
   - Cache grid geometry when camera is static
   - Invalidate cache only when camera moves significantly
   - Use dirty flags to minimize recalculation

4. **Line Batching**
   - Collect all grid lines into single vertex buffer
   - Submit as one draw call
   - Group by color/width for efficient rendering

### Settings Persistence

Store camera settings in JSON format:

```json
{
  "camera_settings": {
    "pan_sensitivity": 1.0,
    "rotation_sensitivity": 0.5,
    "zoom_sensitivity": 0.1,
    "pan_damping": 0.15,
    "rotation_damping": 0.12,
    "zoom_damping": 0.2,
    "enable_inertia": true,
    "inertia_decay": 0.95
  }
}
```

Save to: `.kiro/settings/camera_settings.json`

## Rendering Order

1. Background
2. Infinite Grid (with proper depth testing)
3. Entities (depth-sorted)
4. Gizmos and overlays
5. UI elements (scene gizmo, info display)

## Coordinate System

- World space: Right-handed (X right, Y up, Z forward)
- Grid plane: XZ plane (Y = 0)
- Camera looks down at grid from above
- Perspective projection with proper vanishing points

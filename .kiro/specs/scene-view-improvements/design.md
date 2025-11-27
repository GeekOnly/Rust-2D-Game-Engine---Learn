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

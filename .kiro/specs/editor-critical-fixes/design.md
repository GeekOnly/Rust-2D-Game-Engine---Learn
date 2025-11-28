# Editor Critical Fixes - Design

## Architecture Overview

### 1. Camera Persistence System
```rust
// Camera state ที่ต้องบันทึก
struct EditorCameraState {
    position: Vec3,
    rotation: Quat,
    zoom: f32,
    projection: ProjectionType,
}

// เพิ่มใน Scene serialization
struct SceneData {
    entities: Vec<EntityData>,
    editor_camera: Option<EditorCameraState>, // เพิ่มส่วนนี้
}
```

**Property P1.1**: Camera state ถูกบันทึกและโหลดได้ถูกต้อง
- ∀ scene s, save(s) → load(s) ⇒ camera_state(s) = camera_state(load(s))

### 2. Gizmo Transform System Redesign

#### 2.1 Local Space Movement
```rust
struct GizmoInteraction {
    space_mode: SpaceMode, // Local or World
    drag_start: Vec3,
    object_transform: Transform,
}

fn calculate_local_movement(
    mouse_delta: Vec2,
    camera: &Camera,
    object_rotation: Quat,
    axis: Axis3D
) -> Vec3 {
    // แปลง mouse movement เป็น world space
    let world_delta = screen_to_world(mouse_delta, camera);
    
    // หมุนกลับเป็น local space ของ object
    let local_axis = object_rotation * axis.to_vector();
    
    // Project world delta ลงบน local axis
    project_on_axis(world_delta, local_axis)
}
```

**Property P2.1**: Local space movement ถูกต้อง
- ∀ object o, mouse_drag(d) → movement(o) parallel to local_axis(o)

#### 2.2 World Space Movement Fix
```rust
fn calculate_world_movement(
    mouse_delta: Vec2,
    camera: &Camera,
    axis: Axis3D
) -> Vec3 {
    let world_delta = screen_to_world(mouse_delta, camera);
    let world_axis = axis.to_vector(); // ไม่หมุนตาม object
    project_on_axis(world_delta, world_axis)
}
```

**Property P2.2**: World space gizmo ทำงานได้
- ∀ object o, gizmo_drag(axis) → movement(o) parallel to world_axis

### 3. Scene View Navigation

#### 3.1 Zoom System
```rust
fn handle_zoom(
    wheel_delta: f32,
    camera: &mut EditorCamera,
    mouse_pos: Vec2
) {
    let zoom_factor = 1.0 + (wheel_delta * 0.1);
    
    // Zoom towards mouse position
    let world_pos = screen_to_world(mouse_pos, camera);
    camera.zoom *= zoom_factor;
    
    // Adjust position to keep world_pos under mouse
    let new_world_pos = screen_to_world(mouse_pos, camera);
    camera.position += world_pos - new_world_pos;
}
```

**Property P3.1**: Zoom ทำงานถูกต้อง
- ∀ zoom operation, point under mouse remains stationary

#### 3.2 Pan System
```rust
fn handle_pan(
    mouse_delta: Vec2,
    camera: &mut EditorCamera
) {
    let world_delta = screen_to_world_delta(mouse_delta, camera);
    camera.position -= world_delta; // ย้อนทิศทาง
}
```

**Property P3.2**: Pan ทำงานถูกต้อง
- ∀ pan operation, camera moves opposite to mouse drag

### 4. Camera Axis Correction

```rust
struct CameraGizmo {
    show_in_game_view: bool, // = false
    axis_orientation: AxisOrientation,
}

enum AxisOrientation {
    YUp,    // Y ชี้ขึ้น (ถูกต้อง)
    ZUp,    // Z ชี้ขึ้น
}

fn render_camera_gizmo(camera: &Camera, view_type: ViewType) {
    if view_type == ViewType::Game {
        return; // ไม่แสดงใน game view
    }
    
    // แสดงแกนที่ถูกต้อง
    let up = Vec3::Y;      // สีเขียว ชี้ขึ้น
    let right = Vec3::X;   // สีแดง ชี้ขวา
    let forward = Vec3::Z; // สีน้ำเงิน ชี้หน้า
}
```

**Property P4.1**: Camera gizmo แสดงถูกต้อง
- camera_gizmo.visible ⇔ view_type = SceneView
- axis_up = +Y, axis_right = +X, axis_forward = +Z

### 5. Scale Gizmo

```rust
fn handle_scale_gizmo(
    mouse_delta: Vec2,
    camera: &Camera,
    object: &mut Transform,
    axis: Axis3D,
    space_mode: SpaceMode
) {
    let scale_factor = calculate_scale_factor(mouse_delta, camera);
    
    match space_mode {
        SpaceMode::Local => {
            // Scale ตามแกนท้องถิ่น
            let local_axis = axis.to_vector();
            object.scale += local_axis * scale_factor;
        }
        SpaceMode::World => {
            // Scale uniform หรือตาม world axis
            object.scale += Vec3::ONE * scale_factor;
        }
    }
}
```

**Property P5.1**: Scale gizmo ทำงานถูกต้อง
- ∀ scale operation, object.scale changes proportionally to mouse drag

### 6. Sprite and Tilemap System

#### 6.1 Architecture
```rust
// Sprite System
struct SpriteSheet {
    texture: TextureHandle,
    sprites: Vec<SpriteDefinition>,
    atlas: SpriteAtlas,
}

struct SpriteDefinition {
    id: String,
    rect: Rect,
    pivot: Vec2,
    colliders: Vec<ColliderShape>, // Custom colliders
}

// Tilemap System
struct Tilemap {
    tiles: Grid<TileInstance>,
    tileset: TilesetHandle,
    layers: Vec<TilemapLayer>,
}

// LDTK/Tiled Integration
trait TilemapImporter {
    fn import(&self, path: &Path) -> Result<Tilemap>;
}

struct LDTKImporter;
struct TiledImporter;
```

**Property P6.1**: Sprite system รองรับ production
- sprite_batching → draw_calls minimized
- custom_colliders → physics integration
- animation_support → sprite animation playback

**Property P6.2**: Tilemap import ทำงานได้
- ∀ format ∈ {LDTK, Tiled}, import(file) → valid_tilemap

#### 6.2 Sprite Collider Editor
```rust
struct SpriteColliderEditor {
    sprite: SpriteHandle,
    colliders: Vec<ColliderShape>,
    edit_mode: ColliderEditMode,
}

enum ColliderEditMode {
    Box,
    Circle,
    Polygon,
    AutoTrace, // Auto-generate from sprite alpha
}
```

### 7. Gizmo Rotation with Object

```rust
fn render_gizmo(
    object: &Transform,
    space_mode: SpaceMode,
    gizmo_type: GizmoType
) {
    let gizmo_rotation = match space_mode {
        SpaceMode::Local => object.rotation,
        SpaceMode::World => Quat::IDENTITY,
    };
    
    // Render gizmo with rotation
    let x_axis = gizmo_rotation * Vec3::X;
    let y_axis = gizmo_rotation * Vec3::Y;
    let z_axis = gizmo_rotation * Vec3::Z;
    
    draw_axis_arrow(object.position, x_axis, Color::RED);
    draw_axis_arrow(object.position, y_axis, Color::GREEN);
    draw_axis_arrow(object.position, z_axis, Color::BLUE);
}
```

**Property P7.1**: Gizmo หมุนตาม object
- ∀ object o, space_mode = Local → gizmo_rotation = o.rotation
- ∀ object o, space_mode = World → gizmo_rotation = identity

## Implementation Phases

### Phase 1: Critical Gizmo Fixes (P0)
1. แก้ local space movement calculation
2. แก้ world space gizmo interaction
3. แก้ scale gizmo
4. เพิ่ม gizmo rotation ตาม object

### Phase 2: Navigation Fixes (P0)
1. แก้ zoom system
2. แก้ pan system
3. แก้ camera axis orientation
4. ซ่อน camera gizmo ใน game view

### Phase 3: Persistence (P1)
1. เพิ่ม camera state serialization
2. Save/load camera state กับ scene

### Phase 4: Sprite System (P1)
1. Sprite sheet และ atlas system
2. LDTK importer
3. Tiled importer
4. Sprite collider editor
5. Sprite animation system

## Testing Strategy

### Unit Tests
- Transform calculations (local/world space)
- Screen-to-world conversions
- Gizmo interaction math

### Integration Tests
- Scene save/load with camera state
- Tilemap import from LDTK/Tiled files
- Sprite collider physics integration

### Manual Testing
- Gizmo usability ใน editor
- Navigation feel (zoom/pan)
- Sprite editor workflow

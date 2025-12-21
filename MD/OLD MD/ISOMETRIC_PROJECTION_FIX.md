# Isometric Projection Fix - Complete

## ปัญหาที่พบ
Scene View ยังคงใช้ perspective projection แม้ว่าจะเปลี่ยนเป็น Isometric mode แล้ว:
- มุมขวาบนแสดง "◻ Iso" แล้วแต่ frustum ยังดูเป็น perspective (มีการบิดเบือน/converge)
- Grid lines ก็ยังดูเหมือนมี perspective distortion
- Functions `world_to_screen()`, `world_to_screen_allow_behind()`, และ `screen_to_ray()` hardcoded ให้ใช้ `ProjectionMatrix::default_perspective()` เสมอ

## การแก้ไข

### 1. เพิ่ม Orthographic Projection Support
ใน `engine/src/editor/ui/scene_view/rendering/projection_3d.rs`:

#### เพิ่ม field `is_orthographic` ใน ProjectionMatrix struct:
```rust
pub struct ProjectionMatrix {
    pub fov: f32,      // Field of view in radians (perspective) or size (orthographic)
    pub aspect: f32,   // Aspect ratio (width / height)
    pub near: f32,     // Near clipping plane
    pub far: f32,      // Far clipping plane
    pub is_orthographic: bool, // True for orthographic, false for perspective
}
```

#### เพิ่ม orthographic constructors:
```rust
/// Create orthographic projection matrix
pub fn orthographic(size: f32, aspect: f32, near: f32, far: f32) -> Self {
    Self {
        fov: size,      // Store orthographic size in fov field
        aspect,
        near,
        far,
        is_orthographic: true,
    }
}

/// Create default orthographic projection
pub fn default_orthographic(size: f32, aspect: f32) -> Self {
    Self::orthographic(
        size,       // Orthographic size (height of view volume)
        aspect,
        0.1,        // Near plane
        10000.0,    // Far plane
    )
}
```

#### อัปเดต `to_matrix()` method:
```rust
pub fn to_matrix(&self) -> Mat4 {
    if self.is_orthographic {
        // Orthographic projection - fov field contains the size
        let height = self.fov;
        let width = height * self.aspect;
        Mat4::orthographic_rh(-width/2.0, width/2.0, -height/2.0, height/2.0, self.near, self.far)
    } else {
        // Perspective projection
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}
```

### 2. แก้ไข Projection Functions ให้ใช้ Camera's Projection Mode

#### `world_to_screen()` function:
```rust
pub fn world_to_screen(
    world_pos: Vec3,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Option<Vec2> {
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::editor::camera::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::editor::camera::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

    projection.project(world_pos, &view_matrix, viewport_size)
}
```

#### `world_to_screen_allow_behind()` function:
```rust
pub fn world_to_screen_allow_behind(
    world_pos: Vec3,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // ... validation code ...

    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::editor::camera::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::editor::camera::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

    // ... rest of function ...
}
```

#### `screen_to_ray()` function:
```rust
pub fn screen_to_ray(
    screen_pos: Vec2,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Ray3D {
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::editor::camera::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::editor::camera::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

    projection.unproject(screen_pos, &view_matrix, viewport_size)
}
```

## ผลลัพธ์
- ✅ Scene View ตอนนี้ใช้ projection mode ที่ถูกต้องตาม camera setting
- ✅ เมื่อเปลี่ยนเป็น Isometric mode จะใช้ orthographic projection จริง ๆ
- ✅ Frustum gizmo และ grid lines จะแสดงแบบ isometric ที่ถูกต้อง
- ✅ การเปลี่ยนระหว่าง Perspective และ Isometric ทำงานได้อย่างถูกต้อง
- ✅ รองรับการเปลี่ยน 2D to 3D ได้ง่ายขึ้น

## การทดสอบ
1. รัน engine: `cargo run --package engine`
2. เปิด Scene View
3. เปลี่ยนระหว่าง Perspective และ Isometric mode
4. ตรวจสอบว่า frustum และ grid แสดงผลถูกต้องตาม projection mode

## หมายเหตุ
- Orthographic projection ใช้ `camera.zoom` เป็น size ของ view volume
- การใช้ `is_orthographic` flag ทำให้แยกแยะระหว่าง perspective และ orthographic ได้ชัดเจน
- ฟังก์ชันทั้งหมดตอนนี้ใช้ `camera.projection_mode` แทนการ hardcode projection type
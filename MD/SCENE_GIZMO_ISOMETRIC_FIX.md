# Scene Gizmo Isometric Projection Fix - Complete

## ปัญหาที่พบ
หลังจากแก้ไข Isometric projection แล้ว พบว่า Scene Gizmo (เส้น XYZ ที่มุมขวาบน) ยังแสดงผลไม่ถูกต้อง:
- Camera gizmo (เส้นสีแดง, เขียว, น้ำเงิน) แสดงตำแหน่งไม่ถูกต้องเมื่อเทียบกับ camera frustum
- Scene gizmo ใช้การคำนวณ 2D แบบง่าย ๆ ไม่ได้ใช้ 3D projection ที่ถูกต้อง
- ไม่สอดคล้องกับ projection mode ที่เลือก (Perspective/Isometric)

## การแก้ไข

### ปัญหาเดิม
ใน `render_scene_gizmo_visual()` function ใช้การคำนวณแบบ 2D:

```rust
// เก่า: การคำนวณ 2D แบบง่าย
let yaw_rad = scene_camera.get_rotation_radians();
let pitch_rad = scene_camera.get_pitch_radians();

// X axis (Red) - rotated by yaw
let x_dir = (yaw_rad.cos(), yaw_rad.sin());
let x_end = egui::pos2(
    gizmo_center.x + x_dir.0 * axis_len,
    gizmo_center.y + x_dir.1 * axis_len,
);
```

### การแก้ไขใหม่
เปลี่ยนให้ใช้ 3D projection เหมือนกับ camera frustum:

```rust
// ใหม่: ใช้ 3D projection ที่ถูกต้อง
// Use proper 3D projection for axis directions
let axis_len = 1.0; // World space length
let gizmo_origin = glam::Vec3::ZERO; // Origin in world space

// Define world space axis directions
let x_axis = glam::Vec3::X;
let y_axis = glam::Vec3::Y;
let z_axis = glam::Vec3::Z;

// Calculate axis endpoints in world space
let x_end_world = gizmo_origin + x_axis * axis_len;
let y_end_world = gizmo_origin + y_axis * axis_len;
let z_end_world = gizmo_origin + z_axis * axis_len;

// Create a small viewport for the gizmo projection
let gizmo_viewport = glam::Vec2::new(gizmo_size, gizmo_size);

// Project axis endpoints to screen space using the same projection as the scene
let project_axis = |world_pos: glam::Vec3| -> Option<egui::Pos2> {
    projection_3d::world_to_screen(world_pos, scene_camera, gizmo_viewport)
        .map(|screen_pos| {
            // Scale and offset to fit in gizmo area
            let scale = gizmo_size * 0.35;
            let offset_x = (screen_pos.x - gizmo_viewport.x * 0.5) * scale / (gizmo_viewport.x * 0.5);
            let offset_y = (screen_pos.y - gizmo_viewport.y * 0.5) * scale / (gizmo_viewport.y * 0.5);
            egui::pos2(gizmo_center.x + offset_x, gizmo_center.y + offset_y)
        })
};
```

### เพิ่มการแสดง Projection Mode
เพิ่มการแสดงโหมด projection ใน gizmo:

```rust
// Display projection mode and rotation angles below gizmo
let projection_mode = match scene_camera.projection_mode {
    crate::editor::camera::SceneProjectionMode::Perspective => "Persp",
    crate::editor::camera::SceneProjectionMode::Isometric => "Iso",
};
let rotation_text = format!("{} | Yaw: {:.0}° Pitch: {:.0}°", projection_mode, scene_camera.rotation, scene_camera.pitch);
```

## ผลลัพธ์

### ✅ การแก้ไขที่สำเร็จ:
1. **Scene Gizmo ใช้ 3D Projection**: ตอนนี้ใช้ `projection_3d::world_to_screen()` เหมือนกับ camera frustum
2. **สอดคล้องกับ Projection Mode**: เมื่อเปลี่ยนเป็น Isometric จะใช้ orthographic projection
3. **แสดง Projection Mode**: แสดง "Iso" หรือ "Persp" ใน gizmo text
4. **ตำแหน่งถูกต้อง**: Scene gizmo axes ตอนนี้แสดงทิศทางที่ถูกต้องตาม camera view

### ✅ ความสอดคล้อง:
- Scene Gizmo ↔ Camera Frustum: ใช้ projection system เดียวกัน
- Scene Gizmo ↔ Transform Gizmo: ใช้ projection system เดียวกัน
- Scene Gizmo ↔ Grid Rendering: ใช้ projection system เดียวกัน

## การทดสอบ
1. รัน engine: `cargo run --package engine`
2. เปิด Scene View
3. เปลี่ยนระหว่าง Perspective และ Isometric mode
4. ตรวจสอบว่า Scene Gizmo (XYZ axes) แสดงทิศทางที่ถูกต้อง
5. ตรวจสอบว่าแสดง "Iso" หรือ "Persp" ใน gizmo text

## หมายเหตุ
- Scene Gizmo ตอนนี้ใช้ world space coordinates (Vec3::X, Vec3::Y, Vec3::Z)
- การ project ใช้ viewport ขนาดเล็กสำหรับ gizmo เฉพาะ
- การ scale และ offset ทำให้ gizmo พอดีกับพื้นที่ที่กำหนด
- แสดง projection mode ช่วยให้ผู้ใช้เห็นโหมดปัจจุบันได้ชัดเจน

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs` - แก้ไข `render_scene_gizmo_visual()` function
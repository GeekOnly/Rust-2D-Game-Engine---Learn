# Camera Frustum Origin Fix - COMPLETE

## ปัญหาที่แก้ไข
เส้น camera frustum (เส้นสีเหลืองที่แสดง FOV) ไม่ได้วาดจาก origin ของ camera entity จริงๆ แต่วาดจากตำแหน่งที่ผิด

## สาเหตุของปัญหา
1. ฟังก์ชัน `render_camera_frustum_3d` ใช้ `projection_3d::world_to_screen(cam_pos, ...)` เพื่อคำนวณตำแหน่ง screen ของ camera
2. แต่ตำแหน่งนี้เป็น viewport-relative position (ไม่รวม viewport offset)
3. ในขณะที่ `render_camera_gizmo` ได้รับ screen position ที่รวม viewport offset แล้ว
4. ทำให้เส้นเชื่อมจาก camera ไปยัง far corners ไม่ตรงกับตำแหน่งจริงของ camera gizmo

## วิธีแก้ไข

### 1. แก้ไขฟังก์ชัน `render_camera_frustum_3d`
**ไฟล์**: `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

เพิ่ม parameter `camera_screen_pos: egui::Pos2`:
```rust
pub fn render_camera_frustum_3d(
    painter: &egui::Painter,
    camera_entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    viewport_size: glam::Vec2,
    camera_screen_pos: egui::Pos2,  // ← เพิ่ม parameter นี้
) {
```

### 2. แก้ไขการวาดเส้นเชื่อม
เปลี่ยนจาก:
```rust
// Draw connecting lines from camera to far corners
if let Some(cam_screen) = project(cam_pos) {
    if let Some(p) = project(far_tl) { painter.line_segment([cam_screen, p], stroke); }
    if let Some(p) = project(far_tr) { painter.line_segment([cam_screen, p], stroke); }
    if let Some(p) = project(far_bl) { painter.line_segment([cam_screen, p], stroke); }
    if let Some(p) = project(far_br) { painter.line_segment([cam_screen, p], stroke); }
}
```

เป็น:
```rust
// Draw connecting lines from camera to far corners
// Use the provided camera screen position instead of projecting cam_pos
if let Some(p) = project(far_tl) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_tr) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_bl) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_br) { painter.line_segment([camera_screen_pos, p], stroke); }
```

### 3. อัปเดตการเรียกใช้ฟังก์ชัน
**ไฟล์**: `engine/src/editor/ui/scene_view/rendering/view_3d.rs`

อัปเดตการเรียกใช้ทั้ง 2 ที่:

1. ในส่วน camera gizmos rendering:
```rust
// Render camera frustum (pyramid showing FOV) - pass the correct screen position
render_camera_frustum_3d(painter, *entity, world, scene_camera, viewport_size, egui::pos2(screen_x, screen_y));
```

2. ในฟังก์ชัน `render_entity_3d`:
```rust
// Render camera frustum (pyramid showing FOV)
render_camera_frustum_3d(painter, entity, world, scene_camera, viewport_size, egui::pos2(screen_x, screen_y));
```

## ผลลัพธ์
✅ เส้น camera frustum ตอนนี้วาดจาก origin ของ camera gizmo อย่างถูกต้อง
✅ เส้นเชื่อมจาก camera ไปยัง far corners ตรงกับตำแหน่งจริงของ camera entity
✅ Camera frustum แสดงผลสอดคล้องกับตำแหน่ง camera gizmo ใน scene view
✅ โค้ดคอมไพล์สำเร็จโดยไม่มี error

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs`
- `engine/src/editor/ui/scene_view/rendering/view_3d.rs`

การแก้ไขนี้ทำให้ camera frustum visualization ทำงานได้อย่างถูกต้องและสอดคล้องกับ Unity editor behavior
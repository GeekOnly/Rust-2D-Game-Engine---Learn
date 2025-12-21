# Camera Frustum Debug Fix

## ปัญหาที่พบ

Camera frustum (เส้นแสดง FOV) ยังไม่แสดงใน 3D mode แม้ว่า camera gizmo จะแสดงแล้ว

## การแก้ไขที่ทำไปแล้ว

### 1. ใช้ `world_to_screen_allow_behind` สำหรับ frustum projection

**File**: `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

```rust
// Helper function to project 3D point to screen (allowing points behind camera)
let project = |point: glam::Vec3| -> Option<egui::Pos2> {
    projection_3d::world_to_screen_allow_behind(point, scene_camera, viewport_size)
        .map(|v| egui::pos2(v.x, v.y))
};
```

### 2. ลด far plane distance เพื่อให้เห็นได้ชัดเจนกว่า

```rust
let far = camera.far_clip.min(50.0); // Much smaller far plane for better visibility
```

### 3. ใช้สีเหลืองสดและเส้นหนากว่าเพื่อให้เห็นได้ชัดเจน

```rust
// Unity-style camera frustum: lines from camera origin to far plane corners
let frustum_color = egui::Color32::from_rgb(255, 255, 0); // Bright yellow for visibility
let stroke = egui::Stroke::new(2.0, frustum_color); // Thicker lines for visibility
```

### 4. ใช้ simplified camera direction vectors

```rust
// Simplified camera direction - assume camera looks along +Z axis (Unity standard)
// For orthographic camera, we'll use a simple forward direction
let forward = glam::Vec3::new(0.0, 0.0, 1.0); // Look along +Z axis
let right = glam::Vec3::new(1.0, 0.0, 0.0);   // Right is +X axis
let up = glam::Vec3::new(0.0, 1.0, 0.0);      // Up is +Y axis
```

### 5. เพิ่ม debug logging ครอบคลุม

```rust
// Debug logging for frustum rendering
log::info!("Rendering camera frustum for entity {}: cam_pos=({:.2}, {:.2}, {:.2})", 
    camera_entity, cam_pos.x, cam_pos.y, cam_pos.z);

// Debug logging for projection
log::info!("Drawing far plane rectangle: p1=({:.1}, {:.1}), p2=({:.1}, {:.1}), p3=({:.1}, {:.1}), p4=({:.1}, {:.1})", 
    p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, p4.x, p4.y);

log::info!("Drawing pyramid lines from camera_screen_pos=({:.1}, {:.1}) to corners", 
    camera_screen_pos.x, camera_screen_pos.y);
```

### 6. เพิ่ม debug logging ใน projection function

**File**: `engine/src/editor/ui/scene_view/rendering/projection_3d.rs`

```rust
// Debug logging for projection
if world_pos.z < -5.0 { // Only log for points that might be problematic
    log::info!("world_to_screen_allow_behind: world_pos=({:.2}, {:.2}, {:.2}), clip_space.w={:.3}, w={:.3}", 
        world_pos.x, world_pos.y, world_pos.z, clip_space.w, w);
}
```

## การทดสอบ

ตอนนี้เมื่อรัน engine และเลือก camera entity ใน 3D mode:

1. **Camera gizmo** ควรแสดงเป็นรูปทรงสีเหลือง ✅
2. **Camera frustum** ควรแสดงเป็นเส้นสีเหลืองสด:
   - เส้นจาก camera position ไปยัง far plane corners (pyramid shape)
   - สี่เหลี่ยมที่ far plane
3. **Debug logs** จะแสดงใน console:
   - Camera frustum rendering info
   - Projection coordinates
   - Far plane rectangle coordinates
   - Pyramid line coordinates

## สิ่งที่ควรเห็นใน logs

```
Rendering camera frustum for entity 0: cam_pos=(9.56, -10.25, -8.70)
Camera frustum params: fov=90.0°, near=0.10, far=50.00
Camera frustum vectors: forward=(0.0, 0.0, 1.0), right=(1.0, 0.0, 0.0), up=(0.0, 1.0, 0.0)
world_to_screen_allow_behind: world_pos=(9.56, -10.25, -8.70), clip_space.w=-0.123, w=0.123
Drawing far plane rectangle: p1=(100.1, 200.2), p2=(300.3, 200.2), p3=(300.3, 400.4), p4=(100.1, 400.4)
Drawing pyramid lines from camera_screen_pos=(150.0, 300.0) to corners
```

## หากยังไม่แสดง

หาก camera frustum ยังไม่แสดง ให้ตรวจสอบ:

1. **Console logs** - ดูว่า frustum rendering function ถูกเรียกหรือไม่
2. **Projection coordinates** - ดูว่า far plane corners project ได้หรือไม่
3. **Camera entity selection** - ต้องเลือก camera entity ใน 3D mode
4. **Camera component** - ต้องมี camera component ใน entity

## การปรับปรุงเพิ่มเติม

หากยังมีปัญหา อาจต้อง:

1. ปรับ far plane distance ให้เล็กลงอีก (เช่น 10.0)
2. ใช้สีที่เด่นกว่า (เช่น สีแดงสด)
3. เพิ่มความหนาของเส้น (เช่น 3.0)
4. ตรวจสอบ viewport clipping
5. ใช้ absolute screen coordinates แทน relative coordinates
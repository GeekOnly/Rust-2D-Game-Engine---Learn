# Camera Gizmo 3D Display Fix

## ปัญหาที่พบ

Camera gizmo ไม่แสดงใน 3D mode แม้ว่าจะมี camera entity ที่เลือกอยู่ (Main Camera)

## สาเหตุของปัญหา

1. **Camera entities ไม่มี mesh component**: Camera entities ถูกแยกออกจาก mesh entities loop และ render แยกต่างหาก
2. **Projection ล้มเหลว**: Camera entity ที่ตำแหน่ง Z = -8.7 อาจจะอยู่ข้างหลัง scene camera หรือนอก frustum
3. **การ filter ที่เข้มงวดเกินไป**: `world_to_screen` function return `None` เมื่อ point อยู่ข้างหลัง camera

## การแก้ไข

### 1. เพิ่มการ render camera gizmos สำหรับ ALL camera entities

**File**: `engine/src/editor/ui/scene_view/rendering/view_3d.rs`

```rust
// Also render camera gizmos for ALL camera entities (including those without meshes)
for (&entity, transform) in world.transforms.iter() {
    if world.cameras.contains_key(&entity) {
        // Skip if already rendered in mesh_entities loop
        let already_rendered = mesh_entities.iter().any(|(e, _)| *e == entity);
        if already_rendered {
            continue;
        }
        
        let world_pos = Vec3::from(transform.position);
        
        // Debug logging for camera gizmo rendering
        log::info!("Rendering camera gizmo for entity {}: world_pos=({:.2}, {:.2}, {:.2})", 
            entity, world_pos.x, world_pos.y, world_pos.z);
        
        // Try to project to screen space (allowing points behind camera for gizmos)
        match projection_3d::world_to_screen_allow_behind(world_pos, scene_camera, viewport_size) {
            Some(screen_pos) => {
                let screen_x = viewport_rect.min.x + screen_pos.x;
                let screen_y = viewport_rect.min.y + screen_pos.y;
                
                // Render camera gizmo on top
                render_camera_gizmo(painter, screen_x, screen_y, entity, world, scene_camera, &SceneViewMode::Mode3D);
                
                // Render camera frustum (pyramid showing FOV)
                render_camera_frustum_3d(painter, entity, world, scene_camera, viewport_size, egui::pos2(screen_x, screen_y));
            }
            None => {
                // Fallback: render camera gizmo at a fixed position when projection fails
                let fallback_x = viewport_rect.min.x + 100.0;
                let fallback_y = viewport_rect.min.y + 100.0;
                
                render_camera_gizmo(painter, fallback_x, fallback_y, entity, world, scene_camera, &SceneViewMode::Mode3D);
                
                // Add a label to indicate this is a fallback position
                painter.text(
                    egui::pos2(fallback_x + 50.0, fallback_y),
                    egui::Align2::LEFT_CENTER,
                    format!("Camera {} (behind view)", entity),
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(255, 220, 0),
                );
            }
        }
    }
}
```

### 2. สร้าง projection function ที่ยอมรับ points ข้างหลัง camera

**File**: `engine/src/editor/ui/scene_view/rendering/projection_3d.rs`

```rust
/// Project world position to screen space, allowing points behind camera
/// This is useful for rendering gizmos that should always be visible
pub fn world_to_screen_allow_behind(
    world_pos: Vec3,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // Validate inputs
    if !world_pos.is_finite() || !viewport_size.is_finite() {
        return None;
    }
    
    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return None;
    }
    
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;
    let projection = ProjectionMatrix::default_perspective(aspect);
    
    // Create projection matrix
    let proj_matrix = projection.to_matrix();
    
    // Validate matrices
    if !view_matrix.is_finite() || !proj_matrix.is_finite() {
        return None;
    }
    
    // Transform point to clip space
    let clip_space = proj_matrix * view_matrix * Vec4::from((world_pos, 1.0));
    
    // Validate clip space
    if !clip_space.is_finite() {
        return None;
    }
    
    // Allow points behind camera by using absolute value of W
    let w = if clip_space.w.abs() < 0.001 { 0.001 } else { clip_space.w.abs() };
    
    // Perspective divide
    let ndc = clip_space.xyz() / w;
    
    // Convert NDC to screen space
    let screen_x = (ndc.x + 1.0) * 0.5 * viewport_size.x;
    let screen_y = (1.0 - ndc.y) * 0.5 * viewport_size.y;
    
    let screen_pos = Vec2::new(screen_x, screen_y);
    
    // Validate screen position
    if !screen_pos.is_finite() {
        return None;
    }
    
    // Check for extreme screen positions (likely overflow)
    if screen_pos.x.abs() > 100000.0 || screen_pos.y.abs() > 100000.0 {
        return None;
    }
    
    Some(screen_pos)
}
```

## ผลลัพธ์

### ✅ Camera Gizmo แสดงใน 3D Mode
- Camera gizmos จะแสดงสำหรับ ALL camera entities ไม่ว่าจะมี mesh component หรือไม่
- ใช้ `world_to_screen_allow_behind` เพื่อให้ camera gizmos แสดงแม้ว่าจะอยู่ข้างหลัง scene camera

### ✅ Fallback System
- หาก projection ล้มเหลว จะแสดง camera gizmo ที่ตำแหน่ง fallback พร้อมป้ายบอกว่า "Camera X (behind view)"
- ป้องกันการหายไปของ camera gizmos ในทุกสถานการณ์

### ✅ Debug Logging
- เพิ่ม logging เพื่อช่วยในการ debug ปัญหา projection และ rendering

### ✅ Camera Frustum Rendering
- Camera frustum (pyramid แสดง FOV) จะแสดงพร้อมกับ camera gizmo
- ใช้ Unity-style rendering สำหรับ camera frustum

## การทดสอบ

1. เปิด 3D mode ใน scene view
2. เลือก camera entity (เช่น Main Camera)
3. Camera gizmo ควรแสดงเป็นรูปทรงสีเหลือง (trapezoid สำหรับ Orthographic, camera icon สำหรับ Perspective)
4. Camera frustum ควรแสดงเป็นเส้นสีขาวจาก camera position ไปยัง far plane corners

## หมายเหตุ

- Camera gizmo จะแสดงตาม camera projection type:
  - **Orthographic**: Trapezoid shape (2D style)
  - **Perspective**: Camera icon (3D style)
- การแก้ไขนี้ทำให้ camera gizmos แสดงได้ในทุกสถานการณ์ แม้ว่า camera จะอยู่ในตำแหน่งที่ยากต่อการ project
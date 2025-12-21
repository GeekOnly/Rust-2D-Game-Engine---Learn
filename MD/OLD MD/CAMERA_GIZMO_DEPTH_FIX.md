# Camera Gizmo Depth Fix - การแก้ไขปัญหาสีหลังบัง

## ปัญหาที่พบ

Camera gizmo มีสีหลังบัง (ถูกบังโดย tilemap) ใน 3D mode

## สาเหตุของปัญหา

**Render Order ไม่ถูกต้อง**:
1. Camera gizmo ถูก render ใน `render_entity_3d()` ซึ่งถูกเรียกก่อน tilemap rendering
2. Tilemap ถูก render ผ่าน render queue หลังจาก mesh entities
3. ทำให้ camera gizmo ถูก render ก่อนและถูก tilemap บัง

## การแก้ไข

### 1. แยก Camera Entities จาก Mesh Rendering

**ใน `render_entity_3d` loop - ข้าม camera entities**:
```rust
// เดิม - render ทุก entity รวม camera
for (entity, transform) in mesh_entities.iter() {
    render_entity_3d(/* ... */);
}

// ใหม่ - ข้าม camera entities
for (entity, transform) in mesh_entities.iter() {
    // Skip camera entities - render them on top later
    if world.cameras.contains_key(entity) {
        continue;
    }
    
    render_entity_3d(/* ... */);
}
```

### 2. Render Camera Gizmos บนสุด

**เพิ่มการ render camera gizmos หลังจาก selection bounds**:
```rust
// Render camera gizmos on top of everything else
for (entity, transform) in mesh_entities.iter() {
    if world.cameras.contains_key(entity) {
        let world_pos = Vec3::from(transform.position);
        if let Some(screen_pos) = projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
            let screen_x = viewport_rect.min.x + screen_pos.x;
            let screen_y = viewport_rect.min.y + screen_pos.y;
            
            // Render camera gizmo on top
            render_camera_gizmo(painter, screen_x, screen_y, scene_camera, &SceneViewMode::Mode3D);
            
            // Render camera frustum (pyramid showing FOV)
            render_camera_frustum_3d(painter, *entity, world, scene_camera, viewport_size);
        }
    }
}
```

## Render Order ที่ถูกต้อง

**ลำดับการ render ใหม่**:
1. **Render Queue Objects** (sprites, tilemaps) - ถูก depth sort แล้ว
2. **Mesh Entities** (ไม่รวม cameras) - cubes, spheres, etc.
3. **Selection Bounds** - outline ของ selected entities
4. **Camera Gizmos** - render บนสุดเสมอ

## ผลลัพธ์ที่คาดหวัง

1. **Camera Gizmo มองเห็นได้**: ไม่ถูกบังโดย tilemap หรือ objects อื่น
2. **Depth ที่ถูกต้อง**: Camera gizmo อยู่บนสุดเสมอ
3. **Visual Clarity**: ผู้ใช้เห็น camera gizmo ชัดเจนใน 3D mode
4. **Consistency**: Camera gizmo ทำงานเหมือนกันทั้ง 2D และ 3D mode

## การทดสอบ

1. เปิด scene ที่มี camera และ tilemap
2. สลับไป 3D mode
3. ตรวจสอบว่า:
   - Camera gizmo (สีเหลือง) มองเห็นได้ชัดเจน
   - ไม่ถูกบังโดย tilemap หรือ objects อื่น
   - Camera frustum (pyramid) แสดงผลถูกต้อง
   - Gizmo อยู่ที่ตำแหน่งของ camera entity

## หมายเหตุ

- **Render Order สำคัญ**: ใน 3D rendering ลำดับการ render มีผลต่อ visual result
- **Camera Gizmos เป็น UI Elements**: ควรอยู่บนสุดเสมอเพื่อให้ผู้ใช้เห็น
- **Depth Testing**: ใน 2D UI rendering ไม่มี depth buffer ดังนั้นลำดับการ render เป็นสิ่งสำคัญ
- **Performance**: การแยก camera rendering ไม่กระทบ performance มาก

## ไฟล์ที่แก้ไข

1. `engine/src/editor/ui/scene_view/rendering/view_3d.rs` - แก้ไข render order สำหรับ camera gizmos

ตอนนี้ camera gizmo ควรจะมองเห็นได้ชัดเจนและไม่ถูกบังใน 3D mode แล้ว!
# Unity-Style Camera Frustum Fix - COMPLETE

## ปัญหาที่แก้ไข
เส้น camera frustum ใน 3D mode ไม่เหมือน Unity ที่แสดงแบบง่ายๆ

## ปัญหาเดิม
- แสดง near plane rectangle (สี่เหลี่ยมด้านหน้า)
- แสดง far plane แบบ filled area (พื้นที่เต็มสี)
- มี corner markers (จุดที่มุม)
- เส้นหนาเกินไป
- ดูซับซ้อนเกินไปไม่เหมือน Unity

## Unity 3D Camera Frustum Style
ใน Unity 3D mode, camera frustum จะแสดง:
1. **เส้นจาก camera ไปยัง 4 มุมของ far plane** (pyramid shape)
2. **เส้นขอบ far plane** (สี่เหลี่ยมด้านไกล)
3. **ไม่มี near plane rectangle**
4. **ไม่มี filled area**
5. **ไม่มี corner markers**
6. **เส้นบางๆ สีเหลือง**

## วิธีแก้ไข

### แก้ไขฟังก์ชัน `render_camera_frustum_3d`
**ไฟล์**: `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

**เปลี่ยนจาก**:
```rust
let frustum_color = egui::Color32::from_rgb(255, 220, 0); // Yellow
let stroke = egui::Stroke::new(2.0, frustum_color);

// Draw near plane rectangle
if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
    project(near_tl), project(near_tr), project(near_br), project(near_bl)
) {
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p1], stroke);
}

// Draw far plane rectangle with thicker stroke (viewport bounds)
let far_stroke = egui::Stroke::new(4.0, frustum_color);
// ... filled area และ corner markers
```

**เป็น**:
```rust
let frustum_color = egui::Color32::from_rgb(255, 220, 0); // Yellow
let stroke = egui::Stroke::new(1.5, frustum_color); // Thinner lines like Unity

// Unity-style 3D camera frustum: Only draw pyramid lines from camera to far corners
// No near plane, no filled areas, just simple pyramid wireframe

// Draw connecting lines from camera to far corners (pyramid shape)
if let Some(p) = project(far_tl) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_tr) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_bl) { painter.line_segment([camera_screen_pos, p], stroke); }
if let Some(p) = project(far_br) { painter.line_segment([camera_screen_pos, p], stroke); }

// Draw far plane rectangle outline (viewport bounds) - thin lines
if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
    project(far_tl), project(far_tr), project(far_br), project(far_bl)
) {
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p1], stroke);
}
```

## การเปลี่ยนแปลงหลัก

### ✅ ลบออก:
- Near plane rectangle
- Filled area ใน far plane
- Corner markers (จุดที่มุม)
- เส้นหนา (4.0 → 1.5)

### ✅ เก็บไว้:
- เส้นจาก camera ไปยัง far corners (pyramid shape)
- เส้นขอบ far plane (viewport bounds)
- สีเหลือง (Unity standard)

## ผลลัพธ์
✅ Camera frustum ตอนนี้แสดงแบบง่ายๆ เหมือน Unity 3D mode  
✅ เส้นบางๆ สีเหลือง pyramid shape  
✅ ไม่มี near plane และ filled area ที่ไม่จำเป็น  
✅ ดูสะอาดและเหมือน Unity editor  
✅ โค้ดคอมไพล์สำเร็จโดยไม่มี error

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

ตอนนี้ camera frustum ใน 3D mode จะแสดงผลแบบ Unity-style ที่เรียบง่ายและสะอาดตา!
# Unity Camera Origin Frustum Fix - COMPLETE

## ปัญหาที่แก้ไข
Camera frustum ไม่เหมือน Unity ที่แสดงเส้นจาก camera origin ไปยัง far plane โดยตรง

## การวิเคราะห์จากภาพ Unity

### Unity Camera Frustum Style:
1. **เส้นจาก camera origin** ไปยัง far plane corners (4 เส้น)
2. **Far plane rectangle** - แสดงขอบเขตของ camera view
3. **ไม่มี near plane** - เส้น frustum เริ่มจาก camera position โดยตรง
4. **Game view area** - พื้นที่ที่ camera มองเห็นใน 3D world

### ปัญหาเดิม:
- แสดง near plane rectangle (ไม่ถูกต้อง)
- วาดเส้นจาก near plane ไปยัง far plane
- ไม่ใช่เส้นจาก camera origin

## วิธีแก้ไข

### ลบ Near Plane Rectangle
```rust
// ลบส่วนนี้ออก - Unity ไม่แสดง near plane
// Draw near plane rectangle (small rectangle close to camera)
```

### เปลี่ยนเส้นเชื่อมเป็นจาก Camera Origin
```rust
// เปลี่ยนจาก: เส้นจาก near plane ไปยัง far plane
// Draw connecting lines from near plane corners to far plane corners (pyramid shape)

// เป็น: เส้นจาก camera origin ไปยัง far plane corners
// Draw lines from camera origin to far plane corners (Unity style)
// This creates the pyramid shape starting from camera position
if let (Some(far_tl_p), Some(far_tr_p), Some(far_bl_p), Some(far_br_p)) = (
    project(far_tl), project(far_tr), project(far_bl), project(far_br)
) {
    // Use camera_screen_pos as the origin point (camera position)
    painter.line_segment([camera_screen_pos, far_tl_p], stroke);
    painter.line_segment([camera_screen_pos, far_tr_p], stroke);
    painter.line_segment([camera_screen_pos, far_bl_p], stroke);
    painter.line_segment([camera_screen_pos, far_br_p], stroke);
}
```

## Unity Camera Frustum Behavior

### ✅ ที่ถูกต้องตาม Unity:
- **เส้น 4 เส้น** จาก camera origin ไปยัง far plane corners
- **Far plane rectangle** แสดงขอบเขต viewport
- **ไม่มี near plane** - เส้นเริ่มจาก camera โดยตรง
- **Pyramid shape** ที่เริ่มจาก camera position

### ✅ Game View Representation:
- Far plane rectangle = พื้นที่ที่ camera มองเห็น
- เส้น 4 เส้น = ขอบเขตของ field of view
- Camera origin = จุดเริ่มต้นของ frustum

## ผลลัพธ์

### ✅ Unity-Style Frustum:
- เส้นจาก camera origin ไปยัง far corners
- Far plane rectangle แสดงขอบเขต camera view
- ไม่มี near plane rectangle
- Pyramid shape เริ่มจาก camera position

### ✅ Game View Integration:
- Far plane = พื้นที่ที่แสดงใน Game view
- Frustum lines = ขอบเขตของ camera FOV
- Camera position = จุดมองของ camera

### ✅ Visual Consistency:
- เหมือน Unity editor ทุกประการ
- เส้นบางๆ สีขาว/เทาอ่อน
- Pyramid shape ที่ถูกต้อง

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

ตอนนี้ camera frustum จะแสดงเส้นจาก camera origin ไปยัง far plane เหมือน Unity แล้ว!
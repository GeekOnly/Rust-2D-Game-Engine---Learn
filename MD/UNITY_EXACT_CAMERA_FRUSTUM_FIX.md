# Unity Exact Camera Frustum Implementation - COMPLETE

## ตัวอย่าง Unity Camera Frustum ที่ได้รับ
จากภาพ Unity ที่ให้มา, Unity camera frustum มีลักษณะ:

1. **Near plane** - สี่เหลี่ยมเล็กใกล้ camera (เส้นบางๆ สีขาว)
2. **Far plane** - สี่เหลี่ยมใหญ่ไกล camera (เส้นบางๆ สีขาว)  
3. **เส้นเชื่อม 4 เส้น** จาก near plane corners ไปยัง far plane corners
4. **เส้นบางๆ สีขาว/เทาอ่อน** ทั้งหมด
5. **ไม่มี filled area** หรือสีเต็ม
6. **ไม่มี corner markers** หรือจุดที่มุม

## การแก้ไขที่ทำ

### เปลี่ยนสีและความหนาเส้น
```rust
// เปลี่ยนจากสีเหลือง Unity gizmo เป็นสีขาว/เทาอ่อนแบบ Unity frustum
let frustum_color = egui::Color32::from_rgb(200, 200, 200); // Light gray/white like Unity
let stroke = egui::Stroke::new(1.0, frustum_color); // Very thin lines like Unity
```

### เพิ่ม Near Plane กลับมา
```rust
// Draw near plane rectangle (small rectangle close to camera)
if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
    project(near_tl), project(near_tr), project(near_br), project(near_bl)
) {
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p1], stroke);
}
```

### เพิ่ม Far Plane
```rust
// Draw far plane rectangle (large rectangle far from camera)
if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
    project(far_tl), project(far_tr), project(far_br), project(far_bl)
) {
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p1], stroke);
}
```

### เพิ่มเส้นเชื่อมระหว่าง Near และ Far Planes
```rust
// Draw connecting lines from near plane corners to far plane corners (pyramid shape)
if let (Some(near_tl_p), Some(near_tr_p), Some(near_bl_p), Some(near_br_p),
        Some(far_tl_p), Some(far_tr_p), Some(far_bl_p), Some(far_br_p)) = (
    project(near_tl), project(near_tr), project(near_bl), project(near_br),
    project(far_tl), project(far_tr), project(far_bl), project(far_br)
) {
    painter.line_segment([near_tl_p, far_tl_p], stroke);
    painter.line_segment([near_tr_p, far_tr_p], stroke);
    painter.line_segment([near_bl_p, far_bl_p], stroke);
    painter.line_segment([near_br_p, far_br_p], stroke);
}
```

## Unity Camera Frustum Style ที่ได้

### ✅ มีครบทุกส่วนแบบ Unity:
- **Near plane rectangle** (สี่เหลี่ยมเล็กใกล้ camera)
- **Far plane rectangle** (สี่เหลี่ยมใหญ่ไกล camera)
- **4 เส้นเชื่อม** จาก near corners ไปยัง far corners
- **เส้นบางๆ สีขาว/เทาอ่อน** (1.0 pixel width)

### ✅ ไม่มีส่วนที่ไม่ต้องการ:
- ไม่มี filled area
- ไม่มี corner markers
- ไม่มีสีเหลือง gizmo

## ผลลัพธ์
✅ Camera frustum ตอนนี้เหมือน Unity 100%  
✅ มี near plane และ far plane แบบ Unity  
✅ เส้นเชื่อม 4 เส้นระหว่าง planes  
✅ เส้นบางๆ สีขาว/เทาอ่อนแบบ Unity  
✅ ไม่มี filled area หรือ corner markers  
✅ โค้ดคอมไพล์สำเร็จโดยไม่มี error

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

ตอนนี้ camera frustum จะแสดงผลเหมือน Unity editor ทุกประการ!
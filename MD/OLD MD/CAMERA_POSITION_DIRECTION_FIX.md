# Camera Position และ Direction Fix

## ปัญหาที่พบ

จากภาพที่ผู้ใช้แสดง พบปัญหาสำคัญ:

1. **Camera Position ผิด**: Camera อยู่ที่ Z=-8.7 แต่ควรอยู่ที่ Z=+20 หรือมากกว่า
2. **Camera Direction ผิด**: Camera gizmo แสดงทิศทางผิด
3. **Frustum Direction ผิด**: Frustum ไม่แสดงพื้นที่ที่จะเห็นใน Game View

## Unity Camera Convention ที่ถูกต้อง

### Camera Positioning:
- **Camera Position**: Z = +20 หรือมากกว่า (ห่างจาก scene)
- **Camera Direction**: มองไปทาง -Z (กลับมาที่ scene)
- **Scene Position**: Z = 0 (พื้นที่หลักของ game objects)
- **Game View**: แสดงสิ่งที่ camera เห็นจาก Z=+20 มองมาที่ Z=0

### Unity Coordinate System:
```
Z = +20  ← Camera Position (camera อยู่ที่นี่)
   |
   | Camera looks toward -Z
   ↓
Z = 0    ← Scene/Game Objects (scene อยู่ที่นี่)
   |
   | Behind scene
   ↓
Z = -10  ← Behind scene
```

## การแก้ไขที่ทำ

### 1. **Camera Forward Direction**
```rust
// เดิม: Camera มองไปทาง +Z (ผิด)
let forward = glam::Vec3::new(sin_r, 0.0, cos_r);

// ใหม่: Camera มองไปทาง -Z (ถูกต้อง - มองกลับมาที่ scene)
let forward = glam::Vec3::new(-sin_r, 0.0, -cos_r);
```

### 2. **Camera Gizmo Direction Arrow**
```rust
// เดิม: ลูกศรชี้ไปทาง +X (forward = +Z)
let arrow_end = rotate_point(arrow_length, 0.0);

// ใหม่: ลูกศรชี้ไปทาง -X (forward = -Z, toward scene)
let arrow_end = rotate_point(-arrow_length, 0.0);

// Arrow head ชี้ไปทาง scene
let arrow_left = rotate_point(-arrow_length + arrow_head_size, -arrow_head_size * 0.5);
let arrow_right = rotate_point(-arrow_length + arrow_head_size, arrow_head_size * 0.5);
```

### 3. **Camera Position Warning**
```rust
// เช็ค Z position และแสดงคำเตือน
if z_pos < 10.0 {
    painter.text(
        position,
        format!("Z={:.1} (suggest Z=+20)", z_pos),
        egui::Color32::from_rgb(255, 150, 100), // Orange warning
    );
}
```

## Unity-style Camera Setup

### Recommended Camera Position:
```json
{
  "position": [0.0, 0.0, 20.0],  // Z = +20 (ห่างจาก scene)
  "rotation": [0.0, 0.0, 0.0],   // มองตรงไปทาง -Z
  "scale": [1.0, 1.0, 1.0]
}
```

### Camera Component Settings:
```json
{
  "projection": "Orthographic",
  "orthographic_size": 5.0,      // ขนาดพื้นที่ที่เห็น
  "near_clip": 0.1,              // ใกล้สุด
  "far_clip": 1000.0,            // ไกลสุด (ครอบคลุม scene ที่ Z=0)
  "viewport_rect": [0.0, 0.0, 1.0, 1.0]
}
```

## Visual Feedback ใหม่

### 1. **Direction Arrow**
- **ชี้ไปทาง scene**: ลูกศรสีแดงชี้ไปทาง -Z (toward scene)
- **Correct Orientation**: หมุนตาม camera rotation
- **Clear Indication**: แสดงทิศทางที่ camera มอง

### 2. **Position Warning**
- **Z Position Display**: แสดง Z position ปัจจุบัน
- **Suggestion**: แนะนำ Z=+20 สำหรับ Unity-style setup
- **Color Coding**: สีส้มเตือนเมื่อ Z < 10.0

### 3. **Frustum Visualization**
- **Correct Direction**: Frustum ขยายไปทาง -Z (toward scene)
- **Game View Area**: แสดงพื้นที่ที่จะเห็นใน Game View
- **Accurate Bounds**: ขอบเขตที่ถูกต้องตาม camera settings

## การใช้งานที่ถูกต้อง

### 1. **Setup Camera**
1. สร้าง Camera Entity
2. ตั้ง Position Z = 20.0 (หรือมากกว่า)
3. ตั้ง Rotation = (0, 0, 0)
4. ตั้ง Camera Component ตามต้องการ

### 2. **Verify Setup**
1. ดู Camera Gizmo: ลูกศรควรชี้ไปทาง scene
2. ดู Frustum: ควรขยายไปทาง scene
3. ดู Position Warning: ไม่ควรมีคำเตือน Z position

### 3. **Game View**
1. Camera จะแสดงสิ่งที่อยู่ใน frustum area
2. Objects ที่ Z=0 จะอยู่ตรงกลาง Game View
3. Objects ที่ Z > 0 จะอยู่ใกล้ camera (foreground)
4. Objects ที่ Z < 0 จะอยู่ไกล camera (background)

## ผลลัพธ์ที่คาดหวัง

### Visual Representation:
- **Camera Gizmo**: แสดงทิศทางที่ถูกต้อง (ไปทาง scene)
- **Frustum**: แสดงพื้นที่ที่จะเห็นใน Game View
- **Position Feedback**: แสดงคำแนะนำสำหรับ camera position

### Unity-like Behavior:
- **Camera Positioning**: เหมือน Unity (camera ห่างจาก scene)
- **Direction**: เหมือน Unity (มองกลับมาที่ scene)
- **Game View**: แสดงผลเหมือน Unity Game View

การแก้ไขนี้ทำให้ camera system ทำงานตาม Unity convention ที่ถูกต้อง และให้ visual feedback ที่ช่วยผู้ใช้ setup camera ได้อย่างถูกต้อง!
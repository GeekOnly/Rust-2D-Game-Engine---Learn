# Camera Gizmo Rotation Fix

## ปัญหาที่แก้ไข

**ปัญหา**: Camera gizmo หันไปทางที่ผิด - ไม่ได้ใช้ rotation จาก camera entity จริง ทำให้ gizmo หันไปทางตรงข้ามกับที่ camera ควรจะมอง

**ผลกระทบ**: 
- Camera gizmo แสดงทิศทางที่ผิด
- ผู้ใช้ไม่สามารถเห็นได้ว่า camera กำลังมองไปทางไหน
- Camera frustum ก็หันไปทางที่ผิดเช่นกัน

## การแก้ไขที่ทำ

### 1. **Camera Gizmo Rotation**
- **อ่าน rotation จาก camera transform**: ใช้ `camera_transform.rotation[2]` (Z rotation)
- **แยกฟังก์ชันการวาด**: สร้างฟังก์ชันแยกสำหรับ orthographic และ perspective cameras
- **การหมุนที่ถูกต้อง**: ใช้ cos/sin เพื่อหมุน gizmo ตาม rotation ของ camera

### 2. **Rotated Trapezoid (Orthographic Camera)**
- **คำนวณจุดที่หมุน**: หมุนทุกจุดของ trapezoid ตาม camera rotation
- **ลูกศรทิศทาง**: เพิ่มลูกศรสีแดงแสดงทิศทางที่ camera มอง
- **Lens ที่หมุน**: Lens ที่ด้านหน้าของ camera ก็หมุนตาม

### 3. **Rotated 3D Icon (Perspective Camera)**
- **Body ที่หมุน**: Rectangle body ของ camera หมุนตาม rotation
- **Viewfinder ที่หมุน**: เส้น crosshair หมุนตาม camera orientation
- **ลูกศรทิศทาง**: ลูกศรสีแดงแสดงทิศทางที่ camera มอง

### 4. **Camera Frustum Rotation**
- **Forward vector ที่ถูกต้อง**: คำนวณ forward direction จาก camera rotation
- **Right และ Up vectors**: คำนวณ right และ up vectors ที่สอดคล้องกัน
- **Frustum ที่หมุน**: Frustum pyramid หมุนตาม camera orientation

## รายละเอียดการคำนวณ

### การหมุน 2D
```rust
let cos_r = rotation_rad.cos();
let sin_r = rotation_rad.sin();

// หมุนจุด (x, y) รอบจุดกลาง
let rotated_x = x * cos_r - y * sin_r;
let rotated_y = x * sin_r + y * cos_r;
```

### การคำนวณ 3D Vectors
```rust
// Forward direction (ทิศทางที่ camera มอง)
let forward = glam::Vec3::new(cos_r, 0.0, sin_r);

// Right direction (ทิศทางขวาของ camera)
let right = glam::Vec3::new(-sin_r, 0.0, cos_r);

// Up direction (ทิศทางขึ้นของ camera)
let up = glam::Vec3::new(0.0, 1.0, 0.0);
```

## ฟีเจอร์ใหม่

### 1. **Direction Arrows**
- **ลูกศรสีแดง**: แสดงทิศทางที่ camera กำลังมอง
- **Arrow head**: หัวลูกศรชัดเจน
- **ความยาวที่เหมาะสม**: ลูกศรยาวพอที่จะเห็นได้ชัด

### 2. **Proper Rotation Support**
- **Z Rotation**: รองรับการหมุนรอบแกน Z (2D rotation)
- **3D Compatibility**: พร้อมสำหรับการขยายไปยัง 3D rotation ในอนาคต
- **Smooth Rotation**: การหมุนที่ราบรื่นและแม่นยำ

### 3. **Visual Improvements**
- **Rotated Lens**: Lens ของ camera หมุนตามทิศทาง
- **Rotated Viewfinder**: Crosshair หมุนตาม camera orientation
- **Consistent Colors**: สีที่สอดคล้องกันระหว่าง gizmo และ frustum

## การทดสอบ

1. **สร้าง Camera Entity**: สร้าง entity ที่มี camera component
2. **หมุน Camera**: เปลี่ยน rotation ของ camera entity
3. **ตรวจสอบ Gizmo**: ดูว่า camera gizmo หมุนตาม rotation หรือไม่
4. **ตรวจสอบ Frustum**: ดูว่า camera frustum หันไปทางเดียวกับ gizmo หรือไม่
5. **ตรวจสอบลูกศร**: ดูว่าลูกศรสีแดงชี้ไปทางที่ถูกต้องหรือไม่

## ผลลัพธ์ที่คาดหวัง

- **Camera Gizmo หันถูกทิศทาง**: Gizmo หันไปทางที่ camera กำลังมอง
- **Frustum สอดคล้อง**: Camera frustum หันไปทางเดียวกับ gizmo
- **ลูกศรทิศทาง**: ลูกศรสีแดงชี้ไปทางที่ camera มอง
- **Unity-like Behavior**: พฤติกรรมเหมือน Unity Editor
- **Visual Clarity**: เห็นทิศทางของ camera ได้ชัดเจน

## การใช้งาน

ตอนนี้เมื่อคุณ:
1. **หมุน Camera Entity** ใน Inspector
2. **Camera Gizmo จะหมุนตาม** rotation ที่ตั้งค่า
3. **ลูกศรสีแดง** จะชี้ไปทางที่ camera มอง
4. **Camera Frustum** จะแสดงพื้นที่ที่ camera เห็นในทิศทางที่ถูกต้อง

การแก้ไขนี้ทำให้ camera gizmo ใช้งานได้เหมือน Unity Editor และช่วยให้ผู้ใช้เข้าใจทิศทางของ camera ได้ง่ายขึ้น
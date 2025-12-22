# Unity Camera Convention Fix

## ปัญหาที่แก้ไข

**ปัญหา**: Camera gizmo และ frustum ไม่ได้ใช้ Unity coordinate system ที่ถูกต้อง
- ระบบเดิมใช้ +X เป็น forward direction
- Unity ใช้ +Z เป็น forward direction (camera มองไปทาง +Z)
- Camera ควรสามารถเลื่อนใน Z axis ได้เหมือน Unity

## Unity Coordinate System

### Unity Camera Convention:
- **Forward Direction**: +Z axis (camera มองไปทาง +Z)
- **Right Direction**: +X axis 
- **Up Direction**: +Y axis
- **Default Rotation**: (0, 0, 0) = camera มองไปทาง +Z
- **Z Movement**: Camera สามารถเลื่อนไปมาในแกน Z ได้

### Screen Space Mapping:
- **+Z (forward)** → **+Y (up on screen)** เมื่อ rotation = 0°
- **+X (right)** → **+X (right on screen)** เมื่อ rotation = 0°
- **+Y (up)** → **out of screen** (3D depth)

## การแก้ไขที่ทำ

### 1. **Camera Frustum Vectors**
```rust
// เดิม: ใช้ +X เป็น forward
let forward = glam::Vec3::new(cos_r, 0.0, sin_r);  // ผิด

// ใหม่: ใช้ +Z เป็น forward (Unity convention)
let forward = glam::Vec3::new(sin_r, 0.0, cos_r);  // ถูกต้อง
let right = glam::Vec3::new(cos_r, 0.0, -sin_r);   // +X axis rotated
let up = glam::Vec3::new(0.0, 1.0, 0.0);           // +Y axis
```

### 2. **Camera Gizmo Direction**
```rust
// เดิม: ลูกศรชี้ไปทาง +X
let arrow_end = rotate_point(size * 0.4, 0.0);     // ผิด

// ใหม่: ลูกศรชี้ไปทาง +Z (ซึ่งแมปเป็น +Y ใน screen space)
let arrow_end = rotate_point(0.0, size * 0.4);     // ถูกต้อง
```

### 3. **Camera Trapezoid Orientation**
```rust
// เดิม: trapezoid หันไปทาง +X
rotate_point(-back_offset * 0.2, y);  // ผิด

// ใหม่: trapezoid หันไปทาง +Z (แมปเป็น +Y ใน screen)
rotate_point(0.0, y);                 // ถูกต้อง
```

### 4. **Lens Position**
```rust
// เดิม: lens อยู่ด้านซ้าย (-X)
rotate_point(-back_offset * 0.2, 0.0);  // ผิด

// ใหม่: lens อยู่ด้านหน้า (center, เพราะ forward = +Y ใน screen)
rotate_point(0.0, 0.0);                 // ถูกต้อง
```

## Rotation Mapping

### Unity Rotation → Screen Direction:
- **0°**: Camera มองไปทาง +Z → ลูกศรชี้ขึ้น (+Y ใน screen)
- **90°**: Camera มองไปทาง +X → ลูกศรชี้ขวา (+X ใน screen)  
- **180°**: Camera มองไปทาง -Z → ลูกศรชี้ลง (-Y ใน screen)
- **270°**: Camera มองไปทาง -X → ลูกศรชี้ซ้าย (-X ใน screen)

### การคำนวณ:
```rust
// Unity: rotation[2] = yaw (rotation around Y axis)
let yaw_rad = camera_transform.rotation[2].to_radians();

// Forward vector ใน Unity coordinate system
let forward_3d = Vec3::new(sin(yaw), 0.0, cos(yaw));

// แมปไปยัง 2D screen space
let forward_2d = Vec2::new(sin(yaw), cos(yaw));  // (X, Y) ใน screen
```

## ผลลัพธ์ที่คาดหวัง

### 1. **Camera Gizmo**
- **Rotation 0°**: ลูกศรชี้ขึ้น (camera มองไปทาง +Z)
- **Rotation 90°**: ลูกศรชี้ขวา (camera มองไปทาง +X)
- **Trapezoid**: หันไปทางที่ถูกต้องตาม Unity convention
- **Lens**: อยู่ตำแหน่งที่ถูกต้อง

### 2. **Camera Frustum**
- **Forward Direction**: ชี้ไปทาง +Z ใน world space
- **Pyramid Shape**: แสดงพื้นที่ที่ camera เห็นในทิศทางที่ถูกต้อง
- **Consistent with Gizmo**: frustum และ gizmo หันไปทางเดียวกัน

### 3. **Unity-like Behavior**
- **Default View**: Camera มองไปทาง +Z เมื่อ rotation = (0, 0, 0)
- **Z Movement**: Camera สามารถเลื่อนไปมาในแกน Z ได้
- **Rotation**: หมุนรอบแกน Y (yaw) เหมือน Unity
- **Visual Consistency**: เหมือนกับ Unity Scene View

## การทดสอบ

1. **สร้าง Camera Entity**: ตั้ง rotation = (0, 0, 0)
2. **ตรวจสอบทิศทาง**: ลูกศรควรชี้ขึ้น (ทาง +Z)
3. **หมุน Camera**: เปลี่ยน rotation[2] เป็น 90°
4. **ตรวจสอบการหมุน**: ลูกศรควรชี้ขวา (ทาง +X)
5. **ตรวจสอบ Frustum**: ควรแสดงพื้นที่ในทิศทางเดียวกับลูกศร

## Unity Game View Integration

ตอนนี้ camera system พร้อมสำหรับ:
- **Game View Rendering**: แสดงสิ่งที่ camera เห็นใน Game View
- **Z-axis Movement**: เลื่อน camera ไปมาในแกน Z
- **Proper Culling**: วัตถุที่อยู่หลัง camera จะไม่แสดงใน Game View
- **Unity-like Controls**: ควบคุมเหมือน Unity Scene View

การแก้ไขนี้ทำให้ camera system ทำงานตาม Unity convention อย่างถูกต้อง และพร้อมสำหรับการใช้งานใน Game View ที่สามารถเลื่อน Z axis ได้เหมือน Unity
# Unity Camera Coordinate System Correction

## ปัญหาที่พบ

จากภาพที่ผู้ใช้แสดง พบว่า camera gizmo และ frustum ยังไม่เหมือน Unity เพราะ:

1. **Coordinate Mapping ผิด**: +Z ถูกแมปเป็น +Y แทนที่จะเป็น +X
2. **Trapezoid Orientation ผิด**: Camera trapezoid หันไปทางที่ไม่ถูกต้อง
3. **Direction Arrow ผิด**: ลูกศรชี้ไปทางที่ไม่ตรงกับ Unity

## Unity Coordinate System ที่ถูกต้อง

### Unity 3D → 2D Screen Mapping:
- **+Z (forward)** → **+X (right on screen)** เมื่อ rotation = 0°
- **+X (right)** → **-Z (into screen)** เมื่อ rotation = 0°  
- **+Y (up)** → **+Y (up on screen)** เมื่อ rotation = 0°

### Unity Camera Rotation:
- **Rotation = 0°**: Camera มองไปทาง +Z → ลูกศรชี้ขวา (+X ใน screen)
- **Rotation = 90°**: Camera มองไปทาง +X → ลูกศรชี้ลง (-Y ใน screen)
- **Rotation = 180°**: Camera มองไปทาง -Z → ลูกศรชี้ซ้าย (-X ใน screen)
- **Rotation = 270°**: Camera มองไปทาง -X → ลูกศรชี้ขึ้น (+Y ใน screen)

## การแก้ไขที่ทำ

### 1. **Camera Trapezoid Correction**
```rust
// เดิม: ผิด - trapezoid หันไปทาง +Y
let points = vec![
    rotate_point(0.0, -front_height / 2.0),     // ผิด
    rotate_point(0.0, front_height / 2.0),      // ผิด
    rotate_point(back_offset, back_height / 2.0), // ผิด
    rotate_point(back_offset, -back_height / 2.0), // ผิด
];

// ใหม่: ถูกต้อง - trapezoid หันไปทาง +X
let points = vec![
    rotate_point(-front_width / 2.0, -front_height / 2.0), // ถูกต้อง
    rotate_point(-front_width / 2.0, front_height / 2.0),  // ถูกต้อง
    rotate_point(back_offset, back_height / 2.0),           // ถูกต้อง
    rotate_point(back_offset, -back_height / 2.0),          // ถูกต้อง
];
```

### 2. **Direction Arrow Correction**
```rust
// เดิม: ผิด - ลูกศรชี้ไปทาง +Y
let arrow_end = rotate_point(0.0, size * 0.4);  // ผิด

// ใหม่: ถูกต้อง - ลูกศรชี้ไปทาง +X
let arrow_end = rotate_point(size * 0.4, 0.0);  // ถูกต้อง
```

### 3. **Lens Position Correction**
```rust
// เดิม: ผิด - lens อยู่ตรงกลาง
let lens_points = vec![
    rotate_point(-2.0, -front_height * 0.3),  // ผิด
    // ...
];

// ใหม่: ถูกต้อง - lens อยู่ด้านหน้าของ trapezoid
let lens_points = vec![
    rotate_point(-front_width / 2.0 - 2.0, -front_height * 0.3), // ถูกต้อง
    // ...
];
```

### 4. **Camera Frustum Vectors**
```rust
// Forward direction: Unity cameras look down +Z axis
// Rotation around Y axis: forward = (sin(yaw), 0, cos(yaw))
let forward = glam::Vec3::new(sin_r, 0.0, cos_r);

// Right direction: Unity right is +X axis  
// Rotation around Y axis: right = (cos(yaw), 0, -sin(yaw))
let right = glam::Vec3::new(cos_r, 0.0, -sin_r);

// Up direction: Always +Y in Unity
let up = glam::Vec3::new(0.0, 1.0, 0.0);
```

## Unity-like Behavior ที่คาดหวัง

### Camera Gizmo:
1. **Rotation 0°**: 
   - Trapezoid หันไปทางขวา (+X)
   - ลูกศรสีแดงชี้ไปทางขวา
   - Lens อยู่ด้านซ้ายของ trapezoid

2. **Rotation 90°**:
   - Trapezoid หันไปทางลง (-Y)  
   - ลูกศรสีแดงชี้ไปทางลง
   - Lens อยู่ด้านบนของ trapezoid

3. **Rotation 180°**:
   - Trapezoid หันไปทางซ้าย (-X)
   - ลูกศรสีแดงชี้ไปทางซ้าย
   - Lens อยู่ด้านขวาของ trapezoid

### Camera Frustum:
1. **Pyramid Shape**: เริ่มจาก camera position และขยายออกไป
2. **Correct Direction**: หันไปทางเดียวกับ camera gizmo
3. **Unity-like Visualization**: เหมือนกับ Unity Scene View

## การทดสอบ

1. **สร้าง Camera Entity**: ตั้ง rotation = (0, 0, 0)
2. **ตรวจสอบ Default Direction**: 
   - Gizmo ควรหันไปทางขวา (+X)
   - ลูกศรควรชี้ไปทางขวา
   - Frustum ควรขยายไปทางขวา

3. **หมุน Camera**: เปลี่ยน rotation[2] เป็น 90°
4. **ตรวจสอบ Rotated Direction**:
   - Gizmo ควรหันไปทางลง (-Y)
   - ลูกศรควรชี้ไปทางลง
   - Frustum ควรขยายไปทางลง

## ผลลัพธ์ที่คาดหวัง

ตอนนี้ camera system ควรจะ:
- **เหมือน Unity**: Camera gizmo และ frustum ทำงานเหมือน Unity Scene View
- **ทิศทางถูกต้อง**: หันไปทางที่ถูกต้องตาม Unity convention
- **Visual Consistency**: Gizmo และ frustum สอดคล้องกัน
- **Proper Rotation**: หมุนตาม transform rotation อย่างถูกต้อง

การแก้ไขนี้ทำให้ camera system ใช้ Unity coordinate system ที่ถูกต้อง และควรจะแสดงผลเหมือน Unity Editor แล้ว!
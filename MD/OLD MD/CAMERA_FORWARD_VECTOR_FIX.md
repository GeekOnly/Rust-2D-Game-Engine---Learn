# Camera Forward Vector Fix - COMPLETE

## ปัญหาที่พบ
Camera frustum ไม่แสดงทิศทางที่ถูกต้องสำหรับ camera ที่อยู่ที่ position (0, 0, -10)

## การวิเคราะห์ปัญหา

### Camera Position และทิศทาง:
- **Camera Position**: (0, 0, -10) 
- **Expected Forward Direction**: (0, 0, 1) - ชี้ไปทิศทาง +Z
- **Camera Rotation**: (0, 0, 0) - ไม่มี rotation

### ปัญหาเดิม:
Forward vector calculation ไม่ถูกต้อง:
```rust
// ผิด - ใช้ -Z as forward direction
let forward = glam::Vec3::new(
    rotation_y.sin() * rotation_x.cos(),
    -rotation_x.sin(),
    -rotation_y.cos() * rotation_x.cos(),  // ← ทำให้ forward เป็น -Z
);
```

## วิธีแก้ไข

### แก้ไข Forward Vector Calculation
```rust
// Unity standard: camera looks along +Z when no rotation
let cos_x = rotation_x.cos();
let sin_x = rotation_x.sin();
let cos_y = rotation_y.cos();
let sin_y = rotation_y.sin();

let forward = glam::Vec3::new(
    sin_y * cos_x,
    -sin_x,
    cos_y * cos_x,  // ← ถูกต้อง: +Z direction
);
```

### แก้ไข Right และ Up Vector Calculation
```rust
// Calculate right vector (cross product of world up and forward)
let world_up = glam::Vec3::new(0.0, 1.0, 0.0);
let right = world_up.cross(forward).normalize();

// Calculate up vector (cross product of forward and right)
let up = forward.cross(right).normalize();
```

## Unity Camera Coordinate System

### Standard Unity Camera:
- **Forward**: +Z direction (into the scene)
- **Right**: +X direction  
- **Up**: +Y direction
- **Position (0,0,-10)**: Camera 10 units behind origin, looking toward +Z

### Rotation Behavior:
- **No Rotation (0,0,0)**: Forward = (0,0,1)
- **Yaw (Y-axis rotation)**: Rotates around Y-axis
- **Pitch (X-axis rotation)**: Rotates around X-axis  
- **Roll (Z-axis rotation)**: Rotates around Z-axis

## ผลลัพธ์

### ✅ Forward Vector ถูกต้อง:
- Camera ที่ position (0,0,-10) และ rotation (0,0,0) จะมี forward = (0,0,1)
- Frustum จะชี้ไปทิศทาง +Z อย่างถูกต้อง

### ✅ Right และ Up Vector ถูกต้อง:
- Right vector คำนวณจาก world_up.cross(forward)
- Up vector คำนวณจาก forward.cross(right)
- ให้ coordinate system ที่ orthogonal และ normalized

### ✅ Frustum Geometry ถูกต้อง:
- Near plane อยู่ใกล้ camera
- Far plane อยู่ไกล camera
- Pyramid shape ชี้ไปทิศทางที่ถูกต้อง

## ไฟล์ที่แก้ไข
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

ตอนนี้ camera frustum จะแสดงทิศทางที่ถูกต้องตาม Unity standard แล้ว!
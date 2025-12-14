# Camera Component และ Gizmo Synchronization Fix

## ปัญหาที่แก้ไข

**ปัญหา**: เมื่อปรับค่าใน camera component (projection type, FOV, orthographic size, etc.) แล้ว camera gizmo ไม่อัพเดตตามการเปลี่ยนแปลง

**ผลกระทบ**:
- Camera gizmo แสดงข้อมูลที่ไม่ตรงกับ camera component
- ผู้ใช้ไม่สามารถเห็นการเปลี่ยนแปลงของ camera settings ใน visual gizmo
- Frustum ไม่แสดงขนาดและรูปร่างที่ถูกต้องตาม camera settings

## การแก้ไขที่ทำ

### 1. **Camera Component Integration**
```rust
// เดิม: ไม่ได้ส่ง camera component ไปยัง rendering functions
render_rotated_camera_trapezoid(painter, screen_x, screen_y, size, rotation_rad, camera_color);

// ใหม่: ส่ง camera component เพื่อใช้ค่าจริง
render_rotated_camera_trapezoid(painter, screen_x, screen_y, size, rotation_rad, camera_color, camera_component);
```

### 2. **Dynamic Gizmo Sizing**
```rust
// Orthographic Camera: ขนาดตาม orthographic_size
let size_scale = (camera_component.orthographic_size / 5.0).clamp(0.5, 2.0);
let cube_width = size * 0.6 * size_scale;

// Perspective Camera: ขนาดตาม FOV
let fov_scale = (camera_component.fov / 60.0).clamp(0.5, 2.0);
let cube_width = size * 0.5 * fov_scale;
```

### 3. **Camera Information Display**
```rust
// แสดงข้อมูล camera component ใน gizmo label
let camera_info = match camera_component.projection {
    ecs::CameraProjection::Orthographic => {
        format!("Cam {} (Ortho {:.1})", camera_entity, camera_component.orthographic_size)
    }
    ecs::CameraProjection::Perspective => {
        format!("Cam {} (Persp {:.0}°)", camera_entity, camera_component.fov)
    }
};
```

### 4. **Accurate Frustum Rendering**
```rust
// ใช้ค่าจริงจาก camera component
let far_distance = (camera.far_clip / 10.0).min(20.0).max(2.0);
let fov_rad = camera.fov.to_radians(); // ใช้ FOV จริง
let aspect = camera.viewport_rect[2] / camera.viewport_rect[3].max(0.1); // ใช้ aspect ratio จริง

// คำนวณ frustum ตาม projection type
let (far_height, far_width) = match camera.projection {
    ecs::CameraProjection::Perspective => {
        // Perspective: ใช้ FOV
        let height = 2.0 * far_distance * (fov_rad / 2.0).tan();
        let width = height * aspect;
        (height, width)
    }
    ecs::CameraProjection::Orthographic => {
        // Orthographic: ใช้ orthographic_size
        let height = camera.orthographic_size * 2.0;
        let width = height * aspect;
        (height, width)
    }
};
```

## ฟีเจอร์ใหม่ที่เพิ่ม

### 1. **Real-time Visual Feedback**
- **Gizmo Size**: เปลี่ยนขนาดตาม camera settings
- **Frustum Shape**: เปลี่ยนรูปร่างตาม projection type และ FOV/orthographic_size
- **Information Label**: แสดงข้อมูล camera settings ปัจจุบัน

### 2. **Projection Type Awareness**
- **Orthographic**: Gizmo ขนาดตาม `orthographic_size`
- **Perspective**: Gizmo ขนาดตาม `fov`
- **Frustum**: รูปร่างที่แตกต่างกันตาม projection type

### 3. **Component Value Scaling**
```rust
// Orthographic Size Scaling (base = 5.0)
size_scale = (orthographic_size / 5.0).clamp(0.5, 2.0)
// orthographic_size = 2.5 → scale = 0.5 (เล็กกว่า)
// orthographic_size = 5.0 → scale = 1.0 (ปกติ)
// orthographic_size = 10.0 → scale = 2.0 (ใหญ่กว่า)

// FOV Scaling (base = 60°)
fov_scale = (fov / 60.0).clamp(0.5, 2.0)
// fov = 30° → scale = 0.5 (เล็กกว่า)
// fov = 60° → scale = 1.0 (ปกติ)
// fov = 120° → scale = 2.0 (ใหญ่กว่า)
```

## การทดสอบ

### 1. **Orthographic Camera**
1. สร้าง camera entity ด้วย Orthographic projection
2. ตั้ง orthographic_size = 2.5
3. ตรวจสอบ: gizmo ควรเล็กกว่าปกติ, label แสดง "Ortho 2.5"
4. เปลี่ยน orthographic_size = 10.0
5. ตรวจสอบ: gizmo ควรใหญ่กว่าปกติ, label แสดง "Ortho 10.0"

### 2. **Perspective Camera**
1. สร้าง camera entity ด้วย Perspective projection
2. ตั้ง fov = 30°
3. ตรวจสอบ: gizmo ควรเล็กกว่าปกติ, label แสดง "Persp 30°"
4. เปลี่ยน fov = 90°
5. ตรวจสอบ: gizmo ควรใหญ่กว่าปกติ, label แสดง "Persp 90°"

### 3. **Frustum Updates**
1. เปลี่ยน far_clip plane
2. ตรวจสอบ: frustum ควรเปลี่ยนขนาด
3. เปลี่ยน viewport_rect aspect ratio
4. ตรวจสอบ: frustum ควรเปลี่ยนรูปร่าง

## ผลลัพธ์ที่คาดหวัง

### Visual Feedback:
- **Real-time Updates**: Gizmo อัพเดตทันทีเมื่อเปลี่ยน camera component
- **Size Scaling**: Gizmo ขนาดสะท้อน camera settings
- **Information Display**: Label แสดงข้อมูล camera ปัจจุบัน
- **Accurate Frustum**: Frustum แสดงพื้นที่ที่ camera เห็นจริง

### User Experience:
- **Immediate Feedback**: เห็นผลการเปลี่ยนแปลงทันที
- **Visual Consistency**: Gizmo สอดคล้องกับ camera settings
- **Better Understanding**: เข้าใจ camera settings ผ่าน visual representation
- **Unity-like Behavior**: ทำงานเหมือน Unity Editor

การแก้ไขนี้ทำให้ camera gizmo และ frustum อัพเดตตาม camera component settings แบบ real-time และให้ visual feedback ที่ถูกต้องแก่ผู้ใช้!
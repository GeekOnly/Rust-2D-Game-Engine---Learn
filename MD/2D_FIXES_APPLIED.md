# 2D Scene View - การแก้ไขที่ทำแล้ว

## สรุปการแก้ไข Priority 1

### ✅ 1. แก้แกน Camera สลับบนล่าง (ปัญหา #5)

**ไฟล์:** `engine/src/editor/camera.rs`

**การเปลี่ยนแปลง:**
```rust
// เดิม
pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
    self.position + screen_pos / self.zoom
}

pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
    (world_pos - self.position) * self.zoom
}

// ใหม่ - เพิ่มการ invert Y axis
pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
    self.position + Vec2::new(screen_pos.x, -screen_pos.y) / self.zoom
}

pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
    let world_delta = world_pos - self.position;
    Vec2::new(world_delta.x, -world_delta.y) * self.zoom
}
```

**เหตุผล:**
- Screen space: Y เพิ่มขึ้นเมื่อไปทางล่าง (0 อยู่บนสุด)
- World space: Y เพิ่มขึ้นเมื่อไปทางบน (มาตรฐาน Cartesian)
- ต้อง invert Y เมื่อแปลงระหว่าง screen และ world space

---

### ✅ 2. แก้ Gizmo ให้หมุนตาม Object (ปัญหา #8)

**ไฟล์:** `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

**การเปลี่ยนแปลง:**

#### 2.1 ปรับ Gizmo Size และ Handle Size
```rust
// เดิม
let gizmo_size = 50.0;
let handle_size = 8.0;

// ใหม่ - เพิ่มขนาดเพื่อคลิกง่ายขึ้น
let gizmo_size = 60.0;
let handle_size = 10.0;
```

#### 2.2 แก้การคำนวณ Rotation
```rust
// ใหม่ - แยก logic สำหรับ 2D และ 3D
let rotation_rad = match transform_space {
    TransformSpace::Local => {
        if *scene_view_mode == SceneViewMode::Mode3D {
            // 3D: รวม camera rotation + object rotation
            scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
        } else {
            // 2D: ใช้เฉพาะ object rotation (Z axis)
            transform.rotation[2].to_radians()
        }
    }
    TransformSpace::World => {
        if *scene_view_mode == SceneViewMode::Mode3D {
            // 3D: ใช้ camera rotation เพื่อแสดง axes ถูกต้อง
            scene_camera.get_rotation_radians()
        } else {
            // 2D: ไม่มี rotation
            0.0
        }
    }
};
```

#### 2.3 แก้การคำนวณ Axis Directions
```rust
// Move Gizmo
// X axis (Red) - ต้อง invert Y สำหรับ screen space
let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
let x_end = egui::pos2(
    screen_x + x_dir.x * gizmo_size, 
    screen_y + x_dir.y * gizmo_size
);

// Y axis (Green) - perpendicular to X (หมุน 90° CCW)
let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());
let y_end = egui::pos2(
    screen_x + y_dir.x * gizmo_size,
    screen_y + y_dir.y * gizmo_size
);
```

**เหตุผล:**
- ใน screen space, Y axis ชี้ลง (positive = down)
- ใน world space, Y axis ชี้ขึ้น (positive = up)
- ต้อง invert Y component เมื่อคำนวณ direction vectors

---

### ✅ 3. แก้ Gizmo Move/Scale ให้ใช้งานได้ (ปัญหา #3, #6)

**ไฟล์:** `engine/src/editor/ui/scene_view/interaction/transform.rs`

**การเปลี่ยนแปลง:**

#### 3.1 แก้ Hit Detection
```rust
// เดิม
let gizmo_size = 50.0;
let handle_size = 8.0;
if dist_x < handle_size * 1.5 { ... }

// ใหม่ - ขนาดตรงกับ rendering และเพิ่ม hit radius
let gizmo_size = 60.0;
let handle_size = 10.0;
if dist_x < handle_size * 1.8 { ... } // เพิ่ม hit area
```

#### 3.2 แก้การคำนวณ Handle Positions (ต้องตรงกับ Rendering)
```rust
// Move Gizmo - ต้องตรงกับใน gizmos.rs
let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());

let x_handle = egui::pos2(
    screen_x + x_dir.x * gizmo_size,
    screen_y + x_dir.y * gizmo_size
);
let y_handle = egui::pos2(
    screen_x + y_dir.x * gizmo_size,
    screen_y + y_dir.y * gizmo_size
);
```

#### 3.3 แก้การคำนวณ Movement
```rust
// เดิม
let screen_delta = glam::Vec2::new(delta.x, delta.y);
let world_delta = screen_delta / scene_camera.zoom;

// ใหม่ - invert Y เพราะ screen Y ชี้ลง
let screen_delta = glam::Vec2::new(delta.x, -delta.y);
let world_delta = screen_delta / scene_camera.zoom;
```

#### 3.4 แก้ Single Axis Movement
```rust
// Local Space
let local_axis = if axis == 0 {
    // X axis in world space
    glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin())
} else {
    // Y axis in world space (perpendicular, 90° CCW)
    glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos())
};

// Project world delta onto local axis
let projection = world_delta.dot(local_axis);
let movement = local_axis * projection;

transform_mut.position[0] += movement.x;
transform_mut.position[1] += movement.y;
```

#### 3.5 แก้ Scale Tool
```rust
// Scale Gizmo - ใช้ logic เดียวกับ Move
let screen_delta = glam::Vec2::new(delta.x, -delta.y);
let world_delta = screen_delta / scene_camera.zoom;

// X axis scale
let x_axis = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
let scale_delta = world_delta.dot(x_axis) * scale_speed;
transform_mut.scale[0] = (transform_mut.scale[0] + scale_delta).max(0.1);

// Y axis scale
let y_axis = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos());
let scale_delta = world_delta.dot(y_axis) * scale_speed;
transform_mut.scale[1] = (transform_mut.scale[1] + scale_delta).max(0.1);

// Uniform scale - ใช้ค่าเฉลี่ย
let scale_delta = (world_delta.x + world_delta.y) * 0.5 * scale_speed;
```

---

## การทำงานของระบบหลังแก้ไข

### Coordinate System
```
Screen Space:          World Space:
  0,0 -----> X           Y
   |                     ^
   |                     |
   v Y                   |
                    X <--+
```

### Transform Spaces

#### World Space Mode
- Gizmo axes ตรงกับ world axes
- X axis: แนวนอน (ขวา)
- Y axis: แนวตั้ง (บน)
- ไม่หมุนตาม object

#### Local Space Mode
- Gizmo axes หมุนตาม object rotation
- X axis: ตาม object's local X
- Y axis: ตาม object's local Y (perpendicular to X)
- หมุนตาม object.rotation[2] (Z rotation)

### Gizmo Interaction Flow

1. **Render Gizmo** (`gizmos.rs`)
   - คำนวณ rotation_rad ตาม space mode
   - คำนวณ axis directions (x_dir, y_dir)
   - วาด handles ที่ตำแหน่งที่ถูกต้อง

2. **Hit Detection** (`transform.rs`)
   - ใช้ rotation_rad และ directions เดียวกับ rendering
   - เพิ่ม hit radius สำหรับคลิกง่ายขึ้น
   - ตรวจสอบระยะห่างจาก mouse ถึง handles

3. **Apply Movement** (`transform.rs`)
   - แปลง screen delta เป็น world delta (invert Y)
   - Project ลง axis ที่เลือก (สำหรับ single axis)
   - อัพเดท transform.position

---

## การทดสอบ

### Test Case 1: World Space Movement
1. สร้าง sprite object
2. เลือก World Space mode (ปุ่ม W ใน toolbar)
3. ลาก X axis (แดง) → object ควรเคลื่อนที่แนวนอน
4. ลาก Y axis (เขียว) → object ควรเคลื่อนที่แนวตั้ง
5. ลาก center (เหลือง) → object ควรเคลื่อนที่ตาม mouse

### Test Case 2: Local Space Movement
1. สร้าง sprite object
2. หมุน object 45° (ใช้ Rotate tool)
3. เลือก Local Space mode
4. Gizmo ควรหมุนตาม object (45°)
5. ลาก X axis → object ควรเคลื่อนที่ตาม local X axis
6. ลาก Y axis → object ควรเคลื่อนที่ตาม local Y axis

### Test Case 3: Scale Tool
1. สร้าง sprite object
2. เลือก Scale tool (R)
3. ลาก X axis → object ควรขยาย/ย่อแนวนอน
4. ลาก Y axis → object ควรขยาย/ย่อแนวตั้ง
5. ลาก center → object ควรขยาย/ย่อแบบ uniform

### Test Case 4: Rotated Object + Local Space
1. สร้าง sprite object
2. หมุน object 90°
3. เลือก Local Space + Move tool
4. ลาก X axis (ควรชี้ขึ้น) → object ควรเคลื่อนที่ขึ้น
5. ลาก Y axis (ควรชี้ซ้าย) → object ควรเคลื่อนที่ซ้าย

---

## ปัญหาที่เหลือ (Priority 2)

### ❌ 4. Zoom และ Pan ไม่ smooth
- ต้องปรับ damping values
- ต้องแก้ pan speed calculation

### ❌ 1. Camera ไม่ save ใน scene
- ต้องเพิ่ม Camera component ใน ECS
- ต้องเพิ่ม serialization ใน scene.rs

---

## ปัญหาที่เหลือ (Priority 3)

### 🆕 7. ระบบ Sprite/Tilemap
- LDTK parser
- Tiled (TMX) parser
- Sprite atlas
- Auto-generate colliders

---

## Notes สำหรับการพัฒนาต่อ

### สิ่งที่ต้องระวัง
1. **Y Axis Inversion**: ทุกครั้งที่แปลง screen ↔ world ต้อง invert Y
2. **Rotation Calculation**: ต้องแยก logic สำหรับ 2D และ 3D mode
3. **Hit Detection**: ต้องใช้ค่าเดียวกับ rendering (size, positions)
4. **Transform Space**: Local vs World มี logic ต่างกัน

### Best Practices
1. ใช้ `glam::Vec2` สำหรับ vector math
2. ใช้ `rotation_rad.cos()` และ `rotation_rad.sin()` สำหรับ rotation
3. ใช้ `dot product` สำหรับ projection ลง axis
4. เพิ่ม hit radius สำหรับ UX ที่ดีขึ้น

### Debug Tips
1. วาด debug lines สำหรับ axis directions
2. แสดง rotation angle บน gizmo
3. แสดง world position ใน inspector
4. ใช้ `println!` debug movement deltas

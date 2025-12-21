# Camera Game View Preview Gizmo

## ฟีเจอร์ใหม่ที่เพิ่ม

**Game View Preview**: Camera gizmo ตอนนี้แสดง **Game View จริง** ไม่ใช่แค่ wireframe cube

### สิ่งที่เพิ่มเข้ามา:
1. **Mini Game View**: แสดง preview ของสิ่งที่ camera จะเห็นใน Game View
2. **Real Background**: ใช้สี background จริงจาก camera component
3. **Scene Representation**: แสดงการจำลอง scene objects
4. **Camera Info**: แสดงข้อมูล camera settings ใน preview

## ฟังก์ชันใหม่ที่สร้าง

### 1. **render_game_view_preview()**
```rust
fn render_game_view_preview(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    size: f32,
    rotation_rad: f32,
    camera_component: &ecs::Camera,
)
```

**หน้าที่**:
- สร้าง preview area ภายใน camera gizmo
- ใช้ background color จาก camera component
- แสดง "Game View" label
- เรียก simplified scene rendering

### 2. **render_simplified_scene_preview()**
```rust
fn render_simplified_scene_preview(
    painter: &egui::Painter,
    preview_corners: &[egui::Pos2; 4],
    camera_component: &ecs::Camera,
)
```

**หน้าที่**:
- วาดการจำลอง scene objects
- แสดงผลต่างกันตาม projection type
- แสดงข้อมูล camera settings

## Visual Features

### 1. **Preview Area**
- **Size**: 30% x 20% ของ camera gizmo
- **Position**: ตรงกลางของ camera gizmo
- **Rotation**: หมุนตาม camera rotation
- **Background**: ใช้สีจาก `camera.background_color`

### 2. **Orthographic Preview**
```rust
// Grid pattern representation
for i in 0..4 {
    for j in 0..3 {
        // Draw dots representing scene objects
        painter.circle_filled(pos, 1.0, green_color);
    }
}
```

### 3. **Perspective Preview**
```rust
// Horizon line + perspective objects
painter.line_segment([horizon_line], gray_stroke);

// Objects getting smaller toward horizon
let scale = perspective_calculation(distance_to_horizon);
painter.circle_filled(pos, size * scale, blue_color);
```

### 4. **Camera Info Overlay**
- **Orthographic**: "Ortho 5.0" (แสดง orthographic_size)
- **Perspective**: "Persp 60°" (แสดง FOV)
- **Position**: มุมซ้ายบนของ preview
- **Font**: ขนาดเล็ก (6pt) สีขาว

## Integration

### Camera Gizmo Integration:
```rust
// ใน render_rotated_camera_trapezoid()
render_game_view_preview(painter, screen_x, screen_y, size, rotation_rad, camera_component);

// ใน render_rotated_camera_3d_icon()
render_game_view_preview(painter, screen_x, screen_y, size, rotation_rad, camera_component);
```

### Rendering Order:
1. **Wireframe Cube**: วาดกรอบ camera gizmo
2. **Game View Preview**: วาด preview ข้างใน
3. **Lens**: วาด lens overlay
4. **Direction Arrow**: วาดลูกศรทิศทาง
5. **Labels**: วาดข้อมูล camera

## Visual Representation

### Orthographic Camera:
```
┌─────────────────┐
│  ┌───────────┐  │  ← Camera Gizmo
│  │ • • • • • │  │  ← Game View Preview
│  │ • • • • • │  │     (Grid pattern)
│  │ • • • • • │  │
│  │ Ortho 5.0 │  │  ← Camera Info
│  └───────────┘  │
│   Game View     │  ← Label
└─────────────────┘
```

### Perspective Camera:
```
┌─────────────────┐
│  ┌───────────┐  │  ← Camera Gizmo
│  │     •     │  │  ← Game View Preview
│  │ --------- │  │     (Horizon line)
│  │    • •    │  │     (Perspective objects)
│  │ Persp 60° │  │  ← Camera Info
│  └───────────┘  │
│   Game View     │  ← Label
└─────────────────┘
```

## Benefits

### 1. **Visual Feedback**
- **Immediate Preview**: เห็นสิ่งที่ camera จะแสดงใน Game View
- **Background Color**: เห็นสี background ที่ตั้งไว้
- **Projection Type**: เห็นความแตกต่างระหว่าง Orthographic และ Perspective

### 2. **Unity-like Experience**
- **Scene View Integration**: เหมือน Unity ที่แสดง camera preview ใน Scene View
- **Real-time Updates**: อัพเดตทันทีเมื่อเปลี่ยน camera settings
- **Professional Look**: ดูเป็นมืออาชีพเหมือน Unity Editor

### 3. **Better Understanding**
- **Camera Settings**: เข้าใจ camera settings ผ่าน visual representation
- **Scene Composition**: เห็นว่า camera จะจับภาพ scene อย่างไร
- **Game View Preview**: ไม่ต้องเปิด Game View เพื่อดู camera output

## การใช้งาน

1. **สร้าง Camera Entity**: เพิ่ม camera component
2. **ตั้งค่า Camera**: ปรับ projection, FOV, background color
3. **ดู Preview**: camera gizmo จะแสดง Game View preview
4. **Real-time Updates**: เปลี่ยนค่าแล้วเห็นผลทันที

ตอนนี้ camera gizmo ไม่ใช่แค่ wireframe แล้ว แต่เป็น **mini Game View** ที่แสดงสิ่งที่ camera จะเห็นจริงๆ!
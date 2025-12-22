# Camera Gizmo Cleanup และ Simplification

## ปัญหาที่แก้ไข

**ปัญหา**: Camera gizmo มีเส้นแปลกๆ และดูยุ่งเหยิง
- เส้น wireframe เยอะเกินไป
- Game View preview ไม่ชัดเจน
- เส้นทับซ้อนกันทำให้ดูสับสน

## การแก้ไขที่ทำ

### 1. **ลดความซับซ้อนของ Wireframe**
```rust
// เดิม: Wireframe cube ที่ซับซ้อน (12 เส้น)
// Front face (4 เส้น) + Back face (4 เส้น) + Connecting lines (4 เส้น)

// ใหม่: Rectangle outline เรียบง่าย (4 เส้น)
let corners = [top_left, top_right, bottom_right, bottom_left];
painter.line_segment([corners[0], corners[1]], outline_stroke);
painter.line_segment([corners[1], corners[2]], outline_stroke);
painter.line_segment([corners[2], corners[3]], outline_stroke);
painter.line_segment([corners[3], corners[0]], outline_stroke);
```

### 2. **ปรับปรุง Game View Preview**
```rust
// เดิม: Preview เล็กๆ (30% x 20%)
let preview_width = size * 0.3;
let preview_height = size * 0.2;

// ใหม่: Preview ใหญ่ขึ้น (80% x 80% ของ gizmo)
let preview_width = gizmo_width * 0.8;
let preview_height = gizmo_height * 0.8;
```

### 3. **ลดรายละเอียดใน Scene Content**
```rust
// เดิม: Scene representation ที่ซับซ้อน
// - Grid pattern หลายจุด
// - Horizon line + perspective objects
// - ข้อมูลซ้ำซ้อน

// ใหม่: Scene representation ที่เรียบง่าย
// Orthographic: จุดเล็กๆ แบบ grid
for i in 1..6 {
    for j in 1..4 {
        painter.circle_filled(pos, 0.8, white_transparent);
    }
}

// Perspective: Horizon line + จุดเล็กๆ
painter.line_segment([horizon_line], white_stroke);
painter.circle_filled(center, 1.5, white_transparent);
```

### 4. **ลบเส้นขอบที่ไม่จำเป็น**
```rust
// เดิม: มีขอบรอบ Game View preview
egui::Stroke::new(1.0, gray_color)

// ใหม่: ไม่มีขอบ (เรียบง่าย)
egui::Stroke::NONE
```

## Visual Improvements

### 1. **Clean Rectangle Design**
- **Simple Outline**: เฉพาะ 4 เส้นขอบ
- **No Complex Wireframe**: ไม่มี 3D cube ที่ซับซ้อน
- **Clear Boundaries**: ขอบเขตชัดเจน

### 2. **Prominent Game View**
- **Larger Preview**: ใช้พื้นที่ 80% ของ gizmo
- **Clean Background**: ใช้สี background จาก camera component
- **No Border**: ไม่มีขอบรบกวน

### 3. **Minimal Scene Content**
- **Simple Dots**: จุดเล็กๆ แทน objects ซับซ้อน
- **Subtle Colors**: สีขาวโปร่งใสไม่รบกวน
- **Essential Info Only**: แสดงเฉพาะข้อมูลสำคัญ

### 4. **Consistent Design**
- **Same Style**: Orthographic และ Perspective ใช้สไตล์เดียวกัน
- **Unified Colors**: สีที่สอดคล้องกัน
- **Clean Typography**: ข้อความเรียบง่าย

## ผลลัพธ์ที่ได้

### Before (ปัญหา):
```
┌─────────────────────────┐
│ ╔═══╗     ╔═══╗        │  ← เส้นเยอะ
│ ║ • ║─────║ • ║        │  ← ซับซ้อน
│ ║ • ║  •  ║ • ║        │  ← ยุ่งเหยิง
│ ╚═══╝─────╚═══╝        │
│   │         │          │
│   └─────────┘          │
└─────────────────────────┘
```

### After (แก้ไขแล้ว):
```
┌─────────────────────────┐
│ ┌─────────────────────┐ │  ← เส้นเรียบง่าย
│ │ • • • • •           │ │  ← Game View ใหญ่
│ │ • • • • •  Ortho 5.0│ │  ← เนื้อหาเรียบง่าย
│ │ • • • • •           │ │
│ └─────────────────────┘ │
└─────────────────────────┘
```

## Benefits

### 1. **Visual Clarity**
- **Less Clutter**: เส้นน้อยลง ดูเรียบร้อย
- **Clear Purpose**: เห็นได้ชัดว่าเป็น Game View preview
- **Better Focus**: สายตาไปที่ Game View content

### 2. **Performance**
- **Fewer Draw Calls**: วาดเส้นน้อยลง
- **Simpler Calculations**: คำนวณง่ายขึ้น
- **Faster Rendering**: render เร็วขึ้น

### 3. **Unity-like Appearance**
- **Professional Look**: ดูเป็นมืออาชีพ
- **Familiar Design**: คล้าย Unity Editor
- **Clean Interface**: UI ที่เรียบง่าย

### 4. **Better User Experience**
- **Easy to Understand**: เข้าใจง่าย
- **Less Distraction**: ไม่มีสิ่งรบกวน
- **Clear Information**: ข้อมูลชัดเจน

## การใช้งาน

ตอนนี้ camera gizmo จะ:
1. **แสดง Game View preview ที่ใหญ่และชัดเจน**
2. **ใช้เส้นขอบเรียบง่าย (4 เส้น)**
3. **แสดงเนื้อหา scene แบบเรียบง่าย**
4. **ไม่มีเส้นรบกวนหรือรายละเอียดเกินจำเป็น**

Camera gizmo ตอนนี้เรียบง่าย ชัดเจน และเป็นมืออาชีพเหมือน Unity แล้ว!
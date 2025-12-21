# Unity Camera Wireframe Gizmo Fix

## ปัญหาที่พบ

จากภาพ Unity ที่ผู้ใช้แสดง พบว่า camera gizmo ใน Unity มีลักษณะเป็น:

1. **Wireframe Cube**: ไม่ใช่ trapezoid แต่เป็น cube wireframe
2. **Camera Body**: มีรูปร่างเป็น rectangular box
3. **Lens**: เป็นวงกลมหรือสี่เหลี่ยมเล็กๆ ที่ด้านหน้า
4. **Direction**: มีลูกศรแสดงทิศทางที่ camera มอง

## Unity Camera Gizmo ที่ถูกต้อง

### Orthographic Camera:
- **Wireframe Cube**: กรอบลวดรูปสี่เหลี่ยม
- **Front Face**: หน้าที่ใกล้ viewer
- **Back Face**: หน้าที่ไกล viewer (offset ไปทาง forward)
- **Connecting Lines**: เส้นเชื่อมระหว่าง front และ back face
- **Lens**: สี่เหลี่ยมเล็กๆ ที่ center

### Perspective Camera:
- **Similar Wireframe**: เหมือน orthographic แต่เล็กกว่า
- **Circular Lens**: เป็นวงกลมแทนสี่เหลี่ยม
- **Perspective Hint**: รูปร่างที่บ่งบอกถึง perspective projection

## การแก้ไขที่ทำ

### 1. **เปลี่ยนจาก Trapezoid เป็น Wireframe Cube**
```rust
// เดิม: Trapezoid (ผิด)
let points = vec![
    rotate_point(-front_width / 2.0, -front_height / 2.0),
    rotate_point(-front_width / 2.0, front_height / 2.0),
    rotate_point(back_offset, back_height / 2.0),
    rotate_point(back_offset, -back_height / 2.0),
];

// ใหม่: Wireframe Cube (ถูกต้อง)
// Front face
painter.line_segment([front_tl, front_tr], stroke);
painter.line_segment([front_tr, front_br], stroke);
painter.line_segment([front_br, front_bl], stroke);
painter.line_segment([front_bl, front_tl], stroke);

// Back face + connecting lines
```

### 2. **Unity-style Dimensions**
```rust
// Camera body dimensions (Unity-like proportions)
let cube_width = size * 0.6;   // Width
let cube_height = size * 0.4;  // Height  
let cube_depth = size * 0.8;   // Depth (forward direction)
let back_offset = cube_depth * 0.3; // Back face offset
```

### 3. **Proper Lens Rendering**
```rust
// Orthographic: Square lens
let lens_size = size * 0.15;
// Draw square lens outline

// Perspective: Circular lens
let lens_radius = size * 0.12;
painter.circle_stroke(center, lens_radius, stroke);
```

### 4. **Wireframe Structure**
```rust
// Front face (4 lines)
painter.line_segment([front_tl, front_tr], stroke);
painter.line_segment([front_tr, front_br], stroke);
painter.line_segment([front_br, front_bl], stroke);
painter.line_segment([front_bl, front_tl], stroke);

// Back face (4 lines)
painter.line_segment([back_tl, back_tr], stroke);
painter.line_segment([back_tr, back_br], stroke);
painter.line_segment([back_br, back_bl], stroke);
painter.line_segment([back_bl, back_tl], stroke);

// Connecting lines (4 lines for depth)
painter.line_segment([front_tl, back_tl], stroke);
painter.line_segment([front_tr, back_tr], stroke);
painter.line_segment([front_bl, back_bl], stroke);
painter.line_segment([front_br, back_br], stroke);
```

## Unity-like Visual Features

### 1. **Wireframe Style**
- **No Fill**: เฉพาะเส้นขอบ ไม่มีการเติมสี
- **Consistent Stroke**: เส้นขอบหนา 2px สีเหลือง
- **Clean Lines**: เส้นตรงชัดเจน ไม่มีการเบลอ

### 2. **3D Perspective**
- **Depth Indication**: back face offset แสดงความลึก
- **Proper Proportions**: อัตราส่วนที่เหมาะสมเหมือน Unity
- **Rotation Support**: หมุนตาม camera transform

### 3. **Lens Visualization**
- **Front Position**: lens อยู่ที่ center ของ front face
- **Appropriate Size**: ขนาดที่เหมาะสมไม่ใหญ่เกินไป
- **Different Styles**: square สำหรับ orthographic, circle สำหรับ perspective

### 4. **Direction Arrow**
- **Clear Direction**: ลูกศรสีแดงชี้ทิศทางที่ camera มอง
- **Proper Length**: ยาวพอที่จะเห็นได้ชัด
- **Arrow Head**: หัวลูกศรชัดเจน

## ผลลัพธ์ที่คาดหวัง

### Visual Appearance:
1. **Wireframe Cube**: เหมือน Unity camera gizmo
2. **Proper Orientation**: หันไปทางที่ถูกต้องตาม rotation
3. **Clean Rendering**: เส้นขอบชัดเจน ไม่มีการเติมสี
4. **Appropriate Size**: ขนาดที่เหมาะสมไม่ใหญ่เกินไป

### Behavior:
1. **Rotation**: หมุนตาม camera transform rotation
2. **Direction Arrow**: ชี้ไปทางที่ camera มอง
3. **Lens Position**: อยู่ที่ตำแหน่งที่ถูกต้อง
4. **Unity Consistency**: เหมือนกับ Unity Scene View

## การทดสอบ

1. **สร้าง Camera Entity**: ตั้ง rotation = (0, 0, 0)
2. **ตรวจสอบ Wireframe**: ควรเห็น cube wireframe สีเหลือง
3. **ตรวจสอบ Lens**: ควรเห็น lens ที่ center
4. **ตรวจสอบ Direction**: ลูกศรสีแดงควรชี้ไปทางขวา (+X)
5. **หมุน Camera**: เปลี่ยน rotation และดูว่า gizmo หมุนตาม

ตอนนี้ camera gizmo ควรจะเหมือน Unity แล้ว - เป็น wireframe cube ที่มี lens และ direction arrow ที่ถูกต้อง!
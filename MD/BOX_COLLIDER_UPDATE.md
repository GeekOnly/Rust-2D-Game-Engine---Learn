# Box Collider 2D Update - Unity-like System

## สรุปการอัปเดต

อัปเดต Box Collider 2D ให้มีระบบเหมือน Unity:

### 1. เปลี่ยนจาก Width/Height เป็น Offset + Size

**ก่อน:**
```rust
pub struct Collider {
    pub width: f32,   // ขนาดจริงในโลก
    pub height: f32,
}
```

**หลัง:**
```rust
pub struct Collider {
    pub offset: [f32; 2],  // ตำแหน่งเยื้องจากศูนย์กลาง
    pub size: [f32; 2],    // ขนาดสัมพัทธ์ (default = 1.0)
    // Legacy fields for backward compatibility
    pub width: f32,
    pub height: f32,
}
```

### 2. ใช้ Transform.scale สำหรับขนาดจริง

**ขนาดจริงของ Collider:**
- World Width = `size.x * transform.scale.x`
- World Height = `size.y * transform.scale.y`

**ตัวอย่าง:**
```rust
// Transform
transform.scale = [32.0, 32.0, 1.0]

// Collider
collider.size = [1.0, 1.0]

// ขนาดจริง = 32 x 32 pixels
```

### 3. อัปเดต Inspector UI

**ฟีเจอร์ใหม่:**
- **Edit Collider** button (🔧)
- **Offset** fields (X, Y) - ตำแหน่งเยื้อง
- **Size** fields (X, Y) - ขนาดสัมพัทธ์
- **World size info** - แสดงขนาดจริง (Size × Transform.scale)

**ตัวอย่าง UI:**
```
Edit Collider  [🔧]
Offset    X [0.00]  Y [0.00]
Size      X [1.00]  Y [1.00]

💡 World size: 32.00 x 32.00 (Size × Transform.scale)
```

### 4. Helper Methods

```rust
impl Collider {
    // สร้าง collider ใหม่
    pub fn new(size_x: f32, size_y: f32) -> Self
    
    // สร้างพร้อม offset
    pub fn with_offset(offset_x: f32, offset_y: f32, size_x: f32, size_y: f32) -> Self
    
    // คำนวณขนาดจริง
    pub fn get_world_width(&self, scale_x: f32) -> f32
    pub fn get_world_height(&self, scale_y: f32) -> f32
    
    // คำนวณ offset จริง
    pub fn get_world_offset(&self, scale_x: f32, scale_y: f32) -> [f32; 2]
    
    // Migration จาก legacy
    pub fn migrate_from_legacy(&mut self, transform_scale: [f32; 3])
}
```

### 5. ไฟล์ที่แก้ไข

**Core:**
- `ecs/src/lib.rs` - อัปเดต Collider struct
- `ecs/src/component_manager.rs` - ใช้ Collider::default()

**Rendering:**
- `engine/src/editor/ui/scene_view/rendering/gizmos.rs` - render collider ด้วย offset + size

**UI:**
- `engine/src/editor/ui/inspector.rs` - อัปเดต Box Collider UI

**Transform:**
- `engine/src/editor/ui/scene_view/interaction/transform.rs` - ลบการ scale collider

**Examples:**
- `engine/src/main.rs` - ใช้ Collider::default()
- `projects/Celeste Demo/scenes/main.json` - เพิ่ม offset และ size

## วิธีใช้งาน

### สร้าง Collider ใหม่

```rust
// วิธีที่ 1: ใช้ default (size = 1.0)
world.colliders.insert(entity, Collider::default());

// วิธีที่ 2: กำหนด size
world.colliders.insert(entity, Collider::new(1.0, 1.0));

// วิธีที่ 3: กำหนด offset และ size
world.colliders.insert(entity, Collider::with_offset(
    0.0, 0.5,  // offset X, Y
    1.0, 2.0   // size X, Y
));
```

### ปรับขนาด Collider

```rust
// ปรับขนาดผ่าน Transform.scale (Unity-like)
if let Some(transform) = world.transforms.get_mut(&entity) {
    transform.scale[0] = 64.0; // Width
    transform.scale[1] = 64.0; // Height
}

// หรือปรับ collider.size (สัมพัทธ์)
if let Some(collider) = world.colliders.get_mut(&entity) {
    collider.size[0] = 2.0; // 2x ของ transform.scale
    collider.size[1] = 1.5; // 1.5x ของ transform.scale
}
```

### ปรับ Offset

```rust
// เยื้อง collider จากศูนย์กลาง
if let Some(collider) = world.colliders.get_mut(&entity) {
    collider.offset[0] = 0.0;  // X offset
    collider.offset[1] = 0.5;  // Y offset (เยื้องขึ้น)
}
```

### คำนวณขนาดจริง

```rust
if let Some(collider) = world.colliders.get(&entity) {
    if let Some(transform) = world.transforms.get(&entity) {
        let world_width = collider.get_world_width(transform.scale[0]);
        let world_height = collider.get_world_height(transform.scale[1]);
        println!("Collider size: {} x {}", world_width, world_height);
    }
}
```

## ข้อดี

1. **เหมือน Unity** - ใช้ Offset + Size แทน Width/Height
2. **ยืดหยุ่น** - ปรับ offset ได้อิสระ
3. **สัมพัทธ์กับ Transform** - ขนาดจริง = Size × Scale
4. **Backward Compatible** - รองรับ legacy width/height ด้วย migration

## Migration Guide

### จาก Legacy Collider

```rust
// เก่า
world.colliders.insert(entity, Collider {
    width: 32.0,
    height: 32.0,
});

// ใหม่ (วิธีที่ 1: ใช้ Transform.scale)
world.transforms.insert(entity, Transform {
    scale: [32.0, 32.0, 1.0],
    ...
});
world.colliders.insert(entity, Collider::default()); // size = 1.0

// ใหม่ (วิธีที่ 2: ใช้ collider.size)
world.colliders.insert(entity, Collider::new(32.0, 32.0));
```

### Auto Migration

Collider จะ migrate อัตโนมัติเมื่อเปิดใน Inspector:

```rust
// ถ้ามี legacy width/height
collider.width = 32.0;
collider.height = 32.0;

// จะถูก convert เป็น
collider.size = [32.0 / transform.scale.x, 32.0 / transform.scale.y];
collider.width = 0.0;
collider.height = 0.0;
```

## ตัวอย่างการใช้งาน

### Collider ปกติ (ขนาดเท่า Sprite)

```rust
// Entity ขนาด 32x32
world.transforms.insert(entity, Transform {
    scale: [32.0, 32.0, 1.0],
    ...
});
world.sprites.insert(entity, Sprite::default());
world.colliders.insert(entity, Collider::default()); // size = 1.0
// ขนาดจริง = 32 x 32
```

### Collider ที่เล็กกว่า Sprite

```rust
// Sprite 32x32, Collider 24x24
world.transforms.insert(entity, Transform {
    scale: [32.0, 32.0, 1.0],
    ...
});
world.colliders.insert(entity, Collider::new(0.75, 0.75));
// ขนาดจริง = 24 x 24 (32 * 0.75)
```

### Collider ที่เยื้อง (เช่น ตัวละคร)

```rust
// Collider เยื้องลงมา (ไม่นับหัว)
world.transforms.insert(entity, Transform {
    scale: [32.0, 48.0, 1.0], // สูง 48
    ...
});
world.colliders.insert(entity, Collider::with_offset(
    0.0, -0.25,  // เยื้องลง 25%
    1.0, 0.75    // สูง 75% (36 pixels)
));
// ขนาดจริง = 32 x 36, เยื้องลง 12 pixels
```

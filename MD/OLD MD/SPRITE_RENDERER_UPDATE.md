# Sprite Renderer Update - Unity-like Features

## สรุปการอัปเดต

อัปเดต Sprite Renderer ให้มีฟีเจอร์เหมือน Unity:

### 1. ใช้ Transform.scale สำหรับขนาด Sprite

**ก่อน:**
- Sprite มี `width` และ `height` ที่ใช้สำหรับขนาดจริง
- ต้องแก้ไข sprite.width/height เพื่อปรับขนาด

**หลัง:**
- Sprite มี `width` และ `height` เป็นขนาดต้นฉบับ (base size = 1.0)
- ใช้ `Transform.scale` สำหรับการปรับขนาด (เหมือน Unity)
- ขนาดจริง = `width * scale.x` และ `height * scale.y`

### 2. เพิ่ม Flip X/Y

เพิ่มฟีลด์ใหม่ใน Sprite:
- `flip_x: bool` - พลิกแนวนอน
- `flip_y: bool` - พลิกแนวตั้ง

### 3. อัปเดต Inspector UI

**ฟีเจอร์ใหม่:**
- Sprite picker (คลิกเพื่อเลือก sprite)
- Manual text edit (เมนู ⋮)
- Flip X/Y checkboxes
- Draw Mode dropdown (Simple/Sliced/Tiled)
- ปุ่ม "Open Sprite Editor"
- ข้อความแจ้ง "Use Transform Scale to resize sprite"

### 4. ไฟล์ที่แก้ไข

**Core:**
- `ecs/src/lib.rs` - อัปเดต Sprite struct
- `ecs/src/component_manager.rs` - อัปเดต default sprite

**Rendering:**
- `engine/src/editor/ui/scene_view/rendering/view_2d.rs` - ใช้ Transform.scale
- `engine/src/editor/ui/scene_view/rendering/view_3d.rs` - ใช้ Transform.scale
- `engine/src/runtime/renderer.rs` - ใช้ Transform.scale

**UI:**
- `engine/src/editor/ui/inspector.rs` - อัปเดต Sprite Renderer UI
- `engine/src/editor/ui/hierarchy.rs` - อัปเดต sprite creation

**Transform:**
- `engine/src/editor/ui/scene_view/interaction/transform.rs` - ลบการ scale sprite.width/height

**Examples:**
- `engine/src/main.rs` - อัปเดต sprite creation
- `engine/src/editor/states.rs` - อัปเดต sprite creation
- `projects/Celeste Demo/scenes/main.json` - เพิ่ม flip_x/flip_y

## วิธีใช้งาน

### สร้าง Sprite ใหม่

```rust
// สร้าง sprite entity
let entity = world.spawn();

// ตั้งค่า Transform (ใช้ scale สำหรับขนาด)
world.transforms.insert(entity, Transform {
    position: [0.0, 0.0, 0.0],
    rotation: [0.0, 0.0, 0.0],
    scale: [32.0, 32.0, 1.0], // ขนาด 32x32 pixels
});

// สร้าง Sprite (base size = 1.0)
world.sprites.insert(entity, Sprite {
    texture_id: "player".to_string(),
    width: 1.0,  // Base size
    height: 1.0,
    color: [1.0, 1.0, 1.0, 1.0],
    billboard: false,
    flip_x: false,
    flip_y: false,
});
```

### ปรับขนาด Sprite

```rust
// ปรับขนาดผ่าน Transform.scale (Unity-like)
if let Some(transform) = world.transforms.get_mut(&entity) {
    transform.scale[0] = 64.0; // Width
    transform.scale[1] = 64.0; // Height
}
```

### Flip Sprite

```rust
// พลิก sprite
if let Some(sprite) = world.sprites.get_mut(&entity) {
    sprite.flip_x = true; // พลิกแนวนอน
    sprite.flip_y = false;
}
```

## ข้อดี

1. **เหมือน Unity** - ใช้ Transform.scale สำหรับขนาด
2. **ง่ายต่อการใช้งาน** - ไม่ต้องแก้ไข sprite.width/height
3. **รองรับ Flip** - สามารถพลิก sprite ได้ง่าย
4. **UI ที่ดีขึ้น** - มี sprite picker และ flip controls

## Migration Guide

หากมี sprite เก่าที่ใช้ width/height:

```rust
// เก่า
world.sprites.insert(entity, Sprite {
    width: 32.0,
    height: 32.0,
    ...
});

// ใหม่
world.transforms.insert(entity, Transform {
    scale: [32.0, 32.0, 1.0], // ย้ายขนาดไปที่ scale
    ...
});
world.sprites.insert(entity, Sprite {
    width: 1.0,  // Base size
    height: 1.0,
    flip_x: false,
    flip_y: false,
    ...
});
```

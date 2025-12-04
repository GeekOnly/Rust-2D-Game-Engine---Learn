# Tilemap Collider Usage

## การสร้าง Colliders จาก LDtk IntGrid

LDtk Loader รองรับการสร้าง colliders อัตโนมัติจาก IntGrid layer

### วิธีใช้งาน

```rust
use ecs::{World, loaders::LdtkLoader};

let mut world = World::new();

// 1. Load tilemap (visual)
let tilemap_entities = LdtkLoader::load_project(
    "levels/Level_01.ldtk",
    &mut world
)?;

// 2. Generate colliders from IntGrid (collision_value = 1 means solid)
let collider_entities = LdtkLoader::generate_colliders_from_intgrid(
    "levels/Level_01.ldtk",
    &mut world,
    1  // IntGrid value for solid tiles
)?;

println!("Created {} tilemap entities", tilemap_entities.len());
println!("Created {} collider entities", collider_entities.len());
```

### IntGrid Values

ใน LDtk editor:
- **Value 0**: ว่าง (ไม่มี collision)
- **Value 1**: กำแพง/พื้น (มี collision)

### Collider Properties

แต่ละ collider entity มี:
- **Transform**: ตำแหน่งที่ศูนย์กลางของ tile
- **Collider**: ขนาด 1x1 world unit (= 8x8 pixels)
- **Rigidbody2D**: kinematic (ไม่เคลื่อนที่, แต่มี collision)

### Performance

- แต่ละ solid tile = 1 collider entity
- สำหรับแผนที่ขนาด 37x26 tiles อาจมี colliders หลายร้อยตัว
- ในอนาคตควรใช้ Composite Collider เพื่อรวม tiles ที่ติดกันเป็น box ใหญ่

### ตัวอย่างใน Scene

```json
{
  "tilemaps": [...],
  "colliders": [
    [entity_id, {
      "offset": [0.0, 0.0],
      "size": [1.0, 1.0]
    }]
  ],
  "rigidbodies": [
    [entity_id, {
      "velocity": (0.0, 0.0),
      "gravity_scale": 0.0,
      "is_kinematic": true
    }]
  ]
}
```

### การทดสอบ

1. Load scene ที่มี tilemap
2. เรียก `generate_colliders_from_intgrid()`
3. Character ควรจะยืนบนพื้นและชนกับกำแพงได้

### TODO

- [ ] Composite Collider (รวม tiles ติดกันเป็น box ใหญ่)
- [ ] One-way platforms (แพลตฟอร์มที่กระโดดผ่านได้)
- [ ] Slope colliders (ทางลาด)

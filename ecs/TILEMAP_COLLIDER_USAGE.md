# Tilemap Collider Usage

## การสร้าง Colliders จาก LDtk IntGrid

LDtk Loader รองรับการสร้าง colliders อัตโนมัติจาก IntGrid layer

### วิธีใช้งาน

#### แบบ Simple (1 collider per tile)
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

#### แบบ Composite (Greedy Meshing - แนะนำ)
```rust
use ecs::{World, loaders::LdtkLoader};

let mut world = World::new();

// 1. Load tilemap (visual)
let tilemap_entities = LdtkLoader::load_project(
    "levels/Level_01.ldtk",
    &mut world
)?;

// 2. Generate optimized composite colliders
let collider_entities = LdtkLoader::generate_composite_colliders_from_intgrid(
    "levels/Level_01.ldtk",
    &mut world,
    1  // IntGrid value for solid tiles
)?;

println!("Created {} tilemap entities", tilemap_entities.len());
println!("Created {} composite colliders", collider_entities.len());
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

### Performance Comparison

#### Simple Colliders (1 per tile)
- ✅ ง่าย, ตรงไปตรงมา
- ❌ จำนวน colliders เยอะมาก
- ❌ ช้าในการ collision detection
- **ตัวอย่าง**: แผนที่ 37x26 tiles → ~500 colliders

#### Composite Colliders (Greedy Meshing)
- ✅ จำนวน colliders น้อยมาก (ลดได้ 80-95%)
- ✅ เร็วในการ collision detection
- ✅ ใช้ memory น้อยกว่า
- ✅ หา rectangles ที่ใหญ่ที่สุดโดยอัตโนมัติ
- **ตัวอย่าง**: แผนที่ 37x26 tiles → ~50-100 colliders

### Greedy Meshing Algorithm

Algorithm นี้ทำงานโดย:
1. สแกนแต่ละ tile จากซ้ายไปขวา, บนลงล่าง
2. เมื่อเจอ solid tile, ลองขยายเป็น rectangle 2 แบบ:
   - แบบที่ 1: ขยายแนวนอนก่อน แล้วค่อยขยายแนวตั้ง
   - แบบที่ 2: ขยายแนวตั้งก่อน แล้วค่อยขยายแนวนอน
3. เลือก rectangle ที่มี area ใหญ่กว่า
4. ทำเครื่องหมาย tiles ที่ใช้แล้ว
5. ทำซ้ำจนครบทุก tile

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

### Visualization

```
Before (Simple):          After (Composite):
████████████████         ┌──────────────┐
█ █ █ █ █ █ █ █         │              │
████████████████         │              │
█ █ █ █ █ █ █ █   →     │              │
████████████████         │              │
█ █ █ █ █ █ █ █         └──────────────┘

64 colliders             1 collider
```

### Performance Tips

1. **ใช้ Composite Colliders เสมอ** สำหรับ production
2. **ใช้ Simple Colliders** เฉพาะเมื่อ debug หรือทดสอบ
3. **ตรวจสอบ log** เพื่อดูจำนวน colliders ที่สร้าง
4. **Reload map** เพื่อ cleanup colliders เก่าก่อนสร้างใหม่

### Completed Features

- [x] Simple Collider Generation (1 per tile) ✅
- [x] Composite Collider Generation ✅
- [x] Greedy Meshing Algorithm ✅
- [x] UI Integration (Map Inspector) ✅
- [x] Automatic Cleanup on Reload ✅

### Future Improvements

- [ ] One-way platforms (แพลตฟอร์มที่กระโดดผ่านได้)
- [ ] Slope colliders (ทางลาด)
- [ ] Edge colliders (สำหรับ outline เท่านั้น)
- [ ] Collider visualization in editor
- [ ] Custom collision layers

# LDtk Export Guide - คู่มือการ Export และใช้งาน

## 🎨 การสร้างและ Export LDtk

### 1. สร้าง Project ใหม่ใน LDtk

```
1. เปิด LDtk Editor
2. File > New Project
3. เลือกที่เก็บ: projects/Celeste Demo/levels/
4. ตั้งชื่อ: world.ldtk
5. คลิก Save
```

### 2. ตั้งค่า Project

#### Project Settings (F1)

```
Project Settings:
├─ Default grid size: 16x16 (หรือตามต้องการ)
├─ Export: 
│  └─ Save to disk: ✓ (auto-save)
└─ External levels: ❌ (ใช้ single file ก่อน)
```

#### Tilesets

```
1. คลิก "Tilesets" tab
2. Add tileset > Browse
3. เลือก: projects/Celeste Demo/levels/tilesets/tiles.png
4. ตั้งค่า:
   - Tile size: 16x16
   - Spacing: 0
   - Padding: 0
```

### 3. สร้าง Level

```
1. คลิก "Levels" tab
2. Add level > ตั้งชื่อ "Level_1"
3. ตั้งค่า level:
   - Width: 320 (20 tiles)
   - Height: 180 (11 tiles)
```

### 4. เพิ่ม Layers

```
Layers (จากบนลงล่าง):
├─ Entities (Entity layer)
│  └─ สำหรับวาง player, enemies, items
├─ Collision (IntGrid layer)
│  └─ สำหรับ collision detection
├─ Tiles (Tile layer)
│  └─ สำหรับวาด level
└─ Background (Tile layer)
   └─ สำหรับพื้นหลัง
```

### 5. วาด Level

```
1. เลือก layer "Tiles"
2. เลือก tileset
3. วาด level ของคุณ
4. Save (Ctrl+S)
```

### 6. Export Settings

LDtk **ไม่ต้อง export แยก** - ไฟล์ `.ldtk` เป็น JSON ที่ engine อ่านได้เลย!

```
File > Save (Ctrl+S)
↓
world.ldtk (JSON format)
↓
Engine อ่านได้ทันที!
```

## 🚀 การใช้งานใน Engine

### วิธีที่ 1: Map Component (ง่ายที่สุด)

#### ใน Editor:

```
1. Hierarchy > Create Empty
   └─ ตั้งชื่อ: "Level"

2. Inspector > Add Component > Map
   ├─ Name: Level 1
   ├─ Type: LDtk
   ├─ File: levels/world.ldtk [📁]
   └─ Hot-Reload: ✓

3. คลิก "📥 Load Map"
   └─ Entities จะถูก spawn อัตโนมัติ

4. คลิก "🎨 Open in LDtk"
   └─ แก้ไข level
   └─ Save (Ctrl+S)
   └─ Hot-reload อัตโนมัติ!
```

### วิธีที่ 2: Rust Code

```rust
use ecs::{World, loaders::LdtkLoader};

fn load_level(world: &mut World) {
    // Load LDtk file
    let entities = LdtkLoader::load_project(
        "projects/Celeste Demo/levels/world.ldtk",
        world
    ).expect("Failed to load level");
    
    println!("Loaded {} entities", entities.len());
}
```

### วิธีที่ 3: Hot-Reload Runtime

```rust
use engine::runtime::LdtkRuntime;

let mut ldtk = LdtkRuntime::new();
ldtk.load("levels/world.ldtk", &mut world)?;

// Game loop
loop {
    // Auto hot-reload
    if ldtk.update(&mut world) {
        println!("Level reloaded!");
    }
    
    // Game logic...
}
```

### วิธีที่ 4: Lua Script

```lua
-- ใน on_start()
local ldtk = LdtkRuntime.new()
ldtk:load("levels/world.ldtk")

-- ใน on_update(dt)
if ldtk:update() then
    print("Level reloaded!")
end
```

## 📁 โครงสร้างไฟล์

### แนะนำ:

```
projects/Celeste Demo/
├── levels/
│   ├── world.ldtk              ← Main LDtk file
│   ├── tilesets/
│   │   ├── tiles.png           ← Tileset image
│   │   ├── sprites.png
│   │   └── background.png
│   └── backups/                ← LDtk auto-backups
│       └── world.ldtk.backup
├── scenes/
│   └── main.json               ← Scene with Map component
├── scripts/
│   ├── player_controller.lua
│   └── map_loader.lua          ← Map loading script
└── assets/
    └── textures/
```

### ไฟล์ที่ต้อง Commit (Git):

```gitignore
# Commit these
*.ldtk
*.png
*.json

# Ignore these
*.ldtk.backup
*.ldtk~
```

## 🎯 Workflow แนะนำ

### สำหรับ Level Designer:

```
1. เปิด LDtk Editor
   └─ แก้ไข level

2. Save (Ctrl+S)
   └─ ไฟล์ .ldtk ถูก update

3. เปิด Game Engine
   └─ Hot-reload อัตโนมัติ
   └─ เห็นการเปลี่ยนแปลงทันที

4. Test ใน game
   └─ ถ้าไม่ชอบ กลับไปข้อ 1

5. Commit changes
   └─ git add levels/world.ldtk
   └─ git commit -m "Update level 1"
```

### สำหรับ Programmer:

```rust
// 1. Setup hot-reload
let mut ldtk = LdtkRuntime::new();
ldtk.load("levels/world.ldtk", &mut world)?;

// 2. Game loop
loop {
    // Hot-reload check
    if ldtk.update(&mut world) {
        on_level_reloaded(&mut world);
    }
    
    // Game logic
    update_game(&mut world);
    render(&world);
}

// 3. Handle reload
fn on_level_reloaded(world: &mut World) {
    // Save player state
    let player_pos = get_player_position(world);
    
    // Level reloaded automatically
    
    // Restore player state
    set_player_position(world, player_pos);
}
```

## 🔧 LDtk Settings สำหรับ Engine

### Project Settings ที่แนะนำ:

```json
{
  "defaultGridSize": 16,
  "defaultPivotX": 0,
  "defaultPivotY": 0,
  "exportTiled": false,
  "externalLevels": false,
  "minifyJson": false
}
```

### Layer Types:

1. **Tile Layer** → `Tilemap` component
   - สำหรับวาด tiles
   - Engine render ด้วย TilemapRenderer

2. **IntGrid Layer** → Collision data
   - สำหรับ collision
   - แปลงเป็น Collider components

3. **Entity Layer** → Spawn entities
   - สำหรับวาง objects
   - แปลงเป็น Entity + Components

4. **Auto-Layer** → Generated tiles
   - Auto-tiling rules
   - Engine render เหมือน Tile Layer

## 📊 LDtk Data Format

### ไฟล์ .ldtk เป็น JSON:

```json
{
  "levels": [
    {
      "identifier": "Level_1",
      "pxWid": 320,
      "pxHei": 180,
      "layerInstances": [
        {
          "identifier": "Tiles",
          "__type": "Tiles",
          "__cWid": 20,
          "__cHei": 11,
          "gridTiles": [...]
        }
      ]
    }
  ]
}
```

Engine อ่าน JSON นี้ด้วย `ldtk_rust` crate

## 🎮 ตัวอย่างการใช้งาน

### Example 1: Simple Level Load

```bash
cargo run --example load_ldtk_map
```

### Example 2: Hot-Reload

```bash
cargo run --example ldtk_hot_reload -- levels/world.ldtk
```

### Example 3: ใน Game

```rust
// main.rs
fn main() {
    let mut world = World::new();
    let mut ldtk = LdtkRuntime::new();
    
    // Load level
    ldtk.load("levels/world.ldtk", &mut world)
        .expect("Failed to load level");
    
    // Game loop
    loop {
        ldtk.update(&mut world);
        update_game(&mut world);
        render(&world);
    }
}
```

## 💡 Tips & Tricks

### 1. Dual Monitor Setup

```
Monitor 1: LDtk Editor
Monitor 2: Game Engine

แก้ไข → Save → เห็นผลทันที!
```

### 2. Auto-Save ใน LDtk

```
Settings > Auto-save: ✓
Interval: 30 seconds

ไม่ต้องกด Ctrl+S บ่อยๆ
```

### 3. Backup Strategy

```
LDtk สร้าง backup อัตโนมัติ:
levels/backups/world.ldtk.backup

แต่ควร commit ใน Git บ่อยๆ
```

### 4. Performance

```
ถ้า level ใหญ่มาก:
1. แบ่งเป็นหลาย level files
2. ใช้ external levels
3. Load เฉพาะที่ต้องการ
```

## 🐛 Troubleshooting

### ไฟล์ไม่ load

```
Error: Failed to load LDtk file

แก้ไข:
1. ตรวจสอบ path ถูกต้อง
2. ตรวจสอบไฟล์เป็น valid JSON
3. เปิดไฟล์ใน LDtk ดูว่า corrupt หรือไม่
4. ลอง load backup file
```

### Hot-reload ไม่ทำงาน

```
แก้ไข:
1. ตรวจสอบ hot_reload_enabled = true
2. ตรวจสอบ save สำเร็จใน LDtk
3. ดู console log
4. Restart engine
```

### Tileset ไม่แสดง

```
แก้ไข:
1. ตรวจสอบ tileset path ใน LDtk
2. ตรวจสอบไฟล์ .png มีอยู่
3. ตรวจสอบ texture manager load ไฟล์
4. ดู console สำหรับ texture errors
```

## 📚 Resources

- **LDtk Official**: https://ldtk.io/
- **LDtk Docs**: https://ldtk.io/docs/
- **LDtk Discord**: https://discord.gg/ldtk
- **ldtk_rust**: https://crates.io/crates/ldtk_rust

## 🎓 Next Steps

1. ✅ สร้าง level แรกใน LDtk
2. ✅ Load ใน engine
3. ✅ ทดสอบ hot-reload
4. ✅ เพิ่ม entities และ collision
5. ✅ สร้าง multiple levels
6. ✅ Implement level transitions

---

Happy Level Designing! 🎨✨

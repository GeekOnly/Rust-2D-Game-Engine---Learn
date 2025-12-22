# Map Component - คู่มือการใช้งาน

## 🗺️ Overview

**Map Component** เป็น component สำหรับจัดการ map files (LDtk/Tiled) ใน Game Engine พร้อมฟีเจอร์:
- ✅ เปิดไฟล์ .ldtk หรือ .tmx ใน editor ภายนอก
- ✅ Hot-reload อัตโนมัติ
- ✅ Button เปิด LDtk/Tiled editor โดยตรง
- ✅ Browse และเลือกไฟล์ map
- ✅ แสดงสถานะไฟล์

## 🚀 Quick Start

### 1. เพิ่ม Map Component

1. เลือก Entity ใน Hierarchy
2. คลิก **"➕ Add Component"** ใน Inspector
3. เลือก **"Map"** จากเมนู

### 2. ตั้งค่า Map

ใน Inspector จะมี Map component ปรากฏ:

```
🗺️ Map                                    [✖]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Name: [My Level]
Type: [LDtk ▼]
File: [levels/world.ldtk]  [📁]
✓ File exists

☑ Hot-Reload (Debug mode)

[🎨 Open in LDtk] [📥 Load Map]

ℹ️ Help
```

### 3. เปิดใน LDtk Editor

คลิกปุ่ม **"🎨 Open in LDtk"** เพื่อ:
- เปิดไฟล์ map ใน LDtk editor โดยอัตโนมัติ
- ถ้ายังไม่มี LDtk จะแสดง link download

## 📖 UI Components

### Map Type Selector

เลือกประเภท map:
- **LDtk** - สำหรับไฟล์ .ldtk
- **Tiled** - สำหรับไฟล์ .tmx, .json

### File Path

- **Text Field**: พิมพ์ path ของไฟล์
- **📁 Browse Button**: เปิด file dialog เลือกไฟล์
- **Status**: แสดงว่าไฟล์มีอยู่หรือไม่
  - ✓ File exists (สีเขียว)
  - ✗ File not found (สีแดง)

### Hot-Reload Toggle

- ☑ **Hot-Reload**: เปิด/ปิด hot-reload
- แสดง "(Debug mode)" ถ้าอยู่ใน debug build
- ปิดอัตโนมัติใน production build

### Action Buttons

1. **🎨 Open in LDtk/Tiled**
   - เปิดไฟล์ใน editor ภายนอก
   - ใช้ default application หรือ specific editor

2. **📥 Load Map**
   - Load map เข้า scene
   - Spawn entities จาก map

3. **🔄 Reload** (ถ้า loaded)
   - Reload map ใหม่
   - Update entities

### Help Section

คลิก **"ℹ️ Help"** เพื่อดู:
- ข้อมูล map type
- File extensions ที่รองรับ
- Link download editor
- Hot-reload workflow

## 🎮 Workflow

### แบบ Designer

```
1. สร้าง Entity ใหม่
   └─> Add Map Component

2. Browse เลือกไฟล์ .ldtk
   └─> หรือพิมพ์ path เอง

3. คลิก "Open in LDtk"
   └─> แก้ไข level ใน LDtk

4. Save ใน LDtk (Ctrl+S)
   └─> Hot-reload อัตโนมัติ

5. ดูผลทันทีใน Game
```

### แบบ Programmer

```rust
use ecs::{World, Map, MapType};

// สร้าง entity พร้อม map component
let entity = world.spawn();
world.transforms.insert(entity, Transform::default());
world.names.insert(entity, "Level 1".to_string());

// เพิ่ม map component
let map = Map::ldtk("levels/level_1.ldtk");
world.maps.insert(entity, map);

// หรือใช้ Component Manager
world.add_component(entity, ComponentType::Map)?;
```

## 🔧 Features

### 1. Auto-Open Editor

Engine จะพยายามเปิด editor ด้วยวิธีนี้:
1. ใช้ default application (Windows: file association)
2. ถ้าไม่ได้ จะลองเปิด specific editor
3. ถ้ายังไม่ได้ จะแสดง error และ link download

### 2. File Path Management

- **Relative Path**: แนะนำใช้ relative path จาก project root
- **Absolute Path**: รองรับ แต่ไม่แนะนำ
- **Browse Dialog**: เริ่มที่ project folder
- **Auto-Relative**: แปลง absolute เป็น relative อัตโนมัติ

### 3. Hot-Reload Integration

Map component ทำงานร่วมกับ LdtkRuntime:

```rust
// ใน game loop
if ldtk_runtime.update(&mut world) {
    // Map reloaded!
    // Update game state
}
```

### 4. Multiple Maps

รองรับหลาย map ใน scene เดียว:

```rust
// World map
let world_entity = world.spawn();
world.maps.insert(world_entity, Map::ldtk("levels/world.ldtk"));

// UI layout
let ui_entity = world.spawn();
world.maps.insert(ui_entity, Map::ldtk("levels/ui.ldtk"));
```

## 🎯 Use Cases

### 1. Level Design

```
Entity: "Level 1"
├─ Transform
├─ Map
│  ├─ File: levels/level_1.ldtk
│  ├─ Type: LDtk
│  └─ Hot-Reload: ✓
└─ (Spawned entities from map)
```

### 2. World Map

```
Entity: "World"
├─ Transform
├─ Map
│  ├─ File: levels/world_map.ldtk
│  └─ Type: LDtk
└─ Script: world_controller.lua
```

### 3. UI Layout

```
Entity: "UI Layout"
├─ Transform
└─ Map
   ├─ File: levels/ui_layout.ldtk
   └─ Type: LDtk
```

## 💡 Tips & Best Practices

### 1. File Organization

```
project/
├── levels/
│   ├── world_1/
│   │   ├── level_1.ldtk
│   │   ├── level_2.ldtk
│   │   └── tilesets/
│   └── world_2/
│       └── level_1.ldtk
└── scenes/
    └── main.json
```

### 2. Naming Convention

- **Entity Name**: ตั้งชื่อตาม level
  - "Level 1", "World Map", "Tutorial"
- **File Name**: ใช้ lowercase + underscore
  - `level_1.ldtk`, `world_map.ldtk`

### 3. Hot-Reload Best Practices

```rust
// ปิด hot-reload ใน production
#[cfg(not(debug_assertions))]
map.hot_reload_enabled = false;

// Save state ก่อน reload
if map_reloaded {
    save_player_state();
    // ... reload happens ...
    restore_player_state();
}
```

### 4. Version Control

```gitignore
# Commit map files
*.ldtk
*.tmx

# Ignore backups
*.ldtk.backup
*.ldtk~
*.tmx.backup
```

## 🐛 Troubleshooting

### ไม่สามารถเปิด Editor

**ปัญหา**: คลิก "Open in LDtk" แล้วไม่เกิดอะไร

**แก้ไข**:
1. ตรวจสอบว่าติดตั้ง LDtk แล้ว
2. ตรวจสอบ file association (Windows)
3. ดู console log สำหรับ error
4. Download LDtk จาก https://ldtk.io/

### File Not Found

**ปัญหา**: แสดง "✗ File not found"

**แก้ไข**:
1. ตรวจสอบ path ว่าถูกต้อง
2. ใช้ relative path จาก project root
3. ตรวจสอบว่าไฟล์มีอยู่จริง
4. ใช้ Browse button เลือกไฟล์ใหม่

### Hot-Reload ไม่ทำงาน

**ปัญหา**: แก้ไขไฟล์แล้วไม่ reload

**แก้ไข**:
1. ตรวจสอบว่า Hot-Reload เปิดอยู่
2. ตรวจสอบว่า save สำเร็จใน LDtk
3. ดู console log
4. ลอง Load Map ใหม่

## 📚 Related Documentation

- [LDTK_HOT_RELOAD.md](../ecs/LDTK_HOT_RELOAD.md) - Hot-reload API
- [LDTK_INTEGRATION_GUIDE.md](LDTK_INTEGRATION_GUIDE.md) - Integration guide
- [LDtk Official Docs](https://ldtk.io/docs/)

## 🎓 Tutorial

### สร้าง Level แรก

1. **สร้าง Entity**
   ```
   Hierarchy > Right-click > Create Empty
   ตั้งชื่อ: "Level 1"
   ```

2. **เพิ่ม Map Component**
   ```
   Inspector > Add Component > Map
   ```

3. **สร้างไฟล์ LDtk**
   ```
   - คลิก Browse button
   - Navigate to: project/levels/
   - สร้างไฟล์ใหม่: level_1.ldtk
   ```

4. **เปิดใน LDtk**
   ```
   - คลิก "Open in LDtk"
   - ออกแบบ level
   - Save (Ctrl+S)
   ```

5. **Load ใน Game**
   ```
   - คลิก "Load Map"
   - ดู entities ที่ spawn
   ```

6. **Test Hot-Reload**
   ```
   - แก้ไขใน LDtk
   - Save
   - ดูการเปลี่ยนแปลงทันที
   ```

## 🚀 Advanced

### Custom Map Loader

```rust
// Implement custom map loading logic
impl MapLoader for MyMapLoader {
    fn load(&mut self, map: &Map, world: &mut World) {
        // Custom loading logic
    }
}
```

### Map Events

```rust
// Listen for map events
if map_component_changed {
    on_map_changed(&map);
}

if map_loaded {
    on_map_loaded(&map, &entities);
}
```

---

Happy Level Designing! 🗺️✨

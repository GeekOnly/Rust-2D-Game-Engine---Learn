# Load Map Tutorial - คู่มือการโหลด Map

## 🎯 ปัญหา: กด Load Map แล้วไม่แสดง

### สาเหตุที่เป็นไปได้:

1. ❌ ไฟล์ path ไม่ถูกต้อง
2. ❌ ไฟล์ LDtk ว่างเปล่า (ไม่มี level)
3. ❌ Camera ไม่ได้มองไปที่ map
4. ❌ Tileset ไม่ได้ load
5. ❌ Layer ไม่มี tiles

## ✅ วิธีแก้ไข

### 1. ตรวจสอบ File Path

```
Inspector > Map Component
├─ File: levels/Level_01.ldtk
└─ Status: ✓ File exists  ← ต้องเป็นสีเขียว
```

ถ้าแสดง "✗ File not found":
- ตรวจสอบ path ว่าถูกต้อง
- ใช้ relative path จาก project root
- ลองใช้ Browse button เลือกไฟล์ใหม่

### 2. ตรวจสอบ LDtk File

เปิดไฟล์ `Level_01.ldtk` ใน LDtk Editor:

```
ต้องมี:
✓ อย่างน้อย 1 Level
✓ อย่างน้อย 1 Layer (Tile layer)
✓ มี Tileset ที่ถูกต้อง
✓ มี Tiles วาดอยู่ใน layer
```

### 3. ดู Console Log

เปิด Console ใน Editor:

```
View > Console (หรือ Bottom Panel > Console tab)
```

ดู log messages:
```
✓ Loading map: projects/Celeste Demo/levels/Level_01.ldtk
✓ Loaded 5 entities from map
```

ถ้ามี error:
```
✗ Failed to load map: ...
```

### 4. ตรวจสอบ Entities ที่ Spawn

หลังกด Load Map ดูใน Hierarchy:

```
Hierarchy
├─ Level (Map component)
├─ LDTK Layer: IntGrid_layer  ← Entities ที่ spawn จาก map
├─ LDTK Layer: Tiles
└─ LDTK Layer: Background
```

ถ้าไม่มี entities ใหม่ = map ไม่ได้ load

### 5. ปรับ Camera

Map อาจอยู่นอก camera view:

```
Scene View:
1. เลือก entity ที่ spawn จาก map
2. กด F (Frame Selected)
3. หรือ ปรับ camera position ด้วยตนเอง
```

## 🔧 Debug Steps

### Step 1: ตรวจสอบไฟล์

```bash
# ตรวจสอบว่าไฟล์มีอยู่
dir "projects\Celeste Demo\levels\Level_01.ldtk"
```

### Step 2: ทดสอบ Load ด้วย Example

```bash
cargo run --example load_ldtk_map
```

ถ้า example ทำงาน = ไฟล์ LDtk ถูกต้อง

### Step 3: ดู LDtk File Content

เปิดไฟล์ `.ldtk` ด้วย text editor:

```json
{
  "levels": [
    {
      "identifier": "Level_0",
      "layerInstances": [
        {
          "identifier": "Tiles",
          "gridTiles": [...]  ← ต้องมี tiles
        }
      ]
    }
  ]
}
```

ถ้า `gridTiles` เป็น `[]` = ไม่มี tiles วาด

### Step 4: ตรวจสอบ Tileset

ใน LDtk Editor:

```
1. เปิด Level_01.ldtk
2. ไปที่ Tilesets tab
3. ตรวจสอบว่ามี tileset
4. ตรวจสอบว่า tileset image path ถูกต้อง
```

## 📝 วิธีสร้าง Level ที่ถูกต้อง

### ใน LDtk Editor:

```
1. สร้าง Project ใหม่
   File > New Project
   Save as: projects/Celeste Demo/levels/Level_01.ldtk

2. เพิ่ม Tileset
   Tilesets tab > Add tileset
   Browse: projects/Celeste Demo/assets/tiles.png
   Tile size: 16x16

3. สร้าง Level
   Levels tab > Add level
   Name: Level_0
   Size: 320x180 (20x11 tiles)

4. เพิ่ม Layer
   Layers tab > Add layer
   Type: Tiles
   Name: Tiles
   Tileset: เลือก tileset ที่สร้าง

5. วาด Tiles
   เลือก layer "Tiles"
   เลือก tiles จาก tileset
   วาดใน level

6. Save
   Ctrl+S
```

### ใน Engine:

```
1. เปิด Scene
   projects/Celeste Demo/scenes/main.json

2. สร้าง Entity
   Hierarchy > Create Empty
   Name: "Level"

3. เพิ่ม Map Component
   Inspector > Add Component > Map
   File: levels/Level_01.ldtk
   Type: LDtk

4. Load Map
   คลิก "📥 Load Map"

5. ดู Result
   - ดู Console สำหรับ log
   - ดู Hierarchy สำหรับ entities ใหม่
   - ปรับ Camera ถ้าจำเป็น
```

## 🎮 ตัวอย่างที่ทำงาน

### LDtk File Structure:

```json
{
  "levels": [
    {
      "identifier": "Level_0",
      "pxWid": 320,
      "pxHei": 180,
      "layerInstances": [
        {
          "identifier": "Tiles",
          "__type": "Tiles",
          "__cWid": 20,
          "__cHei": 11,
          "gridTiles": [
            {
              "px": [0, 0],
              "src": [0, 0],
              "t": 0
            }
            // ... more tiles
          ]
        }
      ]
    }
  ]
}
```

### Scene Structure:

```
Hierarchy:
├─ Camera 2D
├─ Player
└─ Level (Map component)
    ├─ Transform: [0, 0, 0]
    └─ Map:
        ├─ File: levels/Level_01.ldtk
        ├─ Type: LDtk
        ├─ Loaded: ✓
        └─ Entities: 3
```

### Expected Result:

```
Console:
[INFO] Loading map: projects/Celeste Demo/levels/Level_01.ldtk
[INFO] ✓ Loaded 3 entities from map

Hierarchy:
├─ Camera 2D
├─ Player
├─ Level
├─ LDTK Layer: Tiles        ← New!
├─ LDTK Layer: Background   ← New!
└─ LDTK Layer: IntGrid      ← New!
```

## 🐛 Common Issues

### Issue 1: "File not found"

```
Error: Map file not found: projects/Celeste Demo/levels/Level_01.ldtk

Fix:
1. ตรวจสอบ path
2. ใช้ forward slash (/) ไม่ใช่ backslash (\)
3. Path ต้อง relative จาก project root
```

### Issue 2: "Failed to parse LDTK JSON"

```
Error: Failed to parse LDTK JSON: ...

Fix:
1. เปิดไฟล์ใน LDtk Editor
2. Save ใหม่ (Ctrl+S)
3. ตรวจสอบว่าไฟล์ไม่ corrupt
```

### Issue 3: "Loaded 0 entities"

```
Info: ✓ Loaded 0 entities from map

Fix:
1. ตรวจสอบว่า level มี layers
2. ตรวจสอบว่า layers มี tiles
3. วาด tiles ใน LDtk Editor
```

### Issue 4: "Entities loaded but not visible"

```
Info: ✓ Loaded 3 entities from map
(แต่ไม่เห็นใน Scene View)

Fix:
1. ปรับ Camera position
2. เลือก entity ที่ spawn แล้วกด F
3. ตรวจสอบ Z-position ของ camera
4. ตรวจสอบว่า tileset texture ถูก load
```

## 💡 Tips

### 1. ใช้ Hot-Reload

```
1. เปิด LDtk Editor
2. เปิด Game Engine
3. แก้ไข level ใน LDtk
4. Save (Ctrl+S)
5. ดูการเปลี่ยนแปลงทันทีใน Engine
```

### 2. Debug Camera Position

```rust
// ใน Scene View
Camera position: [0, 0, -10]
Map position: [0, 0, 0]

// ถ้า map อยู่ที่ [1000, 1000, 0]
// Camera จะมองไม่เห็น!
```

### 3. Check Spawned Entities

```
Hierarchy > เลือก entity ที่ spawn
Inspector > ดู components:
├─ Transform: position, rotation, scale
├─ Tilemap: width, height, tiles
└─ Name: "LDTK Layer: ..."
```

### 4. Verify Tileset

```
1. ใน LDtk: ตรวจสอบ tileset path
2. ใน Engine: ตรวจสอบว่า texture ถูก load
3. Console: ดู texture loading messages
```

## 🎓 Complete Example

### 1. สร้าง Level ใน LDtk

```
File > New Project
Location: projects/Celeste Demo/levels/
Name: Level_01.ldtk

Tilesets:
└─ Add tileset: assets/tiles.png (16x16)

Levels:
└─ Add level: Level_0 (320x180)

Layers:
├─ Tiles (Tile layer)
└─ Background (Tile layer)

วาด tiles ใน Tiles layer
Save (Ctrl+S)
```

### 2. Load ใน Engine

```
Scene: main.json

Hierarchy:
└─ Create Empty > "Level"

Inspector:
└─ Add Component > Map
   ├─ File: levels/Level_01.ldtk
   └─ Type: LDtk

คลิก "📥 Load Map"

Result:
├─ Console: "✓ Loaded 2 entities"
└─ Hierarchy: มี entities ใหม่
```

### 3. Verify

```
Scene View:
1. เลือก "LDTK Layer: Tiles"
2. กด F (Frame Selected)
3. เห็น tiles ที่วาด!
```

---

ถ้ายังมีปัญหา ให้:
1. ดู Console log
2. ตรวจสอบ LDtk file ใน text editor
3. ลอง example: `cargo run --example load_ldtk_map`
4. สร้าง level ใหม่ทดสอบ

Happy Mapping! 🗺️✨

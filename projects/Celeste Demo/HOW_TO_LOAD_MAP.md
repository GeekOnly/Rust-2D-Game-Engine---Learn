# วิธีโหลด Map สำหรับ Celeste Demo

## ✅ ไฟล์ของคุณ

```
projects/Celeste Demo/levels/Level_01.ldtk
```

ไฟล์นี้มี:
- ✓ Level: "AutoLayer" (296x208 pixels)
- ✓ Layer: "IntGrid_layer" (37x26 tiles, 8x8 grid)
- ✓ Tileset: "atlas/Cavernas_by_Adam_Saltsman.png"
- ✓ Data: มี intGrid data (collision/tiles)

## 🚀 วิธีโหลดใน Engine

### Step 1: เปิด Scene

```
File > Open Scene
เลือก: projects/Celeste Demo/scenes/main.json
```

### Step 2: สร้าง Map Entity

```
Hierarchy Panel:
1. Right-click > Create Empty
2. ตั้งชื่อ: "Level Map"
```

### Step 3: เพิ่ม Map Component

```
Inspector Panel (เลือก "Level Map"):
1. คลิก "➕ Add Component"
2. เลือก "Map"
3. ตั้งค่า:
   - Name: Level 01
   - Type: LDtk
   - File: levels/Level_01.ldtk
   - Hot-Reload: ✓
```

### Step 4: Load Map

```
Inspector > Map Component:
คลิก "📥 Load Map"
```

### Step 5: ดู Console

```
Bottom Panel > Console Tab

ควรเห็น:
[INFO] Loading map: projects/Celeste Demo/levels/Level_01.ldtk
[INFO] ✓ Loaded 1 entities from map
```

### Step 6: ดู Hierarchy

```
Hierarchy Panel:

ควรเห็น entities ใหม่:
├─ Level Map (Map component)
└─ LDTK Layer: IntGrid_layer  ← Entity ที่ spawn จาก map
```

### Step 7: ดูใน Scene View

```
Scene View:
1. เลือก "LDTK Layer: IntGrid_layer"
2. กด F (Frame Selected)
3. ควรเห็น map!
```

## 🐛 ถ้ายังไม่เห็น Map

### ตรวจสอบ 1: Tileset Path

ไฟล์ LDtk ของคุณใช้ tileset:
```
atlas/Cavernas_by_Adam_Saltsman.png
```

ตรวจสอบว่าไฟล์นี้มีอยู่:
```
projects/Celeste Demo/atlas/Cavernas_by_Adam_Saltsman.png
```

ถ้าไม่มี:
1. สร้างโฟลเดอร์ `atlas`
2. วาง tileset image ไว้ที่นั่น
3. หรือแก้ path ใน LDtk Editor

### ตรวจสอบ 2: Camera Position

Map อาจอยู่นอก camera view:

```
Scene View:
1. เลือก Camera entity
2. Inspector > Transform
3. Position: [0, 0, -10]  ← ต้องมอง map ได้

หรือ:
1. เลือก "LDTK Layer: IntGrid_layer"
2. กด F (Frame Selected)
```

### ตรวจสอบ 3: Layer Type

Layer "IntGrid_layer" เป็น IntGrid type:
- ใช้สำหรับ collision data
- อาจไม่มี visual tiles

ถ้าต้องการเห็น tiles:
1. เปิด Level_01.ldtk ใน LDtk Editor
2. เพิ่ม Tile Layer
3. วาด tiles
4. Save
5. Reload ใน Engine

### ตรวจสอบ 4: Auto-Layer Rules

IntGrid layer ใช้ auto-tiling rules:
- ต้องมี rules ตั้งค่าใน LDtk
- Rules จะ generate tiles อัตโนมัติ

ใน LDtk Editor:
1. เลือก IntGrid_layer
2. คลิก "RULES" button
3. ตั้งค่า auto-tiling rules
4. Save

## 📝 วิธีเพิ่ม Tile Layer

### ใน LDtk Editor:

```
1. เปิด Level_01.ldtk

2. Layers Tab > Add Layer
   - Type: Tiles
   - Name: Ground
   - Tileset: Cavernas_by_Adam_Saltsman

3. วาด Tiles
   - เลือก layer "Ground"
   - เลือก tiles จาก tileset
   - วาดใน level

4. Save (Ctrl+S)
```

### ใน Engine:

```
1. Inspector > Map Component
2. คลิก "🔄 Reload"
3. ดู Hierarchy:
   ├─ LDTK Layer: IntGrid_layer
   └─ LDTK Layer: Ground  ← Layer ใหม่!
```

## 🎮 Hot-Reload Workflow

```
1. เปิด LDtk Editor
   - แก้ไข Level_01.ldtk

2. เปิด Game Engine
   - Scene: main.json
   - Map Component: Hot-Reload ✓

3. แก้ไขใน LDtk
   - วาด tiles
   - แก้ไข level

4. Save (Ctrl+S)
   - Engine reload อัตโนมัติ!
   - เห็นการเปลี่ยนแปลงทันที
```

## 💡 Tips

### 1. ใช้ Tile Layer แทน IntGrid

IntGrid ใช้สำหรับ collision:
```
IntGrid Layer → Collision data
Tile Layer → Visual tiles
```

สร้าง Tile Layer:
```
LDtk > Layers > Add Layer
Type: Tiles
Tileset: เลือก tileset
```

### 2. ตรวจสอบ Tileset Path

Path ใน LDtk ต้อง relative จาก .ldtk file:
```
Level_01.ldtk อยู่ที่: projects/Celeste Demo/levels/
Tileset อยู่ที่: projects/Celeste Demo/atlas/

Path ใน LDtk: ../atlas/Cavernas_by_Adam_Saltsman.png
หรือ: atlas/Cavernas_by_Adam_Saltsman.png (ถ้า relative จาก project root)
```

### 3. Debug ด้วย Console

```
Console messages:
✓ Loading map: ...
✓ Loaded X entities from map

ถ้า X = 0:
- Level ไม่มี layers
- Layers ไม่มี data
- ตรวจสอบใน LDtk Editor
```

### 4. Frame Selected

```
Scene View:
1. เลือก entity ที่ spawn
2. กด F
3. Camera จะ zoom ไปที่ entity
```

## 🎯 Expected Result

หลัง Load Map สำเร็จ:

```
Console:
[INFO] Loading map: projects/Celeste Demo/levels/Level_01.ldtk
[INFO] ✓ Loaded 1 entities from map

Hierarchy:
├─ Camera 2D
├─ Player
├─ Level Map (Map component)
│  └─ Loaded: ✓
│  └─ Entities: 1
└─ LDTK Layer: IntGrid_layer
   ├─ Transform: [0, 0, 0]
   └─ Tilemap: 37x26

Scene View:
(เห็น map ที่ position [0, 0, 0])
```

## 🔧 Quick Fix

ถ้ายังไม่เห็น map:

```bash
# 1. ตรวจสอบไฟล์
dir "projects\Celeste Demo\levels\Level_01.ldtk"

# 2. ตรวจสอบ tileset
dir "projects\Celeste Demo\atlas\Cavernas_by_Adam_Saltsman.png"

# 3. ทดสอบ load
cargo run --example load_ldtk_map
```

## 📚 Next Steps

1. ✅ Load map สำเร็จ
2. ✅ เพิ่ม Tile Layer สำหรับ visual
3. ✅ ตั้งค่า collision จาก IntGrid
4. ✅ ทดสอบ hot-reload
5. ✅ สร้าง multiple levels

---

ถ้ายังมีปัญหา:
1. ดู Console log
2. ตรวจสอบ tileset path
3. เพิ่ม Tile Layer ใน LDtk
4. ลอง Frame Selected (F)

Good luck! 🎮✨

# แก้ปัญหา LDtk Version Mismatch

## ❌ Error ที่เจอ:

```
Failed to parse LDTK JSON: unknown variant `ExportOldTableOfContentData`
```

## 🔍 สาเหตุ:

- ไฟล์ `Level_01.ldtk` ใช้ LDtk **1.5.3** (ใหม่เกินไป)
- Engine ใช้ `ldtk_rust 0.6` รองรับ LDtk **1.1.3**
- Flag `ExportOldTableOfContentData` ไม่มีใน version เก่า

## ✅ วิธีแก้ (เลือก 1 ใน 3):

### วิธีที่ 1: แก้ไขไฟล์ LDtk (ทำแล้ว ✓)

ลบ flag ที่ไม่รองรับ:

```json
// เดิม:
"flags": [ "ExportOldTableOfContentData", "UseMultilinesType" ]

// แก้เป็น:
"flags": [ "UseMultilinesType" ]
```

**ทำแล้ว!** ลอง load map อีกครั้ง

### วิธีที่ 2: ใช้ไฟล์ใหม่ (สำรอง)

ผมสร้างไฟล์ใหม่ให้แล้ว:
```
projects/Celeste Demo/levels/simple_level.ldtk
```

ใช้ไฟล์นี้แทน:
1. Inspector > Map Component
2. File: `levels/simple_level.ldtk`
3. คลิก Load Map

### วิธีที่ 3: สร้างใหม่ใน LDtk Editor

```
1. เปิด LDtk Editor (version 1.1.3 หรือใกล้เคียง)
2. File > New Project
3. Save as: projects/Celeste Demo/levels/new_level.ldtk
4. สร้าง level ง่ายๆ:
   - Add Tileset
   - Add Tile Layer
   - วาด tiles
5. Save
6. Load ใน Engine
```

## 🎯 ขั้นตอนทดสอบ:

### 1. ลอง Load Level_01.ldtk อีกครั้ง

```
Inspector > Map Component
File: levels/Level_01.ldtk
คลิก "📥 Load Map"

ดู Console:
✓ ถ้าสำเร็จ: "Loaded X entities from map"
✗ ถ้ายังไม่ได้: ลองวิธีที่ 2
```

### 2. ถ้ายังไม่ได้ ลอง simple_level.ldtk

```
Inspector > Map Component
File: levels/simple_level.ldtk
คลิก "📥 Load Map"

ควรเห็น:
- Console: "Loaded 1 entities from map"
- Hierarchy: "LDTK Layer: Ground"
```

### 3. ตรวจสอบ Tileset

ไฟล์ `simple_level.ldtk` ต้องการ:
```
projects/Celeste Demo/assets/tiles.png
```

ถ้าไม่มี:
1. สร้างโฟลเดอร์ `assets`
2. วาง tileset image (16x16 tiles)
3. หรือแก้ path ใน simple_level.ldtk

## 📝 สร้าง Tileset ง่ายๆ

ถ้าไม่มี tileset:

### วิธีที่ 1: ใช้ Placeholder

สร้างไฟล์ 128x128 pixels:
- 8x8 tiles
- แต่ละ tile 16x16 pixels
- สีต่างๆ เพื่อทดสอบ

### วิธีที่ 2: Download Free Tileset

```
1. ไปที่ https://itch.io/game-assets/free/tag-tileset
2. Download tileset ที่ชอบ
3. วางใน projects/Celeste Demo/assets/
4. แก้ path ใน LDtk
```

### วิธีที่ 3: ใช้ Tileset ที่มี

ถ้ามี `Cavernas_by_Adam_Saltsman.png`:
```
1. Copy ไปที่ assets/tiles.png
2. หรือแก้ path ใน simple_level.ldtk:
   "relPath": "../atlas/Cavernas_by_Adam_Saltsman.png"
```

## 🔧 แก้ไข simple_level.ldtk

ถ้าต้องการใช้ tileset ที่มี:

```json
// ใน simple_level.ldtk
"tilesets": [
  {
    "identifier": "Tiles",
    "uid": 2,
    "relPath": "../atlas/Cavernas_by_Adam_Saltsman.png",  // แก้ path
    "pxWid": 256,      // แก้ขนาด
    "pxHei": 256,      // แก้ขนาด
    "tileGridSize": 8, // แก้ tile size
    "spacing": 0,
    "padding": 0
  }
]
```

## 🎮 ทดสอบว่าทำงาน:

```
1. Load Map
   Console: "✓ Loaded 1 entities from map"

2. ดู Hierarchy
   └─ LDTK Layer: Ground

3. เลือก layer นั้น
   Inspector > Tilemap component

4. กด F (Frame Selected)
   Scene View: เห็น tiles!
```

## 💡 Tips

### ถ้ายังไม่เห็น Tiles:

1. **ตรวจสอบ Camera**
   ```
   Scene View > เลือก Camera
   Position: [0, 0, -10]
   ```

2. **ตรวจสอบ Tileset Path**
   ```
   Console: ดู texture loading errors
   ```

3. **ตรวจสอบ Layer**
   ```
   Hierarchy > เลือก "LDTK Layer: Ground"
   Inspector > Tilemap:
   - Width: 20
   - Height: 11
   - Tiles: มี data
   ```

### ถ้า Load สำเร็จแต่ไม่เห็น:

```
Scene View:
1. เลือก "LDTK Layer: Ground"
2. กด F (Frame Selected)
3. Zoom out ถ้าจำเป็น
4. ตรวจสอบ Z-position
```

## 🚀 Next Steps

หลังจาก load สำเร็จ:

1. ✅ ทดสอบ hot-reload
   - แก้ไขใน LDtk Editor
   - Save
   - ดูการเปลี่ยนแปลงใน Engine

2. ✅ เพิ่ม layers
   - Background
   - Foreground
   - Collision

3. ✅ สร้าง multiple levels
   - Level_1, Level_2, etc.

4. ✅ ทดสอบ gameplay
   - Player movement
   - Collision
   - Level transitions

---

**สรุป:**
- ✓ แก้ไข Level_01.ldtk แล้ว (ลบ flag)
- ✓ สร้าง simple_level.ldtk สำรอง
- ✓ ลอง load อีกครั้ง!

Good luck! 🎮✨

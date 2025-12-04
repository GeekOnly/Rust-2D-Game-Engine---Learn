# เพิ่ม Tile Layer ใน LDtk - คู่มือทีละขั้นตอน

## 🎯 เป้าหมาย

เพิ่ม Tile Layer เพื่อแสดง visual tiles ใน Level_01.ldtk

## 📋 ขั้นตอน

### 1. เปิด LDtk Editor

```
1. เปิด LDtk Editor
2. File > Open
3. เลือก: projects/Celeste Demo/levels/Level_01.ldtk
```

### 2. ตรวจสอบ Tileset

```
1. คลิก "Tilesets" tab (ด้านขวา)
2. ควรเห็น: "Cavernas_by_Adam_Saltsman"
3. ถ้าไม่มี ให้เพิ่ม:
   - คลิก "+ Add tileset"
   - Browse: atlas/Cavernas_by_Adam_Saltsman.png
   - Tile size: 8x8
   - Spacing: 0
   - Padding: 0
```

### 3. เพิ่ม Tile Layer

```
1. คลิก "Layers" tab (ด้านขวา)
2. คลิก "+ Add layer" (ด้านล่าง)
3. ตั้งค่า:
   ┌─────────────────────────────┐
   │ Layer Type: Tiles           │
   │ Identifier: Ground          │
   │ Grid size: 8                │
   │ Tileset: Cavernas_by_Adam.. │
   │ Opacity: 1.0                │
   └─────────────────────────────┘
4. คลิก "Create"
```

### 4. จัดเรียง Layers

```
Layers (จากบนลงล่าง):
┌─────────────────────────┐
│ 1. IntGrid_layer        │ ← Collision
│ 2. Ground (NEW!)        │ ← Visual tiles
└─────────────────────────┘

ลาก Ground ไปไว้ใต้ IntGrid_layer
```

### 5. วาด Tiles

```
1. เลือก level "AutoLayer"
2. เลือก layer "Ground" (ด้านซ้าย)
3. เลือก tiles จาก tileset (ด้านขวา)
4. วาดใน level:
   - คลิกซ้าย: วาด tile
   - คลิกขวา: ลบ tile
   - Shift + คลิก: วาดหลาย tiles
```

### 6. วาด Ground Tiles

```
แนะนำ:
1. วาดพื้นด้านล่าง (bottom row)
2. วาดแพลตฟอร์ม (platforms)
3. วาดกำแพง (walls)
4. เพิ่มรายละเอียด (decorations)

Tips:
- ใช้ Grid Snap (เปิดอยู่แล้ว)
- กด Space + ลาก: เลื่อน view
- Scroll: Zoom in/out
- Ctrl+Z: Undo
```

### 7. Save

```
File > Save (Ctrl+S)
```

### 8. Reload ใน Engine

```
1. กลับไปที่ Game Engine
2. Inspector > Map Component
3. คลิก "🔄 Reload"
4. ดู Hierarchy:
   ├─ LDTK Layer: IntGrid_layer
   └─ LDTK Layer: Ground  ← Layer ใหม่!
```

## 🎨 ตัวอย่าง Tile Layout

```
Level Layout (296x208 pixels, 37x26 tiles):

┌─────────────────────────────────┐
│                                 │ ← Sky (empty)
│         🎮                      │ ← Player spawn
│                                 │
│    ████      ████               │ ← Platforms
│                                 │
│                    ████         │
│                                 │
│████████████████████████████████│ ← Ground
└─────────────────────────────────┘
```

## 🔧 Tileset Configuration

### ถ้าใช้ Cavernas_by_Adam_Saltsman.png:

```
Tileset Properties:
├─ Path: atlas/Cavernas_by_Adam_Saltsman.png
├─ Tile size: 8x8 pixels
├─ Grid: 32x32 tiles (256x256 image)
├─ Spacing: 0
└─ Padding: 0

Tile Types:
├─ Ground: Row 0-5
├─ Walls: Row 6-10
├─ Platforms: Row 11-15
└─ Decorations: Row 16+
```

## 💡 Tips & Tricks

### 1. Quick Tile Selection

```
- Number keys (1-9): Quick select tiles
- [ ] keys: Previous/Next tile
- R: Rotate tile
- X/Y: Flip tile
```

### 2. Painting Tools

```
- B: Brush (single tile)
- L: Line tool
- R: Rectangle tool
- F: Fill tool
- E: Eraser
```

### 3. Layer Visibility

```
- Click eye icon: Toggle layer visibility
- Shift+R: Toggle auto-layer rendering
- Alt: Show all layers
```

### 4. Grid & Guides

```
- G: Toggle grid
- Ctrl+G: Grid settings
- Rulers: Show pixel coordinates
```

## 🎯 ตัวอย่างการวาด

### Ground Platform:

```
1. เลือก Ground layer
2. เลือก ground tiles (row 0-2)
3. วาดแถวล่างสุด:
   ████████████████████████████████

4. เพิ่มรายละเอียด:
   - Top edge: ใช้ grass tiles
   - Sides: ใช้ dirt tiles
   - Corners: ใช้ corner tiles
```

### Floating Platform:

```
1. วาดแพลตฟอร์มลอย:
   ████████

2. เพิ่ม shadow ด้านล่าง (optional)
3. เพิ่ม decorations
```

## 🔄 Hot-Reload Workflow

```
1. แก้ไขใน LDtk
   └─> วาด tiles

2. Save (Ctrl+S)
   └─> ไฟล์ถูก update

3. Engine reload อัตโนมัติ
   └─> เห็นการเปลี่ยนแปลงทันที!

4. Test ใน game
   └─> ถ้าไม่ชอบ กลับไปข้อ 1
```

## 🐛 Troubleshooting

### ไม่เห็น Tiles ใน Engine

```
1. ตรวจสอบ layer visibility ใน LDtk
2. ตรวจสอบว่าวาด tiles แล้ว
3. ตรวจสอบ tileset path
4. คลิก Reload ใน Engine
5. กด F (Frame Selected) ใน Scene View
```

### Tileset ไม่แสดง

```
1. ตรวจสอบ path:
   atlas/Cavernas_by_Adam_Saltsman.png

2. ตรวจสอบว่าไฟล์มีอยู่:
   projects/Celeste Demo/atlas/...

3. ถ้าไม่มี:
   - สร้างโฟลเดอร์ atlas
   - วาง tileset image
   - หรือแก้ path ใน LDtk
```

### Tiles ขนาดไม่ถูก

```
1. ตรวจสอบ Grid size:
   - Layer: 8x8
   - Tileset: 8x8

2. ถ้าไม่ตรง:
   - แก้ไข Layer properties
   - หรือแก้ไข Tileset properties
```

## 📚 Resources

### LDtk Keyboard Shortcuts:

```
General:
- Ctrl+S: Save
- Ctrl+Z: Undo
- Ctrl+Y: Redo
- Space+Drag: Pan view
- Scroll: Zoom

Tools:
- B: Brush
- E: Eraser
- L: Line
- R: Rectangle
- F: Fill

View:
- G: Toggle grid
- Shift+R: Toggle auto-layer
- Tab: Toggle panels
```

### LDtk Documentation:

- Official: https://ldtk.io/docs/
- Tutorials: https://ldtk.io/docs/tutorials/
- Community: https://discord.gg/ldtk

## ✅ Checklist

- [ ] เปิด Level_01.ldtk ใน LDtk Editor
- [ ] ตรวจสอบ Tileset มีอยู่
- [ ] เพิ่ม Tile Layer ชื่อ "Ground"
- [ ] เลือก Tileset ที่ถูกต้อง
- [ ] วาด Ground tiles
- [ ] วาด Platform tiles
- [ ] Save (Ctrl+S)
- [ ] Reload ใน Engine
- [ ] ตรวจสอบใน Hierarchy
- [ ] Test ใน Scene View

---

**เมื่อทำเสร็จ:**
- ✅ มี 2 layers: IntGrid_layer + Ground
- ✅ เห็น visual tiles ใน Engine
- ✅ Hot-reload ทำงาน

Happy Tile Painting! 🎨✨

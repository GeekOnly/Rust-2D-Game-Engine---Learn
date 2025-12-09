# How to See Grid Component in Scene View

## Problem
Grid component ไม่แสดงใน Scene View แม้ว่าจะมี Grid entity ใน Hierarchy

## Cause
Scene ถูก save ก่อนที่จะมีการเพิ่ม Grid component support ดังนั้น Grid entity ไม่มีใน scene file

## Solution

### วิธีที่ 1: Reload Map และ Save Scene (แนะนำ)
1. เปิด **Maps panel** (ด้านขวา)
2. หา map ที่ต้องการ (เช่น "Level_01.ldtk")
3. คลิก **Reload** button (🔄)
4. Grid entity จะถูกสร้างใหม่พร้อม Grid component
5. **IMPORTANT**: กด **Ctrl+S** เพื่อ **Save scene** (Grid component จะถูก save)
6. เปลี่ยนเป็น **3D mode** ใน Scene View
7. คุณจะเห็น **สอง grid**:
   - **3D Space Grid** (เข้ม, บาง) - สำหรับ navigation
   - **Grid Component Grid** (สว่าง, หนา) - สำหรับ tilemap layout

**หมายเหตุ**: ถ้ายังไม่เห็น Grid Component Grid หลัง save, ให้ **reload scene** (File > Open Scene > เลือก scene เดิม)

### วิธีที่ 2: Load Map ใหม่
1. เปิด **Maps panel**
2. คลิก **Load Map** button
3. เลือกไฟล์ `.ldtk` (เช่น `projects/Celeste Demo/levels/Level_01.ldtk`)
4. Grid entity จะถูกสร้างพร้อม Grid component

### วิธีที่ 3: สร้าง Grid Entity เอง
1. ใน **Hierarchy**, คลิก **➕** > **Empty GameObject**
2. เปลี่ยนชื่อเป็น "My Grid"
3. ใน **Inspector**, คลิก **➕ Add Component**
4. เลือก **Grid** (ยังไม่มีใน menu - ต้องเพิ่มใน ComponentType)

## Verification

### ตรวจสอบว่า Grid Component ทำงาน:

1. **ใน Hierarchy**:
   - ควรเห็น entity ชื่อ "LDtk Grid - {filename}"
   - มี icon 🗺️

2. **ใน Inspector** (เมื่อเลือก Grid entity):
   - ควรเห็น **Grid** component
   - Properties: Cell Size, Cell Gap, Layout, Plane

3. **ใน Scene View (3D mode)**:
   - ควรเห็น **สอง grid**:
     - Grid เข้ม (dark gray) = 3D Space Grid
     - Grid สว่าง (bright gray) = Grid Component Grid
   - Grid Component Grid จะมีเส้นหนากว่า

4. **ใน Console/Logs**:
   ```
   [INFO] Found 1 Grid component(s) in scene
   [INFO] Rendering Grid component 'LDtk Grid - Level_01' (entity X): plane=XY, cell_size=(0.080, 0.080, 0.000)
   ```

## Troubleshooting

### ไม่เห็น Grid Component Grid
- ✅ ตรวจสอบว่าอยู่ใน **3D mode** (ไม่ใช่ 2D mode)
- ✅ ตรวจสอบว่า Grid entity มีใน **Hierarchy**
- ✅ ตรวจสอบว่า Grid entity มี **Grid component** ใน Inspector
- ✅ ลอง **Reload map** ใน Maps panel
- ✅ ดู **Console logs** ว่ามี "Found X Grid component(s)" หรือไม่

### เห็นแค่ 3D Space Grid
- ❌ Scene ไม่มี Grid entity → **Reload map**
- ❌ Grid entity ไม่มี Grid component → **Reload map**
- ❌ อยู่ใน 2D mode → **เปลี่ยนเป็น 3D mode**

### Grid Component Grid เล็กเกินไป
- Grid cell_size เล็กมาก (< 0.5) → ระบบจะใช้ scene_grid.size แทน
- ถ้ายังเล็ก ให้แก้ไข **Cell Size** ใน Inspector

### Grid Plane ไม่ถูกต้อง
- Default plane คือ **XY** (horizontal)
- เปลี่ยนได้ใน Inspector > Grid > Plane:
  - **XZ** สำหรับ vertical walls
  - **YZ** สำหรับ side view

## Expected Result

เมื่อทำถูกต้อง คุณจะเห็น:

```
Scene View (3D mode):
┌─────────────────────────────────┐
│                                 │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │ ← 3D Space Grid (เข้ม, บาง)
│  ▓                           ▓  │
│  ▓  ████████████████████████ ▓  │ ← Grid Component Grid (สว่าง, หนา)
│  ▓  █                      █ ▓  │
│  ▓  █   Tilemap Layout     █ ▓  │
│  ▓  █                      █ ▓  │
│  ▓  ████████████████████████ ▓  │
│  ▓                           ▓  │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │
│                                 │
└─────────────────────────────────┘
```

**สอง grid จะแสดงพร้อมกัน** โดย Grid Component Grid จะสว่างและหนากว่า

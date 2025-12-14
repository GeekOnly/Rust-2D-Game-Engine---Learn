# Grid Component Usage Guide

## Overview
Grid component ทำงานเหมือน Unity's Grid component โดย:
- **กำหนด cell layout และ coordinate system** สำหรับ tilemaps
- **แปลง Grid cell positions** เป็น local coordinates ของ GameObject
- **ทำงานร่วมกับ Transform component** เพื่อแปลง local coordinates เป็น world space
- **เป็น guide สำหรับจัดวาง GameObjects** เช่น Tiles ตาม layout ที่เลือก

## Features
- ✅ แสดงใน Hierarchy panel พร้อมชื่อที่ชัดเจน (เช่น "LDtk Grid - Level_01")
- ✅ แสดงใน Inspector panel พร้อม properties ที่แก้ไขได้
- ✅ รองรับ 3 plane orientations: XY (horizontal), XZ (vertical), YZ (side)
- ✅ แสดง grid ใน Scene View ตาม plane ที่เลือก
- ✅ รองรับ cell size, cell gap, และ layout types

## Grid Properties (Inspector)

### Cell Size
**Purpose**: The size of a cell on this Grid (in Unity units)
- กำหนดขนาดของแต่ละ cell ใน world units
- X, Y, Z สำหรับ 3D grids
- Default: (0.08, 0.08, 0.0) สำหรับ LDtk maps (8px / 100 PPU)
- **Unity Note**: The Grid component transforms Grid cell positions to the corresponding local coordinates of the GameObject

### Cell Gap
**Purpose**: The size (in Unity units) of gaps between cells on this Grid
- ระยะห่างระหว่าง cells
- X, Y
- Default: (0.0, 0.0)
- **Unity Validation**: ✅ If a negative number with an absolute value higher than the Cell Size is entered, the system will automatically clamp the absolute value to match the Cell Size instead
  - Example: Cell Size = (1, 1), Cell Gap = (-2, -2) → Auto-clamped to (-1, -1)

### Cell Layout
**Purpose**: Defines the shape and arrangement of cells on this Grid
- **Rectangle**: Cells are rectangular (default)
- **Hexagon (Flat Top)**: Cells are hexagonal with flat top
- **Hexagon (Pointy Top)**: Cells are hexagonal with pointy top
- **Isometric**: Cells are rhombus-shaped for an isometric layout
- **Isometric Z as Y**: Similar to Isometric, but Unity converts the Z position of cells to their local Y coordinate

### Cell Swizzle
**Purpose**: The order that Unity reorders the XYZ cell coordinates to for transform conversions
- **XYZ**: Default XYZ cell coordinates
- **XZY**: Reorders XYZ coordinates to XZY
- **YXZ**: Reorders XYZ coordinates to YXZ
- **YZX**: Reorders XYZ coordinates to YZX
- **ZXY**: Reorders XYZ coordinates to ZXY
- **ZYX**: Reorders XYZ coordinates to ZYX

### Plane (Custom - Not in Unity)
**Purpose**: Determines which 2D plane the grid lies on in 3D space
- **XY (Horizontal)**: Grid บนพื้นราบ - เหมาะสำหรับ 2D games (default)
  - X = right (red), Y = up (green)
- **XZ (Vertical)**: Grid แนวตั้ง - เหมาะสำหรับ walls/vertical tilemaps
  - X = right (red), Z = up (blue)
- **YZ (Side)**: Grid มุมข้าง - เหมาะสำหรับ side view
  - Y = right (green), Z = up (blue)

## Usage in Scene View

### 2D Mode
- Grid แสดงเป็น 2D grid บนพื้นราบ
- ใช้ scene_grid.size เป็นขนาด visual

### 3D Mode - Unity-style Behavior

ใน 3D mode จะมี **สอง grid types**:

#### 1. 3D Space Grid (Default Grid)
- **สีเข้ม** (dark gray) - สำหรับ navigation
- แสดงบน **XZ plane** (ground) เสมอ
- แกน X=แดง, Z=น้ำเงิน
- ขนาดใหญ่ (50x50 cells)
- เส้นบาง (0.8px)

#### 2. Grid Component Grid (Tilemap Grid)
- **สีสว่าง** (bright gray) - สำหรับ tilemap layout
- **แสดงเฉพาะเมื่อเลือก Grid entity** ใน Hierarchy (Unity-style)
- แสดงตาม **plane ที่เลือก** ใน Grid component:
  - XY: แกน X=แดง, Y=เขียว (horizontal)
  - XZ: แกน X=แดง, Z=น้ำเงิน (vertical)
  - YZ: แกน Y=เขียว, Z=น้ำเงิน (side)
- ขนาดเล็กกว่า (20x20 cells)
- เส้นหนากว่า (1.2px)
- แกนหลักหนาขึ้น (2.5px)

**Unity-style Behavior**:
- เลือก Grid entity → แสดง Grid component grid
- ยกเลิกการเลือก → ซ่อน Grid component grid
- ทั้งสอง grid จะ fade out ตามระยะห่างจากกล้อง

## Creating Tilemaps in 3D

1. สร้าง Grid entity (หรือใช้ที่ LDtk loader สร้างให้)
2. เลือก Grid entity ใน Hierarchy
3. ใน Inspector, เปลี่ยน Grid Plane:
   - XZ สำหรับ vertical walls
   - YZ สำหรับ side view
4. เปลี่ยน Scene View เป็น 3D mode
5. Grid จะแสดงในระนาบที่เลือก
6. Tilemap layers ที่เป็น children ของ Grid จะถูกวางตาม grid plane

## LDtk Integration

เมื่อ load LDtk map:
- Grid entity ถูกสร้างอัตโนมัติ
- ชื่อ: "LDtk Grid - {filename}"
- Cell size: {grid_size} / 100 (Unity standard PPU)
- Plane: XY (horizontal) - default สำหรับ 2D
- Tilemap layers เป็น children ของ Grid entity

## Visual Differences

| Feature | 3D Space Grid | Grid Component Grid |
|---------|---------------|---------------------|
| Purpose | Navigation | Tilemap layout |
| Color | Dark gray (64,64,64) | Bright gray (100,100,100) |
| Plane | XZ (ground) only | XY/XZ/YZ (configurable) |
| Size | 50x50 cells | 20x20 cells |
| Line Width | 0.8px (thin) | 1.2px (thick) |
| Axis Width | 2.0px | 2.5px |
| Alpha | 76-100 | 120-150 |

## Tips

- **Cell size เล็กเกินไป**: ถ้า cell_size < 0.5, ระบบจะใช้ scene_grid.size เป็น visual scale
- **ไม่เห็น Grid component grid**: ตรวจสอบว่า:
  1. Scene View อยู่ใน 3D mode
  2. Grid plane ถูกต้องสำหรับมุมกล้อง
  3. Grid entity มี Transform component
  4. Grid component grid จะสว่างกว่า space grid
- **เปลี่ยน plane**: แก้ไขได้ใน Inspector > Grid > Plane
- **แยกแยะ grid**: Grid component grid จะมีสีสว่างและเส้นหนากว่า space grid

## Example Workflows

### Example 1: View Horizontal Tilemap Grid
```
1. Load LDtk map → Grid entity created automatically
2. Scene View → 3D mode
3. Hierarchy → Select "LDtk Grid - Level_01"
4. Grid component grid แสดงบน XY plane (horizontal)
5. แก้ไข Cell Size ใน Inspector → เห็นผลทันที
```

### Example 2: Create Vertical Tilemap (Wall)
```
1. Load LDtk map → Grid entity created
2. Hierarchy → Select "LDtk Grid - Level_01"
3. Inspector > Grid > Plane → XZ (Vertical)
4. Scene View → 3D mode (ถ้ายังไม่ได้เปลี่ยน)
5. Grid แสดงแนวตั้ง พร้อมสำหรับ vertical tilemap
6. Tilemap children จะถูกวางตาม vertical grid
```

### Example 3: Adjust Grid for Different Tile Sizes
```
1. Select Grid entity in Hierarchy
2. Inspector > Grid > Cell Size
3. แก้ไข X, Y ตามขนาด tiles (เช่น 0.16, 0.16 สำหรับ 16px tiles)
4. Grid component grid จะปรับขนาดทันที
5. Tiles จะ snap ตาม cell size ใหม่
```

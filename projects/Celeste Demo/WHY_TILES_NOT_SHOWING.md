# ทำไม Tiles ไม่แสดงเหมือนใน LDtk?

## 🤔 คำถาม:

ใน LDtk เห็น tiles สวยงาม (สีส้ม/เหลือง) แต่ใน Engine ไม่แสดง?

## 🔍 คำตอบ:

### ใน LDtk คุณเห็น:

```
┌─────────────────────────────────┐
│ 🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨 │ ← Auto-generated tiles
│ 🟨                         🟨 │
│ 🟨  🟧🟧🟧🟧              🟨 │
│ 🟨                         🟨 │
│ 🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨🟨 │
└─────────────────────────────────┘
```

### ใน Engine เห็น:

```
┌─────────────────────────────────┐
│                                 │ ← ว่างเปล่า!
│                                 │
│                                 │
│                                 │
│                                 │
└─────────────────────────────────┘
```

## 💡 สาเหตุ:

### 1. IntGrid Layer + Auto-Tiling Rules

ใน LDtk:
- **IntGrid Layer** = Collision data (1 = wall, 0 = empty)
- **Auto-Tiling Rules** = Generate tiles อัตโนมัติจาก IntGrid
- **autoLayerTiles** = Tiles ที่ generate ได้

### 2. Engine โหลดแค่ Layer Structure

เดิม Engine โหลด:
```rust
// โหลดแค่ layer info
- Layer name: "IntGrid_layer"
- Size: 37x26
- Position: [0, 0]

// ❌ ไม่ได้โหลด tiles!
```

## ✅ แก้ไขแล้ว!

### Update ล่าสุด:

Engine ตอนนี้ตรวจสอบ:
1. **gridTiles** - Tiles ที่วาดเอง
2. **autoLayerTiles** - Tiles ที่ auto-generate

```rust
// ตรวจสอบว่ามี tiles หรือไม่
let has_grid_tiles = layer["gridTiles"].as_array()...
let has_auto_tiles = layer["autoLayerTiles"].as_array()...

// สร้าง entity เฉพาะ layer ที่มี tiles
if has_grid_tiles || has_auto_tiles {
    // Create tilemap entity
}
```

## 🎮 ทดสอบใหม่:

### 1. Rebuild Engine

```bash
cargo build --release
```

### 2. Load Map อีกครั้ง

```
1. เปิด Engine
2. Inspector > Map Component
3. คลิก "🔄 Reload"
```

### 3. ดู Console

```
[INFO] Layer 'IntGrid_layer': 1234 tiles (auto: true, grid: false)
```

ถ้าเห็น tile count > 0 = มี tiles!

## 🔧 ขั้นตอนต่อไป:

### ปัญหาที่เหลือ:

แม้ engine จะโหลด tiles แล้ว แต่ยังไม่ **render** เพราะ:

1. **TilemapRenderer ยังไม่ implement**
   - ต้องสร้าง vertex buffer
   - ต้อง load tileset texture
   - ต้อง render tiles

2. **Tileset Path ไม่ถูกต้อง**
   - LDtk: `atlas/Cavernas_by_Adam_Saltsman.png`
   - Engine: ต้อง resolve path

3. **Tile Data ยังไม่ได้ parse**
   - ต้องอ่าน tile positions
   - ต้องอ่าน tile IDs
   - ต้องอ่าน tileset coordinates

## 📊 สถานะปัจจุบัน:

```
✅ Load LDtk file
✅ Parse layers
✅ Detect tiles (gridTiles + autoLayerTiles)
✅ Create entities
⬜ Parse tile data
⬜ Load tileset textures
⬜ Render tiles
```

## 🚀 Next Steps:

### Step 1: Parse Tile Data

```rust
// อ่าน autoLayerTiles
for tile in layer["autoLayerTiles"].as_array() {
    let px = tile["px"].as_array(); // [x, y]
    let src = tile["src"].as_array(); // [x, y] in tileset
    let tile_id = tile["t"].as_i64(); // tile ID
    
    // เก็บ tile data
}
```

### Step 2: Load Tileset

```rust
// อ่าน tileset path
let tileset_path = layer["__tilesetRelPath"].as_str();

// Load texture
texture_manager.load(tileset_path);
```

### Step 3: Render Tiles

```rust
// ใน TilemapRenderer
for tile in tilemap.tiles {
    // Create quad
    // Set texture coordinates
    // Render
}
```

## 💡 Workaround ชั่วคราว:

### ถ้าต้องการเห็น tiles เร็วๆ:

1. **Export PNG จาก LDtk**
   ```
   LDtk > File > Export PNG
   ```

2. **ใช้เป็น Sprite**
   ```
   Engine > Add Sprite Component
   Texture: exported_level.png
   ```

3. **หรือเพิ่ม Tile Layer แทน IntGrid**
   ```
   LDtk > Add Layer > Type: Tiles
   วาด tiles เอง (ไม่ใช้ auto-tiling)
   ```

## 📚 Technical Details:

### LDtk Auto-Tiling:

```json
{
  "layerInstances": [{
    "__identifier": "IntGrid_layer",
    "intGridCsv": [1,1,1,0,0,0,...],  // Collision data
    "autoLayerTiles": [                // Generated tiles
      {
        "px": [0, 0],      // Position in level
        "src": [64, 56],   // Position in tileset
        "f": 0,            // Flip flags
        "t": 64            // Tile ID
      }
    ]
  }]
}
```

### Engine Tilemap:

```rust
pub struct Tilemap {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,  // ← ต้องเพิ่ม tile data
}

pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub tile_id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
}
```

## 🎯 Summary:

**ตอนนี้:**
- ✅ Engine รู้ว่า layer มี tiles
- ✅ Engine สร้าง entity สำหรับ layer
- ⬜ แต่ยังไม่ได้ render tiles

**ต้องทำต่อ:**
1. Parse tile data จาก autoLayerTiles
2. Load tileset textures
3. Implement tilemap rendering

---

**ใจเย็นๆ ครับ!** 🎮
เรากำลังทำทีละขั้นตอน:
1. ✅ Load map structure
2. ✅ Detect tiles
3. ⏳ Parse tile data (next!)
4. ⏳ Render tiles

Happy Coding! 🚀✨

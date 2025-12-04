# แก้ปัญหา Texture Path

## ❌ Error ที่เจอ:

```
Failed to load texture projects/Celeste Demo\atlas/Cavernas_by_Adam_Saltsman.png
The system cannot find the path specified. (os error 3)
```

## 🔍 ปัญหา:

### 1. Path Separator ผสมกัน

```
projects/Celeste Demo\atlas/Cavernas_by_Adam_Saltsman.png
                     ↑ backslash
                           ↑ forward slash
```

Windows ต้องการ backslash (`\`) แต่ LDtk ใช้ forward slash (`/`)

### 2. Path Resolution

```
LDtk path: atlas/Cavernas_by_Adam_Saltsman.png
Project: projects/Celeste Demo/
Full path: projects/Celeste Demo/atlas/Cavernas_by_Adam_Saltsman.png

แต่ได้: projects/Celeste Demo\atlas/Cavernas_by_Adam_Saltsman.png
```

## ✅ แก้ไขแล้ว:

### Path Normalization

```rust
// เดิม:
let tex_path = Path::new(&ts.texture_path);

// แก้เป็น:
let normalized_path = ts.texture_path.replace('/', MAIN_SEPARATOR_STR);
let tex_path = Path::new(&normalized_path);
```

ตอนนี้:
- `/` → `\` (on Windows)
- `/` → `/` (on Linux/Mac)

## 🎮 ทดสอบใหม่:

### 1. Rebuild

```bash
cargo build --release
```

### 2. Run Engine

```bash
cargo run --release
```

### 3. Reload Map

```
Inspector > Map Component > Reload
```

### 4. ตรวจสอบ Console

```
ควรเห็น:
[INFO] Layer 'IntGrid_layer': parsed 1234/1234 tiles
[INFO]   Tileset: atlas/Cavernas_by_Adam_Saltsman.png

ไม่ควรเห็น:
[ERROR] Failed to load texture...
```

## 📁 ตรวจสอบไฟล์:

### Path ที่ถูกต้อง:

```
projects/Celeste Demo/
├── levels/
│   ├── Level_01.ldtk
│   └── atlas/
│       └── Cavernas_by_Adam_Saltsman.png  ← ต้องมีไฟล์นี้!
└── scenes/
    └── main.json
```

### ตรวจสอบว่าไฟล์มีอยู่:

```bash
# Windows
dir "projects\Celeste Demo\levels\atlas\Cavernas_by_Adam_Saltsman.png"

# หรือ
ls "projects/Celeste Demo/levels/atlas/Cavernas_by_Adam_Saltsman.png"
```

## 🔧 ถ้ายังไม่ได้:

### Option 1: ย้ายไฟล์

```
จาก: projects/Celeste Demo/levels/atlas/...
ไป:   projects/Celeste Demo/atlas/...

เพราะ LDtk อาจ resolve path จาก project root
```

### Option 2: แก้ path ใน LDtk

```
1. เปิด Level_01.ldtk ใน LDtk Editor
2. Tilesets > Cavernas_by_Adam_Saltsman
3. แก้ path เป็น: atlas/Cavernas_by_Adam_Saltsman.png
4. Save
```

### Option 3: Copy ไฟล์

```bash
# Copy tileset ไปทั้ง 2 ที่
copy "projects\Celeste Demo\levels\atlas\*.png" "projects\Celeste Demo\atlas\"
```

## 🎨 ผลลัพธ์ที่คาดหวัง:

### Before (ตอนนี้):

```
Scene View:
- เห็นสี่เหลี่ยมสี (placeholder)
- Layout ถูกต้อง
- แต่ไม่มี texture
```

### After (หลังแก้):

```
Scene View:
- เห็น tiles จาก tileset!
- Texture แสดงถูกต้อง!
- สวยงามเหมือนใน LDtk!
```

## 📊 Debug Info:

### Console Messages:

```
✓ Good:
[INFO] Loading map: ...
[INFO] Layer 'IntGrid_layer': parsed 1234/1234 tiles
[INFO]   Tileset: atlas/Cavernas_by_Adam_Saltsman.png

✗ Bad:
[ERROR] Failed to load texture ...
[ERROR] The system cannot find the path specified
```

### Fallback Behavior:

```
ถ้า texture load ไม่ได้:
- Engine จะ render สี่เหลี่ยมสี (placeholder)
- แต่ layout ยังถูกต้อง
- Game ยังเล่นได้ แต่ไม่สวย
```

## 💡 Quick Fix:

### ถ้ารีบ:

```
1. ตรวจสอบว่าไฟล์มีอยู่:
   projects/Celeste Demo/levels/atlas/Cavernas_by_Adam_Saltsman.png

2. ถ้าไม่มี ให้ copy จาก:
   projects/Celeste Demo/tilemaps/atlas/...

3. หรือ download tileset ใหม่จาก:
   https://adamatomic.itch.io/cavernas
```

## 🎯 Summary:

**ปัญหา:**
- Path separator ผสมกัน
- Texture load ไม่ได้

**แก้ไข:**
- Normalize path separators
- ตรวจสอบไฟล์มีอยู่

**ผลลัพธ์:**
- Texture load สำเร็จ
- Tiles แสดงสวยงาม!

---

**หลังจาก rebuild แล้ว ลอง reload map ใหม่!** 🎮✨

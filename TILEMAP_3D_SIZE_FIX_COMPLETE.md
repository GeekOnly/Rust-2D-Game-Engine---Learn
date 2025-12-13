# Tilemap 3D Size Fix - Complete Solution

## ปัญหาที่พบ

ขนาด tilemap ไม่เต็ม grid size ใน 3D mode แต่ 2D ขนาดเต็ม grid size

## สาเหตุของปัญหา

Tilemap 3D renderer ไม่ได้รับ `pixels_per_unit` ที่ถูกต้องจาก tilemap settings ทำให้:
- ใช้ค่า default `pixels_per_unit = 100.0` แทนที่จะใช้ `8.0` (LDtk-compatible)
- Tile size ถูกคำนวณเป็น `8px / 100.0 = 0.08 world units` แทนที่จะเป็น `8px / 8.0 = 1.0 world units`
- ทำให้ tilemap ใน 3D mode เล็กกว่า grid cells มาก

## การแก้ไข

### 1. อัปเดต Tilemap 3D Renderer (`engine/src/editor/ui/scene_view/rendering/tilemap_3d.rs`)

**เพิ่ม parameter สำหรับ tilemap settings**:
```rust
pub fn collect_tilemaps(&mut self, world: &World, tilemap_settings: Option<&crate::editor::tilemap_settings::TilemapSettings>) -> Vec<TilemapLayer>
```

**ใช้ pixels_per_unit จาก settings**:
```rust
let pixels_per_unit = tilemap_settings
    .map(|s| s.pixels_per_unit)
    .unwrap_or(8.0); // Default to LDtk-compatible value for grid alignment
```

**คำนวณ tile size ที่ถูกต้อง**:
```rust
let tile_world_width = tileset.tile_width as f32 / pixels_per_unit;
let tile_world_height = tileset.tile_height as f32 / pixels_per_unit;
```

### 2. อัปเดต 3D View Renderer (`engine/src/editor/ui/scene_view/rendering/view_3d.rs`)

**เพิ่ม parameter สำหรับ tilemap settings**:
```rust
pub fn render_scene_3d(
    // ... existing parameters ...
    tilemap_settings: Option<&crate::editor::tilemap_settings::TilemapSettings>,
)
```

**ส่ง tilemap settings ไปให้ tilemap renderer**:
```rust
let mut tilemap_layers = tilemap_renderer.collect_tilemaps(world, tilemap_settings);
```

### 3. อัปเดต Scene View Module (`engine/src/editor/ui/scene_view/mod.rs`)

**เพิ่ม map_manager parameter**:
```rust
pub fn render_scene_view(
    // ... existing parameters ...
    map_manager: &crate::editor::map_manager::MapManager,
)
```

**ส่ง tilemap settings จาก map_manager**:
```rust
rendering::view_3d::render_scene_3d(
    // ... existing parameters ...
    Some(&map_manager.settings),
);
```

### 4. อัปเดต Dock Layout (`engine/src/editor/ui/dock_layout.rs`)

**ส่ง map_manager ไปให้ render_scene_view**:
```rust
scene_view::render_scene_view(
    // ... existing parameters ...
    self.context.map_manager,
);
```

### 5. อัปเดต UI Module (`engine/src/editor/ui/mod.rs`)

**สร้าง dummy map_manager สำหรับ fallback**:
```rust
let dummy_map_manager = crate::editor::map_manager::MapManager::new();

scene_view::render_scene_view(
    // ... existing parameters ...
    &dummy_map_manager,
);
```

## ผลลัพธ์ที่คาดหวัง

1. **ขนาดที่ถูกต้อง**: Tilemap ใน 3D mode จะมีขนาดเต็ม grid cells เหมือนใน 2D mode
2. **การจัดตำแหน่ง**: Tile boundaries จะตรงกับ grid lines อย่างสมบูรณ์
3. **ความสอดคล้อง**: ใช้ pixels_per_unit จาก tilemap settings อย่างถูกต้อง

## การทดสอบ

1. เปิด project ที่มี LDtk tilemap
2. สลับระหว่าง 2D และ 3D mode
3. ตรวจสอบว่า:
   - Tilemap มีขนาดเต็ม grid cells ทั้งใน 2D และ 3D mode
   - Tile boundaries ตรงกับ grid lines
   - ไม่มีความแตกต่างของขนาดระหว่าง 2 modes

## หมายเหตุ

- การแก้ไขนี้ใช้ `pixels_per_unit` จาก tilemap settings แทนค่า hard-coded
- Default value คือ `8.0` สำหรับ LDtk compatibility
- Map manager มี tilemap settings ที่สามารถปรับแต่งได้
- การเปลี่ยนแปลงนี้ไม่กระทบกับ 2D mode ที่ทำงานถูกต้องอยู่แล้ว

## ไฟล์ที่แก้ไข

1. `engine/src/editor/ui/scene_view/rendering/tilemap_3d.rs` - เพิ่ม tilemap settings parameter
2. `engine/src/editor/ui/scene_view/rendering/view_3d.rs` - ส่ง tilemap settings
3. `engine/src/editor/ui/scene_view/mod.rs` - เพิ่ม map_manager parameter
4. `engine/src/editor/ui/dock_layout.rs` - ส่ง map_manager
5. `engine/src/editor/ui/mod.rs` - สร้าง dummy map_manager

ตอนนี้ tilemap ใน 3D mode ควรจะมีขนาดเต็ม grid size เหมือนใน 2D mode แล้ว!
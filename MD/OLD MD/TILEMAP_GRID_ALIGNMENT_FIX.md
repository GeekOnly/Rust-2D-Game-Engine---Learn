# Tilemap Grid Alignment Fix

## ปัญหาที่พบ

ใน 3D mode tilemap ไม่ตรงกับ grid size แต่ใน 2D mode ตรงกัน

## สาเหตุของปัญหา

1. **ใน tilemap 3D renderer** มีการกำหนดขนาด tile เป็น 1.0 world unit แบบ hard-coded:
   ```rust
   let tile_world_width = 1.0;  // 1 tile = 1 grid cell = 1 world unit
   let tile_world_height = 1.0;
   ```

2. **ใน ldtk_loader** มีการสร้าง grid component ด้วย cell_size เป็น (1.0, 1.0, 0.0) แบบ hard-coded:
   ```rust
   cell_size: (1.0, 1.0, 0.0),  // Standard 1x1 world unit grid cells
   ```

3. **ไม่มีการใช้ pixels_per_unit** ในการคำนวณขนาดจริงของ tile ใน world units

## การแก้ไข

### 1. แก้ไข Tilemap 3D Renderer

**ไฟล์:** `engine/src/editor/ui/scene_view/rendering/tilemap_3d.rs`

**เปลี่ยนจาก:**
```rust
let tile_world_width = 1.0;  // 1 tile = 1 grid cell = 1 world unit
let tile_world_height = 1.0;
```

**เป็น:**
```rust
// Get tilemap settings for pixels_per_unit
let tilemap_settings = crate::editor::tilemap_settings::TilemapSettings::load(
    std::path::Path::new(".")
);
let tile_world_width = tileset.tile_width as f32 / tilemap_settings.pixels_per_unit;
let tile_world_height = tileset.tile_height as f32 / tilemap_settings.pixels_per_unit;
```

### 2. แก้ไข LDtk Loader

**ไฟล์:** `ecs/src/loaders/ldtk_loader.rs`

**เปลี่ยนจาก:**
```rust
let grid = crate::Grid {
    cell_size: (1.0, 1.0, 0.0),  // Standard 1x1 world unit grid cells
    // ...
};
```

**เป็น:**
```rust
// Use Unity standard pixels_per_unit (100.0) for consistent scaling
let pixels_per_unit = 100.0;
let cell_width = grid_size / pixels_per_unit;
let cell_height = grid_size / pixels_per_unit;

let grid = crate::Grid {
    cell_size: (cell_width, cell_height, 0.0),  // Match tile size in world units
    // ...
};
```

## ผลลัพธ์ที่คาดหวัง

1. **ใน 2D mode:** tilemap และ grid จะยังคงตรงกันเหมือนเดิม
2. **ใน 3D mode:** tilemap และ grid จะตรงกันแล้ว เพราะใช้ขนาดเดียวกัน
3. **ความสอดคล้อง:** ทั้ง tilemap และ grid ใช้ pixels_per_unit เดียวกัน (100.0 - Unity standard)

## การทดสอบ

1. เปิด project ที่มี LDtk tilemap
2. สลับระหว่าง 2D และ 3D mode
3. ตรวจสอบว่า tilemap ตรงกับ grid ในทั้งสองโหมด

## หมายเหตุ

- ใช้ Unity standard pixels_per_unit = 100.0 (100 pixels = 1 world unit)
- การเปลี่ยนแปลงนี้ไม่ควรส่งผลกระทบต่อ tilemap ที่มีอยู่แล้ว
- หาก tilemap settings มี pixels_per_unit ที่แตกต่าง ควรปรับให้ตรงกัน

## Warning ที่เกิดขึ้น

หลังจากการแก้ไข มี warning "Invalid point in projection" เยอะมาก ซึ่งอาจเป็นผลจาก:
1. การเปลี่ยนแปลงขนาด tile ทำให้การคำนวณ projection มีปัญหา
2. ต้องตรวจสอบและแก้ไข projection_3d.rs เพิ่มเติม

## ขั้นตอนถัดไป

1. ตรวจสอบและแก้ไข warning "Invalid point in projection"
2. ทดสอบการแสดงผล tilemap ใน 3D mode
3. ตรวจสอบว่า grid component แสดงขนาดที่ถูกต้อง
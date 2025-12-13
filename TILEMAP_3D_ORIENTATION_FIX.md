# Tilemap 3D Orientation Fix - การแก้ไขปัญหาทิศทางกลับกัน

## ปัญหาที่พบ

Tilemap ใน 3D mode กลับทิศทางกัน (flipped) เมื่อเทียบกับ 2D mode

## สาเหตุของปัญหา

การใช้ `egui::Rect::from_min_size()` แทน `egui::Rect::from_center_size()` ใน tilemap 3D renderer ทำให้:
- `screen_pos` ที่เป็น center position ถูกใช้เป็น min position
- ทำให้ tile ถูก render ที่ตำแหน่งผิด
- เกิดการกลับทิศทางของ tilemap

## การแก้ไข

### แก้ไข Tile Rect Creation (`engine/src/editor/ui/scene_view/rendering/tilemap_3d.rs`)

**เปลี่ยนจาก `from_min_size` เป็น `from_center_size`**:
```rust
// เดิม - ใช้ screen_pos เป็น min position (ผิด)
let rect = egui::Rect::from_min_size(
    egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
    egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
);

// ใหม่ - ใช้ screen_pos เป็น center position (ถูกต้อง)
// screen_pos is the center position, so we need to use from_center_size
let rect = egui::Rect::from_center_size(
    egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
    egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
);
```

## เหตุผลของการแก้ไข

1. **Screen Position คือ Center**: ใน `project_tile_to_screen()` ฟังก์ชัน `screen_pos` ถูกคำนวณจาก center ของ tile
2. **Projection Logic**: การ project world position ไปยัง screen space ให้ center position ไม่ใช่ min position
3. **Consistency**: ต้องใช้ `from_center_size` เพื่อให้สอดคล้องกับการคำนวณ screen position

## ผลลัพธ์ที่คาดหวัง

1. **ทิศทางที่ถูกต้อง**: Tilemap ใน 3D mode จะมีทิศทางเดียวกันกับ 2D mode
2. **ตำแหน่งที่แม่นยำ**: Tiles จะถูก render ที่ตำแหน่งที่ถูกต้อง
3. **การจัดตำแหน่ง**: Tile boundaries จะตรงกับ grid lines
4. **ความสอดคล้อง**: ไม่มีความแตกต่างของการวางตำแหน่งระหว่าง 2D และ 3D mode

## การทดสอบ

1. เปิด project ที่มี LDtk tilemap
2. สลับระหว่าง 2D และ 3D mode
3. ตรวจสอบว่า:
   - Tilemap มีทิศทางเดียวกันทั้งใน 2D และ 3D mode
   - Tiles อยู่ในตำแหน่งที่ถูกต้อง
   - ไม่มีการกลับทิศทาง (flip) ที่ไม่ต้องการ
   - Layout ของ tilemap ตรงกันระหว่าง 2 modes

## หมายเหตุ

- **Center vs Min Position**: ความแตกต่างระหว่าง center และ min position สำคัญมากในการ render
- **Projection Consistency**: การใช้ center position ทำให้สอดคล้องกับ projection logic
- **Egui Rect Methods**: `from_center_size` vs `from_min_size` ให้ผลลัพธ์ที่แตกต่างกัน
- **3D Rendering**: ใน 3D space การใช้ center position เป็นมาตรฐาน

## ไฟล์ที่แก้ไข

1. `engine/src/editor/ui/scene_view/rendering/tilemap_3d.rs` - เปลี่ยนจาก `from_min_size` เป็น `from_center_size`

ตอนนี้ tilemap ใน 3D mode ควรจะมีทิศทางที่ถูกต้องและไม่กลับกันแล้ว!
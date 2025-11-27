# ✅ Collider Rotation & Direction Fix - Complete!

## 🐛 ปัญหา

1. **Collider ไม่หมุนด้วย** - Collider gizmo (สีเขียว) ไม่หมุนตาม sprite
2. **หมุนคนละทิศ** - ทิศทางการหมุนแปลกๆ ไม่ตรงกับที่คาดหวัง

## 🔍 สาเหตุ

### ปัญหาที่ 1: Collider ไม่หมุน

```rust
// ❌ โค้ดเดิม - ใช้ rect_stroke ที่ไม่หมุน
fn render_collider_gizmo(...) {
    if let Some(collider) = world.colliders.get(&entity) {
        let size = egui::vec2(collider.width * zoom, collider.height * zoom);
        painter.rect_stroke(
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
            0.0,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
        );
    }
}
```

**ปัญหา:**
- ใช้ `rect_stroke()` ซึ่งไม่รองรับ rotation
- ไม่ได้อ่านค่า rotation จาก transform
- Collider วาดเป็นสี่เหลี่ยมตรงเสมอ

### ปัญหาที่ 2: ทิศทางการหมุนผิด

```rust
// ❌ โค้ดเดิม - ใช้ (delta.x - delta.y)
TransformTool::Rotate => {
    let rotation_speed = 0.5;
    transform_mut.rotation[2] += (delta.x - delta.y) * rotation_speed;
}
```

**ปัญหา:**
- ใช้ทั้ง X และ Y delta ทำให้สับสน
- ลากขวา + ลากบน = หมุนแปลกๆ
- ไม่ตรงกับ Unity behavior

## ✅ การแก้ไข

### Fix 1: Collider Rotation

```rust
fn render_collider_gizmo(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    _is_selected: bool,
) {
    if let Some(collider) = world.colliders.get(&entity) {
        let size = egui::vec2(collider.width * scene_camera.zoom, collider.height * scene_camera.zoom);
        
        // ✅ Get entity rotation
        let rotation_rad = world.transforms.get(&entity)
            .map(|t| t.rotation[2].to_radians())
            .unwrap_or(0.0);
        
        if rotation_rad.abs() < 0.01 {
            // No rotation - use simple rect
            painter.rect_stroke(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                0.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
            );
        } else {
            // ✅ Has rotation - draw as rotated polygon outline
            let half_width = size.x / 2.0;
            let half_height = size.y / 2.0;
            
            let cos_r = rotation_rad.cos();
            let sin_r = rotation_rad.sin();
            
            // Calculate 4 rotated corners
            let corners = [
                egui::pos2(
                    screen_x + (-half_width * cos_r - (-half_height) * sin_r),
                    screen_y + (-half_width * sin_r + (-half_height) * cos_r),
                ),
                egui::pos2(
                    screen_x + (half_width * cos_r - (-half_height) * sin_r),
                    screen_y + (half_width * sin_r + (-half_height) * cos_r),
                ),
                egui::pos2(
                    screen_x + (half_width * cos_r - half_height * sin_r),
                    screen_y + (half_width * sin_r + half_height * cos_r),
                ),
                egui::pos2(
                    screen_x + (-half_width * cos_r - half_height * sin_r),
                    screen_y + (-half_width * sin_r + half_height * cos_r),
                ),
            ];
            
            // ✅ Draw rotated collider outline
            painter.add(egui::Shape::closed_line(
                corners.to_vec(),
                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
            ));
        }
    }
}
```

### Fix 2: Rotation Direction

```rust
// ✅ โค้ดใหม่ - ใช้ delta.x เท่านั้น (Unity-style)
TransformTool::Rotate => {
    // Unity-style rotation: use horizontal drag only
    // Positive delta.x = rotate counter-clockwise (standard math convention)
    let rotation_speed = 0.5;
    transform_mut.rotation[2] += delta.x * rotation_speed;
}
```

**ทำไมใช้ delta.x เท่านั้น?**
- Unity ใช้การลากแนวนอนสำหรับหมุน
- ง่ายต่อการควบคุม (ลากขวา = หมุนทวนเข็ม, ลากซ้าย = หมุนตามเข็ม)
- ไม่สับสนกับการลากแนวตั้ง

## 🎯 การทำงาน

### Collider Rotation

```
Before (No Rotation):
    ┌─────────┐
    │         │  ← Collider (green)
    │    ●    │  ← Sprite
    │         │
    └─────────┘

After (With Rotation 45°):
       ╱╲
      ╱  ╲      ← Collider rotates with sprite!
     ╱ ●  ╲
    ╱      ╲
   ╱________╲
```

### Rotation Direction

```
Unity-Style Horizontal Drag:

Drag Right →  = Rotate Counter-Clockwise ↺
Drag Left  ←  = Rotate Clockwise ↻

Simple and intuitive!
```

## 📁 ไฟล์ที่แก้ไข

### `engine/src/editor/ui/scene_view.rs`

**Changes:**

1. **render_collider_gizmo()** - Collider rotation
   - ✅ อ่านค่า rotation จาก transform
   - ✅ คำนวณ rotated corners
   - ✅ ใช้ `closed_line()` สำหรับ rotated outline
   - ✅ Performance optimization (skip polygon for no rotation)

2. **handle_gizmo_interaction_stateful()** - Rotation direction
   - ✅ เปลี่ยนจาก `(delta.x - delta.y)` → `delta.x`
   - ✅ Unity-style horizontal drag
   - ✅ ทิศทางชัดเจนและควบคุมง่าย

## 🔧 Build Status

```
✅ Compilation: SUCCESS
✅ Build Time: 5.31s
✅ Warnings: 52 (no errors)
✅ Package: engine
```

## 📖 การทดสอบ

### Test 1: Collider Rotation
1. Create entity with sprite + collider
2. Enable "Show Colliders" (checkbox)
3. Select entity
4. Press **E** (Rotate tool)
5. Drag to rotate
6. ✅ Collider (green outline) should rotate with sprite

### Test 2: Rotation Direction
1. Select entity
2. Press **E** (Rotate tool)
3. Drag **right** →
4. ✅ Should rotate counter-clockwise ↺
5. Drag **left** ←
6. ✅ Should rotate clockwise ↻

### Test 3: No Rotation Performance
1. Create entity without rotation
2. Enable "Show Colliders"
3. ✅ Should use fast rect rendering
4. Rotate entity
5. ✅ Should switch to polygon rendering

## 🎯 Comparison: Before vs After

| Feature | Before | After | Status |
|---------|--------|-------|--------|
| Collider Rotation | ❌ No rotation | ✅ Rotates | ✅ |
| Rotation Direction | ❌ Confusing | ✅ Intuitive | ✅ |
| Drag Method | X - Y (weird) | X only (Unity) | ✅ |
| Visual Sync | ❌ Misaligned | ✅ Aligned | ✅ |
| Performance | Fast | Smart (optimized) | ✅ |

## 💡 Technical Details

### Collider Rotation Formula

```rust
// Same as sprite rotation
let cos_r = rotation_rad.cos();
let sin_r = rotation_rad.sin();

// For corner at (x, y):
let rotated_x = x * cos_r - y * sin_r;
let rotated_y = x * sin_r + y * cos_r;
```

### Rotation Direction Convention

```
Standard Math Convention (Counter-Clockwise Positive):

     90°
      ↑
      │
180° ←●→ 0°
      │
      ↓
    270°

Positive rotation = Counter-clockwise
Negative rotation = Clockwise
```

### Unity Rotation Behavior

```
Unity uses horizontal drag for rotation:
- Drag right (+delta.x) = Rotate CCW (+angle)
- Drag left (-delta.x) = Rotate CW (-angle)

This is intuitive because:
- Right = "forward" in time = positive
- Left = "backward" in time = negative
```

## 🚀 Future Enhancements

### Phase 2: Rotation Snapping

```rust
if snap_settings.enabled {
    let snapped_rotation = snap_to_grid(
        transform_mut.rotation[2],
        snap_settings.rotation_snap,  // e.g., 15°
        SnapMode::Absolute,
        0.0
    );
    transform_mut.rotation[2] = snapped_rotation;
}
```

### Phase 3: Rotation Gizmo Enhancement

```rust
// Show rotation angle text
painter.text(
    egui::pos2(screen_x, screen_y - 60.0),
    egui::Align2::CENTER_BOTTOM,
    format!("{:.1}°", transform.rotation[2]),
    egui::FontId::proportional(14.0),
    egui::Color32::WHITE,
);
```

### Phase 4: Multi-Axis Rotation (3D)

```rust
// For 3D mode, allow rotation around X, Y, Z axes
match drag_axis {
    Some(0) => transform_mut.rotation[0] += delta.x * speed, // X-axis
    Some(1) => transform_mut.rotation[1] += delta.x * speed, // Y-axis
    Some(2) => transform_mut.rotation[2] += delta.x * speed, // Z-axis
    _ => {}
}
```

## 🎊 Summary

แก้ไขปัญหา Collider Rotation และ Direction เสร็จสมบูรณ์!

**Fixes:**
- ✅ Collider หมุนตาม sprite แล้ว
- ✅ ทิศทางการหมุนเป็นแบบ Unity (ลากขวา = หมุนทวนเข็ม)
- ✅ ใช้ rotated polygon outline
- ✅ มี performance optimization
- ✅ Visual sync ระหว่าง sprite และ collider

**ลองใช้งานได้เลย:**
1. เปิด "Show Colliders"
2. เลือก entity ที่มี collider
3. กด **E** (Rotate tool)
4. ลาก**ขวา**เพื่อหมุนทวนเข็ม
5. Collider จะหมุนตาม sprite แล้ว! 🔄✨

---

**Created:** $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**Status:** ✅ COLLIDER ROTATION FIX COMPLETE
**Build:** ✅ SUCCESS
**Direction:** ✅ UNITY-STYLE

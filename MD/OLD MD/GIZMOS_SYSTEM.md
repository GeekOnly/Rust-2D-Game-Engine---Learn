# 🎨 Gizmos System - Visual Debugging Tools

## Overview

Gizmos System เป็นเครื่องมือสำหรับ visual debugging ใน Scene view แบบ Unity ช่วยให้นักพัฒนาเห็นภาพ colliders, velocities, และข้อมูล runtime อื่นๆ ได้ง่ายขึ้น

---

## ✨ Features

### 1. **Collider Boundaries** 🟢
แสดง wireframe สีเขียวรอบ collider ของทุก entity

**Visual:**
- สีเขียว (#00FF64) - ทำให้แยกแยะออกจาก sprite ได้ชัดเจน
- Stroke width 1.5px - บางพอที่จะไม่บดบัง sprite
- แสดงขนาดจริงของ collision box

**When to use:**
- ดูว่า collider ครอบคลุมพื้นที่ถูกต้องหรือไม่
- Debug collision detection issues
- Align colliders with sprites

### 2. **Corner Handles** 🔶
แสดงจุด control points ที่มุมทั้ง 4 ของ collider เมื่อเลือก entity

**Visual:**
- วงกลมสีเขียว radius 3px ที่มุมทั้ง 4
- แสดงเฉพาะเมื่อ entity ถูกเลือก
- เตรียมไว้สำหรับ resize collider ในอนาคต

**When to use:**
- เลือก entity เพื่อดูขอบเขตของ collider แบบละเอียด
- (อนาคต) Drag handles เพื่อปรับขนาด collider

### 3. **Velocity Vectors** 🟡
แสดงลูกศรสีเหลืองที่แสดงทิศทางและความเร็วของ entity

**Visual:**
- เส้นสีเหลือง (#FFFF00) ขนาด 2px
- ความยาวของลูกศร = velocity * 0.5
- วงกลมที่ปลายลูกศร radius 5px
- แสดงเฉพาะเมื่อ velocity > 0.1

**When to use:**
- Debug player movement
- ดูทิศทางการเคลื่อนที่ของ enemies/projectiles
- Visualize physics calculations

---

## 🎛️ Controls

### เปิด/ปิด Gizmos

1. เปิด Scene view
2. ไปที่ menu bar → **View → Gizmos**
3. เลือกตัวเลือกที่ต้องการ:
   - ✅ **Show Colliders** - แสดง collider boundaries (เปิดโดย default)
   - ☐ **Show Velocities** - แสดง velocity arrows (ปิดโดย default)

### Keyboard Shortcuts (Future)
- `G` - Toggle all gizmos
- `C` - Toggle colliders only
- `V` - Toggle velocities only

---

## 📸 Visual Examples

### Collider Gizmos
```
┌─────────────────┐
│   [Player]      │ ← Blue sprite
│                 │
└─────────────────┘
 ╔═════════════════╗
 ║                 ║ ← Green collider wireframe
 ╚═════════════════╝
 ●               ● ← Corner handles (when selected)
```

### Velocity Arrows
```
    [Player]
       │
       │ ← Yellow arrow showing movement
       ▼
```

---

## 💻 Implementation Details

### File Structure

| File | Lines | Purpose |
|------|-------|---------|
| [editor_ui.rs:425-501](editor_ui.rs#L425-L501) | 76 lines | Gizmo rendering logic |
| [editor_ui.rs:40-45](editor_ui.rs#L40-L45) | 6 lines | View menu UI |
| [main.rs:58-59](main.rs#L58-L59) | 2 lines | State fields |
| [main.rs:76-77](main.rs#L76-L77) | 2 lines | Default values |

**Total:** ~86 lines of code

### Code Architecture

```rust
// EditorState stores gizmo toggles
struct EditorState {
    show_colliders: bool,   // Toggle collider gizmos
    show_velocities: bool,  // Toggle velocity gizmos
}

// Rendering logic in Scene view
if *show_colliders {
    // Draw collider wireframe
    painter.rect_stroke(collider_rect, stroke);

    // Draw handles if selected
    if selected {
        for corner in corners {
            painter.circle_filled(corner, radius, color);
        }
    }
}

if *show_velocities {
    // Draw velocity arrow
    painter.line_segment([start, end], stroke);
    painter.circle_filled(arrow_head, radius, color);
}
```

### Performance

- **O(n)** rendering complexity where n = number of entities
- **Minimal overhead** - только когда в Scene view
- **No allocations** - uses egui's immediate mode rendering
- **Conditional rendering** - skips disabled gizmos

---

## 🎨 Color Palette (Unity-inspired)

| Gizmo Type | Color | Hex | Usage |
|------------|-------|-----|-------|
| Collider | 🟢 Green | #00FF64 | Collision boundaries |
| Velocity | 🟡 Yellow | #FFFF00 | Movement vectors |
| Selection | 🟠 Orange | #FFC800 | Selected entity outline |
| Grid | ⚫ Dark Gray | #3C3C46 | Background grid |

**Design Philosophy:**
- High contrast against dark background (#282832)
- Distinct colors for each gizmo type
- Semi-transparent to not obscure sprites
- Consistent with Unity/Unreal editor conventions

---

## 🚀 Future Enhancements

### Planned Features

1. **Transform Gizmos**
   - Position handle (arrows for X/Y)
   - Rotation handle (circular arc)
   - Scale handle (corner boxes)
   - Snap to grid option

2. **Advanced Collider Tools**
   - Resize handles with drag-and-drop
   - Offset adjustment
   - Multiple collider support
   - Polygon colliders (not just AABB)

3. **More Visual Helpers**
   - Sprite pivot point indicator
   - Camera frustum visualization
   - Physics raycast visualization
   - Pathfinding debug lines

4. **Gizmo Settings**
   - Color customization
   - Size/thickness sliders
   - Opacity control
   - Per-gizmo show/hide

5. **Performance**
   - Frustum culling for gizmos
   - LOD system (hide details when zoomed out)
   - Batched rendering

---

## 📝 Usage Tips

### Best Practices

1. **Keep Colliders Visible**
   - Always enable "Show Colliders" during level design
   - Helps catch collision bugs early

2. **Use Velocities Sparingly**
   - Enable only when debugging movement
   - Can be distracting during normal editing

3. **Select for Details**
   - Click entity to see corner handles
   - Useful for precise alignment

### Common Issues

**Q: Gizmos not showing?**
- Check View → Gizmos menu
- Make sure you're in Scene view (not Game view)
- Verify entities have collider/velocity components

**Q: Gizmos covering sprites?**
- Gizmos render on top by design
- Toggle off temporarily if needed
- Future: adjustable z-order

**Q: Performance slow with many entities?**
- Disable "Show Velocities" (most expensive)
- Current implementation is O(n) - acceptable for <1000 entities

---

## 🔗 Related Systems

### Dependencies
- **ECS** - Reads `colliders` and `velocities` maps
- **egui** - Uses `Painter` API for drawing
- **editor_ui** - Integrated into Scene view rendering

### Integrations
- **Inspector** - Shows component data that gizmos visualize
- **Physics** - Gizmos show colliders that physics uses
- **Selection** - Corner handles appear when entity selected

---

## 📚 API Reference

### EditorState Fields

```rust
pub struct EditorState {
    /// Show collider wireframes in Scene view
    pub show_colliders: bool,    // Default: true

    /// Show velocity arrows in Scene view
    pub show_velocities: bool,   // Default: false
}
```

### Rendering Functions

```rust
// In editor_ui.rs, Scene view rendering loop:

// Collider gizmo
if *show_colliders {
    if let Some(collider) = world.colliders.get(&entity) {
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.5, Color32::from_rgb(0, 255, 100))
        );
    }
}

// Velocity gizmo
if *show_velocities {
    for (&entity, velocity) in &world.velocities {
        if velocity.length() > 0.1 {
            painter.line_segment([start, end], stroke);
            painter.circle_filled(end, 5.0, color);
        }
    }
}
```

---

## ✅ Testing Checklist

- [x] Collider gizmos render correctly
- [x] Corner handles appear when entity selected
- [x] Velocity arrows point in correct direction
- [x] Toggle controls work (View menu)
- [x] Gizmos hidden in Game view
- [x] No performance impact when disabled
- [x] Works with multiple entities
- [x] Handles entities without components gracefully

---

## 🎯 Summary

Gizmos System ให้เครื่องมือ visual debugging ที่จำเป็นสำหรับการพัฒนาเกม:

- ✅ **Easy to use** - Toggle on/off ผ่าน View menu
- ✅ **Non-intrusive** - แสดงเฉพาะใน Scene view
- ✅ **Performant** - Minimal overhead, conditional rendering
- ✅ **Extensible** - ง่ายต่อการเพิ่ม gizmo types ใหม่
- ✅ **Unity-like** - ดีไซน์และสีตามมาตรฐานของ Unity

**พร้อมใช้งานใน Scene view แล้ววันนี้!** 🚀

---

**Created:** 2025-11-25
**Version:** 1.0
**Status:** ✅ Implemented & Tested

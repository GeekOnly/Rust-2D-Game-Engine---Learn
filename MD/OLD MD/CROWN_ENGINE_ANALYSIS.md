# Crown Engine Analysis - Scene View Features

## Overview
Crown Engine เป็น game engine แบบ data-driven ที่เขียนด้วย C++ โดย Level Editor ใช้ **Vala** (GTK-based) สำหรับ UI

## Key Features ที่พบใน Crown Level Editor

### 1. **Tool System** (Transform Tools)
```vala
public enum ToolType {
    PLACE,    // วาง object ใหม่
    MOVE,     // เลื่อน object
    ROTATE,   // หมุน object
    SCALE,    // ปรับขนาด object
}
```

**การใช้งาน:**
- แต่ละ tool มี icon แยกกัน (tool-place, tool-move, tool-rotate, tool-scale)
- ใช้ Toggle Button สำหรับสลับระหว่าง tools
- มี keyboard shortcuts (ไม่ระบุใน code แต่น่าจะมี Q, W, E, R เหมือน Unity)

### 2. **Snap System** (การจัดตำแหน่งแบบ Snap)
```vala
public enum SnapMode {
    RELATIVE,  // Snap แบบสัมพัทธ์ (จากตำแหน่งปัจจุบัน)
    ABSOLUTE   // Snap แบบสัมบูรณ์ (จาก grid origin)
}
```

**Features:**
- Snap to Grid (เปิด/ปิดได้)
- 2 โหมด: Relative และ Absolute
- มี icon แยกกัน (reference-local, reference-world)

### 3. **Reference System** (ระบบพิกัด)
```vala
public enum ReferenceSystem {
    LOCAL,  // พิกัดท้องถิ่น (ตาม object rotation)
    WORLD   // พิกัดโลก (แกน X, Y, Z คงที่)
}
```

**การใช้งาน:**
- Local Space: Gizmo หมุนตาม object
- World Space: Gizmo อยู่ในแนวแกนโลกเสมอ
- มี icon แยกกัน (axis-local, axis-world)

### 4. **Camera View Types** (มุมมองกล้อง)
```vala
public enum CameraViewType {
    PERSPECTIVE,  // มุมมอง Perspective (3D)
    FRONT,        // มองจากด้านหน้า
    BACK,         // มองจากด้านหลัง
    RIGHT,        // มองจากด้านขวา
    LEFT,         // มองจากด้านซ้าย
    TOP,          // มองจากด้านบน
    BOTTOM        // มองจากด้านล่าง
}
```

**Features:**
- 7 preset camera views
- สลับได้ง่ายผ่าน UI หรือ shortcuts
- Orthographic views สำหรับ Front/Back/Right/Left/Top/Bottom

### 5. **UI Layout**
```
┌─────────────────────────────────────────┐
│ Menu Bar                                │
├───┬─────────────────────────────────────┤
│ T │                                     │
│ o │                                     │
│ o │      Scene View                     │
│ l │      (3D Viewport)                  │
│ b │                                     │
│ a │                                     │
│ r │                                     │
└───┴─────────────────────────────────────┘
```

**Toolbar Position:**
- อยู่ด้านซ้ายของ scene view
- Vertical layout
- Grouped by function (Tools, Snap, Reference, Grid)
- มี spacing ระหว่าง groups

## Comparison กับ Scene View ปัจจุบัน

### ✅ Features ที่มีอยู่แล้ว
- ✅ Transform Tools (Move, Rotate, Scale, View)
- ✅ 2D/3D Mode switching
- ✅ Local/World space
- ✅ Camera controls (Pan, Orbit, Zoom)
- ✅ Grid rendering
- ✅ Scene gizmo (แสดงแกน XYZ)

### ❌ Features ที่ยังไม่มี
- ❌ **Place Tool** - สำหรับวาง object ใหม่
- ❌ **Snap to Grid** - การจัดตำแหน่งแบบ snap
- ❌ **Snap Mode** (Relative/Absolute)
- ❌ **Preset Camera Views** (Front, Top, Right, etc.)
- ❌ **Keyboard Shortcuts** สำหรับ tools (Q, W, E, R)
- ❌ **Grid Snapping Value** - ปรับค่า grid size
- ❌ **Visual Feedback** เมื่อ snap

## Recommendations สำหรับการปรับปรุง

### 1. เพิ่ม Snap to Grid System
```rust
pub struct SnapSettings {
    pub enabled: bool,
    pub mode: SnapMode,
    pub grid_size: f32,
    pub rotation_snap: f32,  // degrees
    pub scale_snap: f32,
}

pub enum SnapMode {
    Relative,
    Absolute,
}
```

### 2. เพิ่ม Preset Camera Views
```rust
impl SceneCamera {
    pub fn set_view_front(&mut self) {
        self.rotation = 0.0;
        self.pitch = 0.0;
    }
    
    pub fn set_view_top(&mut self) {
        self.rotation = 0.0;
        self.pitch = 90.0;
    }
    
    pub fn set_view_right(&mut self) {
        self.rotation = 90.0;
        self.pitch = 0.0;
    }
    
    // ... etc
}
```

### 3. เพิ่ม Keyboard Shortcuts
```rust
// ใน handle_keyboard_shortcuts()
if ui.input(|i| i.key_pressed(egui::Key::Q)) {
    *current_tool = TransformTool::View;
}
if ui.input(|i| i.key_pressed(egui::Key::W)) {
    *current_tool = TransformTool::Move;
}
if ui.input(|i| i.key_pressed(egui::Key::E)) {
    *current_tool = TransformTool::Rotate;
}
if ui.input(|i| i.key_pressed(egui::Key::R)) {
    *current_tool = TransformTool::Scale;
}
```

### 4. ปรับปรุง Toolbar UI
```rust
// เพิ่มปุ่ม Snap to Grid
ui.separator();
ui.checkbox(snap_enabled, "Snap to Grid");
ui.horizontal(|ui| {
    ui.label("Grid:");
    ui.add(egui::DragValue::new(grid_size).speed(0.1));
});
```

### 5. เพิ่ม Visual Feedback
- แสดง grid lines ชัดเจนขึ้นเมื่อ snap enabled
- แสดงตำแหน่ง snap point ด้วยจุดสีเหลือง
- แสดงค่า snap ใน status bar

## Implementation Priority

### High Priority (ควรทำก่อน)
1. ✅ Keyboard shortcuts (Q, W, E, R)
2. ✅ Snap to Grid system
3. ✅ Preset camera views (Numpad shortcuts)

### Medium Priority
4. Grid size adjustment UI
5. Snap mode (Relative/Absolute)
6. Visual snap feedback

### Low Priority
7. Place tool (สำหรับ drag & drop จาก asset browser)
8. Multiple selection
9. Gizmo customization

## Code Examples from Crown

### Toolbar Structure
```vala
// Crown uses GTK with vertical toolbar
public class Toolbar : Gtk.Box {
    // Tools group
    add_tool_buttons();
    
    // Snap group
    add_snap_buttons();
    
    // Reference system group
    add_reference_system_buttons();
    
    // Grid snap group
    add_snap_to_grid_buttons();
}
```

### Action System
Crown ใช้ GAction system ของ GTK:
```vala
btn.action_name = "app.tool";
btn.action_target = new GLib.Variant.int32(ToolType.MOVE);
```

สำหรับ Rust + egui เราใช้:
```rust
if ui.selectable_value(current_tool, TransformTool::Move, "➕ Move (W)").clicked() {
    // Tool changed
}
```

## Summary

Crown Engine มี scene view system ที่ดีมาก โดยเน้น:
- **Simplicity** - UI ง่าย ไม่ซับซ้อน
- **Consistency** - ใช้ icon และ layout ที่สอดคล้องกัน
- **Efficiency** - มี shortcuts ครบถ้วน
- **Flexibility** - มีหลาย modes และ views

การนำแนวคิดเหล่านี้มาใช้กับ engine ของคุณจะทำให้ scene view ใช้งานง่ายและมีประสิทธิภาพมากขึ้น!

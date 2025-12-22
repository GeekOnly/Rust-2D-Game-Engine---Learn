# Camera Settings UI System

## Overview
ระบบ UI สำหรับปรับแต่งการตั้งค่ากล้องแบบ real-time ด้วย sliders และ presets

## Features

### 1. Zoom Settings
- **Zoom Speed** (0.01-0.5): ความเร็วในการ zoom
- **Zoom Smoothness** (1.0-50.0): ความนุ่มนวลของการ zoom
- **Zoom Mode**: 
  - ☐ Zoom to Cursor (3D mode)
  - ☑ Zoom to Center (2D mode - default)

### 2. Pan Settings
- **Pan Speed** (0.1-5.0): ความเร็วในการเลื่อนกล้อง
- **Pan Smoothness** (0.0-0.5): damping ของการเลื่อน

### 3. Rotation Settings
- **Rotation Speed** (0.1-2.0): ความเร็วในการหมุนกล้อง

### 4. Presets System
- **Slow**: งานละเอียด (zoom: 0.08, pan: 0.5)
- **Normal**: ค่า default (zoom: 0.12, pan: 1.0)
- **Fast**: navigation เร็ว (zoom: 0.18, pan: 2.0)

### 5. Persistence
- บันทึกเป็น JSON: `.kiro/settings/camera_settings.json`
- โหลดอัตโนมัติเมื่อเริ่มต้น
- Reset to Default

## Implementation

### Files Structure
```
engine/src/editor/
├── ui/
│   ├── camera_settings.rs  # UI panel implementation
│   └── mod.rs              # Module export
├── camera.rs               # Camera with save/load methods
└── states.rs               # EditorState with show_camera_settings flag
```

### Usage Example

#### 1. Open Camera Settings Dialog
```rust
// In menu or toolbar
if ui.button("Camera Settings").clicked() {
    state.show_camera_settings = true;
}
```

#### 2. Render Settings Window
```rust
if state.show_camera_settings {
    egui::Window::new("🎥 Camera Settings")
        .open(&mut state.show_camera_settings)
        .show(ctx, |ui| {
            crate::editor::ui::camera_settings::render_camera_settings(
                ui,
                &mut state.scene_camera,
            );
        });
}
```

#### 3. Compact Toolbar Version
```rust
// In toolbar
crate::editor::ui::camera_settings::render_camera_settings_compact(
    ui,
    &mut state.scene_camera,
);
```

## API Reference

### Main Functions

#### `render_camera_settings(ui: &mut egui::Ui, camera: &mut Camera)`
แสดง full settings panel พร้อม:
- Zoom settings section
- Pan settings section
- Rotation settings section
- Presets buttons
- Advanced settings (collapsible)
- Save/Load/Reset buttons

#### `render_camera_settings_compact(ui: &mut egui::Ui, camera: &mut Camera)`
แสดง compact version สำหรับ toolbar:
- Zoom speed slider only
- Preset buttons
- Settings button (opens full dialog)

### Camera Methods

#### `camera.save_settings() -> Result<(), Box<dyn std::error::Error>>`
บันทึกการตั้งค่าเป็น JSON file

#### `camera.load_settings() -> Result<(), Box<dyn std::error::Error>>`
โหลดการตั้งค่าจาก JSON file

#### `camera.reset_settings_to_default()`
รีเซ็ตการตั้งค่ากลับเป็นค่า default

## Settings Ranges

### Zoom Speed (0.01-0.5)
- **0.01-0.05**: Very Slow - สำหรับงานละเอียดมาก
- **0.06-0.10**: Slow - งานละเอียด
- **0.11-0.15**: Normal - ใช้งานทั่วไป (default: 0.12)
- **0.16-0.30**: Fast - navigation เร็ว
- **0.31-0.50**: Very Fast - navigation เร็วมาก

### Pan Speed (0.1-5.0)
- **0.1-0.5**: Slow - งานละเอียด
- **0.6-1.5**: Normal - ใช้งานทั่วไป (default: 1.0)
- **1.6-3.0**: Fast - navigation เร็ว
- **3.1-5.0**: Very Fast - navigation เร็วมาก

### Zoom Smoothness (1.0-50.0)
- **1.0-10.0**: Instant - ไม่มี smoothing
- **11.0-30.0**: Normal - smoothing ปานกลาง (default: 20.0)
- **31.0-50.0**: Smooth - smoothing มาก

## Integration Guide

### Step 1: Add Menu Item
```rust
ui.menu_button("View", |ui| {
    if ui.button("🎥 Camera Settings...").clicked() {
        state.show_camera_settings = true;
        ui.close_menu();
    }
});
```

### Step 2: Add Keyboard Shortcut
```rust
if ctx.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl && i.modifiers.shift) {
    state.show_camera_settings = !state.show_camera_settings;
}
```

### Step 3: Add Toolbar Button
```rust
if ui.button("⚙ Camera").clicked() {
    state.show_camera_settings = true;
}
```

## Benefits

### For Users
- ✅ ปรับ camera zoom speed ได้แบบ real-time
- ✅ UI ที่ใช้งานง่ายด้วย sliders
- ✅ Preset system สำหรับ setup เร็ว
- ✅ บันทึกการตั้งค่าถาวร
- ✅ ใช้งานได้ทั้ง 2D และ 3D mode

### For Developers
- ✅ Modular design - แยก UI จาก logic
- ✅ Easy integration - เพียง 3 บรรทัด
- ✅ Extensible - เพิ่ม settings ใหม่ได้ง่าย
- ✅ Type-safe - ใช้ Rust type system
- ✅ Persistent - JSON serialization

## Troubleshooting

### ปัญหา: Zoom ไม่ถูกต้อง
**วิธีแก้**: เปลี่ยน Zoom Mode เป็น "Zoom to Center" สำหรับ 2D mode

### ปัญหา: Settings ไม่บันทึก
**วิธีแก้**: ตรวจสอบว่ามี folder `.kiro/settings/` หรือไม่

### ปัญหา: Zoom เร็วเกินไป
**วิธีแก้**: ลด Zoom Speed หรือใช้ Preset "Slow"

## Future Enhancements

### Planned Features
- [ ] Per-project settings
- [ ] Multiple camera profiles
- [ ] Import/Export settings
- [ ] Keyboard shortcut customization
- [ ] Mouse button mapping
- [ ] Touch gesture support
- [ ] VR camera controls

### Advanced Settings
- [ ] Field of View (FOV) slider
- [ ] Near/Far plane controls
- [ ] Orthographic size control
- [ ] Camera shake settings
- [ ] Motion blur settings

## Technical Details

### Dependencies
- `egui`: UI framework
- `serde`: JSON serialization
- `serde_json`: JSON format

### Performance
- Zero-cost abstractions
- No heap allocations in hot path
- Efficient slider updates
- Minimal UI overhead

### Compatibility
- Works with both 2D and 3D cameras
- Compatible with all editor modes
- No breaking changes to existing code

## Conclusion

Camera Settings UI ช่วยให้ users สามารถปรับแต่งพฤติกรรมของกล้องได้ตามต้องการ ด้วย UI ที่ใช้งานง่ายและ settings ที่บันทึกถาวร ทำให้ workflow ในการทำงานกับ editor ดีขึ้นอย่างมาก! 🎉

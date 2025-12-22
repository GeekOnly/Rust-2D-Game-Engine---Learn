# Scene View Camera Controls

## Overview
UI controls สำหรับปรับ camera sensitivity แบบ real-time ใน Scene View

## Features

### 1. Scene View Overlay Controls
Minimal UI สำหรับแสดงใน Scene View toolbar:
- 🔍 Zoom Speed slider (0.01-0.5)
- 🖱 Pan Speed slider (0.1-5.0)
- Quick preset buttons (S/N/F)

### 2. Improved Pan Behavior
- Simplified pan calculation for 2D mode
- Direct X/Y movement (no rotation calculation)
- Immediate response (no damping delay)
- Grid and objects stay aligned

### 3. Zoom to Cursor (Default)
- Zoom in/out towards mouse cursor position
- Better for precise editing
- World position under cursor stays fixed

## API

### Scene View Controls
```rust
use crate::editor::ui::camera_settings;

// In Scene View toolbar
camera_settings::render_scene_view_controls(ui, &mut camera);
```

### Compact Version
```rust
// Slightly larger version with labels
camera_settings::render_camera_settings_compact(ui, &mut camera);
```

### Full Settings Panel
```rust
// Complete settings dialog
camera_settings::render_camera_settings(ui, &mut camera);
```

## Usage Example

### Integration in Scene View
```rust
// In scene view rendering code
egui::TopBottomPanel::top("scene_toolbar").show(ctx, |ui| {
    ui.horizontal(|ui| {
        // Other toolbar buttons...
        
        ui.separator();
        
        // Camera controls
        crate::editor::ui::camera_settings::render_scene_view_controls(
            ui,
            &mut state.scene_camera,
        );
    });
});
```

## Controls

### Zoom Speed (🔍)
- **Range**: 0.01 - 0.5
- **Default**: 0.12
- **Effect**: How fast zoom responds to scroll wheel
- **Tooltip**: Shows current value on hover

### Pan Speed (🖱)
- **Range**: 0.1 - 5.0
- **Default**: 1.0
- **Effect**: How fast camera moves when panning
- **Tooltip**: Shows current value on hover

### Quick Presets
- **S** - Slow: Precise work (zoom: 0.08, pan: 0.5)
- **N** - Normal: General editing (zoom: 0.12, pan: 1.0)
- **F** - Fast: Quick navigation (zoom: 0.18, pan: 2.0)

## Pan Behavior Fix

### Problem
Middle mouse pan caused grid and objects to misalign due to:
- Complex rotation calculations in 2D mode
- Damping causing delayed response
- Inconsistent world space conversion

### Solution
Simplified pan calculation:
```rust
// Direct X/Y movement
let pan_speed = self.settings.pan_sensitivity / self.zoom;
let world_delta = Vec2::new(-delta.x, delta.y) * pan_speed;
self.position += world_delta;
```

### Benefits
- ✅ Grid and objects stay aligned
- ✅ Immediate response (no lag)
- ✅ Predictable movement
- ✅ Works correctly at all zoom levels

## Zoom Behavior

### Zoom to Cursor (Default)
- World position under cursor stays fixed
- Better for precise editing
- Zoom in/out towards mouse position

### How it Works
1. Store world position under cursor before zoom
2. Apply zoom factor
3. Calculate screen offset after zoom
4. Adjust camera position to keep world position under cursor

## Integration Guide

### Step 1: Add to Scene View Toolbar
```rust
egui::TopBottomPanel::top("scene_toolbar").show(ctx, |ui| {
    ui.horizontal(|ui| {
        // Existing buttons...
        ui.separator();
        
        // Camera controls (minimal)
        crate::editor::ui::camera_settings::render_scene_view_controls(
            ui,
            &mut state.scene_camera,
        );
    });
});
```

### Step 2: Add Settings Button
```rust
if ui.button("⚙ Camera").clicked() {
    state.show_camera_settings = true;
}
```

### Step 3: Render Settings Dialog
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

## Troubleshooting

### Pan feels too fast/slow
- Adjust Pan Speed slider (🖱)
- Or use preset buttons (S/N/F)

### Zoom feels too fast/slow
- Adjust Zoom Speed slider (🔍)
- Or use preset buttons (S/N/F)

### Grid and objects misaligned
- This should be fixed now with simplified pan
- If still occurs, check zoom level and pan sensitivity

### Want zoom to center instead of cursor
- Open Camera Settings (⚙)
- Change Zoom Mode to "🎯 Zoom to Center"

## Technical Details

### Pan Calculation
```rust
// Simple 2D pan (no rotation)
let pan_speed = pan_sensitivity / zoom;
let world_delta = Vec2::new(-delta.x, delta.y) * pan_speed;
position += world_delta;
```

### Zoom Calculation
```rust
// Exponential zoom with cursor tracking
let zoom_factor = 1.0 + zoom_sensitivity;
target_zoom *= zoom_factor;

// Adjust position to keep cursor point fixed
if zoom_to_cursor {
    let world_pos = screen_to_world(mouse_pos);
    // ... adjust camera position ...
}
```

## Conclusion

Scene View Camera Controls ให้ users สามารถปรับ camera sensitivity ได้แบบ real-time โดยไม่ต้องเปิด settings dialog พร้อมแก้ไขปัญหา pan ที่ทำให้ grid/objects ไม่ตรงกัน! 🎉

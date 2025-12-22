# Scene View Improvements - Based on Crown Engine

## การปรับปรุงที่แนะนำ

### 1. เพิ่ม Snap to Grid System

#### เพิ่ม Struct สำหรับ Snap Settings
```rust
// ใน engine/src/editor/mod.rs หรือ scene_view.rs
#[derive(Debug, Clone)]
pub struct SnapSettings {
    pub enabled: bool,
    pub mode: SnapMode,
    pub position_snap: f32,    // Grid size for position (e.g., 1.0)
    pub rotation_snap: f32,    // Degrees for rotation (e.g., 15.0)
    pub scale_snap: f32,       // Increment for scale (e.g., 0.1)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapMode {
    Relative,  // Snap relative to current position
    Absolute,  // Snap to absolute grid positions
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: SnapMode::Absolute,
            position_snap: 1.0,
            rotation_snap: 15.0,
            scale_snap: 0.1,
        }
    }
}
```

#### เพิ่ม Snap Function
```rust
// Helper function to snap value to grid
fn snap_to_grid(value: f32, grid_size: f32, mode: SnapMode, original: f32) -> f32 {
    if grid_size <= 0.0 {
        return value;
    }
    
    match mode {
        SnapMode::Absolute => {
            // Snap to absolute grid positions
            (value / grid_size).round() * grid_size
        }
        SnapMode::Relative => {
            // Snap relative to original position
            let delta = value - original;
            let snapped_delta = (delta / grid_size).round() * grid_size;
            original + snapped_delta
        }
    }
}
```

#### ปรับปรุง Gizmo Interaction
```rust
// ใน handle_gizmo_interaction_stateful()
if response.dragged() && *dragging_entity == Some(entity) {
    let delta = response.drag_delta();
    let screen_delta = glam::Vec2::new(delta.x, delta.y);
    
    // ... calculate world_delta_x, world_delta_y ...
    
    if let Some(transform_mut) = world.transforms.get_mut(&entity) {
        match current_tool {
            TransformTool::Move => {
                if let Some(axis) = *drag_axis {
                    let new_x = transform_mut.position[0] + world_delta_x;
                    let new_y = transform_mut.position[1] - world_delta_y;
                    
                    // Apply snapping if enabled
                    if snap_settings.enabled {
                        match axis {
                            0 => {
                                transform_mut.position[0] = snap_to_grid(
                                    new_x,
                                    snap_settings.position_snap,
                                    snap_settings.mode,
                                    drag_start_position.x  // Store this when drag starts
                                );
                            }
                            1 => {
                                transform_mut.position[1] = snap_to_grid(
                                    new_y,
                                    snap_settings.position_snap,
                                    snap_settings.mode,
                                    drag_start_position.y
                                );
                            }
                            2 => {
                                transform_mut.position[0] = snap_to_grid(new_x, snap_settings.position_snap, snap_settings.mode, drag_start_position.x);
                                transform_mut.position[1] = snap_to_grid(new_y, snap_settings.position_snap, snap_settings.mode, drag_start_position.y);
                            }
                            _ => {}
                        }
                    } else {
                        // No snapping
                        match axis {
                            0 => transform_mut.position[0] = new_x,
                            1 => transform_mut.position[1] = new_y,
                            2 => {
                                transform_mut.position[0] = new_x;
                                transform_mut.position[1] = new_y;
                            }
                            _ => {}
                        }
                    }
                }
            }
            TransformTool::Rotate => {
                let rotation_speed = 0.01;
                let new_rotation = transform_mut.rotation[2] + (delta.x - delta.y) * rotation_speed;
                
                if snap_settings.enabled {
                    transform_mut.rotation[2] = snap_to_grid(
                        new_rotation,
                        snap_settings.rotation_snap,
                        SnapMode::Absolute,  // Rotation always absolute
                        0.0
                    );
                } else {
                    transform_mut.rotation[2] = new_rotation;
                }
            }
            TransformTool::Scale => {
                let scale_speed = 0.01;
                let scale_delta = (delta.x + delta.y) * scale_speed;
                let new_scale_x = transform_mut.scale[0] + scale_delta;
                let new_scale_y = transform_mut.scale[1] + scale_delta;
                
                if snap_settings.enabled {
                    transform_mut.scale[0] = snap_to_grid(new_scale_x, snap_settings.scale_snap, SnapMode::Absolute, 1.0).max(0.1);
                    transform_mut.scale[1] = snap_to_grid(new_scale_y, snap_settings.scale_snap, SnapMode::Absolute, 1.0).max(0.1);
                } else {
                    transform_mut.scale[0] = new_scale_x.max(0.1);
                    transform_mut.scale[1] = new_scale_y.max(0.1);
                }
            }
            _ => {}
        }
    }
}
```

### 2. เพิ่ม Keyboard Shortcuts

```rust
// เพิ่มฟังก์ชันใหม่ใน scene_view.rs
fn handle_keyboard_shortcuts(
    ui: &mut egui::Ui,
    current_tool: &mut TransformTool,
    scene_camera: &mut SceneCamera,
    snap_settings: &mut SnapSettings,
) {
    // Tool shortcuts (Unity-like)
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
    
    // Camera view shortcuts (Numpad)
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        scene_camera.set_view_front();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
        scene_camera.set_view_right();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num7)) {
        scene_camera.set_view_top();
    }
    
    // Ctrl + Num for opposite views
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Num1)) {
        scene_camera.set_view_back();
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Num3)) {
        scene_camera.set_view_left();
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Num7)) {
        scene_camera.set_view_bottom();
    }
    
    // Toggle snap with Ctrl+G
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
        snap_settings.enabled = !snap_settings.enabled;
    }
}
```

### 3. เพิ่ม Preset Camera Views

```rust
// ใน engine/src/editor/camera.rs
impl SceneCamera {
    pub fn set_view_front(&mut self) {
        self.rotation = 0.0;
        self.pitch = 0.0;
    }
    
    pub fn set_view_back(&mut self) {
        self.rotation = 180.0;
        self.pitch = 0.0;
    }
    
    pub fn set_view_right(&mut self) {
        self.rotation = 90.0;
        self.pitch = 0.0;
    }
    
    pub fn set_view_left(&mut self) {
        self.rotation = -90.0;
        self.pitch = 0.0;
    }
    
    pub fn set_view_top(&mut self) {
        self.rotation = 0.0;
        self.pitch = 90.0;
    }
    
    pub fn set_view_bottom(&mut self) {
        self.rotation = 0.0;
        self.pitch = -90.0;
    }
    
    pub fn set_view_perspective(&mut self) {
        self.rotation = 45.0;
        self.pitch = 30.0;
    }
}
```

### 4. ปรับปรุง Toolbar UI

```rust
fn render_scene_toolbar(
    ui: &mut egui::Ui,
    current_tool: &mut TransformTool,
    is_playing: bool,
    play_request: &mut bool,
    stop_request: &mut bool,
    scene_view_mode: &mut SceneViewMode,
    transform_space: &mut TransformSpace,
    snap_settings: &mut SnapSettings,  // เพิ่ม parameter
) {
    ui.horizontal(|ui| {
        // Transform tools
        ui.selectable_value(current_tool, TransformTool::View, "🖐 View (Q)");
        ui.selectable_value(current_tool, TransformTool::Move, "➕ Move (W)");
        ui.selectable_value(current_tool, TransformTool::Rotate, "🔄 Rotate (E)");
        ui.selectable_value(current_tool, TransformTool::Scale, "📏 Scale (R)");
        
        ui.separator();
        
        // 2D/3D toggle
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode2D, "2D");
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode3D, "3D");
        
        ui.separator();
        
        // Space: Local/World toggle
        ui.label("Space:");
        ui.selectable_value(transform_space, TransformSpace::Local, "Local");
        ui.selectable_value(transform_space, TransformSpace::World, "World");
        
        ui.separator();
        
        // Snap settings
        ui.checkbox(&mut snap_settings.enabled, "Snap");
        if snap_settings.enabled {
            ui.label("Grid:");
            ui.add(egui::DragValue::new(&mut snap_settings.position_snap)
                .speed(0.1)
                .clamp_range(0.1..=10.0));
        }
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Play/Stop buttons
            if !is_playing {
                if ui.button("▶ Play").clicked() {
                    *play_request = true;
                }
            } else {
                if ui.button("⏹ Stop").clicked() {
                    *stop_request = true;
                }
            }
        });
    });
    ui.separator();
}
```

### 5. เพิ่ม Visual Snap Feedback

```rust
// ใน render_grid_2d() - ทำให้ grid ชัดเจนขึ้นเมื่อ snap enabled
fn render_grid_2d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
    snap_settings: &SnapSettings,  // เพิ่ม parameter
) {
    let grid_size = if snap_settings.enabled {
        snap_settings.position_snap * scene_camera.zoom
    } else {
        scene_grid.size * scene_camera.zoom
    };
    
    let grid_color = if snap_settings.enabled {
        // ทำให้สีชัดขึ้นเมื่อ snap enabled
        egui::Color32::from_rgba_premultiplied(
            (scene_grid.color[0] * 255.0) as u8,
            (scene_grid.color[1] * 255.0) as u8,
            (scene_grid.color[2] * 255.0) as u8,
            ((scene_grid.color[3] * 255.0) as u8).max(150),  // Alpha ขั้นต่ำ 150
        )
    } else {
        egui::Color32::from_rgba_premultiplied(
            (scene_grid.color[0] * 255.0) as u8,
            (scene_grid.color[1] * 255.0) as u8,
            (scene_grid.color[2] * 255.0) as u8,
            (scene_grid.color[3] * 255.0) as u8,
        )
    };
    
    // ... rest of grid rendering ...
}

// แสดง snap point เมื่อกำลัง drag
fn render_snap_preview(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    snap_settings: &SnapSettings,
) {
    if snap_settings.enabled {
        // วงกลมสีเหลืองแสดงตำแหน่ง snap
        painter.circle_filled(screen_pos, 4.0, egui::Color32::from_rgb(255, 255, 0));
        painter.circle_stroke(screen_pos, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)));
    }
}
```

### 6. เพิ่ม Camera View Menu

```rust
// เพิ่มเมนูสำหรับเลือก camera view
fn render_camera_view_menu(
    ui: &mut egui::Ui,
    scene_camera: &mut SceneCamera,
) {
    ui.menu_button("View", |ui| {
        if ui.button("🎥 Perspective").clicked() {
            scene_camera.set_view_perspective();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("⬅ Front (Num1)").clicked() {
            scene_camera.set_view_front();
            ui.close_menu();
        }
        if ui.button("➡ Back (Ctrl+Num1)").clicked() {
            scene_camera.set_view_back();
            ui.close_menu();
        }
        if ui.button("⬆ Right (Num3)").clicked() {
            scene_camera.set_view_right();
            ui.close_menu();
        }
        if ui.button("⬇ Left (Ctrl+Num3)").clicked() {
            scene_camera.set_view_left();
            ui.close_menu();
        }
        if ui.button("🔼 Top (Num7)").clicked() {
            scene_camera.set_view_top();
            ui.close_menu();
        }
        if ui.button("🔽 Bottom (Ctrl+Num7)").clicked() {
            scene_camera.set_view_bottom();
            ui.close_menu();
        }
    });
}
```

## การอัพเดท render_scene_view Function

```rust
pub fn render_scene_view(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    _scene_view_tab: &mut usize,
    is_playing: bool,
    show_colliders: &bool,
    show_velocities: &bool,
    current_tool: &mut TransformTool,
    scene_camera: &mut SceneCamera,
    scene_grid: &SceneGrid,
    play_request: &mut bool,
    stop_request: &mut bool,
    dragging_entity: &mut Option<Entity>,
    drag_axis: &mut Option<u8>,
    scene_view_mode: &mut SceneViewMode,
    projection_mode: &mut ProjectionMode,
    transform_space: &mut TransformSpace,
    snap_settings: &mut SnapSettings,  // เพิ่ม parameter
) {
    // Track previous mode
    let previous_mode = *scene_view_mode;
    
    // Toolbar with snap settings
    render_scene_toolbar(
        ui,
        current_tool,
        is_playing,
        play_request,
        stop_request,
        scene_view_mode,
        transform_space,
        snap_settings,  // ส่งต่อ
    );
    
    // Handle keyboard shortcuts
    handle_keyboard_shortcuts(ui, current_tool, scene_camera, snap_settings);
    
    // ... rest of the function ...
    
    // Render grid with snap feedback
    if scene_grid.enabled {
        match scene_view_mode {
            SceneViewMode::Mode2D => render_grid_2d(&painter, rect, scene_camera, scene_grid, snap_settings),
            SceneViewMode::Mode3D => render_grid_3d(&painter, rect, scene_camera, scene_grid, snap_settings),
        }
    }
    
    // ... rest of rendering ...
}
```

## Summary

การปรับปรุงเหล่านี้จะทำให้ scene view ของคุณ:

1. ✅ **ใช้งานง่ายขึ้น** - มี keyboard shortcuts ครบถ้วน
2. ✅ **แม่นยำขึ้น** - มี snap to grid system
3. ✅ **ยืดหยุ่นขึ้น** - มี preset camera views
4. ✅ **มืออาชีพขึ้น** - UI และ workflow เหมือน Unity/Unreal

ทั้งหมดนี้เป็นแนวทางจาก Crown Engine ที่ปรับให้เข้ากับ Rust + egui!

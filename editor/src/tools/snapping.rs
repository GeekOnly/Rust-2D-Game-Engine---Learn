//! Snapping System
//!
//! Handles snap-to-grid functionality for precise positioning.
//! Supports:
//! - Position snapping
//! - Rotation snapping
//! - Scale snapping
//! - Configurable grid sizes
//! - Visual feedback
//! - Toggle with Ctrl key

use serde::{Serialize, Deserialize};

/// Snap mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SnapMode {
    /// Snap to absolute grid positions
    Absolute,
    /// Snap relative to drag start position
    Relative,
}

/// Snapping settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSettings {
    // Enable/disable
    pub enabled: bool,
    pub mode: SnapMode,
    
    // Grid sizes
    pub position_snap: f32,    // World units (e.g., 1.0)
    pub rotation_snap: f32,    // Degrees (e.g., 15.0)
    pub scale_snap: f32,       // Scale increment (e.g., 0.1)
    
    // Visual
    pub show_grid: bool,
    pub grid_color: [f32; 4],
    pub snap_indicator_color: [f32; 4],
    
    // Behavior
    pub snap_on_create: bool,  // Snap when creating entities
    pub snap_on_move: bool,    // Snap when moving
    pub snap_on_rotate: bool,  // Snap when rotating
    pub snap_on_scale: bool,   // Snap when scaling
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: SnapMode::Absolute,
            position_snap: 1.0, // 1 world unit = 1 LDtk cell (8x8 pixels)
            rotation_snap: 15.0,
            scale_snap: 0.1,
            show_grid: true,
            grid_color: [0.3, 0.3, 0.3, 0.5],
            snap_indicator_color: [1.0, 1.0, 0.0, 0.8],
            snap_on_create: true,
            snap_on_move: true,
            snap_on_rotate: true,
            snap_on_scale: true,
        }
    }
}

impl SnapSettings {
    /// Create with custom grid size
    pub fn with_grid_size(grid_size: f32) -> Self {
        Self {
            position_snap: grid_size,
            ..Default::default()
        }
    }
    
    /// Common presets
    pub fn preset_fine() -> Self {
        Self {
            position_snap: 0.25,
            rotation_snap: 5.0,
            scale_snap: 0.05,
            ..Default::default()
        }
    }
    
    pub fn preset_normal() -> Self {
        Self::default()
    }
    
    pub fn preset_ldtk() -> Self {
        Self {
            position_snap: 1.0, // 1 world unit = 1 LDtk cell (8x8 pixels)
            rotation_snap: 15.0,
            scale_snap: 0.1,
            ..Default::default()
        }
    }
    
    pub fn preset_coarse() -> Self {
        Self {
            position_snap: 5.0,
            rotation_snap: 45.0,
            scale_snap: 0.5,
            ..Default::default()
        }
    }
    
    /// Load from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::path::Path::new(".kiro/settings/snap_settings.json");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::path::Path::new(".kiro/settings");
        std::fs::create_dir_all(dir)?;
        
        let path = dir.join("snap_settings.json");
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

// ============================================================================
// SNAPPING FUNCTIONS
// ============================================================================

/// Snap value to grid
pub fn snap_value(value: f32, grid_size: f32, mode: SnapMode, original: f32) -> f32 {
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

/// Snap position (3D vector)
pub fn snap_position(
    position: [f32; 3],
    settings: &SnapSettings,
    original: Option<[f32; 3]>,
) -> [f32; 3] {
    if !settings.enabled || !settings.snap_on_move {
        return position;
    }
    
    let orig = original.unwrap_or(position);
    
    [
        snap_value(position[0], settings.position_snap, settings.mode, orig[0]),
        snap_value(position[1], settings.position_snap, settings.mode, orig[1]),
        snap_value(position[2], settings.position_snap, settings.mode, orig[2]),
    ]
}

/// Snap rotation (3D vector in degrees)
pub fn snap_rotation(
    rotation: [f32; 3],
    settings: &SnapSettings,
    original: Option<[f32; 3]>,
) -> [f32; 3] {
    if !settings.enabled || !settings.snap_on_rotate {
        return rotation;
    }
    
    let orig = original.unwrap_or(rotation);
    
    [
        snap_value(rotation[0], settings.rotation_snap, settings.mode, orig[0]),
        snap_value(rotation[1], settings.rotation_snap, settings.mode, orig[1]),
        snap_value(rotation[2], settings.rotation_snap, settings.mode, orig[2]),
    ]
}

/// Snap scale (3D vector)
pub fn snap_scale(
    scale: [f32; 3],
    settings: &SnapSettings,
    original: Option<[f32; 3]>,
) -> [f32; 3] {
    if !settings.enabled || !settings.snap_on_scale {
        return scale;
    }
    
    let orig = original.unwrap_or(scale);
    
    [
        snap_value(scale[0], settings.scale_snap, settings.mode, orig[0]).max(0.01),
        snap_value(scale[1], settings.scale_snap, settings.mode, orig[1]).max(0.01),
        snap_value(scale[2], settings.scale_snap, settings.mode, orig[2]).max(0.01),
    ]
}

/// Snap 2D position
pub fn snap_position_2d(
    position: [f32; 2],
    settings: &SnapSettings,
    original: Option<[f32; 2]>,
) -> [f32; 2] {
    if !settings.enabled || !settings.snap_on_move {
        return position;
    }
    
    let orig = original.unwrap_or(position);
    
    [
        snap_value(position[0], settings.position_snap, settings.mode, orig[0]),
        snap_value(position[1], settings.position_snap, settings.mode, orig[1]),
    ]
}

/// Snap single rotation value (degrees)
pub fn snap_rotation_single(
    rotation: f32,
    settings: &SnapSettings,
    original: Option<f32>,
) -> f32 {
    if !settings.enabled || !settings.snap_on_rotate {
        return rotation;
    }
    
    let orig = original.unwrap_or(rotation);
    snap_value(rotation, settings.rotation_snap, settings.mode, orig)
}

/// Snap single scale value
pub fn snap_scale_single(
    scale: f32,
    settings: &SnapSettings,
    original: Option<f32>,
) -> f32 {
    if !settings.enabled || !settings.snap_on_scale {
        return scale;
    }
    
    let orig = original.unwrap_or(scale);
    snap_value(scale, settings.scale_snap, settings.mode, orig).max(0.01)
}

// ============================================================================
// VISUAL FEEDBACK
// ============================================================================

/// Render snap grid
pub fn render_snap_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &crate::SceneCamera,
    settings: &SnapSettings,
) {
    if !settings.show_grid || !settings.enabled {
        return;
    }
    
    let grid_size = settings.position_snap;
    if grid_size <= 0.0 {
        return;
    }
    
    let center = rect.center();
    let zoom = scene_camera.zoom;
    
    // Calculate visible grid range
    let half_width = rect.width() / 2.0;
    let half_height = rect.height() / 2.0;
    
    let world_half_width = half_width / zoom;
    let world_half_height = half_height / zoom;
    
    let cam_pos = scene_camera.position;
    
    let min_x = ((cam_pos.x - world_half_width) / grid_size).floor() * grid_size;
    let max_x = ((cam_pos.x + world_half_width) / grid_size).ceil() * grid_size;
    let min_y = ((cam_pos.y - world_half_height) / grid_size).floor() * grid_size;
    let max_y = ((cam_pos.y + world_half_height) / grid_size).ceil() * grid_size;
    
    let color = egui::Color32::from_rgba_premultiplied(
        (settings.grid_color[0] * 255.0) as u8,
        (settings.grid_color[1] * 255.0) as u8,
        (settings.grid_color[2] * 255.0) as u8,
        (settings.grid_color[3] * 255.0) as u8,
    );
    
    // Draw vertical lines
    let mut x = min_x;
    while x <= max_x {
        let screen_pos = scene_camera.world_to_screen(glam::Vec3::new(x, 0.0, 0.0));
        let screen_x = center.x + screen_pos.x;
        
        painter.line_segment(
            [
                egui::pos2(screen_x, rect.min.y),
                egui::pos2(screen_x, rect.max.y),
            ],
            egui::Stroke::new(1.0, color),
        );
        
        x += grid_size;
    }
    
    // Draw horizontal lines
    let mut y = min_y;
    while y <= max_y {
        let screen_pos = scene_camera.world_to_screen(glam::Vec3::new(0.0, y, 0.0));
        let screen_y = center.y + screen_pos.y;
        
        painter.line_segment(
            [
                egui::pos2(rect.min.x, screen_y),
                egui::pos2(rect.max.x, screen_y),
            ],
            egui::Stroke::new(1.0, color),
        );
        
        y += grid_size;
    }
    
    // Draw origin (thicker lines)
    let origin_screen = scene_camera.world_to_screen(glam::Vec3::ZERO);
    let origin_x = center.x + origin_screen.x;
    let origin_y = center.y + origin_screen.y;
    
    // X axis (red)
    painter.line_segment(
        [
            egui::pos2(origin_x, rect.min.y),
            egui::pos2(origin_x, rect.max.y),
        ],
        egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(255, 0, 0, 100)),
    );
    
    // Y axis (green)
    painter.line_segment(
        [
            egui::pos2(rect.min.x, origin_y),
            egui::pos2(rect.max.x, origin_y),
        ],
        egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(0, 255, 0, 100)),
    );
}

/// Render snap indicator at position
pub fn render_snap_indicator(
    painter: &egui::Painter,
    world_pos: glam::Vec2,
    scene_camera: &crate::SceneCamera,
    center: egui::Pos2,
    settings: &SnapSettings,
) {
    if !settings.enabled {
        return;
    }
    
    let screen_pos = scene_camera.world_to_screen(glam::Vec3::new(world_pos.x, world_pos.y, 0.0));
    let screen_x = center.x + screen_pos.x;
    let screen_y = center.y + screen_pos.y;
    
    let color = egui::Color32::from_rgba_premultiplied(
        (settings.snap_indicator_color[0] * 255.0) as u8,
        (settings.snap_indicator_color[1] * 255.0) as u8,
        (settings.snap_indicator_color[2] * 255.0) as u8,
        (settings.snap_indicator_color[3] * 255.0) as u8,
    );
    
    // Draw crosshair
    let size = 8.0;
    painter.line_segment(
        [
            egui::pos2(screen_x - size, screen_y),
            egui::pos2(screen_x + size, screen_y),
        ],
        egui::Stroke::new(2.0, color),
    );
    painter.line_segment(
        [
            egui::pos2(screen_x, screen_y - size),
            egui::pos2(screen_x, screen_y + size),
        ],
        egui::Stroke::new(2.0, color),
    );
    
    // Draw circle
    painter.circle_stroke(
        egui::pos2(screen_x, screen_y),
        size * 1.5,
        egui::Stroke::new(1.5, color),
    );
}

// ============================================================================
// SETTINGS UI
// ============================================================================

/// Render snap settings UI
pub fn render_snap_settings_ui(ui: &mut egui::Ui, settings: &mut SnapSettings) -> bool {
    let mut changed = false;
    
    ui.heading("Snap Settings");
    
    // Enable/Disable
    if ui.checkbox(&mut settings.enabled, "Enable Snapping").changed() {
        changed = true;
    }
    
    ui.add_space(8.0);
    
    // Mode
    ui.label("Mode:");
    ui.horizontal(|ui| {
        if ui.selectable_label(settings.mode == SnapMode::Absolute, "Absolute").clicked() {
            settings.mode = SnapMode::Absolute;
            changed = true;
        }
        if ui.selectable_label(settings.mode == SnapMode::Relative, "Relative").clicked() {
            settings.mode = SnapMode::Relative;
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // Grid sizes
    ui.label("Grid Sizes:");
    
    ui.horizontal(|ui| {
        ui.label("Position:");
        if ui.add(egui::DragValue::new(&mut settings.position_snap).speed(0.1).clamp_range(0.01..=100.0)).changed() {
            changed = true;
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("Rotation:");
        if ui.add(egui::DragValue::new(&mut settings.rotation_snap).speed(1.0).clamp_range(1.0..=180.0).suffix("°")).changed() {
            changed = true;
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("Scale:");
        if ui.add(egui::DragValue::new(&mut settings.scale_snap).speed(0.01).clamp_range(0.01..=1.0)).changed() {
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // Presets
    ui.label("Presets:");
    ui.horizontal(|ui| {
        if ui.button("Fine").clicked() {
            *settings = SnapSettings::preset_fine();
            settings.enabled = true;
            changed = true;
        }
        if ui.button("Normal").clicked() {
            *settings = SnapSettings::preset_normal();
            settings.enabled = true;
            changed = true;
        }
        if ui.button("LDtk").clicked() {
            *settings = SnapSettings::preset_ldtk();
            settings.enabled = true;
            changed = true;
        }
        if ui.button("Coarse").clicked() {
            *settings = SnapSettings::preset_coarse();
            settings.enabled = true;
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // Visual
    if ui.checkbox(&mut settings.show_grid, "Show Grid").changed() {
        changed = true;
    }
    
    ui.add_space(8.0);
    
    // Behavior
    ui.label("Snap On:");
    if ui.checkbox(&mut settings.snap_on_create, "Create").changed() {
        changed = true;
    }
    if ui.checkbox(&mut settings.snap_on_move, "Move").changed() {
        changed = true;
    }
    if ui.checkbox(&mut settings.snap_on_rotate, "Rotate").changed() {
        changed = true;
    }
    if ui.checkbox(&mut settings.snap_on_scale, "Scale").changed() {
        changed = true;
    }
    
    ui.add_space(8.0);
    
    // Save/Load
    ui.horizontal(|ui| {
        if ui.button("Save Settings").clicked() {
            if let Err(e) = settings.save() {
                eprintln!("Failed to save snap settings: {}", e);
            }
        }
        if ui.button("Load Settings").clicked() {
            if let Ok(loaded) = SnapSettings::load() {
                *settings = loaded;
                changed = true;
            }
        }
    });
    
    changed
}

// ============================================================================
// KEYBOARD SHORTCUTS
// ============================================================================

/// Handle snap keyboard shortcuts
pub fn handle_snap_shortcuts(ctx: &egui::Context, settings: &mut SnapSettings) -> bool {
    ctx.input(|i| {
        // Ctrl+G: Toggle snapping
        if i.modifiers.ctrl && i.key_pressed(egui::Key::G) {
            settings.enabled = !settings.enabled;
            return true;
        }
        
        // Ctrl+Shift+G: Toggle grid visibility
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::G) {
            settings.show_grid = !settings.show_grid;
            return true;
        }
        
        false
    })
}

/// Check if snapping should be temporarily disabled
pub fn is_snap_override(modifiers: &egui::Modifiers) -> bool {
    // Hold Shift to temporarily disable snapping
    modifiers.shift
}

/// Check if snapping should be temporarily enabled
pub fn is_snap_force(modifiers: &egui::Modifiers) -> bool {
    // Hold Ctrl to temporarily enable snapping
    modifiers.ctrl && !modifiers.shift
}

/// Get effective snap state considering overrides
pub fn get_effective_snap_enabled(settings: &SnapSettings, modifiers: &egui::Modifiers) -> bool {
    if is_snap_override(modifiers) {
        false
    } else if is_snap_force(modifiers) {
        true
    } else {
        settings.enabled
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get nearest grid point
pub fn get_nearest_grid_point(position: glam::Vec2, grid_size: f32) -> glam::Vec2 {
    if grid_size <= 0.0 {
        return position;
    }
    
    glam::Vec2::new(
        (position.x / grid_size).round() * grid_size,
        (position.y / grid_size).round() * grid_size,
    )
}

/// Get grid lines near position
pub fn get_grid_lines_near(position: glam::Vec2, grid_size: f32, range: f32) -> Vec<(glam::Vec2, glam::Vec2)> {
    if grid_size <= 0.0 {
        return Vec::new();
    }
    
    let mut lines = Vec::new();
    
    // Vertical lines
    let x_start = ((position.x - range) / grid_size).floor() * grid_size;
    let x_end = ((position.x + range) / grid_size).ceil() * grid_size;
    
    let mut x = x_start;
    while x <= x_end {
        lines.push((
            glam::Vec2::new(x, position.y - range),
            glam::Vec2::new(x, position.y + range),
        ));
        x += grid_size;
    }
    
    // Horizontal lines
    let y_start = ((position.y - range) / grid_size).floor() * grid_size;
    let y_end = ((position.y + range) / grid_size).ceil() * grid_size;
    
    let mut y = y_start;
    while y <= y_end {
        lines.push((
            glam::Vec2::new(position.x - range, y),
            glam::Vec2::new(position.x + range, y),
        ));
        y += grid_size;
    }
    
    lines
}

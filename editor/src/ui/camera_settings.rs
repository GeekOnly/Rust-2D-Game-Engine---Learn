//! Camera Settings UI Panel
//!
//! Provides UI for adjusting camera settings including:
//! - Zoom sensitivity
//! - Pan sensitivity
//! - Rotation sensitivity
//! - Zoom mode (to cursor / to center)
//! - Camera state display (distance, angles, grid size, FPS)

use egui;
use crate::SceneCamera;
use crate::grid::InfiniteGrid;

/// Camera state display configuration
#[derive(Debug, Clone)]
pub struct CameraStateDisplay {
    pub show_distance: bool,
    pub show_angles: bool,
    pub show_grid_size: bool,
    pub show_fps: bool,
}

impl Default for CameraStateDisplay {
    fn default() -> Self {
        Self {
            show_distance: true,
            show_angles: true,
            show_grid_size: true,
            show_fps: true,
        }
    }
}

impl CameraStateDisplay {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Render camera state display overlay
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        camera: &SceneCamera,
        grid: Option<&InfiniteGrid>,
        fps: f32,
    ) {
        ui.vertical(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            
            if self.show_distance {
                let distance = Self::calculate_distance_from_origin(camera);
                ui.label(format!("Distance: {:.2}", distance));
            }
            
            if self.show_angles {
                ui.label(format!("Yaw: {:.1}°", camera.rotation));
                ui.label(format!("Pitch: {:.1}°", camera.pitch));
            }
            
            if self.show_grid_size {
                if let Some(grid) = grid {
                    let grid_size = grid.calculate_grid_level(camera.zoom);
                    ui.label(format!("Grid: {:.2}", grid_size));
                }
            }
            
            if self.show_fps {
                ui.label(format!("FPS: {:.0}", fps));
            }
        });
    }
    
    /// Calculate camera distance from origin
    pub fn calculate_distance_from_origin(camera: &SceneCamera) -> f32 {
        camera.position.length()
    }
    
    /// Format angle for display (normalized to 0-360 range)
    pub fn format_angle(angle: f32) -> String {
        let normalized = angle.rem_euclid(360.0);
        format!("{:.1}°", normalized)
    }
    
    /// Format grid size for display
    pub fn format_grid_size(size: f32) -> String {
        if size >= 1.0 {
            format!("{:.1}", size)
        } else if size >= 0.01 {
            format!("{:.2}", size)
        } else {
            format!("{:.3}", size)
        }
    }
}

/// Render camera settings panel
pub fn render_camera_settings(
    ui: &mut egui::Ui,
    scene_camera: &mut SceneCamera,
) -> bool {
    let mut changed = false;
    
    ui.heading("Camera Settings");
    ui.separator();
    
    // ========================================================================
    // ZOOM SETTINGS
    // ========================================================================
    
    ui.label("Zoom Settings");
    ui.add_space(4.0);
    
    // Zoom Sensitivity Slider
    ui.horizontal(|ui| {
        ui.label("Zoom Speed:");
        let mut sensitivity = scene_camera.get_zoom_sensitivity();
        if ui.add(
            egui::Slider::new(&mut sensitivity, 0.01..=0.5)
                .text("sensitivity")
                .step_by(0.01)
        ).changed() {
            scene_camera.set_zoom_sensitivity(sensitivity);
            changed = true;
        }
    });
    
    // Zoom Speed Slider
    ui.horizontal(|ui| {
        ui.label("Zoom Smoothness:");
        let mut speed = scene_camera.settings.zoom_speed;
        if ui.add(
            egui::Slider::new(&mut speed, 1.0..=50.0)
                .text("speed")
                .step_by(1.0)
        ).changed() {
            scene_camera.set_zoom_speed(speed);
            changed = true;
        }
    });
    
    // Zoom Mode Toggle (Radio Buttons)
    ui.label("Zoom Mode:");
    ui.horizontal(|ui| {
        if ui.radio_value(&mut scene_camera.settings.zoom_to_cursor, false, "🎯 Zoom to Center")
            .on_hover_text("Zoom in/out from viewport center (Unity 2D mode)")
            .changed() 
        {
            changed = true;
        }
        if ui.radio_value(&mut scene_camera.settings.zoom_to_cursor, true, "🖱 Zoom to Cursor")
            .on_hover_text("Zoom in/out towards mouse cursor (Unity 3D mode)")
            .changed() 
        {
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // PAN SETTINGS
    // ========================================================================
    
    ui.label("Pan Settings");
    ui.add_space(4.0);
    
    // Pan Sensitivity Slider
    ui.horizontal(|ui| {
        ui.label("Pan Speed:");
        let mut sensitivity = scene_camera.settings.pan_sensitivity;
        if ui.add(
            egui::Slider::new(&mut sensitivity, 0.1..=5.0)
                .text("sensitivity")
                .step_by(0.1)
        ).changed() {
            scene_camera.settings.pan_sensitivity = sensitivity;
            changed = true;
        }
    });
    
    // Pan Damping Slider
    ui.horizontal(|ui| {
        ui.label("Pan Smoothness:");
        let mut damping = scene_camera.settings.pan_damping;
        if ui.add(
            egui::Slider::new(&mut damping, 0.0..=0.5)
                .text("damping")
                .step_by(0.01)
        ).changed() {
            scene_camera.settings.pan_damping = damping;
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // ROTATION SETTINGS (3D Mode)
    // ========================================================================
    
    ui.label("Rotation Settings (3D)");
    ui.add_space(4.0);
    
    // Rotation Sensitivity Slider
    ui.horizontal(|ui| {
        ui.label("Rotation Speed:");
        let mut sensitivity = scene_camera.settings.rotation_sensitivity;
        if ui.add(
            egui::Slider::new(&mut sensitivity, 0.1..=2.0)
                .text("sensitivity")
                .step_by(0.1)
        ).changed() {
            scene_camera.settings.rotation_sensitivity = sensitivity;
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // PRESETS
    // ========================================================================
    
    ui.label("Presets");
    ui.add_space(4.0);
    
    ui.horizontal(|ui| {
        if ui.button("Slow").clicked() {
            apply_preset_slow(scene_camera);
            changed = true;
        }
        if ui.button("Normal").clicked() {
            apply_preset_normal(scene_camera);
            changed = true;
        }
        if ui.button("Fast").clicked() {
            apply_preset_fast(scene_camera);
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // ADVANCED SETTINGS
    // ========================================================================
    
    ui.collapsing("Advanced", |ui| {
        // Zoom Damping
        ui.horizontal(|ui| {
            ui.label("Zoom Damping:");
            let mut damping = scene_camera.settings.zoom_damping;
            if ui.add(
                egui::Slider::new(&mut damping, 0.0..=0.5)
                    .step_by(0.01)
            ).changed() {
                scene_camera.settings.zoom_damping = damping;
                changed = true;
            }
        });
        
        // Rotation Damping
        ui.horizontal(|ui| {
            ui.label("Rotation Damping:");
            let mut damping = scene_camera.settings.rotation_damping;
            if ui.add(
                egui::Slider::new(&mut damping, 0.0..=0.5)
                    .step_by(0.01)
            ).changed() {
                scene_camera.settings.rotation_damping = damping;
                changed = true;
            }
        });
        
        // Inertia
        if ui.checkbox(&mut scene_camera.settings.enable_inertia, "Enable Inertia").changed() {
            changed = true;
        }
        
        if scene_camera.settings.enable_inertia {
            ui.horizontal(|ui| {
                ui.label("Inertia Decay:");
                let mut decay = scene_camera.settings.inertia_decay;
                if ui.add(
                    egui::Slider::new(&mut decay, 0.8..=0.99)
                        .step_by(0.01)
                ).changed() {
                    scene_camera.settings.inertia_decay = decay;
                    changed = true;
                }
            });
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // QUICK ACTIONS
    // ========================================================================
    
    ui.label("Quick Actions");
    ui.add_space(4.0);
    
    ui.horizontal(|ui| {
        if ui.button("🏠 Reset View")
            .on_hover_text("Reset camera to default position and zoom (Home key)")
            .clicked() 
        {
            scene_camera.reset();
            changed = true;
        }
        
        if ui.button("📐 Frame All")
            .on_hover_text("Frame all objects in view (Shift+F)")
            .clicked() 
        {
            // This will be handled by the caller with world data
            // For now, just reset to a reasonable default view
            scene_camera.set_zoom_level(50.0);
            scene_camera.position = glam::Vec3::ZERO;
            changed = true;
        }
    });
    
    ui.add_space(8.0);
    
    // ========================================================================
    // SAVE/LOAD
    // ========================================================================
    
    ui.horizontal(|ui| {
        if ui.button("💾 Save Settings").clicked() {
            if let Err(e) = scene_camera.save_settings() {
                eprintln!("Failed to save camera settings: {}", e);
            }
        }
        
        if ui.button("📂 Load Settings").clicked() {
            if let Err(e) = scene_camera.load_settings() {
                eprintln!("Failed to load camera settings: {}", e);
            }
            changed = true;
        }
        
        if ui.button("🔄 Reset to Default").clicked() {
            scene_camera.reset_settings_to_default();
            changed = true;
        }
    });
    
    changed
}

/// Render compact camera settings (for toolbar)
pub fn render_camera_settings_compact(
    ui: &mut egui::Ui,
    scene_camera: &mut SceneCamera,
) -> bool {
    let mut changed = false;
    
    ui.horizontal(|ui| {
        ui.label("🔍 Zoom:");
        let mut sensitivity = scene_camera.get_zoom_sensitivity();
        if ui.add(
            egui::Slider::new(&mut sensitivity, 0.01..=0.5)
                .step_by(0.01)
        ).changed() {
            scene_camera.set_zoom_sensitivity(sensitivity);
            changed = true;
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("🖱 Pan:");
        if ui.add(
            egui::Slider::new(&mut scene_camera.settings.pan_sensitivity, 0.1..=5.0)
                .step_by(0.1)
        ).changed() {
            changed = true;
        }
    });
    
    changed
}

/// Render minimal camera controls for scene view overlay
pub fn render_scene_view_controls(
    ui: &mut egui::Ui,
    scene_camera: &mut SceneCamera,
) -> bool {
    let mut changed = false;
    
    ui.horizontal(|ui| {
        ui.label("🔍");
        let mut zoom_sens = scene_camera.get_zoom_sensitivity();
        if ui.add(
            egui::Slider::new(&mut zoom_sens, 0.01..=0.5)
                .show_value(false)
        ).on_hover_text(format!("Zoom Speed: {:.2}", zoom_sens))
        .changed() {
            scene_camera.set_zoom_sensitivity(zoom_sens);
            changed = true;
        }
        
        ui.separator();
        
        ui.label("🖱");
        if ui.add(
            egui::Slider::new(&mut scene_camera.settings.pan_sensitivity, 0.1..=5.0)
                .show_value(false)
        ).on_hover_text(format!("Pan Speed: {:.1}", scene_camera.settings.pan_sensitivity))
        .changed() {
            changed = true;
        }
        
        ui.separator();
        
        // Quick preset buttons
        if ui.small_button("S").on_hover_text("Slow preset").clicked() {
            apply_preset_slow(scene_camera);
            changed = true;
        }
        if ui.small_button("N").on_hover_text("Normal preset").clicked() {
            apply_preset_normal(scene_camera);
            changed = true;
        }
        if ui.small_button("F").on_hover_text("Fast preset").clicked() {
            apply_preset_fast(scene_camera);
            changed = true;
        }
    });
    
    changed
}

// ============================================================================
// PRESETS
// ============================================================================

fn apply_preset_slow(camera: &mut SceneCamera) {
    camera.settings.zoom_sensitivity = 0.08;
    camera.settings.pan_sensitivity = 0.5;
    camera.settings.rotation_sensitivity = 0.3;
    camera.settings.zoom_speed = 10.0;
    camera.settings.pan_damping = 0.12;
    camera.settings.zoom_damping = 0.12;
}

fn apply_preset_normal(camera: &mut SceneCamera) {
    camera.settings.zoom_sensitivity = 0.12;
    camera.settings.pan_sensitivity = 1.0;
    camera.settings.rotation_sensitivity = 0.5;
    camera.settings.zoom_speed = 20.0;
    camera.settings.pan_damping = 0.08;
    camera.settings.zoom_damping = 0.08;
}

fn apply_preset_fast(camera: &mut SceneCamera) {
    camera.settings.zoom_sensitivity = 0.18;
    camera.settings.pan_sensitivity = 2.0;
    camera.settings.rotation_sensitivity = 0.8;
    camera.settings.zoom_speed = 35.0;
    camera.settings.pan_damping = 0.05;
    camera.settings.zoom_damping = 0.05;
}

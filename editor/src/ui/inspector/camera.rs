use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;

pub fn render_camera_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Camera Component (Unity-style)
    let has_camera = world.has_component(entity, ComponentType::Camera);
    let mut remove_camera = false;

    if has_camera {
        let camera_id = ui.make_persistent_id("camera_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), camera_id, true
        );

        render_component_header(ui, "Camera", "📷", false);

        if is_open.is_open() {
            if let Some(camera) = world.cameras.get_mut(&entity) {
                ui.indent("camera_indent", |ui| {
                    egui::Grid::new("camera_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Projection");
                            egui::ComboBox::from_id_source("projection_picker")
                                .selected_text(match camera.projection {
                                    ecs::CameraProjection::Orthographic => "Orthographic",
                                    ecs::CameraProjection::Perspective => "Perspective",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut camera.projection, ecs::CameraProjection::Orthographic, "Orthographic");
                                    ui.selectable_value(&mut camera.projection, ecs::CameraProjection::Perspective, "Perspective");
                                });
                            ui.end_row();

                            // Show different fields based on projection mode
                            match camera.projection {
                                ecs::CameraProjection::Orthographic => {
                                    ui.label("Size");
                                    ui.add(egui::DragValue::new(&mut camera.orthographic_size).speed(0.1).clamp_range(0.1..=1000.0));
                                    ui.end_row();
                                }
                                ecs::CameraProjection::Perspective => {
                                    ui.label("FOV");
                                    ui.add(egui::Slider::new(&mut camera.fov, 1.0..=179.0).suffix("°"));
                                    ui.end_row();
                                }
                            }

                            ui.label("Depth");
                            ui.add(egui::DragValue::new(&mut camera.depth).speed(1.0))
                                .on_hover_text("Camera rendering order (lower renders first)");
                            ui.end_row();

                            ui.label("Background");
                            ui.color_edit_button_rgba_unmultiplied(&mut camera.background_color);
                            ui.end_row();

                            ui.label("Near Clip");
                            ui.add(egui::DragValue::new(&mut camera.near_clip).speed(0.1).clamp_range(0.01..=1000.0));
                            ui.end_row();

                            ui.label("Far Clip");
                            ui.add(egui::DragValue::new(&mut camera.far_clip).speed(1.0).clamp_range(1.0..=10000.0));
                            ui.end_row();
                            
                            ui.label("Pixels Per Unit");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut camera.pixels_per_unit).speed(0.1).clamp_range(0.1..=1000.0))
                                    .on_hover_text("How many pixels = 1 world unit (1 meter)\n100.0 = Unity standard (recommended for 2.5D/3D)\n10.0 = pixel art games");
                                if ui.small_button("Unity").on_hover_text("Unity standard (100 pixels = 1 world unit) - Recommended").clicked() {
                                    camera.pixels_per_unit = 100.0;
                                }
                                if ui.small_button("Pixel").on_hover_text("Pixel art (10 pixels = 1 world unit)").clicked() {
                                    camera.pixels_per_unit = 10.0;
                                }
                            });
                            ui.end_row();
                            
                            // Aspect Ratio Presets
                            ui.label("Aspect Ratio");
                            ui.horizontal(|ui| {
                                // Calculate current aspect ratio
                                let current_aspect = camera.viewport_rect[2] / camera.viewport_rect[3];
                                
                                // Preset buttons
                                if ui.small_button("16:9").on_hover_text("Set to 16:9 (1.778)").clicked() {
                                    camera.viewport_rect[2] = 1.0;
                                    camera.viewport_rect[3] = 9.0 / 16.0;
                                }
                                if ui.small_button("16:10").on_hover_text("Set to 16:10 (1.6)").clicked() {
                                    camera.viewport_rect[2] = 1.0;
                                    camera.viewport_rect[3] = 10.0 / 16.0;
                                }
                                if ui.small_button("4:3").on_hover_text("Set to 4:3 (1.333)").clicked() {
                                    camera.viewport_rect[2] = 1.0;
                                    camera.viewport_rect[3] = 3.0 / 4.0;
                                }
                                if ui.small_button("1:1").on_hover_text("Set to 1:1 (Square)").clicked() {
                                    camera.viewport_rect[2] = 1.0;
                                    camera.viewport_rect[3] = 1.0;
                                }
                                
                                ui.label(egui::RichText::new(format!("{:.2}:1", current_aspect)).small().color(egui::Color32::GRAY));
                            });
                            ui.end_row();
                        });

                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }
                        if ui.button("❌ Remove Component").clicked() {
                            remove_camera = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }

    if remove_camera {
        let _ = world.remove_component(entity, ComponentType::Camera);
    }
}

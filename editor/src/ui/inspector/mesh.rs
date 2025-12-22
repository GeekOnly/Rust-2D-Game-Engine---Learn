use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;
use std::path::Path;

pub fn render_mesh_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity, project_path: Option<&Path>, reload_mesh_assets_request: &mut bool) {
    // Mesh Component (3D) - Unity-style
    let has_mesh = world.has_component(entity, ComponentType::Mesh);
    let mut remove_mesh = false;
    
    if has_mesh {
        let mesh_id = ui.make_persistent_id("mesh_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), mesh_id, true
        );
        
        render_component_header(ui, "Mesh Renderer", "🧊", false);
        
        if is_open.is_open() {
            if let Some(mesh) = world.meshes.get_mut(&entity) {
                ui.indent("mesh_indent", |ui| {
                    egui::Grid::new("mesh_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Mesh Type");

                            // Get current mesh type display
                            let current_text = match &mesh.mesh_type {
                                ecs::MeshType::Cube => "Cube".to_string(),
                                ecs::MeshType::Sphere => "Sphere".to_string(),
                                ecs::MeshType::Cylinder => "Cylinder".to_string(),
                                ecs::MeshType::Plane => "Plane".to_string(),
                                ecs::MeshType::Capsule => "Capsule".to_string(),
                                ecs::MeshType::Asset(path) => format!("Asset: {}", path),
                            };

                            // Track the previous mesh type
                            let prev_is_asset = matches!(&mesh.mesh_type, ecs::MeshType::Asset(_));

                            let combo_response = egui::ComboBox::from_id_source("mesh_type_picker")
                                .selected_text(current_text)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cube, "Cube");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Sphere, "Sphere");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cylinder, "Cylinder");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Plane, "Plane");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Capsule, "Capsule");

                                    // Separator
                                    ui.separator();

                                    // Asset option - create a placeholder if selected
                                    if ui.selectable_label(
                                        matches!(mesh.mesh_type, ecs::MeshType::Asset(_)),
                                        "Asset (GLTF/GLB)"
                                    ).clicked() {
                                        mesh.mesh_type = ecs::MeshType::Asset("assets/model.glb".to_string());
                                        *reload_mesh_assets_request = true;
                                    }
                                });

                            // Check if mesh type changed to non-Asset type (also needs reload to clear old asset)
                            if combo_response.response.changed() {
                                let now_is_asset = matches!(&mesh.mesh_type, ecs::MeshType::Asset(_));
                                if prev_is_asset != now_is_asset {
                                    *reload_mesh_assets_request = true;
                                }
                            }

                            ui.end_row();

                            // If Asset type, show text field to edit path
                            if let ecs::MeshType::Asset(path) = &mut mesh.mesh_type {
                                ui.label("Asset Path");
                                ui.horizontal(|ui| {
                                    if ui.text_edit_singleline(path).changed() {
                                        *reload_mesh_assets_request = true;
                                    }
                                    if ui.button("📁").on_hover_text("Browse...").clicked() {
                                        // Set default directory to project's assets folder if available
                                        let mut dialog = rfd::FileDialog::new()
                                            .add_filter("GLTF/GLB", &["gltf", "glb"]);

                                        if let Some(proj_path) = project_path {
                                            let assets_path = proj_path.join("assets");
                                            if assets_path.exists() {
                                                dialog = dialog.set_directory(&assets_path);
                                            }
                                        }

                                        if let Some(file_path) = dialog.pick_file() {
                                            // Try to make path relative to project folder
                                            if let Some(proj_path) = project_path {
                                                if let Ok(relative) = file_path.strip_prefix(proj_path) {
                                                    *path = relative.to_string_lossy().to_string();
                                                } else {
                                                    *path = file_path.to_string_lossy().to_string();
                                                }
                                            } else {
                                                *path = file_path.to_string_lossy().to_string();
                                            }
                                            // Request reload when path changes
                                            *reload_mesh_assets_request = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                ui.label("");
                                ui.colored_label(
                                    egui::Color32::from_rgb(150, 150, 150),
                                    "💡 Path relative to project folder"
                                );
                                ui.end_row();
                            }
                            
                            ui.label("Color");
                            let mut color = egui::Color32::from_rgba_unmultiplied(
                                (mesh.color[0] * 255.0) as u8,
                                (mesh.color[1] * 255.0) as u8,
                                (mesh.color[2] * 255.0) as u8,
                                (mesh.color[3] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                mesh.color[0] = color.r() as f32 / 255.0;
                                mesh.color[1] = color.g() as f32 / 255.0;
                                mesh.color[2] = color.b() as f32 / 255.0;
                                mesh.color[3] = color.a() as f32 / 255.0;
                            }
                            ui.end_row();
                        });
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }
                        if ui.button("❌ Remove Component").clicked() {
                            remove_mesh = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_mesh {
        let _ = world.remove_component(entity, ComponentType::Mesh);
    }
}

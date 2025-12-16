use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;

pub fn render_mesh_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
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
                            egui::ComboBox::from_id_source("mesh_type_picker")
                                .selected_text(format!("{:?}", mesh.mesh_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cube, "Cube");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Sphere, "Sphere");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cylinder, "Cylinder");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Plane, "Plane");
                                    ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Capsule, "Capsule");
                                });
                            ui.end_row();
                            
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

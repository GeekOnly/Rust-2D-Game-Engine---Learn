use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;

pub fn render_collider_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Collider Component (Unity-style)
    let has_collider = world.has_component(entity, ComponentType::BoxCollider);
    let mut remove_collider = false;
    
    if has_collider {
        let collider_id = ui.make_persistent_id("collider_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), collider_id, true
        );
        
        render_component_header(ui, "Box Collider 2D", "📦", false);
        
        if is_open.is_open() {
            // Migrate legacy colliders
            if let Some(transform) = world.transforms.get(&entity) {
                if let Some(collider) = world.colliders.get_mut(&entity) {
                    collider.migrate_from_legacy(transform.scale);
                }
            }
            
            if let Some(collider) = world.colliders.get_mut(&entity) {
                ui.indent("collider_indent", |ui| {
                    egui::Grid::new("collider_grid")
                        .num_columns(5)
                        .spacing([5.0, 8.0])
                        .show(ui, |ui| {
                            // Edit Collider button
                            ui.label("Edit Collider");
                            if ui.button("🔧").on_hover_text("Edit collider shape").clicked() {
                                // TODO: Open collider editor
                            }
                            ui.end_row();
                            
                            // Offset
                            ui.label("Offset");
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut collider.offset[0]).speed(0.01).max_decimals(2));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut collider.offset[1]).speed(0.01).max_decimals(2));
                            ui.end_row();
                            
                            // Size
                            ui.label("Size");
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut collider.size[0]).speed(0.01).max_decimals(2).clamp_range(0.01..=100.0));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut collider.size[1]).speed(0.01).max_decimals(2).clamp_range(0.01..=100.0));
                            ui.end_row();
                        });
                    
                    ui.add_space(5.0);
                    
                    // Show actual world size
                    if let Some(transform) = world.transforms.get(&entity) {
                        let world_width = collider.get_world_width(transform.scale[0]);
                        let world_height = collider.get_world_height(transform.scale[1]);
                        ui.label(egui::RichText::new(format!(
                            "💡 World size: {:.2} x {:.2} (Size × Transform.scale)",
                            world_width, world_height
                        )).small().color(egui::Color32::from_rgb(150, 150, 150)));
                    }
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }
                        if ui.button("❌ Remove Component").clicked() {
                            remove_collider = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_collider {
        let _ = world.remove_component(entity, ComponentType::BoxCollider);
    }
}

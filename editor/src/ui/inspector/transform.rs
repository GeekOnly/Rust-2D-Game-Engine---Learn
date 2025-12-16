use ecs::{World, Entity};
use egui;
use arboard::Clipboard;
use super::utils::render_component_header;

pub fn render_transform_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    if let Some(transform) = world.transforms.get_mut(&entity) {
        render_component_header(ui, "Transform", "⚙️", true);

        // Transform fields - Unity style (X Y Z in same row)
        egui::Grid::new("transform_grid")
            .num_columns(7)
            .spacing([5.0, 8.0])
            .show(ui, |ui| {
                // Position row
                ui.label("Position");
                ui.label("X");
                ui.add(
                    egui::DragValue::new(&mut transform.position[0])
                        .speed(0.1)
                        .max_decimals(2)
                );
                ui.label("Y");
                ui.add(
                    egui::DragValue::new(&mut transform.position[1])
                        .speed(0.1)
                        .max_decimals(2)
                );
                ui.label("Z");
                ui.add(
                    egui::DragValue::new(&mut transform.position[2])
                        .speed(0.1)
                        .max_decimals(2)
                );
                ui.end_row();

                // Rotation row
                ui.label("Rotation");
                ui.label("X");
                ui.add(
                    egui::DragValue::new(&mut transform.rotation[0])
                        .speed(0.5)
                        .max_decimals(2)
                );
                ui.label("Y");
                ui.add(
                    egui::DragValue::new(&mut transform.rotation[1])
                        .speed(0.5)
                        .max_decimals(2)
                );
                ui.label("Z");
                ui.add(
                    egui::DragValue::new(&mut transform.rotation[2])
                        .speed(0.5)
                        .max_decimals(2)
                );
                ui.end_row();

                // Scale row
                ui.label("Scale");
                ui.label("X");
                ui.add(
                    egui::DragValue::new(&mut transform.scale[0])
                        .speed(0.01)
                        .max_decimals(2)
                );
                ui.label("Y");
                ui.add(
                    egui::DragValue::new(&mut transform.scale[1])
                        .speed(0.01)
                        .max_decimals(2)
                );
                ui.label("Z");
                ui.add(
                    egui::DragValue::new(&mut transform.scale[2])
                        .speed(0.01)
                        .max_decimals(2)
                );
                ui.end_row();
            });

        ui.add_space(5.0);
        
        // Copy buttons for Transform
        ui.horizontal(|ui| {
            if ui.small_button("📋 Pos").on_hover_text("Copy position").clicked() {
                let text = format!("{:.2}, {:.2}, {:.2}", 
                    transform.position[0], transform.position[1], transform.position[2]);
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
            }
            if ui.small_button("📋 Rot").on_hover_text("Copy rotation").clicked() {
                let text = format!("{:.2}, {:.2}, {:.2}", 
                    transform.rotation[0], transform.rotation[1], transform.rotation[2]);
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
            }
            if ui.small_button("📋 Scale").on_hover_text("Copy scale").clicked() {
                let text = format!("{:.2}, {:.2}, {:.2}", 
                    transform.scale[0], transform.scale[1], transform.scale[2]);
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
            }
        });

        ui.add_space(10.0);
    }
}

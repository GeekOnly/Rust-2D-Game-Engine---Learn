use ecs::{World, Entity, ComponentType, ComponentManager, ColliderShape3D};
use egui;

pub fn render_collider_3d_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // 3D Collider Component
    let has_collider = world.has_component(entity, ComponentType::Collider3D);
    let mut remove_collider = false;
    
    if has_collider {
        // We do custom header handling here to support collapse state correctly if render_component_header doesn't handle it the way we prefer or if we want to mimic the logic.
        // Actually, render_component_header handles the *open* state via button. But we want a CollapsingHeader.
        // The previous code uses `is_open.is_open()` checks.
        
        let collider_id = ui.make_persistent_id("collider_3d_component");
        let mut is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), collider_id, true
        );
        
        // Use a simple label or reimplement header if needed.
        // Let's assume we use the same style as existing inspectors.
        // But `render_component_header` signature in `utils.rs` isn't fully shown.
        // Based on usage: render_component_header(ui, title, icon, is_selected?)
        // Wait, render_component_header in `collider.rs` at line 16 doesn't return IsOpen. 
        // It seems `is_open` variable handles the collapsing state manually?
        
        // Actually, let's just use CollapsingHeader for simplicity if valid.
        // Or follow the pattern:
        // 1. Check state
        // 2. Render header (and update state via toggle button if any)
        
        // Let's stick to the pattern in collider.rs:
        let mut header_open = is_open.is_open();
        
        egui::Frame::none()
        .fill(egui::Color32::from_rgb(60, 60, 60))
        .inner_margin(egui::Margin::same(5))
        .show(ui, |ui| {
             ui.horizontal(|ui| {
                if ui.button(if header_open { "▼" } else { "▶" }).clicked() {
                    header_open = !header_open;
                }
                ui.label(egui::RichText::new("📦 Box Collider 3D").strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                     if ui.button("❌").on_hover_text("Remove Component").clicked() {
                         remove_collider = true;
                     }
                });
             });
        });

        is_open.set_open(header_open);
        
        if header_open {
             if let Some(collider) = world.colliders_3d.get_mut(&entity) {
                 ui.indent("collider_3d_indent", |ui| {
                     egui::Grid::new("collider_3d_grid")
                        .num_columns(5)
                        .spacing([5.0, 8.0])
                        .show(ui, |ui| {
                            
                            // Shape (Only Box for now really supported in gizmo)
                            ui.label("Shape");
                            egui::ComboBox::from_id_source("shape_3d_combo")
                                .selected_text(match collider.shape {
                                    ColliderShape3D::Box => "Box",
                                    ColliderShape3D::Sphere => "Sphere",
                                    ColliderShape3D::Capsule => "Capsule",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut collider.shape, ColliderShape3D::Box, "Box");
                                    ui.selectable_value(&mut collider.shape, ColliderShape3D::Sphere, "Sphere");
                                    ui.selectable_value(&mut collider.shape, ColliderShape3D::Capsule, "Capsule");
                                });
                             ui.end_row();

                             // Offset
                             ui.label("Offset");
                             ui.label("X"); ui.add(egui::DragValue::new(&mut collider.offset[0]).speed(0.01));
                             ui.label("Y"); ui.add(egui::DragValue::new(&mut collider.offset[1]).speed(0.01));
                             ui.label("Z"); ui.add(egui::DragValue::new(&mut collider.offset[2]).speed(0.01));
                             ui.end_row();

                             // Size
                             ui.label("Size");
                             ui.label("X"); ui.add(egui::DragValue::new(&mut collider.size[0]).speed(0.01).clamp_range(0.01..=100.0));
                             ui.label("Y"); ui.add(egui::DragValue::new(&mut collider.size[1]).speed(0.01).clamp_range(0.01..=100.0));
                             ui.label("Z"); ui.add(egui::DragValue::new(&mut collider.size[2]).speed(0.01).clamp_range(0.01..=100.0));
                             ui.end_row();
                        });
                 });
             }
             ui.add_space(10.0);
        }
    }
    
    if remove_collider {
        let _ = world.remove_component(entity, ComponentType::Collider3D);
    }
}

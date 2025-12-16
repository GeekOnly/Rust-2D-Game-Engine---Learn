use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use arboard::Clipboard;
use super::utils::render_component_header;

pub fn render_rigidbody_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Rigidbody 2D Component - Unity-style with full properties
    let has_rigidbody = world.has_component(entity, ComponentType::Rigidbody);
    let mut remove_rigidbody = false;
    
    if has_rigidbody {
        let rigidbody_id = ui.make_persistent_id("rigidbody_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), rigidbody_id, true
        );
        
        render_component_header(ui, "Rigidbody 2D", "⚡", false);
        
        if is_open.is_open() {
            // Ensure rigidbody exists (create if only legacy velocity exists)
            if !world.rigidbodies.contains_key(&entity) {
                let vel = world.velocities.get(&entity).copied().unwrap_or((0.0, 0.0));
                let mut rb = ecs::Rigidbody2D::default();
                rb.velocity = vel;
                world.rigidbodies.insert(entity, rb);
            }

            if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                ui.indent("rigidbody_indent", |ui| {
                    egui::Grid::new("rigidbody_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Velocity X");
                            ui.add(egui::DragValue::new(&mut rigidbody.velocity.0).speed(0.1));
                            ui.end_row();
                            
                            ui.label("Velocity Y");
                            ui.add(egui::DragValue::new(&mut rigidbody.velocity.1).speed(0.1));
                            ui.end_row();

                            ui.label("Gravity Scale");
                            ui.add(egui::DragValue::new(&mut rigidbody.gravity_scale).speed(0.1).clamp_range(0.0..=10.0))
                                .on_hover_text("0 = no gravity, 1 = normal gravity");
                            ui.end_row();

                            ui.label("Mass");
                            ui.add(egui::DragValue::new(&mut rigidbody.mass).speed(0.1).clamp_range(0.1..=100.0))
                                .on_hover_text("Affects collision response");
                            ui.end_row();

                            ui.label("Is Kinematic");
                            ui.checkbox(&mut rigidbody.is_kinematic, "")
                                .on_hover_text("If checked, not affected by physics forces");
                            ui.end_row();

                            ui.label("Freeze Rotation");
                            ui.checkbox(&mut rigidbody.freeze_rotation, "")
                                .on_hover_text("Prevent rotation (for 2D games)");
                            ui.end_row();

                            ui.label("Enable CCD");
                            ui.checkbox(&mut rigidbody.enable_ccd, "")
                                .on_hover_text("Continuous Collision Detection - prevents fast objects from tunneling through colliders");
                            ui.end_row();
                        });

                    // Sync with legacy velocity
                    world.velocities.insert(entity, rigidbody.velocity);
                    
                    ui.add_space(10.0);
                    
                    // Debug info
                    ui.label(egui::RichText::new("Debug Info:").color(egui::Color32::GRAY).small());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!(
                            "Speed: {:.2} px/s", 
                            (rigidbody.velocity.0.powi(2) + rigidbody.velocity.1.powi(2)).sqrt()
                        )).small().color(egui::Color32::GRAY));
                        
                        if ui.small_button("📋").on_hover_text("Copy velocity").clicked() {
                            let text = format!("{:.2}, {:.2}", rigidbody.velocity.0, rigidbody.velocity.1);
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        }
                    });
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }
                        if ui.button("❌ Remove Component").clicked() {
                            remove_rigidbody = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_rigidbody {
        let _ = world.remove_component(entity, ComponentType::Rigidbody);
    }
}

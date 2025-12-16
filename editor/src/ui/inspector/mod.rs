pub mod utils;
pub mod transform;
pub mod sprite;
pub mod collider;
pub mod rigidbody;
pub mod mesh;
pub mod camera;
pub mod script;

use ecs::{World, Entity, EntityTag, ComponentType, ComponentManager};
use egui;
use std::collections::HashMap;
use arboard::Clipboard;

pub use utils::parse_hex_color;

/// Renders the Inspector panel showing entity properties and components
pub fn render_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    entity_names: &mut HashMap<Entity, String>,
    edit_script_request: &mut Option<String>,
    project_path: &Option<std::path::PathBuf>,
    open_sprite_editor_request: &mut Option<std::path::PathBuf>,
    sprite_picker_state: &mut crate::ui::sprite_picker::SpritePickerState,
) {
    // Unity-style header
    ui.horizontal(|ui| {
        ui.heading("Inspector");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Copy button for debugging
            if let Some(entity) = *selected_entity {
                if ui.button("📋 Copy All").on_hover_text("Copy all component values to clipboard").clicked() {
                    let debug_info = utils::format_entity_debug_info(world, entity, entity_names);
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(debug_info);
                    }
                }
            }
            
            // Options menu
            ui.menu_button("⋮", |ui| {
                if ui.button("Reset").clicked() {
                    ui.close_menu();
                }
                if ui.button("Copy Component").clicked() {
                    ui.close_menu();
                }
                if ui.button("Paste Component Values").clicked() {
                    ui.close_menu();
                }
            });
        });
    });
    ui.separator();

    if let Some(entity) = *selected_entity {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // ===== Unity-style Inspector Header (Gray bar) =====
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(60, 60, 60))
                .inner_margin(egui::Margin::same(5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Active checkbox
                        let mut is_active = world.active.get(&entity).copied().unwrap_or(true);
                        if ui.checkbox(&mut is_active, "").changed() {
                            world.active.insert(entity, is_active);
                        }

                        // GameObject icon (cube)
                        ui.label("🎲");

                        // Entity name
                        if let Some(name) = entity_names.get_mut(&entity) {
                            ui.add(egui::TextEdit::singleline(name)
                                .desired_width(120.0)
                                .frame(false));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Static dropdown (Unity has this)
                            egui::ComboBox::from_id_source("static_dropdown")
                                .selected_text("Static")
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    ui.label("Nothing");
                                    ui.label("Everything");
                                });
                        });
                    });
                });

            ui.add_space(5.0);

            // Tag and Layer in same row (Unity layout)
            ui.horizontal(|ui| {
                ui.label("Tag");

                let current_tag = world.tags.get(&entity).cloned();
                let tag_text = match &current_tag {
                    Some(EntityTag::Player) => "Player",
                    Some(EntityTag::Item) => "Item",
                    None => "Untagged",
                };

                let mut new_tag = current_tag.clone();
                let mut tag_changed = false;

                egui::ComboBox::from_id_source("tag_dropdown")
                    .selected_text(tag_text)
                    .width(70.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(current_tag.is_none(), "Untagged").clicked() {
                            new_tag = None;
                            tag_changed = true;
                        }
                        if ui.selectable_label(matches!(current_tag, Some(EntityTag::Player)), "Player").clicked() {
                            new_tag = Some(EntityTag::Player);
                            tag_changed = true;
                        }
                        if ui.selectable_label(matches!(current_tag, Some(EntityTag::Item)), "Item").clicked() {
                            new_tag = Some(EntityTag::Item);
                            tag_changed = true;
                        }
                    });

                if tag_changed {
                    if let Some(tag) = new_tag {
                        world.tags.insert(entity, tag);
                    } else {
                        world.tags.remove(&entity);
                    }
                }

                ui.add_space(10.0);
                ui.label("Layer");

                let current_layer = world.layers.get(&entity).copied().unwrap_or(0);
                let layer_names = ["Default", "TransparentFX", "Ignore Raycast", "Water", "UI"];
                let layer_text = if (current_layer as usize) < layer_names.len() {
                    layer_names[current_layer as usize]
                } else {
                    "Custom"
                };

                egui::ComboBox::from_id_source("layer_dropdown")
                    .selected_text(layer_text)
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        for (idx, &name) in layer_names.iter().enumerate() {
                            if ui.selectable_label(current_layer == idx as u8, name).clicked() {
                                world.layers.insert(entity, idx as u8);
                            }
                        }
                    });
            });

            ui.add_space(10.0);

            // --- Components ---
            transform::render_transform_inspector(ui, world, entity);
            sprite::render_sprite_inspector(ui, world, entity, sprite_picker_state, open_sprite_editor_request);
            collider::render_collider_inspector(ui, world, entity);
            rigidbody::render_rigidbody_inspector(ui, world, entity);
            mesh::render_mesh_inspector(ui, world, entity);
            camera::render_camera_inspector(ui, world, entity);
            script::render_script_inspector(ui, world, entity, project_path, edit_script_request);

            // ===== Add Component Button (Unity-style) =====
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 70.0);
                ui.menu_button("➕ Add Component", |ui| {
                    // ใช้ Component Manager เพื่อดึงรายการ Component ที่สามารถเพิ่มได้
                    // Using get_addable_components from World (assuming implemented in ecs crate)
                     // If it's not available, we might need to implement logic here.
                     // The legacy code used `world.get_addable_components(entity)`.
                     
                    let addable_components = world.get_addable_components(entity);
                    if addable_components.is_empty() {
                        ui.label("All components added");
                    } else {
                            // Helper to render component buttons
                            let mut render_component_category = |ui: &mut egui::Ui, name: &str, components: &[ComponentType]| {
                                let has_cat = addable_components.iter().any(|c| components.contains(c));
                                if has_cat {
                                    ui.label(name);
                                    ui.separator();
                                    for component_type in components {
                                        if addable_components.contains(component_type) {
                                            if ui.button(component_type.display_name()).clicked() {
                                                let _ = world.add_component(entity, *component_type);
                                                ui.close_menu();
                                            }
                                        }
                                    }
                                    ui.add_space(5.0);
                                }
                            };

                            render_component_category(ui, "🎨 Rendering", &[ComponentType::Sprite, ComponentType::Mesh]);
                            render_component_category(ui, "⚙️ Physics", &[ComponentType::BoxCollider, ComponentType::Rigidbody, ComponentType::TilemapCollider, ComponentType::LdtkIntGridCollider]);
                            render_component_category(ui, "🗺️ Tilemap", &[ComponentType::LdtkMap]);
                            render_component_category(ui, "📜 Other", &[ComponentType::Camera, ComponentType::Script, ComponentType::Tag, ComponentType::Map]);
                    }
                });
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Delete GameObject button
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 80.0);
                if ui.button("🗑 Delete GameObject").clicked() {
                    world.despawn(entity);
                    entity_names.remove(&entity);
                    *selected_entity = None;
                }
            });
        });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("Select an object to inspect").color(egui::Color32::GRAY));
        });
    }
}

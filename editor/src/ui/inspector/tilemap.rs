use ecs::{World, Entity, ComponentManager, Tilemap, TilemapRenderer, Grid, TilemapRenderMode, MaskInteraction};
use egui;
use crate::ui::inspector::utils;

pub fn render_tilemap_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    if !world.has_component(entity, ecs::ComponentType::Tilemap) {
        return;
    }

    let is_removed = utils::render_component(ui, "Tilemap", "🗺", |ui: &mut egui::Ui| {
        let tilemap = match world.tilemaps.get_mut(&entity) {
            Some(t) => t,
            None => return,
        };

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut tilemap.name);
        });

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Tileset ID");
            ui.text_edit_singleline(&mut tilemap.tileset_id);
        });

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Size");
            ui.add(egui::DragValue::new(&mut tilemap.width).prefix("W: "));
            ui.add(egui::DragValue::new(&mut tilemap.height).prefix("H: "));
        });

        ui.collapsing("Visibility & Layout", |ui: &mut egui::Ui| {
             ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Visible");
                ui.checkbox(&mut tilemap.visible, "");
            });
            
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Opacity");
                ui.add(egui::Slider::new(&mut tilemap.opacity, 0.0..=1.0));
            });

            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Z Order");
                ui.add(egui::DragValue::new(&mut tilemap.z_order));
            });
            
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Orientation");
                // Simple string edit for now, could be enum combo
                ui.text_edit_singleline(&mut tilemap.orientation);
            });
        });

        // Debug info
        ui.label(format!("Tile Count: {}", tilemap.tiles.len()));
    });

    if is_removed {
        let _ = world.remove_component(entity, ecs::ComponentType::Tilemap);
    }
}

pub fn render_tilemap_renderer_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    if !world.has_component(entity, ecs::ComponentType::TilemapRenderer) {
        return;
    }

    let is_removed = utils::render_component(ui, "Tilemap Renderer", "🎨", |ui: &mut egui::Ui| {
        let renderer = match world.tilemap_renderers.get_mut(&entity) {
            Some(r) => r,
            None => return,
        };

        // Render Mode
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Render Mode");
            egui::ComboBox::from_id_salt("render_mode")
                .selected_text(match renderer.mode {
                    TilemapRenderMode::Individual => "Individual",
                    TilemapRenderMode::Chunk => "Chunk",
                })
                .show_ui(ui, |ui: &mut egui::Ui| {
                    ui.selectable_value(&mut renderer.mode, TilemapRenderMode::Individual, "Individual");
                    ui.selectable_value(&mut renderer.mode, TilemapRenderMode::Chunk, "Chunk");
                });
        });

        // Sorting
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Sorting Layer");
            ui.text_edit_singleline(&mut renderer.sorting_layer);
        });

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Order in Layer");
            ui.add(egui::DragValue::new(&mut renderer.order_in_layer));
        });

        // Color/Tint
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Tint Color");
            ui.color_edit_button_rgba_unmultiplied(&mut renderer.color);
        });

        // Chunk Settings
        if renderer.mode == TilemapRenderMode::Chunk {
             ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Chunk Size");
                ui.add(egui::DragValue::new(&mut renderer.chunk_size));
            });
            
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label("Cull Chunks");
                ui.checkbox(&mut renderer.detect_chunk_culling, "");
            });
        }
        
         // Mask Interaction
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Mask Interaction");
             egui::ComboBox::from_id_salt("mask_interaction")
                .selected_text(match renderer.mask_interaction {
                    MaskInteraction::None => "None",
                    MaskInteraction::VisibleInsideMask => "Visible Inside Mask",
                    MaskInteraction::VisibleOutsideMask => "Visible Outside Mask",
                })
                .show_ui(ui, |ui: &mut egui::Ui| {
                    ui.selectable_value(&mut renderer.mask_interaction, MaskInteraction::None, "None");
                    ui.selectable_value(&mut renderer.mask_interaction, MaskInteraction::VisibleInsideMask, "Visible Inside Mask");
                    ui.selectable_value(&mut renderer.mask_interaction, MaskInteraction::VisibleOutsideMask, "Visible Outside Mask");
                });
        });
    });

    if is_removed {
        let _ = world.remove_component(entity, ecs::ComponentType::TilemapRenderer);
    }
}

pub fn render_grid_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    if !world.has_component(entity, ecs::ComponentType::Grid) {
        return;
    }

    let is_removed = utils::render_component(ui, "Grid", "▦", |ui: &mut egui::Ui| {
        let grid = match world.grids.get_mut(&entity) {
            Some(g) => g,
            None => return,
        };

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Cell Size");
            ui.add(egui::DragValue::new(&mut grid.cell_size.0).prefix("X: "));
            ui.add(egui::DragValue::new(&mut grid.cell_size.1).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut grid.cell_size.2).prefix("Z: "));
        });

        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Cell Gap");
            ui.add(egui::DragValue::new(&mut grid.cell_gap.0).prefix("X: "));
            ui.add(egui::DragValue::new(&mut grid.cell_gap.1).prefix("Y: "));
        });
        
        // Layout and Swizzle could be enums, but for now just showing basic props
        // Assuming they are enums as per ecs definition, let's just use basics or defaults for MVP
        // If we need detailed Enum combo boxes, we can add them later.
    });

    if is_removed {
        let _ = world.remove_component(entity, ecs::ComponentType::Grid);
    }
}

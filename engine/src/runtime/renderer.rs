//! Game Runtime Renderer
//!
//! Renders the game scene using Camera components from the ECS.
//! This is separate from the editor's scene view.

use ecs::{World, Camera, Entity};
use egui;
use crate::texture_manager::TextureManager;

/// Render the game view using the main camera
pub fn render_game_view(
    ui: &mut egui::Ui,
    world: &World,
    _texture_manager: &mut TextureManager,
    ui_manager: Option<&mut crate::ui_manager::UIManager>,
    game_view_settings: Option<&crate::runtime::GameViewSettings>,
) {
    let available_rect = ui.available_rect_before_wrap();
    
    // Calculate game view rect based on settings
    let game_rect = if let Some(settings) = game_view_settings {
        settings.calculate_game_rect(available_rect)
    } else {
        available_rect
    };
    
    // Fill background outside game view
    if let Some(settings) = game_view_settings {
        if !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
            let bg_color = egui::Color32::from_rgba_unmultiplied(
                (settings.background_color[0] * 255.0) as u8,
                (settings.background_color[1] * 255.0) as u8,
                (settings.background_color[2] * 255.0) as u8,
                (settings.background_color[3] * 255.0) as u8,
            );
            ui.painter().rect_filled(available_rect, 0.0, bg_color);
        }
    }
    
    let rect = game_rect;
    let painter = ui.painter_at(rect);

    // Find the main camera (first active camera with lowest depth)
    let main_camera = find_main_camera(world);

    if let Some((_camera_entity, _camera, _transform)) = main_camera {
        // Clear background
        // Clear background - DISABLED (Let WGPU render pass clear it)
        /*
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                (camera.background_color[0] * 255.0) as u8,
                (camera.background_color[1] * 255.0) as u8,
                (camera.background_color[2] * 255.0) as u8,
                (camera.background_color[3] * 255.0) as u8,
            ),
        );
        */

        // Render all entities
        // Render all entities - DISABLED (Let WGPU render them)
        // render_entities(ui, world, camera, transform, rect, texture_manager);
        
        // Render UI system on top
        if let Some(ui_mgr) = ui_manager {
            ui_mgr.render(ui, world, rect);
        }
        
        // Render game view overlays (resolution info, safe area)
        if let Some(settings) = game_view_settings {
            render_game_view_overlays(ui, rect, settings);
        }
    } else {
        // No camera found - show default view
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(30, 30, 35),
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Camera Found\n\nAdd a Camera component to an entity",
            egui::FontId::proportional(16.0),
            egui::Color32::from_rgb(150, 150, 150),
        );
    }
}

/// Render game view overlays (resolution info, safe area guides)
fn render_game_view_overlays(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    settings: &crate::runtime::GameViewSettings,
) {
    let painter = ui.painter();
    
    // Show resolution info
    if settings.show_resolution_info && !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
        let (w, h) = settings.resolution.get_size();
        let info_text = format!(
            "{}\n{}x{} ({}%)",
            settings.resolution.get_name(),
            w, h,
            (settings.scale * 100.0) as i32
        );
        
        // Background for text
        let text_pos = rect.left_top() + egui::vec2(8.0, 8.0);
        let text_galley = painter.layout_no_wrap(
            info_text.clone(),
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
        let text_rect = egui::Rect::from_min_size(
            text_pos,
            text_galley.size() + egui::vec2(8.0, 4.0),
        );
        painter.rect_filled(
            text_rect,
            2.0,
            egui::Color32::from_black_alpha(180),
        );
        painter.text(
            text_pos + egui::vec2(4.0, 2.0),
            egui::Align2::LEFT_TOP,
            info_text,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }
    
    // Show safe area guides
    if settings.show_safe_area {
        let safe_margin = 0.05; // 5% margin
        let safe_rect = rect.shrink2(egui::vec2(
            rect.width() * safe_margin,
            rect.height() * safe_margin,
        ));
        
        // Draw safe area border
        painter.rect_stroke(
            safe_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)),
            egui::epaint::StrokeKind::Outside,
        );
        
        // Draw corner markers
        let marker_size = 10.0;
        let corners = [
            safe_rect.left_top(),
            safe_rect.right_top(),
            safe_rect.left_bottom(),
            safe_rect.right_bottom(),
        ];
        
        for corner in corners {
            painter.line_segment(
                [corner, corner + egui::vec2(marker_size, 0.0)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.line_segment(
                [corner, corner + egui::vec2(0.0, marker_size)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
            );
        }
    }
    
    // Draw border around game view
    if !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
            egui::epaint::StrokeKind::Outside,
        );
    }
}

/// Find the main camera (lowest depth, active)
fn find_main_camera(world: &World) -> Option<(Entity, &Camera, &ecs::Transform)> {
    let mut cameras: Vec<_> = world.cameras.iter()
        .filter_map(|(entity, camera)| {
            // Check if entity is active
            if world.active.get(entity).copied().unwrap_or(true) {
                world.transforms.get(entity).map(|transform| (*entity, camera, transform))
            } else {
                None
            }
        })
        .collect();

    // Sort by depth (lowest first)
    cameras.sort_by_key(|(_, camera, _)| camera.depth);

    cameras.into_iter().next()
}



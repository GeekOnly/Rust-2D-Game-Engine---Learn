use egui;
use crate::editor::{Console, AssetManager, DragDropState};
use crate::editor::ui::asset_browser::AssetBrowser;

/// Renders the bottom panel with Assets and Console tabs
pub fn render_bottom_panel(
    ui: &mut egui::Ui,
    bottom_panel_tab: &mut usize,
    asset_manager: &mut Option<AssetManager>,
    console: &mut Console,
    drag_drop: &mut DragDropState,
) {
    // Tab bar
    ui.horizontal(|ui| {
        ui.selectable_value(bottom_panel_tab, 0, "📦 Assets");
        ui.selectable_value(bottom_panel_tab, 1, "📝 Console");
    });
    
    ui.separator();
    
    match *bottom_panel_tab {
        0 => {
            // Assets tab
            if let Some(ref mut manager) = asset_manager {
                AssetBrowser::render(ui, manager, drag_drop);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No project open");
                });
            }
        }
        1 => {
            // Console tab
            console.render(ui);
        }
        _ => {}
    }
}

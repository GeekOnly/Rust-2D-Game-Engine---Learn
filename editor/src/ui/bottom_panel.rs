use egui;
use crate::{Console, AssetManager, DragDropState};
use crate::ui::asset_browser::{AssetBrowser, AssetBrowserAction};

/// Renders the bottom panel with Assets and Console tabs
pub fn render_bottom_panel(
    ui: &mut egui::Ui,
    bottom_panel_tab: &mut usize,
    asset_manager: &mut Option<AssetManager>,
    console: &mut Console,
    drag_drop: &mut DragDropState,
    texture_manager: &mut engine::texture_manager::TextureManager,
    project_path: Option<&std::path::PathBuf>,
) -> Option<AssetBrowserAction> {
    let mut action = None;
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
                action = AssetBrowser::render(ui, manager, drag_drop, texture_manager, project_path);
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
    
    action
}

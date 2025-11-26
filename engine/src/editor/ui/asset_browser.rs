/// Unity/Unreal-like Asset Browser UI
use egui::{Color32, Rect, Response, Sense, Stroke, Vec2};
use crate::editor::asset_manager::{AssetManager, AssetMetadata, AssetType, ViewMode, SortMode};
use crate::editor::UnityTheme;

pub struct AssetBrowser;

impl AssetBrowser {
    /// Render asset browser panel
    pub fn render(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
    ) {
        let colors = UnityTheme::colors();
        
        // Toolbar
        ui.horizontal(|ui| {
            // Navigation buttons
            if ui.button("⬅").clicked() {
                asset_manager.navigate_back();
            }
            if ui.button("➡").clicked() {
                asset_manager.navigate_forward();
            }
            if ui.button("⬆").clicked() {
                asset_manager.navigate_up();
            }
            
            ui.separator();
            
            // Breadcrumbs
            let breadcrumbs = asset_manager.get_breadcrumbs();
            for (i, (name, path)) in breadcrumbs.iter().enumerate() {
                if i > 0 {
                    ui.label(">");
                }
                if ui.selectable_label(false, name).clicked() {
                    asset_manager.navigate_to(path);
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // View mode toggle
                if ui.selectable_label(asset_manager.view_mode == ViewMode::Grid, "⊞").clicked() {
                    asset_manager.view_mode = ViewMode::Grid;
                }
                if ui.selectable_label(asset_manager.view_mode == ViewMode::List, "☰").clicked() {
                    asset_manager.view_mode = ViewMode::List;
                }
                
                ui.separator();
                
                // Sort mode
                egui::ComboBox::from_id_source("sort_mode")
                    .selected_text(format!("{:?}", asset_manager.sort_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut asset_manager.sort_mode, SortMode::Name, "Name");
                        ui.selectable_value(&mut asset_manager.sort_mode, SortMode::Type, "Type");
                        ui.selectable_value(&mut asset_manager.sort_mode, SortMode::Size, "Size");
                        ui.selectable_value(&mut asset_manager.sort_mode, SortMode::Modified, "Modified");
                    });
            });
        });
        
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut asset_manager.search_query);
            if ui.button("✖").clicked() {
                asset_manager.search_query.clear();
            }
        });
        
        ui.separator();
        
        // Asset list
        egui::ScrollArea::vertical().show(ui, |ui| {
            let assets = asset_manager.get_assets();
            
            if assets.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No assets found");
                });
                return;
            }
            
            match asset_manager.view_mode {
                ViewMode::Grid => {
                    Self::render_grid_view(ui, asset_manager, &assets, colors);
                }
                ViewMode::List => {
                    Self::render_list_view(ui, asset_manager, &assets, colors);
                }
            }
        });
    }
    
    /// Render grid view
    fn render_grid_view(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        assets: &[AssetMetadata],
        colors: crate::editor::theme::UnityColors,
    ) {
        let thumbnail_size = asset_manager.thumbnail_size;
        let spacing = 10.0;
        let item_width = thumbnail_size + spacing;
        let available_width = ui.available_width();
        let columns = (available_width / item_width).floor().max(1.0) as usize;
        
        for row_assets in assets.chunks(columns) {
            ui.horizontal(|ui| {
                for asset in row_assets {
                    Self::render_grid_item(ui, asset_manager, asset, thumbnail_size, colors);
                }
            });
        }
    }
    
    /// Render single grid item
    fn render_grid_item(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        asset: &AssetMetadata,
        size: f32,
        colors: crate::editor::theme::UnityColors,
    ) {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(size, size + 30.0),
            Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let is_selected = asset_manager.selected_asset.as_ref() == Some(&asset.path);
            let is_favorite = asset_manager.is_favorite(&asset.path);
            
            // Background
            let bg_color = if is_selected {
                colors.selected
            } else if response.hovered() {
                colors.bg_light
            } else {
                colors.bg_medium
            };
            
            ui.painter().rect_filled(rect, 4.0, bg_color);
            
            // Thumbnail area
            let thumb_rect = Rect::from_min_size(
                rect.min,
                Vec2::splat(size),
            );
            
            // Icon/Thumbnail
            let icon_color = asset.asset_type.color();
            let icon_bg = Color32::from_rgb(icon_color[0], icon_color[1], icon_color[2]);
            ui.painter().rect_filled(thumb_rect, 4.0, icon_bg);
            
            // Icon text
            ui.painter().text(
                thumb_rect.center(),
                egui::Align2::CENTER_CENTER,
                asset.asset_type.icon(),
                egui::FontId::proportional(32.0),
                Color32::WHITE,
            );
            
            // Favorite star
            if is_favorite {
                ui.painter().text(
                    thumb_rect.right_top() + Vec2::new(-10.0, 10.0),
                    egui::Align2::RIGHT_TOP,
                    "⭐",
                    egui::FontId::proportional(16.0),
                    Color32::YELLOW,
                );
            }
            
            // Name
            let name_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, size),
                Vec2::new(size, 30.0),
            );
            
            let name = if asset.name.len() > 12 {
                format!("{}...", &asset.name[..9])
            } else {
                asset.name.clone()
            };
            
            ui.painter().text(
                name_rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(11.0),
                colors.text,
            );
            
            // Handle click
            if response.clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else {
                    asset_manager.selected_asset = Some(asset.path.clone());
                }
            }
            
            // Context menu
            response.context_menu(|ui| {
                Self::render_context_menu(ui, asset_manager, asset);
            });
            
            // Double click to open
            if response.double_clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                }
                // TODO: Open asset in appropriate editor
            }
        }
    }
    
    /// Render list view
    fn render_list_view(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        assets: &[AssetMetadata],
        _colors: crate::editor::theme::UnityColors,
    ) {
        // Header
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.separator();
            ui.label("Type");
            ui.separator();
            ui.label("Size");
            ui.separator();
            ui.label("Modified");
        });
        
        ui.separator();
        
        // Items
        for asset in assets {
            let is_selected = asset_manager.selected_asset.as_ref() == Some(&asset.path);
            let is_favorite = asset_manager.is_favorite(&asset.path);
            
            let response = ui.horizontal(|ui| {
                // Icon
                ui.label(asset.asset_type.icon());
                
                // Favorite
                if is_favorite {
                    ui.label("⭐");
                } else {
                    ui.label("  ");
                }
                
                // Name
                let name_response = ui.selectable_label(is_selected, &asset.name);
                
                ui.separator();
                
                // Type
                ui.label(format!("{:?}", asset.asset_type));
                
                ui.separator();
                
                // Size
                if asset.asset_type != AssetType::Folder {
                    ui.label(AssetManager::format_size(asset.size));
                } else {
                    ui.label("-");
                }
                
                ui.separator();
                
                // Modified
                if let Ok(elapsed) = asset.modified.elapsed() {
                    let secs = elapsed.as_secs();
                    let time_str = if secs < 60 {
                        format!("{}s ago", secs)
                    } else if secs < 3600 {
                        format!("{}m ago", secs / 60)
                    } else if secs < 86400 {
                        format!("{}h ago", secs / 3600)
                    } else {
                        format!("{}d ago", secs / 86400)
                    };
                    ui.label(time_str);
                } else {
                    ui.label("-");
                }
                
                name_response
            }).inner;
            
            // Handle click
            if response.clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else {
                    asset_manager.selected_asset = Some(asset.path.clone());
                }
            }
            
            // Context menu
            response.context_menu(|ui| {
                Self::render_context_menu(ui, asset_manager, asset);
            });
            
            // Double click
            if response.double_clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                }
            }
        }
    }
    
    /// Render context menu
    fn render_context_menu(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        asset: &AssetMetadata,
    ) {
        ui.label(format!("📝 {}", asset.name));
        ui.separator();
        
        if asset.asset_type != AssetType::Folder {
            if ui.button("Open").clicked() {
                // TODO: Open asset
                ui.close_menu();
            }
            ui.separator();
        }
        
        let fav_text = if asset_manager.is_favorite(&asset.path) {
            "Remove from Favorites"
        } else {
            "Add to Favorites"
        };
        
        if ui.button(fav_text).clicked() {
            asset_manager.toggle_favorite(&asset.path);
            ui.close_menu();
        }
        
        ui.separator();
        
        if ui.button("Show in Explorer").clicked() {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer")
                    .arg("/select,")
                    .arg(&asset.path)
                    .spawn();
            }
            ui.close_menu();
        }
        
        if ui.button("Copy Path").clicked() {
            // TODO: Copy to clipboard
            ui.close_menu();
        }
        
        ui.separator();
        
        if ui.button("🗑 Delete").clicked() {
            // TODO: Delete with confirmation
            ui.close_menu();
        }
    }
}

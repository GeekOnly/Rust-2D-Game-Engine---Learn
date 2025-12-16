/// Unity/Unreal-like Asset Browser UI
use egui::{Color32, Rect, Sense, Vec2};
use crate::asset_manager::{AssetManager, AssetMetadata, AssetType, ViewMode, SortMode};
use crate::{UnityTheme, DragDropState, DraggedAsset};
use std::path::PathBuf;

pub struct AssetBrowser;

impl AssetBrowser {
    /// Render asset browser panel (Unity-like 2-column layout)
    pub fn render(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        drag_drop: &mut DragDropState,
        texture_manager: &mut engine::texture_manager::TextureManager,
        project_path: Option<&PathBuf>,
    ) -> Option<AssetBrowserAction> {
        let mut action = None;
        let colors = UnityTheme::colors();
        
        // Toolbar (above both columns)
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
        
        // Unity-like 2-column layout: Folder tree (left) + Asset view (right)
        ui.columns(2, |columns| {
            // Left column: Folder tree
            egui::ScrollArea::vertical()
                .id_source("folder_tree")
                .show(&mut columns[0], |ui| {
                    Self::render_folder_tree(ui, asset_manager, colors);
                });
            
            // Right column: Asset list
            egui::ScrollArea::vertical()
                .id_source("asset_list")
                .show(&mut columns[1], |ui| {
            let assets = asset_manager.get_assets();
            
            if assets.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No assets found");
                });
                return;
            }
            
            match asset_manager.view_mode {
                ViewMode::Grid => {
                    if let Some(a) = Self::render_grid_view(ui, asset_manager, &assets, colors, drag_drop, texture_manager, project_path) {
                        action = Some(a);
                    }
                }
                ViewMode::List => {
                    if let Some(a) = Self::render_list_view(ui, asset_manager, &assets, colors, drag_drop) {
                        action = Some(a);
                    }
                }
            }
                });
        });
        
        action
    }
    
    /// Render folder tree (Unity-like hierarchy)
    fn render_folder_tree(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        colors: crate::theme::UnityColors,
    ) {
        ui.label(egui::RichText::new("Folders").strong());
        ui.separator();
        
        // Get project root (go up from current path until we find project root)
        let mut project_root = asset_manager.current_path.clone();
        while project_root.parent().is_some() {
            // Check if this looks like project root (has scenes, scripts, etc.)
            if project_root.join("scenes").exists() || project_root.join("scripts").exists() {
                break;
            }
            if let Some(parent) = project_root.parent() {
                project_root = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        // Assets folder (project root)
        Self::render_folder_tree_node(ui, asset_manager, &project_root, "Assets", 0, colors);
        
        // Packages folder (if exists)
        let packages_path = project_root.join("packages");
        if packages_path.exists() {
            Self::render_folder_tree_node(ui, asset_manager, &packages_path, "Packages", 0, colors);
        }
    }
    
    /// Render single folder tree node (recursive)
    fn render_folder_tree_node(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        path: &std::path::PathBuf,
        name: &str,
        depth: usize,
        colors: crate::theme::UnityColors,
    ) {
        let indent = depth as f32 * 16.0;
        let is_current = asset_manager.current_path == *path;
        
        ui.horizontal(|ui| {
            ui.add_space(indent);
            
            // Folder icon and name
            let icon = if path.is_dir() { "📁" } else { "📄" };
            let text = format!("{} {}", icon, name);
            
            let response = ui.selectable_label(is_current, text);
            
            if response.clicked() {
                asset_manager.navigate_to(path);
            }
        });
        
        // Show subfolders if this is a directory
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut folders: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .collect();
                
                folders.sort_by(|a, b| {
                    a.file_name().cmp(&b.file_name())
                });
                
                for entry in folders {
                    let sub_path = entry.path();
                    let sub_name = entry.file_name().to_string_lossy().to_string();
                    Self::render_folder_tree_node(ui, asset_manager, &sub_path, &sub_name, depth + 1, colors);
                }
            }
        }
    }
    
    /// Render grid view
    fn render_grid_view(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        assets: &[AssetMetadata],
        colors: crate::theme::UnityColors,
        drag_drop: &mut DragDropState,
        texture_manager: &mut engine::texture_manager::TextureManager,
        project_path: Option<&PathBuf>,
    ) -> Option<AssetBrowserAction> {
        let mut action = None;
        let thumbnail_size = asset_manager.thumbnail_size;
        let spacing = 10.0;
        let item_width = thumbnail_size + spacing;
        let available_width = ui.available_width();
        let columns = (available_width / item_width).floor().max(1.0) as usize;
        
        for row_assets in assets.chunks(columns) {
            ui.horizontal(|ui| {
                for asset in row_assets {
                    if let Some(a) = Self::render_grid_item(ui, asset_manager, asset, thumbnail_size, colors, drag_drop, texture_manager, project_path) {
                        action = Some(a);
                    }
                }
            });
        }
        
        action
    }
    
    /// Render single grid item
    fn render_grid_item(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        asset: &AssetMetadata,
        size: f32,
        colors: crate::theme::UnityColors,
        drag_drop: &mut DragDropState,
        texture_manager: &mut engine::texture_manager::TextureManager,
        project_path: Option<&PathBuf>,
    ) -> Option<AssetBrowserAction> {
        let mut action = None;
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
            
            // Try to load texture preview for sprites
            let mut show_icon = true;
            if matches!(asset.asset_type, AssetType::Sprite | AssetType::SpriteSheet) {
                if let Some(project_path) = project_path {
                    // Calculate relative path from project root
                    let relative_path = if let Ok(rel) = asset.path.strip_prefix(project_path) {
                        rel
                    } else {
                        asset.path.as_path()
                    };
                    
                    // Load texture
                    let texture_id = format!("asset_preview_{}", asset.path.display());
                    if let Some(texture) = texture_manager.load_texture(
                        ui.ctx(),
                        &texture_id,
                        relative_path
                    ) {
                        // Draw texture preview
                        let preview_rect = thumb_rect.shrink(4.0);
                        ui.painter().image(
                            texture.id(),
                            preview_rect,
                            egui::Rect::from_min_max(
                                egui::pos2(0.0, 0.0),
                                egui::pos2(1.0, 1.0)
                            ),
                            Color32::WHITE
                        );
                        show_icon = false;
                    }
                }
            }
            
            // Fallback: Icon text
            if show_icon {
                ui.painter().text(
                    thumb_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    asset.asset_type.icon(),
                    egui::FontId::proportional(32.0),
                    Color32::WHITE,
                );
            }
            
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
            
            // Handle drag
            if response.drag_started() && asset.asset_type != AssetType::Folder {
                drag_drop.start_drag(DraggedAsset {
                    path: asset.path.clone(),
                    name: asset.name.clone(),
                    asset_type: asset.asset_type.clone(),
                });
            }
            
            // Handle click
            if response.clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else {
                    asset_manager.selected_asset = Some(asset.path.clone());
                    
                    // If it's a texture, trigger SelectTexture action to show import settings
                    if matches!(asset.asset_type, AssetType::Sprite | AssetType::SpriteSheet) {
                        action = Some(AssetBrowserAction::SelectTexture(asset.path.clone()));
                    }
                }
            }
            
            // Context menu
            response.context_menu(|ui| {
                if let Some(a) = Self::render_context_menu(ui, asset_manager, asset) {
                    action = Some(a);
                }
            });
            
            // Double click to open
            if response.double_clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else if let Some(ext) = asset.path.extension() {
                    match ext.to_str() {
                        Some("uiprefab") => {
                            action = Some(AssetBrowserAction::OpenUIPrefabEditor(asset.path.clone()));
                        }
                        _ => {
                            // TODO: Open other asset types
                        }
                    }
                }
            }
        }
        
        action
    }
    
    /// Render list view
    fn render_list_view(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        assets: &[AssetMetadata],
        _colors: crate::theme::UnityColors,
        drag_drop: &mut DragDropState,
    ) -> Option<AssetBrowserAction> {
        let mut action = None;
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
            
            // Handle drag
            if response.drag_started() && asset.asset_type != AssetType::Folder {
                drag_drop.start_drag(DraggedAsset {
                    path: asset.path.clone(),
                    name: asset.name.clone(),
                    asset_type: asset.asset_type.clone(),
                });
            }
            
            // Handle click
            if response.clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else {
                    asset_manager.selected_asset = Some(asset.path.clone());
                    
                    // If it's a texture, trigger SelectTexture action to show import settings
                    if matches!(asset.asset_type, AssetType::Sprite | AssetType::SpriteSheet) {
                        action = Some(AssetBrowserAction::SelectTexture(asset.path.clone()));
                    }
                }
            }
            
            // Context menu
            response.context_menu(|ui| {
                if let Some(a) = Self::render_context_menu(ui, asset_manager, asset) {
                    action = Some(a);
                }
            });
            
            // Double click
            if response.double_clicked() {
                if asset.asset_type == AssetType::Folder {
                    asset_manager.navigate_to(&asset.path);
                } else if let Some(ext) = asset.path.extension() {
                    match ext.to_str() {
                        Some("uiprefab") => {
                            action = Some(AssetBrowserAction::OpenUIPrefabEditor(asset.path.clone()));
                        }
                        _ => {
                            // TODO: Open other asset types
                        }
                    }
                }
            }
        }
        
        action
    }
    
    /// Render context menu
    fn render_context_menu(
        ui: &mut egui::Ui,
        asset_manager: &mut AssetManager,
        asset: &AssetMetadata,
    ) -> Option<AssetBrowserAction> {
        let mut action = None;
        
        ui.label(format!("📝 {}", asset.name));
        ui.separator();
        
        if asset.asset_type != AssetType::Folder {
            if ui.button("Open").clicked() {
                // TODO: Open asset
                ui.close_menu();
            }
            
            // Add "Open in Sprite Editor" for PNG files
            if asset.asset_type == AssetType::Sprite {
                if ui.button("🎨 Open in Sprite Editor").clicked() {
                    action = Some(AssetBrowserAction::OpenSpriteEditor(asset.path.clone()));
                    ui.close_menu();
                }
            }
            
            // Add "Edit Sprite Sheet" for .sprite files
            if asset.asset_type == AssetType::SpriteSheet {
                if ui.button("✏ Edit Sprite Sheet").clicked() {
                    // Load the .sprite file to get the texture path
                    if let Ok(metadata) = sprite_editor::SpriteMetadata::load(&asset.path) {
                        // Get the texture path (relative to project)
                        let texture_path = std::path::PathBuf::from(&metadata.texture_path);
                        action = Some(AssetBrowserAction::OpenSpriteEditor(texture_path));
                    }
                    ui.close_menu();
                }
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
        
        action
    }
}

/// Actions that can be triggered from the asset browser
#[derive(Debug, Clone)]
pub enum AssetBrowserAction {
    OpenSpriteEditor(PathBuf),
    SelectTexture(PathBuf),  // Select texture to show import settings
    OpenUIPrefabEditor(PathBuf),  // Open UI Prefab Editor
}

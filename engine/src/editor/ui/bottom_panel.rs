use egui;
use crate::editor::Console;
use crate::editor::ui::resource_manager;
use std::path::PathBuf;

/// Renders the bottom panel with Resources and Console tabs
pub fn render_bottom_panel(
    ui: &mut egui::Ui,
    bottom_panel_tab: &mut usize,
    project_path: &Option<PathBuf>,
    resource_current_folder: &mut String,
    load_file_request: &mut Option<PathBuf>,
    console: &mut Console,
) {
    // Modern tab bar with background
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(45, 45, 48))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 4.0;

                // Resources tab
                let resources_color = if *bottom_panel_tab == 0 {
                    egui::Color32::from_rgb(0, 122, 204)
                } else {
                    egui::Color32::from_rgb(60, 60, 60)
                };

                if ui.add(egui::Button::new(
                    egui::RichText::new("🗂️ Resources").size(13.0).color(egui::Color32::WHITE)
                )
                .fill(resources_color)
                .min_size(egui::vec2(100.0, 28.0)))
                .clicked() {
                    *bottom_panel_tab = 0;
                }

                // Console tab
                let console_color = if *bottom_panel_tab == 1 {
                    egui::Color32::from_rgb(0, 122, 204)
                } else {
                    egui::Color32::from_rgb(60, 60, 60)
                };

                if ui.add(egui::Button::new(
                    egui::RichText::new("📝 Console").size(13.0).color(egui::Color32::WHITE)
                )
                .fill(console_color)
                .min_size(egui::vec2(100.0, 28.0)))
                .clicked() {
                    *bottom_panel_tab = 1;
                }
            });
        });

    ui.separator();

    match *bottom_panel_tab {
        0 => {
            // RESOURCES TAB - Modern Resource Manager
            if let Some(proj_path) = project_path {
                // Modern toolbar
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(37, 37, 38))
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Back button
                            if !resource_current_folder.is_empty() {
                                if ui.button("◀").on_hover_text("Back").clicked() {
                                    // Navigate to parent folder
                                    if let Some(pos) = resource_current_folder.rfind('/') {
                                        *resource_current_folder = resource_current_folder[..pos].to_string();
                                    } else {
                                        resource_current_folder.clear();
                                    }
                                }
                                ui.add_space(5.0);
                            }

                            // Breadcrumb navigation
                            ui.label(egui::RichText::new("📁").size(16.0));

                            if ui.link(egui::RichText::new("Project").strong().size(13.0)).clicked() {
                                resource_current_folder.clear();
                            }

                            if !resource_current_folder.is_empty() {
                                ui.label(egui::RichText::new("/").color(egui::Color32::GRAY));

                                // Display current path
                                let path_parts: Vec<String> = resource_current_folder.split('/').map(|s| s.to_string()).collect();
                                for (i, part) in path_parts.iter().enumerate() {
                                    if i > 0 {
                                        ui.label(egui::RichText::new("/").color(egui::Color32::GRAY));
                                    }

                                    if i == path_parts.len() - 1 {
                                        // Current folder (not clickable)
                                        ui.label(egui::RichText::new(part).size(13.0));
                                    } else {
                                        // Parent folders (clickable)
                                        let partial_path = path_parts[..=i].join("/");
                                        if ui.link(egui::RichText::new(part).size(13.0)).clicked() {
                                            *resource_current_folder = partial_path;
                                        }
                                    }
                                }
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // View options
                                ui.label(egui::RichText::new("⊞").size(16.0).color(egui::Color32::LIGHT_GRAY))
                                    .on_hover_text("Grid View");
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("🔍").size(16.0).color(egui::Color32::LIGHT_GRAY))
                                    .on_hover_text("Search Resources");
                            });
                        });
                    });

                ui.add_space(5.0);

                // Unity/Unreal-style 2-panel layout
                ui.horizontal(|ui| {
                    // LEFT PANEL - Folder Tree
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(37, 37, 38))
                        .show(ui, |ui| {
                            ui.set_width(220.0);
                            ui.set_min_height(ui.available_height());

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.add_space(5.0);

                                    // Assets root (always expanded)
                                    ui.horizontal(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(egui::RichText::new("▼").size(10.0).color(egui::Color32::GRAY));
                                        ui.add_space(3.0);
                                        ui.label(egui::RichText::new("📁").size(14.0));
                                        ui.add_space(3.0);

                                        let is_root_selected = resource_current_folder.is_empty();
                                        let text_color = if is_root_selected {
                                            egui::Color32::from_rgb(0, 122, 204)
                                        } else {
                                            egui::Color32::LIGHT_GRAY
                                        };

                                        if ui.selectable_label(is_root_selected, egui::RichText::new("Assets").color(text_color).strong()).clicked() {
                                            resource_current_folder.clear();
                                        }
                                    });

                                    ui.add_space(3.0);

                                    // Main folders
                                    let folders = vec![
                                        ("scenes", "🎬", "Scenes"),
                                        ("scripts", "📜", "Scripts"),
                                        ("sprites", "🖼️", "Sprites"),
                                        ("audio", "🔊", "Audio"),
                                    ];

                                    for (folder_name, icon, display_name) in folders {
                                        let folder_path = proj_path.join(folder_name);
                                        if !folder_path.exists() {
                                            continue;
                                        }

                                        ui.horizontal(|ui| {
                                            ui.add_space(15.0);

                                            // Expand/collapse triangle (future: for subfolders)
                                            ui.label(egui::RichText::new("▶").size(10.0).color(egui::Color32::GRAY));
                                            ui.add_space(3.0);

                                            ui.label(egui::RichText::new(icon).size(14.0));
                                            ui.add_space(3.0);

                                            let is_selected = *resource_current_folder == folder_name;
                                            let text_color = if is_selected {
                                                egui::Color32::from_rgb(0, 122, 204)
                                            } else {
                                                egui::Color32::LIGHT_GRAY
                                            };

                                            if ui.selectable_label(is_selected, egui::RichText::new(display_name).color(text_color)).clicked() {
                                                *resource_current_folder = folder_name.to_string();
                                            }
                                        });

                                        ui.add_space(2.0);
                                    }
                                });
                        });

                    ui.separator();

                    // RIGHT PANEL - Content View
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 30, 30))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.set_min_height(ui.available_height());

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.add_space(10.0);

                                    // Get current folder path
                                    let current_path = if resource_current_folder.is_empty() {
                                        proj_path.clone()
                                    } else {
                                        proj_path.join(&*resource_current_folder)
                                    };

                                    ui.horizontal_wrapped(|ui| {
                                        ui.style_mut().spacing.item_spacing = egui::vec2(12.0, 12.0);
                                        ui.set_min_width(ui.available_width());

                                        // Show folders and files in current directory
                                        if resource_current_folder.is_empty() {
                                            // Root level - show main folders
                                            let folders = vec![
                                                ("scenes", "🎬", "Scenes", egui::Color32::from_rgb(100, 150, 255)),
                                                ("scripts", "📜", "Scripts", egui::Color32::from_rgb(255, 200, 100)),
                                                ("sprites", "🖼️", "Sprites", egui::Color32::from_rgb(150, 255, 150)),
                                                ("audio", "🔊", "Audio", egui::Color32::from_rgb(255, 150, 200)),
                                            ];

                                            for (folder_name, icon, display_name, accent_color) in folders {
                                                let folder_path = proj_path.join(folder_name);
                                                let exists = folder_path.exists();

                                                let mut file_count = 0;
                                                if exists {
                                                    if let Ok(entries) = std::fs::read_dir(&folder_path) {
                                                        file_count = entries.count();
                                                    }
                                                }

                                                // Render folder card using helper
                                                let count_label = if exists {
                                                    format!("{} items", file_count)
                                                } else {
                                                    "Empty".to_string()
                                                };
                                                let card_response = resource_manager::render_resource_card(
                                                    ui,
                                                    icon,
                                                    display_name,
                                                    &count_label,
                                                    accent_color
                                                );

                                                if card_response.clicked() && exists {
                                                    // Navigate into folder
                                                    if resource_current_folder.is_empty() {
                                                        *resource_current_folder = folder_name.to_string();
                                                    } else {
                                                        *resource_current_folder = format!("{}/{}", resource_current_folder, folder_name);
                                                    }
                                                }

                                                // Right-click context menu for folders
                                                card_response.context_menu(|ui| {
                                                    resource_manager::render_folder_context_menu(ui, folder_name, &folder_path);
                                                });
                                            }
                                        } else {
                                            // Inside a folder - show files
                                            if current_path.exists() {
                                                if let Ok(entries) = std::fs::read_dir(&current_path) {
                                                    let mut entries: Vec<_> = entries.flatten().collect();
                                                    entries.sort_by_key(|e| e.file_name());

                                                    for entry in entries {
                                                        if let Some(name) = entry.file_name().to_str() {
                                                            let path = entry.path();
                                                            let is_dir = path.is_dir();

                                                            // Get file type info using helper
                                                            let file_info = resource_manager::get_file_type_info(name, is_dir);

                                                            // Truncate long names
                                                            let display_name = if name.len() > 15 {
                                                                format!("{}...", &name[..12])
                                                            } else {
                                                                name.to_string()
                                                            };

                                                            // Render card using helper
                                                            let card_response = resource_manager::render_resource_card(
                                                                ui,
                                                                file_info.icon,
                                                                &display_name,
                                                                file_info.type_label,
                                                                file_info.accent_color
                                                            );

                                                            // Click handling
                                                            if card_response.clicked() {
                                                                if is_dir {
                                                                    // Navigate into subfolder
                                                                    *resource_current_folder = format!("{}/{}", resource_current_folder, name);
                                                                }
                                                            }

                                                            if card_response.double_clicked() && !is_dir {
                                                                if name.ends_with(".scene") {
                                                                    *load_file_request = Some(path.clone());
                                                                }
                                                            }

                                                            // Context menu for files
                                                            if !is_dir {
                                                                card_response.context_menu(|ui| {
                                                                    resource_manager::render_file_context_menu(ui, &path, name, load_file_request);
                                                                });
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });  // Close horizontal_wrapped (content grid)
                                });      // Close ScrollArea::vertical (right panel scroll)
                        });              // Close Frame::none (right panel frame)
                });                      // Close ui.horizontal (2-panel container)
            } else {
                // No project open - modern empty state
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(egui::RichText::new("🗂️").size(48.0).color(egui::Color32::DARK_GRAY));
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("No Project Open")
                            .size(18.0)
                            .strong()
                            .color(egui::Color32::GRAY)
                    );
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("Open a project to view resources")
                            .size(13.0)
                            .color(egui::Color32::DARK_GRAY)
                    );
                });
            }
        }
        1 => {
            // CONSOLE TAB
            console.render(ui);
        }
        _ => {}
    }
}

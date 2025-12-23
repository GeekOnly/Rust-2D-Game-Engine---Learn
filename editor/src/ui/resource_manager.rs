use egui;
use std::path::{Path, PathBuf};

// Helper struct for file type information
pub struct FileTypeInfo {
    pub icon: &'static str,
    pub accent_color: egui::Color32,
    pub type_label: &'static str,
}

/// Get file type information based on filename
pub fn get_file_type_info(filename: &str, is_dir: bool) -> FileTypeInfo {
    if is_dir {
        FileTypeInfo {
            icon: "📁",
            accent_color: egui::Color32::from_rgb(255, 200, 100),
            type_label: "Folder",
        }
    } else if filename.ends_with(".scene") {
        FileTypeInfo {
            icon: "🎬",
            accent_color: egui::Color32::from_rgb(100, 150, 255),
            type_label: "Scene",
        }
    } else if filename.ends_with(".lua") {
        FileTypeInfo {
            icon: "📜",
            accent_color: egui::Color32::from_rgb(255, 200, 100),
            type_label: "Script",
        }
    } else if filename.ends_with(".png") || filename.ends_with(".jpg") {
        FileTypeInfo {
            icon: "🖼️",
            accent_color: egui::Color32::from_rgb(150, 255, 150),
            type_label: "Image",
        }
    } else if filename.ends_with(".wav") || filename.ends_with(".ogg") {
        FileTypeInfo {
            icon: "🔊",
            accent_color: egui::Color32::from_rgb(255, 150, 200),
            type_label: "Audio",
        }
    } else {
        FileTypeInfo {
            icon: "📄",
            accent_color: egui::Color32::GRAY,
            type_label: "File",
        }
    }
}

/// Render a resource card (folder or file)
pub fn render_resource_card(
    ui: &mut egui::Ui,
    icon: &str,
    display_name: &str,
    type_label: &str,
    accent_color: egui::Color32,
) -> egui::Response {
    let card_response = egui::Frame::none()
        .fill(egui::Color32::from_rgb(50, 50, 52))
        .rounding(egui::Rounding::same(6))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 72)))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(110.0, 90.0));
            ui.vertical_centered(|ui| {
                // Icon with accent color background
                egui::Frame::none()
                    .fill(accent_color.linear_multiply(0.3))
                    .rounding(egui::Rounding::same(8))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(icon).size(28.0).color(accent_color));
                    });

                ui.add_space(6.0);
                ui.label(egui::RichText::new(display_name).size(11.0));
                ui.label(
                    egui::RichText::new(type_label)
                        .size(9.0)
                        .color(egui::Color32::DARK_GRAY)
                );
            });
        });

    if card_response.response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    card_response.response
}

/// Render file context menu
pub fn render_file_context_menu(
    ui: &mut egui::Ui,
    path: &Path,
    filename: &str,
    load_file_request: &mut Option<PathBuf>,
) {
    ui.set_min_width(160.0);

    // Open/Edit options based on file type
    if filename.ends_with(".scene") {
        if ui.button("📂 Open Scene").clicked() {
            *load_file_request = Some(path.to_path_buf());
            ui.close_menu();
        }
        ui.separator();
    } else if filename.ends_with(".lua") {
        if ui.button("✏️ Edit Script").clicked() {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("notepad")
                    .arg(path)
                    .spawn();
            }
            ui.close_menu();
        }
        ui.separator();
    }

    // Duplicate
    if ui.button("📋 Duplicate").clicked() {
        if let Some(file_name) = path.file_stem() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let mut counter = 1;
            loop {
                let new_name = format!("{}_copy{}.{}", file_name.to_string_lossy(), counter, ext);
                let new_path = path.with_file_name(new_name);
                if !new_path.exists() {
                    let _ = std::fs::copy(path, &new_path);
                    break;
                }
                counter += 1;
            }
        }
        ui.close_menu();
    }

    // Delete
    if ui.button("🗑️ Delete").clicked() {
        let _ = std::fs::remove_file(path);
        ui.close_menu();
    }

    ui.separator();

    // Show in Explorer
    if ui.button("📂 Show in Explorer").clicked() {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn();
        }
        ui.close_menu();
    }
}

/// Render folder context menu for creating new resources
pub fn render_folder_context_menu(
    ui: &mut egui::Ui,
    folder_name: &str,
    folder_path: &Path,
) {
    ui.set_min_width(180.0);

    ui.menu_button("📄 Create", |ui| {
        if folder_name == "scenes" {
            if ui.button("🎬 New Scene").clicked() {
                create_new_scene(folder_path);
                ui.close_menu();
            }
        }

        if folder_name == "scripts" {
            if ui.button("📜 New Script").clicked() {
                create_new_script(folder_path);
                ui.close_menu();
            }
        }

        if ui.button("📁 New Folder").clicked() {
            create_new_folder(folder_path);
            ui.close_menu();
        }
    });

    ui.separator();

    if ui.button("📂 Show in Explorer").clicked() {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(folder_path)
                .spawn();
        }
        ui.close_menu();
    }

    if ui.button("🔄 Refresh").clicked() {
        ui.close_menu();
    }
}

/// Create new scene file
pub fn create_new_scene(folder_path: &Path) {
    let mut counter = 1;
    loop {
        let scene_name = format!("NewScene{}.scene", counter);
        let scene_path = folder_path.join(&scene_name);
        if !scene_path.exists() {
            let empty_scene_json = serde_json::json!({
                "entity_names": {},
                "world": {
                    "transforms": [],
                    "sprites": [],
                    "velocities": [],
                    "colliders": [],
                    "scripts": [],
                    "tags": [],
                    "parents": [],
                    "active": [],
                    "layers": [],
                    "next_entity": 0
                }
            });
            if let Ok(json) = serde_json::to_string_pretty(&empty_scene_json) {
                let _ = std::fs::write(&scene_path, json);
            }
            break;
        }
        counter += 1;
    }
}

/// Create new script file
pub fn create_new_script(folder_path: &Path) {
    let mut counter = 1;
    loop {
        let script_name = format!("NewScript_{}.lua", counter);
        let script_path = folder_path.join(&script_name);
        if !script_path.exists() {
            let template = "-- New Script\n\nfunction on_start()\n    print(\"Script started!\")\nend\n\nfunction on_update(dt)\n    -- Update logic here\nend\n";
            let _ = std::fs::write(&script_path, template);
            break;
        }
        counter += 1;
    }
}

/// Create new folder
pub fn create_new_folder(parent_path: &Path) {
    let mut counter = 1;
    loop {
        let new_folder_name = format!("NewFolder{}", counter);
        let new_folder_path = parent_path.join(&new_folder_name);
        if !new_folder_path.exists() {
            let _ = std::fs::create_dir(&new_folder_path);
            break;
        }
        counter += 1;
    }
}

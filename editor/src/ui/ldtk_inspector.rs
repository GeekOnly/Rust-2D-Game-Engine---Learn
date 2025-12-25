use egui;
use std::path::PathBuf;
use engine_core::assets::{AssetMetadata, ImportSettings, ldtk::LdtkImportSettings};
use ecs::{World, Entity};

/// Action to perform after rendering
pub enum LdtkInspectorAction {
    SaveSettings,
    RevertSettings,
}

pub struct LdtkInspector {
    pub selected_asset: Option<PathBuf>,
    pub settings: LdtkImportSettings,
    pub has_changes: bool,
}

impl LdtkInspector {
    pub fn new() -> Self {
        Self {
            selected_asset: None,
            settings: LdtkImportSettings::default(),
            has_changes: false,
        }
    }

    pub fn set_asset(&mut self, path: PathBuf) {
        // Load settings from .meta if exists
        self.selected_asset = Some(path.clone());
        self.has_changes = false;

        // Try to load existing meta
        let meta_path = AssetMetadata::get_meta_path(&path);
        if let Ok(meta) = AssetMetadata::load_from_file(&meta_path) {
            if let Some(ldtk_settings) = meta.import_settings.ldtk {
                self.settings = ldtk_settings;
                return;
            }
        }

        // Default
        self.settings = LdtkImportSettings::default();
    }

    pub fn clear(&mut self) {
        self.selected_asset = None;
        self.has_changes = false;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<LdtkInspectorAction> {
        let Some(path) = &self.selected_asset else {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("No LDtk file selected").weak());
                ui.label(egui::RichText::new("Select a .ldtk file from Asset Browser").small().weak());
            });
            return None;
        };

        let mut action = None;

        // Header
        ui.heading("🗺️ LDtk Tilemap");
        ui.label(egui::RichText::new(format!("📄 {}",
            path.file_name().and_then(|f| f.to_str()).unwrap_or("Unknown")
        )).weak());
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // === IMPORT SETTINGS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("⚙️ Import Settings").strong());
                ui.add_space(5.0);

                // Pixels Per Unit
                egui::Grid::new("ldtk_settings_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Pixels Per Unit:");
                        if ui.add(egui::DragValue::new(&mut self.settings.pixels_per_unit)
                            .speed(1.0)
                            .range(1.0..=1000.0)
                        ).changed() {
                            self.has_changes = true;
                        }
                        ui.end_row();
                    });

                ui.add_space(5.0);
                ui.label(egui::RichText::new(
                    "💡 Controls world scale. Common values:\n   • 16 = Retro pixel art\n   • 100 = Unity standard\n   • 32 = Moderate pixel art"
                ).small().weak());
            });

            ui.add_space(10.0);

            // === COLLISION SETTINGS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("🛡️ Collision Settings").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.settings.generate_colliders, "Auto-Generate Colliders").changed() {
                    self.has_changes = true;
                }

                if self.settings.generate_colliders {
                    ui.indent("collider_info", |ui| {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(
                            "✓ Generates BoxCollider2D from IntGrid layers\n✓ Value 1 = Solid collision\n✓ Automatically optimized into composite shapes"
                        ).small().weak());
                    });
                }
            });

            ui.add_space(10.0);

            // === USAGE INFO ===
            ui.collapsing("📖 How to Use", |ui| {
                ui.label(egui::RichText::new(
                    "1️⃣ Configure settings above\n\
                     2️⃣ Click 'Apply' to save\n\
                     3️⃣ Drag .ldtk file to Scene View to load\n\
                     4️⃣ Map will use these import settings"
                ).small());
            });

            ui.add_space(20.0);

            // === ACTION BUTTONS ===
            ui.horizontal(|ui| {
                ui.set_enabled(self.has_changes);

                if ui.button("💾 Apply").clicked() {
                    action = Some(LdtkInspectorAction::SaveSettings);
                }

                if ui.button("↩️ Revert").clicked() {
                    action = Some(LdtkInspectorAction::RevertSettings);
                }
            });

            if self.has_changes {
                ui.label(egui::RichText::new("⚠️ You have unsaved changes").small().color(egui::Color32::YELLOW));
            }
        });

        action
    }
}

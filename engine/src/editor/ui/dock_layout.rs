use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use ecs::{World, Entity};
use egui;
use std::collections::HashMap;
use crate::editor::{Console, SceneCamera, SceneGrid, AssetManager, DragDropState};
use super::{TransformTool, hierarchy, inspector, scene_view, asset_browser};

/// Tab types for the docking system
#[derive(Debug, Clone, PartialEq)]
pub enum EditorTab {
    Hierarchy,
    Inspector,
    Scene,
    Game,
    Console,
    Project,
}

/// Context for tab rendering
pub struct TabContext<'a> {
    pub world: &'a mut World,
    pub selected_entity: &'a mut Option<Entity>,
    pub entity_names: &'a mut HashMap<Entity, String>,
    pub edit_script_request: &'a mut Option<String>,
    pub project_path: &'a Option<std::path::PathBuf>,
    pub current_scene_path: &'a Option<std::path::PathBuf>,
    pub load_file_request: &'a mut Option<std::path::PathBuf>,
    pub console: &'a mut Console,
    pub scene_view_tab: &'a mut usize,
    pub is_playing: bool,
    pub show_colliders: &'a mut bool,
    pub show_velocities: &'a mut bool,
    pub current_tool: &'a mut TransformTool,
    pub scene_camera: &'a mut SceneCamera,
    pub scene_grid: &'a SceneGrid,
    pub play_request: &'a mut bool,
    pub stop_request: &'a mut bool,
    pub asset_manager: &'a mut Option<AssetManager>,
    pub drag_drop: &'a mut DragDropState,
}

/// Tab viewer implementation for egui_dock
pub struct EditorTabViewer<'a> {
    pub context: &'a mut TabContext<'a>,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Hierarchy => "Hierarchy".into(),
            EditorTab::Inspector => "Inspector".into(),
            EditorTab::Scene => "Scene".into(),
            EditorTab::Game => "Game".into(),
            EditorTab::Console => "Console".into(),
            EditorTab::Project => "Project".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Hierarchy => {
                hierarchy::render_hierarchy(
                    ui,
                    self.context.world,
                    self.context.entity_names,
                    self.context.selected_entity,
                    self.context.load_file_request,
                    self.context.project_path,
                    self.context.current_scene_path,
                    self.context.console,
                    get_scene_files,
                    &get_entity_icon,
                );
            }
            EditorTab::Inspector => {
                inspector::render_inspector(
                    ui,
                    self.context.world,
                    self.context.selected_entity,
                    self.context.entity_names,
                    self.context.edit_script_request,
                    self.context.project_path,
                );
            }
            EditorTab::Scene | EditorTab::Game => {
                scene_view::render_scene_view(
                    ui,
                    self.context.world,
                    self.context.selected_entity,
                    self.context.scene_view_tab,
                    self.context.is_playing,
                    self.context.show_colliders,
                    self.context.show_velocities,
                    self.context.current_tool,
                    self.context.scene_camera,
                    self.context.scene_grid,
                    self.context.play_request,
                    self.context.stop_request,
                );
            }
            EditorTab::Console => {
                // Render console (simple version for now)
                ui.label("Console");
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.label("Console messages will appear here");
                });
            }
            EditorTab::Project => {
                if let Some(ref mut manager) = self.context.asset_manager {
                    asset_browser::AssetBrowser::render(
                        ui,
                        manager,
                        self.context.drag_drop,
                    );
                } else {
                    ui.label("No project open");
                }
            }
        }
    }
}

/// Create default Unity-like dock layout
pub fn create_default_layout() -> DockState<EditorTab> {
    let mut dock_state = DockState::new(vec![EditorTab::Scene]);

    // Split to create left panel (Hierarchy)
    let [_left, main] = dock_state.main_surface_mut().split_left(
        NodeIndex::root(),
        0.2,
        vec![EditorTab::Hierarchy],
    );

    // Split to create right panel (Inspector)
    let [center, _right] = dock_state.main_surface_mut().split_right(
        main,
        0.25,
        vec![EditorTab::Inspector],
    );

    // Split to create bottom panel (Console/Project)
    let [_top, _bottom] = dock_state.main_surface_mut().split_below(
        center,
        0.7,
        vec![EditorTab::Console, EditorTab::Project],
    );

    dock_state
}

/// Get Unity-like dock style
pub fn get_dock_style() -> Style {
    // Use default style with dark theme
    Style::from_egui(&egui::Style::default())
}

/// Get all .scene files in project scenes folder
fn get_scene_files(project_path: &std::path::Path) -> Vec<String> {
    let scenes_folder = project_path.join("scenes");
    let mut scene_files = Vec::new();

    if scenes_folder.exists() {
        if let Ok(entries) = std::fs::read_dir(&scenes_folder) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(path) = entry.path().to_str() {
                            if path.ends_with(".scene") {
                                if let Ok(relative) = entry.path().strip_prefix(project_path) {
                                    scene_files.push(relative.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    scene_files.sort();
    scene_files
}

/// Get icon for entity based on its components
fn get_entity_icon(world: &World, entity: Entity) -> &'static str {
    use ecs::EntityTag;
    
    if let Some(tag) = world.tags.get(&entity) {
        return match tag {
            EntityTag::Player => "🎮",
            EntityTag::Item => "💎",
        };
    }

    let has_sprite = world.sprites.contains_key(&entity);
    let has_collider = world.colliders.contains_key(&entity);
    let has_velocity = world.velocities.contains_key(&entity);
    let has_script = world.scripts.contains_key(&entity);

    if has_script {
        "📜"
    } else if has_velocity && has_collider {
        "🏃"
    } else if has_sprite && has_collider {
        "📦"
    } else if has_sprite {
        "🖼️"
    } else if has_collider {
        "⬜"
    } else {
        "📍"
    }
}

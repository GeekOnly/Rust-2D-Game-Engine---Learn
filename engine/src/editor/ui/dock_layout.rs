use egui_dock::{DockState, NodeIndex, Style, TabViewer};
use ecs::{World, Entity};
use egui;
use std::collections::HashMap;
use crate::editor::{Console, SceneCamera, SceneGrid, AssetManager, DragDropState};
use super::{TransformTool, hierarchy, inspector, scene_view, asset_browser, texture_inspector};

/// Tab types for the docking system
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EditorTab {
    Hierarchy,
    Inspector,
    Scene,
    Game,
    Console,
    Project,
    MapView,  // LDtk map management panel
    LayerProperties,  // Layer properties panel for tilemap layers
    LayerOrdering,  // Layer ordering panel for reordering tilemap layers
    Performance,  // Performance monitoring panel for tilemap management
    SpriteEditor(std::path::PathBuf),  // Sprite editor for a specific texture file
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
    pub map_view_state: &'a mut super::map_view::MapViewState,
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
    pub dragging_entity: &'a mut Option<Entity>,
    pub drag_axis: &'a mut Option<u8>,
    pub scene_view_mode: &'a mut scene_view::SceneViewMode,
    pub projection_mode: &'a mut scene_view::ProjectionMode,
    pub transform_space: &'a mut scene_view::TransformSpace,
    pub texture_manager: &'a mut crate::texture_manager::TextureManager,
    pub open_sprite_editor_request: &'a mut Option<std::path::PathBuf>,
    pub sprite_editor_windows: &'a mut Vec<crate::editor::SpriteEditorWindow>,
    pub sprite_picker_state: &'a mut super::sprite_picker::SpritePickerState,
    pub texture_inspector: &'a mut texture_inspector::TextureInspector,
    pub show_debug_lines: &'a mut bool,
    pub debug_draw: &'a mut crate::editor::debug_draw::DebugDrawManager,
    pub map_manager: &'a mut crate::editor::map_manager::MapManager,
    pub layer_properties_panel: &'a mut super::layer_properties_panel::LayerPropertiesPanel,
    pub layer_ordering_panel: &'a mut super::layer_ordering_panel::LayerOrderingPanel,
    pub performance_panel: &'a mut super::performance_panel::PerformancePanel,
    pub dt: f32,
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
            EditorTab::MapView => "🗺️ Maps".into(),
            EditorTab::LayerProperties => "🎨 Layer Properties".into(),
            EditorTab::LayerOrdering => "📑 Layer Ordering".into(),
            EditorTab::Performance => "📊 Performance".into(),
            EditorTab::SpriteEditor(path) => {
                let file_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Sprite Editor");
                format!("🖼️ {}", file_name).into()
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Hierarchy => {
                hierarchy::render_hierarchy_with_filter(
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
                    Some(self.context.map_manager), // Pass map_manager to filter map entities
                );
            }
            EditorTab::Inspector => {
                // Show entity inspector or texture inspector based on selection
                if self.context.texture_inspector.selected_texture.is_some() {
                    // Show texture import settings
                    if let Some(action) = self.context.texture_inspector.render(ui) {
                        match action {
                            texture_inspector::TextureInspectorAction::Apply => {
                                // Save settings to .meta file
                                if let Some(ref path) = self.context.texture_inspector.selected_texture {
                                    if let Err(e) = self.context.texture_inspector.settings.save(path) {
                                        self.context.console.error(format!("Failed to save texture settings: {}", e));
                                    } else {
                                        self.context.console.info("Texture settings saved");
                                        self.context.texture_inspector.has_changes = false;
                                    }
                                }
                            }
                            texture_inspector::TextureInspectorAction::Revert => {
                                // Reload settings from .meta file
                                if let Some(ref path) = self.context.texture_inspector.selected_texture {
                                    self.context.texture_inspector.settings = 
                                        crate::editor::texture_import_settings::TextureImportSettings::load(path)
                                            .unwrap_or_default();
                                    self.context.texture_inspector.has_changes = false;
                                    self.context.console.info("Texture settings reverted");
                                }
                            }
                        }
                    }
                } else {
                    // Show entity inspector
                    inspector::render_inspector(
                        ui,
                        self.context.world,
                        self.context.selected_entity,
                        self.context.entity_names,
                        self.context.edit_script_request,
                        self.context.project_path,
                        self.context.open_sprite_editor_request,
                        self.context.sprite_picker_state,
                    );
                }
            }
            EditorTab::Scene => {
                // Scene view - editor view with gizmos and grid
                scene_view::render_scene_view(
                    ui,
                    self.context.world,
                    self.context.selected_entity,
                    self.context.scene_view_tab,
                    self.context.is_playing,
                    self.context.show_colliders,
                    self.context.show_velocities,
                    self.context.show_debug_lines,
                    self.context.debug_draw,
                    self.context.current_tool,
                    self.context.scene_camera,
                    self.context.scene_grid,
                    self.context.play_request,
                    self.context.stop_request,
                    self.context.dragging_entity,
                    self.context.drag_axis,
                    self.context.scene_view_mode,
                    self.context.projection_mode,
                    self.context.transform_space,
                    self.context.texture_manager,
                    self.context.drag_drop,
                );
            }
            EditorTab::Game => {
                // Game view - camera view only (what player sees)
                crate::runtime::render_game_view(ui, self.context.world, self.context.texture_manager);
            }
            EditorTab::Console => {
                // Render console with full functionality
                self.context.console.render(ui);
            }
            EditorTab::MapView => {
                // Render maps panel with MapManager
                super::maps_panel::render_maps_panel_content(
                    ui,
                    self.context.map_manager,
                    self.context.world,
                );
            }
            EditorTab::LayerProperties => {
                // Render layer properties panel
                self.context.layer_properties_panel.render_content(
                    ui,
                    self.context.world,
                    self.context.map_manager,
                );
            }
            EditorTab::LayerOrdering => {
                // Render layer ordering panel
                self.context.layer_ordering_panel.render_content(
                    ui,
                    self.context.world,
                    self.context.map_manager,
                );
            }
            EditorTab::Performance => {
                // Render performance panel
                self.context.performance_panel.render_content(
                    ui,
                    self.context.world,
                    self.context.map_manager,
                );
            }
            EditorTab::Project => {
                if let Some(ref mut manager) = self.context.asset_manager {
                    if let Some(action) = asset_browser::AssetBrowser::render(
                        ui,
                        manager,
                        self.context.drag_drop,
                        self.context.texture_manager,
                        self.context.project_path.as_ref(),
                    ) {
                        match action {
                            asset_browser::AssetBrowserAction::OpenSpriteEditor(path) => {
                                *self.context.open_sprite_editor_request = Some(path);
                            }
                            asset_browser::AssetBrowserAction::SelectTexture(path) => {
                                self.context.texture_inspector.set_texture(path);
                            }
                        }
                    }
                } else {
                    ui.label("No project open");
                }
            }
            EditorTab::SpriteEditor(texture_path) => {
                // Find or create sprite editor window for this texture
                let window_idx = self.context.sprite_editor_windows.iter()
                    .position(|w| w.state.texture_path == *texture_path);

                if let Some(idx) = window_idx {
                    // Render inline without window frame
                    if let Some(window) = self.context.sprite_editor_windows.get_mut(idx) {
                        window.render_inline(ui, self.context.texture_manager, self.context.dt);
                    }
                } else {
                    // Create new sprite editor if not found
                    let mut window = crate::editor::SpriteEditorWindow::new(texture_path.clone());
                    window.render_inline(ui, self.context.texture_manager, self.context.dt);
                    self.context.sprite_editor_windows.push(window);
                }
            }
        }
    }
}

/// Create default Unity-like dock layout
pub fn create_default_layout() -> DockState<EditorTab> {
    // Try to load from embedded default layout file
    const DEFAULT_LAYOUT_JSON: &str = include_str!("default_layout.json");
    
    if let Ok(dock_state) = serde_json::from_str::<DockState<EditorTab>>(DEFAULT_LAYOUT_JSON) {
        return dock_state;
    }
    
    // Fallback to programmatic layout if JSON fails
    let mut dock_state = DockState::new(vec![EditorTab::Scene]);

    // Split to create left panel (Hierarchy)
    let [_left, main] = dock_state.main_surface_mut().split_left(
        NodeIndex::root(),
        0.22,
        vec![EditorTab::Hierarchy],
    );

    // Split to create right panel (Inspector)
    let [center, _right] = dock_state.main_surface_mut().split_right(
        main,
        0.23,
        vec![EditorTab::Inspector],
    );

    // Split center vertically: Scene (top) and bottom area
    let [_scene, bottom_area] = dock_state.main_surface_mut().split_below(
        center,
        0.7,
        vec![EditorTab::Game],
    );

    // Split bottom area: Console/Project (left) and Game (right)
    let [_console, _game] = dock_state.main_surface_mut().split_right(
        bottom_area,
        0.5,
        vec![EditorTab::Console, EditorTab::Project],
    );

    dock_state
}

/// Create alternative layouts
pub fn create_2_column_layout() -> DockState<EditorTab> {
    // 2 columns: Left (Hierarchy + Console) | Right (Scene + Inspector)
    let mut dock_state = DockState::new(vec![EditorTab::Hierarchy, EditorTab::Console]);

    let [_left, right] = dock_state.main_surface_mut().split_right(
        NodeIndex::root(),
        0.7,
        vec![EditorTab::Scene, EditorTab::Game],
    );

    let [_scene, _inspector] = dock_state.main_surface_mut().split_below(
        right,
        0.7,
        vec![EditorTab::Inspector],
    );

    dock_state
}

pub fn create_tall_layout() -> DockState<EditorTab> {
    // Tall layout: Hierarchy | Scene/Game | Inspector with Console at bottom
    let mut dock_state = DockState::new(vec![EditorTab::Scene, EditorTab::Game]);

    let [_left, main] = dock_state.main_surface_mut().split_left(
        NodeIndex::root(),
        0.2,
        vec![EditorTab::Hierarchy],
    );

    let [center, _right] = dock_state.main_surface_mut().split_right(
        main,
        0.25,
        vec![EditorTab::Inspector],
    );

    let [_top, _bottom] = dock_state.main_surface_mut().split_below(
        center,
        0.75,
        vec![EditorTab::Console, EditorTab::Project],
    );

    dock_state
}

pub fn create_wide_layout() -> DockState<EditorTab> {
    // Wide layout: Everything in one row
    let mut dock_state = DockState::new(vec![EditorTab::Hierarchy]);

    let [_left, main] = dock_state.main_surface_mut().split_right(
        NodeIndex::root(),
        0.6,
        vec![EditorTab::Scene, EditorTab::Game],
    );

    let [center, _right] = dock_state.main_surface_mut().split_right(
        main,
        0.5,
        vec![EditorTab::Inspector],
    );

    let [_top, _bottom] = dock_state.main_surface_mut().split_below(
        center,
        0.7,
        vec![EditorTab::Console, EditorTab::Project],
    );

    dock_state
}

/// Save current layout name as default
pub fn save_default_layout(layout_name: &str, project_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let layout_path = project_path.join(".editor_layout.txt");
    std::fs::write(layout_path, layout_name)?;
    Ok(())
}

/// Load default layout name
pub fn load_default_layout_name(project_path: &std::path::Path) -> Option<String> {
    let layout_path = project_path.join(".editor_layout.txt");
    if layout_path.exists() {
        std::fs::read_to_string(layout_path).ok()
    } else {
        None
    }
}

/// Save custom layout configuration with full state
pub fn save_custom_layout_state(
    name: &str,
    dock_state: &DockState<EditorTab>,
    project_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = project_path.join(".editor_layouts");
    std::fs::create_dir_all(&layouts_dir)?;

    let layout_file = layouts_dir.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(dock_state)?;
    std::fs::write(layout_file, json)?;
    Ok(())
}

/// Load custom layout state
pub fn load_custom_layout_state(
    name: &str,
    project_path: &std::path::Path,
) -> Option<DockState<EditorTab>> {
    let layouts_dir = project_path.join(".editor_layouts");
    let layout_file = layouts_dir.join(format!("{}.json", name));

    if layout_file.exists() {
        if let Ok(json) = std::fs::read_to_string(layout_file) {
            if let Ok(dock_state) = serde_json::from_str(&json) {
                return Some(dock_state);
            }
        }
    }
    None
}

/// Save custom layout configuration (legacy - for compatibility)
pub fn save_custom_layout(
    name: &str,
    layout_type: &str,
    project_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = project_path.join(".editor_layouts");
    std::fs::create_dir_all(&layouts_dir)?;

    let layout_file = layouts_dir.join(format!("{}.txt", name));
    std::fs::write(layout_file, layout_type)?;
    Ok(())
}

/// Load custom layouts list
pub fn load_custom_layouts(project_path: &std::path::Path) -> Vec<(String, String)> {
    let layouts_dir = project_path.join(".editor_layouts");
    let mut layouts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(layouts_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                // Support both .json (new) and .txt (legacy) formats
                if name.ends_with(".json") {
                    let layout_name = name.trim_end_matches(".json").to_string();
                    layouts.push((layout_name, "custom".to_string()));
                } else if name.ends_with(".txt") {
                    let layout_name = name.trim_end_matches(".txt").to_string();
                    if let Ok(layout_type) = std::fs::read_to_string(entry.path()) {
                        layouts.push((layout_name, layout_type));
                    }
                }
            }
        }
    }

    layouts.sort_by(|a, b| a.0.cmp(&b.0));
    layouts
}

/// Get layout by name
pub fn get_layout_by_name(name: &str) -> DockState<EditorTab> {
    match name {
        "default" => create_default_layout(),
        "2column" => create_2_column_layout(),
        "tall" => create_tall_layout(),
        "wide" => create_wide_layout(),
        _ => create_default_layout(),
    }
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
    let has_camera = world.cameras.contains_key(&entity);

    if has_camera {
        "📷"
    } else if has_script {
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

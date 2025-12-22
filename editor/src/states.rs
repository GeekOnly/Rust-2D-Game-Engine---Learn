use ecs::{World, Entity};
use engine_core::project::ProjectManager;
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use std::sync::mpsc::Receiver;
use pollster;

/// Application state machine
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum AppState {
    Launcher,
    Editor,
    Playing,
}

/// Editor actions (for handling user commands)
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    NewScene,
    LoadScene(Option<std::path::PathBuf>), // None = Browse, Some = Direct load
    Quit,
    ExportGame,
    DeleteEntity(ecs::Entity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildParams {
    pub output_path: PathBuf,
    pub game_name: String,
    pub target_platform: String,
    pub is_building: bool,
    pub build_output: String,
    pub build_error: Option<String>,
}

impl Default for BuildParams {
    fn default() -> Self {
        Self {
            output_path: std::env::current_dir().unwrap_or_default().join("build"),
            game_name: "RustGame".to_string(),
            target_platform: "windows".to_string(),
            is_building: false,
            build_output: String::new(),
            build_error: None,
        }
    }
}

/// Launcher state - Project selection and creation
#[allow(dead_code)]
pub struct LauncherState {
    pub project_manager: ProjectManager,
    pub new_project_name: String,
    pub new_project_desc: String,
    pub show_new_project_dialog: bool,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
impl LauncherState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            project_manager: ProjectManager::new()?,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            show_new_project_dialog: false,
            error_message: None,
        })
    }
}

/// Editor state - Scene editing and play-in-editor
#[allow(dead_code)]
pub struct EditorState {
    pub world: World,
    pub selected_entity: Option<Entity>,
    pub entity_names: HashMap<Entity, String>,
    pub current_scene_path: Option<PathBuf>,
    pub current_project_path: Option<PathBuf>,
    pub scene_modified: bool,
    pub show_save_required_dialog: bool,
    pub scene_view_tab: usize,
    pub is_playing: bool,
    pub play_world: Option<World>,
    pub keyboard_state: HashMap<String, bool>,
    pub input_system: input::InputSystem,
    pub show_colliders: bool,
    pub show_velocities: bool,
    pub show_debug_lines: bool,  // Show debug draw lines (raycasts, etc.)
    pub console: super::console::Console,
    pub bottom_panel_tab: usize,
    pub show_project_settings: bool,
    pub show_unsaved_changes_dialog: bool,
    pub pending_action: Option<EditorAction>,
    pub asset_browser_path: Option<PathBuf>,
    pub current_tool: super::ui::TransformTool,
    pub resource_current_folder: String,
    pub resource_selected_item: Option<PathBuf>,
    pub show_create_menu: bool,
    pub show_rename_dialog: bool,
    pub rename_buffer: String,
    pub show_camera_settings: bool,  // Camera settings dialog
    pub show_export_dialog: bool,    // Export game dialog
    pub build_params: BuildParams,   // Export parameters
    pub build_receiver: Option<Receiver<String>>, // Channel for build updates
    
    // NEW: Unity-like editor features
    pub shortcut_manager: super::shortcuts::ShortcutManager,
    pub scene_camera: super::SceneCamera,
    pub scene_grid: super::grid::SceneGrid,
    pub infinite_grid: super::grid::InfiniteGrid,  // Enhanced infinite grid for 3D mode
    pub camera_state_display: super::ui::camera_settings::CameraStateDisplay,  // Camera state display
    pub selected_entities: Vec<Entity>,  // Multi-selection support
    pub hierarchy_search: String,        // Search filter
    pub autosave: super::autosave::AutoSave,  // Auto-save system
    pub show_exit_dialog: bool,          // Exit confirmation dialog
    pub should_exit: bool,               // Flag to trigger actual exit
    pub asset_manager: Option<super::asset_manager::AssetManager>,  // Asset manager
    pub drag_drop: super::drag_drop::DragDropState,  // Drag & drop state
    pub dock_state: egui_dock::DockState<super::ui::EditorTab>,  // Docking system
    pub use_docking: bool,               // Toggle between old and new layout
    pub layout_request: Option<String>,  // Layout change request
    pub current_layout_name: String,     // Current layout name (display name)
    pub current_layout_type: String,     // Current layout type (base type: default, 2column, tall, wide)
    pub show_save_layout_dialog: bool,   // Show save layout dialog
    pub save_layout_name: String,        // Name for saving layout
    pub dragging_entity: Option<Entity>, // Entity being dragged
    pub drag_axis: Option<u8>,           // Drag axis: 0=X, 1=Y, 2=Both
    pub scene_view_mode: super::ui::scene_view::SceneViewMode, // 2D or 3D mode
    pub projection_mode: super::ui::scene_view::SceneProjectionMode, // Isometric or Perspective
    pub transform_space: super::ui::scene_view::TransformSpace, // Local or World space
    pub texture_manager: engine::texture_manager::TextureManager, // Texture manager for sprites
    pub undo_stack: super::UndoStack,  // Undo/Redo system
    pub selection: super::SelectionManager,  // Multi-selection system
    pub clipboard: super::Clipboard,  // Copy/Paste/Duplicate system
    pub snap_settings: super::tools::snapping::SnapSettings,  // Snap to Grid system
    pub sprite_editor_windows: Vec<super::SpriteEditorWindow>,  // Open sprite editor windows
    pub open_sprite_editor_request: Option<PathBuf>,  // Request to open sprite editor for a texture
    pub open_prefab_editor_request: Option<PathBuf>,  // Request to open prefab editor for a UI prefab
    pub sprite_picker_state: super::ui::sprite_picker::SpritePickerState,  // Sprite picker popup state
    pub texture_inspector: super::ui::texture_inspector::TextureInspector,  // Texture import settings inspector
    pub map_view_state: super::ui::map_view::MapViewState,  // Map view panel state
    pub debug_draw: super::debug_draw::DebugDrawManager,  // Debug draw system (Unity/Unreal style)
    pub map_manager: super::map_manager::MapManager,  // Map manager for LDtk files
    pub prefab_manager: super::prefab::PrefabManager,  // Prefab manager for reusable entity templates
    pub create_prefab_dialog: super::ui::create_prefab_dialog::CreatePrefabDialog,  // Create prefab dialog
    pub layer_properties_panel: super::ui::panels::layer_properties_panel::LayerPropertiesPanel,  // Layer properties panel for tilemap layers
    pub layer_ordering_panel: super::ui::panels::layer_ordering_panel::LayerOrderingPanel,  // Layer ordering panel for reordering tilemap layers
    pub performance_panel: super::ui::panels::performance_panel::PerformancePanel,  // Performance monitoring panel for tilemap management
    pub collider_settings_panel: super::ui::panels::collider_settings_panel::ColliderSettingsPanel,  // Collider configuration panel for tilemap colliders
    pub game_view_settings: engine::runtime::GameViewSettings,  // Game view resolution and display settings
    pub prefab_editor: super::widget_editor::PrefabEditor,  // Visual UI prefab editor (Unity-style)
    pub ui_manager: engine::ui_manager::UIManager,  // New UI system manager
    pub reload_mesh_assets_request: bool,  // Flag to request reloading mesh assets
}

#[allow(dead_code)]
impl EditorState {
    pub fn new() -> Self {
        let mut console = super::console::Console::new();
        // Add initial message to test console
        console.info("🚀 Editor initialized");
        console.debug("Console logging is working!");
        
        Self {
            world: World::new(),
            selected_entity: None,
            entity_names: HashMap::new(),
            current_scene_path: None,
            current_project_path: None,
            scene_modified: false,
            show_save_required_dialog: false,
            scene_view_tab: 0,
            is_playing: false,
            play_world: None,
            keyboard_state: HashMap::new(),
            input_system: input::InputSystem::new(),
            show_colliders: true,
            show_velocities: false,
            show_debug_lines: true,  // Show debug lines by default
            console,
            bottom_panel_tab: 1,  // Default to Console tab to show logs
            show_project_settings: false,
            show_unsaved_changes_dialog: false,
            pending_action: None,
            asset_browser_path: None,
            current_tool: super::ui::TransformTool::View,
            resource_current_folder: String::new(),
            resource_selected_item: None,
            show_create_menu: false,
            show_rename_dialog: false,
            rename_buffer: String::new(),
            show_camera_settings: false,
            show_export_dialog: false,
            build_params: BuildParams::default(),
            build_receiver: None,
            
            // NEW: Initialize Unity-like features
            shortcut_manager: super::shortcuts::ShortcutManager::new(),
            scene_camera: super::SceneCamera::new(),
            scene_grid: super::grid::SceneGrid::new(),
            infinite_grid: super::grid::InfiniteGrid::new(),
            camera_state_display: super::ui::camera_settings::CameraStateDisplay::new(),
            selected_entities: Vec::new(),
            hierarchy_search: String::new(),
            autosave: super::autosave::AutoSave::new(300), // 5 minutes
            show_exit_dialog: false,
            should_exit: false,
            asset_manager: None, // Initialized when project is opened
            drag_drop: super::drag_drop::DragDropState::new(),
            dock_state: super::ui::create_default_layout(),
            use_docking: true, // Use new docking layout by default
            layout_request: None,
            current_layout_name: "default".to_string(),
            current_layout_type: "default".to_string(),
            show_save_layout_dialog: false,
            save_layout_name: String::new(),
            dragging_entity: None,
            drag_axis: None,
            scene_view_mode: super::ui::scene_view::SceneViewMode::Mode2D,
            projection_mode: super::ui::scene_view::SceneProjectionMode::Perspective, // Unity-style default
            transform_space: super::ui::scene_view::TransformSpace::Local,
            undo_stack: super::UndoStack::new(),
            selection: super::SelectionManager::new(),
            clipboard: super::Clipboard::new(),
            snap_settings: super::tools::snapping::SnapSettings::load().unwrap_or_default(),
            texture_manager: engine::texture_manager::TextureManager::new(),
            sprite_editor_windows: Vec::new(),
            open_sprite_editor_request: None,
            open_prefab_editor_request: None,
            sprite_picker_state: super::ui::sprite_picker::SpritePickerState::new(),
            texture_inspector: super::ui::texture_inspector::TextureInspector::default(),
            map_view_state: super::ui::map_view::MapViewState::default(),
            debug_draw: super::debug_draw::DebugDrawManager::new(),
            map_manager: super::map_manager::MapManager::new(),
            prefab_manager: super::prefab::PrefabManager::new(),
            create_prefab_dialog: super::ui::create_prefab_dialog::CreatePrefabDialog::new(),
            layer_properties_panel: super::ui::panels::layer_properties_panel::LayerPropertiesPanel::new(),
            layer_ordering_panel: super::ui::panels::layer_ordering_panel::LayerOrderingPanel::new(),
            performance_panel: super::ui::panels::performance_panel::PerformancePanel::new(),
            collider_settings_panel: super::ui::panels::collider_settings_panel::ColliderSettingsPanel::new(),
            game_view_settings: engine::runtime::GameViewSettings::default(),
            prefab_editor: super::widget_editor::PrefabEditor::new(),
            ui_manager: engine::ui_manager::UIManager::new(),
            reload_mesh_assets_request: false,
        }
    }

    pub fn get_scripts_folder(&self) -> Option<PathBuf> {
        self.current_project_path.as_ref().map(|p| p.join("scripts"))
    }

    /// Set the current project path and update related components
    pub fn set_project_path(&mut self, path: PathBuf) {
        self.current_project_path = Some(path.clone());
        
        // Set texture manager base path to project assets folder
        self.texture_manager.set_base_path(path.join("assets"));
        
        // Update other components that need project path
        self.map_manager.set_project_path(path.clone());
        self.prefab_manager.set_project_path(path.clone());
        self.asset_browser_path = Some(path);
        
        // Request asset reload when project changes
        self.reload_mesh_assets_request = true;
    }

    /// Load editor layout from project folder
    pub fn load_editor_layout(&mut self) {
        if let Some(ref project_path) = self.current_project_path {
            if let Some(layout_name) = super::ui::load_default_layout_name(project_path) {
                self.dock_state = super::ui::get_layout_by_name(&layout_name);
                self.current_layout_name = layout_name.clone();
                self.console.info(format!("Loaded layout: {}", layout_name));
            }
        }
    }
    /// Save current layout as default
    pub fn save_default_layout(&self) {
        if let Some(ref project_path) = self.current_project_path {
            if let Err(e) = super::ui::save_default_layout(&self.current_layout_name, project_path) {
                eprintln!("Failed to save default layout: {}", e);
            }
        }
    }

    /// Execute editor action
    pub fn execute_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::DeleteEntity(entity) => {
                if let Some(selected) = self.selected_entity {
                    if selected == entity {
                        self.selected_entity = None;
                    }
                }
                self.world.despawn(entity);
                self.console.info(format!("Deleted entity {:?}", entity));
            }
            EditorAction::NewScene => {
                self.world = ecs::World::new();
                self.selected_entity = None;
                self.entity_names.clear();
                self.current_scene_path = None;
                self.console.info("Created new scene".to_string());
            }
            EditorAction::LoadScene(path) => {
                if let Some(path) = path {
                    // TODO: Implement scene loading
                    self.console.info(format!("Loading scene: {:?}", path));
                }
            }
            EditorAction::Quit => {
                self.should_exit = true;
            }
            EditorAction::ExportGame => {
                // TODO: Implement game export
                self.console.info("Exporting game...".to_string());
            }
        }
    }

    pub fn get_default_scene_path(&self, name: &str) -> Option<PathBuf> {
        self.current_project_path.as_ref().map(|p| {
            let scenes_dir = p.join("scenes");
            std::fs::create_dir_all(&scenes_dir).ok();
            scenes_dir.join(format!("{}.json", name))
        })
    }

    pub fn save_scene(&mut self, path: &PathBuf) -> Result<()> {
        // Sync entity_names to world.names before saving
        for (entity, name) in &self.entity_names {
            self.world.names.insert(*entity, name.clone());
        }
        
        let json = self.world.save_to_json()?;
        std::fs::write(path, json)?;
        self.current_scene_path = Some(path.clone());
        self.scene_modified = false;
        
        // Update last_opened_scene in project config
        if let Some(project_path) = &self.current_project_path {
            if let Ok(pm) = ProjectManager::new() {
                // Make path relative to project
                if let Ok(relative_path) = path.strip_prefix(project_path) {
                    let _ = pm.set_last_opened_scene(project_path, Some(relative_path.to_path_buf()));
                }
            }
        }
        
        log::info!("Scene saved to {:?}", path);
        Ok(())
    }

    pub fn load_scene(&mut self, path: &PathBuf, asset_loader: &dyn engine_core::assets::AssetLoader) -> Result<()> {
        let path_str = path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        
        // Use block_on to execute the async load synchronously for now
        // This is safe on native because NativeAssetLoader uses blocking fs calls inside async wrapper (conceptually)
        // For WASM, this will need to be refactored to be truly async
        let json = pollster::block_on(asset_loader.load_text(path_str))?;
        
        self.world.load_from_json(&json)?;
        self.current_scene_path = Some(path.clone());
        self.scene_modified = false;
        self.selected_entity = None;
        


        // Rebuild entity_names from loaded entities
        self.entity_names.clear();
        for &entity in self.world.transforms.keys() {
            // Use name from world if available, otherwise generate one
            let name = if let Some(name) = self.world.names.get(&entity) {
                name.clone()
            } else if let Some(tag) = self.world.tags.get(&entity) {
                format!("{:?}", tag)
            } else {
                format!("Entity {}", entity)
            };
            self.entity_names.insert(entity, name.clone());
            self.world.names.insert(entity, name);
        }

        // Update last_opened_scene in project config
        if let Some(project_path) = &self.current_project_path {
            if let Ok(pm) = ProjectManager::new() {
                // Make path relative to project
                if let Ok(relative_path) = path.strip_prefix(project_path) {
                    let _ = pm.set_last_opened_scene(project_path, Some(relative_path.to_path_buf()));
                }
            }
        }

        log::info!("Scene loaded from {:?}", path);
        Ok(())
    }

    pub fn create_script_file(&self, script_name: &str) -> Result<PathBuf> {
        if let Some(scripts_folder) = self.get_scripts_folder() {
            std::fs::create_dir_all(&scripts_folder)?;
            let script_path = scripts_folder.join(format!("{}.lua", script_name));

            if !script_path.exists() {
                let template = format!(
r#"-- Script: {}
-- Simple player movement script

-- Engine API Functions (provided by the game engine):
-- is_key_pressed(key) - Check if a key is pressed
-- set_velocity(vx, vy) - Set entity velocity
-- get_tag(entity) - Get entity tag
-- destroy_entity(entity) - Destroy an entity

-- Movement speed
local speed = 200.0

function on_start()
    -- Called when the game starts
    print("Script {} started!")
end

function on_update(dt)
    -- Called every frame (dt is delta time in seconds)

    -- Player movement (WASD or Arrow keys)
    local vx = 0.0
    local vy = 0.0

    if is_key_pressed("W") or is_key_pressed("Up") then
        vy = vy - speed
    end
    if is_key_pressed("S") or is_key_pressed("Down") then
        vy = vy + speed
    end
    if is_key_pressed("A") or is_key_pressed("Left") then
        vx = vx - speed
    end
    if is_key_pressed("D") or is_key_pressed("Right") then
        vx = vx + speed
    end

    -- Normalize diagonal movement
    if vx ~= 0.0 and vy ~= 0.0 then
        local length = math.sqrt(vx * vx + vy * vy)
        vx = vx / length * speed
        vy = vy / length * speed
    end

    -- Set velocity
    set_velocity(vx, vy)
end

function on_collision(other_entity)
    -- Called when this entity collides with another
    print("Collision with entity: " .. tostring(other_entity))

    -- Example: Collect item
    local tag = get_tag(other_entity)
    if tag == "Item" then
        print("Collected item!")
        destroy_entity(other_entity)
    end
end
"#,
                    script_name, script_name
                );

                std::fs::write(&script_path, template)?;
                log::info!("Created script file: {:?}", script_path);
            }

            Ok(script_path)
        } else {
            Err(anyhow::anyhow!("No project open"))
        }
    }

    pub fn open_script_in_editor(&self, script_name: &str) -> Result<()> {
        let script_path = self.create_script_file(script_name)?;

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", script_path.to_str().unwrap()])
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&script_path)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&script_path)
                .spawn()?;
        }

        Ok(())
    }

    /// Update all entities that use a specific sprite file when it changes
    pub fn update_entities_using_sprite_file(&mut self, sprite_file_path: &PathBuf) {
        // Load the updated sprite metadata
        let metadata = match sprite_editor::SpriteMetadata::load(sprite_file_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to load sprite metadata for hot-reload: {}", e);
                return;
            }
        };

        // Find all entities that use this sprite file
        let mut updated_count = 0;
        
        // Update SpriteSheet components
        for (entity, sprite_sheet) in self.world.sprite_sheets.iter_mut() {
            // Check if this sprite sheet uses the updated sprite file
            if sprite_sheet.texture_path == metadata.texture_path {
                // Update the frames from the new metadata
                sprite_sheet.frames.clear();
                for sprite_def in &metadata.sprites {
                    sprite_sheet.frames.push(ecs::SpriteFrame {
                        x: sprite_def.x,
                        y: sprite_def.y,
                        width: sprite_def.width,
                        height: sprite_def.height,
                        name: Some(sprite_def.name.clone()),
                    });
                }
                
                sprite_sheet.sheet_width = metadata.texture_width;
                sprite_sheet.sheet_height = metadata.texture_height;
                
                updated_count += 1;
                log::info!("Updated sprite sheet for entity {} with {} frames", entity, sprite_sheet.frames.len());
            }
        }

        if updated_count > 0 {
            self.console.info(format!("🔄 Hot-reloaded sprite file: {} entities updated", updated_count));
            log::info!("Hot-reloaded sprite file {:?}: {} entities updated", sprite_file_path, updated_count);
        }
    }
}

/// Game state - Full-screen game mode (not used with play-in-editor)
#[allow(dead_code)]
pub struct GameState {
    pub world: World,
    pub player_speed: f32,
    pub input_state: HashMap<String, bool>,
}

#[allow(dead_code)]
impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        let input_state = HashMap::new();

        // Initialize player
        let player = world.spawn();
        world.transforms.insert(player, ecs::Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [40.0, 40.0, 1.0], // Use scale for sprite size
        });
        world.velocities.insert(player, (0.0, 0.0));
        world.sprites.insert(player, ecs::Sprite {
            texture_id: "player".to_string(),
            width: 1.0,  // Base size
            height: 1.0,
            color: [0.2, 0.6, 1.0, 1.0],
            billboard: true, // Player sprite faces camera
            ..Default::default()
        });

        Self {
            world,
            player_speed: 200.0,
            input_state,
        }
    }

    #[allow(dead_code)]
    pub fn handle_input(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        let key_str = format!("{:?}", key);
        self.input_state.insert(key_str, pressed);
    }

    #[allow(dead_code)]
    pub fn update(&mut self, dt: f32, physics: &mut physics::PhysicsWorld) {
        // Update based on input (simple WASD movement)
        let mut vx = 0.0;
        let mut vy = 0.0;

        if self.input_state.get("KeyW").copied().unwrap_or(false) {
            vy -= self.player_speed;
        }
        if self.input_state.get("KeyS").copied().unwrap_or(false) {
            vy += self.player_speed;
        }
        if self.input_state.get("KeyA").copied().unwrap_or(false) {
            vx -= self.player_speed;
        }
        if self.input_state.get("KeyD").copied().unwrap_or(false) {
            vx += self.player_speed;
        }

        // Update velocities
        for velocity in self.world.velocities.values_mut() {
            velocity.0 = vx;
            velocity.1 = vy;
        }

        // Update physics
        physics.step(dt, &mut self.world);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_update_entities_using_sprite_file() {
        let mut editor_state = EditorState::new();
        
        // Create a temporary sprite file
        let temp_dir = std::env::temp_dir();
        let sprite_file_path = temp_dir.join("test_sprite.sprite");
        let texture_path = "assets/test_texture.png";
        
        // Create initial sprite metadata
        let metadata = sprite_editor::SpriteMetadata {
            texture_path: texture_path.to_string(),
            texture_width: 256,
            texture_height: 256,
            sprites: vec![
                sprite_editor::SpriteDefinition {
                    name: "sprite_0".to_string(),
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                },
                sprite_editor::SpriteDefinition {
                    name: "sprite_1".to_string(),
                    x: 32,
                    y: 0,
                    width: 32,
                    height: 32,
                },
            ],
        };
        
        // Save the sprite file
        let json = serde_json::to_string_pretty(&metadata).unwrap();
        fs::write(&sprite_file_path, json).unwrap();
        
        // Create an entity with a sprite sheet that uses this texture
        let entity = editor_state.world.spawn();
        let sprite_sheet = ecs::SpriteSheet {
            texture_path: texture_path.to_string(),
            texture_id: "test_texture".to_string(),
            sheet_width: 256,
            sheet_height: 256,
            frames: vec![
                ecs::SpriteFrame {
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                    name: Some("old_sprite_0".to_string()),
                },
            ],
        };
        editor_state.world.sprite_sheets.insert(entity, sprite_sheet);
        
        // Verify initial state
        assert_eq!(editor_state.world.sprite_sheets.get(&entity).unwrap().frames.len(), 1);
        assert_eq!(editor_state.world.sprite_sheets.get(&entity).unwrap().frames[0].name.as_ref().unwrap(), "old_sprite_0");
        
        // Update entities using the sprite file
        editor_state.update_entities_using_sprite_file(&sprite_file_path);
        
        // Verify the sprite sheet was updated
        let updated_sprite_sheet = editor_state.world.sprite_sheets.get(&entity).unwrap();
        assert_eq!(updated_sprite_sheet.frames.len(), 2);
        assert_eq!(updated_sprite_sheet.frames[0].name.as_ref().unwrap(), "sprite_0");
        assert_eq!(updated_sprite_sheet.frames[1].name.as_ref().unwrap(), "sprite_1");
        assert_eq!(updated_sprite_sheet.frames[0].x, 0);
        assert_eq!(updated_sprite_sheet.frames[0].y, 0);
        assert_eq!(updated_sprite_sheet.frames[0].width, 32);
        assert_eq!(updated_sprite_sheet.frames[0].height, 32);
        
        // Clean up
        fs::remove_file(&sprite_file_path).ok();
    }
    
    #[test]
    fn test_update_entities_no_matching_texture() {
        let mut editor_state = EditorState::new();
        
        // Create a temporary sprite file
        let temp_dir = std::env::temp_dir();
        let sprite_file_path = temp_dir.join("test_sprite2.sprite");
        
        // Create sprite metadata with a different texture path
        let metadata = sprite_editor::SpriteMetadata {
            texture_path: "assets/different_texture.png".to_string(),
            texture_width: 256,
            texture_height: 256,
            sprites: vec![
                sprite_editor::SpriteDefinition {
                    name: "sprite_0".to_string(),
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                },
            ],
        };
        
        // Save the sprite file
        let json = serde_json::to_string_pretty(&metadata).unwrap();
        fs::write(&sprite_file_path, json).unwrap();
        
        // Create an entity with a sprite sheet that uses a different texture
        let entity = editor_state.world.spawn();
        let sprite_sheet = ecs::SpriteSheet {
            texture_path: "assets/test_texture.png".to_string(),
            texture_id: "test_texture".to_string(),
            sheet_width: 256,
            sheet_height: 256,
            frames: vec![
                ecs::SpriteFrame {
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                    name: Some("original_sprite".to_string()),
                },
            ],
        };
        editor_state.world.sprite_sheets.insert(entity, sprite_sheet);
        
        // Verify initial state
        assert_eq!(editor_state.world.sprite_sheets.get(&entity).unwrap().frames.len(), 1);
        assert_eq!(editor_state.world.sprite_sheets.get(&entity).unwrap().frames[0].name.as_ref().unwrap(), "original_sprite");
        
        // Update entities using the sprite file (should not affect this entity)
        editor_state.update_entities_using_sprite_file(&sprite_file_path);
        
        // Verify the sprite sheet was NOT updated
        let sprite_sheet = editor_state.world.sprite_sheets.get(&entity).unwrap();
        assert_eq!(sprite_sheet.frames.len(), 1);
        assert_eq!(sprite_sheet.frames[0].name.as_ref().unwrap(), "original_sprite");
        
        // Clean up
        fs::remove_file(&sprite_file_path).ok();
    }
}

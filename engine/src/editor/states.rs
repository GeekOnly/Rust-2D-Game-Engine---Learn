use ecs::{World, Entity};
use engine_core::project::ProjectManager;
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

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
    
    // NEW: Unity-like editor features
    pub shortcut_manager: super::shortcuts::ShortcutManager,
    pub scene_camera: super::camera::SceneCamera,
    pub scene_grid: super::grid::SceneGrid,
    pub selected_entities: Vec<Entity>,  // Multi-selection support
    pub hierarchy_search: String,        // Search filter
    pub autosave: super::autosave::AutoSave,  // Auto-save system
    pub show_exit_dialog: bool,          // Exit confirmation dialog
}

#[allow(dead_code)]
impl EditorState {
    pub fn new() -> Self {
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
            console: super::console::Console::new(),
            bottom_panel_tab: 0,
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
            
            // NEW: Initialize Unity-like features
            shortcut_manager: super::shortcuts::ShortcutManager::new(),
            scene_camera: super::camera::SceneCamera::new(),
            scene_grid: super::grid::SceneGrid::new(),
            selected_entities: Vec::new(),
            hierarchy_search: String::new(),
            autosave: super::autosave::AutoSave::new(300), // 5 minutes
            show_exit_dialog: false,
        }
    }

    pub fn get_scripts_folder(&self) -> Option<PathBuf> {
        self.current_project_path.as_ref().map(|p| p.join("scripts"))
    }

    pub fn get_default_scene_path(&self, name: &str) -> Option<PathBuf> {
        self.current_project_path.as_ref().map(|p| {
            let scenes_dir = p.join("scenes");
            std::fs::create_dir_all(&scenes_dir).ok();
            scenes_dir.join(format!("{}.json", name))
        })
    }

    pub fn save_scene(&mut self, path: &PathBuf) -> Result<()> {
        let json = self.world.save_to_json()?;
        std::fs::write(path, json)?;
        self.current_scene_path = Some(path.clone());
        self.scene_modified = false;
        log::info!("Scene saved to {:?}", path);
        Ok(())
    }

    pub fn load_scene(&mut self, path: &PathBuf) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.world.load_from_json(&json)?;
        self.current_scene_path = Some(path.clone());
        self.scene_modified = false;
        self.selected_entity = None;

        // Rebuild entity_names from loaded entities
        self.entity_names.clear();
        for &entity in self.world.transforms.keys() {
            let name = if let Some(tag) = self.world.tags.get(&entity) {
                format!("{:?}", tag)
            } else {
                format!("Entity {}", entity)
            };
            self.entity_names.insert(entity, name);
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
            scale: [1.0, 1.0, 1.0],
        });
        world.velocities.insert(player, (0.0, 0.0));
        world.sprites.insert(player, ecs::Sprite {
            texture_id: "player".to_string(),
            width: 40.0,
            height: 40.0,
            color: [0.2, 0.6, 1.0, 1.0],
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

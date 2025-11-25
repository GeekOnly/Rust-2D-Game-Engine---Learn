mod editor_ui;
mod states;
mod script_loader;
mod console;

use anyhow::Result;
use engine_core::{EngineContext, EngineModule, project::ProjectManager};
use ecs::{World, Entity, Transform, Sprite, Collider, EntityTag};
use script::ScriptEngine;
use physics::PhysicsWorld;
use render::RenderModule;
use editor::EditorModule;
use editor_ui::{EditorUI, TransformTool};
use input::{Key, MouseButton};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Moved to states module
// pub use states::{AppState, LauncherState, EditorState, GameState};

// Temporary: Keep local definitions until full migration
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Launcher,
    Editor,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
enum EditorAction {
    NewScene,
    LoadScene(Option<std::path::PathBuf>), // None = Browse, Some = Direct load
    Quit,
}

struct LauncherState {
    project_manager: ProjectManager,
    new_project_name: String,
    new_project_desc: String,
    show_new_project_dialog: bool,
    error_message: Option<String>,
}

impl LauncherState {
    fn new() -> Result<Self> {
        Ok(Self {
            project_manager: ProjectManager::new()?,
            new_project_name: String::new(),
            new_project_desc: String::new(),
            show_new_project_dialog: false,
            error_message: None,
        })
    }
}

struct EditorState {
    world: World,
    selected_entity: Option<Entity>,
    entity_names: std::collections::HashMap<Entity, String>,
    current_scene_path: Option<std::path::PathBuf>,
    current_project_path: Option<std::path::PathBuf>,
    scene_modified: bool,
    show_save_required_dialog: bool,
    scene_view_tab: usize,
    is_playing: bool,
    play_world: Option<World>, // Backup world for play mode
    keyboard_state: std::collections::HashMap<String, bool>,
    input_system: input::InputSystem,  // Unified input system
    show_colliders: bool,  // Gizmo toggle
    show_velocities: bool, // Gizmo toggle
    console: console::Console,
    bottom_panel_tab: usize, // 0 = Assets, 1 = Console
    show_project_settings: bool,
    show_unsaved_changes_dialog: bool,
    pending_action: Option<EditorAction>,
    asset_browser_path: Option<std::path::PathBuf>,
    current_tool: TransformTool, // Q/W/E/R - Transform tool selection
}

impl EditorState {
    fn new() -> Self {
        Self {
            world: World::new(),
            selected_entity: None,
            entity_names: std::collections::HashMap::new(),
            current_scene_path: None,
            current_project_path: None,
            scene_modified: false,
            show_save_required_dialog: false,
            scene_view_tab: 0,
            is_playing: false,
            play_world: None,
            keyboard_state: std::collections::HashMap::new(),
            input_system: input::InputSystem::new(),
            show_colliders: true,   // Show colliders by default
            show_velocities: false, // Hide velocities by default
            console: console::Console::new(),
            bottom_panel_tab: 1,    // Start with Console tab
            show_project_settings: false,
            show_unsaved_changes_dialog: false,
            pending_action: None,
            asset_browser_path: None,
            current_tool: TransformTool::Move, // Default to Move tool (W)
        }
    }

    fn get_scripts_folder(&self) -> Option<std::path::PathBuf> {
        self.current_project_path.as_ref().map(|p| p.join("scripts"))
    }

    fn get_scenes_folder(&self) -> Option<std::path::PathBuf> {
        self.current_project_path.as_ref().map(|p| p.join("scenes"))
    }

    fn get_default_scene_path(&self, scene_name: &str) -> Option<std::path::PathBuf> {
        self.get_scenes_folder().map(|p| p.join(format!("{}.scene", scene_name)))
    }

    fn create_script_file(&self, script_name: &str) -> anyhow::Result<std::path::PathBuf> {
        let scripts_folder = self.get_scripts_folder()
            .ok_or_else(|| anyhow::anyhow!("No project open"))?;

        // Create scripts folder if it doesn't exist
        std::fs::create_dir_all(&scripts_folder)?;

        let script_path = scripts_folder.join(format!("{}.lua", script_name));

        // Create default script content if file doesn't exist
        if !script_path.exists() {
            let default_content = format!(r#"-- Script: {}
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
"#, script_name, script_name);

            std::fs::write(&script_path, default_content)?;
        }

        Ok(script_path)
    }

    fn open_script_in_editor(&self, script_name: &str) -> anyhow::Result<()> {
        let script_path = self.create_script_file(script_name)?;

        // Open in default system editor
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

    fn save_scene(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        // Create scenes folder if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save world to JSON
        let world_json = self.world.save_to_json()?;

        // Save entity names
        let entity_names_json = serde_json::to_string_pretty(&self.entity_names)?;

        // Combine into one scene file
        let scene_data = serde_json::json!({
            "world": serde_json::from_str::<serde_json::Value>(&world_json)?,
            "entity_names": serde_json::from_str::<serde_json::Value>(&entity_names_json)?,
        });

        std::fs::write(path, serde_json::to_string_pretty(&scene_data)?)?;
        self.current_scene_path = Some(path.to_path_buf());
        self.scene_modified = false;
        Ok(())
    }

    fn load_scene(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let scene_json = std::fs::read_to_string(path)?;
        let scene_data: serde_json::Value = serde_json::from_str(&scene_json)?;

        // Load world
        if let Some(world_data) = scene_data.get("world") {
            let world_json = serde_json::to_string(world_data)?;
            self.world.load_from_json(&world_json)?;
        }

        // Load entity names
        if let Some(names_data) = scene_data.get("entity_names") {
            let names_json = serde_json::to_string(names_data)?;
            self.entity_names = serde_json::from_str(&names_json)?;
        }

        self.current_scene_path = Some(path.to_path_buf());
        self.scene_modified = false;
        self.selected_entity = None;
        Ok(())
    }
}

struct GameState {
    world: World,
    player: Option<Entity>,
    items: Vec<Entity>,
    collected_items: usize,
    player_speed: f32,
}

impl GameState {
    fn new() -> Self {
        let mut world = World::new();

        // Spawn player
        let player = world.spawn();
        world.transforms.insert(player, Transform {
            position: [400.0, 300.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world.sprites.insert(player, Sprite {
            texture_id: "player".to_string(),
            width: 40.0,
            height: 40.0,
            color: [0.2, 0.6, 1.0, 1.0], // Blue
        });
        world.colliders.insert(player, Collider {
            width: 40.0,
            height: 40.0,
        });
        world.tags.insert(player, EntityTag::Player);

        // Spawn items
        let mut items = Vec::new();
        let item_positions = [
            (200.0, 150.0),
            (600.0, 150.0),
            (200.0, 450.0),
            (600.0, 450.0),
            (400.0, 100.0),
            (100.0, 300.0),
            (700.0, 300.0),
        ];

        for (x, y) in item_positions.iter() {
            let item = world.spawn();
            world.transforms.insert(item, Transform {
                position: [*x, *y, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            });
            world.sprites.insert(item, Sprite {
                texture_id: "item".to_string(),
                width: 30.0,
                height: 30.0,
                color: [1.0, 0.8, 0.0, 1.0], // Gold
            });
            world.colliders.insert(item, Collider {
                width: 30.0,
                height: 30.0,
            });
            world.tags.insert(item, EntityTag::Item);
            items.push(item);
        }

        Self {
            world,
            player: Some(player),
            items,
            collected_items: 0,
            player_speed: 200.0,
        }
    }

    fn update(&mut self, ctx: &EngineContext, _dt: f32) {
        // Update player velocity based on input
        if let Some(player) = self.player {
            let input = ctx.input.get_movement_input(0); // Player 1
            
            let vx = input.x * self.player_speed;
            let vy = input.y * self.player_speed;

            self.world.velocities.insert(player, (vx, vy));
        }

        // Check collisions with items
        if let Some(player) = self.player {
            let mut items_to_remove = Vec::new();

            for &item in &self.items {
                if PhysicsWorld::check_collision(&self.world, player, item) {
                    items_to_remove.push(item);
                }
            }

            for item in items_to_remove {
                self.world.despawn(item);
                self.items.retain(|&e| e != item);
                self.collected_items += 1;
            }
        }
    }
}

struct SampleModule {
    game_state: GameState,
}

impl EngineModule for SampleModule {
    fn name(&self) -> &str { "sample" }
    fn on_load(&mut self, _ctx: &mut EngineContext) -> Result<()> {
        println!("Sample module loaded!");
        Ok(())
    }
    fn on_update(&mut self, ctx: &mut EngineContext, dt: f32) {
        self.game_state.update(ctx, dt);
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Game Engine...");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rust 2D Game Engine - Launcher")
        .with_inner_size(winit::dpi::LogicalSize::new(1000, 700))
        .build(&event_loop)?;

    let mut app_state = AppState::Launcher;
    let mut launcher_state = LauncherState::new()?;
    let mut editor_state = EditorState::new();

    let mut ctx = EngineContext::new();
    let mut sample_module: Option<SampleModule> = None;

    let mut script_engine = ScriptEngine::new()?;
    let mut physics = PhysicsWorld::new();

    let mut last_frame_time = std::time::Instant::now();

    // Initialize renderer with window
    let mut renderer = pollster::block_on(RenderModule::new(&window))?;
    let _editor = EditorModule::new();

    // egui setup
    let egui_ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        None,
        1,
    );

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // Pass events to egui
                let _ = egui_state.on_window_event(&window, event);

                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                            ..
                        },
                        ..
                    } => target.exit(),
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        // Update InputSystem
                        if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                            let key_str = format!("{:?}", key_code);
                            if let Some(key) = Key::from_str(&key_str) {
                                if key_event.state == ElementState::Pressed {
                                    ctx.input.press_key(key);
                                } else {
                                    ctx.input.release_key(key);
                                }
                            }
                        }

                        // Pass keyboard input to game state only in Playing mode
                        if app_state == AppState::Playing {
                            // Input is now handled via ctx.input in update()
                        } else if app_state == AppState::Editor && editor_state.is_playing {
                            // Track keyboard input in editor play mode
                            if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                                let key_name = format!("{:?}", key_code);
                                let is_pressed = key_event.state == winit::event::ElementState::Pressed;

                                // Update InputSystem for scripts
                                if let Some(key) = Key::from_str(&key_name) {
                                    if is_pressed {
                                        editor_state.input_system.press_key(key);
                                    } else {
                                        editor_state.input_system.release_key(key);
                                    }
                                }

                                // Track in legacy keyboard_state HashMap
                                editor_state.keyboard_state.insert(key_name, is_pressed);
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        ctx.input.set_mouse_position(position.x as f32, position.y as f32);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let mouse_button = match button {
                            winit::event::MouseButton::Left => Some(input::MouseButton::Left),
                            winit::event::MouseButton::Right => Some(input::MouseButton::Right),
                            winit::event::MouseButton::Middle => Some(input::MouseButton::Middle),
                            _ => None,
                        };

                        if let Some(mb) = mouse_button {
                            if *state == ElementState::Pressed {
                                ctx.input.press_mouse_button(mb);
                            } else {
                                ctx.input.release_mouse_button(mb);
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor: _, inner_size_writer: _ } => {
                        // In winit 0.29, ScaleFactorChanged provides inner_size_writer, not new_inner_size directly in the same way?
                        // Actually it's simpler to just handle Resized usually.
                        // But let's check docs if possible. 
                        // For now, let's just ignore ScaleFactorChanged or assume it triggers Resized.
                    }
                    WindowEvent::RedrawRequested => {
                        let dt = 1.0 / 60.0; // Fixed time step for now

                        ctx.input.begin_frame();
                        ctx.input.update_gamepads();

                        // Update based on app state
                        if app_state == AppState::Playing {
                            if let Some(ref mut module) = sample_module {
                                module.on_update(&mut ctx, dt);
                                physics.step(dt, &mut module.game_state.world);
                            }
                        }

                        // Egui frame setup
                        let raw_input = egui_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        // Render UI based on app state
                        match app_state {
                            AppState::Launcher => {
                                // Launcher UI
                                egui::CentralPanel::default().show(&egui_ctx, |ui| {
                                    ui.heading("🎮 Rust 2D Game Engine");
                                    ui.add_space(20.0);

                                    ui.horizontal(|ui| {
                                        if ui.button("➕ New Project").clicked() {
                                            launcher_state.show_new_project_dialog = true;
                                            launcher_state.new_project_name.clear();
                                            launcher_state.new_project_desc.clear();
                                        }

                                        if ui.button("📁 Open Project").clicked() {
                                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                                match launcher_state.project_manager.open_project(&folder) {
                                                    Ok(_) => {
                                                        app_state = AppState::Editor;
                                                        editor_state = EditorState::new();
                                                        editor_state.current_project_path = Some(folder.clone());

                                                        // Log to console
                                                        editor_state.console.info(format!("Project opened: {}", folder.display()));
                                                        editor_state.console.info("Welcome to Rust 2D Game Engine!");

                                                        // Load startup scene if configured
                                                        if let Ok(Some(startup_scene)) = launcher_state.project_manager.get_startup_scene(&folder) {
                                                            let scene_path = folder.join(&startup_scene);
                                                            if scene_path.exists() {
                                                                if let Err(e) = editor_state.load_scene(&scene_path) {
                                                                    editor_state.console.error(format!("Failed to load startup scene: {}", e));
                                                                } else {
                                                                    editor_state.current_scene_path = Some(scene_path.clone());
                                                                    editor_state.console.info(format!("Loaded startup scene: {}", startup_scene.display()));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        launcher_state.error_message = Some(format!("Error: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);

                                    // Example projects section
                                    ui.heading("📦 Example Projects");
                                    ui.add_space(5.0);

                                    for (name, desc) in ProjectManager::get_example_projects() {
                                        ui.group(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.strong(name);
                                                    ui.label(desc);
                                                });
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("Open").clicked() {
                                                        // Create example project
                                                        match launcher_state.project_manager.create_project(name, desc) {
                                                            Ok(metadata) => {
                                                                // Check if this is the Item Collection Game example
                                                                if name == "Item Collection Game" {
                                                                    app_state = AppState::Playing;
                                                                    sample_module = Some(SampleModule {
                                                                        game_state: GameState::new(),
                                                                    });
                                                                } else {
                                                                    // Open empty editor for other examples
                                                                    app_state = AppState::Editor;
                                                                    editor_state = EditorState::new();
                                                                    editor_state.current_project_path = Some(metadata.path);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                launcher_state.error_message = Some(format!("Error: {}", e));
                                                            }
                                                        }
                                                    }
                                                });
                                            });
                                        });
                                        ui.add_space(5.0);
                                    }

                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);

                                    // Recent projects
                                    ui.heading("📂 Recent Projects");
                                    ui.add_space(5.0);

                                    match launcher_state.project_manager.list_projects() {
                                        Ok(projects) => {
                                            if projects.is_empty() {
                                                ui.label("No projects yet. Create a new one to get started!");
                                            } else {
                                                for project in projects.iter() {
                                                    ui.group(|ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.vertical(|ui| {
                                                                ui.strong(&project.name);
                                                                ui.label(&project.description);
                                                                ui.label(format!("Last modified: {}", project.last_modified));
                                                            });
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                if ui.button("🗑 Delete").clicked() {
                                                                    if let Err(e) = launcher_state.project_manager.delete_project(&project.path) {
                                                                        launcher_state.error_message = Some(format!("Error: {}", e));
                                                                    }
                                                                }
                                                                if ui.button("▶ Open").clicked() {
                                                                    // Open existing projects in editor mode
                                                                    app_state = AppState::Editor;
                                                                    editor_state = EditorState::new();
                                                                    editor_state.current_project_path = Some(project.path.clone());
                                                                    editor_state.asset_browser_path = Some(project.path.clone());

                                                                    // Load startup scene if configured
                                                                    if let Ok(Some(startup_scene)) = launcher_state.project_manager.get_startup_scene(&project.path) {
                                                                        let scene_path = project.path.join(&startup_scene);
                                                                        if scene_path.exists() {
                                                                            if let Err(e) = editor_state.load_scene(&scene_path) {
                                                                                editor_state.console.error(format!("Failed to load startup scene: {}", e));
                                                                            } else {
                                                                                editor_state.current_scene_path = Some(scene_path.clone());
                                                                                editor_state.console.info(format!("Loaded startup scene: {}", startup_scene.display()));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            });
                                                        });
                                                    });
                                                    ui.add_space(5.0);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            ui.label(format!("Error loading projects: {}", e));
                                        }
                                    }

                                    // Error message
                                    if let Some(ref error) = launcher_state.error_message {
                                        ui.add_space(10.0);
                                        ui.colored_label(egui::Color32::RED, error);
                                    }
                                });

                                // New project dialog
                                if launcher_state.show_new_project_dialog {
                                    egui::Window::new("Create New Project")
                                        .collapsible(false)
                                        .resizable(false)
                                        .show(&egui_ctx, |ui| {
                                            ui.label("Project Name:");
                                            ui.text_edit_singleline(&mut launcher_state.new_project_name);

                                            ui.label("Description:");
                                            ui.text_edit_singleline(&mut launcher_state.new_project_desc);

                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Create").clicked() {
                                                    match launcher_state.project_manager.create_project(
                                                        &launcher_state.new_project_name,
                                                        &launcher_state.new_project_desc,
                                                    ) {
                                                        Ok(metadata) => {
                                                            launcher_state.show_new_project_dialog = false;
                                                            // Open new projects in empty editor
                                                            app_state = AppState::Editor;
                                                            editor_state = EditorState::new();
                                                            editor_state.current_project_path = Some(metadata.path);
                                                        }
                                                        Err(e) => {
                                                            launcher_state.error_message = Some(format!("Error: {}", e));
                                                        }
                                                    }
                                                }
                                                if ui.button("Cancel").clicked() {
                                                    launcher_state.show_new_project_dialog = false;
                                                }
                                            });
                                        });
                                }
                            }
                            AppState::Playing => {
                                // Game UI - collect data first to avoid borrowing issues
                                let (collected, items_remaining, player_pos) = if let Some(ref module) = sample_module {
                                    let collected = module.game_state.collected_items;
                                    let items_remaining = module.game_state.items.len();
                                    let player_pos = if let Some(player) = module.game_state.player {
                                        module.game_state.world.transforms.get(&player)
                                            .map(|t| (t.x(), t.y()))
                                            .unwrap_or((0.0, 0.0))
                                    } else {
                                        (0.0, 0.0)
                                    };
                                    (collected, items_remaining, player_pos)
                                } else {
                                    (0, 0, (0.0, 0.0))
                                };

                                let mut should_return_to_launcher = false;

                                egui::Window::new("Game Stats")
                                    .default_pos([10.0, 10.0])
                                    .show(&egui_ctx, |ui| {
                                        ui.heading("Item Collection Game");
                                        ui.separator();
                                        ui.label(format!("Items Collected: {}", collected));
                                        ui.label(format!("Items Remaining: {}", items_remaining));
                                        ui.separator();
                                        ui.label(format!("Player Position: ({:.0}, {:.0})", player_pos.0, player_pos.1));
                                        ui.separator();
                                        ui.label("Controls:");
                                        ui.label("  WASD or Arrow Keys - Move");
                                        ui.label("  ESC - Quit");
                                        ui.separator();
                                        if ui.button("⬅ Back to Launcher").clicked() {
                                            should_return_to_launcher = true;
                                        }
                                    });

                                // Draw entities
                                if let Some(ref module) = sample_module {
                                    egui::Window::new("Game View")
                                        .default_pos([10.0, 200.0])
                                        .default_size([600.0, 400.0])
                                        .show(&egui_ctx, |ui| {
                                            let (response, painter) = ui.allocate_painter(
                                                egui::vec2(560.0, 360.0),
                                                egui::Sense::hover(),
                                            );
                                            let rect = response.rect;

                                            // Draw background
                                            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));

                                            // Draw entities
                                            let world = &module.game_state.world;
                                            for (&entity, transform) in &world.transforms {
                                                if let Some(sprite) = world.sprites.get(&entity) {
                                                    let screen_x = rect.min.x + transform.x() * 0.7;
                                                    let screen_y = rect.min.y + transform.y() * 0.6;
                                                    let size = egui::vec2(sprite.width * 0.7, sprite.height * 0.6);

                                                    let color = egui::Color32::from_rgba_unmultiplied(
                                                        (sprite.color[0] * 255.0) as u8,
                                                        (sprite.color[1] * 255.0) as u8,
                                                        (sprite.color[2] * 255.0) as u8,
                                                        (sprite.color[3] * 255.0) as u8,
                                                    );

                                                    painter.rect_filled(
                                                        egui::Rect::from_min_size(
                                                            egui::pos2(screen_x - size.x / 2.0, screen_y - size.y / 2.0),
                                                            size,
                                                        ),
                                                        2.0,
                                                        color,
                                                    );
                                                }
                                            }
                                        });
                                }

                                if should_return_to_launcher {
                                    app_state = AppState::Launcher;
                                    sample_module = None;
                                }
                            }
                            AppState::Editor => {
                                let mut save_request = false;
                                let mut save_as_request = false;
                                let mut load_request = false;
                                let mut load_file_request: Option<std::path::PathBuf> = None;
                                let mut new_scene_request = false;
                                let mut play_request = false;
                                let mut stop_request = false;
                                let mut edit_script_request: Option<String> = None;

                                // Handle Q/W/E/R keyboard shortcuts for transform tools
                                egui_ctx.input(|i| {
                                    if i.key_pressed(egui::Key::Q) {
                                        editor_state.current_tool = TransformTool::View;
                                        editor_state.console.info("Tool: View (Q)".to_string());
                                    } else if i.key_pressed(egui::Key::W) {
                                        editor_state.current_tool = TransformTool::Move;
                                        editor_state.console.info("Tool: Move (W)".to_string());
                                    } else if i.key_pressed(egui::Key::E) {
                                        editor_state.current_tool = TransformTool::Rotate;
                                        editor_state.console.info("Tool: Rotate (E)".to_string());
                                    } else if i.key_pressed(egui::Key::R) {
                                        editor_state.current_tool = TransformTool::Scale;
                                        editor_state.console.info("Tool: Scale (R)".to_string());
                                    }
                                });

                                // Editor UI
                                EditorUI::render_editor(
                                    &egui_ctx,
                                    &mut editor_state.world,
                                    &mut editor_state.selected_entity,
                                    &mut editor_state.entity_names,
                                    &mut save_request,
                                    &mut save_as_request,
                                    &mut load_request,
                                    &mut load_file_request,
                                    &mut new_scene_request,
                                    &mut play_request,
                                    &mut stop_request,
                                    &mut edit_script_request,
                                    &editor_state.current_project_path,
                                    &editor_state.current_scene_path,
                                    &mut editor_state.scene_view_tab,
                                    editor_state.is_playing,
                                    &mut editor_state.show_colliders,
                                    &mut editor_state.show_velocities,
                                    &mut editor_state.console,
                                    &mut editor_state.bottom_panel_tab,
                                    &editor_state.current_tool,
                                    &mut editor_state.show_project_settings,
                                );

                                // Handle new scene request
                                if new_scene_request {
                                    if editor_state.scene_modified {
                                        editor_state.show_unsaved_changes_dialog = true;
                                        editor_state.pending_action = Some(EditorAction::NewScene);
                                    } else {
                                        editor_state.world.clear();
                                        editor_state.entity_names.clear();
                                        editor_state.selected_entity = None;
                                        editor_state.current_scene_path = None;
                                        editor_state.scene_modified = false;
                                        editor_state.console.info("New scene created");
                                    }
                                }

                                // Handle save as request
                                if save_as_request {
                                    // Start dialog in project/scenes/ folder if project is open
                                    let mut dialog = rfd::FileDialog::new()
                                        .add_filter("Scene", &["scene"])
                                        .set_file_name("scene.scene");

                                    if let Some(proj_path) = &editor_state.current_project_path {
                                        let scenes_folder = proj_path.join("scenes");
                                        if scenes_folder.exists() {
                                            dialog = dialog.set_directory(&scenes_folder);
                                        }
                                    }

                                    if let Some(file) = dialog.save_file() {
                                        // Validate that a project is open
                                        if editor_state.current_project_path.is_none() {
                                            editor_state.console.error("Cannot save scene: No project is open!".to_string());
                                        } else if let Some(proj_path) = &editor_state.current_project_path {
                                            // Validate that file is inside project/scenes/
                                            let scenes_folder = proj_path.join("scenes");
                                            if !file.starts_with(&scenes_folder) {
                                                editor_state.console.error("Scene must be saved inside project/scenes/ folder!".to_string());
                                            } else {
                                                if let Err(e) = editor_state.save_scene(&file) {
                                                    log::error!("Failed to save scene: {}", e);
                                                    editor_state.console.error(format!("Failed to save scene: {}", e));
                                                } else {
                                                    editor_state.current_scene_path = Some(file.clone());
                                                    editor_state.console.info(format!("Scene saved as: {}", file.display()));
                                                }
                                            }
                                        }
                                    }
                                }

                                // Handle save request
                                if save_request {
                                    // Check if project is open
                                    if editor_state.current_project_path.is_none() {
                                        editor_state.console.error("Cannot save scene: No project is open!".to_string());
                                    } else {
                                        let path_clone = editor_state.current_scene_path.clone();
                                        if let Some(path) = path_clone {
                                            if let Err(e) = editor_state.save_scene(&path) {
                                                log::error!("Failed to save scene: {}", e);
                                                editor_state.console.error(format!("Failed to save scene: {}", e));
                                            } else {
                                                editor_state.console.info(format!("Scene saved: {}", path.display()));
                                            }
                                        } else {
                                            // No current path, use default path in project/scenes/
                                            if let Some(default_path) = editor_state.get_default_scene_path("scene") {
                                                if let Err(e) = editor_state.save_scene(&default_path) {
                                                    log::error!("Failed to save scene: {}", e);
                                                    editor_state.console.error(format!("Failed to save scene: {}", e));
                                                } else {
                                                    editor_state.current_scene_path = Some(default_path.clone());
                                                    editor_state.console.info(format!("Scene saved: {}", default_path.display()));
                                                }
                                            } else {
                                                editor_state.console.error("Cannot create default scene path: No project is open!".to_string());
                                            }
                                        }
                                    }
                                }

                                // Handle load request (Browse)
                                if load_request {
                                    if editor_state.scene_modified {
                                        editor_state.show_unsaved_changes_dialog = true;
                                        editor_state.pending_action = Some(EditorAction::LoadScene(None));
                                    } else {
                                        // Start dialog in project/scenes/ folder if project is open
                                        let mut dialog = rfd::FileDialog::new()
                                            .add_filter("Scene", &["scene"]);

                                        if let Some(proj_path) = &editor_state.current_project_path {
                                            let scenes_folder = proj_path.join("scenes");
                                            if scenes_folder.exists() {
                                                dialog = dialog.set_directory(&scenes_folder);
                                            }
                                        }

                                        if let Some(file) = dialog.pick_file() {
                                            if let Err(e) = editor_state.load_scene(&file) {
                                                log::error!("Failed to load scene: {}", e);
                                                editor_state.console.error(format!("Failed to load scene: {}", e));
                                            } else {
                                                editor_state.current_scene_path = Some(file.clone());
                                                editor_state.console.info(format!("Scene loaded: {}", file.display()));
                                            }
                                        }
                                    }
                                }

                                // Handle load file request (Direct)
                                if let Some(path) = load_file_request {
                                    if editor_state.scene_modified {
                                        editor_state.show_unsaved_changes_dialog = true;
                                        editor_state.pending_action = Some(EditorAction::LoadScene(Some(path)));
                                    } else {
                                        if let Err(e) = editor_state.load_scene(&path) {
                                            log::error!("Failed to load scene: {}", e);
                                            editor_state.console.error(format!("Failed to load scene: {}", e));
                                        } else {
                                            editor_state.current_scene_path = Some(path.clone());
                                            editor_state.console.info(format!("Scene loaded: {}", path.display()));
                                        }
                                    }
                                }

                                // Handle edit script request
                                if let Some(script_name) = edit_script_request {
                                    if let Err(e) = editor_state.open_script_in_editor(&script_name) {
                                        log::error!("Failed to open script: {}", e);
                                    }
                                }

                                // Show save required dialog if needed
                                if editor_state.show_save_required_dialog {
                                    egui::Window::new("Save Required")
                                        .collapsible(false)
                                        .resizable(false)
                                        .show(&egui_ctx, |ui| {
                                            ui.label("Please save the scene before playing.");
                                            if ui.button("OK").clicked() {
                                                editor_state.show_save_required_dialog = false;
                                            }
                                        });
                                }

                                // Show unsaved changes dialog
                                if editor_state.show_unsaved_changes_dialog {
                                    egui::Window::new("Unsaved Changes")
                                        .collapsible(false)
                                        .resizable(false)
                                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                                        .show(&egui_ctx, |ui| {
                                            ui.label("You have unsaved changes. Do you want to save them?");
                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Save").clicked() {
                                                    // Save logic
                                                    let mut saved = false;
                                                    if let Some(path) = editor_state.current_scene_path.clone() {
                                                        if let Ok(_) = editor_state.save_scene(&path) {
                                                            saved = true;
                                                        }
                                                    } else {
                                                        if let Some(file) = rfd::FileDialog::new()
                                                            .add_filter("Scene", &["scene"])
                                                            .set_file_name("scene.scene")
                                                            .save_file()
                                                        {
                                                            if let Ok(_) = editor_state.save_scene(&file) {
                                                                saved = true;
                                                            }
                                                        }
                                                    }

                                                    if saved {
                                                        editor_state.show_unsaved_changes_dialog = false;
                                                        // Proceed with pending action
                                                        match editor_state.pending_action.take() {
                                                            Some(EditorAction::NewScene) => {
                                                                editor_state.world.clear();
                                                                editor_state.entity_names.clear();
                                                                editor_state.selected_entity = None;
                                                                editor_state.current_scene_path = None;
                                                                editor_state.scene_modified = false;
                                                                editor_state.console.info("New scene created");
                                                            }
                                                            Some(EditorAction::LoadScene(target_path)) => {
                                                                if let Some(path) = target_path {
                                                                    // Load specific file
                                                                    if let Err(e) = editor_state.load_scene(&path) {
                                                                        editor_state.console.error(format!("Failed to load scene: {}", e));
                                                                    } else {
                                                                        editor_state.console.info(format!("Scene loaded: {}", path.display()));
                                                                    }
                                                                } else {
                                                                    // Browse
                                                                    if let Some(file) = rfd::FileDialog::new()
                                                                        .add_filter("Scene", &["scene"])
                                                                        .pick_file()
                                                                    {
                                                                        if let Err(e) = editor_state.load_scene(&file) {
                                                                            editor_state.console.error(format!("Failed to load scene: {}", e));
                                                                        } else {
                                                                            editor_state.console.info(format!("Scene loaded: {}", file.display()));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Some(EditorAction::Quit) => {
                                                                target.exit();
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                }

                                                if ui.button("Don't Save").clicked() {
                                                    editor_state.show_unsaved_changes_dialog = false;
                                                    // Proceed with pending action without saving
                                                    match editor_state.pending_action.take() {
                                                        Some(EditorAction::NewScene) => {
                                                            editor_state.world.clear();
                                                            editor_state.entity_names.clear();
                                                            editor_state.selected_entity = None;
                                                            editor_state.current_scene_path = None;
                                                            editor_state.scene_modified = false;
                                                            editor_state.console.info("New scene created (unsaved changes discarded)");
                                                        }
                                                        Some(EditorAction::LoadScene(target_path)) => {
                                                            if let Some(path) = target_path {
                                                                // Load specific file
                                                                if let Err(e) = editor_state.load_scene(&path) {
                                                                    editor_state.console.error(format!("Failed to load scene: {}", e));
                                                                } else {
                                                                    editor_state.console.info(format!("Scene loaded: {}", path.display()));
                                                                }
                                                            } else {
                                                                // Browse
                                                                if let Some(file) = rfd::FileDialog::new()
                                                                    .add_filter("Scene", &["scene"])
                                                                    .pick_file()
                                                                {
                                                                    if let Err(e) = editor_state.load_scene(&file) {
                                                                        editor_state.console.error(format!("Failed to load scene: {}", e));
                                                                    } else {
                                                                        editor_state.console.info(format!("Scene loaded: {}", file.display()));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Some(EditorAction::Quit) => {
                                                            target.exit();
                                                        }
                                                        None => {}
                                                    }
                                                }

                                                if ui.button("Cancel").clicked() {
                                                    editor_state.show_unsaved_changes_dialog = false;
                                                    editor_state.pending_action = None;
                                                }
                                            });
                                        });
                                }

                                // Handle play request - enter play mode in editor
                                if play_request {
                                    if !editor_state.is_playing {
                                        // Check if scene is saved
                                        if editor_state.current_scene_path.is_none() {
                                            // Show warning dialog - need to save first
                                            editor_state.show_save_required_dialog = true;
                                        } else {
                                            // Save before playing
                                            if let Some(ref path) = editor_state.current_scene_path.clone() {
                                                if let Err(e) = editor_state.save_scene(path) {
                                                    log::error!("Failed to save scene: {}", e);
                                                } else {
                                                    // Enter play mode - backup world and switch to Game tab
                                                    editor_state.play_world = Some(editor_state.world.clone());
                                                    editor_state.is_playing = true;
                                                    editor_state.scene_view_tab = 1; // Switch to Game tab
                                                    log::info!("Entering play mode in editor");
                                                    editor_state.console.info("▶️ Entering Play Mode...");

                                                    // Load and initialize all scripts
                                                    let entities_with_scripts: Vec<_> = editor_state.world.scripts.keys().cloned().collect();
                                                    for entity in entities_with_scripts {
                                                        if let Some(script) = editor_state.world.scripts.get(&entity) {
                                                            if script.enabled {
                                                                let script_name = script.script_name.clone();
                                                                if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                                                                    let script_path = scripts_folder.join(format!("{}.lua", script_name));
                                                                    if script_path.exists() {
                                                                        // Load script once
                                                                        if let Ok(content) = std::fs::read_to_string(&script_path) {
                                                                            if let Err(e) = script_engine.load_script(&content) {
                                                                                log::error!("Failed to load script {}: {}", script_name, e);
                                                                                editor_state.console.error(format!("Failed to load script {}: {}", script_name, e));
                                                                            } else {
                                                                                editor_state.console.debug(format!("Loaded script: {}.lua", script_name));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Handle stop request - exit play mode
                                if stop_request {
                                    if editor_state.is_playing {
                                        // Restore world from backup and switch to Scene tab
                                        if let Some(backup_world) = editor_state.play_world.take() {
                                            editor_state.world = backup_world;
                                        }
                                        editor_state.is_playing = false;
                                        editor_state.scene_view_tab = 0; // Switch back to Scene tab
                                        log::info!("Exiting play mode in editor");
                                        editor_state.console.info("⏹️ Exited Play Mode");
                                    }
                                }

                                // Game loop update when playing
                                if editor_state.is_playing {
                                    let now = std::time::Instant::now();
                                    let dt = (now - last_frame_time).as_secs_f32();
                                    last_frame_time = now;

                                    // Update physics
                                    physics.step(dt, &mut editor_state.world);

                                    // Run scripts
                                    let entities_with_scripts: Vec<_> = editor_state.world.scripts.keys().cloned().collect();
                                    for entity in entities_with_scripts {
                                        if let Some(script) = editor_state.world.scripts.get(&entity) {
                                            if script.enabled {
                                                let script_name = script.script_name.clone();
                                                if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                                                    let script_path = scripts_folder.join(format!("{}.lua", script_name));
                                                    if script_path.exists() {
                                                        if let Err(e) = script_engine.run_script(&script_path, entity, &mut editor_state.world, &editor_state.input_system, dt) {
                                                            log::error!("Script error for {}: {}", script_name, e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let full_output = egui_ctx.end_frame();
                        
                        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [renderer.config.width, renderer.config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };
                        
                        // Update textures
                        for (id, image_delta) in &full_output.textures_delta.set {
                            egui_renderer.update_texture(&renderer.device, &renderer.queue, *id, image_delta);
                        }

                        let res = renderer.render_with_callback(|device, queue, encoder, view| {
                            egui_renderer.update_buffers(
                                device,
                                queue,
                                encoder,
                                &paint_jobs,
                                &screen_descriptor,
                            );

                            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui_render"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });

                            egui_renderer.render(
                                &mut rpass,
                                &paint_jobs,
                                &screen_descriptor,
                            );
                        });

                        // Free textures
                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        match res {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}

use crate::states::EditorState;
use ecs::World;
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;

pub struct MenuCommandSystem;

impl MenuCommandSystem {
    pub fn handle_commands(
        editor_state: &mut EditorState,
        script_engine: &mut ScriptEngine,
        physics: &mut dyn std::any::Any,
        save_request: &mut bool,
        save_as_request: &mut bool,
        load_request: &mut bool,
        load_file_request: &mut Option<std::path::PathBuf>,
        new_scene_request: &mut bool,
        play_request: &mut bool,
        stop_request: &mut bool,
        edit_script_request: &mut Option<String>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_manager: &mut render::TextureManager,
        mesh_renderer: &render::MeshRenderer,
        asset_loader: &dyn engine_core::assets::AssetLoader,
        render_cache: &mut engine::runtime::render_system::RenderCache,
    ) {
         // Edit script request
         if let Some(script_name) = edit_script_request {
            if let Some(project_path) = &editor_state.current_project_path {
                let script_path = project_path.join("scripts").join(&script_name);
                if script_path.exists() {
                     if let Err(e) = open::that(&script_path) {
                         editor_state.console.error(format!("Failed to open script: {}", e));
                     } else {
                         editor_state.console.info(format!("Opening script: {}", script_name));
                     }
                } else {
                     editor_state.console.warning(format!("Script file not found: {:?}", script_path));
                }
            }
        }
        
        // Handle sprite editor open requests
        if let Some(texture_path) = editor_state.open_sprite_editor_request.take() {
            if editor_state.use_docking {
                // In docking mode, add sprite editor as a new tab
                // Tab management for docking system - simplified info for now
                 editor_state.console.info(format!("Opening sprite editor for: {}", texture_path.display()));
            } else {
                 // Standalone window logic
                 let already_open = editor_state.sprite_editor_windows.iter()
                    .any(|w| w.state().texture_path == texture_path);

                if !already_open {
                    let window = crate::SpriteEditorWindow::new(texture_path.clone());
                    editor_state.sprite_editor_windows.push(window);
                    editor_state.console.info(format!("Opened sprite editor for: {}", texture_path.display()));
                }
            }
        }
        
        // New Scene
        if *new_scene_request {
            editor_state.world = World::new();
            editor_state.entity_names.clear();
            editor_state.selected_entity = None;
            editor_state.current_scene_path = None;
            editor_state.console.info("New scene created".to_string());
        }

        // Save Scene
        if *save_request {
            if let Some(path) = editor_state.current_scene_path.clone() {
                 if let Err(e) = editor_state.save_scene(&path) {
                      editor_state.console.error(format!("Failed to save scene: {}", e));
                 } else {
                      editor_state.console.info(format!("Scene saved: {:?}", path));
                 }
            } else {
                *save_as_request = true;
            }
        }

        // Save As
        if *save_as_request {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scene", &["json", "scene"])
                .save_file() 
            {
                 if let Err(e) = editor_state.save_scene(&path) {
                      editor_state.console.error(format!("Failed to save scene: {}", e));
                 } else {
                      editor_state.current_scene_path = Some(path.clone());
                      editor_state.console.info(format!("Scene saved: {:?}", path));
                 }
            }
        }

        // Load Scene Request (from menu)
        if *load_request {
             if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scene", &["json", "scene"])
                .pick_file() 
            {
                 if let Err(e) = editor_state.load_scene(&path, asset_loader) {
                      editor_state.console.error(format!("Failed to load scene: {}", e));
                 } else {
                      editor_state.current_scene_path = Some(path.clone());
                      editor_state.console.info(format!("Scene loaded: {:?}", path));
                 }
            }
        }

        // Load File Request (from asset browser)
        if let Some(path) = load_file_request {
             if let Err(e) = editor_state.load_scene(&path, asset_loader) {
                  editor_state.console.error(format!("Failed to load scene: {}", e));
             } else {
                  editor_state.current_scene_path = Some(path.clone());
                  editor_state.console.info(format!("Scene loaded: {:?}", path));
             }
        }

        // Play/Stop
        if *play_request {
            if !editor_state.is_playing {
                 // Start playing
                 editor_state.is_playing = true;
                 editor_state.console.info("▶ Starting Play Mode...".to_string());

                 // Clear script engine to ensure fresh start
                 script_engine.clear();

                 // Process GLTF assets (same as scene loading)
                 if let Some(project_path) = &editor_state.current_project_path {
                     use engine::runtime::render_system::post_process_asset_meshes;
                     post_process_asset_meshes(
                         render_cache,
                         project_path,
                         &mut editor_state.world,
                         device,
                         queue,
                         texture_manager,
                         mesh_renderer,
                         asset_loader,
                     );
                 }

                 // Load scripts (same as Player binary) - INLINE VERSION WITH DEBUG LOGS
                 if let Some(ref project_path) = editor_state.current_project_path {
                     // Inline load_all_scripts to add debug logging
                     let entities_to_load: Vec<_> = editor_state.world.scripts.keys().cloned().collect();
                     editor_state.console.debug(format!("🔍 [LoadScripts] Found {} entities with script components", entities_to_load.len()));

                     let mut load_success = true;
                     for entity in &entities_to_load {
                         if let Some(script) = editor_state.world.scripts.get(entity) {
                             if script.enabled {
                                 let script_name = script.script_name.clone();
                                 // Include project path in the script path
                                let script_path = format!("{}/scripts/{}.lua", project_path.display(), script_name);
                                 editor_state.console.debug(format!("🔍 [LoadScripts] Loading script: {}", script_path));

                                 match pollster::block_on(script_engine.asset_loader.load_text(&script_path)) {
                                     Ok(content) => {
                                         editor_state.console.debug(format!("✅ [LoadScripts] Script file loaded: {} ({} bytes)", script_path, content.len()));
                                         if let Err(e) = script_engine.load_script_for_entity(*entity, &content, &mut editor_state.world) {
                                             editor_state.console.error(format!("Failed to load script {} for entity {}: {}", script_name, entity, e));
                                             load_success = false;
                                         } else {
                                             editor_state.console.debug(format!("✅ [LoadScripts] Script loaded for entity {}: {} (Awake called)", entity, script_name));
                                         }
                                     }
                                     Err(e) => {
                                         editor_state.console.error(format!("Script file not found or failed to load: {} ({})", script_path, e));
                                         load_success = false;
                                     }
                                 }
                             }
                         }
                     }

                     if load_success {
                         editor_state.console.info("Scripts loaded successfully".to_string());
                         
                         // Start scripts (call Start() for all entities with scripts)
                         let entities_with_scripts: Vec<_> = editor_state.world.scripts.keys().copied().collect();
                         editor_state.console.debug(format!("🔍 Found {} entities with scripts", entities_with_scripts.len()));

                         for entity in entities_with_scripts {
                             editor_state.console.debug(format!("🔍 Calling Start() for entity {}", entity));
                             if let Err(e) = script_engine.call_start_for_entity(entity, &mut editor_state.world) {
                                 editor_state.console.error(format!("Script start error for entity {:?}: {}", entity, e));
                             } else {
                                 editor_state.console.debug(format!("✅ Start() completed for entity {}", entity));
                             }
                         }
                     }
                 }
                 
                 // Initialize physics
                 #[cfg(feature = "rapier")]
                 {
                     if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                         rapier_world.sync_from_ecs(&editor_state.world);
                         editor_state.console.info("Physics (Rapier) initialized".to_string());
                     }
                 }
                 #[cfg(not(feature = "rapier"))]
                 {
                     if let Some(_simple_world) = physics.downcast_mut::<PhysicsWorld>() {
                         editor_state.console.info("Physics (Simple) initialized".to_string());
                     }
                 }
            }
        }

        if *stop_request {
            if editor_state.is_playing {
                 editor_state.is_playing = false;
                 editor_state.console.info("⏹ Stopping Play Mode...".to_string());

                 // Clear script engine to remove all entity Lua states
                 script_engine.clear();

                 // Reload scene to reset state
                 if let Some(path) = editor_state.current_scene_path.clone() {
                      if let Err(e) = editor_state.load_scene(&path, asset_loader) {
                           editor_state.console.error(format!("Failed to reload scene after stop: {}", e));
                      }
                 }
            }
        }
    }
}

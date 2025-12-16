use crate::states::EditorState;
use ecs::World;
use engine_core::EngineContext;
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
                 if let Err(e) = editor_state.load_scene(&path) {
                      editor_state.console.error(format!("Failed to load scene: {}", e));
                 } else {
                      editor_state.current_scene_path = Some(path.clone());
                      editor_state.console.info(format!("Scene loaded: {:?}", path));
                 }
            }
        }

        // Load File Request (from asset browser)
        if let Some(path) = load_file_request {
             if let Err(e) = editor_state.load_scene(&path) {
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
                 
                 // Load scripts (same as Player binary)
                 if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                     if let Err(e) = engine::runtime::script_loader::load_all_scripts(&mut editor_state.world, script_engine, &scripts_folder) {
                         editor_state.console.error(format!("Failed to load scripts: {}", e));
                     } else {
                         editor_state.console.info("Scripts loaded successfully".to_string());
                         
                         // Start scripts (call Start() for all entities with scripts)
                         let entities_with_scripts: Vec<_> = editor_state.world.scripts.keys().copied().collect();
                         for entity in entities_with_scripts {
                             if let Err(e) = script_engine.call_start_for_entity(entity, &mut editor_state.world) {
                                 editor_state.console.error(format!("Script start error for entity {:?}: {}", entity, e));
                             }
                         }
                     }
                 }
                 
                 // Initialize physics
                 #[cfg(feature = "rapier")]
                 {
                     if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                         rapier_world.register_colliders(&editor_state.world);
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
                 
                 // Reload scene to reset state
                 if let Some(path) = editor_state.current_scene_path.clone() {
                      if let Err(e) = editor_state.load_scene(&path) {
                           editor_state.console.error(format!("Failed to reload scene after stop: {}", e));
                      }
                 }
            }
        }
    }
}

use engine_core::EngineContext;
use crate::states::{AppState, EditorState, EditorAction};
use ecs::World;
// Sample game removed - use projects/ folder for game content
use crate::ui::{EditorUI, TransformTool};
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use script::ScriptEngine;

pub struct EditorLogic;

impl EditorLogic {
    pub fn handle_editor_frame(
        egui_ctx: &egui::Context,
        app_state: &mut AppState,
        editor_state: &mut EditorState,
        ctx: &mut EngineContext,
        script_engine: &mut ScriptEngine,
        physics: &mut dyn std::any::Any, // Passed as Any because type varies by feature
        physics_accumulator: &mut f32,
        fixed_time_step: f32,
        dt: f32,
    ) {
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
            
            // Delete entity
            if i.key_pressed(egui::Key::Delete) {
                if let Some(entity) = editor_state.selected_entity {
                    editor_state.execute_action(EditorAction::DeleteEntity(entity));
                }
            }
            
            // Focus on object (F) - logic moved to main.rs event loop usually, but can check here
            // Note: F key handling is in main.rs window event loop because it needs camera access and simpler there
        });

        // ---------------------------------------------------------
        // Render Editor UI
        // ---------------------------------------------------------
        if editor_state.use_docking {
             EditorUI::render_editor_with_dock(
                egui_ctx,
                &mut editor_state.dock_state,
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
                &mut editor_state.current_tool,
                &mut editor_state.show_project_settings,
                &mut editor_state.scene_camera,
                &editor_state.scene_grid,
                &mut editor_state.infinite_grid,
                // NEW: Unity-like editor features
                &editor_state.camera_state_display,
                &mut editor_state.show_exit_dialog,
                &mut editor_state.show_export_dialog,
                &mut editor_state.asset_manager,
                &mut editor_state.drag_drop,
                &mut editor_state.layout_request,
                &editor_state.current_layout_name,
                &mut editor_state.dragging_entity,
                &mut editor_state.drag_axis,
                &mut editor_state.scene_view_mode,
                &mut editor_state.projection_mode,
                &mut editor_state.transform_space,
                &mut editor_state.texture_manager,
                &mut editor_state.open_sprite_editor_request,
                &mut editor_state.open_prefab_editor_request,
                &mut editor_state.sprite_editor_windows,
                &mut editor_state.sprite_picker_state,
                &mut editor_state.texture_inspector,
                &mut editor_state.map_view_state,
                &mut editor_state.show_debug_lines,
                &mut editor_state.debug_draw,
                &mut editor_state.map_manager,
                &mut editor_state.prefab_manager,
                &mut editor_state.create_prefab_dialog,
                &mut editor_state.layer_properties_panel,
                &mut editor_state.layer_ordering_panel,
                &mut editor_state.performance_panel,
                &mut editor_state.collider_settings_panel,
                &mut editor_state.game_view_settings,
                &mut editor_state.prefab_editor,
                &mut editor_state.ui_manager,
                dt,
            );
        } else {
             // Fallback to old layout
             EditorUI::render_editor(
                egui_ctx,
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
                &mut editor_state.current_tool,
                &mut editor_state.show_project_settings,
                &mut editor_state.scene_camera,
                &editor_state.scene_grid,
                &mut editor_state.infinite_grid,
                &editor_state.camera_state_display,
                &mut editor_state.show_exit_dialog,
                &mut editor_state.show_export_dialog,
                &mut editor_state.asset_manager,
                &mut editor_state.drag_drop,
                &mut editor_state.layout_request,
                &mut editor_state.texture_manager,
                &mut editor_state.open_sprite_editor_request,
                &mut editor_state.sprite_picker_state,
                &mut editor_state.show_debug_lines,
             );
        }

        // ---------------------------------------------------------
        // Handle Dialogs & Popups
        // ---------------------------------------------------------

        // Render Create Prefab Dialog
        if let Some(prefab_name) = editor_state.create_prefab_dialog.render(
            egui_ctx,
            &editor_state.world,
            &editor_state.entity_names,
            &editor_state.prefab_manager,
        ) {
            // User confirmed prefab creation
            if let Some(entity) = editor_state.create_prefab_dialog.entity {
                match editor_state.prefab_manager.create_prefab(
                    entity,
                    &editor_state.world,
                    &editor_state.entity_names,
                    prefab_name.clone(),
                ) {
                    Ok(path) => {
                        editor_state.console.info(format!("✅ Created prefab: {:?}", path));
                    }
                    Err(e) => {
                        editor_state.console.error(format!("❌ Failed to create prefab: {}", e));
                    }
                }
            }
        }
        
        // Show exit confirmation dialog
        if editor_state.show_exit_dialog {
            egui::Window::new("Exit Editor")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(egui_ctx, |ui| {
                    if editor_state.scene_modified {
                        ui.label("You have unsaved changes. Do you want to save before exiting?");
                    } else {
                        ui.label("Are you sure you want to exit?");
                    }
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if editor_state.scene_modified {
                            if ui.button("Save and Exit").clicked() {
                                // Save scene
                                let mut saved = false;
                                if let Some(ref path) = editor_state.current_scene_path.clone() {
                                    if let Err(e) = editor_state.save_scene(path) {
                                        editor_state.console.error(format!("Failed to save: {}", e));
                                    } else {
                                        saved = true;
                                    }
                                }
                                
                                if saved {
                                    editor_state.should_exit = true;
                                    editor_state.show_exit_dialog = false;
                                }
                            }
                            
                            if ui.button("Exit Without Saving").clicked() {
                                editor_state.should_exit = true;
                                editor_state.show_exit_dialog = false;
                            }
                        } else {
                            if ui.button("Exit").clicked() {
                                editor_state.should_exit = true;
                                editor_state.show_exit_dialog = false;
                            }
                        }
                        
                        if ui.button("Cancel").clicked() {
                            editor_state.show_exit_dialog = false;
                        }
                    });
                });
        }

        // Render sprite picker popup
        if let Some(result) = crate::ui::sprite_picker::render_sprite_picker(
            egui_ctx,
            &mut editor_state.sprite_picker_state,
            editor_state.current_project_path.as_ref(),
            &mut editor_state.texture_manager,
        ) {
            // User selected a sprite - update the selected entity's Sprite component
            if let Some(entity) = editor_state.selected_entity {
                // Check if this is a sprite from a .sprite file
                let is_sprite_sheet = result.sprite_file_path.exists();
                
                // Convert texture path to relative path
                let relative_path = {
                    let path_str = result.texture_path.to_string_lossy();
                    if let Some(idx) = path_str.find("projects/") {
                        let after_projects = &path_str[idx + "projects/".len()..];
                        if let Some(next_slash) = after_projects.find('/') {
                            after_projects[next_slash + 1..].replace('\\', "/")
                        } else {
                            path_str.replace('\\', "/")
                        }
                    } else {
                        path_str.replace('\\', "/")
                    }
                };
                
                if is_sprite_sheet {
                    match sprite_editor::SpriteMetadata::load(&result.sprite_file_path) {
                        Ok(metadata) => {
                            if let Some(sprite_def) = metadata.find_sprite(&result.sprite_name) {
                                let sprite = ecs::Sprite {
                                    texture_id: relative_path.clone(),
                                    width: sprite_def.width as f32,
                                    height: sprite_def.height as f32,
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    billboard: false,
                                    flip_x: false,
                                    flip_y: false,
                                    sprite_rect: Some([sprite_def.x, sprite_def.y, sprite_def.width, sprite_def.height]),
                                    pixels_per_unit: 100.0,
                                };
                                
                                editor_state.world.sprites.insert(entity, sprite);
                                editor_state.scene_modified = true;
                                editor_state.console.info(format!("Selected sprite: {}", result.sprite_name));
                            } else {
                                editor_state.console.error(format!("Sprite '{}' not found in metadata", result.sprite_name));
                            }
                        }
                        Err(e) => {
                            editor_state.console.error(format!("Failed to load sprite metadata: {}", e));
                        }
                    }
                } else {
                    let sprite = ecs::Sprite {
                        texture_id: relative_path,
                        width: 1.0,
                        height: 1.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        billboard: false,
                        flip_x: false,
                        flip_y: false,
                        pixels_per_unit: 100.0,
                        sprite_rect: None,
                    };
                    
                    editor_state.world.sprites.insert(entity, sprite);
                    editor_state.scene_modified = true;
                    editor_state.console.info(format!("Selected texture: {}", result.sprite_name));
                }
            }
        }

        // ---------------------------------------------------------
        // Handle Requests
        // ---------------------------------------------------------

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
                // Tab management for docking system

                // Check if a tab for this texture is already open
                let mut tab_exists = false;
                // Complex check omitted for now, just switch to it if simplest
                // For now, simpler implementation:
                 editor_state.console.info(format!("Opening sprite editor for: {}", texture_path.display()));
                 // Add tab logic needed here or in main
                 // Actually this logic modifies dock state which we have access to
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
        if new_scene_request {
            editor_state.world = World::new();
            editor_state.entity_names.clear();
            editor_state.selected_entity = None;
            editor_state.current_scene_path = None;
            editor_state.console.info("New scene created".to_string());
        }

        // Save Scene
        if save_request {
            if let Some(path) = editor_state.current_scene_path.clone() {
                 if let Err(e) = editor_state.save_scene(&path) {
                      editor_state.console.error(format!("Failed to save scene: {}", e));
                 } else {
                      editor_state.console.info(format!("Scene saved: {:?}", path));
                 }
            } else {
                save_as_request = true;
            }
        }

        // Save As
        if save_as_request {
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
        if load_request {
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
        if play_request {
            if !editor_state.is_playing {
                 // Start playing
                 editor_state.is_playing = true;
                 editor_state.console.info("▶ Starting Play Mode...".to_string());
                 
                 // Initialize scripts
                 // TODO: Initialize scripts - method not available yet
                 // if let Err(e) = script_engine.initialize_scripts(&mut editor_state.world) {
                 //     editor_state.console.error(format!("Script initialization failed: {}", e));
                 // } else {
                 //     editor_state.console.info("Scripts initialized".to_string());
                 // }
                 
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
                     if let Some(simple_world) = physics.downcast_mut::<PhysicsWorld>() {
                         // Simple physics might not need explicit registration if it uses the world directly
                         editor_state.console.info("Physics (Simple) initialized".to_string());
                     }
                 }
            }
        }

        if stop_request {
            if editor_state.is_playing {
                 editor_state.is_playing = false;
                 editor_state.console.info("⏹ Stopping Play Mode...".to_string());
                 
                 // Reset physics/scripts if needed
                 // Reload scene to reset state?
                 if let Some(path) = editor_state.current_scene_path.clone() {
                      if let Err(e) = editor_state.load_scene(&path) {
                           editor_state.console.error(format!("Failed to reload scene after stop: {}", e));
                      }
                 }
            }
        }
        
        // Physics Debug Drawing (if enabled)
        if editor_state.is_playing && editor_state.show_colliders {
             // Logic to draw colliders would go here
        }

        // Render floating sprite editor windows (only in non-docking mode)
        if !editor_state.use_docking {
            let mut reloaded_sprite_files = Vec::new();
            editor_state.sprite_editor_windows.retain_mut(|window| {
                // Check if file was reloaded during render
                let was_reloaded = window.state_mut().check_and_reload(dt);
                if was_reloaded {
                    reloaded_sprite_files.push(window.state().metadata_path.clone());
                }

                window.render(egui_ctx, &mut editor_state.texture_manager, dt);
                window.is_open
            });

            // Update entities that use reloaded sprite files
            for sprite_file_path in reloaded_sprite_files {
                editor_state.update_entities_using_sprite_file(&sprite_file_path);
            }
        }

        // Game loop update when playing
        if editor_state.is_playing {
            // Update gamepads (but don't clear input yet - scripts need to read it first)
            editor_state.input_system.update_gamepads();
            
            // Update debug draw system
            editor_state.debug_draw.update(dt);

            // Update ground states for Rapier (before running scripts)
            #[cfg(feature = "rapier")]
            {
                if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                    let entities_with_rigidbodies: Vec<_> = editor_state.world.rigidbodies.keys().cloned().collect();
                    for entity in entities_with_rigidbodies {
                        // Cast ray 0.15 units down from player center
                        let is_grounded = rapier_world.raycast_ground(entity, &editor_state.world, 0.15);
                        script_engine.set_ground_state(entity, is_grounded);
                        
                        // Debug draw raycast
                        if let Some(transform) = editor_state.world.transforms.get(&entity) {
                            let collider_half_height = if let Some(collider) = editor_state.world.colliders.get(&entity) {
                                collider.get_world_height(transform.scale[1]) / 2.0
                            } else {
                                0.0
                            };
                            
                            let ray_start = [
                                transform.position[0],
                                transform.position[1] - collider_half_height,
                                transform.position[2],
                            ];
                            let ray_end = [
                                transform.position[0],
                                transform.position[1] - collider_half_height - 0.15,
                                transform.position[2],
                            ];
                            
                            // Green if grounded, Red if not
                            if is_grounded {
                                editor_state.debug_draw.draw_line_green(ray_start, ray_end, 0.0);
                            } else {
                                editor_state.debug_draw.draw_line_red(ray_start, ray_end, 0.0);
                            }
                        }
                    }
                }
            }
            
            // Run scripts FIRST (before physics) so they can set velocities
            let entities_with_scripts: Vec<_> = editor_state.world.scripts.keys().cloned().collect();
            
            for entity in entities_with_scripts {
                if let Some(script) = editor_state.world.scripts.get(&entity) {
                    if script.enabled {
                        let script_name = script.script_name.clone();
                        if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                            let script_path = scripts_folder.join(format!("{}.lua", script_name));
                            if script_path.exists() {
                                let mut log_callback = |msg: String| {
                                    editor_state.console.info(msg);
                                };
                                if let Err(e) = script_engine.run_script(&script_path, entity, &mut editor_state.world, &editor_state.input_system, dt, &mut log_callback) {
                                    editor_state.console.error(format!("Script error for {}: {}", script_name, e));
                                }
                            }
                        }
                    }
                }
            }

            // Transfer debug lines from script engine to debug_draw manager
            let script_debug_lines = script_engine.take_debug_lines();
            for line in script_debug_lines {
                // Convert script DebugLine to editor DebugLine
                let color = egui::Color32::from_rgba_premultiplied(
                    (line.color[0] * 255.0) as u8,
                    (line.color[1] * 255.0) as u8,
                    (line.color[2] * 255.0) as u8,
                    (line.color[3] * 255.0) as u8,
                );
                editor_state.debug_draw.draw_line(line.start, line.end, color, line.duration);
            }
            
            // Process UI commands from Lua scripts
            let ui_commands = script_engine.take_ui_commands();
            for command in ui_commands {
                use script::UICommand;
                match command {
                    UICommand::LoadPrefab { path } => {
                        if let Err(e) = editor_state.ui_manager.load_prefab(&path) {
                            editor_state.console.error(format!("Failed to load prefab '{}': {}", path, e));
                        }
                    }
                    UICommand::ActivatePrefab { path, instance_name } => {
                        if let Err(e) = editor_state.ui_manager.activate_prefab(&path, &instance_name) {
                            editor_state.console.error(format!("Failed to activate prefab '{}': {}", path, e));
                        }
                    }
                    UICommand::DeactivatePrefab { instance_name } => {
                        editor_state.ui_manager.deactivate_prefab(&instance_name);
                    }
                    UICommand::SetText { element_path, text } => {
                        editor_state.ui_manager.set_ui_data(&element_path, text);
                    }
                    UICommand::SetImageFill { element_path, fill_amount } => {
                        if let Some((instance, element)) = element_path.split_once('/') {
                            if let Err(e) = editor_state.ui_manager.set_element_fill(instance, element, fill_amount) {
                                editor_state.console.error(format!("Failed to set fill: {}", e));
                            }
                        }
                    }
                    UICommand::SetColor { element_path, r, g, b, a } => {
                        if let Some((instance, element)) = element_path.split_once('/') {
                            if let Err(e) = editor_state.ui_manager.set_element_color(instance, element, r, g, b, a) {
                                editor_state.console.error(format!("Failed to set color: {}", e));
                            }
                        }
                    }
                    UICommand::ShowElement { element_path } => {
                        if let Some((instance, element)) = element_path.split_once('/') {
                            if let Err(e) = editor_state.ui_manager.show_element(instance, element) {
                                editor_state.console.error(format!("Failed to show element: {}", e));
                            }
                        }
                    }
                    UICommand::HideElement { element_path } => {
                        if let Some((instance, element)) = element_path.split_once('/') {
                            if let Err(e) = editor_state.ui_manager.hide_element(instance, element) {
                                editor_state.console.error(format!("Failed to hide element: {}", e));
                            }
                        }
                    }
                }
            }

            // Accumulate frame time for fixed timestep physics
            *physics_accumulator += dt;
            
            // Update physics with fixed timestep (may run multiple times per frame)
            let mut physics_steps = 0;
            while *physics_accumulator >= fixed_time_step {
                #[cfg(feature = "rapier")]
                {
                    if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                        rapier_world.step(fixed_time_step, &mut editor_state.world);
                    }
                }
                #[cfg(not(feature = "rapier"))]
                {
                    if let Some(simple_world) = physics.downcast_mut::<PhysicsWorld>() {
                        simple_world.step(fixed_time_step, &mut editor_state.world);
                    }
                }
                
                *physics_accumulator -= fixed_time_step;
                physics_steps += 1;
                
                // Safety: prevent spiral of death (too many physics steps)
                if physics_steps >= 5 {
                    *physics_accumulator = 0.0;
                    break;
                }
            }
            
            // Check collisions and call collision callbacks (using simple fallback for now or Rapier events if implemented)
            // Note: For Rapier, we should arguably use its EventQueue, but for now maintaining simple check compatibility
            // This is O(N^2) and should be optimized or replaced by physics engine events
            let entities_with_colliders: Vec<_> = editor_state.world.colliders.keys().cloned().collect();
            for i in 0..entities_with_colliders.len() {
                for j in (i + 1)..entities_with_colliders.len() {
                    let e1 = entities_with_colliders[i];
                    let e2 = entities_with_colliders[j];

                    let collision = {
                         #[cfg(feature = "rapier")]
                         {
                             // TODO: Use Rapier contact events
                             PhysicsWorld::check_collision(&editor_state.world, e1, e2) 
                         }
                         #[cfg(not(feature = "rapier"))]
                         {
                             PhysicsWorld::check_collision(&editor_state.world, e1, e2) 
                         }
                    };

                    if collision {
                        // Call on_collision for e1's script
                        if let Some(script) = editor_state.world.scripts.get(&e1).filter(|s| s.enabled) {
                             let script_name = script.script_name.clone();
                             if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                                 let script_path = scripts_folder.join(format!("{}.lua", script_name));
                                 if script_path.exists() {
                                     if let Err(e) = script_engine.call_collision(&script_path, e1, e2, &mut editor_state.world) {
                                         editor_state.console.error(format!("Collision error {}: {}", script_name, e));
                                     }
                                 }
                             }
                        }

                        // Call on_collision for e2's script
                        if let Some(script) = editor_state.world.scripts.get(&e2).filter(|s| s.enabled) {
                             let script_name = script.script_name.clone();
                             if let Some(scripts_folder) = editor_state.get_scripts_folder() {
                                 let script_path = scripts_folder.join(format!("{}.lua", script_name));
                                 if script_path.exists() {
                                     if let Err(e) = script_engine.call_collision(&script_path, e2, e1, &mut editor_state.world) {
                                         editor_state.console.error(format!("Collision error {}: {}", script_name, e));
                                     }
                                 }
                             }
                        }
                    }
                }
            }

            // Clear per-frame input state AFTER scripts have run
            editor_state.input_system.begin_frame();
        }
    }
}

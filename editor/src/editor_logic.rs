use engine_core::EngineContext;
use engine_core::assets::AssetLoader;
use crate::states::{AppState, EditorState};
use crate::ui::EditorUI;
use script::ScriptEngine;
use crate::systems::{play_mode::PlayModeSystem, menu_commands::MenuCommandSystem};
use crate::ui::dialogs::ExitDialog;
use wgpu;
use egui_wgpu;

pub struct EditorLogic;

impl EditorLogic {
    pub fn handle_editor_frame(
        egui_ctx: &egui::Context,
        _app_state: &mut AppState,
        editor_state: &mut EditorState,
        _ctx: &mut EngineContext,
        script_engine: &mut ScriptEngine,
        physics: &mut dyn std::any::Any, // Passed as Any because type varies by feature
        physics_accumulator: &mut f32,
        fixed_time_step: f32,
        dt: f32,
        game_view_renderer: &mut crate::game_view_renderer::GameViewRenderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        egui_renderer: &mut egui_wgpu::Renderer,
        scene_view_renderer: &mut crate::scene_view_renderer::SceneViewRenderer,
        mesh_renderer: &render::MeshRenderer,
        render_texture_manager: &mut render::TextureManager,
        asset_loader: &dyn engine_core::assets::AssetLoader,
        render_cache: &mut engine::runtime::render_system::RenderCache,
    ) {
        let mut save_request = false;
        let mut save_as_request = false;
        let mut load_request = false;
        let mut load_file_request: Option<std::path::PathBuf> = None;
        let mut new_scene_request = false;
        let mut play_request = false;
        let mut stop_request = false;
        let mut edit_script_request: Option<String> = None;

        // Note: Q/W/E/R/F keyboard shortcuts are handled in app.rs (InputSystem/Shortcuts)
        // because we need better control over input priority and context
        
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
                game_view_renderer,
                device,
                queue,
                egui_renderer,
                scene_view_renderer,
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
                &mut editor_state.reload_mesh_assets_request,
                asset_loader,
                render_cache,
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
                scene_view_renderer,
                egui_renderer,
                device,
                queue,
                &mut editor_state.reload_mesh_assets_request,
                asset_loader,
                render_cache,
             );
        }

        // ---------------------------------------------------------
        // Handle Dialogs & Popups
        // ---------------------------------------------------------

        // Create Prefab Dialog
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
        
        // Exit Confirmation Dialog
        ExitDialog::render(egui_ctx, editor_state);

        // Export Game Dialog
        crate::ui::export_dialog::ExportGameDialog::render(egui_ctx, editor_state);

        // Sprite Picker Dialog
        EditorLogic::handle_sprite_picker(egui_ctx, editor_state);

        // ---------------------------------------------------------
        // Handle Logic & Systems
        // ---------------------------------------------------------
        
        // Handle Menu Commands (Save, Load, New, Play, etc.)
        MenuCommandSystem::handle_commands(
            editor_state,
            script_engine,
            physics,
            &mut save_request,
            &mut save_as_request,
            &mut load_request,
            &mut load_file_request,
            &mut new_scene_request,
            &mut play_request,
            &mut stop_request,
            &mut edit_script_request,
            device,
            queue,
            render_texture_manager,
            mesh_renderer,
            asset_loader,
            render_cache,
        );

        // [SCENE POST-PROCESSING]
        // If a scene was loaded (via Menu, File, or Stop Play), we must check for Asset Meshes (GLTF)
        // and load them into the world.
        // Also reload if requested from Inspector (when mesh type(No change)s to/from Asset)
        if load_request || load_file_request.is_some() || (stop_request && !editor_state.is_playing) || editor_state.reload_mesh_assets_request {
             if let Some(project_path) = &editor_state.current_project_path {
                 use engine::runtime::render_system::post_process_asset_meshes;
                 post_process_asset_meshes(
                     render_cache,
                     project_path,
                     &mut editor_state.world,
                     device,
                     queue,
                     render_texture_manager,
                     mesh_renderer,
                     asset_loader,
                 );

                 // Load sprite textures into WGPU TextureManager for 3D scene view rendering
                 EditorLogic::load_scene_textures(
                     &editor_state.world,
                     project_path,
                     device,
                     queue,
                     render_texture_manager,
                     asset_loader,
                 );

                 // Reset the request flag
                 editor_state.reload_mesh_assets_request = false;
             }
        }

        // Render standalone floating windows (only in non-docking mode)
        EditorLogic::handle_floating_windows(egui_ctx, editor_state, dt);

        // Handle Play Mode Logic (Physics, Scripts, Collisions)
        PlayModeSystem::update(
            editor_state,
            _ctx,
            script_engine,
            physics,
            physics_accumulator,
            fixed_time_step,
            dt,
            asset_loader,
        );
    }

    fn handle_sprite_picker(egui_ctx: &egui::Context, editor_state: &mut EditorState) {
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
                                    asset_id: None,
                                    width: sprite_def.width as f32,
                                    height: sprite_def.height as f32,
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    billboard: false,
                                    flip_x: false,
                                    flip_y: false,
                                    sprite_rect: Some([sprite_def.x, sprite_def.y, sprite_def.width, sprite_def.height]),
                                    pixels_per_unit: 100.0,
                                    sorting_layer: "Default".to_string(),
                                    order_in_layer: 0,
                                    rendering_layer_mask: 1,
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
                        asset_id: None,
                        width: 1.0,
                        height: 1.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        billboard: false,
                        flip_x: false,
                        flip_y: false,
                        pixels_per_unit: 100.0,
                        sprite_rect: None,
                        sorting_layer: "Default".to_string(),
                        order_in_layer: 0,
                        rendering_layer_mask: 1,
                    };
                    
                    editor_state.world.sprites.insert(entity, sprite);
                    editor_state.scene_modified = true;
                    editor_state.console.info(format!("Selected texture: {}", result.sprite_name));
                }
            }
        }
    }

    /// Load all scene textures (sprites and tilemaps) into WGPU TextureManager
    /// This is needed for rendering in 3D scene view and Game View
    fn load_scene_textures(
        world: &ecs::World,
        project_path: &std::path::Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_manager: &mut render::TextureManager,
        asset_loader: &dyn engine_core::assets::AssetLoader,
    ) {
        use std::collections::HashSet;

        // Collect unique texture paths from all sprites and tilesets
        // We use the path string as the Texture ID for WGPU lookups
        let mut texture_paths = HashSet::new();
        
        // 1. Sprites
        for sprite in world.sprites.values() {
            if !sprite.texture_id.is_empty() {
                texture_paths.insert(sprite.texture_id.clone());
            }
        }

        // 2. Tilesets
        for tileset in world.tilesets.values() {
            if !tileset.texture_path.is_empty() {
                // Ensure we use the exact string render_system expects (tileset.texture_path)
                texture_paths.insert(tileset.texture_path.clone());
            }
        }

        println!("DEBUG: Loading {} unique textures for WGPU. Project Path: {}", texture_paths.len(), project_path.display());

        // Potential subdirectories to search for textures
        let search_dirs = ["", "assets", "atlas", "tilemaps/atlas", "levels/atlas"];

        // Load each texture into WGPU TextureManager
        for texture_id in texture_paths {
            // Skip if already loaded
            if texture_manager.get_texture(&texture_id).is_some() {
                continue;
            }

            let mut found = false;
            for dir in search_dirs {
                let check_path = if dir.is_empty() {
                    project_path.join(&texture_id)
                } else {
                    project_path.join(dir).join(&texture_id)
                };
                
                let check_path_str = check_path.to_str().unwrap_or("");

                // Attempt load via AssetLoader
                if let Ok(bytes) = pollster::block_on(asset_loader.load_binary(check_path_str)) {
                     match texture_manager.load_texture_from_bytes(device, queue, &bytes, &texture_id) {
                        Ok(_) => {
                            println!("DEBUG: ✓ Loaded texture for WGPU: {} (found in {})", texture_id, dir);
                        }
                        Err(e) => {
                            println!("DEBUG: ✗ Failed to load texture bytes {}: {}", check_path.display(), e);
                        }
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                 println!("DEBUG: ✗ Texture not found ANYWHERE: {}", texture_id);
            }
        }
    }

    fn handle_floating_windows(egui_ctx: &egui::Context, editor_state: &mut EditorState, dt: f32) {
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
    }
}

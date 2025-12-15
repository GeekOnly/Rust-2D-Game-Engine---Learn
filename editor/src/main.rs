// mod editor; // Removed (local crate)
// mod runtime; // Moved to engine
// mod texture_manager; // Moved to engine
// mod ui_manager; // Moved to engine

use anyhow::Result;
use engine_core::{EngineContext, EngineModule, project::ProjectManager};
use ecs::{World, Entity, Transform, Sprite, Collider, EntityTag};
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use render::RenderModule;
// use ::editor::EditorModule as EditorMod; // Removing for now if missing
use editor::ui::{EditorUI, TransformTool};
use editor::states::{AppState, LauncherState, EditorState, EditorAction};
use editor::shortcuts::EditorShortcut; // Explicitly import if needed
use editor::console::Console; // Explicit import
use engine::runtime;
use engine::texture_manager;
use engine::ui_manager;
use editor::theme::UnityTheme;
use input::Key;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Sample game removed - use projects/ folder for game content

fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Game Engine...");
    log::info!("=== Rust 2D Game Engine Starting ===");
    log::info!("Logging initialized");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rust 2D Game Engine - Launcher")
        .with_inner_size(winit::dpi::LogicalSize::new(1000, 700))
        .build(&event_loop)?;

    let mut app_state = AppState::Launcher;
    let mut launcher_state = LauncherState::new()?;
    let mut editor_state = EditorState::new();

    let mut ctx = EngineContext::new();
    // Sample game removed - use projects/ folder for game content

    let mut script_engine = ScriptEngine::new()?;
    #[cfg(feature = "rapier")]
    let mut physics = RapierPhysicsWorld::new();
    #[cfg(not(feature = "rapier"))]
    let mut physics = PhysicsWorld::new();

    let mut last_frame_time = std::time::Instant::now();
    
    // Fixed timestep for physics (60 Hz = 0.0167 seconds per step)
    const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
    let mut physics_accumulator: f32 = 0.0;

    // Initialize renderer with window
    let mut renderer = pollster::block_on(RenderModule::new(&window))?;
    // let _editor = EditorMod::new();

    // egui setup
    let egui_ctx = egui::Context::default();
    
    // Apply Unity-like theme (dark mode)
    editor::UnityTheme::apply(&egui_ctx);
    
    // Force dark mode for egui_dock
    egui_ctx.set_visuals(egui::Visuals::dark());
    
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
                    } => {
                        // If in editor and scene is modified, show exit dialog
                        if app_state == AppState::Editor && editor_state.scene_modified {
                            editor_state.show_exit_dialog = true;
                        } else {
                            target.exit();
                        }
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        // Update modifiers for shortcut manager (from egui context)
                        if app_state == AppState::Editor {
                            let modifiers = egui_ctx.input(|i| i.modifiers);
                            editor_state.shortcut_manager.update_modifiers(modifiers);
                        }
                        
                        // Update InputSystem
                        if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                            let key_str = format!("{:?}", key_code);
                            
                            // Debug: log Space key presses in play mode
                            if app_state == AppState::Editor && editor_state.is_playing && key_str.contains("Space") && key_event.state == ElementState::Pressed {
                                editor_state.console.debug(format!("🔍 Space key detected: key_str={}", key_str));
                            }
                            
                            if let Some(key) = Key::from_str(&key_str) {
                                if key_event.state == ElementState::Pressed {
                                    ctx.input.press_key(key);
                                    // Also update editor input system when in play mode
                                    if app_state == AppState::Editor && editor_state.is_playing {
                                        editor_state.input_system.press_key(key);
                                        // Debug: log key press
                                        if key_str.contains("Space") {
                                            editor_state.console.debug(format!("✅ Space key pressed in input_system"));
                                        }
                                    }
                                } else {
                                    ctx.input.release_key(key);
                                    // Also update editor input system when in play mode
                                    if app_state == AppState::Editor && editor_state.is_playing {
                                        editor_state.input_system.release_key(key);
                                    }
                                }
                            } else if app_state == AppState::Editor && editor_state.is_playing && key_str.contains("Space") {
                                editor_state.console.warning(format!("❌ Space key not mapped: key_str={}", key_str));
                            }
                        }

                        // Handle editor shortcuts (only when not playing)
                        if app_state == AppState::Editor && !editor_state.is_playing {
                            if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                                if key_event.state == ElementState::Pressed {
                                    if let Some(shortcut) = editor_state.shortcut_manager.check_shortcut(key_code) {
                                        use crate::EditorShortcut;
                                        match shortcut {
                                            EditorShortcut::ViewTool => {
                                                editor_state.current_tool = TransformTool::View;
                                                editor_state.console.info("Tool: View (Q)".to_string());
                                            }
                                            EditorShortcut::MoveTool => {
                                                editor_state.current_tool = TransformTool::Move;
                                                editor_state.console.info("Tool: Move (W)".to_string());
                                            }
                                            EditorShortcut::RotateTool => {
                                                editor_state.current_tool = TransformTool::Rotate;
                                                editor_state.console.info("Tool: Rotate (E)".to_string());
                                            }
                                            EditorShortcut::ScaleTool => {
                                                editor_state.current_tool = TransformTool::Scale;
                                                editor_state.console.info("Tool: Scale (R)".to_string());
                                            }
                                            EditorShortcut::Delete => {
                                                if let Some(entity) = editor_state.selected_entity {
                                                    editor_state.world.despawn(entity);
                                                    editor_state.entity_names.remove(&entity);
                                                    editor_state.selected_entity = None;
                                                    editor_state.scene_modified = true;
                                                    editor_state.console.info("Entity deleted".to_string());
                                                }
                                            }
                                            EditorShortcut::FrameSelected => {
                                                if let Some(entity) = editor_state.selected_entity {
                                                    if let Some(transform) = editor_state.world.transforms.get(&entity) {
                                                        let pos = glam::Vec2::new(transform.x(), transform.y());
                                                        let size = if let Some(sprite) = editor_state.world.sprites.get(&entity) {
                                                            glam::Vec2::new(sprite.width, sprite.height)
                                                        } else {
                                                            glam::Vec2::new(50.0, 50.0)
                                                        };
                                                        let viewport = glam::Vec2::new(800.0, 600.0);
                                                        let size_scalar = size.length(); // Convert Vec2 to scalar
                                                        editor_state.scene_camera.frame_object(pos, size_scalar, viewport);
                                                        editor_state.console.info("Framed selected object (F)".to_string());
                                                    }
                                                }
                                            }
                                            EditorShortcut::ToggleGrid => {
                                                editor_state.scene_grid.toggle();
                                                let status = if editor_state.scene_grid.enabled { "ON" } else { "OFF" };
                                                editor_state.console.info(format!("Grid: {}", status));
                                            }
                                            EditorShortcut::Duplicate => {
                                                if let Some(entity) = editor_state.selected_entity {
                                                    if let Some(new_entity) = editor_state.clipboard.duplicate_entity(
                                                        entity,
                                                        &mut editor_state.world,
                                                        &mut editor_state.entity_names
                                                    ) {
                                                        editor_state.selected_entity = Some(new_entity);
                                                        editor_state.scene_modified = true;
                                                        editor_state.console.info("Entity duplicated (Ctrl+D)".to_string());
                                                    }
                                                }
                                            }
                                            EditorShortcut::Copy => {
                                                if let Some(entity) = editor_state.selected_entity {
                                                    editor_state.clipboard.copy_entity(
                                                        entity,
                                                        &editor_state.world,
                                                        &editor_state.entity_names
                                                    );
                                                    editor_state.console.info("Entity copied (Ctrl+C)".to_string());
                                                }
                                            }
                                            EditorShortcut::Paste => {
                                                let new_entities = editor_state.clipboard.paste(
                                                    &mut editor_state.world,
                                                    &mut editor_state.entity_names,
                                                    Some([10.0, 10.0, 0.0]) // Offset by 10 pixels
                                                );
                                                if let Some(&new_entity) = new_entities.first() {
                                                    editor_state.selected_entity = Some(new_entity);
                                                    editor_state.scene_modified = true;
                                                    editor_state.console.info("Entity pasted (Ctrl+V)".to_string());
                                                }
                                            }
                                            EditorShortcut::Undo => {
                                                if editor_state.undo_stack.undo(
                                                    &mut editor_state.world,
                                                    &mut editor_state.entity_names
                                                ) {
                                                    editor_state.scene_modified = true;
                                                    if let Some(desc) = editor_state.undo_stack.undo_description() {
                                                        editor_state.console.info(format!("Undo: {} (Ctrl+Z)", desc));
                                                    } else {
                                                        editor_state.console.info("Undo (Ctrl+Z)".to_string());
                                                    }
                                                } else {
                                                    editor_state.console.warning("Nothing to undo".to_string());
                                                }
                                            }
                                            EditorShortcut::Redo => {
                                                if editor_state.undo_stack.redo(
                                                    &mut editor_state.world,
                                                    &mut editor_state.entity_names
                                                ) {
                                                    editor_state.scene_modified = true;
                                                    if let Some(desc) = editor_state.undo_stack.redo_description() {
                                                        editor_state.console.info(format!("Redo: {} (Ctrl+Y)", desc));
                                                    } else {
                                                        editor_state.console.info("Redo (Ctrl+Y)".to_string());
                                                    }
                                                } else {
                                                    editor_state.console.warning("Nothing to redo".to_string());
                                                }
                                            }
                                            EditorShortcut::SaveScene => {
                                                // Save scene (Ctrl+S)
                                                if let Some(ref path) = editor_state.current_scene_path.clone() {
                                                    if let Err(e) = editor_state.save_scene(path) {
                                                        editor_state.console.error(format!("Failed to save: {}", e));
                                                    } else {
                                                        editor_state.console.info("Scene saved (Ctrl+S)".to_string());
                                                        editor_state.autosave.reset(); // Reset auto-save timer
                                                    }
                                                } else {
                                                    editor_state.console.warning("No scene to save. Use File → Save Scene As...".to_string());
                                                }
                                            }
                                            EditorShortcut::Exit => {
                                                // Exit editor (Ctrl+Q)
                                                editor_state.show_exit_dialog = true;
                                            }
                                            _ => {
                                                // Other shortcuts not yet implemented
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Pass keyboard input to game state only in Playing mode
                        if app_state == AppState::Playing {
                            // Input is now handled via ctx.input in update()
                        } else if app_state == AppState::Editor && editor_state.is_playing {
                            // Input is now handled above in the main InputSystem update section
                            // Track in legacy keyboard_state HashMap for backward compatibility
                            if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                                let key_name = format!("{:?}", key_code);
                                let is_pressed = key_event.state == winit::event::ElementState::Pressed;
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

                        // Sample game removed - use projects/ folder for game content

                        // Egui frame setup
                        let raw_input = egui_state.take_egui_input(&window);
                        
                        // Re-apply Unity theme every frame to prevent override
                        editor::UnityTheme::apply(&egui_ctx);
                        
                        egui_ctx.begin_frame(raw_input);

                        // Auto-save logic (only in editor mode)
                        if app_state == AppState::Editor && !editor_state.is_playing {
                            if editor_state.autosave.should_save() && editor_state.scene_modified {
                                if let Some(scene_path) = &editor_state.current_scene_path {
                                    let autosave_path = editor_state.autosave.create_autosave_path(scene_path);
                                    if let Ok(json) = editor_state.world.save_to_json() {
                                        if std::fs::write(&autosave_path, json).is_ok() {
                                            editor_state.autosave.mark_saved();
                                            editor_state.console.info(format!("Auto-saved to {}", autosave_path.display()));
                                            let _ = editor_state.autosave.cleanup_old_autosaves(scene_path);
                                        }
                                    }
                                }
                            }
                        }

                        // Render UI based on app state
                        match app_state {
                            AppState::Launcher => {
                                // Launcher UI
                                editor::ui::launcher_window::LauncherWindow::render(
                                    &egui_ctx,
                                    &mut app_state,
                                    &mut launcher_state,
                                    &mut editor_state,
                                );
                            }
                            AppState::Playing => {
                                editor::ui::game_window::GameWindow::render(
                                    &egui_ctx,
                                    &mut app_state,
                                );
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

                                // Initialize asset manager if not yet initialized
                                if editor_state.asset_manager.is_none() {
                                    if let Some(ref project_path) = editor_state.current_project_path {
                                        editor_state.asset_manager = Some(editor::AssetManager::new(project_path));
                                    }
                                }
                                
                                // Initialize hot-reload watcher if not yet initialized
                                if editor_state.map_manager.hot_reload_watcher.is_none() && editor_state.map_manager.hot_reload_enabled {
                                    if let Err(e) = editor_state.map_manager.enable_hot_reload() {
                                        editor_state.console.warning(format!("Failed to enable hot-reload: {}", e));
                                    }
                                }
                                
                                // Set map manager project path if not yet set
                                if editor_state.map_manager.project_path.is_none() {
                                    if let Some(ref project_path) = editor_state.current_project_path {
                                        editor_state.map_manager.set_project_path(project_path.clone());
                                    }
                                }
                                
                                // Set prefab manager project path if not yet set
                                if editor_state.prefab_manager.project_path.is_none() {
                                    if let Some(ref project_path) = editor_state.current_project_path {
                                        editor_state.prefab_manager.set_project_path(project_path.clone());
                                    }
                                }
                                
                                // Handle layout change request
                                if let Some(layout_name) = editor_state.layout_request.take() {
                                    if layout_name == "save_default" {
                                        // Save current layout as default
                                        editor_state.save_default_layout();
                                        editor_state.console.info(format!("Saved '{}' as default layout", editor_state.current_layout_name));
                                    } else if layout_name == "save_as" {
                                        // Show save layout dialog
                                        editor_state.show_save_layout_dialog = true;
                                        editor_state.save_layout_name.clear();
                                    } else if layout_name.starts_with("load:") {
                                        // Load built-in layout
                                        let name = layout_name.strip_prefix("load:").unwrap();
                                        editor_state.dock_state = editor::ui::get_layout_by_name(name);
                                        editor_state.current_layout_name = name.to_string();
                                        editor_state.current_layout_type = name.to_string();
                                        editor_state.console.info(format!("Changed to '{}' layout", name));
                                    } else if layout_name.starts_with("custom:") {
                                        // Load custom layout
                                        let name = layout_name.strip_prefix("custom:").unwrap();
                                        if let Some(project_path) = &editor_state.current_project_path {
                                            // Try to load full state first (new format)
                                            if let Some(dock_state) = editor::ui::load_custom_layout_state(name, project_path) {
                                                editor_state.dock_state = dock_state;
                                                editor_state.current_layout_name = name.to_string();
                                                editor_state.current_layout_type = "custom".to_string();
                                                editor_state.console.info(format!("Loaded custom layout '{}'", name));
                                            } else {
                                                // Fallback to legacy format
                                                let layouts = editor::ui::load_custom_layouts(project_path);
                                                if let Some((_, layout_type)) = layouts.iter().find(|(n, _)| n == name) {
                                                    if layout_type != "custom" {
                                                        editor_state.dock_state = editor::ui::get_layout_by_name(layout_type);
                                                        editor_state.current_layout_name = name.to_string();
                                                        editor_state.current_layout_type = layout_type.clone();
                                                        editor_state.console.info(format!("Loaded custom layout '{}' (legacy)", name));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Save Layout Dialog
                                if editor_state.show_save_layout_dialog {
                                    egui::Window::new("Save Layout As")
                                        .collapsible(false)
                                        .resizable(false)
                                        .show(&egui_ctx, |ui| {
                                            ui.label("Layout Name:");
                                            ui.text_edit_singleline(&mut editor_state.save_layout_name);
                                            
                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Save").clicked() && !editor_state.save_layout_name.is_empty() {
                                                    if let Some(ref project_path) = editor_state.current_project_path {
                                                        // Save custom layout with full state (including panel sizes)
                                                        if let Err(e) = editor::ui::save_custom_layout_state(
                                                            &editor_state.save_layout_name,
                                                            &editor_state.dock_state,
                                                            project_path
                                                        ) {
                                                            editor_state.console.error(format!("Failed to save layout: {}", e));
                                                        } else {
                                                            // Update current layout name to the saved name
                                                            let saved_name = editor_state.save_layout_name.clone();
                                                            editor_state.current_layout_name = saved_name.clone();
                                                            editor_state.current_layout_type = "custom".to_string();
                                                            editor_state.console.info(format!("Saved layout as '{}'", saved_name));
                                                            editor_state.show_save_layout_dialog = false;
                                                        }
                                                    }
                                                }
                                                if ui.button("Cancel").clicked() {
                                                    editor_state.show_save_layout_dialog = false;
                                                }
                                            });
                                        });
                                }

                                // Process hot-reload for LDtk maps
                                let reloaded_maps = editor_state.map_manager.process_hot_reload(&mut editor_state.world);
                                if !reloaded_maps.is_empty() {
                                    for map_path in &reloaded_maps {
                                        editor_state.console.info(format!("🔄 Hot-reloaded map: {:?}", map_path));
                                    }
                                }
                                
                                // Display hot-reload error if any
                                if let Some(error) = editor_state.map_manager.get_last_hot_reload_error() {
                                    editor_state.console.error(format!("Hot-reload error: {}", error));
                                    // Clear error after displaying
                                    editor_state.map_manager.clear_hot_reload_error();
                                }

                                // Main Editor Logic (excludes initialization and layout/hot-reload setup above)
                                editor::editor_logic::EditorLogic::handle_editor_frame(
                                    &egui_ctx,
                                    &mut app_state,
                                    &mut editor_state,
                                    &mut ctx,
                                    &mut script_engine,
                                    &mut physics,
                                    &mut physics_accumulator,
                                    FIXED_TIMESTEP,
                                    dt,
                                );
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
                // Check if we should exit
                if editor_state.should_exit {
                    target.exit();
                }
                
                // Always request redraw for continuous updates
                window.request_redraw();
                
                // When playing, ensure continuous updates
                if editor_state.is_playing {
                    target.set_control_flow(ControlFlow::Poll);
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

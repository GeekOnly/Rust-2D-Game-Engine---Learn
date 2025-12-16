use anyhow::Result;
use engine_core::{EngineContext, EngineModule, project::ProjectManager};
use ecs::{World, Entity, Transform, Sprite, Collider, EntityTag};
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use render::RenderModule;
use crate::ui::{EditorUI, TransformTool};
use crate::states::{AppState, LauncherState, EditorState, EditorAction};
use crate::shortcuts::EditorShortcut;
use crate::console::Console;
use engine::runtime;
use engine::texture_manager;
use engine::ui_manager;
use crate::theme::UnityTheme;
use input::Key;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
use std::str::FromStr;

pub struct EditorApp {
    pub window: Window,
    pub app_state: AppState,
    pub launcher_state: LauncherState,
    pub editor_state: EditorState,
    pub ctx: EngineContext,
    pub script_engine: ScriptEngine,
    #[cfg(feature = "rapier")]
    pub physics: RapierPhysicsWorld,
    #[cfg(not(feature = "rapier"))]
    pub physics: PhysicsWorld,
    pub renderer: RenderModule,
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
    pub physics_accumulator: f32,
    pub fixed_timestep: f32,
}

impl EditorApp {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title("Rust 2D Game Engine - Launcher")
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 700))
            .build(event_loop)?;

        let app_state = AppState::Launcher;
        let launcher_state = LauncherState::new()?;
        let editor_state = EditorState::new();

        let ctx = EngineContext::new();
        // Sample game removed - use projects/ folder for game content

        let script_engine = ScriptEngine::new()?;
        #[cfg(feature = "rapier")]
        let physics = RapierPhysicsWorld::new();
        #[cfg(not(feature = "rapier"))]
        let physics = PhysicsWorld::new();

        // Initialize renderer with window
        let renderer = pollster::block_on(RenderModule::new(&window))?;

        // egui setup
        let egui_ctx = egui::Context::default();
        
        // Apply Unity-like theme (dark mode)
        UnityTheme::apply(&egui_ctx);
        
        // Force dark mode for egui_dock
        egui_ctx.set_visuals(egui::Visuals::dark());
        
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.config.format,
            None,
            1,
        );

        Ok(Self {
            window,
            app_state,
            launcher_state,
            editor_state,
            ctx,
            script_engine,
            physics,
            renderer,
            egui_ctx,
            egui_state,
            egui_renderer,
            physics_accumulator: 0.0,
            fixed_timestep: 1.0 / 60.0,
        })
    }

    pub fn handle_event(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                // Pass events to egui
                let _ = self.egui_state.on_window_event(&self.window, event);

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
                        if self.app_state == AppState::Editor && self.editor_state.scene_modified {
                            self.editor_state.show_exit_dialog = true;
                        } else {
                            target.exit();
                        }
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        self.handle_keyboard_input(key_event);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.ctx.input.set_mouse_position(position.x as f32, position.y as f32);
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
                                self.ctx.input.press_mouse_button(mb);
                            } else {
                                self.ctx.input.release_mouse_button(mb);
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                       // Handled by Resized typically
                    }
                    WindowEvent::RedrawRequested => {
                        self.render(target);
                    }
                    _ => {}
                }
            },
            Event::AboutToWait => {
                // Check if we should exit
                if self.editor_state.should_exit {
                    target.exit();
                }
                
                // Always request redraw for continuous updates
                self.window.request_redraw();
                
                // When playing, ensure continuous updates
                if self.editor_state.is_playing {
                    target.set_control_flow(ControlFlow::Poll);
                }
            }
            _ => {}
        }
    }

    fn handle_keyboard_input(&mut self, key_event: &KeyEvent) {
        // Update modifiers for shortcut manager (from egui context)
        if self.app_state == AppState::Editor {
            let modifiers = self.egui_ctx.input(|i| i.modifiers);
            self.editor_state.shortcut_manager.update_modifiers(modifiers);
        }
        
        // Update InputSystem
        if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
            let key_str = format!("{:?}", key_code);
            
            // Debug: log Space key presses in play mode
            if self.app_state == AppState::Editor && self.editor_state.is_playing && key_str.contains("Space") && key_event.state == ElementState::Pressed {
                self.editor_state.console.debug(format!("🔍 Space key detected: key_str={}", key_str));
            }
            
            if let Some(key) = Key::from_str(&key_str) {
                if key_event.state == ElementState::Pressed {
                    self.ctx.input.press_key(key);
                    // Also update editor input system when in play mode
                    if self.app_state == AppState::Editor && self.editor_state.is_playing {
                        self.editor_state.input_system.press_key(key);
                        // Debug: log key press
                        if key_str.contains("Space") {
                            self.editor_state.console.debug(format!("✅ Space key pressed in input_system"));
                        }
                    }
                } else {
                    self.ctx.input.release_key(key);
                    // Also update editor input system when in play mode
                    if self.app_state == AppState::Editor && self.editor_state.is_playing {
                        self.editor_state.input_system.release_key(key);
                    }
                }
            } else if self.app_state == AppState::Editor && self.editor_state.is_playing && key_str.contains("Space") {
                self.editor_state.console.warning(format!("❌ Space key not mapped: key_str={}", key_str));
            }
        }

        // Handle editor shortcuts (only when not playing)
        if self.app_state == AppState::Editor && !self.editor_state.is_playing {
            self.handle_editor_shortcuts(key_event);
        }

        // Pass keyboard input to game state only in Playing mode
        if self.app_state == AppState::Playing {
            // Input is now handled via ctx.input in update()
        } else if self.app_state == AppState::Editor && self.editor_state.is_playing {
            // Input is now handled above in the main InputSystem update section
            // Track in legacy keyboard_state HashMap for backward compatibility
            if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
                let key_name = format!("{:?}", key_code);
                let is_pressed = key_event.state == winit::event::ElementState::Pressed;
                self.editor_state.keyboard_state.insert(key_name, is_pressed);
            }
        }
    }

    fn handle_editor_shortcuts(&mut self, key_event: &KeyEvent) {
        if let winit::keyboard::PhysicalKey::Code(key_code) = key_event.physical_key {
            if key_event.state == ElementState::Pressed {
                if let Some(shortcut) = self.editor_state.shortcut_manager.check_shortcut(key_code) {
                    use crate::EditorShortcut;
                    match shortcut {
                        EditorShortcut::ViewTool => {
                            self.editor_state.current_tool = TransformTool::View;
                            self.editor_state.console.info("Tool: View (Q)".to_string());
                        }
                        EditorShortcut::MoveTool => {
                            self.editor_state.current_tool = TransformTool::Move;
                            self.editor_state.console.info("Tool: Move (W)".to_string());
                        }
                        EditorShortcut::RotateTool => {
                            self.editor_state.current_tool = TransformTool::Rotate;
                            self.editor_state.console.info("Tool: Rotate (E)".to_string());
                        }
                        EditorShortcut::ScaleTool => {
                            self.editor_state.current_tool = TransformTool::Scale;
                            self.editor_state.console.info("Tool: Scale (R)".to_string());
                        }
                        EditorShortcut::Delete => {
                            if let Some(entity) = self.editor_state.selected_entity {
                                self.editor_state.world.despawn(entity);
                                self.editor_state.entity_names.remove(&entity);
                                self.editor_state.selected_entity = None;
                                self.editor_state.scene_modified = true;
                                self.editor_state.console.info("Entity deleted".to_string());
                            }
                        }
                        EditorShortcut::FrameSelected => {
                            if let Some(entity) = self.editor_state.selected_entity {
                                if let Some(transform) = self.editor_state.world.transforms.get(&entity) {
                                    let pos = glam::Vec2::new(transform.x(), transform.y());
                                    let size = if let Some(sprite) = self.editor_state.world.sprites.get(&entity) {
                                        glam::Vec2::new(sprite.width, sprite.height)
                                    } else {
                                        glam::Vec2::new(50.0, 50.0)
                                    };
                                    let viewport = glam::Vec2::new(800.0, 600.0);
                                    let size_scalar = size.length(); // Convert Vec2 to scalar
                                    self.editor_state.scene_camera.frame_object(pos, size_scalar, viewport);
                                    self.editor_state.console.info("Framed selected object (F)".to_string());
                                }
                            }
                        }
                        EditorShortcut::ToggleGrid => {
                            self.editor_state.scene_grid.toggle();
                            let status = if self.editor_state.scene_grid.enabled { "ON" } else { "OFF" };
                            self.editor_state.console.info(format!("Grid: {}", status));
                        }
                        EditorShortcut::Duplicate => {
                            if let Some(entity) = self.editor_state.selected_entity {
                                if let Some(new_entity) = self.editor_state.clipboard.duplicate_entity(
                                    entity,
                                    &mut self.editor_state.world,
                                    &mut self.editor_state.entity_names
                                ) {
                                    self.editor_state.selected_entity = Some(new_entity);
                                    self.editor_state.scene_modified = true;
                                    self.editor_state.console.info("Entity duplicated (Ctrl+D)".to_string());
                                }
                            }
                        }
                        EditorShortcut::Copy => {
                            if let Some(entity) = self.editor_state.selected_entity {
                                self.editor_state.clipboard.copy_entity(
                                    entity,
                                    &self.editor_state.world,
                                    &self.editor_state.entity_names
                                );
                                self.editor_state.console.info("Entity copied (Ctrl+C)".to_string());
                            }
                        }
                        EditorShortcut::Paste => {
                            let new_entities = self.editor_state.clipboard.paste(
                                &mut self.editor_state.world,
                                &mut self.editor_state.entity_names,
                                Some([10.0, 10.0, 0.0]) // Offset by 10 pixels
                            );
                            if let Some(&new_entity) = new_entities.first() {
                                self.editor_state.selected_entity = Some(new_entity);
                                self.editor_state.scene_modified = true;
                                self.editor_state.console.info("Entity pasted (Ctrl+V)".to_string());
                            }
                        }
                        EditorShortcut::Undo => {
                            if self.editor_state.undo_stack.undo(
                                &mut self.editor_state.world,
                                &mut self.editor_state.entity_names
                            ) {
                                self.editor_state.scene_modified = true;
                                if let Some(desc) = self.editor_state.undo_stack.undo_description() {
                                    self.editor_state.console.info(format!("Undo: {} (Ctrl+Z)", desc));
                                } else {
                                    self.editor_state.console.info("Undo (Ctrl+Z)".to_string());
                                }
                            } else {
                                self.editor_state.console.warning("Nothing to undo".to_string());
                            }
                        }
                        EditorShortcut::Redo => {
                            if self.editor_state.undo_stack.redo(
                                &mut self.editor_state.world,
                                &mut self.editor_state.entity_names
                            ) {
                                self.editor_state.scene_modified = true;
                                if let Some(desc) = self.editor_state.undo_stack.redo_description() {
                                    self.editor_state.console.info(format!("Redo: {} (Ctrl+Y)", desc));
                                } else {
                                    self.editor_state.console.info("Redo (Ctrl+Y)".to_string());
                                }
                            } else {
                                self.editor_state.console.warning("Nothing to redo".to_string());
                            }
                        }
                        EditorShortcut::SaveScene => {
                            // Save scene (Ctrl+S)
                            if let Some(ref path) = self.editor_state.current_scene_path.clone() {
                                if let Err(e) = self.editor_state.save_scene(path) {
                                    self.editor_state.console.error(format!("Failed to save: {}", e));
                                } else {
                                    self.editor_state.console.info("Scene saved (Ctrl+S)".to_string());
                                    self.editor_state.autosave.reset(); // Reset auto-save timer
                                }
                            } else {
                                self.editor_state.console.warning("No scene to save. Use File → Save Scene As...".to_string());
                            }
                        }
                        EditorShortcut::Exit => {
                            // Exit editor (Ctrl+Q)
                            self.editor_state.show_exit_dialog = true;
                        }
                        _ => {
                            // Other shortcuts not yet implemented
                        }
                    }
                }
            }
        }
    }

    fn render(&mut self, target: &EventLoopWindowTarget<()>) {
        let dt = 1.0 / 60.0; // Fixed time step for now

        self.ctx.input.begin_frame();
        self.ctx.input.update_gamepads();

        // Egui frame setup
        let raw_input = self.egui_state.take_egui_input(&self.window);
        
        // Re-apply Unity theme every frame to prevent override
        UnityTheme::apply(&self.egui_ctx);
        
        self.egui_ctx.begin_frame(raw_input);

        // Auto-save logic (only in editor mode)
        if self.app_state == AppState::Editor && !self.editor_state.is_playing {
            if self.editor_state.autosave.should_save() && self.editor_state.scene_modified {
                if let Some(scene_path) = &self.editor_state.current_scene_path {
                    let autosave_path = self.editor_state.autosave.create_autosave_path(scene_path);
                    if let Ok(json) = self.editor_state.world.save_to_json() {
                        if std::fs::write(&autosave_path, json).is_ok() {
                            self.editor_state.autosave.mark_saved();
                            self.editor_state.console.info(format!("Auto-saved to {}", autosave_path.display()));
                            let _ = self.editor_state.autosave.cleanup_old_autosaves(scene_path);
                        }
                    }
                }
            }
        }

        // Render UI based on app state
        match self.app_state {
            AppState::Launcher => {
                // Launcher UI
                crate::ui::launcher_window::LauncherWindow::render(
                    &self.egui_ctx,
                    &mut self.app_state,
                    &mut self.launcher_state,
                    &mut self.editor_state,
                );
            }
            AppState::Playing => {
                crate::ui::game_window::GameWindow::render(
                    &self.egui_ctx,
                    &mut self.app_state,
                );
            }
            AppState::Editor => {
                self.render_editor_ui();
            }
        }

        let full_output = self.egui_ctx.end_frame();

        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.renderer.config.width, self.renderer.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        // Update textures
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.renderer.device, &self.renderer.queue, *id, image_delta);
        }

        let res = self.renderer.render_with_callback(|device, queue, encoder, view, texture_manager, batch_renderer| {
            self.egui_renderer.update_buffers(
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

            // If in Playing mode, render the game world using BatchRenderer
            if self.app_state == AppState::Playing {
                runtime::render_system::render_game_world(
                    &self.editor_state.world,
                    batch_renderer,
                    texture_manager,
                    queue,
                    device,
                    self.window.inner_size(),
                );
            }

            self.egui_renderer.render(
                &mut rpass,
                &paint_jobs,
                &screen_descriptor,
            );
        });

        // Free textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        match res {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size),
            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
            Err(e) => eprintln!("{:?}", e),
        }
    }

    fn render_editor_ui(&mut self) {
        let mut save_request = false;
        let mut save_as_request = false;
        let mut load_request = false;
        let mut load_file_request: Option<std::path::PathBuf> = None;
        let mut new_scene_request = false;
        let mut play_request = false;
        let mut stop_request = false;
        let mut edit_script_request: Option<String> = None;

        // Handle Q/W/E/R keyboard shortcuts for transform tools
        self.egui_ctx.input(|i| {
            if i.key_pressed(egui::Key::Q) {
                self.editor_state.current_tool = TransformTool::View;
                self.editor_state.console.info("Tool: View (Q)".to_string());
            } else if i.key_pressed(egui::Key::W) {
                self.editor_state.current_tool = TransformTool::Move;
                self.editor_state.console.info("Tool: Move (W)".to_string());
            } else if i.key_pressed(egui::Key::E) {
                self.editor_state.current_tool = TransformTool::Rotate;
                self.editor_state.console.info("Tool: Rotate (E)".to_string());
            } else if i.key_pressed(egui::Key::R) {
                self.editor_state.current_tool = TransformTool::Scale;
                self.editor_state.console.info("Tool: Scale (R)".to_string());
            }
        });

        // Initialize asset manager if not yet initialized
        if self.editor_state.asset_manager.is_none() {
            if let Some(ref project_path) = self.editor_state.current_project_path {
                self.editor_state.asset_manager = Some(crate::AssetManager::new(project_path));
            }
        }
        
        // Initialize hot-reload watcher if not yet initialized
        if self.editor_state.map_manager.hot_reload_watcher.is_none() && self.editor_state.map_manager.hot_reload_enabled {
            if let Err(e) = self.editor_state.map_manager.enable_hot_reload() {
                self.editor_state.console.warning(format!("Failed to enable hot-reload: {}", e));
            }
        }
        
        // Set map manager project path if not yet set
        if self.editor_state.map_manager.project_path.is_none() {
            if let Some(ref project_path) = self.editor_state.current_project_path {
                self.editor_state.map_manager.set_project_path(project_path.clone());
            }
        }
        
        // Set prefab manager project path if not yet set
        if self.editor_state.prefab_manager.project_path.is_none() {
            if let Some(ref project_path) = self.editor_state.current_project_path {
                self.editor_state.prefab_manager.set_project_path(project_path.clone());
            }
        }
        
        // Handle layout change request
        if let Some(layout_name) = self.editor_state.layout_request.take() {
            // ... (Layout handling code)
            // Simplified for brevity, logic is same as main.rs
             if layout_name == "save_default" {
                self.editor_state.save_default_layout();
                self.editor_state.console.info(format!("Saved '{}' as default layout", self.editor_state.current_layout_name));
            } else if layout_name == "save_as" {
                self.editor_state.show_save_layout_dialog = true;
                self.editor_state.save_layout_name.clear();
            } else if layout_name.starts_with("load:") {
                let name = layout_name.strip_prefix("load:").unwrap();
                self.editor_state.dock_state = crate::ui::get_layout_by_name(name);
                self.editor_state.current_layout_name = name.to_string();
                self.editor_state.current_layout_type = name.to_string();
                self.editor_state.console.info(format!("Changed to '{}' layout", name));
            } else if layout_name.starts_with("custom:") {
                let name = layout_name.strip_prefix("custom:").unwrap();
                if let Some(project_path) = &self.editor_state.current_project_path {
                    if let Some(dock_state) = crate::ui::load_custom_layout_state(name, project_path) {
                        self.editor_state.dock_state = dock_state;
                        self.editor_state.current_layout_name = name.to_string();
                        self.editor_state.current_layout_type = "custom".to_string();
                        self.editor_state.console.info(format!("Loaded custom layout '{}'", name));
                    } else {
                        let layouts = crate::ui::load_custom_layouts(project_path);
                        if let Some((_, layout_type)) = layouts.iter().find(|(n, _)| n == name) {
                            if layout_type != "custom" {
                                self.editor_state.dock_state = crate::ui::get_layout_by_name(layout_type);
                                self.editor_state.current_layout_name = name.to_string();
                                self.editor_state.current_layout_type = layout_type.clone();
                                self.editor_state.console.info(format!("Loaded custom layout '{}' (legacy)", name));
                            }
                        }
                    }
                }
            }
        }

        // Save Layout Dialog
        crate::ui::dialogs::LayoutDialog::render(&self.egui_ctx, &mut self.editor_state);

        // Process hot-reload for LDtk maps
        let reloaded_maps = self.editor_state.map_manager.process_hot_reload(&mut self.editor_state.world);
        if !reloaded_maps.is_empty() {
            for map_path in &reloaded_maps {
                self.editor_state.console.info(format!("🔄 Hot-reloaded map: {:?}", map_path));
            }
        }
        
        // Display hot-reload error if any
        if let Some(error) = self.editor_state.map_manager.get_last_hot_reload_error() {
            self.editor_state.console.error(format!("Hot-reload error: {}", error));
            self.editor_state.map_manager.clear_hot_reload_error();
        }

        // Main Editor Logic
        crate::editor_logic::EditorLogic::handle_editor_frame(
            &self.egui_ctx,
            &mut self.app_state,
            &mut self.editor_state,
            &mut self.ctx,
            &mut self.script_engine,
            &mut self.physics,
            &mut self.physics_accumulator,
            self.fixed_timestep,
            1.0 / 60.0, // dt
        );
    }
}

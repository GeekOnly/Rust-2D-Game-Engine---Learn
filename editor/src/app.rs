use anyhow::Result;
use engine_core::EngineContext;
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use render::{RenderModule, CameraBinding};
use crate::ui::TransformTool;
use crate::states::{AppState, LauncherState, EditorState};
use engine::runtime;
use engine::runtime::render_system::RenderCache;
use engine_core::assets::AssetLoader;
use crate::theme::UnityTheme;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    window::Window,
};
use glam::{Vec3, Mat4, Quat, EulerRot};
use ecs::{Camera, CameraProjection};

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
    pub game_view_renderer: crate::game_view_renderer::GameViewRenderer,
    pub scene_view_renderer: crate::scene_view_renderer::SceneViewRenderer,
    pub scene_camera_binding: CameraBinding,
    pub grid_renderer: render::GridRenderer,
    pub physics_accumulator: f32,
    pub fixed_timestep: f32,
    pub render_cache: RenderCache,
}

impl EditorApp {
    // Map winit keycode to our input system key (same as Player binary)
    fn map_winit_keycode(&self, keycode: winit::keyboard::KeyCode) -> Option<input::Key> {
        use winit::keyboard::KeyCode;
        use input::Key;
        
        match keycode {
            KeyCode::KeyA => Some(Key::A),
            KeyCode::KeyB => Some(Key::B),
            KeyCode::KeyC => Some(Key::C),
            KeyCode::KeyD => Some(Key::D),
            KeyCode::KeyE => Some(Key::E),
            KeyCode::KeyF => Some(Key::F),
            KeyCode::KeyG => Some(Key::G),
            KeyCode::KeyH => Some(Key::H),
            KeyCode::KeyI => Some(Key::I),
            KeyCode::KeyJ => Some(Key::J),
            KeyCode::KeyK => Some(Key::K),
            KeyCode::KeyL => Some(Key::L),
            KeyCode::KeyM => Some(Key::M),
            KeyCode::KeyN => Some(Key::N),
            KeyCode::KeyO => Some(Key::O),
            KeyCode::KeyP => Some(Key::P),
            KeyCode::KeyQ => Some(Key::Q),
            KeyCode::KeyR => Some(Key::R),
            KeyCode::KeyS => Some(Key::S),
            KeyCode::KeyT => Some(Key::T),
            KeyCode::KeyU => Some(Key::U),
            KeyCode::KeyV => Some(Key::V),
            KeyCode::KeyW => Some(Key::W),
            KeyCode::KeyX => Some(Key::X),
            KeyCode::KeyY => Some(Key::Y),
            KeyCode::KeyZ => Some(Key::Z),
            KeyCode::Digit0 => Some(Key::Num0),
            KeyCode::Digit1 => Some(Key::Num1),
            KeyCode::Digit2 => Some(Key::Num2),
            KeyCode::Digit3 => Some(Key::Num3),
            KeyCode::Digit4 => Some(Key::Num4),
            KeyCode::Digit5 => Some(Key::Num5),
            KeyCode::Digit6 => Some(Key::Num6),
            KeyCode::Digit7 => Some(Key::Num7),
            KeyCode::Digit8 => Some(Key::Num8),
            KeyCode::Digit9 => Some(Key::Num9),
            KeyCode::ArrowUp => Some(Key::Up),
            KeyCode::ArrowDown => Some(Key::Down),
            KeyCode::ArrowLeft => Some(Key::Left),
            KeyCode::ArrowRight => Some(Key::Right),
            KeyCode::Space => Some(Key::Space),
            KeyCode::Enter => Some(Key::Enter),
            KeyCode::Escape => Some(Key::Escape),
            KeyCode::Tab => Some(Key::Tab),
            KeyCode::Backspace => Some(Key::Backspace),
            KeyCode::Delete => Some(Key::Delete),
            KeyCode::ShiftLeft => Some(Key::LShift),
            KeyCode::ShiftRight => Some(Key::RShift),
            KeyCode::ControlLeft => Some(Key::LCtrl),
            KeyCode::ControlRight => Some(Key::RCtrl),
            KeyCode::AltLeft => Some(Key::LAlt),
            KeyCode::AltRight => Some(Key::RAlt),
            _ => None,
        }
    }

    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window_attributes = Window::default_attributes()
            .with_title("Rust 2D Game Engine - Launcher")
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 700));
        let window = event_loop.create_window(window_attributes)?;

        let app_state = AppState::Launcher;
        let launcher_state = LauncherState::new()?;
        let editor_state = EditorState::new();

        let asset_loader: std::sync::Arc<dyn engine_core::assets::AssetLoader> = std::sync::Arc::new(engine::assets::native_loader::NativeAssetLoader::new("."));
        let ctx = EngineContext::new(asset_loader.clone());
        // Sample game removed - use projects/ folder for game content

        let script_engine = ScriptEngine::new(asset_loader.clone())?;
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
            None,
        );

        let mut egui_renderer = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.config.format,
            egui_wgpu::RendererOptions::default(),
        );

        let game_view_renderer = crate::game_view_renderer::GameViewRenderer::new(
            &renderer.device,
            &mut egui_renderer,
            1280, // Default resolution
            720,
        );

        let scene_view_renderer = crate::scene_view_renderer::SceneViewRenderer::new(
            &renderer.device,
            &mut egui_renderer,
            1280, // Default resolution
            720,
        );

        let scene_camera_binding = CameraBinding::new(&renderer.device);

        // Initialize Grid Renderer
        let grid_renderer = render::GridRenderer::new(
            &renderer.device,
            &renderer.config,
            &scene_camera_binding.bind_group_layout,
        );

        let render_cache = RenderCache::new();

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
            game_view_renderer,
            scene_view_renderer,
            scene_camera_binding,
            grid_renderer,
            physics_accumulator: 0.0,
            fixed_timestep: 1.0 / 60.0,
            render_cache,
        })
    }

    pub fn handle_event(&mut self, event: Event<()>, target: &ActiveEventLoop) {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                // Pass events to egui and check if it consumed the event
                let egui_consumed = self.egui_state.on_window_event(&self.window, event);

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
                        // When playing in editor, allow game input even if egui wants keyboard
                        // This fixes the issue where Game View panel consumes A/D keys
                        let should_handle = if self.app_state == AppState::Editor && self.editor_state.is_playing {
                            // In play mode, always handle keyboard input for game control
                            true
                        } else {
                            // Otherwise, check if egui wants the input
                            !egui_consumed.consumed
                        };

                        if should_handle {
                            self.handle_keyboard_input(key_event);
                        }
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
            // Use the same key mapping as Player binary
            if let Some(key) = self.map_winit_keycode(key_code) {
                if key_event.state == ElementState::Pressed {
                    self.ctx.input.press_key(key);
                    // Debug: log key press for movement and jump keys when in play mode
                    if self.app_state == AppState::Editor && self.editor_state.is_playing {
                        match key {
                            input::Key::Space => self.editor_state.console.debug(format!("✅ Space key pressed in ctx.input")),
                            input::Key::A => self.editor_state.console.debug(format!("✅ A key pressed in ctx.input")),
                            input::Key::D => self.editor_state.console.debug(format!("✅ D key pressed in ctx.input")),
                            input::Key::W => self.editor_state.console.debug(format!("✅ W key pressed in ctx.input")),
                            input::Key::S => self.editor_state.console.debug(format!("✅ S key pressed in ctx.input")),
                            input::Key::Left => self.editor_state.console.debug(format!("✅ Left key pressed in ctx.input")),
                            input::Key::Right => self.editor_state.console.debug(format!("✅ Right key pressed in ctx.input")),
                            _ => {}
                        }
                    }
                } else {
                    self.ctx.input.release_key(key);
                }
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
                                    let pos = glam::Vec3::new(transform.x(), transform.y(), 0.0);
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

    fn render(&mut self, target: &ActiveEventLoop) {
        let _dt = 1.0 / 60.0; // Fixed time step for now

        // Don't clear input here - let PlayModeSystem handle it after scripts run
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
                    &*self.ctx.asset_loader,
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

        // Update Global Transforms (Hierarchy)
        // This ensures child entities follow their parents
        if self.app_state == AppState::Editor {
             runtime::transform_system::update_global_transforms(&mut self.editor_state.world);
        }

        // Render Game World to Offscreen Textures (for Editor Game View) before the main render pass
        if self.app_state == AppState::Editor {
            // Ensure Asset meshes are loaded (idempotent check)
            if let Some(ref project_path) = self.editor_state.current_project_path {
                 runtime::render_system::post_process_asset_meshes(
                     &mut self.render_cache,
                     project_path,
                     &mut self.editor_state.world,
                     &self.renderer.device,
                     &self.renderer.queue,
                     &mut self.renderer.texture_manager,
                     &mut self.renderer.mesh_renderer,
                     &*self.ctx.asset_loader,
                 );
            }
            
            // Render Scene View and Game View to their respective offscreen textures
            self.render_offscreen_views();
        }

        // Local mutable reference to renderer to avoid self-borrowing conflicts
        let egui_renderer = &mut self.egui_renderer;

        // Manual rendering to handle egui_wgpu lifetime quirk
        let output = match self.renderer.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost) => {
                self.renderer.resize(self.renderer.size);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                target.exit();
                return;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update egui buffers
        egui_renderer.update_buffers(
            &self.renderer.device,
            &self.renderer.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // Render pass
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // SAFETY: egui_wgpu::Renderer::render requires 'static RenderPass due to internal
            // lifetime constraints, but we know the renderer won't store the pass after this call.
            // This casts the lifetime of rpass to 'static.
            let mut rpass: wgpu::RenderPass<'static> = unsafe { std::mem::transmute(rpass) };

            egui_renderer.render(
                &mut rpass,
                &paint_jobs,
                &screen_descriptor,
            );
        }

        self.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Free textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }

    fn render_editor_ui(&mut self) {
        // Local request flags - Moved to EditorLogic::handle_editor_frame
        // We don't need them here anymore.

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
        let asset_loader = self.ctx.asset_loader.clone();
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
            &mut self.game_view_renderer,
            &self.renderer.device,
            &self.renderer.queue,
            &mut self.egui_renderer,
            &mut self.scene_view_renderer,
            &self.renderer.mesh_renderer,
            &mut self.renderer.texture_manager,
            &*asset_loader,
            &mut self.render_cache,
        );
        
        // Clear input state if not in play mode (PlayModeSystem handles it when playing)
        if !self.editor_state.is_playing {
            self.ctx.input.begin_frame();
        }
    }

    fn render_offscreen_views(&mut self) {
        // Render Scene View
        let width = self.scene_view_renderer.width;
        let height = self.scene_view_renderer.height;
        
        if width > 0 && height > 0 {
            let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Scene View Encoder"),
            });

            // Calculate View & Projection Matrices
            let aspect = width as f32 / height as f32;
            let projection = self.editor_state.scene_camera.get_projection_matrix(aspect);
            let view = self.editor_state.scene_camera.get_view_matrix();
            let camera_pos = self.editor_state.scene_camera.position;

            // Update scene camera uniform buffer
            self.scene_camera_binding.update(&self.renderer.queue, view, projection, camera_pos);

            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene View Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.scene_view_renderer.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.15, b: 0.15, a: 1.0 }), // Dark Unity-like background
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.scene_view_renderer.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                
                // Draw Grid (WGPU)
                if self.editor_state.infinite_grid.enabled || self.editor_state.scene_grid.enabled {
                    self.grid_renderer.render(
                        &mut rpass, 
                        &self.scene_camera_binding.bind_group
                    );
                }

                // Render Game World
                runtime::render_system::render_game_world(
                    &mut self.render_cache,
                    &self.editor_state.world,
                    &self.renderer.tilemap_renderer,
                    &mut self.renderer.batch_renderer,
                    &mut self.renderer.mesh_renderer,
                    &self.scene_camera_binding, 
                    &self.renderer.light_binding,
                    &mut self.renderer.texture_manager,
                    &self.renderer.queue,
                    &self.renderer.device,
                    winit::dpi::PhysicalSize::new(width, height),
                    &mut rpass,
                    projection * view, 
                );
            }

            self.renderer.queue.submit(std::iter::once(encoder.finish()));
        }
        
        // Render Game View
        let game_width = self.game_view_renderer.width;
        let game_height = self.game_view_renderer.height;

        if game_width > 0 && game_height > 0 {
             // Find Main Camera
             let mut cameras: Vec<_> = self.editor_state.world.cameras.iter()
                .filter_map(|(entity, camera)| {
                    if self.editor_state.world.active.get(entity).copied().unwrap_or(true) {
                        self.editor_state.world.transforms.get(entity).map(|transform| (entity, camera, transform))
                    } else {
                        None
                    }
                })
                .collect();
            
            // Sort by depth (lowest depth first)
            cameras.sort_by_key(|(_, camera, _)| camera.depth);
            
            if let Some((_, camera, transform)) = cameras.first() {
                 let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Game View Encoder"),
                 });
                 
                 // Calculate View Matrix
                 let rot_rad = Vec3::new(
                    transform.rotation[0].to_radians(),
                    transform.rotation[1].to_radians(),
                    transform.rotation[2].to_radians(),
                 );
                 let cam_rotation = Quat::from_euler(EulerRot::YXZ, rot_rad.y, rot_rad.x, rot_rad.z);
                 let cam_translation = Vec3::from(transform.position);
                 let view = Mat4::from_rotation_translation(cam_rotation, cam_translation).inverse();
                 
                 // Calculate Projection Matrix
                 let aspect = game_width as f32 / game_height as f32;
                 let projection = match camera.projection {
                    CameraProjection::Orthographic => {
                         let height = camera.orthographic_size;
                         let width = height * aspect;
                         Mat4::orthographic_rh(
                            -width, width, 
                            -height, height, 
                            camera.near_clip, camera.far_clip
                        )
                    },
                    CameraProjection::Perspective => {
                        Mat4::perspective_rh(
                            camera.fov.to_radians(), 
                            aspect, 
                            camera.near_clip, 
                            camera.far_clip
                        )
                    }
                 };
                 
                 // Update Camera Binding (Reusing scene_camera_binding is safe because of sequential submission)
                 self.scene_camera_binding.update(&self.renderer.queue, view, projection, Vec3::from(transform.position));
                 
                 {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Game View Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &self.game_view_renderer.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: camera.background_color[0] as f64,
                                    g: camera.background_color[1] as f64,
                                    b: camera.background_color[2] as f64,
                                    a: camera.background_color[3] as f64,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.game_view_renderer.depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                    
                    // Render Game World
                    runtime::render_system::render_game_world(
                        &mut self.render_cache,
                        &self.editor_state.world,
                        &self.renderer.tilemap_renderer,
                        &mut self.renderer.batch_renderer,
                        &mut self.renderer.mesh_renderer,
                        &self.scene_camera_binding, 
                        &self.renderer.light_binding,
                        &mut self.renderer.texture_manager,
                        &self.renderer.queue,
                        &self.renderer.device,
                        winit::dpi::PhysicalSize::new(game_width, game_height),
                        &mut rpass,
                        projection * view, 
                    );
                 }
                
                 self.renderer.queue.submit(std::iter::once(encoder.finish()));
            }
        }
    }
}

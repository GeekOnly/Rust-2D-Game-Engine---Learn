use anyhow::Result;
use engine_core::EngineContext;
use ecs::World;
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use render::RenderModule;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Use the engine library
use engine::runtime;
use engine::texture_manager::TextureManager;
use engine::ui_manager::UIManager;

fn main() -> Result<()> {
    env_logger::init();
    log::info!("=== Game Player Runtime Starting ===");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rust 2D Game Engine Player")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)?;

    // Initialize systems
    let mut ctx = EngineContext::new();
    let script_engine = ScriptEngine::new()?;
    
    #[cfg(feature = "rapier")]
    let mut physics = RapierPhysicsWorld::new();
    #[cfg(not(feature = "rapier"))]
    let mut physics = PhysicsWorld::new();

    // Init Renderer
    let mut renderer = pollster::block_on(RenderModule::new(&window))?;
    let mut texture_manager = TextureManager::new();
    let mut ui_manager = UIManager::new();

    // Initialize egui for rendering the game view (reuse renderer logic)
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
 
    // Load Game Project
    // In a real export, these paths would be relative to the executable
    let project_path = std::env::current_dir()?; 
    log::info!("Loading project from: {:?}", project_path);

    // Initial World
    let mut world = World::new();
    let mut scene_path = project_path.join("scenes/main.json");
    
    // Check if defaults exist, otherwise try to find scene in arguments?
    // For now we assume typical export structure
    if !scene_path.exists() {
        log::warn!("Scene not found at {:?}, trying 'scenes/main.scene'", scene_path);
        let alt_path = project_path.join("scenes/main.scene");
        if alt_path.exists() {
            scene_path = alt_path;
        } else {
             log::error!("No scene file found!");
        }
    }

    if scene_path.exists() {
        if let Ok(json) = std::fs::read_to_string(&scene_path) {
            if let Err(e) = world.load_from_json(&json) {
                log::error!("Failed to load scene: {}", e);
            } else {
                log::info!("Scene loaded successfully");
            }
        }
    }

    // Set texture base path
    texture_manager.set_base_path(project_path.join("assets"));

    let mut last_frame_time = std::time::Instant::now();
    const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
    let mut physics_accumulator: f32 = 0.0;

    // Start scripts (Init) - call for all entities with scripts
    let entities_with_scripts: Vec<_> = world.scripts.keys().copied().collect();
    for entity in entities_with_scripts {
        if let Err(e) = script_engine.call_start_for_entity(entity, &mut world) {
            log::error!("Script start error for entity {:?}: {}", entity, e);
        }
    }

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                let _ = egui_state.on_window_event(&window, event);
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_frame_time).as_secs_f32();
                        last_frame_time = now;

                        // Clear per-frame input state
                        ctx.input.begin_frame();

                        // Scripts Update - call for all entities with scripts
                        let scripts_to_update: Vec<_> = world.scripts.iter()
                            .map(|(entity, script)| (*entity, script.script_name.clone()))
                            .collect();

                        for (entity, script_name) in scripts_to_update {
                            if let Err(_e) = script_engine.call_update(&script_name, dt, &mut world) {
                                // Limit error logging to avoid flooding
                                // log::error!("Script update error for entity {:?}: {}", entity, e);
                            }
                        }

                        // Physics
                        physics_accumulator += dt;
                        while physics_accumulator >= FIXED_TIMESTEP {
                            physics.step(FIXED_TIMESTEP, &mut world);
                            physics_accumulator -= FIXED_TIMESTEP;
                        }

                        // Render
                        let raw_input = egui_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        egui::CentralPanel::default().show(&egui_ctx, |ui| {
                            // Render the game view fullscreen
                            runtime::render_game_view(
                                ui,
                                &world,
                                &mut texture_manager,
                                Some(&mut ui_manager),
                                None, // Default settings (fullscreen)
                            );
                        });

                        let full_output = egui_ctx.end_frame();
                        
                        // Frame rendering using egui_wgpu
                         let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [renderer.config.width, renderer.config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

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
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    })?;

    Ok(())
}

use anyhow::Result;
use engine_core::{EngineContext, EngineModule};
use ecs::World;
use script::ScriptEngine;
use physics::PhysicsWorld;
use render::RenderModule;
use editor::EditorModule;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct SampleModule {
    world: World,
}

impl EngineModule for SampleModule {
    fn name(&self) -> &str { "sample" }
    fn on_load(&mut self, _ctx: &mut EngineContext) -> Result<()> { 
        println!("Sample module loaded!");
        Ok(()) 
    }
    fn on_update(&mut self, _ctx: &mut EngineContext, _dt: f32) {
        // game logic update
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() -> Result<()> {
    env_logger::init();
    println!("Starting minimal engine...");
    
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;
    
    let mut ctx = EngineContext::new();
    ctx.register_module(SampleModule { world: World::new() });

    let script = ScriptEngine::new()?;
    let mut physics = PhysicsWorld::new();
    
    // Initialize renderer with window
    let mut renderer = pollster::block_on(RenderModule::new(&window))?;
    let mut editor = EditorModule::new();

    // egui setup
    let mut egui_ctx = egui::Context::default();
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
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer: _ } => {
                        // In winit 0.29, ScaleFactorChanged provides inner_size_writer, not new_inner_size directly in the same way?
                        // Actually it's simpler to just handle Resized usually.
                        // But let's check docs if possible. 
                        // For now, let's just ignore ScaleFactorChanged or assume it triggers Resized.
                    }
                    WindowEvent::RedrawRequested => {
                        let dt = 1.0 / 60.0; // Fixed time step for now
                        ctx.update(dt);
                        
                        // Fix downcasting
                        if let Some(module) = ctx.modules.values_mut().find(|m| m.name() == "sample") {
                            if let Some(sample) = module.as_any().downcast_mut::<SampleModule>() {
                                physics.step(dt, &mut sample.world);
                            }
                        }
                        
                        // Egui frame setup
                        let raw_input = egui_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        // Egui UI code
                        egui::Window::new("Editor").show(&egui_ctx, |ui| {
                            ui.label("Hello from egui!");
                        });

                        let full_output = egui_ctx.end_frame();
                        
                        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [renderer.config.width, renderer.config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };
                        
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

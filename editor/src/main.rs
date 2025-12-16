use anyhow::Result;
use winit::event_loop::EventLoop;
use editor::app::EditorApp;

fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Game Engine...");
    log::info!("=== Rust 2D Game Engine Starting ===");
    log::info!("Logging initialized");

    let event_loop = EventLoop::new()?;
    let mut app = EditorApp::new(&event_loop)?;

    event_loop.run(move |event, target| {
        app.handle_event(event, target);
    })?;

    Ok(())
}

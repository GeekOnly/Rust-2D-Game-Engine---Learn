// Runtime-only modules (for standalone game builds)
pub mod script_loader;
pub mod renderer;
pub mod render_system;
pub mod physics_system;
pub mod script_system;
pub mod systems;
pub mod ldtk_runtime;
pub mod game_view_settings;

// Re-exports for convenience
pub use renderer::render_game_view;
pub use ldtk_runtime::LdtkRuntime;
pub use game_view_settings::{GameViewSettings, GameViewResolution};

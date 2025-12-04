// Runtime-only modules (for standalone game builds)
pub mod script_loader;
pub mod renderer;
pub mod ldtk_runtime;

// Re-exports for convenience
pub use renderer::render_game_view;
pub use ldtk_runtime::LdtkRuntime;

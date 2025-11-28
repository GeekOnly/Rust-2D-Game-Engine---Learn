// Runtime-only modules (for standalone game builds)
pub mod script_loader;
pub mod renderer;
// Re-exports for convenience
pub use script_loader::{load_all_scripts, run_all_scripts};
pub use renderer::render_game_view;

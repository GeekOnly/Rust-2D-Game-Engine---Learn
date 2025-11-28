// Runtime-only modules (for standalone game builds)
pub mod script_loader;
pub mod renderer;
pub mod physics;

// Re-exports for convenience
pub use script_loader::{load_all_scripts, run_all_scripts};
pub use renderer::render_game_view;
pub use physics::{PhysicsWorld, helpers as physics_helpers};

// Runtime-only modules (for standalone game builds)
pub mod script_loader;

// Re-exports for convenience
pub use script_loader::{load_all_scripts, run_all_scripts};

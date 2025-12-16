//! ECS Backend Implementations
//!
//! Different ECS backends that implement the abstraction traits.

// Custom HashMap-based backend (always available)
// Custom HashMap-based backend (always available)
// pub mod custom; // Defined in lib.rs directly for now

// Optional backends (enabled with feature flags)
#[cfg(any(feature = "hecs", test))]
pub mod hecs_backend;

#[cfg(feature = "specs")]
pub mod specs_backend;

#[cfg(feature = "bevy")]
pub mod bevy_backend;

// Re-export the default backend if specific backend is not selected
// This is just a helper, the main lib.rs decides what "World" means
// Re-export the default backend
// pub use custom::CustomBackend;

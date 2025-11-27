//! ECS Backend Implementations
//!
//! Different ECS backends that implement the abstraction traits.

// Custom HashMap-based backend (always available)
pub mod custom;

// Optional backends (enabled with feature flags)
#[cfg(feature = "hecs")]
pub mod hecs_backend;

#[cfg(feature = "specs")]
pub mod specs_backend;

#[cfg(feature = "bevy")]
pub mod bevy_backend;

// Re-export the default backend
pub use custom::CustomBackend;

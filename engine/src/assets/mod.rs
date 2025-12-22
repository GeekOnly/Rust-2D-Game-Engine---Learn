pub mod core;
pub mod metadata;
pub mod manager;

pub mod gltf_loader;
pub mod model_manager;
pub mod xsg;
pub mod xsg_importer;
pub mod xsg_loader;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_loader;

#[cfg(target_arch = "wasm32")]
pub mod web_loader;

//! HUD System - Screen-Space UI
//! 
//! Manages screen-space HUD elements that are independent of the game world
//! (e.g., player health, minimap, inventory, menus)

pub mod hud_asset;
pub mod hud_manager;
pub mod hud_renderer;

pub use hud_asset::{HudAsset, HudElement, HudElementType, Anchor};
pub use hud_manager::HudManager;
pub use hud_renderer::HudRenderer;

//! # UI System
//!
//! A comprehensive in-game UI system for the XS 2D Game Engine, providing capabilities
//! comparable to Unity's Canvas UI and Unreal Engine's UMG.
//!
//! ## Features
//!
//! - Canvas-based UI rendering with multiple render modes
//! - Flexible RectTransform anchoring and positioning
//! - Hierarchical UI element organization
//! - Rich set of UI components (Image, Text, Button, Panel, etc.)
//! - Automatic layout system (Horizontal, Vertical, Grid)
//! - Event system for user interactions
//! - UI animations with easing functions
//! - Scroll views with clipping and masking
//! - Resolution-independent scaling
//! - Lua scripting integration
//! - UI prefabs and styling system

// Re-export commonly used types from dependencies
pub use glam::{Vec2, Vec3, Vec4};
pub use serde::{Deserialize, Serialize};

// Core types module
mod types;
pub use types::{Color, Rect};

// Module declarations
pub mod canvas;
pub mod canvas_system;
pub mod rect_transform;
pub mod rect_transform_system;
pub mod components;
pub mod layout;
pub mod events;
pub mod rendering;
pub mod animation;
pub mod prefab;
pub mod style;

// Re-export main types for convenience
pub use canvas::{Canvas, CanvasRenderMode, CanvasScaler, ScaleMode};
pub use canvas_system::CanvasSystem;
pub use rect_transform::RectTransform;
pub use rect_transform_system::{RectTransformSystem, Entity};

// Re-export component types
pub use components::{
    UIElement,
    UIImage, ImageType, FillMethod,
    UIText, TextAlignment, OverflowMode,
    UIButton, ButtonState, ButtonTransition,
    UIPanel,
    UISlider, SliderDirection,
    UIToggle, ToggleTransition,
    UIDropdown, DropdownOption,
    UIInputField, ContentType, LineType, InputType, KeyboardType, CharacterValidation,
    UIScrollView, MovementType,
};

// Re-export layout types
pub use layout::{
    HorizontalLayoutGroup,
    VerticalLayoutGroup,
    GridLayoutGroup, GridConstraint,
    Alignment, Corner, Axis,
};

// Re-export event types
pub use events::{
    UIEvent, UIEventHandler, UIEventListener, UIEventType,
};

// Re-export animation types
pub use animation::{
    UIAnimation, AnimatedProperty, AnimationValue,
    EasingFunction, LoopMode,
};

// Re-export rendering types
pub use rendering::{
    UIMask,
};

// Re-export prefab types
pub use prefab::{
    UIPrefab, UIPrefabElement,
};

// Re-export style types
pub use style::{
    UIStyle, UITheme,
};

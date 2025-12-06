//! UI animation system with tweening and easing

use serde::{Deserialize, Serialize};
use glam::Vec2;
use crate::Color;

/// UI Animation (tween-based)
#[derive(Clone, Debug)]
pub struct UIAnimation {
    /// Target entity
    pub entity: u64, // Using u64 as placeholder for Entity
    
    /// Property to animate
    pub property: AnimatedProperty,
    
    /// Start value
    pub from: AnimationValue,
    
    /// End value
    pub to: AnimationValue,
    
    /// Duration in seconds
    pub duration: f32,
    
    /// Easing function
    pub easing: EasingFunction,
    
    /// Delay before starting
    pub delay: f32,
    
    /// Loop mode
    pub loop_mode: LoopMode,
    
    /// Completion callback
    pub on_complete: Option<String>,
    
    /// Runtime state
    pub elapsed: f32,
    pub started: bool,
    pub completed: bool,
}

/// Property that can be animated
#[derive(Clone, Debug, PartialEq)]
pub enum AnimatedProperty {
    AnchoredPosition,
    Scale,
    Rotation,
    Color,
    Alpha,
    SizeDelta,
}

/// Animation value (union type for different value types)
#[derive(Clone, Debug)]
pub enum AnimationValue {
    Vec2(Vec2),
    Float(f32),
    Color(Color),
}

/// Easing function types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseInQuad, EaseOutQuad, EaseInOutQuad,
    EaseInCubic, EaseOutCubic, EaseInOutCubic,
    EaseInQuart, EaseOutQuart, EaseInOutQuart,
    EaseInQuint, EaseOutQuint, EaseInOutQuint,
    EaseInSine, EaseOutSine, EaseInOutSine,
    EaseInExpo, EaseOutExpo, EaseInOutExpo,
    EaseInCirc, EaseOutCirc, EaseInOutCirc,
    EaseInElastic, EaseOutElastic, EaseInOutElastic,
    EaseInBack, EaseOutBack, EaseInOutBack,
    EaseInBounce, EaseOutBounce, EaseInOutBounce,
}

/// Loop mode for animations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

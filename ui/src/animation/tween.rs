//! Animation update system for UI tweening

use glam::Vec2;
use crate::{Color, RectTransform};
use crate::components::UIElement;
use super::{UIAnimation, AnimatedProperty, AnimationValue, LoopMode, easing};

impl UIAnimation {
    /// Create a new animation
    pub fn new(
        entity: u64,
        property: AnimatedProperty,
        from: AnimationValue,
        to: AnimationValue,
        duration: f32,
    ) -> Self {
        Self {
            entity,
            property,
            from,
            to,
            duration,
            easing: super::EasingFunction::Linear,
            delay: 0.0,
            loop_mode: LoopMode::Once,
            on_complete: None,
            elapsed: 0.0,
            started: false,
            completed: false,
        }
    }

    /// Update the animation by delta time
    /// Returns true if the animation is still active, false if completed
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.completed {
            return false;
        }

        // Handle delay
        if !self.started {
            if self.delay > 0.0 {
                self.delay -= delta_time;
                if self.delay > 0.0 {
                    return true;
                }
                // Delay finished, carry over remaining time
                let remaining_time = -self.delay;
                self.delay = 0.0;
                self.started = true;
                self.elapsed += remaining_time;
            } else {
                self.started = true;
                self.elapsed += delta_time;
            }
        } else {
            // Update elapsed time
            self.elapsed += delta_time;
        }

        // Check if animation is complete
        if self.elapsed >= self.duration {
            match self.loop_mode {
                LoopMode::Once => {
                    self.elapsed = self.duration;
                    self.completed = true;
                    return false;
                }
                LoopMode::Loop => {
                    // Reset to beginning
                    self.elapsed = self.elapsed % self.duration;
                }
                LoopMode::PingPong => {
                    // Bounce back and forth
                    let cycles = (self.elapsed / self.duration).floor() as i32;
                    self.elapsed = self.elapsed % self.duration;
                    
                    // On odd cycles, reverse direction
                    if cycles % 2 == 1 {
                        self.elapsed = self.duration - self.elapsed;
                    }
                }
            }
        }

        true
    }

    /// Get the current interpolated value
    pub fn get_current_value(&self) -> AnimationValue {
        let t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let eased_t = easing::evaluate(&self.easing, t);

        match (&self.from, &self.to) {
            (AnimationValue::Vec2(from), AnimationValue::Vec2(to)) => {
                AnimationValue::Vec2(lerp_vec2(*from, *to, eased_t))
            }
            (AnimationValue::Float(from), AnimationValue::Float(to)) => {
                AnimationValue::Float(lerp_f32(*from, *to, eased_t))
            }
            (AnimationValue::Color(from), AnimationValue::Color(to)) => {
                AnimationValue::Color(lerp_color(*from, *to, eased_t))
            }
            _ => {
                // Mismatched types, return from value
                self.from.clone()
            }
        }
    }

    /// Apply the animation to a RectTransform component
    pub fn apply_to_rect_transform(&self, rect_transform: &mut RectTransform) {
        let value = self.get_current_value();

        match self.property {
            AnimatedProperty::AnchoredPosition => {
                if let AnimationValue::Vec2(pos) = value {
                    rect_transform.anchored_position = pos;
                    rect_transform.dirty = true;
                }
            }
            AnimatedProperty::Scale => {
                if let AnimationValue::Vec2(scale) = value {
                    rect_transform.scale = scale;
                    rect_transform.dirty = true;
                }
            }
            AnimatedProperty::Rotation => {
                if let AnimationValue::Float(rotation) = value {
                    rect_transform.rotation = rotation;
                    rect_transform.dirty = true;
                }
            }
            AnimatedProperty::SizeDelta => {
                if let AnimationValue::Vec2(size) = value {
                    rect_transform.size_delta = size;
                    rect_transform.dirty = true;
                }
            }
            _ => {
                // Property doesn't apply to RectTransform
            }
        }
    }

    /// Apply the animation to a UIElement component
    pub fn apply_to_ui_element(&self, ui_element: &mut UIElement) {
        let value = self.get_current_value();

        match self.property {
            AnimatedProperty::Color => {
                if let AnimationValue::Color(color) = value {
                    ui_element.color = color;
                }
            }
            AnimatedProperty::Alpha => {
                if let AnimationValue::Float(alpha) = value {
                    ui_element.alpha = alpha.clamp(0.0, 1.0);
                }
            }
            _ => {
                // Property doesn't apply to UIElement
            }
        }
    }
}

/// Linear interpolation for Vec2
fn lerp_vec2(from: Vec2, to: Vec2, t: f32) -> Vec2 {
    from + (to - from) * t
}

/// Linear interpolation for f32
fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// Linear interpolation for Color (RGBA)
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    [
        lerp_f32(from[0], to[0], t),
        lerp_f32(from[1], to[1], t),
        lerp_f32(from[2], to[2], t),
        lerp_f32(from[3], to[3], t),
    ]
}

/// Animation manager that handles multiple animations
pub struct AnimationManager {
    animations: Vec<UIAnimation>,
    completed_callbacks: Vec<String>,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            completed_callbacks: Vec::new(),
        }
    }

    /// Add an animation to the manager
    pub fn add_animation(&mut self, animation: UIAnimation) {
        self.animations.push(animation);
    }

    /// Update all animations
    /// Returns a list of callback names for completed animations
    pub fn update(&mut self, delta_time: f32) -> Vec<String> {
        self.completed_callbacks.clear();

        // Update all animations and collect completed ones
        self.animations.retain_mut(|anim| {
            let still_active = anim.update(delta_time);
            
            if !still_active {
                if let Some(callback) = &anim.on_complete {
                    self.completed_callbacks.push(callback.clone());
                }
            }
            
            still_active
        });

        self.completed_callbacks.clone()
    }

    /// Get all active animations
    pub fn get_animations(&self) -> &[UIAnimation] {
        &self.animations
    }

    /// Get mutable reference to all active animations
    pub fn get_animations_mut(&mut self) -> &mut Vec<UIAnimation> {
        &mut self.animations
    }

    /// Remove all animations for a specific entity
    pub fn remove_animations_for_entity(&mut self, entity: u64) {
        self.animations.retain(|anim| anim.entity != entity);
    }

    /// Remove all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// Get the number of active animations
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::EasingFunction;

    #[test]
    fn test_animation_creation() {
        let anim = UIAnimation::new(
            1,
            AnimatedProperty::AnchoredPosition,
            AnimationValue::Vec2(Vec2::ZERO),
            AnimationValue::Vec2(Vec2::new(100.0, 100.0)),
            1.0,
        );

        assert_eq!(anim.entity, 1);
        assert_eq!(anim.duration, 1.0);
        assert_eq!(anim.elapsed, 0.0);
        assert!(!anim.started);
        assert!(!anim.completed);
    }

    #[test]
    fn test_animation_update() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::AnchoredPosition,
            AnimationValue::Vec2(Vec2::ZERO),
            AnimationValue::Vec2(Vec2::new(100.0, 100.0)),
            1.0,
        );

        // Update halfway
        assert!(anim.update(0.5));
        assert_eq!(anim.elapsed, 0.5);
        assert!(anim.started);
        assert!(!anim.completed);

        // Update to completion
        assert!(!anim.update(0.5));
        assert_eq!(anim.elapsed, 1.0);
        assert!(anim.completed);
    }

    #[test]
    fn test_animation_with_delay() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );
        anim.delay = 0.5;

        // Update during delay
        assert!(anim.update(0.3));
        assert!(!anim.started);
        assert_eq!(anim.elapsed, 0.0);

        // Update past delay
        assert!(anim.update(0.3));
        assert!(anim.started);
        assert!((anim.elapsed - 0.1).abs() < 0.001); // 0.3 - remaining 0.2 delay
    }

    #[test]
    fn test_animation_loop_mode_once() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );
        anim.loop_mode = LoopMode::Once;

        assert!(anim.update(0.5));
        assert!(!anim.update(0.6)); // Should complete
        assert!(anim.completed);
    }

    #[test]
    fn test_animation_loop_mode_loop() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );
        anim.loop_mode = LoopMode::Loop;

        assert!(anim.update(0.5));
        assert!(anim.update(0.6)); // Should loop, not complete
        assert!(!anim.completed);
        assert!(anim.elapsed < 1.0); // Should have wrapped around
    }

    #[test]
    fn test_get_current_value_vec2() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::AnchoredPosition,
            AnimationValue::Vec2(Vec2::ZERO),
            AnimationValue::Vec2(Vec2::new(100.0, 100.0)),
            1.0,
        );
        anim.easing = EasingFunction::Linear;

        // At start
        if let AnimationValue::Vec2(val) = anim.get_current_value() {
            assert_eq!(val, Vec2::ZERO);
        } else {
            panic!("Expected Vec2 value");
        }

        // At halfway
        anim.update(0.5);
        if let AnimationValue::Vec2(val) = anim.get_current_value() {
            assert!((val.x - 50.0).abs() < 0.01);
            assert!((val.y - 50.0).abs() < 0.01);
        } else {
            panic!("Expected Vec2 value");
        }

        // At end
        anim.update(0.5);
        if let AnimationValue::Vec2(val) = anim.get_current_value() {
            assert_eq!(val, Vec2::new(100.0, 100.0));
        } else {
            panic!("Expected Vec2 value");
        }
    }

    #[test]
    fn test_get_current_value_float() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );
        anim.easing = EasingFunction::Linear;

        anim.update(0.5);
        if let AnimationValue::Float(val) = anim.get_current_value() {
            assert!((val - 0.5).abs() < 0.01);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_get_current_value_color() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Color,
            AnimationValue::Color([0.0, 0.0, 0.0, 1.0]),
            AnimationValue::Color([1.0, 1.0, 1.0, 1.0]),
            1.0,
        );
        anim.easing = EasingFunction::Linear;

        anim.update(0.5);
        if let AnimationValue::Color(val) = anim.get_current_value() {
            assert!((val[0] - 0.5).abs() < 0.01);
            assert!((val[1] - 0.5).abs() < 0.01);
            assert!((val[2] - 0.5).abs() < 0.01);
            assert_eq!(val[3], 1.0);
        } else {
            panic!("Expected Color value");
        }
    }

    #[test]
    fn test_apply_to_rect_transform() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::AnchoredPosition,
            AnimationValue::Vec2(Vec2::ZERO),
            AnimationValue::Vec2(Vec2::new(100.0, 100.0)),
            1.0,
        );
        anim.update(0.5);

        let mut rect_transform = RectTransform::default();
        anim.apply_to_rect_transform(&mut rect_transform);

        assert!((rect_transform.anchored_position.x - 50.0).abs() < 0.01);
        assert!((rect_transform.anchored_position.y - 50.0).abs() < 0.01);
        assert!(rect_transform.dirty);
    }

    #[test]
    fn test_apply_to_ui_element() {
        let mut anim = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );
        anim.update(0.5);

        let mut ui_element = UIElement::default();
        anim.apply_to_ui_element(&mut ui_element);

        assert!((ui_element.alpha - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();

        let anim1 = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );

        let mut anim2 = UIAnimation::new(
            2,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            0.5,
        );
        anim2.on_complete = Some("callback".to_string());

        manager.add_animation(anim1);
        manager.add_animation(anim2);

        assert_eq!(manager.animation_count(), 2);

        // Update halfway - anim2 should complete
        let callbacks = manager.update(0.5);
        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0], "callback");
        assert_eq!(manager.animation_count(), 1);

        // Update to complete anim1
        let callbacks = manager.update(0.5);
        assert_eq!(callbacks.len(), 0);
        assert_eq!(manager.animation_count(), 0);
    }

    #[test]
    fn test_remove_animations_for_entity() {
        let mut manager = AnimationManager::new();

        let anim1 = UIAnimation::new(
            1,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );

        let anim2 = UIAnimation::new(
            2,
            AnimatedProperty::Alpha,
            AnimationValue::Float(0.0),
            AnimationValue::Float(1.0),
            1.0,
        );

        manager.add_animation(anim1);
        manager.add_animation(anim2);

        manager.remove_animations_for_entity(1);
        assert_eq!(manager.animation_count(), 1);
        assert_eq!(manager.get_animations()[0].entity, 2);
    }
}

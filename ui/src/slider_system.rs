//! Slider interaction system
//!
//! Handles slider value updates, handle positioning, and drag interactions.

use crate::{UISlider, SliderDirection, RectTransform, UIElement};
use crate::events::{UIEvent, UIEventType, UIEventDispatcher};
use glam::Vec2;
use std::collections::HashMap;

/// Entity type alias
pub type Entity = u64;

/// Slider interaction system
pub struct SliderSystem {
    /// Currently dragging sliders (entity -> drag start position)
    dragging_sliders: HashMap<Entity, Vec2>,
}

impl Default for SliderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderSystem {
    /// Create a new slider system
    pub fn new() -> Self {
        Self {
            dragging_sliders: HashMap::new(),
        }
    }
    
    /// Update slider values based on events
    pub fn update_from_events(
        &mut self,
        events: &[UIEvent],
        sliders: &mut HashMap<Entity, UISlider>,
        transforms: &HashMap<Entity, RectTransform>,
        elements: &HashMap<Entity, UIElement>,
        event_dispatcher: &UIEventDispatcher,
    ) {
        for event in events {
            match event {
                UIEvent::PointerDown(entity, pos) => {
                    if sliders.contains_key(entity) && elements.get(entity).map(|e| e.interactable).unwrap_or(false) {
                        self.dragging_sliders.insert(*entity, *pos);
                        self.update_slider_from_position(*entity, *pos, sliders, transforms);
                    }
                }
                UIEvent::Drag(entity, pos, _delta) => {
                    if self.dragging_sliders.contains_key(entity) {
                        self.update_slider_from_position(*entity, *pos, sliders, transforms);
                    }
                }
                UIEvent::PointerUp(entity, _pos) => {
                    self.dragging_sliders.remove(entity);
                }
                UIEvent::EndDrag(entity, _pos) => {
                    self.dragging_sliders.remove(entity);
                }
                _ => {}
            }
        }
    }
    
    /// Update slider value from pointer position
    fn update_slider_from_position(
        &self,
        entity: Entity,
        position: Vec2,
        sliders: &mut HashMap<Entity, UISlider>,
        transforms: &HashMap<Entity, RectTransform>,
    ) {
        if let Some(slider) = sliders.get_mut(&entity) {
            if let Some(transform) = transforms.get(&entity) {
                let rect = transform.rect;
                
                // Calculate normalized position (0-1) based on direction
                let normalized = match slider.direction {
                    SliderDirection::LeftToRight => {
                        if rect.width > 0.0 {
                            ((position.x - rect.x) / rect.width).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    SliderDirection::RightToLeft => {
                        if rect.width > 0.0 {
                            1.0 - ((position.x - rect.x) / rect.width).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    SliderDirection::BottomToTop => {
                        if rect.height > 0.0 {
                            ((position.y - rect.y) / rect.height).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                    SliderDirection::TopToBottom => {
                        if rect.height > 0.0 {
                            1.0 - ((position.y - rect.y) / rect.height).clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    }
                };
                
                // Convert normalized position to value
                let mut new_value = slider.min_value + normalized * (slider.max_value - slider.min_value);
                
                // Apply whole numbers constraint
                if slider.whole_numbers {
                    new_value = new_value.round();
                }
                
                // Clamp to min/max
                new_value = new_value.clamp(slider.min_value, slider.max_value);
                
                // Update value if changed
                if (new_value - slider.value).abs() > f32::EPSILON {
                    slider.value = new_value;
                    // Note: In a real implementation, we would trigger the on_value_changed callback here
                }
            }
        }
    }
    
    /// Update handle positions for all sliders
    pub fn update_handle_positions(
        &self,
        sliders: &HashMap<Entity, UISlider>,
        transforms: &mut HashMap<Entity, RectTransform>,
    ) {
        for (entity, slider) in sliders {
            if let Some(handle_entity) = slider.handle_rect {
                // Get parent rect first to avoid borrow conflicts
                let parent_rect = transforms.get(entity).map(|t| t.rect);
                
                if let Some(parent_rect) = parent_rect {
                    if let Some(handle_transform) = transforms.get_mut(&handle_entity) {
                        // Calculate normalized value (0-1)
                        let normalized = if slider.max_value > slider.min_value {
                            (slider.value - slider.min_value) / (slider.max_value - slider.min_value)
                        } else {
                            0.0
                        };
                        
                        let normalized = normalized.clamp(0.0, 1.0);
                        
                        // Calculate handle position based on direction
                        match slider.direction {
                            SliderDirection::LeftToRight => {
                                let x = parent_rect.x + normalized * parent_rect.width;
                                handle_transform.anchored_position.x = x - parent_rect.x;
                            }
                            SliderDirection::RightToLeft => {
                                let x = parent_rect.x + (1.0 - normalized) * parent_rect.width;
                                handle_transform.anchored_position.x = x - parent_rect.x;
                            }
                            SliderDirection::BottomToTop => {
                                let y = parent_rect.y + normalized * parent_rect.height;
                                handle_transform.anchored_position.y = y - parent_rect.y;
                            }
                            SliderDirection::TopToBottom => {
                                let y = parent_rect.y + (1.0 - normalized) * parent_rect.height;
                                handle_transform.anchored_position.y = y - parent_rect.y;
                            }
                        }
                        
                        handle_transform.dirty = true;
                    }
                }
            }
            
            // Update fill rect if present
            if let Some(fill_entity) = slider.fill_rect {
                // Get parent rect first to avoid borrow conflicts
                let parent_rect = transforms.get(entity).map(|t| t.rect);
                
                if let Some(parent_rect) = parent_rect {
                    if let Some(fill_transform) = transforms.get_mut(&fill_entity) {
                        // Calculate normalized value (0-1)
                        let normalized = if slider.max_value > slider.min_value {
                            (slider.value - slider.min_value) / (slider.max_value - slider.min_value)
                        } else {
                            0.0
                        };
                        
                        let normalized = normalized.clamp(0.0, 1.0);
                        
                        // Update fill size based on direction
                        match slider.direction {
                            SliderDirection::LeftToRight | SliderDirection::RightToLeft => {
                                fill_transform.size_delta.x = parent_rect.width * normalized;
                            }
                            SliderDirection::BottomToTop | SliderDirection::TopToBottom => {
                                fill_transform.size_delta.y = parent_rect.height * normalized;
                            }
                        }
                        
                        fill_transform.dirty = true;
                    }
                }
            }
        }
    }
    
    /// Clamp slider value to min/max range
    pub fn clamp_slider_value(slider: &mut UISlider) {
        slider.value = slider.value.clamp(slider.min_value, slider.max_value);
        
        if slider.whole_numbers {
            slider.value = slider.value.round();
        }
    }
    
    /// Set slider value programmatically
    pub fn set_slider_value(
        entity: Entity,
        value: f32,
        sliders: &mut HashMap<Entity, UISlider>,
        transforms: &mut HashMap<Entity, RectTransform>,
    ) {
        if let Some(slider) = sliders.get_mut(&entity) {
            let mut new_value = value;
            
            // Apply whole numbers constraint
            if slider.whole_numbers {
                new_value = new_value.round();
            }
            
            // Clamp to min/max
            new_value = new_value.clamp(slider.min_value, slider.max_value);
            
            slider.value = new_value;
        }
        
        // Update handle position
        let slider_system = SliderSystem::new();
        slider_system.update_handle_positions(sliders, transforms);
    }
    
    /// Clear dragging state
    pub fn clear(&mut self) {
        self.dragging_sliders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    fn create_test_slider() -> UISlider {
        UISlider {
            fill_rect: None,
            handle_rect: Some(2),
            direction: SliderDirection::LeftToRight,
            min_value: 0.0,
            max_value: 100.0,
            value: 50.0,
            whole_numbers: false,
            on_value_changed: None,
        }
    }

    fn create_test_transform(x: f32, y: f32, width: f32, height: f32) -> RectTransform {
        let mut transform = RectTransform::default();
        transform.rect = Rect { x, y, width, height };
        transform
    }

    #[test]
    fn test_slider_value_clamping() {
        let mut slider = create_test_slider();
        
        slider.value = 150.0;
        SliderSystem::clamp_slider_value(&mut slider);
        assert_eq!(slider.value, 100.0);
        
        slider.value = -50.0;
        SliderSystem::clamp_slider_value(&mut slider);
        assert_eq!(slider.value, 0.0);
        
        slider.value = 50.0;
        SliderSystem::clamp_slider_value(&mut slider);
        assert_eq!(slider.value, 50.0);
    }

    #[test]
    fn test_slider_whole_numbers() {
        let mut slider = create_test_slider();
        slider.whole_numbers = true;
        
        slider.value = 50.7;
        SliderSystem::clamp_slider_value(&mut slider);
        assert_eq!(slider.value, 51.0);
        
        slider.value = 50.3;
        SliderSystem::clamp_slider_value(&mut slider);
        assert_eq!(slider.value, 50.0);
    }

    #[test]
    fn test_handle_position_left_to_right() {
        let system = SliderSystem::new();
        let mut sliders = HashMap::new();
        let mut transforms = HashMap::new();
        
        let slider = create_test_slider();
        sliders.insert(1, slider);
        
        transforms.insert(1, create_test_transform(0.0, 0.0, 200.0, 20.0));
        transforms.insert(2, create_test_transform(0.0, 0.0, 10.0, 20.0));
        
        system.update_handle_positions(&sliders, &mut transforms);
        
        let handle_transform = transforms.get(&2).unwrap();
        // Value is 50.0 out of 0-100, so normalized is 0.5
        // Position should be at 0.5 * 200.0 = 100.0
        assert_eq!(handle_transform.anchored_position.x, 100.0);
    }

    #[test]
    fn test_handle_position_right_to_left() {
        let system = SliderSystem::new();
        let mut sliders = HashMap::new();
        let mut transforms = HashMap::new();
        
        let mut slider = create_test_slider();
        slider.direction = SliderDirection::RightToLeft;
        sliders.insert(1, slider);
        
        transforms.insert(1, create_test_transform(0.0, 0.0, 200.0, 20.0));
        transforms.insert(2, create_test_transform(0.0, 0.0, 10.0, 20.0));
        
        system.update_handle_positions(&sliders, &mut transforms);
        
        let handle_transform = transforms.get(&2).unwrap();
        // Value is 50.0 out of 0-100, so normalized is 0.5
        // For right-to-left, position should be at (1.0 - 0.5) * 200.0 = 100.0
        assert_eq!(handle_transform.anchored_position.x, 100.0);
    }

    #[test]
    fn test_set_slider_value() {
        let mut sliders = HashMap::new();
        let mut transforms = HashMap::new();
        
        sliders.insert(1, create_test_slider());
        transforms.insert(1, create_test_transform(0.0, 0.0, 200.0, 20.0));
        transforms.insert(2, create_test_transform(0.0, 0.0, 10.0, 20.0));
        
        SliderSystem::set_slider_value(1, 75.0, &mut sliders, &mut transforms);
        
        let slider = sliders.get(&1).unwrap();
        assert_eq!(slider.value, 75.0);
    }

    #[test]
    fn test_set_slider_value_with_clamping() {
        let mut sliders = HashMap::new();
        let mut transforms = HashMap::new();
        
        sliders.insert(1, create_test_slider());
        transforms.insert(1, create_test_transform(0.0, 0.0, 200.0, 20.0));
        
        SliderSystem::set_slider_value(1, 150.0, &mut sliders, &mut transforms);
        
        let slider = sliders.get(&1).unwrap();
        assert_eq!(slider.value, 100.0);
    }

    #[test]
    fn test_update_from_pointer_down() {
        let mut system = SliderSystem::new();
        let mut sliders = HashMap::new();
        let transforms = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        sliders.insert(1, create_test_slider());
        
        // Add an interactable element
        let mut element = UIElement::default();
        element.interactable = true;
        elements.insert(1, element);
        
        let events = vec![UIEvent::PointerDown(1, Vec2::new(100.0, 10.0))];
        system.update_from_events(&events, &mut sliders, &transforms, &elements, &dispatcher);
        
        assert!(system.dragging_sliders.contains_key(&1));
    }

    #[test]
    fn test_update_from_pointer_up() {
        let mut system = SliderSystem::new();
        let mut sliders = HashMap::new();
        let transforms = HashMap::new();
        let elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        sliders.insert(1, create_test_slider());
        system.dragging_sliders.insert(1, Vec2::ZERO);
        
        let events = vec![UIEvent::PointerUp(1, Vec2::new(100.0, 10.0))];
        system.update_from_events(&events, &mut sliders, &transforms, &elements, &dispatcher);
        
        assert!(!system.dragging_sliders.contains_key(&1));
    }
}

//! Toggle interaction system
//!
//! Handles toggle state changes and visual updates.

use crate::{UIToggle, ToggleTransition, UIElement};
use crate::events::{UIEvent, UIEventType, UIEventDispatcher};
use std::collections::HashMap;

/// Entity type alias
pub type Entity = u64;

/// Toggle interaction system
pub struct ToggleSystem {
    /// Pending toggle state changes (entity -> new state)
    pending_changes: HashMap<Entity, bool>,
}

impl Default for ToggleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ToggleSystem {
    /// Create a new toggle system
    pub fn new() -> Self {
        Self {
            pending_changes: HashMap::new(),
        }
    }
    
    /// Update toggle states based on events
    pub fn update_from_events(
        &mut self,
        events: &[UIEvent],
        toggles: &mut HashMap<Entity, UIToggle>,
        elements: &HashMap<Entity, UIElement>,
        event_dispatcher: &UIEventDispatcher,
    ) {
        for event in events {
            match event {
                UIEvent::PointerClick(entity, _pos) => {
                    if let Some(toggle) = toggles.get(entity) {
                        // Check if the toggle is interactable
                        if elements.get(entity).map(|e| e.interactable).unwrap_or(false) {
                            // Toggle the state
                            let new_state = !toggle.is_on;
                            self.pending_changes.insert(*entity, new_state);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Apply pending changes
        for (entity, new_state) in self.pending_changes.drain() {
            if let Some(toggle) = toggles.get_mut(&entity) {
                toggle.is_on = new_state;
                // Note: In a real implementation, we would trigger the on_value_changed callback here
            }
        }
    }
    
    /// Update visual states for all toggles
    pub fn update_visual_states(
        &self,
        toggles: &HashMap<Entity, UIToggle>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        for (entity, toggle) in toggles {
            if let Some(graphic_entity) = toggle.graphic {
                if let Some(graphic_element) = elements.get_mut(&graphic_entity) {
                    // Update visibility based on toggle state
                    match toggle.toggle_transition {
                        ToggleTransition::None => {
                            // Simply show/hide the graphic
                            graphic_element.alpha = if toggle.is_on { 1.0 } else { 0.0 };
                        }
                        ToggleTransition::Fade => {
                            // Set target alpha (actual fading would be handled by animation system)
                            graphic_element.alpha = if toggle.is_on { 1.0 } else { 0.0 };
                        }
                    }
                }
            }
        }
    }
    
    /// Set toggle state programmatically
    pub fn set_toggle_state(
        entity: Entity,
        is_on: bool,
        toggles: &mut HashMap<Entity, UIToggle>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        if let Some(toggle) = toggles.get_mut(&entity) {
            toggle.is_on = is_on;
        }
        
        // Update visual state
        let system = ToggleSystem::new();
        system.update_visual_states(toggles, elements);
    }
    
    /// Toggle the state programmatically
    pub fn toggle_state(
        entity: Entity,
        toggles: &mut HashMap<Entity, UIToggle>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        if let Some(toggle) = toggles.get(&entity) {
            let new_state = !toggle.is_on;
            Self::set_toggle_state(entity, new_state, toggles, elements);
        }
    }
    
    /// Check if a toggle is on
    pub fn is_toggle_on(entity: Entity, toggles: &HashMap<Entity, UIToggle>) -> bool {
        toggles.get(&entity).map(|t| t.is_on).unwrap_or(false)
    }
    
    /// Clear pending changes
    pub fn clear(&mut self) {
        self.pending_changes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_toggle() -> UIToggle {
        UIToggle {
            graphic: Some(2),
            is_on: false,
            toggle_transition: ToggleTransition::None,
            on_value_changed: None,
        }
    }

    fn create_test_element(interactable: bool) -> UIElement {
        UIElement {
            raycast_target: true,
            blocks_raycasts: true,
            z_order: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            alpha: 1.0,
            interactable,
            ignore_layout: false,
            canvas_entity: None,
        }
    }

    #[test]
    fn test_toggle_on_click() {
        let mut system = ToggleSystem::new();
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        toggles.insert(1, create_test_toggle());
        elements.insert(1, create_test_element(true));
        
        assert!(!toggles.get(&1).unwrap().is_on);
        
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut toggles, &elements, &dispatcher);
        
        assert!(toggles.get(&1).unwrap().is_on);
    }

    #[test]
    fn test_toggle_multiple_clicks() {
        let mut system = ToggleSystem::new();
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        toggles.insert(1, create_test_toggle());
        elements.insert(1, create_test_element(true));
        
        // First click - turn on
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut toggles, &elements, &dispatcher);
        assert!(toggles.get(&1).unwrap().is_on);
        
        // Second click - turn off
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut toggles, &elements, &dispatcher);
        assert!(!toggles.get(&1).unwrap().is_on);
    }

    #[test]
    fn test_toggle_not_interactable() {
        let mut system = ToggleSystem::new();
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        toggles.insert(1, create_test_toggle());
        elements.insert(1, create_test_element(false)); // Not interactable
        
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut toggles, &elements, &dispatcher);
        
        // State should not change
        assert!(!toggles.get(&1).unwrap().is_on);
    }

    #[test]
    fn test_set_toggle_state() {
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        
        toggles.insert(1, create_test_toggle());
        elements.insert(2, create_test_element(true)); // Graphic element
        
        ToggleSystem::set_toggle_state(1, true, &mut toggles, &mut elements);
        
        assert!(toggles.get(&1).unwrap().is_on);
    }

    #[test]
    fn test_toggle_state_programmatic() {
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        
        toggles.insert(1, create_test_toggle());
        elements.insert(2, create_test_element(true));
        
        // Toggle on
        ToggleSystem::toggle_state(1, &mut toggles, &mut elements);
        assert!(toggles.get(&1).unwrap().is_on);
        
        // Toggle off
        ToggleSystem::toggle_state(1, &mut toggles, &mut elements);
        assert!(!toggles.get(&1).unwrap().is_on);
    }

    #[test]
    fn test_is_toggle_on() {
        let mut toggles = HashMap::new();
        
        toggles.insert(1, create_test_toggle());
        assert!(!ToggleSystem::is_toggle_on(1, &toggles));
        
        toggles.get_mut(&1).unwrap().is_on = true;
        assert!(ToggleSystem::is_toggle_on(1, &toggles));
    }

    #[test]
    fn test_visual_state_none_transition() {
        let system = ToggleSystem::new();
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        
        let mut toggle = create_test_toggle();
        toggle.toggle_transition = ToggleTransition::None;
        toggle.is_on = true;
        toggles.insert(1, toggle);
        
        elements.insert(2, create_test_element(true));
        
        system.update_visual_states(&toggles, &mut elements);
        
        let graphic = elements.get(&2).unwrap();
        assert_eq!(graphic.alpha, 1.0);
    }

    #[test]
    fn test_visual_state_fade_transition() {
        let system = ToggleSystem::new();
        let mut toggles = HashMap::new();
        let mut elements = HashMap::new();
        
        let mut toggle = create_test_toggle();
        toggle.toggle_transition = ToggleTransition::Fade;
        toggle.is_on = false;
        toggles.insert(1, toggle);
        
        elements.insert(2, create_test_element(true));
        
        system.update_visual_states(&toggles, &mut elements);
        
        let graphic = elements.get(&2).unwrap();
        assert_eq!(graphic.alpha, 0.0);
    }

    #[test]
    fn test_clear_pending_changes() {
        let mut system = ToggleSystem::new();
        
        system.pending_changes.insert(1, true);
        system.pending_changes.insert(2, false);
        
        system.clear();
        
        assert!(system.pending_changes.is_empty());
    }
}

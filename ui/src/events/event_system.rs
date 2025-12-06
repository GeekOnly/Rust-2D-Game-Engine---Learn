//! UI Event system for dispatching events to UI elements

use std::collections::HashMap;
use super::{UIEvent, UIEventType};
use super::raycast::Entity;

/// Callback function type for UI events
pub type EventCallback = Box<dyn Fn(&UIEvent) + Send + Sync>;

/// UI Event dispatcher
pub struct UIEventDispatcher {
    /// Registered event listeners (entity -> event type -> callbacks)
    listeners: HashMap<Entity, HashMap<UIEventType, Vec<String>>>,
    
    /// Lua callback registry (callback name -> callback function)
    /// In a real implementation, this would integrate with the Lua runtime
    lua_callbacks: HashMap<String, EventCallback>,
}

impl Default for UIEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UIEventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            lua_callbacks: HashMap::new(),
        }
    }
    
    /// Register an event listener for an entity
    pub fn register_listener(&mut self, entity: Entity, event_type: UIEventType, callback: String) {
        self.listeners
            .entry(entity)
            .or_insert_with(HashMap::new)
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(callback);
    }
    
    /// Unregister all listeners for an entity
    pub fn unregister_entity(&mut self, entity: Entity) {
        self.listeners.remove(&entity);
    }
    
    /// Unregister a specific listener for an entity
    pub fn unregister_listener(&mut self, entity: Entity, event_type: UIEventType, callback: &str) {
        if let Some(entity_listeners) = self.listeners.get_mut(&entity) {
            if let Some(callbacks) = entity_listeners.get_mut(&event_type) {
                callbacks.retain(|cb| cb != callback);
                
                // Clean up empty entries
                if callbacks.is_empty() {
                    entity_listeners.remove(&event_type);
                }
            }
            
            if entity_listeners.is_empty() {
                self.listeners.remove(&entity);
            }
        }
    }
    
    /// Register a Lua callback function
    /// In a real implementation, this would store a reference to the Lua function
    pub fn register_lua_callback(&mut self, name: String, callback: EventCallback) {
        self.lua_callbacks.insert(name, callback);
    }
    
    /// Dispatch a single event
    pub fn dispatch_event(&self, event: &UIEvent) {
        let entity = match event {
            UIEvent::PointerEnter(e) => *e,
            UIEvent::PointerExit(e) => *e,
            UIEvent::PointerDown(e, _) => *e,
            UIEvent::PointerUp(e, _) => *e,
            UIEvent::PointerClick(e, _) => *e,
            UIEvent::BeginDrag(e, _) => *e,
            UIEvent::Drag(e, _, _) => *e,
            UIEvent::EndDrag(e, _) => *e,
            UIEvent::Scroll(e, _) => *e,
        };
        
        let event_type = match event {
            UIEvent::PointerEnter(_) => UIEventType::OnPointerEnter,
            UIEvent::PointerExit(_) => UIEventType::OnPointerExit,
            UIEvent::PointerDown(_, _) => UIEventType::OnPointerDown,
            UIEvent::PointerUp(_, _) => UIEventType::OnPointerUp,
            UIEvent::PointerClick(_, _) => UIEventType::OnPointerClick,
            UIEvent::BeginDrag(_, _) => UIEventType::OnBeginDrag,
            UIEvent::Drag(_, _, _) => UIEventType::OnDrag,
            UIEvent::EndDrag(_, _) => UIEventType::OnEndDrag,
            UIEvent::Scroll(_, _) => UIEventType::OnScroll,
        };
        
        // Find listeners for this entity and event type
        if let Some(entity_listeners) = self.listeners.get(&entity) {
            if let Some(callbacks) = entity_listeners.get(&event_type) {
                for callback_name in callbacks {
                    // Invoke the callback
                    if let Some(callback) = self.lua_callbacks.get(callback_name) {
                        callback(event);
                    }
                }
            }
        }
    }
    
    /// Dispatch multiple events
    pub fn dispatch_events(&self, events: &[UIEvent]) {
        for event in events {
            self.dispatch_event(event);
        }
    }
    
    /// Get all listeners for an entity
    pub fn get_entity_listeners(&self, entity: Entity) -> Option<&HashMap<UIEventType, Vec<String>>> {
        self.listeners.get(&entity)
    }
    
    /// Check if an entity has any listeners
    pub fn has_listeners(&self, entity: Entity) -> bool {
        self.listeners.contains_key(&entity)
    }
    
    /// Get the number of registered entities
    pub fn entity_count(&self) -> usize {
        self.listeners.len()
    }
    
    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
    }
}

/// Button state manager for updating button states based on events
pub struct ButtonStateManager {
    /// Current button states (entity -> state)
    button_states: HashMap<Entity, ButtonState>,
}

/// Button state enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Highlighted,
    Pressed,
    Disabled,
}

impl Default for ButtonStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonStateManager {
    /// Create a new button state manager
    pub fn new() -> Self {
        Self {
            button_states: HashMap::new(),
        }
    }
    
    /// Update button states based on events
    pub fn update_from_events(&mut self, events: &[UIEvent], disabled_buttons: &HashMap<Entity, bool>) {
        for event in events {
            match event {
                UIEvent::PointerEnter(entity) => {
                    if !disabled_buttons.get(entity).copied().unwrap_or(false) {
                        self.button_states.insert(*entity, ButtonState::Highlighted);
                    }
                }
                UIEvent::PointerExit(entity) => {
                    if !disabled_buttons.get(entity).copied().unwrap_or(false) {
                        self.button_states.insert(*entity, ButtonState::Normal);
                    }
                }
                UIEvent::PointerDown(entity, _) => {
                    if !disabled_buttons.get(entity).copied().unwrap_or(false) {
                        self.button_states.insert(*entity, ButtonState::Pressed);
                    }
                }
                UIEvent::PointerUp(entity, _) => {
                    if !disabled_buttons.get(entity).copied().unwrap_or(false) {
                        // Check if still hovered
                        let state = self.button_states.get(entity).copied().unwrap_or(ButtonState::Normal);
                        if state == ButtonState::Pressed {
                            self.button_states.insert(*entity, ButtonState::Highlighted);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Update disabled buttons
        for (entity, &is_disabled) in disabled_buttons {
            if is_disabled {
                self.button_states.insert(*entity, ButtonState::Disabled);
            }
        }
    }
    
    /// Get the current state of a button
    pub fn get_state(&self, entity: Entity) -> ButtonState {
        self.button_states.get(&entity).copied().unwrap_or(ButtonState::Normal)
    }
    
    /// Set the state of a button
    pub fn set_state(&mut self, entity: Entity, state: ButtonState) {
        self.button_states.insert(entity, state);
    }
    
    /// Remove a button from tracking
    pub fn remove_button(&mut self, entity: Entity) {
        self.button_states.remove(&entity);
    }
    
    /// Clear all button states
    pub fn clear(&mut self) {
        self.button_states.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_event_dispatcher_creation() {
        let dispatcher = UIEventDispatcher::new();
        assert_eq!(dispatcher.entity_count(), 0);
    }

    #[test]
    fn test_register_listener() {
        let mut dispatcher = UIEventDispatcher::new();
        
        dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_click".to_string());
        
        assert!(dispatcher.has_listeners(1));
        assert_eq!(dispatcher.entity_count(), 1);
        
        let listeners = dispatcher.get_entity_listeners(1).unwrap();
        assert!(listeners.contains_key(&UIEventType::OnPointerClick));
    }

    #[test]
    fn test_unregister_listener() {
        let mut dispatcher = UIEventDispatcher::new();
        
        dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_click".to_string());
        dispatcher.unregister_listener(1, UIEventType::OnPointerClick, "on_click");
        
        assert!(!dispatcher.has_listeners(1));
    }

    #[test]
    fn test_unregister_entity() {
        let mut dispatcher = UIEventDispatcher::new();
        
        dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_click".to_string());
        dispatcher.register_listener(1, UIEventType::OnPointerEnter, "on_enter".to_string());
        
        dispatcher.unregister_entity(1);
        
        assert!(!dispatcher.has_listeners(1));
    }

    #[test]
    fn test_dispatch_event() {
        let mut dispatcher = UIEventDispatcher::new();
        
        // Register a callback
        let callback = Box::new(move |_event: &UIEvent| {
            // In a real test, we'd verify the event details
        });
        
        dispatcher.register_lua_callback("on_click".to_string(), callback);
        dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_click".to_string());
        
        // Dispatch an event
        let event = UIEvent::PointerClick(1, Vec2::new(50.0, 50.0));
        dispatcher.dispatch_event(&event);
        
        // The callback should have been invoked (we can't easily test this without more infrastructure)
    }

    #[test]
    fn test_button_state_manager() {
        let mut manager = ButtonStateManager::new();
        
        assert_eq!(manager.get_state(1), ButtonState::Normal);
        
        manager.set_state(1, ButtonState::Highlighted);
        assert_eq!(manager.get_state(1), ButtonState::Highlighted);
        
        manager.remove_button(1);
        assert_eq!(manager.get_state(1), ButtonState::Normal);
    }

    #[test]
    fn test_button_state_from_events() {
        let mut manager = ButtonStateManager::new();
        let disabled = HashMap::new();
        
        // Pointer enter -> Highlighted
        let events = vec![UIEvent::PointerEnter(1)];
        manager.update_from_events(&events, &disabled);
        assert_eq!(manager.get_state(1), ButtonState::Highlighted);
        
        // Pointer down -> Pressed
        let events = vec![UIEvent::PointerDown(1, Vec2::ZERO)];
        manager.update_from_events(&events, &disabled);
        assert_eq!(manager.get_state(1), ButtonState::Pressed);
        
        // Pointer up -> Highlighted (if still hovered)
        let events = vec![UIEvent::PointerUp(1, Vec2::ZERO)];
        manager.update_from_events(&events, &disabled);
        assert_eq!(manager.get_state(1), ButtonState::Highlighted);
        
        // Pointer exit -> Normal
        let events = vec![UIEvent::PointerExit(1)];
        manager.update_from_events(&events, &disabled);
        assert_eq!(manager.get_state(1), ButtonState::Normal);
    }

    #[test]
    fn test_button_state_disabled() {
        let mut manager = ButtonStateManager::new();
        let mut disabled = HashMap::new();
        disabled.insert(1, true);
        
        // Try to highlight a disabled button
        let events = vec![UIEvent::PointerEnter(1)];
        manager.update_from_events(&events, &disabled);
        assert_eq!(manager.get_state(1), ButtonState::Disabled);
    }

    #[test]
    fn test_clear_dispatcher() {
        let mut dispatcher = UIEventDispatcher::new();
        
        dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_click".to_string());
        dispatcher.register_listener(2, UIEventType::OnPointerClick, "on_click".to_string());
        
        assert_eq!(dispatcher.entity_count(), 2);
        
        dispatcher.clear();
        assert_eq!(dispatcher.entity_count(), 0);
    }

    #[test]
    fn test_clear_button_states() {
        let mut manager = ButtonStateManager::new();
        
        manager.set_state(1, ButtonState::Highlighted);
        manager.set_state(2, ButtonState::Pressed);
        
        manager.clear();
        
        assert_eq!(manager.get_state(1), ButtonState::Normal);
        assert_eq!(manager.get_state(2), ButtonState::Normal);
    }
}

//! Input event processing and UI event generation

use glam::Vec2;
use std::collections::{HashMap, HashSet};
use super::raycast::{UIRaycastSystem, Entity};
use super::UIEvent;

/// Mouse button enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input state for mouse/touch
#[derive(Clone, Debug)]
pub struct InputState {
    /// Current mouse position
    pub mouse_position: Vec2,
    
    /// Previous mouse position (for delta calculation)
    pub previous_mouse_position: Vec2,
    
    /// Mouse buttons currently pressed
    pub buttons_down: HashSet<MouseButton>,
    
    /// Mouse buttons pressed this frame
    pub buttons_pressed: HashSet<MouseButton>,
    
    /// Mouse buttons released this frame
    pub buttons_released: HashSet<MouseButton>,
    
    /// Scroll delta this frame
    pub scroll_delta: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
            previous_mouse_position: Vec2::ZERO,
            buttons_down: HashSet::new(),
            buttons_pressed: HashSet::new(),
            buttons_released: HashSet::new(),
            scroll_delta: 0.0,
        }
    }
}

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update the input state for a new frame
    pub fn begin_frame(&mut self) {
        self.previous_mouse_position = self.mouse_position;
        self.buttons_pressed.clear();
        self.buttons_released.clear();
        self.scroll_delta = 0.0;
    }
    
    /// Set the current mouse position
    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.mouse_position = position;
    }
    
    /// Press a mouse button
    pub fn press_button(&mut self, button: MouseButton) {
        if !self.buttons_down.contains(&button) {
            self.buttons_down.insert(button);
            self.buttons_pressed.insert(button);
        }
    }
    
    /// Release a mouse button
    pub fn release_button(&mut self, button: MouseButton) {
        if self.buttons_down.remove(&button) {
            self.buttons_released.insert(button);
        }
    }
    
    /// Add scroll delta
    pub fn add_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }
    
    /// Check if a button is currently down
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.buttons_down.contains(&button)
    }
    
    /// Check if a button was pressed this frame
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button)
    }
    
    /// Check if a button was released this frame
    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.buttons_released.contains(&button)
    }
    
    /// Get the mouse delta (movement since last frame)
    pub fn get_mouse_delta(&self) -> Vec2 {
        self.mouse_position - self.previous_mouse_position
    }
}

/// UI Input event processor
pub struct UIInputHandler {
    /// Input state
    input_state: InputState,
    
    /// Currently hovered elements
    hovered_elements: HashSet<Entity>,
    
    /// Elements with pointer down
    pressed_elements: HashMap<Entity, Vec2>,
    
    /// Currently dragging element (entity, start position)
    dragging_element: Option<(Entity, Vec2)>,
    
    /// Drag threshold (minimum distance to start drag)
    drag_threshold: f32,
    
    /// Click threshold (maximum time between down and up for click)
    click_time_threshold: f32,
    
    /// Time tracking for clicks
    pointer_down_time: HashMap<Entity, f32>,
    
    /// Current time (updated each frame)
    current_time: f32,
}

impl Default for UIInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl UIInputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            input_state: InputState::new(),
            hovered_elements: HashSet::new(),
            pressed_elements: HashMap::new(),
            dragging_element: None,
            drag_threshold: 5.0,
            click_time_threshold: 0.3,
            pointer_down_time: HashMap::new(),
            current_time: 0.0,
        }
    }
    
    /// Set the drag threshold
    pub fn set_drag_threshold(&mut self, threshold: f32) {
        self.drag_threshold = threshold;
    }
    
    /// Set the click time threshold
    pub fn set_click_time_threshold(&mut self, threshold: f32) {
        self.click_time_threshold = threshold;
    }
    
    /// Begin a new frame
    pub fn begin_frame(&mut self, delta_time: f32) {
        self.input_state.begin_frame();
        self.current_time += delta_time;
    }
    
    /// Update mouse position
    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.input_state.set_mouse_position(position);
    }
    
    /// Press a mouse button
    pub fn press_button(&mut self, button: MouseButton) {
        self.input_state.press_button(button);
    }
    
    /// Release a mouse button
    pub fn release_button(&mut self, button: MouseButton) {
        self.input_state.release_button(button);
    }
    
    /// Add scroll input
    pub fn add_scroll(&mut self, delta: f32) {
        self.input_state.add_scroll(delta);
    }
    
    /// Process input and generate UI events
    /// Returns a list of events that occurred this frame
    pub fn process_input(&mut self, raycast_system: &UIRaycastSystem) -> Vec<UIEvent> {
        let mut events = Vec::new();
        
        // Get the current element under the mouse
        let current_hit = raycast_system.raycast_topmost(self.input_state.mouse_position);
        let current_entity = current_hit.as_ref().map(|hit| hit.entity);
        
        // Process hover events (enter/exit)
        self.process_hover_events(current_entity, &mut events);
        
        // Process pointer down events
        if self.input_state.is_button_pressed(MouseButton::Left) {
            if let Some(entity) = current_entity {
                events.push(UIEvent::PointerDown(entity, self.input_state.mouse_position));
                self.pressed_elements.insert(entity, self.input_state.mouse_position);
                self.pointer_down_time.insert(entity, self.current_time);
            }
        }
        
        // Process pointer up and click events
        if self.input_state.is_button_released(MouseButton::Left) {
            // Check all pressed elements
            let pressed: Vec<Entity> = self.pressed_elements.keys().copied().collect();
            
            for entity in pressed {
                let _down_pos = self.pressed_elements.remove(&entity).unwrap();
                events.push(UIEvent::PointerUp(entity, self.input_state.mouse_position));
                
                // Check if this element was being dragged
                let was_dragging = if let Some((drag_entity, _)) = self.dragging_element {
                    drag_entity == entity
                } else {
                    false
                };
                
                // Only generate click event if we weren't dragging
                if !was_dragging {
                    // Check if this is a click (pointer up on the same element, within time threshold)
                    if current_entity == Some(entity) {
                        if let Some(down_time) = self.pointer_down_time.remove(&entity) {
                            let elapsed = self.current_time - down_time;
                            if elapsed <= self.click_time_threshold {
                                events.push(UIEvent::PointerClick(entity, self.input_state.mouse_position));
                            }
                        }
                    }
                } else {
                    // Clean up the down time for dragged elements
                    self.pointer_down_time.remove(&entity);
                }
                
                // End drag if this element was being dragged
                if was_dragging {
                    events.push(UIEvent::EndDrag(entity, self.input_state.mouse_position));
                    self.dragging_element = None;
                }
            }
        }
        
        // Process drag events
        if self.input_state.is_button_down(MouseButton::Left) {
            let mouse_delta = self.input_state.get_mouse_delta();
            
            // Check if we should start dragging
            if self.dragging_element.is_none() {
                for (entity, down_pos) in &self.pressed_elements {
                    let distance = self.input_state.mouse_position.distance(*down_pos);
                    if distance >= self.drag_threshold {
                        // Start dragging
                        events.push(UIEvent::BeginDrag(*entity, self.input_state.mouse_position));
                        self.dragging_element = Some((*entity, *down_pos));
                        break;
                    }
                }
            }
            
            // Continue dragging
            if let Some((entity, _)) = self.dragging_element {
                if mouse_delta.length() > 0.0 {
                    events.push(UIEvent::Drag(
                        entity,
                        self.input_state.mouse_position,
                        mouse_delta,
                    ));
                }
            }
        }
        
        // Process scroll events
        if self.input_state.scroll_delta != 0.0 {
            if let Some(entity) = current_entity {
                events.push(UIEvent::Scroll(entity, self.input_state.scroll_delta));
            }
        }
        
        events
    }
    
    /// Process hover enter/exit events
    fn process_hover_events(&mut self, current_entity: Option<Entity>, events: &mut Vec<UIEvent>) {
        // Find elements that are no longer hovered
        let mut exited = Vec::new();
        for &entity in &self.hovered_elements {
            if current_entity != Some(entity) {
                exited.push(entity);
            }
        }
        
        // Generate exit events
        for entity in exited {
            self.hovered_elements.remove(&entity);
            events.push(UIEvent::PointerExit(entity));
        }
        
        // Check if we entered a new element
        if let Some(entity) = current_entity {
            if !self.hovered_elements.contains(&entity) {
                self.hovered_elements.insert(entity);
                events.push(UIEvent::PointerEnter(entity));
            }
        }
    }
    
    /// Get the current input state (for external systems)
    pub fn get_input_state(&self) -> &InputState {
        &self.input_state
    }
    
    /// Get the currently hovered elements
    pub fn get_hovered_elements(&self) -> &HashSet<Entity> {
        &self.hovered_elements
    }
    
    /// Get the currently pressed elements
    pub fn get_pressed_elements(&self) -> &HashMap<Entity, Vec2> {
        &self.pressed_elements
    }
    
    /// Get the currently dragging element
    pub fn get_dragging_element(&self) -> Option<(Entity, Vec2)> {
        self.dragging_element
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_creation() {
        let state = InputState::new();
        assert_eq!(state.mouse_position, Vec2::ZERO);
        assert!(state.buttons_down.is_empty());
        assert!(state.buttons_pressed.is_empty());
        assert!(state.buttons_released.is_empty());
        assert_eq!(state.scroll_delta, 0.0);
    }

    #[test]
    fn test_input_state_button_press() {
        let mut state = InputState::new();
        
        state.press_button(MouseButton::Left);
        assert!(state.is_button_down(MouseButton::Left));
        assert!(state.is_button_pressed(MouseButton::Left));
        assert!(!state.is_button_released(MouseButton::Left));
        
        // Begin new frame
        state.begin_frame();
        assert!(state.is_button_down(MouseButton::Left));
        assert!(!state.is_button_pressed(MouseButton::Left)); // Cleared
        assert!(!state.is_button_released(MouseButton::Left));
    }

    #[test]
    fn test_input_state_button_release() {
        let mut state = InputState::new();
        
        state.press_button(MouseButton::Left);
        state.begin_frame();
        state.release_button(MouseButton::Left);
        
        assert!(!state.is_button_down(MouseButton::Left));
        assert!(!state.is_button_pressed(MouseButton::Left));
        assert!(state.is_button_released(MouseButton::Left));
    }

    #[test]
    fn test_input_state_mouse_delta() {
        let mut state = InputState::new();
        
        state.set_mouse_position(Vec2::new(10.0, 20.0));
        state.begin_frame();
        state.set_mouse_position(Vec2::new(15.0, 25.0));
        
        let delta = state.get_mouse_delta();
        assert_eq!(delta, Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_input_state_scroll() {
        let mut state = InputState::new();
        
        state.add_scroll(10.0);
        assert_eq!(state.scroll_delta, 10.0);
        
        state.add_scroll(5.0);
        assert_eq!(state.scroll_delta, 15.0);
        
        state.begin_frame();
        assert_eq!(state.scroll_delta, 0.0);
    }

    #[test]
    fn test_input_handler_creation() {
        let handler = UIInputHandler::new();
        assert_eq!(handler.drag_threshold, 5.0);
        assert_eq!(handler.click_time_threshold, 0.3);
        assert!(handler.hovered_elements.is_empty());
        assert!(handler.pressed_elements.is_empty());
        assert!(handler.dragging_element.is_none());
    }

    #[test]
    fn test_input_handler_thresholds() {
        let mut handler = UIInputHandler::new();
        
        handler.set_drag_threshold(10.0);
        assert_eq!(handler.drag_threshold, 10.0);
        
        handler.set_click_time_threshold(0.5);
        assert_eq!(handler.click_time_threshold, 0.5);
    }
}

//! UI event system for input handling

use std::collections::{HashMap, HashSet};
use glam::Vec2;

/// UI Event types
#[derive(Clone, Debug)]
pub enum UIEvent {
    PointerEnter(u64), // Using u64 as placeholder for Entity
    PointerExit(u64),
    PointerDown(u64, Vec2),
    PointerUp(u64, Vec2),
    PointerClick(u64, Vec2),
    BeginDrag(u64, Vec2),
    Drag(u64, Vec2, Vec2), // entity, position, delta
    EndDrag(u64, Vec2),
    Scroll(u64, f32), // entity, delta
}

/// UI Event handler
pub struct UIEventHandler {
    /// Registered event listeners
    listeners: HashMap<u64, Vec<UIEventListener>>,
    
    /// Current hover state
    hovered_elements: HashSet<u64>,
    
    /// Current pressed state
    pressed_elements: HashMap<u64, Vec2>,
    
    /// Current drag state
    dragging_element: Option<(u64, Vec2)>,
}

impl Default for UIEventHandler {
    fn default() -> Self {
        Self {
            listeners: HashMap::new(),
            hovered_elements: HashSet::new(),
            pressed_elements: HashMap::new(),
            dragging_element: None,
        }
    }
}

impl UIEventHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_listener(&mut self, entity: u64, listener: UIEventListener) {
        self.listeners.entry(entity).or_insert_with(Vec::new).push(listener);
    }

    pub fn unregister_listeners(&mut self, entity: u64) {
        self.listeners.remove(&entity);
    }
}

/// UI Event listener
pub struct UIEventListener {
    pub event_type: UIEventType,
    pub callback: String, // Lua function name
}

/// UI Event type enumeration
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UIEventType {
    OnPointerEnter,
    OnPointerExit,
    OnPointerDown,
    OnPointerUp,
    OnPointerClick,
    OnBeginDrag,
    OnDrag,
    OnEndDrag,
    OnScroll,
}

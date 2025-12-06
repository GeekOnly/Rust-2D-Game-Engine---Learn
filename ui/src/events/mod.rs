//! UI event system for input handling

use glam::Vec2;

// Module declarations
pub mod raycast;
pub mod input_handler;
pub mod event_system;

// Re-export main types
pub use raycast::{UIRaycastSystem, RaycastHit, RaycastElement, Entity};
pub use input_handler::{UIInputHandler, InputState, MouseButton};
pub use event_system::{UIEventDispatcher, ButtonStateManager, ButtonState, EventCallback};

/// UI Event types
#[derive(Clone, Debug)]
pub enum UIEvent {
    PointerEnter(Entity),
    PointerExit(Entity),
    PointerDown(Entity, Vec2),
    PointerUp(Entity, Vec2),
    PointerClick(Entity, Vec2),
    BeginDrag(Entity, Vec2),
    Drag(Entity, Vec2, Vec2), // entity, position, delta
    EndDrag(Entity, Vec2),
    Scroll(Entity, f32), // entity, delta
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

/// Legacy UIEventHandler for backwards compatibility
/// This is now a wrapper around the new event system components
pub struct UIEventHandler {
    dispatcher: UIEventDispatcher,
}

impl Default for UIEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl UIEventHandler {
    pub fn new() -> Self {
        Self {
            dispatcher: UIEventDispatcher::new(),
        }
    }

    pub fn register_listener(&mut self, entity: Entity, listener: UIEventListener) {
        self.dispatcher.register_listener(entity, listener.event_type, listener.callback);
    }

    pub fn unregister_listeners(&mut self, entity: Entity) {
        self.dispatcher.unregister_entity(entity);
    }
    
    /// Get the underlying dispatcher
    pub fn dispatcher(&self) -> &UIEventDispatcher {
        &self.dispatcher
    }
    
    /// Get mutable access to the underlying dispatcher
    pub fn dispatcher_mut(&mut self) -> &mut UIEventDispatcher {
        &mut self.dispatcher
    }
}

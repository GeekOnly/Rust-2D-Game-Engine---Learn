//! Integration tests for the UI event system
//! 
//! These tests verify that raycasting, input handling, and event dispatching
//! work together correctly to process UI interactions.

use ui::{
    UIRaycastSystem, RaycastElement, UIInputHandler, UIEventDispatcher,
    MouseButton, UIEventType, Rect, ButtonStateManager,
};
use ui::events::ButtonState;
use glam::Vec2;
use std::collections::HashMap;

/// Helper function to create a test raycast element
fn create_element(
    entity: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    z_order: i32,
) -> RaycastElement {
    RaycastElement {
        entity,
        rect: Rect::new(x, y, width, height),
        raycast_target: true,
        blocks_raycasts: true,
        z_order,
        canvas_sort_order: 0,
        visible: true,
        interactable: true,
    }
}

#[test]
fn test_click_event_flow() {
    // Setup: Create a button element
    let mut raycast_system = UIRaycastSystem::new();
    let button_element = create_element(1, 0.0, 0.0, 100.0, 50.0, 0);
    raycast_system.update_elements(vec![button_element]);
    
    // Setup: Create input handler and event dispatcher
    let mut input_handler = UIInputHandler::new();
    let mut event_dispatcher = UIEventDispatcher::new();
    
    // Register a click listener
    event_dispatcher.register_listener(1, UIEventType::OnPointerClick, "on_button_click".to_string());
    
    // Simulate: Move mouse over button
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 25.0));
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get a pointer enter event
    assert_eq!(events.len(), 1);
    match &events[0] {
        ui::UIEvent::PointerEnter(entity) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerEnter event"),
    }
    
    // Simulate: Click the button (press)
    input_handler.begin_frame(0.016);
    input_handler.press_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get a pointer down event
    assert_eq!(events.len(), 1);
    match &events[0] {
        ui::UIEvent::PointerDown(entity, _) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerDown event"),
    }
    
    // Simulate: Release the button (within click threshold)
    input_handler.begin_frame(0.016);
    input_handler.release_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get pointer up and click events
    assert_eq!(events.len(), 2);
    match &events[0] {
        ui::UIEvent::PointerUp(entity, _) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerUp event"),
    }
    match &events[1] {
        ui::UIEvent::PointerClick(entity, _) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerClick event"),
    }
    
    // Dispatch events to verify listeners work
    event_dispatcher.dispatch_events(&events);
}

#[test]
fn test_drag_event_flow() {
    // Setup: Create a draggable element
    let mut raycast_system = UIRaycastSystem::new();
    let element = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    raycast_system.update_elements(vec![element]);
    
    let mut input_handler = UIInputHandler::new();
    input_handler.set_drag_threshold(5.0);
    
    // Start: Mouse down on element
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 50.0));
    input_handler.press_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    assert_eq!(events.len(), 2); // PointerEnter + PointerDown
    
    // Drag: Move mouse beyond threshold
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(60.0, 60.0));
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get BeginDrag and Drag events
    // The distance moved is sqrt((10^2 + 10^2)) = ~14.14, which is > 5.0 threshold
    assert!(events.len() >= 2, "Expected BeginDrag and Drag events");
    
    // Verify we got BeginDrag and Drag events
    let has_begin_drag = events.iter().any(|e| matches!(e, ui::UIEvent::BeginDrag(1, _)));
    let has_drag = events.iter().any(|e| matches!(e, ui::UIEvent::Drag(1, _, _)));
    assert!(has_begin_drag, "Expected BeginDrag event");
    assert!(has_drag, "Expected Drag event");
    
    // Continue dragging
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(70.0, 70.0));
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get Drag event
    assert_eq!(events.len(), 1);
    match &events[0] {
        ui::UIEvent::Drag(entity, _, delta) => {
            assert_eq!(*entity, 1);
            assert_eq!(*delta, Vec2::new(10.0, 10.0));
        }
        _ => panic!("Expected Drag event"),
    }
    
    // End drag
    input_handler.begin_frame(0.016);
    input_handler.release_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get PointerUp and EndDrag events (no click because we were dragging)
    assert_eq!(events.len(), 2);
    match &events[0] {
        ui::UIEvent::PointerUp(entity, _) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerUp event"),
    }
    match &events[1] {
        ui::UIEvent::EndDrag(entity, _) => assert_eq!(*entity, 1),
        _ => panic!("Expected EndDrag event"),
    }
}

#[test]
fn test_hover_events() {
    // Setup: Create two overlapping elements
    let mut raycast_system = UIRaycastSystem::new();
    let element1 = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    let element2 = create_element(2, 50.0, 50.0, 100.0, 100.0, 1); // Higher z-order
    raycast_system.update_elements(vec![element1, element2]);
    
    let mut input_handler = UIInputHandler::new();
    
    // Hover over element 2 (topmost)
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(75.0, 75.0));
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get PointerEnter for element 2 (topmost)
    assert_eq!(events.len(), 1);
    match &events[0] {
        ui::UIEvent::PointerEnter(entity) => assert_eq!(*entity, 2),
        _ => panic!("Expected PointerEnter event for element 2"),
    }
    
    // Move to element 1 only
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(25.0, 25.0));
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get PointerExit for element 2 and PointerEnter for element 1
    assert_eq!(events.len(), 2);
    match &events[0] {
        ui::UIEvent::PointerExit(entity) => assert_eq!(*entity, 2),
        _ => panic!("Expected PointerExit event for element 2"),
    }
    match &events[1] {
        ui::UIEvent::PointerEnter(entity) => assert_eq!(*entity, 1),
        _ => panic!("Expected PointerEnter event for element 1"),
    }
}

#[test]
fn test_scroll_events() {
    // Setup: Create a scrollable element
    let mut raycast_system = UIRaycastSystem::new();
    let element = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    raycast_system.update_elements(vec![element]);
    
    let mut input_handler = UIInputHandler::new();
    
    // Hover over element
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 50.0));
    let _ = input_handler.process_input(&raycast_system);
    
    // Scroll
    input_handler.begin_frame(0.016);
    input_handler.add_scroll(10.0);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should get Scroll event
    assert!(events.iter().any(|e| matches!(e, ui::UIEvent::Scroll(1, delta) if *delta == 10.0)));
}

#[test]
fn test_button_state_updates() {
    // Setup: Create a button
    let mut raycast_system = UIRaycastSystem::new();
    let button = create_element(1, 0.0, 0.0, 100.0, 50.0, 0);
    raycast_system.update_elements(vec![button]);
    
    let mut input_handler = UIInputHandler::new();
    let mut button_state_manager = ButtonStateManager::new();
    let disabled_buttons = HashMap::new();
    
    // Initial state should be Normal
    assert_eq!(button_state_manager.get_state(1), ButtonState::Normal);
    
    // Hover over button
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 25.0));
    let events = input_handler.process_input(&raycast_system);
    button_state_manager.update_from_events(&events, &disabled_buttons);
    
    // State should be Highlighted
    assert_eq!(button_state_manager.get_state(1), ButtonState::Highlighted);
    
    // Press button
    input_handler.begin_frame(0.016);
    input_handler.press_button(MouseButton::Left);
    let events = input_handler.process_input(&raycast_system);
    button_state_manager.update_from_events(&events, &disabled_buttons);
    
    // State should be Pressed
    assert_eq!(button_state_manager.get_state(1), ButtonState::Pressed);
    
    // Release button
    input_handler.begin_frame(0.016);
    input_handler.release_button(MouseButton::Left);
    let events = input_handler.process_input(&raycast_system);
    button_state_manager.update_from_events(&events, &disabled_buttons);
    
    // State should be Highlighted (still hovering)
    assert_eq!(button_state_manager.get_state(1), ButtonState::Highlighted);
    
    // Move mouse away
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(200.0, 200.0));
    let events = input_handler.process_input(&raycast_system);
    button_state_manager.update_from_events(&events, &disabled_buttons);
    
    // State should be Normal
    assert_eq!(button_state_manager.get_state(1), ButtonState::Normal);
}

#[test]
fn test_raycast_blocking() {
    // Setup: Create two overlapping elements, top one blocks raycasts
    let mut raycast_system = UIRaycastSystem::new();
    let bottom = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    let mut top = create_element(2, 0.0, 0.0, 100.0, 100.0, 1);
    top.blocks_raycasts = true;
    raycast_system.update_elements(vec![bottom, top]);
    
    let mut input_handler = UIInputHandler::new();
    
    // Click on the overlapping area
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 50.0));
    input_handler.press_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should only get events for the top element (which blocks)
    for event in &events {
        match event {
            ui::UIEvent::PointerEnter(entity) | ui::UIEvent::PointerDown(entity, _) => {
                assert_eq!(*entity, 2, "Only top element should receive events");
            }
            _ => {}
        }
    }
}

#[test]
fn test_non_interactable_elements() {
    // Setup: Create a non-interactable element
    let mut raycast_system = UIRaycastSystem::new();
    let mut element = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    element.interactable = false;
    raycast_system.update_elements(vec![element]);
    
    let mut input_handler = UIInputHandler::new();
    
    // Try to interact with the element
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 50.0));
    input_handler.press_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should not get any events (element is not interactable)
    assert_eq!(events.len(), 0);
}

#[test]
fn test_invisible_elements() {
    // Setup: Create an invisible element
    let mut raycast_system = UIRaycastSystem::new();
    let mut element = create_element(1, 0.0, 0.0, 100.0, 100.0, 0);
    element.visible = false;
    raycast_system.update_elements(vec![element]);
    
    let mut input_handler = UIInputHandler::new();
    
    // Try to interact with the element
    input_handler.begin_frame(0.016);
    input_handler.set_mouse_position(Vec2::new(50.0, 50.0));
    input_handler.press_button(MouseButton::Left);
    
    let events = input_handler.process_input(&raycast_system);
    
    // Should not get any events (element is invisible)
    assert_eq!(events.len(), 0);
}

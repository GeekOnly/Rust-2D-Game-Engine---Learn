//! Advanced UI Example
//!
//! This example demonstrates advanced UI components:
//! - UIScrollView with viewport clipping
//! - UISlider for value selection
//! - UIToggle for boolean options
//! - UIDropdown for option selection
//! - UIInputField for text input
//! - UI animations with easing functions
//! - Event system and callbacks

use ui::{
    RectTransform, UIElement, UIScrollView, MovementType,
    UISlider, SliderDirection,
    UIToggle, ToggleTransition,
    UIDropdown, DropdownOption,
    UIInputField, ContentType, LineType, InputType, KeyboardType, CharacterValidation,
    UIAnimation, AnimatedProperty, AnimationValue, EasingFunction, LoopMode,
    UIEventListener, UIEventType,
    Vec2, Vec4,
};

type Entity = u64;

fn main() {
    println!("=== Advanced UI Example ===\n");
    
    // Example 1: Scroll View
    demo_scroll_view();
    
    // Example 2: Slider
    demo_slider();
    
    // Example 3: Toggle
    demo_toggle();
    
    // Example 4: Dropdown
    demo_dropdown();
    
    // Example 5: Input Field
    demo_input_field();
    
    // Example 6: UI Animations
    demo_animations();
    
    // Example 7: Event System
    demo_events();
    
    println!("\n=== Example Complete ===");
}

/// Demonstrate Scroll View with viewport clipping
fn demo_scroll_view() {
    println!("--- Scroll View ---\n");
    
    // Create scroll view
    let scroll_view = UIScrollView {
        content: Some(2), // Content entity
        viewport: Some(3), // Viewport entity
        horizontal_scrollbar: Some(4),
        vertical_scrollbar: Some(5),
        movement_type: MovementType::Elastic,
        elasticity: 0.1,
        inertia: true,
        deceleration_rate: 0.135,
        scroll_sensitivity: 1.0,
        horizontal: false,
        vertical: true,
        normalized_position: Vec2::new(0.0, 1.0), // Start at top
        velocity: Vec2::ZERO,
    };
    
    println!("Created Scroll View:");
    println!("  Movement Type: {:?}", scroll_view.movement_type);
    println!("  Elasticity: {}", scroll_view.elasticity);
    println!("  Inertia: {}", scroll_view.inertia);
    println!("  Deceleration: {}", scroll_view.deceleration_rate);
    println!("  Vertical Scrolling: {}", scroll_view.vertical);
    println!("  Horizontal Scrolling: {}", scroll_view.horizontal);
    
    // Viewport (visible area)
    let viewport_transform = RectTransform::stretched(
        Vec2::ZERO,
        Vec2::ONE,
        Vec4::new(10.0, 10.0, 10.0, 10.0),
    );
    
    println!("\nViewport:");
    println!("  Anchor: Stretched to fill parent");
    println!("  Margins: 10px on all sides");
    
    // Content (scrollable content - larger than viewport)
    let content_transform = RectTransform::anchored(
        Vec2::new(0.5, 1.0), // Top-center anchor
        Vec2::ZERO,
        Vec2::new(380.0, 1000.0), // Tall content
    );
    
    println!("\nContent:");
    println!("  Size: {}x{}", content_transform.size_delta.x, content_transform.size_delta.y);
    println!("  Anchor: Top-Center");
    println!("  Note: Content is taller than viewport, enabling vertical scrolling");
    
    // Scrollbar
    println!("\nScrollbar:");
    println!("  Position reflects normalized scroll position (0-1)");
    println!("  Handle size reflects viewport/content ratio");
    println!();
}

/// Demonstrate Slider component
fn demo_slider() {
    println!("--- Slider ---\n");
    
    // Create a horizontal slider
    let slider = UISlider {
        fill_rect: Some(2),
        handle_rect: Some(3),
        direction: SliderDirection::LeftToRight,
        min_value: 0.0,
        max_value: 100.0,
        value: 50.0,
        whole_numbers: false,
        on_value_changed: Some("on_slider_changed".to_string()),
    };
    
    println!("Created Slider:");
    println!("  Direction: {:?}", slider.direction);
    println!("  Range: {} to {}", slider.min_value, slider.max_value);
    println!("  Current Value: {}", slider.value);
    println!("  Whole Numbers: {}", slider.whole_numbers);
    println!("  Callback: {:?}", slider.on_value_changed);
    
    // Demonstrate value clamping
    println!("\nValue Clamping:");
    let test_values: Vec<f32> = vec![-10.0, 0.0, 25.0, 50.0, 75.0, 100.0, 150.0];
    for &val in &test_values {
        let clamped = val.clamp(slider.min_value, slider.max_value);
        let normalized = (clamped - slider.min_value) / (slider.max_value - slider.min_value);
        println!("  Value {:.1} -> Clamped {:.1} -> Normalized {:.2}", val, clamped, normalized);
    }
    
    // Different slider types
    println!("\nSlider Variations:");
    println!("  Volume Slider: 0-100, whole numbers");
    println!("  Brightness Slider: 0.0-1.0, decimals");
    println!("  Temperature Slider: -20 to 40, whole numbers");
    println!("  Vertical Slider: Bottom to Top direction");
    println!();
}

/// Demonstrate Toggle component
fn demo_toggle() {
    println!("--- Toggle ---\n");
    
    // Create a toggle (checkbox)
    let toggle = UIToggle {
        graphic: Some(2), // Checkmark entity
        is_on: false,
        toggle_transition: ToggleTransition::Fade,
        on_value_changed: Some("on_toggle_changed".to_string()),
    };
    
    println!("Created Toggle:");
    println!("  Initial State: {}", if toggle.is_on { "ON" } else { "OFF" });
    println!("  Transition: {:?}", toggle.toggle_transition);
    println!("  Callback: {:?}", toggle.on_value_changed);
    
    // Demonstrate state changes
    println!("\nState Changes:");
    let mut current_state = toggle.is_on;
    for i in 1..=5 {
        current_state = !current_state;
        println!("  Click {}: State = {}", i, if current_state { "ON" } else { "OFF" });
    }
    
    // Common use cases
    println!("\nCommon Use Cases:");
    println!("  ☐ Enable Sound Effects");
    println!("  ☐ Show FPS Counter");
    println!("  ☐ Fullscreen Mode");
    println!("  ☐ Accept Terms and Conditions");
    println!();
}

/// Demonstrate Dropdown component
fn demo_dropdown() {
    println!("--- Dropdown ---\n");
    
    // Create a dropdown with options
    let dropdown = UIDropdown {
        template: Some(2), // Dropdown list template
        caption_text: Some(3), // Caption text entity
        item_text: Some(4), // Item text entity in template
        options: vec![
            DropdownOption {
                text: "Low".to_string(),
                image: None,
            },
            DropdownOption {
                text: "Medium".to_string(),
                image: None,
            },
            DropdownOption {
                text: "High".to_string(),
                image: None,
            },
            DropdownOption {
                text: "Ultra".to_string(),
                image: Some("ultra_icon".to_string()),
            },
        ],
        value: 1, // Selected index (Medium)
        on_value_changed: Some("on_dropdown_changed".to_string()),
    };
    
    println!("Created Dropdown:");
    println!("  Options: {}", dropdown.options.len());
    for (i, option) in dropdown.options.iter().enumerate() {
        let selected = if i == dropdown.value as usize { " (selected)" } else { "" };
        let icon = if option.image.is_some() { " [icon]" } else { "" };
        println!("    {}: {}{}{}", i, option.text, icon, selected);
    }
    println!("  Callback: {:?}", dropdown.on_value_changed);
    
    // Demonstrate selection
    println!("\nSelection Changes:");
    for i in 0..dropdown.options.len() {
        let option = &dropdown.options[i];
        println!("  Select index {}: Caption shows '{}'", i, option.text);
    }
    
    // Common use cases
    println!("\nCommon Use Cases:");
    println!("  Graphics Quality: Low, Medium, High, Ultra");
    println!("  Resolution: 1280x720, 1920x1080, 2560x1440");
    println!("  Language: English, Spanish, French, German");
    println!("  Difficulty: Easy, Normal, Hard, Expert");
    println!();
}

/// Demonstrate Input Field component
fn demo_input_field() {
    println!("--- Input Field ---\n");
    
    // Create a standard text input field
    let text_field = UIInputField {
        text_component: Some(2),
        placeholder: Some(3),
        text: String::new(),
        character_limit: 50,
        content_type: ContentType::Standard,
        line_type: LineType::SingleLine,
        input_type: InputType::Standard,
        keyboard_type: KeyboardType::Default,
        character_validation: CharacterValidation::None,
        caret_blink_rate: 0.85,
        caret_width: 1,
        selection_color: [0.65, 0.8, 1.0, 0.75],
        read_only: false,
        on_value_changed: Some("on_text_changed".to_string()),
        on_end_edit: Some("on_text_submitted".to_string()),
        caret_position: 0,
        selection_anchor: 0,
        is_focused: false,
    };
    
    println!("Created Text Input Field:");
    println!("  Content Type: {:?}", text_field.content_type);
    println!("  Line Type: {:?}", text_field.line_type);
    println!("  Character Limit: {}", text_field.character_limit);
    println!("  Read Only: {}", text_field.read_only);
    println!("  Callbacks: value_changed={:?}, end_edit={:?}", 
        text_field.on_value_changed, text_field.on_end_edit);
    
    // Create a number input field
    let number_field = UIInputField {
        content_type: ContentType::IntegerNumber,
        character_validation: CharacterValidation::Integer,
        ..text_field.clone()
    };
    
    println!("\nCreated Number Input Field:");
    println!("  Content Type: {:?}", number_field.content_type);
    println!("  Validation: {:?}", number_field.character_validation);
    println!("  Accepts: 0-9, minus sign");
    
    // Create a password field
    let password_field = UIInputField {
        content_type: ContentType::Password,
        character_limit: 32,
        ..text_field.clone()
    };
    
    println!("\nCreated Password Field:");
    println!("  Content Type: {:?}", password_field.content_type);
    println!("  Character Limit: {}", password_field.character_limit);
    println!("  Display: Characters shown as bullets (•)");
    
    // Create a multi-line text area
    let text_area = UIInputField {
        line_type: LineType::MultiLineNewline,
        character_limit: 500,
        ..text_field.clone()
    };
    
    println!("\nCreated Multi-line Text Area:");
    println!("  Line Type: {:?}", text_area.line_type);
    println!("  Character Limit: {}", text_area.character_limit);
    println!("  Supports: Line breaks, word wrap");
    
    // Content type examples
    println!("\nContent Type Validation:");
    println!("  Standard: Any characters");
    println!("  IntegerNumber: 0-9, minus sign");
    println!("  DecimalNumber: 0-9, decimal point, minus sign");
    println!("  Alphanumeric: A-Z, a-z, 0-9");
    println!("  Name: Letters, spaces, apostrophes");
    println!("  EmailAddress: Valid email format");
    println!("  Password: Hidden characters");
    println!();
}

/// Demonstrate UI animations
fn demo_animations() {
    println!("--- UI Animations ---\n");
    
    // Position animation
    let position_anim = UIAnimation {
        entity: 1,
        property: AnimatedProperty::AnchoredPosition,
        from: AnimationValue::Vec2(Vec2::new(0.0, 0.0)),
        to: AnimationValue::Vec2(Vec2::new(100.0, 50.0)),
        duration: 1.0,
        easing: EasingFunction::EaseOutQuad,
        delay: 0.0,
        loop_mode: LoopMode::Once,
        on_complete: Some("on_move_complete".to_string()),
        elapsed: 0.0,
        started: false,
        completed: false,
    };
    
    println!("Position Animation:");
    println!("  Property: {:?}", position_anim.property);
    println!("  From: (0, 0) -> To: (100, 50)");
    println!("  Duration: {}s", position_anim.duration);
    println!("  Easing: {:?}", position_anim.easing);
    println!("  Loop: {:?}", position_anim.loop_mode);
    
    // Scale animation
    let scale_anim = UIAnimation {
        entity: 1,
        property: AnimatedProperty::Scale,
        from: AnimationValue::Vec2(Vec2::ONE),
        to: AnimationValue::Vec2(Vec2::new(1.5, 1.5)),
        duration: 0.5,
        easing: EasingFunction::EaseInOutBack,
        delay: 0.0,
        loop_mode: LoopMode::PingPong,
        on_complete: None,
        elapsed: 0.0,
        started: false,
        completed: false,
    };
    
    println!("\nScale Animation:");
    println!("  Property: {:?}", scale_anim.property);
    println!("  From: 1.0 -> To: 1.5");
    println!("  Duration: {}s", scale_anim.duration);
    println!("  Easing: {:?}", scale_anim.easing);
    println!("  Loop: {:?} (bounces back and forth)", scale_anim.loop_mode);
    
    // Color animation
    let color_anim = UIAnimation {
        entity: 1,
        property: AnimatedProperty::Color,
        from: AnimationValue::Color([1.0, 1.0, 1.0, 1.0]),
        to: AnimationValue::Color([1.0, 0.0, 0.0, 1.0]),
        duration: 2.0,
        easing: EasingFunction::Linear,
        delay: 0.5,
        loop_mode: LoopMode::Loop,
        on_complete: None,
        elapsed: 0.0,
        started: false,
        completed: false,
    };
    
    println!("\nColor Animation:");
    println!("  Property: {:?}", color_anim.property);
    println!("  From: White -> To: Red");
    println!("  Duration: {}s", color_anim.duration);
    println!("  Delay: {}s", color_anim.delay);
    println!("  Loop: {:?} (repeats forever)", color_anim.loop_mode);
    
    // Alpha fade animation
    let fade_anim = UIAnimation {
        entity: 1,
        property: AnimatedProperty::Alpha,
        from: AnimationValue::Float(1.0),
        to: AnimationValue::Float(0.0),
        duration: 0.3,
        easing: EasingFunction::EaseOutCubic,
        delay: 0.0,
        loop_mode: LoopMode::Once,
        on_complete: Some("on_fade_complete".to_string()),
        elapsed: 0.0,
        started: false,
        completed: false,
    };
    
    println!("\nFade Animation:");
    println!("  Property: {:?}", fade_anim.property);
    println!("  From: 1.0 (opaque) -> To: 0.0 (transparent)");
    println!("  Duration: {}s", fade_anim.duration);
    println!("  Easing: {:?}", fade_anim.easing);
    
    // Easing functions
    println!("\nAvailable Easing Functions:");
    let easings = vec![
        "Linear", "EaseInQuad", "EaseOutQuad", "EaseInOutQuad",
        "EaseInCubic", "EaseOutCubic", "EaseInOutCubic",
        "EaseInBack", "EaseOutBack", "EaseInOutBack",
        "EaseInElastic", "EaseOutElastic", "EaseInOutElastic",
        "EaseInBounce", "EaseOutBounce", "EaseInOutBounce",
    ];
    for (i, easing) in easings.iter().enumerate() {
        if i % 3 == 0 {
            print!("  ");
        }
        print!("{:<20}", easing);
        if (i + 1) % 3 == 0 {
            println!();
        }
    }
    println!();
    
    // Animation use cases
    println!("\nCommon Animation Use Cases:");
    println!("  Button Press: Scale down + color change");
    println!("  Dialog Open: Scale up + fade in");
    println!("  Notification: Slide in + pause + slide out");
    println!("  Loading Spinner: Continuous rotation");
    println!("  Health Bar: Smooth value interpolation");
    println!();
}

/// Demonstrate event system
fn demo_events() {
    println!("--- Event System ---\n");
    
    // Create event listeners
    let click_listener = UIEventListener {
        event_type: UIEventType::OnPointerClick,
        callback: "on_button_clicked".to_string(),
    };
    
    let hover_enter_listener = UIEventListener {
        event_type: UIEventType::OnPointerEnter,
        callback: "on_button_hover_enter".to_string(),
    };
    
    let hover_exit_listener = UIEventListener {
        event_type: UIEventType::OnPointerExit,
        callback: "on_button_hover_exit".to_string(),
    };
    
    let drag_listener = UIEventListener {
        event_type: UIEventType::OnDrag,
        callback: "on_element_dragged".to_string(),
    };
    
    println!("Event Listeners:");
    println!("  Click: {:?} -> {}", click_listener.event_type, click_listener.callback);
    println!("  Hover Enter: {:?} -> {}", hover_enter_listener.event_type, hover_enter_listener.callback);
    println!("  Hover Exit: {:?} -> {}", hover_exit_listener.event_type, hover_exit_listener.callback);
    println!("  Drag: {:?} -> {}", drag_listener.event_type, drag_listener.callback);
    
    // Event types
    println!("\nAvailable Event Types:");
    println!("  OnPointerEnter: Mouse enters element bounds");
    println!("  OnPointerExit: Mouse leaves element bounds");
    println!("  OnPointerDown: Mouse button pressed on element");
    println!("  OnPointerUp: Mouse button released on element");
    println!("  OnPointerClick: Complete click (down + up on same element)");
    println!("  OnBeginDrag: Drag starts on element");
    println!("  OnDrag: Element is being dragged");
    println!("  OnEndDrag: Drag ends");
    println!("  OnScroll: Scroll wheel used on element");
    
    // Event flow
    println!("\nEvent Flow:");
    println!("  1. User interacts with UI (mouse, touch, keyboard)");
    println!("  2. Raycast system finds UI element at interaction point");
    println!("  3. Event system generates appropriate event");
    println!("  4. Event delivered to topmost element (highest Z-order)");
    println!("  5. Registered callback function is invoked");
    println!("  6. Lua script handles the event");
    
    // Raycast blocking
    println!("\nRaycast Blocking:");
    println!("  Elements with blocks_raycasts=true prevent events from");
    println!("  reaching elements behind them (lower Z-order)");
    println!("  Use case: Modal dialogs, full-screen overlays");
    
    // Example event handling
    println!("\nExample Event Handling (Lua):");
    println!("  function on_button_clicked(entity)");
    println!("      print('Button clicked: ' .. entity)");
    println!("      -- Change button color");
    println!("      UI.set_color(entity, 1.0, 0.0, 0.0, 1.0)");
    println!("      -- Play sound");
    println!("      Audio.play('button_click')");
    println!("  end");
    println!();
}


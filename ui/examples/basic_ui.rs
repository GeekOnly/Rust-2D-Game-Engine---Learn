//! Basic UI Example
//!
//! This example demonstrates the core UI components:
//! - Canvas creation and configuration
//! - UIImage for displaying sprites
//! - UIText for displaying text
//! - UIButton for interactive elements
//! - RectTransform anchoring and positioning
//! - Basic layout with parent-child hierarchy

use ui::{
    Canvas, CanvasRenderMode, CanvasScaler, ScaleMode,
    RectTransform, UIElement, UIImage, UIText, UIButton,
    ButtonTransition, TextAlignment,
    Vec2, Vec4, Color,
};

type Entity = u64;

fn main() {
    println!("=== Basic UI Example ===\n");
    
    // Example 1: Create a Canvas
    demo_canvas_creation();
    
    // Example 2: Create UI elements with different anchor modes
    demo_anchoring();
    
    // Example 3: Create a simple button
    demo_button_creation();
    
    // Example 4: Create a UI hierarchy
    demo_ui_hierarchy();
    
    println!("\n=== Example Complete ===");
}

/// Demonstrate Canvas creation with different configurations
fn demo_canvas_creation() {
    println!("--- Canvas Creation ---\n");
    
    // Create a Screen Space Overlay canvas (most common for UI)
    let canvas_overlay = Canvas {
        render_mode: CanvasRenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        camera_entity: None,
        plane_distance: 100.0,
        scaler: CanvasScaler {
            mode: ScaleMode::ScaleWithScreenSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.5, // Blend between width and height
            reference_dpi: 96.0,
            min_scale: 0.5,
            max_scale: 2.0,
            scale_factor: 1.0,
        },
        blocks_raycasts: true,
        cached_screen_size: (1920, 1080),
        dirty: false,
    };
    
    println!("Created Screen Space Overlay Canvas:");
    println!("  Render Mode: {:?}", canvas_overlay.render_mode);
    println!("  Sort Order: {}", canvas_overlay.sort_order);
    println!("  Scale Mode: {:?}", canvas_overlay.scaler.mode);
    println!("  Reference Resolution: {}x{}", 
        canvas_overlay.scaler.reference_resolution.0,
        canvas_overlay.scaler.reference_resolution.1);
    
    // Create a World Space canvas (for in-game UI like health bars)
    let canvas_world = Canvas {
        render_mode: CanvasRenderMode::WorldSpace,
        sort_order: 0,
        camera_entity: Some(1), // Reference to camera entity
        plane_distance: 10.0,
        scaler: CanvasScaler {
            mode: ScaleMode::ConstantPixelSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.0,
            reference_dpi: 96.0,
            min_scale: 1.0,
            max_scale: 1.0,
            scale_factor: 1.0,
        },
        blocks_raycasts: false,
        cached_screen_size: (1920, 1080),
        dirty: false,
    };
    
    println!("\nCreated World Space Canvas:");
    println!("  Render Mode: {:?}", canvas_world.render_mode);
    println!("  Camera Entity: {:?}", canvas_world.camera_entity);
    println!("  Scale Mode: {:?}", canvas_world.scaler.mode);
    println!();
}

/// Demonstrate different anchoring modes
fn demo_anchoring() {
    println!("--- Anchoring Modes ---\n");
    
    // Fixed position anchoring (anchor min == anchor max)
    let fixed_anchor = RectTransform::anchored(
        Vec2::new(0.5, 0.5), // Center anchor
        Vec2::new(0.0, 100.0), // 100 pixels above center
        Vec2::new(200.0, 50.0), // 200x50 size
    );
    
    println!("Fixed Position (Center + Offset):");
    println!("  Anchor: ({}, {})", fixed_anchor.anchor_min.x, fixed_anchor.anchor_min.y);
    println!("  Position: ({}, {})", fixed_anchor.anchored_position.x, fixed_anchor.anchored_position.y);
    println!("  Size: {}x{}", fixed_anchor.size_delta.x, fixed_anchor.size_delta.y);
    println!("  Use case: Buttons, dialogs, fixed UI elements");
    
    // Stretched horizontally
    let stretched_h = RectTransform::stretched(
        Vec2::new(0.0, 0.5), // Left-center to right-center
        Vec2::new(1.0, 0.5),
        Vec4::new(20.0, 0.0, 20.0, 0.0), // 20px margins on left/right
    );
    
    println!("\nStretched Horizontally:");
    println!("  Anchor Min: ({}, {})", stretched_h.anchor_min.x, stretched_h.anchor_min.y);
    println!("  Anchor Max: ({}, {})", stretched_h.anchor_max.x, stretched_h.anchor_max.y);
    println!("  Margins: left={}, right={}", 20.0, 20.0);
    println!("  Use case: Top bars, bottom bars, horizontal panels");
    
    // Stretched vertically
    let stretched_v = RectTransform::stretched(
        Vec2::new(0.5, 0.0), // Center-bottom to center-top
        Vec2::new(0.5, 1.0),
        Vec4::new(0.0, 20.0, 0.0, 20.0), // 20px margins on top/bottom
    );
    
    println!("\nStretched Vertically:");
    println!("  Anchor Min: ({}, {})", stretched_v.anchor_min.x, stretched_v.anchor_min.y);
    println!("  Anchor Max: ({}, {})", stretched_v.anchor_max.x, stretched_v.anchor_max.y);
    println!("  Margins: bottom={}, top={}", 20.0, 20.0);
    println!("  Use case: Side panels, vertical menus");
    
    // Fully stretched (fills parent)
    let stretched_full = RectTransform::stretched(
        Vec2::ZERO, // Bottom-left to top-right
        Vec2::ONE,
        Vec4::new(10.0, 10.0, 10.0, 10.0), // 10px margins on all sides
    );
    
    println!("\nFully Stretched:");
    println!("  Anchor Min: ({}, {})", stretched_full.anchor_min.x, stretched_full.anchor_min.y);
    println!("  Anchor Max: ({}, {})", stretched_full.anchor_max.x, stretched_full.anchor_max.y);
    println!("  Margins: {}", 10.0);
    println!("  Use case: Background panels, full-screen overlays");
    
    // Corner anchoring examples
    println!("\nCorner Anchoring:");
    
    let top_left = RectTransform::anchored(
        Vec2::new(0.0, 1.0),
        Vec2::new(10.0, -10.0),
        Vec2::new(100.0, 40.0),
    );
    println!("  Top-Left: anchor=({}, {}), offset=({}, {})",
        top_left.anchor_min.x, top_left.anchor_min.y,
        top_left.anchored_position.x, top_left.anchored_position.y);
    
    let bottom_right = RectTransform::anchored(
        Vec2::new(1.0, 0.0),
        Vec2::new(-10.0, 10.0),
        Vec2::new(100.0, 40.0),
    );
    println!("  Bottom-Right: anchor=({}, {}), offset=({}, {})",
        bottom_right.anchor_min.x, bottom_right.anchor_min.y,
        bottom_right.anchored_position.x, bottom_right.anchored_position.y);
    
    println!();
}

/// Demonstrate button creation with different states
fn demo_button_creation() {
    println!("--- Button Creation ---\n");
    
    // Create a simple button
    let button_transform = RectTransform::anchored(
        Vec2::new(0.5, 0.5),
        Vec2::ZERO,
        Vec2::new(150.0, 50.0),
    );
    
    let button_element = UIElement {
        raycast_target: true,
        blocks_raycasts: true,
        z_order: 0,
        color: [1.0, 1.0, 1.0, 1.0],
        alpha: 1.0,
        interactable: true,
        ignore_layout: false,
        canvas_entity: None,
    };
    
    let button_image = UIImage::default();
    
    let button = UIButton {
        state: Default::default(),
        transition: ButtonTransition::ColorTint,
        normal_color: [1.0, 1.0, 1.0, 1.0],
        highlighted_color: [0.9, 0.9, 0.9, 1.0],
        pressed_color: [0.7, 0.7, 0.7, 1.0],
        disabled_color: [0.5, 0.5, 0.5, 0.5],
        fade_duration: 0.1,
        highlighted_sprite: None,
        pressed_sprite: None,
        disabled_sprite: None,
        normal_trigger: String::new(),
        highlighted_trigger: String::new(),
        pressed_trigger: String::new(),
        disabled_trigger: String::new(),
        on_click: Some("on_button_clicked".to_string()),
    };
    
    println!("Created Button:");
    println!("  Size: {}x{}", button_transform.size_delta.x, button_transform.size_delta.y);
    println!("  Transition: {:?}", button.transition);
    println!("  Normal Color: {:?}", button.normal_color);
    println!("  Highlighted Color: {:?}", button.highlighted_color);
    println!("  Pressed Color: {:?}", button.pressed_color);
    println!("  Callback: {:?}", button.on_click);
    println!("  Interactable: {}", button_element.interactable);
    
    // Create button text (child of button)
    let text_transform = RectTransform::stretched(
        Vec2::ZERO,
        Vec2::ONE,
        Vec4::new(5.0, 5.0, 5.0, 5.0), // 5px padding
    );
    
    let text = UIText {
        text: "Click Me!".to_string(),
        font: "default".to_string(),
        font_size: 16.0,
        color: [0.0, 0.0, 0.0, 1.0],
        alignment: TextAlignment::MiddleCenter,
        horizontal_overflow: ui::OverflowMode::Wrap,
        vertical_overflow: ui::OverflowMode::Truncate,
        rich_text: false,
        line_spacing: 1.0,
        best_fit: false,
        best_fit_min_size: 10.0,
        best_fit_max_size: 40.0,
    };
    
    println!("\nButton Text:");
    println!("  Text: '{}'", text.text);
    println!("  Font Size: {}", text.font_size);
    println!("  Alignment: {:?}", text.alignment);
    println!("  Color: {:?}", text.color);
    println!();
}

/// Demonstrate creating a UI hierarchy
fn demo_ui_hierarchy() {
    println!("--- UI Hierarchy ---\n");
    
    println!("Creating a simple menu hierarchy:");
    println!();
    
    // Root: Panel (background)
    println!("1. Menu Panel (400x300, centered)");
    let panel_transform = RectTransform::anchored(
        Vec2::new(0.5, 0.5),
        Vec2::ZERO,
        Vec2::new(400.0, 300.0),
    );
    println!("   - Anchor: Center");
    println!("   - Size: {}x{}", panel_transform.size_delta.x, panel_transform.size_delta.y);
    
    // Child 1: Title text
    println!("\n2. Title Text (child of panel)");
    let title_transform = RectTransform::anchored(
        Vec2::new(0.5, 1.0), // Top-center
        Vec2::new(0.0, -30.0), // 30px from top
        Vec2::new(380.0, 40.0),
    );
    let title_text = UIText {
        text: "Main Menu".to_string(),
        font: "default".to_string(),
        font_size: 24.0,
        color: [1.0, 1.0, 1.0, 1.0],
        alignment: TextAlignment::MiddleCenter,
        ..Default::default()
    };
    println!("   - Anchor: Top-Center");
    println!("   - Offset: {} from top", -title_transform.anchored_position.y);
    println!("   - Text: '{}'", title_text.text);
    println!("   - Font Size: {}", title_text.font_size);
    
    // Child 2: Button container with vertical layout
    println!("\n3. Button Container (child of panel)");
    let container_transform = RectTransform::anchored(
        Vec2::new(0.5, 0.5),
        Vec2::ZERO,
        Vec2::new(300.0, 200.0),
    );
    println!("   - Anchor: Center");
    println!("   - Size: {}x{}", container_transform.size_delta.x, container_transform.size_delta.y);
    println!("   - Layout: Vertical with spacing");
    
    // Child 3-5: Buttons (children of container)
    let button_names = vec!["Start Game", "Options", "Quit"];
    for (i, name) in button_names.iter().enumerate() {
        println!("\n{}. Button '{}' (child of container)", i + 4, name);
        let button_transform = RectTransform::anchored(
            Vec2::new(0.5, 0.5),
            Vec2::ZERO,
            Vec2::new(250.0, 50.0),
        );
        println!("   - Size: {}x{}", button_transform.size_delta.x, button_transform.size_delta.y);
        println!("   - Text: '{}'", name);
        println!("   - Positioned by vertical layout");
    }
    
    println!("\nHierarchy Structure:");
    println!("Canvas");
    println!("└── Menu Panel");
    println!("    ├── Title Text");
    println!("    └── Button Container (Vertical Layout)");
    println!("        ├── Start Game Button");
    println!("        │   └── Button Text");
    println!("        ├── Options Button");
    println!("        │   └── Button Text");
    println!("        └── Quit Button");
    println!("            └── Button Text");
    
    println!("\nKey Concepts Demonstrated:");
    println!("  ✓ Canvas as root container");
    println!("  ✓ Panel for background");
    println!("  ✓ Text for labels");
    println!("  ✓ Buttons for interaction");
    println!("  ✓ Parent-child relationships");
    println!("  ✓ Different anchor modes (center, top-center)");
    println!("  ✓ Layout groups for automatic positioning");
    println!();
}


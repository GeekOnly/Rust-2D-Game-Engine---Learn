//! Integration tests for text rendering

use ui::{
    Canvas, CanvasRenderMode, CanvasScaler, ScaleMode,
    RectTransform, UIElement, UIText, TextAlignment, OverflowMode,
    rendering::UIBatchBuilder,
    Rect, Vec2,
};

#[test]
fn test_text_rendering_basic() {
    let mut builder = UIBatchBuilder::new();
    
    // Create a canvas
    let canvas = Canvas {
        render_mode: CanvasRenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        camera_entity: None,
        plane_distance: 100.0,
        scaler: CanvasScaler {
            mode: ScaleMode::ConstantPixelSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.5,
            reference_dpi: 96.0,
            min_scale: 0.5,
            max_scale: 2.0,
            scale_factor: 1.0,
        },
        blocks_raycasts: true,
        cached_screen_size: (1920, 1080),
        dirty: false,
    };
    
    // Create a rect transform
    let rect = RectTransform {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(1.0, 1.0),
        pivot: Vec2::new(0.5, 0.5),
        anchored_position: Vec2::new(0.0, 0.0),
        size_delta: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        world_corners: [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 100.0),
            Vec2::new(200.0, 100.0),
            Vec2::new(200.0, 0.0),
        ],
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        },
        dirty: false,
    };
    
    // Create a UI element
    let element = UIElement {
        raycast_target: true,
        blocks_raycasts: true,
        z_order: 0,
        color: [1.0, 1.0, 1.0, 1.0],
        alpha: 1.0,
        interactable: true,
        ignore_layout: false,
        canvas_entity: None,
    };
    
    // Create a text component
    let text = UIText {
        text: "Hello World".to_string(),
        font: "default".to_string(),
        font_size: 16.0,
        color: [0.0, 0.0, 0.0, 1.0],
        alignment: TextAlignment::MiddleCenter,
        horizontal_overflow: OverflowMode::Overflow,
        vertical_overflow: OverflowMode::Overflow,
        rich_text: false,
        line_spacing: 1.0,
        best_fit: false,
        best_fit_min_size: 10.0,
        best_fit_max_size: 40.0,
    };
    
    // Collect the text element
    builder.collect_element(1, &canvas, &rect, &element, None, Some(&text), None);
    
    // Build batches
    builder.build_batches();
    
    // Verify batches were created
    let batches = builder.get_batches();
    assert_eq!(batches.len(), 1, "Should have one batch for text");
    
    // Verify the batch has vertices (one quad per character)
    let batch = &batches[0];
    assert!(!batch.vertices.is_empty(), "Batch should have vertices");
    assert!(!batch.indices.is_empty(), "Batch should have indices");
    
    // "Hello World" has 11 characters, each needs 4 vertices
    assert_eq!(batch.vertices.len(), 11 * 4, "Should have 4 vertices per character");
    
    // Each character needs 6 indices (2 triangles)
    assert_eq!(batch.indices.len(), 11 * 6, "Should have 6 indices per character");
}

#[test]
fn test_text_with_wrapping() {
    let mut builder = UIBatchBuilder::new();
    
    let canvas = Canvas {
        render_mode: CanvasRenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        camera_entity: None,
        plane_distance: 100.0,
        scaler: CanvasScaler {
            mode: ScaleMode::ConstantPixelSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.5,
            reference_dpi: 96.0,
            min_scale: 0.5,
            max_scale: 2.0,
            scale_factor: 1.0,
        },
        blocks_raycasts: true,
        cached_screen_size: (1920, 1080),
        dirty: false,
    };
    
    let rect = RectTransform {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(0.0, 0.0),
        pivot: Vec2::new(0.5, 0.5),
        anchored_position: Vec2::new(0.0, 0.0),
        size_delta: Vec2::new(100.0, 100.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        world_corners: [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 0.0),
        ],
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        dirty: false,
    };
    
    let element = UIElement {
        raycast_target: true,
        blocks_raycasts: true,
        z_order: 0,
        color: [1.0, 1.0, 1.0, 1.0],
        alpha: 1.0,
        interactable: true,
        ignore_layout: false,
        canvas_entity: None,
    };
    
    // Create text that will wrap
    let text = UIText {
        text: "This is a long text that should wrap to multiple lines".to_string(),
        font: "default".to_string(),
        font_size: 16.0,
        color: [0.0, 0.0, 0.0, 1.0],
        alignment: TextAlignment::TopLeft,
        horizontal_overflow: OverflowMode::Wrap,
        vertical_overflow: OverflowMode::Overflow,
        rich_text: false,
        line_spacing: 1.0,
        best_fit: false,
        best_fit_min_size: 10.0,
        best_fit_max_size: 40.0,
    };
    
    builder.collect_element(1, &canvas, &rect, &element, None, Some(&text), None);
    builder.build_batches();
    
    let batches = builder.get_batches();
    assert_eq!(batches.len(), 1, "Should have one batch");
    
    // Text should be wrapped, so we should have vertices
    let batch = &batches[0];
    assert!(!batch.vertices.is_empty(), "Wrapped text should have vertices");
}

#[test]
fn test_text_color_tint() {
    let mut builder = UIBatchBuilder::new();
    
    let canvas = Canvas {
        render_mode: CanvasRenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        camera_entity: None,
        plane_distance: 100.0,
        scaler: CanvasScaler {
            mode: ScaleMode::ConstantPixelSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.5,
            reference_dpi: 96.0,
            min_scale: 0.5,
            max_scale: 2.0,
            scale_factor: 1.0,
        },
        blocks_raycasts: true,
        cached_screen_size: (1920, 1080),
        dirty: false,
    };
    
    let rect = RectTransform {
        anchor_min: Vec2::new(0.0, 0.0),
        anchor_max: Vec2::new(0.0, 0.0),
        pivot: Vec2::new(0.5, 0.5),
        anchored_position: Vec2::new(0.0, 0.0),
        size_delta: Vec2::new(200.0, 100.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        world_corners: [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 100.0),
            Vec2::new(200.0, 100.0),
            Vec2::new(200.0, 0.0),
        ],
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        },
        dirty: false,
    };
    
    // Create element with alpha = 0.5
    let element = UIElement {
        raycast_target: true,
        blocks_raycasts: true,
        z_order: 0,
        color: [1.0, 1.0, 1.0, 1.0],
        alpha: 0.5,
        interactable: true,
        ignore_layout: false,
        canvas_entity: None,
    };
    
    let text = UIText {
        text: "A".to_string(),
        font: "default".to_string(),
        font_size: 16.0,
        color: [1.0, 0.0, 0.0, 1.0], // Red with full alpha
        alignment: TextAlignment::MiddleCenter,
        horizontal_overflow: OverflowMode::Overflow,
        vertical_overflow: OverflowMode::Overflow,
        rich_text: false,
        line_spacing: 1.0,
        best_fit: false,
        best_fit_min_size: 10.0,
        best_fit_max_size: 40.0,
    };
    
    builder.collect_element(1, &canvas, &rect, &element, None, Some(&text), None);
    builder.build_batches();
    
    let batches = builder.get_batches();
    let batch = &batches[0];
    
    // Check that the alpha was applied (text color alpha * element alpha)
    // Text color is [1.0, 0.0, 0.0, 1.0], element alpha is 0.5
    // Final alpha should be 1.0 * 0.5 = 0.5
    assert_eq!(batch.vertices[0].color[3], 0.5, "Alpha should be multiplied");
    
    // RGB should remain the same (text color)
    assert_eq!(batch.vertices[0].color[0], 1.0, "Red channel should be preserved");
    assert_eq!(batch.vertices[0].color[1], 0.0, "Green channel should be preserved");
    assert_eq!(batch.vertices[0].color[2], 0.0, "Blue channel should be preserved");
}

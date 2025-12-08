//! Prefab instantiation demo
//!
//! This example demonstrates how to create and instantiate UI prefabs
//! with and without parameters.

use ui::{
    UIPrefab, UIPrefabElement, PrefabInstantiator, PrefabParameters,
    RectTransform, UIElement, UIImage, UIText, UIButton, UIPanel,
    Vec2,
};

fn main() {
    println!("=== UI Prefab Instantiation Demo ===\n");
    
    // Create a simple button prefab
    let button_prefab = create_button_prefab();
    
    // Create an instantiator
    let mut instantiator = PrefabInstantiator::new();
    
    // Example 1: Instantiate without parameters
    println!("Example 1: Basic instantiation");
    let result1 = instantiator.instantiate(&button_prefab);
    println!("  Created button with root entity: {}", result1.root_entity);
    println!("  Named entities: {:?}", result1.named_entities);
    
    // Verify components
    if let Some(text) = instantiator.texts.get(&result1.named_entities["ButtonText"]) {
        println!("  Button text: '{}'", text.text);
    }
    println!();
    
    // Example 2: Instantiate with parameters
    println!("Example 2: Instantiation with parameters");
    let mut params = PrefabParameters::new();
    params
        .set_text("ButtonText", "Click Me!".to_string())
        .set_color("ButtonBackground", [0.2, 0.6, 1.0, 1.0])
        .set_size("ButtonBackground", Vec2::new(150.0, 50.0));
    
    let result2 = instantiator.instantiate_with_params(&button_prefab, &params);
    println!("  Created button with root entity: {}", result2.root_entity);
    
    // Verify parameters were applied
    if let Some(text) = instantiator.texts.get(&result2.named_entities["ButtonText"]) {
        println!("  Button text: '{}'", text.text);
    }
    if let Some(element) = instantiator.ui_elements.get(&result2.named_entities["ButtonBackground"]) {
        println!("  Button color: {:?}", element.color);
    }
    if let Some(transform) = instantiator.rect_transforms.get(&result2.named_entities["ButtonBackground"]) {
        println!("  Button size: {:?}", transform.get_size());
    }
    println!();
    
    // Example 3: Create multiple instances
    println!("Example 3: Multiple instances");
    let result3 = instantiator.instantiate(&button_prefab);
    let result4 = instantiator.instantiate(&button_prefab);
    println!("  Created button 3 with entity: {}", result3.root_entity);
    println!("  Created button 4 with entity: {}", result4.root_entity);
    println!("  Total entities created: {}", instantiator.rect_transforms.len());
    println!();
    
    // Example 4: Destroy an entity
    println!("Example 4: Destroying entities");
    println!("  Entities before destroy: {}", instantiator.rect_transforms.len());
    instantiator.destroy_entity(result2.root_entity);
    println!("  Entities after destroy: {}", instantiator.rect_transforms.len());
    println!();
    
    // Example 5: Complex prefab with hierarchy
    println!("Example 5: Complex hierarchical prefab");
    let dialog_prefab = create_dialog_prefab();
    let result5 = instantiator.instantiate(&dialog_prefab);
    println!("  Created dialog with root entity: {}", result5.root_entity);
    println!("  Named entities in dialog:");
    for (name, entity) in &result5.named_entities {
        println!("    - {}: {}", name, entity);
    }
    
    println!("\n=== Demo Complete ===");
}

/// Create a simple button prefab
fn create_button_prefab() -> UIPrefab {
    UIPrefab {
        name: "Button".to_string(),
        root: UIPrefabElement {
            name: "ButtonBackground".to_string(),
            rect_transform: RectTransform::anchored(
                Vec2::new(0.5, 0.5),
                Vec2::ZERO,
                Vec2::new(100.0, 40.0),
            ),
            ui_element: UIElement {
                raycast_target: true,
                blocks_raycasts: true,
                interactable: true,
                ..Default::default()
            },
            image: Some(UIImage::default()),
            button: Some(UIButton::default()),
            text: None,
            panel: None,
            slider: None,
            toggle: None,
            dropdown: None,
            input_field: None,
            scroll_view: None,
            mask: None,
            horizontal_layout: None,
            vertical_layout: None,
            grid_layout: None,
            children: vec![
                UIPrefabElement {
                    name: "ButtonText".to_string(),
                    rect_transform: RectTransform::stretched(
                        Vec2::ZERO,
                        Vec2::ONE,
                        [5.0, 5.0, 5.0, 5.0].into(),
                    ),
                    ui_element: UIElement {
                        raycast_target: false,
                        ..Default::default()
                    },
                    text: Some(UIText {
                        text: "Button".to_string(),
                        ..Default::default()
                    }),
                    image: None,
                    button: None,
                    panel: None,
                    slider: None,
                    toggle: None,
                    dropdown: None,
                    input_field: None,
                    scroll_view: None,
                    mask: None,
                    horizontal_layout: None,
                    vertical_layout: None,
                    grid_layout: None,
                    children: vec![],
                },
            ],
        },
    }
}

/// Create a dialog prefab with multiple elements
fn create_dialog_prefab() -> UIPrefab {
    UIPrefab {
        name: "Dialog".to_string(),
        root: UIPrefabElement {
            name: "DialogPanel".to_string(),
            rect_transform: RectTransform::anchored(
                Vec2::new(0.5, 0.5),
                Vec2::ZERO,
                Vec2::new(400.0, 300.0),
            ),
            ui_element: UIElement {
                raycast_target: true,
                blocks_raycasts: true,
                ..Default::default()
            },
            panel: Some(UIPanel::default()),
            image: None,
            text: None,
            button: None,
            slider: None,
            toggle: None,
            dropdown: None,
            input_field: None,
            scroll_view: None,
            mask: None,
            horizontal_layout: None,
            vertical_layout: None,
            grid_layout: None,
            children: vec![
                UIPrefabElement {
                    name: "DialogTitle".to_string(),
                    rect_transform: RectTransform::anchored(
                        Vec2::new(0.5, 1.0),
                        Vec2::new(0.0, -20.0),
                        Vec2::new(380.0, 40.0),
                    ),
                    ui_element: UIElement::default(),
                    text: Some(UIText {
                        text: "Dialog Title".to_string(),
                        ..Default::default()
                    }),
                    image: None,
                    button: None,
                    panel: None,
                    slider: None,
                    toggle: None,
                    dropdown: None,
                    input_field: None,
                    scroll_view: None,
                    mask: None,
                    horizontal_layout: None,
                    vertical_layout: None,
                    grid_layout: None,
                    children: vec![],
                },
                UIPrefabElement {
                    name: "DialogContent".to_string(),
                    rect_transform: RectTransform::anchored(
                        Vec2::new(0.5, 0.5),
                        Vec2::ZERO,
                        Vec2::new(380.0, 200.0),
                    ),
                    ui_element: UIElement::default(),
                    text: Some(UIText {
                        text: "Dialog content goes here...".to_string(),
                        ..Default::default()
                    }),
                    image: None,
                    button: None,
                    panel: None,
                    slider: None,
                    toggle: None,
                    dropdown: None,
                    input_field: None,
                    scroll_view: None,
                    mask: None,
                    horizontal_layout: None,
                    vertical_layout: None,
                    grid_layout: None,
                    children: vec![],
                },
                UIPrefabElement {
                    name: "OkButton".to_string(),
                    rect_transform: RectTransform::anchored(
                        Vec2::new(0.5, 0.0),
                        Vec2::new(0.0, 20.0),
                        Vec2::new(100.0, 40.0),
                    ),
                    ui_element: UIElement {
                        raycast_target: true,
                        blocks_raycasts: true,
                        interactable: true,
                        ..Default::default()
                    },
                    button: Some(UIButton::default()),
                    image: Some(UIImage::default()),
                    text: None,
                    panel: None,
                    slider: None,
                    toggle: None,
                    dropdown: None,
                    input_field: None,
                    scroll_view: None,
                    mask: None,
                    horizontal_layout: None,
                    vertical_layout: None,
                    grid_layout: None,
                    children: vec![
                        UIPrefabElement {
                            name: "OkButtonText".to_string(),
                            rect_transform: RectTransform::stretched(
                                Vec2::ZERO,
                                Vec2::ONE,
                                [5.0, 5.0, 5.0, 5.0].into(),
                            ),
                            ui_element: UIElement {
                                raycast_target: false,
                                ..Default::default()
                            },
                            text: Some(UIText {
                                text: "OK".to_string(),
                                ..Default::default()
                            }),
                            image: None,
                            button: None,
                            panel: None,
                            slider: None,
                            toggle: None,
                            dropdown: None,
                            input_field: None,
                            scroll_view: None,
                            mask: None,
                            horizontal_layout: None,
                            vertical_layout: None,
                            grid_layout: None,
                            children: vec![],
                        },
                    ],
                },
            ],
        },
    }
}

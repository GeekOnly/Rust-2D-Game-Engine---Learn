//! Migration tests for sample HUD files
//! 
//! Tests the migration tool on various sample HUD files to ensure
//! correct conversion to UIPrefab format.

use ui::hud_converter::{HudAsset, HudToUIPrefabConverter};
use ui::prefab::UIPrefab;

/// Helper function to load and convert a HUD file
fn load_and_convert_hud(hud_content: &str) -> Result<UIPrefab, String> {
    let hud_asset: HudAsset = serde_json::from_str(hud_content)
        .map_err(|e| format!("Failed to parse HUD: {}", e))?;
    
    let prefab = HudToUIPrefabConverter::convert(&hud_asset);
    Ok(prefab)
}

/// Helper function to verify prefab structure
fn verify_prefab_structure(prefab: &UIPrefab, expected_name: &str, expected_element_count: usize) {
    assert_eq!(prefab.name, expected_name, "Prefab name mismatch");
    
    // Count all elements (root + children)
    let total_elements = 1 + prefab.root.children.len();
    assert_eq!(
        total_elements, expected_element_count,
        "Expected {} elements, found {}",
        expected_element_count, total_elements
    );
}

/// Helper function to verify element properties
fn verify_element_exists(prefab: &UIPrefab, element_id: &str) -> bool {
    if prefab.root.name == element_id {
        return true;
    }
    
    prefab.root.children.iter().any(|child| child.name == element_id)
}

#[test]
fn test_simple_hud_migration() {
    // Test HUD with basic text elements
    let hud_content = r#"{
        "name": "Test HUD",
        "elements": [
            {
                "id": "test_text",
                "element_type": {
                    "type": "Text",
                    "text": "HUD IS WORKING!",
                    "font_size": 32.0,
                    "color": [1.0, 0.0, 0.0, 1.0]
                },
                "anchor": "Center",
                "offset": [0.0, 0.0],
                "size": [300.0, 50.0],
                "visible": true
            },
            {
                "id": "top_left_test",
                "element_type": {
                    "type": "Text",
                    "text": "TOP LEFT",
                    "font_size": 24.0,
                    "color": [0.0, 1.0, 0.0, 1.0]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 20.0],
                "size": [200.0, 40.0],
                "visible": true
            },
            {
                "id": "bottom_right_test",
                "element_type": {
                    "type": "Text",
                    "text": "BOTTOM RIGHT",
                    "font_size": 24.0,
                    "color": [0.0, 0.0, 1.0, 1.0]
                },
                "anchor": "BottomRight",
                "offset": [-220.0, -60.0],
                "size": [200.0, 40.0],
                "visible": true
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content).expect("Failed to convert simple HUD");
    
    // Verify structure: 1 root + 3 children = 4 elements
    verify_prefab_structure(&prefab, "Test HUD", 4);
    
    // Verify all elements exist
    assert!(verify_element_exists(&prefab, "test_text"), "test_text element not found");
    assert!(verify_element_exists(&prefab, "top_left_test"), "top_left_test element not found");
    assert!(verify_element_exists(&prefab, "bottom_right_test"), "bottom_right_test element not found");
    
    // Verify text components
    let test_text = prefab.root.children.iter()
        .find(|e| e.name == "test_text")
        .expect("test_text not found");
    
    assert!(test_text.text.is_some(), "Text component missing");
    let text = test_text.text.as_ref().unwrap();
    assert_eq!(text.text, "HUD IS WORKING!");
    assert_eq!(text.font_size, 32.0);
    assert_eq!(text.color, [1.0, 0.0, 0.0, 1.0]);
    
    // Verify anchor conversion for center element
    assert_eq!(test_text.rect_transform.anchor_min, [0.5, 0.5].into());
    assert_eq!(test_text.rect_transform.anchor_max, [0.5, 0.5].into());
    
    // Verify anchor conversion for top-left element
    let top_left = prefab.root.children.iter()
        .find(|e| e.name == "top_left_test")
        .expect("top_left_test not found");
    
    assert_eq!(top_left.rect_transform.anchor_min, [0.0, 1.0].into());
    assert_eq!(top_left.rect_transform.anchor_max, [0.0, 1.0].into());
    
    // Verify anchor conversion for bottom-right element
    let bottom_right = prefab.root.children.iter()
        .find(|e| e.name == "bottom_right_test")
        .expect("bottom_right_test not found");
    
    assert_eq!(bottom_right.rect_transform.anchor_min, [1.0, 0.0].into());
    assert_eq!(bottom_right.rect_transform.anchor_max, [1.0, 0.0].into());
}

#[test]
fn test_complex_hud_migration() {
    // Complex HUD with multiple element types
    let hud_content = r#"{
        "name": "Main HUD",
        "elements": [
            {
                "id": "player_health",
                "element_type": {
                    "type": "HealthBar",
                    "binding": "player.health",
                    "color": [1.0, 0.2, 0.2, 1.0],
                    "background_color": [0.2, 0.2, 0.2, 0.8]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 20.0],
                "size": [200.0, 30.0],
                "visible": true
            },
            {
                "id": "player_mana",
                "element_type": {
                    "type": "ProgressBar",
                    "binding": "player.mana",
                    "color": [0.2, 0.5, 1.0, 1.0],
                    "background_color": [0.2, 0.2, 0.2, 0.8]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 60.0],
                "size": [200.0, 20.0],
                "visible": true
            },
            {
                "id": "score",
                "element_type": {
                    "type": "DynamicText",
                    "format": "Score: {score}",
                    "font_size": 24.0,
                    "color": [1.0, 1.0, 1.0, 1.0]
                },
                "anchor": "TopCenter",
                "offset": [-50.0, 20.0],
                "size": [100.0, 40.0],
                "visible": true
            },
            {
                "id": "fps",
                "element_type": {
                    "type": "DynamicText",
                    "format": "FPS: {fps}",
                    "font_size": 16.0,
                    "color": [0.8, 0.8, 0.8, 1.0]
                },
                "anchor": "TopRight",
                "offset": [-100.0, 10.0],
                "size": [90.0, 30.0],
                "visible": true
            },
            {
                "id": "minimap",
                "element_type": {
                    "type": "Minimap",
                    "zoom": 2.0,
                    "background_color": [0.1, 0.1, 0.1, 0.9]
                },
                "anchor": "BottomRight",
                "offset": [-170.0, -170.0],
                "size": [150.0, 150.0],
                "visible": true
            },
            {
                "id": "interaction_hint",
                "element_type": {
                    "type": "Text",
                    "text": "Press E to interact",
                    "font_size": 18.0,
                    "color": [1.0, 1.0, 1.0, 0.9]
                },
                "anchor": "BottomCenter",
                "offset": [-80.0, -100.0],
                "size": [160.0, 30.0],
                "visible": false
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content).expect("Failed to convert complex HUD");
    
    // Note: HealthBar and ProgressBar each have 2 children (background + fill)
    // So total is: 1 root + 6 top-level children + 4 nested children = 11 elements
    // But verify_prefab_structure only counts root + direct children
    assert_eq!(prefab.name, "Main HUD");
    assert_eq!(prefab.root.children.len(), 6, "Should have 6 top-level children");
    
    // Verify all elements exist
    assert!(verify_element_exists(&prefab, "player_health"), "player_health not found");
    assert!(verify_element_exists(&prefab, "player_mana"), "player_mana not found");
    assert!(verify_element_exists(&prefab, "score"), "score not found");
    assert!(verify_element_exists(&prefab, "fps"), "fps not found");
    assert!(verify_element_exists(&prefab, "minimap"), "minimap not found");
    assert!(verify_element_exists(&prefab, "interaction_hint"), "interaction_hint not found");
    
    // Verify HealthBar conversion (should create background + fill images)
    let health_bar = prefab.root.children.iter()
        .find(|e| e.name == "player_health")
        .expect("player_health not found");
    
    // Health bar should have 2 children: background and fill
    assert_eq!(health_bar.children.len(), 2, "HealthBar should have 2 children");
    
    let background = &health_bar.children[0];
    assert_eq!(background.name, "player_health_background");
    assert!(background.image.is_some(), "Background should have image component");
    
    let fill = &health_bar.children[1];
    assert_eq!(fill.name, "player_health_fill");
    assert!(fill.image.is_some(), "Fill should have image component");
    
    // Verify ProgressBar conversion
    let mana_bar = prefab.root.children.iter()
        .find(|e| e.name == "player_mana")
        .expect("player_mana not found");
    
    assert_eq!(mana_bar.children.len(), 2, "ProgressBar should have 2 children");
    
    // Verify DynamicText conversion (should have note about Lua binding)
    let score_text = prefab.root.children.iter()
        .find(|e| e.name == "score")
        .expect("score not found");
    
    assert!(score_text.text.is_some(), "DynamicText should have text component");
    let text = score_text.text.as_ref().unwrap();
    assert!(text.text.contains("Score:"), "DynamicText should preserve format string");
    
    // Verify visibility
    let hint = prefab.root.children.iter()
        .find(|e| e.name == "interaction_hint")
        .expect("interaction_hint not found");
    
    // Note: visibility is handled by UIElement, not in the test directly
    // but we can verify the element exists
    assert!(hint.text.is_some());
}

#[test]
fn test_nested_container_migration() {
    // HUD with nested containers
    let hud_content = r#"{
        "name": "Nested HUD",
        "elements": [
            {
                "id": "main_panel",
                "element_type": {
                    "type": "Container"
                },
                "anchor": "Center",
                "offset": [0.0, 0.0],
                "size": [400.0, 300.0],
                "visible": true,
                "children": [
                    {
                        "id": "header",
                        "element_type": {
                            "type": "Text",
                            "text": "Header",
                            "font_size": 24.0,
                            "color": [1.0, 1.0, 1.0, 1.0]
                        },
                        "anchor": "TopCenter",
                        "offset": [0.0, -10.0],
                        "size": [200.0, 40.0],
                        "visible": true
                    },
                    {
                        "id": "content_panel",
                        "element_type": {
                            "type": "Container"
                        },
                        "anchor": "Center",
                        "offset": [0.0, 0.0],
                        "size": [350.0, 200.0],
                        "visible": true,
                        "children": [
                            {
                                "id": "item1",
                                "element_type": {
                                    "type": "Text",
                                    "text": "Item 1",
                                    "font_size": 16.0,
                                    "color": [1.0, 1.0, 1.0, 1.0]
                                },
                                "anchor": "TopLeft",
                                "offset": [10.0, -10.0],
                                "size": [150.0, 30.0],
                                "visible": true
                            },
                            {
                                "id": "item2",
                                "element_type": {
                                    "type": "Text",
                                    "text": "Item 2",
                                    "font_size": 16.0,
                                    "color": [1.0, 1.0, 1.0, 1.0]
                                },
                                "anchor": "TopLeft",
                                "offset": [10.0, -50.0],
                                "size": [150.0, 30.0],
                                "visible": true
                            }
                        ]
                    },
                    {
                        "id": "footer",
                        "element_type": {
                            "type": "Text",
                            "text": "Footer",
                            "font_size": 18.0,
                            "color": [0.8, 0.8, 0.8, 1.0]
                        },
                        "anchor": "BottomCenter",
                        "offset": [0.0, 10.0],
                        "size": [200.0, 30.0],
                        "visible": true
                    }
                ]
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content).expect("Failed to convert nested HUD");
    
    // Verify structure
    assert_eq!(prefab.name, "Nested HUD");
    
    // Root should have 1 child (main_panel)
    assert_eq!(prefab.root.children.len(), 1, "Root should have 1 child");
    
    let main_panel = &prefab.root.children[0];
    assert_eq!(main_panel.name, "main_panel");
    
    // main_panel should have 3 children: header, content_panel, footer
    assert_eq!(main_panel.children.len(), 3, "main_panel should have 3 children");
    
    let header = &main_panel.children[0];
    assert_eq!(header.name, "header");
    assert!(header.text.is_some());
    
    let content_panel = &main_panel.children[1];
    assert_eq!(content_panel.name, "content_panel");
    
    // content_panel should have 2 children: item1, item2
    assert_eq!(content_panel.children.len(), 2, "content_panel should have 2 children");
    
    let item1 = &content_panel.children[0];
    assert_eq!(item1.name, "item1");
    assert!(item1.text.is_some());
    assert_eq!(item1.text.as_ref().unwrap().text, "Item 1");
    
    let item2 = &content_panel.children[1];
    assert_eq!(item2.name, "item2");
    assert!(item2.text.is_some());
    assert_eq!(item2.text.as_ref().unwrap().text, "Item 2");
    
    let footer = &main_panel.children[2];
    assert_eq!(footer.name, "footer");
    assert!(footer.text.is_some());
}

#[test]
fn test_visual_output_verification() {
    // Test that converted prefabs can be serialized and deserialized
    let hud_content = r#"{
        "name": "Visual Test HUD",
        "elements": [
            {
                "id": "test_element",
                "element_type": {
                    "type": "Text",
                    "text": "Test",
                    "font_size": 20.0,
                    "color": [1.0, 1.0, 1.0, 1.0]
                },
                "anchor": "Center",
                "offset": [0.0, 0.0],
                "size": [100.0, 30.0],
                "visible": true
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content).expect("Failed to convert HUD");
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&prefab)
        .expect("Failed to serialize prefab");
    
    // Verify JSON is valid
    assert!(!json.is_empty());
    assert!(json.contains("Visual Test HUD"));
    assert!(json.contains("test_element"));
    
    // Deserialize back
    let deserialized: UIPrefab = serde_json::from_str(&json)
        .expect("Failed to deserialize prefab");
    
    // Verify structure is preserved
    assert_eq!(deserialized.name, prefab.name);
    assert_eq!(deserialized.root.children.len(), prefab.root.children.len());
    
    let original_element = &prefab.root.children[0];
    let deserialized_element = &deserialized.root.children[0];
    
    assert_eq!(deserialized_element.name, original_element.name);
    assert_eq!(
        deserialized_element.rect_transform.anchor_min,
        original_element.rect_transform.anchor_min
    );
    assert_eq!(
        deserialized_element.rect_transform.anchor_max,
        original_element.rect_transform.anchor_max
    );
    
    // Verify text component
    assert!(deserialized_element.text.is_some());
    let original_text = original_element.text.as_ref().unwrap();
    let deserialized_text = deserialized_element.text.as_ref().unwrap();
    
    assert_eq!(deserialized_text.text, original_text.text);
    assert_eq!(deserialized_text.font_size, original_text.font_size);
    assert_eq!(deserialized_text.color, original_text.color);
}

#[test]
fn test_all_anchor_positions() {
    // Test all 9 anchor positions
    let anchors = vec![
        ("TopLeft", [0.0, 1.0]),
        ("TopCenter", [0.5, 1.0]),
        ("TopRight", [1.0, 1.0]),
        ("CenterLeft", [0.0, 0.5]),
        ("Center", [0.5, 0.5]),
        ("CenterRight", [1.0, 0.5]),
        ("BottomLeft", [0.0, 0.0]),
        ("BottomCenter", [0.5, 0.0]),
        ("BottomRight", [1.0, 0.0]),
    ];
    
    for (anchor_name, expected_anchor) in anchors {
        let hud_content = format!(r#"{{
            "name": "Anchor Test",
            "elements": [
                {{
                    "id": "test",
                    "element_type": {{
                        "type": "Text",
                        "text": "Test",
                        "font_size": 16.0,
                        "color": [1.0, 1.0, 1.0, 1.0]
                    }},
                    "anchor": "{}",
                    "offset": [0.0, 0.0],
                    "size": [100.0, 30.0],
                    "visible": true
                }}
            ]
        }}"#, anchor_name);
        
        let prefab = load_and_convert_hud(&hud_content)
            .expect(&format!("Failed to convert HUD with {} anchor", anchor_name));
        
        let element = &prefab.root.children[0];
        assert_eq!(
            element.rect_transform.anchor_min,
            expected_anchor.into(),
            "Anchor min mismatch for {}",
            anchor_name
        );
        assert_eq!(
            element.rect_transform.anchor_max,
            expected_anchor.into(),
            "Anchor max mismatch for {}",
            anchor_name
        );
    }
}

#[test]
fn test_celeste_demo_hud_migration() {
    // Test the actual Celeste Demo HUD
    let hud_content = r#"{
        "name": "Celeste Demo HUD",
        "elements": [
            {
                "id": "player_health",
                "element_type": {
                    "type": "HealthBar",
                    "binding": "player.health",
                    "color": [0.2, 1.0, 0.3, 1.0],
                    "background_color": [0.15, 0.15, 0.15, 0.9]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 20.0],
                "size": [180.0, 24.0],
                "visible": true
            },
            {
                "id": "stamina_bar",
                "element_type": {
                    "type": "ProgressBar",
                    "binding": "player.stamina",
                    "color": [1.0, 0.8, 0.2, 1.0],
                    "background_color": [0.15, 0.15, 0.15, 0.9]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 50.0],
                "size": [180.0, 16.0],
                "visible": true
            },
            {
                "id": "dash_indicator",
                "element_type": {
                    "type": "DynamicText",
                    "format": "Dash: {dash_count}",
                    "font_size": 18.0,
                    "color": [0.3, 0.8, 1.0, 1.0]
                },
                "anchor": "TopLeft",
                "offset": [20.0, 75.0],
                "size": [120.0, 30.0],
                "visible": true
            },
            {
                "id": "fps_counter",
                "element_type": {
                    "type": "DynamicText",
                    "format": "FPS: {fps}",
                    "font_size": 16.0,
                    "color": [0.5, 1.0, 0.5, 1.0]
                },
                "anchor": "TopRight",
                "offset": [-100.0, 60.0],
                "size": [90.0, 30.0],
                "visible": true
            },
            {
                "id": "controls_hint",
                "element_type": {
                    "type": "Text",
                    "text": "WASD: Move | Space: Jump | Shift: Dash",
                    "font_size": 14.0,
                    "color": [0.8, 0.8, 0.8, 0.7]
                },
                "anchor": "BottomCenter",
                "offset": [-180.0, -15.0],
                "size": [360.0, 25.0],
                "visible": true
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content)
        .expect("Failed to convert Celeste Demo HUD");
    
    // Verify structure (5 top-level children, HealthBar and ProgressBar have nested children)
    assert_eq!(prefab.name, "Celeste Demo HUD");
    assert_eq!(prefab.root.children.len(), 5, "Should have 5 top-level children");
    
    // Verify specific elements
    assert!(verify_element_exists(&prefab, "player_health"));
    assert!(verify_element_exists(&prefab, "stamina_bar"));
    assert!(verify_element_exists(&prefab, "dash_indicator"));
    assert!(verify_element_exists(&prefab, "fps_counter"));
    assert!(verify_element_exists(&prefab, "controls_hint"));
    
    // Verify health bar has proper structure
    let health_bar = prefab.root.children.iter()
        .find(|e| e.name == "player_health")
        .expect("player_health not found");
    
    assert_eq!(health_bar.children.len(), 2, "HealthBar should have background and fill");
    
    // Verify controls hint text
    let controls = prefab.root.children.iter()
        .find(|e| e.name == "controls_hint")
        .expect("controls_hint not found");
    
    assert!(controls.text.is_some());
    let text = controls.text.as_ref().unwrap();
    assert!(text.text.contains("WASD"));
    assert!(text.text.contains("Jump"));
    assert!(text.text.contains("Dash"));
}

#[test]
fn test_image_element_migration() {
    // Test Image element conversion
    let hud_content = r#"{
        "name": "Image Test",
        "elements": [
            {
                "id": "logo",
                "element_type": {
                    "type": "Image",
                    "texture": "logo.png"
                },
                "anchor": "TopCenter",
                "offset": [0.0, -20.0],
                "size": [128.0, 128.0],
                "visible": true
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content)
        .expect("Failed to convert Image HUD");
    
    let logo = &prefab.root.children[0];
    assert_eq!(logo.name, "logo");
    assert!(logo.image.is_some(), "Image component should exist");
    
    let image = logo.image.as_ref().unwrap();
    assert_eq!(image.sprite, Some("logo.png".to_string()));
}

#[test]
fn test_size_and_offset_preservation() {
    // Test that size and offset are correctly preserved
    let hud_content = r#"{
        "name": "Size Test",
        "elements": [
            {
                "id": "element",
                "element_type": {
                    "type": "Text",
                    "text": "Test",
                    "font_size": 16.0,
                    "color": [1.0, 1.0, 1.0, 1.0]
                },
                "anchor": "TopLeft",
                "offset": [50.0, -30.0],
                "size": [250.0, 75.0],
                "visible": true
            }
        ]
    }"#;
    
    let prefab = load_and_convert_hud(hud_content)
        .expect("Failed to convert HUD");
    
    let element = &prefab.root.children[0];
    
    // Verify size is preserved
    let size = element.rect_transform.size_delta;
    assert_eq!(size.x, 250.0, "Width should be preserved");
    assert_eq!(size.y, 75.0, "Height should be preserved");
    
    // Verify offset is preserved
    let pos = element.rect_transform.anchored_position;
    assert_eq!(pos.x, 50.0, "X offset should be preserved");
    assert_eq!(pos.y, -30.0, "Y offset should be preserved");
}

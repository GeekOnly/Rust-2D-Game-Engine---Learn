//! HUD to UIPrefab converter
//! 
//! Converts legacy HUD assets to the new UI prefab system

use crate::{
    UIPrefab, UIPrefabElement, RectTransform, UIElement, UIImage, UIText, UIPanel,
    Vec2, Color, ImageType, FillMethod, TextAlignment,
};
use glam::Vec4;

/// Converter for transforming HUD assets to UI prefabs
pub struct HudToUIPrefabConverter;

impl HudToUIPrefabConverter {
    /// Convert a HudAsset to a UIPrefab
    /// 
    /// # Arguments
    /// * `hud` - The HUD asset to convert
    /// 
    /// # Returns
    /// A UIPrefab with the converted UI hierarchy
    pub fn convert(hud: &HudAsset) -> UIPrefab {
        // Create root container element
        let mut root = UIPrefabElement {
            name: hud.name.clone(),
            rect_transform: RectTransform::stretched(
                Vec2::ZERO,
                Vec2::ONE,
                Vec4::ZERO,
            ),
            ui_element: UIElement::default(),
            image: None,
            text: None,
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
            children: Vec::new(),
        };
        
        // Convert all HUD elements to UI prefab elements
        for hud_element in &hud.elements {
            let prefab_element = Self::convert_element(hud_element);
            root.children.push(prefab_element);
        }
        
        UIPrefab {
            name: hud.name.clone(),
            root,
        }
    }
    
    /// Convert a single HudElement to a UIPrefabElement
    fn convert_element(hud_element: &HudElement) -> UIPrefabElement {
        // Convert anchor to RectTransform
        let rect_transform = Self::convert_anchor(
            &hud_element.anchor,
            hud_element.offset,
            hud_element.size,
        );
        
        // Create base UI element
        let mut ui_element = UIElement::default();
        ui_element.raycast_target = hud_element.visible;
        ui_element.interactable = hud_element.visible;
        
        // Convert wrapper to internal type
        let element_type: HudElementType = hud_element.element_type.clone().into();
        
        // Handle special cases for HealthBar and ProgressBar
        if let HudElementType::HealthBar { binding, color, background_color } = &element_type {
            return Self::convert_health_bar(hud_element, binding, *color, *background_color);
        }
        
        if let HudElementType::ProgressBar { binding, color, background_color } = &element_type {
            return Self::convert_progress_bar(hud_element, binding, *color, *background_color);
        }
        
        // Convert element type to components
        let (image, text, panel, type_children, notes) = Self::convert_element_type(&element_type);
        
        // Create prefab element
        let mut prefab_element = UIPrefabElement {
            name: hud_element.id.clone(),
            rect_transform,
            ui_element,
            image,
            text,
            button: None,
            panel,
            slider: None,
            toggle: None,
            dropdown: None,
            input_field: None,
            scroll_view: None,
            mask: None,
            horizontal_layout: None,
            vertical_layout: None,
            grid_layout: None,
            children: Vec::new(),
        };
        
        // Add children from element type (for Container type with inline children)
        for child in type_children {
            prefab_element.children.push(Self::convert_element(&child));
        }
        
        // Add children from HudElement itself (for nested structures)
        for child in &hud_element.children {
            prefab_element.children.push(Self::convert_element(child));
        }
        
        // Note: We could add conversion notes to the name, but for now we skip it
        // to keep names clean. Notes can be logged or stored separately if needed.
        // if !notes.is_empty() {
        //     prefab_element.name = format!("{} /* {} */", prefab_element.name, notes);
        // }
        
        prefab_element
    }
    
    /// Convert HealthBar to a container with background and fill images
    fn convert_health_bar(
        hud_element: &HudElement,
        binding: &str,
        color: Color,
        background_color: Color,
    ) -> UIPrefabElement {
        let rect_transform = Self::convert_anchor(
            &hud_element.anchor,
            hud_element.offset,
            hud_element.size,
        );
        
        let mut ui_element = UIElement::default();
        ui_element.raycast_target = hud_element.visible;
        ui_element.interactable = hud_element.visible;
        
        // Create background image
        let background = UIPrefabElement {
            name: format!("{}_background", hud_element.id),
            rect_transform: RectTransform::stretched(Vec2::ZERO, Vec2::ONE, Vec4::ZERO),
            ui_element: UIElement {
                raycast_target: false,
                blocks_raycasts: false,
                z_order: 0,
                color: background_color,
                alpha: background_color[3],
                interactable: false,
                ignore_layout: false,
                canvas_entity: None,
            },
            image: Some(UIImage {
                sprite: None,
                image_type: ImageType::Simple,
                slice_borders: Vec4::ZERO,
                fill_method: FillMethod::Horizontal,
                fill_amount: 1.0,
                fill_origin: 0,
                preserve_aspect: false,
            }),
            text: None,
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
            children: Vec::new(),
        };
        
        // Create fill image
        let fill = UIPrefabElement {
            name: format!("{}_fill", hud_element.id),
            rect_transform: RectTransform::stretched(Vec2::ZERO, Vec2::ONE, Vec4::ZERO),
            ui_element: UIElement {
                raycast_target: false,
                blocks_raycasts: false,
                z_order: 1,
                color,
                alpha: color[3],
                interactable: false,
                ignore_layout: false,
                canvas_entity: None,
            },
            image: Some(UIImage {
                sprite: None,
                image_type: ImageType::Filled,
                slice_borders: Vec4::ZERO,
                fill_method: FillMethod::Horizontal,
                fill_amount: 1.0,
                fill_origin: 0,
                preserve_aspect: false,
            }),
            text: None,
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
            children: Vec::new(),
        };
        
        UIPrefabElement {
            name: hud_element.id.clone(),
            rect_transform,
            ui_element,
            image: None,
            text: None,
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
            children: vec![background, fill],
        }
    }
    
    /// Convert ProgressBar to a container with background and fill images
    fn convert_progress_bar(
        hud_element: &HudElement,
        binding: &str,
        color: Color,
        background_color: Color,
    ) -> UIPrefabElement {
        // Same as health bar
        Self::convert_health_bar(hud_element, binding, color, background_color)
    }
    
    /// Convert HUD Anchor to RectTransform
    /// 
    /// Maps the 9 anchor positions to RectTransform anchor points
    fn convert_anchor(anchor: &Anchor, offset: [f32; 2], size: [f32; 2]) -> RectTransform {
        let (anchor_min, anchor_max, pivot) = match anchor {
            Anchor::TopLeft => (Vec2::new(0.0, 1.0), Vec2::new(0.0, 1.0), Vec2::new(0.0, 1.0)),
            Anchor::TopCenter => (Vec2::new(0.5, 1.0), Vec2::new(0.5, 1.0), Vec2::new(0.5, 1.0)),
            Anchor::TopRight => (Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0)),
            Anchor::CenterLeft => (Vec2::new(0.0, 0.5), Vec2::new(0.0, 0.5), Vec2::new(0.0, 0.5)),
            Anchor::Center => (Vec2::new(0.5, 0.5), Vec2::new(0.5, 0.5), Vec2::new(0.5, 0.5)),
            Anchor::CenterRight => (Vec2::new(1.0, 0.5), Vec2::new(1.0, 0.5), Vec2::new(1.0, 0.5)),
            Anchor::BottomLeft => (Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
            Anchor::BottomCenter => (Vec2::new(0.5, 0.0), Vec2::new(0.5, 0.0), Vec2::new(0.5, 0.0)),
            Anchor::BottomRight => (Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0)),
        };
        
        RectTransform {
            anchor_min,
            anchor_max,
            pivot,
            anchored_position: Vec2::new(offset[0], offset[1]),
            size_delta: Vec2::new(size[0], size[1]),
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: crate::types::Rect::default(),
            dirty: true,
        }
    }
    
    /// Convert HudElementType to UI components
    /// 
    /// Returns: (image, text, panel, children, conversion_notes)
    fn convert_element_type(
        element_type: &HudElementType,
    ) -> (Option<UIImage>, Option<UIText>, Option<UIPanel>, Vec<HudElement>, String) {
        match element_type {
            HudElementType::Text { text, font_size, color } => {
                let ui_text = UIText {
                    text: text.clone(),
                    font: "default".to_string(),
                    font_size: *font_size,
                    color: *color,
                    alignment: TextAlignment::MiddleCenter,
                    ..Default::default()
                };
                (None, Some(ui_text), None, Vec::new(), String::new())
            }
            
            HudElementType::DynamicText { format, font_size, color } => {
                let ui_text = UIText {
                    text: format.clone(),
                    font: "default".to_string(),
                    font_size: *font_size,
                    color: *color,
                    alignment: TextAlignment::MiddleCenter,
                    ..Default::default()
                };
                let notes = format!(
                    "DynamicText: Bind '{}' in Lua using set_text()",
                    format
                );
                (None, Some(ui_text), None, Vec::new(), notes)
            }
            
            HudElementType::HealthBar { .. } => {
                // Handled separately in convert_element
                unreachable!("HealthBar should be handled in convert_element")
            }
            
            HudElementType::ProgressBar { .. } => {
                // Handled separately in convert_element
                unreachable!("ProgressBar should be handled in convert_element")
            }
            
            HudElementType::Image { texture, tint } => {
                let ui_image = UIImage {
                    sprite: Some(texture.clone()),
                    image_type: ImageType::Simple,
                    slice_borders: Vec4::ZERO,
                    fill_method: FillMethod::Horizontal,
                    fill_amount: 1.0,
                    fill_origin: 0,
                    preserve_aspect: false,
                };
                // Store tint in UIElement color (handled by caller)
                let notes = if *tint != [1.0, 1.0, 1.0, 1.0] {
                    format!("Image tint: {:?}", tint)
                } else {
                    String::new()
                };
                (Some(ui_image), None, None, Vec::new(), notes)
            }
            
            HudElementType::Container { children } => {
                // Container becomes a panel with children
                let panel = UIPanel {
                    background: None,
                    use_nine_slice: false,
                    slice_borders: Vec4::ZERO,
                    padding: Vec4::ZERO,
                };
                (None, None, Some(panel), children.clone(), String::new())
            }
            
            HudElementType::Minimap { zoom, background_color } => {
                // Minimap is a custom component - create a panel with notes
                let panel = UIPanel {
                    background: None,
                    use_nine_slice: false,
                    slice_borders: Vec4::ZERO,
                    padding: Vec4::ZERO,
                };
                let notes = format!(
                    "Minimap: Custom component needed. Zoom={}, BG={:?}. Implement in Lua or as custom UI component.",
                    zoom, background_color
                );
                (None, None, Some(panel), Vec::new(), notes)
            }
        }
    }
}

// Re-export HUD types for convenience
// These would normally come from the engine crate, but we define them here
// to avoid circular dependencies during migration

use serde::{Deserialize, Serialize};

/// HUD Asset - defines a complete HUD layout
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudAsset {
    pub name: String,
    pub elements: Vec<HudElement>,
}

/// Individual HUD element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudElement {
    pub id: String,
    pub element_type: HudElementTypeWrapper,
    pub anchor: Anchor,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub visible: bool,
    #[serde(default)]
    pub children: Vec<HudElement>,
}

/// Wrapper for element type with type field
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudElementTypeWrapper {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Legacy enum for internal use
#[derive(Clone, Debug)]
pub enum HudElementType {
    HealthBar {
        binding: String,
        color: Color,
        background_color: Color,
    },
    ProgressBar {
        binding: String,
        color: Color,
        background_color: Color,
    },
    Minimap {
        zoom: f32,
        background_color: Color,
    },
    Text {
        text: String,
        font_size: f32,
        color: Color,
    },
    DynamicText {
        format: String,
        font_size: f32,
        color: Color,
    },
    Image {
        texture: String,
        tint: Color,
    },
    Container {
        children: Vec<HudElement>,
    },
}

impl From<HudElementTypeWrapper> for HudElementType {
    fn from(wrapper: HudElementTypeWrapper) -> Self {
        match wrapper.type_name.as_str() {
            "HealthBar" => {
                let binding = wrapper.data["binding"].as_str().unwrap().to_string();
                let color = serde_json::from_value(wrapper.data["color"].clone()).unwrap();
                let background_color = serde_json::from_value(wrapper.data["background_color"].clone()).unwrap();
                HudElementType::HealthBar { binding, color, background_color }
            }
            "ProgressBar" => {
                let binding = wrapper.data["binding"].as_str().unwrap().to_string();
                let color = serde_json::from_value(wrapper.data["color"].clone()).unwrap();
                let background_color = serde_json::from_value(wrapper.data["background_color"].clone()).unwrap();
                HudElementType::ProgressBar { binding, color, background_color }
            }
            "Minimap" => {
                let zoom = wrapper.data["zoom"].as_f64().unwrap() as f32;
                let background_color = serde_json::from_value(wrapper.data["background_color"].clone()).unwrap();
                HudElementType::Minimap { zoom, background_color }
            }
            "Text" => {
                let text = wrapper.data["text"].as_str().unwrap().to_string();
                let font_size = wrapper.data["font_size"].as_f64().unwrap() as f32;
                let color = serde_json::from_value(wrapper.data["color"].clone()).unwrap();
                HudElementType::Text { text, font_size, color }
            }
            "DynamicText" => {
                let format = wrapper.data["format"].as_str().unwrap().to_string();
                let font_size = wrapper.data["font_size"].as_f64().unwrap() as f32;
                let color = serde_json::from_value(wrapper.data["color"].clone()).unwrap();
                HudElementType::DynamicText { format, font_size, color }
            }
            "Image" => {
                let texture = wrapper.data["texture"].as_str().unwrap().to_string();
                let tint = serde_json::from_value(wrapper.data["tint"].clone())
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                HudElementType::Image { texture, tint }
            }
            "Container" => {
                let children = serde_json::from_value(wrapper.data["children"].clone())
                    .unwrap_or_else(|_| Vec::new());
                HudElementType::Container { children }
            }
            _ => panic!("Unknown element type: {}", wrapper.type_name),
        }
    }
}

/// Anchor points for HUD elements
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_simple_text() {
        let hud = HudAsset {
            name: "TestHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "Label1".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "Text".to_string(),
                        data: serde_json::json!({
                            "text": "Hello World",
                            "font_size": 18.0,
                            "color": [1.0, 1.0, 1.0, 1.0]
                        }),
                    },
                    anchor: Anchor::TopLeft,
                    offset: [10.0, 10.0],
                    size: [200.0, 30.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        
        assert_eq!(prefab.name, "TestHUD");
        assert_eq!(prefab.root.children.len(), 1);
        
        let child = &prefab.root.children[0];
        assert_eq!(child.name, "Label1");
        assert!(child.text.is_some());
        
        let text = child.text.as_ref().unwrap();
        assert_eq!(text.text, "Hello World");
        assert_eq!(text.font_size, 18.0);
    }
    
    #[test]
    fn test_convert_anchor_top_left() {
        let rt = HudToUIPrefabConverter::convert_anchor(
            &Anchor::TopLeft,
            [10.0, 20.0],
            [100.0, 50.0],
        );
        
        assert_eq!(rt.anchor_min, Vec2::new(0.0, 1.0));
        assert_eq!(rt.anchor_max, Vec2::new(0.0, 1.0));
        assert_eq!(rt.pivot, Vec2::new(0.0, 1.0));
        assert_eq!(rt.anchored_position, Vec2::new(10.0, 20.0));
        assert_eq!(rt.size_delta, Vec2::new(100.0, 50.0));
    }
    
    #[test]
    fn test_convert_anchor_center() {
        let rt = HudToUIPrefabConverter::convert_anchor(
            &Anchor::Center,
            [0.0, 0.0],
            [150.0, 75.0],
        );
        
        assert_eq!(rt.anchor_min, Vec2::new(0.5, 0.5));
        assert_eq!(rt.anchor_max, Vec2::new(0.5, 0.5));
        assert_eq!(rt.pivot, Vec2::new(0.5, 0.5));
        assert_eq!(rt.anchored_position, Vec2::ZERO);
        assert_eq!(rt.size_delta, Vec2::new(150.0, 75.0));
    }
    
    #[test]
    fn test_convert_anchor_bottom_right() {
        let rt = HudToUIPrefabConverter::convert_anchor(
            &Anchor::BottomRight,
            [-10.0, -10.0],
            [80.0, 40.0],
        );
        
        assert_eq!(rt.anchor_min, Vec2::new(1.0, 0.0));
        assert_eq!(rt.anchor_max, Vec2::new(1.0, 0.0));
        assert_eq!(rt.pivot, Vec2::new(1.0, 0.0));
        assert_eq!(rt.anchored_position, Vec2::new(-10.0, -10.0));
        assert_eq!(rt.size_delta, Vec2::new(80.0, 40.0));
    }
    
    #[test]
    fn test_convert_all_anchors() {
        let anchors = vec![
            (Anchor::TopLeft, Vec2::new(0.0, 1.0)),
            (Anchor::TopCenter, Vec2::new(0.5, 1.0)),
            (Anchor::TopRight, Vec2::new(1.0, 1.0)),
            (Anchor::CenterLeft, Vec2::new(0.0, 0.5)),
            (Anchor::Center, Vec2::new(0.5, 0.5)),
            (Anchor::CenterRight, Vec2::new(1.0, 0.5)),
            (Anchor::BottomLeft, Vec2::new(0.0, 0.0)),
            (Anchor::BottomCenter, Vec2::new(0.5, 0.0)),
            (Anchor::BottomRight, Vec2::new(1.0, 0.0)),
        ];
        
        for (anchor, expected_pos) in anchors {
            let rt = HudToUIPrefabConverter::convert_anchor(
                &anchor,
                [0.0, 0.0],
                [100.0, 100.0],
            );
            assert_eq!(rt.anchor_min, expected_pos, "Failed for {:?}", anchor);
            assert_eq!(rt.anchor_max, expected_pos, "Failed for {:?}", anchor);
            assert_eq!(rt.pivot, expected_pos, "Failed for {:?}", anchor);
        }
    }
    
    #[test]
    fn test_convert_image() {
        let hud = HudAsset {
            name: "ImageHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "Icon".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "Image".to_string(),
                        data: serde_json::json!({
                            "texture": "icon.png",
                            "tint": [1.0, 1.0, 1.0, 1.0]
                        }),
                    },
                    anchor: Anchor::TopRight,
                    offset: [-50.0, 10.0],
                    size: [40.0, 40.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        let child = &prefab.root.children[0];
        
        assert!(child.image.is_some());
        let image = child.image.as_ref().unwrap();
        assert_eq!(image.sprite, Some("icon.png".to_string()));
    }
    
    #[test]
    fn test_convert_dynamic_text() {
        let hud = HudAsset {
            name: "DynamicHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "Score".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "DynamicText".to_string(),
                        data: serde_json::json!({
                            "format": "Score: {score}",
                            "font_size": 20.0,
                            "color": [1.0, 1.0, 0.0, 1.0]
                        }),
                    },
                    anchor: Anchor::TopCenter,
                    offset: [0.0, 10.0],
                    size: [150.0, 30.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        let child = &prefab.root.children[0];
        
        assert!(child.text.is_some());
        assert!(child.name.contains("DynamicText"));
        assert!(child.name.contains("Lua"));
    }
    
    #[test]
    fn test_convert_container_with_children() {
        let hud = HudAsset {
            name: "ContainerHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "Panel".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "Container".to_string(),
                        data: serde_json::json!({
                            "children": [
                                {
                                    "id": "ChildText",
                                    "element_type": {
                                        "type": "Text",
                                        "text": "Child",
                                        "font_size": 14.0,
                                        "color": [1.0, 1.0, 1.0, 1.0]
                                    },
                                    "anchor": "Center",
                                    "offset": [0.0, 0.0],
                                    "size": [100.0, 20.0],
                                    "visible": true
                                }
                            ]
                        }),
                    },
                    anchor: Anchor::Center,
                    offset: [0.0, 0.0],
                    size: [200.0, 100.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        let container = &prefab.root.children[0];
        
        assert_eq!(container.name, "Panel");
        assert!(container.panel.is_some());
        assert_eq!(container.children.len(), 1);
        
        let child = &container.children[0];
        assert_eq!(child.name, "ChildText");
        assert!(child.text.is_some());
    }
    
    #[test]
    fn test_convert_health_bar() {
        let hud = HudAsset {
            name: "HealthHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "HealthBar".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "HealthBar".to_string(),
                        data: serde_json::json!({
                            "binding": "player.health",
                            "color": [1.0, 0.0, 0.0, 1.0],
                            "background_color": [0.2, 0.2, 0.2, 0.8]
                        }),
                    },
                    anchor: Anchor::TopLeft,
                    offset: [10.0, 10.0],
                    size: [200.0, 20.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        let health_bar = &prefab.root.children[0];
        
        assert!(health_bar.panel.is_some());
        assert!(health_bar.name.contains("HealthBar"));
        assert!(health_bar.name.contains("player.health"));
    }
    
    #[test]
    fn test_convert_minimap() {
        let hud = HudAsset {
            name: "MinimapHUD".to_string(),
            elements: vec![
                HudElement {
                    id: "Minimap".to_string(),
                    element_type: HudElementTypeWrapper {
                        type_name: "Minimap".to_string(),
                        data: serde_json::json!({
                            "zoom": 2.0,
                            "background_color": [0.1, 0.1, 0.1, 0.9]
                        }),
                    },
                    anchor: Anchor::BottomRight,
                    offset: [-150.0, 10.0],
                    size: [140.0, 140.0],
                    visible: true,
                    children: vec![],
                },
            ],
        };
        
        let prefab = HudToUIPrefabConverter::convert(&hud);
        let minimap = &prefab.root.children[0];
        
        assert!(minimap.panel.is_some());
        assert!(minimap.name.contains("Minimap"));
        assert!(minimap.name.contains("Custom component"));
    }
}

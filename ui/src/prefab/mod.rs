//! UI prefab system for reusable UI templates

use serde::{Deserialize, Serialize};
use crate::{
    RectTransform, UIElement, UIImage, UIText, UIButton, UIPanel,
    UISlider, UIToggle, UIDropdown, UIInputField, UIScrollView,
    UIMask, HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup,
};

/// UI Prefab for reusable UI templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefab {
    /// Prefab name
    pub name: String,
    
    /// Root element data
    pub root: UIPrefabElement,
}

/// UI Prefab element (recursive structure)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefabElement {
    /// Element name
    pub name: String,
    
    /// Components
    pub rect_transform: RectTransform,
    pub ui_element: UIElement,
    pub image: Option<UIImage>,
    pub text: Option<UIText>,
    pub button: Option<UIButton>,
    pub panel: Option<UIPanel>,
    pub slider: Option<UISlider>,
    pub toggle: Option<UIToggle>,
    pub dropdown: Option<UIDropdown>,
    pub input_field: Option<UIInputField>,
    pub scroll_view: Option<UIScrollView>,
    pub mask: Option<UIMask>,
    pub horizontal_layout: Option<HorizontalLayoutGroup>,
    pub vertical_layout: Option<VerticalLayoutGroup>,
    pub grid_layout: Option<GridLayoutGroup>,
    
    /// Children
    pub children: Vec<UIPrefabElement>,
}

use std::collections::HashMap;

/// Entity ID type (using u64 as a simple entity identifier)
pub type Entity = u64;

/// Parameters for prefab instantiation
/// 
/// Allows overriding specific properties when instantiating a prefab
#[derive(Clone, Debug, Default)]
pub struct PrefabParameters {
    /// Property overrides: (element_name, property_name, value)
    /// For example: ("Button", "text", "Click Me!")
    overrides: HashMap<String, HashMap<String, PrefabValue>>,
}

impl PrefabParameters {
    /// Create a new empty parameter set
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a text override for an element
    pub fn set_text(&mut self, element_name: &str, text: String) -> &mut Self {
        self.overrides
            .entry(element_name.to_string())
            .or_insert_with(HashMap::new)
            .insert("text".to_string(), PrefabValue::Text(text));
        self
    }
    
    /// Set a color override for an element
    pub fn set_color(&mut self, element_name: &str, color: [f32; 4]) -> &mut Self {
        self.overrides
            .entry(element_name.to_string())
            .or_insert_with(HashMap::new)
            .insert("color".to_string(), PrefabValue::Color(color));
        self
    }
    
    /// Set a sprite override for an element
    pub fn set_sprite(&mut self, element_name: &str, sprite: String) -> &mut Self {
        self.overrides
            .entry(element_name.to_string())
            .or_insert_with(HashMap::new)
            .insert("sprite".to_string(), PrefabValue::Sprite(sprite));
        self
    }
    
    /// Set a position override for an element
    pub fn set_position(&mut self, element_name: &str, position: crate::Vec2) -> &mut Self {
        self.overrides
            .entry(element_name.to_string())
            .or_insert_with(HashMap::new)
            .insert("position".to_string(), PrefabValue::Position(position));
        self
    }
    
    /// Set a size override for an element
    pub fn set_size(&mut self, element_name: &str, size: crate::Vec2) -> &mut Self {
        self.overrides
            .entry(element_name.to_string())
            .or_insert_with(HashMap::new)
            .insert("size".to_string(), PrefabValue::Size(size));
        self
    }
    
    /// Get an override value for an element and property
    fn get(&self, element_name: &str, property_name: &str) -> Option<&PrefabValue> {
        self.overrides
            .get(element_name)
            .and_then(|props| props.get(property_name))
    }
}

/// Values that can be overridden in prefab parameters
#[derive(Clone, Debug)]
pub enum PrefabValue {
    Text(String),
    Color([f32; 4]),
    Sprite(String),
    Position(crate::Vec2),
    Size(crate::Vec2),
}

/// Result of prefab instantiation
#[derive(Clone, Debug)]
pub struct InstantiatedPrefab {
    /// Root entity of the instantiated prefab
    pub root_entity: Entity,
    
    /// Map of element names to entity IDs
    pub named_entities: HashMap<String, Entity>,
}

/// Prefab instantiation system
/// 
/// Handles creating entities from UI prefabs with support for parameterization
pub struct PrefabInstantiator {
    /// Next entity ID to assign
    next_entity_id: Entity,
    
    /// Component storage
    pub rect_transforms: HashMap<Entity, RectTransform>,
    pub ui_elements: HashMap<Entity, UIElement>,
    pub images: HashMap<Entity, UIImage>,
    pub texts: HashMap<Entity, UIText>,
    pub buttons: HashMap<Entity, UIButton>,
    pub panels: HashMap<Entity, UIPanel>,
    pub sliders: HashMap<Entity, UISlider>,
    pub toggles: HashMap<Entity, UIToggle>,
    pub dropdowns: HashMap<Entity, UIDropdown>,
    pub input_fields: HashMap<Entity, UIInputField>,
    pub scroll_views: HashMap<Entity, UIScrollView>,
    pub masks: HashMap<Entity, UIMask>,
    pub horizontal_layouts: HashMap<Entity, HorizontalLayoutGroup>,
    pub vertical_layouts: HashMap<Entity, VerticalLayoutGroup>,
    pub grid_layouts: HashMap<Entity, GridLayoutGroup>,
    
    /// Parent-child relationships
    pub parents: HashMap<Entity, Entity>,
    pub children: HashMap<Entity, Vec<Entity>>,
}

impl PrefabInstantiator {
    /// Create a new prefab instantiator
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            rect_transforms: HashMap::new(),
            ui_elements: HashMap::new(),
            images: HashMap::new(),
            texts: HashMap::new(),
            buttons: HashMap::new(),
            panels: HashMap::new(),
            sliders: HashMap::new(),
            toggles: HashMap::new(),
            dropdowns: HashMap::new(),
            input_fields: HashMap::new(),
            scroll_views: HashMap::new(),
            masks: HashMap::new(),
            horizontal_layouts: HashMap::new(),
            vertical_layouts: HashMap::new(),
            grid_layouts: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    
    /// Instantiate a prefab without parameters
    /// 
    /// Creates all entities from the prefab hierarchy and returns the root entity
    pub fn instantiate(&mut self, prefab: &UIPrefab) -> InstantiatedPrefab {
        self.instantiate_with_params(prefab, &PrefabParameters::new())
    }
    
    /// Instantiate a prefab with parameters
    /// 
    /// Creates all entities from the prefab hierarchy, applies parameter overrides,
    /// and returns the root entity along with named entity mappings
    pub fn instantiate_with_params(
        &mut self,
        prefab: &UIPrefab,
        params: &PrefabParameters,
    ) -> InstantiatedPrefab {
        let mut named_entities = HashMap::new();
        
        let root_entity = self.instantiate_element(&prefab.root, None, params, &mut named_entities);
        
        InstantiatedPrefab {
            root_entity,
            named_entities,
        }
    }
    
    /// Instantiate a single prefab element and its children recursively
    fn instantiate_element(
        &mut self,
        element: &UIPrefabElement,
        parent: Option<Entity>,
        params: &PrefabParameters,
        named_entities: &mut HashMap<String, Entity>,
    ) -> Entity {
        // Create new entity
        let entity = self.next_entity_id;
        self.next_entity_id += 1;
        
        // Store entity name mapping
        named_entities.insert(element.name.clone(), entity);
        
        // Apply RectTransform with parameter overrides
        let mut rect_transform = element.rect_transform.clone();
        if let Some(PrefabValue::Position(pos)) = params.get(&element.name, "position") {
            rect_transform.anchored_position = *pos;
        }
        if let Some(PrefabValue::Size(size)) = params.get(&element.name, "size") {
            rect_transform.set_size(*size);
        }
        self.rect_transforms.insert(entity, rect_transform);
        
        // Apply UIElement with parameter overrides
        let mut ui_element = element.ui_element.clone();
        if let Some(PrefabValue::Color(color)) = params.get(&element.name, "color") {
            ui_element.color = *color;
        }
        self.ui_elements.insert(entity, ui_element);
        
        // Apply optional components with parameter overrides
        if let Some(mut image) = element.image.clone() {
            if let Some(PrefabValue::Sprite(sprite)) = params.get(&element.name, "sprite") {
                image.sprite = Some(sprite.clone());
            }
            self.images.insert(entity, image);
        }
        
        if let Some(mut text) = element.text.clone() {
            if let Some(PrefabValue::Text(text_value)) = params.get(&element.name, "text") {
                text.text = text_value.clone();
            }
            if let Some(PrefabValue::Color(color)) = params.get(&element.name, "text_color") {
                text.color = *color;
            }
            self.texts.insert(entity, text);
        }
        
        if let Some(button) = element.button.clone() {
            self.buttons.insert(entity, button);
        }
        
        if let Some(panel) = element.panel.clone() {
            self.panels.insert(entity, panel);
        }
        
        if let Some(slider) = element.slider.clone() {
            self.sliders.insert(entity, slider);
        }
        
        if let Some(toggle) = element.toggle.clone() {
            self.toggles.insert(entity, toggle);
        }
        
        if let Some(dropdown) = element.dropdown.clone() {
            self.dropdowns.insert(entity, dropdown);
        }
        
        if let Some(input_field) = element.input_field.clone() {
            self.input_fields.insert(entity, input_field);
        }
        
        if let Some(scroll_view) = element.scroll_view.clone() {
            self.scroll_views.insert(entity, scroll_view);
        }
        
        if let Some(mask) = element.mask.clone() {
            self.masks.insert(entity, mask);
        }
        
        if let Some(horizontal_layout) = element.horizontal_layout.clone() {
            self.horizontal_layouts.insert(entity, horizontal_layout);
        }
        
        if let Some(vertical_layout) = element.vertical_layout.clone() {
            self.vertical_layouts.insert(entity, vertical_layout);
        }
        
        if let Some(grid_layout) = element.grid_layout.clone() {
            self.grid_layouts.insert(entity, grid_layout);
        }
        
        // Set up parent-child relationship
        if let Some(parent_entity) = parent {
            self.parents.insert(entity, parent_entity);
            self.children
                .entry(parent_entity)
                .or_insert_with(Vec::new)
                .push(entity);
        }
        
        // Instantiate children recursively
        for child_element in &element.children {
            self.instantiate_element(child_element, Some(entity), params, named_entities);
        }
        
        entity
    }
    
    /// Get an entity by name from a previously instantiated prefab
    pub fn get_entity_by_name(&self, result: &InstantiatedPrefab, name: &str) -> Option<Entity> {
        result.named_entities.get(name).copied()
    }
    
    /// Destroy an entity and all its descendants
    pub fn destroy_entity(&mut self, entity: Entity) {
        // Collect all descendants
        let mut to_destroy = vec![entity];
        let mut i = 0;
        while i < to_destroy.len() {
            let current = to_destroy[i];
            if let Some(children) = self.children.get(&current) {
                to_destroy.extend(children.iter().copied());
            }
            i += 1;
        }
        
        // Remove all entities and their components
        for entity in to_destroy {
            self.rect_transforms.remove(&entity);
            self.ui_elements.remove(&entity);
            self.images.remove(&entity);
            self.texts.remove(&entity);
            self.buttons.remove(&entity);
            self.panels.remove(&entity);
            self.sliders.remove(&entity);
            self.toggles.remove(&entity);
            self.dropdowns.remove(&entity);
            self.input_fields.remove(&entity);
            self.scroll_views.remove(&entity);
            self.masks.remove(&entity);
            self.horizontal_layouts.remove(&entity);
            self.vertical_layouts.remove(&entity);
            self.grid_layouts.remove(&entity);
            self.parents.remove(&entity);
            self.children.remove(&entity);
        }
    }
}

impl Default for PrefabInstantiator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;
    
    fn create_simple_prefab() -> UIPrefab {
        UIPrefab {
            name: "TestPrefab".to_string(),
            root: UIPrefabElement {
                name: "Root".to_string(),
                rect_transform: RectTransform::anchored(
                    Vec2::new(0.5, 0.5),
                    Vec2::ZERO,
                    Vec2::new(200.0, 100.0),
                ),
                ui_element: UIElement::default(),
                image: Some(UIImage::default()),
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
                children: vec![],
            },
        }
    }
    
    fn create_hierarchical_prefab() -> UIPrefab {
        UIPrefab {
            name: "HierarchicalPrefab".to_string(),
            root: UIPrefabElement {
                name: "Root".to_string(),
                rect_transform: RectTransform::default(),
                ui_element: UIElement::default(),
                image: None,
                text: None,
                button: None,
                panel: Some(UIPanel::default()),
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
                        name: "Child1".to_string(),
                        rect_transform: RectTransform::default(),
                        ui_element: UIElement::default(),
                        image: None,
                        text: Some(UIText {
                            text: "Hello".to_string(),
                            ..Default::default()
                        }),
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
                        name: "Child2".to_string(),
                        rect_transform: RectTransform::default(),
                        ui_element: UIElement::default(),
                        image: None,
                        text: None,
                        button: Some(UIButton::default()),
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
    
    #[test]
    fn test_instantiate_simple_prefab() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let result = instantiator.instantiate(&prefab);
        
        // Check that root entity was created
        assert!(result.root_entity > 0);
        
        // Check that components were created
        assert!(instantiator.rect_transforms.contains_key(&result.root_entity));
        assert!(instantiator.ui_elements.contains_key(&result.root_entity));
        assert!(instantiator.images.contains_key(&result.root_entity));
        
        // Check that entity name mapping was created
        assert_eq!(result.named_entities.get("Root"), Some(&result.root_entity));
    }
    
    #[test]
    fn test_instantiate_hierarchical_prefab() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_hierarchical_prefab();
        
        let result = instantiator.instantiate(&prefab);
        
        // Check that all entities were created
        assert!(result.named_entities.contains_key("Root"));
        assert!(result.named_entities.contains_key("Child1"));
        assert!(result.named_entities.contains_key("Child2"));
        
        let root = result.named_entities["Root"];
        let child1 = result.named_entities["Child1"];
        let child2 = result.named_entities["Child2"];
        
        // Check parent-child relationships
        assert_eq!(instantiator.parents.get(&child1), Some(&root));
        assert_eq!(instantiator.parents.get(&child2), Some(&root));
        
        let children = instantiator.children.get(&root).unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1));
        assert!(children.contains(&child2));
        
        // Check components
        assert!(instantiator.panels.contains_key(&root));
        assert!(instantiator.texts.contains_key(&child1));
        assert!(instantiator.buttons.contains_key(&child2));
    }
    
    #[test]
    fn test_instantiate_with_text_parameter() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_hierarchical_prefab();
        
        let mut params = PrefabParameters::new();
        params.set_text("Child1", "Modified Text".to_string());
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        let child1 = result.named_entities["Child1"];
        let text = instantiator.texts.get(&child1).unwrap();
        
        assert_eq!(text.text, "Modified Text");
    }
    
    #[test]
    fn test_instantiate_with_color_parameter() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let mut params = PrefabParameters::new();
        let custom_color = [1.0, 0.0, 0.0, 1.0];
        params.set_color("Root", custom_color);
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        let ui_element = instantiator.ui_elements.get(&result.root_entity).unwrap();
        assert_eq!(ui_element.color, custom_color);
    }
    
    #[test]
    fn test_instantiate_with_position_parameter() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let mut params = PrefabParameters::new();
        let custom_position = Vec2::new(100.0, 50.0);
        params.set_position("Root", custom_position);
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        let rect_transform = instantiator.rect_transforms.get(&result.root_entity).unwrap();
        assert_eq!(rect_transform.anchored_position, custom_position);
    }
    
    #[test]
    fn test_instantiate_with_size_parameter() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let mut params = PrefabParameters::new();
        let custom_size = Vec2::new(300.0, 150.0);
        params.set_size("Root", custom_size);
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        let rect_transform = instantiator.rect_transforms.get(&result.root_entity).unwrap();
        assert_eq!(rect_transform.get_size(), custom_size);
    }
    
    #[test]
    fn test_instantiate_with_sprite_parameter() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let mut params = PrefabParameters::new();
        params.set_sprite("Root", "custom_sprite.png".to_string());
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        let image = instantiator.images.get(&result.root_entity).unwrap();
        assert_eq!(image.sprite, Some("custom_sprite.png".to_string()));
    }
    
    #[test]
    fn test_instantiate_multiple_prefabs() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_simple_prefab();
        
        let result1 = instantiator.instantiate(&prefab);
        let result2 = instantiator.instantiate(&prefab);
        
        // Entities should be different
        assert_ne!(result1.root_entity, result2.root_entity);
        
        // Both should have components
        assert!(instantiator.rect_transforms.contains_key(&result1.root_entity));
        assert!(instantiator.rect_transforms.contains_key(&result2.root_entity));
    }
    
    #[test]
    fn test_get_entity_by_name() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_hierarchical_prefab();
        
        let result = instantiator.instantiate(&prefab);
        
        let child1 = instantiator.get_entity_by_name(&result, "Child1");
        assert!(child1.is_some());
        assert_eq!(child1, result.named_entities.get("Child1").copied());
        
        let nonexistent = instantiator.get_entity_by_name(&result, "NonExistent");
        assert!(nonexistent.is_none());
    }
    
    #[test]
    fn test_destroy_entity() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_hierarchical_prefab();
        
        let result = instantiator.instantiate(&prefab);
        let root = result.root_entity;
        let child1 = result.named_entities["Child1"];
        let child2 = result.named_entities["Child2"];
        
        // Verify entities exist
        assert!(instantiator.rect_transforms.contains_key(&root));
        assert!(instantiator.rect_transforms.contains_key(&child1));
        assert!(instantiator.rect_transforms.contains_key(&child2));
        
        // Destroy root (should destroy all descendants)
        instantiator.destroy_entity(root);
        
        // Verify all entities were destroyed
        assert!(!instantiator.rect_transforms.contains_key(&root));
        assert!(!instantiator.rect_transforms.contains_key(&child1));
        assert!(!instantiator.rect_transforms.contains_key(&child2));
        assert!(!instantiator.panels.contains_key(&root));
        assert!(!instantiator.texts.contains_key(&child1));
        assert!(!instantiator.buttons.contains_key(&child2));
    }
    
    #[test]
    fn test_instantiate_with_multiple_parameters() {
        let mut instantiator = PrefabInstantiator::new();
        let prefab = create_hierarchical_prefab();
        
        let mut params = PrefabParameters::new();
        params
            .set_text("Child1", "New Text".to_string())
            .set_color("Root", [0.5, 0.5, 0.5, 1.0])
            .set_position("Child2", Vec2::new(50.0, 25.0));
        
        let result = instantiator.instantiate_with_params(&prefab, &params);
        
        // Verify all parameters were applied
        let root = result.root_entity;
        let child1 = result.named_entities["Child1"];
        let child2 = result.named_entities["Child2"];
        
        let root_element = instantiator.ui_elements.get(&root).unwrap();
        assert_eq!(root_element.color, [0.5, 0.5, 0.5, 1.0]);
        
        let child1_text = instantiator.texts.get(&child1).unwrap();
        assert_eq!(child1_text.text, "New Text");
        
        let child2_transform = instantiator.rect_transforms.get(&child2).unwrap();
        assert_eq!(child2_transform.anchored_position, Vec2::new(50.0, 25.0));
    }
}

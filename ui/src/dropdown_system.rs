//! Dropdown interaction system
//!
//! Handles dropdown list display/hide, option selection, and caption updates.

use crate::{UIDropdown, DropdownOption, UIElement, UIText};
use crate::events::{UIEvent, UIEventType, UIEventDispatcher};
use std::collections::HashMap;

/// Entity type alias
pub type Entity = u64;

/// Dropdown interaction system
pub struct DropdownSystem {
    /// Currently open dropdowns
    open_dropdowns: HashMap<Entity, bool>,
    
    /// Pending option selections (entity -> selected index)
    pending_selections: HashMap<Entity, i32>,
}

impl Default for DropdownSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl DropdownSystem {
    /// Create a new dropdown system
    pub fn new() -> Self {
        Self {
            open_dropdowns: HashMap::new(),
            pending_selections: HashMap::new(),
        }
    }
    
    /// Update dropdown states based on events
    pub fn update_from_events(
        &mut self,
        events: &[UIEvent],
        dropdowns: &mut HashMap<Entity, UIDropdown>,
        elements: &HashMap<Entity, UIElement>,
        event_dispatcher: &UIEventDispatcher,
    ) {
        for event in events {
            match event {
                UIEvent::PointerClick(entity, _pos) => {
                    // Check if this is a dropdown being clicked
                    if dropdowns.contains_key(entity) {
                        if elements.get(entity).map(|e| e.interactable).unwrap_or(false) {
                            // Toggle dropdown open/closed
                            let is_open = self.open_dropdowns.get(entity).copied().unwrap_or(false);
                            self.open_dropdowns.insert(*entity, !is_open);
                        }
                    } else {
                        // Check if this is a dropdown item being clicked
                        // In a real implementation, we would need to track which dropdown owns which items
                        // For now, we'll use a simplified approach
                        for (dropdown_entity, dropdown) in dropdowns.iter() {
                            if self.is_dropdown_open(*dropdown_entity) {
                                // Check if the clicked entity is a child of the dropdown template
                                // This is simplified - in reality we'd need proper hierarchy tracking
                                if let Some(template) = dropdown.template {
                                    // Assume the entity is an item if it's not the dropdown itself
                                    // In a real implementation, we'd check the hierarchy
                                    if *entity != *dropdown_entity {
                                        // Find which option was clicked based on entity
                                        // This is a placeholder - real implementation would track item entities
                                        let selected_index = 0; // Placeholder
                                        self.pending_selections.insert(*dropdown_entity, selected_index);
                                        self.open_dropdowns.insert(*dropdown_entity, false);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Apply pending selections
        for (entity, selected_index) in self.pending_selections.drain() {
            if let Some(dropdown) = dropdowns.get_mut(&entity) {
                if selected_index >= 0 && (selected_index as usize) < dropdown.options.len() {
                    dropdown.value = selected_index;
                    // Note: In a real implementation, we would trigger the on_value_changed callback here
                }
            }
        }
    }
    
    /// Update caption text for all dropdowns
    pub fn update_caption_texts(
        &self,
        dropdowns: &HashMap<Entity, UIDropdown>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        for (entity, dropdown) in dropdowns {
            if let Some(caption_entity) = dropdown.caption_text {
                if let Some(caption_text) = texts.get_mut(&caption_entity) {
                    // Update caption to show selected option
                    if dropdown.value >= 0 && (dropdown.value as usize) < dropdown.options.len() {
                        caption_text.text = dropdown.options[dropdown.value as usize].text.clone();
                    } else {
                        caption_text.text = String::new();
                    }
                }
            }
        }
    }
    
    /// Update template visibility for all dropdowns
    pub fn update_template_visibility(
        &self,
        dropdowns: &HashMap<Entity, UIDropdown>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        for (entity, dropdown) in dropdowns {
            if let Some(template_entity) = dropdown.template {
                if let Some(template_element) = elements.get_mut(&template_entity) {
                    let is_open = self.is_dropdown_open(*entity);
                    template_element.alpha = if is_open { 1.0 } else { 0.0 };
                }
            }
        }
    }
    
    /// Check if a dropdown is open
    pub fn is_dropdown_open(&self, entity: Entity) -> bool {
        self.open_dropdowns.get(&entity).copied().unwrap_or(false)
    }
    
    /// Open a dropdown programmatically
    pub fn open_dropdown(
        &mut self,
        entity: Entity,
        dropdowns: &HashMap<Entity, UIDropdown>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        if dropdowns.contains_key(&entity) {
            self.open_dropdowns.insert(entity, true);
            self.update_template_visibility(dropdowns, elements);
        }
    }
    
    /// Close a dropdown programmatically
    pub fn close_dropdown(
        &mut self,
        entity: Entity,
        dropdowns: &HashMap<Entity, UIDropdown>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        self.open_dropdowns.insert(entity, false);
        self.update_template_visibility(dropdowns, elements);
    }
    
    /// Close all dropdowns
    pub fn close_all_dropdowns(
        &mut self,
        dropdowns: &HashMap<Entity, UIDropdown>,
        elements: &mut HashMap<Entity, UIElement>,
    ) {
        for entity in dropdowns.keys() {
            self.open_dropdowns.insert(*entity, false);
        }
        self.update_template_visibility(dropdowns, elements);
    }
    
    /// Set dropdown value programmatically
    pub fn set_dropdown_value(
        entity: Entity,
        value: i32,
        dropdowns: &mut HashMap<Entity, UIDropdown>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        if let Some(dropdown) = dropdowns.get_mut(&entity) {
            if value >= 0 && (value as usize) < dropdown.options.len() {
                dropdown.value = value;
            }
        }
        
        // Update caption text
        let system = DropdownSystem::new();
        system.update_caption_texts(dropdowns, texts);
    }
    
    /// Add an option to a dropdown
    pub fn add_option(
        entity: Entity,
        option: DropdownOption,
        dropdowns: &mut HashMap<Entity, UIDropdown>,
    ) {
        if let Some(dropdown) = dropdowns.get_mut(&entity) {
            dropdown.options.push(option);
        }
    }
    
    /// Remove an option from a dropdown
    pub fn remove_option(
        entity: Entity,
        index: usize,
        dropdowns: &mut HashMap<Entity, UIDropdown>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        if let Some(dropdown) = dropdowns.get_mut(&entity) {
            if index < dropdown.options.len() {
                dropdown.options.remove(index);
                
                // Adjust selected value if necessary
                if dropdown.value >= dropdown.options.len() as i32 {
                    dropdown.value = (dropdown.options.len() as i32 - 1).max(0);
                }
            }
        }
        
        // Update caption text
        let system = DropdownSystem::new();
        system.update_caption_texts(dropdowns, texts);
    }
    
    /// Clear all options from a dropdown
    pub fn clear_options(
        entity: Entity,
        dropdowns: &mut HashMap<Entity, UIDropdown>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        if let Some(dropdown) = dropdowns.get_mut(&entity) {
            dropdown.options.clear();
            dropdown.value = 0;
        }
        
        // Update caption text
        let system = DropdownSystem::new();
        system.update_caption_texts(dropdowns, texts);
    }
    
    /// Clear system state
    pub fn clear(&mut self) {
        self.open_dropdowns.clear();
        self.pending_selections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextAlignment, OverflowMode};

    fn create_test_dropdown() -> UIDropdown {
        UIDropdown {
            template: Some(2),
            caption_text: Some(3),
            item_text: Some(4),
            options: vec![
                DropdownOption {
                    text: "Option 1".to_string(),
                    image: None,
                },
                DropdownOption {
                    text: "Option 2".to_string(),
                    image: None,
                },
                DropdownOption {
                    text: "Option 3".to_string(),
                    image: None,
                },
            ],
            value: 0,
            on_value_changed: None,
        }
    }

    fn create_test_element(interactable: bool) -> UIElement {
        UIElement {
            raycast_target: true,
            blocks_raycasts: true,
            z_order: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            alpha: 1.0,
            interactable,
            ignore_layout: false,
            canvas_entity: None,
        }
    }

    fn create_test_text() -> UIText {
        UIText {
            text: String::new(),
            font: "default".to_string(),
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
            alignment: TextAlignment::MiddleCenter,
            horizontal_overflow: OverflowMode::Wrap,
            vertical_overflow: OverflowMode::Truncate,
            rich_text: false,
            line_spacing: 1.0,
            best_fit: false,
            best_fit_min_size: 10.0,
            best_fit_max_size: 40.0,
        }
    }

    #[test]
    fn test_dropdown_toggle_open() {
        let mut system = DropdownSystem::new();
        let mut dropdowns = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        dropdowns.insert(1, create_test_dropdown());
        elements.insert(1, create_test_element(true));
        
        assert!(!system.is_dropdown_open(1));
        
        // Click to open
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut dropdowns, &elements, &dispatcher);
        
        assert!(system.is_dropdown_open(1));
        
        // Click to close
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut dropdowns, &elements, &dispatcher);
        
        assert!(!system.is_dropdown_open(1));
    }

    #[test]
    fn test_set_dropdown_value() {
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        texts.insert(3, create_test_text());
        
        DropdownSystem::set_dropdown_value(1, 1, &mut dropdowns, &mut texts);
        
        let dropdown = dropdowns.get(&1).unwrap();
        assert_eq!(dropdown.value, 1);
        
        let caption = texts.get(&3).unwrap();
        assert_eq!(caption.text, "Option 2");
    }

    #[test]
    fn test_set_dropdown_value_out_of_bounds() {
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        texts.insert(3, create_test_text());
        
        DropdownSystem::set_dropdown_value(1, 10, &mut dropdowns, &mut texts);
        
        let dropdown = dropdowns.get(&1).unwrap();
        // Value should not change if out of bounds
        assert_eq!(dropdown.value, 0);
    }

    #[test]
    fn test_update_caption_text() {
        let system = DropdownSystem::new();
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        let mut dropdown = create_test_dropdown();
        dropdown.value = 2;
        dropdowns.insert(1, dropdown);
        texts.insert(3, create_test_text());
        
        system.update_caption_texts(&dropdowns, &mut texts);
        
        let caption = texts.get(&3).unwrap();
        assert_eq!(caption.text, "Option 3");
    }

    #[test]
    fn test_add_option() {
        let mut dropdowns = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        
        let new_option = DropdownOption {
            text: "Option 4".to_string(),
            image: None,
        };
        
        DropdownSystem::add_option(1, new_option, &mut dropdowns);
        
        let dropdown = dropdowns.get(&1).unwrap();
        assert_eq!(dropdown.options.len(), 4);
        assert_eq!(dropdown.options[3].text, "Option 4");
    }

    #[test]
    fn test_remove_option() {
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        texts.insert(3, create_test_text());
        
        DropdownSystem::remove_option(1, 1, &mut dropdowns, &mut texts);
        
        let dropdown = dropdowns.get(&1).unwrap();
        assert_eq!(dropdown.options.len(), 2);
        assert_eq!(dropdown.options[0].text, "Option 1");
        assert_eq!(dropdown.options[1].text, "Option 3");
    }

    #[test]
    fn test_remove_option_adjusts_value() {
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        let mut dropdown = create_test_dropdown();
        dropdown.value = 2; // Last option
        dropdowns.insert(1, dropdown);
        texts.insert(3, create_test_text());
        
        DropdownSystem::remove_option(1, 2, &mut dropdowns, &mut texts);
        
        let dropdown = dropdowns.get(&1).unwrap();
        assert_eq!(dropdown.value, 1); // Adjusted to last valid index
    }

    #[test]
    fn test_clear_options() {
        let mut dropdowns = HashMap::new();
        let mut texts = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        texts.insert(3, create_test_text());
        
        DropdownSystem::clear_options(1, &mut dropdowns, &mut texts);
        
        let dropdown = dropdowns.get(&1).unwrap();
        assert_eq!(dropdown.options.len(), 0);
        assert_eq!(dropdown.value, 0);
    }

    #[test]
    fn test_open_dropdown_programmatic() {
        let mut system = DropdownSystem::new();
        let mut dropdowns = HashMap::new();
        let mut elements = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        
        system.open_dropdown(1, &dropdowns, &mut elements);
        
        assert!(system.is_dropdown_open(1));
    }

    #[test]
    fn test_close_dropdown_programmatic() {
        let mut system = DropdownSystem::new();
        let dropdowns = HashMap::new();
        let mut elements = HashMap::new();
        
        system.open_dropdown(1, &dropdowns, &mut elements);
        system.close_dropdown(1, &dropdowns, &mut elements);
        
        assert!(!system.is_dropdown_open(1));
    }

    #[test]
    fn test_close_all_dropdowns() {
        let mut system = DropdownSystem::new();
        let mut dropdowns = HashMap::new();
        let mut elements = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        dropdowns.insert(2, create_test_dropdown());
        
        system.open_dropdown(1, &dropdowns, &mut elements);
        system.open_dropdown(2, &dropdowns, &mut elements);
        
        system.close_all_dropdowns(&dropdowns, &mut elements);
        
        assert!(!system.is_dropdown_open(1));
        assert!(!system.is_dropdown_open(2));
    }

    #[test]
    fn test_update_template_visibility() {
        let system = DropdownSystem::new();
        let mut dropdowns = HashMap::new();
        let mut elements = HashMap::new();
        
        dropdowns.insert(1, create_test_dropdown());
        elements.insert(2, create_test_element(true)); // Template element
        
        system.update_template_visibility(&dropdowns, &mut elements);
        
        let template = elements.get(&2).unwrap();
        assert_eq!(template.alpha, 0.0); // Closed by default
    }
}

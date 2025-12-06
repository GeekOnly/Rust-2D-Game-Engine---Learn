//! Input field interaction system
//!
//! Handles text input, cursor positioning, selection, and content validation.

use crate::{UIInputField, ContentType, CharacterValidation, LineType, UIElement, UIText};
use crate::events::{UIEvent, UIEventType, UIEventDispatcher};
use std::collections::HashMap;

/// Entity type alias
pub type Entity = u64;

/// Input field interaction system
pub struct InputFieldSystem {
    /// Currently focused input field
    focused_field: Option<Entity>,
    
    /// Pending text changes (entity -> new text)
    pending_changes: HashMap<Entity, String>,
}

impl Default for InputFieldSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl InputFieldSystem {
    /// Create a new input field system
    pub fn new() -> Self {
        Self {
            focused_field: None,
            pending_changes: HashMap::new(),
        }
    }
    
    /// Update input field states based on events
    pub fn update_from_events(
        &mut self,
        events: &[UIEvent],
        input_fields: &mut HashMap<Entity, UIInputField>,
        elements: &HashMap<Entity, UIElement>,
        event_dispatcher: &UIEventDispatcher,
    ) {
        for event in events {
            match event {
                UIEvent::PointerClick(entity, _pos) => {
                    // Check if clicking on an input field
                    if input_fields.contains_key(entity) {
                        if elements.get(entity).map(|e| e.interactable).unwrap_or(false) {
                            if let Some(field) = input_fields.get(entity) {
                                if !field.read_only {
                                    // Focus this field
                                    self.set_focus(*entity, input_fields);
                                }
                            }
                        }
                    } else {
                        // Clicking outside - unfocus current field
                        if let Some(focused) = self.focused_field {
                            if let Some(field) = input_fields.get_mut(&focused) {
                                field.is_focused = false;
                            }
                            self.focused_field = None;
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Apply pending changes
        for (entity, new_text) in self.pending_changes.drain() {
            if let Some(field) = input_fields.get_mut(&entity) {
                field.text = new_text;
                // Note: In a real implementation, we would trigger the on_value_changed callback here
            }
        }
    }
    
    /// Handle text input for the focused field
    pub fn handle_text_input(
        &mut self,
        character: char,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                if field.read_only {
                    return;
                }
                
                // Validate character
                if !Self::validate_character(character, &field.character_validation) {
                    return;
                }
                
                // Check character limit
                if field.character_limit > 0 && field.text.len() >= field.character_limit as usize {
                    return;
                }
                
                // Insert character at caret position
                let caret_pos = field.caret_position as usize;
                if caret_pos <= field.text.len() {
                    field.text.insert(caret_pos, character);
                    field.caret_position += 1;
                    field.selection_anchor = field.caret_position;
                }
            }
        }
    }
    
    /// Handle backspace for the focused field
    pub fn handle_backspace(
        &mut self,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                if field.read_only {
                    return;
                }
                
                if field.caret_position > 0 {
                    let caret_pos = field.caret_position as usize;
                    if caret_pos <= field.text.len() {
                        field.text.remove(caret_pos - 1);
                        field.caret_position -= 1;
                        field.selection_anchor = field.caret_position;
                    }
                }
            }
        }
    }
    
    /// Handle delete for the focused field
    pub fn handle_delete(
        &mut self,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                if field.read_only {
                    return;
                }
                
                let caret_pos = field.caret_position as usize;
                if caret_pos < field.text.len() {
                    field.text.remove(caret_pos);
                }
            }
        }
    }
    
    /// Move caret left
    pub fn move_caret_left(
        &mut self,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                if field.caret_position > 0 {
                    field.caret_position -= 1;
                    field.selection_anchor = field.caret_position;
                }
            }
        }
    }
    
    /// Move caret right
    pub fn move_caret_right(
        &mut self,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                if field.caret_position < field.text.len() as i32 {
                    field.caret_position += 1;
                    field.selection_anchor = field.caret_position;
                }
            }
        }
    }
    
    /// Set caret position
    pub fn set_caret_position(
        &mut self,
        position: i32,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                field.caret_position = position.clamp(0, field.text.len() as i32);
                field.selection_anchor = field.caret_position;
            }
        }
    }
    
    /// Set focus to an input field
    pub fn set_focus(
        &mut self,
        entity: Entity,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        // Unfocus previous field
        if let Some(prev_focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&prev_focused) {
                field.is_focused = false;
            }
        }
        
        // Focus new field
        if let Some(field) = input_fields.get_mut(&entity) {
            field.is_focused = true;
            field.caret_position = field.text.len() as i32;
            field.selection_anchor = field.caret_position;
        }
        
        self.focused_field = Some(entity);
    }
    
    /// Clear focus
    pub fn clear_focus(
        &mut self,
        input_fields: &mut HashMap<Entity, UIInputField>,
    ) {
        if let Some(focused) = self.focused_field {
            if let Some(field) = input_fields.get_mut(&focused) {
                field.is_focused = false;
            }
        }
        self.focused_field = None;
    }
    
    /// Get currently focused field
    pub fn get_focused_field(&self) -> Option<Entity> {
        self.focused_field
    }
    
    /// Validate a character against validation rules
    fn validate_character(character: char, validation: &CharacterValidation) -> bool {
        match validation {
            CharacterValidation::None => true,
            CharacterValidation::Integer => character.is_ascii_digit() || character == '-',
            CharacterValidation::Decimal => {
                character.is_ascii_digit() || character == '-' || character == '.'
            }
            CharacterValidation::Alphanumeric => character.is_alphanumeric(),
            CharacterValidation::Name => character.is_alphabetic() || character.is_whitespace(),
            CharacterValidation::EmailAddress => {
                character.is_alphanumeric() || "@.-_".contains(character)
            }
        }
    }
    
    /// Validate entire text against content type
    pub fn validate_text(text: &str, content_type: &ContentType) -> bool {
        match content_type {
            ContentType::Standard | ContentType::Autocorrected | ContentType::Password => true,
            ContentType::IntegerNumber => {
                text.is_empty() || text.parse::<i64>().is_ok()
            }
            ContentType::DecimalNumber => {
                text.is_empty() || text.parse::<f64>().is_ok()
            }
            ContentType::Alphanumeric => {
                text.chars().all(|c| c.is_alphanumeric())
            }
            ContentType::Name => {
                text.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
            }
            ContentType::EmailAddress => {
                // Simple email validation
                text.contains('@') && text.contains('.')
            }
            ContentType::Pin => {
                text.chars().all(|c| c.is_ascii_digit())
            }
            ContentType::Custom => true, // Custom validation would be handled elsewhere
        }
    }
    
    /// Set text programmatically
    pub fn set_text(
        entity: Entity,
        text: String,
        input_fields: &mut HashMap<Entity, UIInputField>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        if let Some(field) = input_fields.get_mut(&entity) {
            // Validate text
            if Self::validate_text(&text, &field.content_type) {
                // Apply character limit
                let limited_text = if field.character_limit > 0 {
                    text.chars().take(field.character_limit as usize).collect()
                } else {
                    text
                };
                
                field.text = limited_text;
                field.caret_position = field.text.len() as i32;
                field.selection_anchor = field.caret_position;
            }
        }
        
        // Update text component
        let system = InputFieldSystem::new();
        system.update_text_components(input_fields, texts);
    }
    
    /// Update text components for all input fields
    pub fn update_text_components(
        &self,
        input_fields: &HashMap<Entity, UIInputField>,
        texts: &mut HashMap<Entity, UIText>,
    ) {
        for (entity, field) in input_fields {
            if let Some(text_entity) = field.text_component {
                if let Some(text_component) = texts.get_mut(&text_entity) {
                    text_component.text = field.text.clone();
                }
            }
            
            // Update placeholder visibility
            if let Some(placeholder_entity) = field.placeholder {
                if let Some(placeholder_element) = texts.get_mut(&placeholder_entity) {
                    // Placeholder should be visible when text is empty
                    // In a real implementation, we'd set alpha on UIElement
                    // For now, we just clear the text
                    if field.text.is_empty() {
                        // Placeholder visible
                    } else {
                        // Placeholder hidden
                    }
                }
            }
        }
    }
    
    /// Clear system state
    pub fn clear(&mut self) {
        self.focused_field = None;
        self.pending_changes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputType, KeyboardType, LineType};

    fn create_test_input_field() -> UIInputField {
        UIInputField {
            text_component: Some(2),
            placeholder: Some(3),
            text: String::new(),
            character_limit: 0,
            content_type: ContentType::Standard,
            line_type: LineType::SingleLine,
            input_type: InputType::Standard,
            keyboard_type: KeyboardType::Default,
            character_validation: CharacterValidation::None,
            caret_blink_rate: 0.85,
            caret_width: 1,
            selection_color: [0.65, 0.8, 1.0, 0.75],
            read_only: false,
            on_value_changed: None,
            on_end_edit: None,
            caret_position: 0,
            selection_anchor: 0,
            is_focused: false,
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

    #[test]
    fn test_focus_on_click() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        let mut elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        input_fields.insert(1, create_test_input_field());
        elements.insert(1, create_test_element(true));
        
        let events = vec![UIEvent::PointerClick(1, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut input_fields, &elements, &dispatcher);
        
        assert_eq!(system.get_focused_field(), Some(1));
        assert!(input_fields.get(&1).unwrap().is_focused);
    }

    #[test]
    fn test_unfocus_on_outside_click() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        let elements = HashMap::new();
        let dispatcher = UIEventDispatcher::new();
        
        input_fields.insert(1, create_test_input_field());
        system.set_focus(1, &mut input_fields);
        
        // Click outside (entity 999 doesn't exist)
        let events = vec![UIEvent::PointerClick(999, glam::Vec2::ZERO)];
        system.update_from_events(&events, &mut input_fields, &elements, &dispatcher);
        
        assert_eq!(system.get_focused_field(), None);
        assert!(!input_fields.get(&1).unwrap().is_focused);
    }

    #[test]
    fn test_text_input() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        input_fields.insert(1, create_test_input_field());
        system.set_focus(1, &mut input_fields);
        
        system.handle_text_input('H', &mut input_fields);
        system.handle_text_input('i', &mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "Hi");
        assert_eq!(field.caret_position, 2);
    }

    #[test]
    fn test_backspace() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.text = "Hello".to_string();
        field.caret_position = 5;
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        system.handle_backspace(&mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "Hell");
        assert_eq!(field.caret_position, 4);
    }

    #[test]
    fn test_delete() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.text = "Hello".to_string();
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        // Set caret position after focus (focus sets it to end)
        system.set_caret_position(0, &mut input_fields);
        
        system.handle_delete(&mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "ello");
        assert_eq!(field.caret_position, 0);
    }

    #[test]
    fn test_move_caret() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.text = "Hello".to_string();
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        // Set caret position after focus (focus sets it to end)
        system.set_caret_position(2, &mut input_fields);
        
        system.move_caret_right(&mut input_fields);
        assert_eq!(input_fields.get(&1).unwrap().caret_position, 3);
        
        system.move_caret_left(&mut input_fields);
        assert_eq!(input_fields.get(&1).unwrap().caret_position, 2);
    }

    #[test]
    fn test_character_limit() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.character_limit = 3;
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        system.handle_text_input('A', &mut input_fields);
        system.handle_text_input('B', &mut input_fields);
        system.handle_text_input('C', &mut input_fields);
        system.handle_text_input('D', &mut input_fields); // Should be rejected
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "ABC");
    }

    #[test]
    fn test_integer_validation() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.character_validation = CharacterValidation::Integer;
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        system.handle_text_input('1', &mut input_fields);
        system.handle_text_input('2', &mut input_fields);
        system.handle_text_input('a', &mut input_fields); // Should be rejected
        system.handle_text_input('3', &mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "123");
    }

    #[test]
    fn test_decimal_validation() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.character_validation = CharacterValidation::Decimal;
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        system.handle_text_input('1', &mut input_fields);
        system.handle_text_input('.', &mut input_fields);
        system.handle_text_input('5', &mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "1.5");
    }

    #[test]
    fn test_read_only() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        let mut field = create_test_input_field();
        field.read_only = true;
        input_fields.insert(1, field);
        system.set_focus(1, &mut input_fields);
        
        system.handle_text_input('A', &mut input_fields);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "");
    }

    #[test]
    fn test_validate_text_integer() {
        assert!(InputFieldSystem::validate_text("123", &ContentType::IntegerNumber));
        assert!(InputFieldSystem::validate_text("-456", &ContentType::IntegerNumber));
        assert!(!InputFieldSystem::validate_text("12.3", &ContentType::IntegerNumber));
        assert!(!InputFieldSystem::validate_text("abc", &ContentType::IntegerNumber));
    }

    #[test]
    fn test_validate_text_decimal() {
        assert!(InputFieldSystem::validate_text("123.45", &ContentType::DecimalNumber));
        assert!(InputFieldSystem::validate_text("-67.89", &ContentType::DecimalNumber));
        assert!(!InputFieldSystem::validate_text("abc", &ContentType::DecimalNumber));
    }

    #[test]
    fn test_validate_text_alphanumeric() {
        assert!(InputFieldSystem::validate_text("abc123", &ContentType::Alphanumeric));
        assert!(!InputFieldSystem::validate_text("abc 123", &ContentType::Alphanumeric));
        assert!(!InputFieldSystem::validate_text("abc@123", &ContentType::Alphanumeric));
    }

    #[test]
    fn test_set_text() {
        let mut input_fields = HashMap::new();
        let mut texts = HashMap::new();
        
        input_fields.insert(1, create_test_input_field());
        
        InputFieldSystem::set_text(1, "Hello World".to_string(), &mut input_fields, &mut texts);
        
        let field = input_fields.get(&1).unwrap();
        assert_eq!(field.text, "Hello World");
        assert_eq!(field.caret_position, 11);
    }

    #[test]
    fn test_clear_focus() {
        let mut system = InputFieldSystem::new();
        let mut input_fields = HashMap::new();
        
        input_fields.insert(1, create_test_input_field());
        system.set_focus(1, &mut input_fields);
        
        assert!(input_fields.get(&1).unwrap().is_focused);
        
        system.clear_focus(&mut input_fields);
        
        assert!(!input_fields.get(&1).unwrap().is_focused);
        assert_eq!(system.get_focused_field(), None);
    }
}

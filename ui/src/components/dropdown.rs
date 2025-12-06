//! UIDropdown component

use serde::{Deserialize, Serialize};

/// Dropdown component for option selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIDropdown {
    /// Template entity (the dropdown list)
    pub template: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Caption text entity
    pub caption_text: Option<u64>,
    
    /// Item text entity (in template)
    pub item_text: Option<u64>,
    
    /// Options
    pub options: Vec<DropdownOption>,
    
    /// Current selected index
    pub value: i32,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

impl Default for UIDropdown {
    fn default() -> Self {
        Self {
            template: None,
            caption_text: None,
            item_text: None,
            options: Vec::new(),
            value: 0,
            on_value_changed: None,
        }
    }
}

/// Dropdown option
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DropdownOption {
    pub text: String,
    pub image: Option<String>,
}

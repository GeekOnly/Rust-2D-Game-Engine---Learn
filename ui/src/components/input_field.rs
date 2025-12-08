//! UIInputField component

use serde::{Deserialize, Serialize};
use crate::Color;

/// Input field component for text input
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIInputField {
    /// Text component entity
    pub text_component: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Placeholder entity
    pub placeholder: Option<u64>,
    
    /// Current text
    pub text: String,
    
    /// Character limit (0 = unlimited)
    pub character_limit: i32,
    
    /// Content type
    pub content_type: ContentType,
    
    /// Line type
    pub line_type: LineType,
    
    /// Input type (for mobile keyboards)
    pub input_type: InputType,
    
    /// Keyboard type (for mobile)
    pub keyboard_type: KeyboardType,
    
    /// Character validation
    pub character_validation: CharacterValidation,
    
    /// Caret blink rate
    pub caret_blink_rate: f32,
    
    /// Caret width
    pub caret_width: i32,
    
    /// Selection color
    pub selection_color: Color,
    
    /// Read only
    pub read_only: bool,
    
    /// Lua callbacks
    pub on_value_changed: Option<String>,
    pub on_end_edit: Option<String>,
    
    /// Runtime state
    #[serde(skip)]
    pub caret_position: i32,
    #[serde(skip)]
    pub selection_anchor: i32,
    #[serde(skip)]
    pub is_focused: bool,
}

impl Default for UIInputField {
    fn default() -> Self {
        Self {
            text_component: None,
            placeholder: None,
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
}

/// Content type for input validation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Standard,
    Autocorrected,
    IntegerNumber,
    DecimalNumber,
    Alphanumeric,
    Name,
    EmailAddress,
    Password,
    Pin,
    Custom,
}

/// Line type for input field
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    SingleLine,
    MultiLineSubmit,
    MultiLineNewline,
}

/// Input type for mobile keyboards
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    Standard,
    AutoCorrect,
    Password,
}

/// Keyboard type for mobile
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KeyboardType {
    Default,
    ASCIICapable,
    NumbersAndPunctuation,
    URL,
    NumberPad,
    PhonePad,
    NamePhonePad,
    EmailAddress,
}

/// Character validation mode
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CharacterValidation {
    None,
    Integer,
    Decimal,
    Alphanumeric,
    Name,
    EmailAddress,
}

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Visible {
    pub is_visible: bool,
}

impl Default for Visible {
    fn default() -> Self {
        Self {
            is_visible: true,
        }
    }
}

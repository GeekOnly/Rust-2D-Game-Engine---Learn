use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component holding raw data from LDtk for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdtkEntity {
    pub identifier: String,
    pub iid: String,
    pub width: i32,
    pub height: i32,
    pub tags: Vec<String>,
    pub fields: HashMap<String, serde_json::Value>,
}

impl Default for LdtkEntity {
    fn default() -> Self {
        Self {
            identifier: "Unknown".to_string(),
            iid: "".to_string(),
            width: 0,
            height: 0,
            tags: Vec::new(),
            fields: HashMap::new(),
        }
    }
}

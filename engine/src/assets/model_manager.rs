use std::collections::HashMap;
use std::sync::Arc;
use crate::assets::xsg::XsgFile;

pub struct ModelManager {
    models: HashMap<String, Arc<XsgFile>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn add_model(&mut self, id: String, model: XsgFile) {
        self.models.insert(id, Arc::new(model));
    }

    pub fn get_model(&self, id: &str) -> Option<Arc<XsgFile>> {
        self.models.get(id).cloned()
    }
    
    
    pub fn has_model(&self, id: &str) -> bool {
        self.models.contains_key(id)
    }
}

static mut MODEL_MANAGER: Option<ModelManager> = None;

pub fn get_model_manager() -> &'static mut ModelManager {
    unsafe {
        if MODEL_MANAGER.is_none() {
            MODEL_MANAGER = Some(ModelManager::new());
        }
        MODEL_MANAGER.as_mut().unwrap()
    }
}

use std::collections::HashMap;
use anyhow::Result;
use std::any::Any;

pub mod assets;
pub mod project;

pub trait EngineModule: Any {
    fn name(&self) -> &str;
    fn on_load(&mut self, _ctx: &mut EngineContext) -> Result<()> { Ok(()) }
    fn on_update(&mut self, _ctx: &mut EngineContext, _dt: f32) {}
    fn on_unload(&mut self, _ctx: &mut EngineContext) {}
    fn as_any(&mut self) -> &mut dyn Any;
}

use std::sync::Arc;
use crate::assets::AssetLoader;
use input::InputSystem;

pub struct EngineContext {
    pub modules: HashMap<String, Box<dyn EngineModule>>,
    pub should_quit: bool,
    pub input: InputSystem,
    pub asset_loader: Arc<dyn AssetLoader>,
}

impl EngineContext {
    pub fn new(asset_loader: Arc<dyn AssetLoader>) -> Self {
        Self { 
            modules: HashMap::new(), 
            should_quit: false,
            input: InputSystem::new(),
            asset_loader,
        }
    }

    pub fn register_module<M: EngineModule + 'static>(&mut self, mut module: M) {
        let name = module.name().to_string();
        // call on_load
        let _ = module.on_load(self);
        self.modules.insert(name, Box::new(module));
    }

    pub fn update(&mut self, dt: f32) {
        let keys: Vec<String> = self.modules.keys().cloned().collect();
        for key in keys {
            if let Some(mut m) = self.modules.remove(&key) {
                m.on_update(self, dt);
                self.modules.insert(key, m);
            }
        }
    }
}

pub struct Time {
    pub dt: f32,
    pub time: f32,
}

impl Time {
    pub fn new() -> Self {
        Self { dt: 0.0, time: 0.0 }
    }
}

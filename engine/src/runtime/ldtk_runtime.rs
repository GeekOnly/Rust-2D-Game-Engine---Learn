use ecs::World;
use ecs::loaders::LdtkHotReloader;
use log::{info, error};
use std::path::Path;

/// Runtime LDtk manager with hot-reload support
/// 
/// This provides a simple interface for games to use LDtk files
/// with automatic hot-reloading during development.
pub struct LdtkRuntime {
    reloader: LdtkHotReloader,
    enabled: bool,
}

impl LdtkRuntime {
    /// Create a new LDtk runtime
    pub fn new() -> Self {
        Self {
            reloader: LdtkHotReloader::new(),
            enabled: true,
        }
    }

    /// Load and watch an LDtk file
    /// 
    /// # Example
    /// ```no_run
    /// let mut ldtk = LdtkRuntime::new();
    /// ldtk.load("levels/world.ldtk", &mut world).unwrap();
    /// ```
    pub fn load(&mut self, path: impl AsRef<Path>, world: &mut World) -> Result<(), String> {
        let path = path.as_ref();
        info!("Loading LDtk file: {:?}", path);
        
        match self.reloader.watch(path, world) {
            Ok(entities) => {
                info!("Loaded {} entities from LDtk file", entities.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to load LDtk file: {}", e);
                Err(e)
            }
        }
    }

    /// Update - call this every frame to check for file changes
    /// Returns true if any files were reloaded
    pub fn update(&mut self, world: &mut World) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(entities) = self.reloader.check_updates(world) {
            info!("Hot-reloaded {} entities", entities.len());
            true
        } else {
            false
        }
    }

    /// Enable or disable hot-reload
    /// Useful for disabling in production builds
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!("LDtk hot-reload {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if hot-reload is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get list of watched files
    pub fn watched_files(&self) -> Vec<std::path::PathBuf> {
        self.reloader.watched_files()
    }

    /// Stop watching a specific file
    pub fn unload(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        self.reloader.unwatch(path)
    }
}

impl Default for LdtkRuntime {
    fn default() -> Self {
        Self::new()
    }
}

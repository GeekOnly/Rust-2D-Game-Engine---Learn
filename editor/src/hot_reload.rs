use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Hot-reload watcher for LDtk files
pub struct HotReloadWatcher {
    /// File watcher with debouncing
    _debouncer: Debouncer<notify::RecommendedWatcher, FileIdMap>,
    
    /// Receiver for file change events
    receiver: Receiver<PathBuf>,
    
    /// Watched paths
    watched_paths: Vec<PathBuf>,
}

impl HotReloadWatcher {
    /// Create a new hot-reload watcher
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = channel();
        
        // Create debouncer with 500ms delay to handle rapid file changes
        let debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        for event in events {
                            if let Some(path) = Self::extract_ldtk_path(&event.event) {
                                // Send the path through the channel
                                let _ = tx.send(path);
                            }
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            log::error!("File watcher error: {:?}", error);
                        }
                    }
                }
            },
        ).map_err(|e| format!("Failed to create file watcher: {}", e))?;
        
        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
            watched_paths: Vec::new(),
        })
    }
    
    /// Extract .ldtk file path from event if applicable
    fn extract_ldtk_path(event: &Event) -> Option<PathBuf> {
        // Check if this is a relevant event type
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                // Check if any path in the event is an .ldtk file
                for path in &event.paths {
                    if path.extension().and_then(|s| s.to_str()) == Some("ldtk") {
                        return Some(path.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    /// Watch a specific file
    pub fn watch_file(&mut self, path: &Path) -> Result<(), String> {
        if !self.watched_paths.contains(&path.to_path_buf()) {
            self._debouncer
                .watcher()
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch file {:?}: {}", path, e))?;
            
            self.watched_paths.push(path.to_path_buf());
            log::info!("Watching file for hot-reload: {:?}", path);
        }
        Ok(())
    }
    
    /// Watch a directory recursively
    pub fn watch_directory(&mut self, path: &Path) -> Result<(), String> {
        if !self.watched_paths.contains(&path.to_path_buf()) {
            self._debouncer
                .watcher()
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch directory {:?}: {}", path, e))?;
            
            self.watched_paths.push(path.to_path_buf());
            log::info!("Watching directory for hot-reload: {:?}", path);
        }
        Ok(())
    }
    
    /// Unwatch a file or directory
    pub fn unwatch(&mut self, path: &Path) -> Result<(), String> {
        if let Some(pos) = self.watched_paths.iter().position(|p| p == path) {
            self._debouncer
                .watcher()
                .unwatch(path)
                .map_err(|e| format!("Failed to unwatch {:?}: {}", path, e))?;
            
            self.watched_paths.remove(pos);
            log::info!("Stopped watching: {:?}", path);
        }
        Ok(())
    }
    
    /// Poll for changed files (non-blocking)
    pub fn poll_changes(&self) -> Vec<PathBuf> {
        let mut changes = Vec::new();
        
        // Drain all pending events
        while let Ok(path) = self.receiver.try_recv() {
            changes.push(path);
        }
        
        changes
    }
    
    /// Check if a path is being watched
    pub fn is_watching(&self, path: &Path) -> bool {
        self.watched_paths.contains(&path.to_path_buf())
    }
    
    /// Get all watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

impl Default for HotReloadWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create hot-reload watcher")
    }
}

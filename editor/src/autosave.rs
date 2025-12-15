/// Auto-save system for the editor
use std::time::{Duration, Instant};
use std::path::PathBuf;

pub struct AutoSave {
    /// Last time the scene was saved
    last_save: Instant,
    
    /// Auto-save interval in seconds
    interval: Duration,
    
    /// Whether auto-save is enabled
    enabled: bool,
    
    /// Number of backup files to keep
    backup_count: usize,
    
    /// Last auto-save path
    last_autosave_path: Option<PathBuf>,
}

impl AutoSave {
    /// Create new auto-save system
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            last_save: Instant::now(),
            interval: Duration::from_secs(interval_seconds),
            enabled: true,
            backup_count: 5,
            last_autosave_path: None,
        }
    }
    
    /// Check if it's time to auto-save
    pub fn should_save(&self) -> bool {
        self.enabled && self.last_save.elapsed() >= self.interval
    }
    
    /// Mark that a save was performed
    pub fn mark_saved(&mut self) {
        self.last_save = Instant::now();
    }
    
    /// Reset the timer (e.g., after manual save)
    pub fn reset(&mut self) {
        self.last_save = Instant::now();
    }
    
    /// Get time until next auto-save
    pub fn time_until_next_save(&self) -> Duration {
        let elapsed = self.last_save.elapsed();
        if elapsed >= self.interval {
            Duration::from_secs(0)
        } else {
            self.interval - elapsed
        }
    }
    
    /// Get time since last save
    pub fn time_since_last_save(&self) -> Duration {
        self.last_save.elapsed()
    }
    
    /// Enable/disable auto-save
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if auto-save is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set auto-save interval
    pub fn set_interval(&mut self, seconds: u64) {
        self.interval = Duration::from_secs(seconds);
    }
    
    /// Get auto-save interval in seconds
    pub fn interval_seconds(&self) -> u64 {
        self.interval.as_secs()
    }
    
    /// Create auto-save file path
    pub fn create_autosave_path(&mut self, scene_path: &PathBuf) -> PathBuf {
        let parent = scene_path.parent().unwrap_or(scene_path.as_ref());
        let filename = scene_path.file_stem().unwrap_or_default();
        let extension = scene_path.extension().unwrap_or_default();
        
        // Create autosave filename: scene_autosave_timestamp.json
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let autosave_filename = format!(
            "{}~autosave_{}.{}",
            filename.to_string_lossy(),
            timestamp,
            extension.to_string_lossy()
        );
        
        let autosave_path = parent.join(autosave_filename);
        self.last_autosave_path = Some(autosave_path.clone());
        autosave_path
    }
    
    /// Clean up old auto-save files
    pub fn cleanup_old_autosaves(&self, scene_path: &PathBuf) -> std::io::Result<()> {
        let parent = scene_path.parent().unwrap_or(scene_path.as_ref());
        let filename = scene_path.file_stem().unwrap_or_default();
        
        // Find all autosave files for this scene
        let mut autosave_files: Vec<PathBuf> = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&format!("{}~autosave_", filename.to_string_lossy())) {
                        autosave_files.push(entry.path());
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        autosave_files.sort_by(|a, b| {
            let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });
        
        // Keep only the newest N files
        for old_file in autosave_files.iter().skip(self.backup_count) {
            let _ = std::fs::remove_file(old_file);
        }
        
        Ok(())
    }
    
    /// Get list of available auto-save files
    pub fn get_autosave_files(&self, scene_path: &PathBuf) -> Vec<PathBuf> {
        let parent = scene_path.parent().unwrap_or(scene_path.as_ref());
        let filename = scene_path.file_stem().unwrap_or_default();
        
        let mut autosave_files: Vec<PathBuf> = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&format!("{}~autosave_", filename.to_string_lossy())) {
                        autosave_files.push(entry.path());
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        autosave_files.sort_by(|a, b| {
            let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });
        
        autosave_files
    }
}

impl Default for AutoSave {
    fn default() -> Self {
        Self::new(300) // 5 minutes default
    }
}

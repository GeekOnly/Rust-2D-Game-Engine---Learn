/// Unity/Unreal-like Asset Manager
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    Scene,
    Sprite,
    SpriteSheet,  // .sprite files
    Script,
    Prefab,
    Audio,
    Font,
    Folder,
    Unknown,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "json" => Self::Scene,
            "png" | "jpg" | "jpeg" | "bmp" | "gif" => Self::Sprite,
            "sprite" => Self::SpriteSheet,
            "lua" => Self::Script,
            "prefab" => Self::Prefab,
            "wav" | "mp3" | "ogg" => Self::Audio,
            "ttf" | "otf" => Self::Font,
            _ => Self::Unknown,
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Scene => "🎬",
            Self::Sprite => "🖼️",
            Self::SpriteSheet => "🎨",
            Self::Script => "📜",
            Self::Prefab => "📦",
            Self::Audio => "🔊",
            Self::Font => "🔤",
            Self::Folder => "📁",
            Self::Unknown => "📄",
        }
    }
    
    pub fn color(&self) -> [u8; 3] {
        match self {
            Self::Scene => [100, 150, 255],    // Blue
            Self::Sprite => [255, 150, 100],   // Orange
            Self::SpriteSheet => [255, 100, 200], // Pink/Magenta
            Self::Script => [150, 255, 150],   // Green
            Self::Prefab => [255, 200, 100],   // Yellow
            Self::Audio => [200, 100, 255],    // Purple
            Self::Font => [255, 100, 150],     // Pink
            Self::Folder => [150, 150, 150],   // Gray
            Self::Unknown => [100, 100, 100],  // Dark gray
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub path: PathBuf,
    pub name: String,
    pub asset_type: AssetType,
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub is_favorite: bool,
    pub labels: Vec<String>,
    pub thumbnail: Option<PathBuf>,
}

impl AssetMetadata {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let asset_type = if metadata.is_dir() {
            AssetType::Folder
        } else {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            AssetType::from_extension(ext)
        };
        
        Ok(Self {
            path: path.to_path_buf(),
            name,
            asset_type,
            size: metadata.len(),
            modified: metadata.modified()?,
            is_favorite: false,
            labels: Vec::new(),
            thumbnail: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Name,
    Type,
    Size,
    Modified,
}

pub struct AssetManager {
    /// Current folder path
    pub current_path: PathBuf,
    
    /// View mode (Grid or List)
    pub view_mode: ViewMode,
    
    /// Sort mode
    pub sort_mode: SortMode,
    
    /// Search query
    pub search_query: String,
    
    /// Selected asset
    pub selected_asset: Option<PathBuf>,
    
    /// Asset metadata cache
    metadata_cache: HashMap<PathBuf, AssetMetadata>,
    
    /// Favorites
    favorites: Vec<PathBuf>,
    
    /// Show hidden files
    pub show_hidden: bool,
    
    /// Thumbnail size
    pub thumbnail_size: f32,
    
    /// Navigation history
    history: Vec<PathBuf>,
    history_index: usize,
}

impl AssetManager {
    pub fn new(project_path: &Path) -> Self {
        // Start at project root, not assets folder
        // This allows users to see all project files
        let start_path = project_path.to_path_buf();
        
        Self {
            current_path: start_path.clone(),
            view_mode: ViewMode::Grid,
            sort_mode: SortMode::Name,
            search_query: String::new(),
            selected_asset: None,
            metadata_cache: HashMap::new(),
            favorites: Vec::new(),
            show_hidden: false,
            thumbnail_size: 80.0,
            history: vec![start_path],
            history_index: 0,
        }
    }
    
    /// Get assets in current folder
    pub fn get_assets(&mut self) -> Vec<AssetMetadata> {
        let mut assets = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.current_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip hidden files
                if !self.show_hidden {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name.starts_with('~') {
                            continue;
                        }
                    }
                }
                
                // Get or create metadata
                let metadata = self.metadata_cache.entry(path.clone())
                    .or_insert_with(|| AssetMetadata::from_path(&path).unwrap_or_else(|_| {
                        AssetMetadata {
                            path: path.clone(),
                            name: "Error".to_string(),
                            asset_type: AssetType::Unknown,
                            size: 0,
                            modified: std::time::SystemTime::now(),
                            is_favorite: false,
                            labels: Vec::new(),
                            thumbnail: None,
                        }
                    }));
                
                // Apply search filter
                if !self.search_query.is_empty() {
                    if !metadata.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        continue;
                    }
                }
                
                assets.push(metadata.clone());
            }
        }
        
        // Sort assets
        self.sort_assets(&mut assets);
        
        assets
    }
    
    /// Sort assets
    fn sort_assets(&self, assets: &mut Vec<AssetMetadata>) {
        // Folders first
        assets.sort_by(|a, b| {
            match (&a.asset_type, &b.asset_type) {
                (AssetType::Folder, AssetType::Folder) => std::cmp::Ordering::Equal,
                (AssetType::Folder, _) => std::cmp::Ordering::Less,
                (_, AssetType::Folder) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
        
        // Then by sort mode
        match self.sort_mode {
            SortMode::Name => {
                assets.sort_by(|a, b| {
                    if a.asset_type == AssetType::Folder && b.asset_type == AssetType::Folder {
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    } else if a.asset_type != AssetType::Folder && b.asset_type != AssetType::Folder {
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            SortMode::Type => {
                assets.sort_by(|a, b| {
                    if a.asset_type == AssetType::Folder && b.asset_type == AssetType::Folder {
                        std::cmp::Ordering::Equal
                    } else if a.asset_type != AssetType::Folder && b.asset_type != AssetType::Folder {
                        format!("{:?}", a.asset_type).cmp(&format!("{:?}", b.asset_type))
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            SortMode::Size => {
                assets.sort_by(|a, b| {
                    if a.asset_type == AssetType::Folder && b.asset_type == AssetType::Folder {
                        std::cmp::Ordering::Equal
                    } else if a.asset_type != AssetType::Folder && b.asset_type != AssetType::Folder {
                        b.size.cmp(&a.size)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            SortMode::Modified => {
                assets.sort_by(|a, b| {
                    if a.asset_type == AssetType::Folder && b.asset_type == AssetType::Folder {
                        b.modified.cmp(&a.modified)
                    } else if a.asset_type != AssetType::Folder && b.asset_type != AssetType::Folder {
                        b.modified.cmp(&a.modified)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
        }
    }
    
    /// Navigate to folder
    pub fn navigate_to(&mut self, path: &Path) {
        if path.is_dir() {
            self.current_path = path.to_path_buf();
            
            // Add to history
            if self.history_index < self.history.len() - 1 {
                self.history.truncate(self.history_index + 1);
            }
            self.history.push(path.to_path_buf());
            self.history_index = self.history.len() - 1;
        }
    }
    
    /// Navigate up one level
    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            self.navigate_to(&parent);
        }
    }
    
    /// Navigate back
    pub fn navigate_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }
    
    /// Navigate forward
    pub fn navigate_forward(&mut self) -> bool {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_path = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }
    
    /// Toggle favorite
    pub fn toggle_favorite(&mut self, path: &Path) {
        if let Some(metadata) = self.metadata_cache.get_mut(path) {
            metadata.is_favorite = !metadata.is_favorite;
            
            if metadata.is_favorite {
                if !self.favorites.contains(&path.to_path_buf()) {
                    self.favorites.push(path.to_path_buf());
                }
            } else {
                self.favorites.retain(|p| p != path);
            }
        }
    }
    
    /// Check if asset is favorite
    pub fn is_favorite(&self, path: &Path) -> bool {
        self.favorites.contains(&path.to_path_buf())
    }
    
    /// Get breadcrumb path
    pub fn get_breadcrumbs(&self) -> Vec<(String, PathBuf)> {
        let mut breadcrumbs = Vec::new();
        let mut current = self.current_path.clone();
        
        loop {
            if let Some(name) = current.file_name().and_then(|n| n.to_str()) {
                breadcrumbs.insert(0, (name.to_string(), current.clone()));
            }
            
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        breadcrumbs
    }
    
    /// Format file size
    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        
        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}

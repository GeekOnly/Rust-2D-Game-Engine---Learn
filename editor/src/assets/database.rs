use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;
use engine::assets::core::AssetId;
use engine::assets::metadata::AssetMetadata;

/// The AssetDatabase tracks all assets in the project (editor-side).
/// It maintains a mapping between File Paths and AssetIds.
pub struct AssetDatabase {
    /// Map from AssetId to absolute File Path
    id_to_path: HashMap<AssetId, PathBuf>,
    /// Map from absolute File Path to AssetId
    path_to_id: HashMap<PathBuf, AssetId>,
    /// Project root directory
    project_root: PathBuf,
}

impl AssetDatabase {
    pub fn new(project_root: PathBuf) -> Self {
        let mut db = Self {
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
            project_root,
        };
        db.refresh();
        db
    }

    /// Refresh the database by scanning the assets directory.
    pub fn refresh(&mut self) {
        let assets_dir = self.project_root.join("assets");
        if !assets_dir.exists() {
            log::warn!("Assets directory not found: {:?}", assets_dir);
            return;
        }

        self.id_to_path.clear();
        self.path_to_id.clear();

        for entry in WalkDir::new(&assets_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            // Skip .meta files themselves
            if path.extension().map_or(false, |ext| ext == "meta") {
                continue;
            }

            self.process_asset(path);
        }
    }

    /// Process a single asset file: ensure it has a .meta file and register it.
    fn process_asset(&mut self, path: &Path) {
        // 1. Determine expected meta path
        let meta_path = AssetMetadata::get_meta_path(path);

        // 2. Load or Create Metadata
        let metadata = if meta_path.exists() {
            match AssetMetadata::load_from_file(&meta_path) {
                Ok(meta) => meta,
                Err(e) => {
                    log::error!("Failed to load meta for {:?}: {}", path, e);
                    // Fallback: regenerate? Or skip? For safety, let's skip or warn.
                    return;
                }
            }
        } else {
            // New asset! Generate meta
            let meta = AssetMetadata::generate(path);
            if let Err(e) = meta.save_to_file(&meta_path) {
                log::error!("Failed to save meta for {:?}: {}", path, e);
                return;
            }
            log::info!("Generated meta for new asset: {:?}", path);
            meta
        };

        // 3. Register in Maps
        let abs_path = path.to_path_buf(); // normalized?
        self.id_to_path.insert(metadata.id, abs_path.clone());
        self.path_to_id.insert(abs_path, metadata.id);
    }

    pub fn get_path(&self, id: AssetId) -> Option<&PathBuf> {
        self.id_to_path.get(&id)
    }

    pub fn get_id(&self, path: &Path) -> Option<AssetId> {
        self.path_to_id.get(path).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;

    #[test]
    fn test_database_refresh() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        let assets_dir = project_root.join("assets");
        std::fs::create_dir(&assets_dir).unwrap();

        // Create a test file
        let file_path = assets_dir.join("test.png");
        File::create(&file_path).unwrap();

        let mut db = AssetDatabase::new(project_root.clone());
        
        // Should have generated a meta file and registered ID
        assert!(db.get_id(&file_path).is_some());
        let id = db.get_id(&file_path).unwrap();
        
        let meta_path = AssetMetadata::get_meta_path(&file_path);
        assert!(meta_path.exists());
        
        // Check reverse lookup
        assert_eq!(db.get_path(id), Some(&file_path));
    }
}

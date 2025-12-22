use engine_core::assets::AssetLoader;
use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use async_trait::async_trait;

pub struct NativeAssetLoader {
    base_path: PathBuf,
}

impl NativeAssetLoader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl AssetLoader for NativeAssetLoader {
    async fn load_text(&self, path: &str) -> Result<String> {
        let full_path = self.base_path.join(path);
        fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to load text asset from {:?}", full_path))
    }

    async fn load_binary(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.base_path.join(path);
        fs::read(&full_path)
            .with_context(|| format!("Failed to load binary asset from {:?}", full_path))
    }

    fn get_base_path(&self) -> String {
        self.base_path.to_string_lossy().to_string()
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use egui::{ColorImage, TextureHandle};

pub struct TextureManager {
    textures: HashMap<String, TextureHandle>,
    base_path: Option<PathBuf>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            base_path: None,
        }
    }

    pub fn set_base_path(&mut self, path: PathBuf) {
        self.base_path = Some(path);
    }

    pub fn get_base_path(&self) -> Option<&PathBuf> {
        self.base_path.as_ref()
    }

    /// Load texture with explicit path (no base path prepending)
    /// Useful when the path is already absolute or needs custom handling
    pub fn load_texture_absolute(&mut self, ctx: &egui::Context, texture_id: &str, full_path: &Path) -> Option<&TextureHandle> {
        // Check if already loaded
        if self.textures.contains_key(texture_id) {
            return self.textures.get(texture_id);
        }

        // Load image directly from the provided path
        log::info!("Loading texture (absolute): {} from {}", texture_id, full_path.display());
        match image::open(full_path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();
                
                log::info!("Texture loaded successfully: {}x{}", size[0], size[1]);
                
                let color_image = ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                );

                let texture = ctx.load_texture(
                    texture_id,
                    color_image,
                    egui::TextureOptions::LINEAR,
                );

                self.textures.insert(texture_id.to_string(), texture);
                self.textures.get(texture_id)
            }
            Err(e) => {
                log::error!("Failed to load texture {}: {}", full_path.display(), e);
                None
            }
        }
    }

    pub fn load_texture(&mut self, ctx: &egui::Context, texture_id: &str, path: &Path) -> Option<&TextureHandle> {
        // Check if already loaded
        if self.textures.contains_key(texture_id) {
            return self.textures.get(texture_id);
        }

        // Skip .sprite metadata files (they are not images)
        if let Some(ext) = path.extension() {
            if ext == "sprite" {
                log::debug!("Skipping .sprite metadata file: {}", path.display());
                return None;
            }
        }

        // Resolve full path
        let full_path = if let Some(base) = &self.base_path {
            base.join(path)
        } else {
            path.to_path_buf()
        };

        // Load image
        log::info!("Loading texture: {} from {}", texture_id, full_path.display());
        match image::open(&full_path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();
                
                log::info!("Texture loaded successfully: {}x{}", size[0], size[1]);
                
                let color_image = ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                );

                let texture = ctx.load_texture(
                    texture_id,
                    color_image,
                    egui::TextureOptions::LINEAR,
                );

                self.textures.insert(texture_id.to_string(), texture);
                self.textures.get(texture_id)
            }
            Err(e) => {
                log::error!("Failed to load texture {}: {}", full_path.display(), e);
                None
            }
        }
    }

    pub fn get_texture(&self, texture_id: &str) -> Option<&TextureHandle> {
        self.textures.get(texture_id)
    }
}

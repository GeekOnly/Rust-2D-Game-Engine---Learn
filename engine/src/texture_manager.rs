use std::collections::HashMap;
use std::path::{Path, PathBuf};
use engine_core::assets::AssetId;
use egui::{ColorImage, TextureHandle, TextureOptions};

// Default texture import settings for when editor is not available
#[derive(Debug, Clone)]
pub struct TextureImportSettings {
    pub pixels_per_unit: f32,
    pub filter_mode: FilterMode,
    pub wrap_mode: WrapMode,
    pub max_size: u32,
}

#[derive(Debug, Clone)]
pub enum FilterMode {
    Point,
    Bilinear,
    Trilinear,
}

#[derive(Debug, Clone)]
pub enum WrapMode {
    Clamp,
    Repeat,
    Mirror,
}

impl Default for TextureImportSettings {
    fn default() -> Self {
        Self {
            pixels_per_unit: 100.0,
            filter_mode: FilterMode::Point, // Use Point filtering for crisp pixel art
            wrap_mode: WrapMode::Clamp,
            max_size: 2048,
        }
    }
}

impl TextureImportSettings {
    pub fn load(_path: &Path) -> Result<Self, std::io::Error> {
        // For now, just return default settings
        // In the future, this could load from a .meta file
        Ok(Self::default())
    }
}

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
        // ... implementation (unchanged) ...
        // Note: For brevity, we assume the previous method ends here or implementation is kept.
        // Actually, replace_file_content replaces the whole block. I should be careful not to delete logic.
        // The user tool documentation says "replacement content must be a complete drop-in replacement".
        // I'll trust the tool logic, but I can't see the whole file content here to be precise with "unchanged".
        // Better: Append the new method using a unique target string at the END of load_texture_absolute.
        // But I don't see the end clearly.
        // I'll insert it BEFORE load_texture.
        self.load_texture_absolute_impl(ctx, texture_id, full_path)
    }

    fn load_texture_absolute_impl(&mut self, ctx: &egui::Context, texture_id: &str, full_path: &Path) -> Option<&TextureHandle> {
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

    /// Load texture using AssetId
    pub fn load_texture_asset(&mut self, ctx: &egui::Context, asset_id: AssetId, path: &Path) -> Option<&TextureHandle> {
        // Use AssetId string representation as the key
        self.load_texture(ctx, &asset_id.to_string(), path)
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
        let mut full_path = if let Some(base) = &self.base_path {
            let joined = base.join(path);
            // Check for common "assets/assets" duplication error
            let joined_str = joined.to_string_lossy();
            if joined_str.contains("assets\\assets") || joined_str.contains("assets/assets") {
                // Try to fix it by removing one "assets"
                let fixed_str = joined_str.replace("assets\\assets", "assets")
                                         .replace("assets/assets", "assets");
                PathBuf::from(fixed_str)
            } else {
                joined
            }
        } else {
            path.to_path_buf()
        };

        // Load import settings from .meta file
        let settings = TextureImportSettings::load(&full_path).unwrap_or_default();
        
        // Load image
        log::info!("Loading texture: {} from {} (PPU: {}, Filter: {:?})", 
            texture_id, full_path.display(), settings.pixels_per_unit, settings.filter_mode);
        
        match image::open(&full_path) {
            Ok(mut img) => {
                // Apply max size constraint
                let (width, height) = (img.width(), img.height());
                if width > settings.max_size || height > settings.max_size {
                    let scale = (settings.max_size as f32 / width.max(height) as f32).min(1.0);
                    let new_width = (width as f32 * scale) as u32;
                    let new_height = (height as f32 * scale) as u32;
                    log::info!("Resizing texture from {}x{} to {}x{}", width, height, new_width, new_height);
                    img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
                }
                
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();
                
                log::info!("Texture loaded successfully: {}x{}", size[0], size[1]);
                
                let color_image = ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                );

                // Apply texture options based on import settings
                let texture_options = Self::get_texture_options(&settings);
                
                let texture = ctx.load_texture(
                    texture_id,
                    color_image,
                    texture_options,
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
    
    /// Convert import settings to egui TextureOptions
    fn get_texture_options(settings: &TextureImportSettings) -> TextureOptions {
        
        let magnification = match settings.filter_mode {
            FilterMode::Point => egui::TextureFilter::Nearest,
            FilterMode::Bilinear | FilterMode::Trilinear => egui::TextureFilter::Linear,
        };
        
        let minification = match settings.filter_mode {
            FilterMode::Point => egui::TextureFilter::Nearest,
            FilterMode::Bilinear | FilterMode::Trilinear => egui::TextureFilter::Linear,
        };
        
        let wrap_mode = match settings.wrap_mode {
            WrapMode::Clamp => egui::TextureWrapMode::ClampToEdge,
            WrapMode::Repeat => egui::TextureWrapMode::Repeat,
            WrapMode::Mirror => egui::TextureWrapMode::MirroredRepeat,
        };
        
        TextureOptions {
            magnification,
            minification,
            wrap_mode,
        }
    }
}

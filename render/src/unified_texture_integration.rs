//! Unified Texture Integration System
//!
//! This module provides integration between the legacy TextureManager and the new UnifiedTextureManager,
//! ensuring sprites and tilemaps use the same WGPU texture management with consistent texture update pipeline.

use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use crate::{
    texture::{Texture, TextureManager},
    unified_texture_manager::{
        UnifiedTextureManager, UnifiedTexture, UnifiedTextureDescriptor, 
        UnifiedTextureFormat, TextureUsagePattern, UnifiedTextureFilter
    }
};

/// Texture loading strategy for unified management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureLoadingStrategy {
    /// Use legacy TextureManager for backward compatibility
    Legacy,
    /// Use UnifiedTextureManager for new features
    Unified,
    /// Automatically choose based on usage pattern
    Auto,
}

/// Unified texture loading configuration
#[derive(Debug, Clone)]
pub struct UnifiedTextureConfig {
    /// Loading strategy
    pub strategy: TextureLoadingStrategy,
    /// Default format for sprites
    pub sprite_format: UnifiedTextureFormat,
    /// Default format for tilemaps
    pub tilemap_format: UnifiedTextureFormat,
    /// Default filtering for pixel perfect rendering
    pub pixel_perfect_filter: UnifiedTextureFilter,
    /// Default filtering for smooth rendering
    pub smooth_filter: UnifiedTextureFilter,
    /// Enable automatic mipmap generation
    pub enable_mipmaps: bool,
    /// Memory priority for different texture types
    pub sprite_memory_priority: u8,
    pub tilemap_memory_priority: u8,
}

impl Default for UnifiedTextureConfig {
    fn default() -> Self {
        Self {
            strategy: TextureLoadingStrategy::Auto,
            sprite_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            tilemap_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            pixel_perfect_filter: UnifiedTextureFilter::Nearest,
            smooth_filter: UnifiedTextureFilter::Linear,
            enable_mipmaps: false, // Disabled for pixel perfect rendering
            sprite_memory_priority: 128,
            tilemap_memory_priority: 192, // Higher priority for tilemaps
        }
    }
}

/// Unified texture reference that can point to either legacy or unified textures
#[derive(Debug)]
pub enum UnifiedTextureRef<'a> {
    Legacy(&'a Texture),
    Unified(&'a UnifiedTexture),
}

impl<'a> UnifiedTextureRef<'a> {
    /// Get the bind group for rendering
    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        match self {
            UnifiedTextureRef::Legacy(texture) => texture.bind_group.as_ref(),
            UnifiedTextureRef::Unified(texture) => Some(&texture.bind_group),
        }
    }

    /// Get texture dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self {
            UnifiedTextureRef::Legacy(texture) => (texture.width, texture.height),
            UnifiedTextureRef::Unified(texture) => texture.descriptor.size,
        }
    }

    /// Check if texture supports filtering
    pub fn supports_filtering(&self) -> bool {
        match self {
            UnifiedTextureRef::Legacy(_) => true, // Legacy textures always support filtering
            UnifiedTextureRef::Unified(texture) => texture.descriptor.format.supports_filtering(),
        }
    }
}

/// Integrated texture management system
pub struct UnifiedTextureIntegration {
    /// Configuration for texture loading
    config: UnifiedTextureConfig,
    /// Mapping from texture IDs to loading strategy used
    texture_strategies: HashMap<String, TextureLoadingStrategy>,
}

impl UnifiedTextureIntegration {
    /// Create a new unified texture integration system
    pub fn new(config: Option<UnifiedTextureConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            texture_strategies: HashMap::new(),
        }
    }

    /// Load texture for sprite rendering
    pub fn load_sprite_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        legacy_manager: &mut TextureManager,
        unified_manager: &mut UnifiedTextureManager,
        texture_id: &str,
        path: impl AsRef<Path>,
        pixel_perfect: bool,
    ) -> Result<TextureLoadingStrategy> {
        let strategy = self.determine_loading_strategy(TextureUsagePattern::Static, pixel_perfect);
        
        match strategy {
            TextureLoadingStrategy::Legacy => {
                legacy_manager.load_texture(device, queue, path, texture_id)?;
            }
            TextureLoadingStrategy::Unified => {
                let descriptor = UnifiedTextureDescriptor {
                    id: texture_id.to_string(),
                    format: self.config.sprite_format,
                    usage_pattern: TextureUsagePattern::Static,
                    filter: if pixel_perfect {
                        self.config.pixel_perfect_filter
                    } else {
                        self.config.smooth_filter
                    },
                    generate_mipmaps: self.config.enable_mipmaps && !pixel_perfect,
                    memory_priority: self.config.sprite_memory_priority,
                    ..Default::default()
                };
                
                unified_manager.load_texture_from_file(device, queue, path, descriptor)?;
            }
            TextureLoadingStrategy::Auto => {
                // For auto strategy, prefer unified for pixel perfect, legacy for smooth
                let actual_strategy = if pixel_perfect {
                    TextureLoadingStrategy::Unified
                } else {
                    TextureLoadingStrategy::Legacy
                };
                
                return self.load_sprite_texture(
                    device, queue, legacy_manager, unified_manager,
                    texture_id, path, pixel_perfect
                );
            }
        }
        
        self.texture_strategies.insert(texture_id.to_string(), strategy);
        Ok(strategy)
    }

    /// Load texture for tilemap rendering
    pub fn load_tilemap_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        legacy_manager: &mut TextureManager,
        unified_manager: &mut UnifiedTextureManager,
        texture_id: &str,
        path: impl AsRef<Path>,
        pixel_perfect: bool,
    ) -> Result<TextureLoadingStrategy> {
        let strategy = self.determine_loading_strategy(TextureUsagePattern::Array, pixel_perfect);
        
        match strategy {
            TextureLoadingStrategy::Legacy => {
                legacy_manager.load_texture(device, queue, path, texture_id)?;
            }
            TextureLoadingStrategy::Unified => {
                let descriptor = UnifiedTextureDescriptor {
                    id: texture_id.to_string(),
                    format: self.config.tilemap_format,
                    usage_pattern: TextureUsagePattern::Array,
                    filter: if pixel_perfect {
                        self.config.pixel_perfect_filter
                    } else {
                        self.config.smooth_filter
                    },
                    generate_mipmaps: self.config.enable_mipmaps && !pixel_perfect,
                    memory_priority: self.config.tilemap_memory_priority,
                    ..Default::default()
                };
                
                unified_manager.load_texture_from_file(device, queue, path, descriptor)?;
            }
            TextureLoadingStrategy::Auto => {
                // For tilemaps, prefer unified for better array texture support
                return self.load_tilemap_texture(
                    device, queue, legacy_manager, unified_manager,
                    texture_id, path.as_ref(), pixel_perfect
                );
            }
        }
        
        self.texture_strategies.insert(texture_id.to_string(), strategy);
        Ok(strategy)
    }

    /// Create array texture for tilemap rendering (unified only)
    pub fn create_tilemap_array_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_manager: &mut UnifiedTextureManager,
        texture_id: &str,
        images: &[image::DynamicImage],
        pixel_perfect: bool,
    ) -> Result<()> {
        let descriptor = UnifiedTextureDescriptor {
            id: texture_id.to_string(),
            format: self.config.tilemap_format,
            usage_pattern: TextureUsagePattern::Array,
            array_layers: images.len() as u32,
            filter: if pixel_perfect {
                self.config.pixel_perfect_filter
            } else {
                self.config.smooth_filter
            },
            generate_mipmaps: self.config.enable_mipmaps && !pixel_perfect,
            memory_priority: self.config.tilemap_memory_priority,
            ..Default::default()
        };
        
        unified_manager.create_array_texture(device, queue, images, descriptor)?;
        self.texture_strategies.insert(texture_id.to_string(), TextureLoadingStrategy::Unified);
        Ok(())
    }

    /// Get texture reference from either manager
    pub fn get_texture<'a>(
        &self,
        legacy_manager: &'a TextureManager,
        unified_manager: &'a mut UnifiedTextureManager,
        texture_id: &str,
    ) -> Option<UnifiedTextureRef<'a>> {
        // Check which strategy was used for this texture
        if let Some(&strategy) = self.texture_strategies.get(texture_id) {
            match strategy {
                TextureLoadingStrategy::Legacy => {
                    legacy_manager.get_texture(texture_id).map(UnifiedTextureRef::Legacy)
                }
                TextureLoadingStrategy::Unified => {
                    unified_manager.get_texture(texture_id).map(UnifiedTextureRef::Unified)
                }
                TextureLoadingStrategy::Auto => {
                    // Try unified first, then legacy
                    if let Some(texture) = unified_manager.get_texture(texture_id) {
                        Some(UnifiedTextureRef::Unified(texture))
                    } else {
                        legacy_manager.get_texture(texture_id).map(UnifiedTextureRef::Legacy)
                    }
                }
            }
        } else {
            // Try both managers if strategy is unknown
            if let Some(texture) = unified_manager.get_texture(texture_id) {
                Some(UnifiedTextureRef::Unified(texture))
            } else {
                legacy_manager.get_texture(texture_id).map(UnifiedTextureRef::Legacy)
            }
        }
    }

    /// Get default texture (creates if not exists)
    pub fn get_default_texture<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_manager: &'a mut UnifiedTextureManager,
        default_type: &str,
    ) -> Result<()> {
        // Always use unified manager for default textures for consistency
        unified_manager.get_default_texture(device, queue, default_type)
    }

    /// Get default texture reference after ensuring it exists
    pub fn get_default_texture_ref<'a>(
        &self,
        unified_manager: &'a mut UnifiedTextureManager,
        default_type: &str,
    ) -> Option<UnifiedTextureRef<'a>> {
        let texture_id = unified_manager.get_default_texture_id(default_type);
        unified_manager.get_texture(&texture_id).map(UnifiedTextureRef::Unified)
    }

    /// Update texture data (supports both managers)
    pub fn update_texture_data(
        &self,
        queue: &wgpu::Queue,
        unified_manager: &mut UnifiedTextureManager,
        texture_id: &str,
        data: &[u8],
        layer: u32,
    ) -> Result<()> {
        if let Some(&strategy) = self.texture_strategies.get(texture_id) {
            match strategy {
                TextureLoadingStrategy::Legacy => {
                    // Legacy manager doesn't support data updates
                    anyhow::bail!("Legacy texture manager doesn't support data updates");
                }
                TextureLoadingStrategy::Unified => {
                    unified_manager.update_texture_data(queue, texture_id, data, layer)?;
                }
                TextureLoadingStrategy::Auto => {
                    // Try unified first
                    unified_manager.update_texture_data(queue, texture_id, data, layer)?;
                }
            }
        } else {
            anyhow::bail!("Unknown texture: {}", texture_id);
        }
        
        Ok(())
    }

    /// Unload texture from appropriate manager
    pub fn unload_texture(
        &mut self,
        unified_manager: &mut UnifiedTextureManager,
        texture_id: &str,
    ) -> bool {
        if let Some(&strategy) = self.texture_strategies.get(texture_id) {
            let result = match strategy {
                TextureLoadingStrategy::Legacy => {
                    // Legacy manager doesn't have unload method, so we can't unload
                    false
                }
                TextureLoadingStrategy::Unified => {
                    unified_manager.unload_texture(texture_id)
                }
                TextureLoadingStrategy::Auto => {
                    unified_manager.unload_texture(texture_id)
                }
            };
            
            if result {
                self.texture_strategies.remove(texture_id);
            }
            
            result
        } else {
            false
        }
    }

    /// Get texture loading statistics
    pub fn get_loading_stats(&self) -> HashMap<TextureLoadingStrategy, usize> {
        let mut stats = HashMap::new();
        
        for &strategy in self.texture_strategies.values() {
            *stats.entry(strategy).or_insert(0) += 1;
        }
        
        stats
    }

    /// Set configuration
    pub fn set_config(&mut self, config: UnifiedTextureConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &UnifiedTextureConfig {
        &self.config
    }

    /// Force garbage collection on both managers
    pub fn garbage_collect(
        &self,
        device: &wgpu::Device,
        unified_manager: &mut UnifiedTextureManager,
    ) {
        unified_manager.garbage_collect(device);
        // Legacy manager doesn't support garbage collection
    }

    /// Determine loading strategy based on usage pattern and requirements
    fn determine_loading_strategy(
        &self,
        usage_pattern: TextureUsagePattern,
        pixel_perfect: bool,
    ) -> TextureLoadingStrategy {
        match self.config.strategy {
            TextureLoadingStrategy::Legacy => TextureLoadingStrategy::Legacy,
            TextureLoadingStrategy::Unified => TextureLoadingStrategy::Unified,
            TextureLoadingStrategy::Auto => {
                // Auto strategy decision logic
                match usage_pattern {
                    TextureUsagePattern::Array => {
                        // Array textures are better supported in unified manager
                        TextureLoadingStrategy::Unified
                    }
                    TextureUsagePattern::Dynamic | TextureUsagePattern::Streaming => {
                        // Dynamic textures need unified manager features
                        TextureLoadingStrategy::Unified
                    }
                    TextureUsagePattern::Static => {
                        // For static textures, prefer unified for pixel perfect, legacy for compatibility
                        if pixel_perfect {
                            TextureLoadingStrategy::Unified
                        } else {
                            TextureLoadingStrategy::Legacy
                        }
                    }
                    TextureUsagePattern::RenderTarget => {
                        // Render targets need unified manager
                        TextureLoadingStrategy::Unified
                    }
                }
            }
        }
    }
}

/// Helper functions for texture management integration
impl UnifiedTextureIntegration {
    /// Create a sprite-optimized configuration
    pub fn sprite_optimized_config() -> UnifiedTextureConfig {
        UnifiedTextureConfig {
            strategy: TextureLoadingStrategy::Auto,
            sprite_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            tilemap_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            pixel_perfect_filter: UnifiedTextureFilter::Nearest,
            smooth_filter: UnifiedTextureFilter::Linear,
            enable_mipmaps: false, // Sprites usually don't need mipmaps
            sprite_memory_priority: 128,
            tilemap_memory_priority: 192,
        }
    }

    /// Create a tilemap-optimized configuration
    pub fn tilemap_optimized_config() -> UnifiedTextureConfig {
        UnifiedTextureConfig {
            strategy: TextureLoadingStrategy::Unified, // Prefer unified for array textures
            sprite_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            tilemap_format: UnifiedTextureFormat::Rgba8UnormSrgb,
            pixel_perfect_filter: UnifiedTextureFilter::Nearest,
            smooth_filter: UnifiedTextureFilter::Linear,
            enable_mipmaps: false, // Tilemaps usually use pixel perfect rendering
            sprite_memory_priority: 128,
            tilemap_memory_priority: 255, // Highest priority for tilemaps
        }
    }

    /// Create a memory-optimized configuration
    pub fn memory_optimized_config() -> UnifiedTextureConfig {
        UnifiedTextureConfig {
            strategy: TextureLoadingStrategy::Unified, // Better memory management
            sprite_format: UnifiedTextureFormat::Bc1RgbaUnormSrgb, // Compressed format
            tilemap_format: UnifiedTextureFormat::Bc3RgbaUnormSrgb, // Compressed with alpha
            pixel_perfect_filter: UnifiedTextureFilter::Nearest,
            smooth_filter: UnifiedTextureFilter::Linear,
            enable_mipmaps: true, // Enable mipmaps for memory efficiency
            sprite_memory_priority: 64,  // Lower priority
            tilemap_memory_priority: 128, // Medium priority
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_texture_config_defaults() {
        let config = UnifiedTextureConfig::default();
        
        assert_eq!(config.strategy, TextureLoadingStrategy::Auto);
        assert_eq!(config.sprite_format, UnifiedTextureFormat::Rgba8UnormSrgb);
        assert_eq!(config.tilemap_format, UnifiedTextureFormat::Rgba8UnormSrgb);
        assert!(!config.enable_mipmaps);
        assert_eq!(config.sprite_memory_priority, 128);
        assert_eq!(config.tilemap_memory_priority, 192);
    }

    #[test]
    fn test_loading_strategy_determination() {
        let integration = UnifiedTextureIntegration::new(None);
        
        // Array textures should prefer unified
        let strategy = integration.determine_loading_strategy(TextureUsagePattern::Array, false);
        assert_eq!(strategy, TextureLoadingStrategy::Unified);
        
        // Static pixel perfect should prefer unified
        let strategy = integration.determine_loading_strategy(TextureUsagePattern::Static, true);
        assert_eq!(strategy, TextureLoadingStrategy::Unified);
        
        // Static smooth should prefer legacy for compatibility
        let strategy = integration.determine_loading_strategy(TextureUsagePattern::Static, false);
        assert_eq!(strategy, TextureLoadingStrategy::Legacy);
    }

    #[test]
    fn test_optimized_configs() {
        let sprite_config = UnifiedTextureIntegration::sprite_optimized_config();
        assert!(!sprite_config.enable_mipmaps);
        assert_eq!(sprite_config.pixel_perfect_filter, UnifiedTextureFilter::Nearest);
        
        let tilemap_config = UnifiedTextureIntegration::tilemap_optimized_config();
        assert_eq!(tilemap_config.strategy, TextureLoadingStrategy::Unified);
        assert_eq!(tilemap_config.tilemap_memory_priority, 255);
        
        let memory_config = UnifiedTextureIntegration::memory_optimized_config();
        assert_eq!(memory_config.sprite_format, UnifiedTextureFormat::Bc1RgbaUnormSrgb);
        assert!(memory_config.enable_mipmaps);
    }
}
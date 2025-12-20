//! Unified Texture Management System
//!
//! This module provides a unified texture management system that ensures sprites and tilemaps
//! use the same WGPU texture management with consistent texture update pipeline and memory management.

use std::collections::{HashMap, BTreeMap};
use std::path::Path;
use std::sync::{Arc, Weak};
use anyhow::Result;
use image::GenericImageView;
use wgpu::util::DeviceExt;

/// Texture format configuration for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnifiedTextureFormat {
    /// Standard RGBA8 sRGB for sprites and UI
    Rgba8UnormSrgb,
    /// RGBA8 linear for normal maps and data textures
    Rgba8Unorm,
    /// Single channel for masks and alpha textures
    R8Unorm,
    /// 16-bit formats for high precision
    Rgba16Float,
    /// Compressed formats for memory efficiency
    Bc1RgbaUnormSrgb,
    Bc3RgbaUnormSrgb,
}

impl UnifiedTextureFormat {
    /// Convert to WGPU texture format
    pub fn to_wgpu_format(self) -> wgpu::TextureFormat {
        match self {
            UnifiedTextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            UnifiedTextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            UnifiedTextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            UnifiedTextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            UnifiedTextureFormat::Bc1RgbaUnormSrgb => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            UnifiedTextureFormat::Bc3RgbaUnormSrgb => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
        }
    }

    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            UnifiedTextureFormat::Rgba8UnormSrgb | UnifiedTextureFormat::Rgba8Unorm => 4,
            UnifiedTextureFormat::R8Unorm => 1,
            UnifiedTextureFormat::Rgba16Float => 8,
            UnifiedTextureFormat::Bc1RgbaUnormSrgb => 1, // Compressed: 4x4 block = 8 bytes
            UnifiedTextureFormat::Bc3RgbaUnormSrgb => 1, // Compressed: 4x4 block = 16 bytes
        }
    }

    /// Check if format supports filtering
    pub fn supports_filtering(self) -> bool {
        match self {
            UnifiedTextureFormat::Rgba8UnormSrgb 
            | UnifiedTextureFormat::Rgba8Unorm 
            | UnifiedTextureFormat::R8Unorm 
            | UnifiedTextureFormat::Rgba16Float 
            | UnifiedTextureFormat::Bc1RgbaUnormSrgb 
            | UnifiedTextureFormat::Bc3RgbaUnormSrgb => true,
        }
    }
}

/// Texture usage patterns for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUsagePattern {
    /// Static texture that rarely changes (sprites, UI elements)
    Static,
    /// Dynamic texture that updates frequently (animated sprites, procedural textures)
    Dynamic,
    /// Streaming texture for large textures loaded in chunks
    Streaming,
    /// Render target texture
    RenderTarget,
    /// Array texture for tilemaps and atlases
    Array,
}

/// Texture filtering configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedTextureFilter {
    /// Nearest neighbor (pixel perfect)
    Nearest,
    /// Linear interpolation (smooth)
    Linear,
    /// Anisotropic filtering (high quality)
    Anisotropic(u16),
}

impl UnifiedTextureFilter {
    /// Convert to WGPU filter mode
    pub fn to_wgpu_filter(self) -> wgpu::FilterMode {
        match self {
            UnifiedTextureFilter::Nearest => wgpu::FilterMode::Nearest,
            UnifiedTextureFilter::Linear | UnifiedTextureFilter::Anisotropic(_) => wgpu::FilterMode::Linear,
        }
    }

    /// Get anisotropy level
    pub fn anisotropy_level(self) -> u16 {
        match self {
            UnifiedTextureFilter::Anisotropic(level) => level,
            _ => 1,
        }
    }
}

/// Unified texture descriptor
#[derive(Debug, Clone)]
pub struct UnifiedTextureDescriptor {
    /// Texture identifier
    pub id: String,
    /// Texture format
    pub format: UnifiedTextureFormat,
    /// Texture dimensions
    pub size: (u32, u32),
    /// Number of array layers (1 for regular textures)
    pub array_layers: u32,
    /// Mip level count (1 for no mipmaps)
    pub mip_levels: u32,
    /// Usage pattern for optimization
    pub usage_pattern: TextureUsagePattern,
    /// Filtering configuration
    pub filter: UnifiedTextureFilter,
    /// Address mode for texture coordinates
    pub address_mode: wgpu::AddressMode,
    /// Whether to generate mipmaps automatically
    pub generate_mipmaps: bool,
    /// Memory priority (higher = keep in memory longer)
    pub memory_priority: u8,
}

impl Default for UnifiedTextureDescriptor {
    fn default() -> Self {
        Self {
            id: String::new(),
            format: UnifiedTextureFormat::Rgba8UnormSrgb,
            size: (1, 1),
            array_layers: 1,
            mip_levels: 1,
            usage_pattern: TextureUsagePattern::Static,
            filter: UnifiedTextureFilter::Linear,
            address_mode: wgpu::AddressMode::ClampToEdge,
            generate_mipmaps: false,
            memory_priority: 128, // Medium priority
        }
    }
}

/// Unified texture resource
#[derive(Debug)]
pub struct UnifiedTexture {
    /// WGPU texture
    pub texture: wgpu::Texture,
    /// Texture view
    pub view: wgpu::TextureView,
    /// Sampler
    pub sampler: wgpu::Sampler,
    /// Bind group for shader binding
    pub bind_group: wgpu::BindGroup,
    /// Texture descriptor
    pub descriptor: UnifiedTextureDescriptor,
    /// Reference count for memory management
    pub ref_count: Arc<()>,
    /// Last access time for LRU eviction
    pub last_access_time: std::time::Instant,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Whether texture is currently loaded in GPU memory
    pub is_loaded: bool,
}

impl UnifiedTexture {
    /// Create a new unified texture
    pub fn new(
        device: &wgpu::Device,
        descriptor: UnifiedTextureDescriptor,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let wgpu_format = descriptor.format.to_wgpu_format();
        
        // Calculate memory usage
        let memory_usage = Self::calculate_memory_usage(&descriptor);

        // Create WGPU texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&descriptor.id),
            size: wgpu::Extent3d {
                width: descriptor.size.0,
                height: descriptor.size.1,
                depth_or_array_layers: descriptor.array_layers,
            },
            mip_level_count: descriptor.mip_levels,
            sample_count: 1,
            dimension: if descriptor.array_layers > 1 {
                wgpu::TextureDimension::D2
            } else {
                wgpu::TextureDimension::D2
            },
            format: wgpu_format,
            usage: Self::get_texture_usage(&descriptor),
            view_formats: &[],
        });

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: if descriptor.array_layers > 1 {
                Some(wgpu::TextureViewDimension::D2Array)
            } else {
                Some(wgpu::TextureViewDimension::D2)
            },
            ..Default::default()
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: descriptor.address_mode,
            address_mode_v: descriptor.address_mode,
            address_mode_w: descriptor.address_mode,
            mag_filter: descriptor.filter.to_wgpu_filter(),
            min_filter: descriptor.filter.to_wgpu_filter(),
            mipmap_filter: if descriptor.mip_levels > 1 {
                descriptor.filter.to_wgpu_filter()
            } else {
                wgpu::FilterMode::Nearest
            },
            anisotropy_clamp: descriptor.filter.anisotropy_level(),
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(&format!("{}_bind_group", descriptor.id)),
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
            descriptor,
            ref_count: Arc::new(()),
            last_access_time: std::time::Instant::now(),
            memory_usage,
            is_loaded: true,
        }
    }

    /// Calculate memory usage for a texture descriptor
    fn calculate_memory_usage(descriptor: &UnifiedTextureDescriptor) -> u64 {
        let base_size = descriptor.size.0 as u64 * descriptor.size.1 as u64;
        let bytes_per_pixel = descriptor.format.bytes_per_pixel() as u64;
        let array_size = descriptor.array_layers as u64;
        
        // Calculate mipmap memory usage (approximately 1.33x for full mipmap chain)
        let mip_multiplier = if descriptor.mip_levels > 1 {
            1.33
        } else {
            1.0
        };
        
        ((base_size * bytes_per_pixel * array_size) as f64 * mip_multiplier) as u64
    }

    /// Get WGPU texture usage flags based on usage pattern
    fn get_texture_usage(descriptor: &UnifiedTextureDescriptor) -> wgpu::TextureUsages {
        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        
        match descriptor.usage_pattern {
            TextureUsagePattern::Dynamic | TextureUsagePattern::Streaming => {
                usage |= wgpu::TextureUsages::COPY_SRC;
            }
            TextureUsagePattern::RenderTarget => {
                usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
            }
            _ => {}
        }

        usage
    }

    /// Update last access time
    pub fn touch(&mut self) {
        self.last_access_time = std::time::Instant::now();
    }

    /// Get weak reference for memory management
    pub fn get_weak_ref(&self) -> Weak<()> {
        Arc::downgrade(&self.ref_count)
    }

    /// Check if texture is still referenced
    pub fn is_referenced(&self) -> bool {
        Arc::strong_count(&self.ref_count) > 1
    }
}

/// Texture streaming configuration
#[derive(Debug, Clone)]
pub struct TextureStreamingConfig {
    /// Maximum memory budget for textures (in bytes)
    pub max_memory_budget: u64,
    /// Memory threshold for starting eviction (percentage of max budget)
    pub eviction_threshold: f32,
    /// Maximum number of textures to keep in memory
    pub max_texture_count: usize,
    /// Enable texture compression
    pub enable_compression: bool,
    /// Enable automatic mipmap generation
    pub enable_mipmaps: bool,
    /// Texture quality level (0.0 = lowest, 1.0 = highest)
    pub quality_level: f32,
}

impl Default for TextureStreamingConfig {
    fn default() -> Self {
        Self {
            max_memory_budget: 512 * 1024 * 1024, // 512 MB
            eviction_threshold: 0.8, // 80%
            max_texture_count: 1000,
            enable_compression: true,
            enable_mipmaps: true,
            quality_level: 1.0,
        }
    }
}

/// Unified texture management system
pub struct UnifiedTextureManager {
    /// Loaded textures
    textures: HashMap<String, UnifiedTexture>,
    /// Texture bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Streaming configuration
    streaming_config: TextureStreamingConfig,
    /// Current memory usage
    current_memory_usage: u64,
    /// Texture access order for LRU eviction
    access_order: BTreeMap<std::time::Instant, String>,
    /// Default textures
    default_textures: HashMap<String, String>,
    /// Texture loading queue for async loading
    loading_queue: Vec<String>,
    /// Performance statistics
    stats: TextureManagerStats,
}

/// Performance statistics for texture management
#[derive(Debug, Default)]
pub struct TextureManagerStats {
    /// Total textures loaded
    pub total_textures_loaded: u64,
    /// Total memory allocated
    pub total_memory_allocated: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of textures evicted
    pub textures_evicted: u64,
    /// Time spent loading textures (microseconds)
    pub loading_time_us: u64,
    /// Time spent evicting textures (microseconds)
    pub eviction_time_us: u64,
}

impl TextureManagerStats {
    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f32 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            return 1.0;
        }
        self.cache_hits as f32 / total_requests as f32
    }

    /// Calculate memory efficiency (allocated vs budget)
    pub fn memory_efficiency(&self, budget: u64) -> f32 {
        if budget == 0 {
            return 1.0;
        }
        (self.total_memory_allocated as f32 / budget as f32).min(1.0)
    }
}

impl UnifiedTextureManager {
    /// Create a new unified texture manager
    pub fn new(device: &wgpu::Device, streaming_config: Option<TextureStreamingConfig>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("unified_texture_bind_group_layout"),
        });

        let mut manager = Self {
            textures: HashMap::new(),
            bind_group_layout,
            streaming_config: streaming_config.unwrap_or_default(),
            current_memory_usage: 0,
            access_order: BTreeMap::new(),
            default_textures: HashMap::new(),
            loading_queue: Vec::new(),
            stats: TextureManagerStats::default(),
        };

        // Initialize default textures
        manager.default_textures.insert("white".to_string(), "default_white".to_string());
        manager.default_textures.insert("black".to_string(), "default_black".to_string());
        manager.default_textures.insert("normal".to_string(), "default_normal".to_string());
        manager.default_textures.insert("transparent".to_string(), "default_transparent".to_string());

        manager
    }

    /// Load texture from file
    pub fn load_texture_from_file(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
        descriptor: UnifiedTextureDescriptor,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Check if texture is already loaded
        if self.textures.contains_key(&descriptor.id) {
            let now = std::time::Instant::now();
            if let Some(texture) = self.textures.get_mut(&descriptor.id) {
                texture.last_access_time = now;
            }
            self.access_order.retain(|_, id| id != &descriptor.id);
            self.access_order.insert(now, descriptor.id.clone());
            self.stats.cache_hits += 1;
            return Ok(());
        }

        self.stats.cache_misses += 1;

        // Load image data
        let bytes = std::fs::read(path)?;
        let img = image::load_from_memory(&bytes)?;
        
        // Create texture from image
        let texture = self.create_texture_from_image(device, queue, &img, descriptor)?;
        
        // Check memory budget and evict if necessary
        self.check_memory_budget_and_evict(device);
        
        // Insert texture
        let texture_id = texture.descriptor.id.clone();
        self.current_memory_usage += texture.memory_usage;
        let now = std::time::Instant::now();
        self.access_order.insert(now, texture_id.clone());
        self.textures.insert(texture_id.clone(), texture);
        
        self.stats.total_textures_loaded += 1;
        self.stats.total_memory_allocated += self.textures[&texture_id].memory_usage;
        self.stats.loading_time_us += start_time.elapsed().as_micros() as u64;

        Ok(())
    }

    /// Load texture from raw data
    pub fn load_texture_from_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        descriptor: UnifiedTextureDescriptor,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Check if texture is already loaded
        if self.textures.contains_key(&descriptor.id) {
            let now = std::time::Instant::now();
            if let Some(texture) = self.textures.get_mut(&descriptor.id) {
                texture.last_access_time = now;
            }
            self.access_order.retain(|_, id| id != &descriptor.id);
            self.access_order.insert(now, descriptor.id.clone());
            self.stats.cache_hits += 1;
            return Ok(());
        }

        self.stats.cache_misses += 1;

        // Create texture
        let texture = UnifiedTexture::new(device, descriptor, &self.bind_group_layout);
        
        // Upload data directly
        let bytes_per_pixel = texture.descriptor.format.bytes_per_pixel();
        let bytes_per_row = texture.descriptor.size.0 * bytes_per_pixel;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(texture.descriptor.size.1),
            },
            wgpu::Extent3d {
                width: texture.descriptor.size.0,
                height: texture.descriptor.size.1,
                depth_or_array_layers: 1,
            },
        );
        
        // Check memory budget and evict if necessary
        self.check_memory_budget_and_evict(device);
        
        // Insert texture
        let texture_id = texture.descriptor.id.clone();
        self.current_memory_usage += texture.memory_usage;
        let now = std::time::Instant::now();
        self.access_order.insert(now, texture_id.clone());
        self.textures.insert(texture_id.clone(), texture);
        
        self.stats.total_textures_loaded += 1;
        self.stats.total_memory_allocated += self.textures[&texture_id].memory_usage;
        self.stats.loading_time_us += start_time.elapsed().as_micros() as u64;

        Ok(())
    }

    /// Create array texture for tilemaps
    pub fn create_array_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: &[image::DynamicImage],
        descriptor: UnifiedTextureDescriptor,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Check if texture is already loaded
        if self.textures.contains_key(&descriptor.id) {
            let now = std::time::Instant::now();
            if let Some(texture) = self.textures.get_mut(&descriptor.id) {
                texture.last_access_time = now;
            }
            self.access_order.retain(|_, id| id != &descriptor.id);
            self.access_order.insert(now, descriptor.id.clone());
            self.stats.cache_hits += 1;
            return Ok(());
        }

        self.stats.cache_misses += 1;

        // Create array texture descriptor
        let mut array_descriptor = descriptor;
        array_descriptor.array_layers = images.len() as u32;
        
        // Create texture
        let texture = UnifiedTexture::new(device, array_descriptor, &self.bind_group_layout);
        
        // Upload each image as a layer
        for (layer, img) in images.iter().enumerate() {
            let rgba = img.to_rgba8();
            
            // Upload data directly
            let bytes_per_pixel = texture.descriptor.format.bytes_per_pixel();
            let bytes_per_row = texture.descriptor.size.0 * bytes_per_pixel;

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 },
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(texture.descriptor.size.1),
                },
                wgpu::Extent3d {
                    width: texture.descriptor.size.0,
                    height: texture.descriptor.size.1,
                    depth_or_array_layers: 1,
                },
            );
        }
        
        // Check memory budget and evict if necessary
        self.check_memory_budget_and_evict(device);
        
        // Insert texture
        let texture_id = texture.descriptor.id.clone();
        self.current_memory_usage += texture.memory_usage;
        let now = std::time::Instant::now();
        self.access_order.insert(now, texture_id.clone());
        self.textures.insert(texture_id.clone(), texture);
        
        self.stats.total_textures_loaded += 1;
        self.stats.total_memory_allocated += self.textures[&texture_id].memory_usage;
        self.stats.loading_time_us += start_time.elapsed().as_micros() as u64;

        Ok(())
    }

    /// Get texture by ID
    pub fn get_texture(&mut self, id: &str) -> Option<&UnifiedTexture> {
        let exists = self.textures.contains_key(id);
        if exists {
            self.stats.cache_hits += 1;
            
            // Update last access time and access order
            let now = std::time::Instant::now();
            if let Some(texture) = self.textures.get_mut(id) {
                texture.last_access_time = now;
            }
            
            // Update access order
            self.access_order.retain(|_, texture_id| texture_id != id);
            self.access_order.insert(now, id.to_string());
            
            // Return immutable reference
            self.textures.get(id)
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }

    /// Get or create default texture
    pub fn get_default_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        default_type: &str,
    ) -> Result<()> {
        let texture_id = self.default_textures.get(default_type)
            .cloned()
            .unwrap_or_else(|| format!("default_{}", default_type));

        // Check if already loaded
        if self.textures.contains_key(&texture_id) {
            return Ok(());
        }

        // Create default texture
        let (color, descriptor) = match default_type {
            "white" => ([255, 255, 255, 255], UnifiedTextureDescriptor {
                id: texture_id.clone(),
                size: (1, 1),
                usage_pattern: TextureUsagePattern::Static,
                memory_priority: 255, // Highest priority
                ..Default::default()
            }),
            "black" => ([0, 0, 0, 255], UnifiedTextureDescriptor {
                id: texture_id.clone(),
                size: (1, 1),
                usage_pattern: TextureUsagePattern::Static,
                memory_priority: 255,
                ..Default::default()
            }),
            "normal" => ([128, 128, 255, 255], UnifiedTextureDescriptor {
                id: texture_id.clone(),
                size: (1, 1),
                usage_pattern: TextureUsagePattern::Static,
                memory_priority: 255,
                ..Default::default()
            }),
            "transparent" => ([255, 255, 255, 0], UnifiedTextureDescriptor {
                id: texture_id.clone(),
                size: (1, 1),
                usage_pattern: TextureUsagePattern::Static,
                memory_priority: 255,
                ..Default::default()
            }),
            _ => ([255, 0, 255, 255], UnifiedTextureDescriptor { // Magenta for missing textures
                id: texture_id.clone(),
                size: (1, 1),
                usage_pattern: TextureUsagePattern::Static,
                memory_priority: 255,
                ..Default::default()
            }),
        };

        // Create 1x1 texture with the specified color
        self.load_texture_from_data(device, queue, &color, descriptor)
    }

    /// Update texture data
    pub fn update_texture_data(
        &mut self,
        queue: &wgpu::Queue,
        texture_id: &str,
        data: &[u8],
        layer: u32,
    ) -> Result<()> {
        let exists = self.textures.contains_key(texture_id);
        if exists {
            // Update access order first
            let now = std::time::Instant::now();
            self.access_order.retain(|_, id| id != texture_id);
            self.access_order.insert(now, texture_id.to_string());
            
            // Get mutable reference to texture for updating
            if let Some(texture) = self.textures.get_mut(texture_id) {
                texture.last_access_time = now;
                
                // Upload data directly
                let bytes_per_pixel = texture.descriptor.format.bytes_per_pixel();
                let bytes_per_row = texture.descriptor.size.0 * bytes_per_pixel;

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        aspect: wgpu::TextureAspect::All,
                        texture: &texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
                    },
                    data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(texture.descriptor.size.1),
                    },
                    wgpu::Extent3d {
                        width: texture.descriptor.size.0,
                        height: texture.descriptor.size.1,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        Ok(())
    }

    /// Remove texture from memory
    pub fn unload_texture(&mut self, texture_id: &str) -> bool {
        if let Some(texture) = self.textures.remove(texture_id) {
            self.current_memory_usage = self.current_memory_usage.saturating_sub(texture.memory_usage);
            self.access_order.retain(|_, id| id != texture_id);
            true
        } else {
            false
        }
    }

    /// Get bind group layout
    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &TextureManagerStats {
        &self.stats
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> u64 {
        self.current_memory_usage
    }

    /// Get memory budget
    pub fn get_memory_budget(&self) -> u64 {
        self.streaming_config.max_memory_budget
    }

    /// Get default texture ID for a given type
    pub fn get_default_texture_id(&self, default_type: &str) -> String {
        self.default_textures.get(default_type)
            .cloned()
            .unwrap_or_else(|| format!("default_{}", default_type))
    }

    /// Set streaming configuration
    pub fn set_streaming_config(&mut self, config: TextureStreamingConfig) {
        self.streaming_config = config;
    }

    /// Force garbage collection
    pub fn garbage_collect(&mut self, device: &wgpu::Device) {
        let start_time = std::time::Instant::now();
        
        // Remove unreferenced textures
        let mut to_remove = Vec::new();
        for (id, texture) in &self.textures {
            if !texture.is_referenced() && texture.descriptor.memory_priority < 255 {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.unload_texture(&id);
            self.stats.textures_evicted += 1;
        }

        // Force eviction if still over budget
        self.check_memory_budget_and_evict(device);
        
        self.stats.eviction_time_us += start_time.elapsed().as_micros() as u64;
    }

    /// Create texture from image
    fn create_texture_from_image(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        descriptor: UnifiedTextureDescriptor,
    ) -> Result<UnifiedTexture> {
        let texture = UnifiedTexture::new(device, descriptor, &self.bind_group_layout);
        let rgba = img.to_rgba8();
        
        // Upload data directly
        let bytes_per_pixel = texture.descriptor.format.bytes_per_pixel();
        let bytes_per_row = texture.descriptor.size.0 * bytes_per_pixel;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(texture.descriptor.size.1),
            },
            wgpu::Extent3d {
                width: texture.descriptor.size.0,
                height: texture.descriptor.size.1,
                depth_or_array_layers: 1,
            },
        );
        
        Ok(texture)
    }

    /// Upload texture data to GPU
    fn upload_texture_data(
        &self,
        queue: &wgpu::Queue,
        texture: &UnifiedTexture,
        data: &[u8],
        layer: u32,
    ) -> Result<()> {
        let bytes_per_pixel = texture.descriptor.format.bytes_per_pixel();
        let bytes_per_row = texture.descriptor.size.0 * bytes_per_pixel;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(texture.descriptor.size.1),
            },
            wgpu::Extent3d {
                width: texture.descriptor.size.0,
                height: texture.descriptor.size.1,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }

    /// Update access order for LRU eviction
    fn update_access_order(&mut self, texture_id: &str) {
        let now = std::time::Instant::now();
        
        // Remove old entry
        self.access_order.retain(|_, id| id != texture_id);
        
        // Add new entry
        self.access_order.insert(now, texture_id.to_string());
    }

    /// Check memory budget and evict textures if necessary
    fn check_memory_budget_and_evict(&mut self, device: &wgpu::Device) {
        let threshold = (self.streaming_config.max_memory_budget as f32 * self.streaming_config.eviction_threshold) as u64;
        
        if self.current_memory_usage <= threshold {
            return;
        }

        let start_time = std::time::Instant::now();
        
        // Evict least recently used textures
        let mut to_evict = Vec::new();
        for (_, texture_id) in &self.access_order {
            if let Some(texture) = self.textures.get(texture_id) {
                // Don't evict high priority textures
                if texture.descriptor.memory_priority >= 200 {
                    continue;
                }
                
                // Don't evict referenced textures
                if texture.is_referenced() {
                    continue;
                }
                
                to_evict.push(texture_id.clone());
                
                // Stop when we've freed enough memory
                if self.current_memory_usage <= threshold {
                    break;
                }
            }
        }

        // Remove evicted textures
        for texture_id in to_evict {
            self.unload_texture(&texture_id);
            self.stats.textures_evicted += 1;
        }
        
        self.stats.eviction_time_us += start_time.elapsed().as_micros() as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_format_conversion() {
        assert_eq!(
            UnifiedTextureFormat::Rgba8UnormSrgb.to_wgpu_format(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        
        assert_eq!(UnifiedTextureFormat::Rgba8UnormSrgb.bytes_per_pixel(), 4);
        assert!(UnifiedTextureFormat::Rgba8UnormSrgb.supports_filtering());
    }

    #[test]
    fn test_texture_filter_conversion() {
        assert_eq!(
            UnifiedTextureFilter::Nearest.to_wgpu_filter(),
            wgpu::FilterMode::Nearest
        );
        
        assert_eq!(
            UnifiedTextureFilter::Linear.to_wgpu_filter(),
            wgpu::FilterMode::Linear
        );
        
        assert_eq!(UnifiedTextureFilter::Anisotropic(16).anisotropy_level(), 16);
    }

    #[test]
    fn test_texture_descriptor_defaults() {
        let descriptor = UnifiedTextureDescriptor::default();
        
        assert_eq!(descriptor.format, UnifiedTextureFormat::Rgba8UnormSrgb);
        assert_eq!(descriptor.size, (1, 1));
        assert_eq!(descriptor.array_layers, 1);
        assert_eq!(descriptor.usage_pattern, TextureUsagePattern::Static);
        assert_eq!(descriptor.memory_priority, 128);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let descriptor = UnifiedTextureDescriptor {
            size: (256, 256),
            format: UnifiedTextureFormat::Rgba8UnormSrgb,
            array_layers: 1,
            mip_levels: 1,
            ..Default::default()
        };
        
        let memory_usage = UnifiedTexture::calculate_memory_usage(&descriptor);
        assert_eq!(memory_usage, 256 * 256 * 4); // 256x256 RGBA texture
    }

    #[test]
    fn test_streaming_config_defaults() {
        let config = TextureStreamingConfig::default();
        
        assert_eq!(config.max_memory_budget, 512 * 1024 * 1024);
        assert_eq!(config.eviction_threshold, 0.8);
        assert_eq!(config.max_texture_count, 1000);
        assert!(config.enable_compression);
        assert!(config.enable_mipmaps);
        assert_eq!(config.quality_level, 1.0);
    }

    #[test]
    fn test_texture_manager_stats() {
        let mut stats = TextureManagerStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;
        
        assert_eq!(stats.cache_hit_rate(), 0.8);
        
        stats.total_memory_allocated = 256 * 1024 * 1024; // 256 MB
        let budget = 512 * 1024 * 1024; // 512 MB
        assert_eq!(stats.memory_efficiency(budget), 0.5);
    }
}
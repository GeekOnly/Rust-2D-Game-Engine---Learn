//! Text rendering system for UI
//!
//! This module provides text rendering capabilities for the UI system,
//! including font loading, text layout, and mesh generation.

use crate::{UIText, TextAlignment, OverflowMode, Rect, Vec2};
use std::collections::HashMap;

/// A simple glyph representation for bitmap fonts
#[derive(Clone, Debug)]
pub struct Glyph {
    /// Character this glyph represents
    pub character: char,
    
    /// UV coordinates in the font texture (normalized 0-1)
    pub uv_rect: Rect,
    
    /// Advance width (how much to move cursor after this glyph)
    pub advance: f32,
    
    /// Bearing (offset from baseline)
    pub bearing: Vec2,
    
    /// Size of the glyph in pixels
    pub size: Vec2,
}

/// Font data structure
#[derive(Clone, Debug)]
pub struct Font {
    /// Font name/ID
    pub name: String,
    
    /// Texture ID for the font atlas
    pub texture_id: String,
    
    /// Glyphs in this font
    pub glyphs: HashMap<char, Glyph>,
    
    /// Line height for this font
    pub line_height: f32,
    
    /// Base size of the font
    pub base_size: f32,
}

impl Font {
    /// Create a new font
    pub fn new(name: String, texture_id: String, base_size: f32, line_height: f32) -> Self {
        Self {
            name,
            texture_id,
            glyphs: HashMap::new(),
            line_height,
            base_size,
        }
    }
    
    /// Add a glyph to the font
    pub fn add_glyph(&mut self, glyph: Glyph) {
        self.glyphs.insert(glyph.character, glyph);
    }
    
    /// Get a glyph for a character (returns space if not found)
    pub fn get_glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(&c).or_else(|| self.glyphs.get(&' '))
    }
}

/// Font cache for managing loaded fonts
pub struct FontCache {
    fonts: HashMap<String, Font>,
    default_font: String,
}

impl FontCache {
    /// Create a new font cache
    pub fn new() -> Self {
        let mut cache = Self {
            fonts: HashMap::new(),
            default_font: String::from("default"),
        };
        
        // Create a default font with basic ASCII characters
        cache.add_default_font();
        
        cache
    }
    
    /// Add the default font (simple monospace)
    fn add_default_font(&mut self) {
        let mut font = Font::new(
            "default".to_string(),
            "default_font".to_string(),
            16.0,
            20.0,
        );
        
        // Add basic ASCII glyphs (simplified - in a real implementation,
        // these would be loaded from a font atlas texture)
        let chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        
        for (i, c) in chars.chars().enumerate() {
            let col = i % 16;
            let row = i / 16;
            
            let glyph = Glyph {
                character: c,
                uv_rect: Rect {
                    x: (col as f32) / 16.0,
                    y: (row as f32) / 8.0,
                    width: 1.0 / 16.0,
                    height: 1.0 / 8.0,
                },
                advance: 10.0,
                bearing: Vec2::new(0.0, 12.0),
                size: Vec2::new(10.0, 16.0),
            };
            
            font.add_glyph(glyph);
        }
        
        self.fonts.insert("default".to_string(), font);
    }
    
    /// Load a font (placeholder for future implementation)
    pub fn load_font(&mut self, _name: &str, _path: &str) -> Result<(), String> {
        // TODO: Implement actual font loading from file
        Ok(())
    }
    
    /// Get a font by name
    pub fn get_font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name).or_else(|| self.fonts.get(&self.default_font))
    }
    
    /// Add a font to the cache
    pub fn add_font(&mut self, font: Font) {
        self.fonts.insert(font.name.clone(), font);
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

/// A positioned glyph in a text layout
#[derive(Clone, Debug)]
pub struct PositionedGlyph {
    /// The glyph
    pub glyph: Glyph,
    
    /// Position in the text layout
    pub position: Vec2,
    
    /// Scale factor
    pub scale: f32,
}

/// Text layout result
#[derive(Clone, Debug)]
pub struct TextLayout {
    /// Positioned glyphs
    pub glyphs: Vec<PositionedGlyph>,
    
    /// Total bounds of the text
    pub bounds: Rect,
    
    /// Font used
    pub font_name: String,
    
    /// Texture ID for rendering
    pub texture_id: String,
}

/// Text renderer for generating text layouts
pub struct TextRenderer {
    font_cache: FontCache,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        Self {
            font_cache: FontCache::new(),
        }
    }
    
    /// Get the font cache
    pub fn font_cache(&self) -> &FontCache {
        &self.font_cache
    }
    
    /// Get mutable font cache
    pub fn font_cache_mut(&mut self) -> &mut FontCache {
        &mut self.font_cache
    }
    
    /// Generate a text layout from UIText component
    pub fn layout_text(
        &self,
        text: &UIText,
        bounds: Rect,
    ) -> TextLayout {
        let font = self.font_cache.get_font(&text.font)
            .expect("Font not found");
        
        let scale = text.font_size / font.base_size;
        let line_height = font.line_height * scale;
        
        let mut glyphs = Vec::new();
        let mut cursor = Vec2::new(0.0, 0.0);
        let mut max_width = 0.0f32;
        let mut max_height = line_height;
        
        // Process text based on overflow mode
        let processed_text = match text.horizontal_overflow {
            OverflowMode::Wrap => {
                self.wrap_text(&text.text, font, scale, bounds.width)
            }
            OverflowMode::Truncate => {
                self.truncate_text(&text.text, font, scale, bounds.width)
            }
            OverflowMode::Overflow => {
                text.text.clone()
            }
        };
        
        // Layout glyphs
        for c in processed_text.chars() {
            if c == '\n' {
                cursor.x = 0.0;
                cursor.y += line_height * text.line_spacing;
                max_height = cursor.y + line_height;
                continue;
            }
            
            if let Some(glyph) = font.get_glyph(c) {
                let pos = Vec2::new(
                    cursor.x + glyph.bearing.x * scale,
                    cursor.y + glyph.bearing.y * scale,
                );
                
                glyphs.push(PositionedGlyph {
                    glyph: glyph.clone(),
                    position: pos,
                    scale,
                });
                
                cursor.x += glyph.advance * scale;
                max_width = max_width.max(cursor.x);
            }
        }
        
        // Apply alignment
        let text_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: max_width,
            height: max_height,
        };
        
        self.apply_alignment(&mut glyphs, &text_bounds, bounds, text.alignment);
        
        TextLayout {
            glyphs,
            bounds: text_bounds,
            font_name: font.name.clone(),
            texture_id: font.texture_id.clone(),
        }
    }
    
    /// Wrap text to fit within width
    fn wrap_text(&self, text: &str, font: &Font, scale: f32, max_width: f32) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        
        for word in text.split_whitespace() {
            let word_width: f32 = word.chars()
                .filter_map(|c| font.get_glyph(c))
                .map(|g| g.advance * scale)
                .sum();
            
            if current_width + word_width > max_width && !current_line.is_empty() {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
                current_width = 0.0;
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += font.get_glyph(' ')
                    .map(|g| g.advance * scale)
                    .unwrap_or(0.0);
            }
            
            current_line.push_str(word);
            current_width += word_width;
        }
        
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }
        
        result
    }
    
    /// Truncate text to fit within width
    fn truncate_text(&self, text: &str, font: &Font, scale: f32, max_width: f32) -> String {
        let mut result = String::new();
        let mut current_width = 0.0;
        let ellipsis_width = font.get_glyph('.')
            .map(|g| g.advance * scale * 3.0)
            .unwrap_or(0.0);
        
        for c in text.chars() {
            if c == '\n' {
                result.push(c);
                current_width = 0.0;
                continue;
            }
            
            let char_width = font.get_glyph(c)
                .map(|g| g.advance * scale)
                .unwrap_or(0.0);
            
            if current_width + char_width + ellipsis_width > max_width {
                result.push_str("...");
                break;
            }
            
            result.push(c);
            current_width += char_width;
        }
        
        result
    }
    
    /// Apply text alignment to positioned glyphs
    fn apply_alignment(
        &self,
        glyphs: &mut [PositionedGlyph],
        text_bounds: &Rect,
        container_bounds: Rect,
        alignment: TextAlignment,
    ) {
        // Calculate offset based on alignment
        let (h_align, v_align) = match alignment {
            TextAlignment::TopLeft => (0.0, 0.0),
            TextAlignment::TopCenter => (0.5, 0.0),
            TextAlignment::TopRight => (1.0, 0.0),
            TextAlignment::MiddleLeft => (0.0, 0.5),
            TextAlignment::MiddleCenter => (0.5, 0.5),
            TextAlignment::MiddleRight => (1.0, 0.5),
            TextAlignment::BottomLeft => (0.0, 1.0),
            TextAlignment::BottomCenter => (0.5, 1.0),
            TextAlignment::BottomRight => (1.0, 1.0),
        };
        
        let offset_x = (container_bounds.width - text_bounds.width) * h_align;
        let offset_y = (container_bounds.height - text_bounds.height) * v_align;
        
        // Apply offset to all glyphs
        for glyph in glyphs.iter_mut() {
            glyph.position.x += offset_x + container_bounds.x;
            glyph.position.y += offset_y + container_bounds.y;
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_font_cache_default() {
        let cache = FontCache::new();
        assert!(cache.get_font("default").is_some());
    }
    
    #[test]
    fn test_text_layout_basic() {
        let renderer = TextRenderer::new();
        let text = UIText {
            text: "Hello".to_string(),
            font: "default".to_string(),
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            alignment: TextAlignment::TopLeft,
            horizontal_overflow: OverflowMode::Overflow,
            vertical_overflow: OverflowMode::Overflow,
            rich_text: false,
            line_spacing: 1.0,
            best_fit: false,
            best_fit_min_size: 10.0,
            best_fit_max_size: 40.0,
        };
        
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        };
        
        let layout = renderer.layout_text(&text, bounds);
        assert_eq!(layout.glyphs.len(), 5); // "Hello" has 5 characters
    }
    
    #[test]
    fn test_text_wrapping() {
        let renderer = TextRenderer::new();
        let font = renderer.font_cache.get_font("default").unwrap();
        
        let text = "Hello World";
        let wrapped = renderer.wrap_text(text, font, 1.0, 50.0);
        
        // Should wrap to multiple lines
        assert!(wrapped.contains('\n'));
    }
    
    #[test]
    fn test_text_truncation() {
        let renderer = TextRenderer::new();
        let font = renderer.font_cache.get_font("default").unwrap();
        
        let text = "This is a very long text that should be truncated";
        let truncated = renderer.truncate_text(text, font, 1.0, 100.0);
        
        // Should end with ellipsis
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() < text.len());
    }
}

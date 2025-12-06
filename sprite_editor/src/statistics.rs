//! Sprite Statistics
//!
//! Analysis and validation for sprite sheets.

use crate::metadata::{SpriteDefinition, SpriteMetadata};

/// Statistics and validation results for sprite sheet
#[derive(Debug, Clone, Default)]
pub struct SpriteStatistics {
    pub sprite_count: usize,
    pub texture_coverage_percent: f32,
    pub overlapping_sprites: Vec<(usize, usize)>,
    pub out_of_bounds_sprites: Vec<usize>,
}

impl SpriteStatistics {
    /// Calculate statistics for a sprite metadata
    pub fn calculate(metadata: &SpriteMetadata) -> Self {
        let sprite_count = metadata.sprites.len();

        // Calculate texture coverage
        let texture_area = (metadata.texture_width * metadata.texture_height) as f32;
        let total_sprite_area: u32 = metadata
            .sprites
            .iter()
            .map(|s| s.width * s.height)
            .sum();
        let texture_coverage_percent = if texture_area > 0.0 {
            (total_sprite_area as f32 / texture_area) * 100.0
        } else {
            0.0
        };

        // Detect overlapping sprites
        let mut overlapping_sprites = Vec::new();
        for i in 0..metadata.sprites.len() {
            for j in (i + 1)..metadata.sprites.len() {
                if Self::sprites_overlap(&metadata.sprites[i], &metadata.sprites[j]) {
                    overlapping_sprites.push((i, j));
                }
            }
        }

        // Detect out-of-bounds sprites
        let mut out_of_bounds_sprites = Vec::new();
        for (idx, sprite) in metadata.sprites.iter().enumerate() {
            if sprite.x + sprite.width > metadata.texture_width
                || sprite.y + sprite.height > metadata.texture_height
            {
                out_of_bounds_sprites.push(idx);
            }
        }

        Self {
            sprite_count,
            texture_coverage_percent,
            overlapping_sprites,
            out_of_bounds_sprites,
        }
    }

    /// Check if two sprites overlap
    fn sprites_overlap(sprite1: &SpriteDefinition, sprite2: &SpriteDefinition) -> bool {
        let s1_left = sprite1.x;
        let s1_right = sprite1.x + sprite1.width;
        let s1_top = sprite1.y;
        let s1_bottom = sprite1.y + sprite1.height;

        let s2_left = sprite2.x;
        let s2_right = sprite2.x + sprite2.width;
        let s2_top = sprite2.y;
        let s2_bottom = sprite2.y + sprite2.height;

        // Check if rectangles overlap
        !(s1_right <= s2_left
            || s2_right <= s1_left
            || s1_bottom <= s2_top
            || s2_bottom <= s1_top)
    }

    /// Check if there are any warnings or errors
    pub fn has_issues(&self) -> bool {
        !self.overlapping_sprites.is_empty() || !self.out_of_bounds_sprites.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_overlap_detection() {
        let sprite1 = SpriteDefinition::new("s1".to_string(), 0, 0, 32, 32);
        let sprite2 = SpriteDefinition::new("s2".to_string(), 16, 16, 32, 32);
        let sprite3 = SpriteDefinition::new("s3".to_string(), 64, 64, 32, 32);

        assert!(SpriteStatistics::sprites_overlap(&sprite1, &sprite2));
        assert!(!SpriteStatistics::sprites_overlap(&sprite1, &sprite3));
    }

    #[test]
    fn test_texture_coverage() {
        let mut metadata = SpriteMetadata::new("test.png".to_string(), 256, 256);
        metadata.add_sprite(SpriteDefinition::new("s1".to_string(), 0, 0, 128, 128));

        let stats = SpriteStatistics::calculate(&metadata);

        // 128x128 = 16384 pixels out of 256x256 = 65536 pixels = 25%
        assert!((stats.texture_coverage_percent - 25.0).abs() < 0.01);
    }
}

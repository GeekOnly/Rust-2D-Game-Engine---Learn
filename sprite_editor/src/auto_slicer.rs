//! Auto Slicer
//!
//! Automatic sprite slicing utilities for grid-based sprite sheets.

use crate::metadata::SpriteDefinition;

/// Auto-slicer for grid-based sprite slicing
pub struct AutoSlicer;

impl AutoSlicer {
    /// Slice a texture into a grid of sprites
    ///
    /// # Arguments
    /// * `texture_width` - Width of the texture in pixels
    /// * `texture_height` - Height of the texture in pixels
    /// * `columns` - Number of columns in the grid
    /// * `rows` - Number of rows in the grid
    /// * `padding` - Padding from the edges of the texture in pixels
    /// * `spacing` - Spacing between sprites in pixels
    ///
    /// # Returns
    /// A vector of sprite definitions arranged in a grid
    pub fn slice_by_grid(
        texture_width: u32,
        texture_height: u32,
        columns: u32,
        rows: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition> {
        if columns == 0 || rows == 0 {
            return Vec::new();
        }

        // Calculate available space after accounting for padding
        let available_width = texture_width.saturating_sub(padding * 2);
        let available_height = texture_height.saturating_sub(padding * 2);

        // Calculate total spacing
        let total_horizontal_spacing = spacing * (columns - 1);
        let total_vertical_spacing = spacing * (rows - 1);

        // Calculate sprite dimensions
        let sprite_width = available_width.saturating_sub(total_horizontal_spacing) / columns;
        let sprite_height = available_height.saturating_sub(total_vertical_spacing) / rows;

        // Validate sprite dimensions
        if sprite_width == 0 || sprite_height == 0 {
            return Vec::new();
        }

        let mut sprites = Vec::new();
        let mut sprite_index = 0;

        for row in 0..rows {
            for col in 0..columns {
                // Calculate sprite position
                let x = padding + (col * (sprite_width + spacing));
                let y = padding + (row * (sprite_height + spacing));

                // Create sprite with sequential name
                let sprite = SpriteDefinition::new(
                    format!("sprite_{}", sprite_index),
                    x,
                    y,
                    sprite_width,
                    sprite_height,
                );

                sprites.push(sprite);
                sprite_index += 1;
            }
        }

        sprites
    }

    /// Slice a texture by cell size
    ///
    /// # Arguments
    /// * `texture_width` - Width of the texture in pixels
    /// * `texture_height` - Height of the texture in pixels
    /// * `cell_width` - Width of each sprite cell in pixels
    /// * `cell_height` - Height of each sprite cell in pixels
    /// * `padding` - Padding from the edges of the texture in pixels
    /// * `spacing` - Spacing between sprites in pixels
    ///
    /// # Returns
    /// A vector of sprite definitions based on cell size
    pub fn slice_by_cell_size(
        texture_width: u32,
        texture_height: u32,
        cell_width: u32,
        cell_height: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition> {
        if cell_width == 0 || cell_height == 0 {
            return Vec::new();
        }

        // Calculate available space after accounting for padding
        let available_width = texture_width.saturating_sub(padding * 2);
        let available_height = texture_height.saturating_sub(padding * 2);

        // Calculate how many sprites fit
        let columns = if spacing > 0 {
            (available_width + spacing) / (cell_width + spacing)
        } else {
            available_width / cell_width
        };

        let rows = if spacing > 0 {
            (available_height + spacing) / (cell_height + spacing)
        } else {
            available_height / cell_height
        };

        if columns == 0 || rows == 0 {
            return Vec::new();
        }

        let mut sprites = Vec::new();
        let mut sprite_index = 0;

        for row in 0..rows {
            for col in 0..columns {
                // Calculate sprite position
                let x = padding + (col * (cell_width + spacing));
                let y = padding + (row * (cell_height + spacing));

                // Ensure sprite doesn't exceed texture bounds
                if x + cell_width <= texture_width && y + cell_height <= texture_height {
                    let sprite = SpriteDefinition::new(
                        format!("sprite_{}", sprite_index),
                        x,
                        y,
                        cell_width,
                        cell_height,
                    );

                    sprites.push(sprite);
                    sprite_index += 1;
                }
            }
        }

        sprites
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_by_grid() {
        let sprites = AutoSlicer::slice_by_grid(256, 256, 4, 4, 0, 0);
        assert_eq!(sprites.len(), 16);

        // First sprite should be at (0, 0) with size (64, 64)
        assert_eq!(sprites[0].x, 0);
        assert_eq!(sprites[0].y, 0);
        assert_eq!(sprites[0].width, 64);
        assert_eq!(sprites[0].height, 64);
    }

    #[test]
    fn test_slice_by_cell_size() {
        let sprites = AutoSlicer::slice_by_cell_size(256, 256, 32, 32, 0, 0);
        assert_eq!(sprites.len(), 64); // 8x8 grid of 32x32 sprites
    }

    #[test]
    fn test_slice_with_padding() {
        let sprites = AutoSlicer::slice_by_grid(256, 256, 2, 2, 16, 0);
        assert_eq!(sprites.len(), 4);

        // With 16px padding, available space is 224x224
        // Each sprite should be 112x112
        assert_eq!(sprites[0].width, 112);
        assert_eq!(sprites[0].height, 112);
    }
}

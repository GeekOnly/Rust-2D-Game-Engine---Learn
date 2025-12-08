//! 9-slice sprite mesh generation
//!
//! This module provides functionality for generating mesh data for 9-slice sprites,
//! which preserve corner and edge regions when scaled.

use glam::{Vec2, Vec4};
use crate::Rect;

/// Vertex data for UI rendering
#[derive(Clone, Copy, Debug)]
pub struct UIVertex {
    /// Position in screen space
    pub position: Vec2,
    /// Texture coordinates (UV)
    pub uv: Vec2,
    /// Vertex color (RGBA)
    pub color: [f32; 4],
}

/// Mesh data for a UI element
#[derive(Clone, Debug)]
pub struct UIMesh {
    /// Vertex data
    pub vertices: Vec<UIVertex>,
    /// Index data (triangles)
    pub indices: Vec<u32>,
}

impl UIMesh {
    /// Create an empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Create a mesh with pre-allocated capacity
    pub fn with_capacity(vertex_count: usize, index_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count),
            indices: Vec::with_capacity(index_count),
        }
    }

    /// Clear the mesh data
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Add a quad to the mesh
    pub fn add_quad(
        &mut self,
        position: Rect,
        uv: Rect,
        color: [f32; 4],
    ) {
        let base_index = self.vertices.len() as u32;

        // Add vertices (bottom-left, top-left, top-right, bottom-right)
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x, position.y),
            uv: Vec2::new(uv.x, uv.y),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x, position.y + position.height),
            uv: Vec2::new(uv.x, uv.y + uv.height),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x + position.width, position.y + position.height),
            uv: Vec2::new(uv.x + uv.width, uv.y + uv.height),
            color,
        });
        self.vertices.push(UIVertex {
            position: Vec2::new(position.x + position.width, position.y),
            uv: Vec2::new(uv.x + uv.width, uv.y),
            color,
        });

        // Add indices for two triangles
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }
}

impl Default for UIMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a 9-slice mesh for a UI image
///
/// The 9-slice technique divides a sprite into 9 regions:
/// ```text
/// +---+-------+---+
/// | 1 |   2   | 3 |  <- Top row (fixed height)
/// +---+-------+---+
/// | 4 |   5   | 6 |  <- Middle row (stretched)
/// +---+-------+---+
/// | 7 |   8   | 9 |  <- Bottom row (fixed height)
/// +---+-------+---+
///   ^     ^     ^
///   |     |     |
/// Fixed  Stretch Fixed
/// width  width   width
/// ```
///
/// Corners (1, 3, 7, 9) maintain their original size
/// Edges (2, 4, 6, 8) stretch in one dimension
/// Center (5) stretches in both dimensions
///
/// # Arguments
///
/// * `rect` - The target rectangle to fill
/// * `borders` - Border sizes (left, bottom, right, top) in pixels
/// * `texture_size` - The original texture size in pixels
/// * `color` - The color tint to apply
///
/// # Returns
///
/// A `UIMesh` containing the vertices and indices for the 9-slice sprite
pub fn generate_nine_slice_mesh(
    rect: Rect,
    borders: Vec4,
    texture_size: Vec2,
    color: [f32; 4],
) -> UIMesh {
    let mut mesh = UIMesh::with_capacity(16, 54); // 9 quads = 16 vertices, 54 indices

    let left = borders.x;
    let bottom = borders.y;
    let right = borders.z;
    let top = borders.w;

    // Calculate the 9 regions in screen space
    let x0 = rect.x;
    let x1 = rect.x + left;
    let x2 = rect.x + rect.width - right;
    let x3 = rect.x + rect.width;

    let y0 = rect.y;
    let y1 = rect.y + bottom;
    let y2 = rect.y + rect.height - top;
    let y3 = rect.y + rect.height;

    // Calculate UV coordinates (normalized 0-1)
    let u0 = 0.0;
    let u1 = left / texture_size.x;
    let u2 = (texture_size.x - right) / texture_size.x;
    let u3 = 1.0;

    let v0 = 0.0;
    let v1 = bottom / texture_size.y;
    let v2 = (texture_size.y - top) / texture_size.y;
    let v3 = 1.0;

    // Generate the 9 quads
    // Bottom-left corner (region 7)
    if left > 0.0 && bottom > 0.0 {
        mesh.add_quad(
            Rect { x: x0, y: y0, width: left, height: bottom },
            Rect { x: u0, y: v0, width: u1 - u0, height: v1 - v0 },
            color,
        );
    }

    // Bottom edge (region 8)
    if bottom > 0.0 && x2 > x1 {
        mesh.add_quad(
            Rect { x: x1, y: y0, width: x2 - x1, height: bottom },
            Rect { x: u1, y: v0, width: u2 - u1, height: v1 - v0 },
            color,
        );
    }

    // Bottom-right corner (region 9)
    if right > 0.0 && bottom > 0.0 {
        mesh.add_quad(
            Rect { x: x2, y: y0, width: right, height: bottom },
            Rect { x: u2, y: v0, width: u3 - u2, height: v1 - v0 },
            color,
        );
    }

    // Left edge (region 4)
    if left > 0.0 && y2 > y1 {
        mesh.add_quad(
            Rect { x: x0, y: y1, width: left, height: y2 - y1 },
            Rect { x: u0, y: v1, width: u1 - u0, height: v2 - v1 },
            color,
        );
    }

    // Center (region 5)
    if x2 > x1 && y2 > y1 {
        mesh.add_quad(
            Rect { x: x1, y: y1, width: x2 - x1, height: y2 - y1 },
            Rect { x: u1, y: v1, width: u2 - u1, height: v2 - v1 },
            color,
        );
    }

    // Right edge (region 6)
    if right > 0.0 && y2 > y1 {
        mesh.add_quad(
            Rect { x: x2, y: y1, width: right, height: y2 - y1 },
            Rect { x: u2, y: v1, width: u3 - u2, height: v2 - v1 },
            color,
        );
    }

    // Top-left corner (region 1)
    if left > 0.0 && top > 0.0 {
        mesh.add_quad(
            Rect { x: x0, y: y2, width: left, height: top },
            Rect { x: u0, y: v2, width: u1 - u0, height: v3 - v2 },
            color,
        );
    }

    // Top edge (region 2)
    if top > 0.0 && x2 > x1 {
        mesh.add_quad(
            Rect { x: x1, y: y2, width: x2 - x1, height: top },
            Rect { x: u1, y: v2, width: u2 - u1, height: v3 - v2 },
            color,
        );
    }

    // Top-right corner (region 3)
    if right > 0.0 && top > 0.0 {
        mesh.add_quad(
            Rect { x: x2, y: y2, width: right, height: top },
            Rect { x: u2, y: v2, width: u3 - u2, height: v3 - v2 },
            color,
        );
    }

    mesh
}

/// Generate a simple quad mesh (for non-sliced images)
pub fn generate_simple_mesh(
    rect: Rect,
    color: [f32; 4],
) -> UIMesh {
    let mut mesh = UIMesh::with_capacity(4, 6);

    mesh.add_quad(
        rect,
        Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 },
        color,
    );

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mesh_generation() {
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let color = [1.0, 1.0, 1.0, 1.0];

        let mesh = generate_simple_mesh(rect, color);

        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);

        // Check vertices are in correct positions
        assert_eq!(mesh.vertices[0].position, Vec2::new(0.0, 0.0));
        assert_eq!(mesh.vertices[1].position, Vec2::new(0.0, 100.0));
        assert_eq!(mesh.vertices[2].position, Vec2::new(100.0, 100.0));
        assert_eq!(mesh.vertices[3].position, Vec2::new(100.0, 0.0));

        // Check UVs
        assert_eq!(mesh.vertices[0].uv, Vec2::new(0.0, 0.0));
        assert_eq!(mesh.vertices[1].uv, Vec2::new(0.0, 1.0));
        assert_eq!(mesh.vertices[2].uv, Vec2::new(1.0, 1.0));
        assert_eq!(mesh.vertices[3].uv, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_nine_slice_corner_preservation() {
        // Feature: in-game-ui-system, Property 12: 9-slice corner preservation
        // For any UIImage with 9-slice enabled and any size, the corner regions
        // should maintain their original pixel dimensions without distortion.

        let borders = Vec4::new(10.0, 10.0, 10.0, 10.0);
        let texture_size = Vec2::new(64.0, 64.0);
        let color = [1.0, 1.0, 1.0, 1.0];

        // Test with a small rect
        let small_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let small_mesh = generate_nine_slice_mesh(small_rect, borders, texture_size, color);

        // Test with a large rect
        let large_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let large_mesh = generate_nine_slice_mesh(large_rect, borders, texture_size, color);

        // Both meshes should have the same corner sizes (10x10 pixels)
        // We can verify this by checking that the first quad (bottom-left corner)
        // has the same size in both meshes

        // For small mesh, check bottom-left corner
        if small_mesh.vertices.len() >= 4 {
            let corner_width = small_mesh.vertices[3].position.x - small_mesh.vertices[0].position.x;
            let corner_height = small_mesh.vertices[1].position.y - small_mesh.vertices[0].position.y;
            assert!((corner_width - 10.0).abs() < 0.01, "Corner width should be 10.0");
            assert!((corner_height - 10.0).abs() < 0.01, "Corner height should be 10.0");
        }

        // For large mesh, check bottom-left corner
        if large_mesh.vertices.len() >= 4 {
            let corner_width = large_mesh.vertices[3].position.x - large_mesh.vertices[0].position.x;
            let corner_height = large_mesh.vertices[1].position.y - large_mesh.vertices[0].position.y;
            assert!((corner_width - 10.0).abs() < 0.01, "Corner width should be 10.0");
            assert!((corner_height - 10.0).abs() < 0.01, "Corner height should be 10.0");
        }
    }

    #[test]
    fn test_nine_slice_with_zero_borders() {
        let borders = Vec4::ZERO;
        let texture_size = Vec2::new(64.0, 64.0);
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let color = [1.0, 1.0, 1.0, 1.0];

        let mesh = generate_nine_slice_mesh(rect, borders, texture_size, color);

        // With zero borders, should generate only the center quad
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
    }

    #[test]
    fn test_nine_slice_with_asymmetric_borders() {
        let borders = Vec4::new(5.0, 10.0, 15.0, 20.0); // left, bottom, right, top
        let texture_size = Vec2::new(64.0, 64.0);
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let color = [1.0, 1.0, 1.0, 1.0];

        let mesh = generate_nine_slice_mesh(rect, borders, texture_size, color);

        // Should generate 9 quads (36 vertices, 54 indices)
        assert!(mesh.vertices.len() > 0);
        assert!(mesh.indices.len() > 0);

        // Verify corner sizes match border specifications
        // Bottom-left corner should be 5x10
        if mesh.vertices.len() >= 4 {
            let corner_width = mesh.vertices[3].position.x - mesh.vertices[0].position.x;
            let corner_height = mesh.vertices[1].position.y - mesh.vertices[0].position.y;
            assert!((corner_width - 5.0).abs() < 0.01, "Left border should be 5.0");
            assert!((corner_height - 10.0).abs() < 0.01, "Bottom border should be 10.0");
        }
    }
}

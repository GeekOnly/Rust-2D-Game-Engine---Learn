//! Tilemap 3D Renderer
//!
//! Handles rendering of tilemaps in 3D space with proper depth sorting and projection.

use ecs::{World, Entity};
use egui;
use glam::{Vec2, Vec3};
use std::collections::HashSet;
use crate::editor::SceneCamera;
use super::projection_3d::{self, Transform3D};

/// Tilemap 3D renderer for rendering tilemaps in 3D mode
pub struct Tilemap3DRenderer {
    /// Layers sorted by depth
    layers: Vec<TilemapLayer>,
    
    /// Selected tilemaps
    selected_tilemaps: HashSet<Entity>,
    
    /// Hovered tilemap
    hovered_tilemap: Option<Entity>,
}

/// Tilemap layer data for 3D rendering
#[derive(Clone, Debug)]
pub struct TilemapLayer {
    pub entity: Entity,
    pub z_depth: f32,
    pub tiles: Vec<TileRenderData>,
    pub bounds: egui::Rect,
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
}

/// Tile render data for individual tiles
#[derive(Clone, Debug)]
pub struct TileRenderData {
    pub world_pos: Vec3,
    pub texture_id: String,
    pub tile_rect: [u32; 4], // [x, y, width, height] in texture
    pub color: [f32; 4],
    pub flip_h: bool,
    pub flip_v: bool,
    pub width: f32,
    pub height: f32,
}

/// Screen-space tile data after projection
#[derive(Clone, Debug)]
pub struct ScreenTile {
    pub screen_pos: Vec2,
    pub screen_size: Vec2,
    pub color: egui::Color32,
    pub depth: f32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub texture_id: String,
    pub tile_rect: [u32; 4],
}

impl Tilemap3DRenderer {
    /// Create a new Tilemap3DRenderer
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            selected_tilemaps: HashSet::new(),
            hovered_tilemap: None,
        }
    }
    
    /// Collect all tilemaps from the world
    pub fn collect_tilemaps(&mut self, world: &World) -> Vec<TilemapLayer> {
        let mut layers = Vec::new();
        
        for (&entity, tilemap) in world.tilemaps.iter() {
            // Skip invisible tilemaps
            if !tilemap.visible {
                continue;
            }
            
            // Get transform for this tilemap entity
            let transform = match world.transforms.get(&entity) {
                Some(t) => t,
                None => continue,
            };
            
            // Get tileset for this tilemap
            let tileset = world.tilesets.iter()
                .find(|(_, ts)| ts.texture_id == tilemap.tileset_id)
                .map(|(_, ts)| ts);
            
            let tileset = match tileset {
                Some(ts) => ts,
                None => continue,
            };
            
            // Collect tiles
            let mut tiles = Vec::new();
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            
            for y in 0..tilemap.height {
                for x in 0..tilemap.width {
                    if let Some(tile) = tilemap.get_tile(x, y) {
                        // Skip empty tiles
                        if tile.is_empty() {
                            continue;
                        }
                        
                        // Get tile coordinates in tileset
                        let tile_coords = match tileset.get_tile_coords(tile.tile_id) {
                            Some(coords) => coords,
                            None => continue,
                        };
                        
                        // Calculate world position
                        let tile_world_x = transform.position[0] + (x as f32 * tileset.tile_width as f32);
                        let tile_world_y = transform.position[1] + (y as f32 * tileset.tile_height as f32);
                        let tile_world_z = transform.position[2];
                        
                        // Update bounds
                        min_x = min_x.min(tile_world_x);
                        min_y = min_y.min(tile_world_y);
                        max_x = max_x.max(tile_world_x + tileset.tile_width as f32);
                        max_y = max_y.max(tile_world_y + tileset.tile_height as f32);
                        
                        // Create tile render data
                        tiles.push(TileRenderData {
                            world_pos: Vec3::new(tile_world_x, tile_world_y, tile_world_z),
                            texture_id: tileset.texture_id.clone(),
                            tile_rect: [
                                tile_coords.0,
                                tile_coords.1,
                                tileset.tile_width,
                                tileset.tile_height,
                            ],
                            color: [1.0, 1.0, 1.0, tilemap.opacity],
                            flip_h: tile.flip_h,
                            flip_v: tile.flip_v,
                            width: tileset.tile_width as f32,
                            height: tileset.tile_height as f32,
                        });
                    }
                }
            }
            
            // Create bounds rect
            let bounds = if tiles.is_empty() {
                egui::Rect::NOTHING
            } else {
                egui::Rect::from_min_max(
                    egui::pos2(min_x, min_y),
                    egui::pos2(max_x, max_y),
                )
            };
            
            // Create layer
            layers.push(TilemapLayer {
                entity,
                z_depth: transform.position[2],
                tiles,
                bounds,
                name: tilemap.name.clone(),
                opacity: tilemap.opacity,
                visible: tilemap.visible,
            });
        }
        
        layers
    }
    
    /// Sort tilemap layers by Z depth (farther first for painter's algorithm)
    pub fn depth_sort_layers(&mut self, layers: &mut Vec<TilemapLayer>) {
        // Sort in descending order (farther first)
        layers.sort_by(|a, b| {
            // Handle NaN/Inf depths - treat as far plane
            let safe_depth_a = if !a.z_depth.is_finite() { f32::MAX } else { a.z_depth };
            let safe_depth_b = if !b.z_depth.is_finite() { f32::MAX } else { b.z_depth };
            
            // Use total_cmp for consistent ordering even with special values
            safe_depth_b.total_cmp(&safe_depth_a)
        });
    }
    
    /// Calculate depth of a tile from camera
    pub fn calculate_depth_from_camera(&self, position: &Vec3, camera: &SceneCamera) -> f32 {
        // Use simple distance for now, or project to view space
        // This is used for sorting
        let cam_pos = Vec3::new(camera.position.x, 0.0, camera.position.y);
        let pos_flat = Vec3::new(position.x, 0.0, position.z); // Assuming Y is up in 3D, but here Z is depth?
        // Wait, in this engine:
        // 2D: X, Y are position. Z is depth/layer.
        // 3D View: X, Y are ground plane? Or X, Y are screen plane?
        // The manual projection used:
        // x = transform.x - camera.x
        // y = transform.y
        // z = transform.z - camera.y (camera.y is treated as Z position?)
        
        // Let's stick to the manual projection's logic for consistency if we can't fully infer
        // But projection_3d uses SceneCamera's matrices.
        
        // Let's just use distance to camera position in 3D space
        // SceneCamera has position (Vec2), distance, rotation, pitch.
        // It orbits around a pivot or looks at a target.
        
        let transform = Transform3D::new(*position, 0.0, Vec2::ONE);
        transform.depth_from_camera(camera)
    }
    
    /// Project tilemap layer to screen space
    pub fn project_tilemap_to_screen(
        &self,
        layer: &TilemapLayer,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
    ) -> Vec<ScreenTile> {
        let mut screen_tiles = Vec::new();
        
        for tile in &layer.tiles {
            if let Some(screen_tile) = self.project_tile_to_screen(tile, camera, viewport_rect, layer.opacity) {
                screen_tiles.push(screen_tile);
            }
        }
        
        screen_tiles
    }
    
    /// Project a single tile to screen space
    fn project_tile_to_screen(
        &self,
        tile: &TileRenderData,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
        layer_opacity: f32,
    ) -> Option<ScreenTile> {
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        // Project center position
        let mut screen_pos = projection_3d::world_to_screen(tile.world_pos, camera, viewport_size)?;
        
        // Apply viewport offset
        screen_pos.x += viewport_rect.min.x;
        screen_pos.y += viewport_rect.min.y;
        
        // Calculate size in screen space
        // We need to project another point to determine scale/size
        // Or use the distance to scale
        let dist = self.calculate_depth_from_camera(&tile.world_pos, camera);
        
        // This is an approximation. Ideally we project 4 corners.
        // Let's project top-right corner to get width/height
        let corner_world = tile.world_pos + Vec3::new(tile.width, tile.height, 0.0);
        let corner_screen = projection_3d::world_to_screen(corner_world, camera, viewport_size)?;
        
        // Calculate dimensions
        // Note: This assumes no rotation for the tile itself (billboard-like or flat on plane)
        // If tiles are on the Z-plane (X-Y plane in 2D), they might be rotated in 3D view.
        // Let's assume they are flat on the world plane.
        
        // Actually, let's just project 3 points to get width and height vectors
        let right_world = tile.world_pos + Vec3::new(tile.width, 0.0, 0.0);
        let up_world = tile.world_pos + Vec3::new(0.0, tile.height, 0.0);
        
        let right_screen = projection_3d::world_to_screen(right_world, camera, viewport_size)?;
        let up_screen = projection_3d::world_to_screen(up_world, camera, viewport_size)?;
        
        let width_vec = right_screen - screen_pos;
        let height_vec = up_screen - screen_pos;
        
        let screen_width = width_vec.length();
        let screen_height = height_vec.length();
        
        // Convert color with layer opacity
        let final_alpha = tile.color[3] * layer_opacity;
        let color = egui::Color32::from_rgba_unmultiplied(
            (tile.color[0] * 255.0) as u8,
            (tile.color[1] * 255.0) as u8,
            (tile.color[2] * 255.0) as u8,
            (final_alpha.clamp(0.0, 1.0) * 255.0) as u8,
        );
        
        Some(ScreenTile {
            screen_pos,
            screen_size: Vec2::new(screen_width, screen_height),
            color,
            depth: dist,
            flip_h: tile.flip_h,
            flip_v: tile.flip_v,
            texture_id: tile.texture_id.clone(),
            tile_rect: tile.tile_rect,
        })
    }
    
    /// Render all tilemaps in 3D mode
    pub fn render(
        &self,
        painter: &egui::Painter,
        layers: &[TilemapLayer],
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
        texture_manager: &mut crate::texture_manager::TextureManager,
        ctx: &egui::Context,
    ) {
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        
        // Render each layer (already sorted by depth)
        for layer in layers {
            if !layer.visible {
                continue;
            }
            
            // Project and render tiles
            let screen_tiles = self.project_tilemap_to_screen(layer, camera, viewport_rect);
            
            for screen_tile in screen_tiles {
                // Create rect for tile
                // Note: In 3D, tiles might be skewed/rotated, but we are rendering axis-aligned rects here.
                // For true 3D, we should use Mesh with 4 vertices.
                // But for now, let's stick to Rect if it's close enough or use Mesh if we have rotation.
                // Since we calculated screen_size from projected vectors, it's an approximation.
                
                let rect = egui::Rect::from_min_size(
                    egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
                    egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
                );
                
                // Try to load and render the actual texture
                let texture_path = std::path::Path::new(&screen_tile.texture_id);
                if let Some(texture_handle) = texture_manager.load_texture(ctx, &screen_tile.texture_id, texture_path) {
                    // Get texture size
                    let tex_size = texture_handle.size_vec2();
                    
                    // Calculate UV coordinates from tile_rect
                    let u_min = screen_tile.tile_rect[0] as f32 / tex_size.x;
                    let v_min = screen_tile.tile_rect[1] as f32 / tex_size.y;
                    let u_max = (screen_tile.tile_rect[0] + screen_tile.tile_rect[2]) as f32 / tex_size.x;
                    let v_max = (screen_tile.tile_rect[1] + screen_tile.tile_rect[3]) as f32 / tex_size.y;
                    
                    // Handle flipping
                    let (u_min, u_max) = if screen_tile.flip_h {
                        (u_max, u_min)
                    } else {
                        (u_min, u_max)
                    };
                    
                    let (v_min, v_max) = if screen_tile.flip_v {
                        (v_max, v_min)
                    } else {
                        (v_min, v_max)
                    };
                    
                    let uv = egui::Rect::from_min_max(
                        egui::pos2(u_min, v_min),
                        egui::pos2(u_max, v_max),
                    );
                    
                    // Render textured tile
                    let mut mesh = egui::Mesh::with_texture(texture_handle.id());
                    mesh.add_rect_with_uv(rect, uv, screen_tile.color);
                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    // Fallback: render as colored rectangle if texture not found
                    painter.rect_filled(rect, 0.0, screen_tile.color);
                    
                    // Add red border to indicate missing texture
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(255, 0, 0, 100)),
                    );
                }
            }
        }
    }
    
    /// Render tilemap bounds for selected/hovered tilemaps
    pub fn render_bounds(
        &self,
        painter: &egui::Painter,
        layer: &TilemapLayer,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
        color: egui::Color32,
    ) {
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        
        // Check for zero-size bounds
        if layer.bounds == egui::Rect::NOTHING || 
           !layer.bounds.is_finite() ||
           layer.bounds.width() <= 0.0 || 
           layer.bounds.height() <= 0.0 {
            // Render as a point instead
            let center_pos = Vec3::new(
                layer.bounds.center().x,
                layer.bounds.center().y,
                layer.z_depth
            );
            
            if let Some(mut screen_pos) = projection_3d::world_to_screen(center_pos, camera, viewport_size) {
                // Apply viewport offset
                screen_pos.x += viewport_rect.min.x;
                screen_pos.y += viewport_rect.min.y;

                painter.circle_stroke(
                    egui::pos2(screen_pos.x, screen_pos.y),
                    4.0,
                    egui::Stroke::new(2.0, color),
                );
            }
            return;
        }
        
        // Project bounds corners to screen space
        let min_pos = Vec3::new(layer.bounds.min.x, layer.bounds.min.y, layer.z_depth);
        let max_pos = Vec3::new(layer.bounds.max.x, layer.bounds.max.y, layer.z_depth);
        
        // Project corners
        let corners = [
            Vec3::new(min_pos.x, min_pos.y, min_pos.z),
            Vec3::new(max_pos.x, min_pos.y, min_pos.z),
            Vec3::new(max_pos.x, max_pos.y, min_pos.z),
            Vec3::new(min_pos.x, max_pos.y, min_pos.z),
        ];
        
        let mut screen_corners = Vec::new();
        for corner in &corners {
            if let Some(mut screen_pos) = projection_3d::world_to_screen(*corner, camera, viewport_size) {
                // Apply viewport offset
                screen_pos.x += viewport_rect.min.x;
                screen_pos.y += viewport_rect.min.y;
                screen_corners.push(egui::pos2(screen_pos.x, screen_pos.y));
            }
        }
        
        // Draw bounds as wireframe
        if screen_corners.len() == 4 {
            for i in 0..4 {
                let start = screen_corners[i];
                let end = screen_corners[(i + 1) % 4];
                painter.line_segment([start, end], egui::Stroke::new(2.0, color));
            }
        }
    }
    
    /// Helper method to project a single point to screen space
    fn project_point_to_screen(
        &self,
        point: &Vec3,
        camera: &SceneCamera,
        viewport_size: Vec2,
    ) -> Option<Vec2> {
        projection_3d::world_to_screen(*point, camera, viewport_size)
    }
    
    /// Set selected tilemaps
    pub fn set_selected(&mut self, entities: HashSet<Entity>) {
        self.selected_tilemaps = entities;
    }
    
    /// Set hovered tilemap
    pub fn set_hovered(&mut self, entity: Option<Entity>) {
        self.hovered_tilemap = entity;
    }
}

impl Default for Tilemap3DRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tilemap_3d_renderer_creation() {
        let renderer = Tilemap3DRenderer::new();
        assert_eq!(renderer.layers.len(), 0);
        assert_eq!(renderer.selected_tilemaps.len(), 0);
        assert!(renderer.hovered_tilemap.is_none());
    }
}

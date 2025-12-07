//! Tilemap 3D Renderer
//!
//! Handles rendering of tilemaps in 3D space with proper depth sorting and projection.

use ecs::{World, Entity};
use egui;
use glam::{Vec2, Vec3};
use std::collections::HashSet;
use crate::editor::SceneCamera;
use super::super::types::Point3D;

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
        // Validate inputs
        if !position.is_finite() {
            eprintln!("Warning: Invalid tile position in depth calculation");
            return f32::MAX; // Treat as far away
        }
        
        if !camera.position.is_finite() {
            eprintln!("Warning: Invalid camera position in depth calculation");
            return f32::MAX;
        }
        
        // Transform position relative to camera
        let relative_pos = Vec3::new(
            position.x - camera.position.x,
            position.y,
            position.z - camera.position.y,
        );
        
        // Validate relative position
        if !relative_pos.is_finite() {
            return f32::MAX;
        }
        
        // Apply camera rotation to get depth in camera space
        let yaw = camera.rotation.to_radians();
        let pitch = camera.pitch.to_radians();
        
        // Validate angles
        if !yaw.is_finite() || !pitch.is_finite() {
            eprintln!("Warning: Invalid camera angles in depth calculation");
            return f32::MAX;
        }
        
        // Rotate around Y axis (yaw)
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = relative_pos.x * cos_yaw + relative_pos.z * sin_yaw;
        let rotated_z = -relative_pos.x * sin_yaw + relative_pos.z * cos_yaw;
        
        // Rotate around X axis (pitch)
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let final_y = relative_pos.y * cos_pitch - rotated_z * sin_pitch;
        let final_z = relative_pos.y * sin_pitch + rotated_z * cos_pitch;
        
        // Validate final depth
        if !final_z.is_finite() {
            return f32::MAX;
        }
        
        // Return Z depth (distance from camera)
        final_z
    }
    
    /// Project tilemap layer to screen space
    pub fn project_tilemap_to_screen(
        &self,
        layer: &TilemapLayer,
        camera: &SceneCamera,
        viewport_center: Vec2,
    ) -> Vec<ScreenTile> {
        let mut screen_tiles = Vec::new();
        
        for tile in &layer.tiles {
            if let Some(screen_tile) = self.project_tile_to_screen(tile, camera, viewport_center, layer.opacity) {
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
        viewport_center: Vec2,
        layer_opacity: f32,
    ) -> Option<ScreenTile> {
        // Validate inputs
        if !tile.world_pos.is_finite() {
            eprintln!("Warning: Invalid tile position in projection");
            return None;
        }
        
        if !camera.position.is_finite() {
            eprintln!("Warning: Invalid camera position in projection");
            return None;
        }
        
        if !viewport_center.is_finite() {
            eprintln!("Warning: Invalid viewport center in projection");
            return None;
        }
        
        if !camera.zoom.is_finite() || camera.zoom <= 0.0 {
            eprintln!("Warning: Invalid camera zoom in projection");
            return None;
        }
        
        if !layer_opacity.is_finite() || layer_opacity < 0.0 || layer_opacity > 1.0 {
            eprintln!("Warning: Invalid layer opacity in projection");
            return None;
        }
        
        // Validate tile dimensions
        if !tile.width.is_finite() || !tile.height.is_finite() {
            return None;
        }
        
        if tile.width <= 0.0 || tile.height <= 0.0 {
            return None;
        }
        
        // Transform position relative to camera
        let pos_3d = Point3D::new(
            tile.world_pos.x - camera.position.x,
            tile.world_pos.y,
            tile.world_pos.z - camera.position.y,
        );
        
        // Apply camera rotation
        let yaw = camera.rotation.to_radians();
        let pitch = camera.pitch.to_radians();
        
        // Validate angles
        if !yaw.is_finite() || !pitch.is_finite() {
            eprintln!("Warning: Invalid camera angles in projection");
            return None;
        }
        
        let rotated = pos_3d
            .rotate_y(-yaw)
            .rotate_x(pitch);
        
        // Check if tile is behind camera
        let distance = 500.0;
        let perspective_z = rotated.z + distance;
        
        // Validate perspective_z
        if !perspective_z.is_finite() {
            return None;
        }
        
        if perspective_z <= 10.0 {
            // Tile is behind camera or too close
            return None;
        }
        
        // Check for extreme distances that could cause overflow
        if perspective_z > 1000000.0 {
            return None;
        }
        
        // Calculate perspective scale
        let scale = (distance / perspective_z) * camera.zoom;
        
        // Validate scale
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        
        // Check for extreme scale values that could cause overflow
        if scale > 10000.0 || scale < 0.0001 {
            return None;
        }
        
        // Project to screen space
        let screen_x = viewport_center.x + rotated.x * scale;
        let screen_y = viewport_center.y + rotated.y * scale;
        
        // Validate screen position
        if !screen_x.is_finite() || !screen_y.is_finite() {
            return None;
        }
        
        // Check for extreme screen positions (likely overflow)
        if screen_x.abs() > 1000000.0 || screen_y.abs() > 1000000.0 {
            return None;
        }
        
        // Calculate screen size
        let screen_width = tile.width * scale;
        let screen_height = tile.height * scale;
        
        // Validate screen size
        if !screen_width.is_finite() || !screen_height.is_finite() {
            return None;
        }
        
        // Check for zero or negative screen size
        if screen_width <= 0.0 || screen_height <= 0.0 {
            return None;
        }
        
        // Check for extreme screen sizes (likely overflow)
        if screen_width > 100000.0 || screen_height > 100000.0 {
            return None;
        }
        
        // Convert color with layer opacity
        let final_alpha = tile.color[3] * layer_opacity;
        
        // Validate final alpha
        if !final_alpha.is_finite() {
            return None;
        }
        
        let color = egui::Color32::from_rgba_unmultiplied(
            (tile.color[0] * 255.0) as u8,
            (tile.color[1] * 255.0) as u8,
            (tile.color[2] * 255.0) as u8,
            (final_alpha.clamp(0.0, 1.0) * 255.0) as u8,
        );
        
        Some(ScreenTile {
            screen_pos: Vec2::new(screen_x, screen_y),
            screen_size: Vec2::new(screen_width, screen_height),
            color,
            depth: perspective_z,
            flip_h: tile.flip_h,
            flip_v: tile.flip_v,
        })
    }
    
    /// Render all tilemaps in 3D mode
    pub fn render(
        &self,
        painter: &egui::Painter,
        layers: &[TilemapLayer],
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
    ) {
        let viewport_center = Vec2::new(
            viewport_rect.center().x,
            viewport_rect.center().y,
        );
        
        // Render each layer (already sorted by depth)
        for layer in layers {
            if !layer.visible {
                continue;
            }
            
            // Project and render tiles
            let screen_tiles = self.project_tilemap_to_screen(layer, camera, viewport_center);
            
            for screen_tile in screen_tiles {
                // Create rect for tile
                let rect = egui::Rect::from_min_size(
                    egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
                    egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
                );
                
                // Render tile as filled rectangle (simplified rendering)
                // In a full implementation, this would render the actual texture
                painter.rect_filled(rect, 0.0, screen_tile.color);
                
                // Add subtle border for visibility
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                );
            }
        }
    }
    
    /// Render tilemap bounds for selected/hovered tilemaps
    pub fn render_bounds(
        &self,
        painter: &egui::Painter,
        layer: &TilemapLayer,
        camera: &SceneCamera,
        viewport_center: Vec2,
        color: egui::Color32,
    ) {
        // Validate inputs
        if !viewport_center.is_finite() {
            eprintln!("Warning: Invalid viewport center in bounds rendering");
            return;
        }
        
        if !camera.position.is_finite() {
            eprintln!("Warning: Invalid camera position in bounds rendering");
            return;
        }
        
        if !camera.zoom.is_finite() || camera.zoom <= 0.0 {
            eprintln!("Warning: Invalid camera zoom in bounds rendering");
            return;
        }
        
        if !layer.z_depth.is_finite() {
            eprintln!("Warning: Invalid layer z_depth in bounds rendering");
            return;
        }
        
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
            
            if let Some(screen_pos) = self.project_point_to_screen(&center_pos, camera, viewport_center) {
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
        
        // Validate bounds positions
        if !min_pos.is_finite() || !max_pos.is_finite() {
            return;
        }
        
        // Project corners
        let corners = [
            Vec3::new(min_pos.x, min_pos.y, min_pos.z),
            Vec3::new(max_pos.x, min_pos.y, min_pos.z),
            Vec3::new(max_pos.x, max_pos.y, min_pos.z),
            Vec3::new(min_pos.x, max_pos.y, min_pos.z),
        ];
        
        let mut screen_corners = Vec::new();
        for corner in &corners {
            if let Some(screen_pos) = self.project_point_to_screen(corner, camera, viewport_center) {
                // Validate screen position
                if !screen_pos.is_finite() {
                    continue;
                }
                
                // Check for extreme screen positions (likely off-screen or overflow)
                if screen_pos.x.abs() > 100000.0 || screen_pos.y.abs() > 100000.0 {
                    continue;
                }
                
                screen_corners.push(egui::pos2(screen_pos.x, screen_pos.y));
            } else {
                // If any corner fails to project, don't render bounds
                return;
            }
        }
        
        // Draw bounds as wireframe
        if screen_corners.len() == 4 {
            for i in 0..4 {
                let start = screen_corners[i];
                let end = screen_corners[(i + 1) % 4];
                
                // Validate line segment
                if !start.is_finite() || !end.is_finite() {
                    continue;
                }
                
                painter.line_segment([start, end], egui::Stroke::new(2.0, color));
            }
        }
    }
    
    /// Helper method to project a single point to screen space
    fn project_point_to_screen(
        &self,
        point: &Vec3,
        camera: &SceneCamera,
        viewport_center: Vec2,
    ) -> Option<Vec2> {
        // Validate inputs
        if !point.is_finite() || !camera.position.is_finite() || !viewport_center.is_finite() {
            return None;
        }
        
        let pos_3d = Point3D::new(
            point.x - camera.position.x,
            point.y,
            point.z - camera.position.y,
        );
        
        let yaw = camera.rotation.to_radians();
        let pitch = camera.pitch.to_radians();
        
        // Validate angles
        if !yaw.is_finite() || !pitch.is_finite() {
            return None;
        }
        
        let rotated = pos_3d.rotate_y(-yaw).rotate_x(pitch);
        
        let distance = 500.0;
        let perspective_z = rotated.z + distance;
        
        // Validate perspective_z
        if !perspective_z.is_finite() || perspective_z <= 10.0 {
            return None;
        }
        
        // Check for extreme distances
        if perspective_z > 1000000.0 {
            return None;
        }
        
        let scale = (distance / perspective_z) * camera.zoom;
        
        // Validate scale
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        
        let screen_x = viewport_center.x + rotated.x * scale;
        let screen_y = viewport_center.y + rotated.y * scale;
        
        // Validate screen position
        if !screen_x.is_finite() || !screen_y.is_finite() {
            return None;
        }
        
        Some(Vec2::new(screen_x, screen_y))
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
    
    #[test]
    fn test_calculate_depth_from_camera() {
        let renderer = Tilemap3DRenderer::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Tile in front of camera (positive Z)
        let pos = Vec3::new(0.0, 0.0, 10.0);
        let depth = renderer.calculate_depth_from_camera(&pos, &camera);
        assert!(depth > 0.0, "Tile in front should have positive depth");
        
        // Tile behind camera (negative Z)
        let pos = Vec3::new(0.0, 0.0, -10.0);
        let depth = renderer.calculate_depth_from_camera(&pos, &camera);
        assert!(depth < 0.0, "Tile behind should have negative depth");
    }
    
    #[test]
    fn test_depth_sort_layers() {
        let mut renderer = Tilemap3DRenderer::new();
        
        let mut layers = vec![
            TilemapLayer {
                entity: 1,
                z_depth: 10.0,
                tiles: Vec::new(),
                bounds: egui::Rect::NOTHING,
                name: "layer1".to_string(),
                opacity: 1.0,
                visible: true,
            },
            TilemapLayer {
                entity: 2,
                z_depth: 5.0,
                tiles: Vec::new(),
                bounds: egui::Rect::NOTHING,
                name: "layer2".to_string(),
                opacity: 1.0,
                visible: true,
            },
            TilemapLayer {
                entity: 3,
                z_depth: 15.0,
                tiles: Vec::new(),
                bounds: egui::Rect::NOTHING,
                name: "layer3".to_string(),
                opacity: 1.0,
                visible: true,
            },
        ];
        
        renderer.depth_sort_layers(&mut layers);
        
        // Should be sorted in descending order (farther first)
        assert_eq!(layers[0].z_depth, 15.0);
        assert_eq!(layers[1].z_depth, 10.0);
        assert_eq!(layers[2].z_depth, 5.0);
    }
}

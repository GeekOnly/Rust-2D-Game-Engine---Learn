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
    pub texture_path: String,  // Full path to texture file
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
    pub fn collect_tilemaps(&mut self, world: &World, tilemap_settings: Option<&crate::editor::tilemap_settings::TilemapSettings>) -> Vec<TilemapLayer> {
        // Use pixels_per_unit from tilemap settings, or default to LDtk-compatible value
        // For proper grid alignment: use 8.0 (8px = 1 world unit = 1 grid cell)
        let pixels_per_unit = tilemap_settings
            .map(|s| s.pixels_per_unit)
            .unwrap_or(8.0); // Default to LDtk-compatible value for grid alignment
        
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
            // Calculate tile size in world units using pixels_per_unit
            // This ensures tilemap aligns perfectly with grid
            let tile_world_width = tileset.tile_width as f32 / pixels_per_unit;
            let tile_world_height = tileset.tile_height as f32 / pixels_per_unit;
            
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
                        
                        // Calculate world position in world units (not pixels)
                        // Note: Y coordinate is already flipped in the transform from LDtk loader
                        // We invert Y in 3D view to match 2D Y-Down coordinate system (Visual Fix)
                        let tile_world_x = transform.position[0] + (x as f32 * tile_world_width);
                        let tile_world_y = -(transform.position[1] + (y as f32 * tile_world_height));
                        let tile_world_z = transform.position[2];
                        
                        // Update bounds
                        min_x = min_x.min(tile_world_x);
                        min_y = min_y.min(tile_world_y);
                        // Since we flip Y, [y + height] is essentially [y_base - height] visually? 
                        // Actually, just using min/max on the calculated corner points is safer if we project.
                        // But here we construct bounds from top-left.
                        // If tile_world_y is Top-Left (visual). And height extends "down" (visually).
                        // In 3D Y-Up, "down" is -Y. 
                        // tile_world_y is the anchor.
                        // We construct the tile as a rect.
                        // We should probably explicitly set max_y logic or just let the loop handle it
                        // But wait, the tile rect rendering logic uses width/height.
                        // If we invert Y, does it grow up or down?
                        // Render logic later: `project_tile_to_screen` -> `projected_corners`.
                        // corners use `y - half_height`, `y + half_height`.
                        // So `tile_world_y` represents the CENTER?
                        // No, the calculation above: `pos + x*w`. This is Top-Left of the tile in Grid space.
                        // In `project_tile_to_screen`, it uses `tile.world_pos`.
                        // It projects corners: `pos.y - half_height`.
                        // This implies `tile.world_pos` is treated as CENTER.
                        // BUT `tile_world_x` above is calculated as `index * width`. This is Corner!
                        // We should shift it to CENTER?
                        
                        // Let's check existing code lines 143+:
                        // tiles.push(TileRenderData { world_pos: ... })
                        // And project_tile_to_screen: corners use `world_pos +/- half_width`.
                        // This proves `world_pos` represents CENTER in `project_tile_to_screen`.
                        // BUT `tile_world_x` here calculates TOP-LEFT (Grid Corner).
                        
                        // So we must add half-width/height offset!
                        // Currently existing code: `let tile_world_x = transform.position[0] + (x as f32 * tile_world_width);`
                        // If this is passed as "world_pos", and then treated as center...
                        // THEN THE TILES ARE SHIFTED BY HALF SIZE!
                        // This might be another bug? Or intentional?
                        // Usually Grid (0,0) is center of tile (0,0)? No, usually corner.
                        
                        // If `TileRenderData` expects CENTER (based on corner projection logic), we must convert GridCorner to Center.
                        // offset_x = tile_world_width / 2.0; offset_y = tile_world_height / 2.0;
                        
                        // Let's Fix the Y Inversion first.
                        // And adjust Center offset if needed.
                        // Given user complaint about "incorrect coordinate", maybe the Shift is also part of it.
                        // Let's assume the previous code "worked" for X?
                        
                        // Modifying:
                        let half_w = tile_world_width * 0.5;
                        let half_h = tile_world_height * 0.5;
                        
                        let tile_world_x = transform.position[0] + (x as f32 * tile_world_width) + half_w;
                        let tile_world_y = -(transform.position[1] + (y as f32 * tile_world_height)) - half_h; // Growing Down (Negative Y)
                        let tile_world_z = transform.position[2];

                        // Update bounds
                        min_x = min_x.min(tile_world_x - half_w);
                        min_y = min_y.min(tile_world_y - half_h);
                        max_x = max_x.max(tile_world_x + half_w);
                        max_y = max_y.max(tile_world_y + half_h);
                        
                        // Create tile render data (width/height in world units)
                        tiles.push(TileRenderData {
                            world_pos: Vec3::new(tile_world_x, tile_world_y, tile_world_z),
                            texture_id: tileset.texture_path.clone(),  // Use texture_path instead of texture_id
                            tile_rect: [
                                tile_coords.0,
                                tile_coords.1,
                                tileset.tile_width,
                                tileset.tile_height,
                            ],
                            color: [1.0, 1.0, 1.0, tilemap.opacity],
                            flip_h: tile.flip_h,
                            flip_v: tile.flip_v,
                            width: tile_world_width,
                            height: tile_world_height,
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
        
        // Validate tile position first
        if !tile.world_pos.is_finite() {
            eprintln!("Warning: Invalid tile world position: {:?}", tile.world_pos);
            return None;
        }
        
        // Project center position
        let mut screen_pos = projection_3d::world_to_screen(tile.world_pos, camera, viewport_size)?;
        
        // Validate screen position
        if !screen_pos.is_finite() {
            return None;
        }
        
        // Apply viewport offset
        screen_pos.x += viewport_rect.min.x;
        screen_pos.y += viewport_rect.min.y;
        
        // Calculate size in screen space using distance-based scaling
        // This is more stable than projecting multiple points
        let dist = self.calculate_depth_from_camera(&tile.world_pos, camera);
        
        // Calculate proper screen size by projecting tile corners
        // This ensures tiles connect properly and have correct size
        let tile_half_width = tile.width / 2.0;
        let tile_half_height = tile.height / 2.0;
        
        // Project tile corners to get accurate screen size
        let corner_positions = [
            Vec3::new(tile.world_pos.x - tile_half_width, tile.world_pos.y - tile_half_height, tile.world_pos.z),
            Vec3::new(tile.world_pos.x + tile_half_width, tile.world_pos.y - tile_half_height, tile.world_pos.z),
            Vec3::new(tile.world_pos.x + tile_half_width, tile.world_pos.y + tile_half_height, tile.world_pos.z),
            Vec3::new(tile.world_pos.x - tile_half_width, tile.world_pos.y + tile_half_height, tile.world_pos.z),
        ];
        
        let mut projected_corners = Vec::new();
        for corner in corner_positions {
            if let Some(screen_corner) = projection_3d::world_to_screen(corner, camera, viewport_size) {
                projected_corners.push(screen_corner);
            }
        }
        
        // If we can't project all corners, fall back to distance-based scaling
        let (screen_width, screen_height) = if projected_corners.len() == 4 {
            // Calculate screen size from projected corners
            let min_x = projected_corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = projected_corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = projected_corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = projected_corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
            
            let width = (max_x - min_x).abs();
            let height = (max_y - min_y).abs();
            
            (width, height)
        } else {
            // Fallback: use distance-based scaling with proper base scale
            let scale_factor = if dist > 0.1 {
                // Use camera distance for perspective scaling
                // Adjust base scale to match grid size better
                let base_scale = 50.0; // Reduced from 100.0 for better size
                base_scale / dist.max(0.1)
            } else {
                50.0
            };
            
            (tile.width * scale_factor, tile.height * scale_factor)
        };
        
        // Validate calculated dimensions
        if !screen_width.is_finite() || !screen_height.is_finite() || 
           screen_width <= 0.0 || screen_height <= 0.0 {
            return None;
        }
        
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
            texture_path: tile.texture_id.clone(),  // texture_id already contains the path
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
                
                // screen_pos is the center position, so we need to use from_center_size
                let rect = egui::Rect::from_center_size(
                    egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
                    egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
                );
                
                // Try to load and render the actual texture
                let texture_path = std::path::Path::new(&screen_tile.texture_path);
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

//! Sprite 3D Renderer
//!
//! Handles rendering of sprites in 3D space with proper depth sorting and projection.

use ecs::{World, Entity};
use egui;
use glam::{Vec2, Vec3, Quat};
use std::collections::HashSet;
use crate::SceneCamera;
use super::projection_3d::{self, Transform3D};

/// Sprite 3D renderer for rendering sprites in 3D mode
pub struct Sprite3DRenderer {
    /// Billboard settings
    pub enable_billboard: bool,
    
    /// Depth sorted sprites (entity, depth)
    depth_sorted_sprites: Vec<(Entity, f32)>,
    
    /// Selected entities
    selected_entities: HashSet<Entity>,
    
    /// Hovered entity
    hovered_entity: Option<Entity>,
}

/// Sprite render data for 3D rendering
#[derive(Clone, Debug)]
pub struct SpriteRenderData {
    pub entity: Entity,
    pub position: Vec3,
    pub rotation: Quat, // Changed from f32 to Quat for 3D rotation
    pub scale: Vec2,
    pub texture_id: String,
    pub sprite_rect: Option<[u32; 4]>,
    pub color: [f32; 4],
    pub billboard: bool,
    pub width: f32,
    pub height: f32,
}

/// Screen-space sprite data after projection
#[derive(Clone, Debug)]
pub struct ScreenSprite {
    pub screen_pos: Vec2,
    pub screen_size: Vec2,
    pub rotation: f32, // Screen space rotation (Z) is still f32 usually, but let's calculate properly
    pub color: egui::Color32,
    pub depth: f32,
    pub entity: Entity,
}

impl Sprite3DRenderer {
    /// Create a new Sprite3DRenderer
    pub fn new() -> Self {
        Self {
            enable_billboard: false,
            depth_sorted_sprites: Vec::new(),
            selected_entities: HashSet::new(),
            hovered_entity: None,
        }
    }
    
    /// Collect all sprites from the world
    pub fn collect_sprites(&mut self, world: &World) -> Vec<SpriteRenderData> {
        let mut sprites = Vec::new();
        
        for (&entity, sprite) in world.sprites.iter() {
            if let Some(transform) = world.transforms.get(&entity) {
                // Validate transform data
                if !Self::is_valid_transform(transform) {
                    eprintln!("Warning: Invalid transform for sprite entity {:?}, skipping", entity);
                    continue;
                }
                
                let position = Vec3::new(
                    transform.position[0],
                    transform.position[1],
                    transform.position[2],
                );
                
                // Get full 3D rotation
                let rot_rad = Vec3::new(
                    transform.rotation[0].to_radians(),
                    transform.rotation[1].to_radians(),
                    transform.rotation[2].to_radians(),
                );
                let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                
                let scale = Vec2::new(
                    transform.scale[0],
                    transform.scale[1],
                );
                
                // Validate sprite data
                if !Self::is_valid_sprite_data(sprite, &scale) {
                    eprintln!("Warning: Invalid sprite data for entity {:?}, skipping", entity);
                    continue;
                }
                
                sprites.push(SpriteRenderData {
                    entity,
                    position,
                    rotation,
                    scale,
                    texture_id: sprite.texture_id.clone(),
                    sprite_rect: sprite.sprite_rect,
                    color: sprite.color,
                    billboard: sprite.billboard,
                    // Convert pixel dimensions to world units using PPU
                    // Default to 16.0 PPU if for some reason it's 0 to avoid division by zero
                    width: sprite.width / sprite.pixels_per_unit.max(1.0),
                    height: sprite.height / sprite.pixels_per_unit.max(1.0),
                });
            }
        }
        
        sprites
    }
    
    /// Validate transform data
    fn is_valid_transform(transform: &ecs::Transform) -> bool {
        // Check position is finite
        if !transform.position[0].is_finite() || 
           !transform.position[1].is_finite() || 
           !transform.position[2].is_finite() {
            return false;
        }
        
        // Check rotation is finite
        if !transform.rotation[0].is_finite() || 
           !transform.rotation[1].is_finite() || 
           !transform.rotation[2].is_finite() {
            return false;
        }
        
        // Check scale is finite and positive
        if !transform.scale[0].is_finite() || 
           !transform.scale[1].is_finite() || 
           !transform.scale[2].is_finite() {
            return false;
        }
        
        true
    }
    
    /// Validate sprite data (textures, rectangles, scales)
    fn is_valid_sprite_data(sprite: &ecs::Sprite, scale: &Vec2) -> bool {
        // Check texture ID is not empty
        if sprite.texture_id.is_empty() {
            return false;
        }
        
        // Check sprite dimensions are valid
        if !sprite.width.is_finite() || sprite.width <= 0.0 {
            return false;
        }
        if !sprite.height.is_finite() || sprite.height <= 0.0 {
            return false;
        }
        
        // Check sprite dimensions are within reasonable bounds (0.001 to 10000)
        if sprite.width < 0.001 || sprite.width > 10000.0 {
            return false;
        }
        if sprite.height < 0.001 || sprite.height > 10000.0 {
            return false;
        }
        
        // Check scale is valid and within reasonable bounds (0.001 to 1000.0)
        if !scale.x.is_finite() || !scale.y.is_finite() {
            return false;
        }
        if scale.x < 0.001 || scale.x > 1000.0 {
            return false;
        }
        if scale.y < 0.001 || scale.y > 1000.0 {
            return false;
        }
        
        // Check sprite rect if present
        if let Some(rect) = sprite.sprite_rect {
            // Validate rect dimensions are positive (u32 values are always finite)
            if rect[2] == 0 || rect[3] == 0 {
                return false;
            }
        }
        
        // Check color values are valid (0.0 to 1.0)
        for &c in &sprite.color {
            if !c.is_finite() || c < 0.0 || c > 1.0 {
                return false;
            }
        }
        
        true
    }
    
    /// Sort sprites by depth (Z position) - farther sprites first
    pub fn depth_sort(&mut self, sprites: &mut Vec<SpriteRenderData>, camera: &SceneCamera) {
        // Calculate depth for each sprite relative to camera
        self.depth_sorted_sprites.clear();
        
        for sprite in sprites.iter() {
            let depth = self.calculate_depth_from_camera(&sprite.position, camera);
            
            // Handle NaN/Inf depths - treat as far plane
            let safe_depth = if !depth.is_finite() {
                f32::MAX
            } else {
                depth
            };
            
            self.depth_sorted_sprites.push((sprite.entity, safe_depth));
        }
        
        // Sort sprites by depth (farther first for painter's algorithm)
        sprites.sort_by(|a, b| {
            let depth_a = self.calculate_depth_from_camera(&a.position, camera);
            let depth_b = self.calculate_depth_from_camera(&b.position, camera);
            
            // Handle NaN/Inf depths - treat as far plane for sorting
            let safe_depth_a = if !depth_a.is_finite() { f32::MAX } else { depth_a };
            let safe_depth_b = if !depth_b.is_finite() { f32::MAX } else { depth_b };
            
            // Sort in descending order (farther first)
            // Use total_cmp for consistent ordering even with special values
            safe_depth_b.total_cmp(&safe_depth_a)
        });
    }
    
    /// Calculate depth of a sprite from camera
    pub fn calculate_depth_from_camera(&self, position: &Vec3, camera: &SceneCamera) -> f32 {
        let transform = Transform3D::new(*position, 0.0, Vec2::ONE);
        transform.depth_from_camera(camera)
    }
    
    /// Calculate billboard rotation for a sprite to face the camera
    pub fn calculate_billboard_rotation(&self, sprite_pos: Vec3, camera: &SceneCamera) -> f32 {
        // Validate inputs
        if !sprite_pos.is_finite() {
            eprintln!("Warning: Invalid sprite position in billboard calculation");
            return 0.0;
        }
        
        if !camera.position.is_finite() {
            eprintln!("Warning: Invalid camera position in billboard calculation");
            return 0.0;
        }
        
        // Calculate vector from sprite to camera
        let to_camera = Vec2::new(
            camera.position.x - sprite_pos.x,
            camera.position.y - sprite_pos.z,
        );
        
        // Validate to_camera vector
        if !to_camera.is_finite() {
            return 0.0;
        }
        
        // Handle case where camera is at same position as sprite
        let distance = to_camera.length();
        if !distance.is_finite() || distance < 0.001 {
            return 0.0;
        }
        
        // Calculate rotation angle to face camera
        let angle = to_camera.y.atan2(to_camera.x);
        
        // Validate angle is finite
        if !angle.is_finite() {
            return 0.0;
        }
        
        // Clamp to [-π, π] range
        let clamped_angle = angle.rem_euclid(std::f32::consts::TAU);
        let final_angle = if clamped_angle > std::f32::consts::PI {
            clamped_angle - std::f32::consts::TAU
        } else {
            clamped_angle
        };
        
        // Final validation
        if !final_angle.is_finite() {
            return 0.0;
        }
        
        final_angle
    }
    
    /// Project sprite to screen space
    pub fn project_sprite_to_screen(
        &self,
        sprite: &SpriteRenderData,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
    ) -> Option<ScreenSprite> {
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        
        // Project center position (without viewport offset for size calculation)
        let center_screen = projection_3d::world_to_screen(sprite.position, camera, viewport_size)?;
        
        // Calculate rotation (billboard or world rotation)
        let rotation_quat = if sprite.billboard {
            // If billboard, we ignore the world rotation and face the camera
            // But for bounds calculation we need the effective rotation
             // For now, let's assume billboard means "Z-plane facing camera" logic or similar.
             // Actually, billboard rotation calculation returns f32 (Z angle).
             let z_angle = self.calculate_billboard_rotation(sprite.position, camera);
             Quat::from_rotation_z(z_angle)
        } else {
            sprite.rotation
        };

        // Calculate size in screen space by projecting corner points
        // Use world-space offsets based on sprite dimensions, ROTATED by the sprite's rotation
        let half_width = sprite.width * sprite.scale.x * 0.5;
        let half_height = sprite.height * sprite.scale.y * 0.5;
        
        // Corners in local space
        let right_vec = Vec3::new(half_width, 0.0, 0.0);
        let up_vec = Vec3::new(0.0, half_height, 0.0);
        
        // Rotate corners to world space
        let right_world_offset = rotation_quat * right_vec;
        let up_world_offset = rotation_quat * up_vec;
        
        // We use full width/height extent for bounds
        // Let's project 4 corners to find the bounding box?
        // Or just project Right and Up to estimate size?
        // Projecting Right and Up only works if aligned with screen axes or if we want axes vectors.
        // For a generic 2D rect bounds on screen, we should project all 4 corners and find min/max.
        
        let p1 = sprite.position - right_world_offset - up_world_offset; // BL
        let p2 = sprite.position + right_world_offset - up_world_offset; // BR
        let p3 = sprite.position + right_world_offset + up_world_offset; // TR
        let p4 = sprite.position - right_world_offset + up_world_offset; // TL
        
        let s1 = projection_3d::world_to_screen(p1, camera, viewport_size);
        let s2 = projection_3d::world_to_screen(p2, camera, viewport_size);
        let s3 = projection_3d::world_to_screen(p3, camera, viewport_size);
        let s4 = projection_3d::world_to_screen(p4, camera, viewport_size);
        
        // If any point is behind camera, we might have issues. 
        // world_to_screen returns None if behind.
        // If some are visible, we should try to render?
        // For simplicity, require center to be visible (checked above).
        
        let points = [s1, s2, s3, s4];
        let valid_points: Vec<Vec2> = points.iter().filter_map(|p| *p).collect();
        
        if valid_points.is_empty() {
             return None;
        }
        
        // Find AABB of projected points
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        for p in valid_points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
        
        let screen_width = max_x - min_x;
        let screen_height = max_y - min_y;
        
        // Update screen pos to center of AABB
        let mut screen_pos = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
        screen_pos.x += viewport_rect.min.x;
        screen_pos.y += viewport_rect.min.y;
        
        // Calculate screen-space rotation (approximate for display)
        // If we are drawing an AABB, rotation is effectively 0.
        // If we want to rotate the texture in UI overlay, we need projected angle.
        // But render_bounds draws an AABB stroke. So rotation 0 is fine for bounds.
        // For the fallback texture rendering (render method), it might need rotation.
        // But for Selection Box, AABB is safer.
        let screen_rotation = 0.0; 
        
        // Convert color
        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );
        
        let dist = self.calculate_depth_from_camera(&sprite.position, camera);
        
        Some(ScreenSprite {
            screen_pos,
            screen_size: Vec2::new(screen_width, screen_height),
            rotation: screen_rotation,
            color,
            depth: dist,
            entity: sprite.entity,
        })
    }
    
    /// Render all sprites in 3D mode
    pub fn render(
        &self,
        painter: &egui::Painter,
        sprites: &[SpriteRenderData],
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
        texture_manager: &mut engine::texture_manager::TextureManager,
        ctx: &egui::Context,
    ) {
        let _viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        
        // Project and render each sprite
        for sprite in sprites {
            if let Some(screen_sprite) = self.project_sprite_to_screen(sprite, camera, viewport_rect) {
                // Create rect for sprite
                let rect = egui::Rect::from_center_size(
                    egui::pos2(screen_sprite.screen_pos.x, screen_sprite.screen_pos.y),
                    egui::vec2(screen_sprite.screen_size.x, screen_sprite.screen_size.y),
                );
                
                // Try to load and render the actual texture
                let texture_path = std::path::Path::new(&sprite.texture_id);
                if let Some(texture_handle) = texture_manager.load_texture(ctx, &sprite.texture_id, texture_path) {
                    // Calculate UV coordinates if sprite_rect is specified
                    let uv = if let Some(sprite_rect) = sprite.sprite_rect {
                        // Get texture size
                        let tex_size = texture_handle.size_vec2();
                        
                        // Calculate UV coordinates
                        let u_min = sprite_rect[0] as f32 / tex_size.x;
                        let v_min = sprite_rect[1] as f32 / tex_size.y;
                        let u_max = (sprite_rect[0] + sprite_rect[2]) as f32 / tex_size.x;
                        let v_max = (sprite_rect[1] + sprite_rect[3]) as f32 / tex_size.y;
                        
                        egui::Rect::from_min_max(
                            egui::pos2(u_min, v_min),
                            egui::pos2(u_max, v_max),
                        )
                    } else {
                        // Use full texture
                        egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0),
                        )
                    };
                    
                    // Render textured sprite
                    let mut mesh = egui::Mesh::with_texture(texture_handle.id());
                    mesh.add_rect_with_uv(rect, uv, screen_sprite.color);
                    
                    // Apply rotation if needed
                    if screen_sprite.rotation != 0.0 {
                        mesh.rotate(
                            egui::emath::Rot2::from_angle(screen_sprite.rotation), 
                            rect.center()
                        );
                    }
                    
                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    // Fallback: render as colored rectangle if texture not found
                    painter.rect_filled(rect, 2.0, screen_sprite.color);
                    
                    // Add subtle border to indicate missing texture
                    painter.rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 0, 0, 100)),
                    );
                }
            }
        }
    }
    
    /// Render sprite bounds for selected/hovered sprites
    pub fn render_bounds(
        &self,
        painter: &egui::Painter,
        sprite: &SpriteRenderData,
        camera: &SceneCamera,
        viewport_rect: egui::Rect,
        color: egui::Color32,
    ) {
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
        
        // Calculate 4 corners in World Space
        let half_width = sprite.width * sprite.scale.x * 0.5;
        let half_height = sprite.height * sprite.scale.y * 0.5;

        // Vertices in local space (centered at 0,0, on XY plane)
        let p1 = Vec3::new(-half_width, -half_height, 0.0); // BL
        let p2 = Vec3::new(half_width, -half_height, 0.0);  // BR
        let p3 = Vec3::new(half_width, half_height, 0.0);   // TR
        let p4 = Vec3::new(-half_width, half_height, 0.0);  // TL

        // Calculate rotation
        let rotation_quat = if sprite.billboard {
             // If billboard, assume Z rotation facing camera (simplified for now to match other logic)
             let z_angle = self.calculate_billboard_rotation(sprite.position, camera);
             Quat::from_rotation_z(z_angle)
        } else {
            sprite.rotation
        };

        // Transform to World Space
        let transform_point = |p: Vec3| -> Vec3 {
            sprite.position + (rotation_quat * p)
        };

        let w1 = transform_point(p1);
        let w2 = transform_point(p2);
        let w3 = transform_point(p3);
        let w4 = transform_point(p4);

        // Project to Screen Space
        // Note: world_to_screen returns None if point is behind camera
        let s1 = projection_3d::world_to_screen(w1, camera, viewport_size);
        let s2 = projection_3d::world_to_screen(w2, camera, viewport_size);
        let s3 = projection_3d::world_to_screen(w3, camera, viewport_size);
        let s4 = projection_3d::world_to_screen(w4, camera, viewport_size);
        
        // Collect valid points
        let corners = [s1, s2, s3, s4];
        
        // Only render if we have points. Ideally we'd clip lines against near plane, 
        // but for a simple gizmo, checking if points are visible is a decent start.
        // If some points are behind, we might get weird lines, but world_to_screen handles basic projection.
        
        // Helper to convert to absolute screen pos
        let to_screen_pos = |p: Vec2| -> egui::Pos2 {
            egui::pos2(viewport_rect.min.x + p.x, viewport_rect.min.y + p.y)
        };

        // Draw quad edges
        // We draw lines between 0-1, 1-2, 2-3, 3-0 if both endpoints are valid.
        // Or if one is invalid, we could clip, but skipping is safer for now to avoid artifacts.
        
        let edges = [(0, 1), (1, 2), (2, 3), (3, 0)];
        let stroke = egui::Stroke::new(2.0, color);
        
        for (i, j) in edges {
            if let (Some(c1), Some(c2)) = (corners[i], corners[j]) {
                painter.line_segment([to_screen_pos(c1), to_screen_pos(c2)], stroke);
            }
        }
    }
    
    /// Set selected entities
    pub fn set_selected(&mut self, entities: HashSet<Entity>) {
        self.selected_entities = entities;
    }
    
    /// Set hovered entity
    pub fn set_hovered(&mut self, entity: Option<Entity>) {
        self.hovered_entity = entity;
    }
}

impl Default for Sprite3DRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_3d_renderer_creation() {
        let renderer = Sprite3DRenderer::new();
        assert!(!renderer.enable_billboard);
        assert_eq!(renderer.depth_sorted_sprites.len(), 0);
    }
}

//! Sprite 3D Renderer
//!
//! Handles rendering of sprites in 3D space with proper depth sorting and projection.

use ecs::{World, Entity};
use egui;
use glam::{Vec2, Vec3};
use std::collections::HashSet;
use crate::editor::SceneCamera;
use super::super::types::Point3D;

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
    pub rotation: f32,
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
    pub rotation: f32,
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
                
                let rotation = transform.rotation[2]; // Z rotation for 2D sprites
                
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
                    width: sprite.width,
                    height: sprite.height,
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
        // Validate inputs
        if !position.is_finite() {
            eprintln!("Warning: Invalid sprite position in depth calculation");
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
        viewport_center: Vec2,
    ) -> Option<ScreenSprite> {
        // Validate inputs
        if !sprite.position.is_finite() {
            eprintln!("Warning: Invalid sprite position in projection");
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
        
        // Transform position relative to camera
        let pos_3d = Point3D::new(
            sprite.position.x - camera.position.x,
            sprite.position.y,
            sprite.position.z - camera.position.y,
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
        
        // Check if sprite is behind camera
        let distance = 500.0;
        let perspective_z = rotated.z + distance;
        
        // Validate perspective_z
        if !perspective_z.is_finite() {
            return None;
        }
        
        if perspective_z <= 10.0 {
            // Sprite is behind camera or too close
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
        // In 3D mode, sprite dimensions are in world units, not pixels
        // We need to scale them appropriately for the perspective view
        // Assuming sprite.width/height are in pixels, we convert to world units
        let world_scale = 0.1; // Scale factor to convert pixels to world units
        let screen_width = sprite.width * sprite.scale.x * scale * world_scale;
        let screen_height = sprite.height * sprite.scale.y * scale * world_scale;
        
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
        
        // Calculate rotation (billboard or world rotation)
        let rotation = if sprite.billboard {
            self.calculate_billboard_rotation(sprite.position, camera)
        } else {
            sprite.rotation
        };
        
        // Validate rotation
        if !rotation.is_finite() {
            eprintln!("Warning: Invalid rotation in projection");
            return None;
        }
        
        // Convert color
        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );
        
        Some(ScreenSprite {
            screen_pos: Vec2::new(screen_x, screen_y),
            screen_size: Vec2::new(screen_width, screen_height),
            rotation,
            color,
            depth: perspective_z,
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
        texture_manager: &mut crate::texture_manager::TextureManager,
        ctx: &egui::Context,
    ) {
        let viewport_center = Vec2::new(
            viewport_rect.center().x,
            viewport_rect.center().y,
        );
        
        // Project and render each sprite
        for sprite in sprites {
            if let Some(screen_sprite) = self.project_sprite_to_screen(sprite, camera, viewport_center) {
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
        viewport_center: Vec2,
        color: egui::Color32,
    ) {
        // Validate inputs
        if !viewport_center.is_finite() {
            eprintln!("Warning: Invalid viewport center in bounds rendering");
            return;
        }
        
        if let Some(screen_sprite) = self.project_sprite_to_screen(sprite, camera, viewport_center) {
            // Validate screen sprite data
            if !screen_sprite.screen_pos.is_finite() || !screen_sprite.screen_size.is_finite() {
                eprintln!("Warning: Invalid screen sprite data in bounds rendering");
                return;
            }
            
            // Check for zero-size bounds
            if screen_sprite.screen_size.x <= 0.0 || screen_sprite.screen_size.y <= 0.0 {
                // Render as a point instead
                painter.circle_stroke(
                    egui::pos2(screen_sprite.screen_pos.x, screen_sprite.screen_pos.y),
                    4.0,
                    egui::Stroke::new(2.0, color),
                );
                return;
            }
            
            // Calculate bounds with padding
            let bounds_width = screen_sprite.screen_size.x + 4.0;
            let bounds_height = screen_sprite.screen_size.y + 4.0;
            
            // Validate bounds dimensions
            if !bounds_width.is_finite() || !bounds_height.is_finite() {
                return;
            }
            
            // Check for extreme bounds (likely off-screen or overflow)
            if bounds_width > 100000.0 || bounds_height > 100000.0 {
                return;
            }
            
            let rect = egui::Rect::from_center_size(
                egui::pos2(screen_sprite.screen_pos.x, screen_sprite.screen_pos.y),
                egui::vec2(bounds_width, bounds_height),
            );
            
            // Validate rect
            if !rect.is_finite() {
                return;
            }
            
            painter.rect_stroke(
                rect,
                2.0,
                egui::Stroke::new(2.0, color),
            );
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
    
    #[test]
    fn test_calculate_depth_from_camera() {
        let renderer = Sprite3DRenderer::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Sprite in front of camera (positive Z)
        let pos = Vec3::new(0.0, 0.0, 10.0);
        let depth = renderer.calculate_depth_from_camera(&pos, &camera);
        assert!(depth > 0.0, "Sprite in front should have positive depth");
        
        // Sprite behind camera (negative Z)
        let pos = Vec3::new(0.0, 0.0, -10.0);
        let depth = renderer.calculate_depth_from_camera(&pos, &camera);
        assert!(depth < 0.0, "Sprite behind should have negative depth");
    }
    
    #[test]
    fn test_calculate_billboard_rotation() {
        let renderer = Sprite3DRenderer::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(10.0, 0.0);
        
        let sprite_pos = Vec3::new(0.0, 0.0, 0.0);
        let rotation = renderer.calculate_billboard_rotation(sprite_pos, &camera);
        
        // Rotation should be finite
        assert!(rotation.is_finite());
        
        // Rotation should be in [-π, π] range
        assert!(rotation >= -std::f32::consts::PI && rotation <= std::f32::consts::PI);
    }
    
    #[test]
    fn test_calculate_billboard_rotation_same_position() {
        let renderer = Sprite3DRenderer::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        
        let sprite_pos = Vec3::new(0.0, 0.0, 0.0);
        let rotation = renderer.calculate_billboard_rotation(sprite_pos, &camera);
        
        // Should return 0.0 when camera is at same position
        assert_eq!(rotation, 0.0);
    }
}

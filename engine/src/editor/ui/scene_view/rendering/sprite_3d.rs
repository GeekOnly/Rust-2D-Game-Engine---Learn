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
    
    /// Sort sprites by depth (Z position) - farther sprites first
    pub fn depth_sort(&mut self, sprites: &mut Vec<SpriteRenderData>, camera: &SceneCamera) {
        // Calculate depth for each sprite relative to camera
        self.depth_sorted_sprites.clear();
        
        for sprite in sprites.iter() {
            let depth = self.calculate_depth_from_camera(&sprite.position, camera);
            self.depth_sorted_sprites.push((sprite.entity, depth));
        }
        
        // Sort sprites by depth (farther first for painter's algorithm)
        sprites.sort_by(|a, b| {
            let depth_a = self.calculate_depth_from_camera(&a.position, camera);
            let depth_b = self.calculate_depth_from_camera(&b.position, camera);
            // Sort in descending order (farther first)
            depth_b.partial_cmp(&depth_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    /// Calculate depth of a sprite from camera
    pub fn calculate_depth_from_camera(&self, position: &Vec3, camera: &SceneCamera) -> f32 {
        // Transform position relative to camera
        let relative_pos = Vec3::new(
            position.x - camera.position.x,
            position.y,
            position.z - camera.position.y,
        );
        
        // Apply camera rotation to get depth in camera space
        let yaw = camera.rotation.to_radians();
        let pitch = camera.pitch.to_radians();
        
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
        
        // Return Z depth (distance from camera)
        final_z
    }
    
    /// Calculate billboard rotation for a sprite to face the camera
    fn calculate_billboard_rotation(&self, sprite_pos: Vec3, camera: &SceneCamera) -> f32 {
        // Calculate vector from sprite to camera
        let to_camera = Vec2::new(
            camera.position.x - sprite_pos.x,
            camera.position.y - sprite_pos.z,
        );
        
        // Handle case where camera is at same position as sprite
        if to_camera.length() < 0.001 {
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
        if clamped_angle > std::f32::consts::PI {
            clamped_angle - std::f32::consts::TAU
        } else {
            clamped_angle
        }
    }
    
    /// Project sprite to screen space
    fn project_sprite_to_screen(
        &self,
        sprite: &SpriteRenderData,
        camera: &SceneCamera,
        viewport_center: Vec2,
    ) -> Option<ScreenSprite> {
        // Transform position relative to camera
        let pos_3d = Point3D::new(
            sprite.position.x - camera.position.x,
            sprite.position.y,
            sprite.position.z - camera.position.y,
        );
        
        // Apply camera rotation
        let yaw = camera.rotation.to_radians();
        let pitch = camera.pitch.to_radians();
        let rotated = pos_3d
            .rotate_y(-yaw)
            .rotate_x(pitch);
        
        // Check if sprite is behind camera
        let distance = 500.0;
        let perspective_z = rotated.z + distance;
        
        if perspective_z <= 10.0 {
            // Sprite is behind camera or too close
            return None;
        }
        
        // Calculate perspective scale
        let scale = (distance / perspective_z) * camera.zoom;
        
        // Validate scale
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        
        // Project to screen space
        let screen_x = viewport_center.x + rotated.x * scale;
        let screen_y = viewport_center.y + rotated.y * scale;
        
        // Validate screen position
        if !screen_x.is_finite() || !screen_y.is_finite() {
            return None;
        }
        
        // Calculate screen size
        let screen_width = sprite.width * sprite.scale.x * scale;
        let screen_height = sprite.height * sprite.scale.y * scale;
        
        // Validate screen size
        if !screen_width.is_finite() || !screen_height.is_finite() {
            return None;
        }
        
        // Calculate rotation (billboard or world rotation)
        let rotation = if sprite.billboard {
            self.calculate_billboard_rotation(sprite.position, camera)
        } else {
            sprite.rotation
        };
        
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
                
                // Render sprite as filled rectangle (simplified rendering)
                // In a full implementation, this would render the actual texture
                painter.rect_filled(rect, 2.0, screen_sprite.color);
                
                // Add subtle border
                painter.rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 50)),
                );
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
        if let Some(screen_sprite) = self.project_sprite_to_screen(sprite, camera, viewport_center) {
            let rect = egui::Rect::from_center_size(
                egui::pos2(screen_sprite.screen_pos.x, screen_sprite.screen_pos.y),
                egui::vec2(screen_sprite.screen_size.x + 4.0, screen_sprite.screen_size.y + 4.0),
            );
            
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

//! Render Queue Module
//!
//! Manages the rendering order of all objects in the scene with proper depth sorting.
//! Ensures sprites, tilemaps, grid, and gizmos render in the correct order.

use ecs::Entity;
use crate::SceneCamera;
use super::sprite_3d::SpriteRenderData;
use super::tilemap_3d::TilemapLayer;

/// Render queue for managing render order of all objects
pub struct RenderQueue {
    /// Queue of all renderable objects sorted by depth
    objects: Vec<RenderObject>,
}

/// Enumeration of all renderable object types
#[derive(Clone, Debug)]
pub enum RenderObject {
    /// Grid rendering (always rendered first/farthest)
    Grid,
    /// Sprite with render data
    Sprite(SpriteRenderData),
    /// Tilemap layer with render data
    Tilemap(TilemapLayer),
    /// Gizmo data (always rendered last/closest)
    Gizmo(GizmoData),
}

/// Gizmo render data
#[derive(Clone, Debug)]
pub struct GizmoData {
    pub entity: Entity,
    pub depth: f32,
    pub gizmo_type: GizmoType,
}

/// Types of gizmos
#[derive(Clone, Debug, PartialEq)]
pub enum GizmoType {
    /// Transform gizmo (move, rotate, scale)
    Transform,
    /// Selection bounds
    SelectionBounds,
    /// Hover highlight
    HoverHighlight,
}

impl RenderQueue {
    /// Create a new empty render queue
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
    
    /// Add an object to the render queue
    pub fn push(&mut self, object: RenderObject) {
        self.objects.push(object);
    }
    
    /// Sort all objects by depth (back to front for transparency)
    /// 
    /// Sorting order:
    /// 1. Grid (always farthest)
    /// 2. Sprites and tilemaps sorted by Z depth (farther first)
    /// 3. Gizmos (always closest)
    pub fn sort_by_depth(&mut self, camera: &SceneCamera) {
        // Clone camera position and rotation to avoid borrowing issues
        let camera_pos = camera.position;
        let camera_rotation = camera.rotation;
        let camera_pitch = camera.pitch;
        
        self.objects.sort_by(|a, b| {
            let depth_a = Self::get_object_depth_static(a, camera_pos, camera_rotation, camera_pitch);
            let depth_b = Self::get_object_depth_static(b, camera_pos, camera_rotation, camera_pitch);
            
            // Sort in descending order (farther first for painter's algorithm)
            depth_b.partial_cmp(&depth_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    /// Get the depth of a render object (static version for sorting)
    fn get_object_depth_static(
        object: &RenderObject,
        camera_pos: glam::Vec2,
        camera_rotation: f32,
        camera_pitch: f32,
    ) -> f32 {
        match object {
            RenderObject::Grid => {
                // Grid is always rendered first (farthest)
                f32::MAX
            }
            RenderObject::Sprite(sprite) => {
                // Calculate sprite depth from camera
                Self::calculate_sprite_depth_static(&sprite.position, camera_pos, camera_rotation, camera_pitch)
            }
            RenderObject::Tilemap(layer) => {
                // Use layer's Z depth
                layer.z_depth
            }
            RenderObject::Gizmo(gizmo) => {
                // Gizmos are always rendered last (closest)
                // Use negative depth to ensure they're in front
                -1000.0 - gizmo.depth
            }
        }
    }
    
    /// Calculate depth of a sprite from camera (static version)
    fn calculate_sprite_depth_static(
        position: &glam::Vec3,
        camera_pos: glam::Vec2,
        camera_rotation: f32,
        camera_pitch: f32,
    ) -> f32 {
        // Transform position relative to camera
        let relative_pos = glam::Vec3::new(
            position.x - camera_pos.x,
            position.y,
            position.z - camera_pos.y,
        );
        
        // Apply camera rotation to get depth in camera space
        let yaw = camera_rotation.to_radians();
        let pitch = camera_pitch.to_radians();
        
        // Rotate around Y axis (yaw)
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let _rotated_x = relative_pos.x * cos_yaw + relative_pos.z * sin_yaw;
        let rotated_z = -relative_pos.x * sin_yaw + relative_pos.z * cos_yaw;
        
        // Rotate around X axis (pitch)
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let _final_y = relative_pos.y * cos_pitch - rotated_z * sin_pitch;
        let final_z = relative_pos.y * sin_pitch + rotated_z * cos_pitch;
        
        // Return Z depth (distance from camera)
        final_z
    }
    
    /// Clear the render queue
    pub fn clear(&mut self) {
        self.objects.clear();
    }
    
    /// Get sorted objects for rendering
    pub fn get_sorted(&self) -> &[RenderObject] {
        &self.objects
    }
    
    /// Get mutable reference to sorted objects
    pub fn get_sorted_mut(&mut self) -> &mut [RenderObject] {
        &mut self.objects
    }
    
    /// Get the number of objects in the queue
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
    
    /// Calculate depth of a sprite from camera (public version for testing)
    pub fn calculate_sprite_depth(&self, position: &glam::Vec3, camera: &SceneCamera) -> f32 {
        Self::calculate_sprite_depth_static(position, camera.position, camera.rotation, camera.pitch)
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3};
    
    #[test]
    fn test_render_queue_creation() {
        let queue = RenderQueue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_render_queue_push() {
        let mut queue = RenderQueue::new();
        queue.push(RenderObject::Grid);
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }
    
    #[test]
    fn test_render_queue_clear() {
        let mut queue = RenderQueue::new();
        queue.push(RenderObject::Grid);
        queue.push(RenderObject::Grid);
        assert_eq!(queue.len(), 2);
        
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_depth_sorting_grid_first() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add objects in random order
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 1,
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        queue.push(RenderObject::Grid);
        
        queue.sort_by_depth(&camera);
        
        // Grid should be first (farthest)
        let sorted = queue.get_sorted();
        assert!(matches!(sorted[0], RenderObject::Grid));
    }
    
    #[test]
    fn test_depth_sorting_gizmos_last() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add objects in random order
        queue.push(RenderObject::Gizmo(GizmoData {
            entity: 1,
            depth: 0.0,
            gizmo_type: GizmoType::Transform,
        }));
        queue.push(RenderObject::Grid);
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 2,
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        
        queue.sort_by_depth(&camera);
        
        // Gizmo should be last (closest)
        let sorted = queue.get_sorted();
        assert!(matches!(sorted[sorted.len() - 1], RenderObject::Gizmo(_)));
    }
    
    #[test]
    fn test_depth_sorting_sprites_by_z() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add sprites at different Z depths
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 1,
            position: Vec3::new(0.0, 0.0, 5.0), // Closer
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test1".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 2,
            position: Vec3::new(0.0, 0.0, 15.0), // Farther
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test2".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        
        queue.sort_by_depth(&camera);
        
        // Farther sprite should be first
        let sorted = queue.get_sorted();
        if let RenderObject::Sprite(sprite) = &sorted[0] {
            assert_eq!(sprite.entity, 2, "Farther sprite should be rendered first");
        } else {
            panic!("Expected sprite");
        }
        
        if let RenderObject::Sprite(sprite) = &sorted[1] {
            assert_eq!(sprite.entity, 1, "Closer sprite should be rendered second");
        } else {
            panic!("Expected sprite");
        }
    }
    
    #[test]
    fn test_depth_sorting_tilemaps_by_z() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add tilemap layers at different Z depths
        queue.push(RenderObject::Tilemap(TilemapLayer {
            entity: 1,
            z_depth: 5.0, // Closer
            tiles: Vec::new(),
            bounds: egui::Rect::NOTHING,
            name: "layer1".to_string(),
            opacity: 1.0,
            visible: true,
        }));
        queue.push(RenderObject::Tilemap(TilemapLayer {
            entity: 2,
            z_depth: 15.0, // Farther
            tiles: Vec::new(),
            bounds: egui::Rect::NOTHING,
            name: "layer2".to_string(),
            opacity: 1.0,
            visible: true,
        }));
        
        queue.sort_by_depth(&camera);
        
        // Farther tilemap should be first
        let sorted = queue.get_sorted();
        if let RenderObject::Tilemap(layer) = &sorted[0] {
            assert_eq!(layer.entity, 2, "Farther tilemap should be rendered first");
        } else {
            panic!("Expected tilemap");
        }
        
        if let RenderObject::Tilemap(layer) = &sorted[1] {
            assert_eq!(layer.entity, 1, "Closer tilemap should be rendered second");
        } else {
            panic!("Expected tilemap");
        }
    }
    
    #[test]
    fn test_depth_sorting_mixed_objects() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add mixed objects
        queue.push(RenderObject::Gizmo(GizmoData {
            entity: 1,
            depth: 0.0,
            gizmo_type: GizmoType::Transform,
        }));
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 2,
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        queue.push(RenderObject::Grid);
        queue.push(RenderObject::Tilemap(TilemapLayer {
            entity: 3,
            z_depth: 5.0,
            tiles: Vec::new(),
            bounds: egui::Rect::NOTHING,
            name: "layer".to_string(),
            opacity: 1.0,
            visible: true,
        }));
        
        queue.sort_by_depth(&camera);
        
        // Check order: Grid, Sprite/Tilemap (by depth), Gizmo
        let sorted = queue.get_sorted();
        assert!(matches!(sorted[0], RenderObject::Grid), "Grid should be first");
        assert!(matches!(sorted[sorted.len() - 1], RenderObject::Gizmo(_)), "Gizmo should be last");
    }
    
    #[test]
    fn test_calculate_sprite_depth() {
        let queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Sprite in front of camera (positive Z)
        let pos = Vec3::new(0.0, 0.0, 10.0);
        let depth = queue.calculate_sprite_depth(&pos, &camera);
        assert!(depth > 0.0, "Sprite in front should have positive depth");
        
        // Sprite behind camera (negative Z)
        let pos = Vec3::new(0.0, 0.0, -10.0);
        let depth = queue.calculate_sprite_depth(&pos, &camera);
        assert!(depth < 0.0, "Sprite behind should have negative depth");
    }
    
    #[test]
    fn test_depth_sorting_handles_nan() {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add sprite with NaN position (should be handled gracefully)
        queue.push(RenderObject::Sprite(SpriteRenderData {
            entity: 1,
            position: Vec3::new(f32::NAN, 0.0, 10.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            texture_id: "test".to_string(),
            sprite_rect: None,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            width: 32.0,
            height: 32.0,
        }));
        queue.push(RenderObject::Grid);
        
        // Should not panic
        queue.sort_by_depth(&camera);
        assert_eq!(queue.len(), 2);
    }
}

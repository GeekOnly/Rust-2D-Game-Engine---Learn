//! Unified Depth Sorting System
//!
//! This module provides depth sorting functionality for mixed 2D/3D content rendering.
//! It handles depth buffer integration, manual sort order support, and proper rendering
//! order calculation for sprites, tilemaps, and 3D objects.

use glam::Vec3;
use ecs::{Entity, Transform};
use ecs::components::{UnifiedSprite, UnifiedTilemap, ViewMode};

/// Renderable item type for depth sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderableType {
    /// 2D sprite
    Sprite,
    /// 2D tilemap
    Tilemap,
    /// 3D mesh
    Mesh3D,
}

/// Renderable item with depth information for sorting
#[derive(Debug, Clone)]
pub struct RenderableItem {
    /// Entity ID
    pub entity: Entity,
    /// Type of renderable
    pub renderable_type: RenderableType,
    /// World space position
    pub position: Vec3,
    /// Calculated depth value for sorting
    pub depth: f32,
    /// Manual sort order (for sprites)
    pub sort_order: i32,
    /// Whether the item has transparency
    pub has_transparency: bool,
    /// Layer depth (for tilemaps)
    pub layer_depth: f32,
}

impl RenderableItem {
    /// Create a new renderable item for a sprite
    pub fn sprite(
        entity: Entity,
        transform: &Transform,
        sprite: &UnifiedSprite,
        camera_position: Vec3,
        view_mode: ViewMode,
    ) -> Self {
        let position = Vec3::from_slice(&transform.position);
        let depth = Self::calculate_depth(position, camera_position, view_mode);
        let has_transparency = sprite.color[3] < 1.0;

        Self {
            entity,
            renderable_type: RenderableType::Sprite,
            position,
            depth,
            sort_order: sprite.sort_order,
            has_transparency,
            layer_depth: 0.0,
        }
    }

    /// Create a new renderable item for a tilemap
    pub fn tilemap(
        entity: Entity,
        transform: &Transform,
        tilemap: &UnifiedTilemap,
        camera_position: Vec3,
        view_mode: ViewMode,
    ) -> Self {
        let position = Vec3::from_slice(&transform.position);
        let depth = Self::calculate_depth(position, camera_position, view_mode);

        Self {
            entity,
            renderable_type: RenderableType::Tilemap,
            position,
            depth,
            sort_order: 0, // Tilemaps don't use manual sort order
            has_transparency: false, // Tilemaps are typically opaque
            layer_depth: tilemap.layer_depth,
        }
    }

    /// Create a new renderable item for a 3D mesh
    pub fn mesh_3d(
        entity: Entity,
        transform: &Transform,
        camera_position: Vec3,
        has_transparency: bool,
    ) -> Self {
        let position = Vec3::from_slice(&transform.position);
        let depth = Self::calculate_depth(position, camera_position, ViewMode::Mode3D);

        Self {
            entity,
            renderable_type: RenderableType::Mesh3D,
            position,
            depth,
            sort_order: 0,
            has_transparency,
            layer_depth: 0.0,
        }
    }

    /// Calculate depth value based on camera position and view mode
    fn calculate_depth(position: Vec3, camera_position: Vec3, view_mode: ViewMode) -> f32 {
        match view_mode {
            ViewMode::Mode2D => {
                // In 2D mode, use Z position directly
                position.z
            }
            ViewMode::Mode3D => {
                // In 3D mode, use distance from camera
                position.distance(camera_position)
            }
        }
    }
}

/// Depth sorting system for unified 2D/3D rendering
pub struct DepthSortingSystem {
    /// Cached renderable items
    renderables: Vec<RenderableItem>,
    /// Current view mode
    view_mode: ViewMode,
    /// Camera position for depth calculations
    camera_position: Vec3,
}

impl DepthSortingSystem {
    /// Create a new depth sorting system
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            view_mode: ViewMode::Mode2D,
            camera_position: Vec3::ZERO,
        }
    }

    /// Update camera position and view mode
    pub fn update_camera(&mut self, position: Vec3, view_mode: ViewMode) {
        self.camera_position = position;
        self.view_mode = view_mode;
    }

    /// Clear all renderables
    pub fn clear(&mut self) {
        self.renderables.clear();
    }

    /// Add a sprite to the rendering queue
    pub fn add_sprite(
        &mut self,
        entity: Entity,
        transform: &Transform,
        sprite: &UnifiedSprite,
    ) {
        let item = RenderableItem::sprite(
            entity,
            transform,
            sprite,
            self.camera_position,
            self.view_mode,
        );
        self.renderables.push(item);
    }

    /// Add a tilemap to the rendering queue
    pub fn add_tilemap(
        &mut self,
        entity: Entity,
        transform: &Transform,
        tilemap: &UnifiedTilemap,
    ) {
        let item = RenderableItem::tilemap(
            entity,
            transform,
            tilemap,
            self.camera_position,
            self.view_mode,
        );
        self.renderables.push(item);
    }

    /// Add a 3D mesh to the rendering queue
    pub fn add_mesh_3d(
        &mut self,
        entity: Entity,
        transform: &Transform,
        has_transparency: bool,
    ) {
        let item = RenderableItem::mesh_3d(
            entity,
            transform,
            self.camera_position,
            has_transparency,
        );
        self.renderables.push(item);
    }

    /// Sort all renderables by depth and return the sorted list
    pub fn sort_and_get_renderables(&mut self) -> &[RenderableItem] {
        self.sort_renderables();
        &self.renderables
    }

    /// Sort renderables using the unified depth sorting algorithm
    fn sort_renderables(&mut self) {
        // Separate opaque and transparent objects
        let (mut opaque, mut transparent): (Vec<_>, Vec<_>) = self
            .renderables
            .drain(..)
            .partition(|item| !item.has_transparency);

        // Sort opaque objects by depth (back to front for 2D, front to back for 3D optimization)
        match self.view_mode {
            ViewMode::Mode2D => {
                // In 2D mode, sort back to front (painter's algorithm)
                opaque.sort_by(|a, b| {
                    // Primary: layer depth (for tilemaps)
                    a.layer_depth
                        .partial_cmp(&b.layer_depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        // Secondary: Z position
                        .then_with(|| {
                            a.depth
                                .partial_cmp(&b.depth)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        // Tertiary: manual sort order
                        .then_with(|| a.sort_order.cmp(&b.sort_order))
                        // Quaternary: entity ID for stable sorting
                        .then_with(|| a.entity.cmp(&b.entity))
                });
            }
            ViewMode::Mode3D => {
                // In 3D mode, opaque objects can be sorted front to back for early Z rejection
                // But we'll use back to front for consistency with transparency
                opaque.sort_by(|a, b| {
                    a.depth
                        .partial_cmp(&b.depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.sort_order.cmp(&b.sort_order))
                        .then_with(|| a.entity.cmp(&b.entity))
                });
            }
        }

        // Sort transparent objects back to front (required for correct blending)
        transparent.sort_by(|a, b| {
            // Primary: layer depth (for tilemaps)
            a.layer_depth
                .partial_cmp(&b.layer_depth)
                .unwrap_or(std::cmp::Ordering::Equal)
                // Secondary: depth (back to front)
                .then_with(|| {
                    a.depth
                        .partial_cmp(&b.depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                // Tertiary: manual sort order
                .then_with(|| a.sort_order.cmp(&b.sort_order))
                // Quaternary: entity ID for stable sorting
                .then_with(|| a.entity.cmp(&b.entity))
        });

        // Combine: render opaque first, then transparent
        self.renderables.clear();
        self.renderables.extend(opaque);
        self.renderables.extend(transparent);
    }

    /// Get renderables by type
    pub fn get_renderables_by_type(&self, renderable_type: RenderableType) -> Vec<&RenderableItem> {
        self.renderables
            .iter()
            .filter(|item| item.renderable_type == renderable_type)
            .collect()
    }

    /// Get all sprite renderables
    pub fn get_sprites(&self) -> Vec<&RenderableItem> {
        self.get_renderables_by_type(RenderableType::Sprite)
    }

    /// Get all tilemap renderables
    pub fn get_tilemaps(&self) -> Vec<&RenderableItem> {
        self.get_renderables_by_type(RenderableType::Tilemap)
    }

    /// Get all 3D mesh renderables
    pub fn get_meshes_3d(&self) -> Vec<&RenderableItem> {
        self.get_renderables_by_type(RenderableType::Mesh3D)
    }

    /// Get the number of renderables
    pub fn len(&self) -> usize {
        self.renderables.len()
    }

    /// Check if there are no renderables
    pub fn is_empty(&self) -> bool {
        self.renderables.is_empty()
    }

    /// Get all renderables (sorted)
    pub fn get_all_renderables(&self) -> &[RenderableItem] {
        &self.renderables
    }
}

impl Default for DepthSortingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transform(x: f32, y: f32, z: f32) -> Transform {
        Transform {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    fn create_test_sprite(sort_order: i32, alpha: f32) -> UnifiedSprite {
        UnifiedSprite {
            texture_id: "test".to_string(),
            width: 1.0,
            height: 1.0,
            color: [1.0, 1.0, 1.0, alpha],
            billboard: false,
            world_space_ui: false,
            pixel_perfect: true,
            sort_order,
            pixels_per_unit: None,
        }
    }

    #[test]
    fn test_depth_sorting_2d_mode() {
        let mut system = DepthSortingSystem::new();
        system.update_camera(Vec3::new(0.0, 0.0, -10.0), ViewMode::Mode2D);

        // Add sprites at different Z positions
        let entity1 = 1;
        let entity2 = 2;
        let entity3 = 3;

        system.add_sprite(
            entity1,
            &create_test_transform(0.0, 0.0, 5.0),
            &create_test_sprite(0, 1.0),
        );
        system.add_sprite(
            entity2,
            &create_test_transform(0.0, 0.0, 0.0),
            &create_test_sprite(0, 1.0),
        );
        system.add_sprite(
            entity3,
            &create_test_transform(0.0, 0.0, -5.0),
            &create_test_sprite(0, 1.0),
        );

        let sorted = system.sort_and_get_renderables();

        // Should be sorted back to front (increasing Z)
        assert_eq!(sorted[0].entity, entity3); // Z = -5
        assert_eq!(sorted[1].entity, entity2); // Z = 0
        assert_eq!(sorted[2].entity, entity1); // Z = 5
    }

    #[test]
    fn test_manual_sort_order() {
        let mut system = DepthSortingSystem::new();
        system.update_camera(Vec3::new(0.0, 0.0, -10.0), ViewMode::Mode2D);

        // Add sprites at same Z but different sort orders
        let entity1 = 1;
        let entity2 = 2;
        let entity3 = 3;

        system.add_sprite(
            entity1,
            &create_test_transform(0.0, 0.0, 0.0),
            &create_test_sprite(10, 1.0),
        );
        system.add_sprite(
            entity2,
            &create_test_transform(0.0, 0.0, 0.0),
            &create_test_sprite(5, 1.0),
        );
        system.add_sprite(
            entity3,
            &create_test_transform(0.0, 0.0, 0.0),
            &create_test_sprite(15, 1.0),
        );

        let sorted = system.sort_and_get_renderables();

        // Should be sorted by sort_order
        assert_eq!(sorted[0].entity, entity2); // sort_order = 5
        assert_eq!(sorted[1].entity, entity1); // sort_order = 10
        assert_eq!(sorted[2].entity, entity3); // sort_order = 15
    }

    #[test]
    fn test_transparency_sorting() {
        let mut system = DepthSortingSystem::new();
        system.update_camera(Vec3::new(0.0, 0.0, -10.0), ViewMode::Mode2D);

        // Add opaque and transparent sprites
        let opaque1 = 1;
        let opaque2 = 2;
        let transparent1 = 3;
        let transparent2 = 4;

        system.add_sprite(
            opaque1,
            &create_test_transform(0.0, 0.0, 5.0),
            &create_test_sprite(0, 1.0),
        );
        system.add_sprite(
            transparent1,
            &create_test_transform(0.0, 0.0, 3.0),
            &create_test_sprite(0, 0.5),
        );
        system.add_sprite(
            opaque2,
            &create_test_transform(0.0, 0.0, 1.0),
            &create_test_sprite(0, 1.0),
        );
        system.add_sprite(
            transparent2,
            &create_test_transform(0.0, 0.0, -1.0),
            &create_test_sprite(0, 0.5),
        );

        let sorted = system.sort_and_get_renderables();

        // Opaque objects should come first
        assert!(!sorted[0].has_transparency);
        assert!(!sorted[1].has_transparency);
        // Transparent objects should come last
        assert!(sorted[2].has_transparency);
        assert!(sorted[3].has_transparency);
    }

    #[test]
    fn test_tilemap_layer_depth() {
        let mut system = DepthSortingSystem::new();
        system.update_camera(Vec3::new(0.0, 0.0, -10.0), ViewMode::Mode2D);

        let entity1 = 1;
        let entity2 = 2;
        let entity3 = 3;

        let mut tilemap1 = UnifiedTilemap::default();
        tilemap1.layer_depth = 0.0;

        let mut tilemap2 = UnifiedTilemap::default();
        tilemap2.layer_depth = 10.0;

        let mut tilemap3 = UnifiedTilemap::default();
        tilemap3.layer_depth = 5.0;

        system.add_tilemap(entity1, &create_test_transform(0.0, 0.0, 0.0), &tilemap1);
        system.add_tilemap(entity2, &create_test_transform(0.0, 0.0, 0.0), &tilemap2);
        system.add_tilemap(entity3, &create_test_transform(0.0, 0.0, 0.0), &tilemap3);

        let sorted = system.sort_and_get_renderables();

        // Should be sorted by layer_depth
        assert_eq!(sorted[0].entity, entity1); // layer_depth = 0.0
        assert_eq!(sorted[1].entity, entity3); // layer_depth = 5.0
        assert_eq!(sorted[2].entity, entity2); // layer_depth = 10.0
    }
}

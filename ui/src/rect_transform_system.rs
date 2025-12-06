//! RectTransform calculation system
//!
//! This module provides systems for calculating world-space positions and sizes
//! of UI elements based on their RectTransform components and parent-child relationships.

use glam::Vec2;
use crate::{RectTransform, Rect};
use std::collections::HashMap;

/// Entity ID type (matches ecs crate)
pub type Entity = u64;

/// RectTransform calculation system
///
/// This system calculates world-space corners and rects for all RectTransforms
/// in the hierarchy, handling parent-child relationships and dirty flagging.
pub struct RectTransformSystem {
    /// Cached parent size for each entity
    parent_sizes: HashMap<Entity, Vec2>,
}

impl RectTransformSystem {
    /// Create a new RectTransform system
    pub fn new() -> Self {
        Self {
            parent_sizes: HashMap::new(),
        }
    }

    /// Update all RectTransforms in the hierarchy
    ///
    /// This should be called once per frame to recalculate transforms that are dirty.
    ///
    /// # Arguments
    /// * `rect_transforms` - Mutable map of entity to RectTransform
    /// * `parents` - Map of child entity to parent entity
    /// * `screen_size` - Current screen size (for root elements)
    pub fn update(
        &mut self,
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        parents: &HashMap<Entity, Entity>,
        screen_size: Vec2,
    ) {
        // Clear cached parent sizes
        self.parent_sizes.clear();

        // Find all root entities (entities without parents)
        let mut roots: Vec<Entity> = rect_transforms
            .keys()
            .filter(|e| !parents.contains_key(e))
            .copied()
            .collect();

        // Sort for deterministic order
        roots.sort();

        // Update each root and its children recursively
        for root in roots {
            self.update_recursive(root, screen_size, rect_transforms, parents);
        }
    }

    /// Recursively update a RectTransform and its children
    fn update_recursive(
        &mut self,
        entity: Entity,
        parent_size: Vec2,
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        parents: &HashMap<Entity, Entity>,
    ) {
        // Get the RectTransform (if it doesn't exist, skip)
        let rect_transform = match rect_transforms.get_mut(&entity) {
            Some(rt) => rt,
            None => return,
        };

        // Only recalculate if dirty
        if !rect_transform.dirty {
            // Still need to update children with cached parent size
            if let Some(cached_size) = self.parent_sizes.get(&entity).copied() {
                let children: Vec<Entity> = parents
                    .iter()
                    .filter(|(_, &p)| p == entity)
                    .map(|(&c, _)| c)
                    .collect();

                for child in children {
                    self.update_recursive(child, cached_size, rect_transforms, parents);
                }
            }
            return;
        }

        // Calculate the element's size based on anchors
        let size = Self::calculate_size(rect_transform, parent_size);

        // Calculate anchor points in parent space
        let anchor_min_pos = parent_size * rect_transform.anchor_min;

        // Calculate the rect position and size
        let rect_pos = anchor_min_pos + rect_transform.anchored_position - size * rect_transform.pivot;
        let rect_size = size * rect_transform.scale;

        // Update the rect
        rect_transform.rect = Rect::new(rect_pos.x, rect_pos.y, rect_size.x, rect_size.y);

        // Calculate world corners (bottom-left, top-left, top-right, bottom-right)
        rect_transform.world_corners = [
            Vec2::new(rect_pos.x, rect_pos.y),                           // Bottom-left
            Vec2::new(rect_pos.x, rect_pos.y + rect_size.y),            // Top-left
            Vec2::new(rect_pos.x + rect_size.x, rect_pos.y + rect_size.y), // Top-right
            Vec2::new(rect_pos.x + rect_size.x, rect_pos.y),            // Bottom-right
        ];

        // Mark as clean
        rect_transform.dirty = false;

        // Cache this element's size for its children
        self.parent_sizes.insert(entity, rect_size);

        // Update children recursively
        let children: Vec<Entity> = parents
            .iter()
            .filter(|(_, &p)| p == entity)
            .map(|(&c, _)| c)
            .collect();

        for child in children {
            self.update_recursive(child, rect_size, rect_transforms, parents);
        }
    }

    /// Calculate the size of a RectTransform based on its anchors and parent size
    fn calculate_size(rect_transform: &RectTransform, parent_size: Vec2) -> Vec2 {
        let anchor_min = rect_transform.anchor_min;
        let anchor_max = rect_transform.anchor_max;

        // Calculate the size based on anchor stretch
        let anchor_size = (anchor_max - anchor_min) * parent_size;

        // Add size_delta to get final size
        anchor_size + rect_transform.size_delta
    }

    /// Mark a RectTransform and all its descendants as dirty
    ///
    /// This should be called when a RectTransform's properties change.
    pub fn mark_dirty(
        entity: Entity,
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        parents: &HashMap<Entity, Entity>,
    ) {
        // Mark this entity as dirty
        if let Some(rt) = rect_transforms.get_mut(&entity) {
            rt.dirty = true;
        }

        // Mark all children as dirty recursively
        let children: Vec<Entity> = parents
            .iter()
            .filter(|(_, &p)| p == entity)
            .map(|(&c, _)| c)
            .collect();

        for child in children {
            Self::mark_dirty(child, rect_transforms, parents);
        }
    }

    /// Mark all RectTransforms as dirty
    ///
    /// This should be called when the screen size changes.
    pub fn mark_all_dirty(rect_transforms: &mut HashMap<Entity, RectTransform>) {
        for rt in rect_transforms.values_mut() {
            rt.dirty = true;
        }
    }
}

impl Default for RectTransformSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_size_fixed_anchors() {
        let rt = RectTransform::anchored(Vec2::new(0.5, 0.5), Vec2::ZERO, Vec2::new(100.0, 50.0));
        let parent_size = Vec2::new(800.0, 600.0);

        let size = RectTransformSystem::calculate_size(&rt, parent_size);
        assert_eq!(size, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_calculate_size_stretched_anchors() {
        let rt = RectTransform::stretched(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            glam::Vec4::new(10.0, 10.0, 10.0, 10.0),
        );
        let parent_size = Vec2::new(800.0, 600.0);

        let size = RectTransformSystem::calculate_size(&rt, parent_size);
        // Anchor size = (1.0 - 0.0) * 800 = 800, (1.0 - 0.0) * 600 = 600
        // Size delta = -20, -20
        // Final size = 780, 580
        assert_eq!(size, Vec2::new(780.0, 580.0));
    }

    #[test]
    fn test_update_single_element() {
        let mut system = RectTransformSystem::new();
        let mut rect_transforms = HashMap::new();
        let parents = HashMap::new();

        let entity = 1;
        let rt = RectTransform::anchored(Vec2::new(0.5, 0.5), Vec2::ZERO, Vec2::new(100.0, 50.0));
        rect_transforms.insert(entity, rt);

        let screen_size = Vec2::new(800.0, 600.0);
        system.update(&mut rect_transforms, &parents, screen_size);

        let rt = rect_transforms.get(&entity).unwrap();
        assert!(!rt.dirty);
        // Anchor at center (400, 300), pivot at center (0.5, 0.5)
        // Position = anchor - size * pivot = (400, 300) - (100, 50) * (0.5, 0.5) = (350, 275)
        assert_eq!(rt.rect.x, 350.0);
        assert_eq!(rt.rect.y, 275.0);
        assert_eq!(rt.rect.width, 100.0);
        assert_eq!(rt.rect.height, 50.0);
    }

    #[test]
    fn test_update_parent_child() {
        let mut system = RectTransformSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut parents = HashMap::new();

        let parent_entity = 1;
        let child_entity = 2;

        // Parent: centered, 400x300
        let parent_rt = RectTransform::anchored(
            Vec2::new(0.5, 0.5),
            Vec2::ZERO,
            Vec2::new(400.0, 300.0),
        );
        rect_transforms.insert(parent_entity, parent_rt);

        // Child: top-left corner of parent, 100x50
        let child_rt = RectTransform::anchored(
            Vec2::new(0.0, 1.0),
            Vec2::ZERO,
            Vec2::new(100.0, 50.0),
        );
        rect_transforms.insert(child_entity, child_rt);

        parents.insert(child_entity, parent_entity);

        let screen_size = Vec2::new(800.0, 600.0);
        system.update(&mut rect_transforms, &parents, screen_size);

        let parent_rt = rect_transforms.get(&parent_entity).unwrap();
        assert!(!parent_rt.dirty);

        let child_rt = rect_transforms.get(&child_entity).unwrap();
        assert!(!child_rt.dirty);
        // Child should be positioned relative to parent's size (400x300)
        // Anchor at top-left of parent (0, 300), pivot at center (0.5, 0.5)
        // Position = anchor - size * pivot = (0, 300) - (100, 50) * (0.5, 0.5) = (-50, 275)
        assert_eq!(child_rt.rect.x, -50.0);
        assert_eq!(child_rt.rect.y, 275.0);
    }

    #[test]
    fn test_mark_dirty() {
        let mut rect_transforms = HashMap::new();
        let mut parents = HashMap::new();

        let parent_entity = 1;
        let child_entity = 2;

        let mut parent_rt = RectTransform::default();
        parent_rt.dirty = false;
        rect_transforms.insert(parent_entity, parent_rt);

        let mut child_rt = RectTransform::default();
        child_rt.dirty = false;
        rect_transforms.insert(child_entity, child_rt);

        parents.insert(child_entity, parent_entity);

        RectTransformSystem::mark_dirty(parent_entity, &mut rect_transforms, &parents);

        assert!(rect_transforms.get(&parent_entity).unwrap().dirty);
        assert!(rect_transforms.get(&child_entity).unwrap().dirty);
    }

    #[test]
    fn test_mark_all_dirty() {
        let mut rect_transforms = HashMap::new();

        for i in 1..=5 {
            let mut rt = RectTransform::default();
            rt.dirty = false;
            rect_transforms.insert(i, rt);
        }

        RectTransformSystem::mark_all_dirty(&mut rect_transforms);

        for rt in rect_transforms.values() {
            assert!(rt.dirty);
        }
    }
}

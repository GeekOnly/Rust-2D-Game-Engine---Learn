//! UI Hierarchy propagation system
//!
//! This module provides systems for propagating transforms, visibility, and destruction
//! through the UI element hierarchy.

use std::collections::HashMap;
use crate::{UIElement, Canvas};

/// Entity ID type (matches ecs crate)
pub type Entity = u64;

/// UI Hierarchy system
///
/// This system handles propagation of transforms, visibility, and other properties
/// through the UI element hierarchy.
pub struct UIHierarchySystem {
    /// Cached visibility state for each entity
    visibility_cache: HashMap<Entity, bool>,
}

impl UIHierarchySystem {
    /// Create a new UI hierarchy system
    pub fn new() -> Self {
        Self {
            visibility_cache: HashMap::new(),
        }
    }

    /// Update canvas entity cache for all UI elements
    ///
    /// This propagates the canvas entity reference down the hierarchy so that
    /// each UI element knows which canvas it belongs to.
    ///
    /// # Arguments
    /// * `ui_elements` - Mutable map of entity to UIElement
    /// * `canvases` - Map of entity to Canvas (to identify canvas entities)
    /// * `parents` - Map of child entity to parent entity
    pub fn update_canvas_cache(
        &mut self,
        ui_elements: &mut HashMap<Entity, UIElement>,
        canvases: &HashMap<Entity, Canvas>,
        parents: &HashMap<Entity, Entity>,
    ) {
        // Find all canvas entities
        let canvas_entities: Vec<Entity> = canvases.keys().copied().collect();

        // For each canvas, propagate its entity ID to all descendants
        for canvas_entity in canvas_entities {
            self.propagate_canvas_recursive(
                canvas_entity,
                Some(canvas_entity),
                ui_elements,
                parents,
            );
        }

        // Handle UI elements that are not under any canvas (set to None)
        let all_entities: Vec<Entity> = ui_elements.keys().copied().collect();
        for entity in all_entities {
            if !self.has_canvas_ancestor(entity, canvases, parents) {
                if let Some(ui_element) = ui_elements.get_mut(&entity) {
                    ui_element.canvas_entity = None;
                }
            }
        }
    }

    /// Recursively propagate canvas entity to descendants
    fn propagate_canvas_recursive(
        &self,
        entity: Entity,
        canvas_entity: Option<u64>,
        ui_elements: &mut HashMap<Entity, UIElement>,
        parents: &HashMap<Entity, Entity>,
    ) {
        // Set canvas entity for this element
        if let Some(ui_element) = ui_elements.get_mut(&entity) {
            ui_element.canvas_entity = canvas_entity;
        }

        // Propagate to children
        let children: Vec<Entity> = parents
            .iter()
            .filter(|(_, &p)| p == entity)
            .map(|(&c, _)| c)
            .collect();

        for child in children {
            self.propagate_canvas_recursive(child, canvas_entity, ui_elements, parents);
        }
    }

    /// Check if an entity has a canvas ancestor
    fn has_canvas_ancestor(
        &self,
        entity: Entity,
        canvases: &HashMap<Entity, Canvas>,
        parents: &HashMap<Entity, Entity>,
    ) -> bool {
        // Check if this entity is a canvas
        if canvases.contains_key(&entity) {
            return true;
        }

        // Check parent recursively
        if let Some(&parent) = parents.get(&entity) {
            return self.has_canvas_ancestor(parent, canvases, parents);
        }

        false
    }

    /// Update visibility propagation
    ///
    /// This calculates the effective visibility of each UI element based on its own
    /// visibility and the visibility of all its ancestors. A UI element is only visible
    /// if it and all its ancestors are visible.
    ///
    /// # Arguments
    /// * `ui_elements` - Map of entity to UIElement
    /// * `parents` - Map of child entity to parent entity
    /// * `active_states` - Map of entity to active state (from ECS)
    ///
    /// # Returns
    /// Map of entity to effective visibility (true if visible, false if hidden)
    pub fn update_visibility(
        &mut self,
        ui_elements: &HashMap<Entity, UIElement>,
        parents: &HashMap<Entity, Entity>,
        active_states: &HashMap<Entity, bool>,
    ) -> HashMap<Entity, bool> {
        self.visibility_cache.clear();

        // Calculate visibility for all entities
        let all_entities: Vec<Entity> = ui_elements.keys().copied().collect();
        for entity in all_entities {
            let visible = self.calculate_visibility_recursive(
                entity,
                ui_elements,
                parents,
                active_states,
            );
            self.visibility_cache.insert(entity, visible);
        }

        self.visibility_cache.clone()
    }

    /// Recursively calculate visibility for an entity
    fn calculate_visibility_recursive(
        &self,
        entity: Entity,
        ui_elements: &HashMap<Entity, UIElement>,
        parents: &HashMap<Entity, Entity>,
        active_states: &HashMap<Entity, bool>,
    ) -> bool {
        // Check if already calculated (for efficiency)
        if let Some(&cached) = self.visibility_cache.get(&entity) {
            return cached;
        }

        // Check this entity's active state
        let is_active = active_states.get(&entity).copied().unwrap_or(true);
        if !is_active {
            return false;
        }

        // Check parent's visibility
        if let Some(&parent) = parents.get(&entity) {
            let parent_visible = self.calculate_visibility_recursive(
                parent,
                ui_elements,
                parents,
                active_states,
            );
            if !parent_visible {
                return false;
            }
        }

        // This entity is visible
        true
    }

    /// Get the render order for UI elements based on sibling index and Z-order
    ///
    /// This returns a sorted list of entities in the order they should be rendered.
    /// Elements are sorted by:
    /// 1. Canvas sort order (if they belong to different canvases)
    /// 2. Hierarchy depth (parents before children)
    /// 3. Sibling index (order within the same parent)
    /// 4. Z-order (within siblings)
    ///
    /// # Arguments
    /// * `ui_elements` - Map of entity to UIElement
    /// * `canvases` - Map of entity to Canvas
    /// * `parents` - Map of child entity to parent entity
    /// * `children` - Map of parent entity to list of children
    ///
    /// # Returns
    /// Sorted list of entities in render order
    pub fn get_render_order(
        &self,
        ui_elements: &HashMap<Entity, UIElement>,
        canvases: &HashMap<Entity, Canvas>,
        parents: &HashMap<Entity, Entity>,
        children: &HashMap<Entity, Vec<Entity>>,
    ) -> Vec<Entity> {
        let mut render_list = Vec::new();

        // Get all canvas entities sorted by sort order
        let mut canvas_entities: Vec<(Entity, i32)> = canvases
            .iter()
            .map(|(&entity, canvas)| (entity, canvas.sort_order))
            .collect();
        canvas_entities.sort_by_key(|(_, sort_order)| *sort_order);

        // For each canvas, traverse its hierarchy
        for (canvas_entity, _) in canvas_entities {
            self.traverse_hierarchy_for_render(
                canvas_entity,
                ui_elements,
                children,
                &mut render_list,
            );
        }

        // Handle UI elements not under any canvas
        let all_entities: Vec<Entity> = ui_elements.keys().copied().collect();
        for entity in all_entities {
            if !self.has_canvas_ancestor(entity, canvases, parents) {
                self.traverse_hierarchy_for_render(
                    entity,
                    ui_elements,
                    children,
                    &mut render_list,
                );
            }
        }

        render_list
    }

    /// Recursively traverse hierarchy to build render order
    fn traverse_hierarchy_for_render(
        &self,
        entity: Entity,
        ui_elements: &HashMap<Entity, UIElement>,
        children: &HashMap<Entity, Vec<Entity>>,
        render_list: &mut Vec<Entity>,
    ) {
        // Add this entity to render list
        render_list.push(entity);

        // Get children and sort by sibling index and z-order
        if let Some(child_list) = children.get(&entity) {
            let mut sorted_children: Vec<(Entity, i32)> = child_list
                .iter()
                .map(|&child| {
                    let z_order = ui_elements
                        .get(&child)
                        .map(|ui| ui.z_order)
                        .unwrap_or(0);
                    (child, z_order)
                })
                .collect();

            // Sort by z-order (lower values render first)
            sorted_children.sort_by_key(|(_, z_order)| *z_order);

            // Recursively traverse children
            for (child, _) in sorted_children {
                self.traverse_hierarchy_for_render(
                    child,
                    ui_elements,
                    children,
                    render_list,
                );
            }
        }
    }

    /// Handle entity destruction propagation
    ///
    /// When a UI element is destroyed, this returns a list of all descendant entities
    /// that should also be destroyed.
    ///
    /// # Arguments
    /// * `entity` - The entity being destroyed
    /// * `children` - Map of parent entity to list of children
    ///
    /// # Returns
    /// List of all descendant entities that should be destroyed
    pub fn get_descendants_for_destruction(
        &self,
        entity: Entity,
        children: &HashMap<Entity, Vec<Entity>>,
    ) -> Vec<Entity> {
        let mut descendants = Vec::new();
        self.collect_descendants_recursive(entity, children, &mut descendants);
        descendants
    }

    /// Recursively collect all descendants
    fn collect_descendants_recursive(
        &self,
        entity: Entity,
        children: &HashMap<Entity, Vec<Entity>>,
        descendants: &mut Vec<Entity>,
    ) {
        if let Some(child_list) = children.get(&entity) {
            for &child in child_list {
                descendants.push(child);
                self.collect_descendants_recursive(child, children, descendants);
            }
        }
    }
}

impl Default for UIHierarchySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ui_element() -> UIElement {
        UIElement {
            raycast_target: true,
            blocks_raycasts: true,
            z_order: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            alpha: 1.0,
            interactable: true,
            ignore_layout: false,
            canvas_entity: None,
        }
    }

    fn create_test_canvas() -> Canvas {
        Canvas {
            render_mode: crate::CanvasRenderMode::ScreenSpaceOverlay,
            sort_order: 0,
            camera_entity: None,
            plane_distance: 100.0,
            scaler: crate::CanvasScaler::default(),
            blocks_raycasts: true,
            cached_screen_size: (800, 600),
            dirty: true,
        }
    }

    #[test]
    fn test_canvas_cache_propagation() {
        let mut system = UIHierarchySystem::new();
        let mut ui_elements = HashMap::new();
        let mut canvases = HashMap::new();
        let mut parents = HashMap::new();

        let canvas_entity = 1;
        let child1 = 2;
        let child2 = 3;
        let grandchild = 4;

        // Create hierarchy: canvas -> child1 -> grandchild
        //                           -> child2
        canvases.insert(canvas_entity, create_test_canvas());
        ui_elements.insert(canvas_entity, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());
        ui_elements.insert(child2, create_test_ui_element());
        ui_elements.insert(grandchild, create_test_ui_element());

        parents.insert(child1, canvas_entity);
        parents.insert(child2, canvas_entity);
        parents.insert(grandchild, child1);

        system.update_canvas_cache(&mut ui_elements, &canvases, &parents);

        // All elements should have the canvas entity cached
        assert_eq!(ui_elements.get(&canvas_entity).unwrap().canvas_entity, Some(canvas_entity));
        assert_eq!(ui_elements.get(&child1).unwrap().canvas_entity, Some(canvas_entity));
        assert_eq!(ui_elements.get(&child2).unwrap().canvas_entity, Some(canvas_entity));
        assert_eq!(ui_elements.get(&grandchild).unwrap().canvas_entity, Some(canvas_entity));
    }

    #[test]
    fn test_visibility_propagation() {
        let mut system = UIHierarchySystem::new();
        let mut ui_elements = HashMap::new();
        let mut parents = HashMap::new();
        let mut active_states = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;
        let grandchild = 4;

        // Create UI elements
        ui_elements.insert(parent, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());
        ui_elements.insert(child2, create_test_ui_element());
        ui_elements.insert(grandchild, create_test_ui_element());

        // Create hierarchy: parent -> child1 -> grandchild
        //                          -> child2
        parents.insert(child1, parent);
        parents.insert(child2, parent);
        parents.insert(grandchild, child1);

        // All active
        active_states.insert(parent, true);
        active_states.insert(child1, true);
        active_states.insert(child2, true);
        active_states.insert(grandchild, true);

        let visibility = system.update_visibility(&ui_elements, &parents, &active_states);

        assert_eq!(visibility.get(&parent), Some(&true));
        assert_eq!(visibility.get(&child1), Some(&true));
        assert_eq!(visibility.get(&child2), Some(&true));
        assert_eq!(visibility.get(&grandchild), Some(&true));

        // Hide parent
        active_states.insert(parent, false);
        let visibility = system.update_visibility(&ui_elements, &parents, &active_states);

        // All should be hidden
        assert_eq!(visibility.get(&parent), Some(&false));
        assert_eq!(visibility.get(&child1), Some(&false));
        assert_eq!(visibility.get(&child2), Some(&false));
        assert_eq!(visibility.get(&grandchild), Some(&false));

        // Show parent, hide child1
        active_states.insert(parent, true);
        active_states.insert(child1, false);
        let visibility = system.update_visibility(&ui_elements, &parents, &active_states);

        // Parent and child2 visible, child1 and grandchild hidden
        assert_eq!(visibility.get(&parent), Some(&true));
        assert_eq!(visibility.get(&child1), Some(&false));
        assert_eq!(visibility.get(&child2), Some(&true));
        assert_eq!(visibility.get(&grandchild), Some(&false));
    }

    #[test]
    fn test_render_order_sibling_index() {
        let system = UIHierarchySystem::new();
        let mut ui_elements = HashMap::new();
        let mut canvases = HashMap::new();
        let parents = HashMap::new();
        let mut children = HashMap::new();

        let canvas = 1;
        let child1 = 2;
        let child2 = 3;
        let child3 = 4;

        canvases.insert(canvas, create_test_canvas());
        
        let mut ui1 = create_test_ui_element();
        ui1.z_order = 0;
        let mut ui2 = create_test_ui_element();
        ui2.z_order = 1;
        let mut ui3 = create_test_ui_element();
        ui3.z_order = 2;

        ui_elements.insert(canvas, create_test_ui_element());
        ui_elements.insert(child1, ui1);
        ui_elements.insert(child2, ui2);
        ui_elements.insert(child3, ui3);

        children.insert(canvas, vec![child1, child2, child3]);

        let render_order = system.get_render_order(&ui_elements, &canvases, &parents, &children);

        // Canvas should be first, then children in z-order
        assert_eq!(render_order[0], canvas);
        assert_eq!(render_order[1], child1);
        assert_eq!(render_order[2], child2);
        assert_eq!(render_order[3], child3);
    }

    #[test]
    fn test_descendants_for_destruction() {
        let system = UIHierarchySystem::new();
        let mut children = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;
        let grandchild1 = 4;
        let grandchild2 = 5;

        // Create hierarchy: parent -> child1 -> grandchild1
        //                          -> child2 -> grandchild2
        children.insert(parent, vec![child1, child2]);
        children.insert(child1, vec![grandchild1]);
        children.insert(child2, vec![grandchild2]);

        let descendants = system.get_descendants_for_destruction(parent, &children);

        // Should include all descendants
        assert_eq!(descendants.len(), 4);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
        assert!(descendants.contains(&grandchild1));
        assert!(descendants.contains(&grandchild2));
    }

    #[test]
    fn test_descendants_for_destruction_partial() {
        let system = UIHierarchySystem::new();
        let mut children = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;
        let grandchild1 = 4;
        let grandchild2 = 5;

        children.insert(parent, vec![child1, child2]);
        children.insert(child1, vec![grandchild1]);
        children.insert(child2, vec![grandchild2]);

        // Destroy only child1
        let descendants = system.get_descendants_for_destruction(child1, &children);

        // Should only include child1's descendants
        assert_eq!(descendants.len(), 1);
        assert!(descendants.contains(&grandchild1));
        assert!(!descendants.contains(&child2));
        assert!(!descendants.contains(&grandchild2));
    }
}

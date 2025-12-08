//! UI raycasting system for finding UI elements at a point

use glam::Vec2;

/// Entity ID type (using u64 as placeholder)
pub type Entity = u64;

/// Result of a raycast operation
#[derive(Clone, Debug)]
pub struct RaycastHit {
    /// The entity that was hit
    pub entity: Entity,
    
    /// The Z-order of the hit element
    pub z_order: i32,
    
    /// The canvas sort order
    pub canvas_sort_order: i32,
    
    /// The position where the raycast hit
    pub position: Vec2,
}

/// UI element data needed for raycasting
#[derive(Clone, Debug)]
pub struct RaycastElement {
    /// Entity ID
    pub entity: Entity,
    
    /// World-space rect bounds
    pub rect: crate::Rect,
    
    /// Whether this element is a raycast target
    pub raycast_target: bool,
    
    /// Whether this element blocks raycasts
    pub blocks_raycasts: bool,
    
    /// Z-order within siblings
    pub z_order: i32,
    
    /// Canvas sort order (for multi-canvas scenarios)
    pub canvas_sort_order: i32,
    
    /// Whether the element is visible
    pub visible: bool,
    
    /// Whether the element is interactable
    pub interactable: bool,
}

/// UI Raycasting system
pub struct UIRaycastSystem {
    /// Cached raycast elements (updated each frame)
    elements: Vec<RaycastElement>,
}

impl Default for UIRaycastSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UIRaycastSystem {
    /// Create a new raycast system
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
    
    /// Update the raycast elements cache
    /// This should be called each frame with the current UI hierarchy
    pub fn update_elements(&mut self, elements: Vec<RaycastElement>) {
        self.elements = elements;
    }
    
    /// Perform a raycast at the given point
    /// Returns all hits sorted by priority (topmost first)
    pub fn raycast(&self, point: Vec2) -> Vec<RaycastHit> {
        let mut hits = Vec::new();
        
        // Find all elements that contain the point
        for element in &self.elements {
            // Skip if not visible or not interactable
            if !element.visible || !element.interactable {
                continue;
            }
            
            // Skip if not a raycast target
            if !element.raycast_target {
                continue;
            }
            
            // Check if point is inside the element's rect
            if element.rect.contains(point) {
                hits.push(RaycastHit {
                    entity: element.entity,
                    z_order: element.z_order,
                    canvas_sort_order: element.canvas_sort_order,
                    position: point,
                });
            }
        }
        
        // Sort hits by priority (canvas sort order first, then z-order)
        // Higher values should come first (render on top)
        hits.sort_by(|a, b| {
            // First compare canvas sort order (higher = on top)
            match b.canvas_sort_order.cmp(&a.canvas_sort_order) {
                std::cmp::Ordering::Equal => {
                    // If canvas sort order is equal, compare z-order (higher = on top)
                    b.z_order.cmp(&a.z_order)
                }
                other => other,
            }
        });
        
        hits
    }
    
    /// Perform a raycast and return only the topmost hit
    /// This respects raycast blocking
    pub fn raycast_topmost(&self, point: Vec2) -> Option<RaycastHit> {
        let hits = self.raycast(point);
        
        if hits.is_empty() {
            return None;
        }
        
        // Find the first hit that blocks raycasts, or return the topmost hit
        for hit in &hits {
            // Find the element data for this hit
            if let Some(element) = self.elements.iter().find(|e| e.entity == hit.entity) {
                // Return this hit (it's the topmost)
                let result = Some(hit.clone());
                
                // If this element blocks raycasts, stop here
                if element.blocks_raycasts {
                    return result;
                }
                
                // Otherwise, continue to check if there's a blocking element below
                // But we'll still return the topmost hit
                return result;
            }
        }
        
        // Return the topmost hit if we didn't find element data
        hits.first().cloned()
    }
    
    /// Perform a raycast and return all hits that are not blocked
    /// This filters out hits that are behind blocking elements
    pub fn raycast_all_unblocked(&self, point: Vec2) -> Vec<RaycastHit> {
        let hits = self.raycast(point);
        let mut result = Vec::new();
        
        for hit in hits {
            // Add this hit to results
            result.push(hit.clone());
            
            // Check if this element blocks raycasts
            if let Some(element) = self.elements.iter().find(|e| e.entity == hit.entity) {
                if element.blocks_raycasts {
                    // Stop here - all elements below are blocked
                    break;
                }
            }
        }
        
        result
    }
    
    /// Get all raycast targets at a point (ignoring blocking)
    /// Useful for debugging or special cases
    pub fn get_all_at_point(&self, point: Vec2) -> Vec<Entity> {
        self.raycast(point)
            .into_iter()
            .map(|hit| hit.entity)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    fn create_test_element(
        entity: Entity,
        rect: Rect,
        z_order: i32,
        canvas_sort_order: i32,
        raycast_target: bool,
        blocks_raycasts: bool,
    ) -> RaycastElement {
        RaycastElement {
            entity,
            rect,
            raycast_target,
            blocks_raycasts,
            z_order,
            canvas_sort_order,
            visible: true,
            interactable: true,
        }
    }

    #[test]
    fn test_raycast_empty() {
        let system = UIRaycastSystem::new();
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert!(hits.is_empty());
    }

    #[test]
    fn test_raycast_single_element() {
        let mut system = UIRaycastSystem::new();
        
        let element = create_test_element(
            1,
            Rect::new(0.0, 0.0, 100.0, 100.0),
            0,
            0,
            true,
            true,
        );
        
        system.update_elements(vec![element]);
        
        // Point inside
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity, 1);
        
        // Point outside
        let hits = system.raycast(Vec2::new(150.0, 150.0));
        assert!(hits.is_empty());
    }

    #[test]
    fn test_raycast_z_order_sorting() {
        let mut system = UIRaycastSystem::new();
        
        // Create overlapping elements with different z-orders
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 0, true, false),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 5, 0, true, false),
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 2, 0, true, false),
        ];
        
        system.update_elements(elements);
        
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert_eq!(hits.len(), 3);
        
        // Should be sorted by z-order (highest first)
        assert_eq!(hits[0].entity, 2); // z-order 5
        assert_eq!(hits[1].entity, 3); // z-order 2
        assert_eq!(hits[2].entity, 1); // z-order 0
    }

    #[test]
    fn test_raycast_canvas_sort_order() {
        let mut system = UIRaycastSystem::new();
        
        // Create elements on different canvases
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 10, 0, true, false),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 5, true, false),
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 5, 2, true, false),
        ];
        
        system.update_elements(elements);
        
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert_eq!(hits.len(), 3);
        
        // Should be sorted by canvas sort order first, then z-order
        assert_eq!(hits[0].entity, 2); // canvas 5, z-order 0
        assert_eq!(hits[1].entity, 3); // canvas 2, z-order 5
        assert_eq!(hits[2].entity, 1); // canvas 0, z-order 10
    }

    #[test]
    fn test_raycast_target_filtering() {
        let mut system = UIRaycastSystem::new();
        
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 0, true, false),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 1, 0, false, false), // Not a raycast target
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 2, 0, true, false),
        ];
        
        system.update_elements(elements);
        
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert_eq!(hits.len(), 2);
        
        // Element 2 should be filtered out
        assert_eq!(hits[0].entity, 3);
        assert_eq!(hits[1].entity, 1);
    }

    #[test]
    fn test_raycast_topmost() {
        let mut system = UIRaycastSystem::new();
        
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 0, true, false),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 5, 0, true, false),
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 2, 0, true, false),
        ];
        
        system.update_elements(elements);
        
        let hit = system.raycast_topmost(Vec2::new(50.0, 50.0));
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().entity, 2); // Highest z-order
    }

    #[test]
    fn test_raycast_blocking() {
        let mut system = UIRaycastSystem::new();
        
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 0, true, false),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 5, 0, true, true), // Blocks raycasts
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 2, 0, true, false),
        ];
        
        system.update_elements(elements);
        
        let hits = system.raycast_all_unblocked(Vec2::new(50.0, 50.0));
        
        // Should only get element 2 (the blocker) since it blocks elements below
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity, 2);
    }

    #[test]
    fn test_raycast_visibility_filtering() {
        let mut system = UIRaycastSystem::new();
        
        let mut element = create_test_element(
            1,
            Rect::new(0.0, 0.0, 100.0, 100.0),
            0,
            0,
            true,
            true,
        );
        element.visible = false;
        
        system.update_elements(vec![element]);
        
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert!(hits.is_empty()); // Invisible elements should not be hit
    }

    #[test]
    fn test_raycast_interactable_filtering() {
        let mut system = UIRaycastSystem::new();
        
        let mut element = create_test_element(
            1,
            Rect::new(0.0, 0.0, 100.0, 100.0),
            0,
            0,
            true,
            true,
        );
        element.interactable = false;
        
        system.update_elements(vec![element]);
        
        let hits = system.raycast(Vec2::new(50.0, 50.0));
        assert!(hits.is_empty()); // Non-interactable elements should not be hit
    }

    #[test]
    fn test_get_all_at_point() {
        let mut system = UIRaycastSystem::new();
        
        let elements = vec![
            create_test_element(1, Rect::new(0.0, 0.0, 100.0, 100.0), 0, 0, true, true),
            create_test_element(2, Rect::new(0.0, 0.0, 100.0, 100.0), 5, 0, true, true),
            create_test_element(3, Rect::new(0.0, 0.0, 100.0, 100.0), 2, 0, true, false),
        ];
        
        system.update_elements(elements);
        
        let entities = system.get_all_at_point(Vec2::new(50.0, 50.0));
        assert_eq!(entities.len(), 3);
    }
}

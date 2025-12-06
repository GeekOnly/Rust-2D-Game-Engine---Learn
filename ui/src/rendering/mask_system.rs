//! UI Masking system for stencil-based and alpha-based clipping

use crate::{Rect, UIMask, Vec2};
use std::collections::HashMap;

/// Entity type (placeholder - should match the ECS entity type)
pub type Entity = u64;

/// Mask state for an entity
#[derive(Clone, Debug)]
pub struct MaskState {
    /// The mask component
    pub mask: UIMask,
    
    /// The RectTransform bounds for stencil clipping
    pub bounds: Rect,
    
    /// Stencil reference value for this mask
    pub stencil_ref: u8,
    
    /// Whether this mask is active
    pub active: bool,
    
    /// Sprite texture ID for alpha masking (if use_sprite_alpha is true)
    pub sprite_texture: Option<String>,
}

/// Masking system that manages stencil-based and alpha-based clipping
pub struct MaskingSystem {
    /// Active masks by entity
    masks: HashMap<Entity, MaskState>,
    
    /// Stack of active mask regions (for nested masks)
    mask_stack: Vec<(Entity, MaskState)>,
    
    /// Current stencil reference counter
    stencil_counter: u8,
    
    /// Maximum stencil depth (typically 8 bits = 255 levels)
    max_stencil_depth: u8,
}

impl MaskingSystem {
    /// Create a new masking system
    pub fn new() -> Self {
        Self {
            masks: HashMap::new(),
            mask_stack: Vec::new(),
            stencil_counter: 0,
            max_stencil_depth: 255,
        }
    }
    
    /// Register a mask component
    pub fn register_mask(&mut self, entity: Entity, mask: UIMask, bounds: Rect) {
        self.register_mask_with_sprite(entity, mask, bounds, None);
    }
    
    /// Register a mask component with sprite texture for alpha masking
    pub fn register_mask_with_sprite(
        &mut self,
        entity: Entity,
        mask: UIMask,
        bounds: Rect,
        sprite_texture: Option<String>,
    ) {
        let stencil_ref = self.allocate_stencil_ref();
        
        let state = MaskState {
            mask,
            bounds,
            stencil_ref,
            active: true,
            sprite_texture,
        };
        
        self.masks.insert(entity, state);
    }
    
    /// Unregister a mask component
    pub fn unregister_mask(&mut self, entity: Entity) {
        self.masks.remove(&entity);
    }
    
    /// Update a mask's bounds (called when RectTransform changes)
    pub fn update_mask_bounds(&mut self, entity: Entity, bounds: Rect) {
        if let Some(state) = self.masks.get_mut(&entity) {
            state.bounds = bounds;
        }
    }
    
    /// Push a mask onto the stack (entering a masked region)
    pub fn push_mask(&mut self, entity: Entity) -> Option<MaskState> {
        if let Some(state) = self.masks.get(&entity).cloned() {
            if self.mask_stack.len() >= self.max_stencil_depth as usize {
                eprintln!("Warning: Maximum mask nesting depth reached");
                return None;
            }
            
            self.mask_stack.push((entity, state.clone()));
            Some(state)
        } else {
            None
        }
    }
    
    /// Pop a mask from the stack (leaving a masked region)
    pub fn pop_mask(&mut self) -> Option<(Entity, MaskState)> {
        self.mask_stack.pop()
    }
    
    /// Get the current active mask (top of stack)
    pub fn get_active_mask(&self) -> Option<&MaskState> {
        self.mask_stack.last().map(|(_, state)| state)
    }
    
    /// Get all active masks in the stack (for nested masking)
    pub fn get_active_masks(&self) -> &[(Entity, MaskState)] {
        &self.mask_stack
    }
    
    /// Get the intersection of all active mask bounds (for nested masks)
    pub fn get_intersection_bounds(&self) -> Option<Rect> {
        if self.mask_stack.is_empty() {
            return None;
        }
        
        // Start with the first mask's bounds
        let mut result = self.mask_stack[0].1.bounds;
        
        // Intersect with all other masks
        for (_, state) in &self.mask_stack[1..] {
            result = intersect_rects(&result, &state.bounds)?;
        }
        
        Some(result)
    }
    
    /// Check if a point is inside all active masks
    pub fn is_point_masked(&self, point: Vec2) -> bool {
        if self.mask_stack.is_empty() {
            return false; // No masking active
        }
        
        // Point must be inside ALL active masks
        for (_, state) in &self.mask_stack {
            if !point_in_rect(point, &state.bounds) {
                return true; // Point is masked (outside at least one mask)
            }
        }
        
        false // Point is not masked (inside all masks)
    }
    
    /// Check if a rectangle is completely masked (outside all active masks)
    pub fn is_rect_completely_masked(&self, rect: &Rect) -> bool {
        if let Some(intersection) = self.get_intersection_bounds() {
            // Check if rect intersects with the intersection of all masks
            !rects_intersect(rect, &intersection)
        } else {
            false // No masking active
        }
    }
    
    /// Check if a rectangle is partially masked
    pub fn is_rect_partially_masked(&self, rect: &Rect) -> bool {
        if let Some(intersection) = self.get_intersection_bounds() {
            // Check if rect is partially outside the intersection
            rects_intersect(rect, &intersection) && !rect_contains_rect(&intersection, rect)
        } else {
            false // No masking active
        }
    }
    
    /// Get the clipped bounds of a rectangle based on active masks
    pub fn clip_rect(&self, rect: &Rect) -> Option<Rect> {
        if let Some(intersection) = self.get_intersection_bounds() {
            intersect_rects(rect, &intersection)
        } else {
            Some(*rect) // No masking active, return original rect
        }
    }
    
    /// Get the current stencil depth (number of active masks)
    pub fn get_stencil_depth(&self) -> usize {
        self.mask_stack.len()
    }
    
    /// Clear all active masks (typically called at the start of a frame)
    pub fn clear_stack(&mut self) {
        self.mask_stack.clear();
    }
    
    /// Check if a mask should render its graphic
    /// 
    /// When `show_mask_graphic` is true, the mask's visual representation (sprite/image)
    /// should be rendered along with the clipping effect.
    /// When false, only the clipping effect is applied without rendering the mask graphic.
    pub fn should_render_mask_graphic(&self, entity: Entity) -> bool {
        if let Some(state) = self.masks.get(&entity) {
            state.mask.show_mask_graphic
        } else {
            false
        }
    }
    
    /// Set whether a mask should render its graphic
    pub fn set_show_mask_graphic(&mut self, entity: Entity, show: bool) {
        if let Some(state) = self.masks.get_mut(&entity) {
            state.mask.show_mask_graphic = show;
        }
    }
    
    /// Get the mask state for rendering decisions
    /// 
    /// This provides all information needed by the renderer to:
    /// - Apply stencil clipping (using bounds and stencil_ref)
    /// - Apply alpha masking (using sprite_texture if use_sprite_alpha is true)
    /// - Decide whether to render the mask graphic (using show_mask_graphic)
    pub fn get_mask_state(&self, entity: Entity) -> Option<&MaskState> {
        self.masks.get(&entity)
    }
    
    /// Check if a mask uses sprite alpha
    pub fn uses_sprite_alpha(&self, entity: Entity) -> bool {
        if let Some(state) = self.masks.get(&entity) {
            state.mask.use_sprite_alpha
        } else {
            false
        }
    }
    
    /// Get the sprite texture for alpha masking
    pub fn get_sprite_texture(&self, entity: Entity) -> Option<&str> {
        self.masks.get(&entity)
            .and_then(|state| state.sprite_texture.as_deref())
    }
    
    /// Update the sprite texture for a mask
    pub fn update_sprite_texture(&mut self, entity: Entity, sprite_texture: Option<String>) {
        if let Some(state) = self.masks.get_mut(&entity) {
            state.sprite_texture = sprite_texture;
        }
    }
    
    /// Check if a point passes alpha masking for a specific mask
    /// This is a placeholder - actual implementation would sample the sprite texture
    /// Returns true if the point should be visible (alpha > threshold)
    pub fn check_alpha_mask(&self, entity: Entity, point: Vec2, alpha_threshold: f32) -> bool {
        if let Some(state) = self.masks.get(&entity) {
            if !state.mask.use_sprite_alpha {
                // Not using alpha masking, just check bounds
                return point_in_rect(point, &state.bounds);
            }
            
            // Check if point is in bounds first
            if !point_in_rect(point, &state.bounds) {
                return false;
            }
            
            // In a real implementation, we would:
            // 1. Convert point to UV coordinates within the mask bounds
            // 2. Sample the sprite texture at those UV coordinates
            // 3. Check if alpha > alpha_threshold
            // For now, we just return true if in bounds (placeholder)
            true
        } else {
            true // No mask, point is visible
        }
    }
    
    /// Get UV coordinates for a point within a mask's bounds
    /// Returns None if point is outside bounds
    pub fn point_to_uv(&self, entity: Entity, point: Vec2) -> Option<Vec2> {
        if let Some(state) = self.masks.get(&entity) {
            if !point_in_rect(point, &state.bounds) {
                return None;
            }
            
            // Convert world point to normalized UV coordinates (0-1)
            let u = (point.x - state.bounds.x) / state.bounds.width;
            let v = (point.y - state.bounds.y) / state.bounds.height;
            
            Some(Vec2::new(u, v))
        } else {
            None
        }
    }
    
    /// Allocate a new stencil reference value
    fn allocate_stencil_ref(&mut self) -> u8 {
        let ref_val = self.stencil_counter;
        self.stencil_counter = self.stencil_counter.wrapping_add(1);
        ref_val
    }
}

impl Default for MaskingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if a point is inside a rectangle
fn point_in_rect(point: Vec2, rect: &Rect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

/// Helper function to check if two rectangles intersect
fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.width < b.x
        || a.x > b.x + b.width
        || a.y + a.height < b.y
        || a.y > b.y + b.height)
}

/// Helper function to check if rectangle a contains rectangle b
fn rect_contains_rect(a: &Rect, b: &Rect) -> bool {
    b.x >= a.x
        && b.y >= a.y
        && b.x + b.width <= a.x + a.width
        && b.y + b.height <= a.y + a.height
}

/// Helper function to calculate the intersection of two rectangles
fn intersect_rects(a: &Rect, b: &Rect) -> Option<Rect> {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    
    if x2 <= x1 || y2 <= y1 {
        None // No intersection
    } else {
        Some(Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_and_unregister_mask() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        assert!(system.masks.contains_key(&entity));
        
        system.unregister_mask(entity);
        assert!(!system.masks.contains_key(&entity));
    }
    
    #[test]
    fn test_push_and_pop_mask() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        
        assert_eq!(system.get_stencil_depth(), 0);
        
        system.push_mask(entity);
        assert_eq!(system.get_stencil_depth(), 1);
        
        system.pop_mask();
        assert_eq!(system.get_stencil_depth(), 0);
    }
    
    #[test]
    fn test_nested_masks_intersection() {
        let mut system = MaskingSystem::new();
        
        // First mask: 0,0 to 100,100
        let entity1 = 1;
        let mask1 = UIMask::default();
        let bounds1 = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        system.register_mask(entity1, mask1, bounds1);
        system.push_mask(entity1);
        
        // Second mask: 25,25 to 75,75 (inside first)
        let entity2 = 2;
        let mask2 = UIMask::default();
        let bounds2 = Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        };
        system.register_mask(entity2, mask2, bounds2);
        system.push_mask(entity2);
        
        // Intersection should be the smaller rectangle
        let intersection = system.get_intersection_bounds().unwrap();
        assert_eq!(intersection.x, 25.0);
        assert_eq!(intersection.y, 25.0);
        assert_eq!(intersection.width, 50.0);
        assert_eq!(intersection.height, 50.0);
    }
    
    #[test]
    fn test_is_point_masked() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        system.push_mask(entity);
        
        // Point inside mask - not masked
        assert!(!system.is_point_masked(Vec2::new(50.0, 50.0)));
        
        // Point outside mask - masked
        assert!(system.is_point_masked(Vec2::new(150.0, 150.0)));
    }
    
    #[test]
    fn test_is_rect_completely_masked() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        system.push_mask(entity);
        
        // Rect inside mask - not completely masked
        let rect_inside = Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(!system.is_rect_completely_masked(&rect_inside));
        
        // Rect outside mask - completely masked
        let rect_outside = Rect {
            x: 200.0,
            y: 200.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(system.is_rect_completely_masked(&rect_outside));
    }
    
    #[test]
    fn test_clip_rect() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        system.push_mask(entity);
        
        // Rect partially outside mask - should be clipped
        let rect = Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        };
        let clipped = system.clip_rect(&rect).unwrap();
        assert_eq!(clipped.x, 50.0);
        assert_eq!(clipped.y, 50.0);
        assert_eq!(clipped.width, 50.0);
        assert_eq!(clipped.height, 50.0);
    }
    
    #[test]
    fn test_should_render_mask_graphic() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.show_mask_graphic = true;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask.clone(), bounds);
        assert!(system.should_render_mask_graphic(entity));
        
        mask.show_mask_graphic = false;
        system.register_mask(entity, mask, bounds);
        assert!(!system.should_render_mask_graphic(entity));
    }
    
    #[test]
    fn test_set_show_mask_graphic() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default(); // default is show_mask_graphic = true
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        assert!(system.should_render_mask_graphic(entity));
        
        // Change to not show graphic
        system.set_show_mask_graphic(entity, false);
        assert!(!system.should_render_mask_graphic(entity));
        
        // Change back to show graphic
        system.set_show_mask_graphic(entity, true);
        assert!(system.should_render_mask_graphic(entity));
    }
    
    #[test]
    fn test_get_mask_state() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.show_mask_graphic = false;
        mask.use_sprite_alpha = true;
        let bounds = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 200.0,
        };
        
        system.register_mask_with_sprite(
            entity,
            mask,
            bounds,
            Some("test_sprite.png".to_string()),
        );
        
        let state = system.get_mask_state(entity).unwrap();
        assert!(!state.mask.show_mask_graphic);
        assert!(state.mask.use_sprite_alpha);
        assert_eq!(state.bounds.x, 10.0);
        assert_eq!(state.bounds.y, 20.0);
        assert_eq!(state.bounds.width, 100.0);
        assert_eq!(state.bounds.height, 200.0);
        assert_eq!(state.sprite_texture.as_deref(), Some("test_sprite.png"));
    }
    
    #[test]
    fn test_mask_graphic_visibility_with_clipping() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.show_mask_graphic = false; // Don't show graphic, but still clip
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        system.push_mask(entity);
        
        // Mask graphic should not be rendered
        assert!(!system.should_render_mask_graphic(entity));
        
        // But clipping should still work
        assert!(!system.is_point_masked(Vec2::new(50.0, 50.0))); // Inside - not masked
        assert!(system.is_point_masked(Vec2::new(150.0, 150.0))); // Outside - masked
    }
    
    #[test]
    fn test_uses_sprite_alpha() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.use_sprite_alpha = true;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask.clone(), bounds);
        assert!(system.uses_sprite_alpha(entity));
        
        mask.use_sprite_alpha = false;
        system.register_mask(entity, mask, bounds);
        assert!(!system.uses_sprite_alpha(entity));
    }
    
    #[test]
    fn test_register_mask_with_sprite() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.use_sprite_alpha = true;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let sprite_texture = Some("mask_sprite.png".to_string());
        
        system.register_mask_with_sprite(entity, mask, bounds, sprite_texture.clone());
        
        assert!(system.uses_sprite_alpha(entity));
        assert_eq!(system.get_sprite_texture(entity), Some("mask_sprite.png"));
    }
    
    #[test]
    fn test_update_sprite_texture() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        assert_eq!(system.get_sprite_texture(entity), None);
        
        system.update_sprite_texture(entity, Some("new_sprite.png".to_string()));
        assert_eq!(system.get_sprite_texture(entity), Some("new_sprite.png"));
    }
    
    #[test]
    fn test_point_to_uv() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mask = UIMask::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask(entity, mask, bounds);
        
        // Point at center should give UV (0.5, 0.5)
        let uv = system.point_to_uv(entity, Vec2::new(50.0, 50.0)).unwrap();
        assert!((uv.x - 0.5).abs() < 0.001);
        assert!((uv.y - 0.5).abs() < 0.001);
        
        // Point at top-left should give UV (0, 0)
        let uv = system.point_to_uv(entity, Vec2::new(0.0, 0.0)).unwrap();
        assert!((uv.x - 0.0).abs() < 0.001);
        assert!((uv.y - 0.0).abs() < 0.001);
        
        // Point at bottom-right should give UV (1, 1)
        let uv = system.point_to_uv(entity, Vec2::new(100.0, 100.0)).unwrap();
        assert!((uv.x - 1.0).abs() < 0.001);
        assert!((uv.y - 1.0).abs() < 0.001);
        
        // Point outside bounds should return None
        assert!(system.point_to_uv(entity, Vec2::new(150.0, 150.0)).is_none());
    }
    
    #[test]
    fn test_check_alpha_mask() {
        let mut system = MaskingSystem::new();
        let entity = 1;
        let mut mask = UIMask::default();
        mask.use_sprite_alpha = true;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        system.register_mask_with_sprite(
            entity,
            mask,
            bounds,
            Some("mask_sprite.png".to_string()),
        );
        
        // Point inside bounds should pass (placeholder implementation)
        assert!(system.check_alpha_mask(entity, Vec2::new(50.0, 50.0), 0.5));
        
        // Point outside bounds should fail
        assert!(!system.check_alpha_mask(entity, Vec2::new(150.0, 150.0), 0.5));
    }
}

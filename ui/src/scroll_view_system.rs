//! Scroll view interaction system

use crate::{
    UIScrollView, MovementType, RectTransform, Rect, Vec2,
    rendering::ClipRegion,
};

/// Scroll view system for handling scroll interactions
pub struct ScrollViewSystem {
    /// Delta time for physics calculations
    delta_time: f32,
}

impl ScrollViewSystem {
    /// Create a new scroll view system
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
        }
    }
    
    /// Update the system with delta time
    pub fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
    }
    
    /// Process drag scrolling for a scroll view
    pub fn process_drag_scroll(
        &self,
        scroll_view: &mut UIScrollView,
        content_transform: &mut RectTransform,
        viewport_rect: &Rect,
        content_rect: &Rect,
        drag_delta: Vec2,
    ) {
        // Apply scroll sensitivity
        let scroll_delta = drag_delta * scroll_view.scroll_sensitivity;
        
        // Calculate the scrollable range
        let scrollable_width = (content_rect.width - viewport_rect.width).max(0.0);
        let scrollable_height = (content_rect.height - viewport_rect.height).max(0.0);
        
        // Update content position based on drag
        let mut new_position = content_transform.anchored_position;
        
        if scroll_view.horizontal && scrollable_width > 0.0 {
            new_position.x += scroll_delta.x;
        }
        
        if scroll_view.vertical && scrollable_height > 0.0 {
            new_position.y += scroll_delta.y;
        }
        
        // Apply movement constraints
        new_position = self.apply_movement_constraints(
            scroll_view,
            new_position,
            viewport_rect,
            content_rect,
        );
        
        content_transform.anchored_position = new_position;
        
        // Update normalized position
        scroll_view.normalized_position = self.calculate_normalized_position(
            &new_position,
            viewport_rect,
            content_rect,
        );
        
        // Update velocity for inertia
        if scroll_view.inertia && self.delta_time > 0.0 {
            scroll_view.velocity = scroll_delta / self.delta_time;
        }
    }
    
    /// Apply inertia deceleration
    pub fn apply_inertia(
        &self,
        scroll_view: &mut UIScrollView,
        content_transform: &mut RectTransform,
        viewport_rect: &Rect,
        content_rect: &Rect,
    ) {
        if !scroll_view.inertia || scroll_view.velocity.length() < 0.1 {
            scroll_view.velocity = Vec2::ZERO;
            return;
        }
        
        // Apply deceleration
        let deceleration = scroll_view.deceleration_rate;
        scroll_view.velocity *= 1.0 - deceleration;
        
        // Update position based on velocity
        let mut new_position = content_transform.anchored_position;
        new_position += scroll_view.velocity * self.delta_time;
        
        // Apply movement constraints
        new_position = self.apply_movement_constraints(
            scroll_view,
            new_position,
            viewport_rect,
            content_rect,
        );
        
        content_transform.anchored_position = new_position;
        
        // Update normalized position
        scroll_view.normalized_position = self.calculate_normalized_position(
            &new_position,
            viewport_rect,
            content_rect,
        );
        
        // Stop velocity if we hit a boundary with clamped movement
        if scroll_view.movement_type == MovementType::Clamped {
            let bounds = self.calculate_bounds(viewport_rect, content_rect);
            if new_position.x <= bounds.0 || new_position.x >= bounds.1 {
                scroll_view.velocity.x = 0.0;
            }
            if new_position.y <= bounds.2 || new_position.y >= bounds.3 {
                scroll_view.velocity.y = 0.0;
            }
        }
    }
    
    /// Apply elastic spring-back
    pub fn apply_elastic_spring_back(
        &self,
        scroll_view: &mut UIScrollView,
        content_transform: &mut RectTransform,
        viewport_rect: &Rect,
        content_rect: &Rect,
    ) {
        if scroll_view.movement_type != MovementType::Elastic {
            return;
        }
        
        let bounds = self.calculate_bounds(viewport_rect, content_rect);
        let current_pos = content_transform.anchored_position;
        let mut target_pos = current_pos;
        let mut needs_spring_back = false;
        
        // Check if we're beyond bounds
        if current_pos.x < bounds.0 {
            target_pos.x = bounds.0;
            needs_spring_back = true;
        } else if current_pos.x > bounds.1 {
            target_pos.x = bounds.1;
            needs_spring_back = true;
        }
        
        if current_pos.y < bounds.2 {
            target_pos.y = bounds.2;
            needs_spring_back = true;
        } else if current_pos.y > bounds.3 {
            target_pos.y = bounds.3;
            needs_spring_back = true;
        }
        
        if needs_spring_back {
            // Apply spring force
            let spring_force = (target_pos - current_pos) * scroll_view.elasticity;
            let new_position = current_pos + spring_force;
            
            content_transform.anchored_position = new_position;
            
            // Update normalized position
            scroll_view.normalized_position = self.calculate_normalized_position(
                &new_position,
                viewport_rect,
                content_rect,
            );
            
            // Dampen velocity
            scroll_view.velocity *= 0.9;
        }
    }
    
    /// Set scroll position programmatically
    pub fn set_normalized_position(
        &self,
        scroll_view: &mut UIScrollView,
        content_transform: &mut RectTransform,
        viewport_rect: &Rect,
        content_rect: &Rect,
        normalized_position: Vec2,
    ) {
        // Clamp normalized position to [0, 1]
        let clamped_pos = Vec2::new(
            normalized_position.x.clamp(0.0, 1.0),
            normalized_position.y.clamp(0.0, 1.0),
        );
        
        // Calculate the scrollable range
        let scrollable_width = (content_rect.width - viewport_rect.width).max(0.0);
        let scrollable_height = (content_rect.height - viewport_rect.height).max(0.0);
        
        // Calculate actual position from normalized position
        // Note: In UI coordinates, scrolling down means moving content up (negative Y)
        let new_position = Vec2::new(
            -clamped_pos.x * scrollable_width,
            -clamped_pos.y * scrollable_height,
        );
        
        content_transform.anchored_position = new_position;
        scroll_view.normalized_position = clamped_pos;
        
        // Reset velocity when setting position programmatically
        scroll_view.velocity = Vec2::ZERO;
    }
    
    /// Update scrollbar position based on scroll view state
    pub fn update_scrollbar_position(
        &self,
        scroll_view: &UIScrollView,
        scrollbar_handle_transform: &mut RectTransform,
        is_horizontal: bool,
    ) {
        let normalized_pos = if is_horizontal {
            scroll_view.normalized_position.x
        } else {
            scroll_view.normalized_position.y
        };
        
        // Update scrollbar handle position
        // The handle should move within the scrollbar track based on normalized position
        // This is a simplified version - actual implementation would need track bounds
        if is_horizontal {
            scrollbar_handle_transform.anchored_position.x = normalized_pos * 100.0; // Placeholder
        } else {
            scrollbar_handle_transform.anchored_position.y = -normalized_pos * 100.0; // Placeholder
        }
    }
    
    /// Create a clip region for a scroll view viewport
    pub fn create_viewport_clip_region(&self, viewport_rect: &Rect) -> ClipRegion {
        ClipRegion::new(*viewport_rect)
    }
    
    /// Calculate normalized position from actual position
    fn calculate_normalized_position(
        &self,
        position: &Vec2,
        viewport_rect: &Rect,
        content_rect: &Rect,
    ) -> Vec2 {
        let scrollable_width = (content_rect.width - viewport_rect.width).max(0.0);
        let scrollable_height = (content_rect.height - viewport_rect.height).max(0.0);
        
        let normalized_x = if scrollable_width > 0.0 {
            (-position.x / scrollable_width).clamp(0.0, 1.0)
        } else {
            0.0
        };
        
        let normalized_y = if scrollable_height > 0.0 {
            (-position.y / scrollable_height).clamp(0.0, 1.0)
        } else {
            0.0
        };
        
        Vec2::new(normalized_x, normalized_y)
    }
    
    /// Apply movement constraints based on movement type
    fn apply_movement_constraints(
        &self,
        scroll_view: &UIScrollView,
        position: Vec2,
        viewport_rect: &Rect,
        content_rect: &Rect,
    ) -> Vec2 {
        match scroll_view.movement_type {
            MovementType::Unrestricted => position,
            MovementType::Clamped | MovementType::Elastic => {
                let bounds = self.calculate_bounds(viewport_rect, content_rect);
                
                let mut clamped = position;
                
                // For clamped movement, strictly enforce bounds
                // For elastic movement, allow going beyond bounds (spring-back will handle it)
                if scroll_view.movement_type == MovementType::Clamped {
                    clamped.x = clamped.x.clamp(bounds.0, bounds.1);
                    clamped.y = clamped.y.clamp(bounds.2, bounds.3);
                }
                
                clamped
            }
        }
    }
    
    /// Calculate scroll bounds (min_x, max_x, min_y, max_y)
    fn calculate_bounds(
        &self,
        viewport_rect: &Rect,
        content_rect: &Rect,
    ) -> (f32, f32, f32, f32) {
        let scrollable_width = (content_rect.width - viewport_rect.width).max(0.0);
        let scrollable_height = (content_rect.height - viewport_rect.height).max(0.0);
        
        // Content position bounds
        // When content is at top-left, position is (0, 0)
        // When scrolled to bottom-right, position is (-scrollable_width, -scrollable_height)
        let min_x = -scrollable_width;
        let max_x = 0.0;
        let min_y = -scrollable_height;
        let max_y = 0.0;
        
        (min_x, max_x, min_y, max_y)
    }
}

impl Default for ScrollViewSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_scroll_view() -> UIScrollView {
        UIScrollView {
            content: None,
            viewport: None,
            horizontal_scrollbar: None,
            vertical_scrollbar: None,
            movement_type: MovementType::Clamped,
            elasticity: 0.1,
            inertia: true,
            deceleration_rate: 0.135,
            scroll_sensitivity: 1.0,
            horizontal: true,
            vertical: true,
            normalized_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }
    
    fn create_test_transform() -> RectTransform {
        RectTransform {
            anchor_min: Vec2::ZERO,
            anchor_max: Vec2::ONE,
            pivot: Vec2::new(0.5, 0.5),
            anchored_position: Vec2::ZERO,
            size_delta: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: Rect::default(),
            dirty: false,
        }
    }
    
    #[test]
    fn test_drag_scroll() {
        let mut system = ScrollViewSystem::new();
        system.update(0.016); // 60 FPS
        
        let mut scroll_view = create_test_scroll_view();
        let mut content_transform = create_test_transform();
        
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        let content_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        
        // Drag down (positive Y delta) - this scrolls content up (negative Y in content space)
        system.process_drag_scroll(
            &mut scroll_view,
            &mut content_transform,
            &viewport_rect,
            &content_rect,
            Vec2::new(0.0, -10.0), // Drag up to scroll down
        );
        
        // Content should move down (negative Y in content space to show lower content)
        assert!(content_transform.anchored_position.y < 0.0);
        assert!(scroll_view.velocity.y < 0.0);
    }
    
    #[test]
    fn test_set_normalized_position() {
        let system = ScrollViewSystem::new();
        
        let mut scroll_view = create_test_scroll_view();
        let mut content_transform = create_test_transform();
        
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        let content_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        
        // Set to middle position
        system.set_normalized_position(
            &mut scroll_view,
            &mut content_transform,
            &viewport_rect,
            &content_rect,
            Vec2::new(0.5, 0.5),
        );
        
        assert_eq!(scroll_view.normalized_position.x, 0.5);
        assert_eq!(scroll_view.normalized_position.y, 0.5);
        assert_eq!(content_transform.anchored_position.x, -50.0);
        assert_eq!(content_transform.anchored_position.y, -50.0);
    }
    
    #[test]
    fn test_clamped_movement() {
        let mut system = ScrollViewSystem::new();
        system.update(0.016);
        
        let mut scroll_view = create_test_scroll_view();
        scroll_view.movement_type = MovementType::Clamped;
        let mut content_transform = create_test_transform();
        
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        let content_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        
        // Try to drag beyond bounds
        system.process_drag_scroll(
            &mut scroll_view,
            &mut content_transform,
            &viewport_rect,
            &content_rect,
            Vec2::new(0.0, 200.0), // Large drag
        );
        
        // Should be clamped to max bound (0.0)
        assert!(content_transform.anchored_position.y <= 0.0);
    }
    
    #[test]
    fn test_inertia_deceleration() {
        let mut system = ScrollViewSystem::new();
        system.update(0.016);
        
        let mut scroll_view = create_test_scroll_view();
        // Start with negative velocity (scrolling left) and position in middle
        scroll_view.velocity = Vec2::new(-100.0, 0.0);
        let mut content_transform = create_test_transform();
        content_transform.anchored_position = Vec2::new(-50.0, 0.0); // Middle position
        
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        let content_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        
        let initial_velocity = scroll_view.velocity.x.abs();
        
        // Apply inertia
        system.apply_inertia(
            &mut scroll_view,
            &mut content_transform,
            &viewport_rect,
            &content_rect,
        );
        
        // Velocity magnitude should decrease
        assert!(scroll_view.velocity.x.abs() < initial_velocity);
        assert!(scroll_view.velocity.x.abs() > 0.0);
    }
    
    #[test]
    fn test_elastic_spring_back() {
        let mut system = ScrollViewSystem::new();
        system.update(0.016);
        
        let mut scroll_view = create_test_scroll_view();
        scroll_view.movement_type = MovementType::Elastic;
        let mut content_transform = create_test_transform();
        
        // Set position beyond bounds
        content_transform.anchored_position = Vec2::new(50.0, 0.0); // Beyond max bound
        
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        
        let content_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        
        let initial_pos = content_transform.anchored_position.x;
        
        // Apply spring back
        system.apply_elastic_spring_back(
            &mut scroll_view,
            &mut content_transform,
            &viewport_rect,
            &content_rect,
        );
        
        // Position should move toward bound (0.0)
        assert!(content_transform.anchored_position.x < initial_pos);
    }
}

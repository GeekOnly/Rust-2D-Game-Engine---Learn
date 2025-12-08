//! Viewport clipping system for scroll views

use crate::{Rect, Vec2};

/// Clipping region for viewport
#[derive(Clone, Debug)]
pub struct ClipRegion {
    /// The clipping rectangle in world space
    pub rect: Rect,
    
    /// Whether clipping is enabled
    pub enabled: bool,
}

impl ClipRegion {
    /// Create a new clip region
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            enabled: true,
        }
    }
    
    /// Check if a point is inside the clip region
    pub fn contains_point(&self, point: Vec2) -> bool {
        if !self.enabled {
            return true;
        }
        
        point.x >= self.rect.x
            && point.x <= self.rect.x + self.rect.width
            && point.y >= self.rect.y
            && point.y <= self.rect.y + self.rect.height
    }
    
    /// Check if a rectangle intersects with the clip region
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        if !self.enabled {
            return true;
        }
        
        // Check if rectangles overlap
        !(rect.x + rect.width < self.rect.x
            || rect.x > self.rect.x + self.rect.width
            || rect.y + rect.height < self.rect.y
            || rect.y > self.rect.y + self.rect.height)
    }
    
    /// Check if a rectangle is completely outside the clip region
    pub fn is_culled(&self, rect: &Rect) -> bool {
        if !self.enabled {
            return false;
        }
        
        !self.intersects_rect(rect)
    }
    
    /// Clip a rectangle to the clip region bounds
    pub fn clip_rect(&self, rect: &Rect) -> Option<Rect> {
        if !self.enabled {
            return Some(*rect);
        }
        
        if self.is_culled(rect) {
            return None;
        }
        
        let x1 = rect.x.max(self.rect.x);
        let y1 = rect.y.max(self.rect.y);
        let x2 = (rect.x + rect.width).min(self.rect.x + self.rect.width);
        let y2 = (rect.y + rect.height).min(self.rect.y + self.rect.height);
        
        Some(Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        })
    }
}

/// Viewport clipping system
pub struct ViewportClippingSystem {
    /// Stack of active clip regions (for nested clipping)
    clip_stack: Vec<ClipRegion>,
}

impl ViewportClippingSystem {
    /// Create a new viewport clipping system
    pub fn new() -> Self {
        Self {
            clip_stack: Vec::new(),
        }
    }
    
    /// Push a clip region onto the stack
    pub fn push_clip_region(&mut self, region: ClipRegion) {
        self.clip_stack.push(region);
    }
    
    /// Pop a clip region from the stack
    pub fn pop_clip_region(&mut self) -> Option<ClipRegion> {
        self.clip_stack.pop()
    }
    
    /// Get the current active clip region (intersection of all regions in stack)
    pub fn get_active_clip_region(&self) -> Option<ClipRegion> {
        if self.clip_stack.is_empty() {
            return None;
        }
        
        // Start with the first region
        let mut result = self.clip_stack[0].clone();
        
        // Intersect with all other regions
        for region in &self.clip_stack[1..] {
            if !region.enabled {
                continue;
            }
            
            if !result.enabled {
                result = region.clone();
                continue;
            }
            
            // Calculate intersection
            let x1 = result.rect.x.max(region.rect.x);
            let y1 = result.rect.y.max(region.rect.y);
            let x2 = (result.rect.x + result.rect.width).min(region.rect.x + region.rect.width);
            let y2 = (result.rect.y + result.rect.height).min(region.rect.y + region.rect.height);
            
            if x2 <= x1 || y2 <= y1 {
                // No intersection - empty region
                result.rect = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                };
            } else {
                result.rect = Rect {
                    x: x1,
                    y: y1,
                    width: x2 - x1,
                    height: y2 - y1,
                };
            }
        }
        
        Some(result)
    }
    
    /// Check if a rectangle should be culled based on active clip regions
    pub fn should_cull(&self, rect: &Rect) -> bool {
        if let Some(clip_region) = self.get_active_clip_region() {
            clip_region.is_culled(rect)
        } else {
            false
        }
    }
    
    /// Clear all clip regions
    pub fn clear(&mut self) {
        self.clip_stack.clear();
    }
}

impl Default for ViewportClippingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clip_region_contains_point() {
        let region = ClipRegion::new(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        
        assert!(region.contains_point(Vec2::new(50.0, 50.0)));
        assert!(region.contains_point(Vec2::new(0.0, 0.0)));
        assert!(region.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!region.contains_point(Vec2::new(-1.0, 50.0)));
        assert!(!region.contains_point(Vec2::new(101.0, 50.0)));
    }
    
    #[test]
    fn test_clip_region_intersects_rect() {
        let region = ClipRegion::new(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        
        // Fully inside
        assert!(region.intersects_rect(&Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        }));
        
        // Partially overlapping
        assert!(region.intersects_rect(&Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        }));
        
        // Completely outside
        assert!(!region.intersects_rect(&Rect {
            x: 200.0,
            y: 200.0,
            width: 50.0,
            height: 50.0,
        }));
    }
    
    #[test]
    fn test_clip_rect() {
        let region = ClipRegion::new(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        
        // Fully inside - no clipping
        let rect = Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        };
        let clipped = region.clip_rect(&rect).unwrap();
        assert_eq!(clipped.x, 25.0);
        assert_eq!(clipped.y, 25.0);
        assert_eq!(clipped.width, 50.0);
        assert_eq!(clipped.height, 50.0);
        
        // Partially outside - clipped
        let rect = Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        };
        let clipped = region.clip_rect(&rect).unwrap();
        assert_eq!(clipped.x, 50.0);
        assert_eq!(clipped.y, 50.0);
        assert_eq!(clipped.width, 50.0);
        assert_eq!(clipped.height, 50.0);
        
        // Completely outside - culled
        let rect = Rect {
            x: 200.0,
            y: 200.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(region.clip_rect(&rect).is_none());
    }
    
    #[test]
    fn test_viewport_clipping_system_stack() {
        let mut system = ViewportClippingSystem::new();
        
        // Push first region
        system.push_clip_region(ClipRegion::new(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }));
        
        // Push second region (smaller, inside first)
        system.push_clip_region(ClipRegion::new(Rect {
            x: 25.0,
            y: 25.0,
            width: 50.0,
            height: 50.0,
        }));
        
        // Active region should be the intersection
        let active = system.get_active_clip_region().unwrap();
        assert_eq!(active.rect.x, 25.0);
        assert_eq!(active.rect.y, 25.0);
        assert_eq!(active.rect.width, 50.0);
        assert_eq!(active.rect.height, 50.0);
        
        // Pop second region
        system.pop_clip_region();
        
        // Active region should be the first one
        let active = system.get_active_clip_region().unwrap();
        assert_eq!(active.rect.x, 0.0);
        assert_eq!(active.rect.y, 0.0);
        assert_eq!(active.rect.width, 100.0);
        assert_eq!(active.rect.height, 100.0);
    }
}

//! RectTransform component for UI positioning and sizing

use serde::{Deserialize, Serialize};
use glam::Vec2;
use crate::types::Rect;

/// RectTransform defines the position, size, and anchoring of UI elements
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RectTransform {
    /// Anchor minimum (normalized 0-1 in parent space)
    pub anchor_min: Vec2,
    
    /// Anchor maximum (normalized 0-1 in parent space)
    pub anchor_max: Vec2,
    
    /// Pivot point (normalized 0-1 in local space)
    pub pivot: Vec2,
    
    /// Anchored position (offset from anchor point)
    pub anchored_position: Vec2,
    
    /// Size delta (additional size beyond anchors)
    pub size_delta: Vec2,
    
    /// Local rotation (Z-axis rotation in degrees)
    pub rotation: f32,
    
    /// Local scale
    pub scale: Vec2,
    
    /// Cached world corners (updated by layout system)
    /// Order: Bottom-left, top-left, top-right, bottom-right
    #[serde(skip)]
    pub world_corners: [Vec2; 4],
    
    /// Cached rect (updated by layout system)
    #[serde(skip)]
    pub rect: Rect,
    
    /// Dirty flag
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: Vec2::new(0.5, 0.5),
            anchor_max: Vec2::new(0.5, 0.5),
            pivot: Vec2::new(0.5, 0.5),
            anchored_position: Vec2::ZERO,
            size_delta: Vec2::new(100.0, 100.0),
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: Rect::default(),
            dirty: true,
        }
    }
}

impl RectTransform {
    /// Clamp a Vec2 to the range [0, 1]
    fn clamp_anchor(anchor: Vec2) -> Vec2 {
        Vec2::new(
            anchor.x.clamp(0.0, 1.0),
            anchor.y.clamp(0.0, 1.0),
        )
    }

    /// Create with anchored position (for fixed-size elements)
    /// 
    /// # Arguments
    /// * `anchor` - The anchor point (0-1 normalized in parent space)
    /// * `position` - The offset from the anchor point
    /// * `size` - The size of the element
    pub fn anchored(anchor: Vec2, position: Vec2, size: Vec2) -> Self {
        let clamped_anchor = Self::clamp_anchor(anchor);
        Self {
            anchor_min: clamped_anchor,
            anchor_max: clamped_anchor,
            pivot: Vec2::new(0.5, 0.5),
            anchored_position: position,
            size_delta: size,
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: Rect::default(),
            dirty: true,
        }
    }
    
    /// Create with stretched anchors (for responsive elements)
    /// 
    /// # Arguments
    /// * `anchor_min` - The minimum anchor point (0-1 normalized)
    /// * `anchor_max` - The maximum anchor point (0-1 normalized)
    /// * `margins` - The margins (left, bottom, right, top)
    pub fn stretched(anchor_min: Vec2, anchor_max: Vec2, margins: glam::Vec4) -> Self {
        let clamped_min = Self::clamp_anchor(anchor_min);
        let clamped_max = Self::clamp_anchor(anchor_max);
        Self {
            anchor_min: clamped_min,
            anchor_max: clamped_max,
            pivot: Vec2::new(0.5, 0.5),
            anchored_position: Vec2::new(
                (margins.x - margins.z) * 0.5,
                (margins.y - margins.w) * 0.5,
            ),
            size_delta: Vec2::new(
                -(margins.x + margins.z),
                -(margins.y + margins.w),
            ),
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: Rect::default(),
            dirty: true,
        }
    }
    
    /// Set anchor min, clamping to valid range [0, 1]
    pub fn set_anchor_min(&mut self, anchor_min: Vec2) {
        self.anchor_min = Self::clamp_anchor(anchor_min);
        self.dirty = true;
    }
    
    /// Set anchor max, clamping to valid range [0, 1]
    pub fn set_anchor_max(&mut self, anchor_max: Vec2) {
        self.anchor_max = Self::clamp_anchor(anchor_max);
        self.dirty = true;
    }
    
    /// Set both anchor min and max, clamping to valid range [0, 1]
    pub fn set_anchors(&mut self, anchor_min: Vec2, anchor_max: Vec2) {
        self.anchor_min = Self::clamp_anchor(anchor_min);
        self.anchor_max = Self::clamp_anchor(anchor_max);
        self.dirty = true;
    }
    
    /// Get the calculated size
    pub fn get_size(&self) -> Vec2 {
        // If anchors are the same, size is just size_delta
        // If anchors are different, size depends on parent size (calculated by layout system)
        self.size_delta
    }
    
    /// Set the size (updates size_delta)
    pub fn set_size(&mut self, size: Vec2) {
        self.size_delta = size;
        self.dirty = true;
    }
    
    /// Get world position (center of the rect)
    pub fn get_world_position(&self) -> Vec2 {
        self.rect.center()
    }
    
    /// Check if point is inside rect (for raycasting)
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.rect.contains(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rect_transform() {
        let rt = RectTransform::default();
        assert_eq!(rt.anchor_min, Vec2::new(0.5, 0.5));
        assert_eq!(rt.anchor_max, Vec2::new(0.5, 0.5));
        assert_eq!(rt.pivot, Vec2::new(0.5, 0.5));
        assert_eq!(rt.anchored_position, Vec2::ZERO);
        assert_eq!(rt.size_delta, Vec2::new(100.0, 100.0));
        assert_eq!(rt.rotation, 0.0);
        assert_eq!(rt.scale, Vec2::ONE);
        assert!(rt.dirty);
    }

    #[test]
    fn test_anchored_creation() {
        let anchor = Vec2::new(0.5, 0.5);
        let position = Vec2::new(10.0, 20.0);
        let size = Vec2::new(100.0, 50.0);
        
        let rt = RectTransform::anchored(anchor, position, size);
        assert_eq!(rt.anchor_min, anchor);
        assert_eq!(rt.anchor_max, anchor);
        assert_eq!(rt.anchored_position, position);
        assert_eq!(rt.size_delta, size);
    }

    #[test]
    fn test_stretched_creation() {
        let anchor_min = Vec2::new(0.0, 0.0);
        let anchor_max = Vec2::new(1.0, 1.0);
        let margins = glam::Vec4::new(10.0, 10.0, 10.0, 10.0); // left, bottom, right, top
        
        let rt = RectTransform::stretched(anchor_min, anchor_max, margins);
        assert_eq!(rt.anchor_min, anchor_min);
        assert_eq!(rt.anchor_max, anchor_max);
        assert_eq!(rt.size_delta, Vec2::new(-20.0, -20.0)); // -(left + right), -(bottom + top)
    }

    #[test]
    fn test_get_set_size() {
        let mut rt = RectTransform::default();
        let new_size = Vec2::new(200.0, 150.0);
        
        rt.set_size(new_size);
        assert_eq!(rt.get_size(), new_size);
        assert!(rt.dirty);
    }

    #[test]
    fn test_anchor_clamping() {
        // Test that anchors are clamped to [0, 1] range
        let rt = RectTransform::anchored(Vec2::new(1.5, -0.5), Vec2::ZERO, Vec2::new(100.0, 100.0));
        assert_eq!(rt.anchor_min, Vec2::new(1.0, 0.0));
        assert_eq!(rt.anchor_max, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_set_anchor_min_clamping() {
        let mut rt = RectTransform::default();
        rt.set_anchor_min(Vec2::new(-0.5, 1.5));
        assert_eq!(rt.anchor_min, Vec2::new(0.0, 1.0));
        assert!(rt.dirty);
    }

    #[test]
    fn test_set_anchor_max_clamping() {
        let mut rt = RectTransform::default();
        rt.set_anchor_max(Vec2::new(2.0, -1.0));
        assert_eq!(rt.anchor_max, Vec2::new(1.0, 0.0));
        assert!(rt.dirty);
    }

    #[test]
    fn test_set_anchors_clamping() {
        let mut rt = RectTransform::default();
        rt.set_anchors(Vec2::new(-0.5, 0.5), Vec2::new(1.5, 0.8));
        assert_eq!(rt.anchor_min, Vec2::new(0.0, 0.5));
        assert_eq!(rt.anchor_max, Vec2::new(1.0, 0.8));
        assert!(rt.dirty);
    }

    #[test]
    fn test_contains_point() {
        let mut rt = RectTransform::default();
        // Set up a rect manually for testing
        rt.rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        assert!(rt.contains_point(Vec2::new(50.0, 50.0)));
        assert!(rt.contains_point(Vec2::new(0.0, 0.0)));
        assert!(rt.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!rt.contains_point(Vec2::new(-10.0, 50.0)));
        assert!(!rt.contains_point(Vec2::new(110.0, 50.0)));
    }
}

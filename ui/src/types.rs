//! Core type definitions for the UI system

use serde::{Deserialize, Serialize};
use glam::Vec2;

/// RGBA color represented as [r, g, b, a] with values in range [0.0, 1.0]
pub type Color = [f32; 4];

/// A rectangle defined by position and size
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// X position (left edge)
    pub x: f32,
    /// Y position (bottom edge)
    pub y: f32,
    /// Width of the rectangle
    pub width: f32,
    /// Height of the rectangle
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from min and max points
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }

    /// Get the center point of the rectangle
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// Get the minimum point (bottom-left corner)
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get the maximum point (top-right corner)
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Check if this rectangle overlaps with another
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_from_min_max() {
        let min = Vec2::new(10.0, 20.0);
        let max = Vec2::new(110.0, 70.0);
        let rect = Rect::from_min_max(min, max);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!(center, Vec2::new(50.0, 25.0));
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(Vec2::new(50.0, 40.0)));
        assert!(rect.contains(Vec2::new(10.0, 20.0))); // Edge case: min corner
        assert!(rect.contains(Vec2::new(110.0, 70.0))); // Edge case: max corner
        assert!(!rect.contains(Vec2::new(5.0, 40.0))); // Outside left
        assert!(!rect.contains(Vec2::new(115.0, 40.0))); // Outside right
        assert!(!rect.contains(Vec2::new(50.0, 15.0))); // Outside bottom
        assert!(!rect.contains(Vec2::new(50.0, 75.0))); // Outside top
    }

    #[test]
    fn test_rect_overlaps() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let rect3 = Rect::new(200.0, 200.0, 100.0, 100.0);

        assert!(rect1.overlaps(&rect2));
        assert!(rect2.overlaps(&rect1));
        assert!(!rect1.overlaps(&rect3));
        assert!(!rect3.overlaps(&rect1));
    }
}

//! Layout system for automatic UI element arrangement

use serde::{Deserialize, Serialize};
use glam::{Vec2, Vec4};

/// Horizontal layout group component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HorizontalLayoutGroup {
    /// Padding around the layout - left, bottom, right, top
    pub padding: Vec4,
    
    /// Spacing between elements
    pub spacing: f32,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Whether to force expand children width
    pub child_force_expand_width: bool,
    
    /// Whether to force expand children height
    pub child_force_expand_height: bool,
    
    /// Whether to control child width
    pub child_control_width: bool,
    
    /// Whether to control child height
    pub child_control_height: bool,
}

impl Default for HorizontalLayoutGroup {
    fn default() -> Self {
        Self {
            padding: Vec4::ZERO,
            spacing: 0.0,
            child_alignment: Alignment::UpperLeft,
            child_force_expand_width: true,
            child_force_expand_height: true,
            child_control_width: true,
            child_control_height: true,
        }
    }
}

/// Vertical layout group component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerticalLayoutGroup {
    /// Padding around the layout - left, bottom, right, top
    pub padding: Vec4,
    
    /// Spacing between elements
    pub spacing: f32,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Whether to force expand children width
    pub child_force_expand_width: bool,
    
    /// Whether to force expand children height
    pub child_force_expand_height: bool,
    
    /// Whether to control child width
    pub child_control_width: bool,
    
    /// Whether to control child height
    pub child_control_height: bool,
}

impl Default for VerticalLayoutGroup {
    fn default() -> Self {
        Self {
            padding: Vec4::ZERO,
            spacing: 0.0,
            child_alignment: Alignment::UpperLeft,
            child_force_expand_width: true,
            child_force_expand_height: true,
            child_control_width: true,
            child_control_height: true,
        }
    }
}

/// Grid layout group component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridLayoutGroup {
    /// Padding around the layout - left, bottom, right, top
    pub padding: Vec4,
    
    /// Cell size
    pub cell_size: Vec2,
    
    /// Spacing between cells
    pub spacing: Vec2,
    
    /// Start corner
    pub start_corner: Corner,
    
    /// Start axis
    pub start_axis: Axis,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Constraint mode
    pub constraint: GridConstraint,
    
    /// Constraint count (for FixedColumnCount or FixedRowCount)
    pub constraint_count: i32,
}

impl Default for GridLayoutGroup {
    fn default() -> Self {
        Self {
            padding: Vec4::ZERO,
            cell_size: Vec2::new(100.0, 100.0),
            spacing: Vec2::ZERO,
            start_corner: Corner::UpperLeft,
            start_axis: Axis::Horizontal,
            child_alignment: Alignment::UpperLeft,
            constraint: GridConstraint::Flexible,
            constraint_count: 2,
        }
    }
}

/// Grid constraint mode
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GridConstraint {
    Flexible,
    FixedColumnCount,
    FixedRowCount,
}

/// Alignment options for layout groups
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Alignment {
    UpperLeft,
    UpperCenter,
    UpperRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    LowerLeft,
    LowerCenter,
    LowerRight,
}

/// Corner position for grid layout
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Corner {
    UpperLeft,
    UpperRight,
    LowerLeft,
    LowerRight,
}

/// Axis direction for grid layout
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

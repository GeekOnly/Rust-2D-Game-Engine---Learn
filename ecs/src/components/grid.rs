use serde::{Deserialize, Serialize};

/// Grid component for tilemap organization
/// 
/// Similar to Unity's Grid component, this defines the cell layout
/// and coordinate system for tilemaps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    /// Cell size (width, height) in world units
    pub cell_size: (f32, f32),
    
    /// Gap between cells (spacing)
    pub cell_gap: (f32, f32),
    
    /// Grid layout type
    pub layout: GridLayout,
    
    /// Axis swizzle for 3D grids
    pub swizzle: CellSwizzle,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cell_size: (1.0, 1.0),
            cell_gap: (0.0, 0.0),
            layout: GridLayout::Rectangle,
            swizzle: CellSwizzle::XYZ,
        }
    }
}

impl Grid {
    /// Create a new grid with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a grid with custom cell size
    pub fn with_cell_size(width: f32, height: f32) -> Self {
        Self {
            cell_size: (width, height),
            ..Default::default()
        }
    }
    
    /// Convert cell coordinates to world position
    pub fn cell_to_world(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        match self.layout {
            GridLayout::Rectangle => self.cell_to_world_rectangle(cell_x, cell_y),
            GridLayout::Hexagon(orientation) => self.cell_to_world_hexagon(cell_x, cell_y, orientation),
            GridLayout::Isometric => self.cell_to_world_isometric(cell_x, cell_y),
        }
    }
    
    /// Convert world position to cell coordinates
    pub fn world_to_cell(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        match self.layout {
            GridLayout::Rectangle => self.world_to_cell_rectangle(world_x, world_y),
            GridLayout::Hexagon(orientation) => self.world_to_cell_hexagon(world_x, world_y, orientation),
            GridLayout::Isometric => self.world_to_cell_isometric(world_x, world_y),
        }
    }
    
    /// Get cell center in world coordinates
    pub fn get_cell_center(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        let (x, y) = self.cell_to_world(cell_x, cell_y);
        (
            x + self.cell_size.0 / 2.0,
            y + self.cell_size.1 / 2.0,
        )
    }
    
    // Rectangle layout conversions
    fn cell_to_world_rectangle(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        let x = cell_x as f32 * (self.cell_size.0 + self.cell_gap.0);
        let y = cell_y as f32 * (self.cell_size.1 + self.cell_gap.1);
        (x, y)
    }
    
    fn world_to_cell_rectangle(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let cell_x = (world_x / (self.cell_size.0 + self.cell_gap.0)).floor() as i32;
        let cell_y = (world_y / (self.cell_size.1 + self.cell_gap.1)).floor() as i32;
        (cell_x, cell_y)
    }
    
    // Hexagon layout conversions
    fn cell_to_world_hexagon(&self, cell_x: i32, cell_y: i32, orientation: HexagonOrientation) -> (f32, f32) {
        match orientation {
            HexagonOrientation::FlatTop => {
                let x = cell_x as f32 * (self.cell_size.0 * 0.75 + self.cell_gap.0);
                let y = cell_y as f32 * (self.cell_size.1 + self.cell_gap.1) 
                    + if cell_x % 2 != 0 { self.cell_size.1 / 2.0 } else { 0.0 };
                (x, y)
            }
            HexagonOrientation::PointyTop => {
                let x = cell_x as f32 * (self.cell_size.0 + self.cell_gap.0)
                    + if cell_y % 2 != 0 { self.cell_size.0 / 2.0 } else { 0.0 };
                let y = cell_y as f32 * (self.cell_size.1 * 0.75 + self.cell_gap.1);
                (x, y)
            }
        }
    }
    
    fn world_to_cell_hexagon(&self, world_x: f32, world_y: f32, orientation: HexagonOrientation) -> (i32, i32) {
        // Simplified hexagon conversion (can be improved with proper hex math)
        match orientation {
            HexagonOrientation::FlatTop => {
                let cell_x = (world_x / (self.cell_size.0 * 0.75 + self.cell_gap.0)).round() as i32;
                let offset = if cell_x % 2 != 0 { self.cell_size.1 / 2.0 } else { 0.0 };
                let cell_y = ((world_y - offset) / (self.cell_size.1 + self.cell_gap.1)).round() as i32;
                (cell_x, cell_y)
            }
            HexagonOrientation::PointyTop => {
                let cell_y = (world_y / (self.cell_size.1 * 0.75 + self.cell_gap.1)).round() as i32;
                let offset = if cell_y % 2 != 0 { self.cell_size.0 / 2.0 } else { 0.0 };
                let cell_x = ((world_x - offset) / (self.cell_size.0 + self.cell_gap.0)).round() as i32;
                (cell_x, cell_y)
            }
        }
    }
    
    // Isometric layout conversions
    fn cell_to_world_isometric(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        let x = (cell_x - cell_y) as f32 * (self.cell_size.0 / 2.0);
        let y = (cell_x + cell_y) as f32 * (self.cell_size.1 / 2.0);
        (x, y)
    }
    
    fn world_to_cell_isometric(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let cell_x = ((world_x / (self.cell_size.0 / 2.0) + world_y / (self.cell_size.1 / 2.0)) / 2.0).floor() as i32;
        let cell_y = ((world_y / (self.cell_size.1 / 2.0) - world_x / (self.cell_size.0 / 2.0)) / 2.0).floor() as i32;
        (cell_x, cell_y)
    }
}

/// Grid layout type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridLayout {
    /// Standard rectangular grid
    Rectangle,
    
    /// Hexagonal grid
    Hexagon(HexagonOrientation),
    
    /// Isometric grid (2:1 ratio)
    Isometric,
}

/// Hexagon orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HexagonOrientation {
    /// Flat top hexagons (⬡)
    FlatTop,
    
    /// Pointy top hexagons (⬢)
    PointyTop,
}

/// Cell swizzle for 3D grids
/// 
/// Defines which axis maps to which direction in 3D space
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellSwizzle {
    /// X=right, Y=up, Z=forward (default)
    XYZ,
    
    /// X=right, Z=up, Y=forward
    XZY,
    
    /// Y=right, X=up, Z=forward
    YXZ,
    
    /// Y=right, Z=up, X=forward
    YZX,
    
    /// Z=right, X=up, Y=forward
    ZXY,
    
    /// Z=right, Y=up, X=forward
    ZYX,
}

impl CellSwizzle {
    /// Apply swizzle to a 3D vector
    pub fn apply(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        match self {
            CellSwizzle::XYZ => (x, y, z),
            CellSwizzle::XZY => (x, z, y),
            CellSwizzle::YXZ => (y, x, z),
            CellSwizzle::YZX => (y, z, x),
            CellSwizzle::ZXY => (z, x, y),
            CellSwizzle::ZYX => (z, y, x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_grid() {
        let grid = Grid::with_cell_size(1.0, 1.0);
        
        // Test cell to world
        assert_eq!(grid.cell_to_world(0, 0), (0.0, 0.0));
        assert_eq!(grid.cell_to_world(1, 0), (1.0, 0.0));
        assert_eq!(grid.cell_to_world(0, 1), (0.0, 1.0));
        assert_eq!(grid.cell_to_world(5, 3), (5.0, 3.0));
        
        // Test world to cell
        assert_eq!(grid.world_to_cell(0.0, 0.0), (0, 0));
        assert_eq!(grid.world_to_cell(1.5, 0.5), (1, 0));
        assert_eq!(grid.world_to_cell(5.9, 3.9), (5, 3));
    }

    #[test]
    fn test_grid_with_gap() {
        let mut grid = Grid::with_cell_size(1.0, 1.0);
        grid.cell_gap = (0.1, 0.1);
        
        // With gap, cells are further apart
        assert_eq!(grid.cell_to_world(1, 0), (1.1, 0.0));
        assert_eq!(grid.cell_to_world(0, 1), (0.0, 1.1));
    }

    #[test]
    fn test_cell_center() {
        let grid = Grid::with_cell_size(2.0, 2.0);
        
        // Center should be at cell position + half size
        assert_eq!(grid.get_cell_center(0, 0), (1.0, 1.0));
        assert_eq!(grid.get_cell_center(1, 1), (3.0, 3.0));
    }

    #[test]
    fn test_isometric_grid() {
        let mut grid = Grid::with_cell_size(2.0, 1.0);
        grid.layout = GridLayout::Isometric;
        
        // Test basic isometric conversion
        let (x, y) = grid.cell_to_world(1, 0);
        assert_eq!(x, 1.0);
        assert_eq!(y, 0.5);
        
        let (x, y) = grid.cell_to_world(0, 1);
        assert_eq!(x, -1.0);
        assert_eq!(y, 0.5);
    }

    #[test]
    fn test_swizzle() {
        assert_eq!(CellSwizzle::XYZ.apply(1.0, 2.0, 3.0), (1.0, 2.0, 3.0));
        assert_eq!(CellSwizzle::XZY.apply(1.0, 2.0, 3.0), (1.0, 3.0, 2.0));
        assert_eq!(CellSwizzle::YXZ.apply(1.0, 2.0, 3.0), (2.0, 1.0, 3.0));
    }
}

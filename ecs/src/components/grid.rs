use serde::{Deserialize, Serialize};

/// Grid component for tilemap organization
/// 
/// Similar to Unity's Grid component, this defines the cell layout
/// and coordinate system for tilemaps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    /// Cell size (width, height, depth) in world units
    /// For 2D grids, depth (Z) is typically 0
    pub cell_size: (f32, f32, f32),
    
    /// Gap between cells (spacing)
    pub cell_gap: (f32, f32),
    
    /// Grid layout type
    pub layout: GridLayout,
    
    /// Axis swizzle for 3D grids
    pub swizzle: CellSwizzle,
    
    /// Grid plane orientation (Unity-style)
    /// Determines which plane the grid lies on (XY, XZ, or YZ)
    #[serde(default)]
    pub plane: GridPlane,
}

impl Default for Grid {
    fn default() -> Self {
        // Standard 1x1 world unit grid cells
        // This provides consistent reference for object sizing between 2D and 3D modes
        // Tilemaps render at their actual tile size (e.g., 8px = 0.08, 16px = 0.16 world units at 100 PPU)
        
        Self {
            cell_size: (1.0, 1.0, 0.0),  // Standard 1x1 world unit grid cells
            cell_gap: (0.0, 0.0),
            layout: GridLayout::Rectangle,
            swizzle: CellSwizzle::XYZ,
            plane: GridPlane::XY,  // Default to XY plane (2D horizontal)
        }
    }
}

impl Grid {
    /// Create a new grid with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a grid with custom cell size (2D)
    pub fn with_cell_size(width: f32, height: f32) -> Self {
        Self {
            cell_size: (width, height, 0.0),
            ..Default::default()
        }
    }
    
    /// Create a grid with custom cell size (3D)
    pub fn with_cell_size_3d(width: f32, height: f32, depth: f32) -> Self {
        Self {
            cell_size: (width, height, depth),
            ..Default::default()
        }
    }
    
    /// Create a vertical grid (XZ plane) - Unity style
    pub fn vertical() -> Self {
        Self {
            cell_size: (1.0, 1.0, 0.0),
            cell_gap: (0.0, 0.0),
            layout: GridLayout::Rectangle,
            swizzle: CellSwizzle::XZY,  // X=right, Z=up, Y=forward
            plane: GridPlane::XZ,
        }
    }
    
    /// Create a side grid (YZ plane) - Unity style
    pub fn side() -> Self {
        Self {
            cell_size: (1.0, 1.0, 0.0),
            cell_gap: (0.0, 0.0),
            layout: GridLayout::Rectangle,
            swizzle: CellSwizzle::YZX,  // Y=right, Z=up, X=forward
            plane: GridPlane::YZ,
        }
    }
    
    /// Switch to 3D mode (XZ plane for walls/vertical tilemaps)
    pub fn to_3d_mode(&mut self) {
        self.plane = GridPlane::XZ;
        self.swizzle = CellSwizzle::XZY;  // X=right, Z=up, Y=forward
    }
    
    /// Switch to 2D mode (XY plane for horizontal tilemaps)
    pub fn to_2d_mode(&mut self) {
        self.plane = GridPlane::XY;
        self.swizzle = CellSwizzle::XYZ;  // X=right, Y=up, Z=forward
    }
    
    /// Check if grid is in 3D mode
    pub fn is_3d_mode(&self) -> bool {
        matches!(self.plane, GridPlane::XZ | GridPlane::YZ)
    }
    
    /// Set cell size to match tilemap tile size
    pub fn set_cell_size_from_tilemap(&mut self, tile_width: u32, tile_height: u32, pixels_per_unit: f32) {
        let cell_width = tile_width as f32 / pixels_per_unit;
        let cell_height = tile_height as f32 / pixels_per_unit;
        self.cell_size = (cell_width, cell_height, self.cell_size.2);
    }
    
    /// Set cell size to Unity standard (1x1 world unit grid cells)
    pub fn set_standard_cell_size(&mut self) {
        self.cell_size = (1.0, 1.0, 0.0);
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
    
    /// Get cell center in world coordinates (2D)
    pub fn get_cell_center(&self, cell_x: i32, cell_y: i32) -> (f32, f32) {
        let (x, y) = self.cell_to_world(cell_x, cell_y);
        (
            x + self.cell_size.0 / 2.0,
            y + self.cell_size.1 / 2.0,
        )
    }
    
    /// Get cell center in 3D world coordinates
    /// Takes grid plane into account
    pub fn get_cell_center_3d(&self, cell_x: i32, cell_y: i32, cell_z: i32) -> (f32, f32, f32) {
        let (x, y) = self.cell_to_world(cell_x, cell_y);
        let z = cell_z as f32 * (self.cell_size.2 + self.cell_gap.0);
        
        // Apply plane transformation
        match self.plane {
            GridPlane::XY => (
                x + self.cell_size.0 / 2.0,
                y + self.cell_size.1 / 2.0,
                z + self.cell_size.2 / 2.0,
            ),
            GridPlane::XZ => (
                x + self.cell_size.0 / 2.0,
                z + self.cell_size.2 / 2.0,
                y + self.cell_size.1 / 2.0,
            ),
            GridPlane::YZ => (
                z + self.cell_size.2 / 2.0,
                x + self.cell_size.0 / 2.0,
                y + self.cell_size.1 / 2.0,
            ),
        }
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

/// Grid plane orientation (Unity-style)
/// 
/// Determines which 2D plane the grid lies on in 3D space
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridPlane {
    /// XY plane (horizontal, default for 2D games)
    /// X = right, Y = up, Z = forward/depth
    XY,
    
    /// XZ plane (vertical, like walls)
    /// X = right, Z = up, Y = forward/depth
    XZ,
    
    /// YZ plane (side view)
    /// Y = right, Z = up, X = forward/depth
    YZ,
}

impl Default for GridPlane {
    fn default() -> Self {
        GridPlane::XY
    }
}

impl GridPlane {
    /// Get the "right" axis for this plane
    pub fn right_axis(&self) -> (f32, f32, f32) {
        match self {
            GridPlane::XY => (1.0, 0.0, 0.0),  // X
            GridPlane::XZ => (1.0, 0.0, 0.0),  // X
            GridPlane::YZ => (0.0, 1.0, 0.0),  // Y
        }
    }
    
    /// Get the "up" axis for this plane
    pub fn up_axis(&self) -> (f32, f32, f32) {
        match self {
            GridPlane::XY => (0.0, 1.0, 0.0),  // Y
            GridPlane::XZ => (0.0, 0.0, 1.0),  // Z
            GridPlane::YZ => (0.0, 0.0, 1.0),  // Z
        }
    }
    
    /// Get the "forward" axis (depth) for this plane
    pub fn forward_axis(&self) -> (f32, f32, f32) {
        match self {
            GridPlane::XY => (0.0, 0.0, 1.0),  // Z
            GridPlane::XZ => (0.0, 1.0, 0.0),  // Y
            GridPlane::YZ => (1.0, 0.0, 0.0),  // X
        }
    }
}

/// Cell swizzle for 3D grids
/// 
/// Defines which axis maps to which direction in 3D space
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellSwizzle {
    /// X=right, Y=up, Z=forward (default)
    XYZ,
    
    /// X=right, Z=up, Y=forward (vertical grid)
    XZY,
    
    /// Y=right, X=up, Z=forward
    YXZ,
    
    /// Y=right, Z=up, X=forward (side grid)
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
    
    #[test]
    fn test_vertical_grid() {
        let grid = Grid::vertical();
        
        // Vertical grid should use XZ plane
        assert_eq!(grid.plane, GridPlane::XZ);
        assert_eq!(grid.swizzle, CellSwizzle::XZY);
        
        // Test 3D cell center
        let (x, y, z) = grid.get_cell_center_3d(0, 0, 0);
        assert_eq!(x, 0.5);  // X center
        assert_eq!(y, 0.0);  // Y (depth) = 0
        assert_eq!(z, 0.5);  // Z center (up)
    }
    
    #[test]
    fn test_grid_planes() {
        // XY plane (horizontal)
        assert_eq!(GridPlane::XY.right_axis(), (1.0, 0.0, 0.0));
        assert_eq!(GridPlane::XY.up_axis(), (0.0, 1.0, 0.0));
        assert_eq!(GridPlane::XY.forward_axis(), (0.0, 0.0, 1.0));
        
        // XZ plane (vertical)
        assert_eq!(GridPlane::XZ.right_axis(), (1.0, 0.0, 0.0));
        assert_eq!(GridPlane::XZ.up_axis(), (0.0, 0.0, 1.0));
        assert_eq!(GridPlane::XZ.forward_axis(), (0.0, 1.0, 0.0));
        
        // YZ plane (side)
        assert_eq!(GridPlane::YZ.right_axis(), (0.0, 1.0, 0.0));
        assert_eq!(GridPlane::YZ.up_axis(), (0.0, 0.0, 1.0));
        assert_eq!(GridPlane::YZ.forward_axis(), (1.0, 0.0, 0.0));
    }
}

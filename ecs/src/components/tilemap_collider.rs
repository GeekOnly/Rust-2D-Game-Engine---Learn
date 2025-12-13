use serde::{Deserialize, Serialize};

/// Tilemap Collider component for generating collision from tilemap data
/// Similar to Unity's TilemapCollider2D
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilemapCollider {
    /// Collider generation mode
    pub mode: TilemapColliderMode,
    
    /// Which tile IDs should generate colliders (empty = all non-zero tiles)
    pub collision_tiles: Vec<u32>,
    
    /// Collision layer/category for physics filtering
    pub collision_layer: u32,
    
    /// Physics material properties
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    
    /// Whether to use composite collider (merge adjacent tiles)
    pub use_composite: bool,
    
    /// Offset for generated colliders
    pub offset: [f32; 2],
    
    /// Whether collider is a trigger (no physics response)
    pub is_trigger: bool,
    
    /// Auto-update colliders when tilemap changes
    pub auto_update: bool,
}

impl Default for TilemapCollider {
    fn default() -> Self {
        Self {
            mode: TilemapColliderMode::Individual,
            collision_tiles: Vec::new(),  // Empty = all non-zero tiles
            collision_layer: 0,
            friction: 0.4,
            restitution: 0.0,
            density: 1.0,
            use_composite: true,
            offset: [0.0, 0.0],
            is_trigger: false,
            auto_update: true,
        }
    }
}

impl TilemapCollider {
    /// Create a new tilemap collider
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with specific collision tiles
    pub fn with_collision_tiles(collision_tiles: Vec<u32>) -> Self {
        Self {
            collision_tiles,
            ..Default::default()
        }
    }
    
    /// Create as trigger collider
    pub fn as_trigger() -> Self {
        Self {
            is_trigger: true,
            ..Default::default()
        }
    }
    
    /// Check if a tile ID should generate collision
    pub fn should_collide(&self, tile_id: u32) -> bool {
        if tile_id == 0 {
            return false; // Empty tiles never collide
        }
        
        if self.collision_tiles.is_empty() {
            return true; // All non-zero tiles collide
        }
        
        self.collision_tiles.contains(&tile_id)
    }
}

/// Tilemap collider generation mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TilemapColliderMode {
    /// Generate individual box colliders for each tile
    Individual,
    
    /// Generate optimized composite colliders (merge adjacent tiles)
    Composite,
    
    /// Generate polygon colliders based on tile shapes
    Polygon,
    
    /// No collision generation
    None,
}

/// LDtk IntGrid Collider - specialized for LDtk IntGrid layers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkIntGridCollider {
    /// IntGrid value that represents collision (e.g., 1 = solid)
    pub collision_value: i32,
    
    /// Collider generation mode
    pub mode: TilemapColliderMode,
    
    /// Physics properties
    pub friction: f32,
    pub restitution: f32,
    pub is_trigger: bool,
    
    /// Auto-update when IntGrid changes
    pub auto_update: bool,
}

impl Default for LdtkIntGridCollider {
    fn default() -> Self {
        Self {
            collision_value: 1, // Default LDtk solid value
            mode: TilemapColliderMode::Composite,
            friction: 0.4,
            restitution: 0.0,
            is_trigger: false,
            auto_update: true,
        }
    }
}

impl LdtkIntGridCollider {
    /// Create for specific collision value
    pub fn with_value(collision_value: i32) -> Self {
        Self {
            collision_value,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilemap_collider_default() {
        let collider = TilemapCollider::default();
        assert_eq!(collider.mode, TilemapColliderMode::Individual);
        assert!(collider.collision_tiles.is_empty());
        assert!(collider.use_composite);
        assert!(!collider.is_trigger);
        assert!(collider.auto_update);
    }

    #[test]
    fn test_should_collide() {
        let mut collider = TilemapCollider::new();
        
        // Empty tiles never collide
        assert!(!collider.should_collide(0));
        
        // With empty collision_tiles, all non-zero tiles collide
        assert!(collider.should_collide(1));
        assert!(collider.should_collide(42));
        
        // With specific collision tiles
        collider.collision_tiles = vec![1, 5, 10];
        assert!(collider.should_collide(1));
        assert!(collider.should_collide(5));
        assert!(collider.should_collide(10));
        assert!(!collider.should_collide(2));
        assert!(!collider.should_collide(42));
    }

    #[test]
    fn test_ldtk_intgrid_collider() {
        let collider = LdtkIntGridCollider::with_value(2);
        assert_eq!(collider.collision_value, 2);
        assert_eq!(collider.mode, TilemapColliderMode::Composite);
    }
}
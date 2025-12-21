use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod traits;
pub mod component_manager;
pub mod components;
pub mod loaders;
pub mod backends;
pub mod benchmark_runner;

// Re-export สำหรับใช้งานง่าย
pub use component_manager::{ComponentType, ComponentManager};
pub use components::*;
pub use backends::{EcsBackendType, DynamicWorld, BackendPerformanceInfo, PerformanceLevel};
pub use benchmark_runner::{BenchmarkRunner, BenchmarkSuite, BenchmarkResult};

// ----------------------------------------------------------------------------
// Backend Selection
// ----------------------------------------------------------------------------

#[cfg(feature = "hecs")]
pub use backends::hecs_minimal::HecsMinimal as World;

#[cfg(not(feature = "hecs"))]
pub use CustomWorld as World;

#[cfg(feature = "hecs")]
pub use hecs::Entity;

#[cfg(not(feature = "hecs"))]
pub type Entity = u32;

// ----------------------------------------------------------------------------

/// Prefab system for reusable entity templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub transform: Transform,
    pub sprite: Option<Sprite>,
    pub collider: Option<Collider>,
    pub rigidbody: Option<Rigidbody2D>,
    pub velocity: Option<(f32, f32)>,  // Legacy - kept for backward compatibility
    pub tag: Option<EntityTag>,
    pub script: Option<Script>,
    pub camera: Option<Camera>,
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: None,
        }
    }

    /// Create a Player prefab with Rigidbody
    pub fn player() -> Self {
        Self {
            name: "Player".to_string(),
            transform: Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [40.0, 40.0, 1.0], // Use scale for sprite size
            },
            sprite: Some(Sprite {
                texture_id: "player".to_string(),
                width: 1.0,  // Base size
                height: 1.0,
                color: [0.2, 0.6, 1.0, 1.0],
                billboard: true, // Player sprite faces camera (good for 3D mode)
                ..Default::default()
            }),
            collider: Some(Collider::default()),
            rigidbody: Some(Rigidbody2D::default()),  // Add rigidbody with default values
            velocity: Some((0.0, 0.0)),
            tag: Some(EntityTag::Player),
            script: None,
            camera: None,
        }
    }

    /// Create an Item prefab
    pub fn item() -> Self {
        Self {
            name: "Item".to_string(),
            transform: Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [30.0, 30.0, 1.0], // Use scale for sprite size
            },
            sprite: Some(Sprite {
                texture_id: "item".to_string(),
                width: 1.0,  // Base size
                height: 1.0,
                color: [1.0, 0.8, 0.2, 1.0],
                billboard: true, // Item sprite faces camera
                ..Default::default()
            }),
            collider: Some(Collider::default()),
            rigidbody: None,
            velocity: None,
            tag: Some(EntityTag::Item),
            script: None,
            camera: None,
        }
    }

    /// Create a 2D Camera prefab
    pub fn camera_2d() -> Self {
        Self {
            name: "Camera 2D".to_string(),
            transform: Transform::with_position(0.0, 0.0, -10.0), // Camera behind objects
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: Some(Camera::orthographic_2d()),
        }
    }

    /// Create a 3D Camera prefab
    pub fn camera_3d() -> Self {
        Self {
            name: "Camera 3D".to_string(),
            transform: Transform::with_position(0.0, 5.0, -10.0), // Camera above and behind
            sprite: None,
            collider: None,
            rigidbody: None,
            velocity: None,
            tag: None,
            script: None,
            camera: Some(Camera::perspective_3d()),
        }
    }

    /// Spawn this prefab into the world
    pub fn spawn(&self, world: &mut World) -> Entity {
        use crate::traits::ComponentAccess;

        let entity = world.spawn();
        
        // Use explicit trait method calls to avoid ambiguity
        let _ = ComponentAccess::<Transform>::insert(world, entity, self.transform.clone());
        let _ = ComponentAccess::<String>::insert(world, entity, self.name.clone());

        if let Some(sprite) = &self.sprite {
            let _ = ComponentAccess::<Sprite>::insert(world, entity, sprite.clone());
        }
        if let Some(collider) = &self.collider {
            let _ = ComponentAccess::<Collider>::insert(world, entity, collider.clone());
        }
        if let Some(rigidbody) = &self.rigidbody {
            let _ = ComponentAccess::<Rigidbody2D>::insert(world, entity, rigidbody.clone());
            // Sync velocity from rigidbody
            let _ = ComponentAccess::<(f32, f32)>::insert(world, entity, rigidbody.velocity);
        } else if let Some(velocity) = self.velocity {
            // Legacy velocity support
            let _ = ComponentAccess::<(f32, f32)>::insert(world, entity, velocity);
        }
        if let Some(tag) = &self.tag {
            let _ = ComponentAccess::<EntityTag>::insert(world, entity, tag.clone());
        }
        if let Some(script) = &self.script {
            let _ = ComponentAccess::<Script>::insert(world, entity, script.clone());
        }
        if let Some(camera) = &self.camera {
            let _ = ComponentAccess::<Camera>::insert(world, entity, camera.clone());
        }

        entity
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],  // X, Y, Z
    pub rotation: [f32; 3],  // Euler angles: X, Y, Z (in degrees)
    pub scale: [f32; 3],     // X, Y, Z
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

// Helper methods for backward compatibility and convenience
impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    pub fn with_position_2d(x: f32, y: f32) -> Self {
        Self::with_position(x, y, 0.0)
    }

    // Getters for convenience
    pub fn x(&self) -> f32 { self.position[0] }
    pub fn y(&self) -> f32 { self.position[1] }
    pub fn z(&self) -> f32 { self.position[2] }

    // Setters for convenience
    pub fn set_x(&mut self, x: f32) { self.position[0] = x; }
    pub fn set_y(&mut self, y: f32) { self.position[1] = y; }
    pub fn set_z(&mut self, z: f32) { self.position[2] = z; }
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = [x, y, z];
    }
}

/// Computed Global Transform (World Matrix)
/// This is derived from the hierarchy chain (Parent * Child)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalTransform {
    pub matrix: [f32; 16], // 4x4 matrix column-major
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, 0.0, // Col 1
                0.0, 1.0, 0.0, 0.0, // Col 2
                0.0, 0.0, 1.0, 0.0, // Col 3
                0.0, 0.0, 0.0, 1.0, // Col 4
            ],
        }
    }
}

impl GlobalTransform {
    pub fn identity() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_id: String,
    /// Original sprite width in pixels (use Transform.scale for sizing)
    pub width: f32,
    /// Original sprite height in pixels (use Transform.scale for sizing)
    pub height: f32,
    pub color: [f32; 4], // RGBA
    pub billboard: bool, // If true, sprite always faces camera (3D mode only)
    /// Flip sprite horizontally
    #[serde(default)]
    pub flip_x: bool,
    /// Flip sprite vertically
    #[serde(default)]
    pub flip_y: bool,
    /// Sprite rect in texture (Unity-style) - [x, y, width, height] in pixels
    /// If None, uses full texture. If Some, uses sub-region of texture.
    #[serde(default)]
    pub sprite_rect: Option<[u32; 4]>,
    /// Pixels Per Unit (Unity-style) - how many pixels equal 1 world unit
    /// Default is 100 (like Unity). Lower values = larger sprites in world.
    #[serde(default = "default_pixels_per_unit")]
    pub pixels_per_unit: f32,
    
    /// Sorting Layer (Unity-style) - Group sprites
    #[serde(default = "default_sorting_layer")]
    pub sorting_layer: String,
    
    /// Order within the layer (Higher = On top)
    #[serde(default)]
    pub order_in_layer: i32,
    
    /// Mask for camera culling/lighting (Bitmask)
    #[serde(default = "default_rendering_layer_mask")]
    pub rendering_layer_mask: u32,
}

fn default_sorting_layer() -> String {
    "Default".to_string()
}

fn default_rendering_layer_mask() -> u32 {
    1 // Default Layer (Bit 0)
}

fn default_pixels_per_unit() -> f32 {
    100.0  // Unity standard: 100 pixels = 1 world unit (1 meter) - compatible with 2.5D/3D
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_id: String::new(),
            width: 1.0,  // Default 1x1 pixel
            height: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            flip_x: false,
            flip_y: false,
            sprite_rect: None,
            pixels_per_unit: 100.0,  // Unity standard
            sorting_layer: default_sorting_layer(),
            order_in_layer: 0,
            rendering_layer_mask: default_rendering_layer_mask(),
        }
    }
}

impl Sprite {
    /// Create a new sprite with texture ID and original size
    pub fn new(texture_id: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            texture_id: texture_id.into(),
            width,
            height,
            ..Default::default()
        }
    }
    
    /// Get the actual rendered width (original width * scale)
    pub fn get_rendered_width(&self, scale_x: f32) -> f32 {
        self.width * scale_x * if self.flip_x { -1.0 } else { 1.0 }
    }
    
    /// Get the actual rendered height (original height * scale)
    pub fn get_rendered_height(&self, scale_y: f32) -> f32 {
        self.height * scale_y * if self.flip_y { -1.0 } else { 1.0 }
    }
}

/// Box Collider 2D (Unity-like)
/// Size is relative to Transform.scale
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    /// Offset from entity center (in local space)
    #[serde(default)]
    pub offset: [f32; 2],
    /// Size of collider (default 1.0, actual size = size * transform.scale)
    #[serde(default = "default_collider_size")]
    pub size: [f32; 2],
    /// Legacy width (for backward compatibility)
    #[serde(default)]
    pub width: f32,
    /// Legacy height (for backward compatibility)
    #[serde(default)]
    pub height: f32,
}

fn default_collider_size() -> [f32; 2] {
    [1.0, 1.0]
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            width: 0.0,
            height: 0.0,
        }
    }
}

impl Collider {
    /// Create a new collider with size
    pub fn new(size_x: f32, size_y: f32) -> Self {
        Self {
            offset: [0.0, 0.0],
            size: [size_x, size_y],
            width: 0.0,
            height: 0.0,
        }
    }
    
    /// Create a collider with offset and size
    pub fn with_offset(offset_x: f32, offset_y: f32, size_x: f32, size_y: f32) -> Self {
        Self {
            offset: [offset_x, offset_y],
            size: [size_x, size_y],
            width: 0.0,
            height: 0.0,
        }
    }
    
    /// Get actual world-space width (size.x * scale.x)
    pub fn get_world_width(&self, scale_x: f32) -> f32 {
        self.size[0] * scale_x
    }
    
    /// Get actual world-space height (size.y * scale.y)
    pub fn get_world_height(&self, scale_y: f32) -> f32 {
        self.size[1] * scale_y
    }
    
    /// Get world-space offset
    pub fn get_world_offset(&self, scale_x: f32, scale_y: f32) -> [f32; 2] {
        [self.offset[0] * scale_x, self.offset[1] * scale_y]
    }
    
    /// Migrate from legacy width/height to size
    pub fn migrate_from_legacy(&mut self, transform_scale: [f32; 3]) {
        if self.width > 0.0 || self.height > 0.0 {
            // Convert legacy width/height to size
            self.size[0] = if transform_scale[0] != 0.0 {
                self.width / transform_scale[0]
            } else {
                1.0
            };
            self.size[1] = if transform_scale[1] != 0.0 {
                self.height / transform_scale[1]
            } else {
                1.0
            };
            self.width = 0.0;
            self.height = 0.0;
        }
    }
}

/// Rigidbody 2D properties (Unity-like)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rigidbody2D {
    pub velocity: (f32, f32),      // Current velocity (vx, vy)
    pub gravity_scale: f32,         // Gravity multiplier (0 = no gravity, 1 = normal)
    pub mass: f32,                  // Mass (affects collision response)
    pub is_kinematic: bool,         // If true, not affected by physics (but still collides)
    pub freeze_rotation: bool,      // Prevent rotation (for 2D games)
    #[serde(default)]
    pub enable_ccd: bool,           // Continuous Collision Detection (prevents tunneling)
}

impl Default for Rigidbody2D {
    fn default() -> Self {
        Self {
            velocity: (0.0, 0.0),
            gravity_scale: 1.0,
            mass: 1.0,
            is_kinematic: false,
            freeze_rotation: true,
            enable_ccd: false,
        }
    }
}

/// 3D Mesh component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model3D {
    pub asset_id: String,
}

impl Default for Model3D {
    fn default() -> Self {
        Self {
            asset_id: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_type: MeshType,
    pub color: [f32; 4], // RGBA
    #[serde(default)]
    pub material_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MeshType {
    Cube,
    Sphere,
    Cylinder,
    Plane,
    Capsule,
    Asset(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EntityTag {
    Player,
    Item,
}

impl Default for EntityTag {
    fn default() -> Self {
        EntityTag::Player
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub script_name: String,
    pub enabled: bool,
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, ScriptParameter>,
    /// Lifecycle state (not serialized - runtime only)
    #[serde(skip)]
    pub lifecycle_state: ScriptLifecycleState,
}

/// Script lifecycle state (Unity-style)
#[derive(Clone, Debug, Default)]
pub struct ScriptLifecycleState {
    pub awake_called: bool,
    pub start_called: bool,
    pub enabled_called: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ScriptParameter {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
    Entity(Option<Entity>), // Unity-style GameObject reference
}

/// Camera component for view control (Unity-like)
/// Can be attached to an entity to create game cameras
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    // Camera projection settings
    pub projection: CameraProjection,

    // Field of view (for perspective projection, in degrees)
    pub fov: f32,

    // Orthographic size (for orthographic projection, half-height in world units)
    pub orthographic_size: f32,

    // Near and far clip planes
    pub near_clip: f32,
    pub far_clip: f32,

    // Viewport settings (normalized 0-1)
    pub viewport_rect: [f32; 4], // x, y, width, height

    // Camera depth (lower renders first, like Unity)
    pub depth: i32,

    // Clear flags
    pub clear_flags: CameraClearFlags,
    pub background_color: [f32; 4], // RGBA
    
    // Pixels Per Unit (Unity 2D style) - how many pixels equal 1 world unit
    // Default is 100 (like Unity). Used for pixel-perfect rendering.
    // Set to 1.0 for 1:1 pixel mapping (1 world unit = 1 pixel)
    #[serde(default = "default_camera_pixels_per_unit")]
    pub pixels_per_unit: f32,
}

fn default_camera_pixels_per_unit() -> f32 {
    100.0  // Unity standard: 100 pixels = 1 world unit (1 meter)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CameraProjection {
    Orthographic, // 2D camera
    Perspective,  // 3D camera
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CameraClearFlags {
    SolidColor,
    Skybox,
    DepthOnly,
    DontClear,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            fov: 60.0,
            orthographic_size: 5.0,
            near_clip: 0.3,
            far_clip: 1000.0,
            viewport_rect: [0.0, 0.0, 1.0, 1.0], // Full screen
            depth: 0,
            clear_flags: CameraClearFlags::SolidColor,
            background_color: [0.15, 0.16, 0.18, 1.0], // Dark gray (Unity default)
            pixels_per_unit: 100.0,  // Unity standard
        }
    }
}

impl Camera {
    /// Create a 2D orthographic camera (Unity 2D style)
    pub fn orthographic_2d() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            orthographic_size: 5.0,
            ..Default::default()
        }
    }

    /// Create a 3D perspective camera (Unity 3D style)
    pub fn perspective_3d() -> Self {
        Self {
            projection: CameraProjection::Perspective,
            fov: 60.0,
            ..Default::default()
        }
    }
    
    /// Create a 2D camera with Unity standard scale (100 pixels = 1 world unit)
    pub fn pixel_perfect_2d() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            orthographic_size: 5.0,
            pixels_per_unit: 100.0,
            ..Default::default()
        }
    }
    
    /// Create a Unity-style 2D camera (100 pixels = 1 world unit)
    /// Note: This is kept for compatibility but pixel-perfect (1.0) is recommended
    pub fn unity_2d() -> Self {
        Self {
            projection: CameraProjection::Orthographic,
            orthographic_size: 5.0,
            pixels_per_unit: 100.0,  // Unity default (kept for compatibility)
            ..Default::default()
        }
    }
    
    /// Set pixels per unit (for pixel-perfect rendering)
    pub fn with_pixels_per_unit(mut self, ppu: f32) -> Self {
        self.pixels_per_unit = ppu;
        self
    }
    
    /// Calculate zoom factor for rendering
    /// Returns how many screen pixels = 1 world unit
    pub fn get_zoom(&self) -> f32 {
        self.pixels_per_unit
    }
}

// CustomWorld available always for benchmarking/fallback
pub type CustomEntity = u32;

#[derive(Default, Clone)]
pub struct CustomWorld {
    next_entity: CustomEntity,
    pub transforms: HashMap<CustomEntity, Transform>,
    pub global_transforms: HashMap<CustomEntity, GlobalTransform>, // Computed world transform
    pub velocities: HashMap<CustomEntity, (f32, f32)>,  // Legacy - kept for backward compatibility
    pub rigidbodies: HashMap<CustomEntity, Rigidbody2D>, // New Rigidbody2D component
    pub sprites: HashMap<CustomEntity, Sprite>,
    pub colliders: HashMap<CustomEntity, Collider>,
    pub colliders_3d: HashMap<CustomEntity, Collider3D>, // 3D colliders
    pub meshes: HashMap<CustomEntity, Mesh>,      // 3D meshes
    pub cameras: HashMap<CustomEntity, Camera>,   // Camera components
    pub tags: HashMap<CustomEntity, EntityTag>,
    pub scripts: HashMap<CustomEntity, Script>,
    pub active: HashMap<CustomEntity, bool>,      // Active state (Unity-like)
    pub layers: HashMap<CustomEntity, u8>,        // Layer (0-31, Unity has 32 layers)
    pub parents: HashMap<CustomEntity, CustomEntity>,   // Parent entity
    pub children: HashMap<CustomEntity, Vec<CustomEntity>>, // Children entities
    pub names: HashMap<CustomEntity, String>,     // Entity names (for editor)
    // Sprite sheet and tilemap components
    pub sprite_sheets: HashMap<CustomEntity, SpriteSheet>,
    pub animated_sprites: HashMap<CustomEntity, AnimatedSprite>,
    pub tilemaps: HashMap<CustomEntity, Tilemap>,
    pub tilesets: HashMap<CustomEntity, TileSet>,
    pub tilemap_renderers: HashMap<CustomEntity, TilemapRenderer>,  // Tilemap renderer component
    // Map component (LDtk/Tiled integration)
    pub maps: HashMap<CustomEntity, Map>,
    // Grid component (Unity-like)
    pub grids: HashMap<CustomEntity, Grid>,
    // World-space UI components
    pub world_uis: HashMap<CustomEntity, WorldUI>,
    // LDtk Map components
    pub ldtk_maps: HashMap<CustomEntity, LdtkMap>,
    // Tilemap Collider components
    pub tilemap_colliders: HashMap<CustomEntity, TilemapCollider>,
    pub ldtk_intgrid_colliders: HashMap<CustomEntity, LdtkIntGridCollider>,
    // 3D Model component (Static Props)
    pub model_3ds: HashMap<CustomEntity, Model3D>,
}

impl CustomWorld {
    pub fn new() -> Self { Self::default() }

    pub fn spawn(&mut self) -> CustomEntity {
        let id = self.next_entity;
        self.next_entity += 1;
        // New entities are active by default (Unity behavior)
        self.active.insert(id, true);
        // New entities start on layer 0 (Default layer)
        self.layers.insert(id, 0);
        id
    }

    pub fn despawn(&mut self, e: CustomEntity) {
        // Recursively despawn children
        if let Some(children) = self.children.remove(&e) {
            for child in children {
                self.despawn(child);
            }
        }

        // Remove from parent's children list
        if let Some(parent) = self.parents.remove(&e) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&x| x != e);
            }
        }

        self.transforms.remove(&e);
        self.global_transforms.remove(&e);
        self.velocities.remove(&e);
        self.rigidbodies.remove(&e);
        self.sprites.remove(&e);
        self.colliders.remove(&e);
        self.colliders_3d.remove(&e);
        self.meshes.remove(&e);
        self.cameras.remove(&e);
        self.tags.remove(&e);
        self.scripts.remove(&e);
        self.active.remove(&e);
        self.layers.remove(&e);
        self.names.remove(&e);
        self.sprite_sheets.remove(&e);
        self.animated_sprites.remove(&e);
        self.tilemaps.remove(&e);
        self.tilesets.remove(&e);
        self.tilemap_renderers.remove(&e);
        self.maps.remove(&e);
        self.grids.remove(&e);
        self.world_uis.remove(&e);
        self.ldtk_maps.remove(&e);
        self.tilemap_colliders.remove(&e);
        self.ldtk_intgrid_colliders.remove(&e);
        self.model_3ds.remove(&e);
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.global_transforms.clear();
        self.velocities.clear();
        self.rigidbodies.clear();
        self.sprites.clear();
        self.colliders.clear();
        self.colliders_3d.clear();
        self.meshes.clear();
        self.cameras.clear();
        self.tags.clear();
        self.scripts.clear();
        self.active.clear();
        self.layers.clear();
        self.parents.clear();
        self.children.clear();
        self.names.clear();
        self.sprite_sheets.clear();
        self.animated_sprites.clear();
        self.tilemaps.clear();
        self.tilesets.clear();
        self.tilemap_renderers.clear();
        self.maps.clear();
        self.grids.clear();
        self.world_uis.clear();
        self.ldtk_maps.clear();
        self.tilemap_colliders.clear();
        self.ldtk_intgrid_colliders.clear();
        self.model_3ds.clear();
        self.next_entity = 0;
    }

    pub fn set_parent(&mut self, child: CustomEntity, parent: Option<CustomEntity>) {
        // Remove from old parent
        if let Some(old_parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&x| x != child);
            }
        }

        // Add to new parent
        if let Some(new_parent) = parent {
            self.parents.insert(child, new_parent);
            self.children.entry(new_parent).or_default().push(child);
        }
    }

    pub fn get_children(&self, entity: CustomEntity) -> &[CustomEntity] {
        self.children.get(&entity).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_parent(&self, entity: CustomEntity) -> Option<CustomEntity> {
        self.parents.get(&entity).copied()
    }

    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct SceneData {
            next_entity: CustomEntity,
            transforms: Vec<(CustomEntity, Transform)>,
            velocities: Vec<(CustomEntity, (f32, f32))>,
            sprites: Vec<(CustomEntity, Sprite)>,
            colliders: Vec<(CustomEntity, Collider)>,
            colliders_3d: Vec<(CustomEntity, Collider3D)>,
            rigidbodies: Vec<(CustomEntity, Rigidbody2D)>,  // Added rigidbody serialization
            cameras: Vec<(CustomEntity, Camera)>,
            meshes: Vec<(CustomEntity, Mesh)>,
            tags: Vec<(CustomEntity, EntityTag)>,
            scripts: Vec<(CustomEntity, Script)>,
            active: Vec<(CustomEntity, bool)>,
            layers: Vec<(CustomEntity, u8)>,
            parents: Vec<(CustomEntity, CustomEntity)>,
            names: Vec<(CustomEntity, String)>,
            sprite_sheets: Vec<(CustomEntity, SpriteSheet)>,
            animated_sprites: Vec<(CustomEntity, AnimatedSprite)>,
            tilemaps: Vec<(CustomEntity, Tilemap)>,
            tilesets: Vec<(CustomEntity, TileSet)>,
            tilemap_renderers: Vec<(CustomEntity, TilemapRenderer)>,
            grids: Vec<(CustomEntity, Grid)>,
            maps: Vec<(CustomEntity, Map)>,
            world_uis: Vec<(CustomEntity, WorldUI)>,
            model_3ds: Vec<(CustomEntity, Model3D)>,
        }

        let data = SceneData {
            next_entity: self.next_entity,
            transforms: self.transforms.iter().map(|(k, v)| (*k, v.clone())).collect(),
            velocities: self.velocities.iter().map(|(k, v)| (*k, *v)).collect(),
            sprites: self.sprites.iter().map(|(k, v)| (*k, v.clone())).collect(),
            colliders: self.colliders.iter().map(|(k, v)| (*k, v.clone())).collect(),
            colliders_3d: self.colliders_3d.iter().map(|(k, v)| (*k, v.clone())).collect(),
            rigidbodies: self.rigidbodies.iter().map(|(k, v)| (*k, v.clone())).collect(),
            cameras: self.cameras.iter().map(|(k, v)| (*k, v.clone())).collect(),
            meshes: self.meshes.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tags: self.tags.iter().map(|(k, v)| (*k, v.clone())).collect(),
            scripts: self.scripts.iter().map(|(k, v)| (*k, v.clone())).collect(),
            active: self.active.iter().map(|(k, v)| (*k, *v)).collect(),
            layers: self.layers.iter().map(|(k, v)| (*k, *v)).collect(),
            parents: self.parents.iter().map(|(k, v)| (*k, *v)).collect(),
            names: self.names.iter().map(|(k, v)| (*k, v.clone())).collect(),
            sprite_sheets: self.sprite_sheets.iter().map(|(k, v)| (*k, v.clone())).collect(),
            animated_sprites: self.animated_sprites.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tilemaps: self.tilemaps.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tilesets: self.tilesets.iter().map(|(k, v)| (*k, v.clone())).collect(),
            tilemap_renderers: self.tilemap_renderers.iter().map(|(k, v)| (*k, v.clone())).collect(),
            grids: self.grids.iter().map(|(k, v)| (*k, v.clone())).collect(),
            maps: self.maps.iter().map(|(k, v)| (*k, v.clone())).collect(),
            world_uis: self.world_uis.iter().map(|(k, v)| (*k, v.clone())).collect(),
            model_3ds: self.model_3ds.iter().map(|(k, v)| (*k, v.clone())).collect(),
        };

        serde_json::to_string_pretty(&data)
    }

    pub fn load_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        #[derive(Deserialize)]
        struct SceneData {
            #[serde(default)]
            next_entity: CustomEntity,
            #[serde(default)]
            transforms: Vec<(CustomEntity, Transform)>,
            #[serde(default)]
            velocities: Vec<(CustomEntity, (f32, f32))>,
            #[serde(default)]
            sprites: Vec<(CustomEntity, Sprite)>,
            #[serde(default)]
            colliders: Vec<(CustomEntity, Collider)>,
            #[serde(default)]
            colliders_3d: Vec<(CustomEntity, Collider3D)>,
            #[serde(default)]
            rigidbodies: Vec<(CustomEntity, Rigidbody2D)>,  // Added rigidbody deserialization
            #[serde(default)]
            cameras: Vec<(CustomEntity, Camera)>,
            #[serde(default)]
            meshes: Vec<(CustomEntity, Mesh)>,
            #[serde(default)]
            tags: Vec<(CustomEntity, EntityTag)>,
            #[serde(default)]
            scripts: Vec<(CustomEntity, Script)>,
            #[serde(default)]
            active: Vec<(CustomEntity, bool)>,
            #[serde(default)]
            layers: Vec<(CustomEntity, u8)>,
            #[serde(default)]
            parents: Vec<(CustomEntity, CustomEntity)>,
            #[serde(default)]
            names: Vec<(CustomEntity, String)>,
            #[serde(default)]
            sprite_sheets: Vec<(CustomEntity, SpriteSheet)>,
            #[serde(default)]
            animated_sprites: Vec<(CustomEntity, AnimatedSprite)>,
            #[serde(default)]
            tilemaps: Vec<(CustomEntity, Tilemap)>,
            #[serde(default)]
            tilesets: Vec<(CustomEntity, TileSet)>,
            #[serde(default)]
            tilemap_renderers: Vec<(CustomEntity, TilemapRenderer)>,
            #[serde(default)]
            grids: Vec<(CustomEntity, Grid)>,
            #[serde(default)]
            maps: Vec<(CustomEntity, Map)>,
            #[serde(default)]
            world_uis: Vec<(CustomEntity, WorldUI)>,
            #[serde(default)]
            model_3ds: Vec<(CustomEntity, Model3D)>,
        }

        let data: SceneData = serde_json::from_str(json)?;

        self.clear();
        
        // Set next_entity, or calculate from existing entities if not provided
        if data.next_entity > 0 {
            self.next_entity = data.next_entity;
        } else {
            // Calculate next_entity from max entity ID + 1
            let max_entity = data.transforms.iter()
                .map(|(e, _)| *e)
                .max()
                .unwrap_or(0);
            self.next_entity = max_entity + 1;
        }

        for (entity, transform) in data.transforms {
            self.transforms.insert(entity, transform);
        }
        for (entity, velocity) in data.velocities {
            self.velocities.insert(entity, velocity);
        }
        for (entity, sprite) in data.sprites {
            self.sprites.insert(entity, sprite);
        }
        for (entity, collider) in data.colliders {
            self.colliders.insert(entity, collider);
        }
        for (entity, collider) in data.colliders_3d {
            self.colliders_3d.insert(entity, collider);
        }
        for (entity, rigidbody) in data.rigidbodies {
            self.rigidbodies.insert(entity, rigidbody);
        }
        for (entity, camera) in data.cameras {
            self.cameras.insert(entity, camera);
        }
        for (entity, mesh) in data.meshes {
            self.meshes.insert(entity, mesh);
        }
        for (entity, name) in data.names {
            self.names.insert(entity, name);
        }
        for (entity, tag) in data.tags {
            self.tags.insert(entity, tag);
        }
        for (entity, script) in data.scripts {
            self.scripts.insert(entity, script);
        }
        for (entity, is_active) in data.active {
            self.active.insert(entity, is_active);
        }
        for (entity, layer) in data.layers {
            self.layers.insert(entity, layer);
        }
        for (entity, sprite_sheet) in data.sprite_sheets {
            self.sprite_sheets.insert(entity, sprite_sheet);
        }
        for (entity, animated_sprite) in data.animated_sprites {
            self.animated_sprites.insert(entity, animated_sprite);
        }
        for (entity, tilemap) in data.tilemaps {
            self.tilemaps.insert(entity, tilemap);
        }
        for (entity, tileset) in data.tilesets {
            self.tilesets.insert(entity, tileset);
        }
        for (entity, tilemap_renderer) in data.tilemap_renderers {
            self.tilemap_renderers.insert(entity, tilemap_renderer);
        }
        for (entity, grid) in data.grids {
            self.grids.insert(entity, grid);
        }
        for (entity, map) in data.maps {
            self.maps.insert(entity, map);
        }
        for (entity, world_ui) in data.world_uis {
            self.world_uis.insert(entity, world_ui);
        }
        for (entity, model_3d) in data.model_3ds {
            self.model_3ds.insert(entity, model_3d);
        }
        
        // Reconstruct hierarchy
        for (child, parent) in data.parents {
            self.parents.insert(child, parent);
            self.children.entry(parent).or_default().push(child);
        }

        // Ensure all entities have active and layer (backward compatibility)
        for &entity in self.transforms.keys() {
            self.active.entry(entity).or_insert(true);
            self.layers.entry(entity).or_insert(0);
        }

        Ok(())
    }
}

// ============================================================================
// Trait Implementations for World
// ============================================================================

use traits::{EcsWorld, EcsError, Serializable};

impl EcsWorld for CustomWorld {
    type Entity = CustomEntity;
    type Error = EcsError;
    
    fn spawn(&mut self) -> Self::Entity {
        CustomWorld::spawn(self)
    }
    
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
        // Check if entity exists
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }
        
        CustomWorld::despawn(self, entity);
        Ok(())
    }
    
    fn is_alive(&self, entity: Self::Entity) -> bool {
        // An entity is alive if it has at least one component or is in the active map
        self.transforms.contains_key(&entity) ||
        self.sprites.contains_key(&entity) ||
        self.colliders.contains_key(&entity) ||
        self.meshes.contains_key(&entity) ||
        self.cameras.contains_key(&entity) ||
        self.active.contains_key(&entity)
    }
    
    fn clear(&mut self) {
        CustomWorld::clear(self);
    }
    
    fn entity_count(&self) -> usize {
        self.active.len()
    }
    
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        // Check for circular reference if setting a parent
        if let Some(p) = parent {
            // Check if parent is actually a descendant of child
            let mut current = Some(p);
            while let Some(ancestor) = current {
                if ancestor == child {
                    return Err(EcsError::InvalidHierarchy);
                }
                current = self.get_parent(ancestor);
            }
        }
        
        CustomWorld::set_parent(self, child, parent);
        Ok(())
    }
    
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        CustomWorld::get_parent(self, entity)
    }
    
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        CustomWorld::get_children(self, entity).to_vec()
    }
}

impl Serializable for CustomWorld {
    fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        CustomWorld::save_to_json(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        CustomWorld::load_from_json(self, json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

// Implement ComponentAccess for all component types using the macro
mod custom_world_impls {
    use super::*;
    impl_component_access!(CustomWorld, Transform, transforms, CustomEntity);
    impl_component_access!(CustomWorld, GlobalTransform, global_transforms, CustomEntity);
    impl_component_access!(CustomWorld, Sprite, sprites, CustomEntity);
    impl_component_access!(CustomWorld, Collider, colliders, CustomEntity);
    impl_component_access!(CustomWorld, Rigidbody2D, rigidbodies, CustomEntity);
    impl_component_access!(CustomWorld, Mesh, meshes, CustomEntity);
    impl_component_access!(CustomWorld, Camera, cameras, CustomEntity);
    impl_component_access!(CustomWorld, Script, scripts, CustomEntity);
    impl_component_access!(CustomWorld, EntityTag, tags, CustomEntity);
    impl_component_access!(CustomWorld, SpriteSheet, sprite_sheets, CustomEntity);
    impl_component_access!(CustomWorld, AnimatedSprite, animated_sprites, CustomEntity);
    impl_component_access!(CustomWorld, Tilemap, tilemaps, CustomEntity);
    impl_component_access!(CustomWorld, TileSet, tilesets, CustomEntity);
    impl_component_access!(CustomWorld, TilemapRenderer, tilemap_renderers, CustomEntity);
    impl_component_access!(CustomWorld, Map, maps, CustomEntity);
    impl_component_access!(CustomWorld, Grid, grids, CustomEntity);
    impl_component_access!(CustomWorld, WorldUI, world_uis, CustomEntity);
    impl_component_access!(CustomWorld, LdtkMap, ldtk_maps, CustomEntity);
    impl_component_access!(CustomWorld, TilemapCollider, tilemap_colliders, CustomEntity);
    impl_component_access!(CustomWorld, LdtkIntGridCollider, ldtk_intgrid_colliders, CustomEntity);
    impl_component_access!(CustomWorld, Model3D, model_3ds, CustomEntity);
}

// Manual implementations for tuple and primitive types
// Manual implementations for tuple and primitive types

mod custom_world_manual_impls {
    use super::*;

    impl traits::ComponentAccess<(f32, f32)> for CustomWorld {
        type Entity = CustomEntity;
        type Error = EcsError;
        
        type ReadGuard<'a> = &'a (f32, f32);
        type WriteGuard<'a> = &'a mut (f32, f32);

        fn insert(&mut self, entity: Self::Entity, component: (f32, f32)) 
            -> Result<Option<(f32, f32)>, Self::Error> 
        {
            Ok(self.velocities.insert(entity, component))
        }
        
        fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
            self.velocities.get(&entity)
        }
        
        fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
            self.velocities.get_mut(&entity)
        }
        
        fn remove(&mut self, entity: Self::Entity) 
            -> Result<Option<(f32, f32)>, Self::Error> 
        {
            Ok(self.velocities.remove(&entity))
        }
        
        fn has(&self, entity: Self::Entity) -> bool {
            self.velocities.contains_key(&entity)
        }
    }

    impl traits::ComponentAccess<bool> for CustomWorld {
        type Entity = CustomEntity;
        type Error = EcsError;
        
        type ReadGuard<'a> = &'a bool;
        type WriteGuard<'a> = &'a mut bool;

        fn insert(&mut self, entity: Self::Entity, component: bool) 
            -> Result<Option<bool>, Self::Error> 
        {
            Ok(self.active.insert(entity, component))
        }
        
        fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
            self.active.get(&entity)
        }
        
        fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
            self.active.get_mut(&entity)
        }
        
        fn remove(&mut self, entity: Self::Entity) 
            -> Result<Option<bool>, Self::Error> 
        {
            Ok(self.active.remove(&entity))
        }
        
        fn has(&self, entity: Self::Entity) -> bool {
            self.active.contains_key(&entity)
        }
    }

    impl traits::ComponentAccess<u8> for CustomWorld {
        type Entity = CustomEntity;
        type Error = EcsError;
        
        type ReadGuard<'a> = &'a u8;
        type WriteGuard<'a> = &'a mut u8;

        fn insert(&mut self, entity: Self::Entity, component: u8) 
            -> Result<Option<u8>, Self::Error> 
        {
            Ok(self.layers.insert(entity, component))
        }
        
        fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
            self.layers.get(&entity)
        }
        
        fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
            self.layers.get_mut(&entity)
        }
        
        fn remove(&mut self, entity: Self::Entity) 
            -> Result<Option<u8>, Self::Error> 
        {
            Ok(self.layers.remove(&entity))
        }
        
        fn has(&self, entity: Self::Entity) -> bool {
            self.layers.contains_key(&entity)
        }
    }

    impl traits::ComponentAccess<String> for CustomWorld {
        type Entity = CustomEntity;
        type Error = EcsError;

        type ReadGuard<'a> = &'a String;
        type WriteGuard<'a> = &'a mut String;

        fn insert(&mut self, entity: Self::Entity, component: String) 
            -> Result<Option<String>, Self::Error> 
        {
            Ok(self.names.insert(entity, component))
        }
        
        fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
            self.names.get(&entity)
        }
        
        fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
            self.names.get_mut(&entity)
        }
        
        fn remove(&mut self, entity: Self::Entity) 
            -> Result<Option<String>, Self::Error> 
        {
            Ok(self.names.remove(&entity))
        }
        
        fn has(&self, entity: Self::Entity) -> bool {
            self.names.contains_key(&entity)
        }
    }
}


// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use traits::EcsWorld;
    
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        
        // **Feature: ecs-abstraction, Property 2: Entity spawn increases count**
        // **Validates: Requirements 1.1**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            
            #[test]
            fn entity_spawn_increases_count(spawn_count in 1usize..100) {
                let mut world = World::new();
                let initial_count = world.entity_count();
                
                // Spawn entities
                for _ in 0..spawn_count {
                    world.spawn();
                }
                
                let final_count = world.entity_count();
                
                // Property: spawning N entities should increase count by exactly N
                prop_assert_eq!(final_count, initial_count + spawn_count);
            }
        }
        
        // **Feature: ecs-abstraction, Property 3: Entity despawn decreases count**
        // **Validates: Requirements 1.1**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            
            #[test]
            fn entity_despawn_decreases_count(
                spawn_count in 1usize..50,
                despawn_indices in prop::collection::vec(any::<prop::sample::Index>(), 1..20)
            ) {
                let mut world = World::new();
                
                // Spawn entities
                let mut entities = Vec::new();
                for _ in 0..spawn_count {
                    entities.push(world.spawn());
                }
                
                let count_after_spawn = world.entity_count();
                prop_assert_eq!(count_after_spawn, spawn_count);
                
                // Despawn some entities (without duplicates)
                let mut despawned = std::collections::HashSet::new();
                for index in despawn_indices {
                    let entity = entities[index.index(entities.len())];
                    if despawned.insert(entity) {
                        let _ = world.despawn(entity);
                    }
                }
                
                let final_count = world.entity_count();
                let expected_count = spawn_count - despawned.len();
                
                // Property: despawning entities should decrease count appropriately
                // Note: This accounts for children being despawned recursively
                prop_assert!(final_count <= expected_count, 
                    "Expected count <= {}, got {}", expected_count, final_count);
            }
        }
    }
}

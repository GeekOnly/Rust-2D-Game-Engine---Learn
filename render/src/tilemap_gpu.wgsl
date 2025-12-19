// Vertex shader inputs
struct TilemapUniform {
    transform: mat4x4<f32>,
    map_size: vec2<u32>,   // Size of map in tiles (width, height)
    tile_size: vec2<f32>,  // Size of one tile in pixels (width, height)
    padding: vec2<u32>,
}

struct TilesetUniform {
    sheet_size: vec2<u32>, // Size of tileset sheet in pixels (unused if using array, but good for ref)
    tile_count: u32,       // Number of tiles
    padding: u32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

// Bind Groups
@group(0) @binding(0) var<uniform> camera: CameraUniform; // Global camera

@group(1) @binding(0) var<uniform> tilemap: TilemapUniform;
@group(1) @binding(1) var tilemap_indices: texture_2d<u32>; // R8Uint or R32Uint containing tile IDs

@group(2) @binding(0) var tilemap_data: texture_2d_array<f32>; // Array of tile textures
@group(2) @binding(1) var tilemap_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a quad (0,0) to (1,1)
    // 0: 0,0
    // 1: 0,1
    // 2: 1,0
    // 3: 1,0
    // 4: 0,1
    // 5: 1,1
    
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), // BL - In WGSL UV 0,0 is Top-Left usually? No, standard varies. Let's assume standard UV.
        vec2<f32>(0.0, 0.0), // TL
        vec2<f32>(1.0, 1.0), // BR
        vec2<f32>(1.0, 1.0), // BR
        vec2<f32>(0.0, 0.0), // TL
        vec2<f32>(1.0, 0.0)  // TR
    );
    
    var pos = array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0)
    );

    let uv = uvs[in_vertex_index];
    let raw_pos = pos[in_vertex_index];

    var out: VertexOutput;
    // Map bounds: width * tile_width, height * tile_height
    // Transform scales this 0..1 quad to world size
    // We assume 'tilemap.transform' handles Position, Rotation, and Scale of the entire map object.
    // If the map is 100x100 tiles, and each tile is 16px, the scale should be (1600, 1600, 1).
    
    let world_position = tilemap.transform * vec4<f32>(raw_pos, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.uv = uv; // 0..1 across the whole map
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate tile coordinate
    // uv is 0..1. Map size in tiles is tilemap.map_size (e.g., 20, 15)
    
    let map_coord = in.uv * vec2<f32>(f32(tilemap.map_size.x), f32(tilemap.map_size.y));
    let tile_xy = vec2<u32>(floor(map_coord)); // Integer tile coordinate (e.g., 3, 5)
    
    // 2. Fetch tile ID from index texture
    // textureLoad uses integer coordinates
    // We need to ensure tile_xy is within bounds, though UV logic implies it is.
    
    let tile_id = textureLoad(tilemap_indices, tile_xy, 0).r;
    
    // 3. If tile_id is 0 (or empty sentinel), discard? 
    // Usually 0 is air. Assuming 0 is air for now or need a specific "air" ID. 
    // LDTK usually makes specific intgrids. Let's assume 0 is a valid tile index if using 0-based.
    // Ideally we define 'empty' as a high number or use alpha.
    // For now, let's just sample. 
    
    // 4. Calculate UV within the specific tile
    let tile_uv = fract(map_coord); // 0..1 inside the single tile
    
    // 5. Sample from Texture Array
    // textureSample(t, s, uv, array_index)
    // Note: textureSample requires float UVs.
    // Using texture_2d_array.
    
    let color = textureSample(tilemap_data, tilemap_sampler, tile_uv, tile_id);
    
    if (color.a < 0.01) {
        discard;
    }
    
    return color;
}

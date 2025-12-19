// Unified Tilemap Shader for 2D/3D Rendering with Perfect Pixel Support
// This shader integrates with the unified rendering system to support both 2D and 3D tilemap rendering

// Unified Camera from the unified rendering system
struct UnifiedCameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    view_mode: f32,           // 0.0 = 2D, 1.0 = 3D
    perfect_pixel: vec4<f32>, // pixels_per_unit, snap_threshold, enabled, padding
    viewport: vec4<f32>,      // width, height, scale_factor, padding
}

// Unified Tilemap Uniform
struct UnifiedTilemapUniform {
    transform: mat4x4<f32>,
    map_size: vec2<u32>,      // Size of map in tiles (width, height)
    tile_size: vec2<f32>,     // Size of one tile in pixels (width, height)
    layer_depth: f32,         // Depth layer for sorting
    world_space_scale: f32,   // Scale factor for world space rendering
    pixels_per_unit: f32,     // Perfect pixel settings
    view_mode: f32,           // 0.0 = 2D, 1.0 = 3D (can override camera)
}

// Bind Groups
@group(0) @binding(0) var<uniform> camera: UnifiedCameraUniform;

@group(1) @binding(0) var<uniform> tilemap: UnifiedTilemapUniform;
@group(1) @binding(1) var tilemap_indices: texture_2d<u32>; // R32Uint containing tile IDs

@group(2) @binding(0) var tilemap_data: texture_2d_array<f32>; // Array of tile textures
@group(2) @binding(1) var tilemap_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) view_mode: f32,
    @location(3) layer_depth: f32,
}

// Perfect pixel snapping function
fn snap_to_pixel(position: vec3<f32>, pixels_per_unit: f32) -> vec3<f32> {
    if (pixels_per_unit <= 0.0) {
        return position;
    }
    
    let pixel_size = 1.0 / pixels_per_unit;
    return vec3<f32>(
        round(position.x / pixel_size) * pixel_size,
        round(position.y / pixel_size) * pixel_size,
        position.z // Don't snap Z in 3D
    );
}

@vertex
fn vs_tilemap_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a quad (0,0) to (1,1)
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), // BL
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
    
    // Transform to world space
    var world_position = tilemap.transform * vec4<f32>(raw_pos, 1.0);
    
    // Determine effective view mode (tilemap can override camera)
    let effective_view_mode = max(camera.view_mode, tilemap.view_mode);
    
    // Apply perfect pixel snapping in 2D mode
    if (effective_view_mode < 0.5 && camera.perfect_pixel.z > 0.5) {
        let pixels_per_unit = select(camera.perfect_pixel.x, tilemap.pixels_per_unit, tilemap.pixels_per_unit > 0.0);
        world_position = vec4<f32>(snap_to_pixel(world_position.xyz, pixels_per_unit), world_position.w);
    }
    
    // Apply layer depth for proper sorting
    world_position.z += tilemap.layer_depth;
    
    // Transform to clip space
    out.clip_position = camera.view_proj * world_position;
    out.uv = uv; // 0..1 across the whole map
    out.world_position = world_position.xyz;
    out.view_mode = effective_view_mode;
    out.layer_depth = tilemap.layer_depth;
    
    return out;
}

@fragment
fn fs_tilemap_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate tile coordinate
    // uv is 0..1. Map size in tiles is tilemap.map_size
    let map_coord = in.uv * vec2<f32>(f32(tilemap.map_size.x), f32(tilemap.map_size.y));
    let tile_xy = vec2<u32>(floor(map_coord)); // Integer tile coordinate
    
    // Bounds check
    if (tile_xy.x >= tilemap.map_size.x || tile_xy.y >= tilemap.map_size.y) {
        discard;
    }
    
    // 2. Fetch tile ID from index texture
    let tile_id = textureLoad(tilemap_indices, tile_xy, 0).r;
    
    // Skip empty tiles (tile_id 0 is typically empty/air)
    if (tile_id == 0u) {
        discard;
    }
    
    // 3. Calculate UV within the specific tile
    let tile_uv = fract(map_coord); // 0..1 inside the single tile
    
    // 4. Sample from Texture Array
    let color = textureSample(tilemap_data, tilemap_sampler, tile_uv, tile_id);
    
    // Alpha test
    if (color.a < 0.01) {
        discard;
    }
    
    // 5. Apply view mode specific processing
    if (in.view_mode < 0.5) {
        // 2D mode - simple texture sampling
        return color;
    } else {
        // 3D mode - basic lighting for world-space geometry
        // Simple directional lighting
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let normal = vec3<f32>(0.0, 0.0, 1.0); // Tilemap faces forward
        let light_factor = max(dot(normal, light_dir), 0.3); // Ambient + diffuse
        
        return vec4<f32>(color.rgb * light_factor, color.a);
    }
}

// Optimized 2D vertex shader for perfect pixel rendering
@vertex
fn vs_tilemap_2d(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0)
    );
    
    var pos = array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0)
    );

    let uv = uvs[in_vertex_index];
    let raw_pos = pos[in_vertex_index];

    var out: VertexOutput;
    
    // Transform to world space
    var world_position = tilemap.transform * vec4<f32>(raw_pos, 1.0);
    
    // Always apply perfect pixel snapping for 2D
    if (camera.perfect_pixel.z > 0.5) {
        let pixels_per_unit = select(camera.perfect_pixel.x, tilemap.pixels_per_unit, tilemap.pixels_per_unit > 0.0);
        world_position = vec4<f32>(snap_to_pixel(world_position.xyz, pixels_per_unit), world_position.w);
    }
    
    // Apply layer depth
    world_position.z += tilemap.layer_depth;
    
    out.clip_position = camera.view_proj * world_position;
    out.uv = uv;
    out.world_position = world_position.xyz;
    out.view_mode = 0.0; // Force 2D mode
    out.layer_depth = tilemap.layer_depth;
    
    return out;
}

// Optimized 2D fragment shader
@fragment
fn fs_tilemap_2d(in: VertexOutput) -> @location(0) vec4<f32> {
    let map_coord = in.uv * vec2<f32>(f32(tilemap.map_size.x), f32(tilemap.map_size.y));
    let tile_xy = vec2<u32>(floor(map_coord));
    
    if (tile_xy.x >= tilemap.map_size.x || tile_xy.y >= tilemap.map_size.y) {
        discard;
    }
    
    let tile_id = textureLoad(tilemap_indices, tile_xy, 0).r;
    
    if (tile_id == 0u) {
        discard;
    }
    
    let tile_uv = fract(map_coord);
    let color = textureSample(tilemap_data, tilemap_sampler, tile_uv, tile_id);
    
    if (color.a < 0.01) {
        discard;
    }
    
    return color;
}

// Optimized 3D vertex shader for world-space geometry
@vertex
fn vs_tilemap_3d(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0)
    );
    
    var pos = array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0)
    );

    let uv = uvs[in_vertex_index];
    let raw_pos = pos[in_vertex_index];

    var out: VertexOutput;
    
    // Transform to world space (no pixel snapping in 3D)
    let world_position = tilemap.transform * vec4<f32>(raw_pos, 1.0);
    
    // Apply layer depth
    let final_position = vec3<f32>(world_position.x, world_position.y, world_position.z + tilemap.layer_depth);
    
    out.clip_position = camera.view_proj * vec4<f32>(final_position, 1.0);
    out.uv = uv;
    out.world_position = final_position;
    out.view_mode = 1.0; // Force 3D mode
    out.layer_depth = tilemap.layer_depth;
    
    return out;
}

// Optimized 3D fragment shader with lighting
@fragment
fn fs_tilemap_3d(in: VertexOutput) -> @location(0) vec4<f32> {
    let map_coord = in.uv * vec2<f32>(f32(tilemap.map_size.x), f32(tilemap.map_size.y));
    let tile_xy = vec2<u32>(floor(map_coord));
    
    if (tile_xy.x >= tilemap.map_size.x || tile_xy.y >= tilemap.map_size.y) {
        discard;
    }
    
    let tile_id = textureLoad(tilemap_indices, tile_xy, 0).r;
    
    if (tile_id == 0u) {
        discard;
    }
    
    let tile_uv = fract(map_coord);
    let color = textureSample(tilemap_data, tilemap_sampler, tile_uv, tile_id);
    
    if (color.a < 0.01) {
        discard;
    }
    
    // Apply 3D lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = vec3<f32>(0.0, 0.0, 1.0); // Tilemap faces forward
    let light_factor = max(dot(normal, light_dir), 0.3);
    
    return vec4<f32>(color.rgb * light_factor, color.a);
}
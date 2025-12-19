// Unified 2D/3D Shader for WGPU
// This shader can handle both 2D sprites/tilemaps and 3D meshes based on the view mode

struct UnifiedCameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    view_mode: f32,           // 0.0 = 2D, 1.0 = 3D
    perfect_pixel: vec4<f32>, // pixels_per_unit, snap_threshold, enabled, padding
    viewport: vec4<f32>,      // width, height, scale_factor, padding
}

@group(0) @binding(0)
var<uniform> camera: UnifiedCameraUniform;

@group(1) @binding(0)
var texture_diffuse: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) view_mode: f32,
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
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    var world_position = input.position;
    
    // Apply perfect pixel snapping in 2D mode
    if (camera.view_mode < 0.5 && camera.perfect_pixel.z > 0.5) {
        world_position = snap_to_pixel(world_position, camera.perfect_pixel.x);
    }
    
    // Transform to clip space
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    
    // Pass through other attributes
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    out.world_position = world_position;
    out.normal = input.normal;
    out.view_mode = camera.view_mode;
    
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    var texture_color = textureSample(texture_diffuse, texture_sampler, input.tex_coords);
    
    // Apply vertex color
    var final_color = texture_color * input.color;
    
    // Different shading based on view mode
    if (input.view_mode < 0.5) {
        // 2D mode - simple texture sampling with vertex color
        return final_color;
    } else {
        // 3D mode - basic lighting (can be extended for full PBR)
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let normal = normalize(input.normal);
        let light_factor = max(dot(normal, light_dir), 0.2); // Ambient + diffuse
        
        return vec4<f32>(final_color.rgb * light_factor, final_color.a);
    }
}

// Vertex shader for 2D sprites (optimized path)
@vertex
fn vs_sprite_2d(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    var world_position = input.position;
    
    // Always apply perfect pixel snapping for 2D sprites
    if (camera.perfect_pixel.z > 0.5) {
        world_position = snap_to_pixel(world_position, camera.perfect_pixel.x);
    }
    
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    out.world_position = world_position;
    out.normal = vec3<f32>(0.0, 0.0, 1.0); // Default 2D normal
    out.view_mode = 0.0; // Force 2D mode
    
    return out;
}

// Fragment shader for 2D sprites (optimized path)
@fragment
fn fs_sprite_2d(input: VertexOutput) -> @location(0) vec4<f32> {
    var texture_color = textureSample(texture_diffuse, texture_sampler, input.tex_coords);
    return texture_color * input.color;
}

// Vertex shader for 3D meshes (optimized path)
@vertex
fn vs_mesh_3d(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // No pixel snapping in 3D mode
    out.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    out.world_position = input.position;
    out.normal = input.normal;
    out.view_mode = 1.0; // Force 3D mode
    
    return out;
}

// Fragment shader for 3D meshes (optimized path)
@fragment
fn fs_mesh_3d(input: VertexOutput) -> @location(0) vec4<f32> {
    var texture_color = textureSample(texture_diffuse, texture_sampler, input.tex_coords);
    var final_color = texture_color * input.color;
    
    // Basic lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(input.normal);
    let light_factor = max(dot(normal, light_dir), 0.2);
    
    return vec4<f32>(final_color.rgb * light_factor, final_color.a);
}
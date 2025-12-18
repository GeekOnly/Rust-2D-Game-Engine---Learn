// Infinite Grid Shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // 6 vertices for a quad
    // We render a large quad centered effectively around the camera on the XZ plane
    
    var pos = vec2<f32>(0.0);
    
    // Standard Quad Vertices [-1, 1]
    if (in_vertex_index == 0u) { pos = vec2<f32>(-1.0, -1.0); }
    else if (in_vertex_index == 1u) { pos = vec2<f32>(1.0, -1.0); }
    else if (in_vertex_index == 2u) { pos = vec2<f32>(-1.0, 1.0); }
    else if (in_vertex_index == 3u) { pos = vec2<f32>(-1.0, 1.0); }
    else if (in_vertex_index == 4u) { pos = vec2<f32>(1.0, -1.0); }
    else { pos = vec2<f32>(1.0, 1.0); }
    
    // Scale significantly to cover visible area
    let scale = 2000.0; 
    
    // Snap center to grid to avoid "sliding" artifacts if we were doing scrolling texture, 
    // but here we calculate line pos from world pos so exact center doesn't matter much 
    // as long as it covers the view.
    // Center on camera XZ
    let center_x = camera.view_pos.x;
    let center_z = camera.view_pos.z;
    
    let world_x = center_x + pos.x * scale;
    let world_z = center_z + pos.y * scale;
    
    let world_pos = vec3<f32>(world_x, -0.01, world_z); // Slightly below 0 to avoid Z-fight with floor? Or just 0.
    
    var out: VertexOutput;
    out.world_position = world_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = in.world_position.xz;
    
    // Use derivatives to calculate grid lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let line = min(grid.x, grid.y);
    
    // Base Color
    let color = vec3<f32>(0.5, 0.5, 0.5); 
    
    // Axis lines (X and Z)
    // Identify if we are close to 0 on X or Z
    let axis_width = 0.05; // World units
    let dist_x = abs(in.world_position.z); // Distance from X axis (z=0 line)
    let dist_z = abs(in.world_position.x); // Distance from Z axis (x=0 line)
    
    var axis_alpha = 0.0;
    var final_color = color;
    
    // Check Z-Axis (Blue) - Line where X=0
    if (dist_z < axis_width) {
         final_color = vec3<f32>(0.2, 0.2, 1.0);
         axis_alpha = 1.0;
    }
    // Check X-Axis (Red) - Line where Z=0
    else if (dist_x < axis_width) {
         final_color = vec3<f32>(1.0, 0.2, 0.2);
         axis_alpha = 1.0;
    }

    var alpha = 1.0 - min(line, 1.0);
    
    // Make axis always visible if alpha is low?
    alpha = max(alpha, axis_alpha);

    // Fade distance
    let dist_cam = distance(in.world_position.xz, camera.view_pos.xz);
    let fade_start = 50.0;
    let fade_end = 100.0;
    let fade = 1.0 - clamp((dist_cam - fade_start) / (fade_end - fade_start), 0.0, 1.0);
    
    alpha = alpha * fade;
    
    if (alpha <= 0.05) {
        discard;
    }
    
    return vec4<f32>(final_color, alpha * 0.5); // 0.5 base opacity
}

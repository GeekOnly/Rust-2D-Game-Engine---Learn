// UI Shader for rendering UI elements

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert screen space coordinates to clip space
    // Assuming input position is in pixels, we need to normalize to [-1, 1]
    // This should be done on the CPU side, but for now we'll pass through
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    
    return output;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, input.uv);
    
    // Multiply by vertex color (for tinting and alpha)
    return tex_color * input.color;
}

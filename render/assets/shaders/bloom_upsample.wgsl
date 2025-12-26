// 🌸 Bloom Upsample Shader
// Tent filter for smooth, high-quality upsampling
// Reference: "Next Generation Post Processing in Call of Duty: Advanced Warfare"

// Vertex shader (fullscreen triangle)
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

// Bind Group 0: Lower resolution mip (source)
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

// Bind Group 1: Higher resolution mip (to add to)
@group(1) @binding(0) var t_add: texture_2d<f32>;
@group(1) @binding(1) var s_add: sampler;

// 9-tap tent filter upsample
// Weighted sampling in a 3x3 pattern
//   1  2  1
//   2  4  2
//   1  2  1
// Total weight: 16
@fragment
fn fs_upsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_source));

    // Center (weight: 4)
    var color = textureSample(t_source, s_source, in.uv).rgb * 4.0;

    // Cardinal directions (weight: 2 each)
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0, -1.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0,  1.0) * texel_size).rgb * 2.0;

    // Diagonal directions (weight: 1 each)
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  1.0) * texel_size).rgb;

    color /= 16.0; // Normalize

    // Add the higher-resolution mip
    let add_color = textureSample(t_add, s_add, in.uv).rgb;
    color += add_color;

    return vec4<f32>(color, 1.0);
}

// First upsample (no addition)
@fragment
fn fs_upsample_first(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_source));

    var color = textureSample(t_source, s_source, in.uv).rgb * 4.0;

    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0, -1.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0,  1.0) * texel_size).rgb * 2.0;

    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  1.0) * texel_size).rgb;

    color /= 16.0;

    return vec4<f32>(color, 1.0);
}

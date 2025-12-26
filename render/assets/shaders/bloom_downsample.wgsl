// 🌸 Bloom Downsample Shader
// Efficient mobile bloom using dual-filter approach
// Reference: "Bandwidth-Efficient Rendering" by ARM

// Vertex shader (fullscreen triangle)
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle (no vertex buffer needed)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

// Bind Group 0: Source texture
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

// Bind Group 1: Bloom settings
struct BloomUniforms {
    threshold: f32,          // Luminance threshold (default: 1.0)
    soft_threshold: f32,     // Soft knee range (default: 0.5)
    intensity: f32,          // Overall bloom intensity
    _padding: f32,
}

@group(1) @binding(0) var<uniform> u_bloom: BloomUniforms;

// Luminance calculation (Rec. 709)
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Soft threshold filter (smooth falloff)
fn apply_threshold(color: vec3<f32>) -> vec3<f32> {
    let luma = luminance(color);
    let knee = u_bloom.threshold * u_bloom.soft_threshold;

    // Soft knee curve
    var soft = luma - u_bloom.threshold + knee;
    soft = clamp(soft, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee + 0.00001);

    let contribution = max(soft, luma - u_bloom.threshold) / max(luma, 0.00001);
    return color * contribution;
}

// 13-tap downsample (hardware bilinear + 4x4 box filter)
// This gives us free blur while downsampling!
@fragment
fn fs_downsample_prefilter(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_source));

    // Sample 13 taps in a cross pattern
    // Center (1 tap)
    var color = textureSample(t_source, s_source, in.uv).rgb * 4.0;

    // Inner ring (4 taps)
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  1.0) * texel_size).rgb;

    // Outer ring (8 taps)
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-2.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 2.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0, -2.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0,  2.0) * texel_size).rgb * 2.0;

    color /= 16.0; // Normalize

    // Apply threshold (only on first pass)
    color = apply_threshold(color);

    return vec4<f32>(color, 1.0);
}

// Regular downsample (for mip chain, no threshold)
@fragment
fn fs_downsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_source));

    // 13-tap downsample
    var color = textureSample(t_source, s_source, in.uv).rgb * 4.0;

    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-1.0,  1.0) * texel_size).rgb;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 1.0,  1.0) * texel_size).rgb;

    color += textureSample(t_source, s_source, in.uv + vec2<f32>(-2.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 2.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0, -2.0) * texel_size).rgb * 2.0;
    color += textureSample(t_source, s_source, in.uv + vec2<f32>( 0.0,  2.0) * texel_size).rgb * 2.0;

    color /= 16.0;

    return vec4<f32>(color, 1.0);
}

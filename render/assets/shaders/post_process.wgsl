// 🎬 AAA Mobile Post-Processing: Uber-Shader
// Single-pass post-processing for mobile efficiency
// Combines: Tonemapping, Bloom, Color Grading, Vignette, Gamma

// Bind Group 0: HDR Scene Texture
@group(0) @binding(0) var t_hdr: texture_2d<f32>;
@group(0) @binding(1) var s_hdr: sampler;

// Bind Group 1: Bloom Texture (optional)
@group(1) @binding(0) var t_bloom: texture_2d<f32>;
@group(1) @binding(1) var s_bloom: sampler;

// Bind Group 2: Post-Process Uniforms
struct PostProcessUniforms {
    exposure: f32,
    bloom_intensity: f32,
    contrast: f32,
    saturation: f32,

    vignette_strength: f32,
    vignette_smoothness: f32,
    chromatic_aberration: f32,
    _padding: f32,
}

@group(2) @binding(0) var<uniform> u_post: PostProcessUniforms;

// Vertex Output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Fullscreen Triangle Vertex Shader (no vertex buffer needed)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate fullscreen triangle
    // Vertex 0: (-1, -1)
    // Vertex 1: ( 3, -1)
    // Vertex 2: (-1,  3)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

// ===== Tone Mapping =====

// ACES Filmic Tone Mapping (Narkowicz fit)
// Industry standard for converting HDR to SDR
fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((color * (a * color + b)) / (color * (c * color + d) + e));
}

// Reinhard Tone Mapping (simpler alternative)
fn reinhard_tone_map(color: vec3<f32>) -> vec3<f32> {
    return color / (vec3<f32>(1.0) + color);
}

// Uncharted 2 Tone Mapping
fn uncharted2_tonemap_partial(x: vec3<f32>) -> vec3<f32> {
    let A = 0.15;
    let B = 0.50;
    let C = 0.10;
    let D = 0.20;
    let E = 0.02;
    let F = 0.30;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

fn uncharted2_tone_map(color: vec3<f32>) -> vec3<f32> {
    let exposure_bias = 2.0;
    let curr = uncharted2_tonemap_partial(color * exposure_bias);
    let W = vec3<f32>(11.2);
    let white_scale = vec3<f32>(1.0) / uncharted2_tonemap_partial(W);
    return curr * white_scale;
}

// ===== Color Adjustments =====

// Adjust contrast
fn adjust_contrast(color: vec3<f32>, contrast: f32) -> vec3<f32> {
    return (color - 0.5) * contrast + 0.5;
}

// Adjust saturation
fn adjust_saturation(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luminance), color, saturation);
}

// ===== Gamma Correction =====

// Linear to sRGB
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    // Accurate sRGB conversion
    let cutoff = linear < vec3<f32>(0.0031308);
    let higher = vec3<f32>(1.055) * pow(linear, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let lower = linear * vec3<f32>(12.92);
    return select(higher, lower, cutoff);
}

// Simple gamma correction (faster)
fn gamma_correct(linear: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / gamma));
}

// ===== Effects =====

// Vignette effect
fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, strength: f32, smoothness: f32) -> vec3<f32> {
    let dist = distance(uv, vec2<f32>(0.5));
    let vignette = smoothstep(strength, strength - smoothness, dist);
    return color * vignette;
}

// Chromatic Aberration (simple version)
fn apply_chromatic_aberration(uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let offset_dir = (uv - 0.5) * strength;

    let r = textureSample(t_hdr, s_hdr, uv + offset_dir).r;
    let g = textureSample(t_hdr, s_hdr, uv).g;
    let b = textureSample(t_hdr, s_hdr, uv - offset_dir).b;

    return vec3<f32>(r, g, b);
}

// Film grain (simple noise)
fn film_grain(uv: vec2<f32>, intensity: f32) -> f32 {
    // Simple pseudo-random noise
    let noise = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    return (noise - 0.5) * intensity;
}

// ===== Main Fragment Shader =====

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // 1. Sample HDR color (with optional chromatic aberration)
    var color: vec3<f32>;
    if (u_post.chromatic_aberration > 0.0) {
        color = apply_chromatic_aberration(uv, u_post.chromatic_aberration * 0.01);
    } else {
        color = textureSample(t_hdr, s_hdr, uv).rgb;
    }

    // 2. Exposure adjustment
    color *= u_post.exposure;

    // 3. Add bloom
    let bloom = textureSample(t_bloom, s_bloom, uv).rgb;
    color += bloom * u_post.bloom_intensity;

    // 4. Color grading
    color = adjust_contrast(color, u_post.contrast);
    color = adjust_saturation(color, u_post.saturation);

    // 5. Vignette (before tonemapping for better look)
    color = apply_vignette(color, uv, u_post.vignette_strength, u_post.vignette_smoothness);

    // 6. Tone mapping (HDR to SDR)
    color = aces_tone_map(color);

    // 7. Gamma correction (linear to sRGB)
    color = linear_to_srgb(color);

    // 8. Film grain (optional, after tonemapping)
    // let grain = film_grain(uv, 0.05);
    // color += vec3<f32>(grain);

    return vec4<f32>(color, 1.0);
}

// Alternative simple version (for testing)
@fragment
fn fs_main_simple(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_hdr, s_hdr, in.uv).rgb;

    // Just tonemapping + gamma
    let tonemapped = aces_tone_map(color * u_post.exposure);
    let final_color = linear_to_srgb(tonemapped);

    return vec4<f32>(final_color, 1.0);
}

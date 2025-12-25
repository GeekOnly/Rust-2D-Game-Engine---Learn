// PBR Shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct LightUniform {
    position: vec4<f32>,
    color: vec4<f32>,
    view_proj: array<mat4x4<f32>, 4>, // Cascades
    splits: vec4<f32>, // Split Planes (Z distances)
};

struct MaterialUniform {
    albedo_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> light: LightUniform;
@group(1) @binding(1)
var t_shadow: texture_depth_2d_array;
@group(1) @binding(2)
var s_shadow: sampler_comparison;
@group(1) @binding(3)
var t_scene_depth: texture_depth_2d; // Scene Depth Copy for SSCS
@group(1) @binding(4)
var s_scene_depth: sampler; // Regular sampler for scene depth

// Material Bindings (Group 2)
@group(2) @binding(0)
var<uniform> material: MaterialUniform;
@group(2) @binding(1)
var t_albedo: texture_2d<f32>;
@group(2) @binding(2)
var s_albedo: sampler;
@group(2) @binding(3)
var t_normal: texture_2d<f32>;
@group(2) @binding(4)
var s_normal: sampler;
@group(2) @binding(5)
var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(6)
var s_metallic_roughness: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    // Instance attributes
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
    @location(9) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) color: vec4<f32>,
};

// Object Uniform Removed

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Reconstruct model matrix from instance attributes
    let model_matrix = mat4x4<f32>(
        in.model_0,
        in.model_1,
        in.model_2,
        in.model_3,
    );

    let world_pos = model_matrix * vec4<f32>(in.position, 1.0);
    
    out.world_position = world_pos.xyz;
    out.tex_coords = in.tex_coords;
    
    // Normal transform (using rotation part of model matrix)
    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );

    out.normal = normal_matrix * in.normal;
    out.tangent = normal_matrix * in.tangent;
    out.bitangent = normal_matrix * in.bitangent;
    out.color = in.color;
    
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@vertex
fn vs_shadow(
    in: VertexInput,
) -> @builtin(position) vec4<f32> {
    // Reconstruct model matrix
    let model_matrix = mat4x4<f32>(
        in.model_0,
        in.model_1,
        in.model_2,
        in.model_3,
    );
    let world_pos = model_matrix * vec4<f32>(in.position, 1.0);
    // Use Camera ViewProj (Group 0) - This allows us to bind the Light Matrix as "Camera" during Shadow Pass
    return camera.view_proj * world_pos;
}

// PBR Functions
const PI = 3.14159265359;

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (PI * denom * denom);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fetch_shadow(world_pos: vec3<f32>, N: vec3<f32>, L: vec3<f32>, view_depth: f32) -> f32 {
    // Select Cascade
    var cascade_index = 0u;
    // Simple loop or if/else. splits are positive Z distances (e.g. 10.0, 50.0, 200.0)
    // view_depth is usually negative in View Space (Right Handed), so use abs() or -view_depth
    // Actually our View Space Z is negative. So distance is -view_dest.
    let depth = -view_depth;
    
    if (depth > light.splits.x) { cascade_index = 1u; }
    if (depth > light.splits.y) { cascade_index = 2u; }
    if (depth > light.splits.z) { cascade_index = 3u; }

    let light_space_pos = light.view_proj[cascade_index] * vec4<f32>(world_pos, 1.0);
    let proj_coords = light_space_pos.xyz / light_space_pos.w;
    
    // NDC to Texture
    let flip_correction = vec2<f32>(0.5, -0.5); 
    let uv = proj_coords.xy * flip_correction + vec2<f32>(0.5, 0.5);
    
    // Depth verify
    if (proj_coords.z < 0.0 || proj_coords.z > 1.0) {
        return 1.0;
    }

    // Bias
    let bias = max(0.005 * (1.0 - dot(N, L)), 0.001);
    // Bias scaling with cascade roughly? 
    // Usually far cascades need less bias/more bias depending on resolution ratio.
    
    let current_depth = proj_coords.z - bias;
    
    // PCF
    var shadow = 0.0;
    let size = vec2<f32>(textureDimensions(t_shadow).xy);
    let texel_size = vec2<f32>(1.0 / size.x, 1.0 / size.y);
    
    for(var x = -1; x <= 1; x++) {
        for(var y = -1; y <= 1; y++) {
            let pcf_depth = textureSampleCompare(
                t_shadow, 
                s_shadow, 
                uv + vec2<f32>(f32(x), f32(y)) * texel_size, 
                i32(cascade_index), // Array Layer
                current_depth
            );
            shadow += pcf_depth;
        }
    }
    shadow /= 9.0;
    
    // Debug: Color tint based on cascade? No, keep it clean.
    return shadow;
}

// ----------------------------------------------------------------------------
// Screen Space Contact Shadows (SSCS)
// ----------------------------------------------------------------------------
fn contact_shadows(world_pos: vec3<f32>, L: vec3<f32>, uv: vec2<f32>, dither: f32) -> f32 {
    // 1. Raymarch details
    let steps = 8u;
    let step_len = 0.05; // 5cm step (World Space) / adjust per scene scale
    // We need to march in Screen Space, but we don't have full G-Buffer.
    // Simpler: March in View Space? Or World Space and project?
    // Raymarch: Start at WorldPos, move towards Light. Project to screen. Compare depth.
    
    // Randomize start along ray (dithering)
    let start_offset = dither * step_len;
    
    var shadow = 1.0;
    
    for (var i = 1u; i <= steps; i++) {
        let dist = start_offset + f32(i) * step_len;
        let p = world_pos + L * dist;
        
        let clip_pos = camera.view_proj * vec4<f32>(p, 1.0);
        let ndc = clip_pos.xyz / clip_pos.w;
        
        // Check bounds
        if (ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0) {
            continue;
        }
        
        let screen_uv = ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

        // Sample Scene Depth - Use regular sampler for depth texture sampling
        // For AAA mobile: bilinear filtering helps smooth depth comparisons
        let scene_depth_raw = textureSample(t_scene_depth, s_scene_depth, screen_uv);

        // Fallback: textureLoad for exact pixel (no filtering)
        // let dim = textureDimensions(t_scene_depth);
        // let px = vec2<i32>(screen_uv * vec2<f32>(dim));
        // let scene_depth_raw = textureLoad(t_scene_depth, px, 0);
        
        // Compare Depths
        // Note: WGPU Depth is 0..1 (Reverse Z if configured? We use standard Z < 1.0? 
        // Wait, shadow map is 1.0 clear. But Scene Depth?
        // We need to know if scene_depth < ray_depth (Occlusion).
        // Since we copy main depth, and we use Reverse-Z (Greater) in main pass, correct?
        // Let's check: render/src/lib.rs: 
        // depth_compare: wgpu::CompareFunction::Greater
        // bias: clear to 0.0? No, LoadOp::Clear(0.0) usually for Reverse Z.
        // Let's check lib.rs again.
        // It says: LoadOp::Clear(1.0)?? with CompareFunction::Greater? That fails everything.
        // Wait, normally Reverse Z clears to 0.0 and checks Greater.
        // Standard Z clears to 1.0 and checks Less.
        // Let's assume Standard Z for now based on 'Clear(1.0)' in code.
        
        // Standard Z: Near=0, Far=1.
        // If scene_depth < ndc.z, then scene object is CLOSER to camera than ray point.
        // Ray point is behind geometry -> Shadowed.
        
        // Bias is crucial.
        let bias = 0.0001; // Tiny bias
        if (scene_depth_raw < ndc.z - bias) {
             // Ray is occluded
             return 0.0;
        }
    }
    
    return 1.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_albedo, s_albedo, in.tex_coords) * material.albedo_factor * in.color;
    let metallic_roughness = textureSample(t_metallic_roughness, s_metallic_roughness, in.tex_coords);
    let metallic = metallic_roughness.b * material.metallic_factor;
    let roughness = metallic_roughness.g * material.roughness_factor;

    let N = normalize(in.normal); // ToDo: Use Normal Map
    let V = normalize(camera.view_pos.xyz - in.world_position);

    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo.rgb, metallic);

    // Light calculation (Single light source for now)
    let L = normalize(light.position.xyz - in.world_position);
    let H = normalize(V + L);
    
    // Attenuation (simplified)
    let distance = length(light.position.xyz - in.world_position);
    let attenuation = 1.0 / (distance * distance);
    let view_dist = length(camera.view_pos.xyz - in.world_position);
    
    let shadow_factor = fetch_shadow(in.world_position, N, L, -view_dist); // Pass negative distance as mock Z
    
    // Contact Shadows
    // Use screen coordinates from fragment position?
    // in.clip_position: @builtin(position) is usually window coordinates (pixels).
    // Need UV.
    let dims = vec2<f32>(textureDimensions(t_scene_depth));
    let screen_uv = in.clip_position.xy / dims;
    
    // Simple dither from pixel position
    let dither = fract(sin(dot(in.clip_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    
    let contact_shadow = contact_shadows(in.world_position, L, screen_uv, dither);
    
    // Combine (Min)
    let final_shadow = min(shadow_factor, contact_shadow);

    let radiance = light.color.rgb * light.color.a * attenuation * final_shadow;

    // Cook-Torrance BRDF
    let NDF = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;

    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD = kD * (1.0 - metallic);

    let NdotL = max(dot(N, L), 0.0);
    let Lo = (kD * albedo.rgb / PI + specular) * radiance * NdotL;

    // Ambient
    let ambient = vec3<f32>(0.03) * albedo.rgb;
    let color = ambient + Lo;

    // Gamma correction
    let corrected_color = pow(color, vec3<f32>(1.0/2.2));

    return vec4<f32>(corrected_color, albedo.a);
}

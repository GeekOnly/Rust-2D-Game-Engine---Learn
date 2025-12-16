// PBR Shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct LightUniform {
    position: vec4<f32>,
    color: vec4<f32>,
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
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // Assume model matrix is identity for now (we need instance support later)
    let world_pos = vec4<f32>(model.position, 1.0);
    
    out.world_position = world_pos.xyz;
    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    out.tangent = model.tangent;
    out.bitangent = model.bitangent;
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

// PBR Functions
let PI = 3.14159265359;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_albedo, s_albedo, in.tex_coords) * material.albedo_factor;
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
    let radiance = light.color.rgb * light.color.a * attenuation;

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

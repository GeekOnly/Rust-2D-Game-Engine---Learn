// Toon Shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct LightUniform {
    position: vec4<f32>,
    color: vec4<f32>,
};

struct ToonMaterialUniform {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    params: vec4<f32>, // x: outline_width
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> light: LightUniform;

@group(2) @binding(0)
var<uniform> material: ToonMaterialUniform;

struct ObjectUniform {
    model: mat4x4<f32>,
};

@group(3) @binding(0)
var<uniform> object: ObjectUniform;

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
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = object.model * vec4<f32>(model.position, 1.0);
    out.world_position = world_pos.xyz;
    // For now assume uniform scaling so allow normal transformation by model matrix rotation
    // Ideally use inverse transpose of model matrix
    let normal_matrix = mat3x3<f32>(
        object.model[0].xyz,
        object.model[1].xyz,
        object.model[2].xyz
    );
    out.normal = normal_matrix * model.normal;
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let L = normalize(light.position.xyz - in.world_position);
    
    // Simple Lambert
    let NdotL = max(dot(N, L), 0.0);
    
    // Toon Quantization (3 bands)
    // 0.0 - 0.33 -> Shadow
    // 0.33 - 0.66 -> Mid
    // 0.66 - 1.0 -> Bright
    
    var intensity = NdotL;
    if (intensity > 0.85) {
        intensity = 1.0;
    } else if (intensity > 0.5) {
        intensity = 0.7;
    } else if (intensity > 0.2) {
        intensity = 0.35;
    } else {
        intensity = 0.1;
    }
    
    let light_color = light.color.rgb * light.color.a;
    let final_color = material.color.rgb * light_color * intensity;
    
    return vec4<f32>(final_color, material.color.a);
}

// Outline Shader (Inverted Hull)

struct OutlineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_outline(
    model: VertexInput,
) -> OutlineVertexOutput {
    // Extrude vertex along normal
    let outline_width = material.params.x;
    let extruded_pos = model.position + model.normal * outline_width;
    
    var out: OutlineVertexOutput;
    let world_pos = object.model * vec4<f32>(extruded_pos, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_outline() -> @location(0) vec4<f32> {
    return material.outline_color;
}

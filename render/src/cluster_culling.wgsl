// Cluster Culling Compute Shader

struct GPULight {
    position: vec4<f32>,
    color: vec4<f32>,
    radius: f32,
    padding: array<f32, 3>,
};

struct Cluster {
    offset: u32,
    count: u32,
};

struct LightList {
    lights: array<GPULight>,
};

struct ClusterList {
    data: array<Cluster>,
};

struct GlobalIndexList {
    indices: array<u32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct ClusterUniform {
    inverse_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    screen_size: vec2<f32>,
    near_plane: f32,
    far_plane: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage, read> lights: LightList;
@group(0) @binding(2) var<storage, read_write> clusters: ClusterList;
@group(0) @binding(3) var<storage, read_write> global_light_indices: GlobalIndexList;
@group(0) @binding(4) var<uniform> cluster_uniform: ClusterUniform;

const TILE_SIZE: u32 = 16u;
const MAX_LIGHTS_PER_CLUSTER: u32 = 64u;
const CLUSTER_Z_SLICES: u32 = 24u;

fn test_sphere_aabb(center: vec3<f32>, radius: f32, min_aabb: vec3<f32>, max_aabb: vec3<f32>) -> bool {
    let closest = max(min_aabb, min(center, max_aabb));
    let dist = distance(closest, center);
    return dist <= radius;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tile_x = global_id.x;
    let tile_y = global_id.y;
    let tile_z = global_id.z;
    
    // Check bounds
    let grid_width = u32(ceil(cluster_uniform.screen_size.x / f32(TILE_SIZE)));
    let grid_height = u32(ceil(cluster_uniform.screen_size.y / f32(TILE_SIZE)));

    if (tile_x >= grid_width || tile_y >= grid_height || tile_z >= CLUSTER_Z_SLICES) {
        return;
    }

    let cluster_index = tile_x + tile_y * grid_width + tile_z * grid_width * grid_height;

    // 1. Calculate Cluster AABB in View Space
    // Slice 0 starts at Near.
    let z_near = cluster_uniform.near_plane;
    let z_far = cluster_uniform.far_plane;
    
    // Logarithmic Z distribution
    // Formula: z_depth = near * (far/near)^(slice/num_slices)
    // Note: View Space Z is negative. We compute positive distance then negate.
    
    let slice_step = f32(tile_z) / f32(CLUSTER_Z_SLICES);
    let next_slice_step = f32(tile_z + 1u) / f32(CLUSTER_Z_SLICES);
    
    // Safety check for z_near <= 0.0 (Orthographic might have near < 0 or near = 0)
    // Logarithmic requires near > 0. If perspective, near is usually > 0.
    // Use fallback if near <= 0.
    var min_z_dist = 0.0;
    var max_z_dist = 0.0;
    
    if (z_near <= 0.001) {
        // Fallback to Linear for Ortho or low precision
        min_z_dist = z_near + (z_far - z_near) * slice_step;
        max_z_dist = z_near + (z_far - z_near) * next_slice_step;
    } else {
        min_z_dist = z_near * pow(z_far / z_near, slice_step);
        max_z_dist = z_near * pow(z_far / z_near, next_slice_step);
    }

    let min_z = -max_z_dist; // Farther (View Z is negative)
    let max_z = -min_z_dist; // Closer

    // Compute NDC X/Y corners for this tile
    // WGPU NDC: X[-1, 1], Y[1, -1] ? No, Y is up or down?
    // WGPU (0,0) texture is Top-Left. 
    // Standard NDC: Bottom-Left is (-1, -1). Top-Right is (1, 1).
    // Mapping:
    // Screen X: 0 -> W maps to -1 -> 1.
    // Screen Y: 0 -> H maps to 1 -> -1. (Top is 1, Bottom is -1)
    
    let min_x_ndc = (f32(tile_x) * f32(TILE_SIZE) / cluster_uniform.screen_size.x) * 2.0 - 1.0;
    let max_x_ndc = (f32(tile_x + 1u) * f32(TILE_SIZE) / cluster_uniform.screen_size.x) * 2.0 - 1.0;
    
    // Top is Y=0 -> NDC 1. Bottom is Y=H -> NDC -1.
    // Tile Y=0 (Top) -> NDC Top Y.
    // Tile Y increases downwards.
    let top_y_ndc = 1.0 - 2.0 * (f32(tile_y) * f32(TILE_SIZE) / cluster_uniform.screen_size.y);
    let bottom_y_ndc = 1.0 - 2.0 * (f32(tile_y + 1u) * f32(TILE_SIZE) / cluster_uniform.screen_size.y);
    
    // Bounds initialized
    var min_aabb = vec3<f32>(100000.0);
    var max_aabb = vec3<f32>(-100000.0);
    
    let corners_x = vec2<f32>(min_x_ndc, max_x_ndc);
    let corners_y = vec2<f32>(bottom_y_ndc, top_y_ndc);
    let depths = vec2<f32>(min_z, max_z); 

    for(var cx = 0; cx < 2; cx++) {
        for(var cy = 0; cy < 2; cy++) {
            // Unproject corner at NDC Z=0 (arbitrary, just to get direction)
            // Or unproject at NDC Z=1 to get far? 
            // Better: We construct a point at NDC Z=0.
            let ndc_point = vec4<f32>(corners_x[cx], corners_y[cy], 0.0, 1.0);
            let unproj = cluster_uniform.inverse_proj * ndc_point;
            let view_dir = unproj.xyz / unproj.w; 
            
            // view_dir points from camera intented direction.
            // Scale it so that view_dir.z matches our target depths.
            
            for(var d = 0; d < 2; d++) {
                let target_z = depths[d]; // e.g. -10.0
                // factor * view_dir.z = target_z
                // factor = target_z / view_dir.z
                // Check div by zero
                if (abs(view_dir.z) > 0.0001) {
                    let factor = target_z / view_dir.z;
                    // If factor < 0, it means target z is behind camera? (If view_dir.z is same sign).
                    // View Z is negative (looking down -Z). Target Z is negative.
                    // view_dir.z should be negative. factor should be positive.
                    let point_view = view_dir * factor;
                    
                    min_aabb = min(min_aabb, point_view);
                    max_aabb = max(max_aabb, point_view);
                }
            }
        }
    }

    // 2. Cull lights
    var visible_light_count = 0u;
    var visible_light_indices: array<u32, MAX_LIGHTS_PER_CLUSTER>;

    let light_count = arrayLength(&lights.lights);

    for (var i = 0u; i < light_count && visible_light_count < MAX_LIGHTS_PER_CLUSTER; i++) {
        let light_world_pos = lights.lights[i].position.xyz;
        let light_radius = lights.lights[i].radius;

        // Transform light to View Space
        let light_view_pos = (cluster_uniform.view * vec4<f32>(light_world_pos, 1.0)).xyz;

        if (test_sphere_aabb(light_view_pos, light_radius, min_aabb, max_aabb)) {
            visible_light_indices[visible_light_count] = i;
            visible_light_count++;
        }
    }

    // 3. Write Result
    let base_offset = cluster_index * MAX_LIGHTS_PER_CLUSTER;
    clusters.data[cluster_index].offset = base_offset;
    clusters.data[cluster_index].count = visible_light_count;

    for (var i = 0u; i < visible_light_count; i++) {
        global_light_indices.indices[base_offset + i] = visible_light_indices[i];
    }
}

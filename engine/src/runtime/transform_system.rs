use ecs::World;
use glam::{Vec3, Quat, Mat4};

/// Updates global transforms for all entities based on hierarchy
pub fn update_global_transforms(world: &mut World) {
    // Borrows need to be split manually because we are modifying one field while reading others
    let transforms = &world.transforms;
    let parents = &world.parents;
    let children = &world.children;
    let global_transforms = &mut world.global_transforms;

    // Identify root entities (entities that have a Transform but NO parent)
    let mut roots = Vec::new();
    for (entity, _) in transforms.iter() {
        if !parents.contains_key(entity) {
            roots.push(*entity);
        }
    }

    // Process hierarchy
    for root in roots {
        propagate_recursive(root, Mat4::IDENTITY, transforms, children, global_transforms);
    }
}

fn propagate_recursive(
    entity: u32,
    parent_global_matrix: Mat4,
    transforms: &std::collections::HashMap<u32, ecs::Transform>,
    children_map: &std::collections::HashMap<u32, Vec<u32>>,
    global_transforms: &mut std::collections::HashMap<u32, ecs::GlobalTransform>,
) {
    let local_transform = match transforms.get(&entity) {
        Some(t) => t,
        None => return, // Should not happen if we iterate properly
    };
    
    // Calculate local matrix (Scale -> Rotate -> Translate)
    let rot_rad = Vec3::new(
        local_transform.rotation[0].to_radians(),
        local_transform.rotation[1].to_radians(),
        local_transform.rotation[2].to_radians(),
    );
    let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
    let translation = Vec3::from(local_transform.position);
    let scale = Vec3::from(local_transform.scale);
    
    // T * R * S
    let local_matrix = Mat4::from_scale_rotation_translation(scale, rotation, translation);
    
    // Calculate global matrix: ParentGlobal * Local
    let global_matrix = parent_global_matrix * local_matrix;
    
    // Store in World
    global_transforms.insert(entity, ecs::GlobalTransform { matrix: global_matrix.to_cols_array() });
    
    // Recurse to children
    if let Some(children) = children_map.get(&entity) {
        for child in children {
            propagate_recursive(*child, global_matrix, transforms, children_map, global_transforms);
        }
    }
}

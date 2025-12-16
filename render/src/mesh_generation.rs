use wgpu::util::DeviceExt;
use crate::{Mesh, ModelVertex};

/// Generate GPU mesh from ECS mesh type
pub fn generate_mesh(device: &wgpu::Device, mesh_type: &ecs::MeshType) -> Mesh {
    match mesh_type {
        ecs::MeshType::Cube => generate_cube_mesh(device),
        ecs::MeshType::Sphere => generate_sphere_mesh(device),
        ecs::MeshType::Cylinder => generate_cylinder_mesh(device),
        ecs::MeshType::Plane => generate_plane_mesh(device),
        ecs::MeshType::Capsule => generate_capsule_mesh(device),
    }
}

fn generate_cube_mesh(device: &wgpu::Device) -> Mesh {
    // Cube vertices (positions, normals, UVs)
    let vertices = vec![
        // Front face
        ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        
        // Back face
        ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 0.0, -1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 0.0, -1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 0.0, -1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 0.0, -1.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        
        // Left face
        ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [-1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], normal: [-1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], normal: [-1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [-1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        
        // Right face
        ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], normal: [1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], normal: [1.0, 0.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        
        // Top face
        ModelVertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        
        // Bottom face
        ModelVertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
    ];

    // Cube indices (2 triangles per face)
    let indices: Vec<u32> = vec![
        // Front face
        0, 1, 2,  2, 3, 0,
        // Back face
        4, 5, 6,  6, 7, 4,
        // Left face
        8, 9, 10,  10, 11, 8,
        // Right face
        12, 13, 14,  14, 15, 12,
        // Top face
        16, 17, 18,  18, 19, 16,
        // Bottom face
        20, 21, 22,  22, 23, 20,
    ];

    create_mesh_buffers(device, vertices, indices)
}

fn generate_plane_mesh(device: &wgpu::Device) -> Mesh {
    let vertices = vec![
        ModelVertex { position: [-0.5, 0.0, -0.5], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, 0.0, -0.5], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [ 0.5, 0.0,  0.5], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
        ModelVertex { position: [-0.5, 0.0,  0.5], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 1.0, 0.0] },
    ];

    let indices = vec![0, 1, 2,  2, 3, 0];
    create_mesh_buffers(device, vertices, indices)
}

fn generate_sphere_mesh(device: &wgpu::Device) -> Mesh {
    // Simple UV sphere generation
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let rings = 16;
    let sectors = 32;
    
    // Generate vertices
    for i in 0..=rings {
        let lat = std::f32::consts::PI * i as f32 / rings as f32;
        let y = lat.cos();
        let radius = lat.sin();
        
        for j in 0..=sectors {
            let lon = 2.0 * std::f32::consts::PI * j as f32 / sectors as f32;
            let x = radius * lon.cos();
            let z = radius * lon.sin();
            
            vertices.push(ModelVertex {
                position: [x * 0.5, y * 0.5, z * 0.5],
                tex_coords: [j as f32 / sectors as f32, i as f32 / rings as f32],
                normal: [x, y, z],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            });
        }
    }
    
    // Generate indices
    for i in 0..rings {
        for j in 0..sectors {
            let first = i * (sectors + 1) + j;
            let second = first + sectors + 1;
            
            indices.push(first);
            indices.push(second);
            indices.push(first + 1);
            
            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }
    
    create_mesh_buffers(device, vertices, indices)
}

fn generate_cylinder_mesh(device: &wgpu::Device) -> Mesh {
    // Simple cylinder generation
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let sectors = 16;
    let height = 1.0;
    let radius = 0.5;
    
    // Top and bottom centers
    vertices.push(ModelVertex { position: [0.0, height / 2.0, 0.0], tex_coords: [0.5, 0.5], normal: [0.0, 1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] });
    vertices.push(ModelVertex { position: [0.0, -height / 2.0, 0.0], tex_coords: [0.5, 0.5], normal: [0.0, -1.0, 0.0], tangent: [1.0, 0.0, 0.0], bitangent: [0.0, 0.0, 1.0] });
    
    // Side vertices
    for i in 0..sectors {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / sectors as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        
        // Top ring
        vertices.push(ModelVertex {
            position: [x, height / 2.0, z],
            tex_coords: [i as f32 / sectors as f32, 0.0],
            normal: [x / radius, 0.0, z / radius],
            tangent: [-z / radius, 0.0, x / radius],
            bitangent: [0.0, 1.0, 0.0],
        });
        
        // Bottom ring
        vertices.push(ModelVertex {
            position: [x, -height / 2.0, z],
            tex_coords: [i as f32 / sectors as f32, 1.0],
            normal: [x / radius, 0.0, z / radius],
            tangent: [-z / radius, 0.0, x / radius],
            bitangent: [0.0, 1.0, 0.0],
        });
    }
    
    // Generate indices for caps and sides
    for i in 0..sectors {
        let next = (i + 1) % sectors;
        
        // Top cap
        indices.extend_from_slice(&[0, 2 + i * 2, 2 + next * 2]);
        
        // Bottom cap
        indices.extend_from_slice(&[1, 2 + next * 2 + 1, 2 + i * 2 + 1]);
        
        // Side faces
        let top_curr = 2 + i * 2;
        let top_next = 2 + next * 2;
        let bot_curr = 2 + i * 2 + 1;
        let bot_next = 2 + next * 2 + 1;
        
        indices.extend_from_slice(&[top_curr, bot_curr, top_next]);
        indices.extend_from_slice(&[bot_curr, bot_next, top_next]);
    }
    
    create_mesh_buffers(device, vertices, indices)
}

fn generate_capsule_mesh(device: &wgpu::Device) -> Mesh {
    // For now, use cylinder as placeholder
    generate_cylinder_mesh(device)
}

fn create_mesh_buffers(device: &wgpu::Device, vertices: Vec<ModelVertex>, indices: Vec<u32>) -> Mesh {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Mesh {
        name: "Generated Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: indices.len() as u32,
    }
}


use serde::{Serialize, Deserialize};

pub const XSG_MAGIC: &[u8; 4] = b"XSG\0";
pub const XSG_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CompressionType {
    None,
    Lz4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub compression: CompressionType,
    pub uncompressed_size: u64, // To allocate buffer before decompression
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgFile {
    pub version: u32,
    pub nodes: Vec<XsgNode>,
    pub meshes: Vec<XsgMesh>,
    pub materials: Vec<XsgMaterial>,
    pub textures: Vec<XsgTexture>,
    pub animations: Vec<XsgAnimation>,
    pub root_nodes: Vec<u32>, // Indices into nodes
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgNode {
    pub name: String,
    pub transform: XsgTransform,
    pub children: Vec<u32>, // Indices into nodes
    pub mesh: Option<u32>,  // Index into meshes
    pub skin: Option<u32>,  // Index into skins (future)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub scale: [f32; 3],
}

impl Default for XsgTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgMesh {
    pub name: String,
    pub primitives: Vec<XsgPrimitive>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgPrimitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub material_index: Option<u32>, // Index into materials
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgMaterial {
    pub name: String,
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub base_color_texture: Option<u32>, // Index into textures
    pub normal_texture: Option<u32>,     // Index into textures
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgTexture {
    pub name: String,
    pub uri: Option<String>, // Path to external file (relative to XSG or absolute)
    pub data: Option<Vec<u8>>, // Embed raw bytes (png/jpg) or use raw buffers
    pub mime_type: Option<String>, // "image/png", "image/jpeg"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XsgAnimation {
    pub name: String,
    // TODO: Define animation channels and samplers
}

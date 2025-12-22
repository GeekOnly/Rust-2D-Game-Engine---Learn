
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use anyhow::{Context, Result};
use gltf;
use pollster;
use crate::assets::xsg::*;

pub struct XsgImporter;

impl XsgImporter {
    /// Convert GLTF/GLB file to XSG format and save to disk
    pub fn convert_gltf_to_xsg<P: AsRef<Path>, Q: AsRef<Path>>(input_path: P, output_path: Q) -> Result<()> {
        let xsg = Self::import_from_gltf(input_path)?;
        Self::save_to_file(&xsg, output_path)?;
        Ok(())
    }

    /// Import a GLTF/GLB file and convert it to XSG format
    pub fn import_from_gltf<P: AsRef<Path>>(path: P) -> Result<XsgFile> {
        let path = path.as_ref();
        let (document, buffers, images) = gltf::import(path)
            .context("Failed to import GLTF file")?;
            
        let mut xsg_nodes = Vec::new();
        let mut xsg_meshes = Vec::new();
        let mut xsg_materials = Vec::new();
        let mut xsg_textures = Vec::new();
        let mut root_nodes = Vec::new();

        // 1. Convert Textures
        // 1. Convert Textures
        for image in document.images() {
            let name = image.name().unwrap_or("Texture").to_string();
            let mut uri = None;
            let mut data = None;
            let mut mime_type = None;

            match image.source() {
                gltf::image::Source::Uri { uri: u, mime_type: m } => {
                    uri = Some(u.to_string());
                    mime_type = m.map(|s| s.to_string());
                },
                gltf::image::Source::View { view, mime_type: m } => {
                    mime_type = Some(m.to_string());
                    let start = view.offset();
                    let end = start + view.length();
                    let buffer_index = view.buffer().index();
                    if buffer_index < buffers.len() {
                        data = Some(buffers[buffer_index][start..end].to_vec());
                    }
                }
            }
            
            xsg_textures.push(XsgTexture {
                name,
                uri,
                data,
                mime_type,
            });
        }
        
        // 2. Convert Materials
        for material in document.materials() {
            let pbr = material.pbr_metallic_roughness();
            let base_color = pbr.base_color_factor();
            
            xsg_materials.push(XsgMaterial {
                name: material.name().unwrap_or("Material").to_string(),
                base_color_factor: base_color,
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                base_color_texture: pbr.base_color_texture().map(|t| t.texture().index() as u32),
                normal_texture: material.normal_texture().map(|t| t.texture().index() as u32),
            });
        }
        
        // 3. Convert Meshes
        for mesh in document.meshes() {
            let mut primitives = Vec::new();
            
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                
                let positions: Vec<[f32; 3]> = reader.read_positions()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();
                    
                let normals: Vec<[f32; 3]> = reader.read_normals()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();
                    
                let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect())
                    .unwrap_or_default();
                    
                let indices: Vec<u32> = reader.read_indices()
                    .map(|iter| iter.into_u32().collect())
                    .unwrap_or_default();
                    
                primitives.push(XsgPrimitive {
                    positions,
                    normals,
                    uvs,
                    indices,
                    material_index: primitive.material().index().map(|i| i as u32),
                });
            }
            
            xsg_meshes.push(XsgMesh {
                name: mesh.name().unwrap_or("Mesh").to_string(),
                primitives,
            });
        }
        
        // 4. Convert Nodes
        for node in document.nodes() {
            let (t, r, s) = node.transform().decomposed();
            
            let children: Vec<u32> = node.children().map(|c| c.index() as u32).collect();
            
            xsg_nodes.push(XsgNode {
                name: node.name().unwrap_or("Node").to_string(),
                transform: XsgTransform {
                    position: t,
                    rotation: r,
                    scale: s,
                },
                children,
                mesh: node.mesh().map(|m| m.index() as u32),
                skin: node.skin().map(|s| s.index() as u32),
            });
        }
        
        // 5. Get Scenes/Roots
        for scene in document.scenes() {
            for node in scene.nodes() {
                root_nodes.push(node.index() as u32);
            }
        }
        
        Ok(XsgFile {
            version: XSG_VERSION,
            nodes: xsg_nodes,
            meshes: xsg_meshes,
            materials: xsg_materials,
            textures: xsg_textures,
            animations: Vec::new(), // TODO
            root_nodes,
        })
    }
    
    /// Save XSG file to disk with compression
    pub fn save_to_file<P: AsRef<Path>>(xsg: &XsgFile, path: P) -> Result<()> {
        let file = File::create(path).context("Failed to create output file")?;
        let mut writer = std::io::BufWriter::new(file);
        
        // Serialize the full struct
        let serialized_data = bincode::serialize(xsg).context("Failed to serialize XSG data")?;
        let uncompressed_size = serialized_data.len() as u64;
        
        // Compress data
        // Using LZ4 for speed
        let compressed_data = lz4_flex::compress_prepend_size(&serialized_data);
        
        // Write Header
        let header = XsgHeader {
            magic: *XSG_MAGIC,
            version: XSG_VERSION,
            compression: CompressionType::Lz4,
            uncompressed_size,
        };
        
        // Write header manually or with bincode? 
        // Bincode is convenient but if we want strictly defined header it's better to be manual.
        // But for internal engine use, bincode for header is fine.
        let header_bytes = bincode::serialize(&header)?;
        let header_len = header_bytes.len() as u32;
        
        // Write header length first (4 bytes) so we know how much to read
        writer.write_all(&header_len.to_le_bytes())?;
        writer.write_all(&header_bytes)?;
        
        // Write Data
        writer.write_all(&compressed_data)?;
        
        Ok(())
    }
    
    /// Load XSG file from asset
    pub fn load_from_asset(asset_loader: &dyn engine_core::assets::AssetLoader, path: &str) -> Result<XsgFile> {
        let buffer = pollster::block_on(asset_loader.load_binary(path))
            .with_context(|| format!("Failed to load XSG file: {}", path))?;
        
        let mut cursor = std::io::Cursor::new(&buffer);
        
        // Read Header Length
        let mut header_len_bytes = [0u8; 4];
        cursor.read_exact(&mut header_len_bytes)?;
        let header_len = u32::from_le_bytes(header_len_bytes) as usize;
        
        // Read Header
        let header_end = 4 + header_len;
        if header_end > buffer.len() {
             anyhow::bail!("Invalid XSG file: header length exceeds file size");
        }
        let header_slice = &buffer[4..header_end];
        let header: XsgHeader = bincode::deserialize(header_slice)?;
        
        if &header.magic != XSG_MAGIC {
            anyhow::bail!("Invalid XSG file magic");
        }
        
        // Read Data
        let data_slice = &buffer[header_end..];
        
        let xsg: XsgFile = match header.compression {
            CompressionType::None => {
                bincode::deserialize(data_slice)?
            },
            CompressionType::Lz4 => {
                let decompressed = lz4_flex::decompress_size_prepended(data_slice)
                    .context("Failed to decompress XSG data")?;
                bincode::deserialize(&decompressed)?
            }
        };
        
        Ok(xsg)
    }
}

use ecs::{World, Entity, ScriptParameter};
use egui;
use std::collections::HashMap;

/// Parse hex color string to egui Color32
pub fn parse_hex_color(hex: &str) -> Result<egui::Color32, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color format".to_string());
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;
    
    Ok(egui::Color32::from_rgb(r, g, b))
}

/// Render Unity-style component header
pub fn render_component_header(ui: &mut egui::Ui, name: &str, icon: &str, always_open: bool) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(56, 56, 56))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if !always_open {
                    ui.label("▼");
                }
                ui.label(icon);
                ui.strong(name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("⋮").on_hover_text("Component Options").clicked() {
                        // Component menu
                    }
                });
            });
        });
    ui.add_space(8.0);
}

/// Format entity debug information for copying to clipboard
pub fn format_entity_debug_info(
    world: &World,
    entity: Entity,
    entity_names: &HashMap<Entity, String>,
) -> String {
    let mut info = String::new();
    
    // Entity header
    let name = entity_names.get(&entity).map(|s| s.as_str()).unwrap_or("Unnamed");
    info.push_str(&format!("=== Entity {} ({}) ===\n\n", entity, name));
    
    // Active state
    let is_active = world.active.get(&entity).copied().unwrap_or(true);
    info.push_str(&format!("Active: {}\n", is_active));
    
    // Layer
    let layer = world.layers.get(&entity).copied().unwrap_or(0);
    info.push_str(&format!("Layer: {}\n\n", layer));
    
    // Transform
    if let Some(transform) = world.transforms.get(&entity) {
        info.push_str("Transform:\n");
        info.push_str(&format!("  Position: [{:.2}, {:.2}, {:.2}]\n", 
            transform.position[0], transform.position[1], transform.position[2]));
        info.push_str(&format!("  Rotation: [{:.2}, {:.2}, {:.2}]\n", 
            transform.rotation[0], transform.rotation[1], transform.rotation[2]));
        info.push_str(&format!("  Scale: [{:.2}, {:.2}, {:.2}]\n\n", 
            transform.scale[0], transform.scale[1], transform.scale[2]));
    }
    
    // Sprite
    if let Some(sprite) = world.sprites.get(&entity) {
        info.push_str("Sprite Renderer:\n");
        info.push_str(&format!("  Texture: {}\n", sprite.texture_id));
        info.push_str(&format!("  Size: {:.1} x {:.1}\n", sprite.width, sprite.height));
        info.push_str(&format!("  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n", 
            sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]));
        info.push_str(&format!("  Billboard: {}\n", sprite.billboard));
        if let Some(rect) = sprite.sprite_rect {
            info.push_str(&format!("  Sprite Rect: [{}, {}, {}, {}]\n", rect[0], rect[1], rect[2], rect[3]));
        }
        info.push_str("\n");
    }
    
    // Box Collider
    if let Some(collider) = world.colliders.get(&entity) {
        info.push_str("Box Collider 2D:\n");
        info.push_str(&format!("  Offset: [{:.2}, {:.2}]\n", collider.offset[0], collider.offset[1]));
        info.push_str(&format!("  Size: [{:.2}, {:.2}]\n", collider.size[0], collider.size[1]));
        if let Some(transform) = world.transforms.get(&entity) {
            let world_width = collider.get_world_width(transform.scale[0]);
            let world_height = collider.get_world_height(transform.scale[1]);
            info.push_str(&format!("  World Size: {:.2} x {:.2}\n\n", world_width, world_height));
        } else {
            info.push_str("\n");
        }
    }
    
    // Rigidbody
    if let Some(rigidbody) = world.rigidbodies.get(&entity) {
        info.push_str("Rigidbody 2D:\n");
        info.push_str(&format!("  Velocity: [{:.2}, {:.2}]\n", 
            rigidbody.velocity.0, rigidbody.velocity.1));
        info.push_str(&format!("  Gravity Scale: {:.2}\n", rigidbody.gravity_scale));
        info.push_str(&format!("  Mass: {:.2}\n", rigidbody.mass));
        info.push_str(&format!("  Is Kinematic: {}\n", rigidbody.is_kinematic));
        info.push_str(&format!("  Freeze Rotation: {}\n\n", rigidbody.freeze_rotation));
    } else if let Some(velocity) = world.velocities.get(&entity) {
        info.push_str("Velocity (Legacy):\n");
        info.push_str(&format!("  [{:.2}, {:.2}]\n\n", velocity.0, velocity.1));
    }
    
    // Mesh
    if let Some(mesh) = world.meshes.get(&entity) {
        info.push_str("Mesh Renderer:\n");
        info.push_str(&format!("  Type: {:?}\n", mesh.mesh_type));
        info.push_str(&format!("  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n\n", 
            mesh.color[0], mesh.color[1], mesh.color[2], mesh.color[3]));
    }
    
    // Camera
    if let Some(camera) = world.cameras.get(&entity) {
        info.push_str("Camera:\n");
        info.push_str(&format!("  Projection: {:?}\n", camera.projection));
        info.push_str(&format!("  FOV: {:.1}°\n", camera.fov));
        info.push_str(&format!("  Orthographic Size: {:.1}\n", camera.orthographic_size));
        info.push_str(&format!("  Near Clip: {:.2}\n", camera.near_clip));
        info.push_str(&format!("  Far Clip: {:.1}\n", camera.far_clip));
        info.push_str(&format!("  Depth: {}\n\n", camera.depth));
    }
    
    // Script
    if let Some(script) = world.scripts.get(&entity) {
        info.push_str("Script:\n");
        info.push_str(&format!("  Name: {}\n", script.script_name));
        info.push_str(&format!("  Enabled: {}\n", script.enabled));
        if !script.parameters.is_empty() {
            info.push_str("  Parameters:\n");
            for (key, value) in &script.parameters {
                info.push_str(&format!("    {}: {:?}\n", key, value));
            }
        }
        info.push_str("\n");
    }
    
    // Tag
    if let Some(tag) = world.tags.get(&entity) {
        info.push_str(&format!("Tag: {:?}\n\n", tag));
    }
    
    // Hierarchy
    if let Some(parent) = world.parents.get(&entity) {
        let parent_name = entity_names.get(parent).map(|s| s.as_str()).unwrap_or("Unnamed");
        info.push_str(&format!("Parent: {} ({})\n", parent, parent_name));
    }
    
    let children = world.children.get(&entity);
    if let Some(children) = children {
        if !children.is_empty() {
            info.push_str(&format!("Children: {}\n", children.len()));
            for child in children {
                let child_name = entity_names.get(child).map(|s| s.as_str()).unwrap_or("Unnamed");
                info.push_str(&format!("  - {} ({})\n", child, child_name));
            }
        }
    }
    
    info
}

/// Parse Lua script file to extract variable declarations (Unity-like parameters)
/// Looks for patterns like: `local speed = 10`, `jumpForce = 5.0`, `name = "Player"`
pub fn parse_lua_script_parameters(script_path: &std::path::Path) -> HashMap<String, ScriptParameter> {
    let mut parameters = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(script_path) {
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Match patterns: "local name = value" or "name = value"
            if let Some(equals_pos) = trimmed.find('=') {
                let var_part = &trimmed[..equals_pos].trim();
                let value_part = trimmed[equals_pos + 1..].trim();

                // Remove "local" keyword if present
                let var_name = var_part
                    .strip_prefix("local")
                    .unwrap_or(var_part)
                    .trim()
                    .to_string();

                // Skip if variable name is empty or contains spaces (not a simple variable)
                if var_name.is_empty() || var_name.contains(' ') {
                    continue;
                }

                // Parse value type
                let param = if value_part.starts_with('"') || value_part.starts_with('\'') {
                    // String value
                    let str_value = value_part
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_end_matches(',')
                        .to_string();
                    Some(ScriptParameter::String(str_value))
                } else if value_part == "true" || value_part == "false" {
                    // Boolean value
                    let bool_value = value_part == "true";
                    Some(ScriptParameter::Bool(bool_value))
                } else if value_part.trim_end_matches(',') == "nil" {
                    // Entity reference (Unity-style GameObject)
                    // Pattern: local playerTarget = nil
                    Some(ScriptParameter::Entity(None))
                } else if let Ok(float_value) = value_part.trim_end_matches(',').parse::<f32>() {
                    // Try parsing as float first
                    if value_part.contains('.') {
                        Some(ScriptParameter::Float(float_value))
                    } else {
                        // Integer (no decimal point)
                        Some(ScriptParameter::Int(float_value as i32))
                    }
                } else {
                    None
                };

                if let Some(p) = param {
                    parameters.insert(var_name, p);
                }
            }
        }
    }

    parameters
}

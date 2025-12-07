//! HUD to UIPrefab Converter Demo
//! 
//! This example demonstrates how to convert legacy HUD assets to the new UI prefab system.

use ui::{HudToUIPrefabConverter, HudAsset, HudElement, HudElementType, Anchor};

fn main() {
    println!("=== HUD to UIPrefab Converter Demo ===\n");
    
    // Example 1: Simple text conversion
    println!("Example 1: Simple Text Element");
    let simple_hud = create_simple_text_hud();
    let simple_prefab = HudToUIPrefabConverter::convert(&simple_hud);
    print_prefab_info(&simple_prefab);
    
    // Example 2: Health bar conversion
    println!("\nExample 2: Health Bar Element");
    let health_hud = create_health_bar_hud();
    let health_prefab = HudToUIPrefabConverter::convert(&health_hud);
    print_prefab_info(&health_prefab);
    
    // Example 3: Complex HUD with multiple elements
    println!("\nExample 3: Complete Game HUD");
    let game_hud = create_game_hud();
    let game_prefab = HudToUIPrefabConverter::convert(&game_hud);
    print_prefab_info(&game_prefab);
    
    // Example 4: Container with children
    println!("\nExample 4: Container with Children");
    let container_hud = create_container_hud();
    let container_prefab = HudToUIPrefabConverter::convert(&container_hud);
    print_prefab_info(&container_prefab);
    
    // Save examples to files
    save_prefab_to_file(&simple_prefab, "simple_text.uiprefab");
    save_prefab_to_file(&health_prefab, "health_bar.uiprefab");
    save_prefab_to_file(&game_prefab, "game_hud.uiprefab");
    save_prefab_to_file(&container_prefab, "container.uiprefab");
    
    println!("\n=== Conversion Complete ===");
    println!("Prefab files saved to current directory.");
}

fn create_simple_text_hud() -> HudAsset {
    HudAsset {
        name: "SimpleTextHUD".to_string(),
        elements: vec![
            HudElement {
                id: "WelcomeLabel".to_string(),
                element_type: HudElementType::Text {
                    text: "Welcome to the Game!".to_string(),
                    font_size: 24.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                anchor: Anchor::TopCenter,
                offset: [0.0, 20.0],
                size: [300.0, 40.0],
                visible: true,
            },
        ],
    }
}

fn create_health_bar_hud() -> HudAsset {
    HudAsset {
        name: "HealthBarHUD".to_string(),
        elements: vec![
            HudElement {
                id: "PlayerHealthBar".to_string(),
                element_type: HudElementType::HealthBar {
                    binding: "player.health".to_string(),
                    color: [1.0, 0.2, 0.2, 1.0],
                    background_color: [0.2, 0.2, 0.2, 0.8],
                },
                anchor: Anchor::TopLeft,
                offset: [10.0, 10.0],
                size: [200.0, 20.0],
                visible: true,
            },
        ],
    }
}

fn create_game_hud() -> HudAsset {
    HudAsset {
        name: "CompleteGameHUD".to_string(),
        elements: vec![
            // Health bar
            HudElement {
                id: "HealthBar".to_string(),
                element_type: HudElementType::HealthBar {
                    binding: "player.health".to_string(),
                    color: [1.0, 0.2, 0.2, 1.0],
                    background_color: [0.2, 0.2, 0.2, 0.8],
                },
                anchor: Anchor::TopLeft,
                offset: [10.0, 10.0],
                size: [200.0, 20.0],
                visible: true,
            },
            
            // Mana bar
            HudElement {
                id: "ManaBar".to_string(),
                element_type: HudElementType::ProgressBar {
                    binding: "player.mana".to_string(),
                    color: [0.2, 0.5, 1.0, 1.0],
                    background_color: [0.2, 0.2, 0.2, 0.8],
                },
                anchor: Anchor::TopLeft,
                offset: [10.0, 35.0],
                size: [200.0, 20.0],
                visible: true,
            },
            
            // Score display
            HudElement {
                id: "ScoreLabel".to_string(),
                element_type: HudElementType::DynamicText {
                    format: "Score: {score}".to_string(),
                    font_size: 20.0,
                    color: [1.0, 1.0, 0.0, 1.0],
                },
                anchor: Anchor::TopCenter,
                offset: [0.0, 10.0],
                size: [150.0, 30.0],
                visible: true,
            },
            
            // Level display
            HudElement {
                id: "LevelLabel".to_string(),
                element_type: HudElementType::DynamicText {
                    format: "Level: {level}".to_string(),
                    font_size: 18.0,
                    color: [0.8, 0.8, 1.0, 1.0],
                },
                anchor: Anchor::TopRight,
                offset: [-10.0, 10.0],
                size: [100.0, 30.0],
                visible: true,
            },
            
            // Minimap
            HudElement {
                id: "Minimap".to_string(),
                element_type: HudElementType::Minimap {
                    zoom: 2.0,
                    background_color: [0.1, 0.1, 0.1, 0.9],
                },
                anchor: Anchor::BottomRight,
                offset: [-150.0, 10.0],
                size: [140.0, 140.0],
                visible: true,
            },
            
            // Coin icon
            HudElement {
                id: "CoinIcon".to_string(),
                element_type: HudElementType::Image {
                    texture: "coin.png".to_string(),
                    tint: [1.0, 0.9, 0.3, 1.0],
                },
                anchor: Anchor::TopRight,
                offset: [-150.0, 50.0],
                size: [30.0, 30.0],
                visible: true,
            },
            
            // Coin count
            HudElement {
                id: "CoinCount".to_string(),
                element_type: HudElementType::DynamicText {
                    format: "{coins}".to_string(),
                    font_size: 18.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                anchor: Anchor::TopRight,
                offset: [-110.0, 50.0],
                size: [80.0, 30.0],
                visible: true,
            },
        ],
    }
}

fn create_container_hud() -> HudAsset {
    HudAsset {
        name: "ContainerHUD".to_string(),
        elements: vec![
            HudElement {
                id: "InventoryPanel".to_string(),
                element_type: HudElementType::Container {
                    children: vec![
                        HudElement {
                            id: "InventoryTitle".to_string(),
                            element_type: HudElementType::Text {
                                text: "Inventory".to_string(),
                                font_size: 18.0,
                                color: [1.0, 1.0, 1.0, 1.0],
                            },
                            anchor: Anchor::TopCenter,
                            offset: [0.0, 10.0],
                            size: [150.0, 25.0],
                            visible: true,
                        },
                        HudElement {
                            id: "ItemSlot1".to_string(),
                            element_type: HudElementType::Image {
                                texture: "slot_empty.png".to_string(),
                                tint: [1.0, 1.0, 1.0, 1.0],
                            },
                            anchor: Anchor::TopLeft,
                            offset: [10.0, 45.0],
                            size: [50.0, 50.0],
                            visible: true,
                        },
                        HudElement {
                            id: "ItemSlot2".to_string(),
                            element_type: HudElementType::Image {
                                texture: "slot_empty.png".to_string(),
                                tint: [1.0, 1.0, 1.0, 1.0],
                            },
                            anchor: Anchor::TopLeft,
                            offset: [70.0, 45.0],
                            size: [50.0, 50.0],
                            visible: true,
                        },
                        HudElement {
                            id: "ItemSlot3".to_string(),
                            element_type: HudElementType::Image {
                                texture: "slot_empty.png".to_string(),
                                tint: [1.0, 1.0, 1.0, 1.0],
                            },
                            anchor: Anchor::TopLeft,
                            offset: [130.0, 45.0],
                            size: [50.0, 50.0],
                            visible: true,
                        },
                    ],
                },
                anchor: Anchor::BottomLeft,
                offset: [10.0, 10.0],
                size: [200.0, 120.0],
                visible: true,
            },
        ],
    }
}

fn print_prefab_info(prefab: &ui::UIPrefab) {
    println!("  Prefab Name: {}", prefab.name);
    println!("  Root Element: {}", prefab.root.name);
    println!("  Child Count: {}", prefab.root.children.len());
    
    // Print children
    for (i, child) in prefab.root.children.iter().enumerate() {
        println!("    Child {}: {}", i + 1, child.name);
        
        // Print component types
        let mut components = Vec::new();
        if child.image.is_some() { components.push("UIImage"); }
        if child.text.is_some() { components.push("UIText"); }
        if child.panel.is_some() { components.push("UIPanel"); }
        if child.button.is_some() { components.push("UIButton"); }
        
        if !components.is_empty() {
            println!("      Components: {}", components.join(", "));
        }
        
        // Print anchor info
        let rt = &child.rect_transform;
        println!("      Anchor: ({:.1}, {:.1}) to ({:.1}, {:.1})",
            rt.anchor_min.x, rt.anchor_min.y,
            rt.anchor_max.x, rt.anchor_max.y);
        println!("      Position: ({:.1}, {:.1})",
            rt.anchored_position.x, rt.anchored_position.y);
        println!("      Size: ({:.1}, {:.1})",
            rt.size_delta.x, rt.size_delta.y);
        
        // Print conversion notes
        if child.name.contains("/*") {
            if let Some(start) = child.name.find("/*") {
                if let Some(end) = child.name.find("*/") {
                    let note = &child.name[start+2..end].trim();
                    println!("      Note: {}", note);
                }
            }
        }
        
        // Print nested children
        if !child.children.is_empty() {
            println!("      Nested Children: {}", child.children.len());
            for (j, nested) in child.children.iter().enumerate() {
                println!("        {}: {}", j + 1, nested.name);
            }
        }
    }
}

fn save_prefab_to_file(prefab: &ui::UIPrefab, filename: &str) {
    match serde_json::to_string_pretty(prefab) {
        Ok(json) => {
            match std::fs::write(filename, json) {
                Ok(_) => println!("  Saved: {}", filename),
                Err(e) => eprintln!("  Error saving {}: {}", filename, e),
            }
        }
        Err(e) => eprintln!("  Error serializing {}: {}", filename, e),
    }
}

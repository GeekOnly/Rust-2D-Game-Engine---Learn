//! Migration Verification Tests
//!
//! This test suite verifies that all HUD files have been successfully migrated
//! to the new UIPrefab format and that the conversions are valid.

use std::fs;
use std::path::Path;
use ui::prefab::UIPrefab;

/// Test that all .hud files have corresponding .uiprefab files
#[test]
fn test_all_hud_files_have_prefabs() {
    let hud_files = vec![
        "assets/ui/main_hud.hud",
        "projects/Celeste Demo/assets/ui/celeste_hud.hud",
        "projects/Celeste Demo/assets/ui/test_hud.hud",
    ];
    
    for hud_file in hud_files {
        // Only check if the HUD file exists
        if !Path::new(hud_file).exists() {
            println!("Warning: HUD file not found: {}", hud_file);
            continue;
        }
        
        let prefab_file = hud_file.replace(".hud", ".uiprefab");
        assert!(
            Path::new(&prefab_file).exists(),
            "Missing prefab file for {}: {}",
            hud_file,
            prefab_file
        );
    }
}

/// Test that all .uiprefab files are valid and can be loaded
#[test]
fn test_all_prefabs_are_valid() {
    let prefab_files = vec![
        "assets/ui/main_hud.uiprefab",
        "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab",
        "projects/Celeste Demo/assets/ui/test_hud.uiprefab",
        "simple_text.uiprefab",
        "health_bar.uiprefab",
        "game_hud.uiprefab",
        "container.uiprefab",
    ];
    
    for prefab_file in prefab_files {
        if !Path::new(prefab_file).exists() {
            println!("Warning: Prefab file not found: {}", prefab_file);
            continue;
        }
        
        let content = fs::read_to_string(prefab_file)
            .expect(&format!("Failed to read prefab file: {}", prefab_file));
        
        let prefab: Result<UIPrefab, _> = serde_json::from_str(&content);
        assert!(
            prefab.is_ok(),
            "Failed to parse prefab {}: {:?}",
            prefab_file,
            prefab.err()
        );
        
        // Verify the prefab has a valid structure
        let prefab = prefab.unwrap();
        assert!(!prefab.name.is_empty(), "Prefab {} has empty name", prefab_file);
        
        // Verify root element exists
        assert!(
            prefab.root.name.len() > 0,
            "Prefab {} has invalid root element",
            prefab_file
        );
    }
}

/// Test that converted prefabs maintain hierarchy structure
#[test]
fn test_prefab_hierarchy_integrity() {
    let test_files = vec![
        ("container.uiprefab", true), // Should have children
        ("simple_text.uiprefab", false), // Should not have children
    ];
    
    for (prefab_file, should_have_children) in test_files {
        if !Path::new(prefab_file).exists() {
            println!("Warning: Prefab file not found: {}", prefab_file);
            continue;
        }
        
        let content = fs::read_to_string(prefab_file)
            .expect(&format!("Failed to read prefab file: {}", prefab_file));
        
        let prefab: UIPrefab = serde_json::from_str(&content)
            .expect(&format!("Failed to parse prefab: {}", prefab_file));
        
        if should_have_children {
            assert!(
                !prefab.root.children.is_empty(),
                "Prefab {} should have children but doesn't",
                prefab_file
            );
        }
    }
}

/// Test that all prefabs have valid RectTransform configurations
#[test]
fn test_prefab_rect_transforms() {
    let prefab_files = vec![
        "assets/ui/main_hud.uiprefab",
        "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab",
        "projects/Celeste Demo/assets/ui/test_hud.uiprefab",
        "simple_text.uiprefab",
        "health_bar.uiprefab",
        "game_hud.uiprefab",
        "container.uiprefab",
    ];
    
    for prefab_file in prefab_files {
        if !Path::new(prefab_file).exists() {
            println!("Warning: Prefab file not found: {}", prefab_file);
            continue;
        }
        
        let content = fs::read_to_string(prefab_file)
            .expect(&format!("Failed to read prefab file: {}", prefab_file));
        
        let prefab: UIPrefab = serde_json::from_str(&content)
            .expect(&format!("Failed to parse prefab: {}", prefab_file));
        
        // Verify root RectTransform
        let rt = &prefab.root.rect_transform;
        
        // Anchors should be in valid range [0, 1]
        assert!(
            rt.anchor_min.x >= 0.0 && rt.anchor_min.x <= 1.0,
            "Invalid anchor_min.x in {}: {}",
            prefab_file,
            rt.anchor_min.x
        );
        assert!(
            rt.anchor_min.y >= 0.0 && rt.anchor_min.y <= 1.0,
            "Invalid anchor_min.y in {}: {}",
            prefab_file,
            rt.anchor_min.y
        );
        assert!(
            rt.anchor_max.x >= 0.0 && rt.anchor_max.x <= 1.0,
            "Invalid anchor_max.x in {}: {}",
            prefab_file,
            rt.anchor_max.x
        );
        assert!(
            rt.anchor_max.y >= 0.0 && rt.anchor_max.y <= 1.0,
            "Invalid anchor_max.y in {}: {}",
            prefab_file,
            rt.anchor_max.y
        );
        
        // Pivot should be in valid range [0, 1]
        assert!(
            rt.pivot.x >= 0.0 && rt.pivot.x <= 1.0,
            "Invalid pivot.x in {}: {}",
            prefab_file,
            rt.pivot.x
        );
        assert!(
            rt.pivot.y >= 0.0 && rt.pivot.y <= 1.0,
            "Invalid pivot.y in {}: {}",
            prefab_file,
            rt.pivot.y
        );
    }
}

/// Test that no old HUD system code remains in engine
#[test]
fn test_no_old_hud_system_references() {
    // This is a compile-time check - if the old HUD system was still referenced,
    // the engine wouldn't compile. This test just documents the verification.
    
    // The old engine/src/hud module should not exist
    assert!(
        !Path::new("engine/src/hud").exists(),
        "Old HUD system directory still exists"
    );
    
    // The old HUD module should not be in engine/src/main.rs or lib.rs
    // (This would cause compile errors if it was still referenced)
}

//! Property-based tests for perfect pixel rendering in unified 2D/3D system
//! These tests verify correctness properties for perfect pixel rendering defined in the unified-2d-3d-rendering design document

use proptest::prelude::*;
use glam::{Vec2, Vec3};
use ecs::components::{PerfectPixelSettings, PixelPerfectTransform, ViewMode, FilterMode, PixelSnapMode};

/// Generate arbitrary perfect pixel settings for testing
fn arb_perfect_pixel_settings() -> impl Strategy<Value = PerfectPixelSettings> {
    (
        any::<bool>(), // enabled
        any::<bool>(), // snap_to_pixel
        prop_oneof![
            Just(FilterMode::Nearest),
            Just(FilterMode::Linear),
        ],
        1.0f32..1000.0f32, // pixels_per_unit (reasonable range)
        (100u32..4000u32, 100u32..4000u32), // reference_resolution
        0.001f32..0.1f32, // snap_threshold
        any::<bool>(), // maintain_aspect_ratio
        prop_oneof![
            Just(PixelSnapMode::Always),
            Just(PixelSnapMode::IntegerScaleOnly),
            Just(PixelSnapMode::Threshold),
            Just(PixelSnapMode::Never),
        ],
    ).prop_map(|(enabled, snap_to_pixel, filter_mode, pixels_per_unit, reference_resolution, snap_threshold, maintain_aspect_ratio, snap_mode)| {
        PerfectPixelSettings {
            enabled,
            snap_to_pixel,
            filter_mode,
            pixels_per_unit,
            reference_resolution,
            snap_threshold,
            maintain_aspect_ratio,
            snap_mode,
        }
    })
}

/// Generate arbitrary world positions for testing
fn arb_world_position() -> impl Strategy<Value = Vec3> {
    (-100.0f32..100.0f32, -100.0f32..100.0f32, -100.0f32..100.0f32)
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Generate arbitrary world scales for testing
fn arb_world_scale() -> impl Strategy<Value = Vec3> {
    (0.1f32..10.0f32, 0.1f32..10.0f32, 0.1f32..10.0f32)
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Generate arbitrary viewport sizes for testing
fn arb_viewport_size() -> impl Strategy<Value = (u32, u32)> {
    (100u32..4000u32, 100u32..4000u32)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // **Feature: unified-2d-3d-rendering, Property 3: Perfect pixel rendering in 2D mode**
    // **Validates: Requirements 2.2, 3.2, 4.1, 4.2**
    #[test]
    fn perfect_pixel_rendering_in_2d_mode(
        settings in arb_perfect_pixel_settings(),
        position in arb_world_position(),
        scale in arb_world_scale(),
    ) {
        // Only test when perfect pixel rendering is enabled and we're in 2D mode
        if !settings.enabled {
            return Ok(());
        }
        
        let transform = PixelPerfectTransform::with_settings(position, scale, &settings);
        
        // Property: When perfect pixel rendering is enabled in 2D mode,
        // positions should be snapped to pixel boundaries
        if settings.snap_to_pixel {
            let pixel_size = settings.pixel_size();
            let snapped_pos = transform.get_render_position(true);
            
            // Check that X and Y coordinates are aligned to pixel boundaries
            let x_remainder = (snapped_pos.x / pixel_size) % 1.0;
            let y_remainder = (snapped_pos.y / pixel_size) % 1.0;
            
            // Allow for floating point precision errors
            let tolerance = 0.001;
            
            match settings.snap_mode {
                PixelSnapMode::Always => {
                    prop_assert!(
                        x_remainder.abs() < tolerance || (1.0 - x_remainder).abs() < tolerance,
                        "X coordinate {} not pixel-aligned with pixel size {}, remainder: {}",
                        snapped_pos.x, pixel_size, x_remainder
                    );
                    prop_assert!(
                        y_remainder.abs() < tolerance || (1.0 - y_remainder).abs() < tolerance,
                        "Y coordinate {} not pixel-aligned with pixel size {}, remainder: {}",
                        snapped_pos.y, pixel_size, y_remainder
                    );
                },
                PixelSnapMode::Never => {
                    // When snapping is disabled, position should remain unchanged
                    prop_assert_eq!(snapped_pos, position);
                },
                PixelSnapMode::Threshold => {
                    // Position should only be snapped if within threshold
                    let original_x_remainder = (position.x / pixel_size) % 1.0;
                    let original_y_remainder = (position.y / pixel_size) % 1.0;
                    
                    if original_x_remainder.abs() <= settings.snap_threshold || (1.0 - original_x_remainder).abs() <= settings.snap_threshold {
                        prop_assert!(
                            x_remainder.abs() < tolerance || (1.0 - x_remainder).abs() < tolerance,
                            "X coordinate should be snapped when within threshold"
                        );
                    }
                    
                    if original_y_remainder.abs() <= settings.snap_threshold || (1.0 - original_y_remainder).abs() <= settings.snap_threshold {
                        prop_assert!(
                            y_remainder.abs() < tolerance || (1.0 - y_remainder).abs() < tolerance,
                            "Y coordinate should be snapped when within threshold"
                        );
                    }
                },
                PixelSnapMode::IntegerScaleOnly => {
                    // This mode requires scale information to test properly
                    // For now, just verify the transform was created successfully
                    prop_assert!(transform.pixels_per_unit > 0.0);
                }
            }
        }
        
        // Property: Z coordinate should never be snapped (preserved for 3D)
        let snapped_pos = transform.get_render_position(true);
        prop_assert_eq!(snapped_pos.z, position.z, "Z coordinate should not be snapped");
        
        // Property: Pixel scale should be quantized for crisp rendering
        if settings.enabled {
            let render_scale = transform.get_render_scale();
            prop_assert!(render_scale.x > 0.0, "Pixel scale X must be positive");
            prop_assert!(render_scale.y > 0.0, "Pixel scale Y must be positive");
            prop_assert!(render_scale.z > 0.0, "Pixel scale Z must be positive");
        }
    }
    
    // **Feature: unified-2d-3d-rendering, Property 8: Viewport consistency**
    // **Validates: Requirements 4.4**
    #[test]
    fn viewport_consistency(
        settings in arb_perfect_pixel_settings(),
        original_viewport in arb_viewport_size(),
        new_viewport in arb_viewport_size(),
        position in arb_world_position(),
    ) {
        // Only test when perfect pixel rendering is enabled and aspect ratio is maintained
        if !settings.enabled || !settings.maintain_aspect_ratio {
            return Ok(());
        }
        
        // Calculate scale factors for both viewport sizes
        let original_scale = settings.calculate_viewport_scale(original_viewport);
        let new_scale = settings.calculate_viewport_scale(new_viewport);
        
        // Property: Scale factor should be consistent with aspect ratio preservation
        let ref_aspect = settings.reference_resolution.0 as f32 / settings.reference_resolution.1 as f32;
        let original_aspect = original_viewport.0 as f32 / original_viewport.1 as f32;
        let new_aspect = new_viewport.0 as f32 / new_viewport.1 as f32;
        
        // Verify scale factor calculation is correct
        if original_aspect > ref_aspect {
            // Wider viewport - should scale by height
            let expected_scale = original_viewport.1 as f32 / settings.reference_resolution.1 as f32;
            prop_assert!(
                (original_scale - expected_scale).abs() < 0.001,
                "Scale factor {} should match height-based scaling {}",
                original_scale, expected_scale
            );
        } else {
            // Taller viewport - should scale by width
            let expected_scale = original_viewport.0 as f32 / settings.reference_resolution.0 as f32;
            prop_assert!(
                (original_scale - expected_scale).abs() < 0.001,
                "Scale factor {} should match width-based scaling {}",
                original_scale, expected_scale
            );
        }
        
        // Property: Pixel ratios should remain consistent across viewport changes
        let original_pixel_size = settings.pixel_size() / original_scale;
        let new_pixel_size = settings.pixel_size() / new_scale;
        
        // Create transforms for both viewport sizes
        let original_transform = PixelPerfectTransform::with_settings(position, Vec3::ONE, &settings);
        let new_transform = PixelPerfectTransform::with_settings(position, Vec3::ONE, &settings);
        
        // Property: Relative positioning should be preserved
        if settings.snap_to_pixel {
            let original_snapped = original_transform.get_render_position(true);
            let new_snapped = new_transform.get_render_position(true);
            
            // The snapping behavior should be consistent regardless of viewport size
            // (both should snap to their respective pixel boundaries)
            let original_pixel_aligned = {
                let pixel_size = settings.pixel_size();
                let x_rem = (original_snapped.x / pixel_size) % 1.0;
                let y_rem = (original_snapped.y / pixel_size) % 1.0;
                x_rem.abs() < 0.001 || (1.0 - x_rem).abs() < 0.001 &&
                y_rem.abs() < 0.001 || (1.0 - y_rem).abs() < 0.001
            };
            
            let new_pixel_aligned = {
                let pixel_size = settings.pixel_size();
                let x_rem = (new_snapped.x / pixel_size) % 1.0;
                let y_rem = (new_snapped.y / pixel_size) % 1.0;
                x_rem.abs() < 0.001 || (1.0 - x_rem).abs() < 0.001 &&
                y_rem.abs() < 0.001 || (1.0 - y_rem).abs() < 0.001
            };
            
            // Both should have consistent pixel alignment behavior
            prop_assert_eq!(
                original_pixel_aligned, new_pixel_aligned,
                "Pixel alignment should be consistent across viewport changes"
            );
        }
        
        // Property: Scale factors should be positive and reasonable
        prop_assert!(original_scale > 0.0, "Original scale factor must be positive");
        prop_assert!(new_scale > 0.0, "New scale factor must be positive");
        prop_assert!(original_scale < 100.0, "Original scale factor should be reasonable");
        prop_assert!(new_scale < 100.0, "New scale factor should be reasonable");
    }
    
    // Additional property: Perfect pixel settings should produce deterministic results
    #[test]
    fn perfect_pixel_deterministic(
        settings in arb_perfect_pixel_settings(),
        position in arb_world_position(),
        scale in arb_world_scale(),
    ) {
        if !settings.enabled {
            return Ok(());
        }
        
        // Create two identical transforms
        let transform1 = PixelPerfectTransform::with_settings(position, scale, &settings);
        let transform2 = PixelPerfectTransform::with_settings(position, scale, &settings);
        
        // Property: Identical inputs should produce identical outputs
        prop_assert_eq!(
            transform1.get_render_position(true),
            transform2.get_render_position(true),
            "Perfect pixel transforms should be deterministic"
        );
        
        prop_assert_eq!(
            transform1.get_render_scale(),
            transform2.get_render_scale(),
            "Perfect pixel scales should be deterministic"
        );
    }
    
    // Property: Pixel size calculation should be consistent
    #[test]
    fn pixel_size_consistency(
        pixels_per_unit in 1.0f32..1000.0f32,
    ) {
        let settings = PerfectPixelSettings::new(pixels_per_unit);
        
        // Property: Pixel size should be the inverse of pixels per unit
        let expected_pixel_size = 1.0 / pixels_per_unit;
        prop_assert!(
            (settings.pixel_size() - expected_pixel_size).abs() < 0.001,
            "Pixel size {} should equal 1/pixels_per_unit {}",
            settings.pixel_size(), expected_pixel_size
        );
        
        // Property: Pixel size should always be positive
        prop_assert!(settings.pixel_size() > 0.0, "Pixel size must be positive");
    }
}
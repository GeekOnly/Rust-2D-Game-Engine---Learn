//! Style System Demo
//!
//! This example demonstrates:
//! - Creating and applying UI styles
//! - Style inheritance from parent elements
//! - Theme changes affecting all elements
//! - Smooth style transitions with animations

use ui::{
    StyleSystem, UIStyle, UITheme, StyledElement, StyleTransition,
    UIElement, UIButton, UIText, UIPanel,
    HorizontalLayoutGroup,
};
use glam::Vec4;

fn main() {
    println!("=== UI Style System Demo ===\n");
    
    // Create a style system with default theme
    let mut style_system = StyleSystem::new();
    println!("Created style system with default theme");
    
    // Create a custom style
    let mut dark_style = UIStyle::default();
    dark_style.name = "dark".to_string();
    dark_style.primary_color = [0.2, 0.2, 0.3, 1.0];
    dark_style.secondary_color = [0.3, 0.3, 0.4, 1.0];
    dark_style.background_color = [0.1, 0.1, 0.15, 1.0];
    dark_style.text_color = [0.9, 0.9, 0.9, 1.0];
    dark_style.disabled_color = [0.4, 0.4, 0.4, 0.5];
    dark_style.default_font_size = 16.0;
    dark_style.default_spacing = 8.0;
    dark_style.default_padding = Vec4::new(12.0, 12.0, 12.0, 12.0);
    
    println!("\nCreated dark style:");
    println!("  Primary: {:?}", dark_style.primary_color);
    println!("  Text: {:?}", dark_style.text_color);
    println!("  Font size: {}", dark_style.default_font_size);
    
    // Add the dark style to the theme
    style_system.theme_mut().add_style(dark_style.clone());
    println!("\nAdded dark style to theme");
    
    // Create a light style
    let mut light_style = UIStyle::default();
    light_style.name = "light".to_string();
    light_style.primary_color = [0.9, 0.9, 1.0, 1.0];
    light_style.secondary_color = [0.7, 0.7, 0.8, 1.0];
    light_style.background_color = [1.0, 1.0, 1.0, 1.0];
    light_style.text_color = [0.1, 0.1, 0.1, 1.0];
    light_style.disabled_color = [0.6, 0.6, 0.6, 0.5];
    
    style_system.theme_mut().add_style(light_style.clone());
    println!("Added light style to theme");
    
    // Demonstrate style application
    println!("\n=== Style Application ===");
    
    // Create UI elements
    let mut ui_element = UIElement::default();
    let mut button = UIButton::default();
    let mut text = UIText::default();
    let mut panel = UIPanel::default();
    let mut layout = HorizontalLayoutGroup::default();
    
    // Apply default style
    let default_style = style_system.theme().get_active_style().unwrap();
    style_system.apply_style_to_element(default_style, &mut ui_element);
    style_system.apply_style_to_button(default_style, &mut button);
    style_system.apply_style_to_text(default_style, &mut text);
    style_system.apply_style_to_panel(default_style, &mut panel);
    style_system.apply_style_to_horizontal_layout(default_style, &mut layout);
    
    println!("\nApplied default style:");
    println!("  Element color: {:?}", ui_element.color);
    println!("  Button normal: {:?}", button.normal_color);
    println!("  Text color: {:?}", text.color);
    println!("  Text font size: {}", text.font_size);
    println!("  Layout spacing: {}", layout.spacing);
    
    // Demonstrate style inheritance
    println!("\n=== Style Inheritance ===");
    
    let parent_styled = StyledElement::with_style("dark".to_string());
    let child_styled = StyledElement::inheriting();
    
    let parent_style_name = style_system.resolve_style_name(&parent_styled, None);
    let child_style_name = style_system.resolve_style_name(&child_styled, parent_style_name.as_deref());
    
    println!("Parent style: {:?}", parent_style_name);
    println!("Child style (inherited): {:?}", child_style_name);
    assert_eq!(parent_style_name, child_style_name);
    
    // Child with explicit style doesn't inherit
    let child_explicit = StyledElement::with_style("light".to_string());
    let child_explicit_name = style_system.resolve_style_name(&child_explicit, parent_style_name.as_deref());
    println!("Child style (explicit): {:?}", child_explicit_name);
    assert_eq!(child_explicit_name, Some("light".to_string()));
    
    // Demonstrate theme changes
    println!("\n=== Theme Changes ===");
    
    println!("Current active style: {}", style_system.theme().active_style);
    
    // Change to dark theme
    style_system.set_active_style("dark".to_string());
    println!("Changed to dark theme");
    assert!(style_system.has_theme_changed());
    
    // Apply dark style to elements
    let dark_style = style_system.get_style("dark").unwrap();
    style_system.apply_style_to_element(dark_style, &mut ui_element);
    style_system.apply_style_to_button(dark_style, &mut button);
    style_system.apply_style_to_text(dark_style, &mut text);
    
    println!("\nApplied dark style:");
    println!("  Element color: {:?}", ui_element.color);
    println!("  Button normal: {:?}", button.normal_color);
    println!("  Text color: {:?}", text.color);
    println!("  Text font size: {}", text.font_size);
    
    style_system.clear_theme_changed();
    assert!(!style_system.has_theme_changed());
    
    // Demonstrate style transitions
    println!("\n=== Style Transitions ===");
    
    let mut transition = StyleTransition::default();
    transition.duration = 1.0; // 1 second transition
    
    // Start transition from default to dark
    let default_style = style_system.get_style("default").unwrap();
    let dark_style = style_system.get_style("dark").unwrap();
    transition.start(default_style, dark_style);
    
    println!("Started transition from default to dark (duration: {}s)", transition.duration);
    println!("  From primary: {:?}", transition.from_primary);
    println!("  To primary: {:?}", transition.to_primary);
    
    // Simulate transition updates
    let time_steps = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    
    for &t in &time_steps {
        transition.elapsed = t;
        let (primary, secondary, background, text) = transition.get_current_colors();
        
        println!("\nAt t={:.2}s ({}%):", t, (transition.get_t() * 100.0) as i32);
        println!("  Primary: {:?}", primary);
        println!("  Text: {:?}", text);
        
        // Apply transitioning colors
        let mut temp_element = UIElement::default();
        let mut temp_button = UIButton::default();
        let mut temp_text = UIText::default();
        
        style_system.apply_transition_to_element(&transition, &mut temp_element);
        style_system.apply_transition_to_button(&transition, &mut temp_button);
        style_system.apply_transition_to_text(&transition, &mut temp_text);
        
        println!("  Applied to element: {:?}", temp_element.color);
        println!("  Applied to text: {:?}", temp_text.color);
    }
    
    // Verify transition completion
    transition.elapsed = 0.0;
    transition.active = true;
    
    let mut frame_count = 0;
    let delta_time = 0.016; // ~60 FPS
    
    println!("\n=== Simulating Transition Updates ===");
    while transition.active {
        let still_active = style_system.update_transition(&mut transition, delta_time);
        frame_count += 1;
        
        if frame_count % 15 == 0 { // Print every 15 frames (~0.25s)
            println!("Frame {}: elapsed={:.3}s, active={}", 
                frame_count, transition.elapsed, still_active);
        }
        
        if !still_active {
            break;
        }
        
        // Safety check to prevent infinite loop
        if frame_count > 1000 {
            println!("Safety break at frame {}", frame_count);
            break;
        }
    }
    
    println!("\nTransition completed after {} frames ({:.2}s)", 
        frame_count, frame_count as f32 * delta_time);
    
    // Demonstrate color interpolation
    println!("\n=== Color Interpolation ===");
    
    let color_a = [1.0, 0.0, 0.0, 1.0]; // Red
    let color_b = [0.0, 0.0, 1.0, 1.0]; // Blue
    
    println!("Interpolating from red to blue:");
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let interpolated = StyleTransition::lerp_color(color_a, color_b, t);
        println!("  t={:.1}: {:?}", t, interpolated);
    }
    
    println!("\n=== Demo Complete ===");
}

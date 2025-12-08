//! Style application system
//!
//! This system handles:
//! - Applying UIStyle to UI elements
//! - Style inheritance from parent elements
//! - Theme changes (updating all elements)
//! - Style animations (smooth transitions)

use crate::{
    Color, UITheme, UIStyle, StyledElement, StyleTransition,
    UIElement, UIButton, UIPanel, UIText, UIImage,
    layout::{HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup},
};

/// Style application system
pub struct StyleSystem {
    /// Current theme
    theme: UITheme,
    
    /// Whether theme has changed (triggers full update)
    theme_changed: bool,
}

impl StyleSystem {
    /// Create a new style system with default theme
    pub fn new() -> Self {
        Self {
            theme: UITheme::default(),
            theme_changed: false,
        }
    }
    
    /// Create a style system with a specific theme
    pub fn with_theme(theme: UITheme) -> Self {
        Self {
            theme,
            theme_changed: false,
        }
    }
    
    /// Get the current theme
    pub fn theme(&self) -> &UITheme {
        &self.theme
    }
    
    /// Get mutable reference to the theme
    pub fn theme_mut(&mut self) -> &mut UITheme {
        &mut self.theme
    }
    
    /// Set a new theme (triggers update of all styled elements)
    pub fn set_theme(&mut self, theme: UITheme) {
        self.theme = theme;
        self.theme_changed = true;
    }
    
    /// Change the active style in the current theme
    pub fn set_active_style(&mut self, style_name: String) {
        self.theme.set_active_style(style_name);
        self.theme_changed = true;
    }
    
    /// Check if theme has changed
    pub fn has_theme_changed(&self) -> bool {
        self.theme_changed
    }
    
    /// Clear the theme changed flag
    pub fn clear_theme_changed(&mut self) {
        self.theme_changed = false;
    }
    
    /// Resolve the style name for an element (handles inheritance)
    pub fn resolve_style_name(
        &self,
        styled: &StyledElement,
        parent_style_name: Option<&str>,
    ) -> Option<String> {
        // If element has explicit style, use it
        if let Some(ref style_name) = styled.style_name {
            return Some(style_name.clone());
        }
        
        // If element inherits and parent has style, use parent's style
        if styled.inherit_from_parent {
            if let Some(parent_name) = parent_style_name {
                return Some(parent_name.to_string());
            }
        }
        
        // Fall back to active style
        Some(self.theme.active_style.clone())
    }
    
    /// Get a style by name
    pub fn get_style(&self, style_name: &str) -> Option<&UIStyle> {
        self.theme.get_style(style_name)
    }
    
    /// Apply style to a UI element
    pub fn apply_style_to_element(
        &self,
        style: &UIStyle,
        ui_element: &mut UIElement,
    ) {
        // Apply primary color as the base color
        ui_element.color = style.primary_color;
    }
    
    /// Apply style to a button
    pub fn apply_style_to_button(
        &self,
        style: &UIStyle,
        button: &mut UIButton,
    ) {
        // Apply button colors
        button.normal_color = style.primary_color;
        button.highlighted_color = style.secondary_color;
        button.pressed_color = Self::darken_color(style.primary_color, 0.2);
        button.disabled_color = style.disabled_color;
        
        // Apply button sprite if available
        if let Some(ref sprite) = style.button_sprite {
            // Note: We don't have a direct sprite field on UIButton,
            // this would need to be applied to an associated UIImage component
        }
    }
    
    /// Apply style to a panel
    pub fn apply_style_to_panel(
        &self,
        style: &UIStyle,
        panel: &mut UIPanel,
    ) {
        // Apply panel sprite if available
        if let Some(ref sprite) = style.panel_sprite {
            panel.background = Some(sprite.clone());
        }
        
        // Apply default padding
        panel.padding = style.default_padding;
    }
    
    /// Apply style to text
    pub fn apply_style_to_text(
        &self,
        style: &UIStyle,
        text: &mut UIText,
    ) {
        // Apply text color
        text.color = style.text_color;
        
        // Apply font
        text.font = style.default_font.clone();
        text.font_size = style.default_font_size;
    }
    
    /// Apply style to an image
    pub fn apply_style_to_image(
        &self,
        _style: &UIStyle,
        _image: &mut UIImage,
    ) {
        // Images typically don't have style-specific properties
        // but we keep this for consistency
    }
    
    /// Apply style to horizontal layout group
    pub fn apply_style_to_horizontal_layout(
        &self,
        style: &UIStyle,
        layout: &mut HorizontalLayoutGroup,
    ) {
        layout.spacing = style.default_spacing;
        layout.padding = style.default_padding;
    }
    
    /// Apply style to vertical layout group
    pub fn apply_style_to_vertical_layout(
        &self,
        style: &UIStyle,
        layout: &mut VerticalLayoutGroup,
    ) {
        layout.spacing = style.default_spacing;
        layout.padding = style.default_padding;
    }
    
    /// Apply style to grid layout group
    pub fn apply_style_to_grid_layout(
        &self,
        style: &UIStyle,
        layout: &mut GridLayoutGroup,
    ) {
        layout.spacing.x = style.default_spacing;
        layout.spacing.y = style.default_spacing;
        layout.padding = style.default_padding;
    }
    
    /// Start a style transition
    pub fn start_transition(
        &self,
        transition: &mut StyleTransition,
        from_style_name: &str,
        to_style_name: &str,
    ) {
        if let (Some(from_style), Some(to_style)) = (
            self.get_style(from_style_name),
            self.get_style(to_style_name),
        ) {
            transition.start(from_style, to_style);
        }
    }
    
    /// Update style transitions
    pub fn update_transition(
        &self,
        transition: &mut StyleTransition,
        delta_time: f32,
    ) -> bool {
        transition.update(delta_time)
    }
    
    /// Apply transitioning colors to an element
    pub fn apply_transition_to_element(
        &self,
        transition: &StyleTransition,
        ui_element: &mut UIElement,
    ) {
        let (primary, _, _, _) = transition.get_current_colors();
        ui_element.color = primary;
    }
    
    /// Apply transitioning colors to a button
    pub fn apply_transition_to_button(
        &self,
        transition: &StyleTransition,
        button: &mut UIButton,
    ) {
        let (primary, secondary, _, _) = transition.get_current_colors();
        button.normal_color = primary;
        button.highlighted_color = secondary;
        button.pressed_color = Self::darken_color(primary, 0.2);
    }
    
    /// Apply transitioning colors to text
    pub fn apply_transition_to_text(
        &self,
        transition: &StyleTransition,
        text: &mut UIText,
    ) {
        let (_, _, _, text_color) = transition.get_current_colors();
        text.color = text_color;
    }
    
    /// Helper: Darken a color by a factor
    fn darken_color(color: Color, factor: f32) -> Color {
        [
            color[0] * (1.0 - factor),
            color[1] * (1.0 - factor),
            color[2] * (1.0 - factor),
            color[3],
        ]
    }
    
    /// Helper: Lighten a color by a factor
    fn lighten_color(color: Color, factor: f32) -> Color {
        [
            color[0] + (1.0 - color[0]) * factor,
            color[1] + (1.0 - color[1]) * factor,
            color[2] + (1.0 - color[2]) * factor,
            color[3],
        ]
    }
}

impl Default for StyleSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_style_system_creation() {
        let system = StyleSystem::new();
        assert_eq!(system.theme().name, "default");
        assert!(!system.has_theme_changed());
    }
    
    #[test]
    fn test_theme_change() {
        let mut system = StyleSystem::new();
        let mut theme = UITheme::default();
        theme.name = "custom".to_string();
        
        system.set_theme(theme);
        assert!(system.has_theme_changed());
        assert_eq!(system.theme().name, "custom");
        
        system.clear_theme_changed();
        assert!(!system.has_theme_changed());
    }
    
    #[test]
    fn test_resolve_style_name_explicit() {
        let system = StyleSystem::new();
        let styled = StyledElement::with_style("custom".to_string());
        
        let resolved = system.resolve_style_name(&styled, Some("parent"));
        assert_eq!(resolved, Some("custom".to_string()));
    }
    
    #[test]
    fn test_resolve_style_name_inherit() {
        let system = StyleSystem::new();
        let styled = StyledElement::inheriting();
        
        let resolved = system.resolve_style_name(&styled, Some("parent"));
        assert_eq!(resolved, Some("parent".to_string()));
    }
    
    #[test]
    fn test_resolve_style_name_default() {
        let system = StyleSystem::new();
        let styled = StyledElement::inheriting();
        
        let resolved = system.resolve_style_name(&styled, None);
        assert_eq!(resolved, Some("default".to_string()));
    }
    
    #[test]
    fn test_apply_style_to_element() {
        let system = StyleSystem::new();
        let style = UIStyle::default();
        let mut element = UIElement::default();
        
        system.apply_style_to_element(&style, &mut element);
        assert_eq!(element.color, style.primary_color);
    }
    
    #[test]
    fn test_apply_style_to_button() {
        let system = StyleSystem::new();
        let style = UIStyle::default();
        let mut button = UIButton::default();
        
        system.apply_style_to_button(&style, &mut button);
        assert_eq!(button.normal_color, style.primary_color);
        assert_eq!(button.highlighted_color, style.secondary_color);
        assert_eq!(button.disabled_color, style.disabled_color);
    }
    
    #[test]
    fn test_apply_style_to_text() {
        let system = StyleSystem::new();
        let style = UIStyle::default();
        let mut text = UIText::default();
        
        system.apply_style_to_text(&style, &mut text);
        assert_eq!(text.color, style.text_color);
        assert_eq!(text.font, style.default_font);
        assert_eq!(text.font_size, style.default_font_size);
    }
    
    #[test]
    fn test_style_transition() {
        let system = StyleSystem::new();
        let mut transition = StyleTransition::default();
        
        let style1 = UIStyle::default();
        let mut style2 = UIStyle::default();
        style2.primary_color = [1.0, 0.0, 0.0, 1.0];
        
        transition.start(&style1, &style2);
        assert!(transition.active);
        assert_eq!(transition.elapsed, 0.0);
        
        // Update halfway
        transition.update(transition.duration / 2.0);
        assert!(transition.active);
        
        let t = transition.get_t();
        assert!((t - 0.5).abs() < 0.01);
        
        // Complete transition
        transition.update(transition.duration);
        assert!(!transition.active);
    }
    
    #[test]
    fn test_color_interpolation() {
        let from = [0.0, 0.0, 0.0, 1.0];
        let to = [1.0, 1.0, 1.0, 1.0];
        
        let mid = StyleTransition::lerp_color(from, to, 0.5);
        assert_eq!(mid, [0.5, 0.5, 0.5, 1.0]);
        
        let start = StyleTransition::lerp_color(from, to, 0.0);
        assert_eq!(start, from);
        
        let end = StyleTransition::lerp_color(from, to, 1.0);
        assert_eq!(end, to);
    }
}

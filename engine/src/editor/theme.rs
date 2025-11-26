/// Unity-like dark theme for the editor
use egui::{Color32, Rounding, Stroke, Visuals, Margin};

pub struct UnityTheme;

impl UnityTheme {
    /// Apply Unity-like dark theme to egui context
    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Unity colors
        let bg_dark = Color32::from_rgb(32, 32, 32);           // Main background
        let bg_medium = Color32::from_rgb(42, 42, 42);         // Panel background
        let bg_light = Color32::from_rgb(56, 56, 56);          // Hover/selected
        let border = Color32::from_rgb(26, 26, 26);            // Borders
        let text = Color32::from_rgb(210, 210, 210);           // Text
        let _text_dim = Color32::from_rgb(150, 150, 150);       // Disabled text
        let accent = Color32::from_rgb(44, 93, 135);           // Unity blue
        let accent_hover = Color32::from_rgb(58, 122, 177);    // Lighter blue
        
        // Set dark mode
        style.visuals = Visuals::dark();
        
        // Override with Unity colors
        style.visuals.widgets.noninteractive.bg_fill = bg_medium;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);
        
        style.visuals.widgets.inactive.bg_fill = bg_medium;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
        
        style.visuals.widgets.hovered.bg_fill = bg_light;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text);
        
        style.visuals.widgets.active.bg_fill = accent;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent_hover);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        
        // Selection
        style.visuals.selection.bg_fill = accent;
        style.visuals.selection.stroke = Stroke::new(1.0, accent_hover);
        
        // Window
        style.visuals.window_fill = bg_medium;
        style.visuals.window_stroke = Stroke::new(1.0, border);
        style.visuals.window_rounding = Rounding::same(0.0); // Sharp corners like Unity
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 0.0),
            blur: 0.0,
            spread: 0.0,
            color: Color32::TRANSPARENT,
        }; // No shadow
        
        // Panel
        style.visuals.panel_fill = bg_dark;
        
        // Extreme background (for popups)
        style.visuals.extreme_bg_color = bg_dark;
        
        // Text colors
        style.visuals.override_text_color = Some(text);
        style.visuals.warn_fg_color = Color32::from_rgb(255, 200, 0);
        style.visuals.error_fg_color = Color32::from_rgb(255, 80, 80);
        
        // Spacing (Unity-like tight spacing)
        style.spacing.item_spacing = egui::vec2(4.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.window_margin = Margin::same(8.0);
        style.spacing.menu_margin = Margin::same(4.0);
        
        // Rounding (minimal like Unity)
        style.visuals.widgets.noninteractive.rounding = Rounding::same(2.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(2.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(2.0);
        style.visuals.widgets.active.rounding = Rounding::same(2.0);
        
        ctx.set_style(style);
    }
    
    /// Unity color palette
    pub fn colors() -> UnityColors {
        UnityColors {
            bg_dark: Color32::from_rgb(32, 32, 32),
            bg_medium: Color32::from_rgb(42, 42, 42),
            bg_light: Color32::from_rgb(56, 56, 56),
            border: Color32::from_rgb(26, 26, 26),
            text: Color32::from_rgb(210, 210, 210),
            text_dim: Color32::from_rgb(150, 150, 150),
            accent: Color32::from_rgb(44, 93, 135),
            accent_hover: Color32::from_rgb(58, 122, 177),
            hierarchy_bg: Color32::from_rgb(38, 38, 38),
            inspector_bg: Color32::from_rgb(38, 38, 38),
            scene_bg: Color32::from_rgb(40, 40, 50),
            toolbar_bg: Color32::from_rgb(48, 48, 48),
            selected: Color32::from_rgb(44, 93, 135),
            selected_hover: Color32::from_rgb(58, 122, 177),
        }
    }
}

#[derive(Clone, Copy)]
pub struct UnityColors {
    pub bg_dark: Color32,
    pub bg_medium: Color32,
    pub bg_light: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub hierarchy_bg: Color32,
    pub inspector_bg: Color32,
    pub scene_bg: Color32,
    pub toolbar_bg: Color32,
    pub selected: Color32,
    pub selected_hover: Color32,
}

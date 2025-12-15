/// Debug Draw System - Unity/Unreal style debug visualization
/// 
/// Provides functions to draw debug lines, rays, boxes, etc. in the scene view
/// Similar to Unity's Debug.DrawLine() and Gizmos, or Unreal's DrawDebugLine()

use egui;

#[derive(Clone, Debug)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: egui::Color32,
    pub duration: f32, // How long to show (0 = one frame)
}

#[derive(Clone, Debug)]
pub struct DebugRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub length: f32,
    pub color: egui::Color32,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub struct DebugBox {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: egui::Color32,
    pub duration: f32,
}

/// Debug Draw Manager - stores and renders debug primitives
pub struct DebugDrawManager {
    lines: Vec<(DebugLine, f32)>, // (line, time_remaining)
    rays: Vec<(DebugRay, f32)>,
    boxes: Vec<(DebugBox, f32)>,
}

impl Default for DebugDrawManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugDrawManager {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            rays: Vec::new(),
            boxes: Vec::new(),
        }
    }

    /// Draw a line (Unity-style)
    pub fn draw_line(&mut self, start: [f32; 3], end: [f32; 3], color: egui::Color32, duration: f32) {
        self.lines.push((DebugLine { start, end, color, duration }, duration));
    }

    /// Draw a ray (Unity-style)
    pub fn draw_ray(&mut self, origin: [f32; 3], direction: [f32; 3], length: f32, color: egui::Color32, duration: f32) {
        self.rays.push((DebugRay { origin, direction, length, color, duration }, duration));
    }

    /// Draw a box (Unity-style)
    pub fn draw_box(&mut self, center: [f32; 3], size: [f32; 3], color: egui::Color32, duration: f32) {
        self.boxes.push((DebugBox { center, size, color, duration }, duration));
    }

    /// Update - remove expired debug draws
    pub fn update(&mut self, dt: f32) {
        // Update lines
        self.lines.retain_mut(|(_, time_remaining)| {
            *time_remaining -= dt;
            *time_remaining > 0.0
        });

        // Update rays
        self.rays.retain_mut(|(_, time_remaining)| {
            *time_remaining -= dt;
            *time_remaining > 0.0
        });

        // Update boxes
        self.boxes.retain_mut(|(_, time_remaining)| {
            *time_remaining -= dt;
            *time_remaining > 0.0
        });
    }

    /// Clear all debug draws
    pub fn clear(&mut self) {
        self.lines.clear();
        self.rays.clear();
        self.boxes.clear();
    }

    /// Render debug draws in scene view
    pub fn render(
        &self,
        painter: &egui::Painter,
        camera_pos: [f32; 3],
        zoom: f32,
        viewport_rect: egui::Rect,
    ) {
        // Render lines
        for (line, _) in &self.lines {
            let start_screen = world_to_screen(line.start, camera_pos, zoom, viewport_rect);
            let end_screen = world_to_screen(line.end, camera_pos, zoom, viewport_rect);
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(2.0, line.color),
            );
        }

        // Render rays
        for (ray, _) in &self.rays {
            let end = [
                ray.origin[0] + ray.direction[0] * ray.length,
                ray.origin[1] + ray.direction[1] * ray.length,
                ray.origin[2] + ray.direction[2] * ray.length,
            ];
            
            let start_screen = world_to_screen(ray.origin, camera_pos, zoom, viewport_rect);
            let end_screen = world_to_screen(end, camera_pos, zoom, viewport_rect);
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(2.0, ray.color),
            );
            
            // Draw arrow head
            let arrow_size = 5.0;
            let dir = egui::vec2(end_screen.x - start_screen.x, end_screen.y - start_screen.y).normalized();
            let perp = egui::vec2(-dir.y, dir.x);
            
            let arrow_tip = end_screen;
            let arrow_left = arrow_tip - dir * arrow_size + perp * arrow_size * 0.5;
            let arrow_right = arrow_tip - dir * arrow_size - perp * arrow_size * 0.5;
            
            painter.line_segment([arrow_tip, arrow_left], egui::Stroke::new(2.0, ray.color));
            painter.line_segment([arrow_tip, arrow_right], egui::Stroke::new(2.0, ray.color));
        }

        // Render boxes
        for (debug_box, _) in &self.boxes {
            let half_size = [
                debug_box.size[0] / 2.0,
                debug_box.size[1] / 2.0,
                debug_box.size[2] / 2.0,
            ];
            
            // Draw box corners
            let corners = [
                [debug_box.center[0] - half_size[0], debug_box.center[1] - half_size[1], debug_box.center[2]],
                [debug_box.center[0] + half_size[0], debug_box.center[1] - half_size[1], debug_box.center[2]],
                [debug_box.center[0] + half_size[0], debug_box.center[1] + half_size[1], debug_box.center[2]],
                [debug_box.center[0] - half_size[0], debug_box.center[1] + half_size[1], debug_box.center[2]],
            ];
            
            // Draw box edges
            for i in 0..4 {
                let start = corners[i];
                let end = corners[(i + 1) % 4];
                
                let start_screen = world_to_screen(start, camera_pos, zoom, viewport_rect);
                let end_screen = world_to_screen(end, camera_pos, zoom, viewport_rect);
                
                painter.line_segment(
                    [start_screen, end_screen],
                    egui::Stroke::new(2.0, debug_box.color),
                );
            }
        }
    }

    /// Get number of active debug draws
    pub fn count(&self) -> usize {
        self.lines.len() + self.rays.len() + self.boxes.len()
    }
}

/// Convert world position to screen position
fn world_to_screen(
    world_pos: [f32; 3],
    camera_pos: [f32; 3],
    zoom: f32,
    viewport_rect: egui::Rect,
) -> egui::Pos2 {
    // Calculate relative position to camera
    let rel_x = world_pos[0] - camera_pos[0];
    let rel_y = world_pos[1] - camera_pos[1];
    
    // Apply zoom and convert to screen space
    let screen_x = viewport_rect.center().x + rel_x * zoom;
    let screen_y = viewport_rect.center().y - rel_y * zoom; // Flip Y
    
    egui::pos2(screen_x, screen_y)
}

/// Helper functions for common colors (Unity-style)
impl DebugDrawManager {
    /// Draw green line (success/hit)
    pub fn draw_line_green(&mut self, start: [f32; 3], end: [f32; 3], duration: f32) {
        self.draw_line(start, end, egui::Color32::GREEN, duration);
    }

    /// Draw red line (failure/miss)
    pub fn draw_line_red(&mut self, start: [f32; 3], end: [f32; 3], duration: f32) {
        self.draw_line(start, end, egui::Color32::RED, duration);
    }

    /// Draw yellow line (warning)
    pub fn draw_line_yellow(&mut self, start: [f32; 3], end: [f32; 3], duration: f32) {
        self.draw_line(start, end, egui::Color32::YELLOW, duration);
    }

    /// Draw blue line (info)
    pub fn draw_line_blue(&mut self, start: [f32; 3], end: [f32; 3], duration: f32) {
        self.draw_line(start, end, egui::Color32::BLUE, duration);
    }

    /// Draw green ray (success/hit)
    pub fn draw_ray_green(&mut self, origin: [f32; 3], direction: [f32; 3], length: f32, duration: f32) {
        self.draw_ray(origin, direction, length, egui::Color32::GREEN, duration);
    }

    /// Draw red ray (failure/miss)
    pub fn draw_ray_red(&mut self, origin: [f32; 3], direction: [f32; 3], length: f32, duration: f32) {
        self.draw_ray(origin, direction, length, egui::Color32::RED, duration);
    }
}

use egui::{self, Color32, RichText};
use ecs::World;
use crate::editor::map_manager::MapManager;

/// Performance monitoring panel for tilemap management
pub struct PerformancePanel {
    /// Performance thresholds for warnings
    pub thresholds: PerformanceThresholds,
}

/// Performance warning thresholds
#[derive(Clone)]
pub struct PerformanceThresholds {
    /// Maximum draw calls before warning
    pub max_draw_calls: usize,
    
    /// Maximum triangles before warning
    pub max_triangles: usize,
    
    /// Maximum vertices before warning
    pub max_vertices: usize,
    
    /// Maximum memory usage in MB before warning
    pub max_memory_mb: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_draw_calls: 1000,
            max_triangles: 100_000,
            max_vertices: 200_000,
            max_memory_mb: 512.0,
        }
    }
}

impl PerformancePanel {
    /// Create a new PerformancePanel with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: PerformanceThresholds::default(),
        }
    }
    
    /// Render the performance panel as a standalone window
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &World,
        map_manager: &MapManager,
        open: &mut bool,
    ) {
        egui::Window::new("📊 Performance")
            .open(open)
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui, world, map_manager);
            });
    }
    
    /// Render the performance panel content (for use in docking system)
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &World,
        map_manager: &MapManager,
    ) {
        // Calculate performance metrics
        let metrics = self.calculate_metrics(world, map_manager);
        
        ui.heading("Performance Metrics");
        ui.separator();
        
        // Rendering metrics section
        ui.collapsing(RichText::new("🎨 Rendering").strong(), |ui| {
            self.render_metric(
                ui,
                "Draw Calls",
                metrics.draw_calls,
                self.thresholds.max_draw_calls,
                "",
            );
            
            self.render_metric(
                ui,
                "Triangles",
                metrics.triangles,
                self.thresholds.max_triangles,
                "",
            );
            
            self.render_metric(
                ui,
                "Vertices",
                metrics.vertices,
                self.thresholds.max_vertices,
                "",
            );
        });
        
        ui.separator();
        
        // Memory metrics section
        ui.collapsing(RichText::new("💾 Memory").strong(), |ui| {
            self.render_memory_metric(
                ui,
                "Tilemap Data",
                metrics.tilemap_memory_mb,
                self.thresholds.max_memory_mb * 0.3, // 30% of total
            );
            
            self.render_memory_metric(
                ui,
                "Texture Memory",
                metrics.texture_memory_mb,
                self.thresholds.max_memory_mb * 0.5, // 50% of total
            );
            
            self.render_memory_metric(
                ui,
                "Collider Memory",
                metrics.collider_memory_mb,
                self.thresholds.max_memory_mb * 0.2, // 20% of total
            );
            
            ui.separator();
            
            self.render_memory_metric(
                ui,
                "Total Memory",
                metrics.total_memory_mb,
                self.thresholds.max_memory_mb,
            );
        });
        
        ui.separator();
        
        // Entity counts section
        ui.collapsing(RichText::new("📦 Entity Counts").strong(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Loaded Maps:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", metrics.loaded_maps)).strong());
                });
            });
            
            ui.horizontal(|ui| {
                ui.label("Tilemap Layers:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", metrics.tilemap_count)).strong());
                });
            });
            
            ui.horizontal(|ui| {
                ui.label("Colliders:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", metrics.collider_count)).strong());
                });
            });
            
            ui.horizontal(|ui| {
                ui.label("Total Entities:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", metrics.total_entities)).strong());
                });
            });
        });
        
        ui.separator();
        
        // Threshold settings section
        ui.collapsing(RichText::new("⚙️ Warning Thresholds").strong(), |ui| {
            ui.label(RichText::new("Adjust warning thresholds:").italics());
            
            ui.horizontal(|ui| {
                ui.label("Draw Calls:");
                ui.add(egui::DragValue::new(&mut self.thresholds.max_draw_calls)
                    .speed(10)
                    .clamp_range(100..=10000));
            });
            
            ui.horizontal(|ui| {
                ui.label("Triangles:");
                ui.add(egui::DragValue::new(&mut self.thresholds.max_triangles)
                    .speed(1000)
                    .clamp_range(10000..=1000000));
            });
            
            ui.horizontal(|ui| {
                ui.label("Vertices:");
                ui.add(egui::DragValue::new(&mut self.thresholds.max_vertices)
                    .speed(1000)
                    .clamp_range(10000..=1000000));
            });
            
            ui.horizontal(|ui| {
                ui.label("Memory (MB):");
                ui.add(egui::DragValue::new(&mut self.thresholds.max_memory_mb)
                    .speed(10.0)
                    .clamp_range(64.0..=4096.0));
            });
            
            if ui.button("Reset to Defaults").clicked() {
                self.thresholds = PerformanceThresholds::default();
            }
        });
    }
    
    /// Render a single metric with threshold warning
    fn render_metric(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        value: usize,
        threshold: usize,
        suffix: &str,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show warning indicator if threshold exceeded
                if value > threshold {
                    ui.label(RichText::new("⚠").color(Color32::from_rgb(255, 200, 0)))
                        .on_hover_text(format!("Exceeds threshold of {}{}", threshold, suffix));
                }
                
                let text = if value > threshold {
                    RichText::new(format!("{}{}", value, suffix))
                        .color(Color32::from_rgb(255, 150, 0))
                        .strong()
                } else {
                    RichText::new(format!("{}{}", value, suffix))
                };
                
                ui.label(text);
            });
        });
    }
    
    /// Render a memory metric with threshold warning
    fn render_memory_metric(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        value_mb: f32,
        threshold_mb: f32,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show warning indicator if threshold exceeded
                if value_mb > threshold_mb {
                    ui.label(RichText::new("⚠").color(Color32::from_rgb(255, 200, 0)))
                        .on_hover_text(format!("Exceeds threshold of {:.2} MB", threshold_mb));
                }
                
                let text = if value_mb > threshold_mb {
                    RichText::new(format!("{:.2} MB", value_mb))
                        .color(Color32::from_rgb(255, 150, 0))
                        .strong()
                } else {
                    RichText::new(format!("{:.2} MB", value_mb))
                };
                
                ui.label(text);
            });
        });
    }
    
    /// Calculate performance metrics from world and map manager
    pub fn calculate_metrics(&self, world: &World, map_manager: &MapManager) -> PerformanceMetrics {
        let loaded_maps = map_manager.loaded_maps.len();
        let tilemap_count = world.tilemaps.len();
        let collider_count = world.colliders.len();
        let total_entities = world.transforms.len();
        
        // Calculate draw calls (one per tilemap layer)
        let draw_calls = tilemap_count;
        
        // Calculate triangles and vertices
        // Each tile is 2 triangles (6 vertices)
        let mut total_tiles = 0;
        for tilemap in world.tilemaps.values() {
            total_tiles += tilemap.tiles.len();
        }
        
        let triangles = total_tiles * 2;
        let vertices = total_tiles * 6;
        
        // Estimate memory usage
        // Tilemap data: ~100 bytes per tile (rough estimate)
        let tilemap_memory_mb = (total_tiles * 100) as f32 / (1024.0 * 1024.0);
        
        // Texture memory: estimate based on tilesets
        // Assume average 512x512 RGBA texture = 1MB per tileset
        let unique_tilesets = world.tilesets.len();
        let texture_memory_mb = unique_tilesets as f32 * 1.0;
        
        // Collider memory: ~50 bytes per collider (rough estimate)
        let collider_memory_mb = (collider_count * 50) as f32 / (1024.0 * 1024.0);
        
        let total_memory_mb = tilemap_memory_mb + texture_memory_mb + collider_memory_mb;
        
        PerformanceMetrics {
            loaded_maps,
            tilemap_count,
            collider_count,
            total_entities,
            draw_calls,
            triangles,
            vertices,
            tilemap_memory_mb,
            texture_memory_mb,
            collider_memory_mb,
            total_memory_mb,
        }
    }
}

impl Default for PerformancePanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics calculated from world state
pub struct PerformanceMetrics {
    pub loaded_maps: usize,
    pub tilemap_count: usize,
    pub collider_count: usize,
    pub total_entities: usize,
    pub draw_calls: usize,
    pub triangles: usize,
    pub vertices: usize,
    pub tilemap_memory_mb: f32,
    pub texture_memory_mb: f32,
    pub collider_memory_mb: f32,
    pub total_memory_mb: f32,
}

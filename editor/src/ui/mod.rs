// Sub-modules
pub mod resource_manager;
pub mod menu_bar;
pub mod inspector;
pub mod texture_inspector;
pub mod scene_view;
pub mod project_settings;
pub mod dock_layout;
pub mod camera_settings;
pub mod sprite_picker;
pub mod map_inspector;
pub mod map_view;
pub mod create_prefab_dialog;
pub mod export_dialog;
pub mod dialogs;
pub mod launcher_window;
pub mod game_window;
pub mod panels;

// Re-exports
use ecs::{World, Entity, EntityTag};
use egui;
use std::collections::HashMap;
use crate::{Console, SceneCamera, SceneGrid};
pub use dock_layout::{
    EditorTab, TabContext, EditorTabViewer, 
    create_default_layout,
    get_dock_style, save_default_layout, load_default_layout_name, get_layout_by_name, load_custom_layouts, save_custom_layout_state, load_custom_layout_state
};
use panels::{hierarchy, bottom_panel};
use engine_core::assets::AssetLoader;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformTool {
    View,   // Q - No gizmo, just view
    Move,   // W - Move gizmo
    Rotate, // E - Rotation gizmo
    Scale,  // R - Scale gizmo
}

pub struct EditorUI;

impl EditorUI {

    pub fn render_editor(
        ctx: &egui::Context,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        entity_names: &mut HashMap<Entity, String>,
        save_request: &mut bool,
        save_as_request: &mut bool,
        load_request: &mut bool,
        load_file_request: &mut Option<std::path::PathBuf>,
        new_scene_request: &mut bool,
        play_request: &mut bool,
        stop_request: &mut bool,
        edit_script_request: &mut Option<String>,
        project_path: &Option<std::path::PathBuf>,
        current_scene_path: &Option<std::path::PathBuf>,
        scene_view_tab: &mut usize,
        is_playing: bool,
        show_colliders: &mut bool,
        show_velocities: &mut bool,
        console: &mut Console,
        bottom_panel_tab: &mut usize,
        current_tool: &mut TransformTool,
        show_project_settings: &mut bool,
        scene_camera: &mut SceneCamera,
        scene_grid: &SceneGrid,
        infinite_grid: &mut crate::grid::InfiniteGrid,
        camera_state_display: &crate::ui::camera_settings::CameraStateDisplay,
        show_exit_dialog: &mut bool,
        show_export_dialog: &mut bool,
        asset_manager: &mut Option<crate::AssetManager>,
        drag_drop: &mut crate::DragDropState,
        _layout_request: &mut Option<String>,
        texture_manager: &mut engine::texture_manager::TextureManager,
        open_sprite_editor_request: &mut Option<std::path::PathBuf>,
        sprite_picker_state: &mut sprite_picker::SpritePickerState,
        show_debug_lines: &mut bool,
        scene_view_renderer: &mut crate::scene_view_renderer::SceneViewRenderer,
        egui_renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        reload_mesh_assets_request: &mut bool,
        asset_loader: &dyn AssetLoader,
        render_cache: &mut engine::runtime::render_system::RenderCache,
    ) {
        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
             let mut dummy_layout_request = None;
             menu_bar::render_menu_bar(
                ui,
                world,
                entity_names,
                new_scene_request,
                save_request,
                save_as_request,
                load_request,
                load_file_request,
                play_request,
                stop_request,
                show_project_settings,
                show_colliders,
                show_velocities,
                show_debug_lines,
                project_path,
                current_scene_path,
                is_playing,
                show_exit_dialog,
                show_export_dialog,
                &mut dummy_layout_request,
                "legacy", 
                Self::get_scene_files,
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
             ui.vertical_centered(|ui| {
                 ui.add_space(50.0);
                 ui.heading("Legacy Editor Mode");
                 ui.label("This mode is deprecated. Please enable Docking in settings.");
             });
        });
    }

    /// Get all .scene files in project scenes folder
    fn get_scene_files(project_path: &std::path::Path) -> Vec<String> {
        let scenes_folder = project_path.join("scenes");
        let mut scene_files = Vec::new();

        if scenes_folder.exists() {
            if let Ok(entries) = std::fs::read_dir(&scenes_folder) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(path) = entry.path().to_str() {
                                if path.ends_with(".scene") {
                                    // Get relative path from project root
                                    if let Ok(relative) = entry.path().strip_prefix(project_path) {
                                        scene_files.push(relative.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        scene_files.sort();
        scene_files
    }

    /// Get icon for entity based on its components
    fn get_entity_icon(world: &World, entity: Entity) -> &'static str {
        // Check for specific entity types
        if let Some(tag) = world.tags.get(&entity) {
            return match tag {
                EntityTag::Player => "🎮",
                EntityTag::Item => "💎",
            };
        }

        // Check for components
        let has_sprite = world.sprites.contains_key(&entity);
        let has_collider = world.colliders.contains_key(&entity);
        let has_velocity = world.velocities.contains_key(&entity);
        let has_script = world.scripts.contains_key(&entity);

        // Determine icon based on component combination
        if has_script {
            "📜" // Script
        } else if has_velocity && has_collider {
            "🏃" // Physics object (moving + collision)
        } else if has_sprite && has_collider {
            "📦" // Sprite with collision
        } else if has_sprite {
            "🖼️" // Sprite only
        } else if has_collider {
            "⬜" // Collider only (invisible)
        } else {
            "📍" // Empty GameObject
        }
    }

    /// Render editor with docking system (Unity-like)
    pub fn render_editor_with_dock(
        ctx: &egui::Context,
        dock_state: &mut egui_dock::DockState<EditorTab>,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        entity_names: &mut HashMap<Entity, String>,
        save_request: &mut bool,
        save_as_request: &mut bool,
        load_request: &mut bool,
        load_file_request: &mut Option<std::path::PathBuf>,
        new_scene_request: &mut bool,
        play_request: &mut bool,
        stop_request: &mut bool,
        edit_script_request: &mut Option<String>,
        project_path: &Option<std::path::PathBuf>,
        current_scene_path: &Option<std::path::PathBuf>,
        scene_view_tab: &mut usize,
        is_playing: bool,
        show_colliders: &mut bool,
        show_velocities: &mut bool,
        console: &mut Console,
        _bottom_panel_tab: &mut usize,
        current_tool: &mut TransformTool,
        show_project_settings: &mut bool,
        scene_camera: &mut SceneCamera,
        scene_grid: &SceneGrid,
        infinite_grid: &mut crate::grid::InfiniteGrid,
        // NEW: Unity-like editor features
        camera_state_display: &crate::ui::camera_settings::CameraStateDisplay,
        show_exit_dialog: &mut bool,
        show_export_dialog: &mut bool,
        asset_manager: &mut Option<crate::AssetManager>,
        drag_drop: &mut crate::DragDropState,
        layout_request: &mut Option<String>,
        current_layout_name: &str,
        dragging_entity: &mut Option<Entity>,
        drag_axis: &mut Option<u8>,
        scene_view_mode: &mut scene_view::SceneViewMode,
        projection_mode: &mut scene_view::SceneProjectionMode,
        transform_space: &mut scene_view::TransformSpace,
        game_view_renderer: &mut crate::game_view_renderer::GameViewRenderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        egui_renderer: &mut egui_wgpu::Renderer,
        scene_view_renderer: &mut crate::scene_view_renderer::SceneViewRenderer,
        texture_manager: &mut engine::texture_manager::TextureManager,
        open_sprite_editor_request: &mut Option<std::path::PathBuf>,
        open_prefab_editor_request: &mut Option<std::path::PathBuf>,
        sprite_editor_windows: &mut Vec<crate::SpriteEditorWindow>,
        sprite_picker_state: &mut sprite_picker::SpritePickerState,
        texture_inspector: &mut texture_inspector::TextureInspector,
        map_view_state: &mut map_view::MapViewState,
        show_debug_lines: &mut bool,
        debug_draw: &mut crate::debug_draw::DebugDrawManager,
        map_manager: &mut crate::map_manager::MapManager,
        prefab_manager: &mut crate::PrefabManager,
        create_prefab_dialog: &mut create_prefab_dialog::CreatePrefabDialog,
        layer_properties_panel: &mut panels::layer_properties_panel::LayerPropertiesPanel,
        layer_ordering_panel: &mut panels::layer_ordering_panel::LayerOrderingPanel,
        performance_panel: &mut panels::performance_panel::PerformancePanel,
        collider_settings_panel: &mut panels::collider_settings_panel::ColliderSettingsPanel,
        game_view_settings: &mut engine::runtime::GameViewSettings,
        prefab_editor: &mut crate::PrefabEditor,
        ui_manager: &mut engine::ui_manager::UIManager,
        dt: f32,
        reload_mesh_assets_request: &mut bool,
        asset_loader: &dyn AssetLoader,
        render_cache: &mut engine::runtime::render_system::RenderCache,
    ) {
        // Handle layout change request (will be processed by caller)
        // Layout changes are handled in main.rs to access EditorState

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu_bar::render_menu_bar(
                ui,
                world,
                entity_names,
                new_scene_request,
                save_request,
                save_as_request,
                load_request,
                load_file_request,
                play_request,
                stop_request,
                show_project_settings,
                show_colliders,
                show_velocities,
                show_debug_lines,
                project_path,
                current_scene_path,
                is_playing,
                show_exit_dialog,
                show_export_dialog,
                layout_request,
                current_layout_name,
                Self::get_scene_files,
            );
        });

        // Main docking area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tab_context = TabContext {
                world,
                selected_entity,
                entity_names,
                edit_script_request,
                project_path,
                current_scene_path,
                load_file_request,
                console,
                scene_view_tab,
                map_view_state,
                is_playing,
                show_colliders,
                show_velocities,
                current_tool,
                scene_camera,
                scene_grid,
                infinite_grid,
                camera_state_display,
                delta_time: dt,
                play_request,
                stop_request,
                asset_manager,
                drag_drop,
                dragging_entity,
                drag_axis,
                scene_view_mode,
                projection_mode,
                transform_space,
                texture_manager,
                open_sprite_editor_request,
                open_prefab_editor_request,
                sprite_editor_windows,
                sprite_picker_state,
                texture_inspector,
                show_debug_lines,
                debug_draw,
                map_manager,
                prefab_manager,
                create_prefab_dialog,
                layer_properties_panel,
                layer_ordering_panel,
                performance_panel,
                collider_settings_panel,
                game_view_settings,
                prefab_editor,
                ui_manager,
                dt,
                game_view_renderer,
                device,
                queue,
                reload_mesh_assets_request,
                egui_renderer,
                scene_view_renderer,
                asset_loader,
                render_cache,
            };

            // Handle Layout Requests
            if let Some(request) = layout_request.take() {
                if let Some(proj_path) = project_path {
                    if request == "load:default" {
                        *dock_state = dock_layout::create_default_layout();
                        // *current_layout_name = "default".to_string(); // current_layout_name is &str, immutable
                    } else if request == "load:2column" {
                        *dock_state = dock_layout::create_2_column_layout();
                    } else if request == "load:tall" {
                        *dock_state = dock_layout::create_tall_layout();
                    } else if request == "load:wide" {
                        *dock_state = dock_layout::create_wide_layout();
                    } else if request == "save_as" {
                         let name = format!("Layout_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
                         if let Ok(_) = dock_layout::save_custom_layout_state(&name, dock_state, proj_path) {
                             // Success
                         }
                    } else if request == "save_default" {
                        let _ = dock_layout::save_default_layout(current_layout_name, proj_path);
                    } else if request.starts_with("custom:") {
                        let name = request.trim_start_matches("custom:");
                        if let Some(state) = dock_layout::load_custom_layout_state(name, proj_path) {
                            *dock_state = state;
                        }
                    }
                }
            }
    
            let mut tab_viewer = EditorTabViewer {
                context: &mut tab_context,
            };
    
            let available_size = ui.available_size();
            if available_size.x > 0.0 && available_size.y > 0.0 {
                egui_dock::DockArea::new(dock_state)
                    .style(get_dock_style())
                    .show_inside(ui, &mut tab_viewer);
            }
        });

        // Project Settings Dialog
        project_settings::render_project_settings(
            ctx,
            show_project_settings,
            project_path,
            Self::get_scene_files,
        );
    }
}

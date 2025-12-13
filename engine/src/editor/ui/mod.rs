// Sub-modules
pub mod resource_manager;
pub mod menu_bar;
pub mod hierarchy;
pub mod inspector;
pub mod texture_inspector;
pub mod scene_view;
pub mod bottom_panel;
pub mod project_settings;
pub mod asset_browser;
pub mod dock_layout;
pub mod camera_settings;
pub mod sprite_picker;
pub mod map_inspector;
pub mod map_view;
pub mod maps_panel;
pub mod layer_properties_panel;
pub mod layer_ordering_panel;
pub mod performance_panel;
pub mod collider_settings_panel;
pub mod prefabs_panel;
pub mod create_prefab_dialog;

// Re-exports
use ecs::{World, Entity, EntityTag};
use egui;
use std::collections::HashMap;
use crate::editor::{Console, SceneCamera, SceneGrid};
pub use dock_layout::{
    EditorTab, TabContext, EditorTabViewer, 
    create_default_layout,
    get_dock_style, save_default_layout, load_default_layout_name, get_layout_by_name, load_custom_layouts, save_custom_layout_state, load_custom_layout_state
};

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
        infinite_grid: &mut crate::editor::grid::InfiniteGrid,
        camera_state_display: &crate::editor::camera::CameraStateDisplay,
        show_exit_dialog: &mut bool,
        asset_manager: &mut Option<crate::editor::AssetManager>,
        drag_drop: &mut crate::editor::DragDropState,
        _layout_request: &mut Option<String>,
        texture_manager: &mut crate::texture_manager::TextureManager,
        open_sprite_editor_request: &mut Option<std::path::PathBuf>,
        sprite_picker_state: &mut sprite_picker::SpritePickerState,
        show_debug_lines: &mut bool,
    ) {
        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let mut dummy_layout = None;
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
                &mut dummy_layout,
                "default",
                Self::get_scene_files,
            );
        });

        // Hierarchy Panel (Left)
        egui::SidePanel::left("hierarchy").min_width(200.0).show(ctx, |ui| {
            // Note: This is the old non-dock layout. Map filtering is not available here
            // as MapManager is not passed to this function. Use the dock layout for full features.
            hierarchy::render_hierarchy(
                ui,
                world,
                entity_names,
                selected_entity,
                load_file_request,
                project_path,
                current_scene_path,
                console,
                Self::get_scene_files,
                &Self::get_entity_icon,
            );
        });

        // Inspector Panel (Right)
        egui::SidePanel::right("inspector").min_width(300.0).show(ctx, |ui| {
            inspector::render_inspector(
                ui,
                world,
                selected_entity,
                entity_names,
                edit_script_request,
                project_path,
                open_sprite_editor_request,
                sprite_picker_state,
            );
        });

        // Center Panel - Scene/Game View
        egui::CentralPanel::default().show(ctx, |ui| {
            // Update camera with delta time for smooth interpolation
            let delta_time = ui.input(|i| i.stable_dt);
            
            // Dummy drag state for old layout (not used)
            let mut dummy_dragging_entity = None;
            let mut dummy_drag_axis = None;
            let mut dummy_scene_view_mode = scene_view::SceneViewMode::Mode2D;
            let mut dummy_projection_mode = scene_view::SceneProjectionMode::Perspective;
            let mut dummy_transform_space = scene_view::TransformSpace::Local;
            
            let mut dummy_debug_draw = crate::editor::debug_draw::DebugDrawManager::new();
            let dummy_map_manager = crate::editor::map_manager::MapManager::new();
            
            scene_view::render_scene_view(
                ui,
                world,
                selected_entity,
                scene_view_tab,
                is_playing,
                show_colliders,
                show_velocities,
                show_debug_lines,
                &mut dummy_debug_draw,
                current_tool,
                scene_camera,
                scene_grid,
                infinite_grid,
                camera_state_display,
                play_request,
                stop_request,
                &mut dummy_dragging_entity,
                &mut dummy_drag_axis,
                &mut dummy_scene_view_mode,
                &mut dummy_projection_mode,
                &mut dummy_transform_space,
                texture_manager,
                drag_drop,
                delta_time,
                &dummy_map_manager,
            );
        });

        // Bottom Panel - Assets & Console
        egui::TopBottomPanel::bottom("bottom_panel").min_height(280.0).show(ctx, |ui| {
            bottom_panel::render_bottom_panel(
                ui,
                bottom_panel_tab,
                asset_manager,
                console,
                drag_drop,
                texture_manager,
                project_path.as_ref(),
            );
        });

        // Project Settings Dialog
        project_settings::render_project_settings(
            ctx,
            show_project_settings,
            project_path,
            Self::get_scene_files,
        );
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
        infinite_grid: &mut crate::editor::grid::InfiniteGrid,
        camera_state_display: &crate::editor::camera::CameraStateDisplay,
        show_exit_dialog: &mut bool,
        asset_manager: &mut Option<crate::editor::AssetManager>,
        drag_drop: &mut crate::editor::DragDropState,
        layout_request: &mut Option<String>,
        current_layout_name: &str,
        dragging_entity: &mut Option<Entity>,
        drag_axis: &mut Option<u8>,
        scene_view_mode: &mut scene_view::SceneViewMode,
        projection_mode: &mut scene_view::SceneProjectionMode,
        transform_space: &mut scene_view::TransformSpace,
        texture_manager: &mut crate::texture_manager::TextureManager,
        open_sprite_editor_request: &mut Option<std::path::PathBuf>,
        open_prefab_editor_request: &mut Option<std::path::PathBuf>,
        sprite_editor_windows: &mut Vec<crate::editor::SpriteEditorWindow>,
        sprite_picker_state: &mut sprite_picker::SpritePickerState,
        texture_inspector: &mut texture_inspector::TextureInspector,
        map_view_state: &mut map_view::MapViewState,
        show_debug_lines: &mut bool,
        debug_draw: &mut crate::editor::debug_draw::DebugDrawManager,
        map_manager: &mut crate::editor::map_manager::MapManager,
        prefab_manager: &mut crate::editor::PrefabManager,
        create_prefab_dialog: &mut create_prefab_dialog::CreatePrefabDialog,
        layer_properties_panel: &mut layer_properties_panel::LayerPropertiesPanel,
        layer_ordering_panel: &mut layer_ordering_panel::LayerOrderingPanel,
        performance_panel: &mut performance_panel::PerformancePanel,
        collider_settings_panel: &mut collider_settings_panel::ColliderSettingsPanel,
        game_view_settings: &mut crate::runtime::GameViewSettings,
        prefab_editor: &mut crate::editor::PrefabEditor,
        ui_manager: &mut crate::ui_manager::UIManager,
        dt: f32,
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
            };

            let mut tab_viewer = EditorTabViewer {
                context: &mut tab_context,
            };

            egui_dock::DockArea::new(dock_state)
                .style(get_dock_style())
                .show_inside(ui, &mut tab_viewer);
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

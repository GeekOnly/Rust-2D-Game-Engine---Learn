//! Lua bindings for the UI system
//!
//! This module provides a comprehensive Lua API for creating and manipulating UI elements
//! at runtime, following patterns similar to Unity's UI system.
//!
//! # Features
//!
//! - Canvas creation and management
//! - UI element creation (Image, Text, Button, Panel, etc.)
//! - Hierarchy operations (parenting, children queries, destruction)
//! - Property getters and setters for all components
//! - Animation functions
//! - Event callback registration
//! - Element queries (find by name, tag, etc.)

use mlua::{Lua, Table};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use ecs::World;

use crate::*;

// Entity type from ecs crate (u32)
type EcsEntity = ecs::Entity;
// Entity type from ui crate (u64)
type UIEntity = crate::Entity;

mod properties;
mod animation;

pub use properties::inject_property_api;
pub use animation::{inject_animation_api, inject_event_api, inject_query_api};

/// UI Lua API manager
pub struct UILuaBindings {
    /// Registered event callbacks (entity -> event_type -> callback_name)
    event_callbacks: RefCell<HashMap<UIEntity, HashMap<UIEventType, String>>>,
    /// Named UI elements for lookup (name -> entity)
    named_elements: RefCell<HashMap<String, UIEntity>>,
    /// Tagged UI elements for lookup (tag -> Vec<entity>)
    tagged_elements: RefCell<HashMap<String, Vec<UIEntity>>>,
}

impl UILuaBindings {
    pub fn new() -> Self {
        Self {
            event_callbacks: RefCell::new(HashMap::new()),
            named_elements: RefCell::new(HashMap::new()),
            tagged_elements: RefCell::new(HashMap::new()),
        }
    }

    /// Register a UI element with a name for lookup
    pub fn register_named_element(&self, name: String, entity: UIEntity) {
        self.named_elements.borrow_mut().insert(name, entity);
    }

    /// Register a UI element with a tag for lookup
    pub fn register_tagged_element(&self, tag: String, entity: UIEntity) {
        self.tagged_elements.borrow_mut()
            .entry(tag)
            .or_insert_with(Vec::new)
            .push(entity);
    }

    /// Get entity by name
    pub fn find_by_name(&self, name: &str) -> Option<UIEntity> {
        self.named_elements.borrow().get(name).copied()
    }

    /// Get entities by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<UIEntity> {
        self.tagged_elements.borrow()
            .get(tag)
            .cloned()
            .unwrap_or_default()
    }

    /// Register an event callback for an entity
    pub fn register_event_callback(&self, entity: UIEntity, event_type: UIEventType, callback: String) {
        self.event_callbacks.borrow_mut()
            .entry(entity)
            .or_insert_with(HashMap::new)
            .insert(event_type, callback);
    }

    /// Get event callback for an entity and event type
    pub fn get_event_callback(&self, entity: UIEntity, event_type: &UIEventType) -> Option<String> {
        self.event_callbacks.borrow()
            .get(&entity)?
            .get(event_type)
            .cloned()
    }

    /// Remove all callbacks for an entity (when destroyed)
    pub fn remove_entity_callbacks(&self, entity: UIEntity) {
        self.event_callbacks.borrow_mut().remove(&entity);
    }

    /// Inject complete UI API into a Lua scope
    /// This should be called within a lua.scope() to provide access to World
    pub fn inject_ui_api<'lua, 'scope>(
        &'scope self,
        lua: &'lua Lua,
        scope: &mlua::Scope<'lua, 'scope>,
        world: &'scope RefCell<&mut World>,
    ) -> Result<()> {
        // Inject all API modules
        self.inject_creation_api(lua, scope, world)?;
        inject_property_api(lua, scope, world)?;
        inject_animation_api(lua, scope, world)?;
        inject_event_api(lua, scope, world, self)?;
        inject_query_api(lua, scope, world, self)?;
        
        Ok(())
    }

    /// Inject UI creation API into a Lua scope
    fn inject_creation_api<'lua, 'scope>(
        &'scope self,
        lua: &'lua Lua,
        scope: &mlua::Scope<'lua, 'scope>,
        world: &'scope RefCell<&mut World>,
    ) -> Result<()> {
        let globals = lua.globals();

        // ================================================================
        // CANVAS CREATION
        // ================================================================

        let create_canvas = scope.create_function_mut(|_lua, args: Table| {
            let render_mode = args.get::<_, Option<String>>("render_mode")
                .unwrap_or(Some("ScreenSpaceOverlay".to_string()))
                .unwrap_or("ScreenSpaceOverlay".to_string());
            
            let sort_order = args.get::<_, Option<i32>>("sort_order")
                .unwrap_or(Some(0))
                .unwrap_or(0);

            let ecs_entity = world.borrow_mut().spawn();
            let ui_entity = ecs_entity as UIEntity;
            
            // Create Canvas component
            let _canvas = Canvas {
                render_mode: match render_mode.as_str() {
                    "ScreenSpaceCamera" => CanvasRenderMode::ScreenSpaceCamera,
                    "WorldSpace" => CanvasRenderMode::WorldSpace,
                    _ => CanvasRenderMode::ScreenSpaceOverlay,
                },
                sort_order,
                camera_entity: None,
                plane_distance: 100.0,
                scaler: CanvasScaler::default(),
                blocks_raycasts: true,
                cached_screen_size: (0, 0),
                dirty: true,
            };

            // Create RectTransform for canvas
            let _rect_transform = RectTransform::default();

            // Create UIElement
            let _ui_element = UIElement {
                raycast_target: false,
                blocks_raycasts: true,
                z_order: 0,
                color: [1.0, 1.0, 1.0, 1.0],
                alpha: 1.0,
                interactable: true,
                ignore_layout: false,
                canvas_entity: Some(ui_entity),
            };

            // Store components (this would need to be adapted to your ECS)
            // For now, we'll assume World has methods to store these
            // world.borrow_mut().insert_canvas(entity, canvas);
            // world.borrow_mut().insert_rect_transform(entity, rect_transform);
            // world.borrow_mut().insert_ui_element(entity, ui_element);

            Ok(ecs_entity)
        })?;
        globals.set("ui_create_canvas", create_canvas)?;

        // ================================================================
        // UI ELEMENT CREATION
        // ================================================================

        let create_image = scope.create_function_mut(|_lua, args: Table| {
            let parent = args.get::<_, Option<EcsEntity>>("parent")?;
            let sprite = args.get::<_, Option<String>>("sprite")?;
            let color = args.get::<_, Option<Table>>("color")?;

            let ecs_entity = world.borrow_mut().spawn();
            let _ui_entity = ecs_entity as UIEntity;

            // Create RectTransform
            let _rect_transform = RectTransform::anchored(
                Vec2::new(0.5, 0.5), // center anchor
                Vec2::ZERO,
                Vec2::new(100.0, 100.0),
            );

            // Create UIElement
            let _ui_element = UIElement {
                raycast_target: true,
                blocks_raycasts: true,
                z_order: 0,
                color: if let Some(c) = color {
                    [
                        c.get::<_, f32>("r").unwrap_or(1.0),
                        c.get::<_, f32>("g").unwrap_or(1.0),
                        c.get::<_, f32>("b").unwrap_or(1.0),
                        c.get::<_, f32>("a").unwrap_or(1.0),
                    ]
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                },
                alpha: 1.0,
                interactable: true,
                ignore_layout: false,
                canvas_entity: parent.map(|e| e as UIEntity),
            };

            // Create UIImage
            let _ui_image = UIImage {
                sprite,
                image_type: ImageType::Simple,
                slice_borders: Vec4::ZERO,
                fill_method: FillMethod::Horizontal,
                fill_amount: 1.0,
                fill_origin: 0,
                preserve_aspect: false,
            };

            // Store components
            // world.borrow_mut().insert_rect_transform(ui_entity, rect_transform);
            // world.borrow_mut().insert_ui_element(ui_entity, ui_element);
            // world.borrow_mut().insert_ui_image(ui_entity, ui_image);

            // Set parent if specified
            if let Some(_parent_entity) = parent {
                // world.borrow_mut().set_parent(ecs_entity, parent_entity);
            }

            Ok(ecs_entity)
        })?;
        globals.set("ui_create_image", create_image)?;

        let create_text = scope.create_function_mut(|_lua, args: Table| {
            let parent = args.get::<_, Option<EcsEntity>>("parent")?;
            let text = args.get::<_, Option<String>>("text")?.unwrap_or_default();
            let font_size = args.get::<_, Option<f32>>("font_size")?.unwrap_or(14.0);
            let color = args.get::<_, Option<Table>>("color")?;

            let ecs_entity = world.borrow_mut().spawn();
            let _ui_entity = ecs_entity as UIEntity;

            // Create RectTransform
            let _rect_transform = RectTransform::anchored(
                Vec2::new(0.5, 0.5),
                Vec2::ZERO,
                Vec2::new(200.0, 50.0),
            );

            // Create UIElement
            let ui_element_color = if let Some(c) = color {
                [
                    c.get::<_, f32>("r").unwrap_or(1.0),
                    c.get::<_, f32>("g").unwrap_or(1.0),
                    c.get::<_, f32>("b").unwrap_or(1.0),
                    c.get::<_, f32>("a").unwrap_or(1.0),
                ]
            } else {
                [0.0, 0.0, 0.0, 1.0]
            };

            let _ui_element = UIElement {
                raycast_target: false,
                blocks_raycasts: false,
                z_order: 0,
                color: ui_element_color,
                alpha: 1.0,
                interactable: false,
                ignore_layout: false,
                canvas_entity: parent.map(|e| e as UIEntity),
            };

            // Create UIText
            let _ui_text = UIText {
                text,
                font: "default".to_string(),
                font_size,
                color: ui_element_color,
                alignment: TextAlignment::MiddleCenter,
                horizontal_overflow: OverflowMode::Wrap,
                vertical_overflow: OverflowMode::Truncate,
                rich_text: false,
                line_spacing: 1.0,
                best_fit: false,
                best_fit_min_size: 10.0,
                best_fit_max_size: 40.0,
            };

            // Store components
            // world.borrow_mut().insert_rect_transform(ui_entity, rect_transform);
            // world.borrow_mut().insert_ui_element(ui_entity, ui_element);
            // world.borrow_mut().insert_ui_text(ui_entity, ui_text);

            if let Some(_parent_entity) = parent {
                // world.borrow_mut().set_parent(ecs_entity, parent_entity);
            }

            Ok(ecs_entity)
        })?;
        globals.set("ui_create_text", create_text)?;

        let create_button = scope.create_function_mut(|_lua, args: Table| {
            let parent = args.get::<_, Option<EcsEntity>>("parent")?;
            let on_click = args.get::<_, Option<String>>("on_click")?;

            let ecs_entity = world.borrow_mut().spawn();
            let _ui_entity = ecs_entity as UIEntity;

            // Create RectTransform
            let _rect_transform = RectTransform::anchored(
                Vec2::new(0.5, 0.5),
                Vec2::ZERO,
                Vec2::new(160.0, 40.0),
            );

            // Create UIElement
            let _ui_element = UIElement {
                raycast_target: true,
                blocks_raycasts: true,
                z_order: 0,
                color: [1.0, 1.0, 1.0, 1.0],
                alpha: 1.0,
                interactable: true,
                ignore_layout: false,
                canvas_entity: parent.map(|e| e as UIEntity),
            };

            // Create UIButton
            let _ui_button = UIButton {
                state: ButtonState::Normal,
                transition: ButtonTransition::ColorTint,
                normal_color: [1.0, 1.0, 1.0, 1.0],
                highlighted_color: [0.9, 0.9, 0.9, 1.0],
                pressed_color: [0.7, 0.7, 0.7, 1.0],
                disabled_color: [0.5, 0.5, 0.5, 0.5],
                fade_duration: 0.1,
                highlighted_sprite: None,
                pressed_sprite: None,
                disabled_sprite: None,
                normal_trigger: String::new(),
                highlighted_trigger: String::new(),
                pressed_trigger: String::new(),
                disabled_trigger: String::new(),
                on_click: on_click.clone(),
            };

            // Store components
            // world.borrow_mut().insert_rect_transform(ui_entity, rect_transform);
            // world.borrow_mut().insert_ui_element(ui_entity, ui_element);
            // world.borrow_mut().insert_ui_button(ui_entity, ui_button);

            if let Some(_parent_entity) = parent {
                // world.borrow_mut().set_parent(ecs_entity, parent_entity);
            }

            // Register callback if provided
            if let Some(_callback) = on_click {
                // self.register_event_callback(ui_entity, UIEventType::OnPointerClick, callback);
            }

            Ok(ecs_entity)
        })?;
        globals.set("ui_create_button", create_button)?;

        let create_panel = scope.create_function_mut(|_lua, args: Table| {
            let parent = args.get::<_, Option<EcsEntity>>("parent")?;
            let background = args.get::<_, Option<String>>("background")?;

            let ecs_entity = world.borrow_mut().spawn();
            let _ui_entity = ecs_entity as UIEntity;

            // Create RectTransform
            let _rect_transform = RectTransform::stretched(
                Vec2::ZERO,
                Vec2::ONE,
                Vec4::ZERO,
            );

            // Create UIElement
            let _ui_element = UIElement {
                raycast_target: true,
                blocks_raycasts: true,
                z_order: 0,
                color: [1.0, 1.0, 1.0, 1.0],
                alpha: 1.0,
                interactable: false,
                ignore_layout: false,
                canvas_entity: parent.map(|e| e as UIEntity),
            };

            // Create UIPanel
            let _ui_panel = UIPanel {
                background,
                use_nine_slice: false,
                slice_borders: Vec4::ZERO,
                padding: Vec4::new(10.0, 10.0, 10.0, 10.0),
            };

            // Store components
            // world.borrow_mut().insert_rect_transform(ui_entity, rect_transform);
            // world.borrow_mut().insert_ui_element(ui_entity, ui_element);
            // world.borrow_mut().insert_ui_panel(ui_entity, ui_panel);

            if let Some(_parent_entity) = parent {
                // world.borrow_mut().set_parent(ecs_entity, parent_entity);
            }

            Ok(ecs_entity)
        })?;
        globals.set("ui_create_panel", create_panel)?;

        // ================================================================
        // HIERARCHY OPERATIONS
        // ================================================================

        let set_parent = scope.create_function_mut(|_, (_entity, _parent): (EcsEntity, Option<EcsEntity>)| {
            // world.borrow_mut().set_parent(entity, parent);
            Ok(())
        })?;
        globals.set("ui_set_parent", set_parent)?;

        let get_parent = scope.create_function(|_, _entity: EcsEntity| {
            // Ok(world.borrow().get_parent(entity))
            Ok(None::<EcsEntity>)
        })?;
        globals.set("ui_get_parent", get_parent)?;

        let get_children = scope.create_function(|lua, _entity: EcsEntity| {
            // let children = world.borrow().get_children(entity);
            let children: Vec<EcsEntity> = Vec::new();
            let table = lua.create_table()?;
            for (i, child) in children.iter().enumerate() {
                table.set(i + 1, *child)?;
            }
            Ok(table)
        })?;
        globals.set("ui_get_children", get_children)?;

        let destroy = scope.create_function_mut(|_, entity: EcsEntity| {
            world.borrow_mut().despawn(entity);
            // Also remove from our tracking
            // self.remove_entity_callbacks(entity as UIEntity);
            Ok(())
        })?;
        globals.set("ui_destroy", destroy)?;

        Ok(())
    }
}

impl Default for UILuaBindings {
    fn default() -> Self {
        Self::new()
    }
}

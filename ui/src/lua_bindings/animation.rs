//! Animation and event callback functions for UI Lua bindings

use mlua::{Lua, Table};
use anyhow::Result;
use std::cell::RefCell;
use ecs::World;
use crate::*;

// Entity type from ecs crate (u32)
type EcsEntity = ecs::Entity;
// Entity type from ui crate (u64)
type UIEntity = crate::Entity;

/// Inject animation API into Lua scope
pub fn inject_animation_api<'lua, 'scope>(
    lua: &'lua Lua,
    scope: &mlua::Scope<'lua, 'scope>,
    world: &'scope RefCell<&mut World>,
) -> Result<()> {
    let globals = lua.globals();

    // ================================================================
    // ANIMATION FUNCTIONS
    // ================================================================

    let animate_position = scope.create_function_mut(|_, args: Table| {
        let entity = args.get::<_, EcsEntity>("entity")?;
        let to_x = args.get::<_, f32>("to_x")?;
        let to_y = args.get::<_, f32>("to_y")?;
        let duration = args.get::<_, f32>("duration")?;
        let easing = args.get::<_, Option<String>>("easing")?.unwrap_or("Linear".to_string());
        let on_complete = args.get::<_, Option<String>>("on_complete")?;

        // Create animation
        // let animation = UIAnimation {
        //     entity,
        //     property: AnimatedProperty::AnchoredPosition,
        //     from: AnimationValue::Vec2(current_position),
        //     to: AnimationValue::Vec2(Vec2::new(to_x, to_y)),
        //     duration,
        //     easing: parse_easing(&easing),
        //     delay: 0.0,
        //     loop_mode: LoopMode::Once,
        //     on_complete,
        //     elapsed: 0.0,
        //     started: false,
        //     completed: false,
        // };
        // world.borrow_mut().add_animation(animation);

        Ok(())
    })?;
    globals.set("ui_animate_position", animate_position)?;

    let animate_scale = scope.create_function_mut(|_, args: Table| {
        let entity = args.get::<_, EcsEntity>("entity")?;
        let to_x = args.get::<_, f32>("to_x")?;
        let to_y = args.get::<_, f32>("to_y")?;
        let duration = args.get::<_, f32>("duration")?;
        let easing = args.get::<_, Option<String>>("easing")?.unwrap_or("Linear".to_string());
        let on_complete = args.get::<_, Option<String>>("on_complete")?;

        // Create animation
        // Similar to animate_position but for scale

        Ok(())
    })?;
    globals.set("ui_animate_scale", animate_scale)?;

    let animate_rotation = scope.create_function_mut(|_, args: Table| {
        let entity = args.get::<_, EcsEntity>("entity")?;
        let to_rotation = args.get::<_, f32>("to")?;
        let duration = args.get::<_, f32>("duration")?;
        let easing = args.get::<_, Option<String>>("easing")?.unwrap_or("Linear".to_string());
        let on_complete = args.get::<_, Option<String>>("on_complete")?;

        // Create animation for rotation

        Ok(())
    })?;
    globals.set("ui_animate_rotation", animate_rotation)?;

    let animate_color = scope.create_function_mut(|_, args: Table| {
        let entity = args.get::<_, EcsEntity>("entity")?;
        let to_r = args.get::<_, f32>("to_r")?;
        let to_g = args.get::<_, f32>("to_g")?;
        let to_b = args.get::<_, f32>("to_b")?;
        let to_a = args.get::<_, f32>("to_a")?;
        let duration = args.get::<_, f32>("duration")?;
        let easing = args.get::<_, Option<String>>("easing")?.unwrap_or("Linear".to_string());
        let on_complete = args.get::<_, Option<String>>("on_complete")?;

        // Create animation for color

        Ok(())
    })?;
    globals.set("ui_animate_color", animate_color)?;

    let animate_alpha = scope.create_function_mut(|_, args: Table| {
        let entity = args.get::<_, EcsEntity>("entity")?;
        let to_alpha = args.get::<_, f32>("to")?;
        let duration = args.get::<_, f32>("duration")?;
        let easing = args.get::<_, Option<String>>("easing")?.unwrap_or("Linear".to_string());
        let on_complete = args.get::<_, Option<String>>("on_complete")?;

        // Create animation for alpha

        Ok(())
    })?;
    globals.set("ui_animate_alpha", animate_alpha)?;

    let stop_animation = scope.create_function_mut(|_, entity: EcsEntity| {
        // world.borrow_mut().remove_animations(entity);
        Ok(())
    })?;
    globals.set("ui_stop_animation", stop_animation)?;

    Ok(())
}

/// Inject event callback API into Lua scope
pub fn inject_event_api<'lua, 'scope>(
    lua: &'lua Lua,
    scope: &mlua::Scope<'lua, 'scope>,
    _world: &'scope RefCell<&mut World>,
    bindings: &'scope crate::lua_bindings::UILuaBindings,
) -> Result<()> {
    let globals = lua.globals();

    // ================================================================
    // EVENT CALLBACK REGISTRATION
    // ================================================================

    let on_click = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnPointerClick, callback);
        Ok(())
    })?;
    globals.set("ui_on_click", on_click)?;

    let on_pointer_enter = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnPointerEnter, callback);
        Ok(())
    })?;
    globals.set("ui_on_pointer_enter", on_pointer_enter)?;

    let on_pointer_exit = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnPointerExit, callback);
        Ok(())
    })?;
    globals.set("ui_on_pointer_exit", on_pointer_exit)?;

    let on_pointer_down = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnPointerDown, callback);
        Ok(())
    })?;
    globals.set("ui_on_pointer_down", on_pointer_down)?;

    let on_pointer_up = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnPointerUp, callback);
        Ok(())
    })?;
    globals.set("ui_on_pointer_up", on_pointer_up)?;

    let on_drag = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnDrag, callback);
        Ok(())
    })?;
    globals.set("ui_on_drag", on_drag)?;

    let on_begin_drag = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnBeginDrag, callback);
        Ok(())
    })?;
    globals.set("ui_on_begin_drag", on_begin_drag)?;

    let on_end_drag = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnEndDrag, callback);
        Ok(())
    })?;
    globals.set("ui_on_end_drag", on_end_drag)?;

    let on_scroll = scope.create_function_mut(move |_, (entity, callback): (EcsEntity, String)| {
        bindings.register_event_callback(entity as UIEntity, UIEventType::OnScroll, callback);
        Ok(())
    })?;
    globals.set("ui_on_scroll", on_scroll)?;

    let on_value_changed = scope.create_function_mut(|_, (_entity, _callback): (EcsEntity, String)| {
        // For sliders, toggles, dropdowns, input fields
        // This would need to be handled differently for each component type
        // Store the callback in the component itself (e.g., UISlider.on_value_changed)
        // TODO: Implement component-specific value change callbacks
        Ok(())
    })?;
    globals.set("ui_on_value_changed", on_value_changed)?;

    // Remove event callback
    let remove_event_callback = scope.create_function_mut(move |_, (entity, event_type): (EcsEntity, String)| {
        // Parse event type and remove specific callback
        // For now, we'll just note this needs implementation
        // TODO: Implement selective callback removal
        Ok(())
    })?;
    globals.set("ui_remove_event_callback", remove_event_callback)?;

    // Remove all event callbacks for an entity
    let remove_all_callbacks = scope.create_function_mut(move |_, entity: EcsEntity| {
        bindings.remove_entity_callbacks(entity as UIEntity);
        Ok(())
    })?;
    globals.set("ui_remove_all_callbacks", remove_all_callbacks)?;

    Ok(())
}

/// Inject element query API into Lua scope
pub fn inject_query_api<'lua, 'scope>(
    lua: &'lua Lua,
    scope: &mlua::Scope<'lua, 'scope>,
    _world: &'scope RefCell<&mut World>,
    bindings: &'scope crate::lua_bindings::UILuaBindings,
) -> Result<()> {
    let globals = lua.globals();

    // ================================================================
    // ELEMENT QUERIES
    // ================================================================

    let find_by_name = scope.create_function(move |_, name: String| {
        Ok(bindings.find_by_name(&name).map(|e| e as EcsEntity))
    })?;
    globals.set("ui_find_by_name", find_by_name)?;

    let find_by_tag = scope.create_function(move |lua, tag: String| {
        let entities = bindings.find_by_tag(&tag);
        let table = lua.create_table()?;
        for (i, entity) in entities.iter().enumerate() {
            table.set(i + 1, *entity as EcsEntity)?;
        }
        Ok(table)
    })?;
    globals.set("ui_find_by_tag", find_by_tag)?;

    let set_name = scope.create_function_mut(move |_, (entity, name): (EcsEntity, String)| {
        bindings.register_named_element(name, entity as UIEntity);
        Ok(())
    })?;
    globals.set("ui_set_name", set_name)?;

    let set_tag = scope.create_function_mut(move |_, (entity, tag): (EcsEntity, String)| {
        bindings.register_tagged_element(tag, entity as UIEntity);
        Ok(())
    })?;
    globals.set("ui_set_tag", set_tag)?;

    let get_active = scope.create_function(|_, _entity: EcsEntity| {
        // Check if entity exists and is active
        // TODO: Implement with actual ECS integration
        // Check if entity exists in world and has non-zero alpha
        Ok(true)
    })?;
    globals.set("ui_get_active", get_active)?;

    let set_active = scope.create_function_mut(|_, (_entity, _active): (EcsEntity, bool)| {
        // Set entity active/inactive (affects visibility)
        // TODO: Implement with actual ECS integration
        // if let Some(mut elem) = world.borrow_mut().get_mut::<UIElement>(entity) {
        //     elem.alpha = if active { 1.0 } else { 0.0 };
        // }
        Ok(())
    })?;
    globals.set("ui_set_active", set_active)?;

    // Get entity by index in parent
    let get_child = scope.create_function(|_, (_parent, _index): (EcsEntity, usize)| {
        // TODO: Get child at index from ECS hierarchy
        Ok(None::<EcsEntity>)
    })?;
    globals.set("ui_get_child", get_child)?;

    // Get number of children
    let get_child_count = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get child count from ECS hierarchy
        Ok(0)
    })?;
    globals.set("ui_get_child_count", get_child_count)?;

    // Get sibling index
    let get_sibling_index = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get sibling index from ECS hierarchy
        Ok(0)
    })?;
    globals.set("ui_get_sibling_index", get_sibling_index)?;

    // Set sibling index (reorder in parent)
    let set_sibling_index = scope.create_function_mut(|_, (_entity, _index): (EcsEntity, usize)| {
        // TODO: Set sibling index in ECS hierarchy
        Ok(())
    })?;
    globals.set("ui_set_sibling_index", set_sibling_index)?;

    // Check if entity exists
    let exists = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Check if entity exists in world
        Ok(true)
    })?;
    globals.set("ui_exists", exists)?;

    // Get canvas for element
    let get_canvas = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get canvas entity from UIElement
        Ok(None::<EcsEntity>)
    })?;
    globals.set("ui_get_canvas", get_canvas)?;

    Ok(())
}

/// Helper function to parse easing function name
fn parse_easing(name: &str) -> EasingFunction {
    match name {
        "Linear" => EasingFunction::Linear,
        "EaseInQuad" => EasingFunction::EaseInQuad,
        "EaseOutQuad" => EasingFunction::EaseOutQuad,
        "EaseInOutQuad" => EasingFunction::EaseInOutQuad,
        "EaseInCubic" => EasingFunction::EaseInCubic,
        "EaseOutCubic" => EasingFunction::EaseOutCubic,
        "EaseInOutCubic" => EasingFunction::EaseInOutCubic,
        "EaseInSine" => EasingFunction::EaseInSine,
        "EaseOutSine" => EasingFunction::EaseOutSine,
        "EaseInOutSine" => EasingFunction::EaseInOutSine,
        "EaseInBack" => EasingFunction::EaseInBack,
        "EaseOutBack" => EasingFunction::EaseOutBack,
        "EaseInOutBack" => EasingFunction::EaseInOutBack,
        "EaseInBounce" => EasingFunction::EaseInBounce,
        "EaseOutBounce" => EasingFunction::EaseOutBounce,
        "EaseInOutBounce" => EasingFunction::EaseInOutBounce,
        _ => EasingFunction::Linear,
    }
}

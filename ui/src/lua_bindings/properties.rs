//! Property manipulation functions for UI Lua bindings

use mlua::{Lua, Table};
use anyhow::Result;
use std::cell::RefCell;
use ecs::World;

// Entity type from ecs crate (u32)
type EcsEntity = ecs::Entity;

/// Inject property getter/setter API into Lua scope
pub fn inject_property_api<'lua, 'scope>(
    lua: &'lua Lua,
    scope: &mlua::Scope<'lua, 'scope>,
    _world: &'scope RefCell<&mut World>,
) -> Result<()> {
    let globals = lua.globals();

    // ================================================================
    // RECTTRANSFORM PROPERTIES
    // ================================================================

    let get_position = scope.create_function(|lua, _entity: EcsEntity| {
        // TODO: Get actual position from ECS
        // let rect_transform = world.borrow().get::<RectTransform>(entity)?;
        // let pos = rect_transform.anchored_position;
        let table = lua.create_table()?;
        table.set("x", 0.0)?;
        table.set("y", 0.0)?;
        Ok(table)
    })?;
    globals.set("ui_get_position", get_position)?;

    let set_position = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set position in ECS
        // if let Some(mut rect_transform) = world.borrow_mut().get_mut::<RectTransform>(entity) {
        //     rect_transform.anchored_position = Vec2::new(x, y);
        //     rect_transform.dirty = true;
        // }
        Ok(())
    })?;
    globals.set("ui_set_position", set_position)?;

    let get_size = scope.create_function(|lua, _entity: EcsEntity| {
        // TODO: Get actual size from ECS
        // let rect_transform = world.borrow().get::<RectTransform>(entity)?;
        // let size = rect_transform.get_size();
        let table = lua.create_table()?;
        table.set("x", 100.0)?;
        table.set("y", 100.0)?;
        Ok(table)
    })?;
    globals.set("ui_get_size", get_size)?;

    let set_size = scope.create_function_mut(|_, (_entity, _width, _height): (EcsEntity, f32, f32)| {
        // TODO: Set size in ECS
        // if let Some(mut rect_transform) = world.borrow_mut().get_mut::<RectTransform>(entity) {
        //     rect_transform.set_size(Vec2::new(width, height));
        //     rect_transform.dirty = true;
        // }
        Ok(())
    })?;
    globals.set("ui_set_size", set_size)?;

    let get_anchor_min = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("x", 0.5)?;
        table.set("y", 0.5)?;
        Ok(table)
    })?;
    globals.set("ui_get_anchor_min", get_anchor_min)?;

    let set_anchor_min = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set anchor_min in ECS
        Ok(())
    })?;
    globals.set("ui_set_anchor_min", set_anchor_min)?;

    let get_anchor_max = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("x", 0.5)?;
        table.set("y", 0.5)?;
        Ok(table)
    })?;
    globals.set("ui_get_anchor_max", get_anchor_max)?;

    let set_anchor_max = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set anchor_max in ECS
        Ok(())
    })?;
    globals.set("ui_set_anchor_max", set_anchor_max)?;

    let get_pivot = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("x", 0.5)?;
        table.set("y", 0.5)?;
        Ok(table)
    })?;
    globals.set("ui_get_pivot", get_pivot)?;

    let set_pivot = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set pivot in ECS
        Ok(())
    })?;
    globals.set("ui_set_pivot", set_pivot)?;

    let get_rotation = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get rotation from ECS
        Ok(0.0)
    })?;
    globals.set("ui_get_rotation", get_rotation)?;

    let set_rotation = scope.create_function_mut(|_, (_entity, _rotation): (EcsEntity, f32)| {
        // TODO: Set rotation in ECS
        Ok(())
    })?;
    globals.set("ui_set_rotation", set_rotation)?;

    let get_scale = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("x", 1.0)?;
        table.set("y", 1.0)?;
        Ok(table)
    })?;
    globals.set("ui_get_scale", get_scale)?;

    let set_scale = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set scale in ECS
        Ok(())
    })?;
    globals.set("ui_set_scale", set_scale)?;

    // ================================================================
    // UIELEMENT PROPERTIES
    // ================================================================

    let get_color = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("r", 1.0)?;
        table.set("g", 1.0)?;
        table.set("b", 1.0)?;
        table.set("a", 1.0)?;
        Ok(table)
    })?;
    globals.set("ui_get_color", get_color)?;

    let set_color = scope.create_function_mut(|_, args: Table| {
        let _entity = args.get::<_, EcsEntity>("entity")?;
        let _r = args.get::<_, f32>("r")?;
        let _g = args.get::<_, f32>("g")?;
        let _b = args.get::<_, f32>("b")?;
        let _a = args.get::<_, f32>("a")?;
        // TODO: Set color in ECS
        Ok(())
    })?;
    globals.set("ui_set_color", set_color)?;

    let get_alpha = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get alpha from ECS
        Ok(1.0)
    })?;
    globals.set("ui_get_alpha", get_alpha)?;

    let set_alpha = scope.create_function_mut(|_, (_entity, _alpha): (EcsEntity, f32)| {
        // TODO: Set alpha in ECS
        Ok(())
    })?;
    globals.set("ui_set_alpha", set_alpha)?;

    let get_interactable = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get interactable from ECS
        Ok(true)
    })?;
    globals.set("ui_get_interactable", get_interactable)?;

    let set_interactable = scope.create_function_mut(|_, (_entity, _interactable): (EcsEntity, bool)| {
        // TODO: Set interactable in ECS
        Ok(())
    })?;
    globals.set("ui_set_interactable", set_interactable)?;

    let get_raycast_target = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get raycast_target from ECS
        Ok(true)
    })?;
    globals.set("ui_get_raycast_target", get_raycast_target)?;

    let set_raycast_target = scope.create_function_mut(|_, (_entity, _raycast_target): (EcsEntity, bool)| {
        // TODO: Set raycast_target in ECS
        Ok(())
    })?;
    globals.set("ui_set_raycast_target", set_raycast_target)?;

    // ================================================================
    // UITEXT PROPERTIES
    // ================================================================

    let get_text = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get text from ECS
        Ok("".to_string())
    })?;
    globals.set("ui_get_text", get_text)?;

    let set_text = scope.create_function_mut(|_, (_entity, _text): (EcsEntity, String)| {
        // TODO: Set text in ECS
        Ok(())
    })?;
    globals.set("ui_set_text", set_text)?;

    let get_font_size = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get font_size from ECS
        Ok(14.0)
    })?;
    globals.set("ui_get_font_size", get_font_size)?;

    let set_font_size = scope.create_function_mut(|_, (_entity, _font_size): (EcsEntity, f32)| {
        // TODO: Set font_size in ECS
        Ok(())
    })?;
    globals.set("ui_set_font_size", set_font_size)?;

    let get_text_alignment = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get text alignment from ECS
        Ok("MiddleCenter".to_string())
    })?;
    globals.set("ui_get_text_alignment", get_text_alignment)?;

    let set_text_alignment = scope.create_function_mut(|_, (_entity, _alignment): (EcsEntity, String)| {
        // TODO: Set text alignment in ECS
        Ok(())
    })?;
    globals.set("ui_set_text_alignment", set_text_alignment)?;

    // ================================================================
    // UIIMAGE PROPERTIES
    // ================================================================

    let get_sprite = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get sprite from ECS
        Ok(None::<String>)
    })?;
    globals.set("ui_get_sprite", get_sprite)?;

    let set_sprite = scope.create_function_mut(|_, (_entity, _sprite): (EcsEntity, Option<String>)| {
        // TODO: Set sprite in ECS
        Ok(())
    })?;
    globals.set("ui_set_sprite", set_sprite)?;

    let get_fill_amount = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get fill_amount from ECS
        Ok(1.0)
    })?;
    globals.set("ui_get_fill_amount", get_fill_amount)?;

    let set_fill_amount = scope.create_function_mut(|_, (_entity, _fill_amount): (EcsEntity, f32)| {
        // TODO: Set fill_amount in ECS (clamped to 0-1)
        Ok(())
    })?;
    globals.set("ui_set_fill_amount", set_fill_amount)?;

    // ================================================================
    // UISLIDER PROPERTIES
    // ================================================================

    let get_slider_value = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get slider value from ECS
        Ok(0.0)
    })?;
    globals.set("ui_get_slider_value", get_slider_value)?;

    let set_slider_value = scope.create_function_mut(|_, (_entity, _value): (EcsEntity, f32)| {
        // TODO: Set slider value in ECS
        Ok(())
    })?;
    globals.set("ui_set_slider_value", set_slider_value)?;

    let get_slider_min = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get slider min from ECS
        Ok(0.0)
    })?;
    globals.set("ui_get_slider_min", get_slider_min)?;

    let set_slider_min = scope.create_function_mut(|_, (_entity, _min): (EcsEntity, f32)| {
        // TODO: Set slider min in ECS
        Ok(())
    })?;
    globals.set("ui_set_slider_min", set_slider_min)?;

    let get_slider_max = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get slider max from ECS
        Ok(1.0)
    })?;
    globals.set("ui_get_slider_max", get_slider_max)?;

    let set_slider_max = scope.create_function_mut(|_, (_entity, _max): (EcsEntity, f32)| {
        // TODO: Set slider max in ECS
        Ok(())
    })?;
    globals.set("ui_set_slider_max", set_slider_max)?;

    // ================================================================
    // UITOGGLE PROPERTIES
    // ================================================================

    let get_toggle_value = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get toggle is_on from ECS
        Ok(false)
    })?;
    globals.set("ui_get_toggle_value", get_toggle_value)?;

    let set_toggle_value = scope.create_function_mut(|_, (_entity, _is_on): (EcsEntity, bool)| {
        // TODO: Set toggle is_on in ECS
        Ok(())
    })?;
    globals.set("ui_set_toggle_value", set_toggle_value)?;

    // ================================================================
    // UIDROPDOWN PROPERTIES
    // ================================================================

    let get_dropdown_value = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get dropdown value from ECS
        Ok(0)
    })?;
    globals.set("ui_get_dropdown_value", get_dropdown_value)?;

    let set_dropdown_value = scope.create_function_mut(|_, (_entity, _value): (EcsEntity, i32)| {
        // TODO: Set dropdown value in ECS
        Ok(())
    })?;
    globals.set("ui_set_dropdown_value", set_dropdown_value)?;

    let get_dropdown_options = scope.create_function(|lua, _entity: EcsEntity| {
        // TODO: Get dropdown options from ECS
        let table = lua.create_table()?;
        Ok(table)
    })?;
    globals.set("ui_get_dropdown_options", get_dropdown_options)?;

    let set_dropdown_options = scope.create_function_mut(|_, (_entity, _options): (EcsEntity, Table)| {
        // TODO: Set dropdown options in ECS
        Ok(())
    })?;
    globals.set("ui_set_dropdown_options", set_dropdown_options)?;

    // ================================================================
    // UIINPUTFIELD PROPERTIES
    // ================================================================

    let get_input_text = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get input field text from ECS
        Ok("".to_string())
    })?;
    globals.set("ui_get_input_text", get_input_text)?;

    let set_input_text = scope.create_function_mut(|_, (_entity, _text): (EcsEntity, String)| {
        // TODO: Set input field text in ECS
        Ok(())
    })?;
    globals.set("ui_set_input_text", set_input_text)?;

    let get_input_placeholder = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get input field placeholder from ECS
        Ok("".to_string())
    })?;
    globals.set("ui_get_input_placeholder", get_input_placeholder)?;

    let set_input_placeholder = scope.create_function_mut(|_, (_entity, _placeholder): (EcsEntity, String)| {
        // TODO: Set input field placeholder in ECS
        Ok(())
    })?;
    globals.set("ui_set_input_placeholder", set_input_placeholder)?;

    let get_input_character_limit = scope.create_function(|_, _entity: EcsEntity| {
        // TODO: Get input field character limit from ECS
        Ok(0)
    })?;
    globals.set("ui_get_input_character_limit", get_input_character_limit)?;

    let set_input_character_limit = scope.create_function_mut(|_, (_entity, _limit): (EcsEntity, i32)| {
        // TODO: Set input field character limit in ECS
        Ok(())
    })?;
    globals.set("ui_set_input_character_limit", set_input_character_limit)?;

    // ================================================================
    // UISCROLLVIEW PROPERTIES
    // ================================================================

    let get_scroll_position = scope.create_function(|lua, _entity: EcsEntity| {
        let table = lua.create_table()?;
        table.set("x", 0.0)?;
        table.set("y", 0.0)?;
        Ok(table)
    })?;
    globals.set("ui_get_scroll_position", get_scroll_position)?;

    let set_scroll_position = scope.create_function_mut(|_, (_entity, _x, _y): (EcsEntity, f32, f32)| {
        // TODO: Set scroll position in ECS
        Ok(())
    })?;
    globals.set("ui_set_scroll_position", set_scroll_position)?;

    Ok(())
}

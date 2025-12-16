// Script system for runtime
use ecs::World;
use script::ScriptEngine;
use input::InputSystem;

pub fn update_scripts(
    script_engine: &mut ScriptEngine,
    world: &mut World,
    input: &InputSystem,
    delta_time: f32,
) {
    // Collect entities with scripts to avoid borrowing conflicts
    let entities: Vec<ecs::Entity> = world.scripts.keys().cloned().collect();

    for entity in entities {
        let should_run = if let Some(script) = world.scripts.get(&entity) {
            script.enabled
        } else {
            false
        };

        if should_run {
            let mut log_callback = |msg: String| {
                log::info!("[Lua] {}", msg);
            };

            // Path is currently unused by run_script (it looks up by entity), so passing empty path is safe
            // The script content should have been loaded via load_script_for_entity previously (e.g. in Awake/Start)
            let _ = script_engine.run_script(
                std::path::Path::new(""),
                entity,
                world,
                input,
                delta_time,
                &mut log_callback,
            );
        }
    }
}
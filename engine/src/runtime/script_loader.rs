use ecs::World;
use script::ScriptEngine;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Load and initialize all scripts in the world (Unity-style lifecycle)
#[allow(dead_code)]
pub fn load_all_scripts(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    // scripts_folder argument removed - we use AssetLoader with "scripts/" prefix
) -> Result<()> {
    let entities_with_scripts: Vec<_> = world.scripts.keys().cloned().collect();

    // Phase 1: Load scripts and call Awake() for all entities
    for entity in &entities_with_scripts {
        if let Some(script) = world.scripts.get(entity) {
            if script.enabled {
                let script_name = script.script_name.clone();
                let script_path = format!("scripts/{}.lua", script_name);

                // Use AssetLoader from ScriptEngine
                match pollster::block_on(script_engine.asset_loader.load_text(&script_path)) {
                    Ok(content) => {
                         if let Err(e) = script_engine.load_script_for_entity(*entity, &content, world) {
                            log::error!("Failed to load script {} for entity {}: {}", script_name, entity, e);
                        } else {
                            log::info!("Loaded script: {} (Awake called)", script_name);
                        }
                    }
                    Err(e) => {
                         log::warn!("Script file not found or failed to load: {} ({})", script_path, e);
                    }
                }
            }
        }
    }

    // Phase 2: Call Start() for all entities (after all Awake() calls)
    for entity in &entities_with_scripts {
        if let Some(script) = world.scripts.get(entity) {
            if script.enabled {
                if let Err(e) = script_engine.call_start_for_entity(*entity, world) {
                    log::error!("Failed to call Start() for entity {}: {}", entity, e);
                }
            }
        }
    }

    Ok(())
}

/// Run all enabled scripts in the world
#[allow(dead_code)]
pub fn run_all_scripts(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    // scripts_folder argument removed
    input: &input::InputSystem,
    dt: f32,
) -> Result<()> {
    let entities_with_scripts: Vec<_> = world.scripts.keys().cloned().collect();

    for entity in entities_with_scripts {
        if let Some(script) = world.scripts.get(&entity) {
            if script.enabled {
                // script_path info is effectively unused by run_script (it uses entity state),
                // but we can pass a dummy path or reconstruct if needed for logging.
                let script_path = Path::new("scripts").join(format!("{}.lua", script.script_name));

                let mut log_callback = |msg: String| {
                    log::info!("{}", msg);
                };
                if let Err(e) = script_engine.run_script(
                    &script_path,
                    entity,
                    world,
                    input,
                    dt,
                    &mut log_callback,
                ) {
                    log::error!("Script error for entity {}: {}", entity, e);
                }
            }
        }
    }

    Ok(())
}

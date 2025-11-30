use ecs::World;
use script::ScriptEngine;
use std::path::PathBuf;
use anyhow::Result;

/// Load and initialize all scripts in the world (Unity-style lifecycle)
#[allow(dead_code)]
pub fn load_all_scripts(
    world: &World,
    script_engine: &mut ScriptEngine,
    scripts_folder: &PathBuf,
) -> Result<()> {
    let entities_with_scripts: Vec<_> = world.scripts.keys().cloned().collect();

    // Phase 1: Load scripts and call Awake() for all entities
    for entity in &entities_with_scripts {
        if let Some(script) = world.scripts.get(entity) {
            if script.enabled {
                let script_name = script.script_name.clone();
                let script_path = scripts_folder.join(format!("{}.lua", script_name));

                if script_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&script_path) {
                        if let Err(e) = script_engine.load_script_for_entity(*entity, &content, world) {
                            log::error!("Failed to load script {} for entity {}: {}", script_name, entity, e);
                        } else {
                            log::info!("Loaded script: {} (Awake called)", script_name);
                        }
                    }
                } else {
                    log::warn!("Script file not found: {:?}", script_path);
                }
            }
        }
    }

    // Phase 2: Call Start() for all entities (after all Awake() calls)
    for entity in &entities_with_scripts {
        if let Some(script) = world.scripts.get(entity) {
            if script.enabled {
                if let Err(e) = script_engine.call_start_for_entity(*entity) {
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
    scripts_folder: &PathBuf,
    input: &input::InputSystem,
    dt: f32,
) -> Result<()> {
    let entities_with_scripts: Vec<_> = world.scripts.keys().cloned().collect();

    for entity in entities_with_scripts {
        if let Some(script) = world.scripts.get(&entity) {
            if script.enabled {
                let script_path = scripts_folder.join(format!("{}.lua", script.script_name));

                if let Err(e) = script_engine.run_script(
                    &script_path,
                    entity,
                    world,
                    input,
                    dt,
                ) {
                    log::error!("Script error for entity {}: {}", entity, e);
                }
            }
        }
    }

    Ok(())
}

mod editor;
mod runtime;
mod texture_manager;
mod ui_manager;

use anyhow::Result;
use engine_core::{EngineContext, EngineModule, project::ProjectManager};
use ecs::{World, Entity, Transform, Sprite, Collider, EntityTag};
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
// Always import PhysicsWorld for helper functions
use physics::PhysicsWorld as SimplePhysicsWorld;
use render::RenderModule;
use ::editor::EditorModule as EditorMod;  // From editor crate (workspace)
use crate::editor::{EditorUI, TransformTool, AppState, LauncherState, EditorState, EditorAction};  // From local editor module
use input::Key;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct GameState {
    world: World,
    player: Option<Entity>,
    items: Vec<Entity>,
    collected_items: usize,
    player_speed: f32,

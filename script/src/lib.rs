use mlua::{Lua, Function, UserData, UserDataMethods};
use anyhow::Result;
use ecs::{World, Entity, Transform};

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    pub fn exec(&self, src: &str) -> Result<()> {
        self.lua.load(src).exec()?;
        Ok(())
    }

    pub fn call_update(&self, name: &str, dt: f32, world: &mut World) -> Result<()> {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<_, Function>(name) {
            self.lua.scope(|scope| {
                // Create a table to act as the "world" object
                // We try to use scope.create_table if it exists, or just pass functions directly if not.
                // Since I can't check existence easily, I'll assume I can't put scoped functions in global table.
                // I'll try to pass a UserData that I create via scope, which holds the functions?
                // No, UserData holds data.
                
                // Let's try passing functions directly for now to verify if func.call works with scoped args.
                // If this works, I can wrap them in a Lua table inside Lua?
                // func.call((dt, spawn, set_pos, get_pos))
                // And Lua side: function update(dt, spawn, set_pos, get_pos) ... end
                // This is ugly but functional.
                
                // Better: Create the table in Lua!
                // let make_table = self.lua.create_function(|_, (spawn, set_pos, get_pos)| {
                //    let t = self.lua.create_table()?;
                //    t.set("spawn", spawn)?; ...
                //    Ok(t)
                // })?; 
                // But make_table is global. It returns Table<'lua>.
                // It can't hold spawn (scoped).
                
                // So I can't return a Table<'lua> containing scoped functions.
                // I must pass a Table<'scope>?
                // Does mlua have Table<'scope>?
                // No, Table is Table<'lua>.
                
                // So I CANNOT put scoped functions in a Table.
                // I MUST pass them as arguments or use UserData.
                
                // So UserData is the ONLY way to pass an "object" with scoped lifetime.
                // And UserData failed with static requirement.
                
                // Let's retry UserData with `create_userdata_ref_mut` but ensuring `LuaWorld` is simple.
                // Maybe the error was due to `func.call`?
                
                // Let's try passing ONE function directly.
                
                // use std::cell::RefCell;
                // let world_ref = RefCell::new(world);
                // let w = &world_ref;
                
                let spawn = scope.create_function_mut(move |_, ()| {
                    Ok(world.spawn())
                })?;
                
                // Pass spawn directly
                func.call::<_, ()>((dt, spawn))?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

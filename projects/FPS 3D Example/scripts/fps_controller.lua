-- FPS Controller Script
-- Handles Movement (WASD) and Mouse Look

local move_speed = 5.0
local look_sensitivity = 0.2
local camera_entity = nil
local pitch = 0.0
local yaw = 0.0

function Start()
    print("FPS Controller started")
    
    -- Injected params are available as globals
    -- Note: engine injects them based on parameter names in JSON/Editor
    local params_speed = GetScriptParameter(entity, "move_speed")
    if params_speed and params_speed.Float then
        move_speed = params_speed.Float
        print("Set move_speed to " .. move_speed)
    end
    
    local params_sens = GetScriptParameter(entity, "look_sensitivity")
    if params_sens and params_sens.Float then
        look_sensitivity = params_sens.Float
        print("Set look_sensitivity to " .. look_sensitivity)
    end
    
    -- Find camera child (simple iteration)
    -- Since we parented specific entity, we can search all entities and check parents?
    -- No exposed parent API.
    -- Alternative: We know Camera is Entity 1. But that's hardcoded.
    -- Better: Iterate all entities, check if it has "Camera" component? 
    -- Or just use a known tag?
    -- For now, let's assume Camera is attached as child 1 (Entity 1).
    -- But entity IDs change.
    
    -- Let's try to find an entity named "Main Camera"
    -- We need a name query?
    -- `get_all_entities` gives IDs.
    -- We can try to guess or use a parameter "camera_id" if we could set it.
    
    -- HACK: In the provided main.json, Camera is Entity 1.
    camera_entity = 1
    
    -- Initialize yaw from current rotation
    local rot = get_rotation_euler()
    if rot then
        yaw = rot.y
    end
end

function Update(dt)
    -- 1. Mouse Look
    local delta = get_mouse_delta()
    if delta then
        -- Update yaw (Player rotation)
        yaw = yaw - delta.x * look_sensitivity * 0.1
        
        -- Update pitch (Camera rotation)
        pitch = pitch - delta.y * look_sensitivity * 0.1
        
        -- Clamp pitch
        if pitch > 89.0 then pitch = 89.0 end
        if pitch < -89.0 then pitch = -89.0 end
        
        -- Apply rotations
        -- Player rotates around Y only
        set_rotation_euler(0.0, yaw, 0.0)
        
        -- Camera rotates around local X (Pitch)
        if camera_entity then
            -- We set local rotation of camera (assuming parented)
            -- If camera is child, setting Transform.rotation sets LOCAL rotation?
            -- Engine architecture usually stores Transform.rotation as Local.
            set_rotation_of(camera_entity, pitch, 0.0, 0.0)
        end
    end

    -- 2. Movement (WASD)
    local move_x = 0.0
    local move_z = 0.0
    
    if is_key_down("W") then move_z = -1.0 end
    if is_key_down("S") then move_z = 1.0 end
    if is_key_down("A") then move_x = -1.0 end
    if is_key_down("D") then move_x = 1.0 end
    
    if move_x ~= 0.0 or move_z ~= 0.0 then
        -- Calculate movement direction relative to YAW
        -- World Forward is -Z (if Y is Up and standard OpenGL)
        -- We need to rotate input vector by Yaw
        
        local yaw_rad = math.rad(yaw)
        local cos_y = math.cos(yaw_rad)
        local sin_y = math.sin(yaw_rad)
        
        -- Forward vector (-sin(yaw), 0, -cos(yaw)) ? No,
        -- Standard rotation:
        -- Fwd = (sin(yaw), 0, cos(yaw)) ?
        -- Let's assume standard math:
        -- rotated_x = x * cos - z * sin
        -- rotated_z = x * sin + z * cos
        
        local world_move_x = move_x * cos_y - move_z * sin_y
        local world_move_z = move_x * sin_y + move_z * cos_y
        
        -- Normalize
        local length = math.sqrt(world_move_x*world_move_x + world_move_z*world_move_z)
        if length > 0 then
            world_move_x = world_move_x / length
            world_move_z = world_move_z / length
        end

        local pos_table = get_position()
        if pos_table then
            local current_x = pos_table.x
            local current_y = pos_table.y
            local current_z = pos_table.z or 0.0 

            -- Update position
            local new_x = current_x + world_move_x * move_speed * dt
            local new_z = current_z + world_move_z * move_speed * dt
            
            set_position(new_x, current_y, new_z)
        end
    end
end

-- HUD Controller Script
-- Updates all HUD elements based on player state

-- Configuration
local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"

-- State tracking
local frame_count = 0
local elapsed_time = 0
local current_fps = 0
local hud_loaded = false

function Start()
    print("HUD Controller: Starting...")
    
    -- Load and activate HUD
    UI.load_prefab(hud_prefab_path)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    
    hud_loaded = true
    print("HUD Controller: HUD loaded and activated")
end

function Update(dt)
    if not hud_loaded then
        return
    end
    
    -- Update FPS counter
    UpdateFPS(dt)
    
    -- Update player position and velocity
    UpdatePlayerDebugInfo()
    
    -- Update player state indicators
    UpdatePlayerStateIndicators()
    
    -- Update dash count
    UpdateDashIndicator()
end

function UpdateFPS(dt)
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    
    -- Update every 0.5 seconds
    if elapsed_time >= 0.5 then
        current_fps = math.floor(frame_count / elapsed_time)
        UI.set_text(hud_instance_name .. "/fps_counter", "FPS: " .. current_fps)
        
        frame_count = 0
        elapsed_time = 0
    end
end

function UpdatePlayerDebugInfo()
    -- Get player entity (assuming it's tagged as Player)
    local player_entity = GetEntityByTag("Player")
    if not player_entity then
        return
    end
    
    -- Get player transform
    local transform = GetTransform(player_entity)
    if transform then
        local pos_x = math.floor(transform.position[1] * 10) / 10
        local pos_y = math.floor(transform.position[2] * 10) / 10
        UI.set_text(hud_instance_name .. "/position_debug", string.format("X: %.1f Y: %.1f", pos_x, pos_y))
    end
    
    -- Get player velocity
    local velocity = GetVelocity(player_entity)
    if velocity then
        local vel_x = math.floor(velocity[1] * 10) / 10
        local vel_y = math.floor(velocity[2] * 10) / 10
        UI.set_text(hud_instance_name .. "/velocity_debug", string.format("VX: %.1f VY: %.1f", vel_x, vel_y))
    end
end

function UpdatePlayerStateIndicators()
    -- Get player entity
    local player_entity = GetEntityByTag("Player")
    if not player_entity then
        return
    end
    
    -- Check if player has script component
    local script = GetScript(player_entity)
    if not script then
        return
    end
    
    -- Get player state from script parameters
    local is_grounded = GetScriptParameter(player_entity, "is_grounded")
    local is_touching_wall = GetScriptParameter(player_entity, "is_touching_wall")
    local is_dashing = GetScriptParameter(player_entity, "is_dashing")
    
    -- Update grounded indicator
    if is_grounded then
        UI.show_element(hud_instance_name .. "/grounded_indicator")
    else
        UI.hide_element(hud_instance_name .. "/grounded_indicator")
    end
    
    -- Update wall slide indicator
    if is_touching_wall and not is_grounded then
        UI.show_element(hud_instance_name .. "/wall_slide_indicator")
    else
        UI.hide_element(hud_instance_name .. "/wall_slide_indicator")
    end
    
    -- Update dashing indicator
    if is_dashing then
        UI.show_element(hud_instance_name .. "/dashing_indicator")
    else
        UI.hide_element(hud_instance_name .. "/dashing_indicator")
    end
end

function UpdateDashIndicator()
    -- Get player entity
    local player_entity = GetEntityByTag("Player")
    if not player_entity then
        return
    end
    
    -- Get dash state from script
    local can_dash = GetScriptParameter(player_entity, "can_dash")
    
    -- Update dash text
    if can_dash then
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Ready")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.3, g=0.8, b=1.0, a=1.0})
    else
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Used")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.5, g=0.5, b=0.5, a=0.7})
    end
end

-- Helper functions (these should be provided by the engine)
function GetEntityByTag(tag)
    -- This is a placeholder - engine should provide this
    -- For now, we know Player is entity 11
    if tag == "Player" then
        return 11
    end
    return nil
end

function GetTransform(entity)
    -- Placeholder - engine should provide this
    return nil
end

function GetVelocity(entity)
    -- Placeholder - engine should provide this
    return nil
end

function GetScript(entity)
    -- Placeholder - engine should provide this
    return nil
end

function GetScriptParameter(entity, param_name)
    -- Placeholder - engine should provide this
    return nil
end

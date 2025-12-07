-- Real HUD Controller
-- Updates UI with actual game data from player

local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"

local frame_count = 0
local elapsed_time = 0
local hud_loaded = false
local player_entity = nil

-- Game constants (from player_controller)
local MAX_HEALTH = 100
local MAX_STAMINA = 100
local MAX_DASHES = 1

function Start()
    log("=== HUD Controller Real: Starting ===")
    
    -- Load and activate HUD
    UI.load_prefab(hud_prefab_path)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    
    hud_loaded = true
    log("=== HUD Controller Real: HUD Loaded ===")
    
    -- Find player entity (entity 11 based on scene)
    player_entity = 11
    log("Player entity: " .. player_entity)
    
    -- Set initial values
    UI.set_image_fill(hud_instance_name .. "/player_health_fill", 1.0)
    UI.set_image_fill(hud_instance_name .. "/stamina_bar_fill", 1.0)
    UI.set_color(hud_instance_name .. "/player_health_fill", {r=0.2, g=1.0, b=0.3, a=1.0})
    
    -- Hide indicators initially
    UI.hide_element(hud_instance_name .. "/dashing_indicator")
    UI.hide_element(hud_instance_name .. "/grounded_indicator")
    UI.hide_element(hud_instance_name .. "/wall_slide_indicator")
    
    log("=== HUD Controller Real: Initialization Complete ===")
end

function Update(dt)
    if not hud_loaded or not player_entity then
        log("HUD not loaded or player not found")
        return
    end
    
    -- Update FPS counter
    UpdateFPS(dt)
    
    -- Get player transform
    local transform = GetTransform(player_entity)
    if transform then
        -- Update position debug
        UI.set_text(hud_instance_name .. "/position_debug", 
            string.format("X: %.1f Y: %.1f", transform.x, transform.y))
    else
        log("ERROR: GetTransform returned nil for entity " .. player_entity)
    end
    
    -- Get player velocity from rigidbody
    local velocity = GetVelocity(player_entity)
    if velocity then
        -- Update velocity debug
        UI.set_text(hud_instance_name .. "/velocity_debug", 
            string.format("VX: %.1f VY: %.1f", velocity.x, velocity.y))
    end
    
    -- Get player script parameters
    local is_grounded = GetScriptParameter(player_entity, "is_grounded")
    local is_dashing = GetScriptParameter(player_entity, "is_dashing")
    local can_dash = GetScriptParameter(player_entity, "can_dash")
    local is_touching_wall = GetScriptParameter(player_entity, "is_touching_wall")
    local wall_direction = GetScriptParameter(player_entity, "wall_direction")
    
    -- Update grounded indicator
    if is_grounded and is_grounded.Bool then
        UI.show_element(hud_instance_name .. "/grounded_indicator")
    else
        UI.hide_element(hud_instance_name .. "/grounded_indicator")
    end
    
    -- Update dashing indicator
    if is_dashing and is_dashing.Bool then
        UI.show_element(hud_instance_name .. "/dashing_indicator")
        UI.set_text(hud_instance_name .. "/dashing_indicator", "DASHING!")
    else
        UI.hide_element(hud_instance_name .. "/dashing_indicator")
    end
    
    -- Update dash availability
    if can_dash and can_dash.Bool then
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Ready")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.2, g=1.0, b=0.3, a=1.0})
    else
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Used")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.5, g=0.5, b=0.5, a=1.0})
    end
    
    -- Update wall slide indicator
    if is_touching_wall and is_touching_wall.Bool then
        UI.show_element(hud_instance_name .. "/wall_slide_indicator")
        local direction_text = "Wall: "
        if wall_direction and wall_direction.Int then
            if wall_direction.Int > 0 then
                direction_text = direction_text .. "Right"
            elseif wall_direction.Int < 0 then
                direction_text = direction_text .. "Left"
            else
                direction_text = direction_text .. "None"
            end
        end
        UI.set_text(hud_instance_name .. "/wall_slide_indicator", direction_text)
    else
        UI.hide_element(hud_instance_name .. "/wall_slide_indicator")
    end
    
    -- Update health bar (for now use stamina as proxy, or set to full)
    -- In real game, you'd have health parameter in player script
    UI.set_image_fill(hud_instance_name .. "/player_health_fill", 1.0)
    UI.set_color(hud_instance_name .. "/player_health_fill", {r=0.2, g=1.0, b=0.3, a=1.0})
    
    -- Update stamina bar (use can_dash as proxy - full when can dash, empty when can't)
    if can_dash and can_dash.Bool then
        UI.set_image_fill(hud_instance_name .. "/stamina_bar_fill", 1.0)
    else
        UI.set_image_fill(hud_instance_name .. "/stamina_bar_fill", 0.0)
    end
end

function UpdateFPS(dt)
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    
    -- Update every 0.5 seconds
    if elapsed_time >= 0.5 then
        local fps = math.floor(frame_count / elapsed_time)
        UI.set_text(hud_instance_name .. "/fps_counter", "FPS: " .. fps)
        
        frame_count = 0
        elapsed_time = 0
    end
end

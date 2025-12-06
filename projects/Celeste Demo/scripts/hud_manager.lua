-- HUD Manager for Celeste Demo
-- Manages HUD state and updates data bindings

local HudManager = {}

-- Initialize HUD system
function HudManager.init()
    print("HUD Manager initialized")
    
    -- HUD state
    HudManager.player_health = 1.0  -- 0.0 to 1.0
    HudManager.player_stamina = 1.0
    HudManager.dash_count = 1
    HudManager.is_grounded = false
    HudManager.is_wall_sliding = false
    HudManager.is_dashing = false
    HudManager.fps = 60
    
    -- Position tracking
    HudManager.pos_x = 0
    HudManager.pos_y = 0
    HudManager.vel_x = 0
    HudManager.vel_y = 0
end

-- Update HUD from player state
function HudManager.update(player_entity, world, dt)
    if not player_entity then return end
    
    -- Get player transform
    local transform = world:get_transform(player_entity)
    if transform then
        HudManager.pos_x = math.floor(transform.position[1] * 10) / 10
        HudManager.pos_y = math.floor(transform.position[2] * 10) / 10
    end
    
    -- Get player velocity
    local velocity = world:get_velocity(player_entity)
    if velocity then
        HudManager.vel_x = math.floor(velocity[1] * 10) / 10
        HudManager.vel_y = math.floor(velocity[2] * 10) / 10
    end
    
    -- Get player script parameters
    local script = world:get_script(player_entity)
    if script then
        -- Grounded state
        local is_grounded = script:get_param("is_grounded")
        if is_grounded ~= nil then
            HudManager.is_grounded = is_grounded
        end
        
        -- Wall sliding
        local is_touching_wall = script:get_param("is_touching_wall")
        if is_touching_wall ~= nil then
            HudManager.is_wall_sliding = is_touching_wall
        end
        
        -- Dashing
        local is_dashing = script:get_param("is_dashing")
        if is_dashing ~= nil then
            HudManager.is_dashing = is_dashing
        end
        
        -- Dash count (can_dash = has dash available)
        local can_dash = script:get_param("can_dash")
        if can_dash ~= nil then
            HudManager.dash_count = can_dash and 1 or 0
        end
    end
    
    -- Update stamina (decreases when wall sliding)
    if HudManager.is_wall_sliding then
        HudManager.player_stamina = math.max(0, HudManager.player_stamina - dt * 0.3)
    else
        HudManager.player_stamina = math.min(1.0, HudManager.player_stamina + dt * 0.5)
    end
    
    -- Calculate FPS
    if dt > 0 then
        HudManager.fps = math.floor(1.0 / dt)
    end
end

-- Get data for HUD bindings
function HudManager.get_player_health()
    return HudManager.player_health
end

function HudManager.get_player_stamina()
    return HudManager.player_stamina
end

function HudManager.get_dash_count()
    return HudManager.dash_count
end

function HudManager.get_pos_x()
    return HudManager.pos_x
end

function HudManager.get_pos_y()
    return HudManager.pos_y
end

function HudManager.get_vel_x()
    return HudManager.vel_x
end

function HudManager.get_vel_y()
    return HudManager.vel_y
end

function HudManager.get_fps()
    return HudManager.fps
end

function HudManager.is_player_grounded()
    return HudManager.is_grounded
end

function HudManager.is_player_wall_sliding()
    return HudManager.is_wall_sliding
end

function HudManager.is_player_dashing()
    return HudManager.is_dashing
end

-- Damage player (for testing)
function HudManager.damage_player(amount)
    HudManager.player_health = math.max(0, HudManager.player_health - amount)
    print("Player health: " .. (HudManager.player_health * 100) .. "%")
end

-- Heal player (for testing)
function HudManager.heal_player(amount)
    HudManager.player_health = math.min(1.0, HudManager.player_health + amount)
    print("Player health: " .. (HudManager.player_health * 100) .. "%")
end

return HudManager

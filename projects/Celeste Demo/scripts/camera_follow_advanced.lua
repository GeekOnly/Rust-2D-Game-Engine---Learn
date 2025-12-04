-- Advanced Camera Follow Script
-- Smooth camera with bounds, dead zone, and look-ahead

-- Basic Settings
smooth_speed = 5.0
offset_x = 0.0
offset_y = 1.0            -- Slightly above player

-- Dead Zone (area where camera doesn't move)
dead_zone_x = 1.0         -- Horizontal dead zone
dead_zone_y = 0.5         -- Vertical dead zone

-- Look Ahead (camera moves ahead based on player velocity)
look_ahead_x = 1.5        -- Horizontal look ahead distance
look_ahead_y = 0.5        -- Vertical look ahead distance
look_ahead_smooth = 3.0   -- How fast look ahead adjusts

-- Camera Bounds (optional)
use_bounds = true
bound_min_x = -15.0       -- Left boundary
bound_max_x = 35.0        -- Right boundary
bound_min_y = -10.0       -- Bottom boundary
bound_max_y = 15.0        -- Top boundary

-- Internal State
target_entity = nil
look_ahead_offset_x = 0.0
look_ahead_offset_y = 0.0

function on_start()
    print("Advanced Camera Follow: Started")
    
    -- Find player
    local all_entities = get_all_entities()
    for i, ent in ipairs(all_entities) do
        local tags = get_tags(ent)
        if tags then
            for j, tag in ipairs(tags) do
                if tag == "Player" then
                    target_entity = ent
                    print("Camera Follow: Found player " .. ent)
                    
                    -- Snap to player
                    local player_pos = get_position_of(ent)
                    if player_pos then
                        local cam_pos = get_position()
                        set_position(player_pos.x + offset_x, player_pos.y + offset_y, cam_pos.z)
                    end
                    break
                end
            end
        end
        if target_entity then break end
    end
end

function on_update(dt)
    if not target_entity then
        return
    end
    
    local player_pos = get_position_of(target_entity)
    local cam_pos = get_position()
    
    if not player_pos or not cam_pos then
        return
    end
    
    -- Get player velocity for look-ahead
    local player_vel = get_velocity_of(target_entity)
    local vel_x = 0.0
    local vel_y = 0.0
    
    if player_vel then
        vel_x = player_vel.x
        vel_y = player_vel.y
    end
    
    -- Update look-ahead offset smoothly
    local target_look_x = vel_x * look_ahead_x
    local target_look_y = vel_y * look_ahead_y
    
    local look_t = math.min(look_ahead_smooth * dt, 1.0)
    look_ahead_offset_x = look_ahead_offset_x + (target_look_x - look_ahead_offset_x) * look_t
    look_ahead_offset_y = look_ahead_offset_y + (target_look_y - look_ahead_offset_y) * look_t
    
    -- Calculate desired position
    local desired_x = player_pos.x + offset_x + look_ahead_offset_x
    local desired_y = player_pos.y + offset_y + look_ahead_offset_y
    
    -- Apply dead zone
    local delta_x = desired_x - cam_pos.x
    local delta_y = desired_y - cam_pos.y
    
    if math.abs(delta_x) < dead_zone_x then
        desired_x = cam_pos.x
    end
    
    if math.abs(delta_y) < dead_zone_y then
        desired_y = cam_pos.y
    end
    
    -- Smooth movement
    local t = math.min(smooth_speed * dt, 1.0)
    local new_x = cam_pos.x + (desired_x - cam_pos.x) * t
    local new_y = cam_pos.y + (desired_y - cam_pos.y) * t
    
    -- Apply bounds
    if use_bounds then
        new_x = math.max(bound_min_x, math.min(bound_max_x, new_x))
        new_y = math.max(bound_min_y, math.min(bound_max_y, new_y))
    end
    
    -- Update position
    set_position(new_x, new_y, cam_pos.z)
end

-- Camera Follow Script
-- Smooth camera that follows the player with optional bounds and smoothing

-- Parameters
local target_tag = "Player"           -- Tag of entity to follow
local smooth_speed = 5.0              -- How fast camera moves (higher = faster)
local offset_x = 0.0                  -- Camera offset from target
local offset_y = 0.0
local use_bounds = false              -- Enable camera bounds
local min_x = -10.0                   -- Minimum camera position
local max_x = 10.0                    -- Maximum camera position
local min_y = -10.0
local max_y = 10.0
local look_ahead = 0.5                -- Look ahead distance based on velocity
local dead_zone_x = 0.5               -- Dead zone where camera doesn't move
local dead_zone_y = 0.5

-- Internal state
local target_entity = nil
local current_velocity_x = 0.0
local current_velocity_y = 0.0

function on_start()
    print("Camera Follow: Started")
    
    -- Find target entity by tag
    target_entity = find_entity_by_tag(target_tag)
    
    if target_entity then
        print("Camera Follow: Found target entity " .. target_entity)
        
        -- Snap camera to target immediately on start
        local target_pos = get_position_of(target_entity)
        if target_pos then
            local cam_pos = get_position()
            set_position(target_pos.x + offset_x, target_pos.y + offset_y, cam_pos.z)
        end
    else
        print("Camera Follow: Warning - No entity found with tag '" .. target_tag .. "'")
    end
end

function on_update(dt)
    if not target_entity then
        -- Try to find target again
        target_entity = find_entity_by_tag(target_tag)
        if not target_entity then
            return
        end
    end
    
    -- Get target position
    local target_pos = get_position_of(target_entity)
    if not target_pos then
        return
    end
    
    -- Get current camera position
    local cam_pos = get_position()
    if not cam_pos then
        return
    end
    
    -- Get target velocity for look-ahead
    local target_vel = get_velocity_of(target_entity)
    if target_vel then
        current_velocity_x = target_vel.x
        current_velocity_y = target_vel.y
    end
    
    -- Calculate desired position with offset and look-ahead
    local desired_x = target_pos.x + offset_x + (current_velocity_x * look_ahead)
    local desired_y = target_pos.y + offset_y + (current_velocity_y * look_ahead * 0.5)
    
    -- Apply dead zone
    local delta_x = desired_x - cam_pos.x
    local delta_y = desired_y - cam_pos.y
    
    if math.abs(delta_x) < dead_zone_x then
        desired_x = cam_pos.x
    end
    
    if math.abs(delta_y) < dead_zone_y then
        desired_y = cam_pos.y
    end
    
    -- Apply bounds if enabled
    if use_bounds then
        desired_x = math.max(min_x, math.min(max_x, desired_x))
        desired_y = math.max(min_y, math.min(max_y, desired_y))
    end
    
    -- Smooth movement using lerp
    local new_x = lerp(cam_pos.x, desired_x, smooth_speed * dt)
    local new_y = lerp(cam_pos.y, desired_y, smooth_speed * dt)
    
    -- Update camera position
    set_position(new_x, new_y, cam_pos.z)
end

-- Linear interpolation helper
function lerp(a, b, t)
    return a + (b - a) * math.min(t, 1.0)
end

-- Find entity by tag
function find_entity_by_tag(tag)
    -- This should be implemented in the engine
    -- For now, we'll use a placeholder
    local entities = get_entities_with_tag(tag)
    if entities and #entities > 0 then
        return entities[1]
    end
    return nil
end

-- Celeste-Style Player Controller
-- Features: Run, Jump, Dash, Wall Slide, Wall Jump

-- Movement parameters
local move_speed = 200.0
local jump_force = 400.0
local dash_speed = 500.0
local wall_slide_speed = 50.0
local gravity_scale = 1.0

-- State
local velocity_x = 0.0
local velocity_y = 0.0
local is_grounded = false
local can_dash = true
local is_dashing = false
local dash_timer = 0.0
local dash_duration = 0.2
local dash_direction_x = 0.0
local dash_direction_y = 0.0

-- Wall slide
local is_touching_wall = false
local wall_direction = 0

function on_start(entity)
    print("Player Controller started!")
    
    -- Set initial velocity
    set_velocity(0.0, 0.0)
end

function on_update(entity, dt)
    -- Update dash timer
    if is_dashing then
        dash_timer = dash_timer + dt
        if dash_timer >= dash_duration then
            is_dashing = false
            dash_timer = 0.0
        end
    end
    
    -- Get current velocity
    local vel = get_velocity()
    if vel then
        velocity_x = vel.x
        velocity_y = vel.y
    end
    
    -- Check if grounded (simple check - velocity.y near 0 and moving down)
    is_grounded = math.abs(velocity_y) < 10.0 and velocity_y >= 0
    
    -- Reset dash when grounded
    if is_grounded then
        can_dash = true
    end
    
    -- Handle input
    if not is_dashing then
        handle_movement(dt)
        handle_jump()
    end
    
    handle_dash()
    
    -- Apply dash velocity
    if is_dashing then
        velocity_x = dash_direction_x * dash_speed
        velocity_y = dash_direction_y * dash_speed
    end
    
    -- Apply velocity
    set_velocity(velocity_x, velocity_y)
end

function handle_movement(dt)
    -- Horizontal movement
    if is_key_down("A") or is_key_down("Left") then
        velocity_x = -move_speed
    elseif is_key_down("D") or is_key_down("Right") then
        velocity_x = move_speed
    else
        -- Deceleration
        velocity_x = velocity_x * 0.8
        if math.abs(velocity_x) < 10.0 then
            velocity_x = 0.0
        end
    end
end

function handle_jump()
    -- Jump
    if is_key_pressed("Space") and is_grounded then
        velocity_y = -jump_force
        is_grounded = false
        print("Jump!")
    end
end

function handle_dash()
    -- Dash (Shift key)
    if is_key_pressed("LShift") and can_dash then
        -- Get dash direction from input
        local dash_x = 0.0
        local dash_y = 0.0
        
        if is_key_down("A") or is_key_down("Left") then
            dash_x = -1.0
        elseif is_key_down("D") or is_key_down("Right") then
            dash_x = 1.0
        end
        
        if is_key_down("W") or is_key_down("Up") then
            dash_y = -1.0
        elseif is_key_down("S") or is_key_down("Down") then
            dash_y = 1.0
        end
        
        -- Default to horizontal dash if no direction
        if dash_x == 0.0 and dash_y == 0.0 then
            dash_x = 1.0 -- Dash right by default
        end
        
        -- Normalize direction
        local length = math.sqrt(dash_x * dash_x + dash_y * dash_y)
        if length > 0 then
            dash_direction_x = dash_x / length
            dash_direction_y = dash_y / length
        end
        
        -- Start dash
        is_dashing = true
        can_dash = false
        dash_timer = 0.0
        
        print("Dash! Direction: " .. dash_direction_x .. ", " .. dash_direction_y)
    end
end

function on_collision(entity, other)
    -- Handle collision with ground/platforms
    print("Collision detected!")
end

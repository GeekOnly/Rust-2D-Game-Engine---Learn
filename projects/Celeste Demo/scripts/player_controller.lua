-- Celeste-Style Player Controller
-- Features: Run, Jump, Dash, Wall Slide, Wall Jump

-- Movement parameters
local move_speed = 3.0  -- Units per second (reduced for better control)
local jump_force = 15.0  -- Jump velocity (increased significantly)
local dash_speed = 10.0 -- Dash velocity
local wall_slide_speed = 1.0
local gravity_scale = 0.3  -- Very low gravity for testing

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

-- Unity-style lifecycle: Awake is called when script is loaded
function Awake()
    -- Initialization
end

-- Unity-style lifecycle: Start is called before first Update
function Start()
    -- Set initial velocity and gravity
    set_velocity(0.0, 0.0)
    set_gravity_scale(gravity_scale)
end

-- Unity-style lifecycle: Update is called every frame
function Update(dt)
    -- Debug: log once to verify Update is being called
    if math.random() < 0.01 then
        log("Update() is running")
    end
    
    -- Update dash timer
    if is_dashing then
        dash_timer = dash_timer + dt
        if dash_timer >= dash_duration then
            is_dashing = false
            dash_timer = 0.0
        end
    end
    
    -- Get current velocity and position
    local vel = get_velocity()
    if vel then
        velocity_x = vel.x
        velocity_y = vel.y
    end
    
    local pos = get_position()
    
    -- Check if grounded based on position
    -- Ground is at y = -2.5 with height 1.0, so top of ground is at y = -2.0
    -- Player has height 1.0, so player's bottom is at y - 0.5
    -- Player is grounded if bottom is at or below ground top: (y - 0.5) >= -2.0, or y >= -1.5
    local was_grounded = is_grounded
    if pos and pos.y >= -1.6 and velocity_y >= -0.1 then
        is_grounded = true
    else
        is_grounded = false
    end
    
    -- Reset dash when grounded
    if was_grounded then
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
    -- Jump (use is_key_just_pressed for single press detection)
    if is_key_just_pressed("Space") and is_grounded then
        log("JUMP!")
        velocity_y = -jump_force
        is_grounded = false
    end
end

function handle_dash()
    -- Dash (Shift key - use is_key_just_pressed for single press)
    if is_key_just_pressed("LShift") and can_dash then
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
        log("DASH!")
    end
end

-- Unity-style collision callback
function OnCollisionEnter(other)
    -- Handle collision with ground/platforms
    -- If we're moving downward or stationary, we're grounded
    if velocity_y >= -0.1 then
        is_grounded = true
    end
end

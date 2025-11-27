-- Script: Script_21
-- Simple player movement script

-- Engine API Functions (provided by the game engine):
-- is_key_pressed(key) - Check if a key is pressed
-- set_velocity(vx, vy) - Set entity velocity
-- get_tag(entity) - Get entity tag
-- destroy_entity(entity) - Destroy an entity
local let speed = 200.0  -- Movement speed in units per second

function on_start()
    -- Called when the game starts
    print("Script Script_21 started!")
end

function on_update(dt)
    -- Called every frame (dt is delta time in seconds)

    -- Player movement (WASD or Arrow keys)
    local vx = 0.0
    local vy = 0.0

    if is_key_pressed("W") or is_key_pressed("Up") then
        vy = vy - speed
    end
    if is_key_pressed("S") or is_key_pressed("Down") then
        vy = vy + speed
    end
    if is_key_pressed("A") or is_key_pressed("Left") then
        vx = vx - speed
    end
    if is_key_pressed("D") or is_key_pressed("Right") then
        vx = vx + speed
    end

    -- Normalize diagonal movement
    if vx ~= 0.0 and vy ~= 0.0 then
        local length = math.sqrt(vx * vx + vy * vy)
        vx = vx / length * speed
        vy = vy / length * speed
    end

    -- Set velocity
    set_velocity(vx, vy)
end

function on_collision(other_entity)
    -- Called when this entity collides with another
    print("Collision with entity: " .. tostring(other_entity))

    -- Example: Collect item
    local tag = get_tag(other_entity)
    if tag == "Item" then
        print("Collected item!")
        destroy_entity(other_entity)
    end
end

-- Simple test script to verify script system is working

function Awake()
    log("TEST SCRIPT: Awake() called!")
end

function Start()
    log("TEST SCRIPT: Start() called!")
end

function Update(dt)
    -- Log every 60 frames (about once per second at 60fps)
    if math.random() < 0.016 then
        log("TEST SCRIPT: Update() called - dt: " .. dt)
    end
end
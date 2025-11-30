# Jump Functionality Debug Guide

## ปัญหาปัจจุบัน
ตัวละครเคลื่อนที่ได้แต่กระโดดไม่ได้

## การวิเคราะห์ Scene

### Player Entity (ID: 1)
```json
Position: [0.0, 0.0, 0.0]
Scale: [1.0, 1.0, 1.0]
Rigidbody:
  - gravity_scale: 1.0 (ใน scene)
  - is_kinematic: false
Script: player_controller
  - gravity_scale parameter: 1.0 (ควรเป็น 0.3)
  - jump_force parameter: 400.0 (ควรเป็น 15.0)
```

### Ground Entity (ID: 7)
```json
Position: [0.0, -2.5, 0.0]
Scale: [8.0, 1.0, 1.0]
Rigidbody:
  - is_kinematic: true (ไม่ได้รับผลจาก physics)
```

## ปัญหาที่พบ

### 1. Player ไม่ได้อยู่บนพื้น
- Player อยู่ที่ y = 0.0
- พื้นอยู่ที่ y = -2.5
- ระยะห่าง = 2.5 units
- Player ควรตกลงมาจนกระทั่งชนพื้น

### 2. Script Parameters ไม่ตรงกับ Code
- Scene มี `gravity_scale: 1.0` แต่ script ตั้งเป็น `0.3`
- Scene มี `jump_force: 400.0` แต่ script ใช้ `15.0`
- Parameters ใน scene ถูก override โดย script ใน Start()

### 3. Grounded Detection อาจไม่ทำงาน
- ใช้ `math.abs(velocity_y) < 0.5` เพื่อตรวจสอบว่าอยู่บนพื้น
- ถ้า gravity ทำงาน velocity_y จะไม่เคยเป็น 0 เมื่ออยู่บนพื้น
- ต้องใช้ collision detection แทน

## วิธีแก้ไข

### วิธีที่ 1: ใช้ Collision Detection (แนะนำ)

ให้ OnCollisionEnter ตั้งค่า is_grounded:

```lua
function OnCollisionEnter(other)
    print("Collision! Entity: " .. tostring(other))
    is_grounded = true
end

function Update(dt)
    -- ไม่ต้อง reset is_grounded ทุกเฟรม
    -- ให้ physics system จัดการ
    
    if is_key_just_pressed("Space") and is_grounded then
        velocity_y = -jump_force
        is_grounded = false  -- Reset เมื่อกระโดด
    end
end
```

### วิธีที่ 2: ใช้ Position Check

ตรวจสอบว่า player อยู่ใกล้พื้นหรือไม่:

```lua
function Update(dt)
    local pos = get_position()
    
    -- ถ้า player อยู่ใกล้พื้น (y ≈ -2.0) และ velocity_y ≈ 0
    if pos and pos.y <= -1.9 and pos.y >= -2.1 and math.abs(velocity_y) < 0.5 then
        is_grounded = true
    else
        is_grounded = false
    end
end
```

### วิธีที่ 3: เพิ่ม Raycast (ในอนาคต)

```lua
-- ยิง ray ลงไปด้านล่าง player
local hit = raycast_down(0.6)  -- ตรวจสอบ 0.6 units ด้านล่าง
if hit then
    is_grounded = true
end
```

## การทดสอบ

### ขั้นตอนที่ 1: ตรวจสอบว่า Script ทำงาน
เพิ่ม debug messages:
```lua
function Start()
    print("=== Player Controller Started ===")
    print("Initial position: " .. tostring(get_position()))
    print("Initial velocity: " .. tostring(get_velocity()))
end

function Update(dt)
    -- Print ทุก 60 เฟรม
    if math.random() < 0.016 then
        local pos = get_position()
        local vel = get_velocity()
        print(string.format("Pos: (%.2f, %.2f), Vel: (%.2f, %.2f), Grounded: %s",
            pos.x, pos.y, vel.x, vel.y, tostring(is_grounded)))
    end
end
```

### ขั้นตอนที่ 2: ตรวจสอบ Collision
```lua
function OnCollisionEnter(other)
    print("=== COLLISION DETECTED ===")
    print("Other entity: " .. tostring(other))
    print("My velocity_y: " .. velocity_y)
    is_grounded = true
end
```

### ขั้นตอนที่ 3: ตรวจสอบ Jump Input
```lua
function handle_jump()
    if is_key_just_pressed("Space") then
        print("=== SPACE PRESSED ===")
        print("is_grounded: " .. tostring(is_grounded))
        print("velocity_y before: " .. velocity_y)
        
        if is_grounded then
            velocity_y = -jump_force
            is_grounded = false
            print("JUMP! velocity_y after: " .. velocity_y)
        else
            print("Cannot jump - not grounded")
        end
    end
end
```

## สิ่งที่ต้องตรวจสอบ

1. ✅ Script ถูก load หรือไม่? (ดูข้อความ "Player Controller: Awake() called")
2. ✅ Start() ถูกเรียกหรือไม่? (ดูข้อความ "Player Controller: Start() called")
3. ❓ Update() ถูกเรียกหรือไม่? (ควรเห็น debug messages)
4. ❓ Player ตกลงมาหาพื้นหรือไม่? (ดู position y)
5. ❓ Collision ถูกตรวจจับหรือไม่? (ดูข้อความ "Collision!")
6. ❓ is_grounded ถูกตั้งค่าหรือไม่?
7. ❓ Space key ถูกตรวจจับหรือไม่?

## แนวทางแก้ไขถัดไป

### ถ้า Update() ไม่ถูกเรียก
- ตรวจสอบว่า script engine ทำงานหรือไม่
- ตรวจสอบว่า entity มี script component หรือไม่
- ตรวจสอบว่า script.enabled = true หรือไม่

### ถ้า Collision ไม่ทำงาน
- ตรวจสอบว่า physics system ทำงานหรือไม่
- ตรวจสอบว่า collider มีขนาดถูกต้องหรือไม่
- ตรวจสอบว่า collision detection ถูกเรียกหรือไม่

### ถ้า is_grounded ไม่ถูกตั้งค่า
- ใช้ position check แทน velocity check
- เพิ่ม tolerance ให้มากขึ้น (เช่น < 1.0 แทน < 0.5)
- ใช้ collision callback แทน

## สรุป

ปัญหาหลักน่าจะเป็น:
1. **Grounded detection ไม่ทำงาน** - velocity_y ไม่เคยเป็น 0 เพราะ gravity
2. **Collision callback ไม่ถูกเรียก** - ต้องตรวจสอบว่า physics system ทำงานหรือไม่

แนะนำให้ใช้ **position-based grounded check** หรือ **collision callback** แทนการใช้ velocity check

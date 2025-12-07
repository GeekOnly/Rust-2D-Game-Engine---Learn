# วิธีทดสอบ UI Display

## ขั้นตอนการทดสอบ

1. **เปิด Celeste Demo project**
2. **กด Play** เพื่อเข้า Game View
3. **ดู Console** เพื่อดู log messages:
   - `✓ HUD prefab loaded successfully!`
   - `✓ HUD activated successfully!`
   - `UIManager::render called with 1 active UIs`

## สิ่งที่ควรเห็น

- Health Bar (สีเขียว) มุมซ้ายบน
- Stamina Bar (สีเหลือง) ใต้ Health Bar
- FPS Counter มุมขวาบน
- Debug Info (Position, Velocity)
- Controls Hint ด้านล่าง

## ถ้าไม่เห็น UI

ตรวจสอบ Console logs:
- ถ้าไม่มี "HUD prefab loaded" = ไฟล์ไม่ถูกโหลด
- ถ้าไม่มี "UIManager::render called" = render ไม่ถูกเรียก
- ถ้ามี logs แต่ไม่เห็น UI = ปัญหาการ render

## Debug Commands

เปิด Console และดู:
```
[INFO] ✓ HUD prefab loaded successfully!
[INFO] ✓ HUD activated successfully!
[DEBUG] UIManager::render called with 1 active UIs
[DEBUG] Rendering UI instance: celeste_hud
```

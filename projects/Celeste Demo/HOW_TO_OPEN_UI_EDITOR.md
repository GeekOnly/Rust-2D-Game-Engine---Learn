# 🎨 วิธีเปิด UI Prefab Editor

## ✅ การแก้ไขที่ทำแล้ว (COMPLETED!)

เพิ่มการเปิด UI Prefab Editor เมื่อ double-click ไฟล์ `.uiprefab` ใน Asset Browser

### สิ่งที่เพิ่มเข้ามา:
1. ✅ เพิ่ม `open_prefab_editor_request` field ใน EditorState
2. ✅ เพิ่ม `OpenUIPrefabEditor` action ใน AssetBrowser
3. ✅ เพิ่ม handler ใน main.rs ที่โหลด prefab และเปิดแท็บ PrefabEditor
4. ✅ ระบบจะ auto-create แท็บ PrefabEditor ถ้ายังไม่มี
5. ✅ แสดงข้อความใน Console เมื่อเปิดสำเร็จ

## 🎯 วิธีเปิด UI Editor

### วิธีที่ 1: Double-Click (แนะนำ)
```
1. เปิด "Project" tab (Asset Browser)
2. Navigate ไปที่ assets/ui/
3. Double-click ที่ celeste_hud.uiprefab
4. UI Prefab Editor จะเปิดขึ้นมา
```

### วิธีที่ 2: Context Menu
```
1. Right-click ที่ celeste_hud.uiprefab
2. เลือก "Open in UI Editor" (ถ้ามี)
```

### วิธีที่ 3: PrefabEditor Tab
```
1. คลิกที่ "PrefabEditor" tab ใน dock layout
2. ใช้ File > Open เพื่อเลือกไฟล์
```

## 🎨 UI Prefab Editor Features

### Canvas View
- แสดง preview ของ UI
- Drag & drop elements
- Visual editing

### Hierarchy Panel
- แสดง UI element tree
- Select elements
- Reorder elements

### Properties Panel
- แก้ไข RectTransform
- แก้ไข components (Image, Text, Button, etc.)
- Real-time preview

## 📝 การแก้ไข HUD

### 1. เปิด celeste_hud.uiprefab
```
Double-click: assets/ui/celeste_hud.uiprefab
```

### 2. แก้ไข Elements
```
- เลือก element จาก Hierarchy
- แก้ไข properties ใน Properties Panel
- ดู preview ใน Canvas
```

### 3. บันทึก
```
File > Save หรือ Ctrl+S
```

## 🔧 สิ่งที่แก้ไขได้

### RectTransform
- Anchor (top, bottom, left, right, center)
- Pivot
- Position
- Size

### Image Component
- Sprite
- Color
- Image Type (Simple, Sliced, Filled)
- Fill Amount (สำหรับ health bars)

### Text Component
- Text content
- Font
- Font Size
- Color
- Alignment

### Button Component
- Normal/Highlighted/Pressed colors
- On Click callback

## ⚠️ หมายเหตุ

### ปัจจุบัน
- UI Prefab Editor มีอยู่แล้วใน engine
- Double-click จะโหลด prefab และเปิด editor
- แก้ไขได้แบบ visual

### ข้อจำกัด
- ~~ยังไม่มี tab switching อัตโนมัติ~~ ✅ แก้ไขแล้ว!
- ~~ต้องเปิด PrefabEditor tab manually~~ ✅ เปิดอัตโนมัติแล้ว!
- บาง features อาจจะยังไม่สมบูรณ์

## 🚀 การใช้งาน

### ตัวอย่าง: แก้ไข Health Bar
```
1. Double-click celeste_hud.uiprefab
2. เลือก "player_health_fill" จาก Hierarchy
3. แก้ไข Color ใน Properties Panel
4. แก้ไข Fill Amount
5. Save
6. กด Play เพื่อดูผลลัพธ์
```

### ตัวอย่าง: เพิ่ม Text Element
```
1. เปิด celeste_hud.uiprefab
2. Right-click ใน Hierarchy
3. Add > Text
4. ตั้งค่า RectTransform (anchor, position, size)
5. ตั้งค่า Text (content, font, color)
6. Save
```

## ✅ สรุป

- ✅ Double-click `.uiprefab` เพื่อเปิด editor
- ✅ แก้ไขได้แบบ visual
- ✅ Real-time preview
- ✅ Save และ reload ได้

**ลองเปิด UI Editor เลย!** 🎨✨

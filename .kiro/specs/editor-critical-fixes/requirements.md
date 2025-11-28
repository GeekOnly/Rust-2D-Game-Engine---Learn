# Editor Critical Fixes - Requirements

## Overview
แก้ไขปัญหาสำคัญของ editor และเพิ่มระบบ sprite/tilemap สำหรับ production

## Acceptance Criteria

### AC1: Camera Persistence
**Given** ผู้ใช้ปรับตำแหน่ง/การตั้งค่า camera ใน scene view
**When** บันทึก scene
**Then** การตั้งค่า camera ทั้งหมดถูกบันทึกและโหลดกลับมาได้ถูกต้อง

### AC2: Local Space Gizmo Movement
**Given** ผู้ใช้เลือก Local space mode
**When** ลาก gizmo ด้วย mouse
**Then** object เคลื่อนที่ตามแกนท้องถิ่นของมันเองและตาม mouse cursor อย่างถูกต้อง

### AC3: World Space Gizmo Movement
**Given** ผู้ใช้เลือก World space mode
**When** ลาก gizmo ด้วย mouse
**Then** object เคลื่อนที่ตามแกน world และ gizmo ทำงานได้ปกติ

### AC4: Scene View Navigation
**Given** ผู้ใช้อยู่ใน scene view
**When** ใช้ mouse wheel สำหรับ zoom หรือ middle-click drag สำหรับ pan
**Then** camera zoom และ pan ทำงานได้อย่างราบรื่น

### AC5: Camera Axis Correction
**Given** scene view แสดง camera gizmo
**When** ดูแกน camera
**Then** 
- แกนบน-ล่างถูกต้อง (ไม่สลับกัน)
- Camera gizmo ไม่แสดงใน game view
- แสดงเฉพาะใน scene view เท่านั้น

### AC6: Scale Gizmo Functionality
**Given** ผู้ใช้เลือก scale mode
**When** ลาก scale gizmo
**Then** object ปรับขนาดตามแกนที่เลือกได้อย่างถูกต้อง

### AC7: Sprite and Tilemap System
**Given** ผู้ใช้ต้องการสร้าง 2D level
**When** import sprite sheets และ tilemap data
**Then**
- รองรับ LDTK และ Tiled format
- สามารถปรับแต่ง collider สำหรับ sprite ได้
- มี sprite atlas/batching สำหรับ performance
- มี animation system สำหรับ sprite

### AC8: Gizmo Rotation with Object
**Given** object มีการหมุน (rotation)
**When** เลือก object และเปิด local space mode
**Then** gizmo หมุนตามการหมุนของ object

## Priority
- P0 (Critical): AC2, AC3, AC4, AC5, AC6, AC8
- P1 (High): AC1, AC7

## Dependencies
- ระบบ transform ที่มีอยู่
- ระบบ serialization สำหรับ scene
- Physics system สำหรับ collider integration

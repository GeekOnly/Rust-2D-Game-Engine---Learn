# Editor Critical Fixes - Tasks

## Phase 1: Critical Gizmo Fixes

### Task 1.1: Fix Local Space Movement Calculation
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 4 hours

**Requirements**: AC2
**Properties**: P2.1

**Description**:
แก้ไขการคำนวณ local space movement ให้ gizmo เคลื่อนที่ตาม mouse อย่างถูกต้อง

**Implementation**:
1. แก้ไข `calculate_local_movement()` ใน gizmo system
2. แปลง mouse delta เป็น world space ก่อน
3. Project ลงบน local axis ของ object
4. ทดสอบกับ object ที่หมุนในมุมต่างๆ

**Files**:
- `engine/src/editor/gizmo/transform.rs`
- `engine/src/editor/gizmo/interaction.rs`

---

### Task 1.2: Fix World Space Gizmo Interaction
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 3 hours

**Requirements**: AC3
**Properties**: P2.2

**Description**:
แก้ไข world space gizmo ให้สามารถลากได้

**Implementation**:
1. ตรวจสอบ ray casting สำหรับ gizmo picking
2. แก้ไข drag calculation ใน world space
3. ทดสอบการลากในทุกแกน

**Files**:
- `engine/src/editor/gizmo/interaction.rs`

---

### Task 1.3: Implement Scale Gizmo
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 5 hours

**Requirements**: AC6
**Properties**: P5.1

**Description**:
สร้างและแก้ไข scale gizmo ให้ทำงานได้

**Implementation**:
1. สร้าง scale gizmo rendering
2. Implement scale calculation จาก mouse drag
3. รองรับทั้ง uniform และ per-axis scaling
4. เพิ่ม visual feedback

**Files**:
- `engine/src/editor/gizmo/scale.rs` (new)
- `engine/src/editor/gizmo/mod.rs`

---

### Task 1.4: Add Gizmo Rotation with Object
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 3 hours

**Requirements**: AC8
**Properties**: P7.1

**Description**:
ทำให้ gizmo หมุนตาม object rotation ใน local space mode

**Implementation**:
1. แก้ไข gizmo rendering ให้รับ rotation
2. คำนวณ axis direction จาก object rotation
3. Update gizmo visual ตาม space mode

**Files**:
- `engine/src/editor/gizmo/rendering.rs`

---

## Phase 2: Navigation Fixes

### Task 2.1: Fix Zoom System
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 3 hours

**Requirements**: AC4
**Properties**: P3.1

**Description**:
แก้ไข zoom system ให้ zoom ไปที่ตำแหน่ง mouse

**Implementation**:
1. คำนวณ world position ที่ mouse
2. Adjust camera position หลัง zoom
3. ทดสอบ zoom in/out

**Files**:
- `engine/src/editor/camera.rs`
- `engine/src/editor/input.rs`

---

### Task 2.2: Fix Pan System
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 2 hours

**Requirements**: AC4
**Properties**: P3.2

**Description**:
แก้ไข pan system ให้ทำงานได้

**Implementation**:
1. Implement middle-click drag detection
2. แปลง mouse delta เป็น world space
3. Update camera position

**Files**:
- `engine/src/editor/camera.rs`
- `engine/src/editor/input.rs`

---

### Task 2.3: Fix Camera Axis Orientation
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 2 hours

**Requirements**: AC5
**Properties**: P4.1

**Description**:
แก้ไขแกน camera ให้ถูกต้อง (ไม่สลับบน-ล่าง)

**Implementation**:
1. ตรวจสอบ camera projection matrix
2. แก้ไข up vector ให้เป็น +Y
3. ทดสอบการแสดงผล

**Files**:
- `engine/src/editor/camera.rs`
- `engine/src/editor/rendering_3d.rs`

---

### Task 2.4: Hide Camera Gizmo in Game View
**Status**: TODO
**Priority**: P0
**Estimated Effort**: 1 hour

**Requirements**: AC5
**Properties**: P4.1

**Description**:
ซ่อน camera gizmo ใน game view

**Implementation**:
1. เพิ่ม view type check
2. Render camera gizmo เฉพาะใน scene view

**Files**:
- `engine/src/editor/ui/scene_view/mod.rs`

---

## Phase 3: Persistence

### Task 3.1: Add Camera State Serialization
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 4 hours

**Requirements**: AC1
**Properties**: P1.1

**Description**:
เพิ่มการบันทึก camera state ใน scene

**Implementation**:
1. สร้าง `EditorCameraState` struct
2. เพิ่ม serialization support
3. เพิ่มใน `SceneData`
4. Implement save/load

**Files**:
- `engine/src/editor/camera.rs`
- `engine/src/scene/serialization.rs`

---

## Phase 4: Sprite System

### Task 4.1: Create Sprite Sheet System
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 8 hours

**Requirements**: AC7
**Properties**: P6.1

**Description**:
สร้างระบบ sprite sheet และ atlas

**Implementation**:
1. สร้าง `SpriteSheet` และ `SpriteDefinition`
2. Implement sprite atlas packing
3. เพิ่ม sprite batching renderer
4. สร้าง sprite animation system

**Files**:
- `engine/src/sprite/mod.rs` (new)
- `engine/src/sprite/sheet.rs` (new)
- `engine/src/sprite/atlas.rs` (new)
- `engine/src/sprite/animation.rs` (new)

---

### Task 4.2: Implement LDTK Importer
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 6 hours

**Requirements**: AC7
**Properties**: P6.2

**Description**:
สร้าง LDTK file importer

**Implementation**:
1. Parse LDTK JSON format
2. แปลงเป็น internal tilemap format
3. Import layers และ entities
4. รองรับ auto-layers

**Files**:
- `engine/src/tilemap/ldtk.rs` (new)
- `engine/src/tilemap/mod.rs` (new)

---

### Task 4.3: Implement Tiled Importer
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 6 hours

**Requirements**: AC7
**Properties**: P6.2

**Description**:
สร้าง Tiled file importer

**Implementation**:
1. Parse Tiled TMX/JSON format
2. แปลงเป็น internal tilemap format
3. Import layers และ objects
4. รองรับ tile properties

**Files**:
- `engine/src/tilemap/tiled.rs` (new)

---

### Task 4.4: Create Sprite Collider Editor
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 8 hours

**Requirements**: AC7
**Properties**: P6.1

**Description**:
สร้าง editor สำหรับปรับแต่ง sprite colliders

**Implementation**:
1. สร้าง UI สำหรับ collider editing
2. รองรับ box, circle, polygon colliders
3. Implement auto-trace จาก sprite alpha
4. บันทึก collider data กับ sprite

**Files**:
- `engine/src/editor/ui/sprite_collider_editor.rs` (new)
- `engine/src/sprite/collider.rs` (new)

---

### Task 4.5: Integrate Sprite Physics
**Status**: TODO
**Priority**: P1
**Estimated Effort**: 4 hours

**Requirements**: AC7
**Properties**: P6.1

**Description**:
เชื่อมต่อ sprite colliders กับ physics system

**Implementation**:
1. แปลง sprite colliders เป็น physics colliders
2. Sync sprite transform กับ physics body
3. ทดสอบ collision detection

**Files**:
- `engine/src/sprite/physics.rs` (new)
- `engine/src/runtime/physics.rs`

---

## Testing Tasks

### Task T1: Gizmo Interaction Tests
**Status**: TODO
**Estimated Effort**: 3 hours

**Description**:
สร้าง tests สำหรับ gizmo interactions

**Files**:
- `engine/tests/gizmo_tests.rs` (new)

---

### Task T2: Camera Navigation Tests
**Status**: TODO
**Estimated Effort**: 2 hours

**Description**:
สร้าง tests สำหรับ camera navigation

**Files**:
- `engine/tests/camera_navigation_tests.rs` (new)

---

### Task T3: Sprite System Tests
**Status**: TODO
**Estimated Effort**: 4 hours

**Description**:
สร้าง tests สำหรับ sprite และ tilemap system

**Files**:
- `engine/tests/sprite_tests.rs` (new)
- `engine/tests/tilemap_tests.rs` (new)

---

## Summary

**Total Tasks**: 18
**Total Estimated Effort**: 72 hours

**By Phase**:
- Phase 1 (Gizmo Fixes): 15 hours
- Phase 2 (Navigation): 8 hours
- Phase 3 (Persistence): 4 hours
- Phase 4 (Sprite System): 32 hours
- Testing: 9 hours

**By Priority**:
- P0 (Critical): 23 hours
- P1 (High): 49 hours

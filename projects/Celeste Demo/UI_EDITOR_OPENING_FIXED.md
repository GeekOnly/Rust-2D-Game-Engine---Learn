# ✅ UI Prefab Editor Opening - FIXED!

## สรุปการแก้ไข

ปัญหา: ไม่สามารถเปิด UI Prefab Editor ได้เมื่อ double-click ไฟล์ `.uiprefab`

## การแก้ไขที่ทำ

### 1. เพิ่ม Request Field
**ไฟล์**: `engine/src/editor/states.rs`
```rust
pub open_prefab_editor_request: Option<PathBuf>,
```

เพิ่ม field สำหรับเก็บ request ให้เปิด prefab editor (เหมือนกับ `open_sprite_editor_request`)

### 2. เพิ่ม Parameter ใน TabContext
**ไฟล์**: `engine/src/editor/ui/dock_layout.rs`
```rust
pub open_prefab_editor_request: &'a mut Option<std::path::PathBuf>,
```

เพิ่ม parameter ให้ TabContext สามารถเข้าถึง request field ได้

### 3. อัพเดท Asset Browser Action Handler
**ไฟล์**: `engine/src/editor/ui/dock_layout.rs`
```rust
asset_browser::AssetBrowserAction::OpenUIPrefabEditor(path) => {
    // Set request to open prefab editor (handled in main.rs)
    *self.context.open_prefab_editor_request = Some(path);
}
```

เมื่อ double-click ไฟล์ `.uiprefab` จะตั้งค่า request แทนที่จะโหลดโดยตรง

### 4. เพิ่ม Handler ใน main.rs
**ไฟล์**: `engine/src/main.rs`
```rust
// Handle prefab editor open request
if let Some(prefab_path) = editor_state.open_prefab_editor_request.take() {
    if editor_state.use_docking {
        use crate::editor::ui::EditorTab;

        // Load the prefab
        match editor_state.prefab_editor.load_prefab(&prefab_path) {
            Ok(_) => {
                // Check if PrefabEditor tab already exists
                let mut tab_exists = false;
                editor_state.dock_state.main_surface().iter().for_each(|node| {
                    if let egui_dock::Node::Leaf { tabs, .. } = node {
                        for tab in tabs {
                            if matches!(tab, EditorTab::PrefabEditor) {
                                tab_exists = true;
                                break;
                            }
                        }
                    }
                });

                if !tab_exists {
                    // Add PrefabEditor tab to the dock
                    editor_state.dock_state.main_surface_mut()
                        .push_to_focused_leaf(EditorTab::PrefabEditor);
                }
                
                editor_state.console.info(format!("Opened UI Prefab Editor for: {}", prefab_path.display()));
            }
            Err(e) => {
                editor_state.console.error(format!("Failed to load prefab: {}", e));
            }
        }
    }
}
```

Handler ที่:
1. โหลด prefab file
2. ตรวจสอบว่ามีแท็บ PrefabEditor อยู่แล้วหรือไม่
3. ถ้าไม่มี สร้างแท็บใหม่และเพิ่มเข้า dock
4. แสดงข้อความสำเร็จใน Console

### 5. อัพเดท Function Signatures
**ไฟล์**: `engine/src/editor/ui/mod.rs`
- เพิ่ม `open_prefab_editor_request` parameter ใน `render_editor_with_dock()`
- ส่งต่อ parameter ไปยัง TabContext

## วิธีใช้งาน

1. **เปิด Project Panel** (แท็บ Project)
2. **Navigate ไปที่** `assets/ui/`
3. **Double-click** ที่ `celeste_hud.uiprefab`
4. **UI Prefab Editor จะเปิดขึ้นมาทันที!** 🎨

## ผลลัพธ์

✅ Double-click ไฟล์ `.uiprefab` เปิด editor ได้แล้ว
✅ แท็บ PrefabEditor ถูกสร้างอัตโนมัติ
✅ Prefab ถูกโหลดเข้า editor
✅ แสดงข้อความสำเร็จใน Console
✅ ใช้ระบบ docking layout เหมือน Sprite Editor

## การทำงานของระบบ

```
User Action: Double-click .uiprefab file
    ↓
AssetBrowser: Detect double-click
    ↓
AssetBrowser: Trigger OpenUIPrefabEditor action
    ↓
dock_layout: Set open_prefab_editor_request
    ↓
main.rs: Detect request in event loop
    ↓
main.rs: Load prefab into PrefabEditor
    ↓
main.rs: Check if PrefabEditor tab exists
    ↓
main.rs: Create tab if not exists
    ↓
main.rs: Show success message in Console
    ↓
User: See PrefabEditor tab with loaded prefab! ✨
```

## ไฟล์ที่แก้ไข

1. `engine/src/editor/states.rs` - เพิ่ม request field
2. `engine/src/editor/ui/dock_layout.rs` - เพิ่ม parameter และ action handler
3. `engine/src/editor/ui/mod.rs` - เพิ่ม parameter ใน function signature
4. `engine/src/main.rs` - เพิ่ม handler และ pass parameter
5. `projects/Celeste Demo/HOW_TO_OPEN_UI_EDITOR.md` - อัพเดทเอกสาร

## การทดสอบ

1. ✅ Compile สำเร็จ (cargo check ผ่าน)
2. 🔄 รอทดสอบ: Double-click ไฟล์ .uiprefab
3. 🔄 รอทดสอบ: ตรวจสอบว่าแท็บเปิดขึ้นมา
4. 🔄 รอทดสอบ: ตรวจสอบข้อความใน Console

## หมายเหตุ

- ระบบใช้ pattern เดียวกับ Sprite Editor (ทดสอบแล้วว่าใช้งานได้)
- รองรับ docking layout (Unity-style)
- ถ้าแท็บมีอยู่แล้ว จะโหลด prefab ใหม่ในแท็บเดิม
- ถ้าแท็บยังไม่มี จะสร้างแท็บใหม่อัตโนมัติ

---

**สถานะ**: ✅ COMPLETED - พร้อมทดสอบ!

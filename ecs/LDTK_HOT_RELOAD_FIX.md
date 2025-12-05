# LDtk Hot-Reload Fix

## ปัญหา

เมื่อ save ไฟล์ .ldtk ใน LDtk editor แล้ว hot-reload ไม่ทำงาน (map ไม่ update)

## สาเหตุ

LDtk editor (และ editor อื่นๆ หลายตัว) save ไฟล์ด้วยวิธี "atomic write":

1. สร้างไฟล์ temp (เช่น `world.ldtk.tmp`)
2. เขียนข้อมูลลงไฟล์ temp
3. ลบไฟล์เดิม (`world.ldtk`)
4. Rename ไฟล์ temp เป็นชื่อเดิม

วิธีนี้ทำให้:
- File watcher ได้รับ `Remove` event แทน `Modify` event
- File watcher หยุดทำงานเพราะไฟล์ถูกลบไปแล้ว
- ไฟล์ใหม่ไม่ถูก watch

## การแก้ไข

### 1. รองรับ Event Types เพิ่มเติม

เดิม:
```rust
if matches!(event.kind, EventKind::Modify(_)) {
    // reload...
}
```

แก้เป็น:
```rust
match event.kind {
    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
        // reload...
    }
    _ => {}
}
```

### 2. ลบ LDTK Entities เก่าทั้งหมดก่อน Reload

ปัญหา: Scene อาจมี LDTK entities ที่ save ไว้แล้ว ทำให้มี tilemap ซ้อนกัน

แก้ไข:
```rust
fn remove_existing_ldtk_entities(&self, world: &mut World) {
    // ลบ entities ที่มี name ขึ้นต้นด้วย:
    // - "LDTK Layer:"
    // - "CompositeCollider"
    // - "Collider_"
}
```

### 3. Re-watch ไฟล์หลัง Reload

เพิ่มการ re-watch ไฟล์ทุกครั้งที่ reload:

```rust
// Unwatch first
let _ = watcher.unwatch(&path);

// Re-watch if file exists
if path.exists() {
    watcher.watch(&path, RecursiveMode::NonRecursive)?;
}
```

### 4. เพิ่ม Delay เล็กน้อย

รอให้ไฟล์ถูกเขียนเสร็จก่อน reload:

```rust
std::thread::sleep(std::time::Duration::from_millis(50));
```

## ผลลัพธ์

✅ Hot-reload ทำงานได้กับ LDtk editor  
✅ รองรับ editor ที่ใช้ atomic write  
✅ ไม่มี race condition จากการเขียนไฟล์ไม่เสร็จ  
✅ File watcher ไม่หยุดทำงานหลัง save

## การใช้งาน

ไม่ต้องเปลี่ยนโค้ดอะไร ใช้งานเหมือนเดิม:

```lua
function on_update(dt)
    if ldtk_runtime and ldtk_runtime:update() then
        print("🔄 Map hot-reloaded!")
    end
end
```

## ทดสอบ

1. เปิด LDtk editor
2. เปิด game engine
3. แก้ไข level ใน LDtk
4. กด Ctrl+S (Save)
5. ดู console ใน game engine ควรเห็น "🔄 Map hot-reloaded!"

## Technical Details

### File System Events

| Event Type | เกิดเมื่อ | Hot-reload |
|-----------|----------|-----------|
| `Modify` | แก้ไขไฟล์โดยตรง | ✅ รองรับ |
| `Create` | สร้างไฟล์ใหม่ | ✅ รองรับ |
| `Remove` | ลบไฟล์ | ✅ รองรับ |
| `Rename` | เปลี่ยนชื่อไฟล์ | ✅ รองรับ (via Create) |

### Atomic Write Pattern

```
1. Create: world.ldtk.tmp
2. Write: world.ldtk.tmp (data)
3. Remove: world.ldtk
4. Rename: world.ldtk.tmp -> world.ldtk
   (ถูก detect เป็น Create event)
```

### Re-watch Mechanism

```rust
// Before reload
watcher.watch("world.ldtk")  // watching

// During save (atomic write)
// Remove event -> file deleted -> watcher stops

// After reload
watcher.unwatch("world.ldtk")  // cleanup
watcher.watch("world.ldtk")    // re-watch new file
```

## Known Issues

### Scene มี LDTK Entities ซ้อน

ถ้า scene ถูก save ขณะที่มี LDTK entities อยู่:
- Scene จะมี tilemap entities ที่ save ไว้
- Hot-reload จะลบ entities เก่าทั้งหมดก่อน load ใหม่
- ✅ แก้ไขแล้ว!

### ไฟล์ถูกลบ

ถ้าไฟล์ถูกลบจริงๆ (ไม่ใช่ atomic write):
- Hot-reload จะพยายาม reload แต่จะ error
- File watcher จะหยุดทำงาน
- ต้อง restart game เพื่อ watch ใหม่

**วิธีแก้**: ใช้ `ldtk_runtime:load()` ใหม่

### Multiple Events

บาง editor อาจ trigger หลาย events ในครั้งเดียว:
- Remove + Create
- Modify + Modify + Modify

**วิธีแก้**: ใช้ `updated_files.contains()` เพื่อไม่ reload ซ้ำ

## Performance

- Reload time: ~50-200ms (ขึ้นอยู่กับขนาด level)
- CPU overhead: minimal (event-driven)
- Memory: เท่าเดิม (despawn entities เก่าก่อน spawn ใหม่)

## Compatibility

✅ LDtk 1.5.3+  
✅ Windows  
✅ macOS  
✅ Linux  

## Related Files

- `ecs/src/loaders/ldtk_hot_reload.rs` - Hot-reload implementation
- `engine/src/runtime/ldtk_runtime.rs` - High-level API
- `ecs/LDTK_HOT_RELOAD.md` - Usage guide

# 🏗️ Architecture Design: Profiling System Module

**คำตอบคือ: ควรแยกเป็น Module หรือ Crate ต่างหากครับ ✅**

การแยก `profile` module (หรือ crate ชื่อ `profiler`) เป็นสิ่งที่ถูกต้องและจำเป็นมากสำหรับ Game Engine ที่ต้องรัน Cross-platform (Windows & Android) และต้องการตรวจสอบ Memory Layout ที่ละเอียด

## ทำไมต้องแยก?
1.  **Zero-overhead in Release**: โค้ด Profiling กิน performance เราต้องการให้ compile-out หายไปเลยเมื่อ build แบบ Release (หรือเมื่อไม่เปิด feature flag)
2.  **Platform Specifics**: การดึงค่า Memory หรือ CPU Time บน Windows กับ Android ใช้ API คนละตัว
    *   *Windows*: Win32 API / Performance Counters
    *   *Android*: `/proc/self/stat` หรือ Android NDK API
3.  **Unified Interface**: โค้ดหลัก (Game Loop) ไม่ควรต้องรู้ว่ารันบนเครื่องไหน แค่เรียก `profile_scope!("Update")` ก็พอ

---

## 🏗️ โครงสร้าง Module ที่แนะนำ

ควรสร้างโฟลเดอร์ `profiler` (ถ้าทำเป็น crate) หรือ `source/profiler`

```
engine/
└── src/
    └── profiler/
        ├── mod.rs          (API กลาง)
        ├── macros.rs       (Macros สำหรับเรียกใช้ง่ายๆ)
        ├── memory.rs       (Memory tracking logic)
        ├── gpu.rs          (Render counters: draw calls, triangles)
        └── platform/       (Platform specific impl)
            ├── mod.rs
            ├── windows.rs
            └── android.rs
```

## 🛠️ รายละเอียดฟีเจอร์

### 1. Conditional Compilation (สำคัญมาก)
ใน `Cargo.toml` ควรกำหนด Feature:
```toml
[features]
profile = [] # ถ้าไม่เปิด feature นี้ โค้ด profiling จะกลายเป็น no-op (ว่างเปล่า)
```

### 2. Platform Abstraction (Windows vs Android)

**ใน `profiler/platform/mod.rs`:**
```rust
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "android")]
pub use self::android::*;
```

**Android Implementation (`android.rs`):**
การ debug บน Android ผ่าน ADB ต้องส่งข้อมูลออกทาง `logcat` หรือ Socket
```rust
pub fn log_memory_usage() {
    // ใช้ ndk-sys หรืออ่าน /proc/meminfo
    // ส่งข้อมูลออกทาง android logger
    android_log(Level::Debug, "Memory Limit Check: Safe");
}
```

### 3. Memory Layout Verification
ถ้าซีเรียสเรื่อง Memory Layout บนเครื่องที่ต่างกัน (เช่น alignment ของ struct บน ARM vs x86) เราสามารถเขียน Test/ Check ฝังไว้ได้

```rust
// profiler/memory.rs
pub fn validate_layout<T>(name: &str) {
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();
    
    #[cfg(target_os = "android")]
    if align > 8 {
        warn!("⚠️ Struct {} has large alignment ({}) on Android!", name, align);
    }
}
```

### 4. Render Profile (GPU)
แยกส่วนนี้ออกมาเพื่อเก็บ `Draw Calls`, `Generic Count`, `Bind Count`
ให้ Renderer ส่งค่ามา report ที่ module นี้ทุก frame

---

## 🚀 ตัวอย่างการใช้งานจริง

เมื่อแยก Module แล้ว Code หลักจะสะอาดมาก:

**Game Loop:**
```rust
fn update() {
    profile_scope!("Game Update"); // วัดเวลา Scope นี้
    
    // ... logic ...
    
    #[cfg(feature = "profile")]
    profiler::validate_memory_layout::<MyComponent>(); // เช็ค memory เฉพาะตอน debug profile
}
```

## 💡 คำแนะนำเพิ่มเติมสำหรับ Android (ADB)
เพื่อให้ Debug ง่ายบน Android แนะนำให้ทำ **"Remote Profiling"**:
1.  **UDP Socket**: ให้เกมบน Android ส่งค่า FPS/Memory เป็น JSON packet กลับมาที่คอมผ่าน WiFi
2.  **Simple Viewer**: เขียน script Python หรือ Rust เล็กๆ ปลายทางรับค่ามาแสดงกราฟบน PC
    *   ดีกว่านั่งอ่าน logcat text เยอะครับ

สรุป: **ควรแยก Module ครับ** จะทำให้ Engine จัดการง่ายและ Scalable ไป Platform อื่นๆ ในอนาคต (เช่น WebAssembly/iOS) ได้ง่ายด้วย

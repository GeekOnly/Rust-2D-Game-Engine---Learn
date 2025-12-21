# 🚀 วิธีวัดผล Performance (Benchmarking & Profiling)

เพื่อให้รู้ว่าการปรับ Memory หรือ ECS ของเราทำให้ Performance ดีขึ้นจริงหรือไม่ เราต้องวัดผลด้วย **ตัวเลข** ไม่ใช่แค่ความรู้สึก
นี่คือเครื่องมือและวิธีการที่ควรใช้ใน Rust Game Engine

## 1. วัด FPS และ Frame Time (in-game)
วิธีพื้นฐานที่สุดคือดูว่า game loop วิ่งได้เร็วแค่ไหน

### วิธีการ:
เพิ่มตัวจับเวลาใน main loop เพื่อหา `delta_time`
```rust
// ใน main loop หรือ Editor UI
let frame_start = std::time::Instant::now();

// ... update & render ...

let duration = frame_start.elapsed();
let fps = 1.0 / duration.as_secs_f32();
let frame_ms = duration.as_secs_f32() * 1000.0;

println!("FPS: {:.2}, Frame Time: {:.2}ms", fps, frame_ms);
```
**การเปรียบเทียบ:**
*   ก่อนแก้: 60 FPS (16.6ms) ที่ 10,000 entities
*   หลังแก้: 120 FPS (8.3ms) ที่ 10,000 entities
> *ถ้าเฟรมไทม์ลดลง = CPU ทำงานน้อยลง = ดีขึ้น*

---

## 2. Micro-benchmarking ด้วย `criterion`
ใช้สำหรับวัดผล function หรือ logic เฉพาะจุด เช่น "การ allocate memory แบบ Pool เร็วกว่าแบบ Vec แค่ไหน"

### ติดตั้ง:
แก้ไฟล์ `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

### เขียน Benchmark (`benches/my_benchmark.rs`):
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_allocation(c: &mut Criterion) {
    c.bench_function("vec_push", |b| b.iter(|| {
        let mut v = Vec::new();
        for i in 0..1000 {
            v.push(black_box(i));
        }
    }));
    
    // เทียบกับ Custom Allocator ของเรา
    c.bench_function("arena_alloc", |b| b.iter(|| {
        let mut arena = MyArena::new();
        for i in 0..1000 {
            arena.alloc(black_box(i));
        }
    }));
}

criterion_group!(benches, bench_allocation);
criterion_main!(benches);
```
**รันคำสั่ง:** `cargo bench`
มันจะบอกเลยว่าเร็วกว่ากี่ % (เช่น "Improved by 45%")

---

## 3. Profiling ด้วย `puffin` หรือ `tracy` (Visual Profiler)
เพื่อให้เห็นภาพว่าเวลาเสียไปกับ function ไหนมากที่สุด (Rendering? Physics? Allocation?)

### แนะนำ: `puffin` + `puffin_egui`
ถ้าใช้ `egui` อยู่แล้ว `puffin` จะแสดงกราฟ flamegraph ในหน้าจอเกมได้เลย

### แนะนำ: `tracy` (Advance)
ถ้าต้องการดูละเอียดลึกระดับ memory allocation และ context switch
1. เพิ่ม dependency `tracy-client`
2. ใส่ macro `interval!("system_name")` ใน function ที่อยากวัด
3. เปิดโปรแกรม Tracy (แยกต่างหาก) แล้ว connect เข้ามา

---

## 4. วัด Memory Usage
การวัด Memory ใน Rust อาจจะยากหน่อยเพราะไม่มี GC มาให้ดูง่ายๆ

### วิธีง่ายสุด: OS Tools
*   **Windows**: Task Manager / Resource Monitor
*   **Mac**: Activity Monitor
ดูช่อง **Commit Size** หรือ **Private Bytes**

### วิธี Advance: Custom Tracking
ใน Custom Allocator ของเรา ให้ใส่ counter ไว้:
```rust
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

fn alloc(&self, layout: Layout) -> *mut u8 {
    ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
    // ... alloc logic ...
}
```
แล้ว print ค่า `ALLOCATED_BYTES` ออกมาดูเทียบกัน

---

## สรุป: จะรู้ได้ไงว่าดีขึ้น?

1.  **Frame Time ลดลง** (FPS สูงขึ้น) ใน Scene ที่มี Object เยอะๆ (Stress Test) -> *สำคัญสุด*
2.  **Allocation Count ลดลง** (ดูจาก Profiler) -> *ลดอาการกระตุก (stutter)*
3.  **Memory Usage ลดลง** หรือคงที่ ไม่เพิ่มขึ้นเรื่อยๆ (Memory Leak)
4.  **Benchmark Score ดีขึ้น** สำหรับ function ย่อยๆ

**แนะนำให้ทำ:** สร้าง **"Benchmark Scene"** ที่มี object 50,000 ตัว แล้วรันเปรียบเทียบก่อนและหลังแก้โค้ดครับ

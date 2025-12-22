# 🚀 Game Engine 2026: "The Future of Creation" - Strategic Roadmap

**Vision:** สร้าง Game Engine ที่ **"เร็วที่สุด, สวยที่สุด และใช้งานง่ายที่สุด"** สำหรับนักพัฒนาเกมอินดี้และสตูดิโอขนาดกลาง โดยใช้พลังของ Modern Rust + WebGPU เพื่อประสิทธิภาพระดับ Native และความสามารถในการรันบน Web Browser ได้อย่างไร้รอยต่อ

**Target Year:** 2026 (Ready for Commercial Release)

---

## 🏗️ Phase 1: The "Iron" Foundation (Q1-Q2 2025)
*Goal: ล้างหนี้ทางเทคนิค (Technical Debt) และสร้างรากฐานที่ Thread-safe และเสถียร 100%*

### 1.1 Architecture & Memory Safety (Critical)
- [ ] **Remove Unsafe Global State:** กำจัด `static mut` ใน `render_system.rs` และ Module อื่นๆ ทั้งหมด เปลี่ยนไปใช้ **ECS Resources** (`World::insert_resource`) หรือ `Arc<RwLock<T>>` เพื่อความปลอดภัยของ Memory
- [ ] **Parallel ECS Scheduler:** ปรับปรุงระบบ ECS ให้รัน System แบบขนาน (Multi-threaded) ได้จริง โดยใช้ Job System (เช่น `rayon` หรือ ECS native scheduler)
- [ ] **Unified AssetHandle System:** ยกเลิกการใช้ String ID เป็น Asset Key เปลี่ยนเป็น `AssetHandle<T>` (GUID/UUID) เพื่อรองรับการเปลี่ยนชื่อไฟล์, Hot-reloading และ Reference counting

### 1.2 Rendering Core Refactor
- [ ] **Render Graph (Frame Graph):** ยกเลิก Hardcoded Render Pass เปลี่ยนเป็นระบบ Node-based Render Graph เพื่อให้สามารถเพิ่ม/ลด Effect (SSAO, Bloom, Post-process) ได้โดยไม่ต้องแก้โค้ด Render Loop
- [ ] **Shader Material System:** รองรับ Shader Variants และ Material Instancing เพื่อลดการเปลี่ยน Pipeline State (ซึ่งช้า)

---

## 🎨 Phase 2: Visual Fidelity "Next-Gen" (Q3-Q4 2025)
*Goal: กราฟิกต้องสวย "Wow" เทียบชั้น Unity HDRP / Unreal ในสเกลอินดี้*

### 2.1 Advanced PBR & Lighting
- [ ] **Direct Lighting:** รองรับ Multiple Point/Spot Lights พร้อมเงาแบบ Cascaded Shadow Maps (CSM) สำหรับแสงอาทิตย์ และ PCF Soft Shadows
- [ ] **Indirect Lighting (GI):** เพิ่ม Image Based Lighting (IBL) สำหรับแสง Environment สะท้อนวัสดุโลหะ และพิจารณาใช้ Screen Space Global Illumination (SSGI)
- [ ] **Post-processing Stack:**
    - [ ] Bloom (Physically based)
    - [ ] Tone Mapping (ACES Filmic)
    - [ ] Color Grading (LUTs)
    - [ ] Depth of Field & Motion Blur

### 2.2 Particle & VFX
- [ ] **GPU Particle System:** ระบบ Particle ที่คำนวณบน Compute Shader รองรับ Particle นับหมื่น/แสนตัว
- [ ] **Decal System:** ระบบแปะลวดลายบนพื้นผิว (รอยกระสุน, คราบเลือด)

---

## 🧠 Phase 3: Physics & Simulation "Deep World" (Q1 2026)
*Goal: โลกเกมที่มีปฏิสัมพันธ์ได้จริง ไม่ใช่แค่ฉากแข็งๆ*

### 3.1 3D Physics Integration
- [ ] **Rapier3D Full Implementation:** เพิ่ม `rapier3d` เข้ามาคู่กับ 2D module
- [ ] **Character Controller 2.0:** ระบบควบคุมตัวละครที่เดินขึ้นบันได, ลาดเอียง และชนกำแพงไม่ติดขัด (Kinematic Character Controller)
- [ ] **Ragdoll Physics:** รองรับระบบฟิสิกส์สำหรับโมเดลตัวละครเมื่อตาย

### 3.2 Animation System
- [ ] **Skeletal Animation Blending:** ผสมท่าทาง (เช่น วิ่ง + ยิง) และ State Machine Transition (Walk -> Run -> Jump)
- [ ] **IK (Inverse Kinematics):** ระบบจัดท่าทางเท้าให้เหยียบพื้นตามความสูงจริง

---

## 🛠️ Phase 4: Developer Experience (Q2 2026)
*Goal: เครื่องมือที่ทำให้ Dev ทำงานเสร็จไวขึ้น 10 เท่า*

### 4.1 Editor Evolutions
- [ ] **Visual Shader Editor:** Node-based editor สำหรับสร้าง Shader (คล้าย Shader Graph)
- [ ] **Prefab Variant System:** สร้าง Prefab แม่ลูก ที่แก้ไขตัวแม่แล้วลูกเปลี่ยนตาม (Nested Prefabs)
- [ ] **Visual Scripting (Optional):** พิจารณาระบบ Node-based logic สำหรับ Logic ง่ายๆ

### 4.2 Debugging & Profiling
- [ ] **In-Game Profiler:** กราฟแสดง Frame time, CPU/GPU usage, Memory allocation แบบ Real-time
- [ ] **Frame Debugger:** เครื่องมือ Pause และดู Draw Call ทีละ Step เพื่อ Debug กราฟิก

---

## 🌐 Phase 5: Ecosystem & Platform "Best in Class" (Q3-Q4 2026)
*Goal: รองรับ Cross-Platform จริงจังและเตรียมพร้อมสู่ตลาด*

### 5.1 Platform Support
- [ ] **WebAssembly (WASM) Polish:** ปรับแต่งให้รันบน Browser ได้ลื่นไหล (Multi-threading on Web, Asset streaming)
- [ ] **Android/iOS Build Pipe:** ระบบ Export ลงมือถือแบบ One-click พร้อม Touch Input emulation

### 5.2 Community & Marketplace
- [ ] **Plugin System:** ออกแบบ API ให้คนอื่นเขียน Plugin เสริม Editor ได้ (เช่น Tool สร้าง Map, AI Generator)
- [ ] **Game Template:** มี Template เกม FPS, RPG, 2D Platformer ให้เริ่มโปรเจกต์ได้เลย

---

## ⚠️ Key Technologies to Master (Tech Stack 2026)
1.  **Rendering:** `wgpu` (WebGPU Standard) - ทันสมัยที่สุดและรองรับทุก Platform
2.  **Language:** Rust 2024 Edition (เมื่อออก) - เน้น Async และ Performance
3.  **Scripting:** `Luau` (Type-safe Lua from Roblox) หรือ `Rhai` - เพื่อความปลอดภัยและเร็ว
4.  **UI:** `egui` (Immediate Mode) สำหรับ Editor, `taffy` (Flexbox layout) สำหรับ In-game UI

---
**Summary for Immediate Action (Next Sprint):**
โฟกัสที่ **Phase 1.1** แก้ปัญหา `unsafe code` ใน `render_system.rs` ก่อน เพราะถ้ารากฐานไม่แข็งแรง ฟีเจอร์อื่นในอนาคตจะพังง่ายมาก

🌱 Vegetation Growth Animation System

Technical Specification v1.1 (Consolidated)

1. Objective

ระบบ Growth Animation สำหรับพืชแบบ procedural โดย:

ใช้ Vertex Animation (VAT) เท่านั้น

ไม่ใช้ skeleton / physics runtime

deterministic, network-safe

ประสิทธิภาพสูง (mobile → PC)

รองรับ instance จำนวนมาก

2. Supported Vegetation Types
Type	Core Behavior
Tree	Trunk → Branch → Leaf
Flower	Stem → Bud → Bloom
Vine	Segment-based climbing
3. Growth State Machine
enum class GrowthState {
  Seed,
  Growing,
  Blooming,
  Mature,
  Dormant,
  Dead
};


State เปลี่ยนได้ด้วยเวลา / event

Growth speed ปรับต่อ instance

รองรับ pause / resume / accelerate

4. Growth Phases (Authoring Time)
Phase	Description
Germination	จุดเริ่ม
Structural	โครงสร้างหลัก
Detail	ใบ / กลีบ
Mature	รูปร่างสมบูรณ์

ทุก phase ถูก bake เป็น VAT (multi-stage)

5. Per-Vertex Attributes (Required)
Attribute	Range	Usage
growth_weight	0–1	ลำดับการโต
wind_weight	0–1	ความอ่อน
rigidity	0–1	ต้านแรงลม
segment_index	int	vine
height_mask	0–1	tree
6. Vegetation Type Rules
6.1 Tree

Trunk โต bottom → top

Branch delay ตาม depth

Leaf spawn หลัง branch mature

Root optional (close-up)

6.2 Flower

Stem โต → Bud scale

Petal bloom ด้วย local rotation

Bloom curve แบบ non-linear

6.3 Vine

โตแบบเพิ่ม segment

ปลายเถาอ่อนสุด

Direction bias: surface normal + gravity

7. Growth Evaluation (Bake-Time Only)
growth = smoothstep(t - mask * delay)
final_pos = base_pos * growth


Noise / random ใช้เฉพาะตอน bake

Seed-based deterministic

8. Vertex Animation (VAT)
8.1 Encoding

Delta position from base mesh

Fixed topology required

delta = animated_pos - base_pos

8.2 Storage Layout (Preferred)
struct VatVertex {
  int16 dx, dy, dz;
};
index = frame * vertex_count + vertex_id;


Texture VAT เป็น optional fallback

9. Multi-Stage VAT

แบ่ง VAT ตาม phase:

Stage 0: structural

Stage 1: branch / stem

Stage 2: leaf / bloom

Runtime:

final = blend(stage0, stage1, stage2)

10. Runtime Playback
10.1 Shader Logic
frame = clamp(time * fps, 0, max_frame)
final_pos = base_pos + unpack(delta)

10.2 Instance Parameters
time_offset
growth_speed
variation_seed

11. Wind & Secondary Motion
final_pos += wind_offset * wind_weight * (1 - rigidity)


Wind แยกจาก growth

ปิดได้ตาม state

12. LOD & Optimization
LOD	Behavior
LOD0	Full VAT
LOD1	Reduced frames
LOD2	Static mesh

Growth complete → bake static

Frame decimation

Quantization profile per platform

13. ECS Integration
struct GrowthComponent {
  GrowthState state;
  float time;
  float speed;
};

struct VatComponent {
  BufferHandle vat;
};

14. Tooling
14.1 CLI Baker
vegvat bake plant.glb \
  --type tree \
  --fps 30 \
  --stages 3 \
  --format buffer

14.2 Validation

Topology check

Precision error threshold

Vertex / frame budget

15. Platform Profiles
Platform	Limits
Mobile	≤5k verts, ≤64 frames
PC	≤20k verts, ≤256 frames
Web	Buffer VAT preferred
16. Determinism & Networking

Growth driven by frame index

Seed-based variation

No runtime randomness

17. Deliverables

VAT baker tool

Runtime shader (WGSL / GLSL)

Sample assets (tree / flower / vine)

Engine integration example

🎯 Minimal Production Set
✔ Multi-stage VAT
✔ Per-vertex masks
✔ Growth state machine
✔ Wind blend
✔ LOD switching
✔ Static bake on mature
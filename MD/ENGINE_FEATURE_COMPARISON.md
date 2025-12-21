# Engine Feature Gap Analysis: XS Engine vs Tuanjie (China)

## Overview
Based on the **Tuanjie Engine** (Unity China 2024) features listed in `MD/PlanOtherFeature.md`, here is a breakdown of where our **XS Engine** stands and what is missing.

We are actually **Very Close** in terms of Architecture (ECS/GPU Driven), but missing the "Lighting" advancements.

## 1. Feature Comparison Table

| Feature | Tuanjie Engine | XS Engine Status | Notes |
| :--- | :--- | :--- | :--- |
| **Infinity Particles** | ECS + GPU Merge | ✅ **Designed** (`VFX_PARTICLE_SYSTEM_DESIGN`) | Our design uses Graph-to-Shader + Compute, similar technique. |
| **GPU Vegetation** | ECS + Indirect Draw | ✅ **Designed** (`TERRAIN_SYSTEM_DESIGN`) | We use `draw_indirect` + Compute Culling. Same tech. |
| **AnimGraph** | Retargeting / IK | ✅ **Designed** (`ANIMATION_SYSTEM_DESIGN`) | We have a State Machine plan. Need to implement IK. |
| **Streaming** | World Partition | ✅ **Designed** (`DYNAMIC_WORLD_SWAP`) | We plan to use Chunk Streaming + Offset. |
| **Virtual Shadows** | VSM (High Res) | ❌ **Missing** | We currently assume standard Cascaded Shadow Maps (CSM). |
| **Global Illumination** | Real-time Dynamic | ❌ **Missing** | We have no dynamic GI plan yet. |
| **Virtual Geometry** | Nanite-like | ❌ **Missing** | Very hard to implement. Not recommended for now. |
| **Mobile RayTracing** | Hybrid RT | ⚠️ **Partial** | We designed Audio RT. Visual RT is WIP. |

---

## 2. What Should We Add? (Recommendations)

To match the "AAA Mobile" quality of Tuanjie, we need to upgrade our **Lighting Pipeline**.

### Priority 1: Virtual Shadow Maps (VSM)
**Why:**
*   Standard Shadow Maps get pixelated in Open Worlds.
*   **VSM** caches shadows into a giant texture atlas (like 16k x 16k) but only updates pages that change.
*   *Result:* Sharp shadows everywhere, infinite distance.
*   *Difficulty:* Medium (Compute Shader Page Table).

### Priority 2: SDF Global Illumination (Mobile GI)
**Why:**
*   "TuanjieGI" likely uses probes or SDF.
*   We can generate **Signed Distance Fields (SDF)** for our meshes (Object space).
*   Ray march the SDFs in the shader to get Soft Shadows and Ambient Occlusion.
*   *Result:* Real-time bounce light without baking.
*   *Difficulty:* High.

### Priority 3: Virtual Geometry (Skip for now)
**Why:**
*   This requires "Mesh Shaders" or excessively complex Compute Cull pipelines (Meshlets).
*   *Verdict:* Stick to **CDLOD** (Continuous LOD) for terrain and standard LODs for props. It's proven and fast.

---

## 3. Conclusion

**Is `PlanOtherFeature.md` suitable?**
**Yes.** It is a perfect benchmark. We have already designed about **50%** of these features (The ECS/Simulation side).

**What to add to XS Engine:**
1.  **VSM (Virtual Shadow Maps)** - For the Open World look.
2.  **SDF Global Illumination** - For the "Next-Gen" lighting.

If we implement these two, we are effectively a "Mini Tuanjie" engine.

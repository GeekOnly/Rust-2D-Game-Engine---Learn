# ECS Performance Baseline

Benchmark results for the Custom HashMap-based ECS implementation.

**Date:** 2025-01-27
**Hardware:** (Add your hardware specs here)
**Compiler:** Rust 1.x (release mode with optimizations)

---

## 📊 Summary Table

| Operation | 100 Entities | 1,000 Entities | 10,000 Entities |
|-----------|--------------|----------------|-----------------|
| **Spawn** | 6.1 µs | 61.2 µs | 533.7 µs |
| **Insert Transform** | 7.8 µs | 93.1 µs | 894.5 µs |
| **Insert Multi (3 components)** | 17.5 µs | 199.9 µs | 2.44 ms |
| **Query Transform** | 198 ns | 2.24 µs | 23.0 µs |
| **Query Multi (Transform + Sprite)** | 986 ns | 11.3 µs | 202.8 µs |
| **Mutate Transform** | 9.1 µs | 138.5 µs | 938.9 µs |
| **Remove Component** | 9.7 µs | 124.8 µs | 1.19 ms |
| **Despawn (with 2 components)** | 31.5 µs | 338.7 µs | 4.45 ms |

---

## 🎮 Game Scenario Benchmark

**Realistic Game Loop** (1,000 entities, 60 frames simulation):
- Entities: 1,000 (with Transform, Sprite, Velocity)
- Simulation: 60 frames of physics update + render query
- **Total Time:** 2.41 ms
- **Per-frame average:** ~40 µs/frame
- **Theoretical FPS:** ~25,000 FPS (for this workload)

This means the ECS can handle **~1,000 active entities** at 60 FPS with plenty of headroom.

---

## 📈 Detailed Analysis

### 1. Spawn Performance
- **100 entities:** 6.1 µs → **61 ns/entity**
- **1,000 entities:** 61.2 µs → **61 ns/entity**
- **10,000 entities:** 533.7 µs → **53 ns/entity**

✅ **Good:** Consistent O(1) spawn time per entity
⚠️ **Note:** HashMap-based storage requires allocation per insert

---

### 2. Insert Component Performance

#### Single Component (Transform)
- **100 entities:** 7.8 µs → **78 ns/insert**
- **1,000 entities:** 93.1 µs → **93 ns/insert**
- **10,000 entities:** 894.5 µs → **89 ns/insert**

#### Multi-Component (Transform + Sprite + Collider)
- **100 entities:** 17.5 µs → **175 ns/entity**
- **1,000 entities:** 199.9 µs → **200 ns/entity**
- **10,000 entities:** 2.44 ms → **244 ns/entity**

✅ **Good:** Linear scaling with entity count
⚠️ **Concern:** 3x slowdown for multi-component insert (HashMap overhead)

---

### 3. Query Performance

#### Single Component Query (Transform)
- **100 entities:** 198 ns → **2 ns/entity**
- **1,000 entities:** 2.24 µs → **2.2 ns/entity**
- **10,000 entities:** 23.0 µs → **2.3 ns/entity**

#### Multi-Component Query (Transform + Sprite)
- **100 entities:** 986 ns → **9.9 ns/entity**
- **1,000 entities:** 11.3 µs → **11.3 ns/entity**
- **10,000 entities:** 202.8 µs → **20.3 ns/entity**

✅ **Excellent:** Very fast iteration (cache-friendly HashMap iteration)
⚠️ **Concern:** Multi-component query requires HashMap lookups (5-10x slower)

---

### 4. Mutation Performance

#### Mutate Transform (position update)
- **100 entities:** 9.1 µs → **91 ns/entity**
- **1,000 entities:** 138.5 µs → **138 ns/entity**
- **10,000 entities:** 938.9 µs → **94 ns/entity**

✅ **Good:** Similar to insert performance (expected for HashMap)

---

### 5. Remove/Despawn Performance

#### Remove Component
- **100 entities:** 9.7 µs → **97 ns/entity**
- **1,000 entities:** 124.8 µs → **125 ns/entity**
- **10,000 entities:** 1.19 ms → **119 ns/entity**

#### Despawn Entity (removes 2 components)
- **100 entities:** 31.5 µs → **315 ns/entity**
- **1,000 entities:** 338.7 µs → **339 ns/entity**
- **10,000 entities:** 4.45 ms → **445 ns/entity**

⚠️ **Concern:** Despawn is 3-4x slower than remove (due to multiple HashMap removals)

---

## 🔍 Performance Bottlenecks

### 1. **Multi-Component Queries** 🐌
- Current: 20.3 ns/entity (10k entities)
- Requires HashMap lookup for each component
- **Impact:** Systems that query multiple components are 5-10x slower

### 2. **Component Insertion** 🐌
- Current: 89-244 ns/insert depending on entity count
- HashMap allocation + hashing overhead
- **Impact:** Spawning prefabs with many components is slow

### 3. **Despawning Entities** 🐌
- Current: 445 ns/entity (10k entities)
- Must iterate and remove from each component HashMap
- **Impact:** Destroying many entities at once (e.g., clearing scene) is slow

---

## 🎯 Comparison with Other ECS Implementations

### Expected Performance (based on published benchmarks)

| ECS Library | Query (10k) | Insert (10k) | Notes |
|-------------|-------------|--------------|-------|
| **Custom (HashMap)** | 23.0 µs | 894.5 µs | Current implementation |
| **hecs** | ~5-8 µs | ~200-300 µs | Archetype-based, better cache locality |
| **Specs** | ~10-15 µs | ~400-600 µs | Component-based, similar to Custom |
| **Bevy ECS** | ~3-5 µs | ~100-200 µs | Archetype-based, highly optimized |
| **Legion** | ~4-6 µs | ~150-250 µs | Archetype-based |

**Estimated Speedup Potential:**
- Query: **2-5x faster** with archetype-based ECS
- Insert: **2-4x faster** with better memory layout
- Multi-component query: **5-10x faster** with archetypes

---

## 💡 Recommendations

### For Current Custom ECS:
1. ✅ **Keep it** - Good enough for small-medium games (<5,000 entities)
2. ⚠️ **Optimize if needed:**
   - Use `Vec` instead of `HashMap` for dense component storage
   - Add component type IDs for faster multi-component queries
   - Implement archetype-like grouping for common component combinations

### For Future Scaling:
1. 🚀 **Switch to hecs or Bevy ECS** if:
   - You need >10,000 active entities
   - You have many multi-component systems
   - Performance becomes a bottleneck

2. 📊 **Next Steps:**
   - Implement abstraction layer (trait-based)
   - Add hecs as optional backend
   - Compare performance side-by-side
   - Make data-driven decision

---

## 🔗 How to Run Benchmarks

```bash
# Run all benchmarks
cd ecs
cargo bench

# Run specific benchmark
cargo bench spawn

# Generate HTML report (requires gnuplot or uses plotters)
cargo bench --bench ecs_benchmark
# Open: target/criterion/report/index.html
```

---

## 📝 Benchmark Scenarios Explained

1. **spawn:** Create entities (no components)
2. **insert_transform:** Add Transform component to entities
3. **insert_multi_component:** Add Transform + Sprite + Collider
4. **query_transform:** Iterate all entities with Transform
5. **query_multi_component:** Iterate entities with Transform + Sprite
6. **mutate_transform:** Update Transform position for all entities
7. **remove_component:** Remove Transform from all entities
8. **despawn:** Destroy entities (removes all components)
9. **game_scenario:** Realistic game loop with physics + rendering

---

## 📌 Notes

- All benchmarks run in **release mode** with optimizations
- Uses `criterion` crate for statistical analysis
- Warm-up iterations ensure consistent measurements
- Outliers are detected and reported separately
- Results may vary based on hardware and system load

---

**Conclusion:** The current HashMap-based ECS is **good enough for prototyping and small-medium games**. For larger games or performance-critical applications, consider switching to a more optimized ECS library like **hecs** or **Bevy ECS** using the abstraction layer approach.

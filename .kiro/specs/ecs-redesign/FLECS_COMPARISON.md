# Flecs vs XS Engine ECS - Feature Comparison

## Overview

This document compares Flecs ECS features with our planned XS Engine ECS redesign and identifies features we should adopt.

---

## Feature Comparison Matrix

| Feature | Flecs | Bevy ECS | XS Engine v2 | Should Adopt? |
|---------|-------|----------|--------------|---------------|
| **Core Architecture** |
| Archetype storage | ✅ | ✅ | ✅ | ✅ Already planned |
| SoA (Struct of Arrays) | ✅ | ✅ | ✅ | ✅ Already planned |
| Sparse sets | ✅ | ✅ | ✅ | ✅ Already planned |
| Change detection | ✅ | ✅ | ✅ | ✅ Already planned |
| **Relationships** |
| Entity relationships | ✅ **Native** | ❌ Manual | ❌ Manual | 🟡 **Consider** |
| Parent-child (native) | ✅ | ⚠️ Manual | ⚠️ Manual | 🟡 **Consider** |
| Relationship queries | ✅ | ❌ | ❌ | 🟡 **Consider** |
| Inheritance (IsA) | ✅ | ❌ | ❌ | 🟢 **Nice to have** |
| **Performance** |
| Zero dependencies | ✅ C99 | ❌ | ⚠️ Few | 🟢 Good for mobile |
| Lockless scheduler | ✅ | ✅ | ✅ Planned | ✅ Already planned |
| Millions of entities | ✅ | ✅ | ✅ Target | ✅ Already planned |
| SIMD optimization | ⚠️ Some | ⚠️ Some | ✅ **Aggressive** | ✅ Our advantage |
| **Platform Support** |
| Mobile (Snapdragon) | ✅ | ✅ | ✅ Target | ✅ Already planned |
| Nintendo Switch | ✅ | ⚠️ Unofficial | ⚠️ Future | 🟢 Nice to have |
| WebAssembly | ✅ | ✅ | ✅ Target | ✅ Already planned |
| **Developer Tools** |
| Web-based UI | ✅ Explorer | ❌ | ⚠️ Planned | 🟡 **Consider** |
| Query language | ✅ **Powerful** | ✅ Good | ✅ Planned | ✅ Already planned |
| JSON serialization | ✅ Built-in | ⚠️ Manual | ✅ Planned | ✅ Already planned |
| Reflection | ✅ Built-in | ✅ | ✅ Planned | ✅ Already planned |
| Unit annotations | ✅ | ❌ | ❌ | 🟢 Nice to have |
| Statistics/Profiling | ✅ Addon | ⚠️ Manual | ✅ Planned | ✅ Already planned |
| **API Design** |
| C API | ✅ C99 | ❌ | ❌ | 🟢 For FFI |
| C++ API | ✅ C++17 | ❌ | ❌ | ❌ Rust only |
| Rust API | ⚠️ Bindings | ✅ Native | ✅ Native | ✅ Our focus |
| Free functions | ✅ | ✅ | ✅ Planned | ✅ Already planned |
| Automatic registration | ✅ | ✅ | ✅ Planned | ✅ Already planned |

---

## Key Flecs Features to Adopt

### 1. 🔴 **Entity Relationships (High Priority)**

**What Flecs Does:**
```c
// Parent-child relationship
ecs_entity_t parent = ecs_new(world);
ecs_entity_t child = ecs_new_w_pair(world, EcsChildOf, parent);

// Query children
ecs_query_t *q = ecs_query(world, {
    .terms = {{ .id = ecs_pair(EcsChildOf, parent) }}
});
```

**Why It's Better:**
- ✅ Native support (no manual HashMap)
- ✅ Efficient queries
- ✅ Automatic cleanup (despawn parent → despawn children)
- ✅ Relationship-based queries

**How to Implement in XS Engine:**
```rust
// Add to design.md
pub struct Relationship {
    kind: RelationshipKind,
    target: Entity,
}

pub enum RelationshipKind {
    ChildOf,
    IsA,      // Inheritance
    Custom(u32),
}

// Store in archetype
pub struct Archetype {
    // ... existing fields
    relationships: HashMap<Entity, Vec<Relationship>>,
}

// Query API
world.query::<(&Transform, ChildOf<Entity>)>()
    .iter()
    .for_each(|(transform, parent)| {
        // Process children
    });
```

**Benefits:**
- Better hierarchy performance
- Cleaner API
- Relationship-based queries
- Inheritance support

---

### 2. 🟡 **Query Language Enhancements (Medium Priority)**

**What Flecs Does:**
```c
// Complex queries with relationships
ecs_query(world, {
    .terms = {
        { .id = Position },
        { .id = ecs_pair(EcsChildOf, parent) },
        { .id = Velocity, .oper = EcsNot }  // Without Velocity
    }
});
```

**How to Implement:**
```rust
// Enhanced query filters
world.query::<(&Transform, &Sprite)>()
    .with_relationship(ChildOf, parent_entity)
    .without::<Velocity>()
    .iter();
```

---

### 3. 🟢 **Web-Based Debug UI (Nice to Have)**

**What Flecs Does:**
- Flecs Explorer (web-based UI)
- Real-time entity inspection
- Performance profiling
- Query visualization

**How to Implement:**
```rust
// Add to Phase 3 tasks
- [ ] 20.6 Implement web-based debug UI
  - Create WebSocket server for debug data
  - Export entity/component data as JSON
  - Create web UI (HTML/JS)
  - Real-time updates
  - _Requirements: 12.1_
```

---

### 4. 🟢 **Unit Annotations (Nice to Have)**

**What Flecs Does:**
```c
ECS_COMPONENT(world, Position);
ecs_unit(world, Position, {
    .quantity = EcsLength,
    .unit = EcsMeters
});
```

**How to Implement:**
```rust
#[derive(Component)]
#[unit(quantity = "Length", unit = "Meters")]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
```

**Benefits:**
- Better editor integration
- Unit conversion
- Validation

---

## Platform Support Comparison

### Mobile (Snapdragon)

| Feature | Flecs | XS Engine v2 |
|---------|-------|--------------|
| ARM NEON SIMD | ✅ | ✅ Planned |
| Memory efficiency | ✅ | ✅ Target: 30-50% reduction |
| Battery optimization | ✅ | ✅ Planned |
| Tile-based rendering | ⚠️ External | ✅ Built-in |

### Nintendo Switch

| Feature | Flecs | XS Engine v2 |
|---------|-------|--------------|
| Official support | ✅ | ⚠️ Future |
| ARM CPU | ✅ | ✅ Via NEON |
| Memory constraints | ✅ | ✅ Optimized |

**Recommendation:** Add Switch support in Phase 5 (post-release)

---

## Performance Comparison

### Benchmark Estimates

| Operation | Flecs | Bevy ECS | XS Engine v2 Target |
|-----------|-------|----------|---------------------|
| Spawn 10K entities | ~100 µs | ~150 µs | **<200 µs** |
| Query 10K (single) | ~3 µs | ~5 µs | **<5 µs** |
| Query 10K (multi) | ~15 µs | ~20 µs | **<20 µs** |
| Max entities @ 60 FPS | 1M+ | 100K+ | **100K+** |

**Note:** Flecs is C99 (lower overhead), but our SIMD optimization should compensate.

---

## Recommendations

### 🔴 High Priority (Add to Spec)

1. **Entity Relationships**
   - Add to Phase 2 (after core architecture)
   - Implement ChildOf, IsA relationships
   - Add relationship queries
   - Update requirements.md and design.md

2. **Enhanced Query Language**
   - Add relationship filters
   - Improve query ergonomics
   - Add to Phase 3

### 🟡 Medium Priority (Consider)

3. **Web-Based Debug UI**
   - Add to Phase 3 (debugging tools)
   - Optional feature flag
   - Great for development

4. **C FFI Layer**
   - Add to Phase 4 (for language bindings)
   - Enable C/C++ interop
   - Support other languages

### 🟢 Low Priority (Future)

5. **Unit Annotations**
   - Add to Phase 5 (post-release)
   - Editor enhancement
   - Nice to have

6. **Nintendo Switch Support**
   - Add to Phase 5
   - Requires SDK access
   - Market dependent

---

## Updated Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Public API Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  World API   │  │  Query API   │  │  System API  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Compatibility Layer                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  HashMap-based API Wrapper (for legacy code)         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      Core ECS Engine                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Archetype   │  │  Sparse Set  │  │   Change     │      │
│  │   Storage    │  │   Mapping    │  │  Detection   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Query     │  │   System     │  │   Resource   │      │
│  │   Engine     │  │  Scheduler   │  │   Manager    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ Relationship │  │  Web Debug   │  ← NEW from Flecs      │
│  │   System     │  │     UI       │                        │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Memory Management Layer                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  SIMD-Aligned│  │  Pooled      │  │  Cache-Line  │      │
│  │  Allocator   │  │  Allocator   │  │  Alignment   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## Conclusion

### What We Should Adopt from Flecs:

1. ✅ **Entity Relationships** - Major improvement over manual HashMap
2. ✅ **Enhanced Query Language** - Better developer experience
3. ✅ **Web-Based Debug UI** - Great for development
4. ⚠️ **C FFI Layer** - For language bindings (optional)

### What We Keep from Our Design:

1. ✅ **Aggressive SIMD Optimization** - Our advantage over Flecs
2. ✅ **Rust-First API** - Type safety and ergonomics
3. ✅ **Mobile-First Memory** - Better than Flecs for mobile
4. ✅ **Pixel Art Components** - Game-specific features

### Final Recommendation:

**Yes, we should adopt Flecs' relationship system!** It's a major feature that Bevy doesn't have and will make our ECS more powerful. I'll update the design document to include this.

---

## Next Steps

1. Update `requirements.md` with relationship requirements
2. Update `design.md` with relationship system design
3. Add relationship tasks to `tasks.md` (Phase 2)
4. Keep our SIMD optimization advantage
5. Consider web debug UI for Phase 3

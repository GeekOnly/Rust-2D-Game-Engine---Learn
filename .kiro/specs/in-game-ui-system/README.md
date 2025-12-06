# In-Game UI System Specification

## Overview

This specification defines a comprehensive, production-ready UI system for the XS 2D Game Engine, comparable to Unity's Canvas UI and Unreal Engine's UMG. The system replaces the legacy HUD system with a modern, ECS-based architecture.

## Documentation Structure

### Core Documents

1. **[requirements.md](requirements.md)** - Complete requirements specification
   - User stories and acceptance criteria
   - EARS-compliant requirements
   - Glossary of terms

2. **[design.md](design.md)** - Detailed system design
   - Architecture overview
   - Component specifications
   - Correctness properties
   - Testing strategy

3. **[tasks.md](tasks.md)** - Implementation task list
   - 26 task groups covering implementation and migration
   - Property-based testing tasks
   - Migration tasks for legacy system

4. **[MIGRATION_PLAN.md](MIGRATION_PLAN.md)** - Migration strategy
   - Legacy system analysis
   - Migration phases and timeline
   - Conversion tools and scripts
   - Rollback plan

## Quick Start

### For Developers

**Implementing the UI System:**
1. Read `requirements.md` to understand what needs to be built
2. Review `design.md` for architecture and component details
3. Follow `tasks.md` sequentially for implementation
4. Run property-based tests to verify correctness

**Current Status:**
- ✅ Task 1: UI crate structure and core types (COMPLETED)
- ⬜ Tasks 2-20: Core UI system implementation (IN PROGRESS)
- ⬜ Tasks 21-26: Migration from legacy HUD system (PENDING)

### For Migrators

**Migrating from Legacy HUD System:**
1. Read `MIGRATION_PLAN.md` for complete migration strategy
2. Wait for core UI system completion (tasks 1-20)
3. Use migration tools (tasks 21-22) to convert .hud files
4. Test converted prefabs thoroughly
5. Update Lua scripts to use new API

## System Comparison

### Legacy System (`engine/src/hud`)
- ❌ Simple HUD elements only
- ❌ No interaction/events
- ❌ No layout system
- ❌ No animations
- ❌ egui-based rendering
- ✅ Data binding support

### New System (`ui` crate)
- ✅ Complete UI component library (15+ types)
- ✅ Full event system (click, hover, drag, scroll)
- ✅ Automatic layout system (H/V/Grid)
- ✅ Animation system with easing
- ✅ WGPU-based rendering with batching
- ✅ ECS-integrated architecture
- ✅ Lua scripting support
- ✅ Prefab system
- ✅ Style and theme support

## Key Features

### Core Components
- Canvas with multiple render modes
- RectTransform with flexible anchoring
- UIImage with 9-slice support
- UIText with rich text and overflow handling
- UIButton with state transitions
- UIPanel for containers

### Advanced Components
- UISlider for value selection
- UIToggle for checkboxes
- UIDropdown for option selection
- UIInputField for text input
- UIScrollView with clipping and inertia

### Systems
- Layout System (Horizontal, Vertical, Grid)
- Event System (raycasting, event delivery)
- Animation System (tweens, easing functions)
- Rendering System (batching, culling, masking)
- Prefab System (reusable UI templates)
- Style System (themes, inheritance)

## Architecture

```
Game Application (Lua/Rust)
         ↓
    UI System API
         ↓
    ECS World (ui components)
         ↓
  Rendering Pipeline (WGPU)
```

## File Formats

### UI Prefab (.uiprefab)
```json
{
  "name": "MainMenu",
  "root": {
    "name": "Canvas",
    "rect_transform": { ... },
    "ui_element": { ... },
    "children": [ ... ]
  }
}
```

### Legacy HUD (.hud) - Deprecated
```json
{
  "name": "PlayerHUD",
  "elements": [
    {
      "id": "health_bar",
      "element_type": { "type": "HealthBar", ... },
      "anchor": "TopLeft",
      "offset": [20, 20],
      "size": [200, 30]
    }
  ]
}
```

## Migration Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Foundation | 2 weeks | 🟡 In Progress |
| Phase 2: Advanced Features | 2 weeks | ⬜ Pending |
| Phase 3: Migration Tools | 1 week | ⬜ Pending |
| Phase 4: UI Prefab Editor | 3 weeks | ⬜ Pending |
| Phase 5: Cleanup | 1 week | ⬜ Pending |
| **Total** | **9 weeks** | **10% Complete** |

## Testing

### Unit Tests
- Component creation and initialization
- Anchor calculations
- Layout algorithms
- Serialization/deserialization

### Property-Based Tests
- 64 correctness properties defined
- Minimum 100 iterations per property
- Using proptest library
- Tagged with property numbers

### Integration Tests
- Full UI workflows
- Multi-canvas scenarios
- Lua integration
- Performance benchmarks

## Contributing

### Implementation Guidelines
1. Follow the task list in `tasks.md` sequentially
2. Write property-based tests for universal properties
3. Write unit tests for specific examples
4. Update documentation as you implement
5. Mark tasks as complete using the task status tool

### Code Style
- Use Rust 2021 edition
- Follow existing code conventions
- Add doc comments to all public APIs
- Include usage examples in doc comments

### Testing Requirements
- All property tests must pass
- Unit test coverage > 80%
- No regressions in existing functionality
- Performance benchmarks must meet targets

## Resources

### External References
- [Unity Canvas UI Documentation](https://docs.unity3d.com/Manual/UICanvas.html)
- [Unreal UMG Documentation](https://docs.unrealengine.com/en-US/umg-ui-designer/)
- [egui Documentation](https://docs.rs/egui/)
- [WGPU Documentation](https://docs.rs/wgpu/)

### Internal References
- ECS crate: `ecs/`
- Render crate: `render/`
- Script crate: `script/`
- Legacy HUD: `engine/src/hud/`
- Widget Editor: `engine/src/editor/widget_editor/`

## FAQ

### Q: When will the migration be complete?
A: Estimated 9 weeks from start of implementation. Currently in Phase 1 (Foundation).

### Q: Can I use both systems during migration?
A: Yes, but not recommended. The plan is for a clean migration once the new system is ready.

### Q: Will my existing .hud files work?
A: Not directly. You'll need to convert them to .uiprefab format using the migration tools (tasks 21-22).

### Q: What about Lua scripts using the old HUD API?
A: They'll need to be updated to use the new UI API. Migration guide will provide API mapping.

### Q: Is the new system faster?
A: Yes, the new system uses WGPU with batching, which is significantly faster than egui for game UI.

### Q: Can I create UI from Lua?
A: Yes! The new system has full Lua bindings for creating and manipulating UI at runtime.

## Status

**Last Updated:** 2024-12-06

**Current Phase:** Phase 1 - Foundation (10% complete)

**Next Milestone:** Complete RectTransform system (Task 2)

**Blockers:** None

## Contact

For questions or issues:
- Create issue in project tracker
- Tag with `ui-system` label
- Reference this specification

---

**Note:** This is a living document. It will be updated as implementation progresses and requirements evolve.

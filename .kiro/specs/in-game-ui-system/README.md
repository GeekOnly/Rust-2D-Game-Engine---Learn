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

### Migration Documentation (in ui/ crate)

5. **[MIGRATION_GUIDE.md](../../ui/MIGRATION_GUIDE.md)** - Complete step-by-step migration guide
   - Before you begin checklist
   - Step-by-step migration process
   - Code examples (before and after)
   - Common issues and solutions
   - Testing procedures

6. **[API_CHANGES.md](../../ui/API_CHANGES.md)** - API changes reference
   - Breaking changes summary
   - Rust API changes
   - Lua API changes
   - File format changes
   - Component mapping tables
   - Deprecation notices

7. **[VIDEO_TUTORIAL_SCRIPTS.md](../../ui/VIDEO_TUTORIAL_SCRIPTS.md)** - Video tutorial scripts
   - Introduction to new UI system
   - Migration walkthrough
   - UI Prefab Editor tutorial
   - Creating interactive UIs with Lua

## Quick Start

### For Developers

**Implementing the UI System:**
1. Read `requirements.md` to understand what needs to be built
2. Review `design.md` for architecture and component details
3. Follow `tasks.md` sequentially for implementation
4. Run property-based tests to verify correctness

**Current Status:**
- ✅ Tasks 1-20: Core UI system implementation (COMPLETED)
- ✅ Tasks 21-24: Migration from legacy HUD system (COMPLETED)
- ✅ Task 25: Migration documentation (COMPLETED)
- ⬜ Task 26: Final migration verification (PENDING)

### For Migrators

**Migrating from Legacy HUD System:**

✅ **Migration is now complete!** The legacy HUD system has been removed and all example files have been converted.

**For new migrations:**
1. Read the [Migration Guide](../../ui/MIGRATION_GUIDE.md) for step-by-step instructions
2. Review [API Changes](../../ui/API_CHANGES.md) for breaking changes
3. Use the migration tool: `cargo run --package ui --bin hud_migrator -- --paths . --progress`
4. Test converted prefabs thoroughly
5. Update Lua scripts to use new API (see [Lua API Reference](../../ui/LUA_API.md))

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
| Phase 1: Foundation | 2 weeks | ✅ Complete |
| Phase 2: Advanced Features | 2 weeks | ✅ Complete |
| Phase 3: Migration Tools | 1 week | ✅ Complete |
| Phase 4: UI Prefab Editor | 3 weeks | ✅ Complete |
| Phase 5: Cleanup & Documentation | 1 week | ✅ Complete |
| **Total** | **9 weeks** | **✅ 100% Complete** |

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

### Q: Is the migration complete?
A: Yes! The legacy HUD system has been removed and all example files have been converted to the new UI system.

### Q: How do I migrate my .hud files?
A: Use the migration tool: `cargo run --package ui --bin hud_migrator -- --paths . --progress`. See the [Migration Guide](../../ui/MIGRATION_GUIDE.md) for detailed instructions.

### Q: What changed in the API?
A: The HUD Manager is gone, data binding is manual, and file format changed from .hud to .uiprefab. See [API Changes](../../ui/API_CHANGES.md) for complete mapping.

### Q: What about my Lua scripts?
A: Update them to use the new UI API. The [Migration Guide](../../ui/MIGRATION_GUIDE.md) includes before/after examples for common patterns.

### Q: Is the new system faster?
A: Yes! The new system uses WGPU with batching, which is significantly faster than the legacy egui-based rendering.

### Q: Can I create UI from Lua?
A: Yes! The new system has full Lua bindings for creating and manipulating UI at runtime. See [Lua API Reference](../../ui/LUA_API.md).

### Q: Where can I find examples?
A: Check the `ui/examples/` directory for comprehensive examples, and see [Examples Guide](../../ui/EXAMPLES_GUIDE.md).

## Status

**Last Updated:** December 2025

**Current Phase:** ✅ Complete - All phases finished

**Migration Status:** ✅ Legacy HUD system removed, all files converted

**Documentation:** ✅ Complete migration guide, API changes, and video tutorial scripts available

**Next Steps:** 
- Final verification testing (Task 26)
- Performance optimization
- Additional examples and tutorials

## Contact

For questions or issues:
- Create issue in project tracker
- Tag with `ui-system` label
- Reference this specification

---

**Note:** This is a living document. It will be updated as implementation progresses and requirements evolve.

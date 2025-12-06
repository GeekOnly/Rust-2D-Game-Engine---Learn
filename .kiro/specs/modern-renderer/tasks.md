# Modern AAA Mobile-First Renderer - Implementation Tasks

This document outlines the implementation tasks for building the modern rendering system. Tasks are organized into phases for incremental development, with each task building on previous work.

---

## Phase 1: Core Infrastructure

- [ ] 1. Set up render module foundation
  - Extend existing RenderModule with new subsystems
  - Create module structure for pipeline manager, material system, render graph
  - Set up wgpu device and queue management
  - _Requirements: 0, 21_

- [ ] 1.1 Implement pipeline manager
  - Create PipelineDescriptor struct with hashing
  - Implement pipeline cache (HashMap)
  - Add shader module cache
  - Add bind group layout cache
  - _Requirements: 21.1, 21.2, 21.3_

- [ ]* 1.2 Write property test for pipeline caching
  - **Property 9: Pipeline caching**
  - **Validates: Requirements 21.2**

- [ ] 1.3 Implement shader hot-reload system
  - Set up file watcher for shader directory
  - Implement shader recompilation on file change
  - Invalidate dependent pipelines
  - _Requirements: 21.4, 38.2_

- [ ]* 1.4 Write unit tests for pipeline manager
  - Test pipeline creation and caching
  - Test shader hot-reload
  - Test pipeline invalidation
  - _Requirements: 21_

---

## Phase 2: Material System

- [ ] 2. Implement core material system
  - Create Material struct with parameters and textures
  - Implement MaterialParameters with GPU uniform buffer
  - Add material registry (MaterialSystem)
  - Create default materials (unlit, error)
  - _Requirements: 1.1, 1.4_

- [ ] 2.1 Implement material parameter updates
  - Add set_scalar, set_vector, set_color methods
  - Implement GPU buffer synchronization
  - Add parameter change tracking
  - _Requirements: 1.2_

- [ ]* 2.2 Write property test for material parameter sync
  - **Property 2: Material parameter GPU synchronization**
  - **Validates: Requirements 1.2**

- [ ] 2.3 Implement material serialization
  - Add serde derives to MaterialAsset
  - Implement to_json and from_json methods
  - Handle texture path resolution
  - _Requirements: 44.1, 44.2_

- [ ]* 2.4 Write property test for material serialization round-trip
  - **Property 18: Material serialization round-trip**
  - **Validates: Requirements 44.3**

- [ ] 2.5 Implement material batching
  - Sort entities by material
  - Group consecutive entities with same material
  - Generate batched draw calls
  - _Requirements: 1.3_

- [ ]* 2.6 Write property test for draw call batching
  - **Property 3: Draw call batching**
  - **Validates: Requirements 1.3**

---

## Phase 3: Shader System

- [ ] 3. Implement shader compilation and caching
  - Create shader compiler wrapper
  - Implement shader source loading
  - Add compilation error handling with detailed messages
  - _Requirements: 3.1, 3.2_

- [ ] 3.1 Implement shader include system
  - Parse #include directives
  - Resolve include paths
  - Inline included code
  - Track dependencies
  - _Requirements: 4.1, 4.2_

- [ ]* 3.2 Write property test for shader include resolution
  - **Property 6: Shader include resolution**
  - **Validates: Requirements 4.1**

- [ ] 3.3 Implement circular include detection
  - Build dependency graph
  - Detect cycles using DFS
  - Report error with dependency chain
  - _Requirements: 4.3_

- [ ] 3.4 Implement shader compilation cache
  - Hash shader source code
  - Save compiled binaries to disk
  - Load and validate cached shaders
  - _Requirements: 45.1, 45.2, 45.3_

- [ ]* 3.5 Write property test for shader cache invalidation
  - **Property 21: Shader cache invalidation**
  - **Validates: Requirements 45.3**

---

## Phase 4: Render Graph

- [ ] 4. Implement render graph system
  - Create RenderGraph struct
  - Implement pass registration
  - Add resource management (textures, buffers)
  - _Requirements: 23.1_

- [ ] 4.1 Implement dependency tracking
  - Build dependency graph from pass inputs/outputs
  - Implement topological sort
  - Detect cyclic dependencies
  - _Requirements: 23.2_

- [ ]* 4.2 Write property test for render graph ordering
  - **Property 12: Render graph dependency ordering**
  - **Validates: Requirements 23.2**

- [ ] 4.3 Implement resource reuse optimization
  - Analyze resource lifetimes
  - Reuse textures between passes
  - Pool render targets
  - _Requirements: 23.3_

- [ ]* 4.4 Write property test for resource reuse
  - **Property 13: Render graph resource reuse**
  - **Validates: Requirements 23.3**

- [ ] 4.5 Implement render graph execution
  - Execute passes in topological order
  - Handle pass enable/disable
  - Synchronize GPU resources
  - _Requirements: 23.5_

---

## Phase 5: Forward Rendering Pipeline

- [ ] 5. Implement forward renderer
  - Create ForwardRenderer struct
  - Set up depth texture
  - Implement basic render pass
  - _Requirements: 0.1_

- [ ] 5.1 Implement entity sorting
  - Sort opaque front-to-back (depth)
  - Sort transparent back-to-front (depth)
  - Sort by material for batching
  - _Requirements: 0.1_

- [ ] 5.2 Implement light clustering (Forward+)
  - Create 3D cluster grid
  - Assign lights to clusters
  - Generate light index buffer
  - _Requirements: 0.4, 26.5_

- [ ] 5.3 Implement forward rendering shader
  - Write vertex shader (transform, lighting setup)
  - Write fragment shader (PBR lighting, clustered lights)
  - Add support for multiple lights
  - _Requirements: 0.1, 26_

- [ ]* 5.4 Write unit tests for forward renderer
  - Test entity sorting
  - Test light clustering
  - Test rendering output
  - _Requirements: 0_

---

## Phase 6: Deferred Rendering Pipeline

- [ ] 6. Implement G-buffer
  - Create G-buffer textures (albedo, normal, depth, etc.)
  - Set up G-buffer render targets
  - Implement G-buffer clear and resize
  - _Requirements: 0.2, 24.3_

- [ ] 6.1 Implement geometry pass
  - Create geometry pass pipeline
  - Write G-buffer output shader
  - Render all entities to G-buffer
  - _Requirements: 0.2, 24.3_

- [ ] 6.2 Implement lighting pass
  - Create fullscreen quad
  - Write lighting compute shader
  - Read from G-buffer, compute lighting
  - Output to final render target
  - _Requirements: 0.2_

- [ ] 6.3 Implement light volume rendering
  - Create sphere/cone meshes for lights
  - Render light volumes with stencil
  - Optimize for many lights
  - _Requirements: 0.2, 26_

- [ ]* 6.4 Write unit tests for deferred renderer
  - Test G-buffer creation
  - Test geometry pass
  - Test lighting pass
  - _Requirements: 0_

---

## Phase 7: Post-Processing Stack

- [ ] 7. Implement post-processing framework
  - Create PostProcessor struct
  - Set up ping-pong buffers
  - Implement effect chain execution
  - _Requirements: 5.1, 5.2_

- [ ]* 7.1 Write property test for effect ordering
  - **Property 8: Post-processing effect ordering**
  - **Validates: Requirements 5.2**

- [ ] 7.2 Implement bloom effect
  - Extract bright pixels (threshold)
  - Downsample and blur (multiple passes)
  - Upsample and combine
  - Composite with original
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 7.3 Implement depth of field
  - Calculate circle of confusion from depth
  - Blur based on CoC
  - Implement bokeh shape
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 7.4 Implement color grading and tonemapping
  - Create 3D LUT texture
  - Implement LUT sampling
  - Add exposure, contrast, saturation controls
  - Implement tonemapping (ACES, Reinhard, etc.)
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ]* 7.5 Write property test for post-processing performance
  - **Property 14: Post-processing performance budget**
  - **Validates: Requirements 43.1**

- [ ]* 7.6 Write unit tests for post-processing effects
  - Test bloom extraction and blur
  - Test DOF CoC calculation
  - Test color grading LUT
  - _Requirements: 6, 7, 8_

---

## Phase 8: Material Editor (Node-Based)

- [ ] 8. Implement material node graph
  - Create MaterialNode enum with all node types
  - Implement MaterialGraph with nodes and connections
  - Add node connection validation (type checking)
  - _Requirements: 2.1, 2.2_

- [ ]* 8.1 Write property test for node type validation
  - **Property 4: Node graph type validation**
  - **Validates: Requirements 2.2**

- [ ] 8.2 Implement shader code generator
  - Create ShaderCodegen struct
  - Generate WGSL vertex shader from graph
  - Generate WGSL fragment shader from graph
  - Handle all node types
  - _Requirements: 2.3_

- [ ]* 8.3 Write property test for shader code generation
  - **Property 5: Shader code generation validity**
  - **Validates: Requirements 2.3**

- [ ] 8.4 Implement material editor UI
  - Create node graph widget (using egui or custom)
  - Add node palette
  - Implement drag-and-drop connections
  - Add parameter editing
  - _Requirements: 2.1_

- [ ] 8.5 Implement material preview
  - Create preview renderer
  - Render sphere with material
  - Update preview on graph changes
  - _Requirements: 2.4_

- [ ]* 8.6 Write integration tests for material editor
  - Test node graph creation
  - Test shader generation
  - Test material compilation
  - _Requirements: 2_

---

## Phase 9: Particle System (GPU)

- [ ] 9. Implement GPU particle system
  - Create Particle struct (GPU layout)
  - Allocate particle buffers (double-buffered)
  - Set up particle emitters
  - _Requirements: 11.1_

- [ ] 9.1 Implement particle update compute shader
  - Write WGSL compute shader for particle update
  - Apply forces (gravity, wind, etc.)
  - Update positions and velocities
  - Handle particle lifecycle
  - _Requirements: 11.1, 11.3_

- [ ] 9.2 Implement particle rendering
  - Create instanced rendering pipeline
  - Generate billboarded quads
  - Apply texture atlas
  - Sort particles for transparency
  - _Requirements: 11.4, 11.5_

- [ ] 9.3 Implement particle emitters
  - Create ParticleEmitter struct
  - Implement emission logic
  - Add emission shapes (point, sphere, cone, etc.)
  - _Requirements: 11.1_

- [ ]* 9.4 Write property test for particle performance
  - **Property 15: Particle system performance budget**
  - **Validates: Requirements 43.2**

- [ ]* 9.5 Write unit tests for particle system
  - Test particle spawning
  - Test particle update
  - Test particle rendering
  - _Requirements: 11_

---

## Phase 10: VFX (Trails, Distortion)

- [ ] 10. Implement trail renderer
  - Create trail position history buffer
  - Generate trail mesh from positions
  - Implement trail fading
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_

- [ ] 10.1 Implement screen-space distortion
  - Create distortion render pass
  - Render distortion objects to distortion buffer
  - Apply distortion in post-processing
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

- [ ]* 10.2 Write unit tests for VFX
  - Test trail mesh generation
  - Test distortion rendering
  - _Requirements: 12, 13_

---

## Phase 11: SPH Fluid Simulation

- [ ] 11. Implement SPH fluid simulator
  - Create FluidParticle struct (GPU layout)
  - Allocate particle buffer
  - Set up fluid parameters
  - _Requirements: 14.1_

- [ ] 11.1 Implement spatial hash grid
  - Create grid structure
  - Implement hash function
  - Build grid on GPU (compute shader)
  - _Requirements: 14.2_

- [ ] 11.2 Implement SPH density computation
  - Write compute shader for density calculation
  - Use spatial grid for neighbor search
  - Calculate pressure from density
  - _Requirements: 14.2, 14.3_

- [ ] 11.3 Implement SPH force computation
  - Write compute shader for force calculation
  - Compute pressure forces
  - Compute viscosity forces
  - Compute surface tension
  - _Requirements: 14.3, 14.4_

- [ ] 11.4 Implement SPH integration
  - Write compute shader for position/velocity update
  - Apply forces
  - Handle collisions
  - _Requirements: 14.1_

- [ ] 11.5 Implement fluid rendering
  - Render particles as spheres or metaballs
  - Add surface reconstruction (marching cubes)
  - Apply water material
  - _Requirements: 14.1_

- [ ]* 11.6 Write unit tests for fluid simulation
  - Test spatial hashing
  - Test density computation
  - Test force computation
  - _Requirements: 14_

---

## Phase 12: Water, Fog, Smoke

- [ ] 12. Implement water rendering
  - Create water plane mesh
  - Set up reflection/refraction textures
  - Implement water material shader
  - Add wave animation with normal maps
  - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_

- [ ] 12.1 Implement volumetric fog
  - Create 3D fog density texture
  - Write raymarch shader
  - Implement light scattering
  - Add fog parameters (density, height falloff)
  - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_

- [ ] 12.2 Implement smoke simulation
  - Create 3D grid for velocity and density
  - Implement advection (semi-Lagrangian)
  - Implement diffusion
  - Implement dissipation
  - Render with raymarching
  - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5_

- [ ]* 12.3 Write unit tests for water, fog, smoke
  - Test water reflection/refraction
  - Test fog raymarching
  - Test smoke simulation
  - _Requirements: 15, 16, 17_

---

## Phase 13: Grid-Based Physics

- [ ] 13. Implement 2D cellular automata
  - Create grid buffer (GPU)
  - Implement material types (sand, water, stone, etc.)
  - Write compute shader for CA rules
  - Implement falling, flowing, reactions
  - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_

- [ ] 13.1 Implement 3D voxel physics
  - Create 3D voxel grid
  - Implement voxel update compute shader
  - Add gravity and support checking
  - Implement chunk-based updates
  - _Requirements: 19.1, 19.2, 19.3, 19.4_

- [ ] 13.2 Implement voxel rendering (greedy meshing)
  - Write compute shader for greedy meshing
  - Generate mesh from voxel grid
  - Render voxel mesh
  - _Requirements: 19.5_

- [ ]* 13.3 Write unit tests for grid physics
  - Test 2D CA rules
  - Test 3D voxel physics
  - Test greedy meshing
  - _Requirements: 18, 19_

---

## Phase 14: Destruction System

- [ ] 14. Implement destruction system
  - Create FracturedMesh struct
  - Implement mesh fracturing algorithm (Voronoi)
  - Cache fractured meshes
  - _Requirements: 20.1_

- [ ] 14.1 Implement debris physics
  - Create Debris struct
  - Apply physics forces
  - Update debris transforms
  - Handle debris lifecycle
  - _Requirements: 20.2_

- [ ] 14.2 Implement debris optimization
  - Merge distant debris
  - Fade and remove old debris
  - Limit active debris count
  - _Requirements: 20.3, 20.4, 20.5_

- [ ]* 14.3 Write unit tests for destruction
  - Test mesh fracturing
  - Test debris physics
  - Test debris optimization
  - _Requirements: 20_

---

## Phase 15: Shadow System

- [ ] 15. Implement shadow mapping
  - Create ShadowMap struct
  - Implement shadow render pass
  - Render depth from light perspective
  - _Requirements: 25.1_

- [ ] 15.1 Implement cascaded shadow maps
  - Create multiple shadow cascades
  - Calculate split distances
  - Render each cascade
  - Sample correct cascade in shader
  - _Requirements: 25.1, 25.2_

- [ ] 15.2 Implement PCF filtering
  - Sample multiple shadow map texels
  - Average results for soft shadows
  - Add shadow bias to prevent acne
  - _Requirements: 25.4, 25.3_

- [ ]* 15.3 Write unit tests for shadows
  - Test shadow map creation
  - Test cascade calculation
  - Test PCF filtering
  - _Requirements: 25_

---

## Phase 16: Light System

- [ ] 16. Implement light system
  - Create Light enum (Directional, Point, Spot)
  - Implement LightSystem for management
  - Create light buffer (GPU)
  - _Requirements: 26.1, 26.2_

- [ ] 16.1 Implement light culling
  - Create cluster grid
  - Assign lights to clusters (compute shader)
  - Generate light index buffer
  - _Requirements: 26.4, 26.5_

- [ ]* 16.2 Write unit tests for lighting
  - Test light registration
  - Test light culling
  - Test light buffer updates
  - _Requirements: 26_

---

## Phase 17: PBR Materials

- [ ] 17. Implement PBR material support
  - Add PBR texture slots (albedo, metallic, roughness, normal, AO)
  - Write PBR shader (Cook-Torrance BRDF)
  - Implement image-based lighting (IBL)
  - Add environment map support
  - _Requirements: 27.1, 27.2, 27.3, 27.4, 27.5_

- [ ]* 17.1 Write unit tests for PBR
  - Test BRDF calculation
  - Test IBL sampling
  - Test material properties
  - _Requirements: 27_

---

## Phase 18: Stylized Rendering

- [ ] 18. Implement toon shading
  - Quantize lighting into bands
  - Add cel shading shader
  - Implement outline rendering (inverted hull or edge detection)
  - Add rim lighting
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [ ] 18.1 Implement pixel-perfect rendering
  - Add nearest-neighbor filtering mode
  - Implement pixel snapping for camera
  - Add integer scaling
  - Maintain pixel aspect ratio
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ]* 18.2 Write unit tests for stylized rendering
  - Test toon shading quantization
  - Test outline rendering
  - Test pixel-perfect snapping
  - _Requirements: 9, 10_

---

## Phase 19: Optimization Systems

- [ ] 19. Implement culling systems
  - Implement frustum culling
  - Implement distance culling
  - Implement layer culling
  - _Requirements: 30.1, 30.2, 30.3, 30.4, 30.5_

- [ ] 19.1 Implement occlusion culling
  - Render occluders to depth buffer
  - Test object visibility with GPU queries
  - Skip occluded objects
  - _Requirements: 31.1, 31.2, 31.3, 31.4, 31.5_

- [ ] 19.2 Implement LOD system
  - Define LOD levels per mesh
  - Calculate distance to camera
  - Switch LOD based on distance
  - Blend between LOD levels
  - _Requirements: 29.1, 29.2, 29.3, 29.4, 29.5_

- [ ] 19.3 Implement instanced rendering
  - Create instance buffer
  - Pass per-instance data (transforms)
  - Use GPU instancing for draw calls
  - Implement GPU culling for instances
  - _Requirements: 32.1, 32.2, 32.3, 32.4, 32.5_

- [ ] 19.4 Implement batching system
  - Sort entities by material
  - Combine meshes dynamically
  - Generate batched draw calls
  - _Requirements: 33.1, 33.2, 33.3, 33.4, 33.5_

- [ ]* 19.5 Write unit tests for optimization
  - Test frustum culling
  - Test LOD switching
  - Test instancing
  - Test batching
  - _Requirements: 29, 30, 31, 32, 33_

---

## Phase 20: Mobile Optimization

- [ ] 20. Implement thermal management
  - Query platform thermal API
  - Track temperature history
  - Define quality profiles per thermal state
  - Apply quality reduction when hot
  - _Requirements: 34.1, 34.2, 34.3, 34.4, 34.5_

- [ ] 20.1 Implement adaptive quality
  - Track frame time history
  - Adjust quality based on performance
  - Scale resolution dynamically
  - Adjust shadow quality, particle count, etc.
  - _Requirements: 35.1, 35.2, 35.3, 35.4, 35.5_

- [ ] 20.2 Implement GPU memory management
  - Track GPU memory usage
  - Handle allocation failures gracefully
  - Reduce quality under memory pressure
  - Free unused resources
  - _Requirements: 46.1, 46.2, 46.3, 46.4, 46.5_

- [ ]* 20.3 Write property test for memory tracking
  - **Property 22: GPU memory tracking**
  - **Validates: Requirements 46.1**

- [ ]* 20.4 Write property test for quality reduction
  - **Property 23: Memory pressure quality reduction**
  - **Validates: Requirements 46.3**

- [ ]* 20.5 Write integration tests for mobile optimization
  - Test thermal management
  - Test adaptive quality
  - Test memory management
  - _Requirements: 34, 35, 46_

---

## Phase 21: Developer Tools

- [ ] 21. Implement GPU profiler
  - Create timestamp query set
  - Implement profiling scopes
  - Measure GPU time per pass
  - Display profiling results
  - _Requirements: 36.1, 36.2, 36.3, 36.4, 36.5_

- [ ] 21.1 Implement shader debugger
  - Add debug visualization modes
  - Render normals, UVs, depth, etc.
  - Add wireframe mode
  - Add overdraw visualization
  - _Requirements: 37.1, 37.2, 37.3, 37.4, 37.5_

- [ ]* 21.2 Write unit tests for profiler
  - Test scope timing
  - Test result collection
  - _Requirements: 36_

---

## Phase 22: Texture and Resource Management

- [ ] 22. Implement texture streaming
  - Load low-res mips first
  - Stream high-res mips on demand
  - Unload distant textures
  - Prioritize visible textures
  - _Requirements: 28.1, 28.2, 28.3, 28.4, 28.5_

- [ ] 22.1 Implement render target management
  - Create render target pool
  - Allocate render targets on demand
  - Resize render targets
  - Free unused render targets
  - _Requirements: 39.1, 39.2, 39.3, 39.4, 39.5_

- [ ] 22.2 Implement HDR rendering
  - Use floating-point render targets
  - Preserve HDR values in pipeline
  - Apply tonemapping at end
  - _Requirements: 40.1, 40.2, 40.3, 40.4, 40.5_

- [ ] 22.3 Implement MSAA support
  - Create multisampled render targets
  - Configure sample count (2x, 4x, 8x)
  - Resolve to final target
  - Fall back to FXAA on mobile
  - _Requirements: 41.1, 41.2, 41.3, 41.4, 41.5_

- [ ]* 22.4 Write unit tests for resource management
  - Test texture streaming
  - Test render target pooling
  - Test HDR pipeline
  - Test MSAA
  - _Requirements: 28, 39, 40, 41_

---

## Phase 23: Platform-Specific Optimizations

- [ ] 23. Implement platform detection
  - Detect GPU capabilities
  - Detect platform (desktop, mobile, web)
  - Select appropriate features
  - _Requirements: 42.4_

- [ ] 23.1 Implement platform-specific shaders
  - Create shader variants for platforms
  - Use mobile-optimized shaders on mobile
  - Use high-quality shaders on desktop
  - _Requirements: 42.1, 42.2_

- [ ] 23.2 Implement WebGPU compatibility
  - Ensure all features work with WebGPU
  - Use WebGPU-compatible formats
  - Test on web platform
  - _Requirements: 42.3_

- [ ]* 23.3 Write integration tests for platforms
  - Test desktop features
  - Test mobile features
  - Test web features
  - _Requirements: 42_

---

## Phase 24: Integration and Polish

- [ ] 24. Integrate with ECS
  - Create rendering components (MeshRenderer, Camera, Light)
  - Implement render_system
  - Query entities from ECS
  - Render scene
  - _Requirements: All_

- [ ] 24.1 Implement backward compatibility
  - Support legacy sprite renderer
  - Support legacy tilemap renderer
  - Convert to new material system
  - _Requirements: All_

- [ ] 24.2 Create example materials
  - Unlit material
  - PBR material
  - Toon material
  - Water material
  - Particle material
  - _Requirements: 1, 27, 9_

- [ ] 24.3 Create example scenes
  - PBR showcase scene
  - Particle effects scene
  - Fluid simulation scene
  - Destruction demo scene
  - _Requirements: All_

- [ ]* 24.4 Write integration tests
  - Test full rendering pipeline
  - Test material editor workflow
  - Test hot reload
  - Test mobile optimization
  - _Requirements: All_

---

## Phase 25: Final Checkpoint

- [ ] 25. Final testing and optimization
  - Run all unit tests
  - Run all property tests
  - Run all integration tests
  - Profile performance
  - Optimize bottlenecks
  - _Requirements: All_

- [ ] 25.1 Documentation
  - Write API documentation
  - Write material editor guide
  - Write shader authoring guide
  - Write performance optimization guide
  - Create tutorial videos
  - _Requirements: All_

- [ ] 25.2 Ensure all tests pass, ask the user if questions arise
  - Verify all requirements are met
  - Verify all performance targets are achieved
  - Verify all platforms work correctly
  - _Requirements: All_

---

## Summary

**Total Tasks:** 25 phases, ~120 tasks
**Estimated Timeline:** 4-6 months for full implementation
**Priority Order:** Core → Materials → Shaders → Rendering → VFX → Optimization → Polish

**Key Milestones:**
- Phase 5: Basic forward rendering working
- Phase 10: VFX system functional
- Phase 15: Shadows and lighting complete
- Phase 20: Mobile optimization complete
- Phase 25: Production-ready

**Testing Coverage:**
- 23 property-based tests for correctness
- ~30 unit test suites
- ~10 integration test suites
- Performance benchmarks for all subsystems

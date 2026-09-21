# ADR 0001: Transition from MVP Prototype to Production-Grade Architecture

- **Status**: Accepted
- **Date**: 2026-09-21
- **Author**: MineRust Core Engineering Team

---

## Context
MineRust began as a prototype MVP demonstrating core voxel mechanics: procedurally generated terrain, greedy meshing, LOD scaling, dynamic cellular automata fluid propagation, and basic physics in Bevy.

While functionally rich, the codebase currently exhibits typical rapid-prototyping debt:
1. **Single Monolithic Binary Crate**: All code resides under `src/`, coupling domain math (noise, chunk indexing, fluid rules) directly to Bevy ECS engine structures.
2. **Untyped Coordinate Spaces**: Coordinates across chunks, local voxel offsets, and world space frequently use raw `IVec3`/`IVec2` vectors, inviting silent transposition errors.
3. **Implicit Panic Surfaces**: Widespread reliance on `.unwrap()` and `.expect()` across tick loops and streaming pipelines.
4. **Mega-System ECS Signatures**: Crucial systems (e.g. `world_streaming_system`) declare 9+ arguments, hindering readability and violating ECS modularity best practices.
5. **Absence of Strict Linting & Benchmarks**: 80+ clippy warnings/errors under `-D warnings` and lack of formal regression benchmarks.

## Decision
We formally transition MineRust into a **production-grade software project**, governed by the following decisions:

1. **Multi-Crate Workspace**: We will partition the codebase into decoupled crates:
   - `minerust-core`: Pure Rust domain math, coordinate newtypes, block definitions, chunk memory representation, and fluid CA logic (zero Bevy dependency).
   - `minerust-mesher`: High-performance greedy meshing, vertex generation, LOD masks, and face culling.
   - `minerust-world`: Chunk streaming, persistence, and world generator orchestration.
   - `minerust-client`: Bevy ECS plugins, rendering pipeline, inputs, HUD/UI, and audio.
2. **Semantic Newtypes**: Enforce strong types (`BlockPos`, `ChunkPos`, `LocalBlockPos`, `WorldPos`) across all APIs.
3. **Zero-Panic Policy**: Replace all runtime unwrap calls with typed `Result` errors using `thiserror`.
4. **Centralized Linting**: Enforce `[lints.rust]` and `[lints.clippy]` directly in `Cargo.toml`.
5. **Testing Pyramid**: Complement existing unit tests with property-based testing (`proptest`) and automated performance benchmarks (`criterion`).

## Consequences

### Positive
- **Drastically Reduced Compilation Times**: Modifying game logic or mesher code will no longer trigger re-evaluation of the entire Bevy dependency graph.
- **100% Testability**: Pure domain logic can be tested in milliseconds without requiring graphics initialization or windowing contexts.
- **Architectural Clarity**: Clear boundaries eliminate hidden couplings between game rules and engine presentation.
- **Predictable Performance**: Zero-allocation hot paths and automated benchmarks prevent regressions.

### Negative / Trade-offs
- Requires an upfront phased refactoring effort.
- Developers must maintain strict lint and formatting discipline on every commit.

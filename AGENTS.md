# MineRust — Instructions for Coding Agents

MineRust is a high-performance, multi-threaded Minecraft-like voxel sandbox engine built in 100% Safe Rust powered by Bevy.
This document outlines strict coding guidelines, quality gates, and architectural conventions for autonomous AI coding agents working on this codebase.

---

## 1. Operating Protocol

1. **Autonomous Execution**: Solve technical problems, refactor code, fix linter warnings, and keep tests passing autonomously. Only stop to request input for major product design pivots, external release credentials, or breaking changes to user requirements.
2. **Pre-flight Check**: Before touching code, check git status, the active branch, and verify existing tests pass via `cargo test`.
3. **Preserve Documentation Integrity**: Preserve all existing architectural comments, module docstrings, and design rationale unless specifically instructed to rewrite them.

---

## 2. Never Crash (Zero-Panic Policy)

Stability outranks feature additions. A panic or runtime crash in a game engine destroys the player's world state or disrupts high-speed benchmarks.

- **Forbid Unsafe Code**: `#![forbid(unsafe_code)]` is strictly enabled at crate root and must never be relaxed or circumvented.
- **No Panics in Non-Test Code**:
  - Never use `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, or `unreachable!()` in production paths.
  - Handled errors must return `Result<T, E>` using idiomatic `thiserror` types (`WorldError`, etc.) and the `?` operator.
  - An incomplete or unsupported feature returns an error or safely disables itself; it never panics.
- **External and Hostile Input Handling**:
  - All external data (world seeds, save files, user coordinates, slider percentages, CLI flags) is potentially hostile.
  - Prefer safe accessors like `.get()` instead of direct slice indexing `[i]`.
  - Use checked or saturating arithmetic for coordinate conversions and buffer offsets.
  - Clamp bounding boxes, allocate with size caps, and guard against unbounded recursion.
- **Prove Every Fix**:
  - Every crash or edge-case bugfix must include an automated regression test in the appropriate test module.

---

## 3. Architecture & File Size Limits

- **File Length Limit**: Strictly **under 1,000 Lines of Code (LOC)** per file.
  - If a file approaches ~900 LOC, proactively split and extract cohesive submodules or helper structs into child files (e.g. `streaming.rs` -> helper modules).
- **ECS Separation of Concerns**:
  - Keep domain logic separate: World Grid (`src/world/grid.rs`), Meshing (`src/mesher/`), Fluid Simulation (`src/fluid.rs`), Items & Tools (`src/item.rs`, `src/item_icon.rs`), Inventory & Crafting (`src/inventory/`, `src/crafting/`), World Containers (`src/chest.rs`, `src/furnace/`), Voxel Interaction (`src/interaction/`), Universal Audio (`src/audio.rs`), UI/Menu (`src/menu/`), Profiling (`src/profile.rs`), Benchmarking (`src/benchmark.rs`).
  - Do not introduce circular dependencies between Bevy systems.
- **Open-Source Asset & Fallback Policy**:
  - Never commit copyrighted third-party game audio or textures to the repository (`assets/audio/*.wav` is git-ignored).
  - Every external asset loader (`src/texture.rs`, `src/item_icon.rs`, `src/audio.rs`) must provide a deterministic, zero-panic procedural fallback in memory so fresh clones and CI pipelines pass 100% without external binary files.
- **Resource & Memory Management**:
  - Maintain debouncing and frame pacing on GPU buffer uploads (`MAX_CHUNK_DISPATCH_PER_FRAME`, `MAX_MESHES_PER_FRAME`).
  - Minimize allocations inside hot frame loops; reuse scratch buffers where applicable.

---

## 4. Quality Gates (Mandatory Before Every Commit)

Before committing changes or declaring a task complete, you MUST execute and pass all of the following:

```bash
# 1. Format verification
cargo fmt --all -- --check

# 2. Strict linter verification across all targets and features
cargo clippy --all-targets --all-features -- -D warnings

# 3. Full test suite verification
cargo test

# 4. File size verification (< 1000 LOC)
wc -l src/**/*.rs
```

If any check fails, resolve it immediately before proceeding.

---

## 5. Shell & Tooling Conventions

- **Shell Gotchas**:
  - Tools like `cp` or `mv` might be aliased to interactive mode in user environments; always use `/bin/cp -f` or `/bin/mv -f` when overwriting in automated scripts.
- **Git Commit Standards**:
  - Use Conventional Commits: `feat:`, `fix:`, `perf:`, `refactor:`, `style:`, `test:`, `docs:`, `chore:`.
  - Never push broken builds or unformatted code.

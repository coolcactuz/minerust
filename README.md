# MineRust ⛏️🦀

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Engine](https://img.shields.io/badge/Engine-Bevy_0.15%2F0.19-blue?logo=bevy&logoColor=white)](https://bevyengine.org/)
[![Safety](https://img.shields.io/badge/Safety-%23!%5Bforbid(unsafe__code)%5D-brightgreen)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![Tests](https://img.shields.io/badge/Tests-36%20Passing-success?logo=github-actions&logoColor=white)](https://github.com/)
[![Performance](https://img.shields.io/badge/Framerate-60%2B%20FPS-purple)](https://github.com/)
[![License](https://img.shields.io/badge/License-MIT%2FApache-blue)](LICENSE)

**An ultra-fast, multi-threaded voxel sandbox engine built from scratch in pure Rust using Bevy ECS.**

[Quick Start](#-quick-start) • [Features](#-features-at-a-glance) • [Controls](#-controls) • [Architecture Deep Dive](ARCHITECTURE.md) • [Benchmarks](#-performance--benchmarks)

</div>

---

## 🌟 Overview

**MineRust** is a modern, high-performance voxel engine and sandbox game created to explore the frontiers of data-oriented systems programming, procedural generation, and real-time computer graphics. 

Engineered with an uncompromising commitment to **zero unsafe code** (`#![forbid(unsafe_code)]`), MineRust achieves smooth, uncompromised 60+ FPS gameplay with multi-threaded terrain generation, asymptotic greedy meshing quad reduction, GPU-direct memory streaming, and real-time cellular automaton fluid dynamics.

Whether you are exploring mountainous biomes, digging into caverns, swimming up waterfalls, or benchmarking rendering performance with the in-engine telemetry HUD, MineRust showcases the raw power of modern Rust in game systems engineering.

---

## 🚀 Features at a Glance

### 🌍 Procedural Infinite World
- **Deterministic 64-bit Seeds**: Driven by `SplitMix64` and a 512-permutation Fisher-Yates shuffle for reproducible world generation.
- **Dynamic Biomes**: Distinct ecosystems including Plains, dense Forests, Deserts with cacti, Snowy Tundras with pine trees, steep Mountains with snow caps, sandy Beaches, and Oceans.
- **Subterranean 3D Caves**: Volumetric 3D noise networks carving out winding tunnels and cavernous underground halls.
- **Geological Ore Strata**: Realistic vertical distributions for Coal, Iron, Gold, and Diamond veins down to indestructible Bedrock.
- **Interactive Seed Picker**: New games roll a fresh random seed automatically, with an in-menu alphanumeric input box and seed randomizer.

### ⚡ Cutting-Edge Voxel Performance
- **Multi-Threaded Greedy Meshing**: Merges coplanar adjacent voxel faces into unified rectangular quads, slashing vertex counts by **~75%** and speeding up meshing by **2.5x**.
- **0ms Main-Thread Meshing**: Terrain generation and mesh synthesis execute completely in the background via Bevy's `AsyncComputeTaskPool`.
- **GPU-Direct Memory Streaming**: Mesh buffers utilize `RenderAssetUsages::RENDER_WORLD` to deallocate CPU vertex copies upon GPU upload, eliminating RAM bloat even at expansive 64-chunk render distances.
- **Two-Tier Lookahead Streaming**: Pre-generates voxel data in RAM ahead of the camera's visual view distance to completely eliminate traversal stutters.
- **Dynamic 3D Distance LOD**: Adapts geometry density using real-time 3D Euclidean distance calculations, ensuring fluid performance during vertical creative flight.
- **Max-Y Atmosphere Skip**: Skips empty airspace scanning during meshing for a 2x throughput boost.

### 🌊 Cellular Automata Fluid Dynamics
- **Real-Time Water Physics**: Non-blocking cellular automaton simulation managing downward cascading waterfalls, lateral canal expansion, and seabed void filling.
- **Buoyancy Mechanics**: Realistic drag, water resistance, vertical swimming thrust (`Space`), and diving controls (`Shift`).

### ⛏️ Survival Gameplay Loop & Inventory
- **Authentic Progression**: Players spawn with empty slots, gathering raw materials directly from the environment.
- **Block Mining & Item Drops**: Mining targeted blocks routes items directly into the player's 9-slot Hotbar, then overflows into the 27-slot Main Storage.
- **Full Inventory Modal (`E`)**: Interactive UI supporting slot swapping, item transfer, and active hotbar management.
- **Anti-Self-Trapping Placement**: Raymarching verification prevents accidental player suffocation when placing solid voxels.

### 📊 Developer & Benchmark Telemetry HUD
- **Production Mode by Default**: Release builds lock all optimizations ON, offering clean and streamlined gameplay for players.
- **Diagnostic Mode (`--dev`)**: Unlocks the in-game **F3 HUD** displaying real-time FPS, frame times, vertex tallies, active chunks, and live toggles for every optimization sub-pipeline.
- **Setting Info Cards**: Interactive hover cards in the settings menu explain what each rendering and optimization toggle does under the hood.

---

## 🎮 Controls

| Key / Input | Action |
| :--- | :--- |
| **`W` `A` `S` `D`** | First-person movement with ground friction & inertia |
| **Mouse** | Camera look (Left Click in window to capture mouse / `ESC` to release) |
| **`Space`** | Jump / Swim upward in water / Ascend in Flight Mode |
| **`Shift`** | Sneak (crouch with cliff-edge protection) / Dive / Descend in Flight |
| **`Ctrl`** | Sprint |
| **`F`** | Toggle **Flight Mode** (Creative No-Clip) |
| **Left Click** | Mine targeted voxel block |
| **Right Click** | Place active block |
| **`1` – `9` / Scroll** | Select active Hotbar slot |
| **`E`** | Open / Close Inventory Screen |
| **`F3`** | Toggle Real-time Diagnostic HUD (when launched with `--dev`) |
| **`ESC`** | Pause Game / Return to Menu |

---

## 🏁 Quick Start

### Prerequisites
- [Rust 2024 Edition or latest stable toolchain](https://www.rust-lang.org/)
- Modern graphics drivers supporting Vulkan, DirectX 12, or Metal (via WGPU)

### Running the Game

```bash
# 1. Clone the repository
git clone https://github.com/cactuz/minerust.git
cd minerust

# 2. Run in Production Mode (All optimizations enabled by default)
cargo run --release

# 3. Run in Developer & Benchmark Mode (Enables F3 HUD & Dev Settings)
cargo run --release -- --dev

# 4. Launch with a custom world seed (string or integer)
cargo run --release -- --seed "linkedin_showcase"
cargo run --release -- -s 133742
```

---

## 📈 Performance & Benchmarks

Benchmarked using [`criterion`](https://github.com/bheisler/criterion.rs) on Linux 6.x / AMD Ryzen:

```
chunk_mesher/greedy_meshing_active
                        time:   [112.18 µs 112.51 µs 112.87 µs]
chunk_mesher/naive_meshing_fallback
                        time:   [284.92 µs 285.73 µs 286.60 µs]
--> Performance Gain: 2.54x faster meshing, ~75% fewer vertices sent to GPU

generate_chunk_procedural
                        time:   [183.91 µs 184.22 µs 184.60 µs]
--> Generation Throughput: >5,400 chunks/second per CPU core
```

### Visualizing Greedy Meshing Quad Reduction

```
Naive Face Meshing (16 Quads / 32 Triangles):
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+

Greedy Merged Quad (1 Quad / 2 Triangles):
+---------------+
|               |
|    ~75%       |
|  Vertex Drop  |
|               |
+---------------+
```

---

## 🏛️ Architecture & Technical Deep Dive

Curious about how the engine works under the hood?

Read our comprehensive [**ARCHITECTURE.md**](ARCHITECTURE.md) for in-depth engineering documentation, including:
- **Semantic Coordinate Newtypes** (`BlockPos`, `ChunkPos`, `LocalBlockPos`)
- **GPU-Direct Memory Purge** (`RenderAssetUsages::RENDER_WORLD`)
- **Two-Tier Lookahead Streaming & LRU Caching**
- **Cellular Automaton Fluid Mechanics**
- **Discrete AABB Physics & Edge Raymarching**
- **Binary Delta LZ4 Chunk Serialization**

---

## 🛡️ Code Quality & Verification

Every commit is verified against rigorous production standards:

```bash
# Run complete test suite (36 unit, integration, and property tests)
cargo test

# Enforce strict zero-warning pedantic clippy compliance
cargo clippy --all-targets -- -D warnings

# Check code formatting
cargo fmt --check

# Execute Criterion microbenchmarks
cargo bench
```

---

## 📄 License

This project is licensed under either of:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

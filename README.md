# MineRust ⛏️🦀

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Engine](https://img.shields.io/badge/Engine-Bevy-blue?logo=bevy&logoColor=white)](https://bevyengine.org/)
[![Safety](https://img.shields.io/badge/Safety-%23!%5Bforbid(unsafe__code)%5D-brightgreen)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![License](https://img.shields.io/badge/License-MIT%2FApache_2.0-blue)](#-license)

**A 3D voxel sandbox engine built in pure Rust with Bevy, exploring game development fundamentals and real-time graphics optimization.**

</div>

---

## 💡 About the Project

**MineRust** is a personal learning project created to explore and understand the practical challenges of **game development** and **real-time computer graphics optimization**.

Building a voxel game from scratch offers a hands-on playground to encounter and solve the classic bottlenecks of 3D engines: handling massive amounts of dynamic geometry, streaming continuous open-world terrain, managing GPU memory, and keeping frame times consistent and smooth.

Rather than relying on ready-made high-level game features, the goal of this project is to dive under the hood and learn how game engines tackle:
- **Geometry scalability**: How to render huge procedural landscapes without choking the GPU.
- **Multithreading & streaming**: Keeping the main loop stutter-free while generating and meshing worlds in the background.
- **Rendering pipelines**: Balancing opaque passes, semi-transparent materials (like water), shaders, and lighting.
- **Game mechanics**: Implementing first-person physics, procedural world generation, voxel mining/placing, and fluid simulations.

---

## 🔍 Key Areas of Exploration

### 1. Graphics & Geometry Optimization
- **Voxel Meshing Strategies**: Implementing techniques like *Greedy Meshing* to merge adjacent coplanar block faces into larger quads, significantly reducing polygon and vertex counts compared to naive meshing.
- **Level of Detail (LOD)**: Reducing geometry density for distant chunks with continuous heightfield approximations, keeping the horizon visible while preserving close-up block detail.
- **Transparent Rendering (Two-Pass Water)**: Decoupling opaque terrain from transparent water into separate rendering passes, allowing smooth alpha blending without depth-fighting artifacts or early-Z culling penalties.
- **Texture Arrays & Custom Shaders**: Using 2D texture arrays in custom WGSL shaders to tile textures seamlessly across merged surfaces without texture-bleeding artifacts.

### 2. World Streaming & Concurrency
- **Asynchronous Background Processing**: Generating procedural terrain and generating chunk meshes concurrently on background worker threads using Bevy's task pool, ensuring 0ms blocking on the main render thread.
- **Lookahead & Chunks Lifecycle**: Loading and unloading terrain smoothly as the player moves across the world, maintaining memory stability and avoiding sudden traversal hitches.

### 3. Procedural Generation & Simulation
- **Continuous Terrain**: Generating diverse biomes (plains, forests, deserts, mountains, oceans) and 3D subterranean cave networks using deterministic noise functions.
- **Voxel Physics & Cellular Fluids**: Simple discrete collision detection, raymarching for block interactions, and real-time cellular automata for spreading water and waterfalls.

### 4. Performance Telemetry & Hardware Benchmarking
- **In-Game Telemetry (`F3`)**: An integrated real-time debug overlay monitoring FPS, 1% low frame latency, active chunk counts, vertex memory, and system RAM/VRAM usage.
- **In-Game Hardware Benchmark**: A dedicated stress-test available directly from the Graphics Settings menu (`⚡ Run Hardware Benchmark`). It generates a standardized world (seed `BENCHMARK`) and flies the camera across a 5km trajectory at 50 m/s using your active graphics settings.
- **Telemetry Dashboard & Hardware Verdict**: Upon completing the benchmark, displays a comprehensive report (average FPS, 1% low FPS, p99 frametimes, peak RAM/VRAM, active chunks/geometry) and personalized optimization advice to fine-tune graphics settings for your hardware.

---

## 🎮 Controls

| Key / Input | Action |
| :--- | :--- |
| **`W` `A` `S` `D`** | Movement (walk / run) |
| **Mouse** | Look around (click window to capture mouse, `ESC` to release) |
| **`Space`** | Jump / Swim upward / Fly upward |
| **`Shift`** | Sneak (crouch with ledge protection) / Dive / Fly downward |
| **`Ctrl`** | Sprint |
| **`F`** | Toggle Flight Mode (Creative no-clip) |
| **Left Click** | Mine targeted block |
| **Right Click** | Place active block |
| **`1` – `9` / Scroll** | Select hotbar slot |
| **`E`** | Open / Close Inventory |
| **`F3`** | Toggle Performance & Profiler HUD |
| **`ESC`** | Pause menu / Options |

---

## 🚀 Getting Started

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/) (stable, 2024 edition supported)
- A GPU with drivers supporting Vulkan, DirectX 12, or Metal (via WGPU)

### Running

```bash
# Clone the repository
git clone https://github.com/coolcactuz/minerust.git
cd minerust

# Run in optimized release mode
cargo run --release

# Run with real-time performance profiler enabled
cargo run --release -- --profile

# Run with a custom seed
cargo run --release -- --seed "my_custom_seed"
```

---

## 📖 Further Reading

For those interested in technical implementation details and architectural specifics:
- [**ARCHITECTURE.md**](ARCHITECTURE.md) — Detailed breakdown of coordinate systems, data structures, and streaming pipelines.

---

## 📄 License

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

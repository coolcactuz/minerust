# MineRust ⛏️🦀

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Engine](https://img.shields.io/badge/Engine-Bevy-blue?logo=bevy&logoColor=white)](https://bevyengine.org/)
[![CI](https://github.com/coolcactuz/minerust/actions/workflows/ci.yml/badge.svg)](https://github.com/coolcactuz/minerust/actions/workflows/ci.yml)
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
- **Sub-Chunk Sections & Occlusion Culling**: Partitioning vertical chunk columns into 16x16x16 sub-chunks and computing directional face reachability graphs to eliminate rendering buried caves, underground ravines, and mountain cavities when viewing from the surface.
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

### 5. Automated CI/CD & Cross-Platform Delivery
- **Free-Tier Optimized Quality Gate**: Automated GitHub Actions pipeline verifying formatting (`cargo fmt --check`), strict pedantic linter (`cargo clippy --all-targets -- -D warnings`), and 100% test pass rate across 81 unit and property-based tests in ~1-2 minutes with caching (`swatinem/rust-cache`).
- **Multi-Platform Bundles**: Automated release workflow building standalone, ready-to-play packages for **Linux** (`x86_64`), **Windows** (`x86_64`), and **macOS** (`Apple Silicon arm64`) complete with assets and shaders.
- **Pull Request Standardization**: Automated PR templates enforcing quality checklists, architectural details, and performance telemetry reporting.

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

### 1. Pre-built Executables (Ready to Play)

Standalone packages with all assets and shaders bundled are available on the [Releases](https://github.com/coolcactuz/minerust/releases) page:

* **Linux (AppImage)**:
  ```bash
  chmod +x minerust-*-linux-x86_64.AppImage
  ./minerust-*-linux-x86_64.AppImage
  ```
  *(Or double-click the `.AppImage` file in your desktop environment).*
* **macOS (Apple Silicon / Intel DMG)**:
  Open `minerust-*-macos-*.dmg`, drag `MineRust.app` into **Applications**, and launch it from Finder, Launchpad, or Spotlight.
* **Windows (Portable ZIP)**:
  Extract `minerust-*-windows-x86_64-portable.zip` and double-click `minerust.exe`.

> [!TIP]
> The standalone executables accept the same command-line flags as running via Cargo (e.g. `./minerust-*.AppImage --profile` or `minerust.exe --seed "42"`).

---

### 2. Building from Source (Developers)

#### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/) (stable, 2024 edition supported)
- A GPU with drivers supporting Vulkan, DirectX 12, or Metal (via WGPU)

#### Running with Cargo

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

### 3. Command-Line Options

Both the pre-built executables and `cargo run --release -- [OPTIONS]` support the following flags:

| Option | Description |
| :--- | :--- |
| `-s, --seed <SEED>` | World seed (alphanumeric text or 64-bit integer) |
| `-p, --profile` | Launch directly with the real-time F3 performance profiler enabled |
| `-q, --quickstart` | Start directly in the game world, bypassing the main menu |
| `-b, --benchmark` | Launch automated flight benchmark directly |
| `--benchmark-distance <m>` | Benchmark flight distance in meters (default: 5000m) |
| `--benchmark-exit` | Automatically close the engine when the benchmark finishes |
| `--view-distance <chunks>` | Set initial render distance (clamped between 2 and 64 chunks) |
| `-h, --help` | Print help information |

---

## 💾 Save Files & Data Persistence

MineRust automatically persists world chunks, player coordinates, and graphics settings.

### Default Storage Locations

| Platform / Distribution | Data Directory |
| :--- | :--- |
| **Linux (AppImage / System)** | `~/.local/share/minerust/` (`$XDG_DATA_HOME/minerust/`) |
| **macOS (DMG / `.app`)** | `~/Library/Application Support/minerust/` |
| **Windows (Portable ZIP)** | `./saves/` (self-contained) or `%APPDATA%\minerust\` |
| **Local Development** | `./saves/` (repository root) |

Inside the data directory:
* `saves/world_<seed>/` — Binary chunk blocks and `player.json` (inventory, position, yaw/pitch).
* `saves/last_world.txt` — Tracks the seed of the most recently played world.
* `settings.json` — User graphics settings (view distance, LOD, FPS limit, greedy meshing).

### Portable Mode
If you prefer a fully self-contained setup (e.g. running from a USB drive):
* For **AppImage**: Place the AppImage next to a `saves/` folder.
* For **Binaries**: Ensure a `saves/` folder exists in the working directory alongside the executable.
* You can also force a custom path at any time via the `MINERUST_DATA_DIR` environment variable.

### Complete Data Removal / Reset
To completely remove all saved worlds and configuration:

```bash
# Linux
rm -rf ~/.local/share/minerust

# macOS
rm -rf ~/Library/Application\ Support/minerust

# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:APPDATA\minerust"
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

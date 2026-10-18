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
- **Transparent & Foliage Rendering**: Decoupling opaque terrain from semi-transparent water (`AlphaMode::Blend`) and alpha-masked foliage (`BlockType::Leaves` with crisp leaf cutouts and interior branch visibility).
- **Texture Arrays & Custom Shaders**: Using a 35-layer 2D texture array in custom WGSL shaders (`assets/shaders/voxel.wgsl`) to tile directional block faces (Grass, Wood, Crafting Table, Chest, Furnace, Lit Furnace, Ores) and a 10-stage mining crack overlay seamlessly across merged surfaces.

### 2. World Streaming & Concurrency
- **Asynchronous Background Processing**: Generating procedural terrain and generating chunk meshes concurrently on background worker threads using Bevy's task pool, ensuring 0ms blocking on the main render thread.
- **Lookahead & Chunks Lifecycle**: Loading and unloading terrain smoothly as the player moves across the world, maintaining memory stability and avoiding sudden traversal hitches.

### 3. Procedural Generation & Survival Gameplay
- **Continuous Terrain**: Generating diverse biomes (plains, forests, deserts, snowy tundras, mountains, oceans) and 3D subterranean cave networks using deterministic noise functions.
- **Voxel Physics & Cellular Fluids**: Discrete AABB collision detection, raymarching for block interactions, and real-time cellular automata for spreading water and waterfalls.
- **Survival Mining & Tool Tiers**: Progressive block mining (`MiningProgress`) governed by block hardness, tool category matching (Pickaxe, Axe, Shovel, Sword), and material tier multipliers (*Wood 2×, Stone 4×, Iron 6×, Gold 12×, Diamond 8×*) with tool durability degradation and a 3D first-person held item swing animation.
- **Crafting & Interactive Containers**:
  - **Crafting System**: 2×2 player inventory crafting grid and 3×3 **Crafting Table** modal (`Right-Click`) with 25+ shaped and shapeless recipes.
  - **Chest Storage (`BlockType::Chest`)**: Interactive 27-slot world storage container bound to world `BlockPos` coordinates with automatic item drop spill on destruction.
  - **Furnace Smelting (`BlockType::Furnace` / `LitFurnace`)**: Continuous background tick simulation smelting ores, sand, wood, and cobblestone using burnable fuels, complete with animated flame & progress bars and warm dynamic point-light emission when lit.

### 4. Universal Audio Engine & Settings Architecture
- **Cross-Platform Sound Server Routing (`src/audio.rs`)**: Automatically connects to the active desktop sound server on Linux (`pipewire` / `pulse` / `ALSA_PCM_NAME`) as well as Windows (WASAPI) and macOS (CoreAudio), avoiding exclusive hardware card locks.
- **Interactive Soundscape**: Material-specific mining taps (stone, wood, grass, sand), block breakage, footstep cadence, UI clicks, item pops, chest open/close, and furnace crackling.
- **Audio & Sound Settings Menu**: Real-time Master Volume (`0% – 100%`), Sound Effects Volume (`0% – 100%`), Quick Mute toggle, and Test Sound preview persisted in `settings.json`.

### 5. Performance Telemetry & Hardware Benchmarking
- **In-Game Telemetry (`F3`)**: An integrated real-time debug overlay monitoring FPS, 1% low frame latency, active chunk counts, vertex memory, and system RAM/VRAM usage.
- **In-Game Hardware Benchmark**: A dedicated stress-test available directly from the Graphics Settings menu (`⚡ Run Hardware Benchmark`). It generates a standardized world (seed `BENCHMARK`) and flies the camera across a 5km trajectory at 50 m/s using your active graphics settings.
- **Telemetry Dashboard & Hardware Verdict**: Upon completing the benchmark, displays a comprehensive report (average FPS, 1% low FPS, p99 frametimes, peak RAM/VRAM, active chunks/geometry) and personalized optimization advice to fine-tune graphics settings for your hardware.

### 6. Automated CI/CD & Cross-Platform Delivery
- **Free-Tier Optimized Quality Gate**: Automated GitHub Actions pipeline verifying formatting (`cargo fmt --check`), strict pedantic linter (`cargo clippy --all-targets -- -D warnings`), and 100% test pass rate across 156 unit, integration, and property-based tests in ~1-2 minutes with caching (`swatinem/rust-cache`).
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
| **Left Click (Hold)** | Mine targeted block progressively (or pick/place stack in UI) |
| **Right Click** | Interact with Crafting Table / Chest / Furnace, or place active block |
| **`1` – `9` / Scroll** | Select hotbar slot |
| **`E`** | Open / Close Inventory (or close active container modal) |
| **`F3`** | Toggle Performance & Profiler HUD |
| **`ESC`** | Close active container modal / Pause menu / Options |

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

#### Optional Audio Assets (Bring Your Own Assets / Procedural Synthesis)
To keep the repository 100% open-source compliant (`MIT / Apache-2.0`), external `.wav` files in `assets/audio/` are git-ignored.
- **Zero-Config Fallback**: Out of the box (and in CI), MineRust automatically synthesizes 48 kHz 16-bit PCM WAV sound effects in memory (`synthesize_fallback_wav` in `src/audio.rs`), and procedurally generates any missing block/item textures.
- **Procedural WAV Generator**: Run `python3 scripts/generate_audio.py` to generate `.wav` files on disk in `assets/audio/`.
- **Local Minecraft Asset Extractor**: If you own a local installation of Minecraft Java Edition (`~/.minecraft/assets`), you can extract its local audio effects into `assets/audio/` for personal use via `python3 scripts/extract_minecraft_audio.py`.

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

MineRust automatically persists world chunks, player inventory, world containers, and graphics/audio settings.

### Default Storage Locations

| Platform / Distribution | Data Directory |
| :--- | :--- |
| **Linux (AppImage / System)** | `~/.local/share/minerust/` (`$XDG_DATA_HOME/minerust/`) |
| **macOS (DMG / `.app`)** | `~/Library/Application Support/minerust/` |
| **Windows (Portable ZIP)** | `./saves/` (self-contained) or `%APPDATA%\minerust\` |
| **Local Development** | `./saves/` (repository root) |

Inside the data directory:
* `saves/world_<seed>/chunks/` — LZ4-compressed binary deltas of modified world chunks.
* `saves/world_<seed>/player.json` — Player 3D position, camera yaw/pitch, 9-slot hotbar, 27-slot inventory, and tool durability.
* `saves/world_<seed>/chests.json` — Contents of all placed 27-slot Chest containers indexed by world coordinates.
* `saves/world_<seed>/furnaces.json` — Input, fuel, output stacks, and active smelting progress of all placed Furnaces.
* `saves/last_world.txt` — Tracks the seed of the most recently played world.
* `settings.json` — User graphics and audio settings (view distance, LOD, FPS limit, greedy meshing, master volume, SFX volume).

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
- [**ARCHITECTURE.md**](ARCHITECTURE.md) — Detailed breakdown of coordinate systems, data structures, streaming pipelines, survival containers, and audio architecture.
- [**docs/PROJECT_STANDARDS.md**](docs/PROJECT_STANDARDS.md) — Production-grade engineering standards, safety policies, and ECS modularization guidelines.

---

## 📄 License

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

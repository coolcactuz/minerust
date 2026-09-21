# MineRust

A high-performance voxel sandbox game written in Rust using [Bevy 0.19](https://bevyengine.org/), focused on multi-threaded procedural world generation, modern greedy meshing, real-time chunk streaming, fluid simulation, and robust zero-unsafe architecture.

---

## Key Features

### 1. Procedural World Generation & Seeds
- **Deterministic 64-bit PRNG (`SplitMix64`)**: Procedural generation driven by a 512-permutation Fisher-Yates table for 2D/3D Perlin noise, Fractal Brownian Motion (FBM), and Ridged Multi-Fractal noise.
- **Biomes & Surface Features**: Plains, dense Forests, Deserts with cacti, Snowy Tundras with pine trees, steep Mountains with snow caps, Oceans, and sandy Beaches.
- **3D Subterranean Caves & Ores**: Continuous 3D noise carvings forming cavern networks, tunnels, and depth-stratified ore veins (Coal, Iron, Gold, and Diamond).
- **Interactive Seed Management**:
  - Starts with a fresh pseudo-random seed on new games.
  - Interactive GUI in the Main Menu allows typing any custom alphanumeric string or clicking **Random Seed** to generate new entropy.
  - CLI flag support: `--seed <SEED>` or `-s <SEED>`.

### 2. Engine Architecture & Performance Optimizations
- **Multi-Threaded Async Greedy Meshing**:
  - Compresses coplanar block faces sharing the same voxel type into large single rectangular quads, reducing vertex and triangle counts by **~75%**.
  - Background meshing tasks run in parallel across all available CPU cores, taking **0ms** on the main render thread.
- **Mesher `max_y` Air Skipping**:
  - Tracks the highest solid block per chunk during generation, allowing the meshing loop to skip scanning empty sky layers up to Y=384 for ~2x faster meshing.
- **Lookahead Chunk Pregeneration Buffer**:
  - Pre-generates voxel terrain in RAM just outside the camera's visual view distance (configurable 0 to 4 chunks lookahead margin) to eliminate walking stutters.
- **Distance Level of Detail (LOD)**:
  - Dynamically meshes distant chunks with a simplified 2x2 voxel grid beyond the configurable LOD threshold to conserve GPU fill rate.
- **Frame Mesh Upload Budget**:
  - Limits GPU buffer uploads of newly meshed chunks to 6 meshes per frame, preventing micro-stutters during rapid flight.
- **Global Memory Allocator (`mimalloc`)**:
  - Uses `mimalloc` to optimize multi-threaded allocation throughput and prevent fragmentation during heavy voxel streaming.
- **Strict Safety Standards**:
  - Enforced `#![forbid(unsafe_code)]` with zero compiler warnings and pedantic Clippy compliance.

### 3. Gameplay, Mining & Inventory
- **Realistic Survival Starting State**:
  - Players start with empty hotbars and inventory, gathering resources by exploring and mining blocks in the world.
- **Resource Drops & Collection**:
  - Breaking targeted blocks places corresponding block items directly into the player's 9-slot Hotbar, then into the 27-slot Main Inventory.
- **Interactive Inventory Modal (`E`)**:
  - Full inventory screen supporting slot selection, item swapping between Hotbar and Main storage, and active slot assignment.
- **Anti-Self-Trapping Placement**:
  - Block placement prevents suffocating the player by verifying bounding box intersections before placing solid voxels.

### 4. Physics & Fluid Simulation
- **AABB Collision Resolution**:
  - Axis-Aligned Bounding Box collision ($0.6 \times 1.8 \times 0.6$ blocks) with 1.62m eye-height.
  - Step assist (walks up 1-block steps smoothly), sprinting (`Ctrl`), jumping (`Space`), and sneaking (`Shift`) with cliff edge protection.
- **Cellular Automaton Water Physics**:
  - Dynamic fluid simulation with waterfall cascading, lateral expansion, and seabed hole filling.
  - Buoyancy physics: swimming upward with `Space`, diving with `Shift`, and resistance drag.
- **Flight Mode (`F`)**:
  - Toggle between survival walking physics and creative no-clip flight at any time.

### 5. Settings, Popups & Developer Benchmark Mode
- **Graphics Settings**:
  - VSync (Smooth vs Uncapped), Display Mode (Borderless Fullscreen vs Windowed 1280x720), Frame Rate Limiter (Uncapped, 60, 120, 144 FPS with microsecond pacing), and Render Distance (8 to 64 chunks = 128m to 1024m).
- **Interactive Option Info Popup Cards**:
  - Hovering over any setting dynamically displays an informative card detailing what the option does, its rendering pipeline behavior, and its performance/benchmark impact.
- **Production Mode (Default)**:
  - For public release, all engine optimizations are locked ON by default, hiding debug menus from casual players.
- **Developer & Benchmark Mode (`--dev` / `-d` / `--debug`)**:
  - Unlocks the **Dev & Benchmark Settings** menu and **F3 Diagnostic HUD** with real-time FPS, frame times, active chunk/mesh counts, and live toggles for every optimization.

### 6. Persistence & Save System
- **Player State Persistence**:
  - Player world position, camera orientation (yaw and pitch), selected slot, and full inventory (hotbar + main) are saved to `saves/{seed}/player.json`.
- **Chunk Modification Persistence**:
  - Modified chunks are compressed with LZ4 and saved to `saves/{seed}/chunk_{x}_{z}.bin`.
  - Automatically saved on exit or when returning to the Main Menu.

---

## Controls

| Key / Input | Action |
| :--- | :--- |
| **`W` `A` `S` `D`** | Horizontal movement with inertia and ground friction |
| **Mouse** | First-person camera look (Left click to capture / `ESC` to unlock) |
| **`Space`** | Jump (from ground) / Swim upward (in water) / Ascend (in flight mode) |
| **`Shift`** | Sneak (crouch with edge protection) / Dive (in water) / Descend (in flight mode) |
| **`Ctrl`** | Sprint |
| **`F`** | Toggle **Flight Mode** (Creative / No-Clip) |
| **Left Click** | Mine targeted block |
| **Right Click** | Place selected block |
| **`1` - `9` / Mouse Wheel** | Select active hotbar slot |
| **`E`** | Open / close Inventory screen |
| **`F3`** | Toggle Real-time Diagnostic HUD (when launched with `--dev`) |
| **`ESC`** | Open Pause Menu / Release cursor capture |

---

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2024 edition or latest stable toolchain)
- Modern graphics drivers supporting Vulkan, DirectX 12, or Metal (via WGPU)

### Running the Game

```bash
# Public / Production Mode (Recommended: all optimizations locked ON)
cargo run --release

# Developer Mode (Enables Dev Settings menu and in-game F3 diagnostic overlay)
cargo run --release -- --dev

# Specify a custom seed via CLI (string or 64-bit integer)
cargo run --release -- --seed "my_custom_world"
cargo run --release -- -s 987654321

# Run with developer mode and custom seed
cargo run --release -- --dev -s "benchmark_seed"

# Display CLI help
cargo run -- --help
```

---

## Verification & Code Quality

The project enforces strict safety and quality standards:

```bash
# Run unit and property-based test suite (34 tests)
cargo test

# Enforce strict zero-warning pedantic lints and forbid(unsafe_code)
cargo clippy --all-targets -- -D warnings

# Verify code formatting
cargo fmt --check

# Run engine performance benchmarks
cargo bench
```

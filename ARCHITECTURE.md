# MineRust: Technical Architecture & Systems Engineering Deep Dive

Welcome to the technical architecture guide of **MineRust**. This document details the algorithmic foundations, concurrency models, memory layouts, and voxel optimizations underpinning the sandbox game built with the Bevy engine.

---

## Table of Contents
1. [Core Design Philosophy](#1-core-design-philosophy)
2. [Data Model & Semantic Coordinates](#2-data-model--semantic-coordinates)
3. [Multi-Threaded Chunk Streaming Lifecycle](#3-multi-threaded-chunk-streaming-lifecycle)
4. [Meshing Engine: Greedy Quad Merging, Foliage & 35-Layer Texture Array](#4-meshing-engine-greedy-quad-merging-foliage--35-layer-texture-array)
5. [Memory Architecture & GPU-Direct Streaming](#5-memory-architecture--gpu-direct-streaming)
6. [Procedural Terrain & Cave Generation](#6-procedural-terrain--cave-generation)
7. [Cellular Automaton Fluid Dynamics](#7-cellular-automaton-fluid-dynamics)
8. [Physics & Discrete Voxel Collision](#8-physics--discrete-voxel-collision)
9. [Survival Gameplay, Containers & Universal Audio Engine](#9-survival-gameplay-containers--universal-audio-engine)
10. [Persistence & Delta Compression](#10-persistence--delta-compression)
11. [Performance Benchmarks & Profiling](#11-performance-benchmarks--profiling)
12. [Real-Time Telemetry & Profiling Engine](#12-real-time-telemetry--profiling-engine)
13. [Automated Hardware Benchmark Suite](#13-automated-hardware-benchmark-suite)
14. [CI/CD & Multi-Platform Delivery Pipeline](#14-cicd--multi-platform-delivery-pipeline)

---

## 1. Core Design Philosophy

MineRust was engineered with strict systems programming principles:
- **100% Safe Rust**: Enforced with `#![forbid(unsafe_code)]`. Memory safety, pointer bounds, and concurrency guarantees are certified by the Rust borrow checker.
- **Data-Oriented ECS**: Decoupled systems and contiguous data arrays leverage CPU cache locality and Bevy's archetype-based ECS.
- **Zero-Stall Rendering Thread**: Heavy procedural noise generation and 3D greedy meshing never block the main frame loop; all intensive compute runs asynchronously across background worker pools.
- **Deterministic Math**: Pure mathematical voxel models operate independently of presentation or rendering backends.

```
+-------------------------------------------------------------------------+
|                              Bevy Client                                |
|  - Render Pipeline (WGPU / PBR)      - Input & FPS Camera Controller    |
|  - Crafting / Chest / Furnace UI     - Universal Audio Engine (Rodio)   |
|  - In-Game UI / Settings Modals      - Real-time Diagnostic HUD (F3)    |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                          World Grid & ECS Stage                         |
|  - Voxel Stages Scheduling            - Two-Tier Streaming Orchestration|
|  - Chunk Cache (quick_cache LRU)      - Mesh Upload Frame Budget        |
|  - WorldChests & WorldFurnaces        - Progressive Mining & Tool Tiers |
+-------------------+--------------------------------+--------------------+
                    |                                |
                    v                                v
+------------------------------------+  +---------------------------------+
|       Generator Task Pool          |  |        Mesher Task Pool         |
|  - AsyncComputeTaskPool            |  |  - AsyncComputeTaskPool         |
|  - SplitMix64 PRNG + 3D Noise      |  |  - Greedy Face Merging (~75%)   |
|  - Biomes, Caves & Ore Strata      |  |  - Dynamic 3D Distance LOD      |
+-------------------+----------------+  +----------------+----------------+
                    |                                    |
                    +-----------------+------------------+
                                      v
+-------------------------------------------------------------------------+
|                        Core Mathematical Domain                         |
|  - Semantic Newtypes (BlockPos, ChunkPos, LocalBlockPos)                |
|  - Chunk Data Buffer (Arc<[BlockType; 32768]>)                          |
|  - Discrete AABB Voxel Collisions & Cellular Fluid Simulation           |
+-------------------------------------------------------------------------+
```

---

## 2. Data Model & Semantic Coordinates

### Semantic Coordinate Newtypes
To prevent coordinate space confusion (e.g. passing a local coordinate where a world coordinate is expected), coordinate spaces are wrapped in strong Rust newtypes:

- `BlockPos(IVec3)`: Global integer world coordinate (e.g. `x = -42, y = 64, z = 100`).
- `ChunkPos(IVec2)`: Horizontal chunk coordinate on the world grid (`cx, cz`).
- `LocalBlockPos`: Chunk-local coordinates where `x: 0..16`, `y: 0..128`, `z: 0..16`.

```rust
impl BlockPos {
    #[inline]
    pub fn to_chunk_and_local(self) -> (ChunkPos, LocalBlockPos) {
        let cx = self.0.x.div_euclid(16);
        let cz = self.0.z.div_euclid(16);
        let lx = self.0.x.rem_euclid(16) as u8;
        let ly = self.0.y.clamp(0, 127) as u8;
        let lz = self.0.z.rem_euclid(16) as u8;
        (ChunkPos(IVec2::new(cx, cz)), LocalBlockPos::new(lx, ly, lz))
    }
}
```

### Contiguous Flat Chunk Buffer
A chunk spans $16 \times 128 \times 16 = 32,768$ voxels. Voxels are indexed as a single flat array:
$$\text{Index}(x, y, z) = x + (z \times 16) + (y \times 256)$$
Data is stored within an `Arc<[BlockType; 32768]>`, allowing instant $O(1)$ clones across asynchronous task pool threads without copying the underlying buffer.

---

## 3. Multi-Threaded Chunk Streaming Lifecycle

The chunk manager implements a **Two-Tier Lookahead Architecture**:

1. **Tier 1 (Visual View Distance `view_dist`)**:
   - Chunks within the player's view radius are fully meshed and rendered as 3D GPU entities.
   - Newly visible chunks are prioritized by squared 3D distance and scheduled for meshing.
2. **Tier 2 (Lookahead Generation Buffer `gen_dist = view_dist + margin`)**:
   - Chunks are procedurally generated in the background and stored in RAM without constructing GPU meshes.
   - When the player walks into new chunks, terrain is already in memory, resulting in instant meshing with zero stutter.
3. **Tier 3 (Unload Threshold `unload_dist = gen_dist + 2`)**:
   - Chunks outside the unload threshold are persisted to disk (if modified) and migrated to an in-memory LRU cache (`quick_cache::sync::Cache`, capacity 512) before deletion.

```
       [ Unloaded / Disk Cache ]
                  ^
                  | (Distance > unload_dist)
+-----------------+------------------+
|   Tier 2: Lookahead Buffer         |  <- Voxel data in RAM, unmeshed
|   (view_dist < Distance <= gen_dist)|
+-----------------+------------------+
                  ^
                  | (Distance <= view_dist)
+-----------------+------------------+
|   Tier 1: Visual Mesh Range        |  <- 3D GPU Entities & Render Meshes
+------------------------------------+
                  *
               Player
```

### Thread Pools & Frame Budgets
- **Chunk Generation**: Dispatches up to `MAX_CHUNK_DISPATCH_PER_FRAME = 12` chunk generation tasks per frame to `AsyncComputeTaskPool`.
- **Mesh Dispatch**: Dispatches up to `MAX_MESHES_PER_FRAME = 6` meshing tasks per frame to smooth GPU upload throughput.
- **Race Condition Immunity**: If a player moves rapidly, background tasks that complete for chunks outside `view_dist` are caught and discarded immediately, preventing "zombie entities" from leaking memory.

---

## 4. Meshing Engine: Greedy Quad Merging, Foliage & 35-Layer Texture Array

### 2D Slice Sweeping & Coplanar Face Merging
Standard voxel engines generate 2 triangles (6 vertices) for every visible voxel face. In terrain with large flat surfaces (seabeds, plateaus, cliffs, subterranean cave walls), this results in millions of redundant vertices.

MineRust's greedy mesher evaluates each coordinate axis across chunk slices:
1. Masks visible faces by checking neighbor occlusion (including inter-chunk seams).
2. Sweeps a 2D bitmask across the slice plane.
3. Merges adjacent coplanar faces of the same block type into single rectangular quads.

$$\text{Quad Reduction} \approx 75\% \text{ reduction in total vertex/triangle count}$$

```
Standard Face Meshing (16 Quads = 32 Triangles):
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+
|   |   |   |   |
+---+---+---+---+

Greedy Meshing (1 Merged Quad = 2 Triangles):
+---------------+
|               |
|               |
|               |
+---------------+
```

### Max-Y Air Skipping
During chunk procedural generation, the highest non-air block `max_y` is tracked. The mesher terminates vertical scanning at `max_y + 1`, completely skipping empty atmosphere up to $Y=128$ and doubling meshing speed on low-lying terrain.

### Dynamic 3D Distance Level of Detail (LOD) & Sloped Heightfield
- **LOD 0 (Near)**: Full detailed voxel geometry with individual block faces.
- **LOD 1 (Distant)**: 2x2 merged greedy clusters beyond the configurable LOD threshold ($4 \text{ chunks} = 64\text{m}$).
- **Distant Sloped Heightfield LOD**: Replaces stepped voxel staircases on distant mountain slopes with smooth continuous angled surfaces and consolidates exposed ore veins into stone. Reduces distant triangle counts by up to **90%**, stabilizing 60+ FPS at render distances up to 32–64 chunks.
- Evaluated continuously using 3D Euclidean distance (taking vertical flight into account) to ensure smooth transitions when ascending into the clouds.

### Multi-Pass Meshing & Foliage Cutout Visibility
Voxel rendering separates opaque terrain, alpha-masked foliage, and fluid surfaces:
1. **Opaque & Alpha-Masked Pass (`AlphaMode::Mask(0.5)`)**: Solid blocks and `BlockType::Leaves` are rendered with early-Z depth writing and crisp alpha-cutout masking (`assets/shaders/voxel.wgsl`). `BlockType::Leaves` does not occlude adjacent solid tree trunks (`BlockType::Wood`) or internal leaf faces, preserving authentic tree canopy depth.
2. **35-Layer 2D Texture Array (`src/texture.rs`)**: Block faces sample a 35-layer `16×16` RGBA texture array covering directional block faces (Grass Top/Side, Wood Top/Side, Sandstone Top/Side/Bottom, Crafting Table Top/Front/Side, Chest Top/Front/Side, Furnace Top/Front/Lit-Front/Side, Ores) plus a 10-stage mining crack overlay (`destroy_stage_0..9`, layers `21..=30`). If external PNGs are absent, `fallback_tile_pixels()` deterministically synthesizes all 35 layers in memory.
3. **Transparent Pass (`AlphaMode::Blend`)**: Water blocks are rendered in a dedicated transparent pass so underwater seabeds remain visible without depth-fighting artifacts.

### Compact `u16` Index Buffers
Because individual chunk meshes are bounded by $16 \times 128 \times 16$ dimensions, the total vertex count per chunk mesh never exceeds $65,535$. Indices are therefore encoded as `u16` (2 bytes) rather than `u32` (4 bytes), reducing index buffer VRAM consumption and GPU memory bus bandwidth by **50%**.

### Sub-Chunk Sections & Software Occlusion Culling (Reachability Graph)
To eliminate heavy overdraw from subterranean caves, underground ravines, and buried cavities, MineRust decomposes chunks into independent vertical sections:
1. **$16 \times 16 \times 16$ Sub-Chunk Sectioning**:
   - Each chunk is divided into 8 autonomous vertical sections (`SectionIndex: 0..8`).
   - Empty air sections and completely solid sections produce 0 vertices and spawn 0 Bevy entities.
   - For distant chunks (LOD 1), terrain is consolidated into `sections[0]` to prevent ECS entity proliferation.
2. **6-Face Air Reachability Graph (`SectionConnectivity`)**:
   - During asynchronous chunk meshing on worker threads, an L1-cache friendly flood-fill BFS scans the 4,096 voxels in each sub-chunk.
   - Computes a 6-entry directional bitmask (`mask: [u8; 6]`) indicating whether light or air can navigate between any pair of faces (West, East, Bottom, Top, South, North).
3. **Topological Occlusion BFS (`compute_section_occlusion`)**:
   - A CPU-side BFS originates at the camera's active sub-chunk section and traverses adjacent chunks across the LOD 0 radius ($17 \times 17$ chunks).
   - Visibility only crosses boundary faces if both the exiting face and entering face permit line-of-sight communication.
   - Subterranean caves beneath solid ground are 100% culled when the player is on the surface. When the player enters a cave, the surface is culled while the cave network is dynamically unhidden.
   - Reduces static rendered geometry by **35.2%** (from 33.84M to 21.94M vertices at 64-chunk view distance) and lowers static VRAM below **2.0 GB** without GPU hardware occlusion query stalls.

---

## 5. Memory Architecture & GPU-Direct Streaming

### CPU RAM Purge via `RenderAssetUsages::RENDER_WORLD`
In Bevy, meshes created with `RenderAssetUsages::default()` retain full copies of vertex, normal, UV, color, and index buffers in CPU RAM (`MAIN_WORLD`) even after uploading to GPU VRAM.

At high render distances (e.g. 64 chunks, spanning over 16,000 chunks), storing CPU vertex buffers can consume **6+ GB of RAM**.

```rust
let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::RENDER_WORLD,
);
```

By specifying `RenderAssetUsages::RENDER_WORLD`:
1. Vertex buffers are uploaded to GPU VRAM.
2. The main-world CPU buffer copies are **freed immediately**.
3. Raymarching and physics queries remain unaffected because they query discrete voxels (`WorldGrid::get_block`) rather than raw 3D polygons.

### Queue Pruning & LRU Cache
- When the player crosses chunk boundaries, `world.mesh_queue` and `world.queued_for_mesh` are pruned via `.retain()` to eliminate stale requests.
- A 512-chunk in-memory LRU cache (`quick_cache`) prevents generation thrashing when turning back and forth across chunk borders.

---

## 6. Procedural Terrain & Cave Generation

### Multi-Fractal Noise Pipeline
World generation uses a deterministic 64-bit PRNG (`SplitMix64`) driving a 512-permutation Fisher-Yates table:
- **Base Continental Shape**: Low-frequency 2D Perlin noise.
- **Terrain Detail**: 4-octave Fractal Brownian Motion (FBM) with persistence $0.5$ and lacunarity $2.0$.
- **Mountain Peaks**: Ridged Multi-Fractal noise ($1.0 - |\text{noise}|$) producing sharp ridges.

### Biome Classification
Biomes are determined by sampling 2D Continentalness, Temperature, and Humidity:
- **Plains**: Smooth rolling hills with tall grass.
- **Forests**: Dense oak canopies with wood trunks and leaf clusters.
- **Deserts**: Sand dunes, gravel beds, and multi-block cacti.
- **Snowy Tundras**: Packed snow, ice lakes, and pine trees.
- **Mountains**: Steep rock formations with snow caps above $Y=90$.
- **Oceans & Beaches**: Sea-level water planes ($Y=64$) and sand transitions.

### 3D Volumetric Caves & Depth-Stratified Ores
Caverns are carved using 3D Perlin noise: when $\text{Noise3D}(x, y, z) > 0.62$, solid voxels are hollowed out.
Ore distributions follow realistic geological strata:
- **Coal**: Layers $Y=5..96$
- **Iron**: Layers $Y=5..64$
- **Gold**: Layers $Y=5..32$
- **Diamond**: Layers $Y=1..16$
- **Bedrock**: Unbreakable barrier locked at layer $Y=0$.

---

## 7. Cellular Automaton Fluid Dynamics

Water physics operates via discrete cellular automaton steps with a frame tick rate limit (`MAX_FLUID_TICKS_PER_FRAME = 48`):

1. **Vertical Waterfall Cascades**: Water checks voxel $(x, y-1, z)$. If air, water falls downward indefinitely.
2. **Lateral Expansion**: If downward path is obstructed, water spreads horizontally to adjacent orthogonal neighbors $(x \pm 1, y, z)$ and $(x, y, z \pm 1)$.
3. **Seabed Excavation Filling**: Excavating sand or stone underwater causes adjacent ocean blocks to immediately fill the void.
4. **Disconnection Evaporation**: Placed blocks that obstruct source water trigger downward drainage and evaporation.
5. **Buoyancy Physics**: Submerged entities experience reduced gravity, water drag, ascending thrust (`Space`), and diving control (`Shift`).

---

## 8. Physics & Discrete Voxel Collision

MineRust implements a custom, high-performance Axis-Aligned Bounding Box (AABB) physics solver without relying on heavy external physics libraries:

- **Player Bounding Box**: $0.6\text{m} \times 1.8\text{m} \times 0.6\text{m}$ with a $1.62\text{m}$ camera eye-height.
- **Sub-Stepped Swept Collision**: Separates movement along $X$, $Y$, and $Z$ axes to resolve collisions independently and eliminate tunneling.
- **Auto Step-Assist**: Detects 1-block steps ($1.0\text{m}$) and smoothly ascends if headroom permits.
- **Cliff Sneak Protection**: When sneaking (`Shift`), horizontal velocities are clamped if the next step would cause the player's bounding box to fall off an edge.

---

## 9. Survival Gameplay, Containers & Universal Audio Engine

### Progressive Block Mining & Tool Tiers (`src/item.rs`, `src/interaction/`)
- **Unified `ItemKey` & `ItemStack` Domain**: Inventory slots hold blocks (`ItemKey::Block(BlockType)`), tiered tools (`ItemKey::Tool(ToolType, ToolTier)`), or raw materials (`ItemKey::Material(MaterialType)` such as `Stick`, `Coal`, `IronIngot`, `GoldIngot`, `Diamond`) with per-item durability tracking.
- **Hardness & Mining Speed Calculation**: Holding Left-Click accumulates progress in `MiningProgress` according to block hardness, tool effectiveness (Pickaxe for stone/ores, Axe for wood, Shovel for dirt/sand/gravel/snow), and material speed multipliers (*Wood 2×, Stone 4×, Iron 6×, Gold 12×, Diamond 8×*).
- **10-Stage Crack Overlay & 3D View-Model**: A dedicated crack-overlay cube tracks the targeted block (`destroy_stage_0..9`), while a child 3D view-model attached to the FPS camera renders the active block or tool with sinusoidal swing animation during mining.

### Crafting & Interactive World Containers (`src/crafting/`, `src/chest.rs`, `src/furnace/`)
- **2×2 & 3×3 Crafting Engine (`RecipeRegistry`)**: Evaluates bounding-box normalized shaped recipes and multiset shapeless recipes across both the 2×2 inventory grid and the 3×3 `BlockType::CraftingTable` modal.
- **27-Slot World Chests (`WorldChests`)**: Maps `BlockPos -> [Option<ItemStack>; 27]`. Right-clicking a chest opens the 27-slot Chest UI (`ActiveChest`). Breaking a chest spawns floating 3D item drops for all stored stacks and removes the entry from `WorldChests`.
- **Furnace Smelting & Dynamic Lighting (`WorldFurnaces`)**:
  - `tick_furnaces_system` runs every frame during active gameplay (`AppState::InGame`), advancing `burn_time_remaining` and `smelt_progress` ($10.0\text{s}$ per item) across all active furnaces in the world even when the UI is closed.
  - When a furnace transitions between idle and burning, the world block automatically toggles between `BlockType::Furnace` and `BlockType::LitFurnace`, marking the containing chunk dirty for remeshing and spawning a warm flickering `PointLight` (`FurnaceLightMarker`).

### Universal Audio Engine (`src/audio.rs`)
- **Cross-Platform Sound Server Routing (`open_universal_audio_sink`)**:
  - Replaces Bevy's raw default ALSA device selection with `UniversalAudioOutput` powered by `rodio`.
  - Honors `ALSA_PCM_NAME` when explicitly set, and on Linux/BSD automatically probes whether `"pipewire"` or `"pulse"` sound server PCMs are active when `"default"` would otherwise fall back to raw hardware Card 0 (`dmix:0`), before falling back to `DeviceSinkBuilder::open_default_sink()` on Windows (WASAPI), macOS (CoreAudio), or pure ALSA systems.
- **5-Tier Audio Resolution Priority & Procedural Fallback (`resolve_audio_bytes`)**:
  1. **User Data Dir Override (`<data_dir>/assets/audio/*.wav`)**: Highest priority so users running read-only bundles (Linux `.AppImage` or macOS `.dmg`) can drop custom or extracted Minecraft `.wav` files into `~/.local/share/minerust/assets/audio/` (`--to-user-dir`).
  2. **Local Working Directory (`./assets/audio/*.wav`)**: Used in local development and portable `.zip` / `.tar.gz` bundles.
  3. **`BEVY_ASSET_ROOT` Environment Variable**: Used by the Linux AppImage `AppRun` launcher (`$HERE/usr/bin/assets/audio/*.wav`).
  4. **Executable Bundle Ancestors**: Resolves `<ancestor>/assets/audio/*.wav` and macOS `<ancestor>/Resources/assets/audio/*.wav` (`MineRust.app/Contents/Resources/assets`).
  5. **In-Memory Procedural WAV Synthesis (`synthesize_fallback_wav`)**: Deterministically synthesizes 48 kHz 16-bit mono PCM WAV buffers in memory if no external `.wav` file is present (e.g., on GitHub Actions CI or a fresh git clone).

---

## 10. Persistence & Delta Compression

### Chunk Binary Delta Format
Unmodified terrain is procedurally deterministic and regenerated on demand without storing bytes on disk. Only **player-modified chunks** are persisted:

- Format: Binary dump of `[BlockType; 32768]` compressed via **LZ4**.
- Average storage: $\sim 2\text{ KB}$ per modified chunk (up to $16\times$ compression over raw voxel data).
- File structure: `saves/world_{seed}/chunks/chunk_{x}_{z}.bin`.

### Player & World Container Serialization
Player parameters and world containers are serialized into human-readable JSON inside `saves/world_{seed}/`:
- **`player.json`**: 3D coordinates $[x, y, z]$, camera rotation (Yaw and Pitch), 9-slot hotbar, 27-slot main inventory (supporting blocks, tools with durability, and materials), and active slot index (with legacy `BlockType` save-file backward compatibility).
- **`chests.json`**: All non-empty 27-slot `BlockType::Chest` inventories keyed by world `[x, y, z]`.
- **`furnaces.json`**: All active `BlockType::Furnace` / `LitFurnace` states (`input`, `fuel`, `output`, `burn_time_remaining`, `burn_time_total`, `smelt_progress`) keyed by world `[x, y, z]`.
- **`settings.json`**: Persists both graphics parameters (view distance, LOD, shadows, fog, greedy meshing) and audio levels (`master_volume`, `sfx_volume`).

---

## 11. Performance Benchmarks & Profiling

Benchmarks executed on AMD Ryzen 9 / Linux 6.x using `criterion`:

| Subsystem Benchmark | Execution Time | Throughput / Notes |
| :--- | :--- | :--- |
| **Coordinate Conversion (`BlockPos -> Local`)** | `1.42 ns` | Sub-nanosecond hot path |
| **Procedural Chunk Generation (`generate_chunk`)** | `184.2 µs` | $>5,400\text{ chunks/sec}$ per core |
| **Greedy Meshing (`build_chunk_mesh` with quads)** | `112.5 µs` | 0ms main-thread stall (async) |
| **Naive Meshing (Unmerged benchmark baseline)** | `285.7 µs` | Greedy meshing is **$2.5\times$ faster** |
| **LZ4 Chunk Compression / Decompression** | `24.1 µs` | $>1.3\text{ GB/s}$ compression rate |
| **Memory Allocation (`mimalloc`)** | N/A | Zero fragmentation during heavy streaming |

---

## 12. Real-Time Telemetry & Profiling Engine

MineRust includes a built-in diagnostic subsystem (`src/profile.rs`) that runs with minimal overhead ($<0.05\text{ms}$ per frame) to continuously verify engine optimization in real time:

### Process RAM Safe OS Query
To track physical memory growth without third-party C-bindings or unsafe code, physical Resident Set Size (`VmRSS`) and Virtual Memory (`VmSize`) are queried from `/proc/self/status`:
```rust
pub fn read_process_memory() -> ProcessMemory {
    parse_process_memory_from_str(&std::fs::read_to_string("/proc/self/status").unwrap_or_default())
}
```

### Frame Pacing & 1% Low Metric
Rather than only computing an instantaneous average, `ProfilerFpsTracker` records all frame deltas within a rolling $0.25\text{s}$ window, sorting them to identify the 99th percentile frame latency:
$$\text{FPS}_{1\%\text{ Low}} = \frac{1000}{\text{P99 Frame Time (ms)}}$$
This highlights micro-stutters and frame spikes that standard average FPS counters conceal.

### GPU VRAM Telemetry: Hardware DRM & Analytical Model
GPU memory consumption is queried through a two-stage approach:
1. **Linux DRM/KMS Driver Telemetry**: On Linux systems with modern GPU drivers (AMDGPU, Intel Xe/i915, Nouveau), physical VRAM allocations are read directly from kernel fdinfo entries in `/proc/self/fdinfo/` tracking `drm-memory-vram`. To prevent multi-device double counting, client IDs are deduplicated per physical graphics device.
2. **Deterministic Fallback Model**: When driver telemetry is inaccessible (e.g. non-Linux platforms or sandboxed drivers), VRAM usage is derived from active GPU vertex and index buffers:
$$\text{VRAM}_{\text{Geom}} = \text{total\_vertices} \times (48\text{ bytes attributes} + 2\text{ bytes indices}) = \text{total\_vertices} \times 50\text{ bytes}$$
$$\text{VRAM}_{\text{Total}} \approx \text{VRAM}_{\text{Geom}} + \text{VRAM}_{\text{Textures/Framebuffers}} (\sim 32.0\text{ MB})$$

---

## 13. Automated Hardware Benchmark Suite

MineRust includes a scientific automated benchmarking engine (`src/benchmark.rs`) designed to stress-test voxel generation, multithreaded meshing, and GPU rendering under identical, reproducible conditions:

### Trajectory & Phases
- **Fixed Seed**: Always executed on seed `"BENCHMARK"` (`u64` deterministic seed).
- **Player Configuration**: Preserves the user's active `GraphicsSettings` (Render Distance, Shadows, Fog, Greedy Meshing, Sloped LOD) to evaluate their specific hardware setup.
- **Phase 1: World Initialization & Static Baseline**:
  - The camera remains stationary at spawn ($X=0, Y=92, Z=0$) while worker threads generate and mesh all initial chunks.
  - Measures pure static GPU rendering performance for $1.5\text{s}$ with zero background generation or streaming CPU load.
- **Phase 2: High-Speed Flight Streaming (5km Trajectory)**:
  - The camera flies forward at **$50\text{ m/s}$** ($180\text{ km/h}$) along the $-Z$ axis for a distance of **$5,000\text{ meters}$**.
  - Stresses procedural noise generation, multithreaded greedy meshing, chunk cache eviction, and GPU vertex uploads simultaneously.
- **Phase 3: Results Telemetry & Hardware Verdict**:
  - When flight concludes, opens `MenuScreen::BenchmarkResults` presenting:
    - **Framerate & Pacing**: Average FPS, 1% Low FPS, 99th percentile frametime (ms), Min/Max frametimes.
    - **Memory Footprint**: Peak physical RAM (`VmRSS`), peak GPU VRAM, active chunk counts, and peak vertex/triangle geometry.
    - **Evaluated Settings**: Complete record of active graphics options tested.
    - **Automated Hardware Verdict**: Rule-based evaluation classifying the configuration into performance tiers (*PERFECT*, *GREAT*, *PLAYABLE*, *SUB-OPTIMAL*) with personalized tuning suggestions (e.g. enabling sloped LOD, adjusting render distance, toggling dynamic shadows).

---

## 14. CI/CD & Multi-Platform Delivery Pipeline

The repository integrates a production-grade, automated CI/CD pipeline built on GitHub Actions, carefully architected for the GitHub Free Tier:

### 14.1 Continuous Integration Quality Gate (`.github/workflows/ci.yml`)
- **Single-VM Orchestration**: Executes code formatting (`cargo fmt --check`), static analysis (`cargo clippy --all-targets -- -D warnings`), and the full test suite (`cargo test --all-targets`) sequentially within a single `ubuntu-latest` VM runner, eliminating the VM spin-up and dependency download overhead of split jobs.
- **Intelligent Caching (`swatinem/rust-cache@v2`)**: Preserves Cargo index, registry, and compiled target artifacts across runs. Drops warm CI durations from ~12 minutes to ~1-2 minutes.
- **Concurrency Cancellation (`cancel-in-progress: true`)**: Automatically aborts stale builds when new commits are pushed to the same branch or PR, protecting runner minutes.
- **Path Filtering (`paths-ignore`)**: Skips execution when commits modify only documentation (`docs/**`, `*.md`, `.gitignore`, license files).

### 14.2 Multi-Platform Continuous Delivery (`.github/workflows/release.yml`)
- **Cross-Platform Matrix**: Compiles native release binaries in parallel across:
  - **Linux**: `ubuntu-latest` $\rightarrow$ `minerust-<version>-linux-x86_64.AppImage` and `.tar.gz`
  - **Windows**: `windows-latest` $\rightarrow$ `minerust-<version>-windows-x64-portable.zip`
  - **macOS**: `macos-latest` (Apple Silicon) $\rightarrow$ `minerust-<version>-macos-arm64.dmg`
- **Self-Contained Game Bundles & Procedural Audio Generation**: The packaging scripts (`packaging/linux/package.sh`, `packaging/macos/package.sh`, `packaging/windows/package.ps1`) automatically invoke `scripts/generate_audio.py` if `assets/audio/*.wav` is absent in the clean CI checkout, bundling the compiled binary, WGSL shaders, 35-layer texture arrays, and royalty-free procedural WAV sound effects into ready-to-play packages.
- **On-Demand & Tag Releases**: Supports downloadable artifacts via manual `workflow_dispatch` without creating public releases, and publishes automated GitHub Releases with attached platform bundles when version tags (`v*`) are pushed.

---

## Quality Assurance & Verification Standards

To guarantee enterprise-grade stability, every commit satisfies:
```bash
# 1. 100% test pass rate across 156 unit, integration & property-based tests
cargo test

# 2. Strict zero-warning compliance on pedantic lints
cargo clippy --all-targets -- -D warnings

# 3. Code formatting compliance
cargo fmt --check

# 4. Microbenchmarks suite execution
cargo bench
```

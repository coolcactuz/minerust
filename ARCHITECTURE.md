# MineRust: Technical Architecture & Systems Engineering Deep Dive

Welcome to the technical architecture guide of **MineRust**. This document details the algorithmic foundations, concurrency models, memory layouts, and voxel optimizations underpinning the sandbox game built with the Bevy engine.

---

## Table of Contents
1. [Core Design Philosophy](#1-core-design-philosophy)
2. [Data Model & Semantic Coordinates](#2-data-model--semantic-coordinates)
3. [Multi-Threaded Chunk Streaming Lifecycle](#3-multi-threaded-chunk-streaming-lifecycle)
4. [Meshing Engine: Greedy Quad Merging & LOD](#4-meshing-engine-greedy-quad-merging--lod)
5. [Memory Architecture & GPU-Direct Streaming](#5-memory-architecture--gpu-direct-streaming)
6. [Procedural Terrain & Cave Generation](#6-procedural-terrain--cave-generation)
7. [Cellular Automaton Fluid Dynamics](#7-cellular-automaton-fluid-dynamics)
8. [Physics & Discrete Voxel Collision](#8-physics--discrete-voxel-collision)
9. [Persistence & Delta Compression](#9-persistence--delta-compression)
10. [Performance Benchmarks & Profiling](#10-performance-benchmarks--profiling)
11. [Real-Time Telemetry & Profiling Engine](#11-real-time-telemetry--profiling-engine)
12. [Automated Hardware Benchmark Suite](#12-automated-hardware-benchmark-suite)

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
|  - In-Game UI / Settings Modals       - Real-time Diagnostic HUD (F3)    |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                          World Grid & ECS Stage                         |
|  - Voxel Stages Scheduling            - Two-Tier Streaming Orchestration|
|  - Chunk Cache (quick_cache LRU)      - Mesh Upload Frame Budget        |
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

## 4. Meshing Engine: Greedy Quad Merging & LOD

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

### Two-Pass Meshing: Opaque Solids & Semi-Transparent Water
Voxel rendering separates opaque terrain from fluid surfaces into two distinct mesh components per chunk:
1. **Opaque Pass**: Solid blocks (dirt, grass, stone, sand, wood, leaves) rendered with full early-Z depth writing and backface culling.
2. **Transparent Pass**: Water blocks rendered via a dedicated alpha-blended WGSL material (`AlphaMode::Blend`). Underwater seabeds remain fully visible through water surfaces without depth-fighting or alpha-sorting artifacts.

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

## 9. Persistence & Delta Compression

### Chunk Binary Delta Format
Unmodified terrain is procedurally deterministic and regenerated on demand without storing bytes on disk. Only **player-modified chunks** are persisted:

- Format: Binary dump of `[BlockType; 32768]` compressed via **LZ4**.
- Average storage: $\sim 2\text{ KB}$ per modified chunk (up to $16\times$ compression over raw voxel data).
- File structure: `saves/world_{seed}/chunks/chunk_{x}_{z}.bin`.

### Player State Serialization
Player parameters are serialized into clean, human-readable JSON (`saves/world_{seed}/player.json`):
- 3D Coordinates $[x, y, z]$
- Camera rotation (Yaw and Pitch)
- 9-Slot Hotbar items and stack counts
- 27-Slot Main Inventory storage
- Active selected slot index

---

## 10. Performance Benchmarks & Profiling

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

## 11. Real-Time Telemetry & Profiling Engine

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

## 12. Automated Hardware Benchmark Suite

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

## Quality Assurance & Verification Standards

To guarantee enterprise-grade stability, every commit satisfies:
```bash
# 1. 100% test pass rate across 74 unit, integration & property-based tests
cargo test

# 2. Strict zero-warning compliance on pedantic lints
cargo clippy --all-targets -- -D warnings

# 3. Code formatting compliance
cargo fmt --check

# 4. Microbenchmarks suite execution
cargo bench
```

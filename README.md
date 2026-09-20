# MineRust ⛏️🦀

A Minecraft clone written in Rust with [Bevy](https://bevyengine.org/), created for educational purposes, fun, and exploring modern 3D graphics, procedural generation, and high-performance concurrent programming in Rust.

---

## 🌟 Key Features

1. **Procedural World Generation with Deterministic Seed**:
   - Deterministic PRNG (`SplitMix64`) and Fisher-Yates shuffled 512-permutation table for 2D/3D Perlin noise, Fractal Brownian Motion (FBM), and Ridged Multi-Fractal noise.
   - Diverse biomes: **Plains**, **Forests**, **Deserts** with cacti, **Snowy Tundra** with pine trees, **Mountains** with snow caps, **Oceans**, **Beaches**, and winding **Rivers**.
   - **3D Underground Caves & Tunnels**: procedurally carved cavern networks and winding tunnels beneath the surface.
   - **Ore Veins**: realistic depth-stratified ore generation including **Coal**, **Iron**, **Gold**, and **Diamond**.
   - **Infinite Realtime Chunk Streaming**: view distance of 8 chunks ($17 \times 17$ loaded grid), background queue generation, and disk persistence for player-modified chunks.

2. **Pixel-Art Texture Atlas & PBR Rendering**:
   - Procedural 128x128 texture atlas (8x8 tiles of 16x16 pixels) with nearest-neighbor sampling for an authentic retro voxel aesthetic.
   - Dedicated textures for every block face (e.g., Grass top/side/bottom, Wood bark/rings, Sandstone, Ores, Glass, Ice, Water).
   - Alpha-tested transparency (`AlphaMode::Mask`) for clean glass and cutouts without depth sorting artifacts.
   - Directional face lighting and shadow mapping.

3. **Physics, Gravity & Player Movement**:
   - **AABB Collision Resolution**: realistic player bounding box ($0.6 \times 1.8 \times 0.6$ blocks) with eye level at 1.62m.
   - **Gravity & Inertia**: $-28\,\text{m/s}^2$ downward acceleration, terminal velocity, and smooth ground/air acceleration.
   - **Step Assist**: automatically walks up 1-block terrain steps without needing to jump constantly.
   - **Jumping & Sneaking**: jumping with `Space`, sneaking with `Shift` (includes edge protection to prevent falling off cliffs).
   - **Water Physics**: buoyant fluid dynamics, swimming upward (`Space`), diving (`Shift`), and jumping out onto shores.
   - **Flight Mode Toggle**: toggle between survival walking physics and creative no-clip flight at any time with `F`.
   - **Anti-Self-Trapping**: prevents placing solid blocks inside the player's own bounding box.

4. **Inventory & Quick Items (Hotbar HUD)**:
   - **9-Slot Hotbar HUD**: always visible at the bottom of the screen with active slot highlighting, numeric keys (`1`-`9`), and mouse wheel scrolling.
   - **Full Inventory Screen (`E`)**: accessible modal displaying all 20 blocks with localized names and preview badges; click any block to equip it to the active hotbar slot.
   - **Realtime HUD**: displays coordinates ($X, Y, Z$), current movement mode (Grounded, Airborne, Swimming, Flying), and control prompts.

---

## 🎮 Controls

| Key / Input | Action |
| :--- | :--- |
| **`W` `A` `S` `D`** | Horizontal movement with inertia and ground friction |
| **Mouse** | First-person camera look (Left click to capture / `ESC` to release) |
| **`Space`** | Jump (from ground) / Swim upward (in water) / Ascend (in flight mode) |
| **`Shift`** | Sneak (crouch with edge protection) / Dive (in water) / Descend (in flight mode) |
| **`Ctrl`** | Sprint (speed boost) |
| **`F`** | Toggle **Flight Mode** (No-Clip / Creative) |
| **Left Click** | Mine / break targeted block |
| **Right Click** | Place active block (protected against self-collision) |
| **`1` - `9` / Mouse Wheel** | Select active hotbar slot |
| **`E`** | Open / close full Inventory screen |
| **`ESC`** | Close inventory / unlock mouse cursor |

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (edition 2024 or latest stable)
- Standard development tools and graphics libraries for your OS (Vulkan, DX12, or Metal via WGPU/Bevy)

### Running the Game
```bash
# Run with default seed
cargo run

# Run with a custom numeric or string seed
cargo run -- --seed "my_minecraft_world"
cargo run -- -s 123456789

# Optimized release build (recommended for optimal voxel generation performance)
cargo run --release
```

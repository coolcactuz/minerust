# ADR 0002: In-Game Automated Hardware Benchmark & Telemetry Dashboard

- **Status**: Accepted
- **Date**: 2026-09-24
- **Author**: MineRust Core Engineering Team

---

## Context
As voxel engines stream vast procedural worlds, subjective framerate checks or static spawn inspections fail to reveal true performance characteristics. Critical bottlenecks—such as worker thread queue stalls, multithreaded chunk meshing latency, LRU cache thrashing, and GPU memory bandwidth saturation—only manifest during continuous high-speed movement through varied terrain.

Furthermore, players with diverse hardware configurations need an accessible, objective tool within the game itself to evaluate and optimize their graphical settings (render distance, dynamic shadows, atmospheric fog, greedy meshing, and distant sloped LOD).

## Decision
We implement a comprehensive, fully integrated **Automated Hardware Benchmark Suite** directly into MineRust (`src/benchmark.rs` and `src/menu/`):

1. **In-Game Accessibility**:
   - Integrated directly into the Graphics Settings menu via a prominent action button: `⚡ Run Hardware Benchmark`.
   - Players can test their configuration without requiring command-line flags or developer tools.

2. **Standardized Flight Trajectory**:
   - Reinitializes the world with a deterministic fixed seed: `"BENCHMARK"`.
   - Moves the camera forward at a continuous high speed of **$50\text{ m/s}$** ($180\text{ km/h}$) across a standardized **$5,000\text{m}$ (5km)** trajectory over diverse terrain (mountains, oceans, forests, plateaus).

3. **User Settings Preservation with Unconstrained Throughput**:
   - Preserves all active user graphics settings (Render Distance, Shadows, Distance Fog, Greedy Meshing, Sloped LOD) so players test their exact preferred configuration.
   - Automatically bypasses internal frame rate caps (`FpsLimiter`) during the benchmark to measure raw unconstrained hardware throughput.

4. **Multi-Phase Statistical Telemetry**:
   - **Phase 1 (Static Baseline)**: Measures 1.5s of stationary rendering at spawn with 100% pregenerated chunks (0 background streaming load).
   - **Phase 2 (Dynamic Streaming Flight)**: Collects frame times, peak physical RAM (`VmRSS` via `/proc/self/status`), GPU VRAM (via Linux DRM/KMS `fdinfo` query with analytical fallback), active chunks, and peak vertex/triangle geometry.

5. **Post-Benchmark Results Dashboard (`MenuScreen::BenchmarkResults`)**:
   - Categorized into Framerate & Pacing, Memory & Hardware, and Evaluated Settings.
   - Generates automated, rule-based hardware verdicts (*PERFECT*, *GREAT*, *PLAYABLE*, *SUB-OPTIMAL*) with personalized tuning recommendations (e.g. setting distant sloped LOD to > 4–8 chunks, lowering shadows, or adjusting render distance).

6. **Clean Cancellation & Safety**:
   - Pressing `ESC` at any moment immediately halts the benchmark and safely restores cursor controls and menu state.

## Consequences

### Positive
- **Empowered Players**: Gamers can objectively identify the sweet spot between visual fidelity and stable 60+ FPS for their hardware.
- **Architectural Verification**: Developers gain an automated, repeatable stress test to benchmark streaming and meshing refactors.
- **100% Safe Rust**: Memory telemetry and DRM queries are strictly implemented without unsafe blocks or external C libraries.

### Negative / Trade-offs
- A complete 5km flight at 50 m/s requires approximately 100 seconds to execute (mitigated by instant `ESC` cancellation).

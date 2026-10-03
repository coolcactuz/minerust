#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::time::Instant;

use bevy::math::{IVec2, Vec3};
use minerust::chunk::Chunk;
use minerust::mesher::{CHUNK_SECTIONS, SectionFace, build_chunk_mesh_lod};
use minerust::noise::NoiseGenerator;
use minerust::world::macro_lod::{MACRO_CHUNK_SIZE, merge_chunk_meshes};
use minerust::world::occlusion::{compute_section_occlusion_with_queue, is_under_open_sky};
use minerust::world::streaming::{
    calculate_lod_scan_radius, chunk_distance_sq_to_player, determine_chunk_tier,
};
use minerust::world::{
    WorldGrid, WorldSeed, generate_chunk, get_entered_chunks, get_exited_chunks,
};

/// Command-line configuration for the headless CPU benchmark.
#[derive(Debug, Clone)]
struct BenchCpuOptions {
    view_distance: i32,
    flight_distance: f32,
    flight_speed: f32,
    simulated_fps: f32,
    seed: WorldSeed,
}

impl Default for BenchCpuOptions {
    fn default() -> Self {
        Self {
            view_distance: 32, // Default to 32 chunks for fast, representative execution (<5s)
            flight_distance: 1000.0,
            flight_speed: 50.0,
            simulated_fps: 700.0,
            seed: WorldSeed::from_seed_str("BENCHMARK"),
        }
    }
}

impl BenchCpuOptions {
    fn parse_from_args<I: IntoIterator<Item = String>>(args: I) -> Self {
        let mut opts = Self::default();
        let args: Vec<String> = args.into_iter().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--quick" => {
                    opts.view_distance = 16;
                    opts.flight_distance = 300.0;
                }
                "--standard" => {
                    opts.view_distance = 32;
                    opts.flight_distance = 1000.0;
                }
                "--full" => {
                    opts.view_distance = 64;
                    opts.flight_distance = 1000.0;
                }
                "-v" | "--view-distance" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(v) = args[i].parse::<i32>() {
                            opts.view_distance = v.clamp(4, 64);
                        }
                    }
                }
                "-d" | "--distance" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(d) = args[i].parse::<f32>() {
                            opts.flight_distance = d.max(50.0);
                        }
                    }
                }
                "-s" | "--speed" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(s) = args[i].parse::<f32>() {
                            opts.flight_speed = s.max(1.0);
                        }
                    }
                }
                "--fps" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(fps) = args[i].parse::<f32>() {
                            opts.simulated_fps = fps.clamp(30.0, 5000.0);
                        }
                    }
                }
                "-h" | "--help" => {
                    println!(
                        "MineRust Headless CPU Benchmark\n\
                         Usage: cargo run --release --bin bench_cpu -- [OPTIONS]\n\n\
                         Options:\n  \
                           --quick                  Run fast benchmark (16 chunks, 300m)\n  \
                           --standard               Run standard benchmark (32 chunks, 1000m, default)\n  \
                           --full                   Run heavy benchmark (64 chunks, 1000m)\n  \
                           -v, --view-distance <N>  Render distance in chunks (4 to 64)\n  \
                           -d, --distance <M>       Flight distance in meters (default 1000m)\n  \
                           -s, --speed <M/S>        Flight speed in meters/second (default 50.0)\n  \
                           --fps <FPS>              Target simulated tick rate (default 700.0)\n  \
                           -h, --help               Print help message"
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
            i += 1;
        }
        opts
    }
}

/// Statistics calculated from a series of timing samples in microseconds.
#[derive(Debug, Clone, Default)]
struct TimingStats {
    count: usize,
    mean_us: f64,
    median_us: f64,
    p95_us: f64,
    p99_us: f64,
    min_us: f64,
    max_us: f64,
}

impl TimingStats {
    fn compute(mut samples: Vec<f64>) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        samples.sort_by(|a, b| a.total_cmp(b));
        let count = samples.len();
        let sum: f64 = samples.iter().sum();
        let mean_us = sum / count as f64;
        let min_us = samples[0];
        let max_us = samples[count - 1];

        let median_us = samples[count / 2];
        let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
        let p99_idx = ((count as f64 * 0.99) as usize).min(count - 1);
        let p95_us = samples[p95_idx];
        let p99_us = samples[p99_idx];

        Self {
            count,
            mean_us,
            median_us,
            p95_us,
            p99_us,
            min_us,
            max_us,
        }
    }
}

fn bench_generation(noise: &NoiseGenerator, seed: u64) -> (f64, f64) {
    let num_chunks = 48;
    let t0 = Instant::now();
    for i in 0..num_chunks {
        let cx = (i % 8) - 4;
        let cz = (i / 8) - 3;
        let _ = generate_chunk(cx, cz, noise, seed);
    }
    let elapsed_sec = t0.elapsed().as_secs_f64();
    let per_chunk_us = (elapsed_sec * 1_000_000.0) / num_chunks as f64;
    let chunks_per_sec = num_chunks as f64 / elapsed_sec;
    (per_chunk_us, chunks_per_sec)
}

fn bench_mesher_tiers(noise: &NoiseGenerator, seed: u64) -> [(f64, f64, usize); 3] {
    let sample_chunks: Vec<Chunk> = (-1..=1)
        .flat_map(|x| (-1..=1).map(move |z| generate_chunk(x, z, noise, seed)))
        .collect();

    let center = &sample_chunks[4];
    let north = Some(&sample_chunks[5]);
    let south = Some(&sample_chunks[3]);
    let east = Some(&sample_chunks[7]);
    let west = Some(&sample_chunks[1]);

    let tiers = [
        ("Tier 0 (Standard 1x1)", false, 0),
        ("Tier 1 (Greedy Voxel)", true, 0),
        ("Tier 2 (Sloped LOD)", false, 1),
    ];

    let mut results = [(0.0, 0.0, 0usize); 3];

    for (idx, (_, greedy, lod)) in tiers.iter().enumerate() {
        let iterations = if *lod >= 1 { 200 } else { 100 };
        let t0 = Instant::now();
        let mut total_verts = 0;
        for _ in 0..iterations {
            let mesh = build_chunk_mesh_lod(center, north, south, east, west, true, *greedy, *lod);
            total_verts = mesh.total_vertices();
        }
        let elapsed_sec = t0.elapsed().as_secs_f64();
        let per_chunk_us = (elapsed_sec * 1_000_000.0) / iterations as f64;
        let chunks_per_sec = iterations as f64 / elapsed_sec;
        results[idx] = (per_chunk_us, chunks_per_sec, total_verts);
    }

    results
}

fn bench_macro_lod_merge(noise: &NoiseGenerator, seed: u64) -> (f64, f64) {
    let mut chunk_meshes = Vec::new();
    for slot in 0..(MACRO_CHUNK_SIZE * MACRO_CHUNK_SIZE) as usize {
        let lx = (slot % MACRO_CHUNK_SIZE as usize) as i32;
        let lz = (slot / MACRO_CHUNK_SIZE as usize) as i32;
        let c = generate_chunk(lx, lz, noise, seed);
        let m = build_chunk_mesh_lod(&c, None, None, None, None, true, false, 1);
        if let Some(solid) = m.sections[0].solid.clone() {
            chunk_meshes.push((slot, solid));
        }
    }

    let iterations = 100;
    let t0 = Instant::now();
    for _ in 0..iterations {
        let iter = chunk_meshes.iter().map(|(s, m)| (*s, m));
        let merged = merge_chunk_meshes(iter);
        std::hint::black_box(merged);
    }
    let elapsed_sec = t0.elapsed().as_secs_f64();
    let per_cluster_us = (elapsed_sec * 1_000_000.0) / iterations as f64;
    let clusters_per_sec = iterations as f64 / elapsed_sec;
    (per_cluster_us, clusters_per_sec)
}

fn bench_ecs_mesh_injection(noise: &NoiseGenerator, seed: u64) -> (f64, f64) {
    let mut ecs_world = bevy::ecs::world::World::new();
    let mut asset_meshes = bevy::asset::Assets::<bevy::prelude::Mesh>::default();

    let chunk = generate_chunk(0, 0, noise, seed);

    let count = 100;
    let mut prebuilt = Vec::with_capacity(count);
    for _ in 0..count {
        prebuilt.push(build_chunk_mesh_lod(
            &chunk, None, None, None, None, true, true, 0,
        ));
    }

    let aabb =
        bevy::camera::primitives::Aabb::from_min_max(Vec3::ZERO, Vec3::new(16.0, 16.0, 16.0));
    let t0 = Instant::now();
    for (i, chunk_mesh) in prebuilt.into_iter().enumerate() {
        let pos = Vec3::new((i as f32) * 16.0, 0.0, 0.0);
        for sec in chunk_mesh.sections {
            if let Some(solid_mesh) = sec.solid {
                let handle = asset_meshes.add(solid_mesh);
                ecs_world.spawn((
                    bevy::prelude::Mesh3d(handle),
                    bevy::prelude::Transform::from_translation(pos),
                    aabb,
                    bevy::camera::visibility::NoAutoAabb,
                ));
            }
        }
    }
    let elapsed_sec = t0.elapsed().as_secs_f64();
    let per_chunk_us = (elapsed_sec * 1_000_000.0) / count as f64;
    let chunks_per_sec = count as f64 / elapsed_sec;
    (per_chunk_us, chunks_per_sec)
}

fn run_flight_simulation(
    opts: &BenchCpuOptions,
    world: &mut WorldGrid,
) -> (
    TimingStats,
    TimingStats,
    TimingStats,
    TimingStats,
    TimingStats,
) {
    let dt = 1.0 / opts.simulated_fps;
    let total_steps = ((opts.flight_distance / (opts.flight_speed * dt)).ceil() as usize).max(100);

    let view_dist = opts.view_distance;
    let pregen_margin = 2;
    let gen_dist = view_dist + pregen_margin;
    let lod_threshold_sq = (8.0 * 16.0_f32).powi(2);
    let greedy_threshold_sq = (2.0 * 16.0_f32).powi(2);

    let mut cam_pos = Vec3::new(0.0, 90.0, 0.0);
    let mut last_player_chunk = IVec2::new(i32::MAX, i32::MAX);
    let mut last_lod_pos = Vec3::new(-9999.0, -9999.0, -9999.0);

    let mut occlusion_cache_chunk = IVec2::new(i32::MAX, i32::MAX);
    let mut occlusion_cache_sy = i32::MAX;
    let mut occlusion_cache_outdoors = false;
    let mut occlusion_queue: VecDeque<(IVec2, u8, Option<SectionFace>)> =
        VecDeque::with_capacity(2048);

    let mut total_tick_samples = Vec::with_capacity(total_steps);
    let mut lod_samples = Vec::with_capacity(total_steps);
    let mut occlusion_samples = Vec::with_capacity(total_steps);
    let mut frontier_samples = Vec::with_capacity(total_steps);
    let mut sort_samples = Vec::with_capacity(total_steps);

    for step in 0..total_steps {
        let t_tick_start = Instant::now();

        // 1. Advance camera translation along flight trajectory
        cam_pos.z += opts.flight_speed * dt;
        let px = cam_pos.x.floor() as i32;
        let pz = cam_pos.z.floor() as i32;
        let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(px, pz);

        // 2. Measure Frontier chunk enter/exit calculation
        let t_frontier_start = Instant::now();
        if player_chunk != last_player_chunk {
            let prev = last_player_chunk;
            last_player_chunk = player_chunk;

            if prev.x != i32::MAX {
                let entered = get_entered_chunks(prev, player_chunk, gen_dist);
                let exited = get_exited_chunks(prev, player_chunk, gen_dist);
                let mut newly_needed = Vec::with_capacity(entered.len());
                for coord in entered {
                    if !world.chunks.contains_key(&coord) {
                        newly_needed.push(coord);
                    }
                }
                newly_needed.sort_unstable_by_key(|c| {
                    let diff = *c - player_chunk;
                    -(diff.x * diff.x + diff.y * diff.y)
                });
                world.generation_queue.extend(newly_needed);

                for coord in exited {
                    world.chunks.remove(&coord);
                    world.chunk_lod.remove(&coord);
                    world.chunk_connectivity.remove(&coord);
                }
            }
        }
        let frontier_us = t_frontier_start.elapsed().as_secs_f64() * 1_000_000.0;
        frontier_samples.push(frontier_us);

        // 3. Measure LOD Transition Scanning
        let t_lod_start = Instant::now();
        let should_check_lod = cam_pos.distance_squared(last_lod_pos) >= 4.0;
        if should_check_lod {
            last_lod_pos = cam_pos;
            let scan_radius = calculate_lod_scan_radius(
                true,
                lod_threshold_sq,
                true,
                greedy_threshold_sq,
                view_dist,
            );
            let mut chunks_needing_lod = Vec::new();
            for dz in -scan_radius..=scan_radius {
                for dx in -scan_radius..=scan_radius {
                    let coord = player_chunk + IVec2::new(dx, dz);
                    if let Some(chunk) = world.chunks.get(&coord) {
                        if let Some(&current_tier) = world.chunk_lod.get(&coord) {
                            let dist_sq = chunk_distance_sq_to_player(coord, cam_pos, Some(chunk));
                            let (target_tier, _, _) = determine_chunk_tier(
                                dist_sq,
                                true,
                                lod_threshold_sq,
                                true,
                                2,
                                greedy_threshold_sq,
                            );
                            if current_tier != target_tier {
                                chunks_needing_lod.push(coord);
                            }
                        }
                    }
                }
            }
            world.mesh_queue.extend(chunks_needing_lod);
            world.mesh_queue_dirty = true;
        }
        let lod_us = t_lod_start.elapsed().as_secs_f64() * 1_000_000.0;
        lod_samples.push(lod_us);

        // 4. Measure Mesh Queue Sorting
        let t_sort_start = Instant::now();
        if world.mesh_queue_dirty && world.mesh_queue.len() > 1 {
            let px = player_chunk.x;
            let pz = player_chunk.y;
            world.mesh_queue.sort_unstable_by_key(|c| {
                let dx = c.x - px;
                let dz = c.y - pz;
                -(dx * dx + dz * dz)
            });
            world.mesh_queue_dirty = false;
        }
        let sort_us = t_sort_start.elapsed().as_secs_f64() * 1_000_000.0;
        sort_samples.push(sort_us);

        // 5. Measure Software Cave Occlusion BFS
        let t_occ_start = Instant::now();
        let cam_sy = (cam_pos.y / 16.0).floor() as i32;
        let outdoors = cam_sy >= CHUNK_SECTIONS as i32 || is_under_open_sky(world, cam_pos);

        if player_chunk != occlusion_cache_chunk
            || cam_sy != occlusion_cache_sy
            || outdoors != occlusion_cache_outdoors
        {
            occlusion_cache_chunk = player_chunk;
            occlusion_cache_sy = cam_sy;
            occlusion_cache_outdoors = outdoors;
            let _bitset =
                compute_section_occlusion_with_queue(world, cam_pos, &mut occlusion_queue);
        }
        let occ_us = t_occ_start.elapsed().as_secs_f64() * 1_000_000.0;
        occlusion_samples.push(occ_us);

        let tick_us = t_tick_start.elapsed().as_secs_f64() * 1_000_000.0;
        total_tick_samples.push(tick_us);

        // Keep queue from unbounded growth in headless mock
        if step % 50 == 0 {
            world.mesh_queue.truncate(64);
        }
    }

    (
        TimingStats::compute(total_tick_samples),
        TimingStats::compute(lod_samples),
        TimingStats::compute(occlusion_samples),
        TimingStats::compute(frontier_samples),
        TimingStats::compute(sort_samples),
    )
}

fn main() {
    let opts = BenchCpuOptions::parse_from_args(std::env::args());

    println!("\n================================================================================");
    println!("             MINERUST HEADLESS CPU BENCHMARK & PROFILER TOOL                    ");
    println!("================================================================================");
    println!("Configuration:");
    println!("  - World Seed       : {:?}", opts.seed);
    println!(
        "  - Render Distance  : {} chunks (active world grid radius)",
        opts.view_distance
    );
    println!(
        "  - Flight Distance  : {:.0} meters @ {:.1} m/s",
        opts.flight_distance, opts.flight_speed
    );
    println!(
        "  - Simulated Target : {:.0} FPS ({:.2} ms frame budget)",
        opts.simulated_fps,
        1000.0 / opts.simulated_fps
    );
    println!("--------------------------------------------------------------------------------");

    let noise = NoiseGenerator::new(opts.seed.0);

    // Micro-benchmarks
    print!("Benchmarking Procedural World Generation... ");
    let (gen_per_chunk_us, gen_chunks_sec) = bench_generation(&noise, opts.seed.0);
    println!("Done.");

    print!("Benchmarking Chunk Geometry Meshers (Tier 0, 1, 2)... ");
    let mesher_results = bench_mesher_tiers(&noise, opts.seed.0);
    println!("Done.");

    print!("Benchmarking Macro-Chunk LOD Mesh Merging... ");
    let (macro_merge_us, macro_clusters_sec) = bench_macro_lod_merge(&noise, opts.seed.0);
    println!("Done.");

    print!("Benchmarking Bevy ECS Chunk Mesh Injection (apply_chunk_mesh)... ");
    let (ecs_apply_us, ecs_applies_sec) = bench_ecs_mesh_injection(&noise, opts.seed.0);
    println!("Done.");

    println!("\n[ MICRO-BENCHMARK RESULTS ]");
    println!(
        "  * Procedural Gen (Strata+Caves+Foliage) : {:>7.1} µs/chunk | {:>7.1} chunks/sec",
        gen_per_chunk_us, gen_chunks_sec
    );
    println!(
        "  * Tier 0 Meshing (Standard 1x1 Voxels)  : {:>7.1} µs/chunk | {:>7.1} chunks/sec | {:>5} verts",
        mesher_results[0].0, mesher_results[0].1, mesher_results[0].2
    );
    println!(
        "  * Tier 1 Meshing (Greedy Voxel Merging) : {:>7.1} µs/chunk | {:>7.1} chunks/sec | {:>5} verts",
        mesher_results[1].0, mesher_results[1].1, mesher_results[1].2
    );
    println!(
        "  * Tier 2 Meshing (Sloped Heightfield)   : {:>7.1} µs/chunk | {:>7.1} chunks/sec | {:>5} verts",
        mesher_results[2].0, mesher_results[2].1, mesher_results[2].2
    );
    println!(
        "  * Macro-Chunk Merge (16 chunk cluster)  : {:>7.1} µs/merge | {:>7.1} merges/sec",
        macro_merge_us, macro_clusters_sec
    );
    println!(
        "  * Bevy ECS Asset & Entity Injection     : {:>7.1} µs/chunk | {:>7.1} chunks/sec",
        ecs_apply_us, ecs_applies_sec
    );

    // Setup active world for trajectory flight simulation
    println!("\nInitializing World Grid for Flight Simulation (loading initial chunks)...");
    let mut world = WorldGrid::new(opts.seed);
    let init_radius = opts.view_distance.min(32);
    for dz in -init_radius..=init_radius {
        for dx in -init_radius..=init_radius {
            let coord = IVec2::new(dx, dz);
            let chunk = generate_chunk(dx, dz, &noise, opts.seed.0);
            let dist_sq =
                chunk_distance_sq_to_player(coord, Vec3::new(0.0, 90.0, 0.0), Some(&chunk));
            let (tier, _, _) = determine_chunk_tier(dist_sq, true, 16384.0, true, 2, 1024.0);
            world.chunk_lod.insert(coord, tier);

            // Populate connectivity for occlusion culling
            let north = None;
            let south = None;
            let east = None;
            let west = None;
            let mesh = build_chunk_mesh_lod(&chunk, north, south, east, west, true, false, tier);
            world.chunk_connectivity.insert(coord, mesh.connectivity);
            world.chunks.insert(coord, chunk);
        }
    }
    println!("Active chunks populated in memory: {}", world.chunks.len());

    println!(
        "Simulating {:.0}m Flight Trajectory...",
        opts.flight_distance
    );
    let (total_stats, lod_stats, occ_stats, front_stats, sort_stats) =
        run_flight_simulation(&opts, &mut world);

    let theoretical_fps = if total_stats.mean_us > 0.0 {
        1_000_000.0 / total_stats.mean_us
    } else {
        0.0
    };
    let target_frame_budget_us: f64 = (1000.0 / f64::from(opts.simulated_fps)) * 1000.0;
    let cpu_load_pct = (total_stats.mean_us / target_frame_budget_us) * 100.0;

    println!("\n================================================================================");
    println!(
        "             FLIGHT STREAMING CPU TIMINGS ({} frames simulated)                ",
        total_stats.count
    );
    println!("================================================================================");
    println!(
        "  Metric                Mean       Median (P50)     P95          P99       Min / Max    "
    );
    println!(
        "  --------------------------------------------------------------------------------------------"
    );
    println!(
        "  Total Frame Overhead : {:>6.1} µs   {:>6.1} µs     {:>6.1} µs   {:>6.1} µs   {:>5.1} / {:>6.1} µs",
        total_stats.mean_us,
        total_stats.median_us,
        total_stats.p95_us,
        total_stats.p99_us,
        total_stats.min_us,
        total_stats.max_us
    );
    println!(
        "  - LOD Transition Scan: {:>6.1} µs   {:>6.1} µs     {:>6.1} µs   {:>6.1} µs   {:>5.1} / {:>6.1} µs",
        lod_stats.mean_us,
        lod_stats.median_us,
        lod_stats.p95_us,
        lod_stats.p99_us,
        lod_stats.min_us,
        lod_stats.max_us
    );
    println!(
        "  - Cave Occlusion BFS : {:>6.1} µs   {:>6.1} µs     {:>6.1} µs   {:>6.1} µs   {:>5.1} / {:>6.1} µs",
        occ_stats.mean_us,
        occ_stats.median_us,
        occ_stats.p95_us,
        occ_stats.p99_us,
        occ_stats.min_us,
        occ_stats.max_us
    );
    println!(
        "  - Frontier Enter/Exit: {:>6.1} µs   {:>6.1} µs     {:>6.1} µs   {:>6.1} µs   {:>5.1} / {:>6.1} µs",
        front_stats.mean_us,
        front_stats.median_us,
        front_stats.p95_us,
        front_stats.p99_us,
        front_stats.min_us,
        front_stats.max_us
    );
    println!(
        "  - Mesh Queue Sorting : {:>6.1} µs   {:>6.1} µs     {:>6.1} µs   {:>6.1} µs   {:>5.1} / {:>6.1} µs",
        sort_stats.mean_us,
        sort_stats.median_us,
        sort_stats.p95_us,
        sort_stats.p99_us,
        sort_stats.min_us,
        sort_stats.max_us
    );
    println!("================================================================================");
    println!("THEORETICAL CPU CEILING & BUDGET:");
    println!(
        "  * Mean CPU Frame Time         : {:.1} µs ({:.3} ms)",
        total_stats.mean_us,
        total_stats.mean_us / 1000.0
    );
    println!(
        "  * Theoretical Max CPU Ceiling : \x1b[1;32m{:.0} FPS\x1b[0m",
        theoretical_fps
    );
    println!(
        "  * Frame Budget Consumed by CPU: {:.1}% (at {:.0} FPS target)",
        cpu_load_pct, opts.simulated_fps
    );
    println!(
        "  * Available Budget for GPU    : {:.1} µs ({:.1}%)",
        target_frame_budget_us - total_stats.mean_us,
        100.0 - cpu_load_pct
    );
    println!("================================================================================\n");
}

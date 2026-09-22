use bevy::prelude::*;

use super::*;
use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::coords::LocalBlockPos;
use crate::menu::GraphicsSettings;
use crate::noise::NoiseGenerator;

#[test]
#[allow(clippy::float_cmp)]
fn test_chunk_3d_distance_and_vertical_flight() {
    let coord = IVec2::new(0, 0); // bounds: [0..16, 0..16]
    let mut chunk = Chunk::new();
    chunk.max_y = 70; // terrain reaches up to y=70

    // 1. Standing directly on the terrain
    let player_ground = Vec3::new(8.0, 71.0, 8.0);
    let d_sq_ground = chunk_distance_sq_to_player(coord, player_ground, Some(&chunk));
    assert_eq!(d_sq_ground, 0.0);

    // 2. Flying high vertically in creative mode (e.g. y = 200)
    let player_high = Vec3::new(8.0, 200.0, 8.0);
    let d_sq_high = chunk_distance_sq_to_player(coord, player_high, Some(&chunk));
    let expected_dy = 200.0 - 71.0; // 129.0
    assert_eq!(d_sq_high, expected_dy * expected_dy);

    // Threshold of 4 chunks = 64 meters (threshold_sq = 4096)
    let threshold_sq = (4.0 * 16.0_f32).powi(2); // 4096.0
    assert!(
        d_sq_ground <= threshold_sq,
        "Ground chunk should be detailed (LOD 0)"
    );
    assert!(
        d_sq_high > threshold_sq,
        "Chunk viewed from high altitude should compress to Greedy Mesh (LOD 1)"
    );
}

#[test]
fn test_lru_chunk_cache_insertion_and_retrieval() {
    let world = WorldGrid::default();
    let coord = IVec2::new(42, -99);
    let mut chunk = Chunk::new();
    chunk.set_local(LocalBlockPos::new(1, 2, 3), BlockType::DiamondOre);

    assert!(world.chunk_cache.get(&coord).is_none());

    world.chunk_cache.insert(coord, chunk);

    let retrieved = world
        .chunk_cache
        .get(&coord)
        .expect("chunk should be in cache");
    assert_eq!(
        retrieved.get_local(LocalBlockPos::new(1, 2, 3)),
        BlockType::DiamondOre
    );
}

#[test]
fn test_fused_chunk_generation_strata_and_bedrock() {
    let noise = NoiseGenerator::new(42);
    let chunk = generate_chunk(0, 0, &noise, 42);

    // Bedrock is guaranteed at layer 0 across all columns
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            assert_eq!(chunk.get_fast(lx, 0, lz), BlockType::Bedrock);
        }
    }

    // Chunk max_y must reflect populated terrain height
    assert!(chunk.max_y >= 5, "Terrain height must be at least 5");
    assert!(
        chunk.max_y < CHUNK_HEIGHT,
        "Terrain height must fit within CHUNK_HEIGHT"
    );
}

#[test]
fn test_lookahead_buffer_tier_radii() {
    let view_dist = 16;
    let pregen_margin = 2;
    let gen_dist = view_dist + pregen_margin; // 18
    let unload_dist = gen_dist + 2; // 20

    // Visual Tier 1: radius 16 chunks
    let tier1_chunk = IVec2::new(16, 0);
    assert!(tier1_chunk.x.abs() <= view_dist && tier1_chunk.y.abs() <= view_dist);

    // Lookahead Tier 2 (pre-generated RAM buffer, unmeshed): radius 17..18 chunks
    let tier2_chunk = IVec2::new(17, 1);
    let diff_2d = tier2_chunk.x.abs().max(tier2_chunk.y.abs());
    assert!(
        diff_2d > view_dist,
        "Tier 2 chunk must be outside visual view distance"
    );
    assert!(
        diff_2d <= gen_dist,
        "Tier 2 chunk must be inside lookahead generation radius"
    );
    assert!(diff_2d <= unload_dist, "Tier 2 chunk must not be unloaded");

    // Far chunk: radius 21 chunks
    let far_chunk = IVec2::new(21, 0);
    let far_diff = far_chunk.x.abs().max(far_chunk.y.abs());
    assert!(
        far_diff > unload_dist,
        "Far chunk must exceed unload threshold"
    );
}

#[test]
fn test_world_seed_random_and_alphanumeric() {
    let seed1 = WorldSeed::random();
    let seed2 = WorldSeed::random();

    // Valid positive numbers
    assert!(seed1.0 > 0);
    assert!(seed2.0 > 0);

    // Numeric string parsing
    let num_seed = WorldSeed::from_seed_str("987654321");
    assert_eq!(num_seed.0, 987654321);

    // Alphanumeric string parsing (deterministic)
    let alpha1 = WorldSeed::from_seed_str("minecraft");
    let alpha2 = WorldSeed::from_seed_str("minecraft");
    let alpha3 = WorldSeed::from_seed_str("custom_seed_42");

    assert_eq!(alpha1, alpha2);
    assert_ne!(alpha1, alpha3);
    assert_ne!(alpha1.0, 0);
}

#[test]
fn test_assets_mesh_lifecycle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.update();

    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    let handle = meshes.add(Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    ));
    let id = handle.id();
    assert!(meshes.contains(id));
    assert_eq!(meshes.len(), 1);

    // Spawn entity with Mesh3d(handle)
    let entity = app.world_mut().spawn(Mesh3d(handle)).id();
    app.update();

    // Overwrite entity's Mesh3d with new handle
    let handle2 = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        ));
    let id2 = handle2.id();
    app.world_mut().entity_mut(entity).insert(Mesh3d(handle2));
    for _ in 0..10 {
        app.update();
    }

    let meshes = app.world().resource::<Assets<Mesh>>();
    assert!(
        !meshes.contains(id),
        "Overwritten mesh must be removed from Assets<Mesh>"
    );
    assert!(
        meshes.contains(id2),
        "New mesh must be present in Assets<Mesh>"
    );
    assert_eq!(meshes.len(), 1);

    // Despawn entity
    app.world_mut().entity_mut(entity).despawn();
    for _ in 0..10 {
        app.update();
    }

    let meshes = app.world().resource::<Assets<Mesh>>();
    assert!(
        !meshes.contains(id2),
        "Despawned mesh must be removed from Assets<Mesh>"
    );
    assert_eq!(
        meshes.len(),
        0,
        "Assets<Mesh> must be completely empty after despawn"
    );
}

#[test]
fn test_zombie_mesh_prevention_and_streaming_cleanup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<crate::voxel_material::VoxelBlockMaterial>();

    let mut world_grid = WorldGrid::default();
    // Insert a distant chunk that is in Tier 2 (outside view_dist = 2, inside unload_dist)
    let distant_coord = IVec2::new(5, 5);
    world_grid.chunks.insert(distant_coord, Chunk::new());
    app.insert_resource(world_grid);

    app.insert_resource(ChunkGeneratorPool::default());
    let mesher_pool = ChunkMesherPool::default();
    let tx = mesher_pool.tx.clone();
    app.insert_resource(mesher_pool);

    app.insert_resource(GraphicsSettings {
        view_distance: 2,
        ..default()
    });

    // Spawn camera at (0, 0)
    app.world_mut().spawn((
        FpsCamera::default(),
        Transform::from_xyz(0.0, 50.0, 0.0),
        Projection::Perspective(PerspectiveProjection::default()),
    ));

    app.add_systems(Update, world_streaming_system);
    app.update();

    // Simulate a background mesher thread completing a mesh for distant_coord (5, 5)
    // which is outside the view_distance of 2
    let dummy_mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    tx.send((distant_coord, Some(dummy_mesh), 0)).unwrap();

    // Run streaming update
    app.update();

    let world = app.world().resource::<WorldGrid>();
    // Distant chunk must NOT have an entity spawned or mesh active!
    assert!(
        !world.chunk_entities.contains_key(&distant_coord),
        "Distant chunk outside view_dist must not spawn zombie entity"
    );
    assert_eq!(
        world
            .chunk_vertices
            .get(&distant_coord)
            .copied()
            .unwrap_or(0),
        0
    );

    // Assets<Mesh> must not leak the discarded mesh
    let meshes = app.world().resource::<Assets<Mesh>>();
    assert_eq!(
        meshes.len(),
        0,
        "Discarded mesh must not be added to Assets<Mesh>"
    );
}

#[test]
fn test_smoothstep_and_lerp_properties() {
    assert!((smoothstep(0.0, 1.0, -0.5) - 0.0).abs() < 1e-6);
    assert!((smoothstep(0.0, 1.0, 1.5) - 1.0).abs() < 1e-6);
    assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    assert!((smoothstep(5.0, 5.0, 5.0) - 0.0).abs() < 1e-6);

    assert!((lerp(10.0, 20.0, 0.0) - 10.0).abs() < 1e-6);
    assert!((lerp(10.0, 20.0, 0.5) - 15.0).abs() < 1e-6);
    assert!((lerp(10.0, 20.0, 1.0) - 20.0).abs() < 1e-6);
}

#[test]
fn test_terrain_slope_continuity_and_no_sheer_walls() {
    let noise = NoiseGenerator::new(12345);
    let mut max_single_step_slope = 0i32;
    let mut min_height = i32::MAX;
    let mut max_height = i32::MIN;

    // Traverse a wide horizontal cross section of 2000 blocks across biomes
    let mut prev_h = calculate_biome_and_height(0.0, 0.0, &noise).1;
    for x in 1..2000 {
        let (_, h, _) = calculate_biome_and_height(x as f64, 0.0, &noise);
        let diff = (h - prev_h).abs();
        if diff > max_single_step_slope {
            max_single_step_slope = diff;
        }
        min_height = min_height.min(h);
        max_height = max_height.max(h);
        prev_h = h;
    }

    // Also test along Z axis across 2000 blocks
    prev_h = calculate_biome_and_height(0.0, 0.0, &noise).1;
    for z in 1..2000 {
        let (_, h, _) = calculate_biome_and_height(0.0, z as f64, &noise);
        let diff = (h - prev_h).abs();
        if diff > max_single_step_slope {
            max_single_step_slope = diff;
        }
        min_height = min_height.min(h);
        max_height = max_height.max(h);
        prev_h = h;
    }

    // No unnatural 20+ block vertical cliff faces: single step horizontal slope must be gentle (<= 3 blocks)
    assert!(
        max_single_step_slope <= 3,
        "Terrain has unnatural sheer cliff wall: max single step was {}",
        max_single_step_slope
    );

    // Verify terrain reaches oceans and high peaks
    assert!(
        min_height <= 50,
        "Terrain should include deep water/ocean basins, got min {}",
        min_height
    );
    assert!(
        max_height >= 95,
        "Terrain should include tall mountain peaks, got max {}",
        max_height
    );
}

#[test]
fn test_find_safe_surface_spawn_is_on_dry_land_and_surface() {
    // Test multiple arbitrary seeds
    for seed in [0, 42, 133742, 987654321, 0xcbf29ce484222325] {
        let noise = NoiseGenerator::new(seed);
        let spawn = find_safe_surface_spawn(&noise, seed);

        let wx = spawn.x.floor() as i32;
        let wz = spawn.z.floor() as i32;
        let (biome, h, is_river) = calculate_biome_and_height(wx as f64, wz as f64, &noise);

        // 1. Surface check: spawn Y must be above surface height h
        assert!(spawn.y > h as f32, "Spawn Y must be above ground level h");
        assert!(
            (spawn.y - (h as f32 + 2.8)).abs() < 1e-4,
            "Spawn Y must match eye level above surface"
        );

        // 2. Dry land check: strictly above sea level (SEA_LEVEL = 64)
        assert!(
            h > SEA_LEVEL,
            "Spawn must be strictly above sea level 64, got height {}",
            h
        );

        // 3. Not in water/river
        assert!(!is_river, "Spawn must not be in a river bed");

        // 4. Biome is dry land, not ocean
        assert_ne!(biome, BiomeType::Ocean, "Spawn must not be in ocean");
        assert_ne!(
            biome,
            BiomeType::FrozenOcean,
            "Spawn must not be in frozen ocean"
        );

        // 5. Neighbors are also dry land above sea level
        for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (_, nh, n_river) =
                calculate_biome_and_height((wx + dx) as f64, (wz + dz) as f64, &noise);
            assert!(nh > SEA_LEVEL, "Neighbor block must be above sea level");
            assert!(!n_river, "Neighbor block must not be river");
        }
    }
}

#[test]
fn test_find_safe_surface_spawn_different_seeds() {
    let noise1 = NoiseGenerator::new(111);
    let spawn1 = find_safe_surface_spawn(&noise1, 111);

    let noise2 = NoiseGenerator::new(999);
    let spawn2 = find_safe_surface_spawn(&noise2, 999);

    // Different seeds should produce randomized starting coordinates
    assert!((spawn1.x - spawn2.x).abs() > 1.0);
    assert!((spawn1.z - spawn2.z).abs() > 1.0);
}

#[test]
fn test_determine_chunk_tier_near_mid_distant() {
    use crate::world::streaming::determine_chunk_tier;

    let greedy_threshold = 2; // 2 chunks = 32m
    let greedy_threshold_sq = 32.0 * 32.0; // 1024.0
    let lod_threshold_sq = 128.0 * 128.0; // 8 chunks = 128m = 16384.0

    // 1. Near camera (distance 10m < 32m) -> Tier 0 (Standard 1x1 voxel, no merging)
    let near_dist_sq = 10.0 * 10.0;
    let (tier, greedy, lod) = determine_chunk_tier(
        near_dist_sq,
        true,
        lod_threshold_sq,
        true,
        greedy_threshold,
        greedy_threshold_sq,
    );
    assert_eq!(tier, 0, "Near chunk must be Tier 0 (Standard 1x1)");
    assert!(!greedy, "Near chunk must not be greedy meshed");
    assert_eq!(lod, 0, "Near chunk must be LOD 0");

    // 2. Mid-range (distance 60m: between 32m and 128m) -> Tier 1 (Greedy coplanar merging)
    let mid_dist_sq = 60.0 * 60.0;
    let (tier, greedy, lod) = determine_chunk_tier(
        mid_dist_sq,
        true,
        lod_threshold_sq,
        true,
        greedy_threshold,
        greedy_threshold_sq,
    );
    assert_eq!(tier, 1, "Mid-range chunk must be Tier 1 (Greedy)");
    assert!(greedy, "Mid-range chunk must be greedy meshed");
    assert_eq!(lod, 0, "Mid-range chunk must be LOD 0 (blocky voxel)");

    // 3. Distant (distance 150m > 128m) -> Tier 2 (Sloped heightfield LOD)
    let far_dist_sq = 150.0 * 150.0;
    let (tier, greedy, lod) = determine_chunk_tier(
        far_dist_sq,
        true,
        lod_threshold_sq,
        true,
        greedy_threshold,
        greedy_threshold_sq,
    );
    assert_eq!(tier, 2, "Distant chunk must be Tier 2 (Sloped LOD)");
    assert!(!greedy, "Sloped LOD handles its own triangles");
    assert_eq!(lod, 1, "Distant chunk must be LOD 1");

    // 4. When greedy meshing is toggled OFF: mid-range remains Tier 0 (Standard 1x1)
    let (tier, greedy, lod) = determine_chunk_tier(
        mid_dist_sq,
        true,
        lod_threshold_sq,
        false,
        greedy_threshold,
        greedy_threshold_sq,
    );
    assert_eq!(tier, 0, "When greedy is OFF, mid-range must be Tier 0");
    assert!(!greedy);
    assert_eq!(lod, 0);

    // 5. When distant LOD is toggled OFF: distant chunk remains Tier 1 (Greedy)
    let (tier, greedy, lod) = determine_chunk_tier(
        far_dist_sq,
        false,
        lod_threshold_sq,
        true,
        greedy_threshold,
        greedy_threshold_sq,
    );
    assert_eq!(tier, 1, "When LOD is OFF, distant chunk remains Tier 1");
    assert!(greedy);
    assert_eq!(lod, 0);
}

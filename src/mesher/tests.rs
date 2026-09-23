use super::*;
use crate::block::BlockType;
use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};

#[test]
fn test_greedy_meshing_reduces_vertices() {
    let mut chunk = Chunk::new();
    // Fill a 4x4 area of Stone blocks at y = 10
    for lx in 0..4 {
        for lz in 0..4 {
            chunk.set(lx, 10, lz, BlockType::Stone);
        }
    }

    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();

    let std_verts = mesh_standard.count_vertices();
    let greedy_verts = mesh_greedy.count_vertices();

    assert_eq!(std_verts, 192);
    assert_eq!(greedy_verts, 24);
    assert!(greedy_verts < std_verts);
}

#[test]
fn test_procedural_chunk_greedy_reduces_vertices() {
    let noise = crate::noise::NoiseGenerator::new(133742);
    let chunk = crate::world::generate_chunk(0, 0, &noise, 133742);
    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
    let std_v = mesh_standard.count_vertices();
    let greedy_v = mesh_greedy.count_vertices();
    println!("PROCEDURAL CHUNK: Standard = {} verts, Greedy = {} verts", std_v, greedy_v);
    assert!(greedy_v < std_v);
}

#[test]
fn test_water_ocean_renders_only_on_surface() {
    let mut chunk = Chunk::new();
    // Ocean seabed: fill y = 0..=9 with Bedrock
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 0..=9 {
                chunk.set(lx as i32, ly, lz as i32, BlockType::Bedrock);
            }
        }
    }
    // Ocean water: fill y = 10..=20 with Water
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 10..=20 {
                chunk.set(lx as i32, ly, lz as i32, BlockType::Water);
            }
        }
    }

    // Greedy meshing of this ocean chunk
    let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
    assert_eq!(mesh.count_vertices(), 28);
}

#[test]
fn test_waterfall_renders_sides_in_air() {
    let mut chunk = Chunk::new();
    // 1x1 vertical column of water (waterfall) at (5, y, 5) from y = 10 to y = 12 surrounded by Air
    for ly in 10..=12 {
        chunk.set(5, ly, 5, BlockType::Water);
    }
    let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
    assert!(mesh.count_vertices() > 8);
}

#[test]
fn test_seabed_sand_gravel_greedy_merging() {
    let mut chunk = Chunk::new();
    // Create an alternating checkerboard of Sand and Gravel on the seabed at y = 10, covered with Water at y = 11
    for lx in 0..4 {
        for lz in 0..4 {
            let block = if (lx + lz) % 2 == 0 {
                BlockType::Sand
            } else {
                BlockType::Gravel
            };
            chunk.set(lx, 10, lz, block);
            chunk.set(lx, 11, lz, BlockType::Water);
        }
    }

    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();

    let std_verts = mesh_standard.count_vertices();
    let greedy_verts = mesh_greedy.count_vertices();

    assert!(greedy_verts < std_verts);
}

#[test]
fn test_submerged_terrain_renders_against_water_no_holes() {
    let mut chunk = Chunk::new();
    // Sand block at y = 10, Water above it at y = 11
    chunk.set(0, 10, 0, BlockType::Sand);
    chunk.set(0, 11, 0, BlockType::Water);

    let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
    assert_eq!(mesh.count_vertices(), 40);
}

#[test]
fn test_simplify_block_for_lod() {
    // All ore blocks and cobblestone must simplify to Stone
    assert_eq!(simplify_block_for_lod(BlockType::CoalOre), BlockType::Stone);
    assert_eq!(simplify_block_for_lod(BlockType::IronOre), BlockType::Stone);
    assert_eq!(simplify_block_for_lod(BlockType::GoldOre), BlockType::Stone);
    assert_eq!(
        simplify_block_for_lod(BlockType::DiamondOre),
        BlockType::Stone
    );
    assert_eq!(
        simplify_block_for_lod(BlockType::Cobblestone),
        BlockType::Stone
    );

    // Gravel simplifies to Sand
    assert_eq!(simplify_block_for_lod(BlockType::Gravel), BlockType::Sand);

    // Natural surface blocks preserved
    assert_eq!(simplify_block_for_lod(BlockType::Grass), BlockType::Grass);
    assert_eq!(simplify_block_for_lod(BlockType::Water), BlockType::Water);
    assert_eq!(simplify_block_for_lod(BlockType::Snow), BlockType::Snow);
}

#[test]
fn test_sloped_lod_reduces_mountain_slope_triangles() {
    let mut chunk = Chunk::new();

    // Create a 16x16 steep mountain slope where each column rises in Y
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let height = 20 + lx + lz; // Diagonal slope reaching up to y = 50
            for ly in 0..=height {
                chunk.set(lx as i32, ly as i32, lz as i32, BlockType::Stone);
            }
        }
    }

    // Meshing at LOD 0 (detailed 3D greedy voxel steps)
    let mesh_lod0 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 0).unwrap();
    // Meshing at LOD 1 (sloped heightfield)
    let mesh_lod1 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 1).unwrap();

    let verts_lod0 = mesh_lod0.count_vertices();
    let verts_lod1 = mesh_lod1.count_vertices();

    // LOD 1 sloped heightfield must yield a massive reduction in vertices
    assert!(
        verts_lod1 < verts_lod0,
        "LOD 1 sloped heightfield should use fewer vertices ({}) than LOD 0 stepped voxels ({})",
        verts_lod1,
        verts_lod0
    );

    // Expecting at least a 60% reduction in vertex count on steep diagonal mountain
    assert!(
        (verts_lod1 as f32) < (verts_lod0 as f32) * 0.40,
        "Sloped LOD must reduce mountain vertices by >60%, got {} vs {}",
        verts_lod1,
        verts_lod0
    );
}

#[test]
fn test_sloped_lod_groups_ores_on_mountain() {
    let mut chunk = Chunk::new();

    // Mountain slope with exposed DiamondOre, CoalOre, and GoldOre veins
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let height = 15 + lx;
            let ore = match (lx + lz) % 4 {
                0 => BlockType::DiamondOre,
                1 => BlockType::CoalOre,
                2 => BlockType::GoldOre,
                _ => BlockType::Stone,
            };
            for ly in 0..height {
                chunk.set(lx as i32, ly as i32, lz as i32, BlockType::Stone);
            }
            chunk.set(lx as i32, height as i32, lz as i32, ore);
        }
    }

    // Meshing at LOD 1 should successfully generate a unified mesh
    let mesh_lod1 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 1).unwrap();
    assert!(mesh_lod1.count_vertices() > 0);
}

#[test]
fn test_texture_array_properties_and_repeat_mode() {
    use crate::texture::{LAYER_COUNT, TILE_SIZE, create_texture_array};
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler};
    use bevy::render::render_resource::TextureDimension;

    let image = create_texture_array();
    assert_eq!(image.texture_descriptor.size.width, TILE_SIZE as u32);
    assert_eq!(image.texture_descriptor.size.height, TILE_SIZE as u32);
    assert_eq!(
        image.texture_descriptor.size.depth_or_array_layers,
        LAYER_COUNT as u32
    );
    assert_eq!(image.texture_descriptor.dimension, TextureDimension::D2);

    if let ImageSampler::Descriptor(ref desc) = image.sampler {
        assert_eq!(desc.address_mode_u, ImageAddressMode::Repeat);
        assert_eq!(desc.address_mode_v, ImageAddressMode::Repeat);
        assert_eq!(desc.mag_filter, ImageFilterMode::Nearest);
        assert_eq!(desc.min_filter, ImageFilterMode::Nearest);
    } else {
        panic!("Expected ImageSampler::Descriptor with Repeat address mode");
    }
}

#[test]
fn test_greedy_mesh_repeating_uvs_and_layer_attribute() {
    use crate::block::BlockFace;
    use crate::texture::{TextureId, block_texture};
    use bevy::render::mesh::VertexAttributeValues;

    let mut chunk = Chunk::new();
    // Fill a 4x2 area of Stone blocks at y = 10 (X: 0..4, Z: 0..2)
    for lx in 0..4 {
        for lz in 0..2 {
            chunk.set(lx, 10, lz, BlockType::Stone);
        }
    }

    let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();

    // Verify ATTRIBUTE_UV_0 (repeating coordinates from 0..w and 0..h)
    let uv0_values = mesh.attribute(Mesh::ATTRIBUTE_UV_0).expect("ATTRIBUTE_UV_0 must exist");
    if let VertexAttributeValues::Float32x2(uvs) = uv0_values {
        // Find max U and max V across vertices
        let max_u = uvs.iter().map(|uv| uv[0]).fold(0.0f32, f32::max);
        let max_v = uvs.iter().map(|uv| uv[1]).fold(0.0f32, f32::max);
        // The merged quad of 4x2 blocks must have max U or V equal to 4.0 or 2.0
        assert!(
            max_u >= 4.0 || max_v >= 4.0 || max_u >= 2.0,
            "Greedy merged quad must have repeating UV dimensions, got max_u={}, max_v={}",
            max_u,
            max_v
        );
    } else {
        panic!("Expected Float32x2 for ATTRIBUTE_UV_0");
    }

    // Verify ATTRIBUTE_UV_1 (layer attribute)
    let uv1_values = mesh.attribute(Mesh::ATTRIBUTE_UV_1).expect("ATTRIBUTE_UV_1 must exist");
    if let VertexAttributeValues::Float32x2(uv1s) = uv1_values {
        assert_eq!(uv1s.len(), mesh.count_vertices());
        let expected_stone_layer = block_texture(BlockType::Stone, BlockFace::Top).layer();
        assert!((expected_stone_layer - TextureId::Stone as usize as f32).abs() < f32::EPSILON);
        let found_stone = uv1s.iter().any(|uv| (uv[0] - expected_stone_layer).abs() < 1e-4);
        assert!(found_stone, "Expected at least one vertex with Stone layer ID");
    } else {
        panic!("Expected Float32x2 for ATTRIBUTE_UV_1");
    }
}

#[test]
fn test_all_meshers_use_u16_indices() {
    use bevy::render::mesh::Indices;

    let mut chunk = Chunk::new();
    chunk.set(0, 10, 0, BlockType::Stone);

    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
    assert!(matches!(mesh_standard.indices(), Some(Indices::U16(_))));

    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
    assert!(matches!(mesh_greedy.indices(), Some(Indices::U16(_))));

    let mesh_lod = build_chunk_mesh_sloped_lod(&chunk, None, None, None, None, 15).unwrap();
    assert!(matches!(mesh_lod.indices(), Some(Indices::U16(_))));
}


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

    let meshes_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false);
    let meshes_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true);

    let std_verts = meshes_standard.total_vertices();
    let greedy_verts = meshes_greedy.total_vertices();

    assert_eq!(std_verts, 192);
    assert_eq!(greedy_verts, 24);
    assert!(greedy_verts < std_verts);
}

#[test]
fn test_procedural_chunk_greedy_reduces_vertices() {
    let noise = crate::noise::NoiseGenerator::new(133742);
    let chunk = crate::world::generate_chunk(0, 0, &noise, 133742);
    let meshes_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false);
    let meshes_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true);
    let std_v = meshes_standard.total_vertices();
    let greedy_v = meshes_greedy.total_vertices();
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
    let meshes = build_chunk_mesh(&chunk, None, None, None, None, true, true);
    assert!(meshes.has_solid());
    assert!(meshes.has_water());
    assert_eq!(meshes.total_vertices(), 28);
}

#[test]
fn test_waterfall_renders_sides_in_air() {
    let mut chunk = Chunk::new();
    // 1x1 vertical column of water (waterfall) at (5, y, 5) from y = 10 to y = 12 surrounded by Air
    for ly in 10..=12 {
        chunk.set(5, ly, 5, BlockType::Water);
    }
    let meshes = build_chunk_mesh(&chunk, None, None, None, None, true, true);
    let water_mesh = meshes.first_water_mesh().expect("Water mesh must exist for waterfall");
    assert!(water_mesh.count_vertices() > 8);
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

    let meshes_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false);
    let meshes_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true);

    let std_verts = meshes_standard.total_vertices();
    let greedy_verts = meshes_greedy.total_vertices();

    assert!(greedy_verts < std_verts);
}

#[test]
fn test_submerged_terrain_renders_against_water_no_holes() {
    let mut chunk = Chunk::new();
    // Sand block at y = 10, Water above it at y = 11
    chunk.set(0, 10, 0, BlockType::Sand);
    chunk.set(0, 11, 0, BlockType::Water);

    let meshes = build_chunk_mesh(&chunk, None, None, None, None, true, false);
    assert_eq!(meshes.total_vertices(), 40);
    assert!(meshes.has_solid());
    assert!(meshes.has_water());
}

#[test]
fn test_two_pass_water_and_solid_mesh_separation() {
    let mut chunk = Chunk::new();
    chunk.set(0, 10, 0, BlockType::Stone);
    chunk.set(1, 10, 0, BlockType::Water);

    let meshes = build_chunk_mesh(&chunk, None, None, None, None, true, true);
    assert!(meshes.has_solid(), "Solid mesh must exist for stone block");
    assert!(meshes.has_water(), "Water mesh must exist for water block");
    let solid_mesh = meshes.first_solid().unwrap();
    let water_mesh = meshes.first_water().unwrap();
    assert!(solid_mesh.count_vertices() > 0);
    assert!(water_mesh.count_vertices() > 0);
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
    let meshes_lod0 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 0);
    // Meshing at LOD 1 (sloped heightfield)
    let meshes_lod1 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 1);

    let verts_lod0 = meshes_lod0.total_vertices();
    let verts_lod1 = meshes_lod1.total_vertices();

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
    let meshes_lod1 = build_chunk_mesh_lod(&chunk, None, None, None, None, true, true, 1);
    assert!(meshes_lod1.total_vertices() > 0);
}

#[test]
fn test_texture_array_properties_and_repeat_mode() {
    use crate::texture::{create_texture_array, LAYER_COUNT, TILE_SIZE};
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
    use crate::texture::{block_texture, TextureId};
    use bevy::render::mesh::VertexAttributeValues;

    let mut chunk = Chunk::new();
    // Fill a 4x2 area of Stone blocks at y = 10 (X: 0..4, Z: 0..2)
    for lx in 0..4 {
        for lz in 0..2 {
            chunk.set(lx, 10, lz, BlockType::Stone);
        }
    }

    let meshes = build_chunk_mesh(&chunk, None, None, None, None, true, true);
    let mesh = meshes.first_solid_mesh().expect("Solid mesh must exist");

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

    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).first_solid_mesh().unwrap();
    assert!(matches!(mesh_standard.indices(), Some(Indices::U16(_))));

    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).first_solid_mesh().unwrap();
    assert!(matches!(mesh_greedy.indices(), Some(Indices::U16(_))));

    let mesh_lod = build_chunk_mesh_sloped_lod(&chunk, None, None, None, None, 15).first_solid_mesh().unwrap();
    assert!(matches!(mesh_lod.indices(), Some(Indices::U16(_))));
}

#[test]
fn test_all_meshers_omit_attribute_color_and_pack_shade() {
    use bevy::render::mesh::VertexAttributeValues;

    let mut chunk = Chunk::new();
    chunk.set(0, 10, 0, BlockType::Stone);

    let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).first_solid_mesh().unwrap();
    assert!(mesh_standard.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    let uv1_std = mesh_standard.attribute(Mesh::ATTRIBUTE_UV_1).expect("UV_1 must exist");
    if let VertexAttributeValues::Float32x2(uvs) = uv1_std {
        assert!(uvs.iter().all(|uv| uv[1] > 0.0 && uv[1] <= 1.0), "Shade must be packed in UV_1.y");
    }

    let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).first_solid_mesh().unwrap();
    assert!(mesh_greedy.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    let uv1_greedy = mesh_greedy.attribute(Mesh::ATTRIBUTE_UV_1).expect("UV_1 must exist");
    if let VertexAttributeValues::Float32x2(uvs) = uv1_greedy {
        assert!(uvs.iter().all(|uv| uv[1] > 0.0 && uv[1] <= 1.0), "Shade must be packed in UV_1.y");
    }

    let mesh_lod = build_chunk_mesh_sloped_lod(&chunk, None, None, None, None, 15).first_solid_mesh().unwrap();
    assert!(mesh_lod.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    let uv1_lod = mesh_lod.attribute(Mesh::ATTRIBUTE_UV_1).expect("UV_1 must exist");
    if let VertexAttributeValues::Float32x2(uvs) = uv1_lod {
        assert!(uvs.iter().all(|uv| uv[1] > 0.0 && uv[1] <= 1.0), "Shade must be packed in UV_1.y");
    }
}

#[test]
fn test_subchunk_section_isolation_and_empty_sections() {
    let mut chunk = Chunk::new();

    // Section 0 (y = 0..16): completely solid stone
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 0..16 {
                chunk.set(lx as i32, ly, lz as i32, BlockType::Stone);
            }
        }
    }

    // Section 1 (y = 16..32): completely solid stone EXCEPT a hollow 2x2x2 cave at (5..7, 20..22, 5..7)
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 16..32 {
                chunk.set(lx as i32, ly, lz as i32, BlockType::Stone);
            }
        }
    }
    // Hollow out the cave
    for lx in 5..7 {
        for lz in 5..7 {
            for ly in 20..22 {
                chunk.set(lx, ly, lz, BlockType::Air);
            }
        }
    }

    // Section 2 (y = 32..48): solid stone from y = 32..=35 (terrain surface at y = 35), air above
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 32..=35 {
                chunk.set(lx as i32, ly, lz as i32, BlockType::Grass);
            }
        }
    }

    // Sections 3..8 (y = 48..128): completely Air

    // Solid neighbor chunks on all 4 horizontal sides so no border faces are generated underground
    let mut solid_neighbor = Chunk::new();
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            for ly in 0..36 {
                solid_neighbor.set(lx as i32, ly, lz as i32, BlockType::Stone);
            }
        }
    }

    let meshes = build_chunk_mesh(
        &chunk,
        Some(&solid_neighbor),
        Some(&solid_neighbor),
        Some(&solid_neighbor),
        Some(&solid_neighbor),
        true,
        true,
    );

    // Section 0: completely solid with solid neighbors everywhere around and above it -> 0 vertices, None!
    assert!(
        meshes.sections[0].solid.is_none(),
        "Fully solid underground section must have None mesh (0 vertices)"
    );

    // Section 1: has the interior cave! Must produce mesh ONLY for the cave interior walls!
    assert!(
        meshes.sections[1].solid.is_some(),
        "Section with cave must have a solid mesh"
    );
    let cave_verts = meshes.sections[1].solid.as_ref().unwrap().count_vertices();
    assert!(
        cave_verts > 0,
        "Section 1 should contain vertices for cave walls"
    );

    // Section 2: has surface grass -> Must produce mesh for top surface
    assert!(
        meshes.sections[2].solid.is_some(),
        "Surface section must have a solid mesh"
    );

    // Sections 3..8: completely empty air -> Must all be None (0 vertices, 0 draw calls!)
    for sy in 3..CHUNK_SECTIONS {
        assert!(
            meshes.sections[sy].is_empty(),
            "Empty air section {sy} must produce None mesh"
        );
    }
}

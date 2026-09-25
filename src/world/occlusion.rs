use std::collections::VecDeque;
use bevy::prelude::*;
use crate::chunk::CHUNK_HEIGHT;
use crate::mesher::{SectionFace, CHUNK_SECTIONS, SECTION_HEIGHT};
use crate::world::grid::{ChunkSection, WorldGrid, FULL_CHUNK_SECTION_INDEX};

pub const OCCLUSION_RADIUS: i32 = 8;
pub const OCCLUSION_GRID_WIDTH: usize = (OCCLUSION_RADIUS * 2 + 1) as usize; // 17
pub const OCCLUSION_CHUNKS: usize = OCCLUSION_GRID_WIDTH * OCCLUSION_GRID_WIDTH; // 289

/// Stack/L1-cache friendly bitset recording visible sub-chunk sections within LOD 0 radius (17x17 chunks).
///
/// 289 bytes total: each byte represents one chunk column, where bit `sy` (0..7) indicates whether
/// section `sy` is visible from the camera.
#[derive(Clone, Copy, Debug)]
pub struct OcclusionBitset {
    pub bits: [u8; OCCLUSION_CHUNKS],
}

impl Default for OcclusionBitset {
    fn default() -> Self {
        Self {
            bits: [0; OCCLUSION_CHUNKS],
        }
    }
}

impl OcclusionBitset {
    #[inline]
    #[must_use]
    pub fn is_visible(&self, dx: i32, dz: i32, section_y: u8) -> bool {
        if dx.abs() > OCCLUSION_RADIUS || dz.abs() > OCCLUSION_RADIUS || section_y >= CHUNK_SECTIONS as u8 {
            return true; // Conservative: outside LOD0 radius is always visible
        }
        let ux = (dx + OCCLUSION_RADIUS) as usize;
        let uz = (dz + OCCLUSION_RADIUS) as usize;
        let idx = ux + uz * OCCLUSION_GRID_WIDTH;
        (self.bits[idx] & (1 << section_y)) != 0
    }

    #[inline]
    pub fn mark_visible(&mut self, dx: i32, dz: i32, section_y: u8) {
        if dx.abs() <= OCCLUSION_RADIUS && dz.abs() <= OCCLUSION_RADIUS && section_y < CHUNK_SECTIONS as u8 {
            let ux = (dx + OCCLUSION_RADIUS) as usize;
            let uz = (dz + OCCLUSION_RADIUS) as usize;
            let idx = ux + uz * OCCLUSION_GRID_WIDTH;
            self.bits[idx] |= 1 << section_y;
        }
    }
}

/// Checks if the camera is positioned directly beneath the open sky with no opaque ceiling.
#[must_use]
pub fn is_under_open_sky(world: &WorldGrid, cam_pos: Vec3) -> bool {
    let block_x = cam_pos.x.floor() as i32;
    let block_z = cam_pos.z.floor() as i32;
    let start_y = (cam_pos.y.floor() as i32).clamp(0, CHUNK_HEIGHT as i32 - 1);
    for y in start_y..CHUNK_HEIGHT as i32 {
        if world.get_block(IVec3::new(block_x, y, block_z)).is_opaque() {
            return false;
        }
    }
    true
}

/// Computes the set of visible sub-chunk sections within LOD 0 radius using the section reachability graph.
///
/// If the camera is outdoors under the open sky (or in flight), subterranean cave sections that have
/// no air path to the sky are excluded from the bitset.
/// Computes the set of visible sub-chunk sections within LOD 0 radius using the section reachability graph.
///
/// If the camera is outdoors under the open sky (or in flight), sunlight illuminates all terrain
/// columns from above, keeping surfaces, rivers, oceans, and seabeds fully visible while culling
/// enclosed subterranean caves.
/// If the camera is underground inside a cave, only sections topologically reachable along air corridors
/// from the camera are marked visible.
#[must_use]
pub fn compute_section_occlusion(world: &WorldGrid, cam_pos: Vec3) -> OcclusionBitset {
    let mut queue = VecDeque::with_capacity(2048);
    compute_section_occlusion_with_queue(world, cam_pos, &mut queue)
}

/// Core occlusion BFS algorithm utilizing a reusable `VecDeque` queue to eliminate heap allocations.
#[must_use]
pub fn compute_section_occlusion_with_queue(
    world: &WorldGrid,
    cam_pos: Vec3,
    queue: &mut VecDeque<(IVec2, u8, Option<SectionFace>)>,
) -> OcclusionBitset {
    let mut bitset = OcclusionBitset::default();
    queue.clear();

    let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(
        cam_pos.x.floor() as i32,
        cam_pos.z.floor() as i32,
    );
    let cam_sy_raw = (cam_pos.y / SECTION_HEIGHT as f32).floor() as i32;
    let outdoors = cam_sy_raw >= CHUNK_SECTIONS as i32 || is_under_open_sky(world, cam_pos);

    if outdoors {
        // High in the sky or outdoors on the surface: sunlight illuminates all sections open to the sky
        for dz in -OCCLUSION_RADIUS..=OCCLUSION_RADIUS {
            for dx in -OCCLUSION_RADIUS..=OCCLUSION_RADIUS {
                let chunk_coord = player_chunk + IVec2::new(dx, dz);
                let top_sy = (CHUNK_SECTIONS - 1) as u8;
                let n_conn = world
                    .chunk_connectivity
                    .get(&chunk_coord)
                    .map_or_else(crate::mesher::SectionConnectivity::full, |secs| secs[top_sy as usize]);
                if !n_conn.is_solid && n_conn.mask[SectionFace::Up as usize] != 0 {
                    bitset.mark_visible(dx, dz, top_sy);
                    queue.push_back((chunk_coord, top_sy, Some(SectionFace::Up)));
                }
            }
        }
    } else {
        // Underground inside a cave: seed strictly from camera's current cave section
        let start_sy = cam_sy_raw.clamp(0, (CHUNK_SECTIONS - 1) as i32) as u8;
        bitset.mark_visible(0, 0, start_sy);
        queue.push_back((player_chunk, start_sy, None));
    }

    // BFS flood fill across section boundaries
    while let Some((coord, sy, entry_face)) = queue.pop_front() {
        let diff = coord - player_chunk;
        if diff.x.abs() > OCCLUSION_RADIUS || diff.y.abs() > OCCLUSION_RADIUS {
            continue;
        }

        bitset.mark_visible(diff.x, diff.y, sy);

        // Fetch connectivity for this section (default to full if unmeshed)
        let conn = world
            .chunk_connectivity
            .get(&coord)
            .map_or_else(crate::mesher::SectionConnectivity::full, |secs| secs[sy as usize]);

        if conn.is_solid {
            continue;
        }

        // For each face, test if line of sight can exit through it
        for exit_face in SectionFace::ALL {
            let can_exit = entry_face.map_or(true, |in_face| conn.can_see(in_face, exit_face));
            if !can_exit {
                continue;
            }

            let (n_chunk, n_sy) = match exit_face {
                SectionFace::Up => {
                    if sy + 1 < CHUNK_SECTIONS as u8 {
                        (coord, sy + 1)
                    } else {
                        continue;
                    }
                }
                SectionFace::Down => {
                    if sy > 0 {
                        (coord, sy - 1)
                    } else {
                        continue;
                    }
                }
                SectionFace::North => (coord + IVec2::new(0, 1), sy),
                SectionFace::South => (coord + IVec2::new(0, -1), sy),
                SectionFace::East => (coord + IVec2::new(1, 0), sy),
                SectionFace::West => (coord + IVec2::new(-1, 0), sy),
            };

            let n_diff = n_chunk - player_chunk;
            if n_diff.x.abs() > OCCLUSION_RADIUS || n_diff.y.abs() > OCCLUSION_RADIUS {
                continue;
            }

            if bitset.is_visible(n_diff.x, n_diff.y, n_sy) {
                continue;
            }

            let entry_face = exit_face.opposite();

            // Verify that neighbor section can actually receive light through entry_face
            let n_conn = world
                .chunk_connectivity
                .get(&n_chunk)
                .map_or_else(crate::mesher::SectionConnectivity::full, |secs| secs[n_sy as usize]);

            if n_conn.is_solid || n_conn.mask[entry_face as usize] == 0 {
                // Neighbor section has a solid wall on this boundary face; light cannot penetrate!
                continue;
            }

            bitset.mark_visible(n_diff.x, n_diff.y, n_sy);
            queue.push_back((n_chunk, n_sy, Some(entry_face)));
        }
    }

    bitset
}

/// Resource caching previous camera section to avoid redundant occlusion calculations when stationary.
#[derive(Resource, Default)]
pub struct SectionOcclusionCache {
    pub last_chunk: IVec2,
    pub last_sy: i32,
    pub last_outdoors: bool,
    pub bitset: OcclusionBitset,
    pub queue: VecDeque<(IVec2, u8, Option<SectionFace>)>,
}

/// Bevy system executing the software occlusion culling pass.
///
/// Evaluates the reachability graph from camera position and switches `Visibility::Hidden`
/// on completely enclosed subterranean sections.
pub fn section_occlusion_system(
    world_grid: Option<Res<WorldGrid>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut cache: ResMut<SectionOcclusionCache>,
    mut section_query: Query<(&ChunkSection, &mut Visibility)>,
) {
    let Some(world) = world_grid else { return };
    let Ok(cam_transform) = camera_query.single() else { return };

    let cam_pos = cam_transform.translation;
    let (cam_chunk, _, _) = WorldGrid::world_to_chunk_coord(
        cam_pos.x.floor() as i32,
        cam_pos.z.floor() as i32,
    );
    let cam_sy = (cam_pos.y / SECTION_HEIGHT as f32).floor() as i32;
    let outdoors = cam_sy >= CHUNK_SECTIONS as i32 || is_under_open_sky(&world, cam_pos);

    // Only recompute occlusion BFS when crossing into a different section or indoor/outdoor state
    if cam_chunk != cache.last_chunk || cam_sy != cache.last_sy || outdoors != cache.last_outdoors {
        cache.bitset = compute_section_occlusion_with_queue(&world, cam_pos, &mut cache.queue);
        cache.last_chunk = cam_chunk;
        cache.last_sy = cam_sy;
        cache.last_outdoors = outdoors;

        let bitset = cache.bitset;
        for (section, mut visibility) in &mut section_query {
            // Distant continuous LOD 1 chunk entities are never culled by sub-chunk cave occlusion
            if section.section_y == FULL_CHUNK_SECTION_INDEX {
                if *visibility != Visibility::Inherited {
                    *visibility = Visibility::Inherited;
                }
                continue;
            }

            let diff = section.chunk - cam_chunk;
            if diff.x.abs() <= OCCLUSION_RADIUS && diff.y.abs() <= OCCLUSION_RADIUS {
                let is_vis = bitset.is_visible(diff.x, diff.y, section.section_y);
                let target = if is_vis {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *visibility != target {
                    *visibility = target;
                }
            } else if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;
    use crate::chunk::Chunk;
    use crate::mesher::connectivity::compute_section_connectivity;

    #[test]
    fn test_occlusion_bitset_mark_and_query() {
        let mut bitset = OcclusionBitset::default();
        assert!(!bitset.is_visible(0, 0, 2));
        bitset.mark_visible(0, 0, 2);
        assert!(bitset.is_visible(0, 0, 2));
        assert!(!bitset.is_visible(0, 0, 1));
        assert!(!bitset.is_visible(1, 0, 2));
    }

    #[test]
    fn test_occlusion_bitset_outside_radius_is_always_visible() {
        let bitset = OcclusionBitset::default();
        // Outside LOD 0 radius (e.g. distant LOD 1 chunks) are conservatively visible
        assert!(bitset.is_visible(9, 0, 0));
        assert!(bitset.is_visible(-9, 0, 0));
    }

    #[test]
    fn test_caves_under_solid_terrain_are_culled_from_surface() {
        let mut world = WorldGrid::new(crate::world::types::WorldSeed(12345));

        // Create chunk at (0, 0)
        let mut chunk = Chunk::new();
        // Fill sections 0..3 (y=0..64) with stone, but leave a hollow cave in section 1 (y=20..28)
        for y in 0..64 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Stone);
                }
            }
        }
        // Hollow cave in section 1 (does not touch boundaries)
        for y in 20..28 {
            for z in 4..12 {
                for x in 4..12 {
                    chunk.set_fast(x, y, z, BlockType::Air);
                }
            }
        }

        // Compute connectivity for all 8 sections
        let mut conns = [crate::mesher::SectionConnectivity::default(); CHUNK_SECTIONS];
        for sy in 0..CHUNK_SECTIONS {
            conns[sy] = compute_section_connectivity(&chunk, sy);
        }

        world.chunks.insert(IVec2::ZERO, chunk);
        world.chunk_connectivity.insert(IVec2::ZERO, conns);

        // Camera is on the surface at Y = 70.0 (open to the sky)
        let cam_pos = Vec3::new(8.0, 70.0, 8.0);
        let bitset = compute_section_occlusion(&world, cam_pos);

        // Section 1 (the buried cave) MUST be culled!
        assert!(
            !bitset.is_visible(0, 0, 1),
            "Buried cave section 1 must be culled when camera is on the surface!"
        );

        // Camera enters the cave at Y = 24.0
        let cave_cam_pos = Vec3::new(8.0, 24.0, 8.0);
        let cave_bitset = compute_section_occlusion(&world, cave_cam_pos);

        // Section 1 MUST be visible when inside the cave!
        assert!(
            cave_bitset.is_visible(0, 0, 1),
            "Cave section 1 must be visible when camera is inside the cave!"
        );
    }

    #[test]
    fn test_seabed_is_visible_under_ocean_water_from_surface() {
        let mut world = WorldGrid::new(crate::world::types::WorldSeed(12345));

        // Create ocean chunk at (0, 0)
        let mut chunk = Chunk::new();
        // Seabed stone at y=0..50 (section 0, 1, 2 and lower part of section 3)
        for y in 0..50 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Stone);
                }
            }
        }
        // Sand seabed at y=50..52
        for y in 50..52 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Sand);
                }
            }
        }
        // Ocean water from y=52 to y=64 (section 3 and 4)
        for y in 52..=64 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Water);
                }
            }
        }

        let mut conns = [crate::mesher::SectionConnectivity::default(); CHUNK_SECTIONS];
        for sy in 0..CHUNK_SECTIONS {
            conns[sy] = compute_section_connectivity(&chunk, sy);
        }

        world.chunks.insert(IVec2::ZERO, chunk);
        world.chunk_connectivity.insert(IVec2::ZERO, conns);

        // Camera is on the surface looking down at the ocean at Y = 66.0
        let cam_pos = Vec3::new(8.0, 66.0, 8.0);
        let bitset = compute_section_occlusion(&world, cam_pos);

        // Section 4 (water surface) MUST be visible
        assert!(bitset.is_visible(0, 0, 4), "Water surface must be visible");
        // Section 3 (seabed with sand and water) MUST be visible through the transparent water
        assert!(bitset.is_visible(0, 0, 3), "Seabed section must be visible under water");
        // Deep subterranean bedrock section 0 MUST be culled
        assert!(!bitset.is_visible(0, 0, 0), "Deep bedrock must be culled");
    }
}

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::mesher::build_chunk_mesh;
use crate::noise::{fbm_2d, fbm_3d, perlin_2d, ridged_fbm_2d};

pub const SEA_LEVEL: i32 = 24;
pub const VIEW_DISTANCE: i32 = 4;
pub const MAX_CHUNKS_PER_FRAME: usize = 2;

#[derive(Resource)]
pub struct WorldGrid {
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_entities: HashMap<IVec2, Entity>,
    pub last_player_chunk: IVec2,
    pub generation_queue: Vec<IVec2>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        Self {
            chunks: HashMap::default(),
            chunk_entities: HashMap::default(),
            last_player_chunk: IVec2::new(i32::MAX, i32::MAX),
            generation_queue: Vec::new(),
        }
    }
}

impl WorldGrid {
    #[inline]
    pub fn world_to_chunk_coord(wx: i32, wz: i32) -> (IVec2, usize, usize) {
        let cx = wx.div_euclid(CHUNK_WIDTH as i32);
        let cz = wz.div_euclid(CHUNK_DEPTH as i32);
        let lx = wx.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_DEPTH as i32) as usize;
        (IVec2::new(cx, cz), lx, lz)
    }

    pub fn get_block(&self, pos: IVec3) -> BlockType {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }
        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get(&c_coord) {
            chunk.get(lx as i32, pos.y, lz as i32)
        } else {
            BlockType::Air
        }
    }

    pub fn is_solid_at(&self, pos: IVec3) -> bool {
        self.get_block(pos).is_solid()
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec2> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return Vec::new();
        }

        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        let mut dirty_chunks = Vec::new();

        if let Some(chunk) = self.chunks.get_mut(&c_coord) {
            chunk.set(lx as i32, pos.y, lz as i32, block);
            dirty_chunks.push(c_coord);

            if lx == 0 {
                dirty_chunks.push(c_coord + IVec2::new(-1, 0));
            } else if lx == CHUNK_WIDTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(1, 0));
            }

            if lz == 0 {
                dirty_chunks.push(c_coord + IVec2::new(0, -1));
            } else if lz == CHUNK_DEPTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(0, 1));
            }
        }

        dirty_chunks
    }
}

/// Calcola l'altezza della superficie terrestre in base a mare, pianure, fiumi e montagne
pub fn calculate_surface_height(wx: f64, wz: f64) -> (i32, bool) {
    // 1. Continentalness (Oceani vs Continenti)
    let cont = fbm_2d(wx * 0.003, wz * 0.003, 3, 0.5, 2.0);

    // 2. Colline e alture base
    let hills = fbm_2d(wx * 0.012, wz * 0.012, 3, 0.5, 2.0);

    // 3. Montagne aguzze (Ridged Multi-Fractal)
    let mountain = ridged_fbm_2d(wx * 0.007, wz * 0.007, 4, 0.5, 2.0);

    // Base height calcolata
    let base_height = if cont < -0.15 {
        // Bacino oceanico profondo
        12.0 + (cont + 0.15) * 20.0
    } else if cont < 0.1 {
        // Coste, spiagge e pianure
        (SEA_LEVEL as f64) + hills * 6.0
    } else {
        // Altopiani e catene montuose
        let mountain_weight = ((cont - 0.1) * 3.0).clamp(0.0, 1.0);
        (SEA_LEVEL as f64 + 6.0) + hills * 8.0 + mountain * 24.0 * mountain_weight
    };

    // 4. Fiumi: intagliano canyon e valli che scendono verso il mare
    let river_noise = perlin_2d(wx * 0.005 + 120.0, wz * 0.005 + 240.0).abs();
    let is_river = river_noise < 0.045 && cont > -0.1;

    let final_height = if is_river {
        let river_factor = (river_noise / 0.045).clamp(0.0, 1.0);
        let river_bed = (SEA_LEVEL as f64 - 3.0).min(base_height - 4.0);
        river_bed + (base_height - river_bed) * river_factor
    } else {
        base_height
    };

    let clamped = (final_height.round() as i32).clamp(3, (CHUNK_HEIGHT - 6) as i32);
    (clamped, is_river)
}

fn pseudo_hash(x: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x45d9f3b) ^ (z as u32).wrapping_mul(0x27d4eb2d);
    h = ((h >> 16) ^ h).wrapping_mul(0x45d9f3b);
    (h >> 16) ^ h
}

/// Genera un singolo chunk con montagne, mare, fiumi, caverne 3D e alberi
pub fn generate_chunk(cx: i32, cz: i32) -> Chunk {
    let mut chunk = Chunk::new();
    let world_base_x = cx * CHUNK_WIDTH as i32;
    let world_base_z = cz * CHUNK_DEPTH as i32;

    // Cache delle altezze superficiali per questa colonna 16x16
    let mut surface_heights = [[0i32; CHUNK_DEPTH]; CHUNK_WIDTH];
    let mut river_flags = [[false; CHUNK_DEPTH]; CHUNK_WIDTH];

    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = (world_base_x + lx as i32) as f64;
            let wz = (world_base_z + lz as i32) as f64;
            let (h, is_river) = calculate_surface_height(wx, wz);
            surface_heights[lx][lz] = h;
            river_flags[lx][lz] = is_river;
        }
    }

    // 1. Riempimento blocchi di base (Roccia, Terra, Sabbia, Erba, Neve, Acqua)
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = surface_heights[lx][lz];

            // Bedrock indistruttibile sul fondo
            chunk.set(lx as i32, 0, lz as i32, BlockType::Bedrock);
            if pseudo_hash(wx, wz) % 2 == 0 {
                chunk.set(lx as i32, 1, lz as i32, BlockType::Bedrock);
            }

            for y in 1..=h {
                let block = if y == h {
                    if h < SEA_LEVEL {
                        BlockType::Sand
                    } else if h <= SEA_LEVEL + 2 {
                        BlockType::Sand
                    } else if h > 52 {
                        BlockType::Snow
                    } else if h > 44 {
                        BlockType::Stone
                    } else {
                        BlockType::Grass
                    }
                } else if y >= h - 3 {
                    if h <= SEA_LEVEL + 2 {
                        BlockType::Sand
                    } else {
                        BlockType::Dirt
                    }
                } else {
                    BlockType::Stone
                };

                chunk.set(lx as i32, y, lz as i32, block);
            }

            // Riempimento dell'acqua fino a SEA_LEVEL per mari, fiumi e laghi
            if h < SEA_LEVEL {
                for y in (h + 1)..=SEA_LEVEL {
                    chunk.set(lx as i32, y, lz as i32, BlockType::Water);
                }
            }
        }
    }

    // 2. Caverne e Gallerie Sotterranee con Rumore 3D
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = (world_base_x + lx as i32) as f64;
            let wz = (world_base_z + lz as i32) as f64;
            let h = surface_heights[lx][lz];

            let max_cave_y = (h - 3).min(56);
            if max_cave_y <= 3 {
                continue;
            }

            for y in 3..max_cave_y {
                let wy = y as f64;

                // Non forare il fondale di mare/fiume per non prosciugare l'acqua
                if h <= SEA_LEVEL && y >= h - 4 {
                    continue;
                }

                // Tunnel tortuosi (spaghetti caves)
                let n1 = fbm_3d(wx * 0.04, wy * 0.06, wz * 0.04, 2, 0.5, 2.0);
                let n2 = fbm_3d(wx * 0.04 + 31.4, wy * 0.06, wz * 0.04 + 73.1, 2, 0.5, 2.0);

                let is_tunnel = (n1 * n1 + n2 * n2) < 0.013;

                // Grandi stanze ipogee (cheese caves)
                let is_room = if y < 30 {
                    let n_room = fbm_3d(wx * 0.025, wy * 0.04, wz * 0.025, 2, 0.5, 2.0);
                    n_room < -0.44
                } else {
                    false
                };

                if is_tunnel || is_room {
                    chunk.set(lx as i32, y, lz as i32, BlockType::Air);
                }
            }
        }
    }

    // 3. Alberi procedurali
    for lx in 2..(CHUNK_WIDTH - 2) {
        for lz in 2..(CHUNK_DEPTH - 2) {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = surface_heights[lx][lz];

            if h > SEA_LEVEL + 2
                && h < 44
                && !river_flags[lx][lz]
                && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass
                && pseudo_hash(wx, wz) % 37 == 0
                && h + 6 < CHUNK_HEIGHT as i32
            {
                // Tronco
                for ty in (h + 1)..=(h + 4) {
                    chunk.set(lx as i32, ty, lz as i32, BlockType::Wood);
                }

                // Chioma foglie
                for dx in -2_i32..=2_i32 {
                    for dz in -2_i32..=2_i32 {
                        for dy in (h + 3)..=(h + 6) {
                            if dy == h + 6 && (dx.abs() > 1 || dz.abs() > 1) {
                                continue;
                            }
                            if dx.abs() == 2 && dz.abs() == 2 && dy >= h + 5 {
                                continue;
                            }
                            let tx = lx as i32 + dx;
                            let tz = lz as i32 + dz;
                            if Chunk::in_bounds(tx, dy, tz)
                                && chunk.get(tx, dy, tz) == BlockType::Air
                            {
                                chunk.set(tx, dy, tz, BlockType::Leaves);
                            }
                        }
                    }
                }
            }
        }
    }

    chunk
}

/// Aggiorna o crea la mesh per il chunk indicato
pub fn update_chunk_mesh(
    coord: &IVec2,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let Some(chunk) = world.chunks.get(coord).cloned() else {
        return;
    };

    let new_mesh = build_chunk_mesh(&chunk, coord.x, coord.y, |wx, wy, wz| {
        world.get_block(IVec3::new(wx, wy, wz))
    });

    let world_pos = Vec3::new(
        (coord.x * CHUNK_WIDTH as i32) as f32,
        0.0,
        (coord.y * CHUNK_DEPTH as i32) as f32,
    );

    if let Some(&entity) = world.chunk_entities.get(coord) {
        if let Some(mesh) = new_mesh {
            commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
        } else {
            commands.entity(entity).despawn();
            world.chunk_entities.remove(coord);
        }
    } else if let Some(mesh) = new_mesh {
        let material = materials.add(StandardMaterial {
            cull_mode: None,
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            ..default()
        });

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(world_pos),
            ))
            .id();

        world.chunk_entities.insert(*coord, entity);
    }
}

/// Sistema continuo di streaming dei chunk in base alla posizione della telecamera del giocatore
pub fn world_streaming_system(
    mut commands: Commands,
    camera_query: Query<&Transform, With<FpsCamera>>,
    mut world: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };

    let px = cam_transform.translation.x as i32;
    let pz = cam_transform.translation.z as i32;
    let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(px, pz);

    // Se il giocatore ha cambiato chunk o la coda è vuota, ricalcoliamo la coda di caricamento
    if player_chunk != world.last_player_chunk {
        world.last_player_chunk = player_chunk;

        let mut needed_chunks = Vec::new();
        for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
                let coord = player_chunk + IVec2::new(dx, dz);
                if !world.chunks.contains_key(&coord) {
                    needed_chunks.push(coord);
                }
            }
        }

        // Ordina per vicinanza al giocatore (carica prima i chunk più vicini)
        needed_chunks.sort_by_key(|c| {
            let diff = *c - player_chunk;
            diff.x * diff.x + diff.y * diff.y
        });

        world.generation_queue = needed_chunks;

        // Unload dei chunk fuori dalla view distance (+ 2 di margine per evitare oscillazioni)
        let max_dist = VIEW_DISTANCE + 2;
        let mut chunks_to_remove = Vec::new();

        for coord in world.chunks.keys() {
            let diff = *coord - player_chunk;
            if diff.x.abs() > max_dist || diff.y.abs() > max_dist {
                chunks_to_remove.push(*coord);
            }
        }

        for coord in chunks_to_remove {
            if let Some(entity) = world.chunk_entities.remove(&coord) {
                commands.entity(entity).despawn();
            }
            world.chunks.remove(&coord);
        }
    }

    // Carica e genera fino a MAX_CHUNKS_PER_FRAME per frame per mantenere 60 FPS stabili
    let mut generated_this_frame = 0;

    while generated_this_frame < MAX_CHUNKS_PER_FRAME && !world.generation_queue.is_empty() {
        let coord = world.generation_queue.remove(0);
        if world.chunks.contains_key(&coord) {
            continue;
        }

        let chunk = generate_chunk(coord.x, coord.y);
        world.chunks.insert(coord, chunk);

        // Effettua subito il meshing per il nuovo chunk
        update_chunk_mesh(
            &coord,
            &mut commands,
            &mut world,
            &mut meshes,
            &mut materials,
        );

        // Se necessario, aggiorna i vicini già esistenti ai bordi per evitare cuciture
        for neighbor_coord in [
            coord + IVec2::new(-1, 0),
            coord + IVec2::new(1, 0),
            coord + IVec2::new(0, -1),
            coord + IVec2::new(0, 1),
        ] {
            if world.chunks.contains_key(&neighbor_coord) {
                update_chunk_mesh(
                    &neighbor_coord,
                    &mut commands,
                    &mut world,
                    &mut meshes,
                    &mut materials,
                );
            }
        }

        generated_this_frame += 1;
    }
}

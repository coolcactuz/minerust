use bevy::prelude::Vec3;

use crate::noise::NoiseGenerator;
use crate::world::terrain::{calculate_biome_and_height, pseudo_hash_3d};
use crate::world::types::{BiomeType, SEA_LEVEL};

/// Helper verifying if a candidate coordinate is on stable, dry surface land (not in water)
fn is_safe_spawn_location(wx: i32, wz: i32, noise: &NoiseGenerator, seed: u64) -> bool {
    let (biome, h, is_river) = calculate_biome_and_height(wx as f64, wz as f64, noise);

    // 1. Must be strictly above sea level (water level is SEA_LEVEL = 64) and not in a river
    if h <= SEA_LEVEL || is_river {
        return false;
    }

    // 2. Must not be ocean biomes
    if biome == BiomeType::Ocean || biome == BiomeType::FrozenOcean {
        return false;
    }

    // 3. Avoid excessive high mountain summits (e.g. Y > 95) for a comfortable initial spawn
    if h > 95 {
        return false;
    }

    // 4. Ensure no tree trunk or cactus spawns at this exact voxel
    let hash = pseudo_hash_3d(wx, h, wz, seed);
    let has_tree_or_cactus = match biome {
        BiomeType::Forest => hash.is_multiple_of(16),
        BiomeType::Plains => hash.is_multiple_of(45),
        BiomeType::SnowyTundra => hash.is_multiple_of(35),
        BiomeType::Mountains => h <= 84 && hash.is_multiple_of(30),
        BiomeType::Desert => hash.is_multiple_of(41),
        _ => false,
    };
    if has_tree_or_cactus {
        return false;
    }

    // 5. Check stability of 4 adjacent horizontal neighbors:
    // must also be dry land above sea level, not in river, not ocean, and gentle slope
    for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let (n_biome, nh, n_river) =
            calculate_biome_and_height((wx + dx) as f64, (wz + dz) as f64, noise);
        if nh <= SEA_LEVEL
            || n_river
            || n_biome == BiomeType::Ocean
            || n_biome == BiomeType::FrozenOcean
            || (nh - h).abs() > 2
        {
            return false;
        }
    }

    true
}

/// Finds a safe, random surface spawn position on dry land (not in water).
///
/// Guarantees:
/// 1. Surface elevation: Player spawns standing safely on top of the terrain surface.
/// 2. Dry land: Elevation is strictly above `SEA_LEVEL` (h > 64), not in a river,
///    and not in an ocean biome. Adjacent blocks are also confirmed to be dry and stable.
#[must_use]
pub fn find_safe_surface_spawn(noise: &NoiseGenerator, seed: u64) -> Vec3 {
    let mut rng = seed
        .wrapping_mul(0x517cc1b727220a95)
        .wrapping_add(0x9e3779b97f4a7c15);
    let next_i32 = |state: &mut u64, min: i32, max: i32| -> i32 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let span = (max - min + 1) as u64;
        let val = ((*state >> 32) % span) as i32;
        min + val
    };

    // 1. Try up to 256 random candidate coordinates across a +/- 800 block radius
    for _ in 0..256 {
        let rx = next_i32(&mut rng, -800, 800);
        let rz = next_i32(&mut rng, -800, 800);

        if is_safe_spawn_location(rx, rz, noise, seed) {
            let (_, h, _) = calculate_biome_and_height(rx as f64, rz as f64, noise);
            return Vec3::new(rx as f32 + 0.5, h as f32 + 2.8, rz as f32 + 0.5);
        }
    }

    // 2. Fallback: outward spiral search from (0, 0) in step of 8 blocks
    for radius in (0..=1600).step_by(8) {
        for &(dx, dz) in &[
            (radius, 0),
            (-radius, 0),
            (0, radius),
            (0, -radius),
            (radius, radius),
            (-radius, radius),
            (radius, -radius),
            (-radius, -radius),
        ] {
            if is_safe_spawn_location(dx, dz, noise, seed) {
                let (_, h, _) = calculate_biome_and_height(dx as f64, dz as f64, noise);
                return Vec3::new(dx as f32 + 0.5, h as f32 + 2.8, dz as f32 + 0.5);
            }
        }
    }

    // 3. Absolute fallback: safe height above sea level
    let (_, h, _) = calculate_biome_and_height(0.0, 0.0, noise);
    Vec3::new(0.5, (h.max(SEA_LEVEL + 1) as f32) + 2.8, 0.5)
}

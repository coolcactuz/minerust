use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::block::{BlockFace, BlockType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum TextureId {
    GrassTop = 0,
    GrassSide = 1,
    Dirt = 2,
    Stone = 3,
    Cobblestone = 4,
    WoodSide = 5,
    WoodTop = 6,
    Leaves = 7,
    Planks = 8,
    Sand = 9,
    SandstoneSide = 10,
    SandstoneTop = 11,
    Water = 12,
    Snow = 13,
    SnowSide = 14,
    Bedrock = 15,
    CoalOre = 16,
    IronOre = 17,
    GoldOre = 18,
    DiamondOre = 19,
    CactusSide = 20,
    CactusTop = 21,
    Gravel = 22,
    Ice = 23,
    Glass = 24,
}

pub fn block_texture(block: BlockType, face: BlockFace) -> TextureId {
    match block {
        BlockType::Grass => match face {
            BlockFace::Top => TextureId::GrassTop,
            BlockFace::Bottom => TextureId::Dirt,
            _ => TextureId::GrassSide,
        },
        BlockType::Dirt => TextureId::Dirt,
        BlockType::Stone => TextureId::Stone,
        BlockType::Cobblestone => TextureId::Cobblestone,
        BlockType::Wood => match face {
            BlockFace::Top | BlockFace::Bottom => TextureId::WoodTop,
            _ => TextureId::WoodSide,
        },
        BlockType::Leaves => TextureId::Leaves,
        BlockType::Planks => TextureId::Planks,
        BlockType::Sand => TextureId::Sand,
        BlockType::Sandstone => match face {
            BlockFace::Top => TextureId::SandstoneTop,
            _ => TextureId::SandstoneSide,
        },
        BlockType::Water => TextureId::Water,
        BlockType::Snow => match face {
            BlockFace::Top => TextureId::Snow,
            BlockFace::Bottom => TextureId::Dirt,
            _ => TextureId::SnowSide,
        },
        BlockType::Bedrock => TextureId::Bedrock,
        BlockType::CoalOre => TextureId::CoalOre,
        BlockType::IronOre => TextureId::IronOre,
        BlockType::GoldOre => TextureId::GoldOre,
        BlockType::DiamondOre => TextureId::DiamondOre,
        BlockType::Cactus => match face {
            BlockFace::Top | BlockFace::Bottom => TextureId::CactusTop,
            _ => TextureId::CactusSide,
        },
        BlockType::Gravel => TextureId::Gravel,
        BlockType::Ice => TextureId::Ice,
        BlockType::Glass => TextureId::Glass,
        BlockType::Air => TextureId::Dirt,
    }
}

pub fn get_tile_uvs(texture_id: TextureId) -> [[f32; 2]; 4] {
    let id = texture_id as usize;
    let col = (id % 8) as f32;
    let row = (id / 8) as f32;

    let u_min = col / 8.0;
    let u_max = (col + 1.0) / 8.0;
    let v_min = row / 8.0;
    let v_max = (row + 1.0) / 8.0;

    [
        [u_max, v_min],
        [u_min, v_min],
        [u_min, v_max],
        [u_max, v_max],
    ]
}

/// Genera a runtime l'intero Texture Atlas 128x128 pixel in stile retro 16x16 pixel-art Minecraft
pub fn create_texture_atlas() -> Image {
    const ATLAS_SIZE: usize = 128;
    let mut data = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];

    // Helper per colorare i pixel nel tile (px, py in 0..16)
    let mut set_px = |tile_id: TextureId, px: usize, py: usize, rgba: [u8; 4]| {
        let id = tile_id as usize;
        let tile_x = id % 8;
        let tile_y = id / 8;
        let x = tile_x * 16 + px;
        let y = tile_y * 16 + py;
        let idx = (y * ATLAS_SIZE + x) * 4;
        data[idx..idx + 4].copy_from_slice(&rgba);
    };

    let p_hash = |x: usize, y: usize, salt: usize| -> usize {
        (x * 374761393 + y * 668265263 + salt * 31) ^ (x * 127 + y * 311)
    };

    // 1. Dirt
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 1) % 10;
            let col = match h {
                0..=2 => [108, 73, 47, 255],
                3..=7 => [134, 96, 67, 255],
                _ => [150, 108, 77, 255],
            };
            set_px(TextureId::Dirt, x, y, col);
        }
    }

    // 2. Grass Top
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 2) % 10;
            let col = match h {
                0..=2 => [76, 134, 39, 255],
                3..=7 => [95, 159, 53, 255],
                _ => [115, 178, 68, 255],
            };
            set_px(TextureId::GrassTop, x, y, col);
        }
    }

    // 3. Grass Side (Erba in alto con bavette, terra sotto)
    for x in 0..16 {
        for y in 0..16 {
            let overhang = 3 + (p_hash(x, 0, 3) % 3);
            if y < overhang {
                let h = p_hash(x, y, 2) % 3;
                let col = if h == 0 { [76, 134, 39, 255] } else { [95, 159, 53, 255] };
                set_px(TextureId::GrassSide, x, y, col);
            } else {
                let h = p_hash(x, y, 1) % 3;
                let col = if h == 0 { [108, 73, 47, 255] } else { [134, 96, 67, 255] };
                set_px(TextureId::GrassSide, x, y, col);
            }
        }
    }

    // 4. Stone
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 4) % 10;
            let col = match h {
                0..=2 => [105, 105, 105, 255],
                3..=7 => [125, 125, 125, 255],
                _ => [145, 145, 145, 255],
            };
            set_px(TextureId::Stone, x, y, col);
        }
    }

    // 5. Cobblestone (Pietrisco con giunzioni scure)
    for x in 0..16 {
        for y in 0..16 {
            let is_border = x == 0 || x == 8 || y == 0 || y == 5 || y == 11 || (y < 6 && x == 4) || (y >= 6 && y < 12 && x == 12);
            let col = if is_border {
                [65, 65, 65, 255]
            } else {
                let h = p_hash(x, y, 5) % 4;
                if h == 0 { [100, 100, 100, 255] } else { [135, 135, 135, 255] }
            };
            set_px(TextureId::Cobblestone, x, y, col);
        }
    }

    // 6. Wood Side (Corteccia con striature verticali)
    for x in 0..16 {
        for y in 0..16 {
            let h = (x + p_hash(x, y / 4, 6) % 2) % 4;
            let col = match h {
                0 => [82, 58, 33, 255],
                1 | 2 => [103, 77, 46, 255],
                _ => [122, 91, 55, 255],
            };
            set_px(TextureId::WoodSide, x, y, col);
        }
    }

    // 7. Wood Top (Anelli concentrici di crescita e bordo di corteccia)
    for x in 0..16 {
        for y in 0..16 {
            let dx = (x as i32 - 8).abs();
            let dy = (y as i32 - 8).abs();
            let dist = dx.max(dy);
            let col = if dist >= 7 {
                [82, 58, 33, 255] // Corteccia
            } else if dist == 4 || dist == 1 {
                [140, 105, 65, 255] // Anello scuro
            } else {
                [175, 138, 92, 255] // Legno chiaro
            };
            set_px(TextureId::WoodTop, x, y, col);
        }
    }

    // 8. Leaves (Foglie con trasparenza e variazioni verdi)
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 8) % 10;
            let col = match h {
                0 => [30, 80, 22, 255],
                1..=4 => [45, 115, 35, 255],
                5..=8 => [62, 145, 48, 255],
                _ => [75, 168, 58, 255],
            };
            set_px(TextureId::Leaves, x, y, col);
        }
    }

    // 9. Planks (Assi di legno orizzontali)
    for x in 0..16 {
        for y in 0..16 {
            let is_seam = y % 4 == 0 || (y < 4 && x == 7) || (y >= 4 && y < 8 && x == 14) || (y >= 8 && y < 12 && x == 4) || (y >= 12 && x == 10);
            let col = if is_seam {
                [130, 95, 50, 255]
            } else {
                let h = p_hash(x, y, 9) % 3;
                if h == 0 { [170, 130, 78, 255] } else { [188, 146, 92, 255] }
            };
            set_px(TextureId::Planks, x, y, col);
        }
    }

    // 10. Sand
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 10) % 10;
            let col = match h {
                0..=2 => [205, 195, 140, 255],
                3..=7 => [218, 210, 158, 255],
                _ => [230, 222, 175, 255],
            };
            set_px(TextureId::Sand, x, y, col);
        }
    }

    // 11. Sandstone Side
    for x in 0..16 {
        for y in 0..16 {
            let is_line = y == 0 || y == 4 || y == 12;
            let col = if is_line {
                [180, 165, 115, 255]
            } else {
                let h = p_hash(x, y, 11) % 3;
                if h == 0 { [205, 192, 140, 255] } else { [216, 204, 152, 255] }
            };
            set_px(TextureId::SandstoneSide, x, y, col);
        }
    }

    // 12. Sandstone Top
    for x in 0..16 {
        for y in 0..16 {
            let col = if x == 0 || x == 15 || y == 0 || y == 15 {
                [185, 170, 120, 255]
            } else {
                [216, 204, 152, 255]
            };
            set_px(TextureId::SandstoneTop, x, y, col);
        }
    }

    // 13. Water (Onde azzurre)
    for x in 0..16 {
        for y in 0..16 {
            let wave = (x + y * 2) % 6 == 0;
            let col = if wave {
                [65, 135, 240, 225]
            } else {
                [42, 100, 215, 225]
            };
            set_px(TextureId::Water, x, y, col);
        }
    }

    // 14. Snow
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 14) % 10;
            let col = match h {
                0..=2 => [225, 235, 245, 255],
                _ => [245, 248, 252, 255],
            };
            set_px(TextureId::Snow, x, y, col);
        }
    }

    // 15. Snow Side (Neve sopra, terra sotto)
    for x in 0..16 {
        for y in 0..16 {
            if y < 4 {
                set_px(TextureId::SnowSide, x, y, [245, 248, 252, 255]);
            } else {
                let h = p_hash(x, y, 1) % 3;
                let col = if h == 0 { [108, 73, 47, 255] } else { [134, 96, 67, 255] };
                set_px(TextureId::SnowSide, x, y, col);
            }
        }
    }

    // 16. Bedrock
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 16) % 10;
            let col = match h {
                0..=2 => [15, 15, 15, 255],
                3..=6 => [35, 35, 35, 255],
                _ => [60, 60, 60, 255],
            };
            set_px(TextureId::Bedrock, x, y, col);
        }
    }

    // Helper per disegnare minerali incastonati nella pietra
    let mut draw_ore = |ore_id: TextureId, fleck_color: [u8; 4], fleck_dark: [u8; 4]| {
        for x in 0..16 {
            for y in 0..16 {
                let h = p_hash(x, y, 4) % 10;
                let base = match h {
                    0..=2 => [105, 105, 105, 255],
                    3..=7 => [125, 125, 125, 255],
                    _ => [145, 145, 145, 255],
                };
                set_px(ore_id, x, y, base);
            }
        }
        // Pepite/cristalli sparsi
        let spots = [(3, 4), (4, 4), (3, 5), (10, 8), (11, 8), (10, 9), (6, 12), (7, 12), (13, 3), (13, 4)];
        for (sx, sy) in spots {
            set_px(ore_id, sx, sy, fleck_color);
            set_px(ore_id, (sx + 1) % 16, sy, fleck_dark);
        }
    };

    // 17. Coal Ore
    draw_ore(TextureId::CoalOre, [25, 25, 25, 255], [45, 45, 45, 255]);
    // 18. Iron Ore
    draw_ore(TextureId::IronOre, [210, 175, 145, 255], [175, 135, 105, 255]);
    // 19. Gold Ore
    draw_ore(TextureId::GoldOre, [250, 220, 60, 255], [215, 175, 35, 255]);
    // 20. Diamond Ore
    draw_ore(TextureId::DiamondOre, [75, 230, 235, 255], [45, 185, 195, 255]);

    // 21. Cactus Side
    for x in 0..16 {
        for y in 0..16 {
            let is_stripe = x % 4 == 0;
            let is_thorn = (x + y * 3) % 11 == 0;
            let col = if is_thorn {
                [35, 30, 20, 255]
            } else if is_stripe {
                [42, 120, 45, 255]
            } else {
                [55, 155, 60, 255]
            };
            set_px(TextureId::CactusSide, x, y, col);
        }
    }

    // 22. Cactus Top
    for x in 0..16 {
        for y in 0..16 {
            let dx = (x as i32 - 8).abs();
            let dy = (y as i32 - 8).abs();
            let dist = dx.max(dy);
            let col = if dist >= 7 { [42, 120, 45, 255] } else { [55, 155, 60, 255] };
            set_px(TextureId::CactusTop, x, y, col);
        }
    }

    // 23. Gravel
    for x in 0..16 {
        for y in 0..16 {
            let h = p_hash(x, y, 23) % 10;
            let col = match h {
                0..=2 => [95, 90, 90, 255],
                3..=7 => [120, 115, 115, 255],
                _ => [140, 135, 135, 255],
            };
            set_px(TextureId::Gravel, x, y, col);
        }
    }

    // 24. Ice (Ghiaccio azzurro traslucido con crepe bianche)
    for x in 0..16 {
        for y in 0..16 {
            let is_crack = (x + y) % 7 == 0 || (x == 10 && y > 3 && y < 12);
            let col = if is_crack {
                [230, 245, 255, 230]
            } else {
                [150, 205, 245, 210]
            };
            set_px(TextureId::Ice, x, y, col);
        }
    }

    // 25. Glass (Bordi bianchi e riflessi diagonali trasparenti)
    for x in 0..16 {
        for y in 0..16 {
            let is_border = x == 0 || x == 15 || y == 0 || y == 15;
            let is_glare = (x == y + 2 || x == y + 3) && x > 3 && x < 12;
            let col = if is_border {
                [245, 250, 255, 180]
            } else if is_glare {
                [255, 255, 255, 160]
            } else {
                [210, 235, 255, 45]
            };
            set_px(TextureId::Glass, x, y, col);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: ATLAS_SIZE as u32,
            height: ATLAS_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    // Pixel art nitido con filtro Nearest-Neighbor
    image.sampler = ImageSampler::nearest();
    image
}

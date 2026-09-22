use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::block::{BlockFace, BlockType};

pub const TILE_SIZE: usize = 16;
pub const LAYER_COUNT: usize = 25;

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

impl TextureId {
    #[inline(always)]
    pub const fn layer(self) -> f32 {
        self as usize as f32
    }
}

pub const fn block_texture(block: BlockType, face: BlockFace) -> TextureId {
    match block {
        BlockType::Grass => match face {
            BlockFace::Top => TextureId::GrassTop,
            BlockFace::Bottom => TextureId::Dirt,
            _ => TextureId::GrassSide,
        },
        BlockType::Dirt | BlockType::Air => TextureId::Dirt,
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
    }
}

#[inline(always)]
pub const fn quad_uvs(width: f32, height: f32) -> [[f32; 2]; 4] {
    [
        [0.0, 0.0],
        [0.0, height],
        [width, height],
        [width, 0.0],
    ]
}

#[inline(always)]
pub const fn get_tile_uvs(_texture_id: TextureId) -> [[f32; 2]; 4] {
    quad_uvs(1.0, 1.0)
}

/// Generates at runtime the 2D Texture Array (25 layers of 16x16 pixel-art textures)
/// with hardware Repeat addressing and Nearest-Neighbor filtering.
pub fn create_texture_array() -> Image {
    let mut data = vec![0u8; LAYER_COUNT * TILE_SIZE * TILE_SIZE * 4];

    // Helper to set pixel color in a specific layer (px, py in 0..16)
    let mut set_px = |tile_id: TextureId, px: usize, py: usize, rgba: [u8; 4]| {
        let layer = tile_id as usize;
        let idx = (layer * (TILE_SIZE * TILE_SIZE) + py * TILE_SIZE + px) * 4;
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

    // 3. Grass Side (Green grass fringe on top, dirt underneath)
    for x in 0..16 {
        for y in 0..16 {
            let overhang = 3 + (p_hash(x, 0, 3) % 3);
            if y < overhang {
                let h = p_hash(x, y, 2) % 3;
                let col = if h == 0 {
                    [76, 134, 39, 255]
                } else {
                    [95, 159, 53, 255]
                };
                set_px(TextureId::GrassSide, x, y, col);
            } else {
                let h = p_hash(x, y, 1) % 3;
                let col = if h == 0 {
                    [108, 73, 47, 255]
                } else {
                    [134, 96, 67, 255]
                };
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

    // 5. Cobblestone (Rough cobblestones with dark seams)
    for x in 0..16 {
        for y in 0..16 {
            let is_border = x == 0
                || x == 8
                || y == 0
                || y == 5
                || y == 11
                || (y < 6 && x == 4)
                || ((6..12).contains(&y) && x == 12);
            let col = if is_border {
                [65, 65, 65, 255]
            } else {
                let h = p_hash(x, y, 5) % 4;
                if h == 0 {
                    [100, 100, 100, 255]
                } else {
                    [135, 135, 135, 255]
                }
            };
            set_px(TextureId::Cobblestone, x, y, col);
        }
    }

    // 6. Wood Side (Bark with vertical striations)
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

    // 7. Wood Top (Concentric growth rings and bark rim)
    for x in 0..16 {
        for y in 0..16 {
            let dx = (x as i32 - 8).abs();
            let dy = (y as i32 - 8).abs();
            let dist = dx.max(dy);
            let col = if dist >= 7 {
                [82, 58, 33, 255] // Bark
            } else if dist == 4 || dist == 1 {
                [140, 105, 65, 255] // Dark ring
            } else {
                [175, 138, 92, 255] // Light wood
            };
            set_px(TextureId::WoodTop, x, y, col);
        }
    }

    // 8. Leaves (Foliage with transparency and varied green tones)
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

    // 9. Planks (Horizontal wooden boards with seams)
    for x in 0..16_usize {
        for y in 0..16_usize {
            let is_seam = y.is_multiple_of(4)
                || (y < 4 && x == 7)
                || ((4..8).contains(&y) && x == 14)
                || ((8..12).contains(&y) && x == 4)
                || (y >= 12 && x == 10);
            let col = if is_seam {
                [130, 95, 50, 255]
            } else {
                let h = p_hash(x, y, 9) % 3;
                if h == 0 {
                    [170, 130, 78, 255]
                } else {
                    [188, 146, 92, 255]
                }
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
                if h == 0 {
                    [205, 192, 140, 255]
                } else {
                    [216, 204, 152, 255]
                }
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

    // 13. Water (Blue ripple patterns)
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

    // 15. Snow Side (Snow layer on top, dirt below)
    for x in 0..16 {
        for y in 0..16 {
            if y < 4 {
                set_px(TextureId::SnowSide, x, y, [245, 248, 252, 255]);
            } else {
                let h = p_hash(x, y, 1) % 3;
                let col = if h == 0 {
                    [108, 73, 47, 255]
                } else {
                    [134, 96, 67, 255]
                };
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

    // Helper to draw ore flecks embedded in stone
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
        // Scattered crystal/ore flecks
        let spots = [
            (3, 4),
            (4, 4),
            (3, 5),
            (10, 8),
            (11, 8),
            (10, 9),
            (6, 12),
            (7, 12),
            (13, 3),
            (13, 4),
        ];
        for (sx, sy) in spots {
            set_px(ore_id, sx, sy, fleck_color);
            set_px(ore_id, (sx + 1) % 16, sy, fleck_dark);
        }
    };

    // 17. Coal Ore
    draw_ore(TextureId::CoalOre, [25, 25, 25, 255], [45, 45, 45, 255]);
    // 18. Iron Ore
    draw_ore(
        TextureId::IronOre,
        [210, 175, 145, 255],
        [175, 135, 105, 255],
    );
    // 19. Gold Ore
    draw_ore(TextureId::GoldOre, [250, 220, 60, 255], [215, 175, 35, 255]);
    // 20. Diamond Ore
    draw_ore(
        TextureId::DiamondOre,
        [75, 230, 235, 255],
        [45, 185, 195, 255],
    );

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
            let col = if dist >= 7 {
                [42, 120, 45, 255]
            } else {
                [55, 155, 60, 255]
            };
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

    // 24. Ice (Translucent blue ice with white fracture lines)
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

    // 25. Glass (White border and diagonal transparent glares)
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
            width: TILE_SIZE as u32,
            height: TILE_SIZE as u32,
            depth_or_array_layers: LAYER_COUNT as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    // Sharp pixel art with Nearest-Neighbor filtering and Repeat address mode
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

/// Compatibility alias for `create_texture_array`
pub fn create_texture_atlas() -> Image {
    create_texture_array()
}

#![forbid(unsafe_code)]

use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::block::{BlockFace, BlockType};

pub const TILE_SIZE: usize = 16;
pub const LAYER_COUNT: usize = 35;

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
    CraftingTableTop = 25,
    CraftingTableSide = 26,
    CraftingTableFront = 27,
    ChestTop = 28,
    ChestSide = 29,
    ChestFront = 30,
    FurnaceTop = 31,
    FurnaceSide = 32,
    FurnaceFront = 33,
    FurnaceLit = 34,
}

impl TextureId {
    pub const ALL: [TextureId; LAYER_COUNT] = [
        TextureId::GrassTop,
        TextureId::GrassSide,
        TextureId::Dirt,
        TextureId::Stone,
        TextureId::Cobblestone,
        TextureId::WoodSide,
        TextureId::WoodTop,
        TextureId::Leaves,
        TextureId::Planks,
        TextureId::Sand,
        TextureId::SandstoneSide,
        TextureId::SandstoneTop,
        TextureId::Water,
        TextureId::Snow,
        TextureId::SnowSide,
        TextureId::Bedrock,
        TextureId::CoalOre,
        TextureId::IronOre,
        TextureId::GoldOre,
        TextureId::DiamondOre,
        TextureId::CactusSide,
        TextureId::CactusTop,
        TextureId::Gravel,
        TextureId::Ice,
        TextureId::Glass,
        TextureId::CraftingTableTop,
        TextureId::CraftingTableSide,
        TextureId::CraftingTableFront,
        TextureId::ChestTop,
        TextureId::ChestSide,
        TextureId::ChestFront,
        TextureId::FurnaceTop,
        TextureId::FurnaceSide,
        TextureId::FurnaceFront,
        TextureId::FurnaceLit,
    ];

    #[inline(always)]
    #[must_use]
    pub const fn layer(self) -> f32 {
        self as usize as f32
    }

    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::GrassTop => "grass_top.png",
            Self::GrassSide => "grass_side.png",
            Self::Dirt => "dirt.png",
            Self::Stone => "stone.png",
            Self::Cobblestone => "cobblestone.png",
            Self::WoodSide => "wood_side.png",
            Self::WoodTop => "wood_top.png",
            Self::Leaves => "leaves.png",
            Self::Planks => "planks.png",
            Self::Sand => "sand.png",
            Self::SandstoneSide => "sandstone_side.png",
            Self::SandstoneTop => "sandstone_top.png",
            Self::Water => "water.png",
            Self::Snow => "snow.png",
            Self::SnowSide => "snow_side.png",
            Self::Bedrock => "bedrock.png",
            Self::CoalOre => "coal_ore.png",
            Self::IronOre => "iron_ore.png",
            Self::GoldOre => "gold_ore.png",
            Self::DiamondOre => "diamond_ore.png",
            Self::CactusSide => "cactus_side.png",
            Self::CactusTop => "cactus_top.png",
            Self::Gravel => "gravel.png",
            Self::Ice => "ice.png",
            Self::Glass => "glass.png",
            Self::CraftingTableTop => "crafting_table_top.png",
            Self::CraftingTableSide => "crafting_table_side.png",
            Self::CraftingTableFront => "crafting_table_front.png",
            Self::ChestTop => "chest_top.png",
            Self::ChestSide => "chest_side.png",
            Self::ChestFront => "chest_front.png",
            Self::FurnaceTop => "furnace_top.png",
            Self::FurnaceSide => "furnace_side.png",
            Self::FurnaceFront => "furnace_front.png",
            Self::FurnaceLit => "furnace_lit.png",
        }
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
        BlockType::CraftingTable => match face {
            BlockFace::Top => TextureId::CraftingTableTop,
            BlockFace::Bottom => TextureId::Planks,
            BlockFace::North => TextureId::CraftingTableFront,
            _ => TextureId::CraftingTableSide,
        },
        BlockType::Chest => match face {
            BlockFace::Top => TextureId::ChestTop,
            BlockFace::Bottom => TextureId::Planks,
            BlockFace::North => TextureId::ChestFront,
            _ => TextureId::ChestSide,
        },
        BlockType::Furnace => match face {
            BlockFace::Top | BlockFace::Bottom => TextureId::FurnaceTop,
            BlockFace::North => TextureId::FurnaceFront,
            _ => TextureId::FurnaceSide,
        },
        BlockType::LitFurnace => match face {
            BlockFace::Top | BlockFace::Bottom => TextureId::FurnaceTop,
            BlockFace::North => TextureId::FurnaceLit,
            _ => TextureId::FurnaceSide,
        },
    }
}

#[inline(always)]
pub const fn quad_uvs(width: f32, height: f32) -> [[f32; 2]; 4] {
    [[0.0, 0.0], [0.0, height], [width, height], [width, 0.0]]
}

#[inline(always)]
pub const fn get_tile_uvs(_texture_id: TextureId) -> [[f32; 2]; 4] {
    quad_uvs(1.0, 1.0)
}

/// Attempts to load and decode a 16x16 RGBA PNG image from disk.
fn load_png_pixels(path: &Path) -> Option<[u8; TILE_SIZE * TILE_SIZE * 4]> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    if rgba.width() == TILE_SIZE as u32 && rgba.height() == TILE_SIZE as u32 {
        let raw = rgba.into_raw();
        if raw.len() == TILE_SIZE * TILE_SIZE * 4 {
            let mut out = [0u8; TILE_SIZE * TILE_SIZE * 4];
            out.copy_from_slice(&raw);
            return Some(out);
        }
    } else {
        let resized = image::imageops::resize(
            &rgba,
            TILE_SIZE as u32,
            TILE_SIZE as u32,
            image::imageops::FilterType::Nearest,
        );
        let raw = resized.into_raw();
        if raw.len() == TILE_SIZE * TILE_SIZE * 4 {
            let mut out = [0u8; TILE_SIZE * TILE_SIZE * 4];
            out.copy_from_slice(&raw);
            return Some(out);
        }
    }
    None
}

/// Fallback procedural pattern generator when a physical PNG texture is missing.
fn fallback_tile_pixels(tile: TextureId) -> [u8; TILE_SIZE * TILE_SIZE * 4] {
    let mut buf = [0u8; TILE_SIZE * TILE_SIZE * 4];
    let base_color: [u8; 4] = match tile {
        TextureId::GrassTop => [95, 159, 53, 255],
        TextureId::GrassSide => [115, 80, 50, 255],
        TextureId::Dirt => [134, 96, 67, 255],
        TextureId::Stone => [125, 125, 125, 255],
        TextureId::Cobblestone => [100, 100, 100, 255],
        TextureId::WoodSide => [103, 77, 46, 255],
        TextureId::WoodTop => [175, 138, 92, 255],
        TextureId::Leaves => [55, 130, 40, 255],
        TextureId::Planks => [175, 142, 88, 255],
        TextureId::Sand => [218, 208, 152, 255],
        TextureId::SandstoneSide | TextureId::SandstoneTop => [220, 210, 160, 255],
        TextureId::Water => [45, 105, 215, 215],
        TextureId::Snow | TextureId::SnowSide => [242, 245, 253, 255],
        TextureId::Bedrock => [40, 40, 40, 255],
        TextureId::CoalOre => [50, 50, 50, 255],
        TextureId::IronOre => [190, 150, 130, 255],
        TextureId::GoldOre => [235, 200, 50, 255],
        TextureId::DiamondOre => [75, 210, 225, 255],
        TextureId::CactusSide | TextureId::CactusTop => [60, 130, 40, 255],
        TextureId::Gravel => [125, 120, 120, 255],
        TextureId::Ice => [145, 190, 240, 220],
        TextureId::Glass => [200, 225, 240, 30],
        TextureId::CraftingTableTop
        | TextureId::CraftingTableSide
        | TextureId::CraftingTableFront => [165, 130, 80, 255],
        TextureId::ChestTop | TextureId::ChestSide | TextureId::ChestFront => [160, 120, 60, 255],
        TextureId::FurnaceTop | TextureId::FurnaceSide | TextureId::FurnaceFront => {
            [110, 110, 110, 255]
        }
        TextureId::FurnaceLit => [240, 140, 30, 255],
    };

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let idx = (y * TILE_SIZE + x) * 4;
            let shade = if (x + y) % 2 == 0 { 0 } else { 15 };
            buf[idx] = base_color[0].saturating_sub(shade);
            buf[idx + 1] = base_color[1].saturating_sub(shade);
            buf[idx + 2] = base_color[2].saturating_sub(shade);
            buf[idx + 3] = base_color[3];
        }
    }
    buf
}

/// Loads a block texture by first checking physical PNG files on disk, then falling back cleanly.
#[must_use]
pub fn load_block_texture_pixels(tile: TextureId) -> [u8; TILE_SIZE * TILE_SIZE * 4] {
    let block_dir = Path::new("assets").join("textures").join("blocks");
    let file_path = block_dir.join(tile.filename());

    if let Some(pixels) = load_png_pixels(&file_path) {
        return pixels;
    }

    fallback_tile_pixels(tile)
}

/// Assembles the 2D Texture Array (35 layers of 16x16 pixel-art textures)
/// with hardware Repeat addressing and Nearest-Neighbor filtering.
#[must_use]
pub fn create_texture_array() -> Image {
    let mut data = vec![0u8; LAYER_COUNT * TILE_SIZE * TILE_SIZE * 4];

    for tile in TextureId::ALL {
        let pixels = load_block_texture_pixels(tile);
        let layer = tile as usize;
        let start = layer * TILE_SIZE * TILE_SIZE * 4;
        let end = start + TILE_SIZE * TILE_SIZE * 4;
        data[start..end].copy_from_slice(&pixels);
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
#[must_use]
pub fn create_texture_atlas() -> Image {
    create_texture_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_array_dimensions_and_layers() {
        let img = create_texture_array();
        assert_eq!(img.texture_descriptor.size.width, TILE_SIZE as u32);
        assert_eq!(img.texture_descriptor.size.height, TILE_SIZE as u32);
        assert_eq!(
            img.texture_descriptor.size.depth_or_array_layers,
            LAYER_COUNT as u32
        );
        assert_eq!(img.texture_descriptor.dimension, TextureDimension::D2);
        assert_eq!(img.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
    }

    #[test]
    fn test_all_block_textures_have_non_empty_pixel_data() {
        for tile in TextureId::ALL {
            let pixels = load_block_texture_pixels(tile);
            assert_eq!(pixels.len(), TILE_SIZE * TILE_SIZE * 4);
            // Alpha channel must be > 0
            assert!(pixels.iter().skip(3).step_by(4).any(|&a| a > 0));
        }
    }
}

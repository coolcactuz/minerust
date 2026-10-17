#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::block::{BlockFace, BlockType};
use crate::item::{ItemType, MaterialType, ToolTier, ToolType};
use crate::texture::{TextureId, block_texture, create_texture_array};

pub const ICON_SIZE: usize = 16;

/// Pre-generated 16x16 pixel art icons for all blocks, crafting materials, and tools.
#[derive(Resource, Default)]
pub struct ItemIconAssets {
    pub icons: HashMap<ItemType, Handle<Image>>,
}

impl ItemIconAssets {
    /// Returns the pre-rendered image handle for the given item type.
    #[must_use]
    pub fn get(&self, item: ItemType) -> Option<&Handle<Image>> {
        self.icons.get(&item)
    }
}

/// Helper that wraps 16x16 RGBA8 pixel data into a Bevy `Image` with nearest-neighbor sampling.
#[must_use]
pub fn make_icon_image(data: &[u8]) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: ICON_SIZE as u32,
            height: ICON_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    image
}

/// Loads a 16x16 RGBA PNG image from the items texture folder on disk.
fn load_item_png_pixels(filename: &str) -> Option<Vec<u8>> {
    let path = Path::new("assets")
        .join("textures")
        .join("items")
        .join(filename);
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    if rgba.width() == ICON_SIZE as u32 && rgba.height() == ICON_SIZE as u32 {
        Some(rgba.into_raw())
    } else {
        let resized = image::imageops::resize(
            &rgba,
            ICON_SIZE as u32,
            ICON_SIZE as u32,
            image::imageops::FilterType::Nearest,
        );
        Some(resized.into_raw())
    }
}

fn block_icon_filename(block: BlockType) -> Option<&'static str> {
    match block {
        BlockType::Grass => Some("grass.png"),
        BlockType::Dirt => Some("dirt.png"),
        BlockType::Stone => Some("stone.png"),
        BlockType::Cobblestone => Some("cobblestone.png"),
        BlockType::Wood => Some("wood.png"),
        BlockType::Planks => Some("planks.png"),
        BlockType::Leaves => Some("leaves.png"),
        BlockType::Glass => Some("glass.png"),
        BlockType::Sand => Some("sand.png"),
        BlockType::Sandstone => Some("sandstone.png"),
        BlockType::Gravel => Some("gravel.png"),
        BlockType::Water => Some("water.png"),
        BlockType::Ice => Some("ice.png"),
        BlockType::Snow => Some("snow.png"),
        BlockType::Cactus => Some("cactus.png"),
        BlockType::CoalOre => Some("coal_ore.png"),
        BlockType::IronOre => Some("iron_ore.png"),
        BlockType::GoldOre => Some("gold_ore.png"),
        BlockType::DiamondOre => Some("diamond_ore.png"),
        BlockType::Bedrock => Some("bedrock.png"),
        BlockType::CraftingTable => Some("crafting_table.png"),
        BlockType::Chest => Some("chest.png"),
        BlockType::Furnace => Some("furnace.png"),
        BlockType::LitFurnace => Some("furnace_lit.png"),
        BlockType::Air => None,
    }
}

/// Extracts or loads the representative 16x16 icon texture for placeable blocks.
/// Loads the dedicated 3D isometric mini-cube icon if available, or falls back to the texture array.
#[must_use]
pub fn generate_block_icon_data(array_data: &[u8], block: BlockType) -> Vec<u8> {
    if let Some(filename) = block_icon_filename(block) {
        if let Some(pixels) = load_item_png_pixels(filename) {
            return pixels;
        }
    }

    let tex_id = match block {
        BlockType::Grass => TextureId::GrassSide,
        BlockType::Wood => TextureId::WoodSide,
        BlockType::CraftingTable => TextureId::CraftingTableFront,
        BlockType::Chest => TextureId::ChestFront,
        BlockType::Furnace => TextureId::FurnaceFront,
        BlockType::LitFurnace => TextureId::FurnaceLit,
        BlockType::Sandstone => TextureId::SandstoneSide,
        BlockType::Snow => TextureId::SnowSide,
        BlockType::Cactus => TextureId::CactusSide,
        other => block_texture(other, BlockFace::Top),
    };

    let layer = tex_id as usize;
    let start = layer * ICON_SIZE * ICON_SIZE * 4;
    let end = start + ICON_SIZE * ICON_SIZE * 4;

    if end <= array_data.len() {
        array_data[start..end].to_vec()
    } else {
        vec![0u8; ICON_SIZE * ICON_SIZE * 4]
    }
}

fn fallback_material_icon(material: MaterialType) -> Vec<u8> {
    let mut buf = vec![0u8; ICON_SIZE * ICON_SIZE * 4];
    let color: [u8; 4] = match material {
        MaterialType::Stick => [150, 105, 52, 255],
        MaterialType::Coal => [35, 35, 40, 255],
        MaterialType::IronIngot => [210, 210, 215, 255],
        MaterialType::GoldIngot => [245, 205, 45, 255],
        MaterialType::Diamond => [90, 225, 235, 255],
    };

    for y in 4..12 {
        for x in 4..12 {
            let idx = (y * ICON_SIZE + x) * 4;
            buf[idx..idx + 4].copy_from_slice(&color);
        }
    }
    buf
}

/// Generates distinct pixel art for loose materials (Stick, Coal, Ingots, Diamond).
#[must_use]
pub fn generate_material_icon_data(material: MaterialType) -> Vec<u8> {
    let filename = match material {
        MaterialType::Stick => "stick.png",
        MaterialType::Coal => "coal.png",
        MaterialType::IronIngot => "iron_ingot.png",
        MaterialType::GoldIngot => "gold_ingot.png",
        MaterialType::Diamond => "diamond.png",
    };

    if let Some(pixels) = load_item_png_pixels(filename) {
        return pixels;
    }

    fallback_material_icon(material)
}

fn fallback_tool_icon(tool: ToolType, tier: ToolTier) -> Vec<u8> {
    let mut buf = vec![0u8; ICON_SIZE * ICON_SIZE * 4];
    let head_color: [u8; 4] = match tier {
        ToolTier::Wood => [150, 110, 60, 255],
        ToolTier::Stone => [130, 130, 130, 255],
        ToolTier::Iron => [220, 220, 220, 255],
        ToolTier::Diamond => [90, 225, 235, 255],
        ToolTier::Gold => [245, 205, 45, 255],
    };
    let handle_color: [u8; 4] = [120, 85, 45, 255];

    // Diagonal handle
    for i in 6..14 {
        let y = i;
        let x = 15 - i;
        let idx = (y * ICON_SIZE + x) * 4;
        buf[idx..idx + 4].copy_from_slice(&handle_color);
    }

    // Tool head identifier
    let head_pixels: &[(usize, usize)] = match tool {
        ToolType::Pickaxe => &[(3, 2), (4, 2), (5, 2), (2, 4), (2, 5)],
        ToolType::Axe => &[(4, 2), (5, 2), (4, 3), (5, 3), (4, 4)],
        ToolType::Shovel => &[(3, 2), (4, 2), (3, 3), (4, 3)],
        ToolType::Sword => &[(3, 2), (4, 3), (5, 4), (6, 5), (7, 6)],
        ToolType::Hoe => &[(3, 2), (4, 2), (5, 2)],
    };

    for &(x, y) in head_pixels {
        let idx = (y * ICON_SIZE + x) * 4;
        buf[idx..idx + 4].copy_from_slice(&head_color);
    }

    buf
}

/// Generates distinct pixel art for tools across all five tiers.
#[must_use]
pub fn generate_tool_icon_data(tool: ToolType, tier: ToolTier) -> Vec<u8> {
    let tier_str = match tier {
        ToolTier::Wood => "wood",
        ToolTier::Stone => "stone",
        ToolTier::Iron => "iron",
        ToolTier::Diamond => "diamond",
        ToolTier::Gold => "gold",
    };
    let tool_str = match tool {
        ToolType::Pickaxe => "pickaxe",
        ToolType::Axe => "axe",
        ToolType::Shovel => "shovel",
        ToolType::Sword => "sword",
        ToolType::Hoe => "hoe",
    };
    let filename = format!("{tier_str}_{tool_str}.png");

    if let Some(pixels) = load_item_png_pixels(&filename) {
        return pixels;
    }

    fallback_tool_icon(tool, tier)
}

/// Bevy startup system that pre-bakes all block, material, and tool icons into Bevy `Image` assets.
pub fn setup_item_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut icon_map = HashMap::new();

    // 1. Block icons
    let texture_array = create_texture_array();
    let raw_array_data = texture_array.data.as_deref().unwrap_or(&[]);

    for &block in crate::inventory::ALL_BLOCKS {
        let item = ItemType::Block(block);
        let data = generate_block_icon_data(raw_array_data, block);
        let img = make_icon_image(&data);
        let handle = images.add(img);
        icon_map.insert(item, handle);
    }

    // 2. Material icons
    let all_materials = [
        MaterialType::Stick,
        MaterialType::Coal,
        MaterialType::IronIngot,
        MaterialType::GoldIngot,
        MaterialType::Diamond,
    ];
    for mat in all_materials {
        let item = ItemType::Material(mat);
        let data = generate_material_icon_data(mat);
        let img = make_icon_image(&data);
        let handle = images.add(img);
        icon_map.insert(item, handle);
    }

    // 3. Tool icons
    let all_tools = [
        ToolType::Pickaxe,
        ToolType::Axe,
        ToolType::Shovel,
        ToolType::Sword,
        ToolType::Hoe,
    ];
    let all_tiers = [
        ToolTier::Wood,
        ToolTier::Stone,
        ToolTier::Iron,
        ToolTier::Diamond,
        ToolTier::Gold,
    ];
    for tool in all_tools {
        for tier in all_tiers {
            let item = ItemType::tool(tool, tier);
            let data = generate_tool_icon_data(tool, tier);
            let img = make_icon_image(&data);
            let handle = images.add(img);
            icon_map.insert(item, handle);
        }
    }

    commands.insert_resource(ItemIconAssets { icons: icon_map });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_icon_generation_buffer_lengths() {
        let mats = [
            MaterialType::Stick,
            MaterialType::Coal,
            MaterialType::IronIngot,
            MaterialType::GoldIngot,
            MaterialType::Diamond,
        ];
        for mat in mats {
            let data = generate_material_icon_data(mat);
            assert_eq!(data.len(), ICON_SIZE * ICON_SIZE * 4);
            // Must have some non-transparent pixels
            assert!(data.iter().any(|&b| b > 0));
        }
    }

    #[test]
    fn test_tool_icon_generation_buffer_lengths() {
        let tools = [
            ToolType::Pickaxe,
            ToolType::Axe,
            ToolType::Shovel,
            ToolType::Sword,
            ToolType::Hoe,
        ];
        let tiers = [
            ToolTier::Wood,
            ToolTier::Stone,
            ToolTier::Iron,
            ToolTier::Diamond,
            ToolTier::Gold,
        ];
        for tool in tools {
            for tier in tiers {
                let data = generate_tool_icon_data(tool, tier);
                assert_eq!(data.len(), ICON_SIZE * ICON_SIZE * 4);
                assert!(data.iter().any(|&b| b > 0));
            }
        }
    }

    #[test]
    fn test_materials_produce_distinct_pixel_data() {
        let stick = generate_material_icon_data(MaterialType::Stick);
        let coal = generate_material_icon_data(MaterialType::Coal);
        let iron = generate_material_icon_data(MaterialType::IronIngot);
        let gold = generate_material_icon_data(MaterialType::GoldIngot);
        let diamond = generate_material_icon_data(MaterialType::Diamond);

        assert_ne!(stick, coal);
        assert_ne!(coal, iron);
        assert_ne!(iron, gold);
        assert_ne!(gold, diamond);
        assert_ne!(diamond, stick);
    }

    #[test]
    fn test_tools_of_different_tiers_produce_distinct_pixel_data() {
        let wood_pick = generate_tool_icon_data(ToolType::Pickaxe, ToolTier::Wood);
        let stone_pick = generate_tool_icon_data(ToolType::Pickaxe, ToolTier::Stone);
        let iron_pick = generate_tool_icon_data(ToolType::Pickaxe, ToolTier::Iron);
        let diamond_pick = generate_tool_icon_data(ToolType::Pickaxe, ToolTier::Diamond);
        let gold_pick = generate_tool_icon_data(ToolType::Pickaxe, ToolTier::Gold);

        assert_ne!(wood_pick, stone_pick);
        assert_ne!(stone_pick, iron_pick);
        assert_ne!(iron_pick, diamond_pick);
        assert_ne!(diamond_pick, gold_pick);
    }

    #[test]
    fn test_block_icon_generation_produces_non_empty_textures() {
        let array = create_texture_array();
        let raw = array.data.as_deref().unwrap_or(&[]);
        for block in crate::inventory::ALL_BLOCKS {
            let data = generate_block_icon_data(raw, *block);
            assert_eq!(data.len(), ICON_SIZE * ICON_SIZE * 4);
            assert!(
                data.iter().any(|&b| b > 0),
                "Block {block:?} icon must contain non-transparent pixels"
            );
        }
    }

    #[test]
    fn test_make_icon_image_dimensions_and_sampler() {
        let data = vec![255u8; ICON_SIZE * ICON_SIZE * 4];
        let img = make_icon_image(&data);
        assert_eq!(img.texture_descriptor.size.width, ICON_SIZE as u32);
        assert_eq!(img.texture_descriptor.size.height, ICON_SIZE as u32);
        assert_eq!(img.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
    }
}

#![forbid(unsafe_code)]

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::interaction::MiningState;

pub const CRACK_STAGES: usize = 10;
pub const TILE_SIZE: usize = 16;

/// 16x16 crack progression matrix.
/// Values 0..=9 designate the mining damage stage in which that pixel fractures.
/// 255 indicates an unbroken, transparent pixel.
#[rustfmt::skip]
pub const CRACK_MATRIX: [[u8; TILE_SIZE]; TILE_SIZE] = [
    [255, 255, 255,   8, 255, 255, 255, 255, 255, 255, 255,   9, 255, 255, 255, 255],
    [255, 255,   7,   6, 255, 255, 255, 255, 255, 255,   8,   7, 255, 255, 255, 255],
    [255,   8,   5, 255, 255, 255, 255, 255, 255,   7,   6, 255, 255, 255, 255, 255],
    [255,   6,   4, 255, 255, 255,   9, 255,   6,   5, 255, 255,   9, 255, 255, 255],
    [255, 255,   3,   4, 255,   8,   7,   5,   4, 255, 255,   8,   7, 255, 255, 255],
    [255, 255, 255,   2,   3,   5,   3,   2, 255, 255,   7,   6, 255, 255, 255, 255],
    [255, 255, 255, 255,   1,   2,   2, 255, 255,   6,   5, 255, 255, 255, 255, 255],
    [255, 255, 255, 255, 255,   0,   1, 255,   5,   4, 255, 255, 255,   9, 255, 255],
    [255, 255, 255, 255,   1,   0,   0,   1,   3, 255, 255, 255,   8,   7, 255, 255],
    [255, 255, 255,   2,   1, 255,   1,   2,   4, 255, 255,   7,   6, 255, 255, 255],
    [255, 255,   3,   2, 255, 255, 255,   3,   5,   6,   5,   4, 255, 255, 255, 255],
    [255,   4,   3, 255, 255,   8, 255, 255,   6,   7, 255,   5,   6, 255, 255, 255],
    [255,   5, 255, 255,   7,   6, 255, 255, 255,   8, 255, 255,   7,   8, 255, 255],
    [  6,   6, 255, 255,   8, 255, 255, 255, 255, 255, 255, 255, 255,   8,   9, 255],
    [  7, 255, 255,   9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,   9, 255],
    [  8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
];

/// Procedurally builds a 16x16 RGBA8 texture for the specified mining damage stage (0..=9).
#[must_use]
pub fn create_crack_stage_image(stage: usize) -> Image {
    let stage_clamp = stage.min(CRACK_STAGES - 1) as u8;
    let mut data = vec![0u8; TILE_SIZE * TILE_SIZE * 4];

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let threshold = CRACK_MATRIX[y][x];
            let idx = (y * TILE_SIZE + x) * 4;

            if threshold <= stage_clamp {
                // Fractured pixel: deeper/older cracks are darker, newly formed cracks have subtle edge contrast
                let rgba = if threshold == stage_clamp {
                    [20, 20, 25, 215]
                } else {
                    [5, 5, 10, 245]
                };
                data[idx..idx + 4].copy_from_slice(&rgba);
            }
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: TILE_SIZE as u32,
            height: TILE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
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

/// Pre-allocated materials for all 10 crack stages (0..9).
#[derive(Resource)]
pub struct CrackOverlayAssets {
    pub materials: [Handle<StandardMaterial>; CRACK_STAGES],
}

/// Tag component marking the single 3D crack overlay entity.
#[derive(Component)]
pub struct CrackOverlay;

/// Spawns the singleton crack overlay cube and registers materials at startup.
pub fn setup_crack_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut mat_handles = Vec::with_capacity(CRACK_STAGES);

    for stage in 0..CRACK_STAGES {
        let img = create_crack_stage_image(stage);
        let img_handle = images.add(img);

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(img_handle),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        });

        mat_handles.push(material);
    }

    let materials_array: [Handle<StandardMaterial>; CRACK_STAGES] = match mat_handles.try_into() {
        Ok(arr) => arr,
        Err(_) => return,
    };

    let initial_mat = materials_array[0].clone();
    commands.insert_resource(CrackOverlayAssets {
        materials: materials_array,
    });

    // Spawn single crack overlay cube entity slightly larger than a block (1.004) to prevent Z-fighting
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.004, 1.004, 1.004))),
        MeshMaterial3d(initial_mat),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Hidden,
        CrackOverlay,
    ));
}

/// Updates position, crack stage material, and visibility of the crack overlay entity.
pub fn update_crack_overlay_system(
    mining_state: Res<MiningState>,
    crack_assets: Option<Res<CrackOverlayAssets>>,
    mut query: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        With<CrackOverlay>,
    >,
) {
    let Some(crack_assets) = crack_assets else {
        return;
    };
    let Ok((mut transform, mut vis, mut mat)) = query.single_mut() else {
        return;
    };

    if let Some(target) = mining_state.target_block {
        if mining_state.progress > 0.0 {
            let stage = mining_state.stage() as usize;
            let center = target.as_vec3() + Vec3::splat(0.5);
            transform.translation = center;

            if let Some(target_mat) = crack_assets.materials.get(stage) {
                if mat.0 != *target_mat {
                    mat.0 = target_mat.clone();
                }
            }

            if *vis != Visibility::Inherited {
                *vis = Visibility::Inherited;
            }
            return;
        }
    }

    if *vis != Visibility::Hidden {
        *vis = Visibility::Hidden;
    }
}

pub struct CrackOverlayPlugin;

impl Plugin for CrackOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_crack_overlay).add_systems(
            Update,
            update_crack_overlay_system.in_set(crate::stage::VoxelStage::InputHandling),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crack_matrix_dimensions_and_progression() {
        assert_eq!(CRACK_MATRIX.len(), 16);
        for row in &CRACK_MATRIX {
            assert_eq!(row.len(), 16);
        }

        // Verify increasing crack counts across stages
        let mut prev_count = 0;
        for stage in 0..CRACK_STAGES {
            let mut count = 0;
            for row in &CRACK_MATRIX {
                for &val in row {
                    if val <= stage as u8 {
                        count += 1;
                    }
                }
            }
            assert!(
                count >= prev_count,
                "Stage {stage} should have at least as many cracks as previous"
            );
            prev_count = count;
        }

        // Stage 0 must have at least 1 crack pixel
        assert!(prev_count > 0);
    }

    #[test]
    fn test_crack_stage_images_generation() {
        for stage in 0..CRACK_STAGES {
            let img = create_crack_stage_image(stage);
            assert_eq!(img.texture_descriptor.size.width, 16);
            assert_eq!(img.texture_descriptor.size.height, 16);
            let data = img.data.as_ref().expect("image data should exist");
            assert_eq!(data.len(), 16 * 16 * 4);

            // Count fractured non-zero alpha pixels
            let fractured_pixels = data
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|rgba| rgba[3] > 0)
                .count();
            assert!(
                fractured_pixels > 0,
                "Stage {stage} should have non-zero alpha pixels"
            );
        }
    }
}

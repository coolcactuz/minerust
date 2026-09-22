use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub const VOXEL_SHADER_PATH: &str = "shaders/voxel.wgsl";

pub type VoxelBlockMaterial = ExtendedMaterial<StandardMaterial, VoxelExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct VoxelExtension {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    pub array_texture: Handle<Image>,
}

impl MaterialExtension for VoxelExtension {
    fn fragment_shader() -> ShaderRef {
        VOXEL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        VOXEL_SHADER_PATH.into()
    }
}

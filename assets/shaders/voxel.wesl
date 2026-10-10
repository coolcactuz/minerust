import bevy_pbr::{
    render::{
        pbr_functions::alpha_discard,
        pbr_fragment::pbr_input_from_standard_material,
    },
};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@if(!PREPASS_PIPELINE)
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var voxel_texture: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var voxel_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var uv = vec2<f32>(0.0, 0.0);
@if(VERTEX_UVS_A)
    uv = in.uv;

    var layer = 0;
    var shade = 1.0;
@if(VERTEX_UVS_B) {
    layer = i32(round(in.uv_b.x));
    shade = in.uv_b.y;
}

    let tex_color = textureSample(voxel_texture, voxel_sampler, uv, layer);

@if(VERTEX_COLORS)
    pbr_input.material.base_color = tex_color * in.color;
@if(!VERTEX_COLORS)
    pbr_input.material.base_color = tex_color * vec4<f32>(shade, shade, shade, 1.0);

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

@if(PREPASS_PIPELINE)
    let out = deferred_output(in, pbr_input);
@if(!PREPASS_PIPELINE)
    var out: FragmentOutput;
@if(!PREPASS_PIPELINE)
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
@if(!PREPASS_PIPELINE)
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}

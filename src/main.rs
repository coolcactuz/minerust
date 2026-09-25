#![forbid(unsafe_code)]

use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::window::WindowResolution;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use minerust::benchmark::BenchmarkConfig;
use minerust::camera::{CameraPlugin, FpsCamera};
use minerust::fluid::FluidPlugin;
use minerust::interaction::InteractionPlugin;
use minerust::inventory::InventoryPlugin;
use minerust::menu::{
    GraphicsSettings, MenuPlugin, MenuScreen, MenuState, ProfilerState, SeedInputState,
};
use minerust::physics::{PhysicsPlugin, PlayerPhysics};
use minerust::profile::ProfilePlugin;
use minerust::save::SavePlugin;
use minerust::stage::VoxelStage;
use minerust::texture;
use minerust::voxel_material::{VoxelBlockMaterial, VoxelExtension};
use minerust::world::{WorldGrid, WorldPlugin, WorldSeed};

/// Structured command-line options parsed cleanly at startup.
#[derive(Default, Debug, PartialEq)]
struct CliOptions {
    seed: Option<WorldSeed>,
    profile_mode: bool,
    quickstart: bool,
    view_distance: Option<i32>,
}

impl CliOptions {
    fn parse_from_args<I: IntoIterator<Item = String>>(args: I) -> Result<Self, String> {
        let mut opts = Self::default();
        let args: Vec<String> = args.into_iter().collect();
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "-s" | "--seed" => {
                    i += 1;
                    if i < args.len() {
                        opts.seed = Some(WorldSeed::from_seed_str(&args[i]));
                    }
                }
                "-p" | "--profile" => {
                    opts.profile_mode = true;
                }
                "-q" | "--quickstart" => {
                    opts.quickstart = true;
                }
                "--view-distance" => {
                    i += 1;
                    if i < args.len() {
                        if let Ok(vd) = args[i].parse::<i32>() {
                            opts.view_distance = Some(vd.clamp(2, 64));
                        }
                    }
                }
                "-h" | "--help" => {
                    return Err(Self::help_message());
                }
                _ => {}
            }
            i += 1;
        }

        Ok(opts)
    }

    fn help_message() -> String {
        "MineRust - A High-Performance Voxel Sandbox in Rust\n\
        Usage: minerust [OPTIONS]\n\n\
        Options:\n  \
          -s, --seed <SEED>              Set world generation seed (string or integer)\n  \
          -p, --profile                  Enable Real-time Performance Profiler HUD (F3)\n  \
          -q, --quickstart               Start directly in-game bypassing the main menu\n      \
              --view-distance <chunks>   Render distance (2 to 64 chunks)\n  \
          -h, --help                     Print help information"
            .to_string()
    }
}

fn main() {
    let opts = match CliOptions::parse_from_args(std::env::args()) {
        Ok(opts) => opts,
        Err(help) => {
            println!("{help}");
            return;
        }
    };

    let seed = opts.seed.unwrap_or_else(WorldSeed::random);

    let seed_state = SeedInputState {
        seed_text: seed.0.to_string(),
        is_editing: false,
    };

    let menu_state = if opts.quickstart {
        MenuState {
            screen: MenuScreen::None,
            previous_screen: MenuScreen::None,
            world_active: true,
        }
    } else {
        MenuState::default()
    };

    let profiler_state = ProfilerState {
        visible: opts.profile_mode,
    };

    let mut graphics_settings = GraphicsSettings::load_or_default();
    if let Some(vd) = opts.view_distance {
        graphics_settings.view_distance = vd;
    }

    let present_mode = if graphics_settings.vsync {
        bevy::window::PresentMode::AutoVsync
    } else {
        bevy::window::PresentMode::AutoNoVsync
    };
    let mode = if graphics_settings.fullscreen {
        bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };

    let benchmark_config = BenchmarkConfig::default();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: if opts.profile_mode {
                            format!("MineRust [PROFILE MODE] - Seed: {}", seed.0)
                        } else {
                            format!("MineRust - Seed: {}", seed.0)
                        },
                        resolution: WindowResolution::new(1280, 720),
                        present_mode,
                        mode,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::audio::AudioPlugin>(),
        )
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .insert_resource(WorldGrid::new(seed))
        .insert_resource(graphics_settings)
        .insert_resource(profiler_state)
        .insert_resource(seed_state)
        .insert_resource(menu_state)
        .insert_resource(benchmark_config)
        .configure_sets(
            Update,
            (
                VoxelStage::InputHandling,
                VoxelStage::PlayerPhysics,
                VoxelStage::FluidSimulation,
                VoxelStage::WorldStreaming,
                VoxelStage::MeshBuilding,
            )
                .chain(),
        )
        .add_plugins(MaterialPlugin::<VoxelBlockMaterial>::default())
        .add_plugins((
            CameraPlugin,
            PhysicsPlugin,
            FluidPlugin,
            WorldPlugin,
            InteractionPlugin,
            InventoryPlugin,
            MenuPlugin,
            SavePlugin,
            ProfilePlugin,
            minerust::benchmark::BenchmarkPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut world: ResMut<WorldGrid>,
    mut materials: ResMut<Assets<VoxelBlockMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    graphics_settings: Res<GraphicsSettings>,
    menu_state: Res<minerust::menu::MenuState>,
) {
    // 0. 2D Texture Array (25 layers of 16x16 pixel-art) and shared ExtendedMaterial with Alpha Mask for transparency (Glass)
    let array_image = texture::create_texture_array();
    let array_handle = images.add(array_image);

    let cull_mode = Some(bevy::render::render_resource::Face::Back);

    let block_mat = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            cull_mode,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        },
        extension: VoxelExtension {
            array_texture: array_handle.clone(),
        },
    });
    world.block_material = Some(block_mat);

    let water_mat = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 0.08,
            reflectance: 0.5,
            cull_mode,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: VoxelExtension {
            array_texture: array_handle,
        },
    });
    world.water_material = Some(water_mat);

    if menu_state.world_active {
        let center_chunk = WorldGrid::world_to_chunk_coord(0, 0).0;
        world.pregenerate_spawn_grid(
            center_chunk,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    // 1. Spawn FPS camera with integrated AmbientLight, PlayerPhysics component, and DistanceFog (if enabled in settings)
    let fps_camera = FpsCamera::default();

    let mut cam_builder = commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            far: 2500.0,
            ..default()
        }),
        AmbientLight {
            color: Color::WHITE,
            brightness: 320.0,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_y(fps_camera.yaw)
                * Quat::from_rotation_x(fps_camera.pitch),
            ..default()
        },
        fps_camera,
        PlayerPhysics::default(),
    ));

    if graphics_settings.distance_fog {
        let max_dist = (graphics_settings.view_distance as f32 * 16.0).max(64.0);
        cam_builder.insert(DistanceFog {
            color: Color::srgb(0.53, 0.81, 0.98),
            falloff: FogFalloff::Linear {
                start: (max_dist * 0.70).max(48.0),
                end: (max_dist - 2.0).max(64.0),
            },
            ..default()
        });
    }

    // 2. Sun light (Directional Light) with optimized shadow distance
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: graphics_settings.shadows,
            ..default()
        },
        bevy::light::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 30.0,
            maximum_distance: 120.0,
            num_cascades: 2,
            ..default()
        }
        .build(),
        Transform::from_xyz(200.0, 450.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_options_defaults() {
        let opts = CliOptions::parse_from_args(vec!["minerust".to_string()]).unwrap();
        assert_eq!(
            opts,
            CliOptions {
                seed: None,
                profile_mode: false,
                quickstart: false,
                view_distance: None,
            }
        );
    }

    #[test]
    fn test_cli_options_seed_and_flags() {
        let opts = CliOptions::parse_from_args(vec![
            "minerust".to_string(),
            "-s".to_string(),
            "42".to_string(),
            "-p".to_string(),
            "-q".to_string(),
        ])
        .unwrap();

        assert_eq!(opts.seed, Some(WorldSeed(42)));
        assert!(opts.profile_mode);
        assert!(opts.quickstart);
    }

    #[test]
    fn test_cli_options_view_distance_clamping() {
        let opts = CliOptions::parse_from_args(vec![
            "minerust".to_string(),
            "--view-distance".to_string(),
            "1".to_string(),
        ])
        .unwrap();
        assert_eq!(opts.view_distance, Some(2)); // Min clamp is 2

        let opts2 = CliOptions::parse_from_args(vec![
            "minerust".to_string(),
            "--view-distance".to_string(),
            "128".to_string(),
        ])
        .unwrap();
        assert_eq!(opts2.view_distance, Some(64)); // Max clamp is 64
    }

    #[test]
    fn test_cli_options_help_returns_err_with_message() {
        let res = CliOptions::parse_from_args(vec!["minerust".to_string(), "--help".to_string()]);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Usage: minerust"));
    }
}

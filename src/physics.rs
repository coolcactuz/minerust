use bevy::prelude::*;
use bevy::text::FontSize;

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::menu::MenuState;
use crate::world::{WorldGrid, calculate_biome_and_height};

pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

pub const GRAVITY: f32 = 28.0;
pub const TERMINAL_VELOCITY: f32 = -39.2;
pub const JUMP_VELOCITY: f32 = 8.6;

pub const WALK_SPEED: f32 = 4.3;
pub const SPRINT_SPEED: f32 = 7.0;
pub const SNEAK_SPEED: f32 = 1.8;

pub const FLY_SPEED: f32 = 14.0;
pub const FLY_SPRINT_SPEED: f32 = 28.0;

pub const GROUND_ACCEL: f32 = 45.0;
pub const GROUND_FRICTION: f32 = 14.0;
pub const AIR_ACCEL: f32 = 12.0;

pub const WATER_GRAVITY: f32 = 6.0;
pub const WATER_SWIM_SPEED: f32 = 4.0;
pub const WATER_DRAG: f32 = 5.0;

pub const STEP_HEIGHT: f32 = 1.05;

#[derive(Component)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub is_flying: bool,
    pub in_water: bool,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            is_grounded: false,
            is_flying: false,
            in_water: false,
        }
    }
}

#[derive(Component)]
pub struct PhysicsDebugText;

/// Checks if the player's bounding box (AABB) intersects solid blocks in the world
pub fn check_collision(feet_pos: Vec3, world: &WorldGrid) -> bool {
    let half_w = PLAYER_HALF_WIDTH;
    let height = PLAYER_HEIGHT;
    let p_min = feet_pos + Vec3::new(-half_w, 0.001, -half_w);
    let p_max = feet_pos + Vec3::new(half_w, height - 0.001, half_w);

    let min_bx = p_min.x.floor() as i32;
    let max_bx = p_max.x.floor() as i32;
    let min_by = p_min.y.floor() as i32;
    let max_by = p_max.y.floor() as i32;
    let min_bz = p_min.z.floor() as i32;
    let max_bz = p_max.z.floor() as i32;

    for bx in min_bx..=max_bx {
        for by in min_by..=max_by {
            for bz in min_bz..=max_bz {
                if world.is_solid_at(IVec3::new(bx, by, bz)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Checks if the player is currently immersed in water
pub fn is_in_water(feet_pos: Vec3, world: &WorldGrid) -> bool {
    let check_at = |pos: Vec3| -> bool {
        let bpos = IVec3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        world.get_block(bpos) == BlockType::Water
    };
    check_at(feet_pos + Vec3::new(0.0, 0.2, 0.0))
        || check_at(feet_pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0))
}

pub fn player_physics_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    inventory: Option<Res<Inventory>>,
    menu: Option<Res<MenuState>>,
    world: Res<WorldGrid>,
    mut query: Query<(&FpsCamera, &mut Transform, &mut PlayerPhysics)>,
) {
    if let Some(menu) = menu {
        if menu.is_open() {
            return;
        }
    }
    if let Some(inv) = inventory {
        if inv.is_open {
            return;
        }
    }

    let Ok((fps, mut transform, mut physics)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs().min(0.05);

    // Key F: Toggle Flight Mode (No-Clip / Creative)
    if keys.just_pressed(KeyCode::KeyF) {
        physics.is_flying = !physics.is_flying;
        physics.velocity = Vec3::ZERO;
    }

    let mut feet = transform.translation - Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);

    // 1. FLIGHT MODE (No-Clip / Creative)
    if physics.is_flying {
        let mut forward = Quat::from_rotation_y(fps.yaw) * -Vec3::Z;
        forward.y = 0.0;
        forward = forward.normalize_or_zero();

        let mut right = Quat::from_rotation_y(fps.yaw) * Vec3::X;
        right.y = 0.0;
        right = right.normalize_or_zero();

        let mut move_dir = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            move_dir += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            move_dir -= forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            move_dir -= right;
        }
        if keys.pressed(KeyCode::KeyD) {
            move_dir += right;
        }
        if keys.pressed(KeyCode::Space) {
            move_dir += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            move_dir -= Vec3::Y;
        }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        let speed = if keys.pressed(KeyCode::ControlLeft) {
            FLY_SPRINT_SPEED
        } else {
            FLY_SPEED
        };

        feet += move_dir * speed * dt;
        transform.translation = feet + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
        physics.velocity = Vec3::ZERO;
        physics.is_grounded = false;
        physics.in_water = false;
        return;
    }

    // 2. WALKING MODE (Physics, Gravity, Collisions, Jump, Swim)
    let in_water = is_in_water(feet, &world);
    physics.in_water = in_water;

    let mut forward = Quat::from_rotation_y(fps.yaw) * -Vec3::Z;
    forward.y = 0.0;
    forward = forward.normalize_or_zero();

    let mut right = Quat::from_rotation_y(fps.yaw) * Vec3::X;
    right.y = 0.0;
    right = right.normalize_or_zero();

    let mut wish_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        wish_dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        wish_dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        wish_dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        wish_dir += right;
    }
    if wish_dir.length_squared() > 0.0 {
        wish_dir = wish_dir.normalize();
    }

    let is_sneaking = keys.pressed(KeyCode::ShiftLeft) && physics.is_grounded && !in_water;

    let target_speed = if in_water {
        2.5
    } else if is_sneaking {
        SNEAK_SPEED
    } else if keys.pressed(KeyCode::ControlLeft) {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };

    let current_h_vel = Vec3::new(physics.velocity.x, 0.0, physics.velocity.z);
    let target_h_vel = wish_dir * target_speed;

    let accel = if in_water {
        15.0
    } else if physics.is_grounded {
        if wish_dir.length_squared() > 0.0 {
            GROUND_ACCEL
        } else {
            GROUND_FRICTION
        }
    } else {
        AIR_ACCEL
    };

    let new_h_vel = current_h_vel.move_towards(target_h_vel, accel * dt);
    physics.velocity.x = new_h_vel.x;
    physics.velocity.z = new_h_vel.z;

    // Vertical: Swim / Jump / Gravity
    if in_water {
        if keys.pressed(KeyCode::Space) {
            physics.velocity.y = WATER_SWIM_SPEED;
        } else if keys.pressed(KeyCode::ShiftLeft) {
            physics.velocity.y = -WATER_SWIM_SPEED;
        } else {
            physics.velocity.y = (physics.velocity.y - WATER_GRAVITY * dt).max(-3.0);
            physics.velocity.y *= 1.0 - (WATER_DRAG * dt).min(1.0);
        }
    } else {
        // Jump from ground or leap out of water
        if keys.pressed(KeyCode::Space) && physics.is_grounded {
            physics.velocity.y = JUMP_VELOCITY;
            physics.is_grounded = false;
        }
        // Standard gravity
        physics.velocity.y = (physics.velocity.y - GRAVITY * dt).max(TERMINAL_VELOCITY);
    }

    // X Collision Resolution
    let dx = physics.velocity.x * dt;
    if dx != 0.0 {
        let test_pos = feet + Vec3::new(dx, 0.0, 0.0);
        if !check_collision(test_pos, &world) {
            // Sneak edge protection (prevents walking off ledges with Shift)
            if is_sneaking && !check_collision(test_pos - Vec3::new(0.0, 0.1, 0.0), &world) {
                physics.velocity.x = 0.0;
            } else {
                feet.x = test_pos.x;
            }
        } else {
            // Step assist (automatically steps up terrain obstacles up to STEP_HEIGHT)
            let mut stepped = false;
            if physics.is_grounded {
                for step in [0.25, 0.5, 0.75, STEP_HEIGHT] {
                    let step_pos = feet + Vec3::new(dx, step, 0.0);
                    if !check_collision(step_pos, &world) {
                        feet = step_pos;
                        stepped = true;
                        break;
                    }
                }
            }
            if !stepped {
                physics.velocity.x = 0.0;
            }
        }
    }

    // Z Collision Resolution
    let dz = physics.velocity.z * dt;
    if dz != 0.0 {
        let test_pos = feet + Vec3::new(0.0, 0.0, dz);
        if !check_collision(test_pos, &world) {
            // Sneak edge protection
            if is_sneaking && !check_collision(test_pos - Vec3::new(0.0, 0.1, 0.0), &world) {
                physics.velocity.z = 0.0;
            } else {
                feet.z = test_pos.z;
            }
        } else {
            // Step assist
            let mut stepped = false;
            if physics.is_grounded {
                for step in [0.25, 0.5, 0.75, STEP_HEIGHT] {
                    let step_pos = feet + Vec3::new(0.0, step, dz);
                    if !check_collision(step_pos, &world) {
                        feet = step_pos;
                        stepped = true;
                        break;
                    }
                }
            }
            if !stepped {
                physics.velocity.z = 0.0;
            }
        }
    }

    // Y Collision Resolution (Gravity & Ceiling)
    let dy = physics.velocity.y * dt;
    if dy != 0.0 {
        let test_pos = feet + Vec3::new(0.0, dy, 0.0);
        if !check_collision(test_pos, &world) {
            feet.y = test_pos.y;
            physics.is_grounded = false;
        } else {
            let mut safe_y = feet.y;
            for i in 1..=8 {
                let frac = i as f32 / 8.0;
                let check_y = feet.y + dy * frac;
                if check_collision(Vec3::new(feet.x, check_y, feet.z), &world) {
                    break;
                }
                safe_y = check_y;
            }
            feet.y = safe_y;

            if dy < 0.0 {
                physics.velocity.y = 0.0;
                physics.is_grounded = true;
            } else {
                physics.velocity.y = 0.0;
            }
        }
    }

    // Ground support check
    if !in_water && physics.velocity.y <= 0.0 {
        let ground_check = feet - Vec3::new(0.0, 0.05, 0.0);
        physics.is_grounded = check_collision(ground_check, &world);
    }

    // Void fall protection (Safe surface respawn)
    if feet.y < -20.0 {
        let (_, spawn_y, _) =
            calculate_biome_and_height(feet.x as f64, feet.z as f64, &world.noise);
        feet.y = (spawn_y as f32 + 4.0).max(130.0);
        physics.velocity = Vec3::ZERO;
        physics.is_grounded = false;
    }

    transform.translation = feet + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
}

#[derive(Default)]
pub struct FpsTracker {
    pub fps: f32,
    pub frame_time_ms: f32,
    timer: f32,
    frames: u32,
}

#[derive(Component)]
pub struct PhysicsDebugRoot;

pub fn setup_physics_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.82)),
            BorderColor::all(Color::srgba(0.3, 0.5, 0.7, 0.8)),
            PhysicsDebugRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("⚡ MINERUST BENCHMARK [F3]\nFPS: --\nLoading stats..."),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.5)),
                PhysicsDebugText,
            ));
        });
}

pub fn update_physics_hud_system(
    time: Res<Time>,
    mut fps: Local<FpsTracker>,
    world: Option<Res<WorldGrid>>,
    dev_settings: Option<Res<crate::menu::DevSettings>>,
    mut text_query: Query<&mut Text, With<PhysicsDebugText>>,
    mut root_query: Query<&mut Visibility, With<PhysicsDebugRoot>>,
) {
    let show_hud = dev_settings
        .as_ref()
        .is_some_and(|d| d.dev_mode && d.show_debug_hud);
    if let Ok(mut vis) = root_query.single_mut() {
        let target = if show_hud {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }

    if !show_hud {
        return;
    }

    let dt = time.delta_secs();
    fps.timer += dt;
    fps.frames += 1;

    if fps.timer >= 0.25 {
        fps.fps = fps.frames as f32 / fps.timer;
        fps.frame_time_ms = (fps.timer / fps.frames as f32) * 1000.0;
        fps.timer = 0.0;
        fps.frames = 0;

        if let Ok(mut text) = text_query.single_mut() {
            let (chunks_loaded, meshes_active, gen_q, mesh_q, total_verts) =
                if let Some(ref w) = world {
                    (
                        w.chunks.len(),
                        w.chunk_entities.len(),
                        w.generation_queue.len(),
                        w.mesh_queue.len(),
                        w.total_vertices,
                    )
                } else {
                    (0, 0, 0, 0, 0)
                };

            let (cull, shadow, max_y, fog, budget, async_m, greedy, lod) =
                if let Some(ref dev) = dev_settings {
                    (
                        if dev.backface_culling { "ON" } else { "OFF" },
                        if dev.shadows_enabled { "ON" } else { "OFF" },
                        if dev.max_y_skip { "ON" } else { "OFF" },
                        if dev.distance_fog { "ON" } else { "OFF" },
                        if dev.mesh_budget { "ON" } else { "OFF" },
                        if dev.async_meshing { "ON" } else { "OFF" },
                        if dev.greedy_meshing { "ON" } else { "OFF" },
                        if dev.distance_lod {
                            format!("ON ({}ch)", dev.lod_threshold)
                        } else {
                            "OFF".to_string()
                        },
                    )
                } else {
                    (
                        "ON",
                        "ON",
                        "ON",
                        "ON",
                        "ON",
                        "ON",
                        "ON",
                        "ON (4ch)".to_string(),
                    )
                };

            let verts_str = if total_verts >= 1_000_000 {
                format!("{:.2}M", total_verts as f32 / 1_000_000.0)
            } else if total_verts >= 1_000 {
                format!("{:.0}k", total_verts as f32 / 1_000.0)
            } else {
                format!("{}", total_verts)
            };

            *text = Text::new(format!(
                "⚡ MINERUST BENCHMARK [F3: Toggle]\n\
                 FPS: {:.0} ({:.1} ms) | Verts: {}\n\
                 Chunks: {} | Meshes: {} | GenQ: {} | MeshQ: {}\n\
                 [Cull: {}] [Shadows: {}] [max_y: {}] [Fog: {}] [Budget: {}] [Async: {}] [Greedy: {}] [LOD: {}]",
                fps.fps,
                fps.frame_time_ms,
                verts_str,
                chunks_loaded,
                meshes_active,
                gen_q,
                mesh_q,
                cull,
                shadow,
                max_y,
                fog,
                budget,
                async_m,
                greedy,
                lod
            ));
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_physics_ui).add_systems(
            Update,
            (
                player_physics_system.in_set(crate::stage::VoxelStage::PlayerPhysics),
                update_physics_hud_system,
            ),
        );
    }
}

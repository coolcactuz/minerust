#![forbid(unsafe_code)]

use bevy::prelude::*;

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::item::{ItemType, MaterialType, ToolType};

/// Component tracking the first-person held item viewmodel entity attached to the camera.
#[derive(Component, Default)]
pub struct HeldItemRoot {
    pub last_item: Option<ItemType>,
    pub swing_progress: f32,
}

/// Marker component for mesh entities spawned inside the held item root.
#[derive(Component)]
pub struct HeldItemVisual;

/// Spawns the held item root container as a child of the FPS camera when the camera is created.
pub fn setup_held_item_root_system(
    mut commands: Commands,
    camera_query: Query<Entity, Added<FpsCamera>>,
) {
    for cam_entity in &camera_query {
        commands.entity(cam_entity).with_children(|parent| {
            parent.spawn((
                Transform::from_translation(Vec3::new(0.34, -0.30, -0.50))
                    .with_rotation(Quat::from_euler(EulerRot::YXZ, -0.35, 0.20, -0.10)),
                Visibility::Inherited,
                HeldItemRoot::default(),
            ));
        });
    }
}

/// Dynamically updates the 3D model spawned in the player's hand when the active hotbar slot changes.
pub fn update_held_item_model_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inventory: Res<Inventory>,
    mut root_query: Query<(Entity, &mut HeldItemRoot)>,
    children_query: Query<&Children>,
    visual_query: Query<Entity, With<HeldItemVisual>>,
) {
    let current_item = inventory.selected_item().map(|s| s.item);

    for (root_entity, mut root) in &mut root_query {
        if root.last_item != current_item {
            root.last_item = current_item;

            // Despawn existing held item visual children
            if let Ok(children) = children_query.get(root_entity) {
                for child in children.iter() {
                    if visual_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }

            // Spawn new 3D model according to current equipped item
            let Some(item) = current_item else {
                continue;
            };

            commands.entity(root_entity).with_children(|parent| {
                match item {
                    ItemType::Block(block) => {
                        if block != BlockType::Air {
                            let [r, g, b, _] = block.color(crate::block::BlockFace::Top);
                            let mesh = meshes.add(Cuboid::new(0.20, 0.20, 0.20));
                            let mat = materials.add(StandardMaterial {
                                base_color: Color::srgb(r, g, b),
                                perceptual_roughness: 0.8,
                                ..default()
                            });

                            parent.spawn((
                                Mesh3d(mesh),
                                MeshMaterial3d(mat),
                                Transform::from_xyz(0.0, 0.0, 0.0),
                                HeldItemVisual,
                            ));
                        }
                    }
                    ItemType::Tool { tool, tier } => {
                        let [tr, tg, tb] = tier.color();
                        let wood_mat = materials.add(StandardMaterial {
                            base_color: Color::srgb(0.55, 0.38, 0.22),
                            perceptual_roughness: 0.9,
                            ..default()
                        });
                        let head_mat = materials.add(StandardMaterial {
                            base_color: Color::srgb(tr, tg, tb),
                            metallic: 0.6,
                            perceptual_roughness: 0.3,
                            ..default()
                        });

                        match tool {
                            ToolType::Pickaxe => {
                                // Wooden handle rod
                                let handle_mesh = meshes.add(Cuboid::new(0.03, 0.42, 0.03));
                                parent.spawn((
                                    Mesh3d(handle_mesh),
                                    MeshMaterial3d(wood_mat),
                                    Transform::from_xyz(0.0, 0.0, 0.0),
                                    HeldItemVisual,
                                ));
                                // Pickaxe horizontal head bar
                                let head_mesh = meshes.add(Cuboid::new(0.24, 0.045, 0.045));
                                parent.spawn((
                                    Mesh3d(head_mesh),
                                    MeshMaterial3d(head_mat),
                                    Transform::from_xyz(0.0, 0.19, 0.0),
                                    HeldItemVisual,
                                ));
                            }
                            ToolType::Axe => {
                                // Wooden handle rod
                                let handle_mesh = meshes.add(Cuboid::new(0.03, 0.42, 0.03));
                                parent.spawn((
                                    Mesh3d(handle_mesh),
                                    MeshMaterial3d(wood_mat),
                                    Transform::from_xyz(0.0, 0.0, 0.0),
                                    HeldItemVisual,
                                ));
                                // Axe blade
                                let blade_mesh = meshes.add(Cuboid::new(0.12, 0.12, 0.04));
                                parent.spawn((
                                    Mesh3d(blade_mesh),
                                    MeshMaterial3d(head_mat),
                                    Transform::from_xyz(0.05, 0.16, 0.0),
                                    HeldItemVisual,
                                ));
                            }
                            ToolType::Shovel => {
                                // Wooden handle rod
                                let handle_mesh = meshes.add(Cuboid::new(0.03, 0.42, 0.03));
                                parent.spawn((
                                    Mesh3d(handle_mesh),
                                    MeshMaterial3d(wood_mat),
                                    Transform::from_xyz(0.0, 0.0, 0.0),
                                    HeldItemVisual,
                                ));
                                // Shovel scoop
                                let scoop_mesh = meshes.add(Cuboid::new(0.10, 0.14, 0.025));
                                parent.spawn((
                                    Mesh3d(scoop_mesh),
                                    MeshMaterial3d(head_mat),
                                    Transform::from_xyz(0.0, 0.20, 0.0),
                                    HeldItemVisual,
                                ));
                            }
                            ToolType::Sword => {
                                // Sword handle
                                let handle_mesh = meshes.add(Cuboid::new(0.03, 0.12, 0.03));
                                parent.spawn((
                                    Mesh3d(handle_mesh),
                                    MeshMaterial3d(wood_mat),
                                    Transform::from_xyz(0.0, -0.14, 0.0),
                                    HeldItemVisual,
                                ));
                                // Guard crossbar
                                let guard_mesh = meshes.add(Cuboid::new(0.14, 0.03, 0.04));
                                let guard_mat = materials.add(StandardMaterial {
                                    base_color: Color::srgb(0.35, 0.35, 0.40),
                                    metallic: 0.5,
                                    perceptual_roughness: 0.4,
                                    ..default()
                                });
                                parent.spawn((
                                    Mesh3d(guard_mesh),
                                    MeshMaterial3d(guard_mat),
                                    Transform::from_xyz(0.0, -0.07, 0.0),
                                    HeldItemVisual,
                                ));
                                // Blade
                                let blade_mesh = meshes.add(Cuboid::new(0.05, 0.46, 0.02));
                                parent.spawn((
                                    Mesh3d(blade_mesh),
                                    MeshMaterial3d(head_mat),
                                    Transform::from_xyz(0.0, 0.18, 0.0),
                                    HeldItemVisual,
                                ));
                            }
                            ToolType::Hoe => {
                                // Handle
                                let handle_mesh = meshes.add(Cuboid::new(0.03, 0.42, 0.03));
                                parent.spawn((
                                    Mesh3d(handle_mesh),
                                    MeshMaterial3d(wood_mat),
                                    Transform::from_xyz(0.0, 0.0, 0.0),
                                    HeldItemVisual,
                                ));
                                // Hoe head
                                let head_mesh = meshes.add(Cuboid::new(0.12, 0.04, 0.04));
                                parent.spawn((
                                    Mesh3d(head_mesh),
                                    MeshMaterial3d(head_mat),
                                    Transform::from_xyz(0.045, 0.19, 0.0),
                                    HeldItemVisual,
                                ));
                            }
                        }
                    }
                    ItemType::Material(mat) => {
                        if mat == MaterialType::Stick {
                            let rod_mesh = meshes.add(Cuboid::new(0.03, 0.40, 0.03));
                            let wood_mat = materials.add(StandardMaterial {
                                base_color: Color::srgb(0.55, 0.38, 0.22),
                                perceptual_roughness: 0.9,
                                ..default()
                            });
                            parent.spawn((
                                Mesh3d(rod_mesh),
                                MeshMaterial3d(wood_mat),
                                Transform::from_xyz(0.0, 0.0, 0.0),
                                HeldItemVisual,
                            ));
                        } else {
                            let [mr, mg, mb] = mat.color();
                            let ingot_mesh = meshes.add(Cuboid::new(0.14, 0.09, 0.20));
                            let ingot_mat = materials.add(StandardMaterial {
                                base_color: Color::srgb(mr, mg, mb),
                                metallic: 0.7,
                                perceptual_roughness: 0.3,
                                ..default()
                            });
                            parent.spawn((
                                Mesh3d(ingot_mesh),
                                MeshMaterial3d(ingot_mat),
                                Transform::from_xyz(0.0, 0.0, 0.0),
                                HeldItemVisual,
                            ));
                        }
                    }
                }
            });
        }
    }
}

/// Plays a smooth Minecraft-style swinging animation when mining or left clicking.
pub fn held_item_swing_system(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    inventory: Res<Inventory>,
    mut root_query: Query<(&mut Transform, &mut HeldItemRoot)>,
) {
    let dt = time.delta_secs();
    let is_clicking = mouse_buttons.pressed(MouseButton::Left) && !inventory.is_open;

    for (mut transform, mut root) in &mut root_query {
        if is_clicking {
            // Rapid swing cycle (~3.5 swings/sec while mining or attacking)
            root.swing_progress = (root.swing_progress + dt * 4.5).fract();
        } else if root.swing_progress > 0.0 {
            // Return quickly to idle rest pose
            root.swing_progress = (root.swing_progress + dt * 6.0).min(1.0);
            if root.swing_progress >= 1.0 {
                root.swing_progress = 0.0;
            }
        }

        let base_pos = Vec3::new(0.34, -0.30, -0.50);
        let base_rot = Quat::from_euler(EulerRot::YXZ, -0.35, 0.20, -0.10);

        if root.swing_progress > 0.0 {
            let swing_sin = (root.swing_progress * std::f32::consts::PI).sin();
            let swing_rot = Quat::from_euler(
                EulerRot::XYZ,
                -swing_sin * 0.75,
                swing_sin * 0.25,
                -swing_sin * 0.45,
            );
            transform.translation =
                base_pos + Vec3::new(-swing_sin * 0.06, -swing_sin * 0.04, swing_sin * 0.05);
            transform.rotation = base_rot * swing_rot;
        } else {
            transform.translation = base_pos;
            transform.rotation = base_rot;
        }
    }
}

pub struct HeldItemPlugin;

impl Plugin for HeldItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup_held_item_root_system,
                update_held_item_model_system,
                held_item_swing_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_held_item_root_default() {
        let root = HeldItemRoot::default();
        assert!(root.last_item.is_none());
        assert_eq!(root.swing_progress, 0.0);
    }
}

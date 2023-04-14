use bevy::{
	core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa},
	gltf::Gltf,
	render::{camera::ScalingMode, view::RenderLayers},
};

use crate::prelude::*;

pub struct BackpackPlugin;
impl Plugin for BackpackPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems((spawn,).in_schedule(OnEnter(GameState::InGame)));

		//.add_system(update_inventory_items)
	}
}

#[derive(Component, Clone, Debug, Default, Reflect, FromReflect)]
pub struct Inventory(pub Vec<Entity>);

#[derive(Component)]
pub struct Backpack;

#[derive(Component)]
pub struct InventoryCamera;

fn spawn(mut commands: Commands, game_assets: Res<GameAssets>, gltfs: Res<Assets<Gltf>>) {
	let backpack = gltfs.get(&game_assets.backpack_gltf).unwrap();

	let mut camera_transform =
		Transform::from_rotation(Quat::from_rotation_x(-0.2) * Quat::from_rotation_y(-0.25));
	camera_transform.translation = camera_transform.back() * 10.0 + Vec3::X * -2.0 + Vec3::Y * -2.0;

	// Inventory camera
	commands.spawn((
		Name::new("Inventory Camera"),
		InventoryCamera,
		Camera3dBundle {
			projection: Projection::Orthographic(OrthographicProjection {
				scale: 1.0,
				viewport_origin: Vec2::new(0.0, 0.0),
				scaling_mode: ScalingMode::FixedVertical(8.0),
				..default()
			}),
			transform: camera_transform,
			camera_3d: Camera3d {
				clear_color: ClearColorConfig::None,
				..default()
			},
			camera: Camera {
				order: 1,
				..default()
			},

			..default()
		},
		RenderLayers::layer(2),
		Fxaa::default(),
	));

	commands.spawn((
		Backpack,
		Name::new("Backpack"),
		RigidBody::Fixed,
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2, Group::GROUP_2 | Group::GROUP_5),
		SceneBundle {
			scene: backpack.scenes[1].clone(),
			transform: Transform::from_xyz(0.0, -1.5, 0.0).with_scale(Vec3::new(3.0, 2.6, 3.0)),
			..default()
		},
		NamedMaterials::backpack(),
		// It was fun figuring out the colliders positions manually, recompiling every time
		// Child colliders DO NOT update when you scale thier parent (or even update their local translation)
		// Which is so fucking annoying and I spent literal hours debugging this and trying to find workarounds
		// Compound colliders are the only way. Dimforge pls fix.
		Collider::compound(vec![
			(
				Vec3::new(0.0, 0.02, 0.0),
				Quat::IDENTITY,
				Collider::cuboid(0.27, 0.012, 0.5),
			),
			(
				Vec3::new(-0.27, 0.18, 0.0),
				Quat::IDENTITY,
				Collider::cuboid(0.012, 0.18, 0.5),
			),
			(
				Vec3::new(0.27, 0.18, 0.0),
				Quat::IDENTITY,
				Collider::cuboid(0.012, 0.18, 0.5),
			),
			(
				Vec3::new(-0.252, 0.53, 0.0),
				Quat::from_rotation_z(-6.0f32.to_radians()),
				Collider::cuboid(0.012, 0.173, 0.5),
			),
			(
				Vec3::new(0.252, 0.53, 0.0),
				Quat::from_rotation_z(6.0f32.to_radians()),
				Collider::cuboid(0.012, 0.173, 0.5),
			),
			(
				Vec3::new(-0.32, 0.17, 0.0),
				Quat::IDENTITY,
				Collider::cuboid(0.04, 0.13, 0.5),
			),
			(
				Vec3::new(0.32, 0.17, 0.0),
				Quat::IDENTITY,
				Collider::cuboid(0.04, 0.13, 0.5),
			),
		]),
	));
}

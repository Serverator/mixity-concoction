use bevy::{render::{view::RenderLayers, camera::ScalingMode}, core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa}};

use crate::{prelude::*, assets::Spawnable};

pub struct BackpackPlugin;
impl Plugin for BackpackPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_systems((
			spawn,
				).in_schedule(OnEnter(GameState::InGame))
		);
		// .add_systems((
		// 	test_init,
		// 		).in_set(OnUpdate(GameState::InGame))
		// );
	}
}

fn spawn(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	//mut materials: ResMut<Assets<FoliageMaterial>>,
	mut standard_mat: ResMut<Assets<StandardMaterial>>,
	spawnables: Res<Assets<Spawnable>>,
) {
		
	// In other cam
	commands.spawn((
		Name::new("Inventory Ball"),
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::UVSphere {
				radius: 0.4,
				..default()
			})),
			material: standard_mat.add(StandardMaterial {
				base_color: Color::WHITE,
				..default()
			}),
			transform: Transform::from_xyz(-5.0, 5.0, 0.0),//.with_scale(Vec3::splat(3.0)),
			..default()
		},
		RigidBody::Dynamic,
		Collider::ball(0.4),
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		//SolverGroups::new(Group::GROUP_2,Group::GROUP_2),
	));

	// In other cam
	commands.spawn((
		Name::new("Inventory Shroom"),
		SceneBundle {
			scene: spawnables.iter().nth(3).unwrap().1.scene.clone(),
			transform: Transform::from_xyz(-5.0, 3.0, 0.0),//.with_scale(Vec3::splat(3.0)),

			..default()
		},
		RigidBody::Dynamic,
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		Velocity::default(),
		//SolverGroups::new(Group::GROUP_2,Group::GROUP_2),
	)).with_children(|commands| {
		commands.spawn((
			ColliderDisabled,
			Collider::ball(0.3),
			Transform::from_translation(Vec3::Y * 0.25),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));
	});
	

	// Inventory camera
	commands.spawn((
		Name::new("Inventory Camera"),
		Camera3dBundle {
			projection: Projection::Orthographic(OrthographicProjection {
				scale: 1.0,
				viewport_origin: Vec2::new(0.5,0.5),
				scaling_mode: ScalingMode::FixedVertical(10.0),
				..default()
			}),
			transform: Transform::from_translation(Vec3::Z * 100.0),
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

	let cube = meshes.add(Mesh::from(shape::Cube::new(1.0)));

	commands.spawn((
		Name::new("Backpack"),
		RigidBody::Fixed,
		VisibilityBundle::default(),
		TransformBundle::from_transform(Transform::from_xyz(-5.0, 0.0, 0.0)),
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
	)).with_children(|commands| {
		commands.spawn((
			PbrBundle {
				mesh: cube.clone(),
				transform: Transform::from_translation(Vec3::NEG_Y).with_scale(Vec3::new(2.5,0.2,2.0)),
				..default()
			},
			Collider::cuboid(0.5,0.5,0.5),
			RenderLayers::layer(2),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));

		commands.spawn((
			PbrBundle {
				mesh: cube.clone(),
				transform: Transform::from_translation(Vec3::new(-1.25,0.5,0.0)).with_scale(Vec3::new(0.2,3.0,2.0)),
				..default()
			},
			Collider::cuboid(0.5,0.5,0.5),
			RenderLayers::layer(2),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));

		commands.spawn((
			PbrBundle {
				mesh: cube.clone(),
				transform: Transform::from_translation(Vec3::new(1.25,0.5,0.0)).with_scale(Vec3::new(0.2,3.0,2.0)),
				..default()
			},
			Collider::cuboid(0.5,0.5,0.5),
			RenderLayers::layer(2),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));

		commands.spawn((
			PbrBundle {
				mesh: cube.clone(),
				transform: Transform::from_translation(Vec3::new(0.0,0.5,-1.25)).with_scale(Vec3::new(2.5,3.0,0.2)),
				material: standard_mat.add(StandardMaterial {
						base_color: Color::WHITE,
						..default()
					}),
				..default()
			},
			Collider::cuboid(0.5,0.5,0.5),
			RenderLayers::layer(2),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));

		commands.spawn((
			Transform::from_translation(Vec3::new(0.0,0.5,1.25)).with_scale(Vec3::new(2.5,3.0,0.2)),
			Collider::cuboid(0.5,0.5,0.5),
			RenderLayers::layer(2),
			CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		));
	});
}

// #[derive(Component)]
// struct Test;

// #[allow(clippy::type_complexity)]
// fn test_init(
// 	mut commands: Commands,
// 	scene_manager: Res<SceneSpawner>,
// 	children_query: Query<&Children>,
// 	test_query: Query<(Entity, &SceneInstance), (With<Test>, Or<(Without<SceneLoaded>, (With<SceneLoaded>, Changed<SceneInstance>))>)>
// ) {
// 	for (entity,scene) in &test_query {
// 		if !scene_manager.instance_is_ready(**scene) {
// 			continue;
// 		}

// 		commands.entity(entity).insert(SceneLoaded);

// 		for child in children_query.iter_descendants(entity) {
// 			commands.entity(child).insert((
// 				RenderLayers::layer(2),
// 			));
// 		}
// 	}
// }
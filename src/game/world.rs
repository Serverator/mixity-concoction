use bevy::{math::Vec3Swizzles, scene::SceneInstance};

use crate::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_systems(
			(
				init_world,
				spawn_trees,
				spawn_bushes,
			)
				.in_schedule(OnEnter(GameState::InGame))
		)
		.add_system(
			init_loaded_scenes
				.in_set(OnUpdate(GameState::InGame))
		);
	}
}

fn init_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let plane_mesh = Mesh::from(shape::Plane { size: 400.0, subdivisions: 0 });
	// Plane
	commands.spawn((
		Name::new("Main Plane"),
		Collider::from_bevy_mesh(&plane_mesh, &ComputedColliderShape::TriMesh).unwrap(),
		PbrBundle {
			mesh: meshes.add(plane_mesh),
			material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
			..default()
		},
		RigidBody::Fixed,
	));

	// Light
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_rotation_y(0.5) * Quat::from_rotation_x(-PI/3.0)),
		..default()
	});

	// Ambient light
	commands.insert_resource(AmbientLight {
		color: Color::ALICE_BLUE,
		brightness: 0.15,
	});
}

#[derive(Component)]
pub struct Tree;

fn spawn_trees(
	mut command: Commands,
	asset_server: Res<AssetServer>,
) {
	let tree_scene: Handle<Scene> = asset_server.load("models/Tree.gltf#Scene0");
	let mut rng = thread_rng();

	for i in 0..1500 {
		let position = Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));
		if position.length_squared() < 6.0 {
			continue;
		}

		command.spawn((
			Tree,
			Name::new(format!("Tree {i}")),
			Collider::cylinder(4.0, 0.8),
			SceneBundle {
				scene: tree_scene.clone(),
				transform: Transform::from_translation(position.extend(0.0).xzy()).with_scale(Vec3::splat(rng.gen_range(0.25..0.6))).with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
				..default()
			},
		));
	}
}

#[derive(Component, Default, Debug)]
struct SceneLoaded;

#[allow(clippy::type_complexity)]
fn init_loaded_scenes(
	mut commands: Commands,
	scene_manager: Res<SceneSpawner>,
	mut material_assets: ResMut<Assets<FoliageMaterial>>,
	_material_query: Query<&mut Handle<StandardMaterial>>,
	name_query: Query<&Name>,
	children_query: Query<&Children>,
	tree_query: Query<(Entity,&SceneInstance),(With<Tree>, Or<(Without<SceneLoaded>, (With<SceneLoaded>, Changed<SceneInstance>))>)>
) {
	let mut rng = thread_rng();

	for (entity,scene) in &tree_query {
		if !scene_manager.instance_is_ready(**scene) {
			continue;
		}
		commands.entity(entity).insert(SceneLoaded);

		// let leaves_material = material_assets.add(StandardMaterial {
		// 	base_color: Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
		// 	perceptual_roughness: 0.5,
		// 	..default()
		// });

		let leaves_material = material_assets.add(FoliageMaterial {
			color: Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
			..default()
		});

		// Iterate by children names inside GLTF to set additional components/materials
		// Probably really bad thing to do, but idk how to do it properly
		for child in children_query.iter_descendants(entity) {
			if let Ok(name) = name_query.get(child) {
				match name.as_str() {
					"Trunk Mesh" => {
						//let mut material = material_query.get_mut(child).unwrap();
						//*material = leaves_material.clone();
					}
					"Leaves Mesh" => {
						commands.entity(child)
							.remove::<Handle<StandardMaterial>>()
							.insert(leaves_material.clone());
						//let mut material = material_query.get_mut(child).unwrap();
						//*material = leaves_material.clone();
					}
					_ => {}
				}
			}
		}

	}
}

fn spawn_bushes(
	mut command: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	
	let mut rng = thread_rng();
	for i in 0..1500 {
		let position = Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));
		if position.length_squared() < 6.0 {
			continue;
		}

		command.spawn((
			Name::new(format!("Bush {i}")),
			PbrBundle {
				transform: Transform::from_translation(position.extend(0.6).xzy()),
				mesh: meshes.add(Mesh::try_from(shape::Icosphere { radius: rng.gen_range(1.2..2.5), subdivisions: 0 }).unwrap()),
				material: materials.add(Color::rgb(0.2, 0.8, 0.2).into()),
				..default()
			},
		));
	}
}
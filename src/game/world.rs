use bevy::{math::Vec3Swizzles, scene::SceneInstance, gltf::Gltf, pbr::NotShadowReceiver};

use crate::prelude::*;

use super::effects::{MainEffect, SideEffect};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_systems(
			(
				init_world,
				spawn_vegetation,
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
	mut materials: ResMut<Assets<FoliageMaterial>>,
	mut standard_mat: ResMut<Assets<StandardMaterial>>,
) {
	let plane_mesh = Mesh::from(shape::Plane { size: 600.0, subdivisions: 0 });

	// Plane
	commands.spawn((
		Name::new("Main Plane"),
		Collider::from_bevy_mesh(&plane_mesh, &ComputedColliderShape::TriMesh).unwrap(),
		MaterialMeshBundle::<FoliageMaterial> {
			mesh: meshes.add(plane_mesh),
			material: materials.add(FoliageMaterial {
				color: Color::YELLOW_GREEN, //Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
				..default()
			}),
			..default()
		},
		RigidBody::Fixed,
	));

	// sphere
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::UVSphere {
				radius: 0.5,
				..default()
			})),
			material: standard_mat.add(StandardMaterial {
				base_color: Color::WHITE,
				..default()
			}),
			transform: Transform::from_xyz(1.5, 5.0, 1.5).with_scale(Vec3::splat(3.0)),
			..default()
		},
	));

	// Light
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			shadows_enabled: false,
			..default()
		},
		transform: Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_rotation_y(0.5) * Quat::from_rotation_x(-PI/3.5)),
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

fn spawn_vegetation(
	mut command: Commands,
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
) {

	const SPAWN_SIZE: f32 = 300.0;

	let mut occupied_space = vec![];

	// Trees
	let gltf = gltfs.get(&assets.tree_gltf).unwrap();
	let mut rng = thread_rng();

	command.spawn((
		Name::new("Tree Collection"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	)).with_children(|command| {
		for i in 0..2500 {
			let position = Vec2::new(rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE), rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE));
		
			if position.length_squared() < 6.0 && occupied_space.iter().any(|x| Vec2::length_squared(position - *x) < 80.0) {
				continue;
			}

			occupied_space.push(position);

			let scene = gltf.scenes[rng.gen_range(0..gltf.scenes.len())].clone();
	
			command.spawn((
				Tree,
				NotShadowReceiver,
				Name::new(format!("Tree {i}")),
				Collider::cylinder(4.0, 0.8),
				SceneBundle {
					scene,
					transform: Transform::from_translation(position.extend(0.0).xzy()).with_scale(Vec3::splat(rng.gen_range(0.25..0.6))).with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
					..default()
				},
			));
		}
	});

	// Bushes
	let gltf = gltfs.get(&assets.bush_gltf).unwrap();

	command.spawn((
		Name::new("Bush Collection"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	)).with_children(|command| {
		
		for i in 0..2500 {
			let position = Vec2::new(rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE), rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE));
			if position.length_squared() < 6.0 && occupied_space.iter().any(|x| Vec2::length_squared(position - *x) < 45.0) {
				continue;
			}

			occupied_space.push(position);

			let scene = gltf.scenes[rng.gen_range(0..gltf.scenes.len())].clone();
	
			command.spawn((
				NotShadowReceiver,
				Bush,
				Name::new(format!("Bush {i}")),
				SceneBundle {
					scene,
					transform: Transform::from_translation(position.extend(0.0).xzy()).with_scale(Vec3::splat(rng.gen_range(1.2..1.9))).with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
					..default()
				},
			));
		}
	});

	// Ingridients
	let gltf = gltfs.get(&assets.ingridients_gltf).unwrap();

	command.spawn((
		Name::new("Ingridient Collection"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	)).with_children(|command| {

		for i in 0..550 {
			let position = Vec2::new(rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE), rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE));
			if occupied_space.iter().any(|x| Vec2::length_squared(position - *x) < 20.0) {
				continue;
			}

			occupied_space.push(position);

			let scene = gltf.scenes[rng.gen_range(0..gltf.scenes.len())].clone();
	
			command.spawn((
				Ingridient {
					main_effect: MainEffect,
					side_effect: SideEffect,
				},
				NotShadowReceiver,
				Name::new(format!("Shroom {i}")),
				SceneBundle {
					scene,
					transform: Transform::from_translation(position.extend(0.0).xzy()).with_scale(Vec3::splat(rng.gen_range(2.0..2.8))).with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
					..default()
				},
			));
		}
	});
}

#[derive(Component, Default, Debug)]
struct SceneLoaded;

#[allow(clippy::type_complexity)]
fn init_loaded_scenes(
	mut commands: Commands,
	scene_manager: Res<SceneSpawner>,
	mut material_assets: ResMut<Assets<FoliageMaterial>>,
	name_query: Query<&Name, With<Handle<StandardMaterial>>>,
	children_query: Query<&Children>,
	tree_query: Query<(Entity,&SceneInstance),(Or<(With<Tree>, With<Bush>, With<Ingridient>)>, Or<(Without<SceneLoaded>, (With<SceneLoaded>, Changed<SceneInstance>))>)>
) {
	let mut rng = thread_rng();

	for (entity,scene) in &tree_query {
		if !scene_manager.instance_is_ready(**scene) {
			continue;
		}
		commands.entity(entity).insert(SceneLoaded);

		let is_rare = rng.gen_bool(0.0025); 

		// Iterate by children names inside GLTF to set additional components/materials
		// Probably really bad thing to do, but idk how to do it properly
		for child in children_query.iter_descendants(entity) {
			if let Ok(name) = name_query.get(child) {
				match name.as_str() {
					i if i.contains("Trunk") => {

						const COLORS: &[Color] = &[
							Color::rgb(0.5, 0.3, 0.05),
							Color::rgb(0.55, 0.35, 0.05),
							Color::rgb(0.45, 0.25, 0.05),
						];

						let color = if is_rare {
							Color::rgb(0.85, 0.85, 0.9)
						} else {
							COLORS[rng.gen_range(0..(COLORS.len()))]
						};

						// TODO: Remember material
						let bark_material = material_assets.add(FoliageMaterial {
							color,
							..default()
						});

						commands.entity(child)
						.remove::<Handle<StandardMaterial>>()
						.insert(bark_material.clone());
					}
					i if i.contains("Leaves") => {
						const COLORS: &[Color] = &[
							Color::GREEN,
							Color::LIME_GREEN,
							Color::YELLOW_GREEN,
							Color::ORANGE,
							Color::ORANGE_RED,
							Color::RED,
						];

						let color = if is_rare {
							Color::hsl(rng.gen_range(150.0..330.0), rng.gen_range(0.8..1.0), rng.gen_range(0.4..0.6))
						} else {
							COLORS[rng.gen_range(0..(COLORS.len()))]
						};

						let leaves_material = material_assets.add(FoliageMaterial {
							color,
							sss: true,
							..default()
						});

						commands.entity(child)
						.remove::<Handle<StandardMaterial>>()
						.insert(leaves_material.clone());
					}
					i if i.contains("Cap") => {
						// TODO: Remember material
						let stem_material = material_assets.add(FoliageMaterial {
							color: Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.8..1.0), rng.gen_range(0.4..0.6)),
							..default()
						});

						commands.entity(child)
						.remove::<Handle<StandardMaterial>>()
						.insert(stem_material.clone());
					}
					i if i.contains("Stem") => {
						// TODO: Remember material
						let stem_material = material_assets.add(FoliageMaterial {
							color: Color::rgb(0.8, 0.8, 0.8), //Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
							..default()
						});

						commands.entity(child)
						.remove::<Handle<StandardMaterial>>()
						.insert(stem_material.clone());
					}
					_ => {}
				}
			}
		}

	}
}

#[derive(Component)]
pub struct Bush;

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct Ingridient {
	pub main_effect: MainEffect,
	pub side_effect: SideEffect,
}
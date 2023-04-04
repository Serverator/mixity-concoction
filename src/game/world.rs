use bevy::{math::Vec3Swizzles, scene::SceneInstance, gltf::Gltf};

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
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
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
				scene: gltfs.get(&assets.tree_gltf).unwrap().scenes[0].clone(),
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
	name_query: Query<&Name, With<Handle<StandardMaterial>>>,
	children_query: Query<&Children>,
	tree_query: Query<(Entity,&SceneInstance),(Or<(With<Tree>, With<Bush>)>, Or<(Without<SceneLoaded>, (With<SceneLoaded>, Changed<SceneInstance>))>)>
) {
	let mut rng = thread_rng();

	for (entity,scene) in &tree_query {
		if !scene_manager.instance_is_ready(**scene) {
			continue;
		}
		commands.entity(entity).insert(SceneLoaded);

        const COLORS: &[Color] = &[
            Color::GREEN,
            Color::LIME_GREEN,
            Color::YELLOW_GREEN,
            Color::ORANGE,
            Color::ORANGE_RED,
            Color::RED,
        ];

		let leaves_material = material_assets.add(FoliageMaterial {
			color: COLORS[rng.gen_range(0..(COLORS.len()))], //Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
			..default()
		});

		// Iterate by children names inside GLTF to set additional components/materials
		// Probably really bad thing to do, but idk how to do it properly
		for child in children_query.iter_descendants(entity) {
			if let Ok(name) = name_query.get(child) {
				match name.as_str() {
					i if i.contains("Trunk") => {}
					i if i.contains("Leaves") => {
						commands.entity(child)
							.remove::<Handle<StandardMaterial>>()
							.insert(leaves_material.clone());
					}
					_ => {}
				}
			}
		}

	}
}

#[derive(Component)]
pub struct Bush;

fn spawn_bushes(
	mut command: Commands,
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
	
	let mut rng = thread_rng();
	for i in 0..1500 {
		let position = Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));
		if position.length_squared() < 6.0 {
			continue;
		}

		command.spawn((
            Bush,
			Name::new(format!("Bush {i}")),
            SceneBundle {
				scene: gltfs.get(&assets.bush_gltf).unwrap().scenes[rng.gen_range(0..=1)].clone(),
				transform: Transform::from_translation(position.extend(0.0).xzy()).with_scale(Vec3::splat(rng.gen_range(1.3..1.8))).with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
				..default()
			},
		));
	}
}
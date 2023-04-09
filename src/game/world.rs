use bevy::scene::SceneInstance;

use crate::{prelude::*, assets::{SHADOW_BUNDLE, SpawnableArchetype, Spawnable, SceneInstanceReady}, game::ingredient::{Ingredient, IngredientType}};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
		.init_resource::<OccupiedSpawnSpace>()
		.add_systems((
			init_world,
			spawn_spawnables,
				).in_schedule(OnEnter(GameState::GeneratingWorld))
		)
		.add_system(
			check_if_finished
				.in_set(OnUpdate(GameState::GeneratingWorld))
		);
		// .add_systems((
		// 	set_materials_to_spawnables,
		// 		).in_set(OnUpdate(GameState::InGame))
		// );
	}
}

#[derive(Resource, Default, Debug)]
pub struct OccupiedSpawnSpace(Vec<(Vec2,f32)>);

/// Added to the entities that represent shadows
#[derive(Component, Clone, Copy)]
pub struct Shadow;

#[derive(Component, Clone, Debug)]
pub struct SpawnableInstance {
	pub handle: Handle<Spawnable>,
	pub size: f32,
	pub rare: bool,
	//pub archetype: SpawnableArchetype,
}

fn init_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<FoliageMaterial>>,
	mut standard_mat: ResMut<Assets<StandardMaterial>>,
) {
	
	let plane_mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 });

	// Plane
	commands.spawn((
		Name::new("Main Plane"),
		Collider::from_bevy_mesh(&plane_mesh, &ComputedColliderShape::TriMesh).unwrap(),
		MaterialMeshBundle::<FoliageMaterial> {
			mesh: meshes.add(plane_mesh),
			transform: Transform::from_scale(Vec3::splat(600.0)),
			material: materials.add(FoliageMaterial {
				color: Color::YELLOW_GREEN, //Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
				..default()
			}),
			..default()
		},
		RigidBody::Fixed,
		CollisionGroups::new(Group::GROUP_1, Group::GROUP_1 | Group::GROUP_3),
	));

	// sphere
	commands.spawn((
		Name::new("Ball"),
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::UVSphere {
				radius: 0.4,
				..default()
			})),
			material: standard_mat.add(StandardMaterial {
				base_color: Color::WHITE,
				..default()
			}),
			transform: Transform::from_xyz(1.5, 5.0, 1.5),//.with_scale(Vec3::splat(3.0)),
			..default()
		},
		RigidBody::Dynamic,
		//Velocity::default(),
		Collider::ball(0.4),
		ColliderMassProperties::Density(0.05),
		Restitution::new(0.70),
		GravityScale(1.3),
		Friction::new(0.2),
		Dominance::group(-10),
		//RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_1 | Group::GROUP_3, Group::GROUP_1 | Group::GROUP_3),
	));

	// Light
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			shadows_enabled: false,
			..default()
		},
		transform: Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, 0.33, 0.404)),
		..default()
	});

	// Ambient light
	commands.insert_resource(AmbientLight {
		color: Color::ALICE_BLUE,
		brightness: 0.15,
	});
}

fn check_if_finished(
	unloaded_scenes: Query<(),(With<SceneInstance>, Without<SceneInstanceReady>)>,
	mut next_state: ResMut<NextState<GameState>>
) {
	if unloaded_scenes.is_empty() {
		info!("World generation finished!");
		next_state.set(GameState::InGame);
	}
}

/// Set spawn spawnable objects
fn spawn_spawnables(
	mut commands: Commands,
	mut occupied_space: ResMut<OccupiedSpawnSpace>,
	spawnable_assets: Res<Assets<Spawnable>>,
) {
	const SPAWN_SIZE: f32 = 200.0;

	let mut rng = rand::thread_rng();

	let spawnables = spawnable_assets.iter().collect::<Vec<_>>();
	let weights = spawnables.iter().map(|s| s.1.spawn_weight).collect::<Vec<_>>();

	let choose_spawnable = Choices { choices: &spawnables, weights: Some(&weights) };

	let collection = commands.spawn((
		Name::new("Vegetation collection"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	)).id();

	for i in 0..12000 {
		let Some((spawnable_handle,spawnable)) = choose_spawnable.get_random(&mut rng) else {
			warn!("Couldn't randomly choose spawnable from assets!");
			continue;
		};

		let position = Vec2::new(rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE), rng.gen_range(-SPAWN_SIZE..SPAWN_SIZE));

		let is_rare = rng.gen_bool(0.005);

		let relative_scale = if is_rare {
			rng.gen_range(1.0..2.0)
		} else {
			rng.gen_range(0.7..1.35)
		};

		if position.length_squared() < 12.0 || is_occupied(position, spawnable.size * relative_scale, &occupied_space) {
			continue;
		}

		let mut handle = Handle::<Spawnable>::weak(*spawnable_handle);
		handle.make_strong(&spawnable_assets);

		// Set space as occupied
		occupied_space.0.push((position, spawnable.size * relative_scale));

		let mut entity = commands.spawn((
			RigidBody::Fixed,
			SpawnableInstance {
				handle,
				rare: is_rare,
				size: relative_scale,
				//archetype: spawnable.archetype,
			},
			SceneBundle {
				scene: spawnable.scene.clone(),
				transform: Transform::from_translation(Vec3::new(position.x,0.0,position.y))
					.with_scale(Vec3::splat(relative_scale))
					.with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
				..default()
			},
			CollisionGroups::new(Group::GROUP_1, Group::GROUP_1 | Group::GROUP_3),
			// Applies materials to the spawned scene
			NamedMaterials::generate_materials(spawnable.archetype, is_rare, &mut rng),
		));

		match spawnable.archetype {
    		SpawnableArchetype::Tree => {
				entity.insert((
					Name::new(format!("Tree {i}")),
				));
			},
    		SpawnableArchetype::Bush => {
				entity.insert((
					Name::new(format!("Bush {i}")),
				));
			},
    		SpawnableArchetype::Mushroom => {
				entity.insert((
					Ingredient::generate_random_ingredient(&mut rng, IngredientType::Mushroom, is_rare),
					Name::new(format!("Mushroom {i}")),
				));
			},
		}

		// Collider
		if let Some(collider) = &spawnable.collider {
			entity.insert(collider.clone());
		}

		// Add shadow to entity
		entity.with_children(|commands| {
			commands.spawn((
				Name::new("Shadow"),
				Shadow,
				SHADOW_BUNDLE.get().unwrap().clone(),
				Transform::from_xyz(0.0,0.02,0.0).with_scale(Vec3::splat(spawnable.size)),
				GlobalTransform::default(),
				VisibilityBundle::default(),
			));
		});

		let entity = entity.id();
		commands.entity(collection).add_child(entity);
	}

	fn is_occupied(position: Vec2, size: f32, occupied_space: &OccupiedSpawnSpace) -> bool {
		occupied_space.0.iter().any(|(occupied_pos,occupied_size)| {
			let distance = Vec2::length_squared(position - *occupied_pos);
			
			distance < (size * occupied_size)
		})
	}
}
use bevy::{math::Vec3Swizzles, scene::SceneInstance, gltf::Gltf, pbr::NotShadowReceiver, render::{view::RenderLayers, once_cell::sync::OnceCell, camera::ScalingMode}, ecs::system::EntityCommands, core_pipeline::{fxaa::Fxaa, clear_color::ClearColorConfig}};


use crate::{prelude::*, assets::{SHADOW_BUNDLE, SpawnableCollection, SpawnableType}, game::effects::Ingridient};



pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
		.init_resource::<OccupiedSpawnSpace>()
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

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Bush;

#[derive(Component, Default, Debug)]
struct SceneLoaded;

#[derive(Resource, Default, Debug)]
struct OccupiedSpawnSpace(Vec<(Vec2,f32)>);

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
	));

	// // sphere
	// commands.spawn((
	// 	PbrBundle {
	// 		mesh: meshes.add(Mesh::from(shape::UVSphere {
	// 			radius: 0.5,
	// 			..default()
	// 		})),
	// 		material: standard_mat.add(StandardMaterial {
	// 			base_color: Color::WHITE,
	// 			..default()
	// 		}),
	// 		//transform: Transform::from_xyz(1.5, 5.0, 1.5).with_scale(Vec3::splat(3.0)),
	// 		..default()
	// 	},
	// 	RenderLayers::layer(2),
	// ));

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

	//let cube = meshes.add(Mesh::from(shape::Cube::new(1.0)));

	// commands.spawn((
	// 	Name::new("Backpack"),
	// 	VisibilityBundle::default(),
	// 	TransformBundle::default(),
	// )).with_children(|commands| {
	// 	commands.spawn((
	// 		PbrBundle {
	// 			mesh: cube.clone(),
	// 			transform: 
	// 			..default()
	// 		},
	// 	));
	// });

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

// enum VegetationType {
// 	Tree,
// 	Bush,
// 	Mushroom,
// }

#[derive(Component, Clone, Copy)]
struct Rare;


#[derive(Component, Clone, Copy)]
struct Shadow;


fn spawn_vegetation(
	mut commands: Commands,
	mut occupied_space: ResMut<OccupiedSpawnSpace>,
	spawnables: Res<SpawnableCollection>,
) {
	const SPAWN_SIZE: f32 = 200.0;

	let mut rng = rand::thread_rng();

	let weights = spawnables.0.iter().map(|s| s.spawn_weight ).enumerate().collect::<Vec<_>>();
	let all_weights = weights.iter().fold(0.0, |acc, x| acc + x.1);

	let select_random_id = |rng: &mut ThreadRng| { 
		let mut random_weight = rng.gen_range(0.0..all_weights); 

		for (i, weight) in weights.iter() {
			random_weight -= weight;

			if random_weight < 0.0 {
				return *i;
			}
		}

		weights.len() - 1
	};

	let collection = commands.spawn((
		Name::new("Vegetation collection"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	)).id();

	for i in 0..10000 {
		let spawnable = &spawnables.0[select_random_id(&mut rng)];

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

		// Set space as occupied
		occupied_space.0.push((position, spawnable.size * relative_scale));

		let mut entity = commands.spawn((
			SceneBundle {
				scene: spawnable.scene.clone(),
				transform: Transform::from_translation(Vec3::new(position.x,0.0,position.y))
					.with_scale(Vec3::splat(relative_scale))
					.with_rotation(Quat::from_rotation_y(rng.gen_range(-PI..PI))),
				..default()
			},
		));

		if is_rare {
			entity.insert(Rare);
		}

		match spawnable.stype {
    		SpawnableType::Tree => {
				entity.insert((
					Tree,
					Name::new(format!("Tree {i}")),
					Collider::cylinder(4.0, 0.7),
				));
			},
    		SpawnableType::Bush => {
				entity.insert((
					Bush,
					Name::new(format!("Bush {i}")),
					Collider::cylinder(4.0, 0.2),
				));
			},
    		SpawnableType::Mushroom => {
				entity.insert((
					Ingridient::default(),
					Name::new(format!("Mushroom {i}")),
					Collider::cylinder(4.0, 0.1),
				));
			},
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



#[allow(clippy::type_complexity)]
fn init_loaded_scenes(
	mut commands: Commands,
	scene_manager: Res<SceneSpawner>,
	mut material_assets: ResMut<Assets<FoliageMaterial>>,
	name_query: Query<&Name, With<Handle<StandardMaterial>>>,
	children_query: Query<&Children>,
	tree_query: Query<(Entity,&SceneInstance,Option<&Rare>),(Or<(With<Tree>, With<Bush>, With<Ingridient>)>, Or<(Without<SceneLoaded>, (With<SceneLoaded>, Changed<SceneInstance>))>)>
) {
	let mut rng = thread_rng();

	for (entity,scene,rare) in &tree_query {
		if !scene_manager.instance_is_ready(**scene) {
			continue;
		}
		commands.entity(entity).insert(SceneLoaded);

		let is_rare = rare.is_some(); 

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
							//Color::GREEN,
							Color::LIME_GREEN,
							Color::YELLOW_GREEN,
							Color::ORANGE,
							Color::ORANGE_RED,
							//Color::RED,
						];

						let color = if is_rare {
							Color::hsl(rng.gen_range(150.0..330.0), rng.gen_range(0.8..1.0), rng.gen_range(0.4..0.6))
						} else {
							//let t = 1.0 - rng.gen::<f32>().powf(2.0);
							//Color::hsl(lerp(35.0..=100.0, t), rng.gen_range(0.75..0.90), rng.gen_range(0.45..0.5))
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
						let color = if is_rare {
							Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.8..1.0), rng.gen_range(0.45..0.65))
						} else {
							Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.5..0.65), rng.gen_range(0.35..0.55))
						};
						let stem_material = material_assets.add(FoliageMaterial {
							color,
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
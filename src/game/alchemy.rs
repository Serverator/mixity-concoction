use bevy::{
	gltf::Gltf,
	math::{Vec3Swizzles, Vec4Swizzles},
	render::view::RenderLayers,
};

use crate::{
	assets::{CalculatedColliders, SceneInstanceReady},
	prelude::*,
};

use super::{
	backpack::InventoryCamera,
	effects::{generate_effects_from_qp, generate_qp_from_ingredients, ActiveEffects},
	ingredient::{Grind, Ingredient},
	items::{DroppedItem, Grabber, Item, ItemSize, Potion},
};

pub struct AlchemyPlugin;
impl Plugin for AlchemyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::InGame),
			init_alchemy_table,
		)
		.add_systems(
			Update,
			(
				// Update in game state
				get_cauldron_liquid_material,
				check_mortar_crushing,
				rotate_head,
				(
					calculate_com,
					(
						(check_cauldroned, consume_cauldroned).chain(),
						(check_eaten, consume_eaten).chain()
					)
				).chain(),
				mash_ingredient,
				change_color,
				spawn_new_bottle,
			).run_if(in_state(GameState::InGame))
		)
		.register_type::<Mortar>()
		.register_type::<Cauldron>();
	}
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct Eaten;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct Cauldroned;

#[derive(Component)]
pub struct AlchemyTable;

#[derive(Component, Reflect)]
pub struct Mortar(bool);

#[derive(Component)]
pub struct Pestle;

#[derive(Component, Default, Reflect)]
pub struct Cauldron(SmallVec<[Ingredient; 6]>);

#[derive(Component)]
pub struct PlayerHead;

#[derive(Bundle)]
pub struct SecondWorldBundle {
	pub render_layer: RenderLayers,
	pub collision_group: CollisionGroups,
}

impl Default for SecondWorldBundle {
	fn default() -> Self {
		Self {
			render_layer: RenderLayers::layer(2),
			collision_group: CollisionGroups::new(Group::GROUP_2, Group::GROUP_2),
		}
	}
}

fn init_alchemy_table(
	mut commands: Commands,
	game_assets: Res<GameAssets>,
	calculated_colliders: Res<CalculatedColliders>,
	gltfs: Res<Assets<Gltf>>,
) {
	// Alchemy table
	commands.spawn((
		Name::new("Alchemy Table"),
		AlchemyTable,
		RigidBody::KinematicPositionBased,
		SceneBundle {
			scene: game_assets.table_scene.clone(),
			transform: Transform::from_xyz(5.0, -3.0, 0.0).with_scale(Vec3::splat(3.0)),
			..default()
		},
		Restitution::new(0.2),
		NamedMaterials(smallvec![NamedMaterial::new(
			"Table",
			Color::rgb(0.4, 0.2, 0.04)
		)]),
		Collider::compound(vec![(
			Vec3::Y * 0.53,
			Quat::IDENTITY,
			Collider::cuboid(1.04, 0.05, 0.5),
		)]),
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2, Group::GROUP_2),
	));

	commands.spawn((
		Mortar(false),
		Name::new("Mortar"),
		Item::AlchemyTool,
		ItemSize::new(2.5, false),
		RigidBody::Dynamic,
		Velocity::default(),
		SceneBundle {
			scene: game_assets.mortar_scene.clone(),
			transform: Transform::from_xyz(3.0, 5.0, 0.0).with_scale(Vec3::splat(2.5)),
			..default()
		},
		Damping {
			angular_damping: 0.5,
			linear_damping: 0.5,
		},
		ColliderMassProperties::Density(2.0),
		NamedMaterials(smallvec![NamedMaterial::new("Mortar", Color::GRAY)]),
		LockedAxes::TRANSLATION_LOCKED_Z,
		calculated_colliders.mortar_collider.clone(),
		//Collider::compound(vec![(Vec3::ZERO,Quat::IDENTITY,Collider::cuboid(0.4, 0.4, 0.4))]),
		SecondWorldBundle::default(),
	));

	commands.spawn((
		Pestle,
		Name::new("Pestle"),
		Item::AlchemyTool,
		ItemSize::new(2.5, false),
		RigidBody::Dynamic,
		Velocity::default(),
		SceneBundle {
			scene: game_assets.pestle_scene.clone(),
			transform: Transform::from_xyz(5.0, 5.0, 0.0).with_scale(Vec3::splat(2.5)),
			..default()
		},
		Damping {
			angular_damping: 0.5,
			linear_damping: 0.5,
		},
		ColliderMassProperties::Density(1.5),
		NamedMaterials(smallvec![NamedMaterial::new("Pestle", Color::GRAY)]),
		LockedAxes::TRANSLATION_LOCKED_Z,
		Collider::compound(vec![(
			Vec3::Y * 0.05,
			Quat::IDENTITY,
			Collider::capsule(Vec3::ZERO, Vec3::Y * 0.3, 0.05),
		)]),
		SecondWorldBundle::default(),
	));

	commands.spawn((
		Pestle,
		Name::new("Cauldron"),
		Cauldron::default(),
		Item::AlchemyTool,
		ItemSize::new(2.5, false),
		RigidBody::Dynamic,
		Velocity::default(),
		SceneBundle {
			scene: game_assets.cauldron_scene.clone(),
			transform: Transform::from_xyz(7.0, 5.0, 0.0).with_scale(Vec3::splat(2.5)),
			..default()
		},
		Damping {
			angular_damping: 0.5,
			linear_damping: 0.5,
		},
		ColliderMassProperties::Density(0.8),
		NamedMaterials(smallvec![NamedMaterial::new("Cauldron", Color::DARK_GRAY)]),
		LockedAxes::TRANSLATION_LOCKED_Z,
		calculated_colliders.cauldron_collider.clone(),
		//Collider::compound(vec![(Vec3::ZERO,Quat::IDENTITY,Collider::cuboid(0.4, 0.4, 0.4))]),
		SecondWorldBundle::default(),
	));

	// Player head
	commands.spawn((
		Name::new("Head"),
		PlayerHead,
		//RigidBody::KinematicPositionBased,
		SceneBundle {
			scene: game_assets.player_head_scene.clone(),
			transform: Transform::from_xyz(0.0, 4.0, -1.0).with_scale(Vec3::splat(3.5)),
			..default()
		},
		NamedMaterials(smallvec![
			NamedMaterial::new("Skin", Color::rgb(0.988, 0.812, 0.718)),
			NamedMaterial::new("Teeth", Color::WHITE),
			NamedMaterial::new("Hair", Color::rgb(0.329, 0.204, 0.141)),
			NamedMaterial::new("Mouth", Color::rgb(0.7, 0.2, 0.141)),
			NamedMaterial::new("Glass", Color::rgb(0.902, 0.996, 1.0)),
			NamedMaterial::new("Metal", Color::rgb(0.988, 0.788, 0.333)),
			NamedMaterial::new("Leather", Color::rgb(0.612, 0.294, 0.188)),
		]),
		RenderLayers::layer(2),
		//CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
	));

	spawn_empty_bottle(&mut commands, &game_assets, &gltfs, &calculated_colliders);
}

pub fn spawn_empty_bottle(
	commands: &mut Commands,
	game_assets: &GameAssets,
	gltfs: &Assets<Gltf>,
	colliders: &CalculatedColliders,
) {
	let gltf = gltfs.get(&game_assets.potions_gltf).unwrap();
	let mut rng = thread_rng();
	let i = rng.gen_range(0..gltf.scenes.len());

	let potion = &gltf.scenes[i];
	let collider = colliders.potions[i].clone();

	commands.spawn((
		Name::new("Potion Bottle"),
		SceneBundle {
			scene: potion.clone(),
			transform: Transform::from_xyz(4.0, 6.0, 0.0),
			..default()
		},
		Item::Potion(Potion::Empty),
		RigidBody::Dynamic,
		collider,
		LockedAxes::TRANSLATION_LOCKED_Z,
		Damping {
			angular_damping: 0.5,
			linear_damping: 0.5,
		},
		NamedMaterials(smallvec![
			NamedMaterial::new("Potion", Color::rgb(0.8, 0.8, 0.95)),
			NamedMaterial::new("Cork", Color::rgb(0.6, 0.4, 0.0)),
		]),
		SecondWorldBundle::default(),
		ItemSize::new(1.0, false),
		Velocity::default(),
	));
}

#[derive(Resource, Default)]
struct CauldronLiquidMaterial(Handle<FoliageMaterial>);

fn get_cauldron_liquid_material(
	mut commands: Commands,
	cauldron: Query<Entity, (With<Cauldron>, Added<SceneInstanceReady>)>,
	children_query: Query<&Children>,
	mesh_query: Query<&Name, With<Handle<Mesh>>>,
	mut materials: ResMut<Assets<FoliageMaterial>>,
) {
	let Ok(cauldron) = cauldron.get_single() else {
		return;
	};

	for child in children_query.iter_descendants(cauldron) {
		let Ok(name) = mesh_query.get(child) else { continue; };

		if name.contains("Liquid") {
			let material = materials.add(FoliageMaterial {
				color: Color::rgb(0.6, 0.8, 0.97),
				sss: true,
			});

			commands
				.entity(child)
				.remove::<Handle<StandardMaterial>>()
				.insert(material.clone());

			commands.insert_resource(CauldronLiquidMaterial(material.clone()))
		}
	}
}

/// Will not actually affect physics.
/// Just wanted to calculate it once instead of 500 times per frame :)
///
/// Idk how expensive it is, but I didn't want to risk it.
#[derive(Component)]
struct CenterOfMass(Vec3);

/// Center of mass calculation

fn calculate_com(
	mut commands: Commands,
	items: Query<(Entity, &Collider), (With<Item>, Or<(Changed<Collider>, Without<CenterOfMass>)>)>,
) {
	for (item, collider) in &items {
		let local_com = Vec3::from(collider.raw.mass_properties(1.0).local_com);

		commands.entity(item).insert(CenterOfMass(local_com));
	}
}

// fn shrink_near_consumable(
// 	head_query: Query<&Transform, With<PlayerHead>>,
// 	mut item_query: Query<(Entity, &GlobalTransform, &mut ItemSize, &CenterOfMass, &Item)>,
// 	grabber_query: Query<&Grabber>,
// ) {
// 	let grabber = grabber_query.single();
// 	let head_transform = head_query.single();

// 	for (entity, transform, mut item_size, local_com, _) in item_query.iter_mut() {
// 		item_size.mouth_mult = if grabber.grabbed_entity == Some(entity) {
// 			let global_com = transform.compute_matrix().mul_vec4(local_com.0.extend(1.0)).xyz();
// 			let length = (head_transform.translation - global_com).length_squared();
// 			(length / 1.5).min(1.0).max(0.6)
// 		} else {
// 			1.0
// 		}
// 	}
// }

fn rotate_head(
	mut head: Query<&mut Transform, With<PlayerHead>>,
	windows: Query<&Window>,
	camera: Query<(&Camera, &GlobalTransform), With<InventoryCamera>>,
	rapier_context: Res<RapierContext>,
	time: Res<Time>,
) {
	let window = windows.single();
	let (camera, camera_gt) = camera.single();
	let Some(mouse_position) = window.cursor_position() else { return; };

	let Some(ray) = camera.viewport_to_world(camera_gt, mouse_position) else { return; };

	let filter = QueryFilter::only_dynamic()
		.exclude_sensors()
		.groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	let Some(distance) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter).map(|a| a.1).or(ray.intersect_plane(Vec3::ZERO, Vec3::Z)) else { return; };
	let point = ray.get_point(distance);

	let mut head_transform = head.single_mut();

	let forward = (point - (head_transform.translation + Vec3::Y * 0.3)).normalize();
	let right = Vec3::Y.cross(forward).normalize();
	let up = forward.cross(right);
	let desired_rotation = Quat::slerp(
		Quat::IDENTITY,
		Quat::from_mat3(&Mat3::from_cols(right, up, forward)),
		0.5,
	);

	head_transform.rotation = Quat::slerp(
		head_transform.rotation,
		desired_rotation,
		(1.0 - 0.001f64.powf(time.delta_seconds_f64())) as f32,
	)
}

const HEAD_OFFSET: Vec3 = Vec3::new(-0.05, 0.05, 0.0);

/// Animates and checks animation state.
/// On the end of animation does the thing. :)
fn consume_eaten(
	mut commands: Commands,
	head_single: Query<&Transform, With<PlayerHead>>,
	mut eaten_query: Query<
		(Entity, &mut ItemSize, &mut Transform, &Item),
		(With<Eaten>, Without<PlayerHead>),
	>,
	time: Res<Time>,
	game_assets: Res<GameAssets>,
	mut active_effects: ResMut<ActiveEffects>,
) {
	if eaten_query.is_empty() {
		return;
	}

	let head_transform = head_single.single();

	let mut rng = thread_rng();

	for (entity, mut item_size, mut transform, item) in &mut eaten_query {
		if item_size.current_size() < 0.08 {
			match item {
				Item::AlchemyTool => {
					commands
						.entity(entity)
						.remove::<ColliderDisabled>()
						.remove::<Eaten>();

					transform.translation = Vec3::Y * 15.0;
					item_size.shrinking = false;

					commands.spawn(AudioBundle {
						source: game_assets.blah_sound.clone(),
						..default()
					});
				}
				Item::Ingredient => {
					commands.entity(entity).despawn_recursive();

					if rng.gen_bool(0.05) {
						commands.spawn(AudioBundle {
							source: game_assets.delishs_sound.clone(),
							..default()
						});
					} else {
						commands.spawn(AudioBundle {
							source: game_assets.eat_sound.clone(),
							..default()
						});
					}
				}
				Item::Potion(potion) => {
					if let Potion::Filled {
						ingridients,
						color: _,
					} = potion
					{
						let (quality, purity) = generate_qp_from_ingredients(ingridients);
						let effects = generate_effects_from_qp(quality, purity);

						for effect in effects {
							active_effects.0.push(effect);
						}
						commands.spawn(AudioBundle {
							source: game_assets.drink_sound.clone(),
							..default()
						});
					}

					commands.entity(entity).despawn_recursive();
				}
			}
		} else {
			transform.translation = Vec3::lerp(
				transform.translation,
				head_transform.translation,
				(1.0 - 0.0001f64.powf(time.delta_seconds_f64())) as f32,
			)
		}
	}
}

fn spawn_new_bottle(
	mut commands: Commands,
	game_assets: Res<GameAssets>,
	gltfs: Res<Assets<Gltf>>,
	calculated_colliders: Res<CalculatedColliders>,
	potion: Query<&Item, Without<DroppedItem>>,
) {
	if potion
		.iter()
		.filter(|a| matches!(a, Item::Potion(Potion::Empty)))
		.count() == 0
	{
		spawn_empty_bottle(&mut commands, &game_assets, &gltfs, &calculated_colliders);
	}
}

fn change_color(
	cauldron: Query<&Cauldron>,
	cauldron_mat: Option<Res<CauldronLiquidMaterial>>,
	mut materials: ResMut<Assets<FoliageMaterial>>,
	time: Res<Time>,
) {
	let Some(Some(material)) = cauldron_mat.map(|res| materials.get_mut(&res.0)) else { return; };
	let cauldron = cauldron.single();

	let color = Vec4::from(material.color.as_rgba_f32()).xyz();

	let desired_color = if cauldron.0.is_empty() {
		Vec3::new(0.6, 0.8, 0.97)
	} else {
		cauldron
			.0
			.iter()
			.map(|i| Vec4::from(i.color.as_rgba_f32()).xyz())
			.sum::<Vec3>()
			/ cauldron.0.len() as f32
	};

	let lerped = color.lerp(desired_color, 1.0 - 0.3f32.powf(time.delta_seconds()));

	material.color = Color::rgb(lerped.x, lerped.y, lerped.z);
}

fn consume_cauldroned(
	mut commands: Commands,
	mut cauldron_single: Query<(&GlobalTransform, &mut Cauldron)>,
	mut cauldroned_query: Query<
		(
			Entity,
			&mut ItemSize,
			&mut Transform,
			&mut Item,
			Option<&mut NamedMaterials>,
		),
		(With<Cauldroned>, Without<Cauldron>),
	>,
	ingridient_query: Query<&Ingredient, (Without<DroppedItem>, With<Item>)>,
	time: Res<Time>,
	_cauldron_mat: Option<Res<CauldronLiquidMaterial>>,
	_game_assets: Res<GameAssets>,
) {
	if cauldroned_query.is_empty() {
		return;
	}

	let (cauldron_transform, mut cauldron) = cauldron_single.single_mut();

	let cauldron_pos = cauldron_transform
		.compute_matrix()
		.mul_vec4(Vec4::new(0.0, 0.3, 0.0, 1.0))
		.xyz();

	for (entity, mut item_size, mut transform, mut item, named_mats) in &mut cauldroned_query {
		if item_size.current_size() < 0.08 {
			match item.as_mut() {
				Item::AlchemyTool => {
					commands
						.entity(entity)
						.remove::<ColliderDisabled>()
						.remove::<Cauldroned>();

					transform.translation = Vec3::Y * 15.0;
					item_size.shrinking = false;

					//sound.play(game_assets.blah_sound.clone());
				}
				Item::Ingredient => {
					let ingridient = ingridient_query.get(entity).unwrap().clone();

					cauldron.0.push(ingridient);

					commands.entity(entity).despawn_recursive();
				}
				Item::Potion(potion) => {
					commands
						.entity(entity)
						.remove::<ColliderDisabled>()
						.remove::<Cauldroned>();
					item_size.shrinking = false;

					let color = if cauldron.0.is_empty() {
						Vec3::new(0.6, 0.8, 0.97)
					} else {
						cauldron
							.0
							.iter()
							.map(|i| Vec4::from(i.color.as_rgba_f32()).xyz())
							.sum::<Vec3>() / cauldron.0.len() as f32
					};

					let color = Color::rgb(color.x, color.y, color.z);

					// This is so bad, I'm so so sorry
					// I don't even know how half of my code works myself..
					named_mats.unwrap().0[0].material.color = color;

					let mut new_vec = smallvec![];
					std::mem::swap(&mut new_vec, &mut cauldron.0);

					*potion = Potion::Filled {
						ingridients: new_vec.into_vec(),
						color,
					};
				}
			}
		} else {
			transform.translation = Vec3::lerp(
				transform.translation,
				cauldron_pos,
				(1.0 - 0.0001f64.powf(time.delta_seconds_f64())) as f32,
			)
		}
	}
}

fn check_eaten(
	mut commands: Commands,
	head_query: Query<&Transform, With<PlayerHead>>,
	mut item_query: Query<
		(Entity, &GlobalTransform, &mut ItemSize, &CenterOfMass),
		(
			With<Item>,
			Without<Eaten>,
			Without<Cauldroned>,
			Without<DroppedItem>,
		),
	>,
	mut grabber_query: Query<&mut Grabber>,
	game_assets: Res<GameAssets>,
) {
	let mut grabber = grabber_query.single_mut();
	let head_transform = head_query.single();

	for (item_entity, global_transform, mut item_size, local_com) in &mut item_query {
		let global_com = global_transform
			.compute_matrix()
			.mul_vec4(local_com.0.extend(1.0))
			.xyz();

		let length = (head_transform.translation.xy().extend(0.0) + HEAD_OFFSET
			- global_com.xy().extend(0.0))
		.length_squared();

		if length > 0.8 {
			continue;
		}

		if grabber.grabbed_entity == Some(item_entity) {
			grabber.ungrab = true;
		}
		// Here we are considered "eaten"

		commands
			.entity(item_entity)
			.insert(Eaten)
			.insert(ColliderDisabled)
			.insert(Velocity::zero());
		item_size.shrinking = true;

		commands.spawn(AudioBundle {
			source: game_assets.suck_air_sound.clone(),
			..default()
		});

		continue;
	}
}

// Why another system? Cauldon ate itself and made a hole in space time. (Too lazy to fix)
fn check_cauldroned(
	mut commands: Commands,
	cauldron_query: Query<&GlobalTransform, With<Cauldron>>,
	mut item_query: Query<
		(
			Entity,
			&GlobalTransform,
			&mut ItemSize,
			&CenterOfMass,
			&Item,
			&mut Velocity,
		),
		(
			Without<Cauldron>,
			Without<Eaten>,
			Without<Cauldroned>,
			Without<DroppedItem>,
		),
	>,
	mut grabber_query: Query<&mut Grabber>,
	game_assets: Res<GameAssets>,
	time: Res<Time>,
) {
	let mut grabber = grabber_query.single_mut();
	let Ok(cauldron_gt) = cauldron_query.get_single() else { return; };
	let cauldron_pos = cauldron_gt
		.compute_matrix()
		.mul_vec4(Vec4::new(0.0, 0.45, 0.0, 1.0))
		.xyz();

	for (item_entity, global_transform, mut item_size, local_com, item, mut velocity) in
		&mut item_query
	{
		let global_com = global_transform
			.compute_matrix()
			.mul_vec4(local_com.0.extend(1.0))
			.xyz();

		let relative_vector = cauldron_pos.xy().extend(0.0) - global_com.xy().extend(0.0);
		let length = relative_vector.length_squared();

		if length > 0.50 {
			// || (relative_vector.y < 0.0 && length > 0.4) {
			continue;
		}

		match item {
			Item::Potion(Potion::Filled { .. }) | Item::AlchemyTool => {
				velocity.linvel += cauldron_gt.up() * time.delta_seconds() * 12.0;
				return;
			}
			_ => (),
		};

		if grabber.grabbed_entity == Some(item_entity) {
			grabber.ungrab = true;
		}

		// Here we are considered "CaUlDrOnEd"

		commands
			.entity(item_entity)
			.insert(Cauldroned)
			.insert(ColliderDisabled)
			.insert(Velocity::zero());
		item_size.shrinking = true;

		if let Item::Potion(_) = item {
			commands.spawn(AudioBundle {
				source: game_assets.filling_potion_sound.clone(),
				..default()
			});
		} else {
			commands.spawn(AudioBundle {
				source: game_assets.sploosh_sound.clone(),
				..default()
			});
		}

		continue;
	}
}

fn check_mortar_crushing(
	mut mortar_query: Query<(&GlobalTransform, &mut Mortar)>,
	pestle_query: Query<(&GlobalTransform, &Velocity, &CenterOfMass), With<Pestle>>,
	_time: Res<Time>,
) {
	const MORTAR_RADIUS: f32 = 0.4;

	for (mortar_transfrom, mut mortar) in &mut mortar_query {
		let mut is_crushing = false;
		let (_, mortar_rotation, _) = mortar_transfrom.to_scale_rotation_translation();

		for (pestle_transfrom, pestle_velocity, local_com) in &pestle_query {
			// Velocity relative to the mortar normal
			let relative_vel = mortar_rotation * pestle_velocity.linvel;
			// Return early if velocity perpendicular to the mortar normal is greater, than parallel velocity.
			if relative_vel.y > -1.0 || relative_vel.xz().length_squared() > relative_vel.y.powi(2)
			{
				continue;
			}
			let world_com = pestle_transfrom
				.compute_matrix()
				.mul_vec4(local_com.0.extend(1.0));
			// Pestle translation, relative to mortar (Y is mortar normal)
			let peslte_relative_translation =
				(mortar_transfrom.compute_matrix().inverse() * world_com).xyz();

			// Check if pestle is located within mortar radius
			if !(0.00..0.65).contains(&peslte_relative_translation.y)
				|| peslte_relative_translation.xz().length_squared() > MORTAR_RADIUS.powi(2)
			{
				continue;
			}

			is_crushing = true;
		}
		mortar.0 = is_crushing
	}
}

fn mash_ingredient(
	mut commands: Commands,
	mut ingridient_query: Query<
		(Entity, &GlobalTransform, &CenterOfMass, &mut Ingredient),
		(Without<DroppedItem>, With<Item>),
	>,
	mortar_query: Query<(&GlobalTransform, &CenterOfMass, &Mortar)>,
	game_assets: Res<GameAssets>,
	mut last_sound_time: Local<f32>,
	time: Res<Time>,
) {
	for (mortar_transform, mortar_local_com, _) in mortar_query.iter().filter(|m| m.2 .0) {
		for (entity, item_transform, item_local_com, mut ingredient) in &mut ingridient_query {
			let item_world_com = item_transform
				.compute_matrix()
				.mul_vec4(item_local_com.0.extend(1.0))
				.xyz();
			let mortar_world_com = mortar_transform
				.compute_matrix()
				.mul_vec4(mortar_local_com.0.extend(1.0))
				.xyz();
			let distance = mortar_world_com.distance_squared(item_world_com);

			if distance < 0.35 {
				if let Grind::Grinding(amount) = &mut ingredient.grind {
					*amount += time.delta_seconds();

					if *last_sound_time < time.elapsed_seconds() - 0.5 {
						*last_sound_time = time.elapsed_seconds();
						commands.spawn(AudioBundle {
							source: game_assets.grind_sound
										.choose(&mut thread_rng())
										.unwrap()
										.clone(),
							..default()
						});
					}

					if *amount > 0.35 {
						commands
							.entity(entity)
							.insert(game_assets.crushed_ingredient_scene.clone())
							.insert(NamedMaterials(smallvec![NamedMaterial::new(
								"Mashed",
								ingredient.color
							)]))
							.remove::<SceneInstanceReady>()
							.insert(Collider::compound(vec![(
								Vec3::new(0.0, 0.15, 0.0),
								Quat::IDENTITY,
								Collider::round_cone(0.132, 0.225, 0.05),
							)]));

						ingredient.grind = Grind::Grinded;
					}
				}
			}
		}
	}
}

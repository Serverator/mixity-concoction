use bevy::{math::Vec4Swizzles, render::view::RenderLayers};
use bevy_inspector_egui::egui::lerp;

use crate::{
	assets::{PickUpEvent, SceneInstanceReady, Spawnable},
	prelude::*,
};

use super::{
	backpack::InventoryCamera,
	effects::{ActiveEffects, EffectType},
	ingredient::Ingredient,
	player::Player,
	world::SpawnableInstance,
};

// #[derive(Default, Component, Debug, Clone, Copy)]
// pub struct InventoryItem {
// 	pub inventory: Option<Entity>,
// 	pub size: f32,
// }

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct ItemSize {
	pub shrinking: bool,
	pub size_mult: f32,
	pub mouth_mult: f32,
	current_size: f32,
}

impl Default for ItemSize {
	fn default() -> Self {
		Self::new(1.0, false)
	}
}

impl ItemSize {
	pub fn new(size_multiplier: f32, shrinking: bool) -> Self {
		ItemSize {
			size_mult: size_multiplier,
			shrinking,
			mouth_mult: 1.0,
			current_size: if shrinking {
				1.0 * size_multiplier
			} else {
				0.0
			},
		}
	}

	pub fn reset(&mut self) {
		self.current_size = if self.shrinking {
			1.0 * self.size_mult * self.mouth_mult
		} else {
			0.0
		}
	}

	pub fn current_size(&self) -> f32 {
		self.current_size
	}

	pub fn desired_size(&self) -> f32 {
		if self.shrinking {
			0.01
		} else {
			1.0 * self.size_mult * self.mouth_mult
		}
	}
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum Item {
	AlchemyTool,
	Ingredient,
	Potion(Potion),
}

#[derive(Default, Clone, Reflect, Debug)]
pub enum Potion {
	#[default]
	Empty,
	Filled {
		ingridients: Vec<Ingredient>,
		color: Color,
	},
}

// Thats a chunky boy :)
#[derive(Bundle, Clone)]
pub struct InventoryItemBundle {
	pub scene: Handle<Scene>,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
	pub visibility: Visibility,
	pub computed_visibility: ComputedVisibility,
	pub inventory_item: Item,
	pub item_size: ItemSize,
	pub locked_axes: LockedAxes,
	pub rigidbody: RigidBody,
	pub collider: Collider,
	pub velocity: Velocity,
	pub render_layer: RenderLayers,
	pub collision_group: CollisionGroups,
}

impl Default for InventoryItemBundle {
	fn default() -> Self {
		Self {
			scene: Default::default(),
			transform: Default::default(),
			global_transform: Default::default(),
			visibility: Default::default(),
			computed_visibility: Default::default(),
			inventory_item: Item::Ingredient,
			item_size: Default::default(),
			locked_axes: LockedAxes::TRANSLATION_LOCKED_Z,
			rigidbody: RigidBody::Dynamic,
			collider: Collider::default(),
			velocity: Velocity::default(),
			render_layer: RenderLayers::layer(2),
			collision_group: CollisionGroups::new(Group::GROUP_2, Group::GROUP_2),
		}
	}
}

#[derive(Component, Default)]
pub struct Grabber {
	pub grabbed_entity: Option<Entity>,
	pub ungrab: bool,
}

#[derive(Default, Component)]
pub struct DroppedItem;

pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::InGame),
			(
				init,
			)
		)
		.add_systems(
			Update, (
				pickup_entity,
				( drop_items, animate_size, item_grab_system ).chain()
			).run_if(in_state(GameState::InGame))
		)
		.add_systems(FixedUpdate, move_grabber)
		.register_type::<Item>()
		.register_type::<ItemSize>();
	}
}

fn init(mut commands: Commands) {
	commands.spawn((
		Grabber::default(),
		Name::new("Grabber"),
		TransformBundle::default(),
		RigidBody::Dynamic,
		GravityScale(0.0),
		Velocity::default(),
		CollisionGroups::new(Group::GROUP_5, Group::NONE),
		Collider::ball(0.15),
		Friction::new(0.0),
		ColliderMassProperties::Density(1.0),
		LockedAxes::ROTATION_LOCKED,
		Ccd::default(),
	));
}

fn animate_size(mut inventory_item_query: Query<(&mut Transform, &mut ItemSize)>, time: Res<Time>) {
	for (mut transform, mut item_size) in inventory_item_query.iter_mut() {
		item_size.current_size = lerp(
			item_size.current_size..=item_size.desired_size(),
			(1.0 - 0.0001f64.powf(time.delta_seconds_f64())) as f32,
		);

		transform.scale = Vec3::splat(item_size.current_size);
	}
}

fn drop_items(
	mut commands: Commands,
	mut inventory_item_query: Query<
		(Entity, &mut Transform, &Item, &mut ItemSize, &mut Velocity),
		(Without<Player>, Without<DroppedItem>),
	>,
	mut grabber_query: Query<&mut Grabber>,
	player_query: Query<&Transform, With<Player>>,
	_potion_query: Query<&Item>,
	game_assets: Res<GameAssets>,
) {
	let mut grabber = grabber_query.single_mut();

	for (dropped_item, mut transform, item, mut item_size, mut velocity) in
		inventory_item_query.iter_mut().filter(|(_, t, _, _, _)| {
			t.translation.y < -3.0 || t.translation.y > 8.5 || t.translation.x < -4.0
		}) {
		if grabber.bypass_change_detection().grabbed_entity == Some(dropped_item) {
			grabber.ungrab = true;
		}

		let mut rng = thread_rng();

		match item {
			Item::AlchemyTool | Item::Potion(Potion::Empty) => {
				*velocity = Velocity::default();

				item_size.reset();

				transform.rotation = Quat::IDENTITY;
				transform.translation = Vec3::new(rng.gen_range(3.0..6.0), 7.5, 0.0);

				//transform.scale = Vec3::splat(0.01);
			}
			Item::Ingredient | Item::Potion(Potion::Filled { .. }) => {
				let player_translation = player_query.single().translation;

				*velocity = Velocity {
					angvel: Vec3 {
						x: rng.gen_range(-8.0..8.0),
						y: rng.gen_range(-8.0..8.0),
						z: rng.gen_range(-8.0..8.0),
					},
					linvel: Vec3 {
						x: rng.gen_range(-6.0..6.0),
						y: 10.0,
						z: rng.gen_range(-6.0..6.0),
					},
				};

				item_size.reset();

				commands
					.entity(dropped_item)
					.insert(DroppedItem)
					.insert(RenderLayers::layer(0))
					.insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_1));

				transform.translation = player_translation + Vec3::Y;

				commands.spawn(AudioBundle {
					source: game_assets.drop_item_sound.clone(),
					..default()
				});
			}
		}
	}
}

pub fn pickup_entity(
	mut commands: Commands,
	player_query: Query<(&Transform, &ActionState<Action>), (With<Player>, Without<DroppedItem>)>,
	ingredient_query: Query<
		(
			Entity,
			&Transform,
			&Ingredient,
			&SpawnableInstance,
			&NamedMaterials,
		),
		Without<DroppedItem>,
	>,
	mut dropped_item_query: Query<
		(Entity, &mut Transform, &mut ItemSize, &mut Velocity),
		With<DroppedItem>,
	>,
	finder_query: Query<(Entity, &Name)>,
	child_query: Query<&Children>,
	spawnables: Res<Assets<Spawnable>>,
	game_assets: Res<GameAssets>,
	active_effects: Res<ActiveEffects>,
	mut hallucination_message: Local<bool>,
) {
	let Ok((player_transform, input)) = player_query.get_single() else {
		return;
	};

	if !input.just_pressed(Action::Use) {
		return;
	}

	#[derive(Debug)]
	enum Interactable {
		Pickupable,
		DroppedItem,
	}

	let closest_interactable = ingredient_query
		.iter()
		.map(|q| (q.0, q.1, Interactable::Pickupable))
		.chain(
			dropped_item_query
				.iter()
				.map(|q| (q.0, q.1, Interactable::DroppedItem)),
		)
		.filter_map(|c| {
			let transform = c.1;

			let distance_sq =
				(transform.translation - player_transform.translation).length_squared();
			if distance_sq < 4.0 {
				Some((distance_sq, c))
			} else {
				None
			}
		})
		.min_by(|a, b| a.0.total_cmp(&b.0))
		.map(|c| c.1);

	match closest_interactable {
		None => (),
		Some((entity, _, Interactable::Pickupable)) => {
			let Ok((entity,_ , ingredient, spawnable_instance, named_materials)) = ingredient_query.get(entity) else {
				return;
			};

			let Some(spawnable) = spawnables.get(&spawnable_instance.handle) else {
				warn!("Did not find spawnable in assets!");
				return;
			};

			let ingredient_info = spawnable.ingredient.as_ref().unwrap();

			// Hallucinations
			if let Some(hallucination) = active_effects.has_effect(EffectType::Hallucinations) {
				let mut rng = thread_rng();
				if rng.gen_bool(lerp(0.2..=0.5, hallucination.potency as f64)) {
					commands.entity(entity).despawn_recursive();
					if !*hallucination_message {
						commands.spawn(AudioBundle {
							source: game_assets.insanity_sound.clone(),
							..default()
						});
						*hallucination_message = true;
					} else if rng.gen_bool(0.1) {
						commands.spawn(AudioBundle {
							source: game_assets.wha_sound.clone(),
							..default()
						});
					}
					return;
				}
			}

			// Do something to original entity
			match &ingredient_info.pick_event {
				PickUpEvent::Destroy => commands.entity(entity).despawn_recursive(),
				PickUpEvent::Replace(scene) => {
					commands
						.entity(entity)
						.remove::<Handle<Scene>>()
						.remove::<SceneInstanceReady>()
						.remove::<Ingredient>()
						.insert(scene.clone());
				}
				PickUpEvent::RemoveNamedChild(name) => {
					commands.entity(entity).remove::<Ingredient>();

					for (child, child_name) in child_query
						.iter_descendants(entity)
						.filter_map(|child| finder_query.get(child).ok())
					{
						if child_name.contains(name) {
							commands.entity(child).despawn();
						}
					}
				}
			}

			let mut rng = thread_rng();

			let size = spawnable_instance.size;

			commands.spawn((
				Name::new(ingredient.name.clone()),
				InventoryItemBundle {
					scene: ingredient_info.inventory_scene.clone(),
					transform: Transform::from_xyz(
						rng.gen_range(-0.5..0.5),
						2.0 + rng.gen_range(-0.5..0.5),
						0.0,
					)
					.with_scale(Vec3::splat(0.01)),
					velocity: Velocity {
						angvel: Vec3::new(
							rng.gen_range(-5.0..5.0),
							rng.gen_range(-5.0..5.0),
							rng.gen_range(-5.0..5.0),
						),
						..default()
					},
					inventory_item: Item::Ingredient,
					item_size: ItemSize::new(size, false),
					collider: ingredient_info.collider.clone(),
					..default()
				},
				ingredient.clone(),
				named_materials.clone(),
				Damping {
					linear_damping: 0.5,
					angular_damping: 0.7,
				},
			));

			commands.spawn(AudioBundle {
				source: game_assets.pickup_sound.choose(&mut rng).unwrap().clone(),
				..default()
			});

			if ingredient.is_rare {
				commands.spawn(AudioBundle {
					source: game_assets.rare_sound.clone(),
					..default()
				});
			}
		}
		Some((entity, _, Interactable::DroppedItem)) => {
			let Ok((_,mut transform, mut item_size, mut velocity)) = dropped_item_query.get_mut(entity) else  {
				return;
			};

			let mut rng = thread_rng();

			// Hallucinations
			if let Some(hallucination) = active_effects.has_effect(EffectType::Hallucinations) {
				if rng.gen_bool(lerp(0.2..=0.5, hallucination.potency as f64)) {
					commands.entity(entity).despawn_recursive();
					if !*hallucination_message {
						commands.spawn(AudioBundle {
							source: game_assets.insanity_sound.clone(),
							..default()
						});
						*hallucination_message = true;
					} else if rng.gen_bool(0.1) {
						commands.spawn(AudioBundle {
							source: game_assets.wha_sound.clone(),
							..default()
						});
					}
					return;
				}
			}

			*velocity = Velocity {
				angvel: Vec3 {
					x: rng.gen_range(-5.0..5.0),
					y: rng.gen_range(-5.0..5.0),
					z: rng.gen_range(-5.0..5.0),
				},
				..default()
			};

			item_size.reset();

			commands
				.entity(entity)
				.remove::<DroppedItem>()
				.insert(RenderLayers::layer(2))
				.insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));

			transform.translation = Vec3::new(
				rng.gen_range(-0.5..0.5),
				2.0 + rng.gen_range(-0.5..0.5),
				0.0,
			);
			//transform.scale = Vec3::splat(0.01);

			commands.spawn(AudioBundle {
				source: game_assets.pickup_sound.choose(&mut rng).unwrap().clone(),
				..default()
			});
		}
	}
}

fn move_grabber(
	windows: Query<&Window>,
	_rapier_context: Res<RapierContext>,
	inventory_camera: Query<(&GlobalTransform, &Camera), With<InventoryCamera>>,
	mut grabber: Query<(&Transform, &mut Velocity, &Grabber)>,
) {
	let Ok((grabber_transform, mut velocity, grabber)) = grabber.get_single_mut() else { return; };

	if grabber.grabbed_entity.is_none() {
		return;
	}

	let Some(mouse_position) = windows.single().cursor_position() else { return; };

	let Ok((camera_transform,camera)) = inventory_camera.get_single() else {
		warn!("Couldn't find inventory camera!");
		return;
	};

	let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) else { return; };

	//let filter = QueryFilter::only_dynamic().exclude_sensors().groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	//let Some(distance) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter).map(|a| a.1).or(ray.intersect_plane(Vec3::ZERO, Vec3::Z)) else { return; };

	let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) else { return; };

	let intersect = ray.get_point(distance);

	velocity.linvel = (intersect - grabber_transform.translation).clamp_length_max(3.0) * 8.0;
}

fn item_grab_system(
	mut commands: Commands,
	item_query: Query<Entity, With<Item>>,
	parent_query: Query<&Parent>,
	rapier_context: Res<RapierContext>,
	inventory_camera: Query<(&GlobalTransform, &Camera), With<InventoryCamera>>,
	windows: Query<&Window>,
	input: Query<&ActionState<Action>>,
	transform_query: Query<&GlobalTransform, Without<Camera>>,
	mut grabber_query: Query<(Entity, &mut Transform, &mut Grabber)>,
) {
	let Ok((grabber_entity, mut grabber_transform, mut grabber)) = grabber_query.get_single_mut() else { return; };

	if input.single().just_released(Action::Click) || grabber.ungrab {
		if let Some(grabbed) = grabber.grabbed_entity {
			commands.entity(grabbed).remove::<ImpulseJoint>();
		}
		grabber.grabbed_entity = None;
		grabber.ungrab = false;
		return;
	}

	// let .. else go brrrr
	if !input.single().just_pressed(Action::Click) {
		return;
	}

	let Some(mouse_position) = windows.single().cursor_position() else { return; };

	let Ok((camera_transform,camera)) = inventory_camera.get_single() else {
		warn!("Couldn't find inventory camera!");
		return;
	};

	let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) else { return; };

	let filter = QueryFilter::only_dynamic()
		.exclude_sensors()
		.groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	let Some((entity,distance)) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter) else { return; };

	let i_entity;

	if let Ok(entity) = item_query.get(entity) {
		i_entity = entity;
	} else if let Some(entity) = parent_query
		.iter_ancestors(entity)
		.find_map(|parent| item_query.get(parent).ok())
	{
		i_entity = entity;
	} else {
		return;
	}

	let entity = i_entity;

	let Ok(grabbed_entity_transform) = transform_query.get(entity) else { return; };

	let hit_global_pos = ray.get_point(distance);
	let srt = grabbed_entity_transform.to_scale_rotation_translation();
	let hit_local_pos = grabbed_entity_transform
		.compute_matrix()
		.inverse()
		.mul_vec4(hit_global_pos.extend(1.0))
		.xyz() * srt.0;

	grabber_transform.translation = hit_global_pos;

	grabber.grabbed_entity = Some(entity);

	let joint = SphericalJointBuilder::new()
		.local_anchor1(Vec3::new(0.0, 0.01, 0.0))
		.local_anchor2(hit_local_pos);

	commands
		.entity(entity)
		.insert(ImpulseJoint::new(grabber_entity, joint));
}

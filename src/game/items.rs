use bevy::{render::view::RenderLayers, math::Vec4Swizzles};
use bevy_inspector_egui::egui::lerp;

use crate::{prelude::*, assets::{Spawnable, PickUpEvent, SceneInstanceReady}};

use super::{ingredient::Ingredient, world::SpawnableInstance, player::Player, backpack::InventoryCamera};

#[derive(Default, Component, Debug, Clone, Copy)]
pub struct InventoryItem {
	pub inventory: Option<Entity>,
	pub size: f32,
	pub current_size: f32,
}

#[derive(Component)]
pub struct Grabber(pub Option<Entity>);

#[derive(Default, Component)]
pub struct DroppedItem;

pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(( // Run on game start
				init,
					).in_schedule(OnEnter(GameState::InGame))
			)
			.add_systems(( // Update in game state
				drop_items,
				pickup_entity,
				item_grab_system,
				animate_size,
					).in_set(OnUpdate(GameState::InGame))
			)
			.add_system(move_grabber.in_base_set(CoreSet::FixedUpdate));
	}
}

fn init(
	mut commands: Commands,
) {
	commands.spawn((
		Grabber(None),
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

fn animate_size(
	mut inventory_item_query: Query<(&mut Transform, &mut InventoryItem)>,
	time: Res<Time>,
) {
	for (mut transform, mut inventory_item) in inventory_item_query.iter_mut().filter(|c| c.1.current_size != 1.0) {
		inventory_item.current_size = (inventory_item.current_size + time.delta_seconds() * 3.0).min(1.0);

		transform.scale = Vec3::splat(lerp(0.0..=inventory_item.size, inventory_item.current_size));
	}
}

fn drop_items(
	mut commands: Commands,
	mut inventory_item_query: Query<(Entity, &mut Transform, &mut InventoryItem, &mut Velocity)>,
	transform_query: Query<&Transform, Without<InventoryItem>>,
) {
	for (dropped_item, mut transform, mut inventory_item, mut velocity) in inventory_item_query.iter_mut().filter(|(_,t,_,_)| t.translation.y < -3.0) {
		let Some(inventory) = inventory_item.inventory else { continue; };
		info!("Oops dropped an item!");
		let Ok(parent_transfrom) = transform_query.get(inventory) else {
			error!("Couldn't find transform for parent entity {:?}! Destroying dropped item...", inventory_item.inventory);
			commands.entity(dropped_item).despawn();
			continue;
		};

		let mut rng = thread_rng();

		*velocity = Velocity {
			angvel: Vec3 { x: rng.gen_range(-8.0..8.0), y: rng.gen_range(-8.0..8.0), z: rng.gen_range(-8.0..8.0) },
			linvel: Vec3 { x: rng.gen_range(-6.0..6.0), y: 10.0, z: rng.gen_range(-6.0..6.0) }
		};


		inventory_item.inventory = None;
		inventory_item.current_size = 0.0;

		commands.entity(dropped_item)
			.insert(DroppedItem::default())
			.insert(RenderLayers::layer(0))
			.insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_1));

		transform.translation = parent_transfrom.translation + Vec3::Y;
		transform.scale = Vec3::splat(0.01);
	}
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn pickup_entity(
	mut commands: Commands,
	player_query: Query<(Entity, &Transform, &ActionState<Action>), (With<Player>, Without<DroppedItem>)>,
	ingredient_query: Query<(Entity, &Transform, &Ingredient, &SpawnableInstance, &NamedMaterials), Without<DroppedItem>>,
	mut dropped_item_query: Query<(Entity, &mut Transform, &mut InventoryItem, &mut Velocity), With<DroppedItem>>,
	finder_query: Query<(Entity, &Name)>,
	child_query: Query<&Children>,
	spawnables: Res<Assets<Spawnable>>,
) {
	let Ok((player_entity, player_transform, input)) = player_query.get_single() else {
		return;
	};

	if !input.just_pressed(Action::Use) {
		return;
	}

	#[derive(Debug)]
	enum Interactable<'a> {
		Pickupable((Entity, &'a Transform, &'a Ingredient, &'a SpawnableInstance, &'a NamedMaterials)),
		DroppedItem((Entity, &'a Transform, &'a InventoryItem, &'a Velocity)),
	}

	let closest_interactable = ingredient_query.iter()
		.map(Interactable::Pickupable)
		.chain(dropped_item_query.iter().map(Interactable::DroppedItem)) 
		.filter_map(|c| {
			let transform = match c {
    			Interactable::Pickupable((_,t,_,_,_)) => t,
    			Interactable::DroppedItem((_,t,_,_)) => t,
			};

			let distance_sq = (transform.translation - player_transform.translation).length_squared();
			if distance_sq < 4.0 {
				Some((distance_sq, c))
			} else {
				None
			}
		})
		.min_by(|a,b| a.0.total_cmp(&b.0))
		.map(|c| c.1);

	match closest_interactable {
		None => (),
		Some(Interactable::Pickupable((entity,_ , ingredient, spawnable_instance, named_materials))) => {

			let Some(spawnable) = spawnables.get(&spawnable_instance.handle) else {
				warn!("Did not find spawnable in assets!");
				return;
			};
		
			let ingredient_info = spawnable.ingredient.as_ref().unwrap();
		
			// Do something to original entity
			match &ingredient_info.pick_event {
				PickUpEvent::Destroy => commands.entity(entity).despawn_recursive(),
				PickUpEvent::Replace(scene) => {
					commands.entity(entity)
						.remove::<Handle<Scene>>()
						.remove::<SceneInstanceReady>()
						.remove::<Ingredient>()
						.insert(scene.clone());
				},
				PickUpEvent::RemoveNamedChild(name) => {
					commands.entity(entity)
						.remove::<Ingredient>();
		
					for (child,child_name) in child_query.iter_descendants(entity).filter_map(|child| finder_query.get(child).ok()) {
						if child_name.contains(name) {
							commands.entity(child).despawn_recursive();
						}
					}
				},
			}
		
			let mut rng = thread_rng();
		
			let size = spawnable_instance.size;
		
			let _entity = commands.spawn((
				Name::new(ingredient.name.clone()),
				SceneBundle {
					scene: ingredient_info.inventory_scene.clone(),
					transform: Transform::from_xyz(rng.gen_range(-0.5..0.5), 2.0 + rng.gen_range(-0.5..0.5), 0.0)
						.with_scale(Vec3::splat(0.01)),
					..default()
				},
				ingredient.clone(),
				named_materials.clone(),
				InventoryItem {
					size,
					inventory: Some(player_entity),
					..default() 
				},
				LockedAxes:: TRANSLATION_LOCKED_Z,
				RigidBody::Dynamic,
				RenderLayers::layer(2),
				CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
				Velocity {
					angvel: Vec3 { x: rng.gen_range(-5.0..5.0), y: rng.gen_range(-5.0..5.0), z: rng.gen_range(-5.0..5.0) },
					..default()
				},
				Damping {
    			    linear_damping: 0.5,
    			    angular_damping: 0.7,
    			},
				ingredient_info.collider.clone(),
			));
		}
		Some(Interactable::DroppedItem((entity,_,_,_))) => {
			let Ok((_,mut transform,mut ii,mut velocity)) = dropped_item_query.get_mut(entity) else  {
				return;
			};

			let mut rng = thread_rng();

			ii.inventory = Some(player_entity);
			ii.current_size = 0.0;

			*velocity = Velocity {
				angvel: Vec3 { x: rng.gen_range(-5.0..5.0), y: rng.gen_range(-5.0..5.0), z: rng.gen_range(-5.0..5.0) },
				..default()
			};

			commands.entity(entity)
				.remove::<DroppedItem>()
				.insert(InventoryItem {
					size: ii.size,
					inventory: Some(player_entity), 
					..default() 
				})
				.insert(RenderLayers::layer(2))
				.insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));

			transform.translation = Vec3::new(rng.gen_range(-1.0..1.0), 5.0 + rng.gen_range(-1.0..1.0), 0.0);
			transform.scale = Vec3::splat(0.01);
		}
	}
}

fn move_grabber (
	windows: Query<&Window>,
	rapier_context: Res<RapierContext>,
	inventory_camera: Query<(&GlobalTransform, &Camera), With<InventoryCamera>>,
	mut grabber: Query<(&Transform, &mut Velocity, &Grabber)>,	
) {
	let Ok((grabber_transform, mut velocity, grabber)) = grabber.get_single_mut() else { return; };

	if grabber.0.is_none() {
		return;
	}

	let Some(mouse_position) = windows.single().cursor_position() else { return; };

	let Ok((camera_transform,camera)) = inventory_camera.get_single() else {
		warn!("Couldn't find inventory camera!");
		return;
	};

	let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) else { return; };

	let filter = QueryFilter::only_dynamic().exclude_sensors().groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	let Some(distance) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter).map(|a| a.1).or(ray.intersect_plane(Vec3::ZERO, Vec3::Z)) else { return; };

	let intersect = ray.get_point(distance);

	velocity.linvel = (intersect - grabber_transform.translation).clamp_length_max(3.0) * 8.0;
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn item_grab_system(
	mut commands: Commands,
	ingridient_query: Query<Entity, With<InventoryItem>>,
	parent_query: Query<&Parent>,
	rapier_context: Res<RapierContext>,
	inventory_camera: Query<(&GlobalTransform, &Camera), With<InventoryCamera>>,
	windows: Query<&Window>,
	input: Query<&ActionState<Action>>,
	transform_query: Query<&GlobalTransform, Without<Camera>>,
	mut grabber_query: Query<(Entity, &mut Transform, &mut Grabber)>,
) {
	let Ok((grabber_entity, mut grabber_transform, mut grabber)) = grabber_query.get_single_mut() else { return; };

	if input.single().just_released(Action::Click) { 
		if let Some(grabbed) = grabber.0 {
			commands.entity(grabbed)
				.remove::<ImpulseJoint>();
		}
		grabber.0 = None;
		return; 
	}

	// let .. else go brrrr
	if !input.single().just_pressed(Action::Click) { return; }

	let Some(mouse_position) = windows.single().cursor_position() else { return; };

	let Ok((camera_transform,camera)) = inventory_camera.get_single() else {
		warn!("Couldn't find inventory camera!");
		return;
	};

	let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) else { return; };

	let filter = QueryFilter::only_dynamic().exclude_sensors().groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	let Some((entity,distance)) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter) else { return; };

	let i_entity;

	if let Ok(entity) = ingridient_query.get(entity) {
		i_entity = entity;
	} else if let Some(entity) = parent_query.iter_ancestors(entity).find_map(|parent| ingridient_query.get(parent).ok()) { 
		i_entity = entity;
	} else {
		return;
	}

	let entity = i_entity;

	let Ok(grabbed_entity_transform) = transform_query.get(entity) else { return; };
	
	let hit_global_pos = ray.get_point(distance);
	let srt = grabbed_entity_transform.to_scale_rotation_translation();
	let hit_local_pos = grabbed_entity_transform.compute_matrix().inverse().mul_vec4(hit_global_pos.extend(1.0)).xyz() * srt.0;
	
	grabber_transform.translation = hit_global_pos;

	grabber.0 = Some(entity);

	let joint = SphericalJointBuilder::new()
    	.local_anchor1(Vec3::ZERO)
    	.local_anchor2(hit_local_pos);

	commands.entity(entity)
		.insert(ImpulseJoint::new(grabber_entity,joint));
	
}
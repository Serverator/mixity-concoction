use crate::prelude::*;
use bevy::{core_pipeline::fxaa::Fxaa, math::Vec3Swizzles, render::view::RenderLayers};

use super::effects::{Ingridient, ActiveEffects};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(
				spawn_player
					.in_schedule(OnEnter(GameState::InGame))
			)
			.add_systems(
				(
					camera_follow,
					move_player,
					pickup_entity,
				)
					.in_set(OnUpdate(GameState::InGame))
			);
	}
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Player;

#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerCamera;

#[derive(Component, Clone, Debug, Default)]
pub struct Inventory(Vec<Entity>);



pub fn spawn_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// Spawn player
	commands.spawn((
		Name::new("Player"),
		Player,
		Inventory::default(),
		ActiveEffects::default(),
		PbrBundle {
			mesh: meshes.add(Mesh::from(
				shape::Capsule 
				{ 
					radius: 0.5, 
					depth: 1.3,
					..default() 
				}
			)),
			material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform: Transform::from_xyz(0.0, 0.9, 0.0),
			..default()
		}, 
		// Inputs
		crate::game::input::default_inputs(),

		// Rapier physics components
		RigidBody::KinematicVelocityBased,
		KinematicCharacterController {
			filter_groups: Some(CollisionGroups::new(Group::GROUP_1,Group::GROUP_1)),
			..default()
		},
		Collider::capsule(-Vec3::Y * 0.4, Vec3::Y * 0.4, 0.5),
		Ccd::enabled(),
		Velocity::default(),
		Friction::new(0.0),
		Restitution::new(0.0),
	)).with_children(|command| {
		// Camera
		command.spawn((
			PlayerCamera,
			Camera3dBundle {
				camera: Camera {
					hdr: false,
					..default()
				},
				..default()
			},
			Fxaa::default(),
		));
	});
}

#[derive(Clone, Copy)]
pub struct CameraDistance(f32);
impl Default for CameraDistance {
    fn default() -> Self {
        Self(0.5)
    }
}

pub fn move_player(
	mut player_query: Query<(&mut KinematicCharacterController, &ActionState<Action>), With<Player>>,
	cam_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
	time: Res<Time>,
) {
	let (Ok((mut controller, input)), Ok(camera_transform)) = (player_query.get_single_mut(),cam_query.get_single()) else {
		return;
	};

	if let Some(movement_input) = input.axis_pair(Action::Move) {
		let mut movement_input = movement_input.xy().normalize_or_zero();
		movement_input.y *= -1.0;

		let (y_rot,_,_) = camera_transform.rotation.to_euler(EulerRot::YXZ);

		controller.translation = Some(Quat::from_rotation_y(y_rot) * movement_input.extend(0.0).xzy() * time.delta_seconds() * 9.0); 
	};

}

#[allow(clippy::type_complexity)]
pub fn pickup_entity(
	mut commands: Commands,
	mut player_query: Query<(&Transform, &ActionState<Action>, &mut Inventory), With<Player>>,
	mut ingridient_query: Query<(Entity, &mut Transform), (With<Ingridient>,Without<Player>)>,
	finder_query: Query<&Name>,
	child_query: Query<&Children>,
) {
	let Ok((player_transform, input, mut inventory)) = player_query.get_single_mut() else {
		return;
	};

	if input.just_pressed(Action::Use) {
		let closest_ingridient = &ingridient_query.iter().filter_map(|(entity,ingr_transform)| {
			let distance_sq = (ingr_transform.translation - player_transform.translation).length_squared();
			if distance_sq < 5.0 {
				Some((distance_sq, entity))
			} else {
				None
			}
		}).min_by(|(dist_a,_),(dist_b,_)| dist_a.total_cmp(dist_b));

		if let &Some((_,ingr_entity)) = closest_ingridient {
			inventory.0.push(ingr_entity);

			let mut transform = ingridient_query.get_mut(ingr_entity).unwrap().1;

			transform.translation = Vec3::new(0.0,0.0,0.0);

			dbg!(finder_query.get(ingr_entity).unwrap());

			let mut entity = commands.entity(ingr_entity);

			entity.remove::<Collider>();

			entity.remove_parent();

			entity.insert((
				RigidBody::Dynamic,
				Sleeping {
					sleeping: false,
					..default()
				},
				Velocity::default(),
				Collider::ball(0.5),
				ColliderMassProperties::MassProperties(
					MassProperties {
						mass: 2.0,
						..default()
					}
				),
				RenderLayers::layer(2),
				CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
				SolverGroups::new(Group::GROUP_2,Group::GROUP_2),
			));

			for child in child_query.iter_descendants(entity.id()) {
				let mut child = commands.entity(child);

				child.insert((
					RenderLayers::layer(2),
				));
			}

		}
	}
}

pub fn camera_follow(
	mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
	player_query: Query<(Entity, &Transform, &ActionState<Action>), With<Player>>,
	mut distance: Local<CameraDistance>,
	#[cfg(debug_assertions)]
    mut gui: Query<&mut bevy_inspector_egui::bevy_egui::EguiContext>,
) {
	let (Ok(mut camera_transform), Ok((_player, _player_transform, input))) = (cam_query.get_single_mut(), player_query.get_single()) else {
		warn!("Couldn't find player camera or player from query!");
		return;
	};

	// Camera "zoom"
	#[cfg(debug_assertions)]
	if !gui.single_mut().get_mut().is_pointer_over_area() {
		if let Some(wheel_delta) = input.axis_pair(Action::Zoom) {
			const WHEEL_SENSITIVITY: f32 = 1.0 / 15.0;
			distance.0 = (distance.0 - wheel_delta.y() * WHEEL_SENSITIVITY ).clamp(0.0, 1.0);
		}
	}

	#[cfg(not(debug_assertions))]
	if let Some(wheel_delta) = input.axis_pair(Action::Zoom) {
		const WHEEL_SENSITIVITY: f32 = 1.0 / 15.0;
		distance.0 = (distance.0 - wheel_delta.y() * WHEEL_SENSITIVITY ).clamp(0.0, 1.0);
	}
	
	let (mut y_rot, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

	// Move camera with mouse motion
	if input.pressed(Action::ActivateLook) {
		if let Some(mouse_delta) = input.axis_pair(Action::Look) {
			y_rot += mouse_delta.x() * -0.005;
		}
	}

	camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, y_rot, lerp(-PI/6.2,-PI/4.0,distance.0.powi(2)), 0.0);
	camera_transform.translation = camera_transform.back() * lerp(8.0, 30.0, distance.0.powi(2));

	fn lerp(from: f32, to: f32, t: f32) -> f32 {
		t * to + from * (1.0 - t)
	}
}


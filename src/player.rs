use std::f32::consts::PI;
use bevy_rapier3d::prelude::*;
use bevy::{prelude::*, core_pipeline::fxaa::Fxaa};
use leafwing_input_manager::prelude::*;

use crate::{window::CursorMode, input::Action};


pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(spawn_player)
			.add_system(camera_follow)
			.add_system(move_player);
	}
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Player;

#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerCamera;

pub fn spawn_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// Spawn player
	commands.spawn((
		Name::new("Player"),
		Player,
		PbrBundle {
			mesh: meshes.add(Mesh::from(
				shape::Capsule 
				{ 
					radius: 0.6, 
					depth: 1.8,
					..default() 
				}
			)),
			material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform: Transform::from_xyz(0.0, 1.8, 0.0),
			..default()
		}, 
		InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
				.insert(KeyCode::W, Action::MoveForward)
				.insert(KeyCode::S, Action::MoveBack)
				.insert(KeyCode::A, Action::MoveLeft)
				.insert(KeyCode::D, Action::MoveRight)
				.insert(KeyCode::E, Action::Use)
                .insert(GamepadButtonType::RightTrigger2, Action::Use)
				.insert(DualAxis::right_stick(), Action::Look)
				.insert(DualAxis::mouse_motion(), Action::Look)
				.insert(DualAxis::mouse_wheel(), Action::Zoom)
                .build(),
        },
		// Rapier physics components
		RigidBody::KinematicVelocityBased,
		KinematicCharacterController::default(),
		Collider::capsule(-Vec3::Y * 0.9, Vec3::Y * 0.9, 0.6),
		Ccd::enabled(),
		Velocity::default(),
		//Sleeping::disabled(),
		//Friction::new(0.0),
		//Restitution::new(0.0),
		//GravityScale(5.0),
		//ColliderMassProperties::Density(1.0),
	)).with_children(|parent| {
		// Camera
		parent.spawn((
			PlayerCamera,
			Camera3dBundle::default(),
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
) {
	let Ok((mut controller,input)) = player_query.get_single_mut() else {
		return;
	};

	if let Some(movement_input) = input.axis_pair(Action::Move) {
		dbg!(movement_input);
	}
}

pub fn camera_follow(
	mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
	player_query: Query<(Entity, &Transform, &ActionState<Action>), With<Player>>,
	//mut mouse_input: EventReader<MouseMotion>,
	//mut scroll_input: EventReader<MouseWheel>,
	mut distance: Local<CameraDistance>,
	rapier_context: Res<RapierContext>,
	cursor_mode: Res<CursorMode>,
) {
	let (Ok(mut camera_transform), Ok((player, player_transform, input))) = (cam_query.get_single_mut(), player_query.get_single()) else {
		warn!("Couldn't find player camera or player from query!");
		return;
	};

	// Only move camera if cursor is locked
	if cursor_mode.locked() {
		// Move camera with mouse motion
		if input.pressed(Action::Look) {
			let mouse_delta = input.axis_pair(Action::Look).unwrap();
			let (mut y_rot, mut x_rot, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
			y_rot += mouse_delta.x() * -0.005;
			x_rot = (mouse_delta.y() * -0.005 + x_rot).clamp(-PI/2.0 + 0.01, PI/8.0);
			camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, y_rot, x_rot, 0.0);
		}

		// Camera "zoom"
		let wheel_delta = input.value(Action::Zoom);//.iter().fold(0.0, |acc, motion| acc - motion.y) * 0.05;
		distance.0 = (distance.0 + wheel_delta).clamp(0.0, 1.0);
	}

	let filter = QueryFilter { flags: QueryFilterFlags::EXCLUDE_DYNAMIC, exclude_collider: Some(player), ..default() };

	let mut distance = lerp(6.0, 20.0, distance.0.powi(2));

	if let Some(hit) = rapier_context.cast_ray(player_transform.translation, camera_transform.back(), distance + 2.0, false, filter) {
		distance = (hit.1 * 0.95).min(distance);
	}

	camera_transform.translation = camera_transform.back() * distance;

	fn lerp(from: f32, to: f32, t: f32) -> f32 {
		t * to + from * (1.0 - t)
	}
}


use crate::prelude::*;
use bevy::{core_pipeline::fxaa::Fxaa, math::Vec3Swizzles};

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
				)
					.in_set(OnUpdate(GameState::InGame))
			);
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
			transform: Transform::from_xyz(0.0, 1.5, 0.0),
			..default()
		}, 
		// Inputs
		crate::game::input::default_inputs(),

		// Rapier physics components
		RigidBody::KinematicVelocityBased,
		KinematicCharacterController::default(),
		Collider::capsule(-Vec3::Y * 0.9, Vec3::Y * 0.9, 0.6),
		Ccd::enabled(),
		Velocity::default(),
		Friction::new(0.0),
		Restitution::new(0.0),
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
	mut player_query: Query<(&Transform, &mut KinematicCharacterController, &ActionState<Action>), With<Player>>,
	cam_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
	time: Res<Time>,
	#[cfg(debug_assertions)]
	mut lines: ResMut<DebugLines>,
) {
	let (Ok((_transform, mut controller,input)), Ok(camera_transform)) = (player_query.get_single_mut(),cam_query.get_single()) else {
		return;
	};

	if let Some(movement_input) = input.axis_pair(Action::Move) {
		let mut movement_input = movement_input.xy().normalize_or_zero();
		movement_input.y *= -1.0;

		let (y_rot,_,_) = camera_transform.rotation.to_euler(EulerRot::YXZ);

		#[cfg(debug_assertions)]
		lines.line(_transform.translation, _transform.translation + Quat::from_rotation_y(y_rot) * movement_input.extend(0.0).xzy() * 2.0, 0.0);
		
		controller.translation = Some(Quat::from_rotation_y(y_rot) * movement_input.extend(0.0).xzy() * time.delta_seconds() * 9.0); 
	};

}

pub fn camera_follow(
	mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
	player_query: Query<(Entity, &Transform, &ActionState<Action>), With<Player>>,
	//mut mouse_input: EventReader<MouseMotion>,
	//mut scroll_input: EventReader<MouseWheel>,
	mut distance: Local<CameraDistance>,
) {
	let (Ok(mut camera_transform), Ok((_player, _player_transform, input))) = (cam_query.get_single_mut(), player_query.get_single()) else {
		warn!("Couldn't find player camera or player from query!");
		return;
	};

	// Camera "zoom"
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
	camera_transform.translation = camera_transform.back() * lerp(15.0, 50.0, distance.0.powi(2));

	fn lerp(from: f32, to: f32, t: f32) -> f32 {
		t * to + from * (1.0 - t)
	}
}


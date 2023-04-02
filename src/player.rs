use std::f32::consts::PI;

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseWheel}};

use crate::world::WorldCamera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(spawn_player)
			.add_system(camera_follow);
	}
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Player;

pub fn spawn_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// Spawn player
	commands.spawn((
		Player,
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform: Transform::from_xyz(0.0, 0.5, 0.0),
			..default()
		},
		//VisibilityBundle::default(),
		//TransformBundle::default(),
	));
}

#[derive(Clone, Copy)]
pub struct CameraDistance(f32);
impl Default for CameraDistance {
    fn default() -> Self {
        Self(0.5)
    }
}

pub fn camera_follow(
	mut cam_query: Query<&mut Transform, (With<WorldCamera>, Without<Player>)>,
	player_query: Query<&Transform, (With<Player>, Without<WorldCamera>)>,
	mut mouse_input: EventReader<MouseMotion>,
	mut scroll_input: EventReader<MouseWheel>,
	mut distance: Local<CameraDistance>,
) {
	let (Ok(player),Ok(mut camera)) = (player_query.get_single(), cam_query.get_single_mut()) else {
		warn!("Couldn't find single player or world from query!");
		return;
	};

	let mouse_delta = mouse_input.iter().fold(Vec2::ZERO, |acc, motion| acc - motion.delta) * 0.02;
	let (mut y_rot, mut x_rot, _) = camera.rotation.to_euler(EulerRot::YXZ);
	y_rot += mouse_delta.x;
	x_rot = (mouse_delta.y + x_rot).clamp(-PI/2.0 + 0.01, 0.0);
	camera.rotation = Quat::from_euler(EulerRot::YXZ, y_rot, x_rot, 0.0);

	let wheel_delta = scroll_input.iter().fold(0.0, |acc, motion| acc - motion.y) * 0.07;
	distance.0 = (distance.0 + wheel_delta).clamp(0.0, 1.0);

	camera.translation = player.translation + camera.back() * lerp(2.0, 20.0, distance.0 * distance.0);

	fn lerp(from: f32, to: f32, t: f32) -> f32 {
		t * to + from * (1.0 - t)
	}
}
use std::f32::consts::PI;

use bevy::{prelude::*, core_pipeline::fxaa::Fxaa};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(init_world);
	}
}

#[derive(Component, Clone, Copy, Debug)]
pub struct WorldCamera;

fn init_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// Camera
	commands.spawn((
		WorldCamera,
		Camera3dBundle {
			transform: Transform::from_xyz(0.0, 8.0, 0.0).with_rotation(Quat::from_rotation_x(-PI/6.0)),
			..default()
		},
		Fxaa {
			enabled: true,
			..default()
		},
	));

	// Plane
	commands.spawn((
		Name::new("Main Plane"),
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0, subdivisions: 1 })),
			material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
			..default()
		},
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
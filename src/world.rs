use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(init_world);
	}
}

fn init_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let plane_mesh = Mesh::from(shape::Plane { size: 100.0, subdivisions: 1 });
	// Plane
	commands.spawn((
		Name::new("Main Plane"),
		Collider::from_bevy_mesh(&plane_mesh, &ComputedColliderShape::TriMesh).unwrap(),
		PbrBundle {
			mesh: meshes.add(plane_mesh),
			material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
			..default()
		},
		RigidBody::Fixed,
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
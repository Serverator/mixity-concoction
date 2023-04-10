use crate::{prelude::*, assets::SHADOW_BUNDLE};
use bevy::{core_pipeline::fxaa::Fxaa, math::Vec3Swizzles, gltf::Gltf};

use super::{effects::ActiveEffects, backpack::Inventory, world::Shadow, items::DroppedItem};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(
				spawn_player
					.in_schedule(OnEnter(GameState::InGame))
			)
			.add_systems((
				move_player,
				camera_follow,
					).chain().in_set(OnUpdate(GameState::InGame))
			)
			.register_type::<Inventory>()
			.register_type::<ActiveEffects>();
	}
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Player;

#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerCamera;

pub fn spawn_player(
	mut commands: Commands,
	game_assets: Res<GameAssets>,
	gltfs: Res<Assets<Gltf>>,
) {
	// Spawn player
	commands.spawn((
		Name::new("Player"),
		Player,
		Inventory::default(),
		ActiveEffects::default(),
		SceneBundle {
			scene: game_assets.player_scene.clone(),
			transform: Transform::from_scale(Vec3::splat(1.2)),
			..default()
		},
		NamedMaterials(smallvec![
			NamedMaterial::new("Skin",Color::rgb(0.988, 0.812, 0.718)),
			NamedMaterial::new("Jacket",Color::rgb(0.612, 0.294, 0.188)),
			NamedMaterial::new("Glass",Color::rgb(0.902, 0.996, 1.0)),
			NamedMaterial::new("Metal",Color::rgb(0.988, 0.788, 0.333)),
			NamedMaterial::new("Leather",Color::rgb(0.612, 0.294, 0.188)),
			NamedMaterial::new("Shirt",Color::WHITE),
			NamedMaterial::new("Hair",Color::rgb(0.329, 0.204, 0.141)),
		]),
		crate::game::input::default_inputs(),

		
		( // Rapier physics components
			RigidBody::KinematicPositionBased,
			CollisionGroups::new(Group::GROUP_1,Group::GROUP_1),
			KinematicCharacterController {
				filter_groups: Some(CollisionGroups::new(Group::GROUP_1, Group::GROUP_1)),
				apply_impulse_to_dynamic_bodies: true,
				snap_to_ground: Some(CharacterLength::Relative(1.0)),
				..default()
			},
			Dominance::group(10),
			Collider::compound(vec![(Vec3::Y*0.43, Quat::IDENTITY, Collider::capsule(Vec3::ZERO, Vec3::Y * 1.0, 0.4))]),
			Ccd::enabled(),
			Velocity::default(),
			Friction::new(0.0),
			Restitution::new(0.0),
		)	
	)).with_children(|commands| {
		// Backpack 
		commands.spawn((
			Name::new("Backpack"),
			SceneBundle {
				scene: gltfs.get(&game_assets.backpack_gltf).unwrap().scenes[0].clone(),
				transform: Transform::from_xyz(0.0, 0.65, -0.3).with_rotation(Quat::from_rotation_y(-PI)).with_scale(Vec3::splat(0.8)),
				..default()
			},
			NamedMaterials::backpack(),
		));

		commands.spawn((
			Name::new("Shadow"),
			Shadow,
			SHADOW_BUNDLE.get().unwrap().clone(),
			Transform::from_xyz(0.0,0.02,0.0).with_scale(Vec3::splat(0.6)),
			GlobalTransform::default(),
			VisibilityBundle::default(),
		));

	});

	// Camera
	commands.spawn((
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
}

#[derive(Clone, Copy)]
pub struct CameraDistance(f32);
impl Default for CameraDistance {
    fn default() -> Self {
        Self(0.5)
    }
}

pub fn move_player(
	mut player_query: Query<(&mut KinematicCharacterController, &ActionState<Action>, &mut Transform), With<Player>>,
	cam_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
	mut desired_rotation: Local<Quat>,
	mut current_translation: Local<Vec2>,
	mut desired_translation: Local<Vec2>,
	time: Res<Time>,
) {
	const BASE_PLAYER_SPEED: f32 = 10.0;

	let (Ok((mut controller, input, mut transform)), Ok(camera_transform)) = (player_query.get_single_mut(),cam_query.get_single()) else {
		return;
	};

	if let Some(movement_input) = input.axis_pair(Action::Move) {
		let mut movement_input = movement_input.xy().normalize_or_zero();
		movement_input.y *= -1.0;

		let (y_rot,_,_) = camera_transform.rotation.to_euler(EulerRot::YXZ);

		//velocity.linvel = Quat::from_rotation_y(y_rot) * movement_input.extend(0.0).xzy();
		let desired_direction = Quat::from_rotation_y(y_rot) * movement_input.extend(0.0).xzy();

		if desired_direction.length_squared() != 0.0 {
			*desired_rotation = Quat::from_rotation_arc(Vec3::Z, desired_direction);
		}

		*desired_translation = desired_direction.xz();
		//controller.translation = Some(desired_direction * time.delta_seconds() * 9.0 + Vec3::new(0.0,-0.2,0.0)); 
	} else {
		*desired_translation = Vec2::ZERO; 
	};

	transform.rotation = Quat::slerp(transform.rotation, *desired_rotation, (1.0 - 0.0000001f64.powf(time.delta_seconds_f64())) as f32);
	*current_translation = Vec2::lerp(*current_translation, *desired_translation, (1.0 - 0.0000001f64.powf(time.delta_seconds_f64())) as f32);
	let existing_translation = controller.translation.unwrap_or_default();
	controller.translation = Some(existing_translation + current_translation.extend(-0.2).xzy() * BASE_PLAYER_SPEED * time.delta_seconds());
}

pub fn camera_follow(
	mut cam_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
	player_query: Query<(Entity, &Transform, &ActionState<Action>), With<Player>>,
	mut distance: Local<CameraDistance>,
	mut looking_pos: Local<Vec3>,
	#[cfg(debug_assertions)]
    mut gui: Query<&mut bevy_inspector_egui::bevy_egui::EguiContext>,
	time: Res<Time>,
) {

	let (Ok(mut camera_transform), Ok((_player, player_transform, input))) = (cam_query.get_single_mut(), player_query.get_single()) else {
		warn!("Couldn't find player camera or player from query!");
		return;
	};

	let (mut y_rot, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

	*looking_pos = Vec3::lerp(*looking_pos,player_transform.translation,1.0 - 0.01f32.powf(time.delta_seconds()));

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
	
	

	// Move camera with mouse motion
	if input.pressed(Action::ActivateLook) {
		if let Some(mouse_delta) = input.axis_pair(Action::Look) {
			y_rot += mouse_delta.x() * -0.005;
		}
	}

	camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, y_rot, lerp(-PI/7.5,-PI/4.5,distance.0.powi(2)), 0.0);
	camera_transform.translation = *looking_pos + Quat::from_rotation_y(y_rot) * Vec3::Z * 4.0 + camera_transform.back() * lerp(7.0, 45.0, distance.0.powi(2)) + Vec3::Y * lerp(1.5, 0.0, distance.0.powi(2));

	fn lerp(from: f32, to: f32, t: f32) -> f32 {
		t * to + from * (1.0 - t)
	}
}


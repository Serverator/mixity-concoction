use bevy::input::mouse::MouseMotion;

use crate::prelude::*;

pub struct DebugCameraPlugin;
impl Plugin for DebugCameraPlugin {
    fn build(&self, app: &mut App) {
        app
			.init_resource::<DebugCameraState>()
			.add_systems(PostUpdate, (take_control, move_camera).chain());
    }
}

#[derive(Resource, Default)]
pub struct DebugCameraState {
	pub enabled: bool,
	pub selected_id: usize,
	pub camera: Option<DebugCamera>,
}

impl DebugCameraState {
	pub fn allow_move(&self, camera: Entity) -> bool {
		self.camera.as_ref().map(|c| matches!(c.cam_mode, DebugCameraMode::LookThrough) || c.camera != camera).unwrap_or(true)
	}

	fn set_camera(&mut self, new_cam: Option<Entity>) {
		if let Some(desired_cam) = new_cam {
			if self.camera.as_ref().map(|c| c.camera != desired_cam).unwrap_or(true) {
				self.camera = Some( DebugCamera {
					camera: desired_cam,
					cam_mode: DebugCameraMode::LookThrough,
				});
			}
		} else {
			self.camera = None;
		}
	}
}

pub struct DebugCamera {
	pub camera: Entity,
	pub cam_mode: DebugCameraMode,
}

pub enum DebugCameraMode {
	LookThrough,
	FreeFlight(Transform),
	FollowEntity(Entity),
}

fn take_control(
	camera_q: Query<Entity, With<Camera>>,
	mut cam_state: ResMut<DebugCameraState>,
	input: Res<Input<KeyCode>>,
) {
	let cam_count = camera_q.iter().count();

	cam_state.enabled ^= input.just_pressed(KeyCode::F12) && (!input.pressed(KeyCode::ShiftLeft) || !cam_state.enabled);

	if cam_count == 0 || !cam_state.enabled {
		cam_state.set_camera(None);
		return;
	}

	if cam_state.selected_id > 0 && input.just_pressed(KeyCode::PageDown) {
		cam_state.selected_id -= 1;
	} 
	
	if input.just_pressed(KeyCode::PageUp) {
		cam_state.selected_id += 1;
	}

	cam_state.selected_id = cam_state.selected_id.min(cam_count - 1);

	let id = cam_state.selected_id;

	cam_state.set_camera(camera_q.iter().nth(id));
}

fn move_camera(
	mut camera_q: Query<(&mut Transform, Option<&Parent>), With<Camera>>,
	global_q: Query<&GlobalTransform>,
	mut cam_state: ResMut<DebugCameraState>,
	mut motion_evr: EventReader<MouseMotion>,
	input: Res<Input<KeyCode>>,
	time: Res<Time>,
) {
	let Some(DebugCamera { camera, cam_mode }) = &mut cam_state.camera else { return; };
	
	match cam_mode {
    	DebugCameraMode::LookThrough => {
			if input.pressed(KeyCode::ShiftLeft) && input.just_pressed(KeyCode::F12) {
				let Ok(global) = global_q.get(*camera) else { return; };

				*cam_mode = DebugCameraMode::FreeFlight(global.compute_transform());
			}
		},
    	DebugCameraMode::FreeFlight(transform) => {

			const MOUSE_SENSITIVITY: f32 = 0.005;
			const CAMERA_SPEED: f32 = 9.0;

			let Ok((mut cam_transform, parent)) = camera_q.get_mut(*camera) else { return; };
			let parent_transform = parent.and_then(|p| global_q.get(p.get()).ok().map(|t| t.compute_matrix().inverse())).unwrap_or_default();

			let mouse_input = motion_evr.iter().fold(Vec2::ZERO, |acc,motion| acc - motion.delta) * MOUSE_SENSITIVITY;
			let (y,x,_) = transform.rotation.to_euler(EulerRot::YXZ);
			transform.rotation = Quat::from_euler(EulerRot::YXZ, y + mouse_input.x, (x + mouse_input.y).clamp(-PI/2.0 + 0.01, PI/2.0 - 0.01), 0.0);

			let mut desired_input = Vec3::ZERO;
			if input.pressed(KeyCode::W) { desired_input += Vec3::NEG_Z }
			if input.pressed(KeyCode::S) { desired_input += Vec3::Z }
			if input.pressed(KeyCode::A) { desired_input += Vec3::NEG_X }
			if input.pressed(KeyCode::D) { desired_input += Vec3::X }
			if input.pressed(KeyCode::Space) { desired_input += Vec3::Y }
			if input.pressed(KeyCode::ShiftLeft) { desired_input += Vec3::NEG_Y }

			let (y,_,_) = transform.rotation.to_euler(EulerRot::YXZ);
			transform.translation += Quat::from_rotation_y(y) * desired_input.normalize_or_zero() * time.delta_seconds() * CAMERA_SPEED * if input.pressed(KeyCode::ControlLeft) { 2.0 } else { 1.0 };

			*cam_transform = Transform::from_matrix(parent_transform * transform.compute_matrix());
		},
    	DebugCameraMode::FollowEntity(_) => todo!(),
	}
}

use crate::prelude::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(InputManagerPlugin::<Action>::default())
			.add_system(cursor_grab_system);
	}
}

pub fn default_inputs() -> InputManagerBundle::<Action> {
	InputManagerBundle::<Action> {
		action_state: ActionState::default(),
		input_map: InputMap::default()
			.insert(DualAxis::left_stick(), Action::Move)
			.insert(VirtualDPad::wasd(), Action::Move)
			.insert(KeyCode::E, Action::Use)
			.insert(GamepadButtonType::RightTrigger2, Action::Use)
			.insert(DualAxis::right_stick(), Action::Look)
			.insert(DualAxis::mouse_motion(), Action::Look)
			.insert(MouseButton::Right, Action::ActivateLook)
			.insert(DualAxis::mouse_wheel(), Action::Zoom)
			.build(),
	}
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    Move,
	Use,
	Look,
	ActivateLook,
	Zoom,
}

pub fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    //mouse: Res<Input<MouseButton>>,
	input: Query<&ActionState<Action>>,
    //key: Res<Input<KeyCode>>,
	//mut cursor_mode: ResMut<CursorMode>,
	#[cfg(debug_assertions)]
    mut gui: Query<&mut bevy_inspector_egui::bevy_egui::EguiContext>,
) {
    let mut window = windows.single_mut();

	let Ok(input) = input.get_single() else {
		return;
	};

	#[cfg(debug_assertions)]
	{
		let mut gui = gui.single_mut();
		if input.just_pressed(Action::ActivateLook) && !gui.get_mut().is_pointer_over_area() {
			window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
			window.cursor.visible = false;
		}
	}

	#[cfg(not(debug_assertions))]
    if input.just_pressed(Action::ActivateLook) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if input.just_released(Action::ActivateLook) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
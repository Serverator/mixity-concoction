use std::f32::consts::SQRT_2;

use bevy::{prelude::*};
use leafwing_input_manager::{prelude::*, axislike::DualAxisData, systems::update_action_state, plugin::InputManagerSystem};

pub struct InputPlugin;
impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(InputManagerPlugin::<Action>::default())
			.add_system(mock_input.in_set(InputManagerSystem::ManualControl));
	}
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    Move,
	MoveForward,
	MoveBack,
	MoveLeft,
	MoveRight,
	Use,
	Look,
	Zoom,
}

// I wish I could bind DualAxis to actual keyboard buttons to stop this...
fn mock_input(
	mut input_query: Query<&mut ActionState<Action>>
) {
	use Action::*;

	for mut input in &mut input_query {

		let mut keyboard_input = Vec2 {
			y: match (input.pressed(MoveForward),input.pressed(MoveBack)) {
				(true,false) => 1.0,
				(false,true) => -1.0,
				_ => 0.0 },
			x: match (input.pressed(MoveLeft),input.pressed(MoveRight)) {
				(true,false) => -1.0,
				(false,true) => 1.0,
				_ => 0.0 },
		};

		let length = keyboard_input.length_squared();
		if length != 0.0 {
			if length > 1.0 {
				keyboard_input /= SQRT_2
			}
			let move_data = input.action_data_mut(Move);
			move_data.axis_pair = Some(DualAxisData::from_xy(keyboard_input));
		}
	}
}
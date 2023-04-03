use crate::prelude::*;

pub mod player;
pub mod world;
pub mod physics;
pub mod input;
pub mod materials;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(player::PlayerPlugin)
			.add_plugin(world::WorldPlugin)
			.add_plugin(physics::PhysicsPlugin)
			.add_plugin(input::InputPlugin)
			.add_plugin(materials::CustomMaterialPlugin);
	}
}
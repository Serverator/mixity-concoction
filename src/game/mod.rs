use crate::prelude::*;

use self::ingredient::Ingredient;

pub mod alchemy;
pub mod backpack;
pub mod effects;
pub mod ingredient;
pub mod input;
pub mod items;
pub mod materials;
pub mod physics;
pub mod player;
pub mod world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_plugin(player::PlayerPlugin)
			.add_plugin(world::WorldPlugin)
			.add_plugin(physics::PhysicsPlugin)
			.add_plugin(input::InputPlugin)
			.add_plugin(materials::CustomMaterialPlugin)
			.add_plugin(backpack::BackpackPlugin)
			.add_plugin(alchemy::AlchemyPlugin)
			.add_plugin(items::ItemsPlugin)
			.add_plugin(effects::EffectsPlugin)
			.register_type::<Ingredient>();
	}
}

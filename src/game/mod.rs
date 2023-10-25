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
		app .add_plugins((
				player::PlayerPlugin,
				world::WorldPlugin,
				physics::PhysicsPlugin,
				input::InputPlugin,
				materials::CustomMaterialPlugin,
				backpack::BackpackPlugin,
				alchemy::AlchemyPlugin,
				items::ItemsPlugin,
				effects::EffectsPlugin
			))
			.register_type::<Ingredient>();
	}
}

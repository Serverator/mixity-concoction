use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

mod player;
mod world;
mod effects;
mod physics;
mod window;
mod input;

#[cfg(debug_assertions)]
mod debug;

fn main() {
	let mut app = App::default();

	app.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
				.set(window::get_window_plugin()),
		)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(world::WorldPlugin)
		.add_plugin(physics::PhysicsPlugin)
		.add_plugin(window::WindowPlugin)
		.add_plugin(input::InputPlugin);
		
	#[cfg(debug_assertions)]	
	app.add_plugin(debug::DebugPlugin);

	app.run();
}
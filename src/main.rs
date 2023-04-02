use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

mod player;
mod world;
mod effects;

fn main() {
	App::default()
		.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
		)
		.add_plugin(DebugPlugin)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(world::WorldPlugin)
		.run();
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
	#[cfg(debug_assertions)]
	fn build(&self, app: &mut App) {
		use bevy_inspector_egui::quick::WorldInspectorPlugin;

		app.add_plugin(WorldInspectorPlugin::new());
	}

	#[cfg(not(debug_assertions))]
	fn build(&self, app: &mut App) {}
}
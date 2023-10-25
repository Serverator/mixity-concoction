#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::prelude::*;

mod assets;
mod game;
mod helper;
mod main_menu;
mod loading;

#[cfg(debug_assertions)]
mod debug;

fn main() {
	let mut app = App::default();

	app.add_state::<GameState>()
		.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(
					bevy_embedded_assets::EmbeddedAssetPlugin,
				)
				.set(bevy::prelude::WindowPlugin {
					primary_window: Some(Window {
						title: "Mixity Concoction".to_string(),
						//present_mode: bevy::window::PresentMode::Immediate,
						..default()
					}),
					..default()
				}),
		)
		.add_plugins((
			game::GamePlugin,
			loading::LoadingPlugin,
			assets::AssetLoadingPlugin
		));

	#[cfg(debug_assertions)]
	app.add_plugins(debug::DebugPlugin);

	app.run();
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum GameState {
	#[default]
	LoadingAssets,
	MainMenu,
	InGame,
	GeneratingWorld,
}

pub mod prelude {
	pub use bevy::prelude::*;
	pub use bevy_rapier3d::prelude::*;
	pub use leafwing_input_manager::prelude::*;
	pub use rand::prelude::*;
	pub use smallvec::*;
	pub use std::f32::consts::PI;

	pub use crate::assets::GameAssets;
	pub use crate::game::input::*;
	pub use crate::game::materials::*;
	pub use crate::helper::*;
	pub use crate::GameState;

	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::prelude::*;
	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::quick::WorldInspectorPlugin;
}

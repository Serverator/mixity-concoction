use crate::prelude::*;

mod player;
mod world;
//mod effects;
mod physics;
mod input;
mod materials;

#[cfg(debug_assertions)]
mod debug;

fn main() {
	let mut app = App::default();

	app.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(bevy_embedded_assets::EmbeddedAssetPlugin)
				.set(bevy::prelude::WindowPlugin {
					primary_window: 
						Some(Window {
							title: "Project Concoction".to_string(),
							present_mode: bevy::window::PresentMode::Immediate,
							..default()
					}),
					..default()
				}),
		)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(world::WorldPlugin)
		.add_plugin(physics::PhysicsPlugin)
		.add_plugin(input::InputPlugin)
		.add_plugin(materials::CustomMaterialPlugin);
		
	#[cfg(debug_assertions)]	
	app.add_plugin(debug::DebugPlugin);

	app.run();
}

pub mod prelude {
	pub use bevy::prelude::*;
	pub use bevy_rapier3d::prelude::*;
	pub use leafwing_input_manager::prelude::*;
	pub use std::f32::consts::PI;
	pub use rand::prelude::*;

	pub use crate::input::*;
	pub use crate::materials::*;

	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::quick::WorldInspectorPlugin;
	#[cfg(debug_assertions)]
	pub use bevy_inspector_egui::prelude::*;
	#[cfg(debug_assertions)]
	pub use bevy_prototype_debug_lines::*;
}
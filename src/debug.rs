/// This mod exists only in debug mode

use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		use bevy_inspector_egui::quick::WorldInspectorPlugin;
		app.add_plugin(WorldInspectorPlugin::new());
	}
}
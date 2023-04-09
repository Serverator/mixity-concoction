//use bevy_inspector_egui::quick::StateInspectorPlugin;

/// This mod exists only in debug mode
use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(WorldInspectorPlugin::new())
			//.add_plugin(RapierDebugRenderPlugin {
			//	always_on_top: true,
			//	..default()
			//})
			.add_plugin(DebugLinesPlugin::with_depth_test(true));
	}
}
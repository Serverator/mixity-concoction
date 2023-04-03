/// This mod exists only in debug mode
use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(WorldInspectorPlugin::new())
			.add_plugin(DebugLinesPlugin::with_depth_test(true));
	}
}
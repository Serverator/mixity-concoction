/// This mod exists only in debug mode
use crate::prelude::*;

mod debug_camera;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WorldInspectorPlugin::new(),
            debug_camera::DebugCameraPlugin
        ));
    }
}

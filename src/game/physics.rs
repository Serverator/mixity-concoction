use crate::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(configure_physics)
			.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
	}
}


fn configure_physics(mut config: ResMut<RapierConfiguration>) {
	config.gravity = Vec3::NEG_Y * 9.8;
	config.timestep_mode = TimestepMode::Variable {
		max_dt: 1.0 / 10.0,
		time_scale: 1.0,
		substeps: 1,
	};
}
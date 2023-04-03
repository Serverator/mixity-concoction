use bevy::{render::{render_resource::{ShaderRef, AsBindGroup}, once_cell::sync::OnceCell}, reflect::TypeUuid};

use crate::prelude::*;

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(MaterialPlugin::<FoliageMaterial>::default())
			.add_startup_system(init);
	}
}

static DITHER_HANDLE: OnceCell<Handle<Image>> = OnceCell::new();

pub fn init(
	asset_server: Res<AssetServer>,
) {
	DITHER_HANDLE.set(asset_server.load("textures/dither.png")).unwrap();
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "33fbe40a-eff7-4e20-a44f-997397cf2085"]
pub struct FoliageMaterial {
	// /// Progress from 0.0 to 1.0
	#[uniform(0)]
	pub color: Color,
	#[texture(1)]
	#[sampler(2)]
	pub dither_texture: Handle<Image>,
}

impl Default for FoliageMaterial {
    fn default() -> Self {
        Self { 
			color: Default::default(), 
			dither_texture: DITHER_HANDLE.get().unwrap().clone()
		}
    }
}

impl Material for FoliageMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/foliage.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Mask(0.5)
	}

}

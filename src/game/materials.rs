use bevy::{render::{render_resource::{ShaderRef, AsBindGroup, SamplerDescriptor, AddressMode, FilterMode, ShaderType, AsBindGroupShaderType}, once_cell::sync::OnceCell, texture::ImageSampler}, reflect::TypeUuid};

use crate::prelude::*;

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(MaterialPlugin::<FoliageMaterial>::default())
			.add_system(set_dither_texture.in_schedule(OnExit(GameState::LoadingAssets)));
	}
}

static DITHER_HANDLE: OnceCell<Handle<Image>> = OnceCell::new();

pub fn set_dither_texture(
	game_assets: Res<GameAssets>,
	mut image_assets: ResMut<Assets<Image>>,
) {
	let image_mut = image_assets.get_mut(&game_assets.dither_texture).unwrap();

	image_mut.sampler_descriptor = ImageSampler::Descriptor(
		SamplerDescriptor { 
			address_mode_u: AddressMode::Repeat,
			address_mode_v: AddressMode::Repeat,
			address_mode_w: AddressMode::Repeat,
			mag_filter: FilterMode::Nearest,
			..Default::default() 
		}
	);

	DITHER_HANDLE.set(game_assets.dither_texture.clone()).unwrap();
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "33fbe40a-eff7-4e20-a44f-997397cf2085"]
#[uniform(0, FoliageMaterialUniform)]
pub struct FoliageMaterial {
	// /// Progress from 0.0 to 1.0
	pub color: Color,
	pub sss: bool,

	#[texture(1)]
	#[sampler(2)]
	pub dither_texture: Handle<Image>,
}

#[derive(Clone, Default, ShaderType)]
pub struct FoliageMaterialUniform {
	pub color: Vec4,
	pub sss: u32,
}

impl AsBindGroupShaderType<FoliageMaterialUniform> for FoliageMaterial {
    fn as_bind_group_shader_type(&self, _images: &bevy::render::render_asset::RenderAssets<Image>) -> FoliageMaterialUniform {
        FoliageMaterialUniform {
			color: self.color.as_linear_rgba_f32().into(),
			sss: self.sss.into(),
		}
    }
}

impl Default for FoliageMaterial {
    fn default() -> Self {
        Self { 
			color: Default::default(), 
			sss: false,
			dither_texture: DITHER_HANDLE.get().unwrap().clone()
		}
    }
}

impl Material for FoliageMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/foliage.wgsl".into()
	}

	// TODO: Fix :)
	//fn vertex_shader() -> ShaderRef {
	//	"shaders/foliage.wgsl".into()
	//}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Opaque//Mask(0.5)
	}

}
use std::borrow::Cow;

use bevy::{render::{render_resource::{ShaderRef, AsBindGroup, SamplerDescriptor, AddressMode, FilterMode, ShaderType, AsBindGroupShaderType}, once_cell::sync::OnceCell, texture::ImageSampler}, reflect::TypeUuid};

use crate::{prelude::*, assets::{SceneInstanceReady, SpawnableArchetype}, choice};

use super::world::{Shadow};

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(MaterialPlugin::<FoliageMaterial>::default())
			.add_system(set_dither_texture.in_schedule(OnExit(GameState::LoadingAssets)))
			.add_system(replace_materials.in_base_set(CoreSet::PostUpdate))
			.register_type::<FoliageMaterial>()
			.register_type::<NamedMaterials>()
			.register_type::<NamedMaterial>();
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

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Copy, Default, Reflect, FromReflect)]
#[uuid = "33fbe40a-eff7-4e20-a44f-997397cf2085"]
#[uniform(0, FoliageMaterialUniform)]
pub struct FoliageMaterial {
	pub color: Color,
	pub sss: bool,
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

#[derive(Clone, Reflect, FromReflect, Default, Debug)]
pub struct NamedMaterial {
	name: Cow<'static, str>,
	material: FoliageMaterial,
}

impl NamedMaterial {
	pub fn new(name: impl Into<Cow<'static, str>>, color: Color) -> Self {
		NamedMaterial {
    		name: name.into(),
    		material: FoliageMaterial { color, sss: false },
		}
	}
}

#[derive(Clone, Component, Default, Reflect, Debug)]
pub struct NamedMaterials(pub SmallVec<[NamedMaterial;5]>);

impl NamedMaterials {
	pub fn push(&mut self, value: NamedMaterial) {
		self.0.push(value)
	}

	pub fn iter(&self) -> core::slice::Iter<NamedMaterial> {
		self.0.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<NamedMaterial> {
		self.0.iter_mut()
	}

	pub fn backpack() -> Self {
		const LEATHER_COLOR: Color = Color::rgb(0.45,0.2,0.0);
		const STRAP_COLOR: Color = Color::rgb(0.25,0.08,0.0);
		const BUCKLE_COLOR: Color = Color::rgb(0.7,0.3,0.04);

		NamedMaterials( smallvec![
			NamedMaterial::new("Open", LEATHER_COLOR),
			NamedMaterial::new("Closed", LEATHER_COLOR),
			NamedMaterial::new("Inside", STRAP_COLOR),
			NamedMaterial::new("Straps", STRAP_COLOR),
			NamedMaterial::new("Buckle", BUCKLE_COLOR),
		])
	}
}

impl NamedMaterials {
	pub fn generate_materials(archetype: SpawnableArchetype, is_rare: bool, rng: &mut impl Rng) -> (Self, Color) {
		use SpawnableArchetype::*;

		let main_color;
		
		let named_materials = match archetype {
    		Tree | Bush => {
				const TRUNK_COLORS: Choices<Color> = choice![
					Color::rgb(0.5, 0.3, 0.05),
					Color::rgb(0.55, 0.35, 0.05),
					Color::rgb(0.45, 0.25, 0.05)
				];
		
				let trunk_color = if is_rare {
					Color::rgb(0.85, 0.85, 0.9)
				} else {
					*TRUNK_COLORS.random(rng)
				};
		
				const LEAVES_COLORS: Choices<Color> = choice![
					Color::LIME_GREEN,
					Color::YELLOW_GREEN,
					Color::ORANGE,
					Color::ORANGE_RED,
				];
		
				let leaves_color = if is_rare {
					Color::hsl(rng.gen_range(150.0..330.0), rng.gen_range(0.8..1.0), rng.gen_range(0.4..0.6))
				} else {
					*LEAVES_COLORS.random(rng)
				};

				let berry_color = if is_rare {
					Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.8..1.0), rng.gen_range(0.45..0.65))
				} else {
					Color::hsl(rng.gen_range(190.0..360.0), rng.gen_range(0.5..0.65), rng.gen_range(0.35..0.55))
				};


				main_color = berry_color;
		
				NamedMaterials(smallvec![
					NamedMaterial::new("Trunk", trunk_color),
					NamedMaterial::new("Berry", berry_color),
					NamedMaterial { name: Cow::Borrowed("Leaves"), material: FoliageMaterial { color: leaves_color, sss: true } }
				])
			},
    		Mushroom => {

				let cap_color = if is_rare {
					Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.8..1.0), rng.gen_range(0.45..0.65))
				} else {
					Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.5..0.65), rng.gen_range(0.35..0.55))
				};

				let stem_color = if is_rare {
					Color::rgb(0.8, 0.8, 1.0)
				} else {
					Color::rgb(0.7, 0.7, 0.7)
				};

				main_color = cap_color;

				NamedMaterials(smallvec![
					NamedMaterial::new("Cap", cap_color),
					NamedMaterial::new("Stem", stem_color),
				])
			},
		};

		(named_materials, main_color)
	}


}


pub fn replace_materials(
	mut commands: Commands,
	mut material_assets: ResMut<Assets<FoliageMaterial>>,
	name_query: Query<(Entity, &Name, Option<&Handle<FoliageMaterial>>), (Or<(With<Handle<StandardMaterial>>, With<Handle<FoliageMaterial>>)>, Without<Shadow>)>,
	children_query: Query<&Children>,
	spawnable_query: Query<(Entity, &NamedMaterials), (With<SceneInstanceReady>, Or<(Added<SceneInstanceReady>, Changed<NamedMaterials>)>)>,
) {
	// Don't look at this
	// You WILL have a heart attack
	for (parent, material) in &spawnable_query {

		for (child, name, maybe_material) in children_query.iter_descendants(parent).filter_map(|child| name_query.get(child).ok()) {
			let Some(material) = material.0.iter().find(|x| name.contains(&*x.name)).map(|x| x.material) else {
				continue;
			};

			if let Some(Some(child_material)) = maybe_material.map(|handle| material_assets.get_mut(handle)) {
				*child_material = material;
			} else {
				commands.entity(child)
					.remove::<Handle<StandardMaterial>>()
					.insert(material_assets.add(material));
			}
		}
	}
}

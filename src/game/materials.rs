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

#[derive(Clone, Reflect, FromReflect, Default)]
pub struct NamedMaterial {
	name: Cow<'static, str>,
	material: FoliageMaterial,
}

impl NamedMaterial {
	pub fn trunk(is_rare: bool, rng: &mut impl Rng) -> Self {
		const TRUNK_COLORS: Choices<Color> = choice![
			Color::rgb(0.5, 0.3, 0.05),
			Color::rgb(0.55, 0.35, 0.05),
			Color::rgb(0.45, 0.25, 0.05)
		];

		let color = if is_rare {
			Color::rgb(0.85, 0.85, 0.9)
		} else {
			*TRUNK_COLORS.random(rng)
		};

		NamedMaterial {
    		name: Cow::Borrowed("Trunk"),
    		material: FoliageMaterial {
				color,
				..default()
			}
		}
	}

	pub fn leaves(is_rare: bool, rng: &mut impl Rng) -> Self {
		const LEAVES_COLORS: Choices<Color> = choice![
			Color::LIME_GREEN,
			Color::YELLOW_GREEN,
			Color::ORANGE,
			Color::ORANGE_RED,
		];

		let color = if is_rare {
			Color::hsl(rng.gen_range(150.0..330.0), rng.gen_range(0.8..1.0), rng.gen_range(0.4..0.6))
		} else {
			*LEAVES_COLORS.random(rng)
		};

		NamedMaterial {
    		name: Cow::Borrowed("Leaves"),
    		material: FoliageMaterial {
				color,
				sss: true,
			}
		}
	}

	pub fn mushroom_stem(_is_rare: bool, _rng: &mut impl Rng) -> Self {
		NamedMaterial {
    		name: Cow::Borrowed("Stem"),
    		material: FoliageMaterial {
				color: Color::rgb(0.8, 0.8, 0.8),
				sss: true,
			}
		}
	}

	pub fn mushroom_cap(is_rare: bool, rng: &mut impl Rng) -> Self {
		let color = if is_rare {
			Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.8..1.0), rng.gen_range(0.45..0.65))
		} else {
			Color::hsl(rng.gen_range(0.0..360.0), rng.gen_range(0.5..0.65), rng.gen_range(0.35..0.55))
		};

		NamedMaterial {
    		name: Cow::Borrowed("Cap"),
    		material: FoliageMaterial {
				color,
				..default()
			}
		}
	}

}

#[derive(Clone, Component, Default, Reflect)]
pub struct NamedMaterials(pub SmallVec<[NamedMaterial;4]>);

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
}

impl NamedMaterials {
	pub fn generate_materials(archetype: SpawnableArchetype, is_rare: bool, rng: &mut impl Rng) -> Self {
		use SpawnableArchetype::*;

		let mut materials = NamedMaterials::default();

		match archetype {
    		Tree | Bush => {
				materials.push(NamedMaterial::leaves(is_rare, rng));
				materials.push(NamedMaterial::trunk(is_rare, rng));
			},
    		Mushroom => {
				materials.push(NamedMaterial::mushroom_stem(is_rare, rng));
				materials.push(NamedMaterial::mushroom_cap(is_rare, rng));
			},
		}

		materials
	}


}

#[allow(clippy::type_complexity)]
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
